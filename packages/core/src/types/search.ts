/**
 * Search and query types for Vy semantic memory system
 *
 * Supports rich querying with semantic search, metadata filtering,
 * and temporal constraints
 */

import type { MemoryId, MemoryType, Timestamp } from './memory.js';

/**
 * Time range filter for queries
 */
export interface TimeRange {
  start?: Timestamp;
  end?: Timestamp;
}

/**
 * Metadata filter for precise matching
 */
export type MetadataFilter = Record<string, unknown>;

/**
 * Search query interface - rich querying capabilities
 * Start simple but designed for extensibility
 */
export interface SearchQuery {
  // Semantic search
  query?: string;

  // Type filtering
  types?: MemoryType[];

  // Temporal filtering
  timeRange?: TimeRange;

  // Metadata filtering
  metadata?: MetadataFilter;

  // Tag filtering (for memories that support tags)
  tags?: string[];

  // Relevance threshold (0-1, higher = more similar required)
  minRelevanceScore?: number;

  // Result limiting
  limit?: number;
  offset?: number;
}

/**
 * Simple search query for basic use cases
 */
export interface SimpleSearchQuery {
  query: string;
  limit?: number;
}

/**
 * Search result with relevance scoring
 */
export interface SearchResult<T = unknown> {
  id: MemoryId;
  content: T;
  relevanceScore: number; // 0-1, higher = more relevant
  snippet?: string; // Optional highlighted excerpt
}

/**
 * Search response with metadata
 */
export interface SearchResponse<T = unknown> {
  results: SearchResult<T>[];
  totalCount: number;
  searchTime: number; // in milliseconds
  query: SearchQuery;
}

/**
 * Vector search specific types
 */
export interface VectorSearchQuery {
  embedding: number[];
  limit?: number;
  minSimilarity?: number; // 0-1, cosine similarity threshold
  metadata?: MetadataFilter;
}

export interface VectorSearchResult {
  id: MemoryId;
  similarity: number; // cosine similarity score
  metadata: Record<string, unknown>;
}

/**
 * Faceted search results - for exploring the memory space
 * Phase 2 feature for analytics and discovery
 */
export interface FacetedSearchResponse<T = unknown> extends SearchResponse<T> {
  facets: {
    types: Array<{ type: MemoryType; count: number }>;
    timeRanges: Array<{ range: string; count: number }>;
    tags: Array<{ tag: string; count: number }>;
  };
}

/**
 * Search suggestion - for autocomplete and query assistance
 * Phase 2 feature
 */
export interface SearchSuggestion {
  text: string;
  type: 'query' | 'tag' | 'metadata' | 'semantic';
  score: number;
}

/**
 * Context retrieval query - for getting relevant memories for a new conversation
 * This is key for the "inject relevant context" use case
 */
export interface ContextQuery {
  // Current conversation context
  currentQuery?: string;
  recentMessages?: string[];

  // Context preferences
  maxMemories?: number;
  maxTokens?: number; // Approximate token budget for context

  // Temporal preferences (prefer recent vs. relevant)
  recencyWeight?: number; // 0-1, higher = prefer recent memories
  relevanceWeight?: number; // 0-1, higher = prefer relevant memories

  // Type preferences
  preferredTypes?: MemoryType[];

  // Diversity settings
  ensureDiversity?: boolean; // Avoid too many similar results
}

/**
 * Context result - memories selected for context injection
 */
export interface ContextResult {
  memories: SearchResult[];
  estimatedTokens: number;
  selectionReason: string; // Why these memories were chosen
}
