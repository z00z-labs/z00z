---
id: telemetry.aggregators.planning
title: Planification de l’agrégateur
summary: Cette vue explique la liaison déterministe du batch et de la route shard sans revendiquer l’autorité de règlement.
scope: context
---
## Utiliser cette vue {#current-view}
- Vérifiez le mode, la génération de route, les nombres d’entrées et d’opérations, et les digests.
- Indisponible signifie qu’aucun instantané `BatchPlanned` vérifié n’est connecté.

## Limite fail-closed
- Configuration, génération, digest de table de routes et plan recalculé doivent correspondre.
- La planification ne finalise ni règlement, ni publication, ni vérité de stockage.
