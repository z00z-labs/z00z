---
phase: 066-Strix
plan: 066-10
status: complete
completed_at: 2026-07-03
next_plan: 066-11
summary_artifact_for: .planning/phases/066-Strix/066-10-PLAN.md
requirements_completed:
  - REQ-016
  - REQ-017
  - REQ-018
  - REQ-019
  - REQ-020
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-10 Summary: Portable Pack Unpack Integration

## Outcome

`066-10` is complete.

Phase 066 now has one canonical portable pentest entrypoint at
`./z00z_penetration_tests.sh`. The lane adds an archive-driven Docker wrapper,
teaches `pack_z00z_project.sh` to exclude heavy penetration payload caches by
default, adds a pentest-only restore hook to `unpack_z00z_project.sh`, and
lands validator plus regression coverage that prove the Docker path does not
delegate to the formal-verification restore flow.

The result stays on one repository-local authority path: portable input comes
from `pack_z00z_project.sh`, extraction and symlink verification reuse the
existing unpack logic with a safe `--skip-formal-verification` mode, and Docker
check-only or local runs flow through the same top-level pentest entrypoint
instead of inventing a second orchestration surface.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-10-SUMMARY.md`
- `z00z_penetration_tests.sh`
- `pack_z00z_project.sh`
- `unpack_z00z_project.sh`
- `scripts/penetration/check_pentest_tools.sh`
- `scripts/penetration/validate_pentest_docker_scope.py`
- `tools/penetration/docker/run_pentest_container.sh`
- `tools/penetration/manifests/tool-versions.lock`
- `tests/penetration/test_packaging_portability.py`
- `tests/penetration/test_docker_scope.py`

## Landed Changes

- Canonical portable entrypoint
  - Added `./z00z_penetration_tests.sh` as the human and agent entrypoint for
    local runs and archive-driven Docker runs.
  - Local mode forwards the canonical flag set to
    `scripts/penetration/run_local_pentest.sh`.
  - Docker mode accepts `--archive <path>` or auto-packs a fresh tarball before
    launching the pentest-only container path.
- Portable archive hardening
  - `pack_z00z_project.sh` now excludes heavy `tools/penetration/` payload
    homes by default, including cache, cargo, go, and local Python tool
    payload directories.
  - The packed manifest records those exclusions explicitly so the portable
    archive remains source- and wrapper-oriented instead of shipping heavy
    downloaded tool state.
- Safe unpack reuse
  - `unpack_z00z_project.sh` now supports `--skip-formal-verification`.
  - That mode still performs extraction, placeholder replacement, planning
    runtime restore, symlink verification, and the optional penetration-tool
    check hook, but it skips the formal-verification install and verify chain.
- Docker wrapper and validator
  - Added `tools/penetration/docker/run_pentest_container.sh`.
  - The wrapper requires an archive, extracts it inside the container, runs the
    safe unpack path, validates the Docker scope, executes check-only or local
    pentest flow against the extracted workspace, and copies artifacts or host
    reports back to the host.
  - Added `scripts/penetration/validate_pentest_docker_scope.py` to reject
    forbidden calls to formal-verification flows or the unpack Docker sandbox
    path.
- Regression coverage
  - Added `tests/penetration/test_packaging_portability.py`.
  - Added `tests/penetration/test_docker_scope.py`.
  - Refreshed `tools/penetration/manifests/tool-versions.lock` through the
    canonical tool-status path so the portable flow references current
    penetration-tool status truth.

## Validation

Commands and evidence used for `066-10` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `bash -n z00z_penetration_tests.sh tools/penetration/docker/run_pentest_container.sh unpack_z00z_project.sh pack_z00z_project.sh scripts/penetration/check_pentest_tools.sh`
- `python3 -m py_compile scripts/penetration/validate_pentest_docker_scope.py tests/penetration/test_packaging_portability.py tests/penetration/test_docker_scope.py`
- `python3 -m unittest tests.penetration.test_packaging_portability tests.penetration.test_docker_scope tests.penetration.test_codex_surface_integration`
- `python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration`
- `./z00z_penetration_tests.sh --mode quick --static-only --check-only --artifact-dir /tmp/z00z-pentest-local-check`
- `./pack_z00z_project.sh --output /tmp/z00z-pentest-plan-check.tar.gz --keep-tmp`
- `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-plan-check.tar.gz --mode check-only`
- `./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-plan-check.tar.gz --mode quick --static-only --check-only`
- `./z00z_penetration_tests.sh --docker-sandbox --mode quick --static-only --check-only`
- `bash scripts/penetration/check_pentest_tools.sh --json >/tmp/z00z-pentest-tool-status-refresh.json`
- `git diff --check`

Observed proof points:

- The mandatory `bootstrap_tests.sh` gate completed green before `066-10`
  implementation started, and it completed green again on the final `066-10`
  tree.
- The new portability tests passed green and proved that the packed archive
  includes the canonical pentest sources while excluding heavy penetration
  payload caches.
- The live Docker-scope validator passed green on the repository scripts and
  rejected a forbidden formal-verification invocation fixture.
- The local canonical entrypoint completed a quick static check-only run and
  produced a valid artifact or report pair.
- The direct Docker wrapper completed green against the packed archive in
  check-only mode, proving archive extraction, safe unpack, symlink
  verification, Docker-scope validation, host artifact export, and host report
  export without invoking formal verification.
- The supplied-archive and auto-pack Docker journeys both completed green
  through `./z00z_penetration_tests.sh`.
- The latest `docker-run.json` under
  `.security-artifacts/20260702T212149Z/` records the host archive path,
  extracted container root, container artifact or report roots, and the
  Docker-scope validator artifact.

`cargo test --release` was not rerun as a separate top-level command for
`066-10` because no Rust runtime code changed; the mandatory bootstrap gate
already reran green on release-mode workspace targets after the final edits.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a usable automated prompt-execution
path for this slice.

- Attempt 1
  - `./.github/prompts/gsd-review-tasks-execution.prompt.md current_spec=.planning/phases/066-Strix/066-10-PLAN.md current_task="WS-10" --yolo`
  - Result: failed with `Permission denied` because the prompt file is not an
    executable entrypoint.
- Attempt 2
  - `timeout 60s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-10-PLAN.md current_task="WS-10" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 3
  - `timeout 60s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-10-PLAN.md current_task="WS-10" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83627 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-10-PLAN.md`, the `WS-10` row in `066-TODO.md`, the new
    portable entrypoint, the pack/unpack seams, the Docker wrapper, the
    validator, and the new tests.
  - Found and fixed five in-scope issues: a `pipefail`-triggered `141` exit in
    Docker top-level archive detection, missing `PyYAML` in the container path
    for `validate_scope.py`, host artifact or report symlink export that broke
    artifact identity validation, inline Python quoting drift in
    `docker-run.json` generation, and missing host-archive provenance fields in
    the Docker run manifest.
- Pass 2
  - Re-ran the portability or Docker-scope unit tests, the live validator, the
    local canonical entrypoint, fresh pack generation, the direct Docker
    wrapper path, and the supplied-archive canonical entrypoint path.
  - Result: clean for the `066-10` scope.
- Pass 3
  - Re-ran the auto-pack canonical entrypoint path, refreshed the direct Docker
    wrapper evidence so `docker-run.json` carries host archive provenance, and
    reran the mandatory bootstrap gate on the final tree.
  - Result: clean for the `066-10` scope.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
patch.

## Closeout

`066-10` closes `WS-10` by landing the archive-driven pentest entrypoint, the
safe unpack hook, the pentest-only Docker wrapper and scope validator, the
heavy-cache exclusions in the pack path, and executable portability coverage
for local, supplied-archive, and auto-pack journeys.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-11-PLAN.md`.
