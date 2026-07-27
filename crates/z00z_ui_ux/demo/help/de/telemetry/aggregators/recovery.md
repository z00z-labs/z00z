---
id: telemetry.aggregators.recovery
title: Aggregator-Wiederherstellung
summary: Diese Ansicht erklärt Neustart- und Secondary-Takeover-Prüfungen gegen gebundene Route, Generation, Primärbesitz und Journal-Lineage.
scope: context
---
## Diese Ansicht verwenden {#current-view}
- Prüfen Sie `ShardRecoveryRecord`, Intent, dauerhaften Zustand und Ausführungsticket.
- Nicht verfügbar bedeutet, dass kein gebundener Recovery-Snapshot verbunden ist.

## Fail-closed-Grenze
- Falsche Generation, Primary, Shard, Batch, Route oder Lineage werden abgelehnt.
- Der Renderer kann kein Failover starten oder die Recovery-Wahrheit des Storage ändern.
