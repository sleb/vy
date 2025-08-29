/**
 * Configuration module for Vy
 *
 * This module provides shared configuration types, validation, and utilities
 * that can be used across CLI and server components.
 */

export type {
    ChromaSetupType, ConfigFieldMeta, ConfigFileInfo, ConfigSection, ConfigSource, ConfigValidationError, ConfigValidationResult, ConfigValidationWarning, ConnectionTestResult, LogLevel, PartialVyConfig, VyConfig, VyConfigWithSource
} from './types.js';

export {
    ENV_KEYS
} from './types.js';

export {
    CONFIG_FIELDS,
    CONFIG_SECTIONS, DEFAULT_CONFIG, ENV_TO_CONFIG_PATH,
    getDefaultValue,
    getFieldMeta,
    getRequiredPaths,
    getSensitivePaths
} from './defaults.js';

export {
    ConfigurationError, getConfigValue, isValidConfigPath, parseConfigValue, setConfigValue, validateConfig,
    validateField
} from './validation.js';

