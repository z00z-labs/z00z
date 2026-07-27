---
id: telemetry.aggregators.placement
title: Размещение агрегатора
summary: Экран объясняет shard generation, primary owner, готовность secondary и journal lineage, которыми владеет runtime.
scope: context
---
## Как использовать экран {#current-view}
- Проверяйте контракт `ShardPlacementView`, не делая выводов о глобальной топологии.
- Недоступно означает отсутствие текущего наблюдения placement table.

## Fail-closed граница
- Placement table должна владеть точным shard и routing generation.
- Aggregator ID — операционные данные; endpoints и identity кошелька скрыты.
