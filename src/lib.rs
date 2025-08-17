use anyhow::Result;
use rig::agent::Agent;
use rig::completion::{Message, Prompt, request::CompletionModel};
use std::io::{self, Write};

pub mod simple_memory;
pub mod tools;

use simple_memory::{SimpleMemory, default_memory_file};

pub struct Vy<M: CompletionModel> {
    agent: Agent<M>,
    conversation_history: Vec<Message>,
    model_id: String,
    api_key: String,
    memory_model_id: String,
}

impl<M: CompletionModel> Vy<M> {
    pub fn new(
        agent: Agent<M>,
        model_id: String,
        api_key: String,
        memory_model_id: String,
    ) -> Self {
        Self {
            agent,
            conversation_history: Vec::new(),
            model_id,
            api_key,
            memory_model_id,
        }
    }

    pub async fn chat(mut self) -> Result<()> {
        self.print_welcome();

        loop {
            // Prompt for user input
            print!("💬 You: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            // Handle special commands
            if self.handle_special_commands(input) {
                break;
            }

            if input.is_empty() {
                continue;
            }

            // Check if it's a special command that doesn't exit
            if self.is_non_exit_command(input) {
                continue;
            }

            // Show thinking indicator
            print!("🤖 Vy: ");
            io::stdout().flush()?;

            // Add user message to history
            self.conversation_history.push(Message::user(input));

            // Get response from agent with error handling
            match self
                .agent
                .prompt(input)
                .multi_turn(5)
                .with_history(&mut self.conversation_history)
                .await
            {
                Ok(response) => {
                    println!("{response}");
                    // Add assistant response to history
                    self.conversation_history
                        .push(Message::assistant(&response));
                }
                Err(e) => {
                    let formatted_error = Self::format_error(&e);
                    if formatted_error.contains("Invalid API key") {
                        println!("\n🚨 {formatted_error}\n");
                    } else {
                        eprintln!("❌ Error: {formatted_error}");
                        println!("💡 Try rephrasing your question or check your configuration.");
                    }
                }
            }

            println!(); // Add spacing between exchanges
        }

        // Analyze conversation for memories before saying goodbye
        self.analyze_conversation_memories().await?;

        self.print_goodbye();
        Ok(())
    }

    pub async fn chat_tui(mut self) -> Result<()> {
        use crossterm::{
            event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use ratatui::{
            backend::CrosstermBackend,
            layout::{Constraint, Direction, Layout, Rect},
            style::{Color, Modifier, Style},
            text::{Line, Span, Text},
            widgets::{Block, Borders, Clear, Paragraph, Wrap},
            Terminal,
        };
        use std::time::Duration;

        // Terminal setup
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        if let Err(e) = execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture) {
            // Best-effort cleanup
            let _ = disable_raw_mode();
            return Err(e.into());
        }
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // UI state
        let mut input = String::new();
        let mut chat_log: Vec<String> = Vec::new();
        let mut scroll: i32 = 0;
        let mut show_help = false;

        chat_log.push(format!(
            "🤖 Vy - {} | Press Enter to send, ? for help, q to quit",
            self.model_id
        ));

        // Main loop
        'outer: loop {
            let draw_result: Result<(), anyhow::Error> = (|| {
                terminal.draw(|f| {
                    let size = f.size();

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(1), // status
                            Constraint::Min(1),     // chat
                            Constraint::Length(3), // input
                        ])
                        .split(size);

                    // Status bar
                    let status = Paragraph::new(Line::from(vec![
                        Span::styled(
                            format!(" Vy - {} ", self.model_id),
                            Style::default().fg(Color::Black).bg(Color::Cyan),
                        ),
                        Span::raw("  ⌨  Enter: Send   ↑/↓: Scroll   PgUp/PgDn: Fast scroll   q: Quit"),
                    ]))
                    .style(Style::default())
                    .block(Block::default().borders(Borders::BOTTOM));
                    f.render_widget(status, chunks[0]);

                    // Chat area
                    let chat_text = Text::from(chat_log.join("\n"));
                    let mut chat = Paragraph::new(chat_text)
                        .wrap(Wrap { trim: false })
                        .block(Block::default().title("Chat").borders(Borders::ALL));

                    // Calculate max vertical scroll based on content height rough estimate
                    // We do a simple line-based scroll; ratatui will wrap long lines visually
                    let max_scroll: i32 = 0.max((chat_log.len() as i32) - (chunks[1].height as i32 - 2));
                    let scroll_top = scroll.clamp(0, max_scroll) as u16;
                    chat = chat.scroll((scroll_top, 0));
                    f.render_widget(chat, chunks[1]);

                    // Input area
                    let input_block = Block::default().title("Input").borders(Borders::ALL);
                    let input_paragraph = Paragraph::new(input.as_str())
                        .style(Style::default())
                        .block(input_block.clone());
                    f.render_widget(input_paragraph, chunks[2]);
                    // Put cursor inside the input box
                    let cursor_x = chunks[2].x + 1 + input.len() as u16;
                    let cursor_y = chunks[2].y + 1;
                    f.set_cursor(cursor_x.min(chunks[2].right() - 1), cursor_y);

                    if show_help {
                        let help_area = centered_rect(70, 60, size);
                        let help = Paragraph::new(Text::from(
                            "Controls:\n\n  Enter: Send message\n  q: Quit (when input empty)\n  Esc: Clear input / Close help\n  ↑/↓: Scroll chat\n  PgUp/PgDn: Faster scroll\n  ?: Toggle help\n\nCommands:\n  help      Show this help\n  history   Show message count\n  clear     Clear chat pane\n  exit|quit End session",
                        ))
                        .block(Block::default().title("Help").borders(Borders::ALL))
                        .wrap(Wrap { trim: true });
                        f.render_widget(Clear, help_area);
                        f.render_widget(help, help_area);
                    }
                })?;
                Ok(())
            })();

            if draw_result.is_err() {
                break 'outer;
            }

            // Input handling
            if event::poll(Duration::from_millis(100))? {
                let evt = event::read()?;
                match evt {
                    Event::Key(KeyEvent { code, modifiers, kind, .. }) if kind == KeyEventKind::Press => {
                        match code {
                            KeyCode::Char('?') => {
                                show_help = !show_help;
                            }
                            KeyCode::Esc => {
                                if show_help {
                                    show_help = false;
                                } else if !input.is_empty() {
                                    input.clear();
                                }
                            }
                            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                                break 'outer;
                            }
                            KeyCode::Char('q') if input.is_empty() && !show_help => {
                                break 'outer;
                            }
                            KeyCode::Char(ch) => {
                                input.push(ch);
                            }
                            KeyCode::Backspace => {
                                input.pop();
                            }
                            KeyCode::Enter => {
                                let trimmed = input.trim().to_string();
                                if !trimmed.is_empty() {
                                    // Commands
                                    let lower = trimmed.to_lowercase();
                                    if matches!(lower.as_str(), "exit" | "quit" | "bye" | "q") {
                                        break 'outer;
                                    } else if lower == "clear" {
                                        chat_log.clear();
                                        chat_log.push(format!(
                                            "🤖 Vy - {} | Press Enter to send, ? for help, q to quit",
                                            self.model_id
                                        ));
                                        input.clear();
                                        continue;
                                    } else if lower == "help" {
                                        show_help = true;
                                        input.clear();
                                        continue;
                                    } else if lower == "history" {
                                        chat_log.push(format!(
                                            "📝 Conversation History: {} messages stored",
                                            self.conversation_history.len()
                                        ));
                                        input.clear();
                                        continue;
                                    }

                                    // Send to agent
                                    chat_log.push(format!("💬 You: {}", trimmed));
                                    // Maintain history for memory analysis
                                    self.conversation_history.push(Message::user(&trimmed));

                                    // Blocking call to agent (keeps it simple for now)
                                    match self
                                        .agent
                                        .prompt(&trimmed)
                                        .multi_turn(5)
                                        .with_history(&mut self.conversation_history)
                                        .await
                                    {
                                        Ok(response) => {
                                            for line in response.lines() {
                                                chat_log.push(format!("🤖 Vy: {}", line));
                                            }
                                        }
                                        Err(e) => {
                                            chat_log.push(format!("❌ Error: {}", Self::format_error(&e)));
                                        }
                                    }

                                    input.clear();

                                    // Auto-scroll to bottom after new message
                                    scroll = 0;
                                }
                            }
                            KeyCode::Up => {
                                scroll += 1;
                            }
                            KeyCode::Down => {
                                scroll -= 1;
                                if scroll < 0 { scroll = 0; }
                            }
                            KeyCode::PageUp => {
                                scroll += 5;
                            }
                            KeyCode::PageDown => {
                                scroll -= 5;
                                if scroll < 0 { scroll = 0; }
                            }
                            _ => {}
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }

        // Cleanup terminal
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, event::DisableMouseCapture);
        let _ = terminal.show_cursor();

        // Analyze conversation for memories before saying goodbye
        self.analyze_conversation_memories().await?;
        self.print_goodbye();

        Ok(())

        // Helper: centered rect for popups
        fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ])
                .split(r);

            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ])
                .split(popup_layout[1]);

            horizontal[1]
        }
    }

    fn print_welcome(&self) {
        println!("🤖 Vy - {} | Type 'help' for commands", self.model_id);
        if self.model_id == "gpt-5-mini" {
            println!("⚠️  Note: Google search is disabled for gpt-5-mini compatibility");
        }
        println!();
    }

    fn handle_special_commands(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        matches!(input_lower.as_str(), "exit" | "quit" | "bye" | "q")
    }

    fn is_non_exit_command(&mut self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        match input_lower.as_str() {
            "help" => {
                self.print_help();
                true
            }
            "history" => {
                self.print_history();
                true
            }
            "clear" => {
                // Clear screen (works on most terminals)
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush().ok();
                // Clear conversation history
                let cleared_count = self.conversation_history.len();
                self.conversation_history.clear();
                self.print_welcome();
                if cleared_count > 0 {
                    println!("🧹 Cleared {cleared_count} messages from conversation history.\n");
                }
                true
            }

            _ => false,
        }
    }

    fn print_help(&self) {
        println!("📚 Vy Commands:");
        println!("  exit, quit, bye, q    End the conversation");
        println!("  help                  Show this help message");
        println!("  history              Show conversation history");
        println!("  clear                Clear the screen and start fresh");
        println!();
        println!("💬 Just type naturally to chat with Vy!");
        println!("🧠 Your conversations are automatically remembered for context");
        println!("🔍 Vy has access to real-time Google search and personal memory");
        println!();
    }

    fn print_history(&self) {
        if self.conversation_history.is_empty() {
            println!("📝 No conversation history yet.");
        } else {
            println!(
                "📝 Conversation History: {} messages stored",
                self.conversation_history.len()
            );
            println!("   (Messages are kept for context during this session)");
        }
        println!();
    }

    /// Analyze the entire conversation for memory-worthy information
    async fn analyze_conversation_memories(&self) -> Result<()> {
        if self.conversation_history.is_empty() {
            log::debug!("No conversation history to analyze");
            return Ok(());
        }

        log::debug!("Analyzing conversation for important information...");

        // Get memory file path
        let memory_file = match default_memory_file() {
            Ok(path) => path,
            Err(e) => {
                log::debug!("Failed to get memory file path: {e}");
                return Ok(());
            }
        };

        // Load existing memory
        let mut memory = SimpleMemory::new(memory_file);
        if let Err(e) = memory.load().await {
            log::debug!("Failed to load existing memories: {e}");
            return Ok(());
        }

        // Collect all user messages from the conversation
        let user_messages: Vec<String> = self
            .conversation_history
            .iter()
            .filter_map(|msg| {
                match msg {
                    Message::User { content } => {
                        // Extract text from OneOrMany content using debug format
                        let debug_str = format!("{content:?}");
                        // Parse the text content from the debug string
                        // Format is typically: OneOrMany { first: Text(Text { text: "actual_text" }), rest: [] }
                        if let Some(start) = debug_str.find("text: \"") {
                            let start_idx = start + 7; // length of "text: \""
                            if let Some(end) = debug_str[start_idx..].find("\" }") {
                                let text = &debug_str[start_idx..start_idx + end];
                                return Some(text.to_string());
                            }
                        }
                        None
                    }
                    _ => None,
                }
            })
            .collect();

        // Combine all user messages into one analysis to avoid duplicates
        if user_messages.is_empty() {
            log::debug!("No user messages to analyze");
            return Ok(());
        }

        let combined_conversation = user_messages.join(" ");
        let conversation_id = format!(
            "conversation_{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );

        // Skip very short conversations or pure command conversations
        if combined_conversation.len() < 10
            || combined_conversation
                .chars()
                .all(|c| c.is_ascii_punctuation() || c.is_whitespace())
        {
            log::debug!("Conversation too short or contains no meaningful content");
            return Ok(());
        }

        // Use LLM-based memory analysis for better fact extraction
        match memory
            .learn_from_input(
                &combined_conversation,
                conversation_id.clone(),
                &self.api_key,
                &self.memory_model_id,
            )
            .await
        {
            Ok(facts) => {
                if !facts.is_empty() {
                    log::debug!(
                        "Analyzed {} message(s) from this conversation, stored {} memories",
                        user_messages.len(),
                        facts.len()
                    );
                } else {
                    log::debug!("No new memorable information found");
                }
            }
            Err(e) => {
                log::debug!("Failed to process conversation: {e}");
            }
        }

        Ok(())
    }

    fn print_goodbye(&self) {
        println!("👋 Goodbye! Have a great day! 🌟");
    }

    fn format_error(e: &dyn std::error::Error) -> String {
        let error_str = e.to_string();

        // Make common errors more user-friendly
        if error_str.contains("invalid_api_key") {
            "🔑 Invalid API key detected!\n   Run: vy config set llm_api_key\n   Then paste your OpenAI API key when prompted.".to_string()
        } else if error_str.contains("insufficient_quota") {
            "💳 API quota exceeded. Please check your OpenAI billing and usage.".to_string()
        } else if error_str.contains("rate_limit_exceeded") {
            "⏱️ Rate limit exceeded. Please wait a moment and try again.".to_string()
        } else if error_str.contains("network") || error_str.contains("connection") {
            "🌐 Network error. Please check your internet connection.".to_string()
        } else {
            // Truncate very long error messages
            if error_str.len() > 150 {
                format!("{}...", &error_str[..150])
            } else {
                error_str
            }
        }
    }
}
