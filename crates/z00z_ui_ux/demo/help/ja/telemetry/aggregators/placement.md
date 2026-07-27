---
id: telemetry.aggregators.placement
title: アグリゲーター配置
summary: Runtime が所有する shard generation、primary、secondary readiness、journal lineage を説明します。
scope: context
---
## この画面の使い方 {#current-view}
- グローバルなトポロジーを推測せず `ShardPlacementView` 契約を確認します。
- 利用不可は現在の placement table 観測が接続されていないことを示します。

## Fail-closed 境界
- Table は正確な shard と routing generation を所有する必要があります。
- Aggregator ID は運用データで、endpoint とウォレット identity は非表示です。
