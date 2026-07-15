# Requirements: Z00Z

<!-- markdownlint-disable MD060 -->

**Defined:** 2026-03-23
**Core Value:** Confidential asset and wallet flows must remain correct, explicit, and storage-safe.

## v0.15 Requirements

### Storage Serialization

- [x] **STSER-01**: JMT-backed storage state can be serialized into a deterministic machine-readable artifact inside `z00z_storage`.
- [x] **STSER-02**: Serialized JMT artifacts can be decoded or reconstructed into a storage-owned inspection model.
- [x] **STSER-03**: The storage crate can export deterministic human-readable visualization of nodes, links, hashes, and canonical paths.
- [x] **STSER-04**: Existing `assets`, `checkpoint`, and `snapshot` behavior remains stable unless explicitly using the new serialization surface.

### JMT Search And RedB

- [x] **STREDB-01**: Live `AssetStore` mutations can commit atomically to a RedB-backed durable store together with required snapshot and checkpoint blobs encoded by the existing canonical codecs.
- [x] **STREDB-02**: RedB durable load can rehydrate a usable `AssetStore` with the same canonical root and canonical `AssetPath` semantics as committed state.
- [x] **STREDB-03**: `z00z_storage` exposes storage-owned search APIs for exact canonical path lookup, exact `asset_id` lookup, deterministic `definition_id` or `definition_id + serial_id` listing, and ordered prefix or range pagination.
- [x] **STREDB-04**: Secondary search indexes remain convenience-only and never alter canonical roots or canonical path ownership semantics.

## vNext Requirements

### Scenario 1 Storage Integration

- [x] **SCN1-01**: Scenario 1 claim publication writes canonical claim outputs into storage-owned JMT state through `AssetStore` mutation APIs rather than simulator-only artifacts.
- [x] **SCN1-02**: Scenario 1 regular transfer preparation and the first wallet boundary consume canonical storage witness bytes and full `AssetPath`-bound pre-state instead of compatibility proof shims.
- [x] **SCN1-03**: Scenario 1 regular transfer execution moves to a storage-backed `resolve -> verify -> apply` path, with Stage 6 limited to draft or reload bridging and Stage 7 owning canonical apply.
- [x] **SCN1-04**: Scenario 1 save/load/search plus checkpoint and snapshot reload preserve the same canonical root and canonical-path lookup semantics across RedB reopen.
- [x] **SCN1-05**: Scenario 1 final checkpoint flow keeps draft and final artifacts separate and includes negative tamper coverage for witness, snapshot, and checkpoint materials.

### Scenario 1 Stage Surface Refactor

- [x] **SCN1-06**: Scenario 1 stages 3 through 10 SHALL resolve through one explicit logical stage file or facade per runtime stage instead of mixed multi-stage public containers.
- [x] **SCN1-07**: `scenario_design.yaml` SHALL remain a descriptive scenario document, structurally aligned with `design_scenario_orig.yaml`, rather than acting as an executable control surface.
- [x] **SCN1-08**: The documented order of inputs, outputs, calls, actions, and checks in `scenario_design.yaml` SHALL match the real runtime execution order implemented by `runner.rs` and the stage files.

### Future Storage Extensions

- **STSER-05**: Serialized storage artifacts may later support broader transport or interchange workflows.
- **STSER-06**: Visualization may later add richer renderers beyond stable DOT and text output.

### Wallet Contract Gap Closure

- [x] **PH19-NULL**: WHEN a claim package is reserved, published, rolled back, or finalized, THE SYSTEM SHALL persist canonical nullifier state under a storage-owned transition and SHALL advance asset state and replay-protection state atomically.
- [x] **PH19-SCAN**: WHEN a public receive path evaluates canonical or runtime receive input, THE SYSTEM SHALL expose report-first receive taxonomy and SHALL classify malformed runtime input as `InvalidInput` instead of silently degrading it to `NotMine` or `MaybeMine` across direct callers in scope.
- [x] **PH19-BACKUP**: WHEN a public wallet backup is created or restored, THE SYSTEM SHALL use a `WalletExportPack`-based full-restore contract for the active format and SHALL keep `BackupPayloadV1` readable as a legacy input format.

### Crypto Audit Remediation

- [x] **PH25-CLAIM-GATE**: WHEN the workspace is built with the default production feature set, THE SYSTEM SHALL keep placeholder `claim_v1` proof or authority helpers off the stable public surface and SHALL reject transitional zero-root claims in production verification flows.
- [x] **PH25-CLAIM-V2**: WHEN a claim package is built or verified, THE SYSTEM SHALL use canonical claim statement bytes, a real Tari-backed Schnorr `ClaimAuthoritySigV2`, and versioned `ClaimStmtV2` metadata instead of placeholder hash-derived proof or signature blobs.
- [x] **PH25-SOURCE-PROOF**: WHEN claim verification checks source membership, THE SYSTEM SHALL validate a storage-owned authoritative root and a typed `ClaimSourceProof` inclusion proof instead of trusting an opaque blob or a transitional zero root.
- [x] **PH25-ZKPACK**: WHEN production wallet flows create or open encrypted asset packs, THE SYSTEM SHALL use the standard ChaCha20-Poly1305 wallet facade as the blessed `ZkPack` path and SHALL keep the custom crypto zkpack surface experimental and non-default.
- [x] **PH25-FAILCLOSED**: IF scalar derivation, scalar-from-hash, HMAC initialization, or random-scalar generation fails, THEN THE SYSTEM SHALL return a typed error and SHALL NOT substitute constant outputs, silent fallbacks, or unbounded retry loops.
- [x] **PH25-STEALTH-BIND**: WHEN stealth outputs, range proofs, or claim digests are derived, THE SYSTEM SHALL bind canonical `tag16`, `leaf_ad`, `asset_id`, `chain_id`, versions, and proof context through crypto-owned APIs and frozen vectors.

### Core Crypto Audit Remediation

- [x] **PH26-GENESIS**: WHEN a genesis state is generated or verified for `Mainnet` or `Testnet`, THE SYSTEM SHALL reject missing or mismatched consensus anchors and SHALL reject unknown protected-network parsing or seed-policy fallbacks.
- [x] **PH26-ASSET-ID**: WHEN an asset definition is created, loaded from config, or decoded from a trusted wire path, THE SYSTEM SHALL derive and validate its identifier from one canonical framed definition payload and SHALL keep test-only asset-id domains out of production paths.
- [x] **PH26-REGISTRY**: WHEN a registry snapshot is created or applied, THE SYSTEM SHALL derive registry integrity from the full ordered canonical definition payload and SHALL reject snapshot drift that preserves only definition identifiers.
- [x] **PH26-WIRE**: WHEN an untrusted asset wire or DTO payload is decoded, THE SYSTEM SHALL reject secret-bearing or confidentiality-breaking fields and SHALL preserve or explicitly reject protocol-state flags instead of silently dropping them.
- [x] **PH26-AUTH**: WHEN asset ownership or stealth metadata is verified, THE SYSTEM SHALL bind canonical owner and stealth-critical fields to verifier-checked state and SHALL NOT treat a bare owner signature as sufficient proof of commitment-opening ownership.
- [x] **PH26-NONCE-FEE**: WHEN production code derives nonces, validates fee assets, or checks proof-width-sensitive amounts, THE SYSTEM SHALL fail closed on time-provider errors, SHALL enforce canonical native fee-asset identity, and SHALL tie maximum amount policy to the supported proof-width boundary.

### Utils Audit Remediation

- [x] **PH27-MEMLOCK**: WHEN secret bytes are locked through `z00z_utils::os_hardening`, THE SYSTEM SHALL bind the lock guard lifetime to the backing slice, SHALL zeroize before unlock, and SHALL NOT expose raw backing addresses through debug output.
- [x] **PH27-CONFIG**: WHEN YAML configuration is loaded or combined with environment configuration, THE SYSTEM SHALL enforce bounded reads, SHALL surface malformed or permission-denied YAML errors, and SHALL treat only `NotFound` as an allowed default layered-config downgrade.
- [x] **PH27-TIME**: WHEN production code derives nonce, expiry, ordering, anti-replay, or other security-sensitive timestamps from `z00z_utils`, THE SYSTEM SHALL use fallible time-provider APIs or an explicitly named compatibility helper and SHALL leave no security-critical consumer dependent on lossy zero-fallback wrappers.
- [x] **PH27-RNG**: WHEN deterministic RNG is exposed through `z00z_utils`, THE SYSTEM SHALL preserve approved genesis or simulator reproducibility paths while making unapproved production use materially harder and SHALL NOT present deterministic output as approved unpredictable production entropy.
- [x] **PH27-LOGGER**: WHEN log messages are persisted through `z00z_utils` file-based loggers, THE SYSTEM SHALL sanitize ANSI or control-byte injection, SHALL preserve severity prefixes in rotated log output, and SHALL keep the current symlink or trusted-parent boundary explicit.
- [x] **PH27-IO**: WHEN files are written through `z00z_utils` atomic-write helpers, THE SYSTEM SHALL propagate permission-copy failures and SHALL make the durability or secrecy contract explicit for generic versus private write paths across Unix and non-Unix platforms.
- [x] **PH27-JSON**: WHEN `z00z_utils` exposes JSON-construction helpers or logger macros, THE SYSTEM SHALL record and enforce an explicit repository policy for `serde_json` macro or `Value` usage rather than leaving abstraction-boundary drift implicit.

### Wallet Audit Remediation

- [x] **PH29-RECON**: WHEN Phase 029 execution begins, THE SYSTEM SHALL classify every fused wallet finding against the current tree, SHALL label stale or ambiguous evidence explicitly, and SHALL freeze one authoritative execution target inventory before remediation waves start.
- [x] **PH29-VIEWKEY**: WHEN sender, scanner, spend, or rotation flows derive wallet view keys, THE SYSTEM SHALL use one canonical live or historical derivation policy and SHALL guard that policy with regression coverage.
- [x] **PH29-KDF**: WHEN wallet or RedB persistence derives encryption keys, THE SYSTEM SHALL converge on the versioned RedB V2 KDF contract and SHALL treat weaker legacy behavior as compatibility-only.
- [x] **PH29-BACKUP**: WHEN wallet backups are exported or imported, THE SYSTEM SHALL persist self-describing KDF metadata and SHALL make backup metadata visibility policy explicit.
- [x] **PH29-PANIC**: WHEN operator-reachable wallet runtime flows fail, THE SYSTEM SHALL return typed errors and SHALL NOT rely on `expect()` or `unwrap()` panic paths.
- [x] **PH29-SEEDSALT**: WHEN new wallet writes or reveal or export flows derive seed-encryption salt, THE SYSTEM SHALL stop using deterministic `wallet_id`-derived salt for new writes and SHALL preserve compatibility only where explicitly required.
- [x] **PH29-KEYMGR**: WHEN key-manager allocation or receiver-secret construction runs, THE SYSTEM SHALL enforce gap-limit invariants and SHALL reject unusable receiver secrets at the object boundary.
- [x] **PH29-SECRET**: WHEN wallet code persists or exposes secret-bearing values, THE SYSTEM SHALL use explicit zeroizing ownership boundaries rather than long-lived plaintext wrappers or convenience containers.
- [x] **PH29-DIGEST**: WHEN wallet transaction digests are built, THE SYSTEM SHALL frame variable-length fields explicitly so hashing contracts stay unambiguous.
- [x] **PH29-VALIDATION**: WHEN wallet runtime validation surfaces warnings or failures, THE SYSTEM SHALL expose an explicit validation result contract instead of collapsing warnings into logs or binary valid/error DTOs.

### Long-File Structural Refactor

- [x] **PH30-SEAMS**: WHEN oversized Rust files are refactored, THE SYSTEM SHALL split them along semantically homogeneous responsibility seams and SHALL NOT create shard-like files solely to satisfy line-count targets.
- [x] **PH30-FACADE**: WHEN a structural split touches a caller-visible module surface, THE SYSTEM SHALL preserve the current facade or re-export contract until a dedicated normalization wave closes.
- [x] **PH30-PROTECTED**: WHEN crypto owner surfaces, wallet session or store boundaries, or genesis alias seams are refactored, THE SYSTEM SHALL keep one canonical owner surface and stable boundary contracts throughout the wave.
- [x] **PH30-NORMALIZE**: WHEN external path cleanup is required, THE SYSTEM SHALL prove the caller inventory, normalize consumers in dedicated subwaves, and remove legacy deep paths only after synchronized code, docs, rustdoc, and planning updates.
- [x] **PH30-VERIFY**: WHEN a structural split wave changes Rust code, THE SYSTEM SHALL run `bootstrap_tests.sh` first, then named targeted verification anchors, repeated task-execution reviews, and broader release-style validation where required before closeout.
- [x] **PH30-SYNC**: WHEN refactors rename modules or change module wiring, THE SYSTEM SHALL update docs, YAML, rustdoc, and planning references in the same wave.

### Architecture Boundary Refactor

- [x] **PH31-INV**: WHEN Phase 031 begins, THE SYSTEM SHALL produce one explicit import-graph and caller inventory for every reviewed root, shim, suffix, wildcard export, Tari leak, and simulator deep import before retirement or facade narrowing starts.
- [x] **PH31-CORE**: WHEN `z00z_core` root exports are narrowed, THE SYSTEM SHALL replace wildcard asset exports with a curated stable facade and SHALL preserve the bounded asset-wire decode or upstream-cap contract explicitly.
- [x] **PH31-CRYPTO**: WHEN `z00z_crypto` public surfaces are refactored, THE SYSTEM SHALL keep one Z00Z-owned stable crypto facade, SHALL demote vendor passthroughs to an explicit non-default lane, and SHALL gate test-only AEAD helpers out of non-test profiles.
- [x] **PH31-NET**: WHEN `z00z_networks` boundaries are clarified, THE SYSTEM SHALL keep `rpc` limited to transport and dispatch concerns and SHALL document OnionNet as a node-owned privacy overlay instead of an undefined application service.
- [x] **PH31-WLT-SEAMS**: WHEN wallet service seams are cleaned up, THE SYSTEM SHALL remove `include!` assembly from the canonical service root and SHALL make stable, provisional, and reachability-only lanes explicit.
- [x] **PH31-WLT-ID**: WHEN wallet discovery, open, unlock, or lock flows run, THE SYSTEM SHALL keep persisted wallet identity as the single source of truth and SHALL require an explicit authorization posture for `lock_wallet` transport callers.
- [x] **PH31-WLT-RPC**: WHEN wallet RPC edges are normalized, THE SYSTEM SHALL keep DTOs and dispatcher wiring adapters-owned and SHALL NOT widen transport-facing aliases into the stable wallet root facade.
- [x] **PH31-STORAGE**: WHEN checkpoint artifacts are drafted, finalized, stored, or rehydrated, THE SYSTEM SHALL keep proof-binding and replay semantics explicit and SHALL NOT treat opaque compatibility bytes as canonical checkpoint proof.
- [x] **PH31-SIM**: WHEN simulator stages emit artifacts or reset outputs, THE SYSTEM SHALL use stable-facade imports only, SHALL remove or debug-gate plaintext secret artifacts, and SHALL constrain recursive cleanup to an approved sandbox root.
- [x] **PH31-UTILS**: WHEN Phase 031 closes, THE SYSTEM SHALL leave one README-level or boundary-note description of `z00z_utils` admission policy that explains allowed cross-cutting roles and rejects megacrate drift.
- [x] **PH31-CLOSEOUT**: WHEN suffixes, shims, or provisional exports are retired in Phase 031, THE SYSTEM SHALL require caller-proof-backed retirement evidence, green release-style validation, and synchronized planning artifacts before marking the phase complete.

### Scenario 1 Crypto Audit Remediation

- [x] **PH32-SEM**: WHEN Scenario 1 claim, spend, stealth, or receiver-binding semantics are described, tested, or enforced, THE SYSTEM SHALL use one canonical definition for `leaf_ad_id`, `s_out`, receiver-card/request binding, and public-versus-wallet-local trust language.
- [x] **PH32-CLAIM-BIND**: WHEN a Scenario 1 claim package is signed or verified, THE SYSTEM SHALL bind the authority signature to the full authenticated claim tuple, including `asset_id`, source commitment, `chain_id`, scenario/ruleset version, and the authoritative source root.
- [x] **PH32-CLAIM-TRUST**: WHEN Scenario 1 emits or consumes a claim package, THE SYSTEM SHALL bind accepted claim roots and inclusion proofs to the carried storage-backed membership contract shared by `AssetStore::claim_source_contract_for_item(...)`, simulator bundle verification, and wallet verification, and SHALL reject tuple combinations whose `leaf_ad_id`, root, or proof fields drift from that contract. This closure does not by itself claim a broader anchored authority lifecycle beyond the carried membership contract and signed claim tuple.
- [x] **PH32-SPEND**: WHEN a spend is accepted in Scenario 1 or validator-style flows, THE SYSTEM SHALL use one explicit public verifier contract that binds previous root, input references, output leaves, owner-tag relation, asset-id relation, balance equation, range-proof commitments, deterministic nullifier semantics, `chain_id`, version, and transcript/public-input framing. The delivered persisted public spend contract is already live and now authenticates one signed nullifier field on the public seam, while the witness bridge and structural spend rules enforce deterministic `chain_id || s_in` derivation through the shared canonical helper. This closure does not by itself upgrade the accepted-path boundary into a finished full-ZK spend theorem.
- [x] **PH32-CHECKPOINT**: WHEN checkpoint, apply, or replay-sensitive flows accept tx/spent-set state, THE SYSTEM SHALL reject placeholder proof/spent-set success lanes and SHALL enforce authoritative checkpoint proof plus spent-set validation.
- [x] **PH32-SECRET**: WHEN Scenario 1 runs with default settings, THE SYSTEM SHALL NOT emit plaintext wallet secrets and SHALL keep any debug-secret export behind an explicit non-default gated lane with private output handling, bounded retention, and no release-path reachability.
- [x] **PH32-HONEST**: WHEN Scenario 1 docs, comments, tests, or summaries describe crypto guarantees, THE SYSTEM SHALL NOT claim sender ignorance, trustless validator verification, live STARK/FRI enforcement, or stronger checkpoint authority than the implemented boundary actually provides.

Reclassification gate: Active wording may now reflect the implemented storage-backed claim continuity, deterministic spend nullifier closure, `core::stealth` sender authority, and backend-defined package-coupled checkpoint acceptance because those seams are implemented and re-verified. Append-only historical audit artifacts remain historical evidence.

Phase 034 closure package is rooted in `034-08-SUMMARY.md`, `034-VALIDATION.md`, and `034-CLOSEOUT.md`; later post-closure hygiene is recorded in `034-09-SUMMARY.md`. The `keep_path(...)` sidecar is executed on the live tree as a local non-semantic cleanup and remains outside the closure proof story.

### Phase 034 Mix1 Fixes

- [x] **PH34-CLAIM-CONTINUITY**: WHEN a live claim package is emitted or verified after Phase 034, THE SYSTEM SHALL derive accepted claim roots and inclusion proofs from the carried storage-backed membership contract shared by `AssetStore::claim_source_contract_for_item(...)`, simulator bundle verification, and wallet verification, and SHALL reject root or proof drift instead of reconstructing authority from a helper-owned one-item seam.
- [x] **PH34-SPEND-NULLIFIER**: WHEN a regular public spend is built, verified, or structurally validated after Phase 034, THE SYSTEM SHALL authenticate one signed nullifier field on the public seam and SHALL enforce deterministic `chain_id || s_in` derivation through the witness bridge and structural spend rules via one shared canonical helper, without promoting that accepted-path closure into a finished full-ZK spend theorem.
- [x] **PH34-SENDER-AUTHORITY**: WHEN sender-side output construction is invoked after Phase 034, THE SYSTEM SHALL expose `stealth` as the only public sender-construction authority on the canonical `core::stealth` module path and SHALL treat `core::tx` bridge utilities plus narrow test-only helpers as noncanonical internal surfaces.
- [x] **PH34-CHECKPOINT-BACKEND**: WHEN checkpoint finalize, seal, reload, or Scenario 1 promotion paths accept proof-bearing state after Phase 034, THE SYSTEM SHALL bind acceptance to one backend-defined package-coupled checkpoint contract over proof-system typing, statement shape, exec identity, the persisted snapshot/link tuple, and bound root or payload invariants, while leaving final cryptographic closure and generic standalone proof-backend claims outside the shipped boundary.
- [x] **PH34-DOC-ALLOWLIST**: WHEN active requirements, stage-surface guards, or planning docs describe the Phase 034 seams, THE SYSTEM SHALL describe only the implemented storage-backed claim continuity, deterministic spend nullifier closure, `core::stealth` sender authority, and backend-defined package-coupled checkpoint acceptance truth, and SHALL leave append-only historical audit evidence untouched.
- [x] **PH34-CLOSURE-PROOF**: WHEN Phase 034 is closed, THE SYSTEM SHALL produce one repository-backed closure package showing that Q63, Q64, Q65, and Q47 are no longer the active live blockers or are explicitly re-narrowed by source update.
- [x] **PH34-KEEP-PATH-SIDECAR**: WHERE the optional `keep_path(...)` sidecar is executed, THE SYSTEM SHALL preserve search behavior exactly while simplifying the local predicate structure and SHALL keep the sidecar outside the semantic closure story.
- [x] **PH34-ID-SIGNATURE-HYGIENE**: WHERE the optional identifier-length sidecar is executed, THE SYSTEM SHALL rename only live non-Tari signature-like identifiers that exceed the five-word rule, preserving behavior and updating all affected guards honestly.
- [x] **PH34-SUFFIX-COLLAPSE**: WHERE the optional suffix sidecar is executed, THE SYSTEM SHALL collapse only production-current Rust-facing suffix-bearing names to unsuffixed canonical names and SHALL retain reserved-future compatibility surfaces that current production open, import, or migration paths still require.

### Phase 044 Wallet Assets

- [ ] **PH44-LEDGER**: WHEN wallet asset lifecycle state is persisted or displayed, THE SYSTEM SHALL distinguish `Available`, `Reserved`, `Exported`, `ClaimPending`, `PendingSpend`, `Spent`, `PendingChange`, `PendingReceive`, `ReorgPending`, `Quarantined`, and `Dropped`, and SHALL treat only `Available` as spendable.
- [ ] **PH44-SEND**: WHEN sender build or send runs, THE SYSTEM SHALL perform selection, reservation, output construction, tx package assembly, verification, journal persistence, and admission through the canonical wallet lifecycle engine, and SHALL NOT return `BuiltTxStub` as live behavior.
- [ ] **PH44-OFFLINE**: WHEN a verified tx package is exported or imported offline, THE SYSTEM SHALL preserve portable canonical tx bytes, validate chain and version, scan owned outputs, and write only pending receiver state before confirmation.
- [ ] **PH44-ADMIT**: WHEN real chain submission remains unavailable, THE SYSTEM SHALL represent acceptance through one explicit trait-backed simulated admission adapter instead of RPC-local fake success.
- [ ] **PH44-RECONCILE**: WHEN wallet finalization runs, THE SYSTEM SHALL scan typed storage/checkpoint evidence and transition pending rows idempotently to final states only when evidence matches journal expectations.
- [ ] **PH44-RECEIVE**: WHEN report-only receive runs, THE SYSTEM SHALL NOT mutate claims, tx history, pending rows, or balances; receiver finalization SHALL use `recv_route(..., ReceiveNext::PersistClaim)`.
- [ ] **PH44-BALANCE**: WHEN wallet balance is calculated, THE SYSTEM SHALL derive `available` and `pending` from lifecycle rows and SHALL NOT report `pending = 0` when pending lifecycle rows exist.
- [ ] **PH44-HISTORY**: WHEN tx details, pending lists, or history are queried, THE SYSTEM SHALL derive them from real journal data, role-tagged rows, receipts, tx bytes, and lifecycle status.
- [ ] **PH44-CANCEL**: WHEN cancellation is requested, THE SYSTEM SHALL release inputs only before admission or when no-admission evidence or a fail-closed policy proves release is safe; exported packages SHALL keep sender inputs non-spendable until resolved.
- [ ] **PH44-DRIFT**: WHEN Phase 044 introduces wallet asset lifecycle code, THE SYSTEM SHALL NOT duplicate assembler, verifier, tx schema, receive persistence, wallet asset authority, chain broadcast, or scan-cursor proof semantics.
- [ ] **P-044-001**: WHEN `.wlt` files are written or restored, THE SYSTEM SHALL keep them limited to wallet snapshot, identity, encrypted seed material, and wallet-level restore state, and SHALL NOT store tx packages or unbounded tx history inside `.wlt`.
- [ ] **P-044-002**: WHEN tx history is persisted for a wallet stem, THE SYSTEM SHALL use `wallet_<stem>_tx_history.jsonl` as the canonical live tx-history store, not as a derivative backup export or generated sidecar.
- [ ] **P-044-003**: WHEN Phase 044 stores tx packages, THE SYSTEM SHALL NOT introduce a new broad database for tx packages.
- [ ] **P-044-004**: WHEN new tx-history writes occur, THE SYSTEM SHALL NOT use `wallet_<stem>_tx_history/<tx_hash>.json` or any per-transaction JSON directory as the live tx store.
- [ ] **P-044-005**: WHEN a JSONL entry carries a tx package, THE SYSTEM SHALL preserve the exact canonical tx package bytes as built or imported, including encrypted package fields and encrypted asset data that are part of the package.
- [ ] **P-044-006**: WHEN tx-history JSONL rows are written, THE SYSTEM SHALL NOT include decrypted wallet secrets, private blindings, plaintext seed material, or non-package private wallet state.
- [ ] **P-044-007**: WHEN `TxStorageImpl` is refactored, THE SYSTEM SHALL use a single JSONL-file store whose reads fold rows by `tx_hash` into the current view.
- [ ] **P-044-008**: WHEN tx status changes or delete-like requests occur, THE SYSTEM SHALL append new rows and replace physical deletion with forensic tombstone rows unless a future explicit purge policy is approved.
- [ ] **P-044-009**: WHEN JSONL tx-history is written, THE SYSTEM SHALL write atomically through a temporary sibling file and rename, guarded by a per-wallet file lock or existing wallet session lock.
- [ ] **P-044-010**: WHEN backup is created, THE SYSTEM SHALL read existing live JSONL bytes, validate them, and include the exact JSONL bytes plus manifest in the encrypted forensic backup payload; `Vec<TxRecord>` SHALL remain a derived view, not the authority.
- [ ] **P-044-011**: WHEN restore writes tx history, THE SYSTEM SHALL write archived JSONL bytes back to the restored wallet stem's live JSONL path and SHALL NOT reconstruct history from extracted records or expand rows into per-tx JSON files.
- [ ] **P-044-012**: WHEN existing `wallet_<stem>_tx_history/` directories are encountered, THE SYSTEM SHALL treat them as legacy migration input only, preserve tx bytes, write `Migrated` JSONL entries, and SHALL NOT delete legacy directories automatically.

### Recursive Checkpoint Proof

- [ ] **RCP-069**: WHEN Phase 069 is executed, THE SYSTEM SHALL implement and locally verify the complete non-authoritative Nova IVC plus Plonky3 recursive-STARK checkpoint evidence contract defined by `.planning/phases/069-Recursive-Proof/069-TODO.md`, SHALL preserve the Phase 068 storage-owned theorem and canonical admission path, SHALL close all thirteen required plan groups with real backend and negative-test evidence, and SHALL NOT substitute shape checks, placeholders, deferred local correctness, Nova-only PQ claims, or end-to-end PQ claims over classical nested primitives.

## Out of Scope

| Feature | Reason |
| ------- | ------ |
| Moving serialization ownership into wallet or core crates | Conflicts with current storage boundary |
| Replacing live AssetStore mutation paths during bootstrap | Too risky for initial phase scope |
| Adding a new persistence backend solely for phase 015 bootstrap | Not needed to start canonical planning |
| Storing tx packages or unbounded tx history inside `.wlt` | Conflicts with Phase 044 wallet-state-only snapshot boundary |
| Adding a new broad database for Phase 044 tx packages | Phase 044 requires the canonical live JSONL tx-history file instead |
| Using per-transaction JSON directories for new live tx-history writes | Phase 044 treats those directories as legacy migration input only |

## Traceability

| Requirement | Phase | Status |
| ----------- | ----- | ------ |
| STSER-01 | Phase 015 | Complete |
| STSER-02 | Phase 015 | Complete |
| STSER-03 | Phase 015 | Complete |
| STSER-04 | Phase 015 | Complete |
| STREDB-01 | Phase 016 | Complete |
| STREDB-02 | Phase 016 | Complete |
| STREDB-03 | Phase 016 | Complete |
| STREDB-04 | Phase 016 | Complete |
| SCN1-01 | Phase 017 | Planned |
| SCN1-02 | Phase 017 | Planned |
| SCN1-03 | Phase 018 | Complete |
| SCN1-04 | Phase 018 | Complete |
| SCN1-05 | Phase 018 | Complete |
| SCN1-06 | Phase 021 | Complete |
| SCN1-07 | Phase 021 | Complete |
| SCN1-08 | Phase 021 | Complete |
| PH19-NULL | Phase 19 | Complete |
| PH19-SCAN | Phase 19 | Complete |
| PH19-BACKUP | Phase 19 | Complete |
| PH25-CLAIM-GATE | Phase 025 | Complete |
| PH25-CLAIM-V2 | Phase 025 | Complete |
| PH25-SOURCE-PROOF | Phase 025 | Complete |
| PH25-ZKPACK | Phase 025 | Complete |
| PH25-FAILCLOSED | Phase 025 | Complete |
| PH25-STEALTH-BIND | Phase 025 | Complete |
| PH26-GENESIS | Phase 026 | Complete |
| PH26-ASSET-ID | Phase 026 | Complete |
| PH26-REGISTRY | Phase 026 | Complete |
| PH26-WIRE | Phase 026 | Complete |
| PH26-AUTH | Phase 026 | Complete |
| PH26-NONCE-FEE | Phase 026 | Complete |
| PH27-MEMLOCK | Phase 027 | Complete |
| PH27-CONFIG | Phase 027 | Complete |
| PH27-TIME | Phase 027 | Complete |
| PH27-RNG | Phase 027 | Complete |
| PH27-LOGGER | Phase 027 | Complete |
| PH27-IO | Phase 027 | Complete |
| PH27-JSON | Phase 027 | Complete |
| PH29-RECON | Phase 029 | Complete |
| PH29-VIEWKEY | Phase 029 | Complete |
| PH29-KDF | Phase 029 | Complete |
| PH29-BACKUP | Phase 029 | Complete |
| PH29-PANIC | Phase 029 | Complete |
| PH29-SEEDSALT | Phase 029 | Complete |
| PH29-KEYMGR | Phase 029 | Complete |
| PH29-SECRET | Phase 029 | Complete |
| PH29-DIGEST | Phase 029 | Complete |
| PH29-VALIDATION | Phase 029 | Complete |
| PH30-SEAMS | Phase 030 | Complete |
| PH30-FACADE | Phase 030 | Complete |
| PH30-PROTECTED | Phase 030 | Complete |
| PH30-NORMALIZE | Phase 030 | Complete |
| PH30-VERIFY | Phase 030 | Complete |
| PH30-SYNC | Phase 030 | Complete |
| PH31-INV | Phase 031 | Complete |
| PH31-CORE | Phase 031 | Complete |
| PH31-CRYPTO | Phase 031 | Complete |
| PH31-NET | Phase 031 | Complete |
| PH31-WLT-SEAMS | Phase 031 | Complete |
| PH31-WLT-ID | Phase 031 | Complete |
| PH31-WLT-RPC | Phase 031 | Complete |
| PH31-STORAGE | Phase 031 | Complete |
| PH31-SIM | Phase 031 | Complete |
| PH31-UTILS | Phase 031 | Complete |
| PH31-CLOSEOUT | Phase 031 | Complete |
| PH32-SEM | Phase 032 | Complete |
| PH32-CLAIM-BIND | Phase 032 | Complete |
| PH32-CLAIM-TRUST | Phase 033 | Complete |
| PH32-SPEND | Phase 033 | Complete |
| PH32-CHECKPOINT | Phase 032 | Complete |
| PH32-SECRET | Phase 032 | Complete |
| PH32-HONEST | Phase 032 | Complete |
| PH34-CLAIM-CONTINUITY | Phase 034 | Complete |
| PH34-SPEND-NULLIFIER | Phase 034 | Complete |
| PH34-SENDER-AUTHORITY | Phase 034 | Complete |
| PH34-CHECKPOINT-BACKEND | Phase 034 | Complete |
| PH34-DOC-ALLOWLIST | Phase 034 | Complete |
| PH34-CLOSURE-PROOF | Phase 034 | Complete |
| PH34-KEEP-PATH-SIDECAR | Phase 034 | Complete |
| PH34-ID-SIGNATURE-HYGIENE | Phase 034 | Complete |
| PH34-SUFFIX-COLLAPSE | Phase 034 | Complete |
| PH44-LEDGER | Phase 044 | Planned |
| PH44-SEND | Phase 044 | Planned |
| PH44-OFFLINE | Phase 044 | Planned |
| PH44-ADMIT | Phase 044 | Planned |
| PH44-RECONCILE | Phase 044 | Planned |
| PH44-RECEIVE | Phase 044 | Planned |
| PH44-BALANCE | Phase 044 | Planned |
| PH44-HISTORY | Phase 044 | Planned |
| PH44-CANCEL | Phase 044 | Planned |
| PH44-DRIFT | Phase 044 | Planned |
| P-044-001 | Phase 044 | Planned |
| P-044-002 | Phase 044 | Planned |
| P-044-003 | Phase 044 | Planned |
| P-044-004 | Phase 044 | Planned |
| P-044-005 | Phase 044 | Planned |
| P-044-006 | Phase 044 | Planned |
| P-044-007 | Phase 044 | Planned |
| P-044-008 | Phase 044 | Planned |
| P-044-009 | Phase 044 | Planned |
| P-044-010 | Phase 044 | Planned |
| P-044-011 | Phase 044 | Planned |
| P-044-012 | Phase 044 | Planned |
| RCP-069 | Phase 069 | Planned |

**Coverage:**

- v0.15 + vNext active requirements: 104 total
- Mapped to phases: 104
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-23*
*Last updated: 2026-07-11 after Phase 069 recursive-proof planning authority was registered*
