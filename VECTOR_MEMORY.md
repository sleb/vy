# Vector Memory System for Vy

This document describes Vy's new vector-based memory system that uses cloud-based vector databases for semantic search and enhanced memory capabilities.

## Overview

The vector memory system replaces simple text-based memory storage with semantic vector embeddings, enabling:

- **Semantic search**: Find memories based on meaning, not just keywords
- **Cloud storage**: Memories stored in managed vector databases
- **Better recall**: Related concepts discovered even with different wording
- **Scalability**: Handles thousands of memories efficiently
- **Multi-device sync**: Access memories from any device

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Conversation  │───▶│   Fact Extract   │───▶│  Generate       │
│                 │    │   (LLM-based)    │    │  Embeddings     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
                                                         ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  Semantic       │◀───│   Vector DB      │◀───│   Store Vector  │
│  Search         │    │   (Qdrant)       │    │   + Metadata    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Quick Start

### Prerequisites

1. **Qdrant Database**: Either cloud or local
   - **Cloud**: Sign up at [Qdrant Cloud](https://cloud.qdrant.io/)
   - **Local**: `docker run -p 6334:6334 qdrant/qdrant`

2. **OpenAI API Key**: For generating embeddings
   - Get from [OpenAI Platform](https://platform.openai.com/api-keys)

### Configuration

```rust
use vy_core::vector_memory::{VectorMemory, VectorMemoryConfig};

let config = VectorMemoryConfig {
    qdrant_url: "https://your-cluster.qdrant.cloud".to_string(),
    qdrant_api_key: Some("your-qdrant-api-key".to_string()),
    collection_name: "vy_memories".to_string(),
    openai_api_key: "your-openai-api-key".to_string(),
    embedding_model: "text-embedding-3-small".to_string(),
};

let vector_memory = VectorMemory::new(config).await?;
```

### Basic Usage

```rust
// Store a memory
let memory = MemoryEntry::new(
    "User works as a software engineer at Google".to_string(),
    "conversation_123".to_string(),
);
vector_memory.store_memory(&memory).await?;

// Search memories semantically
let results = vector_memory
    .search_memories("programming job", 5)
    .await?;

for memory in results {
    println!("Found: {}", memory.fact);
}
```

## Supported Vector Databases

### Qdrant Cloud (Recommended)

**Pros:**
- Managed service (no infrastructure)
- Excellent Rust integration
- Hybrid search (vector + filters)
- Good free tier
- Auto-scaling

**Setup:**
1. Sign up at [cloud.qdrant.io](https://cloud.qdrant.io/)
2. Create a cluster
3. Get API key and cluster URL

```rust
let config = VectorMemoryConfig {
    qdrant_url: "https://xyz-abc-def.us-east.aws.cloud.qdrant.io".to_string(),
    qdrant_api_key: Some("your-api-key".to_string()),
    collection_name: "vy_memories".to_string(),
    openai_api_key: "sk-...".to_string(),
    embedding_model: "text-embedding-3-small".to_string(),
};
```

### Local Qdrant

**Pros:**
- No cloud costs
- Full control
- Low latency

**Setup:**
```bash
docker run -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant
```

```rust
let config = VectorMemoryConfig {
    qdrant_url: "http://localhost:6334".to_string(),
    qdrant_api_key: None,
    collection_name: "vy_memories".to_string(),
    openai_api_key: "sk-...".to_string(),
    embedding_model: "text-embedding-3-small".to_string(),
};
```

## Migration from File-Based Memory

To migrate existing memories from the JSON file system:

```rust
use vy_core::memory::{Memory, default_memory_file};
use vy_core::vector_memory::{VectorMemory, VectorMemoryConfig};

// Load existing file-based memories
let memory_file = default_memory_file()?;
let mut old_memory = Memory::new(memory_file);
old_memory.load().await?;

// Get all existing memories
let entries = old_memory.get_all_entries();

// Initialize vector memory
let vector_memory = VectorMemory::new(vector_config).await?;

// Migrate each memory
for entry in entries {
    vector_memory.store_memory(entry).await?;
}

println!("Migrated {} memories to vector database", entries.len());
```

## Configuration Options

### VectorMemoryConfig Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `qdrant_url` | `String` | Qdrant database URL | `http://localhost:6334` |
| `qdrant_api_key` | `Option<String>` | API key for cloud instances | `None` |
| `collection_name` | `String` | Collection name for memories | `vy_memories` |
| `openai_api_key` | `String` | OpenAI API key for embeddings | Required |
| `embedding_model` | `String` | OpenAI embedding model | `text-embedding-3-small` |

### Embedding Models

| Model | Dimensions | Cost | Use Case |
|-------|------------|------|----------|
| `text-embedding-3-small` | 1536 | Low | General purpose (recommended) |
| `text-embedding-3-large` | 3072 | High | Maximum quality |
| `text-embedding-ada-002` | 1536 | Low | Legacy (still supported) |

## API Reference

### VectorMemory

#### Methods

```rust
// Create new instance
pub async fn new(config: VectorMemoryConfig) -> Result<Self>

// Store a memory
pub async fn store_memory(&self, entry: &MemoryEntry) -> Result<u64>

// Search memories semantically
pub async fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>

// Get all memories
pub async fn get_all_memories(&self) -> Result<Vec<MemoryEntry>>

// Delete specific memory
pub async fn delete_memory(&self, timestamp: DateTime<Utc>) -> Result<bool>

// Clear all memories
pub async fn clear_all(&self) -> Result<()>

// Get statistics
pub async fn get_stats(&self) -> Result<VectorMemoryStats>

// Learn from conversation
pub async fn learn_from_conversation(
    &self,
    conversation: &str,
    source: &str,
    memory_model_id: &str,
) -> Result<Vec<String>>
```

## Examples

### Running the Demo

```bash
# Start Qdrant locally
docker run -p 6334:6334 qdrant/qdrant

# Set your OpenAI API key
export OPENAI_API_KEY=sk-your-key-here

# Run the demo
cargo run --example vector_memory_demo --package vy-core
```

### Semantic Search Examples

The vector memory can find related concepts:

```rust
// Store: "User works as a software engineer at Google"
// Search: "programming job" ✅ Found!
// Search: "tech career" ✅ Found!
// Search: "coding work" ✅ Found!

// Store: "User loves hiking in the mountains"
// Search: "outdoor activities" ✅ Found!
// Search: "weekend adventures" ✅ Found!
// Search: "nature walks" ✅ Found!
```

### Advanced Usage

```rust
// Batch operations
let memories = vec![
    MemoryEntry::new("User prefers morning meetings".to_string(), "chat_1".to_string()),
    MemoryEntry::new("User drinks coffee daily".to_string(), "chat_2".to_string()),
    MemoryEntry::new("User has dog named Rex".to_string(), "chat_3".to_string()),
];

for memory in memories {
    vector_memory.store_memory(&memory).await?;
}

// Search with different strategies
let work_memories = vector_memory.search_memories("professional life", 10).await?;
let personal_memories = vector_memory.search_memories("personal preferences", 10).await?;

// Get comprehensive statistics
let stats = vector_memory.get_stats().await?;
println!("Total memories: {}", stats.total_entries);
println!("Date range: {:?} to {:?}", stats.oldest_entry, stats.newest_entry);
```

## Performance Considerations

### Embedding Generation
- **Cost**: ~$0.0001 per 1K tokens with `text-embedding-3-small`
- **Latency**: ~100-200ms per embedding
- **Batching**: Process multiple memories together when possible

### Vector Search
- **Speed**: Sub-100ms searches with proper indexing
- **Accuracy**: Cosine similarity with 1536-dimensional vectors
- **Scale**: Handles millions of vectors efficiently

### Memory Usage
- **Local**: ~6KB per memory entry (including vector)
- **Cloud**: Depends on Qdrant pricing tier
- **Bandwidth**: ~6KB upload per memory, ~1KB per search

## Troubleshooting

### Common Issues

**Connection Failed**
```
Error: Failed to connect to Qdrant
```
- Check Qdrant URL and API key
- Ensure Qdrant service is running
- Verify network connectivity

**Embedding Failed**
```
Error: Failed to request embedding from OpenAI
```
- Check OpenAI API key
- Verify API quota/billing
- Check rate limits

**Collection Not Found**
```
Error: Collection 'vy_memories' not found
```
- Collection is auto-created on first use
- Check Qdrant permissions
- Verify collection name

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run --example vector_memory_demo --package vy-core
```

## Roadmap

### Planned Features
- [ ] Multiple vector database support (Weaviate, Pinecone, Chroma)
- [ ] Hybrid search (semantic + keyword)
- [ ] Memory clustering and categorization
- [ ] Automatic memory cleanup/archival
- [ ] Memory sharing between users
- [ ] Custom embedding models
- [ ] Memory analytics and insights

### Integration Goals
- [ ] CLI commands for vector memory management
- [ ] Configuration file support for vector settings
- [ ] Automatic migration from file-based memory
- [ ] TUI interface for memory exploration
- [ ] Web dashboard for memory management

## Contributing

The vector memory system is designed to be extensible. To add support for other vector databases:

1. Implement the core traits in `src/vector_memory.rs`
2. Add configuration options
3. Create integration tests
4. Update documentation

See the existing Qdrant implementation as a reference.

## License

Vector memory system is part of Vy and follows the same MIT license.
