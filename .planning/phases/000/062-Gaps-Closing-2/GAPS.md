---
goal: Close ready and near-ready gaps extracted from Z00Z-IMPL-PHASES.md
version: 2.0
date_created: 2026-06-21
last_updated: 2026-06-21
status: Planned
source: .planning/phases/Z00Z-IMPL-PHASES.md
---

# Gaps

This document is the execution plan for every `👍` section in `.planning/phases/Z00Z-IMPL-PHASES.md`.
It is self-contained: an implementer must be able to start here, follow task numbers in order, change the named files, add the named tests, run the named commands, and know when the gap is closed.

## 📌 Execution Contract

Follow these rules exactly.

1. Execute tasks in numeric order: `TASK-001`, then `TASK-002`, and so on.
2. Do not skip a task unless its `Skip condition` is satisfied and recorded in the task's completion note.
3. Do not close a source phase until every task mapped to that source phase has passed its validation commands.
4. Keep source-code, tests, comments, commit text, and documentation in English.
5. Use only repository source, tests, and docs as evidence. Do not use `.planning/graphs` as evidence.
6. Preserve current public behavior unless a task explicitly says to add a backward-compatible field or enum value.
7. Prefer additive compatibility for RPC DTOs: add fields and keep old fields unless a task explicitly says to remove one.
8. Keep `TxStatus` storage compatibility unless a task explicitly changes the storage wire format.
9. When a task says "add a test", add a test that fails before the implementation and passes after it.
10. After each completed task, add a short completion note in that task with date, commit or local diff reference, and validation result.

## 🎯 Global Goal

Close the implementable and near-implementable work from these source sections:

- `0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout`
- `9. Storage Claim-Root And Checkpoint Authority Closure`
- `10. Wallet Receive, Scan, Import, And History Authority Closure`
- `11. Field-Native Pack Migration Plan`
- `12. Privacy, Stealth, And Selective Disclosure Primitives`
- `13. Unsupported Receive-Version Taxonomy`
- `14. Wallet Scan Orchestration And Runtime Scan Status`
- `15. Offline TxPackage Verify, Report, And Import Hardening`
- `16. Tx-History And wallet.asset.* Authority Convergence`
- `17. Package Hygiene And Transport Privacy Plan`
- `18. Request-Bound Inbox Helper Plan`
- `19. Local Publication, Simulator Evidence, And Restart/Tamper Harness`
- `20. Simulator Checkpoint, Theorem, Tamper, And Restart Evidence Pack`
- `21. Simulator Receive, Import, And History Evidence Pack`
- `22. Benchmark, Proof-Size, And Evidence Guardrails`
- `27. Optional Proof-Size And Storage Measurement Sidecar`
- `28. Multi-Asset Families, Trust Tiers, And Internal Asset Phase`
- `29. Local Adapter Model For Cross-Chain Inputs Without Live Chains`
- `30. Voucher, Payroll, B2B, And Useful-Work Claim Scenarios`
- `32. Fee Envelope And Rights Wallet Extensions`
- `33. Agentic Rights Local Simulations`
- `34. Machine Capability Local Simulations`
- `36. Spec-Gap Normalization And Residual Hardening Gate`

## 🚫 Out Of Scope For This TODO

Do not implement or claim closure for these future/research areas in this file:

- Recursive proofs from source phases `23`, `24`, `25`, and `26`.
- Linked liability MVP from source phase `31`.
- OnionNet deterministic control plane from source phase `35`.
- Live external DA, live chain bridge, or live transport anonymity.

If an implementation task touches one of these areas, split that work into a separate future phase before continuing.

## ⚙️ Ordered Execution Map

| Order | Source Section | Purpose | Must Finish Before |
|---:|---|---|---|
| 001-004 | `0` | Normalize storage-root truth and HJMT closure boundary. | Claim-root/checkpoint closeout and final spec normalization. |
| 005-006 | `9` | Close claim-root and checkpoint authority. | Simulator checkpoint evidence closure. |
| 007-013 | `19`, `20`, `22`, `27` | Close local publication, checkpoint evidence, proof-size, and benchmark guardrails. | Wallet/simulator receive evidence closure. |
| 014-025 | `10`, `13`, `15`, `16` | Add tx lifecycle, tx-history, package import, and receive-version taxonomy foundations. | Scan status, request inbox, simulator wallet evidence. |
| 026-039 | `10`, `13`, `14`, `15`, `16`, `18`, `21` | Finish scan orchestration, TxPackage import, request inbox helper, and simulator receive/import evidence. | Final wallet closure. |
| 040-048 | `11`, `12`, `17` | Finish field-pack boundary, selective disclosure, and package hygiene. | Final privacy/package closure. |
| 049-062 | `28`, `29`, `30`, `32`, `33`, `34` | Finish bounded object, right, voucher, fee, adapter, agentic, and machine simulations. | Final expansion closure. |
| 063-070 | `36` | Normalize residual gaps, source docs, guardrails, and full verification. | Final completion. |

## ✅ Definition Of Done

The entire TODO is complete only when all conditions below are true.

- Every task from `TASK-001` through `TASK-070` is marked complete with date and validation evidence.
- Every source section listed above has a closeout note in `.planning/phases/Z00Z-IMPL-PHASES.md`.
- No closeout note claims that `AssetStateRoot` is the live public root.
- No closeout note claims recursive proof, linked liability, OnionNet, live bridge, or live transport anonymity is implemented.
- All tests listed under `Final Verification Commands` pass, or each failure is documented with a linked blocker task.

## 💯 Final Verification Commands

Run these commands after the last task and before closing Phase 36.

```bash
cargo test -p z00z_storage --test test_hjmt_backend_conformance
cargo test -p z00z_storage --test test_live_guardrails
cargo test -p z00z_storage --test test_hjmt_compat_equivalence
cargo test -p z00z_storage --test test_claim_source_proof
cargo test -p z00z_storage --test test_checkpoint_root_binding
cargo test -p z00z_storage --test test_checkpoint_finalization
cargo test -p z00z_storage --test test_bench_lanes
cargo test -p z00z_wallets --test test_direct_tx_receive
cargo test -p z00z_wallets --test test_tx_store_integration
cargo test -p z00z_wallets --test test_asset_import_security
cargo test -p z00z_wallets --test test_asset_replay_protection
cargo test -p z00z_wallets --test test_import_error_taxonomy
cargo test -p z00z_wallets --test test_stealth_request
cargo test -p z00z_wallets --test test_stealth_scanner_flow
cargo test -p z00z_wallets --test test_stealth_scanner_cache
cargo test -p z00z_wallets --test test_e2e_req_flow
cargo test -p z00z_wallets --test test_zkpack
cargo test -p z00z_wallets --test test_asset_pack_v2_memo
cargo test -p z00z_wallets --test test_golden_tag16
cargo test -p z00z_wallets --test test_e2e_tag_auth
cargo test -p z00z_wallets --test test_stealth_scanner_prefilter
cargo test -p z00z_wallets --test test_view_key_contract
cargo test -p z00z_wallets --test test_rpc_logging_risk_policy
cargo test -p z00z_simulator --test test_checkpoint_acceptance
cargo test -p z00z_simulator --test test_hjmt_e2e
cargo test -p z00z_simulator --test test_scenario1_stage_surface
cargo test -p z00z_rollup_node --test test_hjmt_node_lifecycle
cargo test -p z00z_validators --test test_hjmt_publication_contract
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_watchers --test test_hjmt_publication_contract
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all
cargo doc --no-deps
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh
```

## 📌 Task Plan

### TASK-001 - 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout

Depends on: none.

Goal: make the source phase reflect the current storage truth.

Change:
- Edit `.planning/phases/Z00Z-IMPL-PHASES.md` section `👍 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout`.
- Replace any closure wording that treats `AssetStateRoot` or `AssetPath` as the live public runtime root.
- State explicitly that the live public semantic root is `SettlementStateRoot`.
- State explicitly that storage paths exposed above the backend are `SettlementPath` or settlement proof paths, not asset-only paths.
- State explicitly that HJMT is the live backend and old forest/asset-only terminology is historical or superseded.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_storage/src/settlement/root_types.md`

Tests:
- No new test is required in this task.

Verify:
```bash
rg -n "AssetStateRoot|AssetPath" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage/src/settlement/root_types.md
cargo test -p z00z_storage --test test_hjmt_backend_conformance
```

Done when:
- The grep output contains no live-root claim for `AssetStateRoot` or `AssetPath`.
- Section `0` says it is closed by current `SettlementStateRoot` + HJMT implementation, not by a future forest rollout.

### TASK-002 - 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout

Depends on: `TASK-001`.

Goal: prove backend configuration is fail-closed and not ambiguous.

Change:
- Confirm `crates/z00z_storage/src/settlement/hjmt_config.rs` accepts only unset backend mode or `Z00Z_STORAGE_BACKEND=hjmt`.
- If a non-HJMT value is accepted, change startup validation to reject it.
- Keep the rejection message redacted and deterministic.

Files:
- `crates/z00z_storage/src/settlement/hjmt_config.rs`
- `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`

Tests:
- Add or update a test that sets `Z00Z_STORAGE_BACKEND=forest`, `redb`, or any non-`hjmt` value and expects startup validation to fail.
- Add or update a test that confirms unset backend mode and `hjmt` both pass.

Verify:
```bash
cargo test -p z00z_storage --test test_hjmt_backend_conformance
```

Done when:
- Invalid backend mode fails before runtime storage opens.
- The phase closeout can cite a passing test for fail-closed backend selection.

### TASK-003 - 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout

Depends on: `TASK-002`.

Goal: prove there is no second semantic root authority above the backend.

Change:
- Confirm `crates/z00z_storage/src/backend/mod.rs` remains a low-level backend trait boundary.
- Confirm `crates/z00z_storage/src/settlement/store.rs` remains the semantic facade through `SettlementTreeBackend`.
- Add a guardrail test if current tests do not assert that public code imports the semantic facade instead of backend root types.

Files:
- `crates/z00z_storage/src/backend/mod.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`

Tests:
- Add a guardrail test that scans public downstream files and fails if they expose backend root authority as a public semantic root.
- The test may use a static source scan if that pattern already exists in `test_live_guardrails.rs`.

Verify:
```bash
cargo test -p z00z_storage --test test_live_guardrails
```

Done when:
- Downstream code cannot accidentally treat `backend_root` as the public state root without failing a guardrail test.

### TASK-004 - 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout

Depends on: `TASK-003`.

Goal: finish Phase 0 closeout evidence.

Change:
- Add a closeout note under source section `0` that lists these evidence anchors:
  - `crates/z00z_storage/src/settlement/root_types.md`
  - `crates/z00z_storage/src/settlement/store.rs`
  - `crates/z00z_storage/src/settlement/hjmt_config.rs`
  - `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`
  - `crates/z00z_storage/tests/test_live_guardrails.rs`
- State "Closed as normalized/superseded by live settlement root and HJMT backend."

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test is required in this task.

Verify:
```bash
rg -n "Closed as normalized/superseded|SettlementStateRoot|HJMT" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 0 has an explicit closeout note with local evidence paths and no future-only backend wording.

### TASK-005 - 9. Storage Claim-Root And Checkpoint Authority Closure

Depends on: `TASK-004`.

Goal: map claim-root and checkpoint authority done criteria to concrete code.

Change:
- Edit source section `👍 9. Storage Claim-Root And Checkpoint Authority Closure`.
- Add a closeout table with one row per criterion:
  - claim source root is bound into proof material,
  - checkpoint statement includes claim root,
  - checkpoint proof is statement-bound,
  - checkpoint store seals through one canonical path,
  - checkpoint link reload revalidates artifact identity,
  - tampered claim-root or checkpoint payload is rejected.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_storage/src/checkpoint/artifact_stmt.rs`
- `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
- `crates/z00z_storage/src/checkpoint/store.rs`

Tests:
- No new test is required if `TASK-006` confirms coverage exists.

Verify:
```bash
rg -n "claim source root|CheckpointStmt|seal_artifact|load_link|tamper" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage/src/checkpoint
```

Done when:
- A reader can trace every Phase 9 done bullet to a local file path without reading graph output.

### TASK-006 - 9. Storage Claim-Root And Checkpoint Authority Closure

Depends on: `TASK-005`.

Goal: prove checkpoint reload and tamper rejection after persistence.

Change:
- Inspect `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`.
- If it already seals, reloads, and rejects tampered checkpoint or claim-root evidence after reload, record that evidence in the closeout note.
- If it does not, add a new test named `test_checkpoint_claim_root_tamper_rejected_after_reload`.
- The new test must:
  - create a checkpoint artifact with a claim root,
  - seal it through the canonical checkpoint store path,
  - drop or reopen the store,
  - load the checkpoint link,
  - mutate claim-root or proof payload bytes,
  - assert reload or validation fails with a typed error,
  - assert no new checkpoint link is accepted.

Files:
- `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
- `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`

Tests:
- Add the missing test only if existing tests do not cover the reload/tamper case.

Verify:
```bash
cargo test -p z00z_storage --test test_claim_source_proof
cargo test -p z00z_storage --test test_checkpoint_root_binding
cargo test -p z00z_storage --test test_checkpoint_finalization
cargo test -p z00z_simulator --test test_checkpoint_acceptance
```

Done when:
- Phase 9 has test evidence for claim-root binding, checkpoint sealing, reload, and tamper rejection.

### TASK-007 - 19. Local Publication, Simulator Evidence, And Restart/Tamper Harness

Depends on: `TASK-006`.

Goal: close local publication evidence without claiming live DA readiness.

Change:
- Edit source section `👍 19. Local Publication, Simulator Evidence, And Restart/Tamper Harness`.
- Add a closeout note that says local publication is a deterministic local artifact and local/mock DA boundary only.
- Link these evidence anchors:
  - `crates/z00z_simulator/README.md`
  - `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`
  - `crates/z00z_rollup_node/src/da.rs`
  - `crates/z00z_runtime/aggregators/README.md`

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test is required unless `TASK-008` fails.

Verify:
```bash
cargo test -p z00z_simulator --test test_hjmt_e2e
cargo test -p z00z_rollup_node --test test_hjmt_node_lifecycle
```

Done when:
- The closeout says "local publication" and does not say "live external DA".

### TASK-008 - 19. Local Publication, Simulator Evidence, And Restart/Tamper Harness

Depends on: `TASK-007`.

Goal: prove publication digest consistency.

Change:
- Confirm simulator tests compare publication digest across leaf/proof/validator/watcher/publication surfaces.
- If missing, add a simulator test named `test_local_publication_digest_consistent_across_observers`.
- The test must compute or read the digest from every observer and assert exact equality.

Files:
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`
- `crates/z00z_runtime/watchers/src/publication.rs`
- `crates/z00z_rollup_node/src/da.rs`

Tests:
- Add the simulator test only if digest consistency is not already covered.

Verify:
```bash
cargo test -p z00z_simulator --test test_hjmt_e2e
```

Done when:
- A tampered or mismatched publication digest cannot pass local simulator evidence checks.

### TASK-009 - 20. Simulator Checkpoint, Theorem, Tamper, And Restart Evidence Pack

Depends on: `TASK-008`.

Goal: close simulator checkpoint evidence pack.

Change:
- Edit source section `👍 20. Simulator Checkpoint, Theorem, Tamper, And Restart Evidence Pack`.
- Add a closeout table for:
  - accepted checkpoint,
  - rejected tampered exec payload,
  - rejected tampered proof payload,
  - restart or reload evidence,
  - stable artifact/report names,
  - typed error redaction.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`

Tests:
- No new test is required if both simulator tests pass and expose the listed rows.

Verify:
```bash
cargo test -p z00z_simulator --test test_checkpoint_acceptance
cargo test -p z00z_simulator --test test_scenario1_stage_surface
```

Done when:
- Phase 20 closeout names all accepted and rejected evidence classes.

### TASK-010 - 22. Benchmark, Proof-Size, And Evidence Guardrails

Depends on: `TASK-009`.

Goal: close benchmark and proof-size guardrails without misleading throughput claims.

Change:
- Edit source section `👍 22. Benchmark, Proof-Size, And Evidence Guardrails`.
- Add a rule that only `durable_root_published_tps` may support user-facing throughput claims.
- State that worker-local TPS, cache-only throughput, compression win, and synthetic proof-size numbers are not user-facing claims.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_storage/benches/settlement_benches.md`

Tests:
- Confirm `crates/z00z_storage/tests/test_bench_lanes.rs` checks accepted and rejected lanes.
- Add a missing assertion if any forbidden lane can be labeled user-facing.

Verify:
```bash
cargo test -p z00z_storage --test test_bench_lanes
```

Done when:
- Benchmark docs and tests agree on the single accepted throughput lane.

### TASK-011 - 22. Benchmark, Proof-Size, And Evidence Guardrails

Depends on: `TASK-010`.

Goal: prove proof-size evidence fields exist and are nonzero.

Change:
- Confirm `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs` validates proof bytes, path counts, verify time, replay result, cache result, and tamper result.
- If any field is missing, add a Stage13 assertion for it.
- A missing, zero, or inconsistent proof-size field must fail the test.

Files:
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`

Tests:
- Extend existing Stage13 test rather than creating a separate simulator harness unless the file is already too large for local style.

Verify:
```bash
cargo test -p z00z_simulator --test test_scenario1_stage_surface
```

Done when:
- Stage13 cannot pass with empty proof bytes, zero verify time, missing path count, or unverified tamper result.

### TASK-012 - 27. Optional Proof-Size And Storage Measurement Sidecar

Depends on: `TASK-011`.

Goal: decide the optional sidecar boundary and remove ambiguity.

Change:
- Edit source section `👍 27. Optional Proof-Size And Storage Measurement Sidecar`.
- Choose exactly one closure mode:
  - `Closed by Stage13 evidence`: no standalone sidecar file is required.
  - `Standalone sidecar required`: create a stable sidecar report.
- Do not leave both modes active.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_storage/benches/settlement_benches.md`

Tests:
- If `Closed by Stage13 evidence`, rely on `test_scenario1_stage_surface`.
- If `Standalone sidecar required`, implement `TASK-013`.

Verify:
```bash
rg -n "Closed by Stage13 evidence|Standalone sidecar required" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- The phase has exactly one selected closure mode.

### TASK-013 - 27. Optional Proof-Size And Storage Measurement Sidecar

Depends on: `TASK-012`.

Skip condition: skip this task only if `TASK-012` selected `Closed by Stage13 evidence`.

Goal: implement a standalone proof-size sidecar when required.

Change:
- Add a sidecar artifact writer in the simulator Stage13 path or storage bench support.
- The sidecar schema must include:
  - `schema_version`,
  - `root_generation`,
  - `root_hex`,
  - `proof_kind`,
  - `proof_bytes`,
  - `path_count`,
  - `verify_time_micros`,
  - `replay_verified`,
  - `cache_verified`,
  - `tamper_verified`.
- Use deterministic ordering for rows.

Files:
- `crates/z00z_simulator/src/scenario_1/`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`

Tests:
- Add assertions that reject missing fields, zero proof bytes, zero path count for non-empty proofs, and false verification booleans.

Verify:
```bash
cargo test -p z00z_simulator --test test_scenario1_stage_surface
```

Done when:
- The sidecar is deterministic and validated by simulator tests.

### TASK-014 - 10. Wallet Receive, Scan, Import, And History Authority Closure

Depends on: `TASK-013` or `TASK-012` skip condition.

Goal: define public wallet transaction lifecycle vocabulary.

Change:
- In `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`, add:
  - `RuntimeTxLifecycle`,
  - `RuntimeTxErrorCode`.
- `RuntimeTxLifecycle` must use `#[serde(rename_all = "snake_case")]` and include:
  - `Created`,
  - `Imported`,
  - `Exported`,
  - `Submitted`,
  - `Admitted`,
  - `Confirmed`,
  - `Failed`,
  - `Cancelled`,
  - `Conflicted`,
  - `AlreadySpent`.
- `RuntimeTxErrorCode` must use `#[serde(rename_all = "snake_case")]` and include:
  - `InvalidEncoding`,
  - `InvalidPackage`,
  - `InvalidDigest`,
  - `UnsupportedPackageVersion`,
  - `UnsupportedReceiveVersion`,
  - `WrongChain`,
  - `InvalidPublicSpendProof`,
  - `NoOwnedOutputs`,
  - `NotImportReady`,
  - `DuplicateConflict`,
  - `AlreadySpent`,
  - `CursorConflict`,
  - `WorkerEvidenceRejected`,
  - `InternalError`.

Files:
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`

Tests:
- Add serialization tests in the same file for both enums.
- Assert JSON strings are snake_case.

Verify:
```bash
cargo test -p z00z_wallets tx::tests
```

Done when:
- The RPC layer has stable typed vocabulary without removing existing `TxStatus`.

### TASK-015 - 16. Tx-History And wallet.asset.* Authority Convergence

Depends on: `TASK-014`.

Goal: preserve physical tx-history event kinds needed for lifecycle projection.

Change:
- In `crates/z00z_wallets/src/backup/backup_wire.rs`, extend `WalletTxHistoryEntryKind` with:
  - `Conflicted`,
  - `AlreadySpent`.
- Keep existing entry variants unchanged.
- Add validation compatibility so old JSONL rows without these variants still decode.
- Do not rename existing variants.

Files:
- `crates/z00z_wallets/src/backup/backup_wire.rs`

Tests:
- Add tests that encode and decode each new entry kind.
- Add a fold test showing that a later `Conflicted` or `AlreadySpent` row overrides an earlier `Pending` folded view.

Verify:
```bash
cargo test -p z00z_wallets backup_wire
```

Done when:
- The append-only tx-history journal can represent conflict and already-spent outcomes without breaking old rows.

### TASK-016 - 16. Tx-History And wallet.asset.* Authority Convergence

Depends on: `TASK-015`.

Goal: expose tx-history rows to RPC projection without breaking `TxStorage`.

Change:
- In `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`, add trait methods:
  - `fn list_history_rows(&self) -> TxStorageResult<Vec<WalletTxHistoryJsonlEntry>>;`
  - `fn record_conflicted(&mut self, tx_hash: &str) -> TxStorageResult<()>;`
  - `fn record_already_spent(&mut self, tx_hash: &str) -> TxStorageResult<()>;`
- Import `WalletTxHistoryJsonlEntry` from the wallet backup module.
- In `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`, implement all three methods.
- `record_conflicted` and `record_already_spent` must set `TxRecord.status = TxStatus::Failed` while appending the richer event kind.
- Update in-test mocks that implement `TxStorage`.

Files:
- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`

Tests:
- Add unit tests for:
  - `list_history_rows` returns physical rows in sequence order,
  - `record_conflicted` appends `Conflicted`,
  - `record_already_spent` appends `AlreadySpent`,
  - folded storage status remains `Failed` for both richer lifecycle outcomes.

Verify:
```bash
cargo test -p z00z_wallets tx_storage_impl
```

Done when:
- RPC code can inspect latest physical history event kind while old coarse storage status remains compatible.

### TASK-017 - 16. Tx-History And wallet.asset.* Authority Convergence

Depends on: `TASK-016`.

Goal: add lifecycle projection helper.

Change:
- In `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`, add:
  - `latest_tx_history_kind(rows: &[WalletTxHistoryJsonlEntry], tx_hash: &str) -> Option<WalletTxHistoryEntryKind>`,
  - `tx_lifecycle_from_record(record: &TxRecord, latest_kind: Option<WalletTxHistoryEntryKind>) -> RuntimeTxLifecycle`.
- Mapping rules:
  - latest `Created` -> `Created`,
  - latest `Imported` -> `Imported`,
  - latest `Exported` -> `Exported`,
  - latest `Submitted` -> `Submitted`,
  - latest `Admitted` -> `Admitted`,
  - latest `Confirmed` or `record.status == Confirmed` -> `Confirmed`,
  - latest `Failed` or `record.status == Failed` -> `Failed`,
  - latest `Cancelled` or `record.status == Cancelled` -> `Cancelled`,
  - latest `Conflicted` -> `Conflicted`,
  - latest `AlreadySpent` -> `AlreadySpent`,
  - no latest kind and `record.imported == true` -> `Imported`,
  - no latest kind and pending non-imported record -> `Created`.

Files:
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`

Tests:
- Add table-driven tests for every mapping rule.

Verify:
```bash
cargo test -p z00z_wallets tx_rpc_storage
```

Done when:
- Lifecycle projection is deterministic and independent of string matching.

### TASK-018 - 10. Wallet Receive, Scan, Import, And History Authority Closure

Depends on: `TASK-017`.

Goal: expose lifecycle in public transaction DTOs.

Change:
- In `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`, add `pub lifecycle: RuntimeTxLifecycle` to:
  - `PersistTxInfo`,
  - `RuntimeTxDetailsResponse`,
  - `RuntimeSendTxResponse`,
  - `RuntimeImportTxResponse`,
  - `RuntimeReconcileTxResponse`.
- Keep existing `status: TxStatus` fields.
- Add `#[serde(default)]` only if needed for inbound compatibility; outbound values must always be populated.
- Update constructors and tests.

Files:
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`

Tests:
- Update existing transaction RPC tests to assert both `status` and `lifecycle`.

Verify:
```bash
cargo test -p z00z_wallets --test test_tx_store_integration
cargo test -p z00z_wallets --test test_direct_tx_receive
```

Done when:
- Public RPC responses can explain lifecycle without overloading the coarse status enum.

### TASK-019 - 13. Unsupported Receive-Version Taxonomy

Depends on: `TASK-018`.

Goal: centralize receive-version error mapping.

Change:
- Create `crates/z00z_wallets/src/adapters/rpc/methods/receive_error_codes.rs` or add an equivalent private module under existing RPC error mapping.
- Add a function:
  - `runtime_tx_error_code_from_message(message: &str) -> RuntimeTxErrorCode`
  - `runtime_tx_error_code_from_wallet_error(error: &WalletError) -> RuntimeTxErrorCode`
- Map unsupported versions from wallet errors, payment requests, receiver cards, claim packages, asset packs, and portable tx packages to `UnsupportedReceiveVersion` or `UnsupportedPackageVersion`.
- Do not expose raw package bytes, memo plaintext, receiver secret, scan key, or encrypted payload internals in error strings.

Files:
- `crates/z00z_wallets/src/adapters/rpc/error_mapping.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/src/receiver/request/mod.rs`
- `crates/z00z_wallets/src/receiver/card/stealth_card.rs`
- `crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl.rs`

Tests:
- Add unit tests that unsupported version errors from each source map to the expected typed code.

Verify:
```bash
cargo test -p z00z_wallets --test test_import_error_taxonomy
cargo test -p z00z_wallets --test test_stealth_request
```

Done when:
- Unsupported receive/package versions have stable typed codes and redacted messages.

### TASK-020 - 15. Offline TxPackage Verify, Report, And Import Hardening

Depends on: `TASK-019`.

Goal: make tx package verify reports deterministic.

Change:
- In `RuntimeVerifyTxPkgResponse`, add:
  - `pub error_codes: Vec<RuntimeTxErrorCode>`,
  - `pub lifecycle: RuntimeTxLifecycle`.
- Keep existing `errors: Vec<String>` for backward compatibility.
- In `verify_transaction_package_impl`, fill `error_codes` from verifier errors and package readiness checks.
- If `verify.valid == false`, lifecycle must be `Failed`.
- If valid but not import-ready, lifecycle must be `Submitted` or `Created` only if package semantics prove it; otherwise use `Created`.
- If valid and import-ready, lifecycle must be `Admitted` for verified/admitted packages or `Confirmed` for confirmed packages.

Files:
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`

Tests:
- Add tests for invalid digest, invalid public spend proof, unsupported package version, and not-import-ready package.
- Assert `error_codes` exactly, not only string messages.

Verify:
```bash
cargo test -p z00z_wallets --test test_direct_tx_receive
```

Done when:
- Verify report has stable machine-readable reasons for every rejection path.

### TASK-021 - 15. Offline TxPackage Verify, Report, And Import Hardening

Depends on: `TASK-020`.

Goal: make tx package import reports deterministic and idempotent.

Change:
- In `RuntimeImportTxResponse`, add:
  - `pub lifecycle: RuntimeTxLifecycle`,
  - `pub error_codes: Vec<RuntimeTxErrorCode>`.
- On successful first import, lifecycle must be `Imported`.
- On repeated import of the same package with identical owned outputs, lifecycle must remain `Imported`, and no duplicate assets or tx-history rows may be created.
- On conflict with an existing different payload for the same tx id or asset id, call `record_conflicted` and return or surface `DuplicateConflict`.
- On already-spent input conflict, call `record_already_spent` and return or surface `AlreadySpent`.

Files:
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`

Tests:
- Add duplicate import test:
  - import valid package,
  - import the same package again,
  - assert one tx-history import event or explicitly idempotent event policy,
  - assert no duplicate owned asset rows,
  - assert response lifecycle is `Imported`.
- Add conflict import test:
  - same tx id with different owned output payload,
  - assert `DuplicateConflict`.
- Add already-spent test:
  - package tries to spend an input already confirmed spent,
  - assert `AlreadySpent`.

Verify:
```bash
cargo test -p z00z_wallets --test test_direct_tx_receive
cargo test -p z00z_wallets --test test_tx_store_integration
cargo test -p z00z_wallets --test test_asset_replay_protection
```

Done when:
- Import can be repeated safely and all conflict outcomes are visible in lifecycle and error codes.

### TASK-022 - 16. Tx-History And wallet.asset.* Authority Convergence

Depends on: `TASK-021`.

Goal: prove tx lifecycle survives restart.

Change:
- Add restart/reload tests using the canonical JSONL tx-history path.
- Test sequence:
  - create tx,
  - import tx,
  - submit tx,
  - admit tx,
  - confirm tx,
  - cancel or fail another tx,
  - record conflicted and already-spent cases,
  - reopen storage,
  - query public RPC history,
  - assert lifecycle projection for every tx.

Files:
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`

Tests:
- Add a test named `test_tx_lifecycle_projection_survives_restart`.

Verify:
```bash
cargo test -p z00z_wallets --test test_tx_store_integration
```

Done when:
- Restart does not collapse `Imported`, `Submitted`, `Admitted`, `Conflicted`, or `AlreadySpent` into opaque `Pending` or `Failed` in public responses.

### TASK-023 - 16. Tx-History And wallet.asset.* Authority Convergence

Depends on: `TASK-022`.

Goal: prove `wallet.asset.*` remains cash-only authority.

Change:
- Confirm `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs` rejects `OwnedObjectPayload::Asset`.
- Confirm `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` is the only write path for claimed cash assets.
- Add tests if either behavior is not covered.

Files:
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs`
- `crates/z00z_wallets/tests/test_asset_import_security.rs`

Tests:
- Add a test named `test_object_inventory_rejects_cash_asset_payload`.
- Add a test named `test_asset_import_cannot_write_object_inventory_as_cash`.

Verify:
```bash
cargo test -p z00z_wallets --test test_asset_import_security
```

Done when:
- Cash assets cannot be inserted through the object inventory path.

### TASK-024 - 10. Wallet Receive, Scan, Import, And History Authority Closure

Depends on: `TASK-023`.

Goal: prove remote worker evidence stays advisory.

Change:
- Add tests around `WalletService::recv_range_with_worker`.
- The tests must pass remote worker evidence into the canonical receive lane and assert only wallet-local scan/persist code mutates assets and cursor.
- Tests must reject:
  - empty chunk hash,
  - non-increasing chunk height,
  - non-contiguous chunk height,
  - empty proof hint bytes,
  - proof hint for unknown checkpoint height,
  - resume hint that rewinds local cursor,
  - resume hint that mismatches local cursor.

Files:
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs`
- `crates/z00z_wallets/tests/test_stealth_scanner_cache.rs`

Tests:
- Add or extend worker-evidence tests in the closest existing receive/scan integration file.

Verify:
```bash
cargo test -p z00z_wallets --test test_stealth_scanner_flow
cargo test -p z00z_wallets --test test_stealth_scanner_cache
```

Done when:
- Invalid worker evidence fails before asset/cursor mutation.

### TASK-025 - 13. Unsupported Receive-Version Taxonomy

Depends on: `TASK-024`.

Goal: prove unsupported versions never mutate wallet state.

Change:
- Add no-mutation tests for:
  - unsupported payment request version,
  - unsupported receiver card version,
  - unsupported asset pack version,
  - unsupported claim package version,
  - unsupported portable tx package version.
- Each test must snapshot wallet state before failure and compare after failure.
- Snapshot must include tx-history row count, owned asset count, object inventory count, scan cursor, and relevant request/pin state.

Files:
- `crates/z00z_wallets/tests/test_stealth_request.rs`
- `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`
- `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- `crates/z00z_wallets/tests/test_tx_serial.rs`

Tests:
- Add one no-mutation test per unsupported version surface.

Verify:
```bash
cargo test -p z00z_wallets --test test_stealth_request
cargo test -p z00z_wallets --test test_import_error_taxonomy
cargo test -p z00z_wallets --test test_tx_serial
cargo test -p z00z_wallets --test test_direct_tx_receive
```

Done when:
- Unsupported version errors have stable codes and leave state unchanged.

### TASK-026 - 14. Wallet Scan Orchestration And Runtime Scan Status

Depends on: `TASK-025`.

Goal: extend existing scan status instead of creating a second scan status type.

Change:
- In `crates/z00z_wallets/src/adapters/rpc/types/chain.rs`, add:
  - `RuntimeReceiveScanOutcome`,
  - optional field `pub last_receive_outcome: Option<RuntimeReceiveScanOutcome>` in `RuntimeScanStatus`.
- `RuntimeReceiveScanOutcome` must use `#[serde(rename_all = "snake_case")]` and include:
  - `Scanned`,
  - `Resumed`,
  - `NoHit`,
  - `ImportedHit`,
  - `WorkerEvidenceRejected`,
  - `CursorConflict`,
  - `UnsupportedVersion`.
- Preserve existing `RuntimeScanStatus::is_scanned()` behavior.

Files:
- `crates/z00z_wallets/src/adapters/rpc/types/chain.rs`

Tests:
- Add serialization tests for `RuntimeReceiveScanOutcome`.
- Update existing `RuntimeScanStatus` tests to cover `last_receive_outcome`.

Verify:
```bash
cargo test -p z00z_wallets chain::tests
```

Done when:
- There is one public scan status DTO with optional receive outcome details.

### TASK-027 - 14. Wallet Scan Orchestration And Runtime Scan Status

Depends on: `TASK-026`.

Goal: produce receive scan outcomes from canonical receive lane.

Change:
- Add a small internal result struct near `recv_range_authoritative`, for example `ReceiveRangeOutcome`.
- Include:
  - `done_ckpt`,
  - `hit_count`,
  - `resume_height`,
  - `cursor_height`,
  - `outcome: RuntimeReceiveScanOutcome` or internal equivalent mapped at RPC boundary.
- Set outcome rules:
  - origin scan with no hits -> `NoHit`,
  - resumed scan with no hits -> `Resumed`,
  - scan with imported assets -> `ImportedHit`,
  - worker validation error -> `WorkerEvidenceRejected`,
  - cursor mismatch during persistence -> `CursorConflict`,
  - unsupported pack/version error -> `UnsupportedVersion`,
  - accepted scan with no special condition -> `Scanned`.

Files:
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/chain_impl.rs`

Tests:
- Add receive-lane tests for each outcome.

Verify:
```bash
cargo test -p z00z_wallets --test test_stealth_scanner_flow
cargo test -p z00z_wallets --test test_e2e_req_flow
```

Done when:
- Every receive path has a deterministic outcome value.

### TASK-028 - 14. Wallet Scan Orchestration And Runtime Scan Status

Depends on: `TASK-027`.

Goal: prove cursor and asset persistence are atomic.

Change:
- Add a restart/resume test for `persist_scan_batch`.
- The test must inject or simulate failure between candidate asset preparation and transaction commit if existing test hooks allow it.
- If failpoint hooks do not exist, add a minimal test-only hook under `#[cfg(test)]`.
- Assert that after failure:
  - no partial owned asset rows exist,
  - scan cursor remains unchanged,
  - retry succeeds,
  - final asset rows and cursor commit together.

Files:
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`
- `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs`

Tests:
- Add a test named `test_scan_batch_assets_and_cursor_commit_atomically`.

Verify:
```bash
cargo test -p z00z_wallets --test test_stealth_scanner_flow
```

Done when:
- A failed scan persistence attempt cannot leave assets without cursor or cursor without assets.

### TASK-029 - 15. Offline TxPackage Verify, Report, And Import Hardening

Depends on: `TASK-028`.

Goal: align portable tx parsing with typed error codes.

Change:
- In `parse_portable_tx`, replace ambiguous string-only failures with helper mapping to `RuntimeTxErrorCode`.
- Keep JSON-RPC error messages concise and redacted.
- Ensure unsupported `package_version` maps to `UnsupportedPackageVersion`.
- Ensure metadata hash mismatch maps to `InvalidDigest`.
- Ensure tx digest mismatch maps to `InvalidDigest`.
- Ensure chain id mismatch maps to `WrongChain`.

Files:
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`

Tests:
- Add one test per mapping.

Verify:
```bash
cargo test -p z00z_wallets --test test_import_error_taxonomy
cargo test -p z00z_wallets --test test_direct_tx_receive
```

Done when:
- Portable tx parse failures are typed, deterministic, redacted, and no-mutation.

### TASK-030 - 18. Request-Bound Inbox Helper Plan

Depends on: `TASK-029`.

Goal: add a local request-bound inbox helper without creating receive authority.

Change:
- Add a local helper module under `crates/z00z_wallets/src/receiver/request/` or `crates/z00z_wallets/src/services/wallet/actions/`.
- Name it `request_inbox` unless local style suggests a clearer name.
- Store only advisory records:
  - request id,
  - chain id,
  - recipient binding,
  - expiry,
  - optional range hint,
  - validation result,
  - created timestamp.
- The helper must not write owned assets, owned objects, tx rows, or scan cursor.
- The only execution path from inbox to wallet mutation must call `WalletService::recv_range` or the same canonical receive lane.

Files:
- `crates/z00z_wallets/src/receiver/request/mod.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`

Tests:
- Add unit tests proving helper insert/list/delete does not mutate wallet state.

Verify:
```bash
cargo test -p z00z_wallets --test test_stealth_request
```

Done when:
- Request inbox is an advisory queue and not a second receive persistence path.

### TASK-031 - 18. Request-Bound Inbox Helper Plan

Depends on: `TASK-030`.

Goal: validate request-bound receive failure modes.

Change:
- Add tests for:
  - wrong chain,
  - expired request,
  - bad signature,
  - pin mismatch,
  - unsupported request version,
  - valid request-assisted receive.
- Every invalid case must leave asset rows, object rows, tx history, and scan cursor unchanged.

Files:
- `crates/z00z_wallets/tests/test_stealth_request.rs`
- `crates/z00z_wallets/tests/test_e2e_req_flow.rs`

Tests:
- Add or extend request-bound receive integration tests.

Verify:
```bash
cargo test -p z00z_wallets --test test_stealth_request
cargo test -p z00z_wallets --test test_e2e_req_flow
```

Done when:
- Request-bound receive is useful locally and still has one canonical mutation lane.

### TASK-032 - 21. Simulator Receive, Import, And History Evidence Pack

Depends on: `TASK-031`.

Goal: add simulator evidence for receive/import lifecycle rows.

Change:
- Extend scenario 1 evidence so it records:
  - imported,
  - submitted,
  - admitted,
  - confirmed,
  - duplicate import,
  - conflicted,
  - already-spent,
  - no-owned-output rejection,
  - wrong-chain rejection,
  - invalid digest rejection,
  - unsupported package version rejection.
- Each row must contain:
  - tx id,
  - lifecycle,
  - coarse status,
  - error code if rejected,
  - whether wallet asset rows changed,
  - whether tx-history row count changed,
  - whether restart verification passed.

Files:
- `crates/z00z_simulator/src/scenario_1/`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`

Tests:
- Extend `test_hjmt_e2e` to assert all listed evidence rows exist.

Verify:
```bash
cargo test -p z00z_simulator --test test_hjmt_e2e
```

Done when:
- Simulator receive/import evidence covers success, duplicate, conflict, restart, and negative package paths.

### TASK-033 - 21. Simulator Receive, Import, And History Evidence Pack

Depends on: `TASK-032`.

Goal: prove simulator receive/import evidence survives restart.

Change:
- Add restart stage in scenario 1 after receive/import and before final evidence verification.
- Reopen wallet/storage state.
- Recompute tx lifecycle projection from tx-history JSONL.
- Re-read owned asset rows.
- Compare to pre-restart evidence.

Files:
- `crates/z00z_simulator/src/scenario_1/`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`

Tests:
- Add assertions for pre-restart and post-restart equality.

Verify:
```bash
cargo test -p z00z_simulator --test test_hjmt_e2e
```

Done when:
- Restart cannot hide receive/import/history mismatch.

### TASK-034 - 10. Wallet Receive, Scan, Import, And History Authority Closure

Depends on: `TASK-033`.

Goal: close wallet receive/import/history authority.

Change:
- Edit source section `👍 10. Wallet Receive, Scan, Import, And History Authority Closure`.
- Add a closeout note that lists:
  - lifecycle projection type,
  - receive status outcome type,
  - advisory worker tests,
  - asset/object authority tests,
  - request-bound helper boundary,
  - simulator receive/import evidence.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "RuntimeTxLifecycle|RuntimeReceiveScanOutcome|advisory worker|wallet.asset|wallet.object" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 10 has a source-backed closeout and points to the implemented tests.

### TASK-035 - 13. Unsupported Receive-Version Taxonomy

Depends on: `TASK-034`.

Goal: close unsupported receive-version taxonomy.

Change:
- Edit source section `👍 13. Unsupported Receive-Version Taxonomy`.
- Add table listing each unsupported-version surface and its public error code:
  - payment request,
  - receiver card,
  - asset pack,
  - claim package,
  - tx package,
  - portable tx package.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "UnsupportedReceiveVersion|UnsupportedPackageVersion|no mutation" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 13 closeout documents one taxonomy and no-mutation guarantees.

### TASK-036 - 14. Wallet Scan Orchestration And Runtime Scan Status

Depends on: `TASK-035`.

Goal: close wallet scan orchestration and runtime scan status.

Change:
- Edit source section `👍 14. Wallet Scan Orchestration And Runtime Scan Status`.
- Add evidence for:
  - canonical scan lane,
  - advisory worker lane,
  - atomic asset/cursor persistence,
  - runtime receive outcomes,
  - restart/resume tests.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "RuntimeReceiveScanOutcome|atomic|worker|resume" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 14 closeout explains every accepted, resumed, rejected, and unsupported scan outcome.

### TASK-037 - 15. Offline TxPackage Verify, Report, And Import Hardening

Depends on: `TASK-036`.

Goal: close TxPackage verify/report/import hardening.

Change:
- Edit source section `👍 15. Offline TxPackage Verify, Report, And Import Hardening`.
- Add evidence for:
  - verify is pre-broadcast advisory,
  - import is idempotent,
  - duplicate import does not duplicate assets/history,
  - invalid digest, wrong chain, unsupported version, invalid proof, no owned output, conflict, and already-spent are typed.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "idempotent|InvalidDigest|WrongChain|AlreadySpent|DuplicateConflict" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 15 closeout maps every reject path to a typed test.

### TASK-038 - 16. Tx-History And wallet.asset.* Authority Convergence

Depends on: `TASK-037`.

Goal: close tx-history and asset authority convergence.

Change:
- Edit source section `👍 16. Tx-History And wallet.asset.* Authority Convergence`.
- Add evidence for:
  - append-only JSONL tx-history,
  - lifecycle projection,
  - restart/reload preservation,
  - conflict/already-spent event kinds,
  - `wallet.asset.*` cash-only authority,
  - `wallet.object.*` non-cash inventory rejection of asset payloads.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "tx-history|lifecycle|AlreadySpent|Conflicted|wallet.asset|wallet.object" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 16 closeout can be verified from code and tests without interpreting wallet internals.

### TASK-039 - 21. Simulator Receive, Import, And History Evidence Pack

Depends on: `TASK-038`.

Goal: close simulator receive/import/history evidence.

Change:
- Edit source section `👍 21. Simulator Receive, Import, And History Evidence Pack`.
- Add evidence table mapping simulator rows to lifecycle/status/error-code expectations.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "Simulator Receive|Imported|Admitted|DuplicateConflict|AlreadySpent" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 21 no longer depends on unresolved wallet status taxonomy work.

### TASK-040 - 11. Field-Native Pack Migration Plan

Depends on: `TASK-039`.

Goal: make field-native pack closure boundary explicit.

Change:
- Inspect current pack implementation:
  - `crates/z00z_core/src/assets/version.rs`,
  - `crates/z00z_core/src/assets/leaf.rs`,
  - `crates/z00z_crypto/src/protocol/zkpack.rs`,
  - `crates/z00z_wallets/src/stealth/zkpack/`.
- Decide exactly one closure mode:
  - close current `ZkPackEncrypted` + memo-pack + unsupported-version slice,
  - or implement field-native cipher-suite migration now.
- The recommended mode is to close current slice and create a future field-native/Poseidon2 follow-up, because current wallet code still uses the existing AEAD facade.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_wallets/src/stealth/zkpack/`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "ZkPackEncrypted|field-native|Poseidon2|future" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_wallets/src/stealth/zkpack
```

Done when:
- Phase 11 does not imply current field-native circuit parity unless that parity has been implemented and tested.

### TASK-041 - 11. Field-Native Pack Migration Plan

Depends on: `TASK-040`.

Goal: finish pack-version and ciphertext negative tests.

Change:
- Add or confirm tests for:
  - basic pack version,
  - memo pack version,
  - unsupported pack version,
  - wrong AAD,
  - wrong recipient key,
  - truncated ciphertext,
  - oversized memo,
  - unknown `AssetPackVersion`.

Files:
- `crates/z00z_wallets/tests/test_zkpack.rs`
- `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs`
- `crates/z00z_wallets/tests/test_golden_tag16.rs`
- `crates/z00z_wallets/src/receiver/scan/types_receive.rs`

Tests:
- Add missing cases to existing test files rather than creating duplicate test suites.

Verify:
```bash
cargo test -p z00z_wallets --test test_zkpack
cargo test -p z00z_wallets --test test_asset_pack_v2_memo
cargo test -p z00z_wallets --test test_golden_tag16
```

Done when:
- Current pack slice fails closed on unsupported versions and malformed ciphertext.

### TASK-042 - 12. Privacy, Stealth, And Selective Disclosure Primitives

Depends on: `TASK-041`.

Goal: define selective-disclosure matrix.

Change:
- Add a test matrix for `WalletReveal::Present`, `WalletReveal::Redacted`, and `WalletReveal::Unavailable`.
- Cover these surfaces:
  - scan result DTO,
  - RPC response DTO,
  - export/report DTO,
  - logs.
- Public/report/log surfaces must not expose memo plaintext, receiver secret, blinding, output secret, or private scan key.

Files:
- `crates/z00z_wallets/src/receiver/scan/types_receive.rs`
- `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs`
- `crates/z00z_wallets/tests/test_view_key_contract.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`

Tests:
- Add `test_wallet_reveal_matrix_public_surfaces`.
- Add log redaction assertions for memo and reveal fields.

Verify:
```bash
cargo test -p z00z_wallets --test test_view_key_contract
cargo test -p z00z_wallets --test test_rpc_logging_risk_policy
```

Done when:
- Each reveal state has an explicit behavior on each surface.

### TASK-043 - 12. Privacy, Stealth, And Selective Disclosure Primitives

Depends on: `TASK-042`.

Goal: keep stealth privacy separate from transport anonymity.

Change:
- Edit source section `👍 12. Privacy, Stealth, And Selective Disclosure Primitives`.
- Add statement:
  - stealth pack privacy is a wallet/crypto receive property,
  - it is not a transport anonymity or OnionNet claim,
  - transport anonymity remains out of this closure.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "transport anonymity|OnionNet|stealth pack privacy" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 12 cannot be misread as closing OnionNet or live transport privacy.

### TASK-044 - 17. Package Hygiene And Transport Privacy Plan

Depends on: `TASK-043`.

Goal: add explicit package redaction tests.

Change:
- Add tests for redaction of:
  - `TxPackage`,
  - `ClaimTxPackage`,
  - verify reports,
  - import reports,
  - parse errors,
  - backup metadata,
  - RPC logs.
- Forbidden output:
  - raw package bytes,
  - session token,
  - seed phrase,
  - memo plaintext,
  - private scan key,
  - receiver secret,
  - encrypted payload internals.

Files:
- `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`
- `crates/z00z_wallets/tests/test_backup_metadata_policy.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`

Tests:
- Add or extend log capture tests to assert forbidden substrings are absent.

Verify:
```bash
cargo test -p z00z_wallets --test test_backup_metadata_policy
cargo test -p z00z_wallets --test test_rpc_logging_risk_policy
```

Done when:
- Diagnostics remain useful without leaking wallet-private or package-private material.

### TASK-045 - 17. Package Hygiene And Transport Privacy Plan

Depends on: `TASK-044`.

Goal: grep for risky package logging and fix findings.

Change:
- Run the grep command below.
- For each result, classify it:
  - safe summary,
  - test fixture,
  - risky logging,
  - dead code.
- Replace risky logging with redacted summaries.
- Do not delete test fixtures unless the fixture itself leaks a live secret.

Files:
- `crates/z00z_wallets/`
- `crates/z00z_runtime/`
- `crates/z00z_simulator/`

Tests:
- Add regression test for each risky logging pattern fixed.

Verify:
```bash
rg -n "debug!|trace!|println!|eprintln!|raw.*package|package.*bytes|session.*token|seed phrase|memo" crates/z00z_wallets crates/z00z_runtime crates/z00z_simulator
cargo test -p z00z_wallets --test test_rpc_logging_risk_policy
```

Done when:
- Every risky logging hit is removed or documented as a safe test fixture.

### TASK-046 - 11. Field-Native Pack Migration Plan

Depends on: `TASK-045`.

Goal: close Phase 11.

Change:
- Edit source section `👍 11. Field-Native Pack Migration Plan`.
- Add closeout mode selected in `TASK-040`.
- If current-slice closure was selected, add a future follow-up line:
  - `Future field-native/Poseidon2 parity remains outside this closure.`

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "Field-Native Pack|Future field-native|ZkPackEncrypted" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- The phase is closed or deferred with no ambiguity about current crypto implementation.

### TASK-047 - 12. Privacy, Stealth, And Selective Disclosure Primitives

Depends on: `TASK-046`.

Goal: close Phase 12.

Change:
- Edit source section `👍 12. Privacy, Stealth, And Selective Disclosure Primitives`.
- Add closeout evidence for:
  - tag16 prefilter,
  - stealth scan detection,
  - reveal state matrix,
  - memo redaction,
  - no transport anonymity claim.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "WalletReveal|tag16|memo redaction|transport anonymity" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 12 has exact privacy claims and explicit exclusions.

### TASK-048 - 17. Package Hygiene And Transport Privacy Plan

Depends on: `TASK-047`.

Goal: close Phase 17.

Change:
- Edit source section `👍 17. Package Hygiene And Transport Privacy Plan`.
- Add evidence for backup metadata redaction, RPC logging redaction, package report redaction, and out-of-scope transport anonymity.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "redaction|backup metadata|package report|transport anonymity" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Package hygiene is closed without implying OnionNet or live transport privacy.

### TASK-049 - 28. Multi-Asset Families, Trust Tiers, And Internal Asset Phase

Depends on: `TASK-048`.

Goal: close the bounded internal object-family slice.

Change:
- Confirm code supports internal settlement object families:
  - `RightLeaf`,
  - `VoucherLeaf`,
  - `RightClass`,
  - `FeeEnvelope`,
  - object policy registry,
  - validator fail-closed policy checks.
- Add missing tests for:
  - unknown policy quarantine or rejection,
  - known policy acceptance,
  - missing right rejection,
  - object/cash separation.

Files:
- `crates/z00z_storage/src/settlement/record.rs`
- `crates/z00z_storage/src/settlement/object_package_contract.rs`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`

Tests:
- Add missing assertions to `test_object_policy_verdicts`.

Verify:
```bash
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_wallets --test test_asset_import_security
```

Done when:
- Internal object families are tested without claiming external trust-tier or live cross-chain readiness.

### TASK-050 - 32. Fee Envelope And Rights Wallet Extensions

Depends on: `TASK-049`.

Goal: expose rights and fee envelope behavior through wallet object inventory.

Change:
- Add wallet object tests proving:
  - rights inventory lists rights,
  - rights rows expose fee envelope metadata,
  - rights rows do not appear in `wallet.asset.*`,
  - locked asset spend without unlock right is rejected,
  - right consumption changes object/right state, not cash asset identity.

Files:
- `crates/z00z_wallets/src/adapters/rpc/methods/object.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`

Tests:
- Add wallet RPC object tests for rights listing and fee envelope projection.
- Extend validator tests for fee boundary rejection if missing.

Verify:
```bash
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_wallets --test test_asset_import_security
```

Done when:
- Fee envelope and rights are visible through object inventory and enforced by validators.

### TASK-051 - 30. Voucher, Payroll, B2B, And Useful-Work Claim Scenarios

Depends on: `TASK-050`.

Goal: add one deterministic voucher scenario.

Change:
- Add a local simulator or validator scenario for voucher issue, accept, redeem, refund, and expire using existing `VoucherAction`.
- The scenario must include:
  - one successful issue,
  - one successful accept,
  - one successful full or partial redeem,
  - one refund or expiry,
  - one replay rejection,
  - one unknown policy rejection.

Files:
- `crates/z00z_storage/src/settlement/tx_plan_types.rs`
- `crates/z00z_simulator/src/scenario_1/`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`

Tests:
- Add `test_voucher_action_lifecycle_local`.

Verify:
```bash
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_simulator --test test_hjmt_e2e
```

Done when:
- Voucher lifecycle has positive and negative local evidence.

### TASK-052 - 30. Voucher, Payroll, B2B, And Useful-Work Claim Scenarios

Depends on: `TASK-051`.

Goal: add one deterministic rights-based business scenario.

Change:
- Add a local payroll or B2B entitlement scenario using `RightLeaf`.
- The scenario must include:
  - entitlement issuance,
  - entitlement use,
  - expiry or invalid use,
  - missing right rejection,
  - malformed fee envelope rejection.
- If useful-work claims cannot be represented with current object/right policy semantics, document useful-work as future scope instead of forcing it into this closure.

Files:
- `crates/z00z_storage/src/settlement/record.rs`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`
- `crates/z00z_simulator/src/scenario_1/`

Tests:
- Add `test_rights_business_entitlement_lifecycle_local`.

Verify:
```bash
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_simulator --test test_hjmt_e2e
```

Done when:
- At least one rights-based business flow executes locally with negative evidence.

### TASK-053 - 29. Local Adapter Model For Cross-Chain Inputs Without Live Chains

Depends on: `TASK-052`.

Goal: implement local adapter fixture for external-input modeling.

Change:
- Add a deterministic local adapter fixture that can publish and resolve external-input batches without live chain access.
- The fixture must use existing local DA or rollup-node adapter boundaries.
- The fixture must expose:
  - batch id,
  - source label,
  - payload digest,
  - publication digest,
  - resolve result,
  - replay id.

Files:
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_runtime/watchers/src/publication.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`

Tests:
- Add or extend rollup-node test for local adapter publish and resolve.

Verify:
```bash
cargo test -p z00z_rollup_node --test test_hjmt_node_lifecycle
```

Done when:
- Local external-input modeling is reproducible and has no live-chain dependency.

### TASK-054 - 29. Local Adapter Model For Cross-Chain Inputs Without Live Chains

Depends on: `TASK-053`.

Goal: fail closed for forged local adapter metadata.

Change:
- Add negative tests for:
  - forged metadata,
  - wrong digest,
  - missing resolve result,
  - replayed external input id.
- Adapter metadata must remain advisory until bound into local publication/checkpoint evidence.

Files:
- `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`

Tests:
- Add one negative test per forged metadata case.

Verify:
```bash
cargo test -p z00z_rollup_node --test test_hjmt_node_lifecycle
cargo test -p z00z_validators --test test_hjmt_publication_contract
cargo test -p z00z_watchers --test test_hjmt_publication_contract
```

Done when:
- Local adapter metadata cannot become authority until publication/checkpoint binding validates it.

### TASK-055 - 33. Agentic Rights Local Simulations

Depends on: `TASK-054`.

Goal: add agentic rights local simulation.

Change:
- Add a local simulator flow for an agent budget or service entitlement.
- The flow must include:
  - issuance,
  - delegation,
  - consumption,
  - expiry,
  - replay rejection,
  - wrong action rejection.
- Evidence rows must include:
  - right id,
  - policy id,
  - action,
  - actor id or local agent label,
  - result,
  - rejection reason.

Files:
- `crates/z00z_storage/src/settlement/record.rs`
- `crates/z00z_simulator/src/scenario_1/`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`

Tests:
- Add `test_agentic_right_lifecycle_local`.

Verify:
```bash
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_simulator --test test_hjmt_e2e
```

Done when:
- Agentic rights have deterministic local evidence and do not grant cash spend authority by themselves.

### TASK-056 - 34. Machine Capability Local Simulations

Depends on: `TASK-055`.

Goal: add machine capability local simulation.

Change:
- Add a local simulator flow for `RightClass::MachineCapability`.
- The flow must include:
  - issuance,
  - one-time use,
  - expiry,
  - wrong object rejection,
  - wrong action rejection,
  - reuse rejection.
- Evidence rows must include:
  - capability id,
  - policy id,
  - authorized action,
  - target object id,
  - expiry,
  - result,
  - rejection reason.

Files:
- `crates/z00z_storage/src/settlement/record.rs`
- `crates/z00z_simulator/src/scenario_1/`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`

Tests:
- Add `test_machine_capability_lifecycle_local`.

Verify:
```bash
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_simulator --test test_hjmt_e2e
```

Done when:
- Machine capability rights are deterministic and fail closed for wrong action, reuse, and expiry.

### TASK-057 - 28. Multi-Asset Families, Trust Tiers, And Internal Asset Phase

Depends on: `TASK-056`.

Goal: close bounded Phase 28.

Change:
- Edit source section `👍 28. Multi-Asset Families, Trust Tiers, And Internal Asset Phase`.
- State that closure covers internal object families, rights, vouchers, policy registry, wallet object display, and validator fail-closed behavior.
- State that external chain trust tiers, linked liability, and live cross-chain settlement are excluded.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "internal object families|external chain trust tiers|linked liability|excluded" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 28 is closed only as a bounded internal implementation.

### TASK-058 - 29. Local Adapter Model For Cross-Chain Inputs Without Live Chains

Depends on: `TASK-057`.

Goal: close bounded Phase 29.

Change:
- Edit source section `👍 29. Local Adapter Model For Cross-Chain Inputs Without Live Chains`.
- State that closure covers deterministic local adapter fixtures only.
- State that live bridges, live external chain settlement, and external trust claims remain out of scope.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "local adapter|live bridge|out of scope" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 29 cannot be interpreted as live cross-chain readiness.

### TASK-059 - 30. Voucher, Payroll, B2B, And Useful-Work Claim Scenarios

Depends on: `TASK-058`.

Goal: close bounded Phase 30.

Change:
- Edit source section `👍 30. Voucher, Payroll, B2B, And Useful-Work Claim Scenarios`.
- List the voucher and rights-based local scenarios implemented in `TASK-051` and `TASK-052`.
- If useful-work was deferred, state the exact reason and future phase requirement.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "voucher|rights-based|useful-work|future" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 30 is closed for implemented local scenarios and does not overclaim future useful-work scope.

### TASK-060 - 32. Fee Envelope And Rights Wallet Extensions

Depends on: `TASK-059`.

Goal: close bounded Phase 32.

Change:
- Edit source section `👍 32. Fee Envelope And Rights Wallet Extensions`.
- Add evidence for:
  - fee envelope validation,
  - rights listing,
  - right consumption,
  - locked asset spend rejection,
  - cash/object separation after restart.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "fee envelope|rights listing|locked asset|wallet.object" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 32 explains wallet-visible rights behavior and validator enforcement.

### TASK-061 - 33. Agentic Rights Local Simulations

Depends on: `TASK-060`.

Goal: close Phase 33.

Change:
- Edit source section `👍 33. Agentic Rights Local Simulations`.
- Add evidence for issuance, delegation, consumption, expiry, replay rejection, and wrong action rejection.
- State that linked liability and live external enforcement are out of scope.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "agentic rights|delegation|replay rejection|linked liability|out of scope" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 33 closes local simulation only.

### TASK-062 - 34. Machine Capability Local Simulations

Depends on: `TASK-061`.

Goal: close Phase 34.

Change:
- Edit source section `👍 34. Machine Capability Local Simulations`.
- Add evidence for one-time use, expiry, wrong object rejection, wrong action rejection, and reuse rejection.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "MachineCapability|one-time|wrong action|reuse rejection" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Phase 34 has local deterministic evidence and fail-closed semantics.

### TASK-063 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-062`.

Goal: create final residual gap register.

Change:
- Add a residual register under source section `👍 36. Spec-Gap Normalization And Residual Hardening Gate`.
- The register must classify each residual as one of:
  - `Closed`,
  - `Bounded closed`,
  - `Deferred future implementation`,
  - `Research only`,
  - `Out of scope`.
- Include at least:
  - Recursive proof backend,
  - linked liability,
  - OnionNet,
  - live external DA,
  - live cross-chain bridge,
  - field-native/Poseidon2 parity if deferred,
  - useful-work scenario if deferred.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "Residual gap register|Recursive|Linked Liability|OnionNet|Out of scope|Research only" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Future/research work is not mixed with closeable implementation work.

### TASK-064 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-063`.

Goal: add stale-term guardrails.

Change:
- Add or update grep/static guardrail tests that fail on stale implementation claims:
  - `AssetStateRoot` as live public root,
  - `AssetPath` as live public settlement path,
  - recursive proof implemented,
  - OnionNet implemented,
  - linked liability implemented.
- Guardrails must allow historical discussion only if clearly marked as future, research, superseded, or out of scope.

Files:
- `crates/z00z_wallets/tests/test_spec_terms_guard.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- Add guardrail cases for each stale claim.

Verify:
```bash
cargo test -p z00z_wallets --test test_spec_terms_guard
cargo test -p z00z_storage --test test_live_guardrails
```

Done when:
- Stale public claims fail tests unless they are explicitly marked non-live.

### TASK-065 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-064`.

Goal: ensure every `👍` source section has closeout status.

Change:
- In `.planning/phases/Z00Z-IMPL-PHASES.md`, every `👍` section must include one of:
  - `Closeout status: Closed`,
  - `Closeout status: Bounded closed`,
  - `Closeout status: Deferred after partial implementation`.
- Do not leave a `👍` section without status.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`

Tests:
- Add a static docs test if no existing docs guardrail enforces this.

Verify:
```bash
rg -n "^## 👍" .planning/phases/Z00Z-IMPL-PHASES.md
rg -n "Closeout status:" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Counts match: number of `## 👍` headings equals number of closeout status lines in those sections.

### TASK-066 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-065`.

Goal: ensure this TODO remains linked from the source phase file.

Change:
- Add a short pointer near the top of `.planning/phases/Z00Z-IMPL-PHASES.md`:
  - `Detailed gap closure execution plan: .planning/phases/TODO-gaps.md`.
- The pointer must say this TODO is the active execution plan for `👍` sections.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `.planning/phases/TODO-gaps.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "TODO-gaps.md|active execution plan" .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- Future readers can find this file from the source phase document.

### TASK-067 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-066`.

Goal: run focused validation suites.

Change:
- Run the focused validation suites listed in `Final Verification Commands`.
- If a command fails, do not mark the task complete.
- Create a blocker note under this task with:
  - command,
  - failing test or error essence,
  - suspected owner file,
  - next action.

Files:
- No code file required unless failures are fixed.

Tests:
- The validation commands are the test.

Verify:
```bash
cargo test -p z00z_storage --test test_hjmt_backend_conformance
cargo test -p z00z_storage --test test_live_guardrails
cargo test -p z00z_storage --test test_hjmt_compat_equivalence
cargo test -p z00z_storage --test test_claim_source_proof
cargo test -p z00z_storage --test test_checkpoint_root_binding
cargo test -p z00z_storage --test test_checkpoint_finalization
cargo test -p z00z_storage --test test_bench_lanes
cargo test -p z00z_wallets --test test_direct_tx_receive
cargo test -p z00z_wallets --test test_tx_store_integration
cargo test -p z00z_wallets --test test_asset_import_security
cargo test -p z00z_wallets --test test_asset_replay_protection
cargo test -p z00z_wallets --test test_import_error_taxonomy
cargo test -p z00z_wallets --test test_stealth_request
cargo test -p z00z_wallets --test test_stealth_scanner_flow
cargo test -p z00z_wallets --test test_stealth_scanner_cache
cargo test -p z00z_wallets --test test_e2e_req_flow
cargo test -p z00z_wallets --test test_zkpack
cargo test -p z00z_wallets --test test_asset_pack_v2_memo
cargo test -p z00z_wallets --test test_golden_tag16
cargo test -p z00z_wallets --test test_e2e_tag_auth
cargo test -p z00z_wallets --test test_stealth_scanner_prefilter
cargo test -p z00z_wallets --test test_view_key_contract
cargo test -p z00z_wallets --test test_rpc_logging_risk_policy
cargo test -p z00z_simulator --test test_checkpoint_acceptance
cargo test -p z00z_simulator --test test_hjmt_e2e
cargo test -p z00z_simulator --test test_scenario1_stage_surface
cargo test -p z00z_rollup_node --test test_hjmt_node_lifecycle
cargo test -p z00z_validators --test test_hjmt_publication_contract
cargo test -p z00z_validators --test test_object_policy_verdicts
cargo test -p z00z_watchers --test test_hjmt_publication_contract
```

Done when:
- All focused validation commands pass or have explicit unresolved blocker notes.

### TASK-068 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-067`.

Goal: run repository-level verification.

Change:
- Run formatting, linting, tests, docs, and full verify.
- Fix failures that are caused by this TODO execution.
- Do not fix unrelated dirty-worktree failures unless they block validating these changes and the fix is clearly local to this work.

Files:
- No specific file; depends on failures.

Tests:
- Repository-level validation.

Verify:
```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all
cargo doc --no-deps
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh
```

Done when:
- Repository-level verification passes or every unrelated failure has a blocker note with exact command and error essence.

### TASK-069 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-068`.

Goal: final source-plan consistency check.

Change:
- Run the grep checks below.
- Inspect each hit manually.
- Every hit must be one of:
  - implementation evidence,
  - explicit future/deferred scope,
  - historical/superseded terminology,
  - test fixture for guardrails.
- Any ambiguous hit must be rewritten.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `.planning/phases/TODO-gaps.md`
- `docs/`
- `crates/`

Tests:
- Static review and grep checks.

Verify:
```bash
rg -n "AssetStateRoot|AssetPath|recursive proof|OnionNet|Linked Liability|LiabilityDomain|FraudProof|LockRegistry|live bridge|transport anonymity" .planning docs crates
```

Done when:
- No ambiguous stale implementation claim remains.

### TASK-070 - 36. Spec-Gap Normalization And Residual Hardening Gate

Depends on: `TASK-069`.

Goal: close Phase 36 and the TODO.

Change:
- Edit source section `👍 36. Spec-Gap Normalization And Residual Hardening Gate`.
- Add final closeout summary with:
  - closed sections,
  - bounded-closed sections,
  - deferred sections,
  - validation commands run,
  - unresolved blockers if any.
- Mark this TODO file front matter `status` from `Planned` to `Completed` only when every task is complete.
- If any blocker remains, set this TODO front matter `status` to `On Hold` and list blockers.

Files:
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `.planning/phases/TODO-gaps.md`

Tests:
- No new test in this task.

Verify:
```bash
rg -n "status: Completed|status: On Hold|Final closeout summary" .planning/phases/TODO-gaps.md .planning/phases/Z00Z-IMPL-PHASES.md
```

Done when:
- The TODO status accurately reflects execution state and the source phase file has final closeout summary.

## 🔑 Source Section Closure Matrix

Use this matrix after executing the task list. Do not mark a row closed until all task ids in that row are complete.

| Source Section | Required Tasks | Closure Type |
|---|---|---|
| `0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout` | `TASK-001` through `TASK-004` | Closed as normalized/superseded |
| `9. Storage Claim-Root And Checkpoint Authority Closure` | `TASK-005` through `TASK-006` | Closed |
| `10. Wallet Receive, Scan, Import, And History Authority Closure` | `TASK-014`, `TASK-018`, `TASK-021` through `TASK-024`, `TASK-034` | Closed after lifecycle/scan/import tests |
| `11. Field-Native Pack Migration Plan` | `TASK-040`, `TASK-041`, `TASK-046` | Closed or deferred-by-boundary |
| `12. Privacy, Stealth, And Selective Disclosure Primitives` | `TASK-042`, `TASK-043`, `TASK-047` | Closed |
| `13. Unsupported Receive-Version Taxonomy` | `TASK-019`, `TASK-025`, `TASK-035` | Closed |
| `14. Wallet Scan Orchestration And Runtime Scan Status` | `TASK-026` through `TASK-028`, `TASK-036` | Closed |
| `15. Offline TxPackage Verify, Report, And Import Hardening` | `TASK-020`, `TASK-021`, `TASK-029`, `TASK-037` | Closed |
| `16. Tx-History And wallet.asset.* Authority Convergence` | `TASK-015` through `TASK-018`, `TASK-022`, `TASK-023`, `TASK-038` | Closed |
| `17. Package Hygiene And Transport Privacy Plan` | `TASK-044`, `TASK-045`, `TASK-048` | Closed |
| `18. Request-Bound Inbox Helper Plan` | `TASK-030`, `TASK-031` | Closed |
| `19. Local Publication, Simulator Evidence, And Restart/Tamper Harness` | `TASK-007`, `TASK-008` | Closed |
| `20. Simulator Checkpoint, Theorem, Tamper, And Restart Evidence Pack` | `TASK-009` | Closed |
| `21. Simulator Receive, Import, And History Evidence Pack` | `TASK-032`, `TASK-033`, `TASK-039` | Closed |
| `22. Benchmark, Proof-Size, And Evidence Guardrails` | `TASK-010`, `TASK-011` | Closed |
| `27. Optional Proof-Size And Storage Measurement Sidecar` | `TASK-012`, optional `TASK-013` | Closed by selected mode |
| `28. Multi-Asset Families, Trust Tiers, And Internal Asset Phase` | `TASK-049`, `TASK-057` | Bounded closed |
| `29. Local Adapter Model For Cross-Chain Inputs Without Live Chains` | `TASK-053`, `TASK-054`, `TASK-058` | Bounded closed |
| `30. Voucher, Payroll, B2B, And Useful-Work Claim Scenarios` | `TASK-051`, `TASK-052`, `TASK-059` | Bounded closed |
| `32. Fee Envelope And Rights Wallet Extensions` | `TASK-050`, `TASK-060` | Bounded closed |
| `33. Agentic Rights Local Simulations` | `TASK-055`, `TASK-061` | Bounded closed |
| `34. Machine Capability Local Simulations` | `TASK-056`, `TASK-062` | Bounded closed |
| `36. Spec-Gap Normalization And Residual Hardening Gate` | `TASK-063` through `TASK-070` | Closed last |

## 🔎 Doublecheck Reading Map

Use this section before implementation. It is the per-task doublecheck map for the recommendations above.

Rules:
- All references below were checked against workspace files, not graph evidence.
- `Z00Z-IMPL-PHASES.md` is currently a consolidated source document. For legacy source-section names such as phase `9`, `10`, or `16`, first read the phase-order rows in `.planning/phases/Z00Z-IMPL-PHASES.md:152-197`, then read the owning consolidated sections named below.
- When a task says to add a type, enum, method, artifact, or status, treat it as a recommended gap-closing change unless the referenced code already defines that exact item. Read the references to confirm the current shape before adding the missing surface.
- Do not implement from this table alone. Read the task body, the listed source section, and the listed code/test references before editing.

| Task | Doublechecked implementation basis | Required reading before code |
|---|---|---|
| `TASK-001` | Storage work must preserve one semantic settlement-root authority while freezing a migration facade. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:152-170`, `crates/z00z_storage/src/settlement/root_types.md:8-25`, `crates/z00z_storage/src/settlement/root_types.md:52-71`, `crates/z00z_storage/src/backend/mod.rs:1-53`, `crates/z00z_storage/src/settlement/store.rs:51-74`, `crates/z00z_storage/src/settlement/store.rs:309-375`. |
| `TASK-002` | Backend selection is already env-bounded; close the gap by making accepted live modes and startup drift explicit. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `crates/z00z_storage/src/settlement/hjmt_config.rs:7-40`, `crates/z00z_storage/src/settlement/hjmt_config.rs:74-98`, `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs:100-166`, `crates/z00z_storage/tests/test_live_guardrails.rs:1-120`. |
| `TASK-003` | Forest/backend rollout must remain a storage-backend equivalence concern, not a second state-root authority. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:152-170`, `crates/z00z_storage/src/backend/mod.rs:1-53`, `crates/z00z_storage/src/settlement/root_types.md:37-47`, `crates/z00z_storage/src/settlement/root_types.md:113-130`, `crates/z00z_storage/src/settlement/store.rs:701-795`. |
| `TASK-004` | Closure docs and tests must prove hard-cutover and no legacy semantic-reader fallback. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `crates/z00z_storage/src/settlement/root_types.md:52-88`, `crates/z00z_storage/src/settlement/root_types.md:132-144`, `crates/z00z_storage/tests/test_live_guardrails.rs:1-120`, `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs:146-166`. |
| `TASK-005` | Claim-root closure is storage/checkpoint authority work; checkpoint statements already carry `claim_root`, but final seal/reload proof reuse must stay one path. | `.planning/phases/Z00Z-IMPL-PHASES.md:665-708`, `crates/z00z_storage/src/checkpoint/artifact_stmt.rs:9-46`, `crates/z00z_storage/src/checkpoint/artifact_stmt.rs:93-150`, `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs:14-39`, `crates/z00z_storage/src/checkpoint/store.rs:201-252`. |
| `TASK-006` | Simulator checkpoint evidence must reuse storage checkpoint artifacts and fail tamper/reload cases through the same verifier path. | `.planning/phases/Z00Z-IMPL-PHASES.md:665-708`, `crates/z00z_storage/src/checkpoint/store.rs:309-363`, `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs:120-173`, `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs:245-333`, `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs:394-452`. |
| `TASK-007` | Local publication evidence already has runtime and watcher binding primitives; simulator work should exercise those, not invent publication truth. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-181`, `crates/z00z_runtime/aggregators/README.md:7-22`, `crates/z00z_runtime/aggregators/src/service.rs:21-48`, `crates/z00z_runtime/watchers/src/publication.rs:21-82`, `crates/z00z_simulator/README.md:4-30`. |
| `TASK-008` | Restart/tamper harness should be scenario evidence over publication/checkpoint/root bindings, not a new rule owner. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-181`, `crates/z00z_simulator/README.md:46-60`, `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:103-135`, `crates/z00z_runtime/watchers/src/publication.rs:31-82`, `crates/z00z_storage/src/checkpoint/store.rs:348-363`. |
| `TASK-009` | The checkpoint evidence pack must assert artifact shape, tamper rejection, and restart consistency using existing Stage 13 evidence surfaces. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-182`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs:562-705`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs:706-829`, `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:29-101`. |
| `TASK-010` | Benchmark/proof-size reports must stay local evidence metadata and must not change storage authority. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-184`, `.planning/phases/Z00Z-IMPL-PHASES.md:901-943`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs:634-705`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs:721-743`. |
| `TASK-011` | Guardrails must reject performance-report overclaims and keep proof-size data tied to local artifact verification. | `.planning/phases/Z00Z-IMPL-PHASES.md:587-624`, `.planning/phases/Z00Z-IMPL-PHASES.md:901-943`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs:721-763`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs:779-824`. |
| `TASK-012` | Optional measurement sidecar can compare active storage/proof behavior only after authority is stable. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-190`, `.planning/phases/Z00Z-IMPL-PHASES.md:901-943`, `crates/z00z_storage/src/settlement/root_types.md:73-88`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs:634-705`. |
| `TASK-013` | Optional sidecar deferral is valid only when docs explicitly say measurement is non-authoritative and no source section depends on it. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:152-190`, `.planning/phases/TODO-gaps.md:2407-2435`, `crates/z00z_storage/src/settlement/root_types.md:73-88`. |
| `TASK-014` | Wallet receive/import/history authority must flow through `recv_range`, asset persistence, tx package verify/import, and tx history; no second cursor/history plane. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:227-352`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:372-432`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs:114-125`. |
| `TASK-015` | Tx history already has a canonical JSONL sidecar, but public status and row kinds are still coarser than the recommended lifecycle. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:51-108`, `crates/z00z_wallets/src/persistence/tx/tx_storage.rs:6-68`, `crates/z00z_wallets/src/backup/backup_wire.rs:82-130`, `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:181-229`. |
| `TASK-016` | History listing/details must be projections of the durable JSONL store, not the in-memory pending list. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:231-410`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs:108-196`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs:1008-1040`. |
| `TASK-017` | JSONL replay/fold behavior already exists; gap-closing must extend row taxonomy without breaking hash chain and fold semantics. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/backup/backup_wire.rs:82-130`, `crates/z00z_wallets/src/backup/backup_wire.rs:435-481`, `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:122-167`, `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:420-650`. |
| `TASK-018` | Wallet asset convergence depends on authoritative scan persistence and claimed-asset cache sync, not tx history alone. | `.planning/phases/Z00Z-IMPL-PHASES.md:199-317`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:264-315`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs:41-125`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs:656-733`. |
| `TASK-019` | Unsupported receive versions must be explicit at asset-pack decode and receive-report layers. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `crates/z00z_core/src/assets/version.rs:1-23`, `crates/z00z_core/src/assets/leaf.rs:94-135`, `crates/z00z_core/src/assets/leaf.rs:266-279`, `crates/z00z_wallets/src/receiver/scan/types_receive.rs:12-75`. |
| `TASK-020` | Offline verify/report is pre-broadcast package validation plus owned-output scan; it is not import/persist. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/tx/tx_verifier/mod.rs:1-44`, `crates/z00z_wallets/src/tx/tx_verifier/mod.rs:103-133`, `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:190-220`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs:881-910`. |
| `TASK-021` | Import hardening must continue from portable package parse, chain check, owned-output scan, persistence, and rollback. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs:87-160`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs:466-525`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs:50-164`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs:239-312`. |
| `TASK-022` | Admission/confirmation rows are present; lifecycle closure should make them user-visible without inventing consensus finality. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:61-94`, `crates/z00z_wallets/src/persistence/tx/tx_storage.rs:129-149`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs:166-237`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs:198-222`. |
| `TASK-023` | Cancel/export/history mutations already journal through `TxStorage`; gap closure should test row order, rollback, and folded view. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:292-410`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs:912-1005`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs:268-312`. |
| `TASK-024` | Receive authority is `recv_range` plus atomic asset/cursor persistence; worker evidence must remain advisory. | `.planning/phases/Z00Z-IMPL-PHASES.md:199-317`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:1-74`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:318-432`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs:656-733`. |
| `TASK-025` | RPC unsupported-version taxonomy must map to stable wallet/RPC error codes and receive reject/status names. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-564`, `crates/z00z_wallets/src/adapters/rpc/error_mapping.rs:196-203`, `crates/z00z_wallets/src/adapters/rpc/error_mapping.rs:345-362`, `crates/z00z_wallets/src/receiver/scan/types_receive.rs:305-380`, `crates/z00z_wallets/src/receiver/request/mod.rs:38-125`. |
| `TASK-026` | Runtime scan status is currently process-local RPC status; gap closure must bind it cleanly to wallet scan authority and documented limits. | `.planning/phases/Z00Z-IMPL-PHASES.md:199-317`, `crates/z00z_wallets/src/adapters/rpc/types/chain.rs:39-75`, `crates/z00z_wallets/src/adapters/rpc/methods/chain_impl.rs:76-111`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:227-352`. |
| `TASK-027` | Scan orchestration must keep local resume/cursor validation and atomic persistence as the authority boundary. | `.planning/phases/Z00Z-IMPL-PHASES.md:199-317`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:227-315`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:318-432`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs:656-733`. |
| `TASK-028` | Worker-assisted scan can feed evidence only after strict local validation and must re-enter `recv_range_authoritative`. | `.planning/phases/Z00Z-IMPL-PHASES.md:199-317`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:1-74`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:415-432`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs:114-125`. |
| `TASK-029` | Verify report details should expose deterministic reject classes while preserving the verifier as pre-broadcast only. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/tx/tx_verifier/mod.rs:19-44`, `crates/z00z_wallets/src/tx/tx_verifier/mod.rs:103-133`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs:881-910`, `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:200-210`. |
| `TASK-030` | Request-bound inbox helpers must consume payment-request validation and remain advisory/off-consensus. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `crates/z00z_wallets/src/receiver/request/mod.rs:20-23`, `crates/z00z_wallets/src/receiver/request/mod.rs:38-125`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:372-432`. |
| `TASK-031` | Inbox ordering must not replace receive scan authority or persistence; it may only feed request metadata into the canonical lane. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `crates/z00z_wallets/src/receiver/request/mod.rs:92-125`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:372-392`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:415-432`. |
| `TASK-032` | Simulator receive/import/history evidence must prove wallet authority after lower-level wallet tasks are stable. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-184`, `crates/z00z_simulator/README.md:4-30`, `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:29-101`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs:87-160`. |
| `TASK-033` | Simulator artifact inventory should include wallet scan/history flows without leaking secrets or turning simulator into a rule owner. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-184`, `crates/z00z_simulator/README.md:46-60`, `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:39-101`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-315`. |
| `TASK-034` | Receive/import/history docs must point to code-owned authority paths: `recv_range`, `TxStorage`, and package import. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:372-432`, `crates/z00z_wallets/src/persistence/tx/tx_storage.rs:97-153`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs:87-160`. |
| `TASK-035` | Unsupported receive-version closeout needs both receive-pack decode tests and RPC/error-mapping tests. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-564`, `crates/z00z_core/src/assets/version.rs:1-23`, `crates/z00z_core/src/assets/leaf.rs:266-279`, `crates/z00z_wallets/src/receiver/scan/types_receive.rs:305-380`, `crates/z00z_wallets/src/adapters/rpc/error_mapping.rs:345-362`. |
| `TASK-036` | Scan status tests must prove status projection and persistence cursor behavior, not just RPC DTO serialization. | `.planning/phases/Z00Z-IMPL-PHASES.md:199-317`, `crates/z00z_wallets/src/adapters/rpc/types/chain.rs:39-120`, `crates/z00z_wallets/src/adapters/rpc/methods/chain_impl.rs:76-111`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs:656-733`. |
| `TASK-037` | Offline package hardening tests must include verify/report/import separation and portable metadata mismatch. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs:881-910`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs:466-525`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs:105-240`. |
| `TASK-038` | Tx-history convergence tests must prove JSONL row hash/fold, current view, tombstone, and status updates. | `.planning/phases/Z00Z-IMPL-PHASES.md:336-439`, `crates/z00z_wallets/src/backup/backup_wire.rs:105-130`, `crates/z00z_wallets/src/backup/backup_wire.rs:435-481`, `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:420-650`. |
| `TASK-039` | Simulator wallet evidence should join scan, import, history, and publication digests without secret leakage. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-184`, `crates/z00z_simulator/README.md:46-60`, `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:29-135`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-315`. |
| `TASK-040` | Field-native pack work must first freeze current `ZkPack_v1` fixed wire behavior and version-aware asset-pack decode. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `crates/z00z_crypto/src/protocol/zkpack.rs:1-70`, `crates/z00z_crypto/src/protocol/zkpack.rs:72-115`, `crates/z00z_core/src/assets/version.rs:1-23`, `crates/z00z_core/src/assets/leaf.rs:94-135`. |
| `TASK-041` | Memo/basic pack lanes exist at decode/report level; any migration plan must avoid claiming production field-native parity. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-564`, `crates/z00z_core/src/assets/leaf.rs:94-135`, `crates/z00z_core/src/assets/leaf.rs:266-279`, `crates/z00z_wallets/src/receiver/scan/types_receive.rs:12-75`, `crates/z00z_crypto/src/protocol/zkpack.rs:32-70`. |
| `TASK-042` | Privacy/stealth primitive closure should preserve request validation, receive-report taxonomy, and wallet-local secret boundaries. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `crates/z00z_wallets/src/receiver/request/mod.rs:38-125`, `crates/z00z_wallets/src/receiver/scan/types_receive.rs:305-380`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:372-392`. |
| `TASK-043` | Selective disclosure and redaction recommendations must be backed by existing logging and backup metadata policies. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-564`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-315`, `crates/z00z_wallets/tests/test_backup_metadata_policy.rs:80-111`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs:29-88`. |
| `TASK-044` | Package hygiene starts from portable package metadata, tx package parse, and log/export redaction policies. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `crates/z00z_wallets/src/adapters/rpc/types/tx.rs:212-220`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs:466-525`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-315`. |
| `TASK-045` | Backup/export hygiene must keep encrypted export payload versioning, AAD, manifest, and metadata redaction aligned. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-564`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs:29-88`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_profile.rs:84-154`, `crates/z00z_wallets/tests/test_backup_metadata_policy.rs:80-111`. |
| `TASK-046` | Field-native deferral or closure must update docs/tests so future proof-system names cannot be read as live pack truth. | `.planning/phases/Z00Z-IMPL-PHASES.md:526-564`, `crates/z00z_crypto/src/protocol/zkpack.rs:1-70`, `crates/z00z_core/src/assets/version.rs:1-23`, `crates/z00z_core/src/assets/leaf.rs:266-279`. |
| `TASK-047` | Privacy closeout must state that request-bound receive is wallet-local policy and does not prove transport anonymity. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-504`, `.planning/phases/Z00Z-IMPL-PHASES.md:1045-1100`, `crates/z00z_wallets/src/receiver/request/mod.rs:92-125`, `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs:372-392`. |
| `TASK-048` | Transport/package privacy tests should prove no secret material leaks through logs, backups, reports, or simulator artifacts. | `.planning/phases/Z00Z-IMPL-PHASES.md:462-564`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-315`, `crates/z00z_wallets/tests/test_backup_metadata_policy.rs:80-111`, `crates/z00z_simulator/README.md:32-44`. |
| `TASK-049` | Multi-asset expansion has live object-family anchors; bounded closure should use existing Right/Voucher leaves and policy verifier surfaces. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-190`, `crates/z00z_storage/src/settlement/record.rs:156-164`, `crates/z00z_storage/src/settlement/record.rs:258-317`, `crates/z00z_storage/src/settlement/tx_plan_types.rs:51-69`, `crates/z00z_storage/src/settlement/object_package_contract.rs:47-94`. |
| `TASK-050` | Fee envelope and rights wallet extension work has live storage and wallet guide anchors; do not add a second wallet authority plane. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-195`, `crates/z00z_storage/src/settlement/record.rs:511-527`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:31-74`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs:27-71`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs:80-117`. |
| `TASK-051` | Voucher scenario closure should reuse `VoucherLeaf`, `VoucherAction`, object package verifier, and simulator object lanes. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-194`, `crates/z00z_storage/src/settlement/record.rs:283-317`, `crates/z00z_storage/src/settlement/tx_plan_types.rs:51-69`, `crates/z00z_storage/src/settlement/object_package_contract.rs:215-305`, `crates/z00z_simulator/README.md:62-93`. |
| `TASK-052` | Payroll/B2B/useful-work scenarios should remain local object/right/voucher evidence, not external oracle or live service truth. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-194`, `crates/z00z_storage/src/settlement/object_package_contract.rs:16-94`, `crates/z00z_storage/src/settlement/object_package_contract.rs:260-420`, `crates/z00z_simulator/README.md:62-93`. |
| `TASK-053` | Local adapter work should use mock DA/runtime boundaries and must not schedule live chains or live DA. | `.planning/phases/Z00Z-IMPL-PHASES.md:105-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:152-193`, `crates/z00z_rollup_node/src/da.rs:1-17`, `crates/z00z_rollup_node/src/runtime.rs:17-82`, `crates/z00z_runtime/aggregators/README.md:14-22`. |
| `TASK-054` | Cross-chain input modeling must stay local adapter semantics and publication/replay evidence; no external custody truth is implied. | `.planning/phases/Z00Z-IMPL-PHASES.md:105-110`, `crates/z00z_rollup_node/src/da.rs:13-17`, `crates/z00z_runtime/aggregators/src/service.rs:21-48`, `crates/z00z_runtime/watchers/src/publication.rs:31-82`, `crates/z00z_simulator/README.md:4-30`. |
| `TASK-055` | Agentic rights simulations should start from live right classes and wallet object inventory, then add local scenario evidence only. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-197`, `crates/z00z_core/src/assets/assets_config.yaml:122-225`, `crates/z00z_storage/src/settlement/record.rs:156-164`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:39-47`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs:27-71`. |
| `TASK-056` | Machine capability simulations must use `RightClass::MachineCapability` and local object package verdict tests before simulator scenarios. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-197`, `crates/z00z_core/src/assets/assets_config.yaml:122-147`, `crates/z00z_storage/src/settlement/record.rs:156-164`, `crates/z00z_storage/src/settlement/object_package_contract.rs:406-420`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs:40-125`. |
| `TASK-057` | Multi-asset bounded closeout should verify known object families, unknown-policy quarantine, and no voucher/right-as-cash leak. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-190`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:13-30`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:48-74`, `crates/z00z_storage/src/settlement/object_package_contract.rs:71-94`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs:40-196`. |
| `TASK-058` | Adapter closeout must prove local/mock status and no live chain/DA/testnet scheduling. | `.planning/phases/Z00Z-IMPL-PHASES.md:105-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:152-193`, `crates/z00z_rollup_node/src/da.rs:1-17`, `crates/z00z_rollup_node/src/runtime.rs:51-82`, `crates/z00z_simulator/README.md:4-30`. |
| `TASK-059` | Voucher/payroll/B2B evidence must be bounded to existing object-flow artifacts and negative evidence cases. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-194`, `crates/z00z_simulator/README.md:62-93`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs:40-196`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs:198-337`. |
| `TASK-060` | Fee/right wallet closeout must prove `wallet.object.*` owns typed inventory and `wallet.asset.*` remains cash-only. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-195`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:48-74`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:76-107`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs:27-117`, `crates/z00z_storage/src/settlement/record.rs:511-527`. |
| `TASK-061` | Agentic closeout must bind agent budget/service/data-access profiles to live right fixtures and local policy verdicts. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-197`, `crates/z00z_core/src/assets/assets_config.yaml:122-225`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:39-47`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs:89-125`, `docs/Z00Z-Litepaper.md:19-22`. |
| `TASK-062` | Machine capability closeout must prove bounded right usage, replay/missing-right failures, and no full wallet authority grant. | `.planning/phases/Z00Z-IMPL-PHASES.md:152-197`, `crates/z00z_core/src/assets/assets_config.yaml:122-147`, `crates/z00z_storage/src/settlement/object_package_contract.rs:406-420`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs:198-337`, `docs/Z00Z-Litepaper.md:203-208`. |
| `TASK-063` | Final spec-gap pass must classify live, compatibility, future, research, and out-of-scope terms after all gap tasks settle. | `.planning/phases/Z00Z-IMPL-PHASES.md:526-564`, `.planning/phases/Z00Z-IMPL-PHASES.md:587-624`, `crates/z00z_crypto/src/protocol/zkpack.rs:1-70`, `crates/z00z_core/src/assets/version.rs:1-23`, `crates/z00z_storage/src/settlement/root_types.md:8-25`. |
| `TASK-064` | Residual hardening must audit secret reveal, backup/export metadata, and public wallet-id caveats against existing tests. | `.planning/phases/Z00Z-IMPL-PHASES.md:526-564`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-315`, `crates/z00z_wallets/tests/test_backup_metadata_policy.rs:80-111`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs:64-88`. |
| `TASK-065` | Documentation normalization must remove current-protocol claims for future proof systems and field-native packs. | `.planning/phases/Z00Z-IMPL-PHASES.md:526-564`, `.planning/phases/Z00Z-IMPL-PHASES.md:734-943`, `crates/z00z_crypto/src/protocol/zkpack.rs:1-70`, `crates/z00z_core/src/assets/leaf.rs:266-279`. |
| `TASK-066` | Cross-crate residual tests must remain owned by the crate that owns the rule; simulator tests prove integration after crate tests. | `.planning/phases/Z00Z-IMPL-PHASES.md:587-624`, `crates/z00z_storage/tests/test_live_guardrails.rs:1-120`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs:251-315`, `crates/z00z_simulator/README.md:46-60`. |
| `TASK-067` | Final closure docs must ensure task status, source section closure, and phase file annotations match actual completed tests. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:152-197`, `.planning/phases/TODO-gaps.md:2407-2435`, `.planning/phases/TODO-gaps.md:2535-2549`. |
| `TASK-068` | Final broad verification should run only after targeted crate/simulator evidence exists and should report exact failed command if blocked. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:587-624`, `.planning/phases/TODO-gaps.md:92-132`, `crates/z00z_simulator/README.md:46-60`. |
| `TASK-069` | Final drift check must reconcile TODO-gaps, phase source, root contracts, wallet authority, package hygiene, and simulator evidence. | `.planning/phases/Z00Z-IMPL-PHASES.md:526-564`, `.planning/phases/TODO-gaps.md:2407-2435`, `crates/z00z_storage/src/settlement/root_types.md:132-144`, `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md:109-135`, `crates/z00z_simulator/README.md:4-30`. |
| `TASK-070` | Final completion note must be written only after all source-section tasks are complete and residual hardening has no unowned gaps. | `.planning/phases/Z00Z-IMPL-PHASES.md:49-110`, `.planning/phases/Z00Z-IMPL-PHASES.md:152-197`, `.planning/phases/TODO-gaps.md:82-132`, `.planning/phases/TODO-gaps.md:2407-2435`, `.planning/phases/TODO-gaps.md:2535-2549`. |

## 🚨 Blocker Note Template

Use this exact template if a task cannot be completed.

```md
Blocker:
- Date: YYYY-MM-DD
- Task: TASK-000
- Command: `exact command`
- Error essence: one sentence
- Owner file: `path/to/file.rs`
- Next action: one exact action
- Status: blocked | needs decision | unrelated failure
```

## ✅ Completion Note Template

Use this exact template when a task is complete.

```md
Completion:
- Date: YYYY-MM-DD
- Task: TASK-000
- Files changed:
  - `path/to/file.rs`
- Tests run:
  - `exact command` -> passed
- Closeout evidence:
  - `path/to/evidence.rs`
```
