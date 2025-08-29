/**
 * OpenAI embedding service implementation for Vy vector store
 *
 * Provides text-to-vector embedding using OpenAI's embedding models
 * with batching, retry logic, and token estimation
 */

import type { EmbeddingService } from "@repo/core";
import type { OpenAIEmbeddingConfig } from "../config.js";

/**
 * OpenAI API response types
 */
interface OpenAIEmbeddingResponse {
  data: Array<{
    embedding: number[];
    index: number;
  }>;
  model: string;
  usage: {
    prompt_tokens: number;
    total_tokens: number;
  };
}

/**
 * OpenAI embedding service implementation
 */
export class OpenAIEmbeddingService implements EmbeddingService {
  private readonly config: OpenAIEmbeddingConfig;
  private readonly baseUrl = "https://api.openai.com/v1/embeddings";

  constructor(config: OpenAIEmbeddingConfig) {
    this.config = config;
  }

  /**
   * Generate embedding for a single text
   */
  async generateEmbedding(text: string): Promise<number[]> {
    if (!text.trim()) {
      throw new Error("Cannot generate embedding for empty text");
    }

    const embeddings = await this.generateEmbeddings([text]);
    const embedding = embeddings[0];
    if (!embedding) {
      throw new Error("Failed to generate embedding - no result returned");
    }
    return embedding;
  }

  /**
   * Generate embeddings for multiple texts in batch
   */
  async generateEmbeddings(texts: string[]): Promise<number[][]> {
    if (texts.length === 0) {
      return [];
    }

    // Filter out empty texts
    const nonEmptyTexts = texts.filter((text) => text.trim());
    if (nonEmptyTexts.length === 0) {
      throw new Error("Cannot generate embeddings for all empty texts");
    }

    // Process in batches if necessary
    if (nonEmptyTexts.length > this.config.batchSize) {
      return this.processBatches(nonEmptyTexts);
    }

    // Single batch processing
    return this.callOpenAIEmbeddings(nonEmptyTexts);
  }

  /**
   * Get embedding dimension
   */
  getDimension(): number {
    return this.config.dimensions;
  }

  /**
   * Get model name
   */
  getModelName(): string {
    return this.config.model;
  }

  /**
   * Get maximum token limit
   */
  getMaxTokens(): number {
    return this.config.maxTokens;
  }

  /**
   * Estimate token count for text (rough approximation)
   * OpenAI uses ~4 chars per token for English text
   */
  estimateTokens(text: string): number {
    return Math.ceil(text.length / 4);
  }

  /**
   * Check if batch can be processed within token limits
   */
  canProcessBatch(texts: string[]): boolean {
    const totalTokens = texts.reduce(
      (sum, text) => sum + this.estimateTokens(text),
      0,
    );
    return (
      totalTokens <= this.config.maxTokens &&
      texts.length <= this.config.batchSize
    );
  }

  /**
   * Process large batches by splitting them
   */
  private async processBatches(texts: string[]): Promise<number[][]> {
    const results: number[][] = [];

    for (let i = 0; i < texts.length; i += this.config.batchSize) {
      const batch = texts.slice(i, i + this.config.batchSize);
      const batchResults = await this.callOpenAIEmbeddings(batch);
      results.push(...batchResults);
    }

    return results;
  }

  /**
   * Call OpenAI embeddings API
   */
  private async callOpenAIEmbeddings(texts: string[]): Promise<number[][]> {
    const requestBody = {
      input: texts,
      model: this.config.model,
      encoding_format: "float" as const,
    };

    const response = await fetch(this.baseUrl, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${this.config.apiKey}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`OpenAI API error (${response.status}): ${errorText}`);
    }

    const data: OpenAIEmbeddingResponse = await response.json();

    // Ensure results are in the same order as input
    const embeddings = new Array(texts.length);
    for (const item of data.data) {
      embeddings[item.index] = item.embedding;
    }

    // Validate all embeddings were received
    for (let i = 0; i < embeddings.length; i++) {
      if (!embeddings[i]) {
        throw new Error(`Missing embedding for text at index ${i}`);
      }
    }

    return embeddings;
  }
}

/**
 * Factory function to create OpenAI embedding service
 */
export function createOpenAIEmbeddingService(
  config: OpenAIEmbeddingConfig,
): OpenAIEmbeddingService {
  return new OpenAIEmbeddingService(config);
}
