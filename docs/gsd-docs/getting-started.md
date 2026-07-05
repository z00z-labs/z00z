<!-- generated-by: gsd-doc-writer -->
# Getting Started

## 🎯 Prerequisites

The workspace root declares `rust-version = "1.90.0"`, so start with a stable Rust toolchain compatible with Rust 1.90 plus the standard Rust components:

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
```

Optional tools depend on the workflow you want:

- `python3` for `./scripts/serve_wasm.sh`
- `wasm32-unknown-unknown` target and `wasm-pack` for wallet browser builds
- `binaryen` if you want `wasm-opt` optimization inside `./scripts/build_wasm.sh`

## ⚙️ Installation Steps

1. Clone the repository:

   ```bash
   git clone https://github.com/vasja34/z00z.git
   ```

2. Enter the workspace:

   ```bash
   cd z00z
   ```

3. Install the required Rust components:

   ```bash
   rustup component add rustfmt clippy
   ```

4. Confirm the workspace resolves and compiles:

   ```bash
   cargo check --workspace
   ```

## 🚀 First Run

The quickest live entrypoint is the rollup-node CLI help surface, because it proves the crate builds and prints the current process contract without needing fixture paths:

```bash
cargo run -p z00z_rollup_node -- --help
```

If you want a checked-in runtime contract after that, run the topology test wired to `config/hjmt_runtime/sim_5a7s/manifest.json`:

```bash
cargo test -p z00z_rollup_node --release --features test-params-fast \
  --test test_hjmt_topology test_grid57_matches_contract -- --nocapture
```

## 🔧 Optional Wallet WASM Setup

For browser-facing wallet work:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
./scripts/build_wasm.sh --dev
./scripts/serve_wasm.sh 8000
```

The build script emits `www/pkg/` only after a successful run, so `www/` should be treated as a generated artifact root rather than a guaranteed checked-in directory.

## ⚠️ Common Setup Issues

### ❌ Missing Rust components

If `cargo fmt --check` or `cargo clippy` fails because the component is missing, install them explicitly:

```bash
rustup component add rustfmt clippy
```

### ❌ Missing WASM tooling

If `./scripts/build_wasm.sh` fails, the usual causes are a missing `wasm32-unknown-unknown` target, missing `wasm-pack`, or absent `wasm-opt`. Install them before retrying.

### ❌ Rollup node launched without required runtime flags

`z00z_rollup_node` only accepts aggregator mode in the current live process contract. Any real run must include all three config flags:

```text
--aggregator-config <path> --planner-config <path> --storage-config <path>
```

### ❌ No Python 3 for the local WASM server

`./scripts/serve_wasm.sh` falls back to `python3 -m http.server` or `python -m SimpleHTTPServer`. Install Python 3 if the script cannot find either executable.

## 📚 Next Steps

- [Development guide](./development.md)
- [Testing guide](../testing/overview.md)
- [Configuration guide](../configuration/overview.md)
- [Architecture overview](../architecture/overview.md)
