/**
 * Utility functions for Vy CLI
 *
 * Common utility functions, error handling, configuration validation,
 * and other shared functionality used across CLI commands.
 */

import chalk from "chalk";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

/**
 * Configuration paths
 */
export const CONFIG_PATHS = {
  home: join(homedir(), ".vy"),
  config: join(homedir(), ".vy", "config.json"),
  logs: join(homedir(), ".vy", "logs"),
  cache: join(homedir(), ".vy", "cache"),
};

/**
 * Environment variable keys for Vy configuration
 */
export const ENV_KEYS = {
  VY_OPENAI_API_KEY: "VY_OPENAI_API_KEY",
  VY_CHROMA_HOST: "VY_CHROMA_HOST",
  VY_CHROMA_PORT: "VY_CHROMA_PORT",
  VY_CHROMA_API_KEY: "VY_CHROMA_API_KEY",
  VY_CHROMA_SSL: "VY_CHROMA_SSL",
  VY_COLLECTION_NAME: "VY_COLLECTION_NAME",
  VY_EMBEDDING_MODEL: "VY_EMBEDDING_MODEL",
  VY_LOG_LEVEL: "VY_LOG_LEVEL",
} as const;

/**
 * Required environment variables
 */
export const REQUIRED_ENV_VARS = [ENV_KEYS.VY_OPENAI_API_KEY];

/**
 * Validate that required configuration is present
 */
export async function validateConfig(): Promise<void> {
  const missingVars: string[] = [];

  for (const envVar of REQUIRED_ENV_VARS) {
    if (!process.env[envVar]) {
      missingVars.push(envVar);
    }
  }

  if (missingVars.length > 0) {
    throw new Error(
      `Missing required environment variables: ${missingVars.join(", ")}\n` +
        "Run 'vy config init' to set up your configuration.",
    );
  }

  // Additional validation for ChromaDB connection if using hosted
  if (process.env[ENV_KEYS.VY_CHROMA_API_KEY]) {
    if (!process.env[ENV_KEYS.VY_CHROMA_HOST]) {
      throw new Error("VY_CHROMA_HOST is required when using hosted ChromaDB");
    }
  }
}

/**
 * Check if configuration exists
 */
export function hasConfig(): boolean {
  return (
    existsSync(CONFIG_PATHS.config) ||
    Boolean(process.env[ENV_KEYS.VY_OPENAI_API_KEY])
  );
}

/**
 * Get current configuration summary (without sensitive data)
 */
export function getConfigSummary(): Record<string, unknown> {
  return {
    hasOpenAIKey: Boolean(process.env[ENV_KEYS.VY_OPENAI_API_KEY]),
    chromaHost: process.env[ENV_KEYS.VY_CHROMA_HOST] || "localhost",
    chromaPort: process.env[ENV_KEYS.VY_CHROMA_PORT] || "8000",
    hasChromaKey: Boolean(process.env[ENV_KEYS.VY_CHROMA_API_KEY]),
    chromaSSL: process.env[ENV_KEYS.VY_CHROMA_SSL] === "true",
    collectionName: process.env[ENV_KEYS.VY_COLLECTION_NAME] || "vy_memories",
    embeddingModel:
      process.env[ENV_KEYS.VY_EMBEDDING_MODEL] || "text-embedding-3-small",
    logLevel: process.env[ENV_KEYS.VY_LOG_LEVEL] || "info",
  };
}

/**
 * Handle errors with appropriate formatting and detail level
 */
export function handleError(error: unknown, verbose = false): never {
  if (error instanceof Error) {
    console.error(chalk.red("❌ Error:"), error.message);

    if (verbose && error.stack) {
      console.error(chalk.gray("\nStack trace:"));
      console.error(chalk.gray(error.stack));
    }

    // Provide helpful hints for common errors
    if (error.message.includes("Missing required environment variables")) {
      console.error(
        chalk.yellow(
          "\n💡 Hint: Run 'vy config init' to set up your configuration",
        ),
      );
    } else if (error.message.includes("ChromaDB")) {
      console.error(
        chalk.yellow("\n💡 Hint: Make sure ChromaDB is running and accessible"),
      );
    } else if (error.message.includes("OpenAI")) {
      console.error(
        chalk.yellow("\n💡 Hint: Check your OpenAI API key configuration"),
      );
    } else if (error.message.includes("MCP server")) {
      console.error(
        chalk.yellow(
          "\n💡 Hint: Try 'vy server health' to check server status",
        ),
      );
    }
  } else {
    console.error(chalk.red("❌ Unknown error:"), String(error));
  }

  if (!verbose) {
    console.error(chalk.gray("\n(Use --verbose for more details)"));
  }

  process.exit(1);
}

/**
 * Format duration in milliseconds to human-readable format
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  } else if (ms < 60000) {
    return `${(ms / 1000).toFixed(1)}s`;
  } else {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  }
}

/**
 * Format timestamp to human-readable format
 */
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();

  // Less than 1 minute ago
  if (diffMs < 60000) {
    return "just now";
  }

  // Less than 1 hour ago
  if (diffMs < 3600000) {
    const minutes = Math.floor(diffMs / 60000);
    return `${minutes}m ago`;
  }

  // Less than 24 hours ago
  if (diffMs < 86400000) {
    const hours = Math.floor(diffMs / 3600000);
    return `${hours}h ago`;
  }

  // Less than 7 days ago
  if (diffMs < 604800000) {
    const days = Math.floor(diffMs / 86400000);
    return `${days}d ago`;
  }

  // More than 7 days ago - show actual date
  return date.toLocaleDateString();
}

/**
 * Format file size in bytes to human-readable format
 */
export function formatFileSize(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(1)}${units[unitIndex]}`;
}

/**
 * Truncate text to specified length with ellipsis
 */
export function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) {
    return text;
  }

  return text.substring(0, maxLength - 3) + "...";
}

/**
 * Format memory content for display
 */
export function formatMemory(content: string, maxLength: number = 100): string {
  // Clean up whitespace and newlines
  const cleaned = content.replace(/\s+/g, " ").replace(/\n+/g, " ").trim();

  return truncate(cleaned, maxLength);
}

/**
 * Parse and validate JSON safely
 */
export function safeJsonParse(json: string): unknown {
  try {
    return JSON.parse(json);
  } catch (error) {
    throw new Error(
      `Invalid JSON format: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

/**
 * Convert string to boolean safely
 */
export function parseBoolean(
  value: string | undefined,
  defaultValue = false,
): boolean {
  if (!value) return defaultValue;

  const lower = value.toLowerCase();
  return lower === "true" || lower === "1" || lower === "yes";
}

/**
 * Get environment variable with default value
 */
export function getEnv(key: string, defaultValue = ""): string {
  return process.env[key] || defaultValue;
}

/**
 * Check if running in development mode
 */
export function isDevelopment(): boolean {
  return (
    process.env.NODE_ENV === "development" || process.env.VY_DEV === "true"
  );
}

/**
 * Check if running in verbose mode
 */
export function isVerbose(): boolean {
  return process.argv.includes("--verbose") || process.argv.includes("-v");
}

/**
 * Sleep for specified milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry function with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxAttempts = 3,
  baseDelay = 1000,
): Promise<T> {
  let lastError: Error;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      if (attempt === maxAttempts) {
        throw lastError;
      }

      // Exponential backoff with jitter
      const delay = baseDelay * Math.pow(2, attempt - 1) + Math.random() * 1000;
      await sleep(delay);
    }
  }

  throw lastError!;
}

/**
 * Debounce function
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number,
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout;

  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Create a spinner with consistent styling
 */
interface Spinner {
  start: () => void;
  succeed: (msg?: string) => void;
  fail: (msg?: string) => void;
}

export function createSpinner(text: string): Spinner {
  // This would be implemented with ora, but keeping it simple for now
  return {
    start: () => console.log(chalk.blue(`⏳ ${text}...`)),
    succeed: (msg?: string) => console.log(chalk.green(`✅ ${msg || text}`)),
    fail: (msg?: string) => console.log(chalk.red(`❌ ${msg || text}`)),
  };
}
