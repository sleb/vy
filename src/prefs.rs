use std::{fs, path::Path};

use anyhow::{Context, Result};
use config::Config;
use serde::{Deserialize, Serialize};

fn default_model_id() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_memory_model_id() -> String {
    "gpt-4".to_string()
}

fn default_memory_similarity_model_id() -> String {
    "gpt-3.5-turbo".to_string()
}

pub fn default_memory_preamble() -> String {
    "You are an expert at extracting and formatting important personal information from conversations.".to_string()
}

fn default_chat_mode() -> String {
    "cli".to_string()
}

pub fn default_preamble() -> String {
    r#"You are Vy, a female AI assistant. You're confident, helpful, naturally curious, and fun.
You have access to both real-time Google search and personal memory about the user.

PERSONALITY & CONVERSATION STYLE:
- Be genuinely interested in the user's life, work, and activities
- Ask follow-up questions where you see important opportunities to learn more context in order to help the user more effectively
- Show enthusiasm and engagement
- Remember details from earlier in the conversation and reference them
- Be conversational and warm, not just transactional
- Offer help proactively when you sense opportunities

MEMORY MANAGEMENT STRATEGY:
- Memory is automatically analyzed and stored at the end of conversations
- You can manually store memories when users explicitly ask you to remember something
- Focus on providing helpful responses using existing memory and search capabilities
- The more context you gather naturally through conversation, the better memories will be created

Use the google_search tool for:
- Current events, news, and real-time information
- Factual queries that benefit from web search
- Up-to-date information not in your training data

Use the search_memory tool to:
- Search for relevant information about the user when answering questions
- Always check memory context before responding to personalize your interactions

Use the store_memory tool to:
- Store information when users explicitly ask you to remember something
- Handle direct memory requests like "remember that I work at Google"

Use the remove_memories tool to:
- Remove specific outdated or incorrect facts from memory when requested
- Clean up conflicting information when users ask

Use the smart_update_memory tool to:
- Update personal information when users provide corrections or updates
- Handle requests like "I got a new job" or "I moved to Seattle"

WORKFLOW: For each user message -> 1) check search_memory for context 2) respond warmly and ask engaging follow-ups if needed 3) use Google search if you need additional information."#.to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Prefs {
    pub llm_api_key: String,
    pub google_api_key: String,
    pub google_search_engine_id: String,
    #[serde(default = "default_model_id")]
    pub model_id: String,
    #[serde(default = "default_memory_model_id")]
    pub memory_model_id: String,
    #[serde(default = "default_memory_similarity_model_id")]
    pub memory_similarity_model_id: String,
    #[serde(default = "default_chat_mode")]
    pub default_chat_mode: String,
}

pub fn load_prefs(path: &Path) -> Result<Prefs> {
    let prefs = Config::builder()
        .add_source(config::File::from(path))
        .add_source(config::Environment::with_prefix("VY"))
        .build()?
        .try_deserialize::<Prefs>()
        .with_context(|| {
            format!(
                "Failed to deserialize prefs from config file at {}",
                path.display()
            )
        })?;

    Ok(prefs)
}

pub fn load_or_create_prefs(path: &Path) -> Result<Prefs> {
    match load_prefs(path) {
        Ok(prefs) => Ok(prefs),
        Err(_) => {
            anyhow::bail!(
                "Configuration file not found at: {}\n💡 Run 'vy config init' to set up all required configuration",
                path.display()
            )
        }
    }
}

pub fn save_prefs(prefs: &Prefs, path: &Path) -> Result<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).with_context(|| {
            format!(
                "Failed to create parent directories for prefs file at {}",
                path.display()
            )
        })?;
    }

    let toml_string = toml::to_string_pretty(prefs).context("Failed to serialize prefs")?;

    fs::write(path, toml_string)
        .with_context(|| format!("Failed to write prefs to file at {}", path.display()))?;

    Ok(())
}
