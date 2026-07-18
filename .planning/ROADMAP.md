# Z00Z Roadmap

<!-- markdownlint-disable MD060 -->

This roadmap tracks the active GSD milestone and its executable phases.

## 🚧 v0.15 Storage Serialization Bootstrap

**Milestone Goal:** Bring `.planning` into canonical GSD shape and route JMT serialization work through a normal phase flow.

### Phase Summary

- [x] **Phase 015: JMT Serialization And Visualization** (completed 2026-03-23)
- [x] **Phase 016: JMT Search And Redb** (completed 2026-03-23)
- [x] **Phase 017: Scenario 1** (completed 2026-03-24)
- [x] **Phase 018: 018-A-B-C** (completed 2026-03-25)
- [x] **Phase 019: 019-gaps-1** (plan execution closed 2026-03-25; verification blocked by unrelated compile issue in `crates/z00z_wallets/src/lib.rs`)
- [x] **Phase 019.1: 019.1-rename** (all six plan summaries present 2026-03-26)
- [x] **Phase 020: Refactor Scenario 1** (completed 2026-03-26)
- [x] **Phase 021: Refactor Continue** (plan execution closed 2026-03-27; full workspace release gate blocked by unrelated read-only vendor `tari_crypto` doctest failures)
- [x] **Phase 025: Crypto Audit Crypto** (completed 2026-03-28)
- [x] **Phase 026: Crypto Audit Core** (completed 2026-03-28)
- [x] **Phase 027: Crypto Audit Utils** (completed 2026-03-29)
- [x] **Phase 028: Crypto Audit Storage** (completed 2026-03-30)
- [x] **Phase 029: Crypto Audit Wallets** (completed 2026-03-30; all six plans are summary-backed and the release-style `z00z_wallets` gate is green)
- [x] **Phase 030: Refactor Long Files** (execution reclosed 2026-04-03; all 25 plans are summary-backed, live non-test Rust residue above 400 is zero, the canonical `full_verify --max-safe-run` gate is green, and a fresh sequential bare workspace release rerun also completed green on 2026-04-04)
- [x] **Phase 031: Refactor Architecture** (completed 2026-04-04; all 10 plans are summary-backed, the `z00z_utils` boundary note and retirement evidence are written down, and the fresh bootstrap, targeted release gates, canonical max-safe run, and corroborating broad workspace release rerun are green)
- [x] **Phase 032: Crypto Audit Scenario 1** (planned 2026-04-05; 7 plans queued) (verification reopened 2026-04-06: `PH32-SPEND` remains open against the original spend-contract wording and `PH32-CLAIM-TRUST` remains open against the original persisted storage-backed continuity wording)
- [x] **Phase 033: Crypto Audit Scenario 2** (added 2026-04-06; directory pre-existing; planning complete; execution in progress with 17 of 23 plans summary-backed) (completed 2026-04-08)
- [x] **Phase 034: Mix1 Fixes** (added 2026-04-09; directory pre-existing; summary-backed complete through `034-09`; semantic closure remains rooted in `034-08`; `034-15` executed as a local non-semantic sidecar; `034-16`, `034-17`, and `034-18` completed as post-closure hygiene; repository-backed closure artifacts live under `.planning/phases/000/034-mix1-fixes/`; next canonical phase is `035`)
- [x] **Phase 035: Mix2 Fixes** (added 2026-04-09; directory pre-existing; completed 2026-04-13; all 19 plans are summary-backed, the final rename acceptance lane is closed, and the mandatory bootstrap gate reran green during the final continuity refresh)
- [x] **Phase 036: Rename** (added 2026-04-14; directory pre-existing; the canonical embedded-versioning closure on `036-a1-versioning-spec.md`, `036-TODO-2.md`, and `036-CONTEXT.md` remains summary-backed complete through `036-10`, while the separate follow-on chains have reopened the phase beyond that closeout: the legacy-removal continuation on `036-a2-legacy-removing-spec.md`, `036-TODO-3.md`, and `036-11` through `036-18` is summary-backed complete, `036-19-SUMMARY.md` now closes the self-contained a4 rename matrix through row 814, `036-20-SUMMARY.md` records a partial shim-removal follow-on rooted in `036-a4-shims-spec.md` with zero focused old-name shim hits and consecutive green bootstrap reruns but an open broad constructor tail plus live `CompatRoot` and `SeqSecureRngProvider` survivors, `036-21-SUMMARY.md` closes the self-contained attribute-audit continuation, `036-22-SUMMARY.md` closes the self-contained hash-domain continuation, `036-23-SUMMARY.md` now closes the self-contained claim-contract continuation rooted in `036-a6_claim-spec.md`, and `036-24-SUMMARY.md` closes the self-contained a7 crypto path-group continuation rooted in `036-a7_crypto-spec.md`)
- [x] **Phase 037: Output Reception** (added 2026-04-22; directory pre-existing; `037-TODO.md` remains the canonical backlog refreshed against the refactored receive codebase; numbered plan artifacts now exist as `037-01-PLAN.md` through `037-10-PLAN.md`; the numbered plan chain is now summary-backed complete through `037-10-SUMMARY.md`, the broader release suite is green, and the remaining open boundary is the partial Task 9 backlog plus pending UAT and final verification artifacts)
- [x] **Phase 040: Spend Proof** (continued 2026-04-28 on `040-10-PLAN.md`; directory pre-existing; `040-Spend-Proof-Spec.md` remains the canonical design source, `040-TODO.md` remains the canonical execution backlog, `040-CONTEXT.md` locks discuss-phase planning authority, `040-09-SUMMARY.md` remains the honest statement-bound baseline, the live wallet spend path now uses `regular_spend_theorem_bpplus` with `CanonicalSpendProofBackend`, and `040-10-PLAN.md` is now the active internal theorem-relation closure track with implementation and full workspace verify green while public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup settlement closure remain open)
- [x] **Phase 041: Renaming Fixes** (added 2026-04-29; directory pre-existing `.planning/phases/000/041-renaming-fixes/`; all 5 plans executed as self-contained waves; phase complete)
- [x] **Phase 042: Refactor Wallets** (added 2026-04-30; completed 2026-05-02; directory pre-existing; core wallet modules refactored via 042-TODO without formal plan files: `core/address`, `core/stealth`, `core/chain`, `core/key`, `services`; test compilation gate green)
- [x] **Phase 043: Gaps Fixes** (added 2026-05-06; directory pre-existing `.planning/phases/000/043-gaps-fixes/`; completed 2026-05-08 through `043-18` with additive spec-2 E2E evidence synced)
- [x] **Phase 044: Wallet Assets** (added 2026-05-09; directory pre-existing `.planning/phases/000/044-wallet-assets/`; use the existing folder only; canonical execution authority is the self-contained `044-TODO.md`; completed 2026-05-09 after the live tx-store backup/restore seam landed and the full release workspace test suite passed)
- [ ] **Phase 046: Wallet Addons** (added 2026-05-13; directory pre-existing `.planning/phases/000/046-wallet-addons/`; reuse the existing folder only; canonical execution authority is the PRD-derived `046-CONTEXT.md`; paused after `046-04` as of 2026-05-15; `046-05` pending and `046-06` paused pending rewrite for the wallet `.wlt` redesign direction)
- [x] **Phase 047: 047-wallet-redesign** (added 2026-05-15; directory pre-existing `.planning/phases/000/047-wallet-redesign/`; reuse the existing folder only; planning complete and review-hardened on `047-TODO.md`, `047-CONTEXT.md`, `047-SPEC-COVERAGE.md`, and `047-01-PLAN.md` through `047-08-PLAN.md`; completed 2026-05-22 with `047-01` through `047-11` summary-backed complete, the bounded post-closeout packet from `047-wallet-addon-spec2.md` closed, and the external simulator wording test remaining a separate workspace blocker outside the closed phase slice)
- [x] **Phase 051: HJMT Facade** (added 2026-05-28; directory pre-existing `.planning/phases/000/051-HJMT-Facade/`; reuse the existing folder only; completed 2026-05-28 with `051-01` through `051-06` summary-backed and final `051-SUMMARY.md`; shipped the storage-owned backend facade, explicit compatibility backend, root taxonomy guards, compatibility proof-envelope v1 hardening, public API guardrails, downstream semantic-authority guards, compatibility golden corpus, proof reject coverage, reload/path-index/checkpoint validation, validator checkpoint source-shape guards, storage docs, and Phase 052 readiness handoff; all Rust validation evidence used release mode)
- [x] **Phase 052: HJMT Backend** (completed 2026-05-29; directory pre-existing `.planning/phases/052-HJMT-Backend/`; `052-01` through `052-11` are summary-backed complete and final `052-SUMMARY.md` records closeout; full fixed-bucket HJMT forest implementation stays behind the Phase 051 facade, treats `CompatibilityBackend` as migration oracle only, preserves root vocabulary, proof envelope, downstream authority ownership, and checkpoint contracts, and records adaptive buckets, occupancy metadata, generalized roots, `RightLeaf`, and `FeeEnvelope` as future candidates only)
- [x] **Phase 053: HJMT Backend** (added 2026-05-29; directory pre-existing `.planning/phases/053-HJMT-Backend/`; reuse the existing folder only; completed 2026-06-05 with `053-01` through `053-20` summary-backed plus final `053-SUMMARY.md`; the repository now ships the live generalized-settlement HJMT backend, rights-enabled corpus and proof families, adaptive policy/cache/scheduler/recovery closeout, downstream integration, docs and executable examples, the production default gate, and the final purge of compatibility/simple-JMT runtime tails without creating a duplicate directory)
- [x] **Phase 054: Refactor Crates** (added 2026-06-08; directory pre-existing `.planning/phases/054-Refactor-Crates/`; reuse the existing folder only; completed 2026-06-09 with `054-01` through `054-08` summary-backed plus refreshed `054-SUMMARY.md`; the post-closeout `054-08` continuation rebound runtime planner digests at ingress, kept one canonical public planner path with no alias, shim, or bypass return path, closed `AS-20260609-001`, reran bootstrap first, reran `cargo test --release`, and refreshed the security and validation ledgers without creating a second phase folder)
- [x] **Phase 055: HJMT Boundary** (added 2026-06-09; directory pre-existing `.planning/phases/055-HJMT-boundary/`; reuse the existing folder only; completed 2026-06-10 with `055-01` through `055-04` summary-backed plus final `055-SUMMARY.md`; the repository now ships the additive storage-owned `BatchProofBlobV1` boundary, fail-closed verifier and builder, byte-authoritative fixtures, canonical settlement bench evidence, Stage 13 batch comparison/tamper evidence, and runner guardrails without creating a duplicate phase folder or a duplicate authority layer)
- [x] **Phase 056: HJMT Storage Aggregator** (added 2026-06-11; directory pre-existing `.planning/phases/056-HJMT-storage- aggregator/`; reuse the existing folder only; completed 2026-06-12 with `056-01-SUMMARY.md` through `056-07-SUMMARY.md` summary-backed; the repository now ships one canonical `SIM-5A7S` runtime home, runtime-owned planner truth, storage-owned semantic handoff and scope birth, lawful same-lineage failover, fail-closed startup preflight, simulator runtime-observability evidence, and closeout benchmark lanes inside the existing storage bench homes without creating a duplicate phase folder or a duplicate authority layer)
- [x] **Phase 057: HJMT Multi Aggregator** (added 2026-06-13; directory pre-existing `.planning/phases/057-HJMT-multi-aggregator/`; reuse the existing folder only; completed 2026-06-14 with `057-01` through `057-07` summary-backed plus final `057-07-SUMMARY.md`; the repository now ships one canonical root-of-shard-roots publication path, layered public-proof continuity, live `SIM-5A7S-PUB` publication traces, lawful join/transfer/carry-forward/recovery evidence, one shared validator/watcher publication-binding path, and the `root_of_roots_publish/*` benchmark lane inside the existing storage bench home without creating a duplicate phase folder or authority layer)
- [x] **Phase 058: HJMT Benchmarks** (added 2026-06-14; directory pre-existing `.planning/phases/058-HJMT-benchmarks/`; reuse the existing folder only; completed 2026-06-16 with `058-01` through `058-07` summary-backed plus final `058-SUMMARY.md`; the repository now ships one canonical HJMT evidence and benchmark closeout path, the shared-proof report closure is frozen, the exact bucket-commit and compatibility-equivalence artifacts are landed, `outputs/settlement/` is the only canonical measured archive home, Appendix C standalone artifacts `C-04`, `C-14`, and `C-16` are landed, and the honest repository verdict is `integrated upgrade`)
- [x] **Phase 059: Core Upgrade** (added 2026-06-16; directory pre-existing `.planning/phases/059-Core-Upgrade/`; reuse the existing folder only; completed 2026-06-18 with `059-01` through `059-10` summary-backed plus final `059-SUMMARY.md`; the repository now ships one canonical Asset/Voucher/Right object-model upgrade across `z00z_core`, `z00z_storage`, `z00z_wallets`, runtime, and the existing `scenario_1`, with additive genesis policies/rights/vouchers, one settlement-root vocabulary, typed wallet inventory and object RPC, Alice/Bob/Charlie simulator evidence, final evidence/UAT closeout, and final release/full-verify validation without creating a duplicate authority layer)
- [x] **Phase 060: Gaps Closing** (added 2026-06-19; directory pre-existing
  `.planning/phases/060-Gaps-Closing/`; completed 2026-06-23 with
  `060-01` through `060-15` summary-backed, `060-14` closed as
  review-context-only for the superseded overlap, `060-15` closing the actual
  narrowed MVP packet, the mandatory `bootstrap_tests.sh` gate green, and the
  final `cargo test --release` rerun green on the current tree including the
  long `scenario_1` broad tail; future full
  `z00z-verification-orchestrator` reruns remain operator-owned manual work)

- [x] **Phase 061: Wallet Refactoring** (added 2026-06-23; directory
  pre-existing `.planning/phases/061-Wallet-Refactoring/`; reuse the existing
  folder only; completed 2026-06-24 with `061-01` through `061-10`
  summary-backed, the final one-level wallet tree contract proven, the flat
  `crates/z00z_wallets/docs/` authority preserved for key/wallet/domain docs,
  `crates/z00z_wallets/docs/egui_views.tar.gz` kept as the last egui reference
  bundle outside `src/`, `wallet_tab_stacking` corrected to
  `wallet_tab_staking`, `app_settings_tab_2.rs` retired, the cross-crate
  `test_live_guardrails` tx-proof anchor repaired, the wallet-config env
  isolation fix landed, the simulator fixture-cache test-serialization fix
  removed the last broad-gate flake, the mandatory bootstrap gate green, and
  the final `cargo test --release` rerun green on the current tree)

- [x] **Phase 062: Gaps Closing 2** (added 2026-06-24; directory
  pre-existing `.planning/phases/062-Gaps-Closing-2/`; reuse the existing
  folder only; the canonical planning packet now exists as `062-CONTEXT.md`,
  `062-COVERAGE.md`, `062-TEST-SPEC.md`, `062-TESTS-TASKS.md`, and
  `062-01-PLAN.md` through `062-27-PLAN.md`; execution resumed on
  2026-06-24; `062-01` through `062-27`
  are summary-backed complete; `062-13` closed the canonical local
  `DaAdapter` seam, local agentic-right and machine-capability simulator
  evidence, and bounded local-only closeout docs while preserving future-only
  or target design wording as live mandatory scope; `062-12` closed direct
  wallet-store and RPC
  cash/object no-leak proofs, bounded internal object-family docs, and the
  live fee or right or voucher object baseline without reopening a second
  planning authority; `062-11` closed wallet-local privacy/reveal
  redaction, bounded verify/import/export package summaries, fail-closed
  backup metadata tamper coverage, and phase-local privacy docs without a
  transport-anonymity overclaim; `062-10` closed explicit live `ZkPack_v1`
  pack truth, fail-closed unsupported-version and unknown-lane coverage, and
  the Stage 13 localized-copy lock needed to keep the broad release rerun
  green without creating a second simulator authority plane; `062-09` closed
  digest-bound simulator wallet lifecycle evidence, restart equality over
  wallet reopen and tx-history replay, and self-healing promoted Stage 13
  cache refresh without creating a second wallet truth plane; green mandatory
  bootstrap reruns and green focused rollup-node/validator/watcher/simulator
  release validation are recorded for `062-13`; `062-14` closed the canonical
  root `GenesisConfig` manifest plus `manifest_refs` section-loader path,
  manifest schema/golden tests, approved genesis-rights snapshot realignment,
  the phase-source settlement-root heading repair, and the wallet asset RPC
  fixture-order release regressions; `062-15` closed the root
  `AssetError`/`ObjectFamily`/`ObjectRoleV1` facade, canonical shared-contract
  import cleanup, and owner-test relocation out of asset-owned include paths;
  `062-16` closed local HJMT route/proof/publication boundary wording,
  wallet/public-proof boundary strings, storage-created-scope ownership, and
  SIM-5A7S runtime-fixture manifest ownership with topology enforcement; the
  mandatory bootstrap reruns, focused validator/watcher/rollup-node/simulator
  release validation, and the final broad `cargo test --release` rerun are
  green; `062-17` closed the canonical local distributed HJMT simulator and
  consensus-adapter seams, real recovery-record replication/catch-up/quorum/
  membership/split-brain evidence, and the SIM-5A7S distributed-truth plus
  adapter-only manifest register with green focused aggregator/rollup-node and
  broad release reruns; `062-18` closed canonical local HJMT route rollout,
  shard-owner scheduler waves, owner-bound remote dispatch, storage-lock
  hazard fencing, watcher advisory runtime notes, and the SIM-5A7S
  rollout/scheduler/dispatch/observability manifest contract with green
  focused aggregator/watcher/rollup-node release reruns and a green broad
  `cargo test --release`; `062-19` closed the canonical thin signed-index DTO,
  signed snapshot/authentication model, typed helper store APIs, live
  thick-or-thin RPC parsing fallback, real-package thin release tests, and the
  Appendix C root-name/signature drift cleanup with green bootstrap and broad
  release reruns; `062-20` closed the canonical thin snapshot cache,
  pin/refresh/fallback behavior, shared thin transport builders, live helper
  expansion before runtime admission, rename-guard helper-file closure, green
  focused wallet release reruns, and a green broad `cargo test --release`;
  `062-21` closed fail-closed thin cache uncertainty, thin-versus-thick
  equivalence and fallback recovery, typed negative thin RPC errors, and
  bounded thin logging summaries with green wallet and broad release reruns;
  `062-22` closed the canonical final closeout register and residual-gap
  ledger, bounded future transport and field-native pack wording, cross-crate
  residual guardrail tests, green focused wallet and storage release reruns,
  and a green broad `cargo test --release`; `062-23` closed the canonical
  wallet `ChainClient` local node simulation path, typed local negative cases,
  and drift-free `062-23` plan wording with green focused wallet and broad
  release reruns; `062-24` closed the canonical wallet broadcast retry or
  confirmation persistence seam, drift-free `062-24` plan wording, scoped
  broadcast test-name cleanup, green focused wallet release validation, and a
  green broad `cargo test --release`; `062-25` closed the canonical wallet
  fee-rate source seam, drift-free `062-25` plan wording, scoped
  fee-estimator test-name cleanup, green focused wallet release validation,
  and a green broad `cargo test --release`; `062-26` closed the canonical
  wallet remote-worker seam, drift-free `062-26` plan wording, future-only
  touched-seam wording cleanup, scoped worker test-name cleanup, green
  focused wallet release validation, and a green broad `cargo test --release`;
  `062-27` closed the canonical wallet spend-policy seam for `TASK-125`,
  added targeted RPC policy coverage, removed stale live-scope and dead-path
  drift from the Phase 062 plan packet, cleaned touched wallet contract
  wording, removed the simulator `runner.rs` full-suite flake by serializing
  the stateful unit tests, and ended with green focused, feature-gate, and
  broad sequential release reruns; no active Phase 062 lane remains)

- [x] **Phase 063: Core Update** (added 2026-06-28; directory pre-existing
  `.planning/phases/063-Core-Update/`; reuse the existing folder only;
  completed 2026-06-29 with `063-01` through `063-13` summary-backed,
  `063-TODO.md` preserved as the canonical planning authority,
  `063-core-examples.md` preserved as a mandatory implementation source,
  future-only or target design wording in the Phase 063 corpus treated as live
  mandatory scope throughout execution, the canonical
  `crates/z00z_core/z00z_config/` root, `z00z_core::config_paths` helper
  surface, flat `crates/z00z_core/tests/` root, flat
  `crates/z00z_core/{benches,bin,examples}` roots, and truthful targeted docs
  or support-surface Markdown all landed, the non-ASCII live doc filename was
  replaced with the ASCII
  `CLAIM_WITH_CRYPTOGRAPHIC_BALANCE_VALIDATION.md`, the rustdoc invalid-HTML
  warning surface in `crates/z00z_core/src/assets/*` was closed, the final
  support-surface closeout removed nested support files and example-local YAML
  authority, made the `cli` feature boundary explicit alongside
  `autobins = false`, `autoexamples = false`, and `autobenches = false`,
  reran the mandatory bootstrap gate green on an isolated target directory,
  ended with green focused release validation, and left no active Phase 063
  lane remaining)

- [x] **Phase 064: Gaps Closing 3** (added 2026-06-29; directory
  pre-existing `.planning/phases/064-Gaps-Closing-3/`; reuse the existing
  folder only; completed 2026-06-30 with `064-01` through `064-05`
  summary-backed, `064-TODO.md` preserved as the canonical normative task
  authority, future-only or target design wording treated as live mandatory
  scope throughout execution, executable `z00z_utils` or `z00z_crypto` or
  `z00z_extensions` boundary audits plus local docs-link guards landed, and
  no active Phase 064 lane remaining)

- [x] **Phase 065: Attack Surface** (added 2026-06-30; directory pre-existing
  `.planning/phases/065-Attack-Surface/`; reuse the existing folder only;
  `065-TODO.md` remains the normative human-readable authority; the legacy
  Phase 065 reports and inventories stay retired into the single TODO; future-only
  or target design wording in the Phase 065 corpus is live mandatory scope;
  `065-01-SUMMARY.md` through `065-13-SUMMARY.md` are complete; additive
  verification-remediation packet `065-10` through `065-13` closed with
  canonical verification-dispatch repair, managed-toolchain or offline gate
  recovery, checkpoint-lineage or storage-determinism closure, persisted
  wallet-chain claim binding, explicit request or receiver-card hash-policy
  proofs, and a green broad `cargo test --release`; no active Phase 065 lane
  remains)

- [x] **Phase 066: Local Pentest Orchestration** (added 2026-07-02;
  directory pre-existing `.planning/phases/066-Strix/`; reuse the existing
  folder only; `066-TODO.md` is the normative human-readable authority for
  registration, planning, and execution scoping; future-only and target-design
  wording in the Phase 066 corpus is live mandatory scope; `066-01-PLAN.md`
  through `066-14-PLAN.md` are generated in the same folder; `066-01` and
  `066-02` and `066-03` and `066-04` and `066-05` and `066-06` and `066-07`
  and `066-08` and `066-09` and `066-10` and `066-11` and `066-12` and
  `066-13` and `066-14` are summary-backed complete on
  `066-01-SUMMARY.md`, `066-02-SUMMARY.md`, `066-03-SUMMARY.md`, and
  `066-04-SUMMARY.md`, `066-05-SUMMARY.md`, `066-06-SUMMARY.md`,
  `066-07-SUMMARY.md`, `066-08-SUMMARY.md`, `066-09-SUMMARY.md`,
  `066-10-SUMMARY.md`, `066-11-SUMMARY.md`, `066-12-SUMMARY.md`, and
  `066-13-SUMMARY.md`, and `066-14-SUMMARY.md`; completed 2026-07-03 after
  repeated green mandatory `bootstrap_tests.sh` gates, bounded DAST scope or
  artifact closure, codex-surface wiring closure, portable pack-unpack
  closure, Docker isolation closure, regression or self-test closure,
  execution-prompt closure, documentation or migration closure, and
  consecutive clean manual review passes; security verified 2026-07-03 on
  `066-SECURITY.md` with `threats_open: 0`; no active Phase 066 lane remains,
  and no duplicate directory or parallel authority path is allowed)

- [x] **Phase 067: Sharded Concensus** (added 2026-07-03; directory
  pre-existing `.planning/phases/000/067-Sharded-Concensus/`; reuse the existing
  folder only; `067-TODO.md` is the normative human-readable authority for
  registration, planning, and execution scoping; its architecture-spec and
  implementation-contract wording supersedes prior working notes; source code,
  tests, and local configuration remain ground truth; future-only and
  target-design wording in the Phase 067 corpus is live mandatory scope;
  expanded planning complete on `067-CONTEXT.md`, `067-COVERAGE.md`,
  `067-verdict.md`, and `067-01-PLAN.md` through `067-21-PLAN.md`;
  `TASK-NNN` count is zero because `067-TODO.md` and `067-verdict.md` contain
  no literal `TASK-NNN` identifiers; required groups `PHASE-0` through
  `PHASE-8` map exactly once to `067-01` through `067-09`, and
  `VERDICT-LCS-01` through `VERDICT-LCS-10` map exactly once to `067-10`
  through `067-19`, and `ADDENDUM-067-20` through `ADDENDUM-067-21` map
  exactly once to `067-20` through `067-21`;
  all `21/21` plan packets are now summary-backed complete on
  `067-01-SUMMARY.md` through `067-18-SUMMARY.md`, `067-20-SUMMARY.md`,
  `067-21-SUMMARY.md`, and the post-addendum final-rerun closeout
  `067-19-SUMMARY.md`, the
  repo-wide terminology
  cleanup and permanent guard are landed, the first-class quorum artifact
  layer and runtime-owned secondary replay verifier are landed, the
  independent `scenario_11` harness plus route-bound publication seam are now
  landed, the `scenario_11` lifecycle fault matrix plus join or rotation
  transition coverage are now landed without new external crates for the
  `067-06` semantic slice, the live adapter path now has dedicated
  quorum-certificate integration coverage, the local DA or theorem or
  validator path now enforces one subject-bound certificate gate, the
  runtime-owned signer or verifier seam plus in-memory vote transport plus
  replay-verified vote service are now landed, structured equivocation or
  payload-withholding evidence is now emitted on the local vote path, exact
  `3f+1` committee math plus `2f+1` quorum proof and the Celestia-local
  artifact path with raw blob-byte or inclusion-reference or retention or
  degraded-state verification are now landed, the manifest-driven multi-process
  devnet harness plus canonical process hold-mode contract are now landed, the
  grouped crate-private wallet debug-export hardening guard is restored,
  the canonical checkpoint contract path remains
  `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`, the workspace
  now treats `crates/z00z_extensions/` as a namespace directory rather than a
  root crate, all `067-*.md` acceptance or task or verify cargo commands are
  now release-only, the current-cycle `bootstrap_tests.sh` reruns plus
  targeted runtime or simulator or guardrail release gates are green, the live
  `z00z_rollup_node` binary plus canonical release-only manifest command
  contract are now landed, the durable consensus store plus restart recovery
  path are now landed, the deterministic replicated planner-authority path
  plus planner claim-honesty registry are now landed, the deterministic
  transport-fault scheduler plus transport evidence registry are now landed,
  the canonical local HotStuff-like backend state machine plus
  validator-binding guard are now landed, the grouped crate-private wallet
  `redb_store` debug-export hardening guard required by the broad rerun was
  restored on the current tree, the captured broad `cargo test --release`
  rerun is green on the final `067-19` or `067-21` tree, the final
  `cargo clippy --release --all-targets --all-features -- -D warnings`,
  `bash scripts/audit/audit_release_feature_guards.sh`, and
  `git diff --check` closeout gates are green, the glossary claim registry
  plus report-honesty audit path now bind governed terms to one mechanical
  claim-level contract, the dedicated
  `old_primary_restart_after_takeover` simulator row plus explicit runtime
  anti-failback tests now keep one canonical failback proof path, the exact
  final rerun artifact roots are recorded under
  `reports/phase-067/20260706T120602Z/` and
  `reports/hjmt-local-devnet/20260706T120602Z/`, the required
  `/GSD-Review-Tasks-Execution` attempts were captured with consecutive clean
  manual-review fallback after final sync, no active `067-*` lane remains,
  and Phase 046 stays paused after `046-04`)

- [x] **Phase 068: Checkpoint Contract** (added 2026-07-07; directory
  pre-existing `.planning/phases/068-Checkpoint-Contract/`; reuse the existing
  folder only; `068-TODO.md` is the normative human-readable authority for
  registration, planning, and execution scoping; source code, tests, and local
  configuration remain ground truth; future-only and target-design wording in
  the Phase 068 corpus is live mandatory scope; required config gate path is
  `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`; owner crate is
  `z00z_storage`; literal `TASK-NNN` count is zero because `068-TODO.md`
  contains no literal `TASK-NNN` identifiers; the canonical packet now
  includes `068-CONTEXT.md`, `068-COVERAGE.md`, `068-01-PLAN.md` through
  `068-16-PLAN.md`, `068-TEST-SPEC.md`, and `068-TESTS-TASKS.md`; execution
  remained in strict order, `068-01` through `068-16` are summary-backed
  closed on the current tree, `068-VERIFICATION.md` is now the final
  phase-close packet, and no active `068-*` lane remains;
  the current-cycle `bootstrap_tests.sh` gate is green again on the current
  tree,
  `bash scripts/audit/audit_068_source_truth.sh` is green,
  `bash scripts/audit/audit_release_feature_guards.sh` is green, the broad
  `cargo test --release` workspace gate is green on the current tree, the
  targeted `test_checkpoint_contract_config` and `068-14` release simulations
  are green on the current tree, a later same-day continuity rerun
  reconfirmed the broad `cargo test --release` workspace gate plus both Phase
  068 audit lanes green on that same tree, and
  repo-default config loading now routes through
  `z00z_storage::checkpoint::repo_default_path()`, the last live storage tests
  now use the same helper, `CheckpointContractConfigV1::resolve_paths(...)`
  now drives one canonical checkpoint and prep-snapshot namespace across
  storage and live Scenario 1 consumers, the new release test file
  `crates/z00z_storage/tests/test_checkpoint_contract_config.rs` now locks the
  literal contract path strings plus the resolved path family, live Phase 067
  refs are normalized to `.planning/phases/000/067-Sharded-Concensus/`, every
  `068-*` verify block now names
  `/.github/prompts/gsd-review-tasks-execution.prompt.md` explicitly as a
  canonical inline/local Codex review loop, discarded `gsd` shell-out
  attempts are routing noise only, the provider-neutral
  `CheckpointDaReferenceV1` and `CheckpointPublicationEvidenceV1` path is
  landed, `CheckpointArchiveManifestV1` is now rooted in
  `statement_core_digest`, bare locator authority and provider-leakage drift
  reject on the canonical codec path, the live Scenario 1 design packet is now
  synced away from legacy checkpoint path strings, validator and watcher
  checkpoint consumers now share one storage-owned publication-readiness
  bundle path while watcher evidence remains advisory, storage now also owns
  one canonical typed PQ audit-anchor path that binds statement or delta or
  witness or archive-manifest or Plonky3 epoch or Nova chain or PQ
  commitment digests through one cadence/helper/validation lane while live
  cadence enforcement rejects missing PQ anchors once the stage gate is
  active, storage now also owns one canonical authority-promotion stage
  machine and typed verified-backend evidence surface that keeps
  `CheckpointProofSystem::VERIFIED` reserved unless every required proof or
  verifier or codec or adapter or chain-evidence or rollback or review gate
  matches one canonical config lane, the deterministic local E2E checkpoint
  lane and the source-truth repair or claim-guardrail lane are now
  summary-backed closed on `068-14-SUMMARY.md`, `068-15-SUMMARY.md`, and
  `068-16-SUMMARY.md`; the final review loop recorded Pass 1 fixing the
  missing direct config-path coverage and Passes 2 and 3 clean; and no
  duplicate directory or parallel authority path is allowed)

### Phase 015: JMT Serialization And Visualization

**Goal:** Add deterministic JMT serialization and human-readable visualization planning inside `z00z_storage` without breaking current storage boundaries.
**Requirements**: STSER-01, STSER-02, STSER-03, STSER-04
**Depends on:** None
**Plans:** 3/3 plans complete

**Canonical refs:**

- `specs/015-z00z-storage-addons/storage-serialization-plan.md`
- `specs/015-z00z-storage-addons/doublecheck-report.md`
- `crates/z00z_storage/src/assets/README.MD`
- `crates/z00z_storage/src/snapshot/codec.rs`
- `crates/z00z_storage/src/snapshot/store.rs`

**Success Criteria**:

1. `.planning` root files exist and phase 015 is discoverable by `gsd-tools`.
2. Phase 015 has a canonical context artifact pointing to the storage serialization scope.
3. `/gsd-plan-phase 015` can proceed through standard GSD flow using roadmap and context.

Plans:

- [x] 015-01-PLAN.md — Define the storage-owned serialization contracts and error surface.
- [x] 015-02-PLAN.md — Implement deterministic build, codec, persistence, and roundtrip coverage.
- [x] 015-03-PLAN.md — Add reconstruction, visualization, and boundary documentation.

### Phase 016: JMT Search And Redb

**Goal:** Add RedB-backed durable live-state storage and storage-owned deterministic asset search inside `z00z_storage` without changing canonical root semantics.
**Requirements**: STREDB-01, STREDB-02, STREDB-03, STREDB-04
**Depends on:** Phase 15
**Plans:** 3/3 plans complete

**Canonical refs:**

- `.planning/phases/016-jmt-search-and-redb/016-CONTEXT.md`
- `crates/z00z_storage/README.md`
- `crates/z00z_storage/src/assets/README.MD`
- `crates/z00z_storage/src/assets/store.rs`
- `crates/z00z_storage/src/snapshot/store.rs`
- `crates/z00z_storage/src/checkpoint/store.rs`

**Success Criteria**:

1. `AssetStore` can durably commit live-state mutations plus required artifact blobs through one RedB-backed write boundary.
2. RedB durable load rebuilds the same canonical root and canonical `AssetPath` lookup semantics as committed state.
3. Storage-owned search APIs expose deterministic exact and ordered listing queries without turning secondary indexes into a new source of truth.

Plans:

- [x] 016-01-PLAN.md — Add the storage-owned RedB backend boundary and synchronous mutation commit seam.
- [x] 016-02-PLAN.md — Persist canonical artifact blobs and implement durable AssetStore rehydration from RedB.
- [x] 016-03-PLAN.md — Expose deterministic search APIs and pagination over canonical ordering.

### Phase 17: 017-scenario_1

**Goal:** Integrate the latest `z00z_storage` claim and regular-transfer JMT execution path into `scenario_1` with storage-backed apply ownership, reload-parity validation, and final checkpoint artifact separation.
**Requirements**: SCN1-01, SCN1-02, SCN1-03, SCN1-04, SCN1-05
**Depends on:** Phase 16
**Plans:** 5 plans

Plans:

- [x] 017-01-PLAN.md — Lock claim publication and Stage 4 transport onto storage-owned artifacts.
- [x] 017-02-PLAN.md — Strengthen the wallet witness boundary around canonical storage proofs and full path binding.
- [x] 017-03-PLAN.md — Move canonical regular-transfer apply ownership into a storage-backed adapter and refactor Stage 6 handoff.
- [x] 017-04-PLAN.md — Wire Stage 7 and Stage 8 into the scenario runner and separate final checkpoint artifacts from draft outputs.
- [x] 017-05-PLAN.md — Add reload-parity, tamper, and unified acceptance coverage for the storage-backed scenario path.

### Phase 18: 018-A-B-C

**Goal:** Close the remaining Scenario 1 proof, wallet, and final-artifact gaps so claim publication, regular transfer apply, Charlie wallet evidence, and finalized checkpoint publication can be proven on one canonical artifact path.
**Requirements**: SCN1-03, SCN1-04, SCN1-05
**Depends on:** Phase 17
**Plans:** 3/3 plans complete

Plans:

- [x] 018-01-PLAN.md — Rebase Stage 4 continuity on the full claim-backed store and emit one canonical ledger-path artifact.
- [x] 018-02-PLAN.md — Add one proof-validated JMT wallet scan path and complete the Charlie runtime evidence story after Stage 7 apply.
- [x] 018-03-PLAN.md — Finalize Stage 8 checkpoint publication, expose sealed artifact surfaces, and prove the full artifact lane end to end.

### Phase 19: 019-gaps-1

**Goal:** Close the remaining wallet-facing contract gaps by making claim replay protection storage-owned, making public receive classification report-first and migration-complete for direct callers, and converging public backup create and restore onto `WalletExportPack` with legacy V1 read compatibility, then close only after wallet and simulator validation gates are green.
**Requirements**: PH19-NULL, PH19-SCAN, PH19-BACKUP
**Depends on:** Phase 18
**Plans:** 3/3 plans complete
**Execution status:** Targeted phase gates are green; full phase verification is blocked outside phase scope by the unrelated compile failure in `crates/z00z_wallets/src/lib.rs`.

Plans:

- [x] 019-01-PLAN.md — Storage-owned nullifier transition.
- [x] 019-02-PLAN.md — Receive taxonomy hardening and caller migration.
- [x] 019-03-PLAN.md — Backup convergence with V1 compatibility and phase gates.

### Phase 020: 020-refactor-scenario-1

**Goal:** Split oversized Scenario 1 stages into an explicit 12-stage homogeneous execution map, deepen YAML synchronization around that layout, and widen scenario_1 cleanup for shared logging and path helpers while preserving the canonical artifact lane and wallet-visible behavior.
**Requirements**: SCN1-03, SCN1-04, SCN1-05
**Depends on:** Phase 19
**Plans:** 4/4 plans complete
**Directory:** `.planning/phases/020-refactor-scenario-1/`

Plans:

- [x] 020-01-PLAN.md — Introduce stages 3-4 as `claim_prepare` and `claim_publish`, and wire the new runner and YAML surface.
- [x] 020-02-PLAN.md — Introduce stages 5-6 as `tx_plan` and `tx_prepare` while preserving ledger-path, prep, and report continuity.
- [x] 020-03-PLAN.md — Introduce stages 7-10 as `transfer_receive`, `transfer_claim`, `bundle_build`, and `bundle_publish`, keep explicit Stage 6 artifact reuse available to later stages, and widen scenario_1 logging and path cleanup.
- [x] 020-04-PLAN.md — Close the phase on final YAML alignment and release-style gates for the 12-stage Scenario 1 map.

### Phase 019.1: 019.1-rename

**Goal:** Apply the existing rename audit plan across `z00z_core`, `z00z_crypto`, `z00z_simulator`, `z00z_storage`, `z00z_utils`, and `z00z_wallets` without creating a new phase directory.
**Requirements**: None
**Depends on:** Phase 20
**Plans:** 6/6 plans complete
**Directory:** `.planning/phases/019.1-rename/`

**Execution status:** All six rename plans now have summary artifacts. Crate-local verification details and any historical broader-gate classifications are recorded in the individual plan summaries.

**Canonical refs:**

- `.planning/phases/019.1-rename/RENAME-PLAN.md`
- `.planning/phases/019.1-rename/019.1-CONTEXT.md`
- `.planning/phases/019.1-rename/019.1-RESEARCH.md`
- `.planning/phases/019.1-rename/019.1-VALIDATION.md`

Plans:

- [x] 019.1-01-PLAN.md — Apply the `z00z_core` rename audit, facade updates, and test-prefix normalization.
- [x] 019.1-02-PLAN.md — Apply the `z00z_crypto` rename audit and dependency-root crypto facade cleanup.
- [x] 019.1-03-PLAN.md — Apply the `z00z_simulator` rename audit after downstream crates absorb their rename fallout.
- [x] 019.1-04-PLAN.md — Apply the `z00z_storage` rename audit, module-path updates, and checkpoint test normalization.
- [x] 019.1-05-PLAN.md — Apply the `z00z_utils` rename audit for test-only files and non-prefixed test functions.
- [x] 019.1-06-PLAN.md — Apply the `z00z_wallets` rename audit, `#[path]` updates, and integration-test prefix cleanup.

### Phase 021: 021-refactor-continue

**Goal:** Finish the Scenario 1 stage-surface refactor by introducing canonical root stage files for the split lanes, restoring `scenario_design.yaml` to a descriptive document patterned after `design_scenario_orig.yaml`, and closing on release-style regression gates without leaving alias or intermediate stage files in the Scenario 1 root.
**Requirements**: SCN1-06, SCN1-07, SCN1-08
**Depends on:** Phase 020
**Plans:** 5/5 plans complete
**Directory:** `.planning/phases/021-refactor-continue/`

**Execution status:** Targeted phase gates and the release `scenario_1` binary are green. The only remaining broader workspace blocker is the unrelated read-only vendor doctest failure in `crates/z00z_crypto/tari/crypto/` (`cargo test --release --features test-fast --features wallet_debug_dump` fails at `-p tari_crypto --doc`).

**Success Criteria**:

1. Every Scenario 1 runtime stage from 3 through 12 resolves through one explicit logical stage file or facade instead of a hidden two-stage container.
2. The Scenario 1 root contains only canonical stage files `stage_1.rs` through `stage_12.rs`; Stage 11 and 12 logic lives behind `stage_11.rs` and `stage_12.rs`, while helpers move under `stage_*_utils/`.
3. `scenario_design.yaml` is descriptive again, structurally aligned to `design_scenario_orig.yaml`, and faithful to the real runtime order of inputs, outputs, calls, actions, and checks.
4. The release-style Scenario 1 gates, the full release test suite, and the release `scenario_1` binary remain green after the split.

Plans:

- [x] 021-01-PLAN.md — Establish the explicit logical stage facade surface for the split lanes without rewiring the public YAML surface before the final sync wave.
- [x] 021-02-PLAN.md — Split the claim lane into dedicated prepare and publish files with explicit `run_claim_genesis` ownership.
- [x] 021-03-PLAN.md — Split the tx lane into dedicated `tx_plan` and `tx_prepare` files and retire the public multi-stage `stage_4.rs` surface.
- [x] 021-04-PLAN.md — Split the transfer and bundle lanes into dedicated stage files and update downstream bridge/apply tests.
- [x] 021-05-PLAN.md — Rewrite `scenario_design.yaml` into descriptive sync with `design_scenario_orig.yaml` and close on release-style gates.

### Phase 025: 025-crypto-audit-crypto

**Goal:** Bring the pre-existing `025-crypto-audit-crypto` directory into the active milestone so the fused `z00z_crypto` audit artifacts can drive standard GSD research, planning, and execution without recreating the folder.
**Requirements**: PH25-CLAIM-GATE, PH25-CLAIM-V2, PH25-SOURCE-PROOF, PH25-ZKPACK, PH25-FAILCLOSED, PH25-STEALTH-BIND
**Depends on:** Phase 021
**Plans:** 5/5 plans complete
**Directory:** `.planning/phases/025-crypto-audit-crypto/`

**Canonical refs:**

- `.planning/phases/025-crypto-audit-crypto/025-FUSION.md`
- `.planning/phases/025-crypto-audit-crypto/FUSION.audit.md`
- `crates/z00z_wallets/src/core/tx/claim_tx.rs`
- `crates/z00z_wallets/src/core/stealth/tag.rs`
- `crates/z00z_wallets/src/core/stealth/facade_zkpack.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`
- `crates/z00z_storage/src/assets/proof.rs`

**Success Criteria**:

1. Placeholder `claim_v1` and custom `aead_zkpack` no longer define the default production security boundary.
2. `claim_v2` uses real Schnorr authorization, storage-owned source proofs, and zero-root rejection.
3. Production `ZkPack`, fail-closed derivation helpers, and crypto-owned stealth binding APIs have frozen vectors and release-style regression coverage.

Plans:

- [x] 025-01-PLAN.md — Define `claim_v2` contracts and storage-owned authoritative root proof interfaces.
- [x] 025-02-PLAN.md — Remove fail-open helper behavior and narrow production `ZkPack` to the standard AEAD path.
- [x] 025-03-PLAN.md — Promote crypto-owned `tag16` and `leaf_ad` APIs and freeze range or stealth binding vectors.
- [x] 025-04-PLAN.md — Migrate wallet and simulator claim flows to authoritative `claim_v2` signatures and source proofs.
- [x] 025-05-PLAN.md — Gate legacy `claim_v1` and custom zkpack surfaces from the default production facade and close on release regression gates.

### Phase 026: 026-crypto-audit-core

**Goal:** Bring the pre-existing `026-crypto-audit-core` directory into the active milestone so the fused `z00z_core` audit artifacts can drive standard GSD research, planning, and execution without creating a duplicate phase folder.
**Requirements**: PH26-GENESIS, PH26-ASSET-ID, PH26-REGISTRY, PH26-WIRE, PH26-AUTH, PH26-NONCE-FEE
**Depends on:** Phase 025
**Plans:** 5/5 plans complete
**Directory:** `.planning/phases/026-crypto-audit-core/`

**Execution status:** Plans `026-01` through `026-05` are summary-backed, `026-VERIFICATION.md` passed, all six mapped requirements are closed, and the prior-phase storage regression gate passed during phase closeout.

**Canonical refs:**

- `.planning/phases/026-crypto-audit-core/026-FUSION.md`
- `.planning/phases/026-crypto-audit-core/FUSION.audit.md`
- `crates/z00z_core/src/genesis/validator.rs`
- `crates/z00z_core/src/genesis/genesis.rs`
- `crates/z00z_core/src/assets/definition.rs`
- `crates/z00z_core/src/assets/snapshot.rs`
- `crates/z00z_core/src/assets/wire.rs`
- `crates/z00z_core/src/assets/wire_pkg.rs`
- `crates/z00z_core/src/assets/assets.rs`
- `crates/z00z_core/src/assets/gas.rs`
- `crates/z00z_core/src/assets/nonce.rs`
- `crates/z00z_core/src/assets/amount.rs`

**Success Criteria**:

1. Protected networks fail closed when expected genesis anchors are missing, mismatched, or parsed through an unsafe fallback path.
2. Asset-definition identity and registry snapshot integrity are both derived from one canonical full-payload framing rule.
3. Untrusted wire and DTO paths reject secret-bearing or confidentiality-breaking fields and preserve or explicitly reject protocol-state flags.
4. Owner and stealth verification bind canonical asset state, and fee checks enforce the canonical native fee asset identity.
5. Nonce and amount policy fail closed on time-provider errors and supported proof-width boundaries.

Plans:

- [x] 026-01-PLAN.md — Canonical asset-definition identity and test-domain isolation.
- [x] 026-02-PLAN.md — Full-payload registry snapshot hashing and validation.
- [x] 026-03-PLAN.md — Protected-network genesis anchors and seed-policy hardening.
- [x] 026-04-PLAN.md — Untrusted wire and DTO boundary hardening.
- [x] 026-05-PLAN.md — Ownership, stealth, fee, nonce, and amount policy hardening.

### Phase 027: 027-crypto-audit-utils

**Goal:** Close the confirmed `z00z_utils` audit blocker cluster and freeze explicit contracts for secret memory, config, time, deterministic RNG, logging, file I/O, and JSON boundary policy without widening scope into new utility abstractions.
**Requirements**: PH27-MEMLOCK, PH27-CONFIG, PH27-TIME, PH27-RNG, PH27-LOGGER, PH27-IO, PH27-JSON
**Depends on:** Phase 026
**Plans:** 6/6 plans complete
**Directory:** `.planning/phases/027-crypto-audit-utils/`

**Execution status:** Plans `027-01` through `027-06` are summary-backed, `027-VERIFICATION.md` is present, all seven mapped requirements are closed, and the full workspace release gate is green.

**Canonical refs:**

- `.planning/phases/027-crypto-audit-utils/027-FUSION.md`
- `.planning/phases/027-crypto-audit-utils/027-CONTEXT.md`
- `.planning/phases/027-crypto-audit-utils/027-RESEARCH.md`
- `.planning/phases/027-crypto-audit-utils/FUSION.audit.md`
- `crates/z00z_utils/src/os_hardening.rs`
- `crates/z00z_utils/src/config/yaml.rs`
- `crates/z00z_utils/src/config/layered.rs`
- `crates/z00z_utils/src/time/traits.rs`
- `crates/z00z_utils/src/rng/traits.rs`
- `crates/z00z_utils/src/logger/mod.rs`
- `crates/z00z_utils/src/io/fs.rs`
- `crates/z00z_utils/src/codec/mod.rs`

**Success Criteria**:

1. `LockedBytes` becomes lifetime-safe at the type level and passes unit plus Miri-oriented validation.
2. YAML and layered-config loading become bounded and fail closed except for explicit missing-file fallback.
3. Security-critical time consumers are migrated off lossy wrappers and deterministic RNG is fenced to approved genesis or simulator domains.
4. File-based logging and generic write helpers document and enforce their sanitization, durability, and permission guarantees explicitly.
5. The `serde_json` boundary policy is made explicit and no longer depends on undocumented direct macro drift in `z00z_utils` logger helpers.

Plans:

- [x] 027-01-PLAN.md — Make `LockedBytes` lifetime-safe and prove the guard contract with unit plus Miri-oriented validation.
- [x] 027-02-PLAN.md — Bound YAML reads and remove implicit `LayeredConfig` fail-open behavior from the default constructor path.
- [x] 027-03-PLAN.md — Freeze the fail-closed time-provider contract and migrate verified downstream consumers off ambiguous lossy wrappers.
- [x] 027-04-PLAN.md — Fence deterministic RNG to approved reproducibility domains and replace approval-sounding trait semantics.
- [x] 027-05-PLAN.md — Harden persisted logging plus generic write semantics for sanitization, severity fidelity, and durability policy.
- [x] 027-06-PLAN.md — Record and enforce the explicit JSON boundary policy inside `z00z_utils` without a wide consumer rewrite.

### Phase 028: 028-crypto-audit-storage

**Goal:** Bring the pre-existing `028-crypto-audit-storage` directory into the active milestone so the fused `z00z_storage` audit artifacts can drive standard GSD research, planning, and execution without creating a duplicate phase folder.
**Requirements**: PH28-CHK-PROOF, PH28-EXEC-PROOF, PH28-ROOT-BIND, PH28-ID-BIND, PH28-TRUST-HOOK, PH28-NULLIFIER
**Depends on:** Phase 027
**Plans:** 5/5 plans executed
**Directory:** `.planning/phases/028-crypto-audit-storage/`

**Execution status:** Plans `028-01` through `028-05` are summary-backed, the release `scenario_1` binary is green, and the full workspace release gate passed after the binary nullifier migration and storage hardening closeout.

**Canonical refs:**

- `.planning/phases/028-crypto-audit-storage/028-FUSION.md`
- `.planning/phases/028-crypto-audit-storage/FUSION.audit.md`
- `crates/z00z_storage/src/checkpoint/artifact.rs`
- `crates/z00z_storage/src/checkpoint/draft.rs`
- `crates/z00z_storage/src/checkpoint/proof.rs`
- `crates/z00z_storage/src/snapshot/store.rs`
- `crates/z00z_storage/src/claim/nullifier.rs`

**Success Criteria**:

1. Checkpoint proof surfaces either enforce authentic proof semantics or are explicitly downgraded so artifact names match the guarantees the crate actually verifies.
2. Persisted execution inputs, proof blobs, and checkpoint links preserve root-binding and replay semantics without synthetic or under-bound proof material.
3. Nullifier ownership, verifier trust boundaries, and artifact identity framing are explicit, minimized, and covered by release-style validation.

Plans:

- [x] 028-01-PLAN.md — Make checkpoint proof semantics truthful, versioned, and explicit about external verifier trust.
- [x] 028-02-PLAN.md — Preserve real execution transcripts and stop persisting placeholder tx-proof bytes as authoritative replay artifacts.
- [x] 028-03-PLAN.md — Add explicit semantic-root and backend-root binding to storage proof blobs and fail closed on cross-root tampering.
- [x] 028-04-PLAN.md — Harden checkpoint IDs and checkpoint links with type-separated, tamper-detectable canonical binding.
- [x] 028-05-PLAN.md — Canonicalize claim-nullifier storage and close the remaining production hardening hazards with parity-safe validation.

### Phase 029: 029-crypto-audit-wallets

**Goal:** Close the confirmed `z00z_wallets` audit blocker cluster by freezing one canonical live view-key path, converging wallet and backup KDF governance onto the RedB V2 baseline, removing runtime panic paths from operator-reachable flows, moving new writes off deterministic seed salt, hardening key-manager and secret-lifecycle boundaries, and framing wallet transaction digests explicitly.
**Requirements**: PH29-RECON, PH29-VIEWKEY, PH29-KDF, PH29-BACKUP, PH29-PANIC, PH29-SEEDSALT, PH29-KEYMGR, PH29-SECRET, PH29-DIGEST, PH29-VALIDATION
**Depends on:** Phase 028
**Plans:** 6/6 plans complete
**Directory:** `.planning/phases/000/029-crypto-audit-wallets/`
**Execution status:** Plans `029-01` through `029-06` are summary-backed, `029-VERIFICATION.md` is present, all ten mapped requirements are closed, and the release-style `z00z_wallets` gate is green after the final encrypted-backup RPC regression update.

**Canonical refs:**

- `.planning/phases/000/029-crypto-audit-wallets/029-FUSION.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-CONTEXT.md`
- `.planning/phases/000/029-crypto-audit-wallets/FUSION.audit.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-TEST-SPEC.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-TESTS-TASKS.md`
- `.planning/phases/000/029-crypto-audit-wallets/storage-audit-gpt54.md`
- `.planning/phases/000/029-crypto-audit-wallets/wallets-audit-m27.md`
- `.planning/phases/000/029-crypto-audit-wallets/wallets-audit-sonet46.md`

**Success Criteria**:

1. A current-tree reconciliation matrix classifies each fused wallet finding as still open, partially closed, validation only, or stale before implementation starts.
2. Sender, scanner, spend, and rotation flows use one explicit live or historical view-key policy with regression coverage that prevents drift.
3. Wallet and backup KDF behavior becomes self-describing, V1 wallets rewrite persistently to the canonical V2 contract, and backup metadata policy is explicit.
4. Runtime wallet flows fail closed without `expect()` or `unwrap()` crashes, new writes stop relying on deterministic seed salt, and key-manager plus secret-wrapper invariants are enforced by tests.
5. Wallet tx digest framing and runtime validation surfaces are explicit, tested, and documented, and the phase closes only on release-style wallet or simulator gates.

Plans:

- [x] 029-01-PLAN.md — Reconcile fused wallet findings against the current tree and freeze the exact execution target inventory.
- [x] 029-02-PLAN.md — Canonicalize the live view-key path and lock sender, scanner, and spend flows to it.
- [x] 029-03-PLAN.md — Converge wallet and backup KDF governance onto the RedB V2 contract with persisted migration.
- [x] 029-04-PLAN.md — Replace runtime panic paths with typed failures and roll new writes off deterministic seed salt.
- [x] 029-05-PLAN.md — Close key-manager invariants and zeroizing secret-wrapper boundaries for receiver-secret persistence.
- [x] 029-06-PLAN.md — Reframe tx digests, finalize validation or metadata surfaces, and close the remaining wallet-audit docs and guards.

### Phase 030: 030-refactor-long-files

**Goal:** Refactor the current oversized Rust files into responsibility-focused facades and extracted modules so the tree becomes easier to review and maintain without changing runtime behavior.
**Requirements**: PH30-SEAMS, PH30-FACADE, PH30-PROTECTED, PH30-NORMALIZE, PH30-VERIFY, PH30-SYNC
**Depends on:** Phase 029
**Plans:** 25/25 plans executed
**Directory:** `.planning/phases/030-refactor-long-files/`

**Execution status:** Plans `030-01` through `030-25` are summary-backed. The extended continuation closeout recorded a live `TOTAL_GT400=0`, synchronized the shallow facade and planning surfaces to the final split tree, reran the canonical `full_verify --max-safe-run` gate green on 2026-04-03, and a fresh sequential bare workspace release rerun also completed green on 2026-04-04.

**Canonical refs:**

- `.planning/phases/030-refactor-long-files/030-CONTEXT.md`
- `.planning/phases/030-refactor-long-files/030-todo.md`
- `.planning/phases/030-refactor-long-files/030-length_stat.md`
- `.planning/codebase/CONVENTIONS.md`
- `.planning/codebase/STRUCTURE.md`
- `.planning/codebase/TESTING.md`
- `.planning/GSD-Workflow.md`

**Success Criteria**:

1. Oversized Rust modules split along semantically homogeneous seams instead of numeric slicing, and no resulting file or facade regresses into a mixed-concern soup.
2. Compatibility-sensitive wallet, core, and crypto surfaces keep stable caller-visible contracts during structural split waves and normalize paths only through dedicated inventory-backed subwaves.
3. Every protected seam closes with bootstrap-first validation, named targeted anchors, repeated task-execution review passes, and broader release-style validation where required.
4. Docs, rustdoc, YAML, and planning references remain synchronized with the final module and facade layout.

Plans:

- [x] 030-01-PLAN.md — Split the RedB wallet-store monolith behind a stable persistence facade and protected session boundary.
- [x] 030-02-PLAN.md — Split the `z00z_core` asset-domain monoliths while preserving wire, nonce, and registry contracts.
- [x] 030-03-PLAN.md — Split `z00z_crypto` protected owner surfaces behind the stable public crypto facade.
- [x] 030-04-PLAN.md — Split simulator lane helpers and `z00z_utils::io::fs` without changing public stage or I/O entrypoints.
- [x] 030-05-PLAN.md — Split wallet address-domain surfaces behind stable address facades.
- [x] 030-06-PLAN.md — Split wallet key and identity surfaces behind stable key and wallet facades.
- [x] 030-07-PLAN.md — Split wallet service orchestration and app-shell surfaces behind stable service facades.
- [x] 030-08-PLAN.md — Split genesis surfaces and move callers to shallow `ChainType` aliases before deep-path cleanup.
- [x] 030-09-PLAN.md — Split wallet tx and RPC surfaces behind stable transaction and RPC facades.
- [x] 030-10-PLAN.md — Normalize wallet caller-visible paths and synchronize docs, YAML, and planning refs.
- [x] 030-11-PLAN.md — Normalize core caller-visible paths and synchronize docs, rustdoc, and planning refs.
- [x] 030-12-PLAN.md — Normalize crypto and support caller-visible paths and synchronize docs, rustdoc, and planning refs.
- [x] 030-13-PLAN.md — Continue the wallet DB persistence split until the remaining RedB and store-support roots fall below the >400 residue band.
- [x] 030-14-PLAN.md — Continue the wallet service and app-shell split across store, session, action, app, and example roots still above the residue band.
- [x] 030-15-PLAN.md — Continue the wallet key and seed split across key-manager, encrypted-seed, BIP32, backup-format, and stealth-key roots.
- [x] 030-16-PLAN.md — Continue the wallet address and stealth-ownership split across address manager, scanner, request, trust, and address-type roots.
- [x] 030-17-PLAN.md — Continue the wallet-core and backup split across wallet entity, snapshot, stub, and backup importer/exporter roots.
- [x] 030-18-PLAN.md — Continue the wallet tx and security-domain split across tx verifier/state/claim/selectors plus stealth and password support roots.
- [x] 030-19-PLAN.md — Continue the wallet RPC split across remaining method, dispatcher, logging, and DTO roots.
- [x] 030-20-PLAN.md — Continue the simulator split across stage lanes, runner/config, and scenario support roots still above the residue band.
- [x] 030-21-PLAN.md — Continue the `z00z_core` split across remaining asset-domain and supporting CLI roots.
- [x] 030-22-PLAN.md — Continue the crypto split across AEAD, KDF, hash, types, backend, ECDH, and shallow-facade support roots.
- [x] 030-23-PLAN.md — Continue the storage and utils split across asset store, checkpoint, snapshot, serialization, IO, and OS-hardening roots.
- [x] 030-24-PLAN.md — Run the second-pass cross-crate normalization and live >400 residue burn-down to zero.
- [x] 030-25-PLAN.md — Reclose the extended Phase 030 with truthful planning artifacts, zero-residue proof, and final workspace verification.

### Phase 031: 031-refactor-architecture

**Goal:** Refactor the reviewed crate boundaries into explicit stable facades, documented provisional seams, and evidence-backed retirement steps so versioned or compatibility exports can be removed without losing crypto, wallet, storage, simulator, or utils safety guarantees.
**Requirements**: PH31-INV, PH31-CORE, PH31-CRYPTO, PH31-NET, PH31-WLT-SEAMS, PH31-WLT-ID, PH31-WLT-RPC, PH31-STORAGE, PH31-SIM, PH31-UTILS, PH31-CLOSEOUT
**Depends on:** Phase 30
**Plans:** 10/10 plans complete

**Execution status:** Phase 031 completed on 2026-04-04. `031-01` through `031-10` are summary-backed, the `z00z_utils` admission policy is explicit, the retirement record names the bounded surviving compatibility lanes, and the fresh bootstrap, targeted wallet and simulator release suites, canonical max-safe run, and corroborating broad workspace release rerun all completed green.

**Canonical refs:**

- `.planning/phases/031-refactor-architecture/031-CONTEXT.md`
- `.planning/phases/031-refactor-architecture/031-TODO.md`
- `.planning/phases/031-refactor-architecture/031-DISCUSSION-LOG.md`
- `.planning/phases/031-refactor-architecture/031-INVENTORY.md`
- `.planning/phases/031-refactor-architecture/031-IMPORT-GRAPH.md`
- `.planning/phases/031-refactor-architecture/031-01-PLAN.md`
- `.planning/phases/031-refactor-architecture/031-01-SUMMARY.md`
- `.planning/phases/031-refactor-architecture/031-02-PLAN.md`
- `.planning/phases/031-refactor-architecture/031-02-SUMMARY.md`
- `.planning/phases/031-refactor-architecture/031-10-PLAN.md`

**Success Criteria**:

1. Wave 0 produces explicit import-graph and caller inventories for every retirement-sensitive seam before any suffix or shim removal starts.
2. `z00z_core`, `z00z_crypto`, and `z00z_networks` end the refactor with curated stable facades and documented non-default or provisional lanes instead of broad wildcard or vendor-leaking roots.
3. `z00z_wallets` closes its service, identity, auth, and RPC boundary drift without `include!` assembly, silent wallet-identity redefinition, or unauthenticated lock mutation.
4. `z00z_storage` and `z00z_simulator` preserve truthful checkpoint or replay semantics, facade-only simulator imports, debug-only secret handling, and sandboxed output cleanup.
5. Phase closeout lands only after the `z00z_utils` boundary note exists, caller-proof-backed seam retirement is documented, and release-style validation is green.

Plans:

- [x] 031-01-PLAN.md — Produce the canonical Wave 0 seam inventory and import-graph proof for all reviewed Phase 031 crates. Closed with `031-INVENTORY.md`, `031-IMPORT-GRAPH.md`, and `031-01-SUMMARY.md`.
- [x] 031-02-PLAN.md — Narrow the `z00z_core` root facade and preserve explicit bounded wire or upstream-cap ownership.
- [x] 031-03-PLAN.md — Split the `z00z_crypto` public surface into stable versus vendor or expert lanes and gate test-only AEAD helpers.
- [x] 031-04-PLAN.md — Clarify `rpc` versus `onionnet` ownership inside `z00z_networks` without inventing a fake full network implementation.
- [x] 031-05-PLAN.md — Replace `include!`-assembled wallet service layout with explicit service-module ownership and truthful placeholder demotion.
- [x] 031-06-PLAN.md — Enforce persisted wallet identity as the source of truth and bind `lock_wallet` to an explicit authorization contract.
- [x] 031-07-PLAN.md — Keep wallet RPC DTOs and dispatcher wiring edge-owned while curating the wallet root facade.
- [x] 031-08-PLAN.md — Normalize checkpoint proof and replay semantics across storage draft, finalization, and rehydrate paths.
- [x] 031-09-PLAN.md — Keep the simulator facade-only, remove default plaintext-secret artifacts, and sandbox output-reset behavior.
- [x] 031-10-PLAN.md — Close Phase 031 with a `z00z_utils` boundary note, evidence-backed seam retirement, and synchronized planning truth.

### Phase 032: Crypto Audit Scenario 1

**Goal:** Turn the Scenario 1 crypto audit into one execution-ready remediation phase that makes claim authenticity, spend verification, checkpoint truthfulness, stealth semantics, and simulator secret hygiene honest and fail-closed instead of placeholder-backed or overstated.
**Requirements**: PH32-SEM, PH32-CLAIM-BIND, PH32-CLAIM-TRUST, PH32-SPEND, PH32-CHECKPOINT, PH32-SECRET, PH32-HONEST
**Depends on:** Phase 031
**Plans:** 7/7 plans complete
**Directory:** `.planning/phases/032-crypto-audit-scenario-1/`

**Execution status:** All seven plan summaries exist, but phase closure is reopened. The current tree does prove a fail-closed persisted spend boundary, yet it does not satisfy the broader original `PH32-SPEND` acceptance wording because the live regular-spend wire and public statement do not carry nullifier semantics. It also does not satisfy the broader original `PH32-CLAIM-TRUST` wording because the current canonical helper still re-derives claim root and proof continuity from a synthetic one-item store contract instead of persisted storage-backed membership state.

**Canonical refs:**

- `.planning/phases/032-crypto-audit-scenario-1/032-CONTEXT.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-TODO.md`
- `.planning/phases/032-crypto-audit-scenario-1/prompt.txt`
- `crates/z00z_crypto/src/claim/v2.rs`
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
- `crates/z00z_storage/src/checkpoint/build.rs`

**Success Criteria**:

1. Scenario 1 claim packages are signed over the full authenticated claim tuple, including the authoritative source root consumed by downstream verification.
2. Accepted Scenario 1 claim paths consume storage-owned source roots and proofs, not simulator-local trust roots or placeholder compatibility lanes.
3. Accepted spend and checkpoint flows no longer rely on structural or placeholder success paths such as `SpendCs`, `PassProof`, `NoSpent`, or non-empty-proof checks as if they were authoritative verification.
4. Default Scenario 1 runs do not emit plaintext secret artifacts, and any surviving debug-secret path is explicitly gated, quarantined, and labeled non-production.
5. Scenario-facing code, docs, tests, and summaries stop claiming sender ignorance, live STARK/FRI enforcement, or trustless verification beyond what the delivered verifier boundaries actually prove.

Plans:

- [x] 032-01-PLAN.md — Freeze Scenario 1 semantics and trust-language before remediation starts.
- [x] 032-02-PLAN.md — Extend `ClaimStmtV2` and wallet claim helpers so the authority signature binds the full authenticated claim tuple.
- [x] 032-03-PLAN.md — Route Scenario 1 claim production and consumption through storage-owned authoritative roots and proof checks.
- [x] 032-04-PLAN.md — Define the canonical public spend-verifier contract and make structural witness gates fail closed.
- [x] 032-05-PLAN.md — Replace placeholder spend/checkpoint acceptance lanes with authoritative verifier and spent-set enforcement.
- [x] 032-06-PLAN.md — Quarantine Scenario 1 secret artifacts, deterministic test-only entropy, and config bleed-through.
- [x] 032-07-PLAN.md — Close the phase on adversarial verification evidence and honest crypto-status documentation.

### Phase 033: Crypto Audit Scenario 2

**Goal:** Materialize a complete executable plan set for the pre-existing `033-crypto-audit-scenario-2` directory so all 65 canonical Scenario 2 audit tasks can execute under normal GSD flow without renaming task titles or widening security claims.
**Depends on:** Phase 032
**Plans:** 23/23 plans complete
**Directory:** `.planning/phases/033-crypto-audit-scenario-2/`

**Execution status:** The phase directory already existed and now contains the executable plan set. Planning is complete; execution is in progress, and `033-01` through `033-19` are now summary-backed.

Governance truth note:

- The corrected active artifact set is authoritative over older optimistic wording.
- Planning truth must defer to implementation truth whenever older artifacts drift.
- Honest reclassification remains blocked until the broader original PH32-CLAIM-TRUST and PH32-SPEND gaps are implemented and re-verified, or formally narrowed and re-approved.

Verification discipline note:

- Phase 033 closeout language is a bootstrap-first targeted-closeout contract.
- Only named reruns that actually happened may be summarized as checked.
- Verification artifacts must reject unsupported broad-suite PASS language
  unless a fresh supporting evidence bundle is produced.

- Logs and manifests stay evidentiary limiters.
- They are not semantic sources of truth.

Whole-chain status note:

- The honest whole-chain story is layered.
- Delivered bucket: live cryptography exists at the claim, stealth, scan, and current-stack spend seams.
- Partial bucket: ownership continuity, checkpoint handoff, and replay/spent gating still rely in part on wallet-local or structural boundaries.
- Not-proved bucket: full validator trustlessness, end-to-end trustless closure, and other final-proof claims remain open or out of scope.
- Review rule: do not flatten the validator leg into fully live crypto or claim full end-to-end trustless closure.

Documentation allowlist note:

- Real canonical receive and Bulletproofs+ seams exist, but they do not close the full trustless theorem.
- Alice knows `s_out`; Bob's separate `receiver_secret` remains a distinct unresolved factor.
- Package-coupled checkpoint integrity exists; authoritative publish-proof closure does not.
- Task 47 remains governed by tasks 25, 27, 63, 64, and 65 before any stronger documentation cleanup can be claimed.

**Canonical refs:**

- `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md`
- `.planning/phases/033-crypto-audit-scenario-2/033-TODO.md`

**Success Criteria**:

1. Scenario 2 audit scope is frozen into canonical context and requirement artifacts before remediation begins.
2. The reused phase directory gains an explicit executable plan set under the normal GSD flow instead of ad hoc artifact drift.
3. Roadmap, state, and phase-directory truth stay synchronized while the pre-existing folder is reused as the canonical Phase 033 surface.

Plans:

- [x] `033-01-PLAN.md` — claim tuple binding, authoritative store continuity, and reject-class precision.
- [x] `033-02-PLAN.md` — publish-bound continuity, anti-theft wording, and canonical `s_out` semantics.
- [x] `033-03-PLAN.md` — `leaf_ad_id`, request/card privacy routing, and end-to-end ownership caution.
- [x] `033-04-PLAN.md` — public spend boundary, theft windows, and proof continuity across handoff.
- [x] `033-05-PLAN.md` — open PH32 element, layered whole-chain security, and placeholder-lane closure.
- [x] `033-06-PLAN.md` — draft/final truth, injective persistence, and replay/stale-artifact resistance.
- [x] `033-07-PLAN.md` — post-scan theft resistance, default secret silence, and explicit debug-lane discipline.
- [x] `033-08-PLAN.md` — seeded RNG bounds, verification discipline, and whole-scheme honesty.
- [x] `033-09-PLAN.md` — delivered-vs-open closure, planning truth, and honest reclassification rules.
- [x] `033-10-PLAN.md` — full authenticated claim tuple, plausible package drift, and self-consistency vs authority.
- [x] `033-11-PLAN.md` — distinct claim reject paths, partial claim-trust seam, and narrower anti-theft rule.
- [x] `033-12-PLAN.md` — receiver-secret ownership gate, canonical decrypt-associated binding, and request/card route separation.
- [x] `033-13-PLAN.md` — exclusivity after scan, exact spend-boundary scope, and semantic-acceptance gating.
- [x] `033-14-PLAN.md` — missing spend-statement element, checkpoint continuity, and operator-boundary protection.
- [x] `033-15-PLAN.md` — replay closure, default secret export, deterministic randomness, and honest status language.
- [x] `033-16-PLAN.md` — documentation allowlist plus stealth/range, `s_out`, and publish-proof caution answers.
- [x] `033-17-PLAN.md` — validator trust, publish trustlessness, and full-ZK spend caution answers.
- [x] `033-18-PLAN.md` — genesis membership, checkpoint placeholder boundary, and receiver identity-binding fix scope.
- [x] `033-19-PLAN.md` — request-bound privacy, real claim authority, and genesis-membership fix sets.
- [x] `033-20-PLAN.md` — checkpoint integrity, secret lifecycle, and RNG/config consolidation fix sets.
- [x] `033-21-PLAN.md` — high-severity task 63: synthetic claim-source continuity.
- [x] `033-22-PLAN.md` — high-severity task 64: audit-row-preserving nullifier-semantics spend remediation.
- [x] `033-23-PLAN.md` — high-severity task 65: audit-row-preserving authoritative checkpoint-backend remediation.

### Phase 034: Mix1 Fixes

**Goal:** Close the first mixed follow-up fix bundle from the pre-existing `034-mix1-fixes` directory without creating a duplicate phase folder.
**Requirements**: [PH34-CLAIM-CONTINUITY, PH34-SPEND-NULLIFIER, PH34-SENDER-AUTHORITY, PH34-CHECKPOINT-BACKEND, PH34-DOC-ALLOWLIST, PH34-CLOSURE-PROOF, PH34-KEEP-PATH-SIDECAR, PH34-ID-SIGNATURE-HYGIENE, PH34-SUFFIX-COLLAPSE]
**Depends on:** Phase 033
**Plans:** 9 plans
**Directory:** `.planning/phases/034-mix1-fixes/`

**Execution status:** Phase 034 is summary-backed complete through `034-09`. `034-01` closed the persisted claim-source contract seam, `034-02` closed the regular-spend nullifier contract integration, `034-03` through `034-05` remain summary-backed with `034-05` carrying the sender-authority retirement artifact, `034-06` closed the checkpoint-backend contract plus integration chain, `034-07` closed the spend and checkpoint semantic validation waves, `034-08` produced the repository-backed closure proof package for Q63, Q64, Q65, and Q47, and `034-09` completed the post-closure hygiene chain for `034-16`, `034-17`, and `034-18` while `034-15` was executed separately as a local non-semantic sidecar. The canonical Phase 034 closure artifacts live under `.planning/phases/000/034-mix1-fixes/`, and the next canonical phase is `035`.

Plans:

- [x] 034-01-PLAN.md — `034-01` and `034-02` claim continuity seam migration
- [x] 034-02-PLAN.md — `034-03` and `034-04` regular-spend nullifier contract integration
- [x] 034-03-PLAN.md — `034-05` legacy sender-construction authority retirement
- [x] 034-04-PLAN.md — `034-06` and `034-07` authoritative checkpoint backend contract plus integration
- [x] 034-05-PLAN.md — `034-08` harness lock-in and `034-09` claim continuity validation wave
- [x] 034-06-PLAN.md — `034-10` spend validation wave and `034-11` checkpoint validation wave
- [x] 034-07-PLAN.md — `034-12` allowlist reclassification and `034-13` wording-guard validation wave
- [x] 034-08-PLAN.md — `034-14` closure proof sweep and optional `034-15` keep-path sidecar
- [x] 034-09-PLAN.md — optional `034-16` identifier hygiene sidecar, `034-17` legacy collision retirement, and `034-18` production-current suffix collapse sidecars

### Phase 035: Mix2 Fixes

**Goal:** Use the pre-existing `035-mix2-fixes` directory as the canonical next phase surface for the second mixed follow-up fix bundle without creating a duplicate phase folder.
**Requirements**: PH35-DEF, PH35-SFX, PH35-GBG, PH35-SND, PH35-STL, PH35-RNM
**Depends on:** Phase 034
**Plans:** 19/19 plans complete
**Directory:** `.planning/phases/035-mix2-fixes/`

**Execution status:** Phase 035 is complete. `035-01-PLAN.md` through `035-19-PLAN.md` are closed on repository-backed summary artifacts, the final rename acceptance lane is captured by `035-19-SUMMARY.md` and `035-19-REVIEW.md`, and the mandatory bootstrap gate reran green during the final continuity refresh.

Plans:

- [x] 035-01-PLAN.md - Freeze deferred-intake authority and bind the live Phase 035 source set
- [x] 035-02-PLAN.md - Lock historical deferred triage, sidecar gating, and closeout honesty rules
- [x] 035-03-PLAN.md - Execute the deferred-consistency validation wave and optional sidecar gate
- [x] 035-04-PLAN.md - Freeze suffix authority and declaration-backed inventory scope
- [x] 035-05-PLAN.md - Land production-head cleanup targeting, filename hygiene, and curated suffix handoff
- [x] 035-06-PLAN.md - Execute suffix validation and cleanup-readiness closure
- [x] 035-07-PLAN.md - Freeze garbage classification and remove only hard-garbage rows
- [x] 035-08-PLAN.md - Review debug-dump retirement, compatibility keep-set boundaries, and current-path drift
- [x] 035-09-PLAN.md - Execute garbage-filter validation and current-path closure
- [x] 035-10-PLAN.md - Freeze the sender seam and land the validated card-only entrypoint
- [x] 035-11-PLAN.md - Converge legacy sender adapters onto the canonical stealth construction seam
- [x] 035-12-PLAN.md - Run downstream sender regressions and documentation correction
- [x] 035-13-PLAN.md - Execute sender validation and acceptance gates
- [x] 035-14-PLAN.md - Freeze stealth scope and land receiver-secret narrowing on the live seam
- [x] 035-15-PLAN.md - Freeze derivation vectors and define the V2 memo contract
- [x] 035-16-PLAN.md - Enable the V2 memo receive path and close the stealth validation lane
- [x] 035-17-PLAN.md - Freeze rename scope and execute test/support file rename wave A
- [x] 035-18-PLAN.md - Execute wallet DB and egui file renames plus mirror-path rewrites
- [x] 035-19-PLAN.md - Finish declaration renames, reference sweeps, validation, and acceptance closure

### Phase 036: Rename

**Goal:** Use the pre-existing `036-rename` directory as the canonical embedded-versioning phase surface for the `036-a1-versioning-spec.md` and `036-TODO-2.md` authority chain, with fixed serial raw-row execution across Steps 0 through 5 plus closure validation and no duplicate phase folder.
**Requirements**: None
**Depends on:** Phase 035
**Plans:** 24 plans
**Directory:** `.planning/phases/036-rename/`

**Execution status:** The phase directory already existed before roadmap insertion. Historical plan artifacts `036-01-PLAN.md` through `036-03-PLAN.md` remain summary-backed records of the earlier TODO1 or suffix-spec planning lineage, but the live Phase 036 authority chain was reopened on 2026-04-16 to `036-a1-versioning-spec.md`, `036-TODO-2.md`, and `036-CONTEXT.md`. The canonical embedded-versioning execution set was the fixed serial continuation `036-04-PLAN.md` through `036-10-PLAN.md`, mapped one-to-one onto the live `036-TODO-2.md` tasks. That chain remains summary-backed complete: the freeze-slice closeout is recorded in `036-04-SUMMARY.md`, the Step 1 compatibility-hold closeout is recorded in `036-05-SUMMARY.md`, the Step 2 rename-ready closeout is recorded in `036-06-SUMMARY.md`, the Step 3 single-version identifier closeout is recorded in `036-07-SUMMARY.md`, the Step 4 outward-contract hold closeout is recorded in `036-08-SUMMARY.md`, the Step 5 local and test cleanup closeout is recorded in `036-09-SUMMARY.md`, and the final row-coverage validation closeout is recorded in `036-10-SUMMARY.md`. On 2026-04-17 a separate follow-on legacy-removal continuation was planned on the `036-a2-legacy-removing-spec.md` -> `036-TODO-3.md` authority chain via `036-11-PLAN.md` through `036-18-PLAN.md`. That continuation is summary-backed complete through `036-18`: `036-13-SUMMARY.md` records truthful blocker carry-forward for the earlier storage subset, `036-14-SUMMARY.md`, `036-15-SUMMARY.md`, and `036-16-SUMMARY.md` record the repaired Wave 3, Wave 4, and Wave 6 closure slices, and `036-17-SUMMARY.md` and `036-18-SUMMARY.md` record the V3 follow-on cleanup, the a3 rename sweep, the appended zero-versioning closure note, and the clean residual scans. `036-11-SUMMARY.md` and `036-12-SUMMARY.md` remain absent historical continuity artifacts. The phase then reopened again on the `036-a4-renames-spec.md` and `036-a4-shims-spec.md` sidecars: `036-19-PLAN.md` is the self-contained a4 rename wave whose authority rows are summary-backed through row 814 in `036-19-SUMMARY.md`, and `036-20-PLAN.md` is the self-contained shim-removal continuation whose partial truth is preserved in `036-20-SUMMARY.md`. On 2026-04-21 the phase gained a fifth self-contained continuation: `036-21-PLAN.md` is rooted in `036-attrib-spec.md` and embeds the full attribute-audit recommendation tables, and `036-21-SUMMARY.md` now closes that matrix truthfully without reducing the still-partial `036-20` closeout. The same day the phase gained a sixth self-contained continuation: `036-22-PLAN.md` is rooted in `036-a5_hashdomain-spec.md` and embeds the full 27-row hash-domain normalization ledger for non-Tari `hash_domain!` declarations and adjacent KDF salts. `036-22-SUMMARY.md` now records that continuation as summary-backed complete, and it does not supersede the still-partial `036-20` boundary. The seventh self-contained continuation is now summary-backed complete too: `036-23-PLAN.md` is rooted in `036-a6_claim-spec.md`, `036-23-SUMMARY.md` records the full 21-row claim-contract/root-version sweep plus the live coverage-proof table, and that closeout still does not supersede the separate partial `036-20` boundary.

**Canonical refs:**

- `.planning/phases/036-rename/036-a1-versioning-spec.md`
- `.planning/phases/036-rename/036-TODO-2.md`
- `.planning/phases/036-rename/036-CONTEXT.md`
- `.planning/phases/036-rename/036-a2-legacy-removing-spec.md`
- `.planning/phases/036-rename/036-TODO-3.md`
- `.planning/phases/036-rename/036-04-PLAN.md`
- `.planning/phases/036-rename/036-05-PLAN.md`
- `.planning/phases/036-rename/036-06-PLAN.md`
- `.planning/phases/036-rename/036-07-PLAN.md`
- `.planning/phases/036-rename/036-08-PLAN.md`
- `.planning/phases/036-rename/036-09-PLAN.md`
- `.planning/phases/036-rename/036-10-PLAN.md`
- `.planning/phases/036-rename/036-11-PLAN.md`
- `.planning/phases/036-rename/036-12-PLAN.md`
- `.planning/phases/036-rename/036-13-PLAN.md`
- `.planning/phases/036-rename/036-14-PLAN.md`
- `.planning/phases/036-rename/036-15-PLAN.md`
- `.planning/phases/036-rename/036-a4-renames-spec.md`
- `.planning/phases/036-rename/036-19-PLAN.md`
- `.planning/phases/036-rename/036-a4-shims-spec.md`
- `.planning/phases/036-rename/036-20-PLAN.md`
- `.planning/phases/036-rename/036-attrib-spec.md`
- `.planning/phases/036-rename/036-21-PLAN.md`
- `.planning/phases/036-rename/036-a5_hashdomain-spec.md`
- `.planning/phases/036-rename/036-22-PLAN.md`
- `.planning/phases/036-rename/036-a6_claim-spec.md`
- `.planning/phases/036-rename/036-23-PLAN.md`
- `.planning/phases/036-rename/036-a7_crypto-spec.md`
- `.planning/phases/036-rename/036-24-PLAN.md`

**Success Criteria**:

1. Phase 036 is discoverable in the active milestone without creating a second directory.
2. The roadmap points to the current versioning-spec or TODO2 authority chain instead of the historical TODO1 or suffix-spec chain.
3. `036-04-PLAN.md` through `036-10-PLAN.md` cover all seven canonical `036-TODO-2.md` tasks without renaming task headings or excluding owned row sets.
4. The reopened execution set remains fixed serial from `036-04-PLAN.md` to `036-10-PLAN.md`, with the raw inventory as the only task-generation surface.
5. Closure at `036-10-PLAN.md` can prove there is no uncovered phase-owned version-bearing signature left in scope.
6. The follow-on legacy-removal continuation remains explicitly separate from the closed `036-TODO-2.md` authority chain and executes only through the `036-a2-legacy-removing-spec.md` -> `036-TODO-3.md` -> `036-11` through `036-16` artifact set.
7. The a4 follow-on waves remain self-contained: `036-19-PLAN.md` owns the embedded rename matrix, and `036-20-PLAN.md` owns the embedded shim-removal matrix plus any explicit survivor disclosures.
8. The attribute-remediation continuation remains self-contained too: `036-21-PLAN.md` owns the embedded attribute-audit tables from `036-attrib-spec.md`, including explicit KEEP AS-IS rows, and may not over-claim closure for the still-partial `036-20` shim-removal lane.
9. The hash-domain continuation remains self-contained too: `036-22-PLAN.md` owns the embedded 27-row normalization ledger from `036-a5_hashdomain-spec.md`, `036-22-SUMMARY.md` records the executed owner/runtime/golden sweep, and neither artifact may use broad `Z00Z/` scan noise to over-claim wider Phase 036 closure.
10. The claim continuation remains self-contained too: `036-23-PLAN.md` owns the embedded claim rename recommendation tables from `036-a6_claim-spec.md`, including the coverage-proof table, and it may not over-claim closure for the still-partial `036-20` shim-removal lane or silently preserve `ClaimRootVer`/`claim_v2` residue in live `crates/**/*.rs`.
11. The a7 path-group continuation remains self-contained too: `036-24-SUMMARY.md` now records the remaining protocol, vendor, AEAD, hash, KDF, and backend rehome work from `036-a7_crypto-spec.md`, while `/src` and `/src/claim` remain baseline-aligned and `036-20-SUMMARY.md` still remains the separate partial boundary.

Plans:

- [x] 036-01-PLAN.md - Historical TODO1-based Phase 036 plan artifact.
- [x] 036-02-PLAN.md - Historical TODO1-based Phase 036 plan artifact.
- [x] 036-03-PLAN.md - Historical TODO1-based Phase 036 plan artifact.
- [x] 036-04-PLAN.md - `036-01 Freeze Explicit Wire Discriminants, Live Lanes, And Literal Contracts`.
- [x] 036-05-PLAN.md - `036-02 Preserve Compatibility Shims And Compatibility Read-Import Lanes`.
- [x] 036-06-PLAN.md - `036-03 Rename Internal Wiring And Diagnostic Noise`.
- [x] 036-07-PLAN.md - `036-04 Rename Current Single-Version Internal And Persisted Identifiers Without Changing Encoded Values`.
- [x] 036-08-PLAN.md - `036-05 Hold Paired Legacy/Public Outward Contracts`.
- [x] 036-09-PLAN.md - `036-06 Clean Local And Test-Only Residue After Production Naming Stabilizes`.
- [x] 036-10-PLAN.md - `036-07 Row-Coverage Validation And Regression Closure`.
- [x] 036-11-PLAN.md - `036-11 Freeze Legacy Owners Into Explicit Delete Dispositions`.
- [x] 036-12-PLAN.md - `036-12 Confirm Proof Burden For Frozen Production Groups`.
- [x] 036-13-PLAN.md - `036-13 Delete Authorized Production Legacy Code` (truth-restored; `036-13-SUMMARY.md` records explicit blocker carry-forward for the surviving storage compatibility lanes rather than a false-closeout).
- [x] 036-14-PLAN.md - `036-14 Delete Deferred Fixtures, Reject Tests, And Local Residue` (`036-14-SUMMARY.md` records the repaired Wave 3 delete-first cleanup and zero-hit substring scan).
- [x] 036-15-PLAN.md - `036-15 Run Validation Closure And Residual Scan Proof` (`036-15-SUMMARY.md` records the green deterministic reruns, zero-hit substring scan, and live-pointer advance to `036-16`).
- [x] 036-16-PLAN.md - `036-16 Remove Non-Tari #[allow(dead_code)] Escape Hatches` (`036-16-SUMMARY.md` records the non-Tari dead_code sweep closeout and the zero-hit substring scan outside Tari).
- [x] 036-17-PLAN.md - `036-17 Remove Remaining Versioned Signatures And Symbols From The V3 Inventory` (`036-17-SUMMARY.md` records the V3 follow-on cleanup, the appended zero-versioning closure note, and the clean residual scan).
- [x] 036-18-PLAN.md - `036-18 Rename The A3 Old-Name Symbols To Their Suggested-Name Targets` (`036-18-SUMMARY.md` records the a3 rename sweep and the clean residual scan proof).
- [x] 036-19-PLAN.md - `036-19 Rename The A4 Old-Name Symbols To Their Suggested-Name Targets` (`036-19-SUMMARY.md` records the truthful a4 rename-matrix closeout through row 814).
- [ ] 036-20-PLAN.md - `036-20 Remove The A4 Compatibility Shims Through Their Canonical Owners` (self-contained shim-removal continuation rooted in `036-a4-shims-spec.md`).
- [x] 036-21-PLAN.md - `036-21 Reconcile The Phase 036 Attribute Audit Recommendations Without Semantic Drift` (`036-21-SUMMARY.md` records the full attribute-matrix closeout, the intentional NARROW residuals, the explicit KEEP AS-IS row for `definition.rs::new()`, and the fact that `036-20` still remains the separate partial shim-removal boundary).
- [x] 036-22-PLAN.md - `036-22 Canonicalize The Non-Tari Hash-Domain And KDF Salt Surface Without Semantic Drift` (`036-22-SUMMARY.md` records the completed 27-row owner/runtime/golden sweep rooted in `036-a5_hashdomain-spec.md` while preserving the still-partial `036-20` boundary).
- [x] 036-23-PLAN.md - `036-23 Rename The Claim Contract Surface And Retire ClaimRootVer Without Semantic Drift` (`036-23-SUMMARY.md` records the completed self-contained claim-contract continuation rooted in `036-a6_claim-spec.md`, including the full claim rename matrix, the clean exact-word old-name residue scans, and the fact that `036-20` still remains the separate partial boundary).
- [x] 036-24-PLAN.md - `036-24 Rehome The Remaining Crypto Module Families Into Canonical Path Groups Without Changing Public Behavior` (self-contained a7 path-group continuation rooted in `036-a7_crypto-spec.md`).

### Phase 037: Output Reception

**Goal:** Use the pre-existing `037-output-reception` directory as the canonical next phase surface for request-aware output reception, ownership-detection truth, persistence gating, and public receive boundary cleanup without creating a duplicate phase folder.
**Requirements**: None
**Depends on:** Phase 036
**Plans:** 10 plans
**Directory:** `.planning/phases/037-output-reception/`

**Execution status:** The phase directory already existed before roadmap insertion. `037-TODO.md` remains the canonical execution backlog, `037-CONTEXT.md` locks the sequential no-skip planning authority chain, and numbered `037-01-PLAN.md` through `037-10-PLAN.md` serialize the execution order. The numbered plan chain is now summary-backed complete through `037-10-SUMMARY.md`; the broader release suite `cargo test --release --features test-fast --features wallet_debug_dump` is green; and the remaining open boundary is the partial Task 9 backlog recorded in `037-TEST-EXECUTION-SUMMARY.md` plus pending `037-UAT.md` and the still-missing `037-VERIFICATION.md` closeout artifact.

**Canonical refs:**

- `.planning/phases/037-output-reception/037-TODO.md`
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs`
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs`
- `crates/z00z_wallets/src/core/chain/scan_engine_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`

**Success Criteria**:

1. Phase 037 is discoverable in the active milestone without creating a second directory.
2. The pre-existing `037-TODO.md` backlog is treated as the canonical planning surface until numbered plans are generated.
3. Phase 037 activates only after an explicit handoff grounded in the live receive surface and summary-backed execution evidence.
4. Phase 037 planning truth stays aligned to the refactored request-aware receive surfaces instead of stale pre-refactor scanner, trait-stack, or RPC assumptions.

Plans:

- [x] `037-01-PLAN.md` — freeze the implemented baseline and make `recv_range(...)` canonical
- [x] `037-02-PLAN.md` — materialize explicit `Tag16Context` requirements and keep proofs downstream
- [ ] `037-03-PLAN.md` — preserve `ReceiveNext::PersistClaim` gating and rebase architecture docs
- [ ] `037-04-PLAN.md` — add inbox-assisted receive at the service boundary and de-scope `ScanEngineImpl`
- [ ] `037-05-PLAN.md` — keep `OptimizedScanner` optional and add only gap-driven tests
- [ ] `037-06-PLAN.md` — enforce receive guardrails and keep one canonical persistence target
- [ ] `037-07-PLAN.md` — rebase reception API and scanner-config language onto live seams
- [ ] `037-08-PLAN.md` — quarantine stale ECC names and make request ordering deterministic
- [ ] `037-09-PLAN.md` — keep observability actionable-only and progress on one live contract
- [ ] `037-10-PLAN.md` — align the public RPC receive seam and quarantine orphan duplicate files

### Phase 040: Spend Proof

**Goal:** Close Phase 040's internal theorem-relation path first, then keep public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup settlement closure explicit until those stronger seams are actually implemented.
**Requirements**: None
**Depends on:** Phase 037
**Plans:** 10 plans (9 summary-backed through `040-09-SUMMARY.md`; `040-10-PLAN.md` implementation and full workspace verify are green, with summary or handoff artifact still separate)
**Directory:** `.planning/phases/040-spend-proof/`

**Execution status:** The phase directory already existed before roadmap insertion. `040-Spend-Proof-Spec.md` remains the canonical design source, `040-TODO.md` remains the canonical execution backlog and task order, and `040-CONTEXT.md` still locks the discuss-phase guardrails for sequential no-drift planning. `040-01-SUMMARY.md` through `040-09-SUMMARY.md` are now the completed numbered baseline, with `040-09-SUMMARY.md` preserving the honest statement-bound closeout truth. `040-10-PLAN.md` is now the active numbered continuation for internal theorem-relation closure. Live wallet code now uses `regular_spend_theorem_bpplus` and `CanonicalSpendProofBackend`; membership witnesses are carried in `SpendProofWitness` and checked against `prev_root` inside the backend prove path. The wallet typed-root unit test now uses the matching membership root, and the canonical full workspace verify gate is green for the internal relation. The public verifier still validates a deterministic canonical artifact rather than a cryptographic proof of witness knowledge, so public/trustless theorem closure, checkpoint theorem finality, and rollup settlement closure remain open.

**Canonical refs:**

- `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md`
- `.planning/phases/040-spend-proof/040-10-PLAN.md`
- `.planning/phases/040-spend-proof/040-TODO.md`
- `.planning/phases/040-spend-proof/040-CONTEXT.md`
- `.planning/phases/040-spend-proof/040-INTEGRITY-GATES.md`
- `.planning/phases/040-spend-proof/040-CLOSEOUT-GATES.md`
- `.planning/phases/040-spend-proof/040-07-SUMMARY.md`
- `.planning/phases/040-spend-proof/040-09-SUMMARY.md`
- `.planning/phases/040-spend-proof/040-VALIDATION.md`
- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/prover.rs`
- `crates/z00z_wallets/src/core/tx/spend_rules.rs`
- `crates/z00z_wallets/src/core/tx/state_update.rs`
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs`

**Success Criteria**:

1. Phase 040 is discoverable in the active milestone without creating a second directory.
2. `040-Spend-Proof-Spec.md` stays the canonical design authority and `040-TODO.md` stays the canonical execution-order authority.
3. Phase 040 planning still reuses the existing spend and checkpoint pipeline instead of introducing a parallel proof layer or duplicate execution surface.
4. `040-CONTEXT.md` freezes the discuss-phase authority chain while the numbered plan chain preserves exact task wording and sequential order.
5. `040-09-SUMMARY.md` remains the honest statement-bound baseline and must not be reinterpreted as a theorem-closeout artifact.
6. `040-10-PLAN.md` now tracks internal theorem-relation closure at `regular_spend_theorem_bpplus` while keeping public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup settlement closure out of completed language.
7. The canonical full workspace verify gate is green for the internal theorem-relation implementation.

Plans:

- [x] 040-01-PLAN.md — Establish the versioned spend-proof carrier and canonical spend statement in the live wallet tx lane.
- [x] 040-02-PLAN.md — Wire the producer and verifier paths through the existing wallet, Stage-4, and checkpoint seams.
- [x] 040-03-PLAN.md — Close nullifier replay enforcement and make the full regular-package verifier mandatory.
- [x] 040-04-PLAN.md — Prove Stage-4 through Stage-11 roundtrip continuity and then evaluate the bounded output-builder follow-up.
- [x] 040-05-PLAN.md — Preserve theorem shape, public-input surface, and digest-root discipline with explicit integrity gates.
- [x] 040-06-PLAN.md — Close checkpoint-pipeline reuse, missing-code outcomes, and shortcut-governance gates before phase closeout.
- [x] 040-07-PLAN.md — Close the remaining theorem gap with a real theorem-carrying proof backend while keeping the current public carrier and checkpoint seam.
- [x] 040-08-PLAN.md — Close theorem-closeout truth alignment and final gate-tail cleanup without widening the public carrier or checkpoint architecture.
- [x] 040-09-PLAN.md — Freeze the honest continuation path, keep the live suite truthful, close the remaining live public-verifier negative matrix, and hand off residual proof gaps explicitly; closed on `040-09-SUMMARY.md`.
- [ ] 040-10-PLAN.md — Continue the phase for internal theorem-relation closure, freezing one theorem suite and one backend relation target while keeping public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup settlement closure explicit until those stronger seams land; implementation and full workspace verify are green, summary or handoff remains separate.

### Phase 041: Renaming Fixes

**Goal:** Apply verifier- and test-adjacent naming fixes from audit inputs without changing runtime behavior, preserving threat-model boundaries and trait-locked/public-contract constraints.
**Requirements**: RENAME-041-01, RENAME-041-02, RENAME-041-03, RENAME-041-04, RENAME-041-05
**Depends on:** Phase 040
**Plans:** 5 plans
**Directory:** `.planning/phases/000/041-renaming-fixes/`

**Execution status:** Phase complete. All 5 plans executed as self-contained waves. Context captured in `041-CONTEXT.md`.

Plans:

- [x] 041-01-PLAN.md — Freeze immutable rename inputs, create execution wrapper around `041-rename-signature-audit.md` (source unchanged), and slice tranche manifests.
- [x] 041-02-PLAN.md — Execute wallet-heavy low-risk suffix rename tranche with atomic call-site/test updates and synchronized pair handling.
- [x] 041-03-PLAN.md — Execute remaining low-risk signature rename tranche across simulator/core/crypto/storage/networks surfaces.
- [x] 041-04-PLAN.md — Execute suffix-semantics and medium-risk internal helper renames, including paired synchronized symbols.
- [x] 041-05-PLAN.md — Run final threat-boundary regression gates, re-audit naming coverage, and close phase artifacts.

---
Last updated: 2026-05-02 after completing Phase 041 with all 5 plans executed.

## Phase 042: Refactor Wallets

**Status**: Planned (formal execution package regenerated)
**Added**: 2026-04-30
**Directory**: `.planning/phases/000/042-refactor-wallets/` (pre-existing)

### Phase 043 Specs

- `042-core-refactore-spec.md`
- `042-db-refactore-spec.md`
- `042-services-refactore-spec.md`

**Plans:** 4 plans queued

### Phase 043 Plans

- [ ] 042-01-PLAN.md — Core refactor row-by-row execution with copied Core target tree + exhaustive ledger and compatibility seam gating.
- [ ] 042-02-PLAN.md — DB refactor row-by-row execution against DB exhaustive ledger and redb boundary split.
- [ ] 042-03-PLAN.md — Services refactor row-by-row execution against services exhaustive ledger and wallet subdomain split.
- [ ] 042-04-PLAN.md — Cross-facade cleanup, invariant regression pass, and end-to-end closeout evidence.

**Scope:** Formalized executable plan set for structural refactor of `crates/z00z_wallets/src/core`, `crates/z00z_wallets/src/db`, and `crates/z00z_wallets/src/services` under strict row-by-row ledger execution.

---
Last updated: 2026-05-02 — Formal phase plan package regenerated (042-01..042-04).

## Phase 043: Gaps Fixes

**Goal:** Close the verified wallet/storage boundary gaps across tx assembly, membership-versus-conservation auditing, the optional forensic archive envelope, receive/tag honesty, and validated stealth-output routing without widening phase scope or changing canonical `.wlt` semantics.
**Requirements**: PH43-TXASM, PH43-CONSERVE, PH43-ASSETAUDIT, PH43-ARCHIVE, PH43-RECEIVE, PH43-TAG, PH43-OUTPUT
**Depends on:** Phase 042
**Plans:** 18/18 plans complete
**Status**: Complete
**Added**: 2026-05-06
**Directory**: `.planning/phases/000/043-gaps-fixes/` (pre-existing)

**Canonical refs:**

- `.planning/phases/000/043-gaps-fixes/043-fixes-spec.md`
- `.planning/phases/000/043-gaps-fixes/043-TODO.md`
- `.planning/phases/000/043-gaps-fixes/043-CONTEXT.md`
- `.planning/phases/000/043-gaps-fixes/043-fixes-spec-2.md`
- `.planning/phases/000/043-gaps-fixes/043-TODO-2.md`

**Success Criteria**:

1. The active `TxAssemblerImpl` path closes reachable Phase 1 stubs and fails closed when public refs are used where resolved confidential input evidence is required.
2. Storage membership, transaction-local Pedersen conservation, and the manual asset-class audit surface remain explicit, separate, and evidence-backed in code, tests, and docs.
3. Canonical `.wlt` semantics remain wallet-state-only while the optional forensic archive stays versioned, hash-bound, explicit, and non-mutating on failed validation.
4. Receive, tag, and stealth-output seams become precise and fail closed without regressing the receiver-native wallet model or the validated approved-sender path.
5. The additive spec-2 slice keeps the public verifier honest without an explicit witness, makes the manual asset-class audit typed around target or status or outcome or mismatch classes, and requires the wallet-prefixed canonical JSONL history artifact beside the `.wlt` snapshot while keeping live tx-history storage and `outputs/tx_exports` distinct.
6. The additive spec-2 E2E test slice proves the typed audit, canonical JSONL export/replay, import-mode gating, artifact separation, redaction, and no-persisted-forensic-toggle contracts through existing Rust test homes and existing closeout artifacts only.

Plans:

- [x] 043-01-PLAN.md — 043-01 Coverage Ledger And Failing-Test Lock-In.
- [x] 043-02-PLAN.md — 043-02 Transaction Assembler Closure.
- [x] 043-03-PLAN.md — 043-03 Storage Membership And Conservation Separation.
- [x] 043-04-PLAN.md — 043-04 Optional Forensic Archive Envelope.
- [x] 043-05-PLAN.md — 043-05 Receive DTO And Status Honesty.
- [x] 043-06-PLAN.md — 043-06 Tag16 Completeness Gate.
- [x] 043-07-PLAN.md — 043-07 Stealth Output Builder Contract Hardening.
- [x] 043-08-PLAN.md — 043-08 Tx And Conservation Regression Wave.
- [x] 043-09-PLAN.md — 043-09 Receive, Tag, And Output Regression Wave.
- [x] 043-10-PLAN.md — 043-10 Archive Closure And Phase Closeout.
- [x] 043-11-PLAN.md — 043-01 Canonical Audit Outcome And Witness Boundary.
- [x] 043-12-PLAN.md — 043-02 Wallet Stem And Canonical Filename Lock-In.
- [x] 043-13-PLAN.md — 043-03 Forensic Transport And Canonical JSONL Emission.
- [x] 043-14-PLAN.md — 043-04 Live Tx-Store And RPC Boundary Lock-In.
- [x] 043-15-PLAN.md — 043-05 Regression Wave And Redaction Checks.
- [x] 043-16-PLAN.md — 043-06 Closeout And Summary.
- [x] 043-17-PLAN.md — 043-17 Spec-2 E2E Integration Tests.
- [x] 043-18-PLAN.md — 043-18 Spec-2 E2E Evidence Sync.

**Scope:** Execute the canonical `043-TODO.md` backlog in strict order, then execute the additive `043-TODO-2.md` and spec-2 E2E test backlog, preserve exact task wording, keep the existing phase directory as the only authority surface, and keep all verification/review/versioning gates subordinate to the Phase 043 specs and planning inputs.

---
Last updated: 2026-05-08 — Phase 043 complete through the additive spec-2 slice; UAT, security, coverage, summary, test-spec, and tests-tasks artifacts are synced.

## Phase 044: Wallet Assets

**Goal:** Close the wallet asset lifecycle around real transaction packages so sender reservation, tx package preservation, offline import, explicit simulated admission, storage-backed reconciliation, receiver finalization, balance views, history, backup, restore, and legacy tx-history migration all share one wallet-centered authority without creating a duplicate store or fake chain-broadcast layer.
**Requirements**: PH44-LEDGER, PH44-SEND, PH44-OFFLINE, PH44-ADMIT, PH44-RECONCILE, PH44-RECEIVE, PH44-BALANCE, PH44-HISTORY, PH44-CANCEL, PH44-DRIFT, P-044-001, P-044-002, P-044-003, P-044-004, P-044-005, P-044-006, P-044-007, P-044-008, P-044-009, P-044-010, P-044-011, P-044-012
**Depends on:** Phase 043
**Plans:** 5 plan files generated; 14 canonical TODO task groups are covered sequentially across the 5 queued waves.
**Status**: Complete; planning artifacts remain authoritative as the implementation record
**Added**: 2026-05-09
**Directory**: `.planning/phases/000/044-wallet-assets/` (pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/000/044-wallet-assets/044-CONTEXT.md`
- `.planning/phases/000/044-wallet-assets/044-TODO.md` (self-contained execution authority)
- `.planning/phases/000/044-wallet-assets/044-wallets-assets-spec.md` (archived source input already embedded into `044-TODO.md`)
- `.planning/phases/000/044-wallet-assets/044-wallets-patch.md` (archived correction input already embedded into `044-TODO.md`)
- `.planning/phases/000/044-wallet-assets/044-01-PLAN.md`
- `.planning/phases/000/044-wallet-assets/044-02-PLAN.md`
- `.planning/phases/000/044-wallet-assets/044-03-PLAN.md`
- `.planning/phases/000/044-wallet-assets/044-04-PLAN.md`
- `.planning/phases/000/044-wallet-assets/044-05-PLAN.md`

**Success Criteria**:

1. Wallet asset rows distinguish available, reserved, exported, pending spend, spent, pending change, pending receive, reorg, quarantine, and dropped lifecycle states, and only `Available` remains spendable.
2. Sender build/send uses the existing selector, assembler, proof backend, full tx verifier, journal, and admission path; no live `BuiltTxStub`, empty details, fake RPC success, or duplicate tx authority remains.
3. `wallet_<stem>_tx_history.jsonl` is the canonical live tx-history store for the wallet stem; `.wlt` remains wallet-state-only, no new broad tx database is introduced, and no new per-tx JSON directory is used for live writes.
4. Every package-bearing JSONL row preserves exact canonical tx package bytes, hashes, sequence, entry linkage, and status history while excluding decrypted wallet secrets and private non-package wallet state.
5. Backup preserves validated live JSONL bytes plus manifest, restore writes JSONL bytes back to the restored wallet stem, and legacy `wallet_<stem>_tx_history/` directories are migration input only.
6. Portable package export/import is role-neutral by tx hash, verifies canonical bytes before mutation, writes only pending receiver state before confirmation, and keeps sender inputs locked after external export unless safe-release evidence exists.
7. Simulated admission is one explicit trait-backed adapter, and storage-backed reconciliation is the only path from pending wallet state to final `Confirmed`, `Spent`, or `Available` lifecycle effects.
8. Report-only receive remains non-persistent, final receiver ownership routes through `recv_route(..., ReceiveNext::PersistClaim)`, and duplicate or conflicting evidence fails closed without increasing available balance.
9. Balance, details, pending lists, history, receipts, and user-facing lifecycle views derive from wallet lifecycle rows, journal data, tx bytes, roles, and typed evidence rather than compatibility defaults.
10. Phase closeout includes the required existing-test updates, new AC/T/PT test scenarios, source-shape guards, coverage mapping, summary artifact, and narrow-first plus release-style validation gates from `044-TODO.md`.

Plans:

- [ ] 044-01 Embedded Coverage And No-Duplicate Lock.
- [ ] 044-02 Wallet Asset Ledger And Reservation Layer.
- [ ] 044-03 Sender Build And Send Lifecycle.
- [ ] 044-04 Tx Journal, Details, Pending Lists, And History.
- [ ] 044-04A Canonical JSONL Live Tx-History Authority And Path Contract.
- [ ] 044-04B JSONL-Backed TxStorageImpl And Folded Reads.
- [ ] 044-04C Backup, Restore, And Forensic JSONL Preservation.
- [ ] 044-04D Legacy Per-Tx JSON Migration And Guards.
- [ ] 044-05 Portable Package Export, Import, And Role-Neutral Submission.
- [ ] 044-06 Admission And Confirmation Boundary.
- [ ] 044-07 Storage-Backed Wallet Reconciliation.
- [ ] 044-08 Receiver Pending, Report-Only, And Finalization Behavior.
- [ ] 044-09 Balance And User-Facing Lifecycle Views.
- [ ] 044-10 Regression Matrix And Source-Shape Guards.

**Scope:** Execute the self-contained `044-TODO.md` backlog in order, preserve the task names and wording during planning, reuse the existing `044-wallet-assets` folder as the only Phase 044 directory, and keep implementation wallet-centered without duplicating tx assembly, verification, receive persistence, wallet asset authority, tx schema, chain broadcast, or scan-cursor proof semantics.

### Phase 047: 047-wallet-redesign

**Goal:** Replace snapshot-owned wallet asset persistence with wallet-native `.wlt` object storage while preserving the live `wallet.tx.*`, receive/scan, backup/restore, TOFU, and simulator behavior without introducing a parallel authority plane.
**Requirements**: `REQ-001` through `REQ-020` from `047-wallet-redesign-spec.md`
**Depends on:** Phase 046
**Plans:** 11 plan files generated; `047-01` through `047-11` are summary-backed complete and no active follow-up lane remains
**Status**: Completed on 2026-05-22 after `047-11-SUMMARY.md` closed the post-closeout remote-scan worker slice. The known simulator wording test remains a separate workspace-wide blocker outside the closed Phase 047 packet.
**Added**: 2026-05-15
**Directory:** `.planning/phases/000/047-wallet-redesign/` (pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/000/047-wallet-redesign/047-TODO.md`
- `.planning/phases/000/047-wallet-redesign/047-CONTEXT.md`
- `.planning/phases/000/047-wallet-redesign/047-SPEC-COVERAGE.md`
- `.planning/phases/000/047-wallet-redesign/047-wallet-redesign-spec.md`
- `.planning/phases/000/047-wallet-redesign/047-wallet-addon-spec.md`
- `.planning/phases/000/047-wallet-redesign/047-wallet-addon-spec2.md`
- `.planning/phases/000/047-wallet-redesign/047-01-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-02-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-03-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-04-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-05-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-06-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-07-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-08-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-09-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-10-SUMMARY.md`
- `.planning/phases/000/047-wallet-redesign/047-11-SUMMARY.md`

**Success Criteria**:

1. `WalletProfilePayload` and `OwnedAssetPayload` become the live `.wlt` authority while Snapshot leaves the normal live write path.
2. Receive, build, reserve, cancel, reconcile, backup, and restore all operate on one owned-asset authority plane plus the explicit JSONL tx-history sidecar.
3. Runtime wallet defaults come from `wallet_config.yaml` or explicit overrides instead of hardcoded Rust literals.
4. Duplicate asset ids fail closed or idempotently no-op only for matching payloads, and asset status transitions keep one stable `object_id`.
5. `WalletPlusHistory` restore remains all-or-nothing and a restored wallet can immediately build from restored owned-asset records.
6. Simulator Stage 13, the phase-local 046 spec copy, and migrated existing tests stop naming Snapshot as the live claimed-asset authority.

**Execution status**: `047-01` is closed on `.planning/phases/000/047-wallet-redesign/047-01-SUMMARY.md`, `047-02` is closed on `.planning/phases/000/047-wallet-redesign/047-02-SUMMARY.md`, `047-03` is closed on `.planning/phases/000/047-wallet-redesign/047-03-SUMMARY.md`, `047-04` is closed on `.planning/phases/000/047-wallet-redesign/047-04-SUMMARY.md`, `047-05` is closed on `.planning/phases/000/047-wallet-redesign/047-05-SUMMARY.md`, `047-06` is closed on `.planning/phases/000/047-wallet-redesign/047-06-SUMMARY.md`, `047-07` is closed on `.planning/phases/000/047-wallet-redesign/047-07-SUMMARY.md`, `047-08` is closed on `.planning/phases/000/047-wallet-redesign/047-08-SUMMARY.md`, `047-09` is closed on `.planning/phases/000/047-wallet-redesign/047-09-SUMMARY.md`, `047-10` is closed on `.planning/phases/000/047-wallet-redesign/047-10-SUMMARY.md`, and `047-11` is closed on `.planning/phases/000/047-wallet-redesign/047-11-SUMMARY.md`. This completed eleven-plan chain preserves the honest Phase 047 core baseline plus the bounded compatibility-cleanup, durable master-key rotation, and remote-scan worker follow-up slices. No active Phase 047 follow-up lane remains.

Plans:

- [x] 047-01-PLAN.md — Schema and payload groundwork for object kinds, versions, indexes, and debug decode support. Closed on `047-01-SUMMARY.md`.
- [x] 047-02-PLAN.md — Low-level object upsert and indexed read API for stable object-id updates. Closed on `047-02-SUMMARY.md`.
- [x] 047-03-PLAN.md — Wallet profile replacement plus YAML-backed runtime default cutover. Closed on `047-03-SUMMARY.md`.
- [x] 047-04-PLAN.md — Owned asset store authority and service/catalog cutover. Closed on `047-04-SUMMARY.md`.
- [x] 047-05-PLAN.md — Receive and scan integration on owned-asset persistence. Closed on `047-05-SUMMARY.md`.
- [x] 047-06-PLAN.md — Transaction build, reservation, cancel, reconcile, and asset-view cutover. Closed on `047-06-SUMMARY.md`.
- [x] 047-07-PLAN.md — Backup, restore, export, and Snapshot compatibility-bridge retirement. Closed on `047-07-SUMMARY.md`.
- [x] 047-08-PLAN.md — Simulator, docs, existing-test migration, and final validation closeout. Closed on `047-08-SUMMARY.md`.

Post-closeout follow-up packet:

- [x] 047-09-PLAN.md — Remove obsolete compatibility state from normal reopen, export, restore, and pack flows so only the canonical profile plus owned-asset plus scan-state plus JSONL history path remains. Closed on `047-09-SUMMARY.md`.
- [x] 047-10-PLAN.md — Turn `wallet.key.rotate_master_key` into an honest durable persisted rotation with restart-safe rewrite semantics. Closed on `047-10-SUMMARY.md`; the required broad rerun still hits only `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs::test_boundary_wording_stays_narrow` outside this wallet slice.
- [x] 047-11-PLAN.md — Add remote scan or worker assistance while keeping wallet-local `recv_range(...)` and `StealthOutputScanner` as the ownership authority. Closed on `047-11-SUMMARY.md`.

**Scope:** Execute the eight-wave migration sequence defined by `047-wallet-redesign-spec.md` inside the existing phase folder only: schema and index groundwork, low-level upsert/query helpers, profile/config cutover, owned-asset authority, receive/scan cutover, tx lifecycle cutover, backup/restore/export cutover, then simulator/doc/test migration and final validation. The bounded post-closeout packet on `047-09` through `047-11` is complete; any later tx-history migration or `wallet.asset.*` convergence work belongs to a later phase, not to this closed packet. Threat IDs `T-047-12` and `T-047-13` in supporting audit docs do not imply extra plan files.

---
Last updated: 2026-05-28 — Phase 044 remains complete with `044-CONTEXT.md` plus `044-01-PLAN.md` through `044-05-PLAN.md`; Phase 046 is paused after `046-04` under the existing six-plan packet, with `046-01` through `046-04` summary-backed complete, `046-05` pending, and `046-06` paused pending rewrite for the wallet `.wlt` redesign direction; Phase 047 is now summary-backed complete through `047-11-SUMMARY.md`, no active follow-up lane remains, and the closed packet stays in `.planning/phases/000/047-wallet-redesign/`, while the known simulator wording test remains a separate workspace blocker outside the closed Phase 047 packet.

## Phase 046: Wallet Addons

**Goal:** Prove the live wallet.tx lifecycle, claimed-asset persistence, WalletPlusHistory restore, payment-request / TOFU handling, session hardening, rotate_master_key in-memory rederive boundaries, stale comment cleanup, and simulator evidence that matches the production wallet boundary without creating a duplicate phase folder or a second wallet authority plane.
**Requirements**: 046-01, 046-02, 046-03, 046-04, 046-05, 046-06, 046-07, 046-08, 046-09
**Depends on:** Phase 044
**Plans:** 6 plan files generated; `046-01` through `046-04` are summary-backed complete, `046-05` is pending, and `046-06` is paused pending rewrite.
**Status**: Paused after `046-04`; the phase packet lives in the existing `.planning/phases/000/046-wallet-addons/` directory.
**Added**: 2026-05-13
**Directory**: `.planning/phases/000/046-wallet-addons/` (pre-existing; do not create a duplicate phase folder)

**Execution status**: `046-01` is closed on `.planning/phases/000/046-wallet-addons/046-01-SUMMARY.md`, `046-02` is closed on `.planning/phases/000/046-wallet-addons/046-02-SUMMARY.md`, `046-03` is closed on `.planning/phases/000/046-wallet-addons/046-03-SUMMARY.md`, and `046-04` is closed on `.planning/phases/000/046-wallet-addons/046-04-SUMMARY.md`. The active tree passed the mandatory bootstrap gate, focused wallet 046-04 gates, and the broad `cargo test --release --features test-fast --features wallet_debug_dump` gate on 2026-05-15. Per user instruction, execution stopped after `046-04`; `046-05` was not executed and `046-06` is paused pending rewrite before any tamper/release-smoke closeout work.

**Canonical refs:**

- `.planning/phases/000/046-wallet-addons/046-CONTEXT.md`
- `.planning/phases/000/046-wallet-addons/046-wallet-addon-spec.md`
- `.planning/phases/000/046-wallet-addons/046-wallet-misses.md`
- `.planning/phases/000/046-wallet-addons/046-storage-explain.md`
- `.planning/phases/000/046-wallet-addons/046-01-SUMMARY.md`
- `.planning/phases/000/046-wallet-addons/046-02-SUMMARY.md`
- `.planning/phases/000/046-wallet-addons/046-03-SUMMARY.md`
- `.planning/phases/000/046-wallet-addons/046-04-SUMMARY.md`
- `.planning/phases/000/046-wallet-addons/046-01-PLAN.md`
- `.planning/phases/000/046-wallet-addons/046-02-PLAN.md`
- `.planning/phases/000/046-wallet-addons/046-03-PLAN.md`
- `.planning/phases/000/046-wallet-addons/046-04-PLAN.md`
- `.planning/phases/000/046-wallet-addons/046-05-PLAN.md`
- `.planning/phases/000/046-wallet-addons/046-06-PLAN.md`

**Success Criteria**:

1. Scenario 1 exposes a 13th stage that routes into the canonical wallet.tx lifecycle rather than a simulator-only side lane.
2. Live wallet.tx build, broadcast, cancel, reconcile, backup, restore, and history evidence all agree on the same claimed-asset ownership plane.
3. Backup restore via `WalletPlusHistory` preserves both claimed assets and canonical tx-history JSONL, and wrong-password restores fail closed without mutating live state.
4. Payment-request validation and session hardening reject stale, cross-chain, or over-limit inputs before wallet.tx can consume them.
5. `wallet.key.rotate_master_key` remains an audited in-memory rederive flow with one-per-hour rate limiting and no durable seed-rotation claim.
6. Stale `stub`, `placeholder`, `Phase 1`, and `residue` wording is removed from the touched wallet comments and RPC labels so the docs match the live boundaries.
7. The exact release simulator and wallet commands prove the phase 13 path with the expected `wallet_debug_dump` coverage flags.

Plans:

- [x] 046-01-PLAN.md — Stage registration, canonical contract expansion, and stage 13 facade scaffold.
- [x] 046-02-PLAN.md — Live wallet.tx lifecycle flow plus root-binding and history evidence helpers.
- [x] 046-03-PLAN.md — WalletPlusHistory restore parity and wallet-side scan-resume authority.
- [x] 046-04-PLAN.md — Payment-request / TOFU hardening plus session limits and rotate-master-key audit boundaries.
- [ ] 046-05-PLAN.md — Compatibility wording cleanup for wallet.asset, receive, reachability, backup, and scanner surfaces. Pending; not executed in this stop point.
- [ ] 046-06-PLAN.md — Tamper-closed regression tests and exact Scenario 1 / wallet release smoke gates. Paused pending rewrite before execution.

**Scope:** Execute the six-plan Phase 046 packet in wave order from the existing `.planning/phases/000/046-wallet-addons/` folder, preserve the locked PRD decisions, keep the wallet.tx boundary canonical, and finish only when the exact release-style wallet and simulator gates are green.

## Phase 051: HJMT Facade

**Goal:** Establish the HJMT storage migration boundary by planning the storage-owned backend facade, compatibility backend, root taxonomy, proof-envelope contract, and equivalence corpus from the existing Phase 051 inventory.
**Requirements**: PH51-BACKEND-FACADE, PH51-COMPAT-BACKEND, PH51-ROOT-TAXONOMY, PH51-PROOF-ENVELOPE, PH51-GUARDRAILS, PH51-EQUIVALENCE, PH51-CHECKPOINT-RELOAD, PH51-ROLLOUT-HANDOFF
**Depends on:** Phase 047 closeout; Phase 046 remains paused separately and is not closed by this addition.
**Plans:** 6/6 plans complete.
**Status**: Completed 2026-05-28 from the review-hardened planning packet; `051-01` through `051-06` are summary-backed complete with release-mode code validation and docs-only hygiene checks.
**Added**: 2026-05-28
**Directory**: `.planning/phases/000/051-HJMT-Facade/` (pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/000/051-HJMT-Facade/051-TODO.md`
- `.planning/phases/000/051-HJMT-Facade/051-CONTEXT.md`
- `.planning/phases/000/051-HJMT-Facade/051-01-PLAN.md`
- `.planning/phases/000/051-HJMT-Facade/051-01-SUMMARY.md`
- `.planning/phases/000/051-HJMT-Facade/051-02-PLAN.md`
- `.planning/phases/000/051-HJMT-Facade/051-02-SUMMARY.md`
- `.planning/phases/000/051-HJMT-Facade/051-03-PLAN.md`
- `.planning/phases/000/051-HJMT-Facade/051-03-SUMMARY.md`
- `.planning/phases/000/051-HJMT-Facade/051-04-PLAN.md`
- `.planning/phases/000/051-HJMT-Facade/051-04-SUMMARY.md`
- `.planning/phases/000/051-HJMT-Facade/051-05-PLAN.md`
- `.planning/phases/000/051-HJMT-Facade/051-05-SUMMARY.md`
- `.planning/phases/000/051-HJMT-Facade/051-06-PLAN.md`
- `.planning/phases/000/051-HJMT-Facade/051-TEST-SPEC.md`
- `.planning/phases/000/051-HJMT-Facade/051-TESTS-TASKS.md`

**Success Criteria**:

1. Phase 051 is discoverable in the active milestone without creating a second directory.
2. `051-CONTEXT.md` captures source-backed decisions from `051-TODO.md`, `docs/Z00Z-JMT-Design.md`, current state-management references, live storage code, and the explicit source, JMT, and state-management coverage matrices.
3. `051-01-PLAN.md` introduces the semantic storage facade and moves the current shared namespaced JMT behind it as the compatibility backend.
4. `051-02-PLAN.md` codifies root taxonomy and the compatibility proof-envelope version boundary.
5. `051-03-PLAN.md` prevents downstream crates from depending on raw backend roots, raw `TreeId`, namespace prefixes, branch ordering, or physical key layout as authority.
6. `051-04-PLAN.md` adds the compatibility golden corpus for insert, delete, replay, proof verification, checkpoint seal/reload, and path-index rebuild behavior.
7. `051-05-PLAN.md` records truthful docs, roadmap/state evidence, and forest-backend handoff without claiming the production forest backend shipped.
8. `051-06-PLAN.md` proves Phase 052 readiness on the canonical seam without introducing a standalone duplicate authority layer.
9. Phase 051 does not duplicate the existing codebase or introduce a parallel storage authority layer.
10. Every `051-TODO.md` bullet plus referenced JMT and state-management requirement is either routed into executable Phase 051 work, recorded as a downstream authority guardrail, or explicitly classified as future forest-backend handoff behind the facade.
11. `051-TEST-SPEC.md` and `051-TESTS-TASKS.md` define the phase-local unit, integration, Rust E2E, negative, proof, checkpoint, reload, golden-corpus, and downstream guardrail coverage without implementing tests or creating a fake forest backend.

Plans:

- [x] 051-01-PLAN.md — Storage facade and compatibility backend wrapper.
- [x] 051-02-PLAN.md — Root taxonomy and compatibility proof-envelope boundary.
- [x] 051-03-PLAN.md — Public API guardrails and downstream semantic facade cutover.
- [x] 051-04-PLAN.md — Compatibility golden corpus for semantic, proof, checkpoint, reload, and path-index behavior.
- [x] 051-05-PLAN.md — Storage docs, closeout evidence, and future forest handoff.
- [x] 051-06-PLAN.md — 052 readiness gate and canonical seam verification.

**Scope:** Reuse `.planning/phases/000/051-HJMT-Facade/` as the only Phase 051 directory. Do not create another `051` folder. Execute the six-plan packet in order. Phase 051 is a facade and compatibility-reference phase; the production HJMT forest backend, fixed bucket rollout, commit journal, and performance enablement remain future work behind the facade. Future work must join through the facade and compatibility corpus, not a duplicate storage authority layer.

## Phase 052: HJMT Backend

**Goal:** Implement the real HJMT forest backend behind the Phase 051 storage facade with fixed buckets, journaled child-before-parent publication, fail-closed proofs, dual-backend equivalence, and configuration-gated rollout.
**Requirements**: JMT-REQ-003, JMT-REQ-004, JMT-REQ-005, JMT-REQ-006, JMT-REQ-007, JMT-REQ-008, JMT-REQ-009, JMT-REQ-010, JMT-REQ-011, JMT-REQ-012
**Depends on:** Phase 051 closeout and readiness handoff.
**Plans:** 11 ordered execution and follow-up slices (`052-01` through `052-11`) in the existing Phase 052 folder.
**Status**: Completed 2026-05-29; `052-01` through `052-11` are summary-backed complete after landing backend mode routing, fixed bucket policy metadata, private forest tree identities, deterministic forest batch planning, in-memory forest commits, durable journaled child-before-parent publication, crash recovery, reload validation, private path-index rebuild, claim replay digest binding, checkpoint metadata hardening, storage-owned forest inclusion proofs, bucket-policy recomputation, chained proof verification, fail-closed forest proof reject coverage, explicit unsupported deletion or non-existence proof families, real forest and dual-verify golden-corpus execution, forest checkpoint-attested execution, dual mismatch hard failures, downstream semantic-authority guardrails, compatibility-default rollout gating, async benchmark evidence, proof-size and proof verification timing evidence, cross-mode `scenario_1` success, closeout validation, green-state audit, first-class follow-up ledger, adaptive bucket migration candidate capture, bucket occupancy metadata privacy candidate capture, generalized settlement-root candidate capture, and RightLeaf/FeeEnvelope candidate capture without creating a new phase folder.
**Added**: 2026-05-28
**Directory**: `.planning/phases/052-HJMT-Backend/` (pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/052-HJMT-Backend/052-TODO.md`
- `.planning/phases/052-HJMT-Backend/052-CONTEXT.md`
- `.planning/phases/052-HJMT-Backend/052-TEST-SPEC.md`
- `.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md`
- `docs/Z00Z-JMT-Design.md`
- `.planning/phases/000/051-HJMT-Facade/051-TODO.md`
- `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
- `crates/z00z_storage/tests/test_phase051_guardrails.rs`

**Success Criteria**:

1. Phase 052 is discoverable in the active milestone without creating a second Phase 052 directory.
2. The existing `.planning/phases/052-HJMT-Backend/052-TODO.md`, `052-CONTEXT.md`, `052-TEST-SPEC.md`, `052-TESTS-TASKS.md`, and `052-01-PLAN.md` through `052-11-PLAN.md` packet remains the canonical Phase 052 implementation contract.
3. All real forest-backend work stays behind the Phase 051 `AssetTreeBackend` facade and uses `CompatibilityBackend` as the migration oracle rather than a second public authority path.
4. Phase 052 does not reopen root vocabulary, proof-envelope ownership, checkpoint contracts, or downstream semantic-authority guardrails locked by Phase 051.
5. Phase 052 planning must route fixed buckets, forest commit journal, reload validation, proof reject matrix, checkpoint validation, and path-index rebuild through one storage-owned implementation path.
6. Storage validation must include the Phase 051 golden corpus plus the Phase 052 forest extensions for insert/delete, proof, reload, checkpoint, and reject behavior.
7. Simulator `scenario_1` remains a consumer of storage semantics only and passes through compatibility, forest, and dual-verify backend modes; Plan 06 records the explicit rollout and scenario-mode matrix.
8. Plans `052-08` through `052-11` preserve first-class follow-up and migration-candidate work for adaptive buckets, bucket occupancy metadata privacy, generalized settlement roots, and RightLeaf/FeeEnvelope without making those concepts live public authority in Phase 052.

Plans:

**Wave 1**

- [x] `052-01-PLAN.md` — backend mode selection, forest skeleton, and fixed bucket policy.

**Wave 2** *(blocked on Wave 1 completion)*

- [x] `052-02-PLAN.md` — private forest tree store and deterministic batch planner.

**Wave 3** *(blocked on Wave 2 completion)*

- [x] `052-03-PLAN.md` — forest commit journal, crash recovery, reload validation, and path-index rebuild.

**Wave 4** *(blocked on Wave 3 completion)*

- [x] `052-04-PLAN.md` — forest proof envelope plus deletion and non-existence proof families.

**Wave 5** *(blocked on Wave 4 completion)*

- [x] `052-05-PLAN.md` — dual-backend equivalence corpus and checkpoint or downstream semantic guardrails.

**Wave 6** *(blocked on Wave 5 completion)*

- [x] `052-06-PLAN.md` — rollout gating, benchmark evidence, scenario validation, and closeout.

**Wave 7** *(blocked on Wave 6 completion)*

- [x] `052-07-PLAN.md` — green-state audit and first-class follow-up ledger.

**Wave 8** *(blocked on Wave 7 completion)*

- [x] `052-08-PLAN.md` — adaptive buckets and migration proofs candidate.

**Wave 9** *(blocked on Wave 8 completion)*

- [x] `052-09-PLAN.md` — bucket occupancy metadata privacy candidate.

**Wave 10** *(blocked on Wave 9 completion)*

- [x] `052-10-PLAN.md` — generalized settlement root model candidate.

**Wave 11** *(blocked on Wave 10 completion)*

- [x] `052-11-PLAN.md` — RightLeaf and FeeEnvelope protocol candidate.

**Cross-cutting constraints:**

- Compatibility remains the default backend until equivalence, guardrails, checkpoint, reload, and benchmark gates are green.
- `AssetTreeBackend`, `AssetStore`, `AssetStateRoot`, `CheckRoot`, `ProofBlob`, and `chk_blob` remain the only caller-facing semantic authority seam.
- No public API or downstream crate may treat `TreeId`, bucket ids, namespace bytes, branch ordering, or backend roots as authority.
- Async `multi-insert` and `multi-delete` benchmarks plus inclusion proof-size evidence and explicit non-existence unsupported fail-closed status are mandatory before phase closeout.
- Adaptive bucket split or merge proofs, proof-visible occupancy counters, generalized settlement roots, `RightLeaf`, and `FeeEnvelope` are first-class follow-up work only until a repository-backed protocol migration explicitly changes the live design source and authority vocabulary.
- Every Rust or test-affecting task must run bootstrap first, then broad cargo validation when relevant, then `/GSD-Review-Tasks-Execution` at least three times with two consecutive clean runs.

**Scope:** Reuse `.planning/phases/052-HJMT-Backend/` as the only Phase 052 directory. Do not create another `052` folder. Planning and execution must begin from the existing `052-TODO.md` backlog and the eleven-plan packet. Phase 052 owns the real HJMT forest backend, fixed bucket policy, journaled child-before-parent publication, fail-closed proof families, dual-backend equivalence, reload/checkpoint/path-index validation, configuration-gated rollout, benchmark and proof-size evidence, simulator scenario validation, and explicit future-migration guardrails, while preserving the Phase 051 facade, compatibility oracle, root vocabulary, proof ownership, and downstream authority boundaries.

## Phase 053: HJMT Backend

**Goal:** Promote the former Phase 052 follow-up packet into the live generalized HJMT generation by replacing the Phase 052 asset-centric runtime with `SettlementStateRoot`, `SettlementPath`, `SettlementLeaf`, `RightLeaf`, `FeeEnvelope`, live deletion and non-existence proofs, adaptive bucket policy proofs, cache and scheduler production machinery, and full downstream integration.
**Requirements**: [PH53-01, PH53-02, PH53-03, PH53-04, PH53-05, PH53-06, PH53-07, PH53-08, PH53-09, PH53-10, PH53-11, PH53-12, PH53-13, PH53-14, PH53-15, PH53-16, PH53-17, PH53-18, PH53-19, PH53-20] derived from canonical `053-TODO.md`; see `.planning/phases/053-HJMT-Backend/053-SOURCE-AUDIT.md` for source coverage.
**Depends on:** Phase 052 closeout and its explicit follow-up ledger; Phase 046 remains paused separately and is not advanced by this registration.
**Plans:** 20 ordered numbered plan files exist in `.planning/phases/053-HJMT-Backend/` (`053-01-PLAN.md` through `053-20-PLAN.md`), plus `053-CONTEXT.md` and `053-SOURCE-AUDIT.md`.
**Status**: Completed 2026-06-05 from the pre-existing phase folder; `053-01-SUMMARY.md` through `053-20-SUMMARY.md` plus final `053-SUMMARY.md` are complete.
**Added**: 2026-05-29
**Directory**: `.planning/phases/053-HJMT-Backend/` (pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/053-HJMT-Backend/053-TODO.md`
- `docs/Z00Z-HJMT-Design.md`
- `.planning/phases/000/051-HJMT-Facade/051-TODO.md`
- `.planning/phases/052-HJMT-Backend/052-TODO.md`
- `.planning/phases/052-HJMT-Backend/052-08-PLAN.md`
- `.planning/phases/052-HJMT-Backend/052-09-PLAN.md`
- `.planning/phases/052-HJMT-Backend/052-10-PLAN.md`
- `.planning/phases/052-HJMT-Backend/052-11-PLAN.md`

**Success Criteria**:

1. Phase 053 is discoverable in the active milestone without creating a second `053` directory.
2. `053-TODO.md` remains the canonical planning inventory, and every future numbered plan must preserve its task ordering and wording.
3. Phase 053 promotes the former Phase 052 future-only packet into live generalized-settlement scope: `SettlementStateRoot`, `SettlementPath`, `SettlementLeaf`, `RightLeaf`, `FeeEnvelope`, deletion/non-existence proofs, adaptive bucket proofs, occupancy privacy evidence, cache/scheduler productionization, downstream integration, and legacy-storage purge.
4. Phase 053 is a dev hard cutover phase: no duplicate legacy runtime, compatibility adapter lane, or old-storage conversion path may become part of the green state.
5. Roadmap/state continuity must point to the existing `.planning/phases/053-HJMT-Backend/` plan packet, with `053-01-PLAN.md` queued first.

Plans:

- [x] 053-01 — Replace future-only guardrails with live-contract guardrails.
- [x] 053-02 — Settlement root generation and hard cutover model.
- [x] 053-03 — SettlementPath, TerminalId, SettlementLeaf, and RightLeaf.
- [x] 053-04 — FeeEnvelope contract and separation from rights.
- [x] 053-05 — HJMT store API and dev hard cutover.
- [x] 053-06 — Core YAML, genesis rights, and full-stack fixture integration.
- [x] 053-07 — Proof envelope generation 2: inclusion, deletion, and non-existence.
- [x] 053-08 — Adaptive buckets, BucketEpoch, SplitProof, MergeProof, and PolicyTransitionProof.
- [x] 053-09 — Occupancy metadata privacy and adaptive threshold evidence.
- [x] 053-10 — Forest cache plane.
- [x] 053-11 — Async forest scheduler and parallel commit pipeline.
- [x] 053-12 — Journal, recovery, and durable policy state.
- [x] 053-13 — RedB persistence, reload, historical proofs, and cache warmup.
- [x] 053-14 — Checkpoint, snapshot, claim source, wallet, and validator integration.
- [x] 053-15 — Scenario 1 production examples.
- [x] 053-16 — Golden corpus, property tests, and fuzz seeds.
- [x] 053-17 — Benchmarks, metrics, and performance gates.
- [x] 053-18 — Documentation, API examples, and hard-cutover notes.
- [x] 053-19 — Closeout and production default gate.
- [x] 053-20 — Legacy storage purge and dead code cleanup.

**Scope:** Reuse `.planning/phases/053-HJMT-Backend/` as the only Phase 053 directory. Do not create another `053` folder. Planning must begin from the existing `053-TODO.md` backlog. This phase owns the live generalized settlement root generation, terminal right and fee contracts, deletion/non-existence/adaptive policy proof families, privacy-reviewed occupancy evidence, cache/scheduler productionization, RedB and journal extensions, checkpoint/snapshot/downstream integration, scenario examples, documentation, benchmarks, and live-code purge of superseded compatibility/simple-JMT storage tails.

## Phase 054: Refactor Crates

**Goal:** Realign `z00z_rollup_node`, `z00z_runtime/*`, and `z00z_storage` around a storage backend seam, a runtime planner-authority split, and a delayed rename wave without creating duplicate authority surfaces or duplicate phase folders.
**Requirements**: `PH54-01` backend guardrails and seam contracts, `PH54-02` backend extraction and `SettlementStore` rewiring, `PH54-03` runtime planner split, `PH54-04` placement and downstream runtime boundaries, `PH54-05` storage canonical-module cleanup, `PH54-06` delayed rename wave, `PH54-07` docs and closeout gates, `PH54-08` runtime digest rebinding and planner-boundary hardening.
**Depends on:** Phase 053 closeout; Phase 046 remains paused separately and is not advanced by this packet.
**Plans:** 8/8 plans complete
**Status**: Phase 054 is fully complete on 2026-06-09 in the pre-existing phase folder: `054-01` through `054-08` are summary-backed complete, `054-SUMMARY.md` records the full closeout, `054-SECURITY.md` now closes the original 21 tracked threats plus the follow-up runtime digest-ingress threat, `054-VALIDATION.md` records Nyquist compliance through the `054-08` continuation, the historical closeout still includes a green `cargo test --all --release -q`, and the final `054-08` follow-up proves that runtime planner route selection, intake ids, and `plan_digest` now bind to one payload-verified digest path with no public bypass or alias return path.
**Added**: 2026-06-08
**Directory**: `.planning/phases/054-Refactor-Crates/` (pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/054-Refactor-Crates/054-TODO.md`
- `.planning/phases/054-Refactor-Crates/054-CONTEXT.md`
- `.planning/phases/054-Refactor-Crates/054-SOURCE-AUDIT.md`
- `.planning/phases/054-Refactor-Crates/054-SECURITY.md`
- `.planning/phases/054-Refactor-Crates/054-VALIDATION.md`
- `.planning/phases/054-Refactor-Crates/054-attack-surface-report.md`
- `.planning/phases/054-Refactor-Crates/054-attack-surface-db.jsonl`
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`
- `docs/tech-papers/Z00Z-HJMT-Key-Terms.md`

**Success Criteria**:

1. Phase 054 is discoverable in the active milestone without creating a second `054` directory.
2. The numbered execution packet exists only inside `.planning/phases/054-Refactor-Crates/` and preserves the existing phase-local authority chain.
3. Backend-seam extraction, planner-authority split, storage canonicalization, delayed renames, and docs closeout stay explicitly separated by wave.
4. The planning packet preserves storage semantic truth ownership, runtime operational boundaries, and legacy source-shape closeout rules without recreating deleted historical artifacts.
5. The review-hardened packet remains terminology-aligned with the referenced HJMT design, upgrade, and key-terms corpus without introducing a parallel implementation layer.
6. Runtime planner authority is explicit in `batch_planner.rs`, while storage `tx_plan` is narrowed back to store-local semantic machinery before placement and rename waves continue.
7. No alias, shim, or duplicate public path remains in the live phase-owned runtime, node, or storage surfaces; validation guards enforce one canonical module path story.
8. The runtime planner boundary rejects forged tx or claim digest metadata locally, and one payload-bound digest path drives route lookup, intake ids, and `plan_digest`.

Plans:

**Wave 1**

- [x] `054-01` Guardrails and backend contract freeze.

**Wave 2** *(blocked on Wave 1 completion)*

- [x] `054-02` Backend extraction and `SettlementStore` rewire.

**Wave 3** *(blocked on Wave 2 completion)*

- [x] `054-03` Runtime batch planner split.
- [x] `054-04` Placement, shard execution, and downstream runtime boundaries.

**Wave 4** *(blocked on Waves 2-3 completion)*

- [x] `054-05` Storage canonical-module cleanup.

**Wave 5** *(blocked on Waves 3-4 completion)*

- [x] `054-06` Delayed rename wave.

**Wave 6** *(blocked on Wave 5 completion)*

- [x] `054-07` Docs, migration tables, and closeout gates.

**Wave 7** *(blocked on Wave 6 completion and the accepted attack-surface audit)*

- [x] `054-08` Runtime digest rebinding and planner canonical-path hardening.

**Cross-cutting constraints:**

- `SettlementStore`, `SettlementTreeBackend`, and the proof/public API remain authoritative until the seam and rename waves are fully stabilized.
- Runtime planner and placement metadata stay operational only; validators and watchers never become alternate truth owners.
- The runtime planner boundary does not route, sort, or hash from raw caller-supplied digest metadata; one verified payload-bound digest path remains canonical.
- `StoreBackendError` and the current runtime public re-exports stay stable until the delayed rename wave.
- `assets_proofs.rs` and the storage suites remain compatibility gates through the storage-heavy waves.
- Every Rust or test-affecting auto task runs bootstrap first, then `cargo test --release` when relevant, then repeated `/GSD-Review-Tasks-Execution`; any commit uses `/z00z-git-versioning`.

**Scope:** Reuse `.planning/phases/054-Refactor-Crates/` as the only Phase 054 directory. Do not create another `054` folder. `054-TODO.md` remains the canonical backlog, `054-CONTEXT.md` and `054-SOURCE-AUDIT.md` lock the planning packet, and `054-TODO.md` section 18 becomes the live legacy carry-over authority whenever `legacy-refactor-spec.md` is absent in the current worktree.

## Phase 055: HJMT Boundary

**Goal:** Freeze and then implement the storage-owned batch-proof boundary so
`BatchProofBlobV1` becomes a deterministic, fail-closed, benchmarked live proof
surface beside the unchanged single-path `ProofBlob`, while Stage 13 gains real
comparison and tamper evidence without introducing a duplicate authority layer
or a duplicate phase folder.
**Requirements**: `PH55-01` exact batch-proof wire contract and codec,
`PH55-02` fail-closed parser and atomic verifier, `PH55-03` storage-owned
builder plus positive fixtures and compatibility baseline, `PH55-04`
benchmarks plus Stage 13 evidence plus docs or guardrails.
**Depends on:** Phase 054 closeout; Phase 046 remains paused separately and is
not advanced by this registration.
**Plans:** 4/4 ordered numbered execution plans are now summary-backed
complete in the pre-existing phase folder: `055-01-PLAN.md` through
`055-04-PLAN.md`.
**Status**: Added 2026-06-09 on the existing
`.planning/phases/055-HJMT-boundary/` directory; `055-TODO.md` remains the
canonical backlog, and `055-CONTEXT.md`, `055-SOURCE-AUDIT.md`, and
`055-TEST-SPEC.md` remain the Phase 1 contract-freeze packet, while
`055-01-SUMMARY.md` through `055-04-SUMMARY.md` now close the exact
storage-owned contract/codec, fail-closed verifier/tamper, builder/fixture,
and benchmark/Stage 13 evidence slices from the Phase 2 execution packet.
Final `055-SUMMARY.md` records phase completion and there is no active `055`
execution lane.
**Added**: 2026-06-09
**Directory**: `.planning/phases/055-HJMT-boundary/` (pre-existing; do not
create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/055-HJMT-boundary/055-TODO.md`
- `.planning/phases/055-HJMT-boundary/055-CONTEXT.md`
- `.planning/phases/055-HJMT-boundary/055-SOURCE-AUDIT.md`
- `.planning/phases/055-HJMT-boundary/055-TEST-SPEC.md`
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
- `docs/tech-papers/Z00Z-HJMT-Key-Terms.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`

**Success Criteria**:

1. Phase 055 is discoverable in the active milestone without creating a second
   `055` directory.

2. The existing `.planning/phases/055-HJMT-boundary/` folder remains the only
   canonical phase root, and `055-TODO.md` remains the backlog authority.

3. Phase 1 contract-freeze deliverables are recorded in packet docs before live
   code execution claims are made.

4. Phase 2 is executed as live-code batch-proof work: exact codec, atomic
   verifier, builder derived from current `ProofBlob` truth, deterministic
   fixtures, benchmark evidence, Stage 13 evidence, and any whitepaper-backed
   boundary requirement that constrains those live seams.

5. `ProofBlob` remains unchanged, `Vec<ProofBlob>` remains the independent
   batch baseline, and `BatchProofBlobV1` is added as a separate envelope.

6. No duplicate runtime, storage, scenario, or bench authority layer is
   introduced while planning or executing Phase 055.

Plans:

- [x] `055-01` — Exact `BatchProofBlobV1` wire contract, deterministic codec,
  parser limits, and export surface.

- [x] `055-02` — Fail-closed parser and atomic verifier plus tamper vectors
  `BPB-T-001` through `BPB-T-008`.

- [x] `055-03` — Storage-owned builder from current `ProofBlob` contexts plus
  positive fixtures `BPB-G-001` through `BPB-G-005` and compatibility
  preservation.

- [x] `055-04` — Batch benchmark lanes, Stage 13 comparison and tamper
  evidence, and the required docs or guardrail updates.

**Cross-cutting constraints:**

- Reuse the existing storage backend seam `StorageBackend` plus
  `JournalBackend`; do not invent a second durable seam.

- Treat the upgrade paper's shard-aware fields, migration vectors, backend
  boundaries, and failover recommendations as live Phase 055 authority inputs.
  Version 1 still accepts only the current non-sharded generation, so
  unsupported shard context or future generation must reject fail-closed on the
  live code path instead of being relabeled as out-of-scope.

- Keep the owner-home inventory names from Phase 1 as canonical targets and
  guardrails inside Phase 055, but satisfy them through the current
  storage/runtime/simulator/bench seams instead of empty placeholder suites or
  parallel harnesses.

- Keep Stage 13 as the only simulator evidence authority for the batch-proof
  extension instead of creating a second scenario lane.

- Use the live manifest feature names `test-params-fast` and
  `wallet_debug_tools` in future verification blocks instead of stale older
  aliases.

**Scope:** Reuse `.planning/phases/055-HJMT-boundary/` as the only Phase 055
directory. Do not create another `055` folder. `055-TODO.md` remains the
canonical backlog, the contract-first packet lives in `055-CONTEXT.md`,
`055-SOURCE-AUDIT.md`, and `055-TEST-SPEC.md`, and all later execution must
stay inside the existing folder and the live storage/runtime/scenario
boundaries already accepted by Phases 053 and 054.

## Phase 056: HJMT Storage Aggregator

**Goal:** Make the first real sharded runtime lawful and reproducible by
freezing one canonical `SIM-5A7S` execution topology, routing committed shard
work through runtime-owned planner truth, running aggregators as independent OS
processes, loading topology and runtime behavior from YAML, preserving
semantic-only runtime-to-storage handoff, and proving same-lineage failover and
startup preflight without creating a duplicate phase folder or a second storage
authority plane.
**Requirements**: `056-G1` through `056-G10` from `056-TODO.md`.
**Depends on:** Phase 055 closeout; Phase 046 remains paused separately and is
not advanced by this registration.
**Plans:** 7/7 plans complete.
**Status**: Added 2026-06-11 on the existing
`.planning/phases/056-HJMT-storage- aggregator/` directory. No new phase
folder was created. Planning is complete, `056-01-SUMMARY.md` closes the
first execution slice, `056-02-SUMMARY.md` closes the route-table/planner
truth freeze slice, `056-03-SUMMARY.md` closes the semantic handoff and
dynamic scope-birth slice, `056-04-SUMMARY.md` closes the journal-lineage,
restart, and lawful-failover slice, `056-05-SUMMARY.md` closes the
YAML-materialization and startup-preflight slice, `056-06-SUMMARY.md` closes
the simulator runtime-evidence slice, and `056-07-SUMMARY.md` closes the
fixture, benchmark, validation, and planning-state sync slice. Phase 056 is
complete as of 2026-06-12.
**Added**: 2026-06-11
**Directory**: `.planning/phases/056-HJMT-storage- aggregator/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/056-HJMT-storage- aggregator/056-TODO.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-CONTEXT.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-SOURCE-AUDIT.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-01-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-01-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-02-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-02-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-03-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-03-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-04-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-04-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-05-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-05-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-06-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-06-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-07-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-07-SUMMARY.md`
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`

**Success Criteria**:

1. Phase 056 is discoverable in the active milestone without creating a second
   `056` directory.

2. The existing `.planning/phases/056-HJMT-storage- aggregator/` folder
   remains the only canonical phase root, and `056-TODO.md` remains the
   backlog and execution authority while `056-CONTEXT.md`,
   `056-SOURCE-AUDIT.md`, and `056-01-PLAN.md` through `056-07-PLAN.md`
   schedule the work.

3. `SIM-5A7S` exists as a reproducible acceptance fixture, every aggregator in
   that fixture runs as a separate OS process, and the runtime does not depend
   on shared-memory multi-aggregator shortcuts.

4. `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, and
   `scenario_config.yaml` are loaded from disk, materially change runtime
   behavior, and prove at least one additional positive non-`5x7` topology
   without code edits.

5. Planner truth remains runtime-owned, storage remains the sole owner of
   subtree lifecycle and proof truth, and the first-seen semantic scope birth
   path is evidenced through `scope_flow.json` without promoting storage
   internals to protocol truth.

6. Same-lineage failover works, illegal lineage or generation or split-brain
   states reject fail-closed, startup preflight rejects invalid route or codec
   or placement or lineage state before live work is accepted, and simulator
   traces stay synchronized with `scenario_design.yaml`.

Plans:

- [x] 056-01-PLAN.md — Topology, process model, and config-home freeze.
- [x] 056-02-PLAN.md — Route-table contract, planner truth, and cross-shard reject.
- [x] 056-03-PLAN.md — Semantic storage handoff and dynamic scope birth.
- [x] 056-04-PLAN.md — Journal lineage, restart, and lawful failover.
- [x] 056-05-PLAN.md — YAML materialization and startup preflight.
- [x] 056-06-PLAN.md — Simulator stage sync and runtime evidence.
- [x] 056-07-PLAN.md — Fixture, benchmark, validation, and closeout sync.

**Scope:** Reuse `.planning/phases/056-HJMT-storage- aggregator/` as the only
Phase 056 directory. Do not create another `056` folder. `056-TODO.md`
remains the canonical backlog and execution authority for the first real
sharded runtime slice: process topology, committed route-table truth,
YAML-driven runtime configuration, semantic runtime-to-storage handoff,
journal-lineage restart safety, lawful failover, startup preflight, and
simulator evidence. Previously future-only design wording in `056-TODO.md` is
live phase scope, not a deferred sidecar backlog. Public checkpoint
publication and final readiness claims remain handoff work for later phases.

## Phase 057: HJMT Multi Aggregator

**Goal:** Turn the lawful sharded runtime lineage from Phase 056 into public
checkpoint truth by making root-of-shard-roots publication, root-generation
transitions, byte-identical carry-forward under partial failure,
join-as-standby, join-as-owner after route generation `N+1`,
route-generation-bound shard transfer, validator/watcher acceptance, and
first-seen scope continuity executable without reopening runtime routing
truth, planner truth, or storage truth and without creating a duplicate phase
folder.
**Requirements**: `057-G1` through `057-G11` from `057-TODO.md`.
**Depends on:** Phase 056 closeout; Phase 046 remains paused separately and is
not advanced by this registration.
**Plans:** 7 numbered plans (`057-01` through `057-07`) plus
`057-SOURCE-AUDIT.md`, `057-TEST-SPEC.md`, and `057-TESTS-TASKS.md`.
**Status**: Completed on 2026-06-14 in the existing
`.planning/phases/057-HJMT-multi-aggregator/` folder; `057-01-SUMMARY.md`
through `057-07-SUMMARY.md` close the publication-contract, layered-proof,
`SIM-5A7S-PUB` publication-integration, lawful-transition,
validator/watcher continuity, original closeout matrix, and renormalized
authority-guardrail continuation slices, and no active Phase 057 execution
lane remains.
**Added**: 2026-06-13
**Directory**: `.planning/phases/057-HJMT-multi-aggregator/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/057-HJMT-multi-aggregator/057-TODO.md`
- `.planning/phases/057-HJMT-multi-aggregator/057-CONTEXT.md`
- `.planning/phases/057-HJMT-multi-aggregator/057-SOURCE-AUDIT.md`
- `.planning/phases/057-HJMT-multi-aggregator/057-TEST-SPEC.md`
- `.planning/phases/057-HJMT-multi-aggregator/057-TESTS-TASKS.md`
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`

**Success Criteria**:

1. Phase 057 is discoverable in the active milestone without creating a
   second `057` directory.

2. The existing `.planning/phases/057-HJMT-multi-aggregator/` folder remains
   the only canonical phase root, `057-TODO.md` remains the backlog and source
   authority, and `057-CONTEXT.md`, `057-SOURCE-AUDIT.md`,
   `057-01-PLAN.md` through `057-07-PLAN.md`, `057-TEST-SPEC.md`, and
   `057-TESTS-TASKS.md` define the canonical execution packet; `057-07`
   is the explicit renormalized continuation of the earlier superseded draft,
   not a parallel scratchpad lane.

3. Publication truth is defined by one canonical ordered `ShardRootLeafV1`
   set and one canonical `CheckpointPublicationV1` digest story, with
   prior-root continuity, monotonicity, and two-layer proof composition
   preserved across root-generation transitions.

4. Join-as-standby, join-as-owner after route generation `N+1`,
   route-generation-bound shard transfer, and byte-identical carry-forward are
   all proven on YAML-loaded topology transitions without reopening runtime,
   planner, or storage authority or silently rerouting failed shards.

5. Validator and watcher evidence bind to the same publication digest, and
   first-seen scope birth remains continuous from Phase 056 scope lineage
   through shard-leaf and public-checkpoint outputs.

Plans:

- [x] `057-01-PLAN.md` — root generation, shard leaf, and checkpoint
  publication contracts

- [x] `057-02-PLAN.md` — two-layer proof composition and historical
  compatibility

- [x] `057-03-PLAN.md` — `SIM-5A7S-PUB` publication integration and trace
  packet

- [x] `057-04-PLAN.md` — join, transfer, carry-forward, and crash recovery
- [x] `057-05-PLAN.md` — validator/watcher binding, scope continuity, and
  scenario sync

- [x] `057-06-PLAN.md` — fixture, benchmark, validation, and planning-state
  closeout

- [x] `057-07-PLAN.md` — canonical-authority guardrails and renormalized
  closeout sync

**Scope:** Reuse `.planning/phases/057-HJMT-multi-aggregator/` as the only
Phase 057 directory. Do not create another `057` folder. `057-TODO.md`
remains the canonical backlog and execution authority, and future-only wording
in the referenced HJMT design packet is live Phase 057 scope. The phase owns
root-of-shard-roots publication, root-generation transitions, carry-forward,
join and migration behavior, validator/watcher acceptance, and public
continuity for first-seen scope birth. Phase 057 publishes the output of the
runtime, planner, and storage layers inherited from Phase 056; it does not
replace those authority paths.

## Phase 058: HJMT Benchmarks

**Goal:** Close the HJMT sharding-upgrade evidence, benchmark, and readiness
ledger without inventing new routing or publication semantics and without
creating a duplicate phase folder.
**Requirements**: `058-G1` through `058-G13` from `058-TODO.md`.
**Depends on:** Phase 056 runtime evidence and Phase 057 publication evidence;
Phase 046 remains paused separately and is not advanced by this registration.
**Plans:** `058-CONTEXT.md`, `058-SOURCE-AUDIT.md`,
`058-EVIDENCE-LEDGER.md`, `058-01-PLAN.md` through `058-07-PLAN.md`,
`058-TEST-SPEC.md`, and `058-TESTS-TASKS.md` now freeze execution order, gate ownership,
live-versus-successor-versus-proposed path honesty, release-mode simulator
discipline, benchmark-home ownership, fixture closure, and the final readiness
verdict rules.
**Status**: Completed as of 2026-06-16 in the existing
`.planning/phases/058-HJMT-benchmarks/` folder. `058-01-SUMMARY.md` through
`058-07-SUMMARY.md` are summary-backed and `058-SUMMARY.md` records the final
integrated-upgrade closeout state: the shared-proof report is frozen, the
exact bucket-commit and compatibility-equivalence artifacts are landed,
`crates/z00z_storage/outputs/settlement/` is the one canonical measured
archive home, and Appendix C standalone artifacts `C-04`, `C-14`, and
`C-16` are closed on exact owner homes.
**Added**: 2026-06-14
**Directory**: `.planning/phases/058-HJMT-benchmarks/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/058-HJMT-benchmarks/058-TODO.md`
- `docs/tech-papers/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/Z00Z-HJMT-Fixture-Checklist.md`
- `docs/tech-papers/Z00Z-HJMT-Design.md`

**Success Criteria**:

1. Phase 058 is discoverable in the active milestone without creating a
   second `058` directory.

2. The existing `.planning/phases/058-HJMT-benchmarks/` folder remains the
   only canonical phase root, and `058-TODO.md` remains the backlog authority
   while the new context, source-audit, numbered-plan, and test-packet
   artifacts schedule execution without replacing it.

3. Phase 058 closes evidence, benchmark, fixture-family, and readiness gates
   `058-G1` through `058-G13` before any release-ready verdict is claimed.

4. Phase 058 verifies and rates the inherited runtime and publication system
   from Phases 056 and 057, but does not re-own or redefine those semantics.

**Scope:** Reuse `.planning/phases/058-HJMT-benchmarks/` as the only Phase 058
directory. Do not create another `058` folder. `058-TODO.md` remains the
canonical backlog and execution authority, while `058-CONTEXT.md`,
`058-SOURCE-AUDIT.md`, `058-01-PLAN.md` through `058-07-PLAN.md`,
`058-TEST-SPEC.md`, and `058-TESTS-TASKS.md` freeze execution order and
acceptance mapping. The phase owns evidence closure, benchmark closure,
fixture-family closure, and final readiness-gate discipline for the HJMT
upgrade; it does not invent new routing semantics from Phase 056 or new
publication semantics from Phase 057.

## Phase 059: Core Upgrade

**Goal:** Execute the canonical Asset/Voucher/Right core-upgrade packet on the
pre-existing `.planning/phases/059-Core-Upgrade/` folder without creating a
duplicate phase directory or a duplicate semantic authority layer.
**Requirements**: Use `059-TODO.md`, `059-CONTEXT.md`, the referenced corpus,
`059-SOURCE-AUDIT.md`, `059-TEST-SPEC.md`, `059-TESTS-TASKS.md`, and
`059-01-PLAN.md` through `059-10-PLAN.md` as the executable Phase 059
authority set.
**Depends on:** Phase 058; Phase 046 remains paused separately and is not
advanced by Phase 059 execution.
**Plans:** 10 ordered numbered plans exist in
`.planning/phases/059-Core-Upgrade/`: `059-01-PLAN.md` through
`059-10-PLAN.md`, plus `059-SOURCE-AUDIT.md`, `059-TEST-SPEC.md`, and
`059-TESTS-TASKS.md`.
**Status**: Executing as of 2026-06-18 on the existing
`.planning/phases/059-Core-Upgrade/` folder only. `059-TODO.md` remains the
canonical planning inventory, `059-CONTEXT.md` decisions `D-01` through `D-72`
are covered by the numbered plan packet, `059-01-SUMMARY.md` closed the
source-audit packet, `059-02-SUMMARY.md` closed the canonical core vocabulary
packet, `059-03-SUMMARY.md` closed the genesis policy/voucher/publication
packet, `059-04-SUMMARY.md` closed the storage voucher leaf/proof packet,
`059-05-SUMMARY.md` closed typed storage object deltas or conservation or
lifecycle or fee-boundary closure, `059-06-SUMMARY.md` closed runtime object
package carriage plus validator or watcher or rollup verdict surfacing,
`059-07-SUMMARY.md` closed wallet typed object inventory/persistence/
quarantine, `059-08-SUMMARY.md` closed wallet object scan/package builder/RPC/
backup integration, `059-09-SUMMARY.md` closed in-place simulator object lanes
plus Alice/Bob/Charlie evidence, and the active execution lane is
`059-10-PLAN.md`.
**Added**: 2026-06-16
**Directory**: `.planning/phases/059-Core-Upgrade/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/059-Core-Upgrade/059-TODO.md`
- `.planning/GSD-Workflow.md`

**Success Criteria**:

1. Phase 059 is discoverable in the active milestone without creating a
   second `059` directory.

2. The existing `.planning/phases/059-Core-Upgrade/` folder remains the only
   canonical phase root and `059-TODO.md` remains the canonical planning
   inventory now that the phase-local context and numbered plan packet are
   created.

3. The numbered plan packet preserves the canonical task meaning from
   `059-TODO.md`, schedules execution sequentially across dependent waves, and
   does not introduce a parallel layer or concept drift.

**Scope:** Reuse `.planning/phases/059-Core-Upgrade/` as the only Phase 059
directory. Do not create another `059` folder. Treat `059-TODO.md` and the
referenced corpus as live scope authority for the Asset/Voucher/Right core
upgrade across core, storage, wallets, simulator, runtime, validators,
watchers, docs, and tests. `059-01` froze the source-audit and canonical
module-path rule so `assets/actions/policies/rights/vauchers` end on one owner
path per concept instead of parallel semantic layers. `059-02` then landed the
first live canonical `z00z_core::{actions,policies,rights,vauchers}` roots
while keeping `assets/*` as compatibility facades instead of a second owner
path. `059-03` then widened the single existing genesis boundary to publish
deterministic policy and bootstrap-voucher artifacts plus manifest-v2 digests,
while storage and simulator consumed the same canonical Stage 1 packet instead
of a parallel export path. `059-04` then extended the existing storage
settlement leaf-family lane in place with voucher record or proof or batch or
cache semantics, one-root documentation, and durable reload or fuzz coverage
instead of a parallel voucher tree. `059-05` then extended the same storage
execution path with typed object deltas, lifecycle-aware voucher transitions,
conservation rules, fee-support separation, durable delta-history replay, and
scheduler-aware commit routing instead of a parallel mixed-object journal.
`059-06` then carried the same object package contract through runtime
admission, validator verdicts, watcher alerts, and rollup reject surfacing
instead of creating runtime-local object truth. `059-07` then widened the
existing wallet inventory seam with typed voucher/right payload persistence,
policy-aware quarantine, and unified object-id projection instead of a parallel
wallet object store. `059-08` then extended the same wallet/RPC/backup path
with family-aware object package preview/build, explicit object lifecycle RPC,
cash-only asset guards, and typed backup/import hardening instead of a second
wallet object protocol. `059-09` then extended the same `scenario_1` simulator
home with one object-flow matrix, tri-actor Alice/Bob/Charlie evidence,
`voucher_flow.json` packet sync, policy/voucher Stage 1 artifact verification,
and negative reject artifacts instead of a parallel simulator or packet path.
Plans:

- [x] `059-01-PLAN.md` - Source audit and live target freeze.
- [x] `059-02-PLAN.md` - Core object vocabulary, policy descriptors, and action pools.
- [x] `059-03-PLAN.md` - Core genesis policies, vouchers, and publication.
- [x] `059-04-PLAN.md` - Storage voucher leaf family and proof semantics.
- [x] `059-05-PLAN.md` - Typed object deltas, conservation, lifecycle, and fee boundary.
- [x] `059-06-PLAN.md` - Runtime admission, validator verdicts, and watcher alerts.
- [x] `059-07-PLAN.md` - Wallet typed object inventory and persistence.
- [x] `059-08-PLAN.md` - Wallet scan, package builder, RPC, and backup.
- [x] `059-09-PLAN.md` - Simulator object lanes and Alice/Bob/Charlie evidence.
- [ ] `059-10-PLAN.md` - Cross-crate test closure, docs, and final verification. Active execution lane.

## Phase 060: Gaps Closing

**Goal:** Close the remaining cross-workstream gaps translated from
`060-TODO.md` across `z00z_core`, HJMT topology or evidence, verification
gates, and wallet MVP object semantics without creating a duplicate phase
folder, duplicate codebase logic, or a parallel authority layer.
**Requirements**: None. `060-TODO.md` remains the canonical execution
authority; the Phase 060 packet intentionally uses no invented `060-REQ-*`
layer.
**Depends on:** Phase 059 closeout; Phase 046 remains paused separately and is
not advanced by this planning packet.
**Plans:** All 15 numbered plans are summary-backed complete. `060-14-SUMMARY.md`
closes the broad review-context-only reopen for the superseded overlap, and
`060-15-SUMMARY.md` closes the actual narrowed MVP packet on the same tree.
**Status**: Added 2026-06-19 on the existing
`.planning/phases/060-Gaps-Closing/` directory and completed 2026-06-23 on
that same folder only. `060-TODO.md`, `060-CONTEXT.md`, `060-TEST-SPEC.md`,
`060-TESTS-TASKS.md`, and `060-01-PLAN.md` through `060-15-SUMMARY.md` form
the canonical closed packet. `060-10-SUMMARY.md` keeps `aggregator_owned` as
the production default, `060-11-SUMMARY.md` leaves any future full
`z00z-verification-orchestrator` rerun operator-owned manual work,
`060-12-SUMMARY.md` through `060-13-SUMMARY.md` close the storage and
wallet/object reopens, `060-14-SUMMARY.md` closes the broad overlapping MVP
subset as review context only, and `060-15-SUMMARY.md` closes refund/source
binding, truthful one-plane object issue/create, live incomplete publication
states, and monotonic `delegate_right` on the current tree. The final broad
`cargo test --release` rerun is green, including the long
`z00z_simulator/tests/scenario_1/main.rs` lane on the same canonical rerun.
**Added**: 2026-06-19
**Directory**: `.planning/phases/060-Gaps-Closing/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/060-Gaps-Closing/060-TODO.md`
- `.planning/phases/060-Gaps-Closing/060-CONTEXT.md`
- `.planning/phases/060-Gaps-Closing/060-TEST-SPEC.md`
- `.planning/phases/060-Gaps-Closing/060-TESTS-TASKS.md`
- `.planning/phases/060-Gaps-Closing/060-TZ1.md`
- `.planning/phases/060-Gaps-Closing/060-TZ2.md`
- `.planning/phases/060-Gaps-Closing/060-z00z-verification-report.md`
- `.planning/GSD-Workflow.md`

**Success Criteria**:

1. Phase 060 remains discoverable on the pre-existing folder only with no
   duplicate `060` directory or second authority surface.

2. Every canonical task from `A1..A5`, `B1..B6`, `C1..C4`, and `D1..D5` is
   routed into the numbered plan packet without creating a parallel
   requirements layer.

3. Every auto-task verify block preserves bootstrap-first validation, relevant
   targeted tests, `cargo test --release` when relevant, repeated
   `/GSD-Review-Tasks-Execution` reruns, and `/z00z-git-versioning` for
   commit workflow.

**Scope:** Reuse `.planning/phases/060-Gaps-Closing/` as the only Phase 060
directory. Do not create another `060` folder or a second semantic layer. Keep
`060-TODO.md` as the canonical authority, use `060-CONTEXT.md` as the transfer
mirror, use `060-TEST-SPEC.md` and `060-TESTS-TASKS.md` as the phase-local
verification packet, and execute the numbered plan chain exactly on the
existing phase folder. Keep `aggregator_owned` as the HJMT production default
until the explicit B6 A/B gate passes, keep wallet semantics on one typed
authority plane, and keep verification performance work subordinate to the same
evidence contract.

Plans:

- [x] `060-01-PLAN.md` — Docs Gate Posture And `ZINV` Traceability.
- [x] `060-02-PLAN.md` — Canonical Bootstrap Authority Freeze.
- [x] `060-03-PLAN.md` — HJMT Process Model And YAML Shard Mapping Contract.
- [x] `060-04-PLAN.md` — Rights Owner Move, Shim Demotion, And Dual-Authority YAML Closure.
- [x] `060-05-PLAN.md` — HJMT Decommission Coverage And `3A7S -> 2A7S -> 5A7S` Scenario.
- [x] `060-06-PLAN.md` — Supply-Chain Review Records And Vet Trust Closure.
- [x] `060-07-PLAN.md` — Wallet MVP Profile Catalog And One-Plane Projection Semantics.
- [x] `060-08-PLAN.md` — `validator_mandate_lock_v1` Contract And Fail-Closed Profile Coverage.
- [x] `060-09-PLAN.md` — Adversarial High-Finding Closure And Count Reconciliation.
- [x] `060-10-PLAN.md` — HJMT Measurement Lanes And A/B Rerun Packet. Closed on `060-10-SUMMARY.md` with the honest A/B packet and the default kept at `aggregator_owned`.
- [x] `060-11-PLAN.md` — Verification-Pipeline Performance And Final Closure Reruns. Closed on `060-11-SUMMARY.md` as the agent-owned optimization and manual-closeout handoff slice; any future full orchestrator rerun is operator-owned manual work.
- [x] `060-12-PLAN.md` — HJMT Core Storage Shard Truth Closure. Closed on `060-12-SUMMARY.md` after the green mandatory bootstrap rerun, the preserved targeted release packet, and a green full `cargo test --release` rerun on the current tree.
- [x] `060-13-PLAN.md` — Prepared Tx Balance, Voucher Conservation, And FeeEnvelope Coverage. Closed on `060-13-SUMMARY.md` after the targeted reject-path packet went green on the current tree and the already-green broad `cargo test --release` evidence remained valid on the same live code tree.
- [x] `060-14-PLAN.md` — Refund Binding, One-Plane Object Issue/Create, And Incomplete Verdict Coverage. Closed on `060-14-SUMMARY.md` as broad supplemental review context only for the superseded overlapping MVP subset; no duplicate implementation lane was executed.
- [x] `060-15-PLAN.md` — MVP Closeout For Refund Source Binding, One-Plane Issue/Create, Incomplete Publication States, And Monotonic Right Delegation. Closed on `060-15-SUMMARY.md` after the green mandatory bootstrap rerun and the green final `cargo test --release` closeout on the current tree.

## Phase 061: Wallet Refactoring

**Goal:** Execute the wallet source-tree rename and flattening packet on the
existing `.planning/phases/061-Wallet-Refactoring/` folder without creating a
duplicate phase directory, widening scope beyond `crates/z00z_wallets/src`, or
introducing a parallel module-authority layer.
**Requirements**: None. `061-TODO.md` remains the canonical requirements and
rename authority; `061-CONTEXT.md` is the derived execution mirror.
**Depends on:** Phase 060 closeout; Phase 046 remains paused separately and is
not advanced by this packet.
**Plans:** Ten ordered execution slices on the existing Phase 061 folder;
`061-01-SUMMARY.md` through `061-10-SUMMARY.md` are present and the phase is
complete.
**Status**: Completed 2026-06-24 on the existing
`.planning/phases/061-Wallet-Refactoring/` directory only. No new phase folder
was created. `061-TODO.md` remains the canonical authority, while
`061-CONTEXT.md` and `061-01-PLAN.md` through `061-10-PLAN.md` remain the
execution packet record. The mandatory `bootstrap_tests.sh` gate reran green
first, future or target design wording across `061-TODO.md` and the referenced
corpus was treated as live mandatory scope, `061-01-SUMMARY.md` through
`061-09-SUMMARY.md` record the ordered preflight through tx/claim/stealth close
steps, and `061-10-SUMMARY.md` closes the remaining domains/service-test/egui
work, proves the one-level source-tree contract, preserves the flat
`crates/z00z_wallets/docs/*` authority for key/wallet/domain docs plus
`crates/z00z_wallets/docs/egui_views.tar.gz`, repairs the cross-crate stale
`test_live_guardrails` tx-proof anchor, records the wallet-config env isolation
fix for `test_rpc_reunlock_verify`, records the simulator fixture-cache
test-serialization validation unblock, and records green release validation on
the current tree.
**Added**: 2026-06-23
**Directory**: `.planning/phases/061-Wallet-Refactoring/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/061-Wallet-Refactoring/061-TODO.md`
- `.planning/phases/061-Wallet-Refactoring/061-CONTEXT.md`
- `crates/z00z_wallets/🔐-разбор-WLT.md`
- `crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md`
- `.planning/GSD-Workflow.md`

**Success Criteria**:

1. No Rust file remains deeper than `src/<domain>/file.rs` when Phase 061
   closes.

2. `adapters::rpc`, `db::redb_wallet_store`, `services::WalletService`, and
   other caller-visible wallet facades remain stable while implementations move.

3. `.wlt` labels, schema versions, KDF labels, and hash-domain strings remain
   unchanged throughout the structural refactor.

4. The existing `.planning/phases/061-Wallet-Refactoring/` folder remains the
   only canonical phase root, and the numbered packet executes without
   introducing a second planning or module authority.

**Scope:** Reuse `.planning/phases/061-Wallet-Refactoring/` as the only Phase
061 directory. Do not create another `061` folder. Treat `061-TODO.md` as the
canonical live execution authority for wallet source-tree rename, flattening,
compatibility-facade preservation, duplicate/stale file retirement, and
naming-boundary cleanup across `crates/z00z_wallets/src` only. Future or target
design wording in the Phase 061 corpus is mandatory current scope, not a later
backlog, and every shipped behavior must keep one canonical module or function
path with no parallel authority layer.

Plans:

- [x] `061-01-PLAN.md` — Preflight Audit, Anchor Freeze, And Drift Reconciliation. Closed on `061-01-SUMMARY.md` after the green mandatory bootstrap rerun, live-row reconciliation for stale service-wrapper and stealth planner paths, release-only verify normalization across the packet, and a green `cargo check --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-02-PLAN.md` — Shared DB Contract Flattening And Wallet Store Crypto Rename. Closed on `061-02-SUMMARY.md` after the flat `src/db/*.rs` cutover, the neutral `wallet_store_crypto*` live path rename, zero stale Rust references to `db::redb_wallet_crypto`, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-03-PLAN.md` — RedB Store Facade Move And Persistence Anchor Preservation. Closed on `061-03-SUMMARY.md` after the flat `src/redb_store/*.rs` backend move, preserved `db::redb_wallet_store` facade wiring, canonical schema-home relocation to `crates/z00z_wallets/schemas/redb-schema.yaml`, updated anchor-sensitive source-inspection tests, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-04-PLAN.md` — RPC Facade Support, Logging, Types, And Wallet Config Anchor Move. Closed on `061-04-SUMMARY.md` after the flat `src/rpc/*.rs` support move, preserved `adapters::rpc` facade wiring, canonical wallet-config relocation to `crates/z00z_wallets/config/wallet_config.yaml`, the repaired key-RPC env anchor, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-05-PLAN.md` — WalletService Internal Flattening And Service Path Preservation. Closed on `061-05-SUMMARY.md` after the flat `src/services/*.rs` service-shard move, preserved `services::WalletService` and `services::AppService` facades, the `R2` store-name shortenings to `wallet_service_store_open_source.rs` and `wallet_service_store_export_pack.rs`, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-06-PLAN.md` — RPC Method Implementation Flattening And Helper Renames. Closed on `061-06-SUMMARY.md` after the flat `src/rpc/method_*.rs` and `src/rpc/test_*.rs` cutover, preserved `adapters::rpc` registration wiring, the helper rename closure to `method_asset_*`, `method_tx_*`, and `method_tx_support.rs`, the replay-audit script path repair, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-07-PLAN.md` — Receiver, Persistence, And Security Vault Flattening. Closed on `061-07-SUMMARY.md` after the flat `src/receiver/*.rs`, `src/persistence/*.rs`, and `src/security/{vault.rs,vault_*.rs}` cutover, the `claim_own` -> `stealth_ownership_check` and `nfc_utils` -> `nfc_ndef` rename closure, the password asset move to `config/security/*`, duplicate receipt or scan shim retirement with live-reference proof, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-08-PLAN.md` — Key Tree Flattening And Seed Or BIP Anchor Preservation. Closed on `061-08-SUMMARY.md` after the flat `src/key/*.rs` cutover, the key-doc relocation to the canonical flat `crates/z00z_wallets/docs/*` home, the key include-anchor and source-inspection test rewrites, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-09-PLAN.md` — Tx, Claim, Stealth, Wallet, Backup, And Chain Leaf Flattening. Closed on `061-09-SUMMARY.md` after the flat leaf-domain cutover, the wallet-guide relocation to `crates/z00z_wallets/docs/WALLET-GUIDE.md`, the canonical `claim_tx_hashing` and `claim_tx_statement` helper split, the repaired `asset_selector_multi.rs` test-path anchor after the first bootstrap failure, a green `cargo check --release -p z00z_wallets --all-targets --all-features`, and a green `cargo test --release -p z00z_wallets --all-targets --all-features`.
- [x] `061-10-PLAN.md` — Domains, Egui, Remaining Cleanup, And Final Tree Closeout. Closed on `061-10-SUMMARY.md` after the final domains/service-test/egui cleanup, the domain snapshot move to `crates/z00z_wallets/docs/domains_snapshot.txt`, the egui reference bundle move to `crates/z00z_wallets/docs/egui_views.tar.gz`, the `wallet_tab_stacking.rs -> wallet_tab_staking.rs` typo closure, the retirement of `app_settings_tab_2.rs` after live-reference proof, the repaired cross-crate `test_live_guardrails` tx-proof anchor, the wallet-config env isolation fix for `test_rpc_reunlock_verify`, the simulator fixture-cache test-serialization fix that restored a green broad release rerun, green bootstrap reruns, green targeted release reruns, a green final `cargo test --release`, and the completed one-level wallet tree audit.

## Phase 062: Gaps Closing 2

**Goal:** Plan and execute the second gap-closing packet on the existing
`.planning/phases/062-Gaps-Closing-2/` folder without creating a duplicate
phase directory, losing the existing audit corpus, or introducing a parallel
planning or implementation authority.
**Requirements**: None. `062-TODO.md` is the canonical planning authority and
task inventory for Phase 062. It fixes `TASK-001` through `TASK-125` and the
required grouped plan ids `PLAN-062-G01` through `PLAN-062-G27`.
**Depends on:** Phase 061 closeout; Phase 046 remains paused separately and is
not advanced by Phase 062 execution.
**Plans:** 27 grouped plans in the canonical execution packet.
`062-01` through `062-27`
are summary-backed complete on the current tree; no active `062` execution
lane remains.
**Status**: Complete as of 2026-06-27 on the existing
`.planning/phases/062-Gaps-Closing-2/` directory only. No new phase folder was
created. `062-TODO.md` remains the normative authority, the full planning
packet now exists as `062-CONTEXT.md`, `062-COVERAGE.md`, `062-TEST-SPEC.md`,
`062-TESTS-TASKS.md`, and `062-01-PLAN.md` through
`062-27-PLAN.md`, and `062-01-SUMMARY.md` through `062-27-SUMMARY.md` record
the current execution proof; graph data was not used as implementation
evidence; future-only or target design wording from `062-TODO.md` and the
referenced corpus stays live mandatory scope for the current code; `062-21`
closed fail-closed thin cache uncertainty by clearing relevant candidate pins
and forcing thick mode until explicit repin, added live thin-versus-thick
equivalence and fallback recovery tests, closed typed negative thin RPC errors,
bounded thin logging summaries to non-secret package-shape metadata, and kept
the mandatory bootstrap reruns, focused wallet release validation, and broad
release rerun green; `062-22` closed the canonical final closeout register and
residual-gap ledger, bounded future transport and field-native pack wording on
the live docs, cross-crate residual guardrail tests for the planning and wallet
surfaces, and the green focused wallet or storage and broad release reruns;
`062-23` closed the canonical wallet `ChainClient` local node simulation path,
typed local block or transaction or network errors, and drift-free `062-23`
plan wording with green focused wallet and broad release reruns; `062-24`
closed the canonical wallet broadcast retry or confirmation persistence seam,
drift-free `062-24` plan wording, scoped broadcast test-name cleanup, and
green focused wallet plus broad release reruns; `062-25` closed the canonical
wallet fee-rate source seam, drift-free `062-25` plan wording, scoped
fee-estimator test-name cleanup, and green focused wallet plus broad release
reruns; `062-26` closed the canonical wallet remote-worker seam, drift-free
`062-26` plan wording, future-only touched-seam wording cleanup, scoped
worker test-name cleanup, and green focused wallet plus broad release reruns;
`062-27` closed the canonical wallet spend-policy seam for `TASK-125`, added
targeted RPC policy coverage, removed stale live-scope and dead-path drift
from the Phase 062 plan packet, cleaned touched wallet contract wording,
removed the simulator `runner.rs` full-suite flake by serializing the
stateful unit tests, and ended with green focused, feature-gate, and broad
sequential release reruns; no active `062` execution lane remains and Phase
046 stays paused after `046-04`.
**Added**: 2026-06-24
**Directory**: `.planning/phases/062-Gaps-Closing-2/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/062-Gaps-Closing-2/062-TODO.md`
- `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md`
- `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`
- `.planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md`
- `.planning/phases/062-Gaps-Closing-2/062-TESTS-TASKS.md`
- `.planning/phases/062-Gaps-Closing-2/062-01-PLAN.md`
- `.planning/phases/062-Gaps-Closing-2/062-01-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-02-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-03-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-05-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-06-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-07-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-08-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-09-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-10-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-11-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-12-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-13-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-14-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-15-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-16-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-17-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-18-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-19-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-20-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-21-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-22-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-23-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-24-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-25-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-26-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/062-27-PLAN.md`
- `.planning/phases/062-Gaps-Closing-2/062-27-SUMMARY.md`
- `.planning/phases/062-Gaps-Closing-2/GAPS.md`
- `.planning/phases/062-Gaps-Closing-2/asset-only.md`
- `.planning/phases/062-Gaps-Closing-2/HJMT-REPORT.md`
- `.planning/phases/062-Gaps-Closing-2/HJMT-RAID -Sharding.md`
- `.planning/phases/062-Gaps-Closing-2/HJMT-Sharding-Storage-Techpaper.md`
- `.planning/phases/062-Gaps-Closing-2/HJMT-структуры.md`
- `.planning/phases/062-Gaps-Closing-2/Z00Z-Thin-Transaction-Mode.md`

**Success Criteria**:

1. Phase 062 is discoverable in the active milestone without creating a second
   `062` directory.

2. The existing `.planning/phases/062-Gaps-Closing-2/` folder remains the
   only canonical phase root, and `062-TODO.md` remains the canonical
   normative authority while the planning packet and summary artifacts stay on
   the same folder and same current tree.

3. `PLAN-062-G23` through `PLAN-062-G27` close the local node simulation,
   broadcast lifecycle persistence, live fee-rate sourcing, remote scan worker
   local simulation, and wallet spend-policy enforcement slices on one
   canonical wallet path with green bootstrap and green wallet-crate release
   validation.

4. Any later planning wave preserves the exact task inventory `TASK-001`
   through `TASK-125`, the grouped-plan contract `PLAN-062-G01` through
   `PLAN-062-G27`, the current workspace path map, and the no-alternate-task
   namespace rule.

**Scope:** Reuse `.planning/phases/062-Gaps-Closing-2/` as the only Phase 062
directory. Do not create another `062` folder. Treat `062-TODO.md` as the
canonical source-audited planning authority for the second gap-closing packet
across settlement-root authority, checkpoint/publication evidence, wallet
lifecycle and receive flows, typed errors, object and policy work, local
distributed HJMT simulation, thin signed-index transaction mode, and the final
full-system closeout register. When planning starts, generate grouped plans
`PLAN-062-G01` through `PLAN-062-G27` from `TASK-001` through `TASK-125` on
current workspace paths only, with no alternate task namespaces and no
graph-derived implementation evidence.

## Phase 063: Core Update

**Goal:** Execute the existing `.planning/phases/063-Core-Update/` packet as
the canonical core-update cleanup lane for bootstrap authority, genesis
execution contracts, object-family semantics, YAML ownership, test ownership,
docs truth, and support-surface layout without creating a duplicate phase
directory or a second implementation authority.
**Requirements**: None. `063-TODO.md` is the canonical planning authority and
normative task inventory for Phase 063. `063-core-examples.md` is a mandatory
implementation source that defines the required example envelope and payload
families for the live core-update scope, and future-only or target design
wording in the Phase 063 corpus is treated as live mandatory scope for the
current code.
**Depends on:** Phase 062 closeout; Phase 046 remains paused separately and is
not advanced by Phase 063 planning.
**Plans:** 13/13 plans executed

- [x] 063-01-PLAN.md
- [x] 063-02-PLAN.md
- [x] 063-03-PLAN.md
- [x] 063-04-PLAN.md
- [x] 063-05-PLAN.md
- [x] 063-06-PLAN.md
- [x] 063-07-PLAN.md
- [x] 063-08-PLAN.md
- [x] 063-09-PLAN.md
- [x] 063-10-PLAN.md
- [x] 063-11-PLAN.md
- [x] 063-12-PLAN.md
- [x] 063-13-PLAN.md

folder: `063-01-PLAN.md` through `063-13-PLAN.md`.
**Status**: Completed 2026-06-29 on the existing
`.planning/phases/063-Core-Update/` directory only. No new phase folder was
created. `063-CONTEXT.md` and the numbered planning packet remained the phase
authority throughout execution, `063-01-SUMMARY.md` through
`063-12-SUMMARY.md` close the first twelve slices, and
`063-13-SUMMARY.md` closes the support-surface flattening and
crate-responsibility lane that finished the packet. `Cargo.toml` now owns flat
support paths with explicit `cli` feature gating plus `autobins = false`,
`autoexamples = false`, and `autobenches = false`; no nested support files,
no nested support `README.md`, and no example-local YAML authority survive;
the mandatory bootstrap gate reran green on an isolated target directory; the
focused `z00z_core` release validations are green; three timed-out
`/GSD-Review-Tasks-Execution` attempts were recorded with manual-review
fallback; truthful `--test` manifest-golden verify commands remain in the
Phase 063 authority docs; and no active Phase 063 execution lane remains.
**Added**: 2026-06-28
**Directory**: `.planning/phases/063-Core-Update/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/063-Core-Update/063-TODO.md`
- `.planning/phases/063-Core-Update/063-core-examples.md`
- `.planning/phases/063-Core-Update/063-CONTEXT.md`
- `.planning/phases/063-Core-Update/063-01-PLAN.md`
- `.planning/phases/063-Core-Update/063-13-PLAN.md`
- `.planning/phases/063-Core-Update/063-13-SUMMARY.md`

**Success Criteria**:

1. Phase 063 is discoverable in the active milestone without creating a second
   `063` directory.

2. The existing `.planning/phases/063-Core-Update/` folder remains the only
   canonical phase root, and Phase 063 registration does not create a
   duplicate folder or a parallel authority layer.

3. `063-TODO.md` remains the canonical planning authority,
   `063-core-examples.md` remains a mandatory implementation source, and
   `063-CONTEXT.md` plus `063-01-PLAN.md` through `063-13-PLAN.md` define the
   executable packet without creating a second task authority or second
   bootstrap path.

4. Phase 063 planning preserves the current-tree recommendation inventory from
   `063-TODO.md`, records that the current file has no canonical `TASK-NNN`
   rows, keeps current workspace paths authoritative, and avoids graph-derived
   implementation claims.

5. Phase 063 execution treats future-only or target design wording in the
   referenced Phase 063 corpus as live mandatory scope, keeps one canonical
   module or function path per behavior, and advances sequentially from
   `063-01` to `063-13` without introducing parallel code or docs layers.

**Scope:** Reuse `.planning/phases/063-Core-Update/` as the only Phase 063
directory. Do not create another `063` folder. Treat `063-TODO.md` as the
canonical source-audited planning authority and `063-core-examples.md` as a
mandatory implementation source for the core-update scope. Phase 063 executed
the 13 current recommendation headings from `063-TODO.md` as the canonical
inventory on current workspace paths only, kept graph data limited to
topology context instead of implementation evidence, treated future-only or
target design wording in the Phase 063 corpus as live mandatory scope, kept
one canonical module or function path per behavior, and closed with the
canonical `z00z_core::vouchers` owner path, the canonical
`z00z_core::ObjectFamily` semantics vocabulary, the canonical bounded
simulator selector anchors, the canonical `crates/z00z_core/z00z_config/`
root, the canonical `z00z_core::config_paths` helper surface, the canonical
flat `crates/z00z_core/tests/` root, the canonical flat
`crates/z00z_core/{benches,bin,examples}` roots, truthful targeted docs or
support-surface Markdown, and no active Phase 063 lane remaining.

## Phase 064: Gaps Closing 3

**Goal:** Use the existing `.planning/phases/064-Gaps-Closing-3/` packet as
the canonical third gap-closing lane for simulator truth, wallet
local-mutation live paths, RPC wiring truth, and adjacent local-only closeout
work without creating a duplicate phase directory or a second planning
authority.
**Requirements**: None. `064-TODO.md` remains the canonical normative task
authority for Phase 064. The numbered execution packet now exists as
`064-CONTEXT.md`, `064-01-PLAN.md` through `064-05-PLAN.md`,
`064-TEST-SPEC.md`, and `064-TESTS-TASKS.md`, and future-only or target
design wording in the Phase 064 corpus is treated as live mandatory scope for
the current code.
**Depends on:** Phase 063 closeout; Phase 046 remains paused separately and is
not advanced by Phase 064 execution.
**Plans:** 5 planned; `064-01` through `064-05` summary-backed complete; no
active execution lane remains

**Status**: Completed as of 2026-06-30 on the existing
`.planning/phases/064-Gaps-Closing-3/` directory only. No new phase folder
was created; the canonical execution packet now exists as `064-CONTEXT.md`,
`064-01-PLAN.md` through `064-05-PLAN.md`, `064-TEST-SPEC.md`, and
`064-TESTS-TASKS.md`; `064-TODO.md` remains the normative task authority;
`064-01-SUMMARY.md` closes the first simulator truth wave,
`064-02-SUMMARY.md` closes wallet mutation truth and RPC route coverage,
`064-03-SUMMARY.md` closes wallet sensitive surface and typed-object
durability with green targeted wallet/storage release validation,
`064-04-SUMMARY.md` closes storage proof boundaries and runtime adversarial
closure with green targeted storage/rollup/runtime release validation, and
`064-05-SUMMARY.md` closes core truth boundaries and repository guardrails
with green targeted core/wallet validation plus executable boundary audits.
No active Phase 064 lane remains; a post-closeout current-tree revalidation
reran the mandatory bootstrap gate green and reproduced the same broad
workspace blockers with no new Phase 064 drift; the broad workspace
`cargo test --release` rerun still honestly reproduces current-tree
`z00z_core` genesis/config blockers outside the modified `064-05` slice.
**Added**: 2026-06-29
**Directory**: `.planning/phases/064-Gaps-Closing-3/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/064-Gaps-Closing-3/064-TODO.md`
- `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md`
- `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md`
- `.planning/phases/064-Gaps-Closing-3/064-01-SUMMARY.md`
- `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md`
- `.planning/phases/064-Gaps-Closing-3/064-02-SUMMARY.md`
- `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md`
- `.planning/phases/064-Gaps-Closing-3/064-03-SUMMARY.md`
- `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md`
- `.planning/phases/064-Gaps-Closing-3/064-04-SUMMARY.md`
- `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md`
- `.planning/phases/064-Gaps-Closing-3/064-05-SUMMARY.md`
- `.planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md`
- `.planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md`

**Success Criteria**:

1. Phase 064 is discoverable in the active milestone without creating a
   second `064` directory.

2. The existing `.planning/phases/064-Gaps-Closing-3/` folder remains the
   only canonical phase root, and registration does not create a duplicate
   folder or a parallel authority layer.

3. `064-TODO.md` remains the canonical normative task authority while
   `064-CONTEXT.md`, `064-01-PLAN.md` through `064-05-PLAN.md`,
   `064-TEST-SPEC.md`, and `064-TESTS-TASKS.md` define execution order on the
   same folder without creating a parallel authority layer.

**Scope:** Completed on the existing `.planning/phases/064-Gaps-Closing-3/`
folder only. No duplicate `064` folder or second authority layer was created.
`064-TODO.md` stayed the canonical source-audited normative task authority,
future-only or target design wording was treated as live mandatory scope
throughout execution, and no active Phase 064 lane remains.

## Phase 065: Attack Surface

**Goal:** Reuse the existing `.planning/phases/065-Attack-Surface/` folder as
the canonical attack-surface closure phase, with one self-contained
`065-TODO.md` that consolidates the still-relevant findings, closure methods,
gate contracts, inputs, outputs, verification duties, proof expectations, and
legacy disposition without creating a duplicate phase directory or a parallel
TODO set.
**Requirements**: None. `065-TODO.md` remains the normative human-readable
Phase 065 authority. `065-CONTEXT.md` maps the additive residual packet, and
`z00z-verification-report-1.md` through `z00z-verification-report-4.md` are
referenced only as residual evidence anchors for `065-10` through `065-13`.
The old legacy Phase 065 Markdown reports, JSONL catalogs, crate inventories,
and run-local verification snapshots remain retired as required implementation
sources.
**Depends on:** Phase 064 closeout; Phase 046 remains paused separately and is
not advanced by Phase 065 registration.
**Plans:** `065-01-PLAN.md` through `065-13-PLAN.md` are the executable
  packets, and all are summary-backed complete on
  `065-01-SUMMARY.md` through `065-13-SUMMARY.md`.

**Status**: Added 2026-06-30 on the existing
`.planning/phases/065-Attack-Surface/` directory only. No new phase folder was
created. `065-TODO.md` remains the normative human-readable Phase 065
authority, its referenced design and whitepaper corpus stays live mandatory
scope, and `065-CONTEXT.md` mapped additive residual units `VR-10` through
`VR-13` without creating a parallel backlog. `065-10` repaired the canonical
verification-dispatch path for `l0-docs`, `l3-verify-fast`, and
`l4-supply-chain`. `065-11` closed managed toolchain and offline recovery with
green install self-test plus green Miri or Kani or fuzz reruns and honest
Verus or HAX or Tamarin outcomes. `065-12` removed the invalid
aggregator-to-wallet release-test feature edge and restored the required
checkpoint-lineage or storage-determinism release acceptance path. `065-13`
then closed the repeated request or stealth or wallet branch by binding
asset-import claim scope to persisted wallet chain state, centralizing asset
RPC chain metadata, pinning explicit `z00z.payment.request.v1` and
`z00z.receiver.card.v1` hash-policy proofs, preserving the existing public
validated stealth path, and keeping `crates/z00z_crypto/tari/**` untouched.
`bootstrap_tests.sh`, the targeted release suites, `cargo fmt --all --check`,
and the broad `cargo test --release` gate are green. Phase 065 is complete and
no active lane remains.
**Added**: 2026-06-30
**Directory**: `.planning/phases/065-Attack-Surface/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/065-Attack-Surface/065-TODO.md`
- `.planning/phases/065-Attack-Surface/065-CONTEXT.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-1.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-2.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-3.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-4.md`

**Success Criteria**:

1. Phase 065 is discoverable in the active milestone without creating a second
   `065` directory.

2. The existing `.planning/phases/065-Attack-Surface/` folder remains the only
   canonical phase root, and Phase 065 registration does not create a
   duplicate folder or a parallel authority layer.

3. `065-TODO.md` remains the sole canonical human-readable Phase 065
   authority, absorbing the live backlog, the gate inventory, the
   verification and proof obligations, and the legacy disposition mapping
   without recreating the old report set.

4. Phase 065 closes only after all `Open` workstreams and repository-wide
   closure gates recorded in `065-TODO.md` are implemented.

5. The additive verification-remediation packet `065-10` through `065-13`
   closes every still-live residual from verification reports `1-4` through
   one canonical Phase 065 path, without recreating a second authority layer.

## Phase 066: Local Pentest Orchestration

**Goal:** Register the existing `.planning/phases/066-Strix/` folder as the
canonical Phase 066 planning root for local pentest orchestration, keeping one
repository-local authority path for the future Strix-inspired skill, script,
agent, prompt, and tooling work without creating a duplicate phase directory.
**Requirements**: `066-TODO.md` remains the normative human-readable Phase 066
authority and defines the live requirements envelope for local-only pentest
orchestration, safety boundaries, tooling placement, report paths, and
portable Docker flow constraints. The generated numbered plans preserve the
`WS-01` through `WS-14` workstreams as one required GSD plan group each.
**Depends on:** Phase 065 closeout; Phase 046 remains paused separately and is
not advanced by Phase 066 registration.
**Plans:** 14 planned packets: `.planning/phases/066-Strix/066-01-PLAN.md`
through `.planning/phases/066-Strix/066-14-PLAN.md`; `066-01` through `066-14`
are summary-backed complete.

**Status**: Added 2026-07-02 on the existing `.planning/phases/066-Strix/`
directory only. No new phase folder was created. Phase 066 planning is
complete with 14 executable GSD plan packets, coverage audit, and local context
artifact in the same canonical folder. `066-01-SUMMARY.md` through
`066-14-SUMMARY.md` close the scope or safety, tool-root, upstream-provenance,
generic-skill, Z00Z-profile, local-runner, report-schema, bounded-local-DAST,
codex-surface, portable pack-unpack, Docker-isolation, regression-self-test,
execution-prompt, and documentation-or-migration lanes. Phase 066 completed on
2026-07-03 and no active execution packet remains.
**Added**: 2026-07-02
**Directory**: `.planning/phases/066-Strix/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/066-Strix/066-TODO.md`
- `.planning/phases/066-Strix/066-CONTEXT.md`
- `.planning/phases/066-Strix/066-COVERAGE.md`
- `.planning/phases/066-Strix/066-01-SUMMARY.md` through
  `.planning/phases/066-Strix/066-14-SUMMARY.md`

- `.planning/phases/066-Strix/066-01-PLAN.md` through
  `.planning/phases/066-Strix/066-14-PLAN.md`

**Success Criteria**:

1. Phase 066 is discoverable in the active milestone without creating a
   second `066` directory.

2. The existing `.planning/phases/066-Strix/` folder remains the only
   canonical phase root, and Phase 066 registration does not create a
   duplicate folder or a parallel authority layer.

3. `066-TODO.md` remains the sole canonical human-readable Phase 066
   authority, and `066-01-PLAN.md` through `066-14-PLAN.md` map each
   `WS-01` through `WS-14` workstream to exactly one executable plan packet.

4. `066-01-SUMMARY.md` proves that the local-only scope, denylist, safety
   reference, validator, and deterministic CLI tests landed on one canonical
   code path before broader pentest tooling execution continues.

5. `066-02-SUMMARY.md` proves that `tools/penetration/` is now the only
   canonical pentest tool root, with repository-local wrappers and truthful
   status manifests that do not reuse `tools/formal_verification/`.

## Phase 067: Sharded Concensus

**Goal:** Plan the existing `.planning/phases/000/067-Sharded-Concensus/` folder as
the canonical Phase 067 execution root for shard-local aggregator consensus
hardening, secondary-aggregator quorum modeling, `scenario_11` end-to-end local
simulation, downstream validator binding, and the later transport/BFT/Celestia
layer without creating a duplicate phase directory or a parallel authority
path.
**Requirements**: `067-TODO.md` and the expanded `067-verdict.md`
Local-Conformance-Simulation gates remain the normative human-readable Phase
067 authority and implementation contract. They supersede prior working notes,
keep scope local-codebase-only, treat source code or tests or local
configuration as ground truth, preserve future-only or target-design wording in
the Phase 067 corpus as live mandatory scope, record `TASK-NNN` count as zero
because `067-TODO.md` and `067-verdict.md` contain no literal task identifiers,
map the nine base groups `PHASE-0` through `PHASE-8` exactly once to
`067-01-PLAN.md` through `067-09-PLAN.md`, map the ten verdict groups
`VERDICT-LCS-01` through `VERDICT-LCS-10` exactly once to `067-10-PLAN.md`
through `067-19-PLAN.md`, and map `ADDENDUM-067-20` plus
`ADDENDUM-067-21` exactly once to `067-20-PLAN.md` and `067-21-PLAN.md`.
**Depends on:** Phase 066 closeout; Phase 046 remains paused separately and is
not advanced by Phase 067 planning.
**Plans:** 21/21 plans executed:
`.planning/phases/000/067-Sharded-Concensus/067-01-PLAN.md` through
`.planning/phases/000/067-Sharded-Concensus/067-21-PLAN.md`.

**Status**: Completed 2026-07-06 on the existing
`.planning/phases/000/067-Sharded-Concensus/` directory only. No new phase folder
was created. Phase 067 expanded planning is complete with `067-CONTEXT.md`,
`067-COVERAGE.md`, `067-verdict.md`, and 21 executable GSD plan packets in the
same canonical folder. `067-TODO.md` plus `067-verdict.md` remain the
normative human-readable authority; their architecture-spec,
implementation-contract, and Local-Conformance-Simulation wording supersede
prior working notes, while source code, tests, and local configuration remain
ground truth. `067-CONTEXT.md` records the planning fact that literal
`TASK-NNN` rows are absent, the base required units are `PHASE-0` through
`PHASE-8`, the verdict expansion units are `VERDICT-LCS-01` through
`VERDICT-LCS-10`, and the addendum closure units are `ADDENDUM-067-20`
through `ADDENDUM-067-21`. `067-COVERAGE.md` maps the `19` required groups
exactly once and records `067-20` plus `067-21` as explicit addendum overlays
on the current branch packet. The supporting `wiki -results.md`
note remains
non-canonical context only. Legacy non-canonical aggregator-consensus
references are tracked as stale drift only and must not become a second
authority path. Execution has now closed all `21/21` packets, including
`067-21-SUMMARY.md` and the post-addendum rerun closeout
`067-19-SUMMARY.md`: the
live subject path still carries exact `3f+1` committee sizing and `2f+1`
quorum proof through `BftCommittee` and `BftEngine`, the `SIM-7A7S` runtime
profile still records truthful local BFT-valid topology, `CelestiaLocalAdapter`
now stores and verifies namespace or raw blob bytes or commitment or theorem
or certificate binding plus inclusion-reference or retention-horizon or
degraded-state metadata on the local blob path, the validator still rejects
detached artifact drift on the resolved batch path, `scenario_11` now records
the executable Celestia-local artifact contract as `simulated-full` while
keeping real Celestia finality on the explicit `live-claim-removed` path, the
canonical checkpoint contract path remains
`crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml` with no
root-level parallel config lane, the intentional `crates/z00z_extensions/`
reorganization remains reflected in workspace membership and inventory
surfaces as a namespace directory rather than a root crate, the live
`z00z_rollup_node` binary plus canonical release-only manifest command
contract are now landed, durable consensus store plus restart recovery are now
landed, the deterministic replicated planner-authority path plus planner
claim-honesty registry are now landed, `crates/z00z_rollup_node/src/process_devnet.rs`
now owns the canonical local hold-mode process contract, `scripts/hjmt_local_devnet.sh`
and `docker/compose.hjmt-local.yaml` route to that same manifest-driven runtime
seam, release tests now prove five-process startup plus persisted restart or
stale-dir or missing-run-id rejection, `scenario_11` now records honest local
process-devnet claim boundaries, the live `InMemoryVoteTransport` seam now owns
the canonical deterministic delay or reorder or duplicate or replay or drop or
partition or heal or restart or reconnect scheduler plus digest-bound
`TransportFaultEvidence`, `scenario_11` now records honest duplicate or replay
or payload-withholding or restart-reconnect transport evidence rows, the
`hotstuff_local.rs` backend now owns the canonical local view or leader or
timeout or view-change or backend-QC state machine while preserving the live
commit-subject or replay or validator seams, the grouped crate-private wallet
`redb_store` debug-export re-export required by `test_production_hardening`
was restored before the final broad rerun, the structured evidence registry
now binds runtime records or fault-matrix rows or replay-vote evidence rows to
one digest-backed canonical path, the glossary claim registry and
`report_honesty.json` path now bind every governed term to one executable owner
or artifact or test or claim-level row, unsupported external-production wording
now fails through the live `scenario_11` report test path, and the current-cycle
`./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` rerun,
focused addendum release gates, `python3 scripts/audit/audit_067_claims.py`,
`cargo clippy --release --all-targets --all-features -- -D warnings`,
the captured broad workspace `cargo test --release` rerun,
`bash scripts/audit/audit_release_feature_guards.sh`, and
`git diff --check` are green on the current tree. The required
`/GSD-Review-Tasks-Execution` attempts were captured with consecutive clean
manual-review fallback after the final doc sync.
The packet now carries the dedicated
`old_primary_restart_after_takeover` fault row plus explicit runtime
anti-failback tests on one canonical proof path, the exact final rerun artifact
roots are recorded under `reports/phase-067/20260706T120602Z/` and
`reports/hjmt-local-devnet/20260706T120602Z/`, the final claim and conformance
packet is frozen in `067-CLAIM-AUDIT.md` and `067-FINAL-CONFORMANCE.md`, no
active `067-*` lane remains, and Phase 046 stays paused after `046-04`.
**Added**: 2026-07-03
**Directory**: `.planning/phases/000/067-Sharded-Concensus/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/000/067-Sharded-Concensus/067-TODO.md`
- `.planning/phases/000/067-Sharded-Concensus/067-CONTEXT.md`
- `.planning/phases/000/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/000/067-Sharded-Concensus/067-01-PLAN.md` through
  `.planning/phases/000/067-Sharded-Concensus/067-21-PLAN.md`

**Success Criteria**:

1. Phase 067 is discoverable in the active milestone without creating a
   second `067` directory.

2. The existing `.planning/phases/000/067-Sharded-Concensus/` folder remains the
   only canonical phase root, and Phase 067 registration does not create a
   duplicate folder or a parallel authority layer.

3. `067-TODO.md` plus `067-verdict.md` remain the canonical human-readable
   Phase 067 authority, `TASK-NNN` count remains truthfully zero, and
   `067-01-PLAN.md` through `067-21-PLAN.md` map `PHASE-0` through
   `PHASE-8` plus `VERDICT-LCS-01` through `VERDICT-LCS-10` plus
   `ADDENDUM-067-20` and `ADDENDUM-067-21` exactly once with no dropped or
   duplicated required group.

4. `067-CONTEXT.md` and `067-COVERAGE.md` preserve the single authority path,
   record legacy non-canonical aggregator-consensus references as stale drift
   only, and keep `wiki -results.md` supporting but non-canonical.

5. `/gsd-plan-phase 067` completes locally against the existing folder only,
   generating the full current-branch `067-01-PLAN.md` through
   `067-21-PLAN.md` packet with coverage, simulation, evidence, and
   anti-placeholder gates for every required Phase 067 unit plus the explicit
   late addendum overlays.

## Phase 068: Checkpoint Contract

**Goal:** Register the existing `.planning/phases/068-Checkpoint-Contract/`
folder as the canonical Phase 068 planning root for the storage-owned
recursive-ready checkpoint contract and replay substrate, keeping checkpoint
theorem or artifact or config-gate ownership in `z00z_storage::checkpoint`
without creating a duplicate phase directory or a parallel authority path.
**Requirements**: `068-TODO.md` remains the normative human-readable Phase 068
authority for registration, planning, and execution scoping. It defines the
storage-first recursive-ready checkpoint contract, keeps source code or tests
or local configuration as ground truth, treats future-only or target-design
wording in the Phase 068 corpus as live mandatory scope, freezes the required
config gate path to `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`,
keeps owner-crate authority in `z00z_storage`, and records literal `TASK-NNN`
count as zero because `068-TODO.md` contains no literal task identifiers.
**Depends on:** Phase 067 closeout; Phase 046 remains paused separately and is
not advanced by Phase 068 registration.
**Plans:** 16 canonical plan packets exist in the pre-existing phase folder.
Execution must proceed strictly in order from `068-01-PLAN`; later plans exist
as packetized execution intent only and MUST NOT be claimed complete out of
order. All discuss, context, coverage, plan, test, or execution artifacts for
Phase 068 MUST remain inside `.planning/phases/068-Checkpoint-Contract/` only.

**Status**: Added 2026-07-07 on the existing
`.planning/phases/068-Checkpoint-Contract/` directory only. No new phase folder
was created. Phase 068 planning packet generation is complete on that same
folder: `068-CONTEXT.md`, `068-COVERAGE.md`, `068-01-PLAN.md` through
`068-16-PLAN.md`, `068-TEST-SPEC.md`, and `068-TESTS-TASKS.md` are the live
phase-local execution packet under the still-normative `068-TODO.md`. Source
code, tests, and local configuration remain ground truth; the required config
gate path remains `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`;
owner-crate authority remains `z00z_storage`; `CheckpointContractConfigV1::load_repo_default()`
now resolves through `z00z_storage::checkpoint::repo_default_path()` so the
repo-owned YAML gate keeps one canonical path independent of caller `cwd`, the
remaining live storage tests now use the same helper instead of rebuilding the
repo path manually, and `CheckpointContractConfigV1::resolve_paths(...)` now
drives one canonical checkpoint and prep-snapshot namespace across storage and
live Scenario 1 consumers. The new release test file
`crates/z00z_storage/tests/test_checkpoint_contract_config.rs` now locks the
literal contract path strings plus the resolved path family. Every Phase 068
plan `<verify>` block now explicitly
names `/.github/prompts/gsd-review-tasks-execution.prompt.md`
(`/GSD-Review-Tasks-Execution`) as a canonical inline/local Codex review loop
with the at-least-three-runs and two-consecutive-clean rule; live Phase 067
references are normalized onto `.planning/phases/000/067-Sharded-Concensus/`;
the repo-default config path cleanup, canonical V1 statement framing,
replay-owned `tx_data_root` execution-input commitment path, the new
storage-owned `delta_root` or `witness_root` or `journal_digest` helper path,
and the provider-neutral `CheckpointDaReferenceV1` or
`CheckpointPublicationEvidenceV1` contract are landed. `CheckpointArchiveManifestV1`
is now rooted in `statement_core_digest`, and bare locator authority plus
provider-leakage or unknown-field drift reject on the canonical codec path.
Explicit predecessor-bound checkpoint links plus `CheckpointLifecycleV1` and
publication-readiness challenge gating are now landed. The checkpoint store and
prep snapshot surfaces now resolve through the same validated
`artifacts/checkpoints/*` family, the live Scenario 1 checkpoint consumers and
the scenario design packet are synced away from legacy checkpoint path
strings, the current-cycle `bootstrap_tests.sh` gate is green on the current
tree again, `bash scripts/audit/audit_068_source_truth.sh` is green,
`bash scripts/audit/audit_release_feature_guards.sh` is green, the broad
current-tree `cargo test --release` workspace gate is green, the targeted
`test_checkpoint_contract_config` and `068-14` rollup-node or validator or
watcher or simulator release reruns are green, and a later same-day continuity
rerun reconfirmed the broad `cargo test --release` workspace gate plus both
Phase 068 audit lanes green on that same tree. The canonical inline/local
workspace-first review loop is now recorded on `068-16-SUMMARY.md` with Pass 1
fixing the missing direct config-path coverage and Passes 2 and 3 clean,
`068-16` is summary-backed closed on `068-16-SUMMARY.md`, the phase
verification packet is landed on `068-VERIFICATION.md`, the repaired Phase 090
source now records current-code anchors plus Phase 068 versus Phase 069
authority boundaries, the real local DA adapter still proves publication
readiness only through the storage-owned publication-evidence boundary,
validator and watcher checkpoint consumers now share one storage-owned
publication-readiness bundle path while watcher evidence remains advisory, the
storage-owned recursive sidecar contract now binds statement digests,
prior-output roots, proof-byte digests, measurements, and 3-to-5-step chain
evidence on one non-authoritative facade path while canonical admission still
rejects the sidecar, storage now also owns one canonical typed PQ audit-anchor
path that binds statement or delta or witness or archive-manifest or Plonky3
epoch or Nova chain or PQ commitment digests through one cadence/helper/
validation lane while live cadence enforcement rejects missing PQ anchors once
the stage gate is active, storage now also owns one canonical
authority-promotion stage machine and typed verified-backend evidence surface
that keeps `CheckpointProofSystem::VERIFIED` reserved unless every required
gate matches one canonical config lane, the deterministic local E2E checkpoint
lane plus the fixture-backed source-truth or claim-guardrail lane are now
summary-backed closed, Phase 068 is verification-backed complete, and no
active `068-*` lane remains.
**Added**: 2026-07-07
**Directory**: `.planning/phases/068-Checkpoint-Contract/`
(pre-existing; do not create a duplicate phase folder)

**Canonical refs:**

- `.planning/phases/068-Checkpoint-Contract/068-TODO.md`
- `.planning/phases/068-Checkpoint-Contract/068-CONTEXT.md`
- `.planning/phases/068-Checkpoint-Contract/068-COVERAGE.md`
- `.planning/phases/068-Checkpoint-Contract/068-01-PLAN.md`
- `.planning/phases/068-Checkpoint-Contract/068-16-PLAN.md`
- `.planning/phases/068-Checkpoint-Contract/068-TEST-SPEC.md`
- `.planning/phases/068-Checkpoint-Contract/068-TESTS-TASKS.md`
- `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`

**Success Criteria**:

1. Phase 068 is discoverable in the active milestone without creating a
   second `068` directory.

2. The existing `.planning/phases/068-Checkpoint-Contract/` folder remains the
   only canonical phase root, and Phase 068 registration does not create a
   duplicate folder or a parallel authority layer.

3. `068-TODO.md` remains the canonical human-readable Phase 068 authority,
   source code or tests or local configuration remain ground truth, literal
   `TASK-NNN` count remains truthfully zero, and future-only or target-design
   wording in the Phase 068 corpus stays live mandatory scope.

4. The canonical config gate path remains
   `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`, and
   checkpoint statement or artifact or config-gate ownership stays in
   `z00z_storage::checkpoint` with no root-level parallel authority path.

5. `/gsd-discuss-phase 068`, `/gsd-plan-phase 068`, `/gsd-add-tests 068`, and
   `/gsd-execute-phase 068` must operate locally against the existing folder
   only and must not create any duplicate phase root, alias authority, or
   parallel plan tree outside that directory.

6. Release-mode simulator verification for Phase 068 must use the canonical
   `scenario_1` test target path and must not rely on a standalone
   `test-params-fast` lane that the live `z00z_simulator` crate rejects in
   release-capable builds.

## Phase 069: Recursive Proof

**Goal:** Deliver a non-authoritative hybrid recursive checkpoint proof lane using
Nova IVC for checkpoint steps and Plonky3 recursive STARK evidence for bounded
epochs over the existing Phase 068 checkpoint contract.

**Requirements:** RCP-069
**Depends on:** Phase 068 completion and its storage-owned checkpoint,
recursive-sidecar, PQ-anchor, and authority-promotion contracts.
**Plans:** 5/14 plans executed

- [x] 069-01-PLAN.md
- [x] 069-02-PLAN.md
- [x] 069-03-PLAN.md
- [x] 069-04-PLAN.md
- [x] 069-05-PLAN.md
- [ ] 069-051-PLAN.md
- [ ] 069-06-PLAN.md
- [ ] 069-07-PLAN.md
- [ ] 069-08-PLAN.md
- [ ] 069-09-PLAN.md
- [ ] 069-10-PLAN.md
- [ ] 069-11-PLAN.md
- [ ] 069-12-PLAN.md
- [ ] 069-13-PLAN.md

**T1 acceptance (2026-07-14):** T1 is complete. The repository-local cutover validates every manifest/root binding, commits its CAS at immediate durability, reloads it, and rejects a second install. The evaluator consumes strict bounded opcode/JMT bytes, independently recomputes raw-SHA JMT transitions, and requires exact terminal→bucket→serial→definition child-root consumption. The authority-defined empty/no-op transition is explicit: config, authority digest, durable manifest, canonical transition, and independent evaluator bind the same execution-input version; only the sentinel handoff and a typed zero-update envelope are accepted. Five YOLO execution reviews ran (the final two clean) and two `doublecheck` passes completed. Bootstrap, targeted release suites, the full 187-unit/package `z00z_storage` release suite, `cargo build --release`, full workspace `cargo test --release`, and the release feature guard completed. T2 is now active; T3–T4 and Plan 06 remain locked on T2 acceptance.

**Verification update (2026-07-14):** The current mandatory bootstrap completed. Release target suites pass: `test_recursive_v2_trace` 6/6 and `test_recursive_v2_cutover` 4/4; config-drift and JMT-envelope tamper regressions pass. The scoped recursive V1 elimination is complete; no executable recursive V1 selector or fallback remains. T1 is accepted and execution advances to T2. T2 dependency preflight pins and release-checks `nova-snark = 0.73.0` with its audited `io` feature only; its circuit/bundle/runner work remains active.

**T2 execution update (2026-07-14):** The mandatory bootstrap reran successfully after the exact Nova dependency pin. T2 implementation is constrained to the sole private path `z00z_storage::checkpoint::recursive_v2::nova`; no public Nova type, V1 compatibility item, runtime SHA-width selector, duplicate JMT evaluator, or second circuit owner is permitted. T3–T4 and Plan 06 remain locked until the real `ShapeCS → PublicParams → RecursiveSNARK → CompressedSNARK::prove/verify` acceptance and review convergence evidence are retained.

**T2 evidence update (2026-07-14):** One fixed 780-cell control relation, capped private authority-bundle framing, and constrained typed source-event phase/opcode/ordinal/payload-commitment limbs are implemented. Release tests pass for a real Nova control proof and one-bit source-event commitment tampering through the actual verifier. The expensive PP/VK bundle round-trip was stopped without a captured exit and is inconclusive. Full replay/SHA/uniqueness/JMT/hierarchy/final-input constraints, continuous runner, and complete adversarial mutation ledger remain open; T2 is not complete and T3–T4 remain locked.

**T2 progression update (2026-07-15):** The private fixed-shape circuit now carries a 16-limb accumulator over canonical T1 source-event bindings and constrains exact ordinal succession. A real two-step compressed Nova proof passed in release mode; verifier-negative cases reject tampered event digests, skipped or reordered ordinals, and wrong initial accumulator limbs. This is not the authority-final trace root or full replay relation. T2 remains active and T3–T4 remain locked.

**T2 terminal-binding update (2026-07-15):** `FINALIZE_BLOCK` now constrains each post-step accumulator limb to a persistent, range-constrained authority-supplied expected-root limb. The real release `real_nova_finalization_binds_authority_trace_root` compressed-proof test passed in 195.40 seconds and rejects early finalization, mismatched expected roots, and tampered expected-root values at verification. The carried accumulator is still not the canonical SHA `RecursiveTracePrecommitV2::trace_digest`, and its expected root is not bound to the canonical authority snapshot. T2 remains active; T3–T4 and Plan 06 remain locked.

**T2 control-machine update (2026-07-15):** The private Nova witness and R1CS legal-edge/next-state relation now consume one explicit 35-edge production control table. A separate test-only 16×13 expected-edge matrix exhaustively checked all 208 phase/done/opcode tuples in release mode: exactly one successor or typed rejection, `FINALIZE_BLOCK` as the sole route to `done=1`, and early, double, generic-no-op, and post-final events rejected. The affected real compressed finalization regression passed in 193.83 seconds. This table is not yet shared with the independent trace validator/evaluator, so T2 remains active; T3–T4 and Plan 06 remain locked.

**T2 verifier-bundle update (2026-07-15):** The private `CheckpointVerifierBundleV2` completed a real release `ShapeCS → PublicParams → recursive/compressed proof → encode → strict canonical load → loaded-key decode/verify` flow in 453.38 seconds. The measured PP/VK/proof/header/bundle sizes are 451,157,344/273,174,184/37,808/490/724,332,018 bytes, below the unchanged 939,525,120-byte bundle cap. Authority/profile/spec/source/lock header mutations (with recomputed project digest) and PP payload mutation reject before proof decoding. This is bundle evidence only: no runner/receipt integration or T2 acceptance claim follows; T3–T4 and Plan 06 remain locked.

**T2 review-pass-2 update (2026-07-15):** Inline `/GSD-Review-Tasks-Execution` pass 2 found and fixed the under-constrained boolean `done`: the allocated bit now has an R1CS equality to `z[DONE_CELL]`, and the exact release `DONE_CELL=2` regression is unsatisfied. The active-shape finalization rerun passed in 197.71 seconds, and the active-shape full PP/VK bundle rerun passed in 461.15 seconds; the pre-fix values remain historical diagnostics only. Pass 2 is not clean: the circuit still lacks the canonical SHA trace/replay/uniqueness/net/JMT/hierarchy/statement relations and source-expander EOF binding; the accumulator is not `RecursiveTracePrecommitV2::trace_digest`; table sharing, selected measured SHA width, complete parser/mutation/Model B/C corpus, and runner/receipt are absent. T2 remains active; T3–T4 and Plan 06 remain locked.

**T2 SHA/preflight update (2026-07-15):** The one private Nova owner now allocates a fixed 64-byte FIPS SHA-256 compression gadget on every step. `SHA_BLOCK` R1CS-binds block bytes, eight u32 chaining words, ordinal, and output; an inactive lane is uniquely zero and preserves SHA state. It reuses `CheckpointSha256BlockV2` only as the native witness/differential source, with no native-validity acceptance path or runtime width selection. Exact sparse shape/preflight values are C=46,719, V=45,645, NZ=223,454, N=223,454, G=262,145; cap+1/overflow paths return typed `Resource` before setup. One real release gated setup→IVC→compression→bundle/load/loaded-key verification passed: PP/VK/proof/header/bundle are 457,647,656/273,174,184/37,808/490/730,822,330 bytes, wall 7:32.03, peak RSS 15.30 GiB, no swap. This is one-compression-lane evidence only, not full framed SHA/trace digest or a selected measured SHA width; T2 remains active and T3–T4/Plan 06 remain locked.

**T2 canonical-hash-control update (2026-07-15):** The private Nova owner now consumes the sole `recursive_trace` expander's source→Begin→Block*→End control sequence. One decoded `HashControlBindingV2` feeds the SHA witness; R1CS binds source hash, role, framed message length, block count/index/offset, final flag, chaining, and digest, while separate state counters constrain global schedule, source-record, and encoded high-bit control ordinals. Canonical sequence and independent metadata/order/block/chaining mutations pass focused release tests; fixed shape is C=47,210, V=45,594, NZ=224,501, with cap+1/overflow preflight rejection before setup. The captured current-shape real setup→IVC→compression→strict bundle/load/loaded-key verify passes in 454.95 seconds; PP/VK/proof/header/bundle are 457,701,320/273,174,184/37,808/490/730,875,994 bytes. This verifies per-source `hash_binding` controls only, not global `RecursiveTracePrecommitV2::trace_digest` or full T2 SHA/replay; T2 remains active and T3–T4/Plan 06 remain locked.

**T2 global-trace update (2026-07-15):** The one spool now emits a grammar-tagged TracePrecommit BEGIN/BLOCK*/END sequence after per-source controls, using the same decoder and one Nova SHA lane. During the existing replay, canonical source records are absorbed exactly once; the global controls bind source count/bytes, schema/role, EOF, exact framed byte length, padding-zero count, bit length, block count/index/offset/final marker, chaining, and the expected `RecursiveTracePrecommitV2::trace_digest`. Two-source canonical R1CS and source/order/length/role/schema/framing/padding/bit-length/count/EOF/digest mutation tests pass. Current shape is C=47,645, V=45,937, NZ=226,133; cap+1/overflow preflight rejects before setup. Captured `real_nova_global_trace_bundle_loads_and_verifies_compressed_proof` passes in 449.56 seconds with PP/VK/proof/header/bundle 457,777,040/273,174,184/37,808/490/730,951,714 bytes. **Review pass 3 correctly rejects this as full source→trace evidence:** the current SHA lane constrains a witness-supplied block stream and its geometry/chaining, but not byte equality to the canonical source records/framing. T2 is therefore redesigning the same one-spool, one-64-byte-lane schedule to stream R1CS-bound canonical record-byte chunks through concurrent O(1) source/global SHA contexts; it may not use a digest/preimage, same-length, raw-64-MiB-state, second-spool, or per-source-only fallback. This does not establish exact trace bytes, replay, uniqueness, JMT, hierarchy, statement, runner, or T2 acceptance; T3–T4 and Plan 06 remain locked.

**T2 canonical-chunk foundation update (2026-07-15):** `recursive_trace` now owns one reusable canonical source-record encoder/parser and strict 64-byte zero-padded `TraceChunk` control grammar (`version|source ordinal LE|chunk ordinal LE|chunk count LE|byte count|bytes[64]`) in a disjoint bit-62 ordinal space. Header/payload cut-point tests exercise reconstruction from that sole byte view; the old Nova fixed-shape fixture explicitly rejects `TraceChunk` rather than manufacturing a no-op edge. This is intentionally not an acceptance claim: emission waits for the single-gadget concurrent source/global context transition, and four release dead-code warnings remain until those production paths consume the grammar. T2 remains active; T3–T4 and Plan 06 remain locked.

**T2 canonical-chunk schedule update (2026-07-15):** The sole spool emits every strict encoder-derived chunk only after its corresponding source record and after that record's `BEGIN_HASH`, immediately before the source block it makes available. The backend-neutral evaluator rejects an over-profile `chunk_count` before allocation, retains at most one bounded source record's chunk sequence, and compares every ordinal, count, meaningful byte, and zero tail against that same canonical encoder before semantic processing or the derived source hash schedule. The integration trace suite and focused release source/global-control fixtures pass; this removes the former production dead-code warnings without suppression. The private Nova table still rejects `TraceChunk`, because a generic self-loop, a digest-only accumulator, or a host-side byte assertion would leave review pass 3's byte-to-SHA gap intact. The next active T2 slice is the single fixed-width R1CS byte-context and exact chunk-to-block equality; T2 is not accepted, and T3–T4/Plan 06 remain locked.

**T2 verifier-material/RSS correction update (2026-07-15):** `CheckpointProverMaterialV2` now keeps `PP + PK` private, while `CheckpointVerifierBundleV2` serializes and decodes only an immutable authority-pinned header (`pp_digest`, `vk_digest`, source/lock/profile/spec/shape bindings) plus VK. The release `real_nova_global_trace_bundle_loads_and_verifies_compressed_proof` regression passed in 327.90 seconds with `VK=273,174,184`, `header=454`, `bundle=273,174,638`, and no PP decoder on the verifier path; header and VK-payload mutations still reject before proof decode. This resolves the previous role-mixed PP+VK verifier artifact, not the heavy VK distribution problem. The current base-shape preflight remains a lower-bound diagnostic only: it must not authorize setup until an exact augmented-primary/secondary-shape synthesis and a worker-enforced measured RSS/time budget exist. The byte-to-SHA R1CS relation, full semantics, runner, mutation matrix, and review convergence also remain open. T2 is active; T3–T4 and Plan 06 remain locked.

**T2 verifier binding/cap and verdict-plan adaptation update (2026-07-15):** Bundle generation now rejects a PP digest that differs from the supplied private prover PP, closing the mixed-generation construction gap before framing. The compressed Nova proof cap is the required 128 KiB (not 16 MiB), with an explicit 128 KiB + 1 rejection before bincode decode; the measured 37,808-byte proof remains below it. The mandatory bootstrap rerun, `cargo build --release`, and full `cargo test --release` pass on this source. `069-051-PLAN.md` now makes the verified lifecycle explicit: the recursive lane is asynchronous and cannot affect canonical finality; Nova fold, compression, persistence, and network publication have separate cadence; a possible 1000-block Plonky3 epoch is not a Nova default; full PP/PK/VK/IVC/recovery and per-peer/fanout measurements are required; and Plan 10 owns private PP/PK recovery while Plan 09 owns verifier/archive retention. These are live authority requirements, not a T2 completion claim. Exact source-byte→SHA R1CS, full semantics, augmented-shape bounded worker, selected width, and review convergence remain blocking S1 work; T3–T4 and Plan 06 stay locked.

**T2 source-selector hardening update (2026-07-15):** `CheckpointNovaCircuitV2` now algebraically equates the `SourceRecord` hash-control stage with the one-hot source-opcode set. A retained release mutation relabels a valid source witness as the otherwise legal idle `BEGIN_HASH` self-loop and reaches the R1CS failure `source_stage_matches_source_opcode`; this closes opcode-stage confusion before `TraceChunk` is made a legal edge. The verifier header's source identity also now commits to both `nova.rs` and the sole canonical trace/feeder owner `recursive_trace.rs`, with a direct regression. The fixed base ShapeCS regression is C=47,648, V=45,938, NZ=226,151 (the matching lower-bound preflight is updated but remains diagnostic only). The three PP/PK-heavy real-Nova tests now share one test-only mutex, so the mandatory bootstrap retains normal parallelism elsewhere but cannot cause a multi-setup RAM race; it completed at `2026-07-15T13:32:40+03:00` with exit `0`. The live T2 PLAN now fixes the only valid cutover as source → BEGIN_HASH → ordered canonical chunks/interleaved derived blocks → exact FIPS padding → END_HASH, with O(1) source/global contexts and per-block R1CS equality; it explicitly forbids a second feeder/spool, digest/same-length stand-in, or control-payload assertion. A fresh same-process real-proof invocation reached PP/VK/proof generation (`VK=273174184`, proof=37808) but did not retain a terminal exit record, so it is explicitly not new verifier or bounded-worker evidence. Review pass 4 is not globally clean: `TraceChunk` still has no Nova edge, so no generic self-loop, host-only assertion, or digest/same-length substitute can falsely represent exact canonical bytes. The O(1) source/global byte-context to SHA-block/final-digest relation, augmented worker measurement, complete semantics, and review convergence remain required; T2 is active and T3–T4/Plan 06 stay locked.

**S1 streaming-order update (2026-07-15):** The only production source expander now uses the planned O(1)-compatible order: source → `BEGIN_HASH` → all prefix blocks immediately available from the canonical role/label framing → each zero-tail-checked canonical `TraceChunk` followed by every block it makes available → exact FIPS final block(s) → `END_HASH`. `CheckpointSha256BlockStreamV2` gained the sole declared-length chunk-feed API, so storage neither duplicates the part-prefix grammar nor accumulates a block tape. The independent evaluator now retains a bounded native canonical reference only to reject reordered, missing, duplicated, relabelled, or changed chunks after their source record. This is prerequisite wiring, not T2 acceptance: Nova still rejects `TraceChunk`, no SHA byte is yet R1CS-derived, the global context is not yet proved, and the post-change bootstrap/release cycle remains open. No V1, second source/trace/SHA owner, generic self-loop, digest/same-length substitute, `VERIFIED` promotion, or Plan 06 advancement is authorized.

**S1 prerequisite verification update (2026-07-15):** The post-change bootstrap completed through its foundational, storage real-Nova, wallet, and compile-check stages. Targeted release tests are green: `test_recursive_v2_trace` 6/6, `test_recursive_v2_cutover` 4/4, and the seven `recursive_trace` units. The narrow two-test Nova skeleton packet is also green and retains the negative invariant that `TraceChunk` is rejected before its complete constrained relation lands. These are only regressions for the one expander/evaluator path; they neither prove the source/global R1CS byte relation nor unlock T3–T4/Plan 06.

**S1 framing-owner prerequisite (2026-07-15):** `CheckpointSha256BlockStreamV2` now exposes the canonical role/DST length prefix used by its own streaming implementation. This prevents Nova from recreating the role-domain grammar when the constrained byte context is wired. The subsequent mandatory bootstrap completed, including its storage real-Nova, wallet, and compile-check stages; its crypto packet passed 257/257. The addition still does not make `TraceChunk` a Nova edge or weaken any T2 lock.

**S1 source/global relation status (2026-07-15):** S1 remains one atomic relation, not two permissive feeders. Direct source review confirms that the current global `TracePrecommit` controls are emitted only after all source schedules. They therefore cannot be declared O(1)-bound by retaining or refeeding their SHA blocks: the sole trace-expander/circuit path must reconcile ordering with a second selector-gated global byte context that consumes the same live canonical chunks. Until then Nova must continue to reject `TraceChunk`; no digest, same-length, native assertion, generic self-loop, second spool/encoder, V1 path, width selector, `VERIFIED` enablement, T3–T4, or Plan 06 is authorized.

**S1 byte-context implementation status (2026-07-15):** The active `nova.rs` work now has an explicit internal `inactive/source/begin/block/end` selector split so ordinary non-hash grammar rows cannot alias a source record, plus a fixed-width `TraceChunk` witness allocation that range-constrains ordinals/counts, derives a one-hot 1..64 byte count, and R1CS-forces every zero tail byte. The byte contexts now keep canonical-byte length separate from role-framed SHA message length. Their fixed queue is 192 bytes, not 128: the authoritative Trace role prefix and source label/part framing occupy 131 bytes before the first canonical chunk, so it must hold two pending compression inputs plus a bounded remainder; the running-state cell is R1CS-bounded to `0..=192`, not merely eight bits. The fresh mandatory bootstrap completed with exit 0 after the layout correction: crypto 257/257, core 273/273, storage 222/222 including all real-Nova regressions (719.54s), utils 167/167, wallet integration, and release bench/example checks. ShapeCS was correspondingly remeasured to C=56,650, V=51,785, NZ=255,918, PP lower bound 19,985,104. This is live S1 implementation work, not a staged acceptance: `TraceChunk` has no legal table edge until the same atomic relation binds source/global queues, every selected SHA input, FIPS padding, and endpoint digest; no broader T2/review closure is claimed.

**S1 live-global schedule update (2026-07-16):** `event_pass` now emits TracePrecommit `BEGIN_HASH` before replaying source records and keeps one global `CheckpointSha256BlockStreamV2` live across that replay. For every canonical source record it absorbs the record bytes, then absorbs exactly each canonical `TraceChunk` at its source-expander callback, before finalizing the global stream with its own exact FIPS blocks and `END_HASH`. This removes the former terminal spool rewind/re-read and does not introduce a second encoder, block tape, or native digest assertion. A clean mandatory bootstrap passed (crypto 257/257, core 273/273, storage 221/221 including the three real-Nova regressions in 725.09s, utils 167/167, wallet integration, release bench/example checks); targeted release replay, global-live-chunk-order, and verifier-bundle tests are each 1/1 green. The global order test is deliberately native-only. `TraceChunk` remains rejected by Nova until one fixed-width R1CS byte-context binds canonical source bytes → chunk bytes → each selected source/global SHA block → exact FIPS padding → final endpoint digest atomically. Thus S1, T2, review convergence, worker evidence, T3–T4, and Plan 06 remain open/locked.

**S1 source-header constraint update (2026-07-16):** Before making a `TraceChunk` edge legal, Nova now allocates on every fixed-shape step a range-constrained source-record identity witness: opcode, ordinal, 32-byte object ID, and u32 payload length. On source steps its opcode and ordinal equal the actual `SourceRecord` event and `payload_len + 45` equals the sole canonical record length in the decoded source control; on every other step the entire object is algebraically zero. It retains no payload bytes and creates no encoder, digest substitute, context transition, or generic chunk self-loop. This establishes the exact fixed header from which first-chunk canonical bytes must next be derived. The mandatory bootstrap completed with exit 0: crypto 257/257, core 273/273, storage 221/221 including all real-Nova tests in 719.96s, utils 167/167, wallet integration, and release bench/example checks. Re-pinned diagnostic base shape is C=57,083, V=52,180, NZ=257,543, PP lower bound=20,060,496. It is not augmented-shape/RSS/worker evidence and does not accept `TraceChunk`, close S1/T2, start reviews, or unlock T3–T4/Plan 06.

**S1 canonical-header derivation update (2026-07-16):** The source identity now has one R1CS-derived canonical header, not parallel byte witnesses: `opcode | ordinal_le[8] | object_id[32] | payload_len_le[4]`. The new fixed-width decomposition range-checks and reconstructs each little-endian integer, then retains no payload byte or second source serialization. It is intentionally still insufficient for S1: no `TraceChunk` row is legal, and no block/padding/digest is accepted from this header until concurrent source/global byte contexts consume the same chunk bytes and bind every selected FIPS input. The mandatory bootstrap rerun completed with exit 0 (storage 221/221 including the real-Nova paths in 723.00s), and the targeted release preflight measured C=57,336, V=52,384, NZ=258,490, G=262,145, PP lower bound=20,104,448, VK lower bound=8,388,640, verifier-bundle lower bound=8,389,094, and Pedersen RSS lower bound=50,331,840. These are base-shape diagnostics only; they do not authorize setup/RSS, accept `TraceChunk`, close S1/T2, start review convergence, or unlock T3–T4/Plan 06.

**S1 source-context lifecycle update (2026-07-16):** The exact R1CS-derived 45-byte source header now lives in the fixed source byte context: source row creates it, source `BEGIN_HASH` marks that context started, every source `SHA_BLOCK` preserves it, and source `END_HASH` clears it. Selector-gated range and transition constraints keep all non-source contexts zero and do not make legacy SHA witness bytes trusted. This is an intentionally fail-closed prerequisite for the atomic cutover: `TraceChunk` still has no Nova edge, and the source/global queues, chunk bytes, selected block bytes, FIPS padding, chaining, and final endpoints remain unbound. The mandatory bootstrap rerun completed with exit 0 (crypto 257/257, core 273/273, storage 221/221 including all real-Nova regressions in 721.40s, utils 167/167, wallet integration, release bench/example checks). The targeted release preflight remeasured base ShapeCS C=60,646, V=53,965, NZ=268,958, G=524,289, PP lower bound=28,991,216, VK lower bound=16,777,248, verifier-bundle lower bound=16,777,702, Pedersen RSS lower bound=100,663,488. These diagnostics do not authorize setup/RSS, accept a chunk, close S1/T2, start review convergence, or unlock T3–T4/Plan 06.

**S1 source-static-frame update (2026-07-16):** On a source `BEGIN_HASH`, Nova now materializes into that same fixed context the exact static pre-chunk bytes returned by the sole `CheckpointSha256BlockStreamV2` framing owner: role/DST prefix, source-label length, source label, and the R1CS-decomposed declared canonical-record length. It constrains `message_bytes = static_frame_bytes + canonical_record_bytes`; no local SHA grammar, payload arena, or host byte assertion was added. This is deliberately still pre-cutover: no selected compression input reads the queue yet, `TraceChunk` remains rejected, and source/global queue transitions, exact FIPS padding, chaining, and endpoint equality remain one required atomic relation. The required bootstrap rerun passed with storage 221/221 including real-Nova regressions in 725.40s; targeted release preflight is C=60,785, V=54,225, NZ=269,502, G=524,289, PP lower bound=29,016,312, VK lower bound=16,777,248, verifier-bundle lower bound=16,777,702, and Pedersen RSS lower bound=100,663,488. These diagnostics do not authorize setup/RSS, accept a chunk, close S1/T2, begin review convergence, or unlock T3–T4/Plan 06.

**S1 global-context-prefix update (2026-07-16):** TracePrecommit `BEGIN_HASH` now initializes the second fixed O(1) byte context with exactly the `CheckpointSha256BlockStreamV2` role/DST prefix, precommitted byte/event counters, framed message length, and FIPS IV; its matching TracePrecommit END clears that context. This uses no terminal rewind, global byte encoder, or host validity assertion. It is deliberately not a legal `TraceChunk` edge: source record-length insertion, shared chunk append/order, selected SHA block bytes, exact FIPS padding, chaining, and final endpoint equality still must land as the one atomic S1 relation. The required bootstrap rerun passed with storage 221/221 including the three real-Nova regressions in 719.89s; targeted release preflight is C=63,099, V=55,076, NZ=276,443, G=524,289, PP lower bound=29,349,488, VK lower bound=16,777,248, verifier-bundle lower bound=16,777,702, and Pedersen RSS lower bound=100,663,488. These diagnostics do not authorize setup/RSS, accept a chunk, close S1/T2, begin review convergence, or unlock T3–T4/Plan 06.

**Status:** `069-051-T0` completed on 2026-07-13 and `069-051-T1` completed on 2026-07-14. `RepositoryLocalNoLiveV1` has a two-process opaque fixture capture; scoped recursive V1 source/package/tests, codec/store/config ingress, stale mirrors, and named release-cache artifacts are removed. V2-only normative authority names one path, `z00z_storage::checkpoint::recursive_v2`; coverage is 1084/1084 without drift; external release consumers prove deleted types/codecs/features fail. T1 provides one resolver-derived repository-local capability and snapshot binding, private bounded source spool, actual pinned-HJMT traces, immediate-durability redb CAS cutover/reload, typed V2 statement, strict independently evaluated opcode/JMT/hierarchy relation, and exactly one authority-defined typed empty/no-op transition. The no-op contract is digest- and manifest-bound and rejects every generic empty handoff or envelope substitution. Bootstrap, full storage/workspace release build/tests, release feature guard, five YOLO reviews (the last two clean), and two doublechecks passed. `069-051-T2` is active; it alone owns the uniform Nova micro-step and immutable verifier bundle. T3–T4 and Plan 06 may not start until T2 accepts. The version-managed release commit awaits explicit authority because the shared worktree has unrelated user changes. `069-02` resolved its documented successor decision:
registry `p3-recursion 0.1.0` remains rejected as a placeholder, while exact
upstream commit `b36339709a7a67ee9760fb578b3d4339fd983709` resolves the P3
`0.6.1` recursion family and executes a real KoalaBear/Poseidon2 recursive
proof. The workspace P3 `0.4.3` hash policy stays private to `z00z_crypto`; the
live recursion adapter may receive only project-owned canonical bytes. The
locked real Nova probe co-resolves that boundary but cannot substitute for P3
evidence or authority. `069-03` froze the one storage-owned bounded predicate,
context/witness/parameter encoding, V2 non-authenticating epoch-evidence
commitment, and stable reject taxonomy; its final isolated root release gate
passed. `069-04` added the non-authoritative isolated Nova/P3 receipt boundary with real
library-smoke verification and no public backend types. `069-05` owns private
exact Goldilocks Poseidon2 and finite-input `hash_zk` R1CS components with
native differential/mutation evidence. The original `069-051-T1` padded
64 MiB arena premise is no longer accepted as a completed contract.
`069-051-T2-CRYPTO-AUDIT-2.md` establishes that the 67,108,864-byte value is a
total storage/witness cap, not one Nova-step shape. The existing pinned-Nova
`ShapeCS` abort—603,979,776 minimum auxiliary variables/constraints before the
semantic relation—remains valid falsification of that representation only.
It is not a theorem-level resource lower bound.

Audit 2 additionally records an executable V1 byte-field alias, insufficient
rate-seven sponge capacity, incomplete native replay/HJMT update semantics,
false-positive fixtures, conflated storage/recursive roots, a test-only
`TrivialCircuit` Nova proof, underbound smoke-only receipt, and invalid
independent per-block IVC restart assumptions. A release Pallas/Vesta
feasibility experiment completed a SHA-bound 64-byte step, real setup, IVC,
Spartan compression, and compressed verification with 55,205 primary
constraints and 261,984 KiB runtime peak RSS. Its temporary source/executable
were removed.

Further direct root-source review found that V1 `SettlementModel::root` is a
separate weak Poseidon2 computation, HJMT paths reconstruct the SHA definition
backend root, V1 batch headers hard-code generation/binds V1, and a root pair in
Nova `z0` would not prove state equivalence. The sole permitted recovery is
therefore one authority-pinned deterministic storage migration in the existing
owners to `RootGeneration::SettlementV2`, whose digest is derived with the
canonical SHA-256 helper from HJMT layout/policy/definition root; one
settlement-owned V2 HJMT envelope and actual-length typed event trace; streamed
field-bound SHA/JMT work; two-pair sorted/permutation uniqueness; one directly
compiled algebraically gated fixed-shape private Nova checkpoint circuit; and one
continuous IVC across blocks with compression only at finalized boundaries.
After T0 capture, V1 must be physically eradicated; only its opaque migration record and inert negative test bytes may remain. There may be no live V1/V2 selector,
recursive projection root, asserted cutover pair, V1 Poseidon settlement lane,
native-validity/sort shim, reduced theorem, alias, duplicate JMT walker, or
parallel circuit/state/root owner. `069-051-PLAN.md` has been rewritten and
reviewed in place from T1 through T4. Its source-bound coverage closes 55/55
Audit-2 E-IDs, 347/347 non-fenced audit list items, A-01..A-17,
DC2-F01..DC2-F24, every named lemma/attack/
vendor row through the mandatory theorem matrix, and all 31 Plan 05 TODO atoms.
The existing canonical coverage audit now checks the active `069-051` overlay,
all four continuation task schemas/verification gates, and those 31 inherited
atoms directly; it no longer relies only on the superseded Plan-05 task text.
The reviewed T2 sequence also measures a finite compile-time SHA batch-width
set and freezes exactly one width before authoritative PP/VK generation; no
runtime-selectable or parallel circuit path is permitted.
Corrected migration/code/tests, production receipt, and end-to-end verifier
evidence remain incomplete; T0 is complete and execution is at partial T1,
while T2–T4 and Plan 06 are locked.
Two independent planning doublechecks pass: the first verifies AUDIT-2 claims,
owners, constraints, tests, and Models A/B/C; the second verifies `069-TODO.md`
against the 1084-atom ledger and the exact 31-atom Plan 05 disposition. The
current mandatory release bootstrap, full root
`cargo test --release` through final doc-tests, full `cargo build --release`,
and release-feature guard all exit `0`; they establish a regression-green
baseline but cannot certify unimplemented T1-T4 behavior. Scoped cargo-deny
source/license/advisory checks pass; its nested path/workspace `bans` diagnostics
remain a production/promotion blocker, not a waiver. The
deterministic audit covers 1084/1084 TODO atoms, 66/66 scoped semantic owners,
and 149/149 test-section owners. These are planning/baseline facts only and do
not certify the unimplemented T1–T4 path.
The 2026-07-14 final source-bound doublecheck binds the 4,325-line AUDIT-2
snapshot `7638cbf46e7410b8627e9d734682a11fd185ddb4bb68f9d8b225f38cfc18751f`,
recomputes the exact Nova fold and two-pair uniqueness bounds, and records five
review passes with the last two clean for audit/plan coverage. It also records
the current blockers DC2-F11–F24 and the absence of a live Nova dependency,
uniform circuit, authority-pinned verifier bundle, continuous runner, and real
Models A/B/C verifier targets. Bootstrap, full workspace release tests/build,
targeted V2 tests, and security guards pass; they do not unlock Plan 06.
Plan 069-01 froze the Phase 068 owner ledger, made target/future design wording
explicit live scope, canonicalized the live-boundary whitepaper source path,
passed the required bootstrap, release-feature, and full release test gates,
and recorded six YOLO reviews with two consecutive clean final passes.
**Directory:** `.planning/phases/069-Recursive-Proof/` (pre-existing; do not
create or duplicate).
**Authority:** `069-TODO.md` and its referenced design/whitepaper corpus are
phase authority and mandatory live implementation scope; current code, tests,
and repository configuration remain implementation ground truth. Target/future
design statements are live scope, not deferred status.
**Execution gate:** `069-03` permits only the frozen byte-only storage predicate
boundary. Plan 04 must consume it without duplicating schema, theorem, reject,
or config ownership. No P3 type/value conversion may cross into storage or
rollup public APIs. Planning and the isolated probe must not enable
`CheckpointProofSystem::VERIFIED` or make recursive evidence authoritative.

**069-051 T2/S1 streaming correction (2026-07-16):** The rejected two-192-byte
quadratic-queue candidate reached arity 1,298/C=199,653. The retained compact
single-path relation derives source static framing blocks and the global role/DST
block by fixed cursor, retains only two O(1) contexts, and has arity 1,170 with
C=108,685/V=66,636/NZ=415,838. `TraceChunk` is no longer rejected or a
self-loop: its zero-tail-checked canonical feeder bytes append to both contexts,
then each selected static or queued SHA block is R1CS-equal to the matching
context before FIPS compression. Padding, big-endian bit length, chaining,
block cursor, source/global EOF, and endpoint digest equality remain in that
one relation; no second spool/encoder, native assertion, raw block witness, or
digest/same-length substitute is introduced. A retained release mutation changes
a canonical chunk payload byte and fails at the selected source/global
compression relation; short-final count, zero-tail, and source-ordinal mutations
fail at their respective R1CS gates. The mandatory bootstrap passes on this
source (storage 222/222 including real proof and verifier-bundle verification in
607.83s, plus crypto/core/utils/wallet and release bench/example checks). This
does not normalize the metric or memory cap, authorize base-shape setup/RSS,
select a SHA width, complete the remaining mutation ledger or replay/uniqueness/
net/JMT/hierarchy/statement relations, or begin review convergence. T2 remains
active; T3, T4, and Plan 06 remain locked.

**069-051 T2 bounded-worker measurement (2026-07-16):** The mandatory
release bootstrap passed after the worker gate was added: storage 221/221,
including both real-Nova paths, in 698.99s; crypto/core/utils, wallet
integration, and release bench/example checks also passed. A separately
reproducible release verifier run under `timeout=900s` and `RLIMIT_AS=24 GiB`
measured the actual augmented Nova shapes as primary C=271,964/V=231,075 and
secondary C=10,349/V=10,331, versus the base C=108,685/V=66,636. It produced
VK=273,174,184 bytes, proof=50,392 bytes, and a header+VK verifier bundle of
273,174,638 bytes; PP/PK are absent from that bundle. The process completed
without swap in 7:23.29 with a 17,264,693,248-byte (16.08 GiB) peak RSS. This
is containment evidence only: it disproves any claim that the base-shape
preflight authorizes safe setup and leaves the 24 GiB ceiling as an emergency
worker limit, not an acceptable prover or watcher operating budget. T2 remains
active pending resource acceptance, remaining semantic/mutation obligations,
and review convergence; T3, T4, and Plan 06 remain locked.
No numerical authority-pinned resource-budget tuple presently exists; therefore
the 24 GiB ceiling, host capacity, historical caps, and the observed result
cannot select or accept this candidate.

**069-051 T2 worker-repair verification (2026-07-16):** The test-only bounded
worker now invokes the PID-pinned live test harness rather than a pathname Cargo
may unlink during a concurrent replacement. The mandatory release bootstrap
then completed with storage 222/222 in 631.07s (including both real-Nova
workers), crypto 257/257, core 273/273, utils 167/167, wallet integration, and
release bench/example checks. A separate release verifier-worker run completed
in 612.074s with peak RSS 17,075,798,016 B. It confirms containment and the
PP/PK-free verifier-bundle path only; it neither establishes a numerical
operating budget nor completes the remaining semantic relation, mutation
ledger, or review convergence. Three current YOLO task-review passes are
non-converged: the first repaired the worker/test diagnostic; the latter two
retain the authority-budget and full-semantic-relation blockers. T2 remains
active; T3, T4, and Plan 06 remain locked.

**069-051 T2 static-preflight re-pin (2026-07-16):** The mandatory bootstrap
first caught stale assertion values for the now-live compact S1 relation rather
than a cap breach or setup OOM. The sole static-shape owner was then re-pinned
to C=108,722, V=66,670, NZ=415,965, G=524,289, PP lower bound=36,025,320 B,
VK lower bound=16,777,248 B, verifier-bundle lower bound=16,777,702 B, and
Pedersen RSS lower bound=100,663,488 B; the mandatory release bootstrap rerun
completed cleanly. These are diagnostic lower bounds for the base circuit, not
an augmented setup/prover RSS estimate, a selected width, a resource budget,
or permission to use the 24 GiB worker safety ceiling as an operating target.
T2 remains active pending the full semantic/mutation ledger, authority-pinned
resource acceptance, and review convergence; T3, T4, and Plan 06 remain locked.

**069-051 T2/S1 global-closure correction (2026-07-16):** Direct review of
the sole canonical trace grammar found that its global TracePrecommit
`BEGIN_HASH` precedes source replay while the matching global `END_HASH` follows
the final source record. The private Nova control relation therefore requires an
active and started global byte context on every source row; `FINALIZE_BLOCK`
arms `TraceClosure`, and only the schema-bound global TracePrecommit `END_HASH`
can transition that phase to finalized Idle after the successor transient cells
are zero. Source-local `END_HASH` remains a live closure row, not a self-loop or
terminal substitute. This retains one private circuit, two O(1) byte contexts,
the sole source encoder and SHA framing owner, and the exact constrained path
from canonical source bytes through `TraceChunk`, selected SHA blocks, FIPS
padding, and endpoint digests. The mandatory release bootstrap completed cleanly
with storage 224/224 and release bench/example checks; the current static
preflight remains C=108,722, V=66,670, NZ=415,965. It does not close S1's full
mutation ledger or T2's replay/uniqueness/net/JMT/hierarchy/statement relations,
does not select a SHA width or resource budget, and does not begin review
convergence. T2 remains active; T3, T4, and Plan 06 remain locked.

**069-051 T2 replay-prefix relation (2026-07-16):** The same private Nova
owner now uses three reserved running-state cells to constrain the canonical
replay grammar, not a native verdict or a second payload codec: `BeginBlock`
starts an input prefix, `ReplayInput` cannot occur after `ReplayOutput`, each
side has an in-circuit nonwrapping 16-bit counter, and
`UniquenessPrecommit` requires both sets jointly empty or nonempty. The final
schema-bound global trace END clears those cells with every other transient.
Three direct R1CS mutations cover input-after-output, the output-prefix
transition, and an unpaired precommit. The sole source header/chunk/SHA path and
two byte contexts are unchanged. The mandatory release bootstrap completed
cleanly with storage 227/227, real Nova proof and verifier-bundle checks,
wallet integration, and release bench/example checks. Measured static preflight
is now C=108,813, V=66,735, NZ=416,285, G=524,289, PP lower bound=36,040,304 B.
This does not decode replay payload semantics, prove uniqueness/net/JMT/
hierarchy/statement relations, select a SHA width or resource budget, or start
review convergence. T2 remains active; T3, T4, and Plan 06 remain locked.

**069-051 T2 canonical replay-payload binding (2026-07-16):** The same sole
private Nova owner parses `CanonicalFlowItemV2` directly from the bounded
meaningful bytes of the canonical `TraceChunk` feeder. It retains no payload
tape and accepts no host-decoded item: the parser constrains exact codec field
order, graphic transaction ID, fixed lower-hex identifier widths, serial width,
terminal text equal to the authenticated source-record object ID, terminal
leaf, and input first-seen flags. `ReplayInput`/`ReplayOutput` select the
replay set, not the storage operation: the payload `op_kind` is independently
and exactly constrained in R1CS as `Put=1 | Delete=2`. Release evidence covers
valid `ReplayInput/Put` and `ReplayOutput/Delete` schedules plus direct
`op_kind=3` rejection at `replay_payload/.../op_kind_canonical`; the terminal
object-ID mutation remains on its direct R1CS gate. The mandatory release
bootstrap completed cleanly with storage 231/231, both real-Nova paths, wallet
integration, and release bench/example checks; its real verifier-bundle worker
completed in 619.28 s. Current static preflight is C=237,651, V=171,700,
NZ=901,209, G=1,048,577, PP lower bound=75,306,592 B, VK lower bound=
33,554,464 B, verifier-bundle lower bound=33,554,918 B, and Pedersen RSS lower
bound=201,326,784 B. These are base-shape diagnostics only; they do not select
a width, accept an operating budget, close uniqueness/net/JMT/hierarchy/
statement relations, or start review convergence. T2 remains active; T3, T4,
and Plan 06 remain locked.

**069-051 T2 canonical uniqueness-precommit byte binding (2026-07-16):** The
same fixed-width source-byte context now streams the existing native
`UniquenessPrecommit` codec directly from its canonical `TraceChunk` bytes.
It accepts neither a host decoder nor another payload path: it requires the
exact 169-byte grammar and version `1`, materializes the two little-endian
32-bit counts and all five 32-byte digest fields as R1CS state, and at the
source `END_HASH` requires zero count high limbs plus equality of the low
limbs to the already constrained replay input/output counters. A canonical
native payload passes; source-byte mutations of its version and stored spent
count reach direct R1CS gates. The mandatory release bootstrap completed
cleanly with storage 231/231, real-Nova paths, wallet integration, and release
bench/example checks. The codec-derived uniqueness state boundary advances each
later reserved state family, so precommit fields cannot alias NET/JMT state.
Current base-shape preflight is C=237,651, V=171,700,
NZ=901,209, G=1,048,577, PP lower bound=75,306,592 B, VK lower bound=
33,554,464 B, verifier-bundle lower bound=33,554,918 B, and Pedersen RSS lower
bound=201,326,784 B. These are diagnostics only. This parser does not yet
prove the five digest meanings, original/sorted order or product, challenge,
net/JMT/hierarchy/statement relations, a selected width/budget, or review
convergence; T2 remains active and T3, T4, and Plan 06 remain locked.

**069-051 T2 post-change validation and review (2026-07-16):** After the
independent canonical `op_kind` relation was corrected, the full workspace
`cargo test --release`, `cargo build --release`, release feature-guard audit,
format check, and diff check passed. The focused fourth execution-review pass
found no further local defect in `{Put=1, Delete=2}`, replay-set independence,
or the direct `op_kind=3` R1CS rejection. It is not a globally clean review:
the authority-pinned per-role resource budget and complete
uniqueness/net/JMT/hierarchy/statement semantics remain unresolved. Therefore
T2 stays active and T3, T4, and Plan 06 remain locked.

**069-051 T2 canonical uniqueness-challenge/precommit linkage (2026-07-16):**
The same private fixed-width Nova owner now streams the native 65-byte
`UniquenessChallenge` codec from the existing canonical `TraceChunk` feeder.
It uses the canonical codec width/version constants rather than a circuit-local
grammar, retains no payload tape, and stores only its four bounded parser
cells plus the sixteen little-endian challenge limbs. During the 32 committed
precommit bytes, each parsed little-endian pair is constrained directly equal
to the stored fifth `UniquenessPrecommit` digest limb; the final 32 canonical
bytes materialize the challenge limbs. Version and committed-precommit-byte
mutations reach direct R1CS parser gates. This is deliberately not a native
assertion or a SHA/digest substitute: the authority-bound SHA-derived challenge
map, digest meanings, uniqueness ordering/product, net, JMT, hierarchy, and
statement relations remain open. The structural `UNIQUENESS_END` now includes
this parser before NET/JMT, preventing state aliasing. Mandatory release
bootstrap completed cleanly with storage 232/232, both real Nova paths, wallet
integration, and release bench/example checks; the storage suite took 718.47 s.
Re-measured base-shape preflight is C=261,266, V=190,904, NZ=992,779,
G=1,048,577, PP lower bound=79,536,152 B, VK lower bound=33,554,464 B,
verifier-bundle lower bound=33,554,918 B, and Pedersen RSS lower bound=
201,326,784 B. These base-shape values do not select width/budget, authorize
actual setup RSS, satisfy the mutation ledger, or begin review convergence;
T2 remains active and T3, T4, and Plan 06 remain locked.

**069-051 Gate 1 truth correction (2026-07-17):** The native V2 path now
binds the immutable pre-definition root, rejects a real cross-substituted JMT
envelope from a converging pre-state, eliminates the evaluator's spent/output
ID vectors, reconstructs `TraceChunk` from its sole source record, commits
native resident capacity, and commits exact declared/consumed opcode counts in
the transition commitment. The final clean bootstrap and targeted release tests
passed. This does **not** establish the current R1CS source-byte theorem: the
SHA compression lane is equality-bound to its context, but that context still
appends witness-supplied `TraceChunk` payload bytes rather than a canonical
source-record payload represented in R1CS. Earlier S1 prose claiming full
canonical-record equality is superseded. The first T2 implementation item is
that fixed-width payload relation;
no digest/native/reduced-cap substitute, width selection, T3/T4, or Plan 06 is
authorized.

**069-051 S1 authority amendment (2026-07-17):** The first T2 item cannot be
made sound by a local rename or another SHA assertion. The sole main path still
emits one derived `SourceMemoryWrite` immediately before the matching
`TraceChunk` from the same stack-local canonical chunk. The authority-reviewed
theorem amendment removes the fixed-depth Poseidon root/frontier after its
bounded worker exhausted the emergency address-space cap: that root had no
independent public or authority endpoint, so it duplicated the already
constrained ordered SHA transcript without adding a binding lemma. The writer
is constrained to the active source ordinal and exact next-chunk cursor;
pending state forces the immediate reader; direct R1CS equality covers every
metadata field, all 64 bytes, and zero tail; that reader enters both constrained
source/global SHA contexts with exact length, EOF, padding, chaining, and final
authority digest. No inner proof, second encoder/spool/owner, bounded arena,
native assertion, digest-only equality, or reduced-cap relation is introduced.

**069-051 S1 amended-path evidence (2026-07-17):** Direct byte, metadata,
cursor/order, zero-tail, source-context, and global-context adversarial tests
reject in R1CS, and every opcode retains one fixed shape. Sharing the mutually
exclusive writer/reader lane, replacing the quadratic zero-tail selector
matrix, decoding the three fixed semantic records by canonical chunk geometry,
and replacing terminal-ID's 64-way matrix with a six-bit direct-equality mux
tree reduced the source-only amended base ShapeCS to
`C=128,769/V=80,844/NZ=493,046`. The
unrelated additive `SOURCE_TRACE_ROOT` pseudo-endpoint is also deleted because
summing event-digest limbs was not a root theorem; final trace authority is now
only the expected digest constrained against the global SHA chaining state. This
places `NZ` below `2^19`, halves the static generator count again from
`1,048,577` to `524,289`, and pins lower bounds PP/VK/bundle/Pedersen RSS at
`39,589,688/16,777,248/16,777,702/100,663,488 B`. A fresh cache-bypassing,
PID-pinned source-only-revision worker completed the full setup, recursive proof, compressed proof,
verifier-bundle load, and verifier check under the unchanged 24 GiB/900 s
emergency limits with exit 0, peak RSS `16,829,259,776 B`, and wall
`343.966 s`. Augmented primary/secondary shapes are `C=303,019/10,349` and
`V=256,347/10,331`. The exact 40-step release measurement records setup `39.179 s`,
39 ordinary folds `6.999 s` (`179.465 ms/fold`), compression `148.280 s`,
verifier load/decode/check `126.381 s`, compressed proof `53,368 B`, and the
complete portable envelope `134,451 B`. The verifier bundle remains
`273,174,638 B`, proving that augmented Nova/Spartan material still dominates
VK distribution despite the smaller base threshold. The prior S1 emergency-cap resource blocker is closed and semantic
T2 work may continue. This is not an authority operating budget, candidate
selection, or T2 acceptance: net effect, full uniqueness, JMT, hierarchy,
root/statement/final-state relations, Models A/B/C, and S1-04 remain open, so
T3/T4 and Plan 06 stay locked.

📌 **069-051 T2 exact-P/uniqueness/accounting update (2026-07-18):** The canonical gap
ledger is now `069-051-T2-GAPS.md`. The sole private
`z00z_storage::checkpoint::recursive_v2::nova` owner consumes each 79-byte
Original/Sorted uniqueness record through its two authenticated
`SourceMemoryWrite`/`TraceChunk` windows. The one 353-byte challenge transcript
carries `P`, both set-specific `U` values and eight domain-separated SHA-256
outputs. A single typed authority path now builds exact declared semantic and
per-opcode work, its count digest, and `P` from version, chain/height/predecessor,
old settlement/definition roots, tx root, executable predicate, profile/spec/
grammar, and verifier-bundle identity. R1CS reconstructs the count and `P`
framing, both `U` values, all eight challenge digests, and all four list hashes
through the shared fixed FIPS lane. It then maps every output as
`2 + u248_le(d[0..31])`, reconstructs all sixteen little-endian `u16` ID
limbs, evaluates `e_beta`, and accumulates two full-field original/sorted grand
product pairs for both spent and output. NetMerge checks all four product
equalities and clears the eight product cells. Sorted rows are emitted in one
globally merged canonical order; per-set and global strict-order gates reject
intra-set duplicates, cross-set duplicates, and reorderings while four counters
remain bound to `UniquenessPrecommit`.

All seventeen expanded trace opcode classes have live non-wrapping consumed
counters. The schema-bound global TraceClosure `END_HASH` is the sole final
gate; it compares every counter with its declared anchor and clears the
transient family. The legacy `FinalizeBlock` final-gate overwrite was removed.

The current release ShapeCS pin is
`C=221,624/V=138,756/NZ=810,066/G=1,048,577`, with PP/VK/bundle/Pedersen lower
bounds `71,276,224/33,554,464/33,554,918/201,326,784 B`. Canonical replay is
now immediately paired with its commit-pass Original row; O(1) R1CS state binds
the exact set and all 32 terminal-ID bytes and rejects skipped or trailing pairs.
Focused release
mutations reach count/P framing bytes, a coherent predicate anchor, challenge/
precommit bytes, direct order/global cross-set/product gates, and the final
per-opcode equality. The mandatory resumed-cycle bootstrap was green before
this slice; focused post-change release check, shape, resource preflight,
exact-P transcript, and final-successor regressions pass. No complete-candidate
worker was run; every earlier proof/RSS/latency/envelope figure predates exact
P and final opcode accounting and remains historical diagnostics only.

This closes replay→Original identifier identity, but remains partial
`I_unique`/Net: identifier equality alone does not establish path/value net
effect or safe unchanged-leaf omission. The frozen
`n_max/q_U/q_V` bound, symbolic/toy-field corpus, Models A/B/C, semantic Net,
JMT/hierarchy/roots, statement/X_h/prior-IVC/final relations, final artifacts,
complete mixed-block proof, reopened T1 evidence, and authority budget remain
open. Developer caches and Celestia integration remain outside T2 as specified
by the gap ledger. T2 is active; T3/T4/Plan 06 remain locked.

**069-051 developer cache boundary (2026-07-17):** `.cache/` has no reusable
Nova material today. After the S1 relation and its shape stabilize, an opt-in,
private PP+PK setup cache may use only a SHA-256 digest of a versioned,
length-delimited identity preimage as its directory key. That preimage includes
the cache role/format, Nova suite/features, authority/profile/spec/grammar/
shape/source/lockfile/manifest digests, and running-state arity; names and
branches are never keys. Strict, capped, canonical decode plus private atomic
writes are mandatory. Proofs, receipts, VKs/verifier bundles, source payloads,
and acceptance results are excluded. Every bounded-worker release measurement
bypasses the cache, so a hit cannot select a SHA width, establish a resource
budget, or advance T2.

🔑 **069-051 T2 full-row, semantic-Net, and JMT-header update (2026-07-18):**
The one authenticated uniqueness element is now the complete 100-byte tuple
`definition_id || serial_id_le || terminal_id || leaf_value_hash`. Replay
derives it from canonical source bytes; the adjacent Original row, all four
list SHA commitments, both product pairs, strict ordering, and semantic Net
consume the same row. A globally repeated terminal is accepted only for one
spent/output pair on the exact same definition/serial path. Delete-only and
insert-only blocks are no longer rejected by a false set-cardinality equality.

The sole 134-byte Net codec now proves exact Delete, Insert, Replace,
Unchanged, and Close rows with one bounded pending spent/output pair. It binds
the exact path and old/new leaf hashes, requires zero on an absent side,
distinguishes changed versus unchanged leaves, rejects changed-path
replacement, and compares effect/mutation counts with the declared Net/JMT
work. Close directly binds the previously constrained precommit, `P`, spent
`U`, and output `U`; it introduces no native verdict or digest-only substitute.

Circuit-spec version 6 retains fifty semantic-row limbs, degree 49 per row,
248-bit mapped challenges, and the separate A-16 `log2(q_U)=128` assumption.
The repository profile commits `n_max=16,000` per set. Symbolic formal-
polynomial and exhaustive toy-field release tests cover unequal/equal
multisets, duplicates, zero factors, the corrected `49n` ceiling, and squared
two-pair collision accounting. Operational `q_V` remains an external authority
resource input and cannot replace `q_U`.

The same authenticated TraceChunk stream now decodes the fixed JMT envelope
header and PromoteChildRoot record in R1CS. It binds the envelope version,
SettlementV2 generation, mutating/no-op kind, trace digest, declared update
count, exact no-op digest and no-trailing-byte width, then requires Promote to
repeat the envelope digest while materializing its definition root. Focused
release mutations reject a changed count, no-op digest, no-op width, and
promoted digest. All historical `#[cfg(any())]` wide/digest-only parser bodies
were removed, leaving one canonical implementation path.

The superseded pre-micro release ShapeCS pin was
`C=272,812/V=163,847/NZ=989,221/G=1,048,577`, with static
PP/VK/bundle/Pedersen lower bounds
`79,670,936/33,554,464/33,554,918/201,326,784 B`. Focused release tests reach direct
gates for all four replay fields, per-set/global order, four product equalities,
Net kind, pending old row, Close transcript binding, effect/mutation counts,
and the JMT-header/Promote mutations above. No fresh real-Nova worker was run because this is not the complete
relation; prior proof/RSS/latency/size figures remain historical diagnostics.
Mutating JMT operation/proof/path algebra, terminal→bucket→serial→definition
induction, settlement roots, typed commitments, `X_h`, prior IVC, exact final
successor, Models A/B/C, final artifacts, the authority budget, and reopened T1
evidence still block T2. T3/T4 and Plan 06 remain locked.

🔑 **069-051 T2 single JMT-micro path update (2026-07-18):** Circuit-spec v6
removes the duplicate source representation that previously emitted both the
opaque JMT envelope body and derived micro records. The sole canonical source
is now one fixed 39-byte authenticated header followed by bounded
`JmtMicroOp` update/operation/value/proof/end records. A storage-owned streaming
inverse decoder reconstructs typed updates, enforces record order, indices and
canonical value chunking, re-runs project raw-SHA semantics plus pinned-JMT and
hierarchy verification, and compares the recomputed micro transcript digest.
It retains no opaque-envelope spool copy; digest derivation also no longer deep
clones `updates`. The profile's source-record cap is derived from the same
content-byte authority rather than the obsolete max-leaf envelope chunk count.

R1CS now requires exactly one fixed-width header, constrains micro-op
version/kind/update/operation framing and completed counts, and keeps Promote
bound to the header digest. The exact release ShapeCS pin is
`C=273,537/V=164,495/NZ=991,969/G=1,048,577`; static
PP/VK/bundle/Pedersen lower bounds are
`79,798,256/33,554,464/33,554,918/201,326,784 B`. Focused release tests and the
mandatory bootstrap pass (`z00z_storage` 248 passed/0 failed/2 milestone-only
ignored; `z00z_utils` 167/167; wallet/compile checks green). This closes the
duplicate-envelope resource/canonical-path gap only. Mutating JMT raw-SHA
path/case/root constraints, hierarchy/settlement-root/commitment/`X_h`/final
relations, Models A/B/C, artifacts, and the authority budget still block T2;
T3/T4 and Plan 06 remain locked.

🔑 **069-051 T2 authenticated JMT transition update (2026-07-18):** The sole
bounded `JmtMicroOp` stream is now a complete R1CS old/path/new-root machine,
not only a framed parser. It constrains the old leaf/null record, exact
leaf/value and parent raw-SHA preimages, sibling order and direction, split
common-prefix count and former-leaf/null prelude, the six insert/update/delete
cases (including delete coalescence and preserved internal parents), declared
new-root equality, and old-root continuity between successive updates. Real
pinned-JMT fixtures cover all six cases plus a two-operation chain. Direct
mutations reject split count/direction/former leaf/parent changes, active/case
aliasing, and a changed declared new root. The release base ShapeCS pin is
`C=325,091/V=206,541/NZ=1,221,306/G=2,097,153`; static
PP/VK/bundle/Pedersen lower bounds are
`123,763,464/67,108,896/67,109,350/402,653,376 B`. The sole transcript now
places prior-value blocks before new-value blocks and R1CS binds their raw SHA,
presence, and key to the authenticated old proof leaf. The mandatory bootstrap is
green (`z00z_storage` 250 passed/0 failed/2 ignored; remaining bootstrap gates
green). This closes the per-update JMT transition relation only. Canonical
terminal→bucket→serial→definition induction, SettlementV2 roots and typed
commitments, `X_h`/prior-IVC/exact final successor, Models A/B/C, complete
measurement, and the authority budget still block T2; T3/T4 and Plan 06 remain
locked.

🔑 **069-051 T2 hierarchy-transition update (2026-07-18):** The single
authenticated JMT relation now retains one canonical hierarchy state machine.
It proves terminal → bucket → serial → definition → optional path-index stage
order, strict role ordering inside each stage, and non-equal old/new roots for
every hierarchy-bearing update. Parent prior/new values are decoded from the
same raw-SHA blocks already authenticated by the JMT proof relation and bind
their exact definition/serial/bucket coordinates and child roots.

Unused or duplicated children are rejected without a retained map: for each of
terminal→bucket, bucket→serial, and serial→definition, R1CS evaluates two
independently challenged products over the fixed 133-byte transition row and
requires both child/parent products plus exact counts to match at the sole
mutating promotion. Exact-P codec version 2 adds `update_trace_digest` as the
twelfth digest anchor; the JMT header must equal it, making these challenges
non-adaptive. The stale constraint that incorrectly required all hierarchy
state to be zero at `PromoteChildRoot` was removed. A real four-level hierarchy
fixture now reaches and satisfies promotion in release TestCS.

The current exact base ShapeCS pin is
`C=337,927/V=216,219/NZ=1,266,770/G=2,097,153`; static
PP/VK/bundle/Pedersen lower bounds are
`125,890,088/67,108,896/67,109,350/402,653,376 B`. The resumed mandatory
bootstrap passed before this semantic slice. This is not full hierarchy or T2
closure: exact SettlementV2 Goldilocks Poseidon2 Serial/Definition operation-key
derivation, settlement roots/typed commitments, statement/`X_h`/prior-IVC/final
successor, Models A/B/C, complete artifacts/measurement, and the authority
budget remain open. T3/T4 and Plan 06 remain locked.

🔑 **069-051 T2 canonical-flow and Net→terminal-JMT update (2026-07-18):**
The semantic relation now proves that every non-Unchanged Net effect is
consumed exactly once by the terminal JMT scheduler. Because Net rows are
globally terminal-sorted while terminal operations are hierarchy-grouped,
R1CS compares two independently challenged products over the exact 132-byte
row `definition || serial_le || terminal || old_hash || new_hash`, plus exact
count, at the unique promotion gate. Insert/delete select a constrained zero
for the absent value; replacement binds both value hashes. A retained release
mutation changes the JMT value hash and reaches the direct product equality.

The redundant second encoding of every flow item under `CommitTypedEvent` was
removed from the sole producer and evaluator. `ReplayInput`/`ReplayOutput` are
now the one canonical flow-item path; `CommitTypedEvent` accepts only the four
ordered checkpoint-core commitments bound from X_h. The fixed control machine
retains Commit→Commit solely for those four records, and its R1CS first-chunk
gate rejects the former Put/Delete tags directly. Current exact base shape
is `C=533,905/V=401,549/NZ=2,036,951/G=2,097,153`; lower bounds are
PP/VK/bundle/Pedersen
`161,400,800/67,108,896/67,109,350/402,653,376 B`.

T2 is not closed. Complete mixed real-proof evidence, final Models A/B/C,
artifact/theorem/A-17 ledgers, reopened T1 evidence, three-review/two-clean
convergence and an authority-pinned numeric operating budget remain mandatory.
The developer-only setup cache is optional and deferred until relation
stabilization; folding recovery and Celestia DA publication remain T3/later
integration work. T3/T4 and Plan 06 remain locked.

**Success Criteria:**

1. All 13 plans execute in dependency order against the existing phase folder.
2. Phase 068 schemas and validators are extended, never duplicated.
3. Real Nova and Plonky3 verification satisfy the acceptance and rejection gates
   before any authority-promotion claim.
