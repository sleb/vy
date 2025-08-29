/**
 * CLI Testing App for Vy MCP Server
 *
 * This application provides a command-line interface for testing and validating
 * the Vy MCP server during development. It can:
 *
 * - Test individual MCP tools (capture_conversation, search_memory, get_context)
 * - Run the MCP server in test mode with mock data
 * - Validate server initialization and health
 * - Simulate client interactions for integration testing
 */

export * from './cli.js';
export * from './mock-data.js';
export * from './test-client.js';
export * from './test-scenarios.js';

/**
 * Test result interfaces
 */
export interface TestResult {
  success: boolean;
  duration: number;
  error?: string;
  data?: unknown;
}

export interface TestSuite {
  name: string;
  description: string;
  tests: TestCase[];
}

export interface TestCase {
  name: string;
  description: string;
  run: () => Promise<TestResult>;
}

/**
 * CLI configuration
 */
export interface CliConfig {
  serverPath?: string;
  timeout?: number;
  verbose?: boolean;
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}

/**
 * Default configuration
 */
export const DEFAULT_CLI_CONFIG: Required<CliConfig> = {
  serverPath: '../mcp-server-basic/dist/cli.js',
  timeout: 30000, // 30 seconds
  verbose: false,
  logLevel: 'info'
};
