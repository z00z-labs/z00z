# Project Skills Discovery

Before execution, check for project-defined instruction surfaces and apply the
relevant rules without inventing alias layers or parallel workflows.

**Discovery steps (shared across all GSD agents):**
1. Prefer live project-local surfaces first:
   - `.github/skills/`
   - `.github/prompts/`
   - `.github/hooks/`
   - `.github/agents/`
   - `.codex/AGENTS.md` when present
2. Then check live user-local Codex surfaces when the current runtime can use
   them:
   - `$HOME/.codex/skills/`
   - `$HOME/.codex/prompts/`
3. Then check project plugin and override surfaces if they are relevant to the
   current task:
   - `.github/plugins/*/skills/`
   - `.github/plugins/*/agents/`
   - `.github/plugins/*/commands/`
   - `.github/plugins/*/.codex-plugin/plugin.json`
   - `.github/plugins/*/.claude-plugin/plugin.json`
   - `.github/deep-wiki-local-overrides/skills/`
   - `.github/deep-wiki-local-overrides/agents/`
4. Use canonical paths only. Do not rely on alias names, shim names, prompt
   nicknames, or inferred virtual locations when a real path exists.
5. Read `SKILL.md` for each task-relevant skill. Read prompt files only when the
   user invoked that prompt or the current workflow explicitly depends on it.
6. Inspect hook files only when the task touches workflow automation, git
   enforcement, session state, or tool-triggered behavior.
7. Do not read agent definition bodies just to "discover" them when the current
   runtime auto-loads agent files. Read agent files directly only when the task
   explicitly targets those files or the runtime requires manual inspection.
8. Load specific `references/*.md`, `rules/*.md`, templates, or scripts only as
   needed during the current task.
9. Treat archive, cache, backup, and patch trees as non-authoritative unless
   the task explicitly targets them:
   - `.github/gsd-local-patches/`
   - `.agents/.install-backups/`
   - `$HOME/.codex/.tmp/`
   - `$HOME/.codex/plugins/cache/`
   - `$HOME/.codex/vendor_imports/`
10. When the same capability appears in multiple places, prefer the live
    project-local file over user-local copies, plugin cache copies, or archived
    patch snapshots.

**Application** — how to apply the loaded rules depends on the calling agent:
- Planners account for project skill patterns and conventions in the plan.
- Planners also record prompt, hook, and runtime-surface constraints when those
  constraints affect execution or verification.
- Executors follow skill rules relevant to the task being implemented.
- Executors honor prompt and hook constraints that affect commands, edits, or
  validation gates.
- Researchers ensure research output accounts for project skill patterns.
- Researchers distinguish live instruction sources from archived or cached
  copies.
- Verifiers apply skill rules when scanning for anti-patterns and verifying quality.
- Verifiers check that claimed prompt, hook, skill, or agent paths actually
  exist at the referenced canonical location.
- Debuggers follow skill rules relevant to the bug being investigated and the fix being applied.

The caller's agent file should specify which application applies.
