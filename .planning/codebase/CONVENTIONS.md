# Coding Conventions

**Analysis Date:** 2026-07-07

## Naming Patterns

**Files:**
- Rust implementation files use `snake_case`, for example `crates/z00z_storage/src/settlement/proof_batch.rs`, `crates/z00z_wallets/src/rpc/tx_rpc_server.rs`, and `crates/z00z_runtime/aggregators/src/batch_planner.rs`.
- Integration tests strongly prefer `test_*.rs`, for example `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs` and `crates/z00z_storage/tests/test_live_guardrails.rs`.
- Simulator stages use numbered directories and module files such as `crates/z00z_simulator/src/scenario_1/stage_6/mod.rs`.

**Functions:**
- Function and method names are `snake_case` and usually verb-led, for example `verify_settlement_theorem`, `build_output_bundle`, `register_all_wallet_rpc_methods`, and `maybe_run_hjmt_process_devnet`.
- Boolean helpers prefer `is_*` and `has_*`, for example methods throughout `crates/z00z_storage/src/checkpoint/` and `crates/z00z_wallets/src/receiver/`.

**Variables:**
- Local names stay domain-specific, for example `wallet_id`, `prep_snapshot_id`, `claim_store`, `route_generation`, and `support_ref`.
- Test support code also uses semantic nouns instead of generic placeholders, for example `managed_test_output_root` in `crates/z00z_wallets/tests/test_inc/test_mod.rs`.

**Types:**
- Public types use `PascalCase`, for example `SettlementStore`, `WalletService`, `ShardPlacementTable`, `StatusSnapshot`, and `ScenarioResult`.
- Trait and surface names frequently end in `Provider`, `Service`, `Store`, `Rpc`, or `Verifier`, as seen in `crates/z00z_utils/src/lib.rs`, `crates/z00z_wallets/src/rpc/`, and `crates/z00z_runtime/*/src/lib.rs`.

## Code Style

**Formatting:**
- `cargo fmt` is the expected formatter.
- Shared Cargo aliases and defaults live in `.cargo/config.toml`.

**Linting:**
- Workspace lint settings live in `Cargo.toml`.
- `.clippy.toml` intentionally stays minimal because vendored Tari code is protected.
- `deny.toml` and `.cargo/audit.toml` govern dependency and advisory policy.
- Most crates use `#![forbid(unsafe_code)]`; `crates/z00z_utils/src/lib.rs` uses `#![warn(unsafe_code)]` to isolate OS-hardening calls.

## Import Organization

**Order:**
1. External crate imports
2. `crate::...` imports
3. `super::...` imports inside nested modules and tests

**Path Aliases:**
- No custom path alias system is used.
- Re-exports are centralized in crate facades such as `crates/z00z_wallets/src/lib.rs`, `crates/z00z_storage/src/lib.rs`, and `crates/z00z_runtime/*/src/lib.rs`.

## Error Handling

**Patterns:**
- Typed error enums built with `thiserror` are the default in workspace crates such as `crates/z00z_storage/src/error.rs`.
- Public surfaces prefer crate-local `Result` aliases and explicit reject codes over generic `anyhow::Result`.
- `expect` and `unwrap` appear heavily in test code and examples, while production-facing seams are more defensive.

## Logging

**Framework:** `tracing` plus project-owned logger abstractions in `crates/z00z_utils/src/logger/`.

**Patterns:**
- Cross-cutting logging contracts live in `z00z_utils`.
- Wallet-specific RPC logging is explicit and policy-driven in files such as `crates/z00z_wallets/src/rpc/logging.rs`, `crates/z00z_wallets/src/rpc/logging_middleware.rs`, and `crates/z00z_wallets/src/rpc/logging_policy.rs`.
- Runtime watcher crates expose alert/evidence types rather than generic print-based diagnostics.

## Comments

**When to Comment:**
- Crate roots and boundary-defining modules usually carry rustdoc explaining ownership and scope.
- Complex guardrails and test contracts often explain why a check exists, especially in `crates/z00z_storage/tests/test_live_guardrails.rs` and `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`.
- Inline comments are used sparingly and mostly around security, build, or boundary rules.

**Rustdoc:**
- README-backed crate docs are common, for example `#![doc = include_str!("../README.md")]` in `crates/z00z_wallets/src/lib.rs`, `crates/z00z_storage/src/lib.rs`, `crates/z00z_rollup_node/src/lib.rs`, and `crates/z00z_utils/src/lib.rs`.
- Public facades also document re-export intent and boundary semantics in crate root comments.

## Function Design

**Size:**
- Facades usually stay thin and re-export-oriented.
- Internal executable/test support files can become very large, especially under `crates/z00z_wallets/src/services/`, `crates/z00z_simulator/src/scenario_1/`, and `crates/z00z_storage/src/settlement/`; treat those as exceptions to the preferred split discipline, not as the style target.

**Parameters:**
- Boundary APIs favor typed wrappers and domain structs, for example `SettlementPath`, `TerminalId`, `PersistWalletId`, `RpcLoggingConfig`, and `WalletService`.
- Builder- or context-style argument grouping is common in storage, wallet, and rollup modules.

**Return Values:**
- Domain services return typed results and explicit enums.
- Tests assert exact enum variants or structured payload fields rather than only checking `is_err()`.

## Module Design

**Exports:**
- Root `lib.rs` files are canonical public entrypoints.
- Internal detail modules are split under subdirectories such as `settlement/`, `rpc/`, `services/`, `receiver/`, and `stage_*/`.

**Barrel Files:**
- `mod.rs` remains common for domain partitions, for example `crates/z00z_storage/src/settlement/mod.rs`, `crates/z00z_wallets/src/services/mod.rs`, and `crates/z00z_simulator/src/scenario_1/mod.rs`.
- Large scenario test suites also use crate-style root files that `mod` many focused test modules, for example `crates/z00z_simulator/tests/scenario_1/main.rs`.

---

*Convention analysis: 2026-07-07*
