/**
 * MCP Client Library for Vy CLI
 *
 * Provides a simple client interface for communicating with the Vy MCP server
 * from CLI commands. Handles connection management, tool calls, and error handling.
 */

import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { spawn } from "node:child_process";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

/**
 * MCP client configuration
 */
export interface McpClientConfig {
  serverPath?: string;
  timeout?: number;
  retries?: number;
  env?: Record<string, string>;
}

/**
 * Default configuration
 */
const DEFAULT_CONFIG: Required<McpClientConfig> = {
  serverPath: join(__dirname, "../../../mcp-server-basic/dist/cli.js"),
  timeout: 30000,
  retries: 3,
  env: {}
};

/**
 * MCP Client wrapper for CLI usage
 */
export class VyMcpClient {
  private client: Client;
  private transport: StdioClientTransport;
  private serverProcess: any;
  private connected = false;
  private config: Required<McpClientConfig>;

  constructor(config: McpClientConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Connect to the MCP server
   */
  async connect(): Promise<void> {
    if (this.connected) {
      return;
    }

    try {
      // Spawn the MCP server process
      this.serverProcess = spawn("node", [this.config.serverPath], {
        stdio: ["pipe", "pipe", "inherit"],
        env: {
          ...process.env,
          ...this.config.env
        }
      });

      // Handle server process errors
      this.serverProcess.on("error", (error: Error) => {
        throw new Error(`Failed to start MCP server: ${error.message}`);
      });

      this.serverProcess.on("exit", (code: number) => {
        if (code !== 0) {
          throw new Error(`MCP server exited with code ${code}`);
        }
      });

      // Create transport using the server's stdio
      this.transport = new StdioClientTransport(
        this.serverProcess.stdout,
        this.serverProcess.stdin
      );

      // Create MCP client
      this.client = new Client(
        {
          name: "vy-cli",
          version: "0.0.1"
        },
        {
          capabilities: {}
        }
      );

      // Connect client to transport
      await this.client.connect(this.transport);

      this.connected = true;
    } catch (error) {
      await this.cleanup();
      throw error;
    }
  }

  /**
   * Call a tool on the MCP server
   */
  async callTool(name: string, arguments_: unknown): Promise<any> {
    if (!this.connected) {
      await this.connect();
    }

    try {
      const result = await this.client.callTool({ name, arguments: arguments_ });
      return result.content;
    } catch (error) {
      throw new Error(`Tool call failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * List available tools
   */
  async listTools(): Promise<any[]> {
    if (!this.connected) {
      await this.connect();
    }

    try {
      const result = await this.client.listTools();
      return result.tools || [];
    } catch (error) {
      throw new Error(`Failed to list tools: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Get server information
   */
  async getServerInfo(): Promise<any> {
    if (!this.connected) {
      await this.connect();
    }

    try {
      // The server info is available after connection
      return {
        name: "vy-mcp-server",
        version: "0.0.1",
        connected: this.connected
      };
    } catch (error) {
      throw new Error(`Failed to get server info: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Close the connection and cleanup
   */
  async close(): Promise<void> {
    await this.cleanup();
  }

  /**
   * Check if client is connected
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Internal cleanup method
   */
  private async cleanup(): Promise<void> {
    this.connected = false;

    try {
      // Close the MCP client connection
      if (this.client) {
        await this.client.close();
      }

      // Close the transport
      if (this.transport) {
        await this.transport.close();
      }

      // Terminate the server process
      if (this.serverProcess) {
        this.serverProcess.kill("SIGTERM");

        // Wait for graceful shutdown, then force kill if needed
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            if (this.serverProcess && !this.serverProcess.killed) {
              this.serverProcess.kill("SIGKILL");
            }
            resolve();
          }, 5000);

          this.serverProcess.on("exit", () => {
            clearTimeout(timeout);
            resolve();
          });
        });
      }
    } catch (error) {
      // Log cleanup errors but don't throw
      console.error("Error during cleanup:", error);
    }
  }
}

/**
 * Create a new MCP client with default configuration
 */
export async function createMcpClient(config?: McpClientConfig): Promise<VyMcpClient> {
  const client = new VyMcpClient(config);
  await client.connect();
  return client;
}

/**
 * Test MCP server connectivity
 */
export async function testConnection(config?: McpClientConfig): Promise<boolean> {
  let client: VyMcpClient | null = null;

  try {
    client = await createMcpClient(config);
    const tools = await client.listTools();
    return Array.isArray(tools) && tools.length > 0;
  } catch {
    return false;
  } finally {
    if (client) {
      await client.close();
    }
  }
}

/**
 * Get available tools from the server
 */
export async function getAvailableTools(config?: McpClientConfig): Promise<string[]> {
  let client: VyMcpClient | null = null;

  try {
    client = await createMcpClient(config);
    const tools = await client.listTools();
    return tools.map((tool: any) => tool.name || "unknown");
  } catch {
    return [];
  } finally {
    if (client) {
      await client.close();
    }
  }
}
