---
id: telemetry.aggregators.publication
title: 聚合器发布
summary: 此视图说明 ordered batch 如何绑定 checkpoint、quorum、DA 和 lifecycle evidence。
scope: context
---
## 使用此视图 {#current-view}
- 查看 `PublicationRequest` 到 `PublishedBatch` 和 `PublicationRecord` 的流程。
- 不可用表示未连接经过验证的 publication 或 readiness bundle。

## Fail-closed 边界
- Provider、height、manifest、payload、statement 或 evidence 不完整或不一致时会被拒绝。
- Storage 拥有 checkpoint root、proof 和 lifecycle truth。
