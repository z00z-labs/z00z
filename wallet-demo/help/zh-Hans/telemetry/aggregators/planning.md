---
id: telemetry.aggregators.planning
title: 聚合器规划
summary: 此视图说明确定性的 batch 和 shard route 绑定，而不声称拥有 settlement 权限。
scope: context
---
## 使用此视图 {#current-view}
- 查看 planner mode、route generation、intake 和 operation 数量以及 digest 所有权。
- 不可用表示未连接经过验证的 `BatchPlanned` snapshot。

## Fail-closed 边界
- 配置、generation、route-table digest 和重新计算的 plan 必须一致。
- 规划不会最终确定 settlement、publication 或 storage truth。
