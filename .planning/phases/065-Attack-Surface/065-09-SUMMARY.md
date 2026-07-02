---
phase: 065-Attack-Surface
plan: 065-09
status: complete
completed_at: 2026-07-01
next_plan: none
summary_artifact_for: .planning/phases/065-Attack-Surface/065-09-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 065-09 Summary: Final Narrowed-Claim Source Sweep

## 🎯 Outcome

`065-09` is complete.

`WS-09` now closes on one honest repo wording story. Live docs, readmes, and
planning anchors no longer point at the retired asset-registry path under
`z00z_core/src/assets/` as if it were still a canonical authority surface, and
the narrowed Phase 065 leftovers stay retired behind an executable wording
audit plus a live guardrail test.

This closes the last open Phase 065 workstream. Phase 065 is now complete on
the existing `.planning/phases/065-Attack-Surface/` folder only.

## 📦 Files Changed

- `.planning/phases/065-Attack-Surface/065-09-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/codebase/STRUCTURE.md`
- `.planning/codebase/ARCHITECTURE.md`
- `.planning/phases/profiling-comprehensive.md`
- `.planning/phases/069-New-Scenarios/066-TODO.md`
- `crates/z00z_core/tests/test_live_guardrails.rs`
- `crates/z00z_storage/src/settlement/root_types.md`
- `scripts/audit_phase065_narrowed_wording.sh`
- `wiki/01-getting-started/workspace-map.md`

## 🔧 Landed Changes

- Canonical path and authority cleanup
  - `STRUCTURE.md` and `ARCHITECTURE.md` now name
    `crates/z00z_core/configs/devnet_genesis_config.yaml` as the canonical
    bootstrap manifest and
    `crates/z00z_core/configs/devnet_assets_config.yaml` as secondary registry
    data.
  - `profiling-comprehensive.md` no longer claims genesis inputs are "frozen in
    `assets_config.yaml`"; it now points at the typed genesis manifest plus its
    referenced subfiles.
  - `root_types.md` now treats the genesis manifest and its referenced
    rights/policies/vouchers subfiles as the canonical regeneration inputs and
    explicitly demotes `devnet_assets_config.yaml` to registry/example data.
- Repo-wide residue cleanup
  - `wiki/01-getting-started/workspace-map.md` now uses the full canonical
    `configs/devnet_assets_config.yaml` path instead of the loose
    `assets_config.yaml` shorthand.
  - the future planning anchor in
    `.planning/phases/069-New-Scenarios/066-TODO.md` no longer points at the
    retired asset-path fixture; it now points at
    `crates/z00z_core/configs/devnet_rights_config.yaml`.
- Executable wording gate
  - `scripts/audit_phase065_narrowed_wording.sh` now fails on the retired
    `src/assets/assets_config.yaml` path, the stale regeneration wording, and
    the old profiling claim.
  - the same script also pins explicit allowlists for the few Phase 065 docs
    that must keep retired phrases only as retired-history notes.
  - `crates/z00z_core/tests/test_live_guardrails.rs` now includes a compact
    `phase065_wording_guard` test that proves the audit script exists and the
    repaired docs keep the new canonical strings.

## ✅ Validation

Commands green during the final `065-09` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `bash scripts/audit_phase065_narrowed_wording.sh`
- `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
- `cargo test --release`

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a usable automated review path for
this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-09-PLAN.md current_task="Final Narrowed-Claim Source Sweep"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-09-PLAN.md current_task="Final Narrowed-Claim Source Sweep" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66675 > 38936`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-09-PLAN.md current_task="Final Narrowed-Claim Source Sweep" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 82817 > 38936`

Equivalent workspace-first review was executed manually against the same
scope.

- Pass 1
  - Re-read `065-09-PLAN.md`, `065-TODO.md`, `065-CONTEXT.md`, the current
    codebase docs, and the live crate docs named by `WS-09`.
  - Result: found live canonical-path drift in `STRUCTURE.md`,
    `ARCHITECTURE.md`, `profiling-comprehensive.md`, `root_types.md`, the
    future `066-TODO.md` anchor, and `workspace-map.md`. Fixed those surfaces,
    then added one repo-owned wording audit script plus one compact live
    guardrail test instead of creating a parallel audit layer.
- Pass 2
  - Ran `bash scripts/audit_phase065_narrowed_wording.sh` and
    `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`,
    then re-read the resulting failures against the touched docs and test
    guard.
  - Result: found two small truth mismatches after the first landing: the
    architecture doc still named only the config root instead of the exact
    genesis manifest path, and one guardrail assertion expected an unsplit
    string from `root_types.md`. Fixed both and reran the same checks clean.
- Pass 3
  - Re-ran `bash scripts/audit_phase065_narrowed_wording.sh`, re-scanned the
    live repo for the retired `src/assets/assets_config.yaml` path and the old
    regeneration wording, and then ran the broad `cargo test --release` gate.
  - Result: clean. The wording audit passed, the only remaining exact-string
    hits were the intentional denylist anchors in the audit script and the
    guardrail test, and the full release workspace gate passed.
- Pass 4
  - Re-read the newly written `065-09-SUMMARY.md`, then re-ran
    `bash scripts/audit_phase065_narrowed_wording.sh` and
    `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`.
  - Result: found one closeout-only regression where the first summary draft
    echoed the retired exact `src/assets/...` path literal. Removed that
    literal, reran the wording audit, and reran the targeted guardrail test
    clean.
- Pass 5
  - Re-ran `bash scripts/audit_phase065_narrowed_wording.sh` and a final
    residue grep for the retired exact path across live roots.
  - Result: clean. The wording audit passed, and the only remaining exact-path
    hits were the intentional denylist anchors inside the audit script and the
    live guardrail test.

Passes 4 and 5 were consecutive clean manual review runs after the last
in-scope fix.

## 🧾 Closeout

`065-09` closes `WS-09` by turning the final narrowed historical leftovers into
an executable repo truth gate instead of a prose reminder. The canonical
bootstrap story now points to `devnet_genesis_config.yaml`, the secondary
registry story now points to `devnet_assets_config.yaml`, the final stale
planning anchor is updated, and a repo-owned audit script plus live guardrail
test keep those truths pinned.

With `065-01` through `065-09` summary-backed complete and the mandatory final
docs sweep proven on green release gates, Phase 065 is complete and no active
Phase 065 lane remains.
