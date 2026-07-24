# 069-06 Nova Cadence And Recovery Profiling Report

## Decision

Generation 2 keeps Nova folding continuous and makes the other three cadence
axes explicit and independent:

| Axis | Selected cadence | Rationale |
| --- | ---: | --- |
| Fold | 1 block | This is the recursive continuity invariant, not a publication policy. |
| Recovery snapshot | 100 blocks | A local checkpoint limits replay while avoiding a 78 MB durable image every block. |
| Compression | 1,000 blocks | Current compression costs about 40.5 s after folding, so automatic per-block compression is not operationally acceptable. |
| Publication | 1,000 blocks | The content-addressed envelope follows the network epoch; ordinary roles receive no proof material. |

The authority-pinned hot-recovery cap is `1,698,758,656 B`: three bounded
`540 MiB` images (active plus two verified snapshots) and a `64 KiB` journal.
The manifest rejects zero, the unrelated `24 GiB` RSS ceiling, a count other
than two retained verified snapshots, and any drift from these independently
encoded axes.

## Identity

| Identity | Value |
| --- | --- |
| Proof-source revision | `0ef121e74dc36cf1d9f61504d7f4fc13cb89054cd78f59a0552825058d763699` |
| Milestone worker source | `a0fd346405c1f3d103d62b7d7b886574ad50d58dd749fcea22f8bf22960ade69` |
| Raw `nova.rs` SHA-256 | `bc6f2482f4d66fc2806da9416448fe61fd0d94d686d10624f95098ef116b8073` |
| `Cargo.lock` SHA-256 | `dc39936ae850926a973d884ba4571eefb4be4f56e68ba459b914ec633b7f85ca` |
| Prover artifact SHA-256 | `004a17ee98ee4a00a1c48e9f3e5ba66b3eed78e9d590d2e187d869576463fa69` |
| Verifier bundle SHA-256 | `363c8727ca1b11aa3c07b036b4d672757cd3d2122e8aef23f154bbcb7fcdf36e` |
| Role-framed verifier-bundle authority pin | `984f6a28296e0d83bd2f381edd46e0fc6a83f9a40c174fefd812bd98e7e3a819` |

The raw artifact SHAs are transport diagnostics. Runtime authority uses the
role-framed bundle digest, validates the full bundle identity on every cache
hit, and never derives authority from a path or proof-selected value.

## Measurement Packet

The generation-2 policy records the conservative source-bound packet below.
The final artifact corpus independently reproduced the same costs within normal
run-to-run variation.

| Measurement | Authority packet | Final-source confirmation |
| --- | ---: | ---: |
| Block interval | 5,000 ms | 5,000 ms contract |
| Representative fold | 554 ms | 564,078 us average over 52 folds |
| Compression setup | 4,445 ms | 4,393 ms |
| Compression prove | 35,848 ms | 35,342 ms |
| Clean verifier load/decode/check | 166,888 ms | 171,292 ms |
| Compressed proof | 123,688 B | 123,688 B |
| Framed envelope | 346,907 B | format/header and proof-body guards passed |
| Peak Nova worker RSS | 10,773,794,816 B | 10,778,722,304 B |
| Accumulator image cap | 536,870,912 B | enforced by codec before allocation |

The final artifact corpus also measured setup at `15,413 ms`, accumulator
creation at `297 ms`, and all 52 post-initial folds at `29,332 ms`. The
retained prover material is `958,329,882 B`; the authority-distributed
verifier bundle is `15,372,615 B`. These files are local evidence and are not
ordinary consensus or wallet payloads.

The independent production-parameter RSS harness completed the full proof,
recomputed Model C, and clean-verifier lifecycle in `2,494,224 ms`. Model C
produced another satisfying `123,688 B` proof for its own statement, the
unchanged verifier accepted it, and the target all-limb comparator rejected it.
The clean verifier took `56,724 ms`; sampled peak VmRSS was `3,352,719,360 B`
and kernel peak VmHWM was `3,444,826,112 B`. Both parent processes exited zero,
process-group cleanup was clean, the worker lock was free, and no setup/cache
path was available to the clean verifier.

## Real Chain And Restart Evidence

The final T3 run used one continuous real Nova accumulator and unchanged
authority bundle:

| Height | Verification time | Peak RSS |
| ---: | ---: | ---: |
| 3 | 221,563,758 us | 9,676,398,592 B |
| 4 | 212,720,627 us | 9,755,803,648 B |
| 5 | 216,252,650 us | 9,755,803,648 B |
| 6 | 218,333,307 us | 9,787,858,944 B |
| 7 | 215,963,067 us | 9,787,858,944 B |

Heights 3 through 5 are the required real 3-step chain; heights 3 through 7
are the required real 5-step chain. Ordered statement, predecessor/output-root,
proof-envelope, artifact, and measurement digests are re-derived from persisted
sidecars rather than copied from the in-memory producer.

At height 8 the process recovered the same fork-bound accumulator from a
`78,449,078 B` snapshot in `177,152 ms`. The measured active-plus-two-snapshot
hot set was `156,898,261 B`, below the `1,698,758,656 B` cap. The same run then
rejected a forked height 9, accepted canonical height 9, reopened storage, and
folded canonical height 10 from the refreshed durable snapshot. It completed
in `2,420.24 s` with exit status 0.

## Operational Interpretation

- Per-block folding remains mandatory even though its current representative
  cost is far above the five-second block interval; scheduling/SLO ownership is
  outside Plan 06 and remains a later phase concern.
- Compression is not a hidden per-block action. Its measured setup/prove cost
  is separately recorded and scheduled only at the 1,000-block boundary or an
  authorized local on-demand request.
- Publication carries a bounded content reference by default. Only an explicit
  recursive-verifier role fetches the authority-pinned verifier bundle and a
  selected proof envelope; validators, watchers, and wallets receive zero
  PP/PK/VK/per-block proof bytes.
- Recovery journal capacity is finite (`256` records). Capacity preflight
  fails before snapshot, quarantine, or head mutation. Plan 10 owns journaled
  local recovery GC after rollback/read references clear.
- Plan 06 emits immutable epoch/range/chain-root/body-digest/generation/reference
  facts. It does not issue a `RetentionTicketV2`, tombstone proof bodies, infer
  deletion from age or directory contents, or change Plan 09 retention policy.

## Evidence

| Evidence | Path |
| --- | --- |
| Final artifact corpus | `crates/z00z_storage/outputs/checkpoint/069-06/task-1/verification/artifacts-final-0ef121-pin.log` |
| Retained final artifacts | `crates/z00z_storage/outputs/checkpoint/069-06/task-1/milestone/artifacts-final-0ef121-pin/` |
| Real 3/5 chain and restart | `crates/z00z_storage/outputs/checkpoint/069-06/task-1/verification/t3-chain-final-0ef121.log` |
| Clean verifier RSS | `crates/z00z_storage/outputs/checkpoint/069-06/task-1/verification/nova-verifier-rss/final-0ef121-persistent/measurement.env` |
| Mandatory bootstrap | `crates/z00z_storage/outputs/checkpoint/069-06/task-1/verification/bootstrap-final-0ef121.log` |
| Targeted release matrix | `crates/z00z_storage/outputs/checkpoint/069-06/task-1/verification/targeted-final-0ef121.log` |

## Security Residuals

Serialized recovery buffers use `Zeroizing<Vec<u8>>`, and recovery debug output
redacts bytes. The decoded third-party `nova-snark::RecursiveSNARK` type does
not implement `Zeroize`; adding that trait is impossible from this crate under
Rust's orphan rules and changing the vendored dependency is forbidden here.
The implementation therefore minimizes serialized copies, contains them in
zeroizing buffers, and keeps the decoded object private to the bounded recovery
session. No Tari vendor source was changed.
