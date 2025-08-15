use anyhow::Result;
use rig::agent::Agent;
use rig::completion::{Message, Prompt, request::CompletionModel};
use std::io::{self, Write};

pub mod memory;
pub mod simple_memory;
pub mod tools;

use simple_memory::{SimpleMemory, default_memory_file};
use tools::auto_memory::AutoMemoryTool;

pub struct Vy<M: CompletionModel> {
    agent: Agent<M>,
    conversation_history: Vec<Message>,
    model_id: String,
}

impl<M: CompletionModel> Vy<M> {
    pub fn new(agent: Agent<M>, model_id: String) -> Self {
        Self {
            agent,
            conversation_history: Vec::new(),
            model_id,
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

            // Show thinking indicator with conversation context
            let msg_count = self.conversation_history.len();
            print!(
                "🤖 Vy ({}): ",
                if msg_count > 0 {
                    format!("{msg_count} msgs")
                } else {
                    "new chat".to_string()
                }
            );
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

    fn print_welcome(&self) {
        println!("┌─────────────────────────────────────────────────────────────────┐");
        println!("│  🤖 Welcome to Vy - Your AI Assistant                           │");
        println!("│  Model: {:<52} │", self.model_id);
        println!("│                                                                 │");
        if self.model_id == "gpt-5-mini" {
            println!("│  ⚠️  Note: Google search is disabled for gpt-5-mini compatibility │");
            println!("│                                                                 │");
        }
        println!("│  Commands:                                                      │");
        println!("│    • 'exit', 'quit', 'bye', 'q' - End conversation              │");
        println!("│    • 'help' - Show available commands                           │");
        println!("│    • 'history' - Show conversation history                      │");
        println!("│    • 'clear' - Clear conversation history                       │");
        println!("└─────────────────────────────────────────────────────────────────┘");
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
        println!("📚 Available Commands:");
        println!("  • exit, quit, bye, q  - End the conversation");
        println!("  • help                - Show this help message");
        println!("  • history             - Show conversation history");
        println!("  • clear               - Clear the screen and start fresh");
        println!("  • Just type naturally to chat with Vy!");
        println!();
    }

    fn print_history(&self) {
        if self.conversation_history.is_empty() {
            println!("📝 No conversation history yet.\n");
            return;
        }

        println!(
            "📝 Conversation History: {} messages stored",
            self.conversation_history.len()
        );
        println!("   (Alternating: You → Vy → You → Vy...)\n");
    }

    /// Analyze the entire conversation for memory-worthy information
    async fn analyze_conversation_memories(&self) -> Result<()> {
        if self.conversation_history.is_empty() {
            return Ok(());
        }

        println!("🧠 Analyzing conversation for important information...");

        // Get memory file path
        let memory_file = match default_memory_file() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("❌ Failed to get memory file path: {e}");
                return Ok(());
            }
        };

        // Load existing memory
        let mut memory = SimpleMemory::new(memory_file);
        if let Err(e) = memory.load().await {
            eprintln!("❌ Failed to load existing memories: {e}");
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
            println!("  ✅ No user messages to analyze");
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
            println!("  ✅ Conversation too short or contains no meaningful content");
            return Ok(());
        }

        // Use the auto memory tool to analyze the combined conversation
        if AutoMemoryTool::should_analyze_message(&combined_conversation) {
            match memory
                .learn_from_input(&combined_conversation, conversation_id.clone())
                .await
            {
                Ok(facts) => {
                    if !facts.is_empty() {
                        println!(
                            "  📝 Analyzed {} message(s) from this conversation",
                            user_messages.len()
                        );
                        match facts.len() {
                            1 => println!("  ✅ Stored 1 new memory"),
                            n => println!("  ✅ Stored {n} new memories"),
                        }
                        println!("  💾 Memories saved for future conversations");
                    } else {
                        println!("  ✅ No new memorable information found");
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to process conversation: {e}");
                }
            }
        } else {
            println!("  ✅ No memorable information detected in conversation");
        }

        Ok(())
    }

    fn print_goodbye(&self) {
        println!("┌─────────────────────────────────────────────────────────────────┐");
        println!("│  👋 Goodbye! Feel free to chat anytime.                         │");
        println!("│  Have a great day! 🌟                                           │");
        println!("└─────────────────────────────────────────────────────────────────┘");
    }

    fn format_error(e: &dyn std::error::Error) -> String {
        let error_str = e.to_string();

        // Make common errors more user-friendly
        if error_str.contains("invalid_api_key") {
            "🔑 Invalid API key detected!\n   Run: vy config set llm_api_key\n   Then paste your OpenAI API key when prompted."
                .to_string()
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
