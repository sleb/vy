use crate::simple_memory::{SimpleMemory, default_memory_file};
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MemoryError(String);

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MemoryError {}

impl MemoryError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct MemoryArgs {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryEntry {
    pub fact: String,
    pub source: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryResponse {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub entries: Vec<MemoryEntry>,
    pub total_count: usize,
}

pub struct MemoryTool;

impl Default for MemoryTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for MemoryTool {
    const NAME: &'static str = "search_memory";

    type Error = MemoryError;
    type Args = MemoryArgs;
    type Output = MemoryResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search personal memories and facts about the user. Find relevant information from past conversations.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let memory_file = default_memory_file()
            .map_err(|e| MemoryError::new(format!("Failed to get memory file path: {e}")))?;
        let mut memory = SimpleMemory::new(memory_file);
        memory
            .load()
            .await
            .map_err(|e| MemoryError::new(format!("Failed to load memory: {e}")))?;

        let results = memory.search(&args.query);
        let entries = results
            .iter()
            .map(|entry| MemoryEntry {
                fact: entry.fact.clone(),
                source: entry.source.clone(),
                timestamp: entry.timestamp.format("%Y-%m-%d %H:%M").to_string(),
            })
            .collect();

        Ok(MemoryResponse {
            action: "search".to_string(),
            success: true,
            message: format!("Found {} memories matching '{}'", results.len(), args.query),
            entries,
            total_count: results.len(),
        })
    }
}
