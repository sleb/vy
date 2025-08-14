use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use log::debug;
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::Vy;

use crate::prefs::{self, Prefs};

#[derive(Debug, Clone)]
enum ConfigKey {
    LlmApiKey,
    GoogleApiKey,
    GoogleSearchEngineId,
}

impl ConfigKey {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "llm_api_key" => Some(Self::LlmApiKey),
            "google_api_key" => Some(Self::GoogleApiKey),
            "google_search_engine_id" => Some(Self::GoogleSearchEngineId),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::LlmApiKey => "llm_api_key",
            Self::GoogleApiKey => "google_api_key",
            Self::GoogleSearchEngineId => "google_search_engine_id",
        }
    }

    fn is_sensitive(&self) -> bool {
        matches!(self, Self::LlmApiKey | Self::GoogleApiKey)
    }

    fn get_value<'a>(&self, prefs: &'a Prefs) -> &'a str {
        match self {
            Self::LlmApiKey => &prefs.llm_api_key,
            Self::GoogleApiKey => &prefs.google_api_key,
            Self::GoogleSearchEngineId => &prefs.google_search_engine_id,
        }
    }

    fn set_value(&self, prefs: &mut Prefs, value: String) {
        match self {
            Self::LlmApiKey => prefs.llm_api_key = value,
            Self::GoogleApiKey => prefs.google_api_key = value,
            Self::GoogleSearchEngineId => prefs.google_search_engine_id = value,
        }
    }

    fn all_keys() -> &'static [ConfigKey] {
        &[
            Self::LlmApiKey,
            Self::GoogleApiKey,
            Self::GoogleSearchEngineId,
        ]
    }
}

fn mask_sensitive_value(value: &str) -> String {
    if value.is_empty() {
        "(not set)".to_string()
    } else if value.len() > 8 {
        format!("{}...{}", &value[..4], &value[value.len() - 4..])
    } else {
        "[HIDDEN]".to_string()
    }
}

fn available_keys_message() -> String {
    let keys: Vec<&str> = ConfigKey::all_keys().iter().map(|k| k.as_str()).collect();
    format!("Available keys: {}", keys.join(", "))
}

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
        /// The value to set (omit for interactive input on sensitive keys like API keys)
        value: Option<String>,
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

                let config_key = ConfigKey::from_str(key).unwrap_or_else(|| {
                    eprintln!("Unknown configuration key: {key}");
                    eprintln!("{}", available_keys_message());
                    std::process::exit(1);
                });

                let value = config_key.get_value(&prefs);
                let display_value = if config_key.is_sensitive() {
                    mask_sensitive_value(value)
                } else if value.is_empty() {
                    "(not set)".to_string()
                } else {
                    value.to_string()
                };

                println!("{}: {}", config_key.as_str(), display_value);
            }
            ConfigAction::Set { key, value } => {
                let config_key = ConfigKey::from_str(key).unwrap_or_else(|| {
                    eprintln!("Unknown configuration key: {key}");
                    eprintln!("{}", available_keys_message());
                    std::process::exit(1);
                });

                let mut prefs = self.load_prefs().unwrap_or_else(|_| {
                    debug!("Creating new preferences file");
                    Prefs {
                        llm_api_key: String::new(),
                        google_api_key: String::new(),
                        google_search_engine_id: String::new(),
                    }
                });

                // Determine the actual value to use
                let actual_value = if let Some(v) = value {
                    // Value provided on command line
                    v.clone()
                } else if config_key.is_sensitive() {
                    // Interactive input for sensitive keys
                    print!("Enter value for '{key}' (input will be hidden): ");
                    use std::io::{self, Write};
                    io::stdout().flush().context("Failed to flush stdout")?;
                    rpassword::read_password().context("Failed to read input")?
                } else {
                    // For non-sensitive keys, require the value to be provided
                    eprintln!("Error: Value must be provided for non-sensitive key '{key}'");
                    eprintln!("Usage: vy config set {key} <value>");
                    std::process::exit(1);
                };

                if actual_value.trim().is_empty() {
                    eprintln!("Error: {} cannot be empty", config_key.as_str());
                    std::process::exit(1);
                }

                config_key.set_value(&mut prefs, actual_value);
                prefs::save_prefs(&prefs, prefs_path).context("Failed to save preferences")?;
                println!("Successfully set {}", config_key.as_str());
                println!("Configuration saved to: {}", prefs_path.display());
            }
            ConfigAction::List => {
                let prefs = self.load_prefs().context("Failed to load configuration. Make sure the config file exists or use 'config set' to create it.")?;
                println!("Configuration file: {}", prefs_path.display());
                println!("Available settings:");

                for config_key in ConfigKey::all_keys() {
                    let value = config_key.get_value(&prefs);
                    let display_value = if config_key.is_sensitive() {
                        mask_sensitive_value(value)
                    } else if value.is_empty() {
                        "(not set)".to_string()
                    } else {
                        value.to_string()
                    };
                    println!("  {}: {}", config_key.as_str(), display_value);
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
