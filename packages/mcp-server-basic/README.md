# Vy MCP Server Basic

A high-performance Model Context Protocol (MCP) server implementation for Vy's semantic memory system.

## Overview

This package provides a production-ready MCP server that enables AI assistants to capture, search, and retrieve contextual memories from conversations. Built with TypeScript and designed for reliability and performance.

## Features

- **Conversation Capture**: Store and analyze conversation data with automatic insight extraction
- **Semantic Search**: Vector-based memory search with relevance scoring
- **Context Retrieval**: Intelligent memory selection for AI context windows
- **Structured Logging**: High-performance logging with Pino
- **Type Safety**: Full TypeScript support with comprehensive type definitions
- **MCP Compliance**: Follows MCP protocol specifications

## Quick Start

### Installation

```bash
npm install @repo/mcp-server-basic
```

### Basic Usage

```typescript
import { VyMcpServer, createServerConfig } from '@repo/mcp-server-basic';

// Create and initialize server
const server = new VyMcpServer();
await server.initialize();

// Connect to transport (typically stdio for MCP)
const transport = /* your MCP transport */;
await server.connect(transport);
```

### CLI Usage

```bash
# Run the MCP server
npx vy-mcp-server

# With custom configuration
VY_LOG_LEVEL=debug VY_CHROMA_URL=http://localhost:8000 npx vy-mcp-server
```

## Architecture

### Core Components

- **VyMcpServer**: Main server class handling MCP protocol
- **MemoryService**: Business logic for memory operations
- **ToolHandlers**: MCP tool implementations
- **Logger**: High-performance structured logging with Pino

### MCP Tools

1. **capture_conversation**
   - Stores conversation data in vector database
   - Extracts insights and action items
   - Returns memory ID for future reference

2. **search_memory**
   - Semantic search across stored memories
   - Configurable relevance scoring
   - Time-based filtering options

3. **get_context**
   - Retrieves relevant memories for current conversation
   - Optimizes for token limits
   - Provides selection reasoning

## Configuration

### Environment Variables

```bash
# Server Configuration
VY_SERVER_NAME="Vy MCP Server"
VY_SERVER_VERSION="1.0.0"
VY_SERVER_DESCRIPTION="Semantic memory server"

# Vector Store
VY_CHROMA_URL="http://localhost:8000"
VY_CHROMA_COLLECTION="vy-memories"

# OpenAI (for embeddings)
VY_OPENAI_API_KEY="your-api-key"
VY_EMBEDDING_MODEL="text-embedding-3-small"

# Logging
VY_LOG_LEVEL="info"  # debug | info | warn | error
VY_LOG_STRUCTURED="true"  # JSON output for production
```

### Programmatic Configuration

```typescript
import { createServerConfig, createLogger } from '@repo/mcp-server-basic';

const config = createServerConfig();
const logger = createLogger('debug', false); // Pretty logging for development
```

## Logging

Uses [Pino](https://github.com/pinojs/pino) for high-performance structured logging:

### Structured Logging (Production)
```json
{"level":30,"time":1640995200000,"msg":"Processing conversation","conversationId":"conv-123","messageCount":5}
```

### Pretty Logging (Development)
```
2024-01-15 10:00:00 [INFO] Processing conversation {"conversationId":"conv-123","messageCount":5}
```

### Log Levels
- `debug`: Detailed execution information
- `info`: General operational messages
- `warn`: Warning conditions
- `error`: Error conditions with stack traces

## Development

### Building

```bash
npm run build
```

### Type Checking

```bash
npm run type-check
```

### Testing

```bash
npm run test
```

### Development Mode

```bash
npm run dev  # Watch mode with auto-rebuild
```

## Dependencies

### Core Dependencies
- `@modelcontextprotocol/sdk`: MCP protocol implementation
- `pino`: High-performance logging
- `@repo/core`: Shared types and utilities
- `@repo/vector-store`: Vector database integration

### Development Dependencies
- `typescript`: Type checking and compilation
- `tsup`: Fast TypeScript bundler
- `vitest`: Testing framework

## Integration

### With Claude Desktop

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "vy-memory": {
      "command": "npx",
      "args": ["@repo/mcp-server-basic"],
      "env": {
        "VY_OPENAI_API_KEY": "your-api-key",
        "VY_CHROMA_URL": "http://localhost:8000"
      }
    }
  }
}
```

### With Other MCP Clients

The server implements standard MCP protocol and works with any compliant client:

```typescript
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { VyMcpServer } from '@repo/mcp-server-basic';

const server = new VyMcpServer();
await server.initialize();

const transport = new StdioServerTransport();
await server.connect(transport);
```

## Error Handling

Comprehensive error handling with structured error responses:

```typescript
// Tool execution errors return structured responses
{
  "success": false,
  "message": "Failed to capture conversation: Invalid format",
  "memoryId": "",
  "extractedInsights": [],
  "actionItems": []
}
```

## Performance

- **High-throughput logging**: Pino for minimal overhead
- **Efficient serialization**: Optimized JSON handling
- **Vector operations**: Delegated to specialized ChromaDB
- **Memory management**: Careful resource cleanup

## Security

- **Input validation**: Comprehensive argument validation
- **Error sanitization**: Safe error message handling
- **Environment isolation**: Configuration via environment variables
- **No credential storage**: API keys handled securely

## Monitoring

### Health Checks

```typescript
const health = server.getHealth();
console.log(health.status); // 'healthy' | 'unhealthy'
```

### Statistics

```typescript
const stats = server.getStats();
console.log(`Uptime: ${stats.uptime}ms, Tool calls: ${stats.toolCalls}`);
```

## Contributing

1. Follow TypeScript best practices
2. Add tests for new functionality
3. Update documentation
4. Ensure proper error handling
5. Use structured logging

## License

MIT License - see LICENSE file for details.

## Related Packages

- `@repo/core`: Shared types and utilities
- `@repo/vector-store`: Vector database abstraction
- `@repo/vy-cli`: Command-line interface

## Support

For issues and questions, please refer to the main Vy project repository.
