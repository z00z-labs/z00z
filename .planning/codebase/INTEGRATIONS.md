# External Integrations

**Analysis Date:** 2026-07-07

## APIs & External Services

**Transport and RPC:**
- Local JSON-RPC transport is implemented in `crates/z00z_networks/rpc/src/lib.rs`.
  - SDK/Client: `jsonrpsee` in `crates/z00z_networks/rpc/Cargo.toml`.
  - Auth: no external API key or hosted RPC provider contract detected in active workspace crates.
- Wallet-facing RPC handlers live in `crates/z00z_wallets/src/rpc/`.
  - SDK/Client: `jsonrpsee` in `crates/z00z_wallets/Cargo.toml`.
  - Auth: wallet session and capability gating are crate-owned, not delegated to an external IdP.

**Data availability and rollup adapters:**
- Rollup data-availability integration is modeled through `crates/z00z_rollup_node/src/da.rs` and `crates/z00z_rollup_node/src/celestia_local.rs`.
  - SDK/Client: project-owned adapter traits and local adapter types.
  - Auth: no hosted DA credentials detected in the committed workspace.

**Privacy overlay seam:**
- OnionNet remains a placeholder crate in `crates/z00z_networks/onionnet/`.
  - SDK/Client: none.
  - Auth: not applicable.

## Data Storage

**Databases:**
- RedB-backed wallet storage in `crates/z00z_wallets/src/db/` and `crates/z00z_wallets/src/redb_store/`.
  - Connection: local file-backed `.wlt` contract, no network DSN detected.
  - Client: `redb` in `crates/z00z_wallets/Cargo.toml`.
- RedB + JMT-backed settlement storage in `crates/z00z_storage/src/backend/` and `crates/z00z_storage/src/settlement/`.
  - Connection: local storage root and backend config, not an external DB service.
  - Client: `redb` and `jmt` in `crates/z00z_storage/Cargo.toml`.

**File Storage:**
- Local filesystem only through `z00z_utils::io` and crate-owned persistence helpers.
- Snapshot/export/archive artifacts are written under crate-local or `target/` paths, for example `crates/z00z_simulator/src/scenario_1/` and `reports/`.

**Caching:**
- In-memory LRU/cache layers in `crates/z00z_wallets/src/key/manager_impl_cache.rs`, `crates/z00z_wallets/src/receiver/asset_scan_ephemeral_cache.rs`, and `crates/z00z_storage/src/settlement/hjmt_cache.rs`.
- No external cache service such as Redis or Memcached detected.

## Authentication & Identity

**Auth Provider:**
- Custom wallet-owned authentication and session model.
  - Implementation: password- and key-based flows in `crates/z00z_wallets/src/services/`, `crates/z00z_wallets/src/rpc/`, `crates/z00z_wallets/src/key/`, and `crates/z00z_wallets/src/security/`.
- No external OAuth, OIDC, SAML, or hosted auth provider detected.

## Monitoring & Observability

**Error Tracking:**
- No external SaaS error tracker detected.

**Logs:**
- Project-owned logging abstractions in `crates/z00z_utils/src/logger/`.
- RPC logging policy and middleware in `crates/z00z_wallets/src/rpc/logging*.rs`.
- Runtime watcher evidence and alerts in `crates/z00z_runtime/watchers/src/`.

## CI/CD & Deployment

**Hosting:**
- Local binaries and local Docker/devnet orchestration are present.
- Devnet compose assets live in `docker/compose.hjmt-local.yaml`.

**CI Pipeline:**
- GitHub Actions workflows in `.github/workflows/boundary-guards.yml`, `.github/workflows/release-safety-guards.yml`, and `.github/workflows/security-hygiene-guards.yml`.
- Audit helpers are shell scripts under `scripts/audit/`.

## Environment Configuration

**Required env vars:**
- No application-wide `.env` contract is committed as a primary integration surface.
- Process-scoped HJMT and run-directory env contracts are defined in `crates/z00z_rollup_node/src/process_devnet.rs`.
- Test toggles and debug logging toggles appear in test code such as `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`.

**Secrets location:**
- Secret values are not stored in the active codebase map and were not read from any `.env*` files.
- Crypto materials and passwords are expected to stay in runtime inputs, encrypted wallet payloads, or CI/runtime environment, not in committed config files.

## Webhooks & Callbacks

**Incoming:**
- None detected as HTTP webhook endpoints.
- Local callback-style flows are internal trait/transport callbacks inside `crates/z00z_networks/rpc/`, `crates/z00z_runtime/aggregators/`, and `crates/z00z_runtime/watchers/`.

**Outgoing:**
- No external webhook emitters detected.
- Outbound communication is modeled as local RPC/WebSocket transport and local file/report generation.

---

*Integration audit: 2026-07-07*
