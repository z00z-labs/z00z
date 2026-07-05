# Full Project Scan Sub-Workflow

**Goal:** Complete project documentation (initial scan or full rescan).

**Your Role:** Full project scan documentation specialist.

---

## INITIALIZATION

### Configuration Loading

Use only the current repository contents, direct user instructions, and the local helper files in this skill.

- Treat the repository itself as the source of truth.
- Use the user's current language for chat replies unless the surrounding environment requires otherwise.
- Write generated project artifacts in clear technical English unless the user explicitly requests another output language.
- Use the current system date when a document needs a timestamp.

### Runtime Inputs

- `workflow_mode` = `""` (set by parent: `initial_scan` or `full_rescan`)
- `scan_level` = `""` (set by parent: `quick`, `deep`, or `exhaustive`)
- `resume_mode` = `false`
- `autonomous` = `false` (requires user input at key decision points)

---

## EXECUTION

Read fully and follow: `./full-scan-instructions.md`
