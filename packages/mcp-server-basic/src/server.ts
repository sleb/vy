/**
 * Main MCP Server implementation for Vy Semantic Memory
 *
 * This module creates and manages the MCP server instance, handling:
 * - MCP protocol integration using @modelcontextprotocol/sdk
 * - Tool registration and request routing
 * - Server lifecycle management
 * - Dependency injection and service orchestration
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { Transport } from "@modelcontextprotocol/sdk/shared/transport.js";
import { CallToolRequestSchema } from "@modelcontextprotocol/sdk/types.js";
import type {
  CaptureConversationArgs,
  GetContextArgs,
  SearchMemoryArgs,
} from "@repo/core";
import {
  createChromaClient,
  createChromaMemoryStore,
  createHostedConfig,
  createLocalConfig,
  createOpenAIEmbeddingService,
  type OpenAIEmbeddingModel,
} from "@repo/vector-store";
import { createServerConfig, getConfigSummary } from "./config.js";
import { createLoggerFromConfig } from "./logger.js";
import { createMemoryService } from "./memory-service.js";
import { createToolHandlers } from "./tools.js";
import type { Logger, ServerConfig, ServerState } from "./types.js";

/**
 * Vy MCP Server
 *
 * This is the main server class that orchestrates all components:
 * - Configuration management
 * - Dependency creation (store, service, handlers)
 * - MCP protocol server setup
 * - Tool registration and routing
 * - Error handling and logging
 */
export class VyMcpServer {
  private readonly server: Server;
  private readonly config: ServerConfig;
  private readonly logger: Logger;
  private readonly state: ServerState;

  // Dependencies (will be initialized in initialize())
  private memoryStore?: Awaited<ReturnType<typeof createChromaMemoryStore>>;
  private memoryService?: Awaited<ReturnType<typeof createMemoryService>>;
  private toolHandlers?: ReturnType<typeof createToolHandlers>;

  constructor() {
    // Initialize configuration and logging first
    this.config = createServerConfig();
    this.logger = createLoggerFromConfig(this.config);

    // Initialize server state
    this.state = {
      isRunning: false,
      startTime: new Date(),
      toolCallCount: 0,
    };

    // Create MCP server instance
    this.server = new Server(
      {
        name: this.config.name,
        version: this.config.version,
        description: this.config.description,
      },
      {
        capabilities: {
          tools: {},
        },
      },
    );

    this.logger.info(
      {
        config: getConfigSummary(this.config),
      },
      "Vy MCP Server created",
    );
  }

  /**
   * Initialize server dependencies and register tools
   *
   * This method sets up the complete dependency chain:
   * 1. Create and initialize the ChromaMemoryStore
   * 2. Create the MemoryService with dependencies
   * 3. Create the tool handlers
   * 4. Register MCP tools with the server
   * 5. Set up error handlers
   */
  async initialize(): Promise<void> {
    this.logger.info("Initializing Vy MCP Server...");

    try {
      // Step 1: Create vector store configuration from server config
      this.logger.debug("Creating vector store configuration...");
      const vectorConfig = this.createVectorStoreConfig();

      // Step 2: Initialize ChromaDB client and embedding service
      this.logger.debug(
        "Initializing ChromaDB client and embedding service...",
      );
      const chromaClient = await createChromaClient(vectorConfig.chroma);
      const embeddingService = createOpenAIEmbeddingService(
        vectorConfig.embedding,
      );

      // Step 3: Create ChromaMemoryStore
      this.logger.debug("Creating ChromaMemoryStore...");
      this.memoryStore = await createChromaMemoryStore(
        chromaClient,
        embeddingService,
        vectorConfig.collections.memories,
      );

      // Step 4: Initialize MemoryService with dependencies
      this.logger.debug("Creating MemoryService...");
      this.memoryService = await createMemoryService(
        this.memoryStore,
        this.config,
        this.logger,
      );

      // Step 5: Initialize tool handlers
      this.logger.debug("Creating tool handlers...");
      this.toolHandlers = createToolHandlers(this.memoryService, this.logger);

      // Step 6: Register MCP tools
      this.logger.debug("Registering MCP tools...");
      this.registerTools();

      // Step 7: Set up error handlers
      this.logger.debug("Setting up error handlers...");
      this.setupErrorHandlers();

      this.logger.info(
        {
          hasMemoryStore: !!this.memoryStore,
          hasMemoryService: !!this.memoryService,
          hasToolHandlers: !!this.toolHandlers,
        },
        "Vy MCP Server initialization complete",
      );
    } catch (error) {
      this.state.lastError =
        error instanceof Error ? error : new Error(String(error));
      this.logger.error(
        { err: this.state.lastError },
        "Failed to initialize server",
      );
      throw error;
    }
  }

  /**
   * Connect to MCP transport and start serving
   *
   * This method completes the server startup process:
   * 1. Validates server is properly initialized
   * 2. Connects to the provided MCP transport
   * 3. Updates server state to running
   * 4. Logs successful startup
   */
  async connect(transport: Transport): Promise<void> {
    if (!this.memoryStore || !this.memoryService || !this.toolHandlers) {
      throw new Error("Server must be initialized before connecting");
    }

    try {
      this.logger.info("Connecting to MCP transport...");
      await this.server.connect(transport);

      this.state.isRunning = true;
      this.state.startTime = new Date();
      this.logger.info(
        {
          startTime: this.state.startTime,
          running: this.state.isRunning,
        },
        "Vy MCP Server is running",
      );
    } catch (error) {
      this.state.lastError =
        error instanceof Error ? error : new Error(String(error));
      this.logger.error(
        { err: this.state.lastError },
        "Failed to connect to transport",
      );
      throw error;
    }
  }

  /**
   * Register MCP tools with the server
   *
   * Registers all three core tools with the MCP protocol:
   * - capture_conversation: Store conversations in semantic memory
   * - search_memory: Search stored memories with semantic similarity
   * - get_context: Retrieve relevant context for AI injection
   */
  private registerTools(): void {
    this.logger.debug("Registering MCP tools...");

    try {
      this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
        const toolName = request.params.name;
        const toolArgs = request.params.arguments;
        this.logger.debug({ toolName, toolArgs }, "Received tool call");

        const result = await this.handleToolCall(toolName, toolArgs);
        return { content: [{ type: "text", text: JSON.stringify(result) }] };
      });

      this.logger.debug(
        {
          toolCount: 3,
          tools: ["capture_conversation", "search_memory", "get_context"],
        },
        "MCP tools registered successfully",
      );
    } catch (error) {
      this.logger.error(
        { err: error instanceof Error ? error : new Error(String(error)) },
        "Failed to register tools",
      );
      throw error;
    }

    this.logger.info(
      {
        tools: ["capture_conversation", "search_memory", "get_context"],
      },
      "MCP tools registered",
    );
  }

  /**
   * Handle individual tool calls with logging and error handling
   *
   * This method provides:
   * - Request lifecycle tracking (timing, metrics)
   * - Tool validation and routing
   * - Comprehensive error handling and logging
   * - Performance monitoring
   */
  private async handleToolCall(
    toolName: string,
    args: unknown,
  ): Promise<unknown> {
    const startTime = Date.now();
    this.state.toolCallCount++;

    this.logger.info(
      {
        toolName,
        callNumber: this.state.toolCallCount,
      },
      "Handling tool call",
    );

    try {
      if (!this.toolHandlers) {
        throw new Error("Tool handlers not initialized");
      }

      // Validate arguments before calling tools
      this.validateToolArgs(args, toolName);

      let result: unknown;
      switch (toolName) {
        case "capture_conversation":
          result = await this.toolHandlers.captureConversation(
            args as CaptureConversationArgs,
          );
          break;
        case "search_memory":
          result = await this.toolHandlers.searchMemory(
            args as SearchMemoryArgs,
          );
          break;
        case "get_context":
          result = await this.toolHandlers.getContext(args as GetContextArgs);
          break;
        default:
          throw new Error(`Unknown tool: ${toolName}`);
      }
      const duration = Date.now() - startTime;
      this.logger.info(
        {
          toolName,
          duration,
          callNumber: this.state.toolCallCount,
        },
        "Tool call completed successfully",
      );

      return result;
    } catch (error) {
      const duration = Date.now() - startTime;
      this.logger.error(
        {
          err: error instanceof Error ? error : new Error(String(error)),
          toolName,
          duration,
          callNumber: this.state.toolCallCount,
        },
        `Tool call failed: ${toolName}`,
      );

      throw error;
    }
  }

  /**
   * Set up server error handlers
   */
  private setupErrorHandlers(): void {
    this.server.onerror = (error) => {
      this.state.lastError = error;
      this.logger.error({ err: error }, "MCP Server error");
    };

    // Handle uncaught exceptions
    process.on("uncaughtException", (error) => {
      this.logger.error({ err: error }, "Uncaught exception");
      this.shutdown(1);
    });

    // Handle unhandled promise rejections
    process.on("unhandledRejection", (reason, promise) => {
      this.logger.error(
        {
          err: new Error(String(reason)),
          promise: String(promise),
        },
        "Unhandled promise rejection",
      );
      this.shutdown(1);
    });
  }

  /**
   * Get server health status
   */
  getHealth(): {
    status: "healthy" | "unhealthy";
    details: Record<string, unknown>;
  } {
    const isHealthy = this.state.isRunning && !this.state.lastError;

    return {
      status: isHealthy ? "healthy" : "unhealthy",
      details: {
        running: this.state.isRunning,
        uptime: Date.now() - this.state.startTime.getTime(),
        toolCalls: this.state.toolCallCount,
        lastError: this.state.lastError?.message,
        hasMemoryStore: !!this.memoryStore,
        hasMemoryService: !!this.memoryService,
        hasToolHandlers: !!this.toolHandlers,
      },
    };
  }

  /**
   * Get server statistics
   */
  getStats(): Record<string, number> {
    return {
      uptime: Date.now() - this.state.startTime.getTime(),
      toolCalls: this.state.toolCallCount,
      // TODO: Add more stats from memory store
    };
  }

  /**
   * Gracefully shutdown the server
   */
  async shutdown(exitCode: number = 0): Promise<void> {
    this.logger.info(
      {
        exitCode,
        uptime: Date.now() - this.state.startTime.getTime(),
        toolCalls: this.state.toolCallCount,
      },
      "Shutting down Vy MCP Server...",
    );

    this.state.isRunning = false;

    try {
      // TODO: Close any open connections
      // TODO: Flush any pending operations
      // TODO: Clean up resources

      this.logger.info("Vy MCP Server shutdown complete");
    } catch (error) {
      this.logger.error(
        { err: error instanceof Error ? error : new Error(String(error)) },
        "Error during shutdown",
      );
    }

    process.exit(exitCode);
  }

  /**
   * Create vector store configuration from server configuration
   *
   * This method translates our MCP server configuration into the format
   * expected by the vector-store package, handling both local and hosted
   * ChromaDB configurations.
   */
  private createVectorStoreConfig() {
    const { vectorStore, embedding } = this.config;

    // Determine if this is a hosted or local configuration
    const isHosted = !!vectorStore.chromaApiKey;

    if (isHosted) {
      // Create hosted configuration
      return createHostedConfig();
    } else {
      // Create local configuration with our specific settings
      return createLocalConfig({
        chroma: {
          host: vectorStore.chromaHost,
          port: vectorStore.chromaPort,
        },
        embedding: {
          apiKey: embedding.openaiApiKey,
          model: embedding.model as OpenAIEmbeddingModel,
          dimensions: this.getEmbeddingDimensions(embedding.model),
          maxTokens: 8192,
          batchSize: 100,
        },
        collections: {
          memories: vectorStore.collectionName,
        },
      });
    }
  }

  /**
   * Validate tool arguments based on tool name
   */
  private validateToolArgs(args: unknown, toolName: string): void {
    if (!args || typeof args !== "object") {
      throw new Error(`Invalid arguments for ${toolName}: must be an object`);
    }

    switch (toolName) {
      case "capture_conversation":
        if (!("conversation" in args)) {
          throw new Error("capture_conversation requires 'conversation' field");
        }
        break;
      case "search_memory":
        if (!("query" in args)) {
          throw new Error("search_memory requires 'query' field");
        }
        break;
      case "get_context":
        // get_context has all optional fields, so just validate it's an object
        break;
      default:
        throw new Error(`Unknown tool: ${toolName}`);
    }
  }

  /**
   * Get embedding dimensions for the specified model
   */
  private getEmbeddingDimensions(model: string): number {
    switch (model) {
      case "text-embedding-3-small":
      case "text-embedding-ada-002":
        return 1536;
      case "text-embedding-3-large":
        return 3072;
      default:
        return 1536; // Default fallback
    }
  }
}

/**
 * Create and initialize a Vy MCP Server
 *
 * This is the main factory function that creates a server instance
 * and initializes all its dependencies.
 */
export async function createVyMcpServer(): Promise<VyMcpServer> {
  const server = new VyMcpServer();
  await server.initialize();
  return server;
}

/**
 * Export server health check for monitoring
 */
export async function checkServerHealth(): Promise<Record<string, unknown>> {
  try {
    const server = new VyMcpServer();
    return server.getHealth();
  } catch (error) {
    return {
      status: "unhealthy",
      error: error instanceof Error ? error.message : String(error),
    };
  }
}
