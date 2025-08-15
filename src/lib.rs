use anyhow::Result;
use rig::agent::Agent;
use rig::completion::{Message, Prompt, request::CompletionModel};
use std::io::{self, Write};

pub mod memory;
pub mod simple_memory;
pub mod tools;

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
