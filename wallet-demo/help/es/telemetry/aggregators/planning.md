---
id: telemetry.aggregators.planning
title: Planificación del agregador
summary: Esta vista explica la vinculación determinista de batch y ruta shard sin afirmar autoridad de liquidación.
scope: context
---
## Usar esta vista {#current-view}
- Revise modo, generación de ruta, recuentos de entradas y operaciones, y propiedad de digests.
- No disponible significa que no hay un snapshot `BatchPlanned` verificado conectado.

## Límite fail-closed
- Configuración, generación, digest de tabla de rutas y plan recalculado deben coincidir.
- La planificación no finaliza liquidación, publicación ni verdad de almacenamiento.
