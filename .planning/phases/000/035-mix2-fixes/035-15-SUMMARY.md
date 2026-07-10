# 035-15 Summary

## Scope

This summary records the completion state for `035-15-PLAN.md`, covering task
`035-35 Stealth Derivation Vector Freeze`, task
`035-36 Derivation Drift Regression Sweep`, and task
`035-37 V2 Memo Contract Definition`.

## Outcome

Plan 15 is fully closed.

Phase 035 now has the second stealth-additions slice closed on repository-
backed vector and decode-boundary evidence rather than planning intent. The
wallet test seam now freezes both card-bound and request-bound derivation
families, drift checks prove that the context families do not silently collapse,
and the core asset layer now exposes an explicit bounded V2 memo decode
contract without changing the live V1 lane or enabling wallet receive support
prematurely.

## Repository Changes

- `.planning/phases/035-mix2-fixes/035-TODO.md` now records `035-35`,
  `035-36`, and `035-37` as closed checklist items.
- `.planning/phases/035-mix2-fixes/035-15-REVIEW.md` records the final
  multi-pass clean review result for the Plan 15 scope.
- `.planning/phases/035-mix2-fixes/035-15-SUMMARY.md` captures the closeout
  evidence and closure boundary for the derivation-freeze and V2 memo slice.
- `crates/z00z_wallets/tests/fixtures/stealth_kdf_vectors.yaml` now freezes
  canonical `base-card` and `request-bound` derivation vectors in the existing
  wallet fixture style.
- `crates/z00z_wallets/tests/test_stealth_kdf_vectors.rs` now verifies the
  frozen vectors, argument-order drift, and request-bound versus card-bound
  divergence on the live wallet seam.
- `crates/z00z_core/src/assets/leaf.rs` now adds `AssetPackPlainV2Memo`,
  `DecodedAssetPack`, bounded memo errors, and strict version-aware decode
  entrypoints while preserving the V1 contract.
- `crates/z00z_core/src/assets/leaf_tests.rs` now exercises the V2 memo round-
  trip, oversize rejection, malformed payload rejection, and explicit version
  lanes.
- `crates/z00z_core/src/assets/mod.rs` now re-exports the explicit version-aware
  decode surface and the new V2 boundary types.
- `.planning/STATE.md` now advances continuity to `035-16-PLAN.md`.
- The verified Phase 035 baseline through Plan 15 remains recorded in the
  planning continuity artifacts and closeout evidence chain.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`:
  passed with `=== BOOTSTRAP COMPLETE ===` on the fresh rerun before closeout.
- `cargo test -p z00z_wallets --test test_stealth_kdf_vectors --release --features test-fast --features wallet_debug_dump`:
  passed (`4 passed; 0 failed`).
- `cargo test -p z00z_core test_v2_memo --release --features test-fast`:
  passed (`5 passed; 0 failed`).
- `cargo test -p z00z_core test_decode_asset_pack --release --features test-fast`:
  passed (`3 passed; 0 failed`).

## Review Loop

The mandatory `GSD-Review-Tasks-Execution` loop ran five times on the Plan 15
scope.

- Earlier review passes found and drove fixes for a serial-based decode shortcut
  that wrongly inferred format from `serial_id`.
- Later review passes found and drove fixes for publicly Serde-deserializable V2
  boundary types that bypassed the bounded memo decode contract.
- Final pass 4: clean.
- Final pass 5: clean.

That satisfies the required closeout rule of at least three review executions
with at least two consecutive clean passes before closure.

## Current Boundary

This summary closes only the Phase 035 derivation-freeze, drift-regression, and
core-side V2 memo contract slice for `035-35`, `035-36`, and `035-37`. It does
not claim wallet receive-path enablement for `V2Memo`; that later work remains
deferred to the subsequent Phase 035 receive-path plans beginning after the
newly active `035-16-PLAN.md`.
