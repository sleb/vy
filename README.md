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

### Phase 1: MVP Foundation (Current Phase) ✅ In Progress

**Goal**: Basic semantic memory with MCP integration

**Core Features**:

- `capture_conversation` - Store conversations with metadata and embeddings
- `search_memory` - Semantic search across stored memories
- Basic vector storage with ChromaDB
- MCP server implementation

**Deliverables**:

- [ ] Monorepo setup with Turborepo
- [ ] Core types and interfaces (`packages/core`)
- [ ] ChromaDB abstraction layer (`packages/vector-store`)
- [ ] Basic MCP server (`packages/mcp-server-basic`)
- [ ] CLI testing tool (`apps/cli-test`)
- [ ] Basic documentation and setup guides

### Phase 2: Enhanced Memory Intelligence 🔄 Planned

**Goal**: Smarter memory management and retrieval

**Features**:

- Memory condensation and summarization
- Context-aware search ranking
- Memory categorization and tagging
- Temporal memory patterns

### Phase 3: Strategic Intelligence 📋 Planned

**Goal**: Proactive insights and task management

**Features**:

- Auto-extraction of TODOs and action items
- Goal alignment tracking
- Pattern recognition across conversations
- Proactive suggestion system

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
- (Optional) Global Turborepo CLI: `npm install -g turbo`

### Quick Start

```bash
# Clone the repository
git clone <repository-url> vy
cd vy

# Install dependencies
npm install

# Run development mode for all packages
turbo dev

# Build all packages
turbo build

# Run tests
turbo test

# Lint all packages
turbo lint
```

### Package-Specific Commands

```bash
# Work on a specific package
turbo dev --filter=mcp-server-basic
turbo build --filter=vector-store
turbo test --filter=core
```

## 🧠 Core Concepts

### Memory Types

- **Conversations**: Complete conversation threads with context
- **Documents**: Processed documents with extracted insights
- **Insights**: Derived patterns and connections
- **Tasks**: Extracted action items with context

### Semantic Search

- Vector embeddings for content similarity
- Metadata filtering for precise queries
- Temporal relevance weighting
- Context-aware ranking

### MCP Integration

The Model Context Protocol allows Vy to integrate seamlessly with AI assistants, providing:

- Tool definitions for memory operations
- Resource management for stored content
- Prompt integration for context-aware responses

## 📚 Documentation

- [Architecture Decisions](docs/architecture/) - Key technical decisions and rationale
- [API Reference](docs/api/) - Detailed API documentation
- [Development Guides](docs/guides/) - Setup and contribution guides
- [MCP Integration](docs/guides/mcp-integration.md) - How to integrate with AI assistants

## 🤝 Development Philosophy

This project emphasizes:

- **Learning-Oriented Development**: Opportunities for hands-on implementation
- **Modern Best Practices**: Current patterns over legacy compatibility
- **Clean Architecture**: Maintainable, well-documented code
- **Iterative Progress**: Phased development with working increments

## 📈 Current Status

**Phase 1 Progress**:

- ✅ Initial Turborepo setup
- ⏳ Core types and interfaces
- ⏳ ChromaDB abstraction layer
- ⏳ Basic MCP server implementation
- ⏳ CLI testing tool
- ⏳ Documentation framework

## 🔗 Key Dependencies

- **@modelcontextprotocol/sdk**: MCP protocol implementation
- **chromadb**: Vector database for semantic storage
- **@types/node**: TypeScript support for Node.js
- **turborepo**: Monorepo build system
- **typescript**: Static type checking
- **eslint**: Code linting
- **prettier**: Code formatting

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

## 🚧 Contributing

This is currently a personal learning project. Documentation and clean architecture are prioritized to support future collaboration.

---

_Built with modern TypeScript, powered by semantic search, designed for intelligence._
