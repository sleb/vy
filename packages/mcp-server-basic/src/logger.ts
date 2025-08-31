/**
 * Pino-based logger for Vy MCP Server
 *
 * Provides structured logging to stderr for MCP server debugging and monitoring.
 * Uses Pino for high-performance structured logging with JSON output.
 */

import pino from "pino";
import type { Logger } from "./types.js";

/**
 * Log levels supported by the system
 */
export type LogLevel = "debug" | "info" | "warn" | "error";

/**
 * Logger configuration options
 */
export interface LoggerConfig {
  level: LogLevel;
  structured: boolean;
}

/**
 * Create a Pino logger configured for MCP server use
 */
export function createLogger(
  level: LogLevel = "info",
  structured: boolean = true,
): Logger {
  if (structured) {
    // Structured JSON logging to stderr
    return pino(
      {
        level,
        timestamp: pino.stdTimeFunctions.isoTime,
        formatters: {
          level: (label) => {
            return { level: label };
          },
        },
      },
      process.stderr,
    );
  } else {
    // Pretty printed logging to stderr for development
    return pino({
      level,
      transport: {
        target: "pino-pretty",
        options: {
          destination: 2, // stderr
          colorize: false,
          translateTime: "yyyy-mm-dd HH:MM:ss",
          ignore: "pid,hostname",
        },
      },
    });
  }
}

/**
 * Create a logger from server configuration
 */
export function createLoggerFromConfig(config: {
  logging: LoggerConfig;
}): Logger {
  return createLogger(config.logging.level, config.logging.structured);
}
