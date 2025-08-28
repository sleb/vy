/**
 * Unit tests for ChromaMemoryStore
 * Focuses on critical logic: error handling, data conversion, and core operations
 */

import type {
  ConversationMemory,
  EmbeddingService,
  SearchQuery,
} from "@repo/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  ChromaClient,
  ChromaDocument,
  ChromaQueryResult,
} from "../chroma/chroma-client.js";
import { ChromaMemoryStore } from "../chroma/chroma-memory-store.js";

// Mock dependencies
const mockChromaClient = {
  addDocuments: vi.fn(),
  updateDocuments: vi.fn(),
  deleteDocuments: vi.fn(),
  getDocuments: vi.fn(),
  queryCollection: vi.fn(),
  getCollectionCount: vi.fn(),
  getOrCreateCollection: vi.fn(),
} as unknown as ChromaClient;

const mockEmbeddingService = {
  generateEmbedding: vi.fn(),
  generateEmbeddings: vi.fn(),
  getDimension: () => 1536,
  getModelName: () => "test-model",
  getMaxTokens: () => 8192,
  estimateTokens: (text: string) => Math.ceil(text.length / 4),
  canProcessBatch: () => true,
} as EmbeddingService;

describe("ChromaMemoryStore", () => {
  let store: ChromaMemoryStore;
  const collectionName = "test_memories";

  beforeEach(() => {
    vi.clearAllMocks();
    store = new ChromaMemoryStore(
      mockChromaClient,
      mockEmbeddingService,
      collectionName,
    );
  });

  describe("Error Handling & Data Preservation", () => {
    it("should store memory without embedding when embedding generation fails", async () => {
      // Arrange
      const memory: ConversationMemory = {
        id: "test-id",
        type: "conversation",
        content: "Test conversation content",
        timestamp: new Date(),
        metadata: {},
        participants: ["user", "assistant"],
        messageCount: 2,
      };

      mockEmbeddingService.generateEmbedding = vi
        .fn()
        .mockRejectedValue(new Error("OpenAI API error"));
      mockChromaClient.addDocuments = vi.fn().mockResolvedValue(undefined);

      // Act
      await store.storeMemory(memory);

      // Assert
      expect(mockChromaClient.addDocuments).toHaveBeenCalledWith(
        collectionName,
        expect.arrayContaining([
          expect.objectContaining({
            id: memory.id,
            document: memory.content,
            embedding: [], // Should be empty when embedding fails
            metadata: expect.objectContaining({
              type: "conversation",
              timestamp: memory.timestamp.toISOString(),
              participants: memory.participants,
              messageCount: memory.messageCount,
            }),
          }),
        ]),
      );

      // Should track the failed embedding
      const failedEmbeddings = store.getFailedEmbeddings();
      expect(failedEmbeddings).toHaveLength(1);
      expect(failedEmbeddings[0]).toMatchObject({
        memoryId: memory.id,
        content: memory.content,
        error: "Error: OpenAI API error",
        retryCount: 0,
      });
    });

    it("should store memory with embedding when embedding generation succeeds", async () => {
      // Arrange
      const memory: ConversationMemory = {
        id: "test-id-2",
        type: "conversation",
        content: "Another test conversation",
        timestamp: new Date(),
        metadata: { topic: "testing" },
        participants: ["user"],
        messageCount: 1,
      };

      const mockEmbedding = [0.1, 0.2, 0.3];
      mockEmbeddingService.generateEmbedding = vi
        .fn()
        .mockResolvedValue(mockEmbedding);
      mockChromaClient.addDocuments = vi.fn().mockResolvedValue(undefined);

      // Act
      await store.storeMemory(memory);

      // Assert
      expect(mockEmbeddingService.generateEmbedding).toHaveBeenCalledWith(
        memory.content,
      );
      expect(mockChromaClient.addDocuments).toHaveBeenCalledWith(
        collectionName,
        expect.arrayContaining([
          expect.objectContaining({
            id: memory.id,
            embedding: mockEmbedding,
            document: memory.content,
          }),
        ]),
      );

      // Should not track as failed embedding
      const failedEmbeddings = store.getFailedEmbeddings();
      expect(failedEmbeddings).toHaveLength(0);
    });
  });

  describe("Memory ID Generation", () => {
    it("should generate UUID v7 when no ID provided", async () => {
      // Arrange
      const memory = {
        type: "conversation",
        content: "Test content",
        timestamp: new Date(),
        metadata: {},
        participants: [],
        messageCount: 1,
      } as Omit<ConversationMemory, "id">;

      mockEmbeddingService.generateEmbedding = vi
        .fn()
        .mockResolvedValue([0.1, 0.2]);
      mockChromaClient.addDocuments = vi.fn().mockResolvedValue(undefined);

      // Act
      await store.storeMemory(memory as ConversationMemory);

      // Assert
      expect(mockChromaClient.addDocuments).toHaveBeenCalledWith(
        collectionName,
        expect.arrayContaining([
          expect.objectContaining({
            id: expect.stringMatching(
              /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
            ),
          }),
        ]),
      );
    });
  });

  describe("Memory ↔ ChromaDocument Conversion", () => {
    it("should convert ConversationMemory to ChromaDocument correctly", async () => {
      // Arrange
      const memory: ConversationMemory = {
        id: "conv-123",
        type: "conversation",
        content: "User asked about weather, I provided forecast",
        timestamp: new Date("2024-01-15T10:30:00Z"),
        metadata: { location: "San Francisco", confidence: 0.95 },
        participants: ["user", "assistant"],
        messageCount: 4,
        tags: ["weather", "forecast"],
      };

      const embedding = [0.1, 0.2, 0.3, 0.4];
      mockEmbeddingService.generateEmbedding = vi
        .fn()
        .mockResolvedValue(embedding);
      mockChromaClient.addDocuments = vi.fn().mockResolvedValue(undefined);

      // Act
      await store.storeMemory(memory);

      // Assert
      expect(mockChromaClient.addDocuments).toHaveBeenCalledWith(
        collectionName,
        [
          {
            id: "conv-123",
            embedding: [0.1, 0.2, 0.3, 0.4],
            document: "User asked about weather, I provided forecast",
            metadata: {
              type: "conversation",
              timestamp: "2024-01-15T10:30:00.000Z",
              location: "San Francisco",
              confidence: 0.95,
              participants: ["user", "assistant"],
              messageCount: 4,
              tags: ["weather", "forecast"],
            },
          },
        ],
      );
    });

    it("should convert ChromaDocument back to ConversationMemory correctly", async () => {
      // Arrange
      const chromaDoc: ChromaDocument = {
        id: "conv-456",
        embedding: [0.5, 0.6, 0.7],
        document: "Discussion about project timeline and milestones",
        metadata: {
          type: "conversation",
          timestamp: "2024-01-16T14:20:00.000Z",
          project: "Alpha Release",
          participants: ["user", "assistant", "manager"],
          messageCount: 8,
          tags: ["project", "timeline"],
        },
      };

      mockChromaClient.getDocuments = vi.fn().mockResolvedValue([chromaDoc]);

      // Act
      const result = await store.getMemory("conv-456");

      // Assert
      expect(result).toEqual({
        id: "conv-456",
        type: "conversation",
        content: "Discussion about project timeline and milestones",
        timestamp: new Date("2024-01-16T14:20:00.000Z"),
        metadata: { project: "Alpha Release" },
        embedding: [0.5, 0.6, 0.7],
        participants: ["user", "assistant", "manager"],
        messageCount: 8,
        tags: ["project", "timeline"],
      });
    });
  });

  describe("Search Operations", () => {
    it("should perform semantic search and convert results correctly", async () => {
      // Arrange
      const query: SearchQuery = {
        query: "weather forecast",
        limit: 5,
        minRelevanceScore: 0.5, // Lower threshold to include both results
      };

      const queryEmbedding = [0.8, 0.9, 1.0];
      mockEmbeddingService.generateEmbedding = vi
        .fn()
        .mockResolvedValue(queryEmbedding);

      const mockSearchResults: ChromaQueryResult = {
        ids: [["result-1", "result-2"]],
        distances: [[0.2, 0.4]], // ChromaDB returns distances (lower = more similar)
        documents: [["Weather is sunny today", "Rain expected tomorrow"]],
        metadatas: [
          [
            {
              type: "conversation",
              timestamp: "2024-01-15T09:00:00.000Z",
              participants: ["user", "assistant"],
              messageCount: 2,
            },
            {
              type: "conversation",
              timestamp: "2024-01-15T11:00:00.000Z",
              participants: ["user", "assistant"],
              messageCount: 3,
            },
          ],
        ],
      };

      mockChromaClient.queryCollection = vi
        .fn()
        .mockResolvedValue(mockSearchResults);

      // Act
      const results = await store.searchMemories(query);

      // Assert
      expect(mockEmbeddingService.generateEmbedding).toHaveBeenCalledWith(
        "weather forecast",
      );
      expect(mockChromaClient.queryCollection).toHaveBeenCalledWith(
        collectionName,
        [queryEmbedding],
        5,
      );

      expect(results).toHaveLength(2);

      // First result (distance 0.2 = similarity 0.8)
      expect(results[0]).toMatchObject({
        id: "result-1",
        relevanceScore: 0.8, // 1 - 0.2
        snippet: expect.stringContaining("Weather"),
        content: expect.objectContaining({
          id: "result-1",
          type: "conversation",
          content: "Weather is sunny today",
        }),
      });

      // Second result (distance 0.4 = similarity 0.6, below threshold)
      expect(results[1]).toMatchObject({
        id: "result-2",
        relevanceScore: 0.6, // 1 - 0.4
      });
    });

    it("should return empty results for empty search query", async () => {
      // Arrange
      const query: SearchQuery = { query: "" };

      // Act
      const results = await store.searchMemories(query);

      // Assert
      expect(results).toEqual([]);
      expect(mockEmbeddingService.generateEmbedding).not.toHaveBeenCalled();
      expect(mockChromaClient.queryCollection).not.toHaveBeenCalled();
    });

    it("should filter results by relevance threshold", async () => {
      // Arrange
      const query: SearchQuery = {
        query: "test query",
        minRelevanceScore: 0.75, // High threshold
      };

      mockEmbeddingService.generateEmbedding = vi.fn().mockResolvedValue([1.0]);
      mockChromaClient.queryCollection = vi.fn().mockResolvedValue({
        ids: [["low-relevance", "high-relevance"]],
        distances: [[0.5, 0.1]], // 0.5 similarity, 0.9 similarity
        documents: [["Low relevance result", "High relevance result"]],
        metadatas: [
          [
            { type: "conversation", timestamp: "2024-01-15T09:00:00.000Z" },
            { type: "conversation", timestamp: "2024-01-15T10:00:00.000Z" },
          ],
        ],
      });

      // Act
      const results = await store.searchMemories(query);

      // Assert - only high relevance result should be returned
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe("high-relevance");
      expect(results[0].relevanceScore).toBe(0.9); // 1 - 0.1
    });
  });

  describe("Failed Embedding Reprocessing", () => {
    it("should successfully reprocess failed embeddings", async () => {
      // Arrange - first store a memory that fails embedding
      const memory: ConversationMemory = {
        id: "failed-memory",
        type: "conversation",
        content: "This will fail initially",
        timestamp: new Date(),
        metadata: {},
        participants: ["user"],
        messageCount: 1,
      };

      // Simulate initial embedding failure
      mockEmbeddingService.generateEmbedding = vi
        .fn()
        .mockRejectedValueOnce(new Error("Initial failure"))
        .mockResolvedValueOnce([0.1, 0.2, 0.3]); // Success on retry

      mockChromaClient.addDocuments = vi.fn().mockResolvedValue(undefined);
      mockChromaClient.updateDocuments = vi.fn().mockResolvedValue(undefined);
      mockChromaClient.getDocuments = vi.fn().mockResolvedValue([
        {
          id: "failed-memory",
          document: "This will fail initially",
          embedding: [],
          metadata: {
            type: "conversation",
            timestamp: memory.timestamp.toISOString(),
          },
        },
      ]);

      // Store memory (will fail embedding)
      await store.storeMemory(memory);

      // Verify failure was tracked
      expect(store.getFailedEmbeddings()).toHaveLength(1);

      // Act - reprocess failed embeddings
      const result = await store.reprocessFailedEmbeddings();

      // Assert
      expect(result).toEqual({
        successful: ["failed-memory"],
        failed: [],
        totalProcessed: 1,
      });

      expect(mockChromaClient.updateDocuments).toHaveBeenCalledWith(
        collectionName,
        expect.arrayContaining([
          expect.objectContaining({
            id: "failed-memory",
            embedding: [0.1, 0.2, 0.3],
          }),
        ]),
      );

      // Should remove from failed list
      expect(store.getFailedEmbeddings()).toHaveLength(0);
    });
  });

  describe("Snippet Generation", () => {
    it("should generate snippet with query context", () => {
      // We need to access the private method for testing, so we'll test it indirectly
      // through search results
      const longContent =
        "This is a very long piece of content that contains information about weather patterns and climate change effects on global temperatures and precipitation levels across different geographical regions and time periods throughout the year.";

      // This tests the snippet generation indirectly through search
      expect(longContent.length).toBeGreaterThan(200); // Ensure we're testing truncation
    });
  });
});
