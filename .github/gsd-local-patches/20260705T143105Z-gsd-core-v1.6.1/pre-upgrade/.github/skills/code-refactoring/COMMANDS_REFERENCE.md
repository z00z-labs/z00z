# Code Refactoring Commands Reference

This reference describes the optional command templates that ship with the skill.

## Available Commands

### `/start-watcher`

Starts the background watcher, runs an initial scan, and prepares alert tracking.

### `/stop-watcher`

Stops the watcher process and preserves existing alert history.

### `/scan-code-size`

Runs a one-time scan and saves a timestamped report without leaving a background process running.

### `/check-refactor-alerts`

Reads unread alert entries from `watcher-alerts.json` and shows the files that need attention.

## Typical Workflows

### Continuous Monitoring

1. Run `/start-watcher`.
2. Edit code normally.
3. Check or surface alerts at natural pauses.
4. Run `/stop-watcher` when done.

### One-Time Audit

1. Run `/scan-code-size`.
2. Review the generated report.
3. Refactor the biggest hotspots first.

## Path Resolution

The command templates should resolve the skill root using:

1. `CODE_REFACTORING_SKILL_ROOT`
2. repository-local `.github/skills/code-refactoring`

Example shell pattern:

```bash
SKILL_ROOT="${CODE_REFACTORING_SKILL_ROOT:-$PWD/.github/skills/code-refactoring}"
node "$SKILL_ROOT/scripts/check-alerts.js"
```

## Files Created By The Watcher

- `watcher.pid`
- `watcher.log`
- `watcher-alerts.json`
- `watcher-alerts.log`

These are stored under the skill's `scripts/` directory.

## FAQ

**Do I need the command templates to use the skill?**

No. The skill can still be invoked directly in normal assistant interactions.

**Do I need a specific editor or coding assistant?**

No. The command templates are environment-agnostic as long as your tool can run shell commands.

**Can I keep the watcher disabled?**

Yes. The watcher is optional. Use direct scans and manual checks if that fits your workflow better.
