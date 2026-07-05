<!-- generated-by: gsd-doc-writer -->
# Testing

## 🎯 Test Framework And Setup

Z00Z is a Rust workspace, so the primary test harness is the built-in Cargo test runner. Most active crates expose crate-local integration suites under `crates/*/tests`, and several crates also define examples and benchmarks that are compiled in release verification flows.

Useful setup pieces:

- `rustfmt` and `clippy` for the formatting/lint gates
- standard `cargo test` for unit, integration, and doc tests
- `cargo bench --no-run` for benchmark target compilation
- optional verification bundle from `scripts/verification-tools/install-verification-tools.sh` for tools such as `cargo-deny`, `cargo-fuzz`, and `cargo-nextest`

The canonical repository-wide quality gate is `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`.

## ⚙️ Running Tests

### ✅ Common commands

```bash
# Fast confidence check
cargo test --workspace

# Canonical release-style suite from full_verify.sh step 3
cargo test --workspace --release --lib --bins --tests --examples --all-features

# Rust doc tests
cargo test --workspace --release --all-features --doc

# Benchmark compilation gate
cargo bench --workspace --all-features --no-run
```

### ✅ Focused commands from live repo workflows

```bash
# Core boundary guard
cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture

# Wallet security/logging guards
cargo test --release -p z00z_wallets --test test_rpc_logging_acceptance -- --nocapture
cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy -- --nocapture
cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture

# Networks RPC transport suite
cargo test --release -p z00z_networks_rpc -- --nocapture

# HJMT topology contract backed by checked-in manifests
cargo test -p z00z_rollup_node --release --features test-params-fast \
  --test test_hjmt_topology test_grid57_matches_contract -- --nocapture
```

### ✅ Canonical full gate

```bash
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh
```

That script runs, in order:

1. `cargo fmt --check`
2. `cargo clippy --workspace --release --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --release --lib --bins --tests --examples --all-features`
4. `cargo test --workspace --release --all-features --doc`
5. `cargo bench --workspace --all-features --no-run`

## 🧪 Where Tests Live

| Area | Location | Notes |
|---|---|---|
| Core protocol | `crates/z00z_core/tests/` | Asset, genesis, manifest, and live guardrail coverage |
| Crypto | `crates/z00z_crypto/tests/` | Proof, domain separation, vector, and fail-closed coverage |
| Storage | `crates/z00z_storage/tests/` | Settlement, proof, HJMT, checkpoint, and replay-oriented suites |
| Wallets | `crates/z00z_wallets/tests/` | RPC, wallet persistence, receiver, backup, and typed object flows |
| Runtime | `crates/z00z_runtime/aggregators/tests/`, `validators/tests/`, `watchers/tests/` | Planner, verdict, and observation-specific guards |
| Rollup node | `crates/z00z_rollup_node/tests/` | CLI/config binding, DA adapter, lifecycle, and topology tests |
| Simulator | `crates/z00z_simulator/tests/` | Scenario and interop validation across workspace boundaries |

## 🔒 CI Guard Workflows

The repository ships three focused guard workflows under `.github/workflows/`:

- `boundary-guards.yml` audits boundary scripts and runs `z00z_core` plus wallet live-boundary tests.
- `security-hygiene-guards.yml` audits secret, RNG, panic, and log-redaction hygiene and then runs storage, wallet, and RPC tests.
- `release-safety-guards.yml` audits release-feature guards and runs wallet production-hardening tests.

These are narrower than `full_verify.sh`, but they document which tests are treated as especially important by the repository.

## 🛠️ Extended Verification

Optional deeper tooling is wired through repository scripts:

```bash
./scripts/verification-tools/install-verification-tools.sh
source ./scripts/verify-env.sh
./scripts/run-cargo-fuzz.sh crates/z00z_core/fuzz list
```

`scripts/verify-env.sh` exports paths and environment variables for the local formal-verification bundle under `tools/formal_verification/`.

## 📚 Related Docs

- [Development guide](../guides/development.md)
- [Configuration guide](../configuration/overview.md)
- [Architecture overview](../architecture/overview.md)
