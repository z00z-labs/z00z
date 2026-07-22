# 069-051 T0 Recursive-Proof V1 Eradication Manifest

**Status:** T0 source/configuration/public API deletion and all required
post-deletion validation are complete. No V2 source was present at this
historical T0 boundary. The final runtime packet is retained by `v1.14.0`, and
patch release `v1.14.1` passed detached clean-clone validation.

## Authority mode and protected evidence

The paired immutable capture and its `RepositoryLocalNoLiveV1` attestation are
in `069-051-T0-AUTHORITY-RESOLUTION.md`. The captured field digests are opaque,
fixture-scoped, and non-deployable. They are neither a decoder input nor a
persisted recursive object. No configured artifact/data/state root, authority
process, service, container, or deployment package existed; the isolated
temporary fixture was safely deleted after capture.

## Removed recursive surface

| Class | Exact owner/path | T0 result |
| --- | --- | --- |
| Storage modules | `checkpoint/{recursive_checkpoint,recursive_circuit,recursive_context,recursive_encoding,recursive_params,recursive_predicate,recursive_reject,recursive_witness}.rs` | Safely moved to trash; no module declaration, alias, cfg copy, or re-export remains. |
| Backend package | `crates/z00z_recursive_proofs/` | Safely moved to trash; removed from workspace and default members. |
| Public/storage ingress | `checkpoint/{mod,codec,store,store_fs}.rs` | Removed recursive codecs, sidecar persistence, paths, constructors, loaders, binding checks, and exports. |
| Configuration | `checkpoint_contract.yaml`, `contract_config.rs` | V2-only disabled streaming labels; no old proof/sidecar paths, limits, decoder branch, or enablement route. |
| Tests/fixtures | `tests/test_recursive_checkpoint.rs`, `test_recursive_predicate.rs`, `test_recursive_predicate_properties.rs` | Safely moved to trash; remaining tests assert T0 fail-closed activation and V2-only configuration. |
| Technical-paper mirrors | `docs/tech-papers/069-051-PLAN.md`, `docs/tech-papers/069-051-T2-CRYPTO-AUDIT-2.md` | Safely moved to trash; canonical planning originals remain under `.planning/`. |
| Build cache | Exact old release libraries, binaries, dep-info, and fingerprint directories named in the cleanup log | Safely moved to trash; final discovery after full release validation found no matching artifact. |

The deleted symbol inventory included the mandatory seed families:
`RecursiveCircuit*V1`, `RECURSIVE_CIRCUIT_*_V1`, `CheckpointNovaCircuitV1`,
`RecursiveCheckpoint*V1`, `RecursiveBackendManifestV1`,
`CheckpointTransitionWitnessMaterialV1`, `RecursiveBoundedObjectV1`,
`CryptographicVerificationReceiptV1`, `ReceiptScopeV1`, `ReceiptVerdictV1`,
old proof/sidecar codecs and tags, legacy backend labels, parameter/profile
bundles, smoke adapters, and every caller in the checkpoint facade. Their only
remaining textual use is this historical evidence, the active T0 plan, and the
AUDIT-2 record; none is compiled or decodable.

## Canonical V2 boundary

The configuration's only forthcoming recursive labels are
`streaming_transition_v2`, `nova_streaming_compressed_v2`, and
`plonky3_stark_epoch_v2`. All branches are disabled and unavailable. The
promotion path stops at `canonical_extended_statement`; any PQ-writer,
verified-backend, branch, flag, or codec activation rejects until its owning
V2 implementation plan. The single future module root is
`z00z_storage::checkpoint::recursive_v2`.

## Direct inventory and zero-hit evidence

| Scope | Command/result |
| --- | --- |
| Source/export/config | `rg` over `crates`, `Cargo.toml`, and `Cargo.lock` for the seeded symbols, labels, old package, and old module paths: zero hits. |
| Workspace graph | `cargo metadata --format-version=1 --no-deps` contains neither the old package nor an old recursive target. |
| Public API | External release consumers cannot import deleted types, codecs, constructors, or verifier paths. |
| Runtime/deployment | Configured artifact/data/state roots absent; no matching process/service/container was discovered; old release cache names were safely removed. |
| Documentation authority | T0 rewrote TODO, patterns, predicate, and both recursive technical papers to V2 terminology. The captured `1,084/1,084` count is historical T0 evidence only; the live Phase-069 coverage inventory is independently regenerated from the current TODO and reports `1,332/1,332` atoms. |

## Retained non-recursive versioned contracts

The following remain by direct source/callgraph evidence and are not recursive
proof APIs: `CheckpointTransitionStatementV1` and its statement digest helpers;
HJMT `BatchProofBlobV1` and settlement/root contracts; DA, archive,
publication, retrieval, snapshot, lifecycle, pruning, and post-quantum anchor
objects; and `CheckpointContractConfigV1`. They have no import, constructor,
decoder, return type, persistence path, or semantic dependency on the removed
recursive package. They remain protected Phase 068 regression contracts.

## Cleanup log

1. The isolated capture consumer and fixture were removed with `gio trash`.
2. The two stale technical-paper mirrors were removed with `gio trash`.
3. Exact old source files, package, and tests were removed with `gio trash`.
4. Exact release libraries, binaries, dep-info, and fingerprint directories
   matching the removed package/test names were removed with `gio trash`.
   One duplicate named input was already absent during the first invocation;
   the remaining dep-info file was removed in a second exact invocation.
5. A second source/runtime/cache discovery scan returned zero candidates. No
   configured canonical checkpoint, HJMT, user data, or unrelated V1 contract
   object was modified or deleted.

## External compile-fail corpus

All consumers were content-hashed temporary directories outside the workspace,
run with `cargo check --release`, then safely removed with `gio trash`.

| Case | Expected failure | stderr SHA-256 | Source SHA-256 |
| --- | --- | --- | --- |
| deleted type/verifier imports | E0432 unresolved imports | `2a5671143fb0f8be54c40b03a88807299a7e952f1dc0a33f01ad8840872611b4` | `Cargo.toml` `80a2d1fcbccaa49870ed0b040c37e72ce49081fc2ae48953111c2b7a1bdba95f`; `main.rs` `1577f5a5b8f7e9090dc31b20b05796f42b83e3c0407eba3f05612716b4504b82` |
| deleted codec/sidecar API imports | E0432 unresolved imports | `69dc7e018ba3fe760fa26763dbfbbb4313d01c8b31f06945fca0cb3bf5e85fdf` | `Cargo.toml` `92a157d4d6bc7d93a4187558d418fdc22a6caa82a71953ff9329258f7f6d93b5`; `main.rs` `f26858a1906d6029a51497c00efdd01db595303f1f7ced24032b4367023d01fd` |
| deleted `recursive_v1` feature | Cargo feature resolution failure | `0d81ff29b0f8264b612d775b53e98746fddfc5201d378ae685302de7491df357` | `Cargo.toml` `a4f2661247409f09c0e973f537663ce4f8768ee816802ede5fb5e6c4acb7c6f7`; `main.rs` `536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4` |

## Release metadata boundary

This is an intentional public API break. The repository version manager is the
only authorized mechanism for its version/release commit. The runtime changes
are retained by `v1.14.0`; the final clean-clone guard repair is retained by
`v1.14.1` at `37ece6c797d3807283eaea611252e657e10faad2`. The latter tag
and local branch are verified, while ordinary remote synchronization remains
pending after DNS failure. No force push is authorized or claimed.
