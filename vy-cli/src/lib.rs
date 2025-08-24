use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use log::debug;

use vy_core::config::{VyConfig, load_config, load_or_create_config};

pub mod chat;
pub mod config;

pub use config::ConfigAction;

static DEFAULT_PREFS_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

fn default_config_path() -> Option<&'static Path> {
    DEFAULT_PREFS_PATH
        .get_or_init(|| {
            directories::ProjectDirs::from("vy", "", "")
                .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
        })
        .as_deref()
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// Optional config path
    #[clap(long, value_parser)]
    config_path: Option<PathBuf>,

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
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Chat { tui, cli } => {
                let config = self.load_config()?;

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
                    config.default_chat_mode == "tui"
                };

                if use_tui {
                    vy_tui::run_tui(&config).await
                } else {
                    chat::run_chat(&config).await
                }
            }
            Commands::Config { edit, action } => {
                let config_path = self
                    .config_path
                    .as_deref()
                    .or(default_config_path())
                    .context("Please specify a config path via --config-path")?;

                if *edit {
                    self.run_config_edit(config_path).await
                } else if let Some(action) = action {
                    config::run_config(action, config_path, |path| self.load_config_strict(path))
                } else {
                    // Default to list when no action is specified
                    config::run_config(&config::ConfigAction::List, config_path, |path| {
                        self.load_config_strict(path)
                    })
                }
            }
        }
    }

    fn load_config(&self) -> Result<VyConfig> {
        let config_path = self
            .config_path
            .as_deref()
            .or(default_config_path())
            .context("Please specify a config path via --config-path")?;
        load_or_create_config(config_path)
    }

    fn load_config_strict(&self, config_path: &Path) -> Result<VyConfig> {
        debug!("config_path: {config_path:?}");

        let config = load_config(config_path)
            .with_context(|| {
                format!(
                    "Failed to load configuration. Make sure the config file exists or use 'vy config init' to create it.\nExpected location: {}",
                    config_path.display()
                )
            })?;
        debug!("config: {config:?}");

        Ok(config)
    }

    async fn run_config_edit(&self, config_path: &Path) -> Result<()> {
        // Ensure config file exists
        if !config_path.exists() {
            anyhow::bail!(
                "Configuration file not found at: {}\n💡 Run 'vy config init' to set up all required configuration first",
                config_path.display()
            );
        }

        // Open in default editor
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let status = std::process::Command::new(&editor)
            .arg(config_path)
            .status()
            .with_context(|| format!("Failed to open editor: {editor}"))?;

        if !status.success() {
            anyhow::bail!("Editor exited with non-zero status");
        }

        println!("✅ Configuration file edited successfully!");

        Ok(())
    }
}
