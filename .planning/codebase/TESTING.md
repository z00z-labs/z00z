# Testing Patterns

**Analysis Date:** 2026-07-07

## Test Framework

**Runner:**
- Rust built-in test harness via `cargo test`.
- Shared test aliases and defaults live in `.cargo/config.toml`.
- Release guardrails run in `.github/workflows/boundary-guards.yml`, `.github/workflows/release-safety-guards.yml`, and `.github/workflows/security-hygiene-guards.yml`.

**Assertion Library:**
- Standard Rust assertions such as `assert!`, `assert_eq!`, `assert_ne!`, `matches!`, `expect`, and `unwrap_err`.
- Property assertions use Proptest macros in crates that enable `proptest`.

**Run Commands:**
```bash
cargo test --workspace
cargo rt --workspace
cargo test --release -p z00z_wallets --test test_rpc_logging_acceptance -- --nocapture
cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture
```

## Test File Organization

**Location:**
- Integration-heavy crates use dedicated `tests/` trees, especially `crates/z00z_wallets/tests/`, `crates/z00z_storage/tests/`, and `crates/z00z_simulator/tests/scenario_1/`.
- Unit-style checks also live alongside implementation files inside `src/`, for example many `test_*` modules under `crates/z00z_wallets/src/` and `crates/z00z_storage/src/`.

**Naming:**
- The repository strongly prefers `test_*.rs`: 424 of 439 Rust files under `crates/*/tests/` match that pattern.
- Scenario suites also use one root test crate plus many `mod test_*;` declarations, for example `crates/z00z_simulator/tests/scenario_1/main.rs`.

**Structure:**
```text
crates/z00z_wallets/tests/
├── fixtures/
├── test_inc/
├── test_rpc_logging_acceptance.rs
├── test_wallet_persist_nostd_fs.rs
└── test_tx_*.rs

crates/z00z_simulator/tests/scenario_1/
├── main.rs
├── test_stage4_*.rs
├── test_stage6_*.rs
└── test_scenario1_*.rs
```

## Test Structure

**Suite Organization:**
```rust
// crates/z00z_simulator/tests/scenario_1/main.rs
mod test_checkpoint_acceptance;
mod test_claim_acceptance;
mod test_hjmt_e2e;
mod test_wallet_integration;
```

```rust
// crates/z00z_wallets/tests/test_inc/test_mod.rs
pub fn managed_test_output_root(case_name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs/tests")
        .join(exe_scope());
    // ...
}
```

**Patterns:**
- Large suites are split into many focused files and wired through a single root module.
- Test output paths are deterministic and hashed from workspace inputs in `crates/z00z_wallets/tests/test_inc/test_mod.rs`.
- Guardrail tests often `include_str!` source, workflow, and planning files to enforce architectural rules, as shown in `crates/z00z_storage/tests/test_live_guardrails.rs`.

## Mocking

**Framework:** no dedicated mocking framework detected.

**Patterns:**
```rust
// crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs
let dispatcher = Arc::new(RpcDispatcher::new());
let base = LocalRpcTransport::new(dispatcher);
let transport = LoggedRpcTransport::new(base, config, logger, time, rng);
```

```rust
// crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs
let dir = tempfile::tempdir().unwrap();
let service = Arc::new(WalletService::with_output_dir(dir.path().join("wallets")));
```

**What to Mock:**
- Prefer fakeable transport and filesystem seams through local transports, temp directories, and deterministic providers from `z00z_utils`.
- Prefer test-owned service construction over patching internals.

**What NOT to Mock:**
- Crypto, proofs, settlement roots, and wallet/store serialization are often exercised through real code paths.
- Storage and simulator tests frequently rebuild live state instead of replacing it with dummy objects.

## Fixtures and Factories

**Test Data:**
```rust
// crates/z00z_storage/tests/test_store_api.rs
fn asset_item(mark: u8) -> StoreItem {
    let core = AssetLeaf::dummy_for_scan(u32::from(mark));
    let leaf = TerminalLeaf::from(core.clone());
    let path = SettlementPath::new(
        DefinitionId::new(bytes(mark)),
        SerialId::new(core.serial_id),
        TerminalId::new(core.asset_id),
    );
    StoreItem::new(path, leaf).expect("asset item")
}
```

**Location:**
- Wallet fixtures: `crates/z00z_wallets/tests/fixtures/`
- Simulator fixtures: `crates/z00z_simulator/tests/scenario_1/fixtures/`
- Storage fixtures: `crates/z00z_storage/tests/fixtures/`
- Generated Kani artifacts: `crates/z00z_wallets/tests/generated_kani_*.rs` and `crates/z00z_storage/tests/generated_kani_checkpoint_artifact.rs`

## Coverage

**Requirements:** no explicit percentage gate detected in the active workspace.

**View Coverage:**
```bash
Not detected as a standard workspace command.
```

## Test Types

**Unit Tests:**
- Common inside source modules for storage, wallet, and runtime internals.
- Often target typed invariants, serializers, path binding, and helper behavior.

**Integration Tests:**
- Dominant pattern for `z00z_wallets`, `z00z_storage`, and `z00z_simulator`.
- Integration tests span multiple crates and validate real end-to-end data flow rather than isolated mocks.

**E2E Tests:**
- E2E-style checks are still Rust tests, not a separate external framework.
- Examples include `crates/z00z_wallets/tests/test_e2e_req_flow.rs`, `crates/z00z_wallets/tests/test_e2e_runtime_parity.rs`, and `crates/z00z_simulator/tests/scenario_1/test_scenario1_unified_gate.rs`.

## Common Patterns

**Async Testing:**
```rust
// crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs
#[tokio::test]
async fn test_rpc_logging_prevents_double() {
    // async RPC roundtrip with LocalRpcTransport
}
```

**Error Testing:**
```rust
// crates/z00z_storage/tests/test_store_api.rs
fn assert_fee_required(err: SettlementStoreError) {
    match err {
        SettlementStoreError::Fee(FeeErr::SupportRequired) => {}
        other => panic!("expected SupportRequired, got {other:?}"),
    }
}
```

**Property Testing:**
- Proptest is enabled in `crates/z00z_core/Cargo.toml`, `crates/z00z_storage/Cargo.toml`, and `crates/z00z_wallets/Cargo.toml`.
- Storage also keeps a Proptest regression corpus in `crates/z00z_storage/tests/test_property_corpus.proptest-regressions`.

---

*Testing analysis: 2026-07-07*
