<!-- generated-by: gsd-doc-writer -->
# Development

## 🎯 Local Setup

1. Clone the repository and enter the workspace root:

   ```bash
   git clone https://github.com/vasja34/z00z.git
   cd z00z
   ```

2. Install the standard Rust components:

   ```bash
   rustup component add rustfmt clippy
   ```

3. Verify the workspace resolves:

   ```bash
   cargo check --workspace
   ```

4. If you need the wallet browser flow, add the WASM target and tools:

   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install wasm-pack
   ```

5. If you need the extended formal-verification toolchain, inspect or install the local bundle:

   ```bash
   ./scripts/verification-tools/install-verification-tools.sh
   source ./scripts/verify-env.sh
   ```

## ⚙️ Build And Verification Commands

| Command | Description |
|---|---|
| `cargo check --workspace` | Fast workspace compile check |
| `cargo build --workspace --release` | Release build across active workspace members |
| `cargo fmt --check` | Formatting gate |
| `cargo clippy --workspace --release --all-targets --all-features -- -D warnings` | Canonical lint gate |
| `cargo test --workspace --release --lib --bins --tests --examples --all-features` | Main release-style test suite |
| `cargo test --workspace --release --all-features --doc` | Rust doc-test gate |
| `cargo bench --workspace --all-features --no-run` | Compile all benchmark targets without executing them |
| `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` | Canonical end-to-end workspace quality gate |
| `cargo deny check advisories bans licenses sources` | Supply-chain and license policy check using `deny.toml` |
| `./scripts/build_wasm.sh --dev` | Wallet WASM dev build; emits `www/pkg/` on success |
| `./scripts/serve_wasm.sh 8000` | Serve generated wallet WASM assets locally |
| `./scripts/run-cargo-fuzz.sh <fuzz-dir> <subcommand>` | Wrapper for crate-local `cargo fuzz` workflows |

The workspace also defines cargo aliases in `.cargo/config.toml`, including:

- `cargo t` -> `cargo test --features test-fast`
- `cargo rt` -> `cargo test --release --features test-fast`
- `cargo clippy-clean`
- `cargo clippy-clean-fast`
- `cargo clippy-clean-all`

## 🔑 Code Style And Local Rules

- The repository uses Rust-first style enforcement through `rustfmt` and `clippy`; `.github/copilot-instructions.md` requires zero clippy warnings and passing tests before closing work.
- `deny.toml` is checked by `cargo deny` and captures advisory, license, and source policy.
- `crates/z00z_crypto/tari/` is treated as read-only vendor code and must not be modified.
- Docs and code must be written in English. Chat responses to the user stay in Russian.
- Full-file rewrites of existing Markdown, JSON, YAML, CSV, or text files require a sibling `.bak` file first.

## 🧱 Branch Conventions

No repository-local `CONTRIBUTING.md` or `.github/PULL_REQUEST_TEMPLATE.md` currently documents a branch naming convention. Use a branch name that is explicit about scope, and do not assume a hidden naming policy from the repository itself.

## 🔄 Practical PR Process

Because there is no checked-in PR template, the practical gate is the combination of repo instructions and CI guard workflows:

1. Keep `cargo fmt`, `cargo clippy`, and the relevant `cargo test` commands green.
2. Run `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` before claiming the workspace is clean.
3. Update documentation when public behavior, config surfaces, or crate boundaries change.
4. Respect guarded areas such as `crates/z00z_crypto/tari/` and avoid destructive git/file operations.
5. Check the focused GitHub Actions workflows in `.github/workflows/` if your change touches boundary, release-safety, or security-hygiene behavior.

## 📚 Related Docs

- [Getting started](./getting-started.md)
- [Testing guide](../testing/overview.md)
- [Configuration guide](../configuration/overview.md)
- [Architecture overview](../architecture/overview.md)
