---
description: Check unread refactoring alerts from the background watcher
---

# Check Refactor Alerts

Read unread alerts from the watcher.

```bash
SKILL_ROOT="${CODE_REFACTORING_SKILL_ROOT:-$PWD/.github/skills/code-refactoring}"
node "$SKILL_ROOT/scripts/check-alerts.js"
```

If alerts exist, summarize them and ask whether the user wants to refactor now, review details, or dismiss them for later.
