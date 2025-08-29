#!/usr/bin/env node

/**
 * Vy - AI-Powered Semantic Memory CLI
 *
 * A command-line interface for interacting with your semantic memory system.
 * Store conversations, search memories, and retrieve relevant context for AI interactions.
 */

import chalk from "chalk";
import { Command } from "commander";
import { configCommands } from "./commands/config/index.js";
import { devCommands } from "./commands/dev/index.js";
import { memCommands } from "./commands/mem/index.js";
import { serverCommands } from "./commands/server/index.js";

const program = new Command();

// Main CLI configuration
program
  .name("vy")
  .description("🧠 Vy - AI-powered semantic memory system")
  .version("0.0.1")
  .option("-v, --verbose", "enable verbose output")
  .option("--no-color", "disable colored output")
  .option("--config <path>", "path to config file");

// Memory operations - the main user interface
const mem = program
  .command("mem")
  .description(
    "📚 Memory operations - capture, search, and retrieve your thoughts",
  );

mem
  .command("capture")
  .description("💭 Capture a conversation or thought in semantic memory")
  .argument("[text]", "text to capture (if not provided, will prompt)")
  .option("-p, --participants <names...>", "conversation participants")
  .option("-t, --tags <tags...>", "tags to associate with this memory")
  .option("-m, --metadata <json>", "additional metadata as JSON string")
  .option("-s, --summary <text>", "optional summary of the content")
  .option("--from-file <path>", "read content from file")
  .action(memCommands.capture);

mem
  .command("search")
  .description("🔍 Search your semantic memories")
  .argument("<query>", "search query")
  .option("-l, --limit <number>", "maximum number of results", "10")
  .option("-t, --types <types...>", "filter by memory types")
  .option("--since <date>", "only show memories since date (YYYY-MM-DD)")
  .option("--until <date>", "only show memories until date (YYYY-MM-DD)")
  .option("--min-score <score>", "minimum relevance score (0-1)", "0.1")
  .option("--json", "output results as JSON")
  .action(memCommands.search);

mem
  .command("context")
  .alias("ctx")
  .description("🎯 Get relevant context for current situation")
  .option("-q, --query <text>", "current context or query")
  .option("-r, --recent <messages...>", "recent conversation messages")
  .option("-n, --max-memories <number>", "maximum memories to return", "10")
  .option("-T, --max-tokens <number>", "token budget for context", "2000")
  .option("--json", "output as JSON")
  .action(memCommands.context);

mem
  .command("list")
  .alias("ls")
  .description("📋 List stored memories")
  .option("-l, --limit <number>", "number of memories to show", "20")
  .option("-t, --type <type>", "filter by memory type")
  .option("--since <date>", "memories since date (YYYY-MM-DD)")
  .option("--sort <field>", "sort by field (date, relevance)", "date")
  .option("--json", "output as JSON")
  .action(memCommands.list);

mem
  .command("delete")
  .alias("rm")
  .description("🗑️  Delete memories")
  .argument("<id>", "memory ID to delete")
  .option("-f, --force", "skip confirmation prompt")
  .action(memCommands.delete);

// Server management
const server = program
  .command("server")
  .description("🖥️  MCP server management");

server
  .command("start")
  .description("🚀 Start the MCP server")
  .option("-d, --daemon", "run as daemon")
  .option("-p, --port <port>", "server port (for debugging)")
  .option("--log-level <level>", "log level (debug, info, warn, error)", "info")
  .action(serverCommands.start);

server
  .command("stop")
  .description("🛑 Stop the MCP server")
  .action(serverCommands.stop);

server
  .command("status")
  .description("📊 Check server status")
  .option("--json", "output as JSON")
  .action(serverCommands.status);

server
  .command("health")
  .description("🏥 Perform health check")
  .option("--timeout <ms>", "health check timeout", "5000")
  .option("--json", "output as JSON")
  .action(serverCommands.health);

server
  .command("logs")
  .description("📜 View server logs")
  .option("-f, --follow", "follow log output")
  .option("-n, --lines <number>", "number of lines to show", "50")
  .option("--since <time>", "show logs since timestamp")
  .action(serverCommands.logs);

// Configuration management
const config = program
  .command("config")
  .description("⚙️  Configuration management");

config
  .command("show")
  .description("📋 Show current configuration")
  .option("--json", "output as JSON")
  .action(configCommands.show);

config
  .command("init")
  .description("🎯 Initialize configuration")
  .option("--force", "overwrite existing config")
  .action(configCommands.init);

config
  .command("test")
  .description("🧪 Test configuration and connections")
  .option("--chromadb", "test ChromaDB connection")
  .option("--openai", "test OpenAI API connection")
  .action(configCommands.test);

config
  .command("set")
  .description("✏️  Set configuration value")
  .argument("<key>", "configuration key (dot notation supported)")
  .argument("<value>", "configuration value")
  .action(configCommands.set);

config
  .command("get")
  .description("📖 Get configuration value")
  .argument("<key>", "configuration key (dot notation supported)")
  .action(configCommands.get);

// Development tools
const dev = program
  .command("dev")
  .description("🔧 Development tools and utilities");

dev
  .command("mock-data")
  .description("🎭 Generate mock data for testing")
  .option("-t, --type <type>", "data type (conversation, search, context)")
  .option("-c, --count <count>", "number of items to generate", "5")
  .option("-o, --output <file>", "output to file")
  .action(devCommands.mockData);

dev
  .command("benchmark")
  .description("⚡ Run performance benchmarks")
  .option("-n, --iterations <number>", "number of iterations", "100")
  .option("--tool <tool>", "specific tool to benchmark")
  .action(devCommands.benchmark);

dev
  .command("debug")
  .description("🐛 Debug server and connections")
  .option("--server", "debug server startup")
  .option("--chromadb", "debug ChromaDB connection")
  .option("--embeddings", "debug embedding generation")
  .action(devCommands.debug);

// Global error handling
process.on("uncaughtException", (error) => {
  console.error(chalk.red("\n💥 Uncaught Exception:"));
  console.error(chalk.red(error.message));

  const verbose = program.opts().verbose;
  if (verbose && error.stack) {
    console.error(chalk.gray("\nStack trace:"));
    console.error(chalk.gray(error.stack));
  } else {
    console.error(chalk.gray("\n(Use --verbose for stack trace)"));
  }

  process.exit(1);
});

process.on("unhandledRejection", (reason, promise) => {
  console.error(chalk.red("\n💥 Unhandled Promise Rejection:"));
  console.error(chalk.red(String(reason)));

  const verbose = program.opts().verbose;
  if (verbose) {
    console.error(chalk.gray("\nPromise:"), promise);
  }

  process.exit(1);
});

// Graceful shutdown
process.on("SIGINT", () => {
  console.log(chalk.yellow("\n\n👋 Goodbye!"));
  process.exit(0);
});

process.on("SIGTERM", () => {
  console.log(chalk.yellow("\n\n🛑 Terminating..."));
  process.exit(0);
});

// Handle no arguments - show help
if (process.argv.length <= 2) {
  program.help();
}

// Parse command line arguments
program.parse();
