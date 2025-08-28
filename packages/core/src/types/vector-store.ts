/**
 * Vector store interfaces for Vy semantic memory system
 *
 * Provides abstraction layer over ChromaDB and other vector databases
 * Domain-aware interfaces that hide low-level vector operations
 */

import type { Memory, MemoryId, MemoryType, Timestamp } from './memory.js';
import type { SearchQuery, SearchResult, VectorSearchQuery, VectorSearchResult } from './search.js';

/**
 * Configuration for vector store connection
 */
export interface VectorStoreConfig {
  // Connection settings
  host?: string;
  port?: number;
  apiKey?: string;

  // Collection/database settings
  collectionName?: string;

  // Embedding settings
  embeddingDimension?: number;
  embeddingModel?: string;

  // Performance settings
  batchSize?: number;
  maxRetries?: number;
  timeoutMs?: number;
}

/**
 * Vector embedding with metadata
 */
export interface VectorDocument {
  id: MemoryId;
  embedding: number[];
  content: string;
  metadata: Record<string, unknown>;
}

/**
 * Batch operation result
 */
export interface BatchOperationResult {
  successful: MemoryId[];
  failed: Array<{ id: MemoryId; error: string }>;
  totalProcessed: number;
}

/**
 * Low-level vector store interface
 * Direct access to vector operations for advanced use cases
 */
export interface VectorStore {
  // Connection management
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  isConnected(): boolean;

  // Collection management
  createCollection(name: string, dimension: number): Promise<void>;
  deleteCollection(name: string): Promise<void>;
  listCollections(): Promise<string[]>;

  // Document operations
  add(document: VectorDocument): Promise<void>;
  addBatch(documents: VectorDocument[]): Promise<BatchOperationResult>;

  update(id: MemoryId, document: Partial<VectorDocument>): Promise<void>;

  delete(id: MemoryId): Promise<void>;
  deleteBatch(ids: MemoryId[]): Promise<BatchOperationResult>;

  get(id: MemoryId): Promise<VectorDocument | null>;
  getBatch(ids: MemoryId[]): Promise<(VectorDocument | null)[]>;

  // Vector search
  search(query: VectorSearchQuery): Promise<VectorSearchResult[]>;
  searchSimilar(id: MemoryId, limit?: number): Promise<VectorSearchResult[]>;

  // Utility operations
  count(): Promise<number>;
  clear(): Promise<void>;
}

/**
 * High-level memory store interface
 * Domain-aware operations that work with Memory objects
 */
export interface MemoryStore {
  // Memory operations
  storeMemory(memory: Memory): Promise<void>;
  storeMemories(memories: Memory[]): Promise<BatchOperationResult>;

  getMemory(id: MemoryId): Promise<Memory | null>;
  getMemories(ids: MemoryId[]): Promise<(Memory | null)[]>;

  updateMemory(id: MemoryId, updates: Partial<Memory>): Promise<void>;

  deleteMemory(id: MemoryId): Promise<void>;
  deleteMemories(ids: MemoryId[]): Promise<BatchOperationResult>;

  // Search operations
  searchMemories(query: SearchQuery): Promise<SearchResult<Memory>[]>;
  findSimilarMemories(memoryId: MemoryId, limit?: number): Promise<SearchResult<Memory>[]>;

  // Type-specific queries
  getMemoriesByType(type: MemoryType, limit?: number): Promise<Memory[]>;
  getRecentMemories(limit?: number, maxAge?: number): Promise<Memory[]>;

  // Analytics and maintenance
  getMemoryCount(): Promise<number>;
  getMemoryCountByType(): Promise<Record<MemoryType, number>>;
  getStorageStats(): Promise<MemoryStoreStats>;

  // Maintenance operations
  optimizeStorage(): Promise<void>;
  rebuildIndex(): Promise<void>;
}

/**
 * Memory store statistics
 */
export interface MemoryStoreStats {
  totalMemories: number;
  memoriesByType: Record<MemoryType, number>;
  oldestMemory: Timestamp;
  newestMemory: Timestamp;
  averageMemorySize: number;
  totalStorageSize: number;
  indexHealth: 'good' | 'degraded' | 'poor';
}

/**
 * Embedding service interface
 * Abstracts different embedding models and providers
 */
export interface EmbeddingService {
  // Generate embeddings
  generateEmbedding(text: string): Promise<number[]>;
  generateEmbeddings(texts: string[]): Promise<number[][]>;

  // Service info
  getDimension(): number;
  getModelName(): string;
  getMaxTokens(): number;

  // Batch processing
  estimateTokens(text: string): number;
  canProcessBatch(texts: string[]): boolean;
}

/**
 * Memory processing pipeline interface
 * Handles the flow from raw content to stored memories
 */
export interface MemoryProcessor {
  // Process single memory
  processMemory(memory: Omit<Memory, 'embedding'>): Promise<Memory>;

  // Process batch of memories
  processMemories(memories: Omit<Memory, 'embedding'>[]): Promise<Memory[]>;

  // Reprocess existing memories (e.g., with new embedding model)
  reprocessMemory(memoryId: MemoryId): Promise<void>;
  reprocessAllMemories(batchSize?: number): Promise<void>;
}

/**
 * Vector store factory interface
 * For creating different types of vector stores
 */
export interface VectorStoreFactory {
  createVectorStore(config: VectorStoreConfig): Promise<VectorStore>;
  createMemoryStore(vectorStore: VectorStore, embeddingService: EmbeddingService): MemoryStore;
  createEmbeddingService(modelName?: string): Promise<EmbeddingService>;
}

/**
 * Health check result for monitoring
 */
export interface HealthCheckResult {
  status: 'healthy' | 'degraded' | 'unhealthy';
  checks: {
    connection: boolean;
    indexHealth: boolean;
    recentWrites: boolean;
    recentReads: boolean;
  };
  metrics: {
    responseTime: number;
    errorRate: number;
    memoryCount: number;
  };
  timestamp: Timestamp;
}

/**
 * Monitoring interface for observability
 */
export interface VectorStoreMonitor {
  healthCheck(): Promise<HealthCheckResult>;
  getMetrics(): Promise<Record<string, number>>;
  getRecentErrors(): Promise<Array<{ timestamp: Timestamp; error: string }>>;
}
