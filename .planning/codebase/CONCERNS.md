# Codebase Concerns

**Analysis Date:** 2026-07-07

## Tech Debt

**Wallet network and directory seams are still explicitly partial:**
- Issue: several wallet-facing network seams still declare themselves as stub or placeholder contracts.
- Files: `crates/z00z_wallets/src/network/network_kernel.rs`, `crates/z00z_wallets/src/services/app_chain_network.rs`, `crates/z00z_wallets/src/services/directory_auth.rs`
- Impact: transport- and network-owned behavior can drift into placeholder modules before the live contract is fully defined.
- Fix approach: move real behavior into stable wallet/runtime ownership seams and keep placeholder modules minimal or feature-gated.

**Extension and telemetry lanes are committed but not live workspace surfaces:**
- Issue: prototype extension crates and the telemetry crate exist on disk, but the root workspace does not include them and many trees still contain `empty_file` placeholders.
- Files: `crates/z00z_extensions/dao/Cargo.toml`, `crates/z00z_extensions/local_economy/Cargo.toml`, `crates/z00z_extensions/protocol_rules/Cargo.toml`, `crates/z00z_extensions/treasury/Cargo.toml`, `crates/z00z_telemetry/src/lib.rs`
- Impact: planning can overestimate what is executable or integrated today.
- Fix approach: either promote a crate into the root workspace with real code/tests or clearly keep it as a documented prototype lane.

**Large files still create broad edit and review blast radius:**
- Issue: several implementation and test files are far beyond a small-auditable-unit size.
- Files: `crates/z00z_wallets/src/services/test_wallet_service.rs` (6758 lines), `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` (6473 lines), `crates/z00z_wallets/src/redb_store/test_store_suite.rs` (5585 lines), `crates/z00z_storage/src/settlement/proof_batch.rs` (2035 lines), `crates/z00z_rollup_node/src/config.rs` (1784 lines)
- Impact: change review quality drops and local regressions become harder to isolate.
- Fix approach: split by responsibility seam first, especially around simulator observability, wallet store tests, proof-batch support, and rollup configuration parsing.

**Guardrail tests are tightly coupled to planning and docs artifacts:**
- Issue: several live tests `include_str!` planning docs, workflow files, and source-policy documents as part of correctness enforcement.
- Files: `crates/z00z_storage/tests/test_live_guardrails.rs`, `crates/z00z_wallets/tests/test_wallet_capability_matrix.rs`, `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`
- Impact: documentation drift can break release gates even when code behavior is unchanged, and policy updates require synchronized code/test/doc edits.
- Fix approach: keep that coupling intentional and update planning/docs in the same wave as code when tests rely on them.

## Known Bugs

**Rollup node process contract is still mode-limited:**
- Symptoms: the executable path currently treats `aggregator` mode as the only supported live mode.
- Files: `crates/z00z_rollup_node/src/main.rs`, `crates/z00z_rollup_node/src/runtime.rs`, `crates/z00z_rollup_node/src/status.rs`
- Trigger: attempting to treat detached validator/watcher bindings as fully executable process modes.
- Workaround: use the current rollup binary as an aggregator-mode process contract only and treat other bindings as structural/runtime metadata until promoted.

## Security Considerations

**Debug and verbose wallet features can expose sensitive material:**
- Risk: feature flags can enable decrypted wallet export or metadata-heavy logging if compiled or run incorrectly.
- Files: `crates/z00z_wallets/Cargo.toml`, `crates/z00z_simulator/Cargo.toml`, `crates/z00z_wallets/src/rpc/logging*.rs`
- Current mitigation: feature comments and release guard workflows explicitly forbid those paths in production-capable builds.
- Recommendations: keep release guardrails green and avoid enabling `wallet_debug_tools`, `eviction-logs`, or `verbose-logging` outside controlled debug/test runs.

**Optional UI dependency chain carries accepted advisory debt:**
- Risk: the optional wallet GUI stack still depends transitively on ignored `quick-xml` advisories.
- Files: `.cargo/audit.toml`, `deny.toml`, `crates/z00z_wallets/Cargo.toml`
- Current mitigation: advisory exceptions are documented with reasons in `deny.toml` and `.cargo/audit.toml`.
- Recommendations: remove the ignore entries once the optional desktop stack can move off the pinned transitive chain.

## Performance Bottlenecks

**HJMT proof generation and settlement proof paths are heavy hotspots:**
- Problem: proof generation, batch proof verification, and reload/recovery flows are large and performance-sensitive.
- Files: `crates/z00z_storage/src/settlement/proof.rs`, `crates/z00z_storage/src/settlement/proof_batch.rs`, `crates/z00z_storage/src/settlement/hjmt_cache.rs`, `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`
- Cause: deep proof logic, large state transitions, and expensive persistence/proof checks.
- Improvement path: continue extracting focused proof helpers and keep bench coverage such as `crates/z00z_storage/benches/settlement_proofs.rs` and `crates/z00z_storage/benches/settlement_hjmt.rs` current.

**Simulator observability and runtime-stage reporting are oversized:**
- Problem: simulator observability/reporting logic is concentrated in very large modules.
- Files: `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`, `crates/z00z_simulator/src/scenario_1/stage_13/hjmt_examples.rs`, `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- Cause: one harness owns many reporting, artifact, and validation concerns at once.
- Improvement path: split report building, artifact capture, and verification helpers into smaller scenario support modules.

## Fragile Areas

**Wallet RedB and RPC boundaries:**
- Files: `crates/z00z_wallets/src/redb_store/`, `crates/z00z_wallets/src/rpc/`, `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`
- Why fragile: storage schema, RPC contracts, and logging policy move together and have both native and WASM implications.
- Safe modification: change one boundary at a time and run focused wallet RPC and persistence tests in release mode.
- Test coverage: broad, but the surface area is large and many tests are integration-heavy.

**Simulator scenario orchestration:**
- Files: `crates/z00z_simulator/src/scenario_1/`, `crates/z00z_simulator/tests/scenario_1/`
- Why fragile: scenario stages encode many cross-crate assumptions plus deterministic artifact and output-root contracts.
- Safe modification: update stage code together with unified-gate and stage-surface tests.
- Test coverage: strong, but failures can be noisy because a single scenario fan-outs across many crates.

## Scaling Limits

**Prototype lanes outside the workspace:**
- Current capacity: prototype crates exist as committed directories only.
- Limit: they do not participate in normal workspace build/test flows from `Cargo.toml`.
- Scaling path: explicitly add a prototype crate to the root workspace only when it has real source, tests, and ownership boundaries.

## Dependencies at Risk

**`quick-xml` advisory exceptions:**
- Risk: accepted advisories remain in the dependency policy for the optional GUI chain.
- Impact: security review must keep treating the desktop GUI stack as a special case.
- Migration plan: upgrade the optional GUI path and remove the advisory exceptions from `deny.toml` and `.cargo/audit.toml`.

**Dual bincode generation split:**
- Risk: `deny.toml` documents a coordinated migration problem because direct code uses bincode v2 while vendored Tari code still ties the workspace to older format assumptions.
- Impact: serialization refactors are harder to stage safely across boundaries.
- Migration plan: coordinate format migration at the workspace boundary instead of changing only one crate.

## Missing Critical Features

**OnionNet is still a placeholder seam:**
- Problem: the privacy-ingress crate exists, but it is intentionally placeholder-shaped and does not own a live overlay stack yet.
- Blocks: treating OnionNet as a production transport boundary.

**Telemetry crate is still skeletal:**
- Problem: `crates/z00z_telemetry/src/lib.rs` is effectively empty.
- Blocks: a dedicated first-party telemetry crate contract beyond the current logger/watcher split.

## Test Coverage Gaps

**Prototype extension crates are effectively untested as executable surfaces:**
- What's not tested: workspace-level execution of `crates/z00z_extensions/dao/`, `local_economy/`, `protocol_rules/`, and `treasury/`.
- Files: `crates/z00z_extensions/*/Cargo.toml`, `crates/z00z_extensions/*/src/empty_file`
- Risk: those crates can look more complete than they are because they are committed but not live in the root workspace.
- Priority: Medium

**Empty telemetry surface has no behavior tests worth keeping green yet:**
- What's not tested: a real telemetry API contract.
- Files: `crates/z00z_telemetry/src/lib.rs`, `crates/z00z_telemetry/tests/empty_file`
- Risk: future telemetry work starts from a blank surface without existing contract tests.
- Priority: Medium

---

*Concerns audit: 2026-07-07*
