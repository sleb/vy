# Vy - AI-Powered Semantic Memory MCP Server

> An intelligent "second brain" that combines semantic memory with strategic project management through the Model Context Protocol (MCP).

## 🎯 Project Vision

Vy is an AI-powered semantic memory system that serves as your personal intelligent assistant. It remembers conversations, extracts insights, and helps with strategic thinking by providing context-aware suggestions and pattern recognition across your work and thoughts.

Think of it as a combination of:

- **Semantic Memory**: Vector-based storage and retrieval of conversations and documents
- **Strategic Intelligence**: Pattern recognition, goal alignment, and proactive insights
- **Intelligent Task Management**: Auto-extraction of TODOs with context-aware suggestions

## 🏗️ Architecture Overview

### Core Technologies

- **Language**: TypeScript (rich MCP ecosystem support, excellent AI/ML tooling)
- **Memory Strategy**: Curated memory with intelligent condensation
- **Vector Database**: ChromaDB (local-first development, cloud-scalable production)
- **Protocol**: Model Context Protocol (MCP) for AI assistant integration
- **Monorepo**: Turborepo for scalable multi-package development

### System Design Principles

- **Local-First**: Works offline, syncs when connected
- **Privacy-Focused**: Your data stays under your control
- **Modular Architecture**: Clean separation of concerns for maintainability
- **Modern Patterns**: Leveraging current best practices over backward compatibility

## 📦 Project Structure

```
vy/
├── packages/
│   ├── core/                    # Shared types, utilities, and interfaces
│   ├── mcp-server-basic/        # MVP MCP server implementation
│   ├── vector-store/            # ChromaDB abstraction layer
│   ├── eslint-config/           # Shared ESLint configuration
│   └── typescript-config/       # Shared TypeScript configuration
├── apps/
│   └── cli-test/               # CLI testing and development tool
├── docs/
│   ├── architecture/           # Architecture decision records (ADRs)
│   ├── api/                   # API documentation
│   └── guides/                # Implementation and usage guides
├── turbo.json                 # Turborepo configuration
├── package.json              # Root package configuration
└── README.md                 # This file
```

## 🚀 Implementation Phases

### Phase 1: MVP Foundation (Current Phase) 🔄 In Progress

**Goal**: Basic semantic memory with MCP integration

**Core Features**:

- `capture_conversation` - Store conversations with metadata and embeddings
- `search_memory` - Semantic search across stored memories
- `get_context` - Retrieve relevant memories for context injection
- Basic vector storage with ChromaDB
- MCP server implementation

**Key Architecture Decisions**:

- **Two-Layer Memory**: Session memory (raw, current) + Long-term memory (processed, persistent)
- **Raw-Only Storage**: Start with full conversations, add condensation in Phase 2
- **Single Collection**: All memories in `vy_memories` collection with type metadata
- **Embed at Storage**: Generate embeddings when storing memories
- **Environment Configuration**: Explicit local vs. hosted configurations

**Deliverables**:

- ✅ Monorepo setup with Turborepo
- ✅ Core types and interfaces (`packages/core`)
- ✅ ChromaDB abstraction layer (`packages/vector-store`)
- ⏳ Basic MCP server (`packages/mcp-server-basic`)
- ⏳ CLI testing tool (`apps/cli-test`)
- ⏳ Basic documentation and setup guides

### Phase 2: Enhanced Memory Intelligence 📋 Planned

**Goal**: Implement the full two-layer architecture

**Features**:

- **Session Processing Pipeline**: End-of-conversation insight extraction
- **Memory Condensation**: AI-powered summarization and insight extraction
- **Multiple Memory Types**: Insights, learnings, facts, action items
- **Context-aware Search**: Smart relevance ranking with recency weighting
- **Memory Categorization**: Automatic tagging and domain classification

### Phase 3: Strategic Intelligence 🎯 Planned

**Goal**: Proactive insights and intelligent assistance

**Features**:

- **Proactive Context Injection**: Automatically surface relevant memories
- **Goal Alignment Tracking**: Connect conversations to longer-term objectives
- **Pattern Recognition**: Identify recurring themes and behaviors
- **Strategic Suggestions**: Proactive recommendations based on memory patterns
- **Analytics Dashboard**: Insights into conversation patterns and productivity

### Phase 4: Advanced Integration 🔮 Future

**Goal**: Rich ecosystem integration

**Features**:

- Multiple MCP server implementations (specialized vs. general-purpose)
- Calendar and task system integration
- Document processing and analysis
- Team collaboration features

## 🛠️ Development Setup

### Prerequisites

- Node.js 18+ and npm
- Git
- OpenAI API key for embeddings
- ChromaDB instance (local or hosted)
- (Optional) Global Turborepo CLI: `npm install -g turbo`

### Quick Start

```bash
# Clone the repository
git clone <repository-url> vy
cd vy

# Install dependencies
npm install

# Set up environment variables
cp .env.example .env
# Edit .env with your OpenAI API key

# Start ChromaDB (local development)
docker run -p 8000:8000 chromadb/chroma

# Run development mode for all packages
turbo dev

# Build all packages
turbo build

# Run tests
turbo test

# Lint all packages
turbo lint
```

### Environment Variables

```bash
# Required
VY_OPENAI_API_KEY=your_openai_api_key_here

# Optional (with defaults for local development)
VY_CHROMA_HOST=localhost
VY_CHROMA_PORT=8000
VY_EMBEDDING_MODEL=text-embedding-3-small
VY_COLLECTION_NAME=vy_memories

# Required for hosted/production environments
VY_CHROMA_API_KEY=your_chroma_api_key
VY_CHROMA_SSL=true
```

### Package-Specific Commands

```bash
# Work on a specific package
turbo dev --filter=mcp-server-basic
turbo build --filter=vector-store
turbo test --filter=core

# Run tests
npm run test                    # All packages
turbo test --filter=vector-store # Specific package
cd packages/vector-store && npm run test # Direct test execution
```

## 🧠 Core Concepts & Design Decisions

### Two-Layer Memory Architecture

**Session Memory (Raw, Current)**:

- Full conversation fidelity during active sessions
- Available for context and clarification
- Discarded after insight extraction

**Long-term Memory (Processed, Persistent)**:

- Extracted insights, learnings, and key facts
- Optimized for semantic search and retrieval
- Condensed to overcome context window limitations

### Memory Types (Extensible Design)

- **Conversations** (Phase 1): Complete conversation threads with metadata
- **Insights** (Phase 2): Derived patterns and strategic learnings
- **Learnings** (Phase 2): Specific knowledge and facts
- **Facts** (Phase 2): Verifiable information and preferences
- **Action Items** (Phase 2): Extracted tasks and TODOs

### Vector Storage Strategy

- **Single Collection**: `vy_memories` with type-based metadata filtering
- **OpenAI Embeddings**: text-embedding-3-small (1536D, cost-optimal)
- **Embed at Storage**: Generate embeddings when storing memories
- **Failure Recovery**: Store raw data even when embedding generation fails
- **Rich Metadata**: Support temporal, type, and custom filtering
- **UUID v7 IDs**: Time-sortable identifiers for efficient database operations

### Configuration Patterns

- **Local Development**: `createLocalConfig()` - no auth required
- **Hosted Production**: `createHostedConfig()` - explicit auth requirements
- **Environment Variables**: Standard deployment pattern
- **Validation**: Type-safe configuration with clear error messages
- **Error Recovery**: Failed embeddings tracked for batch reprocessing

### Testing Strategy

**Comprehensive Unit Testing**:

- **Error Handling**: Validates data preservation when embeddings fail
- **Memory Conversion**: Tests complex domain object ↔ ChromaDB document mapping
- **Search Logic**: Verifies distance-to-similarity conversion and relevance filtering
- **ID Generation**: Ensures UUID v7 format compliance and uniqueness
- **Failed Embedding Recovery**: Tests automatic tracking and reprocessing system

**Test Coverage**: 10/10 tests passing, focusing on critical failure scenarios and data integrity

**Modern Testing Patterns**:

- **Behavior-driven**: Test what the code does, not how it's implemented
- **Mock strategy**: Mock external dependencies, test real domain logic
- **Error simulation**: Explicit testing of failure scenarios
- **Type safety**: Runtime validation of TypeScript interfaces

### MCP Integration

The Model Context Protocol provides:

- **Three Core Tools**: `capture_conversation`, `search_memory`, `get_context`
- **Resource Exposure**: Access to stored memories as resources
- **Context Injection**: Intelligent memory retrieval for new conversations
- **Type-Safe Interfaces**: Full TypeScript definitions for all operations

## 📚 Documentation

- [Architecture Decisions](docs/architecture/) - Key technical decisions and rationale
- [API Reference](docs/api/) - Detailed API documentation
- [Development Guides](docs/guides/) - Setup and contribution guides
- [MCP Integration](docs/guides/mcp-integration.md) - How to integrate with AI assistants

## 🤝 Development Philosophy

This project demonstrates:

- **Learning-Oriented Development**: Hands-on experience with modern TypeScript patterns
- **Design-First Approach**: Thoughtful architecture decisions before implementation
- **Modern Best Practices**: Current patterns over legacy compatibility
- **Clean Architecture**: Domain-driven design with clear separation of concerns
- **Iterative Progress**: Phased development with working increments
- **Living Documentation**: READMEs and code that evolve together

## 📈 Current Status

**Phase 1 Progress** (MVP Foundation):

- ✅ Turborepo monorepo setup with shared configs
- ✅ Core domain types and interfaces (`@repo/core`)
- ✅ Configuration system with local/hosted patterns
- ✅ OpenAI embedding service implementation
- ✅ ChromaDB client wrapper with connection management
- ✅ High-level ChromaMemoryStore implementation
- ✅ Comprehensive unit test suite (10/10 tests passing)
- ⏳ MCP server with tool handlers
- ⏳ CLI testing application
- ⏳ Integration test suites with Docker

**Key Accomplishments**:

- **Robust Error Handling**: Never lose data even when embeddings fail
- **Failed Embedding Recovery**: Automatic tracking and reprocessing system
- **UUID v7 ID Generation**: Time-sortable, unique identifiers
- **Memory Type Conversion**: Seamless domain object ↔ ChromaDB document mapping
- **Semantic Search Logic**: Distance-to-similarity conversion with relevance filtering
- **Comprehensive Testing**: Unit tests covering critical failure scenarios
- **Modern TypeScript Patterns**: Discriminated unions, factory functions, service orchestration

## 🔗 Key Dependencies

- **@modelcontextprotocol/sdk**: MCP protocol implementation
- **chromadb**: Vector database for semantic storage
- **OpenAI API**: Text embedding generation (text-embedding-3-small)
- **uuid**: Time-sortable unique identifier generation (UUID v7)
- **turborepo**: Monorepo build system and task orchestration
- **typescript**: Static type checking and modern language features
- **tsup**: Fast TypeScript bundling for library packages
- **vitest**: Modern testing framework with comprehensive unit tests
- **eslint**: Code linting with modern flat config
- **prettier**: Code formatting

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

## 🚧 Contributing

This is currently a personal learning project. Documentation and clean architecture are prioritized to support future collaboration.

---

_Built with modern TypeScript, powered by semantic search, designed for intelligence._
