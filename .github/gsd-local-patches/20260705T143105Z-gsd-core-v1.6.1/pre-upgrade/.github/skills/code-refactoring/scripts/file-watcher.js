#!/usr/bin/env node

/**
 * Code Refactoring Skill - Real-time File Watcher
 * Monitors files for size violations and alerts proactively
 *
 * Usage:
 *   node file-watcher.js [directory] [options]
 *
 * Options:
 *   --quiet          Only show warnings/errors
 *   --notify         Send desktop notifications (requires node-notifier)
 *   --log=<file>     Write alerts to log file
 *   --threshold=<n>  Custom warning threshold (default: 150)
 */

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

// Configuration
const CONFIG = {
  watchDir: process.argv[2] || process.cwd(),
  quiet: process.argv.includes('--quiet'),
  notify: process.argv.includes('--notify'),
  logFile: process.argv.find(arg => arg.startsWith('--log='))?.split('=')[1],
  threshold: parseInt(process.argv.find(arg => arg.startsWith('--threshold='))?.split('=')[1] || '150'),

  // File patterns to watch
  patterns: {
    react: /\.(tsx|jsx)$/,
    typescript: /\.ts$/,
    javascript: /\.js$/,
    python: /\.py$/,
  },

  // Size thresholds - NOW WITH PATH-BASED INTELLIGENCE! (v2.0)
  // Path-based patterns checked FIRST, then falls back to extension-based
  thresholds: {
    // Path-based thresholds (context-aware)
    pathBased: [
      {
        pattern: /page\.tsx$/i,
        warning: 300, alert: 500, critical: 800,
        reason: 'Page component (educational content/demos allowed)'
      },
      {
        pattern: /[/\\]data[/\\].*\.(tsx|ts)$/i,
        warning: 250, alert: 400, critical: 600,
        reason: 'Data file (mostly static content)'
      },
      {
        pattern: /[/\\]components[/\\].*\.(tsx|jsx)$/i,
        warning: 150, alert: 200, critical: 300,
        reason: 'Component file (standard threshold)'
      },
      {
        pattern: /[/\\](lib|utils)[/\\].*\.(ts|js)$/i,
        warning: 100, alert: 150, critical: 200,
        reason: 'Utility/logic file (strict - should be small)'
      },
      {
        pattern: /[/\\]api[/\\].*\.(ts|js)$/i,
        warning: 100, alert: 150, critical: 250,
        reason: 'API route (thin controller preferred)'
      },
    ],

    // Fallback: Extension-based thresholds (when path doesn't match)
    extensionBased: {
      react: { warning: 150, alert: 200, critical: 300 },
      typescript: { warning: 150, alert: 200, critical: 300 },
      javascript: { warning: 150, alert: 200, critical: 300 },
      python: { warning: 250, alert: 400, critical: 500 },
      default: { warning: 150, alert: 200, critical: 300 },
    },
  },

  // Directories to ignore
  ignore: [
    'node_modules',
    '.git',
    'dist',
    'build',
    'out',
    '.next',
    'coverage',
    '.vercel',
    '__pycache__',
    '.pytest_cache',
  ],
};

// Colors for console output
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  green: '\x1b[32m',
  cyan: '\x1b[36m',
  gray: '\x1b[90m',
};

// Statistics
const stats = {
  filesChecked: 0,
  warnings: 0,
  alerts: 0,
  critical: 0,
  startTime: Date.now(),
};

// Track problematic files for edit reminders
const problematicFiles = new Map(); // filepath -> {level, lines, lastReminded}

/**
 * Get file type from extension
 */
function getFileType(filename) {
  for (const [type, pattern] of Object.entries(CONFIG.patterns)) {
    if (pattern.test(filename)) return type;
  }
  return 'default';
}

/**
 * Get thresholds for a file based on path patterns (v2.0)
 * Checks path-based patterns first, then falls back to extension-based
 */
function getThresholds(filePath) {
  // Normalize path separators for cross-platform compatibility
  const normalizedPath = filePath.replace(/\\/g, '/');

  // Check path-based patterns first (most specific)
  for (const pathThreshold of CONFIG.thresholds.pathBased) {
    if (pathThreshold.pattern.test(normalizedPath)) {
      return {
        warning: pathThreshold.warning,
        alert: pathThreshold.alert,
        critical: pathThreshold.critical,
        source: `path-based (${pathThreshold.reason})`,
        reason: pathThreshold.reason,
      };
    }
  }

  // Fallback to extension-based thresholds
  const fileType = getFileType(filePath);
  const extensionThresholds = CONFIG.thresholds.extensionBased[fileType] || CONFIG.thresholds.extensionBased.default;

  return {
    warning: extensionThresholds.warning,
    alert: extensionThresholds.alert,
    critical: extensionThresholds.critical,
    source: `extension-based (${fileType})`,
    reason: `${fileType} file`,
  };
}

/**
 * Check if path should be ignored
 */
function shouldIgnore(filePath) {
  const relativePath = path.relative(CONFIG.watchDir, filePath);
  return CONFIG.ignore.some(ignored => relativePath.includes(ignored));
}

/**
 * Count lines in file
 */
function countLines(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    return content.split('\n').length;
  } catch (error) {
    return 0;
  }
}

/**
 * Analyze file size and return alert level (v2.0 - now path-aware!)
 */
function analyzeFile(filePath) {
  const lines = countLines(filePath);
  const thresholds = getThresholds(filePath); // NEW: Use path-based or extension-based thresholds
  const fileType = getFileType(filePath);

  stats.filesChecked++;

  if (lines >= thresholds.critical) {
    stats.critical++;
    return { level: 'critical', lines, thresholds, fileType, thresholdSource: thresholds.source };
  } else if (lines >= thresholds.alert) {
    stats.alerts++;
    return { level: 'alert', lines, thresholds, fileType, thresholdSource: thresholds.source };
  } else if (lines >= thresholds.warning) {
    stats.warnings++;
    return { level: 'warning', lines, thresholds, fileType, thresholdSource: thresholds.source };
  }

  return { level: 'ok', lines, thresholds, fileType, thresholdSource: thresholds.source };
}

/**
 * Format alert message (v2.0 - shows threshold source)
 */
function formatAlert(filePath, analysis) {
  const relativePath = path.relative(CONFIG.watchDir, filePath);
  const { level, lines, thresholds, thresholdSource } = analysis;

  const symbols = {
    critical: '🛑',
    alert: '🚨',
    warning: '⚠️ ',
    ok: '✅',
  };

  const levelColors = {
    critical: colors.red,
    alert: colors.yellow,
    warning: colors.yellow,
    ok: colors.green,
  };

  const messages = {
    critical: `CRITICAL: File MUST be refactored immediately! (>${thresholds.critical} lines)`,
    alert: `ALERT: File should be refactored before adding more. (>${thresholds.alert} lines)`,
    warning: `WARNING: File is getting large. Consider refactoring. (>${thresholds.warning} lines)`,
    ok: 'File size is healthy.',
  };

  const color = levelColors[level];
  const symbol = symbols[level];
  const message = messages[level];

  // NEW v2.0: Show threshold source for transparency
  const sourceInfo = thresholdSource ? `${colors.gray}[${thresholdSource}]${colors.reset}` : '';

  return {
    console: `${color}${symbol} ${relativePath}${colors.reset} (${lines} lines) ${sourceInfo}\n   ${message}`,
    plain: `${symbol} ${relativePath} (${lines} lines) [${thresholdSource}] - ${message}`,
  };
}

/**
 * Log alert to file
 */
function logAlert(message) {
  if (!CONFIG.logFile) return;

  const timestamp = new Date().toISOString();
  const logEntry = `[${timestamp}] ${message}\n`;

  fs.appendFileSync(CONFIG.logFile, logEntry);
}

/**
 * Send desktop notification (currently disabled in favor of alert files)
 */
function sendNotification(title, message) {
  // Desktop notifications are disabled for now.
  return;
}

/**
 * Write alert to JSON file for assistant-visible notifications
 */
function writeAlertForAssistant(filePath, lines, level, lineGrowth) {
  try {
    const alertsFile = path.join(__dirname, 'watcher-alerts.json');

    // Read existing alerts
    let alerts = [];
    try {
      const data = fs.readFileSync(alertsFile, 'utf8');
      alerts = JSON.parse(data);
    } catch (e) {
      // File doesn't exist or is invalid, start fresh
      alerts = [];
    }

    // Add new alert
    const alert = {
      timestamp: new Date().toISOString(),
      file: filePath,
      lines: lines,
      level: level,
      lineGrowth: lineGrowth,
      read: false
    };

    alerts.push(alert);

    // Keep only last 50 alerts to prevent file bloat
    if (alerts.length > 50) {
      alerts = alerts.slice(-50);
    }

    // Write back to file
    fs.writeFileSync(alertsFile, JSON.stringify(alerts, null, 2));
  } catch (error) {
    // Silent fail - don't crash watcher if alert logging fails
  }
}

/**
 * Handle file change event
 */
function handleFileChange(eventType, filePath) {
  if (!filePath || shouldIgnore(filePath)) return;

  // Only check relevant file types
  const fileType = getFileType(filePath);
  if (fileType === 'default' && !filePath.match(/\.(tsx?|jsx?|py)$/)) return;

  // Check if file exists (might be deleted)
  if (!fs.existsSync(filePath)) return;

  // Analyze file
  const analysis = analyzeFile(filePath);

  // Check if this is an edit to a problematic file (reminder feature)
  if (eventType === 'change' && problematicFiles.has(filePath)) {
    const tracked = problematicFiles.get(filePath);
    const now = Date.now();
    const timeSinceLastReminder = now - (tracked.lastReminded || 0);

    // Severity-based reminder frequency (for vibe coders who add lots of code quickly)
    const reminderIntervals = {
      critical: 300000,   // 5 minutes - CRITICAL files need frequent reminders!
      alert: 900000,      // 15 minutes - ALERT files
      warning: 1800000,   // 30 minutes - WARNING files
    };

    const reminderInterval = reminderIntervals[tracked.level] || 3600000;

    // Also remind if file grew significantly (50+ lines) - catches vibe coders!
    const lineGrowth = analysis.lines - tracked.lines;
    const significantGrowth = lineGrowth >= 50;

    if (timeSinceLastReminder > reminderInterval || significantGrowth) {
      const relativePath = path.relative(CONFIG.watchDir, filePath);
      const levelEmoji = {
        critical: '🛑',
        alert: '🚨',
        warning: '⚠️',
      };

      console.log('');
      console.log(`${colors.cyan}╔════════════════════════════════════════════════════════════╗${colors.reset}`);
      console.log(`${colors.cyan}║${colors.reset}  ⚡ Refactor Alert: Editing Problematic File              ${colors.cyan}║${colors.reset}`);
      console.log(`${colors.cyan}╚════════════════════════════════════════════════════════════╝${colors.reset}`);
      console.log('');
      console.log(`${colors.yellow}You're editing: ${relativePath}${colors.reset}`);
      console.log(`${colors.yellow}Current size: ${analysis.lines} lines (${tracked.level.toUpperCase()})${colors.reset}`);

      // Show why we're reminding
      if (significantGrowth) {
        console.log(`${colors.red}⚠️  File grew by ${lineGrowth} lines since last check!${colors.reset}`);
      }

      console.log('');
      console.log(`${colors.cyan}💡 Tip: Consider refactoring this file before adding more code.${colors.reset}`);
      console.log(`${colors.cyan}   Ask your coding assistant: "Help me refactor ${relativePath}"${colors.reset}`);
      console.log('');

      // Write alert to JSON file for assistant-visible notifications
      writeAlertForAssistant(relativePath, analysis.lines, tracked.level, significantGrowth ? lineGrowth : null);

      tracked.lastReminded = now;
    }
  }

  // Track problematic files for future reminders
  if (analysis.level !== 'ok') {
    if (!problematicFiles.has(filePath)) {
      problematicFiles.set(filePath, {
        level: analysis.level,
        lines: analysis.lines,
        lastReminded: 0,
      });
    } else {
      // Update the tracked info
      const tracked = problematicFiles.get(filePath);
      tracked.level = analysis.level;
      tracked.lines = analysis.lines;
    }
  } else if (problematicFiles.has(filePath)) {
    // File improved! Remove from tracking
    problematicFiles.delete(filePath);
  }

  // Only report warnings and above
  if (analysis.level === 'ok') {
    if (!CONFIG.quiet) {
      console.log(`${colors.gray}✓ ${path.relative(CONFIG.watchDir, filePath)} (${analysis.lines} lines)${colors.reset}`);
    }
    return;
  }

  // Format and display alert
  const alert = formatAlert(filePath, analysis);
  console.log(alert.console);

  // Log to file
  logAlert(alert.plain);

  // Send notification for critical files
  if (analysis.level === 'critical') {
    sendNotification('Critical File Size', alert.plain);
  }
}

/**
 * Scan directory recursively
 */
function scanDirectory(dir) {
  if (shouldIgnore(dir)) return;

  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      if (entry.isDirectory()) {
        scanDirectory(fullPath);
      } else if (entry.isFile()) {
        handleFileChange('add', fullPath);
      }
    }
  } catch (error) {
    // Skip directories we can't read
  }
}

/**
 * Watch directory using native fs.watch (no dependencies)
 */
function watchDirectory(dir) {
  if (shouldIgnore(dir)) return;

  try {
    // Watch this directory
    fs.watch(dir, { recursive: false }, (eventType, filename) => {
      if (!filename) return;

      const fullPath = path.join(dir, filename);

      // Small delay to ensure file is written
      setTimeout(() => {
        handleFileChange(eventType, fullPath);
      }, 100);
    });

    // Recursively watch subdirectories
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isDirectory()) {
        const subDir = path.join(dir, entry.name);
        watchDirectory(subDir);
      }
    }
  } catch (error) {
    // Skip directories we can't watch
  }
}

/**
 * Print statistics
 */
function printStats() {
  const runtime = Math.floor((Date.now() - stats.startTime) / 1000);
  const minutes = Math.floor(runtime / 60);
  const seconds = runtime % 60;

  console.log('\n' + '═'.repeat(60));
  console.log(`${colors.cyan}File Watcher Statistics${colors.reset}`);
  console.log('═'.repeat(60));
  console.log(`Runtime: ${minutes}m ${seconds}s`);
  console.log(`Files checked: ${stats.filesChecked}`);
  console.log(`${colors.red}Critical files (must refactor): ${stats.critical}${colors.reset}`);
  console.log(`${colors.yellow}Alert files (should refactor): ${stats.alerts}${colors.reset}`);
  console.log(`${colors.yellow}Warning files (watch closely): ${stats.warnings}${colors.reset}`);
  console.log('═'.repeat(60) + '\n');
}

/**
 * Main function
 */
function main() {
  console.log(`${colors.cyan}╔════════════════════════════════════════════════════════════╗${colors.reset}`);
  console.log(`${colors.cyan}║${colors.reset}  Code Refactoring Skill - Real-time File Watcher         ${colors.cyan}║${colors.reset}`);
  console.log(`${colors.cyan}╚════════════════════════════════════════════════════════════╝${colors.reset}\n`);

  console.log(`Watching directory: ${colors.green}${CONFIG.watchDir}${colors.reset}`);
  console.log(`Quiet mode: ${CONFIG.quiet ? 'Yes' : 'No'}`);
  console.log(`Notifications: ${CONFIG.notify ? 'Enabled' : 'Disabled'}`);
  console.log(`Log file: ${CONFIG.logFile || 'None'}`);
  console.log(`Custom threshold: ${CONFIG.threshold} lines\n`);

  console.log(`${colors.gray}Performing initial scan...${colors.reset}\n`);

  // Initial scan
  scanDirectory(CONFIG.watchDir);

  console.log(`\n${colors.green}✓ Initial scan complete!${colors.reset}`);
  printStats();

  console.log(`${colors.cyan}👀 Watching for file changes... (Press Ctrl+C to stop)${colors.reset}\n`);

  // Start watching
  watchDirectory(CONFIG.watchDir);

  // Print stats every 5 minutes
  setInterval(printStats, 5 * 60 * 1000);

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    console.log('\n\n' + colors.cyan + 'Shutting down file watcher...' + colors.reset);
    printStats();
    process.exit(0);
  });
}

// Start the watcher
main();
