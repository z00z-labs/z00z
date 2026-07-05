---
phase: 067
plan: 067-07
status: complete
completed_at: 2026-07-05
next_plan: 067-08
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-07-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-07 Summary: Validator And Theorem Binding

## Outcome

`067-07` is complete.

`PHASE-6` now closes on one live quorum-binding path. Local DA publication and
resolve preserve the exact `ordered_batch`, `CommitSubject`, and
`ShardQuorumCertificate` that the commit path produced, while downstream
validator acceptance now rejects missing, detached, stale, or mismatched quorum
artifacts whenever the certificate-aware gate is enabled.

The closeout removes the last primary-trust shortcut on the local publication
seam. Theorem digest, publication binding, certificate digest, membership
digest, route lineage, and ordered-batch digest now converge on one subject
verification path instead of allowing the validator to trust publication output
without rechecking the quorum artifact.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-07-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-07-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/tests/support/test_theorem_fixture.rs`
- `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`
- `crates/z00z_runtime/aggregators/src/commit_subject.rs`
- `crates/z00z_runtime/aggregators/src/shard_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
- `crates/z00z_runtime/validators/src/checkpoint.rs`
- `crates/z00z_runtime/validators/src/verdict.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`
- `crates/z00z_runtime/validators/tests/test_theorem_support.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_wallets/src/redb_store/mod.rs`

## Landed Changes

- Live quorum-binding DTO continuity
  - `PublicationRequest` now carries the live `ordered_batch`, `subject`, and
    `certificate`, while `PublishedBatch` now records the subject or
    certificate or theorem digests that the downstream validator later checks.
  - `ResolvedBatch` now carries the resolved `subject` and `certificate`
    instead of reconstructing a detached or synthetic batch boundary later.
- Canonical subject verification path
  - `CommitSubject::verify_binding` now rechecks route, route-table,
    membership, plan, payload, theorem, publication, and ordered-batch
    alignment against one subject.
  - `ShardQuorumCertificate::verify_subject` now proves vote-kind consistency,
    unique voters, deterministic local signatures, canonical aggregate digest,
    and exact `subject_digest` continuity.
- Validator fail-closed acceptance
  - `CheckpointFlow` now requires subject or certificate or theorem or
    publication alignment when the quorum-binding gate is enabled.
  - Missing subject, missing certificate, detached theorem digest, detached
    publication digest, stale membership digest, route drift, and exact
    certificate mismatch now reject through the validator seam.
- Local DA publish or resolve honesty
  - Local DA publication now records the live theorem digest plus the live
    publication binding and rejects request drift before persisting.
  - Local DA resolve now returns the original `ordered_batch`, `subject`, and
    `certificate` instead of synthesizing a placeholder ordered batch.
- Proof and regression coverage
  - Added a dedicated local DA quorum-binding test suite, expanded theorem and
    validator negative-path coverage, extended validator fixture support, and
    updated the independent `scenario_11` harness so the happy path preserves
    one commit subject through replay, certificate, theorem, DA, and validator.
- No parallel authority path
  - No new HJMT, crypto, utility, theorem, or validator bypass layer was
    added.
  - The slice stays on existing `z00z_aggregators`, `z00z_rollup_node`,
    `z00z_validators`, and `z00z_simulator` seams only.

## Validation

Commands green during the `067-07` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_quorum_binding -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_rollup_theorem_guard -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_publication_binding -- --nocapture`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture`
- `cargo test --release -p z00z_validators --test test_object_policy_verdicts --no-run`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

Broad release-gate note:

- The first current-cycle `cargo test --release` rerun exposed one real
  closeout regression outside the target acceptance list: the shared theorem
  fixture imported `z00z_validators::SettlementTheoremBundle`, which broke the
  `z00z_aggregators` release test `test_secondary_replay_verifier` because that
  fixture is included there too.
- The fixture was made self-contained by computing the theorem digest locally
  and by pinning the decoded transaction digest to `[u8; 32]`, after which the
  focused `test_secondary_replay_verifier` release gate turned green.
- The next broad rerun then exposed one more current-tree release drift outside
  the plan slice: `crates/z00z_wallets/tests/test_production_hardening.rs`
  requires the grouped crate-private `redb_store` debug-export form
  `pub(crate) use self::debug::{...}`, but
  `crates/z00z_wallets/src/redb_store/mod.rs` had regressed to the single-item
  form.
- After rerunning `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`,
  restoring the grouped crate-private re-export, rerunning
  `cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`,
  and restarting the broad release gate once more, the final
  `cargo test --release` rerun completed green on the current tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-07-PLAN.md current_task="067-07-T1" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83738 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-07-PLAN.md current_task="067-07-T1"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-07-PLAN.md current_task="Validator And Theorem Binding" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66677 > 38936`

Equivalent workspace-first manual review was executed against the same scope.

- Pass 1
  - Re-read the core quorum-binding diff in `crates/z00z_rollup_node/src/da.rs`,
    `crates/z00z_runtime/aggregators/src/commit_subject.rs`,
    `crates/z00z_runtime/aggregators/src/shard_quorum_certificate.rs`,
    `crates/z00z_runtime/aggregators/src/types.rs`,
    `crates/z00z_runtime/validators/src/checkpoint.rs`,
    `crates/z00z_runtime/validators/src/verdict.rs`, and
    `crates/z00z_simulator/src/scenario_11/mod.rs`.
  - Result: found one real issue: the shared theorem fixture pulled a
    validator-only type into an aggregator-owned release test include path.
    Replaced that dependency with a local theorem-digest helper and reran the
    focused release gate.
- Pass 2
  - Re-ran packet-wide grep for `PublicationRequest {`, `PublishedBatch {`,
    `ResolvedBatch::new(`, `quorum_binding_enabled`, `verify_request_quorum_binding`,
    `verify_binding(`, and `verify_subject(` across the touched crates.
  - Result: clean for the live path. Remaining `None` digest fields appear only
    in tests that intentionally keep the gate disabled.
- Pass 3
  - Ran `git diff --check` across the touched Phase 067 docs and code,
    including the follow-up wallet hardening fix.
  - Result: clean.
- Pass 4
  - Re-ran anchored stale-status grep across `067-COVERAGE.md`,
    `067-verdict.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md` for the
    old `067-07` active-lane markers and the old `6/19` progress markers.
  - Result: clean.
- Pass 5
  - Re-read `067-07-SUMMARY.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md`
    after the wallet hardening fix, the final green `bootstrap_tests.sh`
    rerun, and the final green broad `cargo test --release` rerun.
  - Result: clean.

Passes 4 and 5 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-07` closes `PHASE-6` by making downstream acceptance certificate-aware on
the live local quorum path: the same subject now binds replay, certificate,
publication, theorem, DA resolve, and validator acceptance.

`067-08` is now the next canonical execution lane.
