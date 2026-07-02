---
phase: 065-Attack-Surface
plan: 065-03
status: complete
completed_at: 2026-07-01
next_plan: 065-04
summary_artifact_for: .planning/phases/065-Attack-Surface/065-03-PLAN.md
---

# 065-03 Summary: Release Build Hardening For Debug Surfaces

## Outcome

`065-03` is complete.

Release-capable wallet and simulator builds now fail closed when
`test-params-fast` or `wallet_debug_tools` are selected. The old shallow public
wallet secret-export path is gone, the remaining wallet dump helper is exposed
only through the explicit `internal_debug_tools` lane, and settlement cache or
scheduler corruption knobs no longer ship on release-capable storage surfaces.

The repository now carries executable proof for this policy. A dedicated audit
script checks the forbidden release feature matrix and source-shape guards, a CI
workflow runs that audit plus the wallet hardening tests, and the active review
prompt and full-verify skill no longer normalize release-shaped commands that
enable weakened KDF or secret-export features.

## Files Changed

- `.planning/phases/065-Attack-Surface/065-03-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.github/prompts/gsd-review-tasks-execution.prompt.md`
- `.github/skills/z00z-full-verify-gate/SKILL.md`
- `.github/workflows/release-safety-guards.yml`
- `crates/z00z_simulator/src/lib.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3/post_claim.rs`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/src/fixture_support/settlement_corpus.rs`
- `crates/z00z_storage/src/settlement/hjmt_cache.rs`
- `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`
- `crates/z00z_storage/tests/test_async_scheduler.rs`
- `crates/z00z_storage/tests/test_cache_recompute.rs`
- `crates/z00z_storage/tests/test_forest_cache.rs`
- `crates/z00z_wallets/src/db/mod.rs`
- `crates/z00z_wallets/src/lib.rs`
- `crates/z00z_wallets/src/redb_store/mod.rs`
- `crates/z00z_wallets/src/wallet/mod.rs`
- `crates/z00z_wallets/tests/test_live_boundary_claims.rs`
- `crates/z00z_wallets/tests/test_production_hardening.rs`
- `scripts/audit/audit_release_feature_guards.sh`

## Landed Changes

- `crates/z00z_wallets/src/lib.rs`
  - added release-capable compile guards for `test-params-fast` and
    `wallet_debug_tools`
  - introduced the explicit `internal_debug_tools` surface for the remaining
    debug wallet dump helper
- `crates/z00z_wallets/src/db/mod.rs`
  - removed the public `debug_export_wallet` re-export from the wallet DB facade
- `crates/z00z_wallets/src/wallet/mod.rs`
  - removed the public `debug_export_wallet` re-export from the wallet facade
- `crates/z00z_wallets/src/redb_store/mod.rs`
  - demoted debug-export re-exports to `pub(crate)` so the redb store no longer
    leaks secret-export helpers on a public path
- `crates/z00z_simulator/src/lib.rs`
  - added matching release-capable compile guards for `test-params-fast` and
    `wallet_debug_tools`
- `crates/z00z_simulator/src/scenario_1/stage_3/post_claim.rs`
  - switched simulator debug export usage to
    `z00z_wallets::internal_debug_tools::debug_export_wallet`
- `crates/z00z_storage/src/settlement/hjmt_cache.rs`
  - hid `set_forest_cache_test_limit`, `corrupt_forest_cache_for_test`, and
    `corrupt_journal_key_for_test` from release builds
- `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`
  - hid scheduler test tuning and reset hooks from release builds
- `crates/z00z_storage/src/fixture_support/settlement_corpus.rs`
  - made the fast-fixture scheduler relaxation a no-op on release-capable builds
    so the simulator release guard fails on the intended compile policy instead
    of on a downstream missing-hook side effect
- `crates/z00z_storage/tests/test_async_scheduler.rs`,
  `crates/z00z_storage/tests/test_cache_recompute.rs`,
  `crates/z00z_storage/tests/test_forest_cache.rs`
  - restricted the tests that depend on debug-only hooks so broad release
    validation stays honest about the production surface
- `scripts/audit/audit_release_feature_guards.sh`
  - added a fail-closed release matrix for:
    `z00z_wallets --release --features test-params-fast`,
    `z00z_wallets --release --features wallet_debug_tools`,
    `z00z_simulator --release --features wallet_debug_tools`, and
    `z00z_simulator --release --features test-params-fast`
  - added source audits that prove the old public secret-export paths and
    release-visible corruption hooks stay closed
- `.github/workflows/release-safety-guards.yml`
  - added CI coverage for the release guard audit and the wallet hardening tests
- `crates/z00z_wallets/tests/test_production_hardening.rs`
  - added source-audit coverage for release compile guards, the internal-only
    debug export path, release-hidden storage hooks, and the new audit
    artifacts
- `crates/z00z_wallets/tests/test_live_boundary_claims.rs`
  - added doc-truth coverage that rejects release guidance normalizing
    `test-params-fast` or `wallet_debug_tools` on public release lanes
- `.github/prompts/gsd-review-tasks-execution.prompt.md`,
  `.github/skills/z00z-full-verify-gate/SKILL.md`,
  `crates/z00z_storage/benches/settlement_benches.md`
  - removed release-shaped command guidance that previously normalized the
    forbidden feature combinations

## Validation

Commands green on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`
- `bash scripts/audit/audit_release_feature_guards.sh`
- `cargo test --release`
- `git diff --check -- .github/prompts/gsd-review-tasks-execution.prompt.md .github/skills/z00z-full-verify-gate/SKILL.md .github/workflows/release-safety-guards.yml crates/z00z_simulator/src/lib.rs crates/z00z_simulator/src/scenario_1/stage_3/post_claim.rs crates/z00z_storage/benches/settlement_benches.md crates/z00z_storage/src/fixture_support/settlement_corpus.rs crates/z00z_storage/src/settlement/hjmt_cache.rs crates/z00z_storage/src/settlement/hjmt_scheduler.rs crates/z00z_storage/tests/test_async_scheduler.rs crates/z00z_storage/tests/test_cache_recompute.rs crates/z00z_storage/tests/test_forest_cache.rs crates/z00z_wallets/src/db/mod.rs crates/z00z_wallets/src/lib.rs crates/z00z_wallets/src/redb_store/mod.rs crates/z00z_wallets/src/wallet/mod.rs crates/z00z_wallets/tests/test_live_boundary_claims.rs crates/z00z_wallets/tests/test_production_hardening.rs scripts/audit/audit_release_feature_guards.sh`

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still does not provide a reliable callable review path for
this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-03-PLAN.md current_task="Release Build Hardening For Debug Surfaces"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 30s gsd --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-03-PLAN.md current_task="Release Build Hardening For Debug Surfaces" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 30s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-03-PLAN.md current_task="Release Build Hardening For Debug Surfaces" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `065-03-PLAN.md`, `065-TODO.md`, and the named code anchors for
    wallet, simulator, and storage release-surface policy.
  - Result: clean. Release-capable compile guards now exist on the public
    wallet and simulator lanes, the old shallow secret-export paths are gone,
    and the remaining debug export path is explicit and internal-only.
- Pass 2
  - Re-read the authority-doc surfaces, workflow, audit script, and the new
    source-audit tests.
  - Result: clean. The active review prompt, the full-verify skill, and the
    bench closeout doc no longer normalize release-shaped commands with
    `test-params-fast` or `wallet_debug_tools`, and the audit script proves the
    intended fail-closed matrix plus source shape.
- Pass 3
  - Re-checked the targeted wallet tests, the release guard audit matrix, and
    the full `cargo test --release` run against the final tree.
  - Result: clean. Forbidden release feature combinations fail closed, the
    release-hidden storage hooks do not regress broad release validation, and
    the full workspace release suite completed green.

Passes 2 and 3 were consecutive clean manual review runs for the actual
release-surface closure slice before moving the phase to the next plan.

## Closeout

`065-03` closes `WS-03` by making forbidden release feature combinations fail
closed, removing the old public wallet secret-export path, hiding cache and
scheduler corruption hooks from release builds, and pinning the policy in
tests, scripts, CI, and live authority docs. The active Phase 065 lane moves to
`065-04-PLAN.md`.
