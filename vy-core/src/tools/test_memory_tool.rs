//! Test Memory Tool - Minimal implementation to debug schema issues
//!
//! This is a simplified memory tool modeled exactly after the working
//! nutrition analysis tool to isolate the cause of schema validation errors.

use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct TestMemoryError(String);

impl std::fmt::Display for TestMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestMemoryError {}

impl TestMemoryError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

// Args struct - EXACTLY like nutrition tool (only Deserialize, no Serialize)
#[derive(Debug, Deserialize)]
pub struct TestMemoryArgs {
    pub fact: String,
}

// Response struct - EXACTLY like nutrition tool
#[derive(Debug, Serialize)]
pub struct TestMemoryResponse {
    pub success: bool,
    pub message: String,
}

// Tool struct - EXACTLY like nutrition tool
pub struct TestMemoryTool {
    api_key: String,
}

impl TestMemoryTool {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl Tool for TestMemoryTool {
    const NAME: &'static str = "test_memory";

    type Error = TestMemoryError;
    type Args = TestMemoryArgs;
    type Output = TestMemoryResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Test memory tool to debug schema issues. Store a simple fact."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "The fact to store in memory"
                    }
                },
                "required": ["fact"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Simple implementation - no async operations or complex logic
        if self.api_key.is_empty() {
            return Err(TestMemoryError::new("API key not configured"));
        }

        Ok(TestMemoryResponse {
            success: true,
            message: format!("Successfully stored fact: {}", args.fact),
        })
    }
}

/// Create a test memory tool instance
pub fn test_memory_tool(api_key: String) -> TestMemoryTool {
    TestMemoryTool::new(api_key)
}
