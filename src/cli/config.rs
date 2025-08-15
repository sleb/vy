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
    MemoryModelId,
    MemorySimilarityModelId,
    MemoryPreamble,
}

impl ConfigKey {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "llm_api_key" => Some(Self::LlmApiKey),
            "google_api_key" => Some(Self::GoogleApiKey),
            "google_search_engine_id" => Some(Self::GoogleSearchEngineId),
            "model_id" => Some(Self::ModelId),
            "memory_model_id" => Some(Self::MemoryModelId),
            "memory_similarity_model_id" => Some(Self::MemorySimilarityModelId),
            "memory_preamble" => Some(Self::MemoryPreamble),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LlmApiKey => "llm_api_key",
            Self::GoogleApiKey => "google_api_key",
            Self::GoogleSearchEngineId => "google_search_engine_id",
            Self::ModelId => "model_id",
            Self::MemoryModelId => "memory_model_id",
            Self::MemorySimilarityModelId => "memory_similarity_model_id",
            Self::MemoryPreamble => "memory_preamble",
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
            Self::MemoryModelId => &prefs.memory_model_id,
            Self::MemorySimilarityModelId => &prefs.memory_similarity_model_id,
            Self::MemoryPreamble => &prefs.memory_preamble,
        }
    }

    pub fn set_value(&self, prefs: &mut Prefs, value: String) {
        match self {
            Self::LlmApiKey => prefs.llm_api_key = value,
            Self::GoogleApiKey => prefs.google_api_key = value,
            Self::GoogleSearchEngineId => prefs.google_search_engine_id = value,
            Self::ModelId => prefs.model_id = value,
            Self::MemoryModelId => prefs.memory_model_id = value,
            Self::MemorySimilarityModelId => prefs.memory_similarity_model_id = value,
            Self::MemoryPreamble => prefs.memory_preamble = value,
        }
    }

    pub fn all_keys() -> &'static [ConfigKey] {
        &[
            Self::LlmApiKey,
            Self::GoogleApiKey,
            Self::GoogleSearchEngineId,
            Self::ModelId,
            Self::MemoryModelId,
            Self::MemorySimilarityModelId,
            Self::MemoryPreamble,
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
    /// Initialize configuration file with default values
    Init,
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

fn prompt_for_sensitive_field(field_name: &str) -> Result<Option<String>> {
    print!("Enter {} (input will be hidden, press Enter to skip): ", field_name);
    std::io::stdout()
        .flush()
        .context("Failed to flush stdout")?;
    
    // Check if running in test mode via environment variable
    if std::env::var("VY_TEST_MODE").is_ok() {
        // In test mode, read from stdin normally
        let mut value = String::new();
        std::io::stdin()
            .read_line(&mut value)
            .context("Failed to read input")?;
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    } else {
        let value = rpassword::read_password().context("Failed to read input")?;
        if value.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

fn prompt_for_plain_field(field_name: &str, required: bool) -> Result<Option<String>> {
    if required {
        print!("Enter {} (required): ", field_name);
    } else {
        print!("Enter {} (press Enter to skip): ", field_name);
    }
    std::io::stdout()
        .flush()
        .context("Failed to flush stdout")?;
    
    let mut value = String::new();
    std::io::stdin()
        .read_line(&mut value)
        .context("Failed to read input")?;
    
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed))
    }
}

pub fn run_config(
    action: &ConfigAction,
    prefs_path: &Path,
    load_prefs_fn: impl Fn(&Path) -> Result<Prefs>,
) -> Result<()> {
    match action {
        ConfigAction::Get { key } => {
            let prefs = load_prefs_fn(prefs_path).context("Failed to load configuration. Make sure the config file exists or use 'vy config init' to create it.")?;

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
                    preamble: crate::prefs::default_preamble(),
                    memory_model_id: "gpt-4".to_string(),
                    memory_similarity_model_id: "gpt-3.5-turbo".to_string(),
                    memory_preamble: crate::prefs::default_memory_preamble(),
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
            if matches!(
                config_key,
                ConfigKey::ModelId | ConfigKey::MemoryModelId | ConfigKey::MemorySimilarityModelId
            ) {
                if let Err(warning) = validate_model_id(&actual_value) {
                    println!("⚠️  Warning: {warning}");
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
            let prefs = load_prefs_fn(prefs_path).context("Failed to load configuration. Make sure the config file exists or use 'vy config init' to create it.")?;
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
        ConfigAction::Init => {
            if prefs_path.exists() {
                println!(
                    "⚠️  Configuration file already exists at: {}",
                    prefs_path.display()
                );
                print!("Overwrite existing configuration? (y/N): ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .context("Failed to read input")?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Configuration initialization cancelled.");
                    return Ok(());
                }
            }

            println!("🚀 Let's set up your Vy configuration!");
            println!("Fields with defaults will be set automatically. You'll be prompted for required fields.");
            println!();

            // Start with defaults for all fields
            let mut prefs = Prefs {
                llm_api_key: String::new(),
                google_api_key: String::new(),
                google_search_engine_id: String::new(),
                model_id: "gpt-3.5-turbo".to_string(),
                preamble: crate::prefs::default_preamble(),
                memory_model_id: "gpt-4".to_string(),
                memory_similarity_model_id: "gpt-3.5-turbo".to_string(),
                memory_preamble: crate::prefs::default_memory_preamble(),
            };

            // Prompt for fields without defaults
            let mut missing_required = Vec::new();

            // 1. LLM API Key (sensitive, required)
            if let Some(api_key) = prompt_for_sensitive_field("LLM API key")? {
                prefs.llm_api_key = api_key;
            } else {
                missing_required.push("llm_api_key");
            }

            // 2. Google API Key (sensitive, optional)
            if let Some(google_key) = prompt_for_sensitive_field("Google API key (optional)")? {
                prefs.google_api_key = google_key;
            }

            // 3. Google Search Engine ID (plain text, only if Google API key provided)
            if !prefs.google_api_key.is_empty() {
                if let Some(engine_id) = prompt_for_plain_field("Google Search Engine ID", true)? {
                    prefs.google_search_engine_id = engine_id;
                } else {
                    println!("⚠️  Google Search Engine ID is required when Google API key is provided");
                    missing_required.push("google_search_engine_id");
                }
            }

            // Check if any required fields are missing
            if !missing_required.is_empty() {
                eprintln!("❌ Error: Required field(s) not provided: {}", missing_required.join(", "));
                eprintln!("Configuration initialization aborted.");
                eprintln!("💡 Run 'vy config init' again to retry, or set individual fields with 'vy config set <field> <value>'");
                std::process::exit(1);
            }

            // Save the configuration
            prefs::save_prefs(&prefs, prefs_path)?;
            println!();
            println!(
                "✅ Configuration file initialized at: {}",
                prefs_path.display()
            );
            
            // Show what was configured
            println!("📋 Configured settings:");
            println!("   • LLM API key: {}", if prefs.llm_api_key.is_empty() { "❌ Not set" } else { "✅ Set" });
            println!("   • Google API key: {}", if prefs.google_api_key.is_empty() { "⚪ Optional (not set)" } else { "✅ Set" });
            println!("   • Google Search Engine ID: {}", if prefs.google_search_engine_id.is_empty() { "⚪ Not needed" } else { "✅ Set" });
            println!("   • Model: {} (default)", prefs.model_id);
            println!("   • Other fields: Set to defaults");
            
            println!();
            println!("🎉 You're ready to start using Vy!");
            println!("💬 Run 'vy chat' to begin a conversation");
        }
    }

    Ok(())
}
