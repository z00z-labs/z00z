---
id: telemetry.aggregators.ingress
title: 聚合器入口
summary: 此视图说明运行时如何将交易或 claim payload 接纳为绑定 digest 的工作项。
scope: context
---
## 使用此视图 {#current-view}
- 查看从 `WorkPayload` 到 `WorkItem` 或 `RejectRecord` 的契约。
- 不可用表示没有新的 admission snapshot，并不表示已接纳或拒绝。

## Fail-closed 边界
- 绑定 object package 会改变 admission digest 和 intake identity。
- Raw payload、接收方、备注和钱包本地路由不会进入帮助内容。
