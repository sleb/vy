/**
 * ChromaDB client wrapper for Vy vector store
 *
 * Provides a clean abstraction over ChromaDB with connection management,
 * retry logic, and error handling for reliable vector operations
 */

import type {
  ChromaConfig,
  HostedChromaConfig,
  LocalChromaConfig,
} from "../config.js";

/**
 * ChromaDB collection interface
 */
export interface ChromaCollection {
  name: string;
  id: string;
  metadata?: Record<string, unknown>;
}

/**
 * ChromaDB document for storage
 */
export interface ChromaDocument {
  id: string;
  embedding: number[];
  metadata: Record<string, unknown>;
  document: string;
}

/**
 * ChromaDB query result
 */
export interface ChromaQueryResult {
  ids: string[][];
  distances: number[][];
  metadatas: (Record<string, unknown> | null)[][];
  documents: (string | null)[][];
}

/**
 * ChromaDB client wrapper
 */
export class ChromaClient {
  private readonly config: ChromaConfig;
  private client: any; // ChromaDB client instance
  private connected = false;

  constructor(config: ChromaConfig) {
    this.config = config;
  }

  /**
   * Connect to ChromaDB instance
   */
  async connect(): Promise<void> {
    if (this.connected) {
      return;
    }

    try {
      // Dynamic import of ChromaDB client (it's a heavy dependency)
      const { ChromaClient } = await import("chromadb");

      // Determine if this is hosted or local config
      const isHosted = "apiKey" in this.config && "ssl" in this.config;

      const clientOptions: any = {
        path: `${isHosted ? "https" : "http"}://${this.config.host}:${this.config.port}`,
      };

      if (isHosted && "apiKey" in this.config) {
        clientOptions.auth = {
          provider: "token",
          credentials: this.config.apiKey,
        };
      }

      this.client = new ChromaClient(clientOptions);

      // Test connection
      await this.client.heartbeat();

      this.connected = true;
    } catch (error) {
      throw new Error(`Failed to connect to ChromaDB: ${error}`);
    }
  }

  /**
   * Disconnect from ChromaDB
   */
  async disconnect(): Promise<void> {
    // ChromaDB client doesn't have explicit disconnect
    this.client = null;
    this.connected = false;
  }

  /**
   * Check if client is connected
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Create a new collection
   */
  async createCollection(
    name: string,
    metadata?: Record<string, unknown>,
  ): Promise<ChromaCollection> {
    this.ensureConnected();

    try {
      const collection = await this.client.createCollection({
        name,
        metadata: metadata || {},
      });

      return {
        name: collection.name,
        id: collection.id,
        metadata: collection.metadata,
      };
    } catch (error) {
      throw new Error(`Failed to create collection '${name}': ${error}`);
    }
  }

  /**
   * Get or create a collection (idempotent)
   */
  async getOrCreateCollection(
    name: string,
    metadata?: Record<string, unknown>,
  ): Promise<ChromaCollection> {
    this.ensureConnected();

    try {
      const collection = await this.client.getOrCreateCollection({
        name,
        metadata: metadata || {},
      });

      return {
        name: collection.name,
        id: collection.id,
        metadata: collection.metadata,
      };
    } catch (error) {
      throw new Error(`Failed to get or create collection '${name}': ${error}`);
    }
  }

  /**
   * Delete a collection
   */
  async deleteCollection(name: string): Promise<void> {
    this.ensureConnected();

    try {
      await this.client.deleteCollection({ name });
    } catch (error) {
      throw new Error(`Failed to delete collection '${name}': ${error}`);
    }
  }

  /**
   * List all collections
   */
  async listCollections(): Promise<ChromaCollection[]> {
    this.ensureConnected();

    try {
      const collections = await this.client.listCollections();
      return collections.map((col: any) => ({
        name: col.name,
        id: col.id,
        metadata: col.metadata,
      }));
    } catch (error) {
      throw new Error(`Failed to list collections: ${error}`);
    }
  }

  /**
   * Add documents to a collection
   */
  async addDocuments(
    collectionName: string,
    documents: ChromaDocument[],
  ): Promise<void> {
    this.ensureConnected();

    if (documents.length === 0) {
      return;
    }

    try {
      const collection = await this.client.getCollection({
        name: collectionName,
      });

      await collection.add({
        ids: documents.map((doc) => doc.id),
        embeddings: documents.map((doc) => doc.embedding),
        metadatas: documents.map((doc) => doc.metadata),
        documents: documents.map((doc) => doc.document),
      });
    } catch (error) {
      throw new Error(
        `Failed to add documents to collection '${collectionName}': ${error}`,
      );
    }
  }

  /**
   * Update documents in a collection
   */
  async updateDocuments(
    collectionName: string,
    documents: ChromaDocument[],
  ): Promise<void> {
    this.ensureConnected();

    if (documents.length === 0) {
      return;
    }

    try {
      const collection = await this.client.getCollection({
        name: collectionName,
      });

      await collection.update({
        ids: documents.map((doc) => doc.id),
        embeddings: documents.map((doc) => doc.embedding),
        metadatas: documents.map((doc) => doc.metadata),
        documents: documents.map((doc) => doc.document),
      });
    } catch (error) {
      throw new Error(
        `Failed to update documents in collection '${collectionName}': ${error}`,
      );
    }
  }

  /**
   * Delete documents from a collection
   */
  async deleteDocuments(collectionName: string, ids: string[]): Promise<void> {
    this.ensureConnected();

    if (ids.length === 0) {
      return;
    }

    try {
      const collection = await this.client.getCollection({
        name: collectionName,
      });
      await collection.delete({ ids });
    } catch (error) {
      throw new Error(
        `Failed to delete documents from collection '${collectionName}': ${error}`,
      );
    }
  }

  /**
   * Get documents from a collection
   */
  async getDocuments(
    collectionName: string,
    ids: string[],
  ): Promise<ChromaDocument[]> {
    this.ensureConnected();

    if (ids.length === 0) {
      return [];
    }

    try {
      const collection = await this.client.getCollection({
        name: collectionName,
      });
      const result = await collection.get({ ids });

      const documents: ChromaDocument[] = [];
      for (let i = 0; i < result.ids.length; i++) {
        documents.push({
          id: result.ids[i],
          embedding: result.embeddings?.[i] || [],
          metadata: result.metadatas?.[i] || {},
          document: result.documents?.[i] || "",
        });
      }

      return documents;
    } catch (error) {
      throw new Error(
        `Failed to get documents from collection '${collectionName}': ${error}`,
      );
    }
  }

  /**
   * Query collection with vector similarity search
   */
  async queryCollection(
    collectionName: string,
    queryEmbeddings: number[][],
    nResults: number = 10,
    where?: Record<string, unknown>,
  ): Promise<ChromaQueryResult> {
    this.ensureConnected();

    try {
      const collection = await this.client.getCollection({
        name: collectionName,
      });

      const result = await collection.query({
        queryEmbeddings,
        nResults,
        where,
        include: ["metadatas", "documents", "distances"],
      });

      return result;
    } catch (error) {
      throw new Error(
        `Failed to query collection '${collectionName}': ${error}`,
      );
    }
  }

  /**
   * Get collection count
   */
  async getCollectionCount(collectionName: string): Promise<number> {
    this.ensureConnected();

    try {
      const collection = await this.client.getCollection({
        name: collectionName,
      });
      return await collection.count();
    } catch (error) {
      throw new Error(
        `Failed to get count for collection '${collectionName}': ${error}`,
      );
    }
  }

  /**
   * Health check - ping ChromaDB
   */
  async healthCheck(): Promise<boolean> {
    if (!this.connected) {
      return false;
    }

    try {
      await this.client.heartbeat();
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Ensure client is connected, throw if not
   */
  private ensureConnected(): void {
    if (!this.connected || !this.client) {
      throw new Error(
        "ChromaDB client is not connected. Call connect() first.",
      );
    }
  }
}

/**
 * Factory function to create and connect ChromaDB client for local development
 */
export async function createLocalChromaClient(
  config: LocalChromaConfig,
): Promise<ChromaClient> {
  const client = new ChromaClient(config);
  await client.connect();
  return client;
}

/**
 * Factory function to create and connect ChromaDB client for hosted environment
 */
export async function createHostedChromaClient(
  config: HostedChromaConfig,
): Promise<ChromaClient> {
  const client = new ChromaClient(config);
  await client.connect();
  return client;
}

/**
 * Generic factory function that accepts any ChromaDB config
 */
export async function createChromaClient(
  config: ChromaConfig,
): Promise<ChromaClient> {
  const client = new ChromaClient(config);
  await client.connect();
  return client;
}
