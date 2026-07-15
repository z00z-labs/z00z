# Z00Z Recursive Checkpoint Contract

## Status and authority

This is a live Phase 069 authority document. Words such as target, planned, or
future specify mandatory Phase 069 scope; they do not license a design-only
placeholder. The binding technical authority is
`.planning/phases/069-Recursive-Proof/069-051-T2-CRYPTO-AUDIT-2.md`, Sections
12–21, followed by the active 069 plans and this contract.

At the T0 boundary no recursive proof implementation, decoder, receipt,
parameter loader, storage entry point, feature, or configuration enablement
exists. The first source symbol is created only by T1 after T0's evidence and
V1-eradication gates complete. `CheckpointTransitionStatementV1` remains the
Phase 068 canonical state-transition theorem; it is not a recursive
compatibility interface and must not be replaced.

## Canonical statement and hash ownership

Every proof lane binds the same Phase 068 statement, statement-core digest,
execution input, replay identities, HJMT root, delta/journal commitments, and
checkpoint link. A recursive lane may never replace exact transaction proof
bytes, change statement meaning, or create a parallel finality route.

All non-circuit digest computation belongs exclusively to
`z00z_crypto::sha256_256`. V2 defines a typed role/domain registry and frozen
FIPS SHA-256 framing vectors. Storage, Nova, Plonky3, adapters, tests, and
tools consume that registry; none defines a local SHA helper, alternate
framing, hidden conversion, or role alias.

## V2 relation

The only eventual relation is `CheckpointTransitionConsistencyV2`. Its public
input binds:

- V2 context: chain, network, genesis, contract-config digest, predicate
  digest, and role registry identity;
- the canonical statement and statement-core digest;
- height/epoch, predecessor and output roots, replay/delta/journal/link roots;
- exact ordered replay rows and HJMT inclusion/non-existence envelopes; and
- authenticated V2 backend parameters and predecessor recursive output root.

Witness construction is streaming and bounded. It checks every row/path against
the Phase 068 state theorem before circuit assignment and rejects unknown
versions, trailing bytes, aliases, wrong order, oversize lengths, deletion-path
families, root mismatch, and unbound data before allocation. A monolithic
witness arena, opaque byte blob, backend-native storage value, or unconstrained
accumulator is forbidden.

## Backend and evidence rules

Nova's only label is `nova_streaming_compressed_v2`. It proves a V2 streaming
step relation with explicit predecessor-output binding. The independent
Plonky3 epoch relation proves the same public relation over exact epoch
membership and cannot obtain validity solely from Nova output.

A verified V2 receipt binds backend identity and revision, context, predicate,
statement, public input, parameter manifest, verifier verdict, predecessor
binding, and bounded measurement. Library smoke, serialization success,
benchmark output, a configuration bit, or proof shape is not verification.

Evidence remains non-authoritative until the separate promotion conditions are
implemented and independently reviewed. T0 configuration rejects all recursive,
Nova, Plonky3, PQ-writer, and verified-backend activation attempts; there is no
success-returning stub.

## Config and storage contract

`checkpoint_contract.yaml` is V2-only configuration. Its recursive/Nova/
Plonky3 descriptions use the V2 labels and are disabled until the owning plan
lands. The configuration has one validated path resolver and fails closed on
unknown keys, non-normalized paths, collisions, unavailable stages, altered
limits, or enablement requests.

T1 introduces exactly one recursive module root:
`z00z_storage::checkpoint::recursive_v2`. Its `context`, `encoding`,
`witness`, `predicate`, `params`, `nova`, `plonky3`, `evidence`, `reject`, and
`store` modules have the exclusive responsibilities listed in
`069-PATTERNS.md`. No second crate, feature, alias, test-only implementation,
or persisted path is permitted.

V2 persistence occurs only after cryptographic verification. It uses canonical
binary encoding, exact content binding, atomic writes, reload verification, and
separate V2 content-addressed paths. It rejects symlinks, traversal, duplicate
keys, partial/stale writes, unsupported bytes, and reorg-invalidated evidence.
It never imports, reads, migrates, or falls back to the removed recursive
format.

## Post-quantum and retained Phase 068 objects

Phase 068 DA, archive, snapshot, and post-quantum anchor objects retain their
own versioned contracts and are not recursive proof inputs unless the V2
relation explicitly binds their canonical digest. A non-authenticating V2 epoch
evidence commitment is neither a signature nor an admission signal. An outer
post-quantum-oriented proof does not upgrade nested classical primitives to
end-to-end post-quantum validity.

## Failure and test requirements

All failures map to `RecursiveCheckpointRejectReasonV2` and occur before
persistence without exposing witnesses, secrets, or setup material. Mandatory
release-mode tests cover canonical vectors; every constrained-field mutation;
malformed/cross-version input; bounded streaming allocation; native/circuit
equivalence; independent backend verification; parameter mismatch; atomic
write/reload; restart/reorg; authority rejection; and residual-byte rejection.

The T0 test corpus additionally proves deleted public imports, constructors,
codecs, features, and labels fail in content-hashed external temporary
consumers. Static, public-API, metadata, feature, filesystem, deployment, and
runtime scans must show no recursive V1 reachability before T1 starts.

## Ordered delivery

1. T0 resolves authority evidence and removes recursive V1 completely.
2. T1 builds the canonical V2 storage relation and streaming cutover.
3. T2 adds real Nova verification; T3 adds independent Plonky3 epoch
   verification; T4 adds the reviewed authority gate.
4. No phase may advance by renaming, hiding, or documenting an absent feature.

The canonical checkpoint admission remains unchanged until T4's explicit,
reviewed promotion gate.
