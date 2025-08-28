/**
 * Configuration management for Vy MCP Server
 *
 * Handles environment variable parsing, validation, and default values
 * for server configuration. Follows the same patterns as the vector-store
 * package for consistency.
 */

import type { ServerConfig } from "./types.js";
import { ConfigurationError, ENV_KEYS } from "./types.js";

/**
 * Default configuration values
 */
const DEFAULT_CONFIG = {
  server: {
    name: "vy-mcp-server",
    version: "0.0.1",
    description: "Vy semantic memory MCP server",
  },
  vectorStore: {
    chromaHost: "localhost",
    chromaPort: 8000,
    chromaSsl: false,
    collectionName: "vy_memories",
  },
  embedding: {
    model: "text-embedding-3-small",
  },
  logging: {
    level: "info" as const,
    structured: true,
  },
  limits: {
    maxConversationLength: 50000, // ~12,500 tokens
    maxSearchResults: 20,
    maxContextMemories: 10,
  },
};

/**
 * Parse and validate server configuration from environment variables
 */
export function createServerConfig(): ServerConfig {
  try {
    // Required environment variables
    const openaiApiKey = getRequiredEnv(ENV_KEYS.VY_OPENAI_API_KEY);

    // Optional environment variables with defaults
    const config: ServerConfig = {
      name: process.env[ENV_KEYS.VY_SERVER_NAME] || DEFAULT_CONFIG.server.name,
      version:
        process.env[ENV_KEYS.VY_SERVER_VERSION] ||
        DEFAULT_CONFIG.server.version,
      description: DEFAULT_CONFIG.server.description,

      vectorStore: {
        chromaHost:
          process.env[ENV_KEYS.VY_CHROMA_HOST] ||
          DEFAULT_CONFIG.vectorStore.chromaHost,
        chromaPort:
          parsePort(process.env[ENV_KEYS.VY_CHROMA_PORT]) ||
          DEFAULT_CONFIG.vectorStore.chromaPort,
        chromaApiKey: process.env[ENV_KEYS.VY_CHROMA_API_KEY],
        chromaSsl:
          parseBoolean(process.env[ENV_KEYS.VY_CHROMA_SSL]) ||
          DEFAULT_CONFIG.vectorStore.chromaSsl,
        collectionName:
          process.env[ENV_KEYS.VY_COLLECTION_NAME] ||
          DEFAULT_CONFIG.vectorStore.collectionName,
      },

      embedding: {
        openaiApiKey,
        model:
          process.env[ENV_KEYS.VY_EMBEDDING_MODEL] ||
          DEFAULT_CONFIG.embedding.model,
      },

      logging: {
        level:
          parseLogLevel(process.env[ENV_KEYS.VY_LOG_LEVEL]) ||
          DEFAULT_CONFIG.logging.level,
        structured: DEFAULT_CONFIG.logging.structured,
      },

      limits: {
        maxConversationLength:
          parseNumber(process.env[ENV_KEYS.VY_MAX_CONVERSATION_LENGTH]) ||
          DEFAULT_CONFIG.limits.maxConversationLength,
        maxSearchResults:
          parseNumber(process.env[ENV_KEYS.VY_MAX_SEARCH_RESULTS]) ||
          DEFAULT_CONFIG.limits.maxSearchResults,
        maxContextMemories:
          parseNumber(process.env[ENV_KEYS.VY_MAX_CONTEXT_MEMORIES]) ||
          DEFAULT_CONFIG.limits.maxContextMemories,
      },
    };

    // Validate the final configuration
    validateServerConfig(config);

    return config;
  } catch (error) {
    if (error instanceof ConfigurationError) {
      throw error;
    }
    throw new ConfigurationError("Failed to create server configuration", {
      originalError: error instanceof Error ? error.message : String(error),
    });
  }
}

/**
 * Get required environment variable or throw error
 */
function getRequiredEnv(key: string): string {
  const value = process.env[key];
  if (!value) {
    throw new ConfigurationError(
      `Missing required environment variable: ${key}`,
      {
        key,
        availableKeys: Object.keys(process.env).filter((k) =>
          k.startsWith("VY_"),
        ),
      },
    );
  }
  return value;
}

/**
 * Parse port number from string
 */
function parsePort(value: string | undefined): number | undefined {
  if (!value) return undefined;

  const port = parseInt(value, 10);
  if (isNaN(port) || port < 1 || port > 65535) {
    throw new ConfigurationError(`Invalid port number: ${value}`, {
      value,
      validRange: "1-65535",
    });
  }
  return port;
}

/**
 * Parse boolean from string
 */
function parseBoolean(value: string | undefined): boolean | undefined {
  if (!value) return undefined;

  const lower = value.toLowerCase();
  if (lower === "true" || lower === "1" || lower === "yes") return true;
  if (lower === "false" || lower === "0" || lower === "no") return false;

  throw new ConfigurationError(`Invalid boolean value: ${value}`, {
    value,
    validValues: ["true", "false", "1", "0", "yes", "no"],
  });
}

/**
 * Parse log level from string
 */
function parseLogLevel(
  value: string | undefined,
): "debug" | "info" | "warn" | "error" | undefined {
  if (!value) return undefined;

  const lower = value.toLowerCase();
  if (["debug", "info", "warn", "error"].includes(lower)) {
    return lower as "debug" | "info" | "warn" | "error";
  }

  throw new ConfigurationError(`Invalid log level: ${value}`, {
    value,
    validLevels: ["debug", "info", "warn", "error"],
  });
}

/**
 * Parse number from string
 */
function parseNumber(value: string | undefined): number | undefined {
  if (!value) return undefined;

  const num = parseInt(value, 10);
  if (isNaN(num)) {
    throw new ConfigurationError(`Invalid number: ${value}`, { value });
  }
  return num;
}

/**
 * Validate server configuration
 */
function validateServerConfig(config: ServerConfig): void {
  // Validate limits
  if (config.limits.maxConversationLength < 1000) {
    throw new ConfigurationError(
      "maxConversationLength must be at least 1000 characters",
      { current: config.limits.maxConversationLength, minimum: 1000 },
    );
  }

  if (
    config.limits.maxSearchResults < 1 ||
    config.limits.maxSearchResults > 100
  ) {
    throw new ConfigurationError("maxSearchResults must be between 1 and 100", {
      current: config.limits.maxSearchResults,
      validRange: "1-100",
    });
  }

  if (
    config.limits.maxContextMemories < 1 ||
    config.limits.maxContextMemories > 50
  ) {
    throw new ConfigurationError(
      "maxContextMemories must be between 1 and 50",
      { current: config.limits.maxContextMemories, validRange: "1-50" },
    );
  }

  // Validate embedding model
  const supportedModels = [
    "text-embedding-3-small",
    "text-embedding-3-large",
    "text-embedding-ada-002",
  ];
  if (!supportedModels.includes(config.embedding.model)) {
    throw new ConfigurationError(
      `Unsupported embedding model: ${config.embedding.model}`,
      { model: config.embedding.model, supportedModels },
    );
  }

  // Validate vector store configuration
  if (!config.vectorStore.chromaHost) {
    throw new ConfigurationError("ChromaDB host cannot be empty");
  }

  if (
    !config.vectorStore.collectionName ||
    config.vectorStore.collectionName.trim() === ""
  ) {
    throw new ConfigurationError("Collection name cannot be empty");
  }
}

/**
 * Generate configuration summary for logging (without sensitive data)
 */
export function getConfigSummary(
  config: ServerConfig,
): Record<string, unknown> {
  return {
    server: {
      name: config.name,
      version: config.version,
    },
    vectorStore: {
      host: config.vectorStore.chromaHost,
      port: config.vectorStore.chromaPort,
      ssl: config.vectorStore.chromaSsl,
      collection: config.vectorStore.collectionName,
      hasApiKey: !!config.vectorStore.chromaApiKey,
    },
    embedding: {
      model: config.embedding.model,
      hasApiKey: !!config.embedding.openaiApiKey,
    },
    logging: config.logging,
    limits: config.limits,
  };
}
