# Phase 069-051 T2 cryptographic audit 2: theorem and scalable IVC formulation

Status: `DOUBLECHECK COMPLETE / CONDITIONALLY SOUND DESIGN / IMPLEMENTATION BLOCKED — monolithic arena refuted; Sections 22–26 control every earlier conflict; the corrected theorem is internally consistent under its explicit assumptions, but T2–T4 and the live Nova dependency are not yet implemented or certified`

Audit verdict as of 2026-07-13:

- the 5.6 GiB failure is caused by an invalid representation invariant, not by
  the checkpoint theorem;
- a 67,108,864-byte R1CS array is neither stated by `069-TODO.md` nor required
  by Nova;
- the current native predicate, witness schema, positive fixture, and partial
  R1CS do not implement `CheckpointTransitionConsistencyV1`;
- the legacy eight-byte Goldilocks packing has an executable deterministic
  alias and cannot be the exact-byte commitment for the repaired theorem;
- the V1 semantic settlement root also uses that weak sponge and is distinct
  from the canonical SHA-JMT backend root; the sole low-memory live repair is
  an explicit V2 settlement-root generation derived from the HJMT root at one
  cutover, not two continuing root engines;
- the low-memory path is a versioned, typed semantic transition trace consumed
  by one small uniform Nova micro-step relation, with one completed recursive
  segment per checkpoint block;
- a real pinned-Nova release experiment completed setup, IVC, compression, and
  verification for a SHA-bound 64-byte micro-step at 261,984 KiB runtime peak
  RSS; this falsifies the claim that the bounded step itself inherently needs
  multi-gigabyte memory, but is not evidence that the full relation exists;
- `069-051-PLAN.md` is not executable as written. Its full-cap arena and exactly
  one monolithic library fold assumptions must be replaced before T1 resumes.

This is a correction of one theorem and one implementation path. It is not a
permission to add a parallel circuit, native-validity shim, reduced cap, or
digest-only transition proof.

Authority: this file is an audit and recovery record. It does not define a
second checkpoint theorem, backend, public input, or storage contract. The live
authorities remain `069-TODO.md`, the referenced Phase 069 design/whitepapers,
and the canonical storage-owned checkpoint types.

## 1. Audit question

Does `CheckpointTransitionConsistencyV1` require one R1CS instance containing a
fixed, byte-by-byte, 67,108,864-byte witness arena, or can the same theorem be
proved by a bounded sequence of uniform Nova IVC steps whose cumulative input
is capped at 64 MiB?

The distinction is decisive:

- `B`: maximum canonical witness package size in bytes;
- `C`: constraints in one uniform Nova step circuit;
- `N`: number of Nova steps used to consume one checkpoint witness;
- `M`: peak prover memory.

The failed implementation assumed `C = Omega(B)` by allocating at least one
field variable for every possible byte in a single step. Nova IVC only requires
that each invocation use the same step relation and state arity. It does not, by
itself, require all `B` bytes to be present in one invocation. A streaming
formulation targets `C = O(K)` and `N = O(R + H + I + Q)`, where `K` is one
small fixed event/hash-compression budget, `R` is the number of replay
operations, `H` is the total number of state-HJMT path-node operations, `I` is
the total number of replay-ID plus sorted-ID permutation events, and `Q` is
the number of SHA-256 compression blocks for typed commitment material. Path
depth affects `N`, never the matrix size `C`: every HJMT path node is itself
decomposed into the same generic begin/hash/commit subtrace. Opaque
transaction-proof bytes are hashed once by canonical storage
because this theorem constrains their digest, not their verifier. In contrast,
variable-length old/new leaf bytes that determine the live HJMT value hash must
either be consumed by bounded SHA blocks or be replaced at an explicit
versioned state-leaf cutover. The selected minimum-change route streams those
leaf bytes, so worst-case total folds may be `O(B / 64)` while peak shape and
memory remain independent of `B`. The proof must still constrain every
semantic datum and every retained-content digest.

## 2. Canonical theorem reconstructed from live authority

The live authority's intended semantic theorem is currently named
`CheckpointTransitionConsistencyV1`. The proof must establish that one
semantic property only:

```text
CheckpointTransitionConsistency(
    canonical context,
    canonical statement/public input,
    canonical witness material,
    pinned predicate/profile/parameter identities
) = accepted
```

For an accepted checkpoint, the constrained relation must bind and check:

1. the exact `CheckpointTransitionStatementV1` and its canonical digest;
2. context, predicate, circuit/profile/spec, and verifier-parameter identity;
3. ordered replay input, exact transaction-proof bytes or their exact
   in-circuit canonical digest where the predicate requires digest binding;
4. the replay result and its state/delta/witness commitments;
5. HJMT membership/non-membership paths, old/new leaves, journal lineage, and
   root continuity;
6. prior and resulting state and settlement roots;
7. exact row/path/byte counts and rejection of omission, duplication,
   reordering, padding ambiguity, and trailing unconsumed data;
8. the Nova running-state transition and final statement-bound receipt.

The public input remains storage-owned and must be versioned in its existing
canonical owner. Because V1's committed byte/hash semantics are defective, the
incomplete V1 shape cannot be silently mutated while retaining a V1 type/digest.
The corrected executable types/bytes must use an explicit V2 era while V1
becomes read-only. This versions the encoding and executable predicate identity;
it does not add a second semantic theorem or permit simultaneous selectable
live prover paths. A streaming trace is a witness decomposition of the same
transition-consistency property, not a smaller public statement.

### 2.1 Formal relation, before choosing a backend layout

Let `B0` be the canonical SHA-JMT definition-tree root carried from the V2
cutover/running recursive state, and let
`S0 = SettlementStateRootV2(layout, policy, B0)` equal
`statement.prev_settlement_root`; let
`E = [e_0, ..., e_(n-1)]` be the ordered execution rows; and let `T` be a
deterministic typed authentication/update trace. The relation to prove is:

Here `SettlementStateRootV2(...)` denotes the V2 derivation function/value of
the existing `SettlementStateRoot` owner with `RootGeneration::SettlementV2`;
it does not name a second Rust root type.

```text
R(context, params, statement, recursive_public_input; E, T, archive_bindings)
  := ContextBound(context, params, recursive_public_input)
   ∧ StatementBound(statement, recursive_public_input)
   ∧ CanonicalAndBounded(E, T, archive_bindings)
   ∧ TxCommit(E) = statement.tx_data_root
   ∧ ReplayHjmtAndTypedEffects(B0, E, T)
       = (B1, spent_delta, created_delta)
   ∧ SettlementStateRootV2(layout, policy, B0)
       = statement.prev_settlement_root
   ∧ SettlementStateRootV2(layout, policy, B1)
       = statement.new_settlement_root
   ∧ DeltaCommit(spent_delta, created_delta) = statement.delta_root
   ∧ HjmtWitnessCommit(T) = statement.witness_root
   ∧ JournalCommit(T) = statement.journal_digest
   ∧ FinalStatementAndLinkBindingsHold
   ∧ RecursivePredecessorAndFinalStateHold.
```

`Replay` is not a boolean supplied by the host. For every transaction ordinal
and item ordinal it must constrain the exact canonical order below:

1. bind the row's input refs, outputs, and collision-safe digest of the retained
   transaction-proof bytes;
2. reject empty rows, duplicate inputs, duplicate spent IDs across the block,
   and duplicate output IDs across the block;
3. authenticate each consumed old leaf and its path under the root required by
   the canonical replay schedule;
4. constrain each deletion/update and calculate the next running HJMT root;
5. authenticate output absence under the corresponding running root, calculate
   the output leaf hash, insert it, and calculate the next running root;
6. append the exact spent/created delta record at its constrained ordinal;
7. finish only after all declared rows, operations, paths, and witness nodes are
   consumed exactly once.

The existing V1 `CheckRoot` and `SettlementStateRoot` bytes are wrappers over
the legacy semantic-model root, while the SHA-JMT `backend_root` reconstructed
by batch paths is distinct. The corrected live generation makes that
distinction explicit once at cutover and then uses the SHA-256-V2 function
above as its sole canonical settlement/check root. It must not expose a raw
backend root as an untyped alias. V1 `root_bind`/`checkpoint_bind` remain
read-only archive checks, not the security anchor for the repaired proof.

### 2.2 What is computational and what is retained content

The theorem distinguishes three classes of data:

| Class | Circuit obligation | Storage obligation |
| --- | --- | --- |
| Typed replay fields and HJMT update data | Parse as typed values and constrain all transition semantics. | Produce them from the canonical builder/snapshot/store path. |
| Opaque transaction-proof bytes | Constrain their versioned collision-safe content digest inside the ordered row; do not re-run the out-of-scope transaction verifier. | Retain the exact bytes and recompute the same content digest on write/read/audit. |
| Raw HJMT batch codec bytes | Constrain typed membership/non-membership/update semantics and bind the raw-blob content digest in the semantic witness root. The codec/DAG layout itself is not the transition relation. | Strictly decode/re-encode, retain, and content-address the exact blob. |
| Canonical old/new leaf bytes | Constrain every semantically used field and reproduce the exact SHA-256 value/leaf hash that enters the existing HJMT root. Variable payload segments are consumed by bounded SHA-block events. | Emit canonical bytes and ordered update paths from the same builder/store application; do not reimplement a second tree. |

Consequently, removing the raw padded arena does not remove witness soundness.
It removes an unnecessary proof-of-preimage for opaque retention bytes while
keeping every state-changing fact constrained.

### 2.3 Leaf/root binding trilemma

For an old/new leaf whose canonical encoding contains variable ciphertext or
proof bytes, these three requirements cannot all be waived:

1. preserve the existing `Sha256Jmt` root;
2. prove that the semantic leaf used by replay is the leaf committed by that
   root;
3. avoid trusting a host-supplied leaf/value digest.

There are exactly two sound families of solution:

- constrain SHA-256 over the canonical leaf preimage, streamed through a fixed
  64-byte compression lane; or
- create an explicit storage-state version whose HJMT leaf contains only fixed
  semantic fields plus collision-safe content digests, with a fully specified
  state migration/cutover.

Merely passing `leaf_digest` beside typed fields is unsound: a malicious prover
can use the digest that opens the public root and unrelated typed fields unless
the circuit proves their commitment relation. Creating a separate recursive
projection root has the same defect unless a translation proof links it to the
canonical HJMT root, and would violate the requested single canonical path.

The recovery therefore selects streamed canonical leaf hashing. A later
digest-normalized HJMT leaf version may be considered only as a storage-wide
state migration, never as a Phase 069-only backend shortcut.

This removes the false **memory** lower bound, not the real **work** lower
bound. If a theorem proves a collision-resistant hash of an arbitrary `L`-byte
preimage and does not trust a precomputed digest, either its relation processes
those `L` bytes or it verifies another sound proof that did. Thus the selected
route has constant peak circuit shape and `Omega(L)` total hashing work for
variable canonical leaf bytes. No honest reformulation can make both costs
constant without changing what is proved or introducing an inner proof.

## 3. Claim register

Verdicts: `SUPPORTED`, `REFUTED`, `OPEN`, or `REQUIRES CHANGE`.

| ID | Claim | Evidence/falsifier | Verdict |
| --- | --- | --- | --- |
| C-001 | A valid checkpoint witness is bounded and may include exact replay rows, transaction-proof bytes, and HJMT paths. | `069-TODO.md` Witness Contract and `CheckpointTransitionWitnessMaterialV1`. | SUPPORTED |
| C-002 | The 64 MiB bound requires a 64 MiB padded byte array inside one R1CS step. | No such requirement appears in the canonical statement or Nova block contract. It was introduced by `069-051-PLAN.md`/the recovery implementation. | REFUTED |
| C-003 | Nova requires a fixed circuit shape for repeated folds. | Pinned `nova-snark` exposes one `StepCircuit` type, fixed `arity()`, and creates one R1CS shape in `PublicParams::setup`. | SUPPORTED |
| C-004 | Fixed Nova shape means the maximum total computation must fit in one step. | `RecursiveSNARK::prove_step` accepts a new witness-bearing circuit value on every step while checking against the same public parameters/shape. | REFUTED |
| C-005 | A native validity boolean/digest can replace replay/HJMT semantics. | This would make the actual transition unconstrained. Opaque transaction-proof bytes may be represented by a constrained collision-safe content digest only because Phase 069 does not interpret their cryptographic semantics. | REFUTED |
| C-006 | Exact bytes may be consumed over many constrained steps while an in-circuit accumulator binds their framing, order, length, and digest. | This can preserve the committed byte string, but it is unnecessary for opaque retention bytes once a collision-safe digest is constrained and storage separately verifies/retrieves the content. | SUPPORTED BUT NOT SELECTED |
| C-007 | The observed 5.6 GiB RSS/abort proves the checkpoint theorem is infeasible. | It proves only that the current byte-per-variable monolithic `ShapeCS` representation exceeds the 8 GiB test limit before relation synthesis. | REFUTED |
| C-008 | Making the shape builder more memory-efficient is sufficient. | Nova commitment-key size and per-step witness work still scale with the monolithic constraint count. A bounded semantic step relation is required; raw-byte streaming is only a fallback for bytes actually interpreted by the theorem. | REFUTED |
| C-009 | Native validation plus a small digest-only Nova circuit proves the theorem. | Native validation is not cryptographic enforcement and permits a forged witness/proof if the relation does not constrain the validation semantics. | REFUTED |
| C-010 | One small uniform state-machine circuit can represent typed commitment binding, replay, HJMT, and finalization. | Plain Nova permits one uniform circuit; selectors/program counter can select bounded operations. Inactive branches, phase transitions, and termination remain implementation obligations. | SUPPORTED |
| C-011 | SuperNova is needed because operations are non-uniform. | It is one alternative, but Phase 069 names it future-only and selects plain Nova. A uniform universal micro-step may avoid changing backend authority. | OPEN |
| C-012 | The Nova block cadence requires exactly one internal `RecursiveSNARK::prove_step` call per block. | `069-TODO.md` currently says both “one statement-bound IVC step per block” and “fold one step per block.” Under a variable-size complete transition this literal reading conflicts with bounded shape/memory unless a nested proof is introduced. | REQUIRES CHANGE |
| C-013 | The current native `CheckpointTransitionConsistencyV1::evaluate` re-executes the accepted state update. | The implementation compares public/envelope fields and calls `witness.validate`; it does not apply replay operations, calculate the new state/settlement roots, or prove delta effects. | REFUTED |
| C-014 | The current witness schema is already sufficient for the full live predicate. | It carries replay rows and HJMT proof blobs, but the live authority additionally requires old/new leaves, spent/nullifier updates, result roots, and enough data for re-execution. An explicit deterministic transition trace/update witness is missing. | REFUTED |
| C-015 | The current `hash_zk` byte-to-Goldilocks map is injective before the cryptographic permutation. | It packs arbitrary 8-byte words as `u64`; Goldilocks identifies values modulo `p = 2^64 - 2^32 + 1`. Therefore `x` and `x + p` are the same field input whenever both are `u64`. | REFUTED |
| C-016 | The private R1CS `hash_zk` gadget implements the live native hash for every byte string. | `GoldilocksVar::alloc` rejects values `>= p`, while native `Goldilocks::new` accepts non-canonical `u64` representatives. The gadget therefore fails synthesis on some live inputs rather than matching native reduction/equality. | REFUTED |
| C-017 | A streaming circuit alone repairs the exact-byte theorem. | Streaming repairs peak memory, but it cannot repair a collision-prone public byte commitment. Exact-byte soundness also requires a versioned injective byte-to-field encoding or a collision-resistant byte commitment bound by the statement. | REFUTED |
| C-018 | The current positive predicate fixture demonstrates a valid state transition. | The fixture independently assigns arbitrary repeated-byte `new_root`, `delta_root`, `witness_root`, and `journal_digest` values; it never applies the replay row to the HJMT store, yet `evaluate` accepts it. | REFUTED |
| C-019 | “Exact proof bytes must remain available” means every raw proof byte must be allocated in the transition circuit. | `069-TODO.md` separately requires retained exact bytes and a circuit witness schema containing transaction proof-byte **digests**. End-to-end transaction verifier replacement is out of scope. | REFUTED |
| C-020 | A transition proof must prove knowledge of every archived byte preimage. | It must prove the semantic transition and constrain collision-resistant content digests that bind retained bytes. Archive retrievability/canonical content verification is a separate storage obligation and cannot create transition validity. | REFUTED |
| C-021 | Raw HJMT blob codec bytes are the semantic HJMT relation. | The semantic relation is membership/non-membership/update under exact hash/root rules. A typed path/update witness can prove it directly; codec bytes remain content-addressed archive material. | REFUTED |
| C-022 | Phase closure requires a successful real proof of the maximum 64 MiB package on the audit host. | The live failure model says prover memory/witness/time exhaustion produces a typed shadow-job resource outcome and must not block canonical admission. The cap bounds input; it is not a guaranteed local proving SLA. | REFUTED |
| C-023 | A typed resource rejection means the circuit supports only a reduced theorem. | A uniform trace relation can define the complete `<= 64 MiB` domain while a concrete worker rejects a job before proving when its configured resource budget is insufficient. Soundness is unchanged; evidence liveness is degraded and must be reported. | REFUTED |
| C-024 | Current builder-produced HJMT batches authenticate the prior root expected by `recursive_witness.rs`. | `build_stmt_core_v1` requires the first batch root to equal `draft.new_settlement_root()`, while `recursive_witness.rs` requires every batch root to equal its field named `prior_settlement_root`. | REFUTED |
| C-025 | `BatchProofBlobV1::check_contract_v1` proves a state transition. | It proves inclusion/non-existence/deletion openings against the batch header's one root and exact table usage. It does not consume ordered checkpoint operations and derive a second root. | REFUTED |
| C-026 | The current witness constructor has enough inputs to reproduce `build_cp_draft`. | It receives statement/core/link/exec and post-state batches, but not `PrepSnapshot`, `PrepReplayEntry`, the draft, typed old/new leaves, or a running-root update trace. | REFUTED |
| C-027 | `chain_length in 3..=5` is a production proof invariant. | Three- and five-step chains are required evidence fixtures. A per-block proof cannot require its final future chain length to be known and bounded to five. | REFUTED |
| C-028 | `max_witness_bytes` alone defines a practical fixed semantic R1CS shape. | The execution row vectors have no storage-level per-row input/output caps, and the batch format permits large witness DAGs. A byte cap gives a finite theoretical maximum only by returning to `O(bytes)` allocation. | REFUTED |
| C-029 | `limits.max_batch_ops = 1000` currently bounds the transition witness operations. | The value is loaded into `RecursiveCircuitProfileV1`, but `CheckpointExecInput`, `CheckpointTransitionWitnessMaterialV1`, and the partial R1CS do not enforce total input/output/update operations against it. | REFUTED |
| C-030 | The 15-row predicate map uniquely commits to executable transition semantics. | Its digest hashes English labels/column names. Different incomplete implementations can claim the same map; a versioned executable trace grammar/circuit digest is required. | REFUTED |
| C-031 | The legacy-context constructors preserve the one canonical context path. | Public constructors synthesize `legacy_context_digest()` while newer constructors accept explicit context. That is a second live construction path unless deprecated/read-only at a version boundary. | REFUTED |
| C-032 | `prior_recursive_output_root` is fully bound in the recursive public input. | It is optional in the statement core, but the current public input exposes only `prior_output_root = statement.prev_root`; the predecessor Nova accumulator needs an explicit versioned binding. | REFUTED |
| C-033 | The typed semantic design can keep peak R1CS memory independent of the 64 MiB retention cap. | Exact raw bytes are hashed/retained by storage; Nova consumes bounded typed rows/digests and decomposes SHA/HJMT work across uniform folds. Per-fold shape is independent of total archive bytes. | SUPPORTED |
| C-034 | Multiple internal folds necessarily mean multiple block proofs. | Intermediate folds are an incomplete segment state. Only `FINALIZE_BLOCK` may create the one statement-bound block snapshot/receipt and advance to the next block. | REFUTED |
| C-035 | A corrected relation can keep V1 commitment semantics unchanged without processing raw bytes. | Because V1 raw-byte commitments are aliased, a proof must either process the same raw bytes into both legacy and safe commitments or move to an explicit versioned commitment/statement era. The latter is the lower-complexity canonical route. | REFUTED |
| C-036 | Four 64-bit output words give the current rate-seven sponge about 256-bit collision resistance. | Width eight with rate seven leaves capacity one Goldilocks element. Generic sponge collision security is bounded near half the capacity, about 32 bits here, regardless of longer output. | REFUTED |
| C-037 | The existing real Nova fold is a checkpoint-transition proof. | `adapter.rs` aliases `NovaCircuit` to `TrivialCircuit`, proves one state-preserving synthetic step, and can issue only `ReceiptScopeV1::LibrarySmoke`. | REFUTED |
| C-038 | Pinned Nova compressed verification can bind exact execution length and endpoints. | `CompressedSNARK::verify(vk, num_steps, z0)` absorbs the public-parameter digest, exact `num_steps`, `z0`, and the proof-carried final state before verifying both relaxed R1CS proofs. The caller must compare the returned final state with the canonical statement. | SUPPORTED |
| C-039 | Restarting an independent Nova instance at every block and hashing the prior proof externally preserves recursive continuity. | Without verifying the predecessor proof inside the new circuit, an external digest is only an asserted value. The selected path keeps one continuous `RecursiveSNARK` and compresses snapshots at finalized block boundaries. | REFUTED |
| C-040 | A continuous Nova state must retain every historical witness and therefore grows with chain length. | The pinned `RecursiveSNARK` stores fixed-shape running relaxed witnesses/instances, `z0`, current `zi`, and a step counter; it is serializable and does not store a vector of prior steps. Private-state size still requires measurement and bounded recovery encoding. | REFUTED |
| C-041 | A prover-supplied HJMT leaf digest can replace hashing the canonical leaf bytes. | It can open the root while being unrelated to separately supplied typed fields. Soundness requires streamed preimage hashing or an explicit digest-normalized state-leaf version/cutover. | REFUTED |
| C-042 | A Rust `match` on the witness opcode inside `StepCircuit::synthesize` is a valid uniform instruction circuit. | Constraint allocation would depend on the witness/circuit value, so setup could record only the branch used during setup. All opcode selectors and branch constraints must be allocated in every synthesis and gated algebraically. | REFUTED |
| C-043 | A bounded SHA/Nova step itself needs multi-gigabyte runtime memory. | A release Pallas/Vesta experiment with a SHA-bound 64-byte step completed setup, IVC, compression, and verification with 55,205 primary constraints and 261,984 KiB runtime peak RSS. This supports the memory architecture, not full checkpoint correctness. | REFUTED |
| C-044 | The first call to `RecursiveSNARK::prove_step` synthesizes the first application of the step function. | `RecursiveSNARK::new(pp, circuit_0, z0)` synthesizes the base/first step; the first `prove_step` call when `i == 0` only changes the internal count to one. Runner indexing and receipts must use `num_steps()` and exact event ordinals. | REFUTED |
| C-045 | Nova compression or SHA-256 makes the block proof post-quantum/end-to-end transaction-valid. | The selected Nova/Spartan/IPA lane is classical, SHA-256 only supplies hash binding, and Phase 069 does not verify nested authorization/range/signature semantics. | REFUTED |
| C-046 | One `HJMT_NODE` event can both authenticate and update one JMT level through one SHA-256 compression. | The exact internal-node preimage is `b"JMT::IntrnalNode" || left || right`, which is 80 bytes before padding and takes two compression blocks. An update must also derive both old and new parents. Each hash therefore needs an explicit generic hash subtrace; a node commit may advance only after all role-bound results are constrained. | REFUTED |
| C-047 | A fixed-depth duplicate-set authentication path can be allocated inside one semantic micro-step without affecting the low-memory argument. | That would make per-step shape scale with the path depth and duplicate SHA gadgets. The spent/output uniqueness roots must use the same streamed authenticated-path begin/hash/commit grammar as state HJMT nodes. | REFUTED |
| C-048 | Writing `ordered_root(items)` is enough to freeze a canonical V2 checkpoint commitment. | It leaves tree-vs-linear aggregation, count width, item framing, and SHA preimage unspecified and could create another hash owner. V2 uses the existing `sha256_256` framing byte-for-byte and one explicitly typed ordered-commit helper in the storage owner. | REFUTED |
| C-049 | Constant peak memory makes every permitted 64 MiB witness practical to prove on the local worker. | Even the raw unframed 64 MiB lower bound is 1,048,577 compressions; exact V2 role framing requires at least 1,048,578 for one part and more for multiple parts, before semantic/JMT work. The theorem remains supported, but worker preflight may return the already-authorized typed resource outcome; it may never report proof success or reduce the live input cap. | REFUTED |
| C-050 | Hashing host-supplied canonical leaf bytes and separately constraining typed leaf fields binds those fields to the HJMT root. | The two witnesses can differ unless the circuit constrains the exact serialization relation. The current terminal leaf uses tag byte plus bincode-v2 standard encoding, including variable-length fields. The trace must construct/check those exact bytes from constrained fields and lengths before hashing. | REFUTED |
| C-051 | `SettlementStateRoot` in the checkpoint statement is the SHA-JMT root reconstructed by HJMT path proofs. | Storage assigns V1 `settlement_root = plan.root`, while batch paths reconstruct `header.backend_root`; the header separately binds the pair. The repaired live generation must either prove both roots or define one explicit V2 settlement root from the canonical HJMT root. | REFUTED |
| C-052 | Keeping the V1 semantic settlement root beside a strong JMT backend root is the lowest-cost repair. | `SettlementModel::root` recursively hashes variable-length model lists with the defective V1 Poseidon2 sponge. Proving it requires a second streaming Poseidon lane and potentially whole-model witness data, while still retaining a weak public root. A one-time root-generation cutover to a SHA-256-V2 commitment of the canonical HJMT definition root is smaller and gives one live root. | REFUTED |
| C-053 | Proof-local spent/output sets may reuse the live JMT leaf/internal domains unchanged. | That would permit cross-tree role confusion between canonical state paths and ephemeral uniqueness paths. The generic path machine is shared, but proof-local leaf/node/empty hashes must use explicit V2 domains and a constrained `SPENT`/`OUTPUT` tree-kind tag. | REFUTED |
| C-054 | A depth-256 authenticated set is the best primary duplicate detector for a linear Nova trace. | It is sound but requires hundreds of parent hashes per ID. A two-pass strictly sorted list plus Fiat–Shamir grand-product permutation over the existing 16 `u16` ID limbs gives constant state and `O(number of IDs)` folds. Authenticated sets remain a domain-separated fallback only. | REFUTED |
| C-055 | Existing `BatchProofHeaderV1`/`BatchProofBlobV1` can remain the live post-cutover HJMT witness envelope. | Its verifier requires encoding V1, `RootGeneration1`, and V1 `root_bind`/`checkpoint_bind`. It cannot represent `SettlementStateRootV2` securely. The same settlement owner needs one V2 batch/trace envelope; V1 becomes decode/audit-only. | REFUTED |
| C-056 | Putting the last V1 root and first V2/HJMT root beside each other in `z0` proves they represent the same state. | It is only a pair of assertions. The cutover needs a deterministic storage-wide rebuild/migration from the canonical state, an authority-pinned migration manifest with both expected roots, and local fail-closed verification/restart evidence (or a separate translation proof). | REFUTED |

## 4. Confirmed logical errors in the current recovery formulation

### E-001 — Capacity/shape category error

`witness_byte_length <= 64 MiB` is a storage and denial-of-service bound. It was
silently promoted to `one R1CS step has 64 MiB of byte slots`. The latter is an
implementation choice, not a theorem consequence.

Required correction: retain the total byte bound, but consume the canonical
witness through fixed-size, ordinal-bound micro-steps.

### E-002 — Fixed-shape/one-shot equivocation

Nova requires the same step relation for every fold. It does not require a
single step per whole variable-length witness. The step relation can be a
uniform state machine with fixed chunk/path capacity and a constrained program
counter.

Required correction: define the fixed shape at the micro-step level and prove
the unique start-to-finalize trace for each checkpoint.

### E-003 — Recursion removed from the resource model

The current `synthesize_full_cap` attempts to synthesize the complete maximum
package before any replay/HJMT/Nova fold. This uses Nova as a wrapper around a
monolith rather than as incremental verifiable computation.

Required correction: use repeated real `RecursiveSNARK::prove_step` calls over
the same circuit shape; compression cadence remains distinct from fold cadence.

### E-004 — A local allocation failure was generalized into an impossibility

The measured abort is valid negative evidence for the current representation.
It is not a lower bound for all circuits proving the theorem.

Required correction: preserve the measurement as a rejected-design benchmark,
then measure the selected bounded step circuit and a complete multi-step proof.

### E-005 — The acceptance test froze an implementation accident

`test_full_cap_shape_constraints` equates success with materializing the full
padded arena in one `ShapeCS`. This rewards the wrong resource behavior and does
not by itself prove replay, HJMT, or fold correctness.

Required correction: replace the acceptance meaning with:

- one bounded fixed-shape synthesis measurement;
- exact cap enforcement by a constrained cumulative byte counter;
- successful proof/verification for a maximum-*logical*-length deterministic
  trace without allocating the entire witness inside one step;
- mutation tests for bytes, framing, ordinals, replay, HJMT, finalization, and
  recursive chain continuity.

### E-006 — The native facade was mistaken for a reference transition evaluator

The live authority defines `CheckpointTransitionConsistencyV1` as re-executing
the accepted update and calculating every result-root binding. Current
`recursive_predicate.rs::evaluate` performs equality checks and delegates to
`CheckpointTransitionWitnessMaterialV1::validate`. That validator checks exact
replay-row reconstruction and valid HJMT proof envelopes at the prior root, but
does not apply the ordered spent/created operations or derive the resulting
state/settlement/check and HJMT bindings.

Consequences:

- differential tests against this evaluator can certify the same incomplete
  relation in native and R1CS code;
- the 15-row map names semantic categories but is not an executable constraint
  specification;
- constraining every field in the current map is necessary but insufficient.

Required correction: first make one backend-neutral V2 reference transition
trace that reuses the `build_cp_draft`/HJMT plan-store transition primitives,
derives the V2 delta/journal/root commitments, and reproduces every statement
result binding. The Nova relation and later Plonky3 AIR must be differentially
checked against that complete evaluator.

This does **not** pull `EndToEndCheckpointValidityV1` into Phase 069. The
transaction authorization/range/signature verifier remains outside the selected
predicate. Exact transaction-proof bytes and their canonical commitment remain
inputs because the accepted canonical transition already depended on them.

### E-007 — A proof-at-prior-root was treated as an update proof

Current `RecursiveHjmtPathMaterialV1::validate` checks canonical
`BatchProofBlobV1` envelopes and requires their settlement root to equal the
prior root. Membership/non-membership at the prior root proves access facts; it
does not automatically prove the ordered batch update or the resulting root.
Overlapping Merkle paths also cannot be independently updated and then composed
without an explicit batch/DAG rule.

Required correction: the witness trace must include a deterministic batch
update schedule. Each read is authenticated at the correct running root, each
write computes the next root, and shared-path/DAG references are either consumed
by an exact in-circuit batch algorithm or by authenticated-memory operations
whose final root is checked. All old/new leaves and operation ordinals must be
enumerated.

### E-008 — Circuit profile mixed theorem inputs with unrelated envelope caps

`RecursiveCircuitProfileV1` currently commits proof-object, sidecar, epoch,
archive, snapshot, audit, and documentation-packet byte caps into the circuit
profile. These are important live configuration and output-validation limits,
but they are not all dimensions of `CheckpointTransitionConsistencyV1`.

Required correction: bind the complete checkpoint-config digest in context and
the exact circuit-relevant limits in the circuit spec. Enforce output caps at
codec/runner boundaries. Do not allocate or add constraints for unrelated
documentation/archive envelope capacities merely because they share one YAML
`limits` table.

### E-009 — The live byte hash has a deterministic field-encoding alias

The live path is:

```text
arbitrary bytes
  -> `WordPacker` groups 8 bytes as one little-endian `u64`
  -> `Goldilocks::new(u64)`
  -> Poseidon2 permutation
```

The Goldilocks modulus is `p = 0xffff_ffff_0000_0001`. A field element is an
equivalence class modulo `p`; upstream `PartialEq`, arithmetic, and canonical
serialization use the canonical representative. Thus an aligned input word
`0x0000000000000000` and the little-endian bytes of `p` enter the sponge as the
same field element. Length and delimiter framing do not distinguish two
same-length streams differing only by that word.

This is an encoding collision, not a generic birthday-bound attack on
Poseidon2. It applies before the permutation. Large opaque transaction-proof
bytes provide attacker-controlled aligned words, so the exact-byte requirement
cannot assume the input language excludes the collision.

The current R1CS gadget does not even reproduce that legacy behavior for all
inputs: `GoldilocksVar::alloc` rejects a word `>= p`. Therefore the current
native and circuit relations disagree on a non-empty class of valid storage
byte strings.

Required correction before a sound exact-byte proof:

1. introduce one versioned injective byte encoding, e.g. 7-byte limbs (at most
   56 bits, strictly below `p`) with explicit total length/item framing;
2. bind its version/domain/parameters in the statement and backend manifest;
3. provide old/new golden vectors and a deterministic legacy-collision test;
4. migrate the live checkpoint statement/exec/witness/delta commitment era in
   one canonical path; do not silently change V1 digest semantics;
5. make native, Nova R1CS, and Plonky3 AIR use exactly the same word stream.

Using SHA-256 only inside Nova, adding an unbound audit digest, or accepting the
legacy `hash_zk` root and a different private digest would create an unbound
second path and does not close the issue.

### E-010 — The positive test is a false-positive oracle

`crates/z00z_storage/tests/test_recursive_predicate.rs` currently constructs:

- a real prior HJMT store and inclusion/non-existence proof blobs;
- an execution row;
- an arbitrary statement `new_root = [2; 32]`;
- arbitrary core `delta_root = [8; 32]`, `witness_root = [9; 32]`, and
  `journal_digest = [10; 32]`;
- an arbitrary final bind.

It does not apply the execution row to that store and does not derive those
result values from the fixture transition. Nevertheless
`test_native_predicate_accepts_storage_replay_and_hjmt_material` expects
`CheckpointTransitionConsistencyV1::evaluate` to accept.

This test is valuable negative evidence: it proves that current acceptance is
envelope self-consistency, not transition consistency. Reimplementing its
behavior in R1CS would produce a real Nova proof of the wrong relation.

Required correction: replace the positive corpus with a fixture produced by the
real canonical builder/store path. Add a negative case that preserves all
envelope digests while changing each independently derived result root; the
reference evaluator, Nova verifier, and later AIR verifier must all reject.

### E-011 — Retention material was confused with computational witness

The authority makes two separate requirements:

1. exact transaction-proof and HJMT/archive bytes remain retrievable and their
   content digests are bound;
2. `CheckpointTransitionConsistencyV1` re-executes the state update.

It explicitly describes the circuit witness in terms of transaction proof-byte
**digests**, while `EndToEndCheckpointValidityV1` and replacement of the current
transaction verifier are outside Phase 069. Therefore the transition circuit
does not need to parse or cryptographically re-verify up to 64 MiB of opaque
transaction proof bytes. It needs to constrain a collision-resistant
`tx_proof_digest` inside each ordered semantic replay row.

Likewise, the circuit must verify HJMT membership/non-membership/update
semantics from typed leaves, keys, sibling hashes, and root transitions. It does
not need to prove that every byte of the storage codec was parsed if those raw
bytes are separately content-addressed and the semantic witness is itself bound
by a versioned structural commitment.

Required correction:

- retain `max_witness_bytes = 64 MiB` as an input/archive/DoS cap enforced by
  storage before proving;
- remove `arena_bytes` from the R1CS shape;
- define a collision-safe exact-byte content digest outside the semantic row;
- make `tx_data_root` (or a versioned recursive translation commitment) combine
  ordinal, input refs, outputs, and `tx_proof_digest`, not raw proof bytes;
- define a typed HJMT transition trace and recompute old/new roots inside the
  circuit;
- bind archive/witness roots and content digests in the statement/recursive
  public input, while keeping retrievability checks outside proof validity.

This is not a prohibited “digest-only circuit”: the state-transition semantics
remain fully constrained. Only opaque data that Phase 069 deliberately does not
interpret is represented by a collision-resistant digest.

### E-012 — An input cap was misread as a mandatory local proving SLA

`max_witness_bytes = 64 MiB` is a hard bound that protects storage and prover
interfaces. The live failure model explicitly states that exhausted prover
queue, memory, witness, or time budget fails the **shadow job** with a typed
resource outcome and does not block canonical admission (`069-TODO.md:2976`).

Plan 051 instead made one successful 64 MiB monolithic proof on this host the
only acceptable correctness evidence. That contradicts the specified lifecycle
and forced an OOM-prone test to stand in for soundness.

Required correction:

- the circuit relation and trace codec support every input within the logical
  cap without changing circuit shape;
- the runner estimates steps/key/witness/time before setup/proving and returns a
  typed resource error before dangerous allocation;
- representative real proofs cover maximum semantic depth/branch classes and
  multi-step completion;
- property/differential tests cover all length/count boundaries;
- a 64 MiB generated package proves bounded streaming/preflight behavior and
  either completes under an explicitly provisioned profile or returns the exact
  typed resource outcome; SIGABRT/OOM/partial receipt is always failure;
- no resource rejection may be recorded as a cryptographic success or used to
  issue a receipt.

This does not permit a reduced-cap circuit. It separates mathematical
completeness of the relation from the non-authoritative worker's local liveness.

### E-013 — The new-root batch was labelled and validated as a prior-root proof

`build_stmt_core_v1` rejects unless the first `BatchProofBlobV1` authenticates
`draft.new_settlement_root()`. In contrast,
`CheckpointTransitionWitnessMaterialV1::from_storage` stores
`statement.prev_root()` in `prior_settlement_root`, and `validate` requires
every supplied batch to authenticate that value. A real builder-produced
post-state batch therefore cannot satisfy the current recursive witness
contract except in a no-op transition.

This is not a test-fixture inconvenience. It proves that the T1 witness was
designed from a synthetic inclusion/non-existence fixture rather than from the
canonical builder output.

Required correction: replace the ambiguous `hjmt_paths` vector with explicit,
typed roles:

- pre-state resolution/authentication material;
- ordered running-root deletion/insertion update trace;
- post-state batch/archive material used by `witness_root` and journal binding.

Each role has a distinct root and ordinal contract. Do not rename the current
single-root blob and continue.

### E-014 — Batch validity was substituted for transition validity

`BatchProofBlobV1::check_contract_v1` verifies table bounds, exact reference
usage, opening families, SHA-256/JMT root walks, transcript binding, and one
header root. It never accepts `CheckpointExecInput`, never applies its ordered
deletes/inserts, and never derives a second root. Calling it inside native
witness validation cannot establish the checkpoint transition.

Required correction: reuse its storage-owned typed primitives and exact hash
rules, but add an executable transition-trace builder/evaluator whose input is
the canonical snapshot/replay/exec path. Differential tests must compare its
final root and deltas with `build_cp_draft`, not merely with
`check_contract_v1`.

### E-015 — A fixture requirement became a production chain invariant

The current predicate and circuit input require `chain_length` to be in
`3..=5`. The authority requires 3-step and 5-step **evidence fixtures**; it does
not cap the live recursive chain at five or require a block producer to know a
future terminal chain length.

Required correction: remove fixture length from the transition theorem. Bind
height, predecessor accumulator, segment ordinal, and completed-block count.
Construct 3-step and 5-step evidence by verifying three/five consecutive
completed block snapshots.

### E-016 — The semantic operation domain is not actually bounded

`CheckpointTransitionWitnessMaterialV1` caps transaction rows at 64 and HJMT
blob wrappers at 128, but `CheckpointExecTx` has unbounded `input_refs` and
`outputs`. `BatchProofLimitsV1` allows up to 16,384 witness nodes per batch.
`limits.max_batch_ops = 1000` is copied into the profile but is not enforced by
the exec constructor, witness constructor, reference evaluator, or R1CS.

Therefore “fixed semantic shape” is not yet defined. Padding only 64 row
headers does not bound the work inside those rows.

Required correction: the storage contract must define and enforce exact live
semantic caps before allocation:

```text
tx_count <= max_replay_rows
sum(input_count + output_count) <= max_batch_ops
typed_update_count == sum(input_count + output_count)
typed_path_count and typed_witness_node_count <= their explicit live caps
encoded_retention_bytes <= max_witness_bytes
```

The circuit/trace spec commits these caps. The runner preflights them. A byte
cap remains an additional storage/DoS bound, not the circuit-shape definition.

### E-017 — “One step per block” was specified at two incompatible levels

The TODO uses “IVC step” both for the semantic checkpoint transition and for a
literal Nova fold call. If the literal call must cover the whole maximum
variable computation, fixed-shape R1CS must reserve the maximum semantic work;
if the shape is kept small, many fold calls are needed. Both cannot be true
without a nested proof system.

The least-complex solution keeps plain Nova and changes the unit naming:

- **micro-step/fold:** one invocation of the fixed Nova `StepCircuit`;
- **block segment:** `BEGIN_BLOCK`, all required micro-steps, then exactly one
  `FINALIZE_BLOCK`;
- **block snapshot:** the only persistable/receiptable state, emitted after
  finalization;
- **fold cadence one block:** every accepted block has exactly one completed
  segment/snapshot; it does not prescribe exactly one internal fold call.

This wording must become live authority before implementation. If literal one
fold call remains mandatory, the only honest alternative is a separately
specified inner proof verified by that fold, with its own parameters and
soundness analysis. Hiding such an inner proof behind the existing name is
forbidden.

### E-018 — The predicate identity hashes prose rather than executable semantics

`checkpoint_transition_consistency_digest_v1()` hashes fifteen strings such as
`"row input digest"` and `"hjmt_path_table"`. Those strings do not commit to
the operation grammar, hash domains, selectors, state transitions, or circuit
matrices. The current incomplete evaluator and a future complete evaluator can
share the same digest.

Required correction: the next predicate identity must bind canonical bytes of
the executable trace spec, field/public-input encoding, hash parameter set,
semantic caps, and compiled circuit/parameter manifest. A human-readable map
remains documentation only.

### E-019 — Compatibility constructors create a second context path

`RecursiveCheckpointPublicInputV1::new` and `from_statement` synthesize a
`legacy_context_digest`, while `new_with_context` and
`from_statement_with_context` accept the required explicit chain context. Both
are public construction paths. This violates the requested single canonical
owner and permits a caller to select a weaker era accidentally.

Required correction: at the version cutover, make explicit-context
construction the sole live constructor. Legacy decode remains read-only and
cannot enter the prover, verifier, or receipt path.

### E-020 — Recursive predecessor state is not the storage prior root

The statement core can carry `prior_recursive_output_root`, but the current
recursive public input maps `prior_output_root` to `statement.prev_root()`.
Those facts have different meanings: one is the previous Nova accumulator and
one is the previous storage state root. Conflating them makes recursive-chain
binding unverifiable or forces a false equality.

Required correction: the versioned public input carries both separately and
the circuit checks:

```text
storage_prev_root == statement.prev_root
recursive_prev_accumulator == prior completed Nova segment output
next_recursive_accumulator == SHA256_DomainV2(
  recursive_prev_accumulator, statement/trace/final-state bindings
)
```

Neither field may be inferred from the other.

### E-021 — The live sponge capacity is far below the apparent digest size

`poseidon2_hash` uses a width-eight Goldilocks permutation with rate seven and
serializes four output elements. Rate seven means capacity one field element,
approximately 64 bits. Generic sponge collision security is limited by the
birthday bound in capacity, not by concatenating more squeezed/output words.
The [Poseidon2 paper](https://eprint.iacr.org/2023/323) states the generic bound
in terms of `p^c/2` queries and requires capacity sufficient for the target
security. With `c = 1` and `p ≈ 2^64`, the generic collision bound is only about
`2^32` queries, not 128 bits.

Required correction: the primary recovery uses domain-separated SHA-256. Any
later Poseidon2 H2 optimization must version the sponge mode as well as byte packing.
The lowest-change width-eight profile for approximately 128-bit generic
collision security is:

```text
width = 8 Goldilocks elements
rate = 4
capacity = 4  (~256 capacity bits)
output = 4 canonical elements
padding = one field element 1 followed by zeroes to a full rate block
```

The pinned permutation/constants may be reused only after their round/security
profile is checked for the width-eight Goldilocks instantiation and bound in the
backend manifest. A Poseidon2 H2 must not call the V1 `rate = width - 1` helper internally.
Changing output length alone does not fix capacity.

### E-022 — Fixing `hash_zk` in place would be an unversioned consensus rewrite

A source inventory finds 163 `hash_zk` call tokens across 70 Rust files. The
alias/capacity defect therefore has a project-wide blast radius, but silently
changing `poseidon2_hash` would also change every persisted V1 ID/root/vector
and could make old data undecodable or unverifiable.

Required correction for Phase 069: create checkpoint-V2 SHA-256 commitment
domains and migrate the one recursive checkpoint theorem at an explicit cutover.
V1 stays read-only. Record the wider V1 hash risk for its owners, but do not mix
a global consensus migration into the private Nova circuit or retain both V1
and V2 as selectable live prover paths.

### E-023 — Library compatibility evidence was confused with theorem evidence

The only concrete Nova circuit in `adapter.rs` is `TrivialCircuit`. It proves a
single state-preserving synthetic program, compresses it, verifies it, and
issues a receipt whose only scope is `LibrarySmoke`. This is useful dependency
compatibility evidence, but it proves no statement field, replay row, HJMT
update, or checkpoint chain relation.

Required correction: the production adapter must be constructed only from the
storage-owned corrected circuit/trace input and may issue a checkpoint receipt
only after the compressed verifier returns the exact expected final state. The
smoke receipt remains test-only and can never be promoted or renamed.

### E-024 — Independent per-block Nova restarts would break IVC continuity

A digest of the previous compressed proof is not recursive verification. If a
new `RecursiveSNARK` starts at every block without verifying the previous proof
inside the step circuit, a malicious prover can assert an arbitrary predecessor
digest. This would produce a chain of unrelated proofs framed as recursion.

Required correction: use one continuous IVC state across all micro-steps and
blocks. Compress a non-consuming snapshot only when the running state reaches
`FINALIZE_BLOCK`. Verification binds the genesis `z0`, cumulative step count,
returned final state, height, and predecessor accumulator. Restart is allowed
only through a separately specified proof-carrying-state circuit that verifies
the previous proof; that larger composition is not the selected path.

### E-025 — The current receipt shape cannot represent checkpoint proof success

`ReceiptScopeV1` contains only `LibrarySmoke`; the receipt binds digests but not
the cumulative Nova step count or complete returned final state. A verifier can
therefore not use this type to distinguish the exact completed computation from
another length/end state, even though pinned Nova itself accepts those values.
Its exported `receipt_digest` also uses the defective V1 `hash_zk` family.

Required correction: add one versioned checkpoint receipt scope after the real
adapter exists. Its private constructor must bind backend/circuit/parameter
digests, compressed proof digest, genesis/initial-state digest, cumulative
`num_steps`, complete final-state digest, finalized height, statement/link
digests, and predecessor/output accumulator, and must use the checkpoint-V2
SHA-256 commitment family. Persisted receipt bytes never
restore authority; local proof verification must recreate the capability.

### E-026 — The pinned Nova first-step API invites an off-by-one proof

`RecursiveSNARK::new` already synthesizes `circuit_0`. Calling `prove_step` for
the first time only sets `i = 1`; subsequent calls synthesize later circuit
witnesses. A naïve loop that calls `new(circuit[0])` and then calls
`prove_step(circuit[0])` before iterating the remaining events can duplicate or
mislabel the first event while still obtaining a valid proof for a different
trace.

Required correction: define the exact runner schedule and assert after every
event that `recursive.num_steps() == consumed_event_count`. The constructor
consumes event zero; the count-accounting call advances it to one; only event
one and later are passed to the folding branch. Receipt step count comes from
the checked runner state, never from witness metadata.

### E-027 — Witness-dependent circuit construction is not fixed shape

A Rust `match opcode { ... }` that allocates only the selected branch changes
the R1CS matrices with witness values. `PublicParams::setup` sees only the setup
circuit's branch, so later opcodes would be unproved or fail shape agreement.

Required correction: directly compile one finite opcode grammar into one
`StepCircuit`. Every synthesis allocates the same boolean one-hot selectors,
SHA compression lane, semantic lane, and candidate next states in the same
order. Algebraic selection chooses the active candidate; inactive witness cells
are constrained to canonical zero/no-op values. There is no host branch, second
interpreter, or caller-supplied validity flag.

### E-028 — Opaque proof-byte binding was incorrectly generalized to state leaves

Phase 069 explicitly treats transaction-proof verification as out of scope, so
the checkpoint relation may bind a storage-recomputed digest of those exact
bytes. The same argument does not automatically apply to old/new leaf bytes:
their SHA-256 value hash is an input to the public HJMT root and their typed
path/fields drive replay. Supplying both a leaf digest and typed fields without
proving their relation permits equivocation.

Required correction: stream canonical leaf bytes through the exact SHA-256
compression relation and bind the result both to typed replay material and to
the HJMT path. A future fixed digest-leaf format requires a storage-wide
version/cutover; a recursive-only projection root is forbidden.

### E-029 — The existing R1CS SHA helper is one-shot, not a streaming state

`nova/hash.rs::constrain_sha256_bytes` calls the pinned whole-message `sha256`
gadget. Its constraint count scales with the input slice length, so calling it
on a maximum variable leaf or arena recreates the original category error.
The pinned frontend also exposes `sha256_compression_function`, which accepts
one 512-bit block and eight running `UInt32` words.

Required correction: the canonical checkpoint circuit owns a fixed SHA state
of eight range-checked 32-bit words, declared message length, block ordinal, and
padding phase. `SHA_BLOCK` consumes exactly 64 bytes. The final one/two blocks
must constrain the `1` bit, zero fill, and 64-bit big-endian bit length exactly.
HJMT must reproduce the live constants and spelling exactly:
`JMT::LeafNode`, `JMT::IntrnalNode`, and
`SPARSE_MERKLE_PLACEHOLDER_HASH__`. The one-shot helper remains only for small
fixed messages/tests and cannot receive the witness-cap-sized input.

### E-030 — One HJMT event was initially treated as one SHA block

The first low-memory sketch said that `HJMT_NODE` would consume a sibling and
update the running root while the fixed circuit had one SHA compression lane.
That hid two different units of work. The exact live internal-node preimage is:

```text
b"JMT::IntrnalNode" || left_child_32 || right_child_32
```

It is 80 bytes, so SHA-256 processes two padded 64-byte blocks. A state update
must additionally derive an old parent and a new parent from the same
level/direction/sibling: the old result authenticates continuity with the
current root, while the new result advances the root. Domain promotion and
leaf hashing have the same need for explicit, role-bound hash results.

Required correction: the selected `BEGIN_HJMT_PATH_NODE` binds the level,
direction and live HJMT domain,
sibling, old child, and new child; generic `BEGIN_HASH`/`SHA_BLOCK`/`END_HASH`
subtraces derive each required old/new leaf or parent hash using the exact live
bytes; only `COMMIT_HJMT_PATH_NODE` may compare the old result and install the new
result. Thus one Nova fold still performs at most one SHA compression, while
one semantic HJMT level intentionally spans multiple folds. No host-computed
parent digest is trusted.

### E-031 — Duplicate detection could hide a second path-sized circuit

Replacing `BTreeSet` with an authenticated spent/output set is sound only if
its Merkle work is constrained. Allocating all levels of a fixed-depth absence
and insertion path inside `REPLAY_OP` would preserve a finite shape but would
contradict the claimed `C = O(one compression)` architecture and duplicate the
HJMT hashing machinery.

Required correction: do not allocate a set path inside `REPLAY_OP`. The
selected route uses the linear sorted/permutation argument in E-038. If that
argument fails implementation/security review, the fallback uses the same
streamed authenticated-path machinery with explicit proof-local domains as
specified in E-037. Either route keeps path depth out of R1CS shape; neither is
an alternate canonical state engine.

### E-032 — `ordered_root` was an underspecified commitment primitive

The first V2 sketch used `ordered_root(count, item_commitments[])` without
defining whether it was a Merkle tree, a fold, or a framed list. That is not a
cryptographic specification and could accidentally introduce a second hashing
implementation beside `z00z_crypto::hash::sha256_256`.

Required correction: the storage checkpoint owner defines exactly one typed
`ordered_commit_v2` that calls the existing helper as:

```text
sha256_256(domain_v2, label_v2,
           [u32_le(item_count), item_digest_0, ..., item_digest_(n-1)])
```

The helper's existing preimage grammar remains byte-for-byte authoritative:

```text
u64_le(len(dst)) || dst
|| u64_le(len(part_0)) || part_0
|| ...

dst = b"z00z.hash.v1\0"
   || u64_le(len(domain_v2)) || domain_v2
   || u64_le(len(label_v2)) || label_v2
```

“V2” versions the checkpoint domain/label/schema, not the generic helper's
framing tag. The Nova trace must reproduce this exact stream through the
generic SHA lane. No circuit-local SHA domain builder or unspecified Merkle
root is allowed.

### E-033 — Memory feasibility was at risk of being restated as liveness

Constant per-fold memory does not make worst-case total work constant. One
64 MiB SHA-256 message alone has:

```text
ceil((67_108_864 + 1 + 8) / 64) = 1_048_577 compression blocks
```

and an HJMT update adds value, leaf, old-parent, and new-parent hashes. This is
not a multi-gigabyte memory requirement, but it can exceed a worker's time/job
budget.

Required correction: T1 freezes every semantic length/count using existing
canonical format/contract authority; it must not invent a smaller hidden cap.
Where the only authoritative bound is the full 64 MiB package cap, the trace
and counters remain capable of representing that value. T3/T4 preflight
estimates exact fold/hash counts and may return the existing typed shadow-job
resource outcome before proving. Such an outcome preserves canonical
admission and theorem soundness but is not a successful proof or completion
evidence for that witness.

### E-034 — Leaf hashing alone does not bind typed replay fields

`encode_terminal_leaf` emits the terminal-family tag followed by
`BincodeCodec::serialize(TerminalLeaf)`, and `BincodeCodec` currently uses
`bincode::config::standard()`. If the circuit merely hashes a prover-supplied
byte stream while replay uses separately supplied `asset_id`, `serial_id`,
`owner_tag`, ciphertext, proof, and other fields, the prover may open the HJMT
root with one leaf and execute semantics with another.

Required correction: the canonical trace emits field-bound serialization
events. Fixed fields are inserted at exact offsets/encodings; vector lengths
use the exact pinned bincode-v2 standard varint grammar; the streamed
ciphertext/range-proof bytes are consumed at those declared lengths; no
trailing byte is allowed. The same constrained byte stream feeds the JMT value
hash. T1 must freeze codec/version/golden vectors and fail closed on dependency
or schema drift. Replacing this with a digest-normalized leaf requires the
explicit storage-wide state cutover already identified in Section 2.3.

### E-035 — Semantic settlement root and HJMT backend root were conflated

`commit_hjmt_plan_at` installs the semantic `plan.root` as
`SettlementStateRoot`. In contrast, `proof_batch_verify::check_atomic_roots`
walks the SHA-JMT and compares the result with `BatchProofHeaderV1.backend_root`.
The batch header stores both values and links them with `root_bind_v1`; it also
links version/journal data through `checkpoint_bind_v1`. Both bind functions
use the defective V1 `hash_zk` family.

Required correction: the V2 cutover `z0` binds the historical semantic
settlement root and canonical JMT backend root, then derives the sole live
`SettlementStateRootV2` described in E-036. Every post-cutover block derives
the next backend root through the ordered JMT update trace and derives the next
V2 settlement/check root from it. Existing V1 batch
`root_bind`/`checkpoint_bind` may be decoded and checked for historical
compatibility, but cannot anchor the V2 proof and cannot be silently rewritten.
The existing `SettlementStateRoot` owner gains
`RootGeneration::SettlementV2`; the existing `CheckRoot::from(root)` remains
the sole check-root wrapper. The V2 statement/context binds the generation.
No parallel check-root type or raw-backend-root alias is introduced.

### E-036 — Preserving the V1 semantic root would reintroduce a second large computation

`SettlementModel::root` is not the SHA-JMT root. It recursively calls the live
V1 `poseidon2_hash` over all terminal leaves in a serial, all serial leaves in a
definition, and all definition leaves in the model. Therefore a proof that
keeps this root authoritative must receive enough unchanged model data to
recompute affected/full ordered lists, stream the aliased V1 byte packing, and
add a fixed Poseidon2 permutation lane beside SHA. That route is constant-memory
in principle, but can approach whole-state work and retains the approximately
32-bit-capacity commitment as a public security boundary.

Selected broader correction: add one explicit storage root generation:

```text
settlement_root_digest_v2 = sha256_256(
    domain = "z00z.storage.settlement.root",
    label  = "settlement_hjmt_root_v2",
    parts  = [
        root_generation_v2,
        hjmt_layout_version,
        bucket_policy_digest,
        canonical_definition_tree_backend_root
    ]
)
SettlementStateRoot::new(RootGeneration::SettlementV2,
                         settlement_root_digest_v2)
CheckRoot::from(settlement_state_root_v2)
```

The definition-tree backend root transitively commits the canonical
definition/serial/bucket/terminal HJMT hierarchy; the path-index tree remains
non-consensus lookup metadata as its owner documents. T1 must prove this
transitivity with exact builder/store vectors and must include any additional
consensus HJMT root if source inspection shows it is not committed by the
definition root. The V2 builder predicts the next backend root through the
existing transactional HJMT plan/store primitive, derives the V2 root, and
later requires the committed store result to match; it does not maintain a
parallel model/root engine.

At one configured cutover, the migration record binds the last V1 settlement
root, its exact current HJMT definition root/policy/version, and the first V2
root. After cutover, only V2 is constructible/receiptable; V1 roots remain
historical read/audit data. This is a storage-wide root-generation migration,
not a Phase-069 projection root or selectable dual path. It removes the need
to prove `SettlementModel::root` and removes V1 Poseidon2 from the repaired
checkpoint circuit entirely.

### E-037 — Authenticated-set fallback still needs tree-role domain separation

If the fallback proof-local spent/output sets are used, they are not part of
canonical HJMT state. Reusing
raw `JMT::LeafNode`/`JMT::IntrnalNode` hashes for them would make identical
key/value/path material portable between semantic state and uniqueness memory.
Sharing an opcode/constraint implementation does not justify sharing a hash
domain.

Required correction: the constrained `tree_role` selects exact preimages for
`STATE_HJMT`, `UNIQUE_SPENT_V2`, or `UNIQUE_OUTPUT_V2`. The two uniqueness
families use the existing `sha256_256` helper with distinct V2 labels for leaf,
internal node, and empty root, and include the set-kind tag, 256-bit ID, present
bit, level, and ordered children as applicable. Their depth-256 empty roots are
fixed golden-vector constants. A role change, set-kind change, or imported host
root rejects. Only the compression/phase machinery is reused; commitment
domains never overlap.

### E-038 — A sound authenticated set was still unnecessarily expensive

A 256-bit sparse set would require an absence/update path for every spent and
output ID. Even when streamed, this can dominate the fold count. The input IDs
are already committed by the V2 transaction-data root before the duplicate
argument is evaluated, so a standard two-pass permutation/ordering argument is
the smaller fixed-state construction.

Selected construction, separately for spent IDs and output IDs:

1. before replay, derive two independent `(alpha, beta)` challenge pairs with
   constrained `sha256_256` subtraces from context, statement/tx-data root,
   parameter digest, set kind, and challenge ordinal; map each as
   `2 + u248_le(digest[0..31])`, which is injective and nonzero below the Pallas
   scalar modulus, and freeze golden vectors;
2. encode each 256-bit ID as all sixteen existing little-endian `u16` limbs and
   compute `id_beta = sum(limb_j * beta^j)` in-circuit;
3. while replay consumes the canonical IDs, update both original-list grand
   products `P_k *= alpha_k - id_beta_k` and exact counts;
4. in a later `CHECK_SORTED_ID` phase, consume the same number of IDs in strict
   lexicographic byte order, update matching sorted-list products, and require
   every ID to be greater than its predecessor;
5. finalization requires original/sorted counts and both product pairs to be
   equal.

The statement/tx root fixes the original list before the Fiat–Shamir
challenges; strict ordering forbids duplicates in the sorted list; product
equality proves multiset equality with field/ROM soundness rather than a trusted
host sort. Two independently domain-separated challenge pairs reduce the risk
of a single compression/product collision. Zero factors are valid and do not
weaken the second independent product; mutation tests must cover them. If this
argument cannot meet the declared soundness target under the exact maximum
count, T1 must select the E-037 authenticated-set fallback explicitly—never a
native `BTreeSet` boolean.

### E-039 — V2 root generation requires a V2 HJMT witness envelope

`check_batch_header` currently requires
`BATCH_PROOF_ENCODING_VERSION == 1`, `RootGeneration1`, root-bind version 1,
and the V1 `hash_zk` root/checkpoint bind functions. Therefore a live V2
settlement root cannot be inserted into `BatchProofBlobV1` or accepted by its
verifier without an unversioned semantic rewrite.

Required correction: the existing settlement proof-batch owner gains one
explicit V2 envelope/header/codec/verifier that carries the V2 root generation,
canonical definition-tree backend root, HJMT layout/policy identity, ordered
update roles, and SHA-256-V2 root/checkpoint/content binds. It is the source
from which `RecursiveTransitionTraceV2` is built; it does not duplicate the JMT
walk or state update. After cutover, builders issue only V2 envelopes and the
recursive path accepts only V2. V1 batch/proof objects remain strict historical
decode/re-encode/audit inputs and cannot reach a live constructor, prover, or
receipt. Cross-version substitution and a V1 envelope carrying a V2 root must
reject before backend proof verification.

### E-040 — A cutover manifest is not itself a state-equivalence proof

The last V1 semantic root and current HJMT definition root are computed by
different algorithms. Merely hashing both into a migration record or initial
Nova state permits an operator to assert an unrelated HJMT state. Because the
selected V2 IVC explicitly makes no claim about pre-cutover transition history,
the bridge must be established by the storage upgrade, not implied by Nova.

Required correction: define one authority-pinned
`SettlementRootGenerationCutoverV2` manifest containing network/context,
cutover height, source generation/root, destination generation/layout/policy,
expected definition-tree root, expected derived V2 settlement root, canonical
snapshot/archive identity, and manifest digest. At activation, storage rebuilds
or deterministically replays the canonical source snapshot through the existing
HJMT planner/store, compares every expected root/identity, atomically installs
generation V2, and records completion. Restart repeats/validates the same
result; partial, stale, wrong-network, changed-policy, changed-snapshot, or
root-mismatch migrations fail closed. The configured expected V2 root must be
network-authoritative, not operator supplied at runtime.

This one-time migration may perform whole-state work, but it can stream through
the storage backend and is not part of every Nova matrix or witness. If the
project requires a trustless proof of V1-to-V2 equivalence instead of a
deterministic protocol upgrade, Candidate I must be extended with an explicit
state-translation proof; that is a separate authority decision and cannot be
pretended by a digest equality.

## 5. Candidate architectures

| Candidate | Same theorem | Plain Nova | Peak memory independent of 64 MiB | Main risk | Initial disposition |
| --- | --- | --- | --- | --- | --- |
| A. Uniform streaming of every raw archive byte | Yes, if finalization binds the complete transcript and result | Yes | Yes; depends on fixed chunk/path step | Proves retention preimages that are not transition semantics; excessive folds | REJECT AS PRIMARY |
| B. Sparse allocation in one monolithic circuit | Yes | Yes | No | Still linear key/witness memory | REJECT |
| C. Native replay + digest-only circuit | No | Yes | Yes | Unconstrained semantics | REJECT |
| D. Reduced-cap profile | Changes accepted input set | Yes | Yes | Violates mandatory 64 MiB live cap | REJECT |
| E. SuperNova non-uniform steps | Potentially | No; backend change | Yes | Phase authority marks future comparison | HOLD/REJECT FOR 069 |
| F. STARK/zkVM base proof wrapped by Nova | Potentially | Hybrid | Potentially | New theorem/parameter/backend and verifier-in-circuit cost | HOLD |
| G. Monolithic circuit with a streaming `ShapeCS` implementation | Yes | Yes | No for proving/key generation | Treats symptom only | REJECT |
| H. Uniform authenticated-RAM micro-VM inside Nova | Yes, if its program and memory roots are statement/spec-bound | Yes | Yes | More folds and a larger state-machine audit | USE ONLY FOR DUPLICATE-SET MEMORY IF NEEDED |
| I. Inner streaming proof verified by one outer block step | Yes in principle | Requires recursion composition | Yes | Curve/backend composition and duplicated proof layer | FALLBACK |
| J. Typed semantic IVC + collision-safe content digests | Yes | Yes | Yes | Requires corrected reference evaluator and a versioned commitment era | SELECTED |
| K. Linear typed event tape + streaming domain-separated SHA-256 | Yes | Yes | Yes | More folds/time; must constrain phase/termination and duplicate sets | SELECTED CONCRETE FORM |
| L. Keep V1 semantic-model root and also prove JMT backend root | Same V1 output bytes | Yes, with an added streaming Poseidon lane | Yes, but may require whole-model data/work | Retains weak V1 root and two live root calculations | REJECT |
| M. One V2 settlement-root generation derived from canonical HJMT definition root | Same transition-consistency property after explicit cutover | Yes | Yes | Storage-wide root-generation migration and transitivity vectors | SELECTED ROOT FORM |

## 6. Primary solution hypothesis

Use the existing single storage-owned circuit/profile/spec/input authority, but
replace its padded raw-byte arena with one canonical linear typed event tape
consumed by a uniform Nova state-machine circuit. Opaque exact bytes remain in
the storage/archive owner and enter the semantic tape through versioned,
domain-separated SHA-256 content digests. The circuit fully constrains replay,
delta, spent/nullifier, HJMT, root, ordering, and final statement semantics; it
does not trust a native validity boolean.

External V2 checkpoint commitments use the already available domain-separated
SHA-256 helper contract, with new domains/labels and golden vectors. This avoids
both the V1 field alias and its insufficient sponge capacity. A corrected
capacity-four Poseidon2 profile remains a measurable future optimization, not a
precondition for solving T1–T4.

The same cutover introduces the sole live V2 generation of the existing
`SettlementStateRoot` owner, derived by
the exact SHA-256 helper from the canonical HJMT definition-tree root plus
layout/policy/generation. This is necessary because V1 `SettlementModel::root`
is a separate variable-length Poseidon2 computation with the same defective
hash profile. Keeping it would require an additional whole-model proof lane.
V1 root bytes remain historical; there is no simultaneous V1/V2 live selector.

### 6.1 Running state

The Nova state `z_i` must contain field-encoded commitments and counters, not
raw host pointers or trusted native verdicts:

```text
context/predicate/profile/spec/parameter digests
statement/public-input/checkpoint-link digests
phase + step ordinal + done flag
declared and consumed semantic row/operation/path/node counts
typed-trace and retained-content-digest accumulators
replay accumulator/current state root
current derived V2 settlement/check root, underlying SHA-JMT backend root, and journal accumulator
spent/output original-and-sorted grand products, counts, and previous sorted ID
expected final roots/digests
error/validity accumulator constrained to remain valid
```

The exact limb encoding must reuse the Phase 069 field contract. Digests that
are public checkpoint facts must be loaded from the canonical public input and
constrained, not copied from an untrusted step witness.

SHA's private chaining state is eight separately range-checked `u32` words.
At a hash boundary the circuit serializes those words in SHA-256 big-endian byte
order, then maps the resulting 32 bytes to the public-input contract's sixteen
little-endian `u16` limbs. Both byte orders and every range check are
constrained. It must not cast a 256-bit digest into one Pallas scalar or reuse
the defective Goldilocks eight-byte packing.

### 6.2 One directly compiled fixed-shape step

One fixed circuit uses a constrained phase/opcode selector and permits exactly
one event class at a time:

1. `INIT_BLOCK`: bind versioned identities, declared lengths/counts, initial
   roots, predecessor state, and expected final statement values;
2. `BEGIN_HASH`: bind object kind/domain, exact unpadded byte length, semantic
   ordinal, expected padded block count, and initialize SHA state for an object
   hashed in-circuit;
3. `SHA_BLOCK`: consume exactly one 64-byte block, constrain message bytes and
   the unique SHA-256 `0x80 || 0* || bit_length_be_u64` padding at the declared
   boundary, update eight SHA-256 words, and advance exact block/byte counters;
4. `END_HASH`: require the exact padded block count, expose the constrained
   digest, and bind it to the pending replay/HJMT/commitment role;
5. `REPLAY_OP`: constrain one input/output/spent/delta semantic operation;
6. `BEGIN_HJMT_PATH_NODE`: bind one ordered canonical path/update node, key
   bit, sibling side, live domain, level, and the old/new child values without
   changing the root;
7. `COMMIT_HJMT_PATH_NODE`: require the role-bound old/new hash subtraces to be
   complete, authenticate the old result, install the new result, and advance
   exactly one path level in the canonical HJMT trace;
8. `CHECK_SORTED_ID`: constrain one spent/output ID in the post-replay sorted
   phase, enforce strict canonical-byte order, and update both sorted-list
   grand products/counts;
9. `TRACE_COMMIT`: bind one completed typed semantic-event commitment and
   schedule its role-bound ordered accumulator hash;
10. `COMMIT_TRACE`: require that accumulator hash to be complete and install its
   constrained result;
11. `FINALIZE_BLOCK`: require exact counts/lengths, matching original/sorted
   uniqueness products, no pending hash/path/object,
   expected final accumulators/roots, and set the completed block state once;
12. `NOOP_AFTER_DONE`: only if required by snapshot/recovery code, with complete
   state equality and no route to `done` other than `FINALIZE_BLOCK`.

This is not a host-dispatched interpreter. The finite grammar is compiled
directly into the sole `nova/checkpoint.rs` `StepCircuit`. Every call allocates,
in the same order:

- boolean one-hot opcode selectors and legal phase-transition checks;
- one fixed SHA-256 compression candidate lane;
- one fixed semantic/HJMT metadata-and-commit candidate lane;
- candidate counter/root/accumulator states;
- algebraic selection of exactly one next state.

All branch constraints exist on every step; Rust must not `match` the witness
opcode to decide which constraints are synthesized. Inactive cells are
zero/no-op constrained, inactive candidate outputs cannot influence `z_(i+1)`,
and the SHA lane is paid as fixed per-step shape even on a non-SHA opcode. A
semantic node never contains an unaccounted whole-message hash: it opens a
pending role, the generic SHA lane consumes all required blocks over subsequent
folds, and a commit opcode installs the result. This is the cost of retaining
plain Nova while preventing shape drift. It is still one private circuit, one
trace grammar, and one theorem—no alias, shim, parallel root, or second state
engine.

### 6.3 Why this can remove the gigabyte requirement

The public parameters and commitment key are sized for one bounded micro-step,
not for the maximum total witness. Proving a larger witness increases the
number of folds and total time approximately linearly, while peak circuit/key
memory remains bounded by the per-step shape plus the Nova running proof state.
The release experiment in Section 9 measured a conservative whole-message
64-byte SHA step (two SHA compression blocks after padding) at 55,205 primary
constraints and 261,984 KiB runtime peak RSS through compressed verification.
The production lane still requires its own measurement after selectors and
semantic state are added; the experiment validates the asymptotic repair, not
the final budget.

### 6.4 Linear uniqueness and exact-usage checks

The maximum retained package remains an ordinary exact-length storage/archive
object capped at 64 MiB. It is never padded to 64 MiB or copied into Nova state.
Storage hashes each opaque transaction-proof object once with the V2
domain-separated SHA-256 content helper and the reference evaluator produces
typed operation rows carrying those content digests. Canonical old/new leaf
bytes that participate in live HJMT value hashes are the narrower exception:
their actual encoded length is streamed through `SHA_BLOCK` events so replay
fields and the public HJMT root cannot equivocate. The circuit cost follows
actual bytes, not the configured maximum.

The primary trace is linear and expands each semantic HJMT path in consumption
order. It therefore does not reproduce the raw batch codec's DAG/reference
tables inside R1CS. Global spent/output duplicate detection uses the E-038
two-pass sorted/permutation argument. `REPLAY_OP` accumulates the original ID
multisets under two statement-derived challenge pairs. After replay/path work,
`CHECK_SORTED_ID` consumes exactly the declared count for each set, reconstructs
all 32 ID bytes from the sixteen range-checked little-endian `u16` limbs,
enforces strict lexicographic order from byte 0 through byte 31, and accumulates
the comparison products. Finalization requires counts and both product pairs
to match. No host sort verdict or proof-local root is accepted.

This keeps duplicate work linear in ID count. The domain-separated
authenticated-set design in E-037 remains a documented fallback only if the
permutation argument fails the exact soundness review; it cannot coexist as a
runtime-selectable route. Sequential typed rows, expanded HJMT path nodes,
sorted IDs, and SHA blocks need only phase/ordinal/count constraints and
running hash/root/product state; they do not need random-access memory. A
separate storage test
recomputes each exact-byte
content digest from retained bytes and rejects missing, changed, or noncanonical
content. That storage test cannot substitute for Nova semantic verification,
and Nova cannot substitute for the storage availability check.

### 6.5 Required block-cadence correction

The scalable path produces many internal folds while consuming one block, then
emits exactly one statement-bound completed snapshot/receipt after `FINALIZE`.
The continuous IVC state advances to the next checkpoint only from that final
state. No intermediate fold is a block proof or persistable completed receipt.

The phrases “one IVC step per block” and `fold_cadence_blocks: 1` are not safe
to leave ambiguous because `069-TODO.md` also says the adapter “MUST fold one
step per block.” The corrected live contract must mean one **completed
statement-bound IVC segment/snapshot per block**, with one or more uniform
internal folds. This preserves the security property the cadence was intended
to express: no accepted block is skipped and no partial block is receipted.

The segment transition is constrained as:

```text
Idle(height = h - 1, accumulator = A_(h-1))
  --BEGIN_BLOCK(statement_h, trace_root_h, exact counts) -->
Active(height = h, phase = ..., accumulator = A_(h-1))
  --zero or more non-final micro-steps with strictly increasing ordinals -->
Active(height = h, all exact counts consumed)
  --FINALIZE_BLOCK(all result/binding equalities) -->
Idle(height = h, accumulator = A_h)
```

Only the final `Idle(height = h)` state may be compressed/persisted as the block
snapshot. A crash in `Active` is resumable private work, never a proof receipt.

The running `RecursiveSNARK` is continuous from the selected genesis/cutover
`z0`; it is not restarted per block. A compressed proof at height `h` verifies
the cumulative `num_steps` from that `z0` and returns the complete current
state. `CompressedSNARK::prove` snapshots the running object without consuming
it, so later micro-steps continue from the same IVC state. The block receipt
must record and recheck the cumulative step count and returned final-state
digest. A separately serialized `RecursiveSNARK` is private prover recovery
state containing witnesses; it must be encrypted/access-controlled, bounded,
atomically written, and never confused with a public zero-knowledge proof.

Pinned Nova has a non-obvious first-step schedule: `RecursiveSNARK::new` already
synthesizes event zero and the first `prove_step` call only makes the internal
step count one. The canonical runner must assert `num_steps == consumed_events`
after every advancement and must never replay event zero in the subsequent
loop.

If authority insists on exactly one library fold call, Candidate I (inner
streaming proof verified by an outer block circuit) is required and must be
planned as an explicit composition, not hidden behind naming. It has a larger
trusted/parameter surface and is rejected as the primary repair.

## 6.6 Primary research cross-check

- The [Nova paper](https://eprint.iacr.org/2021/370) defines IVC for repeated
  application of one function `F`; all steps share the same R1CS matrices, and
  prover work per incremental step scales with the size of `F`, not the number
  of earlier steps. This supports moving total work into many bounded uniform
  steps; it does not support one giant maximum-input step.
- The pinned [Microsoft Nova implementation](https://github.com/microsoft/Nova)
  describes IVC as incrementally proving long-running sequential computation.
  Its local `StepCircuit`/`prove_step` API independently confirms fixed shape
  with new per-step witness values.
- [SuperNova](https://eprint.iacr.org/2022/1758) formalizes non-uniform IVC for
  multiple instruction circuits. It would avoid charging one universal circuit
  shape for heterogeneous high-level steps, but Phase 069 currently excludes it
  as the active backend.
- [Nebula](https://eprint.iacr.org/2024/1605) provides a primary research path
  for efficient read-write memory in non-uniform IVC. It supports the general
  authenticated-memory direction but is not a drop-in pinned dependency or
  permission to change Phase 069's backend.

### 6.7 Commitment migration decision

Four ways to handle the legacy V1 input-encoding collision were examined:

| Route | Sound exact-byte binding | Same active theorem | Canonical-path cost | Decision |
| --- | --- | --- | --- | --- |
| Silently change V1 `hash_zk` packing | Yes after change | No stable V1 semantics | Breaks every existing V1 vector without a version boundary | REJECT |
| Keep V1 and trust native byte/digest checks | No self-contained proof; legacy collision remains | Superficially | Small | REJECT |
| Keep V1 statement and add a recursive-public-input translation commitment | Yes only if both legacy and new commitments are proved over the same bytes | Yes | Requires an extra streaming translation relation/proof | FALLBACK |
| Create one live versioned checkpoint commitment/statement era | Yes | Yes; same semantic theorem, new encoding version | Explicit codec/vector/migration work | SELECTED |

The selected clean path is a versioned checkpoint commitment era, not a second
simultaneously live theorem:

```text
settlement_root_v2 = SHA256_DomainV2(
    root_generation_v2, hjmt_layout_version,
    bucket_policy_digest, canonical_definition_tree_backend_root
)
tx_proof_content_digest_v2 = SHA256_DomainV2(exact_tx_proof_bytes, exact_length)
tx_row_commitment_v2 = SHA256_DomainV2(
    version, ordinal, canonical input refs, canonical outputs,
    tx_proof_content_digest_v2, exact_length
)
tx_data_root_v2 = ordered_commit_v2(
    domain = checkpoint_exec_v2,
    label = tx_root_v2,
    parts = [u32_le(row_count), tx_row_commitment_v2[]]
)

hjmt_semantic_commitment_v2 = SHA256_DomainV2(
    family, ordinal, prior root, key/path, old leaf, new leaf,
    sibling/root-update trace, canonical proof content digest and length
)
witness_root_v2 = ordered_commit_v2(
    domain = checkpoint_witness_v2,
    label = witness_root_v2,
    parts = [u32_le(count), hjmt_semantic_commitment_v2[]]
)
```

`canonical outputs`, `old leaf`, and `new leaf` above are not unconstrained
digest placeholders. The circuit reconstructs their semantic framing and, for
variable canonical leaf payload bytes needed by the existing JMT value hash,
consumes the actual-length SHA block trace. Only the separately out-of-scope
transaction-proof payload is represented by its statement-bound content digest.

Cutover rules are part of soundness, not deployment prose:

1. one configured cutover height/context selects V2; after it, production
   constructors/prover/verifier accept only V2 checkpoint commitment/public
   input bytes;
2. V1 decode remains historical read/audit support only and cannot issue a new
   checkpoint proof receipt; this includes V1 statement, execution, recursive,
   and HJMT batch-proof envelopes;
3. the V2 IVC `z0` binds cutover height, the last V1 settlement root, the
   canonical HJMT definition root/layout/policy, the derived first V2 root,
   chain context, circuit/parameter digests, and an explicit statement
   that no proof of pre-cutover transition history is claimed;
4. the existing canonical HJMT hierarchy and leaf encodings continue, but the
   public settlement/check root advances once to generation V2 derived from
   the definition-tree root, layout, and policy; this replaces the live V1
   semantic-model root rather than adding a duplicate tree or projection root;
5. no environment flag, backend label, compatibility constructor, or receipt
   field may choose V1 after cutover.

Before item 3 is accepted, the authority-pinned cutover manifest and
deterministic storage rebuild in E-040 must establish the exact first V2 root;
Nova does not turn an operator-provided root pair into migration evidence.

The executable semantic predicate identity therefore becomes the explicit V2
encoding of the same transition-consistency property. Keeping the name/type
suffix `V1` while changing its hash, trace grammar, fields, and matrices would
be an unversioned rewrite and is forbidden. `069-TODO.md`, Plan 051, matrix,
profile, public-input/envelope mappings, and test names must be normalized to
that cutover before implementation can claim T1 completion.

Hash-family comparison for that era:

| Family | Encoding alias | Generic collision target | Circuit cost | Existing project fit | Decision |
| --- | --- | --- | --- | --- | --- |
| V1 Poseidon2 rate 7 / 8-byte words | Deterministic alias | About 32-bit capacity bound | Low | Existing, but unsound for this role | REJECT |
| Poseidon2 H2 rate 4 / 7-byte words | Removed by injective packing | About 128-bit capacity bound | Low | Requires new mode, vectors, and parameter audit | HOLD AS OPTIMIZATION |
| Domain-separated SHA-256 V2 | None in byte input | 128-bit classical collision target | Higher | Already used by HJMT and has native/R1CS primitives | SELECTED |
| BLAKE2b-256 V2 | None in byte input | 128-bit classical collision target | Higher/new gadget burden | Native helper exists, no selected HJMT/R1CS reuse | REJECT FOR T1–T4 |

The selected SHA-256 cost changes total folds/time, not maximum arena/key
memory, because message compression is part of the uniform event tape.

`SHA256_DomainV2` and `ordered_commit_v2` use the existing
`z00z_crypto::hash::sha256_256` mechanics byte-for-byte: the helper's current
`z00z.hash.v1\0` DST framing tag remains unchanged, while new checkpoint-V2
domains/labels and schemas supply the version boundary. Exact cross-backend
golden vectors freeze the full preimage. `SettlementStateRootV2`,
`delta_root_v2`, `journal_digest_v2`, execution-input ID, statement digest, and checkpoint-link
bindings use the same versioned family where they commit arbitrary bytes. V1 remains read-only
compatibility data and cannot be used by the live recursive prover/verifier
after migration. There is one live constructor, statement builder, trace
builder, prover, and verifier path.

The canonical hash ledger is therefore explicit:

| Role | Exact owner/primitive | Circuit rule |
| --- | --- | --- |
| Existing JMT value hash | `Sha256(canonical_leaf_bytes)` | Stream field-bound canonical serialization and exact padding. |
| Existing JMT leaf hash | `Sha256(b"JMT::LeafNode" || key_32 || value_hash_32)` | Exact 77-byte preimage, two compression blocks. |
| Existing JMT internal hash | `Sha256(b"JMT::IntrnalNode" || left_32 || right_32)` | Exact 80-byte preimage, two compression blocks. |
| Existing JMT empty node | literal `b"SPARSE_MERKLE_PLACEHOLDER_HASH__"` | Constrain all 32 bytes; do not hash or reinterpret it. |
| Fallback-only proof-local spent/output set leaf/node/empty roots | existing `sha256_256` with distinct `unique_spent_v2` / `unique_output_v2` role labels | Include tree-kind/level/key/present/ordered-child framing; share only generic compression/path constraints, never state-JMT domains; absent from the selected live profile. |
| Checkpoint V2 settlement/check root, content, row, ordered, delta, witness, journal, statement, link, and receipt commitments | existing `z00z_crypto::hash::sha256_256` plus one storage-owned typed schema/domain/label per role | Reproduce the helper's exact length-prefixed stream through the one generic SHA lane. |
| Historical V1 commitments/batch binds | existing `hash_zk` bytes | Decode/audit only; never establish a V2 proof fact or select a live prover route. |

If changing the canonical statement era is explicitly refused, the fallback is
not a raw V1 circuit. It is a versioned recursive public input that binds both
V1 and SHA-256 V2 commitments plus a real streaming translation proof over the same
bytes. That is more complex and therefore not the preferred recovery.

### 6.8 Poseidon2 H2 alternative (not selected for recovery)

If later measurement shows SHA-256 fold cost is unacceptable, a Poseidon2 H2
optimization is possible. It is not the primary recovery because it needs a new
sponge-security review. It requires both an injective byte map and a new sponge
mode; reusing V1's rate-seven sponge would keep only one field element of
capacity and would not provide the intended collision security.

One concrete auditable H2 grammar is:

```text
transcript =
    u8(hash_version = 2)
  || u32_le(domain_len) || domain
  || u64_le(item_count)
  || for each item: u64_le(item_len) || item

field_words =
    [u64(total_transcript_len)]
  || [u56_le(chunk) for each consecutive 7-byte transcript chunk,
      final chunk zero-padded]
  || [u64(final_chunk_len)]
  || [u64(delimiter = 1)]
  || [zero words until the total word count is a multiple of rate 4]
```

Every packed data word is at most `2^56 - 1`, strictly below
`p = 2^64 - 2^32 + 1`. The explicit transcript length and final-chunk length
make chunk zero padding injective. Item count and item lengths make the
transcript prefix-free. The delimiter plus zeroes is unambiguous field-level
padding. Domain/version values prevent V1/V2 and cross-purpose reuse. H2 then
applies a versioned width-eight, rate-four, capacity-four Poseidon2 sponge and
returns four canonical words. Each full rate block is absorbed and permuted
exactly once; no V1-style extra final permutation is implicit.

Required tests before any Poseidon2 H2 optimization is live:

- exhaustive round trips for all final chunk lengths `0..=6`;
- boundary words `0`, `2^56 - 1`, `p - 1`, and the legacy `p` byte vector;
- domain/item split/order/length mutation rejects or changes the word stream;
- native, Nova, and Plonky3 word-stream/permutation golden vectors agree;
- the implementation and manifest assert `rate = 4`, `capacity = 4`, the exact
  padding rule, round constants, linear layer, S-box, and output rule;
- the legacy `0` versus little-endian `p` pair collides in V1 and does **not**
  collide in H2;
- no V1 constructor can install an H2 digest and no H2 constructor can accept a
  V1 digest without an explicit migration object.

Security note: injective packing removes the deterministic encoding collision,
while capacity four raises the generic sponge collision ceiling to roughly 128
bits. Collision resistance still relies on the selected Poseidon2 permutation
and round parameters; the backend manifest must bind those parameters, capacity,
rate, padding, and framing version.

### 6.9 Sole canonical module/API path after correction

No alias, shim, or second theorem is needed. The versioned owners should be:

| Concern | Sole owner | Required live API role |
| --- | --- | --- |
| V2 checkpoint content/commitment hash | existing `z00z_crypto::hash::sha256_256` owner plus checkpoint-V2 domains | Length-prefixed SHA-256 helper use and golden vectors; no local reimplementation. |
| Statement/exec/delta commitments | existing `z00z_storage::checkpoint` modules | V2 constructors only for live recursive proof production; V1 decode/read remains compatibility-only. |
| HJMT witness envelope and ordered update material | existing `z00z_storage::settlement` proof-batch owner | One V2 header/codec/verifier for the V2 root generation and SHA binds; V1 envelope decode/audit only; no duplicate JMT walker. |
| Semantic transition trace | one replacement of `checkpoint/recursive_circuit.rs` | Profile/spec/trace input derived from snapshot, replay, exec, draft, and post-state batches; no raw arena. |
| Reference relation | `checkpoint/recursive_predicate.rs` | Execute the exact typed trace and derive roots/deltas/commitments. |
| Nova constraints and folding | `z00z_recursive_proofs::nova::checkpoint` | One private uniform micro-step circuit and one segment runner. |
| Proof envelope/sidecar/receipt | existing storage-owned recursive types plus existing receipt owner | Install only a finalized, locally reverified segment snapshot. |

The old padded `RecursiveCircuitInputV1` cannot be kept as a live alternate
input. It must either be removed before merge or made decode/read-only with no
prover caller. The same applies to legacy-context public-input constructors.

### 6.10 Exact T1 trace inputs missing today

The corrected storage constructor cannot have the current signature. To
reproduce the accepted transition it needs, directly or through one typed
builder result:

```text
RecursiveTransitionTraceV2::from_canonical_transition(
    context,
    params,
    prep_snapshot,
    prep_replay_entries,
    exec_input,
    builder_draft,
    statement/core/final/link,
    post_state_batch_proofs,
    archive_manifest_bindings,
    live_limits,
)
```

The constructor must run the existing builder/store primitives while recording
the deterministic authenticated update trace. It then replays that trace
through the backend-neutral evaluator and requires byte-for-byte/digest/root
agreement with the builder output before any prover job is queued. This native
agreement is constructor hygiene and differential evidence; the Nova circuit
still constrains the same operations independently.

### 6.11 Linear event grammar and constant-memory execution

The tape can be linear; it does not need to import the raw batch DAG layout.
One canonical grammar is:

```text
BEGIN_BLOCK
  BEGIN_TX
    BIND_TX_ROW_AND_PROOF_CONTENT_DIGEST
    BEGIN_INPUT
      CHECK_SPENT_SET_ABSENT_AND_INSERT
      OPEN_OLD_LEAF
        BEGIN_HASH(kind = OLD_LEAF)
        SHA_BLOCK*             # actual encoded length plus exact SHA padding
        END_HASH
      BEGIN_HJMT_PATH_NODE
        BEGIN_HASH(role = OLD_PARENT)
        SHA_BLOCK*             # exact JMT domain || left || right, then padding
        END_HASH
        BEGIN_HASH(role = NEW_PARENT)
        SHA_BLOCK*
        END_HASH
      COMMIT_HJMT_PATH_NODE    # authenticate old parent, install new parent
      ...                      # repeat for every ordered path level/promotion
      DELETE_LEAF
      APPEND_SPENT_DELTA
    END_INPUT
    BEGIN_OUTPUT
      CHECK_OUTPUT_SET_ABSENT_AND_INSERT
      OPEN_NONEXISTENCE
      BEGIN_HASH(kind = NEW_LEAF)
      SHA_BLOCK*
      END_HASH
      BEGIN_HJMT_PATH_NODE / role-bound hashes / COMMIT_HJMT_PATH_NODE*
      INSERT_NEW_LEAF
      APPEND_CREATED_DELTA
    END_OUTPUT
  END_TX
  BIND_POST_STATE_BATCH_CONTENT_DIGEST*
FINALIZE_BLOCK
```

Hashing is decomposed into `BEGIN_HASH`, one `SHA_BLOCK` compression per step,
and `END_HASH`. This applies to value hashes, JMT leaf/internal hashes, domain
promotion, and trace-accumulator hashes; none is a host digest shortcut. For
example, the exact 80-byte internal-node preimage takes two `SHA_BLOCK` events
after padding, and an update level derives separate old and new parents. This
granularity is fixed by `RecursiveCircuitSpecV2`; it is not a runtime profile
choice. The measured whole-message 64-byte audit circuit was larger than this
selected one-compression lane and still completed below 256 MiB runtime RSS
(rounded), but the integrated production shape must be measured.

The running Nova state contains only fixed-size values: phase/opcode, height,
ordinals/counts, current HJMT root/path hash, SHA chaining state and partial
block metadata, spent/output-set roots, delta/witness/journal accumulators,
statement/parameter/predecessor bindings, and finalization flag. Trace payloads
are supplied one event at a time. Thus peak R1CS/key memory depends on one event
or hash round, not on 64 MiB or the number of earlier events.

The expected tape root and exact counts are bound at `BEGIN_BLOCK`. Each
semantic event receives a domain-separated canonical event commitment after
any required leaf/content digest is complete; a separate scheduled generic
hash subtrace updates the ordered trace accumulator and `COMMIT_TRACE` installs
it. This avoids trying to use one SHA lane for leaf, JMT parent, and transcript
compression simultaneously. Finalization
requires the accumulator to equal the statement-bound tape root. Every
transition consumes exactly one phase-allowed event and advances the ordinal.
`FINALIZE_BLOCK` requires the declared terminal ordinal, empty partial hash
state, all expected counts, and every final root/digest equality. There is no
generic `NOOP` that can skip an event before finalization.

## 7. Soundness obligations before implementation can be accepted

- **Completeness:** every valid canonical witness has a deterministic trace
  ending in the expected statement values.
- **Content binding:** changing exact retained bytes, framing, or length changes
  the collision-safe storage content digest; changing a constrained content
  digest, typed field, or structural order rejects the circuit/statement.
- **No omission:** finalization requires consumed semantic row/path/update
  counts equal declared canonical values.
- **No padding ambiguity:** the semantic trace has explicit counts and no padded
  raw arena; any fixed micro-step cells not selected by the opcode are
  zero/equality constrained.
- **No reorder/replay:** step ordinal and row/path ordinal are constrained and
  monotonically advance in the allowed phase.
- **No duplicate IDs:** statement-derived challenges, exact original/sorted
  counts, two grand-product equalities, and strict canonical-byte ordering
  constrain spent/output uniqueness; a host sort result is never trusted.
- **Semantic replay:** replay effects are calculated by circuit constraints,
  not imported from native validation.
- **Authenticated storage:** each membership/non-membership/update path is
  checked against the prior root and yields the next root.
- **Leaf/root binding:** semantic old/new leaf fields and variable canonical
  leaf bytes are constrained to the exact `Sha256Jmt` value/leaf hash; a
  prover-supplied digest cannot stand in for this relation.
- **Hash equivalence:** every in-circuit hash/root operation exactly matches the
  storage-owned domain, framing, byte order, and version, or a separately proved
  versioned translation relation binds the two.
- **Unique finalization:** `done` is unreachable before all required work and
  final equality checks; no post-finalization witness can change state.
- **Recursive binding:** verification binds initial state, final state, step
  count, public-parameter digest, and compressed proof to the canonical receipt.
- **Shape invariance:** synthesizing any legal or illegal opcode produces the
  same matrices; opcode values select algebraic results, never Rust control flow.
- **Fail-closed recovery:** a crash cannot persist or load an intermediate fold
  as a completed block receipt.
- **Constant-time boundary:** secret-dependent native preprocessing and circuit
  witness generation must not introduce a new exposed timing oracle; public
  lengths may determine the number of folds only if the threat model permits it.

### 7.1 Maximum permitted cryptographic claim

A valid T3 receipt may claim only that the pinned classical Nova compressed
verifier accepted the exact `CheckpointTransitionConsistencyV2` execution from
the bound initial state for the bound number of micro-steps and returned the
bound finalized state. It must not claim:

- end-to-end transaction validity, because transaction proof verification is
  outside this relation;
- data availability/retrievability, because archive checks are separate;
- canonical admission authority, because the Phase 069 lane is shadow evidence;
- post-quantum security, because Pallas/Vesta Nova/Spartan/IPA relies on
  classical elliptic-curve assumptions and SHA-256 does not change that; or
- security of the legacy V1 commitment family.

The proof's concrete soundness additionally depends on correct R1CS gadgets,
the pinned Nova implementation, parameter/verifier-key integrity, SHA-256
collision/preimage resistance for the roles used, strict canonical encodings,
and local verification before persistence. Those dependencies and exact
revisions belong in the manifest/receipt, not in caller-supplied booleans.

## 8. T1–T4 corrected execution map

| Task | Required correction | Exit evidence |
| --- | --- | --- |
| T1 | Delete the padded-arena invariant; enforce semantic operation caps; add the checkpoint-V2 domain-separated SHA-256 commitment and settlement-root generation; introduce the sole V2 statement/public-input/trace path and one V2 proof-batch envelope in the existing settlement owner; perform the authority-pinned deterministic V1-to-V2 storage migration and derive the live settlement/check root from the canonical HJMT definition root/layout/policy; build the trace from real snapshot/replay/exec/draft/HJMT primitives; emit field-bound actual-length canonical leaf serialization/SHA blocks and ordered update paths from that same owner; make the native reference evaluator derive the transition. | Builder-produced positive vectors; permanent V1 alias regression; SHA-256 V2 framing vectors; exact terminal-tag/bincode-v2 field/varint/trailing-byte vectors; exact JMT value/leaf/internal/placeholder vectors; successful clean/restart/idempotent migration plus stale/snapshot/policy/network/root/partial-write rejection; V1-to-V2 root-generation/envelope vectors and definition-root transitivity proof; arbitrary-result fixture now rejects; typed-field/canonical-byte mismatch, raw-backend-root aliasing, V1-envelope/V2-root substitution, and pre/new-root role mismatch reject; all raw bytes remain retrievable; source scan shows no second live constructor/profile/theorem/JMT walker. |
| T2 | Implement one directly compiled fixed-shape R1CS with one-hot selectors, an always-present one-block SHA compression lane, one generic HJMT path begin-and-commit lane, algebraic next-state selection, typed replay, ordered old/new HJMT parent subtraces, two-pair sorted/permutation uniqueness accumulators, V2 settlement/check-root derivation from the resulting canonical backend root, delta/journal derivation, predecessor binding, and block finalization. | One production micro-step `ShapeCS` and real Nova setup fit the declared budget; complete native/R1CS differential corpus agrees; every opcode/selector/order/root/path/leaf-byte/padding/digest/count/parameter/challenge/sorted-ID mutation rejects; exact 80-byte JMT internal hashes use two blocks and each update level derives/authenticates old and new parents; changing backend root, layout/policy, or derived V2 settlement root rejects; duplicate work is linear in ID count; setup under one opcode accepts valid traces containing every other opcode without shape drift. No native validity bit, V1 Poseidon2 settlement-root lane, native sort verdict, or host `match` controls constraints. |
| T3 | Implement one continuous real Nova IVC across blocks; snapshot/compress only after `FINALIZE_BLOCK`; locally verify exact genesis `z0`, cumulative step count, returned final state, proof/params/public input/predecessor; add typed preflight/cancel/private-state recovery/atomic persistence and a production-only verifier-gated receipt scope. | One valid multi-fold block proof and 3/5 completed-block chains verify from the same running IVC; restart-with-digest, duplicated event zero, intermediate fold, skipped work, wrong step count, changed endpoint/predecessor/proof/parameter/recovery state, timeout, cancellation, and resource exhaustion cannot issue a receipt. |
| T4 | Measure per-micro-step shape/key/RSS, exact fold-count estimates, valid minimum and maximum-depth semantic traces, full block segment proving/compression/verification, and 64 MiB retention/preflight behavior; run crypto/security/spec/performance reviews. | Peak memory is bounded by the micro-step profile rather than archive cap; real release verifier evidence exists; 64 MiB data is retained and SHA-256-V2-bound without a 64 MiB R1CS array; preflight exposes the exact role-framed block count (at least 1,048,578 for one full-cap trace part) plus semantic work; any worker budget failure is typed, never SIGABRT/OOM or false success. |

### 8.1 Mandatory task order

T2 cannot start by extending the current arena circuit. T1 must first freeze
the corrected relation and remove the incompatible live constructors. T3
cannot prove a partial T2 relation. T4 cannot turn a resource rejection into a
cryptographic pass. Plan 06 remains locked until all four corrected tasks have
their actual release evidence.

### 8.2 Required replacement of erroneous Plan 051 clauses

The following Plan 051 requirements are invalid and must be rewritten rather
than “worked around”:

| Current clause | Why invalid | Replacement |
| --- | --- | --- |
| Fixed `67,108,864`-byte arena allocated and constrained in one R1CS | Category error; caused the measured OOM and does not implement replay. | 64 MiB storage cap plus fixed semantic caps and a small micro-step circuit. |
| `test_full_cap_shape_constraints` must materialize the arena | Tests an implementation accident. | Test one fixed micro-step shape, full semantic trace completion, exact cap/preflight, and real proof verification. |
| One Nova fold call proves one whole block | Conflicts with bounded memory for variable work. | One finalized block segment contains one or more folds; one snapshot/receipt per block. |
| Chain length constrained to `3..=5` inside the block predicate | Confuses evidence fixtures with production state. | Unbounded contiguous block chain; separate 3/5 evidence tests. |
| Current HJMT blob vector is transition witness | It contains single-root batch proofs and its root role conflicts with the builder. | Explicit pre-state, running update, and post-state trace roles. |
| Current predicate map digest freezes the theorem | It hashes prose labels, not executable relation. | Versioned trace grammar + circuit/parameter digest. |
| Successful native validation can seed the circuit | Native facade is incomplete and false-positive. | Independent constraints plus differential comparison to corrected evaluator. |
| V1 `hash_zk` is the exact checkpoint byte commitment | Its eight-byte field map is aliased and its rate-seven sponge has only one field of capacity. | One explicit checkpoint-V2 SHA-256 commitment era; V1 read-only. |
| Constraining raw HJMT codec/DAG bytes proves the state update | The current verifier opens typed paths against one root; it does not derive the ordered post-root. | Canonical builder emits typed sequential update paths; circuit hashes canonical leaves and derives every running root. |
| `SettlementStateRoot` is the HJMT path root and V1 `root_bind` securely links any remaining value | Paths reconstruct the distinct `backend_root`; the V1 pair/checkpoint binds use the defective V1 hash. | One root-generation cutover defines live `SettlementStateRootV2` as the exact SHA-256-V2 commitment of generation/layout/policy/definition-tree root; V1 root/binds become read-only. |
| Existing `prior_output_root`/`output_root` are Nova accumulator roots | The current public input maps them to storage roots; the concepts are different. | Versioned public input binds storage roots and recursive accumulator endpoints separately. |
| Existing `issue_checkpoint_receipt_v1` can be added to the smoke receipt shape | Current receipt scope is smoke-only, omits exact step/end-state bindings, and uses V1 `hash_zk` for its receipt digest. | Versioned checkpoint scope with SHA-256-V2 digest, private verifier-gated construction, exact `z0`/step/final-state bindings. |
| New independent Nova per block plus prior-proof digest is a recursive chain | No predecessor proof is verified in the new circuit. | One continuous running IVC; compress non-consuming snapshots at finalized block boundaries. |
| Full 64 MiB proof success on this host is required for theorem correctness | The live failure contract permits typed resource failure of the non-authoritative proving job. | Prove maximum semantic correctness with bounded constructors/traces; measure representative real proofs and full-cap preflight/retention without false success. |
| A witness-opcode `match` can implement heterogeneous steps under plain Nova | It changes synthesized matrices with the setup witness. | Allocate every opcode lane/select candidate in every step and gate algebraically, or explicitly change backend authority. |

## 9. Evidence log

### 2026-07-13 — rejected monolithic representation

- The current full-cap `ShapeCS` run aborted under an 8 GiB limit before
  replay/HJMT/fold synthesis.
- Peak recorded RSS was approximately 5.6 GiB at abort.
- The current source-level estimate has at least 603,979,776 variables or
  constraints and drives the pinned Pedersen commitment key toward a power-of-two
  generator count far beyond the host budget.
- Verdict: valid rejection evidence for the current representation only.

### 2026-07-13 — pinned Nova API check

- `StepCircuit::arity` is fixed at synthesis time.
- `PublicParams::setup` synthesizes one primary step shape.
- `RecursiveSNARK::prove_step` accepts `&C` on every call, allowing different
  witness values while reusing the same shape and public parameters.
- Verdict: a uniform witness-varying micro-step circuit is compatible with the
  pinned Nova API in principle.

### 2026-07-13 — real bounded SHA/Nova release measurement

A temporary repository binary instantiated the pinned Pallas/Vesta Nova path
with one fixed 64-byte SHA-256 message step. The step allocated all 512 input
bits, used the pinned whole-message SHA gadget (therefore two compression blocks
including padding), bound 128 digest bits into the returned Nova state, then
performed public-parameter setup, one-step IVC verification, Spartan
compression, and compressed verification.

Commands, both release-only:

```text
/usr/bin/time -v timeout 600 cargo run --release \
  -p z00z_recursive_proofs --bin audit_sha256_microstep
/usr/bin/time -v timeout 300 \
  target/workspace/release/audit_sha256_microstep
```

Observed ready-binary result, exit `0`:

```text
primary_constraints=55205
primary_variables=55151
secondary_constraints=10349
secondary_variables=10331
setup_ms=877
ivc_ms=41
compress_and_verify_ms=1783
maximum_resident_set_size_kib=261984
swaps=0
```

The first command, including parallel release compilation, peaked at 1,344,000
KiB and also exited `0`. The temporary binary was deleted after measurement so
it cannot become an alternate circuit/API path.

Verdict: the multi-gigabyte failure is not intrinsic to a bounded SHA/Nova
step. A production direct-opcode circuit should use the public pinned
`sha256_compression_function` for exactly one block and must be remeasured with
the full selector/state lanes. This experiment is feasibility evidence only;
it does not close T1, T2, or T3.

### 2026-07-13 — existing fold/receipt scope check

- `adapter.rs` defines `NovaCircuit = TrivialCircuit` under `cfg(test)`.
- `nova_fold_compress_verify` proves and verifies one state-preserving step with
  `num_steps = 1` and serializes the resulting compressed proof.
- `receipt.rs` exposes only `ReceiptScopeV1::LibrarySmoke`; its documentation
  explicitly forbids checkpoint/canonical authority.
- The pinned `RecursiveSNARK` and `CompressedSNARK` are serializable and expose
  exact `num_steps` verification; compressed verification returns the final
  state for caller comparison.

Verdict: dependency compatibility is real, but T3 checkpoint proof and receipt
do not exist. C-037 is refuted; C-038 is supported.

### 2026-07-13 — executable V1 field-alias falsifier

A temporary release example constructed two same-length, same-framing inputs
whose only payload word was either eight zero bytes or the little-endian bytes
of Goldilocks modulus `p = 0xffff_ffff_0000_0001`. It asserted that the framed
word streams differed exactly at that payload word and that
`poseidon2_hash` outputs were equal.

Command:

```text
cargo run --release -p z00z_crypto --example audit_poseidon2_v1_alias
```

Observed exit: `0`.

Observed common digest:

```text
[177, 241, 202, 175, 131, 218, 14, 172,
 111, 131, 225, 113, 101, 52, 226, 192,
 233, 163, 1, 124, 234, 114, 65, 129,
 83, 82, 120, 108, 28, 123, 43, 101]
```

The temporary example was removed after recording the result so it cannot
become an alternative production API. The permanent corrected test belongs
with the V2 commitment family and must assert the V1 collision plus
SHA-256-V2 separation. A future Poseidon2 H2 test must prove the same
separation.

Verdict: C-015 and E-009 are experimentally confirmed, not speculative.

### 2026-07-13 — sponge-capacity calculation

Live constants in `z00z_crypto::hash::policy` are width `8`, rate `7`, and four
output words over Goldilocks `p ≈ 2^64`. Therefore capacity is `8 - 7 = 1`
field element. The Poseidon2 paper's generic sponge bound is controlled by
capacity, so the profile reaches a generic collision bound around `2^32`
queries. Four output words do not raise this capacity bound.

Verdict: V1 cannot be promoted as a 128-bit commitment hash. Recovery uses
SHA-256 V2; any later Poseidon2 H2 must use a versioned
rate-four/capacity-four sponge profile and bind it in parameters.

### 2026-07-13 — canonical builder/witness root-role check

- `checkpoint/build.rs::build_stmt_core_v1` requires the first supplied batch
  root to equal `draft.new_settlement_root()`.
- `checkpoint/recursive_witness.rs::from_storage` stores
  `statement.prev_root()` as `prior_settlement_root`.
- its `validate` then requires every recursive HJMT material root to equal that
  prior value.
- the current positive recursive fixture manually creates arbitrary statement
  results and prior-root inclusion/non-existence batches; it never comes from
  `build_cp_draft` + `build_stmt_core_v1`.

Verdict: the existing recursive witness contract cannot consume the canonical
non-no-op builder result and must be replaced, not patched with a fixture.

### 2026-07-13 — semantic-bound audit

- storage caps replay wrappers at 64 and recursive HJMT wrappers at 128;
- `CheckpointExecTx` does not cap per-row input/output counts;
- `BatchProofLimits::v1` permits 1,024 paths and 16,384 witness nodes per batch;
- repository config declares `max_batch_ops = 1000`, but the recursive witness
  and R1CS do not enforce it against total transition operations.

Verdict: a finite low-memory circuit profile requires an explicit semantic cap
contract plus micro-step decomposition. The 64 MiB cap alone is insufficient
as a practical circuit-shape contract.

### 2026-07-13 — YOLO execution-review convergence

The repository-local
`.github/prompts/gsd-review-tasks-execution.prompt.md` procedure was applied to
the audit/recovery deliverable in six consecutive passes. Each pass treated
source code and live Phase 069 authority as evidence; none treated this document
as implementation proof.

| Pass | Result | Resolution |
| --- | --- | --- |
| 1 | Significant findings | Split HJMT metadata from SHA compression; corrected the exact 80-byte internal-node preimage, streamed old/new parents, removed hidden path-sized allocation, specified exact `sha256_256` framing, quantified full-cap fold count, and bound typed fields to canonical leaf serialization (E-030–E-034). |
| 2 | Significant findings | Separated V1 semantic and JMT backend roots, rejected whole-model V1 Poseidon proof work, selected one V2 root generation, replaced authenticated duplicate sets with two-pair sorted/permutation accumulators, and domain-separated the fallback (E-035–E-038). |
| 3 | Significant finding | Proved that V1 HJMT batch envelopes hard-code root/bind generation 1 and required one settlement-owned V2 envelope with V1 decode/audit-only (E-039). |
| 4 | Significant finding | Rejected an asserted root pair as migration evidence and required authority-pinned deterministic storage rebuild/restart/atomic cutover evidence (E-040). |
| 5 | No significant issue | Sequential claim/error IDs, solution atoms, stale-term scan, source truth, implementation-absence ceiling, and Markdown whitespace checks passed. |
| 6 | No significant issue | T1–T4 coverage, soundness dimensions, negative/mutation evidence, single-owner/cutover rules, resource arithmetic, and implementation-absence ceiling passed. |

Passes 5 and 6 are two consecutive clean code/significant-issue reviews. The
minimum three-pass rule and the two-consecutive-clean stopping rule are both
satisfied. This closes review of the **audit formulation**, not T1–T4 code.

### 2026-07-13 — two independent doublechecks

Doublecheck 1 extracted all claims, verified the pinned Nova first-step/state
API, live JMT preimage lengths/constants, `sha256_256` framing, V1 hash call
inventory, and absence of corrected V2 production symbols. It found and fixed
the initial HJMT/hash/framing/root representation omissions recorded above.

Doublecheck 2 independently re-read the live one-step-per-block wording and
typed resource-failure contract, then checked semantic-model root assignment,
distinct batch `backend_root`, V1 root-generation lock, non-consensus path
index, final completion ceiling, and V2 symbol absence. It was clean. The
authority conflict remains explicit `REQUIRES CHANGE`; it is not silently
resolved by terminology.

### 2026-07-13 — final current-tree release regression evidence

The mandatory validation sequence completed against the audited worktree:

| Gate | Result | What it proves and does not prove |
| --- | --- | --- |
| `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` | exit `0`, `=== BOOTSTRAP COMPLETE ===` | Early release-mode regression gate passed. |
| `cargo test --release` | exit `0` through final workspace integration and doc-tests | The current tree is regression-green. Tests explicitly marked ignored by their owners remained ignored; this command does not manufacture the missing T1–T4 tests. |
| `cargo build --release` | exit `0`, 58.18 seconds | The complete workspace release build succeeds. |
| `bash scripts/audit/audit_release_feature_guards.sh` | exit `0` | Release-only feature misuse guards remain fail-closed. |

The full release test was monitored to its real final exit code rather than
inferred from partial output. In particular, the existing recursive-proofs
smoke tests and current storage tests pass, while the old full-cap arena test
remains an explicitly ignored falsifier of the rejected representation. None
of these results alters the implementation-absence ceiling: the V2 migration,
typed trace, corrected circuit, continuous checkpoint IVC, production receipt,
and their required mutation tests still do not exist.

## 10. Source-anchor evidence map

These anchors are the direct code/spec basis for the audit. Line numbers refer
to the audited 2026-07-13 worktree and must be refreshed after edits.

| Fact | Direct anchor |
| --- | --- |
| Full transition, old/new leaves, proof-byte digests, and differential backends are mandatory. | `069-TODO.md:548-575` |
| One statement-bound update/fold is currently worded as one step per block. | `069-TODO.md:801-806`, `069-TODO.md:1315-1321` |
| Resource exhaustion is a typed shadow-job failure, not a canonical-admission failure. | `069-TODO.md:2968-2977` |
| Builder post-state witness root role. | `crates/z00z_storage/src/checkpoint/build.rs:183-202` |
| Recursive witness labels/checks the same material as prior-root material. | `crates/z00z_storage/src/checkpoint/recursive_witness.rs:295-314`, `399-457` |
| Native predicate ends with equality checks plus `witness.validate`, not state application. | `crates/z00z_storage/src/checkpoint/recursive_predicate.rs:189-272` |
| Production predicate incorrectly constrains evidence chain length to 3–5. | `recursive_predicate.rs:226-230`, `recursive_circuit.rs:567-568,659-660` |
| V1 allocates the full 64 MiB host arena before synthesis. | `recursive_circuit.rs:545-642`, especially line 581 |
| Partial R1CS is deliberately fail-closed because semantic relations are absent. | `crates/z00z_recursive_proofs/src/nova/checkpoint.rs:23-61` |
| V1 byte packing uses eight-byte words and rate seven. | `crates/z00z_crypto/src/hash/policy.rs:11-26,122-164,182-233` |
| Existing R1CS SHA helper hashes a whole input slice; pinned frontend exposes one-block compression separately. | `crates/z00z_recursive_proofs/src/nova/hash.rs:340-351`; `nova-snark-0.73.0/src/frontend/gadgets/sha256.rs:29-75` |
| Live HJMT leaf/internal hash formula, constants, path bit order, and domain promotion. | `crates/z00z_storage/src/settlement/proof_batch_verify.rs:455-570,690-720`; `hjmt_batch_proof.rs:1-95` |
| Settlement plan root and SHA-JMT backend root are distinct and batch headers bind both through V1 hash commitments. | `hjmt_commit.rs:585-645`; `proof_batch.rs:1091-1185,1905-1931`; `proof_batch_verify.rs:190-235,445-458,720-766` |
| V1 batch verification hard-codes encoding/root/bind generation 1. | `proof_batch_verify.rs:188-235`; `proof_batch.rs:1091-1185` |
| Terminal leaf canonical encoding contains variable ciphertext and range-proof bytes. | `crates/z00z_storage/src/settlement/leaf.rs:18-42,151-173` |
| Terminal leaf bytes are tag plus bincode-v2 standard serialization. | `crates/z00z_storage/src/settlement/leaf.rs:151-173`; `crates/z00z_utils/src/codec/bincode.rs:29-93` |
| Existing real Nova path is a test-only `TrivialCircuit` smoke. | `crates/z00z_recursive_proofs/src/adapter.rs:42-51,180-235` |
| Existing receipt scope is library-smoke-only and lacks a checkpoint scope. | `crates/z00z_recursive_proofs/src/receipt.rs:20-45,88-140` |
| Pinned Nova synthesizes the first step in `new`, binds exact step count/endpoints in verify, and keeps serializable running state. | `nova-snark-0.73.0/src/nova/mod.rs:319-475,567-610,679-940` |
| Semantic op limit exists in config but is not applied by the witness relation. | `checkpoint_contract.yaml:249-251`; compare `recursive_witness.rs:253-314,399-457` |
| Legacy and explicit context constructors coexist. | `recursive_checkpoint.rs:137-323`, especially lines 159 and 260 |
| 163 V1 hash call tokens exist across 70 Rust files. | Reproducible `rg` inventory recorded in E-022. |

## 11. Remaining implementation blockers and resolved questions

Resolved by this audit:

1. The 64 MiB padded arena is not required and must be removed.
2. The fixed-shape unit is a bounded typed micro-step, not the maximum retained
   byte package.
3. The primary V2 checkpoint commitment family is domain-separated,
   length-prefixed SHA-256. A capacity-four/seven-byte Poseidon2 H2 profile is
   documented only as a later measured optimization.
4. Opaque transaction-proof bytes are retained/content-addressed; only their
   SHA-256-V2 digest enters the semantic row because their verifier is out of scope.
5. One checkpoint block is one finalized IVC segment, not necessarily one
   library fold call. This wording still needs to be promoted into live TODO/
   plan authority before code can claim compliance.
6. Current builder batches, witness material, native evaluator, and fixture do
   not constitute a transition proof.
7. Existing HJMT roots require a proved relation from semantic old/new leaves
   to their SHA value/leaf hashes; the minimum-change route is actual-length
   64-byte SHA streaming, not a host leaf digest or recursive projection root.
8. The active Nova object must remain continuous across blocks; compressed
   finalized snapshots do not require independent per-block IVC restarts.
9. A real SHA-bound pinned-Nova step, compression, and verification fit in
   261,984 KiB runtime RSS on the audit host. The monolithic arena OOM is not a
   lower bound for the corrected circuit.
10. The V1 semantic-model settlement root and the canonical HJMT definition
    root are distinct; keeping both live would require a second weak
    whole-model computation. The selected one-path repair is an explicit
    `SettlementStateRootV2` generation derived from HJMT layout/policy/root,
    with V1 historical after cutover.

Still blocking implementation certification:

1. Freeze the exact semantic caps and enforce them in storage constructors.
2. Implement the SHA-256 checkpoint-V2 commitment domains and the sole V2
   settlement/check-root generation and settlement-owned V2 HJMT witness
   envelope, including authority-pinned migration manifest, deterministic
   rebuild/restart/atomic-install gates, exact
   streaming SHA block/padding grammar, JMT formula/transitivity vectors, and
   cross-backend vectors; retain V1 read-only and keep Poseidon2 H2 disabled.
3. Instrument the canonical builder/HJMT owner to predict/emit the pre-state,
   ordered
   update, and post-state typed trace data without a second state engine.
4. Replace the false-positive native evaluator/fixture and prove differential
   agreement with `build_cp_draft`/`build_stmt_core_v1`.
5. Replace the audit-only SHA feasibility binary (already removed) with the
   production directly compiled fixed-shape selector/SHA/semantic step and
   measure setup/prove/verify/compress RSS before fixing the resource profile.
6. Rewrite `069-051-PLAN.md` acceptance clauses to the corrected T1–T4 map.
7. Implement the complete circuit, continuous multi-fold/multi-block runner,
   proof envelope, exact step/endpoint verifier, production receipt scope,
   private recovery boundary, and all mutations.
8. After corrected code exists, rerun release bootstrap/test/build plus crypto, security, performance,
   spec-to-code, three YOLO execution reviews, and two consecutive clean final
   reviews.

No T1, T2, T3, T4, Plan 051, or Plan 05 completion claim is permitted while any
corresponding item above lacks code, tests, and reproducible verifier evidence.
The solution is identified; it is not yet implemented.

## 12. Formal doublecheck amendment: scope, corrections, and proof status

This section is the controlling mathematical/security review of the proposal in
Sections 1–11. It preserves the earlier reasoning as audit history but
supersedes any conflicting construction or claim. In particular, the original
E-038 challenge schedule and the earlier generic HJMT parent-walk description
are not safe enough to implement verbatim.

### 12.1 Exact meaning of “proved” in this audit

Four different statements must not be conflated:

| Level | What can be established now | Status |
| --- | --- | --- |
| Mathematical relation | Given the assumptions in Section 14, every accepting execution of the corrected relation has the stated transition property except with the explicit advantage bound in Section 16.11. | CONDITIONALLY PROVED AS A DESIGN |
| Backend theorem | The pinned Nova/Spartan/Pallas/Vesta construction provides the required IVC argument if its published assumptions and the corrected two-curve implementation apply. | CONDITIONALLY SUPPORTED |
| Rust implementation | The live Rust constraints implement the corrected relation exactly. | FALSE TODAY: THE CODE DOES NOT EXIST |
| Production certification | The implementation is secure against all relevant attacks and side channels. | NOT CLAIMED; REQUIRES IMPLEMENTATION, INDEPENDENT AUDIT, AND RELEASE EVIDENCE |

No finite document or test suite can unconditionally prove collision resistance
of SHA-256, discrete-log hardness on Pasta curves, random-oracle behavior of
concrete hashes, correctness of a third-party crate, or absence of all Rust
bugs. The maximum honest result is a reduction: a successful forgery implies a
break of at least one enumerated assumption, an implementation defect, a
trusted-cutover violation, or a violation of the canonical input contract.

### 12.2 Audited source snapshot

The formal review used the live worktree at Git `466ed5d3fcc6d2ce2d2c4c7536fe6e1e2ef6f252`
with uncommitted Phase 069 work preserved. Relevant file digests were:

```text
Cargo.lock                                                        9f7b98df6a62dbebaa5daeb8958747b5c275b7651e4dc7862a8656e6cdc93dbe
crates/z00z_recursive_proofs/Cargo.toml                           9d989f85c91dfcaabcef873a73be3bf2e851cbad0b7e9f36dd857fd55b6732b7
crates/z00z_recursive_proofs/src/adapter.rs                       8244fd2bd7ef6d835056ba72f4178b9dd2014be3b421dbe44dfc6470e8d21f5a
crates/z00z_recursive_proofs/src/nova/checkpoint.rs               2444cbd9e0ea472aeb13a1fd28151e2ef334d5cab18a9bae477c2e13dd3a623a
crates/z00z_storage/src/settlement/hjmt_store.rs                  164b60a0776a2291cc9f916d51c58ed9cd1b0bdaf2cdaf0810d8bb853e2dbb4e
crates/z00z_storage/src/settlement/hjmt_commit.rs                 6883eccbcc19f8e725f5d105c2a6f0aa2d8702f0d8904893b14ed9b0f9ace44c
crates/z00z_storage/src/settlement/proof_batch_verify.rs          4459e66498306126208f42e81f2fc886b0086c9f4891c496fd4891c58fad8951
crates/z00z_crypto/src/hash/sha256_hash.rs                        2863b83d2366b01a9e8adbb695a1deb32c3bf3a15822b419aea556a295866d82
```

The pinned cryptographic stack is `nova-snark = 0.73.0`, `jmt = 0.12.0`,
`bincode = 2.0.1`, and `sha2 = 0.10.x` as resolved by that lockfile. Any change
to these files or versions invalidates line-level conclusions and requires this
audit to be rerun.

Primary cryptographic references are the
[Nova paper](https://eprint.iacr.org/2021/370), the security correction for the
[two-curve Nova construction](https://doi.org/10.4230/LIPIcs.AFT.2023.18), the
[HyperNova random-instance zero-knowledge construction](https://eprint.iacr.org/2023/573),
the [Spartan paper](https://eprint.iacr.org/2019/550), the
[Microsoft Nova implementation](https://github.com/microsoft/Nova), and
[FIPS 180-4](https://doi.org/10.6028/NIST.FIPS.180-4). These references support
conditional reductions and algorithm definitions; they do not certify the
project-specific circuit.

### 12.3 New confirmed logical errors

#### E-041 — The original permutation witness was adaptive to known challenges

E-038 derived `(alpha, beta)` from the statement and transaction root, then let
the prover supply the sorted list. That ordering does not justify a
Schwartz–Zippel bound: after learning the evaluation point, a malicious prover
may choose a different sorted list designed to collide at that point.

Correction: before any challenge is derived, a collision-resistant
`uniqueness_precommit_v2` must bind the exact original ID sequence, exact sorted
candidate sequence, set kind, counts, a frozen **pre-uniqueness context**,
circuit, parameters, and encoding version. That context excludes the uniqueness
precommits themselves and every challenge/product derived from them. The circuit
recomputes the context and precommit from both lists.
Only then may it derive four independently domain-separated 248-bit challenges
and make a second pass over the same committed lists. Section 16.6 proves the
corrected bound. An authenticated-set construction remains the deterministic
fallback if this precommit/two-pass contract is rejected.

#### E-042 — An opening proof was still being treated as an update proof

`BatchProofBlobV1` authenticates a leaf/non-leaf under one root. JMT insertion
can split an existing leaf; deletion can coalesce a leaf or preserve an internal
node. Hashing an “old parent” and a “new parent” along one assumed path does not
cover those cases and can admit an unrelated new tree.

Correction: the sole `HjmtStore` owner must obtain the exact upstream
`UpdateMerkleProof<Sha256>` from `Sha256Jmt::put_value_set_with_proof`. The V2
trace freezes its leaf and typed sibling-node data and the circuit independently
implements all upstream transition cases: empty insertion, same-key update,
split insertion, deletion with internal sibling, deletion with leaf coalescing,
and sole-leaf deletion. Native `verify_update` is differential evidence only.
The circuit additionally rejects semantic deletion of an absent key and output
insertion over an existing key, even though the generic JMT update API permits
some no-op/update cases.

#### E-043 — The cutover was stronger operationally than cryptographically

The V1 semantic root uses a commitment with a demonstrated encoding alias and
approximately 32-bit generic sponge-capacity bound. No migration procedure can
retroactively make that root uniquely bind one historical state. Rebuilding an
HJMT root and putting both roots in a manifest proves only that the configured
snapshot produced both values.

Correction: the post-cutover theorem starts from an authority-pinned canonical
snapshot digest and expected first V2 root. Its claim explicitly excludes
trustless proof of unique pre-cutover history. If a trustless translation claim
is required, the circuit must process the same snapshot through both root
algorithms, and even that only proves existence of a common preimage—not
uniqueness under the weak V1 commitment. This is a hard theorem ceiling, not an
implementation inconvenience.

#### E-044 — Cryptographic assumptions were phrased as mathematical facts

Statements such as “SHA-256 binds bytes,” “Nova proves the computation,” and
“compressed proof is private” are conditional. SHA-256 collision resistance,
Pasta discrete-log hardness, Fiat–Shamir/random-oracle behavior, Pedersen
generator independence, Nova folding soundness, Spartan knowledge soundness,
and random-instance zero knowledge are assumptions or published reductions.

Correction: Section 14 enumerates every assumption and Section 16.11 gives a
union-bound-style maximum claim. Tests can falsify implementations; they cannot
prove these hardness assumptions.

#### E-045 — The overall security level was overstated by local 248-bit challenges

The pinned Nova library truncates protocol challenges to 128 bits, uses
Poseidon as its folding random oracle, Keccak-256 for the IPA transcript, and
SHAKE256-derived Pedersen generators. Therefore a 248-bit or two-pair local
permutation argument does not make the complete system stronger than the
weakest 128-bit backend component.

Correction: target at most 128-bit classical security before accounting for the
actual cumulative fold count. The local permutation error must be comfortably
below the backend error; it is not the system security level. E-055 adds the
mandatory finite-fold correction. The receipt must identify the exact backend/
hash/curve/parameter suite and cumulative step count.

#### E-046 — Digest-to-field conversion was not fully frozen

A 256-bit digest generally does not fit injectively into the Pallas scalar
field. Modulus reduction creates aliases, and byte-order ambiguity creates
cross-backend disagreement.

Correction: every public 32-byte digest/root is represented as sixteen
range-constrained little-endian `u16` limbs. The SHA digest itself is produced
as eight big-endian `u32` words per FIPS 180-4, serialized to 32 bytes, then
split into the same sixteen little-endian limbs. No single-field digest cast is
permitted. Challenge derivation is a separate explicit 31-byte map and may not
be reused for commitments.

#### E-047 — A streamed circuit can still have an unbounded host witness

Replacing the R1CS arena with `Vec<TraceEvent>` would move, not remove, the
memory problem. A 64 MiB object plus expanded SHA/JMT events could again consume
large memory before proving.

Correction: the storage owner exposes a bounded, rewindable trace source. Pass
one computes exact commitments/counts; pass two yields one fixed-size event at
a time. It may use retained storage and bounded read buffers, never a complete
expanded event vector. Peak host memory, circuit shape, commitment key, and
private recovery state each receive separate measurements.

#### E-048 — Continuous IVC state and public compressed proof have different privacy

`RecursiveSNARK` serializes relaxed witnesses and is private recovery material.
The pinned implementation documents zero knowledge only for the compressed
proof after random-instance folding. Treating the running object as a public
proof leaks witness material; publishing an `Active` state also leaks partial
hash/state data and permits incomplete receipts.

Correction: only a locally verified `CompressedSNARK` at an `Idle/done` block
boundary is public. Recovery state is either not persisted or is stored through
an authenticated-encryption/atomic private-state owner; it is never accepted as
a receipt and is wiped on invalidation/reorg.

#### E-049 — Canonical serialization dependency drift was a consensus risk

The terminal leaf is tag byte plus `bincode::config::standard()` output. A
version, serde-schema, varint, enum-layout, or trailing-byte change changes the
JMT value hash. Merely pinning a crate version in prose is insufficient.

Correction: V2 freezes a leaf-schema digest and golden byte vectors for every
leaf family, all varint boundaries, empty/maximum variable fields, and strict
no-trailing-byte decoding. The circuit uses the frozen grammar, not Rust layout
intuition. Any schema/dependency drift requires an explicit state/commitment
version and migration.

#### E-050 — A design proof was still at risk of becoming a completion claim

The current `CheckpointNovaCircuitV1` allocates the monolithic arena, binds only
two public counters, and intentionally returns `Unsatisfiable`. The only real
Nova execution is a test-only `TrivialCircuit`; the receipt is
`LibrarySmoke`-only. None of the corrected V2 types, JMT update traces,
constraints, continuous runner, or production receipts exist.

Correction: the final verdict remains `BLOCKED FOR IMPLEMENTATION`. Section 18
is an exact implementation contract, not evidence that it has been performed.

#### E-051 — The first corrected transcript was still circular

The first draft of Section 16.6 placed `statement_digest` inside `U`, while the
public statement itself contained `U`. That is a hash fixed-point dependency,
not a non-circular Fiat–Shamir transcript. A prover could not construct it by
the stated order and no Schwartz–Zippel argument applied to that order.

Correction: Section 16.6 now defines `pre_uniqueness_context_digest` only from
authority-selected identities, the pre-state, transaction-data root, grammar,
parameters, and declared counts. It explicitly excludes `U`, challenges,
products, final roots, and every digest that transitively contains them. The two
list commitments and then `U` are computed from that context; the final public
statement is computed last and binds both `U` values. This dependency graph is
acyclic and must be unit-tested as a typed builder, not assembled from strings.

#### E-052 — The first RAM theorem omitted sorting workspace

A rewindable event iterator does not by itself make creation of the sorted ID
sequence bounded-memory. A `Vec`/`BTreeSet` of every ID would make host RAM grow
with the block even if the R1CS stayed small.

Correction: the canonical trace builder uses a bounded-memory external merge
sort into private, append-only, digest-checked spool files, or an existing
snapshot-owned ordered cursor proven byte-for-byte equivalent. There is no
complete expanded event vector. Disk bytes may be `O(B)`; resident sort buffers
are capped by the profile and included in measured `b`. Section 16.12 claims
independence from `B` only for resident memory under this contract and uses
measured library memory functions rather than an unsupported linear bound.

#### E-053 — Cross-set ID semantics was inferred from an incomplete owner

`apply_batch_checkpoint` keeps separate batch-wide `spent_seen` and `out_seen`
sets and, viewed alone, permits one terminal ID to be consumed and recreated at
a different path. The live HJMT commit owner is stricter: `SeenOps::touch`
rejects one `TerminalId` at two distinct `SettlementPath` values, while
`check_exec_ops` requires the same terminal operation set as the execution
rows. A delete-old/put-new path move is therefore not a canonical live
checkpoint operation. The two owners currently disagree because the draft
builder does not enforce the storage restriction itself.

Correction: V2 permits one ID once in each set only as an exact same-path
replacement, including a possibly identical leaf. A changed definition/serial/
bucket path is rejected by the checkpoint relation. T1 must make the existing
draft builder and storage commit share this single rule and add differential
tests; it must not preserve two interpretations. A real path/policy move belongs
to a separately authorized, versioned policy-transition theorem, not a hidden
checkpoint branch.

#### E-054 — The public statement and expected Nova endpoint formed a second cycle

The first amendment said the public statement bound the “expected finalized
output,” while the finalized running state itself contained the public-statement
digest. Hashing either full object into the other creates another fixed-point
dependency.

Correction: Section 13 freezes an acyclic construction order. The transition
statement is built first from canonical transition results; checkpoint/link data
then binds that statement; the recursive public input binds both plus the prior
finalized-state digest and verifier identities; only then is expected `z_N`
constructed with the recursive-public-input digest in its state cells. Neither
`z_N`, proof bytes, nor a receipt is an input to that public-input digest.

#### E-055 — “128-bit Nova” ignored cumulative finite-challenge error

The audited crate source samples each folding challenge from 128 bits. The
relaxed-R1CS fold contains terms through `r^2`; for a false
degree-at-most-two residual, a conservative Schwartz–Zippel bound is at most
`2 / 2^128 = 2^-127` for one independent NIFS challenge. `N` is the number of
logical application steps, not the number of such challenges. For the exact
`nova-snark 0.73.0` construction and `N >= 1`:

```text
F_recursive(N)  = 2*(N-1)
F_compressed(N) = 2*(N-1) + 3 = 2*N + 1

epsilon_fold_recursive(N)  <= 2*(N-1) / 2^127
epsilon_fold_compressed(N) <= (2*N+1) / 2^127.
```

The two per post-base step are the secondary and primary ordinary NIFS folds;
compressed proof creation adds one ordinary secondary fold and two relaxed
randomization folds. Verification recomputes these challenges and does not add
new independent fold challenges. This is not the complete Nova security
reduction, but any complete concrete claim must be no stronger than its
finite-challenge term. For example, at `N = 2^20` the local term is
`2^-106 + 2^-127`; the conservative integer `lambda_fold` is therefore 105,
not 107 or 128.
The million-step streaming route therefore changes both performance and the
concrete error budget.

Correction: profile, public input, receipt, and measurements bind the exact
cumulative `N` and proof form; Section 16.11 includes the corrected term. T4
reports `lambda_fold = 127 - ceil(log2(max(1,F_compressed(N))))` for the local
ceiling and never labels the result “128-bit” without an instantiated published
reduction justifying that label. Across multiple adversarial proof attempts, T4
also reports the sum of distinct challenged folds, not only the largest `N`.
Nova remains non-PQ, non-admission evidence. A production segment length or SHA
batch width may be frozen only after this bound and measured liveness are
acceptable.

## 13. Corrected formal statement and adversary model

### 13.1 Public statement and private witness

Let `F` be the Pallas scalar field with modulus

```text
p = 0x40000000000000000000000000000000224698fc0994a8dd8c46eb2100000001.
```

Let `enc32(x)` be the injective vector of sixteen `u16` little-endian limbs of
one 32-byte string `x`. Each limb is range-constrained to `[0, 2^16)`. Let all
counters be separately bit/range constrained; field arithmetic is never used
as an implicit wrapping integer operation.

The canonical V2 objects are built in this strict dependency order:

```text
base authority/pre-state/execution inputs
  -> transaction root, exact counts, original/sorted list commitments
  -> P -> spent/output U -> local challenges
  -> transition results, delta/witness/trace/journal roots
  -> transition_statement_digest
  -> checkpoint_id -> checkpoint_link_digest
  -> recursive_public_input_digest X_h
  -> expected finalized z_h
  -> compressed proof -> verified receipt
```

No arrow may point backward. In particular, trace/witness commitments exclude
`transition_statement_digest` and `X_h`; the transition statement excludes the
link and recursive proof; `X_h` excludes `z_h`, proof bytes, and receipt; and the
receipt is built last.

The recursive public block input `X_h` must contain or
collision-resistently bind:

```text
version, chain/network/genesis/context
height h and predecessor height h-1
predicate/circuit/profile/spec digests
Nova public-parameter and compressed-verifier-key digests
previous and next V2 settlement/check roots
previous and next canonical definition-tree backend roots
transaction-data, delta, witness/trace, journal, and checkpoint-link digests
uniqueness_precommit_v2 for spent and output lists
exact row/input/output/net-operation/JMT-update/hash-block/event counts
prior finalized IVC-state digest/required fields and expected transition fields
```

The typed statement builder first computes
`pre_uniqueness_context_digest` from version, chain/network/genesis/context,
height/predecessor, old V2 settlement/backend roots, transaction-data root,
predicate/profile/spec/trace-grammar/verifier-bundle identities, and exact
declared counts. It excludes both uniqueness precommits, all derived
challenges/products, new roots, delta/witness/journal/link digests, and the
final statement/public-input digest. After both uniqueness precommits and the
derived transition outputs are known, `transition_statement_digest` binds the
transition fields. The checkpoint/link is derived next. Finally `X_h` binds the
transition statement, link, prior finalized-state digest, both precommits,
parameters, counts, and expected transition fields. Expected `z_h` is built
from `X_h` afterward and is never hashed back into `X_h`. This order is part of
the canonical encoding.

The private witness `W_h` consists of canonical replay rows, retained-content
digest openings, typed old/new leaves, the two original/sorted ID sequences,
canonical net effects, exact JMT update proofs for every hierarchy level, and
the bounded event stream. Opaque transaction proof bytes are not interpreted by
this theorem; their exact V2 content digest and length are. Any old/new leaf
bytes that determine a live JMT value hash are fully related to their typed
fields and streamed hash input.

Define the deterministic block relation `R_step(z_i, w_i) = z_(i+1)` by the
fixed micro-step circuit in Section 18. The block relation is:

```text
R_block(z_(h-1), X_h; W_h) = z_h
```

iff a finite, nonempty sequence of legal `R_step` applications begins at the
prior `Idle` state, consumes all exact counts, derives all expected commitments
and roots, and reaches one `Idle(done_h = 1)` finalized state. The cumulative
relation from cutover state `z0` through height `h` is the ordinary repeated
application required by Nova.

### 13.2 Adversary

The adversary controls all prover-supplied witness/event/proof bytes, ordering,
padding, private recovery bytes, and worker reports. It may choose malformed
JMT proofs, exploit field aliases, grind Fiat–Shamir transcripts, attempt
cross-context/cross-version replay, select an unauthorized verifier key, stop
before finalization, continue after finalization, or report false resource
success. It may corrupt an external proving worker and archive provider.

The adversary does not control the verifier's authority-pinned context,
cutover manifest, expected verifier bundle, live limits, canonical statement,
or locally executed verifier. Canonical admission remains outside this shadow
proof lane. If those trusted inputs are attacker-controlled, the theorem does
not promise canonical correctness.

### 13.3 Security goals

- **Knowledge soundness:** an accepted compressed proof implies knowledge of a
  witness satisfying the exact cumulative R1CS relation, under backend
  assumptions.
- **Transition soundness:** a satisfying witness derives the claimed V2 state
  transition and cannot omit/reorder/duplicate semantic work except with the
  stated cryptographic error.
- **Context non-malleability at the application layer:** the verifier compares
  every context, parameter, step-count, predecessor, and final-state limb with
  its authority-selected values. A related-statement folding proof is not
  accepted under a different receipt.
- **Zero knowledge of the compressed proof:** only the upstream compressed
  construction's conditional ZK claim is inherited. Public statement fields,
  counts, total step count, timing, and proof size are intentionally leaked.
- **Fail-closed robustness:** malformed, stale, partial, oversized, cancelled,
  resource-exhausted, or unverifiable artifacts cannot issue a receipt.

Not claimed: end-to-end transaction validity, archive availability, unique
pre-cutover history, post-quantum security, canonical-admission authority, or
constant-time execution independent of public trace length.

## 14. Explicit assumptions and trust ledger

| ID | Assumption | Kind | Failure consequence |
| --- | --- | --- | --- |
| A-01 | The verifier receives the correct canonical Phase 069 statement/context/limits from the storage authority. | Protocol trust | A proof may correctly establish the wrong externally selected statement. |
| A-02 | The authority-pinned cutover snapshot digest and first V2 root identify the intended state. | Upgrade trust | Post-cutover proofs may start from an attacker-selected state. |
| A-03 | SHA-256 is collision/second-preimage resistant for all V2 commitment and JMT roles under the total query budget. | Cryptographic | Bytes, tree nodes, statements, or receipts may equivocate. |
| A-04 | Domain separation and the exact `sha256_256` length grammar are injective before SHA compression. | Deterministic | Cross-role or split/concat ambiguity may occur. This is proved for the specified grammar in Section 16.2. |
| A-05 | The concrete Poseidon random oracles used by Nova behave as required by the Nova reductions and their 128-bit challenge truncation. | Cryptographic/ROM heuristic | Folding soundness may fail. |
| A-06 | Keccak-256 transcript challenges and SHAKE256-derived Pedersen generators behave as required; generator discrete-log relations are unknown. | Cryptographic/ROM heuristic | IPA/Pedersen binding or transcript soundness may fail. |
| A-07 | Discrete logarithm is hard in the prime-order Pallas/Vesta groups for the target security level. | Cryptographic | Pedersen/IPA and the classical proof system may be forged. |
| A-08 | `nova-snark 0.73.0` implements the corrected secure two-curve Nova construction, including both curve checks, and its published Nova/Spartan reductions apply to the selected types. | Dependency correctness | A valid library verification may not imply R1CS knowledge soundness. |
| A-09 | Random-instance folding plus the selected Spartan compression provides zero knowledge as documented by the pinned implementation. | Cryptographic/dependency | Compressed proofs may leak witness information. Soundness is a separate property. |
| A-10 | OS randomness used by Nova is unpredictable and not repeated or attacker-controlled. | Operational cryptography | Folding blinding/ZK and possibly proof security may degrade. |
| A-11 | The frozen leaf, JMT, statement, trace, and receipt encodings are canonical and native/circuit implementations agree byte-for-byte. | Implementation correctness | A proof may bind a different object than storage. |
| A-12 | The fixed-shape circuit really allocates all lanes for all witnesses, constrains inactive lanes, and has no unconstrained variables affecting output. | Implementation correctness | Setup/proving shape drift or forged transitions may occur. |
| A-13 | Verifier parameters/keys are authority-pinned, strictly decoded, canonically re-encoded, digest-checked, and never accepted from the proof itself. | Key/config integrity | An attacker may choose a weak/different relation or verifier. |
| A-14 | All declared maximum counts fit their frozen integer widths and constructors reject overflow before field conversion. | Arithmetic correctness | Counter wrap or length ambiguity may permit omission. |
| A-15 | Public artifacts are persisted only after local compressed verification and exact endpoint comparison; writes are atomic/idempotent. | Operational correctness | Partial or unverified artifacts may be mistaken for proofs. |
| A-16 | Conditioned on a fixed precommit, the four domain-separated local SHA-256 challenge outputs behave as independent random-oracle samples; adversarial precommit grinding is charged as `q_U`. | ROM heuristic | The two-pair permutation error bound may not apply to concrete SHA-256. |
| A-17 | The exact selected Nova construction satisfies a named polynomial-depth knowledge-soundness theorem, including its algebraic-group, Fiat–Shamir, hash, adaptive-query, and recursion-depth premises, for the pinned implementation and parameter suite. | Cryptographic/reduction applicability | Local per-fold root counting does not imply knowledge soundness for the cumulative IVC depth. |

Assumptions A-11 and A-12 are not acceptable as permanent “trust us” items.
They are implementation proof obligations addressed by differential vectors,
mutation tests, shape fingerprints, code review, and external audit. Tests
reduce implementation risk but do not turn A-03/A-05–A-10/A-17 into theorems.

## 15. Canonical corrected construction

### 15.1 One storage-owned, rewindable trace

`RecursiveTransitionTraceV2` is a logical object, not a materialized `Vec`.
The sole storage constructor returns a bounded `RecursiveTransitionTraceSourceV2`
that supports:

1. `precommit_pass()` — stream canonical replay/net/JMT data to compute exact
   counts, content commitments, trace commitment, and both uniqueness
   precommits;
2. `event_pass()` — restart from the same immutable snapshot/version and yield
   one fixed-size `CheckpointStepWitnessV2` at a time;
3. `finish()` — prove that the source version/snapshot did not change and that
   pass-two counts and digests exactly equal pass one.

The committed `source_record_stream_v2` and the derived Nova control schedule
are different layers of the same authority, not two authorities. The source
stream contains canonical replay, leaf, net-effect, JMT, hierarchy, and
finalization records. The schedule deterministically expands those records
into fixed-shape micro-steps. `trace_digest` commits only the source-record
bytes. It MUST NOT include the `BEGIN_HASH`/`SHA_BLOCK`/`END_HASH` instructions
used to recompute that digest; otherwise trace expansion is recursively defined
as identified by DC2-F17. The spec digest fixes the expansion algorithm, and
the circuit constrains source offsets, record boundaries, expansion counts,
and final digest so this separation creates no second path.

The source holds bounded read buffers and snapshot handles only. Mutation of
the underlying version, short reads, changed bytes, different event counts, or
different pass digests fails before a receipt. This is the mechanism that makes
host peak memory independent of the 64 MiB cap.

Original and sorted ID sequences are produced without an unbounded collection.
The builder writes fixed-width `(set_kind, terminal_id, replay_ordinal)` records
to a private append-only original spool while hashing them, then performs a
stable external merge sort with a profile-capped resident run buffer. It writes
one private sorted spool, rejects adjacent equal IDs, and hashes that exact
sequence. Every pass rechecks file length, record count, canonical encoding,
and digest before yielding events. Temporary paths are never authority inputs;
only their constrained bytes and commitments are. Files use exclusive create,
owner-only permissions, atomic finalization, and deletion/zeroization policy.
An existing snapshot-owned ordered cursor may replace the spool only after a
differential test proves the same exact record sequence.

### 15.2 Replay to canonical net effects

Replay is processed in transaction order and constrains the exact builder
rules. For every input, it binds the resolved old leaf/path and records one
spent ID. For every output, it binds the complete new leaf/path and records one
output ID. It enforces per-row non-emptiness, exact tx-proof content digest,
global uniqueness within each set, and current-state semantics.

The spent and output lists are independently sorted by their 32-byte terminal
ID after being precommitted as specified in Section 16.6. A deterministic merge
then emits one net effect per terminal ID:

| Input occurrence | Output occurrence | Net effect |
| --- | --- | --- |
| yes | no | delete old path/leaf |
| no | yes | insert new path/leaf after proving absence |
| yes | yes, same exact storage path | replace old leaf with new leaf; identical old/new leaf is an explicit root-preserving transition |
| yes | yes, changed storage path | invalid checkpoint transition; fail before JMT scheduling |
| no | no | impossible/no emitted row |

Strict uniqueness makes these cases exhaustive. The merge is linear and uses
only two current IDs/rows. Each emitted net effect is committed and later
consumed exactly once by the canonical HJMT scheduler: a byte-different
same-path replacement emits one update, while a byte-identical replacement
emits one constrained `UNCHANGED_LEAF` record and no JMT operation. This matches
the live planner's `old_item == new_item` skip without allowing an arbitrary
omission. The distinction is necessary because the storage planner intentionally
collapses ordered replay into the final old-versus-new operation per touched ID.

### 15.3 Exact JMT update source

`HjmtStore::commit_snap` must be refactored internally, not duplicated. For
each existing `Sha256Jmt::put_value_set` call, the same owner calls
`put_value_set_with_proof` and receives:

```text
(new_root, UpdateMerkleProof<Sha256>, TreeUpdateBatch)
```

The returned update batch remains the only batch applied to storage. The proof
is strictly serialized and decoded into a frozen project-owned V2 wire that
retains, for every sequential update:

```text
tree role/id, old/new tree version, batch version, ordinal, old root, new root, key
operation kind and optional exact new value bytes
optional old leaf key/value hash
ordered sibling node kinds: Null, Leaf(key,value_hash), Internal(hash)
exact sibling count and key-bit/nibble interpretation
```

The native constructor immediately runs upstream `verify_update(old,new,ops)`
and the backend-neutral reference evaluator. The circuit does not trust either
boolean: it independently reproduces the update algorithm. Serialization is a
transport bridge for the pinned proof object, not a second tree engine.

### 15.4 Hierarchical root schedule

The circuit consumes exactly the storage planner's canonical order:

1. terminal-tree net updates grouped by `(definition_id, serial_id, bucket_id)`;
2. each changed terminal root encoded into the exact old/new `BucketRootLeaf`;
3. bucket-tree updates grouped by `(definition_id, serial_id)`;
4. each changed bucket root encoded into the exact old/new `SerialRootLeaf`;
5. serial-tree updates grouped by `definition_id`;
6. each changed serial root encoded into the exact old/new `DefinitionRootLeaf`;
7. the definition-tree update yielding the one canonical backend root;
8. V2 settlement/check-root derivation from generation/layout/policy/backend root;
9. journal/delta/witness/link finalization.

Every group/key/operation ordinal is strictly increasing under its canonical
byte ordering. Child-root bytes are reconstructed in the parent leaf encoding;
they are not caller-supplied digests. The non-consensus path index may be
updated by storage but is not included in the state-root theorem; its
consistency remains a separate storage invariant.

### 15.5 Fixed-shape micro-step

One `CheckpointNovaCircuitV2` implements `StepCircuit<PallasScalar>`. Every
call allocates the same one-hot opcode selectors and all candidate lanes:

- counter/phase/finalization lane;
- typed replay/net-merge lane;
- uniqueness/precommit/grand-product lane;
- exact JMT update metadata/case lane;
- one FIPS SHA-256 compression lane;
- commitment/root/receipt-state lane;
- algebraic next-state multiplexer.

Rust control flow may populate witness values, but it may not select which
constraints are allocated. For selector bits `s_j`, the circuit enforces
`s_j in {0,1}`, `sum(s_j)=1`, phase/opcode legality, inactive-input zeroing,
and for every next-state cell:

```text
z_next = sum_j s_j * candidate_next_j.
```

Each candidate lane is allocated on every step. `FINALIZE_BLOCK` is legal only
with all counts exact, all hash/JMT submachines idle, all commitments equal,
and no pending event. After finalization, either the next legal event is
`BEGIN_BLOCK` at `h+1` or a fully state-preserving done padding step explicitly
required by the runner; there is no pre-finalization generic no-op.

## 16. Mathematical soundness argument

The following lemmas prove the project-specific composition, conditional on
Section 14. They are not proofs of the underlying cryptographic assumptions.

### 16.1 Lemma: integer and digest encoding is injective

For a byte string `x = x_0 || ... || x_31`, define

```text
enc32(x)_j = x_(2j) + 2^8 * x_(2j+1),  j in [0,15].
```

Every byte and limb is range constrained. If `enc32(x)=enc32(y)`, then equality
of each base-256 pair gives `x_(2j)=y_(2j)` and
`x_(2j+1)=y_(2j+1)`; therefore `x=y`. Since every limb is below `2^16 << p`,
field equality cannot introduce modular aliases. The same argument applies to
canonical `u32/u64` decompositions when all limbs/carries and maximum sums are
range constrained.

Consequence: roots and digests represented as sixteen limbs cannot collide due
to Pallas modulus reduction. Cryptographic hash collisions remain possible only
under A-03.

### 16.2 Lemma: `sha256_256` framing is prefix-free and role-bound

For valid domain and label strings, the helper constructs:

```text
D = "z00z.hash.v1\0"
    || u64_le(|domain|) || domain
    || u64_le(|label|)  || label

M = u64_le(|D|) || D
    || for each part_i: u64_le(|part_i|) || part_i.
```

The fixed tag identifies the helper grammar. Starting from the left, every
length determines one unique following slice and the end of the complete input
determines the part count. Thus two different `(domain,label,ordered parts)`
tuples cannot produce the same pre-SHA byte string, assuming lengths fit `u64`
and null-containing domain/label inputs are rejected as the owner requires.

V2 role separation is sound only if every role has a unique frozen
domain/label/schema tuple. Reusing one tuple for two grammars invalidates the
lemma even though the helper itself is prefix-free. `ordered_commit_v2` also
includes an exact `u32` item count as its first part; order changes the byte
string.

### 16.3 Lemma: the streamed SHA lane equals native SHA-256

For an unpadded **fully framed SHA message** of `L < 2^61` bytes, FIPS 180-4 appends one `1` bit,
the unique minimum number of zero bits, and the 64-bit big-endian value `8L`.
The resulting block count is

```text
Q(L) = ceil((L + 1 + 8) / 64).
```

The circuit stores the eight FIPS chaining words, consumes exactly one
512-bit block with the pinned `sha256_compression_function`, and constrains:

- the initial state to the eight FIPS IV words at `BEGIN_HASH`;
- each nonfinal block to contain exactly 64 message bytes;
- the final one/two blocks to contain the unique padding for the declared `L`;
- `consumed_bytes = L`, `consumed_blocks = Q(L)`, and no post-final block;
- every byte/bit and `u32` state conversion in the FIPS big-endian order.

By induction on the block index, the circuit chaining state equals the native
FIPS state after every block. The final state therefore equals
`SHA256(message)`. The length/padding constraints prevent accepting a length
extension or a different message boundary. For a raw byte payload `B`, `L` is
not generally `B`: the V2 helper hashes
`8 + |DST_role| + sum_i(8 + |part_i|)` bytes. A single 64 MiB trace part under
the current role strings gives `L=67,108,951` and `Q(L)=1,048,578`; splitting
the same bytes across `E` parts adds `8E` framing bytes. This is a work count,
not a per-step memory requirement, and it must be derived per role rather than
from the raw payload cap.

Implementation caveat: the pinned gadget's IV helper is private. The canonical
`z00z_crypto` hash-spec owner must export the frozen IV/round-contract constants
or a project wrapper; `checkpoint.rs` must not silently copy an independently
maintained SHA profile.

### 16.4 Lemma: typed leaf fields are bound to the JMT value hash

Let `Ser(leaf)` be the frozen tag-plus-bincode V2-authorized canonical bytes.
The circuit constructs every fixed field at its exact offset, constrains every
varint/length, streams each variable segment at that declared length, and
forbids trailing data. By canonicality A-11, the typed leaf has exactly one
`Ser(leaf)`.

By Lemma 16.3 the circuit value hash is `SHA256(Ser(leaf))`. It then computes
the fixed 77-byte JMT leaf preimage:

```text
"JMT::LeafNode" || key_32 || value_hash_32.
```

Therefore using typed fields from one leaf and a JMT opening from another
requires either violating a serialization constraint or finding a SHA-256
collision/second preimage. This is the missing relation in a host-supplied
`leaf_digest` design.

### 16.5 Lemma: one constrained JMT update changes only the declared key

Fix an old root `r`, key `k`, optional new canonical value `v`, and a frozen
`SparseMerkleProof` containing an optional leaf plus ordered typed siblings.
The circuit implements the pinned `jmt 0.12.0` cases:

1. reconstruct and equal `r` from the old leaf/placeholder and siblings;
2. same-key update: replace only the value hash and fold the same siblings;
3. empty insertion: replace the placeholder with the new leaf and fold;
4. split insertion: constrain the two distinct keys, exact common-prefix/nibble
   calculation, inserted null siblings, branch side, and fold;
5. deletion: require old-key membership, then constrain the first non-null
   sibling and exact internal-preserve/leaf-coalesce/empty-tree case;
6. equal the calculated result with the declared next root.

The node hashes are the exact fixed JMT leaf/internal formulas and literal null
hash. Under collision resistance, an accepting path authenticates the old
tree fragment and uniquely commits the reconstructed new fragment. All sibling
subtrees remain identical, so only `k` changes. Split/coalesce case selectors,
prefix lengths, sibling kinds, and key bits are constrained; a native enum or
branch verdict is not trusted.

For a sequence of update proofs, induction on the operation ordinal with
`r_(i+1)` from operation `i` constrained equal to `r_i` of operation `i+1`
proves that the declared final root is exactly the ordered application of all
declared key/value operations. Reordering, omission, or use of unrelated roots
breaks a root/ordinal/count equality except through A-03.

### 16.6 Lemma: corrected two-pass uniqueness/permutation argument

Let an ID `x` have sixteen little-endian limbs `x_j in [0,2^16)`. For
`beta in F`, define the formal polynomial

```text
e_beta(x) = sum_(j=0)^15 x_j * beta^j.
```

Before challenges, `uniqueness_precommit_v2` collision-resistently commits to
both fixed equal-length sequences `A=(a_1,...,a_n)` and `B=(b_1,...,b_n)`, their
order, set kind, count, and complete pre-uniqueness/parameter context. `B` is
required to be strictly increasing in canonical 32-byte lexicographic order.

First compute the frozen acyclic context:

```text
P = sha256_256(
  domain = "z00z.storage.checkpoint.uniqueness.v2",
  label  = "pre_uniqueness_context_v2",
  parts  = [version, chain_context, height, predecessor_height,
            old_settlement_root, old_definition_root, tx_data_root,
            predicate_digest, profile_digest, spec_digest,
            trace_grammar_digest, verifier_bundle_digest,
            count_digest])
```

`count_digest` is a separately framed commitment to every declared row, input,
output, net-effect, JMT-update, hash-block, and event count in fixed order.

`P` excludes `U`, all challenge/product values, the new roots,
delta/witness/journal/link digests, the transition-statement digest, and `X_h`.
The exact
non-circular list transcript is then:

```text
orig_commit = ordered_commit_v2(
  domain = "z00z.storage.checkpoint.uniqueness.v2",
  label  = "original_ids_v2",
  parts  = [u32_le(n), id_0[32], ..., id_(n-1)[32]])

sort_commit = ordered_commit_v2(
  domain = "z00z.storage.checkpoint.uniqueness.v2",
  label  = "sorted_ids_v2",
  parts  = [u32_le(n), sorted_id_0[32], ..., sorted_id_(n-1)[32]])

U = sha256_256(
  domain = "z00z.storage.checkpoint.uniqueness.v2",
  label  = "id_lists_precommit_v2",
  parts  = [P, set_kind, u32_le(n), orig_commit, sort_commit])

d_(pair,coordinate) = sha256_256(
  domain = "z00z.storage.checkpoint.uniqueness.v2",
  label  = "id_permutation_challenge_v2",
  parts  = [U, trace_grammar_digest, set_kind, pair_u8, coordinate_u8])
```

`coordinate=0` selects `alpha`; `coordinate=1` selects `beta`; `pair` is `0`
or `1`. Challenge-dependent products and challenge bytes are deliberately not
inside `U`, so there is no circular fixed-point transcript. Both list
commitments are recomputed through the constrained generic SHA lane before the
challenge-derivation phase opens. The transition-statement digest is computed
only after both set precommits and transition outputs exist. Checkpoint/link
binding follows, and `X_h` is computed last from those acyclic inputs; expected
`z_h` is then constructed from `X_h` as specified in Section 13.1.

Four independent domain-separated SHA-256 outputs are mapped as:

```text
challenge(d) = 2 + u248_le(d[0..31]).
```

The challenge set `S={2,...,2^248+1}` has size `2^248` and lies strictly below
the Pallas scalar modulus (`p > 2^254`). There is no reduction bias or alias.
For each independent pair `(alpha_k,beta_k)`, the circuit checks

```text
product_i (alpha_k - e_(beta_k)(a_i))
  = product_i (alpha_k - e_(beta_k)(b_i)).
```

If multisets `A` and `B` differ, then

```text
D(alpha,beta) = product_i(alpha-e_beta(a_i))
              - product_i(alpha-e_beta(b_i))
```

is a nonzero polynomial. To see this, regard it as a monic polynomial in
`alpha` over `F[beta]`: unique factorization makes the products identical only
when the multisets of formal limb polynomials `e_beta(x)` are identical, which
is equivalent to equality of all ID limbs with multiplicity. Its total degree
is at most `15n`. The Schwartz–Zippel bound over `S x S` therefore gives

```text
Pr[D(alpha,beta)=0] <= 15n / 2^248.
```

For two independent pairs, conditional on the fixed precommit:

```text
epsilon_perm(n) <= (15n / 2^248)^2.
```

Counts are checked exactly, so unequal lengths do not enter this bound. For the
very conservative `n <= 2^32-1`, one pair is below `2^-212` and two pairs are
below `2^-424`. If an adversary can grind `q_U` distinct precommits, a random
oracle union bound is `q_U * epsilon_perm(n_max)`, plus the probability of a
SHA-256 precommit collision. For the explicit 128-bit adversary budget
`q_U <= 2^128`, the local algebraic term is below `2^-296`. This does not turn
concrete SHA-256 into a proven random oracle; it is exactly assumption A-16.

Strict byte-lexicographic ordering of `B` is constrained by scanning bytes
`0..31`, maintaining a boolean equal-prefix flag, and requiring exactly one
first differing position with `previous_byte < current_byte`; all bytes and
comparison borrows are range constrained. This is exactly Rust `[u8; 32]`
lexicographic order and forbids duplicates. If `A` contains a duplicate, then
`A` and `B` cannot be equal multisets, so acceptance is bounded above. A zero
grand-product factor is not a special unsoundness: it is one root of the same
nonzero difference polynomial and is included in the bound.

This proof is invalid without the prechallenge commitment to both lists. That
is why E-041 supersedes the original E-038 schedule.

### 16.7 Lemma: replay and net-effect merge are equivalent

The circuit proves the original spent/output sequences are the exact sequences
extracted from ordered replay rows. Lemma 16.6 proves the corresponding sorted
sequences contain exactly the same IDs except with `epsilon_perm`. Strict
ordering gives at most one occurrence in each set. A standard two-pointer merge
therefore emits exactly one of the four cases in Section 15.2 for every ID in
the union and emits no other ID.

For each case, old/new path and leaf commitments are copied from the constrained
replay row and checked by Lemmas 16.4–16.5. Thus the emitted canonical net list
is the final pointwise effect of the ordered replay. Any attempt to replace a
row, lose an ID, invent an ID, or choose a different old/new leaf either violates
an equality/precommit or falls under `epsilon_perm`/A-03.

The canonicalized rule permits one ID to occur once in both sets only when the
resolved old and new `SettlementPath` values are byte-identical. The merge then
emits one replacement; an identical leaf is allowed but root-preserving. A path
change is rejected before JMT scheduling. Any future move rule requires an
explicit semantic version and separate policy-transition authority.

### 16.8 Lemma: hierarchical HJMT root transitively binds all consensus leaves

Apply Lemma 16.5 to every terminal-tree net batch. Lemma 16.4 binds each
resulting terminal root into its `BucketRootLeaf`; apply Lemma 16.5 to the
bucket tree. Repeat for `SerialRootLeaf` and `DefinitionRootLeaf`. By induction
over the four hierarchy levels and all canonical groups, the final definition
root commits every unchanged subtree plus every declared updated terminal leaf.

The V2 settlement root is the prefix-free V2 SHA commitment of the exact root
generation, layout version, bucket-policy digest, and final definition root.
By Lemmas 16.1–16.2 and A-03, it cannot be replaced by a raw backend root, a
different policy/layout, or a root from another generation. `CheckRoot` remains
only the existing typed wrapper of this one settlement root.

### 16.9 Lemma: finalization prevents omission and partial acceptance

Every legal non-final step advances exactly one phase-appropriate ordinal or
one bounded hash/JMT subcounter. All declared totals are public-statement-bound;
counters are monotone, range constrained, and cannot wrap under A-14.
`FINALIZE_BLOCK` requires equality of every consumed/declared count, empty
pending submachines, exact final roots/commitments, and `done=0`, then sets
`done=1`. No other opcode can set `done`.

Therefore early finalization, event omission, trailing events, double
finalization, and post-final mutation are unsatisfiable. This conclusion fails
if a counter is not public-bound, can wrap, or an inactive lane can affect the
selected next state; shape/constraint review must check those facts
microscopically.

### 16.10 Lemma: Nova binds the complete sequence and endpoints

Let the fixed circuit relation be `F_step`. The pinned constructor evaluates
event zero in `RecursiveSNARK::new`; the first `prove_step` call only changes
the internal count from zero to one. The only correct runner schedule is:

```text
recursive = RecursiveSNARK::new(pp, event[0], z0)
recursive.prove_step(pp, event[0])       # records the already-evaluated step
assert recursive.num_steps() == 1
for j in 1..N:
    recursive.prove_step(pp, event[j])
    assert recursive.num_steps() == j+1
```

The verifier supplies the authority-pinned verifier key, exact cumulative `N`,
and exact cutover `z0`. The compressed verifier internally binds its public-
parameter digest and returns `z_N`; the project verifier compares every returned
limb with the canonical finalized state, including height, predecessor,
statement, roots, accumulator, and done flag.

At every `BEGIN_BLOCK`, the circuit hashes all required cells of the actual
prior finalized running state and compares that digest/fields with `X_h` before
clearing transient cells. This prior state is the preceding Nova output, not a
caller-provided witness. Expected `z_h` is constructed only after `X_h` exists,
and verification compares all cells; no digest of `z_h` is fed back into `X_h`.

Under A-05–A-10, A-16–A-17, and backend knowledge soundness, acceptance implies knowledge
of witnesses for all `N` applications of the one fixed R1CS relation. Combined
with Lemmas 16.1–16.9, this gives the cumulative transition claim. A digest of
an independent prior proof is insufficient; continuity comes from the same
running recursive object and `z0`/`N`/`z_N` verification.

### 16.11 Composition theorem and maximum advantage claim

Let:

```text
Adv_backend   = advantage against pinned Nova + Spartan/IPA knowledge soundness
Adv_dlog      = advantage against Pallas/Vesta discrete log/Pedersen binding
Adv_ro        = distinguishing/Fiat–Shamir advantage for Poseidon, Keccak, SHAKE, and local SHA challenges
Adv_sha       = SHA-256 collision/second-preimage advantage over the total application query budget
epsilon_perm  = q_U * (15*n_max / 2^248)^2
F_fold        = 2*N+1 for one compressed proof of N >= 1 exact Nova steps
epsilon_fold  = F_fold / 2^127 for the conservative degree-two, 128-bit-challenge local union bound
Bad_impl      = event that circuit/native/encoding/parameter code violates A-11..A-15
Bad_cutover   = event that the authority-selected initial snapshot/root is wrong
Bad_authority = event that runtime authority/config/snapshot identity differs from A-01/A-13
```

Then a conservative application-level reduction is:

```text
Adv_forge <= Adv_backend + Adv_dlog + Adv_ro + Adv_sha
           + epsilon_perm + epsilon_fold
           + Pr[Bad_impl] + Pr[Bad_cutover] + Pr[Bad_authority].
```

This formula is a responsibility ledger, not a numerical certification.
`Pr[Bad_impl]`, `Pr[Bad_cutover]`, and `Pr[Bad_authority]` cannot be assigned
cryptographic numbers by unit tests. `Adv_backend` must be instantiated from the corrected Nova/Spartan
reduction for the exact suite and may already subsume tighter or additional
finite-challenge terms; the explicit `epsilon_fold` is retained as a
fail-closed ceiling, not double-counted in a final numerical report. The
complete-system claim is **no more than 128-bit classical security and may be
materially lower at large `N`**. It exists only after implementation review
makes the non-cryptographic terms acceptably small. There is no post-quantum
claim.

### 16.12 Theorem: peak memory need not scale with 64 MiB

Let `C` be the constraint/variable count of one fixed micro-step, `A` its state
arity, and `b` the total capped resident read/external-sort buffer. Let
`M_setup(C,A)`, `M_fold(C,A)`, `M_compress(C,A)`, and `M_verify(C,A)` be the
actual peak allocations of the pinned library paths, including commitment keys
and temporary vectors. For plain Nova with the rewindable source:

```text
M_peak <= max(M_setup(C,A), M_fold(C,A),
              M_compress(C,A), M_verify(C,A)) + O(b)
resident trace/sort memory = O(b)
private spool/retained input bytes = O(B) on storage, not resident RAM
total constraint work = O(N * C)
```

The inspected `RecursiveSNARK` stores current relaxed/strict instances and
witnesses, not a vector of all prior steps. `N` grows with replay/JMT operations
and SHA compression blocks, but no project or library structure is permitted to
store all `N` steps. Thus peak resident memory is independent of the 64 MiB
package cap **if and only if** the production source uses the bounded external
sort/stream contract, the running proof remains constant-size in `N`, and the
integrated `C` itself fits the declared budget.

This proves the asymptotic correction and refutes the mandatory-gigabyte claim.
It does not predict the final constant. Real `ShapeCS`, setup, fold,
compression, verifier, trace-source, and recovery-state RSS measurements remain
mandatory before T4 can pass.

### 16.13 Side-channel claim boundary

Fixed R1CS shape prevents the proof relation from changing with a witness; it
does **not** prove the Rust prover is constant-time. Private replay/content/leaf
bytes, JMT paths, event buffers, the running `RecursiveSNARK`, and recovery
state are secrets. Statement fields, declared counts, cumulative step count,
proof size, and authorized resource outcomes are public by protocol design.

The circuit may branch in Rust to assign witnesses only when every branch
allocates the identical constraints, but trace construction, parsing, external
sorting, storage reads, error paths, and third-party proving arithmetic can
still have witness-dependent timing or memory access. Therefore the proven
privacy claim is only the pinned compressed proof's conditional zero knowledge
against a proof recipient. It does not cover a hostile OS, co-resident process,
cache observer, or physical attacker.

Production must isolate the proving worker, disable core dumps, restrict spool
and recovery files, avoid secret-bearing logs/errors, zeroize bounded buffers,
and expose only typed public progress. If hostile co-residency enters scope, the
profile must additionally pad I/O/event schedules to public maxima and an
optimized-artifact constant-time review must pass; source inspection and timing
tests alone cannot certify that property.

### 16.14 Liveness and throughput are separate from soundness

The streaming theorem removes `C = Omega(B)` and bounds resident memory, but a
one-compression-block step can require more than one million Nova steps for
64 MiB before replay and JMT work. That is a potential compute/latency DoS, not
a soundness failure. The document therefore does not claim the selected
granularity is operationally viable.

T4 must measure candidate compile-time SHA batch widths off the live path,
including setup/CK RSS, fold count, wall time, cancellation latency, and proof
verification. Exactly one width is then frozen in `RecursiveCircuitProfileV2`
before production parameters are generated; all candidates and diagnostic
profiles remain non-live. If no measured width meets both memory and latency
budgets, T2 remains blocked and authority must select one different theorem-
preserving route, such as a single audited inner chunk-proof relation recursively
verified by the same canonical outer owner. A digest-only/native-only shortcut
is never an alternative.

## 17. Adversarial security and robustness matrix

| Attack/failure | Required invariant/control | Residual risk |
| --- | --- | --- |
| Setup with one opcode, prove with another shape | All lanes synthesized unconditionally; shape/parameter digest equal for every opcode witness. | Compiler/library or missed witness-dependent allocation defect. |
| Non-boolean or multiple opcode selectors | Boolean constraints plus `sum(selectors)=1`. | None beyond constraint correctness. |
| Inactive-lane data influences state | Zero/equality constrain inactive inputs and algebraically select every output cell. | Missed output/input cell in selector audit. |
| Omit/reorder/replay a trace event | Public exact counts, phase machine, strict ordinals, ordered trace commitment, unique finalization. | SHA collision or constraint omission. |
| Choose sorted IDs after seeing challenges | Both original and sorted sequences included in prechallenge V2 precommit; circuit recomputes it. | ROM grinding bound and SHA collision. |
| Duplicate spent/output ID | Strict sorted list plus two-pair committed permutation proof. | `epsilon_perm`; one occurrence across both sets is allowed only for exact same-path replacement. |
| Same ID recreated at a different path | Circuit compares complete resolved old/new `SettlementPath` and rejects mismatch before JMT scheduling. | A future path/policy move needs its own authorized versioned theorem. |
| Digest field-modulus alias | Sixteen range-constrained `u16` limbs, never one reduced scalar. | Byte-order implementation defect. |
| SHA split/concat/domain confusion | Exact helper framing, unique role labels, golden preimage vectors. | Role registry collision or schema drift. |
| SHA padding/length extension | Exact declared unpadded length, unique FIPS padding, exact block count, no post-final block. | SHA gadget defect. |
| Typed leaf differs from hashed leaf | Circuit constructs frozen tag/bincode bytes from constrained fields and variable segments. | Codec/schema mismatch. |
| JMT opening substituted for update | Exact `UpdateMerkleProof` cases and old-to-new running-root induction. | Incorrect mirror of pinned JMT proof wire/algorithm. |
| Split-insertion prefix error | Range-constrained 256 key bits; exact common-prefix and null-sibling count; boundary tests for every prefix length. | JMT dependency semantic drift. |
| Delete nonexistent becomes accepted no-op | Replay semantic selector requires matching old leaf/key before generic delete. | Missed semantic gate. |
| Deletion coalesces wrong sibling kind | Sibling kind constrained and all internal/leaf/null branches allocated; new root independently derived. | Gadget defect. |
| Child root substituted at hierarchy boundary | Exact parent leaf serialization includes child root and identity fields; next tree update consumes that hash. | SHA collision/serialization defect. |
| Raw backend root masquerades as settlement root | Typed generation plus V2 derivation binds layout, policy, and backend root; verifier compares all limbs. | Authority selects wrong generation/policy. |
| V1 proof/root/receipt downgrade | The recursive-proof V1 decoder, constructor, feature, config, and runtime path are absent; inert migration bytes are content-addressed and never decoded by the live binary. | Forgotten compatibility constructor/caller or stale deployed binary. |
| Prover chooses parameters/verifier key | Authority-pinned verifier bundle digest checked before decode/verify; exact Nova pp digest also checked internally. | Compromised authority/config. |
| Nova event-zero off-by-one | Fixed constructor/first-call schedule and `num_steps()==consumed_events` assertion. | Runner regression. |
| Independent per-block proof chain | One continuous `RecursiveSNARK`; cumulative `z0,N,zN`; compressed snapshots borrow the same running object. | Lost private state may require replay, not a new asserted chain. |
| Intermediate fold issued as receipt | Receipt constructor requires final `Idle(done=1)` state and local compressed verification. | Private/public type confusion. |
| Related-statement folding malleability | Application verifier compares exact canonical statement/context/predecessor/final state, not just library success. | Backend vulnerability outside pinned corrected construction. |
| Malformed/oversized proof decode | Bound bytes before allocation, strict decode, canonical re-encode, then cryptographic verify. | Third-party deserializer bug/DoS within cap. |
| Tampered private recovery state | AEAD/atomic generation binding or no persistence; on load, validate context/pp/state/step count and resume only privately. | Host compromise and key theft. |
| Worker claims success without proof | Only local verifier-gated receipt has success capability. | Local verifier compromise. |
| Reorg/stale lineage | Bind chain context, height, predecessor, statement, and generation; quarantine/wipe stale running state and artifacts. | Incorrect canonical fork selection by trusted input. |
| OOM/time/cancel | Preflight exact step/hash counts; bounded buffers; typed resource outcome; never catch abort as success. | Liveness loss; no proof for that block. |
| Witness leakage through logs/debug/core dumps | Secret types omit `Debug`/serialization, zeroize bounded buffers, disable core dumps in worker profile, restrict recovery files. | OS/hardware side channels; proving time and memory/disk access may reveal witness-dependent structure unless all such structure is public. |
| V1→V2 false equivalence claim | Authority-pinned snapshot digest/root and explicit post-cutover-only theorem. | Trust in upgrade authority; no cryptographic recovery of weak V1 uniqueness. |

## 18. Exact implementation blueprint

This is the required order. A later step may not be used to hide an unfinished
earlier relation. Names ending in V2 denote the one post-cutover live encoding;
the recursive-proof V1 code, decoder, feature, configuration, and runtime path
are physically absent. Historical V1 bytes may survive only as inert,
content-addressed migration/audit evidence that no live decoder consumes. No
alias, shim, second constructor, second JMT walker, or runtime-selectable
theorem is permitted.

### 18.1 Authority normalization before code

Update `069-TODO.md`, `069-051-PLAN.md`, matrix/profile/stop-split, and the
referenced recursive-proof specification so they state exactly:

- one completed statement-bound IVC **segment** per block, containing one or
  more uniform internal Nova steps;
- 64 MiB is a storage/witness-package cap, not a monolithic R1CS allocation;
- the post-cutover commitment/predicate/root encoding is V2 and the complete
  recursive-proof V1 executable/decoder surface is removed before V2 lands;
- the corrected precommitted permutation argument and exact JMT update-proof
  relation are mandatory;
- Plan 06 remains blocked until T1–T4 have real code and release evidence.

If authority refuses any item, planning must select and fully specify a
different sound route (for example the authenticated-set fallback or an inner
proof). Code must not reinterpret the prose locally.

### 18.2 T1 — canonical storage relation and trace

1. In `z00z_crypto::hash`, keep `sha256_256` as the sole byte-hash owner. Add a
   typed V2 domain/label registry and exported frozen FIPS SHA profile constants
   needed by the circuit; do not add a second SHA implementation.
2. In `settlement::identity`, add `RootGeneration::SettlementV2` and the sole
   `SettlementStateRoot::settlement_v2` constructor. Post-cutover live builders
   cannot construct V1. Existing `CheckRoot::from(SettlementStateRoot)` remains
   unchanged and unique.
3. In the existing settlement root owner, implement
   `derive_settlement_root_v2(generation, layout, policy_digest, definition_root)`
   using exact `sha256_256` framing and golden preimage vectors.
4. Add `SettlementRootGenerationCutoverV2` in the existing migration/storage
   owner. Its only constructor consumes authority configuration, canonical
   snapshot identity, expected V1/V2 roots, network/context, and cutover height.
   Rebuild/restart/atomic-install checks from E-040 are mandatory.
5. Refactor `HjmtStore::commit_snap` internally to a single
   `commit_snap_with_update_trace` implementation using
   `put_value_set_with_proof`. The existing commit path delegates to it and
   discards the trace only for explicitly non-recursive historical callers; do
   not execute `put_value_set` separately.
6. Define a frozen `JmtUpdateTraceV2` wire in the existing settlement
   proof-batch owner. Strictly translate the pinned update proof, retain node
   kinds needed for split/coalesce, run upstream native `verify_update`, and
   freeze encode/decode/golden vectors.
7. Add exactly one V2 settlement proof-batch/trace envelope for V2 generation,
   SHA binds, ordered net operations, hierarchy roles, and update proofs. V1
   objects cannot enter its live constructor or verifier.
8. Replace the live padded `recursive_circuit.rs` data model with
   `RecursiveCircuitProfileV2`, `RecursiveCircuitSpecV2`, and the rewindable
   `RecursiveTransitionTraceSourceV2`. No old recursive V1 input, type,
   constructor, decoder, fixture, feature, or compatibility symbol may remain.
9. Freeze semantic maxima in storage constructors: rows, inputs/row,
   outputs/row, total spent/output/net ops, touched tree groups, JMT updates,
   siblings/update, leaf bytes, content bytes, SHA blocks, events, and cumulative
   step count. Every derived multiplication/addition uses checked arithmetic.
   The profile also freezes the external-sort resident buffer, spool byte cap,
   exclusive-create/permission policy, and maximum sort merge fan-in.
   Spool I/O uses the canonical `z00z_utils` streaming/atomic I/O boundary; if
   exclusive streaming creation is missing, extend that owner once instead of
   calling raw filesystem APIs from the checkpoint theorem.
10. Rewrite `recursive_predicate.rs` as the backend-neutral evaluator of the
    exact replay, committed sort/merge, JMT update, hierarchy, V2 root, delta,
    journal, and finalization relation. It must derive results, not compare a
    caller-supplied validity result.
11. The trace constructor consumes the real prep snapshot/replay/exec/draft and
    post-state builder/store artifacts. It compares its result with
    `build_cp_draft`, `build_stmt_core_v1`'s versioned successor, the applied JMT
    batch, and reloaded storage before exposing a proving source.
12. Add one private storage-orchestration value,
    `CanonicalCheckpointTransitionV2`, inside the existing checkpoint owner.
    It resolves replay, enforces same-path replacement, creates the one HJMT
    operation schedule, commits through `commit_snap_with_update_trace`, and
    derives draft/statement/trace artifacts from that same result. Live V2
    callers must enter through this value. No historical recursive V1
    `BuildState` verifier or reconstruction path may remain; unrelated
    versioned storage contracts are retained only when the T0 reachability
    inventory proves they are not part of the recursive-proof V1 scheme.

Required core APIs, kept within their existing owners:

```text
derive_settlement_root_v2(...) -> SettlementStateRoot
HjmtStore::commit_snap_with_update_trace(...) -> (RootHash, JmtUpdateTraceV2, HjmtTreeSnap)
RecursiveTransitionTraceSourceV2::from_canonical_transition(...) -> Result<Self,...>
RecursiveTransitionTraceSourceV2::precommit_pass(...) -> RecursiveTraceCommitmentsV2
RecursiveTransitionTraceSourceV2::event_pass(...) -> bounded iterator/reader
CheckpointTransitionConsistencyV2::evaluate_stream(...) -> RecursiveFinalStateV2
CanonicalCheckpointTransitionV2::from_exec(...) -> Result<Self,...>
```

### 18.3 T2 — one complete fixed-shape R1CS

Implement only private
`z00z_recursive_proofs::nova::checkpoint::CheckpointNovaCircuitV2`. Its
`StepCircuit::arity()` is the frozen length of `CheckpointRunningStateV2`.
Storage/public crates never export a Nova type.

The running state has a fixed canonical field order:

```text
invariant enc32 digests: context, predicate, profile, spec, pp/vk, statement,
  trace/witness, spent-precommit, output-precommit
chain: generation, height, previous height, prior/final accumulator bindings
phase: phase, prior opcode, event ordinal, done, validity
declared and consumed counters for every row/list/net/tree/hash/event class
SHA substate: active, role, message length, bytes, blocks, eight u32 words
uniqueness: four challenges per set, original/sorted counts and two products,
  prior sorted IDs and presence flags
net merge: spent/output cursors, current IDs, case, emitted operation count
JMT substate: tree role/id, old/new tree version, batch version,
  operation/case, key, old/new root, old leaf,
  sibling index/count/kind, prefix/coalesce state, old/new running hashes
hierarchy: current terminal/bucket/serial/definition roots and group ordinals
commitments: tx, delta, witness/trace, journal, link, V2 settlement/check roots
expected final values
```

All transient/private cells must be zero at finalized `Idle`; otherwise they
would appear in the verifier-returned final state. Digest/root cells use
sixteen limbs. SHA words use constrained `u32` decomposition. Counters use
frozen integer widths with non-wrapping checks.

Use a directly compiled opcode grammar such as:

```text
BEGIN_BLOCK
BIND_REPLAY_ROW / BIND_REPLAY_INPUT / BIND_REPLAY_OUTPUT
BEGIN_HASH / SHA_BLOCK / END_HASH
COMMIT_UNIQUENESS_PREIMAGE / DERIVE_UNIQUENESS_CHALLENGES
ACCUMULATE_ORIGINAL_ID / CHECK_AND_ACCUMULATE_SORTED_ID
MERGE_NET_EFFECT / COMMIT_UNCHANGED_LEAF
BEGIN_JMT_UPDATE / CHECK_OLD_JMT_LEAF
JMT_SPLIT_STEP / JMT_COALESCE_STEP / JMT_PARENT_HASH_COMMIT
END_JMT_UPDATE / PROMOTE_CHILD_ROOT
COMMIT_TYPED_EVENT / COMMIT_DELTA_JOURNAL_LINK
FINALIZE_BLOCK
```

The exact opcode numbers and phase transitions belong in
`RecursiveCircuitSpecV2` and its digest. The generic SHA lane performs one
compression block per `SHA_BLOCK`; every JMT/content/commitment role schedules
that same lane. All JMT cases are allocated and selector-gated on every step.

The integrated circuit must pass both a satisfying constraint-system corpus
and a real `ShapeCS -> PublicParams::setup -> RecursiveSNARK ->
CompressedSNARK::prove/verify` path. A test-system-only green result is not T2.

### 18.4 Parameter and verifier bundle

Use Pallas/Vesta, Pedersen commitments, the pinned Poseidon RO, Keccak
transcript, and `RelaxedR1CSSNARK<_, ipa_pc::EvaluationEngine<_>>` explicitly.
Call `PublicParams::setup` with the selected Spartan `ck_floor()` functions.
Do not enable `test-utils` or a pairing/SRS backend implicitly.

Create one authority-owned immutable verifier bundle containing:

```text
format/version and backend suite
Cargo.lock/dependency revision digest
circuit/profile/spec/trace-grammar digests
canonical serialized PublicParams and CompressedSNARK verifier key
Nova pp.digest canonical field bytes
constraint/variable/arity counts for both curves
project SHA-256 digest over all preceding framed fields
```

On load: byte-cap, strict decode, canonical re-encode equality, suite/version
check, project digest check, Nova pp digest check, and expected authority digest
must all pass before proof decode. Proof bytes cannot carry/select this bundle.

### 18.5 T3 — continuous runner, compression, receipt, and recovery

Implement one private `CheckpointNovaRunnerV2` that owns one continuous
`RecursiveSNARK` from cutover `z0`. It streams events using the exact schedule
in Lemma 16.10, checks `num_steps()` after every call, and never restarts per
block. At `FINALIZE_BLOCK` it:

1. compares the native reference final state with the expected statement;
2. creates a non-consuming compressed snapshot;
3. immediately verifies it with authority-pinned `vk`, exact cumulative steps,
   exact `z0`, and compares every returned final-state limb;
4. installs the compressed bytes into the one storage-owned proof envelope and
   existing accepted-sidecar path;
5. re-verifies the complete storage envelope/context/link/predecessor;
6. only then calls the private constructor for a V2 checkpoint receipt.

The receipt binds the full verifier bundle digest, `z0`, cumulative step count,
complete `zN` digest/required fields, statement/link/predecessor, compressed
proof digest, backend revision, and verification result with V2 SHA framing.
There is no caller-set accepted boolean.

Recovery state is private. Preferred order: first implement deterministic
replay from the last finalized snapshot; add persistence only through an
existing project-owned AEAD/secret-storage facility. If no such canonical
facility exists, do not invent an unaudited encryption shim—leave private IVC
recovery persistence disabled and replay.

### 18.6 T4 — measurement and operational gates

Measure release-mode values separately for:

- direct step shape/variables/nonzeros and commitment-key generators;
- PublicParams and compressed-key setup time/RSS/serialized size;
- every opcode class and a mixed valid block;
- fold, compression, and verification time/RSS;
- trace precommit/event passes and bounded-buffer RSS;
- private running/recovery serialized size;
- 1/3/5-block continuous chains and reorg/replay;
- cumulative `N`, `epsilon_fold(N)`, and `lambda_fold` for minimum,
  representative, full-cap, and maximum authorized segments;
- representative maximum semantic counts;
- 64 MiB retention/content hashing and exact preflight step count.

The pass condition is bounded process behavior and real verifier success for
supported jobs. A full-cap job may return the authorized typed resource outcome
after preflight, but it may not OOM/SIGABRT, silently reduce caps, or be counted
as proof success.

## 19. Required proof and test corpus

### 19.1 T1 deterministic/native tests

- `sha256_256` full-preimage golden vectors for every V2 role; domain, label,
  part count/order/length mutation changes the result.
- FIPS SHA-256 byte/block vectors and lengths `0,1,55,56,63,64,65`, maximum
  permitted leaf/content length, and forbidden overflow.
- permanent V1 `0` versus Goldilocks-modulus alias falsifier and V2 separation.
- `enc32` exhaustive byte-order round trips and rejection of non-u16 limbs.
- frozen leaf bytes for all leaf families, bincode varint boundaries, empty and
  maximum vectors, tag/schema/version mutation, and trailing-byte rejection.
- JMT parity vectors for empty insert, same-key replace, split at every common
  prefix length `0..=255`, delete with internal sibling, delete with leaf
  coalescing, sole-leaf delete, absent-delete semantic rejection, malformed
  sibling kind/order/count, and old/new root mutation.
- multi-operation JMT batches proving every intermediate root is chained.
- hierarchy vectors touching one/multiple buckets, serials, definitions,
  empty-tree creation/removal, and child-root substitution; changed-path
  recreation is a negative checkpoint case.
- replay-to-net merge for delete, insert, replace, duplicate spent,
  duplicate output, required same-path consume/recreate including identical
  leaf, forbidden changed-path recreation, a second occurrence in either set,
  missing row, and reorder.
- external-sort run/merge boundaries, equal IDs split across two runs, corrupt/
  truncated/swapped spool, permission/open race, changed snapshot, buffer cap,
  cleanup after success/error/cancel, and original/sorted commitment parity.
- uniqueness precommit mutation and an executable regression demonstrating that
  the old postchallenge sorted-witness scheme is not accepted by the new API.
- cutover clean/restart/idempotent paths and wrong snapshot/network/height/
  policy/layout/V1 root/V2 root/partial-write rejections.
- differential agreement among canonical builder/store result, upstream
  `verify_update`, backend-neutral evaluator, trace pass one, and trace pass two.

### 19.2 T2 circuit and real-proof tests

- synthesize every opcode and every JMT case under the same setup witness;
  compare arity, primary/secondary constraint/variable counts, serialized shape
  or public-parameter digest, and verifier bundle digest.
- a complete valid block including every opcode verifies through compressed
  Nova and matches the native evaluator.
- mutate individually every statement limb, parameter/profile/spec digest,
  count, opcode/selector, inactive cell, phase, ordinal, SHA byte/length/padding/
  state, uniqueness precommit/challenge/product/sorted ID, net case, JMT key/
  leaf/sibling kind/hash/order/prefix/coalesce/root, hierarchy child/root,
  delta/journal/link, predecessor, final state, and done flag; each actual
  compressed verifier call rejects or returns a state rejected by exact compare.
- early/double finalization, event zero replay, skipped/trailing event, counter
  overflow, post-final mutation, and witness-dependent Rust branch all fail.
- mutation tests use independently recomputed proof bytes where necessary; a
  mere envelope checksum failure does not prove the R1CS mutation is caught.

### 19.3 T3/T4 integration, security, and recovery tests

- one block with many folds and continuous 3/5-block chains verify from the
  same `z0`; cumulative step counts and final states are exact.
- compressed snapshots at two consecutive heights do not consume/restart the
  running IVC; changing/skipping a predecessor rejects.
- wrong pp/vk, suite, dependency digest, proof bytes, proof encoding, step
  count, `z0`, `zN`, statement, context, link, root generation, backend label,
  and checkpoint hint reject before receipt.
- bounded decoder fuzz/property corpus for malformed field/curve/proof data,
  oversized lengths, trailing bytes, and noncanonical re-encoding.
- recovery replay and, if enabled, encrypted persistence: truncation, bit flip,
  nonce/tag/key/context mismatch, stale generation, reorg, rollback, partial
  rename/write, and concurrent resume reject without a receipt.
- cancellation/timeout/memory preflight produces typed non-success and leaves
  no accepted partial artifact.
- secret-log/core-dump/file-permission scans and zeroization tests cover event
  buffers and recovery artifacts.
- release performance runs record real exit codes and RSS; no result is inferred
  from partial output.

### 19.4 Mandatory verification order after implementation

For every T1–T4 task, the `<verify>` sequence is:

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`; on any
   failure stop broader validation, fix, and rerun the bootstrap.
2. Targeted release-only tests for that task.
3. `cargo test --release` and `cargo build --release` when Rust is affected.
4. Crypto, security, performance, spec-to-code, design-foundation, canonical-
   path, feature-guard, and secret-boundary gates from `.github/`.
5. `/GSD-Review-Tasks-Execution` in YOLO mode at least three times, fixing every
   issue/warning, and continuing until two consecutive runs find no significant
   code issue.
6. Two independent `/doublecheck` passes: first claim-to-source/constraint
   mapping; second adversarial falsification from proof bytes back to canonical
   storage state.

No compile-only, docs-only, native-only, shape-only, receipt-only, reduced-cap,
or ignored-test evidence closes any task.

## 20. Doublecheck convergence, supersession index, and final verdict

### 20.1 Controlling supersession index

Sections 1–11 remain historical evidence. An implementation reviewer must use
the following controlling replacements and reject the superseded wording:

| Historical item | Controlling correction |
| --- | --- |
| E-038 / Section 6.4 postchallenge sorted witness | E-041, E-051, A-16, and Section 16.6 precommit both lists before deriving challenges |
| E-030 / Section 6.11 generic HJMT parent walk | E-042 and Sections 15.3/16.5 exact pinned `UpdateMerkleProof` relation |
| E-035/E-036/E-040 V1/V2 root migration claims | E-043 and Sections 15.4/18.1–18.2 authority-pinned post-cutover V2 theorem ceiling |
| Sections 6.5/8 endpoint and receipt wording | E-048/E-054 and Sections 13.1/16.10/18.5 acyclic public-input and private-running-state contract |
| Section 7 generic “128-bit” ceiling | E-044/E-045/E-055 and Sections 14/16.11 exact conditional advantage ledger with cumulative-fold term |
| Sections 6.3/6.11 resident-memory claim | E-047/E-052 and Sections 15.1/16.12/16.14 bounded external sort plus measured library allocations |
| Earlier cross-set/path interpretation | E-053 and Sections 15.2/16.7 same-path-only replacement through one storage owner |

### 20.2 Independent doublecheck results

| Verification layer | Evidence | Result |
| --- | --- | --- |
| Live-source identity | Recomputed SHA-256 digests in Section 12.2; inspected current checkpoint builder, HJMT planner/store, hash owner, pinned Nova, and pinned JMT sources. | PASS for audit snapshot; rerun on any source/dependency change. |
| Transcript dependency graph | Topologically traced base inputs → `P` → list commitments/`U` → challenges → transition statement → checkpoint/link → `X_h` → `z_h` → proof/receipt. | PASS after E-051/E-054; no self-reference remains in the corrected construction. |
| Encoding algebra | Proved injective limb encoding, prefix-free pre-SHA framing, exact streamed FIPS padding, and typed-leaf-to-JMT hashing under A-03/A-04/A-11. | CONDITIONAL PASS. |
| Uniqueness algebra | Fixed both lists before challenge; proved the nonzero bivariate polynomial and two-pair bound; charged precommit grinding explicitly. | CONDITIONAL PASS under A-03/A-16. |
| State transition | Replaced one-root opening logic with the exact sequential update-proof cases and hierarchy induction; resolved unchanged leaf and path mismatch. | DESIGN PASS; live circuit/evaluator absent. |
| Nova composition | Verified event-zero API behavior, endpoint comparison, private running state, 128-bit challenge constant, and degree-two fold term. | CONDITIONAL DESIGN PASS; concrete security depends on exact `N` and backend reduction. |
| Security/privacy | Enumerated authority, parameter, decode, recovery, DoS, side-channel, reorg, and cutover trust boundaries. | DESIGN PASS within stated threat model; no hostile-host or post-quantum Nova claim. |
| Resource model | Replaced `C = Omega(B)` with a fixed step plus bounded resident stream/external sort; retained disk `O(B)` and total work `O(N*C)`. | ASYMPTOTIC PASS; constants/throughput remain T4 measurements. |
| Canonical architecture | One hash owner, one storage transition owner, one HJMT commit/update trace, one private Nova circuit/runner, one verifier-gated receipt. | DESIGN PASS; implementation must remove/prevent live V1/V2 parallel ownership. |
| Spec-to-code | Searched for corrected V2 profile, trace, circuit, runner, update wire, proof mutations, and production receipt. | FAIL/BLOCKED: these artifacts do not exist in live Rust. |

### 20.3 Review convergence record

1. Mandatory fail-fast bootstrap completed successfully before audit work.
2. Source/claim doublecheck confirmed pinned versions, hash framing, Nova
   event-zero behavior, JMT update APIs/cases, and expected V2 implementation
   absence.
3. YOLO review pass 1 found material transcript and resident-sort gaps; E-051
   and E-052 plus the bounded spool design corrected them.
4. YOLO review pass 2 adversarially followed proof bytes back to canonical
   storage and found the endpoint hash cycle, incomplete path-owner inference,
   unchanged-leaf omission, and cumulative-fold overclaim; E-053–E-055 and the
   corresponding lemmas/implementation gates corrected them.
5. YOLO review pass 3 found no significant remaining issue in the corrected
   document. A final independent pass is required after this convergence record
   itself is checked; its result must be recorded below.
6. YOLO review pass 4 rechecked the complete amended document against live
   sources, pinned dependency algorithms, transcript cycles, path ownership,
   error bounds, security boundaries, and V2 symbol absence. It found no
   significant document issue. Pass 5 repeated the same checks without a
   document change and also found no significant issue. Passes 4 and 5 are the
   required two consecutive clean results.

### 20.4 Final decision

The corrected mathematical construction is internally consistent **as a
conditional design reduction**: an accepted proof maps to the canonical V2
transition unless a listed cryptographic assumption, backend theorem,
authority/cutover trust premise, or implementation obligation fails, with the
explicit bound in Section 16.11.

It is not a proof that Phase 069-051 is implemented. T1–T4 remain open until the
single canonical Rust path in Section 18 exists and the complete release test,
mutation, security, performance, recovery, and real compressed-verifier corpus
in Section 19 passes. Plan 06 must not start before that evidence. No result in
this document converts the current fail-closed partial circuit, native helper,
test-only Nova smoke, or a reduced-cap run into completion evidence.

## 21. Pinned Nova/Poseidon implementation audit and triple falsification gate

This append-only vendor and integration audit narrows every claim to the exact
dependency and backend selected by this repository. It does not convert the
corrected design into live code and does not authorize Plan 06.

### 21.1 Audit target and reproducible source identity

The audited backend is `nova-snark = 0.73.0`, selected as Pallas primary,
Vesta secondary, Pedersen commitment, and Spartan `RelaxedR1CSSNARK` with IPA.
Nova embeds a Neptune-derived Poseidon implementation; there is no independent
runtime `neptune` crate on this path.

| Identity item | Audited value |
| --- | --- |
| crate checksum | `62afd983558f08e4a27a11edd6701177961379761b866a6f34b4f2bc39d1bbfa` |
| upstream commit | `666e3b25bfb9f8b2106f8b4d8057010f28b1ee79` |
| workspace `Cargo.lock` SHA-256 | `9f7b98df6a62dbebaa5daeb8958747b5c275b7651e4dc7862a8656e6cdc93dbe` |
| recursive-proofs `Cargo.toml` SHA-256 | `9d989f85c91dfcaabcef873a73be3bf2e851cbad0b7e9f36dd857fd55b6732b7` |
| Nova `src/constants.rs` SHA-256 | `f1c4100cb03dea718c504e5d32b6f5a5675512a7a0fd31f4a674b04b799e016e` |
| Nova `src/nova/mod.rs` SHA-256 | `2ceb4789f8228ffd8622bd2e3580fd03ddfb3491cbf69c5835bef7eb0d898f8d` |
| Nova `src/nova/nifs.rs` SHA-256 | `503d332048e09e3724ff6733af2bb53c9053b1aa8aea08678a0504936e74fffa` |
| Nova `provider/poseidon.rs` SHA-256 | `801c348fa773065eeb196be2cec449370fa388b7e7b8c1b1fbcdc15e018f0f0f` |
| Nova `frontend/gadgets/poseidon/poseidon_inner.rs` SHA-256 | `4f369267feda728fb66744e9058fe9c89699f1791cb622c3d43ed662afb19567` |
| Nova `frontend/gadgets/poseidon/sponge/api.rs` SHA-256 | `4acf0151bb16e4bcb01997321b26aa0838b25cad528f0f66ecd2346b38cdd325` |
| Nova `frontend/gadgets/poseidon/hash_type.rs` SHA-256 | `dd743d40834abe7a20e2a5cb6af3ad07f089d00966cacc1113e49de6b93859f7` |
| Nova `gadgets/ecc.rs` SHA-256 | `19139e95dc3affc6e8c7e9232fab4dcdef800ceccc0b552a12724714757ccfb5` |
| Nova `provider/pedersen.rs` SHA-256 | `bb2d78af5d7fb4f003cce8181e9bf7691ab3d4996ff9d030fbda46577ca16511` |

The registry source and a read-only clone at the recorded upstream commit had
no source difference after excluding registry packaging metadata, manifests,
and `.git`. The implementation MUST rerun this identity gate if a selected
file, feature, backend, checksum, or lockfile entry changes.

Applicability references are the exact
[upstream commit](https://github.com/microsoft/Nova/commit/666e3b25bfb9f8b2106f8b4d8057010f28b1ee79),
the historical [two-cycle fix](https://github.com/microsoft/Nova/commit/afd7403336fdf6625658108256f4b4163da197c9),
the upstream [security surface](https://github.com/microsoft/Nova/security),
the [2023 two-cycle analysis](https://eprint.iacr.org/2023/969),
the [2024 Nova analysis](https://eprint.iacr.org/2024/232.pdf), the
[Poseidon paper](https://eprint.iacr.org/2019/458), and recent
[Poseidon/Neptune](https://eprint.iacr.org/2025/954) and
[Graeffe](https://eprint.iacr.org/2025/950) analyses. Absence of a published
advisory is not evidence that no defect exists.

### 21.2 Exact verifier obligations

Application acceptance is valid only when one canonical path proves all of the
following:

1. the authority-pinned parameter/verifier bundle is selected before decoding
   proof bytes, and its project digest equals the storage statement binding;
2. `num_steps` is nonzero, equals the typed semantic trace length, stays below
   a checked project ceiling, and is never narrowed or wrapped;
3. every `z_0` limb equals the statement input and every returned `z_N` limb
   equals the independently evaluated canonical endpoint;
4. ordinary NIFS is used only in Nova's augmented-circuit context, where
   `U2.X[0]` binds the hash of `U1`, while relaxed NIFS absorbs both instances;
5. unchanged Nova verification passes the cross-curve hashes, both relaxed
   instances, the strict last secondary instance, the NIFS folds,
   derandomization, and both Spartan/IPA proofs; and
6. envelope, chain, height, link, context, profile, spec, parameter, prior
   output, and final output are reverified before receipt/admission.

Nova's internal public-parameter digest maps a SHA3-256 digest of legacy
little-endian fixed-integer serialization into 250 field bits. It is not
transport authentication. The project MUST bind a canonical versioned byte
representation of the approved parameter/profile/spec/verifier bundle with a
domain-separated project digest and MUST NOT let proof bytes choose it.

### 21.3 Nova implementation findings

| Candidate | Exact check | Classification | Mandatory disposition |
| --- | --- | --- | --- |
| Historical two-cycle bug | The fix commit is an ancestor of the pin; current verification recomputes cross-curve hashes and has the repaired last-instance structure. | Historical critical issue, fixed on selected source; no live exploit. | Pin checksum/commit and keep regression vectors. |
| First-step scheduling | `RecursiveSNARK::new` executes the first circuit application while the recursive counter remains zero; the first `prove_step` advances it to one before later ordinary folds. | Confirmed API/integration footgun, not verifier forgery. | One private runner MUST encode the exact schedule and assert count/endpoints at every boundary. |
| Step-count conversion | Application and backend counter types can differ. | Correctness/soundness integration risk if unchecked. | Checked maximum/conversions; overflow and wrap tests. |
| Ordinary NIFS transcript | `U1` is bound through the augmented-circuit `U2.X[0]`, so the shortened transcript is theorem-specific. | Sound only inside exact Nova construction. | Keep private; MUST NOT copy into a generic folding API. |
| Returned endpoint | Nova proves its own returned `z_N`; it cannot know the storage-intended endpoint. | Application theorem obligation. | Compare all limbs to the independent evaluator after real verifier acceptance. |
| Deserialization allocation | Length-prefixed vectors may allocate before semantic validation. | Availability/DoS integration blocker, not proof-forgery evidence. | Cap bytes before decode; strict no-trailing decode; canonical re-encode equality; fuzz/property corpus. |
| Variable-time MSM/Rayon | Pedersen/Spartan use variable-time and parallel primitives. | Local/co-resident leakage boundary; no soundness break established. | No constant-time claim; isolate prover, prevent witness logs/core dumps, zeroize project buffers where feasible, profile resources. |
| Proof-depth theorem | Security is conditional on the stated folding/group/random-oracle assumptions; standard-model Fiat–Shamir analysis adds General Zero-Testing. | Conditional theorem limitation, not an exploit. | State assumptions and enforce a finite project step ceiling; no unconditional/post-quantum claim. |

With Nova's 128-bit fold challenge, the per-compressed-proof local ceiling is:

```text
epsilon_fold(N) <= (2*N+1) / 2^127,  N >= 1
```

Backend acceptance does not discharge `Bad_impl`, `Bad_cutover`,
`Bad_authority`, SHA/JMT assumptions, or the exact application endpoint
comparison.

### 21.4 Embedded Neptune-derived Poseidon findings

The selected default wide mode has width 25 and capacity 1; the supported
narrow mode has width 6 and capacity 1. Source parameters are:

| Width | Full rounds | Partial rounds | S-box |
| ---: | ---: | ---: | --- |
| 6 | 8 | 56 | `x^5` |
| 25 | 8 | 59 | `x^5` |

Constants use the deterministic Grain-LFSR construction and a symmetric Cauchy
MDS matrix. The consulted analyses do not establish a practical break of these
selected parameters; model-specific estimates MUST NOT be generalized into an
unconditional proof.

Native and circuit random oracles use the same constants and IO pattern. The
pattern binds absorb/squeeze order into a 128-bit capacity tag. Nova calls only
fixed 128- or 250-bit squeezes. Caller-controlled zero/out-of-range squeeze
sizes can panic in the generic API, but they are not exposed by the selected
path. This is a feature-regression/robustness hazard, not false-proof evidence.

`HashType::Sponge` has a zero domain tag; separation comes from the IO pattern
and absorbed Nova transcript. The implementation MUST NOT reuse this internal
sponge as checkpoint content hash, JMT hash, envelope digest, or project domain
separator. Unsupported variable-length APIs with `TODO`/`unimplemented`
branches are not reachable; making them reachable invalidates this audit.

### 21.5 Mandatory three-model candidate verification

Every suspected theorem or cryptographic defect MUST pass all three sequential
models below before it may be called a verifier-acceptance vulnerability.
Evidence from one model cannot substitute for another.

#### Model A — logical attack on the statement

Construct two different semantic executions, witnesses, or endpoints that
satisfy the written relation under identical bound public inputs, or identify
a reduction step whose premise does not follow. The record MUST name the exact
lemma/assumption and retain executable bytes or algebraic values. OOM, panic,
decode rejection, performance regression, or an unstated assumption alone is
not a Model A forgery.

#### Model B — native/circuit/transcript/reference differential

Run identical typed trace bytes through the independent canonical V2 evaluator,
the Nova `StepCircuit`, exact native/in-circuit transcripts, and an independent
test-only transcript implementation that does not call the production helper
under test. Compare every intermediate field, domain tag, absorb, challenge,
counter, root, `z_0`, and `z_N`. A mismatch counts only when reproducible in
release mode from retained fixture bytes and after minimization.

Decoder-, checksum-, or native-only rejection does not close or prove a
cryptographic candidate because it does not reach the circuit/verifier claim.

#### Model C — repeated unchanged-verifier false-proof acceptance

Use the authority-pinned, unmodified `nova-snark 0.73.0` verifier and the exact
project endpoint to repeatedly accept proof bytes for a false semantic
transition. The harness MUST start from a valid compressed proof, mutate one
theorem component while preserving unrelated encoding/checksum fields, call
the real compressed verifier, compare every returned endpoint limb, repeat in
clean processes, and retain the minimized proof/statement/parameter fixture,
exact command, source hashes, exit status, and verifier trace.

A mock, decoder, native precheck, receipt parser, reduced circuit, or rejection
before the real verifier is not Model C evidence. Only a candidate passing A,
B, and C is confirmed critical. Only then may an exploit regression and minimal
local fork be evaluated under the existing cryptographic owner; the fork MUST
retain provenance, patch the smallest theorem-preserving surface, rerun all
upstream/workspace release suites, and MUST NOT create a second proof system or
parameter authority.

### 21.6 Triple-check candidate ledger

| Candidate | A: statement attack | B: differential | C: false proof accepted | Verdict |
| --- | --- | --- | --- | --- |
| Historical two-cycle issue | Published attack targets older construction; selected source contains fix. | Current repaired cross-curve path and regressions agree. | Not reproduced on pin. | Fixed/non-applicable. |
| First-step schedule | No break when exact schedule is followed. | Incorrect wrapper can disagree on count; mandatory regression. | No false acceptance with exact count/endpoint comparison. | Integration hazard. |
| Missing application `z_N` compare | Can turn a valid proof of the wrong endpoint into a false application claim. | Reference/circuit endpoint mismatch is detectable. | Backend may accept its endpoint; exact project endpoint MUST reject it. | Open until T3/T4 tests pass. |
| Decoder allocation | No false relation constructed. | Can allocate/reject before theorem. | Verifier not reached or rejects. | Availability gate. |
| Poseidon squeeze panic | No collision/statement attack. | Invalid generic input panics consistently; internal transcript is fixed. | No false acceptance. | Non-reachable robustness hazard. |
| Poseidon native/circuit divergence | No concrete divergence found. | Upstream parity passed; independent project differential remains mandatory. | No false acceptance. | No confirmed defect. |
| Proof-selected parameters | Breaks application authority if unbound. | Bundle digest mismatch is deterministic. | Proof can verify for attacker bundle but exact project endpoint MUST reject. | Open integration gate. |
| Variable-time proving | No false relation. | Resource traces may correlate with witness work. | No false acceptance. | Confidentiality/operations boundary. |
| Polynomial-depth/GZT assumptions | Conditional premise, no concrete counterexample. | No implementation divergence follows by itself. | No false acceptance reproduced. | Explicit theorem limitation. |

No candidate passed all three models. No critical Nova/Poseidon verifier flaw is
confirmed, no exploit is claimed, and local vendoring/forking is not authorized.
Open rows are mandatory T1–T4 implementation gates, never permission to weaken
or bypass the theorem.

### 21.7 Release evidence and its limit

The exact upstream source passed targeted release tests for Poseidon parity,
identity-point circuit addition, and nontrivial compressed IVC. Its full release
suite passed 94 tests with zero failures and three explicitly ignored external
parameter/PTau cases; doctests passed with the documented ignored examples. The
workspace lockfile graph passed the recursive-proofs release suite. The ignored
project 64 MiB arena test is evidence only for the rejected monolithic design;
it is not evidence for the corrected streaming theorem.

These runs establish dependency integrity and regression evidence. They do not
prove the absent V2 checkpoint circuit, endpoint binding, or storage transition.
T1–T4 require the complete Models A/B/C corpus against the implemented path.

### 21.8 Mandatory size and profiling evidence

Implementation completion must serialize and measure actual PublicParams,
proving/verifier keys, running/recovery state, compressed proof, storage proof
envelope, sidecar, receipt, publication/network payload, trace, spool, and
recovery artifacts. Type-size estimates, zero placeholders, and inferred values
do not count. Minimum, representative, maximum, and 64 MiB content fixtures must
enforce every cap before allocation, decode, write, or transport.

The installed mandatory tool set is `/usr/bin/time -v`, `perf stat`,
`perf record/report`, `valgrind --tool=massif`, `cargo flamegraph` or `samply`,
and `strace -c`. Missing optional tools (`hyperfine`, `heaptrack`,
`cargo-bloat`, `cargo-llvm-lines`, and `cargo-nextest`) are not completion gates.
Measurements must separate trace precommit/event passes, external sorting, SHA,
HJMT, storage commit, setup, each opcode, fold, compression, verification,
endpoint comparison, recovery, and end-to-end work. They must record cold/warm
samples, supported percentiles, CPU/RSS/allocation/page-fault/context-switch/I/O
data, serialized sizes, cap utilization, cumulative `N`, and
`epsilon_fold(N)`.

Single-thread and bounded-Rayon runs must produce identical canonical
statements, trace bytes, roots, endpoints, and receipt eligibility. Async and
thread-pool orchestration stays outside the deterministic cryptographic state
machine. Oversubscription, cancellation, timeout, backpressure, restart, and
partial-output cleanup need explicit negative tests.

### 21.9 Transfer decision for executable planning

`069-051-PLAN.md` must be adapted in place; creating `069-052-PLAN.md` would
create a second recovery authority for the same Plan 05 obligation. The plan
must contain the corrected T1 storage relation, T2 uniform circuit and verifier
bundle, T3 continuous runner/strict acceptance/recovery, T4 theorem-to-code and
measurement closeout, all E-001 through E-055 corrections, and the Model A/B/C
gate. Plan 06 remains locked until real T1–T4 release evidence passes.

## 22. Controlling formal doublecheck addendum (2026-07-14)

This addendum is the controlling interpretation where Sections 1–21 conflict
with it. It was produced by two independent proof passes and one live-code
pass:

1. a forward derivation from the authority-pinned statement to every required
   transition invariant;
2. a backward adversarial derivation from verifier acceptance to every premise
   an attacker would have to violate; and
3. a source-level check of the current workspace plus the exact cached
   `nova-snark 0.73.0` source identified in Section 21.

The result is **conditionally sound as a design and blocked as an
implementation**. A mathematical reduction can establish an implication from
explicit assumptions to an abstract relation. It cannot assign a negligible
probability to a Rust bug, an incorrect cutover snapshot, unreviewed unsafe
code, an operator choosing the wrong verifier bundle, or code that does not yet
exist. Tests can reduce implementation uncertainty; they cannot prove SHA-256,
discrete-log hardness, the random-oracle heuristic, or a library theorem.

### 22.1 Live implementation snapshot

The current workspace is later than the source snapshot recorded in Section
21:

- the recursive V1 crate/path covered by T0 has been removed from the current
  workspace;
- T1 has partial V2 work in the existing owners: typed checkpoint SHA roles,
  a streaming role-framed SHA owner, V2 settlement-root derivation, and a
  project-owned JMT update-trace wire;
- the current `Cargo.lock` and workspace manifests contain no `nova-snark`
  package, so cached registry source is a **candidate-source audit**, not a
  live dependency attestation;
- no complete `CheckpointNovaCircuitV2`, authority-pinned Nova bundle, private
  continuous runner, compressed-proof acceptance endpoint, or T4
  theorem-to-code matrix exists in the live tree; and
- T2, T3, and T4 therefore remain unproved regardless of native unit-test
  success.

The source hashes in Section 21 remain valid for the cached candidate files,
but its old workspace lockfile and removed recursive-crate manifest hashes are
historical evidence only. T2 MUST reintroduce the selected dependency through
the canonical storage owner and regenerate checksum, feature, source-file,
lockfile, backend, curve, transcript, commitment, and SNARK identities before
any live-backend claim.

### 22.2 Corrective finding ledger

| ID | Finding | Logical consequence | Mandatory closure |
| --- | --- | --- | --- |
| DC2-F01 | Earlier fold accounting equated logical steps with NIFS challenges. Exact source executes two NIFS folds per post-base step and three more during compressed-proof creation. | The earlier `N/2^127` term and `2^-107` million-step statement were optimistic. | Use `F_compressed(N)=2*N+1` and Section 23.3. Test the count against source instrumentation for `N=1,2,3,max`. |
| DC2-F02 | A finite root-counting bound is not a proof of polynomial-depth Nova knowledge soundness. | A million-step claim does not follow from Schwartz–Zippel alone. | State the exact security model and paper version; treat the 2024 polynomial-depth/Fiat–Shamir premises as assumptions unless the selected construction demonstrably satisfies them. |
| DC2-F03 | Neptune's `IOPattern` value is a deterministic polynomial accumulator in wrapping `u128`, initialized into Poseidon capacity. It is not a collision-resistant project domain separator. | Cross-protocol separation cannot be delegated to the IO-pattern tag. | Keep Nova transcript formats fixed and private; bind the project verifier bundle and statement externally; test all reached native/circuit patterns and forbid using this sponge for project content/JMT/domain hashes. |
| DC2-F04 | The audited Nova source is cached but absent from the live dependency graph. | “The current backend is Nova 0.73.0” is presently false. | T2 pin plus fresh source/feature/checksum and unchanged-verifier evidence. |
| DC2-F05 | A hierarchy tree can first materialize at global version `v>0`; its logical old root is the canonical JMT null root even when no `v-1` root node is stored. | Requiring a stored predecessor node rejects valid first updates and broke 35 storage tests. | Use stored predecessor root or canonical null root; retain a release regression for a first update at a later global version and require native/circuit parity. |
| DC2-F06 | The live write path now calls only `commit_snap_with_update_trace`, passes the exact `HjmtTreeId -> JmtTreeRoleV2`, returns each `TreeOut.trace` in commit order, and constructs one `SettlementUpdateTraceEnvelopeV2` consumed by the canonical transition owner. | The earlier role/drop defect is closed in the native path. T2 still must consume these exact retained bytes without regenerating a proof or trusting native verification. | Retain this one-owner path and add circuit/native byte parity plus restart/persistence evidence. Any later reconstruction, second update execution, default role, or trace drop reopens the finding. |
| DC2-F07 | `JmtUpdateTraceV2` caps operation count and proof bytes but does not yet cap each value, total encoded trace bytes after serialization, or enforce version adjacency in native verification. | A locally built trace can exceed the 64 MiB contract or carry semantically unrelated version fields despite a valid root update proof. | Enforce per-value, aggregate, canonical-byte, nested-decode, and work caps before allocation; require `(old,new)=(0,0)` only for genesis and otherwise `old+1=new`; bind versions/tree role in circuit and statement. |
| DC2-F08 | A streaming-SHA test that computes its expected value through `sha256_256_role` tests the same implementation twice after that role helper delegates to the stream. | The test can pass under a shared framing defect. | Compare streaming output to the independent generic `sha256_256(domain,label,parts)` path and frozen external vectors; compare every compression-state word in the circuit differential. |
| DC2-F09 | A V2 derivation must not rely on debug-only generation checks. The current live helper now returns `GenerationMix` in release mode, but the generation remains caller supplied. | Release behavior is fail-closed today, yet the API still permits a mixed-generation attempt. | Prefer a V2-only internal constructor that hardcodes generation 2; reject decoded legacy generation at the boundary before calling it. If the argument remains, retain release negative tests and prove every caller propagates the error. |
| DC2-F10 | Native update-proof verification is not the checkpoint theorem. | A valid upstream JMT proof can still have wrong replay origin, role, ordinal, counts, hierarchy parent, statement, or Nova endpoint. | T2 must independently constrain the exact update relation; no native boolean enters the circuit. |
| DC2-F11 | The live evaluator no longer accepts a caller-selected root: it reads the post-commit definition root from `SettlementStore`. However, `TraceGrammarV2` still checks only coarse opcode counts; it does not decode replay payloads, reassemble/verify the update envelope, derive net effects/HJMT/hierarchy, or prove that the observed store root was caused by this trace. | Observing an already-mutated store is not an independent transition evaluator. The current positive test is a real native commit but still cannot detect a trace/store semantic mismatch hidden behind the same post-state observation. | Derive every semantic effect and final root from the exact retained `JmtUpdateTraceV2` bytes independently of the mutation executor, then compare that derived result to the reloaded store. Enforce the full phase/nesting/count/EOF grammar and add trace/payload/root cross-substitution tests. |
| DC2-F12 | `CanonicalCheckpointTransitionV2` now owns event construction and the event-slice method is crate-private, closing the public second-authority API. It still materializes `Vec<RecursiveTraceEventV2>` and sorted ID vectors before spooling, and verifies only the caller handle's root against storage before commit. | Authority ownership improved, but the resident-memory theorem and complete immutable-snapshot binding do not follow. Caller-supplied snapshot ID/generation/count/content digest are not independently resolved from the store. | Stream directly from the frozen storage/planner owner into bounded spool/external runs; resolve and revalidate every snapshot-handle field from storage at pass boundaries. No whole event tape, caller-asserted handle metadata, or alternate trace builder may remain. |
| DC2-F13 | Live code now commits spent and output IDs separately and excludes structural opcodes, closing the original all-event-ID error. It still rejects every ID appearing in both sets, while the authority permits exact same-path replacement, and the grammar still accepts opaque internal payload multiplicities with only coarse counts. | Valid replacement can be rejected, while malformed event schedules can pass the native scaffold. | Bind path/old/new leaf semantics and allow one cross-set occurrence only for the exact same-path replacement case; implement exact phases, submachine nesting, declared counts, payload codecs, and authority-defined empty/no-op behavior. Add semantic mutation tests for every opcode and list boundary. |
| DC2-F14 | The former `32,000` event/`1,000,000` step contradiction is now rejected. The current repository fixture derives `max_source_records=65,002`, `max_sha_blocks=1,056,703`, and `E_max=1,595,719`, then pins both event and step caps to `2,000,000`. This closes the original arithmetic example, but the equation is not yet theorem-complete: source records and derived control steps still share one opcode vocabulary, and no instrumentation proves a bijection from every semantic/hash/JMT operation to exactly the counted terms. | The current fixture has `404,281` nominal steps of headroom, but an omitted work class or double expansion can still make a profile accepted yet unprovable. A conservative-looking constant is not a proof of completeness. | Freeze separate source/control types, define the checked role-by-role/event-by-event equation, and prove it by generator instrumentation: every emitted source record and every derived control step increments exactly one named term, totals equal the profile formula, and cap+1 rejects. Keep the former contradictory profile as a permanent negative test. |
| DC2-F15 | `CanonicalCheckpointTransitionV2::from_exec` commits `apply_exec_handoff_v2` before `canonical_events`, spool creation, and precommit. Any later limit, encoding, disk, or I/O error returns `Err` after storage has already changed, without returning a typed committed-root/shadow-proof outcome. | Callers can interpret failure as “no transition” while the canonical state advanced; retry/recovery and receipt logic can diverge. Rolling back after a durable commit would be equally unsafe. | Preflight every deterministic bound before commit and make the commit plus exact trace capture atomic in the existing storage transaction. After canonical admission, represent proof-source/resource failure as an explicit non-authoritative shadow outcome carrying the committed root/version, never as an ambiguous rollback-like `Err`. Test every post-commit fault point, retry, reload, and idempotence. |
| DC2-F16 | `sha_blocks_for_role_parts` now includes the frozen DST, outer prefix, every part prefix, and FIPS padding. It correctly gives `L=67,108,951`, `Q=1,048,578` for one 64 MiB trace part and `Q=1,056,703` for the fixture's declared 65,002 parts. The remaining gap is aggregation: `max_sha_blocks` names only the trace transcript, while the theorem also hashes spent/output original lists, both sorted lists, context/precommit/challenges, typed leaves/JMT nodes, statement, link, and other role transcripts. Some are represented by coarse “uniqueness steps,” but exact SHA compression totals are neither named nor proved. | Taking the maximum of role-specific work or charging one trace total does not bound several sequential hash transcripts. The final control schedule can exceed `E_max` even when each individual role fits its local cap. | Define `Q_role` for every reached transcript and use the checked **sum of all transcript instances required by one maximum transition**, with exact multiplicities; use a maximum only for mutually exclusive lanes proved selector-exclusive. Bind the full ledger into profile/spec, compare native/circuit compression counts per role, and reject missing/duplicate role charges. |
| DC2-F17 | The design vocabulary currently conflates the immutable semantic source-record stream with the derived micro-step/opcode stream. If `TRACE_DIGEST` commits every `SHA_BLOCK` micro-step while those same steps are needed to compute `TRACE_DIGEST`, event expansion is self-referential and has no finite canonical fixed point. The current native scaffold hashes materialized event encodings but the planned circuit grammar also introduces hash-expansion events. | A circular trace definition can make the honest witness impossible or allow native/circuit implementations to commit different objects while using the same word “trace.” Adding more SHA steps does not resolve the recursion. | Freeze two distinct objects without parallel authority: (1) one storage-owned canonical semantic source-record byte stream, precommitted before challenges and recomputed in-circuit; (2) a deterministic, spec-digest-bound control schedule derived from those records. `TRACE_DIGEST` hashes source records only, never the SHA opcodes used to compute it. Nova constrains every control step, exact expansion counts, source offsets, and final source digest. Add a dependency-DAG test and reject any constructor/hash input edge from a control step back into the source commitment it computes. |
| DC2-F18 | `RecursiveCircuitProfileV2::new` accepts resource profiles that the source cannot instantiate. In particular `total_spool_bytes == max_content_bytes` passes the constructor, but `IdentifierPrecommitV2::new` then assigns zero bytes to each sorted-ID spool and rejects. The constructor also does not prove `ceil(max_set_ids/resident_ids) <= max_spool_runs <= merge_fan_in` or that each split sort budget covers its maximum set. | An authority-pinned profile can validate and digest successfully, then deterministically fail before proving. This violates the claim that profile validation is the complete preflight and makes resource failures input-shape dependent. | Derive exact per-spool byte/run/fan-in feasibility with checked arithmetic in the profile constructor. Either split bytes according to spent/output caps or require each half to cover its own maximum. Test zero/one/exact/cap+1 bytes, IDs per run, run count, unequal set caps, and every constructor-accepted profile through successful source creation. |
| DC2-F19 | `RecursiveAuthorityContextV2::new` is public and accepts caller-selected network/config/layout/generation; `RecursiveSnapshotHandleV2::from_store` accepts an arbitrary caller `PrepSnapshotId`. `CanonicalCheckpointTransitionV2::from_exec` checks only policy equality and rederives current store fields under that same arbitrary ID. It does not resolve the canonical contract configuration or a real immutable prep-snapshot record by ID. | “Authority-pinned” and “immutable snapshot” are currently caller assertions wrapped around live-store observations. A correct proof could later bind the wrong network/config/snapshot identity even if roots and hashes are internally consistent. | Make authority/snapshot constructors private to one resolver that loads the canonical contract config and actual prep-snapshot owner under a guarded generation. Verify ID, height, predecessor, content/count/root, policy/layout, and config generation at every pass; reject caller injection and config/snapshot rotation. |
| DC2-F20 | `EvaluatedCheckpointTransitionV2::statement_digest` currently binds authority/snapshot digests, post root, trace/list digests, and only total event/byte counts. It omits explicit height/predecessor, profile/spec/trace-grammar/verifier-bundle identities, prior/final backend roots, and per-class declared/consumed counts. `RecursiveCircuitSpecV2` exists but is not consumed by this statement path. | Distinct proof relations, heights, grammars, or work decompositions can share the same native statement surface. Nova cannot repair an application field that was never bound or independently compared. | Implement one typed statement builder after the corrected acyclic precommit/transition order. Bind every Section 13.1 field and exact per-class count, make profile/spec/grammar/bundle authority-selected, and add one-field-at-a-time substitution tests reaching the unchanged real verifier plus endpoint comparison. |
| DC2-F21 | The current SHA role registry is coarser than the theorem's semantic grammars. `Trace` is reused for structural-event IDs, JMT-envelope trace digest, and the whole source trace; `Content` is reused for profile and snapshot content/handle; `Statement` is reused for spec and transition statement; spent/output sequences share `OriginalIds`/`SortedIds` without an encoded set kind. Length framing prevents split/concat ambiguity, but it does not provide the claimed one-role/one-schema domain separation. | A cross-protocol substitution is no longer rejected by a distinct domain tag; safety falls back to statement position and SHA collision/second-preimage resistance. The work ledger also cannot identify which transcript a generic role count represents. | Give every theorem-distinct grammar a unique typed role/label or prepend a frozen schema tag and set kind that the circuit constructs identically. Generate a registry uniqueness report, frozen preimage vectors, cross-role substitution tests, and per-role work rows; forbid raw role reuse unless a proof documents identical semantics. |
| DC2-F22 | `JmtUpdateTraceV2::from_canon` runs `verify_native`, but `SettlementUpdateTraceEnvelopeV2::from_canon` deserializes nested update structs directly and checks only `envelope.canonical_bytes() == bytes`. `canonical_bytes` validates sizes/version/digest but never calls each update's `verify_native`. An attacker can therefore alter an update proof, recompute the envelope trace digest/canonical bytes, and pass envelope decode without a valid JMT update proof. | The public strict-envelope decoder does not establish the native-proof property its type/comment implies. No live Nova verifier consumes it yet, so this is not Model C, but using it as T1 evidence would create a verification bypass. | After bounded strict decode, call `verify_native` on every update and preferably frame/decode each update through its own strict constructor. Add a retained exploit regression that mutates proof/root/operation data, recomputes all noncryptographic envelope bindings, and proves unchanged `from_canon` rejects before any evaluator/circuit use. |
| DC2-F23 | `SettlementRootGenerationCutoverV2::repository_local_fixture` hashes the caller-provided opaque last-root record but has no authority-pinned expected record digest to compare. `install_repository_fixture` only rechecks the already-live store, flips an in-memory `installed` boolean, and returns the existing root; it performs no atomic persisted installation, CAS generation write, restart reload, or durable idempotence check. | A test can report “cutover installed” without changing or durably recording canonical state. Restart erases the only installation flag, and an arbitrary opaque record is self-consistently hashed rather than authenticated. | Keep the repository-local branch non-production, but make its cutover simulator use the real atomic storage primitive and a pinned content-addressed record. Persist generation/manifest/root in one transaction, fsync/reload, enforce CAS/idempotence, and test crash points plus wrong/replayed record digest. Never promote this fixture to production authority. |
| DC2-F24 | Private witness-bearing types are currently exposed too broadly: `RecursiveTraceEventV2`, `JmtUpdateOpV2`, `JmtUpdateTraceV2`, and the envelope derive `Debug`; the public `recursive_v2` facade reexports JMT update/value types; `CanonicalCheckpointTransitionV2::update_trace` returns the envelope; and `JmtUpdateOpV2::value` exposes exact new value bytes. No direct log sink was found, but the API contradicts the audit's “secret types omit Debug/public serialization” boundary. | Accidental debug/error logging, telemetry capture, crash reports, or downstream serialization can disclose replay/JMT witness values even if the compressed proof is zero knowledge. Backend ZK does not erase host-side API leakage. | Keep trace/value/prover-state owners crate-private and expose only typed public digests/counts/resource status. Remove or redact `Debug`, audit error/log paths and core-dump policy, zeroize bounded buffers where meaningful, and add compile-fail/public-API plus secret-canary log/debug tests. |

DC2-F05 was reproduced by the mandatory release bootstrap: the primary failure
was `Root node not found for version 0`; poisoned environment locks and snapshot
failures were consequences. The storage owner now falls back to the canonical
null root, and a later-version first-update regression is retained. This fixes
the native half of that T1 defect. `commit_snap_with_update_trace` also now
receives the exact tree role, and the canonical commit path retains the
produced traces in `SettlementUpdateTraceEnvelopeV2`, closing the native role/
drop defect in DC2-F06. T2 still does not decode and constrain those same bytes,
and restart/persistence parity does not exist, so the end-to-end finding is not
closed.

DC2-F11–F24 are source-proven implementation/design blockers, not claims of a
new cryptographic break. F14 and the raw-framing part of F16 have materially
improved in the current code, but their complete-work premises remain open as
stated in the ledger. The remaining findings fail canonical-statement,
authority, atomicity, feasibility, or circuit-completeness premises before Nova
is invoked. Because the live dependency graph contains no Nova verifier, they
do not satisfy Model C and MUST NOT be reported as a Nova/Poseidon exploit.

## 23. Corrected mathematical argument

### 23.1 Typed transition system

Let `S` be the finite product of all persistent and transient state cells in
Section 15, let `E` be the finite tagged event set, and let
`R_pp(S_i,E_i,S_(i+1))` be the one fixed R1CS relation under an
authority-selected parameter bundle `pp`. Define the following invariants:

- `I_shape`: every opcode synthesizes the same variables and constraints;
- `I_sel`: opcode selectors are boolean, exactly one is active, and inactive
  lanes have no effect on selected next state;
- `I_range`: every byte, limb, counter, ordinal, length, shift, carry, and
  selector is range constrained before arithmetic;
- `I_bind`: profile/spec/trace grammar/verifier bundle/context/height/
  predecessor/statement/count identities equal their authority-pinned values;
- `I_sha`: each streamed SHA state is the FIPS 180-4 compression state for the
  exact role-framed prefix and finalizes with the exact total bit length;
- `I_replay`: each replay event is the next ordered canonical row and derives,
  rather than accepts, its semantic state effect;
- `I_unique`: the two precommitted ID lists satisfy strict sorted uniqueness
  and both independent permutation checks, except with `epsilon_perm`;
- `I_jmt`: every update starts at the current typed tree root, authenticates the
  old path, derives the exact new leaf/path/root, and chains to the next update;
- `I_hierarchy`: terminal, bucket, serial, and definition roots are propagated
  through their exact typed parent-leaf encodings;
- `I_root`: the sole V2 settlement/check root is derived from generation,
  layout, policy, and final definition root through the frozen role framing;
- `I_count`: declared and consumed event/byte/row/update/hash counts agree and
  cannot wrap; and
- `I_done`: only `FINALIZE_BLOCK` can set `done`, and it requires every pending
  machine empty and every final statement field equal.

Base case: the authority-selected `z_0` satisfies the cutover invariants,
contains no active transient submachine, and binds the approved predecessor.

Inductive step: for every legal event, `R_pp` first enforces the applicable
prior invariants, then derives exactly one next-state candidate, and finally
selects it under `I_sel`. The opcode-specific lemmas in Section 16 preserve all
unmodified cells and establish the modified invariant. Any illegal event,
counter, phase, role, path, digest, or root makes at least one enforced equality
false.

Termination: `I_done` and `I_count` imply that an accepting final state has no
omitted, duplicated, reordered, padded, or trailing event. `I_root` and
`I_hierarchy` identify its final storage state; `I_bind` identifies the exact
application statement.

Therefore, assuming the circuit implements all listed constraints and A-01
through A-17, an R1CS-satisfying trace implies the canonical V2 checkpoint
transition except with the explicitly composed cryptographic error. The
converse is constructive: T1's deterministic evaluator emits the unique legal
event sequence for a valid canonical transition, and each event supplies a
satisfying witness. T4 MUST prove both directions per opcode; a one-way
“valid fixtures accept” test is insufficient.

### 23.2 Acyclic commitment order

The only allowed dependency order is:

```text
authority snapshot and prior finalized state
  -> immutable source precommit and exact counts
  -> replay rows and original/sorted ID-list precommits
  -> SHA-derived uniqueness challenges and constrained products
  -> canonical net effects and HJMT update trace
  -> final definition/settlement/delta/witness/journal roots
  -> transition statement
  -> checkpoint/link data
  -> recursive public input X_h
  -> expected final state z_h
  -> Nova proof and compressed snapshot
  -> strict verification and complete z_N comparison
  -> cryptographic receipt and persistence/publication
```

No node may hash a descendant. In particular, `U` excludes challenge products
and final roots; the transition statement excludes `X_h`, `z_h`, proof, and
receipt; `X_h` excludes `z_h`, proof, and receipt. A code-level dependency graph
or constructor accepting a later object at an earlier stage is a T4 failure.

### 23.3 Corrected concrete error ledger

For one compressed proof covering `N >= 1` logical steps, exact cached source
contains two ordinary NIFS challenges for each step after the base and three
additional challenges during compression:

```text
F_recursive(N)  = 2*(N-1)
F_compressed(N) = 2*N+1
```

Under the random-oracle and nonzero-degree premises, each degree-at-most-two
fold residual hits a 128-bit challenge with probability at most `2^-127`.
Hence the local per-proof ceiling is:

```text
epsilon_fold_one(N) <= (2*N+1) / 2^127.
```

For an attack game containing distinct challenged folds across proof attempts
`i`, use the safer accounting variable:

```text
Q_fold = number of distinct NIFS challenge invocations exposed in the game
epsilon_fold_total <= Q_fold / 2^127
                  <= sum_i(2*N_i+1) / 2^127.
```

This is a local root-counting ceiling, not a standalone Nova knowledge-
soundness theorem. The 2024 analysis identifies a gap in the original
recursive extractor for more than logarithmically many rounds and obtains a
polynomial-round result only in its refined algebraic-group model with extra
Fiat–Shamir/hash premises. The project must name the exact theorem/version and
show that the selected implementation satisfies its premises; otherwise
polynomial-depth soundness remains A-17, not a proved fact.

The complete responsibility bound is therefore:

```text
Adv_forge <= Adv_Nova/Spartan_exact_model
           + Adv_DLOG/Pedersen
           + Adv_Poseidon/Keccak/SHAKE_RO
           + Adv_SHA256_application
           + q_U*(15*n_max/2^248)^2
           + Q_fold/2^127
           + Pr[Bad_impl] + Pr[Bad_cutover] + Pr[Bad_authority].
```

Terms already included in an instantiated backend reduction must not be
double-counted in a numerical claim. `Bad_impl`, `Bad_cutover`, and
`Bad_authority` remain audit events without cryptographic probabilities. The
maximum defensible claim is conditional, classical, and no stronger than its
weakest instantiated term; no post-quantum or unconditional claim exists.

### 23.4 Robustness and liveness

Safety and liveness are separate:

- safety is the statement “accepted implies the typed transition,” conditional
  on Sections 23.1–23.3;
- memory safety requires all outer and nested lengths, point encodings, event
  counts, and work budgets to be checked before allocation or expensive work;
- crash consistency requires authenticated running-state persistence,
  re-verification after reload, atomic replace/fsync ordering, and quarantine of
  partial/stale state;
- deterministic concurrency requires the cryptographic state machine to remain
  sequential while bounded threads only prepare independent data whose ordered
  results are rechecked before absorption; and
- liveness additionally assumes available disk, entropy, CPU, memory, and a
  non-failing storage backend. A proof system cannot mathematically guarantee
  those environmental conditions.

For fixed circuit size `C`, state arity `A`, and pre-allocation resident cap
`b`, the streaming construction keeps matrix/setup/fold memory independent of
the 64 MiB content cap:

```text
M_peak <= max(M_setup(C,A), M_fold(C,A), M_compress(C,A), M_verify(C,A))
          + O(b).
```

This is an asymptotic design theorem, not a byte limit. T4 must measure every
term with the selected release binary and reject the configuration if measured
peak RSS/allocation/serialization exceeds its authority-pinned cap.

## 24. Nova and Poseidon dependency doublecheck

### 24.1 Exact source observations

The cached `nova-snark 0.73.0` source identified in Section 21 confirms:

- `NUM_CHALLENGE_BITS=128` and `NUM_HASH_BITS=250`;
- `RecursiveSNARK::new` executes the first primary circuit while `i=0`;
- the first `prove_step` only records that step as `i=1`;
- every later `prove_step` invokes secondary then primary `NIFS::prove`;
- `CompressedSNARK::prove` invokes one ordinary and two relaxed NIFS folds;
- compressed verification receives `num_steps` and `z_0`, verifies both
  Spartan-backed folded instances, and returns its own `z_N`; and
- the application, not Nova, must compare every returned endpoint limb and bind
  the authority-selected verifier bundle.

The [AFT 2023 two-cycle analysis](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2023.18)
documents the historical implementation vulnerability, its corrected
construction, and folding malleability. The
[2024 Nova security analysis](https://eprint.iacr.org/2024/232.pdf) makes the
depth/model limitation explicit. These papers support conditional reductions;
they do not turn a cached source tree into a verified live dependency.

### 24.2 Poseidon statement

The source-derived width/round/S-box/constants findings in Section 21 remain
accurate for the cached candidate: wide `(t=25, R_F=8, R_P=59)`, narrow
`(t=6, R_F=8, R_P=56)`, `x^5`, deterministic Grain constants, and generated
MDS matrices. The original
[Poseidon paper](https://eprint.iacr.org/2019/458) and later analyses provide
cryptanalytic evidence, not a proof that this concrete implementation is an
ideal random oracle.

`HashType::Sponge` contributes zero as its hash-type tag. The secure sponge API
places the fixed IO pattern's wrapping-`u128` accumulator in the capacity
element. For Nova's fixed shapes this is a deterministic transcript-format
commitment, but it MUST NOT be described as a collision-resistant domain
separator or reused as a project hash. Required tests enumerate every reached
pattern, mode, absorb count, squeeze count, native output, circuit output, and
source parameter digest; any native/circuit mismatch blocks T2.

No new critical Nova/Poseidon exploit was established. The historical
two-cycle attack does not apply to the audited corrected candidate source, and
no suspected new issue passed Models A, B, and C. Consequently, local vendoring
or a cryptographic fork is not justified. If a future candidate passes all
three models, the exploit fixture and three independent reproductions are
mandatory before the minimal owner-preserving patch process in Section 21.

## 25. Exact implementation and verification obligations

### 25.1 T1 — deterministic source and canonical trace

T1 MUST:

1. resolve one immutable authority snapshot and V2-only role registry;
2. derive the V2 settlement root with generation 2 fixed or release-checked;
3. consume the same immutable snapshot in both rewindable passes;
4. enforce per-field, per-object, aggregate-byte, operation, and work caps
   before decode/allocation;
5. emit the exact role, tree ID, old/new version, old/new root, operations,
   update proof, replay origin, ordinal, and hierarchy parent for every JMT
   transition;
6. treat a late-born tree's absent stored predecessor as the canonical null
   root without inventing a second root rule;
7. precommit original and sorted ID lists before deriving challenges;
8. derive the transition statement only after all semantic results exist; and
9. make the independent native evaluator derive the expected final state from
   typed inputs rather than accept a caller-supplied result;
10. stream events directly from the immutable canonical storage/planner owner,
    never from a public caller-owned event slice, and keep peak resident input
    bounded before the first allocation; and
11. precommit separate spent and output original/sorted ID lists, not the
    `object_id` field of every opcode, while enforcing the complete phase,
    nested-submachine, count, payload, EOF, and authority-defined no-op grammar;
    and
12. derive a checked worst-case event/step budget from every semantic cap and
    require the 64 MiB profile to cover the complete role-framed SHA work
    (at least 1,048,578 blocks for one full-cap trace part) plus
    begin/end, replay, uniqueness, JMT, hierarchy, commit, finalization, and any
    explicitly justified recovery events; and
13. separate atomic canonical admission from the non-authoritative proof worker:
    preflight deterministic trace bounds before commit, capture the exact trace
    in the same storage transaction, and return typed committed-root shadow
    outcomes for every post-admission resource/I/O failure;
14. distinguish source-record counts from derived control-step counts and bind
    the checked sum of every reached SHA transcript instance, not merely the
    largest role or the trace-role count;
15. reject every profile whose spool bytes, resident IDs, run count, or fan-in
    cannot realize its own maximum spent/output sets, and execute source
    creation for every constructor-accepted boundary profile;
16. resolve authority and prep-snapshot identity from the canonical storage and
    contract owners, never from public caller-created context/ID values; and
17. build one typed statement containing every Section 13.1 identity, root,
    height/predecessor, profile/spec/grammar/bundle digest, and exact per-class
    declared/consumed count; and
18. assign a unique typed SHA role/schema tag to every theorem-distinct
    transcript, including profile, spec, snapshot content/handle, source trace,
    JMT envelope, structural event, and spent/output list kinds; and
19. make strict envelope decode reverify every nested JMT update proof after
    canonical decoding, including adversarial inputs with recomputed envelope
    digests and encodings; and
20. replace the in-memory fixture “install” with a real atomic, persisted,
    content-addressed cutover simulation using the canonical storage primitive,
    CAS generation, fsync/reload, crash recovery, and idempotence; and
21. make all witness-bearing trace/JMT/prover types crate-private, remove or
    redact `Debug`, and expose only public commitments/counts/status required by
    the protocol.

Required release tests include every SHA boundary; independent framing vectors;
outer/nested/trailing decode failures; version wrap/gap; late-born JMT trees;
all insert/replace/delete/coalesce/null-root cases; exact tree-role mutation;
immutable-source TOCTOU; cap and cap+1; list grinding fixtures; generation mix;
builder/evaluator byte-for-byte determinism across clean processes; rejection
of the former arbitrary-root/arbitrary-payload fixture; proof that a public
caller-event slice cannot be constructed; separate spent/output duplicate and
same-path replacement cases; and valid/invalid authority-defined empty/no-op
transitions; plus a 64 MiB cross-cap proof that neither typed-event nor
cumulative-step limits are smaller than the fully derived worst-case trace;
and fault injection after commit/before events, spool create/write/rewind,
precommit, reload, retry, and restart proving no ambiguous `Err` can hide an
advanced canonical state. They also include exact role-summed SHA accounting;
profile-constructor-to-source feasibility for zero/exact/cap+1 bytes, IDs,
runs, and fan-in; caller authority/snapshot/config substitution and rotation;
and one-at-a-time mutation of every typed statement field.
They include a generated role/schema uniqueness report, frozen preimage vectors,
and cross-role/cross-set substitution attempts for every reached SHA transcript.
They include a retained nested-envelope bypass regression that mutates a JMT
proof/root/operation, recomputes the envelope digest, and still must reject.
They include cutover crash-point/restart/replay tests proving a persisted
generation/root/record transition, not an in-memory boolean flip.
They include public-API compile-fail scans and secret-canary debug/log/core-dump
tests covering replay payloads, JMT values, spools, and recovery state.

### 25.2 T2 — uniform circuit and pinned bundle

T2 MUST implement one fixed `StepCircuit` in the existing private storage
owner. Every opcode allocates the same shape; one-hot selectors gate algebraic
next-state selection, never host branching. The circuit independently
constrains framing SHA, replay, uniqueness, old/new JMT relations, hierarchy,
root derivation, counts, phases, and finalization. Native proof verification,
sorting, parsing, checksums, or booleans are not circuit witnesses of validity.

The pinned bundle MUST identify crate checksum/source, features, curves,
commitment engine, Poseidon constants/modes, Spartan engine, circuit shape,
state arity, profile, spec, trace grammar, caps, and project digest. Proof bytes
cannot select any component. Setup and real compressed proof/verification must
run in release mode for `N=1`, multi-opcode traces, maximum declared shape, and
mutation corpus. Shape, constraint, variable, public-input, serialized-key, and
peak-memory values are measured, not estimated.

### 25.3 T3 — continuous runner and strict acceptance

The sole private runner MUST call `RecursiveSNARK::new` exactly once at the V2
cutover, record event zero with the first `prove_step`, and thereafter continue
the same object. Checkpoint boundaries use `BEGIN_BLOCK/FINALIZE_BLOCK`; they do
not restart Nova. Compressed snapshots are non-consuming views.

Acceptance order is fixed:

1. select and pin the authority/verifier bundle before proof decode;
2. cap the envelope and every nested item before allocation/work;
3. strict-decode and canonical-reencode proof and statement bytes;
4. compare all profile/spec/context/height/predecessor/count/bundle identities;
5. run the unchanged real compressed verifier with exact cumulative `N` and
   cutover `z_0`;
6. compare every returned `z_N` limb to the independent evaluator;
7. rederive statement/checkpoint/link/receipt bindings;
8. persist atomically; then reload and repeat cryptographic verification before
   publication/admission.

Any error, panic, cancellation, rotation, stale generation, timeout, partial
write, or mismatched limb fails closed and emits no accepted receipt.

### 25.4 T4 — theorem-to-code proof and measurements

T4 MUST produce a bidirectional matrix for every invariant, assumption,
constructor, serialized field, opcode, constraint block, evaluator function,
positive test, negative test, fuzz/property target, and residual risk. It MUST
also retain actual sizes/times/resource profiles for parameters, keys, running
state, recovery state, trace/spool, proof, envelope, receipt, network payload,
and archive at minimum/representative/maximum/64 MiB fixtures.

The mandatory three-model gate remains exact:

- Model A: produce two different semantic executions/endpoints satisfying one
  bound statement or identify a failed reduction premise;
- Model B: reproduce a native/circuit/transcript/reference mismatch for the
  same retained typed bytes; and
- Model C: make the unchanged authority-pinned verifier repeatedly accept a
  proof whose exact project statement or endpoint is false.

Decoder rejection, checksum failure, native-only failure, OOM, panic, reduced
circuit, mock verifier, or receipt parser is not a cryptographic exploit. Every
candidate must pass A, B, and C before “critical vulnerability” or local fork is
permitted.

## 26. Final doublecheck verdict

| Layer | Result | Reason |
| --- | --- | --- |
| Statement logic | CONDITIONAL PASS | The acyclic typed transition and induction are consistent after the corrections above; no surviving statement counterexample was found. |
| Uniqueness argument | CONDITIONAL PASS | Both lists are precommitted; the nonzero bivariate polynomial and two independent challenge pairs give the stated bound under A-03/A-16. |
| HJMT/root argument | CONDITIONAL PASS | The abstract ordered old/new path reconstruction and hierarchy induction are sound under exact SHA/JMT encodings; the native owner now retains exact trace bytes and reads the store root, but the evaluator does not derive that root from those bytes or enforce the complete relation (DC2-F06, F07, F10–F13). Authority/snapshot and statement binding remain incomplete (F19–F20), and nested envelope decode omits native proof verification (F22). |
| Nova composition | CONDITIONAL/RISKY | The corrected candidate source fixes the historical two-cycle flaw, but the concrete depth/Fiat–Shamir model remains conditional and the workspace does not currently pin Nova. |
| Poseidon transcript | CONDITIONAL/RISKY | Parameters match the audited source and no concrete break was found; RO behavior and fixed-pattern separation remain assumptions, not proofs. |
| Host-side witness privacy | BLOCKED | Conditional proof ZK does not cover current public/debug trace and JMT value surfaces; F24 must close before any privacy claim. |
| Rust implementation | BLOCKED | T2/T3/T4 do not exist; partial T1 cannot establish circuit or verifier acceptance. |
| Operational robustness | BLOCKED | Full cap/decode/recovery/concurrency/resource evidence is not yet produced. The former F14/raw-F16 arithmetic examples are fixed, but complete source/control and role-summed work accounting remains unproved; F15 leaves post-commit failures ambiguous, F17 permits a future transcript cycle, F18 allows constructor-valid but unrealizable resource profiles, F21 shows the live role registry is not yet one-schema/one-role, and F23's cutover “install” is not durable. |
| Overall | RISKY BUT SALVAGEABLE | The streaming theorem is viable and avoids multi-gigabyte matrix growth, but Plan 06 remains forbidden until T1–T4 and all gates pass. |

The doublecheck therefore proves neither “everything works” nor an
unconditional 100% guarantee. It proves a sharper result: the proposed
streaming relation has a coherent conditional theorem, identifies its exact
cryptographic assumptions and finite-error accounting, and supplies a complete
implementation/falsification route. The remaining blocker is implementation
and evidence, not a need for a 64 MiB monolithic R1CS array.

## 27. Final source-bound doublecheck and validation evidence (2026-07-14)

This section controls the final review status of this audit. It does not weaken
Sections 22–26 and does not convert a planned constraint into implemented
evidence. The review was repeated from the current source tree after the
corrective ledger reached DC2-F24.

### 27.1 Exact reviewed snapshot

| Source | SHA-256 | Review meaning |
| --- | --- | --- |
| `Cargo.lock` | `56414379d3934b002c04218148065b13415ad51d5d14bbd10c547af7989aa5d7` | No live `nova-snark` or `neptune` package is present. |
| `checkpoint/recursive_circuit.rs` | `dd75a1663e390322081fe8279e6ea8f31c005a4f8ff32932c9679732817cc4b4` | Current profile/work-bound implementation. |
| `checkpoint/recursive_trace.rs` | `75646b68cf1c3649b11e43924ee542186255af86be386dc9c3163fe2f0bccc6c` | Current source-trace/precommit implementation. |
| `checkpoint/recursive_predicate.rs` | `1cf5e44de4fd466484e4bdbc7b2683a85385259bab2ca47b27ae6558d711508f` | Current native evaluator/statement surface. |
| `checkpoint/recursive_context.rs` | `ee843704069b0d2ad34af994c13e06475d53b41e6464b3163ecaeb39be0c745c` | Current caller-facing authority context. |
| `checkpoint/canonical_transition.rs` | `5a3f7bb6ae2e810a9bb292ffdfd4a6dd3acceba49d45ee40a379053040872d9f` | Current commit, cutover, trace, and evaluation order. |
| `settlement/proof_batch.rs` | `00d66ee5ecfd21a829243fca44f85e76c0766bdaff7fe0833707655892cf4b9f` | Current JMT trace/envelope wire and verification boundary. |

The cached `nova-snark 0.73.0` hashes in Section 21 identify only the candidate
source used to check schedule, transcript, and historical vulnerabilities. A
future T2 dependency addition changes the live cryptographic snapshot and
therefore invalidates any claim that this table attests the installed backend.

### 27.2 Recomputed mathematical checks

The final arithmetic pass independently recomputed, rather than copied, the
two concrete probabilistic ledgers:

| Input | Recomputed result | Consequence |
| --- | --- | --- |
| `N=1` | `F_compressed=3`, `lambda_fold=125` | Constructor/first-step/compression special case is not treated as one challenge. |
| `N=2` | `F_compressed=5`, `lambda_fold=124` | The first post-base step charges both ordinary folds. |
| `N=3` | `F_compressed=7`, `lambda_fold=124` | Exact source schedule remains `2*N+1`. |
| `N=2^20` | `F_compressed=2,097,153`, `lambda_fold=105` | A million-step proof cannot be advertised as 128-bit from the local root bound. |
| `n=2^32-1` | one permutation pair `>212.093` bits; two pairs `>424.186` bits | The degree-`15n` bivariate argument matches Lemma 16.6. |
| additionally `q_U<=2^128` | local two-pair term `>296.186` bits | Grinding is explicitly charged and remains subordinate to SHA/RO assumptions. |

Here `lambda_fold = 127-ceil(log2(2*N+1))`. These are local
Schwartz–Zippel/union-bound ceilings only. They do not prove knowledge
soundness, SHA-256, discrete-log hardness, a random oracle, or absence of an
implementation bug; those responsibilities remain in A-01–A-17 and Section
23.3. The uniqueness proof was rechecked in both directions: equality of the
two formal products implies equality of their factor multisets over
`F[beta]`; unequal byte-ID multisets therefore yield a nonzero polynomial of
total degree at most `15n`. The conclusion still requires both sequences to be
fixed before four independently domain-separated challenges, exactly as
required by A-16.

The dependency graph in Section 23.2 was also checked as a directed acyclic
graph. `TRACE_DIGEST` may commit only the immutable semantic source records;
it cannot include the `SHA_BLOCK` control records used to compute itself.
Challenges depend on precommits, final roots depend on constrained execution,
and proof/receipt bytes are descendants of the statement. Any reverse edge is
a theorem failure, not an implementation optimization.

### 27.3 Independent review convergence

The mandatory YOLO review was executed as five source-bound passes. “Clean”
below means no new unrecorded audit/plan defect was found; it does **not** mean
the blocked implementation is complete.

1. **Forward theorem pass — significant findings fixed in the audit.** It
   corrected logical-step versus NIFS-fold accounting, separated the local
   root bound from polynomial-depth knowledge soundness, and introduced A-17.
2. **Backward verifier-to-storage pass — significant findings fixed in the
   audit and executable plan.** It found that the current evaluator observes
   the post-store root rather than deriving it, the profile did not prove the
   complete sequential work/resource schedule, and authority/statement fields
   were caller-selected or omitted. These are DC2-F11 and DC2-F14–F20.
3. **Adversarial codec/operations/privacy pass — significant findings fixed in
   the audit and executable plan.** It found role/schema reuse, the nested JMT
   envelope verification bypass, non-durable cutover simulation, and public
   witness/debug surfaces. These are DC2-F21–F24. No candidate reached Model C
   because no live Nova verifier exists.
4. **First clean coverage pass.** It rederived the assumption/error ledgers,
   source identities, commitment order, exact endpoint comparison, Models
   A/B/C, and mapped all unique `E-001..E-055`, `A-01..A-17`, and
   `DC2-F01..DC2-F24` identifiers into `069-051-PLAN.md`; no missing family
   member or second recursive authority was found.
5. **Second consecutive clean coverage pass.** Starting at the acceptance
   endpoint and walking backward through bundle selection, bounded decode,
   unchanged verification, full `z_N` comparison, statement, trace, replay,
   JMT, storage admission, snapshot, and cutover found no new unrecorded
   premise. The conclusion remained BLOCKED at the same live-code boundaries;
   no checksum/native-only test was promoted into proof evidence.

Thus the review-convergence requirement is satisfied for the correctness and
coverage of this audit and its executable plan. It is intentionally **not**
satisfied for T1–T4 implementation completion: DC2-F11–F24, the absent circuit,
and the absent real verifier are still significant live-code issues.

### 27.4 Executed release evidence

| Gate | Result | What it proves and does not prove |
| --- | --- | --- |
| `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` | PASS, exit 0 | Mandatory early regression gate passed after the later-version null-root defect was corrected. |
| `cargo test --release` | PASS, exit 0 | The complete workspace release suite completed; explicitly ignored heavy/performance cases remain non-evidence. |
| `cargo test --release -p z00z_storage --test test_recursive_v2_trace -- --nocapture` | PASS, 3/3 | Current native trace binds one real HJMT commit and rejects the exercised stale/generation substitutions. It does not prove replay/HJMT in R1CS. |
| `cargo test --release -p z00z_storage --test test_recursive_v2_cutover -- --nocapture` | PASS, 3/3 | Current repository fixture rejects the tested root/snapshot substitutions and is exactly-once in memory. It does not prove durable atomic cutover; DC2-F23 remains. |
| exact contradictory-profile regression | PASS, 1/1 | The former 32,000/1,000,000 cap contradiction is retained as a rejection. It does not close the complete work-ledger proof. |
| `cargo build --release` | PASS, exit 0 | Workspace release compilation passed. Compile success is not runtime or theorem evidence. |
| release feature guard, crypto facade, RNG hygiene, secret-type hygiene scripts | PASS | Existing generic policy checks passed. In particular, their success does not close the newly identified recursive witness API exposure in DC2-F24. |
| `cargo fmt --all -- --check` | WARN/FAIL | Two pre-existing unrelated checkpoint test files require formatting. No audit conclusion depends on treating this hygiene failure as green, and unrelated user changes were not rewritten. |

Static gates also confirmed that the live recursive V1 profile/input/spec crate
path is absent, the V2 checkpoint owners do not call the rejected recursive
`hash_zk` fallback, and all 14 Section 16 proof units are present. Conversely,
`CheckpointNovaCircuitV2`, live `RecursiveSNARK`/`CompressedSNARK`, a pinned
`VerifierBundleV2`, and executable Model A/B/C verifier targets are absent.

### 27.5 Final proof status

The strongest defensible result is:

```text
if A-01..A-17 hold
and T1 emits the unique bounded canonical source/control trace
and T2 implements every I_shape..I_done constraint in one uniform circuit
and T3 verifies one continuous authority-pinned Nova execution
and T3 compares every returned z_N limb to the independent evaluator
and T4 closes every bidirectional theorem-to-code row and retained attack,
then accepted implies the canonical V2 checkpoint transition,
except for the explicitly composed finite cryptographic error in Section 23.3.
```

The antecedent is not yet true in live code. Therefore this audit confirms a
mathematically coherent, bounded-memory implementation route and rejects the
monolithic 64 MiB R1CS representation, but it cannot confirm cryptographic
operation, security, correctness, robustness, or network-size performance of
an implementation that does not yet contain the circuit/verifier. Plan 06
remains blocked until T1–T4 close every listed gate with retained release
evidence.

## 🔑 28. Live Semantic-Row Amendment (2026-07-18)

This amendment supersedes only the terminal-ID polynomial geometry in Sections
13, 16, 23, and 27. The earlier `enc32`/sixteen-limb/`15n` argument applied to
the old terminal-only row and must not be quoted for the live relation.

The canonical Original and Sorted element is now the complete 100-byte tuple:

```text
definition_id[32] || serial_id_le[4] || terminal_id[32] || leaf_value_hash[32]
```

It is encoded as fifty little-endian `u16` limbs. Therefore

```text
e_beta(x) = sum_(j=0)^49 limb_j * beta^j
deg(e_beta) <= 49
deg(D(alpha,beta)) <= 49n
epsilon_perm(n) <= (49n / 2^248)^2
epsilon_perm(n_max,q_U) <= q_U * (49n_max / 2^248)^2.
```

The nonzero-polynomial argument is unchanged: injective formal limb
polynomials identify complete semantic rows, the product is monic in `alpha`,
and unequal semantic-row multisets yield a nonzero difference polynomial. The
two independently domain-separated pairs and A-16 random-oracle assumption are
still required. Operational verifier admission `q_V` remains separate from the
theoretical `q_U` precommit-query budget.

Circuit-spec version 3 commits the live values `(50, 49, 248, 128)` for row
limbs, per-row degree, mapped challenge bits, and the conservative `log2(q_U)`
assumption. The repository fixture separately commits `n_max=16,000` for each
spent/output set through its profile. Symbolic integer-polynomial and
exhaustive toy-field tests are implementation evidence for the algebra and
wiring only; they do not prove A-16 or substitute observations for the bound.

The live circuit also consumes the complete row in both products and implements
semantic Delete/Insert/Replace/Unchanged/Close Net rows. This amendment does
not close JMT proof algebra, hierarchy/root induction, typed commitments,
`X_h`, prior IVC, final successor, Models A/B/C, final parameter artifacts, or
the authority operating budget.
