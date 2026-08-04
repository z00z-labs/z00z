---
id: telemetry.aggregators.publication
title: Aggregator-Publikation
summary: Diese Ansicht erklärt die Bindung eines geordneten Batch an Checkpoint-, Quorum-, DA- und Lifecycle-Nachweise.
scope: context
---
## Diese Ansicht verwenden {#current-view}
- Folgen Sie `PublicationRequest` zu `PublishedBatch` und `PublicationRecord`.
- Nicht verfügbar bedeutet, dass keine verifizierte Publikation oder Readiness-Bundle verbunden ist.

## Fail-closed-Grenze
- Unvollständige oder abweichende Provider-, Höhen-, Manifest-, Payload-, Statement- oder Evidence-Daten werden abgelehnt.
- Storage besitzt Checkpoint-Wurzeln, Beweise und Lifecycle-Wahrheit.
