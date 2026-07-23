"use strict";

window.Z00ZI18n.registerLocale("de", {
  common: { unavailable: "Nicht verfügbar", readOnly: "Schreibgeschützt", chain: "Blockchain", chainLocked: "Wählen Sie sorgfältig. Nach dem Erstellen des Wallets kann die Blockchain nicht mehr geändert werden.", on: "Ein", off: "Aus", close: "Schließen", back: "Zurück" },
  app: {
    documentTitle: "Z00Z Wallet — Interaktives Konzept", menu: "Menü", wallets: "Wallets", network: "Netzwerk",
    addWallet: "Wallet hinzufügen", removeWallet: "Wallet entfernen", settings: "Einstellungen", home: "Startseite",
    homeContext: "Ihr privates Geld auf einen Blick", settingsContext: "App-Einstellungen", walletContext: "Wallet {wallet}", interactiveConcept: "Interaktives Konzept · keine echten Mittel", conceptBuild: "Konzept 0.4 · keine echten Mittel", logOut: "Abmelden",
    general: "Allgemein", language: "Sprache", languageHelp: "Sprache, die in dieser Wallet-Anwendung verwendet wird",
    notifications: "Benachrichtigungen", notificationsHelp: "Wallet-Aktualisierungen und erforderliche Aktionen anzeigen",
    regionalFormat: "Regionales Format", regionalFormatHelp: "Steuert Datums-, Zahlen- und Dezimaltrennzeichen unabhängig von der Sprache",
    timeZone: "Zeitzone", timeZoneHelp: "Zeitstempel werden in UTC gespeichert und in dieser Zeitzone angezeigt",
    networkUnits: "Netzwerkeinheiten", networkUnitsHelp: "Netzwerkraten verwenden dezimale Bits pro Sekunde",
    decimalBitrate: "Dezimale Bits pro Sekunde", translationCoverage: "Übersetzungsabdeckung",
    translationCoverageHelp: "{count} Sprachkataloge sind mit englischen Quellschlüsseln synchronisiert",
    languageChanged: "Sprache lokal geändert."
  },
  nav: { home: "Startseite", assets: "Vermögenswerte", history: "Verlauf", swap: "Tausch", exchange: "Börse", staking: "Staking", backup: "Sicherung", settings: "Einstellungen" },
  network: { routeTelemetry: "Routentelemetrie", carrierTelemetry: "Trägertelemetrie", publicationTelemetry: "Veröffentlichungstelemetrie" },
  walletShell: {
    balanceAvailable: "{value} verfügbar",
    current: "Aktuell", scanning: "Wird gescannt", identityAria: "Wallet wechseln. Aktuelle Wallet: {wallet}", lockLabel: "Wallet {wallet}", copyAddress: "Vollständige Adresse der Wallet {wallet} kopieren",
    available: "Verfügbar", locked: "Gesperrt", pendingIn: "Eingang ausstehend", pendingOut: "Ausgang ausstehend", routeTelemetry: "Routentelemetrie"
  },
  assets: {
    wallet: "Wallet", sections: "Wallet-Bereiche", sectionAssets: "Vermögenswerte", sectionAssetsHelp: "Verfügbarer und gehaltener Wert", sectionVouchers: "Gutscheine", sectionVouchersHelp: "Bedingter Wert", sectionPermissions: "Berechtigungen", sectionPermissionsHelp: "Gebundene Autorität",
    family: "Anlageklasse", title: "Ihre Vermögenswerte", description: "Native Mittel, ausgegebene Token und Sammlerstücke bleiben unterscheidbar. Nur ausgabefähige native Mittel zählen als Verfügbar.", send: "Senden", moneyTotals: "Geldsummen", available: "Verfügbar", readyToUse: "Einsatzbereit", receiving: "Eingehend", sending: "Ausgehend", waitingToSettle: "Wartet auf Abwicklung",
    filters: "Asset-Filter", all: "Alle", filterCoins: "Münzen", filterTokens: "Token", filterNfts: "NFTs", kindCoin: "Münze", kindToken: "Token", kindCollectible: "NFT", needsReview: "Prüfung nötig", owned: "Gehaltene Vermögenswerte", ownedHelp: "Klasse, Vertrauen und Ausgabefähigkeit sind eindeutig", name: "Name", balance: "Bestand", value: "Wert", price: "Preis",
    viewDetails: "Details für {asset} anzeigen", receiveAsset: "{asset} empfangen", sendAsset: "{asset} senden", receive: "Empfangen", noMarketFeed: "Kein Marktfeed", nativeCatalog: "Nativer Vermögenswert · vertrauenswürdiger Katalog", declaredDomain: "Deklarierte Domain · Prüfung nötig", uniqueCollectible: "Einzigartiges Sammlerstück · Metadaten verfügbar", excludedNotice: "Gutscheine, Berechtigungen, quarantänisierte Objekte, nicht-native Token, Sammlerstücke und experimentelle Kompatibilitäts-Assets sind von Verfügbar ausgeschlossen.",
    noVouchers: "Noch keine Gutscheine", noVouchersHelp: "Erstellen Sie einen Gutschein, wenn dieses Wallet übertragbaren bedingten Wert benötigt.", createVoucher: "Gutschein erstellen", noPermissions: "Noch keine Berechtigungen", noPermissionsHelp: "Erstellen Sie eine begrenzte Berechtigung, wenn dieses Wallet übertragbare Autorität benötigt.", createPermission: "Berechtigung erstellen"
  },
  history: {
    honestSettlement: "Nachvollziehbare Abwicklung", title: "Alle Änderungen", description: "Übermittlung und endgültige Abwicklung werden als getrennte Zustände angezeigt. Öffnen Sie einen Eintrag für Beleg und technische Zeitleiste.", filters: "Verlaufsfilter", all: "Alle", assets: "Vermögenswerte", vouchers: "Gutscheine", permissions: "Berechtigungen", system: "System", needsAttention: "Prüfung nötig", search: "Verlauf durchsuchen", results: "Verlaufsergebnisse",
    settling: "In Abwicklung", settled: "Abgewickelt", active: "Aktiv", ready: "Bereit", paymentTo: "Zahlung an {recipient}", sentWaiting: "Gesendet · wartet auf Abwicklung", allocationClaimed: "Zuweisung beansprucht", verifiedClaimWaiting: "Anspruch geprüft · wartet auf Abwicklung", receivedFrom: "Erhalten von {sender}", yesterday: "Gestern", travelRefundVoucher: "Reise-Rückerstattungsgutschein", offeredReviewBefore: "Angeboten · vor {date} prüfen", deliveryReceiptAccess: "Zugriff auf Lieferbeleg", dataAccessUsesRemain: "Datenzugriff · {used} von {total} Nutzungen verbleiben", uses: "{count} Nutzungen", jul12: "12. Juli", localBackupCreated: "Lokales Backup erstellt", integrityPassed: "Integritätsprüfung bestanden", jul10: "10. Juli", transferFrom: "Übertrag von {wallet}", jul3: "3. Juli", recoveryCheckCompleted: "Wiederherstellungsprüfung abgeschlossen", localVerificationPassed: "Lokale Prüfung bestanden", jun30: "30. Juni", jul21: "21. Juli", waitingToSettle: "Wartet auf Abwicklung", minutesAgo: "vor {count} Min.", noMatching: "Kein passender Verlauf", tryAnother: "Versuchen Sie einen anderen Filter oder Suchbegriff.", details: "Verlaufsdetails", status: "Status", when: "Wann", fee: "Gebühr", feeIncluded: "Enthalten", feeNotApplicable: "Nicht anwendbar", privacy: "Privatsphäre", privacyValue: "Zielsimulation · keine Live-Telemetrie", carrierChain: "Träger und Chain", carrierChainValue: "Reticulum-Ziel · Mainnet-Entwurf", technicalDetails: "Technische Details", idLabel: "ID", lifecycleLabel: "Lebenszyklus", lifecyclePending: "erstellt → übermittelt → zugelassen", lifecycleConfirmed: "erstellt → übermittelt → zugelassen → bestätigt", receiptLabel: "Beleg", copyReceipt: "Beleg kopieren", done: "Fertig"
  },
  staking: {
    eyebrow: "Wallet-Staking", heading: "Staking aus {wallet}", description: "Staking-Bedingungen und ausstehender Wert bleiben vom ausgabefähigen Verfügbar-Guthaben der Wallet getrennt.", badge: "Hauptnetz · Konzept", totals: "Staking-Summen", availableToStake: "Zum Staking verfügbar", walletValueBefore: "Wallet-Wert vor dem Einreichen eines Stakings", staked: "Gestakt", nothingDelegated: "In diesem Konzept ist nichts delegiert", rewards: "Belohnungen", accrualNotSimulated: "Ertrag wird nicht simuliert", prepare: "Staking vorbereiten", prepareHelp: "Wählen Sie den Betrag und prüfen Sie die Validatorbedingungen vor der Autorisierung.", amount: "Betrag", availableBalance: "Verfügbar: {value}", validator: "Validator", validatorPlaceholder: "Nach Chain-Prüfung auswählen", review: "Staking prüfen", safeguards: "Staking-Schutzmaßnahmen", validatorStatus: "Validatorstatus", unlockPeriod: "Entsperrzeit", notSelected: "Nicht ausgewählt", notProjected: "Nicht prognostiziert", notice: "Das Konzept schätzt keine Rendite und verbirgt kein Sperrrisiko. Ein Staking bleibt bis zur Chain-Abwicklung ausstehend."
  },
  settings: { sections: "Einstellungsbereiche", application: "Anwendung", connectivity: "Konnektivität", general: "Allgemein", generalHelp: "Sprache und App-Benachrichtigungen", appearance: "Darstellung", appearanceHelp: "Thema, Palette, Dichte und YAML-Hervorhebung", networkPrivacy: "Netzwerk", networkPrivacyHelp: "Private Route, Chain und Synchronisierung", networkSections: "Netzwerkbereiche", overview: "Überblick" },
  walletSettings: { password: "Wallet-Passwort", passwordHelp: "Ändern Sie dieses Wallet-Passwort lokal. Das aktuelle Passwort ist erforderlich, bevor ein neues akzeptiert wird.", changePassword: "Passwort ändern", changePasswordTitle: "Wallet-Passwort ändern", changePasswordSubtitle: "Bestätigen Sie das aktuelle Passwort und wählen Sie dann ein neues", currentPassword: "Aktuelles Passwort", newPassword: "Neues Passwort", confirmNewPassword: "Neues Passwort bestätigen", passwordChangeHint: "Verwenden Sie mindestens 8 Zeichen. Passwörter werden sofort aus der Demo gelöscht.", changePasswordSubmit: "Passwort ändern", passwordChangedTitle: "Passwort aktualisiert", passwordChangedResult: "Das Passwort wurde nur in diesem lokalen Konzept geändert. Die Demo löscht jede Eingabe und speichert kein Geheimnis.", passwordCurrentError: "Geben Sie das aktuelle Passwort ein (mindestens 8 Zeichen).", passwordNewError: "Verwenden Sie mindestens 8 Zeichen für das neue Passwort.", passwordSameError: "Wählen Sie ein Passwort, das sich vom aktuellen unterscheidet.", passwordMismatchError: "Die neuen Passwörter stimmen nicht überein." },
  reticulum: {
    title: "Reticulum-Telemetrie",
    summary: "Eine lokale Ansicht von Träger- und Zustellnachweisen. Sie unterstützt Verfügbarkeitsentscheidungen, ohne Transportmetadaten in Nutzeranalysen zu verwandeln.",
    localCapability: "Lokale Funktion nicht verfügbar",
    localCapabilityHelp: "In dieser Wallet-Demo ist keine lokale Reticulum-Statusbrücke registriert. Es werden keine scheinbar echten Trägerdaten erfunden.",
    tabs: { overview: "Überblick", node: "Knoten", interfaces: "Schnittstellen", radio: "Funk", entrypoints: "Einstiegspunkte", paths: "Pfade", probes: "Prüfungen", links: "Verbindungen" }
  },
  aggregators: {
    title: "Aggregator-Telemetrie",
    summary: "Schreibgeschützte Dienst- und Veröffentlichungsnachweise für die Aggregierung. Dieser Bereich erhält niemals Wallet-Schlüssel, Seed-Phrasen oder Richtliniengeheimnisse.",
    localCapability: "Lokale Funktion nicht verfügbar",
    localCapabilityHelp: "Die Wallet hat keine Brücke zum Aggregator-Status-Snapshot registriert; diese Seite zeigt daher korrekt keine Verfügbarkeit an.",
    tabs: { overview: "Überblick" }
  },
  onionnet: {
    title: "OnionNet-Telemetrie",
    summary: "Eine grenzbewusste Ansicht des deterministischen Control-Plane-Zustands, lokaler Nachweise und aggregierter synthetischer Gesundheit. Dieser Bereich ändert nie eine Route oder Datenschutzrichtlinie.",
    localCapability: "Lokale Funktion nicht verfügbar",
    localCapabilityHelp: "In dieser Wallet-Demo ist keine OnionNet-Statusbrücke registriert. Es werden keine scheinbar echten Routen-, Topologie- oder Datenschutzwerte erfunden.",
    tabs: { overview: "Überblick", epoch: "Epoche", privacy: "Datenschutzschwelle", transport: "Transport", queues: "Warteschlangen und Wiederholungen", probation: "Bewährung", ingress: "Eingangsgrenze" }
  },
  status: { up: "Aktiv", down: "Inaktiv", connecting: "Verbindung", degraded: "Eingeschränkt" },
  units: { bitPerSecond: "{value} bit/s", kilobitPerSecond: "{value} kbit/s", megabitPerSecond: "{value} Mbit/s" }
});
