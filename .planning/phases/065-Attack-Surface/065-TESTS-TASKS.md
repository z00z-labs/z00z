---
phase: 065-Attack-Surface
artifact: tests-tasks
status: executed-and-verified
source: 065-TEST-SPEC.md
updated: 2026-07-02
---

# Phase 065 Tests Tasks

## 🎯 Purpose

📌 This document translates `065-TEST-SPEC.md` into one canonical execution
order for Phase 065 test work.

📌 The execution order below is no longer hypothetical. It is the summary-backed
sequence that closed `065-01` through `065-13` on the current tree, with
`065-TODO.md` still acting as the normative requirement surface.

📌 This file is not a second planning authority. It records how the phase was
executed and how it must be rerun if any Phase 065 seam is reopened.

## 📥 Scope Inputs

- `.planning/phases/065-Attack-Surface/065-TEST-SPEC.md`
- `.planning/phases/065-Attack-Surface/065-TODO.md`
- `.planning/phases/065-Attack-Surface/065-CONTEXT.md`
- `.planning/phases/065-Attack-Surface/065-01-PLAN.md` through
  `065-13-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-01-SUMMARY.md` through
  `065-13-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/z00z-verification-report-1.md` through
  `z00z-verification-report-4.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## 🧭 Execution Strategy

📌 The canonical order is:

1. freeze one authority chain and one test-anchor map;
2. close `WS-01` through `WS-09` in TODO priority order;
3. close additive residual packet `VR-10` through `VR-13`;
4. end on a broad `cargo test --release` gate with no parallel test layer.

📌 Dependency rules:

- `T1` must land before `T2`, because accepted-path theorem truth defines what
  canonical checkpoint persistence must carry.
- `T2` must land before `T4`, because public simulator evidence may not infer
  truth from raw lanes.
- `T3` must land before `T4` through `T13`, because release guards keep later
  behavior tests honest.
- `T5` must land before `T6`, because mutation and restore flows are privileged
  wallet paths.
- `T7` cross-cuts the whole phase, because redaction and panic hygiene are
  repository-wide trust-boundary guards.
- `T10` through `T13` are additive residual closure waves and must not be used
  to reinterpret or renumber the canonical `WS-*` packet.

📌 Reopen rule:

- if a change touches one wave, rerun that wave and every downstream dependent
  wave;
- always rerun the broad `cargo test --release` gate after targeted proofs;
- never add a second "temporary" audit path or alternate test harness to avoid
  the canonical wave sequence.

📌 Gate map:

- `T1` closes `G-01` and `G-02`
- `T2` closes `G-03` and `G-04`
- `T5` closes `G-05`
- `T6` closes `G-06` and `G-07`
- `T8` closes `G-08` and `G-09`

## 📚 Wave Evidence Map

| Wave | Authority | Backing Files | Status |
| --- | --- | --- | --- |
| `T0` | doc refresh and authority freeze | `065-TEST-SPEC.md`, `065-TODO.md`, `065-CONTEXT.md`, `STATE.md`, `ROADMAP.md` | current refresh only |
| `T1` | `WS-01` | `065-01-PLAN.md`, `065-01-SUMMARY.md` | complete |
| `T2` | `WS-02` | `065-02-PLAN.md`, `065-02-SUMMARY.md` | complete |
| `T3` | `WS-03` | `065-03-PLAN.md`, `065-03-SUMMARY.md` | complete |
| `T4` | `WS-04` | `065-04-PLAN.md`, `065-04-SUMMARY.md` | complete |
| `T5` | `WS-05` | `065-05-PLAN.md`, `065-05-SUMMARY.md` | complete |
| `T6` | `WS-06` | `065-06-PLAN.md`, `065-06-SUMMARY.md` | complete |
| `T7` | `WS-07` | `065-07-PLAN.md`, `065-07-SUMMARY.md` | complete |
| `T8` | `WS-08` | `065-08-PLAN.md`, `065-08-SUMMARY.md` | complete |
| `T9` | `WS-09` | `065-09-PLAN.md`, `065-09-SUMMARY.md` | complete |
| `T10` | `VR-10` | `065-10-PLAN.md`, `065-10-SUMMARY.md` | complete |
| `T11` | `VR-11` | `065-11-PLAN.md`, `065-11-SUMMARY.md` | complete |
| `T12` | `VR-12` | `065-12-PLAN.md`, `065-12-SUMMARY.md` | complete |
| `T13` | `VR-13` | `065-13-PLAN.md`, `065-13-SUMMARY.md` | complete |

## 🔐 Global Verification Rules

- Every `<verify>` block starts with
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
- Every `<verify>` block uses release-safe commands only; Phase 065 does not
  close on debug-only proof.
- Every `<verify>` block includes a broad `cargo test --release` rerun after
  targeted commands.
- Every `<verify>` block requires `/GSD-Review-Tasks-Execution` at least three
  times in YOLO mode until two consecutive runs are clean. If the prompt path
  is unavailable or token-limited, record the exact blocker and run three
  manual equivalent review passes over the same scope, as captured in the
  executed summaries.
- If a commit is needed during a rerun, use `/z00z-git-versioning`.

## ⚙️ Task Waves

### ⚙️ Wave T0: Authority Freeze And Harness Confirmation

- Source: `065-TEST-SPEC.md`, `065-TODO.md`, `065-CONTEXT.md`, `STATE.md`,
  `ROADMAP.md`
- Objective: keep one canonical Phase 065 test authority chain and remove stale
  planned-only wording from the test artifacts.
- Completion gate: all referenced plan, summary, script, and test anchors
  exist; Phase 065 status is still complete; the live release-guard audit path
  is `scripts/audit/audit_release_feature_guards.sh`.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `rg --files .planning/phases/065-Attack-Surface | rg '065-(TEST-SPEC|TESTS-TASKS|TODO|CONTEXT|[0-9]{2}-PLAN|[0-9]{2}-SUMMARY)|z00z-verification-report-[1-4]'`
   - `rg -n "status: Phase 065 Complete|completed_plans: 13|percent: 100" .planning/STATE.md`
   - `rg -n "Phase 065: Attack Surface|065-01-SUMMARY.md|065-13-SUMMARY.md" .planning/ROADMAP.md`
   - `rg --files | rg 'audit_release_feature_guards|audit_phase065_narrowed_wording|test_wasm_client_redaction|test_hash_policy'`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against the current `065-TEST-SPEC.md`
   and `065-TESTS-TASKS.md` refresh scope at least three times in YOLO mode
   until two consecutive runs are clean. If the prompt path is unavailable or
   token-limited, record the exact blocker and execute three manual equivalent
   review passes over the same scope.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T1: Theorem-Verified Validator Acceptance (`065-01`)

- Source: `065-01-PLAN.md`, `065-01-SUMMARY.md`, `WS-01`
- Primary files: rollup theorem verifier and DA resolution, validator
  checkpoint or verdict flow, theorem support fixtures and tests
- Landed result: accepted paths now require the full theorem bundle and bind
  theorem, publication, and checkpoint coherence together.
- Completion gate: no accepted path survives without theorem inputs; wrong
  link, exec-input, snapshot, route, or publication binding all reject.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
   - `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture`
   - `cargo test --release -p z00z_validators --test test_object_policy_verdicts -- --nocapture`
   - `cargo test --release -p z00z_rollup_node --test test_da_local_sim -- --nocapture`
   - `cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-01-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T2: Canonical Checkpoint Persistence (`065-02`)

- Source: `065-02-PLAN.md`, `065-02-SUMMARY.md`, `WS-02`
- Primary files: checkpoint store, store FS, artifact-proof draft handling,
  link codec, stage-12 finalization bridge
- Landed result: canonical checkpoint artifacts are seal-only and raw lanes are
  quarantined.
- Completion gate: `save_link()` and canonical consumers reject missing
  evidence rows, mismatched ids, and raw-lane impersonation.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `cargo test --release -p z00z_storage --test test_checkpoint_store -- --nocapture`
   - `cargo test --release -p z00z_storage --test test_checkpoint_finalization -- --nocapture`
   - `cargo test --release -p z00z_storage --test test_checkpoint_link_injective -- --nocapture`
   - `cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-02-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T3: Release Build Hardening For Debug Surfaces (`065-03`)

- Source: `065-03-PLAN.md`, `065-03-SUMMARY.md`, `WS-03`
- Primary files: wallet and simulator public feature surfaces, storage
  corruption or scheduler controls, release-safety audit script
- Landed result: release-capable builds fail closed on forbidden debug or
  fast-test feature combinations.
- Completion gate: public wallet and simulator release paths cannot compile
  with `test-params-fast` or `wallet_debug_tools`.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `bash scripts/audit/audit_release_feature_guards.sh`
   - `cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture`
   - `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-03-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T4: Draft And Debug Simulator Evidence Truth (`065-04`)

- Source: `065-04-PLAN.md`, `065-04-SUMMARY.md`, `WS-04`
- Primary files: simulator config, stage-12 publication path, runtime
  observability, public-lane wallet integration test
- Landed result: draft-only or synthetic evidence no longer looks like final
  public publication truth.
- Completion gate: `DraftOnly` is rejected on public lanes and the default
  public lane remains secret-free.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture`
   - `cargo test --release -p z00z_storage --test test_checkpoint_finalization -- --nocapture`
   - `cargo test --release -p z00z_simulator --test scenario_1 test_stage6_checkpoint_final_gate::test_draft_publication_rejected -- --exact --nocapture`
   - `cargo test --release -p z00z_simulator --test scenario_1 test_wallet_integration::test_public_lane_secret_free -- --exact --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-04-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T5: Capability-Typed Privileged Wallet Paths (`065-05`)

- Source: `065-05-PLAN.md`, `065-05-SUMMARY.md`, `WS-05`
- Primary files: session guard types, privileged RPC handlers, dispatcher
  wiring, route audit, stealth output API
- Landed result: privileged wallet flows now rely on typed guard paths and
  explicit target capability truth.
- Completion gate: route audit stays clean, raw stealth builder remains visibly
  noncanonical, and unsupported wasm capabilities fail explicitly.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `bash crates/z00z_wallets/scripts/audit_rpc_method_wiring.sh`
   - `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session --test test_wallet_capability_matrix --test test_stealth_output --test test_rpc_route_coverage -- --nocapture`
   - `cargo test --release -p z00z_wallets`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-05-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T6: Canonical Wallet Mutation And Restore Ownership (`065-06`)

- Source: `065-06-PLAN.md`, `065-06-SUMMARY.md`, `WS-06`
- Primary files: asset RPC mutation path, broadcast implementation, restore
  service, tx digest framing, chain-client and tx-store tests
- Landed result: one canonical owner now controls local mutation truth and
  restore retry semantics stay explicit.
- Completion gate: partial failures keep mutation, history, publish, and retry
  state coherent.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `cargo test --release -p z00z_wallets --test test_asset_rpc_mutations -- --nocapture`
   - `cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture`
   - `cargo test --release -p z00z_wallets --test test_chain_broadcast_retry -- --nocapture`
   - `cargo test --release -p z00z_wallets --test test_tx_store_integration -- --nocapture`
   - `cargo test --release -p z00z_wallets --test test_chain_client_sim -- --nocapture`
   - `cargo test --release -p z00z_core test_new_confidential_with_blinding_is_deterministic -- --nocapture`
   - `cargo test --release -p z00z_simulator 'scenario_1::stage_6::test_tx_lane_runtime_suite::test_tx_validation_chain_id' -- --exact --nocapture`
   - `cargo test --release -p z00z_simulator 'scenario_1::stage_6::test_tx_lane_runtime_suite::test_tx_validation_nullifier_drift' -- --exact --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-06-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T7: Fail-Closed Construction, Redaction, And Meta-Gates (`065-07`)

- Source: `065-07-PLAN.md`, `065-07-SUMMARY.md`, `WS-07`
- Primary files: storage open or load boundary, wasm RPC client, security type
  surfaces, CI hygiene audit scripts
- Landed result: trust-boundary constructors fail closed and transport logging
  emits only redacted summaries.
- Completion gate: all five hygiene audits stay green and boundary tests show no
  raw secret-bearing logging or panic-at-boundary behavior.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `bash scripts/audit/audit_secret_type_hygiene.sh`
   - `bash scripts/audit/audit_secret_eq_hygiene.sh`
   - `bash scripts/audit/audit_crypto_rng_hygiene.sh`
   - `bash scripts/audit/audit_boundary_panic_hygiene.sh`
   - `bash scripts/audit/audit_log_redaction_hygiene.sh`
   - `cargo test --release -p z00z_networks_rpc -- --nocapture`
   - `cargo test --release -p z00z_wallets --test test_rpc_logging_acceptance -- --nocapture`
   - `cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy -- --nocapture`
   - `cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture`
   - `cargo test --release -p z00z_storage --test test_hjmt_adaptive_policy_proofs -- --nocapture`
   - `cargo test --release -p z00z_rollup_node 'da::tests::test_local_adapter_roundtrip' -- --exact --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-07-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T8: Placeholder Public RPC And DTO Contract Cleanup (`065-08`)

- Source: `065-08-PLAN.md`, `065-08-SUMMARY.md`, `WS-08`
- Primary files: chain service, public RPC DTOs, app wiring, runtime validation
  result and stub-behavior tests
- Landed result: public-facing chain and transaction RPC surfaces now describe
  live behavior honestly instead of placeholder truth.
- Completion gate: production-facing DTOs and docs no longer normalize
  synthetic placeholder-only proof semantics.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `cargo test --release -p z00z_wallets --test test_rpc_truth --test test_rpc_types_serialization --test test_rpc_wiring_spec_a --test test_runtime_validation_result --test test_stub_behavior -- --nocapture`
   - `cargo test --release -p z00z_wallets test_receipt_info_serialization -- --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-08-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T9: Final Narrowed-Claim Source Sweep (`065-09`)

- Source: `065-09-PLAN.md`, `065-09-SUMMARY.md`, `WS-09`
- Primary files: narrowed-wording audit script, core live guardrails, codebase
  and planning docs
- Landed result: retired historical claims stay retired and no stale human
  artifact re-promotes them as live truth.
- Completion gate: audit script and guardrail test agree on the narrowed
  current-tree wording.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `bash scripts/audit_phase065_narrowed_wording.sh`
   - `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-09-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T10: Canonical Verification Gate Entry Paths (`065-10`)

- Source: `065-10-PLAN.md`, `065-10-SUMMARY.md`, `VR-10`
- Primary files: verification orchestrator and direct L0 or L3 or L4 gate
  scripts
- Landed result: orchestrator dispatch now uses the same canonical script paths
  that the report metadata names.
- Completion gate: wrapper-path failures disappear and remaining failures are
  genuine downstream docs or toolchain or offline issues.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `./.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh report project --dry-run`
   - `Z00Z_L0_STRICT=1 ./.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`
   - `RUSTUP_TOOLCHAIN=stable ./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh --dry-run`
   - `RUSTUP_TOOLCHAIN=stable ./.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`
   - `RUSTUP_TOOLCHAIN=stable ./.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh report project --level l0,l3,l4`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-10-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T11: Managed Toolchain And Offline Gate Recovery (`065-11`)

- Source: `065-11-PLAN.md`, `065-11-SUMMARY.md`, `VR-11`
- Primary files: install and env scripts, verifier scripts, fuzz script, live
  guardrail path
- Landed result: managed verification now reports honest local state instead of
  wrapper or bootstrap noise.
- Completion gate: toolchain self-test is green, missing local targets report
  truthful `UNKNOWN`, and the broad release gate stays green.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `./scripts/install-verification-tools.sh --check --profile all --strict`
   - `./scripts/install-verification-tools.sh --self-test --profile all --strict`
   - `./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh`
   - `./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-miri.sh`
   - `./.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh`
   - `./.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-hax.sh`
   - `./.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-tamarin.sh`
   - `./.github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-short.sh`
   - `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-11-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T12: Checkpoint Lineage And Storage Determinism Closure (`065-12`)

- Source: `065-12-PLAN.md`, `065-12-SUMMARY.md`, `VR-12`
- Primary files: aggregator feature edge, aggregator release tests, storage
  lineage or root-generation tests, release guard audit
- Landed result: aggregator release-fast tests no longer leak wallet fast-test
  features, while lineage and determinism suites stay green.
- Completion gate: `cargo tree` shows wallet default features only under the
  aggregator release-fast path and all focused aggregator or storage suites pass.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_consensus`
   - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_route_rollout`
   - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage`
   - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_live_guardrails`
   - `cargo test --release -p z00z_storage --test test_checkpoint_finalization`
   - `cargo test --release -p z00z_storage --test test_hjmt_root_generation`
   - `cargo test --release -p z00z_storage --test test_live_guardrails`
   - `bash scripts/audit/audit_release_feature_guards.sh`
   - `cargo tree -e features -p z00z_aggregators --features test-params-fast -i z00z_wallets`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-12-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

### ⚙️ Wave T13: Payment Request And Stealth Binding Closure (`065-13`)

- Source: `065-13-PLAN.md`, `065-13-SUMMARY.md`, `VR-13`
- Primary files: hash-policy tests, asset RPC claim-scope helpers, request or
  stealth public-path tests
- Landed result: request domains stay pinned to `Poseidon2`, claim receipts use
  persisted wallet chain state, and public stealth behavior remains canonical.
- Completion gate: targeted crypto and wallet suites are green, the broad
  release gate is green, and the vendor subtree stays untouched.

<verify>
1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
2. Run:
   - `cargo test --release -p z00z_crypto --test test_hash_policy`
   - `cargo test --release -p z00z_wallets test_claim_scope_chain -- --nocapture`
   - `cargo test --release -p z00z_crypto`
   - `cargo test --release -p z00z_wallets --test test_payment_request`
   - `cargo test --release -p z00z_wallets --test test_asset_replay_protection`
   - `cargo test --release -p z00z_wallets --test test_e2e_req_flow`
   - `cargo test --release -p z00z_wallets --test test_stealth_output`
   - `cargo test --release -p z00z_wallets --test test_view_key_contract`
   - `cargo test --release -p z00z_wallets --test test_adversarial`
   - `cargo test --release -p z00z_wallets --test test_rpc_route_coverage`
   - `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session`
   - `cargo test --release -p z00z_wallets --test test_import_error_taxonomy`
   - `cargo fmt --all --check`
3. Run `cargo test --release`.
4. Run `/GSD-Review-Tasks-Execution` against
   `.planning/phases/065-Attack-Surface/065-13-PLAN.md` at least three times
   in YOLO mode until two consecutive runs are clean. If the prompt path is
   unavailable or token-limited, record the exact blocker and execute three
   manual equivalent review passes over the same files.
5. If committing, use `/z00z-git-versioning`.
</verify>

## 🛡️ Regression Hold Set

- Keep `crates/z00z_storage/tests/test_claim_source_proof.rs` green to preserve
  claim-source continuity.
- Keep `crates/z00z_wallets/tests/test_object_quarantine.rs` green to preserve
  object quarantine and promotion semantics.
- Keep `crates/z00z_storage/tests/test_object_reject_codes.rs` green to
  preserve object reject-code alignment.
- Keep `crates/z00z_wallets/tests/test_claim_resume_core.rs` and
  `crates/z00z_simulator/tests/scenario_1/test_claim_resume.rs` green to
  preserve recovery takeover and resume ownership behavior.
- Keep
  `crates/z00z_simulator/tests/scenario_1/test_wallet_integration.rs` green to
  preserve the default public-lane secret-free boundary.
- Keep wallet public-truth suites honest for persisted
  `rotate_master_key` wording and receipt truth; do not revive the old
  placeholder-only claim as a separate bug without new code evidence.

## ✅ Completion Record

- `T1` through `T13` are summary-backed complete on the current tree.
- `065-TEST-SPEC.md` is now aligned with executed Phase 065 reality instead of
  the old pre-execution snapshot.
- No planned-only Phase 065 test work remains. Any new drift belongs in a new
  summary-backed continuation, not as an untracked addition to this file.

## 🔁 Reopen Rule

- Reopen the smallest affected wave.
- Rerun that wave, every dependent downstream wave, and the broad
  `cargo test --release` gate.
- Update the matching `065-0N-SUMMARY.md` or create a new continuation summary
  if the reopened scope does not fit an existing wave.
- Keep `065-TODO.md` normative and keep this file synchronized with the live
  on-disk command paths only.
