---
phase: 064-Gaps-Closing-3
artifact: test-spec
status: complete
source:
  - live code and tests
  - .planning/phases/064-Gaps-Closing-3/064-TODO.md
  - .planning/phases/064-Gaps-Closing-3/064-CONTEXT.md
  - .planning/phases/064-Gaps-Closing-3/064-01-PLAN.md
  - .planning/phases/064-Gaps-Closing-3/064-02-PLAN.md
  - .planning/phases/064-Gaps-Closing-3/064-03-PLAN.md
  - .planning/phases/064-Gaps-Closing-3/064-04-PLAN.md
  - .planning/phases/064-Gaps-Closing-3/064-05-PLAN.md
updated: 2026-06-30
---

<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 -->

# Phase 064 Test Specification

## 🎯 Purpose

This file is the live Phase 064 test authority for the implemented coverage
packet on the current tree.
`.planning/phases/064-Gaps-Closing-3/064-TODO.md` remains the normative
recommendation inventory. `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md`
remains the anti-drift mirror. `PLAN-064-G01` through `PLAN-064-G05`
remain the grouped execution packet.

Use this file against live code, live tests, emitted artifacts, and
release-mode validation output. Do not treat summaries as proof when the code
or test tree says otherwise.

This file defines:

- which local E2E and integration behaviors must be proven;
- which unit, integration, simulator, and repository-gate seams own the proof;
- which realistic examples demonstrate successful execution;
- which negative paths must reject, fail closed, or stay explicitly deferred;
- which checkpoint artifacts, roots, proofs, signatures, session checks, and
  publication bindings must be observed; and
- which measurable pass signals are required before any slice can be called
  closed.

Phase 064 is not a new product layer. It is a closure phase over the existing
simulator, wallet, storage, runtime, rollup, core, and repository-boundary
surfaces. The required tests must therefore reuse current truthful homes in
the codebase. Do not create a parallel simulator truth lane, wallet mutation
lane, checkpoint lane, theorem lane, publication-binding lane, or docs
authority lane.

In this repository, E2E for Phase 064 means real `scenario_1`, real
wallet-local mutation flows, real storage or rollup or runtime primitives, or
deterministic local distributed simulation using real project state. Browser
automation, real external chain transport, real OnionNet, and real DA network
transport are not Phase 064 proof targets.

## 📌 Coverage Contract

1. `064-S01` through `064-S05` map one-to-one to `PLAN-064-G01` through
   `PLAN-064-G05`.
2. All `28` `REC-064-*` rows must appear in the coverage appendix below and
   remain owned by exactly one scenario.
3. All `17` Markdown docs named by `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md`
   must stay attached to at least one owning scenario in the docs-corpus table
   below.
4. The `57` TODO rows and directives preserved by the strict row-class lock in
   `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md` remain contextual
   authority; this test packet may not silently reinterpret or collapse them.
5. A scenario is incomplete if any included `REC-064-*` row lacks a truthful
   test home, realistic positive example, fail-closed negative case, or
   measurable closure signal.
6. Graphify may orient codebase structure only; it is never admissible as
   factual source truth, coverage evidence, or acceptance proof for this test
   packet.
7. Execution-critical references in this file must use canonical repo-relative
   paths, not brace patterns, filename ranges, or shorthand aliases.

## 📌 TODO Directive Carry-Forward

### Top-Level TODO Steps 1-5

| TODO directive | Preserved here as | Owning scenario and plan | Carry-forward rule |
| --- | --- | --- | --- |
| Top-level TODO step 1: simulator | `064-S01` simulator final truth and packet integrity | `064-S01` / `PLAN-064-G01` | The canonical simulator lane must close first and must prove final checkpoint truth, hermetic stages, exact-home artifacts, secret-clean packets, and facade-only imports. |
| Top-level TODO step 2: wallet | `064-S02` wallet-local mutation truth | `064-S02` / `PLAN-064-G02` | Wallet-local mutation semantics must land on real local chain, broadcast, and durable tx owners before any downstream claim of live wallet closure. |
| Top-level TODO step 3: rpc | `064-S02` RPC audit truth and route coverage | `064-S02` / `PLAN-064-G02` | RPC route truth stays on the same second-wave owner set and must prove include-based registration coverage plus `app.wallet.open_wallet_source` wiring. |
| Top-level TODO step 4: wallet services | `064-S03` sensitive wallet surface and placeholder-owner closure | `064-S03` / `PLAN-064-G03` | Only live-backed wallet-service seams may be promoted; placeholder-only services stay explicitly bounded and must not advertise native or production guarantees they do not own. |
| Top-level TODO step 5: runtime/rollup | `064-S04` storage, theorem, recovery, DA, and publication-binding closure | `064-S04` / `PLAN-064-G04` | Runtime and rollup proof surfaces must stay local, deterministic, fail-closed, and singular; no real network or second semantic owner may replace the live local truth lane. |

### Ordered Closeout Groups

| TODO directive | Preserved here as | Owning scenario and plan | Carry-forward rule |
| --- | --- | --- | --- |
| Numbered closeout group `1-5` | `REC-064-P0-04` through `REC-064-P0-08` | `064-S03` / `PLAN-064-G03`, plus `064-S01` / `PLAN-064-G01` for `REC-064-P0-08` | These items close before numbered group `6-13`; secret-clean packet proof stays attached to the simulator owner because default release-packet secrecy is inseparable from canonical simulator truth. |
| Numbered closeout group `6-13` | `REC-064-P1-04` through `REC-064-P1-11` | `064-S03` / `PLAN-064-G03` and `064-S04` / `PLAN-064-G04` | These items close only after simulator, wallet-local mutation, and RPC truth are restored; no item may bypass storage, runtime, rollup, or wallet owners. |
| Numbered closeout group `14-18` | `REC-064-P2-05` through `REC-064-P2-09` | `064-S05` / `PLAN-064-G05`, plus `064-S01` / `PLAN-064-G01` for `REC-064-P2-07` | Boundary-CI and docs-hygiene guardrails close last, except the simulator-facade guard, which remains attached to the first-wave simulator owner because it protects the same canonical harness boundary. |

### Meta Directives

- The TODO instruction "Only new points below; do not repeat
  simulator/wallet-asset/RPC/genesis/local-DA themes" remains preserved by
  keeping the top-level five-step order separate from the numbered closeout
  groups above. This file must not merge those two directive surfaces into one
  generic list.
- The TODO instruction "Do not go to network/onion/remote-chain now" remains
  preserved through `064-S05`, the defer invariants, and the global reject
  conditions below. No Phase 064 acceptance path may depend on real OnionNet,
  real remote chain, or real DA transport.
- Graphify may be used for codebase orientation only. It is not evidence for
  the `17`-document corpus, the `28` recommendation rows, the `57` TODO
  directives, or any scenario acceptance claim.

## 📚 Docs Corpus Attachment

| Markdown source | Scenario ownership | Why it must stay attached |
| --- | --- | --- |
| `crates/z00z_simulator/README.md` | `064-S01` | Defines the canonical `scenario_1` release-packet truth lane and exact-home artifact expectations. |
| `crates/z00z_networks/onionnet/README.md` | `064-S05` | Freezes honest defer wording for OnionNet and prevents fake live-network claims. |
| `crates/z00z_runtime/aggregators/README.md` | `064-S04` | Names publication-binding and recovery semantics that must stay singular under local simulation. |
| `crates/z00z_storage/src/settlement/README.md` | `064-S04` | Defines settlement proof boundaries and the semantic-root vs backend-root split. |
| `crates/z00z_utils/README.md` | `064-S05` | Anchors one-source-of-truth infra boundary checks for file I/O, serialization, time, and RNG. |
| `crates/z00z_crypto/README.md` | `064-S05` | Anchors the workspace crypto-facade contract and vendor isolation rule. |
| `crates/z00z_extensions/README.md` | `064-S05` | Anchors the anti-dumping boundary that keeps extensions out of core semantic ownership. |
| `docs/Z00Z-Main-Whitepaper.md` | `064-S05` | Supplies the defer-language corpus for non-live chain, DA, anonymity, slashing, and fraud-engine claims. |
| `docs/tech-papers/Z00Z-Roadmap-Blueprint.md` | `064-S05` | Supplies the defer-language corpus for roadmap-level transport and live-network boundaries. |
| `wiki/03-core-protocol/genesis-caveats.md` | `064-S05` | Anchors truthful core and genesis wording after simulator and wallet closure. |
| `wiki/04-wallet-and-rpc/receiver-request-flow.md` | `064-S03` | Anchors request validation, signature, expiry, TOFU, and browser-boundary claims. |
| `wiki/04-wallet-and-rpc/wallet-object-packages.md` | `064-S02` | Anchors the live `wallet.object.*` typed-object path and rejects stale “stub” wording. |
| `wiki/04-wallet-and-rpc/wallet-object-quarantine.md` | `064-S03` | Anchors quarantine roundtrip and promotion/no-promotion behavior. |
| `wiki/04-wallet-and-rpc/wallet-stub-surface.md` | `064-S03` | Anchors honest placeholder-service ownership and non-live seam labeling. |
| `wiki/05-storage-runtime/prep-snapshot-replay.md` | `064-S04` | Anchors the adversarial snapshot matrix and deterministic replay failure lanes. |
| `wiki/06-simulator-and-quality/scenario-pipeline.md` | `064-S01`, `064-S05` | Anchors secret-clean packet rules, simulator facade boundaries, and offline-safe local doc links. |
| `wiki/06-simulator-and-quality/scenario1-object-artifacts.md` | `064-S01` | Anchors exact-home object packet artifacts and their emitted proof expectations. |

## ⚙️ Classification Summary

| Class | Meaning in Phase 064 | Representative homes | Use when |
| --- | --- | --- | --- |
| TDD / unit | Pure validator, taxonomy, packet redaction, session-gate, theorem-negative, or audit-script behavior | `crates/z00z_simulator/tests/scenario_1/test_stage2_secret_artifacts.rs`, `crates/z00z_wallets/tests/test_rpc_route_coverage.rs`, `crates/z00z_storage/tests/test_object_reject_codes.rs`, `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`, `scripts/audit_crypto_facade.sh` | One seam can be proven without a broader stateful run. |
| Integration / scenario | Multi-owner behavior across simulator, wallet, storage, runtime, rollup, or docs guards | `crates/z00z_wallets/tests/test_asset_rpc_mutations.rs`, `crates/z00z_wallets/tests/test_wallet_restore_atomic.rs`, `crates/z00z_storage/tests/test_checkpoint_store.rs`, `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs` | One result depends on a state transition across subsystem boundaries. |
| Simulator / local E2E | Real `scenario_1`, local chain simulation, or deterministic local runtime simulation | `crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs`, `crates/z00z_wallets/tests/test_chain_client_sim.rs`, `crates/z00z_rollup_node/tests/test_da_local_sim.rs`, `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs` | The behavior requires end-to-end local execution rather than a single seam. |
| Diagnostics / evidence | Grep guardrails, route audits, doc-link checks, and CI-enforced repo boundaries | `crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`, `scripts/audit_z00z_utils_boundary.sh`, `scripts/audit_extensions_boundary.sh`, `scripts/audit_local_docs_links.sh` | The proof is repo-gate output rather than one Rust assertion. |
| Skip | Planning packet files or read-only vendor subtree | `.planning/phases/064-Gaps-Closing-3/064-*.md`, `crates/z00z_crypto/tari/**` | The file is authority or vendor input, not a runtime target to modify or “test into truth.” |

## ⏰ Ordered Scenario Packet

1. `064-S01` / `PLAN-064-G01`: simulator final truth, hermetic stage closure,
   exact-home packet artifacts, secret-clean packet output, and facade imports.
2. `064-S02` / `PLAN-064-G02`: wallet-local mutation truth, typed-object path
   truth, route-audit truth, and public dispatcher coverage.
3. `064-S03` / `PLAN-064-G03`: atomic restore, sensitive RPC session gates,
   raw-builder bans, honest wasm/native boundaries, quarantine durability, and
   stable reject-code mapping.
4. `064-S04` / `PLAN-064-G04`: canonical seal path, adversarial snapshot
   matrix, theorem-boundary negatives, recovery failover, local DA simulation,
   and publication-binding anti-fork proof.
5. `064-S05` / `PLAN-064-G05`: truthful core wording, explicit defer
   boundaries, infra-boundary audit scripts, crypto-facade discipline,
   extensions anti-dumping, and local-link doc hygiene.

Every scenario validates in the same gate order:

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
2. scenario-owned narrow commands and scans
3. `cargo test --release` when Rust or test behavior changes
4. `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times, fixing all
   issues and warnings until `2` consecutive runs are clean
5. `/z00z-git-versioning` when a commit is required

All cargo validations in this packet run in release mode when cargo supports
it.

## 🔑 Required Invariants

| Invariant | How Phase 064 must prove it |
| --- | --- |
| `.planning/phases/064-Gaps-Closing-3/064-TODO.md` stays normative | Scenario ownership and proof paths remain traceable through `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md` and the grouped plan packet; no `REC-064-*` row may disappear or split silently. |
| `scenario_1` default publication becomes truthful by default | Tests must show non-null final checkpoint artifacts, final verdicts, and no `draft_only` or `incomplete` default release path. |
| Canonical stages stay hermetic | Tests and scans must prove stages `9-12` do not self-close through `step_stub` fallback coverage. |
| Default packet remains secret-clean | Packet tests must reject seed, wallet secret, receiver secret, lock bytes, or similar plaintext artifact leakage. |
| `wallet.asset.*` uses real wallet-local mutation flow | Mutation RPCs must traverse `LocalNodeSim`, `ChainClientImpl`, `BroadcastImpl`, and durable tx storage rather than `stub_default()` or fake tx ids. |
| `wallet.object.*` stays the live typed-object authority | Docs, route tests, and package-building tests must prove `wallet.object.*` stays live and must fail if it regresses to “stub” wording. |
| Sensitive wallet actions are session-gated | Every live sensitive RPC path must prove `verify_session()` or `verify_session_no_touch()` coverage. |
| Request validation cannot be bypassed by raw builders | Payment-request, chain, expiry, signature, and TOFU checks stay on the production path, while raw stealth-output builders remain out of app/RPC production flows. |
| Browser and wasm surfaces stay explicitly narrower than native | Tests and docs must reject native-only `.wlt`, TOFU, or inbox guarantees on wasm/browser surfaces. |
| Quarantine and reject-code semantics stay durable | Restore/export/import must preserve quarantined rows, and every `ObjectRejectCode` must keep stable validator and RPC mapping. |
| `seal_artifact()` stays canonical checkpoint truth | Raw-save lanes may exist for non-canonical plumbing, but tests must fail if they become public semantic truth. |
| Semantic roots stay distinct from backend proof state | `SettlementStateRoot`, backend roots, flat ids, and theorem proof payloads must stay separated by tests and docs. |
| `PublicationBinding` stays singular | Publication binding remains the only anti-fork digest and route-acceptance authority across simulator, rollup, runtime, and docs. |
| Non-live network and DA claims stay deferred | OnionNet, real remote chain, real DA transport, slashing, and fraud-engine wording must stay honest and explicitly non-live. |
| `z00z_utils` and `z00z_crypto` stay canonical facades | Boundary scripts must fail on business-crate infra drift or direct Tari vendor imports outside approved owners. |
| `z00z_extensions` stays non-semantic | Repository guards must fail if extensions start absorbing core protocol or business semantics without an explicit extension plan. |
| Internal docs citations stay local-path and offline-safe | Wiki and docs checks must reject internal GitHub blob links when a local repository path exists. |

## ✅ Scenario Matrix

| Scenario ID | Plan | Class | Primary homes | What it proves |
| --- | --- | --- | --- | --- |
| `064-S01` | `PLAN-064-G01` | Simulator / local E2E + diagnostics | simulator config, stage `9` and `12`, runtime observability, `scenario_1` tests, simulator docs | The canonical simulator lane is final, hermetic, exact-home, secret-clean, and facade-owned. |
| `064-S02` | `PLAN-064-G02` | Integration + local node-simulation | wallet asset RPCs, local chain/broadcast, dispatcher routes, RPC audit script, object-package docs/tests | Wallet mutation flows are real locally, route coverage is truthful, and `wallet.object.*` stays live. |
| `064-S03` | `PLAN-064-G03` | Integration + negative boundary tests | wallet restore/session/request/quarantine owners, wasm/native capability guard, object-reject mapping | Sensitive wallet surfaces are atomic, session-gated, honest across native vs wasm, and durable through quarantine semantics. |
| `064-S04` | `PLAN-064-G04` | Integration + deterministic local simulation | checkpoint store, snapshot store, theorem verifier, local DA adapter, recovery, publication binding | Storage, rollup, and runtime truth stay singular and fail closed on proof, lineage, route, and binding drift. |
| `064-S05` | `PLAN-064-G05` | Diagnostics / evidence + guard tests | core and wallet guard tests, repo boundary scripts, defer-boundary docs | Repository wording and imports stay truthful, deferred where required, and fail closed on boundary drift. |

## ✅ Detailed Scenario Contracts

### ✅ `064-S01` Simulator Final Truth And Packet Integrity

| Homes | Critical integration paths | Positive examples | Negative examples | Must observe | Pass conditions |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_simulator/src/config.rs`, `crates/z00z_simulator/src/config/config_accessors.rs`, `crates/z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs`, `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`, `crates/z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs`, `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`, `crates/z00z_simulator/src/lib.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_filtered_runs.rs`, `crates/z00z_simulator/tests/scenario_1/test_stage2_secret_artifacts.rs`, `wiki/06-simulator-and-quality/scenario-pipeline.md`, `wiki/06-simulator-and-quality/scenario1-object-artifacts.md` | `ScenarioCfg -> Stage12 proof mode -> finalize_stage12() -> CheckpointStore seal path -> packet export -> validator/watcher verdict`; `stage 9-12 runner -> filtered harness -> runtime observability inventory -> emitted object-flow packet`; `public facade import -> harness entrypoints` | default profile emits non-null checkpoint id, artifact, link, and audit evidence; filtered canonical stages complete without fallback closure; `asset_flow.json`, `voucher_flow.json`, and `right_flow.json` are emitted from exact homes; default packet contains no plaintext seed, private key, raw receiver secret, or lock bytes | default profile stays `draft_only` or `incomplete`; `step_stub` survives as canonical closure; exact-home anchors remain `pending_exact_home`; packet leaks secret material; harness deep-imports owner internals instead of public facades | final checkpoint ids, artifact/link/audit rows, publication state, packet inventory rows, stage-surface traces, secret-redaction boundaries | All `PLAN-064-G01` tests pass; filtered-run and packet-surface tests prove final checkpoint evidence, emitted exact-home artifact inventory, secret-clean default packets, and stable-facade imports. |

### ✅ `064-S02` Wallet Mutation Truth And RPC Route Coverage

| Homes | Critical integration paths | Positive examples | Negative examples | Must observe | Pass conditions |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/rpc/asset_rpc_server_ops.rs`, `crates/z00z_wallets/src/rpc/asset_rpc_server_catalog.rs`, `crates/z00z_wallets/src/chain/local_node_sim.rs`, `crates/z00z_wallets/src/chain/chain_client_impl.rs`, `crates/z00z_wallets/src/chain/broadcast_impl.rs`, `crates/z00z_wallets/src/persistence/tx_storage.rs`, `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`, `crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`, `crates/z00z_wallets/src/rpc/wallet_dispatcher_routes.rs`, `crates/z00z_wallets/src/rpc/wallet_dispatcher_wiring.rs`, `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs`, `crates/z00z_wallets/src/rpc/app_rpc_impl.rs`, `crates/z00z_wallets/tests/test_asset_rpc_mutations.rs`, `crates/z00z_wallets/tests/test_chain_client_sim.rs`, `crates/z00z_wallets/tests/test_chain_broadcast_retry.rs`, `crates/z00z_wallets/tests/test_rpc_route_coverage.rs`, `crates/z00z_wallets/tests/test_object_rpc_packages.rs`, `wiki/04-wallet-and-rpc/wallet-object-packages.md` | `wallet.asset.* RPC -> asset_rpc_server_ops -> LocalNodeSim/ChainClientImpl -> BroadcastImpl -> durable TxStorage`; `wallet.object.* -> wallet_dispatcher_routes -> object_rpc_impl -> object_package_contract`; `register_all_wallet_rpc_methods -> include-based route assembly -> audit script and route-coverage tests` | asset split/stake/swap/unstake/merge return real locally tracked tx handles; typed object-package build/preview routes stay live and documented as live; `app.wallet.open_wallet_source` is registered and reachable through the public dispatcher; audit script counts include-based registrations truthfully | impossible amounts, stale assets, or replayed lifecycle states are accepted; stub responses or fake tx ids survive; `wallet.object.*` docs regress to “stub”; include-based routes are undercounted; `app.wallet.open_wallet_source` remains missing | durable tx ids, local broadcast lifecycle state, route registration names, package validation roots and reject codes, public dispatcher coverage | All `PLAN-064-G02` tests and the audit script pass; grep guards show no live-path `stub_default` or `stub_tx_*`; the route surface matches the real dispatcher assembly. |

### ✅ `064-S03` Wallet Sensitive Surface And Typed-Object Durability

| Homes | Critical integration paths | Positive examples | Negative examples | Must observe | Pass conditions |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/services/wallet_actions_backup.rs`, `crates/z00z_wallets/src/services/wallet_session_manager.rs`, `crates/z00z_wallets/src/services/wallet_session_runtime_limits.rs`, `crates/z00z_wallets/src/services/wallet_store_restore.rs`, `crates/z00z_wallets/src/services/wallet_actions_receive.rs`, `crates/z00z_wallets/src/receiver/request.rs`, `crates/z00z_wallets/src/stealth/output.rs`, `crates/z00z_wallets/src/services/network_service.rs`, `crates/z00z_wallets/src/services/storage_service.rs`, `crates/z00z_wallets/src/services/key_service.rs`, `crates/z00z_wallets/src/services/backup_service.rs`, `crates/z00z_wallets/src/redb_store/owned_objects.rs`, `crates/z00z_wallets/src/redb_store/object_queries.rs`, `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`, `crates/z00z_storage/src/settlement/object_package_contract.rs`, `crates/z00z_wallets/tests/test_wallet_restore_atomic.rs`, `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs`, `crates/z00z_wallets/tests/test_payment_request.rs`, `crates/z00z_wallets/tests/test_wallet_capability_matrix.rs`, `crates/z00z_wallets/tests/test_object_quarantine.rs`, `crates/z00z_storage/tests/test_object_reject_codes.rs`, `wiki/04-wallet-and-rpc/wallet-stub-surface.md`, `wiki/04-wallet-and-rpc/receiver-request-flow.md`, `wiki/04-wallet-and-rpc/wallet-object-quarantine.md` | `restore_wallet_pack_atomic -> staged history/.wlt writes -> in-memory publication`; `sensitive RPC -> session manager -> runtime limits`; `PaymentRequest.validate_all -> receive flow -> production output builders`; `owned object quarantine -> export/import/restore -> explicit promotion or no-promotion`; `ObjectRejectCode enum -> validator contract -> RPC mapping` | torn-commit injection leaves no partial restore or split-brain local state; every sensitive RPC proves `verify_session()` or `verify_session_no_touch()`; valid request passes chain, expiry, signature, and TOFU checks; wasm/browser docs and source guards reflect narrower guarantees than native; quarantined rows survive restore/export/import and only promote when policy truth exists; every reject code has stable validator and RPC coverage | torn history-vs-pack commit leaves partial state; sensitive RPC path bypasses session gate; production app/RPC flows call raw `build_tx_stealth_output()` directly; browser surface claims native TOFU, inbox, or `.wlt` guarantees; quarantined rows disappear silently; reject-code mapping diverges across validator and RPC surfaces | atomic history-vs-pack commit behavior, session verification calls, request signatures and expiry, TOFU boundaries, wasm/native capability guard, quarantine flags, reject-code taxonomy and mapping stability | All `PLAN-064-G03` tests pass, wasm/native source-doc guards stay truthful, and grep guards prove placeholder-service wording, raw-builder usage, and browser-capability claims stay bounded and truthful. |

### ✅ `064-S04` Storage Proof Boundaries And Runtime Adversarial Closure

| Homes | Critical integration paths | Positive examples | Negative examples | Must observe | Pass conditions |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_storage/src/checkpoint/store.rs`, `crates/z00z_storage/src/snapshot/store.rs`, `crates/z00z_storage/src/settlement/README.md`, `crates/z00z_storage/src/settlement/proof.rs`, `crates/z00z_simulator/src/scenario_1/stage_4/storage_view.rs`, `crates/z00z_rollup_node/src/lib.rs`, `crates/z00z_rollup_node/src/da.rs`, `crates/z00z_runtime/aggregators/src/batch_planner.rs`, `crates/z00z_runtime/aggregators/src/recovery.rs`, `crates/z00z_runtime/aggregators/src/types.rs`, `crates/z00z_runtime/aggregators/src/dist_sim.rs`, `crates/z00z_runtime/aggregators/README.md`, `crates/z00z_storage/tests/test_checkpoint_store.rs`, `crates/z00z_storage/tests/test_prep_snapshot.rs`, `crates/z00z_storage/tests/test_settlement_proof_boundaries.rs`, `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`, `crates/z00z_rollup_node/tests/test_da_local_sim.rs`, `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`, `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`, `wiki/05-storage-runtime/prep-snapshot-replay.md` | `checkpoint save/seal -> canonical statement-bound artifact -> downstream publication input`; `PrepSnapshot validator -> duplicate and mix failure matrix`; `public theorem bundle -> rollup theorem verifier`; `LocalDaAdapter -> batch planner -> recovery -> publication binding -> route acceptance` | canonical seal path produces statement-bound checkpoint evidence; every duplicate path, duplicate terminal, witness mix, leaf mix, and root mix lane fails independently; theorem verification accepts good public bundles and rejects detached or mismatched artifacts; local DA simulation covers replay and metadata drift without claiming live transport; recovery tests cover lineage, generation, root, version, split-brain, standby, and stale-root negatives; publication binding remains one anti-fork digest | raw-save lane is accepted as canonical truth; any snapshot mix lane passes accidentally; backend roots or flat ids are exposed as public semantic truth; wrong proof payload, wrong exec id, wrong prep id, or wrong link root pass theorem verification; stale lineage, wrong generation, split-brain, standby-down, or stale-root recovery branches pass; a second binding digest or cloned route-acceptance path appears | checkpoint ids and seals, semantic roots vs backend roots, theorem proof and link ids, publication-binding digest, route-table lineage, recovery state and telemetry | All `PLAN-064-G04` tests pass; scans show one semantic-root and publication-binding story; every named adversarial lane is exercised and fail-closed on real local primitives. |

### ✅ `064-S05` Core Truth Boundaries And Repository Guardrails

| Homes | Critical integration paths | Positive examples | Negative examples | Must observe | Pass conditions |
| --- | --- | --- | --- | --- | --- |
| `wiki/03-core-protocol/genesis-caveats.md`, `docs/Z00Z-Main-Whitepaper.md`, `docs/tech-papers/Z00Z-Roadmap-Blueprint.md`, `crates/z00z_networks/onionnet/README.md`, `crates/z00z_core/src/genesis/genesis_run.rs`, `crates/z00z_core/src/vouchers/voucher_bootstrap.rs`, `crates/z00z_wallets/src/app/app_kernel.rs`, `crates/z00z_wallets/src/chain/chain_client_impl.rs`, `crates/z00z_utils/README.md`, `crates/z00z_crypto/README.md`, `crates/z00z_crypto/src/lib.rs`, `crates/z00z_extensions/README.md`, `wiki/06-simulator-and-quality/scenario-pipeline.md`, `crates/z00z_wallets/tests/test_live_boundary_claims.rs`, `crates/z00z_core/tests/test_live_guardrails.rs`, `scripts/audit_z00z_utils_boundary.sh`, `scripts/audit_crypto_facade.sh`, `scripts/audit_extensions_boundary.sh`, `scripts/audit_local_docs_links.sh`, `.github/workflows/boundary-guards.yml` | truthful docs and code wording -> live guard tests -> boundary audit scripts -> CI workflow; `z00z_utils` facade checks -> business-crate owner boundaries; workspace imports -> `z00z_crypto` facade and vendor isolation; wiki source refs -> local-path offline-safe doc links | docs and guard tests state OnionNet, remote chain, and DA as explicitly deferred; core guard tests keep simulator and wallet closure as prerequisites instead of reopening them; infra-boundary scripts pass with approved owner modules only; crypto-facade script passes with no direct vendor imports outside approved homes; extensions boundary stays narrow; internal citations use local repository paths | docs advertise live anonymity, live remote chain, or live DA behavior; core cleanup reopens already-closed simulator or wallet prerequisites; business crates widen direct `std::fs`, `serde_json`, `serde_yaml`, `SystemTime::now`, or similar infra calls outside approved owners; workspace code imports `tari_crypto::*` directly; `z00z_extensions` absorbs core semantics; internal GitHub blob links survive in local docs | defer wording, import boundaries, approved owner modules, offline-safe local doc links, CI entrypoint coverage | All `PLAN-064-G05` tests and scripts pass; grep guards show no forbidden live-claim or boundary drift; repository boundaries remain executable and fail closed under CI. |

## 🔎 Coverage Appendix

| Requirement | Scenario | Plan | Primary proof homes | What must be proven |
| --- | --- | --- | --- | --- |
| `REC-064-P0-01` | `064-S01` | `PLAN-064-G01` | `test_scenario1_stage_surface.rs`, `test_scenario_settlement.rs`, `stage_12/mod.rs`, `finalize_flow.rs` | Default `scenario_1` publication is final and truthful by default. |
| `REC-064-P0-02` | `064-S01` | `PLAN-064-G01` | `test_scenario1_filtered_runs.rs`, `bundle_lane_impl.rs`, `stage_runner_support.rs`, `fixture_cache.rs` | Canonical stages `9-12` no longer close via `step_stub`. |
| `REC-064-P0-03` | `064-S01` | `PLAN-064-G01` | `test_scenario1_object_flows.rs`, `runtime_observability.rs`, `scenario1-object-artifacts.md` | `asset_flow.json`, `voucher_flow.json`, and `right_flow.json` emit from exact homes. |
| `REC-064-P0-08` | `064-S01` | `PLAN-064-G01` | `test_stage2_secret_artifacts.rs`, `scenario-pipeline.md`, simulator README | Default release packets stay secret-clean and fail closed on plaintext leakage. |
| `REC-064-P2-07` | `064-S01` | `PLAN-064-G01` | `crates/z00z_simulator/src/lib.rs`, `test_scenario1_filtered_runs.rs`, `scenario-pipeline.md` | Simulator harness imports stay on stable public facades. |
| `REC-064-P1-01` | `064-S02` | `PLAN-064-G02` | `test_asset_rpc_mutations.rs`, `test_chain_client_sim.rs`, `test_chain_broadcast_retry.rs`, `asset_rpc_server_ops.rs` | Asset mutation RPCs use the live wallet-local chain path and durable tx state. |
| `REC-064-P1-02` | `064-S02` | `PLAN-064-G02` | `test_object_rpc_packages.rs`, `wallet_dispatcher_routes.rs`, `wallet-object-packages.md` | `wallet.object.*` stays the live typed-object path and is never described as stubbed. |
| `REC-064-P1-03` | `064-S02` | `PLAN-064-G02` | `audit_rpc_method_wiring.py`, `test_rpc_route_coverage.rs`, `app_dispatcher_wiring.rs` | Include-based route registration is audited truthfully and `app.wallet.open_wallet_source` is wired. |
| `REC-064-P2-01` | `064-S03` | `PLAN-064-G03` | `wallet-stub-surface.md`, service owners, grep guards | Placeholder services resolve to honest live owners or explicit non-live seams. |
| `REC-064-P0-04` | `064-S03` | `PLAN-064-G03` | `test_wallet_restore_atomic.rs`, `wallet_actions_backup.rs` | Restore remains atomic across staged `.wlt`, history, and in-memory publication steps. |
| `REC-064-P0-05` | `064-S03` | `PLAN-064-G03` | `test_sensitive_rpc_session.rs`, `wallet_session_manager.rs` | All sensitive RPCs prove session verification on the live path. |
| `REC-064-P0-06` | `064-S03` | `PLAN-064-G03` | `test_payment_request.rs`, `receiver/request.rs`, `stealth/output.rs` | Raw stealth-output builders stay out of production app/RPC flows. |
| `REC-064-P0-07` | `064-S03` | `PLAN-064-G03` | `test_wallet_capability_matrix.rs`, `wallet_store_restore.rs` | Browser and wasm surfaces do not claim native TOFU, inbox, or `.wlt` guarantees. |
| `REC-064-P1-04` | `064-S03` | `PLAN-064-G03` | `test_object_quarantine.rs`, `owned_objects.rs`, `wallet-object-quarantine.md` | Quarantined inventory survives restore/export/import and explicit promotion checks. |
| `REC-064-P1-05` | `064-S03` | `PLAN-064-G03` | `test_object_reject_codes.rs`, `object_package_contract.rs`, `object_rpc_impl.rs` | Every `ObjectRejectCode` has stable validator and RPC coverage. |
| `REC-064-P2-02` | `064-S04` | `PLAN-064-G04` | `test_da_local_sim.rs`, `test_recovery_failover.rs`, `da.rs`, `batch_planner.rs` | Local DA/runtime simulation expands without fake live-network claims. |
| `REC-064-P1-06` | `064-S04` | `PLAN-064-G04` | `test_checkpoint_store.rs`, `checkpoint/store.rs`, `stage_4/storage_view.rs` | Raw-save paths stay off the canonical checkpoint truth path. |
| `REC-064-P1-07` | `064-S04` | `PLAN-064-G04` | `test_prep_snapshot.rs`, `snapshot/store.rs`, `prep-snapshot-replay.md` | Every explicit `PrepSnapshot` adversarial lane fails independently. |
| `REC-064-P1-08` | `064-S04` | `PLAN-064-G04` | `test_settlement_proof_boundaries.rs`, `settlement/README.md` | Semantic roots stay separated from backend proof state and raw proof types. |
| `REC-064-P1-09` | `064-S04` | `PLAN-064-G04` | `test_rollup_theorem_guard.rs`, `rollup_node/src/lib.rs` | Detached statements, wrong proof payloads, wrong ids, and wrong link roots fail theorem verification. |
| `REC-064-P1-10` | `064-S04` | `PLAN-064-G04` | `test_recovery_failover.rs`, `recovery.rs` | Recovery failover branches fail closed under deterministic local simulation. |
| `REC-064-P1-11` | `064-S04` | `PLAN-064-G04` | `test_publication_binding.rs`, `aggregators/src/types.rs`, `aggregators/README.md` | `PublicationBinding` remains the single anti-fork digest and route authority. |
| `REC-064-P2-03` | `064-S05` | `PLAN-064-G05` | `test_live_guardrails.rs`, `genesis-caveats.md`, `genesis_run.rs` | Core/genesis cleanup stays truthful and subordinate to simulator/wallet closure. |
| `REC-064-P2-04` | `064-S05` | `PLAN-064-G05` | `test_live_boundary_claims.rs`, `Z00Z-Main-Whitepaper.md`, `Z00Z-Roadmap-Blueprint.md`, OnionNet README | Non-live network, DA, slashing, and fraud-engine surfaces remain explicitly deferred. |
| `REC-064-P2-05` | `064-S05` | `PLAN-064-G05` | `scripts/audit_z00z_utils_boundary.sh`, `.github/workflows/boundary-guards.yml`, `crates/z00z_utils/README.md` | Business crates do not quietly widen direct infra calls outside approved owners. |
| `REC-064-P2-06` | `064-S05` | `PLAN-064-G05` | `scripts/audit_crypto_facade.sh`, `crates/z00z_crypto/README.md`, `crates/z00z_crypto/src/lib.rs` | Workspace code uses `z00z_crypto` instead of direct vendor crypto paths. |
| `REC-064-P2-08` | `064-S05` | `PLAN-064-G05` | `scripts/audit_extensions_boundary.sh`, `crates/z00z_extensions/README.md` | `z00z_extensions` does not become a semantic dumping ground. |
| `REC-064-P2-09` | `064-S05` | `PLAN-064-G05` | `scripts/audit_local_docs_links.sh`, `wiki/06-simulator-and-quality/scenario-pipeline.md` | Internal citations stay local-path and offline-safe. |

## 🧪 Canonical Commands

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario_settlement -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_stage2_secret_artifacts -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_asset_rpc_mutations -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_chain_client_sim -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_chain_broadcast_retry -- --nocapture`
- `python3 crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`
- `cargo test --release -p z00z_wallets --test test_rpc_route_coverage -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_object_rpc_packages -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_payment_request -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_wallet_capability_matrix -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_object_quarantine -- --nocapture`
- `cargo test --release -p z00z_storage --test test_object_reject_codes -- --nocapture`
- `rg -n "browser builds do not get this live session model|native-only today|Rejects wasm32 and routes native load through spawn_blocking|\\.wlt persistence is not supported on wasm32|\\.wlt owned-asset loading is not supported on wasm32" wiki/04-wallet-and-rpc crates/z00z_wallets/src/services`
- `cargo test --release -p z00z_storage --test test_checkpoint_store -- --nocapture`
- `cargo test --release -p z00z_storage --test test_prep_snapshot -- --nocapture`
- `cargo test --release -p z00z_storage --test test_settlement_proof_boundaries -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_da_local_sim -- --nocapture`
- `cargo test --release -p z00z_aggregators --test test_recovery_failover -- --nocapture`
- `cargo test --release -p z00z_aggregators --test test_publication_binding -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`
- `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
- `bash scripts/audit_z00z_utils_boundary.sh`
- `bash scripts/audit_crypto_facade.sh`
- `bash scripts/audit_extensions_boundary.sh`
- `bash scripts/audit_local_docs_links.sh`
- `cargo test --release`

## 🔎 Canonical Scans

- `rg -n "open_wallet_source|wallet.object\\.|stub_default|stub_tx_" crates/z00z_wallets/src crates/z00z_wallets/tests wiki/04-wallet-and-rpc/wallet-object-packages.md`
- `rg -n "verify_session|verify_session_no_touch|build_tx_stealth_output|wasm32|placeholder|Quarantined|ObjectRejectCode" crates/z00z_wallets crates/z00z_storage wiki/04-wallet-and-rpc`
- `rg -n "save_artifact|seal_artifact|DupPath|DupTerminalId|RootMix|backend_root|SettlementStateRoot|PublicationBinding|split-brain|wrong link root" crates/z00z_storage crates/z00z_rollup_node crates/z00z_runtime/aggregators`
- `rg -n "github\\.com/z00z-labs/z00z/blob|tari_crypto::|std::fs|serde_json|serde_yaml|SystemTime::now|not shipped|placeholder|future transport" crates wiki docs scripts .github`

<verify>

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
   first. If it fails, stop, fix, and rerun before broader validation.
2. Re-run packet-consistency checks:
   `bash -lc 'test "$(rg --no-filename -o "REC-064-[A-Z0-9-]+" .planning/phases/064-Gaps-Closing-3/064-CONTEXT.md .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md | sort -u | wc -l)" -eq 28'`
   `bash -lc 'test "$(rg --no-filename -o "PLAN-064-G0[1-5]" .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md | sort -u | wc -l)" -eq 5'`
   `bash -lc 'test "$(rg --no-filename -o "064-S0[1-5]" .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md | sort -u | wc -l)" -eq 5'`
   `bash -lc 'test "$(awk '"'"'/^## 📚 Docs Corpus Attachment/{flag=1;next}/^## /{flag=0}flag && /^\| / && index($0,".md"){count++}END{print count+0}'"'"' .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md)" -eq 17'`
   `bash -lc 'test "$(awk '"'"'/^## 🔎 Coverage Appendix/{flag=1;next}/^## /{flag=0}flag && /^\| / && index($0,"REC-064-"){count++}END{print count+0}'"'"' .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md)" -eq 28'`
   `bash -lc 'for s in "Top-level TODO step 1" "Top-level TODO step 2" "Top-level TODO step 3" "Top-level TODO step 4" "Top-level TODO step 5" "Numbered closeout group \`1-5\`" "Numbered closeout group \`6-13\`" "Numbered closeout group \`14-18\`" "Graphify may orient codebase structure only" "canonical repo-relative paths"; do rg -n --fixed-strings "$s" .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md >/dev/null || { echo "missing directive carry-forward: $s"; exit 1; }; done'`
   3. Run the scenario-owned narrow tests and scans above, then run
      `cargo test --release` when Rust, tests, docs guards, simulator behavior,
      wallet behavior, or verification scripts changed.
4. Run `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times against the
   touched scenario or full Phase 064 packet. Fix all issues and warnings and
   continue until at least `2` consecutive runs show no significant issues.
5. If a commit is required after verification, use `/z00z-git-versioning`.

## ✅ Completion Criteria

- All `PLAN-064-G01` through `PLAN-064-G05` remain represented by
  `064-S01` through `064-S05`.
- All `28` `REC-064-*` requirements remain mapped to executable tests, scans,
  or boundary scripts in the coverage appendix.
- All `17` docs-corpus paths named by `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md`
  remain attached to at least one owning scenario.
- No second simulator, wallet mutation, checkpoint, theorem, publication, or
  docs authority lane is introduced by the test packet.
- The next engineer or agent can implement Phase 064 coverage without guessing
  scenario boundaries, proof homes, negative cases, or pass signals.
