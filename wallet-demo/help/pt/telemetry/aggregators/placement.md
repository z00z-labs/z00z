---
id: telemetry.aggregators.placement
title: Colocação do agregador
summary: Esta vista explica geração shard, proprietário primário, prontidão dos secundários e linhagem do journal do runtime.
scope: context
---
## Utilizar esta vista {#current-view}
- Verifique `ShardPlacementView` sem inferir uma topologia global.
- Indisponível significa que nenhuma observação atual da tabela de colocação está ligada.

## Limite fail-closed
- A tabela deve possuir exatamente o shard e a geração de routing.
- IDs de agregador são dados operacionais; endpoints e identidades da carteira ficam ocultos.
