---
phase: 067
plan: 067-11
status: complete
completed_at: 2026-07-05
next_plan: 067-12
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-11-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-11 Summary: Durable Consensus Evidence Store

## Outcome

`067-11` is complete.

`VERDICT-LCS-02` now closes on one live durable consensus-evidence path. The
repository now persists commit subject, vote material, quorum certificate,
publication binding, published checkpoint anchor, and validator decision
evidence through one digest-bound consensus-store contract, and restart recovery
now reloads the exact subject or certificate path from disk or fails closed
before vote counting, DA continuation, or validator acceptance.

The closeout keeps the honesty boundary explicit. `067-11` proves durable local
restart evidence, but it does not overclaim planner HA, runnable multi-process
devnet proof, real network transport, HotStuff, or real Celestia finality.
Those remain later verdict-lane work, with `067-12` now the next canonical
execution lane.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-11-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-11-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/Cargo.toml`
- `crates/z00z_runtime/aggregators/src/consensus_store.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/tests/test_consensus_store.rs`
- `crates/z00z_runtime/aggregators/tests/test_consensus_recovery_restart.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/src/scenario_11/report.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`
- `crates/z00z_wallets/src/redb_store/mod.rs`

## Landed Changes

- Canonical durable consensus-store API
  - Added `crates/z00z_runtime/aggregators/src/consensus_store.rs` with one
    explicit backend contract `json_directory_v1`, one schema version
    `1`, one batch-record path `batches/<batch-id>.json`, and one route-cursor
    path `routes/<route-key>.json`.
  - Route cursor identity is digest-bound from live route metadata rather than
    display strings or fixture names.
- Persisted restart evidence is exact or fail-closed
  - `ConsensusStoreRecord` now binds `ShardRecoveryRecord`, `CommitSubject`,
    all `ShardVote` rows, `ShardQuorumCertificate`,
    `ConsensusStorePublication`, and `ConsensusValidatorDecision`.
  - Store load and verification reject schema drift, backend drift, route-key
    drift, header or recovery drift, missing vote material, publication-anchor
    drift, missing publication before validator decision, stale root drift, and
    corrupt JSON.
- Recovery now reloads from durable store
  - Added `PersistedConsensusRestart` plus
    `RecoveryBoundary::resume_from_store(...)` under
    `crates/z00z_runtime/aggregators/src/recovery.rs`.
  - The restart path now requires live placement or lineage or root parity and
    returns the persisted record together with the resumed execution ticket.
- Service and simulator paths now use one persistence seam
  - Added `publication_record_for_published(...)`,
    `validator_decision_snapshot(...)`,
    `persist_consensus_commit(...)`,
    `persist_consensus_publication(...)`, and
    `persist_validator_decision(...)` under
    `crates/z00z_runtime/aggregators/src/service.rs`.
  - `scenario_11` now writes durable consensus evidence, reopens the store from
    disk, resumes secondary takeover through
    `RecoveryBoundary::resume_from_store(...)`, and emits
    `consensus_store_report.json`.
- Release-guard drift was repaired inside the broad gate
  - `crates/z00z_wallets/src/redb_store/mod.rs` was normalized back to the
    grouped crate-private debug-export form required by
    `test_production_hardening`.
  - This was a release-hardening surface drift exposed by the mandatory broad
    `cargo test --release` rerun, not a second Phase 067 implementation lane.

## Validation

Commands green during the `067-11` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_store -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_recovery_restart -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`

Broad release-gate note:

- The final current-cycle `cargo test --release` rerun completed green on the
  current tree after the grouped crate-private wallet debug-export guard shape
  was restored in `crates/z00z_wallets/src/redb_store/mod.rs`.
- No additional out-of-slice Phase 067 release regression surfaced during the
  final `067-11` broad rerun.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-11-PLAN.md current_task="067-11-T1" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-11-PLAN.md current_task="067-11-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-11-PLAN.md current_task="Durable Consensus Evidence Store" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.

Equivalent workspace-first manual review was executed with the `/doublecheck`
three-layer posture against the same scope.

- Pass 1
  - Re-read the durable restart path in
    `crates/z00z_runtime/aggregators/src/consensus_store.rs`,
    `recovery.rs`, `service.rs`,
    `crates/z00z_runtime/aggregators/tests/test_consensus_store.rs`,
    `test_consensus_recovery_restart.rs`, and
    `crates/z00z_simulator/src/scenario_11/mod.rs`.
  - Result: clean for one canonical persistence seam. No second in-memory-only
    recovery path or detached validator-decision path remained under the
    reviewed scope.
- Pass 2
  - Re-ran anchored grep for the live seam markers
    `CONSENSUS_STORE_BACKEND`, `CONSENSUS_STORE_SCHEMA_VERSION`,
    `resume_from_store`, `persist_consensus_commit`,
    `persist_consensus_publication`, `persist_validator_decision`,
    `validator_decision_snapshot`, `consensus_store_report`, and
    `reloaded_from_store` across the touched runtime, simulator, and test
    surfaces.
  - Result: clean. The landed code and tests agree on one canonical durable
    store path and one restart resume path.
- Pass 3
  - Ran `git diff --check` across the touched code and the wallet hardening
    guard repair.
  - Result: clean.
- Pass 4
  - Re-read `067-11-SUMMARY.md`, `.planning/STATE.md`, `.planning/ROADMAP.md`,
    `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`, and
    `.planning/phases/067-Sharded-Concensus/067-verdict.md` after the final
    status sync.
  - Result: clean.

Passes 3 and 4 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-11` closes `VERDICT-LCS-02` by making restart evidence durable and
reloadable on the live local-conformance path, with exact subject or
certificate or publication or validator binding continuity and fail-closed
rejection on stale or partial or divergent persisted state.

`067-12` is now the next canonical execution lane.
