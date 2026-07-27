---
id: telemetry.aggregators.placement
title: Placement de l’agrégateur
summary: Cette vue explique la génération shard, le primaire, l’état des secondaires et la lignée de journal détenus par le runtime.
scope: context
---
## Utiliser cette vue {#current-view}
- Vérifiez `ShardPlacementView` sans déduire une topologie globale.
- Indisponible signifie qu’aucune observation actuelle de la table de placement n’est connectée.

## Limite fail-closed
- La table doit posséder exactement le shard et la génération de routage.
- Les IDs d’agrégateur sont opérationnels ; endpoints et identités du portefeuille restent cachés.
