---
name: "GSD-Audit-4"
agent: agent
description: "Use when you need to audit all crates in a phase directory with four mandatory audit passes, generate a structured FULL-AUDIT report, fix actionable findings in the same execution, rerun the audit, and finish with doublecheck-backed closure evidence."
argument-hint: "phase_dir = <path>"
---

# GSD Audit 4

Audit all in-scope crates for a phase directory, generate or append a rigorously formatted phase FULL-AUDIT report, fix all actionable findings in YOLO mode, rerun the audit, then run doublecheck and fix any remaining actionable issues.

## 🎯 Mission

Own the workflow end to end: scope discovery, audit, proof-oriented reporting, direct fixes, re-audit, doublecheck, and closeout.

Do not stop at recommendations.
Do not treat partial execution as completion.
Judge success by clean rerun evidence and truthful closure reporting in the final FULL-AUDIT file.

> [!IMPORTANT]
> The FULL-AUDIT file is a proof artifact, not a loose note dump. Every audit run must follow the canonical report format in [references/gsd-audit-4-full-audit-report-format.md](references/gsd-audit-4-full-audit-report-format.md).

## 📥 Input

- `phase_dir`: full or relative path to the phase directory.

If `phase_dir` is missing, invalid, or ambiguous, stop and resolve that first.

## 📂 Derived Paths

Derive all other paths from `phase_dir`.

1. Read all artifacts inside `phase_dir` and treat them as the source of truth for audit scope.
2. Derive the phase prefix from the directory basename before the first `-`.
  - Example: `031-refactor-architecture` -> prefix `031`
3. Create or append to the audit file at:
  - `<phase_dir>/<prefix>-FULL-AUDIT.md`
  - Example: `.planning/phases/031-refactor-architecture/031-FULL-AUDIT.md`
4. If the phase prefix cannot be derived safely, fail closed and report the blocker instead of guessing a file name.

## 🧭 Required Audit Scope

You must verify all of the following:

- the exact crate scope implied by `phase_dir` artifacts;
- all crates explicitly named by the phase documents;
- all crates materially implied by file targets, implementation notes, or audit notes inside the phase directory;
- exclusion of crates that are outside the phase scope;
- full append-only audit history in the phase FULL-AUDIT file;
- direct code fixes for all actionable findings;
- re-audit after fixes;
- final `doublecheck` verification and final cleanup of remaining actionable issues.

## 🔍 Mandatory Audit Passes

For every in-scope crate, run these audit passes in this exact order:

1. `crypto-architect` as auditor
2. `security-audit`
3. `spec-to-code-compliance`
4. `z00z-design-foundation-compliance`

If a named skill cannot be invoked directly, apply its documented audit logic manually and mark that audit pass as `manual fallback` in the FULL-AUDIT file.

## 🧪 Mandatory Verification Model

Before judging closure, explicitly derive from the phase artifacts and live code all relevant:

- critical user journeys;
- state transitions;
- proof paths;
- failure paths;
- end-to-end behaviors that must be proven;
- critical integration paths;
- successful execution examples;
- negative scenarios that must prove rejection or failure handling;
- assertions that prove correctness;
- measurable success or failure conditions.

> [!WARNING]
> If a journey, transition, proof path, or failure path cannot be proven end to end, record that as a closure gap. Do not silently narrow scope.

## 🛠️ Required Execution Process

1. Read the full `phase_dir` contents before deciding crate scope.
2. Determine the final in-scope crate list only from `phase_dir` artifacts.
3. State the final target crate list before starting audits.
4. Derive the FULL-AUDIT path from `phase_dir` and use it as the single append-only audit log.
5. Run all four mandatory audit passes for every in-scope crate.
6. Write the audit run using the exact section order and formatting rules from [references/gsd-audit-4-full-audit-report-format.md](references/gsd-audit-4-full-audit-report-format.md).
7. Use timestamps in the exact format `YYYY-MM-DD HH:MM:SS`.
8. Use H1 through H4 headings and semantic leading emoji on H2 through H4 headings.
9. Use GitHub alert blocks where relevant:
  - `NOTE`
  - `TIP`
  - `IMPORTANT`
  - `WARNING`
  - `CAUTION`
10. Place a `Findings Summary` section before detailed findings and group findings by severity.
11. For every material finding, use the mandatory finding card structure from the reference format, including:
  - issue title;
  - location;
  - code snippet or proof snippet when relevant;
  - why the issue matters;
  - recommendation;
  - severity;
  - category;
  - proof status;
  - verification status.
12. After all audit passes are complete, fix all actionable findings directly in code in YOLO mode unless they are explicitly blocked by wider-scope constraints.
13. Append a `Fixes Applied` section that states exactly what changed and what remains blocked.
14. Rerun the same four audit passes on the same in-scope crate list.
15. Append a `Re-Audit Results` section with exact commands, evidence, and current disposition.
16. Run `doublecheck` on both:
   - the final code changes;
   - the final contents of the FULL-AUDIT file.
17. Fix all remaining actionable issues found by `doublecheck` in YOLO mode.
18. Append a `Doublecheck Results` section to the FULL-AUDIT file.
19. End the report with one consolidated `Exact Fixes Required Summary` table using this exact column order:
  - `Q`
  - `Title`
  - `Proof Status`
  - `Verification`
  - `Severity`
  - `Missing Evidence Or Blocker`
  - `Gap Closure Path`
20. Finish only when all actionable issues are fixed or explicitly marked as blocked with reasons.

## 🧱 FULL-AUDIT Output Rules

- The FULL-AUDIT file must be append-only.
- Each audit run must open with an audit-run section that includes setup, scope, verification model, and findings summary.
- Each run must close with the final summary table and a short final-status section.
- No free-form closure claims are allowed without repository-backed evidence.
- Every broader gap that remains open must appear in the final summary table.

> [!TIP]
> Treat the final summary table as the canonical closure ledger for the audit run.

## 🚫 Execution Rules

- Act, do not only recommend.
- Stay inside the current execution; do not recursively invoke `gsd-quick` from inside this prompt.
- Do not use `skill-selector`.
- Do not overwrite the FULL-AUDIT file; append only.
- Do not widen the crate scope beyond what `phase_dir` supports.
- Do not stop after the first audit wave.
- Do not treat partial execution as completion.
- Do not leave actionable findings unaddressed without an explicit blocker.
- Do not invent scope, crate names, or audit file names when the phase directory does not prove them.
- Do not mark a phase fully closed if unresolved `🔴 CRITICAL` or `🟠 HIGH` closure gaps remain in the final summary table.

> [!Caution]
> If the phase directory does not provide enough evidence to determine in-scope crates safely, fail closed and report the exact ambiguity instead of guessing the crate list.

## 📚 Mandatory Context

You must read and follow these files before and during execution:

- [.github/requirements/Z00Z_DESIGN_FOUNDATION.md](../requirements/Z00Z_DESIGN_FOUNDATION.md)
- [.github/copilot-instructions.md](../copilot-instructions.md)
- [references/gsd-audit-4-full-audit-report-format.md](references/gsd-audit-4-full-audit-report-format.md)

All requirements in those files must be satisfied.

## ✅ Completion Criteria

Completion requires all of these:

- `phase_dir` was read end to end;
- the in-scope crate list was derived from the phase directory and stated explicitly;
- all four mandatory audit passes were run for every in-scope crate;
- every audit result was appended to the phase FULL-AUDIT file in the canonical format;
- all actionable findings were fixed directly in code or explicitly marked as blocked with reasons;
- a `Fixes Applied` section was appended;
- the full re-audit was executed and appended;
- `doublecheck` was executed on both the final code changes and the FULL-AUDIT file;
- all remaining actionable `doublecheck` findings were fixed or explicitly marked as blocked;
- the final `Exact Fixes Required Summary` table was appended with `Severity` after `Verification`;
- the final state is supported by append-only evidence in the FULL-AUDIT file.

## 🗣️ Output Style

- Give short, direct execution updates.
- State the phase directory being processed.
- State the derived FULL-AUDIT file path.
- State the final target crate list before audits start.
- State which audit pass is running.
- State what was fixed.
- State what was re-audited.
- State whether `doublecheck` found any remaining material issues.
- State blockers explicitly if any remain.

## 🛑 Non-Negotiable Rules

- Do not skip any of the four audit passes.
- Do not skip append logging to the FULL-AUDIT file.
- Do not skip the fix phase.
- Do not skip the re-audit phase.
- Do not skip `doublecheck`.
- Do not leave found actionable issues unresolved.
- Do not require more than `phase_dir` as input.
- Follow all instructions above without exception.

## 📎 Example Invocation

`/gsd-audit-4 phase_dir=.planning/phases/031-refactor-architecture`
