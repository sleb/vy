//! Vy - AI-powered chatbot with multiple interface options
//!
//! This is the main binary that coordinates between different interface crates:
//! - vy-core: Core chatbot functionality
//! - vy-cli: Command-line interface
//! - vy-tui: Terminal user interface
//!
//! The binary provides a unified entry point and can route to different interfaces
//! based on user preferences and command-line arguments.

use clap::Parser;
use log::debug;
use vy_cli::Cli;

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();
    debug!("CLI: {cli:#?}");

    if let Err(e) = cli.run().await {
        debug!("Error: {e:?}");
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
