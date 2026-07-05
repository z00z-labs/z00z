---
phase: 067
plan: 067-09
status: complete
completed_at: 2026-07-05
next_plan: 067-10
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-09-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-09 Summary: Bft And Celestia Backend

## Outcome

`067-09` is complete.

`PHASE-8` now closes on one live local BFT/Celestia authority path. The
repository now proves exact `3f+1` committee sizing and `2f+1` quorum
thresholds behind the existing commit-subject seam, and the
Celestia-compatible local adapter now resolves the same
subject-or-certificate-or-theorem-bound artifact contract already enforced by
the validator path.

The closeout keeps the honesty boundary explicit. `067-09` lands simulated-full
local BFT math and Celestia-local artifact conformance, but it does not
overclaim real HotStuff installation, real Celestia finality, or runnable
multi-process devnet proof. Those remain later verdict-lane work.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-09-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-09-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `config/hjmt_runtime/sim_7a7s/manifest.json`
- `config/hjmt_runtime/sim_7a7s/planner/planner-config.yaml`
- `config/hjmt_runtime/sim_7a7s/aggregators/agg-*/aggregator-config.yaml`
- `crates/z00z_rollup_node/src/celestia_local.rs`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_runtime/aggregators/src/bft_committee.rs`
- `crates/z00z_runtime/aggregators/src/bft_engine.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`

## Landed Changes

- Exact BFT committee math on the live subject path
  - Added `BftThresholds`, `BftCommittee`, and `BftEngine` under
    `z00z_aggregators`.
  - The simulated backend now rejects sub-`3f+1` membership, requires exact
    `2f+1` quorum for certificate creation, and keeps the existing
    `CommitSubject` or `ShardQuorumCertificate` contract as the only commit
    authority path.
  - The release test surface now proves `7`, `10`, and `13` member committee
    thresholds and rejects the old `2-of-3` CFT profile as a false BFT claim.
- Canonical `SIM-7A7S` local BFT topology truth
  - Added the `config/hjmt_runtime/sim_7a7s/` runtime profile with seven
    aggregator identities, seven shard rows, and BFT-valid primary plus ready
    secondary committee inventory.
  - The manifest truth now records local-only scope explicitly: simulator-proven
    BFT math and artifact binding are live, while process/devnet realism
    remains a later proof lane rather than an implicit claim.
- Celestia-local artifact conformance
  - Added `CelestiaLocalAdapter`, which records namespace, blob commitment,
    payload digest, publication digest, subject digest, certificate digest,
    theorem digest, local height, anchor height, challenge window, and
    unanchored-height policy.
  - Publish or resolve now stays bound to the same live theorem or certificate
    contract already accepted on the local DA path.
  - Negative tests now reject namespace drift, commitment drift, missing
    payload, stale anchor, certificate drift, unanchored-height overflow, and
    validator-facing artifact drift.
- Release-test closure and no overclaim boundary
  - The BFT rule test now boxes `RejectRecord` through one release-safe error
    adapter so the negative-path proof compiles under `cargo test --release`.
  - The closeout keeps one canonical local claim level: no real HotStuff,
    operator-key signature, public network, or real Celestia finality claim was
    added to close `067-09`.

## Validation

Commands green during the `067-09` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_bft_committee_rules -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`

Broad release-gate note:

- The final current-cycle `cargo test --release` rerun completed green on the
  current tree after the `test_bft_committee_rules` release-test harness was
  repaired to map `RejectRecord` into a boxed standard error for the
  `Result<(), Box<dyn Error>>` test form.
- No additional out-of-slice Phase 067 release regression surfaced during the
  final `067-09` broad rerun.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-09-PLAN.md current_task="067-09-T1" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83738 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-09-PLAN.md current_task="067-09-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-09-PLAN.md current_task="Bft And Celestia Backend" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66676 > 38936`

Equivalent workspace-first manual review was executed with the `/doublecheck`
three-layer posture against the same scope.

- Pass 1
  - Re-read the core BFT and Celestia-local diff in
    `crates/z00z_runtime/aggregators/src/bft_committee.rs`,
    `bft_engine.rs`, `crates/z00z_rollup_node/src/celestia_local.rs`,
    `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`,
    `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`, and
    `config/hjmt_runtime/sim_7a7s/manifest.json`.
  - Result: clean for one canonical local BFT/Celestia path. No alternate
    commit authority or detached artifact return path was found.
- Pass 2
  - Re-ran anchored grep for the live seam markers `BftCommittee`,
    `BftEngine`, `CelestiaLocalAdapter`, `3f+1`, `2f+1`,
    `MissingPayload`, `UnanchoredHeightExceeded`, and `SIM-7A7S` across the
    touched code, config, and tests.
  - Result: clean for the live path. The landed code and tests agree on one
    BFT-valid local profile and one Celestia-local artifact contract.
- Pass 3
  - Re-ran the overclaim audit for `process_model`, `start_cmd`,
    `real Celestia finality`, `production signatures`, and related status
    wording across the touched code and planning artifacts.
  - Result: clean. The manifest still records process identities as config
    truth only, and runnable process/devnet proof remains explicitly deferred to
    `067-10` and later verdict lanes.
- Pass 4
  - Ran `git diff --check` across the touched code and phase-closeout docs.
  - Result: clean.
- Pass 5
  - Re-read `067-09-SUMMARY.md`, `.planning/STATE.md`, `.planning/ROADMAP.md`,
    `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`, and
    `.planning/phases/067-Sharded-Concensus/067-verdict.md` after the final
    status sync.
  - Result: clean.

Passes 4 and 5 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-09` closes `PHASE-8` by making BFT-valid committee math and
Celestia-local artifact binding executable on the live certificate-aware local
path, without overclaiming real HotStuff, real Celestia finality, or runnable
process/devnet behavior.

`067-10` is now the next canonical execution lane.
