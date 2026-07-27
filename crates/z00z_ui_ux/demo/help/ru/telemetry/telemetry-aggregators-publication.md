---
id: telemetry.aggregators.publication
title: Публикация агрегатора
summary: Экран объясняет привязку ordered batch к checkpoint, quorum, data availability и lifecycle evidence.
scope: context
---
## Как использовать экран {#current-view}
- Следуйте цепочке `PublicationRequest` → `PublishedBatch` → `PublicationRecord`.
- Недоступно означает отсутствие проверенной публикации или readiness bundle.

## Fail-closed граница
- Неполные или несовпадающие provider, height, manifest, payload, statement и evidence отклоняются.
- Storage владеет checkpoint roots, proofs и lifecycle truth.
