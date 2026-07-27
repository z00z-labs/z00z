---
id: telemetry.aggregators.recovery
title: アグリゲーター復旧
summary: 確定した route、generation、primary、journal lineage に対する再起動と secondary takeover の検査を説明します。
scope: context
---
## この画面の使い方 {#current-view}
- `ShardRecoveryRecord`、recovery intent、durable state、execution ticket を確認します。
- 利用不可は確定済み recovery snapshot が接続されていないことを示します。

## Fail-closed 境界
- Generation、primary、shard、batch、route、lineage が誤っていれば拒否されます。
- Renderer は failover を開始したり Storage の recovery truth を変更したりできません。
