use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use log::debug;

use crate::prefs::{self, Prefs};

pub mod chat;
pub mod config;
pub mod simple_memory;

pub use config::ConfigAction;
pub use simple_memory::SimpleMemoryCommand;

static DEFAULT_PREFS_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

fn default_prefs_path() -> Option<&'static Path> {
    DEFAULT_PREFS_PATH
        .get_or_init(|| {
            directories::ProjectDirs::from("vy", "", "")
                .map(|proj_dirs| proj_dirs.config_dir().join("prefs.toml"))
        })
        .as_deref()
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// Optional prefs path
    #[clap(long, value_parser)]
    prefs_path: Option<PathBuf>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Start the chatbot (default: from config, use --tui/--cli to override)
    Chat {
        /// Use TUI (Terminal User Interface) mode - modern full-screen interface with real-time chat
        #[clap(long)]
        tui: bool,
        /// Use CLI mode - classic text-based interface
        #[clap(long)]
        cli: bool,
    },
    /// Manage configuration values
    Config {
        /// Edit the preferences file in your default editor
        #[clap(long)]
        edit: bool,
        #[clap(subcommand)]
        action: Option<ConfigAction>,
    },
    /// Memory management
    Remember {
        #[clap(subcommand)]
        action: SimpleMemoryCommand,
    },
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Chat { tui, cli } => {
                let prefs = self.load_prefs()?;

                // Determine which mode to use
                let use_tui = if *tui && *cli {
                    // Both flags specified - show error
                    anyhow::bail!(
                        "Cannot specify both --tui and --cli flags. Choose one or rely on default configuration."
                    );
                } else if *tui {
                    true
                } else if *cli {
                    false
                } else {
                    // No explicit flag, use configuration default
                    prefs.default_chat_mode == "tui"
                };

                if use_tui {
                    chat::run_chat_tui(&prefs).await
                } else {
                    chat::run_chat(&prefs).await
                }
            }
            Commands::Config { edit, action } => {
                let prefs_path = self
                    .prefs_path
                    .as_deref()
                    .or(default_prefs_path())
                    .context("Please specify a prefs path via --prefs-path")?;

                if *edit {
                    self.run_config_edit(prefs_path).await
                } else if let Some(action) = action {
                    config::run_config(action, prefs_path, |path| self.load_prefs_strict(path))
                } else {
                    // Default to list when no action is specified
                    config::run_config(&config::ConfigAction::List, prefs_path, |path| {
                        self.load_prefs_strict(path)
                    })
                }
            }
            Commands::Remember { action } => action.clone().run().await,
        }
    }

    fn load_prefs(&self) -> Result<Prefs> {
        let prefs_path = self
            .prefs_path
            .as_deref()
            .or(default_prefs_path())
            .context("Please specify a prefs path via --prefs-path")?;
        prefs::load_or_create_prefs(prefs_path)
    }

    fn load_prefs_strict(&self, prefs_path: &Path) -> Result<Prefs> {
        debug!("prefs_path: {prefs_path:?}");

        let prefs = prefs::load_prefs(prefs_path)
            .with_context(|| {
                format!(
                    "Failed to load configuration. Make sure the config file exists or use 'vy config init' to create it.\nExpected location: {}",
                    prefs_path.display()
                )
            })?;
        debug!("prefs: {prefs:?}");

        Ok(prefs)
    }

    async fn run_config_edit(&self, prefs_path: &Path) -> Result<()> {
        // Ensure prefs file exists
        if !prefs_path.exists() {
            anyhow::bail!(
                "Configuration file not found at: {}\n💡 Run 'vy config init' to set up all required configuration first",
                prefs_path.display()
            );
        }

        // Open in default editor
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let status = std::process::Command::new(&editor)
            .arg(prefs_path)
            .status()
            .with_context(|| format!("Failed to open editor: {editor}"))?;

        if !status.success() {
            anyhow::bail!("Editor exited with non-zero status");
        }

        println!("✅ Configuration file edited successfully!");

        Ok(())
    }
}
