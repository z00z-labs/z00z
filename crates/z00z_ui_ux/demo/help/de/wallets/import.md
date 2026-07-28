---
id: wallet.import
title: Importieren
summary: Importieren prüft ein öffentliches Asset-Paket von der Festplatte, bevor es an die aktive Wallet übergeben wird.
scope: context
---
## Diese Ansicht verwenden {#current-view}
- Wählen Sie ein `AssetPkgWire`-JSON-Paket mit höchstens 64 KiB.
- Prüfen Sie Wallet, Netzwerk, Klasse, Betrag, Serien-ID, Domain, Statusflags und Eigentümerbindung.
- Wählen Sie **Asset importieren**; die native Wallet prüft Kryptografie, Eigentum, Replay und Claim-Konflikte.

## Lokales und sicheres Verhalten
- Das Feld `secret` ist verboten; der absolute Dateipfad wird weder gespeichert noch an RPC gesendet.
- Das Ergebnis unterscheidet neuen Import, vorhandenes Asset und einen eindeutigen `IMPORT_*`-Grund.
