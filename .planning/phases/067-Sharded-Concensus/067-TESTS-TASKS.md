---
phase: 067
artifact: tests-tasks
status: planned
source: 067-TEST-SPEC.md
updated: 2026-07-05
---

# Phase 067 Test Tasks

This document converts the Phase 067 planning packet into an implementation-ready test worklist. It is intentionally phase-local and does not claim that the underlying runtime behavior already exists.

## 📌 Purpose

📌 This document translates `067-TEST-SPEC.md` into one concrete implementation order for test work.

📌 It preserves the `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-01-PLAN.md` through `067-19-PLAN.md`, and `067-VERDICT-ITEM-AUDIT.md` authority chain without adding a parallel consensus, storage, transport, validator, report, or simulator layer.

## 📚 Scope Inputs

- `.planning/phases/067-Sharded-Concensus/067-TEST-SPEC.md`
- `.planning/phases/067-Sharded-Concensus/067-TODO.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `.planning/phases/067-Sharded-Concensus/067-CONTEXT.md`
- `.planning/phases/067-Sharded-Concensus/067-VERDICT-ITEM-AUDIT.md`
- `.planning/phases/067-Sharded-Concensus/067-01-PLAN.md` through `.planning/phases/067-Sharded-Concensus/067-19-PLAN.md`
- `.planning/phases/090-New-Scenarios/90-TODO.md` section `15`
- `crates/z00z_runtime/aggregators/README.md`

## 🧭 Execution Strategy

- Start with packet integrity because every later test must preserve the one-to-one mapping from `PHASE-0` through `PHASE-8` and `VERDICT-LCS-01` through `VERDICT-LCS-10`.
- Land primitive and boundary tests before end-to-end assertions: terminology, subject encoding, replay, certificate validation, DA/validator binding, and transport gates must exist before `scenario_11` can be a meaningful proof.
- Implement verdict expansion tests in dependency order: runnable process surface, durable store, planner authority, local process harness, fault scheduler, BFT/HotStuff-local backend, Celestia-local artifact, evidence registry, claim audit, final conformance.
- Treat each task as blocked until it has positive cases, negative cases, exact commands, expected artifacts, and anti-placeholder evidence.
- Do not create runtime tests for APIs that do not yet exist; create them in the owning slice when the implementation lands, using the target homes below.

## 🌊 Task Waves

| Wave | Tasks | Why this order is required | Completion gate |
| --- | --- | --- | --- |
| `T0 source lock` | `TT-00` | prevents coverage drift before test implementation starts | packet mapping and source refs verified |
| `T1 local primitives` | `TT-01` through `TT-04` | terminology, subject, replay, and QC primitives are prerequisites for every later integration test | targeted aggregator and rollup-node tests pass |
| `T2 base scenario and lifecycle` | `TT-05` through `TT-06` | `scenario_11` and lifecycle continuity need primitive correctness first | simulator evidence proves package-to-validator and membership transitions |
| `T3 publication and transport` | `TT-07` through `TT-09` | DA/validator, signatures, transport, BFT, and Celestia-local tests depend on subject/QC binding | validator, DA, adapter, BFT, and Celestia-local commands pass |
| `T4 verdict readiness` | `TT-10` through `TT-12` | process, store, and planner tests must land before devnet and fault simulations can be honest | CLI, metadata, durable recovery, and planner drift tests pass |
| `T5 process and fault realism` | `TT-13` through `TT-16` | devnet, network faults, HotStuff-local, and Celestia-local require executable runtime seams | process smoke, fault matrix, backend, and artifact tests pass |
| `T6 evidence and closure` | `TT-17` through `TT-19` | structured evidence and claim audit are the final proof layer over all prior slices | final conformance bundle has no blocker, overclaim, or placeholder closure |

## 🎯 Execution Model

- Implement tests slice by slice in `TS-01` through `TS-19` order.
- This document is a planning artifact for `/gsd-add-tests 067`; do not create runtime tests until the corresponding Phase 067 implementation slice exists.
- If a target test file already exists, extend it in place.
- If a target test file is listed in the plan packet but does not exist yet, create it at the proposed path instead of inventing a parallel home.
- Keep one runtime authority. Do not duplicate route, placement, certificate, publication, theorem, or validator logic inside tests.
- `scenario_11` is the only new end-to-end harness for this phase. Do not fold this work into `scenario_1`.

## 📍 Current Workspace Reality

### ✅ Existing anchors that should be extended

- `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`
- `crates/z00z_runtime/aggregators/tests/test_commit_subject.rs`
- `crates/z00z_runtime/aggregators/tests/test_shard_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs`
- `crates/z00z_runtime/aggregators/tests/test_local_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

### 🆕 Verdict expansion test homes

These homes are required by `067-verdict.md`. Some are already present in the current workspace; presence alone is not closure until the cases below pass with real project primitives.

| File | Current status | Required slice |
| --- | --- | --- |
| `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs` | present in current workspace | `TS-07` |
| `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs` | present in current workspace | `TS-08` |
| `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs` | present in current workspace | `TS-08` |
| `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs` | present in current workspace | `TS-08` |
| `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs` | missing | `TS-09`, `TS-15` |
| `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs` | missing | `TS-09`, `TS-16` |
| `crates/z00z_runtime/aggregators/tests/test_consensus_store.rs` | missing | `TS-11` |
| `crates/z00z_runtime/aggregators/tests/test_consensus_recovery_restart.rs` | missing | `TS-11` |
| `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs` | missing | `TS-12` |
| `crates/z00z_rollup_node/tests/test_hjmt_process_devnet.rs` | missing | `TS-13` |
| `crates/z00z_runtime/aggregators/tests/test_transport_fault_matrix.rs` | missing | `TS-14` |
| `crates/z00z_runtime/aggregators/tests/test_hotstuff_local_backend.rs` | missing | `TS-15` |
| `crates/z00z_runtime/aggregators/tests/test_structured_evidence_registry.rs` | missing | `TS-17` |
| `scripts/audit/audit_067_claims.py` | missing | `TS-18` |

## 🔒 Shared Gates

- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` before each verification pass.
- Run `cargo test --release` whenever a slice touches broad Rust or shared test surfaces.
- Keep all repo artifacts in English.
- Use existing project abstractions and real crypto primitives.
- Reject any implementation that closes on placeholder digests, fixture-only byte comparisons, or report strings without live state evidence.
- Treat `067-verdict.md` and `067-VERDICT-ITEM-AUDIT.md` as coverage gates: no task is complete while a verdict bullet lacks a test, artifact, result, or evidence row.

## 🧷 Verdict Gate Implementation Index

This index binds the strong gates from `067-verdict.md` to concrete test tasks. A task is not complete if the listed gate has only prose, a crate import, a config field, a TODO, or an unexercised scaffold.

| Verdict gate | Primary task | Supporting tasks | Required closeout artifact |
| --- | --- | --- | --- |
| Gate 1 Glossary Traceability | `TT-18` | `TT-19` | `067-GLOSSARY-CLAIMS.md`, `067-CLAIM-AUDIT.md`, and `scripts/audit/audit_067_claims.py` reject missing, duplicate, or prose-only rows |
| Gate 2 One Real End-To-End Path | `TT-19` | `TT-05`, `TT-07`, `TT-11`, `TT-13` through `TT-18` | final `scenario_11` evidence proves one package and one subject digest from ingress through validator verdict |
| Gate 3 Validator Requires QC | `TT-07` | `TT-16`, `TT-19` | validator/publication tests reject missing, detached, stale, or mismatched QC bindings |
| Gate 4 Celestia-Local Is Artifact-Complete | `TT-16` | `TT-09`, `TT-19` | Celestia-local artifact schema includes namespace, blob bytes or digest, commitment, height, inclusion ref, retention, unanchored/degraded state, resolve/retrieve/verify |
| Gate 5 Celestia Negative Tests | `TT-16` | `TT-19` | wrong namespace, wrong commitment, missing payload, stale anchor, QC mismatch, unanchored limit, and validator-rejected blob all fail closed |
| Gate 6 Network Simulation Is Not Vote Injection | `TT-14` | `TT-08`, `TT-17`, `TT-19` | fault matrix shows no delivered message counts as a vote without replay and signature proof |
| Gate 7 BFT Claims Need BFT Math | `TT-15` | `TT-09`, `TT-19` | BFT committee tests enforce `n >= 3f + 1`, quorum `>= 2f + 1`, and reject CFT-as-BFT |
| Gate 8 HotStuff-Like Backend Behind The Seam | `TT-15` | `TT-19` | HotStuff-local tests bind view, leader, proposal, backend QC, subject, replay, and validator gate |
| Gate 9 Production Signature Seam | `TT-08` | `TT-14`, `TT-18`, `TT-19` | signer/verifier tests reject wrong signer, subject, membership, replayed signature, and emit equivocation evidence |
| Gate 10 Multi-Process Or Devnet Harness | `TT-13` | `TT-10`, `TT-11`, `TT-12`, `TT-19` | local process/devnet smoke emits distinct identity, port, data-dir, log, restart, kill, and partition evidence |
| Gate 11 Planner HA Or Claim Removal | `TT-12` | `TT-18`, `TT-19` | planner authority tests prove deterministic replicated recomputation or claim audit removes live HA wording |
| Gate 12 Crash Recovery | `TT-11` | `TT-13`, `TT-19` | consensus-store/recovery tests reload exact votes, QC, anchors, and recovery cursors or fail closed |
| Gate 13 Membership Reconfiguration | `TT-06` | `TT-12`, `TT-19` | observer, unready, removed, mixed-generation, and rotated members obey membership digest and lineage gates |
| Gate 14 Structured Evidence Artifacts | `TT-17` | `TT-08`, `TT-14`, `TT-16`, `TT-19` | evidence registry emits structured rows for equivocation, payload withholding, missing blob, wrong root, wrong route, stale member, and split brain |
| Gate 15 Report Honesty | `TT-18` | `TT-19` | final report and `report_honesty.json` classify each sensitive term without unqualified BFT, Celestia, devnet, HotStuff, production signature, slashing, finality, or planner HA claims |
| Hard Blockers | `TT-19` | all predecessor tasks | final conformance fails if QC is optional, Celestia-local is detached, network can inject votes, BFT is claimed on `2-of-3`, process model is manifest-only, planner HA is untested, or glossary terms are prose-only |

## 🧪 Ordered Tasks

### ✅ TT-00 Packet Integrity And Source Lock

- Goal:
  - confirm that the engineer is implementing from the locked packet only.
- Read first:
  - `067-TODO.md`
  - `067-verdict.md`
  - `067-CONTEXT.md`
  - `067-VERDICT-ITEM-AUDIT.md`
  - `067-TEST-SPEC.md`
  - `067-01-PLAN.md` through `067-19-PLAN.md`
  - `.planning/phases/090-New-Scenarios/90-TODO.md` section `15`
  - `crates/z00z_runtime/aggregators/README.md`
- Required checks:
  - preserve the exact `PHASE-0` through `PHASE-8` mapping and the
    `VERDICT-LCS-01` through `VERDICT-LCS-10` expansion mapping;
  - confirm every row in `067-VERDICT-ITEM-AUDIT.md` maps to one `TS-01` through `TS-19` test slice;
  - keep `scenario_11` independent;
  - do not add a second authority document.
- Completion signal:
  - all later tasks reference only packet-owned anchors and current code owners.

### ✅ TT-01 TS-01 Terminology And Boundary Cleanup

- Target files:
  - `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
  - existing quorum freeze test seam in `z00z_aggregators`
- Required cases:

| Case id | Target | What it must prove | Pass condition |
| --- | --- | --- | --- |
| `067-T01-01` | topology | `secondary` config fields load successfully | the topology graph is identical to the pre-rename ownership model |
| `067-T01-01a` | generated homes | generated temp homes derived from `sim_5a7s` also load with `secondary` fields only | generated runtime homes load without alias keys |
| `067-T01-02` | preflight | duplicate secondary ids reject | load fails with a deterministic error |
| `067-T01-03` | preflight | unknown secondary ids reject | load fails before runtime start |
| `067-T01-04` | preflight | primary id cannot appear inside the secondary set | load fails deterministically |
| `067-T01-05` | preflight | stale `standby` keys reject | parser fails; no compatibility alias is accepted |
| `067-T01-06` | consensus parity | split-brain freeze behavior is unchanged by the rename | the same conflict still freezes or rejects |
| `067-T01-07` | grep audit | no active `standby` naming survives | grep returns zero active hits |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast test_quorum_freezes_term_roots -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_preflight -- --nocapture`
  - `cargo test --release`
  - `rg -n "standby|TakeoverStandby|standby_ids" crates/z00z_runtime crates/z00z_rollup_node crates/z00z_simulator config/hjmt_runtime/sim_5a7s --glob '!**/*.md'`
- Anti-placeholder reminders:
  - a doc-only rename does not satisfy this slice;
  - a comment-only CFT/BFT wording change does not satisfy this slice;
  - dual alias fields such as `standby_ids` plus `secondary_ids` are forbidden.

### ✅ TT-02 TS-02 Commit Subject And Certificate Types

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_commit_subject.rs`
  - `crates/z00z_runtime/aggregators/tests/test_shard_quorum_certificate.rs`
- Required positive cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T02-01` | same live fixture yields the same subject digest every time | repeated encodes match byte-for-byte |
| `067-T02-02` | valid active vote set yields one certificate | one certificate is built and validates |
| `067-T02-02a` | active placement restriction | the certificate builder accepts only the active placement members for one shard and one generation | any off-shard, inactive, or stale-generation member causes rejection |

- Required mutation cases:

| Case id | Mutation | Pass condition |
| --- | --- | --- |
| `067-T02-03` | route digest drift | subject digest changes or validation rejects |
| `067-T02-04` | generation drift | subject digest changes or validation rejects |
| `067-T02-05` | root drift | subject digest changes or validation rejects |
| `067-T02-06` | lineage drift | subject digest changes or validation rejects |
| `067-T02-07` | proof version drift | subject digest changes or validation rejects |
| `067-T02-08` | policy generation drift | subject digest changes or validation rejects |
| `067-T02-09` | wrong voter role | vote or certificate rejects |
| `067-T02-10` | duplicate voter | certificate rejects |
| `067-T02-11` | inactive voter | certificate rejects |
| `067-T02-12` | mixed membership digest | certificate rejects |
| `067-T02-13` | mixed term | certificate rejects |
| `067-T02-14` | below quorum | certificate rejects |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_commit_subject -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_shard_quorum_certificate -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast test_quorum_freezes_term_roots -- --nocapture`
- Anti-placeholder reminders:
  - constant, debug-string, JSON-order, fixture-name, or hard-coded expected digests are invalid;
  - a certificate that wraps only voter ids without canonical vote material is invalid.

### ✅ TT-03 TS-03 Secondary Replay Verifier

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs`
  - `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
  - existing journal coverage in `test_hjmt_dist_journal`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T03-01` | exact primary subject is replayed and accepted | verifier returns an explicit accept result |
| `067-T03-02` | route drift rejects before vote creation | reject code is deterministic |
| `067-T03-03` | planner digest drift rejects | reject code is deterministic |
| `067-T03-04` | root drift rejects | reject code is deterministic |
| `067-T03-05` | lineage drift rejects | reject code is deterministic |
| `067-T03-06` | proof-version drift rejects | reject code is deterministic |
| `067-T03-07` | policy-generation drift rejects | reject code is deterministic |
| `067-T03-08` | publication-binding drift rejects | reject code is deterministic |
| `067-T03-09` | theorem-digest drift rejects | reject code is deterministic |
| `067-T03-10` | stale secondary state rejects | no vote is created |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_dist_journal -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
- Anti-placeholder reminders:
  - fixture-byte comparison without recomputing the subject does not satisfy this slice;
  - vote creation from hard-coded expected digests does not satisfy this slice.

### ✅ TT-04 TS-04 Local Quorum Certificate Integration

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
  - `crates/z00z_runtime/aggregators/tests/test_local_quorum_certificate.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T04-01` | honest local quorum returns one certificate-bound decision | certificate exists and commit result matches the honest path |
| `067-T04-02` | same-term conflicting subjects cannot both commit | freeze or reject occurs before dual commit |
| `067-T04-03` | duplicate voter is rejected | certificate formation fails |
| `067-T04-04` | removed voter is rejected | certificate formation fails |
| `067-T04-05` | joined-but-not-ready secondary is rejected | certificate formation fails |
| `067-T04-06` | mixed membership digest is rejected | certificate formation fails |
| `067-T04-07` | mixed term is rejected | certificate formation fails |
| `067-T04-08` | certificate path preserves parity with legacy honest decision | same final commit decision is observed |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_consensus -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_local_quorum_certificate -- --nocapture`
- Anti-placeholder reminders:
  - wrapping the old majority result in a synthetic certificate does not satisfy this slice;
  - leaving `ConsensusCommit` as the only proof object does not satisfy this slice.

### ✅ TT-05 TS-05 Scenario 11 Base End-To-End Harness

- Target files:
  - `crates/z00z_simulator/src/scenario_11/mod.rs`
  - `crates/z00z_simulator/src/scenario_11/report.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required end-to-end cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T05-01` | one-shard happy path | one valid package reaches one accepted validator verdict with one `2-of-3` certificate |
| `067-T05-02` | dual-primary owner path | one owner may control two shards without merging their quorums |
| `067-T05-03` | all-shard sweep | all seven shards emit owner, route, membership, and verdict evidence |
| `067-T05-04` | one secondary offline | the shard either forms a real honest quorum or fails closed |
| `067-T05-05` | primary crash before quorum | no certificate and no DA publication occur |
| `067-T05-06` | primary crash after quorum before DA | the exact same certificate is resumed into publication |
| `067-T05-07` | stale secondary | replay rejects before vote creation |
| `067-T05-08` | wrong route or planner digest | scenario rejects with deterministic evidence |
| `067-T05-09` | wrong dispatch owner or shard-owner mismatch | scenario rejects with deterministic evidence |
| `067-T05-10` | report honesty | output explicitly rejects unsupported BFT or Celestia claims |

- Required artifact assertions:
  - `package_ingress_report.json` and `route_plan_report.json` carry the same package identity.
  - `commit_subject.json`, `secondary_replay_votes.json`, `quorum_certificate.json`, `local_da_binding.json`, and `validator_verdict_report.json` carry one subject digest.
  - `fault_matrix.json` records every failure case with expected and observed status.
  - `report_honesty.json` marks unsupported claims as forbidden.
- Commands:
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_dispatch -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_shard_routing -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
  - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- Anti-placeholder reminders:
  - extending `scenario_1` with extra stage fields does not satisfy this slice;
  - emitting report files without live vote, certificate, DA, and validator evidence does not satisfy this slice;
  - any global-five-aggregator quorum counting is forbidden.

### ✅ TT-06 TS-06 Join Removal And Rotation Simulation

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
  - `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T06-01` | observer catch-up to ready secondary | observer cannot vote before readiness, then can vote after readiness |
| `067-T06-02` | planned rotation at checkpoint or generation boundary | old primary stops committing after activation |
| `067-T06-02a` | rolling secondary replacement continuity | once the replacement is ready, the next shard commit uses the new committee without route drift or quorum split |
| `067-T06-03` | removed member rejection | removed member cannot vote or count toward quorum |
| `067-T06-04` | emergency takeover with matching lineage and generation | takeover succeeds only with exact required lineage and generation |
| `067-T06-04a` | rolling primary takeover continuity | takeover on one shard allows the next lawful commit there while unrelated shards continue producing certificates |
| `067-T06-05` | stale-lineage takeover reject | fail-closed before vote counting |
| `067-T06-06` | stale-generation takeover reject | fail-closed before vote counting |
| `067-T06-06a` | stale route-generation takeover reject | fail-closed before vote counting |
| `067-T06-07` | divergent-root takeover reject | fail-closed before vote counting |
| `067-T06-08` | mixed-generation certificate reject | certificate validation fails |
| `067-T06-08a` | mixed-lineage vote-set reject | certificate validation fails before commit |
| `067-T06-09` | exact publication resume after crash | only the exact prior certificate/publication state resumes |
| `067-T06-10` | partition and heal | no conflicting certificate is synthesized during minority isolation |
| `067-T06-11` | offline-member minority | no synthetic quorum appears |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_join -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_route_rollout -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- Anti-placeholder reminders:
  - toggling role names without readiness or lineage enforcement does not satisfy this slice;
  - a failover path that ignores route generation does not satisfy this slice;
  - simulator-only lifecycle assertions without runtime placement or recovery enforcement do not satisfy this slice.

### ✅ TT-07 TS-07 Validator And Theorem Binding

- Target files:
  - `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
  - `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs`
  - `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`
  - `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
  - `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T07-01` | DA publish stores certificate binding | publication record contains a non-placeholder digest or reference |
| `067-T07-02` | DA resolve preserves certificate binding | resolved batch still refers to the same binding |
| `067-T07-03` | validator happy path | validator accepts only when certificate, theorem, publication, and ordered batch share one subject |
| `067-T07-04` | missing certificate reject | validator or resolve path rejects |
| `067-T07-05` | mismatched certificate reject | validator or resolve path rejects |
| `067-T07-06` | detached publication reject | theorem or validator path rejects |
| `067-T07-07` | stale certificate from inactive or mixed membership reject | validator path rejects |
| `067-T07-08` | local CFT proof harness | exhaustive or proof-based test shows agreement does not depend on trusting the primary |

- Commands:
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_quorum_binding -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_rollup_theorem_guard -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_publication_binding -- --nocapture`
  - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- Anti-placeholder reminders:
  - a constant or zero certificate digest in DA records does not satisfy this slice;
  - validator acceptance that ignores the certificate binding does not satisfy this slice.

### ✅ TT-08 TS-08 Network And Signature Adapter

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs`
  - `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs`
  - `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T08-01` | correct signer and digest validation | valid signed vote is accepted only after replay |
| `067-T08-02` | wrong signer reject | signature verification fails deterministically |
| `067-T08-03` | wrong membership digest reject | signature or vote validation fails |
| `067-T08-04` | wrong subject digest reject | signature or vote validation fails |
| `067-T08-05` | transport cannot bypass replay | transport-delivered vote without replay rejects |
| `067-T08-06` | duplicate message idempotency | duplicate delivery does not double count |
| `067-T08-07` | equivocation evidence | conflicting same-voter votes emit deterministic evidence containing both vote materials |
| `067-T08-08` | payload withholding | evidence or degraded state is emitted; no silent success |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_signature_adapter -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_adapter -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_equivocation_evidence -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- Anti-placeholder reminders:
  - a trait stub with no exercising tests does not satisfy this slice;
  - missing-payload paths may not create synthetic votes.

### ✅ TT-09 TS-09 BFT And Celestia Backend

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`
  - `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
  - `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T09-01` | `3f+1` committee membership rule | valid BFT mode only starts with `3f+1` members |
| `067-T09-02` | below-`3f+1` reject | BFT mode rejects deterministically |
| `067-T09-03` | `2f+1` quorum rule | valid BFT certificate requires `2f+1` votes |
| `067-T09-04` | below-`2f+1` reject | BFT certificate validation fails |
| `067-T09-05` | Celestia-local happy path | blob resolution returns the same artifact contract as local DA |
| `067-T09-06` | wrong blob commitment reject | resolution fails deterministically |
| `067-T09-07` | wrong namespace reject | resolution fails deterministically |
| `067-T09-08` | missing payload during challenge window | reject or degraded mode is recorded |
| `067-T09-09` | unanchored height or settlement delay | degraded mode is recorded instead of overclaiming finality |
| `067-T09-10` | detached certificate or blob binding reject | validator path rejects without proposer trust |
| `067-T09-11` | commit-certificate verification failure or state-root mismatch | validator or backend verification rejects deterministically |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_bft_committee_rules -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
  - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- Anti-placeholder reminders:
  - renaming a `2-of-3` local quorum as BFT does not satisfy this slice;
  - feature flags or adapter stubs without `3f+1` and `2f+1` proof do not satisfy this slice;
  - a Celestia-local resolver that returns constant or detached artifacts does not satisfy this slice.

### ✅ TT-10 TS-10 Dependency And Runnable Aggregator Readiness

- Maps to `TS-10`, `VERDICT-LCS-01`, and `067-10`.
- Target files:
  - `Cargo.toml`
  - `Cargo.lock`
  - `crates/z00z_runtime/aggregators/Cargo.toml`
  - `crates/z00z_rollup_node/Cargo.toml`
  - `crates/z00z_simulator/Cargo.toml`
  - `crates/z00z_rollup_node/src/main.rs`
  - `crates/z00z_rollup_node/src/config.rs`
  - `crates/z00z_rollup_node/src/runtime.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T10-01` | dependency graph is valid | `cargo metadata --format-version 1` succeeds |
| `067-T10-02` | direct dependencies are owned by the crate that uses them | audit output names the owner and exercised API for each direct addition |
| `067-T10-03` | rollup-node command is real | `cargo run --release -p z00z_rollup_node -- --help` succeeds and prints the required config flags |
| `067-T10-04` | `sim_5a7s` process commands are executable | manifest `start_cmd` and `restart_cmd` resolve to the real cargo target and parse required args |
| `067-T10-05` | external backend crate claims are honest | uninstalled or unexercised backends are recorded as non-claims |
| `067-T10-06` | missing command surface fails closed | missing target or missing `--aggregator-config`, `--planner-config`, or `--storage-config` rejects |

- Commands:
  - `cargo metadata --format-version 1`
  - `cargo run --release -p z00z_rollup_node -- --help`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`

- Evidence assertions:
  - dependency audit classifies `redb`, `object_store`, `ed25519-dalek`, `bytes`, `borsh` or project-owned canonical binary codec, `tracing`, `tracing-subscriber`, `metrics`, `prometheus`, `proptest`, `hotstuff_rs`, `libp2p`, `celestia-client`, `celestia-rpc`, `celestia-types`, `reed-solomon-erasure`, `reed-solomon-simd`, and `openraft`;
  - dependency audit maps each installed or candidate dependency to the owning plan, crate owner, exercised API, positive test, negative test, and non-claim rule;
  - final report cannot name an uninstalled external backend as live.

- Anti-placeholder reminders:
  - adding a crate without an exercised API does not satisfy this task;
  - manifest command strings without a runnable binary do not satisfy this task.

### ✅ TT-11 TS-11 Durable Consensus Evidence Store

- Maps to `TS-11`, `VERDICT-LCS-02`, and `067-11`.
- Target files:
  - `crates/z00z_runtime/aggregators/src/consensus_store.rs`
  - `crates/z00z_runtime/aggregators/src/lib.rs`
  - `crates/z00z_runtime/aggregators/src/recovery.rs`
  - `crates/z00z_runtime/aggregators/src/service.rs`
  - `crates/z00z_runtime/aggregators/tests/test_consensus_store.rs`
  - `crates/z00z_runtime/aggregators/tests/test_consensus_recovery_restart.rs`
  - `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T11-01` | votes, QCs, anchors, indexes, and cursors persist | fresh runtime reads the same schema version and digests |
| `067-T11-02` | post-quorum pre-DA resume | exact stored QC enters publication without regenerated votes |
| `067-T11-03` | post-DA recovery | recovered DA/Celestia-local anchor still binds to the same QC digest |
| `067-T11-04` | partial or corrupt store rejects | no vote counting, DA resolve, or validator acceptance occurs |
| `067-T11-05` | stale membership or route generation rejects | recovery fails before consensus state advances |
| `067-T11-06` | divergent root or lineage rejects | recovery fails closed with structured evidence |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_store -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_recovery_restart -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`

- Evidence assertions:
  - store records include schema version, subject digest, vote digests, QC digest, anchor digest, membership generation, lineage, and recovery cursor;
  - restart tests instantiate a fresh runtime or process boundary.

- Anti-placeholder reminders:
  - in-memory maps, logs, or fixture regeneration after restart do not satisfy this task.

### ✅ TT-12 TS-12 Planner Authority And Failover Claim Boundary

- Maps to `TS-12`, `VERDICT-LCS-03`, and `067-12`.
- Target files:
  - `crates/z00z_runtime/aggregators/src/batch_planner.rs`
  - `crates/z00z_runtime/aggregators/src/dist_dispatch.rs`
  - `crates/z00z_runtime/aggregators/src/service.rs`
  - `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
  - `.planning/phases/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T12-01` | identical config recomputes identical plan | all aggregators produce same digest |
| `067-T12-02` | planner config drift rejects | no dispatch or vote creation |
| `067-T12-03` | stale route digest rejects | fail-closed before consensus |
| `067-T12-04` | mixed planner generation rejects | fail-closed before vote, dispatch, or validator acceptance |
| `067-T12-05` | secondary recomputes instead of copying primary plan bytes | replay proof cites local planner output |
| `067-T12-06` | planner HA overclaim rejects | report guard fails bare HA claim unless a real HA service exists |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_planner_authority -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_planner -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_dispatch -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`

- Evidence assertions:
  - scenario output records planner config digest, route-table digest, placement generation, and claim level `live-claim-removed` for planner HA unless implemented.

- Anti-placeholder reminders:
  - docs-only HA removal or a planner service stub without failover tests does not satisfy this task.

### ✅ TT-13 TS-13 Multi Process Devnet Harness

- Maps to `TS-13`, `VERDICT-LCS-04`, and `067-13`.
- Target files:
  - `scripts/hjmt_local_devnet.sh`
  - `docker/compose.hjmt-local.yaml`
  - `crates/z00z_rollup_node/src/main.rs`
  - `crates/z00z_rollup_node/src/runtime.rs`
  - `crates/z00z_rollup_node/src/status.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_process_devnet.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T13-01` | five local identities start | distinct ports, data dirs, and logs |
| `067-T13-02` | primary killed before quorum | no QC or DA publication |
| `067-T13-03` | primary killed after quorum before DA | exact stored QC resumes |
| `067-T13-04` | minority partition cannot commit | no synthetic quorum |
| `067-T13-05` | stale process data dir rejects | restart fails before vote counting or validator accept |
| `067-T13-06` | duplicate port rejects before start | harness reports deterministic config error |

- Commands:
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process_devnet -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process -- --nocapture`
  - `bash scripts/hjmt_local_devnet.sh --profile sim_5a7s --smoke --timeout 30`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`

- Evidence assertions:
  - per-process evidence includes process id, aggregator id, port, data dir, log dir, restart count, partition state, QC digest, and DA/Celestia-local anchor state.

- Anti-placeholder reminders:
  - Docker or manifest files without a runnable smoke script do not satisfy this task.

### ✅ TT-14 TS-14 Network Fault Matrix And Transport Conformance

- Maps to `TS-14`, `VERDICT-LCS-05`, and `067-14`.
- Target files:
  - `crates/z00z_runtime/aggregators/src/transport.rs`
  - `crates/z00z_runtime/aggregators/src/evidence.rs`
  - `crates/z00z_runtime/aggregators/tests/test_transport_fault_matrix.rs`
  - `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs`
  - `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T14-01` | delay/reorder/drop/duplicate matrix | deterministic observed order and evidence |
| `067-T14-02` | transport-injected vote rejects | unreplayed vote is never counted |
| `067-T14-03` | replayed envelope does not double count | idempotent or rejected |
| `067-T14-04` | partition/heal safety | stale conflicting subject rejects |
| `067-T14-05` | restart/reconnect cannot replay old membership generation | stale envelope rejects before vote count |
| `067-T14-06` | minority partition cannot synthesize QC | no certificate is emitted |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_fault_matrix -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_adapter -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_equivocation_evidence -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`

- Evidence assertions:
  - every counted vote has replay result, signature result, envelope id, membership generation, subject digest, and voter id;
  - fault rows cover delay, reorder, duplicate, drop, replay, partition, heal, restart, and reconnect.

- Anti-placeholder reminders:
  - tests that call the certificate builder directly without transport envelopes do not satisfy this task.

### ✅ TT-15 TS-15 HotStuff Like Local Backend Contract

- Maps to `TS-15`, `VERDICT-LCS-06`, and `067-15`.
- Target files:
  - `crates/z00z_runtime/aggregators/src/bft_committee.rs`
  - `crates/z00z_runtime/aggregators/src/bft_engine.rs`
  - `crates/z00z_runtime/aggregators/src/hotstuff_local.rs`
  - `crates/z00z_runtime/aggregators/src/lib.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hotstuff_local_backend.rs`
  - `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T15-01` | view and leader proposal path | proposal binds real subject digest |
| `067-T15-02` | view-change after timeout | structured timeout evidence emitted |
| `067-T15-03` | CFT cannot be renamed BFT | `sim_5a7s` BFT mode rejects |
| `067-T15-04` | backend QC cannot bypass validator gate | detached backend QC rejects |
| `067-T15-05` | `n >= 3f + 1` and `quorum >= 2f + 1` are enforced | below-threshold membership or votes reject |
| `067-T15-06` | stale membership digest in view-change rejects | view-change is not accepted |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hotstuff_local_backend -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_bft_committee_rules -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_fault_matrix -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`

- Evidence assertions:
  - evidence records include view, leader, proposal digest, subject digest, timeout/view-change reason, backend QC digest, and validator binding.

- Anti-placeholder reminders:
  - a trait named HotStuff without view/leader/state-machine tests does not satisfy this task.

### ✅ TT-16 TS-16 Celestia Local Artifact Conformance

- Maps to `TS-16`, `VERDICT-LCS-07`, and `067-16`.
- Target files:
  - `crates/z00z_rollup_node/src/celestia_local.rs`
  - `crates/z00z_rollup_node/src/da.rs`
  - `crates/z00z_rollup_node/src/lib.rs`
  - `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`
  - `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
  - `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T16-01` | valid local blob resolves | validator receives same artifact contract as local DA |
| `067-T16-02` | wrong namespace rejects | deterministic reject |
| `067-T16-03` | wrong commitment or missing payload rejects | deterministic reject |
| `067-T16-04` | stale anchor or unanchored-height limit | reject or degraded mode recorded |
| `067-T16-05` | mismatched QC digest rejects | validator-facing artifact fails |
| `067-T16-06` | blob content resolving to validator-rejected artifact rejects | no final accept |

- Commands:
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
  - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`

- Evidence assertions:
  - artifact JSON includes namespace, blob bytes or digest, blob commitment, height, inclusion reference, QC digest, retention state, unanchored height, degraded flag, resolve output, retrieve output, and verify result.

- Anti-placeholder reminders:
  - provider-name-only adapters, constant commitments, or detached payload resolvers do not satisfy this task.

### ✅ TT-17 TS-17 Structured Evidence Registry

- Maps to `TS-17`, `VERDICT-LCS-08`, and `067-17`.
- Target files:
  - `crates/z00z_runtime/aggregators/src/evidence.rs`
  - `crates/z00z_runtime/aggregators/src/lib.rs`
  - `crates/z00z_runtime/aggregators/tests/test_structured_evidence_registry.rs`
  - `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
  - `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`
  - `crates/z00z_simulator/src/scenario_11/report.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T17-01` | every Gate 14 evidence type is emitted | structured record with artifact digest |
| `067-T17-02` | string-only evidence rejects | report validation fails |
| `067-T17-03` | equivocation evidence binds conflict material | conflicting vote/signature material present |
| `067-T17-04` | missing blob evidence binds namespace and commitment | artifact fields present |
| `067-T17-05` | wrong root and wrong route digest evidence are digest-bound | evidence references subject/route artifacts |
| `067-T17-06` | stale member and split-brain evidence include membership generation | malformed rows reject |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_structured_evidence_registry -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_equivocation_evidence -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`

- Evidence assertions:
  - every required evidence type has positive emission, serialization/reload, digest binding, and malformed negative coverage.

- Anti-placeholder reminders:
  - evidence enums that are never emitted by runtime tests do not satisfy this task.

### ✅ TT-18 TS-18 Glossary Claim Registry And Report Honesty

- Maps to `TS-18`, `VERDICT-LCS-09`, and `067-18`.
- Target files:
  - `.planning/phases/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md`
  - `.planning/phases/067-Sharded-Concensus/067-CLAIM-AUDIT.md`
  - `scripts/audit/audit_067_claims.py`
  - `crates/z00z_simulator/src/scenario_11/report.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T18-01` | every term has owner/artifact/tests/claim level | audit passes |
| `067-T18-02` | missing row rejects | audit fails fixture |
| `067-T18-03` | bare external backend claim rejects | report guard fails |
| `067-T18-04` | live-claim-removed stays non-live | final report cannot promote it |
| `067-T18-05` | simulated-full cites executable local evidence | audit fails if evidence refs are absent |
| `067-T18-06` | duplicate term or duplicate claim row rejects | audit fails deterministically |

- Commands:
  - `python3 scripts/audit/audit_067_claims.py`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
  - `rg -n "BFT|Celestia|devnet|HotStuff|production signature|planner HA|slashing|finality" .planning/phases/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md`

- Evidence assertions:
  - registry rows include term, code owner, artifact/API, positive test, negative test, claim level, evidence refs, and plan id;
  - forbidden live claims fail report validation before final closure.

- Anti-placeholder reminders:
  - a manual glossary table without audit script/test does not satisfy this task.

### ✅ TT-19 TS-19 Final Local Conformance Simulation Gate

- Maps to `TS-19`, `VERDICT-LCS-10`, and `067-19`.
- Target files:
  - `crates/z00z_simulator/src/scenario_11/mod.rs`
  - `crates/z00z_simulator/src/scenario_11/report.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
  - `.planning/phases/067-Sharded-Concensus/067-FINAL-CONFORMANCE.md`
  - `.planning/phases/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T19-01` | full package-to-validator chain | package, route, plan, subject, votes, QC, DA/Celestia-local, validator, and report share one subject |
| `067-T19-02` | predecessor slice coverage | every `TS-01` through `TS-19` case exists in a test file or simulator row |
| `067-T19-03` | final evidence bundle completeness | every required JSON artifact is produced and schema-asserted |
| `067-T19-04` | verdict hard blockers reject | all blocker negatives from `067-verdict.md` produce expected reject/degraded status |
| `067-T19-05` | process/devnet smoke is represented | smoke evidence links to final report and claim registry |
| `067-T19-06` | no parallel authority was introduced | tests reuse route, placement, theorem, storage, crypto, and validator owners |

- Negative matrix:
  - missing QC rejects;
  - detached QC rejects;
  - network-injected vote rejects;
  - BFT below-threshold profile rejects;
  - Celestia-local wrong namespace or commitment rejects;
  - stale planner or route table rejects;
  - durable restart with divergent root rejects;
  - report overclaim rejects;
  - missing glossary claim row rejects.
- Required final commands:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
  - `python3 scripts/audit/audit_067_claims.py`
  - `bash scripts/hjmt_local_devnet.sh --profile sim_5a7s --smoke --timeout 30`
  - `cargo test --release -p z00z_aggregators --features test-params-fast -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast -- --nocapture`
  - `cargo test --release -p z00z_validators -- --nocapture`
  - `cargo test --release`

- Evidence assertions:
  - `067-FINAL-CONFORMANCE.md` records exact commands, timestamps or deterministic run ids, artifact paths, artifact hashes, pass/fail states, claim levels, and review results;
  - final `report_honesty.json` agrees with `067-GLOSSARY-CLAIMS.md`.

- Anti-placeholder reminders:
  - final closure cannot rely on compile-only, docs-only, disconnected unit tests, TODOs, stubs, fixture-only checks, or string-only evidence.

## ✅ Completion Checklist

- `TS-01` through `TS-19` are implemented with positive and negative coverage.
- `scenario_11` emits the full artifact set with stable evidence fields.
- every cryptographic, membership, lineage, and publication-binding invariant has at least one positive proof and one reject path.
- no closure relies on comments, TODOs, feature flags, or constant digests.
