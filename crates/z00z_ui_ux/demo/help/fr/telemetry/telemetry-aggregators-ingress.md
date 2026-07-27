---
id: telemetry.aggregators.ingress
title: Entrée de l’agrégateur
summary: Cette vue explique comment le runtime admet une transaction ou une réclamation comme travail lié à un digest.
scope: context
---
## Utiliser cette vue {#current-view}
- Vérifiez le contrat `WorkPayload` vers `WorkItem` ou `RejectRecord`.
- Indisponible signifie qu’aucun instantané d’admission récent n’existe, pas que le travail est accepté ou rejeté.

## Limite fail-closed
- La liaison d’un object package modifie le digest d’admission et l’identité d’entrée.
- Payloads bruts, destinataires, mémos et routes locales du portefeuille restent hors de l’aide.
