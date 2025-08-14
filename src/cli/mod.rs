use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use log::debug;
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::Vy;

use crate::prefs::{self, Prefs};

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

#[derive(Debug, clap::Subcommand)]
enum ConfigAction {
    /// Get a configuration value
    Get {
        /// The configuration key to retrieve
        key: String,
    },
    /// Set a configuration value
    Set {
        /// The configuration key to set
        key: String,
        /// The value to set
        value: String,
    },
    /// List all configuration values
    List,
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Chat => self.run_chat().await,
            Commands::Config { action } => self.run_config(action).await,
        }
    }

    async fn run_chat(&self) -> Result<()> {
        let prefs = self.load_prefs()?;

        let client = openai::Client::builder(&prefs.llm_api_key)
            .build()
            .context("Failed to create LLM client")?;

        let agent = client.agent("gpt-5-nano").preamble("you are a helpful, female, chatbot with a slightly snarky tone. You provide thoughtful and helpful resposes to the user's queries").build();

        let vy = Vy::new(agent);
        vy.chat().await.context("Failed to start Vy chatbot")?;

        Ok(())
    }

    async fn run_config(&self, action: &ConfigAction) -> Result<()> {
        let prefs_path = self
            .prefs_path
            .as_deref()
            .or(default_prefs_path())
            .context("Please specify a prefs path via --prefs-path")?;

        match action {
            ConfigAction::Get { key } => {
                let prefs = self.load_prefs().context("Failed to load configuration. Make sure the config file exists or use 'config set' to create it.")?;
                match key.as_str() {
                    "llm_api_key" => {
                        if prefs.llm_api_key.is_empty() {
                            println!("llm_api_key: (not set)");
                        } else {
                            // Show first few characters for confirmation but hide the rest
                            let masked = if prefs.llm_api_key.len() > 8 {
                                format!(
                                    "{}...{}",
                                    &prefs.llm_api_key[..4],
                                    &prefs.llm_api_key[prefs.llm_api_key.len() - 4..]
                                )
                            } else {
                                "[HIDDEN]".to_string()
                            };
                            println!("llm_api_key: {}", masked);
                        }
                    }
                    _ => {
                        eprintln!("Unknown configuration key: {}", key);
                        eprintln!("Available keys: llm_api_key");
                        std::process::exit(1);
                    }
                }
            }
            ConfigAction::Set { key, value } => {
                let mut prefs = self.load_prefs().unwrap_or_else(|_| {
                    debug!("Creating new preferences file");
                    Prefs {
                        llm_api_key: String::new(),
                    }
                });

                match key.as_str() {
                    "llm_api_key" => {
                        if value.trim().is_empty() {
                            eprintln!("Error: API key cannot be empty");
                            std::process::exit(1);
                        }
                        prefs.llm_api_key = value.clone();
                        prefs::save_prefs(&prefs, prefs_path)
                            .context("Failed to save preferences")?;
                        println!("Successfully set llm_api_key");
                        println!("Configuration saved to: {}", prefs_path.display());
                    }
                    _ => {
                        eprintln!("Unknown configuration key: {}", key);
                        eprintln!("Available keys: llm_api_key");
                        std::process::exit(1);
                    }
                }
            }
            ConfigAction::List => {
                let prefs = self.load_prefs().context("Failed to load configuration. Make sure the config file exists or use 'config set' to create it.")?;
                println!("Configuration file: {}", prefs_path.display());
                println!("Available settings:");
                if prefs.llm_api_key.is_empty() {
                    println!("  llm_api_key: (not set)");
                } else {
                    let masked = if prefs.llm_api_key.len() > 8 {
                        format!(
                            "{}...{}",
                            &prefs.llm_api_key[..4],
                            &prefs.llm_api_key[prefs.llm_api_key.len() - 4..]
                        )
                    } else {
                        "[HIDDEN]".to_string()
                    };
                    println!("  llm_api_key: {}", masked);
                }
            }
        }

        Ok(())
    }

    fn load_prefs(&self) -> Result<Prefs> {
        let prefs_path = self
            .prefs_path
            .as_deref()
            .or(default_prefs_path())
            .context("Please specify a prefs path via --prefs-path")?;
        debug!("prefs_path: {prefs_path:?}");

        let prefs = prefs::load_prefs(prefs_path)
            .with_context(|| format!("Failed to load prefs from path: {}", prefs_path.display()))?;
        debug!("prefs: {prefs:?}");

        Ok(prefs)
    }
}
