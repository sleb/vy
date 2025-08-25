# Vy Configuration Guide

This document describes Vy's configuration system, including the recent changes to simplify setup and improve security.

## Configuration Philosophy

Vy's configuration follows these principles:

1. **Hard-coded sensible defaults for models** - No need to research which models to use
2. **Mandatory API keys and credentials** - No insecure defaults, explicit requirement for all keys
3. **Easy override capability** - Can customize any setting when needed
4. **Clear error messages** - Helpful guidance when configuration is missing or invalid

## Configuration Structure

### Mandatory Fields (No Defaults)

These fields **must** be provided by the user and have no fallback values:

- `llm_api_key` - OpenAI API key for main AI responses
- `google_api_key` - Google API key for web search functionality
- `google_search_engine_id` - Google Custom Search Engine ID
- `vector_memory.openai_api_key` - OpenAI API key for embeddings (can be same as main key)

### Hard-coded Defaults (Override Available)

These fields have sensible hard-coded defaults but can be customized:

#### Models

- `llm_model_id` - Default: `gpt-4o-mini`
- `memory_model_id` - Default: `gpt-4o-mini`
- `memory_similarity_model_id` - Default: `gpt-4o-mini`

#### Vector Memory

- `vector_memory.qdrant_url` - Default: `http://localhost:6334`
- `vector_memory.collection_name` - Default: `vy_memories`
- `vector_memory.embedding_model` - Default: `text-embedding-3-small`

#### Other Settings

- `default_chat_mode` - Default: `cli`
- `system_prompt` - Default: Comprehensive Vy personality prompt

#### Optional Fields

- `vector_memory.qdrant_api_key` - Only required for Qdrant Cloud usage

## Setup Methods

### Interactive Setup (Recommended)

```bash
vy config init
```

This guides you through:

1. Setting all mandatory API keys with helpful links
2. Optionally overriding model defaults
3. Optionally configuring vector memory settings
4. Automatic validation of configuration

### Manual Configuration

```bash
# Set mandatory fields
vy config set llm_api_key "your-openai-key"
vy config set google_api_key "your-google-key"
vy config set google_search_engine_id "your-engine-id"
vy config set vector_memory_embedding_api_key "your-openai-key"

# Override defaults (optional)
vy config set llm_model_id "gpt-4o"
vy config set memory_model_id "gpt-4"
```

### Environment Variables (Web Deployment)

For web deployments, set these environment variables:

**Mandatory:**

```bash
VY_LLM_API_KEY="your-openai-key"
VY_GOOGLE_API_KEY="your-google-key"
VY_GOOGLE_SEARCH_ENGINE_ID="your-engine-id"
```

**Optional (hard-coded defaults used if not set):**

```bash
VY_LLM_MODEL_ID="gpt-4o"
VY_MEMORY_MODEL_ID="gpt-4o-mini"
VY_MEMORY_SIMILARITY_MODEL_ID="gpt-4o-mini"
VY_VECTOR_MEMORY_OPENAI_API_KEY="your-openai-key"  # Defaults to VY_LLM_API_KEY
VY_QDRANT_URL="http://localhost:6334"
VY_COLLECTION_NAME="vy_memories"
VY_EMBEDDING_MODEL="text-embedding-3-small"
VY_QDRANT_API_KEY="your-qdrant-key"  # Only for cloud
```

## Configuration File Location

**Default locations:**

- Linux: `~/.config/vy/config.toml`
- macOS: `~/Library/Application Support/vy/config.toml`
- Windows: `%APPDATA%\vy\config.toml`

**Override with:**

```bash
vy --config /path/to/custom/config.toml chat
```

## Validation and Error Handling

### Missing Required Configuration

If any mandatory field is missing, Vy will show a clear error:

```
❌ Missing required configuration: google_api_key
💡 Set it with: vy config set google_api_key <your-api-key>
```

### Invalid Model Configuration

Unsupported models will be caught with helpful suggestions:

```
❌ Error: gpt-5 is not currently supported due to tool calling compatibility issues.
💡 Please use one of these supported models instead:
   • gpt-4o
   • gpt-4o-mini
   • gpt-4
   • gpt-3.5-turbo
```

## Model Recommendations

### Supported Models

**OpenAI Models (All supported):**

- `gpt-4o` - Best quality, higher cost
- `gpt-4o-mini` - Great balance, recommended default ⭐
- `gpt-4` - High quality, moderate cost
- `gpt-3.5-turbo` - Fast and economical
- `o1-preview` - Advanced reasoning (limited tool support)
- `o1-mini` - Faster reasoning model

### Model Selection Guidelines

**For main LLM (`llm_model_id`):**

- Production: `gpt-4o-mini` (default) - best cost/performance
- High quality needs: `gpt-4o`
- Budget conscious: `gpt-3.5-turbo`

**For memory processing (`memory_model_id`):**

- Default: `gpt-4o-mini` - optimized for memory analysis
- Can use same as main model for consistency

**For embeddings (`vector_memory.embedding_model`):**

- Default: `text-embedding-3-small` - cost effective
- Alternative: `text-embedding-3-large` - higher quality

## Configuration Commands Reference

```bash
# View all configuration
vy config list

# Get specific value
vy config get llm_model_id

# Set value
vy config set llm_model_id "gpt-4o"

# Interactive setup
vy config init

# All available configuration keys
vy config list --keys
```

## Configuration Keys Reference

### Core Settings

- `llm_api_key` - Main OpenAI API key (mandatory)
- `google_api_key` - Google API key (mandatory)
- `google_search_engine_id` - Google Custom Search Engine ID (mandatory)
- `llm_model_id` - Main AI model
- `memory_model_id` - Memory processing model
- `memory_similarity_model_id` - Memory similarity search model
- `default_chat_mode` - Interface preference (`cli`, `tui`, `web`)
- `system_prompt` - AI personality and instructions

### Vector Memory Settings

- `vector_memory_qdrant_url` - Qdrant database URL
- `vector_memory_qdrant_api_key` - Qdrant API key (optional for local)
- `vector_memory_collection_name` - Qdrant collection name
- `vector_memory_embedding_api_key` - OpenAI API key for embeddings (mandatory)
- `vector_memory_embedding_model` - Embedding model to use

## Security Best Practices

1. **Never commit API keys** to version control
2. **Use environment variables** for deployment
3. **Rotate API keys** regularly
4. **Limit API key permissions** where possible (OpenAI usage limits)
5. **Use separate keys** for different environments (dev/prod)

## Troubleshooting

### Configuration Not Loading

```bash
# Check file exists and is readable
ls -la ~/.config/vy/config.toml

# Validate TOML syntax
vy config list
```

### Environment Variables Not Working

```bash
# Check environment is set
env | grep VY_

# Test with explicit config
VY_LLM_API_KEY="test" vy config list
```

### Permission Issues

```bash
# Fix config directory permissions
chmod 700 ~/.config/vy/
chmod 600 ~/.config/vy/config.toml
```

For additional help, run `vy config --help` or see the main README.
