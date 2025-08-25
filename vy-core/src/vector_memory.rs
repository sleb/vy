//! Vector-based Memory System for Vy
//!
//! This module provides a cloud-based memory system using Qdrant vector database
//! for semantic search and storage of conversation memories.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use qdrant_client::Payload;
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, DeletePointsBuilder, Distance, PointStruct, ScrollPointsBuilder,
        SearchPointsBuilder, UpsertPointsBuilder, Value, VectorParamsBuilder, value::Kind,
        vectors_config::Config as VectorsConfig,
    },
};
use rig::client::completion::CompletionClientDyn;
use rig::completion::Prompt;
use rig::providers::openai;
use serde_json;
use std::collections::HashMap;

use crate::memory::MemoryEntry;

/// Vector memory configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorMemoryConfig {
    #[serde(default = "default_qdrant_url")]
    pub qdrant_url: String,
    pub qdrant_api_key: Option<String>,
    #[serde(default = "default_collection_name")]
    pub collection_name: String,
    // Mandatory - no default, must be provided by user
    pub openai_api_key: String,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
}

// Hard-coded defaults for vector memory configuration
fn default_qdrant_url() -> String {
    "http://localhost:6334".to_string()
}

fn default_collection_name() -> String {
    "vy_memories".to_string()
}

fn default_embedding_model() -> String {
    "text-embedding-3-small".to_string()
}

impl Default for VectorMemoryConfig {
    fn default() -> Self {
        Self {
            qdrant_url: default_qdrant_url(),
            qdrant_api_key: None,
            collection_name: default_collection_name(),
            // No default for API key - must be provided by user
            openai_api_key: String::new(),
            embedding_model: default_embedding_model(),
        }
    }
}

/// Vector-based memory system using Qdrant
pub struct VectorMemory {
    client: Qdrant,
    config: VectorMemoryConfig,
    openai_client: openai::Client,
}

impl VectorMemory {
    /// Create a new vector memory instance
    pub async fn new(config: VectorMemoryConfig) -> Result<Self> {
        // Create Qdrant client - try different URL formats for cloud compatibility
        let client_url = if config.qdrant_url.contains(".cloud.qdrant.io") {
            // For Qdrant Cloud, try with explicit port 6334 for gRPC
            if config.qdrant_url.contains(":6334") {
                config.qdrant_url.clone()
            } else {
                format!("{}:6334", config.qdrant_url)
            }
        } else {
            config.qdrant_url.clone()
        };

        log::debug!("Attempting to connect to Qdrant at: {client_url}");

        let client = if let Some(api_key) = &config.qdrant_api_key {
            Qdrant::from_url(&client_url)
                .api_key(api_key.clone())
                .build()
                .context("Failed to create Qdrant client with API key")?
        } else {
            Qdrant::from_url(&client_url)
                .build()
                .context("Failed to create Qdrant client")?
        };

        // Create OpenAI client for embeddings
        let openai_client = openai::Client::builder(&config.openai_api_key)
            .build()
            .context("Failed to create OpenAI client for embeddings")?;

        let memory = Self {
            client,
            config,
            openai_client,
        };

        // Ensure collection exists
        memory.ensure_collection().await?;

        Ok(memory)
    }

    /// Ensure the Qdrant collection exists with proper configuration
    async fn ensure_collection(&self) -> Result<()> {
        // Check if collection exists
        match self
            .client
            .collection_info(&self.config.collection_name)
            .await
        {
            Ok(_) => {
                log::debug!(
                    "Collection '{}' already exists",
                    self.config.collection_name
                );
                return Ok(());
            }
            Err(_) => {
                log::info!("Creating collection '{}'", self.config.collection_name);
            }
        }

        // Create collection with vector configuration
        let create_collection = CreateCollectionBuilder::new(&self.config.collection_name)
            .vectors_config(VectorsConfig::Params(
                VectorParamsBuilder::new(1536, Distance::Cosine).build(),
            ))
            .build();

        self.client
            .create_collection(create_collection)
            .await
            .context("Failed to create Qdrant collection")?;

        log::info!(
            "Successfully created collection '{}'",
            self.config.collection_name
        );
        Ok(())
    }

    /// Generate embedding for text using OpenAI
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Use reqwest client directly for embeddings since rig doesn't have embedding support
        let client = reqwest::Client::new();

        let request = serde_json::json!({
            "input": text,
            "model": self.config.embedding_model,
            "encoding_format": "float"
        });

        let response = client
            .post("https://api.openai.com/v1/embeddings")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.openai_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to request embedding from OpenAI")?;

        let response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse embedding response")?;

        let embedding = response["data"][0]["embedding"]
            .as_array()
            .context("Invalid embedding response format")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    /// Store a memory entry in the vector database
    pub async fn store_memory(&self, entry: &MemoryEntry) -> Result<u64> {
        let embedding = self.generate_embedding(&entry.fact).await?;

        // Create point ID from timestamp to ensure uniqueness
        let point_id = entry.timestamp.timestamp_millis() as u64;

        // Prepare payload with metadata using Qdrant Value types
        let mut payload = HashMap::new();
        payload.insert(
            "fact".to_string(),
            Value {
                kind: Some(Kind::StringValue(entry.fact.clone())),
            },
        );
        payload.insert(
            "source".to_string(),
            Value {
                kind: Some(Kind::StringValue(entry.source.clone())),
            },
        );
        payload.insert(
            "timestamp".to_string(),
            Value {
                kind: Some(Kind::StringValue(entry.timestamp.to_rfc3339())),
            },
        );

        let point = PointStruct::new(point_id, embedding, Payload::from(payload));

        let upsert_points =
            UpsertPointsBuilder::new(&self.config.collection_name, vec![point]).build();

        self.client
            .upsert_points(upsert_points)
            .await
            .context("Failed to store memory in vector database")?;

        log::debug!("Stored memory with ID: {point_id}");
        Ok(point_id)
    }

    /// Search for memories using semantic similarity
    pub async fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let query_embedding = self.generate_embedding(query).await?;

        let search_points =
            SearchPointsBuilder::new(&self.config.collection_name, query_embedding, limit as u64)
                .with_payload(true)
                .build();

        let search_result = self
            .client
            .search_points(search_points)
            .await
            .context("Failed to search memories")?;

        let mut memories = Vec::new();
        for scored_point in search_result.result {
            let memory_entry = self.payload_to_memory_entry(scored_point.payload)?;
            memories.push(memory_entry);
        }

        Ok(memories)
    }

    /// Convert Qdrant payload back to MemoryEntry
    fn payload_to_memory_entry(&self, payload: HashMap<String, Value>) -> Result<MemoryEntry> {
        let fact = payload
            .get("fact")
            .and_then(|v| match &v.kind {
                Some(Kind::StringValue(s)) => Some(s.clone()),
                _ => None,
            })
            .context("Missing or invalid 'fact' field")?;

        let source = payload
            .get("source")
            .and_then(|v| match &v.kind {
                Some(Kind::StringValue(s)) => Some(s.clone()),
                _ => None,
            })
            .context("Missing or invalid 'source' field")?;

        let timestamp_str = payload
            .get("timestamp")
            .and_then(|v| match &v.kind {
                Some(Kind::StringValue(s)) => Some(s.clone()),
                _ => None,
            })
            .context("Missing or invalid 'timestamp' field")?;

        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .context("Failed to parse timestamp")?
            .with_timezone(&Utc);

        Ok(MemoryEntry {
            timestamp,
            fact,
            source,
        })
    }

    /// Get all memories (for migration/backup purposes)
    pub async fn get_all_memories(&self) -> Result<Vec<MemoryEntry>> {
        let scroll_points = ScrollPointsBuilder::new(&self.config.collection_name)
            .limit(1000) // Adjust based on expected memory count
            .with_payload(true)
            .build();

        let scroll_result = self
            .client
            .scroll(scroll_points)
            .await
            .context("Failed to retrieve all memories")?;

        let mut memories = Vec::new();
        for point in scroll_result.result {
            let memory_entry = self.payload_to_memory_entry(point.payload)?;
            memories.push(memory_entry);
        }

        // Sort by timestamp (newest first)
        memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(memories)
    }

    /// Delete a memory by timestamp (used as unique identifier)
    pub async fn delete_memory(&self, timestamp: DateTime<Utc>) -> Result<bool> {
        let point_id = timestamp.timestamp_millis() as u64;

        let delete_points = DeletePointsBuilder::new(&self.config.collection_name)
            .points(vec![point_id])
            .build();

        let result = self
            .client
            .delete_points(delete_points)
            .await
            .context("Failed to delete memory from vector database")?;

        Ok(result
            .result
            .as_ref()
            .map(|r| r.status() == qdrant_client::qdrant::UpdateStatus::Completed)
            .unwrap_or(false))
    }

    /// Clear all memories
    pub async fn clear_all(&self) -> Result<()> {
        // Delete the entire collection and recreate it
        self.client
            .delete_collection(&self.config.collection_name)
            .await
            .context("Failed to delete collection")?;

        // Recreate the collection
        self.ensure_collection().await?;

        log::info!("Cleared all memories from vector database");
        Ok(())
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> Result<VectorMemoryStats> {
        let info = self
            .client
            .collection_info(&self.config.collection_name)
            .await
            .context("Failed to get collection info")?;

        let points_count = info
            .result
            .as_ref()
            .and_then(|r| r.points_count)
            .unwrap_or(0);

        // Get oldest and newest memories by scrolling
        let all_memories = self.get_all_memories().await?;
        let oldest_entry = all_memories.last().map(|e| e.timestamp);
        let newest_entry = all_memories.first().map(|e| e.timestamp);

        Ok(VectorMemoryStats {
            total_entries: points_count as usize,
            oldest_entry,
            newest_entry,
        })
    }

    /// Extract and store facts from conversation using LLM analysis
    pub async fn learn_from_conversation(
        &self,
        conversation: &str,
        source: &str,
        memory_model_id: &str,
    ) -> Result<Vec<String>> {
        // Get existing memories to provide context and avoid duplicates
        let existing_memories = self.search_memories(conversation, 10).await?;
        let existing_facts: Vec<String> = existing_memories
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

{existing_context}

Conversation: "{conversation}"

Only extract NEW facts that are NOT already in the existing memories. Format each fact as a clear, specific statement. Include names, dates, times, and places when mentioned.

Return ONLY a JSON array of NEW facts, like:
["Has a son named Henry who is turning 18", "Has appointment with nutritionist Michael at 4pm", "Needs to update food logging in Cronometer"]"#
        );

        let agent = self
            .openai_client
            .agent(memory_model_id)
            .preamble("You are an expert at extracting and formatting important personal information from conversations.")
            .build();

        let response = agent
            .prompt(&prompt)
            .await
            .context("Failed to get LLM response for fact extraction")?;

        // Parse the JSON response
        let response_cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        let facts: Vec<String> = serde_json::from_str(response_cleaned)
            .context("Failed to parse LLM response as JSON array")?;

        // Filter out very short, vague, or generic facts
        let filtered_facts: Vec<String> = facts
            .into_iter()
            .filter(|fact| {
                let fact_lower = fact.to_lowercase();
                fact.len() > 8
                    && !fact_lower.starts_with("user is")
                    && !fact_lower.contains("user said")
                    && !fact_lower.contains("user mentioned")
                    && !fact_lower.contains("user talked about")
                    && !fact.trim().is_empty()
            })
            .collect();

        // Store the facts in the vector database
        for fact in &filtered_facts {
            let entry = MemoryEntry::new(fact.clone(), source.to_string());
            self.store_memory(&entry).await?;
        }

        if !filtered_facts.is_empty() {
            log::debug!(
                "Stored {} new facts from conversation",
                filtered_facts.len()
            );
        }

        Ok(filtered_facts)
    }
}

/// Memory statistics for vector database
#[derive(Debug)]
pub struct VectorMemoryStats {
    pub total_entries: usize,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

impl VectorMemoryStats {
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
            "Vector Memory Statistics:\n\
             Total entries: {}\n\
             Oldest entry: {}\n\
             Newest entry: {}",
            self.total_entries, oldest, newest
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Qdrant instance and OpenAI API key
    async fn test_vector_memory_basic() {
        let config = VectorMemoryConfig {
            qdrant_url: "http://localhost:6334".to_string(),
            qdrant_api_key: None,
            collection_name: "test_memories".to_string(),
            openai_api_key: std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
            embedding_model: "text-embedding-3-small".to_string(),
        };

        let memory = VectorMemory::new(config).await.unwrap();

        // Clear any existing test data
        memory.clear_all().await.unwrap();

        // Store a test memory
        let entry = MemoryEntry::new(
            "User works as a software engineer at Google".to_string(),
            "test".to_string(),
        );
        memory.store_memory(&entry).await.unwrap();

        // Search for it
        let results = memory
            .search_memories("software engineer", 5)
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert!(results[0].fact.contains("software engineer"));

        // Clean up
        memory.clear_all().await.unwrap();
    }
}
