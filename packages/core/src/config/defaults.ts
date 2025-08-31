/**
 * Default configuration values for Vy
 *
 * These defaults provide sensible starting values for all configuration options.
 * They prioritize local development convenience while maintaining production readiness.
 */

import type { ConfigFieldMeta, ConfigSection, VyConfig } from "./types.js";

/**
 * Default configuration values
 */
export const DEFAULT_CONFIG: VyConfig = {
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
    // chromaApiKey is undefined by default
  },
  embedding: {
    openaiApiKey: "", // Required - no default
    model: "text-embedding-3-small",
  },
  logging: {
    level: "info",
    structured: true,
  },
  limits: {
    maxConversationLength: 50000, // ~12,500 tokens
    maxSearchResults: 20,
    maxContextMemories: 10,
  },
};

/**
 * Configuration field metadata for UI generation and validation
 */
export const CONFIG_FIELDS: ConfigFieldMeta[] = [
  // OpenAI Configuration
  {
    path: "embedding.openaiApiKey",
    label: "OpenAI API Key",
    description: "API key for OpenAI embeddings (required)",
    type: "string",
    required: true,
    sensitive: true,
  },
  {
    path: "embedding.model",
    label: "Embedding Model",
    description: "OpenAI embedding model to use",
    type: "select",
    required: false,
    options: [
      "text-embedding-3-small",
      "text-embedding-3-large",
      "text-embedding-ada-002",
    ],
  },

  // ChromaDB Configuration
  {
    path: "vectorStore.chromaHost",
    label: "ChromaDB Host",
    description: "Hostname or IP address of ChromaDB server",
    type: "string",
    required: true,
  },
  {
    path: "vectorStore.chromaPort",
    label: "ChromaDB Port",
    description: "Port number for ChromaDB server",
    type: "number",
    required: true,
    validation: {
      min: 1,
      max: 65535,
    },
  },
  {
    path: "vectorStore.chromaApiKey",
    label: "ChromaDB API Key",
    description: "API key for hosted ChromaDB (optional for local)",
    type: "string",
    required: false,
    sensitive: true,
  },
  {
    path: "vectorStore.chromaSsl",
    label: "Use SSL",
    description: "Enable SSL/TLS for ChromaDB connection",
    type: "boolean",
    required: false,
  },
  {
    path: "vectorStore.collectionName",
    label: "Collection Name",
    description: "Name of the ChromaDB collection for memories",
    type: "string",
    required: true,
  },

  // Server Configuration
  {
    path: "server.name",
    label: "Server Name",
    description: "Identifier for the MCP server instance",
    type: "string",
    required: false,
  },
  {
    path: "logging.level",
    label: "Log Level",
    description: "Minimum log level for server output",
    type: "select",
    required: false,
    options: ["debug", "info", "warn", "error"],
  },

  // Limits Configuration
  {
    path: "limits.maxConversationLength",
    label: "Max Conversation Length",
    description: "Maximum character length for conversations",
    type: "number",
    required: false,
    validation: {
      min: 1000,
      max: 1000000,
    },
  },
  {
    path: "limits.maxSearchResults",
    label: "Max Search Results",
    description: "Maximum number of search results to return",
    type: "number",
    required: false,
    validation: {
      min: 1,
      max: 100,
    },
  },
  {
    path: "limits.maxContextMemories",
    label: "Max Context Memories",
    description: "Maximum memories to include in context injection",
    type: "number",
    required: false,
    validation: {
      min: 1,
      max: 50,
    },
  },
];

/**
 * Configuration sections for organized display and setup
 */
export const CONFIG_SECTIONS: ConfigSection[] = [
  {
    key: "essential",
    label: "Essential Configuration",
    description: "Core settings to get Vy running",
    required: true,
    fields: [
      CONFIG_FIELDS.find((f) => f.path === "embedding.openaiApiKey")!,
      CONFIG_FIELDS.find((f) => f.path === "vectorStore.chromaHost")!,
      CONFIG_FIELDS.find((f) => f.path === "vectorStore.chromaPort")!,
      CONFIG_FIELDS.find((f) => f.path === "vectorStore.collectionName")!,
    ],
  },
  {
    key: "chromadb",
    label: "ChromaDB Advanced Settings",
    description: "Optional ChromaDB connection settings",
    required: false,
    fields: [
      CONFIG_FIELDS.find((f) => f.path === "vectorStore.chromaApiKey")!,
      CONFIG_FIELDS.find((f) => f.path === "vectorStore.chromaSsl")!,
    ],
  },
  {
    key: "openai",
    label: "OpenAI Advanced Settings",
    description: "Optional OpenAI embedding configuration",
    required: false,
    fields: [CONFIG_FIELDS.find((f) => f.path === "embedding.model")!],
  },
  {
    key: "server",
    label: "Server & Logging",
    description: "Server behavior and logging configuration",
    required: false,
    fields: CONFIG_FIELDS.filter(
      (f) => f.path.startsWith("server.") || f.path.startsWith("logging."),
    ),
  },
  {
    key: "limits",
    label: "Performance Limits",
    description: "Resource limits and performance tuning",
    required: false,
    fields: CONFIG_FIELDS.filter((f) => f.path.startsWith("limits.")),
  },
];

/**
 * Environment variable to config path mapping
 */
export const ENV_TO_CONFIG_PATH: Record<string, string> = {
  VY_SERVER_NAME: "server.name",
  VY_SERVER_VERSION: "server.version",
  VY_LOG_LEVEL: "logging.level",
  VY_CHROMA_HOST: "vectorStore.chromaHost",
  VY_CHROMA_PORT: "vectorStore.chromaPort",
  VY_CHROMA_API_KEY: "vectorStore.chromaApiKey",
  VY_CHROMA_SSL: "vectorStore.chromaSsl",
  VY_COLLECTION_NAME: "vectorStore.collectionName",
  VY_OPENAI_API_KEY: "embedding.openaiApiKey",
  VY_EMBEDDING_MODEL: "embedding.model",
  VY_MAX_CONVERSATION_LENGTH: "limits.maxConversationLength",
  VY_MAX_SEARCH_RESULTS: "limits.maxSearchResults",
  VY_MAX_CONTEXT_MEMORIES: "limits.maxContextMemories",
};

/**
 * Get default value for a specific config path
 */
export function getDefaultValue(path: string): unknown {
  const parts = path.split(".");
  let current: unknown = DEFAULT_CONFIG;

  for (const part of parts) {
    if (current && typeof current === "object" && part in current) {
      current = (current as Record<string, unknown>)[part];
    } else {
      return undefined;
    }
  }

  return current;
}

/**
 * Get field metadata by config path
 */
export function getFieldMeta(path: string): ConfigFieldMeta | undefined {
  return CONFIG_FIELDS.find((field) => field.path === path);
}

/**
 * Get all required config paths
 */
export function getRequiredPaths(): string[] {
  return CONFIG_FIELDS.filter((field) => field.required).map(
    (field) => field.path,
  );
}

/**
 * Get all sensitive config paths (for secure handling)
 */
export function getSensitivePaths(): string[] {
  return CONFIG_FIELDS.filter((field) => field.sensitive).map(
    (field) => field.path,
  );
}
