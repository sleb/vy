# Vy - AI-Powered Chatbot with Multiple Interfaces

Vy is a sophisticated AI chatbot built in Rust that provides multiple interface options including CLI, TUI, and (soon) mobile and web interfaces. It features persistent memory, real-time Google search, and advanced conversation capabilities.

## 🏗️ Architecture

This project uses a modular workspace architecture with separate crates for different concerns:

```
vy/
├── vy-core/           # Core chatbot logic and functionality
├── vy-cli/            # Command-line interface
├── vy-tui/            # Terminal user interface (basic implementation)
├── vy/                # Main binary that ties everything together
└── Cargo.toml         # Workspace configuration
```

### Crate Responsibilities

- **`vy-core`**: Contains the core `VyCore` struct, memory system, tools (Google search, nutrition analysis), and configuration management. This crate is interface-agnostic and can be used by any frontend.

- **`vy-cli`**: Provides the command-line interface, configuration management commands, and memory management utilities.

- **`vy-tui`**: Terminal User Interface (currently minimal - shows placeholder message).

- **`vy`**: Main binary crate that coordinates between different interfaces and provides the unified `vy` command.

## 🚀 Features

- **Multiple Interface Options**: CLI and TUI modes (with more coming)
- **Persistent Memory**: Automatically learns and remembers information from conversations
- **Real-time Search**: Google search integration for current information
- **Nutrition Analysis**: Analyze meal photos for ingredient breakdown
- **Configurable Models**: Support for various OpenAI models
- **Error Handling**: User-friendly error messages and recovery

## 📦 Installation

### From Source

```bash
git clone <repository-url>
cd vy
cargo build --release
```

The binary will be available at `target/release/vy`.

## 🔧 Setup

1. **Initialize Configuration**:

   ```bash
   vy config init
   ```

   This will prompt you for:
   - OpenAI API key
   - Google API key
   - Google Custom Search Engine ID
   - Model preferences

2. **Verify Configuration**:
   ```bash
   vy config list
   ```

## 💬 Usage

### Chat Modes

**CLI Mode (Default)**:

```bash
vy chat
# or explicitly
vy chat --cli
```

**TUI Mode**:

```bash
vy chat --tui
```

### Configuration Management

```bash
# List all settings
vy config list

# Get a specific setting
vy config get llm_model_id

# Set a setting
vy config set llm_model_id gpt-4o-mini

# Edit config file directly
vy config --edit
```

### Memory Management

```bash
# List stored memories
vy remember list

# Search memories
vy remember search "work project"

# Add a memory manually
vy remember add "I work at Amazon as a Senior Developer"

# View memory statistics
vy remember stats

# Clear all memories (with confirmation)
vy remember clear --confirm
```

## ⚙️ Configuration

Configuration is stored in `~/.config/vy/config.toml` (or your system's config directory).

### Key Settings

| Setting                   | Description                 | Default         |
| ------------------------- | --------------------------- | --------------- |
| `llm_api_key`             | OpenAI API key              | (required)      |
| `google_api_key`          | Google API key for search   | (required)      |
| `google_search_engine_id` | Custom search engine ID     | (required)      |
| `llm_model_id`            | Main chat model             | `gpt-3.5-turbo` |
| `memory_model_id`         | Model for memory processing | `gpt-4o-mini`   |
| `default_chat_mode`       | Default interface mode      | `cli`           |

### Supported Models

- `gpt-4o` (recommended for best quality)
- `gpt-4o-mini` (good balance of cost/quality)
- `gpt-4`
- `gpt-3.5-turbo`

## 🧠 Memory System

Vy automatically analyzes conversations and extracts important information to remember for future interactions. Memories are stored locally and used to provide personalized responses.

### Memory Features

- **Automatic Learning**: Extracts facts from natural conversation
- **Smart Search**: Finds relevant memories based on context
- **LLM-Enhanced**: Uses AI to determine what's worth remembering
- **Local Storage**: All memories stored on your device

## 🔍 Tools & Capabilities

- **Google Search**: Real-time web search for current information
- **Memory System**: Personal information storage and retrieval
- **Nutrition Analysis**: Analyze food photos for nutritional content
- **Error Recovery**: Graceful handling of API errors and rate limits

## 🛠️ Development

### Project Structure

The codebase is organized as a Cargo workspace with the following structure:

```
vy/
├── vy-core/src/
│   ├── lib.rs           # Core VyCore struct and main logic
│   ├── config.rs        # Configuration management
│   ├── memory.rs        # Memory system implementation
│   └── tools/           # Various tools (search, memory, etc.)
├── vy-cli/src/
│   ├── lib.rs           # CLI application logic
│   ├── chat.rs          # Chat interface implementation
│   ├── config.rs        # Configuration commands
│   └── memory.rs        # Memory management commands
├── vy-tui/src/
│   └── lib.rs           # TUI implementation (minimal)
└── vy/src/
    └── main.rs          # Main binary entry point
```

### Adding New Interfaces

To add a new interface (e.g., web, mobile):

1. Create a new crate (e.g., `vy-web`)
2. Add `vy-core` as a dependency
3. Use `vy_core::builder::build_openai_vy()` to create a VyCore instance
4. Implement your interface around the VyCore methods
5. Add the new crate to the workspace

### Key APIs

The `VyCore` struct provides these main methods:

```rust
// Send a message and get response
async fn send_message(&mut self, input: &str) -> Result<String>

// Access conversation history
fn conversation_history(&self) -> &[Message]

// Clear conversation history
fn clear_history(&mut self) -> usize

// Analyze conversation for memories
async fn analyze_conversation_memories(&self) -> Result<()>
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Test specific crate
cargo test --package vy-core
```

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 🚧 Roadmap

- [ ] Complete TUI implementation with full feature parity
- [ ] Web interface (vy-web)
- [ ] Native mobile apps (vy-mobile)
- [ ] Plugin system for custom tools
- [ ] Multi-provider support (Anthropic, etc.)
- [ ] Voice interface support
- [ ] Conversation export/import
- [ ] Advanced memory management UI

## 📞 Support

For issues, feature requests, or questions:

1. Check existing issues on GitHub
2. Create a new issue with detailed description
3. Include configuration details (with sensitive data removed)

---

**Note**: This refactored version provides a solid foundation for adding mobile interfaces while maintaining clean separation of concerns between the core functionality and different user interfaces.
