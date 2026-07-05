---
name: "GSD-Review-Tasks-Execution"
agent: agent
description: "Use when you need to review and fix the implementation of one specific task from a spec file, with focus on logic, crypto correctness, checklist compliance, and warnings."
argument-hint: "current_spec=<path> current_task=\"<exact task phrase>\""
---

# Review Tasks Execution

Review and fix only the implementation of the requested task from the current spec.

## Inputs

- `current_spec`:  full or relative path to the active spec file(s).
- `current_task`: (optional) exact task phrase to find in `current_spec` or a unique heading prefix that identifies exactly one section in `current_spec`. If is missing or ambiguous, review ALL implemented tasks in `current_spec`.

## Mission

Find `current_task` inside `current_spec`.
Review the implementation and tests for `current_task` OR all implemented tasks in `current_spec` if `current_task` is missing

Your role is crypto code analyst and code reviewer.
Be critical, constructive, and objective.
Focus on critical blockers, obvious mistakes, and spec drift before style-level concerns.
Own the review end to end: inspect, triage, fix, validate, update checklist state, and finish only when the current task review is actually complete.

## 🎯 Skill Routing Matrix

Use the smallest skill set that matches the current task surface.

| Trigger | Skills | When to use |
| --- | --- | --- |
| Crypto, proofs, signatures, timing-sensitive code, or serialization and literal contract changes | `/crypto-architect`, `/constant-time-analysis`, `/z00z-crypto-auditor` | Use when the task changes proof code, cryptographic boundaries, or secret-sensitive execution paths. |
| Trust boundaries, API surface, authn/authz, secrets, permissions, or dependency introduction | `/security-audit`, `/threat-model-expert` | Use when the task changes attack surface, authorization flow, or externally reachable behavior. |
| Drift against baseline, historical plans, or prior phase truth | `/alert-concept-drift` | Use when the task must be compared to an older branch, phase, or summary-backed baseline. |
| Hot paths, latency, throughput, memory pressure, or scalability risks | `/performance-engineer` | Use when the task could change runtime cost, buffering, caching, or resource usage. |
| Dead code, duplication, legacy noise, or cleanup-only changes | `/code-cleanup` | Use when the task is mostly simplification, removal of stale paths, or naming cleanup. |
| Substantive implementation review | `/code-reviewer` | Use as the default review skill for every non-trivial task. |

Do not run every specialist skill on every task. Route only the skills that match the current surface, then widen the set only if the review evidence shows a new risk area.

Verify all of the following:

- logical correctness;
- architectural consistency;
- unresolved pitfalls or edge cases;
- structure and type integrity;
- cryptographic correctness;
- checklist compliance for `current_task`;
- relevant errors and warnings.

Fix all found issues in YOLO mode.

> [!Caution]
> Prioritize correctness, security, and spec drift over stylistic cleanup. Do not spend scope on low-value refactors while task-critical issues remain.

## Required Review Process

1. Read the full context in `current_spec` from the start of the document up to and including `current_task`.
2. Read the implementation and tests related to `current_task`.
3. Run the skill or skills selected by the routing matrix on `current_spec`, `current_task`, and the implemented codebase, and treat their output as mandatory review evidence only when the task surface actually matches their trigger.
4. Extract the concrete risks, invariants, misuse cases, and required corrections from the selected skill outputs before deciding whether the implementation is acceptable.
5. Check whether the implementation matches the spec exactly.
6. Triage findings by severity: correctness, security, cryptographic risk, compatibility, and testability first; minor style issues last.
7. Identify bugs, inconsistencies, missing tests, unsafe assumptions, ambiguous behavior, and checklist drift.
8. Fix all issues that are in scope for `current_task`.
9. Reuse existing helpers, patterns, and architecture before introducing new structures.
10. If a local blocker prevents correct review of `current_task`, fix it automatically when it is required for correctness or completion and still within scope.
11. If completion requires a significant architectural change or another decision not clearly implied by the spec, stop and report the exact checkpoint, proposed change, impact, and smallest decision needed.
12. Use the `/doublecheck` skill in one-shot mode on the full review evidence when the review contains substantive repo-specific factual claims, citations, or other verifiable assertions that materially affect the decision.
13. Extract all verifiable claims and run the full three-layer verification pipeline on the evidence that materially affects the decision.
14. Resolve or explicitly report every material issue raised by `/doublecheck` before treating the task review as complete.
15. Update the matching checklist entries in `current_spec` to reflect the actual completed state after the skill-driven review results are incorporated.
16. Re-run relevant validation and fix all introduced errors and warnings.
> [!Important]
> Checklist state must reflect reality after the fixes, not reviewer intent. Leave items unchecked when implementation, validation, or acceptance is still incomplete.

## Scope Rules

- Review only `current_task` (if given)
- Do not broaden scope beyond fixes required for correctness, integration, or validation of `current_task`  (if given)
- If another issue blocks correct review of `current_task`, fix the blocker and keep the scope narrow.
- Make reasonable assumptions from the spec and codebase when details are missing, and ask questions only when genuinely blocked.
- Prefer root-cause fixes over narrow patches when both remain within task scope.
- The `/crypto-architect`, `/security-audit`, and `/doublecheck` steps are conditional execution gates selected by the routing matrix and the review evidence; they are not mandatory on every task.
- Do not present a final review verdict, checklist update, or completion claim unless all triggered skill runs were executed and their material findings were incorporated.
- Build the review, fixes, checklist updates, and final judgment on the results of the triggered skill runs.
- If any triggered skill is unavailable, incomplete, or returns unresolved material risk, fail closed and report the exact blocker instead of soft-approving the task.

## Mandatory Context

You must read and follow these files before and during review:

- [.github/requirements/Z00Z_DESIGN_FOUNDATION.md](../requirements/Z00Z_DESIGN_FOUNDATION.md)
- [.github/copilot-instructions.md](../copilot-instructions.md)

All requirements in those files must be satisfied and verified.

## Naming And Design Constraints

- All new code, comments, and documentation must be in English.
- All functions and identifiers must satisfy the `@MUST` naming constraints.
- No identifier may exceed 5 words under the project word-count rules.
- Preserve the one-source-of-truth architecture rules.
- Do not introduce alternative designs when the spec already defines the shape.

## Validation Rules

Always validate with release-safe commands when relevant:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test --release
bash scripts/audit/audit_release_feature_guards.sh
```

Do not normalize `test-params-fast` or `wallet_debug_tools` on release-capable
builds unless the current task explicitly documents an internal-only exception.
Use smaller targeted commands while iterating if useful, but do not finish
without the required validation scope unless blocked.
Batch related fixes logically, then validate. Do not leave the task in a half-reviewed or half-fixed state.

Completion requires all of these:

- the implementation for `current_task` has been reviewed critically;
- any triggered specialist skills were run and their material findings were integrated into the review and fixes;
- `/doublecheck` was run on the review evidence when the review contained substantive verifiable claims, and all material verification issues were resolved or explicitly blocked;
- all found in-scope issues have been fixed;
- all completed checklist items for `current_task` are marked in `current_spec`;
- all relevant errors and warnings introduced by the work are fixed;
- required validation has been run or an exact blocker is stated;
- no open blocker, TODO, or partial review step remains hidden in the working state.

## Output Style

- Give short, direct, informative updates.
- Present findings first when reviewing.
- Prefer evidence-backed findings with exact spec/code references.
- State which files were reviewed and fixed.
- State what was fixed, what checklist items were updated, and what validation passed or failed.
- If no substantive issues are found, state that explicitly.
- Do not write long Markdown reports.

## Non-Negotiable Rules

- Do not skip checklist updates.
- Do not leave found in-scope issues unresolved.
- Do not move to a new task before the current task review is complete.
- Do not ignore warnings caused by the task implementation.
- Do not waste time on low-value nitpicks while critical correctness or spec issues remain.
- Follow all instructions above without exception.

## Example Invocation

`/GSD-Review-Tasks-Execution current_spec=specs/014-z00z-storage/jmt-migration-spec.md current_task="12.1 Step 1. Freeze The Semantic Boundary"`
