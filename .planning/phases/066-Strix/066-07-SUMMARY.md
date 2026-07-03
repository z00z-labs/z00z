---
phase: 066-Strix
plan: 066-07
status: complete
completed_at: 2026-07-02
next_plan: 066-08
summary_artifact_for: .planning/phases/066-Strix/066-07-PLAN.md
requirements_completed:
  - REQ-007
  - REQ-019
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-07 Summary: Artifact Schema And Report Contract

## 🎯 Outcome

`066-07` is complete.

Phase 066 now has one explicit report schema for
`.security-artifacts/<timestamp>/`, one explicit findings schema for
`normalized/findings.json`, one canonical host report root under
`reports/z00z-pentests_report-<timestamp>/`, and one fail-closed evidence gate
that rejects any `confirmed` finding without source evidence, scanner
artifact, local proof, confidence, fix recommendation, and regression test.

The landed result makes the report contract usable by humans and automation
without reading raw scanner noise, while still preserving raw artifacts in the
machine-readable tree.

## 📦 Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-07-SUMMARY.md`
- `.security/report-template.md`
- `.github/skills/pentest-report/references/report-schema.md`
- `scripts/penetration/build_pentest_report.py`
- `scripts/penetration/validate_artifacts.py`
- `tests/penetration/test_artifact_schema.py`
- `tests/penetration/test_report_builder.py`

## 🔧 Landed Changes

- Canonical report template
  - Added `.security/report-template.md` with the required host-facing section
    order, evidence rules, redaction notes, and doublecheck slot.
- Canonical schema reference
  - Added `.github/skills/pentest-report/references/report-schema.md`.
  - The schema now defines the artifact-root shape, manifest contract,
    normalized findings contract, severity values, status values, and the
    host-report rules.
- Findings schema enforcement
  - `build_pentest_report.py` now reads `normalized/findings.json` when it is
    present.
  - The builder now rejects invalid `confirmed`, `false-positive`,
    `unconfirmed`, or `skipped` finding rows instead of silently downgrading
    them.
- Report summary and detail sections
  - The report metadata now records severity counts, confirmed count,
    false-positive count, unconfirmed count, skipped count, and signal counts.
  - The Markdown report now includes findings summary and per-finding detail
    sections when findings are present.
- Artifact validator hardening
  - `validate_artifacts.py` now validates finding severity, finding status,
    host report path pairing, and evidence requirements for `confirmed`
    findings.
- Regression tests
  - Added builder tests that prove host report generation and fail-closed
    rejection of scanner-only `confirmed` findings.
  - Added validator tests that prove a built artifact tree passes and that an
    invalid report metadata packet fails.

## ✅ Validation

Commands and evidence used for `066-07` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `python3 -m py_compile scripts/penetration/*.py tests/penetration/*.py`
- `python3 -m unittest tests/penetration/test_artifact_schema.py tests/penetration/test_report_builder.py`
- `python3 scripts/penetration/validate_artifacts.py .security-artifacts/20260702T200651Z`
- `python3 scripts/penetration/build_pentest_report.py --artifact-dir .security-artifacts/20260702T200651Z --report-dir reports/z00z-pentests_report-20260702T200651Z --profile generic`
- `rg -n "normalized/findings.json|confirmed finding|false-positive|unconfirmed|skip_reason|report_dir|security-report.md|report-metadata.json|z00z-pentests_report" scripts/penetration/build_pentest_report.py scripts/penetration/validate_artifacts.py .security/report-template.md .github/skills/pentest-report/references/report-schema.md tests/penetration/test_artifact_schema.py tests/penetration/test_report_builder.py`
- `git diff --check -- scripts/penetration/build_pentest_report.py scripts/penetration/validate_artifacts.py .security/report-template.md .github/skills/pentest-report/references/report-schema.md tests/penetration/test_artifact_schema.py tests/penetration/test_report_builder.py`

Observed proof points:

- The mandatory `bootstrap_tests.sh` gate completed green after the `066-07`
  schema, builder, validator, and test artifacts landed.
- The new unittest modules passed green and proved both the positive build path
  and the negative evidence-gate path.
- The real artifact run at `.security-artifacts/20260702T200651Z/` was rebuilt
  by the updated report builder and still passed the artifact validator.
- The rebuilt `report-metadata.json` now records the new summary shape,
  including `severity_counts`, `false_positive_total`,
  `unconfirmed_finding_total`, and `skipped_finding_total`.
- Diff hygiene checks were clean.

`cargo test --release` was not rerun as a separate command for `066-07`
because no Rust runtime code changed and the mandatory bootstrap gate already
completed green on the release path.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime did not provide a usable automated prompt-execution path
for this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-07-PLAN.md current_task="WS-07" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-07-PLAN.md current_task="WS-07" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-07-PLAN.md current_task="WS-07" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-07-PLAN.md`, `066-TODO.md`, `build_pentest_report.py`,
    `validate_artifacts.py`, the new template, the new schema reference, and
    the new unittest modules.
  - Checked that `confirmed` findings fail closed without evidence and that
    every human-facing report path stays under the paired host report root.
  - Result: no material drift found.
- Pass 2
  - Re-ran py-compile, the two new unittest modules, the live artifact
    validator, and the live report rebuild command.
  - Result: clean for the `066-07` scope.
- Pass 3
  - Re-ran schema-string grep, diff hygiene checks, and live metadata
    inspection against `.security-artifacts/20260702T200651Z/report/report-metadata.json`.
  - Result: clean for the `066-07` scope. No scanner-only confirmed finding
    path or alternate host-report export root remained.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
patch.

## 🧾 Closeout

`066-07` closes `WS-07` by landing the canonical report template, the report
schema reference, the normalized findings evidence gate, the hardened report
builder, the hardened artifact validator, and regression tests that prove both
the positive and negative contract paths.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-08-PLAN.md`.
