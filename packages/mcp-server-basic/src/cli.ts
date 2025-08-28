#!/usr/bin/env node

// CLI entry point for Vy MCP server
// This will be the executable that starts the MCP server

import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { createVyMcpServer } from './server.js';

async function main() {
  // Create the MCP server instance
  const server = await createVyMcpServer();

  // Create stdio transport for MCP communication
  const transport = new StdioServerTransport();

  // Connect server to transport
  await server.connect(transport);

  console.error('Vy MCP Server started and listening on stdio');
}

// Handle process termination gracefully
process.on('SIGINT', () => {
  console.error('Shutting down Vy MCP Server...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.error('Shutting down Vy MCP Server...');
  process.exit(0);
});

// Start the server
main().catch((error) => {
  console.error('Failed to start Vy MCP Server:', error);
  process.exit(1);
});
