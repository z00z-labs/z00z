# 035-16 Summary

## Scope

This summary records the completion state for `035-16-PLAN.md`, covering task
`035-38 V2 Memo Receive-Path Enablement`, task
`035-39 Stealth Additions Validation Wave`, and task
`035-40 Stealth Additions Acceptance Gate`.

## Outcome

Plan 16 is fully closed.

Phase 035 now has the final stealth-additions slice closed on repository-
backed receive-path, validation, and acceptance evidence rather than on the
earlier fail-closed placeholder. The live wallet seam can now classify and
decode the bounded V2 memo lane, memo data stays private decrypted metadata
instead of leaking into public leaf fields, malformed memo payloads fail
closed, and the stealth lane closes only on the approved A -> B -> C additions
without importing broader routing, proof, or Poseidon-scope claims.

## Repository Changes

- `.planning/phases/035-mix2-fixes/035-TODO.md` now records `035-38`,
  `035-39`, and `035-40` as closed checklist items and promotes the final
  stealth validation rows from planned to completed.
- `.planning/phases/035-mix2-fixes/035-16-SUMMARY.md` now captures the closeout
  evidence and boundary for the final stealth-additions slice.
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` and
  `crates/z00z_wallets/src/core/address/stealth_scanner.rs` now use the
  explicit version-aware detected-pack seam so V2 memo outputs are decoded on
  the approved live path while keeping memo bytes private and fail-closed.
- `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` now stays aligned
  with the same live detected-pack seam for receiver-card handling.
- `crates/z00z_wallets/src/core/address/leaf_scan.rs`,
  `crates/z00z_wallets/src/core/address/mod.rs`,
  `crates/z00z_wallets/src/core/address/stealth_scanner/types_receive.rs`,
  `crates/z00z_wallets/src/core/tx/witness_gate.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_runtime_support.rs`,
  and `crates/z00z_core/src/assets/mod.rs` now carry the version-aware
  detected-pack, downstream compatibility, and DTO plumbing needed by the live
  receive path.
- `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` now provides dedicated
  regression coverage for V2 memo classification, memo privacy, and malformed
  memo rejection on the receive path.
- `.planning/STATE.md` and `.planning/ROADMAP.md` now advance the next active
  handoff to `035-17-PLAN.md`.

## Validation

- The repository now carries dedicated V2 memo receive-path regression coverage
  in `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` for version-aware
  classification, private memo handling, and fail-closed malformed payloads.
- Plan 16 acceptance closed only after the validation wave reconfirmed the
  intended A -> B -> C execution order, preserved the explicit out-of-scope
  list, and kept the wallet or scanner memo path aligned with the approved live
  seam rather than broadening public-address, routing, or proof semantics.

## Current Boundary

This summary closes only the final Phase 035 stealth-additions slice for
`035-38`, `035-39`, and `035-40`. It does not claim closure for the unrelated
`035-a*` planning rename wave or for the subsequent rename-focused work that
begins at the newly active `035-17-PLAN.md`.
