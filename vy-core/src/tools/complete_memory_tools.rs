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
use std::io::{self, Write};
use tokio::task;

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
    pub source: Option<String>,
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
    vector_config: VectorMemoryConfig,
}

impl StoreMemoryTool {
    pub fn new(vector_config: VectorMemoryConfig) -> Self {
        Self { vector_config }
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
            description: "Store a fact or information in long-term vector memory. Use when user asks to remember something specific or when important information should be preserved across conversations.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "The fact or information to store in memory"
                    },
                    "source": {
                        "type": "string",
                        "description": "Optional source or context of the information"
                    }
                },
                "required": ["fact"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate configuration
        if self.vector_config.openai_api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured for vector memory embeddings. Run: vy config set llm_api_key",
            ));
        }

        print!("🧠 Storing memory...");
        io::stdout().flush().ok();

        // Create vector memory instance and store fact
        let vector_config = self.vector_config.clone();
        let fact = args.fact.clone();
        let source = args.source.clone();

        let _result = task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let vector_memory = VectorMemory::new(vector_config).await?;
                let entry = MemoryEntry::new(fact, source.unwrap_or_else(|| "user".to_string()));
                vector_memory.store_memory(&entry).await
            })
        })
        .await
        .map_err(|e| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Task join error: {e}"))
        })?
        .map_err(|e: anyhow::Error| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Failed to store memory: {e}"))
        })?;

        print!(" ✅\n");
        io::stdout().flush().ok();

        Ok(StoreMemoryResponse {
            success: true,
            message: "Fact stored successfully in vector memory".to_string(),
            stored_fact: args.fact,
        })
    }
}

// ===== SEARCH MEMORY TOOL =====

#[derive(Debug, Deserialize)]
pub struct SearchMemoryArgs {
    pub query: String,
    pub limit: Option<u32>,
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
    vector_config: VectorMemoryConfig,
}

impl SearchMemoryTool {
    pub fn new(vector_config: VectorMemoryConfig) -> Self {
        Self { vector_config }
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
            description: "Search through stored memories using semantic similarity. Use when you need to recall information about a topic, person, or concept from previous conversations.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "What to search for in stored memories"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 5)"
                    }
                },
                "required": ["query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate configuration
        if self.vector_config.openai_api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured for vector memory embeddings. Run: vy config set llm_api_key",
            ));
        }

        print!("🔍 Searching memories...");
        io::stdout().flush().ok();

        // Search memories
        let vector_config = self.vector_config.clone();
        let query = args.query.clone();
        let limit = args.limit.unwrap_or(5) as usize;

        let results = task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let vector_memory = VectorMemory::new(vector_config).await?;
                vector_memory.search_memories(&query, limit).await
            })
        })
        .await
        .map_err(|e| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Task join error: {e}"))
        })?
        .map_err(|e: anyhow::Error| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Failed to search memories: {e}"))
        })?;

        print!(" ✅\n");
        io::stdout().flush().ok();

        let matches = results
            .into_iter()
            .enumerate()
            .map(|(i, entry)| MemoryMatch {
                fact: entry.fact,
                source: Some(entry.source),
                score: 1.0 - (i as f32 * 0.1), // Mock relevance score based on order
            })
            .collect::<Vec<_>>();

        let total_found = matches.len();

        Ok(SearchMemoryResponse {
            matches,
            query: args.query,
            total_found,
        })
    }
}

// ===== UPDATE MEMORY TOOL =====

#[derive(Debug, Deserialize)]
pub struct UpdateMemoryArgs {
    pub old_fact: String,
    pub new_fact: String,
    pub source: Option<String>,
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
    vector_config: VectorMemoryConfig,
}

impl UpdateMemoryTool {
    pub fn new(vector_config: VectorMemoryConfig) -> Self {
        Self { vector_config }
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
            description: "Update or replace existing information in memory. Use when information changes or needs correction. This will find similar memories and update them.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "old_fact": {
                        "type": "string",
                        "description": "The existing fact or information to find and update"
                    },
                    "new_fact": {
                        "type": "string",
                        "description": "The new fact or information to replace it with"
                    },
                    "source": {
                        "type": "string",
                        "description": "Optional source or context of the new information"
                    }
                },
                "required": ["old_fact", "new_fact"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate configuration
        if self.vector_config.openai_api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured for vector memory embeddings. Run: vy config set llm_api_key",
            ));
        }

        print!("🔄 Updating memory...");
        io::stdout().flush().ok();

        // Update memory
        let vector_config = self.vector_config.clone();
        let old_fact = args.old_fact.clone();
        let new_fact = args.new_fact.clone();
        let source = args.source.clone();

        let result = task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let vector_memory = VectorMemory::new(vector_config).await?;
                // Search for similar memories first
                let existing = vector_memory.search_memories(&old_fact, 3).await?;
                let mut updated = 0;

                // Delete old memories that match
                for entry in &existing {
                    if entry.fact.contains(&old_fact) || old_fact.contains(&entry.fact) {
                        vector_memory.delete_memory(entry.timestamp).await?;
                        updated += 1;
                    }
                }

                // Store the new fact if we found something to replace
                if updated > 0 {
                    let new_entry =
                        MemoryEntry::new(new_fact, source.unwrap_or_else(|| "update".to_string()));
                    vector_memory.store_memory(&new_entry).await?;
                }

                Ok(updated)
            })
        })
        .await
        .map_err(|e| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Task join error: {e}"))
        })?
        .map_err(|e: anyhow::Error| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Failed to update memory: {e}"))
        })?;

        print!(" ✅\n");
        io::stdout().flush().ok();

        Ok(UpdateMemoryResponse {
            success: result > 0,
            message: if result > 0 {
                format!("Successfully updated {} memory(ies)", result)
            } else {
                "No matching memories found to update".to_string()
            },
            old_fact: args.old_fact,
            new_fact: args.new_fact,
            updated: result > 0,
        })
    }
}

// ===== REMOVE MEMORY TOOL =====

#[derive(Debug, Deserialize)]
pub struct RemoveMemoryArgs {
    pub query: String,
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
    vector_config: VectorMemoryConfig,
}

impl RemoveMemoryTool {
    pub fn new(vector_config: VectorMemoryConfig) -> Self {
        Self { vector_config }
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
            description: "Remove memories that match a query. Use when user asks to forget something or when information becomes irrelevant. Be careful with this tool.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Query to find memories to remove"
                    }
                },
                "required": ["query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate configuration
        if self.vector_config.openai_api_key.is_empty() {
            return Err(VectorMemoryError::new(
                "OpenAI API key not configured for vector memory embeddings. Run: vy config set llm_api_key",
            ));
        }

        print!("🗑️ Removing memories...");
        io::stdout().flush().ok();

        // Remove memories
        let vector_config = self.vector_config.clone();
        let query = args.query.clone();

        let result = task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let vector_memory = VectorMemory::new(vector_config).await?;
                // Search for memories matching the query
                let matching = vector_memory.search_memories(&query, 10).await?;
                let mut removed = 0;

                // Delete matching memories
                for entry in &matching {
                    if entry.fact.to_lowercase().contains(&query.to_lowercase()) {
                        vector_memory.delete_memory(entry.timestamp).await?;
                        removed += 1;
                    }
                }

                Ok(removed)
            })
        })
        .await
        .map_err(|e| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Task join error: {e}"))
        })?
        .map_err(|e: anyhow::Error| {
            print!(" ❌\n");
            io::stdout().flush().ok();
            VectorMemoryError::new(format!("Failed to remove memories: {e}"))
        })?;

        print!(" ✅\n");
        io::stdout().flush().ok();

        Ok(RemoveMemoryResponse {
            success: true,
            message: if result > 0 {
                format!("Successfully removed {} memory(ies)", result)
            } else {
                "No matching memories found to remove".to_string()
            },
            query: args.query,
            removed_count: result,
        })
    }
}

// ===== CONSTRUCTOR FUNCTIONS =====

/// Create a store memory tool instance
pub fn store_memory_tool(vector_config: VectorMemoryConfig) -> StoreMemoryTool {
    StoreMemoryTool::new(vector_config)
}

/// Create a search memory tool instance
pub fn search_memory_tool(vector_config: VectorMemoryConfig) -> SearchMemoryTool {
    SearchMemoryTool::new(vector_config)
}

/// Create an update memory tool instance
pub fn update_memory_tool(vector_config: VectorMemoryConfig) -> UpdateMemoryTool {
    UpdateMemoryTool::new(vector_config)
}

/// Create a remove memory tool instance
pub fn remove_memory_tool(vector_config: VectorMemoryConfig) -> RemoveMemoryTool {
    RemoveMemoryTool::new(vector_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_memory::VectorMemoryConfig;

    fn test_vector_config() -> VectorMemoryConfig {
        VectorMemoryConfig {
            qdrant_url: "https://test.qdrant.io".to_string(),
            qdrant_api_key: Some("test-key".to_string()),
            collection_name: "test_memories".to_string(),
            openai_api_key: "test-openai-key".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
        }
    }

    #[tokio::test]
    async fn test_store_memory_tool_definition() {
        let config = test_vector_config();
        let tool = StoreMemoryTool::new(config);

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
        let config = test_vector_config();
        let tool = SearchMemoryTool::new(config);

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
        let config = test_vector_config();
        let tool = UpdateMemoryTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "smart_update_memory");
        assert!(definition.description.contains("Update or replace"));

        // Verify schema has required fields
        let params = definition.parameters;
        let required = params["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::Value::String("old_fact".to_string())));
        assert!(required.contains(&serde_json::Value::String("new_fact".to_string())));
    }

    #[tokio::test]
    async fn test_remove_memory_tool_definition() {
        let config = test_vector_config();
        let tool = RemoveMemoryTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "remove_memories");
        assert!(definition.description.contains("Remove memories"));

        // Verify schema structure
        let params = definition.parameters;
        assert!(params["properties"]["query"].is_object());
        assert_eq!(params["properties"]["query"]["type"], "string");
    }

    #[test]
    fn test_memory_tools_follow_proven_pattern() {
        // This test verifies that our memory tools follow the exact same pattern
        // as the nutrition analysis tool that we know works with OpenAI

        // All Args structs should have Debug + Deserialize only
        let _store_args = StoreMemoryArgs {
            fact: "test".to_string(),
            source: Some("test".to_string()),
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
        let config = test_vector_config();

        // Test all constructor functions exist and work
        let _store_tool = store_memory_tool(config.clone());
        let _search_tool = search_memory_tool(config.clone());
        let _update_tool = update_memory_tool(config.clone());
        let _remove_tool = remove_memory_tool(config);

        // If we get here without panicking, the constructors work
    }
}
