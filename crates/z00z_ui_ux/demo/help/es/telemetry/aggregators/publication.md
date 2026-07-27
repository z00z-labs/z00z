---
id: telemetry.aggregators.publication
title: Publicación del agregador
summary: Esta vista explica cómo un batch ordenado se vincula a checkpoint, quorum, DA y evidencia de ciclo de vida.
scope: context
---
## Usar esta vista {#current-view}
- Siga `PublicationRequest` hacia `PublishedBatch` y `PublicationRecord`.
- No disponible significa que no hay publicación o paquete de readiness verificado.

## Límite fail-closed
- Datos incompletos o divergentes de proveedor, altura, manifiesto, payload, statement o evidence se rechazan.
- Storage es autoridad de raíces, pruebas y ciclo de vida del checkpoint.
