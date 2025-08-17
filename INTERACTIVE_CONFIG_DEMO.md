# Vy Interactive Configuration Demo

This document demonstrates the new interactive `vy config init` command that guides users through setting up their Vy configuration with all required settings.

## Interactive Flow Example

When a user runs `vy config init`, they'll see:

```
🚀 Welcome to Vy! Let's set up your configuration.
📋 You'll be prompted for API keys and model preferences.
💡 Press Enter to accept default values shown in [brackets], or type a new value.

🔑 OpenAI API Key (required for LLM functionality):
   Get your API key at: https://platform.openai.com/api-keys
Enter your OpenAI API key (input will be hidden): [user enters key]

🤖 Main Model ID [gpt-3.5-turbo]:
   This model will be used for general chat conversations.
   💡 Popular choices: gpt-4o (best), gpt-4 (good), gpt-3.5-turbo (fast & cheap)
Model ID: gpt-4o

🧠 Memory Model ID [gpt-4]:
   This model extracts and processes memories from conversations.
   💡 Recommended: gpt-4 or gpt-4o for better memory extraction
Memory Model ID: [press Enter for default]

🔍 Memory Similarity Model ID [gpt-3.5-turbo]:
   This model finds relevant memories when searching.
   💡 gpt-3.5-turbo is usually sufficient for similarity matching
Memory Similarity Model ID: [press Enter for default]

🌐 Google Search Configuration (required):
   Google search allows Vy to look up current information and recent events.
   Both API key and Search Engine ID are required for Vy to work properly.

🔑 Google API Key:
   Get one at: https://console.developers.google.com/
Enter your Google API key (input will be hidden): [user enters key]

🔍 Google Search Engine ID:
   Create a custom search engine at: https://cse.google.com/
Search Engine ID: [user enters ID]

🎉 Configuration Setup Complete!
═══════════════════════════════════
📁 Config file saved to: /Users/username/.config/vy/prefs.toml
✅ OpenAI API key configured
✅ Google search configured
✅ Models configured:
   • Main chat: gpt-4o
   • Memory extraction: gpt-4
   • Memory similarity: gpt-3.5-turbo

🚀 Next Steps:
   • Start chatting: vy chat
   • Test memory: vy chat (memories are auto-saved after conversations)

📝 Manage Your Configuration:
   • View all settings: vy config list
   • Update a setting: vy config set <key> <value>
   • Edit config file: vy config --edit
```

## Key Features

### 1. **User-Friendly Prompts**

- Clear descriptions for each configuration option
- Helpful links for getting API keys
- Emoji icons for visual clarity
- Default values shown in brackets

### 2. **Smart Input Handling**

- Hidden input for sensitive API keys (uses `rpassword`)
- Model validation with helpful suggestions
- Required field validation
- Confirmation prompts for questionable model choices

### 3. **Comprehensive Feedback**

- Clear success indicators
- Configuration summary
- Next steps guidance
- Management command references

### 4. **Required Configuration**

- All essential fields must be provided
- No partial configurations that would cause failures
- Model validation with override capability
- Existing config file protection (asks before overwriting)

## Configuration Fields Required

1. **llm_api_key** (required, hidden input)
2. **model_id** (with validation and suggestions)
3. **memory_model_id** (with validation and suggestions)
4. **memory_similarity_model_id** (with validation and suggestions)
5. **google_api_key** (required, hidden input)
6. **google_search_engine_id** (required)

All fields are prompted during interactive configuration.

## Error Handling & Validation

- **Required field validation**: Empty required fields prompt retry
- **Model ID validation**: Validates against common models, allows override
- **API key validation**: Ensures keys are not empty
- **Directory creation**: Creates parent directories if needed
- **Config file protection**: Asks before overwriting existing files

## What Happens If Fields Are Empty

The interactive setup will not proceed until all required fields are filled:

- **Empty OpenAI API Key**: Shows error and prompts again
- **Empty Google API Key**: Shows error and prompts again
- **Empty Google Search Engine ID**: Shows error and prompts again
- **Invalid Model ID**: Shows warning but allows override with confirmation

## Testing the Feature

To test the interactive configuration:

```bash
# Remove any existing config
rm ~/.config/vy/prefs.toml

# Run interactive setup
cargo run -- config init

# Verify the configuration
cargo run -- config list
```

## API Key Setup Instructions

### OpenAI API Key

1. Go to https://platform.openai.com/api-keys
2. Sign in to your OpenAI account
3. Click "Create new secret key"
4. Copy the key and paste it during config init

### Google API Key

1. Go to https://console.developers.google.com/
2. Create a new project or select existing one
3. Enable the Custom Search API
4. Create credentials (API key)
5. Copy the key and paste it during config init

### Google Search Engine ID

1. Go to https://cse.google.com/
2. Click "Add" to create a new search engine
3. Configure your search engine settings
4. Copy the Search Engine ID and paste it during config init
