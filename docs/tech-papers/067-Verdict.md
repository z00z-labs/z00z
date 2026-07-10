# 🎯 067 Verdict

<!-- markdownlint-disable MD060 -->

[TOC]

`067-TODO.md` describes a staged local-conformance buildout rather than a
production BFT network: first a live local CFT quorum pipeline, then validator
binding, then signature or transport seams, then a simulated-full
BFT/Celestia layer. The current branch packet now includes
`067-01-PLAN.md` through `067-21-PLAN.md`: `067-01` through `067-19` remain
the original required group mapping, while `067-20` and `067-21` are late
addendum closure plans that reopen the final gate for certificate-bound resume,
packet-truth reconciliation, and final conformance closeout.

## 🔎 Ответы по твоим пунктам

- RAID-like логика здесь не RAID в storage-смысле, а shard committee placement: каждый shard имеет `primary aggregator + ready secondary aggregators`. Новый агрегатор проходит `observer -> catch-up -> ready secondary -> voting member -> possible primary`, только на checkpoint/generation boundary. Упавший primary до quorum не публикует ничего; после quorum, но до DA, ready secondary может продолжить только тот же subject/certificate. Перераспределение shard ownership не случайное и не per-batch: оно идёт через route/placement generation, а mixed-generation commits должны reject.

- `SIM-5A7S` — canonical local HJMT/quorum profile для текущей фазы, но не глобальный default для всех тестов. Конфиги: manifest.json, planner-config.yaml, агрегаторы в `config/hjmt_runtime/sim_5a7s/aggregators/agg-*/aggregator-config.yaml`, например aggregator-config.yaml. Там 5 aggregators, 7 shards, per-shard quorum `2 of 3`.

- Consensus сейчас реально CFT: majority `floor(n / 2) + 1`, для `sim_5a7s` это `2-of-3` на shard. BFT ещё не production. 067-09-PLAN.md требует simulated-full BFT with `3f+1` / `2f+1`, но `hotstuff_rs`, `libp2p`, `celestia-*`, `ed25519-dalek` сейчас не установлены в workspace. Если под “BDR” имелся в виду `redb`: `redb` есть в storage/wallet crates, но не как durable consensus store в `z00z_aggregators`.

- `067-08` закрыл production-facing signature seam как runtime-owned signer/verifier trait с deterministic local signer implementation. Это всё ещё не operator-key production security: текущая реализация остаётся доменно-разделённым local deterministic signer path без private-key ownership, peer-authenticated operator identity, или real key-rotation story.

- Local quorum certificate уже live: `CommitSubject`, `ShardVote`, `ShardQuorumCertificate`, `SecondaryReplayVerifier`, `ReplayVerifiedVoteService`, in-memory vote transport, и structured equivocation/payload-withholding evidence есть и проходят targeted release tests. Failover/local lifecycle тоже частично live через `scenario_11`. Но HotStuff, real network transport, operator-key production signatures, real Celestia finality — ещё не live. Все glossary terms не должны магически стать production code в 067; правильная классификация: `live`, `simulated-full`, или `live-claim-removed`.

- Несколько aggregators теперь не только описаны как отдельные process identities в manifest.json, но и проходят локальный process-backed smoke path через `067-13`: `process_model: os_process`, `listen_addr`, `start_cmd`, отдельные data/log dirs, persisted restart state, и fail-closed stale-dir rejection теперь подтверждены release tests и `scripts/hjmt_local_devnet.sh`. Это всё ещё честно ограничено локальным `simulated-full` process/devnet claim: real network transport, HotStuff, и real Celestia finality остаются поздними verdict lanes.

- Planner сейчас честно закрыт через `067-12` как deterministic replicated planner config + route table, а не отдельная failover-кластерная роль с primary/secondary planners. Aggregator secondary replay пересчитывает через planner logic, planner drift reject покрыт, а `planner HA` оставлен как `live-claim-removed` до отдельного tested service model.

## 🎯 Targeted Additions To Close The Missing Subquestions

### 1. RAID-Like Redistribution Is Placement-Generation Reassignment, Not Automatic RAID Rebuild

The architecture described by `067-TODO.md` is RAID-like only in the narrow sense
that every shard has a primary plus secondary aggregators, and a quorum can keep
the shard progressing when one member is unavailable. It is not RAID parity,
automatic storage striping, or automatic data reconstruction across all
aggregators.

The live redistribution model is:

1. Routing maps package/work digest to exactly one shard.
2. Placement maps that shard to one primary aggregator plus ready secondary
   aggregators.
3. The membership digest is recalculated from the active placement set for that
   shard and generation.
4. New aggregators start as observers, catch up to route table, placement
   generation, journal lineage, and root state, then become ready secondaries at
   a generation/checkpoint boundary.
5. A ready secondary may become primary only through planned rotation or lawful
   emergency takeover.
6. Removed aggregators must disappear from later membership digests and cannot
   vote or take over.

What is not live yet:

- automatic online rebalance of all shards when a node joins;
- a production placement-controller daemon that computes a new placement plan;
- durable consensus metadata store for headers/votes/certificates in the
  aggregator crate;
- real network dissemination of placement updates.

For Phase 067 to be production-approximate, redistribution must be proven with
tests that mutate placement generation and show: old members fail, new unready
members fail, ready members pass, mixed-generation certificates fail, and planned
rotation or emergency takeover preserves subject digest and lineage.

### 2. Exact `SIM-5A7S` Meaning And Config Ownership

`SIM-5A7S` is the canonical Phase 067 local topology profile, not the global
default for every repository test. It is the default evidence profile for the
Phase 067 shard-quorum discussion because it provides enough structure to test
multi-shard ownership and per-shard 2-of-3 quorum.

The current `SIM-5A7S` facts are:

- profile home: `config/hjmt_runtime/sim_5a7s/`;
- manifest: `config/hjmt_runtime/sim_5a7s/manifest.json`;
- planner config: `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml`;
- aggregator configs: `config/hjmt_runtime/sim_5a7s/aggregators/agg-*/aggregator-config.yaml`;
- storage config: `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml`;
- route table: `config/hjmt_runtime/sim_5a7s/shard_route_tables/route-table-v1.canon.hex`;
- five configured aggregator identities;
- seven configured shard identities;
- every shard has one primary and two secondary aggregators;
- local quorum is 2-of-3 per shard, not 5-of-5 globally.

The manifest also records `process_model: os_process` and per-aggregator
`listen_addr`, `start_cmd`, and `restart_cmd`. That is config-level evidence for
separate process identities. It is not, by itself, proof that the current
`scenario_11` test launches five operating-system processes and exercises real
socket traffic.

### 3. CFT, BFT, HotStuff, And The `BDR`/`redb` Clarification

The currently proven consensus behavior is CFT-style local quorum:

```text
quorum = floor(member_count / 2) + 1
```

For the current 3-member per-shard committee, this tolerates one crash or one
offline member. It does not tolerate a Byzantine member. A 3-member committee
cannot honestly claim one-fault BFT because BFT requires:

```text
n >= 3f + 1
quorum >= 2f + 1
```

The repository currently does not install `hotstuff_rs`, `libp2p`,
`celestia-client`, `celestia-rpc`, `celestia-types`, `ed25519-dalek`,
`openraft`, `object_store`, or `blsful` in the workspace dependency graph.
`redb` exists in storage/wallet dependency scopes, but not as a dedicated
aggregator consensus metadata store. If `BDR` means a separate Byzantine or
durable replication library, there is no such installed library in the live
Phase 067 aggregator path.

The correct Phase 067 closure is therefore:

- local CFT quorum is live-code evidence;
- BFT is a simulated-full target for `067-09`;
- HotStuff is a later backend candidate behind the proven subject interface;
- real Celestia is an external DA backend candidate, not a current ordering or
  execution correctness engine;
- any final BFT claim must be backed by 3f+1/2f+1 tests, not by renaming CFT.

### 4. Deterministic Simulator Signature Versus Production Signature

The deterministic simulator signature proves byte binding, not operator
authenticity.

It currently means:

- hash the unsigned vote fields with a local signature domain;
- produce reproducible bytes for tests and deterministic simulation;
- verify that vote fields were not mutated after construction;
- avoid private keys and external key management while local semantics are
  still being proven.

It does not mean:

- no non-repudiation;
- no private-key ownership;
- no peer-authenticated operator identity;
- no real network adversary resistance;
- no production key rotation or key revocation story.

A production signature must add a signer trait and a verifier trait, bind the
canonical vote bytes, verify against public keys from the active committee, and
emit equivocation evidence when one signer signs conflicting subjects for the
same shard, term, and membership digest.

### 5. What Will Work At End Of 067, And What Must Not Be Claimed

Expected end-of-067 closure should be classified like this:

| Term or capability         | End-of-067 acceptable state                                  | Forbidden overclaim                                       |
| -------------------------- | ------------------------------------------------------------ | --------------------------------------------------------- |
| Local quorum certificate   | Live code                                                    | Network BFT finality                                      |
| Secondary replay           | Live code                                                    | Passive byte-copy replication as correctness proof        |
| Failover/takeover          | Live or simulated-full over real recovery seams              | Arbitrary takeover across lineage/generation              |
| Validator certificate gate | Live code after `067-07`                                     | Validator acceptance without certificate binding          |
| Production signature seam  | Simulated-full or live trait seam after `067-08`             | Real operator security if only digest signatures exist    |
| Transport                  | In-memory or local multi-process proof after `067-08`        | Production P2P without libp2p or authenticated peer tests |
| BFT committee              | Simulated-full after `067-09`                                | Calling 2-of-3 CFT BFT                                    |
| HotStuff                   | Adapter boundary or local simulated backend only             | Installed production HotStuff consensus                   |
| Celestia                   | Local Celestia-compatible blob adapter after `067-09`        | Real Celestia finality or ordering                        |
| Slashing/economics         | Evidence format only                                         | Production slashing                                       |
| Glossary terms             | `live`, `simulated-full`, `deferred`, or `live-claim-removed` | Treating every glossary term as production code           |

This is the hard honesty rule: every glossary term must end Phase 067 with a
disposition. If the repository cannot prove it, the final artifact must remove
the live claim rather than carry it forward as a vague promise.

### 6. Multi-Aggregator Quorum Process Model

The current configs are written as if aggregators can be run as separate
processes. Each aggregator config has its own id, address, data directory,
journal path, log path, start command, and restart command. That is necessary
but not sufficient.

`067-13` closed the first required half of this question. The repository now
has a manifest-driven multi-process smoke harness that starts several local
aggregator processes from `SIM-5A7S`, proves distinct identity or port or
data-dir or log-dir ownership, exercises kill or restart against persisted
state, and emits machine-readable process evidence. That closes the
manifest-only gap.

What is still not claimed here is real network transport. `scenario_11`
continues to carry deterministic in-process transport semantics for partition
or heal or crash-report honesty, and the executable transport-fault matrix now
closes on `067-14`. The stronger production-approximation target is now
explicitly split across landed workstreams: `067-13` provides minimal
multi-process operator realism, `067-14` provides deterministic transport
exhaustiveness without bypassing replay or signature verification, `067-15`
closes the HotStuff-like local backend contract, `067-16` closes the
Celestia-local artifact contract, and `067-18` is now the next lane for the
glossary claim registry and report honesty closeout.

### 7. Planner Authority, Primary/Secondary Planner Question, And Failover

There is no live primary-planner/secondary-planner failover protocol today. The
planner truth is currently deterministic route-table configuration and planner
logic that aggregators can reuse. The planner config records mode `central`, but
the live quorum proof does not depend on a separate planner daemon electing a
leader.

`067-12` already chose one explicit model:

| Planner model                    | Required Phase 067 proof                                     |
| -------------------------------- | ------------------------------------------------------------ |
| Deterministic replicated planner | Every aggregator can recompute the same route/plan from the same config and route-table digest; config drift rejects; no planner primary/secondary role is claimed. |
| Planner service with failover    | Planner has its own primary/secondary identity, election or activation boundary, durable plan state, failover tests, and stale-plan rejection. |

The live repository now follows the first model. Planner truth is a
deterministic replicated function over canonical config plus route-table digest,
and config or route-table drift fails closed before replay or dispatch. If a
future phase wants a planner service, it must be designed as a separate
authority surface rather than smuggled into Phase 067.

### 8. Additional Questions That Must Be Solved In 067

These are the extra phase-exam questions that should be treated as implementation
coverage requirements, not as optional review commentary:

1. Can a stale route table produce a locally plausible but invalid certificate?
2. Can a certificate survive restart and still bind to the exact same subject?
3. Can a removed aggregator replay old local state and be rejected before vote
   counting?
4. Can a new aggregator catch up without becoming vote-eligible too early?
5. Can a primary crash after quorum but before DA be resumed without changing
   the certificate digest?
6. Can local DA resolve a batch whose certificate digest is missing, stale, or
   detached, and does validator reject it?
7. Can one voter sign or sim-sign two subjects in the same term, and is evidence
   emitted from the conflicting material rather than from a string label?
8. Can a transport adapter inject a vote that was never produced by replay?
9. Can a BFT profile reject below-3f+1 membership and below-2f+1 quorum?
10. Can a Celestia-compatible local adapter reject wrong namespace, wrong blob
    commitment, missing payload, and detached artifact while still resolving the
    same artifact shape as local DA?
11. Can every report artifact be regenerated from live execution instead of
    handwritten fixture rows?
12. Can the final report prove which terms are live, simulated-full, deferred,
    or removed as live claims?

If any of these answers stays unresolved, it should remain in Phase 067 as an
open blocker unless the final phase summary explicitly downgrades the related
claim.

### 9. Can Celestia, Network, And Devnet Be Fully Simulated Locally?

Yes, Phase 067 can make the Celestia, network, and devnet glossary terms
`simulated-full` locally if the implementation uses executable artifact/API
conformance adapters rather than placeholder mocks. The correct claim is:

```text
ALL glossary terms live-code locally = live | simulated-full | explicitly rejected overclaim
```

This means Phase 067 can close local simulation for external systems when each
external boundary has a real local adapter, stable artifacts, negative tests,
and validator-facing proof that the adapter preserves the same contract as the
future production backend.

Required local simulation shape:

- Celestia-compatible DA: add a local adapter that stores blob bytes, namespace,
  blob commitment, deterministic inclusion height or anchor, payload retention,
  unanchored-height state, degraded mode, and a resolve path that returns the
  same artifact contract as local DA.
- Network: add a deterministic transport adapter with peer identities, signed or
  signature-seam envelopes, delay, reorder, duplicate, drop, partition, heal,
  replay protection, and no path to inject votes before secondary replay.
- Devnet: add a local topology runner or harness that binds aggregators,
  planner config, ports, data dirs, restart/failover behavior, and optional
  Docker or multi-process execution to the same manifest-derived topology.
- BFT/HotStuff-like backend: add a local BFT-valid engine behind the proven
  commit-subject interface, with `3f+1` committee membership, `2f+1` quorum,
  leader/view-change artifacts, and equivocation evidence.
- Production-signature boundary: add signer/verifier traits plus deterministic
  local signer tests; do not claim production operator security unless real
  cryptographic key verification and key lifecycle are implemented.

What this still cannot honestly claim:

- real Celestia provider finality;
- real libp2p or public network behavior;
- installed HotStuff production consensus;
- public devnet operation;
- production slashing or economic finality.

**Minimum acceptance gate for local `simulated-full` closure:**

1. Validator rejects missing, detached, stale, or mismatched quorum certificate.
2. Celestia-local adapter rejects wrong namespace, wrong blob commitment,
   missing payload, stale anchor, and detached artifact.
3. Network simulator cannot create or count a vote that was not produced through
   secondary replay and signature verification.
4. BFT mode is impossible on the current 2-of-3 CFT profile; it requires local
   `3f+1` membership and `2f+1` quorum tests.
5. Devnet harness proves either real local multi-process execution or an
   explicitly scoped deterministic local transport boundary.
6. Report honesty tests classify every glossary term as `live`,
   `simulated-full`, `deferred`, or `live-claim-removed`.

So the answer is yes for local artifact/API simulation, no for pretending those
external systems are already production-installed. `067-09` now closes the
local proof-backed BFT/Celestia layer without external dependency overclaim,
while runnable devnet/process proof remains later-lane work.

## ✅ Pro-Con And Doublecheck Audit

This section records why the verdict keeps the local conformance-simulation
answer while rejecting production overclaim. It also records why duplicate tail
material was consolidated instead of preserved as a second answer block.

### Pro: Local Conformance Simulation Is A Valid Phase 067 Target

- `067-TODO.md` explicitly treats current behavior as local CFT and external
  BFT/Celestia as later layers behind the proven subject interface.
- `067-09-PLAN.md` requires local BFT-valid committee tests, a
  Celestia-compatible local binding path, and deterministic simulation when
  real third-party dependencies are unavailable.
- The current workspace already proves the lower layer: commit subject,
  secondary replay, shard quorum certificate, and `scenario_11` are live enough
  to serve as the contract that simulated external adapters must preserve.
- A local adapter can be meaningful if it stores real artifacts, exposes a real
  resolve/verify API, and has negative tests for detached or stale evidence.

### Con: Local Simulation Must Not Be Marketed As Production External Infra

- `hotstuff_rs`, `libp2p`, `celestia-*`, and production vote-signature
  dependencies are not currently installed in the live workspace dependency
  graph.
- Current `scenario_11` is deterministic in-process evidence, not proof of
  operating-system process scheduling, socket authentication, public network
  partitions, real Celestia provider behavior, or live devnet operations.
- The validator certificate gate is now live after `067-07`, so downstream
  acceptance can honestly be described as certificate-mandatory on the current
  local quorum path.
- If a final report says only `BFT`, `Celestia`, or `devnet` without the
  `local simulated-full` qualifier, the claim overstates the repository state.

### Decision

Keep the answer: Phase 067 can make every glossary term locally live-code only
if each term is classified as `live`, `simulated-full`, `deferred`, or
`live-claim-removed`, and if `simulated-full` means executable artifacts, APIs,
and negative tests. Do not claim real external infrastructure until actual
dependencies, process runners, and provider integrations exist.

### Consolidation Rule

Duplicate answer blocks after the main verdict should be merged into this single
source-of-truth section, not preserved as a second conclusion. The reason is not
cosmetic cleanup: repeated conclusion blocks make the claim level harder to
audit and can reintroduce stale local paths, stale line references, or markdown
diagnostics. The current retained file is consolidated, has a single claim-level
analysis path, and should keep any future duplicate material only after it is
reduced into this audited section.

## 🧷 Strong Acceptance Gate For 067

Phase 067 should be considered closed only if every glossary term from the
phase authority has one explicit claim state:

```text
full live-code | simulated-full live-code | explicitly rejected overclaim
```

No term may remain only `docs`, `design`, `TODO`, `stub`, `fixture-only`, or
`future maybe`. The reason is simple: Phase 067 is about reducing consensus
claim ambiguity. Leaving a glossary term without executable disposition would
recreate the same ambiguity under a cleaner heading.

### Gate 1: Glossary Traceability

For every glossary term, the phase closeout must contain one row or structured
record with:

```text
term -> code owner -> artifact/API -> positive test -> negative test -> claim level
```

If a term is only explained in Markdown, the gate fails. If a term is not
intended to land in Phase 067, the term must be marked `live-claim-removed` or
`deferred` with the exact reason.

### Gate 2: One Real End-To-End Path

At least one executable scenario must prove the full chain:

```text
wallet package -> ingress -> route -> plan -> primary exec -> secondary replay -> votes -> QC -> DA/Celestia-local -> resolve -> validator verdict
```

Disconnected unit tests are still useful, but they cannot close the phase by
themselves because the hard risk is boundary detachment: route, vote,
certificate, DA, theorem, and validator can each be locally correct while still
not binding the same subject.

### Gate 3: Validator Requires QC

The validator path must reject:

- missing quorum certificate;
- detached certificate;
- stale membership certificate;
- wrong subject digest;
- wrong publication/theorem binding;
- any trust-primary path that bypasses quorum.

Until this gate lands, the repository can prove local quorum formation but not
downstream certificate-required acceptance.

### Gate 4: Celestia-Local Is Artifact-Complete

The local Celestia-compatible adapter must have real local fields and API shape,
not just a provider name. Required local artifacts:

- namespace;
- blob bytes;
- blob commitment;
- deterministic local height;
- tx hash or local inclusion reference;
- payload retention policy;
- unanchored height policy;
- degraded or unavailable mode;
- resolve, retrieve, and verify API.

The adapter may be local-only, but the artifact contract must be strong enough
that a later real Celestia adapter can be substituted behind the same boundary.

### Gate 5: Celestia Negative Tests

The local Celestia adapter must reject:

- wrong namespace;
- wrong blob commitment;
- missing payload;
- stale anchor;
- mismatched quorum-certificate digest;
- exceeded unanchored-height limit;
- blob content that resolves to an artifact the validator would reject.

This is the difference between conformance simulation and a fake provider.

### Gate 6: Network Simulation Is Not Vote Injection

The transport simulator must model:

- peer identity;
- signed or signature-seam message envelope;
- delay, reorder, drop, and duplicate delivery;
- partition and heal;
- restart or reconnect;
- replay protection.

Critical rule: transport cannot create or count a vote unless the replay and
signature path produced or accepted that vote. Transport may deliver evidence;
it must not manufacture consensus truth.

### Gate 7: BFT Claims Need BFT Math

Any BFT mode must require:

- `n >= 3f + 1`;
- `quorum >= 2f + 1`;
- below-`3f+1` membership rejects;
- below-`2f+1` quorum rejects;
- the current `2-of-3` profile remains explicitly CFT-only.

Renaming a `2-of-3` local majority certificate to BFT is a phase failure.

### Gate 8: HotStuff-Like Backend Behind The Seam

A local HotStuff-like backend may simulate rounds, leaders, view changes, and
backend QCs, but it must stay behind the already-proven commit-subject seam. It
cannot bypass:

- commit subject;
- membership digest;
- secondary replay;
- validator certificate gate.

The backend may decide ordering mechanics only after the local subject is valid
and available under the same artifact contract.

### Gate 9: Production Signature Seam

The simulator signature may remain for deterministic tests, but Phase 067 needs
a production-facing signer/verifier trait or API with tests for:

- wrong signer reject;
- wrong subject reject;
- wrong membership reject;
- replayed signature reject;
- equivocation evidence emitted from conflicting signed material.

This can still run locally, but the API must be shaped like real cryptographic
operator signing, not like a hard-coded fixture digest.

### Gate 10: Multi-Process Or Devnet Harness

The strong gate requires actual local process-level evidence or a Docker/local
task equivalent with:

- separate aggregator identities;
- separate ports;
- separate data directories;
- shared planner config or explicit planner authority;
- restart, kill, and partition tests;
- logs and artifacts per process.

If Phase 067 chooses not to implement this, it must explicitly mark production
process/devnet behavior as not claimed rather than relying on manifest-only
`process_model` fields.

### Gate 11: Planner HA Or Claim Removal

The central planner cannot remain a silent SPOF claim. Phase 067 must choose:

- implement planner primary/secondary failover with durable plan state and stale
  plan rejection; or
- mark planner HA as `live-claim-removed` and define planner truth as a
  deterministic replicated function over canonical route-table config.

The second option is the safer current path unless Phase 067 intentionally grows
a separate planner-service authority.

### Gate 12: Crash Recovery

After crash/restart, the system must recover or explicitly downgrade:

- votes;
- quorum certificate;
- publication state;
- DA/Celestia-local anchor;
- validator decision path.

In-memory-only evidence is insufficient for production approximation if the
final report claims restart or devnet behavior.

### Gate 13: Membership Reconfiguration

Join, removal, and rotation must stay checkpoint/generation-bound and tested:

- observer cannot vote;
- unready secondary cannot vote;
- removed aggregator cannot vote or take over;
- mixed-generation certificate rejects;
- primary rotation preserves lineage and membership digest.

This is the concrete version of the RAID-like redistribution claim.

### Gate 14: Structured Evidence Artifacts

Every safety-relevant fault must emit structured evidence, not only log strings:

- equivocation;
- payload withholding;
- missing blob;
- wrong root;
- wrong route digest;
- stale member;
- split brain.

String-only evidence is not enough because later validators, operators, and
report-honesty gates need reproducible artifacts.

### Gate 15: Report Honesty

The final Phase 067 report must explicitly mark each relevant term as:

- real implemented locally;
- simulated-full;
- not claimed;
- future external integration.

If a report says `BFT`, `Celestia`, or `devnet` without `local simulated-full`
qualification, the gate fails.

### Hard Blockers

Phase 067 must not close while any of these remain true:

- QC is not required by validator acceptance.
- Celestia-local adapter returns constant or detached artifacts.
- Network simulator can inject votes.
- BFT is claimed on `2-of-3`.
- Process model exists only in config, not a runnable harness or explicit
  non-claim.
- Planner failover is undocumented or untested while planner HA is still
  claimed.
- Any glossary term is only prose.

## 📦 Conformance Simulation Libraries And Frameworks

This list is about local conformance simulation first. It is not approval to
install every external backend immediately. Add a dependency only when it is the
smallest truthful way to prove the corresponding artifact/API contract.

### Already Present Or Already Reused

- `z00z_aggregators`: current owner for commit subject, votes, quorum
  certificate, placement, replay, and consensus adapter seams.
- `z00z_rollup_node`: current owner for local DA publish/resolve adapter and
  future Celestia-local adapter seam.
- `z00z_validators`: current owner for validator verdict and theorem boundary.
- `z00z_simulator`: current owner for `scenario_11` local conformance harness.
- `z00z_crypto`: preferred project-owned crypto/domain-separation surface before
  adding raw signature dependencies to public APIs.
- `z00z_storage`: storage/root/checkpoint/recovery primitives and existing redb
  experience.
- `tokio`: already used by simulator/wallet/rpc crates; needed for async local
  process/transport harnesses.
- `serde`, `serde_json`, `thiserror`, `sha2`, `hex`, `tempfile`: already used
  across the workspace for deterministic artifacts, errors, hashing, fixtures,
  and temp harnesses.

### Direct Additions For Strong Local Conformance

Items in this subsection may already exist somewhere else in the workspace or
lockfile. `Direct addition` means adding the dependency to the phase-owning
crate only when the local conformance owner needs that API directly.

- `redb`: add directly to the aggregator or future consensus crate only if Phase
  067 persists headers, votes, QCs, anchors, indexes, and restart recovery state.
- `object_store`: add when payload/blob/chunk/proof storage needs a backend-like
  local API instead of only in-memory maps.
- `ed25519-dalek`: add only behind a project signer/verifier trait when
  `z00z_crypto` does not already provide the needed vote-signature primitive.
- `bytes`: useful for payload chunks, transport envelopes, and blob bodies when
  network/object-store APIs become byte-buffer heavy.
- `borsh` or a project-owned canonical binary codec: needed if signing bytes or
  backend proposal bytes require deterministic non-JSON encoding. This must not
  replace existing project codec rules without an explicit seam.
- `tracing` and `tracing-subscriber`: local process/devnet harness logs and
  per-process evidence collection.
- `metrics` and `prometheus`: local backend metrics and alert assertions when
  the simulated devnet exposes operator-style observability.
- `proptest`: property tests for committee thresholds, quorum intersection,
  invalid certificate mutation, and restart-point replay.

### External-Backend Drivers After Local Semantics Pass

- `hotstuff_rs`: candidate HotStuff backend after the local commit-subject,
  replay, validator, and BFT math gates are proven.
- `libp2p`: candidate real transport after the in-memory transport proves that
  vote delivery cannot bypass replay/signature gates.
- `celestia-client`, `celestia-rpc`, `celestia-types`: real Celestia backend
  candidates after Celestia-local blob/namespace/commitment conformance passes.
- `blsful`: later compact aggregate-certificate candidate, not required for the
  Phase 067 MVP.
- `reed-solomon-erasure` or `reed-solomon-simd`: optional chunk recovery/erasure
  coding for payload availability tests.
- `openraft`: only for a trusted internal operator cluster. It must not be used
  as independent public aggregator consensus because Raft is CFT, not BFT.

### Non-Rust Harness Tools

- Docker or Docker Compose: useful for local multi-process/devnet runs with
  separate ports, data directories, kill/restart, and logs per aggregator.
- Existing repository scripts should own orchestration before adding a new
  framework. A future helper should start from the checked-in `sim_5a7s` or
  `sim_7a7s` manifest rather than inventing a parallel topology file.

## 🧰 Concrete Add List To Implement The Whole Verdict

This is the minimum direct add list for completing the verdict. It is split by
claim boundary because installing every candidate crate at once would create a
large dependency surface before the repository has proven the local API contract
that those dependencies must obey. Plan-listed artifacts not repeated here still
remain required by their plan; this section names the direct implementation
homes, tests, and tools that make the verdict enforceable.

### Add For `067-07`: Validator And DA Certificate Binding

- `crates/z00z_rollup_node/src/da.rs`: extend the local DA record/publish/resolve
  path so it carries or references the quorum-certificate digest.
- `crates/z00z_runtime/aggregators/src/types.rs`: add or adapt shared DTO fields
  only where the publication/validator bridge needs an aggregator-owned type.
- `crates/z00z_runtime/validators/src/verdict.rs`: extend `ResolvedBatch` or an
  adjacent validated binding type so validator logic can require certificate
  alignment with publication and theorem data.
- `crates/z00z_runtime/validators/src/engine.rs`: enforce the certificate-aware
  acceptance gate in the validator boundary.
- `crates/z00z_runtime/validators/src/checkpoint.rs`: keep checkpoint/publication
  binding aligned with the same subject digest.
- `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs`: add negative
  coverage for missing, detached, stale, and mismatched certificate bindings.
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`: keep the existing local
  DA simulation green while adding certificate-aware binding.
- `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`: keep theorem
  tamper coverage aligned with certificate-aware publication binding.
- `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`: keep the
  aggregator-side publication-binding seam aligned with the new certificate
  digest/reference.
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`: add
  validator-level reject coverage for trust-primary and no-QC acceptance paths.

Crate additions should not be needed for this slice unless a new canonical
encoding/helper is required. Prefer existing `z00z_aggregators`,
`z00z_rollup_node`, `z00z_validators`, `z00z_storage`, `z00z_utils`, and
`z00z_crypto` primitives.

### Add For `067-08`: Signature, Transport, And Evidence Seams

- `crates/z00z_runtime/aggregators/src/signature.rs`: signer/verifier trait,
  deterministic local signer, production-facing signature bytes, and wrong
  signer/subject/membership/replay rejects.
- `crates/z00z_runtime/aggregators/src/transport.rs`: in-memory vote transport
  with peer identity, delay, reorder, drop, duplicate, partition/heal, restart,
  and replay protection.
- `crates/z00z_runtime/aggregators/src/evidence.rs`: structured equivocation,
  payload-withholding, stale-member, missing-payload, wrong-root, and split-brain
  evidence records.
- `crates/z00z_runtime/aggregators/src/service.rs`: wire the transport and
  signature seams without allowing transport to create/count votes directly.
- `crates/z00z_runtime/aggregators/src/lib.rs`: export only the approved public
  signature, transport, and evidence seams; keep internals private.
- `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs`: verify
  signer and verifier behavior.
- `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs`: prove
  network simulation cannot bypass replay verification.
- `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`: prove
  conflicting same-voter signed material emits structured evidence.
- `crates/z00z_simulator/tests/test_scenario_11.rs`: keep scenario conformance
  green through the new signature/transport path.

Recommended direct crates only if existing project primitives or dependencies
are not enough:

- `ed25519-dalek`: real local vote/evidence signature primitive behind the
  project signer/verifier trait.
- `bytes`: transport envelopes, payload chunks, and blob bodies.
- `borsh` or a project-owned canonical binary codec: stable signing/proposal
  bytes when JSON is too loose for signatures.
- `tracing` and `tracing-subscriber`: per-peer and per-process evidence logs.
- `proptest`: mutation/property tests for signatures, evidence, and transport
  replay behavior.

Do not add `libp2p` here unless the in-memory transport already proves the
replay/signature gate. `libp2p` belongs after the semantic transport contract is
hard to bypass.

### Add For `067-09`: BFT And Celestia Local Backend

- `config/hjmt_runtime/sim_7a7s/manifest.json`: local BFT-valid topology with
  enough members to prove `3f+1` membership and `2f+1` quorum behavior.
- `config/hjmt_runtime/sim_7a7s/planner/planner-config.yaml`: planner config for
  the larger simulated committee.
- `config/hjmt_runtime/sim_7a7s/aggregators/agg-*/aggregator-config.yaml`:
  process identity, port, storage path, and lifecycle metadata for each local
  simulated aggregator.
- `crates/z00z_runtime/aggregators/src/bft_committee.rs`: BFT committee math,
  below-`3f+1` reject, below-`2f+1` reject, and CFT/BFT claim separation.
- `crates/z00z_runtime/aggregators/src/bft_engine.rs`: local BFT-like backend
  behind `CommitSubject`, membership digest, replay, and validator gate.
- `crates/z00z_runtime/aggregators/src/lib.rs`: expose only the approved BFT
  committee/backend types needed by tests and later adapters.
- `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`: prove
  the BFT math and reject CFT-as-BFT claims.
- `crates/z00z_rollup_node/src/celestia_local.rs`: Celestia-compatible local DA
  adapter with namespace, blob bytes, blob commitment, deterministic local
  height, tx hash/local inclusion ref, retention, unanchored-height policy,
  degraded mode, and resolve/retrieve/verify API.
- `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`: reject wrong
  namespace, wrong blob commitment, missing payload, stale anchor, mismatched QC
  digest, exceeded unanchored height, and blob content rejected by validator.
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`: prove the new local BFT
  topology remains manifest-backed and does not drift from checked-in config.
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`: keep
  validator publication/certificate acceptance aligned with the simulated BFT
  and Celestia-local path.
- `crates/z00z_simulator/tests/test_scenario_11.rs`: extend the scenario to
  exercise the BFT-local and Celestia-local adapters without claiming production
  external infrastructure.

Recommended direct crates for this slice:

- `redb`: durable local metadata store for headers, votes, QCs, anchors, indexes,
  and restart recovery. The current workspace has older redb versions in other
  crates; Phase 067 must choose and pin a direct version deliberately if it
  persists aggregator consensus state.
- `object_store`: backend-like blob/chunk/proof storage API for Celestia-local
  and payload retention tests.
- `proptest`: committee-threshold, quorum-intersection, invalid-certificate,
  invalid-blob, and restart-point property tests.
- `metrics` and `prometheus`: local backend metrics and alert assertions if the
  devnet harness exposes operator-facing observability.
- `tracing` and `tracing-subscriber`: structured run logs for local BFT/devnet
  and Celestia-local artifact flow.
- `reed-solomon-erasure` or `reed-solomon-simd`: optional only if Phase 067
  chooses to simulate chunk recovery and payload availability rather than simple
  blob retention.

External backend drivers stay optional until the local backend is proven:

- `hotstuff_rs`: real HotStuff candidate after `bft_engine.rs` proves the local
  backend contract.
- `libp2p`: real transport candidate after the local transport contract proves
  vote injection impossible.
- `celestia-client`, `celestia-rpc`, `celestia-types`: real Celestia candidates
  after `celestia_local.rs` proves the local blob contract.
- `arc-malachitebft-*`: optional alternative BFT backend after audit.
- `blsful`: later aggregate-certificate candidate, not required for local MVP.
- `openraft`: only for one trusted operator cluster; never for independent
  public aggregator BFT.

Metrics, alert, and runbook claims remain optional unless the final Phase 067
report mentions them. If mentioned, they require explicit artifacts and tests,
not only configuration keys.

### Add For Multi-Process Or Devnet Harness

- Docker or Docker Compose: isolate aggregator processes with separate ports,
  data dirs, logs, kill/restart, and partition tests.
- A repo-owned orchestration script, for example under `scripts/`, that reads
  `sim_5a7s` or `sim_7a7s` manifests and starts the declared aggregators instead
  of inventing a second topology file.
- JSON evidence validators using existing `serde_json` or shell/JQ-style checks
  to prove every run emitted route, subject, vote, QC, DA/Celestia-local, and
  validator artifacts.

This harness is necessary only if the final Phase 067 report wants to claim
devnet/process realism. Otherwise it must explicitly say process/devnet behavior
is not claimed.

### Add For Planner HA Or Remove The Claim

If Phase 067 claims planner HA, add:

- planner identity records;
- planner primary/secondary activation boundary;
- durable plan journal;
- stale-plan reject tests;
- failover tests that show aggregators reject mixed planner generations.

If Phase 067 does not implement those, do not add a planner HA library. Mark
planner HA as `live-claim-removed` and define planner truth as deterministic
recomputation from canonical route-table config.

## 🛑 MUST решить в 067, не переносить

- Validator certificate gate: DA/validator должны reject missing, detached, or
  mismatched certificate. Simulate and verify in `067-07-PLAN.md` rollup and
  validator tests.
- Production signature seam: add trait, deterministic local signer, and real
  verification boundary. Simulate and verify through `067-08-PLAN.md` signature
  tests.
- Equivocation evidence: same voter, same term, different subject must produce
  structured evidence. Simulate through in-memory transport and evidence tests.
- Multi-process realism: add a test/Docker/local-task harness that launches
  multiple aggregators, or explicitly prove why in-process transport is the
  phase boundary. Tie this to the `sim_5a7s` process model or an explicit local
  transport harness.
- Planner availability model: choose config-replicated deterministic planner or
  real primary/secondary planner failover. Prove the chosen model in the
  simulator fault matrix.
- BFT math: prove `3f+1` membership, `2f+1` quorum, and below-threshold reject
  with planned `sim_7a7s` or larger local committees.
- Celestia-local backend: implement local blob/namespace/commitment adapter that
  resolves the same artifact shape as local DA. Verify through `067-09-PLAN.md`.
- Durable consensus evidence: votes, certs, headers, DA bindings, and recovery
  must survive restart or be explicitly marked simulation-only. Use redb,
  object-store, or a documented local durable seam.
- Glossary disposition: every glossary term must be marked live,
  simulated-full, deferred, or forbidden-overclaim. Prove through phase guard
  and report-honesty tests.
- Report honesty: no BFT, Celestia, production-signature, slashing, or finality
  claim is allowed unless executable tests prove it. Keep the `scenario_11`
  style guard.

## 🧭 Executable Plan Expansion Addendum

Four extra umbrella plans are not enough for this verdict. They would hide
independent hard blockers under large mixed scopes: dependency installation,
durability, planner authority, process/devnet realism, transport faults,
HotStuff-like backend semantics, Celestia-local artifact conformance, structured
evidence, glossary/report honesty, and final integrated evidence. Phase 067 now
uses ten additional executable plan groups instead of four.

### Coverage Audit

- Unique `TASK-NNN` identifiers in `067-TODO.md` plus `067-verdict.md`: `0`.
- Base required GSD plan groups from `067-TODO.md` Section 14: `9`
  (`067-01` through `067-09`).
- Required verdict Local-Conformance-Simulation groups: `10`
  (`067-10` through `067-19`).
- Total required Phase 067 executable plan groups after this addendum: `19`.
- Current branch plan corpus: `21` plan files = `19` required groups plus late
  addendum plans `067-20` and `067-21`.
- No `TASK-NNN` is invented. The verdict expansion uses `VERDICT-LCS-01`
  through `VERDICT-LCS-10` because the source files contain no literal
  `TASK-NNN` rows.
- Existing `PHASE-0` through `PHASE-8` mappings remain unchanged and must not be
  renumbered, merged, or reinterpreted.

### Plan ID Lock

Plan ids are the file numbers. Wave numbers are execution ordering only and must
not be read as plan ids. The verdict expansion is exactly these ten files:

| Required file | Task id | Scope |
| --- | --- | --- |
| `.planning/phases/000/067-Sharded-Concensus/067-10-PLAN.md` | `VERDICT-LCS-01` | Dependency and runnable aggregator readiness |
| `.planning/phases/000/067-Sharded-Concensus/067-11-PLAN.md` | `VERDICT-LCS-02` | Durable consensus evidence store |
| `.planning/phases/000/067-Sharded-Concensus/067-12-PLAN.md` | `VERDICT-LCS-03` | Planner authority and failover claim boundary |
| `.planning/phases/000/067-Sharded-Concensus/067-13-PLAN.md` | `VERDICT-LCS-04` | Multi-process devnet harness |
| `.planning/phases/000/067-Sharded-Concensus/067-14-PLAN.md` | `VERDICT-LCS-05` | Network fault matrix and quorum transport conformance |
| `.planning/phases/000/067-Sharded-Concensus/067-15-PLAN.md` | `VERDICT-LCS-06` | HotStuff-like local backend contract |
| `.planning/phases/000/067-Sharded-Concensus/067-16-PLAN.md` | `VERDICT-LCS-07` | Celestia-local artifact conformance |
| `.planning/phases/000/067-Sharded-Concensus/067-17-PLAN.md` | `VERDICT-LCS-08` | Structured evidence registry |
| `.planning/phases/000/067-Sharded-Concensus/067-18-PLAN.md` | `VERDICT-LCS-09` | Glossary claim registry and report honesty |
| `.planning/phases/000/067-Sharded-Concensus/067-19-PLAN.md` | `VERDICT-LCS-10` | Final integrated Local-Conformance-Simulation gate |

The files `067-12-PLAN.md`, `067-13-PLAN.md`, `067-14-PLAN.md`, and
`067-15-PLAN.md` are mandatory standalone implementation plans. They must not be
merged into `067-10`, `067-11`, `067-16`, or any wave-summary row.

The current branch also carries:

| Addendum file | Scope | Relation |
| --- | --- | --- |
| `.planning/phases/000/067-Sharded-Concensus/067-20-PLAN.md` | certificate-bound publication resume and recovered-primary anti-failback closure | late runtime addendum that MUST be consumed by the final rerun |
| `.planning/phases/000/067-Sharded-Concensus/067-21-PLAN.md` | packet closure, claim-audit completion, flow-alias reconciliation, and final conformance closeout | late packet addendum that MUST be consumed by the final rerun |

### Added Waves

This table is a sequencing summary only. The authoritative one-to-one coverage
mapping is the Task-To-Plan Coverage Table below.

| PLAN id | Wave | Task id | Purpose | Primary strong gates |
| --- | --- | --- | --- | --- |
| `067-10` | 6 | `VERDICT-LCS-01` | Dependency and runnable aggregator readiness | dependency/tool install, process command truth |
| `067-11` | 7 | `VERDICT-LCS-02` | Durable consensus evidence store | Gate 12, restart recovery |
| `067-12` | 8 | `VERDICT-LCS-03` | Planner authority and failover claim boundary | Gate 11, report honesty |
| `067-13` | 9 | `VERDICT-LCS-04` | Multi-process devnet harness | Gate 10, crash/restart realism |
| `067-14` | 10 | `VERDICT-LCS-05` | Network fault matrix and transport conformance | Gate 6, partition/heal |
| `067-15` | 11 | `VERDICT-LCS-06` | HotStuff-like local backend contract | Gates 7 and 8 |
| `067-16` | 12 | `VERDICT-LCS-07` | Celestia-local artifact conformance | Gates 4 and 5 |
| `067-17` | 13 | `VERDICT-LCS-08` | Structured evidence registry | Gate 14 |
| `067-18` | 14 | `VERDICT-LCS-09` | Glossary claim registry and report honesty | Gates 1 and 15 |
| `067-19` | 15 | `VERDICT-LCS-10` | Final local conformance simulation gate | Gate 2 and all hard blockers |

### Added Plan File Inventory

The verdict Local-Conformance-Simulation expansion is exactly `067-10` through
`067-19`. Plans `067-12` through `067-15` are intentionally separate executable
groups and must not be merged into adjacent work.

The branch inventory is broader than the required-group mapping: it now also
includes `067-20-PLAN.md` and `067-21-PLAN.md` as explicit closeout addenda.
They do not create new `VERDICT-LCS-*` ids, but any exhaustive packet-inventory
claim MUST name them.

This table is a file inventory, not a second coverage map. The authoritative
task-to-plan mapping remains the Task-To-Plan Coverage Table below.

| PLAN id | File | Wave | Task id | Execution scope |
| --- | --- | --- | --- | --- |
| `067-10` | `.planning/phases/000/067-Sharded-Concensus/067-10-PLAN.md` | 6 | `VERDICT-LCS-01` | dependency ownership, Cargo graph truth, runnable process command |
| `067-11` | `.planning/phases/000/067-Sharded-Concensus/067-11-PLAN.md` | 7 | `VERDICT-LCS-02` | durable consensus store and restart recovery |
| `067-12` | `.planning/phases/000/067-Sharded-Concensus/067-12-PLAN.md` | 8 | `VERDICT-LCS-03` | planner authority and planner HA claim boundary |
| `067-13` | `.planning/phases/000/067-Sharded-Concensus/067-13-PLAN.md` | 9 | `VERDICT-LCS-04` | multi-process/devnet harness |
| `067-14` | `.planning/phases/000/067-Sharded-Concensus/067-14-PLAN.md` | 10 | `VERDICT-LCS-05` | deterministic network fault matrix and transport conformance |
| `067-15` | `.planning/phases/000/067-Sharded-Concensus/067-15-PLAN.md` | 11 | `VERDICT-LCS-06` | HotStuff-like local backend contract |
| `067-16` | `.planning/phases/000/067-Sharded-Concensus/067-16-PLAN.md` | 12 | `VERDICT-LCS-07` | Celestia-local artifact/API conformance |
| `067-17` | `.planning/phases/000/067-Sharded-Concensus/067-17-PLAN.md` | 13 | `VERDICT-LCS-08` | structured evidence registry |
| `067-18` | `.planning/phases/000/067-Sharded-Concensus/067-18-PLAN.md` | 14 | `VERDICT-LCS-09` | glossary claim registry and report honesty |
| `067-19` | `.planning/phases/000/067-Sharded-Concensus/067-19-PLAN.md` | 15 | `VERDICT-LCS-10` | final integrated Local-Conformance-Simulation gate |

### Installation And Implementation Sequence

`067-10` is the dependency and runnable-command gate. Later plans may add a
crate only when the phase-owning crate exercises the API in runtime or tests.
Dependency-only changes, feature-only scaffolds, and untested external backend
imports fail the anti-placeholder gate.

| Step | Plan gate | Crates, libs, and tools | Install rule |
| --- | --- | --- | --- |
| 1 | `067-10` | `cargo metadata`, `cargo run --release -p z00z_rollup_node -- --help`, direct Cargo owner audit | establish runnable command truth before process/devnet or backend claims |
| 2 | `067-10`/`067-11` | `redb` or existing `z00z_storage` seam | prefer repository storage ownership; add direct `redb` to `z00z_aggregators` only if tests exercise consensus-store tables |
| 3 | `067-10`/`067-14`/`067-17` | `tracing`, `tracing-subscriber`, existing `serde`/`serde_json` | add only where structured logs/evidence records are emitted and asserted |
| 4 | `067-10`/`067-15`/`067-16` | `proptest` | dev-dependency for BFT thresholds, invalid certificates, invalid blobs, and restart properties |
| 5 | `067-10`/`067-16` | `object_store` | add to the DA-owning crate only if Celestia-local retention/retrieve tests exercise backend-like blob storage |
| 6 | `067-10`/`067-08` | `ed25519-dalek` | add only behind the project signer/verifier trait if existing `z00z_crypto` primitives cannot provide the required vote signature seam |
| 7 | `067-10`/`067-08`/`067-15`/`067-16` | `bytes`, `borsh` or a project-owned canonical binary codec | add only when transport envelopes, payload chunks, blob bodies, signed vote bytes, or backend proposal bytes are exercised by tests |
| 8 | `067-10`/`067-09`/`067-13`/`067-14`/`067-17` | `metrics`, `prometheus` | add only when local backend/devnet/fault/evidence metrics are emitted and asserted by tests |
| 9 | `067-13` | `bash`, optional Docker or Docker Compose | local OS-process smoke remains required even when Docker is unavailable |
| 10 | `067-18` | `python3 scripts/audit/audit_067_claims.py` | claim audit must run locally and fail missing owner/artifact/test/claim-level rows |
| 11 | blocked until adapter contracts pass | `hotstuff_rs`, `libp2p`, `celestia-client`, `celestia-rpc`, `celestia-types`, `arc-malachitebft-*`, `blsful`, `reed-solomon-*`, `openraft` | do not install or claim as live production infrastructure unless a tested local adapter contract already exists and the crate is exercised |
| 12 | `067-19` | smart-tests bootstrap, targeted release tests, full `cargo test --release`, GSD review prompt | final gate reruns after all dependency and implementation surfaces are settled |

### Task-To-Plan Coverage Table

| TASK-NNN | PLAN id | Source refs | Inputs | Artifacts | Tests | Expected results | Simulation requirement | Anti-placeholder proof | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| none; `VERDICT-LCS-01` | `067-10` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, `090-New-Scenarios/90-TODO.md`, rollup-node config/process anchors | workspace Cargo graph, `sim_5a7s` process commands | Cargo files, runnable rollup-node target, process tests | cargo metadata, CLI help, process/topology tests | manifest commands are executable and dependency claims are honest | external backend crates remain non-claims until exercised | no crate-only or command-string-only closure | complete |
| none; `VERDICT-LCS-02` | `067-11` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, `090-New-Scenarios/90-TODO.md`, recovery and storage anchors | QC, DA binding, recovery, storage primitives | durable consensus store, restart tests | store, recovery, scenario tests | votes/QCs/anchors survive restart or fail closed | storage commit/recovery is real; wall-clock may be simulated | no in-memory-only or log-only evidence | complete |
| none; `VERDICT-LCS-03` | `067-12` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, `090-New-Scenarios/90-TODO.md`, planner code/config anchors | planner config, route digest, placement generation | planner authority checks, claim-registry row | planner drift and dispatch tests | planner HA is implemented or removed as a live claim | each aggregator recomputes canonical plan locally | no docs-only HA removal or planner stub | complete |
| none; `VERDICT-LCS-04` | `067-13` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, `090-New-Scenarios/90-TODO.md`, `sim_5a7s` manifest/process anchors | runnable binary, durable store, planner authority | local devnet script, optional Compose, process evidence | process tests, smoke script, scenario test | process/devnet behavior is local simulated-full | local OS/Docker boundary only; consensus primitives real | no manifest-only or Docker-file-only proof | complete |
| none; `VERDICT-LCS-05` | `067-14` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, transport/vote/QC anchors | signature, transport, durable evidence, process harness | fault scheduler, transport evidence | transport fault, adapter, scenario tests | network simulation cannot inject consensus truth | fake delivery timing only; replay/signature real | no label-only fault simulation | complete |
| none; `VERDICT-LCS-06` | `067-15` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, BFT and subject/replay anchors | BFT committee, subject/replay, transport, store | HotStuff-local backend, view-change evidence | HotStuff-local, BFT rules, scenario tests | local HotStuff-like claim is executable | backend local; subject/replay/validator real | no name-only HotStuff or CFT-as-BFT | complete |
| none; `VERDICT-LCS-07` | `067-16` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, DA and validator anchors | QC binding, local DA, validator, BFT local artifacts | Celestia-local adapter and artifact schema | Celestia binding, DA, validator, scenario tests | Celestia term is local simulated-full | fake provider transport only; artifact contract real | no provider-name-only or constant blob adapter | complete |
| none; `VERDICT-LCS-08` | `067-17` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, evidence/report anchors | transport, Celestia-local, durable store, report writer | evidence registry, scenario evidence JSON | evidence and scenario tests | safety evidence is machine-auditable | local faults simulated; artifact refs real | no string-only or un-emitted evidence | complete |
| none; `VERDICT-LCS-09` | `067-18` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, all plan files, glossary/report anchors | all plan files, glossary terms, evidence artifacts | claim registry, audit script, report tests | claim audit and scenario tests | every term has enforced claim state | simulated-full must cite executable local evidence | no prose-only glossary | planned |
| none; `VERDICT-LCS-10` | `067-19` | `067-TODO.md`, `067-verdict.md`, `067-CONTEXT.md`, `067-COVERAGE.md`, all plan files, scenario_11 anchors | all prior plans and evidence | final conformance doc, final evidence bundle | scenario_11, claim audit, devnet smoke, release tests | integrated local conformance proof closes all hard blockers | only allowed external boundaries faked | no compile-only or disconnected-unit closure | planned |

## ✅ Проверка

Прошли свежие targeted release checks:

```bash
cargo test --release -p z00z_aggregators --features test-params-fast --test test_shard_quorum_certificate -- --nocapture
cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture
cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture
```

Результат: `3 + 3 + 3` tests passed. Exam structure тоже проверена: `25` questions, `25` blank `Ans:` slots, JSONL валиден.
