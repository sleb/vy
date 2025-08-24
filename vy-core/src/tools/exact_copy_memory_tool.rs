//! Exact Copy Memory Tool - Mirrors nutrition analysis tool exactly
//!
//! This tool is an exact structural copy of the nutrition analysis tool
//! to test if there's something specific about how we implement memory tools.

use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ExactCopyMemoryError(String);

impl std::fmt::Display for ExactCopyMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ExactCopyMemoryError {}

impl ExactCopyMemoryError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct ExactCopyMemoryArgs {
    pub fact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExactCopyMemoryResponse {
    pub success: bool,
    pub message: String,
    pub stored_fact: String,
}

impl std::fmt::Display for ExactCopyMemoryResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

pub struct ExactCopyMemoryTool {
    api_key: String,
}

impl ExactCopyMemoryTool {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl Tool for ExactCopyMemoryTool {
    const NAME: &'static str = "store_memory_exact";

    type Error = ExactCopyMemoryError;
    type Args = ExactCopyMemoryArgs;
    type Output = ExactCopyMemoryResponse;

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
            return Err(ExactCopyMemoryError::new(
                "OpenAI API key not configured. Run: vy config set llm_api_key",
            ));
        }

        // Simple success response
        Ok(ExactCopyMemoryResponse {
            success: true,
            message: "Fact stored successfully".to_string(),
            stored_fact: args.fact,
        })
    }
}

/// Create exact copy memory tool instance
pub fn exact_copy_memory_tool(api_key: String) -> ExactCopyMemoryTool {
    ExactCopyMemoryTool::new(api_key)
}
