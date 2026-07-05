# Skill Selector Reference

## Purpose

This file defines how `skill-selector` ranks skills, resolves ties, and builds
multi-step delegation chains.

It also defines the execution contract after routing: when the host environment
can invoke the selected skills directly, `skill-selector` must continue into
execution and produce the requested artifact instead of stopping at the chain
recommendation.

## Source Priority

Use these source priorities by default:

1. Workspace-local skills from `.github/skills`
2. Workspace-local compatibility skills from `.agents/skills`
3. User-level compatibility skills from `~/.agents/skills`
4. External skills discovered from installed agent plugin catalogs
5. External skills discovered from installed VS Code extensions

Only break this rule when the external skill is materially more specific than
every local candidate.

If multiple discovered skills share the same `name`, keep exactly one routing
winner using the priority list above. Record every losing candidate in the
index-level `shadowed_skills` diagnostics so collisions stay explainable.

## Source Coverage

The generated index currently scans these roots:

1. `.github/skills`
2. `.agents/skills`
3. `/home/vadim/.agents/skills`
4. `/home/vadim/.config/Code/agentPlugins`
5. `/home/vadim/.vscode/extensions`

This broader scan matters because several high-value workflow skills are not
shipped through the awesome-copilot catalog alone. A concrete example is
`address-pr-comments`, which lives under the GitHub Pull Request extension and
should win for active pull-request feedback tasks.

## Scoring Model

Treat the JSON index as the machine-readable source of truth.

Each candidate skill can receive points from these buckets:

| Bucket | Meaning | Typical importance |
| --- | --- | --- |
| Exact trigger phrase | Direct phrase match between task and skill triggers | highest |
| Name match | Task contains the skill name or its split words | high |
| Keyword overlap | Task shares extracted keywords with the skill | high |
| Artifact fit | Skill is specialized for the requested output | high |
| Scope fit | Skill is workspace-local and relevant to the repo | medium |
| Domain fit | Skill is aligned to the technology or subsystem | medium |
| Genericity penalty | Skill is a meta-skill when a worker skill already fits | negative |

## Tie-Breaking Rules

If two skills are close, break ties in this order:

1. Specific worker over generic helper.
2. Workspace-local over external.
3. Single-purpose skill over broad orchestrator.
4. Existing repo conventions over marketplace convenience.
5. Fewer handoffs over longer chains.
6. For self-review of the current skill or the assistant's own work, prefer a local skeptical
   reviewer over a generic marketplace `review` skill.

## Self-Review Rule

Treat these queries as self-review requests when they ask to inspect the
assistant's own work or the current skill:

- `review your own skill`
- `review this current skill`
- `review your own work quality`
- transliterated variants such as `prover sobstvennuju rabotu`

When this happens:

1. Keep the target scope anchored to the currently invoked skill artifacts.
2. Prefer `review-extreme-skepticism` as the primary worker when it is present.
3. Prefer `code-reviewer` as the first strong alternate.
4. Demote the generic external `review` skill unless no stronger local review
   worker fits.

## Chain Assembly Rules

Use a multi-step chain only when the steps are meaningfully different.

## Execution Contract

After a chain is chosen:

1. Treat the chain as an execution plan, not the final answer.
2. Materialize the plan through the executor layer in `scripts/executor.py`.
3. If direct invocation is possible, execute the chosen skill or chain in YOLO
   mode immediately.
4. If only a dispatcher adapter is available, use it to execute each step in
   order and pass the previous output forward.
5. Continue until the original user task is solved or the requested artifact is
   produced.
6. Run `doublecheck` on the final substantive output, not only on the routing
   note.
   The `doublecheck` input must carry the original task, explicit request
   points extracted from that task, and a checklist that verifies the final
   artifact against each point rather than only against the last worker output.
7. Only return the raw `Primary skill / Order / Alternates` structure as the
   final user-facing output when direct invocation is impossible.

### Acceptable chain patterns

- discovery -> worker -> doublecheck
- scan -> create -> doublecheck
- create -> review -> doublecheck
- reconcile -> implement -> verify -> doublecheck

### Poor chain patterns

- planner -> planner -> builder -> doublecheck
- reviewer -> skeptic-reviewer -> generic-reviewer -> doublecheck
- several near-identical external skills with no new capability

## When To Add A Preparation Step

Add a preparation skill only if the task requires one of these first:

- repository mapping
- architecture context
- brownfield documentation scan
- implementation planning that the worker skill depends on
- UI audit before UI edits

## When To Add A Follow-Up Step

Add a follow-up skill only if the primary worker does not naturally cover:

- review
- documentation packaging
- architecture capture
- validation beyond the main deliverable

## Mandatory Final Verification

Always end the chain with `doublecheck` for:

- plans
- factual summaries
- architecture work
- documentation intended as guidance
- code reviews
- skill-selection reports and routing recommendations

You may skip `doublecheck` only for transient status updates or when the host
environment cannot invoke it and the output contains no material factual claims.

## JSON Index Schema

The generated `skills-index.json` uses these core fields:

| Field | Meaning |
| --- | --- |
| `name` | skill name from frontmatter or folder name |
| `description` | frontmatter description |
| `scope` | `workspace`, `user`, or `external` |
| `source_root` | scanned root directory |
| `source_label` | stable label for source precedence and diagnostics |
| `source_priority` | deterministic source-order rank where lower wins |
| `skill_dir` | skill directory path |
| `skill_md` | path to `SKILL.md` |
| `keywords` | deduplicated keyword tokens used for ranking |
| `trigger_phrases` | extracted phrases from description and trigger sections |
| `sections` | captured section snippets such as `When to Use` or `Activation` |
| `metadata` | custom frontmatter fields preserved under the Agent Skills metadata key |
| `meta_skill` | whether the skill appears to be orchestration-heavy |

The top-level index also carries `shadowed_skills`, which lists every name
collision, the chosen winner, and the shadowed candidates that were excluded
from active routing.

## Freshness Rules

Rebuild the index when any of these are true:

- a skill directory is added or removed
- a skill description changes
- a skill gains new trigger wording
- the JSON index or markdown catalog is missing

Canonical chat command:

```text
/skill-selector --rebuild-index
```

Backend rebuild command:

```bash
python .github/skills/skill-selector/scripts/build_skill_index.py --rebuild-index
```

For the regression assertion pass, use:

```bash
python .github/skills/skill-selector/scripts/verify_skill_selector.py
python .github/skills/skill-selector/scripts/verify_skill_selector_e2e.py
```

The regression must verify behavior, not only wording:

- selected chain order
- step-by-step dispatch through the executor layer
- propagation of prior step output into the next dispatch input
- final `doublecheck` execution

The e2e verifier must also check final artifacts:

- `skills-index.json` generation
- `SKILLS_CONTENT.MD` generation
- CLI `--rebuild-index` output for rebuilt artifact paths
- CLI `--execute` output for a multi-step chain
- CLI `--execute` output for self-review routing

## Invocation Examples

Use these examples when invoking `skill-selector` from chat or when documenting
the expected maintainer workflow.

### Rebuild The Index

Chat-style invocation:

```text
/skill-selector --rebuild-index
```

Backend command:

```bash
python .github/skills/skill-selector/scripts/build_skill_index.py --rebuild-index
```

### Route A Review Task

```text
/skill-selector review this Rust crate for security issues and missing tests
```

### Route A Documentation Task

```text
/skill-selector document this brownfield repo and then create a README
```

### Route A Self-Review Task

```text
/skill-selector prover sobstvennuju rabotu na kachestvo sdelaj sebe polnocennoe review
```

### Route A Skill-Creation Task

```text
/skill-selector create a reusable skill from this workflow and validate the structure
```

### Execute The Selected Chain

Backend command:

```bash
python .github/skills/skill-selector/scripts/build_skill_index.py --query "document this repo and write a README" --execute --top 3
```

This path must print the ranked matches, the suggested chain, and the executed
dispatch trace.

## Suggested Smoke Queries

Use these sample queries after rebuilding the index:

```bash
python .github/skills/skill-selector/scripts/build_skill_index.py --query "review this Rust crate for security issues"
python .github/skills/skill-selector/scripts/build_skill_index.py --query "create a new skill from this workflow"
python .github/skills/skill-selector/scripts/build_skill_index.py --query "document this repo and write a README"
python .github/skills/skill-selector/scripts/build_skill_index.py --query "document this repo and write a README" --execute --top 3
python .github/skills/skill-selector/scripts/build_skill_index.py --query "fix review comments on the active pull request"
python .github/skills/skill-selector/scripts/build_skill_index.py --query "document this brownfield project for humans and AI context"
python .github/skills/skill-selector/scripts/build_skill_index.py --query "prover sobstvennuju rabotu na kachestvo sdelaj sebe polnocennoe review"
```

The result should prefer a strong primary worker and append `doublecheck` as the
final verification step.

## Verification Baseline

The selector is in a good state when these internal checks hold:

| Query | Expected top match | Expected chain |
| --- | --- | --- |
| `document this repo and write a README` | `document-project` | `document-project -> create-readme -> doublecheck` |
| `fix review comments on the active pull request` | `address-pr-comments` | `address-pr-comments -> doublecheck` |
| `create a new skill from this workflow and validate the structure` | `skill-builder` | `skill-builder -> doublecheck` |
| `document this brownfield project for humans and AI context` | `document-project` | `document-project -> create-readme -> doublecheck` |
| `review this Rust crate for security issues and missing tests` | `code-reviewer` | `code-reviewer -> rust-fuzz-coverage -> doublecheck` |
| `prover sobstvennuju rabotu na kachestvo sdelaj sebe polnocennoe review` | `review-extreme-skepticism` | `review-extreme-skepticism -> doublecheck` |

The current internal assertion batch also expects the rebuilt index to include at
least 400 skills after installed plugin and extension sources are scanned.

For the skill-creation query above, the first alternate should stay
`crypto-skill-builder`, not a generic structure-only helper such as
`create-readme`.

For the self-review query above, the first alternate should stay
`code-reviewer`, and the generic external `review` skill should not win the top
slot.
