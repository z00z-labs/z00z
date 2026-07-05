<!-- generated-by: gsd-doc-writer -->
# Configuration

## 🎯 Overview

Z00Z configuration is split across workspace-level Cargo settings, checked-in runtime fixture manifests under `config/`, crate-local YAML resources, and optional verification environment scripts. The active runtime example in the repository is the HJMT fixture tree under `config/hjmt_runtime/`.

## ⚙️ Workspace And Tooling Configuration

| Path | Purpose |
|---|---|
| `Cargo.toml` | Workspace membership, shared package metadata, lint policy, and shared dependencies |
| `.cargo/config.toml` | Shared target directory, test aliases, and clippy helper aliases |
| `deny.toml` | Cargo-deny advisories, license allowlist, and source policy |
| `scripts/verify-env.sh` | Exports the extended verification toolchain environment |
| `scripts/verification-tools/versions.env` | Version pins for the local verification bundle |

The active workspace members are:

```text
z00z_core
z00z_crypto
z00z_networks/onionnet
z00z_networks/rpc
z00z_rollup_node
z00z_runtime/aggregators
z00z_runtime/validators
z00z_runtime/watchers
z00z_simulator
z00z_storage
z00z_telemetry
z00z_utils
z00z_wallets
```

`crates/z00z_extensions/` exists on disk but is not part of `workspace.members` or `default-members`.

## 🧭 Runtime Fixture Configuration

### 📌 HJMT fixture roots

| Path | Role |
|---|---|
| `config/hjmt_runtime/sim_5a7s/manifest.json` | Primary checked-in topology manifest |
| `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml` | Planner route policy, cadence, and evidence/runtime output paths |
| `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml` | Storage backend, sync, cache, and runtime data paths |
| `config/hjmt_runtime/sim_5a7s/aggregators/agg-*/aggregator-config.yaml` | Per-aggregator shard ownership, network, lifecycle, and evidence paths |
| `config/hjmt_runtime/sim_5a7s/shard_route_tables/route-table-v1.canon.hex` | Route-table byte contract referenced by planner and aggregators |
| `config/hjmt_runtime/sim_7a7s/` | Second fixture tree with the same layout pattern |

### 📌 Verified values from `sim_5a7s`

`config/hjmt_runtime/sim_5a7s/manifest.json` currently declares:

- profile: `SIM-5A7S`
- aggregator ids: `0..4`
- shard ids: `0..6`
- runtime fixture home: `config/hjmt_runtime`
- planner config path: `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`
- storage config path: `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`
- canonical topology test command:

```bash
cargo test -p z00z_rollup_node --release --features test-params-fast \
  --test test_hjmt_topology test_grid57_matches_contract -- --nocapture
```

### 📌 Example aggregator launch

The checked-in aggregator config for `agg-0` includes this lifecycle command:

```bash
cargo run --release -p z00z_rollup_node -- \
  --mode aggregator \
  --aggregator-config config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml \
  --planner-config config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml \
  --storage-config config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml
```

Planner and storage configs in the same fixture tree resolve generated runtime state under `var/hjmt_runtime/sim_5a7s/...`.

## 🔑 Wallet And Crate-Local Configuration

The wallet crate ships local configuration resources under `crates/z00z_wallets/src/config/`:

| Path | Purpose |
|---|---|
| `wallet_config.yaml` | Wallet defaults/config surface owned by the wallet crate |
| `wallet_config_defaults.rs` | Wallet-side compiled defaults |
| `redb-schema.yaml` | RedB schema description for wallet persistence |
| `common-passwords.txt` | Password denylist source |
| `password_denylist.bloom` | Bloom filter derived from the denylist set |

Other crates keep configuration closer to code rather than in top-level runtime YAML files. For example, `z00z_rollup_node` exposes `NodeConfig` and `AggRunArgs` in `crates/z00z_rollup_node/src/config.rs`, while `z00z_core` exposes typed bootstrap surfaces such as `GenesisConfig`.

## 🛠️ Verification Environment

Use the local verification bundle when you need supply-chain or formal-analysis tools in a consistent location:

```bash
./scripts/verification-tools/install-verification-tools.sh
source ./scripts/verify-env.sh
```

`scripts/verify-env.sh` prepends tool directories from `tools/formal_verification/` and exports related variables such as `CARGO_HOME`, `RUSTUP_HOME`, `KANI_HOME`, and `Z00Z_CARGO_PROFILE_ARGS`.

## ⚠️ Current Caveats

- `config/z00z_blockchain_config.yaml` is currently empty.
- `onionnet` does not yet define a live runtime configuration tree; its crate surface is a reserved boundary with placeholder types.
- `www/` is not a stable checked-in configuration root; it is generated on demand by the wallet WASM build flow.

## 📚 Related Docs

- [Getting started](../guides/getting-started.md)
- [Development guide](../guides/development.md)
- [Testing guide](../testing/overview.md)
- [Architecture overview](../architecture/overview.md)
