---
id: telemetry.aggregators.placement
title: Aggregator-Platzierung
summary: Diese Ansicht erklärt Shard-Generation, Primärbesitz, Sekundärbereitschaft und Journal-Lineage der Runtime.
scope: context
---
## Diese Ansicht verwenden {#current-view}
- Prüfen Sie `ShardPlacementView`, ohne eine globale Topologie abzuleiten.
- Nicht verfügbar bedeutet, dass keine aktuelle Placement-Table-Beobachtung verbunden ist.

## Fail-closed-Grenze
- Die Tabelle muss exakt den Shard und die Routing-Generation besitzen.
- Aggregator-IDs sind Betriebsdaten; Endpoints und Wallet-Identitäten bleiben verborgen.
