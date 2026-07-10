# Codebase Structure

**Analysis Date:** 2026-07-07

## Directory Layout

```text
z00z/
├── `.github/`          # Repo-local skills, prompts, agents, workflows, and requirements
├── `.planning/`        # GSD planning state, phases, graphs, and codebase documents
├── `config/`           # Root runtime configuration
├── `crates/`           # Rust workspace crates and non-workspace extension prototypes
├── `docs/`             # Whitepapers, specs, research, and tech papers
├── `scripts/`          # Build, audit, verification, profiling, and pentest helpers
├── `reports/`          # Generated verification and scenario artifacts
├── `website/`          # Website assets and collateral
├── `Cargo.toml`        # Workspace manifest
└── `.cargo/`           # Shared cargo aliases and build defaults
```

## Directory Purposes

**`.github/`:**
- Purpose: repository-local instructions and automation control plane.
- Contains: `skills/`, `prompts/`, `agents/`, `requirements/`, `workflows/`, and `gsd-core/`.
- Key files: `.github/copilot-instructions.md`, `.github/workflows/*.yml`, `.github/agents/gsd-codebase-mapper.agent.md`.

**`.planning/`:**
- Purpose: GSD planning state and generated repo intelligence.
- Contains: `STATE.md`, `ROADMAP.md`, `phases/`, `graphs/`, `codebase/`.
- Key files: `.planning/STATE.md`, `.planning/ROADMAP.md`, `.planning/codebase/*.md`.

**`crates/z00z_utils/`:**
- Purpose: shared abstractions for config, codec, I/O, logger, metrics, RNG, time, and hardening.
- Contains: `src/config/`, `src/codec/`, `src/io/`, `src/logger/`, `src/metrics/`, `src/rng/`, `src/time/`.
- Key files: `crates/z00z_utils/src/lib.rs`, `crates/z00z_utils/src/io/mod.rs`, `crates/z00z_utils/src/logger/mod.rs`.

**`crates/z00z_crypto/`:**
- Purpose: public crypto facade and vendor boundary.
- Contains: `src/`, benches/tests, and the read-only `tari/` subtree.
- Key files: `crates/z00z_crypto/src/lib.rs`, `crates/z00z_crypto/src/hash_domain.rs`, `crates/z00z_crypto/tari/`.

**`crates/z00z_core/`:**
- Purpose: protocol-domain objects, genesis, config schemas, and CLI tooling.
- Contains: `src/assets/`, `src/genesis/`, `configs/`, `bin/`, `tests/`, `examples/`, `benches/`.
- Key files: `crates/z00z_core/src/lib.rs`, `crates/z00z_core/configs/devnet_genesis_config.yaml`, `crates/z00z_core/configs/devnet_assets_config.yaml`, `crates/z00z_core/bin/genesis_cli.rs`.

**`crates/z00z_storage/`:**
- Purpose: settlement storage, checkpoints, serialization, snapshots, and fixture support.
- Contains: `src/backend/`, `src/settlement/`, `src/checkpoint/`, `src/serialization/`, `src/snapshot/`, `src/fixture_support/`, `tests/`, `benches/`.
- Key files: `crates/z00z_storage/src/lib.rs`, `crates/z00z_storage/src/settlement/store.rs`, `crates/z00z_storage/src/checkpoint/build.rs`.

**`crates/z00z_wallets/`:**
- Purpose: wallet runtime and application surface.
- Contains: `src/app/`, `src/backup/`, `src/chain/`, `src/config/`, `src/db/`, `src/key/`, `src/persistence/`, `src/receiver/`, `src/redb_store/`, `src/rpc/`, `src/services/`, `src/tx/`, `src/wasm/`, `src/egui_views/`, `tests/`, `bin/`, `benches/`.
- Key files: `crates/z00z_wallets/src/lib.rs`, `crates/z00z_wallets/src/config/wallet_config.yaml`, `crates/z00z_wallets/src/rpc/mod.rs`, `crates/z00z_wallets/src/services/mod.rs`.

**`crates/z00z_networks/`:**
- Purpose: transport-focused crates.
- Contains: `rpc/`, `onionnet/`, and `docker_nodes/`.
- Key files: `crates/z00z_networks/rpc/src/lib.rs`, `crates/z00z_networks/rpc/src/wasm_client.rs`, `crates/z00z_networks/onionnet/src/lib.rs`.

**`crates/z00z_runtime/`:**
- Purpose: runtime policy crates.
- Contains: `aggregators/`, `validators/`, `watchers/`.
- Key files: `crates/z00z_runtime/aggregators/src/lib.rs`, `crates/z00z_runtime/validators/src/lib.rs`, `crates/z00z_runtime/watchers/src/lib.rs`.

**`crates/z00z_rollup_node/`:**
- Purpose: rollup node runtime and DA/process wiring.
- Contains: `src/`, `README.md`, and tests through `#[cfg(test)]` plus dev-dependencies.
- Key files: `crates/z00z_rollup_node/src/lib.rs`, `crates/z00z_rollup_node/src/main.rs`, `crates/z00z_rollup_node/src/config.rs`.

**`crates/z00z_simulator/`:**
- Purpose: integration harness and scenario execution.
- Contains: `src/scenario_1/`, `src/scenario_11/`, shared actor/context/result modules, `tests/scenario_1/`, `bin/scenario_1.rs`.
- Key files: `crates/z00z_simulator/src/lib.rs`, `crates/z00z_simulator/src/scenario_1/runner.rs`, `crates/z00z_simulator/tests/scenario_1/main.rs`.

**`crates/z00z_extensions/`:**
- Purpose: extension prototypes and future slices that are committed but not active workspace members.
- Contains: prototype crates such as `dao/`, `local_economy/`, `protocol_rules/`, `treasury/`, plus design/support folders.
- Key files: `crates/z00z_extensions/dao/Cargo.toml`, `crates/z00z_extensions/local_economy/Cargo.toml`, `crates/z00z_extensions/protocol_rules/Cargo.toml`, `crates/z00z_extensions/treasury/Cargo.toml`.

**`crates/z00z_telemetry/`:**
- Purpose: reserved telemetry crate boundary.
- Contains: minimal crate skeleton with benches/examples/tests placeholders.
- Key files: `crates/z00z_telemetry/Cargo.toml`, `crates/z00z_telemetry/src/lib.rs`.

## Key File Locations

**Entry Points:**
- `Cargo.toml`: workspace membership and shared metadata.
- `crates/z00z_wallets/src/lib.rs`: wallet crate facade.
- `crates/z00z_networks/rpc/src/lib.rs`: transport facade.
- `crates/z00z_runtime/aggregators/src/lib.rs`: aggregator policy facade.
- `crates/z00z_runtime/validators/src/lib.rs`: validator policy facade.
- `crates/z00z_runtime/watchers/src/lib.rs`: watcher policy facade.
- `crates/z00z_rollup_node/src/main.rs`: rollup process entrypoint.
- `crates/z00z_simulator/bin/scenario_1.rs`: simulator binary entrypoint.

**Configuration:**
- `.cargo/config.toml`: build aliases and shared test-fast defaults.
- `config/z00z_blockchain_config.yaml`: root chain config.
- `crates/z00z_core/configs/*.yaml`: canonical devnet manifests and schemas, including the typed genesis manifest and the secondary asset-registry catalog at `crates/z00z_core/configs/devnet_assets_config.yaml`.
- `crates/z00z_wallets/src/config/wallet_config.yaml`: wallet config.
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`: scenario config.
- `crates/z00z_rollup_node/src/config.rs`: node/process config model.

**Core Logic:**
- `crates/z00z_core/src/assets/`: asset and related protocol-domain logic.
- `crates/z00z_core/src/genesis/`: genesis generation and validation.
- `crates/z00z_storage/src/settlement/`: settlement state, proofs, and HJMT flows.
- `crates/z00z_storage/src/checkpoint/`: checkpoint artifact and finalization logic.
- `crates/z00z_wallets/src/receiver/`: inbound and scan flows.
- `crates/z00z_wallets/src/tx/`: tx build/verify/state flows.
- `crates/z00z_wallets/src/rpc/`: wallet RPC surface.
- `crates/z00z_runtime/aggregators/src/`: routing, dispatch, quorum, and publication logic.
- `crates/z00z_simulator/src/scenario_1/`: staged integration flow.

**Testing:**
- `crates/z00z_core/tests/`
- `crates/z00z_storage/tests/`
- `crates/z00z_wallets/tests/`
- `crates/z00z_simulator/tests/scenario_1/`
- `.github/workflows/*.yml`

## Naming Conventions

**Files:**
- Rust modules use `snake_case.rs` and `mod.rs`, for example `crates/z00z_storage/src/settlement/proof_batch.rs` and `crates/z00z_wallets/src/services/mod.rs`.
- Integration tests strongly prefer `test_*.rs`, for example `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`.
- Scenario stage code lives under numbered directories such as `crates/z00z_simulator/src/scenario_1/stage_6/`.

**Directories:**
- Crate `src/` trees are grouped by responsibility, not by technical layer dumping grounds.
- Scenario and runtime directories encode domain lanes directly: `receiver/`, `tx/`, `settlement/`, `checkpoint/`, `aggregators/`, `validators/`, `watchers/`.

## Where to Add New Code

**New protocol feature:**
- Primary code: `crates/z00z_core/src/assets/` or `crates/z00z_core/src/genesis/`
- Tests: `crates/z00z_core/tests/`

**New settlement or checkpoint flow:**
- Primary code: `crates/z00z_storage/src/settlement/`, `crates/z00z_storage/src/checkpoint/`, `crates/z00z_storage/src/serialization/`, or `crates/z00z_storage/src/snapshot/`
- Tests: `crates/z00z_storage/tests/`

**New wallet runtime feature:**
- Primary code: `crates/z00z_wallets/src/receiver/`, `crates/z00z_wallets/src/tx/`, `crates/z00z_wallets/src/services/`, or `crates/z00z_wallets/src/rpc/`
- Persistence: `crates/z00z_wallets/src/db/`, `crates/z00z_wallets/src/redb_store/`, `crates/z00z_wallets/src/persistence/`
- Tests: `crates/z00z_wallets/tests/`

**New runtime policy:**
- Implementation: `crates/z00z_runtime/aggregators/`, `crates/z00z_runtime/validators/`, or `crates/z00z_runtime/watchers/`

**New rollup behavior:**
- Implementation: `crates/z00z_rollup_node/src/`

**New simulator stage or harness helper:**
- Implementation: `crates/z00z_simulator/src/scenario_1/` or `crates/z00z_simulator/src/scenario_11/`
- Tests: `crates/z00z_simulator/tests/scenario_1/`

**New shared helper:**
- Shared helpers: `crates/z00z_utils/src/`

## Special Directories

**`crates/z00z_crypto/tari/`:**
- Purpose: vendored Tari cryptography backend.
- Generated: No.
- Committed: Yes.

**`crates/z00z_extensions/*`:**
- Purpose: committed prototype extension crates and design slices.
- Generated: No.
- Committed: Yes.

**`.planning/codebase/`:**
- Purpose: generated codebase intelligence for GSD planning/execution.
- Generated: Yes.
- Committed: Yes.

**`.codegraph/`:**
- Purpose: local CodeGraph index used for structural code navigation.
- Generated: Yes.
- Committed: No.

**`target/workspace/`:**
- Purpose: shared Cargo target directory enforced by `.cargo/config.toml`.
- Generated: Yes.
- Committed: No.

---

*Structure analysis: 2026-07-07*
