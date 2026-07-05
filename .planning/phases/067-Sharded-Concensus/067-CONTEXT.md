# Phase 067: Sharded Concensus - Context

**Gathered:** 2026-07-03
**Status:** Reviewed for plan coverage
**Source:** PRD Express Path (`.planning/phases/067-Sharded-Concensus/067-TODO.md`)

<verdict>
## 🎯 Verdict

`067-TODO.md` is the normative Phase 067 authority for this planning pass.
Literal `TASK-NNN` rows do not exist in the current artifact, so planning must
map the required implementation groups from `067-TODO.md` Section 14
(`14.1` through `14.9`) to numbered `067-NN-PLAN.md` files without inventing a
parallel task ledger.

Legacy aggregator-consensus spec references conflict with the live workspace
and with the explicit user instruction that `067-TODO.md` is normative.
Planning therefore treats those non-canonical references as stale internal
drift and must not create a second authority plane to satisfy them.

`wiki -results.md` is supporting non-canonical context only.

`scenario_11` in `.planning/phases/090-New-Scenarios/90-TODO.md` is mandatory
planning input for the full local harness, evidence artifacts, and verification
anchors, but it does not replace `067-TODO.md` as the Phase 067 authority.
</verdict>

<domain>
## ⚙️ Phase Boundary

Phase 067 hardens the existing shard-local aggregator seam inside
`crates/z00z_runtime/aggregators` into a real local quorum-certificate pipeline.
The immediate target is not live network BFT and not live Celestia. The
immediate target is:

1. wallet-style package ingress normalization;
2. deterministic route-table shard selection;
3. primary execution subject construction;
4. independent secondary replay before voting;
5. per-shard quorum certificate formation;
6. local DA binding of the certificate digest;
7. validator acceptance or rejection against the same subject;
8. independent `scenario_11` evidence and fault-matrix coverage.

External transport, external DA transport, and devnet/provider behavior may be
simulated locally, but no plan may close on placeholder DTOs, no-op runners,
docs-only claims, or compile-only proof for runtime behavior.
</domain>

<decisions>
## 🔑 Implementation Decisions

### D-067-01 Normative Authority And Drift Handling
- `067-TODO.md` is normative.
- No plan may create or imply a second Phase 067 authority file.
- Legacy non-canonical aggregator-consensus references are treated as stale
  drift unless a later explicit repository task rebinds them to the same live
  authority text.

### D-067-02 Required Group Inventory
- Unique literal `TASK-NNN` count is `0`.
- Base required plan groups are the nine implementation groups in
  `067-TODO.md` Section 14: `PHASE-0` through `PHASE-8`.
- `067-verdict.md` adds ten required Local-Conformance-Simulation groups:
  `VERDICT-LCS-01` through `VERDICT-LCS-10`.
- Total required Phase 067 executable plan groups after the verdict addendum:
  `19`.
- Each required group maps to exactly one `067-NN-PLAN.md`.

### D-067-03 Existing Runtime Seam First
- Implementation starts inside `crates/z00z_runtime/aggregators`.
- Public exports go through the `z00z_aggregators` crate root.
- Do not create a separate production consensus crate before the local seam is
  proven through tests and `scenario_11`.

### D-067-04 Honest Consensus Vocabulary
- The live system is local deterministic CFT quorum until stronger proof exists.
- Protocol prose uses `secondary aggregator`.
- Active code, config, fixtures, tests, and docs must remove live `standby`
  naming without aliases or compatibility shims.

### D-067-05 First-Class Quorum Artifacts
- Add `CommitSubject`, `ShardVote`, `ShardQuorumCertificate`, and
  `SecondaryReplayVerifier`.
- Digest material must use stable binary encoding with explicit domain and
  version bytes.
- Membership digest is the ordered set `{primary_id} + ready secondary
  aggregators` for one shard and one placement generation only.

### D-067-06 Real Project Primitives Only
- Routing uses real `IngressBoundary`, `WorkItem`, `ShardRouteTable`, and
  `BatchPlanner`.
- Recovery uses real `RecoveryBoundary`, `ShardRecoveryRecord`, and live
  journal-lineage and root metadata.
- Publication uses real `PublicationRequest`, `PublishedBatch`, and
  `LocalDaAdapter`.
- Validation uses real theorem and validator boundaries.
- Signature and digest work must use current `z00z_crypto` domain-separation and
  signature primitives when real signatures are added.
- HJMT, state-root, checkpoint, publication, validator, crypto, and utility
  behavior must reuse existing `z00z_storage`, `z00z_core`, `z00z_crypto`, and
  `z00z_utils` primitives rather than introducing a parallel implementation
  layer.

### D-067-07 Independent Scenario Ownership
- `scenario_11` is the owner of the end-to-end quorum-certificate harness.
- `scenario_1` remains reference-only and must not gain new
  quorum-certificate-specific stages or observability fields.

### D-067-08 External Layers Stay Behind Local Proof
- Network BFT, HotStuff, libp2p, and Celestia are valid design targets only
  after the local harness and local certificate semantics are proven.
- Local deterministic simulation may fake the external transport boundary or DA
  provider boundary, but it must still use real routing, planning, execution,
  replay, storage, publication, theorem, and validator logic.
- Dependency candidates listed in `067-TODO.md` Section `8.1` are deferred
  adapter options, not blanket approval to add third-party crates while an
  equivalent repository-owned primitive already exists.

### D-067-09 Anti-Placeholder Rule
- No plan may close on placeholder, scaffold-only, TODO-only, constant-digest,
  hard-coded vote, string-only, docs-only, or compile-only behavior.
- Acceptable implementation depths are `full`, `simulated-full`, and
  `live-claim-removed`.

### D-067-10 Strict TODO Section Lock
- All `19` H2 sections and all `55` H3 sections from `067-TODO.md` remain
  normative for the packet.
- Exact line-range, bullet-count, and plan-owner traceability is locked in
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`.
- No review or execution step may silently drop Sections `1` through `19`,
  collapse Section `16` into an untracked note, or ignore Section `17` code
  anchors and Section `19` addendum rules.

### D-067-11 No Graphify Authority And No Parallel Layer
- Graphify may be used only for codebase orientation and must never be used as
  evidence for coverage, acceptance, or implementation truth.
- No duplicate codebase logic, mirror abstraction, or parallel planning
  authority may be introduced where the live codebase already has an owner.
- New file, module, test, config, or doc homes named in the numbered plans are
  proposed targets when absent in the current worktree; implementation must
  prefer a tighter existing owner if one already exists.

### D-067-12 Verdict Expansion Is Mandatory Local Scope
- The expanded verdict work is not a future backlog. `067-10` through `067-19`
  are required Phase 067 plan groups.
- The expansion deliberately rejects a four-plan umbrella split because that
  would hide independent blockers for dependencies, durability, planner
  authority, process/devnet realism, transport faults, HotStuff-like local
  semantics, Celestia-local artifacts, structured evidence, glossary/report
  honesty, and the final integrated gate.
- Acceptable verdict task implementation depths remain `full`,
  `simulated-full`, or `live-claim-removed`.
</decisions>

<constraints>
## 🛑 Requirement Gate Contract

- `REQ-067-001`: `067-TODO.md` stays normative for planning and execution
  scoping.
- `REQ-067-002`: The plan corpus must not create a second authority layer for
  Phase 067.
- `REQ-067-003`: Existing runtime work stays inside
  `crates/z00z_runtime/aggregators` and the `z00z_aggregators` facade until the
  local seam is proven.
- `REQ-067-004`: Active protocol-facing `standby` naming must be removed from
  live code, config, fixtures, tests, and docs without aliases.
- `REQ-067-005`: Quorum artifacts must use canonical binary encoding with
  explicit domain/version bytes and deterministic digest tests.
- `REQ-067-006`: Membership digest includes only the live primary and ready
  secondary aggregators for one shard and one placement generation.
- `REQ-067-007`: Validator and local DA acceptance must reject missing,
  detached, stale, mixed, or mismatched certificate binding once the gate is
  enabled.
- `REQ-067-008`: `scenario_11` must be an independent scenario target and must
  not modify `scenario_1` ownership.
- `REQ-067-009`: Fake only external transport, remote process, external DA
  transport, wall-clock, or unavailable third-party network behavior. Use real
  project primitives everywhere else.
- `REQ-067-010`: Every plan must contain explicit artifacts, tests, expected
  results, simulation requirements, evidence gates, and anti-placeholder proof.
- `REQ-067-011`: External BFT and Celestia work may close only through local
  deterministic simulation and 3f+1 or 2f+1 proof, not through future-only
  prose.
- `REQ-067-012`: Verdict expansion groups `VERDICT-LCS-01` through
  `VERDICT-LCS-10` must be planned and executed inside Phase 067, not deferred
  to later phases.
</constraints>

<verdict_coverage_matrix>
## 🧾 Verdict Coverage Matrix

Every bullet family in `067-verdict.md` must resolve to at least one numbered
plan and must stay inside the existing runtime, storage, DA, validator, and
simulator owners. This matrix is normative traceability for plan review only; it
does not create a second task ledger.

### Direct Answers And Targeted Additions

| Verdict bullet family | Owning plans | Required reflection |
| --- | --- | --- |
| RAID-like redistribution as placement-generation reassignment | `067-06`, `067-12`, `067-19` | observer catch-up, lawful promotion, removed-member rejection, membership-digest recalculation, route-generation proof |
| Exact `SIM-5A7S` meaning and config ownership | `067-05`, `067-13`, `067-19` | default local 5 aggregator / 7 shard topology, existing `config/hjmt_runtime/sim_5a7s`, no BFT claim on 2-of-3 CFT quorums |
| CFT/BFT/HotStuff and `BDR`/`redb` clarification | `067-01`, `067-09`, `067-10`, `067-15` | dependency truth, no `BDR` claim, `redb` only where already owned, BFT only with `3f+1` and `2f+1` proof |
| Deterministic simulator signature versus production signature | `067-08`, `067-10`, `067-14`, `067-18` | deterministic local signer is test-only; production signature seam binds canonical bytes, domain, membership, term, subject, and route |
| End-of-067 claim honesty for failover, HotStuff, local QC, and glossary terms | `067-11`, `067-15`, `067-18`, `067-19` | implemented locally, simulated-full, live-claim-removed, or not claimed; no future maybe in final report |
| Multi-aggregator quorum process model | `067-10`, `067-13`, `067-14`, `067-19` | runnable local process/devnet harness or explicit equivalent local-task proof; transport faults cannot manufacture votes |
| Planner authority and planner failover question | `067-12`, `067-19` | planner HA must be implemented or removed as a live claim; every aggregator recomputes the canonical plan locally |
| Additional questions 1-12 | `067-06`, `067-11`, `067-12`, `067-14`, `067-15`, `067-16`, `067-17`, `067-18`, `067-19` | stale route, restart, removed member replay, early vote eligibility, post-quorum crash, DA mismatch, equivocation, injected vote, BFT profile, Celestia negatives, regenerated reports, and final claim levels |
| Celestia, network, and devnet local simulation answer | `067-13`, `067-14`, `067-15`, `067-16`, `067-19` | fake only allowed external boundaries; use real route, package, replay, QC, storage, DA artifact, validator, watcher, and state primitives |
| Pro/con/doublecheck decision and consolidation rule | `067-10` through `067-19` | local conformance simulation is valid only when production-external claims are fenced or removed |

### Strong Acceptance Gates

| Gate | Owning plans | Required proof |
| --- | --- | --- |
| Gate 1 Glossary Traceability | `067-18`, `067-19` | term -> code owner -> artifact/API -> positive test -> negative test -> claim level |
| Gate 2 One Real End-To-End Path | `067-05`, `067-19` | wallet package through route, subject, replay, QC, DA/Celestia-local binding, validator verdict, and report |
| Gate 3 Validator Requires QC | `067-07`, `067-16`, `067-19` | reject missing, detached, stale, or mismatched QC |
| Gate 4 Celestia-Local Artifact Complete | `067-16` | namespace, blob, commitment, height, anchor, QC digest, payload, challenge/degraded state |
| Gate 5 Celestia Negative Tests | `067-16` | wrong namespace, wrong commitment, missing payload, stale anchor, mismatched QC, unanchored height, validator-rejected blob |
| Gate 6 Network Simulation Is Not Vote Injection | `067-08`, `067-14` | peer identity, envelopes, delay, reorder, drop, duplicate, partition, heal, restart/reconnect, replay protection |
| Gate 7 BFT Claims Need BFT Math | `067-09`, `067-15` | `n >= 3f + 1`, `quorum >= 2f + 1`, below-threshold rejection |
| Gate 8 HotStuff-Like Backend Behind The Seam | `067-09`, `067-15` | views, leaders, proposals, view-change, local backend QC, no bypass around subject/replay/validator |
| Gate 9 Production Signature Seam | `067-08`, `067-10`, `067-18` | deterministic signer fenced to simulation; production seam uses real crypto over canonical bytes |
| Gate 10 Multi-Process Or Devnet Harness | `067-13`, `067-19` | process/local-task evidence for start, partition, crash, restart, reconnect, and quorum |
| Gate 11 Planner HA Or Claim Removal | `067-12`, `067-18` | HA implemented or live claim removed; planner authority not ambiguous |
| Gate 12 Crash Recovery | `067-11`, `067-13`, `067-19` | votes, QC, publication state, DA/Celestia-local anchor, and validator decision recover or explicitly downgrade |
| Gate 13 Membership Reconfiguration | `067-06`, `067-11`, `067-19` | observer catch-up, ready transition, promotion, removal, stale rejection, quorum-size proof |
| Gate 14 Structured Evidence Artifacts | `067-08`, `067-14`, `067-17`, `067-19` | equivocation, withholding, missing blob, wrong root, wrong route digest, stale member, split brain |
| Gate 15 Report Honesty | `067-18`, `067-19` | no term is reported as live unless backed by runtime proof; unsupported terms are downgraded or removed |

### Hard Blockers And Concrete Add List

| Verdict source | Owning plans | Required closure |
| --- | --- | --- |
| Hard blocker: validator rejects bad QC | `067-07`, `067-16`, `067-19` | runtime rejection tests, not docs-only claims |
| Hard blocker: Celestia-local negatives | `067-16` | executable wrong namespace/blob/anchor/QC/payload cases |
| Hard blocker: network cannot create votes | `067-08`, `067-14` | transport-delivered messages require replay and signature verification |
| Hard blocker: BFT profile cannot be 2-of-3 CFT | `067-09`, `067-15` | invalid committee and quorum tests |
| Hard blocker: devnet/process evidence | `067-13`, `067-19` | local process or deterministic task isolation with fault evidence |
| Hard blocker: report honesty | `067-18`, `067-19` | generated claim registry and anti-placeholder audit |
| Concrete add list for `067-07` | `067-07`, `067-16`, `067-19` | validator/DA certificate binding and rejection cases |
| Concrete add list for `067-08` | `067-08`, `067-14`, `067-17` | signature seam, transport seam, evidence records |
| Concrete add list for `067-09` | `067-09`, `067-15`, `067-16` | BFT math, HotStuff-like local backend, Celestia-local adapter |
| Multi-process/devnet add list | `067-10`, `067-13`, `067-19` | runnable commands and local process/devnet proof |
| Planner HA or claim-removal add list | `067-12`, `067-18` | implemented HA or downgraded/removed live claim |

### Must-Solve Scope

The `067-verdict.md` MUST-solve list is fully in-phase. No item may move to a
later phase unless it is explicitly marked `live-claim-removed` and the final
claim registry proves the user-facing claim was removed. Closure owners are:
`067-04` for local QC, `067-06` for membership redistribution, `067-08` for the
signature seam, `067-11` for durable recovery, `067-12` for planner authority,
`067-13` for devnet/process realism, `067-14` for network faults, `067-15` for
HotStuff-like local semantics, `067-16` for Celestia-local artifacts, `067-17`
for structured evidence, `067-18` for glossary/report honesty, and `067-19` for
the integrated final Local-Conformance-Simulation gate.

### Exact Verdict Trace Addendum

The following rows preserve exact verdict bullets that are easy to lose when the
plans summarize them. These rows are traceability only; they do not create a new
code owner or a parallel implementation layer.

| Verdict exact item | Owning plans | Required planning interpretation |
| --- | --- | --- |
| `automatic online rebalance of all shards when a node joins` | `067-06`, `067-11`, `067-19` | not live by default; must be implemented as generation-bound membership reconfiguration or left as a removed live claim |
| `real network dissemination of placement updates` | `067-08`, `067-13`, `067-14`, `067-19` | not proven by config strings; must go through transport/process evidence or be downgraded |
| `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml` | `067-05`, `067-10`, `067-13`, `067-19` | exact storage config path for the canonical local topology profile |
| `five configured aggregator identities` | `067-05`, `067-10`, `067-13`, `067-19` | exact `SIM-5A7S` fact; does not imply BFT or live process execution by itself |
| `no non-repudiation` | `067-08`, `067-10`, `067-18` | deterministic simulator signatures remain simulation-only unless real cryptographic signing lands |
| `no private-key ownership` | `067-08`, `067-10`, `067-18` | production signature claims require a signer/verifier seam and key-backed verification |
| `no peer-authenticated operator identity` | `067-08`, `067-14`, `067-18` | network simulation must not claim production peer authentication without exercised cryptographic identity checks |
| `no real network adversary resistance` | `067-08`, `067-14`, `067-18` | local transport faults prove conformance, not public-network security |
| `Term or capability` table | `067-18`, `067-19` | every row becomes a claim-registry row with owner, artifact/API, tests, and claim level |
| `Slashing/economics` | `067-17`, `067-18` | evidence format only; production slashing remains a forbidden overclaim |
| `any trust-primary path that bypasses quorum` | `067-07`, `067-16`, `067-19` | validator and DA acceptance must reject no-QC or primary-only paths |

Dependency and tool trace from `067-verdict.md`:

- `Already Present Or Already Reused`: `z00z_aggregators`,
  `z00z_rollup_node`, `z00z_validators`, `z00z_simulator`, `z00z_crypto`,
  `z00z_storage`, `tokio`, `serde`, `serde_json`, `thiserror`, `sha2`, `hex`,
  and `tempfile` stay preferred before adding parallel primitives.
- Direct addition candidates: `redb`, `object_store`, `ed25519-dalek`, `bytes`,
  `borsh` or a project-owned canonical binary codec, `tracing`,
  `tracing-subscriber`, `metrics`, `prometheus`, and `proptest` may be added
  only to the phase-owning crate that exercises the API in tests.
- External-backend candidates: `hotstuff_rs`, `libp2p`, `celestia-client`,
  `celestia-rpc`, `celestia-types`, `blsful`, `reed-solomon-erasure`, and
  `reed-solomon-simd` remain gated behind local conformance proof; `openraft`
  is allowed only for a trusted internal CFT operator cluster and never as
  independent public aggregator BFT.
- `Non-Rust Harness Tools`: Docker or Docker Compose and existing repository
  scripts may be used for local multi-process/devnet evidence, but they must
  read checked-in topology manifests instead of inventing a second topology.
- Concrete add anchors from the verdict include
  `crates/z00z_runtime/validators/src/checkpoint.rs`,
  `crates/z00z_runtime/aggregators/src/signature.rs`,
  `crates/z00z_runtime/aggregators/src/lib.rs`, `tracing`,
  `tracing-subscriber`, `proptest`, `reed-solomon-erasure`, and
  `reed-solomon-simd`; missing files are proposed artifacts only until created
  by their owning plans.
- `Pro-Con And Doublecheck Audit`: keep the verdict decision that local
  conformance simulation is valid only for artifact/API-backed
  `simulated-full` claims while production external-infra overclaims remain
  forbidden.
- `crates/z00z_runtime/validators/src/checkpoint.rs`: keep
  checkpoint/publication binding aligned with the same subject digest.
- `crates/z00z_runtime/aggregators/src/lib.rs`: expose only the approved BFT
  committee/backend or signature/transport/evidence seams needed by tests and
  later adapters.
- `proptest`: committee-threshold, quorum-intersection, invalid-certificate,
  invalid-blob, restart-point, signature, evidence, and transport replay
  properties are direct test obligations when the relevant API lands.
</verdict_coverage_matrix>

<threat_model>
## 🔐 Threat Model And Trust Boundaries

Security-critical assets for this packet:

- package digests and route digests;
- placement and membership digests;
- recovery lineage, root generation, and proof metadata;
- commit-subject, vote, and certificate digests;
- publication bindings, theorem digests, and validator verdicts;
- payload bytes and any later blob commitments or settlement anchors.

Packet adversaries and fault sources:

- stale or unready secondary aggregators;
- removed members or mixed-generation voters;
- equivocal voters or conflicting same-term subjects;
- crashed primaries before or after local quorum;
- drifted route tables, plan digests, state roots, proof versions, or policy generations;
- detached or mismatched publication, theorem, certificate, or blob bindings;
- transport delay, reorder, partition, offline-member, restart, or external-DA outage conditions.

Trust boundaries that must remain explicit:

- wallet-style package ingress normalization;
- route-table and placement-generation ownership;
- recovery record and lineage truth;
- local DA publish or resolve boundaries;
- validator checkpoint and theorem verification boundaries;
- external transport and external DA provider boundaries, which may be simulated
  but may not replace real local runtime primitives.

Fail-closed review rule:

- Any plan that allows votes, certificates, publications, or validator
  acceptance across these boundaries without explicit digest, membership,
  lineage, and state checks is invalid.
</threat_model>

<source_corpus>
## 📚 Source Corpus

- `.planning/phases/067-Sharded-Concensus/067-TODO.md`
  - authoritative Phase 067 spec, implementation phases, failure semantics,
    local DA model, theorem requirements, and addendum constraints.
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
  - normative strong Local-Conformance-Simulation verdict for this review pass,
    including direct-answer bullet families, Strong Gates, hard blockers,
    dependency guidance, concrete add lists, and MUST-solve scope.
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
  - coverage evidence and current phase status referenced by `067-verdict.md`;
    evidence only, not a second authority layer.
- `.planning/STATE.md`
  - roadmap state snapshot referenced by `067-verdict.md`; status evidence only,
    not a second authority layer.
- `.planning/phases/090-New-Scenarios/90-TODO.md`
  - `scenario_11` target, artifact list, unit/integration/E2E tests, fault
    matrix, and verification anchors.
  - Section `15` scenario_11 lock: range `1050-1290`, `15` H3 sections,
    `81` dash bullets, and `13` numbered items. Exact fault rows including
    `Duplicate voter` and exact E2E paths including `one-secondary-stale` must
    stay reflected in the numbered plans.
- `crates/z00z_runtime/aggregators/README.md`
  - live doc path named by the rename matrix in `067-TODO.md`.
- `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`
  - exact section, bullet, and corpus traceability evidence for the Phase 067
    packet, including `067-TODO.md` and `067-verdict.md`; evidence only, not a
    second authority layer.
- `.planning/phases/067-Sharded-Concensus/067-VERDICT-ITEM-AUDIT.md`
  - item-level traceability for all `253` dash or numbered items from
    `067-verdict.md`; evidence only, not a second task ledger.
- `.planning/phases/067-Sharded-Concensus/wiki -results.md`
  - supporting codebase Q&A; non-canonical.
- `.github/copilot-instructions.md`
  - English-only artifacts, protected Tari vendor rule, tone signal, naming,
    verification, and compact-output policy.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
  - one-source-of-truth, trait injection, domain separation, vendor isolation,
    concurrency, error handling, API design, and NASA-style rules.
- `.github/instructions/rust.instructions.md`
  - Rust testing, module, documentation, error-handling, and naming rules.
</source_corpus>

<count_answer>
## 🔢 Count Answer

- Unique `TASK-NNN` identifiers in `067-TODO.md` plus `067-verdict.md`: `0`
- Base required GSD Plan Groups: `9`
- Required verdict Local-Conformance-Simulation groups: `10`
- Total required GSD Plan Groups: `19`
- Base required group source: `067-TODO.md` Sections `14.1` through `14.9`
- Verdict required group source: `067-verdict.md` Strong Acceptance Gate,
  Concrete Add List, and MUST-solve list
- Coverage rule: each required group maps to exactly one `067-NN-PLAN.md`
- Duplicate group status: none at planning start
- Missing group status: none at planning start
- Planning fail condition: any missing group, duplicate mapping, dropped
  acceptance clause, or unbound verification anchor is phase-failing.
- Exact structure lock: `19` H2 sections, `55` H3 sections, `447` dash-list
  bullets, and `80` numbered-list items are traced in
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`.
- Exact verdict lock: `9` H2 sections, `44` H3 sections, `227` dash-list
  bullets, `26` numbered-list items, and all `253` verdict dash or numbered
  items are traced in
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`.
- Exact verdict item audit:
  `.planning/phases/067-Sharded-Concensus/067-VERDICT-ITEM-AUDIT.md` contains
  `253/253` item-level rows with source line, item kind, source section,
  context owner, and plan owners.
</count_answer>

<section_lock>
## 🧷 Strict TODO Section Lock

- `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md` records exact
  section-to-owner traceability for the full
  `.planning/phases/067-Sharded-Concensus/067-TODO.md` packet.
- The packet must preserve:
  - all direct-answer sections in `3.*`;
  - all live-model and `sim_5a7s` truth in `4.*` and `5`;
  - the Mermaid flow and lifecycle semantics in `6.*`;
  - all current-vs-external boundary rules in `7.*` and `8.*`;
  - the full target model and failure semantics in `9.*` and `10.*`;
  - the DA, theorem, and simulation contracts in `11` through `13`;
  - the exact phase mapping in `14.*`;
  - the rotation discipline, doublecheck matrix, source evidence map, bottom
    line, and addendum rules in `15` through `19`.
- Any future edit that changes the TODO section inventory, bullet counts, or
  source corpus must update
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md` before
  execution continues.
</section_lock>

<required_groups>
## 📌 Required GSD Plan Groups

| Required group | Source section | Planned packet | Purpose |
| --- | --- | --- | --- |
| `PHASE-0` | `14.1` | `067-01-PLAN.md` | Terminology and boundary cleanup |
| `PHASE-1` | `14.2` | `067-02-PLAN.md` | Commit subject and certificate types |
| `PHASE-2` | `14.3` | `067-03-PLAN.md` | Secondary replay verifier |
| `PHASE-3` | `14.4` | `067-04-PLAN.md` | Local quorum certificate integration |
| `PHASE-4` | `14.5` | `067-05-PLAN.md` | End-to-end `sim_5a7s` harness |
| `PHASE-5` | `14.6` | `067-06-PLAN.md` | Join, removal, and rotation simulation |
| `PHASE-6` | `14.7` | `067-07-PLAN.md` | Validator and theorem binding |
| `PHASE-7` | `14.8` | `067-08-PLAN.md` | Network and signature adapter |
| `PHASE-8` | `14.9` | `067-09-PLAN.md` | BFT and Celestia local backend |
| `VERDICT-LCS-01` | `067-verdict.md` Strong Acceptance Gate and dependency add list | `067-10-PLAN.md` | Dependency and runnable aggregator readiness |
| `VERDICT-LCS-02` | `067-verdict.md` Gate 12 and durable evidence blocker | `067-11-PLAN.md` | Durable consensus evidence store |
| `VERDICT-LCS-03` | `067-verdict.md` Gate 11 | `067-12-PLAN.md` | Planner authority and failover claim boundary |
| `VERDICT-LCS-04` | `067-verdict.md` Gate 10 | `067-13-PLAN.md` | Multi-process devnet harness |
| `VERDICT-LCS-05` | `067-verdict.md` Gate 6 | `067-14-PLAN.md` | Network fault matrix and quorum transport conformance |
| `VERDICT-LCS-06` | `067-verdict.md` Gates 7 and 8 | `067-15-PLAN.md` | HotStuff-like local backend contract |
| `VERDICT-LCS-07` | `067-verdict.md` Gates 4 and 5 | `067-16-PLAN.md` | Celestia-local artifact conformance |
| `VERDICT-LCS-08` | `067-verdict.md` Gate 14 | `067-17-PLAN.md` | Structured evidence registry |
| `VERDICT-LCS-09` | `067-verdict.md` Gates 1 and 15 | `067-18-PLAN.md` | Glossary claim registry and report honesty |
| `VERDICT-LCS-10` | `067-verdict.md` Gate 2 and hard blockers | `067-19-PLAN.md` | Final local conformance simulation gate |
</required_groups>

<pre_plan_blockers>
## ⚠️ Pre-Plan Blockers

- `067-CONTEXT.md` did not exist before this planning pass.
- `067-TODO.md` contains no literal `TASK-NNN` rows; the plan corpus must use
  required group ids instead of fabricating a task ledger.
- Legacy non-canonical aggregator-consensus references still exist in older
  review text; planning must not recreate them as a second authority layer.
- `scenario_11` verification commands are normative only after the target and
  tests exist; each relevant plan must create those targets before claiming the
  commands are runnable.
- Live code, config, tests, and docs still expose `standby` naming in active
  paths, so Phase 067 must start with a breaking terminology cleanup.
</pre_plan_blockers>

<artifact_contract>
## 🧪 Artifact/Test/Result Proof Contract

Every numbered plan must include:

- `plan_id`
- `task_ids`
- copied source rows from the controlling section
- exact `source_refs`
- concrete `inputs`
- concrete `outputs`
- explicit `dependencies`
- executable `acceptance_tests`
- explicit `simulation_gate`
- explicit `negative_tests`
- `plan_artifacts`
- `plan_tests`
- `plan_results`
- per-group `task_artifacts`
- per-group `task_tests`
- per-group `task_results`
- `anti_placeholder_gate`
- `current_code_refs`
- `blockers`
- `evidence_gate`
- `not_recommendation_gate`

Per-group implementation depth must be one of:

- `full`
- `simulated-full`
- `live-claim-removed`
</artifact_contract>

<current_code_refs>
## 🔍 Current Code Evidence Anchors

### Aggregator Runtime
- `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`
- `crates/z00z_runtime/aggregators/src/commit_subject.rs`
- `crates/z00z_runtime/aggregators/src/secondary_replay.rs`
- `crates/z00z_runtime/aggregators/src/placement.rs`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_runtime/aggregators/src/ingress.rs`
- `crates/z00z_runtime/aggregators/src/recovery.rs`
- `crates/z00z_runtime/aggregators/src/dist_sim.rs`
- `crates/z00z_runtime/aggregators/src/dist_dispatch.rs`
- `crates/z00z_runtime/aggregators/src/dist_scheduler.rs`
- `crates/z00z_runtime/aggregators/src/shard_vote.rs`
- `crates/z00z_runtime/aggregators/src/shard_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/src/signature.rs`
- `crates/z00z_runtime/aggregators/src/transport.rs`
- `crates/z00z_runtime/aggregators/src/evidence.rs`
- `crates/z00z_runtime/aggregators/src/bft_committee.rs`
- `crates/z00z_runtime/aggregators/src/bft_engine.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`

### Local DA And Validator Boundaries
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/src/celestia_local.rs`
- `crates/z00z_runtime/validators/src/checkpoint.rs`
- `crates/z00z_runtime/validators/src/engine.rs`
- `crates/z00z_runtime/validators/src/verdict.rs`

### Simulator And Topology
- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`
- `config/hjmt_runtime/sim_7a7s/manifest.json`
- `config/hjmt_runtime/sim_7a7s/planner/planner-config.yaml`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/src/scenario_11/report.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_rollup_node/src/config.rs`

### Current Test Anchors
- `crates/z00z_runtime/aggregators/tests/test_commit_subject.rs`
- `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_dist_journal.rs`
- `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
- `crates/z00z_runtime/aggregators/tests/test_local_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
- `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs`
- `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs`
- `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
- `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs`
- `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
- `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

### Current Live Gaps
- `067-10` now owns a live `z00z_rollup_node` binary and the canonical manifest command head is `cargo run --release -p z00z_rollup_node --`; later slices must preserve that runnable contract before any devnet or process-backed claim closes.
- The following late-slice artifacts are still absent and remain plan-owned targets only until created: `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs`, `crates/z00z_runtime/aggregators/tests/test_transport_fault_matrix.rs`, `crates/z00z_runtime/aggregators/tests/test_hotstuff_local_backend.rs`, `crates/z00z_runtime/aggregators/tests/test_structured_evidence_registry.rs`, `crates/z00z_rollup_node/tests/test_hjmt_process_devnet.rs`, `scripts/hjmt_local_devnet.sh`, `docker/compose.hjmt-local.yaml`, `scripts/audit/audit_067_claims.py`, `.planning/phases/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md`, `.planning/phases/067-Sharded-Concensus/067-CLAIM-AUDIT.md`, and `.planning/phases/067-Sharded-Concensus/067-FINAL-CONFORMANCE.md`.
</current_code_refs>

<plan_waves>
## 🌊 Plan Waves

Plan ids are file numbers. Wave numbers are execution order only. The verdict
expansion must remain ten standalone files: `067-10-PLAN.md` through
`067-19-PLAN.md`. In particular, `067-12-PLAN.md`, `067-13-PLAN.md`,
`067-14-PLAN.md`, and `067-15-PLAN.md` are required files and must not be
collapsed into neighboring plans.

- Wave 1: `067-01` through `067-03`
  - terminology cleanup
  - first-class quorum artifacts
  - replay verifier
- Wave 2: `067-04` through `067-05`
  - certificate-producing commit integration
  - independent `scenario_11`
- Wave 3: `067-06` through `067-07`
  - join/removal/rotation and failover matrix
  - validator/theorem/local-DA certificate binding
- Wave 4: `067-08`
  - real signature seam and transport trait
- Wave 5: `067-09`
  - local simulated BFT/Celestia backend behind the proven subject interface
- Wave 6: `067-10`
  - dependency ownership, Cargo graph truth, and runnable aggregator command
- Wave 7: `067-11`
  - durable consensus evidence and restart recovery
- Wave 8: `067-12`
  - deterministic replicated planner authority or live-claim removal
- Wave 9: `067-13`
  - local multi-process/devnet harness
- Wave 10: `067-14`
  - deterministic transport fault matrix
- Wave 11: `067-15`
  - HotStuff-like local backend behind the subject seam
- Wave 12: `067-16`
  - Celestia-local artifact/API conformance
- Wave 13: `067-17`
  - structured evidence registry
- Wave 14: `067-18`
  - glossary claim registry and report honesty
- Wave 15: `067-19`
  - final integrated Local-Conformance-Simulation gate
</plan_waves>

<task_inventory>
## 🗂️ Canonical Task Inventory

- `PHASE-0`: prevent concept drift and rename live `standby` debt.
- `PHASE-1`: make commit subjects and certificates first-class artifacts.
- `PHASE-2`: make secondary votes meaningful through independent replay.
- `PHASE-3`: connect the current consensus seam to the new artifact model.
- `PHASE-4`: prove the end-to-end local package-to-validator path in
  `sim_5a7s`.
- `PHASE-5`: make join/removal/rotation and takeover safe under the new
  certificate model.
- `PHASE-6`: require the quorum artifact in local DA publication and validator
  acceptance.
- `PHASE-7`: add real signature and transport seams without bypassing local
  replay.
- `PHASE-8`: add simulated network-BFT and Celestia-style local backends behind
  the already-proven subject interface.
- `VERDICT-LCS-01`: make dependency installation and process commands executable.
- `VERDICT-LCS-02`: persist and recover consensus evidence.
- `VERDICT-LCS-03`: settle planner authority and planner HA claim boundary.
- `VERDICT-LCS-04`: prove local process/devnet realism.
- `VERDICT-LCS-05`: prove transport fault conformance without vote injection.
- `VERDICT-LCS-06`: implement HotStuff-like local backend semantics.
- `VERDICT-LCS-07`: implement Celestia-local artifact/API conformance.
- `VERDICT-LCS-08`: emit structured evidence for all safety faults.
- `VERDICT-LCS-09`: enforce glossary claim and report honesty.
- `VERDICT-LCS-10`: run final integrated local conformance gate.
</task_inventory>

<simulation_register>
## 🔬 Local Full-System Simulation Closure Register

Local deterministic simulation is mandatory for:

- replication and quorum formation;
- conflict detection and same-term freeze;
- secondary catch-up and stale-secondary rejection;
- route rollout and dispatch ownership;
- membership join/removal and planned rotation;
- restart and publication resume after crash;
- partition, heal, and offline-member behavior;
- divergent roots, lineage drift, and digest drift;
- local DA binding, resolve, and validator theorem alignment;
- process/devnet start, kill, restart, and partition smoke evidence;
- deterministic planner recomputation and planner-claim boundary;
- local BFT/HotStuff-like view, leader, and quorum behavior;
- Celestia-local namespace, blob, commitment, anchor, and degraded-mode behavior;
- structured fault evidence and honest scenario reporting;
- glossary claim registry and final report honesty.

The only allowed simulated boundaries are:

- external transport;
- remote process boundary;
- external DA transport;
- wall-clock or fault scheduler;
- unavailable third-party network.
</simulation_register>

<verification_checklist>
## ✅ Verification Checklist

Each numbered plan verify block must require:

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
2. relevant targeted `cargo test` commands
3. `cargo test --release` when Rust or test-affecting changes are relevant
4. `/GSD-Review-Tasks-Execution` at least 3 times in YOLO mode, or an explicit
   workspace-first fallback note if the runner is unavailable
5. nested use of `doublecheck`, smart tests, spec-to-code compliance, and Z00Z
   verification gates where relevant

Planning is invalid if any required group lacks inputs, outputs, artifacts,
tests, results, acceptance tests, negative tests, simulation gate, or evidence
gate.
</verification_checklist>
