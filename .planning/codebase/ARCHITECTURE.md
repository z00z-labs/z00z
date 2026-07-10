<!-- refreshed: 2026-07-07 -->
# Architecture

**Analysis Date:** 2026-07-07

## System Overview

```text
Workspace root
├── User-facing and orchestration
│   ├── `crates/z00z_wallets/`
│   ├── `crates/z00z_rollup_node/`
│   └── `crates/z00z_simulator/`
├── Runtime policy and transport
│   ├── `crates/z00z_runtime/aggregators/`
│   ├── `crates/z00z_runtime/validators/`
│   ├── `crates/z00z_runtime/watchers/`
│   └── `crates/z00z_networks/`
├── Protocol and state authority
│   ├── `crates/z00z_core/`
│   └── `crates/z00z_storage/`
└── Shared foundation
    ├── `crates/z00z_utils/`
    └── `crates/z00z_crypto/`
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| Workspace manifest | Declares active workspace members, lints, shared package metadata, and vendored Tari keep-set | `Cargo.toml` |
| `z00z_utils` | Owns config, codec, I/O, logging, metrics, RNG, time, and OS hardening abstractions | `crates/z00z_utils/src/lib.rs` |
| `z00z_crypto` | Exposes the public crypto facade over the read-only Tari backend | `crates/z00z_crypto/src/lib.rs` |
| `z00z_core` | Owns protocol objects, configs, genesis generation, and canonical domain types | `crates/z00z_core/src/lib.rs` |
| `z00z_storage` | Owns settlement storage, checkpoint artifacts, serialization, and snapshot contracts | `crates/z00z_storage/src/lib.rs` |
| `z00z_wallets` | Owns wallet runtime, RPC, persistence, receiver flows, tx logic, and native/WASM adapters | `crates/z00z_wallets/src/lib.rs` |
| `z00z_networks_rpc` | Owns transport-only JSON-RPC plumbing for native and WASM callers | `crates/z00z_networks/rpc/src/lib.rs` |
| `onionnet` | Reserves the privacy-overlay crate boundary without owning live transport logic yet | `crates/z00z_networks/onionnet/src/lib.rs` |
| `z00z_aggregators` | Owns batch planning, routing, quorum, dispatch, and publication services | `crates/z00z_runtime/aggregators/src/lib.rs` |
| `z00z_validators` | Owns settlement theorem, checkpoint, spend, claim, and tx verification seams | `crates/z00z_runtime/validators/src/lib.rs` |
| `z00z_watchers` | Owns alerts, publication watches, censorship, and evidence export | `crates/z00z_runtime/watchers/src/lib.rs` |
| `z00z_rollup_node` | Owns local rollup node runtime wiring, DA adapters, and process/devnet orchestration | `crates/z00z_rollup_node/src/lib.rs` |
| `z00z_simulator` | Owns end-to-end scenario harnesses and integration artifact flows | `crates/z00z_simulator/src/lib.rs` |

## Pattern Overview

**Overall:** layered Rust workspace with crate-root facades, storage-owned settlement state, and simulator-as-harness rather than simulator-as-domain-owner.

**Key Characteristics:**
- Public APIs are exposed from narrow `src/lib.rs` facades and re-export lists.
- Storage is settlement-first: the live state model sits under `crates/z00z_storage/src/settlement/`, not an older `assets/` subtree.
- Wallet orchestration is split across domain modules such as `receiver/`, `tx/`, `rpc/`, `services/`, `db/`, `redb_store/`, and `wasm/`.
- Runtime policy is broken into dedicated aggregator, validator, and watcher crates instead of one monolith.
- Transport is intentionally separated into `crates/z00z_networks/rpc/` and does not own higher-level business state.

## Layers

**Foundation:**
- Purpose: reusable abstractions and cryptographic primitives.
- Location: `crates/z00z_utils/`, `crates/z00z_crypto/`
- Contains: codec, config, I/O, logger, metrics, RNG, time, OS hardening, crypto facade, domain-separated hashing, proofs.
- Depends on: standard Rust crates plus vendored Tari under `crates/z00z_crypto/tari/`.
- Used by: every active business crate in the workspace.

**Protocol:**
- Purpose: canonical domain models, genesis, and config authority.
- Location: `crates/z00z_core/src/`, `crates/z00z_core/configs/`
- Contains: `assets/`, `genesis/`, vouchers, rights, domain helpers, CLI bins, config schemas.
- The typed bootstrap authority lives at `crates/z00z_core/configs/devnet_genesis_config.yaml`; `crates/z00z_core/configs/devnet_assets_config.yaml` remains secondary registry data.
- Depends on: `z00z_utils`, `z00z_crypto`.
- Used by: storage, wallets, runtime policy crates, rollup node, and simulator.

**State and proof authority:**
- Purpose: settlement storage, checkpoint contracts, serialization, and snapshots.
- Location: `crates/z00z_storage/src/`
- Contains: `backend/`, `settlement/`, `checkpoint/`, `serialization/`, `snapshot/`, `fixture_support/`.
- Depends on: `z00z_core`, `z00z_crypto`, `z00z_utils`.
- Used by: wallets, runtime policy crates, rollup node, and simulator.

**Wallet application layer:**
- Purpose: wallet-owned runtime, local RPC, persistence, receiver flows, tx construction, and browser/native splits.
- Location: `crates/z00z_wallets/src/`
- Contains: `app/`, `backup/`, `chain/`, `claim/`, `config/`, `db/`, `domains/`, `key/`, `network/`, `persistence/`, `receiver/`, `redb_store/`, `rpc/`, `security/`, `services/`, `stealth/`, `tx/`, `wallet/`, `wasm/`, `egui_views/`.
- Depends on: `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_utils`, and `z00z_networks_rpc`.
- Used by: local binaries, simulator, and rollup-facing verification/test flows.

**Runtime policy and node layer:**
- Purpose: route planning, validation, watching, and node composition.
- Location: `crates/z00z_runtime/*`, `crates/z00z_rollup_node/src/`
- Contains: planner, quorum, dispatch, validator verdicts, publication watches, DA adapters, node runtime, and process-devnet files.
- Depends on: storage, wallets, crypto, utils, and core protocol types.
- Used by: rollup runtime and simulator scenarios.

**Transport and harness layer:**
- Purpose: local/WASM JSON-RPC transport plus integration harnesses.
- Location: `crates/z00z_networks/`, `crates/z00z_simulator/`
- Contains: `rpc/`, `onionnet/`, simulator `scenario_1/`, simulator `scenario_11/`, shared actor/context/design/result modules.
- Depends on: wallets, runtime policy, storage, core, and crypto.
- Used by: browser/native RPC clients, local simulations, and release guard tests.

## Data Flow

### Primary Request Path

1. Local RPC requests enter through `crates/z00z_wallets/src/rpc/` and `crates/z00z_networks/rpc/src/lib.rs`.
2. Wallet-owned orchestration delegates to `crates/z00z_wallets/src/services/`, `crates/z00z_wallets/src/receiver/`, and `crates/z00z_wallets/src/tx/`.
3. Canonical state and proof material are persisted and checked through `crates/z00z_storage/src/settlement/`, `crates/z00z_storage/src/checkpoint/`, and `crates/z00z_storage/src/snapshot/`.
4. Downstream runtime verification flows consume the result through `crates/z00z_runtime/validators/src/verdict.rs`, `crates/z00z_runtime/watchers/src/publication.rs`, and `crates/z00z_rollup_node/src/lib.rs`.

### Scenario and Replay Flow

1. `crates/z00z_simulator/bin/scenario_1.rs` and `crates/z00z_simulator/src/scenario_1/runner.rs` orchestrate scenario execution.
2. Stage modules under `crates/z00z_simulator/src/scenario_1/stage_*` call wallet, storage, runtime, and rollup facades.
3. Unified release gates under `crates/z00z_simulator/tests/scenario_1/` validate the resulting artifacts, roots, proofs, and persistence.

**State Management:**
- Shared durable settlement state is storage-owned in `crates/z00z_storage/src/settlement/`.
- Wallet-private durable state is owned by `crates/z00z_wallets/src/db/`, `crates/z00z_wallets/src/redb_store/`, and `crates/z00z_wallets/src/persistence/`.
- Simulator artifacts are scenario-owned under `crates/z00z_simulator/src/scenario_1/` and `target/workspace/`.

## Key Abstractions

**Shared abstraction boundary:**
- Purpose: prevent direct low-level I/O/time/serialization/logging drift.
- Examples: `crates/z00z_utils/src/io/`, `crates/z00z_utils/src/config/`, `crates/z00z_utils/src/logger/`, `crates/z00z_utils/src/rng/`, `crates/z00z_utils/src/time/`.
- Pattern: central project-owned wrappers used across business crates.

**Settlement store contract:**
- Purpose: keep storage, roots, proofs, and checkpoint handoff under one owner.
- Examples: `crates/z00z_storage/src/settlement/store.rs`, `crates/z00z_storage/src/checkpoint/build.rs`, `crates/z00z_storage/src/snapshot/store.rs`.
- Pattern: typed storage contract with backend-private implementation details.

**Wallet transport/runtime split:**
- Purpose: keep transport-only JSON-RPC wiring separate from wallet business logic.
- Examples: `crates/z00z_wallets/src/rpc/`, `crates/z00z_wallets/src/services/`, `crates/z00z_networks/rpc/src/lib.rs`.
- Pattern: local RPC methods map into service and tx/receiver domains rather than owning state directly.

**Runtime service split:**
- Purpose: preserve clear routing between planning, validation, and watching.
- Examples: `crates/z00z_runtime/aggregators/src/lib.rs`, `crates/z00z_runtime/validators/src/lib.rs`, `crates/z00z_runtime/watchers/src/lib.rs`.
- Pattern: one crate per policy boundary.

## Entry Points

**Workspace root:**
- Location: `Cargo.toml`
- Triggers: any workspace build, test, lint, or binary invocation.
- Responsibilities: workspace membership, shared metadata, shared lint settings, vendored Tari keep-set.

**Wallet library facade:**
- Location: `crates/z00z_wallets/src/lib.rs`
- Triggers: wallet binaries, simulator, runtime callers, RPC registration.
- Responsibilities: exports wallet/public error types, receiver types, tx verification helpers, services, native/WASM splits.

**Rollup node executable:**
- Location: `crates/z00z_rollup_node/src/main.rs`
- Triggers: `z00z_rollup_node` binary execution.
- Responsibilities: process-contract CLI parsing and aggregator-mode startup.

**Simulator entrypoints:**
- Location: `crates/z00z_simulator/bin/scenario_1.rs`, `crates/z00z_simulator/src/lib.rs`
- Triggers: local scenario execution and integration tests.
- Responsibilities: scenario orchestration, harness wiring, artifact reporting.

## Architectural Constraints

- **Threading:** async/native orchestration relies on Tokio, while proof- and batch-heavy work uses Rayon; see `crates/z00z_networks/rpc/Cargo.toml`, `crates/z00z_wallets/Cargo.toml`, and `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`.
- **Global state:** no single app-wide singleton dominates the workspace, but scenario/test caches and atomic scheduler state exist in `crates/z00z_simulator/src/scenario_1/support/fixture_cache.rs` and `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`.
- **Circular imports:** crate-root layering stays one-directional; simulator and rollup depend downward on wallet/storage/runtime facades rather than the inverse.
- **Vendor boundary:** `crates/z00z_crypto/tari/` is read-only vendor code and must not be modified.
- **Inactive side lanes:** `crates/z00z_extensions/*` contain committed prototype crates that are not root workspace members, and `crates/z00z_telemetry/` is still an empty crate boundary.

## Anti-Patterns

### Bypassing the shared utility boundary

**What happens:** business crates call low-level I/O, config, or time APIs directly.
**Why it's wrong:** it breaks the repository's one-source-of-truth rule and fragments behavior.
**Do this instead:** route file, codec, RNG, logger, and time calls through `crates/z00z_utils/src/lib.rs` and its submodules.

### Letting transport or simulator code become the domain owner

**What happens:** RPC or simulator code starts carrying canonical wallet/storage/runtime behavior.
**Why it's wrong:** it creates parallel authority paths and weakens public seams.
**Do this instead:** keep transport in `crates/z00z_networks/rpc/` and keep simulator logic dependent on stable crate facades such as `crates/z00z_wallets/src/lib.rs` and `crates/z00z_storage/src/lib.rs`.

### Treating placeholder seams as production-complete

**What happens:** callers extend `crates/z00z_networks/onionnet/`, `crates/z00z_wallets/src/network/network_kernel.rs`, or empty extension crates as if they were live contracts.
**Why it's wrong:** those seams are explicitly partial or placeholder-shaped and do not yet own full production behavior.
**Do this instead:** extend live contracts under wallet, storage, runtime, or rollup crates first, then narrow placeholder seams only when the owning behavior is ready.

## Error Handling

**Strategy:** crate-local typed errors plus explicit result aliases at ownership boundaries.

**Patterns:**
- `thiserror` is used throughout workspace crates such as `crates/z00z_storage/src/error.rs` and `crates/z00z_wallets/src/lib.rs`.
- Validation-heavy crates export typed domain errors instead of generic transport errors by default.
- Tests use `expect` and `unwrap` freely, while production surfaces center error typing and boundary checks.

## Cross-Cutting Concerns

**Logging:** centralized through `crates/z00z_utils/src/logger/` and specialized in wallet RPC logging modules under `crates/z00z_wallets/src/rpc/`.

**Validation:** layered across protocol (`crates/z00z_core/`), storage (`crates/z00z_storage/src/settlement/`), wallet tx/receiver logic (`crates/z00z_wallets/src/tx/`, `crates/z00z_wallets/src/receiver/`), and runtime validators (`crates/z00z_runtime/validators/src/`).

**Authentication:** custom wallet session, password, and capability gating live in `crates/z00z_wallets/src/services/`, `crates/z00z_wallets/src/security/`, and `crates/z00z_wallets/src/rpc/`.

---

*Architecture analysis: 2026-07-07*
