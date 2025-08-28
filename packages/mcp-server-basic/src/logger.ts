/**
 * Structured logger for Vy MCP Server
 *
 * Provides structured logging to stderr for MCP server debugging and monitoring.
 * Follows MCP best practices for subprocess logging.
 */

import type { Logger } from './types.js';

/**
 * Log levels with numeric values for filtering
 */
const LOG_LEVELS = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3
} as const;

type LogLevel = keyof typeof LOG_LEVELS;

/**
 * Structured log entry
 */
interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  meta?: Record<string, unknown>;
  error?: {
    name: string;
    message: string;
    stack?: string;
  };
}

/**
 * Logger implementation with structured output to stderr
 */
export class StructuredLogger implements Logger {
  private readonly minLevel: number;
  private readonly structured: boolean;

  constructor(
    level: LogLevel = 'info',
    structured: boolean = true
  ) {
    this.minLevel = LOG_LEVELS[level];
    this.structured = structured;
  }

  debug(message: string, meta?: Record<string, unknown>): void {
    this.log('debug', message, meta);
  }

  info(message: string, meta?: Record<string, unknown>): void {
    this.log('info', message, meta);
  }

  warn(message: string, meta?: Record<string, unknown>): void {
    this.log('warn', message, meta);
  }

  error(message: string, error?: Error, meta?: Record<string, unknown>): void {
    const errorMeta = error ? {
      error: {
        name: error.name,
        message: error.message,
        stack: error.stack
      }
    } : {};

    this.log('error', message, { ...meta, ...errorMeta });
  }

  /**
   * Core logging method
   */
  private log(level: LogLevel, message: string, meta?: Record<string, unknown>): void {
    // Filter based on log level
    if (LOG_LEVELS[level] < this.minLevel) {
      return;
    }

    if (this.structured) {
      const entry: LogEntry = {
        timestamp: new Date().toISOString(),
        level,
        message,
        ...(meta && Object.keys(meta).length > 0 && { meta })
      };

      // Output to stderr for MCP servers
      console.error(JSON.stringify(entry));
    } else {
      // Simple format for development
      const timestamp = new Date().toISOString();
      const levelStr = level.toUpperCase().padStart(5);
      const metaStr = meta && Object.keys(meta).length > 0
        ? ` ${JSON.stringify(meta)}`
        : '';

      console.error(`${timestamp} [${levelStr}] ${message}${metaStr}`);
    }
  }
}

/**
 * Create a logger with the specified configuration
 */
export function createLogger(
  level: LogLevel = 'info',
  structured: boolean = true
): Logger {
  return new StructuredLogger(level, structured);
}

/**
 * Create a logger from server configuration
 */
export function createLoggerFromConfig(config: {
  logging: { level: LogLevel; structured: boolean };
}): Logger {
  return createLogger(config.logging.level, config.logging.structured);
}
