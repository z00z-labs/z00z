---
name: token-discipline
description: Use when the user asks for shorter answers, fewer output tokens, compact status updates, terse code reviews, concise debugging guidance, reduced verbosity, or lower model cost. Trigger on phrases like be brief, be concise, fewer tokens, shorter answer, compact mode, terse review, minimal output, or less verbose. Do not use when the user explicitly asks for exhaustive detail, legal drafting, safety-critical analysis, or full research narratives.
---

# Token Discipline Skill

Use this skill to compress visible output without dropping correctness, failure signals, or next actions.

## When to Use This Skill

- The user says `be brief`, `be concise`, `fewer tokens`, `compact mode`, `shorter answer`, `minimal output`, or `less verbose`
- You need a compact implementation closeout with changed files, commands, and test status
- You need findings-first code review output without extra explanation
- You need terse debugging guidance that still names the next discriminating step
- You want lower-cost prompt or response surfaces while preserving critical details

## When Not to Use This Skill

- The user explicitly asks for a deep dive, tutorial, or exhaustive walkthrough
- The task is legal, compliance, postmortem, or safety-critical and requires full nuance
- The problem is ambiguous enough that compression would hide key uncertainty
- The user wants exploratory research rather than an action-biased result

## Prerequisites

- `tiktoken` for token counting via [token_count.py](./scripts/token_count.py)
- `llmlingua` for optional prompt compression via [llmlingua_compress.py](./scripts/llmlingua_compress.py)
- `promptfoo` for repeatable offline evals via [run_eval.sh](./scripts/run_eval.sh)

## Quick Invocation Phrases

Use or adapt phrases like these when you want to trigger the skill intentionally:

- `Answer in compact mode.`
- `Be brief but keep risks.`
- `Give me the terse review version.`
- `Use fewer tokens and no filler.`
- `Short status only: files, tests, risks.`

This is the skill-level equivalent of an argument hint.

## Goal

Reduce visible output tokens while preserving correctness, task completion, and safety.

This skill does not mean "be vague".
It means "remove non-essential tokens".

### Default mode: `compact`

## Core rules

1. Answer the actual task first.
2. Do not repeat the user's request.
3. Do not add generic introductions.
4. Do not include reasoning traces unless they are required to justify a risky decision.
5. Prefer structured outputs: tables, YAML, JSON, diffs, commands.
6. Prefer references to files/paths over pasted full content.
7. Prefer one strong recommendation over many weak alternatives.
8. Use short section headers.
9. Collapse obvious steps.
10. Never omit critical risk, test failure, security issue, data-loss warning, or breaking change.

## Mode selection

Classify each response before writing:

```yaml
mode_matrix:
  tiny:
    use_for: "simple factual answer, yes/no, command lookup"
    visible_budget: "<=40 words"
  compact:
    use_for: "normal coding help, small explanation, short design choice"
    visible_budget: "<=120 words"
  standard:
    use_for: "multi-step implementation, debugging, architecture"
    visible_budget: "<=350 words"
  deep:
    use_for: "explicitly requested deep analysis, audits, legal/high-stakes"
    visible_budget: "as needed, but no filler"
  patch_only:
    use_for: "code changes requested"
    visible_budget: "diff/commands + <=5 bullets"
```

The bundled `budget_guard.py` enforces approximate token ceilings for these visible word budgets.

## Failure policy

If token budget conflicts with correctness, correctness wins.

Never shorten away:

```
never_omit:
  - security risks
  - failing tests
  - data loss
  - breaking changes
  - legal/compliance risk
  - unsafe commands
  - irreversible operations
```

## Output Contracts

Use the output shapes in [output-contracts.md](./references/output-contracts.md) when you need stable, terse formatting.

### Coding output contract

When editing code:

```
output_order:
  - changed_files
  - commands_run
  - tests_result
  - risks_or_followups
limits:
  changed_files: "paths only unless user asks for full file"
  commands_run: "only commands that matter"
  tests_result: "pass/fail + relevant error"
  explanation: "<=5 bullets"
```

## Bundled Scripts

- [token_count.py](./scripts/token_count.py): count tokens, words, and characters
- [budget_guard.py](./scripts/budget_guard.py): fail if a draft exceeds a mode budget
- [prompt_audit.py](./scripts/prompt_audit.py): catch filler phrases and duplicate headings
- [compact_markdown.py](./scripts/compact_markdown.py): strip filler lines and collapse blank space
- [llmlingua_compress.py](./scripts/llmlingua_compress.py): optional prompt compression using `llmlingua`
- [promptfoo_local_provider.mjs](./scripts/promptfoo_local_provider.mjs): offline local provider for repeatable promptfoo evals
- [run_eval.sh](./scripts/run_eval.sh): run the promptfoo suite without external model keys
- [run_demo.sh](./scripts/run_demo.sh): end-to-end demo of compaction, audit, budget checks, llmlingua compression, and promptfoo eval

## Reasoning budget

Use internal reasoning as needed, but keep visible reasoning compressed.

```
reasoning_policy:
  trivial:
    visible_reasoning: "none"
  simple:
    visible_reasoning: "one sentence max"
  medium:
    visible_reasoning: "key decision bullets only"
  hard:
    visible_reasoning: "summary of trade-offs, not full chain"
```

## Abbreviation policy

Use abbreviations only when they are standard or defined once.

Allowed:

```
allowed:
  - API
  - CLI
  - CI
  - PR
  - AST
  - RAG
  - LLM
  - JSON
  - YAML
```

Do not invent dense Caveman-style abbreviations unless user explicitly asks.

Bad:

```
impl cfg w/ ctx cmpct + eval
```

Good:

```
Implement compact config, then validate with evals.
```

## Expansion protocol

If the answer is compressed and more detail may be useful, add one final line:

```
EXPANDABLE: <topic1>, <topic2>.
```

Do not expand automatically.

## Step-by-Step Workflows

### 1. Compact answer workflow

1. Pick a mode from `tiny`, `compact`, `standard`, `deep`, or `patch_only`.
2. Answer the task first.
3. Remove request restatement and generic preambles.
4. Keep one decisive recommendation instead of a menu of weak options.
5. Add `EXPANDABLE:` only if more detail would genuinely help.

### 2. Compact code-change workflow

1. Show changed files first.
2. Show only commands that materially validate the change.
3. Report pass or fail directly.
4. Keep follow-ups or risks to a short flat list.

### 3. Draft audit workflow

From the skill directory:

```bash
python ./scripts/prompt_audit.py draft.md --json
python ./scripts/token_count.py draft.md --json
python ./scripts/budget_guard.py draft.md --mode compact --json
python ./scripts/compact_markdown.py draft.md --out cleaned.md
```

### 4. Optional llmlingua prompt compression workflow

Use this when you want to compress a prompt surface before sending it to a model:

```bash
python ./scripts/llmlingua_compress.py prompt.txt \
  --instruction 'Compress without losing setup steps.' \
  --question 'What is the minimal correct answer?' \
  --target-token 80 \
  --json
```

The bundled script defaults to the small `sshleifer/tiny-gpt2` model so the workflow is practical on CPU.
It uses cache-only loading by default so repeated runs stay quiet once the tiny model has been warmed locally.

### 5. Offline validation workflow

```bash
./scripts/run_eval.sh
./scripts/run_demo.sh
```

Use [testing-and-demo.md](./references/testing-and-demo.md) for the full command list and expected signals.

## Output examples

### Example: simple

User asks: "How to count tokens?"

Answer:

```
Use `tiktoken` for OpenAI models:

pip install tiktoken
```

### Example: coding

```
changed_files:
  - src/token_budget.py
tests_result:
  status: pass
  command: pytest tests/test_token_budget.py
notes:
  - Added hard cap validation.
  - No API behavior change.
```

### Example: architecture

```
recommendation: "Use AGENTS.md + token-discipline skill + promptfoo evals."
why:
  - "AGENTS.md is always loaded."
  - "Skill is reusable and explicitly invokable."
  - "Evals prevent quality loss."
risk: "Too aggressive caps can hide important failures."
```

### Example: review

```yaml
findings:
  - severity: high
    file: src/cache.rs
    issue: eviction path drops the last error
    fix: return the original error after cleanup
```

## Troubleshooting

| Problem | What to check | Fix |
|---|---|---|
| `promptfoo` asks for an API key | The provider was overridden to a remote model | Use `./scripts/run_eval.sh` or set `TOKEN_DISCIPLINE_PROVIDER` to the local provider |
| `promptfoo` refuses to start after an upgrade | The latest promptfoo needs a newer Node patch than the system default | Use `./scripts/run_eval.sh`, which runs promptfoo through a compatible local `node@22.22.0` runtime |
| `llmlingua` cannot find the local model cache | Cache-only mode is active and the tiny model has not been warmed yet | Re-run `llmlingua_compress.py` once with `--allow-download`, then go back to the default quiet cache-only path |
| Budget check fails unexpectedly | Budget guard measures tokens, not words | Re-run `token_count.py --json` and switch modes if the task needs more room |
| Audit flags a harmless line | The line matches a filler heuristic | Rewrite the sentence or inspect the JSON output from `prompt_audit.py` |

## References

- [Output contracts](./references/output-contracts.md)
- [Testing and demo guide](./references/testing-and-demo.md)
