#!/bin/bash

# Code Refactoring Skill - Start File Watcher (Unix/Linux/macOS)
# Usage: ./start-watcher.sh [directory] [options]

set -e

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Default directory
WATCH_DIR="${1:-$PWD}"

# PID and log files
PID_FILE="$SCRIPT_DIR/watcher.pid"
LOG_FILE="$SCRIPT_DIR/watcher.log"

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}  Starting Code Refactoring File Watcher...                 ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if already running
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p "$OLD_PID" > /dev/null 2>&1; then
        echo -e "${RED}[ERROR] File watcher is already running with PID: $OLD_PID${NC}"
        echo ""
        echo "To stop it, run: ./stop-watcher.sh"
        echo "To check status, run: ./watcher-status.sh"
        echo ""
        exit 1
    else
        echo -e "${YELLOW}[INFO] Cleaning up stale PID file...${NC}"
        rm -f "$PID_FILE"
    fi
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}[ERROR] Node.js is not installed or not in PATH!${NC}"
    echo ""
    echo "Please install Node.js from: https://nodejs.org/"
    echo ""
    exit 1
fi

echo -e "${GREEN}[INFO] Starting file watcher for directory: $WATCH_DIR${NC}"
echo -e "${GREEN}[INFO] Log file: $LOG_FILE${NC}"
echo ""

# Start the watcher in background
nohup node "$SCRIPT_DIR/file-watcher.js" "$WATCH_DIR" "${@:2}" >> "$LOG_FILE" 2>&1 &
WATCHER_PID=$!

# Save PID
echo "$WATCHER_PID" > "$PID_FILE"

# Wait a moment to ensure it started successfully
sleep 1

# Check if still running
if ps -p "$WATCHER_PID" > /dev/null 2>&1; then
    echo -e "${GREEN}[SUCCESS] File watcher started successfully!${NC}"
    echo -e "${GREEN}[INFO] PID: $WATCHER_PID${NC}"
    echo ""
    echo "To view live output: tail -f $LOG_FILE"
    echo "To stop watcher: ./stop-watcher.sh"
    echo "To check status: ./watcher-status.sh"
    echo ""
else
    echo -e "${RED}[ERROR] Failed to start file watcher!${NC}"
    echo "Check the log file for details: $LOG_FILE"
    echo ""
    rm -f "$PID_FILE"
    exit 1
fi
