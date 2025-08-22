//! Vy TUI - Terminal User Interface for the Vy AI chatbot
//!
//! This crate provides a modern, interactive terminal user interface for Vy.
//! Currently this is a minimal implementation that will be expanded later.

use anyhow::Result;
use vy_core::config::VyConfig;

/// Run the TUI interface
pub async fn run_tui(_config: &VyConfig) -> Result<()> {
    eprintln!("❌ TUI mode is not yet implemented in this refactored version.");
    eprintln!("💡 Please use CLI mode instead: vy chat --cli");
    eprintln!("🚧 TUI support will be added in a future update.");

    Ok(())
}

/// Check if the terminal supports TUI mode
pub fn is_tui_supported() -> bool {
    // Basic check for terminal environment
    std::env::var("TERM").is_ok() && !std::env::var("TERM").unwrap_or_default().is_empty()
}
