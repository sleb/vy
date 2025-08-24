//! CLI chat interface for Vy
//!
//! This module provides the command-line interface for chatting with Vy,
//! using the core functionality from vy-core.

use anyhow::Result;
use std::io::{self, Write};
use vy_core::{VyCore, builder, config::VyConfig};

/// Run the CLI chat interface
pub async fn run_chat(config: &VyConfig) -> Result<()> {
    // Check for unsupported models
    if config.llm_model_id == "gpt-5-mini" || config.llm_model_id == "gpt-5" {
        eprintln!(
            "❌ Error: {} is not currently supported due to tool calling compatibility issues.",
            config.llm_model_id
        );
        eprintln!("💡 Please use one of these supported models instead:");
        eprintln!("   • gpt-4o");
        eprintln!("   • gpt-4o-mini");
        eprintln!("   • gpt-4");
        eprintln!("   • gpt-3.5-turbo");
        eprintln!("\n   To change your model: vy config set llm_model_id");
        return Ok(());
    }

    // For now, only support OpenAI models to avoid type compatibility issues
    if !config.llm_model_id.starts_with("gpt-") && !config.llm_model_id.starts_with("o1-") {
        eprintln!("❌ Error: Only OpenAI models are currently supported in this CLI version.");
        eprintln!("💡 Please use one of these supported models:");
        eprintln!("   • gpt-4o");
        eprintln!("   • gpt-4o-mini");
        eprintln!("   • gpt-4");
        eprintln!("   • gpt-3.5-turbo");
        eprintln!("\n   To change your model: vy config set llm_model_id");
        return Ok(());
    }

    // Create Vy core instance using the OpenAI builder
    let mut vy = builder::build_openai_vy(config).await?;

    print_welcome(&vy);

    loop {
        // Prompt for user input
        print!("💬 You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Handle special commands
        if handle_special_commands(input) {
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Check if it's a special command that doesn't exit
        if handle_non_exit_commands(input, &mut vy) {
            continue;
        }

        // Show thinking indicator
        print!("🤖 Vy: ");
        io::stdout().flush()?;

        // Get response from agent with error handling
        match vy.send_message(input).await {
            Ok(response) => {
                println!("{response}");
            }
            Err(e) => {
                let formatted_error = vy_core::format_error(&*e);
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

    // TODO: Re-enable vector memory analysis after fixing Sync issues
    // vy.analyze_conversation_memories(&vector_memory).await?;

    print_goodbye();
    Ok(())
}

fn print_welcome<M>(vy: &VyCore<M>)
where
    M: rig::completion::request::CompletionModel,
{
    println!("🤖 Vy - {} | Type 'help' for commands", vy.model_id());
    if vy.model_id() == "gpt-5-mini" {
        println!("⚠️  Note: Google search is disabled for gpt-5-mini compatibility");
    }
    println!();
}

fn handle_special_commands(input: &str) -> bool {
    let input_lower = input.to_lowercase();
    matches!(input_lower.as_str(), "exit" | "quit" | "bye" | "q")
}

fn handle_non_exit_commands<M>(input: &str, vy: &mut VyCore<M>) -> bool
where
    M: rig::completion::request::CompletionModel,
{
    let input_lower = input.to_lowercase();
    match input_lower.as_str() {
        "help" => {
            print_help();
            true
        }
        "history" => {
            print_history(vy);
            true
        }
        "clear" => {
            // Clear screen (works on most terminals)
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().ok();
            // Clear conversation history
            let cleared_count = vy.clear_history();
            print_welcome(vy);
            if cleared_count > 0 {
                println!("🧹 Cleared {cleared_count} messages from conversation history.\n");
            }
            true
        }
        _ => false,
    }
}

fn print_help() {
    println!("📚 Vy Commands:");
    println!("  exit, quit, bye, q    End the conversation");
    println!("  help                  Show this help message");
    println!("  history              Show conversation history");
    println!("  clear                Clear the screen and start fresh");
    println!();
    println!("💬 Just type naturally to chat with Vy!");
    println!("🧠 Your conversations are automatically remembered for context");
    println!("🔍 Vy has access to real-time Google search and personal memory");
    println!("🍽️ Ask Vy to analyze meal photos for ingredient breakdown (perfect for Cronometer)");
    println!();
}

fn print_history<M>(vy: &VyCore<M>)
where
    M: rig::completion::request::CompletionModel,
{
    let history_count = vy.conversation_history().len();
    if history_count == 0 {
        println!("📝 No conversation history yet.");
    } else {
        println!("📝 Conversation History: {history_count} messages stored");
        println!("   (Messages are kept for context during this session)");
    }
    println!();
}

fn print_goodbye() {
    println!("👋 Goodbye! Have a great day! 🌟");
}
