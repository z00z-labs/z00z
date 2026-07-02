<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->
# Phase 065: Attack-Surface - Context

**Gathered:** 2026-06-30  
**Status:** Planned from the current-tree Phase 065 corpus on the existing
`.planning/phases/065-Attack-Surface/` directory only  
**Source:** PRD Express Path (`.planning/phases/065-Attack-Surface/065-TODO.md`
plus live code anchors re-checked on 2026-06-30)

<domain>
## 🎯 Phase Boundary

Phase 065 turns `.planning/phases/065-Attack-Surface/065-TODO.md` into an
ordered executable closure packet for the remaining attack-surface backlog.
`065-TODO.md` is normative, not advisory. The existing
`.planning/phases/065-Attack-Surface/` folder is the only canonical Phase 065
root. Historical Phase 065 Markdown reports and JSONL catalogs are superseded
inputs, not a second planning authority.

### Current-Tree Planning Rule

- `.planning/phases/065-Attack-Surface/065-TODO.md` contains **zero** canonical
  `TASK-NNN` rows.
- `.planning/phases/065-Attack-Surface/065-TODO.md` contains **nine** canonical
  workstreams: `WS-01` through `WS-09`.
- Phase 065 planning must therefore preserve the existing `WS-*` namespace as
  the only normative execution-unit inventory. No fake `TASK-NNN` namespace may
  be invented to satisfy older prompts.
- The generated plan packet materializes one executable grouped plan per
  workstream: `PLAN-065-G01` through `PLAN-065-G09`, stored as
  `065-01-PLAN.md` through `065-09-PLAN.md`.
- Verification-report residuals are carried only as an additive remediation
  packet: `PLAN-065-G10` through `PLAN-065-G13`, stored as
  `065-10-PLAN.md` through `065-13-PLAN.md`. These additive plans do not
  replace, rename, or remap the canonical `WS-*` namespace.

### Phase 065 Delivers

1. `.planning/phases/065-Attack-Surface/065-CONTEXT.md` with the coverage
   answer, gate-to-plan mapping, and live current-code anchors.
2. Ordered executable plans
   `.planning/phases/065-Attack-Surface/065-01-PLAN.md` through
   `.planning/phases/065-Attack-Surface/065-09-PLAN.md`.
3. One canonical local closure packet for theorem-verified validator
   acceptance, canonical checkpoint persistence, build hardening, simulator
   evidence truth, privileged wallet capability sealing, canonical wallet
   mutation and restore ownership, fail-closed boundary construction and
   redaction, placeholder public RPC demotion or implementation, and the final
   narrowed-source sweep.
4. One additive verification-remediation packet
   `.planning/phases/065-Attack-Surface/065-10-PLAN.md` through
   `.planning/phases/065-Attack-Surface/065-13-PLAN.md` for the still-open
   residuals from `z00z-verification-report-1.md` through
   `z00z-verification-report-4.md`.

### Phase 065 Does Not Deliver

- No invented `TASK-NNN` inventory.
- No restored historical Markdown reports as live authority.
- No placeholder-only, scaffold-only, panic-only, string-only, no-op, or
  compile-only closure path.
- No docs-only proof for code behavior.
- No live claim for remote chain transport, remote DA transport, remote worker
  processes, or third-party network availability when the repo still only owns
  local deterministic primitives.
- No edits under `crates/z00z_crypto/tari/**`.

</domain>

<decisions>
## ⚙️ Locked Decisions

- **D-01:** `.planning/phases/065-Attack-Surface/065-TODO.md` is the single
  canonical planning authority for Phase 065.
- **D-02:** Coverage is phase-failing unless all **9** canonical workstreams,
  all **9** normative grouped plan ids, and all **4** additive residual grouped
  plan ids in the current Phase 065 packet are present and mapped exactly once.
- **D-03:** The older `TASK-NNN` wording in the invoking prompt is satisfied by
  an explicit current-tree exception: there are zero `TASK-NNN` rows to map, so
  the only valid coverage unit is the existing `WS-*` namespace. No fake `TASK-NNN`
  namespace may be invented.
- **D-04:** `PLAN-065-G01` through `PLAN-065-G09` keep the same priority order
  as the TODO: `WS-01`, `WS-02`, `WS-03`, `WS-04`, `WS-05`, `WS-06`, `WS-07`,
  `WS-08`, `WS-09`.
- **D-05:** Gate ownership is fixed:
  - `WS-01` owns `G-01` and `G-02`
  - `WS-02` owns `G-03` and `G-04`
  - `WS-05` owns `G-05`
  - `WS-06` owns `G-06` and `G-07`
  - `WS-08` owns `G-08` and `G-09`
- **D-06:** The permanent repository-wide meta-gates are execution scope, not
  commentary. Their CI/source-audit enforcement is attached to `PLAN-065-G07`
  because `WS-07` is the explicit fail-closed boundary and redaction owner.
- **D-07:** `.planning/phases/065-Attack-Surface/065-TODO.md` does not point to
  any additional Markdown source rows that must be read before planning. The
  `.md` filenames named in the TODO are superseded deletion-candidates, not
  normative implementation sources.
- **D-08:** Every generated plan must carry the user-specified matrix fields:
  `plan_id`, `task_ids`, `copied_task_rows`, `source_refs`, `inputs`,
  `outputs`, `dependencies`, `acceptance_tests`, `simulation_gate`,
  `negative_tests`, `plan_artifacts`, `plan_tests`, `plan_results`,
  `task_artifacts`, `task_tests`, `task_results`, `anti_placeholder_gate`,
  `current_code_refs`, `blockers`, `evidence_gate`, and
  `not_recommendation_gate`.
- **D-09:** Every generated `<verify>` block starts with
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, runs
  slice-specific commands, runs `cargo test --release` when Rust or tests are
  affected, runs `/GSD-Review-Tasks-Execution` at least three times until two
  consecutive runs are clean, and uses `/z00z-git-versioning` for any commit.
- **D-10:** Only external transport, remote process boundaries, external DA
  transport, and unavailable third-party network or clock/fault scheduling may
  be simulated. Rollup theorem verification, checkpoint sealing, publication
  binding, wallet lifecycle persistence, restore rollback, route tables, and
  validator or watcher checks must stay on real project primitives.
- **D-11:** Graphify may help orient the codebase, but it is not factual
  authority for Phase 065 coverage, invariants, or acceptance claims.
- **D-12:** Verification reports `z00z-verification-report-1.md` through
  `z00z-verification-report-4.md` are additive authority only for still-open
  residual verification findings; they do not reopen retired or superseded
  attack-surface wording from the deleted Phase 065 corpus.
- **D-13:** The additive residual packet is fixed to four execution units:
  `VR-10`, `VR-11`, `VR-12`, and `VR-13`, materialized as
  `PLAN-065-G10` through `PLAN-065-G13`.
- **D-14:** Later verification reports win over earlier ones for status
  reconciliation. Findings that later runs already upgraded to
  `BOUNDED_VERIFIED`, `FORMALLY_PROVED`, or another stronger passing state are
  excluded from live residual scope unless fresh current-tree code evidence
  reopens them.
- **D-15:** Protected-vendor concentration findings under
  `crates/z00z_crypto/tari/**` remain report-only. Residual closure must come
  from project-owned wrapper, domain, RPC, storage, and verification-path
  controls without editing the vendor subtree.

</decisions>

<threat_model>
## 🛡️ Threat Model And Trust Boundaries

- **Assets:** settlement theorem bundles, checkpoint artifacts and links,
  snapshot and exec-input bindings, validator verdicts, privileged wallet
  operations, restore durability, wallet tx lifecycle truth, transport logs,
  release-only feature matrices, public RPC contracts, and repository wording
  about closed historical claims.
- **Adversaries:** partial theorem inputs, raw checkpoint write lanes, build
  flags that re-enable debug surfaces, draft publication evidence that looks
  final, handler-level guard omissions, helper-composition drift, constructor
  panics, raw transport logging, placeholder DTOs presented as canonical truth,
  and stale docs that re-promote narrowed findings as live.
- **Failure model:**
  - compile-only, grep-only, docs-only, or placeholder-only closure is invalid;
  - any second authority lane for checkpoint truth, wallet mutation truth,
    restore truth, publication truth, or feature-policy truth is phase-failing;
  - any public API that still looks production-capable while remaining
    placeholder or synthetic is phase-failing unless the live-looking claim is
    explicitly removed from the contract.

</threat_model>

<canonical_refs>
## 📚 Canonical References

### Planning Authority

- `.planning/phases/065-Attack-Surface/065-TODO.md`
- `.planning/phases/065-Attack-Surface/065-CONTEXT.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-1.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-2.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-3.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-4.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.github/copilot-instructions.md`
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`

### Live Code Anchors By Workstream

- `WS-01`
  - `crates/z00z_rollup_node/src/lib.rs`
  - `crates/z00z_rollup_node/src/da.rs`
  - `crates/z00z_runtime/validators/src/verdict.rs`
  - `crates/z00z_runtime/validators/src/checkpoint.rs`
  - `crates/z00z_runtime/validators/src/engine.rs`
- `WS-02`
  - `crates/z00z_storage/src/checkpoint/store.rs`
  - `crates/z00z_storage/src/checkpoint/link.rs`
  - `crates/z00z_storage/tests/test_checkpoint_store.rs`
  - `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
  - `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`
- `WS-03`
  - `crates/z00z_storage/src/settlement/hjmt_cache.rs`
  - `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`
  - `crates/z00z_wallets/src/db/mod.rs`
  - `crates/z00z_wallets/src/wallet/mod.rs`
  - `crates/z00z_simulator/Cargo.toml`
  - `crates/z00z_utils/src/lib.rs`
- `WS-04`
  - `crates/z00z_simulator/src/config.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`
  - `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `WS-05`
  - `crates/z00z_wallets/src/services/wallet_session_runtime_limits.rs`
  - `crates/z00z_wallets/src/rpc/key_rpc_server_admin.rs`
  - `crates/z00z_wallets/src/stealth/output.rs`
  - `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs`
- `WS-06`
  - `crates/z00z_wallets/src/rpc/asset_rpc_support_state.rs`
  - `crates/z00z_wallets/src/chain/broadcast_impl.rs`
  - `crates/z00z_wallets/src/services/wallet_actions_backup.rs`
  - `crates/z00z_wallets/src/services/wallet_store_export_pack.rs`
- `WS-07`
  - `crates/z00z_storage/src/settlement/store.rs`
  - `crates/z00z_networks/rpc/src/wasm_client.rs`
  - `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`
  - `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`
- `WS-08`
  - `crates/z00z_wallets/src/services/chain_service.rs`
  - `crates/z00z_wallets/src/rpc/chain_rpc.rs`
  - `crates/z00z_wallets/src/rpc/chain_rpc_impl.rs`
  - `crates/z00z_wallets/src/rpc/tx_types.rs`
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_admission.rs`
- `WS-09`
  - `.planning/codebase/STRUCTURE.md`
  - `.planning/codebase/ARCHITECTURE.md`
  - `.planning/phases/profiling-comprehensive.md`
  - `crates/z00z_core/README.md`
  - `crates/z00z_core/src/genesis/README.md`
  - `crates/z00z_core/src/assets/mod.rs`
  - `crates/z00z_core/src/assets/registry_catalog.rs`
  - `crates/z00z_storage/src/settlement/root_types.md`

### Verification-Report Residual Anchors

- `VR-10`
  - `.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh`
  - `.github/skills/z00z-verification-orchestrator/SKILL.md`
  - `.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`
  - `.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh`
  - `.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`
- `VR-11`
  - `.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-hax.sh`
  - `.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-tamarin.sh`
  - `.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh`
  - `.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-miri.sh`
  - `.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh`
  - `.github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-short.sh`
  - `scripts/install-verification-tools.sh`
  - `scripts/verify-env.sh`
  - `.cargo/config.toml`
- `VR-12`
  - `crates/z00z_runtime/aggregators/src/batch_planner.rs`
  - `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`
  - `crates/z00z_runtime/aggregators/src/dist_dispatch.rs`
  - `crates/z00z_storage/src/backend/query.rs`
  - `crates/z00z_storage/src/backend/redb/helpers.rs`
  - `crates/z00z_storage/src/checkpoint/audit.rs`
  - `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`
  - `crates/z00z_storage/src/settlement/timing.rs`
- `VR-13`
  - `crates/z00z_crypto/src/domains.rs`
  - `crates/z00z_crypto/src/hash/policy.rs`
  - `crates/z00z_crypto/src/claim.rs`
  - `crates/z00z_crypto/src/error.rs`
  - `crates/z00z_wallets/src/rpc/app_types.rs`
  - `crates/z00z_wallets/src/rpc/asset_ownership_check.rs`
  - `crates/z00z_wallets/src/rpc/asset_rpc_impl.rs`

</canonical_refs>

<normative_mirror>
## Executive conclusion

| TODO class | Current Phase 065 meaning | Owner plans |
| --- | --- | --- |
| still-live implementation gaps | executable backlog only; no retired claim may be silently reopened | `PLAN-065-G01` through `PLAN-065-G08` |
| boundaries sealed mostly by current code plus regression proof | keep explicit regression coverage; do not demote these items to prose-only notes | `PLAN-065-G02`, `PLAN-065-G04`, `PLAN-065-G05`, `PLAN-065-G06`, `PLAN-065-G07` |
| historical or overbroad claims | do not carry the old wording forward verbatim; keep only narrowed live remainders and doc-truth demotion | `PLAN-065-G01`, `PLAN-065-G06`, `PLAN-065-G08`, `PLAN-065-G09` |

## Deletion Contract

| Superseded corpus slice | Files absorbed by `065-TODO.md` | Why deletion is safe now |
| --- | --- | --- |
| old Phase 065 Markdown authority | `readme.md`, `attack-surfaces-resolve-spec.md`, `attack-surfaces-TODO.md`, `attack-surfaces-AUDIT-2.md`, `attack-surfaces-AUDIT-3.md`, `attack-surfaces-placeholders.md`, `attack-surfaces-wallet-simul.md`, `060-attack-surface-report.md`, `062-attack-surface-report.md`, `064-attack-surface-report.md` | backlog authority, gate semantics, and finding disposition now live in `065-TODO.md` plus this context and the grouped plans |
| phase-local JSONL catalogs | `060-attack-surface-db.jsonl`, `062-attack-surface-db.jsonl`, `064-attack-surface-db.jsonl`, `attack-surface-db.jsonl`, `attack-surface-crates.jsonl`, `attack-surface-crates-symbols.jsonl` | these are evidence or inventory snapshots, not unique implementation requirements |
| generated inventory or snapshot docs | `attack-surface-crates-report.md`, `attack-surface-crates-inventory.md`, `attack-surface-crates-security-snapshot.md` | package, advisory, and symbol inventory meaning is absorbed as disposition or meta-gate policy rather than live backlog rows |
| run-local verification report | `z00z-verification-report.md` | exact run-local status names are reduced to enduring verify and closure gates; the report is not a second authority source |

- Backlog authority now lives in `065-TODO.md`, this context file, and the
  grouped executable plans only.
- Finding coverage now lives in the active disposition and owner maps below.
- Gate contracts now live in the fixed `G-01` through `G-09` ownership map and
  in each owner plan's matrix.
- Remaining non-TODO files are evidence breadcrumbs only.

### Absorbed non-backlog artifacts

- `attack-surface-crates-inventory.md` is an environment and dependency
  inventory only.
- `attack-surface-crates-security-snapshot.md` is advisory context only.
- `attack-surface-crates-symbols.jsonl` is search aid only.
- `z00z-verification-report.md` contributes only enduring gate families:
  `l0-docs`; `l1-alloy`, `l1-apalache`, `l1-tla`; `l2-cryptol`, `l2-domain`,
  `l2-proverif`, `l2-refinement-map`, `l2-saw`, `l2-tamarin`, `l2-transcript`;
  `l3-kani`, `l3-miri`, `l3-verify-fast`; `l4-adversarial-review`,
  `l4-constant-time`, `l4-fuzz`, `l4-supply-chain`, `l4-unsafe`.

## Verification model used for this consolidation

Every carried Phase 065 item passed all three filters below:

1. it existed in a historical Phase 065 report or attack-surface catalog;
2. live code and current tests were re-checked on `2026-06-30`;
3. it was classified as `Open`, `Seal`, or `Historical/Narrowed`, and only the
   first two remain execution scope.

## Legacy CF Numbering Note

- Legacy `CF-003` split into two different historical meanings:
  raw-checkpoint or compatibility-proof semantics now live under `WS-02`, while
  old rotate-master-key placeholder wording is narrowed into `WS-06`.
- Legacy `CF-005` is not a current production-trusted-path blocker. Reopen it
  only if a default or public build depends on the guarded `claim_v1`
  compatibility lane again.

## Priority order

The execution order is fixed and unchanged from `065-TODO.md`:
`WS-01` -> `WS-02` -> `WS-03` -> `WS-04` -> `WS-05` -> `WS-06` -> `WS-07` ->
`WS-08` -> `WS-09`, with the same dependency chain across
`PLAN-065-G01` through `PLAN-065-G09`.

### Exact Priority Order Mirror

This subsection mirrors the exact numbered order from `065-TODO.md`. It is a
coverage mirror only and does not replace the TODO as authority.

1. `WS-01` theorem-verified validator acceptance
2. `WS-02` canonical checkpoint persistence and proof semantics
3. `WS-03` release/build hardening for debug and test-only surfaces
4. `WS-04` release-packet truth for simulator draft/debug lanes
5. `WS-05` capability-typed sealing for privileged wallet paths
6. `WS-06` canonical ownership of wallet mutation and restore truth
7. `WS-07` fail-closed construction and operator-visible logging
8. `WS-08` placeholder public RPC and stub DTO cleanup
9. `WS-09` document and source sweep for the few narrowed historical leftovers

## Canonical Gate Inventory: Inputs, Outputs, Verification, Proofs

| Gate | Owner workstream | Owner plan | Current-tree contract source |
| --- | --- | --- | --- |
| `G-01` Rollup Settlement Theorem Gate | `WS-01` | `PLAN-065-G01` | `065-TODO.md` lines `133-161` |
| `G-02` Validator Checkpoint And Publication Gate | `WS-01` | `PLAN-065-G01` | `065-TODO.md` lines `163-187` |
| `G-03` Checkpoint Seal Gate | `WS-02` | `PLAN-065-G02` | `065-TODO.md` lines `189-214` |
| `G-04` Checkpoint Link Bind And Codec Gate | `WS-02` | `PLAN-065-G02` | `065-TODO.md` lines `216-243` |
| `G-05` Privileged Session Gate | `WS-05` | `PLAN-065-G05` | `065-TODO.md` lines `245-268` |
| `G-06` Wallet Mutation Submission Gate | `WS-06` | `PLAN-065-G06` | `065-TODO.md` lines `270-295` |
| `G-07` Atomic Restore Gate | `WS-06` | `PLAN-065-G06` | `065-TODO.md` lines `297-323` |
| `G-08` Public Chain Scan And Tip RPC Gate | `WS-08` | `PLAN-065-G08` | `065-TODO.md` lines `325-349` |
| `G-09` Transaction Receipt And Verification DTO Gate | `WS-08` | `PLAN-065-G08` | `065-TODO.md` lines `351-377` |

## 📊 Coverage Audit

| Inventory item | Count | Decision |
| --- | ---: | --- |
| Canonical `TASK-NNN` rows in `065-TODO.md` | 0 | Must not be invented |
| Canonical workstreams in `065-TODO.md` | 9 | `WS-01` through `WS-09` |
| Canonical gates in `065-TODO.md` | 9 | `G-01` through `G-09` |
| Required normative workstream plans | 9 | `PLAN-065-G01` through `PLAN-065-G09` |
| Required additive verification-remediation plans | 4 | `PLAN-065-G10` through `PLAN-065-G13` |
| Total plan ids in the current Phase 065 packet | 13 | `PLAN-065-G01` through `PLAN-065-G13` |
| Active disposition rows | 33 | 22 `Open`, 6 `Seal`, 5 `Policy gate` |
| Task-row Markdown sources to read before planning | 0 | No canonical task-row links exist in `065-TODO.md` |

## 🔐 Coverage Rule

| Rule | Decision |
| --- | --- |
| Canonical unit of execution | `WS-01` through `WS-09` |
| Additive verification residual units | `VR-10` through `VR-13` |
| Alternate fake task namespace | Forbidden |
| Grouped plan ids | `PLAN-065-G01` through `PLAN-065-G13` |
| Atomic fallback ids | Not introduced during planning because no canonical `TASK-NNN` rows exist |
| Requirement status | Every `WS-*` section and every gate/meta-gate obligation is normative |
| Required implementation reading | `065-TODO.md`, `065-CONTEXT.md`, verification-report residual sources, and the relevant live code anchors for the owning workstream or residual unit |

No fake `TASK-NNN` namespace may be invented.

## 🔢 Workstream-To-Plan Coverage Table

| Normative unit | Plan file | Plan id | Gates | Source findings | Primary owner surfaces | Dependency |
| --- | --- | --- | --- | --- | --- | --- |
| `WS-01` | `065-01-PLAN.md` | `PLAN-065-G01` | `G-01`, `G-02` | `AS-20260627-001`, `AS-20260630-064-11`, narrowed `CF-006` remainder | rollup theorem verifier + validator acceptance | none |
| `WS-02` | `065-02-PLAN.md` | `PLAN-065-G02` | `G-03`, `G-04` | `AS-20260630-064-10`, `AS-20260501-029`, `AS-20260501-030`, `CF-002`, carried `CF-010` parts | checkpoint store + link binding | `PLAN-065-G01` |
| `WS-03` | `065-03-PLAN.md` | `PLAN-065-G03` | build-policy companion scope | crate `AS-20260623-001`, crate `AS-20260623-003`, `AS-20260501-027`, `AS-20260501-031` | release feature gating + debug-surface removal | `PLAN-065-G02` |
| `WS-04` | `065-04-PLAN.md` | `PLAN-065-G04` | simulator publication-evidence companion scope | `AS-20260630-064-01`, `AS-20260630-064-07`, narrowed `CF-002` remainder | simulator proof mode + public packet evidence | `PLAN-065-G03` |
| `WS-05` | `065-05-PLAN.md` | `PLAN-065-G05` | `G-05` | `AS-20260630-064-03`, `AS-20260630-064-05`, `AS-20260630-064-06` | verified session capabilities + stealth output API | `PLAN-065-G04` |
| `WS-06` | `065-06-PLAN.md` | `PLAN-065-G06` | `G-06`, `G-07` | `AS-20260630-064-02`, `AS-20260630-064-04`, narrowed rotate wording remainder | mutation executor + restore journal | `PLAN-065-G05` |
| `WS-07` | `065-07-PLAN.md` | `PLAN-065-G07` | fail-closed boundary + all permanent meta-gates | `AS-20260627-002`, crate `AS-20260623-002`, logging/panic cluster, policy-gate rows | fallible constructors + log redaction + CI audits | `PLAN-065-G06` |
| `WS-08` | `065-08-PLAN.md` | `PLAN-065-G08` | `G-08`, `G-09` | `CF-004`, `CF-007`, `CF-008`, carried `CF-010` parts | chain scan/tip public contract + receipt DTO semantics | `PLAN-065-G07` |
| `WS-09` | `065-09-PLAN.md` | `PLAN-065-G09` | doc truth closeout companion scope | `AS-20260623-001` from `060-attack-surface-db.jsonl`, narrowed leftovers | repo docs/examples/readmes + attack-surface wording audits | `PLAN-065-G08` |

## Exact TODO Workstream Source Mirrors

This section mirrors the exact `Sources:` bullets from `065-TODO.md` and maps
them to owner plans. It is a coverage mirror only and does not replace the TODO
as authority.

### `WS-01` -> `PLAN-065-G01`

- `AS-20260627-001`
- `AS-20260630-064-11`
- narrowed remainder of old `CF-006`

### `WS-02` -> `PLAN-065-G02`

- `AS-20260630-064-10`
- `AS-20260501-029`
- `AS-20260501-030`
- `CF-002`
- carried parts of `CF-010`

### `WS-03` -> `PLAN-065-G03`

- crate `AS-20260623-001`
- crate `AS-20260623-003`
- `AS-20260501-027`
- `AS-20260501-031`

### `WS-04` -> `PLAN-065-G04`

- `AS-20260630-064-01`
- `AS-20260630-064-07`
- narrowed simulator remainder of old `CF-002`

### `WS-05` -> `PLAN-065-G05`

- `AS-20260630-064-03`
- `AS-20260630-064-05`
- `AS-20260630-064-06`

### `WS-06` -> `PLAN-065-G06`

- `AS-20260630-064-02`
- `AS-20260630-064-04`
- narrowed remainder of old rotate-master-key wording

### `WS-07` -> `PLAN-065-G07`

- `AS-20260627-002`
- crate `AS-20260623-002`
- legacy logging and panic cluster:
  - `AS-20260501-002`
  - `AS-20260501-004`
  - `AS-20260501-007`
  - `AS-20260501-011`
  - `AS-20260501-020`
  - `AS-20260501-023`
  - `AS-20260501-024`
  - `AS-20260501-025`
  - `AS-20260501-026`

### `WS-08` -> `PLAN-065-G08`

- `CF-004`
- `CF-007`
- `CF-008`
- carried placeholder parts of `CF-010`

### `WS-09` -> `PLAN-065-G09`

- `AS-20260623-001` from `060-attack-surface-db.jsonl`
- `AS-20260501-021`
- `AS-20260501-022`
- `AS-20260501-028`

## Verification-Report Residual Addendum

### Reconciled Residual Scope

| Residual class | Live decision |
| --- | --- |
| report-1-only missing-model gates (`l1-*`, `l2-cryptol`, `l2-proverif`, `l2-refinement-map`, `l2-transcript`, `l4-constant-time`, `l4-unsafe`) | exclude from this additive packet because later reports either supersede them, narrow them to tooling bootstrap outside the current packet, or do not survive as repeated active blockers in reports `2-4` |
| `l2-crux-mir` and `l2-saw` | exclude from live residual scope because later reports upgrade them to `BOUNDED_VERIFIED` and `FORMALLY_PROVED` |
| repeated path and dispatch failures (`l0-docs`, `l3-verify-fast`, `l4-supply-chain`) | keep live under `VR-10` |
| repeated managed-toolchain and offline proof failures (`l2-hax`, `l2-tamarin`, `l3-kani`, `l3-miri`, `l3-verus`, `l4-fuzz`) | keep live under `VR-11` |
| repeated `l4-adversarial-review` umbrella finding | keep live, but decompose it across `VR-12` and `VR-13`; project-owned hypotheses must close by code, tests, or deterministic simulation rather than a manual-only note |
| repeated project-owned checkpoint or storage adversarial hypotheses | keep live under `VR-12` as the checkpoint/storage branch of `l4-adversarial-review` |
| repeated project-owned payment-request, stealth, and wallet-RPC adversarial hypotheses | keep live under `VR-13` as the request/stealth/wallet branch of `l4-adversarial-review` |
| protected-vendor concentration findings | keep report-visible only; close through project-owned wrapper or boundary proof, never vendor edits |

## Verification-Report Residual Coverage Table

| Additive unit | Plan file | Plan id | Residual findings | Primary owner surfaces | Dependency |
| --- | --- | --- | --- | --- | --- |
| `VR-10` | `065-10-PLAN.md` | `PLAN-065-G10` | `l0-docs`, `l3-verify-fast`, `l4-supply-chain` self-directory path failures | verification orchestrator dispatch and canonical gate entry paths | `PLAN-065-G09` |
| `VR-11` | `065-11-PLAN.md` | `PLAN-065-G11` | `l2-hax`, `l2-tamarin`, `l3-kani`, `l3-miri`, `l3-verus`, `l4-fuzz` | managed local verifier toolchain, offline caches, release-mode gate scripts | `PLAN-065-G10` |
| `VR-12` | `065-12-PLAN.md` | `PLAN-065-G12` | `l4-adversarial-review` checkpoint/storage branch: checkpoint lineage and delta integrity; storage concentration; validator-like scheduler/timing nondeterminism | project-owned runtime aggregators plus storage checkpoint or settlement paths | `PLAN-065-G11` |
| `VR-13` | `065-13-PLAN.md` | `PLAN-065-G13` | `l4-adversarial-review` request/stealth/wallet branch: PaymentRequest replay and compact-request rebinding; stealth delivery and inbox confusion; wallet-RPC concentration; protected-vendor crypto concentration by wrapper proof only | project-owned crypto domains, claim binding, wallet RPC receive paths, wrapper-level no-edit vendor boundaries | `PLAN-065-G12` |

## Seal-Only Items That Must Keep Regression Coverage

| Seal-only item | Active row or narrowed source | Owner plan | Required preserved proof |
| --- | --- | --- | --- |
| claim-source continuity | `CF-001` | `PLAN-065-G02` | keep negative tests for missing or drifted membership and keep `ClaimSourceRoot::new_settlement(...)` documented as typed wrapper only |
| object quarantine roundtrip and promotion semantics | `AS-20260630-064-08` | `PLAN-065-G02` | keep existing quarantine and promotion regressions while raw lanes are quarantined |
| object reject-code contract across storage, wallet, and rollup | `AS-20260630-064-09` | `PLAN-065-G07` | keep current cross-boundary reject-code regressions while fail-closed construction policy is hardened |
| recovery takeover and resume ownership path | `AS-20260630-064-12` | `PLAN-065-G06` | keep adversarial recovery ownership regressions while restore journaling is tightened |
| default simulator packet secret lane | `AS-20260630-064-07` | `PLAN-065-G04` | keep anti-regression tests proving the public lane never emits plaintext wallet-secret artifacts |
| current persisted `rotate_master_key` flow | narrowed old `CF-003` wording | `PLAN-065-G06` | keep wording and receipt-truth tests honest without reopening the retired placeholder claim |

### Exact Seal-Only Bullet Mirror

- Claim-source continuity. `claim_source_contract_for_item()` now checks
  persisted membership before emitting a claim contract. Keep negative tests
  for missing or drifted items. Keep
  `ClaimSourceRoot::new_settlement(...)` documented as a typed root wrapper,
  not proof of stored authority.
- Object quarantine roundtrip and promotion semantics. Current tests are
  strong; keep them.
- Object reject-code contract across storage, wallet, and rollup. Current
  tests are strong; keep them.
- Recovery takeover and resume ownership path. Current code is strongly
  fail-closed; preserve adversarial tests.
- Default simulator packet secret lane. The default public lane is currently
  fail-closed; keep anti-regression tests while the real open work stays in
  `WS-03` and `WS-04`.
- Current persisted `rotate_master_key` flow. Keep wording and receipt-truth
  tests honest, but do not treat the retired rotate-master-key placeholder
  wording as a live bug.

## Historical Or Narrowed Findings

| Historical claim | Current disposition | Live remainder owner | Doc-truth owner |
| --- | --- | --- | --- |
| legacy `CF-006` "no nullifier semantics" wording | retired verbatim; only validator theorem-input closure remains live | `PLAN-065-G01` | `PLAN-065-G09` |
| legacy `CF-003` rotate-master-key placeholder wording | retired verbatim; only mutation or restore ownership and wording truth remain live | `PLAN-065-G06` | `PLAN-065-G09` |
| legacy `CF-009` invalid-owner-signature downgrade wording | closed by current fail-closed receive path; do not reopen without fresh code evidence | none | `PLAN-065-G09` |
| legacy `CF-011` `V2 memo unsupported` wording | no current trusted-path attack-surface task; reopen only via fresh feature spec and code evidence | none | `PLAN-065-G09` |
| legacy `CF-005` guarded `claim_v1` lane wording | do not treat as active unless a default or public build depends on it again | none | `PLAN-065-G09` |

### Exact Historical Bullet Mirror

- Legacy `CF-006` wording about missing nullifier semantics is
  obsolete. Current spend verification explicitly documents a delivered
  deterministic nullifier surface. The remaining live gap is validator
  theorem-input closure in `WS-01`.
- Legacy `CF-003` wording about "`wallet.key.rotate_master_key` is only
  placeholder semantics" is obsolete as a standalone bug statement. The
  remaining live concern is narrower: canonical mutation/restore ownership and
  long-term contract truth, which stay carried in `WS-06`.
- Legacy `CF-009` invalid-owner-signature downgrade wording is closed by the
  current fail-closed receive path, which rejects invalid-signature assets
  instead of silently scrubbing them into claimed storage.
- Legacy `CF-011` `V2 memo unsupported` wording was not reproduced as a
  current trusted-path attack-surface ticket in this pass. Reopen only with
  fresh code evidence and a concrete feature contract.

## Permanent Repository-Wide Meta-Gates From The Legacy Catalog

| Meta-gate class | Legacy ids absorbed | Owner plan | Enforcement mode |
| --- | --- | --- | --- |
| secret-bearing type hygiene | `AS-20260501-001/003/005/006/008/013/014/015/016/019` | `PLAN-065-G07` | source-audit script or CI gate |
| constant-time and equality hygiene | `AS-20260501-010/012/018` | `PLAN-065-G07` | source-audit script or CI gate |
| RNG hygiene | `AS-20260501-009/017` | `PLAN-065-G07` | source-audit script or CI gate |
| panic hygiene | `AS-20260501-007/011/020` | `PLAN-065-G07` | source-audit script or CI gate |
| operator-visible logging hygiene | `AS-20260501-002/004/023/024/025/026` | `PLAN-065-G07` | source-audit script or CI gate |

### Exact Meta-Gate Policy Bullet Mirror

- Secret-bearing type hygiene. Ban unreviewed `Debug`, `Serialize`, or
  `Deserialize` exposure on secret wrappers and key material.
- Constant-time and equality hygiene. Ban direct `==` or ordinary `PartialEq`
  checks on secret material unless the code routes through an explicit
  constant-time helper.
- RNG hygiene. Ban non-cryptographic RNG near key generation, nonce
  generation, salts, proof witnesses, or similar cryptographic inputs.
- Panic hygiene. Ban `unwrap`, `expect`, and `panic!` in auth, storage
  open/load, restore, seed, key, and proof-verification boundaries.
- Operator-visible logging hygiene. Ban plaintext seeds, raw secret material,
  internal storage dumps, filesystem paths, and similar boundary leaks unless
  explicitly allowlisted.

### Exact Meta-Gate Implementation Mirror

- Add source-audit tests or lint-style CI checks for each meta-gate.
- Keep allowlists small, path-scoped, and reviewed.
- Fail CI on new violations.

### Exact Legacy Id Mirror

- Secret-bearing type hygiene:
  - `AS-20260501-001`
  - `AS-20260501-003`
  - `AS-20260501-005`
  - `AS-20260501-006`
  - `AS-20260501-008`
  - `AS-20260501-013`
  - `AS-20260501-014`
  - `AS-20260501-015`
  - `AS-20260501-016`
  - `AS-20260501-019`
- Operator-visible logging and error-boundary hygiene:
  - `AS-20260501-002`
  - `AS-20260501-004`
  - `AS-20260501-023`
  - `AS-20260501-024`
  - `AS-20260501-025`
  - `AS-20260501-026`
- Panic-at-boundary hygiene:
  - `AS-20260501-007`
  - `AS-20260501-011`
  - `AS-20260501-020`
- RNG-near-crypto hygiene:
  - `AS-20260501-009`
  - `AS-20260501-017`
- Equality-on-secret hygiene:
  - `AS-20260501-010`
  - `AS-20260501-012`
  - `AS-20260501-018`

## Active Disposition Map

| Owner plan | Active disposition rows carried |
| --- | --- |
| `PLAN-065-G01` | `AS-20260627-001`, `AS-20260630-064-11` |
| `PLAN-065-G02` | `AS-20260630-064-10`, `AS-20260501-029`, `AS-20260501-030`, `CF-001` claim-source continuity, `CF-002` raw/compatibility proof semantics, raw-lane part of `CF-010` proofless scaffold/helper surfaces, `AS-20260630-064-08` |
| `PLAN-065-G03` | crate `AS-20260623-001`, crate `AS-20260623-003`, `AS-20260501-027`, `AS-20260501-031` |
| `PLAN-065-G04` | `AS-20260630-064-01`, `AS-20260630-064-07` |
| `PLAN-065-G05` | `AS-20260630-064-03`, `AS-20260630-064-05`, `AS-20260630-064-06` |
| `PLAN-065-G06` | `AS-20260630-064-02`, `AS-20260630-064-04`, `AS-20260630-064-12`, narrowed rotate-master-key wording remainder |
| `PLAN-065-G07` | `AS-20260627-002`, crate `AS-20260623-002`, `AS-20260630-064-09`, all five policy-gate classes |
| `PLAN-065-G08` | `CF-004` public chain scan/tip placeholder, `CF-007` stub helper coexistence, `CF-008` placeholder proof fields on DTOs, placeholder-DTO part of `CF-010` proofless scaffold/helper surfaces |
| `PLAN-065-G09` | `AS-20260623-001` from `060-attack-surface-db.jsonl` plus all doc-truth demotion of narrowed or retired claims |
| `PLAN-065-G10` | repeated `l0-docs`, `l3-verify-fast`, and `l4-supply-chain` self-directory path failures from verification reports `1-4` |
| `PLAN-065-G11` | repeated `l2-hax`, `l2-tamarin`, `l3-kani`, `l3-miri`, `l3-verus`, and `l4-fuzz` managed-toolchain or offline failures from verification reports `2-4` |
| `PLAN-065-G12` | repeated `l4-adversarial-review` checkpoint/storage branch: checkpoint-lineage, delta-integrity, storage concentration, and validator-like nondeterminism hypotheses from verification reports `2-4` |
| `PLAN-065-G13` | repeated `l4-adversarial-review` request/stealth/wallet branch: payment-request replay, compact-request rebinding, stealth-delivery confusion, wallet-RPC concentration, and protected-vendor concentration findings from verification reports `2-4` |

## Mandatory Closure Gate Before Marking Phase 065 Closed

| Required gate before phase close | Owner plans proving it |
| --- | --- |
| targeted tests in `z00z_storage`, `z00z_wallets`, `z00z_rollup_node`, and `z00z_simulator` for every workstream | `PLAN-065-G01` through `PLAN-065-G09` |
| build-policy checks proving forbidden release-feature combinations fail | `PLAN-065-G03` |
| source-audit checks enforcing the meta-gates | `PLAN-065-G07` |
| negative tests for theorem-input absence, raw checkpoint misuse, unguarded privileged RPCs, draft-only publication evidence, and transport-log leakage | `PLAN-065-G01`, `PLAN-065-G02`, `PLAN-065-G05`, `PLAN-065-G04`, `PLAN-065-G07` |
| final docs and public-comment sweep so narrowed historical claims stay retired | `PLAN-065-G09` |
| verification-report residual packet reruns green with no stale path drift, no toolchain downgrade shortcuts, and no unresolved project-owned adversarial hypothesis left without code or test closure | `PLAN-065-G10`, `PLAN-065-G11`, `PLAN-065-G12`, `PLAN-065-G13` |

### Exact Mandatory Closure Bullet Mirror

- targeted tests in `z00z_storage`, `z00z_wallets`, `z00z_rollup_node`, and
  `z00z_simulator` for every workstream listed above;
- build-policy checks proving forbidden release-feature combinations fail;
- source-audit checks enforcing the meta-gates;
- negative tests for theorem-input absence, raw checkpoint misuse, unguarded
  privileged RPCs, draft-only publication evidence, and transport-log leakage;
- one final pass over docs and public API comments so the repo stops
  describing narrowed historical claims as if they were still live.

## 🧪 Mandatory Verify Contract

Every `PLAN-065-GNN` `<verify>` block must preserve this exact contract:

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
   first. If it fails, stop, fix, rerun, and only then continue.
2. Run the slice-specific commands named by the plan.
3. Run `cargo test --release` whenever Rust or test-affecting changes are
   relevant.
4. Run `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least 3 times and keep
   iterating until at least 2 consecutive runs show no significant code issues.
5. If a commit is needed, use `/z00z-git-versioning`.
6. Use nested `.github/` verification skills and prompts as needed, especially
   `doublecheck`, spec-to-code compliance, smart tests, and the relevant Z00Z
   gates.

## 🧾 Current Code Evidence Anchors

| Workstream | Current-tree evidence |
| --- | --- |
| `WS-01` | `ResolvedBatch` now carries `SettlementTheoremBundle`, DA resolution constructs that bundle before validator admission, and `CheckpointFlow::try_from_resolved()` plus validator regression tests reject theorem, route, `pub_in`, link, and checkpoint drift on the accepted path. |
| `WS-02` | Canonical checkpoint birth is now `seal_artifact()`-only, raw final export is isolated behind `export_noncanonical_final_bundle()`, canonical loads reject the noncanonical lane, and checkpoint-link writes fail closed when snapshot or exec-input evidence rows drift. |
| `WS-03` | Release-capable build policy is enforced by `test_production_hardening`, `test_live_boundary_claims`, and `scripts/audit/audit_release_feature_guards.sh`; forbidden `wallet_debug_tools` or `test-params-fast` combinations no longer normalize as acceptable release commands. |
| `WS-04` | `Stage6ProofMode::DraftOnly` remains only as an explicitly noncanonical simulator mode; stage-finalization and public packet tests prove draft/debug output cannot masquerade as finalized public checkpoint or publication evidence. |
| `WS-05` | Privileged wallet paths now consume typed `VerifiedSession` or `VerifiedSessionNoTouch` capabilities, route-audit tests enumerate the guarded RPC set, and the raw stealth-output builder is kept visibly noncanonical relative to the validated public path. |
| `WS-06` | Asset mutation RPCs route through shared submission and tx-journal helpers, restore and retry semantics are exercised through journal-backed rollback tests, and rotate-master-key wording plus receipt checks now track the persisted lifecycle contract instead of the retired placeholder claim. |
| `WS-07` | `SettlementStore::new()` is now a managed local non-panicking constructor, `try_new()` or `load()` own the fallible operator boundary, wasm transport logging emits redacted shape summaries only, and panic or redaction meta-gates are enforced by executable audit scripts. |
| `WS-08` | Public chain RPCs are narrowed to explicitly wallet-local `start_local_scan` / `stop_local_scan` / `get_local_scan_status` / `get_local_scan_tip` surfaces, route and doc tests keep the old production-looking names retired, and receipt serialization omits placeholder proof fields from production DTO defaults. |
| `WS-09` | Narrowed historical wording is fenced to explicit planning references only, stale compatibility-bootstrap claims are rejected by `scripts/audit_phase065_narrowed_wording.sh`, and human-readable docs no longer re-promote the retired Phase 065 leftovers as live current-tree truth. |

</normative_mirror>

<execution_contract>
## ✅ Planning Contract

- Each `065-0N-PLAN.md` must stay executable rather than advisory.
- Each plan must include the exact `WS-*` unit it covers and must not merge
  multiple workstreams away.
- Each plan must include every source finding id already named by the owning
  `WS-*` section.
- Each plan must name exact code/docs/config/test artifacts and exact commands.
- If a workstream closes by contract demotion instead of full live
  implementation, the plan must mark `implementation_depth` as
  `live-claim-removed` and include explicit negative proof that the old
  production-looking surface is gone.
- `WS-09` has no standalone `Required tests:` section in the TODO, so its plan
  must derive proof from the phase closure gate and the narrowed-current-tree
  wording sweep without inventing a new semantic requirement.

</execution_contract>
