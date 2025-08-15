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
    /// Start the chatbot
    Chat,
    /// Manage configuration values
    Config {
        #[clap(subcommand)]
        action: ConfigAction,
    },
    /// Memory management
    Remember {
        #[clap(subcommand)]
        action: SimpleMemoryCommand,
    },
    /// View or edit the preamble
    Preamble {
        /// Edit the preamble in your default editor
        #[clap(long)]
        edit: bool,
    },
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Chat => {
                let prefs = self.load_prefs()?;
                chat::run_chat(&prefs).await
            }
            Commands::Config { action } => {
                let prefs_path = self
                    .prefs_path
                    .as_deref()
                    .or(default_prefs_path())
                    .context("Please specify a prefs path via --prefs-path")?;

                config::run_config(action, prefs_path, |path| self.load_prefs_from_path(path))
            }
            Commands::Remember { action } => action.clone().run().await,
            Commands::Preamble { edit } => {
                let prefs_path = self
                    .prefs_path
                    .as_deref()
                    .or(default_prefs_path())
                    .context("Please specify a prefs path via --prefs-path")?;

                self.run_preamble(*edit, prefs_path).await
            }
        }
    }

    fn load_prefs(&self) -> Result<Prefs> {
        let prefs_path = self
            .prefs_path
            .as_deref()
            .or(default_prefs_path())
            .context("Please specify a prefs path via --prefs-path")?;
        self.load_prefs_from_path(prefs_path)
    }

    fn load_prefs_from_path(&self, prefs_path: &Path) -> Result<Prefs> {
        debug!("prefs_path: {prefs_path:?}");

        let prefs = prefs::load_prefs(prefs_path)
            .with_context(|| format!("Failed to load prefs from path: {}", prefs_path.display()))?;
        debug!("prefs: {prefs:?}");

        Ok(prefs)
    }

    async fn run_preamble(&self, edit: bool, prefs_path: &Path) -> Result<()> {
        let mut prefs = self.load_prefs_from_path(prefs_path)?;

        if edit {
            // Create a temporary file with the current preamble
            let temp_file = std::env::temp_dir().join("vy_preamble.txt");
            std::fs::write(&temp_file, &prefs.preamble)?;

            // Open in default editor
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let status = std::process::Command::new(&editor)
                .arg(&temp_file)
                .status()
                .with_context(|| format!("Failed to open editor: {}", editor))?;

            if !status.success() {
                anyhow::bail!("Editor exited with non-zero status");
            }

            // Read back the edited content
            let new_preamble =
                std::fs::read_to_string(&temp_file).context("Failed to read edited preamble")?;

            // Clean up temp file
            let _ = std::fs::remove_file(&temp_file);

            // Update prefs if changed
            if new_preamble != prefs.preamble {
                prefs.preamble = new_preamble;
                prefs::save_prefs(&prefs, prefs_path)?;
                println!("✅ Preamble updated successfully!");
            } else {
                println!("ℹ️  Preamble unchanged.");
            }
        } else {
            // Just display the current preamble
            println!("Current preamble:");
            println!("{}", "─".repeat(50));
            println!("{}", prefs.preamble);
            println!("{}", "─".repeat(50));
            println!("To edit: vy preamble --edit");
        }

        Ok(())
    }
}
