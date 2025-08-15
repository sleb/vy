//! Simple Memory Journal for Vy
//!
//! A lightweight memory system that stores conversation facts in a simple text format.
//! This is much simpler than the full memory system and focuses on just capturing
//! and retrieving basic facts from conversations.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rig::client::completion::CompletionClientDyn;
use rig::completion::Prompt;
use rig::providers::openai;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use tokio::fs as async_fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub timestamp: DateTime<Utc>,
    pub fact: String,
    pub source: String, // What conversation this came from
}

impl MemoryEntry {
    pub fn new(fact: String, source: String) -> Self {
        Self {
            timestamp: Utc::now(),
            fact,
            source,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryJournal {
    pub entries: Vec<MemoryEntry>,
}

impl MemoryJournal {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a new memory entry
    pub fn add_entry(&mut self, fact: String, source: String) {
        let entry = MemoryEntry::new(fact, source);
        self.entries.push(entry);
    }

    /// Search for entries containing the given text (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| entry.fact.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get all entries as a formatted string for display
    pub fn to_display_string(&self) -> String {
        if self.entries.is_empty() {
            return "No memories stored yet.".to_string();
        }

        let mut result = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            result.push_str(&format!(
                "{}. [{}] {}\n   Source: {}\n\n",
                i + 1,
                entry.timestamp.format("%Y-%m-%d %H:%M"),
                entry.fact,
                entry.source
            ));
        }
        result
    }
}

pub struct SimpleMemory {
    journal: MemoryJournal,
    file_path: PathBuf,
}

impl SimpleMemory {
    /// Create a new simple memory system with the given storage file
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            journal: MemoryJournal::new(),
            file_path,
        }
    }

    /// Load existing memories from disk
    pub async fn load(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            // Create parent directories if they don't exist
            if let Some(parent) = self.file_path.parent() {
                async_fs::create_dir_all(parent).await?;
            }
            return Ok(()); // No file exists yet, that's fine
        }

        let content = async_fs::read_to_string(&self.file_path)
            .await
            .with_context(|| format!("Failed to read memory file: {:?}", self.file_path))?;

        if content.trim().is_empty() {
            return Ok(());
        }

        self.journal = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse memory file: {:?}", self.file_path))?;

        Ok(())
    }

    /// Save memories to disk
    pub async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.journal)?;

        // Create parent directories if they don't exist
        if let Some(parent) = self.file_path.parent() {
            async_fs::create_dir_all(parent).await?;
        }

        async_fs::write(&self.file_path, content)
            .await
            .with_context(|| format!("Failed to write memory file: {:?}", self.file_path))?;

        Ok(())
    }

    /// Extract facts using LLM analysis for better memory detection
    pub async fn extract_facts_llm(
        &self,
        user_input: &str,
        api_key: &str,
        memory_model_id: &str,
        memory_preamble: &str,
    ) -> Result<Vec<String>> {
        let client = openai::Client::builder(api_key)
            .build()
            .context("Failed to create LLM client for fact extraction")?;

        // Get existing memories to provide context and avoid duplicates
        let existing_facts: Vec<String> = self
            .journal
            .entries
            .iter()
            .map(|entry| entry.fact.clone())
            .collect();

        let existing_context = if existing_facts.is_empty() {
            "No existing memories.".to_string()
        } else {
            format!("Existing memories:\n{}", existing_facts.join("\n"))
        };

        let prompt = format!(
            r#"Analyze this conversation and extract NEW important facts about the user to remember for future conversations.

Extract facts about:
- Personal details: name, age, family members, relationships
- Professional: job, company, career, work projects
- Location: where they live, work, places they frequent
- Appointments & commitments: meetings, events, scheduled activities
- Health: doctors, medical appointments, treatments, conditions
- Preferences: likes, dislikes, hobbies, interests
- Action items: reminders, tasks they need to do
- Life events: birthdays, anniversaries, milestones
- Projects & goals: what they're working on, planning

{}

Conversation: "{}"

Only extract NEW facts that are NOT already in the existing memories. Format each fact as a clear, specific statement. Include names, dates, times, and places when mentioned.

Return ONLY a JSON array of NEW facts, like:
["Has a son named Henry who is turning 18", "Has appointment with nutritionist Michael at 4pm", "Needs to update food logging in Cronometer"]"#,
            existing_context, user_input
        );

        let agent = client
            .agent(memory_model_id)
            .preamble(memory_preamble)
            .build();

        let response = agent
            .prompt(&prompt)
            .await
            .context("Failed to get LLM response for fact extraction")?;

        // Try to parse as JSON array
        let response_cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
        let facts: Vec<String> = serde_json::from_str(response_cleaned)
            .context("Failed to parse LLM response as JSON array")?;

        Ok(facts)
    }

    /// Add a memory from user input using LLM analysis
    pub async fn learn_from_input(
        &mut self,
        user_input: &str,
        source: String,
        api_key: &str,
        memory_model_id: &str,
        memory_preamble: &str,
    ) -> Result<Vec<String>> {
        // Use LLM-based extraction
        let mut facts = self
            .extract_facts_llm(user_input, api_key, memory_model_id, memory_preamble)
            .await?;

        // Filter out very short, vague, or generic facts
        facts.retain(|fact| {
            let fact_lower = fact.to_lowercase();
            fact.len() > 8
                && !fact_lower.starts_with("user is")
                && !fact_lower.contains("user said")
                && !fact_lower.contains("user mentioned")
                && !fact_lower.contains("user talked about")
                && !fact.trim().is_empty()
        });

        for fact in &facts {
            self.journal.add_entry(fact.clone(), source.clone());
        }

        if !facts.is_empty() {
            self.save().await?;
        }

        Ok(facts)
    }

    /// Search for relevant memories
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.journal.search(query)
    }

    /// Get all memories as a display string
    pub fn list_all(&self) -> String {
        self.journal.to_display_string()
    }

    /// Get a memory entry by index (1-based, as shown in list output) for preview
    pub fn get_entry_by_index(&self, index: usize) -> Option<&MemoryEntry> {
        if index == 0 || index > self.journal.entries.len() {
            return None;
        }
        self.journal.entries.get(index - 1)
    }

    /// Delete a memory by index (1-based, as shown in list output)
    pub async fn delete_by_index(&mut self, index: usize) -> Result<Option<MemoryEntry>> {
        if index == 0 || index > self.journal.entries.len() {
            return Ok(None);
        }

        let deleted_entry = self.journal.entries.remove(index - 1);
        self.save().await?;
        Ok(Some(deleted_entry))
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_entries: self.journal.entries.len(),
            oldest_entry: self.journal.entries.first().map(|e| e.timestamp),
            newest_entry: self.journal.entries.last().map(|e| e.timestamp),
        }
    }

    /// Clear all memories (for testing or reset)
    pub async fn clear(&mut self) -> Result<()> {
        self.journal = MemoryJournal::new();
        self.save().await
    }

    /// Remove entries matching a predicate
    #[allow(dead_code)]
    pub async fn remove_entries<F>(&mut self, predicate: F) -> Result<Vec<MemoryEntry>>
    where
        F: Fn(&MemoryEntry) -> bool,
    {
        let mut removed_entries = Vec::new();
        let mut i = 0;
        while i < self.journal.entries.len() {
            if predicate(&self.journal.entries[i]) {
                removed_entries.push(self.journal.entries.remove(i));
            } else {
                i += 1;
            }
        }
        if !removed_entries.is_empty() {
            self.save().await?;
        }
        Ok(removed_entries)
    }

    /// Get all entries (for updating operations)
    #[allow(dead_code)]
    pub fn get_all_entries(&self) -> &Vec<MemoryEntry> {
        &self.journal.entries
    }

    /// Add a raw memory entry directly
    #[allow(dead_code)]
    pub async fn add_entry(&mut self, entry: MemoryEntry) -> Result<()> {
        self.journal.entries.push(entry);
        self.save().await
    }

    /// Add a raw memory entry with fact and source (for testing)
    #[allow(dead_code)]
    pub fn add_entry_direct(&mut self, fact: String, source: String) {
        self.journal.add_entry(fact, source);
    }

    /// Get the number of memory entries
    pub fn entry_count(&self) -> usize {
        self.journal.entries.len()
    }

    /// Consolidate memory entries using LLM-based similarity detection
    pub async fn vacuum(&mut self, api_key: &str, memory_similarity_model_id: &str) -> Result<()> {
        let original_count = self.journal.entries.len();

        if original_count <= 1 {
            return Ok(());
        }

        // Step 1: Remove exact duplicates
        let mut unique_facts = std::collections::HashSet::new();
        let mut deduplicated = Vec::new();

        for entry in &self.journal.entries {
            let fact_normalized = entry.fact.trim().to_lowercase();
            if unique_facts.insert(fact_normalized) {
                deduplicated.push(entry.clone());
            }
        }

        // Step 2: Use LLM for intelligent similarity detection
        let client = openai::Client::builder(api_key)
            .build()
            .context("Failed to create LLM client for vacuum operation")?;

        let mut consolidated: Vec<MemoryEntry> = Vec::new();

        for entry in deduplicated {
            let mut should_keep = true;

            // Check against existing consolidated entries
            for existing in &consolidated {
                if self
                    .are_facts_similar_llm(
                        &client,
                        &entry.fact,
                        &existing.fact,
                        memory_similarity_model_id,
                    )
                    .await?
                {
                    should_keep = false;
                    break;
                }
            }

            if should_keep {
                consolidated.push(entry);
            }
        }

        // Step 3: Sort by timestamp (newest first) to keep most recent information
        consolidated.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        self.journal.entries = consolidated;
        Ok(())
    }

    /// LLM-based similarity check for facts
    async fn are_facts_similar_llm(
        &self,
        client: &openai::Client,
        fact1: &str,
        fact2: &str,
        memory_similarity_model_id: &str,
    ) -> Result<bool> {
        let prompt = format!(
            r#"Compare these two memory facts and determine if they contain essentially the same information.
Consider them similar if they refer to the same person, place, or concept with equivalent meaning,
even if worded differently.

Fact 1: "{}"
Fact 2: "{}"

Are these facts essentially the same? Respond with only "YES" or "NO"."#,
            fact1, fact2
        );

        let agent = client
            .agent(memory_similarity_model_id)
            .preamble("You are a helpful assistant that compares information for similarity.")
            .build();

        let response = agent
            .prompt(&prompt)
            .await
            .context("Failed to get LLM response for similarity check")?;

        let answer = response.trim().to_uppercase();
        Ok(answer == "YES")
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

impl MemoryStats {
    pub fn to_display_string(&self) -> String {
        let oldest = self
            .oldest_entry
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "None".to_string());

        let newest = self
            .newest_entry
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "None".to_string());

        format!(
            "Memory Statistics:\n\
             Total entries: {}\n\
             Oldest entry: {}\n\
             Newest entry: {}",
            self.total_entries, oldest, newest
        )
    }
}

/// Get the default memory file path
pub fn default_memory_file() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "vy")
        .context("Could not determine user directories")?;

    let data_dir = project_dirs.data_dir();
    Ok(data_dir.join("simple_memory.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_search() {
        let mut journal = MemoryJournal::new();
        journal.add_entry("User likes coffee".to_string(), "chat".to_string());
        journal.add_entry("User works at Google".to_string(), "chat".to_string());

        let results = journal.search("coffee");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fact, "User likes coffee");
    }

    #[tokio::test]
    async fn test_save_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        // Create and save some memories
        {
            let mut memory = SimpleMemory::new(file_path.clone());
            memory.add_entry_direct("User's name is Bob".to_string(), "test".to_string());
            memory.save().await.unwrap();
        }

        // Load in a new instance
        {
            let mut memory = SimpleMemory::new(file_path);
            memory.load().await.unwrap();
            let results = memory.search("Bob");
            assert_eq!(results.len(), 1);
        }
    }
}
