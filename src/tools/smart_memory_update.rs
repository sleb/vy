use crate::simple_memory::{SimpleMemory, default_memory_file};
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct SmartMemoryUpdateError(String);

impl std::fmt::Display for SmartMemoryUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SmartMemoryUpdateError {}

impl SmartMemoryUpdateError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct SmartMemoryUpdateArgs {
    pub new_information: String,
}

#[derive(Debug, Serialize)]
pub struct SmartMemoryUpdateResponse {
    pub success: bool,
    pub message: String,
    pub current_memories: Vec<String>,
    pub suggestions: Vec<String>,
    pub memory_count: usize,
}

pub struct SmartMemoryUpdateTool;

impl Default for SmartMemoryUpdateTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartMemoryUpdateTool {
    pub fn new() -> Self {
        Self
    }

    async fn process_update(
        &self,
        memory: &mut SimpleMemory,
        new_information: &str,
    ) -> Result<SmartMemoryUpdateResponse, SmartMemoryUpdateError> {
        // Load current memory
        memory
            .load()
            .await
            .map_err(|e| SmartMemoryUpdateError::new(format!("Failed to load memory: {e}")))?;

        // Get all current memories for context
        let all_memories: Vec<String> = memory
            .get_all_entries()
            .iter()
            .map(|entry| entry.fact.clone())
            .collect();

        // Search for potentially related memories using broad terms
        let search_terms = Self::extract_search_terms(new_information);
        let mut related_memories = Vec::new();

        for term in &search_terms {
            let results = memory.search(term);
            for result in results {
                if !related_memories.contains(&result.fact) {
                    related_memories.push(result.fact.clone());
                }
            }
        }

        // Create suggestions for the LLM to consider
        let mut suggestions = Vec::new();
        if !related_memories.is_empty() {
            suggestions.push(format!(
                "Found {} potentially related memories that may need updating: {}",
                related_memories.len(),
                related_memories.join("; ")
            ));
        }

        suggestions.push(format!("New information to process: '{new_information}'"));

        suggestions.push(
            "Consider what type of information this is (employment, personal details, preferences, location, etc.) and whether it conflicts with existing memories.".to_string()
        );

        // Store the new information as a raw memory for now
        // The LLM can decide what to do with conflicting information
        let _learned_facts = memory
            .learn_from_input(new_information, "smart_update_input".to_string())
            .await
            .map_err(|e| SmartMemoryUpdateError::new(format!("Failed to store memory: {e}")))?;

        Ok(SmartMemoryUpdateResponse {
            success: true,
            message: format!(
                "Processed new information and found {} related memories. Please review and decide what updates are needed.",
                related_memories.len()
            ),
            current_memories: all_memories,
            suggestions,
            memory_count: memory.get_all_entries().len(),
        })
    }

    fn extract_search_terms(text: &str) -> Vec<String> {
        let mut terms = Vec::new();

        // Extract all meaningful words (longer than 2 characters)
        // Remove brittle keyword matching - just extract actual words from the text
        for word in text.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
            if cleaned.len() > 2 {
                terms.push(cleaned.to_lowercase());
            }
        }

        // Remove duplicates and return first 5 terms
        terms.sort();
        terms.dedup();
        terms.truncate(5);
        terms
    }
}

impl Tool for SmartMemoryUpdateTool {
    const NAME: &'static str = "smart_update_memory";

    type Error = SmartMemoryUpdateError;
    type Args = SmartMemoryUpdateArgs;
    type Output = SmartMemoryUpdateResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Intelligently process new personal information and provide context about existing memories that may need updating. This tool finds related memories and provides suggestions for updates, letting you make informed decisions about memory management.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "new_information": {
                        "type": "string",
                        "description": "The new information to process and analyze against existing memories"
                    }
                },
                "required": ["new_information"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let memory_file = default_memory_file().map_err(|e| {
            SmartMemoryUpdateError::new(format!("Failed to get memory file path: {e}"))
        })?;
        let mut memory = SimpleMemory::new(memory_file);

        self.process_update(&mut memory, &args.new_information)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_memory::SimpleMemory;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_search_term_extraction() {
        let terms = SmartMemoryUpdateTool::extract_search_terms(
            "I'm now a Sr Software Development Manager at Amazon",
        );

        // Based on actual output: ["age", "amazon", "development", "i'm", "manager"]
        assert!(terms.contains(&"manager".to_string()));
        assert!(terms.contains(&"amazon".to_string()));
        assert!(terms.contains(&"development".to_string()));
        assert!(terms.len() <= 5); // Should be limited to 5 terms
    }

    #[tokio::test]
    async fn test_smart_update_context() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = SimpleMemory::new(file_path);

        // Set up initial employment info
        memory.add_entry_direct(
            "User is a software engineer".to_string(),
            "manual".to_string(),
        );
        memory.add_entry_direct("User works at Microsoft".to_string(), "manual".to_string());
        memory.save().await.unwrap();

        let tool = SmartMemoryUpdateTool::new();

        let args = SmartMemoryUpdateArgs {
            new_information: "I'm now a Sr Software Development Manager at Amazon".to_string(),
        };

        let result = tool
            .process_update(&mut memory, &args.new_information)
            .await
            .unwrap();

        assert!(result.success);
        assert!(!result.suggestions.is_empty());
        assert!(result.memory_count >= 2); // Should include existing memories

        // Should have suggestions about processing the new information
        assert!(
            result
                .suggestions
                .iter()
                .any(|s| s.contains("New information to process") || s.contains("related memories"))
        );
    }
}
