---
id: telemetry.aggregators.recovery
title: Aggregator recovery
summary: Recovery explains restart and secondary-takeover checks against committed route, generation, primary, and journal lineage.
scope: context
---
## Use this view {#current-view}
- Review the `ShardRecoveryRecord`, recovery intent, durable state, and execution-ticket contract.
- Unavailable means no committed recovery snapshot is connected.

## Fail-closed boundary
- Wrong generation, primary, shard, batch, route, or lineage is rejected.
- The renderer cannot initiate failover or mutate storage recovery truth.
