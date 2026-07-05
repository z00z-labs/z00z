# 085 Cross-Crate Test Matrix - Gap-Only TODO

Audit date: 2026-07-04

Scope:
- `.planning/phases/085-Cross-Crate-Test-Matrix/1-todo.md`
- `.planning/phases/085-Cross-Crate-Test-Matrix/2-todo.md`
- Current crate implementation evidence under `crates/`
- Existing phase specs and TODOs in phases 067-075

Rules:
- This file lists only unresolved gaps.
- Every gap MUST be closed inside an existing phase 067-075.
- This file MUST NOT introduce a new phase.
- "Current evidence" names the closest implemented or planned surface so the closeout work does not duplicate already implemented foundations.
- Simulator evidence MUST NOT replace lower-level owner-crate tests. Simulator scenarios MUST cite the lower-level tests they depend on.

## 1. Gap Index

| Gap ID | 085 source | Gap | Implement in |
| --- | --- | --- | --- |
| GAP-085-001 | Cross-crate matrix ownership and simulator dependency rules | Missing machine-checkable dependency ledger from simulator scenarios to lower-level crate tests | Phase 70 Rollup Node |
| GAP-085-002 | Core object, claim-root, and checkpoint authority hardening | Claim-root emission policy is not yet proven as real/absent/rejected across claim and non-claim checkpoints | Phase 70 Rollup Node, constrained by Phase 068 Checkpoint Contract |
| GAP-085-003 | Core object, claim-root, and checkpoint authority hardening | Checkpoint proof validation exists but is split across multiple entrypoints instead of one authoritative verifier | Phase 70 Rollup Node |
| GAP-085-004 | Wallet receive/scan/import authority | Request-Bound Inbox has wallet-level tests and spec scenarios, but missing live cross-crate simulator scenarios | Phase 071 Request-Bound Inbox |
| GAP-085-005 | Offline/import/package graph coverage | Offline bundle and handoff graph are specified, but `OfflineTxBundleV1` and its cross-crate admission tests are not implemented | Phase 072 Offline Transaction |
| GAP-085-006 | Privacy/selective disclosure | Redaction and checkpoint audit foundations exist, but no selective audit/disclosure package scenario exists | Phase 075 Linked Liability |
| GAP-085-007 | Voucher/payroll/useful-work | Voucher and right object flows exist, but payroll, B2B, and useful-work vertical scenarios are missing | Phase 075 Linked Liability |
| GAP-085-008 | Linked liability | Linked-liability enforcement pipeline is still whitepaper/planning-only | Phase 075 Linked Liability |
| GAP-085-009 | Fee envelopes/rights wallet, machine capabilities, agentic rights | Right lifecycle foundations exist, but concrete economic verticals are missing | Phase 075 Linked Liability |
| GAP-085-010 | OnionNet local control plane | OnionNet is a placeholder seam; 067-075 need boundary and overclaim tests, not live overlay implementation | Phase 075 Linked Liability |

## 2. Detailed Gaps

### GAP-085-001 - Cross-Crate Scenario Dependency Ledger

085 requirement:
- Each owner crate MUST have local rule tests before a simulator scenario is accepted as integration evidence.
- Every simulator scenario MUST state the lower-level tests it depends on.
- Fixture names MUST make local/mock/test boundaries explicit.

Current evidence:
- `crates/z00z_simulator/tests/scenario_1/` contains broad integration coverage for object flows, HJMT proof-size/storage reports, rights lifecycle, publication/recovery, and local DA paths.
- Existing scenario reports include some evidence file paths, but there is no single machine-checkable matrix that maps each simulator scenario to owner-crate rule tests.
- No repository-level artifact currently proves that simulator coverage is never used as a substitute for core, crypto, storage, wallet, or rollup-node unit/integration tests.

Gap:
- The test matrix exists as planning intent, not as an enforced artifact.
- A future agent can still add simulator-only proof for a business rule without citing the crate-local tests that define the rule.

Implement in:
- Phase 70 Rollup Node.

Required closeout:
- Add a versioned cross-crate matrix artifact, for example `crates/z00z_simulator/tests/scenario_1/cross_crate_test_matrix.yaml` or an equivalent Rust test fixture.
- The artifact MUST list: row id, rule owner crate, lower-level test paths, simulator scenario path, fixture label, mock/local/test boundary, and accepted evidence files.
- CI or a Rust integration test MUST fail if a simulator scenario declares rule coverage without at least one owner-crate lower-level test.
- Scenario-specific rows SHOULD be contributed by phases 071, 072, and 075, but the enforcement harness SHOULD live in Phase 70 to avoid duplicating matrix logic.

### GAP-085-002 - Honest Claim-Root Emission Policy

085 requirement:
- `ClaimSourceRoot` and checkpoint `claim_root` MUST remain distinct from normal settlement/checkpoint roots.
- `claim_root` MUST be emitted only for claim-carrying checkpoint content.
- Synthetic aliases and unrelated root conversions MUST be rejected.
- Tests MUST cover real claim roots, absent claim roots, and rejected synthetic claim roots.

Current evidence:
- `crates/z00z_storage/src/settlement/identity.rs` defines separate `ClaimSourceRoot`, `CheckRoot`, and reject paths for invalid root conversions.
- Storage tests already cover rejected `TxDigest` to `CheckRoot` conversion and claim-source proof alignment.
- Checkpoint tests cover mismatched claim-root rejection.
- `crates/z00z_storage/src/checkpoint/build.rs` currently attaches a `claim_root` during batch checkpoint construction from settlement root material, which is too broad for the 085 rule.

Gap:
- Non-claim checkpoint content is not yet proven to produce an absent `claim_root`.
- Claim-root emission is not yet gated by explicit claim-carrying content.
- A normal settlement root can still be treated too closely to a claim root at the checkpoint-building boundary.

Implement in:
- Phase 70 Rollup Node.
- Phase 068 Checkpoint Contract supplies the contract constraint: optional `claim_root` is valid only when checkpoint statement content actually carries claim evidence.

Required closeout:
- Add storage tests for all three modes: claim-carrying checkpoint emits `claim_root`, non-claim checkpoint omits `claim_root`, synthetic or mismatched `claim_root` rejects.
- Refactor checkpoint batch construction so `claim_root` is derived only from a storage-owned claim source, not from a generic settlement root alias.
- Simulator checkpoint evidence MUST include both claim and non-claim checkpoint paths.
- Code MUST NOT silently coerce `TxDigest`, `SettlementStateRoot`, or `CheckRoot` into `ClaimSourceRoot`.

### GAP-085-003 - Single Checkpoint Proof Verifier Entrypoint

085 requirement:
- Proof validation authority MUST be owned by `z00z_storage::checkpoint`.
- Artifact seal, reload, and rehydrate paths MUST reuse one checkpoint-owned verification entrypoint.
- Divergent proof verifiers MUST NOT appear in codec, store, simulator, or rollup-node code.

Current evidence:
- Checkpoint proof validation exists in several places: proof constructor, draft finalization, artifact construction, codec contract checks, and store sealing.
- Tests already reject malformed proof-system ids, mismatched public input, tampered backend payloads, and some reload/tamper cases.
- Phase 70 explicitly names shared checkpoint proof verifier work as a local backlog item.

Gap:
- The implementation is behaviorally guarded but structurally split.
- Multiple validation surfaces make it easy for future decode, reload, or rehydrate paths to drift.

Implement in:
- Phase 70 Rollup Node.

Required closeout:
- Introduce one internal verifier function or verifier type under `z00z_storage::checkpoint`.
- Seal, codec decode, artifact load, reload, rehydrate, and simulator evidence checks MUST call that same verifier.
- Tests MUST prove that tampered proof bytes, wrong proof system, wrong public input, and wrong backend payload fail through every public artifact path.
- New checkpoint consumers MUST NOT implement local proof verification copies.

### GAP-085-004 - Request-Bound Inbox Cross-Crate Simulator Scenarios

085 requirement:
- Wallet receive, scan, and import authority MUST be tested at owner-crate level and then as cross-crate simulator behavior.
- Simulator scenarios MUST demonstrate integration only after lower-level wallet, crypto, and storage tests exist.

Current evidence:
- Wallet code includes request inbox and stealth request support.
- Wallet tests cover unsupported versions, remote scan workers, stale hints, and related receive/scan failure paths.
- Phase 071 defines scenario names such as request happy path, collision storm, expired flood, wrong-chain request, remote scan partition, stale hint defense, offline package handoff, and reorg rescue.
- Current crate search did not find those named `scenario_rbi_*` simulator scenarios implemented under `crates/z00z_simulator`.

Gap:
- Request-Bound Inbox is not yet represented as cross-crate simulator evidence.
- Current tests prove important wallet-local rules, but not the full integration flow through simulator, storage, and rollup/publication boundaries.

Implement in:
- Phase 071 Request-Bound Inbox.

Required closeout:
- Add simulator scenarios for the Phase 071 request-bound flow set: happy path, collision storm, expired flood, wrong chain, remote partition, stale hint, offline handoff, pruned history, and reorg rescue.
- Each scenario MUST cite lower-level wallet, crypto, storage, or rollup-node tests in the cross-crate dependency ledger from GAP-085-001.
- Negative scenarios MUST prove stale hints, wrong chain ids, expired requests, duplicate/collision cases, and unauthorized imports are rejected.
- Simulator fixtures MUST be labeled local/mock/test and MUST NOT imply live network delivery.

### GAP-085-005 - Offline Bundle And Handoff Graph Admission

085 requirement:
- Portable transaction/package parsing MUST be bounded and deterministic.
- Simulator claim-package consumers MUST bind to storage-owned claim-source truth.
- Offline and import flows MUST not create a second transaction authority outside storage/checkpoint verification.

Current evidence:
- Phase 072 specifies `OfflineTxBundleV1` as a wrapper over existing transaction/package semantics rather than a second transaction family.
- Current code has transaction package and claim package foundations, but `OfflineTxBundleV1` was not found in crate implementation.
- Phase 072 requires bundle admission rules, topological checks, replay rejection, working-window storage application, and report-only verification.

Gap:
- The offline handoff graph is still planning-level.
- Cross-crate admission tests do not yet prove that offline bundles preserve storage/checkpoint authority and bounded package parsing.

Implement in:
- Phase 072 Offline Transaction.

Required closeout:
- Implement `OfflineTxBundleV1` or the final agreed equivalent as a wrapper around existing transaction/package primitives.
- Add parser tests for size limits, version limits, malformed packages, unknown fields, replay, and duplicate edges.
- Add storage tests for working-window admission, topological closure, claim-source binding, and non-authoritative wallet pre-broadcast state.
- Add simulator offline handoff scenarios that cite lower-level package, storage, and wallet tests.
- Offline bundle logic MUST NOT create a new transaction family or bypass checkpoint/storage authority.

### GAP-085-006 - Selective Audit And Disclosure Package Scenario

085 requirement:
- Privacy/selective disclosure tests MUST cover credential show, selective audit, redaction, view-key paths, and leakage checks.
- Audit artifacts MUST prove only the intended disclosure surface is exposed.

Current evidence:
- Wallet debug export and logging tests include redaction behavior.
- `z00z_storage::checkpoint` has checkpoint audit foundations.
- Phase 075 privacy material defines selective disclosure, selective audit, leakage contracts, and "prove metric over reveal records" direction.
- No concrete selective audit package, disclosure package, or simulator scenario was found in crate implementation.

Gap:
- Redaction exists, but selective audit is not yet a tested cross-crate protocol surface.
- There is no artifact proving which fields are disclosed, which fields remain hidden, and which verifier accepts the disclosure.

Implement in:
- Phase 075 Linked Liability.

Required closeout:
- Define a purpose-bound selective audit/disclosure DTO or test artifact with explicit disclosed fields, hidden fields, verifier inputs, and leakage budget.
- Add wallet tests for export, redaction, view-key or disclosure authorization, and denied over-disclosure.
- Add storage/checkpoint tests proving disclosed audit claims bind to checkpoint evidence without revealing hidden records.
- Add simulator scenario coverage for at least one positive selective audit and one over-disclosure rejection.
- Selective audit code MUST NOT expose raw wallet history, hidden liabilities, secrets, or unrelated asset metadata.

### GAP-085-007 - Payroll, B2B, And Useful-Work Verticals

085 requirement:
- Voucher/payroll/useful-work coverage MUST include vertical flows, negative cases, and explicit non-cash-equivalence boundaries.

Current evidence:
- Core policies and simulator object-flow matrix cover voucher issue, offer, accept, reject, transfer, redeem, refund, expiry, rights, right-gated voucher actions, fee support, and wrong-family rejection.
- Wallet tests distinguish vouchers/rights from base asset inventory.
- Search evidence did not show concrete payroll, B2B, or useful-work vertical scenarios beyond planning or closeout wording.

Gap:
- Voucher mechanics are implemented, but payroll, B2B, and useful-work are not yet proven as concrete cross-crate verticals.
- The system does not yet demonstrate business-policy-level behavior for those verticals.

Implement in:
- Phase 075 Linked Liability.

Required closeout:
- Add policy descriptors or fixture definitions for payroll, B2B invoice/settlement, and useful-work reward flows.
- Add wallet projection tests that keep these objects separate from cash-like spendable assets unless a verifier explicitly permits redemption.
- Add storage tests for object state transitions, expiry, revocation, and replay rejection.
- Add simulator scenarios with positive and negative cases for each vertical.
- Vertical flows MUST NOT imply legal tender, deposit, or cash-equivalent semantics unless a later explicit compliance phase defines that semantics.

### GAP-085-008 - Linked Liability Enforcement Pipeline

085 requirement:
- Linked liability coverage MUST include linked debt, hidden commitment, fraud proof, lock/unlock, penalty policy, and wallet enforcement.

Current evidence:
- Phase 075 linked-liability whitepaper defines the target terms and openly states that full fraud proof, lock registry, live slashing, and exact wire/proof formats are not landed.
- Current crate search did not find concrete protocol types such as `LiabilityDomain`, `HiddenLiabilityCommitment`, `FraudProof`, or `PenaltyPolicy`.
- Existing object-flow tests include unrelated local lock-registry naming, but not linked-liability enforcement semantics.

Gap:
- Linked liability is still a planning and threat-model concept, not a crate-backed enforcement pipeline.
- There is no end-to-end proof that hidden liability commitments can be locked, challenged, proven fraudulent, penalized, and unlocked safely.

Implement in:
- Phase 075 Linked Liability.

Required closeout:
- Add core object and policy types for liability domain, commitment, lock, challenge, fraud proof, penalty, and unlock.
- Add crypto statement/hash definitions only after the privacy and exculpability leakage boundaries are finalized.
- Add storage persistence for lock/challenge/evidence state and replay-safe transitions.
- Add wallet enforcement that prevents spending or acting through locked liabilities and extracts conflict evidence without leaking unrelated private state.
- Add simulator positive and negative scenarios for lock, valid unlock, invalid unlock, fraud proof, penalty, replay, and privacy boundary.
- Linked-liability implementation MUST NOT rely on social assertions or wallet-only state as authority.

### GAP-085-009 - Agent And Machine Economic Verticals

085 requirement:
- Fee envelopes/rights wallet MUST cover fee-supported and agent/machine fee execution.
- Machine capabilities MUST cover charging, access, relay, and compute.
- Agentic rights MUST cover tool purchase, escrow, and payout.

Current evidence:
- Core rights config supports machine capability, data access, service entitlement, validator mandate, and one-time use classes.
- Wallet and simulator tests cover agentic right lifecycle and machine capability lifecycle, including consume, expiry, replay, and wrong binding/action rejection.
- Fee-supported transition tests exist in simulator object flows.
- No concrete end-to-end scenarios were found for machine charging/access/relay/compute as separate verticals, or for agent tool purchase/escrow/payout.

Gap:
- The rights substrate exists, but the economic vertical semantics requested by 085 are not fully implemented.
- Current tests prove lifecycle validity more than product-level execution behavior.

Implement in:
- Phase 075 Linked Liability.

Required closeout:
- Add concrete policy fixtures for machine charging, machine access, machine relay, machine compute, agent tool purchase, escrow, and payout.
- Add wallet tests for right projection, fee envelope selection, denied unauthorized execution, and fail-closed unknown right classes.
- Add storage tests for escrow/payout state if escrow or payout becomes persisted state.
- Add simulator verticals for at least one positive and one negative path per machine and agentic category.
- Agent/machine execution MUST NOT bypass fee envelope accounting, rights checks, or replay protection.

### GAP-085-010 - OnionNet Local Control-Plane Scope Reconciliation

085 requirement:
- OnionNet local control-plane coverage is listed in the matrix for route builder, packet classify, replay ledger, local mix, and helper flows.

Current evidence:
- `crates/z00z_networks/onionnet` is a placeholder seam and explicitly not a live overlay implementation.
- Wallet/app code routes through local fallback behavior.
- Phase 70 states live OnionNet work is out of scope and requires negative scheduling checks.
- Phase 075 privacy material discusses helper/OnionNet metadata leakage and non-claims boundaries.

Gap:
- The 085 row can be misread as requiring live OnionNet semantics in 067-075.
- Current implementation has placeholder types and boundary tests, but not a 067-075 closure artifact proving what is intentionally out of scope and what local/privacy boundary is still tested.

Implement in:
- Phase 075 Linked Liability.

Required closeout:
- Add privacy/helper boundary tests that prove wallet fallback remains local and no live OnionNet route/packet semantics are claimed.
- Add metadata-leak tests for helper-facing request fields that are in scope for Phase 075 privacy analysis.
- Add a Phase 70 negative scheduling guard or reuse its existing guard to prove live OnionNet implementation is not scheduled inside phases 067-075.
- Documentation and tests MUST label OnionNet behavior as placeholder/local/mock where applicable.
- Phases 067-075 MUST NOT implement live route building, packet mixing, or overlay replay-ledger semantics. Those belong to the later OnionNet phase.

