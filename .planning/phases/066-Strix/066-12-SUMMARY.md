---
phase: 066-Strix
plan: 066-12
status: complete
completed_at: 2026-07-03
next_plan: 066-13
summary_artifact_for: .planning/phases/066-Strix/066-12-PLAN.md
requirements_completed:
  - REQ-012
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-12 Summary: Regression Tests And Self Tests

## Outcome

`066-12` is complete.

Phase 066 now has an executable offline regression suite for the local pentest
orchestration path. The suite stays on the existing repository-local
implementation, adds no parallel validator or report model, and proves the
core safety invariants with real scripts plus deterministic fixtures: local
scope acceptance, public-target rejection, denied-tool rejection, truthful
missing-tool recording, explicit DAST skip artifacts, report redaction,
artifact or host-report pairing, and `.codex` symlink integrity.

The missing `test_tool_manifest.py` lane is now landed, the fixture corpus now
exists under `tests/penetration/fixtures/{scope,tool_status,scanner_outputs,reports}/`,
and the report or artifact suites now consume those fixtures instead of relying
only on inline temporary payloads.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-12-SUMMARY.md`
- `tests/penetration/fixtures/reports/artifact_contract.json`
- `tests/penetration/fixtures/scanner_outputs/findings_missing_regression_test.json`
- `tests/penetration/fixtures/scanner_outputs/findings_mixed.json`
- `tests/penetration/fixtures/scanner_outputs/findings_scanner_only_confirmed.json`
- `tests/penetration/fixtures/scanner_outputs/raw_gitleaks_secret.json`
- `tests/penetration/fixtures/scanner_outputs/raw_semgrep_secret.json`
- `tests/penetration/fixtures/scope/broad_cidr_scope.yaml`
- `tests/penetration/fixtures/scope/local_url_scope.yaml`
- `tests/penetration/fixtures/scope/public_ip_scope.yaml`
- `tests/penetration/fixtures/scope/public_url_scope.yaml`
- `tests/penetration/fixtures/scope/source_only_scope.yaml`
- `tests/penetration/fixtures/scope/wildcard_host_scope.yaml`
- `tests/penetration/fixtures/tool_status/expected_contract.json`
- `tests/penetration/test_artifact_schema.py`
- `tests/penetration/test_codex_surface_integration.py`
- `tests/penetration/test_report_builder.py`
- `tests/penetration/test_scope_validation.py`
- `tests/penetration/test_tool_manifest.py`

## Landed Changes

- Scope and DAST-skip regression fixtures
  - Added fixture-backed scope variants for localhost URLs, public URL
    rejection, public IP rejection, wildcard-host rejection, broad-CIDR
    rejection, and source-only no-target scope.
  - `test_scope_validation.py` now proves localhost DAST URLs are accepted on
    the same validator path and still proves the `SKIP` outcome for a
    source-only scope with `--require-dast-targets`.
- Tool-root and provenance-manifest regression coverage
  - Added `test_tool_manifest.py` to prove the checker emits machine-readable
    JSON, keeps the canonical local root under `tools/penetration/`, writes
    the manifest locks under that root, records missing required tools
    truthfully, and fails closed in `--strict` mode even when an external
    binary is present on `PATH`.
  - Added a fixture contract for required tool names, payload suffixes,
    canonical environment roots, and forbidden path fragments such as
    `tools/formal_verification`.
- Artifact and report fail-closed coverage
  - Reworked `test_artifact_schema.py` and `test_report_builder.py` around a
    shared fixture corpus for mixed findings, scanner-only confirmed findings,
    missing regression-test anchors, secret-bearing raw scanner outputs, and
    expected report sections.
  - Added coverage for missing manifest, missing `manifest.report_dir`, missing
    paired host-report root, confirmed findings without evidence, confirmed
    findings without regression coverage, false-positive preservation, skipped
    finding preservation, stable severity ordering, redaction of raw secret
    material from Markdown, and the mandatory `Doublecheck` section.
- `.codex` negative-path regression
  - Extended `test_codex_surface_integration.py` with a broken-symlink
    fixture so the suite fails if the compatibility surface points at a
    missing canonical directory.

## Validation

Commands and evidence used for `066-12` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `python3 -m pytest tests/penetration`
- `python3 -m unittest discover tests/penetration`
- `bash -n scripts/penetration/*.sh`
- `python3 -m py_compile scripts/penetration/*.py tests/penetration/*.py`

Observed proof points:

- The mandatory `bootstrap_tests.sh` fail-fast gate completed green on the
  current tree before `066-12` closeout.
- `python3 -m pytest tests/penetration` could not run because this environment
  does not have `pytest` installed and returned
  `/usr/bin/python3: No module named pytest`.
- The documented fallback `python3 -m unittest discover tests/penetration`
  completed green twice on the final tree with `37` passing tests per run.
- `bash -n scripts/penetration/*.sh` completed green.
- `python3 -m py_compile scripts/penetration/*.py tests/penetration/*.py`
  completed green.
- The final fixture inventory now contains the required `scope`,
  `tool_status`, `scanner_outputs`, and `reports` subtrees under
  `tests/penetration/fixtures/`.

`cargo test --release` was not rerun as a separate top-level command for
`066-12` because no Rust code changed in this slice; the mandatory bootstrap
gate already reran green on the final tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still could not execute the automated prompt due an
external token-credit ceiling.

- Attempt 1
  - `gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-12-PLAN.md current_task="WS-12" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 2
  - `gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-12-PLAN.md current_task="WS-12" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 3
  - `gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-12-PLAN.md current_task="WS-12" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83627 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-12-PLAN.md`, the `WS-12` row in `066-TODO.md`,
    `066-TEST-SPEC.md`, and `066-TESTS-TASKS.md`, then compared them against
    the live `tests/penetration/` tree.
  - Landed the missing `test_tool_manifest.py` suite, the missing fixture
    subtrees, the localhost-URL positive scope case, the redaction and
    classification report cases, the host-report-root validator negatives, and
    the broken-symlink regression.
- Pass 2
  - Re-read the final changed test files and fixture contracts against the
    same `WS-12` requirements and reran the full `unittest` fallback suite.
  - Result: no new significant issues were found.
- Pass 3
  - Re-ran the `WS-12` spec-to-suite mapping, verified the final fixture
    inventory, and reran the full `unittest` fallback suite again.
  - Result: no new significant issues were found.

Passes 2 and 3 ended with consecutive clean reruns on the final `066-12` tree.

## Closeout

`066-12` closes `WS-12` by proving that the live local pentest orchestration
path now has executable, offline, phase-local regression coverage for its
safety boundaries, report contract, and compatibility surface without
inventing a second validator, a second report model, or a second phase
authority path.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-13-PLAN.md`.
