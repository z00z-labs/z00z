# Z00Z HJMT Threat Model

Version: 2026-06-16

## 🎯 Purpose

This phase-local document closes Appendix C row `C-14` from
[Z00Z-HJMT-Upgrade.md](Z00Z-HJMT-Upgrade.md).

It scopes the live HJMT misuse classes that Phase 058 must bound explicitly:

- parser misuse
- replay misuse
- authority misuse
- transition misuse

The threat model is repository-owned. It does not create a second protocol
authority beyond the existing storage, routing, publication, and failover
contracts.

## ⚙️ Threat Classes

| Threat class | Adversarial move | Required invariant | Live owner home |
| --- | --- | --- | --- |
| parser misuse | malformed batch-proof bytes, mixed proof-family tags, mixed opening kinds, bad witness references, stale codec bytes | every proof parser and verifier path rejects before semantic acceptance | `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs`; `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs` |
| replay misuse | stale route generations, stale root generations, stale shard epochs, stale journal lineage, replayed failover state | old route, publication, and failover state cannot be reinterpreted as current authority | `crates/z00z_storage/tests/test_hjmt_root_generation.rs`; `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`; `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs` |
| authority misuse | treating path indexes, runtime placement, imported bytes, or startup config as protocol truth without validation | storage and committed routing stay authoritative; startup and import paths reject drift before live work | `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`; `crates/z00z_storage/tests/test_hjmt_import_export.rs`; `crates/z00z_storage/tests/test_hjmt_storage_boundary.rs`; `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs` |
| transition misuse | stale policy transitions, wrong occupancy evidence, wrong proof-before-ownership ordering, wrong route/publication lineage during replay | adaptive and historical transition paths reject stale or reinterpreted state | `crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs`; `crates/z00z_storage/tests/test_occupancy_privacy.rs`; `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs` |

## 🔒 Mandatory Security Rules

1. Parsers are bounded and fail-closed.
2. Historical proofs must be interpreted against imported historical metadata,
   not current config.
3. Runtime placement is operational metadata only; it is never route or proof
   authority.
4. Path indexes and caches remain rebuildable helpers, not proof truth.
5. Same-lineage failover is legal only when shard, generation, root, and
   journal-lineage checks all succeed.
6. Policy-transition and occupancy evidence may not be replayed under a newer
   route or root generation.

## ✅ Verification Commands

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_backend_conformance -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_import_export -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_privacy -- --nocapture
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture
cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture
cargo test -p z00z_simulator --release --test test_scenario_settlement -- --nocapture
bash scripts/audit/audit_release_feature_guards.sh
```
