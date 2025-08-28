/**
 * Model Context Protocol (MCP) interfaces for Vy semantic memory system
 *
 * Defines tool interfaces, resource management, and prompt integration
 * for seamless AI assistant integration
 */

import type { MemoryId } from './memory.js';

/**
 * MCP tool argument schemas
 */
export interface CaptureConversationArgs {
  conversation: string;
  participants?: string[];
  metadata?: Record<string, unknown>;
  tags?: string[];
  summary?: string;
}

export interface SearchMemoryArgs {
  query: string;
  limit?: number;
  types?: string[];
  timeRange?: {
    start?: string; // ISO string
    end?: string;   // ISO string
  };
  minRelevanceScore?: number;
}

export interface GetContextArgs {
  currentQuery?: string;
  recentMessages?: string[];
  maxMemories?: number;
  maxTokens?: number;
}

/**
 * MCP tool response types
 */
export interface CaptureConversationResult {
  success: boolean;
  memoryId: MemoryId;
  message: string;
  extractedInsights?: string[];
  actionItems?: string[];
}

export interface SearchMemoryResult {
  success: boolean;
  results: Array<{
    id: MemoryId;
    content: string;
    relevanceScore: number;
    timestamp: string; // ISO string
    type: string;
    snippet?: string;
  }>;
  totalCount: number;
  searchTime: number;
}

export interface GetContextResult {
  success: boolean;
  memories: Array<{
    content: string;
    relevanceScore: number;
    timestamp: string;
    type: string;
  }>;
  estimatedTokens: number;
  selectionReason: string;
}

/**
 * MCP resource types for exposing stored memories
 */
export interface MemoryResource {
  uri: string;
  name: string;
  description: string;
  mimeType: 'text/plain' | 'application/json';
}

/**
 * MCP resource content
 */
export interface MemoryResourceContent {
  uri: string;
  mimeType: string;
  text: string;
}

/**
 * MCP tool definitions
 */
export interface McpToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: Record<string, unknown>;
    required?: string[];
  };
}

/**
 * Core MCP tools for Vy
 */
export const VY_MCP_TOOLS: McpToolDefinition[] = [
  {
    name: 'capture_conversation',
    description: 'Store a conversation in semantic memory with automatic insight extraction',
    inputSchema: {
      type: 'object',
      properties: {
        conversation: {
          type: 'string',
          description: 'The full conversation content to store'
        },
        participants: {
          type: 'array',
          items: { type: 'string' },
          description: 'List of conversation participants'
        },
        metadata: {
          type: 'object',
          description: 'Additional metadata about the conversation'
        },
        tags: {
          type: 'array',
          items: { type: 'string' },
          description: 'Tags to associate with this conversation'
        },
        summary: {
          type: 'string',
          description: 'Optional summary of the conversation'
        }
      },
      required: ['conversation']
    }
  },
  {
    name: 'search_memory',
    description: 'Search semantic memory for relevant conversations and insights',
    inputSchema: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Semantic search query'
        },
        limit: {
          type: 'number',
          description: 'Maximum number of results to return',
          minimum: 1,
          maximum: 50
        },
        types: {
          type: 'array',
          items: { type: 'string' },
          description: 'Filter by memory types (conversation, insight, etc.)'
        },
        timeRange: {
          type: 'object',
          properties: {
            start: { type: 'string', format: 'date-time' },
            end: { type: 'string', format: 'date-time' }
          },
          description: 'Filter by time range'
        },
        minRelevanceScore: {
          type: 'number',
          minimum: 0,
          maximum: 1,
          description: 'Minimum relevance score threshold'
        }
      },
      required: ['query']
    }
  },
  {
    name: 'get_context',
    description: 'Get relevant memories for context injection in a new conversation',
    inputSchema: {
      type: 'object',
      properties: {
        currentQuery: {
          type: 'string',
          description: 'Current conversation context or query'
        },
        recentMessages: {
          type: 'array',
          items: { type: 'string' },
          description: 'Recent messages from current conversation'
        },
        maxMemories: {
          type: 'number',
          description: 'Maximum number of memories to return',
          minimum: 1,
          maximum: 20
        },
        maxTokens: {
          type: 'number',
          description: 'Approximate token budget for context',
          minimum: 100,
          maximum: 10000
        }
      }
    }
  }
];

/**
 * MCP tool handler interface
 */
export interface McpToolHandler {
  captureConversation(args: CaptureConversationArgs): Promise<CaptureConversationResult>;
  searchMemory(args: SearchMemoryArgs): Promise<SearchMemoryResult>;
  getContext(args: GetContextArgs): Promise<GetContextResult>;
}

/**
 * MCP resource handler interface
 */
export interface McpResourceHandler {
  listResources(): Promise<MemoryResource[]>;
  getResource(uri: string): Promise<MemoryResourceContent>;
}

/**
 * MCP server configuration
 */
export interface McpServerConfig {
  name: string;
  version: string;
  description: string;

  // Memory store configuration
  memoryStore: {
    type: 'chromadb' | 'memory' | 'custom';
    config: Record<string, unknown>;
  };

  // Embedding service configuration
  embeddingService: {
    provider: 'openai' | 'huggingface' | 'local' | 'custom';
    model: string;
    config: Record<string, unknown>;
  };

  // Feature flags
  features: {
    insightExtraction: boolean;
    actionItemDetection: boolean;
    resourceExposure: boolean;
    analyticsLogging: boolean;
  };

  // Limits and quotas
  limits: {
    maxConversationLength: number;
    maxSearchResults: number;
    maxContextMemories: number;
    rateLimitPerHour?: number;
  };
}

/**
 * MCP server interface
 */
export interface McpServer {
  // Server lifecycle
  start(): Promise<void>;
  stop(): Promise<void>;
  isRunning(): boolean;

  // Tool handling
  handleToolCall(toolName: string, args: unknown): Promise<unknown>;

  // Resource handling
  listResources(): Promise<MemoryResource[]>;
  getResource(uri: string): Promise<MemoryResourceContent>;

  // Health and monitoring
  getHealth(): Promise<{ status: 'healthy' | 'unhealthy'; details: Record<string, unknown> }>;
  getStats(): Promise<Record<string, number>>;
}

/**
 * Prompt templates for context injection
 */
export interface PromptTemplate {
  name: string;
  description: string;
  template: string;
  variables: string[];
}

/**
 * Context injection strategies
 */
export type ContextInjectionStrategy =
  | 'prepend'    // Add context at the beginning of conversation
  | 'append'     // Add context at the end of conversation
  | 'summarize'  // Summarize context and inject summary
  | 'inline'     // Inject context inline with messages
  | 'metadata';  // Provide context as metadata only

/**
 * Context injection configuration
 */
export interface ContextInjectionConfig {
  strategy: ContextInjectionStrategy;
  maxTokens: number;
  template?: PromptTemplate;
  includeSources: boolean;
  includeTimestamps: boolean;
  includeRelevanceScores: boolean;
}
