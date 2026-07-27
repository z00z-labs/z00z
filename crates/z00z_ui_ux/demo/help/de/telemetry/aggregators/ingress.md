---
id: telemetry.aggregators.ingress
title: Aggregator-Eingang
summary: Diese Ansicht erklärt, wie die Runtime eine Transaktion oder Claim-Nutzlast als digest-gebundenes WorkItem zulässt.
scope: context
---
## Diese Ansicht verwenden {#current-view}
- Prüfen Sie den Vertrag `WorkPayload` zu `WorkItem` oder `RejectRecord`.
- Nicht verfügbar bedeutet, dass kein aktueller Admission-Snapshot vorliegt, nicht Annahme oder Ablehnung.

## Fail-closed-Grenze
- Das Binden eines Object Package ändert Admission-Digest und Intake-Identität.
- Rohe Payloads, Empfänger, Memos und lokale Wallet-Routen bleiben außerhalb der Hilfe.
