---
phase: 066-Strix
plan: 066-08
status: complete
completed_at: 2026-07-02
next_plan: 066-09
summary_artifact_for: .planning/phases/066-Strix/066-08-PLAN.md
requirements_completed:
  - REQ-006
  - REQ-007
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-08 Summary: Local DAST Scope Runner

## 🎯 Outcome

`066-08` is complete.

Phase 066 now has one canonical local-only DAST path under
`scripts/penetration/run_local_dast.sh` that validates scope before any
network-capable command, records a clean skip artifact when no allowed local
DAST target exists, rejects public targets before tool execution, and keeps
default dynamic execution bounded to the owned `nmap`, `nuclei`, `httpx`,
`katana`, and `ffuf` path only.

The lane also closes the missing file contract in `066-CONTEXT.md` and
`066-08-PLAN.md` by adding the required Strix DAST reference copies under
`.github/skills/pentest-local-dast/references/strix/`, and it closes a local
inventory blocker by registering `nmap` on the same canonical
`tools/penetration` checker path that the rest of Phase 066 already uses.

## 📦 Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-08-SUMMARY.md`
- `.github/skills/pentest-local-dast/SKILL.md`
- `.github/skills/pentest-local-dast/references/strix/nmap.md`
- `.github/skills/pentest-local-dast/references/strix/nuclei.md`
- `.github/skills/pentest-local-dast/references/strix/httpx.md`
- `.github/skills/pentest-local-dast/references/strix/katana.md`
- `.github/skills/pentest-local-dast/references/strix/ffuf.md`
- `scripts/penetration/run_local_dast.sh`
- `scripts/penetration/check_pentest_tools.sh`
- `scripts/penetration/install_pentest_tools.sh`
- `tools/penetration/README.md`
- `tools/penetration/manifests/tool-versions.lock`
- `tests/penetration/test_dast_runner_integration.py`

## 🔧 Landed Changes

- Canonical DAST runner expansion
  - `run_local_dast.sh` now performs scope validation before any DAST command,
    writes skip or failure artifacts explicitly, and uses one bounded execution
    path only.
  - The default path now uses a two-pass `nmap` model: one small discovery pass
    over scoped hosts, then one enrichment pass only for discovered open ports.
  - `httpx`, `nuclei`, and `katana` now run with explicit bounded flags,
    rate limits, concurrency limits, timeouts, retries, and structured output.
  - `ffuf` now runs with a generated small wordlist, explicit rate/thread/time
    limits, `-noninteractive`, and JSON output.
- DAST artifact and status hygiene
  - Per-tool or per-target status files now stay unique in `dast/summary.json`;
    the duplicate `status_files` path bug in the list-runner helper was removed.
  - The no-target path still writes `dast/skipped.json`, and the public-target
    rejection path still writes `dast/validation-failed.json`.
- Canonical tool inventory repair
  - `nmap` is now part of the same `tools/penetration` checker and wrapper
    inventory surface as the rest of the DAST tool family.
  - The local tool root README and lock manifest now reflect the expanded DAST
    inventory truth.
- Reference corpus closure
  - Added curated Strix reference copies for `nmap`, `nuclei`, `httpx`,
    `katana`, and `ffuf` with provenance headers and reference-only
    disposition.
- Regression coverage
  - Added `tests/penetration/test_dast_runner_integration.py`.
  - The new integration suite proves public-target rejection before tool
    execution, no-target skip behavior, and bounded command construction for
    the two-pass `nmap` plus local URL tool path.

## ✅ Validation

Commands and evidence used for `066-08` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `bash -n scripts/penetration/run_local_dast.sh scripts/penetration/check_pentest_tools.sh scripts/penetration/install_pentest_tools.sh`
- `bash scripts/penetration/install_pentest_tools.sh --json`
- `python3 -m unittest tests.penetration.test_scope_validation tests.penetration.test_dast_runner_integration tests.penetration.test_artifact_schema tests.penetration.test_report_builder`
- `python3 -m unittest tests.penetration.test_dast_runner_integration tests.penetration.test_scope_validation`
- `python3 scripts/penetration/validate_artifacts.py .security-artifacts/20260702T202315Z --json`
- `git diff --check`
- `find .github/skills/pentest-local-dast/references/strix -maxdepth 1 -type f | sort`
- no-forbidden-invocation assertion against `scripts/penetration/run_local_dast.sh`
- docs grep for `reference-only`, `Do not run`, and `default DAST path` in `.github/skills/pentest-local-dast/SKILL.md`
- uniqueness assertion for `dast/summary.json` `status_files` on a local fixture run

Observed proof points:

- The mandatory `bootstrap_tests.sh` gate completed green before `066-08`
  implementation continued.
- The new integration suite passed green and proved the three critical `WS-08`
  behaviors: early public-target rejection, explicit no-target skip, and
  bounded local command construction.
- The refreshed tool-state output now includes `nmap` on the canonical local
  checker path and reports the truthful `missing` state when the local payload
  is not installed.
- The bounded-command sweep found no forbidden default-tool invocation path in
  `run_local_dast.sh`.
- The DAST skill now contains the required reference-only and default-path
  exclusion wording.
- The uniqueness check confirmed that `dast/summary.json` no longer duplicates
  `status_files`.
- Diff hygiene checks were clean.

`cargo test --release` was not rerun as a separate command for `066-08`
because no Rust runtime code changed and the mandatory bootstrap gate already
completed green on the release path.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a usable automated prompt-execution
path for this slice.

- Attempt 1
  - `./.github/prompts/gsd-review-tasks-execution.prompt.md current_spec=.planning/phases/066-Strix/066-08-PLAN.md current_task='WS-08'`
  - Result: failed with `Permission denied` because the prompt file is not an
    executable entrypoint.
- Attempt 2
  - `timeout 60s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-08-PLAN.md current_task="WS-08"'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66666 > 38936`
- Attempt 3
  - `timeout 60s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-08-PLAN.md current_task="WS-08"'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83625 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-08-PLAN.md`, the `WS-08` row in `066-TODO.md`, the Phase 066
    context or test-spec anchors, the DAST skill, the runner, the tool
    inventory scripts, and the new DAST integration tests.
  - Found and fixed four in-scope issues: missing required DAST reference
    files, missing `nmap` inventory registration on the canonical tool-status
    path, duplicate `status_files` entries in `dast/summary.json`, and a too
    loose `nuclei` command that lacked an explicit template-family bound.
- Pass 2
  - Re-ran shell syntax checks, the full Phase 066 Python regression slice,
    the live artifact validator, and diff hygiene.
  - Result: clean for the `066-08` scope.
- Pass 3
  - Re-ran the focused DAST integration or scope suite, the refreshed
    installer/checker inventory path, the no-forbidden-invocation assertion,
    the DAST docs grep, and the `status_files` uniqueness fixture check.
  - Result: clean for the `066-08` scope.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
patch.

## 🧾 Closeout

`066-08` closes `WS-08` by landing the canonical local DAST runner behavior,
the missing Strix DAST reference corpus, the repaired DAST tool inventory
surface, and executable regression coverage that proves the bounded scope and
artifact rules end to end.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-09-PLAN.md`.
