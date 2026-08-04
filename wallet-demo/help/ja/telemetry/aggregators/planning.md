---
id: telemetry.aggregators.planning
title: アグリゲーター計画
summary: Settlement 権限を主張せず、決定論的な batch と shard route の結合を説明します。
scope: context
---
## この画面の使い方 {#current-view}
- Planner mode、route generation、intake と operation の数、digest の所有者を確認します。
- 利用不可は検証済み `BatchPlanned` snapshot が接続されていないことを示します。

## Fail-closed 境界
- 設定、generation、route-table digest、再計算した plan は一致する必要があります。
- 計画は settlement、publication、storage truth を確定しません。
