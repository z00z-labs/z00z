# Code Refactoring Command Templates

These files are optional command templates for environments that support custom prompt commands or slash-style command shortcuts.

## Included Templates

- `start-watcher.md`
- `stop-watcher.md`
- `scan-code-size.md`
- `check-refactor-alerts.md`

## How To Use Them

Copy them into whatever command directory your coding environment supports.

Common patterns:

- project-local commands directory
- editor-specific commands directory
- shared tooling directory for your team

Each template resolves the skill location through:

1. `CODE_REFACTORING_SKILL_ROOT`
2. repository-local `.github/skills/code-refactoring`

## Command Root Pattern

```bash
SKILL_ROOT="${CODE_REFACTORING_SKILL_ROOT:-$PWD/.github/skills/code-refactoring}"
```

If your commands run from somewhere other than repository root, set `CODE_REFACTORING_SKILL_ROOT` explicitly.

## Notes

- The watcher is optional.
- The templates do not depend on a specific assistant product.
- The templates are safe to adapt to your editor, agent runner, or custom workflow.

## Related Files

- `../SKILL.md`
- `../COMMANDS_REFERENCE.md`
- `../scripts/`
