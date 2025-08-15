//! Simple Memory Journal for Vy
//!
//! A lightweight memory system that stores conversation facts in a simple text format.
//! This is much simpler than the full memory system and focuses on just capturing
//! and retrieving basic facts from conversations.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
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

    /// Extract simple facts from user input using basic patterns
    pub fn extract_facts(&self, user_input: &str) -> Vec<String> {
        let mut facts = Vec::new();
        let input_lower = user_input.to_lowercase();

        // Basic patterns for fact extraction
        let patterns = [
            ("my name is ", "User's name is "),
            ("i am ", "User is "),
            ("i'm ", "User is "),
            ("i work at ", "User works at "),
            ("i work for ", "User works for "),
            ("i live in ", "User lives in "),
            ("i'm from ", "User is from "),
            ("my birthday is ", "User's birthday is "),
            ("i was born ", "User was born "),
            ("i like ", "User likes "),
            ("i love ", "User loves "),
            ("i hate ", "User hates "),
            ("i dislike ", "User dislikes "),
            ("my favorite ", "User's favorite "),
            ("i prefer ", "User prefers "),
            ("i have ", "User has "),
            ("i own ", "User owns "),
            ("i drive ", "User drives "),
            ("i study ", "User studies "),
            ("i'm studying ", "User is studying "),
            ("i can't stand ", "User can't stand "),
            ("i don't like ", "User doesn't like "),
            ("i enjoy ", "User enjoys "),
            ("i play ", "User plays "),
            ("i went to ", "User went to "),
            ("i graduated from ", "User graduated from "),
            ("my job is ", "User's job is "),
            ("my profession is ", "User's profession is "),
        ];

        for (trigger, prefix) in &patterns {
            if let Some(pos) = input_lower.find(trigger) {
                let start = pos + trigger.len();
                if let Some(rest) = user_input.get(start..) {
                    // Extract until end of sentence, conjunction, or reasonable stopping point
                    let fact_part = rest
                        .split(&['.', '!', '?', '\n'])
                        .next()
                        .unwrap_or(rest)
                        .trim();

                    // Stop at conjunctions for better fact separation
                    let fact_part = fact_part
                        .split(" and ")
                        .next()
                        .unwrap_or(fact_part)
                        .split(" but ")
                        .next()
                        .unwrap_or(fact_part)
                        .split(" or ")
                        .next()
                        .unwrap_or(fact_part)
                        .split(" so ")
                        .next()
                        .unwrap_or(fact_part)
                        .trim();

                    if !fact_part.is_empty() && fact_part.len() < 200 {
                        facts.push(format!("{prefix}{fact_part}"));
                    }
                }
            }
        }

        facts
    }

    /// Add a memory from user input
    pub async fn learn_from_input(
        &mut self,
        user_input: &str,
        source: String,
    ) -> Result<Vec<String>> {
        let facts = self.extract_facts(user_input);

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
    fn test_fact_extraction() {
        let memory = SimpleMemory::new(PathBuf::from("test.json"));

        let facts = memory.extract_facts("Hi, my name is Alice and I work at Google.");

        assert_eq!(facts.len(), 2);
        assert!(facts.contains(&"User's name is Alice".to_string()));
        assert!(facts.contains(&"User works at Google".to_string()));
    }

    #[test]
    fn test_preferences_extraction() {
        let memory = SimpleMemory::new(PathBuf::from("test.json"));

        let facts = memory.extract_facts("I love pizza and I hate broccoli.");

        assert_eq!(facts.len(), 2);
        assert!(facts.contains(&"User loves pizza".to_string()));
        assert!(facts.contains(&"User hates broccoli".to_string()));
    }

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
            memory
                .learn_from_input("My name is Bob", "test".to_string())
                .await
                .unwrap();
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
