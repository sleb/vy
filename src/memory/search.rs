//! Semantic search functionality for the memory system
//!
//! This module provides advanced search capabilities including:
//! - Vector similarity search
//! - Hybrid text + semantic search
//! - Memory ranking and relevance scoring
//! - Search result optimization

use crate::memory::{Embedding, EmbeddingProvider, Memory, MemoryQuery, MemoryStore, MemoryType};
use anyhow::Result;
use std::collections::HashMap;

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memory: Memory,
    pub relevance_score: f32,
    pub similarity_score: f32,
    pub text_match_score: f32,
}

/// Advanced memory search engine
pub struct MemorySearchEngine<S: MemoryStore, E: EmbeddingProvider> {
    store: S,
    embeddings: E,
}

impl<S: MemoryStore, E: EmbeddingProvider> MemorySearchEngine<S, E> {
    pub fn new(store: S, embeddings: E) -> Self {
        Self { store, embeddings }
    }

    /// Perform semantic search using vector similarity
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        similarity_threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the query
        let query_embedding = self.embeddings.generate_embedding(query).await?;

        // Get all memories (in a real implementation, we'd use a vector database)
        let all_memories = self.store.get_all_memories().await?;

        let mut results = Vec::new();

        // Calculate similarity for each memory
        for memory in all_memories {
            if let Some(embedding_str) = memory.metadata.get("embedding") {
                if let Ok(memory_embedding) = serde_json::from_str::<Embedding>(embedding_str) {
                    let similarity = self
                        .embeddings
                        .cosine_similarity(&query_embedding, &memory_embedding);

                    if similarity >= similarity_threshold {
                        results.push(SearchResult {
                            memory,
                            relevance_score: similarity,
                            similarity_score: similarity,
                            text_match_score: 0.0, // Will be calculated if needed
                        });
                    }
                }
            }
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(limit);

        Ok(results)
    }

    /// Perform hybrid search combining semantic similarity and text matching
    pub async fn hybrid_search(
        &self,
        query: &str,
        limit: usize,
        semantic_weight: f32,
        text_weight: f32,
    ) -> Result<Vec<SearchResult>> {
        // Get semantic search results
        let semantic_results = self.semantic_search(query, limit * 2, 0.1).await?;

        // Calculate text match scores
        let mut hybrid_results = Vec::new();

        for mut result in semantic_results {
            let text_score = self.calculate_text_match_score(query, &result.memory.content);
            result.text_match_score = text_score;

            // Calculate combined relevance score
            result.relevance_score =
                (result.similarity_score * semantic_weight) + (text_score * text_weight);

            hybrid_results.push(result);
        }

        // Sort by combined relevance score
        hybrid_results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        hybrid_results.truncate(limit);

        Ok(hybrid_results)
    }

    /// Search for memories related to specific entities
    pub async fn entity_search(
        &self,
        entities: &[String],
        memory_types: Option<Vec<MemoryType>>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query = MemoryQuery {
            entities: Some(entities.to_vec()),
            memory_types,
            limit,
            ..Default::default()
        };

        let memories = self.store.query_memories(&query).await?;

        let results = memories
            .into_iter()
            .map(|memory| SearchResult {
                memory,
                relevance_score: 1.0, // Entity matches are highly relevant
                similarity_score: 0.0,
                text_match_score: 0.0,
            })
            .collect();

        Ok(results)
    }

    /// Find memories similar to a given memory
    pub async fn find_similar_memories(
        &self,
        target_memory: &Memory,
        limit: usize,
        exclude_self: bool,
    ) -> Result<Vec<SearchResult>> {
        let results = self
            .semantic_search(&target_memory.content, limit + 1, 0.3)
            .await?;

        let filtered_results = if exclude_self {
            results
                .into_iter()
                .filter(|result| result.memory.id != target_memory.id)
                .take(limit)
                .collect()
        } else {
            results.into_iter().take(limit).collect()
        };

        Ok(filtered_results)
    }

    /// Search for memories by time range
    pub async fn temporal_search(
        &self,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let all_memories = self.store.get_all_memories().await?;

        let filtered_memories: Vec<Memory> = all_memories
            .into_iter()
            .filter(|memory| {
                let created_at = memory.created_at;

                if let Some(start) = start_time {
                    if created_at < start {
                        return false;
                    }
                }

                if let Some(end) = end_time {
                    if created_at > end {
                        return false;
                    }
                }

                true
            })
            .take(limit)
            .collect();

        let results = filtered_memories
            .into_iter()
            .map(|memory| SearchResult {
                memory,
                relevance_score: 1.0,
                similarity_score: 0.0,
                text_match_score: 0.0,
            })
            .collect();

        Ok(results)
    }

    /// Calculate text match score using simple heuristics
    fn calculate_text_match_score(&self, query: &str, content: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        if query_words.is_empty() || content_words.is_empty() {
            return 0.0;
        }

        // Calculate word overlap
        let mut matches = 0;
        for query_word in &query_words {
            if content_words
                .iter()
                .any(|word| word.contains(query_word) || query_word.contains(word))
            {
                matches += 1;
            }
        }

        let overlap_score = matches as f32 / query_words.len() as f32;

        // Boost score for exact phrase matches
        let phrase_boost = if content_lower.contains(&query_lower) {
            0.3
        } else {
            0.0
        };

        // Boost score for keyword density
        let density_boost = (matches as f32 / content_words.len() as f32) * 0.2;

        (overlap_score + phrase_boost + density_boost).min(1.0)
    }

    /// Rank search results based on multiple factors
    pub fn rank_results(&self, mut results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        for result in &mut results {
            let mut ranking_score = result.relevance_score;

            // Boost recent memories
            let days_old = (chrono::Utc::now() - result.memory.created_at).num_days() as f32;
            let recency_boost = (1.0 / (1.0 + days_old * 0.1)).min(0.3);
            ranking_score += recency_boost;

            // Boost high-confidence memories
            let confidence_boost = (result.memory.confidence - 0.5) * 0.2;
            ranking_score += confidence_boost;

            // Boost memories with relevant tags
            let tag_boost = self.calculate_tag_relevance(&result.memory.tags, query) * 0.1;
            ranking_score += tag_boost;

            result.relevance_score = ranking_score;
        }

        // Sort by final ranking score
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Calculate tag relevance to query
    fn calculate_tag_relevance(&self, tags: &[String], query: &str) -> f32 {
        if tags.is_empty() {
            return 0.0;
        }

        let query_lower = query.to_lowercase();
        let mut relevance = 0.0;

        for tag in tags {
            let tag_lower = tag.to_lowercase();
            if query_lower.contains(&tag_lower) || tag_lower.contains(&query_lower) {
                relevance += 1.0;
            }
        }

        relevance / tags.len() as f32
    }

    /// Get search suggestions based on query
    pub async fn get_search_suggestions(
        &self,
        partial_query: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        // Get all memories and extract common terms
        let all_memories = self.store.get_all_memories().await?;
        let mut term_counts: HashMap<String, usize> = HashMap::new();

        for memory in &all_memories {
            // Extract terms from content
            let content_lower = memory.content.to_lowercase();
            let words: Vec<&str> = content_lower.split_whitespace().collect();
            for word in words {
                if word.len() > 2 {
                    *term_counts.entry(word.to_string()).or_insert(0) += 1;
                }
            }

            // Extract terms from tags
            for tag in &memory.tags {
                *term_counts.entry(tag.to_lowercase()).or_insert(0) += 1;
            }

            // Extract terms from entities
            for entity in &memory.entities {
                *term_counts.entry(entity.to_lowercase()).or_insert(0) += 1;
            }
        }

        // Filter suggestions that match partial query
        let partial_lower = partial_query.to_lowercase();
        let mut suggestions: Vec<(String, usize)> = term_counts
            .into_iter()
            .filter(|(term, _)| term.starts_with(&partial_lower) || term.contains(&partial_lower))
            .collect();

        // Sort by frequency (descending)
        suggestions.sort_by(|a, b| b.1.cmp(&a.1));

        // Return top suggestions
        Ok(suggestions
            .into_iter()
            .take(limit)
            .map(|(term, _)| term)
            .collect())
    }

    /// Generate search analytics
    pub async fn get_search_analytics(&self) -> Result<SearchAnalytics> {
        let all_memories = self.store.get_all_memories().await?;

        let mut analytics = SearchAnalytics {
            total_memories: all_memories.len(),
            ..Default::default()
        };

        let mut type_counts: HashMap<MemoryType, usize> = HashMap::new();
        let mut entity_counts: HashMap<String, usize> = HashMap::new();
        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for memory in &all_memories {
            *type_counts.entry(memory.memory_type.clone()).or_insert(0) += 1;

            for entity in &memory.entities {
                *entity_counts.entry(entity.clone()).or_insert(0) += 1;
            }

            for tag in &memory.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        analytics.memory_type_distribution = type_counts;
        analytics.top_entities = entity_counts.into_iter().collect::<Vec<_>>();
        analytics.top_entities.sort_by(|a, b| b.1.cmp(&a.1));
        analytics.top_entities.truncate(10);

        analytics.top_tags = tag_counts.into_iter().collect::<Vec<_>>();
        analytics.top_tags.sort_by(|a, b| b.1.cmp(&a.1));
        analytics.top_tags.truncate(10);

        Ok(analytics)
    }
}

#[derive(Debug, Default)]
pub struct SearchAnalytics {
    pub total_memories: usize,
    pub memory_type_distribution: HashMap<MemoryType, usize>,
    pub top_entities: Vec<(String, usize)>,
    pub top_tags: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{MockEmbeddingProvider, SqliteMemoryStore};

    #[tokio::test]
    async fn test_text_match_score() {
        let store = SqliteMemoryStore::new_in_memory().await.unwrap();
        let embeddings = MockEmbeddingProvider::new(256);
        let search_engine = MemorySearchEngine::new(store, embeddings);

        let score1 =
            search_engine.calculate_text_match_score("coffee", "I love coffee in the morning");
        assert!(score1 > 0.0);

        let score2 = search_engine
            .calculate_text_match_score("coffee beans", "I love coffee and fresh beans");
        assert!(score2 > score1); // Should score higher due to multiple matches

        let score3 =
            search_engine.calculate_text_match_score("tea", "I love coffee in the morning");
        assert!(score3 == 0.0); // No match
    }

    #[tokio::test]
    async fn test_tag_relevance() {
        let store = SqliteMemoryStore::new_in_memory().await.unwrap();
        let embeddings = MockEmbeddingProvider::new(256);
        let search_engine = MemorySearchEngine::new(store, embeddings);

        let tags = vec![
            "coffee".to_string(),
            "morning".to_string(),
            "drink".to_string(),
        ];

        let relevance1 = search_engine.calculate_tag_relevance(&tags, "coffee");
        assert!(relevance1 > 0.0);

        let relevance2 = search_engine.calculate_tag_relevance(&tags, "tea");
        assert!(relevance2 == 0.0);

        let relevance3 = search_engine.calculate_tag_relevance(&[], "coffee");
        assert!(relevance3 == 0.0);
    }

    #[tokio::test]
    async fn test_entity_search() {
        let store = SqliteMemoryStore::new_in_memory().await.unwrap();
        let embeddings = MockEmbeddingProvider::new(256);
        let mut search_engine = MemorySearchEngine::new(store, embeddings);

        // Store a test memory
        let memory = Memory::new(
            MemoryType::Fact,
            "User likes coffee".to_string(),
            vec!["user".to_string(), "coffee".to_string()],
        );
        search_engine.store.store_memory(memory).await.unwrap();

        // Search for memories containing "user" entity
        let results = search_engine
            .entity_search(&["user".to_string()], None, 10)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].memory.entities.contains(&"user".to_string()));
    }
}
