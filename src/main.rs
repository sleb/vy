use clap::Parser;
use log::debug;

use crate::cli::Cli;

mod cli;
mod prefs;

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();
    println!("CLI: {cli:#?}");

    if let Err(e) = cli.run().await {
        debug!("Error: {e:?}");
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
