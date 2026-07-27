---
id: telemetry.aggregators.publication
title: Publication de l’agrégateur
summary: Cette vue explique la liaison d’un batch ordonné au checkpoint, quorum, DA et preuves de cycle de vie.
scope: context
---
## Utiliser cette vue {#current-view}
- Suivez `PublicationRequest` vers `PublishedBatch` et `PublicationRecord`.
- Indisponible signifie qu’aucune publication ou bundle de disponibilité vérifié n’est connecté.

## Limite fail-closed
- Provider, hauteur, manifeste, payload, statement et evidence incomplets ou divergents sont rejetés.
- Le stockage détient les racines, preuves et la vérité du cycle de vie.
