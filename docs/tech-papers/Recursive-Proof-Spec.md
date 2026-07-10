---
phase: 069
artifact: recursive-proof-spec
status: planned
updated: 2026-07-03
authority: phase-local
---

# Phase 069 Recursive Proof Spec

## 🎯 Purpose

📌 This document is the self-contained Phase 069 specification for recursive
checkpoint proof work in Z00Z. It converts the Phase 69 idea corpus into one
code-facing contract that can be implemented, tested, and audited without
promoting speculative proof claims into live checkpoint authority.

📌 Phase 069 MUST implement the recursive-ready proof architecture over the
checkpoint transition statement from
`docs/tech-papers/Recursive-Ready-Checkpoint-Contract.md`. The selected
implementation architecture is hybrid:

- Nova compressed proof on every checkpoint block as a fast classical IVC lane.
- Plonky3 recursive STARK epoch proof every configured PQ cadence interval,
  default `1000` blocks, as the long-horizon PQ-friendly history lock.
- Canonical checkpoint artifacts, exact transaction proof bytes, HJMT roots,
  witness roots, DA/archive manifests, and checkpoint links remain mandatory.

📌 The Plonky3 epoch proof MUST prove or re-check the canonical transition range
for the epoch. It MUST NOT be a STARK wrapper that only proves "1000 Nova proofs
verified", because that would inherit Nova's ECC/DLOG soundness and would not
survive the quantum rewrite threat model. The epoch proof MAY bind the Nova
chain root as additional performance/audit evidence, but the PQ authority must
come from canonical checkpoint statements, witnesses, deltas, roots, and
archive commitments.

📌 Phase 069 still MUST NOT replace the current spend verifier, range-proof
verifier, checkpoint artifact codec, checkpoint link, or canonical replay path.
`CheckpointProofSystem::VERIFIED` remains disabled until a later promotion
stage has real verifier code, codec support, rollback policy, benchmarks,
negative tests, and security review.

📌 The primary output of this phase is a deterministic hybrid recursive
checkpoint contract: typed public inputs, Nova block proof objects, Plonky3
epoch statement/proof objects, epoch manifests, verifier adapter interfaces,
typed reject reasons, measurements, storage path gates, and 3 to 5 step chain
evidence for prior-output binding.

📌 The secondary output is a recursive documentation and audit packet that makes
the hybrid lane reviewable: statement vectors, object schemas, rejection matrix,
benchmark metadata, backend-manifest red lines, Plonky3/Nova parameter
decisions, and PQ epoch artifact evidence for every configured 1000-block
boundary once enforcement is active.

## 🧭 Document Structure

📌 This spec is intentionally long-form. It is the Phase 069 single source of
truth. Implementation agents MUST NOT need to re-open the research chats to
decide what to build, reject, test, or measure.

Reading order:

1. Authority and source disposition define which upstream claims survived.
2. Current code truth anchors the design in live repository modules.
3. Architecture doublecheck ledger records each major decision, source, code
   confirmation, and gate.
4. Object contracts define the statement, public input, witness, proof, sidecar,
   chain evidence, measurements, PQ anchor, and documentation packet.
5. Config gates define the real repository YAML file that controls modes,
   limits, paths, stage promotion, retention, DA, and PQ cadence.
6. Module placement defines where the implementation lives across storage,
   runtime, rollup node, crypto, simulator, and the future recursive proof crate.
7. Diagrams provide C4 and Mermaid-spectrum views of the same architecture.
8. Failure model, tests, acceptance criteria, implementation slices, and
   artifacts define how Phase 069 is accepted.

## 🔐 Authority Chain

📌 Live normative sources for this spec:

- This document as the Phase 069 single source of truth.
- `docs/tech-papers/Recursive-Ready-Checkpoint-Contract.md`
- `.planning/phases/068-Checkpoint-Contract/068-TODO.md`
- Current code under `crates/z00z_storage/src/checkpoint/`
- Current code under `crates/z00z_storage/src/settlement/`
- Current validator, watcher, rollup-node, and simulator checkpoint/publication
  seams under `crates/z00z_runtime/`, `crates/z00z_rollup_node/`, and
  `crates/z00z_simulator/`

📌 Historical Phase 069 proposal inputs fully incorporated into this spec and
removed from `.planning/phases/69-Recursive-Proof/` after extraction:

- `+69-70-proposal.md`
- `+69-70-proposal.audit.md`

📌 Historical supporting research inputs fully incorporated into this spec and
removed from `.planning/phases/69-Recursive-Proof/` after extraction:

- `README.md`
- `README-recursive_proofs.md`
- `z00z-recursive-proofs.md`
- `nova-supernova.md`
- `20-Recursive checkpoint proof.md`
- `11_Z00Z_Recursive_StateProof.md`
- `12_chat-PQ Recursive Proof-last.md`
- `13_chat-Recursive Proof Analysis.md`
- `14_chat-Обзор PQ рекурсивных доказательств.md`

⚠️ The supporting research inputs are not direct implementation authority.
They contribute failure cases, threat-model warnings, retention requirements,
measurement questions, and backend evaluation checklists only after they are
filtered by this spec and the recursive-ready checkpoint contract.

## 📚 Source Disposition

| Source | Accepted into Phase 069 | Rejected or deferred |
| --- | --- | --- |
| `README.md` and `README-recursive_proofs.md` | Exact statement first, no spend verifier replacement, 3 to 5 step chain, proof-size/prover/verifier measurements, recursive sidecar evidence. | Backend-first design that does not bind the checkpoint contract or tries to use proof-system choice as theorem authority. |
| `z00z-recursive-proofs.md` | State-transition model with `root_old`, update witness, `root_new`, prior-output binding, and Nova-compatible per-block IVC flow. | Nova or SuperNova as PQ/final authority. |
| `nova-supernova.md` | Nova compressed per-block lane, SuperNova as future non-uniform-step comparison, and classical IVC measurement categories. | Proof-size claims as Z00Z implementation facts before local measurement; ECC proofs as quantum-safe history locks. |
| `20-Recursive checkpoint proof.md` | Current code truth: no recursive proof chain exists today. | Treating recursive header fields as live code. |
| `11_Z00Z_Recursive_StateProof.md` | Chain failure cases, nullifier/spent retention, fraud/audit evidence categories. | Link tag as recursive proof authority, 100 byte proofs, 200 KB active state, IPFS-only history. |
| `12_chat-PQ Recursive Proof-last.md` | PQ cautions, storage/recovery warnings, nullifier permanence, DA availability warnings. | Magic aggregation, same-challenge aggregation, production PQ theorem closure. |
| `13_chat-Recursive Proof Analysis.md` | ECC attack model, RNG/reuse risks, genesis trust, quantum migration warning. | DLP-based proof as long-term PQ-safe authority. |
| `14_chat-Обзор PQ рекурсивных доказательств.md` | PQ backend evidence checklist: parameters, ABI, vectors, PoK, center-lift, domain slots, constant-time review; STARK/FRI as the practical PQ-friendly implementation direction for Phase 069. | Pedersen binding as PQ-safe, unproved RLWE/folding claims, LatticeFold/RLWE/Fractal as live production backends. |
| `plonky3-stark.md` | Primary real implementation target: Nova compressed every block plus Plonky3 recursive STARK epoch proof every `post_quantum.cadence_blocks`; Plonky3 must prove canonical range and may bind Nova root. | Plonky3 proof that only verifies Nova proofs; permanent storage of all per-block STARK proofs; exact proof-size claims without Z00Z benchmarks. |
| `068-TODO.md` | Checkpoint-contract-first architecture, recursive branch surfaces, DA readiness gate, retention policy, stage transitions, and 1000-block PQ epoch cadence. | Treating a generic PQ anchor as enough once Plonky3 epoch proof is selected, or enabling live cadence before `pq_anchor_writer`. |
| `+69-70-proposal.md` | Wave 69 scope and Wave 70 dependency boundaries. | DA/publication evidence as recursive proof authority. |
| `+69-70-proposal.audit.md` | Conflict resolutions and residual risks. | Any source group marked rejected or research-only. |

### 🔗 Imported Backend Reference Ledger

📌 The following references were imported from `plonky3-stark.md` so the Phase
069 spec can stand alone after the idea file is removed. These links are
evidence for backend selection and risk classification; they are not runtime
configuration and MUST NOT bypass repository dependency pinning.

| Reference | Phase 069 use | Imported decision |
| --- | --- | --- |
| [Plonky3](https://github.com/Plonky3/Plonky3) | Primary STARK toolkit candidate. | Use as the default STARK family because it is Rust-native, supports FRI/STARK-oriented primitives, and includes KoalaBear/Poseidon2-relevant components. |
| [Plonky3-recursion](https://github.com/Plonky3/Plonky3-recursion/) | Primary recursive STARK implementation target. | Use for `plonky3_stark_epoch_v1` only behind non-authoritative Phase 069 gates until audit and promotion evidence exists. |
| [Plonky3-recursion benchmarks](https://plonky3.github.io/Plonky3-recursion/appendix/benchmark.html) | Measurement planning input. | Treat any timing or proof-size number as a Z00Z measurement target, not as a verified Z00Z result. |
| [Plonky3 audit report](https://leastauthority.com/wp-content/uploads/2024/11/Updated_071124_Polygon_Plonky3_Final_Audit_Report.pdf) | Security-review reference for the Plonky3 base stack. | Audit coverage is not equivalent to full audit coverage of `Plonky3-recursion`; Phase 069 must still record recursion-specific risk. |
| [Nova](https://github.com/microsoft/Nova) | Secondary fast classical IVC baseline. | Use `NovaCompressedBlockProofV1` every block for continuity, measurement, and audit, but never as PQ authority. |
| [Nova paper](https://par.nsf.gov/servlets/purl/10440508) | Classical proof-size and IVC reasoning reference. | Treat 8 KiB to 30 KiB as a rough planning target until Z00Z fixtures measure local proofs. |
| [Arecibo](https://github.com/lurk-lang/arecibo) | Nova/SuperNova-family comparison lane. | Keep as benchmark or future non-uniform-step candidate, not as the Phase 069 primary backend. |
| [LatticeFold](https://github.com/NethermindEth/latticefold) | PQ folding research track. | Keep as research-only until production parameters, ABI, vectors, audit, and implementation gates exist. |
| [Fractal paper](https://link.springer.com/chapter/10.1007/978-3-030-45721-1_27) and [libiop](https://github.com/scipr-lab/libiop) | Transparent recursive-proof design reference. | Do not use as implementation base because the available implementation path is C++/academic-reference oriented. |
| [HyperPlonk](https://github.com/EspressoSystems/hyperplonk) | Future prover/circuit comparison. | Do not select as primary because Phase 069 needs recursion-first STARK epoch finality. |
| [Winterfell](https://github.com/facebook/winterfell) | Generic STARK fallback reference. | Do not use standalone Winterfell as the recursive backend because Phase 069 selects Plonky3-recursion for the STARK lane. |
| [STARK paper](https://eprint.iacr.org/2018/046) | Transparent/STARK/FRI security-family reference. | Use only for security-family reasoning; implementation authority remains the selected, pinned backend plus local Z00Z tests. |

📌 `plonky3-stark.md` import coverage:

| Source block | Spec home | Status |
| --- | --- | --- |
| Backend pros/cons table for Nova, SuperNova, Plonky3, Fractal, HyperPlonk, LatticeFold, RLWE, and generic STARK/Winterfell. | `Hybrid Backend Architecture` and `Backend And PQ Policy`. | Imported and normalized into Phase 069 decisions. |
| Proof-size and overhead tables. | `Nova Block Proof Contract`, `Plonky3 Epoch Proof Contract`, `Config Gates`, and `Measurement Contract`. | Imported as targets and hard caps; exact numbers remain measurement-gated. |
| Hybrid flow: Nova every block, Plonky3 every 1000 blocks. | `Hybrid Backend Architecture`, `Post-Quantum Cadence Contract`, `Required Workflow`, and `Gate Flow`. | Imported and made normative through config gates. |
| Warning that Plonky3 must not only verify Nova proofs. | `Non-Negotiable Invariants`, `Plonky3 Epoch Proof Contract`, `Reject Reason Contract`, `Failure Model`, and acceptance criteria `RCP-AC-021`/`RCP-AC-022`. | Imported as a hard rejection rule. |
| Retention rule: do not store every per-block STARK proof permanently. | `Hybrid Backend Architecture`, `Long-Term Archive Retention Contract`, `Config Gates`, and `Required Artifacts`. | Imported and extended with archive manifests, receipts, retrieval audits, snapshots, and pruning gates. |
| Config deltas for Nova, Plonky3, PQ cadence, and proof-size caps. | `Config Gates`, `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`, and `CheckpointContractConfigV1`. | Imported into the active runtime gate. |

## 🧱 Implementation Dependency And Installation Matrix

📌 Phase 068 defines the checkpoint theorem, storage contract, and YAML gate.
It does not pin recursive backend libraries. Phase 069 is the dependency
authority for Nova, Plonky3, and IPFS/Kubo integration.

📌 Default ownership for dependency wiring:

- `crates/z00z_storage/src/checkpoint/` keeps the statement, artifact, sidecar,
  PQ anchor codecs, and config validation.
- A future `z00z_recursive_proofs` crate SHOULD own Nova and Plonky3 adapters,
  proving/verifying APIs, benchmark harnesses, and backend dependency pins.
- `z00z_rollup_node` SHOULD own Kubo/IPFS RPC wiring and archive-operator
  process integration, not checkpoint theorem bytes.

| Dependency surface | Install target | Required package(s) | Primary docs | Phase 069 rule |
| --- | --- | --- | --- | --- |
| Nova block proof lane | Future `z00z_recursive_proofs` crate | [`nova-snark`](https://docs.rs/nova-snark/), [Microsoft/Nova repository](https://github.com/microsoft/Nova), [Nova paper](https://par.nsf.gov/servlets/purl/10440508) | `docs.rs`, repository README/examples, paper | MUST implement `NovaCompressedBlockProofV1`; MUST stay non-PQ; MUST pin one exact crate version or git revision in workspace metadata before the first adapter lands. |
| Plonky3 recursion lane | Future `z00z_recursive_proofs` crate | [`p3-recursion`](https://docs.rs/crate/p3-recursion/latest), [`p3-uni-stark`](https://docs.rs/crate/p3-uni-stark/latest), [`p3-fri`](https://docs.rs/crate/p3-fri/latest), [`p3-commit`](https://docs.rs/crate/p3-commit/latest), [`p3-challenger`](https://docs.rs/crate/p3-challenger/latest), [`p3-field`](https://docs.rs/crate/p3-field/latest), [`p3-matrix`](https://docs.rs/crate/p3-matrix/latest), [Plonky3 repository](https://github.com/Plonky3/Plonky3), and [Plonky3-recursion repository](https://github.com/Plonky3/Plonky3-recursion/) | `docs.rs`, upstream repositories, local benchmark vectors | MUST implement `Plonky3EpochProofV1`; MUST pin one approved compatibility set; MUST NOT mix unrelated `p3-*` release families inside the same workspace. |
| Plonky3 field/hash profile | Same crate plus shared crypto helpers | [`p3-koala-bear`](https://docs.rs/crate/p3-koala-bear/latest), [`p3-poseidon2`](https://docs.rs/crate/p3-poseidon2/latest), and when sponge helpers are needed [`p3-symmetric`](https://docs.rs/crate/p3-symmetric/latest) | `docs.rs` plus the upstream Plonky3 workspace manifest | MUST match config `field: koala_bear` and `hash: poseidon2`; any alternative field/hash pair rejects unless config, vectors, proofs, and tests move together. |
| IPFS archive RPC client | `z00z_rollup_node` archive adapter or future archive crate | [`ipfs-api-backend-hyper`](https://docs.rs/crate/ipfs-api-backend-hyper/latest) | `docs.rs` crate docs | MUST talk to an external pinned Kubo node over local or private RPC; MUST emit provider receipts and pinning evidence; MUST NOT embed IPFS node logic into canonical checkpoint modules. |
| IPFS/Kubo daemon | Operator host, CI fixture, archive node | [Kubo install guide](https://docs.ipfs.tech/install/command-line/), [Kubo quick start](https://docs.ipfs.tech/how-to/command-line-quick-start/), and [Kubo RPC reference](https://docs.ipfs.tech/reference/kubo/rpc/) | Official IPFS docs | MUST be installed as an external service when `ipfs_pinned` backend is enabled; RPC MUST stay localhost or otherwise private; public RPC exposure is forbidden. |

📌 Version-lock rules:

- A coder MUST NOT start the real backend implementation with floating versions
  such as `*`, `latest`, or unreviewed HEAD checkouts.
- The workspace already contains `p3-field`, `p3-goldilocks`,
  `p3-poseidon2`, and `p3-symmetric` under `crates/z00z_crypto/Cargo.toml`.
  Phase 069 implementation MUST either migrate that family to the selected
  recursion-compatible set in one reviewed wave or isolate recursive backend
  dependencies behind a compatibility-safe crate boundary.
- `cargo add` without an exact version or reviewed git revision is forbidden
  for `nova-snark`, `p3-*`, and the IPFS client crate.
- The exact pinned dependency set and the Kubo binary version used by local
  fixtures or CI MUST be recorded in the Phase 069 documentation packet and in
  the future backend crate manifest.

## 🧾 Key Terms

| Term | Meaning in Phase 069 |
| --- | --- |
| `CheckpointTransitionStatementV1` | The canonical storage-owned statement defined by the recursive-ready checkpoint contract. Every recursive sidecar binds this exact statement digest. |
| `RecursiveCheckpointPublicInputV1` | The proof-facing public input object derived from the statement digest, roots, chain position, backend label, and prior-output binding. |
| `RecursiveCheckpointWitnessV1` | Local witness fixture and archive reference material used by mock or candidate proof adapters. It is not canonical state truth. |
| `RecursiveCheckpointProofV1` | Versioned envelope that can carry a Nova per-block proof, a Plonky3 epoch proof reference, or a local test proof. The active repository profile requires `recursive_hybrid_v1`. |
| `NovaCompressedBlockProofV1` | Classical per-block compressed IVC proof over one `CheckpointTransitionStatementV1`. It binds statement digest, checkpoint link, prior Nova output, and output root; it is not PQ authority. |
| `NovaEpochChainRootV1` | Merkle or commitment root over the ordered Nova block proof digests and statement digests for one epoch. It is optional evidence inside the Plonky3 epoch statement, not the source of PQ soundness. |
| `EpochRangeStatementV1` | Statement for one configured epoch range. It binds start/end heights, start/end roots, previous PQ anchor root, ordered statement root, DA/archive manifest root, witness/delta roots, and optional Nova chain root. |
| `Plonky3EpochProofV1` | Recursive STARK proof for one epoch range using Plonky3/Plonky3-recursion. It must prove the canonical transition range and bind public inputs; it is the selected PQ-friendly epoch lane. |
| `EpochManifestV1` | Retained manifest for a completed epoch: statements, canonical artifacts, links, DA refs, witness roots, Nova proof digests, Plonky3 proof digest, sizes, and retention locations. |
| `Archive Retention Layer` | Z00Z-owned long-term retrieval layer over content-addressed archive manifests, pinned IPFS/CID references, archive node replicas, provider receipts, and retrieval audits. It is separate from Celestia DA. |
| `CheckpointArchiveManifestV1` | Permanent compact metadata root over raw packages, exact proof bytes, witness chunks, deltas, DA payload commitment, archive provider receipts, and retrieval audit roots. |
| `ArchiveProviderReceiptV1` | Provider-neutral evidence that a configured archival backend stores a content-addressed object. It may reference IPFS pinning, paid archival providers, Filecoin-like storage, local archive nodes, or cold object storage. |
| `RetrievalAuditV1` | Periodic proof that archived objects are still retrievable from enough independent replicas. It is an availability/retrieval gate, not state validity. |
| `StateSnapshotV1` | Bootstrap object binding state root, settlement root, latest Plonky3 epoch proof, latest epoch manifest root, archive manifest root, snapshot chunk root, and PQ anchor root. |
| `Full-node pruning` | Local deletion of replay bytes by ordinary full nodes after dispute, Plonky3, manifest, archive replication, and retrieval-audit gates pass. |
| `Archive-node pruning` | Deletion by archive replicas. This is forbidden in the default Phase 069 profile. |
| `RecursiveCheckpointSidecarV1` | Non-authoritative attachment that stores the proof object, verdict, reject reason, and measurements for one checkpoint transition or epoch proof reference. |
| `RecursiveCheckpointChainEvidenceV1` | A 3 to 5 step ordered chain of sidecars proving prior-output binding and tamper rejection. |
| `RecursiveCheckpointMeasurementV1` | Local measurement payload for proof bytes, witness bytes, prover time, verifier time, memory, chain length, and backend label. |
| `RecursiveCheckpointRejectReasonV1` | Stable machine-readable rejection taxonomy for sidecars, proof objects, codecs, and chains. |
| `Canonical branch` | The current authoritative checkpoint path using `CheckpointArtifact`, `CheckpointLink`, `CheckpointExecInput`, exact transaction proof bytes, and `CheckpointProofSystem::OPAQUE_ATTEST`. |
| `Recursive branch` | Hybrid proof lane over the same statement: Nova per block plus Plonky3 epoch proof. It cannot admit checkpoints in Phase 069. |
| `Fast classical lane` | Nova compressed per-block proof lane used for fast local recursion, UX, audit, and benchmarking inside an open epoch. |
| `PQ epoch lane` | Plonky3 recursive STARK lane that locks each completed epoch under hash/FRI/STARK assumptions. |
| `Open epoch` | The last not-yet-Plonky3-finalized range of fewer than `post_quantum.cadence_blocks` blocks. During a quantum break it remains protected by canonical replay and retention, not by a completed PQ epoch proof. |
| `Future verified branch` | A later backend-promotion stage after proof object, verifier API, codec, negative tests, benchmarks, security review, and rollback rules exist. |
| `Prior-output binding` | The recursive chain rule where a previous proof output root equals the next statement previous root. |
| `Backend label` | A configured identifier for the proof lane. In the active profile the required labels are `nova_compressed_v1` and `plonky3_stark_epoch_v1`; local test labels cannot be promoted. |
| `PostQuantumCheckpointAnchorV1` | Epoch audit envelope emitted on configured cadence blocks after live enforcement starts. In this profile it binds Plonky3 epoch proof material, canonical archive roots, and the optional Nova chain root. |
| `PQ cadence` | The configured block interval, default `1000`, where a Plonky3 epoch proof and PQ anchor are required once `authority_promotion.stage >= pq_anchor_writer`. |
| `RecursiveCheckpointDocumentationPacketV1` | Phase 069 closeout packet containing schemas, vectors, chain evidence, measurements, reject matrix, PQ cadence evidence, and rejected-claim register. |

## 🧭 Current Code Truth

📌 Phase 069 starts from these live facts:

| Current surface | Current truth | Phase 069 implication |
| --- | --- | --- |
| `crates/z00z_storage/src/checkpoint/artifact_final.rs` | `CheckpointArtifact` carries version, height, roots, settlement roots, optional claim root, spent/created deltas, optional snapshot and exec IDs, proof system, and proof payload. | Extend by attachment and sidecar evidence only. Do not bypass the artifact. |
| `crates/z00z_storage/src/checkpoint/artifact_stmt.rs` | `CheckpointTransitionStatementV1` binds current checkpoint roots, settlement roots, optional claim root, deltas, prep snapshot ID, and exec input ID. | Phase 069 must derive recursive public inputs from the checkpoint statement, not from an ad hoc theorem. |
| `crates/z00z_storage/src/checkpoint/exec_input.rs` | `CheckpointExecTx` rejects empty input refs, outputs, or `tx_proof`; exact upstream proof bytes are preserved. | Recursive work must not remove or synthesize transaction proof bytes. |
| `crates/z00z_storage/src/checkpoint/artifact_types.rs` | `CheckpointProofSystem::OPAQUE_ATTEST` is live. `CheckpointProofSystem::VERIFIED` is reserved. | Phase 069 cannot enable verified admission. |
| `crates/z00z_storage/src/checkpoint/link.rs` | `CheckpointLink` binds checkpoint ID, prep snapshot ID, and exec input ID with a domain-separated link bind. | Recursive chain evidence must not replace checkpoint links. |
| `crates/z00z_storage/src/checkpoint/mod.rs` | Checkpoint public surface is already a narrow facade over split implementation files. | Add new surfaces through the facade only after ownership is stable. |
| `crates/z00z_storage/src/settlement/` | Settlement owns HJMT roots, journals, proof blobs, witness DAGs, and state lineage. | Witness and delta roots must come from settlement-owned material. |
| Runtime publication seams | Validators, watchers, rollup node, and simulator have publication/binding concepts, but not the exact `CheckpointPublicationEvidenceV1` contract from the reference spec. | Phase 069 may consume a fixture or existing local binding as non-authoritative input, but must not claim Phase 70 publication readiness is implemented. |
| Checkpoint contract config loader | `crates/z00z_storage/src/checkpoint/contract_config.rs` defines `CheckpointContractConfigV1`, loads `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml` through bounded YAML IO, denies unknown fields, validates branch authority, DA, archive retention, snapshots, pruning, PQ cadence, retention, paths, limits, documentation gates, and exposes `is_pq_cadence_height`. | Phase 069 now has a real storage-owned config gate. Runtime writers still must call this validated surface before checkpoint, sidecar, PQ, archive, snapshot, pruning, documentation, or promotion writes. |

## 🧪 Architecture Doublecheck Ledger

📌 Every major architecture decision below was checked against Phase 068,
Phase 069 source files, and live code. If a future implementation deviates, it
MUST update this ledger and add a test that proves the new boundary.

| Decision | Source confirmation | Code truth confirmation | Phase 069 result | Required gate |
| --- | --- | --- | --- | --- |
| Checkpoint contract first, not backend first. | Phase 068 `CheckpointTransitionStatementV1`; Phase 69 audit says exact statement first. | `z00z_storage::checkpoint` owns artifact, statement, exec input, link, and codec. | Recursive proof work binds one storage-owned statement. | `RCP-INV-001`, statement golden vectors. |
| Recursive sidecar is non-authoritative hybrid evidence. | Phase 69 proposal requires sidecar evidence and `VERIFIED` disabled; `plonky3-stark.md` selects Nova+Plonky3 roles. | `CheckpointProofSystem::OPAQUE_ATTEST` is live; `VERIFIED` is reserved and codec rejects unsupported proof classes. | Sidecar cannot mutate or admit canonical artifacts; Nova and Plonky3 evidence remains separate from canonical admission until promotion. | Config branch gate plus artifact admission tests. |
| Storage owns contract objects. | Phase 068 says config validator and statement live under storage checkpoint module. | `CheckpointFsStore`, codec, link validation, exec input, and `CheckpointContractConfigV1` live under `crates/z00z_storage/src/checkpoint/`. | Add sidecar envelope, PQ anchor envelope, and digest helpers through the same storage facade; keep config validation storage-owned. | Storage config tests and path gates. |
| Runtime validators consume theorem; they do not redefine it. | Phase 068 forbids duplicate checkpoint theorem. | `crates/z00z_runtime/validators` is crate `z00z_validators`; `SettlementTheoremBundle` verifies artifact, exec input, link, and tx package. | Validators may reject sidecar authority but MUST NOT own recursive theorem. | Validator non-authority integration tests. |
| Rollup node wires lifecycle and DA adapters. | Phase 068 keeps provider SDKs behind DA/export adapter boundary. | `crates/z00z_rollup_node/src/da.rs` has `DaAdapter`, `LocalDaAdapter`, publication binding, and resolve path. | Rollup node loads/passes config and DA evidence; it does not define statement bytes. | DA SDK leakage tests. |
| Watchers observe publication readiness only. | Phase 068 says watcher evidence is not settlement authority. | `z00z_watchers::PublicationWatch` checks runtime, validator, and storage bindings. | Watchers can report readiness/gaps, not recursive validity. | Publication readiness and no-authority tests. |
| Real recursive backend crate is allowed after storage contract gates. | User-approved architecture selects real implementation targets, not placeholder scaffolds. | No current `z00z_recursive_proofs` crate exists; storage config is already the authority gate. | A future crate MAY own Nova and Plonky3 adapters, but storage still owns statement bytes, artifact codecs, path gates, and reject taxonomy. | Implementation slices `069-05` through `069-08`. |
| Nova compressed proof runs every block. | `plonky3-stark.md` resolves Nova as fast classical lane. | Config now contains `branches.nova.cadence_blocks: 1`, `proof_system: nova_compressed_v1`, and `is_pq_authoritative: false`. | Every checkpoint block should be able to emit a Nova proof binding statement digest, checkpoint link, prior Nova output, and output root. | Nova branch config tests and chain tests. |
| Plonky3 recursive STARK locks every epoch. | `plonky3-stark.md` resolves Plonky3/STARK as primary PQ-friendly epoch lane. | Config now contains `branches.plonky3_epoch.cadence_blocks: 1000`, `proof_system: plonky3_stark_epoch_v1`, `field: koala_bear`, `hash: poseidon2`, `security_bits: 124`, `recursion_library: p3_recursion`. | At every configured positive cadence height, emit a Plonky3 epoch statement/proof/anchor that binds the canonical 1000-block range. | Cadence tests at heights 999 and 1000; Plonky3 config tests. |
| Plonky3 MUST NOT depend only on Nova. | Quantum attack can break ECC/Nova and then a STARK wrapper over Nova verifier would only prove false classical proofs verify. | Config has `must_prove_canonical_transition_range: true` and `must_not_depend_only_on_nova: true`. | Plonky3 epoch proof may bind `nova_chain_root`, but PQ soundness must come from canonical transition range, roots, witnesses, deltas, and archive commitments. | `Plonky3DependsOnlyOnNova` negative test. |
| PQ epoch artifact budget is explicit. | Plonky3/STARK proofs are larger than ECC/Nova; per-block STARK retention is too expensive. | Config limits reserve 8 MiB recursive proof cap, 16 MiB Plonky3 epoch proof/PQ anchor caps, 128 KiB Nova block proof cap, and 128 MiB Nova epoch archive cap. | Store final epoch proof and manifest permanently; retain Nova block proofs through the PQ epoch window; avoid permanent storage of every internal STARK aggregation proof. | Proof-size cap tests and retention tests. |
| Celestia is DA, not forever archive. | Phase 068 separates DA reference from archive manifest; Celestia-style DA availability does not imply indefinite historical retrieval. | Config has `archive_retention.celestia_is_da_only: true` and long-term retrieval gates. | DA publication can start challenge timing, but Z00Z archive retention owns long-term retrieval. | Celestia-as-archive negative config test. |
| Recursive proofs do not replace retrievability. | Recursive proof validates a statement; it does not store raw tx packages, witness bytes, exact proof bytes, explorer data, or migration material. | Config requires archive manifests, provider receipts, retrieval audits, and snapshots in addition to Nova/Plonky3 proof objects. | Full nodes may prune locally only after archive/snapshot gates; archive replicas must not prune. | Archive retention and pruning tests. |
| IPFS requires pinning and receipts. | Content addressing alone only names bytes; it does not guarantee they remain hosted. | Config requires `ipfs_pinning_required: true`, provider receipts, and retrieval audits. | IPFS may be one backend, but never the only persistence guarantee without pins and audits. | IPFS-without-pinning negative test. |
| Exact tx proof bytes remain retained. | Phase 068 retention gate and Phase 69 audit require raw/witness retention. | `CheckpointExecTx::new` rejects empty `tx_proof` and stores exact bytes. | Recursive proofs cannot remove replay material. | Storage integration test. |
| Config YAML is a runtime gate, not documentation. | Phase 068 requires `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml` and startup validation; Phase 069 adds Nova and Plonky3 branch gates. | `z00z_storage::checkpoint::contract_config` now implements strict schema validation for this exact file; `z00z_rollup_node::config` has other YAML loaders and may call or mirror storage validation for startup reporting. | The repository config is active architecture state, not a decorative fixture. Runtime call-sites that write checkpoint-family artifacts must use the validated storage contract. | `cargo test -p z00z_storage checkpoint::contract_config -- --nocapture`. |

## 🎯 Scope

### ✅ In Scope

- Define the Phase 069 recursive sidecar data contract.
- Define `RecursiveCheckpointPublicInputV1` canonical binary bytes.
- Define `RecursiveCheckpointWitnessV1` local witness fixture requirements.
- Define `NovaCompressedBlockProofV1`, `Plonky3EpochProofV1`,
  `EpochRangeStatementV1`, and strict codec rules.
- Define `RecursiveCheckpointSidecarV1` and storage attachment rules.
- Define `RecursiveCheckpointVerifierV1` and Nova/Plonky3 proof-adapter
  semantics.
- Define `RecursiveCheckpointRejectReasonV1` with deterministic failure outputs.
- Define `RecursiveCheckpointMeasurementV1` and benchmark metadata requirements.
- Build deterministic 3 to 5 step chain evidence over checkpoint statements,
  with Nova per-block proof semantics.
- Prove canonical checkpoint admission is unchanged by recursive sidecars.
- Preserve exact transaction proof bytes and witness/archive obligations.
- Define `PostQuantumCheckpointAnchorV1` and `EpochManifestV1` enough for
  Phase 069 fixtures, cadence tests, and closeout audit evidence.
- Define Plonky3 epoch proof cadence, public inputs, proof-size limits,
  retention, and negative gates.
- Define Nova block proof cadence, per-block chain root, retention, and negative
  gates.
- Define the Archive Retention Layer that treats Celestia as DA-only and
  requires long-term retrieval through content-addressed archive manifests,
  pinned IPFS or equivalent backends, archive provider receipts, and retrieval
  audits.
- Define `StateSnapshotV1` enough for future bootstrap from verified snapshots
  without replaying from genesis.
- Define full-node pruning gates that allow local deletion only after dispute,
  Plonky3 epoch finality, epoch manifest finality, archive replication, and
  retrieval audit success.
- Define the required `RecursiveCheckpointDocumentationPacketV1` closeout
  content.
- Add tests, fixtures, and scenario evidence for all gates in this spec.

### 🚫 Out Of Scope

- Treating Nova, SuperNova, Fractal, HyperPlonk, LatticeFold, RLWE, or generic
  STARK claims as production truth outside the selected Nova+Plonky3 profile.
- Treating Nova compressed proofs as PQ authority.
- Treating a Plonky3 epoch proof that only wraps Nova verifier execution as
  PQ-finality.
- Permanently storing every internal per-block STARK aggregation proof as the
  default consensus retention path.
- Enabling `CheckpointProofSystem::VERIFIED` for canonical admission.
- Replacing current spend verification, range-proof verification, transaction
  package verification, or checkpoint replay verification.
- Removing exact transaction proof bytes from `CheckpointExecInput`.
- Treating link tags as recursive proof authority.
- Claiming 100 byte recursive proofs, 200 KB active state, or Mina-equivalent
  state size without measured backend evidence.
- Treating DA publication, watcher evidence, or publication readiness as
  state-transition validity.
- Treating Celestia DA as permanent historical storage.
- Treating IPFS CID publication without pinning, provider receipts, and
  retrieval audits as archival persistence.
- Treating recursive proofs or Plonky3 epoch proofs as a replacement for all
  retrievable raw/witness/archive material.
- Allowing archive nodes to prune retained history in the default profile.
- Claiming post-quantum recursive security from classical commitments,
  discrete-log assumptions, unproved folding sketches, or Nova proof validity.
- Implementing weak subjectivity, fraud economics, bridge support, or production
  DA provider integration as Phase 069 closure requirements.

## 🔒 Non-Negotiable Invariants

| Invariant | Requirement | Proof surface |
| --- | --- | --- |
| `RCP-INV-001 Statement First` | Every recursive object MUST bind the exact `CheckpointTransitionStatementV1` digest. | Codec tests, golden vectors, chain tests. |
| `RCP-INV-002 Same Theorem` | A backend MUST NOT introduce a second checkpoint theorem. | Public input tests, backend manifest review. |
| `RCP-INV-003 Shadow Only` | Recursive sidecars MUST remain non-authoritative in Phase 069. | Config tests, artifact admission tests. |
| `RCP-INV-004 Canonical Replay` | Exact `tx_proof` bytes MUST remain in canonical replay. | Storage integration tests. |
| `RCP-INV-005 Prior Binding` | Each chained step MUST prove prior-output binding. | 3-step and 5-step chain tests. |
| `RCP-INV-006 Strict Codec` | Version, length, digest, backend label, and unknown-field rules MUST fail closed. | Unit and fuzz tests. |
| `RCP-INV-007 Measurements Honest` | Measurements MUST be local evidence only, not production proof claims. | Measurement validation tests. |
| `RCP-INV-008 PQ Honesty` | PQ material is an audit/evaluation envelope only in Phase 069. | Red-line tests and docs audit. |
| `RCP-INV-009 Retention` | Raw, witness, delta, exact proof, and archive material MUST remain available through configured windows. | Retention tests and fixture checks. |
| `RCP-INV-010 No SDK Leakage` | Provider SDK types MUST NOT enter recursive statement or public input bytes. | Digest input tests. |
| `RCP-INV-011 PQ Cadence` | Every positive height divisible by configured `cadence_blocks` MUST have a complete Plonky3 epoch proof and PQ anchor once live cadence enforcement is active. | Config tests, cadence tests, audit packet. |
| `RCP-INV-012 Recursive Docs` | Phase 069 MUST leave enough stable docs, vectors, and rejection evidence for a future backend review without re-reading research chats. | Documentation packet and source audit. |
| `RCP-INV-013 Nova Classical Only` | Nova compressed proofs MUST NOT be described or enforced as PQ authority. | Config tests, backend manifest tests, docs audit. |
| `RCP-INV-014 Plonky3 Canonical Range` | A Plonky3 epoch proof MUST prove the canonical transition range, not only Nova verifier acceptance. | Epoch statement tests and negative backend manifest tests. |
| `RCP-INV-015 Epoch Finality Window` | Blocks after the last completed Plonky3 cadence height are an open epoch and MUST be treated as not yet PQ-finalized. | Cadence tests and operator status tests. |
| `RCP-INV-016 Hybrid No Downgrade` | Disabling either the Nova branch or Plonky3 epoch branch in the default profile MUST reject config. | Config negative tests. |
| `RCP-INV-017 Proof Size Budgets` | Nova and Plonky3 proof objects MUST obey explicit byte caps and target/cap reporting. | Codec tests, limit tests, measurement tests. |
| `RCP-INV-018 DA Is Not Archive` | Celestia or any DA layer MUST NOT be treated as indefinite historical storage. | Config tests and docs audit. |
| `RCP-INV-019 Retrievability Is Not Validity` | Archive receipts and retrieval audits prove bytes are retrievable; they MUST NOT prove state-transition validity. | Validator and watcher non-authority tests. |
| `RCP-INV-020 Recursive Proof Is Not Storage` | Nova and Plonky3 proofs MAY justify local pruning after gates, but they MUST NOT remove network-level archive obligations. | Pruning tests and retention tests. |
| `RCP-INV-021 Snapshot Bootstrap` | `StateSnapshotV1` MUST bind the latest Plonky3 epoch proof, epoch manifest, archive manifest, state root, settlement root, chunk root, and PQ anchor root. | Snapshot binding tests. |
| `RCP-INV-022 Archive Replication Before Pruning` | Local full-node pruning MUST require archive replication threshold and retrieval audit success; archive-node pruning MUST reject. | Archive retention and pruning config tests. |

## 🧱 Statement Contract

📌 Phase 069 consumes the checkpoint statement defined by the recursive-ready
checkpoint contract. It MUST NOT define a smaller alternate statement.

```text
CheckpointTransitionStatementV1:
  height
  + prev_root
  + prev_settlement_root
  + checkpoint_exec_input_id
  + prep_snapshot_id
  + ordered tx_data_root
  + delta_root
  + witness_root
  + journal_digest
  + da_ref
  + optional claim_root
  + optional prior_recursive_output_root
  + optional pq_anchor_root
  -> new_root
  + new_settlement_root
```

📌 Phase 069 statement rules:

- The statement digest domain MUST be `z00z.checkpoint.transition.v1`.
- The statement digest MUST include version, domain, field names, chain context,
  proof-system family, and length-delimited canonical field bytes.
- The same final `statement_digest_v1` MUST be used by canonical artifacts,
  recursive sidecars, and future verified backends.
- `statement_core_digest_v1` MUST bind the checkpoint theorem before DA and PQ
  evidence.
- `statement_digest_v1` MUST bind `statement_core_digest_v1`, `da_ref`, and
  optional future-era `pq_anchor_root` according to the checkpoint contract.
- Human-readable JSON, YAML, report paths, temp paths, hostnames, and local
  operator metadata MUST NOT be authoritative digest inputs.
- Missing optional fields MUST be encoded as explicit absence.
- V1 canonical admission MUST keep `pq_anchor_root` absent.
- `PostQuantumCheckpointAnchorV1` is external audit evidence in Phase 069. Its
  root MUST NOT be embedded into a V1 canonical admission artifact.
- A non-absent `pq_anchor_root` in a V1 canonical-admission path MUST reject as
  `PqInlineAnchorUnsupported`.
- If Phase 068 has not yet exposed one live field, Phase 069 tests MAY use a
  typed fixture with an explicit `fixture_only` label, but runtime code MUST
  fail closed rather than silently omitting the field.

## 🔢 Canonical Byte Contract

📌 Every Phase 069 committed object MUST have canonical binary bytes.

| Rule | Requirement |
| --- | --- |
| Versioning | Every committed object MUST include an explicit version. |
| Framing | Every variable-length field MUST be length-delimited. |
| Field order | Field order MUST be defined by this spec, not by serializer declaration order. |
| Optional fields | Optional fields MUST encode present or absent explicitly. |
| Collections | Ordered collections preserve semantic order; unordered collections sort by canonical key. |
| Paths | Local filesystem paths MUST NOT be digest inputs. |
| Unknown fields | Unknown fields reject in authoritative codecs. |
| Golden vectors | Every digest and root introduced by Phase 069 MUST ship with golden vectors. |

📌 Phase 069 MUST use the storage-owned 32-byte domain-separated digest helper
family already used by checkpoint IDs and storage proof binds unless a new
versioned statement era is created.

## 🧩 Public Input Contract

📌 `RecursiveCheckpointPublicInputV1` is the proof-facing public input. It is
derived from `CheckpointTransitionStatementV1`; it is not a new theorem.

```yaml
recursive_checkpoint_public_input_v1:
  version: 1
  statement_digest: "0x..."
  statement_core_digest: "0x..."
  height: 0
  chain_index: 0
  chain_length: 5
  epoch_index: 0
  epoch_start_height: 1
  epoch_end_height: 1000
  prev_root: "0x..."
  output_root: "0x..."
  prior_output_root: "0x..."
  delta_root: "0x..."
  witness_root: "0x..."
  checkpoint_link_digest: "0x..."
  backend_label: nova_compressed_v1
  verifier_params_digest: "0x..."
  proof_mode: fast_classical_compressed
```

📌 Public input rules:

- `statement_digest` MUST equal the canonical checkpoint statement digest.
- `statement_core_digest` MUST equal the statement core digest from the
  checkpoint contract.
- `prev_root` MUST equal the statement `prev_root`.
- `output_root` MUST equal the statement `new_root`.
- `prior_output_root` MUST equal the statement `prev_root` for the current step.
- `delta_root` and `witness_root` MUST be copied from the statement, not rebuilt
  from sidecar-local bytes.
- `checkpoint_link_digest` MUST bind the storage-owned checkpoint link for the
  same statement.
- `backend_label` MUST be one of the configured backend labels:
  `nova_compressed_v1` for per-block proof input or
  `plonky3_stark_epoch_v1` for epoch proof input.
- `proof_mode` MUST be `fast_classical_compressed` for Nova block proofs and
  `pq_epoch_finality` for Plonky3 epoch proofs.
- `chain_length` MUST be at least 3 and no more than 5 for required local
  evidence unless a later spec changes the target.
- Public input digest MUST be computed from canonical public input bytes, not
  from proof bytes or measurement payloads.
- A local test profile MAY use `recursive_mock_v1`, but only behind a test-only
  config fixture that cannot pass the repository `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`
  validator.

## 📦 Witness Contract

📌 `RecursiveCheckpointWitnessV1` is local proof input material. It exists to
drive mock and future candidate adapters. It is not canonical checkpoint truth.

```yaml
recursive_checkpoint_witness_v1:
  version: 1
  statement_digest: "0x..."
  exec_input_id: "0x..."
  prep_snapshot_id: "0x..."
  tx_data_root: "0x..."
  delta_root: "0x..."
  witness_root: "0x..."
  journal_digest: "0x..."
  witness_package_digest: "0x..."
  archive_manifest_root: "0x..."
  checkpoint_link_digest: "0x..."
  source: local_fixture_or_archive
```

📌 Witness rules:

- The witness MUST bind the same statement digest as the public input.
- The witness MUST reference exact execution input and prep snapshot IDs.
- The witness MUST preserve exact transaction proof bytes through the canonical
  storage path.
- The witness MUST include enough fixture material to reproduce valid and
  tampered local transitions.
- The witness MUST NOT contain wallet private keys, plaintext secrets, or
  provider SDK-native receipts.
- The witness MAY use archive references for large data, but those references
  MUST bind content digests and byte lengths.
- A missing witness package MUST reject sidecar creation.
- Witness fixture generation MUST be deterministic.

## 🧬 Hybrid Backend Architecture

📌 Phase 069 uses a two-lane recursive architecture:

| Lane | Cadence | Backend | Security role | Storage role |
| --- | --- | --- | --- | --- |
| Nova block lane | Every block | `nova_compressed_v1` | Fast classical recursion and local proof continuity. Not PQ. | Stored under `paths.nova_block_proofs`; retained at least until the enclosing Plonky3 epoch is finalized. |
| Plonky3 epoch lane | Every `post_quantum.cadence_blocks`, default `1000` | `plonky3_stark_epoch_v1` using `p3_recursion` | PQ-friendly epoch history lock under STARK/FRI/hash assumptions. | Stored under `paths.plonky3_epoch_proofs` with permanent epoch manifest metadata. |

📌 Architecture rules:

- The canonical checkpoint artifact remains the source of state-transition
  truth.
- Nova proofs MUST bind canonical statement digests and checkpoint links.
- Plonky3 epoch proofs MUST bind an ordered 1000-block canonical transition
  range when default cadence is used.
- Plonky3 MAY bind `nova_chain_root` as a consistency/audit input.
- Plonky3 MUST NOT rely only on Nova proof verification for PQ-finality.
- A block inside an open epoch has canonical replay plus Nova evidence, but it
  is not yet Plonky3/PQ-finalized until the epoch proof lands.
- Per-block STARK proofs are not the default permanent retention path. The
  permanent record is the final epoch proof plus `EpochManifestV1` and archive
  commitments.

📌 Backend decision matrix:

| Backend family | Phase 069 role | Why | Reject boundary |
| --- | --- | --- | --- |
| Nova / Arecibo | Required fast classical per-block lane and benchmark baseline. | Direct IVC fit, small compressed proofs, good model for step-by-step checkpoints. | Not PQ; cannot be final long-term history authority. |
| Plonky3 + Plonky3-recursion | Primary real implementation target for epoch PQ lane. | Rust STARK stack, recursive STARK verification support, transparent setup, field/hash choices aligned with Plonky3 recursion examples. | Recursion code is active-development/unaudited; cannot enable canonical `VERIFIED` without later audit/promotion gates. |
| SuperNova | Future comparison if checkpoint steps become materially non-uniform. | Better fit for non-uniform IVC than plain Nova. | Not active Phase 069 target; classical only. |
| Fractal | Design reference for transparent recursive proof reasoning. | Strong theoretical relevance. | C++/academic PoC, not Z00Z implementation base. |
| HyperPlonk | Future prover/circuit comparison only. | Rust and lookup/high-degree gate relevance. | Not recursion-first and not selected. |
| LatticeFold/RLWE | PQ research track. | Relevant to future lattice folding and commitment research. | Not live backend; no production parameters/audit/ABI in Phase 069. |
| Generic STARK/Winterfell | Fallback design vocabulary. | Transparent proof family. | Plonky3-recursion is the selected STARK path; standalone STARK claims are insufficient. |

## ⚡ Nova Block Proof Contract

📌 `NovaCompressedBlockProofV1` is emitted for each block when
`branches.nova.is_enabled` is true.

```yaml
nova_compressed_block_proof_v1:
  version: 1
  proof_system: nova_compressed_v1
  mode: fast_classical_compressed
  height: 42
  statement_digest: "0x..."
  public_input_digest: "0x..."
  checkpoint_link_digest: "0x..."
  prior_nova_output_root: "0x..."
  nova_output_root: "0x..."
  prior_output_root: "0x..."
  output_root: "0x..."
  verifier_params_digest: "0x..."
  proof_bytes_digest: "0x..."
  proof_bytes: "0x..."
```

📌 Nova block proof rules:

- `height` MUST match the checkpoint statement height.
- `statement_digest` MUST match `CheckpointTransitionStatementV1`.
- `checkpoint_link_digest` MUST match the storage-owned checkpoint link.
- `prior_output_root` and `output_root` MUST match the checkpoint roots.
- `prior_nova_output_root` MUST equal the previous block's `nova_output_root`
  for contiguous chain evidence.
- `proof_system` MUST be `nova_compressed_v1`.
- `mode` MUST be `fast_classical_compressed`.
- `proof_bytes` MUST be non-empty and capped by
  `limits.max_nova_block_proof_bytes`.
- Nova proof digests for an epoch MUST be retained until the Plonky3 epoch proof
  and `EpochManifestV1` are finalized.
- Nova MUST NOT be used as PQ authority. Any config or proof object that sets
  Nova as `is_pq_authoritative: true` MUST reject.

📌 Target and cap:

| Quantity | Target | Hard cap |
| --- | --- | --- |
| One compressed Nova block proof | 8 KiB to 30 KiB after Z00Z measurement target | 128 KiB via `max_nova_block_proof_bytes` |
| 1000 Nova proofs archive window | 8 MiB to 30 MiB target | 128 MiB via `max_epoch_nova_archive_bytes` |

## 🛡️ Plonky3 Epoch Proof Contract

📌 `EpochRangeStatementV1` is the public epoch statement for the Plonky3 proof.

```yaml
epoch_range_statement_v1:
  version: 1
  proof_system: plonky3_stark_epoch_v1
  mode: pq_epoch_finality
  epoch_index: 0
  start_height: 1
  end_height: 1000
  cadence_blocks: 1000
  prev_pq_anchor_root: "0x..."
  start_root: "0x..."
  end_root: "0x..."
  statement_digest_root: "0x..."
  checkpoint_link_root: "0x..."
  delta_root: "0x..."
  witness_root: "0x..."
  da_archive_manifest_root: "0x..."
  nova_chain_root: "0x..."
  field: koala_bear
  hash: poseidon2
  recursion_library: p3_recursion
  security_bits: 124
```

📌 `Plonky3EpochProofV1` is emitted at each completed epoch boundary.

```yaml
plonky3_epoch_proof_v1:
  version: 1
  proof_system: plonky3_stark_epoch_v1
  mode: pq_epoch_finality
  epoch_statement_digest: "0x..."
  public_inputs_digest: "0x..."
  epoch_index: 0
  start_height: 1
  end_height: 1000
  cadence_blocks: 1000
  proves_canonical_transition_range: true
  depends_only_on_nova: false
  binds_nova_chain_root: true
  verifier_params_digest: "0x..."
  proof_bytes_digest: "0x..."
  proof_bytes: "0x..."
```

📌 Plonky3 epoch rules:

- `end_height - start_height + 1` MUST equal `cadence_blocks` for a completed
  default epoch unless a later profile explicitly changes partial-epoch rules.
- `end_height` MUST be a positive PQ cadence height.
- `statement_digest_root` MUST commit to every ordered checkpoint statement in
  the epoch.
- `checkpoint_link_root` MUST commit to the corresponding checkpoint links.
- `delta_root`, `witness_root`, and `da_archive_manifest_root` MUST commit to
  retained canonical replay material.
- `nova_chain_root` MAY be present and SHOULD bind the ordered Nova proof
  digests for the epoch.
- `proves_canonical_transition_range` MUST be true.
- `depends_only_on_nova` MUST be false.
- `field` MUST be `koala_bear`, `hash` MUST be `poseidon2`, and
  `recursion_library` MUST be `p3_recursion` for the default profile.
- `security_bits` MUST be at least `124`.
- Proof bytes MUST be capped by `limits.max_plonky3_epoch_proof_bytes`.
- Sidecar/report bytes for the epoch MUST be capped by
  `limits.max_plonky3_epoch_sidecar_bytes`.
- Plonky3-recursion's active-development/unaudited status MUST be recorded in
  the backend manifest. This blocks canonical `VERIFIED` promotion until a
  later review, but it does not block Phase 069 from targeting the real
  implementation architecture.

📌 Target and cap:

| Quantity | Target | Hard cap |
| --- | --- | --- |
| Final Plonky3 epoch proof for 1000 blocks | 0.5 MiB to 4 MiB after Z00Z measurement target | 16 MiB via `max_plonky3_epoch_proof_bytes` |
| Final PQ anchor / epoch artifact | 0.5 MiB to 4 MiB target | 16 MiB via `max_pq_anchor_bytes` |
| Amortized permanent PQ storage per block | 0.5 KiB to 4 KiB target | 16 KiB per block budget implied by the 16 MiB epoch cap |

## 📦 Epoch Manifest Contract

📌 `EpochManifestV1` is the retained index for one completed epoch.

```yaml
epoch_manifest_v1:
  version: 1
  epoch_index: 0
  start_height: 1
  end_height: 1000
  cadence_blocks: 1000
  statement_digest_root: "0x..."
  checkpoint_artifact_root: "0x..."
  checkpoint_link_root: "0x..."
  da_archive_manifest_root: "0x..."
  witness_root: "0x..."
  delta_root: "0x..."
  nova_chain_root: "0x..."
  plonky3_epoch_statement_digest: "0x..."
  plonky3_epoch_proof_digest: "0x..."
  retention:
    nova_block_proofs: archive_until_pq_epoch
    plonky3_epoch_proofs: permanent_metadata
    epoch_manifests: permanent_metadata
```

📌 Manifest rules:

- The manifest MUST bind all statement digests and canonical artifact digests in
  the epoch.
- The manifest MUST bind the DA/archive root and witness/delta roots used by the
  Plonky3 epoch proof.
- The manifest MUST bind `nova_chain_root` when Nova proofs exist for the epoch.
- The manifest MUST bind the Plonky3 epoch statement and proof digests.
- A manifest missing any configured required artifact MUST reject.
- Manifest metadata is permanent. Large proof/witness/archive data follows the
  retention policy and content-addressed archive roots.

## 🗄️ Long-Term Archive Retention Contract

📌 Celestia and equivalent DA layers are publication and availability layers,
not indefinite history stores. Phase 069 MUST preserve this separation:

```text
DA layer                 -> data was published and available in the DA window
Archive Retention Layer  -> bytes remain retrievable after the DA window
Recursive proof layer    -> canonical transition validity is compactly proven
Snapshot layer           -> new nodes can bootstrap from a compact verified base
Pruning policy           -> ordinary full nodes can delete local bytes after gates
```

📌 `CheckpointArchiveManifestV1` is permanent compact metadata for one checkpoint
or epoch archive package.

```yaml
checkpoint_archive_manifest_v1:
  version: 1
  statement_digest: "0x..."
  epoch_manifest_root: "0x..."
  raw_tx_package_root: "0x..."
  exact_tx_proof_bytes_root: "0x..."
  witness_archive_root: "0x..."
  delta_journal_root: "0x..."
  da_payload_commitment: "0x..."
  archive_provider_receipt_root: "0x..."
  retrieval_audit_root: "0x..."
  content_address_root: "0x..."
  min_archive_replicas: 3
```

📌 `ArchiveProviderReceiptV1` records one configured archival backend.

```yaml
archive_provider_receipt_v1:
  version: 1
  backend: ipfs_pinned
  content_cid_or_digest: "0x..."
  byte_length: 0
  provider_identity_digest: "0x..."
  receipt_digest: "0x..."
  pinned: true
  paid_or_operator_committed: true
```

📌 `RetrievalAuditV1` records periodic retrieval evidence.

```yaml
retrieval_audit_v1:
  version: 1
  height: 1000
  interval_blocks: 1000
  archive_manifest_root: "0x..."
  requested_entries_root: "0x..."
  successful_receipts_root: "0x..."
  failed_receipts_root: "0x..."
  successful_replica_count: 3
  passed: true
```

📌 Archive retention rules:

- `archive_retention.celestia_is_da_only` MUST be true.
- DA publication readiness MAY start challenge timing, but MUST NOT satisfy
  long-term retrieval by itself.
- Every archive entry MUST be content addressed by digest or CID and bound by
  byte length, retention class, object type, and ordinal.
- IPFS MAY be used only as `ipfs_pinned`; plain CID publication without pinning
  MUST reject.
- At least `archive_retention.min_archive_replicas` independent archive receipts
  MUST exist before local full-node pruning.
- Retrieval audits MUST run every
  `archive_retention.retrieval_audit_interval_blocks`, default equal to PQ
  cadence `1000`.
- Retrieval audit success MUST NOT be treated as state validity. It only proves
  required bytes are still retrievable.
- Recursive proofs MAY reduce replay cost, but they MUST NOT remove the network
  obligation to keep enough raw/witness/archive bytes for audit, migration,
  recovery, and bug-forensics windows.

📌 Allowed archive backend classes in the default profile:

| Backend | Allowed role | Required gate |
| --- | --- | --- |
| `z00z_archive_node` | First-party or community archive replica. | Must serve content-addressed objects and retrieval audit challenges. |
| `ipfs_pinned` | CID-addressed replication. | Must be pinned and accompanied by provider/operator receipt. |
| `paid_archival_provider` | Commercial archival storage or DA archival endpoint. | Must provide receipt and retrieval proof material. |
| `filecoin_or_equivalent` | Incentivized storage backend. | Must bind storage deal/receipt to content digest and byte length. |
| `cold_object_store` | S3-compatible or offline cold archive. | Must provide digest-bound manifest and retrieval audit result. |

## 📸 State Snapshot Contract

📌 `StateSnapshotV1` is a bootstrap object, not a validity shortcut.

```yaml
state_snapshot_v1:
  version: 1
  height: 10000
  cadence_epochs: 10
  cadence_blocks: 10000
  state_root: "0x..."
  settlement_root: "0x..."
  last_plonky3_epoch_proof_digest: "0x..."
  last_epoch_manifest_root: "0x..."
  archive_manifest_root: "0x..."
  snapshot_chunk_root: "0x..."
  pq_anchor_root: "0x..."
  retrieval_audit_root: "0x..."
```

📌 Snapshot rules:

- Snapshot cadence MUST be a positive multiple of PQ cadence.
- The default profile uses `cadence_epochs: 10` and `cadence_blocks: 10000`.
- A snapshot MUST bind state root and settlement root.
- A snapshot MUST bind the latest completed Plonky3 epoch proof digest.
- A snapshot MUST bind the latest epoch manifest root.
- A snapshot MUST bind archive manifest root and retrieval audit root.
- A snapshot MUST bind snapshot chunk root so chunks can be content-addressed
  and independently retrieved.
- A snapshot MUST bind PQ anchor root.
- Bootstrap from snapshot MAY be used by new nodes only if archive retrieval
  audit has passed and compact metadata remains available.

## ✂️ Pruning Contract

📌 Pruning is local full-node storage relief, not network history deletion.

```yaml
pruning_decision_v1:
  version: 1
  node_class: full_node
  prune_scope: local_full_node_only
  target_epoch: 10
  dispute_window_elapsed: true
  plonky3_epoch_finalized: true
  epoch_manifest_finalized: true
  archive_replication_threshold_met: true
  retrieval_audit_passed: true
  compact_metadata_retained: true
  state_snapshot_retained: true
```

📌 Pruning rules:

- Full-node pruning MAY be allowed after all configured gates pass.
- Archive-node pruning MUST reject in the default Phase 069 profile.
- Local pruning MUST keep compact metadata, epoch manifests, state snapshots,
  PQ anchors, and archive manifests.
- Local pruning MUST NOT delete the last `pruning.min_retain_recent_epochs`
  epochs.
- Local pruning MUST NOT reduce archive replicas below
  `archive_retention.min_archive_replicas`.
- A pruning decision before dispute window elapsed MUST reject.
- A pruning decision before Plonky3 epoch proof finality MUST reject.
- A pruning decision before epoch manifest finality MUST reject.
- A pruning decision before archive replication threshold and retrieval audit
  success MUST reject.

## 🧪 Proof Object Contract

📌 `RecursiveCheckpointProofV1` is a common envelope for one proof artifact.
Concrete payloads are `NovaCompressedBlockProofV1` or `Plonky3EpochProofV1`.
`recursive_mock_v1` MAY exist only in local tests and MUST NOT pass the default
repository config.

```yaml
recursive_checkpoint_proof_v1:
  version: 1
  mode: fast_classical_compressed
  backend_label: nova_compressed_v1
  statement_digest: "0x..."
  public_input_digest: "0x..."
  prior_output_root: "0x..."
  output_root: "0x..."
  verifier_params_digest: "0x..."
  proof_bytes_digest: "0x..."
  proof_bytes: "0x..."
```

📌 Proof object rules:

- `mode` MUST match the configured branch: `fast_classical_compressed` for Nova
  block proofs or `pq_epoch_finality` for Plonky3 epoch proofs.
- `backend_label` MUST be configured and versioned.
- `statement_digest` MUST match the public input statement digest.
- `public_input_digest` MUST match canonical `RecursiveCheckpointPublicInputV1`
  bytes.
- `prior_output_root` MUST match public input `prior_output_root`.
- `output_root` MUST match public input `output_root`.
- `proof_bytes` MUST be non-empty for Nova, Plonky3, and any local test
  adapter.
- `proof_bytes_digest` MUST be computed from canonical proof bytes only.
- Proof bytes over the relevant configured limit MUST reject:
  `max_nova_block_proof_bytes`, `max_plonky3_epoch_proof_bytes`, or the generic
  `max_recursive_proof_bytes` envelope cap.
- A proof object claiming canonical admission or `VERIFIED` authority MUST reject.
- A Plonky3 proof object with `depends_only_on_nova: true` MUST reject as
  `Plonky3DependsOnlyOnNova`.
- A Nova proof object claiming PQ authority MUST reject as
  `NovaPqAuthorityUnsupported`.

## 📎 Sidecar Contract

📌 `RecursiveCheckpointSidecarV1` attaches recursive evidence to a checkpoint
statement without changing checkpoint admission.

```yaml
recursive_checkpoint_sidecar_v1:
  version: 1
  mode: fast_classical_compressed
  statement_digest: "0x..."
  public_input_digest: "0x..."
  checkpoint_id_hint: "0x..."
  proof:
    version: 1
    backend_label: nova_compressed_v1
  verifier_verdict: accepted
  reject_reason: null
  chain_index: 0
  chain_length: 5
  measurements:
    version: 1
```

📌 Sidecar rules:

- A sidecar MUST bind the same statement digest as the canonical artifact.
- A sidecar MAY include `checkpoint_id_hint` for local lookup, but the statement
  digest remains the proof authority for sidecar evidence.
- A sidecar MUST NOT mutate `CheckpointArtifact`, `CheckpointLink`, or
  `CheckpointProofSystem`.
- A sidecar MUST NOT make `CheckpointProofSystem::VERIFIED` admissible.
- A sidecar MUST NOT promote lifecycle status, start challenge timing, or decide
  settlement validity.
- Sidecars MUST be written under the configured recursive sidecar path and MUST
  NOT alias canonical artifact directories.
- Sidecars MUST remain immutable after write. Replacement requires a new object
  identity and a new chain evidence object.

## 🔗 Chain Evidence Contract

📌 `RecursiveCheckpointChainEvidenceV1` proves local recursive chain semantics
over 3 to 5 ordered checkpoint statements.

```yaml
recursive_checkpoint_chain_evidence_v1:
  version: 1
  mode: fast_classical_compressed
  backend_label: nova_compressed_v1
  chain_length: 5
  first_statement_digest: "0x..."
  last_statement_digest: "0x..."
  first_prev_root: "0x..."
  last_output_root: "0x..."
  nova_chain_root: "0x..."
  step_digests:
    - "0x..."
  measurements_root: "0x..."
```

📌 Chain rules:

- Required local evidence MUST include one 3-step chain and one 5-step chain.
- Step order MUST be deterministic.
- `step[i].statement_digest` MUST be unique within a chain.
- `step[i].output_root` MUST equal `step[i + 1].prior_output_root`.
- `step[i].statement.new_root` MUST equal `step[i + 1].statement.prev_root`.
- Step height MUST be strictly increasing.
- Skipped, repeated, or reordered steps MUST reject.
- A chain with a broken prior-output binding MUST reject.
- A chain with missing intermediate statement digest MUST reject.
- Chain evidence MUST be reproducible from persisted sidecars and fixtures.

## ⚙️ Adapter And Verifier API

📌 Phase 069 MUST define proof adapter seams for real target backends. The
storage statement contract remains backend-independent, but the implementation
plan targets Nova and Plonky3 adapters rather than placeholder scaffolds.

```rust
pub trait RecursiveCheckpointProofAdapter {
    fn prove_step(
        &self,
        input: &RecursiveCheckpointPublicInputV1,
        witness: &RecursiveCheckpointWitnessV1,
    ) -> Result<RecursiveCheckpointProofV1, RecursiveCheckpointRejectReasonV1>;

    fn verify_step(
        &self,
        input: &RecursiveCheckpointPublicInputV1,
        proof: &RecursiveCheckpointProofV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1>;

    fn verify_chain(
        &self,
        chain: &RecursiveCheckpointChainEvidenceV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1>;
}

pub trait RecursiveEpochProofAdapter {
    fn prove_epoch(
        &self,
        statement: &EpochRangeStatementV1,
        manifest: &EpochManifestV1,
    ) -> Result<Plonky3EpochProofV1, RecursiveCheckpointRejectReasonV1>;

    fn verify_epoch(
        &self,
        statement: &EpochRangeStatementV1,
        proof: &Plonky3EpochProofV1,
    ) -> Result<(), RecursiveCheckpointRejectReasonV1>;
}
```

📌 API rules:

- The adapter MUST be injected through traits, not hard-coded into checkpoint
  logic.
- The Nova adapter MUST prove and verify per-block compressed proofs over
  `RecursiveCheckpointPublicInputV1`.
- The Plonky3 adapter MUST prove and verify `EpochRangeStatementV1` and
  `Plonky3EpochProofV1`.
- `verify_step` MUST return typed reject reasons, not string-only errors.
- `verify_chain` MUST validate order, roots, statement digests, chain length,
  backend labels, and measurements.
- Production code MUST not use `unwrap()`, undocumented `expect()`, or panic
  control flow for adapter errors.
- Backend adapters MUST live behind explicit crate boundaries and MUST not own
  checkpoint theorem bytes.
- Local mock adapters MAY remain in tests for negative cases, but MUST be
  impossible to enable through the repository config.

## 🚫 Reject Reason Contract

📌 `RecursiveCheckpointRejectReasonV1` MUST be stable enough for deterministic
tests and operator diagnostics.

| Reason | Required trigger |
| --- | --- |
| `UnsupportedVersion` | Object version is unknown. |
| `UnknownField` | Authoritative codec receives unknown field. |
| `StatementDigestMismatch` | Proof, public input, witness, or sidecar binds a different statement. |
| `PublicInputDigestMismatch` | Proof digest does not match public input bytes. |
| `PriorOutputMismatch` | Prior-output root does not match the statement previous root or previous step output. |
| `OutputRootMismatch` | Proof output root does not match statement new root. |
| `BackendUnsupported` | Backend label is not configured. |
| `BackendClaimUnsupported` | Backend claims production, PQ, audited, or verified status without promotion. |
| `ProofBytesEmpty` | Proof bytes are empty. |
| `ProofBytesTooLarge` | Proof bytes exceed configured limit. |
| `NovaPqAuthorityUnsupported` | Nova proof or config claims PQ/final authority. |
| `NovaChainRootMismatch` | Ordered Nova proof digests do not match the epoch `nova_chain_root`. |
| `Plonky3CanonicalRangeMissing` | Plonky3 epoch proof does not prove or bind the canonical transition range. |
| `Plonky3DependsOnlyOnNova` | Plonky3 epoch proof only proves Nova verifier acceptance. |
| `Plonky3UnauditedPromotion` | Plonky3 backend is promoted to canonical verified admission before audit and promotion gates. |
| `HybridCadenceMismatch` | Nova/Plonky3 branch cadence does not match the configured hybrid policy. |
| `EpochManifestIncomplete` | Epoch manifest misses required statement, archive, witness, Nova, or Plonky3 bindings. |
| `ProofSizeBudgetExceeded` | Nova, Plonky3, PQ anchor, or sidecar bytes exceed configured cap. |
| `CelestiaPermanentStorageUnsupported` | Config, docs, or runtime treats Celestia DA as indefinite historical storage. |
| `IpfsPinningMissing` | IPFS/CID archive reference lacks pinning, receipt, or retrieval-audit evidence. |
| `ArchiveReplicationInsufficient` | Archive replicas are below configured threshold. |
| `ArchiveProviderReceiptMissing` | Required archive provider receipt root or backend receipt is missing. |
| `RetrievalAuditMissing` | Required periodic retrieval audit is missing or stale. |
| `RetrievalAuditFailed` | Retrieval audit cannot fetch enough required archive objects. |
| `SnapshotBindingIncomplete` | `StateSnapshotV1` misses state, settlement, Plonky3, epoch manifest, archive manifest, chunk, PQ anchor, or retrieval audit binding. |
| `PruningBeforeArchiveFinality` | Full-node pruning is attempted before dispute, Plonky3, manifest, archive replication, or retrieval audit gates pass. |
| `ArchiveNodePruningUnsupported` | Archive node pruning is enabled or attempted in the default profile. |
| `SidecarAuthoritative` | Sidecar or config attempts to make recursive evidence authoritative. |
| `MeasurementMissing` | Required measurement field is absent. |
| `ChainTooShort` | Chain has fewer than 3 steps. |
| `ChainTooLong` | Chain has more than 5 required evidence steps. |
| `StepSkipped` | Required local chain step is missing. |
| `StepRepeated` | Statement digest or chain index repeats. |
| `StepReordered` | Step order is not deterministic or height is not increasing. |
| `WitnessUnavailable` | Required local witness fixture or archive reference is missing. |
| `CanonicalAdmissionAttempt` | Recursive proof is passed as checkpoint admission authority. |
| `VerifiedCodecMissing` | A verified proof class is used before explicit codec support. |
| `MixedEra` | Statement, proof, sidecar, or codec era mismatch lacks a compatibility adapter. |
| `DaReadinessMissing` | Runtime path requires DA readiness but no provider-neutral evidence is available. |
| `PqInlineAnchorUnsupported` | `pq_anchor_root` is present in a V1 canonical-admission statement or artifact. |
| `PqCadenceDisabled` | PQ policy is disabled under the recursive-ready checkpoint profile. |
| `PqCadenceInvalid` | Cadence is zero, overflows, or is not representable by the config type. |
| `PqLiveCadenceStageMismatch` | Live cadence enforcement is enabled before `pq_anchor_writer` or disabled at/after it. |
| `PqAnchorMissing` | A positive cadence height lacks `PostQuantumCheckpointAnchorV1` while live enforcement is active. |
| `PqAnchorDigestMismatch` | PQ anchor statement, delta, witness, or archive manifest binding mismatches the checkpoint. |
| `PqAnchorIncomplete` | A required PQ anchor artifact field is missing. |
| `RecursiveDocumentationIncomplete` | Closeout lacks required schemas, vectors, reject matrix, measurements, or PQ cadence evidence. |

## 📊 Measurement Contract

📌 `RecursiveCheckpointMeasurementV1` records local evidence only.

```yaml
recursive_checkpoint_measurement_v1:
  version: 1
  backend_label: nova_compressed_v1
  mode: fast_classical_compressed
  chain_length: 5
  epoch_length: 1000
  aggregation_nodes: 999
  proof_family: nova
  security_bits: 0
  proof_bytes: 0
  witness_bytes: 0
  prover_ms: 0
  verifier_ms: 0
  peak_memory_bytes: 0
  statement_bytes: 0
  public_input_bytes: 0
```

📌 Measurement rules:

- Every required field MUST be present before authority-promotion evidence is
  accepted.
- `chain_length` MUST match the chain evidence object.
- `backend_label` MUST match proof and sidecar backend labels.
- Nova measurements MUST be labeled `fast_classical_compressed` and
  `proof_family: nova`.
- Plonky3 measurements MUST be labeled `pq_epoch_finality`,
  `proof_family: stark`, and `security_bits >= 124`.
- A measurement with missing backend label, missing chain length, missing byte
  counts, or undocumented timing units MUST reject.
- Measurements MUST NOT be used to claim final throughput, production security,
  or final proof size before backend-specific benchmark gates pass.
- Benchmark harnesses SHOULD use repeatable local fixtures and Criterion once a
  candidate backend is explicitly evaluated.

## 🛂 Config Gates

📌 Phase 069 uses the storage-owned checkpoint contract config path:

```text
crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml
```

📌 This file is an active runtime gate. Implementations MUST load and validate
it before checkpoint production, sidecar generation, PQ anchor writing,
publication readiness, pruning, or verified-backend promotion. A test fixture is
not enough; the repository file MUST be parseable by the storage-owned
validator.

📌 Full required repository YAML shape:

```yaml
version: 1
profile: checkpoint-contract-recursive-ready-v1
architecture_mode: checkpoint_contract_first

statement:
  version: 1
  domain: z00z.checkpoint.transition.v1
  required_fields:
    - height
    - prev_root
    - new_root
    - prev_settlement_root
    - new_settlement_root
    - checkpoint_exec_input_id
    - prep_snapshot_id
    - tx_data_root
    - delta_root
    - witness_root
    - journal_digest
    - da_ref
  optional_fields:
    - claim_root
    - prior_recursive_output_root
    - pq_anchor_root

branches:
  canonical:
    is_enabled: true
    is_authoritative: true
    proof_system: opaque_attest
    has_exact_tx_proof_bytes: true
    has_checkpoint_link: true
    has_replay_ids: true
  recursive:
    is_enabled: true
    is_authoritative: false
    mode: hybrid_nova_plonky3
    proof_system: recursive_hybrid_v1
    has_prior_output_binding: true
    min_chain_steps: 3
    target_chain_steps: 5
  nova:
    is_enabled: true
    cadence_blocks: 1
    is_authoritative: false
    is_pq_authoritative: false
    mode: fast_classical_compressed
    proof_system: nova_compressed_v1
    has_prior_output_binding: true
    must_bind_statement_digest: true
    must_bind_checkpoint_link: true
    retain_until_pq_epoch: true
  plonky3_epoch:
    is_enabled: true
    cadence_blocks: 1000
    is_authoritative: false
    is_pq_authoritative: true
    mode: pq_epoch_finality
    proof_system: plonky3_stark_epoch_v1
    must_prove_canonical_transition_range: true
    may_bind_nova_chain_root: true
    must_not_depend_only_on_nova: true
    field: koala_bear
    hash: poseidon2
    security_bits: 124
    recursion_library: p3_recursion

authority_promotion:
  stage: spec_only
  recursive_authority_allowed: false
  verified_backend_allowed: false
  allowed_next_stages:
    - config_gate

gates:
  inputs:
    has_statement_fields: true
    has_exec_input_id: true
    has_prep_snapshot_id: true
    has_da_ref: true
    has_exact_tx_proof_bytes: true
  outputs:
    has_checkpoint_artifact: true
    has_checkpoint_link: true
    has_da_export: true
    has_archive_manifest: true
  artifacts:
    has_recursive_sidecar_non_authoritative: true
    has_pq_anchor_on_cadence: true
    has_mixed_era_fail_closed: true

da:
  provider_sdk_boundary: adapter_only
  publication_readiness_gate: required
  challenge_window_start: da_publication_ready
  allowed_sync_modes:
    - da_only
    - hybrid_p2p_da_verified
  provider_families:
    - local_archive
    - sovereign_sdk_adapter
    - celestia_adapter

archive_retention:
  celestia_is_da_only: true
  long_term_retrieval_required: true
  content_addressing_required: true
  ipfs_pinning_required: true
  provider_receipts_required: true
  retrieval_audit_required: true
  retrievability_is_not_validity: true
  min_archive_replicas: 3
  retrieval_audit_interval_blocks: 1000
  allowed_backends:
    - z00z_archive_node
    - ipfs_pinned
    - paid_archival_provider
    - filecoin_or_equivalent
    - cold_object_store
  required_artifacts:
    - archive_manifest_root
    - raw_tx_package_root
    - exact_tx_proof_bytes_root
    - witness_archive_root
    - delta_journal_root
    - da_payload_commitment
    - retrieval_audit_root
    - archive_provider_receipt_root

post_quantum:
  is_enabled: true
  cadence_blocks: 1000
  mode: plonky3_epoch_proof
  enforcement_stage: pq_anchor_writer
  enforce_live_cadence: false
  required_artifacts:
    - pq_statement_digest
    - pq_delta_root
    - pq_witness_root
    - pq_archive_manifest_root
    - plonky3_epoch_statement_digest
    - plonky3_epoch_proof_digest
    - plonky3_public_inputs_digest
    - nova_chain_root
    - pq_signature_or_commitment

snapshots:
  is_enabled: true
  cadence_epochs: 10
  cadence_blocks: 10000
  object_type: state_snapshot_v1
  bootstrap_allowed_from_snapshot: true
  requires_retrieval_audit: true
  must_bind_state_root: true
  must_bind_settlement_root: true
  must_bind_last_plonky3_epoch_proof: true
  must_bind_last_epoch_manifest_root: true
  must_bind_archive_manifest_root: true
  must_bind_snapshot_chunk_root: true
  must_bind_pq_anchor_root: true

pruning:
  full_node_pruning_allowed: true
  archive_node_pruning_allowed: false
  prune_scope: local_full_node_only
  min_retain_recent_epochs: 2
  requires_dispute_window_elapsed: true
  requires_plonky3_epoch_finalized: true
  requires_epoch_manifest_finalized: true
  requires_archive_replication_threshold_met: true
  requires_retrieval_audit_passed: true
  must_keep_compact_metadata: true
  must_keep_epoch_manifest: true
  must_keep_state_snapshot: true
  must_not_prune_archive_replicas: true

retention:
  dispute_window_blocks: 1555200
  challenge_window_start: da_publication_ready
  raw_tx_packages: archive_required
  witness_data: archive_required
  tx_proof_bytes: canonical_until_verified_backend
  nova_block_proofs: archive_until_pq_epoch
  plonky3_epoch_proofs: permanent_metadata
  epoch_manifests: permanent_metadata
  compact_metadata: permanent_metadata
  da_blobs: da_required_until_archive_replicated

paths:
  checkpoint_artifacts: artifacts/checkpoints/final
  checkpoint_links: artifacts/checkpoints/links
  exec_inputs: artifacts/checkpoints/exec_input
  prep_snapshots: artifacts/checkpoints/prep_snapshot
  delta_journals: artifacts/checkpoints/delta_journal
  witness_archives: artifacts/checkpoints/witness_archive
  recursive_sidecars: artifacts/checkpoints/recursive_shadow
  nova_block_proofs: artifacts/checkpoints/nova_block
  pq_checkpoints: artifacts/checkpoints/pq_anchor
  plonky3_epoch_proofs: artifacts/checkpoints/plonky3_epoch
  epoch_manifests: artifacts/checkpoints/epoch_manifest
  archive_manifests: artifacts/checkpoints/archive_manifest
  state_snapshots: artifacts/checkpoints/state_snapshot
  retrieval_audits: artifacts/checkpoints/retrieval_audit
  archive_receipts: artifacts/checkpoints/archive_receipt
  da_exports: artifacts/da/checkpoints
  documentation_packets: artifacts/checkpoints/recursive_docs

limits:
  max_batch_ops: 1000
  max_batch_bytes: 8388608
  max_witness_bytes: 67108864
  max_recursive_proof_bytes: 8388608
  max_recursive_sidecar_bytes: 12582912
  max_nova_block_proof_bytes: 131072
  max_epoch_nova_archive_bytes: 134217728
  max_plonky3_epoch_proof_bytes: 16777216
  max_plonky3_epoch_sidecar_bytes: 25165824
  max_pq_anchor_bytes: 16777216
  max_archive_manifest_bytes: 8388608
  max_state_snapshot_manifest_bytes: 16777216
  max_retrieval_audit_bytes: 4194304
  max_documentation_packet_bytes: 8388608

documentation:
  recursive_packet_required: true
  include_source_disposition: true
  include_object_schemas: true
  include_golden_vectors: true
  include_chain_evidence_ids: true
  include_measurements: true
  include_pq_cadence_evidence: true
  include_backend_manifest: true
  include_rejected_claim_register: true
```

📌 Required recursive subset:

```yaml
branches:
  recursive:
    is_enabled: true
    is_authoritative: false
    mode: hybrid_nova_plonky3
    proof_system: recursive_hybrid_v1
    has_prior_output_binding: true
    min_chain_steps: 3
    target_chain_steps: 5
  nova:
    is_enabled: true
    cadence_blocks: 1
    is_authoritative: false
    is_pq_authoritative: false
    mode: fast_classical_compressed
    proof_system: nova_compressed_v1
    has_prior_output_binding: true
    must_bind_statement_digest: true
    must_bind_checkpoint_link: true
    retain_until_pq_epoch: true
  plonky3_epoch:
    is_enabled: true
    cadence_blocks: 1000
    is_authoritative: false
    is_pq_authoritative: true
    mode: pq_epoch_finality
    proof_system: plonky3_stark_epoch_v1
    must_prove_canonical_transition_range: true
    may_bind_nova_chain_root: true
    must_not_depend_only_on_nova: true
    field: koala_bear
    hash: poseidon2
    security_bits: 124
    recursion_library: p3_recursion

authority_promotion:
  recursive_authority_allowed: false
  verified_backend_allowed: false

gates:
  artifacts:
    has_recursive_sidecar_non_authoritative: true
    has_pq_anchor_on_cadence: true
    has_mixed_era_fail_closed: true

archive_retention:
  celestia_is_da_only: true
  long_term_retrieval_required: true
  content_addressing_required: true
  ipfs_pinning_required: true
  provider_receipts_required: true
  retrieval_audit_required: true
  retrievability_is_not_validity: true
  min_archive_replicas: 3
  retrieval_audit_interval_blocks: 1000
  allowed_backends:
    - z00z_archive_node
    - ipfs_pinned
    - paid_archival_provider
    - filecoin_or_equivalent
    - cold_object_store
  required_artifacts:
    - archive_manifest_root
    - raw_tx_package_root
    - exact_tx_proof_bytes_root
    - witness_archive_root
    - delta_journal_root
    - da_payload_commitment
    - retrieval_audit_root
    - archive_provider_receipt_root

post_quantum:
  is_enabled: true
  cadence_blocks: 1000
  mode: plonky3_epoch_proof
  enforcement_stage: pq_anchor_writer
  enforce_live_cadence: false
  required_artifacts:
    - pq_statement_digest
    - pq_delta_root
    - pq_witness_root
    - pq_archive_manifest_root
    - plonky3_epoch_statement_digest
    - plonky3_epoch_proof_digest
    - plonky3_public_inputs_digest
    - nova_chain_root
    - pq_signature_or_commitment

snapshots:
  is_enabled: true
  cadence_epochs: 10
  cadence_blocks: 10000
  object_type: state_snapshot_v1
  bootstrap_allowed_from_snapshot: true
  requires_retrieval_audit: true
  must_bind_state_root: true
  must_bind_settlement_root: true
  must_bind_last_plonky3_epoch_proof: true
  must_bind_last_epoch_manifest_root: true
  must_bind_archive_manifest_root: true
  must_bind_snapshot_chunk_root: true
  must_bind_pq_anchor_root: true

pruning:
  full_node_pruning_allowed: true
  archive_node_pruning_allowed: false
  prune_scope: local_full_node_only
  min_retain_recent_epochs: 2
  requires_dispute_window_elapsed: true
  requires_plonky3_epoch_finalized: true
  requires_epoch_manifest_finalized: true
  requires_archive_replication_threshold_met: true
  requires_retrieval_audit_passed: true
  must_keep_compact_metadata: true
  must_keep_epoch_manifest: true
  must_keep_state_snapshot: true
  must_not_prune_archive_replicas: true

paths:
  recursive_sidecars: artifacts/checkpoints/recursive_shadow
  nova_block_proofs: artifacts/checkpoints/nova_block
  pq_checkpoints: artifacts/checkpoints/pq_anchor
  plonky3_epoch_proofs: artifacts/checkpoints/plonky3_epoch
  epoch_manifests: artifacts/checkpoints/epoch_manifest
  archive_manifests: artifacts/checkpoints/archive_manifest
  state_snapshots: artifacts/checkpoints/state_snapshot
  retrieval_audits: artifacts/checkpoints/retrieval_audit
  archive_receipts: artifacts/checkpoints/archive_receipt

limits:
  max_recursive_proof_bytes: 8388608
  max_recursive_sidecar_bytes: 12582912
  max_nova_block_proof_bytes: 131072
  max_epoch_nova_archive_bytes: 134217728
  max_plonky3_epoch_proof_bytes: 16777216
  max_plonky3_epoch_sidecar_bytes: 25165824
  max_pq_anchor_bytes: 16777216
  max_archive_manifest_bytes: 8388608
  max_state_snapshot_manifest_bytes: 16777216
  max_retrieval_audit_bytes: 4194304
```

📌 Gate rules:

- Missing or bypassed config validator rejects runtime sidecar use.
- `branches.recursive.is_authoritative: true` rejects.
- `authority_promotion.recursive_authority_allowed: true` rejects before
  `verified_backend_enabled`.
- `authority_promotion.verified_backend_allowed: true` rejects before proof
  object, verifier API, codec support, negative tests, benchmarks, security
  review, and rollback policy exist.
- `min_chain_steps < 3` rejects.
- `target_chain_steps < min_chain_steps` rejects.
- `target_chain_steps > 5` rejects for required Phase 069 evidence.
- `branches.recursive.mode != hybrid_nova_plonky3` rejects.
- `branches.recursive.proof_system != recursive_hybrid_v1` rejects.
- Missing `branches.nova` or disabled Nova branch rejects.
- `branches.nova.cadence_blocks != 1` rejects.
- `branches.nova.is_pq_authoritative: true` rejects.
- `branches.nova.proof_system != nova_compressed_v1` rejects.
- Missing `branches.plonky3_epoch` or disabled Plonky3 epoch branch rejects.
- `branches.plonky3_epoch.cadence_blocks != post_quantum.cadence_blocks`
  rejects.
- `branches.plonky3_epoch.is_pq_authoritative: false` rejects.
- `branches.plonky3_epoch.proof_system != plonky3_stark_epoch_v1` rejects.
- `branches.plonky3_epoch.must_prove_canonical_transition_range: false`
  rejects.
- `branches.plonky3_epoch.must_not_depend_only_on_nova: false` rejects.
- `branches.plonky3_epoch.field != koala_bear`, `hash != poseidon2`, or
  `recursion_library != p3_recursion` rejects in the default profile.
- `branches.plonky3_epoch.security_bits < 124` rejects.
- `archive_retention.celestia_is_da_only: false` rejects.
- `archive_retention.long_term_retrieval_required: false` rejects.
- `archive_retention.content_addressing_required: false` rejects.
- `archive_retention.ipfs_pinning_required: false` rejects.
- `archive_retention.provider_receipts_required: false` rejects.
- `archive_retention.retrieval_audit_required: false` rejects.
- `archive_retention.retrievability_is_not_validity: false` rejects.
- `archive_retention.min_archive_replicas < 3` rejects.
- `archive_retention.retrieval_audit_interval_blocks != post_quantum.cadence_blocks`
  rejects.
- Missing archive retention required artifacts reject.
- `snapshots.is_enabled: false` rejects.
- `snapshots.cadence_epochs == 0` rejects.
- `snapshots.cadence_blocks != snapshots.cadence_epochs * post_quantum.cadence_blocks`
  rejects.
- `snapshots.object_type != state_snapshot_v1` rejects.
- Any snapshot binding flag set to false rejects.
- `pruning.full_node_pruning_allowed: false` rejects in the default profile.
- `pruning.archive_node_pruning_allowed: true` rejects.
- `pruning.prune_scope != local_full_node_only` rejects.
- `pruning.min_retain_recent_epochs == 0` rejects.
- Any pruning prerequisite flag set to false rejects.
- Empty, absolute, traversing, or colliding recursive sidecar paths reject.
- Empty, absolute, traversing, or colliding Nova proof, Plonky3 epoch proof, or
  epoch manifest paths reject.
- Empty, absolute, traversing, or colliding archive manifest, state snapshot,
  retrieval audit, or archive receipt paths reject.
- Empty, absolute, traversing, or colliding PQ checkpoint paths reject.
- Zero or overflowed `max_recursive_proof_bytes` rejects.
- Zero or overflowed `max_recursive_sidecar_bytes` rejects.
- Zero or overflowed `max_nova_block_proof_bytes` rejects.
- Zero or overflowed `max_epoch_nova_archive_bytes` rejects.
- Zero or overflowed `max_plonky3_epoch_proof_bytes` rejects.
- Zero or overflowed `max_plonky3_epoch_sidecar_bytes` rejects.
- Zero or overflowed `max_pq_anchor_bytes` rejects.
- Zero or overflowed `max_archive_manifest_bytes` rejects.
- Zero or overflowed `max_state_snapshot_manifest_bytes` rejects.
- Zero or overflowed `max_retrieval_audit_bytes` rejects.
- Zero or overflowed `max_documentation_packet_bytes` rejects.
- `post_quantum.is_enabled: false` rejects for the default recursive-ready
  contract unless a later profile explicitly replaces PQ audit anchoring.
- `post_quantum.cadence_blocks == 0` rejects.
- `post_quantum.mode` MUST be `plonky3_epoch_proof` in the default Phase 069
  profile.
- `post_quantum.enforce_live_cadence: true` rejects while
  `authority_promotion.stage` is before `pq_anchor_writer`.
- `post_quantum.enforce_live_cadence: false` rejects at or after
  `pq_anchor_writer`.
- Missing `post_quantum.required_artifacts` entries reject.
- `documentation.recursive_packet_required: false` rejects.
- Any `documentation.include_*` flag set to false rejects Phase 069 closeout.
- Unknown YAML fields reject.

📌 Active-use rules:

- `z00z_storage::checkpoint::CheckpointContractConfigV1` MUST be the canonical
  typed representation of this file.
- `z00z_storage::checkpoint` exposes the canonical validator in
  `contract_config.rs`; implementations MUST call it before any checkpoint,
  sidecar, PQ anchor, documentation packet, or pruning write when the config is
  missing or invalid.
- `z00z_rollup_node::config` MAY load the file for node startup and config
  digest reporting, but it MUST call or mirror the storage-owned validator
  rather than inventing a second rule set.
- `CheckpointFsStore` and future sidecar/PQ stores SHOULD accept a validated
  config handle or a validated path policy before writing.
- Simulator and integration tests MUST use `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`,
  not only an inline fixture.
- A repository state with this YAML file present but runtime writers bypassing
  `CheckpointContractConfigV1` MUST be treated as implementation-incomplete,
  not gate-complete.

## 🧬 Backend And PQ Policy

📌 Phase 069 backend policy:

- `recursive_hybrid_v1` is the required repository proof profile.
- `nova_compressed_v1` is the required fast classical per-block lane.
- `plonky3_stark_epoch_v1` is the required PQ-friendly epoch lane.
- Nova/Arecibo dependencies MUST be pinned by repository dependency policy
  before implementation work lands.
- Plonky3 and Plonky3-recursion dependencies MUST be pinned by commit or exact
  package version before implementation work lands.
- The Plonky3 epoch lane MUST use the configured default profile:
  `field: koala_bear`, `hash: poseidon2`, `recursion_library: p3_recursion`,
  and `security_bits >= 124`.
- A backend implementation MUST substitute proof bytes over the same checkpoint
  statement. It MUST NOT require a new checkpoint theorem.
- A backend implementation MUST provide parameter docs, canonical ABI, test
  vectors, reject reasons, measured fixtures, audit notes, and rollback policy
  before future canonical promotion review.
- `recursive_mock_v1` MAY remain only as a local test utility for codec and
  negative-path tests; it MUST NOT be the default repository profile.
- SuperNova, Fractal, HyperPlonk, LatticeFold, RLWE, and generic STARK
  references remain research or benchmark tracks unless a later spec promotes
  them with the same gates.

📌 Why this choice:

- Nova gives the best near-term shape for per-block IVC: it is naturally
  incremental, compressed, and small enough to attach every block.
- Nova is curve/ECC based and therefore cannot protect history after a quantum
  break of DLOG assumptions.
- Plonky3-recursion gives a real Rust recursive STARK path with transparent
  setup and hash/FRI security assumptions that match the PQ epoch objective.
- Plonky3 proofs are larger and proving is more expensive, so they run at
  epoch cadence by default instead of every block.
- The 1000-block cadence is a configurable compromise: recent open-epoch blocks
  keep fast classical proofs and full replay material, while completed epochs
  gain a Plonky3/STARK history lock.

📌 PQ red lines:

- Pedersen binding MUST NOT be described as post-quantum safe.
- Discrete-log, pairing, or curve-based security assumptions MUST NOT be used as
  PQ evidence.
- Nova compressed proofs MUST NOT be described as PQ evidence.
- A Plonky3 proof that only proves Nova verifier acceptance MUST NOT be
  described as PQ epoch finality.
- Same-challenge aggregation across proofs MUST NOT be accepted without a real
  reviewed folding construction.
- RLWE or lattice PoK sketches MUST NOT be promoted without explicit challenge
  domain, Fiat-Shamir with aborts if required, center-lift rules, canonical ABI,
  parameter compatibility, PoK reduction, test vectors, constant-time review,
  and implementation benchmarks.
- Plonky3 epoch proofs remain non-canonical admission evidence until a future
  verified backend is implemented, audited, reviewed, benchmarked, and
  explicitly promoted.

## 🧬 Post-Quantum Cadence Contract

📌 `PostQuantumCheckpointAnchorV1` is the Phase 069 PQ epoch audit object. It is
external to V1 canonical checkpoint admission and binds the Plonky3 epoch proof,
retained canonical material, and optional Nova chain root for long-horizon audit
and migration.

```yaml
post_quantum_checkpoint_anchor_v1:
  version: 1
  height: 1000
  cadence_blocks: 1000
  statement_digest: "0x..."
  pq_statement_digest: "0x..."
  pq_delta_root: "0x..."
  pq_witness_root: "0x..."
  pq_archive_manifest_root: "0x..."
  plonky3_epoch_statement_digest: "0x..."
  plonky3_epoch_proof_digest: "0x..."
  plonky3_public_inputs_digest: "0x..."
  nova_chain_root: "0x..."
  pq_signature_or_commitment: "0x..."
  mode: plonky3_epoch_proof
  enforcement_stage: pq_anchor_writer
```

📌 Cadence rules:

- Default cadence is exactly `1000` blocks.
- Cadence applies to positive non-genesis heights.
- `is_pq_cadence_height(height)` MUST return true when
  `height % cadence_blocks == 0` and `height > 0`.
- With default cadence, height `999` MUST NOT require an anchor and height
  `1000` MUST require an anchor once live enforcement is active.
- Before `pq_anchor_writer`, the policy is declared-only: fixtures and docs MAY
  be emitted, but runtime MUST NOT claim live PQ cadence enforcement.
- At `pq_anchor_writer` or later, every cadence height MUST write a complete
  Plonky3 epoch proof, epoch manifest, and PQ anchor before the cadence audit is
  considered complete.
- Live cadence enforcement MUST NOT start challenge timing, admit checkpoints,
  or replace canonical replay. It is an audit-completeness gate only unless a
  later authority spec expands its role.
- A completed cadence height closes the previous epoch. Heights after that
  boundary are an open epoch until the next Plonky3 epoch proof is written.

📌 Anchor binding rules:

- `statement_digest` MUST equal the checkpoint statement digest used by the
  canonical artifact and recursive sidecar.
- `pq_statement_digest` MUST be a domain-separated digest over the PQ anchor
  statement bytes, not a duplicate unframed string.
- `pq_delta_root` MUST bind the same touched-delta transition committed by the
  checkpoint statement.
- `pq_witness_root` MUST bind retained witness archive material.
- `pq_archive_manifest_root` MUST equal the archive manifest root used by the
  DA/archive contract for the same checkpoint.
- `plonky3_epoch_statement_digest` MUST equal the digest of
  `EpochRangeStatementV1`.
- `plonky3_epoch_proof_digest` MUST equal the digest of the
  `Plonky3EpochProofV1` proof bytes and metadata.
- `plonky3_public_inputs_digest` MUST equal the canonical public inputs used by
  the Plonky3 verifier.
- `nova_chain_root` MUST match the ordered Nova proof digest root when Nova
  proofs exist for the epoch.
- `pq_signature_or_commitment` MUST be present and versioned, but Phase 069 MUST
  describe it as auxiliary audit evidence, not as the primary PQ proof.
- The Plonky3 proof MUST prove the canonical transition range or reject as
  `Plonky3CanonicalRangeMissing`.
- The anchor MUST reject as `Plonky3DependsOnlyOnNova` if the epoch proof only
  demonstrates Nova verifier acceptance.
- The anchor MUST NOT include local filesystem paths, provider SDK-native
  receipts, hostnames, or operator notes in committed bytes.
- The anchor MUST be immutable after write. Correction requires a new versioned
  object and explicit audit linkage.

📌 Required PQ documentation:

- The closeout packet MUST contain at least one default-cadence fixture proving
  height `999` has no required anchor and height `1000` has a complete anchor in
  live-enforcement mode.
- The packet MUST include a rejected-claim register for any source text that
  implies classical commitments, DLP assumptions, or unreviewed folding sketches
  are already post-quantum secure.
- The packet MUST say explicitly that the Plonky3 epoch proof is the selected
  real PQ-friendly implementation target but is still not canonical admission
  authority until a later verified-backend promotion gate succeeds.

## 📚 Recursive Documentation Packet Contract

📌 `RecursiveCheckpointDocumentationPacketV1` is a required Phase 069 artifact.
It makes this spec operational by bundling the concrete evidence needed for
review, re-run, and backend promotion decisions.

```yaml
recursive_checkpoint_documentation_packet_v1:
  version: 1
  spec_path: .planning/phases/69-Recursive-Proof/069-TODO.md
  config_path: crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml
  dependency_manifest_digest: "0x..."
  external_docs_root: "0x..."
  statement_schema_digest: "0x..."
  source_disposition_digest: "0x..."
  object_schema_digests:
    recursive_checkpoint_public_input_v1: "0x..."
    recursive_checkpoint_witness_v1: "0x..."
    recursive_checkpoint_proof_v1: "0x..."
    nova_compressed_block_proof_v1: "0x..."
    nova_epoch_chain_root_v1: "0x..."
    epoch_range_statement_v1: "0x..."
    plonky3_epoch_proof_v1: "0x..."
    epoch_manifest_v1: "0x..."
    checkpoint_archive_manifest_v1: "0x..."
    archive_provider_receipt_v1: "0x..."
    retrieval_audit_v1: "0x..."
    state_snapshot_v1: "0x..."
    pruning_decision_v1: "0x..."
    recursive_checkpoint_sidecar_v1: "0x..."
    recursive_checkpoint_chain_evidence_v1: "0x..."
    recursive_checkpoint_measurement_v1: "0x..."
    post_quantum_checkpoint_anchor_v1: "0x..."
  golden_vector_root: "0x..."
  chain_evidence_root: "0x..."
  measurement_root: "0x..."
  pq_cadence_evidence_root: "0x..."
  archive_retention_evidence_root: "0x..."
  retrieval_audit_evidence_root: "0x..."
  snapshot_evidence_root: "0x..."
  pruning_decision_evidence_root: "0x..."
  backend_manifest_digest: "0x..."
  rejected_claim_register_digest: "0x..."
```

📌 Packet rules:

- The packet MUST be generated from persisted artifacts, not handwritten after
  the fact.
- It MUST include exact pinned versions or reviewed git revisions for
  `nova-snark`, the selected `p3-*` family, `ipfs-api-backend-hyper`, and the
  Kubo binary used by fixtures, CI, or operator runbooks.
- It MUST include direct documentation links for every external crate and
  binary named in the implementation dependency matrix.
- It MUST include source disposition for every Phase 69 idea file and
  `068-TODO.md`.
- It MUST include one positive and one negative vector for every object codec
  introduced by Phase 069.
- It MUST include 3-step and 5-step chain evidence IDs and the corresponding
  tamper-case IDs.
- It MUST include measurement IDs and state that measurements are local spike
  evidence only.
- It MUST include default PQ cadence evidence for heights `999` and `1000`,
  including the Plonky3 epoch statement/proof IDs for height `1000`.
- It MUST include archive retention evidence proving Celestia is DA-only,
  archive manifests are content-addressed, IPFS entries are pinned, provider
  receipts exist, and retrieval audits pass at the configured cadence.
- It MUST include state snapshot evidence binding state root, settlement root,
  latest Plonky3 epoch proof, latest epoch manifest, archive manifest, snapshot
  chunk root, PQ anchor, and retrieval audit.
- It MUST include pruning decision evidence showing full-node pruning is local
  only and archive-node pruning rejects.
- It MUST include a backend manifest that labels `nova_compressed_v1` as the
  per-block classical lane and `plonky3_stark_epoch_v1` as the epoch
  PQ-friendly lane.
- It MUST record that Plonky3-recursion is active-development/unaudited and
  therefore blocked from canonical `VERIFIED` promotion until later review.
- It MUST include a rejected-claim register for overclaims about production
  recursive validity, PQ safety, proof size, state size, DA-as-validity, or
  pruning.
- It MUST be written under `paths.documentation_packets`.
- A packet that omits dependency pins or documentation links MUST reject
  closeout as incomplete.
- A missing packet or missing required section MUST reject Phase 069 closeout.

## 🧱 Module Placement

| Module or crate | Owns in Phase 069 | MUST NOT own |
| --- | --- | --- |
| `z00z_storage::checkpoint` | Statement binding, Nova/Plonky3 sidecar envelopes, epoch manifest, archive manifest, archive receipts, retrieval audits, state snapshots, pruning decisions, sidecar codec, PQ anchor envelope, config subset, artifact attachment, reject reason facade. | Recursive backend security claims, archive provider SDK ownership, or spend verifier replacement. |
| `z00z_storage::settlement` | HJMT roots, delta rows, witness roots, journal digest, witness package references. | Recursive proof theorem. |
| `z00z_crypto` | Domain separators and digest helper facades only. | Unreviewed recursive verifier truth. |
| Future `z00z_recursive_proofs` | Nova compressed adapter, Plonky3 epoch adapter, recursive verifier traits, chain verifier, benchmark harness after storage contract gates pass. | Canonical checkpoint theorem, artifact codec authority, or spend/range verifier replacement. |
| `z00z_simulator` | Deterministic 3-step and 5-step chain evidence, Nova tamper fixtures, Plonky3 cadence fixtures, measurement output. | Production recursive backend claims before measured evidence. |
| `z00z_runtime::validators` | Rejection that sidecars are not admission authority. | Duplicate checkpoint theorem. |
| `z00z_runtime::watchers` | Gap reporting and publication observation when Phase 70 evidence exists. | Settlement validity. |
| `z00z_rollup_node` | DA/publication readiness dependencies for later phases. | Recursive statement bytes or provider SDK types in proof input. |

### Module Ownership Plan

📌 Phase 069 implementation MUST use this ownership plan unless a later spec
updates the architecture doublecheck ledger and tests the migration.

| Surface | Primary home | Candidate files | Runtime role | MUST NOT do |
| --- | --- | --- | --- | --- |
| Checkpoint contract config | `z00z_storage::checkpoint` | Implemented in `contract_config.rs`, exported from `mod.rs` | Validate `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml` before checkpoint, sidecar, PQ, pruning, documentation, or promotion writes. | Hide validation inside rollup-node only. |
| Extended transition statement | `z00z_storage::checkpoint` | `transition_statement.rs` or extension of `artifact_stmt.rs` | Produce statement/core/final digests from storage-owned fields. | Rebuild a second theorem in runtime or recursive crate. |
| Canonical artifact compatibility | `z00z_storage::checkpoint` | `codec.rs`, `artifact_final.rs`, `artifact_types.rs` | Keep `OPAQUE_ATTEST` canonical and `VERIFIED` reserved. | Admit recursive proof bytes as canonical. |
| Recursive sidecar envelope | `z00z_storage::checkpoint` first; future `z00z_recursive_proofs` may consume it | `recursive_sidecar.rs`, `recursive_chain.rs`, `recursive_reject.rs` | Persist and verify non-authoritative sidecar evidence. | Mutate `CheckpointArtifact` or `CheckpointLink`. |
| Nova block proof object | `z00z_storage::checkpoint` schema; future `z00z_recursive_proofs` prover/verifier | `nova_block_proof.rs`, `recursive_chain.rs` | Bind one checkpoint statement and checkpoint link per block; retain until PQ epoch. | Claim PQ authority or bypass canonical replay. |
| Plonky3 epoch statement/proof | `z00z_storage::checkpoint` schema; future `z00z_recursive_proofs` prover/verifier | `epoch_range_statement.rs`, `plonky3_epoch_proof.rs` | Bind configured epoch range, canonical statements, DA/archive manifests, witness roots, and optional Nova chain root. | Depend only on Nova verifier acceptance. |
| Epoch manifest | `z00z_storage::checkpoint` | `epoch_manifest.rs` | Persist permanent metadata tying canonical artifacts, Nova proof digests, Plonky3 proof digest, and archive roots. | Store unverifiable local paths as committed data. |
| Archive manifest and receipts | `z00z_storage::checkpoint` schema; rollup/storage exporter writes receipts | `archive_manifest.rs`, `archive_receipt.rs`, `retrieval_audit.rs` | Bind raw packages, exact proof bytes, witness chunks, deltas, DA payload commitment, provider receipts, and retrieval audits. | Treat Celestia DA or unpinned IPFS CID as permanent history. |
| State snapshot | `z00z_storage::checkpoint` schema; simulator/rollup fixture generation | `state_snapshot.rs` | Bind state root, settlement root, latest Plonky3 proof, latest epoch manifest, archive manifest, chunk root, PQ anchor, and retrieval audit. | Replace proof verification or archive retrieval with a naked root. |
| Pruning decision | `z00z_storage::checkpoint` | `pruning.rs` | Permit local full-node deletion only after dispute, Plonky3, manifest, archive replication, and retrieval audit gates pass. | Permit archive-node pruning or network-wide history deletion. |
| Recursive adapter trait | Future `z00z_recursive_proofs`; storage tests may use local test traits | `adapter.rs`, `nova.rs`, `plonky3.rs`, `measurement.rs` | Provide Nova `prove_step`/`verify_step` and Plonky3 `prove_epoch`/`verify_epoch`. | Own statement schema or settlement roots. |
| PQ anchor envelope | `z00z_storage::checkpoint` | `pq_anchor.rs` | Write and validate cadence audit objects under configured paths; bind Plonky3 epoch proof and manifest roots. | Claim canonical admission before verified-backend promotion. |
| Documentation packet | `z00z_storage::checkpoint` plus simulator/test harness | `documentation_packet.rs` or closeout artifact generator | Index vectors, chain evidence, measurements, PQ anchors, and rejected claims. | Replace tests or code validation with prose. |
| Rollup integration | `z00z_rollup_node` | `config.rs`, `runtime.rs`, `da.rs` | Load config path, pass validated gates, publish/resolve DA payloads. | Put provider SDK types into statement/public input bytes. |
| Validator integration | `crates/z00z_runtime/validators` crate `z00z_validators` | `checkpoint.rs`, `verdict.rs` | Verify canonical settlement theorem and reject sidecar authority. | Re-derive recursive theorem or mark sidecar as verdict authority. |
| Watcher integration | `crates/z00z_runtime/watchers` crate `z00z_watchers` | `publication.rs`, `evidence_export.rs`, `engine.rs` | Observe publication readiness, gaps, and evidence bindings. | Treat DA availability as validity. |
| Simulator evidence | `z00z_simulator` | scenario fixtures and test helpers | Produce deterministic 3-step, 5-step, Nova tamper, Plonky3 cadence, and doc-packet evidence. | Claim production backend performance. |
| Crypto helpers | `z00z_crypto` | domain separator and digest facade modules | Provide domain-separated digest helpers if storage needs a shared facade. | Import unreviewed backend security assumptions. |

📌 Crate creation rule:

- Phase 069 SHOULD create `z00z_recursive_proofs` only after the storage-owned
  sidecar envelope, reject taxonomy, public input vectors, and active config
  gates are stable.
- If a crate is created during Phase 069, it MUST target real Nova and Plonky3
  adapters, but it MUST remain non-authoritative until promotion gates pass.
- The future crate MUST depend on the storage statement contract; storage MUST
  NOT depend on a concrete recursive backend.

## 🗺️ Architecture Diagrams

📌 Diagram plan:

- C4 system context: shows people and external systems around the
  recursive-ready checkpoint system.
- C4 container: shows workspace crates and artifact/config stores.
- C4 component: shows the `z00z_storage::checkpoint` implementation boundary.
- C4 dynamic: shows the end-to-end checkpoint, sidecar, DA, and PQ cadence flow.
- C4 deployment: shows local node/runtime placement and file-backed gates.
- Mermaid flowchart: shows gate execution and fail-closed branches.
- Mermaid state: shows checkpoint lifecycle states.
- Mermaid ER: shows persisted object relationships.
- Mermaid requirement trace: maps requirements to modules and tests.

### C4 System Context

```mermaid
flowchart LR
  Wallet[Wallets\nSubmit transaction packages]
  Operator[Node operators\nRun Z00Z nodes]
  Auditor[Auditors\nReview checkpoint and recursive evidence]
  Z00Z[Z00Z Hybrid Recursive Checkpoint System\nCanonical theorem, Nova block lane, Plonky3 epoch lane]
  DA[DA / Archive Provider\nPublishes retrievable checkpoint payloads]
  Nova[Nova compressed lane\nFast classical per-block IVC]
  Plonky3[Plonky3 recursive STARK lane\nPQ-friendly epoch proof]

  Wallet -->|submits packages| Z00Z
  Operator -->|loads active config| Z00Z
  Auditor -->|checks docs, vectors, anchors| Z00Z
  Z00Z -->|exports archive package| DA
  DA -->|returns availability evidence| Z00Z
  Nova -. binds every checkpoint statement .-> Z00Z
  Plonky3 -. proves canonical epoch range .-> Z00Z

  style Wallet fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Operator fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Auditor fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Z00Z fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style DA fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Nova fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Plonky3 fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
```

### C4 Container View

```mermaid
flowchart LR
  Wallet[Wallets]
  DA[DA / Archive Provider]

  subgraph Workspace[Z00Z Workspace]
    Rollup[z00z_rollup_node\nConfig loading, node lifecycle, DA adapter]
    Aggregators[z00z_aggregators\nOrdering, route planning, publication request]
    Validators[z00z_validators\nSettlement theorem and verdicts]
    Watchers[z00z_watchers\nPublication observation and gap alerts]
    Storage[z00z_storage\nCheckpoint statement, artifacts, sidecars, PQ anchors]
    Crypto[z00z_crypto\nDomain separators and digest helpers]
    Simulator[z00z_simulator\nDeterministic fixtures and E2E evidence]
    FutureRec[z00z_recursive_proofs future\nNova and Plonky3 adapters]
    Config[(crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml\nActive architecture gate)]
    ArtifactStore[(Checkpoint artifact stores\nCanonical, sidecar, PQ, docs)]
  end

  Wallet -->|tx packages| Aggregators
  Aggregators -->|PublicationRequest| Rollup
  Rollup -->|seal/load checkpoint objects| Storage
  Rollup -->|publish/resolve payload| DA
  DA -->|availability evidence| Rollup
  Rollup -->|ResolvedBatch| Validators
  Validators -->|Verdict| Watchers
  Watchers -->|PublicationWatch| Rollup
  Storage -->|hash domains| Crypto
  FutureRec -. Nova prove_step / verify_step .-> Storage
  FutureRec -. Plonky3 prove_epoch / verify_epoch .-> Storage
  Simulator -->|fixtures and tamper cases| Rollup
  Config -->|validated before writes| Rollup
  Config -->|storage-owned validation| Storage
  Storage -->|persist| ArtifactStore

  style Wallet fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style DA fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Rollup fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Aggregators fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Validators fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Watchers fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Storage fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Crypto fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Simulator fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style FutureRec fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Config fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style ArtifactStore fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
```

### C4 Component View: `z00z_storage::checkpoint`

```mermaid
flowchart LR
  subgraph Checkpoint[z00z_storage::checkpoint]
    ConfigGate[CheckpointContractConfigV1\nValidates YAML gates]
    Statement[CheckpointTransitionStatementV1\nSingle checkpoint theorem]
    ExecInput[CheckpointExecInput\nExact replay and tx proof bytes]
    ArtifactCodec[CheckpointArtifact Codec\nOpaque canonical admission]
    LinkStore[CheckpointLink Store\nSnapshot and exec binding]
    SidecarCodec[RecursiveCheckpointSidecarV1\nShadow proof evidence]
    NovaProof[NovaCompressedBlockProofV1\nPer-block classical IVC]
    EpochStmt[EpochRangeStatementV1\nCanonical 1000-block range]
    PlonkyProof[Plonky3EpochProofV1\nRecursive STARK epoch proof]
    EpochManifest[EpochManifestV1\nPermanent epoch metadata]
    ArchiveManifest[CheckpointArchiveManifestV1\nLong-term retrieval metadata]
    RetrievalAudit[RetrievalAuditV1\nReplica retrieval gate]
    StateSnapshot[StateSnapshotV1\nVerified bootstrap base]
    PruningDecision[PruningDecisionV1\nLocal full-node pruning gate]
    ChainVerifier[RecursiveCheckpointChainEvidenceV1\nPrior-output binding]
    PqAnchor[PostQuantumCheckpointAnchorV1\nCadence audit envelope]
    DocPacket[RecursiveCheckpointDocumentationPacketV1\nCloseout evidence]
    Store[CheckpointFsStore\nFile-backed persistence]
  end

  Settlement[z00z_storage::settlement\nHJMT roots, deltas, witnesses, journals]
  Crypto[z00z_crypto\nDigest helper facade]
  Rollup[z00z_rollup_node\nRuntime wiring]

  ConfigGate -->|enables safe modes| Store
  Settlement -->|roots and witness commitments| Statement
  ExecInput -->|exec id and replay bytes| Statement
  Statement -->|opaque payload today| ArtifactCodec
  ArtifactCodec -->|checkpoint id| LinkStore
  SidecarCodec -->|same statement digest| Statement
  NovaProof -->|binds statement and link| Statement
  EpochStmt -->|binds ordered statement root| Statement
  PlonkyProof -->|proves epoch statement| EpochStmt
  EpochManifest -->|indexes proof and archives| PlonkyProof
  EpochManifest -->|binds Nova chain root| NovaProof
  ArchiveManifest -->|binds raw, witness, delta, DA payload| EpochManifest
  RetrievalAudit -->|checks archive replicas| ArchiveManifest
  StateSnapshot -->|binds latest epoch and archive roots| EpochManifest
  StateSnapshot -->|requires retrieval audit| RetrievalAudit
  PruningDecision -->|requires finalized proof and archive audit| StateSnapshot
  ChainVerifier -->|loads sidecars| SidecarCodec
  PqAnchor -->|binds Plonky3 proof and archive roots| ArchiveManifest
  DocPacket -->|indexes vectors and evidence| SidecarCodec
  Store -->|persists canonical and shadow objects| ArtifactCodec
  Store -->|persists sidecars, PQ anchors, docs| SidecarCodec
  Statement -->|domain hashes| Crypto
  Rollup -->|calls through facade| ConfigGate

  style ConfigGate fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Statement fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style ExecInput fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style ArtifactCodec fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style LinkStore fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style SidecarCodec fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style NovaProof fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style EpochStmt fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style PlonkyProof fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style EpochManifest fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style ArchiveManifest fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style RetrievalAudit fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style StateSnapshot fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style PruningDecision fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style ChainVerifier fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style PqAnchor fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style DocPacket fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Store fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Settlement fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Crypto fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Rollup fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
```

### C4 Dynamic View: Checkpoint With Sidecar And PQ Cadence

```mermaid
sequenceDiagram
  box rgb(227,242,253) Public actors
    participant Wallet
    participant Auditor
  end
  box rgb(255,243,224) Runtime
    participant Agg as z00z_aggregators
    participant Rollup as z00z_rollup_node
  end
  box rgb(255,224,178) Storage
    participant Store as z00z_storage::checkpoint
    participant Settlement as z00z_storage::settlement
  end
  box rgb(243,229,245) Validation
    participant Val as z00z_validators
    participant Watch as z00z_watchers
  end
  box rgb(237,231,246) Proof and PQ
    participant Rec as Recursive adapter
    participant PQ as PQ anchor writer
  end
  box rgb(232,245,233) DA / Archive
    participant DA
  end

  Wallet->>Agg: submit transaction package
  Agg->>Rollup: build PublicationRequest
  Rollup->>Store: load and validate crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml
  Rollup->>Store: save CheckpointExecInput with exact tx proof bytes
  Store->>Settlement: obtain roots, deltas, witness root, journal digest
  Store->>Store: build CheckpointTransitionStatementV1
  Store->>DA: export DA/archive payload through adapter boundary
  DA-->>Rollup: provider-neutral availability reference
  Rollup->>Store: seal opaque CheckpointArtifact and CheckpointLink
  Rollup->>Val: resolve batch for settlement theorem validation
  Val-->>Rollup: verdict over canonical artifact, exec input, link, tx package
  Rollup->>Watch: publish local publication evidence
  Watch-->>Rollup: observation or gap alert
  Store->>Rec: build public input and witness fixture
  Rec-->>Store: write non-authoritative RecursiveCheckpointSidecarV1
  Store->>Rec: verify 3-step and 5-step chain evidence
  Store->>PQ: if height % cadence_blocks == 0 and live stage, write PQ anchor
  PQ-->>Store: PostQuantumCheckpointAnchorV1 audit envelope
  Store-->>Auditor: documentation packet with vectors, measurements, rejects, anchors
```

### C4 Deployment View: Local Node And File Gates

```mermaid
flowchart LR
  Operator[Node operator]

  subgraph Host[Local or CI host]
    ConfigFile[(crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml)]
    NodeProc[Rollup node process\nz00z_rollup_node]
    RuntimeSvc[Runtime crates\naggregators, validators, watchers]
    StorageRoot[(Storage root\ncheckpoint artifacts)]
    DaArchive[(Local DA/archive export)]
    Sim[Simulator / tests\nz00z_simulator]
  end

  ExternalDA[External DA provider\nfuture adapter]

  Operator -->|starts node| NodeProc
  ConfigFile -->|must validate before writes| NodeProc
  NodeProc -->|coordinates| RuntimeSvc
  RuntimeSvc -->|checkpoint writes| StorageRoot
  NodeProc -->|local publish/resolve| DaArchive
  NodeProc -. future provider adapter .-> ExternalDA
  Sim -->|fixture runs| NodeProc
  Sim -->|asserts artifacts| StorageRoot

  style Operator fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style ConfigFile fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style NodeProc fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style RuntimeSvc fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style StorageRoot fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style DaArchive fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Sim fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style ExternalDA fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
```

### Gate Flow

```mermaid
flowchart TD
  Start[Checkpoint interval] --> LoadCfg[Load crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml]
  LoadCfg --> CfgGate{All config gates pass?}
  CfgGate -->|no| RejectCfg[Reject before any write]
  CfgGate -->|yes| BuildStmt[Build CheckpointTransitionStatementV1]
  BuildStmt --> InputGate{Input gates pass?}
  InputGate -->|no| RejectInput[Reject statement build]
  InputGate -->|yes| DAExport[Export DA/archive payload]
  DAExport --> OutputGate{Output and DA gates pass?}
  OutputGate -->|no| RejectOutput[Do not start challenge timing]
  OutputGate -->|yes| Seal[Seal opaque artifact and checkpoint link]
  Seal --> Sidecar[Write recursive shadow sidecar]
  Sidecar --> Chain{3 and 5 step chains verify?}
  Chain -->|no| RejectSidecar[Reject sidecar evidence only]
  Chain -->|yes| Cadence{Positive height divisible by cadence?}
  Cadence -->|no| Docs[Write documentation packet]
  Cadence -->|yes| Live{Stage >= pq_anchor_writer?}
  Live -->|no| DeclaredOnly[Record declared-only PQ fixture if needed]
  Live -->|yes| PqAnchor[Require complete PQ anchor]
  PqAnchor --> PqOk{PQ anchor validates?}
  PqOk -->|no| RejectPq[Reject cadence audit completeness]
  PqOk -->|yes| Docs
  DeclaredOnly --> Docs
  Docs --> Done[Phase evidence complete]

  style Start fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style LoadCfg fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style CfgGate fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style RejectCfg fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style BuildStmt fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style InputGate fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style RejectInput fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style DAExport fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style OutputGate fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style RejectOutput fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style Seal fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Sidecar fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Chain fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style RejectSidecar fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style Cadence fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Live fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style DeclaredOnly fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style PqAnchor fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style PqOk fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style RejectPq fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style Docs fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Done fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
```

### Hybrid Recursive Proof Flow

```mermaid
flowchart TD
  Start[Checkpoint block h] --> Canonical[Build canonical CheckpointTransitionStatementV1]
  Canonical --> Artifact[Seal OPAQUE_ATTEST CheckpointArtifact and CheckpointLink]
  Artifact --> NovaInput[Build RecursiveCheckpointPublicInputV1 for Nova]
  NovaInput --> NovaProof[NovaCompressedBlockProofV1]
  NovaProof --> NovaGate{Nova proof binds statement, link, prior output?}
  NovaGate -->|no| NovaReject[Reject Nova sidecar only]
  NovaGate -->|yes| NovaStore[Store under paths.nova_block_proofs]
  NovaStore --> Cadence{h % cadence_blocks == 0 and h > 0?}
  Cadence -->|no| OpenEpoch[Open epoch continues\nNot Plonky3/PQ finalized yet]
  Cadence -->|yes| Manifest[Build EpochManifestV1\n1000 statements, links, DA, witness, Nova root]
  Manifest --> EpochStmt[Build EpochRangeStatementV1]
  EpochStmt --> PlonkyProve[Prove with Plonky3 recursive STARK]
  PlonkyProve --> CanonicalRange{Proves canonical transition range?}
  CanonicalRange -->|no| RangeReject[Reject Plonky3CanonicalRangeMissing]
  CanonicalRange -->|yes| NovaOnly{Depends only on Nova verifier?}
  NovaOnly -->|yes| NovaOnlyReject[Reject Plonky3DependsOnlyOnNova]
  NovaOnly -->|no| PlonkyStore[Store Plonky3EpochProofV1]
  PlonkyStore --> PQAnchor[Write PostQuantumCheckpointAnchorV1]
  PQAnchor --> Closed[Epoch closed under Plonky3/PQ-friendly proof]

  style Start fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Canonical fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Artifact fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style NovaInput fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style NovaProof fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style NovaGate fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style NovaReject fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style NovaStore fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Cadence fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style OpenEpoch fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Manifest fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style EpochStmt fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style PlonkyProve fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style CanonicalRange fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style RangeReject fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style NovaOnly fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style NovaOnlyReject fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  style PlonkyStore fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style PQAnchor fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Closed fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
```

### Checkpoint Lifecycle

```mermaid
stateDiagram-v2
  classDef entry fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  classDef types fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  classDef domain fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  classDef store fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  classDef tests fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  classDef danger fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C

  [*] --> ConfigLoaded
  ConfigLoaded --> Prepared: gates pass
  Prepared --> StatementBuilt: exec input and roots bound
  StatementBuilt --> PublishedToDA: archive package exported
  PublishedToDA --> CanonicalSealed: opaque artifact and link sealed
  CanonicalSealed --> PublicationReady: provider-neutral readiness evidence
  PublicationReady --> NovaProved: Nova block proof attached
  NovaProved --> ChainMeasured: 3 and 5 step evidence verified
  ChainMeasured --> PqCadenceChecked: cadence evaluated
  PqCadenceChecked --> Plonky3EpochProved: live cadence height
  PqCadenceChecked --> DocsPacketWritten: non-cadence or declared-only stage
  Plonky3EpochProved --> PqAnchored: PQ anchor binds epoch proof
  PqAnchored --> DocsPacketWritten
  DocsPacketWritten --> AuditReady
  AuditReady --> [*]
  ConfigLoaded --> Rejected: config gate fails
  StatementBuilt --> Rejected: digest or root mismatch
  PublishedToDA --> Rejected: DA/archive binding mismatch
  NovaProved --> Rejected: Nova sidecar authority or chain mismatch
  Plonky3EpochProved --> Rejected: canonical range missing or Nova-only dependency
  PqCadenceChecked --> Rejected: missing live Plonky3 epoch proof or PQ anchor

  class ConfigLoaded entry
  class Prepared types
  class StatementBuilt domain
  class PublishedToDA store
  class CanonicalSealed store
  class PublicationReady tests
  class NovaProved domain
  class ChainMeasured tests
  class PqCadenceChecked types
  class Plonky3EpochProved domain
  class PqAnchored domain
  class DocsPacketWritten tests
  class AuditReady tests
  class Rejected danger
```

### Storage Data Model

```mermaid
erDiagram
  CHECKPOINT_STATEMENT ||--|| CHECKPOINT_ARTIFACT : admits
  CHECKPOINT_STATEMENT ||--|| CHECKPOINT_EXEC_INPUT : references
  CHECKPOINT_STATEMENT ||--|| PREP_SNAPSHOT : references
  CHECKPOINT_STATEMENT ||--|| DELTA_JOURNAL : commits
  CHECKPOINT_STATEMENT ||--|| WITNESS_ARCHIVE : commits
  CHECKPOINT_STATEMENT ||--|| DA_REFERENCE : binds
  CHECKPOINT_ARTIFACT ||--|| CHECKPOINT_LINK : links
  CHECKPOINT_ARTIFACT ||--o| PUBLICATION_EVIDENCE : gates_timing
  CHECKPOINT_STATEMENT ||--o{ RECURSIVE_SIDECAR : shadows
  RECURSIVE_SIDECAR ||--|| RECURSIVE_PROOF : contains
  RECURSIVE_SIDECAR ||--|| RECURSIVE_MEASUREMENT : measures
  RECURSIVE_SIDECAR ||--o{ CHAIN_EVIDENCE : participates
  CHECKPOINT_STATEMENT ||--|| NOVA_BLOCK_PROOF : proves_classically
  NOVA_BLOCK_PROOF }o--|| NOVA_EPOCH_CHAIN_ROOT : commits_to
  CHECKPOINT_STATEMENT }o--|| EPOCH_MANIFEST : indexed_by
  EPOCH_MANIFEST ||--|| PLONKY3_EPOCH_PROOF : binds
  EPOCH_MANIFEST ||--|| EPOCH_RANGE_STATEMENT : describes
  CHECKPOINT_STATEMENT ||--o| PQ_ANCHOR : anchors_on_cadence
  PQ_ANCHOR ||--|| PLONKY3_EPOCH_PROOF : anchors
  EPOCH_MANIFEST ||--|| ARCHIVE_MANIFEST : binds_retrieval
  ARCHIVE_MANIFEST ||--o{ ARCHIVE_PROVIDER_RECEIPT : replicated_by
  ARCHIVE_MANIFEST ||--o{ RETRIEVAL_AUDIT : audited_by
  STATE_SNAPSHOT ||--|| EPOCH_MANIFEST : bootstraps_from
  STATE_SNAPSHOT ||--|| ARCHIVE_MANIFEST : binds_archive
  PRUNING_DECISION ||--|| STATE_SNAPSHOT : requires
  PRUNING_DECISION ||--|| RETRIEVAL_AUDIT : requires
  CHECKPOINT_STATEMENT ||--o| DOCUMENTATION_PACKET : documents

  CHECKPOINT_STATEMENT {
    int version
    int height
    bytes statement_digest
    bytes prev_root
    bytes new_root
    bytes delta_root
    bytes witness_root
    bytes da_ref
  }
  CHECKPOINT_ARTIFACT {
    int version
    int proof_system
    bytes cp_proof
    bytes checkpoint_id
  }
  CHECKPOINT_EXEC_INPUT {
    int version
    bytes exec_input_id
    bytes prep_snapshot_id
    bytes exact_tx_proof_bytes
  }
  RECURSIVE_SIDECAR {
    int version
    string mode
    string backend_label
    bytes statement_digest
    bytes public_input_digest
  }
  NOVA_BLOCK_PROOF {
    int version
    int height
    bytes statement_digest
    bytes checkpoint_link_digest
    bytes nova_output_root
    bytes proof_bytes_digest
  }
  EPOCH_MANIFEST {
    int version
    int start_height
    int end_height
    bytes statement_digest_root
    bytes nova_chain_root
    bytes plonky3_epoch_proof_digest
  }
  EPOCH_RANGE_STATEMENT {
    int version
    int start_height
    int end_height
    bytes statement_digest_root
    bytes da_archive_manifest_root
  }
  PLONKY3_EPOCH_PROOF {
    int version
    bytes epoch_statement_digest
    bytes public_inputs_digest
    bool proves_canonical_transition_range
    bool depends_only_on_nova
  }
  PQ_ANCHOR {
    int version
    int height
    int cadence_blocks
    bytes pq_statement_digest
    bytes pq_archive_manifest_root
    bytes plonky3_epoch_proof_digest
    bytes nova_chain_root
  }
  ARCHIVE_MANIFEST {
    int version
    bytes raw_tx_package_root
    bytes exact_tx_proof_bytes_root
    bytes witness_archive_root
    bytes retrieval_audit_root
  }
  ARCHIVE_PROVIDER_RECEIPT {
    int version
    string backend
    bytes content_digest
    bool pinned
  }
  RETRIEVAL_AUDIT {
    int version
    int height
    int successful_replica_count
    bool passed
  }
  STATE_SNAPSHOT {
    int version
    int height
    bytes state_root
    bytes last_plonky3_epoch_proof_digest
    bytes snapshot_chunk_root
  }
  PRUNING_DECISION {
    int version
    string prune_scope
    bool archive_replication_threshold_met
    bool retrieval_audit_passed
  }
  DOCUMENTATION_PACKET {
    int version
    bytes packet_digest
    bytes vectors_root
    bytes reject_matrix_root
    bytes pq_cadence_evidence_root
  }
```

### Requirement Trace

```mermaid
requirementDiagram

requirement req_statement {
  id: "RCP-REQ-001"
  text: "Every proof lane shall bind the same CheckpointTransitionStatementV1"
  risk: High
  verifymethod: Test
}

requirement req_shadow {
  id: "RCP-REQ-002"
  text: "Recursive sidecars shall remain non-authoritative in Phase 069"
  risk: High
  verifymethod: Test
}

requirement req_pq {
  id: "RCP-REQ-003"
  text: "Live PQ cadence shall require an anchor on every positive configured cadence height"
  risk: Medium
  verifymethod: Test
}

requirement req_config {
  id: "RCP-REQ-004"
  text: "crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml shall be a strict active runtime gate"
  risk: High
  verifymethod: Test
}

requirement req_hybrid {
  id: "RCP-REQ-005"
  text: "Nova shall prove every block classically and Plonky3 shall prove each completed epoch over canonical transition range"
  risk: High
  verifymethod: Test
}

requirement req_archive {
  id: "RCP-REQ-006"
  text: "Long-term retrieval shall be provided by archive manifests, provider receipts, retrieval audits, snapshots, and pruning gates; DA alone is insufficient"
  risk: High
  verifymethod: Test
}

element storage {
  type: "z00z_storage::checkpoint"
}

element rollup {
  type: "z00z_rollup_node"
}

element tests {
  type: "unit + integration + simulator tests"
}

storage - satisfies -> req_statement
storage - satisfies -> req_shadow
storage - satisfies -> req_pq
storage - satisfies -> req_config
storage - satisfies -> req_hybrid
storage - satisfies -> req_archive
rollup - satisfies -> req_config
tests - verifies -> req_statement
tests - verifies -> req_shadow
tests - verifies -> req_pq
tests - verifies -> req_config
tests - verifies -> req_hybrid
tests - verifies -> req_archive
```

## 🔄 Required Workflow

📌 The Phase 069 local sidecar workflow MUST be:

1. Load and validate checkpoint contract config.
2. Load or build a canonical `CheckpointTransitionStatementV1`.
3. Compute `statement_core_digest_v1` and `statement_digest_v1`.
4. Build `RecursiveCheckpointPublicInputV1`.
5. Load deterministic `RecursiveCheckpointWitnessV1` fixture or archive refs.
6. Run `nova_compressed_v1.prove_step`.
7. Run `nova_compressed_v1.verify_step`.
8. Write `RecursiveCheckpointSidecarV1` and `NovaCompressedBlockProofV1`
   under the configured recursive and Nova proof paths.
9. Repeat for 3 to 5 ordered checkpoint statements.
10. Build and verify `RecursiveCheckpointChainEvidenceV1`.
11. Emit `RecursiveCheckpointMeasurementV1`.
12. Prove canonical checkpoint admission remains unchanged.
13. If the fixture height is a PQ cadence height, build `EpochManifestV1` and
    `EpochRangeStatementV1` for the completed epoch.
14. If live enforcement is active, run `plonky3_stark_epoch_v1.prove_epoch`,
    verify `Plonky3EpochProofV1`, and reject if canonical range proof is
    missing or the proof depends only on Nova.
15. Emit and validate `PostQuantumCheckpointAnchorV1` binding the Plonky3 epoch
    proof, epoch manifest, archive roots, and Nova chain root.
16. Emit or update `CheckpointArchiveManifestV1` with raw package, exact proof
    bytes, witness, delta, DA payload, provider receipt, and retrieval audit
    roots.
17. Emit `ArchiveProviderReceiptV1` entries for configured archive backends,
    including pinned IPFS entries when IPFS is used.
18. Run `RetrievalAuditV1` at configured archive cadence and require enough
    successful replicas before any local pruning decision.
19. If snapshot cadence is reached, emit `StateSnapshotV1` binding state root,
    settlement root, latest Plonky3 epoch proof, latest epoch manifest, archive
    manifest, snapshot chunk root, PQ anchor, and retrieval audit.
20. Allow `PruningDecisionV1` only for local full-node storage after all pruning
    gates pass. Archive-node pruning MUST reject.
21. Emit or update `RecursiveCheckpointDocumentationPacketV1` with vectors,
    measurements, reject matrix, backend manifest red lines, and PQ cadence
    evidence.

📌 Runtime MUST reject any attempt to shortcut from Nova sidecar or Plonky3
epoch proof directly to checkpoint admission.

## 🧨 Failure Model

| Failure | Required response |
| --- | --- |
| Config missing recursive subset | Reject sidecar runtime mode. |
| Recursive branch marked authoritative | Reject config fail-closed. |
| Missing statement digest | Reject public input. |
| Wrong statement digest in proof | Reject proof. |
| Wrong public input digest | Reject proof. |
| Wrong output root | Reject proof. |
| Wrong prior output root | Reject proof or chain. |
| Unsupported backend label | Reject sidecar. |
| Backend claims verified status | Reject sidecar and config. |
| Nova branch disabled or cadence not 1 | Reject config fail-closed. |
| Nova proof claims PQ authority | Reject as `NovaPqAuthorityUnsupported`. |
| Nova chain root mismatches ordered Nova proof digests | Reject epoch manifest or Plonky3 anchor. |
| Plonky3 branch disabled or cadence mismatches PQ cadence | Reject config fail-closed. |
| Plonky3 epoch proof omits canonical transition range | Reject as `Plonky3CanonicalRangeMissing`. |
| Plonky3 epoch proof depends only on Nova verifier acceptance | Reject as `Plonky3DependsOnlyOnNova`. |
| Plonky3 epoch proof exceeds size cap | Reject as `ProofSizeBudgetExceeded`. |
| Epoch manifest misses statement, link, archive, witness, Nova, or Plonky3 binding | Reject as `EpochManifestIncomplete`. |
| Empty proof bytes | Reject proof. |
| Oversized proof bytes | Reject proof before write. |
| Sidecar path aliases artifact path | Reject config before directory creation. |
| Unknown sidecar codec field | Reject decode. |
| Missing measurement field | Reject promotion evidence. |
| Broken 3-step chain | Reject chain evidence. |
| Broken 5-step chain | Reject chain evidence. |
| Skipped or repeated step | Reject chain evidence. |
| Reordered height | Reject chain evidence. |
| Missing exact tx proof bytes | Reject canonical replay. |
| `CheckpointProofSystem::VERIFIED` used for admission | Reject artifact until future codec support exists. |
| Provider SDK receipt enters public input bytes | Reject public input or statement fixture. |
| PQ claim from classical commitment | Reject documentation and backend manifest review. |
| PQ policy disabled in default recursive-ready profile | Reject config fail-closed. |
| PQ cadence is zero or overflows cadence arithmetic | Reject config fail-closed. |
| Live PQ cadence is enabled before `pq_anchor_writer` | Reject config fail-closed. |
| Live PQ cadence is disabled at or after `pq_anchor_writer` | Reject config fail-closed. |
| Height 1000 lacks Plonky3 epoch proof, epoch manifest, or PQ anchor while live enforcement is active | Reject cadence audit completeness. |
| Height 999 requires a PQ anchor under default cadence | Reject cadence calculator. |
| PQ anchor statement digest mismatches checkpoint statement | Reject PQ anchor. |
| PQ anchor archive manifest root mismatches DA/archive manifest | Reject PQ anchor. |
| PQ anchor omits required PQ commitment field | Reject PQ anchor. |
| PQ anchor omits Plonky3 epoch proof digest or Nova chain root | Reject PQ anchor. |
| `pq_anchor_root` appears in V1 canonical admission | Reject statement or artifact. |
| Config treats Celestia as permanent archive | Reject as `CelestiaPermanentStorageUnsupported`. |
| IPFS archive entry lacks pinning or receipt | Reject as `IpfsPinningMissing`. |
| Archive replica count is below threshold | Reject as `ArchiveReplicationInsufficient`. |
| Required archive provider receipt is missing | Reject as `ArchiveProviderReceiptMissing`. |
| Retrieval audit is missing, stale, or failed | Reject as `RetrievalAuditMissing` or `RetrievalAuditFailed`. |
| State snapshot misses Plonky3, epoch manifest, archive manifest, chunk, PQ, or retrieval binding | Reject as `SnapshotBindingIncomplete`. |
| Full-node pruning is attempted before all configured gates pass | Reject as `PruningBeforeArchiveFinality`. |
| Archive node pruning is attempted or enabled | Reject as `ArchiveNodePruningUnsupported`. |
| Recursive documentation packet lacks vectors, reject matrix, or PQ cadence evidence | Reject phase closeout. |
| Witness or archive material unavailable | Reject sidecar creation. |

## 🧪 Test Plan

### ✅ Unit Tests

Phase 069 MUST add unit tests for:

- Config accepts the full repository `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`.
- Config accepts the valid recursive subset only when embedded in the full
  checkpoint-contract profile.
- Unknown config fields reject.
- Recursive authority enabled rejects.
- Verified backend enabled rejects before promotion gates.
- Recursive profile that is not `hybrid_nova_plonky3` rejects.
- Missing or disabled Nova branch rejects.
- Nova cadence other than 1 rejects.
- Nova `is_pq_authoritative: true` rejects.
- Missing or disabled Plonky3 epoch branch rejects.
- Plonky3 cadence mismatch with `post_quantum.cadence_blocks` rejects.
- Plonky3 `must_prove_canonical_transition_range: false` rejects.
- Plonky3 `must_not_depend_only_on_nova: false` rejects.
- Plonky3 field/hash/recursion library mismatch rejects.
- Plonky3 security bits below 124 rejects.
- `archive_retention.celestia_is_da_only: false` rejects.
- Archive replica threshold below 3 rejects.
- IPFS archive backend without pinning rejects.
- Missing provider receipts requirement rejects.
- Missing retrieval audit requirement rejects.
- Retrieval audit interval mismatch with PQ cadence rejects.
- Snapshot disabled rejects.
- Snapshot cadence not equal to `cadence_epochs * post_quantum.cadence_blocks`
  rejects.
- Snapshot missing latest Plonky3 epoch proof binding rejects.
- Snapshot missing archive manifest or retrieval audit binding rejects.
- Archive-node pruning enabled rejects.
- Full-node pruning without dispute, Plonky3, manifest, archive replication, or
  retrieval audit prerequisites rejects.
- Recursive chain length below 3 rejects.
- Recursive target chain length above 5 rejects for required evidence.
- Empty, absolute, traversing, and colliding recursive paths reject.
- Zero and overflowed recursive proof limits reject.
- Zero and overflowed recursive sidecar, PQ anchor, and documentation packet
  limits reject.
- Zero and overflowed Nova proof, Nova archive, Plonky3 proof, and Plonky3
  sidecar limits reject.
- Zero and overflowed archive manifest, state snapshot manifest, and retrieval
  audit limits reject.
- Missing or false documentation packet flags reject closeout.
- PQ policy disabled rejects under the recursive-ready profile.
- PQ cadence `0` rejects.
- `is_pq_cadence_height(999)` is false for default cadence.
- `is_pq_cadence_height(1000)` is true for default cadence.
- Live PQ cadence before `pq_anchor_writer` rejects.
- Non-live PQ cadence at or after `pq_anchor_writer` rejects.
- Missing required PQ anchor artifacts reject.
- Missing Plonky3 epoch proof artifacts reject.
- Missing Nova chain root artifact rejects.
- PQ anchor digest mismatch rejects.
- V1 canonical admission with `pq_anchor_root` rejects.
- Public input canonical bytes produce stable golden vectors.
- Public input digest rejects field mutation.
- Statement digest mismatch rejects public input, witness, proof, and sidecar.
- Proof object codec rejects unsupported version.
- Proof object codec rejects empty proof bytes.
- Proof object codec rejects oversized proof bytes.
- Proof object codec rejects wrong backend label.
- Nova proof codec rejects PQ authority claims.
- Plonky3 proof codec rejects `depends_only_on_nova: true`.
- Epoch manifest codec rejects missing canonical statement/link/archive roots.
- Archive manifest codec rejects missing raw package, exact proof bytes,
  witness, delta, DA payload, provider receipt, or retrieval audit roots.
- Archive provider receipt rejects unpinned IPFS records.
- Retrieval audit rejects insufficient successful replicas.
- State snapshot codec rejects missing Plonky3, epoch manifest, archive
  manifest, chunk, PQ, or retrieval bindings.
- Pruning decision rejects archive-node target and early full-node pruning.
- Sidecar codec rejects unknown fields.
- Sidecar marked authoritative rejects.
- Reject reason serialization is stable.
- Measurements reject missing units, missing byte counts, missing backend label,
  and mismatched chain length.

### ✅ Integration Tests

Phase 069 MUST add integration tests for:

- Recursive sidecar attaches to a checkpoint statement without changing
  `CheckpointArtifact`.
- Current checkpoint artifact codec still rejects verified proof-system
  admission until explicit future support exists.
- `CheckpointExecInput` preserves exact `tx_proof` bytes while sidecar evidence
  is built.
- `CheckpointLink` continuity remains required and sidecar evidence cannot
  replace it.
- Settlement roots, delta roots, witness roots, and journal digest are copied
  from storage-owned material.
- Nova block proof binds the same statement digest and checkpoint link as the
  canonical artifact.
- Nova block proof chain root matches ordered per-block proof digests.
- Plonky3 epoch statement binds the full canonical 1000-block statement range.
- Plonky3 epoch proof cannot pass when it only proves Nova verifier acceptance.
- Epoch manifest binds canonical artifacts, checkpoint links, DA/archive root,
  Nova chain root, and Plonky3 proof digest.
- Archive manifest binds raw packages, exact tx proof bytes, witness archive,
  delta journal, DA payload commitment, provider receipts, and retrieval audit.
- Celestia DA reference cannot satisfy long-term retrieval without archive
  receipts and retrieval audits.
- IPFS CID reference cannot satisfy long-term retrieval unless pinned and
  receipt-bound.
- State snapshot binds latest Plonky3 epoch proof, epoch manifest, archive
  manifest, snapshot chunks, PQ anchor, and retrieval audit.
- Full-node pruning passes only after dispute, Plonky3, manifest, archive
  replication, and retrieval audit gates pass.
- Archive-node pruning is rejected.
- Provider SDK-native receipts, locator structs, or local paths cannot enter
  public input digest bytes.
- Missing witness archive reference rejects sidecar creation.
- Replayed fixture generation is deterministic.
- PQ anchor fixture binds the same statement digest as the sidecar fixture.
- PQ anchor fixture binds the same archive manifest root as the DA/archive
  fixture.
- Cadence-height audit fails when the required anchor is missing under live
  enforcement and passes when the full anchor is present.
- Cadence-height audit fails when the Plonky3 epoch proof or epoch manifest is
  missing under live enforcement.

### ✅ Chain Tests

Phase 069 MUST add chain tests for:

- Valid 3-step mock chain.
- Valid 5-step mock chain.
- Broken prior-output binding.
- Wrong statement digest in one middle step.
- Skipped step.
- Repeated step.
- Reordered step.
- Wrong output root.
- Unsupported backend label in one step.
- Missing intermediate measurement.
- Nova proof digest root mismatch.
- Plonky3 epoch range with a missing middle statement.

### ✅ Simulator Tests

Phase 069 MUST add local simulator evidence for:

- A deterministic 3-step recursive sidecar run.
- A deterministic 5-step recursive sidecar run.
- A deterministic Nova per-block proof run.
- A deterministic Plonky3 epoch proof fixture at height 1000.
- Tampered middle proof rejection.
- Tampered witness package rejection.
- Canonical checkpoint admission unchanged before and after sidecar emission.
- Measurement artifact emission with local-spike scope.
- Default PQ cadence evidence for height 999 and height 1000, including open
  epoch status at 999 and Plonky3 epoch closure at 1000.
- Archive retention evidence with at least three replica receipts.
- Retrieval audit evidence at height 1000.
- State snapshot evidence at height 10000.
- Local full-node pruning positive fixture after all gates pass and negative
  fixture before retrieval audit.
- Recursive documentation packet emission with schemas, golden-vector names,
  chain-evidence IDs, measurement IDs, PQ cadence IDs, and rejected-claim
  register.

### ✅ Property And Fuzz Tests

Phase 069 SHOULD add:

- Codec fuzz tests for malformed sidecar and proof bytes.
- Property tests that any mutation of statement digest, root, backend label, or
  chain order rejects.
- Property tests that repeated statement digests reject.
- Property tests that public input canonical bytes round-trip without changing
  digest.

### ✅ Audit Tests

Phase 069 MUST add source or documentation guards proving:

- No active code path describes recursive sidecars as canonical admission.
- No active docs claim production recursive backend security.
- No active docs claim Nova is PQ-safe.
- No active docs claim Plonky3 epoch proof can depend only on Nova.
- No active docs claim Pedersen binding is post-quantum safe.
- No active docs claim Celestia stores history forever.
- No active docs claim IPFS CID without pinning is archival persistence.
- No active docs claim recursive proofs remove all archive obligations.
- No active docs claim 100 byte proof size or 200 KB active state as current
  implementation fact.
- No Phase 069 output says DA publication proves state validity.
- No Phase 069 output says a Plonky3 epoch proof is canonical admission before
  promotion gates.
- Required recursive documentation packet sections are present.

## 🚪 Acceptance Criteria

| ID | Given | When | Then |
| --- | --- | --- | --- |
| `RCP-AC-001` | A valid checkpoint statement fixture | Public input is built | The public input digest matches the golden vector. |
| `RCP-AC-002` | A valid public input and witness | Nova adapter proves and verifies one step | `NovaCompressedBlockProofV1` verifies and remains non-authoritative. |
| `RCP-AC-003` | Three ordered checkpoint statements | Chain evidence is verified | The 3-step chain passes prior-output binding. |
| `RCP-AC-004` | Five ordered checkpoint statements | Chain evidence is verified | The 5-step chain passes prior-output binding. |
| `RCP-AC-005` | A middle step output root is tampered | Chain evidence is verified | The chain rejects with `PriorOutputMismatch` or `OutputRootMismatch`. |
| `RCP-AC-006` | A sidecar claims authority | Config or codec validates | Validation rejects fail-closed. |
| `RCP-AC-007` | A proof object exceeds size limit | Proof is decoded or written | The proof rejects before write. |
| `RCP-AC-008` | `CheckpointProofSystem::VERIFIED` is used for admission | Artifact codec validates | Artifact rejects until future verified codec support exists. |
| `RCP-AC-009` | Canonical replay includes exact transaction proof bytes | Sidecar evidence is created | Replay bytes remain unchanged. |
| `RCP-AC-010` | Measurement metadata is missing a required field | Promotion evidence is evaluated | Evidence rejects. |
| `RCP-AC-011` | A backend manifest claims PQ safety without evidence | Audit guard runs | The claim rejects. |
| `RCP-AC-012` | A provider SDK receipt is included in digest input | Public input digest is built | Digest construction rejects. |
| `RCP-AC-013` | Default PQ cadence is configured | Height 999 is evaluated | No PQ anchor is required. |
| `RCP-AC-014` | Default PQ cadence and live enforcement are active | Height 1000 is evaluated | A complete `PostQuantumCheckpointAnchorV1` is required. |
| `RCP-AC-015` | A PQ anchor binds the wrong archive manifest root | The anchor is validated | Validation rejects with `PqAnchorDigestMismatch`. |
| `RCP-AC-016` | A V1 canonical artifact contains `pq_anchor_root` | The artifact or statement is decoded | Validation rejects with `PqInlineAnchorUnsupported`. |
| `RCP-AC-017` | Phase 069 closeout is evaluated | Recursive docs packet is missing PQ cadence evidence | Closeout rejects with `RecursiveDocumentationIncomplete`. |
| `RCP-AC-018` | Active config is loaded | Nova branch is marked PQ authoritative | Config rejects with `NovaPqAuthorityUnsupported` or equivalent fail-closed config error. |
| `RCP-AC-019` | Active config is loaded | Plonky3 cadence differs from PQ cadence | Config rejects with `HybridCadenceMismatch` or equivalent fail-closed config error. |
| `RCP-AC-020` | A completed 1000-block epoch exists | `EpochRangeStatementV1` is built | It binds every canonical statement digest, checkpoint link, witness root, DA/archive root, and optional Nova chain root. |
| `RCP-AC-021` | A Plonky3 proof only verifies Nova proofs | Epoch proof is validated | Validation rejects with `Plonky3DependsOnlyOnNova`. |
| `RCP-AC-022` | A Plonky3 epoch proof omits canonical range binding | Epoch proof is validated | Validation rejects with `Plonky3CanonicalRangeMissing`. |
| `RCP-AC-023` | A valid height-1000 epoch proof exists | PQ anchor is built | Anchor binds Plonky3 epoch statement digest, proof digest, public inputs digest, archive roots, and Nova chain root. |
| `RCP-AC-024` | Nova or Plonky3 proof bytes exceed configured cap | Proof is decoded or written | Validation rejects with `ProofSizeBudgetExceeded` before write. |
| `RCP-AC-025` | Active config is loaded | `archive_retention.celestia_is_da_only` is false | Config rejects with `CelestiaPermanentStorageUnsupported` or equivalent fail-closed config error. |
| `RCP-AC-026` | An archive manifest uses IPFS | The IPFS entry is not pinned or receipt-bound | Validation rejects with `IpfsPinningMissing`. |
| `RCP-AC-027` | Archive receipts are counted | Fewer than 3 independent replicas are available | Validation rejects with `ArchiveReplicationInsufficient`. |
| `RCP-AC-028` | Retrieval audit cadence is evaluated | Height 1000 lacks `RetrievalAuditV1` | Archive completeness rejects with `RetrievalAuditMissing`. |
| `RCP-AC-029` | A state snapshot is emitted | It omits latest Plonky3 epoch proof or archive manifest binding | Validation rejects with `SnapshotBindingIncomplete`. |
| `RCP-AC-030` | A full node requests pruning | Dispute, Plonky3, manifest, archive replication, or retrieval audit gate is missing | Validation rejects with `PruningBeforeArchiveFinality`. |
| `RCP-AC-031` | An archive node requests pruning | Pruning decision is validated | Validation rejects with `ArchiveNodePruningUnsupported`. |

## 🧱 Implementation Slices

| Slice | Goal | Exit gate |
| --- | --- | --- |
| `069-01` | Land this spec and source-disposition audit. | Spec exists, links resolve, rejected claims are explicit. |
| `069-02` | Land the storage-owned `CheckpointContractConfigV1` validator for `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`. | Repo config loads; unknown fields, unsafe recursive authority, invalid PQ cadence, invalid paths, path collisions, and missing documentation gates reject. |
| `069-03` | Add public input and witness fixture contracts. | Golden vectors and mutation tests pass. |
| `069-04` | Add proof object, sidecar, codec, and reject reasons for hybrid profile. | Codec and reject taxonomy tests pass, including Nova PQ overclaim and Plonky3 Nova-only rejection. |
| `069-05` | Add Nova compressed block proof object and adapter boundary. | One-step Nova prove/verify tests pass; proof binds statement, checkpoint link, prior output, and output root. |
| `069-06` | Add Nova chain evidence verifier. | 3-step and 5-step positive and tamper tests pass; Nova chain root vectors exist. |
| `069-07` | Add Plonky3 epoch statement/proof contract. | Epoch statement binds canonical 1000-block range; Nova-only Plonky3 proof rejects. |
| `069-08` | Add epoch manifest and storage attachment. | Sidecar, Nova proof, Plonky3 proof, manifest, and PQ anchor cannot affect checkpoint admission. |
| `069-09` | Add simulator chain evidence and measurements. | Deterministic Nova and Plonky3 simulator evidence emits and rejects tampering. |
| `069-10` | Add Plonky3 PQ epoch cadence contract and fixtures. | Height 999/1000 cadence tests, epoch proof tests, anchor digest tests, and stage-gate tests pass. |
| `069-11` | Add Archive Retention Layer, retrieval audits, and provider receipt contracts. | Celestia-as-archive, unpinned IPFS, insufficient replicas, and missing retrieval audit reject. |
| `069-12` | Add state snapshot and pruning contracts. | Snapshot binding tests pass; full-node pruning requires all gates; archive-node pruning rejects. |
| `069-13` | Add recursive documentation packet and audit guards. | No production admission, Nova PQ, Plonky3 Nova-only, size, DA-overclaim, Celestia-forever-storage, IPFS-without-pinning, or proof-as-storage claim survives and required docs are present. |

## 🧾 Required Artifacts

| Artifact | Required path or owner | Required content |
| --- | --- | --- |
| Spec | `.planning/phases/69-Recursive-Proof/069-TODO.md` | This document. |
| Source audit | Phase 069 closeout or audit doc | Source disposition and rejected-claim register. |
| Config fixture | `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml` | Full active config with canonical, recursive, DA, PQ, retention, paths, limits, and documentation gates. |
| Dependency manifest | Future recursive proof crate `Cargo.toml` or Phase 069 closeout packet | Exact pinned `nova-snark`, `p3-*`, `ipfs-api-backend-hyper`, and Kubo binary versions or reviewed git revisions, plus documentation links and compatibility notes. |
| Storage config validator | `crates/z00z_storage/src/checkpoint/contract_config.rs` | Strict `CheckpointContractConfigV1` loader, schema validator, path validator, authority gate, PQ cadence helper, and negative tests. |
| Golden vectors | Storage or recursive proof tests | Statement, public input, Nova proof, Plonky3 epoch statement/proof, epoch manifest, proof object, and sidecar digests. |
| Sidecar objects | `artifacts/checkpoints/recursive_shadow` in local runs | `RecursiveCheckpointSidecarV1` bytes and JSON report if needed. |
| Nova block proofs | `artifacts/checkpoints/nova_block` in local runs | `NovaCompressedBlockProofV1` bytes, digest, statement binding, checkpoint-link binding, and prior-output binding. |
| Chain evidence | Simulator output | 3-step and 5-step `RecursiveCheckpointChainEvidenceV1`. |
| Epoch manifest | `artifacts/checkpoints/epoch_manifest` in local runs | `EpochManifestV1` binding statement roots, canonical artifact roots, link roots, archive roots, Nova root, and Plonky3 proof digest. |
| Archive manifest | `artifacts/checkpoints/archive_manifest` in local runs | `CheckpointArchiveManifestV1` binding raw package, exact tx proof bytes, witness, delta, DA payload, provider receipts, retrieval audit, and content-address root. |
| Archive receipts | `artifacts/checkpoints/archive_receipt` in local runs | `ArchiveProviderReceiptV1` entries for archive replicas, including pinned IPFS receipts when IPFS is used. |
| Retrieval audits | `artifacts/checkpoints/retrieval_audit` in local runs | `RetrievalAuditV1` proving enough configured archive replicas remain retrievable. |
| State snapshots | `artifacts/checkpoints/state_snapshot` in local runs | `StateSnapshotV1` binding state, settlement, Plonky3, epoch manifest, archive manifest, chunk, PQ, and retrieval roots. |
| Pruning decisions | Storage checkpoint pruning tests | Positive local full-node pruning decision after gates and negative archive-node/early-pruning decisions. |
| Plonky3 epoch proof | `artifacts/checkpoints/plonky3_epoch` in local runs | `Plonky3EpochProofV1` at default cadence heights with canonical range proof flag and Nova-only rejection evidence. |
| Measurements | Simulator or benchmark output | `RecursiveCheckpointMeasurementV1` with local-spike label. |
| PQ anchor object | `artifacts/checkpoints/pq_anchor` in local runs | `PostQuantumCheckpointAnchorV1` at default cadence heights binding Plonky3 epoch proof and Nova chain root when live enforcement is active. |
| PQ cadence vectors | Storage, recursive proof, or simulator tests | Height 999 no-anchor fixture, height 1000 anchor fixture, stage-gate positive and negative cases. |
| Recursive docs packet | Phase 069 closeout | Schemas, vectors, sidecar and chain IDs, measurements, PQ cadence evidence, backend manifest, rejected-claim register. |
| Reject report | Test output or closeout | Stable reject reason matrix. |
| Audit report | Phase 069 closeout | Overclaim, PQ, DA, and canonical-admission guard results. |

## 🔭 Future Backend Promotion

📌 Phase 069 closure does not enable production recursive proofs. A future
verified backend needs a separate accepted spec and all of these gates:

- Stable verified proof object.
- Stable verifier API with typed rejects.
- Artifact codec support for verified proofs.
- Nova and Plonky3 backend adapters behind traits.
- Deterministic 3 to 5 step chain over the same statement.
- Negative tests for wrong root, wrong delta, wrong witness, wrong proof, wrong
  link, unsupported backend, and mixed-era artifacts.
- Benchmarks for proof size, prover time, verifier time, memory, and witness
  size.
- Security review of assumptions, parameters, implementation, and side-channel
  risks.
- Rollback procedure that disables the backend without changing the statement.
- Evidence that canonical replay and data-retention obligations remain safe.
- Evidence that archive manifests, provider receipts, retrieval audits, and
  snapshots preserve retrievability after DA windows expire.
- Evidence that pruning is local full-node cleanup only and cannot delete
  archive replicas.
- Third-party or equivalent security review of Plonky3-recursion integration
  before any canonical verified-admission enablement.

## 🚫 Non-Negotiable Rejections

Phase 069 MUST reject any design that:

- Makes recursive sidecars authoritative.
- Enables `CheckpointProofSystem::VERIFIED` without future codec support.
- Removes exact transaction proof bytes from canonical replay.
- Removes raw, witness, delta, or archive material before retention policy.
- Replaces spend verification or range-proof verification.
- Uses link tag proofs as checkpoint transition authority.
- Treats DA availability as state-transition validity.
- Treats Celestia DA as indefinite archive storage.
- Treats IPFS CID without pinning, receipts, and retrieval audits as archival
  persistence.
- Treats recursive proofs as replacement for all retrievable history.
- Treats watcher evidence as settlement authority.
- Imports provider SDK types into statement or public input bytes.
- Claims PQ safety from Pedersen binding, DLP assumptions, or unproved folding.
- Claims Nova compressed proofs are PQ-safe.
- Claims Plonky3 PQ epoch proof can depend only on Nova proof verification.
- Claims production proof sizes or state sizes without measured backend evidence.
- Adds a second checkpoint theorem.

## ✅ Phase 069 Done Definition

Phase 069 is complete only when:

- `RecursiveCheckpointPublicInputV1`, `RecursiveCheckpointWitnessV1`,
  `RecursiveCheckpointProofV1`, `NovaCompressedBlockProofV1`,
  `EpochRangeStatementV1`, `Plonky3EpochProofV1`, `EpochManifestV1`,
  `RecursiveCheckpointSidecarV1`, `RecursiveCheckpointRejectReasonV1`,
  `RecursiveCheckpointMeasurementV1`, and `RecursiveCheckpointChainEvidenceV1`
  are specified and implemented or tracked as exact implementation tasks.
- `CheckpointArchiveManifestV1`, `ArchiveProviderReceiptV1`,
  `RetrievalAuditV1`, `StateSnapshotV1`, and `PruningDecisionV1` are specified
  and implemented or tracked as exact implementation tasks.
- The Nova adapter proves and verifies one step.
- The Plonky3 epoch adapter has a concrete implementation task and rejects
  Nova-only epoch proofs.
- Local 3-step and 5-step chain evidence passes.
- Tampered chain evidence rejects.
- Canonical checkpoint admission remains unchanged.
- Exact transaction proof bytes remain preserved.
- `CheckpointContractConfigV1` loads the repository config and config gates
  prevent recursive authority, invalid PQ cadence, invalid paths, path
  collisions, and incomplete documentation closeout.
- PQ cadence gates enforce height 1000 anchor requirements only at/after
  `pq_anchor_writer` and keep pre-stage policy declared-only.
- `PostQuantumCheckpointAnchorV1` is specified, tested, and documented as a
  Plonky3 epoch audit object, not canonical admission authority.
- Celestia is specified, tested, and documented as DA-only.
- IPFS is allowed only with pinning, provider receipts, and retrieval audits.
- State snapshot and local full-node pruning gates are specified and tested.
- `RecursiveCheckpointDocumentationPacketV1` exists or is tracked as an exact
  implementation task with all required sections.
- Measurements are emitted as local spike evidence only.
- Audit guards reject production backend, PQ, DA, and size overclaims.
- All required unit, integration, chain, simulator, and audit tests pass.

## 📎 Related Documents

- `docs/tech-papers/Recursive-Ready-Checkpoint-Contract.md`
- `.planning/phases/068-Checkpoint-Contract/068-TODO.md`
- `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`
- `crates/z00z_storage/src/checkpoint/mod.rs`
- `crates/z00z_storage/src/checkpoint/contract_config.rs`
- `crates/z00z_storage/src/checkpoint/artifact_final.rs`
- `crates/z00z_storage/src/checkpoint/artifact_stmt.rs`
- `crates/z00z_storage/src/checkpoint/exec_input.rs`
- `crates/z00z_storage/src/checkpoint/link.rs`
