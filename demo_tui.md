# Vy TUI Demo Guide

This guide demonstrates the new Terminal User Interface (TUI) features in Vy.

## Prerequisites

1. Ensure Vy is built: `cargo build --release`
2. Configure your API keys: `vy config init`
3. Have a terminal that supports ANSI colors and cursor positioning

## TUI vs CLI Comparison

### CLI Mode (Classic)

```bash
# Start classic CLI mode
vy chat

# Simple text-based interface
💬 You: Hello!
🤖 Vy: Hi there! How can I help you today?
💬 You: quit
```

### TUI Mode (New!)

```bash
# Start modern TUI mode (force TUI regardless of config)
vy chat --tui

# Start using configured default mode
vy chat

# Full-screen interface with live updates, colors, and navigation
```

## TUI Features Demo

### 1. Launch TUI Mode

```bash
# Method 1: Force TUI mode (ignores config)
./target/release/vy chat --tui

# Method 2: Set TUI as default and use `vy chat`
./target/release/vy config set default_chat_mode tui
./target/release/vy chat
```

### 2. Interface Tour

When you launch TUI mode, you'll see:

```
┌────────────────────────────────────────────────────────────────────┐
│ 🤖 Vy - gpt-4 | F1: Help | Esc: Exit                             │
├────────────────────────────────────────────────────────────────────┤
│ Chat                                                               │
│ ℹ️  Welcome to Vy TUI - gpt-4! Type your message and press Enter │
│ ℹ️  Press F1 for help, Esc to exit.                            │
│                                                                    │
│                                                                    │
├────────────────────────────────────────────────────────────────────┤
│ Input                                                              │
│ █                                                                  │
├────────────────────────────────────────────────────────────────────┤
│ Messages: 2 | Scroll: ↑↓ | Page: PgUp/PgDn | History: 0 msgs     │
└────────────────────────────────────────────────────────────────────┘
```

### 3. Color-Coded Messages

- **Green**: Your messages (user input)
- **Blue**: Vy's responses
- **Red**: Error messages
- **Yellow**: System messages and thinking indicators
- **Gray**: Status and informational text

### 4. Keyboard Navigation

Try these key combinations:

#### Basic Chat

- Type a message and press `Enter`
- Watch for the yellow "🤖 Thinking..." indicator
- See the response appear in blue

#### Navigation

- `↑` / `↓` arrows: Scroll through message history
- `PgUp` / `PgDn`: Jump by pages through long conversations
- `←` / `→` arrows: Move cursor in input field
- `Backspace`: Delete characters

#### Help System

- Press `F1`: Open comprehensive help screen
- Press `q` or `Esc`: Close help screen

#### Exit

- Press `Esc`: Exit the application
- All conversation history is automatically analyzed for memory

### 5. Live Demo Script

Here's a suggested conversation to showcase TUI features:

```
1a. Launch (force TUI): ./target/release/vy chat --tui
   OR
1b. Set default and launch:
    ./target/release/vy config set default_chat_mode tui
    ./target/release/vy chat

2. Type: "Hi, my name is [Your Name] and I'm testing the new TUI interface!"
   - Watch the thinking indicator
   - See the blue response appear

3. Press F1 to show help
   - Notice the popup overlay
   - Press Esc to close

4. Type a longer message to demonstrate scrolling:
   "Can you tell me about the benefits of using a Terminal User Interface
   over a traditional command-line interface? I'm particularly interested
   in user experience improvements."

5. Use arrow keys to scroll up and down through messages

6. Try page up/down if you have enough messages

7. Press Esc to exit and watch memory analysis
```

### 6. Error Handling Demo

The TUI includes robust error handling:

#### Terminal Compatibility

```bash
# If your terminal doesn't support TUI:
TERM=dumb ./target/release/vy chat --tui
# Will show warning and offer CLI fallback
```

#### API Errors

- Invalid API key: Shows user-friendly error in red
- Network issues: Displays connection status
- Rate limiting: Shows retry information

### 7. Memory Integration

The TUI preserves all existing Vy memory features:

- Conversations are automatically analyzed when you exit
- Previous memories enhance future conversations
- All manual memory commands still work: `vy remember list`

## Configuration Integration

### Setting Default Mode

```bash
# Make TUI the default
./target/release/vy config set default_chat_mode tui

# Make CLI the default
./target/release/vy config set default_chat_mode cli

# Check current setting
./target/release/vy config get default_chat_mode
```

### Override Default Mode

```bash
# Force TUI (regardless of config)
vy chat --tui

# Force CLI (regardless of config)
vy chat --cli

# Use configured default
vy chat
```

## Advanced Features

### Real-time Experience

- No lag between typing and display
- Instant visual feedback
- Live status updates in bottom bar

### Message Management

- Auto-scrolling to new messages
- Preserve scroll position when desired
- Message count tracking

### Visual Feedback

- Cursor position indicator
- Status bar with useful information
- Color-coded message types
- Professional terminal aesthetics

## Troubleshooting

### TUI Won't Start

```bash
# Check terminal support
echo $TERM

# Try with different terminal
# kitty, alacritty, iTerm2, Windows Terminal recommended

# Fall back to CLI mode
vy chat
```

### Display Issues

```bash
# Resize terminal if layout looks wrong
# Most terminals: Cmd+Plus/Minus or Ctrl+Plus/Minus

# Try different color schemes if colors don't show
```

### Performance

```bash
# For very slow terminals, CLI mode may be better
vy chat

# TUI is optimized but requires more terminal features
```

## Comparison Summary

| Feature       | CLI Mode       | TUI Mode               |
| ------------- | -------------- | ---------------------- |
| Interface     | Text-based     | Visual/Interactive     |
| Colors        | Basic          | Full color scheme      |
| Navigation    | Linear         | Scrollable history     |
| Help          | Text commands  | Interactive popup (F1) |
| Status        | None           | Live status bar        |
| Input         | Line-by-line   | Real-time with cursor  |
| Compatibility | Universal      | Modern terminals       |
| Performance   | Minimal        | Optimized for UX       |
| Configuration | Can be default | Can be default         |

## Next Steps

After trying the TUI:

1. **Set your preferred default**: `vy config set default_chat_mode tui` (or `cli`)
2. **Use it for daily conversations**: `vy chat` (uses your default)
3. **Compare both modes**: Try `vy chat --tui` vs `vy chat --cli`
4. **Explore all keyboard shortcuts**: Master the navigation (F1 for help)
5. **Test error scenarios**: See robust error handling
6. **Enjoy the modern interface**: Better conversation flow

The TUI mode makes Vy feel like a modern application while preserving all the powerful AI and memory features you already love! Plus with configurable defaults, you can use whichever interface suits your workflow.
