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

#[derive(Debug, Deserialize)]
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
    pub async fn new(config: VectorMemoryConfig) -> Result<Self> {
        Ok(Self { config })
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
        let vector_memory = VectorMemory::new(self.config.clone()).await.map_err(|e| {
            VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
        })?;

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
}

// ========== STORE MEMORY TOOL ==========

#[derive(Debug, Deserialize)]
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
    pub async fn new(config: VectorMemoryConfig) -> Result<Self> {
        Ok(Self { config })
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
        let vector_memory = VectorMemory::new(self.config.clone()).await.map_err(|e| {
            VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
        })?;

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
}

// ========== REMOVE MEMORY TOOL ==========

#[derive(Debug, Deserialize)]
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
    pub async fn new(config: VectorMemoryConfig) -> Result<Self> {
        Ok(Self { config })
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
        let vector_memory = VectorMemory::new(self.config.clone()).await.map_err(|e| {
            VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
        })?;

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
}

// ========== SMART MEMORY UPDATE TOOL ==========

#[derive(Debug, Deserialize)]
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
    pub async fn new(config: VectorMemoryConfig) -> Result<Self> {
        Ok(Self { config })
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
        let vector_memory = VectorMemory::new(self.config.clone()).await.map_err(|e| {
            VectorMemoryError::new(format!("Failed to connect to vector memory: {e}"))
        })?;

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
}

// ========== CONVENIENCE FUNCTIONS ==========

/// Create vector memory search tool with config
pub async fn vector_memory_search_tool(
    config: VectorMemoryConfig,
) -> Result<VectorMemorySearchTool> {
    VectorMemorySearchTool::new(config).await
}

/// Create vector memory store tool with config
pub async fn vector_memory_store_tool(config: VectorMemoryConfig) -> Result<VectorMemoryStoreTool> {
    VectorMemoryStoreTool::new(config).await
}

/// Create vector memory remove tool with config
pub async fn vector_memory_remove_tool(
    config: VectorMemoryConfig,
) -> Result<VectorMemoryRemoveTool> {
    VectorMemoryRemoveTool::new(config).await
}

/// Create vector memory update tool with config
pub async fn vector_memory_update_tool(
    config: VectorMemoryConfig,
) -> Result<VectorMemoryUpdateTool> {
    VectorMemoryUpdateTool::new(config).await
}
