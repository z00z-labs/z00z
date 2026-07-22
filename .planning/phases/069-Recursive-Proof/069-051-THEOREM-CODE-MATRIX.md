# 069-051 theorem-to-code matrix

## Current T3/T4 evidence status (2026-07-22)

Status: **T3/T4 COMPLETE ON CURRENT SOURCE.**
T3 constrains the next block's predecessor checkpoint and role-framed
`RecursiveFinalizedIvcStateV2` digest inside one uninterrupted R1CS
accumulator. The active candidate identities are:

- source revision:
  `1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3`;
- circuit shape:
  `c0206283f9de5e4d75b007d0b05ea8491d8272665512a7ddd2f273229d16036e`;
- milestone worker:
  `5573f73e36922368b8179551b47b2b03a31bf88ff6b67b23552eccf099961cf5`;
- authority generation / role-framed bundle pin:
  `2` / `d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff`.

The current format-4 authority bundle omits deterministic Pedersen key vectors
from its private compact wire and reconstructs them from pinned labels plus
exact authority-bound counts. The strict artifact corpus passed 3/3, including
selected-digest grammar/layout mutations, and its source-binding gate passed.
The current public-ingress/storage-reload chain passed from one `z_0` at
`316/948/1580` cumulative steps. Block 1 is fold-only; explicitly requested
non-consuming snapshots at blocks 3 and 5 each emitted a `346,907 B` framed
envelope (`346,859 B` fixed payload plus `48 B` registry preheader). It
completed in `1,634.91 s` test time with no restart or V1 path. The current
1,727-step proof/Model-C harness also passed: the bounded worker completed in
`2,526.747 s` at `8,111,263,744 B` peak RSS, the isolated clean verifier
completed in `58.343 s` at `3,348,504,576 B` kernel peak HWM,
and the independently recomputed Model-C proof reached the unchanged verifier
before the target all-limb comparator rejected it. The artifact corpus and
public T3 chain cover the installed `1..=5` role pin directly.

## Superseded generation-1 T2 closure record

Status: **T2 COMPLETE** on source revision
`e58e2f9a2f715a64b37dd464248b57601e7deda4254086c0b6598160cf30dbd6`;
T3 is unlocked as the next task and is not implemented by this closure.
This is the final source digest; the matrix retains the corresponding mutation
evidence rather than inferring coverage from proof success.

Canonical owner: private `z00z_storage::checkpoint::nova` in
`crates/z00z_storage/src/checkpoint/nova.rs`. The former nested owner has no
module declaration, shim, re-export, cache identity, or test selector.

| Theorem obligation | Live code owner | Retained evidence | Disposition |
| --- | --- | --- | --- |
| One canonical source byte stream | `CanonicalCheckpointTransitionV2`, `RecursiveTransitionTraceSourceV2` | `test_source_chunks_match_encoder` | implemented |
| SourceMemoryWrite → TraceChunk direct equality | `CheckpointNovaCircuitV2::synthesize` source-memory lane | `test_source_window_binding`, zero-tail/cursor mutations | implemented |
| Source/global FIPS SHA relation | `synthesize_sha_compression_lane` and two byte contexts | fixed FIPS schedule, padding, binding and order mutation tests | implemented |
| Full original/sorted uniqueness | uniqueness parsers, list hashes, two product pairs, strict lex order | sorted-row/version/order/cardinality/product tests | implemented |
| SHA-derived P/U/challenges | uniqueness transcript state and SHA lane | precommit/challenge byte-binding tests and toy-field polynomial tests | implemented |
| Semantic Net effect and unchanged leaf | `synthesize_net_merge_payload` | kind/path/value/pending/Close mutations | implemented |
| Exact Net→terminal-JMT permutation | Net/JMT product and count cells | `test_net_mutations_map_jmt` | implemented |
| All six JMT update cases | `synthesize_jmt_hierarchy_payload` | native cases plus authenticated transcript mutations | implemented |
| Hierarchy induction and SettlementV2 roots | hierarchy child/parent machine and root SHA jobs | hierarchy, promoted-root, policy/layout/definition mutation tests | implemented |
| Four typed checkpoint commitments | typed commitment progress/digest cells | `test_checkpoint_commitments_bind_fields` | implemented |
| Statement, X_h and prior IVC binding | `RecursiveCheckpointPublicInputV2`, anchor/final cells | statement construction and final-successor mutations | implemented |
| Unique final successor | `expected_public_state`, `RecursiveFinalizedIvcStateV2::expected_successor` | final opcode-count and endpoint mutations | implemented |
| Fixed shape and total control relation | `CONTROL_TRANSITION_TABLE_V2`, `circuit_shape_digest` | every-opcode shape and independent frozen-edge enumeration | implemented |
| Split prover/verifier authority | `NovaProverMaterialV2`, `NovaVerifierBundleV2`, `validate_pinned_verifier_key_wire`, `NovaProofEnvelopeV2` | generation-1 strict 858,785,714-byte PP+PK recovery roundtrip/mutations; 47,008,185-byte authority-selected VK bundle; structural Pedersen/IPA identity/default/swap corpus; clean VK-only child; activation-bound keyless envelope; source-binding worker | implemented and passed in bounded release mode after the execution-marker correction; superseded by the generation-2 values above |
| Bounded segmented HJMT source | `CanonicalHjmtSegmentWriterV2`, `CanonicalHjmtSegmentReaderV2`, traced commit/store path | exact-cap, missing/duplicate/reorder/trailing/context/digest mutations; resident oracle equivalence | implemented; production is segment-only and retains no full traced result vector |
| Incremental hierarchy verification | `CanonicalProofBatchHierarchyVerifierV2` and streaming decoder | exact-consumption tests, per-operation verification/drop, child-level release at barriers | implemented and release-tested |
| PP/PK cache and recovery identity | private setup-cache owner plus sealed segment context | strict identity/cap/canonical re-encode, corruption/symlink/mix/rollback regeneration, deterministic replay | implemented; one 1 GiB private cache entry, no proof/verdict/VK-distribution state |
| Complete mixed compressed proof | `test_nova_checkpoint_proves_relation` plus `test_mixed_fixture_satisfies_testcs` | independent evaluator/TestCS evidence plus 1,727 RecursiveSNARK steps → compressed VK-only verify → exact endpoints → independently recomputed Model C | generation-1 proof/Model C passed in 2,317.436 s; clean verification was 29.340 s with 3,056,861,184 B kernel peak VmHWM; superseded by the generation-2 values above |
| A-17 polynomial-depth applicability | `069-051-A17-APPLICABILITY.md`, `069-051-A17-RESIDUAL-ACCEPTANCE-PACKET.md`, ePrint 2024/232 revision 2026-02-13 Theorem 5 | exact EAGM/GZT/Pallas-Vesta-DL comparison plus separate compression premise | conditionally accepted under decision `phase-069-t2-interactive-authority-2026-07-20`; no unconditional 128-bit cumulative claim |
| Authority operating envelope and selected candidate | active generation-1 numeric authority tuple | finite segment/thread/prover/width candidates, measured roots/digests/RSS, `q_V=1,048,576`, `N=4,294,967,296` | accepted: 1 MiB segment, one HJMT worker, 2 MiB result admission, separate 64 MiB input/snapshot reservation, one prover, `k=1`, 1 GiB cache, deterministic replay |
| T1 durable failpoint/secret/work ledgers | T1/DC2 evidence packet | F12 exact streaming equation/RSS; F23 five-seam crash/reopen; F24 six-outcome plus segment/cache secret corpus | F12/F23/F24 accepted; dependency-owned hard-kill zeroization remains an explicit non-claim |

## Conditional theorem statement

Under the explicit A-01..A-17 assumptions, including the conditionally accepted
EAGM/GZT, Pallas/Vesta discrete-log and separate compression premises, the
strict artifact loaders bind the selected parameters, the independent evaluator
and complete mixed real proof agree on every state limb, and the selected tuple
satisfies its active numeric operating record. Verifier acceptance therefore
implies the unique canonical V2 checkpoint successor for this frozen identity.

## Live T4 matrix vocabulary

Every row below uses the same frozen live boundary unless it explicitly states
that it is an offline theorem or a residual assumption:

- `GEN2` means authority generation `2`, authority digest
  `8ae07172f268f67bf4d5d2b4b11562f6625d9b18e269741ce6d018fb01a4661c`,
  and verifier-bundle role pin
  `d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff`;
- `INGRESS` is
  `RecursiveCheckpointEvidenceStoreV2::produce(RecursiveCheckpointChainBlockV2)`;
- `G01..G16` are the exact ordered `LiveGateIdV2` values
  `AuthorityResolved`, `FamilyCapSelected`, `OuterDecodeBounded`,
  `InnerDecodeBounded`, `CanonicalCurveValid`, `BundleMatched`,
  `BackendVerified`, `FinalStateLimbsMatched`,
  `StatementLinkPredecessorMatched`, `StorageEndpointReloaded`,
  `PrewriteComplete`, `AtomicWrite`, `ByteReload`,
  `PostwriteBackendVerified`, `PostwriteEndpointMatched`, and
  `ReceiptIssued`;
- `RELOAD` is the `ByteReload` bounded read followed by
  `PostwriteBackendVerified` and `PostwriteEndpointMatched` in `produce`;
- `MODEL-C` is
  `test_nova_checkpoint_proves_relation`, which
  independently recomputes a minimally changed proof candidate and sends it to
  the unchanged compressed verifier and exact endpoint comparison;
- `SEMANTIC-36` is the guarded list of 36 exact ignored tests in
  `nova_milestone_tests.sh semantic`; `TESTCS-1727` is
  `test_mixed_fixture_satisfies_testcs`.

No row treats `SEMANTIC-36`, `TESTCS-1727`, a scanner, or runtime evidence as a
proof of a cryptographic assumption.

## A-01 through A-17 assumption ledger

| ID | Exact code/constraint owner | Positive and minimally changed negative evidence | Live boundary / failure | Closure class | Residual condition |
| --- | --- | --- | --- | --- | --- |
| A-01 | `RecursiveAuthoritySnapshotV2`; `RecursiveCheckpointChainBlockV2::new`; `live_gate_context_digest` | `test_real_chain_public_receipt`; authority generation/digest substitution in `test_receipt_failpoints_keep_prefix` | GEN2; INGRESS; G01; `AuthorityMismatch`; RELOAD | differential/runtime evidence | Storage/config authority supplies the intended canonical statement and limits. |
| A-02 | `SettlementRootGenerationCutoverV2`; `CheckpointVersionRegistryV2` | `test_cutover_is_exactly_once`; `test_cutover_rejects_state_substitution`, `test_cutover_rejects_unpinned_record` | GEN2; cutover before INGRESS; `CutoverMismatch`; G10 reloads canonical endpoint | differential/runtime evidence + residual assumption | The pinned cutover record identifies the intended pre-cutover state. |
| A-03 | `z00z_crypto::sha256_256`; `synthesize_sha_compression_lane` | frozen SHA/FIPS vectors and `test_hash_controls_bind_fips`; byte/state/padding mutations in `test_hash_controls_reject_mutations` | GEN2; G07/G14; `ConstraintUnsatisfied`; RELOAD | reduction to named assumption + differential/runtime evidence | SHA-256 collision and second-preimage resistance under the total query budget. |
| A-04 | `sha256_256` domain/label/length grammar; `HashJobKindV2` registry | `test_hash_registry_is_injective`; domain/label/part-order/length mutations in `test_hash_controls_reject_mutations` | GEN2; G07/G14; `BindingMismatch`; RELOAD | algebraic proof + exhaustive finite-state check | Depends only on A-03 after prefix-free grammar injectivity. |
| A-05 | pinned Neptune/Poseidon constants and `IOPattern`; Nova transcript use in `nova.rs` | `test_nova_poseidon_wires_pinned`; pattern/domain/order mutations in `test_nova_dependency_transcript_pinned` | GEN2; G06/G07/G14; `VerifierRejected`; RELOAD | reduction to named assumption + differential/runtime evidence | Concrete Poseidon invocations behave as random oracles with the pinned 128-bit truncation. |
| A-06 | pinned Nova Keccak transcript and SHAKE256 Pedersen generator derivation | `test_nova_keccak_transcript_pinned`, `test_nova_pasta_identity_pinned`; source/constant mutation guard | GEN2; G05..G07/G14; `VerifierRejected`; RELOAD | reduction to named assumption + differential/runtime evidence | Keccak/SHAKE random-oracle behavior and unknown generator discrete-log relations. |
| A-07 | Pallas/Vesta cycle and compressed IPA verifier in pinned `nova-snark 0.73.0` | `test_nova_pasta_identity_pinned`; identity/default/swapped-point corpus in `test_prover_material_rejects_substitution` | GEN2; G05/G07/G14; `CanonicalCurveEncoding`; RELOAD | reduction to named assumption | Discrete-log hardness in the selected prime-order groups. |
| A-08 | `CompressedSNARK::verify`; `NovaVerifierBundleV2`; source and Cargo.lock pins | valid corpus `test_nova_bundle_verifies_proof`; proof/key/source mutations plus MODEL-C | GEN2; G06/G07/G14; `VerifierRejected`; RELOAD | reduction to named assumption + differential/runtime evidence | Correctness and knowledge-soundness reductions apply to the exact pinned Nova source and feature set. |
| A-09 | compressed Nova/Spartan proof path; private running-state and witness buffers | valid proof plus `test_secret_process_outcomes`; canary/log/debug mutations | GEN2; G07/G14; no public witness field; RELOAD | residual assumption + differential/runtime evidence | Conditional zero knowledge of the pinned construction; tests establish only host-side non-disclosure, not ZK. |
| A-10 | production `OsRng` sites in pinned Nova source; feature/source guard | `test_nova_dependency_transcript_pinned`; deterministic/repeated/test-RNG reachability mutation rejects | GEN2; setup/prove before INGRESS; source mismatch blocks G06 | residual assumption + differential/runtime evidence | OS entropy is unpredictable, non-repeated, and not attacker controlled. |
| A-11 | `encode_canonical_source_record`; strict V2 codecs; native evaluator and R1CS byte windows | `test_source_chunks_match_encoder`, `TESTCS-1727`; canonical-byte/trailing/version/source-window mutations | GEN2; G03..G05/G07/G14; `NonCanonicalEncoding`; RELOAD | exhaustive finite-state check + differential/runtime evidence | Canonical encoders and independent native/circuit consumers cover every registered grammar. |
| A-12 | `CheckpointNovaCircuitV2::synthesize`; `CONTROL_TRANSITION_TABLE_V2`; inactive-cell zero constraints | `test_opcodes_use_fixed_shape`, `test_control_machine_matches_matrix`; non-boolean done, inactive, opcode and witness-shape mutations | GEN2; G07/G14; `ConstraintUnsatisfied`; RELOAD | exhaustive finite-state check + differential/runtime evidence | Constraint implementation is complete for the frozen circuit source identity. |
| A-13 | `validate_pinned_verifier_key_wire`; `VerifierBundleBindingV2`; `authority_artifacts.rs` | active GEN2 artifact corpus; key/payload/length/generation/profile/spec/grammar/source substitution corpus | GEN2; G01/G06; `BundleMismatch`; RELOAD | differential/runtime evidence + residual assumption | Governance distributes the intended authority bundle and preserves its private prover material. |
| A-14 | checked profile/spec constructors; `nova_resource_preflight`; bounded decoders | exact-cap controls; `test_nova_preflight_rejects_caps`, checked-overflow test | GEN2; G02..G04; `ResourceLimit`/`ArithmeticOverflow`; no write | algebraic proof + exhaustive finite-state check | Rust checked arithmetic and declared integer widths match the frozen wire grammar. |
| A-15 | `LiveGateStageV2` typestate; `persist_content_addressed`; `revalidate_or_quarantine` | `test_real_chain_public_receipt`; all receipt/write/reload failpoints in `test_receipt_failpoints_keep_prefix` | GEN2; G07..G16; stable failure prefix; RELOAD mandatory before G16 | exhaustive finite-state check + differential/runtime evidence | Filesystem durability/atomic-rename semantics of the deployment platform. |
| A-16 | `IdentifierPrecommitV2`; `UniquenessTranscriptV2`; four role-separated SHA challenges | delete-only and ordinary valid controls; precommit/cardinality/order/challenge-byte mutations in SEMANTIC-36 and symbolic small-field checker | GEN2; G07/G14; `ConstraintUnsatisfied`; RELOAD | algebraic proof + reduction to named assumption + differential/runtime evidence | Conditioned on the immutable precommit, four SHA outputs are independent RO samples; grinding is charged as `q_U`. |
| A-17 | uninterrupted `RecursiveSNARK::prove_step` chain; `CompressedSNARK::prove/verify`; A17 acceptance packet | 1/3/5 chain and MODEL-C; predecessor/step-count/final-limb/proof mutations | GEN2; G07..G10/G14/G15; `VerifierRejected`/`EndpointMismatch`; RELOAD | reduction to named assumption + differential/runtime evidence | Named polynomial-depth Nova theorem applies with its EAGM/GZT, algebraic-group, FS/hash/query/depth, cycle-DL, and implementation premises. |

## Lemmas 16.1 through 16.14

| Lemma | Exact symbols and constrained relation | Positive / negative evidence | Real verifier and live gate | Closure class | Residual assumption |
| --- | --- | --- | --- | --- | --- |
| 16.1 injective integer/digest encoding | fixed-width LE encoders, `RecursiveCheckpointPublicInputV2`, canonical field-limb decoders | boundary vectors `0`, `2^56-1`, `p-1`; width, high-bit, legacy-`p`, limb and trailing mutations | MODEL-C; GEN2 G05/G07/G08/G14/G15; RELOAD | algebraic proof + exhaustive finite-state check | A-11/A-14. |
| 16.2 prefix-free role-bound `sha256_256` | `sha256_256`; `HashJobKindV2`; exact DST/label/part-length grammar | `test_hash_registry_is_injective`; domain/label/part count/order/length cross-role mutations | MODEL-C where statement changes; GEN2 G07/G14; RELOAD | algebraic proof + reduction to A-03 | A-03 only after grammar injectivity. |
| 16.3 streamed SHA/native equivalence | `synthesize_sha_compression_lane`, source/global SHA contexts, independent native `sha256_256` | FIPS/frozen vectors and `test_trace_chunk_binds_contexts`; state/padding/block/order mutations | MODEL-C and SEMANTIC-36; G07/G14; RELOAD | algebraic proof + differential/runtime evidence | A-03 for collision resistance, not for equivalence. |
| 16.4 typed-leaf/value-hash binding | typed replay decoders, `synthesize_jmt_hierarchy_payload`, value-hash and path cells | `test_replay_terminal_binds_object`, typed commitment control; kind/path/value mutations | MODEL-C; G07/G08/G14/G15; RELOAD | algebraic proof + differential/runtime evidence | A-03/A-11. |
| 16.5 one-key JMT update soundness | six-case JMT machine, sibling order/prefix/coalesce/root constraints | native controls `test_jmt_machine_accepts_mutations`; authenticated proof/root/key/sibling/order mutations reject | MODEL-C; G07/G08/G14/G15; RELOAD | algebraic proof + differential/runtime evidence | A-03 and pinned JMT semantics. |
| 16.6 precommitted uniqueness/permutation | `IdentifierPrecommitV2`, external sorted runs, strict ordering, two independent product pairs and count equality | delete-only/replace controls; duplicate/order/version/count/precommit/challenge/product mutations | MODEL-C; G07/G14; RELOAD | algebraic proof + reduction to A-16 + differential/runtime evidence | `epsilon_perm(n_max,q_U)` and A-16; no substitution of `q_V`. |
| 16.7 replay/net-effect equivalence | replay input/output automaton, `synthesize_net_merge_payload`, terminal Net→JMT permutation | `test_net_merge_streams_source`; kind/path/value/pending/Close/cardinality mutations | MODEL-C; G07/G08/G14/G15; RELOAD | algebraic proof + exhaustive finite-state check + differential/runtime evidence | A-11 and A-03. |
| 16.8 hierarchical-HJMT induction | `CanonicalProofBatchHierarchyVerifierV2`, JMT child/parent machine, SettlementV2 root SHA jobs | hierarchy/policy/layout/definition valid control; role/parent/value/promoted-root/order mutations | MODEL-C; G07/G08/G10/G14/G15; RELOAD | algebraic proof + differential/runtime evidence | A-03 and native JMT proof validity. |
| 16.9 complete unique finalization | `expected_public_state`, done boolean, exact counts/cursors, terminal `TraceEnd` edge | `test_successor_erases_cursors`; early/double finalize, count, trailing, nonzero transient and done mutations | MODEL-C; G07/G08/G14/G15; RELOAD | exhaustive finite-state check + differential/runtime evidence | A-12. |
| 16.10 Nova sequence/endpoint binding | continuous `RecursiveNovaRunningStateV2`; predecessor digest; cumulative `num_steps`; every returned `z_N` limb | T3 1/3/5 chain; `test_continuous_blocks_share_state`; restart/skip/reorder/predecessor/final-limb mutations | MODEL-C; G07..G10/G14/G15; RELOAD | reduction to A-17 + differential/runtime evidence | A-05..A-08/A-17. |
| 16.11 conditional composition/advantage ledger | executable ledger for `epsilon_perm`, `F_compressed(N)=2N+1`, `epsilon_fold_total=Q_fold/2^127` | symbolic/small-domain controls; overflow/omitted budget/`q_U=q_V`/unconditional-128-bit report mutations | no single runtime gate proves the bound; G07/G14 supply backend events | algebraic proof + reduction to named assumptions + residual assumption | `Adv_backend`, A-03/A-05..A-10/A-16/A-17 and unquantified `Bad_*`. |
| 16.12 bounded-memory streaming | `RecursiveTransitionTraceSourceV2`, private spool runs, `CanonicalHjmtSegmentReaderV2`, preflight | exact/max/cap+1 source and segment controls; missing/duplicate/reorder/trailing/run/fan-in mutations; measured RSS | GEN2 G02..G04 before setup and G07/G14 thereafter; RELOAD | algebraic resource accounting + differential/runtime evidence | Allocator/dependency peak allocation is measured, not mathematically excluded. |
| 16.13 side-channel claim boundary | private witness/prover state, zeroizing buffers, no public `Debug`/serde path | secret-canary process corpus; debug/log/telemetry/artifact canary mutations | not an acceptance theorem; INGRESS emits only public digests/counters, G16 receipt | differential/runtime evidence + residual assumption | No constant-time or ZK claim for host proving; OS/hardware/process side channels remain. |
| 16.14 liveness/throughput separation | `nova_resource_preflight`, queue/work caps, cancellation and snapshot request path | successful supported-cap runs; timeout/cancel/queue/cap+1 fail closed without receipt | G02 preflight or bounded failure prefix; supported inputs continue through G16/RELOAD | differential/runtime evidence + residual assumption | Scheduling, throughput, and availability do not strengthen soundness. |

## E-001 through E-055 correction closure

Each correction is carried through GEN2. `G07/G14 + RELOAD` means both the
initial and postwrite unchanged verifier see the same relation and the exact
endpoint is compared after bounded byte reload; pre-verifier format/resource
rows instead name their deepest gate and do not claim Model-C closure.

| ID | Corrected live owner / relation | Positive and minimally changed negative evidence | Deepest gate / stable failure | Closure class and residual |
| --- | --- | --- | --- | --- |
| E-001 | `RecursiveCircuitProfileV2` separates semantic caps, source records, control steps, and host resources | frozen profile control; cap+1/overflow tests in `nova_resource_preflight_*` | G02; `ResourceLimit` | algebraic accounting + runtime evidence; allocator peak remains measured |
| E-002 | `CheckpointNovaCircuitV2::synthesize` allocates one witness-independent shape | `test_opcodes_use_fixed_shape`; opcode/inactive/witness-shape mutations | G07/G14; `VerifierRejected` | exhaustive finite-state + runtime evidence; A-12 |
| E-003 | `RecursiveNovaRunningStateV2` performs continuous recursion; snapshots are non-consuming views | 1/3/5 T3 chain; restart/step-count mutation | G07..G10/G14/G15; endpoint mismatch | reduction to A-17 + runtime evidence |
| E-004 | `nova_resource_preflight` reports a typed unsupported capacity before setup | supported control; cap+1, overflow and bounded-worker failure corpus | G02; `ResourceLimit` and no write | runtime evidence; no universal feasibility claim |
| E-005 | authoritative shape/profile/source identities are pinned, not inferred from an old failure | GEN2 clean ShapeCS and artifact regeneration; any identity drift blocks guard | G01/G06; `BundleMismatch` | runtime evidence; GEN2 is the only active tuple |
| E-006 | `CheckpointTransitionConsistencyV2::evaluate_stream` is the independent native evaluator | real native commit control; `evaluator_rejects_*` trace/store substitutions | before proof and G07; `TransitionMismatch` | differential/runtime evidence; A-11 |
| E-007 | `JmtUpdateTraceV2::verify_native` verifies an update from declared old root to new root | six JMT case controls; old/new/proof/key mutation | G07/G14; constraint/verifier rejection | algebraic + runtime evidence; pinned JMT semantics |
| E-008 | `RecursiveCircuitProfileV2`, `RecursiveCircuitSpecV2`, and object-cap authority are distinct digests | exact GEN2 bindings; profile/spec/cap substitution corpus | G02 or G06; `ResourceLimit`/`BundleMismatch` | runtime evidence; authority supplies intended tuple |
| E-009 | fixed-width canonical bytes are constrained before digest-to-field conversion | boundary controls; high-bit, width, modulus and limb mutation | G05/G07/G08/G14 | algebraic + exhaustive finite-state; A-11/A-14 |
| E-010 | MODEL-C is an independent recomputation sent to the unchanged verifier and endpoint check | valid proof; minimally changed recomputed candidate rejects | G07/G08 and G14/G15 | differential/runtime evidence; no self-oracle claim |
| E-011 | `NovaProverMaterialV2`, private running/recovery state, verifier bundle, envelope and receipt are separate types | size/cap corpus; PP/PK-in-wire/public-type scans | G03/G04/G06; noncanonical/bundle mismatch | exhaustive type/wire + runtime evidence; host privacy remains A-09 boundary |
| E-012 | 64 MiB is a bounded input domain and preflight contract, not a promise that every host can prove it | supported-cap control and typed preflight non-success | G02 and no receipt | runtime evidence; capacity ledger distinguishes proved from fail-closed |
| E-013 | `JmtUpdateTraceV2` names and checks old-root→new-root transition semantics | native and circuit valid cases; swapped root/direction mutation | G07/G14 | algebraic + runtime evidence; A-03 |
| E-014 | full transition validity is owned by evaluator + circuit, never a native JMT boolean | valid mixed transition; replay/net/hierarchy substitutions | G07/G08/G14/G15 | algebraic + runtime evidence; A-11/A-12 |
| E-015 | `repository_local_fixture` remains non-production cutover evidence, not a chain invariant | fixture and live-cutover controls; wrong fixture/record/generation reject | pre-INGRESS cutover gate | runtime evidence; production authority is A-02 |
| E-016 | profile/spec constructors and grammar counts bound every semantic and control domain | exact-max controls; count/opcode/source/EOF cap+1 mutations | G02/G07/G14 | algebraic accounting + exhaustive finite-state; A-14 |
| E-017 | one fold is required per accepted block while a block contains a fixed counted micro-step schedule | T3 1/3/5 folds; zero/two fold cadence and step-count mutation | G07..G10/G14/G15 | runtime evidence + reduction to A-17 |
| E-018 | `circuit_shape_digest` and spec/grammar/source digests commit executable semantics | exact identity guards; source/spec/grammar/shape substitution | G01/G06 | runtime evidence; source review remains required on drift |
| E-019 | V2 constructors are canonical; recursive V1/compatibility declarations and paths are absent | owner/public-symbol/canonical-path guards; reintroduced name fails scan | before build/INGRESS | exhaustive source/reachability evidence; no decoder fallback |
| E-020 | `RecursiveFinalizedIvcStateV2::expected_successor` binds the storage predecessor and prior endpoint | continuous-chain control; predecessor/root/link/height mutations | G08..G10/G15 | algebraic + runtime evidence; A-15/A-17 |
| E-021 | project content/JMT/statement roles use SHA-256; Nova Poseidon remains private to its transcript | role registry and source guard; attempted project Poseidon/IO-pattern role fails scan | G06/G07 | source + runtime evidence; A-03/A-05 |
| E-022 | `RootGeneration::SettlementV2` is a declared new generation with a cutover registry entry | V2 derivation/cutover control; legacy/mixed generation rejects | cutover or G01 | exhaustive registry + runtime evidence; A-02 |
| E-023 | the live proof corpus uses the exact production circuit and compressed verifier, not a trivial library smoke circuit | real GEN2 proof and T3 chain; smoke/alternate-owner scan and MODEL-C | G07/G14 | differential/runtime evidence; A-08/A-17 |
| E-024 | `RecursiveNovaRunningStateV2` is consumed and advanced without per-block restart | 1/3/5 continuous chain; reordered/skipped/restarted predecessor mutation | G07..G10/G14/G15 | runtime evidence + reduction to A-17 |
| E-025 | `CryptographicVerificationReceiptV2` is privately issued only after G15 | `test_receipt_gate_is_last`, public receipt control; failpoint/replay/deserialize-constructor mutations | G16 or exact earlier prefix | exhaustive typestate + runtime evidence; A-15 |
| E-026 | first block initializes once, subsequent blocks use exactly the previous finalized state and cumulative step count | 1/3/5 snapshots and `test_continuous_blocks_share_state`; off-by-one/first-step mutations | G08/G09/G15 | algebraic + runtime evidence; A-17 |
| E-027 | all selectors/lanes are allocated for every witness; inactive cells are zero constrained | fixed ShapeCS control; every-opcode/nonboolean/inactive/source-stage mutations | G07/G14 | exhaustive finite-state + runtime evidence; A-12 |
| E-028 | typed leaf/JMT/state fields are constrained independently of opaque proof-byte identity | typed commitments and replay controls; leaf field/value/path/type mutations | G07/G08/G14/G15 | algebraic + runtime evidence; A-03/A-11 |
| E-029 | `synthesize_sha_compression_lane` carries explicit FIPS state block-by-block | streamed/native controls; intermediate state/block/padding mutation | G07/G14 | algebraic + differential evidence; A-03 |
| E-030 | role-specific SHA block geometry charges every compression block, not one block per event | exact geometry/shape metrics control; block-count/padding/role mutations | G02/G07 | algebraic accounting + runtime evidence; A-14 |
| E-031 | immutable precommit, private external sorting, strict order, counts, and two product pairs form one uniqueness relation | valid spent/output/delete/replace controls; duplicate/order/count/product mutations | G07/G14 | algebraic + reduction to A-16 + runtime evidence |
| E-032 | every ordered collection uses a typed prefix-free SHA commitment with count/order semantics | valid transcript registry; reorder/count/domain/label mutations | G07/G14 | algebraic + reduction to A-03/A-04 |
| E-033 | resource feasibility, liveness, and soundness are separate ledgers | supported run and fail-closed resource controls; timeout/cancel/OOM attempt yields no receipt | G02 or exact failure prefix | runtime evidence + residual operational assumption |
| E-034 | replay kind/path/old/new value and typed terminal are bound before leaf hashing | replay/JMT valid control; source-object/kind/path/value mutations | G07/G14 | algebraic + runtime evidence; A-03/A-11 |
| E-035 | backend JMT roots, four typed checkpoint commitments, and final SettlementV2 root are distinct constrained fields | typed commitment control; backend/semantic/root-role substitution | G08..G10/G15 | algebraic + runtime evidence; A-03 |
| E-036 | V1 semantic-root computation and compatibility entry points are physically absent | zero-symbol/reachability guard; any V1 declaration/export/caller fails | before build/INGRESS | exhaustive source evidence; historical bytes remain reject-only |
| E-037 | `JmtTreeRoleV2` and hierarchy level/parent context are part of every tree hash grammar | hierarchy/JMT controls; role/level/parent substitution | G07/G14 | algebraic + reduction to A-03 |
| E-038 | two precommitted product pairs replace a path-sized authenticated-set circuit | valid uniqueness controls; pair/product/order/challenge mutations | G07/G14 | algebraic + reduction to A-16; `epsilon_perm` explicit |
| E-039 | `SettlementUpdateTraceEnvelopeV2` carries the exact V2 JMT witness consumed once by evaluator/circuit | real commit trace control; proof/root/op/version/trailing mutation | before proof or G07 | differential/runtime evidence; A-11 |
| E-040 | cutover proves durable identity/atomicity; it does not claim historical V1/V2 state equivalence | exactly-once/reload control; record/state/CAS/crash mutation | cutover gate before INGRESS | runtime evidence + residual A-02 |
| E-041 | original and sorted lists are committed before challenges are derived | valid precommit controls; postchallenge row/count/precommit/challenge mutation | G07/G14 | algebraic + reduction to A-16 |
| E-042 | the circuit consumes the exact six-case update proof, not an opening proof | JMT native/R1CS controls; opening-like wrong direction/sibling/root mutations | G07/G14 | algebraic + runtime evidence; A-03 |
| E-043 | post-cutover theorem begins at the pinned first V2 root and makes no V1 equivalence claim | clean cutover/start control; alternate old record/first root/generation mutation | cutover/G01/G10 | runtime evidence + residual A-02 |
| E-044 | all cryptographic claims are explicitly conditional on A-03/A-05..A-10/A-16/A-17 | ledger validator control; unconditional/proved-from-tests wording mutation rejects | offline report gate plus G07 evidence | reduction to named assumptions + residual assumptions |
| E-045 | no local 248-bit field challenge is reported as overall security strength | exact advantage-ledger control; collapsed/unconditional bit-level report mutation | offline report gate | algebraic ledger + residual assumptions |
| E-046 | digest-to-field conversion is canonical, fixed-width, order-bound, and checked against modulus/range rules | public-input control; limb/order/high-bit/modulus mutation | G05/G08/G15 | algebraic + exhaustive finite-state; A-11/A-14 |
| E-047 | host source, sorting, hierarchy, and proof material are separately bounded and measured | segment/spool controls and RSS corpus; run/fan-in/cap/trailing mutations | G02..G04 then G07 | algebraic accounting + runtime evidence; allocator residual |
| E-048 | private running IVC state is never serialized as the public compressed proof/envelope | wire/type/size controls; private-field/PP/PK/running-state presence scan | G03/G04/G06 | exhaustive wire/source + runtime evidence; A-09 |
| E-049 | Cargo.lock, Nova source, profile/spec/grammar/shape and artifacts are identity-pinned together | clean source/artifact guards; any lock/source/feature/wire drift blocks | G01/G06 | runtime/source evidence; dependency review required after drift |
| E-050 | theorem/design rows close only after real verifier, endpoint, reload and review evidence | MODEL-C + T3 chain control; pre-verifier-only candidate cannot close row | G07..G16 | differential/runtime evidence; named assumptions stay residual |
| E-051 | dependency DAG is acyclic: semantic source → precommit → challenges → statement → endpoint → proof/receipt | source/control DAG guard and valid trace; control-to-source/precommit feedback mutation | G07/G14 | algebraic dependency proof + runtime evidence |
| E-052 | external-sort runs, merge fan-in and both set budgets are explicit parts of profile feasibility | multi-run controls; zero/exact/cap+1 bytes/runs/fan-in/set split mutations | G02; `ResourceLimit` | algebraic accounting + runtime evidence |
| E-053 | cross-set duplicates are accepted only for exact same-path replacement through the canonical flow owner | replacement control; unrelated cross-set ID/path/value mutations | before proof and G07 | exhaustive semantic check + runtime evidence; A-11 |
| E-054 | expected public endpoint is built from prior public data and canonical statement; private final state is compared afterward, never fed back | valid successor/MODEL-C; coherent endpoint/statement/final-state cycle mutation | G08/G09/G15 | algebraic acyclicity + runtime evidence; A-17 |
| E-055 | fold error uses exact `F_compressed(N)=2N+1`/`Q_fold`, separate from permutation error and backend advantage | executable min/representative/max ledger; `N`, count, overflow, million-step and unconditional-128-bit mutations | offline ledger plus G07/G14 observations | algebraic accounting + reduction to A-17; `Bad_*` unquantified |

## DC2-F01 through DC2-F24 live correction ledger

| ID | Exact corrected owner / check | Positive and minimally changed negative evidence | Live gate / result | Closure class and residual |
| --- | --- | --- | --- | --- |
| DC2-F01 | source-pinned fold accounting records two NIFS folds per post-base step plus three at compression: `F_compressed(N)=2N+1` | executable `N=1,2,3,max` count controls; old `N` and `N/2^127` formulas reject | offline ledger cross-checked with GEN2 G07/G14 | algebraic accounting; A-17 remains |
| DC2-F02 | A17 packet names the polynomial-depth theorem and all application premises | valid conditional claim; finite-root-counting-only or unnamed theorem claim rejects | authority review gate before G01 | reduction to named assumption; A-17 residual |
| DC2-F03 | Nova `IOPattern` is private dependency transcript state; project roles use `sha256_256` | pinned Poseidon/IO pattern controls; project content/JMT role using IO-pattern fails source guard | G06/G07 | source/runtime evidence; A-05 |
| DC2-F04 | live dependency is pinned `nova-snark 0.73.0` with exact source/Cargo.lock hashes | clean manifest/source guards and real proof; missing/different dependency rejects | G06; `BundleMismatch` | differential/runtime evidence; A-08 |
| DC2-F05 | first JMT materialization at later global version uses canonical null predecessor root | `first_update_at_later_global_version_uses_canonical_null_root` and six-case controls; forced stored-predecessor mutation rejects | before proof/G07 | exhaustive case + runtime evidence; A-03 |
| DC2-F06 | `commit_snap_with_update_trace` is the sole recursive write and returns role-tagged traces in commit order | real HJMT commit/replay control; role/default/drop/reorder trace mutation | before proof/G07/G10 | source reachability + runtime evidence; A-11 |
| DC2-F07 | strict `JmtUpdateTraceV2` decode caps each value, operation count, total bytes and version adjacency | exact/cap controls; value/aggregate/nested/version/trailing cap+1 corpus | G03/G04 or pre-evaluator | exhaustive finite-state + runtime evidence; A-14 |
| DC2-F08 | streamed SHA compares against independent generic `sha256_256` and frozen vectors | independent/FIPS controls; shared-helper, state, padding and block mutation | G07/G14 | differential/runtime evidence; A-03 |
| DC2-F09 | V2 root constructor hardcodes `RootGeneration::SettlementV2`; decoded mixed/legacy generations reject | live V2 derivation control; caller/mixed/legacy generation mutation | cutover/G01 | exhaustive registry/runtime evidence; A-02 |
| DC2-F10 | native update verification is only an input check; evaluator + R1CS + endpoint own checkpoint theorem | full mixed control; replay/role/count/hierarchy/statement substitution | G07/G08/G14/G15 | algebraic + runtime evidence; A-11/A-12 |
| DC2-F11 | `CheckpointTransitionConsistencyV2::evaluate_stream` independently derives replay/net/JMT/hierarchy/final root from exact trace bytes | real commit control; converging pre-state and recommitted sorted-ID substitutions reject | before proof and G07 | differential/runtime evidence; A-11 |
| DC2-F12 | `RecursiveTransitionTraceSourceV2` streams one immutable handle into bounded spool/runs; pass boundaries revalidate authority | single/multi-run controls; handle/root/digest/count/inode/pass-boundary mutation | G01/G02 before proof | algebraic resource + runtime evidence; filesystem identity residual |
| DC2-F13 | semantic rows bind path/old/new values and allow only exact same-path replacement | valid replacement/delete/insert controls; unrelated cross-set ID, malformed multiplicity/opcode mutations | before proof/G07 | exhaustive finite-state + runtime evidence; A-11 |
| DC2-F14 | separate source/control vocabularies and checked role-by-role equation produce exact schedule counts | frozen schedule instrumentation and ShapeCS control; omitted/double-charged term and former contradictory profile reject | G02/G07 | algebraic accounting + exhaustive finite-state; A-12/A-14 |
| DC2-F15 | deterministic bounds preflight before commit; post-commit proof-source failure is typed shadow non-acceptance with durable root | valid transition; every pre/postcommit fault, retry/reload and generation change corpus | exact trace prefix; no G16 after failure | exhaustive typestate + runtime evidence; A-15 |
| DC2-F16 | `HashJobKindV2` ledger sums every transcript instance with selector-exclusive maxima only where proved | exact role-count controls; missing/duplicate role and max-vs-sum mutation | G02/G07 | algebraic accounting + runtime evidence; A-14 |
| DC2-F17 | canonical semantic source is immutable and `TRACE_DIGEST` excludes derived SHA/control microsteps | source/control ordinal/DAG controls; source/control masquerade and feedback-edge mutations | G07/G14 | algebraic dependency proof + runtime evidence; A-11 |
| DC2-F18 | profile constructor proves per-spool bytes, run count, merge fan-in and unequal-set feasibility | zero/one/exact controls; cap+1 bytes/IDs/runs/fan-in/split mutations | G02; `ResourceLimit` | algebraic accounting + exhaustive finite-state |
| DC2-F19 | private authority/snapshot resolver obtains canonical config and immutable store-owned snapshot under one generation | real chain control; caller ID/config/snapshot/generation rotation mutations | G01/G10/G15 | exhaustive typestate + runtime evidence; A-01/A-02 |
| DC2-F20 | one typed statement binds height/predecessor, authority/profile/spec/grammar/bundle, roots, commitments, counts and work decomposition | canonical statement/MODEL-C; one-field substitution corpus | G06..G09/G14/G15 | algebraic + differential/runtime evidence; A-11/A-17 |
| DC2-F21 | each theorem-distinct SHA grammar has a typed role/schema/set-kind and unique registry row | `test_hash_registry_is_injective`; cross-role/set-kind/domain substitutions | G07/G14 | algebraic prefix-freeness + reduction to A-03 |
| DC2-F22 | `SettlementUpdateTraceEnvelopeV2::from_canon` bounded-decodes then invokes every nested update's native proof verification | valid strict envelope; recomputed envelope bindings with altered proof/root/op reject | G03/G04 or pre-evaluator | differential/runtime evidence; pinned JMT semantics |
| DC2-F23 | durable cutover stores generation/manifest/root atomically with pinned opaque-record digest and clean-process reload | `test_cutover_clean_process_reload`, exactly-once control; five crash/CAS/wrong/replayed-record seams | cutover gate before G01 | differential/runtime evidence; filesystem durability residual |
| DC2-F24 | witness/value/prover-state owners are private/redacted/zeroizing and absent from public facade/receipt | secret canary and six process outcomes; Debug/log/telemetry/public-API/artifact mutations | not authority; G16 exposes public digests only | differential/runtime evidence; dependency hard-kill zeroization is a non-claim |

## Section 17 adversarial matrix

| Attack / failure | Exact control and test | Deepest live result | Closure class / residual |
| --- | --- | --- | --- |
| S17-01 setup/prove opcode shape split | unconditional lanes plus `test_opcodes_use_fixed_shape`; change opcode/witness allocation | GEN2 G06/G07 rejects shape/bundle mismatch | exhaustive finite-state + runtime; A-12/compiler residual |
| S17-02 nonboolean/multiple selector | boolean selector constraints and `sum=1`; `test_done_cell_rejects_nonboolean` plus multi-selector mutation | G07/G14 constraint/verifier rejection | algebraic + runtime; A-12 |
| S17-03 inactive lane influences state | inactive input/output zero/equality constraints; inactive-cell mutation corpus | G07/G08/G14/G15 rejects | algebraic + runtime; missed-cell risk reduced to A-12 |
| S17-04 omit/reorder/replay event | exact counts, phases, ordinals, source commitment and unique finalization; skip/reorder/replay mutations | G07/G14 rejects | exhaustive finite-state + A-03/A-12 |
| S17-05 choose sort after challenge | both sequences in `IdentifierPrecommitV2`; precommit/challenge substitution corpus | G07/G14 rejects | algebraic + A-16/`epsilon_perm` |
| S17-06 duplicate spent/output ID | strict sorted rows, counts and two product pairs; duplicate/order/count mutations | G07/G14 rejects | algebraic + A-16/`epsilon_perm` |
| S17-07 recreate ID at different path | canonical flow compares full old/new `SettlementPath`; changed-path replacement mutation | evaluator or G07 rejects | exhaustive semantic/runtime; future moves need new theorem |
| S17-08 digest modulus alias | sixteen constrained `u16` limbs; legacy modulus/high-bit/limb mutations | G05/G08/G15 rejects | algebraic + exhaustive finite-state; A-11 |
| S17-09 SHA split/concat/domain confusion | prefix-free `sha256_256` and injective role registry; domain/label/part mutations | G07/G14 rejects | algebraic + reduction to A-03 |
| S17-10 SHA padding/length extension | exact FIPS length/block/finalization state; padding/length/post-final block mutations | G07/G14 rejects | algebraic + runtime; A-03 |
| S17-11 typed leaf differs from hash | circuit encodes frozen tag/fields/variable bytes; tag/schema/value mutation | G07/G14 rejects | algebraic + A-03/A-11 |
| S17-12 opening substituted for update | six-case update relation and running root induction; old/new/proof/sibling mutation | G07/G14 rejects | algebraic + pinned JMT semantics |
| S17-13 split-prefix error | constrained 256 key bits/common prefix/null siblings; prefix `0..=255` corpus and boundary mutation | evaluator/G07 rejects | exhaustive finite-state + runtime |
| S17-14 absent delete accepted as noop | replay semantics require old key/leaf; absent-delete mutation | evaluator/G07 rejects | exhaustive semantic/runtime |
| S17-15 wrong deletion coalesce kind | all leaf/internal/null sibling branches allocated; kind/coalesce/root mutations | G07/G14 rejects | exhaustive finite-state + runtime |
| S17-16 child-root hierarchy substitution | parent leaf encodes role/identity/child root; child/root/order mutation | G07/G08/G14/G15 rejects | algebraic + A-03/A-11 |
| S17-17 raw backend root as settlement root | SettlementV2 generation/layout/policy/definition/backend binding and limb compare | G08/G10/G15 rejects substitution | algebraic + runtime; A-01/A-02 |
| S17-18 V1 proof/root/receipt downgrade | zero V1 decoder/constructor/config/runtime reachability; inert byte and name reintroduction corpus | before allocation/INGRESS | exhaustive source/registry evidence; stale deployed binary residual |
| S17-19 prover-selected parameters | authority bundle resolved before proof decode; key/profile/spec/source substitutions | G01/G06 rejects `BundleMismatch` | runtime evidence; A-13 |
| S17-20 Nova event-zero off-by-one | private runner schedule and `num_steps()==consumed`; event-zero/count mutation | G08/G09 rejects | algebraic/runtime; A-17 |
| S17-21 independent block proofs | one continuous running state and cumulative endpoints; restart/skip/reorder mutations | G08..G10/G15 rejects | runtime + reduction to A-17 |
| S17-22 intermediate fold as receipt | final `Idle(done=1)` plus local verify and G16-only private constructor; early/double finalization | G07/G08 or typestate before G16 rejects | exhaustive typestate/runtime; A-15 |
| S17-23 related-statement folding malleability | exact context/statement/predecessor/full endpoint compare; coherent wrong-statement MODEL-C | G08/G09/G15 rejects | runtime + reduction to A-17 |
| S17-24 malformed/oversized proof | family cap, bounded outer/inner decode, curve canonicality, re-encode; size/trailing/point mutations | G02..G05 rejects before verifier | exhaustive finite-state/runtime; bounded decoder residual |
| S17-25 tampered private recovery state | generation/context/PP/step binding or deterministic replay; stale/truncate/bitflip/rollback mutations | private recovery rejects before INGRESS | runtime evidence; host/key compromise residual |
| S17-26 worker success without proof | only G16-issued nonreplayable receipt follows both verifier and endpoint passes | false worker status/replayed capability cannot construct receipt | exhaustive typestate/runtime; local verifier trust A-08 |
| S17-27 reorg/stale lineage | chain context/height/predecessor/generation rechecks and quarantine | G01/G09/G10/G15 exact failure prefix | runtime evidence; trusted fork selection residual |
| S17-28 OOM/time/cancel | checked preflight, bounded buffers and typed shadow non-success; cap/timeout/cancel corpus | G02 or exact prefix; never G16 | runtime evidence; liveness only |
| S17-29 witness logs/debug/core dump | private/redacted types, zeroizing buffers, permission/canary process corpus | no secret reaches public receipt/artifact | runtime evidence; OS/hardware timing/access residual |
| S17-30 V1→V2 false equivalence | authority-pinned first V2 snapshot/root and post-cutover-only statement | alternate V1 record/root claim rejects at cutover/G01 | runtime + residual A-02; no cryptographic V1 recovery claim |

## Section 19 proof and test corpus

| ID | Required corpus and exact owner | Positive / negative result and gate | Closure class / residual |
| --- | --- | --- | --- |
| S19-01 | `sha256_256` preimage vectors and `HashJobKindV2` registry | valid role vectors; domain/label/part count/order/length changes reject at G07/G14 | algebraic + differential; A-03 |
| S19-02 | FIPS SHA byte/block lengths `0,1,55,56,63,64,65,max` in native and R1CS lane | valid vectors; overflow/padding/block mutation rejects G02 or G07/G14 | exhaustive boundary/runtime; A-03/A-14 |
| S19-03 | inert V1 `0` versus legacy modulus alias and V2 limb encoding | V2 separates; legacy alias rejects G05/G08 | algebraic + exhaustive finite-state |
| S19-04 | fixed `enc32` byte order and sixteen `u16` limbs | roundtrip control; non-u16/order/high-bit/trailing mutation rejects G05/G08/G15 | exhaustive finite-state; A-11 |
| S19-05 | strict frozen leaf bytes for every registered family | empty/max/varint controls; tag/schema/version/trailing/cap mutations reject G03..G05/G07 | exhaustive registry/runtime; A-11 |
| S19-06 | six-case JMT parity including split prefix `0..=255` | native/R1CS controls; sibling kind/order/count/root/absent-delete mutations reject before proof or G07 | exhaustive finite-state/runtime; pinned JMT semantics |
| S19-07 | multi-operation JMT intermediate-root chain | valid batch; skipped/reordered/intermediate-root mutation rejects G07/G14 | algebraic/runtime; A-03 |
| S19-08 | hierarchy one/multiple bucket/serial/definition and create/remove vectors | valid hierarchy; child-root/role/changed-path mutations reject G07/G08/G15 | exhaustive finite-state/runtime |
| S19-09 | replay→net delete/insert/replace/same-path/duplicate/missing/reorder corpus | valid controls; every semantic mutation rejects evaluator or G07/G14 | exhaustive finite-state/runtime; A-11 |
| S19-10 | private external-sort run/merge and cleanup boundaries | one/multi-run controls; equal-across-runs/corrupt/truncate/swap/permission/race/cancel/cap mutations reject G02 or preproof | runtime/resource evidence; filesystem residual |
| S19-11 | immutable uniqueness precommit and old adaptive-witness regression | valid committed lists; postchallenge construction/precommit/count/challenge mutation rejects G07/G14 | algebraic + A-16 |
| S19-12 | cutover clean/restart/idempotent/atomic corpus | valid exactly-once/reload; snapshot/network/height/policy/layout/root/partial-write mutations reject before G01 | runtime evidence; A-02/filesystem residual |
| S19-13 | builder/store/upstream proof/evaluator/pass-one/pass-two differential | all valid intermediates match; any byte/root/count substitution rejects before proof | differential/runtime; A-11 |
| S19-14 | every opcode/JMT case under one setup/shape/bundle | ShapeCS/profile control; witness-dependent opcode/shape mutation blocks G06/G07 | exhaustive finite-state/runtime; A-12 |
| S19-15 | complete every-opcode mixed block through compressed Nova | TESTCS-1727 and MODEL-C valid control; independently recomputed false candidate rejects G07/G08 | differential/runtime + A-17 |
| S19-16 | one-field statement/profile/spec/count/opcode/SHA/uniqueness/net/JMT/hierarchy/link/final mutations | SEMANTIC-36 plus MODEL-C; each reaches intended constraint/verifier/endpoint gate and rejects | algebraic/runtime; A-03/A-12/A-16/A-17 |
| S19-17 | early/double finalization, event-zero, skip/trailing, overflow, post-final and Rust-branch corpus | valid finalization control; all mutations reject G02/G07/G08/G14/G15 | exhaustive finite-state/runtime |
| S19-18 | checksum-preserving/adversarial proof recomputation | MODEL-C starts from valid proof and preserves unrelated framing; unchanged verifier or endpoint rejects | G07/G08 and repeated G14/G15 | differential/runtime; no checksum-only closure |
| S19-19 | one/many-fold and continuous 1/3/5 block chains from same `z_0` | exact cumulative endpoints/steps pass; restart/skip/reorder rejects G08..G10/G15 | runtime + A-17 |
| S19-20 | consecutive non-consuming compressed snapshots | boundaries preserve running state; changed predecessor/terminal flag rejects G08/G09/G15 | runtime + A-17 |
| S19-21 | wrong PP/VK/suite/source/proof/steps/state/context/link/generation/backend corpus | active control passes; each mutation rejects at exact G01..G15 prefix before G16 | exhaustive typestate/runtime; A-13/A-17 |
| S19-22 | strict decoder fuzz/property corpus | canonical control; malformed length/field/curve/proof/trailing/re-encode mutation rejects G02..G05 | exhaustive bounded-decode/runtime; third-party decoder residual |
| S19-23 | recovery/reorg/crash/concurrent-resume corpus | deterministic replay/clean reload control; stale/truncate/bitflip/rollback/partial rename/write mutations produce no receipt | private recovery or exact G01..G15 prefix | runtime evidence; filesystem/host residual |
| S19-24 | cancellation/timeout/resource preflight corpus | supported control reaches G16; typed non-success leaves no accepted partial artifact | G02 or exact prefix, never G16 | runtime evidence; liveness only |
| S19-25 | secret-log/core/file-permission/zeroization process corpus | public evidence remains canary-free; debug/log/file/child-outcome mutations fail | no authority gate accepts secret evidence | runtime evidence; OS/hardware residual |
| S19-26 | release time/RSS/exit evidence and immutable source provenance | terminal exit/status/RSS records retained; partial output, source drift or missing exit cannot close row | measurement/review gate | differential/runtime; samples are not cryptographic proof |

## Section 21 pinned Nova/Poseidon findings and Models A/B/C

| ID | Exact vendor/integration finding and owner | Positive / minimally changed negative evidence | Gate / verdict | Closure class / residual |
| --- | --- | --- | --- | --- |
| S21-01 | `nova-snark 0.73.0`, Pallas/Vesta, Pedersen + IPA, source revision `2c0ed651…`, Nova source SHA `418d7425…`, Cargo.lock SHA `23a86f33…` | `test_nova_backend_owner_locked`; any source/lock/backend/feature mutation blocks | G06; no proof decode on drift | source/runtime evidence; re-audit required after drift |
| S21-02 | authority bundle is selected and bound before proof bytes | active GEN2 artifact control; proof-carried/substituted PP/VK/profile/spec source rejects | G01/G06 `BundleMismatch` | exhaustive typestate/runtime; A-13 |
| S21-03 | nonzero checked `num_steps` equals cumulative typed control schedule | T3 1/3/5 chain; zero/wrap/narrow/omitted/changed count rejects | G08/G09/G15 | algebraic/runtime; A-14/A-17 |
| S21-04 | every `z_0` and returned `z_N` limb is compared to canonical statement/evaluator endpoint | MODEL-C valid control; one-limb/input/final-state substitution rejects | G08/G15 | algebraic/runtime; A-17 |
| S21-05 | ordinary NIFS remains private to Nova augmented-circuit context with `U2.X[0]` binding | exact source/transcript guard; generic/copied transcript or context mutation blocks | G06/G07 | source/runtime evidence; A-05/A-08 |
| S21-06 | unchanged verifier checks both curves, relaxed instances, strict last secondary instance, folds, derandomization and both IPA proofs | active verifier control plus curve/instance/proof mutations | G05/G07 and repeated G14 | differential/runtime; A-06..A-08/A-17 |
| S21-07 | application rechecks envelope, chain, height, link, context, identities and prior/final outputs | public chain control; one-field corpus and authority rotation at every boundary | G01..G16 exact prefix | exhaustive typestate/runtime; A-01/A-15 |
| S21-08 | historical two-cycle flaw is fixed in pinned ancestor and guarded by repaired cross-curve vectors | repaired-source control; old/source/substituted last-instance structure blocks identity gate | G06; no live exploit reproduced | source/runtime evidence; A-08 |
| S21-09 | first-step scheduling is private and exact | continuous-chain control; first-call/off-by-one/event-zero mutation | G08/G09/G15 rejects | algebraic/runtime; A-17 |
| S21-10 | application/backend step-count conversions are checked | max control; overflow/narrow/wrap mutation | G02/G08 rejects | algebraic + exhaustive finite-state; A-14 |
| S21-11 | shortened ordinary-NIFS transcript is never exposed as a generic project folding API | private-owner/source scan; public export/copy mutation fails canonical-path review | before build/INGRESS | exhaustive source evidence; A-08 |
| S21-12 | returned Nova endpoint is not trusted without independent application comparison | MODEL-C control; backend-valid/wrong-application-endpoint candidate fails G08/G15 | G08/G15 | differential/runtime; A-17 |
| S21-13 | length-prefixed proof vectors are byte-capped before semantic decode | canonical control; oversized/truncated/trailing/noncanonical corpus | G02..G05 | exhaustive bounded-decode/runtime; third-party decoder residual |
| S21-14 | variable-time MSM/Rayon is an explicit local leakage boundary, not a soundness defect | valid proof and profiler/canary evidence; no constant-time claim allowed | G07/G14 may pass; report boundary remains | differential/runtime + residual side-channel assumption |
| S21-15 | polynomial-depth/GZT premises remain explicit and maximum depth is finite | A17 packet and max ledger control; unconditional/post-quantum/unnamed reduction mutation rejects | authority/report gate | reduction to named assumption; A-17 |
| S21-16 | Poseidon width/round/S-box/constants for both cycle fields are source-pinned | `test_nova_poseidon_wires_pinned`; constant/width/round/S-box mutation blocks | G06/G07 | source/runtime evidence; A-05 |
| S21-17 | native and circuit Poseidon use identical constants and exact fixed IO patterns | parity/source controls; absorb/order/pattern/field mutation rejects | G06/G07 | differential/runtime; A-05 |
| S21-18 | generic invalid squeeze panic is unreachable because Nova squeeze sizes are fixed at 128/250 bits | feature/reachability control; caller-controlled or zero/out-of-range squeeze path fails guard | before G06 | exhaustive reachability/source evidence; future feature drift residual |
| S21-19 | zero `HashType::Sponge` tag is never used for project domain separation | project SHA role registry/source guard; content/JMT/envelope use of Nova sponge fails | before build/G06 | exhaustive source evidence; A-03/A-05 |
| S21-20 | unsupported Poseidon variable-length TODO/unimplemented APIs are unreachable | feature/callgraph guard; introduced caller/export fails review | before build/G06 | exhaustive reachability evidence; dependency drift residual |
| S21-A | Model A constructs two semantic executions/endpoints for one bound statement or identifies a missing reduction premise | valid statement controls; each candidate has retained algebraic bytes or is classified non-A | offline theorem gate before B | algebraic analysis; no OOM/decode-only evidence accepted |
| S21-B | Model B compares canonical bytes through evaluator, StepCircuit, native/circuit transcript and independent helper | valid differential controls; minimized intermediate mismatch required | release differential gate before C | differential/runtime; shared-helper-only tests insufficient |
| S21-C | Model C repeats a checksum-preserving false candidate against unchanged verifier and exact endpoint | valid proof + independent recomputation; no candidate accepted falsely | G07/G08 repeated G14/G15 | differential/runtime; only A+B+C could confirm critical |
| S21-C01 | historical two-cycle candidate | selected repair and cross-curve parity agree; old attack not accepted on pin | fixed/non-applicable | source/runtime evidence; A-08 |
| S21-C02 | first-step schedule candidate | exact wrapper agrees; wrong count differs but cannot pass endpoint/count gate | integration hazard closed by G08/G09/G15 | runtime + A-17 |
| S21-C03 | missing application `z_N` compare candidate | backend may verify its own endpoint; exact project comparison rejects wrong endpoint | G08/G15; no false application acceptance | differential/runtime; A-17 |
| S21-C04 | decoder allocation candidate | malformed input can only reach bounded decode rejection, never a false theorem | G02..G05; availability gate | runtime; no Model A/C forgery |
| S21-C05 | Poseidon squeeze panic candidate | invalid generic input is unreachable; fixed internal transcript remains | non-reachable robustness hazard | source/reachability evidence; future drift residual |
| S21-C06 | native/circuit Poseidon divergence candidate | independent parity/source controls agree; no false proof accepted | G06/G07 | differential/runtime; A-05 remains |
| S21-C07 | proof-selected parameters candidate | attacker bundle can verify only under itself; GEN2 bundle/context comparison rejects | G01/G06 before proof or G08 | exhaustive typestate/runtime; A-13 |
| S21-C08 | variable-time proving candidate | resource traces may correlate with work but produce no alternate accepted endpoint | confidentiality/operations boundary, not soundness | runtime evidence; side-channel residual |
| S21-C09 | polynomial-depth/GZT candidate | premise is conditional; no implementation divergence or false acceptance reproduced | explicit theorem limitation | reduction to named assumption; A-17 |

No Section 21 candidate passes Models A, B, and C. Therefore this matrix records
zero confirmed critical verifier exploit and authorizes no Nova/Poseidon fork.

## Frozen opcode coverage

All 17 public source/control opcodes are indexed by
`RecursiveTraceEventCountsV2`, appear in `CONTROL_TRANSITION_TABLE_V2`, and are
synthesized by the same `CheckpointNovaCircuitV2`; absent transitions reject
instead of self-looping.

| Opcode | Exact constrained responsibility | Positive / negative evidence | Gate / closure |
| --- | --- | --- | --- |
| `BeginBlock=1` | enter Replay once and bind block/authority/source header | full mixed control; duplicate/late begin mutation | G07/G14; exhaustive finite-state/runtime |
| `ReplayInput=2` | consume canonical spent row and old leaf/path | input replay control; input-after-output, kind/path/value mutation | G07/G14; algebraic/runtime |
| `ReplayOutput=3` | consume canonical output row and new leaf/path | output replay control; wrong prefix/kind/path/value mutation | G07/G14; algebraic/runtime |
| `BeginHash=4` | initialize one exact role/message/block schedule | SHA controls; active/role/length/order mutation | G07/G14; algebraic/runtime |
| `ShaBlock=5` | constrain one FIPS compression state transition | FIPS/native controls; block/state/padding mutation | G07/G14; algebraic/runtime + A-03 |
| `EndHash=6` | require exact last block and publish role digest | valid closure; early/late/post-final block mutation | G07/G14; exhaustive finite-state/runtime |
| `UniquenessPrecommit=7` | parse/bind original+sorted list commitments and counts before challenges | valid precommit; version/count/digest/trailing mutation | G07/G14; algebraic + A-16 |
| `UniquenessChallenge=8` | derive/bind four challenges per set from committed transcript | valid challenge; context/precommit/output-byte mutation | G07/G14; algebraic + A-16 |
| `NetMerge=9` | merge spent/output semantic rows into exact net effect | delete/insert/replace controls; pending/kind/cardinality/Close mutation | G07/G14; exhaustive finite-state/runtime |
| `JmtUpdate=10` | bind typed update-envelope header and exact update count | valid envelope; tree/version/root/count/digest mutation | G07/G14; algebraic/runtime |
| `PromoteChildRoot=11` | promote constrained child result into exact parent leaf/update | hierarchy control; role/parent/child-root/order mutation | G07/G08/G14/G15; algebraic/runtime |
| `CommitTypedEvent=12` | commit four typed checkpoint event families in canonical order | typed commitment control; type/order/count/field mutation | G07/G08/G14/G15; algebraic/runtime + A-03 |
| `FinalizeBlock=13` | require complete counts/cursors/roots and unique `done=1` successor | valid finalize; early/double/count/transient/done mutation | G07/G08/G14/G15; exhaustive finite-state/runtime |
| `SourceMemoryWrite=14` | write exact canonical chunk metadata/bytes into private memory lane | paired source-window control; ordinal/count/byte/padding mutation | G07/G14; algebraic/runtime |
| `TraceChunk=15` | feed the same canonical chunk to source/global SHA contexts | shared live-chunk control; mismatched write/read/order/zero-tail mutation | G07/G14; algebraic/runtime |
| `UniquenessSorted=16` | stream source-authenticated sorted semantic row, strict version/order/set | valid sorted streams; row/version/global order/duplicate mutation | G07/G14; algebraic + A-16 |
| `JmtMicroOp=17` | stream exact typed JMT operation/proof/value/sibling micro-records | six-case controls; every authenticated transcript field mutation | G07/G14; exhaustive finite-state/runtime |

## Running-state field coverage

The state is one private `[Scalar; RUNNING_STATE_ARITY_V2]`; the table lists
every semantic range owner. Each range has a valid control in TESTCS-1727 and
at least one minimally changed SEMANTIC-36 or MODEL-C negative. All finalized
transient ranges must be zero before G08/G15.

| State field/range | Exact cells and invariant | Negative mutation / result | Closure |
| --- | --- | --- | --- |
| invariant digests | `ANCHOR_DIGEST_CELLS` covering context, predicate, profile, spec, PP/VK, statement, trace/witness, spent/output precommits | one-limb/profile/spec/bundle/source mutation rejects G06..G08/G14/G15 | algebraic/runtime; A-01/A-13 |
| authority scalar anchors | `ANCHOR_SCALAR_START..ANCHOR_SEMANTIC_COUNT_START` including generation/height/predecessor/layout/count identities | scalar/height/generation/predecessor mutation rejects G01/G08/G09/G15 | algebraic/runtime |
| declared semantic counts | `ANCHOR_SEMANTIC_COUNT_START..ANCHOR_OPCODE_COUNT_START` eight checked work counts | omitted/overflow/cap+1 count rejects G02/G07/G14 | algebraic/exhaustive finite-state |
| declared opcode counts | `ANCHOR_OPCODE_COUNT_START..ANCHOR_CELLS` all 17 opcode counts | any count mutation rejects finalization/G07/G14 | exhaustive finite-state/runtime |
| source trace cursor | `SOURCE_TRACE_ORDINAL_CELL`, `SOURCE_TRACE_BYTE_COUNT_CELL`, reserved chain binding cell | skip/reorder/byte-count mutation rejects G07/G14 | algebraic/runtime |
| current source identity | `SOURCE_EVENT_DIGEST_START..END` | source digest limb mutation rejects G07/G14 | algebraic/runtime + A-03 |
| pending source-memory window | active, source ordinal, chunk ordinal/count, byte count and `SOURCE_MEMORY_PENDING_BYTES_START..END` | write/read/chunk/zero-tail mutation rejects G07/G14 | exhaustive finite-state/runtime |
| control terminal | `PHASE_CELL`, `PRIOR_OPCODE_CELL`, `ORDINAL_CELL`, `DONE_CELL` | illegal phase/edge/ordinal/nonboolean/early/double done rejects G07/G14 | exhaustive 64-edge finite-state/runtime |
| hash schedule | active, ordinal, source ordinal, message bytes, block count/index, final flag, source hash limbs, role in `COUNTERS_START..COUNTERS_END` | role/count/index/final/source-hash mutation rejects G07/G14 | algebraic/exhaustive finite-state |
| SHA compression lane | `SHA_ACTIVE_CELL`, block ordinal, eight chaining words, 64 block bytes in `SHA_START..SHA_END` | state/word/byte/padding/block mutation rejects G07/G14 | algebraic/runtime + A-03 |
| replay parser | mode, input/output counts, parse active/header/stage/remaining, pending set/semantic row | stage/prefix/count/header/row mutation rejects G07/G14 | exhaustive 16-stage finite-state/runtime |
| uniqueness precommit parser | active/header/offset/low byte, spent/output count limbs and digest limbs | version/offset/count/digest/EOF mutation rejects G07/G14 | exhaustive finite-state/runtime |
| sorted-row parser/order | active/pass/set/list/current row, four counts, spent/output/global last-active/set/row ranges | version/set/order/duplicate/global-row mutation rejects G07/G14 | algebraic + A-16 |
| challenge parser/outputs | active/header/offset/low byte, three context digests, eight output words/full digests | context/precommit/output/count mutation rejects G07/G14 | algebraic + A-16 |
| permutation accumulators | `UNIQUENESS_PRODUCT_START..END` eight products | one-product/challenge/row/count mutation rejects G07/G14 | algebraic + `epsilon_perm`/A-16 |
| net parser/pending merge | active/kind/current row/new hash, pending spent/output rows, effect count, mutation count, closed | kind/path/value/pending/count/Close mutation rejects G07/G14 | exhaustive finite-state/runtime |
| JMT envelope header | progress/kind/trace digest/update count in `JMT_START..JMT_HEADER_STATE_END` | kind/digest/count/trailing mutation rejects G07/G14 | algebraic/runtime |
| JMT micro cursor | stage, completed/current update, next operation, record/proof stage, value chunk cursor/count | stage/order/operation/chunk/count mutation rejects G07/G14 | exhaustive finite-state/runtime |
| JMT tree identity/version/roots | tree tag/definition/serial/terminal, old/new versions, old/new roots | role/tree/version/root mutation rejects G07/G14 | algebraic/runtime + A-03 |
| JMT operation/value/leaf | expected op, key, value presence/length, proof-leaf presence/data | op/key/value/leaf/tag/length mutation rejects G07/G14 | algebraic/runtime |
| JMT sibling/split/coalesce | expected/next sibling, type/direction, previous key, mutation case, split cursors, parent/coalesced flags | every six-case sibling/prefix/coalesce mutation rejects G07/G14 | exhaustive finite-state/runtime |
| JMT running hashes | prior operation root/value metadata, raw chain, value/old-leaf/sibling/old-current/new-current hashes | any hash/order/root mutation rejects G07/G14 | algebraic/runtime + A-03 |
| hierarchy induction | progress, definition root, stage, previous role, parent identity/prior/new roots, child/parent products and counts | role/parent/child/root/product/count mutation rejects G07/G08/G14/G15 | algebraic/runtime |
| settlement/typed commitments | post-settlement root hex, typed commitment progress/active, Net/JMT products/count in `COMMITMENTS_START..END` | order/type/root/product/count mutation rejects G07/G08/G14/G15 | algebraic/runtime + A-03 |
| expected final public fields | expected trace/public-input/prior-state/post settlement/post definition/typed commitments/statement identities | any limb/identity mutation rejects G08/G09/G15 | algebraic/runtime + A-17 |
| consumed opcode counts | `CONSUMED_OPCODE_COUNT_START..END` all 17 actual counts | declared/consumed mismatch rejects finalization/G07/G14 | exhaustive finite-state/runtime |
| source/global byte contexts | both `BYTE_CONTEXT_WIDTH` ranges: counters, chaining words, buffer, header | context/chunk/header/counter/state mutation rejects G07/G14 | algebraic/runtime + A-03 |
| uniqueness hash contexts | list/transcript next-job cells and their byte contexts | job/order/context/chunk/digest mutation rejects G07/G14 | algebraic/runtime + A-16 |
