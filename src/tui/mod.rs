use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use rig::agent::Agent;
use rig::completion::{Message, Prompt, request::CompletionModel};
use std::io;
use tokio::sync::mpsc;

pub mod app;
pub mod input;
pub mod message;

use app::App;
use input::InputHandler;
use message::{ChatMessage, MessageType};

use crate::simple_memory::{SimpleMemory, default_memory_file};

/// Main TUI interface for Vy chatbot
pub struct VyTui<M: CompletionModel> {
    agent: Agent<M>,
    model_id: String,
    api_key: String,
    memory_model_id: String,
}

impl<M: CompletionModel> VyTui<M> {
    pub fn new(
        agent: Agent<M>,
        model_id: String,
        api_key: String,
        memory_model_id: String,
    ) -> Self {
        Self {
            agent,
            model_id,
            api_key,
            memory_model_id,
        }
    }

    pub async fn run(self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create app state
        let mut app = App::new(self.model_id.clone());

        // Create channels for async communication
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Create input handler
        let mut input_handler = InputHandler::new();

        let result = self
            .run_app(&mut terminal, &mut app, &mut input_handler, tx, &mut rx)
            .await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        // Analyze conversation for memories before exiting
        if let Err(e) = self
            .analyze_conversation_memories(&app.conversation_history)
            .await
        {
            eprintln!("Warning: Failed to analyze conversation memories: {e}");
        }

        result
    }

    async fn run_app<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        app: &mut App,
        _input_handler: &mut InputHandler,
        _tx: mpsc::UnboundedSender<String>,
        rx: &mut mpsc::UnboundedReceiver<String>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f, app))?;

            // Handle events
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') if app.in_help_mode => {
                                app.in_help_mode = false;
                            }
                            KeyCode::F(1) if !app.in_help_mode => {
                                app.in_help_mode = true;
                            }
                            KeyCode::Esc => {
                                if app.in_help_mode {
                                    app.in_help_mode = false;
                                } else {
                                    break;
                                }
                            }
                            KeyCode::Enter
                                if !app.in_help_mode && !app.input_handler.input.is_empty() =>
                            {
                                let user_input = app.input_handler.input.clone();
                                app.input_handler.clear();

                                // Add user message to chat
                                app.add_message(ChatMessage::new(
                                    MessageType::User,
                                    user_input.clone(),
                                ));

                                // Add to conversation history
                                app.conversation_history.push(Message::user(&user_input));

                                // Show thinking message
                                app.add_message(ChatMessage::new(
                                    MessageType::Thinking,
                                    "🤖 Thinking...".to_string(),
                                ));

                                // Send request to agent
                                self.handle_user_message(user_input, app).await?;
                            }
                            KeyCode::Char(c) if !app.in_help_mode => {
                                app.input_handler.handle_char_input(c);
                            }
                            KeyCode::Backspace if !app.in_help_mode => {
                                app.input_handler.handle_backspace();
                            }
                            KeyCode::Left if !app.in_help_mode => {
                                app.input_handler.move_cursor_left();
                            }
                            KeyCode::Right if !app.in_help_mode => {
                                app.input_handler.move_cursor_right();
                            }
                            KeyCode::Up if !app.in_help_mode => {
                                app.scroll_messages_up();
                            }
                            KeyCode::Down if !app.in_help_mode => {
                                app.scroll_messages_down();
                            }
                            KeyCode::PageUp if !app.in_help_mode => {
                                app.scroll_messages_page_up();
                            }
                            KeyCode::PageDown if !app.in_help_mode => {
                                app.scroll_messages_page_down();
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Check for async responses
            while let Ok(response) = rx.try_recv() {
                // Remove thinking message
                app.remove_thinking_messages();

                // Add assistant response
                app.add_message(ChatMessage::new(MessageType::Assistant, response.clone()));
                app.conversation_history.push(Message::assistant(&response));
            }
        }

        Ok(())
    }

    async fn handle_user_message(&self, user_input: String, app: &mut App) -> Result<()> {
        // Process the message with the agent
        match self
            .agent
            .prompt(&user_input)
            .multi_turn(5)
            .with_history(&mut app.conversation_history.clone())
            .await
        {
            Ok(response) => {
                // Remove thinking message and add response
                app.remove_thinking_messages();
                app.add_message(ChatMessage::new(MessageType::Assistant, response.clone()));
                app.conversation_history.push(Message::assistant(&response));
            }
            Err(e) => {
                app.remove_thinking_messages();
                let error_msg = self.format_error(&e);
                app.add_message(ChatMessage::new(MessageType::Error, error_msg));
            }
        }

        Ok(())
    }

    fn ui(&self, f: &mut Frame, app: &App) {
        if app.in_help_mode {
            self.draw_help(f);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Messages
                Constraint::Length(3), // Input
                Constraint::Length(1), // Status
            ])
            .split(f.area());

        // Header
        let header = Paragraph::new(format!("🤖 Vy - {} | F1: Help | Esc: Exit", app.model_id))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, chunks[0]);

        // Messages
        let messages: Vec<ListItem> = app
            .messages
            .iter()
            .skip(app.scroll_offset)
            .map(|msg| {
                let style = match msg.message_type {
                    MessageType::User => Style::default().fg(Color::Green),
                    MessageType::Assistant => Style::default().fg(Color::Blue),
                    MessageType::Error => Style::default().fg(Color::Red),
                    MessageType::Thinking => Style::default().fg(Color::Yellow),
                    MessageType::System => Style::default().fg(Color::Gray),
                };

                let prefix = match msg.message_type {
                    MessageType::User => "💬 You: ",
                    MessageType::Assistant => "🤖 Vy: ",
                    MessageType::Error => "❌ Error: ",
                    MessageType::Thinking => "",
                    MessageType::System => "ℹ️  ",
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                    Span::styled(&msg.content, style),
                ]))
            })
            .collect();

        let messages_list = List::new(messages)
            .block(Block::default().borders(Borders::ALL).title("Chat"))
            .style(Style::default().fg(Color::White));
        f.render_widget(messages_list, chunks[1]);

        // Input
        let input = Paragraph::new(app.input_handler.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[2]);

        // Set cursor position
        f.set_cursor_position((
            chunks[2].x + app.input_handler.cursor_position as u16 + 1,
            chunks[2].y + 1,
        ));

        // Status bar
        let status = format!(
            "Messages: {} | Scroll: ↑↓ | Page: PgUp/PgDn | History: {} msgs",
            app.messages.len(),
            app.conversation_history.len()
        );
        let status_bar = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));
        f.render_widget(status_bar, chunks[3]);
    }

    fn draw_help(&self, f: &mut Frame) {
        let help_text = vec![
            "🤖 Vy - Terminal User Interface Help",
            "",
            "⌨️  Keyboard Controls:",
            "  Enter        - Send message",
            "  Esc          - Exit application (or close help)",
            "  F1           - Show/hide this help",
            "  ↑/↓          - Scroll through messages",
            "  PgUp/PgDn    - Scroll messages by page",
            "  ←/→          - Move cursor in input field",
            "  Backspace    - Delete character",
            "",
            "💡 Features:",
            "  • Real-time chat with AI assistant",
            "  • Conversation history maintained",
            "  • Memory integration for context",
            "  • Google search capabilities",
            "  • Nutrition analysis from photos",
            "",
            "🎨 Message Colors:",
            "  Green   - Your messages",
            "  Blue    - Vy's responses",
            "  Red     - Error messages",
            "  Yellow  - System/thinking messages",
            "",
            "Press 'q' or 'Esc' to close this help screen",
        ];

        let help_content: Vec<ListItem> = help_text
            .into_iter()
            .map(|line| {
                let style = if line.starts_with("🤖") {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("  ") && line.contains(" - ") {
                    Style::default().fg(Color::White)
                } else if line.starts_with("⌨️") || line.starts_with("💡") || line.starts_with("🎨")
                {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with("  Green")
                    || line.starts_with("  Blue")
                    || line.starts_with("  Red")
                    || line.starts_with("  Yellow")
                {
                    let color = if line.contains("Green") {
                        Color::Green
                    } else if line.contains("Blue") {
                        Color::Blue
                    } else if line.contains("Red") {
                        Color::Red
                    } else {
                        Color::Yellow
                    };
                    Style::default().fg(color)
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(Line::from(Span::styled(line, style)))
            })
            .collect();

        let help_popup = List::new(help_content)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().bg(Color::Black));

        let area = self.centered_rect(80, 80, f.area());
        f.render_widget(Clear, area);
        f.render_widget(help_popup, area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// Analyze the entire conversation for memory-worthy information
    async fn analyze_conversation_memories(&self, conversation_history: &[Message]) -> Result<()> {
        if conversation_history.is_empty() {
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
        let user_messages: Vec<String> = conversation_history
            .iter()
            .filter_map(|msg| {
                match msg {
                    Message::User { content } => {
                        // Extract text from OneOrMany content using debug format
                        let debug_str = format!("{content:?}");
                        // Parse the text content from the debug string
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
            "tui_conversation_{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );

        // Skip very short conversations
        if combined_conversation.len() < 10 {
            log::debug!("Conversation too short for analysis");
            return Ok(());
        }

        // Use LLM-based memory analysis
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
                        "TUI: Analyzed {} message(s), stored {} memories",
                        user_messages.len(),
                        facts.len()
                    );
                }
            }
            Err(e) => {
                log::debug!("Failed to process TUI conversation: {e}");
            }
        }

        Ok(())
    }

    fn format_error(&self, e: &dyn std::error::Error) -> String {
        let error_str = e.to_string();

        // Make common errors more user-friendly
        if error_str.contains("invalid_api_key") {
            "🔑 Invalid API key! Run: vy config set llm_api_key".to_string()
        } else if error_str.contains("insufficient_quota") {
            "💳 API quota exceeded. Check your OpenAI billing.".to_string()
        } else if error_str.contains("rate_limit_exceeded") {
            "⏱️ Rate limit exceeded. Please wait and try again.".to_string()
        } else if error_str.contains("network") || error_str.contains("connection") {
            "🌐 Network error. Check your internet connection.".to_string()
        } else {
            // Truncate very long error messages
            if error_str.len() > 100 {
                format!("{}...", &error_str[..100])
            } else {
                error_str
            }
        }
    }
}
