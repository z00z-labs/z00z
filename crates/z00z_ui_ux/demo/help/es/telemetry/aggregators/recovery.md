---
id: telemetry.aggregators.recovery
title: Recuperación del agregador
summary: Esta vista explica controles de reinicio y toma secundaria contra ruta, generación, primario y linaje confirmados.
scope: context
---
## Usar esta vista {#current-view}
- Revise `ShardRecoveryRecord`, intención, estado durable y ticket de ejecución.
- No disponible significa que no hay snapshot de recuperación confirmado conectado.

## Límite fail-closed
- Generación, primario, shard, batch, ruta o linaje incorrectos se rechazan.
- El renderer no puede iniciar failover ni modificar la verdad de recuperación de storage.
