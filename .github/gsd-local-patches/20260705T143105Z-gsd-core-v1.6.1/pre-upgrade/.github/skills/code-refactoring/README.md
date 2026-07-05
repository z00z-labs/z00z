# Code Refactoring Skill

Language-agnostic refactoring guidance for any coding assistant or developer workflow.

## Overview

This skill helps prevent accidental complexity by:

- checking file size before new edits
- suggesting extraction points as files grow
- watching for common complexity patterns
- supporting optional background alerts through helper scripts

It is designed to work in repository-local setups and in custom shared skill installations.

## Quick Start

Use the skill directly from this repository:

```bash
cd .github/skills/code-refactoring/scripts
npm install
```

If you keep skills in another location, set a root override:

```bash
export CODE_REFACTORING_SKILL_ROOT="/path/to/code-refactoring"
```

The documentation and helper commands assume this resolution order:

1. `CODE_REFACTORING_SKILL_ROOT`
2. repository-local `.github/skills/code-refactoring`

## Directory Layout

```text
code-refactoring/
├── SKILL.md
├── REFERENCE.md
├── FORMS.md
├── README.md
├── CHANGELOG.md
├── scripts/
│   ├── file-watcher.js
│   ├── check-alerts.js
│   ├── start-watcher-interactive.js
│   ├── auto-start-watcher.js
│   ├── start-watcher.sh
│   ├── stop-watcher.sh
│   ├── check-size.sh
│   ├── analyze-codebase.sh
│   └── package.json
└── slashes-commands/
    ├── start-watcher.md
    ├── stop-watcher.md
    ├── scan-code-size.md
    └── check-refactor-alerts.md
```

## Core Usage

Typical usage patterns:

- ask for a refactor plan before adding more code to a large file
- scan a directory for oversized files before a cleanup sprint
- start the watcher if you want background reminders while editing

Examples:

```text
/code-refactoring check whether src/components/Dashboard.tsx should be split first
```

```text
/code-refactoring audit src for the top 5 refactor hotspots
```

```text
Before editing this module, check whether it is already too large.
```

## Helper Commands

Repository-local commands:

```bash
bash .github/skills/code-refactoring/scripts/check-size.sh src/components/Dashboard.tsx
bash .github/skills/code-refactoring/scripts/analyze-codebase.sh src
node .github/skills/code-refactoring/scripts/check-alerts.js
node .github/skills/code-refactoring/scripts/start-watcher-interactive.js
```

Shared-install commands using the environment override:

```bash
bash "$CODE_REFACTORING_SKILL_ROOT/scripts/check-size.sh" src/components/Dashboard.tsx
node "$CODE_REFACTORING_SKILL_ROOT/scripts/check-alerts.js"
```

## Optional Background Watcher

The watcher is optional. Use it when you want reminders while editing files.

Behavior:

- scans the watched directory for large JavaScript, TypeScript, JSX, TSX, and Python files
- writes alerts to `watcher-alerts.json`
- prints reminders when problematic files keep growing

Recommended workflow:

1. Start the watcher.
2. Edit code normally.
3. Check alerts before large changes or at natural breakpoints.
4. Stop the watcher when you are done.

## Session Hooks

If your coding environment supports session-start hooks, you can auto-start the watcher with:

```bash
SKILL_ROOT="${CODE_REFACTORING_SKILL_ROOT:-$PWD/.github/skills/code-refactoring}"
node "$SKILL_ROOT/scripts/auto-start-watcher.js"
```

The exact hook registration mechanism depends on the environment you use.

## Command Integration

The `slashes-commands/` directory contains optional prompt-command templates.
They are written to be adaptable to any environment that can run shell commands.

Use a project-specific commands directory of your choice, for example:

- `.github/commands/`
- `.tools/commands/`
- another command directory supported by your editor or assistant

## What Changed From Provider-Specific Versions

This repository version is intentionally coder-agnostic:

- no assistant-product assumptions
- no hard dependency on product-specific hidden directories
- no marketplace-specific install instructions
- generic `SKILL_ROOT` resolution for commands and scripts

## Troubleshooting

If helper scripts do not work:

1. Check that Node.js is installed for watcher-related commands.
2. Confirm `npm install` was run in `scripts/`.
3. Verify `CODE_REFACTORING_SKILL_ROOT` if you are not using the repository-local path.
4. On Unix-like systems, ensure shell scripts are executable.

Useful checks:

```bash
ls -la .github/skills/code-refactoring/scripts
cat .github/skills/code-refactoring/scripts/watcher.log
cat .github/skills/code-refactoring/scripts/watcher-alerts.json
```

## Related Files

- `SKILL.md` for trigger logic and workflow
- `REFERENCE.md` for detailed patterns
- `FORMS.md` for templates and checklists
- `COMMANDS_REFERENCE.md` for the watcher command set

## Attribution

Created by Madina Gbotoe.

License: Creative Commons Attribution 4.0 International (CC BY 4.0).
