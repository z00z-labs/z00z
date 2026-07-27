---
id: telemetry.aggregators.planning
title: Aggregator planning
summary: Planning explains deterministic batch and shard-route binding without claiming settlement authority.
scope: context
---
## Use this view {#current-view}
- Review planner mode, route generation, intake count, operation count, and digest ownership.
- Unavailable means no verified `BatchPlanned` snapshot is connected.

## Fail-closed boundary
- Planner configuration, generation, route-table digest, and recomputed plan must agree.
- Planning never finalizes settlement, publication, or storage truth.
