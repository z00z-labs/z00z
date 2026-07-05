# Repository Agent Rules and Guidance

- Use repository docs, planning files, and source code directly.
- Keep instructions local to this repository and avoid external documentation graph workflows.
- Validate changes with the repository's normal build, test, and review commands.

## Canonical Instruction Surfaces

- Primary live instruction roots are `./.github/skills/`, `./.github/prompts/`, `./.github/hooks/`, `./.github/agents/`, and `./.github/plugins/`.
- Repo-local Codex context lives at `./.codex/AGENTS.md`; user-local Codex surfaces, when relevant, live at `$HOME/.codex/skills/` and `$HOME/.codex/prompts/`.
- Use canonical paths only. Do not rely on alias names, shim names, or cached prompt nicknames when a real path exists.
- Treat `./.github/gsd-local-patches/`, `./.github/deep-wiki-local-overrides/`, `./.agents/.install-backups/`, `$HOME/.codex/.tmp/`, `$HOME/.codex/plugins/cache/`, and `$HOME/.codex/vendor_imports/` as non-authoritative unless the task explicitly targets them.

## Token discipline

Use compact output by default.

Before producing long text, classify the task:

- simple: answer directly
- medium: short plan + concrete steps
- complex: summary first, then structured sections

For implementation work:

- Prefer code changes over prose.
- Do not explain unchanged code.
- Do not paste full files unless requested.
- Use paths and line references where possible.
- When tests fail, show only the failing command, error essence, and next action.

## Forbidden verbosity

Avoid:

- "Sure, here is..."
- restating the task
- generic caveats
- duplicate bullet points
- long background explanations
- full tutorials unless requested

## Expansion protocol

If more detail is needed, end with:

`EXPANDABLE: details available for <topic>.`

Do not expand unless the user asks.

## Init Workflow

At the start of every new session in this repository, run the `/z00z-chat-init`
skill before any planning, editing, review, or test work.

## CodeGraph

- Prefer CodeGraph first for structural codebase questions when `.codegraph/` exists.
- Use CodeGraph before manual `rg` or file reads for architecture, call-flow, dependency, blast-radius, and "where is X implemented?" questions.
- Prefer `codegraph explore` first; use `node`, `query`, `callers`, `callees`, `impact`, and `status` only when the task is narrower or scriptable.
- If CodeGraph reports stale or pending-sync files, read those files directly before relying on the graph result for edits.
- Fall back to normal filesystem and code reads when CodeGraph does not cover the needed area or when exact live line-level context is still required.

## Order Of Operations

- For live implementation questions: `CodeGraph` -> direct file, test, and config reads -> `Deep Wiki` for codebase explanation or page generation -> `@wiki` only if the result should become durable knowledge.
- For durable documentation or knowledge-base work: `@wiki` and the local `.wiki/` first -> ingested/raw docs -> `CodeGraph` only when implementation validation is needed -> `Deep Wiki` only when a codebase-specific research/page artifact is needed.
- Do not treat `Deep Wiki` or `@wiki` as the primary evidence source for current implementation while live code, tests, and configs have not been checked.
- After writing or updating local wiki content, prefer `@wiki compile --local` and `@wiki lint --local --fix`.

<!-- GSD Configuration — managed by gsd-core installer -->
# Instructions for GSD

- Use the gsd-core skill when the user asks for GSD or uses a `gsd-*` command.
- Treat `/gsd-...` or `gsd-...` as command invocations and load the matching file from `.github/skills/gsd-*`.
- When a command says to spawn a subagent, prefer a matching custom agent from `.github/agents`.
- Do not apply GSD workflows unless the user explicitly asks for them.
- After completing any `gsd-*` command (or any deliverable it triggers: feature, bug fix, tests, docs, etc.), ALWAYS: (1) offer the user the next step by prompting via `ask_user`; repeat this feedback loop until the user explicitly indicates they are done.
<!-- /GSD Configuration -->
