/**
 * Server-specific types for Vy MCP Server
 *
 * These types are specific to the MCP server implementation and complement
 * the core types defined in @repo/core
 */

import type { ChromaMemoryStore } from "@repo/vector-store";

/**
 * Server configuration derived from environment variables
 */
export interface ServerConfig {
  // Server identification
  name: string;
  version: string;
  description: string;

  // Vector store configuration
  vectorStore: {
    chromaHost: string;
    chromaPort: number;
    chromaApiKey?: string;
    chromaSsl: boolean;
    collectionName: string;
  };

  // Embedding service configuration
  embedding: {
    openaiApiKey: string;
    model: string;
  };

  // Server behavior
  logging: {
    level: "debug" | "info" | "warn" | "error";
    structured: boolean;
  };

  // Tool limits
  limits: {
    maxConversationLength: number;
    maxSearchResults: number;
    maxContextMemories: number;
  };
}

/**
 * Server runtime state
 */
export interface ServerState {
  isRunning: boolean;
  startTime: Date;
  toolCallCount: number;
  lastError?: Error;
}

/**
 * Memory service dependencies
 */
export interface MemoryServiceDeps {
  store: ChromaMemoryStore;
  config: ServerConfig;
  logger: Logger;
}

/**
 * Structured logger interface
 */
export interface Logger {
  debug(message: string, meta?: Record<string, unknown>): void;
  info(message: string, meta?: Record<string, unknown>): void;
  warn(message: string, meta?: Record<string, unknown>): void;
  error(message: string, error?: Error, meta?: Record<string, unknown>): void;
}

/**
 * Tool execution context
 */
export interface ToolContext {
  toolName: string;
  startTime: Date;
  logger: Logger;
}

/**
 * Server error types
 */
export class VyServerError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = "VyServerError";
  }
}

export class ConfigurationError extends VyServerError {
  constructor(message: string, details?: Record<string, unknown>) {
    super(message, "CONFIGURATION_ERROR", details);
    this.name = "ConfigurationError";
  }
}

export class ToolExecutionError extends VyServerError {
  constructor(
    toolName: string,
    message: string,
    details?: Record<string, unknown>,
  ) {
    super(`Tool '${toolName}' failed: ${message}`, "TOOL_EXECUTION_ERROR", {
      toolName,
      ...details,
    });
    this.name = "ToolExecutionError";
  }
}

/**
 * Environment variable keys
 */
export const ENV_KEYS = {
  // Server
  VY_SERVER_NAME: "VY_SERVER_NAME",
  VY_SERVER_VERSION: "VY_SERVER_VERSION",
  VY_LOG_LEVEL: "VY_LOG_LEVEL",

  // Vector store (reuse from vector-store package)
  VY_CHROMA_HOST: "VY_CHROMA_HOST",
  VY_CHROMA_PORT: "VY_CHROMA_PORT",
  VY_CHROMA_API_KEY: "VY_CHROMA_API_KEY",
  VY_CHROMA_SSL: "VY_CHROMA_SSL",
  VY_COLLECTION_NAME: "VY_COLLECTION_NAME",

  // Embedding service
  VY_OPENAI_API_KEY: "VY_OPENAI_API_KEY",
  VY_EMBEDDING_MODEL: "VY_EMBEDDING_MODEL",

  // Limits
  VY_MAX_CONVERSATION_LENGTH: "VY_MAX_CONVERSATION_LENGTH",
  VY_MAX_SEARCH_RESULTS: "VY_MAX_SEARCH_RESULTS",
  VY_MAX_CONTEXT_MEMORIES: "VY_MAX_CONTEXT_MEMORIES",
} as const;
