# 069-051 T0 Authority Resolution

**Status:** `RepositoryLocalNoLiveV1` selected; two-process isolated capture,
source eradication, release validation, review convergence, and final
source/metadata/cache scans are complete.

## Decision

T0 does not require an externally supplied snapshot in this repository state. The
only repository-owned checkpoint configuration is
`crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`; its configured
paths are relative artifact paths and no configured live V1 authority exists.
Therefore T0 must create a deterministic, test-only final-V1 fixture with the
current canonical builders, capture it twice in clean processes, compare the
opaque exports byte-for-byte, then remove both the temporary fixture and all
recursive-proof V1 implementation surfaces. This is deletion evidence only; it
makes no deployment or live-state claim.

## Observed repository state (2026-07-13)

| Check | Result |
| --- | --- |
| Authority/config root | Exactly one: `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml` |
| Runtime artifact roots | `artifacts/`, `data/`, and `state/` absent |
| Snapshot authority environment | No `Z00Z_`, `CHECKPOINT_`, `RECURSIVE_`, or `NOVA_` variables configured |
| Live authority process/service/container | None discovered |
| Release build output | `target/workspace/release` exists, but is build output and never snapshot authority |

## Integrity anchors

| Input | SHA-256 |
| --- | --- |
| `Cargo.lock` | `56414379d3934b002c04218148065b13415ad51d5d14bbd10c547af7989aa5d7` |
| `checkpoint_contract.yaml` | `d8f09845569434a56b7fe8bea6b3cb9c4e0e7af63fe6e4767c68765cf95a7586` |
| `069-051-PLAN.md` | `dce919a59e483cdf466a8bd6d2b01f068916a1b9facac56a79449d63de7dd4ed` |
| `069-051-T2-CRYPTO-AUDIT-2.md` | `c9479b6e451ab3fbf812b42dd54b4056efdf7228d905bb1407cc6a108b99d951` |

## Repository-local capture record

The temporary consumer was outside the workspace at
`/tmp/z00z-069-051-t0-capture`; it invoked current canonical builders only:
`snapshot_fix::snap`/`save`, `build_cp_draft`, `build_stmt_core_v1`, and the
canonical HJMT inclusion-batch owner. It did not listen for requests, issue a
proof or receipt, create a configured storage root, or retain decoded V1 data.
Its temporary fixture store was an automatically removed `tempfile` directory.

| Capture process | Command | Result |
| --- | --- | --- |
| A | `cargo run --release --manifest-path /tmp/z00z-069-051-t0-capture/Cargo.toml` | success |
| B | same command in a separate process | success; byte-identical record fields |

| Field | Both clean processes |
| --- | --- |
| snapshot ID / SHA-256 | `7475d016ba7753d278bbe8ba485e2d4925e724f2beb2712f0a485c27ae4f2baf` |
| execution SHA-256 | `836c5c7a4918beedd8640755238f8ba53d9b78a0a66399777d10c168301dc9a9` |
| draft SHA-256 | `c2bd96ef6272a514e4e57cd6c0d8e393a9e37d7c91b38571c9c11e8ab5979e6f` |
| statement-core SHA-256 | `5d6899091b5d95d7327dc43edbb0fa20d1709f57c250db84b2b39e321b8db5a4` |
| canonical batch SHA-256 | `cacee0556050a0745ead30062f82a5cd8f76ba4abaaee7de3d51b5cc5c635fc8` |
| opaque capture-record SHA-256 | `e2cc19b13fe116edd106d5a64a91235a67cf6610e95978c784a6fdf5afffe3a0` |
| entry count | `1` |
| previous check root | `41e8aec090c4ca4edd3e8c0cd49a079bb7b0a46884193ed261f9a05b62621117` |
| final V1 / HJMT root | `0a358f514d5a9f7c14bf9550857603e3605015368aa487055a3caba8cc92b295` |
| journal digest | `a59b7021525355d0e27b0e42257a166b66e7a5a8661b43596e759664ab34ea00` |

The temporary source was content-addressed as follows before safe removal:

| Temporary input | SHA-256 |
| --- | --- |
| `Cargo.toml` | `419fdfa89f44691cc0c7455e08e1899e0bf44fdf5274e2bd04727ea6e70a5b05` |
| `src/main.rs` | `4f2b648fa3c8a1c83bc4cee1de9d8a47de28056b128287a778366960d8328b88` |

Only the non-decodable field digests above are retained. The temporary consumer
and its four exact release artifacts were removed with `gio trash` and their
absence was verified. The remaining shared cache was not removed because it is
not a named V1 artifact and may contain user-owned build output. The digests are
fixture-scoped deletion/cutover regression evidence, never production migration
evidence or a deployable snapshot.

## T0 completion evidence

1. `bootstrap_tests.sh`, `cargo build --release`, and `cargo test --release`
   passed after deletion; the full Scenario 1 release suite completed in
   2,233.77 seconds.
2. Final source, Cargo-metadata, and build-cache scans returned zero V1 hits;
   newly discovered stale dep-info/fingerprint files were safely moved to trash
   before the final scan.
3. Five YOLO task-review passes were completed. The last two were clean, as
   were two independent doublechecks. The T0/T2 runtime packet is retained by
   `v1.14.0`; final clean-clone reproducibility is retained by patch release
   `v1.14.1` at `37ece6c797d3807283eaea611252e657e10faad2`. The ordinary
   remote push of `v1.14.1` remains pending after DNS failure.
