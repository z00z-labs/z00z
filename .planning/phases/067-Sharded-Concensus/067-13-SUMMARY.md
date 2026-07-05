---
phase: 067
plan: 067-13
status: complete
completed_at: 2026-07-05
next_plan: 067-14
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-13-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-13 Summary: Multi Process Devnet Harness

## Outcome

`067-13` is complete.

`VERDICT-LCS-04` now closes on one live local process/devnet path.
`z00z_rollup_node` exposes one canonical process-backed hold mode through
`process_devnet.rs`, and both `scripts/hjmt_local_devnet.sh` and
`docker/compose.hjmt-local.yaml` route to that same manifest-driven runtime
surface instead of inventing a second harness path.

The closeout turns the former manifest-only process language into executable
local evidence. Five `SIM-5A7S` aggregator identities can now start as separate
OS processes with distinct ports, data directories, log paths, run ids, and
persisted restart state; stale process directories fail closed; and
`scenario_11` now records the honest boundary that this is a local
`simulated-full` devnet/process claim rather than production networking. This
plan still does not claim real network transport, HotStuff, or real Celestia
finality. Those remain later verdict-lane work, with `067-14` now the next
canonical execution lane.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/067-Sharded-Concensus/067-13-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-13-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `crates/z00z_rollup_node/Cargo.toml`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/src/main.rs`
- `crates/z00z_rollup_node/src/process_devnet.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_process_devnet.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`
- `docker/compose.hjmt-local.yaml`
- `scripts/hjmt_local_devnet.sh`

## Landed Changes

- Canonical process-devnet contract
  - Added `crates/z00z_rollup_node/src/process_devnet.rs` as the single live
    process-devnet seam for run-root paths, ready or heartbeat or event files,
    persisted boot state, stale-dir rejection, and stop-file handling.
  - `crates/z00z_rollup_node/src/main.rs` now calls
    `maybe_run_hjmt_process_devnet(&launch)` after the existing ready print, so
    the same release binary supports both the ordinary quick-exit readiness path
    and the held local process harness path.
- Manifest-driven smoke harness and optional Compose wrapper
  - Added `scripts/hjmt_local_devnet.sh` as the canonical repo-owned smoke
    entry point for `SIM-5A7S`.
  - Added `docker/compose.hjmt-local.yaml` as a thin optional wrapper around
    that same script rather than a second source of truth.
- Process realism, restart evidence, and fail-closed negative paths
  - Added `crates/z00z_rollup_node/tests/test_hjmt_process_devnet.rs` to prove
    five-process startup, unique identity or port or data-dir or log-dir
    assignment, kill or restart with persisted boot-count reload, duplicate-port
    rejection, missing-binary rejection, stale-dir rejection, and missing-run-id
    rejection.
  - The smoke run emits `process-devnet-smoke.json`, and the script emits
    `process-devnet-evidence.json` under `reports/hjmt-local-devnet/<run_id>/`.
- Honest scenario boundary
  - Extended `crates/z00z_simulator/tests/test_scenario_11.rs` with
    `scenario11_process_devnet_fault_contract` so the phase report explicitly
    distinguishes live local process-backed quorum or restart evidence from
    still-simulated transport or partition behavior.

## Validation

Commands green during the `067-13` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process_devnet -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process -- --nocapture`
- `bash scripts/hjmt_local_devnet.sh --profile sim_5a7s --smoke --timeout 30`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`

Broad release-gate note:

- The current-cycle `cargo test --release` rerun completed green after the
  process-devnet landing; no additional repair beyond the landed `067-13` write
  set was required in this closeout slice.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-13-PLAN.md current_task="067-13-T1" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-13-PLAN.md current_task="067-13-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`.
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-13-PLAN.md current_task="Multi Process Devnet Harness" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.

Equivalent workspace-first manual review was executed with the `/doublecheck`
three-layer posture against the same scope.

- Pass 1
  - Re-ran anchored grep for the canonical process-devnet seam markers
    `HJMT_PROCESS_MODE_ENV`, `Z00Z_HJMT_PROCESS_MODE`,
    `Z00Z_HJMT_RUN_ID`, `process-ready.json`, `process-heartbeat.json`,
    `process-events.jsonl`, and `process-state.json` across the touched
    rollup-node runtime and test surfaces.
  - Result: clean. One canonical process-devnet path remained under the
    reviewed scope.
- Pass 2
  - Re-ran anchored grep for the harness and evidence markers
    `hjmt_local_devnet.sh`, `compose.hjmt-local.yaml`,
    `process-devnet-smoke.json`, `process-devnet-evidence.json`, and
    `scenario11_process_devnet_fault_contract` across the touched repo and
    planning surfaces.
  - Result: clean. The script, optional Compose wrapper, tests, and evidence
    names agree on one live local smoke path.
- Pass 3
  - Ran `git diff --check` across the touched `067-13` code and planning
    artifacts.
  - Result: clean.
- Pass 4
  - Re-read `067-13-PLAN.md`, `067-13-SUMMARY.md`, `067-COVERAGE.md`,
    `067-verdict.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md` after
    the final status sync.
  - Result: clean.

Passes 3 and 4 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-13` closes `VERDICT-LCS-04` by turning the `SIM-5A7S` process model into
an executable local process-backed claim with persisted restart evidence and one
canonical smoke path.

`067-14` is now the next canonical execution lane.
