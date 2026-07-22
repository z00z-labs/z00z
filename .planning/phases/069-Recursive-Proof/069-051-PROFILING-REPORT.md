# 069-051 T0–T4 resource and lifecycle profile

Status: **T0-T4 COMPLETE ON CURRENT SOURCE; NOT A PRODUCTION CAPACITY
APPROVAL.**

The active candidate is now proof-source digest
`1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3`
with Nova owner SHA-256
`dc075b43760601b3330e4738aae59312fdcb4415740333d96e2559d7b9aa07ef`.
The mandatory bootstrap, 36-case semantic corpus, 1,727-step TestCS replay,
three artifact gates, full proof plus independent Model C, clean verifier, and
continuous 1/3/5-block path all pass on that identity. The milestone-worker
identity is
`5573f73e36922368b8179551b47b2b03a31bf88ff6b67b23552eccf099961cf5`
and `Cargo.lock` SHA-256 is
`23a86f3341579b25ad5be96080a642405633df5f8c6e99dd4c3329d7d51f2a11`.
The installed authority is generation `2` with digest
`8ae07172f268f67bf4d5d2b4b11562f6625d9b18e269741ce6d018fb01a4661c`
and role-framed bundle pin
`d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff`.
Here, "source revision" is the project-owned proof-source digest embedded in
the verifier binding; the containing Git release is `v1.18.0` at
`df36df3d1b395c28cff91fc9d33c2494541e37de`.

## Executive answer

The live ConfigV3 **declared planning profile** is listed below. Its runtime
recovery/publication activation remains fail-closed until Plan 06 measures and
pins the snapshot codec, positive byte cap, and cadence manifest:

- one logical Nova fold for every accepted block: every 5 seconds;
- one local recovery-snapshot opportunity every 100 blocks: every 500 seconds,
  or 8 minutes 20 seconds;
- one explicit Nova compression opportunity every 1000 blocks: every 5000
  seconds, or 83 minutes 20 seconds;
- one Nova publication opportunity every 1000 blocks, independently of the
  recovery cadence;
- one Plonky3/PQ/retrieval-audit epoch every 1000 blocks;
- one state snapshot every 10,000 blocks: every 13 hours 53 minutes 20 seconds.

Therefore, Nova does **not** create and publish a compressed proof every five
seconds, and the 100-block event is a local recovery snapshot, not a portable
checkpoint proof. The same accumulator is advanced sequentially for every
block; compression is a non-consuming snapshot of that running chain.

The current source-bound diagnostic fold measurement required about `211.45 s`
of sequential Nova microfold work for a representative 316-microstep block.
That is about `42.29×`
slower than the five-second block interval before compression, persistence,
networking, or retry work. Adding independent cores or machines does not repair
this same-chain dependency: block `h+1` consumes the accumulator output of
block `h`. The current implementation is therefore suitable only as a lagging,
non-authoritative shadow lane. No honest production hardware specification can
claim that it keeps up with five-second finality.

## Evidence classes

| Class | Meaning |
| --- | --- |
| CURRENT MEASURED | Command-produced evidence for the exact identities at the top of this report. |
| PRIOR SOURCE-BOUND | A completed measurement of the same architecture on an earlier source identity. It is useful for sizing, but cannot accept the current source. |
| AUTHORITY LIMIT | A configured ceiling or selected policy, not observed usage. |
| DERIVED | Arithmetic from a measured size or live cadence. |
| LIVE REQUIREMENT, UNIMPLEMENTED | Mandatory scope in `069-TODO.md` with no canonical codec/runtime path to measure yet. |

Estimated values are never substituted for serialized bytes. `N/A` is not used
as an acceptance result: an absent mandatory implementation is recorded as a
blocker.

The active-candidate release `ShapeCS`/preflight now passes with the exact
base relation below. These are static lower bounds derived from the complete
sparse matrices, not substitutes for the serialized artifacts and process RSS
measured later in the lifecycle:

| Active-candidate base metric | Exact value |
| --- | ---: |
| Constraints / public inputs / auxiliaries | `809,802 / 1 / 675,408` |
| Nonzeros | `3,332,400` |
| Pedersen generator count | `1,048,577` |
| PP payload lower bound | `186,285,856 B` |
| VK payload lower bound | `186,285,856 B` |
| Verifier-bundle lower bound | `523 B` |
| Pedersen setup RSS lower bound | `201,326,784 B` |

The exact retained command output is
`final/semantic-source-1da/run.log`; it validates these values against caps
`1 GiB / 1 GiB / 64 MiB / 24 GiB` respectively.

## Measurement host

| Resource | Host used for the retained profile |
| --- | --- |
| CPU | Intel Core i7-14700KF; 20 physical cores, 28 logical CPUs |
| RAM | 67,194,576,896 bytes (`62.58 GiB`), no swap |
| Storage | NVMe, `1.732 TB` total and about `1.002 TB` available at capture |
| Kernel | Linux `6.8.0-136` |
| Rust | `rustc 1.96.1`, `cargo 1.96.1`, LLVM `22.1.2` |
| Profilers | GNU time, perf 6.8.12, Valgrind 3.22, flamegraph 0.6.8, samply 0.13.1, strace 6.8 |

The exact host/tool capture is retained at
`crates/z00z_storage/outputs/checkpoint/069-051/final/system/host-and-tools.txt`.
The host exposes no readable `/sys/class/powercap/**/energy_uj` counter to this
process, so watts and kWh are not fabricated from CPU utilization; the cost
section therefore provides an explicit measured-watts input formula.

## Cadence and event counts

At a nominal five-second finalized block interval:

| Event | Blocks/event | Interval | Events/day | Events/90 days |
| --- | ---: | ---: | ---: | ---: |
| Canonical block and logical Nova fold | 1 | 5 s | 17,280 | 1,555,200 |
| Local Nova recovery snapshot | 100 | 500 s | 172.8 | 15,552 |
| Nova compression/publication | 1000 | 5,000 s | 17.28 | 1,555 complete events, 1,555.2 steady-state average |
| Plonky3/PQ/retrieval epoch | 1000 | 5,000 s | 17.28 | 1,555 complete epochs |
| State snapshot | 10,000 | 50,000 s | 1.728 | 155 complete snapshots |

The 90-day challenge window is exactly `1,555,200` finalized blocks and begins
at `da_publication_ready`, not at local wall-clock time.

## T0–T4 lifecycle profile

### T0 — recursive V1 eradication and authority cutover

T0 is a migration/source-integrity operation, not a recurring proving stage.
Its steady-state RAM, CPU, DA, and IPFS cost is zero. Its retained costs are
repository/build evidence and isolated migration/cutover artifacts. T0 must not
be counted as per-block runtime.

### T1 — canonical trace, external sort, and HJMT transition

The selected native profile uses one `1 MiB` segment, one HJMT worker, `2 MiB`
of bounded in-flight results, two 65,581-byte source records, and a `2 MiB`
sorter buffer. The exact profile-accounted resident equation is `5,374,042 B`;
the separate input/snapshot reservation is `64 MiB`.

The retained one-transition comparison is:

| HJMT workers | Wall time | Peak RSS | Disposition |
| ---: | ---: | ---: | --- |
| 1 | 1.17 s | 588,700 KiB | authority-selected |
| 2 | 1.16 s | 573,184 KiB | measured but not selectable |
| 4 | 1.16 s | 572,460 KiB | measured but not selectable |

The negligible speedup is why the live profile remains one HJMT worker.

### T2 — setup, fold, compression, and verification baseline

The current source-bound 53-step diagnostic baseline measured:

| Stage | Wall time / throughput | Peak RSS or bytes |
| --- | --- | --- |
| Fresh PP setup | 16.424 s | included in full worker peak |
| Accumulator creation | 0.321 s | included in full worker peak |
| Remaining 52 folds | 30.475 s | `586.067 ms` per microfold |
| Compression setup | 4.305 s | included in full worker peak |
| Compression prove | 35.689 s | included in full worker peak |
| Bundle load/decode plus adversarial compact-wire corpus | 168.910 s | diagnostic, not clean-verifier latency |
| Bounded artifact-verifier worker | 355.403 s | 10,640,736,256 B peak RSS |
| Source-binding worker | 191.304 s | 10,564,128,768 B peak RSS |
| Full 1,727-step proof plus independently recomputed Model C | 2,526.747 s bounded worker / 2,527.058 s complete harness | 8,111,263,744 B worker peak; Model C 1,190.058 s and target comparator rejected |
| Clean format-4 verifier process | 58.343 s cold load/reconstruct/verify | sampled VmRSS 3,289,395,200 B; kernel VmHWM 3,348,504,576 B |

The 1,727-step proof harness constructs a same-format generation-2 bundle; its
proof-binding digest is
`e6bd650d3020977f24a2c2cbac36009a6e465cc4cac784f7bbce0bf5be2125ec`.
It uses the same authority digest, PP/VK, shape, proof source, and codec as the
installed artifact. The active artifact corpus and public 1/3/5 T3 chain
separately prove the installed `d76c545b…e1ff` role pin. The 58.343-second cold
sample is exact verifier evidence but is still one sample, not a production
p95/p99 SLO.

The current generation-2 artifact construction measured:

| Object/stage | Current source-bound measured value |
| --- | ---: |
| Private prover material | 958,329,882 B (`pp=958,329,152`, `pk=208`, header `522`) |
| Decoded VK | 977,729,672 B |
| Compact deterministic-key-omitting VK | 910,096,377 B |
| Compressed compact VK | 15,372,093 B |
| Compressed proof body | 123,688 B |
| Authority verifier bundle | 15,372,615 B |
| Private material raw SHA-256 | `c449daa46d2522acfa9456a02c37e341edd4fca53483da39b9dfafd831f298cb` |
| Bundle raw SHA-256 | `86da72808877492cf73bb5ac3e0878abfd8c97ecbe9e91b2a0efb3d6d68fdf38` |
| Authority role-framed bundle pin | `d76c545b57d2cffec8e2e2564241858d5ab66d91ea537ed18306461eaab0e1ff` |
| Three active-pin artifact gates | 15:15.88 total command wall; 10,640,736,256 B maximum worker peak RSS; swap 0 |
| Source-binding artifact worker | 10,564,128,768 B peak RSS |

The current authority limit is `12 GiB` for setup/fold/prover work, `4 GiB`
for a clean verifier, `1 GiB` for the private prover-material cache, `64 MiB`
for the encoded verifier bundle, `128 KiB` for a compressed proof body, and
`512 KiB` for the outer proof-envelope authority cap. The stricter fixed-codec
payload maximum is `449 + 2×417 + 2×110,944 + 131,072 = 354,243 B`; the
measured `346,859 B` payload has the same `7,384 B` headroom as its proof body.
The public V2 object adds the `48 B` registry preheader, so the retained framed
object is `346,907 B`. The
format-4 verifier bundle is
`15,372,615 B`, below the `64 MiB` authority distribution limit by
`51,736,249 B`. Its private compact wire omits only deterministic Pedersen key
vectors and reconstructs them from pinned labels and authority-bound exact
counts; strict format, length, offset, count, expanded-digest, canonical-zstd,
key-layout, and selected-digest gates cover the reconstruction boundary.
The isolated verifier remains a VK-only role: it never loads or generates
`PublicParams`, PP, or PK. Its cold path intentionally regenerates the four
public deterministic Pedersen commitment-key vectors from pinned `ck`/`ipa`
labels and authority-bound counts before decoding the VK. Therefore the
58.343-second number is cold bundle load, deterministic key reconstruction,
decode, and proof verification together—not proof checking alone.

### T3 — continuous same-accumulator chain baseline

The current source-bound continuous run used the following exact chain
geometry:

| Block milestone | Cumulative microsteps | Serialized envelope |
| ---: | ---: | ---: |
| 1 | 316 | none (`FoldOnly`) |
| 3 | 948 | 346,907 B framed (`346,859 B` payload + `48 B` preheader) |
| 5 | 1,580 | 346,907 B framed (`346,859 B` payload + `48 B` preheader) |

Blocks 3 and 5 are the two explicitly requested non-consuming snapshots. Block
1 advances the same accumulator but emits no envelope, sidecar, receipt, or
network publication.

The 1/3/5-block chain passed in `1,634.91 s` test time (`27:15.67` command
wall), or `326.982 s/block` across the five accepted blocks. It reached
`9,843,589,120 B` (`9,612,880 KiB`) command peak RSS and used no swap. The proof body is
`123,688 B`; the envelope contains two `417 B` portable public inputs, two
`110,944 B` Nova public states, and a `449 B` header. The exact codec sum is
`449 + 2×417 + 2×110,944 + 123,688 = 346,859 B`; registry framing makes the
persisted/network V2 object `346,907 B`. The `128 KiB` cap applies to the proof
body, not to the whole envelope.

Each height-3/5 snapshot also has one canonical `533 B` reference-only sidecar
(`48 B` registry preheader plus `485 B` canonical bincode payload). The
content-addressed `354 B` fixed-layout persisted checkpoint claim is shared by
both snapshots. Its cryptographic
verification receipt is exactly `572 B` (`48 B` preheader plus `524 B`
payload), but it is opaque, returned in memory, has no decoder, and is
intentionally **not persisted**. Thus the first successful snapshot set writes
`346,907 + 533 + 354 = 347,794 B`; the second adds `346,907 + 533 = 347,440 B`
because the claim is already present. The two T3 snapshots write exactly
`695,234 B` before filesystem metadata. The receipt contributes
no durable or network bytes. These sizes follow the sole live codecs and the
height-3/5 field values; the terminal live-file monitor independently checks
the persisted envelope, sidecar, and claim lengths. The sidecar payload uses
`485 / 65,536 B` of its seeded-decoder cap (leaving `65,051 B`), while the
claim uses `354 / 1,024 B` of its bounded reload cap (leaving `670 B`).

### T4 — terminal validation and tooling

The post-fix active-candidate bootstrap completed in `4:47.49`, reached
`10,041,600 KiB` maximum RSS, performed no swaps, and exited `0`. Bootstrap is an early
regression detector, not an aggregator steady-state requirement.

The current-source full-proof verifier harness passed in `42:07.08` command
wall (`2,527.058 s` harness measurement), at `7,921,156 KiB`
(`8,111,263,744 B`) maximum RSS and no swap. It completed all 1,727 folds,
compressed and verified a `123,688 B` proof inside a `346,859 B` full-fixture
envelope, ran the separate clean verifier, recomputed a second complete Model-C
proof in `1,190.058 s`, and observed the unchanged verifier accept that proof
for its own statement before the target all-limb comparator rejected it.
Process-group cleanup was clean and the single-flight worker lock was free.

The active-candidate semantic, TestCS, proof/Model-C, artifacts, continuous
chain, clean verifier, Criterion, GNU time, Massif, and strace runs are
terminal current-source artifacts. `perf` and Samply were attempted against the
exact resolved release executable but the host kernel denied PMU access at
`perf_event_paranoid=4`; passwordless privilege was unavailable and the sysctl
remained unchanged. Those metrics are explicitly unavailable, not fabricated.
Broad release and review convergence remain separate closure gates.

## One cycle and one checkpoint: measured resource summary

| Unit | Time | Peak RAM | Durable bytes attributable to the unit |
| --- | ---: | ---: | ---: |
| One representative logical block fold (316 microsteps, diagnostic arithmetic) | `211.454876 s` | included in bounded Nova worker | none by default; accumulator advances in memory |
| One accepted block in the full 1/3/5 lifecycle | `326.982 s/block` average | command peak `9,843,589,120 B` | no default proof publication; test/evidence I/O is not a production write rate |
| One requested compressed snapshot | compression setup `4.305 s`; prove `35.689 s` in the 53-step diagnostic | included in `10,640,736,256 B` mutation-worker peak | proof body `123,688 B`; envelope payload/framed object `346,859/346,907 B`; local sidecar `533 B`; first-set claim `354 B`; first persisted set `347,794 B`; in-memory-only receipt `572 B` |
| One authority artifact validation generation | three active-pin gates `15:15.88` | `10,640,736,256 B` worker peak | private material + bundle `973,702,497 B` (`928.60 MiB`) |
| One 100-block recovery opportunity | every `500 s` | production snapshot RSS not implemented/measured | `R` unknown; active cap is zero, so no false zero-byte claim |
| One declared 1000-block publication candidate | every `5,000 s` after Plan-06 activation | clean cold verifier `58.343 s`, HWM `3,348,504,576 B` | `346,907 B` canonical framed envelope, excluding DA provider framing |

`/usr/bin/time` filesystem-block counters include release compilation, test
harness and retained evidence, so they are not relabelled as protocol disk
writes. Exact production spool/journal amplification requires the Plan-06/10
scheduler and recovery codec; those live requirements remain mandatory.

## Five-second feasibility

A representative block currently expands to 316 Nova microsteps. From the
preceding source-bound fold measurement:

```text
fold_time_per_block = 316 * 0.669161 s = 211.454876 s
required_microfold_time = 5 s / 316 = 15.8228 ms
diagnostic_slowdown = 211.454876 / 5 = 42.2910x
diagnostic_maximum_rate = 86400 / 211.454876 = 408.60 blocks/day
diagnostic_backlog_growth = 17280 - 408.60 = 16871.40 blocks/day
```

The final 1/3/5 lifecycle measurement is slower (`326.982 s/block`, about
`65.40×` the five-second budget) because it includes requested compression,
verification, persistence, and reload work. It cannot be declared five-second
capable until a terminal run demonstrates at most five seconds per block with
bounded p95/p99 latency.

The command averaged `832%` CPU, approximately `8.32` fully occupied logical
cores, while the sequential accumulator still fell behind by about
`17,015.77 blocks/day` (`17,280 - 86,400 / 326.982`). This is the measured
prototype core demand, not a sufficient production core count. More cores
cannot parallelize block `h+1` ahead of the accumulator output for block `h`.

The active authority has four distinct time budgets that must not be
collapsed into the five-second block interval:

| Budget | Active value | Meaning |
| --- | ---: | --- |
| Canonical block interval | 5 s | Consensus/finality cadence; it does not wait for recursive evidence. |
| Cancellation response | 5 s | A running worker must observe a requested cancellation within this window. |
| Hard kill after cancellation | 15 s | Plan-10 worker-isolation backstop; never a successful proof deadline. |
| Complete prover attempt | 3,600 s | Maximum wall time of one supported proof attempt before typed deadline failure. |

The T3 run guard therefore uses the 3,600-second complete-prover ceiling while
checking the cancellation flag at every fold step. Interpreting the
cancellation-response budget as the entire proof lifetime would reject every
currently measured honest proof and is forbidden. Current-source cancellation
p99 and hard-kill isolation still require retained T4/Plan-10 evidence.

Thirty-six hypothetical independent prover lanes would require roughly
`36 * 12 GiB = 432 GiB` under the authority cap. That calculation is useful
only as a warning: the same accumulator cannot fold dependent blocks in
parallel, so it is not a deployment solution.

## Bytes written every five seconds, 100 blocks, and 1000 blocks

Use these symbols until the missing codecs are implemented and measured:

- `S_da`: exact serialized `CheckpointDaPayloadV2` bytes per finalized block;
- `D`: average unique exact challenge bytes added per finalized block after
  digest deduplication and omission of derivable witness data;
- `R`: exact serialized local `NovaAccumulatorSnapshotV2` bytes;
- `E`: exact serialized portable framed `NovaProofEnvelopeV2` bytes
  (`346,907 B`; its fixed payload is `346,859 B`);
- `P`: exact serialized current reference sidecar (`533 B` for T3 height 3/5);
- `C`: exact fixed-layout current persisted claim (`354 B`);
- `Q`: exact opaque in-memory receipt (`572 B`, never persisted or decoded);
- `S_state`: logical live HJMT current-state bytes;
- `U`: unique challenge bytes generated per day (`17,280 * D`).

| Window | DA write | Nova/recovery write | Challenge input |
| --- | ---: | ---: | ---: |
| Every 5 s / 1 block | `S_da` | one in-memory fold; no default compressed proof write | `D` |
| Every 100 blocks | `100 * S_da` | one local recovery snapshot `R` | `100 * D` |
| Every 1000 blocks | `1000 * S_da` | one scheduled envelope `E`, plus the ten recovery opportunities already counted | `1000 * D` |
| Per day | `17,280 * S_da` | `172.8 * R + 17.28 * E` before GC/dedup | `17,280 * D` |
| 90 days | `1,555,200 * S_da` | recovery is a rolling local set, not 15,552 retained copies; Nova bodies use their separate finite lifecycle | `1,555,200 * D` logical |

`CheckpointDaPayloadV2` is mandatory live scope in `069-TODO.md`, but no such
canonical Rust type/codec is present in the current code. Its exact byte count
cannot be fabricated from the legacy JSON/Celestia-local request. Likewise,
`max_nova_hot_recovery_bytes` is currently `0`, and the canonical
`NovaAccumulatorSnapshotV2` persistence codec is not active. Consequently,
`S_da` and `R` are production blockers, not zero-byte costs.

When the current explicit `Snapshot` ingress is invoked, its first local
evidence set is `E + P + C = 347,794 B`; a later snapshot that shares the same
content-addressed claim adds `E + P = 347,440 B`. `Q` remains memory-only. This is a measured
object-layout statement, not a claim that the downstream 1000-block scheduler
or publisher already invokes that ingress. Network Nova traffic below counts
only the portable envelope `E`; provider framing, DA payload, sidecar policy,
and challenge-pack bytes cannot be added until their live codecs exist.

## Nova publication traffic

Using the current canonical framed `E = 346,907 B` and the declared
1000-block publication candidate (not an active network publisher):

| Quantity | Derived bytes |
| --- | ---: |
| One publication | 346,907 B |
| Average per day | 5,994,552.96 B (`5.72 MiB`) |
| 1,555 complete publications in 90 days | 539,440,385 B (`514.45 MiB`) |
| Steady average traffic | 69.3814 B/s |

For comparison, the same measured envelope at other cadences is:

| Envelope cadence | Complete writes / 90 days | Bytes / 90 days |
| --- | ---: | ---: |
| Every block (not configured) | 1,555,200 | 539,509,766,400 B (`502.45 GiB`) |
| Every 100 blocks (recovery opportunity, not default publication) | 15,552 | 5,395,097,664 B (`5.02 GiB`) |
| Every 1000 blocks (declared publication candidate) | 1,555 | 539,440,385 B (`514.45 MiB`) |

If an operator incorrectly published the envelope every block, it would write
`5,994,552,960 B/day` and `539,509,766,400 B/90 days` (`502.45 GiB`). That is
not the declared downstream policy.

The sidecar and claim are local evidence-store objects in the current path and
are not silently included in the Nova publication table. If a later authority
requires either object on DA/IPFS, its exact selected framing must add those
bytes explicitly. The 572-byte receipt remains excluded because the live type
is deliberately write-only/in-memory and has no persistence or decoding path.

The mandatory Plan-09 proof-body retention target is bounded to 16 bodies and
`2 MiB`, with at most two bodies per epoch and eight pending PQ epochs; its
runtime ledger/GC path is not implemented by Plan 051. This body cap does not
include the public states/framing of full envelopes and does not replace the
90-day canonical challenge archive.

## DA and 90-day challenge archive

The DA payload is a small commitment envelope. Raw transaction packages, exact
transaction-proof bytes, replay bytes, justified non-derivable witness/delta
bytes, and recursive proof bodies are excluded from it. The exact challenge
bytes must live once in `EpochChallengePackV2` and be encoded with mandatory
RS(10,16), whose exact coding overhead is `16/10 = 1.6×`.

For average unique challenge bytes `D` per block:

```text
logical_90d = D * 1,555,200
rs_physical_90d = D * 1,555,200 * 1.6 = D * 2,488,320
```

| Unique bytes/block `D` | Logical 90-day ring | Full RS(10,16) network storage |
| ---: | ---: | ---: |
| 1 KiB | 1,592,524,800 B (`1.483 GiB`) | 2,548,039,680 B (`2.373 GiB`) |
| 10 KiB | 15,925,248,000 B (`14.832 GiB`) | 25,480,396,800 B (`23.730 GiB`) |
| 100 KiB | 159,252,480,000 B (`148.315 GiB`) | 254,803,968,000 B (`237.305 GiB`) |
| 1 MiB | 1,630,745,395,200 B (`1.483 TiB`) | 2,609,192,632,320 B (`2.373 TiB`) |

Equivalently, if `U` unique challenge bytes are generated per day:

- logical 90-day plateau: `90 * U`;
- all 16 RS shards across the network: `144 * U`;
- one shard placement: `9 * U`;
- maximum allowed in one failure domain (two shards): `18 * U`.

These are plateau sizes after the window is full. Legal holds and unresolved
disputes increase them because deletion must pause.

## IPFS/Kubo capacity

IPFS is valid only as `ipfs_pinned` with provider receipts and retrieval
audits. A bare or unpinned CID is rejected. Under the selected placement rule,
no failure domain may hold more than two of the 16 shards.

An archive/Kubo operator that stores one shard needs at least `9 * U` bytes for
the 90-day payload plateau; an operator storing the allowed maximum of two
shards needs `18 * U`. Add filesystem/blockstore metadata, pin metadata,
temporary repair space, and compaction headroom. Until real CAR/blockstore
measurements exist, capacity planning should reserve at least 25% operational
headroom without relabelling that reserve as protocol data.

The mandatory `EpochChallengePackV2`, RS shard codec,
`CheckpointArchiveAvailabilityManifestV2`, pinned-Kubo adapter, provider
receipts, and exact size instrumentation are not present in current code.
Therefore no exact IPFS disk number is currently an admissible measurement.

## Current state, snapshots, and permanent history

The mandatory Phase-069 current-state target declares 16 logical shards with
replication factor 3, write quorum 2, read quorum 1, and no full-state replica;
its runtime sharding/placement implementation is owned by Plan 09. Ignoring metadata
and transient COW/repair overhead, physical current-state storage is therefore
approximately `3 * S_state` across the replica set, not `48 * S_state`.

The mandatory downstream state-snapshot target is every 10,000 blocks with at
least the latest three generations retained. Exact snapshot chunk bytes have not yet been
measured, so their disk requirement is:

```text
snapshot_disk >= 3 * exact_snapshot_generation_bytes + repair/temp headroom
```

Permanent compact history is capped at `102,400 B/day`, or `9,216,000 B` per
90 days, excluding the current head and operator indexes. Epoch anchors are
capped at `4 KiB` each with a `1 KiB` target. Full per-block bodies are not
permanent history.

## Aggregator and role requirements

### What can be specified today

| Role | CPU | RAM | Local disk | Status |
| --- | --- | --- | --- | --- |
| Canonical aggregator without in-process Nova | Existing canonical runtime requirement plus one asynchronous bounded queue | Existing runtime requirement; Nova worker must not consume it | `3 * S_state` across replica set plus journals/snapshots | Canonical finality may continue while shadow evidence lags. |
| Dedicated Nova prototype worker | One in-flight prover; measured average `8.32` logical cores on this host. Reserve at least 12 dedicated modern cores for the measured prototype, but no tested core count meets five seconds. | `16 GiB` minimum from the 12-GiB authority ceiling plus OS headroom; `24 GiB` is the emergency harness ceiling, not production approval | `958,329,882 B` private material plus `15,372,615 B` verifier bundle, trace/replay spool, `2 * R`, atomic-replacement space, and job temp space | Does not meet five-second throughput. |
| Clean Nova verifier | One current generation-2 cold sample passed in `58.343 s`, only `1.657 s` below the 60-s budget; persistent caching and p95/p99 measurement remain necessary. | Kernel peak `3,348,504,576 B`, leaving `946,462,720 B` under 4 GiB; deploy with service/OS headroom, never at the bare cap. | bundle plus atomic replacement and logs; active bundle `15,372,615 B` | Meets the same-format single-sample authority ceiling; not yet a production SLO. |
| Challenge archive network | Encoding/repair/audit cores depend on measured `U` and repair SLA | Index/cache memory depends on pack/shard implementation | `144 * U` total RS plateau plus metadata/headroom | Codec and adapter are mandatory but unimplemented. |
| One Kubo failure domain | Retrieval/audit concurrency must meet 1000-block audit cadence | Kubo index/cache must be measured | at most `18 * U` payload plateau plus headroom | Pin/receipt/audit integration is unimplemented. |

For the Nova-only worker, two complete authority-artifact generations for
atomic replacement require `2 * 973,702,497 = 1,947,404,994 B`. A practical
prototype floor is therefore `4 GiB` of fast local disk before trace/replay,
logs, build outputs, or recovery snapshot `R`; `16 GiB` is the recommended
Nova-service allocation for those operational files, but is not a measured
protocol minimum. Canonical-state, DA, 90-day challenge, and Kubo storage must
be added with the formulas above rather than hidden inside that number.

### What cannot be specified honestly yet

There is no production aggregator requirement such as “N cores and M GiB meet
five-second Nova” because the measured sequential lane misses the deadline by
more than 40×. Production activation requires either a materially faster
relation/IVC implementation or an architecture that changes the dependency
model while preserving the same theorem. Hardware multiplication alone is not
an accepted remedy.

The current `CheckpointNovaRunnerV2` remains a private storage owner used by
tests. It is not yet integrated into the production aggregator scheduler,
admission queue, crash-recovery store, or backpressure policy. Plans 06 and 10
own those live integrations; this report does not claim they exist.

## Operating-cost formulas

Let:

- `C_disk` be the operator's price per GB-month;
- `C_egress` be the price per GB transferred;
- `P_avg` be measured average watts for the role;
- `C_kwh` be electricity price per kWh;
- `F` be configured network fanout;
- `B_da = 1,555,200 * S_da` be 90-day DA payload bytes;
- `B_archive = 2,488,320 * D` be full-network RS archive bytes;
- `B_nova` be retained Nova envelope/body bytes under the lifecycle state.

Then:

```text
90d_disk_cost = ((B_archive + B_nova) / 1e9) * 3 * C_disk
90d_da_egress_cost = (B_da / 1e9) * F * C_egress
90d_energy_cost = (P_avg / 1000) * 24 * 90 * C_kwh
```

PP/PK and VK distribution are generation events, not per-block payload. Account
for their egress separately as `bundle_bytes * recipients`. Current market
prices are intentionally not embedded in a source-bound engineering report;
operators can substitute their own tariff inputs.

## Production blockers after Plan 051

1. The sequential Nova fold lane must meet the five-second p95/p99 budget or be
   explicitly kept non-authoritative with a bounded backlog policy. It currently
   does not keep up.
2. The current proof body (`123,688 B`) is under the `128 KiB` hard cap by
   `7,384 B` but is more than four times the `30 KiB` target; optimization or a
   formally approved target change is required.
3. `max_nova_hot_recovery_bytes` is `0`; a positive finite cap cannot be
   activated until the real `R` snapshot size is implemented and measured.
4. `CheckpointDaPayloadV2` and its canonical finite codec are absent, so exact
   per-five-second DA bytes `S_da` are unknown.
5. `EpochChallengePackV2`, RS(10,16) shard encoding, V2 availability manifest,
   pinned-IPFS adapter, provider receipts, and retrieval-audit path are absent,
   so exact `D`, `U`, disk, repair traffic, and Kubo RAM are unknown.
6. The production aggregator scheduler/queue/restart path is not wired to the
   private Nova runner. Later-plan ownership does not make this live today.
7. The clean verifier passed the 60-second cold ceiling by only 1.657 seconds
   in one sample. Plan 11 must establish warm/cold p95/p99 behavior and the
   persistent generation-bound cache before this can become a production SLO.

These are mandatory live Phase-069 requirements owned by Plans 06–13, not
optional future ideas and not false implementation claims for Plan 051. The
format-4 verifier bundle no longer blocks Plan 051: it is below its hard cap.

## Evidence locations

- Mandatory post-fix active-candidate bootstrap:
  `crates/z00z_storage/outputs/checkpoint/069-051/final/bootstrap-receipt-guard-fix/`
- Current artifact corpus:
  `crates/z00z_storage/outputs/checkpoint/069-051/final/artifacts/source-1da-current.log`
  and `final/artifacts/source-1da-active-material/`.
- Current continuous T3 chain:
  `crates/z00z_storage/outputs/checkpoint/069-051/final/t3-chain-source-1da/`
- Canonical sidecar/receipt/claim layouts:
  `crates/z00z_storage/src/checkpoint/sidecar.rs`,
  `crates/z00z_storage/src/checkpoint/receipt.rs`, and
  `crates/z00z_storage/src/checkpoint/adapter.rs`.
- Current isolated verifier RSS/full-proof/Model-C:
  `crates/z00z_storage/outputs/checkpoint/069-051/final/verifier-rss-source-1da/`
  (`measurement/measurement.env` SHA-256
  `0bfaa954bd3b230ebeb69300cd2cf8d9023c89b812f35c1aad16eb94c35e418d`).
- Exact release-profiler commands and outputs:
  `crates/z00z_storage/outputs/checkpoint/069-051/final/profilers/`.
- Current T4 profile root:
  `crates/z00z_storage/outputs/checkpoint/069-051/final/`
- Host/tool capture and current source identities:
  `crates/z00z_storage/outputs/checkpoint/069-051/final/system/host-and-tools.txt`
  and `system/current-source-identities.txt`. The former is an immutable host
  snapshot and retains the source hash visible when it was captured; the latter
  is the authoritative current-source hash record.
- Prior benchmark ledger: `069-051-BENCHMARKS.md`
- Active authority decision: `069-051-AUTHORITY-OPERATING-BUDGET-DRAFT.md`
- Cadence/retention authority:
  `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`
- Live design authority: `069-TODO.md`

Every final table update must retain the command, terminal exit code, exact
source/worker/lock identities, `/usr/bin/time -v` output, and serialized object
bytes under `crates/z00z_storage/outputs/checkpoint`. No evidence may be written
to `test-results`.
