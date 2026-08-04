---
id: telemetry.aggregators.publication
title: アグリゲーター公開
summary: Ordered batch が checkpoint、quorum、DA、lifecycle evidence に結び付く方法を説明します。
scope: context
---
## この画面の使い方 {#current-view}
- `PublicationRequest` から `PublishedBatch`、`PublicationRecord` への流れを確認します。
- 利用不可は検証済み publication または readiness bundle が接続されていないことを示します。

## Fail-closed 境界
- Provider、height、manifest、payload、statement、evidence が不完全または不一致なら拒否されます。
- Storage が checkpoint root、proof、lifecycle truth を所有します。
