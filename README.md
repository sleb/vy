# Vy - AI-Powered Assistant with Memory

Vy is a sophisticated AI assistant built in Rust that remembers your conversations and provides intelligent, personalized responses. Choose between a modern Terminal User Interface (TUI) or classic Command Line Interface (CLI) to chat naturally with AI that gets to know you over time.

## ✨ Key Features

- **🧠 Persistent Memory**: Automatically learns and remembers information about you across conversations
- **🔍 Real-time Search**: Google search integration for current information
- **🖥️ Multiple Interfaces**: Choose between TUI (visual) or CLI (text-based) modes
- **📊 Nutrition Analysis**: Analyze meal photos for ingredient breakdown
- **⚙️ Configurable**: Support for various OpenAI models and customizable settings
- **🛡️ Privacy-First**: All memories stored locally or in your private cloud instance

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

You'll need:

- **OpenAI API key** (required for AI responses)
- **Google API key** (optional, for web search)
- **Google Custom Search Engine ID** (optional, for web search)

### 3. Start Chatting

```bash
# Launch with your preferred interface
vy chat

# Or force a specific mode
vy chat --tui    # Visual terminal interface
vy chat --cli    # Classic text interface
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

### TUI Mode (Recommended)

Modern, visual interface with:

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
vy config set default_chat_mode tui  # or 'cli'
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

| Setting             | Description                 | Default         |
| ------------------- | --------------------------- | --------------- |
| `llm_model_id`      | Main AI model               | `gpt-3.5-turbo` |
| `memory_model_id`   | Model for memory processing | `gpt-4o-mini`   |
| `default_chat_mode` | Interface preference        | `cli`           |

**Recommended Models:**

- `gpt-4o` - Best quality, higher cost
- `gpt-4o-mini` - Great balance of quality and cost ⭐
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

# Verify API key is set
vy config get llm_api_key
```

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

## 📞 Support

- **Issues**: Create a GitHub issue with details
- **Questions**: Include your config (remove sensitive keys): `vy config list`
- **Feature Requests**: Describe your use case and desired outcome

---

**Start having smarter conversations today:**

```bash
vy config init && vy chat
```

Vy learns about you naturally through conversation, making each interaction more helpful than the last! 🧠✨
