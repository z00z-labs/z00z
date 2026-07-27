---
id: telemetry.aggregators.planning
title: Planeamento do agregador
summary: Esta vista explica a ligação determinística de batch e rota shard sem reivindicar autoridade de settlement.
scope: context
---
## Utilizar esta vista {#current-view}
- Verifique modo, geração da rota, contagens de entradas e operações e propriedade dos digests.
- Indisponível significa que nenhum snapshot `BatchPlanned` verificado está ligado.

## Limite fail-closed
- Configuração, geração, digest da tabela de rotas e plano recalculado devem coincidir.
- O planeamento não finaliza settlement, publicação ou verdade de armazenamento.
