---
description: Scan code for oversized files and save a report without starting persistent monitoring
---

# Scan Code Size

Run a one-time size scan.

```bash
SKILL_ROOT="${CODE_REFACTORING_SKILL_ROOT:-$PWD/.github/skills/code-refactoring}"
WATCH_DIR="."
if [ -d "./src" ]; then
  WATCH_DIR="./src"
elif [ -d "../src" ]; then
  WATCH_DIR="../src"
fi

REPORT_FILE="code-size-report-$(date +%Y%m%d-%H%M%S).txt"

echo "Scanning $WATCH_DIR for oversized files..."
timeout 10 node "$SKILL_ROOT/scripts/file-watcher.js" "$WATCH_DIR" 2>&1 | tee "$REPORT_FILE"
echo ""
echo "Scan complete. Full report saved to: $REPORT_FILE"
```
