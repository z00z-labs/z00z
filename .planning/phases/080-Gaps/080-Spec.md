# Z00Z Wallet Extensions Implementation Specification

[TOC]

Version: 2026-07-04

Status: canonical Phase 080 implementation specification.

Owner crates: `z00z_wallets`, `z00z_storage`

Integration crates: `z00z_core`, `z00z_rollup_node`

Primary authority surfaces:

- encrypted wallet database: `.wlt`
- wallet export bundle: `WalletExportPack`
- live transaction history sidecar: `wallet_<stem>_tx_history.jsonl`
- wallet receive authority: `WalletService::recv_range(...)` / `recv_range_authoritative(...)`
- settlement object policy inspection: `ObjectPolicyRegistryV1` + `RuntimeObjectPackageV1`

## 1. Purpose And Scope

Phase 080 implements wallet extensions that make Z00Z safer and easier to use without creating a second wallet authority, a tracing backdoor, a general scripting VM, or a new settlement theorem.

The work accepted into this phase is:

1. wallet-local privacy linting and action reports;
2. action-recipe presentation for `Pay`, `Claim`, `Use`, `Delegate`, and `Prove`;
3. scoped disclosure evidence over existing object and right-policy seams;
4. wallet policy mode over the existing object policy registry seam;
5. typed remote scan evidence while keeping wallet-local receive authority;
6. local package-build transcripts for audit and debugging;
7. wallet-local compaction policy for non-authoritative housekeeping;
8. canonical `ValidatorMandate` lock workflow over `RightLeaf`;
9. measurement-led storage proof locality hooks that support wallet scan evidence.

The phase MUST preserve the current rule that clean cash stays on `wallet.asset.*`, while vouchers and rights stay on `wallet.object.*`. Native Z00Z cash MUST NOT become arbitrary programmable value.

## 2. Reader Contract

After reading this specification, an engineer SHOULD be able to answer:

| Question | Answer |
| --- | --- |
| Which wallet surfaces are authoritative? | `.wlt`, `WalletExportPack`, and `wallet_<stem>_tx_history.jsonl`. |
| Can remote scan workers claim ownership? | No. They return advisory evidence only. |
| Can compliance or disclosure expose a full wallet history? | No. Disclosure is scoped, receipted, and bounded to an approved field set. |
| Should `stake_assets` become the canonical staking primitive? | No. The canonical lock primitive is `RightLeaf::ValidatorMandate` with wallet-owned lock inventory and approved transitions. |
| Should Phase 080 add a general rights VM? | No. Use bounded policy descriptors and deterministic validation only. |
| Should wallet compaction rewrite protocol truth? | No. It is local housekeeping only. |

## 3. Key Terms

| Term | Meaning in this specification |
| --- | --- |
| Wallet authority | The durable wallet-local source that may affect reopen, receive, spend, export, or restore behavior. |
| Advisory evidence | Remote or helper-supplied data that the wallet may validate and consume, but that cannot mutate wallet state by itself. |
| `PrivacyReport` | A wallet-local report produced before package emission and after receive scan. It contains warnings and blocking conditions, but it is not a public settlement artifact. |
| Action recipe | A user-facing action category that maps protocol objects to one simple operation: `Pay`, `Claim`, `Use`, `Delegate`, or `Prove`. |
| `EvidencePackage` | A wallet-retained, non-authoritative record that binds a business or compliance document hash to a scoped proof request. |
| `DisclosureReceipt` | A holder-controlled receipt that records exactly what was disclosed, for what purpose, under which policy, and until when. |
| `AuditViewProof` | A bounded proof over commitments, roots, paths, policy IDs, and selected field commitments. It MUST NOT be a wallet-history export. |
| `WalletPolicyMode` | A local wallet mode that requires known policy IDs and permitted actions before right or object transitions. |
| `RemoteScanProofHintV1` | A versioned advisory proof-hint envelope bound to returned chunks, checkpoint identity, and root context. |
| Package transcript | A wallet-local build record for a package. It helps diagnose and audit construction without becoming public settlement input. |
| `ValidatorMandate` lock | A `RightLeaf` profile that encumbers a wallet-owned asset and removes it from ordinary spend selection until an approved transition is available. |

## 4. Current Code Truth

The implementation plan MUST be based on these live surfaces:

| Surface | Current truth | Phase 080 implication |
| --- | --- | --- |
| `crates/z00z_wallets/src/chain/scan_engine.rs` | `RemoteScanEvidence` contains chunks, opaque `RemoteScanProofHint`, and optional resume hints. The trait is evidence-only. | Upgrade proof hints to a typed V1 envelope. Do not change remote workers into receive authorities. |
| `crates/z00z_wallets/src/services/wallet_actions_receive.rs` | `recv_range_authoritative(...)` derives receiver keys, scans chunks, persists owned assets and cursor atomically, and rejects malformed remote evidence. | Keep this lane canonical. New evidence fields must feed this lane only. |
| `crates/z00z_wallets/src/receiver/request.rs` | `PaymentRequest::validate_all(...)` checks version, chain, expiry, signature, and TOFU/pinning outcome. | Privacy linting should reuse this gate for request freshness, first contact, and identity drift warnings. |
| `crates/z00z_wallets/src/tx/tx_wire.rs` | `TxPackage`, `TxInputWire`, `TxOutputWire`, `TxOutRole`, spend proof, and package metadata are canonical wallet transaction wire types. | Package transcripts and privacy reports must wrap or annotate this flow locally, not alter package digest semantics. |
| `crates/z00z_wallets/src/persistence/tx_storage_impl.rs` | The JSONL sidecar is the live append-only transaction history lane and validates the `wallet_<stem>_tx_history.jsonl` path. | Compaction must not replace or silently rewrite this authority. |
| `crates/z00z_wallets/src/wallet/persistence_types.rs` | `WalletExportPack` carries profile, assets, objects, scan state, TOFU pins, keys, and `tx_history_plane`. | New wallet evidence must either remain non-authoritative or be explicitly carried in a versioned export extension. |
| `crates/z00z_storage/src/settlement/record.rs` | `SettlementLeaf::Right(RightLeaf)` and `RightClass` are live settlement families. | New rights work should use `RightLeaf`, not a new `StakeLeaf` or `ReserveLeaf`. |
| `crates/z00z_storage/src/settlement/object_package_contract.rs` | `ObjectPolicyRegistryV1`, `RuntimeObjectPackageV1`, right witnesses, replay checks, and disclosure commitment requirements already exist. | Phase 080 should build wallet-facing policy mode and disclosure receipts around this seam. |
| `crates/z00z_wallets/src/rpc/object_rpc_impl.rs` | Object RPC constructs a local `ObjectPolicyRegistryV1` from request descriptors for inspection. | Wallet policy mode can require registered descriptors before object actions. |
| `crates/z00z_core/configs/devnet_genesis_config.yaml` | Devnet policy profiles already name retention, disclosure receipt, purpose, expiry, document-hash, and checkpoint-anchor requirements. | Treat these as policy/profile configuration seeds, not as evidence-package wallet implementation. |
| `crates/z00z_wallets/src/tx/spend_rules.rs` | `validator_mandate_lock_v1` helpers already derive and match lock commitments. | Finish the wallet-facing lock workflow instead of treating compatibility `stake_assets` as canonical. |
| `crates/z00z_wallets/src/rpc/storage_rpc_impl.rs` | Current compaction only removes obvious temporary files and reports aggregate file-layout stats. | Extend compaction through a conservative policy and dry-run plan, not through protocol-state pruning. |
| `crates/z00z_wallets/src/rpc/asset_rpc_server_catalog.rs` | `merge_assets` already builds a live `TxPackage` through `local_mutation_exec(...)`. | Do not carry the old `stub_tx_merge` backlog item as active Phase 080 work. Keep regression tests. |
| `crates/z00z_storage/src/backend/mod.rs` | Low-level backend contracts `ReadTxn`, `WriteTxn`, `StorageBackend`, and `JournalBackend` already exist below `settlement::SettlementTreeBackend`. | Wallet extension work MUST use settlement/checkpoint semantic facades and MUST NOT expose backend contracts as wallet authority. |
| `crates/z00z_storage/src/backend/redb/*` and `crates/z00z_storage/src/backend/memory/*` | Redb and memory backend lanes already exist under the backend seam. | Treat the backend split as landed baseline. Do not reintroduce `settlement/redb_backend` or add a `rocksdb` stub in Phase 080. |
| `crates/z00z_storage/src/settlement/store.rs` | `SettlementStore` owns semantic settlement state and `SettlementTreeBackend` remains the semantic proof/root contract. | Phase 080 proof, policy, disclosure, and scan-evidence work must preserve one settlement root lineage and public proof compatibility. |
| `crates/z00z_runtime/aggregators/src/batch_planner.rs` | `BatchPlanner`, route-table validation, and `BatchPlanned` digest logic are runtime-owned. | If Phase 080 touches runtime admission, use these runtime surfaces and do not move store-local dry-run or rollback helpers out of storage. |
| `crates/z00z_runtime/aggregators/src/placement.rs` and `crates/z00z_runtime/aggregators/src/shard_exec.rs` | `AggregatorId`, `ShardPlacementTable`, `ShardPlacementView`, `ShardExecutor`, and `ShardExecTicket` are live runtime placement/execution surfaces. | Placement metadata may guide runtime orchestration, but it MUST NOT become wallet, validator, or watcher semantic truth. |
| `crates/z00z_runtime/validators/src/lib.rs` and `crates/z00z_runtime/watchers/src/lib.rs` | `ValidatorService`, `ValidatorBoundary`, `WatcherService`, and `WatcherBoundary` are stable facade exports over already resolved or published runtime state. | Phase 080 MUST NOT make validators or watchers planner authorities. |
| `crates/z00z_rollup_node/src/runtime.rs` | `NodeRuntime` composes aggregator, validator, watcher, DA, placement, publication, and status services. | Rollup node remains composition/orchestration root and MUST NOT become wallet-state authority. |

## 5. Invariants

### ZINV-WALLET-EXT-001

Wallet extension features MUST NOT introduce a second wallet authority. `.wlt`, `WalletExportPack`, and the explicit tx-history JSONL sidecar remain the only live wallet-state authority surfaces.

### ZINV-WALLET-EXT-002

Remote scan workers, helpers, auditors, compliance reviewers, sponsor programs, and app integrations MUST NOT mutate wallet receive state or claim asset ownership. They may provide advisory evidence only.

### ZINV-WALLET-EXT-003

Clean cash MUST remain ordinary cash. Voucher, right, fee, proof, budget, access, claim, and disclosure semantics MUST stay outside `wallet.asset.*` and use object or tx-specific surfaces.

### ZINV-WALLET-EXT-004

Disclosure MUST be scoped, purpose-bound, expiring, and holder- or policy-approved. No Phase 080 feature may introduce a universal disclosure key or hidden surveillance authority.

### ZINV-WALLET-EXT-005

Right policies MUST be deterministic, bounded, and fail closed. No general VM, unbounded script, dynamic network call, or dependency-specific script type may become a public wallet or settlement API.

### ZINV-WALLET-EXT-006

Cross-crate refactor boundaries MUST preserve one semantic settlement authority. Backend seams, runtime planner surfaces, watcher projections, validator verdicts, and rollup composition MUST NOT create parallel wallet, settlement, checkpoint, or proof authority.

## 6. Workstream 080-A: Wallet Privacy Reports

### 6.1 Recommendation

Implement a local `PrivacyReport` and stable `PrivacyWarning` enum under the wallet transaction and receive boundary. The report should run before emitting a `TxPackage` and after receive scan imports candidate assets.

This is the highest-value first step because the current wallet already has strong request validation and transaction package boundaries, but user or app behavior can still create privacy loss through input merge, linkable change, stale requests, or remote-helper confusion.

### 6.2 Required Data Shapes

```text
PrivacyReportV1:
  version
  action_kind
  warnings[]
  blocking_reason?
  related_tx_digest?
  related_request_id?
  local_only = true

PrivacyWarningV1:
  kind
  severity
  subject_ref?
  evidence_ref?
```

Required warning kinds:

| Warning kind | Trigger | Expected default |
| --- | --- | --- |
| `input_merge_risk` | package builder selects unrelated receive contexts in one transaction | warn or require confirmation |
| `linkable_change` | `TxOutRole::Change` is likely linkable to input ownership | warn |
| `expired_request` | `PaymentRequest::is_expired()` is true | block |
| `near_expiry_request` | request is close to expiry threshold | warn |
| `first_contact_receiver` | `ValidationOutcome::RequiresUserConfirmation` | require confirmation |
| `receiver_identity_drift` | `ValidationOutcome::IdentityMismatch` | block unless explicit rotation flow exists |
| `remote_evidence_only` | receive result came from helper evidence | informational, never authority |
| `worker_evidence_rejected` | remote chunk/proof/resume validation failed | block remote-assisted path |
| `locked_asset_excluded` | validator mandate lock removed an asset from spendable selection | informational |

### 6.3 Implementation Tasks

- **080-A-001**: Add `PrivacyReportV1`, `PrivacyWarningV1`, `PrivacyWarningKindV1`, and severity enum in `z00z_wallets`.
- **080-A-002**: Wire pre-send linting into transaction build paths before package bytes are returned or broadcast.
- **080-A-003**: Wire post-receive linting into `recv_range_authoritative(...)` after imported assets and scan outcome are known.
- **080-A-004**: Keep reports local by default. Do not write privacy telemetry to public settlement, logs, or tx packages unless a future explicit export is added.
- **080-A-005**: Expose report summaries through RPC responses where the caller naturally expects action feedback.

### 6.4 Acceptance Criteria

- **AC-080-A-001**: Given an expired payment request, when the wallet attempts to build a payment, then the action is blocked and `expired_request` is reported.
- **AC-080-A-002**: Given a new receiver identity, when the wallet validates a request, then `first_contact_receiver` is reported and the package is not silently built.
- **AC-080-A-003**: Given an identity mismatch, when the wallet validates a request, then the report contains `receiver_identity_drift` and the action fails closed.
- **AC-080-A-004**: Given helper-provided scan evidence, when the receive path imports an asset, then the report identifies remote provenance without treating the helper as owner.
- **AC-080-A-005**: No `PrivacyReport` field changes the canonical `TxPackage` digest.

## 7. Workstream 080-B: Action Recipes

### 7.1 Recommendation

Implement a thin wallet-facing action vocabulary:

```text
Pay
Claim
Use
Delegate
Prove
```

The protocol layer may still use `Asset`, `Voucher`, `Right`, `FeeEnvelope`, `TxPackage`, `ClaimTxPackage`, and checkpoint evidence. Normal wallet users and partner app integrations should see action recipes, not raw protocol vocabulary.

### 7.2 Required Mapping

| User action | Internal families | Implementation rule |
| --- | --- | --- |
| `Pay` | clean `Asset` + `TxPackage` | no custom policy on native cash |
| `Claim` | voucher or claim package | one claim, replay-safe, expiry-aware |
| `Use` | `RightLeaf` with service/data/machine capability | provider- and expiry-bounded |
| `Delegate` | `RightLeaf` or budget envelope | max budget, max duration, provider scope |
| `Prove` | disclosure receipt and audit proof | minimum field set, no history export |

Every recipe response SHOULD also carry an action status:

| Status | Meaning |
| --- | --- |
| `ready` | the wallet can build the action now under current policy |
| `needs_confirmation` | the wallet has enough data, but privacy, TOFU, policy, or external-trust risk requires approval |
| `blocked` | the wallet cannot safely build the action and MUST return a stable reason |

### 7.3 Implementation Tasks

- **080-B-001**: Add `RuntimeWalletActionKind` with the five action values above.
- **080-B-002**: Add recipe metadata to wallet RPC responses for send, claim, object use, delegation, and proof/disclosure endpoints.
- **080-B-003**: Hide protocol nouns from end-user fields unless the endpoint is explicitly expert/debug-facing.
- **080-B-004**: Represent fee status as `included`, `sponsored`, `deducted`, or `external`, without giving ordinary users or agents unrestricted fee wallets.
- **080-B-005**: Keep partner-app copy and API schemas honest about external dependencies such as lockers, issuers, bridges, and redemption providers.
- **080-B-006**: Add stable recipe status output so callers can render one primary action with `ready`, `needs_confirmation`, or `blocked` state.

### 7.4 Acceptance Criteria

- **AC-080-B-001**: A private payment can be represented as `Pay` without exposing `AssetLeaf`, `CheckpointArtifact`, or proof internals in the normal response.
- **AC-080-B-002**: A reward flow can be represented as `Claim` and still carry replay and expiry protection.
- **AC-080-B-003**: A bounded agent budget can be represented as `Delegate` without granting full wallet authority.
- **AC-080-B-004**: Expert/debug APIs may expose protocol terms, but normal recipe responses do not require users to choose raw right policy fields.
- **AC-080-B-005**: Private Pay, Private Claim, and Private Budget flows each return one action kind, one fee status, and one stable action status.

## 8. Workstream 080-C: Scoped Disclosure And Evidence

### 8.1 Recommendation

Build scoped disclosure as wallet-owned evidence over existing settlement object and policy seams. Do not build a universal view key.

The first implementation should support one or two right classes, preferably `DataAccess` and `ServiceEntitlement`, before broad corporate finance claims are attempted.

### 8.2 Required Data Shapes

```text
ComplianceProfileIdV1:
  profile_id
  canonical_profile_hash
  display_name
  allowed_right_classes[]
  allowed_field_sets[]

EvidencePackageV1:
  version
  profile_id
  subject_commitment
  action_kind
  policy_id
  field_set_commitment
  retained_document_hashes[]
  checkpoint_anchor?
  expires_at?

DisclosureReceiptV1:
  version
  disclosure_id
  profile_id
  purpose
  field_set_commitment
  policy_id
  retention_policy_id
  evidence_package_hash
  disclosed_at
  expires_at
  holder_signature

AuditViewRequestV1:
  version
  requester_commitment
  profile_id
  requested_field_set
  purpose
  expiry
  request_signature

AuditViewProofV1:
  version
  receipt_id
  policy_id
  settlement_path_commitment
  root_commitment
  checkpoint_anchor?
  proof_bytes
```

### 8.3 Implementation Tasks

- **080-C-001**: Add wallet-owned `EvidencePackageV1` and `DisclosureReceiptV1` types.
- **080-C-002**: Add `AuditViewRequestV1` validation. The request is not authority by itself.
- **080-C-003**: Add a holder-approval or wallet-policy approval gate before generating any disclosure response.
- **080-C-004**: Add a minimal `AuditViewProofV1` skeleton that binds policy IDs, selected fields, root/path commitments, and optional checkpoint anchors.
- **080-C-005**: Store evidence in an encrypted wallet-owned archive or `.wlt` extension. It MUST NOT be required for normal reopen unless the wallet has enabled a compliance profile.
- **080-C-006**: If evidence is exportable, add a versioned `WalletExportPack` extension or manifest reference. Do not create a second canonical export bundle.

### 8.4 Acceptance Criteria

- **AC-080-C-001**: A disclosure proof reveals only the requested field set and does not expose unrelated assets, rights, tx history, or wallet inventory.
- **AC-080-C-002**: A disclosure receipt records purpose, field set, policy, retention, expiry, and holder authorization.
- **AC-080-C-003**: Unknown or expired audit requests fail closed.
- **AC-080-C-004**: Evidence package loss cannot corrupt cash balance, scan cursor, tx history, or object inventory.

## 9. Workstream 080-D: Wallet Policy Mode

### 9.1 Recommendation

Use the existing `ObjectPolicyRegistryV1` and policy descriptor machinery as the registry seam. Phase 080 should add wallet-facing policy mode and profile enforcement rather than inventing a second policy registry.

### 9.2 Required Behavior

- Wallet policy mode MUST require deterministic policy bytes, stable policy IDs, and known action descriptors before object transitions.
- Unknown policy IDs MUST keep non-asset objects quarantined or unavailable.
- Policy IDs MUST be commitments to rules, not mutable off-chain labels.
- The evaluator MUST stay deterministic and bounded.
- Policy validation MUST enter storage through facade seams, not raw HJMT/JMT internals.

### 9.3 Implementation Tasks

- **080-D-001**: Add `WalletPolicyModeV1` with at least `off`, `warn`, and `enforce`.
- **080-D-002**: Add wallet configuration and RPC surfaces to set policy mode per wallet or account.
- **080-D-003**: Bind `WalletPolicyModeV1::enforce` to `ObjectPolicyRegistryV1` inspection before object use, transfer, delegation, disclosure, revoke, consume, or challenge flows.
- **080-D-004**: Map object policy failures to stable wallet-facing reject codes.
- **080-D-005**: Keep `wallet.asset.*` cash paths from absorbing voucher or right semantics.

### 9.4 Acceptance Criteria

- **AC-080-D-001**: In enforce mode, a right with missing policy availability cannot be used or delegated.
- **AC-080-D-002**: In warn mode, the wallet returns a warning but does not silently mutate object state.
- **AC-080-D-003**: Policy descriptor hash drift rejects the object package.
- **AC-080-D-004**: A right carrying declared value units is rejected as `right_used_as_value` or equivalent stable code.

## 10. Workstream 080-E: Typed Remote Scan Evidence

### 10.1 Recommendation

Upgrade opaque remote scan proof hints into typed, versioned advisory envelopes. Keep the wallet-owned scanner and cursor mutation as the only receive authority.

### 10.2 Required Data Shapes

```text
RemoteScanProofKindV1:
  chunk_membership
  checkpoint_link
  witness_availability

RemoteWorkerRefV1:
  worker_id_commitment
  attestation_label?
  signed_at?

RemoteScanProofHintV1:
  version
  checkpoint_height
  checkpoint_id
  chunk_hash
  prev_root
  post_root
  proof_kind
  proof_bytes
  worker_ref?

RemoteScanEvidenceV1:
  version
  chunks[]
  proof_hints[]
  resume_hint?
```

### 10.3 Implementation Tasks

- **080-E-001**: Add `RemoteScanProofHintV1` and `RemoteScanEvidenceV1` while preserving compatibility with the current `RemoteScanEvidence` seam where needed.
- **080-E-002**: Bind every proof hint to one returned chunk by height and chunk hash.
- **080-E-003**: Validate checkpoint ID and root fields when the corresponding data is available.
- **080-E-004**: Preserve existing strict chunk validation: non-empty hashes, strictly increasing heights, contiguous chunks, and resume hints that cannot rewind or set local cursor from origin.
- **080-E-005**: Ensure worker evidence can only feed `recv_range_authoritative(...)`.
- **080-E-006**: Add worker provenance fields for accountability, but do not make worker identity a settlement authority.

### 10.4 Acceptance Criteria

- **AC-080-E-001**: A proof hint whose chunk hash does not match a returned chunk is rejected.
- **AC-080-E-002**: A worker resume hint cannot advance, rewind, or overwrite local cursor authority by itself.
- **AC-080-E-003**: Rejected worker evidence records `worker_evidence_rejected` in receive outcome/reporting.
- **AC-080-E-004**: Local scanning still derives ownership from wallet receiver keys and `StealthOutputScanner`.

## 11. Workstream 080-F: Package Build Transcripts

### 11.1 Recommendation

Add optional local package-build transcripts for debugability and audit. They MUST NOT become public settlement artifacts and MUST NOT require an online receiver.

### 11.2 Required Transcript Fields

```text
PackageBuildTranscriptV1:
  version
  action_kind
  input_selection_hash
  receiver_descriptor_hash?
  payment_request_id?
  output_binding_hash
  spend_statement_hash?
  package_digest
  privacy_report_hash?
  absent_private_fields[]
  created_at
```

### 11.3 Implementation Tasks

- **080-F-001**: Add a transcript builder next to the transaction assembler or build orchestration path.
- **080-F-002**: Hash input selection, receiver descriptor or request, output binding, spend statement, final package digest, and privacy report.
- **080-F-003**: Store transcripts only when requested by local policy, debug mode, or compliance profile.
- **080-F-004**: Redact absent/private fields explicitly so auditors know what was intentionally not retained.
- **080-F-005**: Keep transcript serialization out of `TxPackage` digest inputs.

### 11.4 Acceptance Criteria

- **AC-080-F-001**: A package can be built without a transcript.
- **AC-080-F-002**: Enabling transcripts does not change package bytes or digest.
- **AC-080-F-003**: A transcript can prove which request/card and privacy warnings were used without exposing secret keys or full wallet inventory.

## 12. Workstream 080-G: Wallet-Local Compaction Policy

### 12.1 Recommendation

Extend storage compaction from temporary-file cleanup into a conservative wallet-local housekeeping policy. The live tx-history JSONL lane, `.wlt`, scan cursor, and `WalletExportPack` semantics MUST remain stable.

### 12.2 Allowed Actions

Compaction MAY:

- remove temporary files through `z00z_utils::io` abstractions;
- report reclaimable bytes and dry-run plans;
- archive or mark redundant non-authoritative wallet rows only when a retention policy says they are derivable or no longer live;
- produce a local compaction report;
- update a non-authoritative `last_compact_at` statistic.

Compaction MUST NOT:

- rewrite or truncate the live append-only tx-history JSONL sidecar without a separate archive manifest and restore test;
- change the spendable set;
- change scan cursor semantics;
- create a compact-only export bundle;
- turn wallet-local spent flags into proof of protocol spent status;
- hide required restore state outside `WalletExportPack`.

### 12.3 Implementation Tasks

- **080-G-001**: Add `WalletCompactionPlanV1` with dry-run output before mutation.
- **080-G-002**: Add per-surface statistics for `.wlt`, tx-history JSONL, temporary files, archived rows, assets, vouchers, and rights.
- **080-G-003**: Add retention policy hooks for non-authoritative evidence and archived rows.
- **080-G-004**: Keep compaction atomic at the wallet-file level. Failed compaction must leave reopen, scan state, and tx-history fold unchanged.
- **080-G-005**: Add export/restore regression tests for compacted wallets.

### 12.4 Acceptance Criteria

- **AC-080-G-001**: Running compaction does not change the wallet spendable set before or after reopen.
- **AC-080-G-002**: A compacted wallet exports and restores through the same `WalletExportPack` shape and tx-history plane.
- **AC-080-G-003**: Dry-run and actual compaction report the same candidate surfaces before mutation.
- **AC-080-G-004**: Removing temporary files is recoverable from the live wallet state.

## 13. Workstream 080-H: Validator Mandate Locks

### 13.1 Recommendation

Finish a canonical `ValidatorMandate` lock workflow as a `RightLeaf` profile. The current compatibility `stake_assets` and `unstake_assets` RPC methods are not canonical staking. They may remain compatibility surfaces, but they MUST NOT define settlement authority.

The first version should be non-slashable or challenge-bounded. Full slashing requires a separate proof model, fraud/challenge process, appeal rules, and formal tests.

### 13.2 Required Behavior

`ValidatorMandateLockV1` MUST:

- use `RightClass::ValidatorMandate`;
- use profile label `validator_mandate_lock_v1`;
- bind locked asset ID, locked amount, validity window, challenge window, nonce, transition policy, revocation policy, disclosure policy, and retention policy into `payload_commitment`;
- persist as `OwnedRightPayload`;
- remove the locked asset from ordinary spend selection while the right is active and policy-available;
- allow unlock only through an approved transition after policy and time checks;
- preserve wallet ownership and export/restore of both the asset and lock record.

### 13.3 Implementation Tasks

- **080-H-001**: Add wallet RPC/object action for creating a `ValidatorMandateLockV1` right over an owned asset.
- **080-H-002**: Add wallet RPC/object action for unlocking through the approved transition path.
- **080-H-003**: Reconcile compatibility `stake_assets` responses so they clearly remain compatibility UX and do not bypass the right profile.
- **080-H-004**: Add object inventory views that show lock state, unlock readiness, beneficiary commitment, and policy availability.
- **080-H-005**: Add export/restore coverage for locked assets and their `OwnedRightPayload`.

### 13.4 Acceptance Criteria

- **AC-080-H-001**: A locked asset cannot be selected for ordinary send, merge, split, swap, or local mutation.
- **AC-080-H-002**: An unrelated unlocked asset remains spendable while another asset is locked.
- **AC-080-H-003**: Unlock before `valid_until` or without the required transition policy fails closed.
- **AC-080-H-004**: Export/restore preserves lock status and does not turn the locked asset into spendable cash.
- **AC-080-H-005**: The wallet never reports APY or passive yield as canonical protocol truth for the lock.

## 14. Workstream 080-I: Storage Proof Locality Measurements

### 14.1 Recommendation

Add measurement-led hooks for wallet scan and proof locality only after the typed remote evidence contract is in place. Do not change `SettlementStateRoot` semantics or expose backend roots as public wallet authority.

### 14.2 Implementation Tasks

- **080-I-001**: Add counters for proof generation hot paths relevant to wallet scanning and helper evidence.
- **080-I-002**: Add cache invalidation tests keyed by semantic path plus root generation.
- **080-I-003**: Keep `backend_root` proof-local and private.
- **080-I-004**: Add benchmarks before any cache becomes default.

### 14.3 Acceptance Criteria

- **AC-080-I-001**: A cache entry invalidates on root generation change.
- **AC-080-I-002**: `SettlementStateRoot` remains the public semantic root.
- **AC-080-I-003**: Wallet or rollup APIs do not expose backend roots as authority.

## 15. Workstream 080-J: Cross-Crate Refactor Boundary And Runtime/Storage Alignment

### 15.1 Recommendation

Phase 080 must carry the relevant cross-crate refactor decisions as implementation guardrails. The goal is not to run a broad rename or storage migration inside the wallet phase. The goal is to make wallet extensions integrate with the current runtime/storage topology without reopening old architecture mistakes.

The current baseline already includes the backend seam, redb backend lane, memory backend lane, runtime batch planner, runtime placement table, shard executor, validator facade, watcher facade, and rollup runtime composition root. Phase 080 work must build on those surfaces.

### 15.2 Carried Refactor Decisions

| Area | Self-contained decision | Phase 080 rule |
| --- | --- | --- |
| Rollup node | `z00z_rollup_node` is orchestration and composition only. | Keep wallet authority out of rollup node. Use it only to compose runtime services, DA, status, publication, and placement. |
| Aggregators | Runtime owns batch admission, route-table validation, shard placement, and shard execution through `BatchPlanner`, `BatchPlanned`, `ShardPlacementTable`, `ShardExecutor`, and `ShardExecTicket`. | If wallet object or disclosure flows touch runtime admission, integrate through these surfaces. Do not move all storage `tx_plan` helpers into runtime. |
| Validators | Validators check already resolved runtime batches and object policy verdicts. | Validators may consume placement or exec metadata attached to a batch, but they must not become planner authority. |
| Watchers | Watchers observe published state, provider signals, evidence exports, and status projections. | Watcher output is observational evidence only, not wallet state, planner truth, or settlement authority. |
| Storage backend seam | `z00z_storage::backend` owns raw row and journal contracts below settlement semantics. | Wallet, RPC, proof, policy, and disclosure APIs must not expose raw backend transactions, redb types, backend roots, or `StoreBackendError`. |
| Settlement semantic facade | `SettlementStore` and `SettlementTreeBackend` own semantic root, proof, replay, and model behavior. | Phase 080 must preserve public proof behavior and one root lineage while adding policy/disclosure/scan-evidence hooks. |
| Checkpoint, snapshot, serialization | These are separate surface areas. Snapshot is backup/restore; checkpoint is root/publication lineage; serialization is artifact encoding/restore support. | Do not collapse them into one generic backup layer or bypass them through backend internals. |
| Planner split | Runtime planner authority is separate from store-side semantic dry-run, duplicate-path precheck, rollback, and model helpers. | Keep store-local helpers in storage when they depend on settlement model state. |
| Rename wave | Rename-only cleanup must follow semantic stabilization. | Do not mix Phase 080 wallet behavior with unrelated module/file/public-symbol renames. |
| Proof compatibility | Existing storage proof tests and `settlement_proofs` bench lanes are compatibility guards. | Any proof-locality or disclosure proof work must keep these lanes meaningful and must not weaken malformed/root-version reject behavior. |

### 15.3 Implementation Tasks

- **080-J-001**: Add a Phase 080 boundary checklist to implementation PRs that marks every touched surface as `wallet`, `storage_semantic`, `storage_backend`, `runtime_planner`, `validator`, `watcher`, or `rollup_composition`.
- **080-J-002**: For privacy reports, disclosure, typed remote scan evidence, wallet policy mode, and package transcripts, integrate through wallet APIs and settlement/checkpoint semantic facades only.
- **080-J-003**: Prohibit public wallet/RPC DTOs from exposing `z00z_storage::backend::*`, redb types, memory backend types, raw backend roots, or `StoreBackendError`.
- **080-J-004**: Keep `SettlementStore`, `SettlementTreeBackend`, checkpoint lineage, replay rejection, and public proof behavior stable for any Phase 080 storage-adjacent change.
- **080-J-005**: If Phase 080 touches runtime admission or publication, use `BatchPlanner`, `BatchPlanned`, `ShardPlacementTable`, `ShardExecutor`, and `ShardExecTicket`; do not revive storage-owned runtime planner authority.
- **080-J-006**: Keep validators and watchers downstream from committed or resolved runtime state. They may report verdicts, alerts, or evidence, but must not plan batches, mutate wallet state, or define semantic roots.
- **080-J-007**: Keep `snapshot`, `serialization`, and `checkpoint` facades separate. Wallet export/restore and compaction work must not create a parallel backup layer.
- **080-J-008**: Avoid rename-only churn in Phase 080. File/module/public-symbol renames are allowed only when required by a Phase 080 behavior change and protected by compatibility tests.
- **080-J-009**: Add or maintain guard tests that prove wallet-facing APIs do not leak backend internals and that proof/root semantics remain stable.

### 15.4 Acceptance Criteria

- **AC-080-J-001**: No new public wallet, RPC, or export type exposes backend modules, redb implementation types, raw backend roots, or backend-local errors.
- **AC-080-J-002**: A proof or disclosure path added in Phase 080 binds to settlement/checkpoint semantic roots, not to backend-specific root material.
- **AC-080-J-003**: Runtime planner work, if touched, stays in aggregator surfaces and does not move store-local semantic dry-run or rollback helpers out of storage.
- **AC-080-J-004**: Validator and watcher changes remain downstream of committed or resolved runtime state and cannot mutate wallet receive state.
- **AC-080-J-005**: Storage proof compatibility tests and relevant proof benches still exercise malformed, stale-root, root-version, and mixed-family rejection lanes after Phase 080 storage-adjacent changes.

### 15.5 Rename And Move Table Audit

The cross-crate rename and move inventory was audited against the live repository. Phase 080 MUST treat this table as the current source of truth for whether an old migration row is still actionable.

| Inventory area | Live audit result | Phase 080 decision |
| --- | --- | --- |
| `z00z_rollup_node` DA facade | `src/da_adapter.rs` is gone and `src/da.rs` exists. | Landed. No Phase 080 rename work. |
| `z00z_rollup_node` lifecycle/runtime facade | `src/lifecycle.rs` is gone and `src/runtime.rs` exists. `NodeRuntime` composes aggregator, validator, watcher, DA, placement, publication, and status. | Landed. Keep rollup node as composition root only. |
| `z00z_rollup_node` placeholder files | `src/empty_file`, `bin/empty_file`, `examples/empty_file`, `benches/empty_file`, and `tests/empty_file` are absent. | Landed. Do not recreate placeholders. |
| `z00z_rollup_node` settlement theorem test row | `tests/test_settlement_theorem.rs` is absent; current HJMT/topology/runtime tests cover the live rollout shape. | Superseded. Do not recreate the old test filename solely to satisfy the migration table. |
| Aggregator `agg_*` files | `agg_iface.rs`, `agg_ingress.rs`, `agg_ordering.rs`, `agg_recovery.rs`, `agg_scheduler.rs`, and `agg_types.rs` are gone. `service.rs`, `ingress.rs`, `ordering.rs`, `recovery.rs`, `scheduler.rs`, and `types.rs` exist. | Landed. Public names such as `AggregatorService`, `IngressBoundary`, `OrderingBoundary`, `RecoveryBoundary`, and `SchedulerBoundary` remain stable. |
| Aggregator planner and placement files | `batch_planner.rs`, `placement.rs`, and `shard_exec.rs` exist. `BatchPlanner`, `BatchPlanned`, `AggregatorId`, `ShardPlacementTable`, `ShardExecutor`, and `ShardExecTicket` are live exports. | Landed. Use these surfaces if Phase 080 touches runtime admission or publication. |
| Aggregator placeholder files | `bin/empty_file`, `examples/empty_file`, `benches/empty_file`, and `tests/empty_file` are absent. | Landed. Do not recreate placeholders. |
| Validator file renames | `artifact_decode.rs`, `checkpoint_flow.rs`, `claim_nulls.rs`, `claim_pkg_verify.rs`, `reconcile_rules.rs`, `spend_rules.rs`, `tx_pkg_verify.rs`, `val_engine.rs`, and `verdicts.rs` are gone. `artifact.rs`, `checkpoint.rs`, `nullifier.rs`, `claim_verify.rs`, `reconcile.rs`, `spend.rs`, `tx_verify.rs`, `engine.rs`, and `verdict.rs` exist. | Landed. Keep `ValidatorService` and `ValidatorBoundary` stable. |
| Validator placeholder files | `bin/empty_file`, `examples/empty_file`, `benches/empty_file`, and `tests/empty_file` are absent. | Landed. Do not recreate placeholders. |
| Watcher file renames | `censorship_watch.rs`, `provider_compare.rs`, `publication_watch.rs`, `status_view.rs`, and `watcher_engine.rs` are gone. `censorship.rs`, `provider.rs`, `publication.rs`, `status.rs`, and `engine.rs` exist. | Landed. Keep `WatcherService`, `WatcherBoundary`, and `WatcherInput` stable. |
| Watcher placeholder files | `bin/empty_file`, `examples/empty_file`, `benches/empty_file`, and `tests/empty_file` are absent. | Landed. Do not recreate placeholders. |
| Storage backend seam | `src/backend/mod.rs` and `src/backend/error.rs` exist with `ReadTxn`, `WriteTxn`, `StorageBackend`, and `JournalBackend`. | Landed. Do not expose this seam through wallet APIs. |
| Storage redb adapter move | `src/backend/redb/{mod,helpers,hjmt,state,validate}.rs` exist and `src/settlement/redb_backend/mod.rs` is absent. | Landed. Do not reintroduce `settlement/redb_backend`. |
| Storage backend helper move | `src/backend/{codec,query,roots,rows,types}.rs` exist. Old `src/settlement/store/store_*.rs` helpers are absent. | Landed. Keep backend helpers below settlement semantics. |
| Storage memory backend | `src/backend/memory.rs` exists as a flat module, not `src/backend/memory/mod.rs`. | Landed with flat layout. Do not require a directory move for Phase 080. |
| Storage rocksdb adapter | `src/backend/rocksdb/mod.rs` is absent. | Deferred. Do not add a stub in Phase 080. |
| Storage feature flags | `z00z_storage/Cargo.toml` does not split `redb` and `memory` into backend feature flags; `redb` is still a direct dependency and memory is internal. | Open but not Phase 080 work. Phase 080 MUST NOT depend on new backend feature flags. |
| Checkpoint facade and helpers | `checkpoint/mod.rs`, `artifact_*`, `build.rs`, `build_prepare.rs`, `build_state.rs`, `store.rs`, and `store_fs.rs` exist. Proposed shorter names such as `prepare.rs`, `state.rs`, or `fs.rs` are not live. | Keep current names during Phase 080. Rename-only cleanup is out of scope. |
| Snapshot facade | `snapshot/mod.rs`, `codec.rs`, `error.rs`, `store.rs`, `test_snapshot.rs`, and `types.rs` exist. `snapshot/store/tests.rs` is absent. | Landed/superseded. Keep snapshot as backup/restore facade; do not create `SnapshotBackend` for Phase 080. |
| Serialization temp-tree helper | `serialization/temp_tree.rs` exists. Both old duplicate locations `serialization/build/temp_tree.rs` and `serialization/build_temp_tree.rs` are absent. | Duplicate resolved. Keep current live layout; no Phase 080 rename. |
| Settlement README casing | `settlement/README.md` exists and `settlement/README.MD` is absent. | Landed. |
| Settlement root-types naming | `settlement/root_types.md` exists and `settlement/root-types.md` is absent. | Landed. |
| Settlement type files | `identity.rs`, `query.rs`, and `record.rs` exist. Old `types_identity.rs`, `types_query.rs`, and `types_record.rs` are absent. | Landed. |
| Settlement store file | `settlement/store.rs` exists and `settlement/settlement_store.rs` is absent. | Keep current live name. File rename is not required for Phase 080 and would be rename-only churn. |
| Settlement planner split | Runtime `batch_planner.rs` exists. Storage retains `tx_plan_help.rs` and `tx_plan_types.rs` for store-side helpers. | Landed in split form. Do not move store-local semantic helpers out of storage for Phase 080. |
| Settlement HJMT file names | `hjmt_cache.rs`, `hjmt_commit.rs`, `hjmt_config.rs`, `hjmt_journal.rs`, `hjmt_plan.rs`, `hjmt_policy.rs`, `hjmt_proof.rs`, `hjmt_scheduler.rs`, and `hjmt_store.rs` exist as flat files. | Keep current live names. Dropping prefixes is rename-only churn outside Phase 080. |
| Storage tests and proof benches | Guardrail tests and proof benches exist, including `test_layout_guardrails.rs`, `test_downstream_guardrails.rs`, `test_live_guardrails.rs`, `test_default_gate.rs`, and `benches/settlement_proofs.rs`. | Keep as compatibility gates for Phase 080 storage-adjacent work. |

Implementation conclusion: the rename/move table has no remaining mandatory Phase 080 rename task. Remaining open items are either already landed, superseded by current file names, deliberately deferred backend work, or rename-only cleanup that must not be mixed into wallet-extension implementation.

## 16. Not Carried Into Phase 080

The following ideas are not Phase 080 implementation tasks:

| Idea | Reason |
| --- | --- |
| Offline transferable bearer cash | Requires double-spend accountability and checkpoint reconciliation spec first. |
| Market-regime treasury subsidies, fee-support vaults, and anonymous evaluator committees | Tokenomics and governance surface, not wallet extension baseline. |
| `Proof of Non-Control`, `Proof of Non-Occurrence`, solvency/liability overlays | Valuable future product lines, but each needs its own spec and proof boundary. |
| Continuity rights, underwriting markets, workflow fabric, privacy-budget public good | Strategic concepts, not sufficiently bounded for this wallet phase. |
| General scripting VM or TariScript clone | Violates bounded deterministic policy requirement. |
| Master trace key or regulator backdoor | Violates scoped disclosure and privacy invariants. |
| New `StakeLeaf`, `EscrowLeaf`, `ReserveLeaf`, or similar leaf families | Use `RightLeaf` profiles and policy descriptors first. |
| Bulk storage/backend/runtime rename wave | The relevant runtime/storage guardrails are carried in Workstream 080-J. Broad rename-only cleanup is not required for wallet extensions. |
| New `rocksdb` adapter or stub | Add only when a real adapter and tests exist. Do not create placeholder backend surface in Phase 080. |

## 17. Module Placement

Recommended placement:

| Feature | Primary placement | Notes |
| --- | --- | --- |
| Privacy reports | `crates/z00z_wallets/src/tx/` and receive/RPC types | Local reports only; avoid tx digest changes. |
| Action recipes | `crates/z00z_wallets/src/rpc/*_types.rs` and wallet action orchestration | User-facing DTO layer. |
| Evidence packages and disclosure receipts | `crates/z00z_wallets/src/rpc/object_*`, wallet persistence extension, or dedicated wallet evidence module | Encrypted wallet-owned evidence; non-authoritative. |
| Wallet policy mode | `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`, wallet config, and `z00z_storage::settlement::ObjectPolicyRegistryV1` integration | Use existing registry seam. |
| Remote scan evidence V1 | `crates/z00z_wallets/src/chain/scan_engine.rs` and receive validation | Advisory evidence only. |
| Package transcripts | transaction assembler/build orchestration | Optional and local. |
| Compaction plan | `crates/z00z_wallets/src/rpc/storage_*` and `WalletService` reachability hooks | Conservative local housekeeping. |
| Validator mandate locks | `crates/z00z_wallets/src/tx/spend_rules.rs`, object inventory, asset RPC/object RPC | Prefer object action path for canonical behavior. |
| Storage locality measurements | `crates/z00z_storage/src/settlement/*` tests/benches | No public semantic root changes. |
| Runtime/storage refactor guardrails | `crates/z00z_storage/src/backend/*`, `crates/z00z_storage/src/settlement/*`, `crates/z00z_runtime/aggregators/src/*`, validators/watchers facades, and `z00z_rollup_node::runtime` | Use as boundaries and compatibility guards, not as a wallet authority layer. |

## 18. Test And Validation Plan

### 18.1 Unit Tests

- `PrivacyWarningKindV1` serialization and stable names.
- Request freshness, first-contact, and identity-drift linting.
- Action recipe kind, fee status, and action status mapping.
- `RemoteScanProofHintV1` chunk hash and height binding.
- `DisclosureReceiptV1` canonical hash/signature coverage.
- `WalletPolicyModeV1` mode behavior.
- `ValidatorMandateLockV1` commitment derivation and unlock readiness.
- `WalletCompactionPlanV1` dry-run determinism.
- Public wallet/RPC type scans do not expose backend internals.

### 18.2 Integration Tests

- Wallet send path blocks expired requests and identity drift.
- Wallet receive path rejects malformed worker evidence.
- Object action path rejects missing policies in enforce mode.
- Disclosure proof hides unrelated inventory.
- Compaction preserves reopen, spendable set, scan cursor, export, and restore.
- Locked assets stay excluded from send, merge, split, swap, and local mutations.
- Runtime/storage boundary guardrails prove wallet changes do not create new settlement, checkpoint, planner, validator, watcher, or rollup authority.

### 18.3 Regression Commands

Run focused tests as implementation lands:

```bash
cargo test -p z00z_wallets privacy --all-features
cargo test -p z00z_wallets remote_scan --all-features
cargo test -p z00z_wallets object --all-features
cargo test -p z00z_wallets storage --all-features
cargo test -p z00z_wallets tx_send --all-features
cargo test -p z00z_storage object --all-features
cargo test -p z00z_storage test_downstream_guardrails --all-features
cargo test -p z00z_aggregators hjmt --all-features
cargo test -p z00z_validators object_policy --all-features
cargo test -p z00z_watchers hjmt_publication_contract --all-features
```

Before closing the phase, run the repository Rust gate:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

Run `cargo doc --no-deps` if public Rust APIs or docs are changed.

## 19. Implementation Phasing

### Phase 080.1: Safety Reports

Implement `PrivacyReportV1`, warning taxonomy, pre-send linting, and post-receive reporting.

### Phase 080.2: Remote Evidence V1

Upgrade remote scan evidence and preserve wallet-local receive authority tests.

### Phase 080.3: Policy Mode

Add `WalletPolicyModeV1`, enforce/warn behavior, and stable object reject mapping.

### Phase 080.4: Disclosure

Add evidence package, disclosure receipt, audit request, and minimal proof skeleton.

### Phase 080.5: Validator Mandate Locks

Add canonical lock and unlock actions over `RightLeaf::ValidatorMandate`.

### Phase 080.6: Action Recipes And Package Transcripts

Expose five action recipes and optional local package-build transcripts.

### Phase 080.7: Compaction

Add compaction plan, stats, retention hooks, and export/restore guardrails.

### Phase 080.8: Measurement Hooks

Add proof locality counters and cache invalidation tests only if benchmarks justify them.

### Phase 080.9: Runtime/Storage Boundary Guardrails

Apply Workstream 080-J to all storage-adjacent, runtime-adjacent, validator-adjacent, watcher-adjacent, and rollup-adjacent Phase 080 changes.

## 20. Phase Acceptance Criteria

Phase 080 is complete only when:

- privacy reports exist and do not alter package digest semantics;
- remote scan proof hints are typed and bound to returned chunks;
- wallet policy mode can fail closed on unknown or missing object policies;
- scoped disclosure can produce a receipt and bounded proof without wallet-history export;
- `ValidatorMandate` locks are canonical rights, not soft UI locks;
- compaction preserves reopen, export, restore, spendable set, and scan cursor;
- action recipes expose simple user actions without hiding critical external trust boundaries;
- runtime/storage refactor guardrails prevent backend internals, planner authority, watcher projections, validator verdicts, and rollup composition from becoming wallet authority;
- no new public authority root, wallet authority surface, trace key, or general VM was introduced;
- focused tests and the relevant repository Rust gate pass.

## 21. Non-Negotiable Rejections

- Do not modify `crates/z00z_crypto/tari/`.
- Do not bypass `z00z_utils` for wallet file I/O, time, serialization, RNG, or logging boundaries.
- Do not expose `serde_*`, raw backend roots, or dependency-specific script types as stable public wallet APIs.
- Do not log secrets, seed phrases, disclosure payloads, private field values, or wallet inventory.
- Do not create a second tx-history plane.
- Do not let remote helpers, auditors, or workers mutate wallet state.
- Do not use `unwrap()`, undocumented `expect()`, or panic-driven production control flow.
- Do not claim offline finality, issuer solvency, external reserve integrity, or universal legal compliance.
- Do not mix wallet behavior work with broad rename-only refactors.
- Do not make `z00z_rollup_node`, validators, watchers, or backend adapters into wallet-state authority.

## 22. Doublecheck Summary

The recommendations above were doublechecked against live repository surfaces before this specification was written:

- merge/consolidation is not carried as a new implementation task because `merge_assets` already builds live `TxPackage` metadata through `local_mutation_exec(...)`;
- remote scan evidence is still advisory and therefore safe to upgrade as a typed envelope;
- `ObjectPolicyRegistryV1` already exists, so Phase 080 should extend wallet policy mode around it instead of adding a second registry;
- `validator_mandate_lock_v1` helper logic already exists, so the remaining task is canonical wallet workflow and persistence coverage;
- compaction exists only as temporary-file cleanup, so deeper compaction requires a conservative plan and export/restore guardrails;
- backend seam, redb backend lane, memory backend lane, runtime batch planner, placement table, shard executor, validator facade, watcher facade, and rollup runtime composition already exist, so Phase 080 carries their guardrails rather than re-running their migration map;
- the path-level rename and move table was audited against live files; it has no remaining mandatory Phase 080 rename task, and remaining open rows are deferred backend work, superseded names, or rename-only cleanup;
- `PrivacyReport`, `DisclosureReceipt`, `AuditViewRequest`, `AuditViewProof`, `EvidencePackage`, and `WalletPolicyMode` are not live Rust/API types yet; related disclosure/evidence labels in genesis configuration still need wallet implementation.
