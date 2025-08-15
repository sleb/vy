use crate::simple_memory::{SimpleMemory, default_memory_file};
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

    /// Find and update related memories based on entity matching
    async fn update_related_memories(
        &self,
        memory: &mut SimpleMemory,
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

        // Check for job/employment related updates
        if self.is_employment_update(new_fact) {
            // Remove existing employment info
            let employment_keywords = [
                "works at",
                "employed at",
                "job is",
                "profession is",
                "work at",
                "work for",
                "is a software engineer",
                "is a software developer",
                "is a developer",
                "is a manager",
                "is a senior",
                "is a sr",
                "is an engineer",
                "is an architect",
                "is a director",
                "is a lead",
                "is a principal",
                "i'm a senior",
                "i'm a sr",
                "i'm a software",
                "i'm a manager",
                "i'm a developer",
                "i'm an engineer",
                "i am a senior",
                "i am a sr",
                "i am a software",
                "i am a manager",
                "i am a developer",
                "i am an engineer",
                "software engineer",
                "software developer",
                "development manager",
                "engineering manager",
                "technical lead",
                "tech lead",
                "product manager",
                "program manager",
            ];

            // Remove employment-related entries
            let removed_entries = memory
                .remove_entries(|entry| {
                    let fact_lower = entry.fact.to_lowercase();
                    employment_keywords
                        .iter()
                        .any(|keyword| fact_lower.contains(keyword))
                })
                .await
                .map_err(|e| MemoryUpdateError::new(format!("Failed to remove entries: {e}")))?;

            for entry in removed_entries {
                removed_facts.push(entry.fact);
            }
        }
        // Check for name updates
        else if self.is_name_update(new_fact) {
            let name_keywords = ["name is", "called", "i'm", "i am"];

            let removed_entries = memory
                .remove_entries(|entry| {
                    let fact_lower = entry.fact.to_lowercase();
                    name_keywords
                        .iter()
                        .any(|keyword| fact_lower.contains(keyword))
                })
                .await
                .map_err(|e| MemoryUpdateError::new(format!("Failed to remove entries: {e}")))?;

            for entry in removed_entries {
                removed_facts.push(entry.fact);
            }
        }
        // Check for location updates
        else if self.is_location_update(new_fact) {
            let location_keywords = ["lives in", "live in", "from", "located in"];

            let removed_entries = memory
                .remove_entries(|entry| {
                    let fact_lower = entry.fact.to_lowercase();
                    location_keywords
                        .iter()
                        .any(|keyword| fact_lower.contains(keyword))
                })
                .await
                .map_err(|e| MemoryUpdateError::new(format!("Failed to remove entries: {e}")))?;

            for entry in removed_entries {
                removed_facts.push(entry.fact);
            }
        }
        // For general updates, remove similar facts based on search query
        else {
            // Search for existing memories that might conflict
            let existing_matches = memory.search(search_query);

            if !existing_matches.is_empty() {
                let facts_to_remove: Vec<String> = existing_matches
                    .iter()
                    .map(|entry| entry.fact.clone())
                    .collect();

                let removed_entries = memory
                    .remove_entries(|entry| facts_to_remove.contains(&entry.fact))
                    .await
                    .map_err(|e| {
                        MemoryUpdateError::new(format!("Failed to remove entries: {e}"))
                    })?;

                for entry in removed_entries {
                    removed_facts.push(entry.fact);
                }
            }
        }

        // Add the new fact
        let learned_facts = memory
            .learn_from_input(new_fact, "update".to_string())
            .await
            .map_err(|e| MemoryUpdateError::new(format!("Failed to store new memory: {e}")))?;

        added_facts.extend(learned_facts.clone());

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

    fn is_employment_update(&self, fact: &str) -> bool {
        let employment_indicators = [
            "work at",
            "works at",
            "employed at",
            "job is",
            "profession is",
            "sr software",
            "senior software",
            "manager at",
            "developer at",
            "engineer at",
            "work for",
            "works for",
            "is a software",
            "is a senior",
            "is a sr",
            "is an engineer",
            "is a manager",
            "is a developer",
            "is a director",
            "is a lead",
            "is a principal",
            "i'm a senior",
            "i'm a sr",
            "i'm a software",
            "i'm a manager",
            "i'm a developer",
            "i'm an engineer",
            "i am a senior",
            "i am a sr",
            "i am a software",
            "i am a manager",
            "i am a developer",
            "i am an engineer",
            "software engineer",
            "software developer",
            "development manager",
            "engineering manager",
            "technical lead",
            "product manager",
            "program manager",
        ];
        let fact_lower = fact.to_lowercase();
        employment_indicators
            .iter()
            .any(|indicator| fact_lower.contains(indicator))
    }

    fn is_name_update(&self, fact: &str) -> bool {
        let name_indicators = ["name is", "called", "i'm", "i am"];
        let fact_lower = fact.to_lowercase();
        name_indicators
            .iter()
            .any(|indicator| fact_lower.contains(indicator))
    }

    fn is_location_update(&self, fact: &str) -> bool {
        let location_indicators = ["live in", "lives in", "from", "located in", "based in"];
        let fact_lower = fact.to_lowercase();
        location_indicators
            .iter()
            .any(|indicator| fact_lower.contains(indicator))
    }
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
        let mut memory = SimpleMemory::new(memory_file);

        self.update_related_memories(&mut memory, &args.search_query, &args.new_fact)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_memory::SimpleMemory;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_employment_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = SimpleMemory::new(file_path);

        // Add some initial employment info
        memory
            .learn_from_input("I work at Microsoft", "test".to_string())
            .await
            .unwrap();
        memory
            .learn_from_input("I am a software engineer", "test".to_string())
            .await
            .unwrap();

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
        assert!(!result.removed_facts.is_empty());
        assert!(!result.added_facts.is_empty());

        // Verify old employment info is removed
        let microsoft_results = memory.search("Microsoft");
        assert!(microsoft_results.is_empty());

        // Verify new employment info is added
        let amazon_results = memory.search("Amazon");
        assert!(!amazon_results.is_empty());
    }

    #[tokio::test]
    async fn test_scott_scenario() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = SimpleMemory::new(file_path);

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

        // Verify both old job title AND company are removed
        let engineer_results = memory.search("software engineer");
        assert!(
            engineer_results.is_empty(),
            "Software engineer should be removed"
        );

        let microsoft_results = memory.search("Microsoft");
        assert!(microsoft_results.is_empty(), "Microsoft should be removed");

        // Verify new employment info is added
        let amazon_results = memory.search("Amazon");
        assert!(!amazon_results.is_empty(), "Amazon should be found");

        let manager_results = memory.search("Manager");
        assert!(!manager_results.is_empty(), "Manager should be found");

        // Verify non-employment memories remain
        let coffee_results = memory.search("coffee");
        assert!(
            !coffee_results.is_empty(),
            "Coffee preference should remain"
        );

        let pizza_results = memory.search("pizza");
        assert!(!pizza_results.is_empty(), "Pizza preference should remain");
    }

    #[tokio::test]
    async fn test_name_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = SimpleMemory::new(file_path);

        // Add initial name
        memory
            .learn_from_input("My name is John", "test".to_string())
            .await
            .unwrap();

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

        // Verify old name is removed
        let john_results = memory.search("John");
        assert!(john_results.is_empty());

        // Verify new name is added
        let scott_results = memory.search("Scott");
        assert!(!scott_results.is_empty());
    }

    #[tokio::test]
    async fn test_general_search_update() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let mut memory = SimpleMemory::new(file_path);

        // Add initial preference
        memory
            .learn_from_input("I like tea", "test".to_string())
            .await
            .unwrap();

        let tool = MemoryUpdateTool::new();

        // Update preference using search
        let args = MemoryUpdateArgs {
            search_query: "tea".to_string(),
            new_fact: "I like coffee".to_string(),
        };

        let result = tool
            .update_related_memories(&mut memory, &args.search_query, &args.new_fact)
            .await
            .unwrap();

        assert!(result.success);

        // Verify tea preference is removed
        let tea_results = memory.search("tea");
        assert!(tea_results.is_empty());

        // Verify coffee preference is added
        let coffee_results = memory.search("coffee");
        assert!(!coffee_results.is_empty());
    }

    #[test]
    fn test_employment_detection() {
        let tool = MemoryUpdateTool::new();

        assert!(tool.is_employment_update("I work at Google"));
        assert!(tool.is_employment_update("Sr Software Development Manager at Amazon"));
        assert!(tool.is_employment_update("My job is engineer"));
        assert!(tool.is_employment_update("I am a software engineer"));
        assert!(tool.is_employment_update("User is a software engineer"));
        assert!(tool.is_employment_update("I'm a senior developer"));
        assert!(!tool.is_employment_update("I like my job"));
    }

    #[test]
    fn test_name_detection() {
        let tool = MemoryUpdateTool::new();

        assert!(tool.is_name_update("My name is Alice"));
        assert!(tool.is_name_update("I'm Bob"));
        assert!(tool.is_name_update("I am called Charlie"));
        assert!(!tool.is_name_update("I have a name"));
    }

    #[test]
    fn test_location_detection() {
        let tool = MemoryUpdateTool::new();

        assert!(tool.is_location_update("I live in Seattle"));
        assert!(tool.is_location_update("I'm from New York"));
        assert!(tool.is_location_update("Based in California"));
        assert!(!tool.is_location_update("I want to live somewhere nice"));
    }
}
