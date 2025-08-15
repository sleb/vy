# Vy 🤖

**Your AI Assistant with Memory**

Vy is a command-line AI assistant built in Rust that combines the power of large language models with persistent memory capabilities. Unlike traditional chatbots that forget everything between sessions, Vy can remember facts about you, your preferences, and past conversations.

## ✨ Features

- **💬 Interactive Chat**: Natural conversation with AI models (OpenAI GPT)
- **🧠 Persistent Memory**: Remembers facts, preferences, and relationships across sessions
- **🔍 Smart Search**: Find relevant memories using semantic search
- **🔧 Configuration Management**: Easy setup and customization
- **🌐 Google Search Integration**: Access real-time information (model-dependent)
- **📊 Memory Analytics**: Track and analyze your stored memories
- **🚀 Fast & Efficient**: Built in Rust for performance

## 🚀 Quick Start

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/vy.git
   cd vy
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Install globally** (optional):
   ```bash
   cargo install --path .
   ```

### Initial Setup

1. **Configure your API key**:
   ```bash
   vy config set llm_api_key
   ```
   Enter your OpenAI API key when prompted.

2. **Start chatting**:
   ```bash
   vy chat
   ```

3. **Explore memory features**:
   ```bash
   vy remember add "I love hiking in the mountains"
   vy remember list
   vy remember search "hiking"
   ```

## 📋 Commands

### Chat
```bash
vy chat                    # Start interactive chat session
```

**Chat Commands** (available during conversation):
- `help` - Show available commands
- `history` - Show conversation history
- `clear` - Clear screen and conversation history
- `exit`, `quit`, `bye`, `q` - End conversation

### Configuration
```bash
vy config set <key>        # Set a configuration value
vy config get <key>        # Get a configuration value
vy config list             # List all configuration values
vy config delete <key>     # Delete a configuration value
```

**Available Config Keys**:
- `llm_api_key` - OpenAI API key
- `llm_model` - Model to use (default: gpt-4)
- `google_search_api_key` - Google Custom Search API key
- `google_search_engine_id` - Google Custom Search Engine ID

### Memory Management
```bash
vy remember add <fact>     # Add a fact to memory
vy remember list           # List all stored memories
vy remember search <query> # Search memories
vy remember stats          # Show memory statistics
vy remember delete <index> # Delete a memory by number
vy remember clear --confirm # Clear all memories
vy remember extract <text> # Test fact extraction
```

## 🧠 Memory System

Vy's memory system is one of its key features, allowing it to maintain context across conversations and sessions.

### Memory Types

The system categorizes memories into different types:

- **Facts** - Objective information about entities and events
- **Opinions** - User preferences and subjective views
- **Personal** - Information about the user
- **Relationships** - Connections between people and entities
- **Conversations** - Important conversation excerpts
- **Knowledge** - General insights and learned information

### Memory Features

- **Automatic Extraction**: Vy can automatically extract and store important facts from conversations
- **Semantic Search**: Find relevant memories using natural language queries
- **Confidence Scoring**: Memories have confidence scores that can decay over time
- **Entity Recognition**: Identify and track people, places, and concepts
- **Temporal Queries**: Search memories by time periods

### Memory Commands Examples

```bash
# Add memories manually
vy remember add "I work at Google as a software engineer"
vy remember add "My favorite programming language is Rust"

# Search your memories
vy remember search "work"
vy remember search "programming"

# View memory statistics
vy remember stats

# List all memories with details
vy remember list
```

## 🛠️ Configuration

### Configuration File

Vy stores configuration in `~/.config/vy/prefs.toml` (or equivalent on your platform).

Example configuration:
```toml
llm_api_key = "sk-..."
llm_model = "gpt-4"
google_search_api_key = "your-google-api-key"
google_search_engine_id = "your-search-engine-id"
```

### Custom Configuration Path

You can specify a custom configuration path:
```bash
vy --prefs-path /path/to/custom/prefs.toml chat
```

## 🏗️ Architecture

Vy is built with a modular architecture:

- **CLI Layer** (`src/cli/`) - Command-line interface and argument parsing
- **Memory System** (`src/memory/`) - Persistent memory with semantic search
- **Tools** (`src/tools/`) - External integrations (Google Search, etc.)
- **Core Library** (`src/lib.rs`) - Main chat interface and conversation management

### Key Components

- **SQLite Storage**: Persistent memory storage with indexing
- **Embedding System**: Semantic similarity search using OpenAI embeddings
- **Agent System**: Built on [rig-core](https://github.com/0xPlaygrounds/rig) for LLM interactions
- **Async Runtime**: Built with Tokio for efficient async operations

## 🔧 Development

### Prerequisites

- Rust 1.70+ with 2024 edition support
- SQLite (bundled)
- OpenAI API key for full functionality

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- chat
```

### Project Structure

```
vy/
├── src/
│   ├── cli/           # Command-line interface
│   ├── memory/        # Memory system (see memory/README.md)
│   ├── tools/         # External tool integrations
│   ├── lib.rs         # Core library
│   ├── main.rs        # Main entry point
│   └── ...
├── Cargo.toml         # Dependencies and metadata
├── LICENSE            # MIT License
└── README.md          # This file
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

## 📊 Memory System Details

For detailed information about the memory system, see [`src/memory/README.md`](src/memory/README.md).

Key highlights:
- **Hybrid search**: Combines text and semantic vector search
- **Memory categorization**: Six distinct memory types
- **Performance**: Handles 100k+ memories efficiently
- **Extensible**: Plugin architecture for new memory types

## 🔍 Examples

### Basic Chat Session
```bash
$ vy chat
┌─────────────────────────────────────────────────────────────────┐
│  🤖 Welcome to Vy - Your AI Assistant                           │
│  Model: gpt-4                                                   │
│  ...                                                            │
└─────────────────────────────────────────────────────────────────┘

💬 You: My name is Alice and I love mountain hiking
🤖 Vy (new chat): Nice to meet you, Alice! Mountain hiking sounds wonderful...

💬 You: What do you remember about me?
🤖 Vy (2 msgs): I remember that your name is Alice and you love mountain hiking...
```

### Memory Management
```bash
$ vy remember add "Alice works as a software engineer at Google"
✅ Added memory: Alice works as a software engineer at Google

$ vy remember search "Alice work"
🔍 Found 1 matching memories:
1. [Personal] Alice works as a software engineer at Google
   Entities: Alice, Google
   Added: 2025-01-02 10:30:15
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Acknowledgments

- Built with [rig-core](https://github.com/0xPlaygrounds/rig) for LLM integration
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Powered by [tokio](https://github.com/tokio-rs/tokio) for async runtime
- Memory storage with [rusqlite](https://github.com/rusqlite/rusqlite)

## 📞 Support

- **Issues**: Report bugs and feature requests on GitHub Issues
- **Discussions**: Join the community discussions for questions and ideas
- **Documentation**: Check `src/memory/README.md` for memory system details

---

**Made with ❤️ in Rust**
