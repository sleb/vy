use std::{io::Write, path::Path};

use anyhow::{Context, Result};
use clap::Subcommand;

use vy_core::config::{VyConfig, default_system_prompt, save_config};

#[derive(Debug, Clone)]
pub enum ConfigKey {
    LlmApiKey,
    GoogleApiKey,
    GoogleSearchEngineId,
    ModelId,
    MemoryModelId,
    MemorySimilarityModelId,
    DefaultChatMode,
    VectorMemoryQdrantUrl,
    VectorMemoryQdrantApiKey,
    VectorMemoryCollectionName,
    VectorMemoryEmbeddingModel,
}

impl ConfigKey {
    pub fn from_key(s: &str) -> Option<Self> {
        match s {
            "llm_api_key" => Some(Self::LlmApiKey),
            "google_api_key" => Some(Self::GoogleApiKey),
            "google_search_engine_id" => Some(Self::GoogleSearchEngineId),
            "llm_model_id" | "model_id" => Some(Self::ModelId),
            "memory_model_id" => Some(Self::MemoryModelId),
            "memory_similarity_model_id" => Some(Self::MemorySimilarityModelId),
            "default_chat_mode" => Some(Self::DefaultChatMode),
            "vector_memory_qdrant_url" => Some(Self::VectorMemoryQdrantUrl),
            "vector_memory_qdrant_api_key" => Some(Self::VectorMemoryQdrantApiKey),
            "vector_memory_collection_name" => Some(Self::VectorMemoryCollectionName),
            "vector_memory_embedding_model" => Some(Self::VectorMemoryEmbeddingModel),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LlmApiKey => "llm_api_key",
            Self::GoogleApiKey => "google_api_key",
            Self::GoogleSearchEngineId => "google_search_engine_id",
            Self::ModelId => "llm_model_id",
            Self::MemoryModelId => "memory_model_id",
            Self::MemorySimilarityModelId => "memory_similarity_model_id",
            Self::DefaultChatMode => "default_chat_mode",
            Self::VectorMemoryQdrantUrl => "vector_memory_qdrant_url",
            Self::VectorMemoryQdrantApiKey => "vector_memory_qdrant_api_key",
            Self::VectorMemoryCollectionName => "vector_memory_collection_name",
            Self::VectorMemoryEmbeddingModel => "vector_memory_embedding_model",
        }
    }

    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Self::LlmApiKey | Self::GoogleApiKey | Self::VectorMemoryQdrantApiKey
        )
    }

    pub fn get_value(&self, config: &VyConfig) -> String {
        match self {
            Self::LlmApiKey => config.llm_api_key.clone(),
            Self::GoogleApiKey => config.google_api_key.clone(),
            Self::GoogleSearchEngineId => config.google_search_engine_id.clone(),
            Self::ModelId => config.llm_model_id.clone(),
            Self::MemoryModelId => config.memory_model_id.clone(),
            Self::MemorySimilarityModelId => config.memory_similarity_model_id.clone(),
            Self::DefaultChatMode => config.default_chat_mode.clone(),
            Self::VectorMemoryQdrantUrl => config.vector_memory.qdrant_url.clone(),
            Self::VectorMemoryQdrantApiKey => config
                .vector_memory
                .qdrant_api_key
                .clone()
                .unwrap_or_default(),
            Self::VectorMemoryCollectionName => config.vector_memory.collection_name.clone(),
            Self::VectorMemoryEmbeddingModel => config.vector_memory.embedding_model.clone(),
        }
    }

    pub fn set_value(&self, config: &mut VyConfig, value: String) {
        match self {
            Self::LlmApiKey => config.llm_api_key = value,
            Self::GoogleApiKey => config.google_api_key = value,
            Self::GoogleSearchEngineId => config.google_search_engine_id = value,
            Self::ModelId => config.llm_model_id = value,
            Self::MemoryModelId => config.memory_model_id = value,
            Self::MemorySimilarityModelId => config.memory_similarity_model_id = value,
            Self::DefaultChatMode => config.default_chat_mode = value,
            Self::VectorMemoryQdrantUrl => config.vector_memory.qdrant_url = value,
            Self::VectorMemoryQdrantApiKey => {
                config.vector_memory.qdrant_api_key =
                    if value.is_empty() { None } else { Some(value) };
            }
            Self::VectorMemoryCollectionName => config.vector_memory.collection_name = value,
            Self::VectorMemoryEmbeddingModel => config.vector_memory.embedding_model = value,
        }
    }

    pub fn all_keys() -> &'static [ConfigKey] {
        &[
            ConfigKey::LlmApiKey,
            ConfigKey::GoogleApiKey,
            ConfigKey::GoogleSearchEngineId,
            ConfigKey::ModelId,
            ConfigKey::MemoryModelId,
            ConfigKey::MemorySimilarityModelId,
            ConfigKey::DefaultChatMode,
            ConfigKey::VectorMemoryQdrantUrl,
            ConfigKey::VectorMemoryQdrantApiKey,
            ConfigKey::VectorMemoryCollectionName,
            ConfigKey::VectorMemoryEmbeddingModel,
        ]
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigAction {
    /// Initialize configuration with interactive prompts
    Init,
    /// Get a configuration value
    Get { key: String },
    /// Set a configuration value
    Set { key: String, value: Option<String> },
    /// List all configuration values
    List,
}

pub fn run_config<F>(action: &ConfigAction, config_path: &Path, config_loader: F) -> Result<()>
where
    F: Fn(&Path) -> Result<VyConfig>,
{
    match action {
        ConfigAction::Init => run_init(config_path),
        ConfigAction::Get { key } => run_get(key, config_path, config_loader),
        ConfigAction::Set { key, value } => {
            run_set(key, value.as_deref(), config_path, config_loader)
        }
        ConfigAction::List => run_list(config_path, config_loader),
    }
}

fn run_init(config_path: &Path) -> Result<()> {
    println!("🔧 Initializing Vy configuration...");
    println!();

    let mut config = VyConfig {
        llm_api_key: String::new(),
        google_api_key: String::new(),
        google_search_engine_id: String::new(),
        llm_model_id: "gpt-4o-mini".to_string(),
        memory_model_id: "gpt-4o-mini".to_string(),
        memory_similarity_model_id: "gpt-4o-mini".to_string(),
        system_prompt: default_system_prompt(),
        default_chat_mode: "cli".to_string(),
        vector_memory: vy_core::vector_memory::VectorMemoryConfig::default(),
    };

    // Prompt for required values (excluding vector memory keys which are handled separately)
    for key in ConfigKey::all_keys() {
        match key {
            ConfigKey::LlmApiKey => {
                println!("🤖 Enter your OpenAI API key:");
                println!("   💡 Get one at: https://platform.openai.com/api-keys");
                let value = prompt_for_value(true)?;
                key.set_value(&mut config, value);
            }
            ConfigKey::GoogleApiKey => {
                println!("\n🔍 Enter your Google API key (for web search):");
                println!("   💡 Get one at: https://console.developers.google.com/");
                let value = prompt_for_value(true)?;
                key.set_value(&mut config, value);
            }
            ConfigKey::GoogleSearchEngineId => {
                println!("\n🔍 Enter your Google Custom Search Engine ID:");
                println!("   💡 Create one at: https://cse.google.com/");
                let value = prompt_for_value(false)?;
                key.set_value(&mut config, value);
            }
            // Skip vector memory keys - handled in dedicated section below
            ConfigKey::VectorMemoryQdrantUrl
            | ConfigKey::VectorMemoryQdrantApiKey
            | ConfigKey::VectorMemoryCollectionName
            | ConfigKey::VectorMemoryEmbeddingModel => continue,
            _ => {
                // Optional fields with defaults
                println!(
                    "\n⚙️  {} (default: {}):",
                    key.as_str(),
                    key.get_value(&config)
                );
                print!("   Enter new value or press Enter to keep default: ");
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();

                if !input.is_empty() {
                    key.set_value(&mut config, input.to_string());
                }
            }
        }
    }

    // Collect vector memory configuration (now always enabled)
    println!("\n🚀 Vector Memory Configuration:");
    println!("   Vector memory provides semantic search and cloud sync capabilities.");
    println!("   For cloud setup, get credentials at: https://cloud.qdrant.io/");

    print!("   Qdrant URL (default: http://localhost:6334): ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if !input.is_empty() {
        config.vector_memory.qdrant_url = input.to_string();
    }

    print!("   Qdrant API Key (optional for local, required for cloud): ");
    std::io::stdout().flush()?;
    let api_key = prompt_for_value(true)?;
    if !api_key.is_empty() {
        config.vector_memory.qdrant_api_key = Some(api_key);
    }

    print!("   Collection name (default: vy_memories): ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if !input.is_empty() {
        config.vector_memory.collection_name = input.to_string();
    }

    // Set the OpenAI API key for embeddings (same as LLM key)
    config.vector_memory.openai_api_key = config.llm_api_key.clone();

    println!(
        "   ✅ Vector memory configured with Qdrant at: {}",
        config.vector_memory.qdrant_url
    );

    save_config(&config, config_path)?;

    println!("\n✅ Configuration saved to: {}", config_path.display());
    println!("🚀 You can now run: vy chat");

    Ok(())
}

fn run_get<F>(key: &str, config_path: &Path, config_loader: F) -> Result<()>
where
    F: Fn(&Path) -> Result<VyConfig>,
{
    let config_key =
        ConfigKey::from_key(key).ok_or_else(|| anyhow::anyhow!("Unknown config key: {}", key))?;

    let config = config_loader(config_path)?;
    let value = config_key.get_value(&config);
    let display_value = if config_key.is_sensitive() {
        mask_sensitive_value(&value)
    } else if value.is_empty() {
        "(not set)".to_string()
    } else {
        value
    };

    println!("{}: {}", config_key.as_str(), display_value);

    Ok(())
}

fn run_set<F>(key: &str, value: Option<&str>, config_path: &Path, config_loader: F) -> Result<()>
where
    F: Fn(&Path) -> Result<VyConfig>,
{
    let config_key =
        ConfigKey::from_key(key).ok_or_else(|| anyhow::anyhow!("Unknown config key: {}", key))?;

    let mut config = config_loader(config_path)?;

    let new_value = match value {
        Some(v) => v.to_string(),
        None => {
            if config_key.is_sensitive() {
                prompt_for_value(true)?
            } else {
                print!("Enter value for {}: ", config_key.as_str());
                std::io::stdout().flush()?;
                prompt_for_value(false)?
            }
        }
    };

    config_key.set_value(&mut config, new_value);
    save_config(&config, config_path)?;

    println!("✅ Updated {}", config_key.as_str());

    Ok(())
}

fn run_list<F>(config_path: &Path, config_loader: F) -> Result<()>
where
    F: Fn(&Path) -> Result<VyConfig>,
{
    let config = config_loader(config_path)?;

    println!("🔧 Configuration values:");

    for key in ConfigKey::all_keys() {
        let value = key.get_value(&config);
        let display_value = if key.is_sensitive() {
            mask_sensitive_value(&value)
        } else if value.is_empty() {
            "(not set)".to_string()
        } else {
            value
        };

        println!("  {}: {}", key.as_str(), display_value);
    }

    println!("\n📁 Config file: {}", config_path.display());

    Ok(())
}

fn prompt_for_value(is_sensitive: bool) -> Result<String> {
    print!("   > ");
    std::io::stdout().flush()?;

    if is_sensitive {
        rpassword::read_password().context("Failed to read password")
    } else {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read input")?;
        Ok(input.trim().to_string())
    }
}

fn mask_sensitive_value(value: &str) -> String {
    if value.is_empty() {
        "(not set)".to_string()
    } else if value.len() <= 8 {
        "*".repeat(value.len())
    } else {
        format!("{}...{}", &value[..4], "*".repeat(value.len() - 4))
    }
}
