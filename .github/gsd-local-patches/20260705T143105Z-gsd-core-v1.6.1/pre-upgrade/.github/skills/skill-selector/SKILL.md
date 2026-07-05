---
name: skill-selector
description: Select and delegate the right skill for a task when the user asks which skill to use, pick the best workflow, route this request, find the right helper, or choose between multiple candidate skills. Also triggers on skill discovery, skill routing, delegation, trigger matching, capability selection, and multi-skill orchestration. Always finish the delegation plan with the doublecheck skill as the final verification stage.
argument-hint: "task=\"<task description>\" | --rebuild-index"
metadata:
   argument-hint: "task=\"<task description>\" | --rebuild-index"
---

# Skill Selector

Use this skill to choose the best skill for a task, decide whether multiple
skills should run in sequence, execute that chain when possible, and hand the
final result to `doublecheck`.

## When to Use

- The user asks which skill should be used for a task.
- Another skill needs to delegate work instead of doing everything itself.
- Multiple skills look relevant and the host assistant needs a clear execution
  order.
- The task spans discovery, implementation, review, or documentation and needs
  an explicit chain.
- The host assistant needs one fast place to search both workspace and external
  skill catalogs.

## Required Inputs

- A task statement, prompt, or summarized subtask.
- Optional constraints such as workspace scope, preferred output, or files.

## Core Files

- Machine index: `.github/skills/skill-selector/skills-index.json`
- Human catalog: `.github/skills/SKILLS_CONTENT.MD`
- Index builder: `scripts/build_skill_index.py`
- Executor layer: `scripts/executor.py`
- Regression verifier: `scripts/verify_skill_selector.py`
- E2E verifier: `scripts/verify_skill_selector_e2e.py`
- Selection rules: `REFERENCE.md`
- Shadowing diagnostics: `shadowed_skills` in `.github/skills/skill-selector/skills-index.json`

## How It Works

1. Load `.github/skills/skill-selector/skills-index.json`.
   The index is built from these roots in descending priority order:
   - `.github/skills`
   - `.agents/skills`
   - `~/.agents/skills`
   - installed agent plugin skill catalogs
   - installed VS Code extension skill catalogs
2. If the index is missing, stale, or the available skill set has changed, run:

   ```text
   /skill-selector --rebuild-index
   ```

   This command must rebuild the full machine index and the human catalog.

   Backend rebuild command:

   ```bash
   python .github/skills/skill-selector/scripts/build_skill_index.py --rebuild-index
   ```

3. Normalize the task into these routing signals:
   - desired outcome
   - operation type such as create, review, debug, plan, refactor, or document
   - domain such as Rust, frontend, architecture, Azure, or testing
   - expected artifact such as code, plan, audit, or README
   - scope such as current repo, current file, workspace, or external guidance
4. Rank candidates using the index and the rules in `REFERENCE.md`.
   If two discovered skills share the same name, keep only the highest-priority
   winner in the active routing set and record every shadowed candidate in the
   index diagnostics.
5. Prefer the smallest specialized skill that fully fits the task.
6. If the task needs more than one skill, build an ordered chain:
   - optional preparation skill
   - one primary worker skill
   - optional follow-up skill for review or packaging
   - mandatory final `doublecheck` verification
7. Return the delegation decision in a compact, explicit format:
   - selected skill
   - why it won
   - optional alternates
   - exact call order if more than one skill is needed
8. Convert the chosen chain into executable dispatch steps through the executor
   layer.
9. If the environment supports direct skill invocation, immediately call the
   selected skill or ordered skill chain in YOLO mode and continue until the
   requested artifact or task outcome is produced.
10. If only a dispatcher adapter is available, execute the same chain through
   that adapter instead of falling back to a printed route.
11. Do not stop at the recommendation step when direct execution is possible.
   The recommendation is an internal routing decision, not the final user-facing
   outcome.
12. Always invoke `doublecheck` on the final substantive output. If direct
   invocation is not possible, the host assistant must continue the task under
   the chosen skill instructions and run `doublecheck` immediately after the
   worker step completes.
   The `doublecheck` payload must include the original task, explicit request
   points derived from that task, and a checklist that verifies the final
   artifact against every listed point instead of validating only the last step
   output.
13. If the task asks for a self-review of the current skill or the assistant's
   own work, interpret the review target as the currently invoked skill
   artifacts first:
   - the current skill directory
   - its `SKILL.md`, `REFERENCE.md`, and executor/routing scripts when relevant
   - its local scripts and generated routing artifacts when relevant
   - not the unrelated repository diff by default
14. When direct execution starts, keep the selected chain short and purposeful:
   each invoked skill must add a distinct capability on the way to the final
   artifact.
15. After changing ranking, index-building behavior, or the executor layer,
    run:

   ```bash
   python .github/skills/skill-selector/scripts/verify_skill_selector.py
   python .github/skills/skill-selector/scripts/verify_skill_selector_e2e.py
   ```

## Ranking Rules

1. Exact task-trigger match beats broad semantic similarity.
2. Workspace-local skills beat external skills when both cover the same task.
3. Specialized worker skills beat generic builder or orchestration skills for
   the primary work step.
4. Primary artifact fit beats general domain fit.
5. If the user asks for review, prefer review-oriented skills before creation
   or planning skills.
6. If the user asks for creation, prefer creation or implementation skills
   before review-only skills.
7. If confidence remains low after ranking, return the top 2 or 3 candidates in
   order with the uncertainty called out explicitly.
8. For skill-creation tasks, prefer skill-specific alternates such as
   `crypto-skill-builder` over generic structure-oriented helpers when the
   primary worker stays `skill-builder`.
9. For self-review tasks such as `review your own skill` or transliterated
   variants like `prover sobstvennuju rabotu`,
   prefer a local skeptical review worker over a generic marketplace review
   skill, and keep the scope anchored to the current skill artifacts.

## Delegation Output Format

Use this structure when reporting the decision internally before execution, or
when direct invocation is impossible:

```text
Primary skill: <skill-name>
Why: <1-2 sentence reason>
Order:
1. <skill-a>
2. <skill-b>
3. doublecheck
Alternates: <comma-separated names or none>
```

## Examples

### Example 1

Task: `Review this Rust crate for security issues and missing tests.`

Expected routing:

```text
Primary skill: code-reviewer
Why: The task is a repository review focused on bugs, security, and test gaps.
Order:
1. code-reviewer
2. doublecheck
Alternates: review-extreme-skepticism, crypto-architect
```

### Example 2

Task: `Create a reusable skill from this workflow and validate the structure.`

Expected routing:

```text
Primary skill: skill-builder
Why: The task is explicitly about creating a new skill with repository-local structure rules.
Order:
1. skill-builder
2. doublecheck
Alternates: generate-project-context
```

### Example 3

Task: `Document this brownfield repo and then create a README.`

Expected routing:

```text
Primary skill: document-project
Why: The first required artifact is repository documentation, and README work is downstream from that scan.
Order:
1. document-project
2. create-readme
3. doublecheck
Alternates: tech-stack-blueprint, agent-tech-writer
```

### Example 4

Task: `Review your own current skill and verify the quality of your work.`

Expected routing:

```text
Primary skill: review-extreme-skepticism
Why: The request is a self-review of the currently invoked skill, so the review should stay anchored to the current skill artifacts rather than the whole repository.
Order:
1. review-extreme-skepticism
2. doublecheck
Alternates: code-reviewer, code-recon
```

## Guardrails

- Do not choose a broad orchestration skill as the primary worker if a more
  specialized skill clearly fits.
- Do not build long chains unless each step adds a distinct capability.
- Do not skip `doublecheck` for substantive outputs, plans, reviews, or factual
  summaries.
- Do not route to external skills when a workspace-local skill already covers
  the task well enough.
- Do not assume the human-readable catalog is complete; the JSON index is the
  machine source of truth.
- Do not interpret `own`, `self`, `current skill`, or equivalent transliterated
   self-review phrasing as permission to review unrelated repo changes first.
- Do not end the interaction with only a skill recommendation when the selected
   skill chain can be executed in the current environment.

## Invocation Examples

Use these examples when invoking `skill-selector` from chat or when explaining
the expected operator workflow.

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

## User Hints

- Use `/skill-selector --rebuild-index` after adding, removing, renaming, or materially rewriting skills.
- Phrase the task in terms of the artifact you want, such as `review`, `README`, `plan`, `skill`, or `tests`.
- If you want execution instead of only routing, ask for the final artifact directly instead of only asking which skill should be used.
- For self-review, include words like `own`, `current skill`, or transliterated phrases such as `prover sobstvennuju rabotu` so the scope stays anchored to the current skill artifacts.
- If the task spans more than one step, describe the sequence in one prompt, for example `document this repo and then create a README`.