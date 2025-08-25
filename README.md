# Vy - AI-Powered Assistant with Memory

> ⚠️ **PRE-ALPHA SOFTWARE** - Vy is in active development with rapid iteration. Expect breaking changes, evolving APIs, and frequent updates. This project currently prioritizes clean, maintainable code over backward compatibility.

Vy is a sophisticated AI assistant built in Rust that remembers your conversations and provides intelligent, personalized responses. Choose between a modern Terminal User Interface (TUI) or classic Command Line Interface (CLI) to chat naturally with AI that gets to know you over time.

## ✨ Key Features

- **🧠 Persistent Memory**: Automatically learns and remembers information about you across conversations
- **🔍 Real-time Search**: Google search integration for current information
- **🖥️ Multiple Interfaces**: Choose between Web (mobile-first), TUI (visual), or CLI (text-based) modes
- **📊 Nutrition Analysis**: Analyze meal photos for ingredient breakdown
- **⚙️ Configurable**: Support for various OpenAI models and customizable settings
- **🛡️ Privacy-First**: All memories stored locally or in your private cloud instance

## 🚧 Development Status

**Pre-Alpha:** Vy is under active development with a focus on:

- **Rapid iteration** and experimentation with new features
- **Clean, maintainable code** over backward compatibility
- **Breaking changes** are common and expected
- **API stability** is not yet a priority

This approach allows for faster development and cleaner architecture. Once core features stabilize, we'll transition to semantic versioning with compatibility guarantees.

## 🚀 Quick Start

### 1. Installation

```bash
git clone <repository-url>
cd vy
cargo build --release
```

The `vy` command will be available at `target/release/vy`.

### 2. Setup

Initialize your configuration (you'll be prompted for API keys):

```bash
vy config init
```

**Required API keys and credentials:**

- **OpenAI API key** (required - for AI responses)
- **Google API key** (required - for web search functionality)
- **Google Custom Search Engine ID** (required - for web search)
- **OpenAI API key for embeddings** (required - can be same as main key)

All API keys are mandatory with no defaults. The system will guide you through obtaining them during setup.

### 3. Start Chatting

```bash
# Launch with your preferred interface
vy chat

# Or force a specific mode
vy chat --tui    # Visual terminal interface
vy chat --cli    # Classic text interface

# For web interface, start the web server:
vy web                    # Spawns vy-web server process on :3001
cd web && npm run dev     # Starts Next.js app on :3000
```

That's it! Vy will start learning about you from your first conversation.

## 💬 Natural Conversations with Memory

Vy remembers information automatically as you chat:

```
💬 You: Hi! I'm Sarah and I work as a software engineer at Google. I love hiking on weekends.

🤖 Vy: Nice to meet you, Sarah! It's great that you're a software engineer at Google.
       Hiking is such a wonderful way to spend weekends! What's your favorite hiking spot?

💬 You: What do you know about my job?

🤖 Vy: 🔍 Searching memories...

You work as a software engineer at Google. Is there something specific about
your work you'd like to discuss?
```

### Memory Commands

**Store Information:**

- "Remember that I'm allergic to peanuts"
- "Keep in mind that I prefer morning meetings"
- "Note that my favorite coffee shop is Blue Bottle"

**Recall Information:**

- "What are my dietary restrictions?"
- "What do you know about my work preferences?"
- "Remind me about my hobbies"

**Update Information:**

- "Actually, I'm not vegetarian anymore - I eat chicken now"
- "Update my work schedule to include Wednesdays remote"

**Remove Information:**

- "Forget about my old job at Microsoft"
- "Remove my outdated phone number"

## 🖥️ Interface Options

### Web Interface (Mobile-First)

Modern web application with:

- **📱 Mobile-optimized**: PWA support, touch-friendly design
- **🔄 Real-time chat**: Instant messaging experience
- **🌙 Modern UI**: Tailwind CSS with dark/light mode
- **📶 Offline ready**: Progressive Web App capabilities
- **🔔 Push notifications**: Stay connected (coming soon)

**To use the web interface:**

```bash
# Terminal 1: Start the Rust API server (spawns vy-web process)
vy web

# Terminal 2: Start the Next.js frontend
cd web && npm run dev

# Visit http://localhost:3000 in your browser
```

### TUI Mode

Visual terminal interface with:

- Full-screen layout with scrollable chat history
- Color-coded messages (green for you, blue for Vy)
- Live status indicators and help system (F1)
- Keyboard navigation (↑↓ for scrolling, Esc to exit)

### CLI Mode

Classic text-based interface:

- Simple line-by-line conversation
- Works on any terminal
- Minimal resource usage
- Perfect for automation or limited terminals

**Set your preferred default:**

```bash
vy config set default_chat_mode web  # 'web', 'tui', or 'cli'
```

## ⚙️ Configuration

### View and Modify Settings

```bash
# List all settings
vy config list

# Get a specific setting
vy config get llm_model_id

# Update a setting
vy config set llm_model_id gpt-4o

# Edit config file directly
vy config --edit
```

### Key Settings

| Setting                         | Description                        | Default                  |
| ------------------------------- | ---------------------------------- | ------------------------ |
| `llm_model_id`                  | Main AI model                      | `gpt-4o-mini`            |
| `memory_model_id`               | Model for memory processing        | `gpt-4o-mini`            |
| `memory_similarity_model_id`    | Model for memory similarity search | `gpt-4o-mini`            |
| `default_chat_mode`             | Interface preference               | `cli`                    |
| `vector_memory_embedding_model` | Embedding model for vector memory  | `text-embedding-3-small` |

**Model Configuration:**

- All models use hard-coded sensible defaults
- Override any model with custom values: `vy config set llm_model_id gpt-4o`
- API keys are mandatory and must be configured during initialization

**Recommended Models:**

- `gpt-4o` - Best quality, higher cost
- `gpt-4o-mini` - Great balance of quality and cost ⭐ (default)
- `gpt-4` - High quality, moderate cost
- `gpt-3.5-turbo` - Fast and economical

## 🧠 Memory Management

### Automatic Memory

Vy automatically identifies and stores:

- Personal preferences and interests
- Important dates and events
- Work and project details
- Family and relationship information
- Goals and aspirations
- Health and dietary information

### Manual Memory Commands

```bash
# View stored memories
vy remember list

# Search for specific memories
vy remember search "work project"

# Add a memory manually
vy remember add "I work at Amazon as a Senior Developer"

# View memory statistics
vy remember stats

# Clear all memories (with confirmation)
vy remember clear --confirm
```

### Memory Tips

**✅ Great for memory:**

- Personal preferences ("I love spicy food")
- Important contacts ("My dentist is Dr. Smith - 555-1234")
- Work information ("I work remotely on Tuesdays")
- Goals ("I want to learn Spanish this year")

**❌ Not ideal for memory:**

- Temporary information ("It's raining today")
- Frequently changing data ("Gas costs $3.50")
- Sensitive data (passwords, personal ID numbers)

## 🔍 Additional Tools

- **Google Search**: Ask about current events, recent news, or real-time information
- **Nutrition Analysis**: Share food photos for ingredient and nutritional breakdown
- **Memory Search**: Semantic search through your stored memories using AI understanding

## 🆘 Troubleshooting

**Vy won't start?**

```bash
# Check your configuration
vy config list

# Initialize configuration if needed
vy config init

# Verify API key is set
vy config get llm_api_key
```

**Missing required configuration?**

- All API keys are mandatory - run `vy config init` to set them up
- Error messages will indicate exactly which keys are missing

**Memory not working?**

- Ensure you have an OpenAI API key configured
- Check internet connection for cloud memory storage
- Try explicit memory commands: "Remember exactly: [fact]"

**TUI display issues?**

- Try resizing your terminal window
- Use `vy chat --cli` as fallback
- Ensure terminal supports ANSI colors (most modern terminals do)

## 📄 Need More Technical Details?

For architecture information, development setup, and contributing guidelines, see [`DEVELOPER.md`](DEVELOPER.md).

## 🚧 Contributing & Development Philosophy

**Pre-Alpha Development Approach:**

- **Breaking changes are welcome** when they improve code quality or user experience
- **No backward compatibility guarantees** during pre-alpha phase
- **Clean code and maintainability** take priority over preserving old interfaces
- **Rapid iteration** is preferred over extensive deprecation periods

This allows us to build the best possible foundation without being constrained by early architectural decisions. We'll transition to semantic versioning and stability guarantees once the core features mature.

## 📞 Support

- **Issues**: Create a GitHub issue with details
- **Questions**: Include your config (remove sensitive keys): `vy config list`
- **Feature Requests**: Describe your use case and desired outcome

---

## 🏗️ Project Structure

```
vy/
├── vy-core/           # Core AI logic and memory
├── vy-cli/            # Command-line interface
├── vy-tui/            # Terminal UI interface
├── vy-web/            # Dedicated web API server
├── vy/                # Main binary (CLI entry point)
└── web/               # Next.js frontend
    ├── app/           # Next.js 13+ app directory
    ├── components/    # React components
    └── public/        # Static assets
```

**Development Setup:**

```bash
# Build all binaries
./scripts/build.sh

# OR build manually:
cargo build --workspace
cd web && npm install

# Start development servers
vy web &                    # Spawns vy-web process on :3001
cd web && npm run dev       # Frontend on :3000

# OR use the dev script:
./scripts/dev-web.sh        # Starts both servers
```

---

**Start having smarter conversations today:**

```bash
# Terminal setup
vy config init && vy chat

# Or web interface
vy web &                    # Spawns web server
cd web && npm run dev       # Start frontend
# Visit http://localhost:3000

# Quick setup with build script:
./scripts/build.sh --install && vy config init
```

Vy learns about you naturally through conversation, making each interaction more helpful than the last! 🧠✨
