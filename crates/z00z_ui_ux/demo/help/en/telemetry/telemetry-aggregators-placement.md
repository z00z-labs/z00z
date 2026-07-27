---
id: telemetry.aggregators.placement
title: Aggregator placement
summary: Placement explains the runtime-owned shard generation, primary owner, secondary readiness, and journal lineage view.
scope: context
---
## Use this view {#current-view}
- Review the `ShardPlacementView` contract without inferring global topology.
- Unavailable means no current placement-table observation is connected.

## Fail-closed boundary
- The placement table must own the exact shard and routing generation.
- Aggregator IDs are operational data; endpoints and wallet identities stay hidden.
