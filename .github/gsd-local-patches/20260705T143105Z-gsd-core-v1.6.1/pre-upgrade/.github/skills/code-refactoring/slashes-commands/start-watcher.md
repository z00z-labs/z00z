---
description: Start the code-refactoring watcher and prepare background alerts
---

# Start Watcher

Start the watcher for the current project.

```bash
SKILL_ROOT="${CODE_REFACTORING_SKILL_ROOT:-$PWD/.github/skills/code-refactoring}"
node "$SKILL_ROOT/scripts/start-watcher-interactive.js"
```

After the scan completes, summarize the critical, alert, and warning counts and ask whether the user wants to refactor now or later.

Before later responses, you may optionally check for unread alerts with:

```bash
SKILL_ROOT="${CODE_REFACTORING_SKILL_ROOT:-$PWD/.github/skills/code-refactoring}"
cat "$SKILL_ROOT/scripts/watcher-alerts.json"
```
