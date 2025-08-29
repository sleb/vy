/**
 * Formatters for Vy CLI
 *
 * Consistent formatting functions for displaying data in the CLI interface.
 * Handles timestamps, durations, memory content, tables, and other UI elements.
 */

import chalk from "chalk";

/**
 * Format duration in milliseconds to human-readable format
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  } else if (ms < 60000) {
    return `${(ms / 1000).toFixed(1)}s`;
  } else {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  }
}

/**
 * Format timestamp to human-readable relative time
 */
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();

  // Less than 1 minute ago
  if (diffMs < 60000) {
    return "just now";
  }

  // Less than 1 hour ago
  if (diffMs < 3600000) {
    const minutes = Math.floor(diffMs / 60000);
    return `${minutes}m ago`;
  }

  // Less than 24 hours ago
  if (diffMs < 86400000) {
    const hours = Math.floor(diffMs / 3600000);
    return `${hours}h ago`;
  }

  // Less than 7 days ago
  if (diffMs < 604800000) {
    const days = Math.floor(diffMs / 86400000);
    return `${days}d ago`;
  }

  // More than 7 days ago - show actual date
  return date.toLocaleDateString();
}

/**
 * Format memory content for display
 */
export function formatMemory(content: string, maxLength: number = 100): string {
  if (!content) {
    return chalk.gray("(empty)");
  }

  // Clean up whitespace and newlines
  const cleaned = content
    .replace(/\s+/g, " ")
    .replace(/\n+/g, " ")
    .trim();

  // Truncate if too long
  if (cleaned.length <= maxLength) {
    return cleaned;
  }

  return cleaned.substring(0, maxLength - 3) + "...";
}

/**
 * Format file size in bytes to human-readable format
 */
export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(1)}${units[unitIndex]}`;
}

/**
 * Format percentage with color coding
 */
export function formatPercentage(value: number, colorize = true): string {
  const percentage = `${(value * 100).toFixed(1)}%`;

  if (!colorize) {
    return percentage;
  }

  if (value >= 0.8) {
    return chalk.green(percentage);
  } else if (value >= 0.5) {
    return chalk.yellow(percentage);
  } else {
    return chalk.red(percentage);
  }
}

/**
 * Format memory type with consistent styling
 */
export function formatMemoryType(type: string): string {
  const typeColors: Record<string, typeof chalk.yellow> = {
    conversation: chalk.blue,
    insight: chalk.green,
    learning: chalk.yellow,
    fact: chalk.cyan,
    'action-item': chalk.magenta
  };

  const color = typeColors[type] || chalk.gray;
  return color(type);
}

/**
 * Format a list with bullets
 */
export function formatList(items: string[], bullet = "•"): string {
  return items.map(item => `   ${chalk.gray(bullet)} ${item}`).join('\n');
}

/**
 * Format a key-value pair for display
 */
export function formatKeyValue(key: string, value: unknown, indent = 0): string {
  const spacing = " ".repeat(indent);
  const formattedKey = chalk.yellow(`${key}:`);

  if (typeof value === 'boolean') {
    return `${spacing}${formattedKey} ${value ? chalk.green('yes') : chalk.red('no')}`;
  } else if (typeof value === 'number') {
    return `${spacing}${formattedKey} ${chalk.cyan(value)}`;
  } else if (value === null || value === undefined) {
    return `${spacing}${formattedKey} ${chalk.gray('(not set)')}`;
  } else {
    return `${spacing}${formattedKey} ${String(value)}`;
  }
}

/**
 * Format a status indicator
 */
export function formatStatus(status: 'healthy' | 'unhealthy' | 'unknown', text?: string): string {
  const statusColors = {
    healthy: chalk.green,
    unhealthy: chalk.red,
    unknown: chalk.yellow
  };

  const statusIcons = {
    healthy: "✅",
    unhealthy: "❌",
    unknown: "⚠️"
  };

  const color = statusColors[status];
  const icon = statusIcons[status];
  const displayText = text || status;

  return `${icon} ${color(displayText)}`;
}

/**
 * Format a table header
 */
export function formatTableHeader(headers: string[]): string {
  return chalk.bold(chalk.blue(headers.join(' | ')));
}

/**
 * Format error message with context
 */
export function formatError(message: string, context?: Record<string, unknown>): string {
  let output = chalk.red(`❌ ${message}`);

  if (context) {
    const contextLines = Object.entries(context)
      .map(([key, value]) => `   ${chalk.gray(key)}: ${value}`)
      .join('\n');
    output += '\n' + contextLines;
  }

  return output;
}

/**
 * Format success message
 */
export function formatSuccess(message: string): string {
  return chalk.green(`✅ ${message}`);
}

/**
 * Format warning message
 */
export function formatWarning(message: string): string {
  return chalk.yellow(`⚠️  ${message}`);
}

/**
 * Format info message
 */
export function formatInfo(message: string): string {
  return chalk.blue(`ℹ️  ${message}`);
}

/**
 * Format a progress bar
 */
export function formatProgressBar(current: number, total: number, width = 20): string {
  const percentage = total > 0 ? current / total : 0;
  const filled = Math.round(width * percentage);
  const empty = width - filled;

  const bar = '█'.repeat(filled) + '░'.repeat(empty);
  const percent = `${(percentage * 100).toFixed(1)}%`;

  return `${chalk.cyan(bar)} ${chalk.yellow(percent)} (${current}/${total})`;
}

/**
 * Truncate text with ellipsis
 */
export function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) {
    return text;
  }

  return text.substring(0, maxLength - 3) + "...";
}

/**
 * Center text within specified width
 */
export function center(text: string, width: number): string {
  const padding = Math.max(0, width - text.length);
  const leftPad = Math.floor(padding / 2);
  const rightPad = padding - leftPad;

  return ' '.repeat(leftPad) + text + ' '.repeat(rightPad);
}

/**
 * Format JSON with syntax highlighting
 */
export function formatJson(obj: unknown, indent = 2): string {
  const json = JSON.stringify(obj, null, indent);

  return json
    .replace(/(".*?")\s*:/g, chalk.blue('$1') + ':')
    .replace(/:\s*(".*?")/g, ': ' + chalk.green('$1'))
    .replace(/:\s*(true|false)/g, ': ' + chalk.yellow('$1'))
    .replace(/:\s*(null)/g, ': ' + chalk.gray('$1'))
    .replace(/:\s*(\d+)/g, ': ' + chalk.cyan('$1'));
}
