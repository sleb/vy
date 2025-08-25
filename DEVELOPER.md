# Vy - Developer Documentation

This document provides technical details about Vy's architecture, development setup, and contribution guidelines.

## 🏗️ Architecture Overview

Vy uses a modular workspace architecture with clear separation of concerns:

```
vy/
├── vy-core/           # Core AI logic, memory system, tools
├── vy-cli/            # Command-line interface implementation
├── vy-tui/            # Terminal user interface implementation
├── vy/                # Main binary that coordinates interfaces
└── Cargo.toml         # Workspace configuration
```

### Crate Responsibilities

#### `vy-core` - Core Engine
The interface-agnostic brain of Vy containing:

- **`VyCore` struct**: Main AI orchestration and conversation management
- **Memory system**: Vector-based semantic memory with automatic fact extraction
- **Tools integration**: Google search, nutrition analysis, memory operations
- **Configuration management**: Settings, API keys, model selection
- **Error handling**: Robust error types and recovery mechanisms

#### `vy-cli` - Command Line Interface
Classic text-based interface providing:

- **Chat implementation**: Line-by-line conversation flow
- **Configuration commands**: `vy config init/list/get/set`
- **Memory management**: `vy remember list/search/add/clear`
- **Error presentation**: User-friendly error formatting

#### `vy-tui` - Terminal User Interface
Modern visual interface featuring:

- **Full-screen layout**: Scrollable chat history with status bar
- **Real-time updates**: Live typing indicators and message streaming
- **Keyboard navigation**: F1 help, arrow key scrolling, Esc to exit
- **Color-coded messages**: Visual distinction between user/AI/system messages

#### `vy` - Main Binary
Coordination layer that:

- **Parses command-line arguments**: Routes to appropriate interface
- **Handles interface selection**: CLI vs TUI mode switching
- **Manages shared resources**: Configuration loading, error handling
- **Provides unified CLI**: Single `vy` command for all functionality

## 🔧 Development Setup

### Prerequisites

```bash
# Rust toolchain (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Development dependencies
rustup component add clippy rustfmt
```

### Build & Test

```bash
# Clone and build
git clone <repository-url>
cd vy
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- chat

# Format and lint
cargo fmt
cargo clippy
```

### Project Structure Deep Dive

```
vy-core/src/
├── lib.rs              # VyCore struct, main API surface
├── config.rs           # Configuration management
├── memory.rs           # Memory system implementation
├── error.rs            # Error types and handling
└── tools/
    ├── mod.rs           # Tool registry and trait definitions
    ├── google_search.rs # Real-time web search
    ├── memory_tools.rs  # Memory CRUD operations
    └── nutrition.rs     # Food photo analysis

vy-cli/src/
├── lib.rs              # CLI application structure
├── chat.rs             # Chat loop implementation
├── config.rs           # Configuration subcommands
├── memory.rs           # Memory management commands
└── error.rs            # CLI-specific error formatting

vy-tui/src/
├── lib.rs              # TUI application and event loop
├── ui.rs               # Layout and rendering
├── input.rs            # Input handling and history
└── events.rs           # Keyboard and terminal events

vy/src/
└── main.rs             # Command parsing and interface routing
```

## 🧠 Memory System Architecture

### Vector Memory Pipeline

```
Conversation → Fact Extraction → Embeddings → Vector Storage → Semantic Search
     │              │               │             │               │
   (LLM)         (gpt-4o-mini)   (OpenAI)      (Qdrant)      (Similarity)
```

### Memory Components

**VectorMemory (`vy-core/src/memory.rs`)**
- Manages Qdrant cloud connections
- Handles embedding generation via OpenAI
- Implements semantic search with similarity scoring
- Provides CRUD operations for memory storage

**Memory Tools (`vy-core/src/tools/memory_tools.rs`)**
- `store_memory` - Add new facts
- `search_memory` - Semantic similarity search
- `smart_update_memory` - Update existing memories
- `remove_memories` - Delete memories by query

**Conversation Analysis**
- Automatic fact extraction from chat history
- LLM-powered relevance scoring
- Background processing to avoid blocking chat

### Configuration System

Configuration is stored in `~/.config/vy/config.toml`:

```toml
llm_api_key = "sk-..."
google_api_key = "AIza..."
google_search_engine_id = "..."
llm_model_id = "gpt-4o-mini"
memory_model_id = "gpt-4o-mini"
default_chat_mode = "cli"
```

**Configuration Management (`vy-core/src/config.rs`)**
- Cross-platform config directory detection
- TOML serialization with serde
- Validation and default value handling
- Secure API key storage

## 🛠️ Core APIs

### VyCore - Main Interface

```rust
pub struct VyCore {
    client: Client<OpenAIConfig>,
    config: VyConfig,
    conversation_history: Vec<Message>,
    memory_system: Option<VectorMemory>,
}

impl VyCore {
    // Primary chat interface
    pub async fn send_message(&mut self, input: &str) -> Result<String, VyError>

    // Conversation management
    pub fn conversation_history(&self) -> &[Message]
    pub fn clear_history(&mut self) -> usize

    // Memory operations
    pub async fn analyze_conversation_memories(&self) -> Result<(), VyError>
    pub async fn search_memories(&self, query: &str) -> Result<Vec<Memory>, VyError>
    pub async fn store_memory(&self, fact: &str) -> Result<(), VyError>
}
```

### Tool System

```rust
#[async_trait]
pub trait Tool {
    const NAME: &'static str;
    type Error: std::error::Error + Send + Sync + 'static;
    type Args: for<'de> Deserialize<'de>;
    type Output: Serialize;

    async fn definition(&self, prompt: String) -> ToolDefinition;
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error>;
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum VyError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Memory system error: {0}")]
    Memory(#[from] VectorMemoryError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## 🧪 Testing Strategy

### Unit Tests

```bash
# Core logic tests
cargo test --package vy-core

# Interface tests
cargo test --package vy-cli
cargo test --package vy-tui

# Integration tests
cargo test --package vy
```

### Test Organization

- **Unit tests**: In-module `#[cfg(test)]` blocks
- **Integration tests**: `tests/` directories in each crate
- **Mock objects**: Test doubles for external APIs
- **Fixtures**: Sample configurations and conversations

### Key Test Areas

- Configuration loading and validation
- Memory storage and retrieval accuracy
- Tool schema validation with OpenAI
- Error handling and recovery
- Interface rendering and input handling

## 🔌 Adding New Features

### Adding a New Tool

1. **Create tool implementation** in `vy-core/src/tools/`:

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::tools::{Tool, ToolDefinition};

#[derive(Debug, Deserialize)]
pub struct MyToolArgs {
    pub input: String,
}

#[derive(Debug, Serialize)]
pub struct MyToolResponse {
    pub result: String,
}

pub struct MyTool {
    config: String,
}

#[async_trait]
impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    type Error = MyToolError;
    type Args = MyToolArgs;
    type Output = MyToolResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            function_type: "function".to_string(),
            function: serde_json::json!({
                "name": Self::NAME,
                "description": "Description of what the tool does",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "Input parameter description"
                        }
                    },
                    "required": ["input"]
                }
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Implementation here
        Ok(MyToolResponse {
            result: format!("Processed: {}", args.input)
        })
    }
}
```

2. **Register in builder** (`vy-core/src/lib.rs`):

```rust
pub fn build_openai_vy(config: VyConfig) -> Result<VyCore, VyError> {
    let tools = vec![
        // existing tools...
        Box::new(MyTool::new(config.clone())?) as Box<dyn Tool<Error = VyError>>,
    ];
    // ...
}
```

### Adding a New Interface

1. **Create new crate**: `vy-<interface>/`
2. **Add to workspace**: Update root `Cargo.toml`
3. **Implement interface**: Use `VyCore` for AI operations
4. **Add CLI integration**: Update `vy/src/main.rs`

Example structure:
```rust
// vy-web/src/lib.rs
use vy_core::{VyCore, builder::build_openai_vy};

pub struct WebInterface {
    core: VyCore,
}

impl WebInterface {
    pub async fn handle_message(&mut self, msg: &str) -> Result<String, VyError> {
        self.core.send_message(msg).await
    }
}
```

## 🚀 Performance Considerations

### Memory System
- **Embedding caching**: Avoid re-embedding identical queries
- **Connection pooling**: Reuse Qdrant connections
- **Batch operations**: Group memory operations when possible
- **Background processing**: Async memory analysis doesn't block chat

### Interface Optimization
- **TUI rendering**: Minimal screen updates, efficient layouts
- **CLI streaming**: Real-time response streaming for long outputs
- **Error recovery**: Graceful degradation when services unavailable

## 📦 Release Process

### Version Management

```bash
# Update version in all Cargo.toml files
# Create release tag
git tag v0.1.0
git push origin v0.1.0

# Build release binaries
cargo build --release

# Run full test suite
cargo test --all-features
```

### Distribution

- **Source distribution**: GitHub releases with tarball
- **Binary releases**: Cross-compiled binaries for major platforms
- **Cargo crates**: Publish to crates.io (future)

## 🤝 Contributing

### Code Style

- **Formatting**: `cargo fmt` (rustfmt.toml configured)
- **Linting**: `cargo clippy` with deny warnings
- **Documentation**: Public APIs must have doc comments
- **Testing**: New features require tests

### Pull Request Process

1. **Fork repository** and create feature branch
2. **Implement changes** with tests and documentation
3. **Run full test suite**: `cargo test && cargo clippy`
4. **Submit PR** with detailed description
5. **Address review feedback** and iterate

### Issue Guidelines

- **Bug reports**: Include steps to reproduce, config (sanitized), logs
- **Feature requests**: Describe use case and expected behavior
- **Questions**: Check existing issues and documentation first

## 🔒 Security Considerations

### API Key Management
- Store in OS-specific secure locations
- Never log or display full API keys
- Support environment variable overrides
- Validate key formats before storage

### Memory Privacy
- All memories stored in user's private cloud instance
- No sharing of memory data between users
- Support for local-only memory storage (future)
- Clear data deletion capabilities

### Input Sanitization
- Validate all user inputs before processing
- Escape special characters in tool outputs
- Limit message lengths and memory storage
- Rate limiting for API calls

## 📚 Additional Resources

- **OpenAI API Documentation**: https://platform.openai.com/docs
- **Qdrant Vector Database**: https://qdrant.tech/documentation/
- **Rust Async Programming**: https://rust-lang.github.io/async-book/
- **TUI Development**: https://github.com/ratatui-org/ratatui

---

**Happy coding! 🦀**

This architecture provides a solid foundation for extending Vy with new interfaces, tools, and capabilities while maintaining clean separation of concerns.
