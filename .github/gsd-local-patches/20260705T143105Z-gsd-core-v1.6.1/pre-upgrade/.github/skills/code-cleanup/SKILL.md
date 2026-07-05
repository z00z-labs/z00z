---
name: code-cleanup
description: 'Perform a deep cleanup pass on the current codebase without changing intended behavior. Use when removing dead code, legacy code, duplicated logic, tight coupling, outdated comments, incorrect comments, or missing critical explanations. Trigger words: cleanup code, deep cleanup, dead code, legacy code, structural cleanup, duplicate logic, outdated comments, incorrect comments, missing critical explanations, yolo cleanup.'
argument-hint: 'path, crate, module, or repo scope to clean up'
---

# Cleanup Code

## When to Use

- The user asks for cleanup code, deep cleanup, dead-code removal, or behavior-preserving structural cleanup.
- A module contains legacy logic, duplicate branches, tight coupling, or stale comments that should be fixed without changing intended behavior.
- The user explicitly wants YOLO cleanup limited to clearly safe, reversible fixes.
- The task is broader than formatting but narrower than a redesign or feature rewrite.

## Mission

Act as a behavior-preserving cleanup engineer.

Your job is to deeply clean the requested codebase scope without changing its
intended behavior. Prefer clarity over cleverness, avoid over-engineering, and
treat ambiguity as a stop sign rather than an invitation to guess.

When the user asks for YOLO mode, that means:

- apply all clearly safe cleanup fixes without waiting for approval
- do not stop at analysis if the fix is local, reversible, and behavior-preserving
- do not change public behavior, storage formats, wire formats, security
  semantics, or compatibility contracts unless the user explicitly asks for that

If a suspected fix might change behavior, label it `requires clarification`
instead of applying it.

For ambiguity-heavy cases, use the
[cleanup decision matrix](./references/cleanup-decision-matrix.md).

## Primary Outputs

Every run must produce these outputs:

1. A summary of issues found, grouped by category.
2. A list of fixes applied.
3. A list of findings marked `requires clarification`.
4. Validation results for the changed scope.

Do not claim success without both findings review and post-change validation.

## Cleanup Categories

Classify findings into these categories exactly:

1. Dead code
   - unreachable code
   - unused functions
   - unused variables
2. Legacy code
   - outdated patterns
   - deprecated logic
   - compatibility leftovers
3. Structural problems
   - duplicated logic
   - tight coupling
4. Comments and documentation
   - outdated comments
   - incorrect comments
   - missing critical explanations

If a finding does not clearly fit one of these groups, keep it out of scope
unless the user explicitly widens the cleanup pass.

## Non-Negotiable Rules

- Preserve intent.
- Preserve public behavior.
- Prefer deletion over abstraction when dead code is proven dead.
- Prefer small, local refactors over wide architectural rewrites.
- Do not rewrite working code just to make it look newer.
- Do not invent reasons for a cleanup; every finding needs evidence.
- Be strict about ambiguity.
- If unsure, mark the item `requires clarification`.
- Before deleting a legacy or unused module, verify impact against the whole
   codebase, not just the local folder.
- Remove risky elements one at a time, never as a bulk batch.

Evidence must come from at least one concrete source in the target scope:

- call-site search
- compiler or linter output
- tests
- local documentation or comments that still match the implementation
- type or module boundaries visible in the code

When a finding sits near a compatibility, security, or API boundary, consult the
[cleanup decision matrix](./references/cleanup-decision-matrix.md) before
classifying it as safe to auto-fix.

Module-removal guard:

- do not delete a module, package, crate, or cross-file component until the
   whole workspace has been checked for direct references, indirect references,
   tests, scripts, generated-code hooks, docs, and configuration touchpoints
- if more than one candidate element qualifies for deletion, process them one by
   one with validation after each removal
- if one removal fails validation, stop the deletion sequence and move the
   remaining candidates to `requires clarification`

Import cleanup rule:

- when multiple imports come from the same crate, module, package, or namespace,
   group them into one import statement when the language idiom and formatter
   allow it
- do not scatter imports from the same source across separate lines without a
   concrete reason
- preserve aliases, side-effect imports, required ordering, and formatter or
   linter constraints

## Workflow

### 1. Scope the cleanup

- Determine the requested cleanup target: repository, crate, package, module,
  directory, or file.
- Read the nearby code, tests, and relevant documentation before editing.
- Check for existing workspace conventions that constrain refactors.

### 2. Analyze the target

Build an issue inventory under the required cleanup categories.

For each finding, capture:

- location
- category
- why it is a problem
- confidence level: `high`, `medium`, or `low`
- whether it is safe to auto-fix

Only mark a finding as safe to auto-fix when the evidence is concrete.

Do not infer dead code from style alone. Prove it through references, compiler
signals, or unreachable control flow.

For any module-, package-, or crate-level deletion candidate, the analysis must
also include:

- whole-workspace reference search
- tests and fixtures search
- config, script, and documentation search
- cross-crate and cross-module dependency review when the repository is
   multi-crate or multi-package

### 3. Verify findings with doublecheck

Before reporting or fixing findings, run the `doublecheck` skill in one-shot
mode on the findings summary.

Verification goals:

- confirm the finding is real
- catch hallucinated dead code or fake duplication
- challenge assumptions about behavior preservation
- downgrade uncertain findings to `requires clarification`

If `doublecheck` contradicts a finding, do not apply the fix until the evidence
is reconciled.

If the environment cannot invoke the `doublecheck` skill directly, perform the
same three checks manually before proceeding:

- extract the exact claims behind each finding
- challenge them against the local evidence in the codebase
- downgrade anything uncertain to `requires clarification`

### 4. Decide what can be fixed automatically

Safe auto-fix examples:

- remove unused locals, imports, helpers, and branches proven unreachable
- delete compatibility leftovers that have no live call sites
- extract obviously duplicated internal logic into one helper when behavior is
  unchanged and the change does not widen coupling
- correct or delete stale comments that conflict with the code
- add a short critical explanation where code is otherwise easy to misuse
- delete one legacy or unused module only after whole-codebase proof shows it is
   isolated and after the deletion can be validated on its own

Requires clarification examples:

- removing exported APIs
- changing serialization, database, protocol, or config behavior
- deleting code that is referenced dynamically or by reflection/macros/build
  scripts and cannot be proven unused
- collapsing intentionally duplicated logic across security or trust boundaries
- replacing a legacy path when compatibility expectations are unclear
- renaming symbols that may be part of external tooling, scripts, or user-facing
   workflows
- deleting multiple legacy modules in one batch
- deleting a module whose absence has not been validated against the rest of the
   workspace

Use the [cleanup decision matrix](./references/cleanup-decision-matrix.md) as
the tiebreaker for disputed or borderline cases.

### 5. Apply fixes in YOLO mode

If the user requested YOLO mode, apply every safe auto-fix in one pass.

Implementation rules:

- make the smallest change that fully resolves the issue
- keep naming and structure consistent with the surrounding codebase
- do not mix unrelated cleanups into the same pass
- add or update tests if the cleanup touches behavior-adjacent code paths
- leave a concise `requires clarification` record for anything not fixed
- if deleting a module or similarly large unit, delete exactly one element,
  validate it, and only then consider the next candidate

YOLO mode does not waive the ambiguity gate. It only removes the need to ask
for approval before applying fixes already proven safe.

YOLO mode does not permit batch deletion of legacy modules, crates, or packages.

### 6. Validate after cleanup

Run the narrowest reliable validation for the edited scope.

Examples:

- targeted tests for touched modules
- project lint or static analysis for the changed files
- compile checks for the affected crate or package
- repository-specific verification commands when the project requires them

If validation fails, fix cleanup-induced failures immediately. If the failure is
pre-existing and unrelated, report it separately.

At minimum, validation must cover one of these after every edited scope:

- parse or compile success
- linter success for changed files
- targeted tests for touched behavior-adjacent paths

If a module, package, or crate was removed, validation must also include the
widest reliable workspace check available for affected dependents.

### 7. Report the result

Return a concise final report with these sections:

- `Summary of issues found`
- `List of fixes`
- `Requires clarification`
- `Validation`

Keep the report concrete. Do not pad it with generic advice.

## Decision Standard

Use this decision table:

| Situation | Action |
| --- | --- |
| Real issue, clearly behavior-preserving fix | Fix it |
| Real issue, but behavior impact is uncertain | Mark `requires clarification` |
| Possible issue, weak evidence | Do not fix |
| Comment conflicts with code | Update or delete comment |
| Code looks old but is still the clearest correct option | Leave it alone |

When two rules conflict, prefer this order:

1. Preserve behavior
2. Remove ambiguity
3. Reduce code
4. Improve structure

## Quality Bar

A cleanup pass is complete only when all of these are true:

- findings are categorized
- findings were challenged with `doublecheck`
- all safe fixes were applied
- unclear items were explicitly separated
- validation was run after edits
- the final report lists both issues found and fixes made
- every applied fix can be traced back to a specific verified finding
- every module-level deletion was preceded by whole-codebase review and followed
   by standalone validation before any further deletion

## Z00Z Notes

When the target is this repository, also respect these constraints:

- keep all technical content in English
- do not modify `z00z_crypto/tari/`
- preserve the ONE SOURCE OF TRUTH boundaries around `z00z_utils`
- do not introduce identifiers longer than 5 words in Rust code
- keep same-module Rust imports grouped in a single brace import
- apply the same import-grouping principle in Python and other languages when
   their local style and tooling allow a grouped form
- prefer repository-standard verification over ad hoc commands

## Example Prompts

- `/code-cleanup clean up crates/z00z_utils without changing behavior`
- `/code-cleanup deep cleanup crates/z00z_storage/src/assets in yolo mode`
- `/code-cleanup find dead code and stale comments in crates/z00z_wallets`
- `/code-cleanup audit duplicated logic in this module and fix only safe items`