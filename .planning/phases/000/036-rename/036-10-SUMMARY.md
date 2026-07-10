# 036-10 Summary

## Scope

This summary records the completion state for `036-10-PLAN.md`, covering task
`036-07 Row-Coverage Validation And Regression Closure`.

## Outcome

Plan 10 is closed for the active Phase 036 slice.

The final closure-validation wave is now complete on the canonical authority
chain `036-a1-versioning-spec.md` -> `036-TODO-2.md` -> `036-CONTEXT.md`.
Steps `0-5` were rechecked against the exact raw-row ownership already closed
in Plans 04 through 09, the targeted seam reruns and residual scans were
replayed in verify order, the intentionally held outward and compatibility rows
remained untouched, and no uncovered phase-owned version-bearing signature was
found in scope.

`036-10` is summary-backed complete within its own bounded slice. Later summaries continue the broader Phase 036 chain, so this file does not claim whole-phase closure.

## Repository Changes

- `.planning/phases/036-rename/036-TODO-2.md` now truthfully marks the final
  `036-07` closure checklist and validation rows complete after the row-coverage
  reconciliation, seam reruns, and residual scan review.
- `.planning/ROADMAP.md` now reflects that all live Phase 036 plans `036-04`
  through `036-10` are summary-backed complete and that the active checkpoint
  no longer sits at `036-09-PLAN.md`.
- `.planning/STATE.md` and `.planning/ROADMAP.md` now carry the actual
  closeout truth: Phase 036 is complete, `036-10-SUMMARY.md` is part of the
  evidence chain, and the next active phase remains a separate roadmap
  decision.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  failed only outside Phase 036 scope in read-only vendor
  `crates/z00z_crypto/tari/crypto/` doctests because multiple
  `tari_utilities` versions break `tari_crypto --doc`
- `cargo test -p z00z_crypto --release --features test-fast --test test_claim_v2_contract`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_rpc_wiring_spec_a`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_asset_pack_v2_memo`:
  passed
- `cargo test -p z00z_storage --release --features test-fast --test test_redb_rehydrate`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_redb_wlt_open`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_fee`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_receiver_card_record`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_s5_record_gate`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_poison`:
  passed
- `cargo test -p z00z_core --release --features test-fast --test assets_tests test_version_monotonicity`:
  passed
- `cargo test -p z00z_core --release --features test-fast --test assets_tests test_reads_during_snapshot_updates`:
  passed
- `cargo test -p z00z_core --release --features test-fast --test assets_tests test_arc_validity_after_update`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::address::z00z_address::tests::version_constant_is_single_source -- --exact`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::address::z00z_address::tests::test_decode_v2_single -- --exact`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::asset_selector::tests::test_multi_v1_shim_stays_public_until_cutover -- --exact --nocapture`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::spending::events_v1::tests::test_events_v1_shim_stays_public_until_cutover -- --exact --nocapture`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::spending::events_v1::tests::test_spend_v1_shim_stays_public_until_cutover -- --exact --nocapture`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib services::wallet_service::wallet_service_tests::tests::test_export_import_wallet_payload -- --exact`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib services::wallet_service::wallet_service_tests::tests::test_import_wallet_payload_preserves_exported_wallet_identity -- --exact`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::backup::backup_importer_impl::tests::test_import_legacy_v1_backup_is_rejected -- --exact`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib adapters::rpc::methods::key_impl::tests::test_export_public_material_v2 -- --exact`:
  passed
- `cargo test -p z00z_storage --release --features test-fast serialization::artifact::tests::test_version_v1_is_supported -- --exact`:
  passed
- `cargo test -p z00z_storage --release --features test-fast assets::store::whitebox_proofs::test_proof_blob_decode_legacy_v0_upgrades_root_bind -- --exact`:
  passed
- residual survivor scan across live crate and test paths: passed, with only
  intended hold rows still present, including `ClaimV2Err`, `CLAIM_V2_TAG`,
  `HKDF_INFO_REDB_*_V2`, `RANGE_PROOF_BITS_V2`, `MAX_PROOF_SIZE_V2`,
  `ProofBlobV0`, `ClaimNullRecV0`, `ReceiverCardRecordV1`, `claim_stmt_hash_v2`,
  `CLAIM_PROOF_V2`, `export_public_material_v2`, `encode_single_v2`,
  `encode_dual_v2`, `decode_v2`, `spend_v1`, `events_v1`, `multi_v1`, and
  `derive_key_v2_zero_padding`
- targeted Step 5 residue scan across touched files: passed, with only
  explicit version-scenario text mentions remaining after the last row-11
  `container_v1` occurrence was renamed on the export path
- canonical authority drift scan across the live `036-04` through `036-10`
  chain: passed, with historical TODO1 or suffix-spec mentions confined to
  non-authority warning text and older artifacts

## Review Loop

The review loop closed truthfully in three passes:

1. review pass 1 found no material code defect in the bounded Phase 036 slice,
   but it did find closure-truth drift because the canonical `036-07` backlog
   rows were still unchecked and the top-level roadmap synopsis still pointed at
   `036-09-PLAN.md`
2. those canonical planning surfaces were synchronized, this summary artifact
   was written, and the exact seam evidence was rechecked against the final
   closeout claim
3. review passes 2 and 3 then found no significant in-scope issues, making
   them the required consecutive clean review runs after the last fix cycle

The exact runtime commands for `/crypto-architect`, `/security-audit`, and
`/doublecheck` were not directly available in this environment, so the review
evidence used the repo-local best-effort path: canonical spec rereads,
planning-authority reconciliation, residual scans, deterministic targeted
reruns, and repeated review passes in YOLO mode.

## Current Boundary

This summary closes Plan 10 and the active Phase 036 chain. It does not choose
the next active phase. Phase 032 remains separately reopened in the roadmap,
but any return to execution must be selected explicitly rather than inferred
from the now-closed rename chain.
