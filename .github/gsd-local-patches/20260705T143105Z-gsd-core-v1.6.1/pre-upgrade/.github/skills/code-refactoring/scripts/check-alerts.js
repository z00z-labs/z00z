#!/usr/bin/env node

/**
 * Code Refactoring Skill - Alert Checker
 * Reads watcher-alerts.json and displays unread alerts
 */

const fs = require('fs');
const path = require('path');

// Configuration
const ALERTS_FILE = path.join(__dirname, 'watcher-alerts.json');

// Colors for console output
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  green: '\x1b[32m',
  cyan: '\x1b[36m',
  gray: '\x1b[90m',
  bold: '\x1b[1m',
};

/**
 * Read alerts from JSON file
 */
function readAlerts() {
  try {
    if (!fs.existsSync(ALERTS_FILE)) {
      return null;
    }

    const data = fs.readFileSync(ALERTS_FILE, 'utf8');
    return JSON.parse(data);
  } catch (error) {
    console.error(`${colors.red}Error reading alerts file:${colors.reset}`, error.message);
    return null;
  }
}

/**
 * Write alerts back to JSON file
 */
function writeAlerts(alerts) {
  try {
    fs.writeFileSync(ALERTS_FILE, JSON.stringify(alerts, null, 2));
  } catch (error) {
    console.error(`${colors.red}Error writing alerts file:${colors.reset}`, error.message);
  }
}

/**
 * Format and display alerts
 */
function displayAlerts() {
  const alerts = readAlerts();

  // No alerts file
  if (alerts === null) {
    console.log(`${colors.yellow}No refactoring alerts found.${colors.reset}`);
    console.log(`${colors.gray}File watcher may not be running or no alerts have been generated yet.${colors.reset}\n`);
    console.log(`${colors.cyan}💡 To start monitoring: Run /start-watcher${colors.reset}`);
    console.log(`${colors.cyan}💡 To check watcher status: Run /watcher-status${colors.reset}`);
    return;
  }

  // Filter unread alerts
  const unreadAlerts = alerts.filter(alert => !alert.read);

  // No unread alerts
  if (unreadAlerts.length === 0) {
    console.log(`${colors.green}✅ No unread refactoring alerts.${colors.reset}`);
    console.log(`${colors.gray}All monitored files are within acceptable size limits or alerts have been reviewed.${colors.reset}\n`);
    console.log(`${colors.cyan}Monitoring ${alerts.length} file check(s) across your codebase.${colors.reset}`);
    return;
  }

  // Group alerts by severity
  const grouped = {
    critical: unreadAlerts.filter(a => a.level === 'critical'),
    alert: unreadAlerts.filter(a => a.level === 'alert'),
    warning: unreadAlerts.filter(a => a.level === 'warning'),
  };

  // Display header
  console.log('');
  console.log(`${colors.cyan}╔════════════════════════════════════════════════════════════╗${colors.reset}`);
  console.log(`${colors.cyan}║${colors.reset}  🚨 Code Refactoring Alerts                               ${colors.cyan}║${colors.reset}`);
  console.log(`${colors.cyan}╚════════════════════════════════════════════════════════════╝${colors.reset}`);
  console.log('');

  // Display critical alerts
  if (grouped.critical.length > 0) {
    console.log(`${colors.red}${colors.bold}CRITICAL (🛑 Must refactor immediately):${colors.reset}`);
    grouped.critical.forEach(alert => {
      const growth = alert.lineGrowth ? ` - grew by ${alert.lineGrowth} lines` : '';
      console.log(`${colors.red}  • ${alert.file}${colors.reset} (${alert.lines} lines)${growth}`);
    });
    console.log('');
  }

  // Display alert-level alerts
  if (grouped.alert.length > 0) {
    console.log(`${colors.yellow}${colors.bold}ALERT (⚠️  Should refactor before adding more):${colors.reset}`);
    grouped.alert.forEach(alert => {
      const growth = alert.lineGrowth ? ` - grew by ${alert.lineGrowth} lines` : '';
      console.log(`${colors.yellow}  • ${alert.file}${colors.reset} (${alert.lines} lines)${growth}`);
    });
    console.log('');
  }

  // Display warnings
  if (grouped.warning.length > 0) {
    console.log(`${colors.cyan}${colors.bold}WARNING (💡 Watch closely):${colors.reset}`);
    grouped.warning.forEach(alert => {
      const growth = alert.lineGrowth ? ` - grew by ${alert.lineGrowth} lines` : '';
      console.log(`${colors.cyan}  • ${alert.file}${colors.reset} (${alert.lines} lines)${growth}`);
    });
    console.log('');
  }

  // Display summary
  console.log(`${colors.bold}Total: ${unreadAlerts.length} unread alert(s)${colors.reset}`);
  console.log('');

  // Display suggestions
  console.log(`${colors.cyan}💡 Next steps:${colors.reset}`);
  console.log(`${colors.cyan}   • To refactor a file: Say "Help me refactor [filename]"${colors.reset}`);
  console.log(`${colors.cyan}   • To stop alerts: Run /stop-watcher${colors.reset}`);
  console.log(`${colors.cyan}   • To scan all files: Run /scan-code-size${colors.reset}`);
  console.log('');

  // Mark alerts as read
  alerts.forEach(alert => {
    if (!alert.read && unreadAlerts.includes(alert)) {
      alert.read = true;
    }
  });

  writeAlerts(alerts);
}

// Run the checker
displayAlerts();
