/**
 * Shared configuration validation logic for Vy
 *
 * This module provides comprehensive validation for Vy configuration,
 * ensuring type safety and business rule compliance across CLI and server components.
 */

import { CONFIG_FIELDS, getFieldMeta } from "./defaults.js";
import type {
  ConfigValidationError,
  ConfigValidationResult,
  ConfigValidationWarning,
  LogLevel,
  VyConfig,
} from "./types.js";

/**
 * Configuration error types
 */
export class ConfigurationError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = "ConfigurationError";
  }
}

/**
 * Validate complete configuration
 */
export function validateConfig(config: VyConfig): ConfigValidationResult {
  const errors: ConfigValidationError[] = [];
  const warnings: ConfigValidationWarning[] = [];

  // Validate individual fields
  for (const field of CONFIG_FIELDS) {
    const value = getConfigValue(config, field.path);
    const fieldErrors = validateField(field.path, value);
    errors.push(...fieldErrors);
  }

  // Cross-field validation
  const crossFieldResults = validateCrossField(config);
  errors.push(...crossFieldResults.errors);
  warnings.push(...crossFieldResults.warnings);

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Validate a single configuration field
 */
export function validateField(
  path: string,
  value: unknown,
): ConfigValidationError[] {
  const errors: ConfigValidationError[] = [];
  const fieldMeta = getFieldMeta(path);

  if (!fieldMeta) {
    errors.push({
      path,
      message: `Unknown configuration field: ${path}`,
      value,
      code: "UNKNOWN_FIELD",
    });
    return errors;
  }

  // Required field validation
  if (
    fieldMeta.required &&
    (value === undefined || value === null || value === "")
  ) {
    errors.push({
      path,
      message: `${fieldMeta.label} is required`,
      value,
      code: "REQUIRED_FIELD",
    });
    return errors; // Don't validate further if required field is missing
  }

  // Skip validation for optional empty values
  if (
    !fieldMeta.required &&
    (value === undefined || value === null || value === "")
  ) {
    return errors;
  }

  // Type validation
  const typeErrors = validateFieldType(path, value, fieldMeta.type);
  errors.push(...typeErrors);

  if (typeErrors.length > 0) {
    return errors; // Don't validate further if type is wrong
  }

  // Field-specific validation
  const specificErrors = validateFieldSpecific(path, value);
  errors.push(...specificErrors);

  // Custom validation rules
  if (fieldMeta.validation) {
    const validationErrors = validateFieldRules(
      path,
      value,
      fieldMeta.validation,
    );
    errors.push(...validationErrors);
  }

  return errors;
}

/**
 * Validate field type
 */
function validateFieldType(
  path: string,
  value: unknown,
  expectedType: string,
): ConfigValidationError[] {
  const errors: ConfigValidationError[] = [];
  const fieldMeta = getFieldMeta(path)!;

  switch (expectedType) {
    case "string":
      if (typeof value !== "string") {
        errors.push({
          path,
          message: `${fieldMeta.label} must be a string`,
          value,
          code: "INVALID_TYPE",
          details: { expectedType: "string", actualType: typeof value },
        });
      }
      break;

    case "number":
      if (typeof value !== "number" || isNaN(value)) {
        errors.push({
          path,
          message: `${fieldMeta.label} must be a valid number`,
          value,
          code: "INVALID_TYPE",
          details: { expectedType: "number", actualType: typeof value },
        });
      }
      break;

    case "boolean":
      if (typeof value !== "boolean") {
        errors.push({
          path,
          message: `${fieldMeta.label} must be a boolean`,
          value,
          code: "INVALID_TYPE",
          details: { expectedType: "boolean", actualType: typeof value },
        });
      }
      break;

    case "select":
      if (typeof value !== "string" || !fieldMeta.options?.includes(value)) {
        errors.push({
          path,
          message: `${fieldMeta.label} must be one of: ${fieldMeta.options?.join(", ")}`,
          value,
          code: "INVALID_OPTION",
          details: { validOptions: fieldMeta.options, actualValue: value },
        });
      }
      break;
  }

  return errors;
}

/**
 * Field-specific validation logic
 */
function validateFieldSpecific(
  path: string,
  value: unknown,
): ConfigValidationError[] {
  const errors: ConfigValidationError[] = [];

  switch (path) {
    case "embedding.openaiApiKey":
      if (typeof value === "string") {
        if (!value.startsWith("sk-")) {
          errors.push({
            path,
            message: 'OpenAI API key must start with "sk-"',
            code: "INVALID_API_KEY_FORMAT",
          });
        }
        if (value.length < 20) {
          errors.push({
            path,
            message: "OpenAI API key appears to be too short",
            code: "INVALID_API_KEY_LENGTH",
          });
        }
      }
      break;

    case "embedding.model":
      if (typeof value === "string") {
        const supportedModels = [
          "text-embedding-3-small",
          "text-embedding-3-large",
          "text-embedding-ada-002",
        ];
        if (!supportedModels.includes(value)) {
          errors.push({
            path,
            message: `Unsupported embedding model: ${value}`,
            code: "UNSUPPORTED_MODEL",
            details: { supportedModels },
          });
        }
      }
      break;

    case "vectorStore.chromaHost":
      if (typeof value === "string") {
        if (value.trim() === "") {
          errors.push({
            path,
            message: "ChromaDB host cannot be empty",
            code: "EMPTY_HOST",
          });
        }
        // Basic hostname validation
        if (value.includes(" ") || value.includes("\n")) {
          errors.push({
            path,
            message: "ChromaDB host contains invalid characters",
            code: "INVALID_HOST_FORMAT",
          });
        }
      }
      break;

    case "vectorStore.chromaPort":
      if (typeof value === "number") {
        if (value < 1 || value > 65535) {
          errors.push({
            path,
            message: "Port must be between 1 and 65535",
            code: "INVALID_PORT_RANGE",
            details: { validRange: "1-65535", actualValue: value },
          });
        }
      }
      break;

    case "vectorStore.collectionName":
      if (typeof value === "string") {
        if (value.trim() === "") {
          errors.push({
            path,
            message: "Collection name cannot be empty",
            code: "EMPTY_COLLECTION_NAME",
          });
        }
        // ChromaDB collection name rules
        if (!/^[a-zA-Z0-9_-]+$/.test(value)) {
          errors.push({
            path,
            message:
              "Collection name can only contain letters, numbers, underscores, and hyphens",
            code: "INVALID_COLLECTION_NAME_FORMAT",
          });
        }
      }
      break;

    case "logging.level":
      // Already validated by select type, but double-check
      if (typeof value === "string") {
        const validLevels: LogLevel[] = ["debug", "info", "warn", "error"];
        if (!validLevels.includes(value as LogLevel)) {
          errors.push({
            path,
            message: `Invalid log level: ${value}`,
            code: "INVALID_LOG_LEVEL",
            details: { validLevels },
          });
        }
      }
      break;
  }

  return errors;
}

/**
 * Validate field rules (min, max, pattern)
 */
function validateFieldRules(
  path: string,
  value: unknown,
  validation: { min?: number; max?: number; pattern?: string },
): ConfigValidationError[] {
  const errors: ConfigValidationError[] = [];
  const fieldMeta = getFieldMeta(path)!;

  // Min/max validation for numbers
  if (typeof value === "number") {
    if (validation.min !== undefined && value < validation.min) {
      errors.push({
        path,
        message: `${fieldMeta.label} must be at least ${validation.min}`,
        value,
        code: "VALUE_TOO_LOW",
        details: { minimum: validation.min, actualValue: value },
      });
    }
    if (validation.max !== undefined && value > validation.max) {
      errors.push({
        path,
        message: `${fieldMeta.label} must be at most ${validation.max}`,
        value,
        code: "VALUE_TOO_HIGH",
        details: { maximum: validation.max, actualValue: value },
      });
    }
  }

  // Pattern validation for strings
  if (typeof value === "string" && validation.pattern) {
    const regex = new RegExp(validation.pattern);
    if (!regex.test(value)) {
      errors.push({
        path,
        message: `${fieldMeta.label} does not match required pattern`,
        value,
        code: "PATTERN_MISMATCH",
        details: { pattern: validation.pattern },
      });
    }
  }

  return errors;
}

/**
 * Cross-field validation (relationships between fields)
 */
function validateCrossField(config: VyConfig): {
  errors: ConfigValidationError[];
  warnings: ConfigValidationWarning[];
} {
  const errors: ConfigValidationError[] = [];
  const warnings: ConfigValidationWarning[] = [];

  // If SSL is enabled, hosted ChromaDB usually requires an API key
  if (config.vectorStore.chromaSsl && !config.vectorStore.chromaApiKey) {
    warnings.push({
      path: "vectorStore.chromaApiKey",
      message:
        "SSL is enabled but no API key provided - this may be required for hosted ChromaDB",
      suggestion: "Consider providing an API key for hosted ChromaDB instances",
    });
  }

  // If using non-localhost host, SSL is usually recommended
  if (
    config.vectorStore.chromaHost !== "localhost" &&
    config.vectorStore.chromaHost !== "127.0.0.1" &&
    !config.vectorStore.chromaSsl
  ) {
    warnings.push({
      path: "vectorStore.chromaSsl",
      message: "Remote ChromaDB connection without SSL may be insecure",
      suggestion: "Consider enabling SSL for remote connections",
    });
  }

  // Performance warnings
  if (config.limits.maxSearchResults > 50) {
    warnings.push({
      path: "limits.maxSearchResults",
      message: "High search result limit may impact performance",
      suggestion: "Consider using a lower limit for better performance",
    });
  }

  if (config.limits.maxContextMemories > 20) {
    warnings.push({
      path: "limits.maxContextMemories",
      message: "High context memory limit may exceed token budgets",
      suggestion: "Consider using a lower limit to stay within context windows",
    });
  }

  return { errors, warnings };
}

/**
 * Parse string value to appropriate type based on field metadata
 */
export function parseConfigValue(path: string, stringValue: string): unknown {
  const fieldMeta = getFieldMeta(path);
  if (!fieldMeta) {
    return stringValue;
  }

  switch (fieldMeta.type) {
    case "number":
      const num = parseInt(stringValue, 10);
      return isNaN(num) ? stringValue : num;

    case "boolean":
      const lower = stringValue.toLowerCase();
      if (lower === "true" || lower === "1" || lower === "yes") return true;
      if (lower === "false" || lower === "0" || lower === "no") return false;
      return stringValue;

    case "string":
    case "select":
    default:
      return stringValue;
  }
}

/**
 * Get configuration value by dot-notation path
 */
export function getConfigValue(config: VyConfig, path: string): unknown {
  const parts = path.split(".");
  let current: any = config;

  for (const part of parts) {
    if (current && typeof current === "object" && part in current) {
      current = current[part];
    } else {
      return undefined;
    }
  }

  return current;
}

/**
 * Set configuration value by dot-notation path
 */
export function setConfigValue(
  config: VyConfig,
  path: string,
  value: unknown,
): VyConfig {
  const parts = path.split(".");
  const result = JSON.parse(JSON.stringify(config)); // Deep clone
  let current: any = result;

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (part && (!current[part] || typeof current[part] !== "object")) {
      current[part] = {};
    }
    if (part) {
      current = current[part];
    }
  }

  const lastPart = parts[parts.length - 1];
  if (lastPart && current) {
    current[lastPart] = value;
  }

  return result;
}

/**
 * Check if a configuration path exists
 */
export function isValidConfigPath(path: string): boolean {
  return CONFIG_FIELDS.some((field) => field.path === path);
}
