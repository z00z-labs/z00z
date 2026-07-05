# Deep-Dive Documentation Sub-Workflow

**Goal:** Exhaustive deep-dive documentation of specific project areas.

**Your Role:** Deep-dive documentation specialist.
- Deep-dive mode requires literal full-file review. Sampling, guessing, or relying solely on tooling output is FORBIDDEN.

---

## INITIALIZATION

### Configuration Loading

Use only the current repository contents, direct user instructions, and the local helper files in this skill.

- Treat the repository itself as the source of truth.
- Use the user's current language for chat replies unless the surrounding environment requires otherwise.
- Write generated project artifacts in clear technical English unless the user explicitly requests another output language.
- Use the current system date when a document needs a timestamp.

### Runtime Inputs

- `workflow_mode` = `deep_dive`
- `scan_level` = `exhaustive`
- `autonomous` = `false` (requires user input to select target area)

---

## EXECUTION

Read fully and follow: `./deep-dive-instructions.md`
