# z00z_runtime aggregators

This crate is the runtime-owned planning, placement, and publication surface.
Runtime owns bind/publish, storage owns settlement roots plus proof and recovery
truth, and rollup consumes only the final theorem bundle. No downstream crate
may fork a second authority path.

## Canonical modules

- `ingress`: recompute payload-bound digests for `TxPackage` and `ClaimTxPackage`, reject forged `tx_digest_hex` metadata, and emit the only planner-ready `WorkItem` type.
- `batch_planner`: route verified `WorkItem` input, apply the live `ShardRouteTable`, enforce the exact `ShardRouteTableV1` byte contract, and build `BatchPlanned` or `OrderedBatch`. The current runtime wave stays single-shard per batch.
- `placement`: operational shard ownership via `AggregatorId`, `ShardPlacement`, `ShardPlacementTable`, `ShardPlacementView`, and secondary metadata.
- `consensus_adapter`: deterministic local quorum, term, and membership seam over real `ShardRecoveryRecord` and `SettlementRecoveryState` inputs. External replicated-log or discovery systems stay adapter-only until this seam is proven locally.
- `dist_dispatch`: local route-rollout activation, owner-bound remote dispatch, process restart fencing, storage-lock hazards, and advisory runtime notes over real planner and placement inputs.
- `dist_scheduler`: deterministic scheduler waves over shard-owned workers. It groups planner-ready work by live shard ownership and keeps durable throughput claims scoped to published roots rather than scheduler fanout.
- `dist_sim`: deterministic local multi-aggregator simulator for journal replication, secondary catch-up, delay/drop/replay/partition faults, and fail-closed replay fencing.
- `shard_exec`: turn planned batches into `ShardExecTicket` lifecycle states. These states are runtime metadata, not verifier-visible truth.
- `service`: `AggregatorIngress`, `AggregatorOrdering`, `AggregatorRecovery`, and `AggregatorService` are the live boundary traits. `AggregatorService::bind_exec_handoff(...)` is the only runtime-owned path that may bind one ordered batch to `SettlementExecHandoff`, and `bind_publication_contract(...)` is the only runtime-owned path that may derive `PublicationBinding` from route metadata plus `CheckpointPubIn`.
- `ordering`, `recovery`, `scheduler`, and `types`: narrow runtime helpers and DTOs re-exported from `src/lib.rs`. `scheduler` stays a thin facade over `dist_scheduler` so there is one live scheduler path.

## Boundaries

- Planner authority lives here, not in storage, validators, watchers, or the rollup node.
- `PlannerMode` vocabulary is runtime-owned here and reused by config loaders instead of being redefined downstream.
- `PlannerAuthority` plus `BatchPlanner` are the only live planner-authority path; planner primary or secondary HA remains `live-claim-removed` until a separate durable service exists and is tested.
- Caller-supplied digest strings are never planner authority; runtime ingress must rebind them to payload bytes before route lookup, intake ids, or `plan_digest` construction.
- Placement data may flow downstream, but it must not become checkpoint authority or semantic state.
- Local distributed HJMT evidence must run through `dist_scheduler`, `dist_dispatch`, `dist_sim`, and `consensus_adapter`; external transport, network membership, or chain-facing replicated-log bindings must stay adapter-only exclusions until they satisfy the same local contract.
- Validators, watchers, and simulator traces must reuse the runtime-owned `PublicationBinding`, and they must consume storage-owned `PublicationRouteSnapshotV1` when exact shard-set publication coverage is required. They must not fork a second publication digest or binding-construction path. They must not fork a second route-table acceptance contract.
- Runtime may forward only semantic `StoreOp` work plus committed route metadata through `SettlementExecHandoff`; it must not invent subtree ids, backend tree inventory, or proof truth.
- Runtime notes for rollout, scheduler waves, stalls, freeze, disputes, drift, failover, and storage-lock hazards stay advisory observability only. They must never become proof, checkpoint, or consensus truth.
- This crate does not own settlement semantics, proof verification, or rollup orchestration.

## Recovery and failover

- `RecoveryBoundary` and `ShardRecoveryRecord` are the only runtime recovery export for restart or secondary-aggregator takeover.
- `ShardPlacement.expected_journal_lineage` binds runtime placement to the live durable journal lineage exported by storage.
- Same-lineage secondary-aggregator takeover is lawful only when shard id, routing generation, journal lineage, and live local root metadata all match.
- Wrong lineage, wrong generation, stale local root, stale restart, secondary aggregator down, and split-brain states reject fail-closed without silent reroute.
- Deterministic local distributed simulation is live scope: delay, drop, replay, partition, heal, stale-lineage, and quorum-term conflicts must be proven against real recovery records before any external adapter claim is treated as live.
- Route rollout, remote dispatch, duplicate or reordered delivery, owner restart, and storage-lock hazards must be proven locally against the live route table and shard ownership rules before any external transport claim is treated as live.
- A shared cross-aggregator WAL is not runtime or protocol truth in v1; the current local durable journal remains the only durability baseline behind storage `JournalBackend`.
