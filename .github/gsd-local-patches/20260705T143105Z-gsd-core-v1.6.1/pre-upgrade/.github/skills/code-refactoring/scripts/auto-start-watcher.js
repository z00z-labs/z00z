#!/usr/bin/env node

/**
 * Code Refactoring Skill - Universal Auto-start Helper
 * Silently starts watcher if not already running
 * Works on Windows, Linux, macOS, WSL
 * Designed for use with generic session-start hooks
 */

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const SCRIPT_DIR = __dirname;
const PID_FILE = path.join(SCRIPT_DIR, 'watcher.pid');
const LOG_FILE = path.join(SCRIPT_DIR, 'watcher.log');
const WATCH_DIR = process.argv[2] || process.cwd();

/**
 * Check if process is running by PID
 */
function isProcessRunning(pid) {
  try {
    // Sending signal 0 checks if process exists without actually signaling it
    process.kill(pid, 0);
    return true;
  } catch (e) {
    return false;
  }
}

/**
 * Main function
 */
function main() {
  // Check if already running
  if (fs.existsSync(PID_FILE)) {
    try {
      const oldPid = parseInt(fs.readFileSync(PID_FILE, 'utf8').trim());
      if (isProcessRunning(oldPid)) {
        // Already running, exit silently
        process.exit(0);
      } else {
        // Stale PID file, clean up
        fs.unlinkSync(PID_FILE);
      }
    } catch (error) {
      // Error reading PID file, clean up
      try {
        fs.unlinkSync(PID_FILE);
      } catch (e) {
        // Ignore cleanup errors
      }
    }
  }

  // Start the watcher in detached mode
  const watcher = spawn(
    'node',
    [path.join(SCRIPT_DIR, 'file-watcher.js'), WATCH_DIR, '--quiet'],
    {
      detached: true,
      stdio: ['ignore', 'ignore', 'ignore'], // Fully detached
      cwd: SCRIPT_DIR,
    }
  );

  // Unref so parent can exit
  watcher.unref();

  // Save PID
  try {
    fs.writeFileSync(PID_FILE, watcher.pid.toString());
  } catch (error) {
    // Ignore save errors
  }

  // Exit immediately
  process.exit(0);
}

// Run with error handling
try {
  main();
} catch (error) {
  // Exit silently on any error
  process.exit(0);
}
