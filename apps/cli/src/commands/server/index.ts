/**
 * Server Commands - MCP server management for Vy CLI
 *
 * These commands handle starting, stopping, monitoring, and managing
 * the Vy MCP server process.
 */

import chalk from "chalk";
import { spawn } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import ora from "ora";
import { table } from "table";
import {
  formatDuration,
  formatStatus,
  formatTimestamp,
} from "../../lib/formatters.js";
import { testConnection } from "../../lib/mcp-client.js";
import { handleError, validateConfig } from "../../lib/utils.js";

/**
 * Server process management
 */
const SERVER_PID_FILE = join(homedir(), ".vy", "server.pid");
const SERVER_LOG_FILE = join(homedir(), ".vy", "server.log");

/**
 * Start the MCP server
 */
async function start(options?: Record<string, unknown>): Promise<void> {
  const spinner = ora("Starting Vy MCP server...").start();

  try {
    // Check if server is already running
    if (isServerRunning()) {
      spinner.warn("Server is already running");
      const pid = getServerPid();
      console.log(chalk.yellow(`   PID: ${pid}`));
      console.log(chalk.gray("   Use 'vy server stop' to stop the server"));
      return;
    }

    // Validate configuration before starting
    await validateConfig();
    spinner.text = "Configuration validated, starting server...";

    // Determine server path
    const serverPath = join(
      process.cwd(),
      "packages/mcp-server-basic/dist/cli.js",
    );
    if (!existsSync(serverPath)) {
      throw new Error(`Server executable not found: ${serverPath}`);
    }

    // Prepare environment
    const env = {
      ...process.env,
      VY_LOG_LEVEL: options?.logLevel || "info",
    };

    // Start server process
    const serverProcess = spawn("node", [serverPath], {
      detached: true,
      stdio: options?.daemon ? "ignore" : ["ignore", "pipe", "pipe"],
      env,
    });

    if (options?.daemon) {
      // Daemon mode - detach and save PID
      serverProcess.unref();
      saveServerPid(serverProcess.pid!);
      spinner.succeed(
        `Server started in daemon mode (PID: ${serverProcess.pid})`,
      );

      console.log(chalk.green("\n🚀 Vy MCP server is running"));
      console.log(chalk.gray(`   PID: ${serverProcess.pid}`));
      console.log(chalk.gray(`   Logs: ${SERVER_LOG_FILE}`));
      console.log(chalk.gray("   Use 'vy server stop' to stop the server"));
    } else {
      // Foreground mode
      spinner.succeed("Server started in foreground mode");
      console.log(chalk.green("\n🚀 Vy MCP server is running"));
      console.log(chalk.gray("   Press Ctrl+C to stop\n"));

      // Handle logs in foreground mode
      if (serverProcess.stdout) {
        serverProcess.stdout.on("data", (data) => {
          process.stdout.write(data);
        });
      }

      if (serverProcess.stderr) {
        serverProcess.stderr.on("data", (data) => {
          process.stderr.write(data);
        });
      }

      // Handle process exit
      serverProcess.on("exit", (code) => {
        if (code === 0) {
          console.log(chalk.yellow("\n👋 Server stopped"));
        } else {
          console.log(chalk.red(`\n💥 Server exited with code ${code}`));
        }
        process.exit(code || 0);
      });

      // Handle Ctrl+C
      process.on("SIGINT", () => {
        console.log(chalk.yellow("\n🛑 Stopping server..."));
        serverProcess.kill("SIGTERM");
      });
    }
  } catch (error) {
    spinner.fail("Failed to start server");
    handleError(error, options?.verbose);
  }
}

/**
 * Stop the MCP server
 */
async function stop(options?: Record<string, unknown>): Promise<void> {
  const spinner = ora("Stopping Vy MCP server...").start();

  try {
    if (!isServerRunning()) {
      spinner.warn("Server is not running");
      return;
    }

    const pid = getServerPid();
    if (!pid) {
      throw new Error("Server PID not found");
    }

    // Try graceful shutdown first
    try {
      process.kill(pid, "SIGTERM");

      // Wait for graceful shutdown
      let attempts = 0;
      const maxAttempts = 10;

      while (attempts < maxAttempts && isProcessRunning(pid)) {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        attempts++;
      }

      // Force kill if still running
      if (isProcessRunning(pid)) {
        process.kill(pid, "SIGKILL");
        await new Promise((resolve) => setTimeout(resolve, 1000));
      }

      // Clean up PID file
      clearServerPid();
      spinner.succeed(`Server stopped (PID: ${pid})`);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ESRCH") {
        // Process doesn't exist, clean up PID file
        clearServerPid();
        spinner.succeed("Server was not running, cleaned up PID file");
      } else {
        throw error;
      }
    }
  } catch (error) {
    spinner.fail("Failed to stop server");
    handleError(error, options?.verbose);
  }
}

/**
 * Check server status
 */
async function showStatus(options: Record<string, unknown>): Promise<void> {
  const spinner = ora("Checking server status...").start();

  try {
    const isRunning = isServerRunning();
    const pid = getServerPid();

    spinner.stop();

    if (options?.json) {
      const statusData = {
        running: isRunning,
        pid: pid || null,
        uptime: isRunning && pid ? getProcessUptime(pid) : null,
        healthy: false,
      };

      // Test connection if running
      if (isRunning) {
        try {
          statusData.healthy = await testConnection();
        } catch {
          statusData.healthy = false;
        }
      }

      console.log(JSON.stringify(statusData, null, 2));
      return;
    }

    // Human-readable status
    console.log(chalk.blue("\n🖥️  Vy MCP Server Status\n"));

    const tableData = [
      [
        "Status",
        isRunning
          ? formatStatus("healthy", "Running")
          : formatStatus("unhealthy", "Stopped"),
      ],
      ["PID", pid ? String(pid) : chalk.gray("N/A")],
      [
        "Uptime",
        isRunning && pid
          ? formatDuration(getProcessUptime(pid))
          : chalk.gray("N/A"),
      ],
    ];

    // Test connectivity if running
    if (isRunning) {
      spinner.start("Testing connectivity...");
      try {
        const connected = await testConnection();
        spinner.stop();
        tableData.push([
          "Connectivity",
          connected
            ? formatStatus("healthy", "Connected")
            : formatStatus("unhealthy", "Not responding"),
        ]);
      } catch {
        spinner.stop();
        tableData.push([
          "Connectivity",
          formatStatus("unhealthy", "Connection failed"),
        ]);
      }
    }

    console.log(
      table(tableData, {
        border: {
          topBody: "─",
          topJoin: "┬",
          topLeft: "┌",
          topRight: "┐",
          bottomBody: "─",
          bottomJoin: "┴",
          bottomLeft: "└",
          bottomRight: "┘",
          bodyLeft: "│",
          bodyRight: "│",
          bodyJoin: "│",
        },
      }),
    );
  } catch (error) {
    spinner.fail("Failed to check status");
    handleError(error, options?.verbose);
  }
}

/**
 * Perform health check
 */
async function health(options?: Record<string, unknown>): Promise<void> {
  const spinner = ora("Performing health check...").start();
  const timeout = parseInt(options?.timeout || "5000");

  try {
    // Check if server process is running
    const isRunning = isServerRunning();
    if (!isRunning) {
      throw new Error("Server is not running");
    }

    // Test configuration
    await validateConfig();
    spinner.text = "Configuration validated...";

    // Test MCP connectivity with timeout
    const connected = await Promise.race([
      testConnection(),
      new Promise<boolean>((_, reject) =>
        setTimeout(() => reject(new Error("Health check timeout")), timeout),
      ),
    ]);

    if (!connected) {
      throw new Error("Server is not responding to MCP requests");
    }

    spinner.succeed("Health check passed");

    if (options?.json) {
      console.log(
        JSON.stringify(
          {
            status: "healthy",
            timestamp: new Date().toISOString(),
            checks: {
              process: true,
              configuration: true,
              connectivity: true,
            },
          },
          null,
          2,
        ),
      );
    } else {
      console.log(chalk.green("\n✅ Server is healthy"));
      console.log(chalk.gray("   All health checks passed"));
    }
  } catch (error) {
    spinner.fail("Health check failed");

    if (options?.json) {
      console.log(
        JSON.stringify(
          {
            status: "unhealthy",
            timestamp: new Date().toISOString(),
            error: error instanceof Error ? error.message : String(error),
          },
          null,
          2,
        ),
      );
    } else {
      handleError(error, options?.verbose);
    }
  }
}

/**
 * View server logs
 */
async function logs(options?: Record<string, unknown>): Promise<void> {
  try {
    if (!existsSync(SERVER_LOG_FILE)) {
      console.log(chalk.yellow("📜 No log file found"));
      console.log(chalk.gray(`   Expected location: ${SERVER_LOG_FILE}`));
      console.log(
        chalk.gray("   Logs are only available when running in daemon mode"),
      );
      return;
    }

    const lines = parseInt(options?.lines || "50");
    const logContent = readFileSync(SERVER_LOG_FILE, "utf-8");
    const logLines = logContent.split("\n").filter((line) => line.trim());

    if (options?.follow) {
      console.log(chalk.blue("📜 Following server logs (Ctrl+C to stop)\n"));

      // Show last N lines first
      const recentLines = logLines.slice(-lines);
      recentLines.forEach((line) => console.log(formatLogLine(line)));

      // TODO: Implement log following with fs.watchFile
      console.log(chalk.yellow("\n⚠️  Log following not yet implemented"));
      console.log(
        chalk.gray(
          "   Use 'vy server logs' without --follow to view recent logs",
        ),
      );
    } else {
      console.log(chalk.blue(`📜 Server logs (last ${lines} lines)\n`));

      const displayLines = logLines.slice(-lines);
      if (displayLines.length === 0) {
        console.log(chalk.gray("   No log entries found"));
      } else {
        displayLines.forEach((line) => console.log(formatLogLine(line)));
      }
    }
  } catch (error) {
    console.error(chalk.red("❌ Failed to read logs"));
    handleError(error, options?.verbose);
  }
}

// Helper functions

/**
 * Check if server is running
 */
function isServerRunning(): boolean {
  const pid = getServerPid();
  return pid !== null && isProcessRunning(pid);
}

/**
 * Get server PID from file
 */
function getServerPid(): number | null {
  try {
    if (!existsSync(SERVER_PID_FILE)) {
      return null;
    }

    const pid = parseInt(readFileSync(SERVER_PID_FILE, "utf-8").trim());
    return isNaN(pid) ? null : pid;
  } catch {
    return null;
  }
}

/**
 * Save server PID to file
 */
function saveServerPid(pid: number): void {
  const vyDir = join(homedir(), ".vy");
  if (!existsSync(vyDir)) {
    mkdirSync(vyDir, { recursive: true });
  }

  writeFileSync(SERVER_PID_FILE, String(pid), "utf-8");
}

/**
 * Clear server PID file
 */
function clearServerPid(): void {
  try {
    if (existsSync(SERVER_PID_FILE)) {
      unlinkSync(SERVER_PID_FILE);
    }
  } catch {
    // Ignore errors when cleaning up
  }
}

/**
 * Check if process is running
 */
function isProcessRunning(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/**
 * Get process uptime in milliseconds
 */
function getProcessUptime(): number {
  try {
    // This is a simplified uptime calculation
    // In a real implementation, you'd read from /proc/{pid}/stat on Linux
    return Date.now() - statSync(SERVER_PID_FILE).mtimeMs;
  } catch {
    return 0;
  }
}

/**
 * Format log line for display
 */
function formatLogLine(line: string): string {
  try {
    // Try to parse as JSON log entry
    const entry = JSON.parse(line);
    const timestamp = formatTimestamp(entry.timestamp);
    const level = entry.level?.toUpperCase() || "INFO";

    let levelColor = chalk.blue;
    if (level === "ERROR") levelColor = chalk.red;
    else if (level === "WARN") levelColor = chalk.yellow;
    else if (level === "DEBUG") levelColor = chalk.gray;

    return `${chalk.gray(timestamp)} ${levelColor(level.padStart(5))} ${entry.message}`;
  } catch {
    // Not JSON, display as plain text
    return chalk.gray(line);
  }
}

// Export all commands
export { health, logs, start, showStatus as status, stop };
