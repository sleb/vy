//! Memory management system for Vy
//!
//! This module provides long-term memory capabilities including:
//! - Facts and information storage
//! - Relationship mapping
//! - Opinion and preference tracking
//! - Semantic search over memories

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod embeddings;
pub mod search;
pub mod storage;

pub use embeddings::*;
pub use search::*;
pub use storage::*;

/// Represents different types of memories Vy can store
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Factual information about entities, events, etc.
    Fact,
    /// Relationships between entities
    Relationship,
    /// User preferences and opinions
    Opinion,
    /// Important conversation excerpts
    Conversation,
    /// Personal information about the user
    Personal,
    /// General knowledge or insights
    Knowledge,
}

/// Core memory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub entities: Vec<String>,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Memory {
    pub fn new(memory_type: MemoryType, content: String, entities: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type,
            content,
            entities,
            confidence: 1.0,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Query structure for memory retrieval
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub content: String,
    pub memory_types: Option<Vec<MemoryType>>,
    pub entities: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub limit: usize,
    pub min_confidence: f32,
    pub similarity_threshold: f32,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            content: String::new(),
            memory_types: None,
            entities: None,
            tags: None,
            limit: 10,
            min_confidence: 0.5,
            similarity_threshold: 0.7,
        }
    }
}

/// Trait for memory storage backends
#[async_trait::async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a new memory
    async fn store_memory(&mut self, memory: Memory) -> Result<()>;

    /// Retrieve memories by query
    async fn query_memories(&self, query: &MemoryQuery) -> Result<Vec<Memory>>;

    /// Update an existing memory
    async fn update_memory(&mut self, memory: Memory) -> Result<()>;

    /// Delete a memory by ID
    async fn delete_memory(&mut self, id: &str) -> Result<bool>;

    /// Get memory by ID
    async fn get_memory(&self, id: &str) -> Result<Option<Memory>>;

    /// Get all memories (for debugging/export)
    async fn get_all_memories(&self) -> Result<Vec<Memory>>;

    /// Clear all memories
    async fn clear_memories(&mut self) -> Result<()>;
}

/// Main memory manager that coordinates storage and retrieval
pub struct MemoryManager<S: MemoryStore, E: EmbeddingProvider> {
    store: S,
    embeddings: E,
}

impl<S: MemoryStore, E: EmbeddingProvider> MemoryManager<S, E> {
    pub fn new(store: S, embeddings: E) -> Self {
        Self { store, embeddings }
    }

    /// Add a new memory with automatic embedding generation
    pub async fn add_memory(&mut self, memory: Memory) -> Result<()> {
        // Generate embedding for the memory content
        let embedding = self.embeddings.generate_embedding(&memory.content).await?;

        // Store memory with embedding
        let mut memory_with_embedding = memory;
        memory_with_embedding
            .metadata
            .insert("embedding".to_string(), serde_json::to_string(&embedding)?);

        self.store.store_memory(memory_with_embedding).await
    }

    /// Search for similar memories
    pub async fn search_memories(&self, query: &MemoryQuery) -> Result<Vec<Memory>> {
        self.store.query_memories(query).await
    }

    /// Extract and store memories from conversation content
    pub async fn extract_memories_from_conversation(
        &mut self,
        user_input: &str,
        _assistant_response: &str,
    ) -> Result<Vec<Memory>> {
        // This would use an extraction model to identify memorable content
        // For now, we'll implement a simple heuristic-based approach
        let mut memories = Vec::new();

        // Look for factual statements, preferences, etc.
        if let Some(memory) = self.extract_user_preference(user_input).await? {
            memories.push(memory);
        }

        if let Some(memory) = self.extract_factual_information(user_input).await? {
            memories.push(memory);
        }

        // Store extracted memories
        for memory in &memories {
            self.add_memory(memory.clone()).await?;
        }

        Ok(memories)
    }

    async fn extract_user_preference(&self, input: &str) -> Result<Option<Memory>> {
        // Simple heuristic - look for preference indicators
        let preference_indicators = [
            "i like",
            "i love",
            "i hate",
            "i dislike",
            "i prefer",
            "my favorite",
            "i enjoy",
            "i can't stand",
        ];

        let input_lower = input.to_lowercase();
        for indicator in &preference_indicators {
            if input_lower.contains(indicator) {
                return Ok(Some(Memory::new(
                    MemoryType::Opinion,
                    input.to_string(),
                    vec!["user".to_string()],
                )));
            }
        }

        Ok(None)
    }

    async fn extract_factual_information(&self, input: &str) -> Result<Option<Memory>> {
        // Simple heuristic - look for factual statements
        let fact_indicators = [
            "my name is",
            "i am",
            "i work at",
            "i live in",
            "i was born",
            "i have",
            "my birthday",
        ];

        let input_lower = input.to_lowercase();
        for indicator in &fact_indicators {
            if input_lower.contains(indicator) {
                return Ok(Some(Memory::new(
                    MemoryType::Personal,
                    input.to_string(),
                    vec!["user".to_string()],
                )));
            }
        }

        Ok(None)
    }

    /// Get contextually relevant memories for a conversation
    pub async fn get_relevant_memories(
        &self,
        current_input: &str,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        let query = MemoryQuery {
            content: current_input.to_string(),
            limit,
            similarity_threshold: 0.6,
            ..Default::default()
        };

        self.search_memories(&query).await
    }

    /// Update memory confidence based on usage patterns
    pub async fn update_memory_confidence(
        &mut self,
        memory_id: &str,
        adjustment: f32,
    ) -> Result<()> {
        if let Some(mut memory) = self.store.get_memory(memory_id).await? {
            memory.confidence = (memory.confidence + adjustment).clamp(0.0, 1.0);
            memory.updated_at = Utc::now();
            self.store.update_memory(memory).await?;
        }
        Ok(())
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self) -> Result<MemoryStats> {
        let all_memories = self.store.get_all_memories().await?;

        let mut stats = MemoryStats::default();
        stats.total_memories = all_memories.len();

        for memory in &all_memories {
            match memory.memory_type {
                MemoryType::Fact => stats.fact_count += 1,
                MemoryType::Relationship => stats.relationship_count += 1,
                MemoryType::Opinion => stats.opinion_count += 1,
                MemoryType::Conversation => stats.conversation_count += 1,
                MemoryType::Personal => stats.personal_count += 1,
                MemoryType::Knowledge => stats.knowledge_count += 1,
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Default)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub fact_count: usize,
    pub relationship_count: usize,
    pub opinion_count: usize,
    pub conversation_count: usize,
    pub personal_count: usize,
    pub knowledge_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(
            MemoryType::Fact,
            "The user likes coffee".to_string(),
            vec!["user".to_string(), "coffee".to_string()],
        );

        assert_eq!(memory.memory_type, MemoryType::Fact);
        assert_eq!(memory.content, "The user likes coffee");
        assert_eq!(memory.entities, vec!["user", "coffee"]);
        assert_eq!(memory.confidence, 1.0);
    }

    #[test]
    fn test_memory_query_default() {
        let query = MemoryQuery::default();
        assert_eq!(query.limit, 10);
        assert_eq!(query.min_confidence, 0.5);
        assert_eq!(query.similarity_threshold, 0.7);
    }
}
