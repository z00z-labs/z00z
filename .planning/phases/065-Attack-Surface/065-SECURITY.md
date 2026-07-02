---
phase: 065
slug: 065-attack-surface
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-02
---

# Phase 065 - Security

> Per-phase security contract: closed threat register, trust boundaries, accepted risks, and audit trail for `.planning/phases/065-Attack-Surface/`.

This audit ran in State B of `gsd-secure-phase`: no prior `065-SECURITY.md`
existed, all `065-01..13` plan files and matching executed summaries were
present, and every plan carried a parseable `<threat_model>` block. The
register below preserves that plan-time threat set and records the current
live-code evidence that closes it.

## 🔑 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Validator theorem acceptance | Accepted-path settlement truth must cross from rollup publication into one validator-owned theorem verifier and one mandatory theorem bundle. | `TxPackage`, `CheckpointArtifact`, `CheckpointExecInput`, `CheckpointLink`, `checkpoint_id` |
| Checkpoint seal vs export | Final checkpoint authority must stay on the canonical seal lane while post-tx visibility stays explicitly noncanonical. | artifact bytes, link rows, final-lane markers, draft attestation bytes |
| Release build hardening | Release-capable binaries must reject debug or fast-test surfaces instead of merely documenting them. | Cargo features, debug export helpers, storage corruption hooks |
| Simulator final-public evidence | Draft-only simulator outputs must not cross into public publication packets or public checkpoint identifiers. | `evidence_class`, `checkpoint_id_hex`, publication binding, public traces |
| Privileged wallet RPC capability gate | Privileged wallet handlers must consume typed verified capabilities, not raw session assumptions. | `SessionToken`, `VerifiedSession`, `VerifiedSessionNoTouch`, route guard kind |
| Local mutation and restore durability | Wallet mutation identity and restore recovery must cross file, broadcast, and retry boundaries through one canonical digest and durable marker contract. | `tx_digest_hex`, package bytes, restore marks, publish stages |
| Local startup and transport hygiene | Local storage startup, RPC transport summaries, and secret handling must fail closed and remain redacted. | storage roots, request or response summaries, session tokens, secret comparisons |
| Wallet-local chain observation and receipt DTO truth | Public wallet chain observation must stay explicitly wallet-local and receipt DTOs must not serialize placeholder proof claims. | `app.chain.*` route names, runtime receipt roots, optional proof fields |
| Documentation authority wording | Canonical bootstrap and registry paths must stay truthful across docs and planning surfaces. | config paths, bootstrap manifest references, narrowed wording |
| Verification orchestrator dispatch | Reported checker modules must match the scripts actually executed by the orchestrator. | gate path metadata, executed script path, gate result |
| Managed verification toolchain | Managed verifier bootstrap must pin one local tool root, one explicit offline policy, and honest `UNKNOWN` states when local targets do not exist. | toolchain pins, sysroots, generated harnesses, gate statuses |
| Runtime and storage acceptance path | Aggregator acceptance tests must reach the real lineage and finalization logic without weakening wallet release guards. | feature graph edges, route rollout, lineage proofs, checkpoint finalization |
| Payment request, stealth, and receive ownership | Receive-side request binding must stay separated by domain, persisted chain scope, and explicit wallet-RPC chain metadata. | request domains, `claim_scope`, wallet chain label, receiver-card metadata |

## 🧪 Evidence Index

All 13 implementation slices reran `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`. The table below lists the distinctive live-code anchors and runtime proofs that close the slice-specific security claims.

| Evidence ID | Scope | Live code anchors | Runtime proof |
|-------------|-------|-------------------|---------------|
| `E01` | `065-01` theorem-owned validator acceptance | `crates/z00z_runtime/validators/src/verdict.rs:23,61,151,201`<br>`crates/z00z_rollup_node/src/da.rs:135,142,152,231,239`<br>`crates/z00z_runtime/validators/src/checkpoint.rs:27-32` | `test_hjmt_publication_contract`<br>`test_rollup_theorem_guard`<br>`test_da_local_sim`<br>`cargo test --release` |
| `E02` | `065-02` canonical checkpoint persistence | `crates/z00z_storage/src/checkpoint/store.rs:218,300,332,356,401,442`<br>`crates/z00z_storage/src/checkpoint/store_fs.rs:65,124,134`<br>`crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs:19,65` | `test_checkpoint_store`<br>`test_checkpoint_finalization`<br>`test_checkpoint_link_injective`<br>`test_checkpoint_acceptance`<br>`cargo test --release --quiet` |
| `E03` | `065-03` release build hardening | `crates/z00z_wallets/src/lib.rs:97,100,151`<br>`crates/z00z_simulator/src/lib.rs:6,11`<br>`.github/workflows/release-safety-guards.yml:21-24`<br>`scripts/audit/audit_release_feature_guards.sh` | `test_production_hardening`<br>`test_live_boundary_claims`<br>`bash scripts/audit/audit_release_feature_guards.sh`<br>`cargo test --release` |
| `E04` | `065-04` draft/debug simulator evidence truth | `crates/z00z_simulator/src/config.rs:343,352`<br>`crates/z00z_simulator/src/scenario_1/stage_12/mod.rs:41,150`<br>`crates/z00z_simulator/src/scenario_1/runtime_observability.rs:508,5688,5914,5948` | `test_draft_publication_rejected`<br>`test_public_lane_secret_free`<br>`cargo test --release -p z00z_simulator --test scenario_1`<br>`cargo test --release --quiet` |
| `E05` | `065-05` capability-typed privileged wallet paths | `crates/z00z_wallets/src/rpc/wallet_dispatcher_routes.rs:17,76,148,249`<br>`crates/z00z_wallets/src/rpc/key_rpc_impl.rs:83,93,103`<br>`crates/z00z_wallets/src/services/wallet_service.rs:92`<br>`crates/z00z_wallets/src/stealth/output.rs:969,997` | `test_sensitive_rpc_session`<br>`test_wallet_capability_matrix`<br>`test_rpc_route_coverage`<br>`test_stealth_output` |
| `E06` | `065-06` canonical wallet mutation and restore ownership | `crates/z00z_wallets/src/rpc/asset_rpc_support_state.rs:14,82,221`<br>`crates/z00z_wallets/src/tx/tx_digest.rs:26`<br>`crates/z00z_wallets/src/chain/broadcast_impl.rs:97,115`<br>`crates/z00z_wallets/src/chain/local_node_sim.rs:231`<br>`crates/z00z_wallets/src/services/wallet_actions_backup.rs:14,23,794,1054,1152`<br>`crates/z00z_wallets/src/rpc/key_rpc.rs:177` | `test_asset_rpc_mutations`<br>`test_wallet_restore_atomic`<br>`test_chain_client_sim`<br>`test_tx_digest_framing`<br>`cargo test --release` |
| `E07` | `065-07` fail-closed construction, redaction, and meta-gates | `crates/z00z_storage/src/settlement/store.rs:483`<br>`crates/z00z_storage/src/backend/redb/mod.rs:63,65`<br>`crates/z00z_networks/rpc/src/wasm_client.rs:60,135`<br>`crates/z00z_wallets/src/rpc/security_types.rs:188`<br>`crates/z00z_wallets/src/services/wallet_session_manager.rs:11`<br>`crates/z00z_wallets/src/tx/spend_verification.rs:6`<br>`.github/workflows/security-hygiene-guards.yml:21-36` | `bash scripts/audit/audit_secret_type_hygiene.sh`<br>`bash scripts/audit/audit_secret_eq_hygiene.sh`<br>`bash scripts/audit/audit_crypto_rng_hygiene.sh`<br>`bash scripts/audit/audit_boundary_panic_hygiene.sh`<br>`bash scripts/audit/audit_log_redaction_hygiene.sh`<br>`test_live_guardrails`<br>`cargo test --release` |
| `E08` | `065-08` placeholder public RPC and DTO cleanup | `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs:131,134,150,157`<br>`crates/z00z_wallets/src/rpc/tx_types.rs:127,158,179,324,338,467-482` | `test_rpc_truth`<br>`test_rpc_types_serialization`<br>`test_rpc_wiring_spec_a`<br>`test_receipt_info_serialization`<br>`cargo test --release` |
| `E09` | `065-09` final narrowed-claim source sweep | `.planning/codebase/STRUCTURE.md:134-135`<br>`.planning/codebase/ARCHITECTURE.md:221`<br>`crates/z00z_storage/src/settlement/root_types.md:86-89`<br>`scripts/audit_phase065_narrowed_wording.sh`<br>`crates/z00z_core/tests/test_live_guardrails.rs:38,310` | `bash scripts/audit_phase065_narrowed_wording.sh`<br>`cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`<br>`cargo test --release` |
| `E10` | `065-10` canonical verification gate entry paths | `.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh:860,875,886`<br>`.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh:1386,1470,1540` | `orchestrate.sh report project --dry-run`<br>`check-docs.sh` direct run<br>`verify-fast.sh --dry-run`<br>`audit-supply-chain.sh` direct run<br>`cargo test --release` |
| `E11` | `065-11` managed toolchain and offline gate recovery | `.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh:722-723`<br>`scripts/verification-tools/versions.env:12`<br>`scripts/install-verification-tools.sh:1-78`<br>`.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh:1-120,175`<br>`crates/z00z_core/src/lib.rs:83-87`<br>`crates/z00z_runtime/validators/src/lib.rs:5-9`<br>`.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh:36,63`<br>`.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-hax.sh:42,162,180,192,199`<br>`.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-tamarin.sh:40` | `install-verification-tools.sh --check --profile all --strict`<br>`install-verification-tools.sh --self-test --profile all --strict`<br>`verify-kani.sh`<br>`verify-miri.sh`<br>`verify-verus.sh`<br>`run-fuzz-short.sh`<br>`run-hax.sh`<br>`run-tamarin.sh`<br>`cargo test --release` |
| `E12` | `065-12` checkpoint lineage and storage determinism closure | `crates/z00z_runtime/aggregators/Cargo.toml:26-29`<br>`scripts/audit/audit_release_feature_guards.sh` | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_consensus`<br>`test_hjmt_route_rollout`<br>`test_hjmt_failover_same_lineage`<br>`test_live_guardrails`<br>`cargo test --release -p z00z_storage --test test_checkpoint_finalization`<br>`test_hjmt_root_generation`<br>`test_live_guardrails`<br>`cargo test --release`<br>`cargo tree -e features -p z00z_aggregators --features test-params-fast -i z00z_wallets` |
| `E13` | `065-13` payment request and stealth binding closure | `crates/z00z_wallets/src/rpc/asset_rpc_support_claims.rs:166,176,182`<br>`crates/z00z_wallets/src/rpc/asset_rpc_impl.rs:197,205`<br>`crates/z00z_crypto/tests/test_hash_policy.rs:117,123` | `test_claim_scope_chain`<br>`test_payment_request`<br>`test_asset_replay_protection`<br>`test_e2e_req_flow`<br>`test_stealth_output`<br>`test_view_key_contract`<br>`test_adversarial`<br>`test_sensitive_rpc_session`<br>`cargo test --release` |

## 📋 Threat Register

All 39 plan-time threats are dispositioned as `mitigate` and are `closed`.
Evidence IDs point to the live-code anchors and executable proofs listed in the
Evidence Index above.

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| `P01-T1` | Tampering | Validator theorem acceptance | mitigate | `E01`: one validator-owned theorem verifier and one mandatory theorem bundle on the accepted path | closed |
| `P01-T2` | Tampering | Publication binding coherence | mitigate | `E01`: publish and resolve paths bind artifact, exec input, link, and checkpoint id coherently | closed |
| `P01-T3` | Integrity | Accepted-path theorem contract | mitigate | `E01`: `ResolvedBatch` requires theorem-bearing construction, so placeholder or partial acceptance cannot ship | closed |
| `P02-T1` | Tampering | Canonical checkpoint authority lane | mitigate | `E02`: canonical final artifacts are admitted only through `seal_artifact`, not a peer raw save lane | closed |
| `P02-T2` | Integrity | Draft vs final proof contract | mitigate | `E02`: draft attestation bytes stay explicitly noncanonical and separate from final `cp_proof()` ownership | closed |
| `P02-T3` | Tampering | Link-evidence persistence | mitigate | `E02`: write-time link evidence checks and final-lane markers fail closed on mismatch or wrong lane | closed |
| `P03-T1` | Elevation | Release feature guard | mitigate | `E03`: release-capable wallet and simulator builds compile-fail on forbidden debug or fast-test features | closed |
| `P03-T2` | Information Disclosure | Public wallet debug export | mitigate | `E03`: public secret-export paths are removed and the remaining dump helper is isolated behind `internal_debug_tools` | closed |
| `P03-T3` | Tampering | Release-visible corruption hooks | mitigate | `E03`: cache and scheduler corruption hooks are hidden from release-capable surfaces and pinned by audit script plus CI | closed |
| `P04-T1` | Integrity | Draft-only stage-12 evidence | mitigate | `E04`: draft runs are classified private-only and blocked from producing public publication evidence | closed |
| `P04-T2` | Spoofing | Public checkpoint identity | mitigate | `E04`: public flows require real finalized `checkpoint_id_hex` and reject synthetic publication binding | closed |
| `P04-T3` | Information Disclosure | Public-lane secret exposure | mitigate | `E04`: public lane stays secret-free under explicit regression coverage | closed |
| `P05-T1` | Elevation | Privileged route guard enforcement | mitigate | `E05`: dispatcher routes consume typed capability objects through explicit `typed_handler_cap(...)` guard registration | closed |
| `P05-T2` | Integrity | Native vs wasm capability truth | mitigate | `E05`: verified capability truth is explicit in code and tests and fails closed on unsupported wasm cases | closed |
| `P05-T3` | Tampering | Raw stealth builder semantics | mitigate | `E05`: raw builders are visibly demoted to `*_unchecked` and no longer look canonical | closed |
| `P06-T1` | Integrity | Mutation tx identity | mitigate | `E06`: local mutations use one canonical digest story across build, broadcast, and local-node verification | closed |
| `P06-T2` | Availability | Restore durability and rollback | mitigate | `E06`: restore stages persist through `WalletRestoreMark` and recover through `resume_restore_mark(...)` | closed |
| `P06-T3` | Integrity | Rotation wording truth | mitigate | `E06`: `rotate_master_key` wording is pinned to persisted rewrite semantics rather than a placeholder story | closed |
| `P07-T1` | Availability | Local settlement-store startup | mitigate | `E07`: `SettlementStore::new()` uses a managed local backend and avoids panic or env-root drift | closed |
| `P07-T2` | Information Disclosure | Wasm RPC logging redaction | mitigate | `E07`: transport logging emits summaries only, while secret-bearing types render redacted output | closed |
| `P07-T3` | Tampering | Secret-equality and hygiene meta-gates | mitigate | `E07`: constant-time comparisons and repository audit scripts are live, enforced, and CI-wired | closed |
| `P08-T1` | Spoofing | Wallet-local chain observation naming | mitigate | `E08`: public RPC routes explicitly say `local_scan` or local tip and no longer imply durable global chain truth | closed |
| `P08-T2` | Integrity | Receipt proof DTO truth | mitigate | `E08`: `RuntimeConfirmationReceipt` carries explicit root or checkpoint evidence while placeholder `merkle_proof` stays absent | closed |
| `P08-T3` | Integrity | Placeholder contract removal | mitigate | `E08`: public DTO and route cleanup removed production-looking placeholder semantics from live surfaces | closed |
| `P09-T1` | Integrity | Canonical bootstrap-path wording | mitigate | `E09`: docs now point to `devnet_genesis_config.yaml` for bootstrap authority and to `devnet_assets_config.yaml` only as secondary data | closed |
| `P09-T2` | Integrity | Repo wording audit | mitigate | `E09`: a repo-owned wording audit plus guardrail test fail on retired canonical-path claims | closed |
| `P09-T3` | Integrity | Historical anchor containment | mitigate | `E09`: the remaining historical phrases are constrained to explicit allowlists instead of leaking into live authority text | closed |
| `P10-T1` | Integrity | Orchestrator dispatch path | mitigate | `E10`: the orchestrator now executes the same canonical skill-owned scripts it reports as checker modules | closed |
| `P10-T2` | Availability | Direct gate reachability | mitigate | `E10`: repeated `l0`, `l3`, and `l4` gate calls reach real scripts instead of nonexistent local wrappers | closed |
| `P10-T3` | Integrity | Canonical checker-module path | mitigate | `E10`: one canonical path now exists for each repeated verification gate entry | closed |
| `P11-T1` | Availability | Managed verification bootstrap | mitigate | `E11`: managed install, self-test, sysroot refresh, and explicit offline defaults close the bootstrap-failure class | closed |
| `P11-T2` | Integrity | Generated harness and tool-root coherence | mitigate | `E11`: Kani harness discovery, crate-self aliases, and pinned verifier tool roots now agree on one canonical graph | closed |
| `P11-T3` | Integrity | Honest `UNKNOWN` verification states | mitigate | `E11`: Verus, HAX, and Tamarin gates now surface truthful `UNKNOWN` outcomes when local targets or models are absent | closed |
| `P12-T1` | Integrity | Aggregator acceptance release path | mitigate | `E12`: aggregator `test-params-fast` no longer forwards the forbidden wallet feature edge | closed |
| `P12-T2` | Integrity | Lineage and finalization proof path | mitigate | `E12`: release-mode aggregator and storage suites now execute the intended route-rollout, lineage, and finalization logic | closed |
| `P12-T3` | Integrity | Release-guard preservation | mitigate | `E12`: wallet and simulator release guards remain fail-closed while the aggregator acceptance path is repaired | closed |
| `P13-T1` | Replay | Payment-request domain separation | mitigate | `E13`: explicit hash-policy tests pin request and receiver-card domains to the intended hash policy | closed |
| `P13-T2` | Spoofing | Persisted wallet-chain claim scope | mitigate | `E13`: claim receipts bind to persisted wallet chain state rather than mutable runtime environment drift | closed |
| `P13-T3` | Elevation | Wallet-RPC chain metadata authority | mitigate | `E13`: asset RPC chain metadata now comes from one canonical helper path instead of duplicate local mappings | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

## 🚩 Unregistered Flags

None. No `065-*-SUMMARY.md` file introduced a `## Threat Flags` section with a
new unmapped threat outside the plan-time register.

## ✅ Accepted Risks Log

No accepted risks. All 39 threats are closed by live code, executable
guardrails, or release-mode validation evidence on the project-owned path.

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-02 | 39 | 39 | 0 | Codex (`/gsd-secure-phase 065`) |

## ✅ Sign-Off

- [x] All threats have a disposition (`mitigate`)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-02
