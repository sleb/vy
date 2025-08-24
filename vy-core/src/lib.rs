//! Vy Core - The heart of the Vy AI chatbot
//!
//! This crate provides the core functionality for the Vy chatbot, including:
//! - Chat agent management and conversation handling
//! - Memory system for persistent learning
//! - Tool integration (search, etc.)
//! - Configuration and preferences management
//!
//! This core can be used by different interface crates (CLI, TUI, web, mobile, etc.)

use anyhow::Result;
use rig::agent::Agent;
use rig::completion::{Message, Prompt, request::CompletionModel};

pub mod config;
pub mod memory;
pub mod tools;
pub mod vector_memory;

// Re-export memory types from vector_memory for compatibility
pub use crate::memory::MemoryEntry;
pub use vector_memory::{VectorMemory, VectorMemoryConfig};

/// The core Vy chatbot engine
///
/// This struct contains all the core functionality needed to run a Vy chatbot
/// instance, independent of the user interface.
pub struct VyCore<M: CompletionModel> {
    agent: Agent<M>,
    conversation_history: Vec<Message>,
    model_id: String,
    memory_model_id: String,
}

impl<M: CompletionModel> VyCore<M> {
    /// Create a new Vy core instance
    pub fn new(agent: Agent<M>, model_id: String, memory_model_id: String) -> Self {
        Self {
            agent,
            conversation_history: Vec::new(),
            model_id,
            memory_model_id,
        }
    }

    /// Get the model ID
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Get the conversation history
    pub fn conversation_history(&self) -> &[Message] {
        &self.conversation_history
    }

    /// Clear the conversation history
    pub fn clear_history(&mut self) -> usize {
        let count = self.conversation_history.len();
        self.conversation_history.clear();
        count
    }

    /// Send a message and get a response
    pub async fn send_message(&mut self, input: &str) -> Result<String> {
        // Add user message to history
        self.conversation_history.push(Message::user(input));

        // Get response from agent with error handling
        match self
            .agent
            .prompt(input)
            .multi_turn(5)
            .with_history(&mut self.conversation_history)
            .await
        {
            Ok(response) => {
                // Add assistant response to history
                self.conversation_history
                    .push(Message::assistant(&response));
                Ok(response)
            }
            Err(e) => {
                // Remove the user message if we failed to get a response
                self.conversation_history.pop();
                Err(e.into())
            }
        }
    }

    /// Analyze the conversation for memory-worthy information using vector memory
    pub async fn analyze_conversation_memories(&self, vector_memory: &VectorMemory) -> Result<()> {
        if self.conversation_history.is_empty() {
            log::debug!("No conversation history to analyze");
            return Ok(());
        }

        log::debug!("Analyzing conversation for important information...");

        // Collect all user messages from the conversation
        let user_messages: Vec<String> = self
            .conversation_history
            .iter()
            .filter_map(|msg| {
                match msg {
                    Message::User { content } => {
                        // Extract text from OneOrMany content using debug format
                        let debug_str = format!("{content:?}");
                        // Parse the text content from the debug string
                        // Format is typically: OneOrMany { first: Text(Text { text: "actual_text" }), rest: [] }
                        if let Some(start) = debug_str.find("text: \"") {
                            let start_idx = start + 7; // length of "text: \""
                            if let Some(end) = debug_str[start_idx..].find("\" }") {
                                let text = &debug_str[start_idx..start_idx + end];
                                return Some(text.to_string());
                            }
                        }
                        None
                    }
                    _ => None,
                }
            })
            .collect();

        // Combine all user messages into one analysis to avoid duplicates
        if user_messages.is_empty() {
            log::debug!("No user messages to analyze");
            return Ok(());
        }

        let combined_conversation = user_messages.join(" ");
        let conversation_id = format!(
            "conversation_{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );

        // Skip very short conversations or pure command conversations
        if combined_conversation.len() < 10
            || combined_conversation
                .chars()
                .all(|c| c.is_ascii_punctuation() || c.is_whitespace())
        {
            log::debug!("Conversation too short or contains no meaningful content");
            return Ok(());
        }

        // Use vector memory's conversation learning capabilities
        match vector_memory
            .learn_from_conversation(
                &combined_conversation,
                &conversation_id,
                &self.memory_model_id,
            )
            .await
        {
            Ok(facts) => {
                if !facts.is_empty() {
                    log::debug!(
                        "Analyzed {} message(s) from this conversation, stored {} memories",
                        user_messages.len(),
                        facts.len()
                    );
                } else {
                    log::debug!("No new memorable information found");
                }
            }
            Err(e) => {
                log::debug!("Failed to process conversation: {e}");
            }
        }

        Ok(())
    }
}

/// Format an error for user display
pub fn format_error(e: &dyn std::error::Error) -> String {
    let error_str = e.to_string();

    // Make common errors more user-friendly
    if error_str.contains("invalid_api_key") {
        "🔑 Invalid API key detected!\n   Run: vy config set llm_api_key\n   Then paste your OpenAI API key when prompted."
            .to_string()
    } else if error_str.contains("insufficient_quota") {
        "💳 API quota exceeded. Please check your OpenAI billing and usage.".to_string()
    } else if error_str.contains("rate_limit_exceeded") {
        "⏱️ Rate limit exceeded. Please wait a moment and try again.".to_string()
    } else if error_str.contains("network") || error_str.contains("connection") {
        "🌐 Network error. Please check your internet connection.".to_string()
    } else {
        // Truncate very long error messages
        if error_str.len() > 150 {
            format!("{}...", &error_str[..150])
        } else {
            error_str
        }
    }
}

/// Utility functions for creating Vy instances with common configurations
pub mod builder {
    use super::*;
    use crate::config::VyConfig;
    use crate::vector_memory::{VectorMemory, VectorMemoryConfig};
    use rig::client::completion::CompletionClientDyn;
    use rig::providers::{anthropic::Client as AnthropicClient, openai::Client as OpenAIClient};

    /// Build a Vy instance with OpenAI
    pub async fn build_openai_vy(config: &VyConfig) -> Result<VyCore<impl CompletionModel>> {
        let client = OpenAIClient::builder(&config.llm_api_key).build()?;

        let mut agent_builder = CompletionClientDyn::agent(&client, &config.llm_model_id)
            .preamble(&config.system_prompt);

        // Add tools based on model compatibility
        if config.llm_model_id != "gpt-5-mini" {
            agent_builder = agent_builder.tool(crate::tools::google_search(
                config.google_api_key.clone(),
                config.google_search_engine_id.clone(),
            ));
        }

        // Add vector memory tools
        let vector_memory_config = config.vector_memory.clone();

        let agent = agent_builder
            .tool(crate::tools::nutrition_analysis_tool(
                config.llm_api_key.clone(),
            ))
            .tool(crate::tools::vector_memory_search_tool(
                vector_memory_config.clone(),
            ))
            .tool(crate::tools::vector_memory_store_tool(
                vector_memory_config.clone(),
            ))
            .tool(crate::tools::vector_memory_update_tool(
                vector_memory_config.clone(),
            ))
            .tool(crate::tools::vector_memory_remove_tool(
                vector_memory_config.clone(),
            ))
            .build();

        Ok(VyCore::new(
            agent,
            config.llm_model_id.clone(),
            config.memory_model_id.clone(),
        ))
    }

    /// Build a Vy instance with Anthropic
    pub async fn build_anthropic_vy(config: &VyConfig) -> Result<VyCore<impl CompletionModel>> {
        let client = AnthropicClient::new(&config.llm_api_key);

        // Add vector memory tools
        let vector_memory_config = config.vector_memory.clone();

        let agent = CompletionClientDyn::agent(&client, &config.llm_model_id)
            .preamble(&config.system_prompt)
            .tool(crate::tools::google_search(
                config.google_api_key.clone(),
                config.google_search_engine_id.clone(),
            ))
            .tool(crate::tools::nutrition_analysis_tool(
                config.llm_api_key.clone(),
            ))
            .tool(crate::tools::vector_memory_search_tool(
                vector_memory_config.clone(),
            ))
            .tool(crate::tools::vector_memory_store_tool(
                vector_memory_config.clone(),
            ))
            .tool(crate::tools::vector_memory_update_tool(
                vector_memory_config.clone(),
            ))
            .tool(crate::tools::vector_memory_remove_tool(
                vector_memory_config.clone(),
            ))
            .build();

        Ok(VyCore::new(
            agent,
            config.llm_model_id.clone(),
            config.memory_model_id.clone(),
        ))
    }

    /// Example function showing how to use vector memory directly
    pub async fn example_vector_memory_usage() -> Result<()> {
        let vector_config = VectorMemoryConfig {
            qdrant_url: "http://localhost:6334".to_string(),
            qdrant_api_key: None, // For local instance
            collection_name: "vy_memories".to_string(),
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            embedding_model: "text-embedding-3-small".to_string(),
        };

        let vector_memory = VectorMemory::new(vector_config).await?;

        // Example: Store a memory
        let memory_entry = crate::memory::MemoryEntry::new(
            "User works as a software engineer at Google".to_string(),
            "conversation_example".to_string(),
        );
        vector_memory.store_memory(&memory_entry).await?;

        // Example: Search for memories
        let results = vector_memory
            .search_memories("software engineer job", 5)
            .await?;

        for memory in results {
            println!("Found memory: {}", memory.fact);
        }

        Ok(())
    }
}
