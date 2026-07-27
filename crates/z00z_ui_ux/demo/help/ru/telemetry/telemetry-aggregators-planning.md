---
id: telemetry.aggregators.planning
title: Планирование агрегатора
summary: Экран объясняет детерминированную привязку batch и shard route без заявления settlement authority.
scope: context
---
## Как использовать экран {#current-view}
- Проверяйте planner mode, generation маршрута, число intake и операций, а также владельцев digest.
- Недоступно означает, что проверенный снимок `BatchPlanned` не подключён.

## Fail-closed граница
- Конфигурация, generation, route-table digest и пересчитанный plan должны совпадать.
- Планирование не финализирует settlement, publication или storage truth.
