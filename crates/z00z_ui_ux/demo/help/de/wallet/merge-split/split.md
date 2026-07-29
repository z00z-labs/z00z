---
id: wallet.split
title: "Wallet: Aufteilen"
route: wallet.merge-split
scope: context
---

# Wallet: Aufteilen

[TOC]

## App-Ansicht {#current-view}

![Wallet-Ansicht zum Aufteilen](help/assets/en/wallet-split.png)

Dieses Bild wurde aus der aktuellen Aufteilen-Ansicht der Demo aufgenommen.

## Überblick {#overview}

Aufteilen verbraucht ein vertrauliches Asset-Fragment und bereitet zwei oder mehr Ausgaben vor. Jede Ausgabe behält die `definition_id` und die Basis-`serial_id` der Quelle; alle positiven Ausgabebeträge müssen exakt den Eingabebetrag ergeben. Die Anordnung der Ausgaben ändert sich, nicht aber die Asset-Definition oder das Angebot.

Jedes entstehende Fragment erhält eine eigene konkrete Ausgabeidentität und bleibt zugleich Teil derselben Ausgabeserie.

## Diese Ansicht verwenden {#how-to-use-this-view}

1. Prüfen Sie das aktive Wallet und das Netzwerk im App-Kopf.
2. Wählen Sie **Aufteilen**.
3. Wählen Sie ein verfügbares Quellfragment.
4. Geben Sie zwischen zwei und acht positive Ausgabebeträge ein.
5. Prüfen Sie, dass **Erhaltung** den Wert **Exakt** anzeigt.
6. Wählen Sie **Vorschau der Aufteilung** und prüfen Sie die Quelle sowie jede vorgeschlagene Ausgabe.
7. Fahren Sie nur in einem nativen Wallet fort, das Autorisierung, Gebühren, Übermittlung und Abgleich erneut prüfen kann.

## Begriffe und Steuerelemente {#terms-and-controls}

| Begriff oder Steuerelement | Erklärung |
| --- | --- |
| Quell-Asset | Das einzelne verfügbare Fragment, das von der vorgeschlagenen Aufteilung verbraucht wird. |
| Definitions-ID | Unveränderliche Kennung des Asset-Typs und seiner Richtlinie. Jede Ausgabe behält die Quelldefinition. |
| Serien-ID | Basis-Ausgabeserie. Jede Ausgabe behält die Serie der Quelle. |
| Ausgabenzuweisung | Zwischen zwei und acht positive Beträge für die vorgeschlagenen Ausgaben. |
| Erhaltung | Exakte Gleichheit zwischen Eingabebetrag und Summe aller Ausgabebeträge. |
| Ausgabe hinzufügen | Fügt bis zur Obergrenze der Oberfläche ein weiteres Feld für einen positiven Betrag hinzu. |
| Vorschau der Aufteilung | Nur zur Prüfung bestimmte Absicht mit Quelle und vorgeschlagenen Ausgaben; sie signiert oder übermittelt nichts. |

## Sicherheit und Grenzen {#safety-and-limits}

- Aufteilen ändert niemals die Quelldefinition oder die Basisserie.
- Nullbeträge, negative, übermäßige oder nicht erhaltende Zuweisungen müssen abgelehnt werden.
- Das native Wallet muss eine inzwischen gesperrte, ausgegebene, eingefrorene, verbrannte, sanktionierte oder anderweitig nicht verfügbare Quelle ablehnen.
- Wiederholte oder ungewöhnlich gemusterte Zuweisungen können die Korrelation zusammengehöriger Ausgaben erleichtern.
- Die JavaScript-Demo verwendet öffentliche Testdaten und endet bei der Vorschau. Sie hält keine Schlüssel, weist kein Eigentum nach, erstellt keine Signaturen, berechnet keine Gebühren, übermittelt kein Paket und gleicht kein unsicheres Ergebnis ab.
- Der aktuelle Helfer `wallet.asset.split_asset` ist eine Kompatibilitätsoberfläche und beansprucht keine kanonische Ledger-Abgleichsautorität. Die native Integration muss die Bestätigung über den maßgeblichen Wallet-Transaktionspfad führen.

<!-- help-sync:source {"page_path":"wallet/merge-split/split.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-split.png","topic_id":"wallet.split"} -->
