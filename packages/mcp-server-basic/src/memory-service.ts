/**
 * Memory Service - Business logic layer for Vy MCP Server
 *
 * This service orchestrates memory operations and provides the business logic
 * for MCP tools. It sits between the tool handlers and the ChromaMemoryStore.
 */

import type {
  CaptureConversationArgs,
  CaptureConversationResult,
  ConversationMemory,
  GetContextArgs,
  GetContextResult,
  MemoryType,
  SearchMemoryArgs,
  SearchMemoryResult,
  SearchQuery,
} from "@repo/core";
import type { ChromaMemoryStore } from "@repo/vector-store";
import { v7 as uuidv7 } from "uuid";
import type { Logger, MemoryServiceDeps, ServerConfig } from "./types.js";
import { ToolExecutionError } from "./types.js";

/**
 * MemoryService handles the business logic for all memory operations
 *
 * This class:
 * - Validates input arguments
 * - Orchestrates complex operations across multiple store methods
 * - Handles error cases and provides meaningful error messages
 * - Implements business rules (limits, formatting, etc.)
 * - Provides structured logging for observability
 */
export class MemoryService {
  private readonly store: ChromaMemoryStore;
  private readonly config: ServerConfig;
  private readonly logger: Logger;

  constructor(deps: MemoryServiceDeps) {
    this.store = deps.store;
    this.config = deps.config;
    this.logger = deps.logger;
  }

  /**
   * Capture and store a conversation in semantic memory
   *
   * Phase 2 Implementation:
   * 1. Validate and sanitize conversation input
   * 2. Create ConversationMemory with rich metadata
   * 3. Extract initial insights and action items (if enabled)
   * 4. Store in vector database with embedding
   * 5. Track storage metrics and performance
   * 6. Return detailed success result with extracted data
   *
   * Learning opportunities:
   * - Input validation patterns
   * - Domain object creation
   * - Error handling and recovery
   * - Business rule enforcement
   */
  async captureConversation(
    args: CaptureConversationArgs,
  ): Promise<CaptureConversationResult> {
    this.logger.info(
      {
        conversationLength: args.conversation.length,
        hasParticipants: !!args.participants?.length,
        hasMetadata: !!args.metadata,
        hasTags: !!args.tags?.length,
        hasSummary: !!args.summary,
      },
      "Capturing conversation",
    );

    const startTime = Date.now();

    try {
      // Step 1: Validate conversation input
      this.validateConversationInput(args);

      // Step 2: Create ConversationMemory object
      const memoryId = this.generateMemoryId();
      const timestamp = new Date();

      const conversationMemory: ConversationMemory = {
        id: memoryId,
        type: "conversation" as const,
        content: args.conversation,
        timestamp,
        metadata: {
          // Core metadata
          source: "mcp_server",
          version: "1.0",
          captured_at: timestamp.toISOString(),

          // User-provided metadata
          ...args.metadata,
        },

        // ConversationMemory specific fields
        participants: args.participants || ["user"],
        messageCount: this.estimateMessageCount(args.conversation),
        summary: args.summary,
        tags: args.tags || [],
      };

      // Step 3: Store the memory
      await this.store.storeMemory(conversationMemory);

      // Step 4: Extract basic insights (Phase 2 feature)
      const insights = this.extractBasicInsights(args.conversation);
      const actionItems = this.extractActionItems(args.conversation);

      const processingTime = Date.now() - startTime;

      // Step 5: Log success metrics
      this.logger.info(
        {
          memoryId,
          conversationLength: args.conversation.length,
          messageCount: conversationMemory.messageCount,
          insightsExtracted: insights.length,
          actionItemsExtracted: actionItems.length,
          processingTime,
        },
        "Conversation captured successfully",
      );

      // Step 6: Return comprehensive result matching MCP interface
      return {
        success: true,
        memoryId,
        message: `Conversation captured successfully with ${conversationMemory.messageCount} estimated messages`,
        // Phase 2 intelligence features
        extractedInsights: insights,
        actionItems,
      };
    } catch (error) {
      const processingTime = Date.now() - startTime;

      this.logger.error(
        {
          err: error instanceof Error ? error : new Error(String(error)),
          processingTime,
          conversationLength: args.conversation?.length,
        },
        "Failed to capture conversation",
      );

      throw new ToolExecutionError(
        "capture_conversation",
        error instanceof Error ? error.message : "Unknown error",
        { args, processingTime },
      );
    }
  } /**
   * Search semantic memory for relevant content
   * Phase 2: Enhanced search with context-aware features
   */
  async searchMemory(args: SearchMemoryArgs): Promise<SearchMemoryResult> {
    this.logger.info(
      {
        query:
          args.query.substring(0, 100) + (args.query.length > 100 ? "..." : ""),
        limit: args.limit,
        types: args.types,
        timeRange: args.timeRange,
        minRelevanceScore: args.minRelevanceScore,
      },
      "Searching memory",
    );

    const startTime = Date.now();

    try {
      // Build search query from args
      const searchQuery: SearchQuery = {
        query: args.query,
        limit: args.limit || 10,
        minRelevanceScore: args.minRelevanceScore || 0.7,
      };

      // Add type filters if specified
      if (args.types && args.types.length > 0) {
        searchQuery.types = args.types as MemoryType[];
      }

      // Add time range filter if specified
      if (args.timeRange) {
        searchQuery.timeRange = {
          start: args.timeRange.start
            ? new Date(args.timeRange.start)
            : undefined,
          end: args.timeRange.end ? new Date(args.timeRange.end) : undefined,
        };
      }

      // Execute search using ChromaMemoryStore
      const searchResults = await this.store.searchMemories(searchQuery);
      const searchTime = Date.now() - startTime;

      this.logger.info(
        {
          resultsCount: searchResults.length,
          searchTime,
        },
        "Search completed",
      );

      return {
        success: true,
        results: searchResults.map((result) => ({
          id: result.id,
          content:
            typeof result.content === "string"
              ? result.content
              : ((result.content as unknown as Record<string, unknown>)
                  ?.content as string) || "",
          relevanceScore: result.relevanceScore,
          timestamp:
            typeof result.content === "object" &&
            result.content &&
            "timestamp" in result.content
              ? ((result.content as unknown as Record<string, unknown>)
                  .timestamp as string)
              : new Date().toISOString(),
          type:
            typeof result.content === "object" &&
            result.content &&
            "type" in result.content
              ? ((result.content as unknown as Record<string, unknown>)
                  .type as string)
              : "unknown",
          snippet: this.generateSnippet(
            typeof result.content === "string"
              ? result.content
              : ((result.content as unknown as Record<string, unknown>)
                  ?.content as string) || "",
            args.query,
          ),
        })),
        totalCount: searchResults.length,
        searchTime,
      };
    } catch (error) {
      this.logger.error(
        { err: error instanceof Error ? error : new Error(String(error)) },
        "Failed to search memory",
      );
      throw new ToolExecutionError(
        "search_memory",
        error instanceof Error ? error.message : "Unknown error",
        { args, searchTime: Date.now() - startTime },
      );
    }
  }

  /**
   * Get contextual memories for conversation injection
   * Phase 2: AI-powered context selection
   */
  async getContext(args: GetContextArgs): Promise<GetContextResult> {
    this.logger.info(
      {
        hasCurrentQuery: !!args.currentQuery,
        recentMessageCount: args.recentMessages?.length || 0,
        maxMemories: args.maxMemories,
        maxTokens: args.maxTokens,
      },
      "Getting context",
    );

    try {
      const maxMemories = args.maxMemories || 5;
      let searchQuery: SearchQuery;

      if (args.currentQuery) {
        // Search for relevant memories using the current query
        searchQuery = {
          query: args.currentQuery,
          limit: maxMemories,
          minRelevanceScore: 0.6, // Lower threshold for context
        };
      } else {
        // Get recent memories when no specific query
        searchQuery = {
          limit: maxMemories,
          minRelevanceScore: 0.3, // Very low threshold for recent memories
        };
      }

      // Phase 2: Execute the search
      this.logger.debug({ searchQuery }, "Executing context search");
      const searchResults = await this.store.searchMemories(searchQuery);

      // Convert results to context format
      const contextMemories = searchResults.map((result) => ({
        content:
          typeof result.content === "string"
            ? result.content
            : ((result.content as unknown as Record<string, unknown>)
                ?.content as string) || "",
        relevanceScore: result.relevanceScore,
        timestamp:
          typeof result.content === "object" &&
          result.content &&
          "timestamp" in result.content
            ? ((result.content as unknown as Record<string, unknown>)
                .timestamp as string)
            : new Date().toISOString(),
        type:
          typeof result.content === "object" &&
          result.content &&
          "type" in result.content
            ? ((result.content as unknown as Record<string, unknown>)
                .type as string)
            : "unknown",
      }));

      // Phase 2 enhancement: Estimate token usage
      const estimatedTokens = this.estimateTokens(contextMemories);

      // Generate selection reasoning
      const selectionReason = this.generateSelectionReason(
        args,
        contextMemories.length,
      );

      this.logger.info(
        {
          memoriesSelected: contextMemories.length,
          estimatedTokens,
        },
        "Context retrieval completed",
      );

      return {
        success: true,
        memories: contextMemories,
        estimatedTokens,
        selectionReason,
      };
    } catch (error) {
      this.logger.error(
        { err: error instanceof Error ? error : new Error(String(error)) },
        "Failed to get context",
      );
      throw new ToolExecutionError(
        "get_context",
        error instanceof Error ? error.message : "Unknown error",
        { args },
      );
    }
  }

  // Helper methods that we'll use in our implementations

  /**
   * Validate conversation input
   */
  private validateConversationInput(args: CaptureConversationArgs): void {
    if (!args.conversation || typeof args.conversation !== "string") {
      throw new Error("Conversation content is required and must be a string");
    }

    if (args.conversation.length > this.config.limits.maxConversationLength) {
      throw new Error(
        `Conversation exceeds maximum length of ${this.config.limits.maxConversationLength} characters`,
      );
    }

    // TODO: Add more validation as needed
  }

  /**
   * Generate a UUID v7 for new memories
   */
  private generateMemoryId(): string {
    return uuidv7();
  }

  /**
   * Estimate message count from conversation content
   * Simple heuristic based on line breaks and message patterns
   */
  private estimateMessageCount(conversation: string): number {
    // Count potential message boundaries
    const lines = conversation
      .split("\n")
      .filter((line) => line.trim().length > 0);
    const messageIndicators = [
      /^(user|assistant|system|human|ai):/i,
      /^[A-Z][a-z]+:/,
      /^\d+\./,
      /^-/,
      /^>/,
    ];

    let messageCount = 0;
    for (const line of lines) {
      if (messageIndicators.some((pattern) => pattern.test(line.trim()))) {
        messageCount++;
      }
    }

    // Fall back to line-based estimation if no patterns found
    return Math.max(messageCount, Math.ceil(lines.length / 3));
  }

  /**
   * Extract basic insights from conversation content
   * Phase 2 feature: Simple pattern-based insight extraction
   */
  private extractBasicInsights(conversation: string): string[] {
    const insights: string[] = [];
    const text = conversation.toLowerCase();

    // Pattern-based insight extraction
    const patterns = [
      {
        pattern: /learn(ed|ing|s)\s+(?:that\s+)?(.{10,100})/gi,
        type: "learning",
      },
      {
        pattern: /understand(?:s)?\s+(?:that\s+)?(.{10,100})/gi,
        type: "understanding",
      },
      {
        pattern: /realize[ds]?\s+(?:that\s+)?(.{10,100})/gi,
        type: "realization",
      },
      { pattern: /prefer(?:s)?\s+(.{5,50})/gi, type: "preference" },
      {
        pattern: /goal(?:s)?\s+(?:is|are|include[s]?)\s+(.{10,100})/gi,
        type: "goal",
      },
    ];

    for (const { pattern, type } of patterns) {
      const matches = [...text.matchAll(pattern)];
      for (const match of matches) {
        if (match[1] || match[2]) {
          const insight = (match[2] || match[1] || "").trim();
          if (insight.length > 10) {
            insights.push(`${type}: ${insight}`);
          }
        }
      }
    }

    // Limit to most relevant insights
    return insights.slice(0, 5);
  }

  /**
   * Extract action items from conversation content
   * Phase 2 feature: Simple pattern-based action item extraction
   */
  private extractActionItems(conversation: string): string[] {
    const actionItems: string[] = [];
    const lines = conversation.split("\n");

    const actionPatterns = [
      /(?:need to|should|will|must|have to|going to)\s+(.{5,100})/gi,
      /(?:todo|to-do|task):\s*(.{5,100})/gi,
      /(?:action|next step):\s*(.{5,100})/gi,
      /\[ ?\]\s+(.{5,100})/gi, // Checkbox format
    ];

    for (const line of lines) {
      for (const pattern of actionPatterns) {
        const matches = [...line.matchAll(pattern)];
        for (const match of matches) {
          if (match[1]) {
            const action = match[1].trim().replace(/[.!?]+$/, "");
            if (action.length > 5 && action.length < 100) {
              actionItems.push(action);
            }
          }
        }
      }
    }

    // Remove duplicates and limit
    return [...new Set(actionItems)].slice(0, 10);
  }

  /**
   * Estimate token count for context memories
   * Phase 2 feature: Simple token estimation
   */
  private estimateTokens(
    memories: Array<{ content: string; [key: string]: unknown }>,
  ): number {
    let totalTokens = 0;

    for (const memory of memories) {
      // Simple heuristic: ~4 characters per token on average
      const contentLength = (memory.content || "").length;
      const metadataLength = JSON.stringify(memory).length - contentLength;
      totalTokens += Math.ceil((contentLength + metadataLength) / 4);
    }

    return totalTokens;
  }

  /**
   * Generate selection reasoning for context retrieval
   * Phase 2 feature: Explain why certain memories were selected
   */
  private generateSelectionReason(
    args: GetContextArgs,
    selectedCount: number,
  ): string {
    const reasons: string[] = [];

    if (args.currentQuery) {
      reasons.push(
        `searched for memories matching "${args.currentQuery.substring(0, 50)}"`,
      );
    } else {
      reasons.push("selected most recent memories");
    }

    if (args.maxMemories && selectedCount >= args.maxMemories) {
      reasons.push(`limited to ${args.maxMemories} memories as requested`);
    }

    if (selectedCount === 0) {
      return "No relevant memories found matching the criteria";
    }

    return `Selected ${selectedCount} memories by ${reasons.join(" and ")}.`;
  }

  /**
   * Estimate token count for text content
   * Simple approximation: ~4 characters per token for English
   */
  private estimateTokenCount(text: string): number {
    return Math.ceil(text.length / 4);
  }

  /**
   * Generate a snippet from content for search results
   */
  private generateSnippet(
    content: string,
    query: string,
    maxLength: number = 200,
  ): string {
    // TODO: We'll implement intelligent snippet generation
    // For now, just truncate
    return content.length > maxLength
      ? content.substring(0, maxLength) + "..."
      : content;
  }

  /**
   * Convert ChromaDB distance to relevance score (0-1)
   */
  private distanceToRelevanceScore(distance: number): number {
    // Convert distance (lower = more similar) to relevance score (higher = more relevant)
    // This is a simple linear conversion - we can make it more sophisticated later
    return Math.max(0, Math.min(1, 1 - distance));
  }
}

/**
 * Create a MemoryService with dependencies
 */
export async function createMemoryService(
  store: ChromaMemoryStore,
  config: ServerConfig,
  logger: Logger,
): Promise<MemoryService> {
  return new MemoryService({
    store,
    config,
    logger,
  });
}
