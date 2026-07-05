---
phase: 067
plan: 067-06
status: complete
completed_at: 2026-07-04
next_plan: 067-07
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-06-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-06 Summary: Join Removal And Rotation Simulation

## Outcome

`067-06` is complete.

`PHASE-5` now closes on one canonical lifecycle-transition proof path. The
live runtime and the independent `scenario_11` harness now prove that observer
admission, readiness promotion, planned primary rotation, emergency takeover,
removed-member rejection, mixed-generation rejection, divergent-root freeze,
partition or heal behavior, and rolling takeover continuity stay bound to the
same subject digest, lineage, term, and routing-generation rules as the
steady-state local quorum path.

The closeout also removes the release-gate drifts discovered during the broad
reruns. `z00z_core` and `z00z_storage` exposed stale live-guardrail authority
bindings, while `z00z_wallets` later surfaced both a crate-private
debug-export shape regression and a wallet-guide transport-anonymity
overclaim guard drift; all affected seams were repaired against existing live
authority sources without restoring missing files, widening the wallet debug
surface, or introducing a second authority path.

## Files Changed

- `.planning/phases/000/062-Gaps-Closing-2/GAPS.md`
- `.planning/phases/067-Sharded-Concensus/067-06-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-06-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-09-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_core/tests/test_live_guardrails.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_wallets/docs/WALLET-GUIDE.md`
- `crates/z00z_wallets/src/redb_store/mod.rs`

## Landed Changes

- Runtime lifecycle transition coverage
  - `test_hjmt_join` now proves a pending observer cannot vote before
    readiness, a ready observer becomes vote-eligible on the same lawful
    membership path, an old primary cannot keep the primary vote role after
    rotation, and a lawful rotated committee still commits through the same
    runtime seam.
- `scenario_11` lifecycle fault matrix
  - Added explicit lifecycle cases for `one_secondary_offline`,
    `one_secondary_stale`, `observer_not_ready_before_readiness`,
    `observer_ready_after_catchup`, `removed_member_vote`,
    `mixed_generation_certificate`, `same_term_divergent_root_freeze`,
    `partition_and_heal`, and `rolling_primary_takeover_continuity`.
  - The partition or heal lane now proves deferred minority behavior, lawful
    recovery application, and replay-ignore continuity instead of synthesizing
    a conflicting certificate.
- Canonical source and guardrail normalization
  - Phase 067 packet references now use `.planning/phases/090-New-Scenarios/90-TODO.md`
    as the canonical Scenario 11 planning source and treat future-only or
    target-design wording as live mandatory scope.
  - Explicit missing-file authority wording was removed in favor of legacy
    non-canonical aggregator-consensus drift wording.
  - `z00z_core` and `z00z_storage` live-guardrail tests now bind to existing
    canonical planning sources in the current worktree instead of deleted or
    superseded paths.
  - `z00z_wallets` keeps the `redb_store` debug export in the required grouped
    `pub(crate)` form so the internal-only debug surface remains fail-closed
    under the production-hardening guard.
  - `WALLET-GUIDE.md` restores the exact `Phase 062 does not claim live
    transport anonymity.` exclusion string so wallet privacy wording stays
    bounded and out of live OnionNet transport claims under
    `test_spec_terms_guard`.
- No parallel implementation layer
  - No new HJMT, crypto, utility, or alternate membership path was added.
  - The slice stays on existing `z00z_core`, `z00z_crypto`, `z00z_storage`,
    `z00z_utils`, `z00z_aggregators`, and `z00z_simulator` seams only.

## Validation

Commands green during the `067-06` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_join -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_route_rollout -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_spec_terms_guard -- --nocapture`
- `cargo test --release`

Broad release-gate note:

- Early current-cycle reruns exposed only release-gate drift outside the core
  membership logic: `z00z_core` still referenced the stale
  `090-New-Scenarios/066-TODO.md` path, `z00z_storage` still depended on a
  deleted Phase 080 source file plus a too-strict closeout-shape assertion for
  the live `GAPS.md` task authority, a later rerun caught a `z00z_wallets`
  crate-private debug-export guard regression, and the final broad rerun then
  surfaced wallet-guide transport-anonymity bounded-wording drift under
  `test_spec_terms_guard`.
- After rebinding those guards to existing canonical sources, restoring the
  grouped crate-private wallet debug-export form, and reinstating the exact
  Phase 062 transport-anonymity exclusion string in `WALLET-GUIDE.md`, the
  closeout returned to one live authority path for the affected planning,
  storage, and wallet seams.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-06-PLAN.md current_task="067-06-T1" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83738 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-06-PLAN.md current_task="067-06-T1"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-06-PLAN.md current_task="Join Removal And Rotation Simulation" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66679 > 38936`

Equivalent workspace-first manual review was executed against the same scope.

- Pass 1
  - Re-read the diff for `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`,
    `crates/z00z_simulator/src/scenario_11/mod.rs`, and
    `crates/z00z_simulator/tests/test_scenario_11.rs`.
  - Result: found one real repo-rule issue: newly added test identifiers
    exceeded the five-word limit. Renamed the new test functions to
    `pending_observer_blocks_voting`, `rotated_primary_rejects_old_role`,
    `scenario11_happy_path_consistent`, and `scenario11_fault_matrix_covers`.
- Pass 2
  - Ran packet-wide stale-authority grep over the Phase 067 packet and touched
    guardrail sources.
  - Result: found one real canonical-path drift: `067-09-PLAN.md` still named
    the missing `Agg-Concensus-Spec.md` file directly. Normalized it to legacy
    non-canonical aggregator-consensus drift wording.
- Pass 3
  - Re-ran stale-authority grep for `090-New-Scenarios/066-TODO.md`,
    `Agg-Concensus-Spec.md`, and the superseded long test names, then re-read
    the lifecycle anchors for `TakeoverSecondary`, `placement.secondaries`, and
    the new `scenario_11` fault IDs.
  - Result: clean.
- Pass 4
  - Re-read `067-06-SUMMARY.md`, `067-06-PLAN.md`, `067-COVERAGE.md`,
    `.planning/STATE.md`, and `.planning/ROADMAP.md` after the closeout-doc
    sync.
  - Result: clean.
- Pass 5
  - Re-ran stale-authority grep against the active Phase 067 plans, status
    docs, and touched code after the wallet guard sync.
  - Result: clean.
- Pass 6
  - Re-read `067-06-SUMMARY.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md`
    after the wallet hardening note sync.
  - Result: clean.
- Pass 7
  - Re-ran stale-authority and overclaim grep across the active Phase 067
    packet, status docs, lifecycle tests, and `crates/z00z_wallets/docs/WALLET-GUIDE.md`
    for stale plan refs, superseded test names, `067-06 active`, and live
    OnionNet transport-anonymity wording.
  - Result: clean.
- Pass 8
  - Re-read `067-06-SUMMARY.md`, `.planning/STATE.md`, `.planning/ROADMAP.md`,
    and `crates/z00z_wallets/docs/WALLET-GUIDE.md` after the final green broad
    `cargo test --release` rerun.
  - Result: clean.

Passes 7 and 8 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-06` closes `PHASE-5` by proving that membership transitions stay on the
same lawful runtime and simulator path as steady-state quorum: no unready
observer vote, no old-primary role reuse, no mixed-generation certificate, no
divergent-root takeover, and no partition minority certificate synthesis.

`067-07` is now the next canonical execution lane.
