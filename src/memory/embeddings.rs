//! Embedding providers for memory system
//!
//! This module provides interfaces and implementations for generating
//! vector embeddings of text content for semantic similarity search.

use anyhow::Result;
use async_trait::async_trait;
use rig::embeddings::EmbeddingModel;
use serde::{Deserialize, Serialize};

/// Vector embedding representation
pub type Embedding = Vec<f32>;

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for the given text
    async fn generate_embedding(&self, text: &str) -> Result<Embedding>;

    /// Generate embeddings for multiple texts
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Embedding>>;

    /// Calculate cosine similarity between two embeddings
    fn cosine_similarity(&self, a: &Embedding, b: &Embedding) -> f32;

    /// Get the dimensionality of embeddings produced by this provider
    fn embedding_dimension(&self) -> usize;
}

/// OpenAI embedding provider using rig-core
pub struct RigEmbeddingProvider<M: EmbeddingModel> {
    model: M,
    dimension: usize,
}

impl<M: EmbeddingModel> RigEmbeddingProvider<M> {
    pub fn new(model: M, dimension: usize) -> Self {
        Self { model, dimension }
    }
}

#[async_trait]
impl<M: EmbeddingModel + Send + Sync> EmbeddingProvider for RigEmbeddingProvider<M> {
    async fn generate_embedding(&self, text: &str) -> Result<Embedding> {
        let embedding = self.model.embed_text(text).await?;
        Ok(embedding.vec.into_iter().map(|x| x as f32).collect())
    }

    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.generate_embedding(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    fn cosine_similarity(&self, a: &Embedding, b: &Embedding) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }
}

/// Mock embedding provider for testing
#[derive(Clone)]
pub struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn generate_embedding(&self, text: &str) -> Result<Embedding> {
        // Generate a deterministic but pseudo-random embedding based on text hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = Vec::with_capacity(self.dimension);
        let mut seed = hash;

        for _ in 0..self.dimension {
            // Simple LCG for deterministic "random" values
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let value = ((seed >> 16) & 0x7fff) as f32 / 32767.0 - 0.5;
            embedding.push(value);
        }

        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        Ok(embedding)
    }

    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.generate_embedding(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    fn cosine_similarity(&self, a: &Embedding, b: &Embedding) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }
}

/// Configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub dimension: usize,
    pub api_key: Option<String>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
            api_key: None,
        }
    }
}

/// Utility functions for embedding operations
pub struct EmbeddingUtils;

impl EmbeddingUtils {
    /// Batch embeddings into chunks to avoid rate limits
    pub fn chunk_texts(texts: &[String], chunk_size: usize) -> Vec<Vec<String>> {
        texts
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Find the most similar embeddings to a query embedding
    pub fn find_most_similar(
        query_embedding: &Embedding,
        candidate_embeddings: &[(String, Embedding)],
        provider: &dyn EmbeddingProvider,
        top_k: usize,
    ) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = candidate_embeddings
            .iter()
            .map(|(id, embedding)| {
                let similarity = provider.cosine_similarity(query_embedding, embedding);
                (id.clone(), similarity)
            })
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k
        similarities.into_iter().take(top_k).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_provider() {
        let provider = MockEmbeddingProvider::new(256);

        let text1 = "Hello world";
        let text2 = "Hello world";
        let text3 = "Goodbye world";

        let embedding1 = provider.generate_embedding(text1).await.unwrap();
        let embedding2 = provider.generate_embedding(text2).await.unwrap();
        let embedding3 = provider.generate_embedding(text3).await.unwrap();

        assert_eq!(embedding1.len(), 256);
        assert_eq!(embedding2.len(), 256);
        assert_eq!(embedding3.len(), 256);

        // Same text should produce identical embeddings
        assert_eq!(embedding1, embedding2);

        // Different text should produce different embeddings
        assert_ne!(embedding1, embedding3);

        // Similarity between identical embeddings should be 1.0
        let similarity_identical = provider.cosine_similarity(&embedding1, &embedding2);
        assert!((similarity_identical - 1.0).abs() < 1e-6);

        // Similarity between different embeddings should be less than 1.0
        let similarity_different = provider.cosine_similarity(&embedding1, &embedding3);
        assert!(similarity_different < 1.0);
    }

    #[tokio::test]
    async fn test_batch_embeddings() {
        let provider = MockEmbeddingProvider::new(128);
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = provider.generate_embeddings(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);

        for embedding in &embeddings {
            assert_eq!(embedding.len(), 128);
        }
    }

    #[test]
    fn test_embedding_utils() {
        let texts = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];

        let chunks = EmbeddingUtils::chunk_texts(&texts, 2);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 2);
        assert_eq!(chunks[1].len(), 2);
        assert_eq!(chunks[2].len(), 1);
    }
}
