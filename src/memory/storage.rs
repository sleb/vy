//! Storage implementations for the memory system
//!
//! This module provides persistent storage backends for memories,
//! starting with a SQLite implementation for local storage.

use crate::memory::{Memory, MemoryQuery, MemoryStore, MemoryType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
use tokio_rusqlite::Connection;

/// SQLite-based memory storage implementation
pub struct SqliteMemoryStore {
    connection: Connection,
}

impl SqliteMemoryStore {
    /// Create a new SQLite memory store
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let connection = Connection::open(db_path).await?;

        let store = Self { connection };
        store.initialize_schema().await?;

        Ok(store)
    }

    /// Create an in-memory SQLite store for testing
    pub async fn new_in_memory() -> Result<Self> {
        let connection = Connection::open(":memory:").await?;

        let store = Self { connection };
        store.initialize_schema().await?;

        Ok(store)
    }

    /// Initialize the database schema
    async fn initialize_schema(&self) -> Result<()> {
        let create_memories_table = r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                memory_type TEXT NOT NULL,
                content TEXT NOT NULL,
                entities TEXT NOT NULL,  -- JSON array
                confidence REAL NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                tags TEXT NOT NULL,      -- JSON array
                metadata TEXT NOT NULL   -- JSON object
            )
        "#;

        let create_indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_memories_type ON memories (memory_type)",
            "CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories (created_at)",
            "CREATE INDEX IF NOT EXISTS idx_memories_confidence ON memories (confidence)",
        ];

        self.connection
            .call(move |conn| {
                conn.execute(create_memories_table, [])?;

                for index_sql in create_indexes {
                    conn.execute(index_sql, [])?;
                }

                Ok(())
            })
            .await?;

        Ok(())
    }

    /// Convert MemoryType to string for database storage
    fn memory_type_to_string(memory_type: &MemoryType) -> &'static str {
        match memory_type {
            MemoryType::Fact => "Fact",
            MemoryType::Relationship => "Relationship",
            MemoryType::Opinion => "Opinion",
            MemoryType::Conversation => "Conversation",
            MemoryType::Personal => "Personal",
            MemoryType::Knowledge => "Knowledge",
        }
    }

    /// Convert string to MemoryType
    fn string_to_memory_type(s: &str) -> MemoryType {
        match s {
            "Fact" => MemoryType::Fact,
            "Relationship" => MemoryType::Relationship,
            "Opinion" => MemoryType::Opinion,
            "Conversation" => MemoryType::Conversation,
            "Personal" => MemoryType::Personal,
            "Knowledge" => MemoryType::Knowledge,
            _ => MemoryType::Fact, // Default fallback
        }
    }

    /// Search memories by content similarity (basic text search for now)
    async fn search_by_content(&self, query_content: &str, limit: usize) -> Result<Vec<Memory>> {
        let query_content = query_content.to_string();
        let limit = limit as i64;

        let memories = self
            .connection
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, memory_type, content, entities, confidence,
                           created_at, updated_at, tags, metadata
                    FROM memories
                    WHERE content LIKE '%' || ?1 || '%'
                    ORDER BY confidence DESC, updated_at DESC
                    LIMIT ?2
                    "#,
                )?;

                let memory_iter =
                    stmt.query_map([query_content.as_str(), &limit.to_string()], |row| {
                        let entities_json: String = row.get("entities")?;
                        let tags_json: String = row.get("tags")?;
                        let metadata_json: String = row.get("metadata")?;
                        let created_at_str: String = row.get("created_at")?;
                        let updated_at_str: String = row.get("updated_at")?;
                        let memory_type_str: String = row.get("memory_type")?;

                        let entities: Vec<String> = serde_json::from_str(&entities_json)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                        let tags: Vec<String> = serde_json::from_str(&tags_json)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                        let metadata: HashMap<String, String> =
                            serde_json::from_str(&metadata_json).map_err(|e| {
                                rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                            })?;

                        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                            .with_timezone(&Utc);
                        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                            .with_timezone(&Utc);

                        Ok(Memory {
                            id: row.get("id")?,
                            memory_type: Self::string_to_memory_type(&memory_type_str),
                            content: row.get("content")?,
                            entities,
                            confidence: row.get("confidence")?,
                            created_at,
                            updated_at,
                            tags,
                            metadata,
                        })
                    })?;

                let mut memories = Vec::new();
                for memory_result in memory_iter {
                    memories.push(memory_result?);
                }

                Ok(memories)
            })
            .await?;

        Ok(memories)
    }
}

#[async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn store_memory(&mut self, memory: Memory) -> Result<()> {
        let entities_json = serde_json::to_string(&memory.entities)?;
        let tags_json = serde_json::to_string(&memory.tags)?;
        let metadata_json = serde_json::to_string(&memory.metadata)?;
        let memory_type_str = Self::memory_type_to_string(&memory.memory_type);
        let created_at_str = memory.created_at.to_rfc3339();
        let updated_at_str = memory.updated_at.to_rfc3339();

        let id = memory.id.clone();
        let content = memory.content.clone();
        let confidence = memory.confidence;

        self.connection
            .call(move |conn| {
                conn.execute(
                    r#"
                    INSERT OR REPLACE INTO memories
                    (id, memory_type, content, entities, confidence, created_at, updated_at, tags, metadata)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    "#,
                    (
                        &id,
                        memory_type_str,
                        &content,
                        &entities_json,
                        confidence,
                        &created_at_str,
                        &updated_at_str,
                        &tags_json,
                        &metadata_json,
                    ),
                )?;
                Ok(())
            })
            .await?;

        Ok(())
    }

    async fn query_memories(&self, query: &MemoryQuery) -> Result<Vec<Memory>> {
        // For now, implement basic content search
        let mut memories = if !query.content.is_empty() {
            self.search_by_content(&query.content, query.limit).await?
        } else {
            // Get recent memories
            let limit = query.limit as i64;
            self.connection
                .call(move |conn| {
                    let mut stmt = conn.prepare(
                        r#"
                        SELECT id, memory_type, content, entities, confidence,
                               created_at, updated_at, tags, metadata
                        FROM memories
                        ORDER BY updated_at DESC
                        LIMIT ?1
                        "#,
                    )?;

                    let memory_iter = stmt.query_map([limit], |row| {
                        let entities_json: String = row.get("entities")?;
                        let tags_json: String = row.get("tags")?;
                        let metadata_json: String = row.get("metadata")?;
                        let created_at_str: String = row.get("created_at")?;
                        let updated_at_str: String = row.get("updated_at")?;
                        let memory_type_str: String = row.get("memory_type")?;

                        let entities: Vec<String> =
                            serde_json::from_str(&entities_json).unwrap_or_default();
                        let tags: Vec<String> =
                            serde_json::from_str(&tags_json).unwrap_or_default();
                        let metadata: HashMap<String, String> =
                            serde_json::from_str(&metadata_json).unwrap_or_default();

                        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now());
                        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now());

                        Ok(Memory {
                            id: row.get("id")?,
                            memory_type: Self::string_to_memory_type(&memory_type_str),
                            content: row.get("content")?,
                            entities,
                            confidence: row.get("confidence")?,
                            created_at,
                            updated_at,
                            tags,
                            metadata,
                        })
                    })?;

                    let mut memories = Vec::new();
                    for memory_result in memory_iter {
                        memories.push(memory_result?);
                    }

                    Ok(memories)
                })
                .await?
        };

        // Apply filters
        if let Some(memory_types) = &query.memory_types {
            memories.retain(|m| memory_types.contains(&m.memory_type));
        }

        if let Some(entities) = &query.entities {
            memories.retain(|m| m.entities.iter().any(|e| entities.contains(e)));
        }

        if let Some(tags) = &query.tags {
            memories.retain(|m| m.tags.iter().any(|t| tags.contains(t)));
        }

        // Filter by confidence
        memories.retain(|m| m.confidence >= query.min_confidence);

        // Limit results
        memories.truncate(query.limit);

        Ok(memories)
    }

    async fn update_memory(&mut self, memory: Memory) -> Result<()> {
        // Same as store_memory for SQLite (INSERT OR REPLACE)
        self.store_memory(memory).await
    }

    async fn delete_memory(&mut self, id: &str) -> Result<bool> {
        let id = id.to_string();
        let rows_affected = self
            .connection
            .call(move |conn| {
                let rows = conn.execute("DELETE FROM memories WHERE id = ?1", [id.as_str()])?;
                Ok(rows)
            })
            .await?;

        Ok(rows_affected > 0)
    }

    async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        let id = id.to_string();
        let memory = self
            .connection
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, memory_type, content, entities, confidence,
                           created_at, updated_at, tags, metadata
                    FROM memories WHERE id = ?1
                    "#,
                )?;

                let mut rows = stmt.query_map([id.as_str()], |row| {
                    let entities_json: String = row.get("entities")?;
                    let tags_json: String = row.get("tags")?;
                    let metadata_json: String = row.get("metadata")?;
                    let created_at_str: String = row.get("created_at")?;
                    let updated_at_str: String = row.get("updated_at")?;
                    let memory_type_str: String = row.get("memory_type")?;

                    let entities: Vec<String> =
                        serde_json::from_str(&entities_json).unwrap_or_default();
                    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                    let metadata: HashMap<String, String> =
                        serde_json::from_str(&metadata_json).unwrap_or_default();

                    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());
                    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    Ok(Memory {
                        id: row.get("id")?,
                        memory_type: Self::string_to_memory_type(&memory_type_str),
                        content: row.get("content")?,
                        entities,
                        confidence: row.get("confidence")?,
                        created_at,
                        updated_at,
                        tags,
                        metadata,
                    })
                })?;

                if let Some(row) = rows.next() {
                    Ok(Some(row?))
                } else {
                    Ok(None)
                }
            })
            .await?;

        Ok(memory)
    }

    async fn get_all_memories(&self) -> Result<Vec<Memory>> {
        let memories = self
            .connection
            .call(|conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, memory_type, content, entities, confidence,
                           created_at, updated_at, tags, metadata
                    FROM memories
                    ORDER BY updated_at DESC
                    "#,
                )?;

                let memory_iter = stmt.query_map([], |row| {
                    let entities_json: String = row.get("entities")?;
                    let tags_json: String = row.get("tags")?;
                    let metadata_json: String = row.get("metadata")?;
                    let created_at_str: String = row.get("created_at")?;
                    let updated_at_str: String = row.get("updated_at")?;
                    let memory_type_str: String = row.get("memory_type")?;

                    let entities: Vec<String> =
                        serde_json::from_str(&entities_json).unwrap_or_default();
                    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                    let metadata: HashMap<String, String> =
                        serde_json::from_str(&metadata_json).unwrap_or_default();

                    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());
                    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    Ok(Memory {
                        id: row.get("id")?,
                        memory_type: Self::string_to_memory_type(&memory_type_str),
                        content: row.get("content")?,
                        entities,
                        confidence: row.get("confidence")?,
                        created_at,
                        updated_at,
                        tags,
                        metadata,
                    })
                })?;

                let mut memories = Vec::new();
                for memory_result in memory_iter {
                    memories.push(memory_result?);
                }

                Ok(memories)
            })
            .await?;

        Ok(memories)
    }

    async fn clear_memories(&mut self) -> Result<()> {
        self.connection
            .call(|conn| {
                conn.execute("DELETE FROM memories", [])?;
                Ok(())
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_sqlite_memory_store_basic_operations() {
        let mut store = SqliteMemoryStore::new_in_memory().await.unwrap();

        // Create a test memory
        let memory = Memory {
            id: "test-1".to_string(),
            memory_type: MemoryType::Fact,
            content: "The user likes coffee".to_string(),
            entities: vec!["user".to_string(), "coffee".to_string()],
            confidence: 0.9,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["preference".to_string()],
            metadata: HashMap::new(),
        };

        // Store the memory
        store.store_memory(memory.clone()).await.unwrap();

        // Retrieve the memory
        let retrieved = store.get_memory("test-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.content, "The user likes coffee");

        // Query memories
        let query = MemoryQuery {
            content: "coffee".to_string(),
            ..Default::default()
        };
        let results = store.query_memories(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test-1");

        // Delete the memory
        let deleted = store.delete_memory("test-1").await.unwrap();
        assert!(deleted);

        // Verify deletion
        let retrieved = store.get_memory("test-1").await.unwrap();
        assert!(retrieved.is_none());
    }
}
