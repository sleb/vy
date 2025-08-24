use anyhow::Result;
use clap::Parser;
use log::debug;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    debug!("Starting Vy");

    let cli = vy_cli::Cli::parse();
    cli.run().await
}
