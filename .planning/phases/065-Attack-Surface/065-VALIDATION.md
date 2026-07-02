---
phase: 065
slug: 065-attack-surface
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-02
---

# Phase 065 - Validation Strategy

> Per-phase validation contract for feedback sampling and automated proof coverage on the live `065-Attack-Surface` tree.

This file was reconstructed in State B of `gsd-validate-phase`: no prior
`065-VALIDATION.md` existed, all `065-01..13` plans and summaries were
present, `065-TEST-SPEC.md` and `065-TESTS-TASKS.md` already described the
executed test authority chain, and `065-SECURITY.md` closed the corresponding
threat packet. No Phase 065 validation gap remains on the current tree.

## 🎯 Validation Basis

- Context authority:
  `.planning/phases/065-Attack-Surface/065-CONTEXT.md`
- Normative requirement authority:
  `.planning/phases/065-Attack-Surface/065-TODO.md`
- Coverage and rerun authority:
  `.planning/phases/065-Attack-Surface/065-TEST-SPEC.md`
- Executed wave order:
  `.planning/phases/065-Attack-Surface/065-TESTS-TASKS.md`
- Threat cross-reference:
  `.planning/phases/065-Attack-Surface/065-SECURITY.md`

Runtime evidence chain used for validation reconstruction:

- `.planning/phases/065-Attack-Surface/065-01-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-02-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-03-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-04-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-05-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-06-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-07-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-08-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-09-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-10-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-11-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-12-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-13-SUMMARY.md`

The current-tree audit for this validation reconstruction confirmed:

- Phase 065 remains complete in `.planning/STATE.md` and `.planning/ROADMAP.md`
- `065-T01` through `065-T13` still map to existing on-disk tests, scripts,
  or verifier gates
- `065-TEST-SPEC.md` still records `No Phase 065 coverage gap remains on the
  current tree`
- Phase 065 still uses one canonical release-guard audit path only:
  `scripts/audit/audit_release_feature_guards.sh`

## 🔧 Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test --release` plus repository shell audit scripts and verifier gate scripts |
| **Config file** | `./Cargo.toml` plus per-crate `Cargo.toml` files |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | Cache-dependent; broad release suite and managed verifier checks dominate |

## ⏱️ Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the matching targeted `065-T01..T13` release bundle, then `cargo test --release`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** Cache-dependent; bounded by the broad `cargo test --release` gate

## 📋 Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `065-T01` | `01` | `T1` | `WS-01`, `G-01`, `G-02` | `P01-T1..P01-T3` | Accepted validator paths require one theorem bundle and one coherent publication or link story | integration + release | `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`<br>`cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture`<br>`cargo test --release -p z00z_rollup_node --test test_da_local_sim -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T02` | `02` | `T2` | `WS-02`, `G-03`, `G-04` | `P02-T1..P02-T3` | Canonical checkpoint artifacts are born only through sealing and raw lanes stay noncanonical | integration + release | `cargo test --release -p z00z_storage --test test_checkpoint_store -- --nocapture`<br>`cargo test --release -p z00z_storage --test test_checkpoint_finalization -- --nocapture`<br>`cargo test --release -p z00z_storage --test test_checkpoint_link_injective -- --nocapture`<br>`cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T03` | `03` | `T3` | `WS-03` | `P03-T1..P03-T3` | Release-capable wallet and simulator surfaces reject weakened or debug feature sets | source-audit + release | `bash scripts/audit/audit_release_feature_guards.sh`<br>`cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`<br>`cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T04` | `04` | `T4` | `WS-04` | `P04-T1..P04-T3` | Draft or debug simulator lanes cannot emit production-shaped public evidence | integration + release | `cargo test --release -p z00z_simulator --test scenario_1 test_stage6_checkpoint_final_gate::test_draft_publication_rejected -- --exact --nocapture`<br>`cargo test --release -p z00z_simulator --test scenario_1 test_wallet_integration::test_public_lane_secret_free -- --exact --nocapture`<br>`cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T05` | `05` | `T5` | `WS-05`, `G-05` | `P05-T1..P05-T3` | Privileged wallet services require typed capability proof and raw stealth builders stay visibly noncanonical | integration + source-audit | `bash crates/z00z_wallets/scripts/audit_rpc_method_wiring.sh`<br>`cargo test --release -p z00z_wallets --test test_sensitive_rpc_session --test test_wallet_capability_matrix --test test_stealth_output --test test_rpc_route_coverage -- --nocapture`<br>`cargo test --release -p z00z_wallets`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T06` | `06` | `T6` | `WS-06`, `G-06`, `G-07` | `P06-T1..P06-T3` | One canonical wallet mutation owner and one explicit restore retry contract stay coherent under repeated failure | integration + release | `cargo test --release -p z00z_wallets --test test_asset_rpc_mutations -- --nocapture`<br>`cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture`<br>`cargo test --release -p z00z_wallets --test test_chain_client_sim -- --nocapture`<br>`cargo test --release -p z00z_wallets --test test_tx_digest_framing -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T07` | `07` | `T7` | `WS-07` | `P07-T1..P07-T3` | Boundary constructors fail closed, transport logs stay redacted, and repository hygiene gates stay executable | source-audit + integration | `bash scripts/audit/audit_secret_type_hygiene.sh`<br>`bash scripts/audit/audit_secret_eq_hygiene.sh`<br>`bash scripts/audit/audit_crypto_rng_hygiene.sh`<br>`bash scripts/audit/audit_boundary_panic_hygiene.sh`<br>`bash scripts/audit/audit_log_redaction_hygiene.sh`<br>`cargo test --release -p z00z_networks_rpc -- --nocapture`<br>`cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T08` | `08` | `T8` | `WS-08`, `G-08`, `G-09` | `P08-T1..P08-T3` | Public chain and transaction RPCs are either real or explicitly wallet-local or non-production | integration + serialization | `cargo test --release -p z00z_wallets --test test_rpc_truth --test test_rpc_types_serialization --test test_rpc_wiring_spec_a --test test_runtime_validation_result --test test_stub_behavior -- --nocapture`<br>`cargo test --release -p z00z_wallets test_receipt_info_serialization -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T09` | `09` | `T9` | `WS-09` | `P09-T1..P09-T3` | Narrowed historical leftovers stay retired across docs, planning artifacts, and live guardrails | source-audit + guardrail | `bash scripts/audit_phase065_narrowed_wording.sh`<br>`cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T10` | `10` | `T10` | `VR-10` | `P10-T1..P10-T3` | Verification orchestrator dispatches only to the owning gate scripts it reports | tooling + smoke | `./.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh report project --dry-run`<br>`Z00Z_L0_STRICT=1 ./.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`<br>`RUSTUP_TOOLCHAIN=stable ./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh --dry-run`<br>`RUSTUP_TOOLCHAIN=stable ./.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh` | ✅ | ✅ green |
| `065-T11` | `11` | `T11` | `VR-11` | `P11-T1..P11-T3` | Managed verifier tooling reports honest local state and no longer hides it behind bootstrap noise | tooling + release | `./scripts/install-verification-tools.sh --check --profile all --strict`<br>`./scripts/install-verification-tools.sh --self-test --profile all --strict`<br>`./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh`<br>`./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-miri.sh`<br>`./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh`<br>`./.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-hax.sh`<br>`./.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-tamarin.sh`<br>`./.github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-short.sh`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T12` | `12` | `T12` | `VR-12` | `P12-T1..P12-T3` | Aggregator release-fast tests stay release-safe and lineage-deterministic without reactivating wallet fast-test features | integration + feature-audit | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_consensus`<br>`cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_route_rollout`<br>`cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage`<br>`bash scripts/audit/audit_release_feature_guards.sh`<br>`cargo tree -e features -p z00z_aggregators --features test-params-fast -i z00z_wallets`<br>`cargo test --release` | ✅ | ✅ green |
| `065-T13` | `13` | `T13` | `VR-13` | `P13-T1..P13-T3` | Payment request and receiver-card paths stay domain-bound and claim scope stays persisted-chain-bound | crypto + integration | `cargo test --release -p z00z_crypto --test test_hash_policy`<br>`cargo test --release -p z00z_wallets test_claim_scope_chain -- --nocapture`<br>`cargo test --release -p z00z_wallets --test test_payment_request`<br>`cargo test --release -p z00z_wallets --test test_asset_replay_protection`<br>`cargo test --release -p z00z_wallets --test test_e2e_req_flow`<br>`cargo test --release -p z00z_wallets --test test_adversarial`<br>`cargo test --release -p z00z_wallets --test test_import_error_taxonomy`<br>`cargo test --release` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## ✅ Wave 0 Requirements

Existing infrastructure covers all phase requirements.

Phase 065 already owns the required Rust workspace, per-crate integration test
layout, audit scripts, managed verifier tooling scripts, and phase-local test
authority documents. No Wave 0 test stubs or framework installation work are
needed.

## ✅ Manual-Only Verifications

All phase behaviors have automated verification.

The `/GSD-Review-Tasks-Execution` prompt-path failures recorded in several
summary files were execution-review limitations, not missing runtime behavior
tests. They do not create a Phase 065 Nyquist coverage gap because every
product or tooling behavior closed by this phase is still covered by an
executable release-mode command or audit script above.

## ✅ Validation Sign-Off

- [x] All tasks have automated verification on the live tree
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 is already satisfied by existing infrastructure
- [x] No watch-mode flags
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-02
