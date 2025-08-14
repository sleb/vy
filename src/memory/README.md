# Vy Memory System

## Overview

The Vy Memory System provides long-term memory capabilities for the Vy AI assistant, enabling it to remember facts, relationships, opinions, and other important information across conversations. This system uses a hybrid approach combining structured storage with semantic search to provide efficient and meaningful memory retrieval.

## Architecture

### Core Components

1. **Memory Types** - Categorized storage for different kinds of information
2. **Storage Layer** - Persistent storage using SQLite with optional vector database integration
3. **Embedding System** - Semantic similarity search using vector embeddings
4. **Memory Manager** - High-level interface for memory operations
5. **Search Engine** - Advanced querying and ranking capabilities

### Memory Types

The system supports six primary memory types:

- **Fact** - Objective information about entities, events, or concepts
- **Opinion** - User preferences, likes, dislikes, and subjective views
- **Personal** - Information about the user (name, location, background, etc.)
- **Relationship** - Connections between entities and people
- **Conversation** - Important excerpts from past conversations
- **Knowledge** - General insights and learned information

## Implementation Status

### ✅ Completed Components

- **Core Data Structures** (`mod.rs`)
  - Memory struct with metadata, confidence scoring, and timestamps
  - MemoryType enum with comprehensive categorization
  - MemoryQuery interface for flexible searching
  - Memory statistics and analytics

- **Storage Backend** (`storage.rs`)
  - SQLite-based persistent storage
  - Async interface using tokio-rusqlite
  - Schema with indexing for performance
  - CRUD operations with error handling
  - Memory filtering and basic text search

- **Embedding Interface** (`embeddings.rs`)
  - Trait-based abstraction for embedding providers
  - OpenAI embedding integration via rig-core
  - Mock provider for testing and development
  - Cosine similarity calculations
  - Batch processing utilities

- **Search Engine** (`search.rs`)
  - Semantic similarity search
  - Hybrid text + vector search
  - Memory ranking with multiple factors
  - Entity-based search
  - Temporal queries
  - Search suggestions and analytics

- **CLI Demo** (`cli/memory_demo.rs`)
  - Basic command interface
  - Status reporting and feature overview

### 🚧 In Development

- **Full CLI Commands** (`cli/memory.rs`)
  - Complete CRUD operations
  - Search and filtering
  - Import/export functionality
  - Memory statistics dashboard
  - *Status: Needs refactoring due to ownership issues*

### 📋 Planned Features

- **Chat Integration**
  - Automatic memory extraction from conversations
  - Context-aware memory injection into prompts
  - Confidence-based memory validation
  - Real-time memory updates during chat

- **Advanced Search**
  - Vector database integration (ChromaDB/Qdrant)
  - Graph-based relationship queries
  - Fuzzy matching and typo tolerance
  - Relevance scoring improvements

- **Memory Management**
  - Memory consolidation and deduplication
  - Automatic confidence decay over time
  - Memory validation and fact-checking
  - User feedback integration

## Usage Examples

### Basic Memory Operations

```rust
use vy::memory::{Memory, MemoryType, MemoryManager, SqliteMemoryStore, MockEmbeddingProvider};

// Initialize storage and embeddings
let store = SqliteMemoryStore::new("memories.db").await?;
let embeddings = MockEmbeddingProvider::new(256);
let mut manager = MemoryManager::new(store, embeddings);

// Create and store a memory
let memory = Memory::new(
    MemoryType::Personal,
    "User's name is Alice and she lives in San Francisco".to_string(),
    vec!["Alice".to_string(), "San Francisco".to_string()]
);
manager.add_memory(memory).await?;

// Search for relevant memories
let results = manager.get_relevant_memories("Tell me about Alice", 5).await?;
```

### Memory Extraction from Conversations

```rust
// Extract memories from a conversation
let memories = manager.extract_memories_from_conversation(
    "I love hiking in the mountains on weekends",
    "That sounds wonderful! Mountain hiking is great exercise."
).await?;

// Memories are automatically categorized and stored
```

### CLI Usage

```bash
# Show memory system demo
vy memory demo

# Future CLI commands (when implemented):
vy memory stats                    # Show memory statistics
vy memory list --limit 10          # List recent memories
vy memory search "coffee"          # Search for coffee-related memories
vy memory add --type fact "User likes tea"  # Add a memory manually
vy memory export --output backup.json       # Export all memories
```

## Configuration

### Database Configuration

Memories are stored in SQLite by default. The database location is determined by:

1. User data directory: `~/.local/share/vy/memories.db` (Linux/macOS)
2. Can be configured via environment variables or config file

### Embedding Providers

The system supports multiple embedding providers:

- **OpenAI** (via rig-core) - Production use with API key
- **Mock Provider** - Development and testing
- **Future**: Local embeddings, Hugging Face models

### Memory Limits

Current default limits:
- Memory content: 10,000 characters
- Entities per memory: 50
- Tags per memory: 20
- Search results: 100 per query

## Development Guide

### Adding New Memory Types

1. Add variant to `MemoryType` enum in `mod.rs`
2. Update storage serialization in `storage.rs`
3. Add extraction logic in memory manager
4. Update CLI commands and documentation

### Implementing New Storage Backends

1. Implement the `MemoryStore` trait
2. Handle async operations and error cases
3. Add connection pooling for performance
4. Include migration support for schema changes

### Adding Embedding Providers

1. Implement the `EmbeddingProvider` trait
2. Handle API authentication and rate limits
3. Provide appropriate error handling
4. Add configuration options

## Performance Considerations

### Storage Performance

- SQLite with proper indexing handles 100k+ memories efficiently
- Consider connection pooling for high-concurrency scenarios
- Batch operations for bulk memory insertion

### Embedding Performance

- OpenAI embeddings: ~1000 requests/minute rate limit
- Consider caching embeddings for frequently accessed memories
- Use batch API calls when available

### Search Performance

- Text search scales linearly with memory count
- Vector similarity search requires specialized databases at scale
- Consider implementing search result caching

## Testing

### Unit Tests

```bash
cargo test memory::
```

### Integration Tests

```bash
# Test with real SQLite database
cargo test --test memory_integration

# Test with mock embeddings
cargo test --features mock_embeddings
```

### Performance Tests

```bash
# Benchmark memory operations
cargo bench --bench memory_bench
```

## Troubleshooting

### Common Issues

1. **Database Lock Errors**
   - Check for multiple concurrent connections
   - Ensure proper async handling
   - Consider connection timeouts

2. **Embedding API Failures**
   - Verify API key configuration
   - Check rate limit status
   - Implement exponential backoff

3. **Memory Search Issues**
   - Verify embedding generation
   - Check similarity thresholds
   - Review memory categorization

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=vy::memory=debug vy chat
```

## Future Roadmap

### Short Term (Next Release)
- [ ] Complete CLI command implementation
- [ ] Basic chat integration
- [ ] Memory extraction improvements
- [ ] Performance optimizations

### Medium Term
- [ ] Vector database integration
- [ ] Advanced search features
- [ ] Memory validation system
- [ ] User feedback integration

### Long Term
- [ ] Multi-user support
- [ ] Memory sharing between instances
- [ ] Advanced AI-powered memory curation
- [ ] Privacy and encryption features

## Contributing

When contributing to the memory system:

1. Follow the existing async patterns
2. Add comprehensive tests for new features
3. Update documentation for API changes
4. Consider backwards compatibility
5. Profile performance impact of changes

## License

The Vy Memory System is part of the Vy project and follows the same licensing terms.
