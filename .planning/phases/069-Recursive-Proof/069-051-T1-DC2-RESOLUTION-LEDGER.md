# 069-051 T1 DC2 Resolution Ledger

**Status:** closed supporting evidence for T1/T2. This ledger tracks DC2-F11
through DC2-F24 from the authoritative Audit 2 against the final storage-owned
V2 path. No `partial` or `open` row remains; any future such row reopens the
corresponding acceptance gate.

**Current source digests (2026-07-20):**

| Source | SHA-256 |
| --- | --- |
| `checkpoint/recursive_context.rs` | `e49d2503553c77cbd50759d40bb5d040d05cc26b5fa00a56f32bc3501327217a` |
| `checkpoint/recursive_trace.rs` | `c238f12fab5afae219c1dda149276a6fa236c3109e95a29a10d92b2558fc12ad` |
| `checkpoint/recursive_predicate.rs` | `7a9855b80ac77f64cbc20f5280e82b2ebfc2e8b7ce9e2cba90fdce4a3f8ed0e2` |
| `checkpoint/recursive_statement.rs` | `c1cc1167d67b68427a958b8200d47ad707ecf127a8978255af4f338b9dfae620` |
| `checkpoint/canonical_transition.rs` | `fbe59c64b19522a5ae256bd090aa90b3fb512bf0b492f4e1fb8528d525fdf4dc` |
| `settlement/store.rs` | `8502fd89255fac6268a5be752232fcfe96304dc8ac5e4f472874998b0b6fefa0` |
| `settlement/proof_batch.rs` | `e4f27e54177985bc1224520dd689114abd7010ed4e910debfa2323c3d08ff9ac` |
| `backend/redb/helpers.rs` | `ed9d7589d7f9b2814690e28b50a871c9422a44ebd007c3d2c62a8798cc28ac4a` |
| `backend/redb/mod.rs` | `1f8739d161a180345a4c21beb5e8568662f0c6868f819c5d08a3a55288a7bf13` |
| `tests/test_recursive_v2_nova_adversarial.rs` | `324bf25f4e5a63bbde7586fe0c006ac08c98b3aeaee6c083103fec4d07f1e021` |
| `z00z_utils/tests/test_os_hardening_integration.rs` | `98efe53dba6e8c2150541f69c27f45b90dec841e2b753dafdc3502e83f649356` |

## Frozen dependency DAG

```text
authority + immutable pre-state + execution
  -> independently evaluated storage transition
  -> transition commitment
  -> checkpoint/link binding
  -> recursive public input X_h + parameter/bundle identity + prior IVC state
  -> expected z_h
  -> proof
  -> receipt
```

`RecursiveTransitionStatementV2` is only the transition-commitment row.  It
binds the checkpoint/link, immutable pre/post settlement roots, immutable
pre/post definition roots, profile/spec/grammar digests, trace and JMT-envelope
digests, and total source counts.  It is not `X_h`, does not accept a proof or
receipt, and must not absorb later-layer identities before T2 adds their
relations.

## Finding ledger

| Finding | Status | Current source gate | Release evidence | Remaining condition |
| --- | --- | --- | --- | --- |
| DC2-F11 | implemented relation and global proof evidence | `TraceSemanticMachineV2::verify_jmt_envelope` and the sole Nova JMT/hierarchy lane authenticate the same envelope bytes, old/new proofs, hierarchy promotions, and immutable pre-definition root. | Native converging-prestate rejection; six-case/two-operation JMT fixtures; JMT/hierarchy R1CS mutation corpus; complete 1,727-step proof and recomputed Model C. | Final review packet only. |
| DC2-F12 | accepted — final measured streaming bound | `CanonicalCheckpointTransitionV2::from_exec` resolves the immutable handle from `SettlementStore`; the sole versioned HJMT segment stream is capped at 1 MiB, in-flight result capacity is 2 MiB, and input/snapshot reservation is separately capped at 64 MiB. | `test_profile_binds_evaluator_memory` passes at exactly `5,374,042 B`; the selected one-real-HJMT workload passes under the authority-selected single-thread path. | Repository authority accepted the final measured streaming bound under `phase-069-t2-interactive-authority-2026-07-20`. |
| DC2-F13 | implemented relation and global proof evidence | Replay rows carry the complete semantic identifier/value row; original/sorted commitments, two product pairs, strict order, exact-path replacement, semantic Net, and Net-to-terminal-JMT permutation are constrained in the sole Nova path. | Duplicate/order/cardinality/product/path/value/Net-kind mutations; complete mixed proof and recomputed Model C. | Final review packet only. |
| DC2-F14 | implemented | Source records and derived controls are separate opcode classes; `RecursiveCircuitProfileV2` derives and checks source and SHA bounds; the statement carries exact declared semantic/per-opcode work and finalization compares all 17 consumed counters. | Contradictory-cap rejection; exact 1,727-event mixed schedule with every opcode; complete proof reaches the unique zeroized successor. | Preserve the equation during authority selection/review. |
| DC2-F15 | implemented for native admission | All fallible trace/spool construction occurs on the exact preflight clone before the live HJMT commit. | `test_preflight_cannot_advance_store`. | Retain fault/restart evidence during the T4 durability audit. |
| DC2-F16 | implemented relation and complete-candidate evidence | Source/global FIPS framing and exact block controls are constrained from canonical chunks; list/transcript jobs use the same fixed lane and finalization requires every semantic hash job. | SHA-control mutations, exact 20-row role registry, 854 SHA blocks in the 1,727-step complete proof, and final cursor/context zeroization. | Authority must decide whether the measured candidate fits its external budget. |
| DC2-F17 | implemented for the current source/SHA relation | `trace_digest` commits source records only; `event_pass_with_source_context` gives derived controls a synchronous source borrow and `canonical_chunk` reconstructs one fixed-width chunk without a vector tape. | `test_source_chunks_match_encoder`; `test_trace_chunk_binds_contexts`. | Preserve this single-source relation while T2 adds semantic families. |
| DC2-F18 | implemented for source/sort feasibility | Profile construction checks content and two ID-sort spool allocations, resident capacity, run count, and merge fan-in before source creation. | `test_profile_requires_sorter_spool`; boundary profile tests. | Final authority budget must include measured evaluator and worker memory. |
| DC2-F19 | implemented for current resolver boundary | `RecursiveSnapshotHandleV2::from_store` captures one storage binding; canonical transition rechecks it before admission. | `test_snapshot_binds_definition_root`; stale-handle test. | Revalidate configuration/authority rotation when T3 creates the public-input layer. |
| DC2-F20 | implemented relation and global proof evidence | Typed statement plus `RecursiveCheckpointPublicInputV2` bind roots, checkpoint/link, profile/spec/grammar, executable predicate, PP/VK/bundle identity, prior IVC state, declared/consumed counts, and the independently derived successor without circular `z_N` input. | Statement/count/public-input/final-successor one-field mutations; strict bundle/envelope identity corpus; recomputed Model C reaches unchanged VK and target comparator. | Final review packet only. |
| DC2-F21 | implemented registry and work evidence | Current source, statement, identifier and Settlement-root transcripts use typed SHA roles and frozen codecs. The source/local, global, four list and fourteen uniqueness-transcript jobs occupy distinct `(schema, role)` rows. | `test_hash_registry_is_injective`; exact declared/emitted/consumed opcode equation in the complete mixed proof; final semantic-job cursor zeroization. | Authority selection remains external, not missing work accounting. |
| DC2-F22 | implemented native/R1CS relation and global proof evidence | `SettlementUpdateTraceEnvelopeV2::from_canon` verifies every native update proof, and `checkpoint::nova` constrains the same header/micro-op/proof/preimage/hierarchy bytes rather than trusting that verdict. | Strict-decode, all-six-case JMT, hierarchy, authenticated-byte/root mutations, complete mixed proof and recomputed Model C. | Final review packet only. |
| DC2-F23 | accepted — owned-boundary equivalence with redb residual | The storage owner validates policy/generation/pre-definition/post-settlement roots, then the redb owner performs one `Durability::Immediate` transaction with active-generation/definition-root CAS, exactly-once insert, commit, and strict durable readback. Test-only seams are absent from production and terminate without Rust destructor unwinding before the write transaction, before manifest insert, after insert/before commit, after commit/before readback, and after readback/before success. | Release `recursive_v2_cutover_owned_boundary_crash_corpus` passes all five subprocess/reopen stages; pre-commit stages permit one complete retry and post-commit stages expose exactly one committed manifest. Existing stale-CAS, substitution, atomicity, exactly-once, and clean-reload tests remain green. | Repository authority accepted owned-boundary equivalence while retaining the explicit non-claim about redb-internal fsync/directory-sync failpoints. |
| DC2-F24 | accepted — project-owned boundary plus dependency residual | JMT updates/envelopes and Nova witnesses are private and omit payload-bearing `Debug`. Project-owned trace payloads, segmented-spool frames, streaming decoder values/witness hashes, and JMT prior/new-value buffers zeroize explicitly. The bounded secret-process corpus clears inherited environment, captures diagnostics, applies fail-closed core-zero/non-dumpable hardening, and uses private atomic artifacts. | Release `test_secret_process_outcomes` passes success, explicit failure, sanitized panic, timeout, cooperative cancellation, and hard-kill children with canary-free stdout/stderr/artifacts, `0700/0600` modes, and observable zeroization. `test_secret_buffers_stay_private`, `test_hardening_clean_process_disables_core_dumps`, `test_secret_canary_stays_redacted`, trace/JMT/streaming-decoder zeroization, and public API/privacy scans pass. | Repository authority accepted the dependency/upstream hard-kill allocation-zeroization residual. No dependency-owned zeroization is claimed. |

The ledger is an index to retained implementation and release evidence; it is
not a substitute for that evidence. Every row is implemented or explicitly
authority-accepted on the final T2 source. Changing a bound owner or residual
disposition reopens that row and requires fresh positive, mutation,
evaluator/circuit differential and, where applicable, real-verifier evidence.

## Historical Gate 1 review record (2026-07-17)

- YOLO pass 1 found a stale current-status claim that the R1CS SHA lane already
  derived all canonical record bytes. It was corrected in `STATE.md`,
  `ROADMAP.md`, and the interim record: the lane is context-equality-bound,
  but `TraceChunk` payload bytes are not yet canonical source payload cells.
- YOLO passes 2 and 3 found no additional Gate 1 defect after checking the
  native converging-prestate rejection, statement count serialization,
  profile-resident accounting, one-owner/public-type scan, and release tests.
  These are scoped clean passes only; S1-01's fixed-width R1CS payload relation
  and the other documented T2 blockers remain significant globally.
- Doublecheck 1 independently traced `source identity -> first chunk header ->
  TraceChunk witness payload -> contexts -> SHA lane`; it confirmed the missing
  payload edge and the corrected status. Doublecheck 2 retraced the
  converging-prestate fixture and count equality from source pass through the
  statement; it confirmed native rejection/count binding only, not Model C.

## Historical residual-corpus review record (2026-07-19)

- YOLO pass 1 found that the F23 post-commit stages accepted any reload error.
  The corpus now directly proves manifest absence before commit, manifest
  presence after commit, and exactly-once publication after retry; its focused
  release rerun passes.
- YOLO passes 2 and 3 were consecutive significant-clean passes over exact vs
  equivalent durability wording, production exclusion of test hooks,
  project-owned secret outcomes, authority placeholders, A-17, and promotion
  locks.
- Doublecheck 1 mapped the F12 equation, all five F23 seams, all six F24
  outcomes, source identities, and A-17 fields to live source. Doublecheck 2
  found and corrected the two stale source digests above, then confirmed the
  wallet blocker, proof identity, external residuals, and T3/`VERIFIED` locks.
