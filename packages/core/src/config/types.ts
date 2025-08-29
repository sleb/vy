/**
 * Shared configuration types for Vy
 *
 * These types define the configuration structure used across CLI and server components.
 * Centralizing these types ensures consistency and enables shared validation logic.
 */

/**
 * Complete Vy configuration structure
 */
export interface VyConfig {
  // Server identification
  server: {
    name: string;
    version: string;
    description: string;
  };

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
    level: LogLevel;
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
 * Partial configuration for updates and user input
 */
export type PartialVyConfig = {
  [K in keyof VyConfig]?: Partial<VyConfig[K]>;
};

/**
 * Configuration with source information (for precedence tracking)
 */
export interface VyConfigWithSource {
  config: VyConfig;
  sources: {
    [K in keyof VyConfig]: {
      [P in keyof VyConfig[K]]: ConfigSource;
    };
  };
}

/**
 * Configuration source for precedence tracking
 */
export type ConfigSource = 'default' | 'user-config' | 'env-var' | 'cli-arg';

/**
 * Log level enumeration
 */
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

/**
 * ChromaDB setup type for interactive configuration
 */
export type ChromaSetupType = 'local' | 'hosted' | 'custom';

/**
 * Configuration validation result
 */
export interface ConfigValidationResult {
  isValid: boolean;
  errors: ConfigValidationError[];
  warnings: ConfigValidationWarning[];
}

/**
 * Configuration validation error
 */
export interface ConfigValidationError {
  path: string;
  message: string;
  value?: unknown;
  code: string;
  details?: Record<string, unknown>;
}

/**
 * Configuration validation warning
 */
export interface ConfigValidationWarning {
  path: string;
  message: string;
  value?: unknown;
  suggestion?: string;
}

/**
 * Environment variable keys for configuration
 */
export const ENV_KEYS = {
  // Server
  VY_SERVER_NAME: 'VY_SERVER_NAME',
  VY_SERVER_VERSION: 'VY_SERVER_VERSION',
  VY_LOG_LEVEL: 'VY_LOG_LEVEL',

  // Vector store
  VY_CHROMA_HOST: 'VY_CHROMA_HOST',
  VY_CHROMA_PORT: 'VY_CHROMA_PORT',
  VY_CHROMA_API_KEY: 'VY_CHROMA_API_KEY',
  VY_CHROMA_SSL: 'VY_CHROMA_SSL',
  VY_COLLECTION_NAME: 'VY_COLLECTION_NAME',

  // Embedding service
  VY_OPENAI_API_KEY: 'VY_OPENAI_API_KEY',
  VY_EMBEDDING_MODEL: 'VY_EMBEDDING_MODEL',

  // Limits
  VY_MAX_CONVERSATION_LENGTH: 'VY_MAX_CONVERSATION_LENGTH',
  VY_MAX_SEARCH_RESULTS: 'VY_MAX_SEARCH_RESULTS',
  VY_MAX_CONTEXT_MEMORIES: 'VY_MAX_CONTEXT_MEMORIES',
} as const;

/**
 * Configuration field metadata for UI generation
 */
export interface ConfigFieldMeta {
  path: string;
  label: string;
  description: string;
  type: 'string' | 'number' | 'boolean' | 'select';
  required: boolean;
  sensitive?: boolean; // For API keys, passwords, etc.
  options?: string[]; // For select types
  validation?: {
    min?: number;
    max?: number;
    pattern?: string;
  };
}

/**
 * Configuration sections for organized display
 */
export interface ConfigSection {
  key: string;
  label: string;
  description: string;
  fields: ConfigFieldMeta[];
  required: boolean; // If any field in section is required
}

/**
 * Connection test result
 */
export interface ConnectionTestResult {
  service: string;
  success: boolean;
  message: string;
  details?: Record<string, unknown>;
  duration?: number;
}

/**
 * Configuration file metadata
 */
export interface ConfigFileInfo {
  path: string;
  exists: boolean;
  readable: boolean;
  writable: boolean;
  permissions?: string;
  lastModified?: Date;
  size?: number;
}
