---
id: telemetry.aggregators.recovery
title: Восстановление агрегатора
summary: Экран объясняет проверки restart и secondary takeover по committed route, generation, primary и journal lineage.
scope: context
---
## Как использовать экран {#current-view}
- Проверяйте `ShardRecoveryRecord`, recovery intent, durable state и execution ticket.
- Недоступно означает отсутствие подключённого committed recovery snapshot.

## Fail-closed граница
- Неверные generation, primary, shard, batch, route или lineage отклоняются.
- Renderer не может запускать failover или изменять storage recovery truth.
