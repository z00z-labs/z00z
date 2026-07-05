#!/bin/bash

# Code Refactoring Skill - Auto-start Helper (Unix/Linux/macOS)
# Silently starts watcher if not already running
# Designed for use with generic session-start hooks

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# PID and log files
PID_FILE="$SCRIPT_DIR/watcher.pid"
LOG_FILE="$SCRIPT_DIR/watcher.log"

# Default directory
WATCH_DIR="${1:-$PWD}"

# Check if already running
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p "$OLD_PID" > /dev/null 2>&1; then
        # Already running, exit silently
        exit 0
    else
        # Stale PID file, clean up
        rm -f "$PID_FILE"
    fi
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    # Node.js not installed, exit silently
    exit 0
fi

# Start watcher silently in background
nohup node "$SCRIPT_DIR/file-watcher.js" "$WATCH_DIR" --quiet >> "$LOG_FILE" 2>&1 &
WATCHER_PID=$!

# Save PID
echo "$WATCHER_PID" > "$PID_FILE"

# Exit silently
exit 0
