# Z00Z Implementable Scenario Backlog

[TOC]

Date: 2026-06-22

Status: current-code-only scenario backlog.

Scope: independent scenarios that can be implemented against the repository as it exists now, using local fixtures, unit/integration harnesses, and the current `z00z_simulator`, wallet, storage, runtime, validator, watcher, rollup-node, core, and crypto surfaces. This document deliberately excludes scenarios that require a live testnet, Celestia DA, live external chains, a real OnionNet transport, production DAO machinery, or post-quantum suite registry.

## 🎯 1. Corrected Task Understanding

The goal is not to generate many speculative scenarios. The goal is to converge the whitepaper use cases into a small set of scenario families that can be built now and that demonstrate different aspects of Z00Z as a system.

`scenario_1` remains reference-only. New scenarios below must be independent targets such as `scenario_2`, `scenario_3`, and so on. They may reuse current crates and current fixture patterns, but they must not append new stages to `scenario_1`.

## 🔑 2. Current-Code Feasibility Rule

A scenario stays in this backlog only if it can be implemented with current local code surfaces:

- local genesis/corpus generation;
- local typed core config catalog, including `crates/z00z_core/configs/devnet_rights_config.yaml`;
- local wallet create/export/restore/receive/scan/tx-history surfaces;
- local storage settlement, checkpoint, HJMT, fee replay, and proof surfaces;
- local runtime aggregator ingress, route planning, placement, and recovery;
- local validator object-policy verdicts and checkpoint/publication checks;
- local watcher observation, alert, evidence, and status projection;
- local simulator artifacts and deterministic fixture profiles.

A scenario is removed from this backlog if it requires:

- live testnet;
- live Celestia or real DA provider integration;
- live external-chain bridge, locker, issuer, or relayer;
- real OnionNet runtime;
- post-quantum suite registry/migration implementation;
- live DAO/treasury/AI governance machinery;
- production useful-work coordination layer.

## 📚 3. Glossary

Reader: an internal engineer selecting or implementing one backlog scenario.
Post-read action: pick the correct scenario, understand its boundaries, and add
tests without turning external-blocked architecture into current-code claims.

| Term | Meaning in this document |
| --- | --- |
| Accepted verdict | Validator result that local artifacts, policy, checkpoint, and publication inputs satisfy the current implemented rules. |
| Anti-drift rule | Rule that prevents a plan from copying obsolete architecture, retired paths, or external-blocked claims into active implementation. |
| Anti-placeholder gate | Test requirement that fails if a scenario emits reports without real live computation behind them. |
| Artifact | Deterministic file or record emitted by a scenario and checked by tests. |
| Backlog scenario | Independent implementable simulator or integration target, such as `scenario_2` through `scenario_11`. |
| Celestia DA | External data-availability integration blocked until adapter/devnet work exists. It is excluded from this current-code backlog. |
| Checkpoint authority | Storage-owned rule set that builds, verifies, seals, and reloads checkpoint artifacts. |
| Claim source authority | Storage-owned source of truth for claim-source contracts and replay-related settlement rows. |
| Completion criteria | Minimum observable behavior required before a scenario can be marked complete. |
| Current-code confidence | Estimate of how much required behavior already exists in local code surfaces. |
| Current-code-only | Scenario boundary that uses local repository code and deterministic fixtures, not external services or non-implemented systems. |
| DA adapter | Interface for local or external data-availability publication and resolution. |
| Deterministic fixture | Local repeatable input set whose outputs are stable enough for tests and reports. |
| E2E test | End-to-end local test that crosses multiple crates and proves integration behavior, not just one helper function. |
| Evidence record | Structured record explaining accepted behavior, rejection, tamper, watcher alert, or replay outcome. |
| Existing-crate work | Required change in production crates before a simulator scenario can honestly claim closure. |
| Fault matrix | Table of injected failures, injection point, expected boundary, and expected result. |
| External-blocked feature | Whitepaper or architecture target that is not implemented enough to be claimed by current scenarios because it needs external network, provider, devnet, or deployment work. |
| HJMT | Storage settlement proof family used by current local storage and scenario evidence. |
| Independent scenario | New scenario target that does not append stages to `scenario_1`. |
| Integration test | Test that joins crate boundaries, for example aggregator plus local DA or validator plus checkpoint flow. |
| Local DA | Local deterministic data-availability adapter used for tests and simulator evidence. |
| Local finality boundary | Honest statement that local artifacts are accepted by local rules but are not public network finality. |
| Local mock DA | Deterministic mock DA mode used only as local publication evidence. |
| Local publication | Runtime-owned publication binding plus local DA and validation evidence, not live DA finality. |
| Local whole-system scenario | Scenario that composes stable local surfaces after lower-level scenarios are proven. |
| Nullifier bridge | Wallet-to-storage transition that preserves spend/nullifier state when storage accepts final spent state. |
| Placeholder | Stubbed, hard-coded, or report-only behavior that pretends a scenario ran without executing the required live path. |
| Publication binding | Digest binding publication inputs so validators and watchers see the same checkpoint and route context. |
| Receiver taxonomy | Wallet receive classifications that distinguish ownership, unsupported versions, and reject causes. |
| Report honesty | Requirement that scenario summaries state exactly what local code proved and what remains external-blocked or not-current-code. |
| Scenario home | Approved source/test directory or binary target where an independent scenario should live. |
| Scenario source shape | Named origin of evidence, such as local publication trace, wallet tx RPC evidence, or HJMT examples. |
| Scenario target | Runnable test, binary, or harness that owns one scenario's artifacts. |
| Shard quorum certificate | Local CFT certificate over one shard commit subject, built from primary and secondary-aggregator votes. |
| Simulator closure | Final scenario evidence after lower-level crate rules already exist and are tested. |
| Soft confirmation | Pre-final publication signal that must not mark wallet transactions confirmed by itself. |
| Source-shape contract | Rule that reports must name the actual current source shape that produced evidence. |
| Trace envelope | Narrow persisted scalar record derived from canonical DTOs; it must not replace canonical runtime types. |
| Unit test | Focused test for one type, digest, validator, or rule. |
| Verification anchor | Command or test home that must be used to prove an implementation slice. |
| Wallet confirmation evidence | Typed evidence that lets wallet history move from pending/admitted to confirmed. |

## 🧱 4. Existing-Crate Work Required Outside Simulator

The scenarios below are not simulator-only work. These existing crate seams must
be extended before the corresponding scenario can honestly close.

| Area | Required existing-crate work | Current code anchors | Scenario dependency |
| --- | --- | --- | --- |
| Storage checkpoint authority | Stop aliasing checkpoint `claim_root` from `new_root`; add one checkpoint-owned proof verifier and make seal plus persisted reload use the same rule. | `crates/z00z_storage/src/checkpoint/build.rs`, `crates/z00z_storage/src/checkpoint/store.rs`, `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`, `crates/z00z_storage/src/backend/redb/validate.rs` | `scenario_5`, `scenario_9`, `scenario_10` |
| Storage claim-source and replay authority | Keep `claim_source_contract_for_item(...)` as the claim-source authority and keep final spent/nullifier replay state in storage settlement rows. Do not add a duplicate `claim_source_root_hex` carrier or second replay registry. | `crates/z00z_storage/src/backend/query.rs`, `crates/z00z_storage/src/backend/rows.rs`, `crates/z00z_storage/src/backend/types.rs`, `crates/z00z_storage/src/settlement/store.rs` | `scenario_5`, `scenario_8`, `scenario_10` |
| Validator checkpoint consumer | Keep `CheckpointFlow::try_from_resolved(...)` as a consumer of storage-owned checkpoint artifacts and extend it only through current `Verdict` or `RejectClass` vocabulary. Do not copy checkpoint proof formulas into validators. | `crates/z00z_runtime/validators/src/checkpoint.rs`, `crates/z00z_runtime/validators/src/engine.rs`, `crates/z00z_runtime/validators/src/verdict.rs` | `scenario_7`, `scenario_9`, `scenario_10` |
| Wallet receive taxonomy | Add an explicit unsupported-version receive status or reject path instead of collapsing unsupported asset-pack lanes into `NotMine`. Keep `recv_claim_asset(...)` as the only claimed-asset persistence gate. | `crates/z00z_wallets/src/receiver/scan/types_receive.rs`, `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs`, `crates/z00z_wallets/src/receiver/scan/leaf_scan.rs`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs` | `scenario_3`, `scenario_10` |
| Wallet scan evidence and runtime scan status | If remote scan evidence is used, implement `RemoteScanWorkerImpl` as a fetch-only worker that returns `RemoteScanEvidence` for `WalletService::recv_range(...)`. Derive `ChainService` and `ChainScanRpcImpl` status from real scan state instead of process-local placeholder jobs when a scenario exposes runtime scan progress. Do not wire the excluded duplicate `wallet_service_actions_runtime.rs` as receive authority. | `crates/z00z_wallets/src/chain/scan_engine.rs`, `crates/z00z_wallets/src/chain/scan_engine_impl.rs`, `crates/z00z_wallets/src/services/chain_service.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/chain_impl.rs`, `crates/z00z_wallets/src/adapters/rpc/types/chain.rs`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`, `crates/z00z_wallets/src/services/wallet_service_actions.rs` | `scenario_3`, `scenario_8`, `scenario_10` |
| Wallet nullifier bridge | Make the wallet reserved-to-spent transition explicit and preserve nullifier bytes, `chain_id`, and `tx_digest` when storage accepts the final spent state. | `crates/z00z_wallets/src/claim/nullifier.rs`, `crates/z00z_wallets/src/claim/nullifier_store.rs`, `crates/z00z_storage/src/backend/types.rs`, `crates/z00z_storage/src/backend/rows.rs`, `crates/z00z_simulator/src/scenario_1/claim_pkg_consumer.rs` | `scenario_5`, `scenario_8`, `scenario_10` |
| Local publication contracts | Reuse crate-root facades for publication types, DA adapter, validator verdicts, watcher evidence, and wallet confirmation evidence. Add persisted envelopes only as trace/store records, not replacement DTOs. | `crates/z00z_runtime/aggregators/src/types.rs`, `crates/z00z_runtime/aggregators/src/service.rs`, `crates/z00z_rollup_node/src/da.rs`, `crates/z00z_runtime/validators/src/verdict.rs`, `crates/z00z_runtime/watchers/src/evidence_export.rs`, `crates/z00z_wallets/src/persistence/tx/tx_storage.rs` | `scenario_9`, `scenario_10` |
| Shard quorum certificate and secondary replay | Add first-class local `CommitSubject`, `ShardVote`, `ShardQuorumCertificate`, and `SecondaryReplayVerifier` artifacts under the existing aggregator seam. Bind the certificate digest through local DA and validator acceptance. Do not introduce network BFT, Celestia, or duplicate runtime DTOs for this local scenario. | `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`, `crates/z00z_runtime/aggregators/src/batch_planner.rs`, `crates/z00z_runtime/aggregators/src/placement.rs`, `crates/z00z_runtime/aggregators/src/ingress.rs`, `crates/z00z_runtime/aggregators/src/recovery.rs`, `crates/z00z_runtime/aggregators/src/dist_sim.rs`, `crates/z00z_rollup_node/src/da.rs`, `crates/z00z_runtime/validators/src/checkpoint.rs`, `crates/z00z_runtime/validators/src/engine.rs` | `scenario_11`, `scenario_10` |

Required targeted tests for these crate additions:

- storage: `crates/z00z_storage/tests/test_checkpoint_draft_build.rs`, `crates/z00z_storage/tests/test_checkpoint_finalization.rs`, `crates/z00z_storage/tests/test_checkpoint_store_api.rs`, `crates/z00z_storage/tests/test_redb_reload.rs`, and `crates/z00z_storage/tests/test_claim_source_proof.rs`;
- wallet: source-local tests under `crates/z00z_wallets/src/receiver/scan/`, `crates/z00z_wallets/src/claim/nullifier_store/test_mod.rs`, plus `crates/z00z_wallets/tests/test_claim_import_reason_codes.rs`;
- runtime and node: `crates/z00z_runtime/aggregators/tests/`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, and `crates/z00z_rollup_node/tests/`;
- simulator: current Scenario 1 homes under `crates/z00z_simulator/tests/scenario_1/`, especially `claim_pkg_crypto.rs`, `test_claim_pkg_runtime.rs`, `test_checkpoint_acceptance.rs`, `test_stage6_checkpoint.rs`, `test_stage6_checkpoint_final_gate.rs`, `test_stage6_checkpoint_storage_bridge.rs`, and `test_stage4_tamper.rs`.
- shard quorum certificate: new aggregator tests for subject digest, vote validation, quorum certificate formation, secondary replay rejection, membership drift, and same-term conflicts; new rollup-node and validator tests for certificate digest binding through local DA and resolved-batch validation; a new independent simulator scenario target for full E2E package-to-certificate evidence.

If storage needs a new checkpoint verifier module, add it under
`crates/z00z_storage/src/checkpoint/` and keep it crate-owned.

### 🔗 4.1. Existing-Crate Dependency Order

1. Recheck claim-source baseline first and close it as no-op cleanup unless a
   real rebinding gap is reproduced against
   `AssetStore::claim_source_contract_for_item(...)`.
2. Implement honest checkpoint `claim_root` propagation before verifier
   convergence, because the verifier must validate the final claim-root
   semantics carried by drafts and public inputs.
3. Add one storage-owned checkpoint proof verifier and make
   `CheckpointStore::seal_artifact(...)` plus persisted reload validation call
   the same rule.
4. Extend validator checkpoint consumption only after storage authority is
   unified; validators must consume `z00z_storage::checkpoint` artifacts through
   current `CheckpointFlow`, `Verdict`, and `RejectClass` surfaces.
5. Add unsupported receive taxonomy before scan-engine or runtime scan-status
   work, so scan progress never publishes temporary ownership classes.
6. Extend `RemoteScanWorkerImpl`, `ChainService`, and `ChainScanRpcImpl` only as
   orchestration/status surfaces over `ScanChunk`, `ScanStatePayload`, and
   `WalletService::recv_range(...)`.
7. Add the explicit wallet nullifier reserved-to-spent bridge after checkpoint
   and claim-source semantics are stable.
8. Run simulator closure last; simulator scenarios prove landed storage,
   validator, receive, scan, and nullifier seams instead of discovering them.

### 🚫 4.2. Existing-Crate Anti-Drift Rules

- Do not add a parallel `claim_source_root_hex` carrier, second claim-source
  proof object, second replay registry, or generic spend-replay flattening.
- Do not add a checkpoint verifier outside `z00z_storage::checkpoint`; if a new
  helper is needed, keep it under `crates/z00z_storage/src/checkpoint/`.
- Do not change checkpoint ID semantics to hash arbitrary proof bytes unless
  IDs, codecs, persisted data rules, and tests are migrated in one explicit
  later decision.
- Do not describe current attested `cp_proof` bytes as a finished trustless or
  recursive proof backend.
- Do not let validators invent a checkpoint artifact schema or duplicate
  checkpoint proof formulas; validators remain consumers of storage-owned
  checkpoint authority.
- Do not describe wallet receive sync as delta-only import. The live boundary is
  `ScanChunk` plus `ScanStatePayload`.
- Do not add a second scan cursor model or persist raw runtime scan DTOs as a
  receive authority.
- Do not reopen `recv_claim_asset(...)` for unsupported-version taxonomy work;
  keep it as the canonical claimed-asset persistence gate.
- Do not use simulator closure as a substitute for the earlier storage,
  validator, receive, scan, or nullifier crate work.

### 🧪 4.3. Existing-Crate Verification Anchors

Use current package and feature names only. The workspace has no package named
`z00z_runtime`; runtime code is split into `z00z_aggregators`,
`z00z_validators`, and `z00z_watchers`.

```bash
cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_draft_build -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture
cargo test -p z00z_storage --release --test test_redb_reload -- --nocapture
cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools --test test_claim_import_reason_codes -- --nocapture
cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools test_recv_range_restart -- --nocapture
cargo test -p z00z_aggregators --features test-params-fast
cargo test -p z00z_validators
cargo test -p z00z_watchers
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test scenario_1 test_checkpoint_acceptance -- --nocapture
```

Replace these with narrower commands only when an implementation PLAN proves the
nearest current test home is different. Do not leave unsupported feature names
or nonexistent package names in an active PLAN.

## 📌 5. Consolidated Scenario Map

| Scenario | Name | Why It Exists | Current-code confidence |
| --- | --- | --- | --- |
| `scenario_2` | Genesis, Crypto, And Domain Integrity Drill | Prove genesis, range-proof, encoding, domain, and theorem boundaries | High |
| `scenario_3` | Wallet Receive, Recovery, And Privacy Drill | Prove wallet-local authority, receive safety, backup/restore, and redaction | High |
| `scenario_4` | Typed Object Policy Matrix | Prove asset/voucher/right/fee policy behavior and negative verdicts | High |
| `scenario_5` | Settlement, Checkpoint, HJMT, And Fee Replay Drill | Prove storage roots, checkpoint links, proof families, and replay protection | High |
| `scenario_6` | Runtime Route, Aggregator Churn, And Recovery Drill | Prove local aggregator planning, route-table validation, and split-brain rejection | High |
| `scenario_7` | Validator And Watcher Evidence Drill | Prove verdict projection, watcher alerts, evidence export, and status snapshots | High |
| `scenario_8` | Offline Package And Local Reconciliation Drill | Prove portable tx package, pending/admitted/confirmed states, and replay-like local failures | Medium |
| `scenario_9` | Local Publication, Evidence, Restart, And Tamper Drill | Prove mock DA publication, evidence persistence, wallet confirmation gate, restart, replay, and tamper handling | Medium |
| `scenario_11` | Shard Quorum Certificate And Secondary Replay Drill | Prove local shard quorum certificate, secondary replay, DA binding, and validator acceptance without changing `scenario_1` | Medium |
| `scenario_10` | Local Whole-System Scenario | Compose the stable local surfaces into one end-to-end organism without external DA/testnet | Medium |

## ✅ 6. scenario_2 Genesis, Crypto, And Domain Integrity Drill

### 🎯 6.1. Purpose

Demonstrate the cryptographic maturity that is already represented in the repository: genesis proof verification, deterministic genesis state hashing, ZkPack wire rejection, asset-pack version rejection, domain separation stability, and public-artifact settlement theorem checks.

This scenario is intentionally local. It does not claim network finality, DA availability, or post-quantum migration.

### 🔎 6.2. Whitepaper Basis

- Cryptographic integrity and proof discipline: `docs/Z00Z-Main-Whitepaper.md:276-326`.
- Cryptography detail boundary: `docs/Z00Z-Main-Whitepaper.md:1323-1363`.
- Implementation status and evidence boundaries: `docs/Z00Z-Main-Whitepaper.md:1150-1288`.
- PQ migration is design work outside this current-code scenario and must not be claimed here: `docs/Z00Z-PQ-Migration-Whitepaper.md:149-225`.

### ⚙️ 6.3. Current Code To Study First

- Genesis range-proof batch verification: `crates/z00z_core/src/genesis/genesis_verification.rs:1-41`.
- Genesis state hash over assets, rights, and vouchers: `crates/z00z_core/src/genesis/genesis_verification.rs:99-205`.
- ZkPack fixed wire format and reject paths: `crates/z00z_crypto/src/protocol/zkpack.rs:1-70`.
- Asset-pack version detection: `crates/z00z_core/src/assets/version.rs:1-23`.
- Asset-pack decode rejection: `crates/z00z_core/src/assets/leaf.rs:266-279`.
- Wallet domain freeze tests: `crates/z00z_wallets/src/domains/definitions/test_mod.rs:121-180`.
- Settlement theorem public-artifact verifier: `crates/z00z_rollup_node/src/lib.rs:97-139`.
- Output proof range-proof requirement: `crates/z00z_rollup_node/src/lib.rs:274-290`.

### ⚙️ 6.4. Scenario Flow

1. Build a deterministic local genesis corpus.
2. Verify genesis asset range proofs.
3. Compute genesis state hash and write a stable artifact.
4. Mutate one right field and verify the state hash changes.
5. Mutate one voucher field and verify the state hash changes.
6. Build valid and invalid ZkPack wires.
7. Build valid and unsupported asset-pack decode cases.
8. Run domain snapshot stability checks.
9. Run settlement theorem verification on a local public-artifact fixture.
10. Inject wrong proof payload, wrong root, and missing range proof.

### 🔑 6.5. Invariants

- Missing range proof rejects.
- Unsupported ZkPack version rejects.
- Wrong ZkPack length rejects.
- Unsupported asset-pack lane rejects.
- Domain strings stay stable and unique.
- Genesis hash commits rights and vouchers, not only cash assets.
- Settlement theorem verifier uses public artifacts and does not rebuild private witnesses.

### 📦 6.6. Required Artifacts

- `scenario_2/genesis_proof_report.json`
- `scenario_2/genesis_state_hash.json`
- `scenario_2/genesis_mutation_matrix.json`
- `scenario_2/zkpack_wire_rejects.json`
- `scenario_2/asset_pack_rejects.json`
- `scenario_2/domain_snapshot_report.json`
- `scenario_2/settlement_theorem_report.json`

### ✅ 6.7. Tests

- valid genesis corpus verifies;
- asset with missing range proof rejects;
- right mutation changes genesis hash;
- voucher mutation changes genesis hash;
- invalid ZkPack version and length reject;
- unsupported asset-pack version rejects;
- domain strings are stable;
- settlement theorem rejects wrong checkpoint proof/root/replay binding.

### ✅ 6.8. Completion Criteria

`scenario_2` is complete when it proves the current local crypto/integrity surfaces with one positive path and deterministic negative cases, without making post-quantum or network-finality claims.

## ✅ 7. scenario_3 Wallet Receive, Recovery, And Privacy Drill

### 🎯 7.1. Purpose

Demonstrate wallet-local authority and privacy hygiene that can be tested now: payment request validation, receiver scan behavior, request-bound KDF order, tx history states, backup/export/restore, typed object persistence, logging redaction, and metadata redaction.

### 🔎 7.2. Whitepaper Basis

- Wallet responsibilities, request-bound receive, light sync, recovery, and untrusted receiver inputs: `docs/Z00Z-Main-Whitepaper.md:533-559`.
- Stealth ownership and receiver privacy: `docs/Z00Z-Main-Whitepaper.md:574-592`.
- Wallet UX/defaults and QA hooks: `docs/Z00Z-Privacy-Threat-Model-Whitepaper.md:681-748`.
- Wallet and interface boundary: `docs/Z00Z-Legal-Architecture-Whitepaper.md:661-745`.

### ⚙️ 7.3. Current Code To Study First

- Payment request validation: `crates/z00z_wallets/src/receiver/request/mod.rs:92-125`.
- Receiver scan types and persist decision: `crates/z00z_wallets/src/receiver/scan/types_receive.rs:305-390`.
- Request-bound scan KDF ordering: `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs:110-124`.
- Unsupported pack version scan skip: `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs:59-72`.
- Wallet tx status and confirmation evidence: `crates/z00z_wallets/src/persistence/tx/tx_storage.rs:19-68`.
- Wallet guide canonical model and state planes: `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:109-149`.
- RPC log redaction tests: `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-325`.
- Backup metadata redaction: `crates/z00z_wallets/tests/test_backup_metadata_policy.rs:80-111`.

### ⚙️ 7.4. Scenario Flow

1. Create local wallets and receiver material.
2. Build a valid payment request and validate it.
3. Try wrong chain id, expired request, revoked pin, and identity mismatch.
4. Scan outputs with request-bound candidates before request-less fallback.
5. Scan unsupported asset-pack lane and verify skip/reject behavior.
6. Create tx history entries covering pending, confirmed, failed, and cancelled.
7. Export encrypted backup and restore into a fresh local wallet.
8. Verify assets, objects, tx sidecar, scan state, and quarantine state round-trip.
9. Capture logs and verify no password, seed, token, memo, or full wallet id leaks.

### 🔑 7.5. Invariants

- Receiver validation rejects wrong chain and expired requests.
- Request-bound scan candidates are tried before generic fallback.
- Unsupported pack lane does not become a wallet-owned spendable object.
- `.wlt` remains the canonical wallet authority.
- Tx history is explicit sidecar behavior, not hidden consensus state.
- Public artifacts do not leak secrets.

### 📦 7.6. Required Artifacts

- `scenario_3/payment_request_matrix.json`
- `scenario_3/receiver_scan_order.json`
- `scenario_3/unsupported_pack_report.json`
- `scenario_3/tx_history_state_report.json`
- `scenario_3/wallet_restore_roundtrip.json`
- `scenario_3/object_projection_after_restore.json`
- `scenario_3/redaction_report.json`

### ✅ 7.7. Tests

- valid request approves;
- wrong-chain request rejects;
- expired request rejects;
- identity mismatch reports mismatch;
- unsupported asset-pack lane is not imported as spendable;
- backup/restore preserves owned assets and owned objects;
- tx sidecar behavior is explicit;
- logs and artifacts redact secrets and full wallet ids.

### ✅ 7.8. Completion Criteria

`scenario_3` is complete when wallet-local receive/recovery/privacy behavior is covered by deterministic local artifacts and redaction tests.

## ✅ 8. scenario_4 Typed Object Policy Matrix

### 🎯 8.1. Purpose

Consolidate voucher, rights, fee-credit, service entitlement, machine capability, and one-time-use ideas into one implementable typed-object scenario. This avoids a garden of similar scenarios and demonstrates the object model that already exists in storage, validators, watcher alerts, wallet guide, and genesis fixtures.

### 🔎 8.2. Whitepaper Basis

- Asset/voucher/right triad: `docs/Assets-Rights-Vauchers-Whitepaper.md:158-240`.
- Voucher lifecycle and conditional value: `docs/Assets-Rights-Vauchers-Whitepaper.md:330-446`.
- Rights as authority without value: `docs/Assets-Rights-Vauchers-Whitepaper.md:463-531`.
- Policy/action/witness/fee-support boundaries: `docs/Assets-Rights-Vauchers-Whitepaper.md:544-707`.
- Service, machine, and agent rights: `docs/Z00Z-UseCases-Whitepaper.md:611-730`.
- Agentic and offline economy rights primitives: `docs/Z00Z-Agentic-Offline-Economy-Whitepaper.md:143-462`.

### ⚙️ 8.3. Current Code To Study First

- `RightClass` variants: `crates/z00z_storage/src/settlement/record.rs:156-164`.
- `RightLeaf`, `VoucherLeaf`, and `FeeEnvelope`: `crates/z00z_storage/src/settlement/record.rs:258-317`, `crates/z00z_storage/src/settlement/record.rs:511-527`.
- Voucher actions and settlement actions: `crates/z00z_storage/src/settlement/tx_plan_types.rs:36-69`.
- Runtime object package and reject taxonomy: `crates/z00z_storage/src/settlement/object_package_contract.rs:27-94`.
- Object package inspection: `crates/z00z_storage/src/settlement/object_package_contract.rs:215-305`.
- Required-right rejection mapping: `crates/z00z_storage/src/settlement/object_package_contract.rs:406-424`.
- Existing validator object-policy tests: `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs:40-220`.
- Rights fixtures: `crates/z00z_core/configs/devnet_rights_config.yaml:1-106`.
- Wallet typed object catalog: `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:13-74`.

### ⚙️ 8.4. Scenario Flow

1. Build a local object corpus with assets, vouchers, and rights.
2. Create a valid voucher action package.
3. Create valid right grant/delegate/consume cases using current right classes.
4. Create a fee-supported object action using `FeeEnvelope`.
5. Run validator verdicts for accepted cases.
6. Inject unknown policy, wrong family proof, voucher-as-cash, right-as-value, missing right, expired right, revoked right, consumed right, replay, double redeem, stale root, forced acceptance, and fee boundary defects.
7. Project object rows through wallet object inventory rules.
8. Emit watcher-style reject families for invalid verdicts.

### 🔑 8.5. Invariants

- Assets are final spendable value.
- Vouchers are conditional claims and do not appear as ordinary cash before valid redeem.
- Rights are zero-value authority and must not be spendable value.
- Fee envelopes are processing support and do not prove ownership or right validity.
- Unknown-policy objects stay quarantined.
- Every invalid object package maps to a stable reject code.

### 📦 8.6. Required Artifacts

- `scenario_4/object_corpus_manifest.json`
- `scenario_4/valid_object_actions.json`
- `scenario_4/object_reject_matrix.json`
- `scenario_4/validator_verdict_matrix.json`
- `scenario_4/wallet_object_projection.json`
- `scenario_4/watch_object_alerts.json`

### ✅ 8.7. Tests

- valid object package accepted;
- unknown policy rejects;
- missing/expired/revoked/consumed right rejects specifically;
- fee boundary violation rejects;
- voucher-as-cash rejects;
- right-as-value rejects;
- double redeem rejects;
- unknown-policy object remains outside spendable balance.

### ✅ 8.8. Completion Criteria

`scenario_4` is complete when one consolidated object-policy matrix demonstrates the current asset/voucher/right/fee model with both accepted and rejected behavior.

## ✅ 9. scenario_5 Settlement, Checkpoint, HJMT, And Fee Replay Drill

### 🎯 9.1. Purpose

Demonstrate local storage maturity: settlement root authority, checkpoint draft/artifact/link/exec-input boundaries, HJMT proof families, batch proof route checks, privacy-safe occupancy evidence, and fee replay metadata.

### 🔎 9.2. Whitepaper Basis

- Canonical state objects and cryptographic integrity: `docs/Z00Z-Main-Whitepaper.md:173-326`.
- Checkpoints as validation boundary: `docs/Z00Z-Main-Whitepaper.md:300-326`.
- Scalability, publication, and proof evidence boundary: `docs/Z00Z-Main-Whitepaper.md:770-862`.
- Privacy metrics and exact-count leakage risk: `docs/Z00Z-Privacy-Threat-Model-Whitepaper.md:464-604`.

### ⚙️ 9.3. Current Code To Study First

- Settlement root public contract: `crates/z00z_storage/src/settlement/root_types.md:8-37`.
- Development hard cutover and live HJMT backend: `crates/z00z_storage/src/settlement/root_types.md:73-130`.
- Checkpoint store facade: `crates/z00z_storage/src/checkpoint/store.rs:201-252`.
- Checkpoint seal and load-link validation: `crates/z00z_storage/src/checkpoint/store.rs:309-363`.
- Checkpoint draft claim-root source: `crates/z00z_storage/src/checkpoint/build.rs`.
- Persisted checkpoint reload validation: `crates/z00z_storage/src/backend/redb/validate.rs`.
- Storage claim-source and replay rows: `crates/z00z_storage/src/backend/query.rs`, `crates/z00z_storage/src/backend/rows.rs`, `crates/z00z_storage/src/backend/types.rs`.
- HJMT proof blob and batch mode: `crates/z00z_storage/src/settlement/hjmt_proof.rs:153-180`.
- Batch proof and route checks: `crates/z00z_storage/src/settlement/proof_batch_verify.rs:65-180`.
- HJMT privacy regression tests: `crates/z00z_storage/tests/test_hjmt_privacy_regression.rs:14-111`.
- Fee replay metadata and fixture: `crates/z00z_storage/tests/test_fee_replay.rs:41-55`, `crates/z00z_storage/tests/test_fee_replay.rs:137-154`.

### ⚙️ 9.4. Scenario Flow

1. Create local settlement store with generalized assets, vouchers, and rights.
2. Apply put/delete settlement operations.
3. Emit inclusion proof for existing path.
4. Emit deletion/non-existence proof for absent or deleted path.
5. Build batch proof and validate contract.
6. Build checkpoint draft, snapshot, exec input, proof, artifact, and link.
7. Load link and verify replay/root binding.
8. Execute fee-supported operation and record replay metadata.
9. Inject stale prior context, tampered occupancy evidence, route mismatch, duplicate shard, wrong checkpoint proof, wrong exec input, stale root, and fee replay tamper.
10. Prove claim-carrying checkpoint drafts use a storage-backed `claim_root` and non-claim drafts keep `claim_root` absent.
11. Reuse one checkpoint-owned proof verifier for seal and persisted reload paths.
12. Add theorem-style local assertions for state roots, claim roots, nullifier transitions, artifact links, snapshot IDs, exec input IDs, and fragment IDs.
13. Restart memory-backed and redb-backed storage fixtures where available and verify pending, sealed, resolved, and tampered artifacts.
14. Keep optional recursive proof artifacts as non-authoritative sidecar evidence only.

### 🔑 9.5. Invariants

- `SettlementStateRoot` is the public semantic root.
- No old asset-root plane is accepted as authority.
- Checkpoint seal requires matching snapshot and exec input.
- Link loading rechecks artifact, snapshot, exec input, and roots.
- `claim_root` is storage-owned or absent; it must not silently alias `new_root`.
- Seal, reload, and validator consumers must not disagree about checkpoint proof rules.
- HJMT proof-visible artifacts do not expose exact local occupancy counts or timing.
- Fee replay metadata survives persistence/recovery.
- Local theorem checks are evidence about current storage artifacts, not production theorem rollout or recursive proof replacement.

### 📦 9.6. Required Artifacts

- `scenario_5/settlement_root_report.json`
- `scenario_5/hjmt_proof_family_matrix.json`
- `scenario_5/hjmt_batch_contract_report.json`
- `scenario_5/checkpoint_chain_report.json`
- `scenario_5/checkpoint_tamper_matrix.json`
- `scenario_5/checkpoint_claim_root_matrix.json`
- `scenario_5/checkpoint_theorem_pack.json`
- `scenario_5/checkpoint_restart_matrix.json`
- `scenario_5/fee_replay_report.json`
- `scenario_5/proof_privacy_report.json`

### ✅ 9.7. Tests

- inclusion proof validates;
- deletion/non-existence proof validates;
- stale prior context rejects;
- tampered occupancy evidence rejects;
- route mismatch rejects;
- duplicate shard rejects;
- checkpoint wrong proof/root/exec rejects;
- claim-carrying checkpoint emits a real storage-backed `claim_root`;
- non-claim checkpoint keeps `claim_root` absent;
- checkpoint seal and persisted reload call the same proof rule;
- theorem-style local checks reject statement digest, claim root, snapshot ID, exec input ID, fragment ID, and publication-ref drift;
- restart matrix reloads pending, sealed, resolved, and tampered artifacts with deterministic outcomes;
- fee replay rejects duplicate/tampered replay.

### ✅ 9.8. Completion Criteria

`scenario_5` is complete when storage/checkpoint/HJMT/fee replay behavior is proven locally with positive and negative artifacts, one storage-owned checkpoint proof rule, honest `claim_root` handling, and restart/tamper evidence that makes no node, DA, testnet, or recursive-proof rollout claim.

## ✅ 10. scenario_6 Runtime Route, Aggregator Churn, And Recovery Drill

### 🎯 10.1. Purpose

Demonstrate the runtime layer that exists now without requiring a live network: aggregator ingress digest rebinding, route table validation, batch planning, placement, recovery capture/resume, standby takeover, and split-brain/stale recovery rejection.

### 🔎 10.2. Whitepaper Basis

- Rollup ordering, publication, and verification roles: `docs/Z00Z-Main-Whitepaper.md:326-442`.
- Malicious aggregators and operational failure handling: `docs/Z00Z-Main-Whitepaper.md:868-977`.
- Operator and recovery boundary: `docs/Z00Z-Main-Whitepaper.md:1405-1433`.

### ⚙️ 10.3. Current Code To Study First

- Aggregator README boundaries: `crates/z00z_runtime/aggregators/README.md:14-30`.
- Ingress digest rebinding: `crates/z00z_runtime/aggregators/src/ingress.rs:9-75`.
- Route error taxonomy: `crates/z00z_runtime/aggregators/src/batch_planner.rs:17-33`.
- Route table validation and canonical bytes: `crates/z00z_runtime/aggregators/src/batch_planner.rs:100-220`.
- Aggregator service boundary and publication binding helper: `crates/z00z_runtime/aggregators/src/service.rs:13-48`.
- Recovery capture/resume rules: `crates/z00z_runtime/aggregators/src/recovery.rs:12-213`.

### ⚙️ 10.4. Scenario Flow

1. Admit valid tx and claim payloads through ingress.
2. Reject forged tx digest and forged claim digest.
3. Build valid route table and planned batch.
4. Inject route gaps, overlaps, duplicate shards, foreign shards, unused shards, bad previous generation, truncated bytes, and trailing bytes.
5. Capture recovery record after handoff.
6. Resume primary restart with matching durable state.
7. Resume lawful standby takeover.
8. Inject split-brain, wrong lineage, wrong route digest, stale replay, stale local root, stale restart, and standby-down states.

### 🔑 10.5. Invariants

- Caller-supplied digest is never planner authority.
- Route table must be sorted, complete, non-overlapping, and generation-linked.
- Placement data may flow downstream but does not become checkpoint authority.
- Same-lineage standby takeover is lawful only under current recovery checks.
- Split-brain and stale recovery reject fail-closed.

### 📦 10.6. Required Artifacts

- `scenario_6/ingress_digest_report.json`
- `scenario_6/route_table_validation_matrix.json`
- `scenario_6/batch_planning_report.json`
- `scenario_6/recovery_capture_report.json`
- `scenario_6/recovery_resume_matrix.json`
- `scenario_6/split_brain_rejects.json`

### ✅ 10.7. Tests

- forged tx digest rejects;
- forged claim digest rejects;
- valid route table accepts;
- every route-table defect rejects;
- primary restart succeeds with matching state;
- lawful standby takeover succeeds;
- non-standby takeover rejects;
- wrong lineage/route/stale root/stale replay rejects.

### ✅ 10.8. Completion Criteria

`scenario_6` is complete when runtime planning and recovery behavior can be demonstrated locally without testnet or real aggregator processes.

## ✅ 11. scenario_7 Validator And Watcher Evidence Drill

### 🎯 11.1. Purpose

Demonstrate validator and watcher behavior that is already implemented locally: resolved batch verdicts, object verdict projection, publication binding checks, watcher checked snapshots, watcher alerts, evidence records, provider signal projection, and rollup-node status projection.

This scenario replaces speculative DA/Celestia scenarios. It does not require a live DA provider.

### 🔎 11.2. Whitepaper Basis

- Stateless validator and watcher paths: `docs/Z00Z-Main-Whitepaper.md:424-436`.
- Failure handling, fraud-proof/slashing direction, recovery/publication ledger: `docs/Z00Z-Main-Whitepaper.md:912-977`.
- Observation and disclosure boundaries: `docs/Z00Z-Privacy-Threat-Model-Whitepaper.md:791-862`.

### ⚙️ 11.3. Current Code To Study First

- Validator boundary and verdict construction: `crates/z00z_runtime/validators/src/engine.rs:13-80`.
- Verdict, reject classes, and object reject mapping: `crates/z00z_runtime/validators/src/verdict.rs:69-133`.
- Watcher README boundaries: `crates/z00z_runtime/watchers/README.md:1-34`.
- Watcher checked snapshots and object alerts: `crates/z00z_runtime/watchers/src/engine.rs:15-125`.
- Watcher alert severity mapping: `crates/z00z_runtime/watchers/src/engine.rs:145-155`.
- Evidence record export: `crates/z00z_runtime/watchers/src/evidence_export.rs:15-88`.
- Publication watch route/binding checks: `crates/z00z_runtime/watchers/src/publication.rs:30-82`.
- Rollup node status projection: `crates/z00z_rollup_node/src/status.rs:10-39`.

### ⚙️ 11.4. Scenario Flow

1. Build local `ResolvedBatch` fixture with accepted checkpoint flow.
2. Build local `ResolvedBatch` fixture with object rejects.
3. Run `ValidatorBoundary::verdict_for_batch`.
4. Feed accepted and rejected verdicts to watcher.
5. Build watcher checked snapshots.
6. Export evidence records.
7. Project node status object reject codes.
8. Inject missing verdict, missing binding, batch mismatch, checkpoint mismatch, binding mismatch, route mismatch, and exec mismatch.

### 🔑 11.5. Invariants

- Validator owns verdict construction.
- Watcher observes already-published runtime state.
- Watcher does not own settlement semantics.
- Evidence records preserve binding digest and object reject codes.
- Status projection exposes object reject codes without changing verdict meaning.

### 📦 11.6. Required Artifacts

- `scenario_7/validator_verdicts.json`
- `scenario_7/object_reject_projection.json`
- `scenario_7/watcher_checked_snapshots.json`
- `scenario_7/publication_watch_rejects.json`
- `scenario_7/evidence_records.json`
- `scenario_7/status_projection.json`

### ✅ 11.7. Tests

- accepted resolved batch produces accepted verdict;
- rejected object package maps to expected reject class;
- watcher checked snapshot includes binding/route digest;
- missing verdict rejects;
- binding mismatch rejects;
- route mismatch rejects;
- evidence record exposes object reject codes;
- status projection returns object reject codes.

### ✅ 11.8. Completion Criteria

`scenario_7` is complete when validator/watcher/status behavior is covered locally with deterministic positive and negative evidence.

## ✅ 12. scenario_8 Offline Package And Local Reconciliation Drill

### 🎯 12.1. Purpose

Demonstrate the offline-first story only to the extent current code supports it: portable tx packages, wallet tx states, local admission/confirmation receipts, import-ready verification shape, and replay-like duplicate handling in local harnesses.

This scenario must not claim full offline double-spend arbitration or network reconciliation beyond current code.

### 🔎 12.2. Whitepaper Basis

- Offline-first private cash: `docs/Z00Z-UseCases-Whitepaper.md:197-266`.
- Wallet-local ownership and spend-then-reconcile model: `docs/Z00Z-Main-Whitepaper.md:442-559`.
- Linked liability direction is partial and outside this current-code scenario; it should not be overclaimed: `docs/Z00Z-Linked-Liability-Whitepaper.md:22-64`.

### ⚙️ 12.3. Current Code To Study First

- Runtime admission and confirmation receipts: `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:69-94`.
- Verify tx package response shape: `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:190-220`.
- Portable wallet tx package: `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:212-220`.
- Tx storage states and confirmation evidence: `crates/z00z_wallets/src/persistence/tx/tx_storage.rs:6-68`.
- Tx storage trait lifecycle methods: `crates/z00z_wallets/src/persistence/tx/tx_storage.rs:97-153`.
- Receiver request validation: `crates/z00z_wallets/src/receiver/request/mod.rs:92-125`.

### ⚙️ 12.4. Scenario Flow

1. Build portable wallet tx package locally.
2. Verify package shape and import readiness.
3. Record pending tx in tx history.
4. Record submitted/admitted state.
5. Attach typed confirmation evidence.
6. Attempt duplicate package import/submission.
7. Attempt wrong-chain/expired receiver request.
8. Attempt malformed metadata hash or tx bytes.

### 🔑 12.5. Invariants

- Offline package is not final settlement.
- Pending tx becomes confirmed only with typed confirmation evidence.
- Wrong-chain and expired request reject before receive path approval.
- Duplicate package handling is explicit.
- Scenario summary must state this is local reconciliation, not full offline fraud arbitration.

### 📦 12.6. Required Artifacts

- `scenario_8/portable_tx_package.json`
- `scenario_8/package_verify_report.json`
- `scenario_8/tx_state_timeline.json`
- `scenario_8/confirmation_evidence.json`
- `scenario_8/duplicate_package_report.json`
- `scenario_8/offline_claim_boundary.md`

### ✅ 12.7. Tests

- package can be stored as pending;
- admission receipt moves tx to admitted state;
- confirmation evidence moves tx to confirmed state;
- malformed package rejects;
- duplicate import/submission is deterministic;
- wrong-chain/expired request rejects.

### ✅ 12.8. Completion Criteria

`scenario_8` is complete when local portable package and tx-state reconciliation behavior is shown honestly without claiming unavailable network-level arbitration.

## ✅ 13. scenario_9 Local Publication, Evidence, Restart, And Tamper Drill

### 🎯 13.1. Purpose

Demonstrate the local publication and evidence story that can be built now:
wallet transaction packages, ordered batches, local mock DA publication refs,
checkpoint artifacts, soft confirmations, validator verdicts, watcher evidence,
wallet confirmation evidence, restart, replay, tamper handling, and report
honesty.

This scenario is local-only. It must not claim live DA, public finality, node
deployment, operator recovery, or production incident closure.

### 🔎 13.2. Whitepaper Basis

- Rollup ordering, publication, verification, and role split:
  `docs/Z00Z-Main-Whitepaper.md`.
- Local failure handling and recovery boundary:
  `docs/Z00Z-Main-Whitepaper.md`.
- DA as publication layer and failure-mode taxonomy:
  `docs/Z00Z-Cross-Chain-Integration-Whitepaper.md`.
- Checkpoint, DA commitment, watcher evidence, and recovery-record separation:
  `docs/tech-papers/Z00Z-Multi-DA-and-Checkpoint-Architecture.md`.

### ⚙️ 13.3. Current Code To Study First

- Aggregator publication types: `crates/z00z_runtime/aggregators/src/types.rs`.
- Aggregator service traits and publication binding:
  `crates/z00z_runtime/aggregators/src/service.rs`.
- Local DA adapter seam: `crates/z00z_rollup_node/src/da.rs`.
- Validator checkpoint and verdict seams:
  `crates/z00z_runtime/validators/src/checkpoint.rs`,
  `crates/z00z_runtime/validators/src/engine.rs`,
  `crates/z00z_runtime/validators/src/verdict.rs`.
- Watcher input, evidence, provider signal, and alerts:
  `crates/z00z_runtime/watchers/src/engine.rs`,
  `crates/z00z_runtime/watchers/src/evidence_export.rs`,
  `crates/z00z_runtime/watchers/src/da_health.rs`,
  `crates/z00z_runtime/watchers/src/alerts.rs`.
- Rollup-node local status projection:
  `crates/z00z_rollup_node/src/status.rs`.
- Wallet confirmation evidence:
  `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_admission.rs`.
- Current Scenario 1 fixture producers, used only as source-shape references:
  `crates/z00z_simulator/src/scenario_1/stage_3/claim_pkg.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_6/`,
  `crates/z00z_simulator/src/scenario_1/stage_9/`,
  `crates/z00z_simulator/src/scenario_1/stage_10/`,
  `crates/z00z_simulator/src/scenario_1/stage_11/`,
  `crates/z00z_simulator/src/scenario_1/stage_12/`,
  `crates/z00z_simulator/src/scenario_1/stage_13/`.
- Current wallet tx RPC and confirmation-evidence bridge:
  `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_admission.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`.

### 🧭 13.4. Workspace And Stage Guards

- The workspace has no package named `z00z_runtime`; use package-level crates
  `z00z_aggregators`, `z00z_validators`, and `z00z_watchers` in commands and
  tests.
- `scenario_9` should be an independent target or test harness. Do not append
  runner stage 14 to `scenario_1` unless a later PLAN explicitly chooses that
  older integration style and updates the current runner contract.
- Current Scenario 1 runner stage 8 is `transfer_claim` at
  `crates/z00z_simulator/src/scenario_1/stage_8/`.
- Current checkpoint-finalization step IDs `S8-*` are implemented by runner
  stage 12, `checkpoint_finalize`, at
  `crates/z00z_simulator/src/scenario_1/stage_12/` with
  `scenario_config.yaml::stage8_paths`.
- Current runner stage 10 is `bundle_publish` at
  `crates/z00z_simulator/src/scenario_1/stage_10/`.
- Current runner stage 13 is `hjmt_settlement_examples` at
  `crates/z00z_simulator/src/scenario_1/stage_13/`; it is not a wallet tx RPC
  lifecycle stage.

### 🧭 13.5. Source-Shape Contract

Persisted local evidence records must state the current source shape that
actually produced them. Tests must fail if a report claims a source shape that
the run did not execute.

- `stage4_tx_package`: prepared wallet tx package from the current
  `tx_prepare` lane.
- `stage5_receive_or_claim`: transfer receive or explicit claim handoff from
  current runner stages 7 and 8.
- `stage6_checkpoint_bridge`: bundle bridge, fragments, exec input, and
  idempotency inputs from current runner stages 9 and 10.
- `stage7_checkpoint_apply`: storage-backed root application from current
  runner stage 11.
- `stage8_checkpoint_finalize`: sealed checkpoint artifact and link evidence
  from current runner stage 12.
- `stage13_hjmt_settlement_examples`: HJMT settlement proof examples from
  current runner stage 13 when those artifacts are used.
- `wallet_tx_rpc_evidence`: wallet tx package, admission, confirmation, and
  tx-history evidence from current wallet RPC/storage surfaces, not from
  Scenario 1 runner stage 13.
- `local_mock_da`: mock DA blob refs and provider signals.
- `local_publication_store`: restart, reload, replay, and persisted trace
  evidence.

### 📦 13.6. Trace Envelope Fields

Do not duplicate canonical runtime DTOs. If persistence needs a stable local
envelope, add a narrow trace shape such as `LocalPublicationTrace` or
`PersistPublicationTrace` with only scalar fields derived from current runtime,
wallet, checkpoint, and watcher values:

- `trace_version`;
- `source_shape`;
- `scenario_id`;
- `stage_id`;
- `batch_id_hex` or checkpoint draft ID equivalent;
- `intake_ids`;
- `tx_digest_hexes`;
- `checkpoint_id_hex`;
- `exec_input_id_hex`;
- `checkpoint_artifact_digest_hex`;
- `checkpoint_link_digest_hex` or explicit link ID fields;
- `provider_name`;
- `provider_namespace`;
- `provider_stage`;
- `provider_outcome`;
- `blob_ref`;
- `blob_digest_hex`;
- `publication_state`;
- `soft_confirmation_present`;
- `verdict_kind`;
- `reject_class`;
- `wallet_confirmation_evidence_present`;
- `evidence_sequence`;
- `created_at` from an injected `TimeProvider` or existing deterministic
  simulator timestamp source.

The envelope is a persisted trace and must not replace `WorkItem`,
`OrderedBatch`, `PublicationRequest`, `PublishedBatch`, `SoftConfirmation`,
`ResolvedBatch`, `Verdict`, `WatcherInput`, or `EvidenceRecord`.

### ⚙️ 13.7. Scenario Flow

1. Add an independent `scenario_9` simulator target or test harness; do not
   append runner stage 14 to `scenario_1`.
2. Create a bounded local publication trace envelope with `trace_version`,
   `source_shape`, batch, checkpoint, blob, verdict, provider, evidence, and
   wallet-confirmation fields.
3. Persist trace records and mock DA blobs under deterministic scenario output
   paths using `z00z_utils::io` and project codec helpers.
4. Convert existing wallet tx packages and claim packages into `WorkItem`
   values where current fixtures support them.
5. Build deterministic `OrderedBatch` and `PublicationRequest`.
6. Publish and resolve through a local mock `DaAdapter`.
7. Validate `ResolvedBatch` through `ValidatorBoundary` and current
   `CheckpointFlow`.
8. Convert publication, provider signal, verdict, and observation into watcher
   evidence records.
9. Project accepted verdict evidence into `TxConfirmationEvidence`.
10. Restart from persisted traces and prove idempotent replay versus
    conflicting duplicate evidence.
11. Inject tamper cases for package bytes, tx digest, namespace, blob ref, blob
    bytes, checkpoint artifact, proof payload, link ID, exec input ID, snapshot
    ID, claim root, roots, wallet history row, and storage replay contract.
12. Emit report fields that explicitly state `boundary_mode:
    local_publication`, `da_mode: local_mock`, `settlement_authority:
    local_checkpoint_artifact`, and `wallet_finality_source:
    tx_confirmation_evidence`.

### 🔑 13.8. Invariants

- `SoftConfirmation` is pre-final and never marks a wallet tx confirmed.
- `PublicationState::Posted`, `PublicationState::Seen`, and provider success
  are publication evidence only.
- Wallet confirmation requires accepted local checkpoint/verdict evidence.
- Local mock DA is deterministic and exposes its fault mode in output records.
- Reloaded evidence verifies digest, namespace, batch ID, checkpoint ID, and
  source shape before use.
- Replayed evidence is idempotent when exact and rejected or critical when
  conflicting.
- Scenario reports must not mention live DA closure, deployed node closure,
  production finality, public testnet closure, or operator recovery closure.

### 📦 13.9. Required Artifacts

- `scenario_9/publication/local_publication_trace.jsonl`
- `scenario_9/publication/local_publication_report.json`
- `scenario_9/publication/store/index.json`
- `scenario_9/publication/mock_da/blobs/`
- `scenario_9/publication/replay/restart_report.json`
- `scenario_9/publication/tamper/tamper_matrix.json`
- `scenario_9/publication/wallet_confirmation_report.json`
- `scenario_9/publication/report_honesty.json`

### ✅ 13.10. Tests

- canonical runtime publication types are imported through crate-root facades;
- no duplicate canonical `WorkItem`, `OrderedBatch`, `PublishedBatch`,
  `SoftConfirmation`, `ResolvedBatch`, `Verdict`, or `EvidenceRecord` is
  introduced in simulator-local modules;
- trace envelope roundtrip rejects unknown version, missing digest, wrong
  namespace, wrong checkpoint ID, and corrupt JSON;
- local mock DA `publish` is deterministic and `resolve` fails closed on blob,
  digest, namespace, or checkpoint drift;
- good resolved batch returns `VerdictKind::Accepted`;
- missing artifact, proof drift, replay conflict, provider mismatch, and root
  mismatch map into current `RejectClass` variants where represented;
- watcher happy path has no critical alerts and missing blob or provider
  divergence produces non-final evidence;
- soft confirmation alone keeps wallet tx pending;
- accepted local verdict writes `TxConfirmationEvidence`;
- rejected or incomplete verdict does not mutate wallet tx history;
- restart reload does not duplicate evidence records;
- every tamper case fails closed and preserves wallet/storage state before
  confirmation;
- golden report tests reject forbidden overclaim phrases.

### 🚩 13.11. Fault Matrix

| Fault | Injection Point | Expected Boundary | Expected Result |
| --- | --- | --- | --- |
| Missing blob | Local mock DA store | `DaAdapter::resolve` | recoverable resolve failure and missing-blob watcher alert |
| Wrong digest | Blob reload | validator or store reload | rejected verdict, no wallet confirmation |
| Wrong namespace | Blob metadata | provider validation | provider invalid or provider divergence evidence |
| Stale soft confirmation | Soft confirmation replay | wallet gate | tx remains pending, replay evidence recorded |
| Mismatched checkpoint link | stage 12 checkpoint-finalize link data | validator | rejected verdict with proof, shape, or reconcile class |
| Replayed evidence | Local publication store | store replay policy | idempotent if exact, rejected if conflicting |
| Tampered package bytes | current tx package or claim package source | wallet/package verifier | no asset mutation, no tx-history confirmation |
| Tampered proof payload | Checkpoint artifact | checkpoint verifier | proof rejection and critical evidence |
| Tampered root binding | storage replay contract | storage/root check | root mismatch rejection |
| Missing artifact | Published batch resolve | validator | incomplete or artifact-missing verdict |
| Provider failure | Mock provider signal | watcher | retry-pending or failed provider evidence |
| Corrupt persisted JSON | Local trace reload | store codec | typed load error, no partial state adoption |

### 💯 13.12. Scenario 9 PLAN Handoff Checklist

Every implementation PLAN for `scenario_9` must include:

- exact files to modify;
- existing type or trait each new module must reuse;
- independent `scenario_9` target or test-harness decision;
- source-shape names introduced or consumed;
- store path and codec decision using `z00z_utils::io` and project codec
  helpers;
- fault modes covered in that PLAN;
- wallet mutation gate for that PLAN;
- report honesty checks for that PLAN;
- targeted verification commands with current package and feature names;
- explicit statement that live DA and public finality remain out of scope.

### 🧪 13.13. Verification Anchors

Use targeted gates first:

```bash
cargo test -p z00z_aggregators --features test-params-fast
cargo test -p z00z_validators
cargo test -p z00z_watchers
cargo test -p z00z_rollup_node --features test-params-fast
cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_tools
```

If an implementation adds a new `scenario_9` binary or test target, add the
corresponding `Cargo.toml` target and replace the Scenario 1 command with the
new local publication runner command in the PLAN.

### ✅ 13.14. Completion Criteria

`scenario_9` is complete when local publication emits trace, mock DA, resolved
batch, verdict, provider signal, watcher evidence, and wallet confirmation
evidence where source artifacts support them; restart and tamper matrices fail
closed; reports name only local evidence truth; and no canonical runtime DTO or
checkpoint verifier has been duplicated in simulator code.

## ✅ 14. scenario_10 Local Whole-System Scenario

### 🎯 14.1. Purpose

Demonstrate Z00Z as a local integrated organism using only implemented surfaces. This is the capstone for the current codebase, not a testnet or Celestia scenario.

### 🔎 14.2. Whitepaper Basis

- Z00Z as private digital cash and asynchronous rights layer: `docs/Z00Z-Main-Whitepaper.md:75-173`.
- Use-case family selection: `docs/Z00Z-UseCases-Whitepaper.md:106-164`.
- Unique features: spendable capability objects, fee envelopes, offline machine/agent rights direction: `docs/Z00Z-Uniqueness-Whitepaper.md:194-291`.
- Implementation status and expansion path: `docs/Z00Z-Main-Whitepaper.md:1150-1288`.

### ⚙️ 14.3. Required Scenario Dependencies

`scenario_10` should be implemented only after these are stable:

- `scenario_2`: crypto/genesis/domain integrity;
- `scenario_3`: wallet receive/recovery/privacy;
- `scenario_4`: typed object policy matrix;
- `scenario_5`: storage/checkpoint/HJMT/fee replay;
- `scenario_6`: runtime route/recovery;
- `scenario_7`: validator/watcher evidence;
- `scenario_8`: local offline package/reconciliation boundary;
- `scenario_9`: local publication/evidence/restart/tamper boundary.
- `scenario_11`: shard quorum certificate, secondary replay, local DA binding,
  and validator certificate acceptance.

### ⚙️ 14.4. Scenario Flow

1. Create local genesis corpus.
2. Create wallets and restore one wallet from backup.
3. Execute one local cash transfer or package lifecycle.
4. Execute one voucher action and one right action.
5. Execute one fee-supported object action.
6. Apply settlement operations and produce storage proof/checkpoint artifacts.
7. Pass work through local runtime ingress and route planning.
8. Produce local publication trace and accepted or rejected evidence.
9. Produce validator verdict.
10. Produce watcher checked snapshot and evidence record.
11. Emit one public summary that states exactly which implemented surfaces were demonstrated.

### 🔑 14.5. Invariants

- Every subsystem contributes at least one artifact.
- No external DA/testnet/bridge claim is made.
- No external-blocked or not-current-code feature is treated as implemented.
- Every negative injection has a typed local reject.
- Public summary distinguishes implemented behavior from whitepaper target architecture.

### 📦 14.6. Required Artifacts

- `scenario_10/system_timeline.json`
- `scenario_10/subsystem_artifact_manifest.json`
- `scenario_10/cross_surface_invariants.json`
- `scenario_10/negative_injection_report.json`
- `scenario_10/watch_status_summary.json`
- `scenario_10/current_code_claims.md`

### ✅ 14.7. Tests

- dependency artifacts exist and match expected schema;
- local genesis/wallet/object/storage/runtime/validator/watcher path completes;
- injected object defect rejects;
- injected route defect rejects;
- injected checkpoint/proof defect rejects;
- summary contains no external DA/testnet/bridge/PQ/DAO/OnionNet claims.

### ✅ 14.8. Completion Criteria

`scenario_10` is complete when current implemented Z00Z surfaces operate together locally and the resulting summary is accurate about what is implemented now.

## ✅ 15. scenario_11 Shard Quorum Certificate And Secondary Replay Drill

### 🎯 15.1. Purpose

Demonstrate the aggregator consensus review as a new independent local scenario,
not as a new stage in `scenario_1`. The goal is to prove that a wallet-created
package can become a shard-routed commit subject, be independently replayed by
secondary aggregators, form a local 2-of-3 quorum certificate in
`sim_5a7s`, bind
that certificate through local DA publication, and be accepted or rejected by
the validator boundary using the same subject digest.

This scenario is local CFT quorum evidence. It must not claim network BFT,
Celestia finality, production signatures, slashing, or public settlement
finality.

### 🔎 15.2. Review Basis

- Aggregator consensus spec:
  `.planning/phases/000/067-Sharded-Concensus/Agg-Concensus-Spec.md`.
- Runtime route and recovery drill baseline: `scenario_6`.
- Local publication and tamper baseline: `scenario_9`.
- Current `sim_5a7s` topology: `config/hjmt_runtime/sim_5a7s`.

### ⚙️ 15.3. Current Code To Study First

- Aggregator quorum seam:
  `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`.
- Route planning and shard routing:
  `crates/z00z_runtime/aggregators/src/batch_planner.rs`.
- Placement and current `standby` naming debt:
  `crates/z00z_runtime/aggregators/src/placement.rs`.
- Ingress normalization:
  `crates/z00z_runtime/aggregators/src/ingress.rs`.
- Recovery and secondary catch-up behavior:
  `crates/z00z_runtime/aggregators/src/recovery.rs`,
  `crates/z00z_runtime/aggregators/src/dist_sim.rs`.
- Local DA publication and resolve:
  `crates/z00z_rollup_node/src/da.rs`.
- Validator checkpoint and verdict boundary:
  `crates/z00z_runtime/validators/src/checkpoint.rs`,
  `crates/z00z_runtime/validators/src/engine.rs`,
  `crates/z00z_runtime/validators/src/verdict.rs`.

### 🧭 15.4. Scenario Home

Do not modify `crates/z00z_simulator/src/scenario_1/` for this scenario. Add one
of these independent homes in the implementation PLAN:

- preferred: `crates/z00z_simulator/src/scenario_11/` with a dedicated
  `scenario_11` binary or integration test target;
- acceptable first slice: `crates/z00z_simulator/tests/scenario_11/` if the
  implementation is still a test-only harness;
- not allowed: `scenario_1` stage 14, `scenario_1` runner contract changes, or
  new `scenario_1` runtime-observability fields that imply quorum-certificate
  closure.

### ⚙️ 15.5. Required Existing-Crate Additions

- Add `CommitSubject` under `z00z_aggregators` with canonical encode and digest.
- Add `ShardVote` with voter id, role, shard id, term or epoch, membership
  digest, subject digest, and deterministic simulator signature.
- Add `ShardQuorumCertificate` with canonical sorted votes, quorum rule,
  membership digest, subject digest, and certificate digest.
- Add `SecondaryReplayVerifier` that recomputes the subject from live route,
  placement, recovery, publication, and theorem inputs before voting.
- Extend the current `ConsensusAdapter` path or add a sibling certificate path
  so `ConsensusCommit` cannot be treated as complete quorum proof by itself.
- Bind the quorum certificate digest or reference into local DA publication
  records without replacing `PublicationRequest`, `PublishedBatch`, or
  `ResolvedBatch`.
- Extend validator acceptance so a resolved batch can require and verify the
  certificate binding when the scenario enables the certificate gate.

### ⚙️ 15.6. Scenario Flow

1. Build a wallet-style package fixture that can be normalized by aggregator
   ingress.
2. Recompute the package digest at ingress and reject caller-provided digest
   drift.
3. Convert the package into a route-keyed work item.
4. Route it through the live shard route table.
5. Plan the batch with `BatchPlanner`.
6. Resolve placement from the real `sim_5a7s` table.
7. Build a primary `CommitSubject` from route, plan, placement, recovery,
   execution, publication, and theorem inputs.
8. Run `SecondaryReplayVerifier` for every secondary aggregator in
   that shard's quorum group.
9. Produce `ShardVote` only for aggregator voters that recompute the exact
   subject digest.
10. Form a 2-of-3 `ShardQuorumCertificate`.
11. Publish through `LocalDaAdapter` with certificate digest binding.
12. Resolve local DA and pass the resolved artifact to `ValidatorBoundary`.
13. Emit scenario evidence proving package digest, route digest, subject digest,
   vote set, certificate digest, DA digest, publication binding, and validator
   verdict all refer to the same commit subject.

### 🔑 15.7. Invariants

- Routing chooses the shard; the primary does not choose the shard.
- The quorum group is exactly primary plus secondary aggregators for
  one shard and one placement generation.
- A secondary vote means independent deterministic replay, not byte copying.
- A 2-of-3 local quorum is CFT evidence only and must not be called BFT.
- A certificate is invalid if any vote uses a different route digest, placement
  digest, subject digest, term, epoch, or membership digest.
- Local DA availability does not prove execution correctness; it only carries
  bytes and certificate-bound publication evidence.
- Validator acceptance must fail if the certificate is missing, detached,
  mismatched, stale, or formed from inactive members.

### 📦 15.8. Required Artifacts

- `scenario_11/quorum/package_ingress_report.json`
- `scenario_11/quorum/route_plan_report.json`
- `scenario_11/quorum/placement_membership.json`
- `scenario_11/quorum/commit_subject.json`
- `scenario_11/quorum/secondary_replay_votes.json`
- `scenario_11/quorum/quorum_certificate.json`
- `scenario_11/quorum/local_da_binding.json`
- `scenario_11/quorum/validator_verdict_report.json`
- `scenario_11/quorum/fault_matrix.json`
- `scenario_11/quorum/report_honesty.json`

### ✅ 15.9. Unit Tests

- `CommitSubject` canonical digest is stable across repeated encoding.
- `CommitSubject` digest changes when route table digest changes.
- `CommitSubject` digest changes when routing generation changes.
- `CommitSubject` digest changes when placement membership digest changes.
- `CommitSubject` digest changes when plan digest changes.
- `CommitSubject` digest changes when previous or new state root changes.
- `CommitSubject` digest changes when journal lineage or proof version changes.
- `ShardVote` rejects inactive voter ids.
- `ShardVote` rejects duplicate voter ids.
- `ShardVote` rejects wrong voter role for the placement generation.
- `ShardVote` rejects mismatched subject digest.
- `ShardQuorumCertificate` sorts votes canonically.
- `ShardQuorumCertificate` rejects duplicate signers.
- `ShardQuorumCertificate` rejects below-quorum vote sets.
- `ShardQuorumCertificate` rejects mixed term or mixed membership digest.
- `SecondaryReplayVerifier` accepts the exact primary subject.
- `SecondaryReplayVerifier` rejects wrong route, plan, root, lineage, proof,
  publication binding, and theorem digest.

### ✅ 15.10. Integration Tests

- `ConsensusAdapter` still preserves same-term split-brain freeze.
- Certificate-producing commit path returns the same CFT quorum decision as the
  existing local majority path for honest inputs.
- Certificate-producing commit path rejects removed aggregator votes.
- Certificate-producing commit path rejects newly joined but not-ready
  secondary votes.
- Stale secondary recovery state prevents vote creation.
- Secondary takeover after primary crash requires matching lineage and subject
  digest.
- Local DA publish stores the certificate digest or reference with the
  publication record.
- Local DA resolve rejects detached or mismatched certificate digest.
- `ValidatorBoundary` accepts a resolved batch only when certificate,
  publication binding, theorem bundle, and ordered batch share the same subject.
- `ValidatorBoundary` rejects missing certificate once certificate gate is
  enabled.

### ✅ 15.11. E2E Tests

- `scenario_11` happy path runs one shard with one primary and two
  secondary aggregators, then forms a 2-of-3 certificate and accepted
  validator verdict.
- `scenario_11` dual-primary owner path covers an aggregator that owns two
  shards in `sim_5a7s`, proving per-shard quorum isolation.
- `scenario_11` all-shard sweep proves each of the seven shards can resolve its
  primary plus secondary-aggregator membership and local quorum count.
- `scenario_11` primary-crash-before-quorum path produces no certificate and no
  DA publication.
- `scenario_11` primary-crash-after-quorum-before-DA path resumes publication
  only from the same subject and certificate.
- `scenario_11` one-secondary-offline path still forms 2-of-3 quorum.
- `scenario_11` one-secondary-stale path rejects the stale vote and either
  commits with the other secondary or fails closed if quorum is unavailable.
- `scenario_11` wrong route digest, placement generation, plan digest, state
  root, proof version, publication binding, theorem digest, and certificate
  digest each fail closed.
- `scenario_11` report-honesty test rejects phrases that claim network BFT,
  Celestia finality, production signatures, slashing, or public finality.

### 🚩 15.12. Fault Matrix

| Fault | Injection Point | Expected Boundary | Expected Result |
| --- | --- | --- | --- |
| Caller digest drift | Ingress package fixture | ingress normalization | recomputed digest wins or input rejects |
| Wrong route digest | `CommitSubject` builder | secondary replay | no secondary vote |
| Wrong placement generation | placement membership | vote validation | vote and certificate reject |
| Duplicate voter | certificate builder | quorum validation | certificate reject |
| Removed voter | active membership check | vote validation | vote reject |
| New unready voter | placement readiness | vote validation | vote reject |
| Stale secondary root | recovery state | secondary replay | no vote |
| Same-term conflicting subject | consensus commit | split-brain guard | freeze or reject |
| Primary crash before quorum | primary execution | certificate builder | no certificate, no publication |
| Primary crash after quorum before DA | publication resume | local DA binding | resume only exact certificate |
| Certificate digest drift | local DA record | resolve or validator | reject |
| Theorem digest drift | validator boundary | validator verdict | reject |
| Missing certificate | validator gate | validator verdict | incomplete or rejected |

### 🛑 15.13. Anti-Placeholder Gates

Every implementation PLAN for `scenario_11` must include tests that fail if:

- a certificate can be created without recomputing the secondary subject;
- secondary votes are hard-coded from expected fixtures;
- quorum is counted globally across five aggregators instead of per shard;
- DA binding uses a constant or zero certificate digest;
- validator acceptance ignores the certificate digest;
- report artifacts are emitted without corresponding live vote, certificate,
  DA, and validator evidence;
- any scenario summary claims BFT, Celestia, slashing, or production finality.

### 🧪 15.14. Verification Anchors

Use current package names only. These commands become mandatory after the listed
tests or targets are added by implementation:

```bash
cargo test -p z00z_aggregators --features test-params-fast --test test_shard_quorum_certificate -- --nocapture
cargo test -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture
cargo test -p z00z_rollup_node --features test-params-fast --test test_da_local_quorum_binding -- --nocapture
cargo test -p z00z_validators --test test_hjmt_publication_contract -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture
```

Until `scenario_11` exists as a target, do not replace current `scenario_1`
runner commands with it in release gates.

### ✅ 15.15. Completion Criteria

`scenario_11` is complete when a local package-to-validator path forms a real
per-shard quorum certificate from independently replayed primary and secondary
aggregator subjects, binds that certificate through local DA, rejects every drift
case above, emits evidence artifacts with no placeholder-only rows, and reports
only local CFT quorum claims.

## 🧭 16. Source Consolidation And Retirement Coverage

This file is the self-contained implementation backlog for the new scenarios.
Implementation PLAN files should not pre-read the retired 045, 053, or 054
files after this consolidation.

| Retired source | Execution-useful content that survives | Where it now lives |
| --- | --- | --- |
| `045-NEW-State-Spec.md` and `045-TODO.md` | storage-owned claim-source authority, honest checkpoint `claim_root`, shared checkpoint verifier, validator checkpoint consumer, unsupported receive taxonomy, scan evidence and runtime scan-status boundary, nullifier reserved-to-spent bridge, dependency order, anti-drift rules, and regression homes | `Existing-Crate Work Required Outside Simulator`, `scenario_3`, `scenario_5`, `scenario_7`, `scenario_8`, `scenario_10` |
| `053-TODO.md` | local publication trace, mock DA adapter, aggregator/validator/watcher evidence harnesses, workspace package guard, current stage/source-shape guard, trace envelope fields, wallet confirmation gate, restart/replay/tamper matrix, fault matrix, PLAN handoff checklist, report honesty, and local-only finality boundary | `Existing-Crate Work Required Outside Simulator`, `scenario_9`, `scenario_10` |
| `054-TODO.md` | checkpoint theorem-style local assertions, storage verifier reuse, checkpoint tamper fixtures, restart recovery, report cleanup, and optional recursive proof sidecar as non-authoritative evidence | `scenario_5`, `scenario_9`, `scenario_10` |

Current-code path replacements for drifted legacy references:

- use `crates/z00z_storage/src/backend/query.rs`,
  `crates/z00z_storage/src/backend/redb/validate.rs`,
  `crates/z00z_storage/src/backend/rows.rs`, and
  `crates/z00z_storage/src/backend/types.rs` instead of retired
  `crates/z00z_storage/src/assets/store_internal/...` paths;
- use `crates/z00z_runtime/validators/src/checkpoint.rs`,
  `crates/z00z_runtime/validators/src/verdict.rs`, and
  `crates/z00z_runtime/validators/src/engine.rs` instead of retired
  `checkpoint_flow.rs`, `verdicts.rs`, and `val_engine.rs` names;
- use `crates/z00z_runtime/aggregators/src/types.rs` and
  `crates/z00z_runtime/aggregators/src/service.rs` instead of retired
  `agg_types.rs` and `agg_iface.rs` names;
- use `crates/z00z_runtime/watchers/src/engine.rs`,
  `crates/z00z_runtime/watchers/src/status.rs`, and
  `crates/z00z_runtime/watchers/src/evidence_export.rs` instead of retired
  `watcher_engine.rs` wording;
- use `crates/z00z_rollup_node/src/da.rs` instead of retired
  `da_adapter.rs`;
- use `crates/z00z_wallets/src/chain/scan_engine.rs` and
  `crates/z00z_wallets/src/chain/scan_engine_impl.rs` instead of retired
  `crates/z00z_wallets/src/chain/scan/...` paths;
- use `crates/z00z_wallets/src/services/chain_service.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/chain_impl.rs`, and
  `crates/z00z_wallets/src/adapters/rpc/types/chain.rs` instead of retired
  `crates/z00z_wallets/src/services/runtime/chain_service.rs`;
- use `crates/z00z_simulator/src/scenario_1/stage_3/claim_pkg.rs` and
  `crates/z00z_simulator/src/scenario_1/claim_pkg_consumer.rs` instead of
  retired `stage_3_utils/claim_pkg.rs` and root-level
  `claim_pkg_consumer.rs` paths;
- use current source shape `stage13_hjmt_settlement_examples` for current
  Scenario 1 runner stage 13; use `wallet_tx_rpc_evidence` for wallet
  RPC/storage evidence instead of retired `stage13_wallet_tx_lifecycle`
  wording;
- use `crates/z00z_storage/tests/test_redb_reload.rs` instead of retired
  `test_redb_rehydrate.rs`;
- use `crates/z00z_simulator/tests/scenario_1/claim_pkg_crypto.rs` instead of
  retired `test_claim_pkg_crypto_support.rs`;
- use current feature names `test-params-fast` and `wallet_debug_tools`
  instead of retired `test-fast` and `wallet_debug_dump`.

## 🧹 17. Deletion Readiness Gate

The other consolidation inputs can be deleted without damaging implementation
completeness or understandability if this file remains available.

Deletion-safe coverage:

- `045-NEW-State-Spec.md`: covered by existing-crate work, dependency order,
  anti-drift rules, verification anchors, `scenario_3`, `scenario_5`,
  `scenario_8`, and `scenario_10`.
- `045-TODO.md`: covered by existing-crate dependency order, explicit crate
  seams, current regression homes, and current command anchors.
- `053-TODO.md`: covered by `scenario_9` purpose, source-shape contract,
  workspace/stage guards, trace envelope fields, fault matrix, PLAN handoff
  checklist, verification anchors, and `scenario_10` dependency composition.
- `054-TODO.md`: covered by `scenario_5` checkpoint theorem/tamper/restart
  scope, `scenario_9` publication-ref tamper coverage, and `scenario_10`
  integrated local evidence.

Before deleting the retired files, run a repo text check and update any PLAN or
handoff file that still points implementers at the retired paths as execution
sources. Historical references are acceptable only when they clearly point back
to this `063-TODO.md` as the maintained source of truth.

## 🚫 18. Removed Or Deferred From Current-Code Backlog

These ideas are intentionally not scenario targets right now:

- Live Celestia or multi-DA publication failover.
- Live external-chain lockers, bridge flows, issuer-native rails, or relayers.
- Real testnet operations.
- OnionNet transport routing/backpressure.
- Post-quantum migration/rewrite.
- DAO treasury, AI governance, and useful-work production coordination.
- Official DEX, bridge, launchpad, issuer, ramp, marketplace, or liquidity scenario.
- Broad corporate payroll or legal/compliance workflows that require disclosure products not implemented yet.
- Humanitarian/aid product flows as separate scenarios; current implementable coverage is the typed voucher/right object matrix in `scenario_4`.
- Agent/machine economies as separate scenarios; current implementable coverage is the typed right classes and policy matrix in `scenario_4`.

Reason: these may be whitepaper-valid, but they require new product surfaces, external services, or design contracts that are not present enough in the current codebase for an executable scenario.

Local mock DA is not deferred. It is in scope for `scenario_9` publication
evidence and `scenario_11` quorum-certificate binding when it stays behind the
current `DaAdapter` trait and reports itself as local mock publication evidence,
not as live DA finality.

## 🧭 19. Recommended Build Order

1. `scenario_2 Genesis, Crypto, And Domain Integrity Drill`
2. `scenario_5 Settlement, Checkpoint, HJMT, And Fee Replay Drill`
3. `scenario_4 Typed Object Policy Matrix`
4. `scenario_3 Wallet Receive, Recovery, And Privacy Drill`
5. `scenario_6 Runtime Route, Aggregator Churn, And Recovery Drill`
6. `scenario_7 Validator And Watcher Evidence Drill`
7. `scenario_8 Offline Package And Local Reconciliation Drill`
8. `scenario_9 Local Publication, Evidence, Restart, And Tamper Drill`
9. `scenario_11 Shard Quorum Certificate And Secondary Replay Drill`
10. `scenario_10 Local Whole-System Scenario`

This order builds the base proof and state machinery first, then object/wallet behavior, then runtime/watcher behavior, then local publication evidence, then shard quorum-certificate evidence, then local integration.

## 💯 20. Coverage Summary

The reduced backlog still demonstrates the main current capabilities:

- cryptographic maturity: `scenario_2`;
- storage and checkpoint integrity: `scenario_5`;
- typed asset/voucher/right model: `scenario_4`;
- wallet privacy and recovery: `scenario_3`;
- local runtime resilience: `scenario_6`;
- validator/watcher evidence: `scenario_7`;
- offline package boundary: `scenario_8`;
- local publication/restart/tamper evidence: `scenario_9`;
- local shard quorum certificate and secondary replay evidence: `scenario_11`;
- whole local system behavior: `scenario_10`.

The removed scenarios are not rejected as product ideas. They are deferred until the codebase has the required local implementation surfaces.



---

---



## 21. Legacy Closeout 19: Local Publication, Simulator Evidence, And Restart/Tamper Harness

**Goal:**

- Keep publication evidence local, deterministic, and route-bound without upgrading the external DA adapter boundary into live authority.

**Closeout note:**

- `Local publication` means runtime-owned publication binding plus storage-owned route snapshot plus simulator-owned evidence artifacts.
- External DA transport remains adapter-only. The live claim stops at local publication and local or mock DA behavior.
- Evidence anchors:
  - `crates/z00z_runtime/aggregators/README.md`
  - `crates/z00z_runtime/watchers/src/publication.rs`
  - `crates/z00z_rollup_node/src/da.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`
  - `crates/z00z_simulator/README.md`
  - `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`

**Closeout table:**

| Criterion                                                    | Evidence anchors                                             | Status   |
| ------------------------------------------------------------ | ------------------------------------------------------------ | -------- |
| local publication stays local or mock DA only                | `crates/z00z_rollup_node/src/da.rs`; `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`; `crates/z00z_simulator/README.md` | `Closed` |
| publication digest stays identical across leaf, proof, validator, watcher, and publication observers | `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`; `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs` | `Closed` |
| watcher and publication route binding reuse runtime and storage primitives | `crates/z00z_runtime/watchers/src/publication.rs`; `crates/z00z_runtime/aggregators/README.md`; `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs` | `Closed` |
| restart and tamper evidence does not invent a second publication truth plane | `crates/z00z_runtime/watchers/src/publication.rs`; `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`; `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs` | `Closed` |

## 22. Legacy Closeout 20: Simulator Checkpoint, Theorem, Tamper, And Restart Evidence Pack

**Goal:**

- Keep checkpoint evidence on stage-owned storage and checkpoint artifacts and on fail-closed reload paths.

**Closeout table:**

| Evidence row                     | Evidence anchors                                             | Status   |
| -------------------------------- | ------------------------------------------------------------ | -------- |
| accepted checkpoint              | `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs` | `Closed` |
| rejected tampered exec payload   | `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs` | `Closed` |
| rejected tampered proof payload  | `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs` | `Closed` |
| restart or reload evidence       | `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`; `crates/z00z_storage/src/checkpoint/store.rs` | `Closed` |
| stable artifact and report names | `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs` with `transactions/checkpoint_s7.json`, `transactions/checkpoint_s8.json`, `pub_flow.json`, and `watch_flow.json` | `Closed` |
| typed error redaction            | `crates/z00z_storage/src/error.rs`; `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`; `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs` | `Closed` |

## 23. Legacy Closeout 21: Simulator Receive, Import, And History Evidence Pack

**Goal:**

- Keep simulator receive/import/history evidence downstream of the live wallet,
  tx-history, and publication primitives.
- Prove restart, duplicate, conflict, and negative package behavior without
  leaking secrets or creating a second wallet truth plane.

**Closeout note:**

- `scenario_1` `hist_flow.json` now carries `wallet_scan_digest_hex` plus a
  `wallet_lifecycle_rows` matrix bound to the same publication digest and
  tx-history digest packet used by the rest of Stage 13 evidence.
- Wallet lifecycle simulation is path-invariant: deterministic seeds now derive
  from wallet-scan and publication digests instead of absolute output paths.
- Restart proof is live for every required row. The simulator reopens wallet
  state, replays tx-history JSONL, re-reads owned asset rows, and compares the
  reopened lifecycle projection against the pre-restart evidence packet.
- Promoted shared Stage 13 roots are revalidated and rebuilt on drift, so stale
  cache markers cannot silently preserve obsolete evidence contracts.

**Evidence anchors:**

- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13/shared_cases.rs`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
- `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`

**Closeout table:**

| Simulator row                 | Expected lifecycle | Expected coarse status | Expected typed error code   | Evidence anchors                                             | Status   |
| ----------------------------- | ------------------ | ---------------------- | --------------------------- | ------------------------------------------------------------ | -------- |
| `submitted`                   | `Submitted`        | `pending`              | none                        | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_runtime_state.rs` | `Closed` |
| `admitted`                    | `Admitted`         | `pending`              | none                        | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_runtime_state.rs` | `Closed` |
| `imported`                    | `Imported`         | `pending`              | none                        | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/tests/test_direct_tx_receive.rs` | `Closed` |
| `confirmed`                   | `Confirmed`        | `confirmed`            | none                        | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs` | `Closed` |
| `duplicate_import`            | `Imported`         | `pending`              | none                        | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs` | `Closed` |
| `conflicted`                  | `Conflicted`       | `rejected`             | `DuplicateConflict`         | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs` | `Closed` |
| `already_spent`               | `AlreadySpent`     | `rejected`             | `AlreadySpent`              | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs` | `Closed` |
| `no_owned_output`             | `Exported`         | `rejected`             | `NoOwnedOutputs`            | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs` | `Closed` |
| `wrong_chain`                 | `Failed`           | `rejected`             | `WrongChain`                | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`; `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs` | `Closed` |
| `invalid_digest`              | `Failed`           | `rejected`             | `InvalidDigest`             | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`; `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs` | `Closed` |
| `unsupported_package_version` | `Failed`           | `rejected`             | `UnsupportedPackageVersion` | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`; `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs` | `Closed` |
