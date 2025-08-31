/**
 * Memory Commands - Core user interface for Vy semantic memory
 *
 * These commands provide the main user interface for capturing, searching,
 * and retrieving memories from the semantic memory system.
 */

import type {
  CaptureConversationArgs,
  GetContextArgs,
  SearchMemoryArgs,
} from "@repo/core";
import chalk from "chalk";
import { readFile } from "node:fs/promises";
import ora from "ora";
import prompts from "prompts";
import { table } from "table";
import {
  formatDuration,
  formatMemory,
  formatTimestamp,
} from "../../lib/formatters.js";
import { createMcpClient } from "../../lib/mcp-client.js";
import { handleError, validateConfig } from "../../lib/utils.js";

/**
 * Capture a conversation or thought in semantic memory
 */
export async function capture(
  text?: string,
  options?: Record<string, unknown>,
): Promise<void> {
  const spinner = ora("Initializing memory capture...").start();

  try {
    // Validate configuration
    await validateConfig();
    spinner.succeed("Configuration validated");

    // Get content to capture
    let content: string;
    if (options?.fromFile) {
      spinner.start("Reading content from file...");
      content = await readFile(options.fromFile, "utf-8");
      spinner.succeed(`Content loaded from ${options.fromFile}`);
    } else if (text) {
      content = text;
    } else {
      spinner.stop();
      const response = await prompts({
        type: "text",
        name: "content",
        message: "What would you like to capture?",
        validate: (value: string) =>
          value.trim().length > 0 || "Content cannot be empty",
      });

      if (!response.content) {
        console.log(chalk.yellow("👋 Capture cancelled"));
        return;
      }
      content = response.content;
      spinner.start("Processing content...");
    }

    // Parse metadata if provided
    let metadata;
    if (options?.metadata) {
      try {
        metadata = JSON.parse(options.metadata);
      } catch {
        throw new Error("Invalid metadata JSON format");
      }
    }

    // Build capture arguments
    const args: CaptureConversationArgs = {
      conversation: content,
      ...(options?.participants && { participants: options.participants }),
      ...(options?.tags && { tags: options.tags }),
      ...(options?.summary && { summary: options.summary }),
      ...(metadata && { metadata }),
    };

    // Connect to MCP server and capture
    spinner.text = "Connecting to memory system...";
    const client = await createMcpClient();

    spinner.text = "Capturing memory...";
    const startTime = Date.now();
    const result = await client.callTool("capture_conversation", args);
    const duration = Date.now() - startTime;

    await client.close();
    spinner.succeed(`Memory captured in ${formatDuration(duration)}`);

    // Display results
    if (result.success) {
      console.log(chalk.green("\n✅ Successfully captured memory"));
      console.log(chalk.gray(`   Memory ID: ${result.memoryId}`));

      if (result.extractedInsights?.length) {
        console.log(chalk.blue("\n💡 Extracted insights:"));
        result.extractedInsights.forEach((insight: string) => {
          console.log(chalk.gray(`   • ${insight}`));
        });
      }

      if (result.actionItems?.length) {
        console.log(chalk.yellow("\n📋 Action items:"));
        result.actionItems.forEach((item: string) => {
          console.log(chalk.gray(`   • ${item}`));
        });
      }
    } else {
      throw new Error(result.message || "Failed to capture memory");
    }
  } catch (error) {
    spinner.fail("Memory capture failed");
    handleError(error, options?.verbose);
  }
}

/**
 * Search semantic memories
 */
export async function search(
  query: string,
  options?: Record<string, unknown>,
): Promise<void> {
  const spinner = ora("Searching memories...").start();

  try {
    await validateConfig();

    // Build search arguments
    const args: SearchMemoryArgs = {
      query,
      limit: parseInt(options?.limit || "10"),
      ...(options?.types && { types: options.types }),
      ...(options?.minScore && {
        minRelevanceScore: parseFloat(options.minScore),
      }),
    };

    // Handle date filters
    if (options?.since || options?.until) {
      args.timeRange = {};
      if (options.since) {
        args.timeRange.start = new Date(options.since).toISOString();
      }
      if (options.until) {
        args.timeRange.end = new Date(options.until).toISOString();
      }
    }

    // Connect and search
    const client = await createMcpClient();
    const startTime = Date.now();
    const result = await client.callTool("search_memory", args);
    const duration = Date.now() - startTime;

    await client.close();
    spinner.succeed(
      `Found ${result.totalCount} memories in ${formatDuration(duration)}`,
    );

    // Display results
    if (result.success && result.results?.length) {
      if (options?.json) {
        console.log(JSON.stringify(result, null, 2));
        return;
      }

      console.log(chalk.blue(`\n🔍 Search results for: "${query}"`));
      console.log(
        chalk.gray(
          `   Found ${result.totalCount} memories (showing ${result.results.length})\n`,
        ),
      );

      // Format as table
      const tableData = [["Score", "Type", "Date", "Snippet"]];

      result.results.forEach((memory: Record<string, unknown>) => {
        tableData.push([
          chalk.green((memory.relevanceScore * 100).toFixed(1) + "%"),
          chalk.yellow(memory.type),
          chalk.gray(formatTimestamp(memory.timestamp)),
          formatMemory(memory.snippet || memory.content, 60),
        ]);
      });

      console.log(
        table(tableData, {
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
          },
        }),
      );
    } else {
      console.log(chalk.yellow("🔍 No memories found matching your query"));

      if (query.length < 3) {
        console.log(chalk.gray("   Try using a longer, more specific query"));
      }
    }
  } catch (error) {
    spinner.fail("Search failed");
    handleError(error, options?.verbose);
  }
}

/**
 * Get relevant context for current situation
 */
export async function context(
  options?: Record<string, unknown>,
): Promise<void> {
  const spinner = ora("Retrieving context...").start();

  try {
    await validateConfig();

    // Get current query if not provided
    let currentQuery = options?.query;
    if (!currentQuery && !options?.recent?.length) {
      spinner.stop();
      const response = await prompts({
        type: "text",
        name: "query",
        message: "What context are you looking for?",
        validate: (value: string) =>
          value.trim().length > 0 || "Query cannot be empty",
      });

      if (!response.query) {
        console.log(chalk.yellow("👋 Context retrieval cancelled"));
        return;
      }
      currentQuery = response.query;
      spinner.start("Retrieving context...");
    }

    // Build context arguments
    const args: GetContextArgs = {
      ...(currentQuery && { currentQuery }),
      ...(options?.recent && { recentMessages: options.recent }),
      maxMemories: parseInt(options?.maxMemories || "10"),
      maxTokens: parseInt(options?.maxTokens || "2000"),
    };

    // Connect and get context
    const client = await createMcpClient();
    const startTime = Date.now();
    const result = await client.callTool("get_context", args);
    const duration = Date.now() - startTime;

    await client.close();
    spinner.succeed(`Context retrieved in ${formatDuration(duration)}`);

    // Display results
    if (result.success && result.memories?.length) {
      if (options?.json) {
        console.log(JSON.stringify(result, null, 2));
        return;
      }

      console.log(chalk.blue("\n🎯 Relevant context:"));
      console.log(
        chalk.gray(
          `   ${result.memories.length} memories, ~${result.estimatedTokens} tokens\n`,
        ),
      );

      result.memories.forEach(
        (memory: Record<string, unknown>, index: number) => {
          const score = (memory.relevanceScore * 100).toFixed(1);
          console.log(
            chalk.green(
              `${index + 1}. [${score}%] ${formatTimestamp(memory.timestamp)}`,
            ),
          );
          console.log(chalk.gray(`   ${formatMemory(memory.content, 120)}\n`));
        },
      );

      if (result.selectionReason) {
        console.log(chalk.blue("🤔 Selection reasoning:"));
        console.log(chalk.gray(`   ${result.selectionReason}`));
      }
    } else {
      console.log(chalk.yellow("🎯 No relevant context found"));
      console.log(
        chalk.gray("   Try a different query or capture more memories first"),
      );
    }
  } catch (error) {
    spinner.fail("Context retrieval failed");
    handleError(error, options?.verbose);
  }
}

/**
 * List stored memories
 */
export async function list(options?: Record<string, unknown>): Promise<void> {
  const spinner = ora("Loading memories...").start();

  try {
    await validateConfig();

    // Use search with empty query to list memories
    const args: SearchMemoryArgs = {
      query: "",
      limit: parseInt(options?.limit || "20"),
      ...(options?.type && { types: [options.type] }),
    };

    // Handle date filters
    if (options?.since) {
      args.timeRange = { start: new Date(options.since).toISOString() };
    }

    const client = await createMcpClient();
    const result = await client.callTool("search_memory", args);
    await client.close();

    spinner.succeed(`Loaded ${result.results?.length || 0} memories`);

    if (result.success && result.results?.length) {
      if (options?.json) {
        console.log(JSON.stringify(result, null, 2));
        return;
      }

      console.log(chalk.blue("\n📋 Your memories:"));
      console.log(chalk.gray(`   Total: ${result.totalCount} memories\n`));

      // Sort by date or relevance
      const sortField = options?.sort || "date";
      const sortedResults = [...result.results].sort(
        (a: Record<string, unknown>, b: Record<string, unknown>) => {
          if (sortField === "relevance") {
            return b.relevanceScore - a.relevanceScore;
          }
          return (
            new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
          );
        },
      );

      sortedResults.forEach(
        (memory: Record<string, unknown>, index: number) => {
          console.log(
            chalk.yellow(
              `${index + 1}. ${formatTimestamp(memory.timestamp)} [${memory.type}]`,
            ),
          );
          console.log(
            chalk.gray(
              `   ${formatMemory(memory.content || memory.snippet, 100)}`,
            ),
          );
          console.log(chalk.gray(`   ID: ${memory.id}\n`));
        },
      );
    } else {
      console.log(chalk.yellow("📋 No memories found"));
      console.log(
        chalk.gray("   Start by capturing some conversations or thoughts:"),
      );
      console.log(chalk.gray('   vy mem capture "Your first thought..."'));
    }
  } catch (error) {
    spinner.fail("Failed to load memories");
    handleError(error, options?.verbose);
  }
}

/**
 * Delete memories
 */
export async function remove(
  memoryId: string,
  options?: Record<string, unknown>,
): Promise<void> {
  const spinner = ora("Preparing to delete memory...").start();

  try {
    await validateConfig();
    spinner.stop();

    // Confirmation prompt unless --force is used
    if (!options?.force) {
      const response = await prompts({
        type: "confirm",
        name: "confirmed",
        message: `Are you sure you want to delete memory ${memoryId}?`,
        initial: false,
      });

      if (!response.confirmed) {
        console.log(chalk.yellow("👋 Deletion cancelled"));
        console.log(chalk.gray("Memory deletion cancelled"));
        return;
      }
    }

    spinner.text = `Deleting memory ${memoryId}...`;

    // TODO: Implement memory deletion when available
    // For now, show that the command structure is ready
    spinner.fail("Memory deletion not yet implemented");
    console.log(
      chalk.yellow("⚠️  Memory deletion will be implemented in Phase 2"),
    );
    console.log(chalk.gray(`   Memory ID: ${memoryId}`));
  } catch (error) {
    spinner.fail("Delete operation failed");
    handleError(error, options?.verbose);
  }
}

// Export all commands
export const memoryCommands = {
  capture,
  search,
  context,
  list,
  delete: remove,
};
