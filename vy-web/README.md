# Vy Web Server

The `vy-web` binary provides a standalone web API server for Vy, designed to be stateless and deployable to any environment.

## Architecture

`vy-web` receives all configuration via environment variables, making it:
- **Stateless**: No dependency on config files or filesystem state
- **Portable**: Can run in any directory or container environment
- **Secure**: API keys passed via environment variables (not visible in process lists)
- **Deployable**: Ready for serverless platforms like Cloud Run, Railway, etc.

## Running the Web Server

### Via CLI (Recommended for Development)

```bash
# From your project directory
vy web
```

This command:
1. Loads your project's `vy.toml` configuration
2. Spawns `vy-web` with all necessary environment variables
3. Handles process management and signal forwarding

### Directly (Advanced Usage)

You can run `vy-web` directly, but you must provide all configuration via environment variables:

```bash
# Required
export VY_LLM_API_KEY="sk-..."

# Optional (with defaults)
export VY_GOOGLE_API_KEY="your-google-api-key"
export VY_GOOGLE_SEARCH_ENGINE_ID="your-search-engine-id"
export VY_LLM_MODEL_ID="gpt-4o-mini"
export VY_MEMORY_MODEL_ID="gpt-4o-mini"
export VY_MEMORY_SIMILARITY_MODEL_ID="gpt-4o-mini"
export VY_DEFAULT_CHAT_MODE="cli"
export VY_QDRANT_URL="http://localhost:6333"
export VY_QDRANT_API_KEY="your-qdrant-api-key"  # optional
export VY_COLLECTION_NAME="vy_memories"
export VY_EMBEDDING_MODEL="text-embedding-3-small"

# Server options
export PORT="3001"
export HOST="0.0.0.0"

# Run the server
vy-web
```

### Command Line Options

```bash
vy-web --help
```

- `--port, -p`: Port to bind to (default: 3001, can also use PORT env var)
- `--host`: Host to bind to (default: 0.0.0.0, can also use HOST env var)

## Environment Variables Reference

### Required

| Variable | Description |
|----------|-------------|
| `VY_LLM_API_KEY` | OpenAI API key for language model access |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `VY_GOOGLE_API_KEY` | *(empty)* | Google API key for web search |
| `VY_GOOGLE_SEARCH_ENGINE_ID` | *(empty)* | Google Custom Search Engine ID |
| `VY_LLM_MODEL_ID` | `gpt-4o-mini` | Primary language model |
| `VY_MEMORY_MODEL_ID` | `gpt-4o-mini` | Model for memory processing |
| `VY_MEMORY_SIMILARITY_MODEL_ID` | `gpt-4o-mini` | Model for similarity analysis |
| `VY_DEFAULT_CHAT_MODE` | `cli` | Default chat interface mode |
| `VY_QDRANT_URL` | `http://localhost:6333` | Qdrant vector database URL |
| `VY_QDRANT_API_KEY` | *(none)* | Qdrant API key (for cloud instances) |
| `VY_COLLECTION_NAME` | `vy_memories` | Vector memory collection name |
| `VY_EMBEDDING_MODEL` | `text-embedding-3-small` | Text embedding model |
| `PORT` | `3001` | Server port |
| `HOST` | `0.0.0.0` | Server host/bind address |

## Deployment

### Cloud Run Example

```yaml
# cloudbuild.yaml
steps:
  - name: 'gcr.io/cloud-builders/docker'
    args: ['build', '-t', 'gcr.io/$PROJECT_ID/vy-web', '.']

# Dockerfile
FROM rust:1.70 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin vy-web

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/vy-web /usr/local/bin/
CMD ["vy-web"]
```

Set environment variables in your Cloud Run service configuration.

### Railway Example

1. Connect your GitHub repo to Railway
2. Set the build command: `cargo build --release --bin vy-web`
3. Set the start command: `./target/release/vy-web`
4. Configure environment variables in the Railway dashboard

## API Endpoints

- `GET /health` - Health check endpoint
- `POST /api/chat` - Chat with Vy
  - Request: `{"message": "Hello", "conversation_id": "optional-uuid"}`
  - Response: `{"response": "Hi there!", "conversation_id": "uuid"}`

## Development

The web server includes CORS headers and request logging for development use. For production, configure appropriate security headers and logging levels via environment variables:

```bash
export RUST_LOG=info  # or debug for verbose logging
```
