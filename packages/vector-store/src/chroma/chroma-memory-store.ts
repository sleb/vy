/**
 * ChromaMemoryStore - High-level memory store implementation
 *
 * Orchestrates ChromaDB operations, embedding generation, and domain object conversion
 * for the Vy semantic memory system. Handles failures gracefully by preserving raw data.
 */

import type {
    BatchOperationResult,
    EmbeddingService,
    Memory,
    MemoryId,
    MemoryStore,
    MemoryStoreStats,
    MemoryType,
    SearchQuery,
    SearchResult,
} from '@repo/core';
import { v7 as uuidv7 } from 'uuid';

import type { ChromaClient, ChromaDocument } from './chroma-client.js';

/**
 * Failed embedding record for reprocessing
 */
interface FailedEmbedding {
  memoryId: MemoryId;
  content: string;
  timestamp: Date;
  error: string;
  retryCount: number;
}

/**
 * ChromaDB-based implementation of MemoryStore
 */
export class ChromaMemoryStore implements MemoryStore {
  private readonly chromaClient: ChromaClient;
  private readonly embeddingService: EmbeddingService;
  private readonly collectionName: string;
  private readonly failedEmbeddings: Map<MemoryId, FailedEmbedding> = new Map();

  constructor(
    chromaClient: ChromaClient,
    embeddingService: EmbeddingService,
    collectionName: string,
  ) {
    this.chromaClient = chromaClient;
    this.embeddingService = embeddingService;
    this.collectionName = collectionName;
  }

  /**
   * Store a memory with embedding generation
   * Preserves raw data even if embedding fails
   */
  async storeMemory(memory: Memory): Promise<void> {
    const memoryId = memory.id || this.generateMemoryId();
    const memoryWithId = { ...memory, id: memoryId };

    try {
      // Generate embedding for the content
      const embedding = await this.embeddingService.generateEmbedding(memory.content);

      // Convert to ChromaDB document format
      const document = this.memoryToDocument(memoryWithId, embedding);

      // Store in ChromaDB with embedding
      await this.chromaClient.addDocuments(this.collectionName, [document]);

      // Remove from failed embeddings if it was previously failed
      this.failedEmbeddings.delete(memoryId);

    } catch (embeddingError) {
      // Store memory without embedding so we don't lose the data
      const document = this.memoryToDocument(memoryWithId, []);
      await this.chromaClient.addDocuments(this.collectionName, [document]);

      // Track failed embedding for later reprocessing
      this.failedEmbeddings.set(memoryId, {
        memoryId,
        content: memory.content,
        timestamp: new Date(),
        error: String(embeddingError),
        retryCount: 0,
      });

      console.warn(`Failed to generate embedding for memory ${memoryId}, stored without embedding:`, embeddingError);
    }
  }

  /**
   * Store multiple memories in batch
   */
  async storeMemories(memories: Memory[]): Promise<BatchOperationResult> {
    const results: BatchOperationResult = {
      successful: [],
      failed: [],
      totalProcessed: memories.length,
    };

    // Process each memory individually for now (simple approach)
    for (const memory of memories) {
      try {
        await this.storeMemory(memory);
        results.successful.push(memory.id || 'unknown');
      } catch (error) {
        results.failed.push({
          id: memory.id || 'unknown',
          error: String(error),
        });
      }
    }

    return results;
  }

  /**
   * Retrieve a memory by ID
   */
  async getMemory(id: MemoryId): Promise<Memory | null> {
    try {
      const documents = await this.chromaClient.getDocuments(this.collectionName, [id]);
      return documents.length > 0 ? this.documentToMemory(documents[0]) : null;
    } catch (error) {
      throw new Error(`Failed to get memory ${id}: ${error}`);
    }
  }

  /**
   * Retrieve multiple memories by IDs
   */
  async getMemories(ids: MemoryId[]): Promise<(Memory | null)[]> {
    if (ids.length === 0) return [];

    try {
      const documents = await this.chromaClient.getDocuments(this.collectionName, ids);

      // Ensure results are in the same order as requested IDs
      const documentMap = new Map(documents.map(doc => [doc.id, doc]));
      return ids.map(id => {
        const doc = documentMap.get(id);
        return doc ? this.documentToMemory(doc) : null;
      });
    } catch (error) {
      throw new Error(`Failed to get memories: ${error}`);
    }
  }

  /**
   * Update an existing memory
   */
  async updateMemory(id: MemoryId, updates: Partial<Memory>): Promise<void> {
    const existing = await this.getMemory(id);
    if (!existing) {
      throw new Error(`Memory ${id} not found`);
    }

    const updated = { ...existing, ...updates, id };

    try {
      // Regenerate embedding if content changed
      let embedding = existing.embedding || [];
      if (updates.content && updates.content !== existing.content) {
        try {
          embedding = await this.embeddingService.generateEmbedding(updates.content);
          this.failedEmbeddings.delete(id); // Remove from failed list if successful
        } catch (embeddingError) {
          // Keep old embedding, track failure
          this.failedEmbeddings.set(id, {
            memoryId: id,
            content: updates.content,
            timestamp: new Date(),
            error: String(embeddingError),
            retryCount: (this.failedEmbeddings.get(id)?.retryCount || 0) + 1,
          });
          console.warn(`Failed to update embedding for memory ${id}:`, embeddingError);
        }
      }

      const document = this.memoryToDocument(updated, embedding);
      await this.chromaClient.updateDocuments(this.collectionName, [document]);
    } catch (error) {
      throw new Error(`Failed to update memory ${id}: ${error}`);
    }
  }

  /**
   * Delete a memory
   */
  async deleteMemory(id: MemoryId): Promise<void> {
    try {
      await this.chromaClient.deleteDocuments(this.collectionName, [id]);
      this.failedEmbeddings.delete(id);
    } catch (error) {
      throw new Error(`Failed to delete memory ${id}: ${error}`);
    }
  }

  /**
   * Delete multiple memories
   */
  async deleteMemories(ids: MemoryId[]): Promise<BatchOperationResult> {
    if (ids.length === 0) {
      return { successful: [], failed: [], totalProcessed: 0 };
    }

    try {
      await this.chromaClient.deleteDocuments(this.collectionName, ids);

      // Clean up failed embeddings
      ids.forEach(id => this.failedEmbeddings.delete(id));

      return {
        successful: ids,
        failed: [],
        totalProcessed: ids.length,
      };
    } catch (error) {
      return {
        successful: [],
        failed: ids.map(id => ({ id, error: String(error) })),
        totalProcessed: ids.length,
      };
    }
  }

  /**
   * Search memories with semantic similarity
   * Starts simple - just semantic search for now
   */
  async searchMemories(query: SearchQuery): Promise<SearchResult<Memory>[]> {
    if (!query.query || !query.query.trim()) {
      return []; // Require semantic query for now
    }

    try {
      // Generate embedding for search query
      const queryEmbedding = await this.embeddingService.generateEmbedding(query.query);

      // Perform vector similarity search
      const results = await this.chromaClient.queryCollection(
        this.collectionName,
        [queryEmbedding],
        query.limit || 10,
      );

      // Convert ChromaDB results to SearchResult<Memory>
      const searchResults: SearchResult<Memory>[] = [];

      if (results.ids.length > 0) {
        for (let i = 0; i < results.ids[0].length; i++) {
          const id = results.ids[0][i];
          const distance = results.distances?.[0]?.[i] || 1.0;
          const metadata = results.metadatas?.[0]?.[i] || {};
          const document = results.documents?.[0]?.[i] || '';

          // Convert distance to similarity score (0-1, higher = more similar)
          const similarity = Math.max(0, 1 - distance);

          // Apply relevance threshold if specified
          if (query.minRelevanceScore && similarity < query.minRelevanceScore) {
            continue;
          }

          // Reconstruct memory from ChromaDB data
          const memory = this.reconstructMemoryFromSearch(id, document, metadata);

          searchResults.push({
            id,
            content: memory,
            relevanceScore: similarity,
            snippet: this.generateSnippet(document, query.query),
          });
        }
      }

      return searchResults;
    } catch (error) {
      throw new Error(`Search failed: ${error}`);
    }
  }

  /**
   * Find similar memories to a given memory
   */
  async findSimilarMemories(memoryId: MemoryId, limit?: number): Promise<SearchResult<Memory>[]> {
    const memory = await this.getMemory(memoryId);
    if (!memory) {
      throw new Error(`Memory ${memoryId} not found`);
    }

    return this.searchMemories({
      query: memory.content,
      limit: limit || 10,
    });
  }

  /**
   * Get memories by type (simple metadata filtering)
   */
  async getMemoriesByType(type: MemoryType, limit?: number): Promise<Memory[]> {
    // For now, get all documents and filter - can optimize later
    try {
      const results = await this.chromaClient.queryCollection(
        this.collectionName,
        [[]], // Empty embedding for metadata-only search
        limit || 50,
        { type }, // Metadata filter
      );

      const memories: Memory[] = [];
      if (results.ids.length > 0) {
        for (let i = 0; i < results.ids[0].length; i++) {
          const id = results.ids[0][i];
          const document = results.documents?.[0]?.[i] || '';
          const metadata = results.metadatas?.[0]?.[i] || {};

          memories.push(this.reconstructMemoryFromSearch(id, document, metadata));
        }
      }

      return memories;
    } catch (error) {
      throw new Error(`Failed to get memories by type ${type}: ${error}`);
    }
  }

  /**
   * Get recent memories (sorted by timestamp)
   */
  async getRecentMemories(limit?: number, maxAge?: number): Promise<Memory[]> {
    // Simple implementation - get all and sort by timestamp
    // Can optimize with time-based filtering later
    const memories = await this.getMemoriesByType('conversation', limit || 50);

    // Filter by max age if specified
    if (maxAge) {
      const cutoff = new Date(Date.now() - maxAge);
      return memories
        .filter(memory => memory.timestamp >= cutoff)
        .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
        .slice(0, limit || 50);
    }

    return memories
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(0, limit || 50);
  }

  /**
   * Get total memory count
   */
  async getMemoryCount(): Promise<number> {
    try {
      return await this.chromaClient.getCollectionCount(this.collectionName);
    } catch (error) {
      throw new Error(`Failed to get memory count: ${error}`);
    }
  }

  /**
   * Get memory count by type
   */
  async getMemoryCountByType(): Promise<Record<MemoryType, number>> {
    // Simple implementation - can optimize later
    const counts: Record<MemoryType, number> = {
      conversation: 0,
      insight: 0,
      learning: 0,
      fact: 0,
      action_item: 0,
    };

    for (const type of Object.keys(counts) as MemoryType[]) {
      const memories = await this.getMemoriesByType(type, 1000); // Limit for performance
      counts[type] = memories.length;
    }

    return counts;
  }

  /**
   * Get storage statistics
   */
  async getStorageStats(): Promise<MemoryStoreStats> {
    const totalMemories = await this.getMemoryCount();
    const memoriesByType = await this.getMemoryCountByType();
    const recentMemories = await this.getRecentMemories(1);
    const oldestMemories = await this.getRecentMemories(1000);

    return {
      totalMemories,
      memoriesByType,
      oldestMemory: oldestMemories.length > 0 ?
        oldestMemories[oldestMemories.length - 1].timestamp : new Date(),
      newestMemory: recentMemories.length > 0 ?
        recentMemories[0].timestamp : new Date(),
      averageMemorySize: 0, // TODO: Calculate from actual data
      totalStorageSize: 0, // TODO: Get from ChromaDB if available
      indexHealth: 'good' as const, // TODO: Implement health check
    };
  }

  /**
   * Reprocess failed embeddings
   */
  async reprocessFailedEmbeddings(maxRetries: number = 3): Promise<BatchOperationResult> {
    const results: BatchOperationResult = {
      successful: [],
      failed: [],
      totalProcessed: this.failedEmbeddings.size,
    };

    for (const [memoryId, failedEmbedding] of this.failedEmbeddings) {
      if (failedEmbedding.retryCount >= maxRetries) {
        results.failed.push({
          id: memoryId,
          error: `Max retries (${maxRetries}) exceeded`,
        });
        continue;
      }

      try {
        // Try to generate embedding again
        const embedding = await this.embeddingService.generateEmbedding(failedEmbedding.content);

        // Update the memory with the new embedding
        const memory = await this.getMemory(memoryId);
        if (memory) {
          const document = this.memoryToDocument(memory, embedding);
          await this.chromaClient.updateDocuments(this.collectionName, [document]);

          this.failedEmbeddings.delete(memoryId);
          results.successful.push(memoryId);
        }
      } catch (error) {
        // Increment retry count
        failedEmbedding.retryCount++;
        failedEmbedding.error = String(error);

        results.failed.push({
          id: memoryId,
          error: String(error),
        });
      }
    }

    return results;
  }

  /**
   * Get list of memories with failed embeddings
   */
  getFailedEmbeddings(): FailedEmbedding[] {
    return Array.from(this.failedEmbeddings.values());
  }

  /**
   * Optimize storage (placeholder for future implementation)
   */
  async optimizeStorage(): Promise<void> {
    // TODO: Implement storage optimization
    console.log('Storage optimization not yet implemented');
  }

  /**
   * Rebuild index (placeholder for future implementation)
   */
  async rebuildIndex(): Promise<void> {
    // TODO: Implement index rebuilding
    console.log('Index rebuilding not yet implemented');
  }

  /**
   * Generate unique memory ID using UUID v7
   */
  private generateMemoryId(): MemoryId {
    return uuidv7();
  }

  /**
   * Convert Memory to ChromaDB document format
   */
  private memoryToDocument(memory: Memory, embedding: number[]): ChromaDocument {
    return {
      id: memory.id,
      embedding,
      document: memory.content,
      metadata: {
        type: memory.type,
        timestamp: memory.timestamp.toISOString(),
        ...memory.metadata,
        // Add type-specific metadata
        ...(memory.type === 'conversation' && {
          participants: (memory as any).participants || [],
          messageCount: (memory as any).messageCount || 0,
          tags: (memory as any).tags || [],
        }),
      },
    };
  }

  /**
   * Convert ChromaDB document to Memory object
   */
  private documentToMemory(document: ChromaDocument): Memory {
    const { type, timestamp, participants, messageCount, tags, ...restMetadata } = document.metadata;

    const baseMemory = {
      id: document.id,
      type: type as MemoryType,
      content: document.document,
      timestamp: new Date(timestamp as string),
      metadata: restMetadata,
      embedding: document.embedding.length > 0 ? document.embedding : undefined,
    };

    // Add type-specific fields
    if (type === 'conversation') {
      return {
        ...baseMemory,
        participants: participants as string[] || [],
        messageCount: messageCount as number || 0,
        tags: tags as string[] || [],
      } as any; // TODO: Improve typing
    }

    return baseMemory as Memory;
  }

  /**
   * Reconstruct memory from search results (may have incomplete data)
   */
  private reconstructMemoryFromSearch(
    id: string,
    document: string,
    metadata: Record<string, unknown>,
  ): Memory {
    const { type, timestamp, ...restMetadata } = metadata;

    return {
      id,
      type: (type as MemoryType) || 'conversation',
      content: document,
      timestamp: timestamp ? new Date(timestamp as string) : new Date(),
      metadata: restMetadata,
    } as Memory;
  }

  /**
   * Generate a snippet from content for search results
   */
  private generateSnippet(content: string, query: string): string {
    const maxLength = 200;
    if (content.length <= maxLength) {
      return content;
    }

    // Simple snippet generation - find query terms and show context
    const queryWords = query.toLowerCase().split(/\s+/);
    const contentLower = content.toLowerCase();

    // Find first occurrence of any query word
    let bestStart = 0;
    for (const word of queryWords) {
      const index = contentLower.indexOf(word);
      if (index !== -1) {
        bestStart = Math.max(0, index - 50);
        break;
      }
    }

    const snippet = content.substring(bestStart, bestStart + maxLength);
    return bestStart > 0 ? '...' + snippet + '...' : snippet + '...';
  }
}

/**
 * Factory function to create ChromaMemoryStore
 */
export async function createChromaMemoryStore(
  chromaClient: ChromaClient,
  embeddingService: EmbeddingService,
  collectionName: string,
): Promise<ChromaMemoryStore> {
  // Ensure collection exists
  await chromaClient.getOrCreateCollection(collectionName);

  return new ChromaMemoryStore(chromaClient, embeddingService, collectionName);
}
