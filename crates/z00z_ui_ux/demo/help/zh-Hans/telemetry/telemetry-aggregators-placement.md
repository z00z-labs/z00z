---
id: telemetry.aggregators.placement
title: 聚合器放置
summary: 此视图说明运行时拥有的 shard generation、primary、secondary readiness 和 journal lineage。
scope: context
---
## 使用此视图 {#current-view}
- 查看 `ShardPlacementView` 契约，不要推断全局拓扑。
- 不可用表示未连接当前 placement table 观测。

## Fail-closed 边界
- Table 必须拥有精确的 shard 和 routing generation。
- Aggregator ID 是运行数据；endpoint 和钱包 identity 保持隐藏。
