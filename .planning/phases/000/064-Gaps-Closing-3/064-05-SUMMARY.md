---
phase: 064-Gaps-Closing-3
plan: 064-05
status: complete
completed_at: 2026-06-30
next_plan: none
summary_artifact_for: .planning/phases/064-Gaps-Closing-3/064-05-PLAN.md
---

# 064-05 Summary: Core Truth Boundaries And Repository Guardrails

## Outcome

`064-05` is complete. `PLAN-064-G05` now closes `REC-064-P2-03`,
`REC-064-P2-04`, `REC-064-P2-05`, `REC-064-P2-06`, `REC-064-P2-08`, and
`REC-064-P2-09`, and it completes the full Phase 064 packet on the existing
`.planning/phases/064-Gaps-Closing-3/` folder only.

The final slice keeps core/genesis wording truthful without reopening earlier
simulator or wallet closure work. `genesis-caveats.md` now uses local-path
source refs only, and `crates/z00z_core/tests/test_live_guardrails.rs`
explicitly pins the canonical genesis thread-count and local-path caveat
strings while forbidding GitHub blob links or stray OnionNet/remote-chain
claims in that page.

Deferred network and fraud-surface boundaries also stay honest on the current
tree. `crates/z00z_wallets/tests/test_live_boundary_claims.rs` now proves that
the whitepaper, roadmap, OnionNet README, wallet app kernel, and chain client
all keep OnionNet, real remote chain transport, real DA provider execution,
slashing, and fraud-engine claims explicitly deferred instead of pretending
those surfaces are already shipped.

Repository boundary discipline is now executable instead of advisory.
`scripts/audit_z00z_utils_boundary.sh`,
`scripts/audit_crypto_facade.sh`, `scripts/audit_extensions_boundary.sh`, and
`scripts/audit_local_docs_links.sh` each fail closed on their respective
boundary drifts, and `.github/workflows/boundary-guards.yml` wires those
audits together with the new core and wallet boundary tests in CI.

The supporting Phase 064 docs corpus was truth-restored in the same slice.
`scenario-pipeline.md`, `genesis-caveats.md`, `wallet-stub-surface.md`, and
`scenario1-object-artifacts.md` now use local-path source refs, and
`scripts/audit_local_docs_links.sh` derives the attached docs corpus directly
from `064-TEST-SPEC.md` so the full Phase 064 packet rejects internal GitHub
blob links instead of only spot-checking a subset of pages. Phase 064 source
citations are now offline-safe and local-path canonical. No alias, shim,
duplicate authority layer, or parallel boundary story was introduced.

## Files Changed

- `wiki/03-core-protocol/genesis-caveats.md`
- `wiki/06-simulator-and-quality/scenario-pipeline.md`
- `crates/z00z_core/tests/test_live_guardrails.rs`
- `crates/z00z_wallets/tests/test_live_boundary_claims.rs`
- `scripts/audit_z00z_utils_boundary.sh`
- `scripts/audit_crypto_facade.sh`
- `scripts/audit_extensions_boundary.sh`
- `scripts/audit_local_docs_links.sh`
- `.github/workflows/boundary-guards.yml`
- `.planning/phases/064-Gaps-Closing-3/064-05-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`
- `bash scripts/audit_z00z_utils_boundary.sh`
- `bash scripts/audit_crypto_facade.sh`
- `bash scripts/audit_extensions_boundary.sh`
- `bash scripts/audit_local_docs_links.sh`
- `cargo test --release`
- `git diff --check -- wiki/03-core-protocol/genesis-caveats.md wiki/06-simulator-and-quality/scenario-pipeline.md crates/z00z_core/tests/test_live_guardrails.rs crates/z00z_wallets/tests/test_live_boundary_claims.rs scripts/audit_z00z_utils_boundary.sh scripts/audit_crypto_facade.sh scripts/audit_extensions_boundary.sh scripts/audit_local_docs_links.sh .github/workflows/boundary-guards.yml`
- `rg -n 'https://github\\.com/z00z-labs/z00z/blob|tari_crypto::' wiki/03-core-protocol/genesis-caveats.md wiki/06-simulator-and-quality/scenario-pipeline.md crates -g '!crates/z00z_crypto/**'`
- `rg -n 'privacy against network-level traffic analysis is not yet a shipped base-layer guarantee|OnionNet currently exists as a reserved boundary crate|there is no fully landed slashing or fraud-proof execution engine|Real remote-node transport remains an explicit adapter-only seam|remote node adapter is not configured' docs/Z00Z-Main-Whitepaper.md docs/tech-papers/Z00Z-Roadmap-Blueprint.md crates/z00z_networks/onionnet/README.md crates/z00z_wallets/src/app/app_kernel.rs crates/z00z_wallets/src/chain/chain_client_impl.rs`

Result:

- the mandatory bootstrap gate had already completed green before `064-05`
  execution continued;
- the targeted `z00z_core` and `z00z_wallets` release-mode acceptance tests
  for `PLAN-064-G05` passed;
- all four executable boundary/citation audit scripts passed;
- the broad `cargo test --release` rerun still honestly reproduces the
  current-tree `z00z_core` genesis/config blockers outside the modified
  `064-05` boundary slice:
  `genesis::genesis_manifest::test_genesis_plan_rights_only_requires_policy_resolution_when_needed`
  fails with
  `ConfigParseFailed("wallet profile validator_mandate_lock_v1 references unknown locked_asset_id z00z")`,
  and `genesis::genesis_rights::test_genesis_rights_deterministic`
  still reports current rights snapshot drift rooted in
  `crates/z00z_core/configs/devnet_genesis_config.yaml`;
- the scoped `git diff --check` stayed clean;
- the checked Phase 064 docs corpus and crate sources no longer contain
  internal GitHub blob links or direct `tari_crypto::` imports outside the
  `z00z_crypto` facade.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice:

- Attempt 1
  - `timeout 90s gsd --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md current_task="Core truth boundaries and repository guardrails" --yolo'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66492 > 38936`
- Attempt 2
  - `timeout 90s gsd --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md current_task="Keep non-live OnionNet remote-chain and DA claims explicitly deferred" --yolo'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66497 > 38936`
- Attempt 3
  - `timeout 90s gsd --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md current_task="Turn repo boundary and local-link rules into executable audits" --yolo'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66496 > 38936`

Equivalent workspace-first review passes were executed manually against the
same scope.

- Pass 1
  - Re-read `064-05-PLAN.md`, the touched wiki pages, the new audit scripts,
    the new boundary tests, and the current truth-boundary anchors in the
    whitepaper, roadmap, OnionNet README, app kernel, and chain client.
  - `bash scripts/audit_local_docs_links.sh`
  - `rg -n 'tari_crypto::' crates -g '!crates/z00z_crypto/**'`
  - Result: the local-path and crypto-facade boundary claims matched the
    current tree with no stale blob links across the attached Phase 064 docs
    corpus and no direct vendor imports on the checked crate surface
- Pass 2
  - `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
  - `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`
  - `bash scripts/audit_z00z_utils_boundary.sh`
  - `bash scripts/audit_crypto_facade.sh`
  - `bash scripts/audit_extensions_boundary.sh`
  - `bash scripts/audit_local_docs_links.sh`
  - `git diff --check -- wiki/03-core-protocol/genesis-caveats.md wiki/06-simulator-and-quality/scenario-pipeline.md crates/z00z_core/tests/test_live_guardrails.rs crates/z00z_wallets/tests/test_live_boundary_claims.rs scripts/audit_z00z_utils_boundary.sh scripts/audit_crypto_facade.sh scripts/audit_extensions_boundary.sh scripts/audit_local_docs_links.sh .github/workflows/boundary-guards.yml`
  - Result: clean
- Pass 3
  - `cargo test --release`
  - Result: no significant issues remained in the modified `064-05` slice;
    only the current-tree `z00z_core` genesis/config blockers outside the
    changed scope were reproduced

Passes 2 and 3 were consecutive clean review runs for the modified `064-05`
scope.

## Closeout

- `064-05-SUMMARY.md` closes `PLAN-064-G05` and completes the full Phase 064
  packet on the existing `.planning/phases/064-Gaps-Closing-3/` directory
  only.
- Phase 064 now has no active execution lane remaining.
- The remaining broad workspace blocker is still the pre-existing
  `z00z_core` genesis/config surface, not the modified `064-05`
  boundary-guard slice.
