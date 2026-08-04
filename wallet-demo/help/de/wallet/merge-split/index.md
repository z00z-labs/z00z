---
id: wallet.merge
title: "Wallet: Zusammenführen"
route: wallet.merge-split
scope: context
---

# Wallet: Zusammenführen

[TOC]

## App-Ansicht {#current-view}

![Wallet-Ansicht zum Zusammenführen](help/assets/en/wallet-merge.png)

Dieses Bild wurde aus der aktuellen Zusammenführen-Ansicht der Demo aufgenommen.

## Überblick {#overview}

Zusammenführen kombiniert zwei oder mehr kompatible vertrauliche Asset-Fragmente zu einer Ausgabe. Die Ausgabe behält dieselbe `definition_id` und dieselbe Basis-`serial_id`; ihr Betrag entspricht der Summe der ausgewählten Eingaben. Die Anordnung der Ausgaben ändert sich, nicht aber die Asset-Definition oder das Angebot.

Kandidaten werden nach Definition und Serie gruppiert. Fragmente aus unterschiedlichen Gruppen können nicht zusammengeführt werden, auch wenn sie dasselbe Anzeigesymbol verwenden.

## Diese Ansicht verwenden {#how-to-use-this-view}

1. Prüfen Sie das aktive Wallet und das Netzwerk im App-Kopf.
2. Wählen Sie **Zusammenführen**.
3. Wählen Sie mindestens zwei verfügbare Fragmente aus einer Kompatibilitätsgruppe.
4. Prüfen Sie die Anzahl der Eingaben, den Gesamtausgabebetrag, die Definition und die Serie.
5. Wählen Sie **Vorschau der Zusammenführung** und prüfen Sie jede Eingabe sowie die vorgeschlagene Einzelausgabe.
6. Fahren Sie nur in einem nativen Wallet fort, das Autorisierung, Gebühren, Übermittlung und Abgleich erneut prüfen kann.

## Begriffe und Steuerelemente {#terms-and-controls}

| Begriff oder Steuerelement | Erklärung |
| --- | --- |
| Definitions-ID | Unveränderliche Kennung des Asset-Typs und seiner Richtlinie. Alle ausgewählten Eingaben müssen sie teilen. |
| Serien-ID | Basis-Ausgabeserie. Alle Eingaben und die zusammengeführte Ausgabe behalten dieselbe Serie. |
| Asset-ID | Kennung einer konkreten vertraulichen Ausgabe. Kompatible Fragmente dürfen unterschiedliche Asset-IDs haben. |
| Kompatibilitätsgruppe | Verfügbare Fragmente mit derselben Definitions-ID und Serien-ID. |
| Gesperrt | Das Fragment ist als Kontext sichtbar, kann aber nicht ausgewählt werden. |
| Gesamtausgabe | Exakte Summe der ausgewählten Eingaben vor einer separaten nativen Gebührenrichtlinie. |
| Vorschau der Zusammenführung | Nur zur Prüfung bestimmte Absicht mit Eingaben und vorgeschlagener Ausgabe; sie signiert oder übermittelt nichts. |

## Sicherheit und Grenzen {#safety-and-limits}

- Diese Oberfläche führt niemals unterschiedliche Definitionen oder Basisserien zusammen.
- Das native Wallet muss gesperrte, ausgegebene, eingefrorene, verbrannte, sanktionierte oder anderweitig nicht verfügbare Eingaben ablehnen, selbst wenn eine veraltete Ansicht sie zuvor angezeigt hat.
- Das Zusammenführen von Fragmenten kann die Korrelation zusammengehöriger Eingaben erleichtern. Prüfen Sie die Auswirkungen auf die Privatsphäre vor wiederholten oder stark gemusterten Vorgängen.
- Die JavaScript-Demo verwendet öffentliche Testdaten und endet bei der Vorschau. Sie hält keine Schlüssel, weist kein Eigentum nach, erstellt keine Signaturen, berechnet keine Gebühren, übermittelt kein Paket und gleicht kein unsicheres Ergebnis ab.
- Der aktuelle Helfer `wallet.asset.merge_assets` ist eine Kompatibilitätsoberfläche und beansprucht keine kanonische Ledger-Abgleichsautorität. Die native Integration muss die Bestätigung über den maßgeblichen Wallet-Transaktionspfad führen.

<!-- help-sync:source {"page_path":"wallet/merge-split/index.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-merge.png","topic_id":"wallet.merge"} -->
