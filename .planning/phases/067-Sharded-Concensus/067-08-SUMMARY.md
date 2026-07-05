---
phase: 067
plan: 067-08
status: complete
completed_at: 2026-07-05
next_plan: 067-09
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-08-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-08 Summary: Network And Signature Adapter

## Outcome

`067-08` is complete.

`PHASE-7` now closes on one live replay-verified vote path. `ShardVote` no
longer depends on a simulator-only fixed digest field, vote creation now flows
through a production-facing signer seam, in-memory transport delivery can no
longer create or count votes without replay verification, and conflicting or
payload-withholding paths now emit structured runtime-owned evidence instead of
silently succeeding or synthesizing votes.

The closeout keeps one canonical local conformance lane. The deterministic local
signer remains the only current implementation behind the signer trait, but the
runtime-owned `ReplayVerifiedVoteService` now proves where future operator-key
signers and future transports must attach without bypassing replay, membership,
or subject binding.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-08-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-08-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_crypto/src/domains.rs`
- `crates/z00z_runtime/aggregators/src/evidence.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/src/shard_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/src/shard_vote.rs`
- `crates/z00z_runtime/aggregators/src/signature.rs`
- `crates/z00z_runtime/aggregators/src/transport.rs`
- `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
- `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs`
- `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/src/scenario_11/report.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

## Landed Changes

- Production-facing vote signature seam
  - Added `VoteSigner`, `VoteSignatureVerifier`, `VoteSignature`, and
    `VoteSignatureScheme` under `z00z_aggregators`.
  - Replaced the embedded simulator-only signature field on `ShardVote` with a
    generic signature envelope and generic signature validation path.
  - Kept `DeterministicLocalVoteSigner` as the canonical local implementation so
    the current phase stays fully local and deterministic while future real
    signers attach at one runtime-owned seam.
- Replay-verified transport seam
  - Added `VoteTransportEnvelope`, `VoteTransport`, and
    `InMemoryVoteTransport`.
  - Added `ReplayVerifiedVoteService`, which refuses to create votes until
    `SecondaryReplayVerifier` accepts the claimed subject and which treats
    duplicate transport envelopes idempotently.
  - Payload-missing delivery now emits evidence instead of creating a synthetic
    vote.
- Structured safety evidence
  - Added `EquivocationEvidence`, `PayloadWithholdingEvidence`, `VoteEvidence`,
    and `VoteEvidenceTracker`.
  - Conflicting same-voter same-term same-membership votes now emit canonical
    equivocation evidence with both conflicting votes embedded.
  - Missing payload delivery now emits canonical payload-withholding evidence
    with subject and payload binding.
- Local conformance harness upgrade
  - `scenario_11` now routes its secondary-vote happy path and fault paths
    through `InMemoryVoteTransport` plus `ReplayVerifiedVoteService` instead of
    directly manufacturing replay-accepted secondary votes.
  - The report surface now records transport verdicts and signature schemes so
    the local artifacts explicitly distinguish delivered votes, replay rejects,
    evidence emission, and deferred/offline cases.
- No parallel authority path
  - No external transport crate, new consensus crate, alternate replay path, or
    parallel signature authority layer was added.
  - The slice stays on existing `z00z_aggregators`, `z00z_crypto`, and
    `z00z_simulator` seams only.

## Validation

Commands green during the `067-08` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_signature_adapter -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_adapter -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_equivocation_evidence -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`
- `cargo test --release`

Broad release-gate note:

- The first current-cycle `cargo test --release` rerun exposed one real
  closeout regression outside the targeted aggregator acceptance list:
  `crates/z00z_wallets/tests/test_production_hardening.rs`
  `test_debug_export_surface_is_internal_only` failed because
  `crates/z00z_wallets/src/redb_store/mod.rs` had drifted back to the
  single-item `pub(crate) use self::debug::debug_export_wallet;` form instead
  of the grouped crate-private debug-export form required by the release
  hardening contract.
- After rerunning
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`,
  restoring the grouped crate-private re-export, rerunning
  `cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`,
  and restarting the broad release gate, the final `cargo test --release`
  rerun completed green on the current tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-08-PLAN.md current_task="067-08-T1" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83738 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-08-PLAN.md current_task="067-08-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-08-PLAN.md current_task="Network And Signature Adapter" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66677 > 38936`

Equivalent workspace-first manual review was executed with the `/doublecheck`
three-layer posture against the same scope.

- Pass 1
  - Re-read `crates/z00z_runtime/aggregators/src/signature.rs`,
    `transport.rs`, `evidence.rs`, `service.rs`, `shard_vote.rs`,
    `shard_quorum_certificate.rs`, and the replayed vote paths in
    `crates/z00z_simulator/src/scenario_11/mod.rs`.
  - Result: clean for one canonical replay-verified vote creation path; no code
    path was found that lets transport mint or count a vote before replay
    verification.
- Pass 2
  - Re-ran anchored grep for stale pre-067-08 strings and for the live seam
    markers `ReplayVerifiedVoteService`, `VoteTransportEnvelope`,
    `transport_verdict`, and `signature_scheme` across the touched code and
    planning files.
  - Result: clean for the live path. Remaining `SecondaryReplayVerifier`
    references are expected owner-path references, not stale bypass paths.
- Pass 3
  - Ran `git diff --check` across the touched code and phase-closeout docs.
  - Result: clean.
- Pass 4
  - Re-ran anchored status grep across `067-COVERAGE.md`, `067-verdict.md`,
    `.planning/STATE.md`, and `.planning/ROADMAP.md` for stale `067-08` active
    markers and the old `7/19` progress markers.
  - Result: clean.
- Pass 5
  - Re-read `067-08-SUMMARY.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md`
    after the final status sync.
  - Result: clean.

Passes 4 and 5 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-08` closes `PHASE-7` by making the signature, transport, and evidence seam
live without weakening the replay-first local quorum contract: future signers
and future transports now have one runtime-owned entrypoint, and the current
local harness proves that missing payloads, duplicates, and equivocation do not
silently turn into votes.

`067-09` is now the next canonical execution lane.
