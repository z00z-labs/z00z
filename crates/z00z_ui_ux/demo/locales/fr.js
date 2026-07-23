"use strict";

window.Z00ZI18n.registerLocale("fr", {
  common: { unavailable: "Indisponible", readOnly: "Lecture seule", chain: "Chaîne", chainLocked: "Choisissez avec soin. La chaîne ne peut plus être modifiée après la création du portefeuille.", on: "Activé", off: "Désactivé", close: "Fermer" },
  app: {
    documentTitle: "Z00Z Wallet — Concept interactif", wallets: "Portefeuilles", network: "Réseau",
    addWallet: "Ajouter un portefeuille", removeWallet: "Supprimer un portefeuille", settings: "Paramètres", home: "Accueil",
    homeContext: "Vos fonds privés en un coup d’œil", settingsContext: "Préférences de l’application", walletContext: "Portefeuille {wallet}", interactiveConcept: "Concept interactif · aucun fonds réel", conceptBuild: "Concept 0.4 · aucun fonds réel", logOut: "Se déconnecter",
    general: "Général", language: "Langue", languageHelp: "Langue utilisée dans toute cette application de portefeuille",
    notifications: "Notifications", notificationsHelp: "Afficher les mises à jour du portefeuille et les actions requises",
    regionalFormat: "Format régional", regionalFormatHelp: "Contrôle les dates, les nombres et les séparateurs décimaux indépendamment de la langue",
    timeZone: "Fuseau horaire", timeZoneHelp: "Les horodatages sont stockés en UTC et affichés dans ce fuseau horaire",
    networkUnits: "Unités réseau", networkUnitsHelp: "Les débits réseau utilisent des bits décimaux par seconde",
    decimalBitrate: "Bits décimaux par seconde", translationCoverage: "Couverture de traduction",
    translationCoverageHelp: "{count} catalogues linguistiques sont synchronisés avec les clés sources anglaises",
    languageChanged: "Langue modifiée localement."
  },
  nav: { home: "Accueil", assets: "Actifs", history: "Historique", swap: "Échange", exchange: "Bourse", staking: "Staking", backup: "Sauvegarde", settings: "Paramètres" },
  network: { routeTelemetry: "Télémétrie de route", carrierTelemetry: "Télémétrie de transport", publicationTelemetry: "Télémétrie de publication" },
  walletShell: {
    balanceAvailable: "{value} disponibles",
    current: "Actuel", scanning: "Analyse en cours", identityAria: "Changer de portefeuille. Portefeuille actuel : {wallet}", lockLabel: "Portefeuille {wallet}", copyAddress: "Copier l’adresse complète du portefeuille {wallet}",
    available: "Disponible", locked: "Bloqué", pendingIn: "Entrant en attente", pendingOut: "Sortant en attente", routeTelemetry: "Télémétrie de route"
  },
  assets: {
    wallet: "Portefeuille", sections: "Sections du portefeuille", sectionAssets: "Actifs", sectionAssetsHelp: "Valeur détenue et disponible", sectionVouchers: "Bons", sectionVouchersHelp: "Valeur conditionnelle", sectionPermissions: "Autorisations", sectionPermissionsHelp: "Autorité limitée",
    family: "Famille d’actifs", title: "Vos actifs", description: "La monnaie native, les jetons émis et les objets de collection restent distincts. Seule la monnaie native dépensable entre dans Disponible.", send: "Envoyer", moneyTotals: "Totaux monétaires", available: "Disponible", readyToUse: "Prêt à utiliser", receiving: "Réception", sending: "Envoi", waitingToSettle: "En attente de règlement",
    filters: "Filtres d’actifs", all: "Tous", filterCoins: "Monnaies", filterTokens: "Jetons", filterNfts: "NFT", kindCoin: "Monnaie", kindToken: "Jeton", kindCollectible: "NFT", needsReview: "À vérifier", owned: "Actifs détenus", ownedHelp: "La classe, la confiance et la dépensabilité sont explicites", name: "Nom", balance: "Solde", value: "Valeur", price: "Prix",
    viewDetails: "Voir les détails de {asset}", receiveAsset: "Recevoir {asset}", sendAsset: "Envoyer {asset}", receive: "Recevoir", noMarketFeed: "Aucun flux de marché", nativeCatalog: "Actif natif · catalogue fiable", declaredDomain: "Domaine déclaré · vérification requise", uniqueCollectible: "Objet de collection unique · métadonnées disponibles", excludedNotice: "Les bons, autorisations, objets en quarantaine, jetons non natifs, objets de collection et actifs de compatibilité expérimentale sont exclus de Disponible.",
    noVouchers: "Aucun bon pour le moment", noVouchersHelp: "Créez un bon lorsque ce portefeuille a besoin d’une valeur conditionnelle transférable.", createVoucher: "Créer un bon", noPermissions: "Aucune autorisation pour le moment", noPermissionsHelp: "Créez une autorisation limitée lorsque ce portefeuille a besoin d’une autorité transférable.", createPermission: "Créer une autorisation"
  },
  history: {
    honestSettlement: "Règlement transparent", title: "Tous les changements", description: "La soumission et le règlement final sont affichés comme des états distincts. Ouvrez un élément pour sa preuve et sa chronologie technique.", filters: "Filtres d’historique", all: "Tous", assets: "Actifs", vouchers: "Bons", permissions: "Autorisations", system: "Système", needsAttention: "À examiner", search: "Rechercher dans l’historique", results: "Résultats de l’historique",
    settling: "En règlement", settled: "Réglé", active: "Actif", ready: "Prêt", paymentTo: "Paiement à {recipient}", sentWaiting: "Envoyé · en attente de règlement", allocationClaimed: "Allocation réclamée", verifiedClaimWaiting: "Réclamation vérifiée · en attente de règlement", receivedFrom: "Reçu de {sender}", yesterday: "Hier", travelRefundVoucher: "Bon de remboursement voyage", offeredReviewBefore: "Offert · à examiner avant le {date}", deliveryReceiptAccess: "Accès au reçu de livraison", dataAccessUsesRemain: "Accès aux données · {used} utilisations sur {total} restantes", uses: "{count} utilisations", jul12: "12 juil.", localBackupCreated: "Sauvegarde locale créée", integrityPassed: "Contrôle d’intégrité réussi", jul10: "10 juil.", transferFrom: "Virement depuis {wallet}", jul3: "3 juil.", recoveryCheckCompleted: "Contrôle de récupération terminé", localVerificationPassed: "Vérification locale réussie", jun30: "30 juin", jul21: "21 juil.", waitingToSettle: "En attente de règlement", minutesAgo: "il y a {count} min", noMatching: "Aucun historique correspondant", tryAnother: "Essayez un autre filtre ou terme de recherche.", details: "Détails de l’historique", status: "État", when: "Quand", fee: "Frais", feeIncluded: "Inclus", feeNotApplicable: "Sans objet", privacy: "Confidentialité", privacyValue: "Simulation de cible · pas de télémétrie en direct", carrierChain: "Transport et chaîne", carrierChainValue: "Cible Reticulum · maquette du réseau principal", technicalDetails: "Détails techniques", idLabel: "ID", lifecycleLabel: "Cycle de vie", lifecyclePending: "créé → soumis → admis", lifecycleConfirmed: "créé → soumis → admis → confirmé", receiptLabel: "Reçu", copyReceipt: "Copier le reçu", done: "Terminé"
  },
  staking: {
    eyebrow: "Staking du portefeuille", heading: "Staker depuis {wallet}", description: "Les conditions de staking et la valeur en attente restent distinctes du solde Disponible dépensable du portefeuille.", badge: "Réseau principal · concept", totals: "Totaux de staking", availableToStake: "Disponible à staker", walletValueBefore: "Valeur du portefeuille avant la soumission", staked: "Staké", nothingDelegated: "Rien n’est délégué dans ce concept", rewards: "Récompenses", accrualNotSimulated: "L’accumulation n’est pas simulée", prepare: "Préparer un staking", prepareHelp: "Choisissez le montant et examinez les conditions du validateur avant l’autorisation.", amount: "Montant", availableBalance: "Disponible : {value}", validator: "Validateur", validatorPlaceholder: "Choisir après vérification de la chaîne", review: "Vérifier le staking", safeguards: "Garanties de staking", validatorStatus: "État du validateur", unlockPeriod: "Période de déblocage", notSelected: "Non sélectionné", notProjected: "Non projeté", notice: "Le concept n’estime jamais le rendement et ne masque pas le risque de blocage. Un staking reste en attente jusqu’au règlement de la chaîne."
  },
  settings: { sections: "Sections des paramètres", application: "Application", connectivity: "Connectivité", general: "Général", generalHelp: "Langue et notifications de l’application", appearance: "Apparence", appearanceHelp: "Thème, palette, densité et coloration YAML", networkPrivacy: "Réseau", networkPrivacyHelp: "Route privée, chaîne et synchronisation", networkSections: "Sections réseau", overview: "Vue d’ensemble" },
  walletSettings: { password: "Mot de passe du portefeuille", passwordHelp: "Modifiez localement le mot de passe de ce portefeuille. Le mot de passe actuel est requis avant l’acceptation du nouveau.", changePassword: "Modifier le mot de passe", changePasswordTitle: "Modifier le mot de passe du portefeuille", changePasswordSubtitle: "Vérifiez le mot de passe actuel, puis choisissez-en un nouveau", currentPassword: "Mot de passe actuel", newPassword: "Nouveau mot de passe", confirmNewPassword: "Confirmer le nouveau mot de passe", passwordChangeHint: "Utilisez au moins 8 caractères. Les mots de passe sont immédiatement effacés de la démo.", changePasswordSubmit: "Modifier le mot de passe", passwordChangedTitle: "Mot de passe mis à jour", passwordChangedResult: "Le mot de passe a été modifié uniquement dans ce concept local. La démo efface chaque saisie et ne conserve aucun secret.", passwordCurrentError: "Saisissez le mot de passe actuel (au moins 8 caractères).", passwordNewError: "Utilisez au moins 8 caractères pour le nouveau mot de passe.", passwordSameError: "Choisissez un mot de passe différent du mot de passe actuel.", passwordMismatchError: "Les nouveaux mots de passe ne correspondent pas." },
  reticulum: {
    title: "Télémétrie Reticulum",
    summary: "Une vue locale des preuves de transport et de livraison. Elle aide à évaluer la disponibilité sans transformer les métadonnées de transport en analyses utilisateur.",
    localCapability: "Capacité locale indisponible",
    localCapabilityHelp: "Aucun pont de statut Reticulum local n'est enregistré dans cette démo de portefeuille. Aucune donnée de transport n'est simulée comme réelle.",
    tabs: { overview: "Vue d’ensemble", node: "Nœud", interfaces: "Interfaces", radio: "Radio", entrypoints: "Points d’entrée", paths: "Chemins", probes: "Sondes", links: "Liens" }
  },
  aggregators: {
    title: "Télémétrie des agrégateurs",
    summary: "Preuves en lecture seule sur les services et les publications d’agrégation. Cet espace ne reçoit jamais les clés, phrases de récupération ou secrets de politique du portefeuille.",
    localCapability: "Capacité locale indisponible",
    localCapabilityHelp: "Le portefeuille n’a aucun pont enregistré vers l’instantané de statut des agrégateurs ; cette page affiche donc correctement l’indisponibilité.",
    tabs: { overview: "Vue d’ensemble" }
  },
  onionnet: {
    title: "Télémétrie OnionNet",
    summary: "Une vue respectant les limites des données de l’état déterministe du plan de contrôle, des preuves locales et de la santé synthétique agrégée. Cet espace ne modifie jamais une route ou une politique de confidentialité.",
    localCapability: "Capacité locale indisponible",
    localCapabilityHelp: "Aucun pont de statut OnionNet n’est enregistré dans cette démo de portefeuille. Aucune valeur de route, topologie ou confidentialité semblant réelle n’est inventée.",
    tabs: { overview: "Vue d’ensemble", epoch: "Époque", privacy: "Seuil de confidentialité", transport: "Transport", queues: "Files et relectures", probation: "Probation", ingress: "Frontière d’entrée" }
  },
  status: { up: "Actif", down: "Inactif", connecting: "Connexion", degraded: "Dégradé" },
  units: { bitPerSecond: "{value} bit/s", kilobitPerSecond: "{value} kbit/s", megabitPerSecond: "{value} Mbit/s" }
});
