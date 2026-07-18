---
phase: 064-Gaps-Closing-3
artifact: tests-tasks
status: complete
source:
  - live code and tests
  - .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md
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

# Phase 064 Test Tasks

## 🎯 Purpose

This file is the executable verification checklist for the live Phase 064 test
packet. It preserves the one-plan-per-scenario boundary, names the exact
commands and scans to run, and freezes the proof expectations that must be met
before a slice can be called closed.

Use it to verify the implementation directly from code, tests, file trees, and
release-mode commands. Do not treat summaries as proof.

This is not a second design. The normative sources remain:

- `.planning/phases/064-Gaps-Closing-3/064-TODO.md` for the recommendation inventory;
- `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md` for the anti-drift mirror and row-class lock;
- `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md`,
  `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md`,
  `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md`,
  `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md`, and
  `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` for exact execution
  seams and artifacts; and
- `.planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md` for scenario
  boundaries, docs-corpus ownership, invariants, examples, and pass signals.

The `17`-document Markdown corpus attached in `.planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md`
is live requirement input wherever the grouped plans cite it. Do not treat
cited docs as optional commentary.

## 📌 TODO Directive Carry-Forward

### Top-Level TODO Steps 1-5

| TODO directive | Preserved here as | Owning execution step | Carry-forward rule |
| --- | --- | --- | --- |
| Top-level TODO step 1: simulator | `064-01` simulator final truth and packet integrity | `064-01` / `064-S01` / `PLAN-064-G01` | The execution checklist must close simulator final truth, hermetic stages, exact-home artifacts, secret-clean packet output, and facade-only imports before moving on. |
| Top-level TODO step 2: wallet | `064-02` wallet-local mutation truth | `064-02` / `064-S02` / `PLAN-064-G02` | Asset mutations must land on live local chain, broadcast, and durable tx owners instead of stub or fake lanes. |
| Top-level TODO step 3: rpc | `064-02` RPC audit truth and route coverage | `064-02` / `064-S02` / `PLAN-064-G02` | Include-based route coverage and `app.wallet.open_wallet_source` wiring remain part of the same second-wave execution step. |
| Top-level TODO step 4: wallet services | `064-03` sensitive wallet surface and placeholder-owner closure | `064-03` / `064-S03` / `PLAN-064-G03` | Only live-backed wallet-service seams may be promoted; placeholder-only seams stay honest and bounded. |
| Top-level TODO step 5: runtime/rollup | `064-04` storage, theorem, recovery, DA, and publication-binding closure | `064-04` / `064-S04` / `PLAN-064-G04` | Runtime and rollup proof surfaces must stay local, deterministic, fail-closed, and singular. |

### Ordered Closeout Groups

| TODO directive | Preserved here as | Owning execution step | Carry-forward rule |
| --- | --- | --- | --- |
| Numbered closeout group `1-5` | `REC-064-P0-04` through `REC-064-P0-08` | `064-03`, plus `064-01` for `REC-064-P0-08` | This group closes before numbered group `6-13`; default packet secrecy stays on the simulator owner surface. |
| Numbered closeout group `6-13` | `REC-064-P1-04` through `REC-064-P1-11` | `064-03` and `064-04` | This group closes only after simulator, wallet-local mutation, and RPC truth are restored. |
| Numbered closeout group `14-18` | `REC-064-P2-05` through `REC-064-P2-09` | `064-05`, plus `064-01` for `REC-064-P2-07` | Boundary-CI guardrails close last, except the simulator-facade guard, which stays with the first-wave simulator owner. |

- The TODO instruction "Only new points below; do not repeat
  simulator/wallet-asset/RPC/genesis/local-DA themes" remains preserved by
  keeping the top-level five-step order distinct from the numbered closeout
  groups above.
- The TODO instruction "Do not go to network/onion/remote-chain now" remains
  preserved by `064-05`, the defer-boundary reject conditions, and the ban on
  real OnionNet, real remote chain, or real DA transport as proof targets.
- Graphify may orient codebase structure only. It is never acceptable as
  coverage evidence, docs-corpus truth, or acceptance proof for any step here.

## 📌 Ordered Task List

| Step | Scenario / plan | Primary homes | What to implement or extend | Must prove before moving on |
| --- | --- | --- | --- | --- |
| `064-00` | packet integrity | `.planning/phases/064-Gaps-Closing-3/064-TODO.md`, `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md`, `.planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md`, `.planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md`, `.planning/phases/064-Gaps-Closing-3/064-01-PLAN.md`, `.planning/phases/064-Gaps-Closing-3/064-02-PLAN.md`, `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md`, `.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md`, `.planning/phases/064-Gaps-Closing-3/064-05-PLAN.md` | Freeze the one-to-one mapping from `064-S01`, `064-S02`, `064-S03`, `064-S04`, and `064-S05` to `PLAN-064-G01`, `PLAN-064-G02`, `PLAN-064-G03`, `PLAN-064-G04`, and `PLAN-064-G05`, confirm all `28` `REC-064-*` rows still have one owner, confirm all `17` Markdown corpus paths remain attached to the spec, and confirm the TODO step order plus numbered closeout groups are still explicit in the test packet. | The executor can point to one scenario, one grouped plan, one truthful proof-home family, one docs-corpus attachment, and one preserved TODO directive owner for every Phase 064 requirement. |
| `064-01` | `064-S01` / `PLAN-064-G01` | simulator config, stage `9` and `12`, runtime observability, `scenario_1` tests, simulator docs | Implement or extend tests for default final publication, exact-home packet artifacts, hermetic canonical stages, packet redaction, and facade imports without creating a second simulator lane. | Default `scenario_1` publication is final and exact-home by default, filtered stages stay hermetic, and default packets remain secret-clean. |
| `064-02` | `064-S02` / `PLAN-064-G02` | asset RPC ops, local chain/broadcast, tx storage, route assembly, audit script, object-package docs/tests | Replace stub asset mutations with live wallet-local semantics, make route-audit truth include include-based registrations, and keep `wallet.object.*` visibly live. | Wallet-local mutation RPCs return real durable tx behavior, route coverage is truthful, and `wallet.object.*` never regresses to “stub.” |
| `064-03` | `064-S03` / `PLAN-064-G03` | restore/session/request/quarantine owners, reject-code mapping, wasm capability guard | Add or extend atomic-restore, sensitive-session, request-validation, capability-matrix, quarantine, and reject-code tests while keeping placeholder seams honest. | Sensitive wallet surfaces stay session-gated and atomic, raw builders remain out of production flows, and quarantine/reject-code semantics stay durable. |
| `064-04` | `064-S04` / `PLAN-064-G04` | checkpoint store, snapshot store, theorem verifier, local DA, recovery, publication binding | Add or extend seal-path, snapshot-negative, theorem-negative, recovery, and publication-binding tests with real local primitives only. | Storage, theorem, recovery, and publication-binding truth stay singular and fail closed on all named adversarial paths. |
| `064-05` | `064-S05` / `PLAN-064-G05` | live-boundary tests, defer-boundary docs, repo scripts, CI workflow | Add or extend truthful wording guards, defer-boundary tests, infra-boundary scripts, crypto-facade checks, extensions-boundary checks, and local-link hygiene checks. | Docs and repository boundaries stay truthful, explicit about deferral, and executable under CI without direct-vendor or direct-infra drift. |

## 🔧 Required Commands And Evidence By Step

| Step | Mandatory narrow commands or checks | Evidence that must be preserved |
| --- | --- | --- |
| `064-00` | `bash -lc 'test "$(rg --no-filename -o "REC-064-[A-Z0-9-]+" .planning/phases/064-Gaps-Closing-3/064-CONTEXT.md .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md | sort -u | wc -l)" -eq 28'`; `bash -lc 'test "$(rg --no-filename -o "PLAN-064-G0[1-5]" .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md | sort -u | wc -l)" -eq 5'`; `bash -lc 'test "$(rg --no-filename -o "064-S0[1-5]" .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md | sort -u | wc -l)" -eq 5'`; `bash -lc 'test "$(awk '"'"'/^## 📚 Docs Corpus Attachment/{flag=1;next}/^## /{flag=0}flag && /^\| / && index($0,".md"){count++}END{print count+0}'"'"' .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md)" -eq 17'`; `bash -lc 'test "$(awk '"'"'/^## 🔎 Coverage Appendix/{flag=1;next}/^## /{flag=0}flag && /^\| / && index($0,"REC-064-"){count++}END{print count+0}'"'"' .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md)" -eq 28'`; `bash -lc 'docs=(crates/z00z_simulator/README.md crates/z00z_networks/onionnet/README.md crates/z00z_runtime/aggregators/README.md crates/z00z_storage/src/settlement/README.md crates/z00z_utils/README.md crates/z00z_crypto/README.md crates/z00z_extensions/README.md docs/Z00Z-Main-Whitepaper.md docs/tech-papers/Z00Z-Roadmap-Blueprint.md wiki/03-core-protocol/genesis-caveats.md wiki/04-wallet-and-rpc/receiver-request-flow.md wiki/04-wallet-and-rpc/wallet-object-packages.md wiki/04-wallet-and-rpc/wallet-object-quarantine.md wiki/04-wallet-and-rpc/wallet-stub-surface.md wiki/05-storage-runtime/prep-snapshot-replay.md wiki/06-simulator-and-quality/scenario-pipeline.md wiki/06-simulator-and-quality/scenario1-object-artifacts.md); for p in "${docs[@]}"; do rg -n --fixed-strings "$p" .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md >/dev/null || { echo "missing doc ref: $p"; exit 1; }; done'`; `bash -lc 'for f in .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md; do for s in "Top-level TODO step 1" "Top-level TODO step 2" "Top-level TODO step 3" "Top-level TODO step 4" "Top-level TODO step 5" "Numbered closeout group \`1-5\`" "Numbered closeout group \`6-13\`" "Numbered closeout group \`14-18\`" "Graphify may orient codebase structure only"; do rg -n --fixed-strings "$s" "$f" >/dev/null || { echo "missing directive carry-forward in $f: $s"; exit 1; }; done; done'` | Count output proving `28` unique requirements, `5` grouped plans, `5` scenarios, `17` docs-corpus rows, `28` coverage-appendix rows, one direct spec mention for every required Markdown source, and explicit carry-forward of the top-level TODO order plus grouped closeout directives in both test-packet files. |
| `064-01` | `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface -- --nocapture`; `cargo test --release -p z00z_simulator --test scenario_1 test_scenario_settlement -- --nocapture`; `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows -- --nocapture`; `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture`; `cargo test --release -p z00z_simulator --test scenario_1 test_stage2_secret_artifacts -- --nocapture` | Passing simulator tests showing final checkpoint evidence, no canonical fallback closure, emitted exact-home object packet anchors, and no plaintext secret leakage in default packet surfaces. |
| `064-02` | `cargo test --release -p z00z_wallets --test test_asset_rpc_mutations -- --nocapture`; `cargo test --release -p z00z_wallets --test test_chain_client_sim -- --nocapture`; `cargo test --release -p z00z_wallets --test test_chain_broadcast_retry -- --nocapture`; `python3 crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`; `cargo test --release -p z00z_wallets --test test_rpc_route_coverage -- --nocapture`; `cargo test --release -p z00z_wallets --test test_object_rpc_packages -- --nocapture`; `rg -n "open_wallet_source|wallet.object\\.|stub_default|stub_tx_" crates/z00z_wallets/src crates/z00z_wallets/tests wiki/04-wallet-and-rpc/wallet-object-packages.md` | Passing mutation, chain, broadcast, route, and object-package tests; audit-script output showing truthful route counts; grep output proving live-path stubs are removed and `wallet.object.*` stays live. |
| `064-03` | `cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture`; `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session -- --nocapture`; `cargo test --release -p z00z_wallets --test test_payment_request -- --nocapture`; `cargo test --release -p z00z_wallets --test test_wallet_capability_matrix -- --nocapture`; `cargo test --release -p z00z_wallets --test test_object_quarantine -- --nocapture`; `cargo test --release -p z00z_storage --test test_object_reject_codes -- --nocapture`; `rg -n "browser builds do not get this live session model|native-only today|Rejects wasm32 and routes native load through spawn_blocking|\\.wlt persistence is not supported on wasm32|\\.wlt owned-asset loading is not supported on wasm32" wiki/04-wallet-and-rpc crates/z00z_wallets/src/services`; `rg -n "verify_session|verify_session_no_touch|build_tx_stealth_output|wasm32|placeholder|Quarantined|ObjectRejectCode" crates/z00z_wallets crates/z00z_storage wiki/04-wallet-and-rpc` | Passing restore, session, request, capability, quarantine, and reject-code tests; wasm/native source-doc guard output; grep output proving raw builders and placeholder seams are bounded and capability claims stay honest. |
| `064-04` | `cargo test --release -p z00z_storage --test test_checkpoint_store -- --nocapture`; `cargo test --release -p z00z_storage --test test_prep_snapshot -- --nocapture`; `cargo test --release -p z00z_storage --test test_settlement_proof_boundaries -- --nocapture`; `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture`; `cargo test --release -p z00z_rollup_node --test test_da_local_sim -- --nocapture`; `cargo test --release -p z00z_aggregators --test test_recovery_failover -- --nocapture`; `cargo test --release -p z00z_aggregators --test test_publication_binding -- --nocapture`; `rg -n "save_artifact|seal_artifact|DupPath|DupTerminalId|RootMix|backend_root|SettlementStateRoot|PublicationBinding|split-brain|wrong link root" crates/z00z_storage crates/z00z_rollup_node crates/z00z_runtime/aggregators` | Passing checkpoint, snapshot, theorem, DA, recovery, and publication-binding tests plus grep output proving one canonical seal path, one semantic-root story, and one binding digest story. |
| `064-05` | `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`; `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`; `bash scripts/audit_z00z_utils_boundary.sh`; `bash scripts/audit_crypto_facade.sh`; `bash scripts/audit_extensions_boundary.sh`; `bash scripts/audit_local_docs_links.sh`; `rg -n "github\\.com/z00z-labs/z00z/blob|tari_crypto::|std::fs|serde_json|serde_yaml|SystemTime::now|not shipped|placeholder|future transport" crates wiki docs scripts .github` | Passing boundary tests and scripts plus grep output proving truthful defer wording, infra-boundary discipline, crypto-facade discipline, extensions-boundary discipline, and offline-safe local doc links. |

## 🔁 Shared Validation Rules

| Rule | Requirement |
| --- | --- |
| Bootstrap fail-fast | Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first for every step. If it fails, stop, fix, rerun, and only then continue. |
| Release-only cargo | Use `--release` for every cargo validation command in this packet when cargo supports it. |
| Broad Rust gate | Run `cargo test --release` after Rust, tests, docs guards, simulator behavior, wallet behavior, or verification scripts change. |
| Review repetition | Run `./.github/prompts/gsd-review-tasks-execution.prompt.md` (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times and continue until at least `2` consecutive runs show no significant issues. |
| Commit discipline | If a step needs a commit after validation, use `/z00z-git-versioning`. |
| Evidence discipline | Preserve the exact output, emitted artifact, audit result, grep result, or simulator packet that proves closure. Do not replace evidence with prose. |
| Real-primitive discipline | Fake only the allowed external transport, remote process boundary, unavailable DA transport, or scheduler boundary. Use real checkpoints, route tables, publication bindings, payment-request validation, object-package validation, and wallet persistence. |

## 🚫 Global Reject Conditions

| Condition | Why it fails the phase |
| --- | --- |
| A second simulator, wallet mutation, checkpoint, theorem, publication-binding, or docs authority lane appears | It duplicates existing logic instead of proving the current owner surface. |
| A default packet, report, or artifact leaks plaintext secret material | Phase 064 explicitly requires secret-clean default outputs. |
| A live wallet mutation or object path still depends on stubs or fake tx ids | The user-facing local wallet path would remain non-truthful. |
| A browser or wasm surface advertises native-only guarantees | It would make capability boundaries ambiguous and unsafe. |
| A raw-save, backend-root, or second binding digest becomes semantic truth | It violates the single-root and single-binding contract. |
| Docs claim live OnionNet, live remote chain, or live DA behavior | Phase 064 must keep those surfaces explicitly deferred until real owners exist. |
| Business crates bypass `z00z_utils` or import vendor crypto directly | It violates the repository boundary and facade rules. |
| Internal GitHub blob links remain where local repository paths exist | It breaks the local-first, offline-safe documentation contract. |

## 🧭 Implementation Notes For The Next Engineer

| Area | Reuse | Keep explicit |
| --- | --- | --- |
| Simulator closure | Existing `scenario_1` finalize flow, packet exporters, runtime observability, and object-flow tests | Default finalization, hermetic stages, exact-home packet rows, secret-clean defaults, and facade-only imports |
| Wallet mutation truth | Existing `LocalNodeSim`, `ChainClientImpl`, `BroadcastImpl`, `TxStorage`, dispatcher routes, and object-package path | Real mutation semantics, truthful route counting, and no stale “stub” wording |
| Wallet sensitive surfaces | Existing restore, session, request, quarantine, and object-policy owners | Atomic restore, session gates, raw-builder bans, honest wasm/native boundaries, and reject-code exhaustiveness |
| Storage/runtime closure | Existing checkpoint, snapshot, theorem, local DA, recovery, and publication-binding owners | One semantic-root story, fail-closed theorem negatives, deterministic recovery negatives, and one anti-fork digest |
| Repository guardrails | Existing live-boundary tests, README contracts, and CI workflow | Truthful defer wording, infra-boundary discipline, crypto-facade discipline, extensions anti-drift, and local-only docs links |

<verify>

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` as
   the mandatory fail-fast gate.
2. Re-run the packet-integrity commands from step `064-00` so the scenario
   packet still covers all `28` requirements, all `5` plans, all `5`
   scenarios, and all `17` docs-corpus paths.
3. Run `cargo test --release` when the touched slice changes Rust, tests,
   simulator behavior, public APIs, wallet behavior, or verification scripts.
4. Run `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times. Fix all
   issues and warnings and continue until at least `2` consecutive runs show
   no significant issues.
5. If a commit is required after verification, use `/z00z-git-versioning`.

## ✅ Exit Conditions

1. Every grouped plan `PLAN-064-G01` through `PLAN-064-G05` has one owning
   scenario in `.planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md`.
2. Every scenario has exact commands, realistic positive examples, required
   negative cases, and measurable pass conditions.
3. The next engineer or agent can implement the coverage without inventing a
   second authority plane, a parallel simulator truth path, or undocumented
   scenario boundaries.
4. All `28` `REC-064-*` rows and all `17` docs-corpus paths are still visible
   in the planning packet after local edits.
5. Any seam that still cannot be honestly proven is recorded back into the
   Phase 064 planning packet as a blocker instead of being widened into vague
   non-executable coverage language.
