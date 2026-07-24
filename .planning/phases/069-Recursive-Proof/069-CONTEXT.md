# Phase 069: Recursive Proof - Context

**Gathered:** 2026-07-11
**Status:** Plans generated; repeated adversarial ownership and test-manifest review corrected
**Source:** PRD Express Path (`.planning/phases/069-Recursive-Proof/069-TODO.md`)

<domain>
## Phase Boundary

Phase 069 delivers a non-authoritative hybrid recursive checkpoint proof lane
over the existing storage-owned checkpoint theorem:

- Nova IVC folds every accepted checkpoint step, with compression cadence
  selected from local measurements and separated from recovery/publication.
- Plonky3 proves the same transition-consistency predicate as a base STARK and
  recursively proves exact ordered epoch ranges plus a separate rolling-history
  base/successor/rotation relation that verifies predecessor proofs, not digests.
- Verified backend receipts bind proof, public input, context, predicate, and
  parameter manifests into the inherited recursive sidecar, epoch manifest,
  archive, and PQ audit-anchor surfaces.
- Canonical checkpoint admission, replay, exact transaction proof bytes, and
  `CheckpointProofSystem::VERIFIED` remain unchanged and non-authoritative.
- After the height-1000 block satisfies inherited canonical DA-ready/QC gates,
  it closes its epoch and commits an outbox without waiting for Nova, Plonky3,
  PQ, evidence-publication, or archive workers. Later recursive evidence can
  block evidence promotion or deletion, never canonical finality.
- Z00Z is a client-centric notary: the network keeps current state/recovery data
  as a mandatory multi-shard distributed-HJMT service—not a root or full-state
  replica—plus compact certified anchors/MMR/rotation history; wallets keep personal
  receipts/history/backups; Phase 069 reserves a disabled/unreachable encrypted-
  delivery handoff for Phase 071 while seed recovery is implemented as the
  current-unspent fallback; canonical raw/non-derivable witness/delta/exact
  transaction-proof bytes live in a lossless bounded challenge ring and expire
  only through the challenge scope of `RetentionTicketV2`. Nova compressed proof
  bodies follow their separate bounded Plan-09 lifecycle.

The complete normative boundary, invariants, object contracts, rejection rules,
tests, acceptance criteria, and done definition remain in `069-TODO.md` and
must be read directly by every planner and executor.

</domain>

<decisions>
## Implementation Decisions

### Normative authority

- `069-TODO.md` is normative rather than advisory. Current source, tests, and
  repository-local configuration remain implementation ground truth.
- Target-design, future-design, planned, and equivalent forward-looking
  statements in `069-TODO.md` and its listed normative references are mandatory
  live-code scope for Phase 069. Wording records implementation order, not a
  deferral; canonical `CheckpointProofSystem::VERIFIED` promotion remains the
  explicit fail-closed security boundary.
- Phase 068 storage-owned contracts are inherited and regression-tested; Phase
  069 must not create a duplicate checkpoint theorem, schema family, codec, or
  authority path.
- API suffix, wire type/version, cryptographic/transcript version, root/public-
  input encoding generation, config schema generation, runtime profile
  generation, and authority/parameter generation are seven independent axes.
  No plan, codec, migration, or
  audit may infer one from another or try another version after decode/
  verification failure.
- Phase registration and every later GSD command must reuse the existing
  `.planning/phases/069-Recursive-Proof/` directory. The append-only
  `phase.add` operation must not be called for this phase.
- There are no literal `TASK-NNN` identifiers or task rows in `069-TODO.md`.
  Planning must record `task_ids: []` and `copied_task_rows: []` rather than
  inventing identifiers.
- There are thirteen required owner groups. The repository contains fourteen
  plan packets because historical/stopped `069-05` is preserved and active
  corrective overlay `069-051` executes Group 05; Groups 01-04 and 06-13 keep
  their same-numbered packets. Closeout MUST distinguish historical evidence
  from the active Group 05 contract.

### Dependency and feasibility gates

- Dependency, license, MSRV, feature, API, and compile compatibility preflight
  precedes backend implementation.
- Exact compatible `nova-snark` and `p3-*`/`p3-recursion` revisions must be
  pinned; floating revisions and unreviewed feature combinations are forbidden.
- A failed feasibility gate produces a stop/split evidence packet. It cannot be
  converted into mock-backed, placeholder-backed, or docs-only completion.

### Predicate and encoding

- Nova R1CS and Plonky3 AIR prove the same versioned
  `CheckpointTransitionConsistencyV2` predicate and must agree with a
  backend-neutral evaluator and native checkpoint replay.
- The predicate constrains ordered input/output application, HJMT paths,
  spent/nullifier transitions, delta and journal commitments, root continuity,
  transaction proof-byte digests, and state plus settlement output roots.
- `RecursiveCheckpointContextV2` binds chain, network, genesis, checkpoint
  config, and predicate identity without silently changing V1 checkpoint bytes.
- Canonical 32-byte values use sixteen range-constrained little-endian `u16`
  limbs. Direct modular reduction and backend-local theorem substitutions are
  forbidden.
- Hash gadgets or versioned translation proofs must preserve the existing
  storage-owned commitment semantics.
- Configuration changes use two distinct operations: first a same-bytes
  schema-2 API-owner correction to `CheckpointContractConfigV2`, then an
  explicit atomic schema-2-to-schema-3 migration to
  `CheckpointContractConfigV3`. ConfigV3 must define separate backend-body,
  proof-envelope, and
  sidecar cap precedence, separate Nova fold, recovery, compression, and
  publication cadences, define the 1,555,200-block challenge window and compact
  history budgets,
  and migrate legacy `PqEpochFinality`/`is_pq_authoritative` semantics to
  evidence-oriented names or quarantine them as compatibility-only labels.
- V2 replaces `pq_signature_or_commitment` with the non-authenticating
  `epoch_evidence_commitment` framed under the storage PQ-anchor domain and an
  explicit legacy label/version. Legacy bytes are available only through the
  registry-declared bounded offline migration/audit entry point, never a
  signature, verifier result, receipt, PQ authority, online fallback, or
  canonical authority signal.

### Version registry and cutover

- `z00z_storage::checkpoint` owns one exhaustive
  `CheckpointVersionRegistryV2`. Every public, persisted, or networked family
  declares API owner, type ID/magic, write/read wire versions, domain/transcript
  generation, root/public-input encoding generation, config schema generation,
  runtime profile identifier/generation/authority-pinned manifest digest,
  authority/parameter generation, cap, lifecycle, activation
  boundary, migration function, and
  typed reject mapping.
- The public Phase 069 recursive/epoch V2 family uses wire version 2 by explicit
  registry assignment, not by suffix. Private V2 trace/circuit subrecords may
  use profile-bound local codec version 1 only when unreachable from public
  ingress.
- `CheckpointTransitionStatementV1`, `CheckpointDaReferenceV1`,
  `CheckpointPublicationEvidenceV1`, `CheckpointLifecycleV1`,
  `ArchiveProviderReceiptV1`, `RetrievalAuditV1`, and `StateSnapshotV1` remain
  live inherited contracts. Archive-manifest, pruning-decision, and PQ-anchor V1
  families become bounded offline migration/audit inputs at their Plan 09
  cutovers. Recursive-proof V1 families have no typed decoder after 069-051 T0.
- The live `CheckpointContractConfigV1` owner already enforces schema
  `version == 2`; 069-051 first renames that API owner and all callers to
  `CheckpointContractConfigV2` while preserving exact YAML bytes, validated
  semantics, and config digest. It then migrates through a complete
  `ConfigV3RenameLedger` to the sole active `CheckpointContractConfigV3`
  runtime/writer. The migration binds source/destination bytes and digests,
  rejects defaults/aliases/unknown fields, and atomically exposes either the
  complete V2 or complete V3 head. No speculative schema-1 migrator is allowed
  without actual bytes.
- ConfigV2/ConfigV3 use registry mode `TypedConfigSchema`: cap raw YAML first,
  reject aliases/anchors/tags/duplicate or merge keys/trailing documents, read
  one top-level schema version, and invoke exactly one decoder. Config schema 3
  is not recursive-object wire version 3; profile generation 2 is independently
  selected only after exact schema decode and full profile-manifest validation.
  ConfigV3, the registry, config head, migration/activation record, jobs,
  proofs/receipts, measurements, and rotation bridges must bind the same runtime
  profile identifier/generation/authority-pinned canonical manifest digest.
  Reusing an identifier/generation with altered manifest bytes is a substitution
  failure, and the profile manifest itself must remain below registry/config/
  head/migration/activation in the acyclic digest DAG.
- New portable decoders and untyped ingress select a fixed preheader/type/
  version/cap before allocation. An immutable inherited V1 grammar without a
  preheader is reachable only through its registry-declared typed API/storage
  namespace, which selects exactly one capped decoder before reading payload
  semantics. Its original canonical bytes/digest remain unchanged; the outer V2
  adapter or migration record binds the legacy tuple and digest. Unknown,
  ambiguous, cross-type, downgrade, trailing, mixed-generation, suffix-derived,
  and fallback cases reject. Real migrations are bounded dual-read/single-write,
  validate both canonical forms, bind source/destination domains and digests,
  and atomically persist the new object plus idempotent cutover record.
- Plans 06–13 consume/extend the same registry; Plans 10/12 prove crash recovery,
  Plan 11 keeps measurements generation-homogeneous, and Plan 13 requires zero
  unclassified recursive V1/V2 or config V1/V2/V3 occurrence plus registry/
  code/docs/scenario parity.

### Backend behavior

- Real Nova proving and verification is required for one-step folding,
  recursive chains, compression, restart, and reorg coverage.
- Nova folding cadence and compressed proof cadence are separate decisions.
- Real Plonky3 base-STARK verification precedes recursive epoch aggregation.
- Plonky3 epoch proofs bind exact ordered heights, real leaf count, aggregation
  shape, context, predicate, and parameters; Nova-only wrapping rejects.
- Rolling-history successors verify the previous history proof plus the exact
  next epoch proof; parameter/AIR/transcript/VK changes require an explicit
  `HistoryRotationBridgeV2`.
- Shape checks, arbitrary non-empty proof bytes, digest checks, metadata
  booleans, compile-only probes, or scaffold APIs cannot close backend plans.

### Security and authority

- Recursive evidence stays shadow-only and cannot block canonical admission
  when proving lags, crashes, exhausts resources, or is disabled pre-promotion.
- A PQ-oriented outer Plonky3 proof does not upgrade nested classical
  signatures, commitments, range proofs, or spend proofs to end-to-end PQ
  validity.
- Cross-network, cross-genesis, cross-config, cross-predicate, mixed-parameter,
  stale-fork, and downgrade replay must fail closed.
- Proving workers are untrusted; only pinned local verifier success plus atomic
  persistence can create a verified receipt.
- The sole verifier-to-storage seam is
  `z00z_storage::checkpoint::recursive_v2`: it keeps concrete backend types
  private and exposes only storage-owned opaque evidence. Rollup consumes this
  facade; no recursive backend crate, dependency inversion, or second
  conversion seam exists. Persisted records regain verified capability only
  after actual backend re-verification.
- Witness secrecy, CSPRNG use, parameter integrity, bounded queues, memory/time
  budgets, cancellation, restart, and reorg invalidation are required.
- Retained witness/archive material follows the normative retention policy, but
  transient prover-only byte buffers, assignments, and derived copies must have
  bounded lifetimes, minimize cloning, and use an approved zeroize-on-drop
  wrapper on success, error, timeout, cancellation, and unwind paths. If a
  selected backend type cannot be wiped safely, Plan 02 must record that fact as
  a security blocker or define an audited non-copying boundary before execution.

### Storage, retention, and simulation

- Existing sidecar, PQ anchor, archive, retrieval-audit, snapshot, pruning, and
  config ownership is reused. Versioned challenge-pack, compact-anchor, rolling-
  history, retention-ticket/ledger, wallet-receipt, and backup contracts extend
  those owners without creating a parallel storage theorem or authority path.
- The first active profile uses the existing HJMT/runtime route and placement
  owners with 16 logical shards, three distinct-domain replicas/write quorum two
  per shard, no full-state placement, one global root, and COW copy/root-verify/
  delta-catch-up/CAS resharding. Seed recovery scans all live shards, locally
  authenticates/decrypts the bounded `{r_pub, tag16, enc_pack}` recovery data (or
  its explicit migrated successor), and verifies a fresh current-root witness;
  `tag16` alone is never ownership.
- Archive retrievability is not state validity; snapshots cannot become a trust
  root before a promoted canonical validity path succeeds.
- Celestia remains DA-only. IPFS is allowed only with pinning, independent
  provider receipts, and unpredictable retrieval audits.
- Default finalization math is 5 seconds/block, 1000 blocks/epoch (17.28
  epochs/day), and 1,555,200 finalized blocks/90-day window from DA readiness.
  Permanent anchors SHOULD target 1 KiB, MUST be <= 4 KiB, and total compact
  historical growth MUST remain <= 100 KiB/day excluding current state.
- `EpochChallengePackV2` is canonical, finite, lossless, content-deduplicated,
  and encoded by the mandatory measured versioned RS(10,16) profile from the
  first retention-enabled launch. Full-replica fallback is forbidden; missing
  feasibility/placement blocks promotion. Retained challenge bytes MUST plateau
  after expiry.
- `RetentionTicketV2` is the sole deletion authority. Current state, compact
  anchors/MMR/rotation commitments, ticket/audit digests, referenced generation
  metadata, and the latest three verified COW state snapshots MUST survive.
  Its challenge and Nova proof-body scopes are non-interchangeable.
- Plan 09 owns `NovaRetentionStateV2`: at most two accepted compressed bodies per
  epoch, eight pending-PQ epochs, 16 bodies, and 2 MiB. Normal body deletion waits
  for independent exact-epoch Plonky3 verification, evidence join, later certified
  evidence-MMR inclusion, two certified epochs, cleared holds/references, and CAS.
  At the cap, canonical close/fold continue while compression/publication stops;
  an uncompressed epoch becomes terminal body-less `GapRecorded` without
  ticket/GC, while a truthful body-bearing `Abandoned` scope may delete only its
  exact Nova body.
  Canonical challenge bytes keep their independent 90-day/history/RS/audit clock.
- Plan 06 separately rotates the active Nova accumulator plus two verified local
  recovery snapshots; Plan 10 performs journaled ticket/head-before-unlink GC for
  proof bodies and journaled replacement-head-before-unlink local snapshot GC.
  Plan 11 must select a measured positive finite hot-recovery byte cap before
  production activation; neither 24-GiB RSS nor the 128-MiB experiment ceiling is
  a storage capacity value.
- `FinalizedWalletReceiptV2` and `WalletBackupV5` keep user history client-owned;
  providers are untrusted. Backup encrypts the canonical generation before
  erasure coding, verifies `k`-of-`n` ciphertext reconstruction/decryption/
  readback before compare-and-swap, and keeps at least three reconstructible
  logical generations.
- The offline receipt mailbox is a Phase-071 delivery class, distinct from the
  wallet-local `RequestInboxRecordV1` and live-state recovery capsule. Phase 069
  reserves only unique `ReservedUnreachable` type/domain rows, ConfigV3
  `declared_only`/cap zero, the generic finalized-output/outbox seam, and
  mailbox-absent seed recovery. It has no codec/store/provider/reader/writer/
  activation/ACK/GC/benchmark/scenario. Phase 071 owns the immutable AEAD entry,
  per-output ECDH locator, replication, post-finality activation, ACK/GC,
  measurement, and delivery tests.
- Canonical commit/outbox, recursive-evidence promotion, and wallet backup are
  three independent durable state machines. Crash recovery is idempotent; no
  cross-node/provider atomic transaction or side-state finality is allowed.
  Canonical commit binds touched-shard prepare/quorum receipts, active route and
  shard-manifest digests, state/head/certificate, and durable promote outbox.
  The Phase-071 handoff requires future mailbox upload/activation/ACK/GC to be
  subordinate idempotent outbox consumers inside these authority domains, not a
  fourth state machine or distributed transaction; none is live in Phase 069.
- Local simulation must use real project cryptography, package verification,
  HJMT journal/state transitions, storage commit/recovery, publication
  bindings, validator/watcher checks, and per-component state. Only external
  transports, remote process boundaries, wall clock/fault schedulers, or
  unavailable third-party networks may be faked.

### Plan packet contract

- Preserve fourteen plan packets (`069-01`..`069-13` plus `069-051`, where
  `069-05` is historical/stopped and `069-051` is its active corrective overlay)
  plus `069-COVERAGE.md`; do not renumber, erase history, merge away, or defer
  any of the thirteen owner groups.
- Every plan must contain the user-required plan/task/evidence fields, concrete
  files and APIs, source references, current-code anchors, positive and negative
  tests, simulation gates, anti-placeholder proof, and a Coverage Appendix.
- Every auto task verify block must run the smart-test bootstrap first, then
  relevant release tests, and the local `GSD-Review-Tasks-Execution` loop at
  least three times until two consecutive runs are clean.
- Local correctness cannot be deferred or marked best-effort. Runtime behavior
  cannot close through compile-only evidence, and code behavior cannot close
  through documentation-only evidence.
- `069-COVERAGE.md` is the planning-time atomic coverage mirror. Its deterministic
  inventory traces every current TODO bullet, numbered item, data-table row,
  normative prose rule, exact rejection, test, artifact, cap case, and SHOULD
  to exactly one primary plan plus task/action/test/result/evidence locations.
  Section-only coverage is navigation and cannot close planning.

### the agent's Discretion

- Exact task boundaries inside each fixed plan group, provided every normative
  contract and acceptance criterion is covered exactly once or explicitly
  identified as a cross-cutting regression gate.
- Exact module/function names for newly introduced backend internals when
  `069-TODO.md` does not already freeze a public name.
- Wave parallelism after preserving the strict feasibility, predicate, backend,
  integration, lifecycle, benchmark, simulator, and closeout dependency order.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Normative phase and predecessor contracts

- `.planning/phases/069-Recursive-Proof/069-TODO.md` — complete Phase 069 PRD,
  invariants, contracts, test matrix, implementation groups, and done gate.
- `.planning/phases/000/068-Checkpoint-Contract/068-TODO.md` — checkpoint theorem
  and recursive-ready architecture consumed by Phase 069.
- `.planning/phases/068-Checkpoint-Contract/068-TODO.md` — predecessor scope and
  storage-owned contracts that Phase 069 inherits.
- `.planning/phases/068-Checkpoint-Contract/068-VERIFICATION.md` — verified
  predecessor baseline and evidence boundary.

### Repository rules and verification

- `.github/copilot-instructions.md` — repository execution and review rules.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` — mandatory architecture,
  cryptographic, storage, and naming constraints.
- `.github/prompts/gsd-review-tasks-execution.prompt.md` — required repeated
  execution review loop.
- `.github/skills/smart-tests-bootstrap/SKILL.md` — mandatory bootstrap gate.
- `.github/skills/z00z-full-verify-gate/SKILL.md` — canonical full verification
  gate when phase closure reaches its final packet.

### Current storage-owned implementation anchors

- `crates/z00z_storage/src/checkpoint/artifact_stmt.rs` — canonical checkpoint
  statement and digest construction.
- `crates/z00z_storage/src/checkpoint/exec_input.rs` — ordered replay input and
  exact transaction proof bytes.
- `crates/z00z_storage/src/checkpoint/recursive_checkpoint.rs` — inherited
  public input, proof envelope, sidecar, chain evidence, codecs, and shape
  verifier.
- `crates/z00z_storage/src/checkpoint/pq_anchor.rs` — inherited PQ audit-anchor
  envelope and validation.
- `crates/z00z_storage/src/checkpoint/contract_config.rs` — authority stages,
  limits, paths, cadence, retention, and degraded-mode configuration.
- `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml` — repository
  config instance and compatibility labels.
- `crates/z00z_storage/src/checkpoint/link.rs` — predecessor continuity binding.
- `crates/z00z_storage/src/checkpoint/state_snapshot.rs` — inherited snapshot
  binding and cadence rules.
- `crates/z00z_storage/src/checkpoint/mod.rs` — public storage ownership surface.
- `crates/z00z_rollup_node/src/da.rs` — DA/publication boundary that must remain
  separate from proof validity.

</canonical_refs>

<specifics>
## Specific Ideas

- The default PQ cadence is 1000 positive checkpoint heights; height 0 is
  genesis, and epoch math is explicitly defined in `069-TODO.md`. Cadence
  closes the canonical epoch immediately and schedules asynchronous proof/
  history/PQ publication.
- Local 3-step and 5-step Nova chains plus one bounded Plonky3 epoch fixture are
  mandatory real-backend evidence.
- Benchmarks must report proof size, folding/compression/proving/verifying time,
  peak memory, witness size, backend and parameter digests, build profile, and
  fixture identity as local evidence rather than production claims.

</specifics>

<coverage_contract>

## 🔎 Normative Obligation Ownership

<!-- TODO_ATOMIC_CONTEXT: sha256=226976b17c8783cb51b77fae255c5e4eaffda482a49412021b084ac2ad6766c5 atoms=1335 shoulds=8 owners=13 -->

This context incorporates every atomic obligation from the exact TODO revision
identified by the marker above through the hash-checked inventory in
`069-COVERAGE.md`. The incorporation is normative, not a summary waiver: each
inventory row retains its exact TODO line, content digest, plan/task owner, and
action/test/result/evidence pointers. If the TODO hash, atom count, ownership,
or any row changes, `069-COVERAGE-AUDIT.py` must fail until this context marker,
the inventory, and the affected executable plan semantics are reviewed together.
No atom is considered covered merely because its parent section is mentioned.
The audit additionally locks all `67/67` Purpose, In Scope, Out of Scope, and
Required Artifacts rows to explicit semantic owners, plus targeted task-level
assertions for obligations whose correct executor differs from a keyword or
nearest-section fallback.
It also locks all `153/153` Unit, Integration, Chain, Simulator, Property/Fuzz,
and Audit test-section rows to their reviewed executable owners, preventing
component keywords from stealing the primary test owner.

The planning preflight found `0` literal `TASK-NNN` rows and exactly `13`
required plan groups. `069-COVERAGE.md` is incorporated into this context as
the detailed source-ref, current-code-anchor, and clause mirror.

| Primary plan | Invariants | Acceptance criteria | Done bullets | Handoff items |
| --- | --- | --- | --- | --- |
| `069-01` | `004` | `008`, `009`, `016` | `01`, `03`, `12` | `01`, `02`, `11` |
| `069-02` | none | none | none | `03`, `12` |
| `069-03` | `001`, `002`, `010` | `001`, `012`, `018`, `019` | `07`, `13` | `06`, `08`, `09` |
| `069-04` | none | `032` | `06` | `07` |
| `069-05` | `013`, `023` | `002`, `033`, `034`, `042` | `14` | none |
| `069-06` | `005` | `003`, `004`, `005` | `04`, `09`, `10` | `04` |
| `069-07` | none | `035` | none | `05` |
| `069-08` | `014` | `020`, `021`, `022`, `036` | `05` | none |
| `069-09` | `003`, `006`, `019`, `021`, `022`, `024`, `025` | `006`, `007`, `015`, `023`-`027`, `029`-`031`, `041`, `043`, `047` | `02`, `11`, `16`-`21` | none |
| `069-10` | `016` | `039`, `040`, `046` | `23` | `10` |
| `069-11` | `007`, `017` | `010` | `22` | none |
| `069-12` | `011`, `015` | `013`, `014`, `028`, `037`, `044`, `045` | `08`, `15` | none |
| `069-13` | `008`, `009`, `012`, `018`, `020` | `011`, `017`, `038` | `24`-`26` | `13`, `14` |

The totals are `25/25` invariants, `47/47` acceptance criteria, `26/26` Done
Definition bullets, and `14/14` Planning Handoff items. Cross-plan mentions are
secondary regression or audit links and must not create another owner.

</coverage_contract>

<deferred>
## Deferred Ideas

No local correctness item is deferred. Only the explicit out-of-scope and
future backend-promotion items in `069-TODO.md` remain outside Phase 069.

</deferred>

---

*Phase: 069-recursive-proof*
*Context gathered: 2026-07-11 via PRD Express Path*
