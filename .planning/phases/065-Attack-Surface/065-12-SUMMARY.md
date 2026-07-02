---
phase: 065-Attack-Surface
plan: 065-12
status: complete
completed_at: 2026-07-02
next_plan: 065-13
summary_artifact_for: .planning/phases/065-Attack-Surface/065-12-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 065-12 Summary: Checkpoint Lineage And Storage Determinism Closure

## 🎯 Outcome

`065-12` is complete.

`VR-12` closes on one canonical checkpoint-lineage and storage-determinism
story. The current runtime and storage code already enforced the required
single-path route rollout, quorum freeze, same-lineage takeover, root
generation, checkpoint finalization, and live-guardrail contracts. The real
remaining blocker in this slice was execution-path drift in the acceptance
gates: `z00z_aggregators` forwarded `test-params-fast` into `z00z_wallets`,
which made the required release-mode aggregator tests fail before reaching the
runtime or storage logic under review.

Removing that single invalid feature edge restored the canonical release-safe
test path without weakening the wallet release guard. After the fix:

- the required `z00z_aggregators` release tests with
  `--features test-params-fast` now execute green;
- the required `z00z_storage` release tests remain green;
- the broad `cargo test --release` gate is green;
- `bash scripts/audit/audit_release_feature_guards.sh` stays green, proving the
  direct wallet and simulator release guards still fail closed on prohibited
  debug or fast-test features;
- `cargo tree -e features -p z00z_aggregators --features test-params-fast -i z00z_wallets`
  now shows that aggregator release-fast tests no longer activate the wallet
  `test-params-fast` feature.

## 📦 Files Changed

- `.planning/phases/065-Attack-Surface/065-12-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/Cargo.toml`

## 🔧 Landed Changes

- Acceptance-path repair
  - `crates/z00z_runtime/aggregators/Cargo.toml` no longer forwards
    `z00z_wallets/test-params-fast` from the aggregator
    `test-params-fast` feature.
  - The aggregator release-fast acceptance commands now exercise the intended
    runtime and storage proofs instead of tripping the unrelated wallet
    production hardening guard.
- Canonical guard preservation
  - `crates/z00z_wallets/src/lib.rs` still contains the release-capable
    `test-params-fast` compile guard.
  - `scripts/audit/audit_release_feature_guards.sh` still proves that direct
    release builds for `z00z_wallets` and `z00z_simulator` fail closed when
    forbidden feature combinations are requested.
- Runtime and storage closure evidence
  - The current-tree focused release suites prove route-rollout digest and
    generation binding, checkpoint-ack activation gating, same-term split-brain
    freeze, same-lineage takeover, publication and root-generation monotonicity,
    deterministic checkpoint finalization, and storage live-guardrail
    invariants on the existing canonical code paths.

## ✅ Validation

Commands and evidence used for `065-12` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_consensus`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_route_rollout`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_live_guardrails`
- `cargo test --release -p z00z_storage --test test_checkpoint_finalization`
- `cargo test --release -p z00z_storage --test test_hjmt_root_generation`
- `cargo test --release -p z00z_storage --test test_live_guardrails`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`
- `cargo tree -e features -p z00z_aggregators --features test-params-fast -i z00z_wallets`

Observed proof points:

- `bootstrap_tests.sh` completed green.
- The three focused `z00z_aggregators` release-fast suites and the focused
  aggregator live-guardrail suite completed green after the feature-edge fix.
- The three focused `z00z_storage` suites completed green.
- The broad `cargo test --release` workspace gate completed green.
- `audit_release_feature_guards.sh` completed green, so the wallet and
  simulator release guards remain fail-closed.
- The post-fix feature tree shows `z00z_wallets` under the aggregator release
  test path with default features only; the wallet `test-params-fast` feature
  is no longer activated transitively.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a usable automated review path for
this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-12-PLAN.md current_task="Checkpoint Lineage And Storage Determinism Closure" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-12-PLAN.md current_task="Checkpoint Lineage And Storage Determinism Closure" --yolo'`
  - Result: exited with code `1` and reported `Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-12-PLAN.md current_task="Checkpoint Lineage And Storage Determinism Closure" --yolo'`
  - Result: exited with code `1` and reported `Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `065-12-PLAN.md`, `065-TODO.md`, `065-CONTEXT.md`, the residual
    verification-report sections for `VR-12`, the focused aggregator or storage
    code anchors, and the focused release tests.
  - Result: found that the current runtime and storage invariants were already
    implemented, and the real blocker was the transitive aggregator-to-wallet
    `test-params-fast` feature edge that prevented the mandated release tests
    from reaching the target logic.
- Pass 2
  - Re-checked the feature graph, patched the aggregator feature forwarder, and
    reran the focused `z00z_aggregators` and `z00z_storage` release suites.
  - Result: clean for the in-scope runtime and storage lineage hypotheses; the
    focused acceptance evidence now executed on the intended canonical paths.
- Pass 3
  - Ran the broad `cargo test --release`, reran
    `bash scripts/audit/audit_release_feature_guards.sh`, and checked the
    post-fix feature tree for `z00z_wallets` under the aggregator release-fast
    path.
  - Result: clean for the `065-12` scope. No remaining material drift was
    found.

Passes 2 and 3 were consecutive clean review runs after the final in-scope fix.

## 🧾 Closeout

`065-12` closes `VR-12` by proving that the project-owned runtime and storage
surfaces already hold one canonical lineage and deterministic support story,
and by removing the last invalid release-test feature edge that obscured that
fact. The next active lane is `065-13`.
