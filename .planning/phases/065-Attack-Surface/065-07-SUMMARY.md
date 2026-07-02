---
phase: 065-Attack-Surface
plan: 065-07
status: complete
completed_at: 2026-07-01
next_plan: 065-08
summary_artifact_for: .planning/phases/065-Attack-Surface/065-07-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 065-07 Summary: Fail-Closed Construction, Redaction, And Meta-Gates

## 🎯 Outcome

`065-07` is complete.

`WS-07` now closes on one non-panicking local startup seam, redacted
transport summaries, and executable repository meta-gates. The live
`SettlementStore::new()` path no longer panics or follows env-root drift,
`WasmRpcClient` no longer logs raw request or response bodies, touched secret
comparisons are constant-time, and the absorbed attack-surface policy classes
now fail through checked scripts and CI rather than memory.

## 📦 Files Changed

- `.github/workflows/security-hygiene-guards.yml`
- `.planning/phases/065-Attack-Surface/065-07-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_networks/rpc/src/wasm_client.rs`
- `crates/z00z_networks/rpc/tests/test_wasm_client_redaction.rs`
- `crates/z00z_storage/src/backend/redb/helpers.rs`
- `crates/z00z_storage/src/backend/redb/mod.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_wallets/src/redb_store/debug_export.rs`
- `crates/z00z_wallets/src/rpc/security_types.rs`
- `crates/z00z_wallets/src/services/wallet_session_manager.rs`
- `crates/z00z_wallets/src/tx/spend_verification.rs`
- `docs/tech-papers/benchmarks.md`
- `scripts/audit/audit_boundary_panic_hygiene.sh`
- `scripts/audit/audit_crypto_rng_hygiene.sh`
- `scripts/audit/audit_log_redaction_hygiene.sh`
- `scripts/audit/audit_secret_eq_hygiene.sh`
- `scripts/audit/audit_secret_type_hygiene.sh`

## 🔧 Landed Changes

- Fail-closed settlement-store startup
  - `SettlementStore::new()` now creates a managed local store for tests,
    simulations, and local benches, ignores `Z00Z_STORAGE_REDB_ROOT`, and
    never panics on startup drift.
  - `StoragePlane::managed_default()` plus `managed_default_backend()` keep a
    managed local RedB backend outside `#[cfg(test)]`.
  - managed-root helpers now allocate a fresh `run-*` child path and reject
    silent reuse; `SettlementStore::try_new()` and `load(...)` remain the
    canonical fallible operator-bound durable seams.
- Transport redaction and secret equality hygiene
  - `WasmRpcClient` now logs only endpoint events plus request/response shape
    summaries; the old raw params and raw response strings are gone.
  - `SessionToken` no longer derives `Debug`; it uses a manual redacted
    formatter that emits `"<redacted>"`.
  - session-token, receiver-secret, and debug-export master-key comparisons
    now route through constant-time helpers instead of direct equality.
- Permanent meta-gates
  - new source-audit scripts now enforce secret-type, secret-equality,
    crypto-RNG, boundary-panic, and log-redaction hygiene.
  - `.github/workflows/security-hygiene-guards.yml` wires those audits into
    repository CI.
  - `test_live_guardrails` now pins the workflow and script artifacts and the
    truthful `SettlementStore::new()` bench contract.

## ✅ Validation

Commands green during the final `065-07` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `bash scripts/audit/audit_secret_type_hygiene.sh`
- `bash scripts/audit/audit_secret_eq_hygiene.sh`
- `bash scripts/audit/audit_crypto_rng_hygiene.sh`
- `bash scripts/audit/audit_boundary_panic_hygiene.sh`
- `bash scripts/audit/audit_log_redaction_hygiene.sh`
- `cargo test --release -p z00z_networks_rpc -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_rpc_logging_acceptance -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy -- --nocapture`
- `cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture`
- `cargo test --release -p z00z_storage --test test_hjmt_adaptive_policy_proofs -- --nocapture`
- `cargo test --release -p z00z_rollup_node 'da::tests::test_local_adapter_roundtrip' -- --exact --nocapture`
- `cargo test --release`

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a callable review path for this
slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-07-PLAN.md current_task="Fail-Closed Construction, Redaction, And Meta-Gates"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-07-PLAN.md current_task="Fail-Closed Construction, Redaction, And Meta-Gates" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66679 > 38936`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-07-PLAN.md current_task="Fail-Closed Construction, Redaction, And Meta-Gates" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 82821 > 38936`

Equivalent workspace-first review was executed manually against the same
scope.

- Pass 1
  - Re-read `065-07-PLAN.md`, `065-TODO.md`, `065-CONTEXT.md`,
    `store.rs`, `wasm_client.rs`, `test_live_guardrails.rs`, and the new
    audit scripts.
  - Result: found stale benchmark wording and one direct `std::fs::write`
    call in the new guardrail test. Fixed both so the docs and the guardrail
    path matched the intended one-source rules.
- Pass 2
  - Re-ran the broad workspace release gate and traced the new failure to the
    initial `SettlementStore::new()` fallback choice.
  - Result: `StoragePlane::off()` broke adaptive-policy historical validation,
    so `SettlementStore::new()` was moved to a managed local backend while
    keeping `try_new()` and `load(...)` as the canonical fallible durable
    seams.
- Pass 3
  - Re-ran the focused storage and rollup proofs and re-read the managed-root
    allocation helpers.
  - Result: found two allocator issues: `default_root()` returned the base dir
    instead of a fresh child, and the managed creator could silently reuse an
    existing `run-*` directory. Fixed both so each managed startup allocates a
    fresh local root and rejects silent reuse.
- Pass 4
  - Re-ran the five audit scripts plus
    `test_live_guardrails`, `test_hjmt_adaptive_policy_proofs`, and
    `da::tests::test_local_adapter_roundtrip`.
  - Result: clean. The meta-gates, storage guardrail lane, adaptive-proof
    history lane, and local DA roundtrip all stayed green after the allocator
    fixes.
- Pass 5
  - Re-ran the full workspace `cargo test --release`.
  - Result: clean with exit code `0`.

Passes 4 and 5 were consecutive clean manual review runs after the last
in-scope fix.

## 🧾 Closeout

`065-07` closes `WS-07` with one non-panicking env-root-free local startup
seam, redacted wasm transport summaries only, constant-time secret
comparisons, and permanent executable meta-gates for the absorbed attack
surface classes. The active Phase 065 lane moves to `065-08-PLAN.md`.
