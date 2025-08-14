use std::{io::Write, path::Path};

use anyhow::{Context, Result};
use clap::Subcommand;
use log::debug;

use crate::prefs::{self, Prefs};

#[derive(Debug, Clone)]
pub enum ConfigKey {
    LlmApiKey,
    GoogleApiKey,
    GoogleSearchEngineId,
    ModelId,
}

impl ConfigKey {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "llm_api_key" => Some(Self::LlmApiKey),
            "google_api_key" => Some(Self::GoogleApiKey),
            "google_search_engine_id" => Some(Self::GoogleSearchEngineId),
            "model_id" => Some(Self::ModelId),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LlmApiKey => "llm_api_key",
            Self::GoogleApiKey => "google_api_key",
            Self::GoogleSearchEngineId => "google_search_engine_id",
            Self::ModelId => "model_id",
        }
    }

    pub fn is_sensitive(&self) -> bool {
        matches!(self, Self::LlmApiKey | Self::GoogleApiKey)
    }

    pub fn get_value<'a>(&self, prefs: &'a Prefs) -> &'a str {
        match self {
            Self::LlmApiKey => &prefs.llm_api_key,
            Self::GoogleApiKey => &prefs.google_api_key,
            Self::GoogleSearchEngineId => &prefs.google_search_engine_id,
            Self::ModelId => &prefs.model_id,
        }
    }

    pub fn set_value(&self, prefs: &mut Prefs, value: String) {
        match self {
            Self::LlmApiKey => prefs.llm_api_key = value,
            Self::GoogleApiKey => prefs.google_api_key = value,
            Self::GoogleSearchEngineId => prefs.google_search_engine_id = value,
            Self::ModelId => prefs.model_id = value,
        }
    }

    pub fn all_keys() -> &'static [ConfigKey] {
        &[
            Self::LlmApiKey,
            Self::GoogleApiKey,
            Self::GoogleSearchEngineId,
            Self::ModelId,
        ]
    }
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
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

fn validate_model_id(model_id: &str) -> Result<(), String> {
    let valid_models = [
        "gpt-3.5-turbo",
        "gpt-4",
        "gpt-4-turbo",
        "gpt-4o",
        "gpt-4o-mini",
        "gpt-5-mini",
    ];

    if valid_models.contains(&model_id) {
        Ok(())
    } else {
        Err(format!(
            "Model '{}' is not in the list of common models. Valid options: {}",
            model_id,
            valid_models.join(", ")
        ))
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

pub fn run_config(
    action: &ConfigAction,
    prefs_path: &Path,
    load_prefs_fn: impl Fn(&Path) -> Result<Prefs>,
) -> Result<()> {
    match action {
        ConfigAction::Get { key } => {
            let prefs = load_prefs_fn(prefs_path).context("Failed to load configuration. Make sure the config file exists or use 'config set' to create it.")?;

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

            let mut prefs = load_prefs_fn(prefs_path).unwrap_or_else(|_| {
                debug!("Creating new preferences file");
                Prefs {
                    llm_api_key: String::new(),
                    google_api_key: String::new(),
                    google_search_engine_id: String::new(),
                    model_id: "gpt-3.5-turbo".to_string(),
                }
            });

            // Determine the actual value to use
            let actual_value = if let Some(v) = value {
                // Value provided on command line
                v.clone()
            } else if config_key.is_sensitive() {
                // Interactive input for sensitive keys
                print!("Enter value for '{key}' (input will be hidden): ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
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

            // Validate model_id if that's what we're setting
            if matches!(config_key, ConfigKey::ModelId) {
                if let Err(warning) = validate_model_id(&actual_value) {
                    println!("⚠️  Warning: {}", warning);
                    print!("Continue anyway? (y/N): ");
                    std::io::stdout()
                        .flush()
                        .context("Failed to flush stdout")?;
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .context("Failed to read input")?;
                    if !input.trim().to_lowercase().starts_with('y') {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                }
            }

            config_key.set_value(&mut prefs, actual_value);
            prefs::save_prefs(&prefs, prefs_path).context("Failed to save preferences")?;
            println!("Successfully set {}", config_key.as_str());
            println!("Configuration saved to: {}", prefs_path.display());
        }
        ConfigAction::List => {
            let prefs = load_prefs_fn(prefs_path).context("Failed to load configuration. Make sure the config file exists or use 'config set' to create it.")?;
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
