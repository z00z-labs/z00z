---
id: telemetry.aggregators.placement
title: Colocación del agregador
summary: Esta vista explica generación shard, propietario primario, estado de secundarios y linaje del journal del runtime.
scope: context
---
## Usar esta vista {#current-view}
- Revise `ShardPlacementView` sin deducir una topología global.
- No disponible significa que no hay una observación actual de la tabla de colocación.

## Límite fail-closed
- La tabla debe poseer exactamente el shard y la generación de enrutamiento.
- Los IDs de agregador son datos operativos; endpoints e identidades de cartera permanecen ocultos.
