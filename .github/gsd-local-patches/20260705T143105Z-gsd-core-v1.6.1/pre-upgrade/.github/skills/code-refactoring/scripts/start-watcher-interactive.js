#!/usr/bin/env node

/**
 * Code Refactoring Skill - Interactive Watcher Starter
 * Shows initial scan, asks user preference, then starts background watcher
 * Used by /start-watcher command templates
 */

const fs = require('fs');
const path = require('path');
const { spawn, execSync } = require('child_process');

const SCRIPT_DIR = __dirname;
const PID_FILE = path.join(SCRIPT_DIR, 'watcher.pid');
const WATCHER_SCRIPT = path.join(SCRIPT_DIR, 'file-watcher.js');

// Auto-detect src directory
let WATCH_DIR = process.cwd();
const srcPath = path.join(WATCH_DIR, 'src');
const parentSrcPath = path.join(WATCH_DIR, '..', 'src');

if (fs.existsSync(srcPath)) {
  WATCH_DIR = srcPath;
  console.log(`📁 Detected src/ directory - watching: ${WATCH_DIR}`);
} else if (fs.existsSync(parentSrcPath)) {
  WATCH_DIR = parentSrcPath;
  console.log(`📁 Detected src/ directory - watching: ${WATCH_DIR}`);
} else {
  console.log(`📁 No src/ directory found - watching: ${WATCH_DIR}`);
}

console.log('');
console.log('🔍 Running initial scan...');
console.log('');

// Run initial scan synchronously to show results
try {
  execSync(`node "${WATCHER_SCRIPT}" "${WATCH_DIR}"`, {
    stdio: 'inherit',
    timeout: 10000, // 10 second timeout
  });
} catch (error) {
  // Timeout or error - that's OK, continue
}

console.log('');
console.log('━'.repeat(60));
console.log('');
console.log('🎯 Scan complete!');
console.log('');

// Start persistent background watcher with VISIBLE output
const LOG_FILE = path.join(SCRIPT_DIR, 'watcher-alerts.log');
const logStream = fs.openSync(LOG_FILE, 'a');

const watcher = spawn(
  'node',
  [WATCHER_SCRIPT, WATCH_DIR], // Alerts are written to files for the assistant to read
  {
    detached: true,
    stdio: ['ignore', logStream, logStream], // Log to file so we can see alerts!
    cwd: SCRIPT_DIR,
  }
);

// Unref so parent can exit
watcher.unref();

// Save PID
try {
  fs.writeFileSync(PID_FILE, watcher.pid.toString());
  console.log(`✅ Background watcher started (PID: ${watcher.pid})`);
} catch (error) {
  console.log('✅ Background watcher started');
}

console.log('   - Will alert you when editing problematic files');
console.log('   - Use /watcher-status to check status');
console.log('   - Use /stop-watcher to stop monitoring');
console.log('');

// Exit after starting background process
process.exit(0);
