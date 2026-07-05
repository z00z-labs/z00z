# Phase 067 Source Audit

**Generated:** 2026-07-05
**Purpose:** prove that the Phase 067 packet covers the full normative
`067-TODO.md` authority, the exact `067-verdict.md` Local-Conformance-Simulation
authority, their exact Markdown corpus references, the mandatory `scenario_11`
carry-forward from `090-New-Scenarios/90-TODO.md`, and the anti-duplication or
anti-parallel-layer rules before implementation starts.

## Markdown Corpus Lock

| Source | Status | Packet owner | Notes |
| --- | --- | --- | --- |
| `.planning/phases/067-Sharded-Concensus/067-TODO.md` | COVERED | `067-CONTEXT.md`, `067-COVERAGE.md`, `067-01` through `067-09` | Base normative human-readable Phase 067 authority for planning and execution scoping. |
| `.planning/phases/067-Sharded-Concensus/067-verdict.md` | COVERED | `067-CONTEXT.md`, `067-COVERAGE.md`, `067-10` through `067-19` | Normative Local-Conformance-Simulation expansion source for the strong acceptance gate. |
| `.planning/phases/067-Sharded-Concensus/067-CONTEXT.md` | COVERED | `067-01` through `067-19`, `067-PLAN-REVIEW.md` | Context packet referenced by verdict and plans; evidence only, not a second authority layer. |
| `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md` | COVERED | `067-CONTEXT.md`, `067-01` through `067-19`, `067-PLAN-REVIEW.md` | Coverage packet referenced by verdict; evidence only, not a replacement for plan execution. |
| `.planning/phases/067-Sharded-Concensus/067-VERDICT-ITEM-AUDIT.md` | COVERED | `067-CONTEXT.md`, `067-19`, `067-PLAN-REVIEW.md` | Item-level traceability for all `253` dash or numbered items from `067-verdict.md`; evidence only, not a second task ledger. |
| `.planning/STATE.md` | COVERED | `067-CONTEXT.md`, `067-19`, `067-PLAN-REVIEW.md` | Status snapshot referenced by verdict; status evidence only. |
| `.planning/phases/067-Sharded-Concensus/067-10-PLAN.md` through `.planning/phases/067-Sharded-Concensus/067-19-PLAN.md` | COVERED | `067-CONTEXT.md`, `067-COVERAGE.md`, `067-19`, `067-PLAN-REVIEW.md` | Exact verdict expansion plan files; plan ids are file ids, not wave ids. |
| `.planning/phases/090-New-Scenarios/90-TODO.md` | COVERED | `067-CONTEXT.md`, `067-COVERAGE.md`, `067-01` through `067-19` | Mandatory `scenario_11` owner, artifact, test, and verification-anchor source. |
| `crates/z00z_runtime/aggregators/README.md` | COVERED | `067-CONTEXT.md`, `067-01-PLAN.md` | Explicitly named in the rename matrix and kept on the live doc path, not a copied doc lane. |
| Legacy non-canonical aggregator-consensus references | COVERED AS STALE DRIFT | `067-CONTEXT.md`, `067-09-PLAN.md`, `067-PLAN-REVIEW.md` | Older internal references point outside the live authority chain; they are tracked only as stale drift and never recreated as a second authority file. |
| `.planning/phases/067-Sharded-Concensus/wiki -results.md` | COVERED AS SUPPORTING ONLY | `067-CONTEXT.md` | Supporting non-canonical context only; not an authority source for claims, coverage, or acceptance. |

## Referenced Corpus Detailed Lock

| Source slice | Range | Inventory | Packet owner(s) | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `.planning/phases/090-New-Scenarios/90-TODO.md` `## 15. scenario_11 Shard Quorum Certificate And Secondary Replay Drill` | `1050-1290` | `h3=15`, `dash=81`, `num=13` | `067-CONTEXT.md`, `067-02`, `067-05`, `067-06`, `067-07`, `067-08`, `067-09`, `067-11`, `067-13` through `067-19` | COVERED | Scenario purpose, current-code anchors, independent home, required existing-crate additions, flow, invariants, artifact list, unit/integration/E2E tests, fault matrix, anti-placeholder gates, verification anchors, and completion criteria are carried into plans. Exact rows `Duplicate voter` and `one-secondary-stale` are explicitly preserved. |
| `crates/z00z_runtime/aggregators/README.md` rename target | `README phrase` | `secondary-aggregator takeover` | `067-CONTEXT.md`, `067-01` | COVERED | README wording remains a live doc target for the `standby` to `secondary aggregator` cleanup without creating a duplicate documentation lane. |

## Verdict Expansion Addendum

This source audit locks the original `067-TODO.md` Markdown inventory. The
expanded verdict work is tracked by `067-verdict.md`, `067-COVERAGE.md`, and
`067-10-PLAN.md` through `067-19-PLAN.md`; it does not renumber or reinterpret
the original `PHASE-0` through `PHASE-8` rows.

## Exact Verdict Inventory Lock

`067-verdict.md` is normative for the Local-Conformance-Simulation expansion.
This lock covers all verdict Markdown items:

- `9` H2 sections
- `44` H3 sections
- `227` dash-list bullets
- `26` numbered-list items
- `253` total dash or numbered verdict items
- `253/253` item-level rows in
  `.planning/phases/067-Sharded-Concensus/067-VERDICT-ITEM-AUDIT.md`

Every verdict item is owned by the nearest source range below and by the
corresponding numbered plan owners in `067-CONTEXT.md` and `067-10-PLAN.md`
through `067-19-PLAN.md`. A missing owner in this table fails planning.

| Section | Range | Bullet inventory | Packet owner(s) | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `## 🔎 Ответы по твоим пунктам` | `9-24` | `dash=7`, `num=0` | `067-CONTEXT.md`, `067-06`, `067-12`, `067-19` | COVERED | Direct answers remain in context and final conformance owners. |
| `## 🎯 Targeted Additions To Close The Missing Subquestions` | `25-308` | `dash=38`, `num=26` | `067-CONTEXT.md`, `067-05`, `067-06`, `067-08`, `067-09`, `067-10` through `067-19` | COVERED | Redistribution, config ownership, signature limits, process model, planner authority, additional questions, and local simulation requirements stay mapped. |
| `## ✅ Pro-Con And Doublecheck Audit` | `309-359` | `dash=8`, `num=0` | `067-CONTEXT.md`, `067-10` through `067-19` | COVERED | Local simulation remains valid only for artifact/API-backed claims; production external-infra overclaim remains forbidden. |
| `## 🧷 Strong Acceptance Gate For 067` | `360-594` | `dash=78`, `num=0` | `067-CONTEXT.md`, `067-07` through `067-19` | COVERED | All Strong Gates, hard blockers, and report-honesty constraints map to numbered plans. |
| `## 📦 Conformance Simulation Libraries And Frameworks` | `595-665` | `dash=24`, `num=0` | `067-CONTEXT.md`, `067-10` through `067-19` | COVERED | Already-present, direct-addition, external-backend, and non-Rust tool guidance remains dependency-owned and non-parallel. |
| `## 🧰 Concrete Add List To Implement The Whole Verdict` | `666-841` | `dash=56`, `num=0` | `067-CONTEXT.md`, `067-10` through `067-19` | COVERED | Concrete add anchors are either existing live files or proposed artifacts owned by exactly one plan. |
| `## 🛑 MUST решить в 067, не переносить` | `842-872` | `dash=10`, `num=0` | `067-CONTEXT.md`, `067-10` through `067-19` | COVERED | MUST-solve items stay in Phase 067 unless explicitly closed as `live-claim-removed`. |
| `## 🧭 Executable Plan Expansion Addendum` | `873-992` | `dash=6`, `num=0` | `067-CONTEXT.md`, `067-COVERAGE.md`, `067-10` through `067-19` | COVERED | Coverage audit, plan id lock, waves, inventory, installation sequence, and task-to-plan table remain exact. |
| `## ✅ Проверка` | `993-1003` | `dash=0`, `num=0` | `067-CONTEXT.md`, `067-PLAN-REVIEW.md` | COVERED | Verification conclusion is tracked as review evidence only. |

## Exact TODO Section Traceability

### H2 Coverage

| Section | Range | Bullet inventory | Packet owner(s) | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `## 1. Glossary` | `13-71` | `dash=0`, `num=0` | `067-CONTEXT.md`, `067-01` through `067-09` | COVERED | Vocabulary and object names are locked through the packet and must not drift. |
| `## 2. Executive Decision` | `72-87` | `dash=0`, `num=7` | `067-CONTEXT.md`, `067-05`, `067-07` | COVERED | End-to-end wallet package -> route -> replay -> certificate -> DA -> validator flow is preserved. |
| `## 3. Direct Answers` | `88-204` | `dash=24`, `num=10` | `067-CONTEXT.md`, `067-01` through `067-09` | COVERED | Child sections are mapped below; none are dropped or softened into advisory prose. |
| `## 4. Current Live Model` | `205-286` | `dash=30`, `num=0` | `067-CONTEXT.md`, `067-01` through `067-07` | COVERED | Live ingress, routing, placement, recovery, and theorem boundaries are packet-owned. |
| ``## 5. How `sim_5a7s` Works Today`` | `287-329` | `dash=19`, `num=0` | `067-CONTEXT.md`, `067-01`, `067-04`, `067-05`, `067-06` | COVERED | Current topology truth, missing harness work, and no-global-quorum rule are preserved. |
| `## 6. Mermaid View Pack` | `330-474` | `dash=6`, `num=0` | `067-CONTEXT.md`, `067-05`, `067-06`, `067-07` | COVERED | Diagram semantics are carried as flow and lifecycle constraints, not as optional illustration. |
| `## 7. Target Material Assessment` | `475-505` | `dash=18`, `num=0` | `067-CONTEXT.md`, `067-01`, `067-02`, `067-03`, `067-07`, `067-08`, `067-09` | COVERED | Current directives and external-blocker rules are explicitly locked. |
| `## 8. External and Deferred Target Appendix` | `506-1200` | `dash=131`, `num=33` | `067-CONTEXT.md`, `067-08`, `067-09` | COVERED | External backend material remains packet-owned as simulated local proof, not as live implementation overclaim. |
| `## 9. Correct Target Model` | `1201-1322` | `dash=60`, `num=0` | `067-CONTEXT.md`, `067-02`, `067-03`, `067-04`, `067-05`, `067-07`, `067-08`, `067-09` | COVERED | Core invariants and artifact shapes are bound to concrete plan owners below. |
| `## 10. Failure, Join, and Rotation Semantics` | `1323-1376` | `dash=8`, `num=5` | `067-CONTEXT.md`, `067-04`, `067-05`, `067-06`, `067-08` | COVERED | Crash, split-brain, stale, join, removal, and rotation flows remain explicit execution work. |
| `## 11. DA and Publication Model` | `1377-1402` | `dash=7`, `num=0` | `067-CONTEXT.md`, `067-07`, `067-09` | COVERED | Current local DA and external Celestia carry-forward are mapped to the right downstream plans. |
| `## 12. Theorem and Proof Work` | `1403-1419` | `dash=8`, `num=0` | `067-CONTEXT.md`, `067-07`, `067-08`, `067-09` | COVERED | Agreement, replay equivalence, and publication-binding proof obligations remain mandatory. |
| `## 13. Required Full Simulation Harness` | `1420-1527` | `dash=32`, `num=13` | `067-CONTEXT.md`, `067-05`, `067-06`, `067-07` | COVERED | `scenario_11` scope, artifacts, all-shard and failure matrix requirements are preserved. |
| `## 14. Implementation Phases` | `1528-1683` | `dash=64`, `num=0` | `067-CONTEXT.md`, `067-01` through `067-09` | COVERED | Exact phase-to-plan mapping is one-to-one and is not reinterpreted. |
| `## 15. Practical Recommendation on Rotation` | `1684-1702` | `dash=7`, `num=0` | `067-CONTEXT.md`, `067-06`, `067-09` | COVERED | Stable-primary and safe-activation guidance is preserved as normative planning discipline. |
| `## 16. Doublecheck Coverage Matrix` | `1703-1727` | `dash=4`, `num=0` | `067-CONTEXT.md`, `067-PLAN-REVIEW.md`, `067-01`, `067-09` | COVERED | Anti-drift statements and no-graphify-evidence rule are preserved in the review packet. |
| `## 17. Source Evidence Map` | `1728-1761` | `dash=29`, `num=0` | `067-CONTEXT.md`, `067-01` through `067-09` | COVERED | Code anchors are carried into current-code-ref sections and review evidence. |
| `## 18. Bottom Line` | `1762-1767` | `dash=0`, `num=0` | `067-CONTEXT.md`, `067-01` through `067-09` | COVERED | Local quorum-certificate work stays first; BFT/Celestia stays second-layer only. |
| `## 19. Implementation Contract Addendum` | `1768-1784` | `dash=0`, `num=12` | `067-CONTEXT.md`, `067-02` through `067-09` | COVERED | Addendum rules are preserved as packet-level hard gates and proposed-target discipline. |

### H3 Coverage

| Section | Range | Bullet inventory | Packet owner(s) | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `### 3.1. What is the quorum made from?` | `90-100` | `dash=2`, `num=0` | `067-02`, `067-04`, `067-05` | COVERED | Primary-plus-ready-secondary membership semantics stay explicit. |
| `### 3.2. Do secondary aggregators also need to recalculate everything?` | `101-114` | `dash=7`, `num=0` | `067-03`, `067-05` | COVERED | Independent replay obligations remain mandatory. |
| `### 3.3. Is this consensus BFT today?` | `115-120` | `dash=0`, `num=0` | `067-01`, `067-09` | COVERED | CFT-now and BFT-later wording is explicit. |
| `### 3.4. What does Celestia publish today?` | `121-126` | `dash=0`, `num=0` | `067-01`, `067-07`, `067-09` | COVERED | Local DA now, external Celestia later remains explicit. |
| `### 3.5. What does aggregator consensus prove?` | `127-139` | `dash=6`, `num=0` | `067-02`, `067-04`, `067-07` | COVERED | Route-, membership-, and publication-bound proof semantics remain explicit. |
| `### 3.6. Protocol terminology` | `140-154` | `dash=4`, `num=0` | `067-01` | COVERED | `secondary aggregator` vocabulary is preserved with no active alias lane. |
| `### 3.7. Concrete live-code rename recommendation` | `155-204` | `dash=5`, `num=10` | `067-01` | COVERED | Full rename matrix, no alias rules, and CI guard stay explicit. |
| `### 4.1. Ingress and routing` | `207-225` | `dash=6`, `num=0` | `067-03`, `067-05` | COVERED | Route digest, sorted shards, and full coverage checks are preserved. |
| `### 4.2. Placement and dispatch` | `226-237` | `dash=3`, `num=0` | `067-01`, `067-05`, `067-06` | COVERED | Live placement and dispatch-owner semantics remain explicit. |
| `### 4.3. Local consensus seam` | `238-253` | `dash=7`, `num=0` | `067-04` | COVERED | Same-term freeze, local majority, and generation-bound membership remain explicit. |
| `### 4.4. Recovery and failover` | `254-276` | `dash=14`, `num=0` | `067-03`, `067-06` | COVERED | Full reject matrix and secondary readiness remain plan-owned. |
| `### 4.5. DA and theorem path` | `277-286` | `dash=0`, `num=0` | `067-07` | COVERED | Publication-binding and theorem gate remain downstream requirements. |
| `### 6.1. C4 Component View: Shard-Local Quorum Boundary` | `344-400` | `dash=0`, `num=0` | `067-05`, `067-07`, `067-09` | COVERED | Component-boundary semantics are preserved through plan owners, not by copied diagrams. |
| `### 6.2. Dynamic View: Required Full Simulation Path` | `401-443` | `dash=0`, `num=0` | `067-05`, `067-07` | COVERED | Package-to-validator flow is preserved as the `scenario_11` contract. |
| `### 6.3. Lifecycle View: Secondary, Primary, Rotation, and Failure` | `444-474` | `dash=0`, `num=0` | `067-06` | COVERED | Observer, ready-secondary, primary, frozen, and retired lifecycle states remain explicit. |
| `### 7.1. Promote to current implementation directives` | `479-491` | `dash=10`, `num=0` | `067-02`, `067-03`, `067-07`, `067-08` | COVERED | DA-before-ordering, deterministic replay, availability, evidence, and non-goals are preserved. |
| `### 7.2. Keep external blockers out of current claims` | `492-505` | `dash=8`, `num=0` | `067-01`, `067-03`, `067-04`, `067-09` | COVERED | No BFT-overclaim, no new consensus crate, and no bypass of live routing rules remain explicit. |
| `### 8.1. Dependency candidates and links` | `518-646` | `dash=0`, `num=0` | `067-09` | COVERED | External dependencies remain backend-planning inputs only. |
| `### 8.2. External crate layout suggestion` | `647-706` | `dash=7`, `num=0` | `067-09` | COVERED | Dedicated external crate stays optional and later-layer only. |
| `### 8.3. BFT object model retained` | `707-736` | `dash=6`, `num=0` | `067-02`, `067-04`, `067-09` | COVERED | `CommitSubject` and certificate artifacts stay the bridge to later BFT forms. |
| `### 8.4. Networked BFT protocol pipeline retained` | `737-763` | `dash=0`, `num=16` | `067-09` | COVERED | External pipeline is preserved as a simulated backend contract only. |
| `### 8.5. BFT proposal validation checklist retained` | `764-799` | `dash=19`, `num=0` | `067-09` | COVERED | Proposal-validation conditions remain explicit backend acceptance rules. |
| `### 8.6. Backend, network, and storage suggestions retained` | `800-931` | `dash=24`, `num=0` | `067-08`, `067-09` | COVERED | Backend adapters, transports, durable stores, and crash recovery remain planned surfaces. |
| `### 8.7. External Celestia and settlement suggestions retained` | `932-984` | `dash=11`, `num=9` | `067-09` | COVERED | External DA sequencing, blob shape, invariants, and degraded-mode behavior remain explicit. |
| `### 8.8. APIs, config, and metrics retained` | `985-1044` | `dash=14`, `num=0` | `067-09` | COVERED | Config, metrics, and alert carry-forward remain explicit in the BFT/Celestia plan. |
| `### 8.9. Tests, fuzzing, chaos, and runbooks retained` | `1045-1107` | `dash=36`, `num=0` | `067-08`, `067-09` | COVERED | Signature, transport, withholding, crash, chaos, and operator-runbook bullets remain explicit. |
| `### 8.10. Networked BFT implementation phases retained` | `1108-1120` | `dash=0`, `num=8` | `067-09` | COVERED | Later backend phase order remains explicit and bounded behind the proven subject interface. |
| `### 8.11. Networked BFT invariants and parameters retained` | `1121-1155` | `dash=11`, `num=0` | `067-07`, `067-08`, `067-09` | COVERED | Agreement, availability, and settlement-finality invariants remain explicit proof obligations. |
| `### 8.12. External integration boundary retained` | `1156-1200` | `dash=0`, `num=0` | `067-09` | COVERED | Integration boundary remains a later-layer contract only. |
| `### 9.1. Core invariant` | `1203-1212` | `dash=0`, `num=0` | `067-CONTEXT.md`, `067-02` through `067-07` | COVERED | Same-subject alignment across replay, certificate, DA, and validator remains explicit. |
| `### 9.2. Primary role` | `1213-1227` | `dash=8`, `num=0` | `067-04`, `067-05`, `067-07` | COVERED | Primary duties remain bound to ordered batch, replay collection, and DA submission. |
| `### 9.3. Secondary aggregator role` | `1228-1246` | `dash=12`, `num=0` | `067-03`, `067-05`, `067-06`, `067-08` | COVERED | Replay inputs and role boundaries remain explicit. |
| `### 9.4. Commit subject` | `1247-1276` | `dash=21`, `num=0` | `067-02`, `067-03`, `067-04`, `067-07` | COVERED | Full subject-field list remains explicit and domain-separated. |
| `### 9.5. Vote artifact` | `1277-1291` | `dash=8`, `num=0` | `067-02`, `067-04`, `067-08` | COVERED | Vote-field list, role binding, and signature seam remain explicit. |
| `### 9.6. Quorum certificate` | `1292-1322` | `dash=11`, `num=0` | `067-02`, `067-04`, `067-07`, `067-09` | COVERED | Certificate-field list and reject conditions remain explicit. |
| `### 10.1. Primary failure before commit` | `1325-1328` | `dash=0`, `num=0` | `067-05`, `067-06` | COVERED | Pre-quorum crash and no-publication behavior remain explicit. |
| `### 10.2. Primary failure after local commit but before DA publication` | `1329-1334` | `dash=0`, `num=0` | `067-05`, `067-06`, `067-07` | COVERED | Exact-certificate publication resume remains explicit. |
| `### 10.3. Split brain` | `1335-1342` | `dash=3`, `num=0` | `067-04`, `067-08` | COVERED | Conflict freeze and evidence remain explicit. |
| `### 10.4. Stale secondary aggregator` | `1343-1346` | `dash=0`, `num=0` | `067-03`, `067-05`, `067-06` | COVERED | Stale-root reject remains explicit. |
| `### 10.5. Adding a new aggregator` | `1347-1358` | `dash=0`, `num=5` | `067-06` | COVERED | Observer -> ready-secondary -> voting-member -> possible-primary lifecycle remains explicit. |
| `### 10.6. Removing an aggregator` | `1359-1362` | `dash=0`, `num=0` | `067-06` | COVERED | Removed-member rejection remains explicit. |
| `### 10.7. Planned primary rotation` | `1363-1376` | `dash=5`, `num=0` | `067-06` | COVERED | Boundary-only rotation and mixed-generation reject remain explicit. |
| `### 11.1. Current local DA` | `1379-1388` | `dash=0`, `num=0` | `067-07` | COVERED | Current local DA path remains the first downstream owner. |
| `### 11.2. External Celestia DA` | `1389-1402` | `dash=7`, `num=0` | `067-09` | COVERED | External Celestia artifacts remain explicit but later-layer and locally simulated. |
| `### 13.1. Full local simulation code-dependency map` | `1446-1465` | `dash=0`, `num=0` | `067-05`, `067-06`, `067-07` | COVERED | The exact local-owner seam map remains explicit in the harness plans. |
| `### 13.2. Full local simulation flow contract` | `1466-1527` | `dash=32`, `num=0` | `067-05`, `067-06`, `067-07` | COVERED | Happy path, all-shard sweep, crash, stale, offline, drift, rotation, and validator reject matrices remain explicit. |
| `### 14.1. Phase 0: Terminology and boundary cleanup` | `1530-1545` | `dash=6`, `num=0` | `067-01` | COVERED | Exact phase owner. |
| `### 14.2. Phase 1: Commit subject and certificate types` | `1546-1565` | `dash=10`, `num=0` | `067-02` | COVERED | Exact phase owner. |
| `### 14.3. Phase 2: Secondary replay verifier` | `1566-1581` | `dash=6`, `num=0` | `067-03` | COVERED | Exact phase owner. |
| `### 14.4. Phase 3: Local quorum certificate integration` | `1582-1598` | `dash=7`, `num=0` | `067-04` | COVERED | Exact phase owner. |
| ``### 14.5. Phase 4: End-to-end `sim_5a7s` harness`` | `1599-1614` | `dash=6`, `num=0` | `067-05` | COVERED | Exact phase owner. |
| `### 14.6. Phase 5: Join, removal, and rotation simulation` | `1615-1633` | `dash=9`, `num=0` | `067-06` | COVERED | Exact phase owner. |
| `### 14.7. Phase 6: Validator and theorem binding` | `1634-1649` | `dash=6`, `num=0` | `067-07` | COVERED | Exact phase owner. |
| `### 14.8. Phase 7: Network and signature adapter` | `1650-1666` | `dash=7`, `num=0` | `067-08` | COVERED | Exact phase owner. |
| `### 14.9. Phase 8: BFT and Celestia backend` | `1667-1683` | `dash=7`, `num=0` | `067-09` | COVERED | Exact phase owner. |

## Coverage Summary

- Exact Markdown corpus refs named by `067-TODO.md`: `3/3` live references plus
  one supporting non-canonical note are accounted for.
- Exact TODO structure accounted for:
  - `19` H2 sections
  - `55` H3 sections
  - `447` dash-list bullets
  - `80` numbered-list items
- `067-SOURCE-AUDIT.md` is traceability evidence only. It is not a second
  planning authority and does not replace `067-TODO.md`.
- Graphify is not used as evidence for any row in this audit. Coverage is tied
  only to local docs, plans, tests, config, and code anchors.
- No TODO section is satisfied by inventing a mirror code path, a second
  simulator lane, a fake live BFT claim, or a second authority layer.
