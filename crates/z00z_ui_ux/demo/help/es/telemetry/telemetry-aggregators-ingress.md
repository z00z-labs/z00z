---
id: telemetry.aggregators.ingress
title: Entrada del agregador
summary: Esta vista explica cómo el runtime admite una transacción o reclamación como trabajo ligado a un digest.
scope: context
---
## Usar esta vista {#current-view}
- Revise el contrato `WorkPayload` hacia `WorkItem` o `RejectRecord`.
- No disponible significa que no hay snapshot reciente de admisión, no que fue aceptado o rechazado.

## Límite fail-closed
- Vincular un object package cambia el digest de admisión y la identidad de entrada.
- Payloads sin filtrar, receptores, notas y rutas locales de cartera no entran en la ayuda.
