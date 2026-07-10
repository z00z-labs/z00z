# Technology Stack

**Analysis Date:** 2026-07-07

## Languages

**Primary:**
- Rust 2021 workspace baseline for `Cargo.toml` members such as `crates/z00z_core/`, `crates/z00z_storage/`, `crates/z00z_wallets/`, `crates/z00z_runtime/*`, `crates/z00z_rollup_node/`, and `crates/z00z_simulator/`.

**Secondary:**
- Rust 2018 for the vendored-bridge crypto crate in `crates/z00z_crypto/Cargo.toml`.
- Shell for repo automation and guardrails in `scripts/` and `.github/workflows/*.yml`.
- Python for verification and audit helpers such as `scripts/cargo_build.py` and `scripts/penetration/*.py`.
- YAML/JSON/Markdown for configuration and planning in `config/`, `crates/z00z_core/configs/`, `crates/z00z_wallets/src/config/`, `crates/z00z_simulator/src/scenario_1/`, and `.planning/`.
- WebAssembly bindings for browser-facing surfaces in `crates/z00z_wallets/Cargo.toml` and `crates/z00z_networks/rpc/Cargo.toml`.

## Runtime

**Environment:**
- Native Rust binaries for local execution, validation, and simulation.
- Tokio async runtime for RPC, wallet, and simulator flows in `crates/z00z_wallets/Cargo.toml`, `crates/z00z_networks/rpc/Cargo.toml`, and `crates/z00z_simulator/Cargo.toml`.
- Rayon for CPU-bound batch and proof work in `crates/z00z_core/Cargo.toml`, `crates/z00z_storage/Cargo.toml`, and `crates/z00z_wallets/Cargo.toml`.
- `wasm32-unknown-unknown` test/runtime support via `.cargo/config.toml`, `wasm-bindgen`, and `wasm-bindgen-test`.

**Package Manager:**
- Cargo workspace managed from `Cargo.toml`.
- Lockfile: `Cargo.lock` present.

## Frameworks

**Core:**
- `jsonrpsee` 0.26 for JSON-RPC transport and WASM WebSocket client support in `crates/z00z_networks/rpc/Cargo.toml` and `crates/z00z_wallets/Cargo.toml`.
- `redb` 2/3.x for local wallet and storage persistence in `crates/z00z_storage/Cargo.toml` and `crates/z00z_wallets/Cargo.toml`.
- `jmt` 0.12.0 for settlement-tree storage and proof machinery in `crates/z00z_storage/Cargo.toml`.
- `tari_crypto` and `tari_bulletproofs_plus` through the read-only backend in `crates/z00z_crypto/Cargo.toml` and `crates/z00z_crypto/tari/`.

**Testing:**
- Rust built-in test harness across all workspace crates.
- `criterion` 0.5 benches in `crates/z00z_core/`, `crates/z00z_crypto/`, `crates/z00z_storage/`, and `crates/z00z_wallets/`.
- `proptest` 1.5 in `crates/z00z_core/Cargo.toml`, `crates/z00z_storage/Cargo.toml`, and `crates/z00z_wallets/Cargo.toml`.
- `wasm-bindgen-test` for browser-target tests in `crates/z00z_wallets/Cargo.toml` and `crates/z00z_networks/rpc/Cargo.toml`.

**Build/Dev:**
- `cargo fmt`, `cargo clippy`, and release-mode guardrails described in `.cargo/config.toml`, `.github/workflows/`, and `.github/copilot-instructions.md`.
- `cargo-deny` policy in `deny.toml` and `.cargo/audit.toml`.
- CodeGraph index present in `.codegraph/` for code navigation.

## Key Dependencies

**Critical:**
- `z00z_utils` as the shared abstraction layer in `crates/z00z_utils/src/lib.rs`.
- `z00z_crypto` as the crypto facade over Tari in `crates/z00z_crypto/src/lib.rs`.
- `z00z_storage` as the settlement/checkpoint/snapshot authority in `crates/z00z_storage/src/lib.rs`.
- `z00z_wallets` as the wallet, RPC, persistence, and native/WASM application surface in `crates/z00z_wallets/src/lib.rs`.
- `z00z_aggregators`, `z00z_validators`, and `z00z_watchers` as runtime policy crates in `crates/z00z_runtime/*/src/lib.rs`.

**Infrastructure:**
- `argon2`, `hkdf`, `chacha20poly1305`, `bip39`, and `bip32` for wallet cryptography in `crates/z00z_wallets/Cargo.toml` and `crates/z00z_crypto/Cargo.toml`.
- `tracing` and `tracing-subscriber` for structured logging in `crates/z00z_utils/`, `crates/z00z_core/`, and `crates/z00z_wallets/`.
- `rust_xlsxwriter` for simulator report output in `crates/z00z_simulator/Cargo.toml`.
- `zip` and `jsonschema` for snapshot/export/config validation in `crates/z00z_core/Cargo.toml` and `crates/z00z_wallets/Cargo.toml`.
- `lru` for wallet cache layers in `crates/z00z_wallets/Cargo.toml`.

## Configuration

**Environment:**
- Workspace-wide Cargo behavior is centralized in `.cargo/config.toml`.
- Root blockchain config lives in `config/z00z_blockchain_config.yaml`.
- Core live config set lives in `crates/z00z_core/configs/`.
- Wallet config lives in `crates/z00z_wallets/src/config/wallet_config.yaml`.
- Simulator scenario config lives in `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` and `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`.
- Rollup/HJMT process contracts are defined in `crates/z00z_rollup_node/src/config.rs` and `crates/z00z_rollup_node/src/process_devnet.rs`.

**Build:**
- Shared workspace members and lint policy live in `Cargo.toml`.
- Advisory and license policy live in `deny.toml` and `.cargo/audit.toml`.
- CI guard workflows live in `.github/workflows/boundary-guards.yml`, `.github/workflows/release-safety-guards.yml`, and `.github/workflows/security-hygiene-guards.yml`.

## Platform Requirements

**Development:**
- Stable Rust toolchain with Cargo.
- Native test/build environment for release-mode guardrails used in `.github/workflows/*.yml`.
- `wasm-bindgen-test-runner` configured in `.cargo/config.toml` for WASM test targets.

**Production:**
- Native runtime targets for wallet, rollup, and simulator binaries.
- Optional desktop GUI path through `eframe` under the `egui` feature in `crates/z00z_wallets/Cargo.toml`.
- Local-first persistence and process orchestration rather than hosted cloud runtime contracts.

---

*Stack analysis: 2026-07-07*
