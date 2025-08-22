use crate::memory::{Memory, default_memory_file};
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MemoryUpdateError(String);

impl std::fmt::Display for MemoryUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MemoryUpdateError {}

impl MemoryUpdateError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct MemoryUpdateArgs {
    pub search_query: String,
    pub new_fact: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryUpdateResponse {
    pub success: bool,
    pub message: String,
    pub updated_facts: Vec<String>,
    pub removed_facts: Vec<String>,
    pub added_facts: Vec<String>,
}

pub struct MemoryUpdateTool;

impl Default for MemoryUpdateTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryUpdateTool {
    pub fn new() -> Self {
        Self
    }

    /// Find and update related memories using simple search-based approach
    /// All keyword-based categorization has been removed to avoid brittle logic
    async fn update_related_memories(
        &self,
        memory: &mut Memory,
        search_query: &str,
        new_fact: &str,
    ) -> Result<MemoryUpdateResponse, MemoryUpdateError> {
        let mut removed_facts = Vec::new();
        let mut added_facts = Vec::new();

        // Load memory first to avoid borrowing issues
        memory
            .load()
            .await
            .map_err(|e| MemoryUpdateError::new(format!("Failed to load memory: {e}")))?;

        // Search for existing memories that might conflict using the search query
        let existing_matches = memory.search(search_query);

        if !existing_matches.is_empty() {
            let facts_to_remove: Vec<String> = existing_matches
                .iter()
                .map(|entry| entry.fact.clone())
                .collect();

            let removed_entries = memory
                .remove_entries(|entry| facts_to_remove.contains(&entry.fact))
                .await
                .map_err(|e| MemoryUpdateError::new(format!("Failed to remove entries: {e}")))?;

            for entry in removed_entries {
                removed_facts.push(entry.fact);
            }
        }

        // Add the new fact directly - LLM extraction happens at conversation level
        memory.add_entry_direct(new_fact.to_string(), "update".to_string());
        memory
            .save()
            .await
            .map_err(|e| MemoryUpdateError::new(format!("Failed to save memory: {e}")))?;

        added_facts.push(new_fact.to_string());

        let message = if !removed_facts.is_empty() {
            format!(
                "Successfully updated memory. Removed {} old facts and added {} new facts.",
                removed_facts.len(),
                added_facts.len()
            )
        } else {
            format!("Added {} new facts to memory.", added_facts.len())
        };

        Ok(MemoryUpdateResponse {
            success: true,
            message,
            updated_facts: added_facts.clone(),
            removed_facts,
            added_facts,
        })
    }

    // Removed all brittle keyword-based categorization methods
    // The update logic now relies on search-based matching instead of hardcoded patterns
}

impl Tool for MemoryUpdateTool {
    const NAME: &'static str = "update_memory";

    type Error = MemoryUpdateError;
    type Args = MemoryUpdateArgs;
    type Output = MemoryUpdateResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Update or replace existing memories with new information. This tool can find related memories and update them with new facts, handling conflicts intelligently.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "search_query": {
                        "type": "string",
                        "description": "Keywords to find related memories that should be updated (e.g., 'job', 'employment', 'work', 'name')"
                    },
                    "new_fact": {
                        "type": "string",
                        "description": "The new fact to replace the old information with"
                    }
                },
                "required": ["search_query", "new_fact"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let memory_file = default_memory_file()
            .map_err(|e| MemoryUpdateError::new(format!("Failed to get memory file path: {e}")))?;
        let mut memory = Memory::new(memory_file);

        self.update_related_memories(&mut memory, &args.search_query, &args.new_fact)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_employment_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = Memory::new(file_path);

        // Add some initial employment info
        memory.add_entry_direct("User works at Microsoft".to_string(), "test".to_string());
        memory.add_entry_direct(
            "User is a software engineer".to_string(),
            "test".to_string(),
        );

        let tool = MemoryUpdateTool::new();

        // Update employment info
        let args = MemoryUpdateArgs {
            search_query: "employment".to_string(),
            new_fact: "I am a Sr Software Development Manager at Amazon".to_string(),
        };

        let result = tool
            .update_related_memories(&mut memory, &args.search_query, &args.new_fact)
            .await
            .unwrap();

        assert!(result.success);
    }

    #[tokio::test]
    async fn test_scott_scenario() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = Memory::new(file_path);

        // Recreate Scott's exact memory scenario
        memory.add_entry_direct("User's name is Scott".to_string(), "manual".to_string());
        memory.add_entry_direct("User loves coffee".to_string(), "manual".to_string());
        memory.add_entry_direct(
            "User is a software engineer".to_string(),
            "manual".to_string(),
        );
        memory.add_entry_direct("User works at Microsoft".to_string(), "test".to_string());
        memory.add_entry_direct("User loves pizza".to_string(), "test".to_string());
        memory.save().await.unwrap();

        let tool = MemoryUpdateTool::new();

        // Test the exact update Scott tried
        let args = MemoryUpdateArgs {
            search_query: "work".to_string(),
            new_fact: "I'm a Sr Software Development Manager at Amazon".to_string(),
        };

        let result = tool
            .update_related_memories(&mut memory, &args.search_query, &args.new_fact)
            .await
            .unwrap();

        assert!(result.success);
        println!("Removed facts: {:?}", result.removed_facts);
        println!("Added facts: {:?}", result.added_facts);

        // Verify non-employment memories remain
        let _coffee_results = memory.search("coffee");
        assert!(
            !_coffee_results.is_empty(),
            "Coffee preference should remain"
        );

        let pizza_results = memory.search("pizza");
        assert!(!pizza_results.is_empty(), "Pizza preference should remain");
    }

    #[tokio::test]
    async fn test_name_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = Memory::new(file_path);

        // Add initial name
        memory.add_entry_direct("User's name is John".to_string(), "test".to_string());

        let tool = MemoryUpdateTool::new();

        // Update name
        let args = MemoryUpdateArgs {
            search_query: "name".to_string(),
            new_fact: "My name is Scott".to_string(),
        };

        let result = tool
            .update_related_memories(&mut memory, &args.search_query, &args.new_fact)
            .await
            .unwrap();

        assert!(result.success);
        assert!(!result.removed_facts.is_empty());

        let john_results = memory.search("John");
        assert!(john_results.is_empty());
    }

    #[tokio::test]
    async fn test_general_search_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = Memory::new(file_path);

        // Add initial preference using add_entry_direct since learn_from_input now returns empty
        memory.add_entry_direct("User likes tea".to_string(), "test".to_string());
        memory.add_entry_direct("User enjoys coffee".to_string(), "test".to_string());

        let tool = MemoryUpdateTool::new();

        // Update preference using search
        let args = MemoryUpdateArgs {
            search_query: "coffee".to_string(),
            new_fact: "I prefer espresso now".to_string(),
        };

        let result = tool
            .update_related_memories(&mut memory, &args.search_query, &args.new_fact)
            .await
            .unwrap();

        assert!(result.success);

        let tea_results = memory.search("tea");
        assert!(!tea_results.is_empty());
    }

    // Removed tests for keyword-based detection methods that were deleted
    // These brittle pattern matching methods have been replaced with LLM-based analysis
}
