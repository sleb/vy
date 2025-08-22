use crate::memory::{Memory, default_memory_file};
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MemoryRemoveError(String);

impl std::fmt::Display for MemoryRemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MemoryRemoveError {}

impl MemoryRemoveError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct MemoryRemoveArgs {
    pub facts_to_remove: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MemoryRemoveResponse {
    pub success: bool,
    pub message: String,
    pub removed_facts: Vec<String>,
    pub removed_count: usize,
    pub not_found: Vec<String>,
}

pub struct MemoryRemoveTool;

impl Default for MemoryRemoveTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRemoveTool {
    pub fn new() -> Self {
        Self
    }

    async fn remove_specific_memories(
        &self,
        memory: &mut Memory,
        facts_to_remove: &[String],
    ) -> Result<MemoryRemoveResponse, MemoryRemoveError> {
        // Load current memory
        memory
            .load()
            .await
            .map_err(|e| MemoryRemoveError::new(format!("Failed to load memory: {e}")))?;

        let mut removed_facts = Vec::new();
        let mut not_found = Vec::new();

        for fact_to_remove in facts_to_remove {
            let mut found = false;

            // Search for memories that match this fact exactly or contain it
            let matching_entries = memory
                .get_all_entries()
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry.fact == *fact_to_remove
                        || entry.fact.contains(fact_to_remove)
                        || fact_to_remove.contains(&entry.fact)
                })
                .map(|(index, entry)| (index, entry.fact.clone()))
                .collect::<Vec<_>>();

            if !matching_entries.is_empty() {
                // Remove the matching entries
                let facts_to_match: Vec<String> = matching_entries
                    .iter()
                    .map(|(_, fact)| fact.clone())
                    .collect();
                let removed_entries = memory
                    .remove_entries(|entry| facts_to_match.contains(&entry.fact))
                    .await
                    .map_err(|e| {
                        MemoryRemoveError::new(format!("Failed to remove entries: {e}"))
                    })?;

                for entry in removed_entries {
                    removed_facts.push(entry.fact);
                    found = true;
                }
            }

            if !found {
                not_found.push(fact_to_remove.clone());
            }
        }

        let removed_count = removed_facts.len();
        let message = if removed_count > 0 {
            if not_found.is_empty() {
                format!("Successfully removed {removed_count} facts from memory")
            } else {
                format!(
                    "Removed {} facts from memory. {} facts were not found.",
                    removed_count,
                    not_found.len()
                )
            }
        } else {
            "No matching facts were found to remove".to_string()
        };

        Ok(MemoryRemoveResponse {
            success: removed_count > 0,
            message,
            removed_facts,
            removed_count,
            not_found,
        })
    }
}

impl Tool for MemoryRemoveTool {
    const NAME: &'static str = "remove_memories";

    type Error = MemoryRemoveError;
    type Args = MemoryRemoveArgs;
    type Output = MemoryRemoveResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Remove specific facts from memory by providing the exact text or partial matches. This tool is useful for removing outdated or incorrect information before adding new facts.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "facts_to_remove": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Array of facts to remove from memory. Can be exact matches or partial text that will match existing memories."
                    }
                },
                "required": ["facts_to_remove"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let memory_file = default_memory_file()
            .map_err(|e| MemoryRemoveError::new(format!("Failed to get memory file path: {e}")))?;
        let mut memory = Memory::new(memory_file);

        self.remove_specific_memories(&mut memory, &args.facts_to_remove)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_remove_specific_memories() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = Memory::new(file_path);

        // Add some test memories
        memory.add_entry_direct("User's name is Scott".to_string(), "manual".to_string());
        memory.add_entry_direct("User loves coffee".to_string(), "manual".to_string());
        memory.add_entry_direct(
            "User is a software engineer".to_string(),
            "manual".to_string(),
        );
        memory.add_entry_direct("User works at Microsoft".to_string(), "test".to_string());
        memory.save().await.unwrap();

        let tool = MemoryRemoveTool::new();

        // Test removing specific facts
        let args = MemoryRemoveArgs {
            facts_to_remove: vec![
                "User is a software engineer".to_string(),
                "User works at Microsoft".to_string(),
            ],
        };

        let result = tool
            .remove_specific_memories(&mut memory, &args.facts_to_remove)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.removed_count, 2);
        assert!(result.not_found.is_empty());

        // Verify the facts were removed
        let engineer_results = memory.search("software engineer");
        assert!(engineer_results.is_empty());

        let microsoft_results = memory.search("Microsoft");
        assert!(microsoft_results.is_empty());

        // Verify other facts remain
        let coffee_results = memory.search("coffee");
        assert!(!coffee_results.is_empty());

        let name_results = memory.search("Scott");
        assert!(!name_results.is_empty());
    }

    #[tokio::test]
    async fn test_remove_partial_matches() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = Memory::new(file_path);

        memory.add_entry_direct(
            "User is a software engineer at Microsoft".to_string(),
            "manual".to_string(),
        );
        memory.add_entry_direct("User loves pizza".to_string(), "manual".to_string());
        memory.save().await.unwrap();

        let tool = MemoryRemoveTool::new();

        // Test removing with partial matches
        let args = MemoryRemoveArgs {
            facts_to_remove: vec!["software engineer".to_string()],
        };

        let result = tool
            .remove_specific_memories(&mut memory, &args.facts_to_remove)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.removed_count, 1);

        // Verify the matching fact was removed
        let engineer_results = memory.search("software engineer");
        assert!(engineer_results.is_empty());

        // Verify other facts remain
        let pizza_results = memory.search("pizza");
        assert!(!pizza_results.is_empty());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_facts() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = Memory::new(file_path);

        memory.add_entry_direct("User loves coffee".to_string(), "manual".to_string());
        memory.save().await.unwrap();

        let tool = MemoryRemoveTool::new();

        let args = MemoryRemoveArgs {
            facts_to_remove: vec!["User hates tea".to_string()],
        };

        let result = tool
            .remove_specific_memories(&mut memory, &args.facts_to_remove)
            .await
            .unwrap();

        assert!(!result.success);
        assert_eq!(result.removed_count, 0);
        assert_eq!(result.not_found.len(), 1);
        assert_eq!(result.not_found[0], "User hates tea");
    }
}
