---
id: telemetry.aggregators.recovery
title: 聚合器恢复
summary: 此视图说明针对已提交 route、generation、primary 和 journal lineage 的重启与 secondary takeover 检查。
scope: context
---
## 使用此视图 {#current-view}
- 查看 `ShardRecoveryRecord`、recovery intent、durable state 和 execution ticket。
- 不可用表示未连接已提交的 recovery snapshot。

## Fail-closed 边界
- Generation、primary、shard、batch、route 或 lineage 错误时会被拒绝。
- Renderer 不能启动 failover 或修改 Storage recovery truth。
