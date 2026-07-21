---
title: Z00Z Recursive Checkpoint Proof Specification
phase: 069
status: implementation-in-progress
updated: 2026-07-20
canonical_authority: .planning/phases/069-Recursive-Proof/069-TODO.md
---

# Z00Z Recursive Checkpoint Proof Specification

This technical paper is the public, implementation-facing synopsis of the Phase
069 recursive checkpoint contract. The canonical atomic requirements and owner
assignments remain in
[`069-TODO.md`](../../.planning/phases/069-Recursive-Proof/069-TODO.md). When
this synopsis and that authority differ, the phase-local authority wins and
this document must be corrected in the same change.

The implementation is not yet production-authoritative. Recursive evidence is
shadow evidence, `CheckpointProofSystem::VERIFIED` remains disabled, and Plan
06 remains locked until Plan 051 Tasks T0 through T4 have complete source-bound
release evidence.

## Current implementation status

Every contract below is a target unless this table says it is closed. Present
source shape, compilation, or a benchmark scaffold is not acceptance evidence.

| Surface | Current status on 2026-07-20 |
| --- | --- |
| Core chain/genesis identity | SOURCE PRESENT/NOT ACCEPTED. `GenesisChainIdentityV2` is exported and can be installed only after bounded manifest loading and full bootstrap reproduction; source-current release evidence is pending. |
| Exact portable context | SOURCE PRESENT/NOT ACCEPTED. The public context has exactly the six target fields and derives chain/genesis identity only from the installed core authority; source-current release evidence is pending. |
| Stable semantic reject enum | SOURCE PRESENT/NOT ACCEPTED. `RecursiveCheckpointRejectReasonV2` owns the exact stable 47-code `u16` taxonomy; operational failures remain internal `CheckpointError` variants. |
| Sole V2 facade | SOURCE PRESENT/NOT ACCEPTED. `z00z_storage::checkpoint::recursive_v2` is the sole public recursive surface and exposes no private step input or backend type. |
| Independent registry axes | SOURCE PRESENT/NOT ACCEPTED. Public rows independently bind cryptographic domain, transcript, roots, public-input encoding, config, parameter, authority, and runtime-profile generations; source-current release evidence is pending. |
| Preallocation proof | SOURCE PRESENT/NOT ACCEPTED. Typed preheaders run before active ConfigV3 full-object ingress caps, while the Nova body and seeded sidecar payload retain separate smaller bounds; source-current release evidence is pending. |
| T3/T4 evidence | CANDIDATE SOURCE/NOT ACCEPTED. The 83-entry manifest-pinned source revision is `0cd690bc4c04ee14e5047861ac50d68f473fcc6c7e59e383540f8ba05d6c61c3`; no source-current `accepted.marker` exists, and obsolete-source drafts or diagnostic runs cannot authorize closure. |

## Scope and security claim

Phase 069 proves the existing canonical checkpoint transition statement. It
does not create a second checkpoint theorem and does not grant canonical
admission by itself. The selected fast lane is a continuous Nova IVC chain with
one completed statement-bound segment per accepted block and explicitly
requested compressed snapshots. A later Plonky3 lane may prove the same
transition-consistency predicate for an epoch; it may not merely prove that a
list of Nova proofs verified.

The proof claim is bounded by the assumptions of the nested transaction,
signature, commitment, range-proof, hash, setup, implementation, cutover, and
authority systems. Neither successful tests nor observed proof diversity turn
those assumptions into unconditional 128-bit or post-quantum security claims.

## Target canonical ownership

The sole public recursive storage facade is
`z00z_storage::checkpoint::recursive_v2`. Portable objects have explicit type
IDs and wire versions registered by `CheckpointVersionRegistryV2`; no decoder
infers a version from a Rust suffix, backend label, filename, or failed decode.
Private trace and circuit records use separately registered local codec
versions and are unreachable from public ingress.

The inherited canonical theorem remains `CheckpointTransitionStatementV1`.
That inherited name identifies the unchanged Phase 068 statement contract; it
is not a recursive-proof compatibility surface. All new recursive context,
public-input, proof, sidecar, receipt, witness, trace, and rejection contracts
are V2-only.

## Target chain and genesis identity

`z00z_core::genesis` owns `GenesisChainIdentityV2`. It is derived only after
typed genesis configuration and settlement-manifest validation:

- `chain_id` is the canonical `u32` `GenesisConfig.chain.id` value.
- `network_id` is `z00z_crypto::sha256_256` under frozen domain
  `z00z.core.genesis.chain_identity.v2`, label `network_id`, and exact framed
  part order: `chain.id` as `u32-LE`, canonical `ChainType::as_str()`, chain
  name UTF-8, and four `magic_bytes`. The raw config spelling is parsed and
  canonicalized before hashing.
- `genesis_digest` is the raw 32-byte result of the core-owned
  `compute_genesis_identity_digest_v2`, under the same domain and label
  `genesis_identity`. Its exact framed parts are manifest version as `u32-LE`;
  canonical network; seven counts checked and encoded as `u64-LE`; root
  generation as `u64-LE`; nine exact lowercase-hex digest strings decoded to
  raw 32-byte values; five collision counts checked and encoded as `u64-LE`;
  and exact policies, rights, and vouchers artifact filenames.
- The legacy `compute_genesis_manifest_hash` uses native-width counts and
  unframed strings. Its persisted lowercase hexadecimal self-hash remains a
  mandatory compatibility check but is excluded from the V2 identity digest.
- One core validator rejects unknown manifest JSON fields, wrong manifest
  version/root generation/network, count disagreement or overflow, nonzero
  collision counts, malformed digest text, wrong artifact names, and a legacy
  self-hash mismatch before it derives and installs the identity.
- Startup pins the derived identity once for the process. Reinstalling the same
  identity is idempotent; any different identity, including an ABA transition,
  rejects.

Checkpoint code consumes this typed process pin. It does not accept caller-
provided raw chain, network, or genesis digest shadows. The constructor and
derivation helpers stay private or crate-private. Export and wiring remain
blocked until strict unknown-field rejection, golden vectors, every-field plus
domain/order/version mutations, and a global strict-load-to-storage integration
test pass.

## Target recursive context and public input

The canonical `RecursiveCheckpointContextV2` fields, in order, are:

1. `version = 2`
2. `chain_id: u32`
3. `network_id: [u8; 32]`
4. `genesis_digest: [u8; 32]`
5. `checkpoint_config_digest: [u8; 32]`
6. `predicate_digest: [u8; 32]`

The config digest comes from one resolved active ConfigV3 head. The predicate
digest is the executable Nova predicate digest. The canonical context digest is
bound into verifier parameters and every portable proof object, rejecting
cross-network, cross-genesis, cross-config, and cross-predicate replay.

The canonical `RecursiveCheckpointPublicInputV2` fields, in order, are:

1. version and context digest
2. statement digest and statement-core digest
3. height, chain index, and chain length
4. epoch index, start height, and end height
5. previous, output, and prior-output roots
6. delta root, witness root, and checkpoint-link digest
7. exact backend label `nova_streaming_compressed_v2`
8. verifier-parameter digest
9. exact proof mode `fast_classical_streaming_v2`

All 32-byte values are represented to the recursive predicate as sixteen
little-endian range-constrained `u16` limbs. Every `u64` value is four such
limbs. Direct field reduction of 256-bit digests is forbidden.

## Target continuous Nova lifecycle

The live path performs one fold for every accepted block. The same running
`RecursiveSNARK` owns cumulative `z_0`, step count, and `z_N`; compression
borrows that state and never restarts the chain. Fold cadence is fixed at one.
Recovery snapshot, compression, and publication cadences are independent and
remain Plan 06 policy decisions.

The public ingress-to-receipt sequence is exact:

```text
authority snapshot
  -> registered object family and cap
  -> bounded strict decode
  -> canonical re-encode and curve validity
  -> verifier-bundle identity
  -> unchanged Nova verification
  -> complete z_N comparison
  -> statement/link/predecessor comparison
  -> storage endpoint reload
  -> atomic shadow-sidecar write
  -> exact byte reload
  -> repeat Nova and endpoint checks
  -> receipt
```

Each transition consumes a private linear typestate value. Those values are not
cloneable, serializable, default-constructible, or reconstructible from a
boolean verdict. Authority rotation, reorg, cancellation, timeout, crash, or a
mismatch invalidates the attempt and requires a fresh attempt before decode.

## Target portable objects and caps

`NovaProofEnvelopeV2` is the sole portable Nova proof body, decoder, and writer
row. It uses public type ID `0x06900101` and wire version 2. Complete framed
envelopes use the active ConfigV3 17 MiB cap; proof bytes remain independently
nonempty and capped at 128 KiB before allocation, decode, or write.

`RecursiveCheckpointProofV2` is an embedded typed reference, not a second
portable proof body. It binds version, exact backend and mode labels, statement
and public-input digests, prior and output roots, verifier-bundle digest,
envelope digest and byte length, plus the retention-state reference slot. While
the storage-owned Plan 09 retention/pruning lifecycle remains `declared_only`,
that slot is exactly `NOVA_RETENTION_STATE_UNASSIGNED_V2` (all zeroes). The
sentinel means "no retention-state reference assigned"; it is not a digest of
configuration or any other object, and every nonzero value rejects. Plan 09
must introduce the real typed reference and its migration before writing a
nonzero value.

`RecursiveCheckpointSidecarV2` is an independent shadow object with public type
ID `0x06900102`, wire version 2, and active ConfigV3 24 MiB complete-object cap;
its fixed reference-only bincode payload retains the smaller 64 KiB seeded
schema ceiling. It may carry the typed proof reference and public evidence, but
never proof bytes, a verifier verdict, or a receipt capability.

`CryptographicVerificationReceiptV2` uses public type ID `0x06900103`, wire
version 2, and 16 KiB cap. It can be constructed only after the sidecar has been
written atomically, reloaded byte-for-byte, and the unchanged verifier plus
complete endpoint comparison have run again. Deserialized receipt bytes never
authorize canonical admission.

## Target rejection contract

`RecursiveCheckpointRejectReasonV2` is the sole public/wire semantic rejection
taxonomy. Its canonical codec is exactly one `u16` little-endian discriminant;
unknown values reject. The frozen codes are:

| Code | Reason | Code | Reason |
| ---: | --- | ---: | --- |
| 1 | `UnsupportedVersion` | 25 | `SnapshotBindingIncomplete` |
| 2 | `UnknownField` | 26 | `PruningBeforeArchiveFinality` |
| 3 | `StatementDigestMismatch` | 27 | `ArchiveNodePruningUnsupported` |
| 4 | `PublicInputDigestMismatch` | 28 | `SidecarAuthoritative` |
| 5 | `PriorOutputMismatch` | 29 | `MeasurementMissing` |
| 6 | `OutputRootMismatch` | 30 | `ChainTooShort` |
| 7 | `BackendUnsupported` | 31 | `ChainTooLong` |
| 8 | `BackendClaimUnsupported` | 32 | `StepSkipped` |
| 9 | `ProofBytesEmpty` | 33 | `StepRepeated` |
| 10 | `ProofBytesTooLarge` | 34 | `StepReordered` |
| 11 | `NovaPqAuthorityUnsupported` | 35 | `WitnessUnavailable` |
| 12 | `NovaChainRootMismatch` | 36 | `CanonicalAdmissionAttempt` |
| 13 | `Plonky3CanonicalRangeMissing` | 37 | `VerifiedCodecMissing` |
| 14 | `Plonky3DependsOnlyOnNova` | 38 | `MixedEra` |
| 15 | `Plonky3UnauditedPromotion` | 39 | `DaReadinessMissing` |
| 16 | `HybridCadenceMismatch` | 40 | `PqInlineAnchorUnsupported` |
| 17 | `EpochManifestIncomplete` | 41 | `PqCadenceDisabled` |
| 18 | `ProofSizeBudgetExceeded` | 42 | `PqCadenceInvalid` |
| 19 | `CelestiaPermanentStorageUnsupported` | 43 | `PqLiveCadenceStageMismatch` |
| 20 | `IpfsPinningMissing` | 44 | `PqAnchorMissing` |
| 21 | `ArchiveReplicationInsufficient` | 45 | `PqAnchorDigestMismatch` |
| 22 | `ArchiveProviderReceiptMissing` | 46 | `PqAnchorIncomplete` |
| 23 | `RetrievalAuditMissing` | 47 | `RecursiveDocumentationIncomplete` |
| 24 | `RetrievalAuditFailed` | | |

Operational failures such as malformed internal bytes, duplicate identifiers,
storage I/O, CAS conflicts, resource exhaustion, verifier implementation
failure, and internal invariants remain `CheckpointError`. They are not
misreported as a nearby semantic rejection. `CheckpointError::RecursiveRejected`
is the only bridge from a stable semantic reason into the operational channel.

## Target version registry and configuration

The version registry keeps independent columns for API suffix, public type ID,
wire version, domain/transcript version, settlement-root version, public-input
encoding version, config schema generation, runtime-profile generation and
manifest digest, semantic owner phase, lifecycle, cap, reader, and writer.
Cross-products are validated explicitly. The active Nova values use public wire
version 2, domain/transcript version 2, settlement-root version 2, and public-
input encoding version 1.

ConfigV3 is the only online writer after atomic activation and retains complete
ConfigV2 source/destination digest parity through an exhaustive leaf rename
ledger. It declares the state-sharding profile and reserves the offline receipt
mailbox schema for Phase 071. The mailbox remains `declared_only`, has zero
admission bytes per block, and has no Phase 069 decoder, reader, or writer.

## Evidence and completion boundary

Acceptance requires command-produced, source- and dependency-bound release
evidence. The retained packet must include:

- theorem-to-code coverage for every E-item, DC2 finding, assumption, attack,
  lemma, opcode, state field, gate, and vendor finding;
- three-model adversarial reproduction with unchanged-verifier and exact-
  endpoint outcomes;
- complete public ingress, gate-trace, write/reload, receipt, and canonical-
  admission reachability;
- mutation and branch evidence with no unresolved security-critical survivor;
- actual serialized object sizes and separated prover-local, verifier-cached,
  and per-snapshot/network material;
- release lifecycle measurements, bounded resource/cancellation/restart fault
  evidence, dependency identity, strict codec tests, and clean reviews.

Evidence under an obsolete source identity is invalidated and cannot carry an
acceptance marker. Estimated sizes, inferred exits, ignored tests, partial logs,
blank matrix cells, and smaller fixtures described as full-cap proving do not
close the phase. Plan 06 unlocks only after Tasks T0 through T4 pass together.
