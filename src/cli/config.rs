use std::{io::Write, path::Path};

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::prefs::{self, Prefs};

#[derive(Debug, Clone)]
pub enum ConfigKey {
    LlmApiKey,
    GoogleApiKey,
    GoogleSearchEngineId,
    ModelId,
    MemoryModelId,
    MemorySimilarityModelId,
    DefaultChatMode,
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
            "default_chat_mode" => Some(Self::DefaultChatMode),
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
            Self::DefaultChatMode => "default_chat_mode",
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
            Self::DefaultChatMode => &prefs.default_chat_mode,
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
            Self::DefaultChatMode => prefs.default_chat_mode = value,
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
            Self::DefaultChatMode,
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
    /// Interactive setup - configure all required API keys and settings
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
            "Model '{}' is not in the list of common models.\n   💡 Common options: {}",
            model_id,
            valid_models.join(", ")
        ))
    }
}

fn validate_chat_mode(mode: &str) -> Result<(), String> {
    let valid_modes = ["cli", "tui"];

    if valid_modes.contains(&mode) {
        Ok(())
    } else {
        Err(format!(
            "Chat mode '{}' is not valid.\n   💡 Valid options: {}",
            mode,
            valid_modes.join(", ")
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
                eprintln!("Configuration file not found. Please run 'vy config init' to set up all required configuration.");
                std::process::exit(1);
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

            if matches!(config_key, ConfigKey::DefaultChatMode) {
                if let Err(error) = validate_chat_mode(&actual_value) {
                    println!("❌ {error}");
                    return Ok(());
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

            println!("🚀 Welcome to Vy! Let's set up your configuration.");
            println!("📋 You'll be prompted for API keys and model preferences.");
            println!(
                "💡 Press Enter to accept default values shown in [brackets], or type a new value.\n"
            );

            let mut prefs = Prefs {
                llm_api_key: String::new(),
                google_api_key: String::new(),
                google_search_engine_id: String::new(),
                model_id: "gpt-3.5-turbo".to_string(),
                memory_model_id: "gpt-4".to_string(),
                memory_similarity_model_id: "gpt-3.5-turbo".to_string(),
                default_chat_mode: "cli".to_string(),
            };

            // Prompt for LLM API key (required)
            println!("🔑 OpenAI API Key (required for LLM functionality):");
            println!("   Get your API key at: https://platform.openai.com/api-keys");
            print!("Enter your OpenAI API key (input will be hidden): ");
            std::io::stdout()
                .flush()
                .context("Failed to flush stdout")?;
            let llm_api_key = rpassword::read_password().context("Failed to read API key")?;
            if llm_api_key.trim().is_empty() {
                println!(
                    "⚠️  Warning: No API key provided. You'll need to set this later with 'vy config set llm_api_key'"
                );
            } else {
                prefs.llm_api_key = llm_api_key.trim().to_string();
            }

            // Prompt for model ID with validation
            loop {
                println!("\n🤖 Main Model ID [{}]:", prefs.model_id);
                println!("   This model will be used for general chat conversations.");
                println!(
                    "   💡 Popular choices: gpt-4o (best), gpt-4 (good), gpt-3.5-turbo (fast & cheap)"
                );
                print!("Model ID: ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .context("Failed to read input")?;
                let input = input.trim();

                if input.is_empty() {
                    break; // Use default
                } else {
                    match validate_model_id(input) {
                        Ok(_) => {
                            prefs.model_id = input.to_string();
                            break;
                        }
                        Err(warning) => {
                            println!("⚠️  {warning}");
                            print!("Use this model anyway? (y/N): ");
                            std::io::stdout()
                                .flush()
                                .context("Failed to flush stdout")?;
                            let mut confirm = String::new();
                            std::io::stdin()
                                .read_line(&mut confirm)
                                .context("Failed to read input")?;
                            if confirm.trim().to_lowercase().starts_with('y') {
                                prefs.model_id = input.to_string();
                                break;
                            }
                        }
                    }
                }
            }

            // Prompt for memory model ID with validation
            loop {
                println!("\n🧠 Memory Model ID [{}]:", prefs.memory_model_id);
                println!("   This model extracts and processes memories from conversations.");
                println!("   💡 Recommended: gpt-4 or gpt-4o for better memory extraction");
                print!("Memory Model ID: ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .context("Failed to read input")?;
                let input = input.trim();

                if input.is_empty() {
                    break; // Use default
                } else {
                    match validate_model_id(input) {
                        Ok(_) => {
                            prefs.memory_model_id = input.to_string();
                            break;
                        }
                        Err(warning) => {
                            println!("⚠️  {warning}");
                            print!("Use this model anyway? (y/N): ");
                            std::io::stdout()
                                .flush()
                                .context("Failed to flush stdout")?;
                            let mut confirm = String::new();
                            std::io::stdin()
                                .read_line(&mut confirm)
                                .context("Failed to read input")?;
                            if confirm.trim().to_lowercase().starts_with('y') {
                                prefs.memory_model_id = input.to_string();
                                break;
                            }
                        }
                    }
                }
            }

            // Prompt for memory similarity model ID with validation
            loop {
                println!(
                    "\n🔍 Memory Similarity Model ID [{}]:",
                    prefs.memory_similarity_model_id
                );
                println!("   This model finds relevant memories when searching.");
                println!("   💡 gpt-3.5-turbo is usually sufficient for similarity matching");
                print!("Memory Similarity Model ID: ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .context("Failed to read input")?;
                let input = input.trim();

                if input.is_empty() {
                    break; // Use default
                } else {
                    match validate_model_id(input) {
                        Ok(_) => {
                            prefs.memory_similarity_model_id = input.to_string();
                            break;
                        }
                        Err(warning) => {
                            println!("⚠️  {warning}");
                            print!("Use this model anyway? (y/N): ");
                            std::io::stdout()
                                .flush()
                                .context("Failed to flush stdout")?;
                            let mut confirm = String::new();
                            std::io::stdin()
                                .read_line(&mut confirm)
                                .context("Failed to read input")?;
                            if confirm.trim().to_lowercase().starts_with('y') {
                                prefs.memory_similarity_model_id = input.to_string();
                                break;
                            }
                        }
                    }
                }
            }

            // Google Search configuration (required)
            println!("\n🌐 Google Search Configuration (required):");
            println!(
                "   Google search allows Vy to look up current information and recent events."
            );
            println!("   Both API key and Search Engine ID are required for Vy to work properly.");

            // Google API Key (required)
            loop {
                println!("\n🔑 Google API Key:");
                println!("   Get one at: https://console.developers.google.com/");
                print!("Enter your Google API key (input will be hidden): ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
                let google_api_key =
                    rpassword::read_password().context("Failed to read Google API key")?;

                if google_api_key.trim().is_empty() {
                    println!("❌ Google API key is required. Please enter a valid API key.");
                    continue;
                }

                prefs.google_api_key = google_api_key.trim().to_string();
                break;
            }

            // Google Search Engine ID (required)
            loop {
                println!("\n🔍 Google Search Engine ID:");
                println!("   Create a custom search engine at: https://cse.google.com/");
                print!("Search Engine ID: ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
                let mut search_engine_id = String::new();
                std::io::stdin()
                    .read_line(&mut search_engine_id)
                    .context("Failed to read search engine ID")?;

                if search_engine_id.trim().is_empty() {
                    println!(
                        "❌ Google Search Engine ID is required. Please enter a valid Search Engine ID."
                    );
                    continue;
                }

                prefs.google_search_engine_id = search_engine_id.trim().to_string();
                break;
            }

            // Default chat mode (optional)
            loop {
                println!("\n💬 Default Chat Mode:");
                println!("   • 'cli' - Classic text-based interface (simple, universal)");
                println!("   • 'tui' - Modern terminal UI (visual, interactive)");
                print!("Choose default mode (cli/tui) [default: cli]: ");
                std::io::stdout()
                    .flush()
                    .context("Failed to flush stdout")?;
                let mut chat_mode = String::new();
                std::io::stdin()
                    .read_line(&mut chat_mode)
                    .context("Failed to read chat mode")?;

                let chat_mode = chat_mode.trim();
                if chat_mode.is_empty() {
                    prefs.default_chat_mode = "cli".to_string();
                    break;
                } else {
                    match validate_chat_mode(chat_mode) {
                        Ok(_) => {
                            prefs.default_chat_mode = chat_mode.to_string();
                            break;
                        }
                        Err(error) => {
                            println!("❌ {error}");
                            continue;
                        }
                    }
                }
            }

            prefs::save_prefs(&prefs, prefs_path)?;

            println!("\n🎉 Configuration Setup Complete!");
            println!("═══════════════════════════════════");
            println!("📁 Config file saved to: {}", prefs_path.display());

            // Status summary
            if !prefs.llm_api_key.is_empty() {
                println!("✅ OpenAI API key configured");
            } else {
                println!("⚠️  OpenAI API key not set");
            }

            println!("✅ Google search configured");

            println!("✅ Models configured:");
            println!("   • Main chat: {}", prefs.model_id);
            println!("   • Memory extraction: {}", prefs.memory_model_id);
            println!(
                "   • Memory similarity: {}",
                prefs.memory_similarity_model_id
            );
            println!("✅ Default chat mode: {}", prefs.default_chat_mode);

            // Next steps
            println!("\n🚀 Next Steps:");
            if !prefs.llm_api_key.is_empty() {
                if prefs.default_chat_mode == "tui" {
                    println!("   • Start chatting: vy chat (will use TUI mode by default)");
                    println!("   • Force CLI mode: vy chat --cli");
                } else {
                    println!("   • Start chatting: vy chat (will use CLI mode by default)");
                    println!("   • Try TUI mode: vy chat --tui");
                }
                println!("   • Test memory: vy chat (memories are auto-saved after conversations)");
            } else {
                println!("   • Set your OpenAI API key: vy config set llm_api_key");
                println!("   • Then start chatting: vy chat");
            }

            println!("\n📝 Manage Your Configuration:");
            println!("   • View all settings: vy config list");
            println!("   • Update a setting: vy config set <key> <value>");
            println!("   • Edit config file: vy config --edit");
        }
    }

    Ok(())
}
