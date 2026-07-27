---
id: telemetry.aggregators.recovery
title: Récupération de l’agrégateur
summary: Cette vue explique les contrôles de redémarrage et reprise secondaire sur route, génération, primaire et lignée engagés.
scope: context
---
## Utiliser cette vue {#current-view}
- Vérifiez `ShardRecoveryRecord`, l’intention, l’état durable et le ticket d’exécution.
- Indisponible signifie qu’aucun instantané de récupération engagé n’est connecté.

## Limite fail-closed
- Génération, primaire, shard, batch, route ou lignée incorrects sont rejetés.
- Le renderer ne peut ni déclencher le failover ni modifier la vérité de récupération du stockage.
