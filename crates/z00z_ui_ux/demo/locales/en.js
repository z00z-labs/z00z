"use strict";

window.Z00ZI18n.registerLocale("en", {
  common: {
    unavailable: "Unavailable",
    readOnly: "Read-only",
    chain: "Chain",
    chainLocked: "Choose carefully. The chain cannot be changed after this wallet is created.",
    on: "On",
    off: "Off",
    close: "Close",
    back: "Back"
  },
  app: {
    documentTitle: "Z00Z Wallet — Interactive Concept",
    menu: "Menu",
    wallets: "Wallets",
    network: "Network",
    addWallet: "Add wallet",
    removeWallet: "Remove wallet",
    settings: "Settings",
    home: "Home",
    homeContext: "Your private money at a glance",
    settingsContext: "Application preferences",
    walletContext: "{wallet} wallet",
    interactiveConcept: "Interactive concept · no real funds",
    conceptBuild: "Concept 0.4 · no real funds",
    logOut: "Log out",
    general: "General",
    language: "Language",
    languageHelp: "Language used throughout this wallet application",
    notifications: "Notifications",
    notificationsHelp: "Show wallet updates and required actions",
    regionalFormat: "Regional format",
    regionalFormatHelp: "Controls dates, numbers, and decimal separators independently from language",
    timeZone: "Time zone",
    timeZoneHelp: "Timestamps are stored in UTC and displayed in this time zone",
    networkUnits: "Network units",
    networkUnitsHelp: "Network rates use decimal bits per second",
    decimalBitrate: "Decimal bits per second",
    translationCoverage: "Translation coverage",
    translationCoverageHelp: "{count} language catalogues are synchronized with English source keys",
    languageChanged: "Language changed locally."
  },
  nav: {
    home: "Home",
    assets: "Assets",
    history: "History",
    swap: "Swap",
    exchange: "Exchange",
    staking: "Staking",
    backup: "Backup",
    settings: "Settings"
  },
  network: {
    routeTelemetry: "Route telemetry",
    carrierTelemetry: "Carrier telemetry",
    publicationTelemetry: "Publication telemetry"
  },
  walletShell: {
    balanceAvailable: "{value} available",
    current: "Current",
    scanning: "Scanning",
    identityAria: "Switch wallet. Current wallet: {wallet}",
    lockLabel: "{wallet} wallet",
    copyAddress: "Copy full address for {wallet} wallet",
    available: "Available",
    locked: "Locked",
    pendingIn: "Pending in",
    pendingOut: "Pending out",
    routeTelemetry: "Route telemetry"
  },
  assets: {
    wallet: "Wallet", sections: "Wallet sections", sectionAssets: "Assets", sectionAssetsHelp: "Spendable and owned value", sectionVouchers: "Vouchers", sectionVouchersHelp: "Conditional value", sectionPermissions: "Permissions", sectionPermissionsHelp: "Bounded authority",
    family: "Asset family", title: "Your assets", description: "Native cash, issued tokens, and collectibles stay distinguishable. Only spendable native cash enters Available.", send: "Send", moneyTotals: "Money totals", available: "Available", readyToUse: "Ready to use", receiving: "Receiving", sending: "Sending", waitingToSettle: "Waiting to settle",
    filters: "Asset filters", all: "All", filterCoins: "Coins", filterTokens: "Tokens", filterNfts: "NFTs", kindCoin: "Coin", kindToken: "Token", kindCollectible: "NFT", needsReview: "Needs review", owned: "Owned assets", ownedHelp: "Class, trust, and spendability are explicit", name: "Name", balance: "Balance", value: "Value", price: "Price",
    viewDetails: "View details for {asset}", receiveAsset: "Receive {asset}", sendAsset: "Send {asset}", receive: "Receive", noMarketFeed: "No market feed", nativeCatalog: "Native asset · trusted catalog", declaredDomain: "Declared domain · review needed", uniqueCollectible: "Unique collectible · metadata available", excludedNotice: "Vouchers, permissions, quarantined objects, non-native tokens, collectibles, and experimental compatibility assets are excluded from Available.",
    noVouchers: "No vouchers yet", noVouchersHelp: "Create a voucher when this wallet needs transferable conditional value.", createVoucher: "Create voucher", noPermissions: "No permissions yet", noPermissionsHelp: "Create a bounded permission when this wallet needs transferable authority.", createPermission: "Create permission"
  },
  history: {
    honestSettlement: "Honest settlement", title: "Everything that changed", description: "Submission and final settlement are shown as different states. Open an item for its receipt and technical timeline.", filters: "History filters", all: "All", assets: "Assets", vouchers: "Vouchers", permissions: "Permissions", system: "System", needsAttention: "Needs attention", search: "Search history", results: "History results",
    settling: "Settling", settled: "Settled", active: "Active", ready: "Ready", paymentTo: "Payment to {recipient}", sentWaiting: "Sent · waiting to settle", allocationClaimed: "Allocation claimed", verifiedClaimWaiting: "Verified claim · waiting to settle", receivedFrom: "Received from {sender}", yesterday: "Yesterday", travelRefundVoucher: "Travel refund voucher", offeredReviewBefore: "Offered · review before {date}", deliveryReceiptAccess: "Delivery receipt access", dataAccessUsesRemain: "Data access · {used} of {total} uses remain", uses: "{count} uses", jul12: "12 Jul", localBackupCreated: "Local backup created", integrityPassed: "Integrity check passed", jul10: "10 Jul", transferFrom: "Transfer from {wallet}", jul3: "3 Jul", recoveryCheckCompleted: "Recovery check completed", localVerificationPassed: "Local verification passed", jun30: "30 Jun", jul21: "21 Jul", waitingToSettle: "Waiting to settle", minutesAgo: "{count} min ago", noMatching: "No matching history", tryAnother: "Try another filter or search term.", details: "History details", status: "Status", when: "When", fee: "Fee", feeIncluded: "Included", feeNotApplicable: "Not applicable", privacy: "Privacy", privacyValue: "Target simulation · not live telemetry", carrierChain: "Carrier & chain", carrierChainValue: "Reticulum target · Main mock", technicalDetails: "Technical details", idLabel: "ID", lifecycleLabel: "Lifecycle", lifecyclePending: "created → submitted → admitted", lifecycleConfirmed: "created → submitted → admitted → confirmed", receiptLabel: "Receipt", copyReceipt: "Copy receipt", done: "Done"
  },
  staking: {
    eyebrow: "Wallet staking", heading: "Stake from {wallet}", description: "Staking terms and pending value remain distinct from the wallet's spendable Available balance.", badge: "Main · concept", totals: "Staking totals", availableToStake: "Available to stake", walletValueBefore: "Wallet value before a stake is submitted", staked: "Staked", nothingDelegated: "Nothing delegated in this concept", rewards: "Rewards", accrualNotSimulated: "Accrual is not simulated", prepare: "Prepare a stake", prepareHelp: "Choose the amount and inspect the validator terms before authorization.", amount: "Amount", availableBalance: "Available: {value}", validator: "Validator", validatorPlaceholder: "Choose after chain verification", review: "Review stake", safeguards: "Staking safeguards", validatorStatus: "Validator status", unlockPeriod: "Unlock period", notSelected: "Not selected", notProjected: "Not projected", notice: "The concept never estimates yield or hides lock-up risk. A stake remains pending until chain settlement."
  },
  settings: {
    sections: "Settings sections",
    application: "Application",
    connectivity: "Connectivity",
    general: "General",
    generalHelp: "Language and application notifications",
    appearance: "Appearance",
    appearanceHelp: "Theme, palette, density, and YAML highlighting",
    networkPrivacy: "Network",
    networkPrivacyHelp: "Private route, chain, and synchronization",
    networkSections: "Network sections",
    overview: "Overview"
  },
  walletSettings: {
    password: "Wallet password", passwordHelp: "Change this wallet password locally. The current password is required before a new one is accepted.", changePassword: "Change password",
    changePasswordTitle: "Change wallet password", changePasswordSubtitle: "Verify the current password, then choose a new one", currentPassword: "Current password", newPassword: "New password", confirmNewPassword: "Confirm new password",
    passwordChangeHint: "Use at least 8 characters. Passwords are cleared from the demo immediately.", changePasswordSubmit: "Change password", passwordChangedTitle: "Password updated", passwordChangedResult: "The password was changed only in this local concept. The demo clears every entry and does not retain a secret.",
    passwordCurrentError: "Enter the current password (at least 8 characters).", passwordNewError: "Use at least 8 characters for the new password.", passwordSameError: "Choose a password different from the current one.", passwordMismatchError: "New passwords do not match."
  },
  reticulum: {
    title: "Reticulum telemetry",
    summary: "A local-only view of carrier and delivery evidence. It supports availability decisions without turning transport metadata into user analytics.",
    localCapability: "Local capability unavailable",
    localCapabilityHelp: "Reticulum has no registered local status bridge in this wallet demo. No live-looking carrier data is invented.",
    tabs: {
      overview: "Overview",
      node: "Node",
      interfaces: "Interfaces",
      radio: "Radio",
      entrypoints: "Entry points",
      paths: "Paths",
      links: "Links",
      probes: "Probes"
    }
  },
  aggregators: {
    title: "Aggregators telemetry",
    summary: "Read-only service and publication evidence for aggregation work. It never receives wallet keys, seeds, or policy secrets.",
    localCapability: "Local capability unavailable",
    localCapabilityHelp: "The wallet has no registered bridge to the aggregator status snapshot, so this page correctly shows unavailable.",
    tabs: { overview: "Overview" }
  },
  onionnet: {
    title: "OnionNet telemetry",
    summary: "A boundary-aware view of deterministic control-plane state, local evidence, and aggregate synthetic health. This workspace never changes a route or privacy policy.",
    localCapability: "Local capability unavailable",
    localCapabilityHelp: "OnionNet has no registered status bridge in this wallet demo. No live-looking route, topology, or privacy values are invented.",
    tabs: {
      overview: "Overview",
      epoch: "Epoch",
      privacy: "Privacy",
      transport: "Transport",
      queues: "Queues & Replay",
      probation: "Probation",
      ingress: "Ingress"
    }
  },
  help: {
    title: "Help", openGlobal: "Open application help", openContext: "Help for this view",
    close: "Close help", contents: "Contents", section: "Help section {current} of {total}",
    unavailable: "Help is unavailable for this view."
  },
  status: {
    up: "Up",
    down: "Down",
    connecting: "Connecting",
    degraded: "Degraded"
  },
  units: {
    bitPerSecond: "{value} bit/s",
    kilobitPerSecond: "{value} kbit/s",
    megabitPerSecond: "{value} Mbit/s"
  }
});
