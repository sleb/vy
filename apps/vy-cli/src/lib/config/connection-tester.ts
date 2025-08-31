/**
 * Connection testing utilities for Vy CLI
 *
 * Tests connections to external services like OpenAI API and ChromaDB
 * to validate configuration and diagnose connectivity issues.
 */

import type { ConnectionTestResult, VyConfig } from "@repo/core";
import { performance } from "perf_hooks";

/**
 * Connection tester for validating external service connections
 */
export class ConnectionTester {
  constructor(private config: VyConfig) {}

  /**
   * Test all connections
   */
  async testAll(): Promise<ConnectionTestResult[]> {
    const results = await Promise.allSettled([
      this.testOpenAI(),
      this.testChromaDB(),
    ]);

    return results.map((result, index) => {
      const service = index === 0 ? "OpenAI API" : "ChromaDB";

      if (result.status === "fulfilled") {
        return result.value;
      } else {
        return {
          service,
          success: false,
          message: `Test failed: ${result.reason?.message || "Unknown error"}`,
          details: { error: result.reason },
        };
      }
    });
  }

  /**
   * Test OpenAI API connection and authentication
   */
  async testOpenAI(): Promise<ConnectionTestResult> {
    const startTime = performance.now();

    try {
      const response = await fetch("https://api.openai.com/v1/models", {
        method: "GET",
        headers: {
          Authorization: `Bearer ${this.config.embedding.openaiApiKey}`,
          "Content-Type": "application/json",
        },
        signal: AbortSignal.timeout(10000), // 10 second timeout
      });

      const duration = performance.now() - startTime;

      if (!response.ok) {
        const errorText = await response.text().catch(() => "Unknown error");

        if (response.status === 401) {
          return {
            service: "OpenAI API",
            success: false,
            message: "Authentication failed - invalid API key",
            details: {
              status: response.status,
              error: errorText,
              keyPrefix:
                this.config.embedding.openaiApiKey.substring(0, 7) + "...",
            },
            duration,
          };
        }

        if (response.status === 429) {
          return {
            service: "OpenAI API",
            success: false,
            message: "Rate limit exceeded - too many requests",
            details: {
              status: response.status,
              error: errorText,
            },
            duration,
          };
        }

        return {
          service: "OpenAI API",
          success: false,
          message: `API request failed with status ${response.status}`,
          details: {
            status: response.status,
            error: errorText,
          },
          duration,
        };
      }

      const data = await response.json();

      // Type guard for OpenAI API response
      const isValidModelsResponse = (
        data: unknown,
      ): data is { data: Array<{ id: string }> } => {
        return (
          typeof data === "object" &&
          data !== null &&
          "data" in data &&
          Array.isArray((data as { data: unknown }).data)
        );
      };

      if (!isValidModelsResponse(data)) {
        return {
          service: "OpenAI API",
          success: false,
          message: "Invalid response format from OpenAI API",
          details: { error: "Unexpected response structure" },
          duration,
        };
      }

      // Check if the configured embedding model is available
      const hasEmbeddingModel = data.data.some(
        (model) => model.id === this.config.embedding.model,
      );

      if (!hasEmbeddingModel) {
        return {
          service: "OpenAI API",
          success: false,
          message: `Configured embedding model '${this.config.embedding.model}' not found`,
          details: {
            configuredModel: this.config.embedding.model,
            availableModels: data.data
              .filter((m) => m.id.includes("embedding"))
              .map((m) => m.id),
          },
          duration,
        };
      }

      return {
        service: "OpenAI API",
        success: true,
        message: `Connected successfully - model '${this.config.embedding.model}' is available`,
        details: {
          embeddingModel: this.config.embedding.model,
          totalModels: data.data.length,
        },
        duration,
      };
    } catch (error: unknown) {
      const duration = performance.now() - startTime;

      if ((error as Error).name === "AbortError") {
        return {
          service: "OpenAI API",
          success: false,
          message: "Connection timeout after 10 seconds",
          details: { error: "Timeout" },
          duration,
        };
      }

      if (error.cause?.code === "ENOTFOUND") {
        return {
          service: "OpenAI API",
          success: false,
          message: "DNS resolution failed - check internet connection",
          details: { error: error.message },
          duration,
        };
      }

      if (error.cause?.code === "ECONNREFUSED") {
        return {
          service: "OpenAI API",
          success: false,
          message: "Connection refused - service may be down",
          details: { error: error.message },
          duration,
        };
      }

      return {
        service: "OpenAI API",
        success: false,
        message: `Connection failed: ${error.message}`,
        details: { error: error.message },
        duration,
      };
    }
  }

  /**
   * Test ChromaDB connection
   */
  async testChromaDB(): Promise<ConnectionTestResult> {
    const startTime = performance.now();
    const { chromaHost, chromaPort, chromaSsl, chromaApiKey } =
      this.config.vectorStore;

    const protocol = chromaSsl ? "https" : "http";
    const baseUrl = `${protocol}://${chromaHost}:${chromaPort}`;

    try {
      // Test basic connectivity with heartbeat endpoint
      const headers: Record<string, string> = {
        "Content-Type": "application/json",
      };

      if (chromaApiKey) {
        headers["Authorization"] = `Bearer ${chromaApiKey}`;
      }

      const response = await fetch(`${baseUrl}/api/v1/heartbeat`, {
        method: "GET",
        headers,
        signal: AbortSignal.timeout(10000), // 10 second timeout
      });

      const duration = performance.now() - startTime;

      if (!response.ok) {
        const errorText = await response.text().catch(() => "Unknown error");

        if (response.status === 401) {
          return {
            service: "ChromaDB",
            success: false,
            message: "Authentication failed - invalid API key",
            details: {
              status: response.status,
              error: errorText,
              url: baseUrl,
              hasApiKey: !!chromaApiKey,
            },
            duration,
          };
        }

        if (response.status === 404) {
          return {
            service: "ChromaDB",
            success: false,
            message: "ChromaDB API not found - check version compatibility",
            details: {
              status: response.status,
              error: errorText,
              url: baseUrl,
              suggestion:
                "ChromaDB may be an older version without /api/v1/heartbeat endpoint",
            },
            duration,
          };
        }

        return {
          service: "ChromaDB",
          success: false,
          message: `ChromaDB request failed with status ${response.status}`,
          details: {
            status: response.status,
            error: errorText,
            url: baseUrl,
          },
          duration,
        };
      }

      // Try to get version information
      let versionInfo: Record<string, unknown> = {};
      try {
        const versionResponse = await fetch(`${baseUrl}/api/v1/version`, {
          method: "GET",
          headers,
          signal: AbortSignal.timeout(5000),
        });

        if (versionResponse.ok) {
          versionInfo = await versionResponse.json();
        }
      } catch {
        // Version endpoint not available - not critical
      }

      // Test collection access
      const collectionName = this.config.vectorStore.collectionName;
      let collectionExists = false;

      try {
        const collectionsResponse = await fetch(
          `${baseUrl}/api/v1/collections`,
          {
            method: "GET",
            headers,
            signal: AbortSignal.timeout(5000),
          },
        );

        if (collectionsResponse.ok) {
          const collectionsData = await collectionsResponse.json();

          // Type guard for collections response
          const isValidCollectionsResponse = (
            data: unknown,
          ): data is Array<{ name: string }> => {
            return (
              Array.isArray(data) &&
              data.every(
                (item) =>
                  typeof item === "object" && item !== null && "name" in item,
              )
            );
          };

          if (isValidCollectionsResponse(collectionsData)) {
            collectionExists = collectionsData.some(
              (col) => col.name === collectionName,
            );
          }
        }
      } catch {
        // Collections endpoint not accessible - not critical for connectivity test
      }

      return {
        service: "ChromaDB",
        success: true,
        message: `Connected successfully to ChromaDB at ${baseUrl}`,
        details: {
          url: baseUrl,
          ssl: chromaSsl,
          authenticated: !!chromaApiKey,
          version: versionInfo.version || "unknown",
          collectionName,
          collectionExists,
        },
        duration,
      };
    } catch (error: unknown) {
      const duration = performance.now() - startTime;

      if ((error as Error).name === "AbortError") {
        return {
          service: "ChromaDB",
          success: false,
          message: "Connection timeout after 10 seconds",
          details: {
            error: "Timeout",
            url: baseUrl,
            suggestion: "ChromaDB may be slow to respond or unavailable",
          },
          duration,
        };
      }

      if (error.cause?.code === "ENOTFOUND") {
        return {
          service: "ChromaDB",
          success: false,
          message: `Cannot resolve hostname '${chromaHost}'`,
          details: {
            error: error.message,
            host: chromaHost,
            suggestion: "Check if ChromaDB host is correct and accessible",
          },
          duration,
        };
      }

      if (error.cause?.code === "ECONNREFUSED") {
        return {
          service: "ChromaDB",
          success: false,
          message: `Connection refused to ${chromaHost}:${chromaPort}`,
          details: {
            error: error.message,
            url: baseUrl,
            suggestion: "Check if ChromaDB is running and port is correct",
          },
          duration,
        };
      }

      if (
        error.cause?.code === "CERT_HAS_EXPIRED" ||
        error.cause?.code === "UNABLE_TO_VERIFY_LEAF_SIGNATURE"
      ) {
        return {
          service: "ChromaDB",
          success: false,
          message: "SSL certificate verification failed",
          details: {
            error: error.message,
            url: baseUrl,
            suggestion:
              "Check SSL certificate validity or disable SSL if using local development",
          },
          duration,
        };
      }

      return {
        service: "ChromaDB",
        success: false,
        message: `Connection failed: ${error.message}`,
        details: {
          error: error.message,
          url: baseUrl,
        },
        duration,
      };
    }
  }

  /**
   * Test specific service by name
   */
  async testService(
    serviceName: "openai" | "chromadb",
  ): Promise<ConnectionTestResult> {
    switch (serviceName) {
      case "openai":
        return this.testOpenAI();
      case "chromadb":
        return this.testChromaDB();
      default:
        return {
          service: serviceName,
          success: false,
          message: `Unknown service: ${serviceName}`,
        };
    }
  }
}

/**
 * Create connection tester instance
 */
export function createConnectionTester(config: VyConfig): ConnectionTester {
  return new ConnectionTester(config);
}
