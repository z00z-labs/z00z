# 035-17 Summary

## Scope

This summary records the completion state for `035-17-PLAN.md`, covering task
`035-41 Rename Scope Freeze`, task
`035-42 Live Rename Manifest And Lane Split`, and task
`035-43 File Rename Wave A - Test And Support Files`.

## Outcome

Plan 17 is fully closed.

Phase 035 now has the first rename-focused slice closed on repository-backed
scope, manifest, and static review evidence rather than on stale planning
continuity. The rename authority is frozen to the recovered curated table plus
the high-confidence delta, the raw 814-row matrix remains inventory-only, the
live rename manifest is split into file-first and signature-after lanes, and
the approved Wave A test/support file renames have landed before any later
mirror or declaration rewrite claims begin.

## Repository Changes

- `.planning/phases/035-mix2-fixes/035-a6-renames.md` now carries the frozen
  curated rename authority, the live lane split, and the explicit lockstep
  mirror markers for the approved Wave A rows.
- `.planning/phases/035-mix2-fixes/035-TODO.md` now records `035-41`,
  `035-42`, and `035-43` as closed checklist items while leaving the later
  rename waves pending.
- `crates/z00z_core/src/assets/test_asset_suite.rs`,
  `crates/z00z_core/src/genesis/test_genesis_suite.rs`,
  `crates/z00z_crypto/src/test_aead_suite.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/test_tx_lane_runtime_suite.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/test_tx_lane_runtime_support.rs`,
  `crates/z00z_storage/src/checkpoint/test_artifact_suite.rs`,
  `crates/z00z_utils/src/io/test_fs_suite.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_suite.rs`, and
  `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_body.rs` now hold
  the approved Wave A filesystem renames for the curated test/support rows.
- `crates/z00z_core/src/assets/assets.rs`,
  `crates/z00z_core/src/genesis/genesis.rs`,
  `crates/z00z_crypto/src/aead.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`,
  `crates/z00z_storage/src/checkpoint/artifact.rs`,
  `crates/z00z_utils/src/io/fs.rs`, and
  `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_tests.rs` now point at
  the new Wave A filenames on the approved mirror rows that belong inside this
  plan.
- `.planning/STATE.md` and `.planning/ROADMAP.md` now advance the next active
  handoff to `035-18-PLAN.md`.

## Validation

- `.planning/phases/035-mix2-fixes/035-17-REVIEW.md` is clean and records zero
  findings across the approved Plan 17 review surface.
- The review evidence confirms that the approved file-first rows (`1, 6, 8,
  22, 23, 26, 30, 37, 39`) landed as file moves, that the required lockstep
  mirror row `24` was updated, and that the deferred wallets `tx_impl` child
  include rows (`40-45`) remain intentionally outside Wave A.
- Closure is accepted only on repository-static rename-slice evidence for
  `035-41` through `035-43`; it does not rely on unrelated mixed-worktree code
  changes outside this rename lane.

## Current Boundary

This summary closes only the first Phase 035 curated rename slice for
`035-41`, `035-42`, and `035-43`. It does not claim closure for the later
wallet DB or egui file renames, the remaining mirror rewrites, the curated
declaration or callsite rename waves, or unrelated mixed-worktree changes that
coexist in the repository while the next active handoff advances to
`035-18-PLAN.md`.
