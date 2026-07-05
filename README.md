<!-- generated-by: gsd-doc-writer -->
# Z00Z

Privacy-focused Rust workspace for confidential assets, wallets, settlement storage, rollup runtime, and reproducible simulation.

## 🎯 Overview

Z00Z is a multi-crate Rust workspace built around confidential transaction flows and typed settlement objects. The active workspace combines protocol and utility foundations (`z00z_core`, `z00z_crypto`, `z00z_utils`), persistence and client surfaces (`z00z_storage`, `z00z_wallets`), runtime execution (`z00z_aggregators`, `z00z_validators`, `z00z_watchers`, `z00z_rollup_node`), transport helpers (`z00z_networks_rpc`), and an end-to-end simulator (`z00z_simulator`).

The repository also contains `crates/z00z_extensions/`, but that directory is not listed in `workspace.members` and is not part of the active build graph. The `onionnet` crate is present in the workspace as a reserved boundary with placeholder types rather than a live overlay implementation.

## ⚙️ Installation

The workspace root declares `rust-version = "1.90.0"`. Start with the stable Rust toolchain plus the standard Rust quality tools:

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
git clone https://github.com/vasja34/z00z.git
cd z00z
cargo check --workspace
```

If you need the wallet WASM flow, add the extra toolchain pieces:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
# Optional WASM optimizer used by scripts/build_wasm.sh
sudo apt install binaryen
```

## 🚀 Quick Start

1. Clone the repository and enter the workspace root.
2. Confirm the workspace compiles:

   ```bash
   cargo check --workspace
   ```

3. Inspect the rollup node entrypoint:

   ```bash
   cargo run -p z00z_rollup_node -- --help
   ```

4. Run one live topology contract backed by the checked-in HJMT runtime fixtures:

   ```bash
   cargo test -p z00z_rollup_node --release --features test-params-fast \
     --test test_hjmt_topology test_grid57_matches_contract -- --nocapture
   ```

## 📦 Workspace Map

| Area | Crates | Role |
|---|---|---|
| Foundations | `z00z_utils`, `z00z_crypto`, `z00z_core` | Shared primitives, cryptography, asset/genesis/object semantics |
| Persistence | `z00z_storage` | Settlement roots, proofs, snapshots, backend seams |
| Runtime | `z00z_aggregators`, `z00z_validators`, `z00z_watchers`, `z00z_rollup_node` | Batch planning, verdicts, observation, aggregator-mode process entrypoint |
| Client | `z00z_wallets` | HD wallet, typed object inventory, RPC and backup flows |
| Transport | `z00z_networks_rpc`, `onionnet` | RPC dispatch today; reserved OnionNet boundary for future overlay transport |
| Integration | `z00z_simulator`, `z00z_telemetry` | End-to-end harness and shared observability facade |

## 📌 Common Workflows

### ✅ Run the canonical workspace verification gate

```bash
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh
```

### ✅ Launch the rollup-node CLI help surface

```bash
cargo run -p z00z_rollup_node -- --help
```

The current live process contract accepts only aggregator mode:

```text
z00z_rollup_node --mode aggregator --aggregator-config <path> --planner-config <path> --storage-config <path>
```

### ✅ Build the wallet WASM artifacts

```bash
./scripts/build_wasm.sh --dev
./scripts/serve_wasm.sh 8000
```

`./scripts/build_wasm.sh` writes generated browser artifacts into `www/pkg/` when the build succeeds.

## 📚 Documentation

- [Architecture overview](./docs/architecture/overview.md)
- [Getting started](./docs/guides/getting-started.md)
- [Development guide](./docs/guides/development.md)
- [Testing guide](./docs/testing/overview.md)
- [Configuration guide](./docs/configuration/overview.md)
- [Deep Wiki Guide](./deep-wiki-readme.md)
- [LLM Wiki Guide](./llm-wiki-readme.md)

## 📄 License

Z00Z is licensed under MIT, as declared in the workspace manifest.
