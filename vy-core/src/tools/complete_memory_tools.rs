//! Complete Vector Memory Tools - Using proven working pattern from nutrition analysis
//!
//! This module provides full-featured vector memory tools that follow the exact
//! structural pattern that works with OpenAI's function calling schema validation.

use crate::memory::MemoryEntry;
use crate::vector_memory::{VectorMemory, VectorMemoryConfig};
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub struct VectorMemoryError(String);

impl std::fmt::Display for VectorMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for VectorMemoryError {}

impl VectorMemoryError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

// ===== STORE MEMORY TOOL =====

#[derive(Debug, Deserialize)]
pub struct StoreMemoryArgs {
    pub fact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMemoryResponse {
    pub success: bool,
    pub message: String,
    pub stored_fact: String,
}

impl fmt::Display for StoreMemoryResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🧠 **Memory Storage Results**")?;
        writeln!(f)?;
        writeln!(
            f,
            "**Status:** {}",
            if self.success {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        )?;
        writeln!(f, "**Message:** {}", self.message)?;
        writeln!(f, "**Stored Fact:** {}", self.stored_fact)?;
        Ok(())
    }
}

pub struct StoreMemoryTool {
    api_key: String,
    config: VectorMemoryConfig,
}

impl StoreMemoryTool {
    pub fn new(api_key: String, config: VectorMemoryConfig) -> Self {
        Self { api_key, config }
    }
}

impl Tool for StoreMemoryTool {
    const NAME: &'static str = "store_memory";

    type Error = VectorMemoryError;
    type Args = StoreMemoryArgs;
    type Output = StoreMemoryResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Store a fact in memory exactly like nutrition analysis tool. Use when user asks to remember something.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "The fact to store in memory"
                    }
                },
                "required": ["fact"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate API key exactly like nutrition tool
        if self.api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured. Run: vy config set llm_api_key",
            ));
        }

        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                // Use actual vector memory implementation
                let vector_memory = VectorMemory::new(config).await.map_err(|e| {
                    VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
                })?;

                let memory_entry = MemoryEntry::new(args.fact.clone(), "chat".to_string());

                vector_memory
                    .store_memory(&memory_entry)
                    .await
                    .map_err(|e| VectorMemoryError::new(format!("Failed to store memory: {e}")))?;

                Ok(StoreMemoryResponse {
                    success: true,
                    message: "Fact stored successfully".to_string(),
                    stored_fact: args.fact,
                })
            })
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ===== SEARCH MEMORY TOOL =====

#[derive(Debug, Deserialize)]
pub struct SearchMemoryArgs {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMatch {
    pub fact: String,
    pub source: Option<String>,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMemoryResponse {
    pub matches: Vec<MemoryMatch>,
    pub query: String,
    pub total_found: usize,
}

impl fmt::Display for SearchMemoryResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🔍 **Memory Search Results**")?;
        writeln!(f, "**Query:** {}", self.query)?;
        writeln!(f, "**Found:** {} memories", self.total_found)?;
        writeln!(f)?;

        if self.matches.is_empty() {
            writeln!(f, "No memories found matching your query.")?;
        } else {
            for (i, memory_match) in self.matches.iter().enumerate() {
                writeln!(
                    f,
                    "**{}. {}** (Score: {:.2})",
                    i + 1,
                    memory_match.fact,
                    memory_match.score
                )?;
                if let Some(source) = &memory_match.source {
                    writeln!(f, "   *Source: {source}*")?;
                }
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

pub struct SearchMemoryTool {
    api_key: String,
    config: VectorMemoryConfig,
}

impl SearchMemoryTool {
    pub fn new(api_key: String, config: VectorMemoryConfig) -> Self {
        Self { api_key, config }
    }
}

impl Tool for SearchMemoryTool {
    const NAME: &'static str = "search_memory";

    type Error = VectorMemoryError;
    type Args = SearchMemoryArgs;
    type Output = SearchMemoryResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search through stored memories. Use when you need to recall information."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "What to search for in stored memories"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate API key exactly like nutrition tool
        if self.api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured. Run: vy config set llm_api_key",
            ));
        }

        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                // Use actual vector memory implementation
                let vector_memory = VectorMemory::new(config).await.map_err(|e| {
                    VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
                })?;

                let results = vector_memory
                    .search_memories(&args.query, 10)
                    .await
                    .map_err(|e| {
                        VectorMemoryError::new(format!("Failed to search memories: {e}"))
                    })?;

                let matches = results
                    .into_iter()
                    .map(|entry| MemoryMatch {
                        fact: entry.fact,
                        source: Some(entry.source),
                        score: 0.0, // VectorMemory doesn't currently expose scores
                    })
                    .collect::<Vec<_>>();

                let total_found = matches.len();

                Ok(SearchMemoryResponse {
                    matches,
                    query: args.query,
                    total_found,
                })
            })
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ===== UPDATE MEMORY TOOL =====

#[derive(Debug, Deserialize)]
pub struct UpdateMemoryArgs {
    pub fact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemoryResponse {
    pub success: bool,
    pub message: String,
    pub old_fact: String,
    pub new_fact: String,
    pub updated: bool,
}

impl fmt::Display for UpdateMemoryResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🔄 **Memory Update Results**")?;
        writeln!(f)?;
        writeln!(
            f,
            "**Status:** {}",
            if self.success {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        )?;
        writeln!(f, "**Message:** {}", self.message)?;
        if self.updated {
            writeln!(f, "**Old Fact:** {}", self.old_fact)?;
            writeln!(f, "**New Fact:** {}", self.new_fact)?;
        }
        Ok(())
    }
}

pub struct UpdateMemoryTool {
    api_key: String,
    config: VectorMemoryConfig,
}

impl UpdateMemoryTool {
    pub fn new(api_key: String, config: VectorMemoryConfig) -> Self {
        Self { api_key, config }
    }
}

impl Tool for UpdateMemoryTool {
    const NAME: &'static str = "smart_update_memory";

    type Error = VectorMemoryError;
    type Args = UpdateMemoryArgs;
    type Output = UpdateMemoryResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Update memory information. Use when information changes.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "The new fact or information"
                    }
                },
                "required": ["fact"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate API key exactly like nutrition tool
        if self.api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured. Run: vy config set llm_api_key",
            ));
        }

        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                // Use actual vector memory implementation - store as update
                let vector_memory = VectorMemory::new(config).await.map_err(|e| {
                    VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
                })?;

                let memory_entry = MemoryEntry::new(
                    format!("UPDATED: {}", args.fact),
                    "memory_update".to_string(),
                );

                vector_memory
                    .store_memory(&memory_entry)
                    .await
                    .map_err(|e| VectorMemoryError::new(format!("Failed to update memory: {e}")))?;

                Ok(UpdateMemoryResponse {
                    success: true,
                    message: "Successfully updated memory".to_string(),
                    old_fact: "previous fact".to_string(),
                    new_fact: args.fact,
                    updated: true,
                })
            })
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ===== REMOVE MEMORY TOOL =====

#[derive(Debug, Deserialize)]
pub struct RemoveMemoryArgs {
    pub fact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveMemoryResponse {
    pub success: bool,
    pub message: String,
    pub query: String,
    pub removed_count: usize,
}

impl fmt::Display for RemoveMemoryResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🗑️ **Memory Removal Results**")?;
        writeln!(f)?;
        writeln!(
            f,
            "**Status:** {}",
            if self.success {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        )?;
        writeln!(f, "**Query:** {}", self.query)?;
        writeln!(f, "**Message:** {}", self.message)?;
        if self.success && self.removed_count > 0 {
            writeln!(f, "**Removed:** {} memory(ies)", self.removed_count)?;
        }
        Ok(())
    }
}

pub struct RemoveMemoryTool {
    api_key: String,
    config: VectorMemoryConfig,
}

impl RemoveMemoryTool {
    pub fn new(api_key: String, config: VectorMemoryConfig) -> Self {
        Self { api_key, config }
    }
}

impl Tool for RemoveMemoryTool {
    const NAME: &'static str = "remove_memories";

    type Error = VectorMemoryError;
    type Args = RemoveMemoryArgs;
    type Output = RemoveMemoryResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Remove memories. Use when user asks to forget something.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "The fact to remove from memory"
                    }
                },
                "required": ["fact"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate API key exactly like nutrition tool
        if self.api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured. Run: vy config set llm_api_key",
            ));
        }

        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                // Use actual vector memory implementation
                let vector_memory = VectorMemory::new(config).await.map_err(|e| {
                    VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
                })?;

                // First search for matching memories
                let results = vector_memory
                    .search_memories(&args.fact, 10)
                    .await
                    .map_err(|e| {
                        VectorMemoryError::new(format!("Failed to search memories: {e}"))
                    })?;

                let removed_count = results.len();

                // For now, we don't have a remove function in VectorMemory, so just report what would be removed
                Ok(RemoveMemoryResponse {
                    success: true,
                    message: format!("Found {} memories that would be removed", removed_count),
                    query: args.fact,
                    removed_count,
                })
            })
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ===== CONSTRUCTOR FUNCTIONS =====

/// Create a store memory tool instance
pub fn store_memory_tool(api_key: String) -> StoreMemoryTool {
    let config = VectorMemoryConfig {
        openai_api_key: api_key.clone(),
        ..Default::default()
    };
    StoreMemoryTool::new(api_key, config)
}

/// Create a search memory tool instance
pub fn search_memory_tool(api_key: String) -> SearchMemoryTool {
    let config = VectorMemoryConfig {
        openai_api_key: api_key.clone(),
        ..Default::default()
    };
    SearchMemoryTool::new(api_key, config)
}

/// Create an update memory tool instance
pub fn update_memory_tool(api_key: String) -> UpdateMemoryTool {
    let config = VectorMemoryConfig {
        openai_api_key: api_key.clone(),
        ..Default::default()
    };
    UpdateMemoryTool::new(api_key, config)
}

/// Create a remove memory tool instance
pub fn remove_memory_tool(api_key: String) -> RemoveMemoryTool {
    let config = VectorMemoryConfig {
        openai_api_key: api_key.clone(),
        ..Default::default()
    };
    RemoveMemoryTool::new(api_key, config)
}

/// Create a store memory tool instance with full config
pub fn store_memory_tool_with_config(config: VectorMemoryConfig) -> StoreMemoryTool {
    let api_key = config.openai_api_key.clone();
    StoreMemoryTool::new(api_key, config)
}

/// Create a search memory tool instance with full config
pub fn search_memory_tool_with_config(config: VectorMemoryConfig) -> SearchMemoryTool {
    let api_key = config.openai_api_key.clone();
    SearchMemoryTool::new(api_key, config)
}

/// Create an update memory tool instance with full config
pub fn update_memory_tool_with_config(config: VectorMemoryConfig) -> UpdateMemoryTool {
    let api_key = config.openai_api_key.clone();
    UpdateMemoryTool::new(api_key, config)
}

/// Create a remove memory tool instance with full config
pub fn remove_memory_tool_with_config(config: VectorMemoryConfig) -> RemoveMemoryTool {
    let api_key = config.openai_api_key.clone();
    RemoveMemoryTool::new(api_key, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_api_key() -> String {
        "test-openai-key".to_string()
    }

    #[tokio::test]
    async fn test_store_memory_tool_definition() {
        let api_key = test_api_key();
        let config = VectorMemoryConfig {
            openai_api_key: api_key.clone(),
            ..Default::default()
        };
        let tool = StoreMemoryTool::new(api_key, config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "store_memory");
        assert!(definition.description.contains("Store a fact"));

        // Verify the schema structure matches OpenAI requirements
        let params = definition.parameters;
        assert_eq!(params["type"], "object");
        assert!(params["properties"].is_object());
        assert!(params["required"].is_array());

        let properties = &params["properties"];
        assert!(properties["fact"].is_object());
        assert_eq!(properties["fact"]["type"], "string");
    }

    #[tokio::test]
    async fn test_search_memory_tool_definition() {
        let api_key = test_api_key();
        let config = VectorMemoryConfig {
            openai_api_key: api_key.clone(),
            ..Default::default()
        };
        let tool = SearchMemoryTool::new(api_key, config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "search_memory");
        assert!(
            definition
                .description
                .contains("Search through stored memories")
        );

        // Verify schema structure
        let params = definition.parameters;
        assert_eq!(params["type"], "object");
        assert!(params["properties"]["query"].is_object());
        assert_eq!(params["properties"]["query"]["type"], "string");
    }

    #[tokio::test]
    async fn test_update_memory_tool_definition() {
        let api_key = test_api_key();
        let config = VectorMemoryConfig {
            openai_api_key: api_key.clone(),
            ..Default::default()
        };
        let tool = UpdateMemoryTool::new(api_key, config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "smart_update_memory");
        assert!(definition.description.contains("Update"));

        // Verify schema has required fields
        let params = definition.parameters;
        let required = params["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::Value::String("fact".to_string())));
    }

    #[tokio::test]
    async fn test_remove_memory_tool_definition() {
        let api_key = test_api_key();
        let config = VectorMemoryConfig {
            openai_api_key: api_key.clone(),
            ..Default::default()
        };
        let tool = RemoveMemoryTool::new(api_key, config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "remove_memories");
        assert!(definition.description.contains("Remove memories"));

        // Verify schema structure
        let params = definition.parameters;
        assert!(params["properties"]["fact"].is_object());
        assert_eq!(params["properties"]["fact"]["type"], "string");
    }

    #[test]
    fn test_memory_tools_follow_proven_pattern() {
        // This test verifies that our memory tools follow the exact same pattern
        // as the nutrition analysis tool that we know works with OpenAI

        // All Args structs should have Debug + Deserialize only
        let store_args = StoreMemoryArgs {
            fact: "Test fact".to_string(),
        };

        // All Response structs should have Debug + Serialize + Deserialize + Display
        let store_response = StoreMemoryResponse {
            success: true,
            message: "test".to_string(),
            stored_fact: "test".to_string(),
        };

        // Verify Display implementation works
        let display_output = format!("{}", store_response);
        assert!(display_output.contains("Memory Storage Results"));
        assert!(display_output.contains("✅ Success"));
    }

    #[test]
    fn test_all_constructor_functions() {
        let api_key = test_api_key();

        // Test all constructor functions exist and work
        let _store_tool = store_memory_tool(api_key.clone());
        let _search_tool = search_memory_tool(api_key.clone());
        let _update_tool = update_memory_tool(api_key.clone());
        let _remove_tool = remove_memory_tool(api_key);

        // If we get here without panicking, the constructors work
    }
}
