use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use log::debug;

use crate::prefs::{self, Prefs};

pub mod chat;
pub mod config;

pub use config::ConfigAction;

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
}
