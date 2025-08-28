/**
 * Configuration types and constants for Vy vector store
 *
 * Provides environment variable-based configuration with sensible defaults
 * for ChromaDB connection, OpenAI embeddings, and system limits
 */

/**
 * ChromaDB connection configuration for local development
 */
export interface LocalChromaConfig {
  host: string;
  port: number;
}

/**
 * ChromaDB connection configuration for hosted/production instances
 */
export interface HostedChromaConfig {
  host: string;
  port: number;
  apiKey: string;
  ssl: boolean;
}

/**
 * Union type for ChromaDB configurations
 */
export type ChromaConfig = LocalChromaConfig | HostedChromaConfig;

/**
 * OpenAI embedding service configuration
 */
export interface OpenAIEmbeddingConfig {
  apiKey: string;
  model: string;
  dimensions: number;
  maxTokens: number;
  batchSize: number;
}

/**
 * Vector store configuration for local development
 */
export interface LocalVectorStoreConfig {
  chroma: LocalChromaConfig;
  embedding: OpenAIEmbeddingConfig;
  collections: {
    memories: string;
  };
  limits: {
    maxRetries: number;
    timeoutMs: number;
    maxBatchSize: number;
  };
}

/**
 * Vector store configuration for hosted/production instances
 */
export interface HostedVectorStoreConfig {
  chroma: HostedChromaConfig;
  embedding: OpenAIEmbeddingConfig;
  collections: {
    memories: string;
  };
  limits: {
    maxRetries: number;
    timeoutMs: number;
    maxBatchSize: number;
  };
}

/**
 * Union type for vector store configurations
 */
export type VectorStoreConfig =
  | LocalVectorStoreConfig
  | HostedVectorStoreConfig;

/**
 * Default configuration values for local development
 */
export const DEFAULT_LOCAL_CONFIG: Omit<LocalVectorStoreConfig, "embedding"> = {
  chroma: {
    host: "localhost",
    port: 8000,
  },
  collections: {
    memories: "vy_memories",
  },
  limits: {
    maxRetries: 3,
    timeoutMs: 30000, // 30 seconds
    maxBatchSize: 100,
  },
};

/**
 * OpenAI embedding model configurations
 */
export const OPENAI_EMBEDDING_MODELS = {
  "text-embedding-3-small": {
    dimensions: 1536,
    maxTokens: 8192,
    batchSize: 100,
  },
  "text-embedding-3-large": {
    dimensions: 3072,
    maxTokens: 8192,
    batchSize: 100,
  },
  "text-embedding-ada-002": {
    dimensions: 1536,
    maxTokens: 8192,
    batchSize: 100,
  },
} as const;

export type OpenAIEmbeddingModel = keyof typeof OPENAI_EMBEDDING_MODELS;

/**
 * Create configuration for local development
 */
export function createLocalConfig(
  overrides?: Partial<LocalVectorStoreConfig>,
): LocalVectorStoreConfig {
  // OpenAI API key is required
  const openaiApiKey = process.env.VY_OPENAI_API_KEY;
  if (!openaiApiKey) {
    throw new Error("VY_OPENAI_API_KEY environment variable is required");
  }

  // Embedding model with default
  const embeddingModel =
    (process.env.VY_EMBEDDING_MODEL as OpenAIEmbeddingModel) ||
    "text-embedding-3-small";
  const modelConfig = OPENAI_EMBEDDING_MODELS[embeddingModel];

  if (!modelConfig) {
    throw new Error(`Unsupported embedding model: ${embeddingModel}`);
  }

  const baseConfig: LocalVectorStoreConfig = {
    ...DEFAULT_LOCAL_CONFIG,
    chroma: {
      host: process.env.VY_CHROMA_HOST || DEFAULT_LOCAL_CONFIG.chroma.host,
      port: parseInt(
        process.env.VY_CHROMA_PORT || String(DEFAULT_LOCAL_CONFIG.chroma.port),
        10,
      ),
    },
    embedding: {
      apiKey: openaiApiKey,
      model: embeddingModel,
      ...modelConfig,
    },
    collections: {
      memories:
        process.env.VY_COLLECTION_NAME ||
        DEFAULT_LOCAL_CONFIG.collections.memories,
    },
    limits: {
      maxRetries: parseInt(
        process.env.VY_MAX_RETRIES ||
          String(DEFAULT_LOCAL_CONFIG.limits.maxRetries),
        10,
      ),
      timeoutMs: parseInt(
        process.env.VY_TIMEOUT_MS ||
          String(DEFAULT_LOCAL_CONFIG.limits.timeoutMs),
        10,
      ),
      maxBatchSize: parseInt(
        process.env.VY_MAX_BATCH_SIZE ||
          String(DEFAULT_LOCAL_CONFIG.limits.maxBatchSize),
        10,
      ),
    },
  };

  return overrides ? { ...baseConfig, ...overrides } : baseConfig;
}

/**
 * Create configuration for hosted/production environment
 */
export function createHostedConfig(): HostedVectorStoreConfig {
  // All required environment variables
  const openaiApiKey = process.env.VY_OPENAI_API_KEY;
  const chromaHost = process.env.VY_CHROMA_HOST;
  const chromaApiKey = process.env.VY_CHROMA_API_KEY;

  if (!openaiApiKey) {
    throw new Error("VY_OPENAI_API_KEY environment variable is required");
  }
  if (!chromaHost) {
    throw new Error(
      "VY_CHROMA_HOST environment variable is required for hosted config",
    );
  }
  if (!chromaApiKey) {
    throw new Error(
      "VY_CHROMA_API_KEY environment variable is required for hosted config",
    );
  }

  // Embedding model with default
  const embeddingModel =
    (process.env.VY_EMBEDDING_MODEL as OpenAIEmbeddingModel) ||
    "text-embedding-3-small";
  const modelConfig = OPENAI_EMBEDDING_MODELS[embeddingModel];

  if (!modelConfig) {
    throw new Error(`Unsupported embedding model: ${embeddingModel}`);
  }

  return {
    chroma: {
      host: chromaHost,
      port: parseInt(process.env.VY_CHROMA_PORT || "443", 10),
      apiKey: chromaApiKey,
      ssl: process.env.VY_CHROMA_SSL !== "false", // Default to true for hosted
    },
    embedding: {
      apiKey: openaiApiKey,
      model: embeddingModel,
      ...modelConfig,
    },
    collections: {
      memories:
        process.env.VY_COLLECTION_NAME ||
        DEFAULT_LOCAL_CONFIG.collections.memories,
    },
    limits: {
      maxRetries: parseInt(
        process.env.VY_MAX_RETRIES ||
          String(DEFAULT_LOCAL_CONFIG.limits.maxRetries),
        10,
      ),
      timeoutMs: parseInt(
        process.env.VY_TIMEOUT_MS ||
          String(DEFAULT_LOCAL_CONFIG.limits.timeoutMs),
        10,
      ),
      maxBatchSize: parseInt(
        process.env.VY_MAX_BATCH_SIZE ||
          String(DEFAULT_LOCAL_CONFIG.limits.maxBatchSize),
        10,
      ),
    },
  };
}

/**
 * Validate configuration values
 */
export function validateConfig(config: VectorStoreConfig): void {
  if (!config.embedding.apiKey) {
    throw new Error("OpenAI API key is required");
  }

  if (config.chroma.port <= 0 || config.chroma.port > 65535) {
    throw new Error("ChromaDB port must be between 1 and 65535");
  }

  if (config.limits.maxRetries < 0) {
    throw new Error("maxRetries must be non-negative");
  }

  if (config.limits.timeoutMs <= 0) {
    throw new Error("timeoutMs must be positive");
  }

  if (config.limits.maxBatchSize <= 0) {
    throw new Error("maxBatchSize must be positive");
  }

  if (!config.collections.memories.trim()) {
    throw new Error("Collection name cannot be empty");
  }

  // Additional validation for hosted config
  if ("apiKey" in config.chroma && "ssl" in config.chroma) {
    if (!config.chroma.apiKey.trim()) {
      throw new Error("ChromaDB API key cannot be empty for hosted config");
    }
  }
}
