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

3. **Start chatting with automatic memory**:

   ```bash
   vy chat
   # Vy will automatically remember important information when you exit
   ```

4. **Explore manual memory features**:
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

Vy's memory system is designed to be simple yet effective, automatically capturing important information from your conversations.

### How Memory Works

**Conversation-End Analysis**: When you finish a chat session (by typing `quit`, `exit`, `bye`, or `q`), Vy automatically analyzes the entire conversation for memorable information such as:

- **Personal Information** - Your name, job, location, relationships
- **Preferences** - Things you like, dislike, or find interesting
- **Life Events** - New jobs, moves, purchases, achievements
- **Goals & Projects** - What you're working on or planning

**During Conversation**: Vy uses the current conversation history for context, so it remembers everything you've said in the current session without any processing overhead.

**Persistent Storage**: Important facts are saved between sessions, so Vy can reference them in future conversations.

### Memory Features

- **Smart Detection**: Automatically identifies memory-worthy information using pattern matching
- **No Interruption**: Memory processing happens only when you exit, keeping conversations fast
- **Manual Control**: Add, remove, or search memories manually when needed
- **Simple Storage**: Facts are stored as plain text with timestamps and conversation context

### Memory Examples

**Automatic Memory (happens when you quit a conversation):**

```bash
$ vy chat
💬 You: Hi, I'm Sarah and I just started working at Microsoft as a data scientist
🤖 Vy: Nice to meet you, Sarah! Congratulations on the new position...
💬 You: quit

🧠 Analyzing conversation for important information...
  📝 Analyzed 1 message(s) from this conversation
  ✅ Stored 2 new memories
  💾 Memories saved for future conversations
```

**Manual Memory Commands:**

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

- **Simple JSON Storage**: Lightweight memory persistence with timestamps
- **Pattern Matching**: Rule-based fact extraction from conversations
- **Agent System**: Built on [rig-core](https://github.com/0xPlaygrounds/rig) for LLM interactions
- **Conversation-End Processing**: Memory analysis only happens when conversations end

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
│   ├── cli/              # Command-line interface
│   ├── tools/            # External tool integrations
│   ├── simple_memory.rs  # Simple memory system
│   ├── lib.rs            # Core library
│   ├── main.rs           # Main entry point
│   └── ...
├── Cargo.toml            # Dependencies and metadata
├── LICENSE               # MIT License
└── README.md             # This file
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

## 📊 Memory System Details

Key highlights:

- **Simplicity**: Lightweight JSON storage with pattern-based extraction
- **Performance**: No processing during conversation, analysis only on exit
- **Automatic**: Detects important personal information without user intervention
- **Manual Control**: Full CLI tools for memory management when needed

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

💬 You: quit
🧠 Analyzing conversation for important information...
  📝 Analyzed 1 message(s) from this conversation
  ✅ Stored 2 new memories
  💾 Memories saved for future conversations
```

### Memory Management

```bash
$ vy remember add "Alice works as a software engineer at Google"
✅ Added memory: Alice works as a software engineer at Google

$ vy remember search "Alice work"
🔍 Found 1 matching memories:
1. [2025-01-02 10:30] User's name is Alice
   Source: conversation_20250102_103015

2. [2025-01-02 10:30] User works as a software engineer at Google
   Source: conversation_20250102_103015
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Acknowledgments

- Built with [rig-core](https://github.com/0xPlaygrounds/rig) for LLM integration
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Powered by [tokio](https://github.com/tokio-rs/tokio) for async runtime
- Simple JSON storage with [serde_json](https://github.com/serde-rs/json)

## 📞 Support

- **Issues**: Report bugs and feature requests on GitHub Issues
- **Discussions**: Join the community discussions for questions and ideas

---

**Made with ❤️ in Rust**
