use crate::simple_memory::{SimpleMemory, default_memory_file};
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MemoryStoreError(String);

impl std::fmt::Display for MemoryStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MemoryStoreError {}

impl MemoryStoreError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct MemoryStoreArgs {
    pub fact: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryStoreResponse {
    pub success: bool,
    pub message: String,
    pub stored_facts: Vec<String>,
    pub count: usize,
}

pub struct MemoryStoreTool;

impl Default for MemoryStoreTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStoreTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for MemoryStoreTool {
    const NAME: &'static str = "store_memory";

    type Error = MemoryStoreError;
    type Args = MemoryStoreArgs;
    type Output = MemoryStoreResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Store new facts and information about the user in memory for future reference."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "The fact or information about the user to remember"
                    }
                },
                "required": ["fact"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let memory_file = default_memory_file()
            .map_err(|e| MemoryStoreError::new(format!("Failed to get memory file path: {e}")))?;
        let mut memory = SimpleMemory::new(memory_file);
        memory
            .load()
            .await
            .map_err(|e| MemoryStoreError::new(format!("Failed to load memory: {e}")))?;

        // Store the fact directly - LLM extraction happens at conversation level
        memory.add_entry_direct(args.fact.clone(), "tool".to_string());
        memory
            .save()
            .await
            .map_err(|e| MemoryStoreError::new(format!("Failed to save memory: {e}")))?;

        Ok(MemoryStoreResponse {
            success: true,
            message: "Successfully stored fact to memory".to_string(),
            stored_facts: vec![args.fact],
            count: 1,
        })
    }
}
