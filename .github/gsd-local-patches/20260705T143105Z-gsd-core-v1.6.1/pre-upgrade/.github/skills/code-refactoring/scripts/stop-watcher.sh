#!/bin/bash

# Code Refactoring Skill - Stop File Watcher (Unix/Linux/macOS)

set -e

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# PID file
PID_FILE="$SCRIPT_DIR/watcher.pid"

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}  Stopping Code Refactoring File Watcher...                 ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if PID file exists
if [ ! -f "$PID_FILE" ]; then
    echo -e "${YELLOW}[WARNING] File watcher is not running (no PID file found)${NC}"
    echo ""
    exit 0
fi

# Read PID
WATCHER_PID=$(cat "$PID_FILE")

# Check if process is running
if ps -p "$WATCHER_PID" > /dev/null 2>&1; then
    echo -e "${GREEN}[INFO] Found running watcher with PID: $WATCHER_PID${NC}"
    echo -e "${GREEN}[INFO] Stopping process...${NC}"

    # Kill the process
    kill "$WATCHER_PID" 2>/dev/null || kill -9 "$WATCHER_PID" 2>/dev/null

    # Wait for it to stop
    sleep 1

    if ps -p "$WATCHER_PID" > /dev/null 2>&1; then
        echo -e "${RED}[WARNING] Process still running, forcing termination...${NC}"
        kill -9 "$WATCHER_PID" 2>/dev/null || true
    fi

    echo -e "${GREEN}[SUCCESS] File watcher stopped successfully!${NC}"
else
    echo -e "${YELLOW}[WARNING] Process not found (may have already stopped)${NC}"
fi

# Clean up PID file
rm -f "$PID_FILE"

echo -e "${GREEN}[INFO] Cleanup complete${NC}"
echo ""
