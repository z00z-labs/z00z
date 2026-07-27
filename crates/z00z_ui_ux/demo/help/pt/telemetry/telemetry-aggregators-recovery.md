---
id: telemetry.aggregators.recovery
title: Recuperação do agregador
summary: Esta vista explica verificações de reinício e takeover secundário contra rota, geração, primário e linhagem comprometidos.
scope: context
---
## Utilizar esta vista {#current-view}
- Verifique `ShardRecoveryRecord`, intenção, estado durável e ticket de execução.
- Indisponível significa que nenhum snapshot de recuperação comprometido está ligado.

## Limite fail-closed
- Geração, primário, shard, batch, rota ou linhagem incorretos são rejeitados.
- O renderer não pode iniciar failover ou alterar a verdade de recuperação do storage.
