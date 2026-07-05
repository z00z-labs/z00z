---
name: z00z-chat-init
description: 'Workspace startup skill for the z00z repository. Use at the beginning of any z00z task to load the full repository instructions, restate and enforce every @MUST rule from .github/copilot-instructions.md, load the full .github/requirements/Z00Z_DESIGN_FOUNDATION.md, consult nested .github instructions and skills as needed, and always finish the user-facing cycle with ./scripts/play_tone.sh. Trigger words: z00z init, z00z chat init, start z00z task, load z00z rules, repo startup.'
argument-hint: '[task or scope]'
---

# Z00Z Chat Init

📌 Use this skill at the start of any z00z task to lock the assistant onto repository rules before planning, editing, reviewing, testing, or running commands.

## When to Use

📌 Invoke this skill when the user starts a new z00z task and wants the assistant to initialize repository context first.

- Start of a new implementation, refactor, review, debug, or documentation task in this repository.
- Any request that mentions loading project rules, startup context, repository constraints, or chat initialization.
- Any task that may touch Rust code, repository documentation, git workflow, testing, or project-wide conventions.

## Do Not Use

📌 Do not use this skill for non-z00z repositories or for generic programming tasks that do not need repository-specific startup rules.

- Do not use it as a replacement for a task-specific skill such as code review, refactoring, or documentation generation.
- Do not skip task-specific skills after initialization when a narrower skill clearly matches the user request.

## Startup Sequence

📌 Follow this sequence in order before substantial work.

1. Read `.github/copilot-instructions.md` in full from top to bottom, not only the header or summary.
2. Read `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` in full from top to bottom, not only the header or summary.
3. Load any additional task-relevant files under `.github/instructions/`, `.github/skills/`, `.github/prompts/`, and related repository docs before making assumptions.
4. Inspect the existing code, tests, docs, and scripts that are directly relevant to the user request.
5. Restate the active hard constraints internally and apply them during the entire task.
6. Keep user-facing chat responses in Russian Cyrillic, while keeping repository artifacts, code, comments, documentation, commit messages, and technical content in English.
7. End every user-facing completion cycle with `./scripts/play_tone.sh` when feasible.

## Mandatory Rules From .github/copilot-instructions.md

📌 Treat every item in this section as non-negotiable for all z00z work.

1. English-only repository artifacts: all code, comments, documentation, commit messages, PR text, configuration, errors, and logs must be in English.
2. Safe file operations only: never use destructive delete commands such as `rm`, `rm -r`, or `rm -rf` without explicit user confirmation; prefer `trash-put` or `gio trash` on Linux.
3. Version and git operations must use `./.github/skills/z00z-git-versioning/scripts/version-manager.sh` when version updates or managed git/version flows are requested.
4. `crates/z00z_crypto/tari/` is read-only vendor code and must never be modified.
5. At the end of each Copilot cycle, when user interaction is expected, run `./scripts/play_tone.sh`.
6. Markdown documentation must follow repository documentation standards, including emoji-led paragraphs and ISO 8601 dates.
7. Naming conventions are mandatory: types use `PascalCase`, functions use `snake_case`, booleans use `is_*` or `has_*`, constants use `SCREAMING_SNAKE_CASE`, and modules use `snake_case`.
8. No identifier may exceed five words after splitting on underscores, hyphens, and camel or Pascal case boundaries; existing violations must be reported and scheduled for rename.

## Mandatory Rules From Z00Z Design Foundation

📌 Treat the full `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` file as a hard gate, and at minimum enforce the following rules on every relevant task.

1. ONE SOURCE OF TRUTH: business logic must use `z00z_utils` abstractions for file I/O, serialization, time, logging, configuration, and random number generation.
2. Trait-based dependency injection: inject external dependencies through traits and use deterministic real implementations for tests when needed.
3. Zero-overhead abstractions: prefer static dispatch and zero-cost patterns over runtime indirection when possible.
4. Cryptographic domain separation is mandatory for every crypto operation.
5. Vendor isolation is mandatory: do not modify Tari vendor code.
6. English-only repository content remains mandatory.
7. Use the correct concurrency model: `rayon` for CPU-bound work, `tokio` for I/O-bound work, and `spawn_blocking` for CPU work inside async contexts.
8. Error handling must use typed errors such as `thiserror`; production code must not rely on `unwrap()`, undocumented `expect()`, or panic-driven control flow.
9. Public APIs require Rustdoc with runnable examples where applicable.
10. Tests, formatting, linting, and documentation checks are required to the extent relevant to the change.
11. Naming, identifier-length, and grouped-import requirements must be enforced.
12. Sensitive data must use the approved wrappers and must never be leaked via logs or errors.

## Execution Rules

📌 Use these execution rules after initialization.

1. Prefer existing project patterns over inventing new ones.
2. Build context from the codebase before proposing or applying changes.
3. Keep changes minimal, focused, and consistent with existing architecture.
4. Do not modify unrelated files or revert unrelated user changes.
5. Use task-specific skills from `.github/skills/` when they are a better fit after startup is complete.
6. If a request touches git versioning, use the repository version manager workflow instead of ad hoc git flows.
7. If a request touches protected or vendor code, stop and redirect the change to supported extension points.

## Completion Gate

📌 Before ending the current cycle, confirm that these conditions hold.

- The full `.github/copilot-instructions.md` file was read.
- The full `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` file was read.
- Task-relevant nested `.github` instructions, prompts, or skills were loaded when needed.
- User-facing chat stayed in Russian Cyrillic.
- Repository artifacts remained in English.
- No protected vendor files were changed.
- `./scripts/play_tone.sh` is scheduled or executed for the end of the cycle.

## Example Prompts

📌 Use prompts like these to trigger this skill.

- `Initialize this z00z task before reviewing the wallet crate.`
- `Load z00z startup rules, then help me refactor storage code.`
- `Start this repository task with full z00z constraints.`
- `Run z00z chat init, then inspect the failing simulator tests.`