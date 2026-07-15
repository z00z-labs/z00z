# Phase 069 Recursive Proof Specification

## Live scope

This specification is a mandatory Phase 069 requirement source. It must be
read with AUDIT-2 Sections 12–21, `069-PREDICATE.md`, `069-PATTERNS.md`, and
the active 069 plan. Historical wording in superseded material does not defer
any requirement. Phase ordering is a safety dependency: T0 has no recursive
implementation; T1, T2, T3, and T4 add one verified V2 capability at a time.

## T0 invariant

The repository contains no recursive V1 source, public API, crate, feature,
configuration enablement, wire decoder, persisted reader, test constructor,
smoke adapter, receipt path, parameter bundle loader, generated page, runtime
binary, or fallback. The only retained information is the opaque,
content-addressed authority-resolution record described by the active T0
manifest. It cannot compile, decode, serve, or authorize a proof.

T0 does not create a placeholder V2 prover or verifier. Any attempt to enable
recursive, Nova, Plonky3, PQ-writer, or verified-backend configuration fails
closed as unavailable. The removal is an intentional breaking change and its
version/release metadata is managed only by the repository versioning workflow.

## Single relation and inputs

`CheckpointTransitionConsistencyV2` is the sole relation. It binds the
unchanged Phase 068 `CheckpointTransitionStatementV1` plus:

1. V2 context and predicate digest;
2. exact configuration and parameter-manifest identity;
3. statement/core digest, checkpoint height, epoch, predecessor/output roots,
   replay, HJMT, delta, journal, and link commitments;
4. ordered exact execution rows and inclusion/non-existence HJMT envelopes;
5. predecessor recursive output binding; and
6. backend-specific proof public input encoded through one canonical V2 writer.

The Phase 068 statement remains canonical. A proof may not omit transaction
proof bytes, weaken replay, substitute a root, bind a different context, or
claim canonical admission merely by verifying a backend object.

## Canonical byte, hash, and field rules

The sole native hash implementation is `z00z_crypto::sha256_256`. T1 freezes
typed V2 roles, domain separation, FIPS SHA-256 framing, digest compression,
and field decomposition with golden vectors. All encoders and circuits consume
the same bytes. Strict decoders reject unrecognized schema, noncanonical
encoding, trailing material, integer overflow, reduction aliasing, and
unbounded lengths before allocating memory.

The V2 witness is streamed. Its capacity is bounded by explicit row, path,
byte, and recursion-depth maxima; each cap has checked arithmetic and
adversarial boundary tests. No complete-witness arena, hidden accumulator,
unconstrained digest, provider-native object, private key, or plaintext secret
may enter the relation.

## Backend requirements

### Nova block relation — T2

The only Nova label is `nova_streaming_compressed_v2`. The implementation uses
a real audited Nova API, binds the exact V2 context/manifest and predecessor
output root, and verifies the compressed block proof against canonical public
inputs. Setup material is authenticated, content-addressed, bounded, and
rejected on mismatch. A mock, library smoke call, serializer, or size check is
not a proof result.

### Plonky3 epoch relation — T3

Plonky3 builds an independent epoch proof over the same relation and exact
epoch membership. It validates its own verifier/parameter identity and cannot
derive validity solely from Nova output. The epoch evidence commitment is
`non_authenticating_digest_v2`; it carries no signature or authority claim.
The implementation must explicitly document the security boundary between an
outer post-quantum-oriented proof and classical nested primitives.

### Authority — T4

Only a reviewed T4 gate can make verified evidence eligible for the configured
authority policy. It requires successful real verification, complete negative
tests, reproducible benchmarks, parameter/config pinning, rollback without
statement change, independent security review, and evidence from the selected
T0 authority mode. Until then `CheckpointProofSystem::VERIFIED` is not an
admission route.

## Module and persistence design

T1 creates only `z00z_storage::checkpoint::recursive_v2` with canonical
`context`, `encoding`, `witness`, `predicate`, `params`, `nova`, `plonky3`,
`evidence`, `reject`, and `store` modules. Each public function is owned by one
module; re-exports occur only at `z00z_storage::checkpoint`. A second crate,
alias, façade, cfg-gated copy, or test-only implementation violates the
canonical-path rule.

Persistence happens after verification only. It uses canonical binary encoding,
atomic write/rename, exact statement/context/manifest/root binding, reload
validation, and separate V2 paths. It rejects symlinks, traversal, stale or
partial files, duplicate keys, unrecognized bytes, and reorg-invalidated
evidence. It never migrates or reads the removed format.

## Security and fault model

`RecursiveCheckpointRejectReasonV2` is the one public reject taxonomy.
Reject before persistence on any context, statement, root, replay, HJMT,
parameter, proof, predecessor, epoch, cap, codec, or storage mismatch. Failures
must not disclose witness/private/setup material and must not alter canonical
checkpoint state. Reorg, restart, parameter rotation, concurrent writers,
filesystem escape, and malformed external data are explicit adversarial cases.

## Required verification

- Run the repository bootstrap gate before work and again after cleanup.
- Use release-mode builds/tests only for Cargo validation.
- Prove every constrained public-input field and every rejection branch with
  positive, mutation, boundary, and property tests.
- Prove native/circuit byte and relation equivalence; prove both backends with
  their actual verifiers; prove bounded streaming allocation.
- Run static/export/feature/metadata/filesystem/runtime scans and external
  compile-fail consumers for the deleted surface.
- Preserve the 1,084 coverage identities with no missing, duplicate, or drift;
  update semantic pointers only intentionally.
- Run task-execution review in YOLO mode at least three times and continue until
  two consecutive reviews have no significant issue; run two independent
  doublecheck passes before advancing a plan.

## Completion rule

T0 completes only when the V1-free workspace passes its release validation and
all zero-hit, evidence, cleanup, and review gates. T1–T4 may then advance in
order. A renamed old symbol, a decoder behind a feature, a compatibility note,
a stale generated artifact, or a configuration-only success path is a failure,
not progress.
