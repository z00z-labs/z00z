---
id: telemetry.aggregators.planning
title: Aggregator-Planung
summary: Diese Ansicht erklärt deterministische Batch- und Shard-Routenbindung ohne Settlement-Autorität zu behaupten.
scope: context
---
## Diese Ansicht verwenden {#current-view}
- Prüfen Sie Modus, Routengeneration, Intake- und Operationsanzahl sowie Digest-Besitz.
- Nicht verfügbar bedeutet, dass kein verifizierter `BatchPlanned`-Snapshot verbunden ist.

## Fail-closed-Grenze
- Konfiguration, Generation, Routentabellen-Digest und neu berechneter Plan müssen übereinstimmen.
- Planung finalisiert weder Settlement noch Publikation oder Storage-Wahrheit.
