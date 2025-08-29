/**
 * Configuration commands for Vy CLI
 *
 * Implements the configuration management commands:
 * - init: Interactive configuration setup
 * - show: Display current configuration with sources
 * - get/set: Individual configuration value management
 * - test: Connection testing and validation
 */

import {
  CONFIG_SECTIONS,
  getSensitivePaths,
  parseConfigValue,
  validateConfig,
  validateField,
  type VyConfig,
} from "@repo/core";
import chalk from "chalk";
import ora from "ora";
import prompts from "prompts";
import { table } from "table";
import { createConnectionTester } from "../../lib/config/connection-tester.js";
import { ConfigFileManager } from "../../lib/config/file-manager.js";

const configManager = new ConfigFileManager();

/**
 * Configuration command implementations
 */
export const configCommands = {
  /**
   * Initialize configuration interactively
   */
  async init(options: { force?: boolean }) {
    const spinner = ora();

    try {
      // Check if config already exists
      const exists = await configManager.configExists();
      if (exists && !options.force) {
        console.log(chalk.yellow("⚠️  Configuration already exists!"));
        console.log(
          chalk.gray(
            `Use ${chalk.white("vy config init --force")} to overwrite.`,
          ),
        );
        return;
      }

      console.log(chalk.cyan("🧠 Welcome to Vy Configuration Setup!\n"));

      // Interactive configuration setup
      const config = await runInteractiveSetup();

      // Save configuration
      spinner.start("Saving configuration...");
      await configManager.saveConfig(config);
      spinner.succeed(
        chalk.green(
          `✅ Configuration saved to ${await configManager.getConfigFileInfo().then((info) => info.path)}`,
        ),
      );

      // Ask if user wants to test configuration
      const { shouldTest } = await prompts({
        type: "confirm",
        name: "shouldTest",
        message: "🧪 Would you like to test the configuration now?",
        initial: true,
      });

      if (shouldTest) {
        console.log(); // Add spacing
        await testConfiguration(config as VyConfig, { verbose: true });
      }

      console.log(
        chalk.green("\n🎉 Setup complete! You can now use Vy commands."),
      );
    } catch (error) {
      spinner.fail("Configuration setup failed");
      console.error(
        chalk.red(
          `Error: ${error instanceof Error ? error.message : String(error)}`,
        ),
      );
      process.exit(1);
    }
  },

  /**
   * Show current configuration
   */
  async show(options: { json?: boolean }) {
    try {
      const { config, sources } = await configManager.loadConfig();

      if (options.json) {
        console.log(JSON.stringify({ config, sources }, null, 2));
        return;
      }

      console.log(chalk.cyan("🔧 Current Vy Configuration\n"));

      const sensitiveKeys = getSensitivePaths();

      // Display each section
      for (const section of CONFIG_SECTIONS) {
        console.log(chalk.bold(`${section.label}:`));

        const sectionData: string[][] = [
          [chalk.gray("Setting"), chalk.gray("Value"), chalk.gray("Source")],
        ];

        for (const field of section.fields) {
          const value = getNestedValue(config, field.path);
          const source = getNestedValue(sources, field.path);

          let displayValue: string;
          if (field.sensitive && value) {
            displayValue = chalk.yellow("****** (hidden)");
          } else if (value === undefined || value === null) {
            displayValue = chalk.gray("(not set)");
          } else {
            displayValue = chalk.white(String(value));
          }

          const sourceColor = getSourceColor(source);
          const displaySource = sourceColor(String(source));

          sectionData.push([field.label, displayValue, displaySource]);
        }

        console.log(
          table(sectionData, {
            border: {
              topBody: "─",
              topJoin: "┬",
              topLeft: "┌",
              topRight: "┐",
              bottomBody: "─",
              bottomJoin: "┴",
              bottomLeft: "└",
              bottomRight: "┘",
              bodyLeft: "│",
              bodyRight: "│",
              bodyJoin: "│",
              joinBody: "─",
              joinLeft: "├",
              joinRight: "┤",
              joinJoin: "┼",
            },
          }),
        );
      }

      // Show config file info
      const fileInfo = await configManager.getConfigFileInfo();
      console.log(chalk.gray("\nConfiguration Files:"));
      console.log(
        chalk.gray(
          `  User config: ${fileInfo.path} ${fileInfo.exists ? "✓" : "✗"}`,
        ),
      );
      console.log(
        chalk.gray(
          `  Local .env: ./.env ${process.env.VY_OPENAI_API_KEY ? "✓" : "✗"}`,
        ),
      );
    } catch (error) {
      console.error(
        chalk.red(
          `Failed to load configuration: ${error instanceof Error ? error.message : String(error)}`,
        ),
      );
      process.exit(1);
    }
  },

  /**
   * Get configuration value
   */
  async get(key: string) {
    try {
      const value = await configManager.getConfigValue(key);

      if (value === undefined) {
        console.log(chalk.yellow(`Configuration key '${key}' not found`));
        return;
      }

      // Check if this is a sensitive key
      const sensitiveKeys = getSensitivePaths();
      if (sensitiveKeys.includes(key)) {
        console.log(chalk.yellow("****** (hidden for security)"));
      } else {
        console.log(String(value));
      }
    } catch (error) {
      console.error(
        chalk.red(
          `Failed to get configuration: ${error instanceof Error ? error.message : String(error)}`,
        ),
      );
      process.exit(1);
    }
  },

  /**
   * Set configuration value
   */
  async set(key: string, value: string) {
    try {
      // Parse the value to appropriate type
      const parsedValue = parseConfigValue(key, value);

      // Validate the field
      const validationResult = validateField(key, parsedValue);
      if (validationResult.length > 0) {
        console.error(chalk.red("Validation failed:"));
        for (const error of validationResult) {
          console.error(chalk.red(`  • ${error.message}`));
        }
        process.exit(1);
      }

      await configManager.setConfigValue(key, parsedValue);

      const sensitiveKeys = getSensitivePaths();
      const displayValue = sensitiveKeys.includes(key)
        ? "****** (hidden)"
        : String(parsedValue);

      console.log(chalk.green(`✅ Set ${key} = ${displayValue}`));
    } catch (error) {
      console.error(
        chalk.red(
          `Failed to set configuration: ${error instanceof Error ? error.message : String(error)}`,
        ),
      );
      process.exit(1);
    }
  },

  /**
   * Test configuration and connections
   */
  async test(options: {
    chromadb?: boolean;
    openai?: boolean;
    verbose?: boolean;
  }) {
    try {
      const { config } = await configManager.loadConfig();

      // Validate configuration first
      const validationResult = validateConfig(config);
      if (!validationResult.isValid) {
        console.log(chalk.red("❌ Configuration validation failed:\n"));
        for (const error of validationResult.errors) {
          console.log(chalk.red(`  • ${error.path}: ${error.message}`));
        }

        if (validationResult.warnings.length > 0) {
          console.log(chalk.yellow("\nWarnings:"));
          for (const warning of validationResult.warnings) {
            console.log(
              chalk.yellow(`  • ${warning.path}: ${warning.message}`),
            );
          }
        }

        process.exit(1);
      }

      if (validationResult.warnings.length > 0) {
        console.log(chalk.yellow("⚠️  Configuration warnings:\n"));
        for (const warning of validationResult.warnings) {
          console.log(chalk.yellow(`  • ${warning.path}: ${warning.message}`));
          if (warning.suggestion) {
            console.log(chalk.gray(`    ${warning.suggestion}`));
          }
        }
        console.log();
      }

      await testConfiguration(config, options);
    } catch (error) {
      console.error(
        chalk.red(
          `Configuration test failed: ${error instanceof Error ? error.message : String(error)}`,
        ),
      );
      process.exit(1);
    }
  },
};

/**
 * Run interactive configuration setup
 */
async function runInteractiveSetup(): Promise<Partial<VyConfig>> {
  const config: any = {};

  for (const section of CONFIG_SECTIONS.filter((s) => s.required)) {
    console.log(chalk.bold(`\n${section.label}`));
    console.log(chalk.gray(section.description));

    for (const field of section.fields) {
      let promptType = "text";
      let choices;

      if (field.type === "boolean") {
        promptType = "confirm";
      } else if (field.type === "number") {
        promptType = "number";
      } else if (field.type === "select" && field.options) {
        promptType = "select";
        choices = field.options.map((option) => ({
          title: option,
          value: option,
        }));
      }

      const response = await prompts({
        type: promptType as any,
        name: "value",
        message: field.required
          ? `${field.label}:`
          : `${field.label} (optional):`,
        initial: getNestedValue(
          {
            embedding: { model: "text-embedding-3-small" },
            vectorStore: {
              chromaHost: "localhost",
              chromaPort: 8000,
              chromaSsl: false,
              collectionName: "vy_memories",
            },
            logging: { level: "info" },
          },
          field.path,
        ),
        choices,
        validate: field.required
          ? (value: any) => {
              if (value === "" || value === null || value === undefined) {
                return "This field is required";
              }
              return true;
            }
          : undefined,
      });

      if (response.value !== undefined && response.value !== "") {
        setNestedValue(config, field.path, response.value);
      }
    }
  }

  // Ask about optional advanced settings
  const { configureAdvanced } = await prompts({
    type: "confirm",
    name: "configureAdvanced",
    message: "🔧 Configure advanced options?",
    initial: false,
  });

  if (configureAdvanced) {
    for (const section of CONFIG_SECTIONS.filter((s) => !s.required)) {
      console.log(chalk.bold(`\n${section.label}`));
      console.log(chalk.gray(section.description));

      for (const field of section.fields) {
        let promptType = "text";
        let choices;

        if (field.type === "boolean") {
          promptType = "confirm";
        } else if (field.type === "number") {
          promptType = "number";
        } else if (field.type === "select" && field.options) {
          promptType = "select";
          choices = field.options.map((option) => ({
            title: option,
            value: option,
          }));
        }

        const response = await prompts({
          type: promptType as any,
          name: "value",
          message: `${field.label} (press enter for default):`,
          initial: getNestedValue(
            {
              server: { name: "vy-mcp-server" },
              logging: { level: "info" },
              limits: {
                maxConversationLength: 50000,
                maxSearchResults: 20,
                maxContextMemories: 10,
              },
            },
            field.path,
          ),
          choices,
        });

        if (response.value !== undefined && response.value !== "") {
          setNestedValue(config, field.path, response.value);
        }
      }
    }
  }

  return config;
}

/**
 * Test configuration and connections
 */
async function testConfiguration(
  config: VyConfig,
  options: { chromadb?: boolean; openai?: boolean; verbose?: boolean },
) {
  const tester = createConnectionTester(config);
  const spinner = ora();

  console.log(chalk.cyan("🧪 Testing Configuration\n"));

  let results;

  if (options.openai) {
    spinner.start("Testing OpenAI API connection...");
    results = [await tester.testService("openai")];
  } else if (options.chromadb) {
    spinner.start("Testing ChromaDB connection...");
    results = [await tester.testService("chromadb")];
  } else {
    spinner.start("Testing all connections...");
    results = await tester.testAll();
  }

  spinner.stop();

  // Display results
  for (const result of results) {
    const icon = result.success ? "✅" : "❌";
    const color = result.success ? chalk.green : chalk.red;

    console.log(color(`${icon} ${result.service}: ${result.message}`));

    if (result.duration) {
      console.log(
        chalk.gray(`   Response time: ${Math.round(result.duration)}ms`),
      );
    }

    if (options.verbose && result.details) {
      console.log(chalk.gray("   Details:"));
      for (const [key, value] of Object.entries(result.details)) {
        console.log(chalk.gray(`     ${key}: ${String(value)}`));
      }
    }

    console.log();
  }

  const allSuccess = results.every((r) => r.success);

  if (allSuccess) {
    console.log(chalk.green("🎉 All tests passed! Configuration is valid."));
  } else {
    console.log(
      chalk.red("❌ Some tests failed. Please check your configuration."),
    );
    process.exit(1);
  }
}

/**
 * Get nested value from object using dot notation
 */
function getNestedValue(obj: any, path: string): any {
  return path.split(".").reduce((current, part) => current?.[part], obj);
}

/**
 * Set nested value in object using dot notation
 */
function setNestedValue(obj: any, path: string, value: any): void {
  const parts = path.split(".");
  let current = obj;

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!current[part] || typeof current[part] !== "object") {
      current[part] = {};
    }
    current = current[part];
  }

  const lastPart = parts[parts.length - 1];
  current[lastPart] = value;
}

/**
 * Get color for configuration source
 */
function getSourceColor(source: string): (text: string) => string {
  switch (source) {
    case "env-var":
      return chalk.cyan;
    case "user-config":
      return chalk.blue;
    case "default":
      return chalk.gray;
    default:
      return chalk.white;
  }
}
