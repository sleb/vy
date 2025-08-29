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
import {
  createChromaClient,
  createChromaMemoryStore,
  createHostedConfig,
  createLocalConfig,
  createOpenAIEmbeddingService,
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

    this.logger.info("Vy MCP Server created", {
      config: getConfigSummary(this.config),
    });
  }

  /**
   * Initialize server dependencies and register tools
   *
   * TODO: We'll implement this together! This should:
   * 1. Create and initialize the ChromaMemoryStore
   * 2. Create the MemoryService with dependencies
   * 3. Create the tool handlers
   * 4. Register MCP tools with the server
   * 5. Set up error handlers
   * 6. Validate that everything is working
   *
   * Learning opportunities:
   * - Dependency injection patterns
   * - Async initialization patterns
   * - MCP tool registration
   * - Error handling in server setup
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

      this.logger.info("Vy MCP Server initialization complete", {
        hasMemoryStore: !!this.memoryStore,
        hasMemoryService: !!this.memoryService,
        hasToolHandlers: !!this.toolHandlers,
      });
    } catch (error) {
      this.state.lastError =
        error instanceof Error ? error : new Error(String(error));
      this.logger.error("Failed to initialize server", this.state.lastError);
      throw error;
    }
  }

  /**
   * Connect to MCP transport and start serving
   *
   * TODO: We'll implement this together! This should:
   * 1. Ensure server is initialized
   * 2. Connect to the provided transport
   * 3. Update server state
   * 4. Set up graceful shutdown handlers
   * 5. Log server startup information
   *
   * Learning opportunities:
   * - MCP transport integration
   * - Server lifecycle management
   * - Graceful shutdown patterns
   * - Process signal handling
   */
  async connect(transport: any): Promise<void> {
    if (!this.memoryStore || !this.memoryService || !this.toolHandlers) {
      throw new Error("Server must be initialized before connecting");
    }

    try {
      this.logger.info("Connecting to MCP transport...");

      // TODO: Connect server to transport
      // await this.server.connect(transport);

      // TODO: Update server state
      // this.state.isRunning = true;
      // this.state.startTime = new Date();

      // TODO: Log successful startup
      // this.logger.info('Vy MCP Server is running', { ... });

      throw new Error("Not implemented yet - we'll do this together!");
    } catch (error) {
      this.state.lastError =
        error instanceof Error ? error : new Error(String(error));
      this.logger.error("Failed to connect to transport", this.state.lastError);
      throw error;
    }
  }

  /**
   * Register MCP tools with the server
   *
   * TODO: We'll implement this together! This should:
   * 1. Register the capture_conversation tool
   * 2. Register the search_memory tool
   * 3. Register the get_context tool
   * 4. Set up tool call handlers that route to our tool handlers
   * 5. Add request/response logging and metrics
   *
   * Learning opportunities:
   * - MCP tool registration patterns
   * - Request routing and middleware
   * - Observability and metrics collection
   * - Tool schema validation
   */
  private registerTools(): void {
    this.logger.debug("Registering MCP tools...");

    try {
      // TODO: Register capture_conversation tool
      // this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      //   if (request.params.name === 'capture_conversation') {
      //     return this.handleToolCall('capture_conversation', request.params.arguments);
      //   }
      // });

      // TODO: Register search_memory tool

      // TODO: Register get_context tool

      // TODO: Add catch-all handler for unknown tools

      throw new Error("Not implemented yet - we'll do this together!");
    } catch (error) {
      this.logger.error(
        "Failed to register tools",
        error instanceof Error ? error : new Error(String(error)),
      );
      throw error;
    }
  }

  /**
   * Handle individual tool calls with logging and error handling
   *
   * TODO: We'll implement this together! This should:
   * 1. Log the incoming tool call
   * 2. Validate the tool name
   * 3. Route to the appropriate tool handler
   * 4. Handle any errors gracefully
   * 5. Log the response and update metrics
   * 6. Return properly formatted MCP response
   *
   * Learning opportunities:
   * - Request/response middleware patterns
   * - Error handling and user experience
   * - Metrics and observability
   * - MCP response formatting
   */
  private async handleToolCall(toolName: string, args: unknown): Promise<any> {
    const startTime = Date.now();
    this.state.toolCallCount++;

    this.logger.info("Handling tool call", {
      toolName,
      callNumber: this.state.toolCallCount,
    });

    try {
      if (!this.toolHandlers) {
        throw new Error("Tool handlers not initialized");
      }

      // TODO: Route to appropriate tool handler
      // switch (toolName) {
      //   case 'capture_conversation':
      //     return await this.toolHandlers.captureConversation(args);
      //   case 'search_memory':
      //     return await this.toolHandlers.searchMemory(args);
      //   case 'get_context':
      //     return await this.toolHandlers.getContext(args);
      //   default:
      //     throw new Error(`Unknown tool: ${toolName}`);
      // }

      throw new Error("Not implemented yet - we'll do this together!");
    } catch (error) {
      const duration = Date.now() - startTime;
      this.logger.error(
        `Tool call failed: ${toolName}`,
        error instanceof Error ? error : new Error(String(error)),
        {
          toolName,
          duration,
          callNumber: this.state.toolCallCount,
        },
      );

      // TODO: Return formatted error response
      throw error;
    }
  }

  /**
   * Set up server error handlers
   */
  private setupErrorHandlers(): void {
    this.server.onerror = (error) => {
      this.state.lastError = error;
      this.logger.error("MCP Server error", error);
    };

    // Handle uncaught exceptions
    process.on("uncaughtException", (error) => {
      this.logger.error("Uncaught exception", error);
      this.shutdown(1);
    });

    // Handle unhandled promise rejections
    process.on("unhandledRejection", (reason, promise) => {
      this.logger.error(
        "Unhandled promise rejection",
        new Error(String(reason)),
        {
          promise: String(promise),
        },
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
    this.logger.info("Shutting down Vy MCP Server...", {
      exitCode,
      uptime: Date.now() - this.state.startTime.getTime(),
      toolCalls: this.state.toolCallCount,
    });

    this.state.isRunning = false;

    try {
      // TODO: Close any open connections
      // TODO: Flush any pending operations
      // TODO: Clean up resources

      this.logger.info("Vy MCP Server shutdown complete");
    } catch (error) {
      this.logger.error(
        "Error during shutdown",
        error instanceof Error ? error : new Error(String(error)),
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
          model: embedding.model as any, // Type assertion needed for model compatibility
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
