# Scenario 2: hybrid Nova and Plonky3 capacity

Scenario 2 is a release-only, end-to-end capacity workload for the production
aggregator, HJMT settlement, checkpoint, Nova, and Plonky3 boundaries. The
canonical configuration runs 10 Plonky3 epochs of 2,000 blocks. Every block
contains 1,000 independently generated and fully verified regular
transactions, for a total of 20,000 blocks and 20,000,000 transactions.
All wallet, output, and proof randomness is derived from the configured master
seed so identical inputs remain reproducible across capacity runs.

The wallet route repeats `A -> B -> C -> B -> A`. One spendable coin is kept
per transaction lane. Every transaction resolves its current HJMT membership
proof, constructs recipient and fee outputs with balanced commitments, builds
the public spend contract, and passes full `TxPackage` verification before it
can enter aggregator ingress.

## Block pipeline

1. Resolve 1,000 live HJMT membership proofs.
2. Build and verify 1,000 full regular transaction packages in the bounded
   Rayon pool.
3. Normalize packages at aggregator ingress and create one deterministic
   `OrderedBatch` and `SettlementExecHandoff`.
4. Apply the handoff to the durable HJMT projection store and derive the exact
   post-state root.
5. Encode the ordered batch with `celestia-types` 1.0.0 as namespace blobs,
   sparse shares, a tail-padded original data square, a Leopard extended data
   square, and a `DataAvailabilityHeader`. Persist raw 512-byte EDS shares plus
   a manifest that binds the batch ID and HJMT pre/post roots, then reload the
   files and reconstruct every blob through the official Celestia types.
6. Persist and reload the prep snapshot, execution input, draft, archive
   manifest, DA reference, checkpoint link, and final artifact.
7. Submit the same handoff through the sole public recursive-checkpoint V2
   ingress. That ingress owns the authoritative live HJMT commit, one
   sequential Nova accumulator, and a linear capture of the exact canonical
   transition before Nova closes its private source.
8. Commit that capture to `EpochTransitionStreamV2`. Every eight transitions
   produce one bounded full linked-table direct-AIR work item. Prove and
   actual-verify it with `Plonky3EpochChunkWorkerV2`, admit it to the durable
   frontier, and consume all ready recursive merges sequentially.
9. Compare live and projection roots, verify both HJMT caches, validate the
   recursive height, and perform configured cold checkpoint/HJMT reloads.
10. At exactly 2,000 transitions, close and reload the immutable work manifest,
    reopen the durable frontier, seal and actual-verify the Plonky3 epoch proof,
    prove and verify the rolling history base/successor, construct the only
    publishable `EpochManifestV2`, then persist, reload, decode, and reverify
    every final artifact.

At recovery-aligned cycle boundaries the recursive evidence store is reopened.
The next block must therefore resume the same accumulator lineage from the
latest durable recovery snapshot instead of relying only on in-process state.

The authority-pinned cadence is used without overrides: Nova fold every block,
Nova recovery snapshot every 100 blocks, Nova compression/publication snapshot
every 1,000 blocks, and a full Plonky3 epoch every 2,000 blocks. A 2,000-block
epoch contains 250 direct-AIR chunks of eight transitions. Only one heavyweight
chunk proof is in flight, matching the current bounded-memory release path.

This epoch path is the production direct-AIR -> recursive frontier -> epoch
seal -> rolling history -> publishable manifest scheme. It does not substitute
the older standalone per-transition base-proof adapter for the full epoch
theorem.

## Profiling evidence

Every stage records wall time, process CPU ticks, RSS/high-water RSS, process
thread count, context switches, kernel-reported disk I/O, HJMT cache counters,
and HJMT scheduler pressure. Per-cycle evidence is written under `profile/`.
Nova ingress is classified as ordinary fold, recovery snapshot, or
compression/publication. Canonical Plonky3 transition preparation must occur
inside that same ingress before Nova closes the source, so this shared boundary
is named explicitly in profiles. Transition-stream commit, direct chunk
prove/verify/admit, frontier merge, epoch close/reopen, epoch seal/verify,
history prove/verify, and final manifest persist/reload are measured separately.
Nova's other internal trace/evaluation/fold steps remain one authority-owned
operation and are not falsely presented as independently timed.
Celestia blob/share encoding, EDS construction, atomic EDS persistence, and
disk reload plus DAH/blob verification are recorded as separate stages. These
are local Celestia-compatible artifacts; the scenario does not claim network
submission, PayForBlobs execution, consensus inclusion, or finality.
Successful completion also writes:

- `profile/profile_summary.json`: mean, p50, p95, p99, maximum, throughput, and
  system high-water measurements by stage.
- `minimum_aggregator_requirements.json`: observed resource floors plus the
  configured safety headroom and recursive authority bounds.
- `optimization_candidates.md`: measurement-ranked optimization targets that
  preserve deterministic ordering and the single Nova accumulator.
- `run_summary.json`: final block/transaction counts and terminal checkpoint,
  recursive, settlement, Plonky3 epoch, history, and manifest identities.
- `plonky3/epoch-NNNN/`: durable frontier state, work manifest, epoch proof,
  history proof, publishable epoch manifest, availability evidence, and an
  epoch summary for each 2,000-block cycle.

The live and projection stores deliberately coexist so every recursive commit
has an independently computed expected root. Therefore process RSS and total
run-directory disk are conservative compared with an aggregator that does not
retain the simulation-only projection mirror.

## Build and execution

Compilation does not execute the workload:

```text
cargo check -p z00z_simulator --release --bin scenario_2
```

An intentional run requires retained authority-matching `prover-material.bin`
and `verifier-bundle.bin` at the paths in `scenario_config.yaml`:

```text
cargo run -p z00z_simulator --release --bin scenario_2
```

Debug execution is rejected. Workload, file-size, thread, directory-scan, and
profiling bounds are validated before the run directory is created.
