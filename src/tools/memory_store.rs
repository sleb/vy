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

        let learned_facts = memory
            .learn_from_input(&args.fact, "tool".to_string())
            .await
            .map_err(|e| MemoryStoreError::new(format!("Failed to store memory: {e}")))?;

        if learned_facts.is_empty() {
            Ok(MemoryStoreResponse {
                success: false,
                message: "No extractable facts found in the provided text".to_string(),
                stored_facts: vec![],
                count: 0,
            })
        } else {
            let count = learned_facts.len();
            Ok(MemoryStoreResponse {
                success: true,
                message: format!("Successfully stored {count} new facts"),
                stored_facts: learned_facts,
                count,
            })
        }
    }
}
