/**
 * Configuration file management utilities for Vy CLI
 *
 * Handles reading, writing, and managing user configuration files with proper
 * security permissions and error handling. Supports the configuration precedence
 * system with local .env files taking priority over user config files.
 */

import {
  DEFAULT_CONFIG,
  ENV_TO_CONFIG_PATH,
  getConfigValue,
  type ConfigFileInfo,
  type PartialVyConfig,
  type VyConfig,
  type VyConfigWithSource,
} from "@repo/core";
import { promises as fs } from "fs";
import { homedir } from "os";
import { dirname, join } from "path";

/**
 * Type guard to check if error is a NodeJS.ErrnoException
 */
function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof (error as NodeJS.ErrnoException).code === "string"
  );
}

/**
 * Configuration file paths
 */
export const CONFIG_PATHS = {
  // User config directory (~/.vy/)
  USER_CONFIG_DIR: join(homedir(), ".vy"),
  USER_CONFIG_FILE: join(homedir(), ".vy", "config.json"),

  // Local development config file
  LOCAL_ENV_FILE: ".env",

  // Backup locations
  XDG_CONFIG_DIR: process.env.XDG_CONFIG_HOME
    ? join(process.env.XDG_CONFIG_HOME, "vy")
    : join(homedir(), ".config", "vy"),
} as const;

/**
 * Configuration file manager
 */
export class ConfigFileManager {
  private userConfigPath: string;
  private userConfigDir: string;

  constructor(customConfigPath?: string) {
    this.userConfigPath = customConfigPath || CONFIG_PATHS.USER_CONFIG_FILE;
    this.userConfigDir = dirname(this.userConfigPath);
  }

  /**
   * Load complete configuration with source tracking
   */
  async loadConfig(): Promise<VyConfigWithSource> {
    // Start with defaults
    let config: VyConfig = JSON.parse(JSON.stringify(DEFAULT_CONFIG));
    const sources: VyConfigWithSource["sources"] = this.createDefaultSources();

    // Load user config file
    const userConfig = await this.loadUserConfig();
    if (userConfig) {
      config = this.mergeConfig(config, userConfig);
      this.updateSources(sources, userConfig, "user-config");
    }

    // Load environment variables (highest precedence)
    const envConfig = this.loadEnvConfig();
    if (envConfig && Object.keys(envConfig).length > 0) {
      config = this.mergeConfig(config, envConfig);
      this.updateSources(sources, envConfig, "env-var");
    }

    return { config, sources };
  }

  /**
   * Save configuration to user config file
   */
  async saveConfig(config: PartialVyConfig): Promise<void> {
    await this.ensureConfigDirectory();

    // Load existing config to merge with new values
    const existingConfig = await this.loadUserConfig();
    const mergedConfig = existingConfig
      ? this.mergeConfig(existingConfig, config)
      : this.mergeConfig({} as PartialVyConfig, config);

    const configJson = JSON.stringify(mergedConfig, null, 2);

    // Write with restricted permissions (600 = owner read/write only)
    await fs.writeFile(this.userConfigPath, configJson, {
      encoding: "utf8",
      mode: 0o600, // Owner read/write only for security
    });
  }

  /**
   * Get configuration value by path
   */
  async getConfigValue(path: string): Promise<unknown> {
    const { config } = await this.loadConfig();
    return getConfigValue(config, path);
  }

  /**
   * Set configuration value by path
   */
  async setConfigValue(path: string, value: unknown): Promise<void> {
    // Load existing user config (not full merged config)
    const existingUserConfig = await this.loadUserConfig();

    // We need to work with the partial config directly since setConfigValue expects a full VyConfig
    // but we only want to save the user overrides
    const updatedUserConfig = { ...existingUserConfig };

    // Set the value in the partial config using dot notation
    const parts = path.split(".");
    let current: Record<string, unknown> = updatedUserConfig as Record<
      string,
      unknown
    >;

    for (let i = 0; i < parts.length - 1; i++) {
      const part = parts[i];
      if (!part) continue;
      if (!current[part] || typeof current[part] !== "object") {
        current[part] = {};
      }
      current = current[part] as Record<string, unknown>;
    }

    const lastPart = parts[parts.length - 1];
    if (lastPart) {
      current[lastPart] = value;
    }

    await this.saveConfig(updatedUserConfig);
  }

  /**
   * Get configuration file information
   */
  async getConfigFileInfo(): Promise<ConfigFileInfo> {
    try {
      const stats = await fs.stat(this.userConfigPath);
      return {
        path: this.userConfigPath,
        exists: true,
        readable: true,
        writable: true,
        permissions: (stats.mode & 0o777).toString(8),
        lastModified: stats.mtime,
        size: stats.size,
      };
    } catch (error: unknown) {
      if (isNodeError(error) && error.code === "ENOENT") {
        return {
          path: this.userConfigPath,
          exists: false,
          readable: false,
          writable: await this.canWriteToConfigDir(),
        };
      }

      return {
        path: this.userConfigPath,
        exists: true,
        readable: false,
        writable: false,
      };
    }
  }

  /**
   * Check if user config file exists
   */
  async configExists(): Promise<boolean> {
    try {
      await fs.access(this.userConfigPath, fs.constants.F_OK);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Initialize configuration directory and file
   */
  async initConfig(force = false): Promise<void> {
    const exists = await this.configExists();

    if (exists && !force) {
      throw new Error(
        `Configuration already exists at ${this.userConfigPath}. Use --force to overwrite.`,
      );
    }

    await this.ensureConfigDirectory();
    await this.saveConfig(DEFAULT_CONFIG);
  }

  /**
   * Remove user configuration file
   */
  async removeConfig(): Promise<void> {
    try {
      await fs.unlink(this.userConfigPath);
    } catch (error: unknown) {
      if (isNodeError(error) && error.code !== "ENOENT") {
        throw error;
      }
      // Ignore if file doesn't exist - ENOENT means file already doesn't exist
    }
  }

  /**
   * Load user configuration file
   */
  private async loadUserConfig(): Promise<PartialVyConfig | null> {
    try {
      const configData = await fs.readFile(this.userConfigPath, "utf8");
      return JSON.parse(configData);
    } catch (error: unknown) {
      if (isNodeError(error) && error.code === "ENOENT") {
        return null; // File doesn't exist
      }
      throw new Error(
        `Failed to load config file: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  /**
   * Load configuration from environment variables
   */
  private loadEnvConfig(): PartialVyConfig {
    const envConfig: Record<string, unknown> = {};

    for (const [envKey, configPath] of Object.entries(ENV_TO_CONFIG_PATH)) {
      const envValue = process.env[envKey];
      if (envValue !== undefined) {
        const parsedValue = this.parseEnvValue(envValue);
        this.setNestedValue(envConfig, configPath, parsedValue);
      }
    }

    return envConfig;
  }

  /**
   * Parse environment variable value to appropriate type
   */
  private parseEnvValue(value: string): unknown {
    // Boolean values
    const lower = value.toLowerCase();
    if (lower === "true" || lower === "false") {
      return lower === "true";
    }

    // Numeric values
    const num = Number(value);
    if (!isNaN(num) && isFinite(num)) {
      return num;
    }

    // String values
    return value;
  }

  /**
   * Set nested value in object using dot notation
   */
  private setNestedValue(
    obj: Record<string, unknown>,
    path: string,
    value: unknown,
  ): void {
    const parts = path.split(".");
    let current: Record<string, unknown> = obj;

    for (let i = 0; i < parts.length - 1; i++) {
      const part = parts[i];
      if (!part) continue;

      if (!current[part] || typeof current[part] !== "object") {
        current[part] = {};
      }
      current = current[part] as Record<string, unknown>;
    }

    const lastPart = parts[parts.length - 1];
    if (lastPart) {
      current[lastPart] = value;
    }
  }

  /**
   * Merge partial configuration into base configuration
   */
  private mergeConfig(
    base: Record<string, unknown>,
    partial: Record<string, unknown>,
  ): Record<string, unknown> {
    const result = { ...base };

    for (const [key, value] of Object.entries(partial)) {
      if (value && typeof value === "object" && !Array.isArray(value)) {
        result[key] = this.mergeConfig(result[key] || {}, value);
      } else {
        result[key] = value;
      }
    }

    return result;
  }

  /**
   * Create default source tracking structure
   */
  private createDefaultSources(): VyConfigWithSource["sources"] {
    const sources: Record<string, Record<string, "default">> = {};

    // Initialize all paths to 'default'
    const initializeSection = (
      section: Record<string, unknown>,
      prefix = "",
    ): void => {
      for (const [key, value] of Object.entries(section)) {
        const path = prefix ? `${prefix}.${key}` : key;
        if (value && typeof value === "object" && !Array.isArray(value)) {
          sources[key] = sources[key] || {};
          initializeSection(value as Record<string, unknown>, path);
        } else {
          if (prefix) {
            const parts = prefix.split(".");
            let current: Record<string, unknown> = sources;
            for (const part of parts) {
              current[part] = current[part] || {};
              current = current[part] as Record<string, unknown>;
            }
            current[key] = "default";
          } else {
            sources[key] = sources[key] || {};
            (sources[key] as Record<string, unknown>)[key] = "default";
          }
        }
      }
    };

    initializeSection(DEFAULT_CONFIG);
    // Create proper nested structure matching VyConfig
    sources.server = {};
    sources.vectorStore = {};
    sources.embedding = {};
    sources.logging = {};
    sources.limits = {};

    // Set all fields to 'default' initially
    Object.keys(DEFAULT_CONFIG.server).forEach((key) => {
      sources.server[key] = "default";
    });
    Object.keys(DEFAULT_CONFIG.vectorStore).forEach((key) => {
      sources.vectorStore[key] = "default";
    });
    Object.keys(DEFAULT_CONFIG.embedding).forEach((key) => {
      sources.embedding[key] = "default";
    });
    Object.keys(DEFAULT_CONFIG.logging).forEach((key) => {
      sources.logging[key] = "default";
    });
    Object.keys(DEFAULT_CONFIG.limits).forEach((key) => {
      sources.limits[key] = "default";
    });

    return sources;
  }

  /**
   * Update source tracking for changed values
   */
  private updateSources(
    sources: VyConfigWithSource["sources"],
    config: Record<string, unknown>,
    source: "user-config" | "env-var",
  ): void {
    const updateSection = (
      sourceSection: Record<string, unknown>,
      configSection: Record<string, unknown>,
    ) => {
      for (const [key, value] of Object.entries(configSection)) {
        if (value !== undefined) {
          if (value && typeof value === "object" && !Array.isArray(value)) {
            if (sourceSection[key]) {
              updateSection(
                sourceSection[key] as Record<string, unknown>,
                value as Record<string, unknown>,
              );
            }
          } else {
            if (sourceSection[key] !== undefined) {
              sourceSection[key] = source;
            }
          }
        }
      }
    };

    updateSection(sources as Record<string, unknown>, config);
  }

  /**
   * Ensure configuration directory exists
   */
  private async ensureConfigDirectory(): Promise<void> {
    try {
      await fs.mkdir(this.userConfigDir, { recursive: true, mode: 0o700 });
    } catch (error: unknown) {
      throw new Error(
        `Failed to create config directory ${this.userConfigDir}: ${(error as Error).message}`,
      );
    }
  }

  /**
   * Check if we can write to config directory
   */
  private async canWriteToConfigDir(): Promise<boolean> {
    try {
      await fs.access(dirname(this.userConfigPath), fs.constants.W_OK);
      return true;
    } catch {
      try {
        // Try to create the directory
        await this.ensureConfigDirectory();
        return true;
      } catch {
        return false;
      }
    }
  }
}

/**
 * Global config file manager instance
 */
export const configManager = new ConfigFileManager();
