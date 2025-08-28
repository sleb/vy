/**
 * Vy Core - Shared types and interfaces for semantic memory system
 *
 * This package provides the foundational types used across all Vy packages
 * including memory types, search interfaces, vector store abstractions,
 * and MCP protocol definitions.
 */

// Memory types and core domain objects
export * from "./types/memory.js";

// Search and query interfaces
export * from "./types/search.js";

// Vector store abstractions
export * from "./types/vector-store.js";

// MCP protocol definitions
export * from "./types/mcp.js";

// Re-export commonly used types for convenience
export type {
  ConversationMemory,
  Memory,
  MemoryId,
  MemoryType,
} from "./types/memory.js";

export type { SearchQuery, SearchResult } from "./types/search.js";

export type { MemoryStore, VectorStore } from "./types/vector-store.js";

export type { McpToolHandler } from "./types/mcp.js";
