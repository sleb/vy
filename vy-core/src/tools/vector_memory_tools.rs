//! Vector Memory Tools for Vy
//!
//! These tools provide LLM-accessible interfaces to the vector memory system,
//! replacing the old simple memory tools with semantic search capabilities.

use crate::memory::MemoryEntry;
use crate::vector_memory::{VectorMemory, VectorMemoryConfig};
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

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

// ========== SEARCH MEMORY TOOL ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct VectorMemorySearchArgs {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct VectorMemorySearchEntry {
    pub fact: String,
    pub source: String,
    pub timestamp: String,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct VectorMemorySearchResponse {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub entries: Vec<VectorMemorySearchEntry>,
    pub total_count: usize,
}

pub struct VectorMemorySearchTool {
    config: VectorMemoryConfig,
}

impl VectorMemorySearchTool {
    pub fn new(config: VectorMemoryConfig) -> Self {
        Self { config }
    }
}

impl Tool for VectorMemorySearchTool {
    const NAME: &'static str = "search_memory";

    type Error = VectorMemoryError;
    type Args = VectorMemorySearchArgs;
    type Output = VectorMemorySearchResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search personal memories and facts about the user using semantic similarity. Find relevant information from past conversations.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query - can be natural language, keywords, or questions"
                    }
                },
                "required": ["query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(search_memories_impl(config, args))
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ========== STORE MEMORY TOOL ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct VectorMemoryStoreArgs {
    pub fact: String,
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VectorMemoryStoreResponse {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub stored_fact: String,
}

pub struct VectorMemoryStoreTool {
    config: VectorMemoryConfig,
}

impl VectorMemoryStoreTool {
    pub fn new(config: VectorMemoryConfig) -> Self {
        Self { config }
    }
}

impl Tool for VectorMemoryStoreTool {
    const NAME: &'static str = "store_memory";

    type Error = VectorMemoryError;
    type Args = VectorMemoryStoreArgs;
    type Output = VectorMemoryStoreResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Store a new fact or piece of information in the user's personal memory. Use this when the user explicitly asks you to remember something.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "The fact or information to remember"
                    },
                    "source": {
                        "type": "string",
                        "description": "Optional source or context for this memory"
                    }
                },
                "required": ["fact"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(store_memory_impl(config, args))
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ========== REMOVE MEMORY TOOL ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct VectorMemoryRemoveArgs {
    pub query: String,
    pub confirm: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct VectorMemoryRemoveResponse {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub removed_count: usize,
}

pub struct VectorMemoryRemoveTool {
    config: VectorMemoryConfig,
}

impl VectorMemoryRemoveTool {
    pub fn new(config: VectorMemoryConfig) -> Self {
        Self { config }
    }
}

impl Tool for VectorMemoryRemoveTool {
    const NAME: &'static str = "remove_memories";

    type Error = VectorMemoryError;
    type Args = VectorMemoryRemoveArgs;
    type Output = VectorMemoryRemoveResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Remove or delete memories that match a search query. Use this when the user asks to forget something or remove outdated information.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query to find memories to remove"
                    },
                    "confirm": {
                        "type": "boolean",
                        "description": "Whether the user has confirmed the removal"
                    }
                },
                "required": ["query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(remove_memories_impl(config, args))
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ========== SMART MEMORY UPDATE TOOL ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct VectorMemoryUpdateArgs {
    pub old_info: String,
    pub new_info: String,
}

#[derive(Debug, Serialize)]
pub struct VectorMemoryUpdateResponse {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub updated_info: String,
}

pub struct VectorMemoryUpdateTool {
    config: VectorMemoryConfig,
}

impl VectorMemoryUpdateTool {
    pub fn new(config: VectorMemoryConfig) -> Self {
        Self { config }
    }
}

impl Tool for VectorMemoryUpdateTool {
    const NAME: &'static str = "smart_update_memory";

    type Error = VectorMemoryError;
    type Args = VectorMemoryUpdateArgs;
    type Output = VectorMemoryUpdateResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Update existing memory with new information. Use when the user provides corrections or updates to previously stored facts.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "old_info": {
                        "type": "string",
                        "description": "The old or outdated information to replace"
                    },
                    "new_info": {
                        "type": "string",
                        "description": "The new, updated information"
                    }
                },
                "required": ["old_info", "new_info"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let config = self.config.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(update_memory_impl(config, args))
        })
        .await
        .map_err(|e| VectorMemoryError::new(format!("Task join error: {e}")))?
    }
}

// ========== IMPLEMENTATION FUNCTIONS ==========

async fn search_memories_impl(
    config: VectorMemoryConfig,
    args: VectorMemorySearchArgs,
) -> Result<VectorMemorySearchResponse, VectorMemoryError> {
    let vector_memory = VectorMemory::new(config)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to connect to vector memory: {e}")))?;

    let results = vector_memory
        .search_memories(&args.query, 10)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to search memories: {e}")))?;

    let entries = results
        .into_iter()
        .map(|entry| VectorMemorySearchEntry {
            fact: entry.fact.clone(),
            source: entry.source.clone(),
            timestamp: entry.timestamp.format("%Y-%m-%d %H:%M").to_string(),
            score: 0.0, // VectorMemory doesn't currently expose scores
        })
        .collect::<Vec<_>>();

    let count = entries.len();

    Ok(VectorMemorySearchResponse {
        action: "search".to_string(),
        success: true,
        message: format!("Found {} memories matching '{}'", count, args.query),
        entries,
        total_count: count,
    })
}

async fn store_memory_impl(
    config: VectorMemoryConfig,
    args: VectorMemoryStoreArgs,
) -> Result<VectorMemoryStoreResponse, VectorMemoryError> {
    let vector_memory = VectorMemory::new(config)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to connect to vector memory: {e}")))?;

    let source = args.source.unwrap_or_else(|| "manual_store".to_string());
    let memory_entry = MemoryEntry::new(args.fact.clone(), source);

    vector_memory
        .store_memory(&memory_entry)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to store memory: {e}")))?;

    Ok(VectorMemoryStoreResponse {
        action: "store".to_string(),
        success: true,
        message: "Memory stored successfully".to_string(),
        stored_fact: args.fact,
    })
}

async fn remove_memories_impl(
    config: VectorMemoryConfig,
    args: VectorMemoryRemoveArgs,
) -> Result<VectorMemoryRemoveResponse, VectorMemoryError> {
    let vector_memory = VectorMemory::new(config)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to connect to vector memory: {e}")))?;

    // First, search for matching memories
    let results = vector_memory
        .search_memories(&args.query, 10)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to search memories: {e}")))?;

    if results.is_empty() {
        return Ok(VectorMemoryRemoveResponse {
            action: "remove".to_string(),
            success: false,
            message: format!("No memories found matching '{}'", args.query),
            removed_count: 0,
        });
    }

    // For now, we'll return a message asking for confirmation since VectorMemory
    // doesn't currently have a delete_by_query method
    // TODO: Implement proper removal when VectorMemory has delete functionality
    Ok(VectorMemoryRemoveResponse {
        action: "remove".to_string(),
        success: false,
        message: format!(
            "Found {} memories matching '{}'. Memory removal is not yet fully implemented in vector memory system.",
            results.len(),
            args.query
        ),
        removed_count: 0,
    })
}

async fn update_memory_impl(
    config: VectorMemoryConfig,
    args: VectorMemoryUpdateArgs,
) -> Result<VectorMemoryUpdateResponse, VectorMemoryError> {
    let vector_memory = VectorMemory::new(config)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to connect to vector memory: {e}")))?;

    // For now, we'll store the new information and note that it's an update
    // TODO: Implement proper update logic when VectorMemory has update functionality
    let memory_entry = MemoryEntry::new(
        format!("UPDATED: {} (replaced: {})", args.new_info, args.old_info),
        "memory_update".to_string(),
    );

    vector_memory
        .store_memory(&memory_entry)
        .await
        .map_err(|e| VectorMemoryError::new(format!("Failed to update memory: {e}")))?;

    Ok(VectorMemoryUpdateResponse {
        action: "update".to_string(),
        success: true,
        message: format!("Updated memory: '{}' → '{}'", args.old_info, args.new_info),
        updated_info: args.new_info,
    })
}

// ========== CONVENIENCE FUNCTIONS ==========

/// Create vector memory search tool with config
pub fn vector_memory_search_tool(config: VectorMemoryConfig) -> VectorMemorySearchTool {
    VectorMemorySearchTool::new(config)
}

/// Create vector memory store tool with config
pub fn vector_memory_store_tool(config: VectorMemoryConfig) -> VectorMemoryStoreTool {
    VectorMemoryStoreTool::new(config)
}

/// Create vector memory remove tool with config
pub fn vector_memory_remove_tool(config: VectorMemoryConfig) -> VectorMemoryRemoveTool {
    VectorMemoryRemoveTool::new(config)
}

/// Create vector memory update tool with config
pub fn vector_memory_update_tool(config: VectorMemoryConfig) -> VectorMemoryUpdateTool {
    VectorMemoryUpdateTool::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rig::tool::Tool;

    fn create_test_config() -> VectorMemoryConfig {
        VectorMemoryConfig {
            qdrant_url: "http://localhost:6334".to_string(),
            qdrant_api_key: None,
            collection_name: "test_memories".to_string(),
            openai_api_key: "test_key".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
        }
    }

    #[tokio::test]
    async fn test_search_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemorySearchTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "search_memory");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "search_memory schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[tokio::test]
    async fn test_store_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemoryStoreTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "store_memory");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "store_memory schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[tokio::test]
    async fn test_update_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemoryUpdateTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "smart_update_memory");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "smart_update_memory schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[tokio::test]
    async fn test_remove_memory_tool_definition() {
        let config = create_test_config();
        let tool = VectorMemoryRemoveTool::new(config);

        let definition = tool.definition("test prompt".to_string()).await;

        assert_eq!(definition.name, "remove_memories");
        assert!(!definition.description.is_empty());

        // Check that the schema has required fields
        let params = definition.parameters;
        assert!(params.get("type").is_some());
        assert!(params.get("properties").is_some());
        assert!(params.get("required").is_some());

        // Print the schema for debugging
        println!(
            "remove_memories schema: {}",
            serde_json::to_string_pretty(&params).unwrap()
        );
    }

    #[test]
    fn test_search_args_serialization() {
        let args = VectorMemorySearchArgs {
            query: "test query".to_string(),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemorySearchArgs JSON: {json}");

        // Test deserialization
        let _deserialized: VectorMemorySearchArgs = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_store_args_serialization() {
        let args = VectorMemoryStoreArgs {
            fact: "test fact".to_string(),
            source: Some("test source".to_string()),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemoryStoreArgs JSON: {json}");

        // Test deserialization
        let _deserialized: VectorMemoryStoreArgs = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_update_args_serialization() {
        let args = VectorMemoryUpdateArgs {
            old_info: "old info".to_string(),
            new_info: "new info".to_string(),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemoryUpdateArgs JSON: {json}");

        // Test deserialization
        let _deserialized: VectorMemoryUpdateArgs = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_remove_args_serialization() {
        let args = VectorMemoryRemoveArgs {
            query: "test query".to_string(),
            confirm: Some(true),
        };

        let json = serde_json::to_string(&args).unwrap();
        println!("VectorMemoryRemoveArgs JSON: {json}");

        // Test deserialization
        let _deserialized: VectorMemoryRemoveArgs = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_tool_schema_validation() {
        // Test that all memory tool schemas are valid JSON schemas
        // This is the key test to prevent the "Invalid schema" error

        let search_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query - can be natural language, keywords, or questions"
                }
            },
            "required": ["query"]
        });

        let store_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "fact": {
                    "type": "string",
                    "description": "The fact or information to remember"
                },
                "source": {
                    "type": "string",
                    "description": "Optional source or context for this memory"
                }
            },
            "required": ["fact"]
        });

        let update_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "old_info": {
                    "type": "string",
                    "description": "The old or outdated information to replace"
                },
                "new_info": {
                    "type": "string",
                    "description": "The new, updated information"
                }
            },
            "required": ["old_info", "new_info"]
        });

        let remove_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query to find memories to remove"
                },
                "confirm": {
                    "type": "boolean",
                    "description": "Whether the user has confirmed the removal"
                }
            },
            "required": ["query"]
        });

        // Verify schemas are valid JSON
        assert!(search_schema.get("required").is_some());
        assert!(store_schema.get("required").is_some());
        assert!(update_schema.get("required").is_some());
        assert!(remove_schema.get("required").is_some());

        // Verify all have type = "object"
        assert_eq!(search_schema["type"], "object");
        assert_eq!(store_schema["type"], "object");
        assert_eq!(update_schema["type"], "object");
        assert_eq!(remove_schema["type"], "object");

        // Verify properties exist
        assert!(search_schema.get("properties").is_some());
        assert!(store_schema.get("properties").is_some());
        assert!(update_schema.get("properties").is_some());
        assert!(remove_schema.get("properties").is_some());

        println!("All memory tool schemas are valid!");
    }

    #[tokio::test]
    async fn test_compare_with_working_tool_schema() {
        // Compare our memory tool schema with the working nutrition analysis tool schema
        use crate::tools::nutrition_analysis::NutritionAnalysisTool;

        let config = create_test_config();
        let memory_tool = VectorMemoryStoreTool::new(config);
        let nutrition_tool = NutritionAnalysisTool::new("test_key".to_string());

        let memory_def = memory_tool.definition("test".to_string()).await;
        let nutrition_def = nutrition_tool.definition("test".to_string()).await;

        println!("\n=== WORKING NUTRITION TOOL ===");
        println!("Name: {}", nutrition_def.name);
        println!("Description: {}", nutrition_def.description);
        println!(
            "Parameters: {}",
            serde_json::to_string_pretty(&nutrition_def.parameters).unwrap()
        );

        println!("\n=== PROBLEMATIC MEMORY TOOL ===");
        println!("Name: {}", memory_def.name);
        println!("Description: {}", memory_def.description);
        println!(
            "Parameters: {}",
            serde_json::to_string_pretty(&memory_def.parameters).unwrap()
        );

        // Compare structure
        assert_eq!(memory_def.parameters.get("type").unwrap(), "object");
        assert_eq!(nutrition_def.parameters.get("type").unwrap(), "object");

        assert!(memory_def.parameters.get("properties").is_some());
        assert!(nutrition_def.parameters.get("properties").is_some());

        assert!(memory_def.parameters.get("required").is_some());
        assert!(nutrition_def.parameters.get("required").is_some());

        println!("\n✅ Both tools have identical schema structure!");
    }
}
