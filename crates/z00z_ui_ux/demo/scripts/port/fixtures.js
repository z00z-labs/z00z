"use strict";

((root) => {
  function deepFreeze(value) {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  }

  function clone(value) {
    return typeof structuredClone === "function"
      ? structuredClone(value)
      : JSON.parse(JSON.stringify(value));
  }

  const ASSET_ICON_LUT = deepFreeze({
    z00z: "assets/z00z-friendly/Coins/z00z-logo-gold.svg",
    algorand: "assets/z00z-friendly/Coins/algorand-algo-logo-z00z.svg",
    avalanche: "assets/z00z-friendly/Coins/avalanche-avax-logo-z00z.svg",
    bitcoin: "assets/z00z-friendly/Coins/bitcoin-btc-logo-z00z.svg",
    bold: "assets/z00z-friendly/Coins/BOLD_logo-z00z.svg",
    cardano: "assets/z00z-friendly/Coins/cardano-ada-logo-z00z.svg",
    dai: "assets/z00z-friendly/Coins/dai-dai-logo-z00z.svg",
    ethereum: "assets/z00z-friendly/Coins/ethereum-eth-logo-z00z.svg",
    hyperliquid: "assets/z00z-friendly/Coins/hyperliquid-hype-logo-z00z.svg",
    liquity: "assets/z00z-friendly/Coins/liquity-lqty-logo-z00z.svg",
    solana: "assets/z00z-friendly/Coins/solana-sol-logo-z00z.svg",
    zcash: "assets/z00z-friendly/Coins/zcash-zec-logo-z00z.svg",
    rain: "assets/z00z-friendly/Tokens/rain-rain.svg",
    sky: "assets/z00z-friendly/Tokens/sky-sky.svg",
    bcapNft: "assets/z00z-friendly/NFTs/bcap-nft.svg",
    stableNft: "assets/z00z-friendly/NFTs/stable-nft.svg"
  });

  const FRIENDLY_ASSET_CATALOG = deepFreeze([
    { key: "z00z", type: "coin", label: "Z00Z", ticker: "Z00Z", unit: "Z00Z", iconSrc: ASSET_ICON_LUT.z00z, divisible: true, owner: "Z00Z protocol", assetId: "z00z:main:coin", currentSupply: "18,450,000 Z00Z", maxSupply: "21,000,000 Z00Z" },
    { key: "algorand", type: "coin", label: "wAlgorand", ticker: "ALGO", unit: "ALGO", iconSrc: ASSET_ICON_LUT.algorand, divisible: true, owner: "Algorand network", assetId: "external:algorand:algo" },
    { key: "avalanche", type: "coin", label: "wAvalanche", ticker: "AVAX", unit: "AVAX", iconSrc: ASSET_ICON_LUT.avalanche, divisible: true, owner: "Avalanche network", assetId: "external:avalanche:avax" },
    { key: "bitcoin", type: "coin", label: "wBitcoin", ticker: "BTC", unit: "BTC", iconSrc: ASSET_ICON_LUT.bitcoin, divisible: true, owner: "Bitcoin network", assetId: "external:bitcoin:btc" },
    { key: "bold", type: "token", label: "wBOLD", ticker: "BOLD", unit: "BOLD", iconSrc: ASSET_ICON_LUT.bold, divisible: true, owner: "Liquity protocol", assetId: "external:ethereum:bold" },
    { key: "cardano", type: "coin", label: "wCardano", ticker: "ADA", unit: "ADA", iconSrc: ASSET_ICON_LUT.cardano, divisible: true, owner: "Cardano network", assetId: "external:cardano:ada" },
    { key: "dai", type: "token", label: "wDAI", ticker: "DAI", unit: "DAI", iconSrc: ASSET_ICON_LUT.dai, divisible: true, owner: "Sky protocol", assetId: "external:ethereum:dai" },
    { key: "ethereum", type: "coin", label: "wEthereum", ticker: "ETH", unit: "ETH", iconSrc: ASSET_ICON_LUT.ethereum, divisible: true, owner: "Ethereum network", assetId: "external:ethereum:eth" },
    { key: "hyperliquid", type: "coin", label: "wHyperliquid", ticker: "HYPE", unit: "HYPE", iconSrc: ASSET_ICON_LUT.hyperliquid, divisible: true, owner: "Hyperliquid network", assetId: "external:hyperliquid:hype" },
    { key: "liquity", type: "token", label: "wLiquity", ticker: "LQTY", unit: "LQTY", iconSrc: ASSET_ICON_LUT.liquity, divisible: true, owner: "Liquity protocol", assetId: "external:ethereum:lqty" },
    { key: "solana", type: "coin", label: "wSolana", ticker: "SOL", unit: "SOL", iconSrc: ASSET_ICON_LUT.solana, divisible: true, owner: "Solana network", assetId: "external:solana:sol" },
    { key: "zcash", type: "coin", label: "wZcash", ticker: "ZEC", unit: "ZEC", iconSrc: ASSET_ICON_LUT.zcash, divisible: true, owner: "Zcash network", assetId: "external:zcash:zec" },
    { key: "rain", type: "token", label: "Rain", ticker: "RAIN", unit: "RAIN", iconSrc: ASSET_ICON_LUT.rain, divisible: true, owner: "Rain", assetId: "external:ethereum:rain" },
    { key: "sky", type: "token", label: "Sky", ticker: "SKY", unit: "SKY", iconSrc: ASSET_ICON_LUT.sky, divisible: true, owner: "Sky protocol", assetId: "external:ethereum:sky" },
    { key: "bcapNft", type: "nft", label: "BCAP", ticker: "BCAP", unit: "NFT", iconSrc: ASSET_ICON_LUT.bcapNft, divisible: false, owner: "Blockchain Capital", assetId: "nft:bcap:demo", currentSupply: "1 NFT", maxSupply: "1 NFT" },
    { key: "stableNft", type: "nft", label: "STABLE", ticker: "STABLE", unit: "NFT", iconSrc: ASSET_ICON_LUT.stableNft, divisible: false, owner: "Stable", assetId: "nft:stable:demo", currentSupply: "1 NFT", maxSupply: "1 NFT" }
  ]);

  const DEFAULT_FRIENDLY_ASSET_KEYS = deepFreeze(FRIENDLY_ASSET_CATALOG.map(({ key }) => key));
  const ASSET_CATALOG = FRIENDLY_ASSET_CATALOG;

  const INITIAL_WALLET_FIXTURES = deepFreeze([
    {
      id: "everyday",
      name: "Everyday",
      chainId: "mainnet",
      initials: "E",
      address: "ZxChpo…2Mj8Pt",
      fullAddress: "ZxChpoioBEFR1PRJPamJxh5aWdEb94ek8J52PmT8PYAEa8RKVtSs9X3UPgaSaHvMMZKcQoiyVFhEE256vcyGPeFV23d2Mj8Pt",
      summary: { available: "12,480.75", locked: "0.00", pendingIn: "960.00", pendingOut: "240.00", scan: "Current" },
      assetKeys: [...DEFAULT_FRIENDLY_ASSET_KEYS],
      vouchers: [
        { id: "voucher-221", kind: "refund", title: "Travel refund voucher", detail: "Northwind Travel · consumed-asset backing · acceptance required · refund allowed", value: "86.00 Z00Z", status: "Offered", tone: "ready", detailFlow: "voucher-review", transferable: false },
        { id: "voucher-settled", kind: "redeemed", title: "Event deposit return", detail: "Riverside Events · redeemed and settled 12 Jul", value: "150.00 Z00Z", status: "Redeemed", tone: "settled", detailFlow: "voucher-settled", transferable: false },
        { id: "voucher-travel", kind: "travel", title: "Transit travel credit", detail: "MetroLink · accepted · valid for regional fares", value: "42.00 Z00Z", status: "Accepted", tone: "active", detailFlow: "voucher-detail", expiry: "30 Sep 2026", transferable: true },
        { id: "voucher-gift", kind: "gift", title: "Partner gift voucher", detail: "North Market · redeemable at verified merchants", value: "25.00 Z00Z", status: "Redeemable", tone: "active", detailFlow: "voucher-detail", expiry: "31 Dec 2026", transferable: true },
        { id: "voucher-service", kind: "service", title: "Service outage credit", detail: "Cloud Relay · issued for verified service interruption", value: "18.00 Z00Z", status: "Offered", tone: "ready", detailFlow: "voucher-detail", expiry: "15 Oct 2026", transferable: false },
        { id: "voucher-deposit", kind: "deposit", title: "Rental deposit voucher", detail: "Harbor Rentals · pending issuer release", value: "300.00 Z00Z", status: "Pending release", tone: "ready", detailFlow: "voucher-detail", expiry: "20 Aug 2026", transferable: false },
        { id: "voucher-restricted", kind: "restricted", title: "Restricted merchant voucher", detail: "Imported policy · merchant scope requires review", value: "65.00 Z00Z", status: "Needs review", tone: "attention", detailFlow: "voucher-detail", expiry: "01 Nov 2026", transferable: false },
        { id: "voucher-community", kind: "community", title: "Community voucher", detail: "Z00Z Commons · general local redemption", value: "10.00 Z00Z", status: "Held", tone: "active", detailFlow: "voucher-detail", expiry: "31 Jan 2027", transferable: true }
      ],
      permissions: [
        { id: "receipt", kind: "receipt", title: "Delivery receipt access", detail: "Data access · view · receipts.example · cannot delegate", remaining: "2 of 5 uses", classLabel: "Data access", action: "View receipt", scope: "receipts.example", delegation: "Forbidden", expiry: "31 Jul 2026", rightId: "right_54ac…1f88", typeLabel: "data_access", status: "Held", tone: "active", transferable: false },
        { id: "deploy", kind: "deploy", title: "Deploy to staging", detail: "Machine capability · deploy · staging.example · attenuation only", remaining: "1 use", classLabel: "Machine capability", action: "Deploy", scope: "staging.example", delegation: "Attenuation only", expiry: "19 Aug 2026", rightId: "right_8d9e…4a62", typeLabel: "machine_capability", status: "Held", tone: "active", transferable: false },
        { id: "publish", kind: "publish", title: "Publish release notes", detail: "Content authority · publish · releases.example · bounded", remaining: "3 uses", classLabel: "Content authority", action: "Publish", scope: "releases.example", delegation: "Forbidden", expiry: "30 Sep 2026", rightId: "right_a31c…7e02", typeLabel: "content_authority", status: "Held", tone: "active", transferable: true },
        { id: "approve", kind: "approve", title: "Approve vendor invoice", detail: "Approval right · invoices.example · up to 2 decisions", remaining: "2 uses", classLabel: "Approval right", action: "Approve invoice", scope: "invoices.example", delegation: "Forbidden", expiry: "15 Aug 2026", rightId: "right_b55d…2a91", typeLabel: "approval_right", status: "Held", tone: "active", transferable: false },
        { id: "audit", kind: "audit", title: "Read audit trail", detail: "Audit access · read-only · compliance.example", remaining: "12 uses", classLabel: "Audit access", action: "Read audit trail", scope: "compliance.example", delegation: "Attenuation only", expiry: "31 Dec 2026", rightId: "right_c02e…91d4", typeLabel: "audit_access", status: "Held", tone: "active", transferable: true },
        { id: "device", kind: "device", title: "Pair field terminal", detail: "Device authority · pair · terminal-07 · one use", remaining: "1 use", classLabel: "Device authority", action: "Pair device", scope: "terminal-07", delegation: "Forbidden", expiry: "05 Aug 2026", rightId: "right_d73b…63f0", typeLabel: "device_authority", status: "Held", tone: "active", transferable: false },
        { id: "emergency", kind: "emergency", title: "Emergency route override", detail: "High-risk authority · route override · review required", remaining: "1 use", classLabel: "Emergency authority", action: "Override route", scope: "carrier.emergency", delegation: "Forbidden", expiry: "24 Jul 2026", rightId: "right_e84a…0c33", typeLabel: "emergency_authority", status: "Needs review", tone: "attention", transferable: false },
        { id: "view", kind: "view", title: "View service status", detail: "Read-only capability · status.example · reusable", remaining: "20 uses", classLabel: "Read-only capability", action: "View status", scope: "status.example", delegation: "Attenuation only", expiry: "31 Jan 2027", rightId: "right_f19d…5b70", typeLabel: "read_only_capability", status: "Held", tone: "active", transferable: true }
      ],
      activities: [
        { id: "tx-7f31", type: "money", direction: "out", titleKey: "history.paymentTo", titleValues: { recipient: "Mira" }, detailKey: "history.sentWaiting", amount: "− 240.00 Z00Z", timeKey: "history.minutesAgo", timeValues: { count: 2 }, status: "settling" },
        { id: "claim-014", type: "asset", direction: "in", titleKey: "history.allocationClaimed", detailKey: "history.verifiedClaimWaiting", amount: "+ 86.00 Z00Z", timeKey: "history.minutesAgo", timeValues: { count: 18 }, status: "settling" },
        { id: "tx-7e88", type: "money", direction: "in", titleKey: "history.receivedFrom", titleValues: { sender: "Niko" }, detailKey: "history.settled", amount: "+ 1,200.00 Z00Z", timeKey: "history.yesterday", status: "settled" },
        { id: "voucher-221", type: "voucher", direction: "neutral", titleKey: "history.travelRefundVoucher", detailKey: "history.offeredReviewBefore", detailValueKeys: { date: "history.jul21" }, amount: "86.00 Z00Z", timeKey: "history.yesterday", status: "attention" },
        { id: "right-221", type: "permission", direction: "neutral", titleKey: "history.deliveryReceiptAccess", detailKey: "history.dataAccessUsesRemain", detailValues: { used: 2, total: 5 }, amountKey: "history.uses", amountValues: { count: 2 }, timeKey: "history.yesterday", status: "active" },
        { id: "tx-7d12", type: "money", direction: "out", titleKey: "history.paymentTo", titleValues: { recipient: "Coffee Lab" }, detailKey: "history.settled", amount: "− 18.50 Z00Z", timeKey: "history.jul12", status: "settled" },
        { id: "security-4", type: "security", direction: "neutral", titleKey: "history.localBackupCreated", detailKey: "history.integrityPassed", amount: "", timeKey: "history.jul10", status: "settled" }
      ]
    },
    {
      id: "savings",
      name: "Savings",
      chainId: "mainnet",
      initials: "S",
      address: "ZxR5vK…8Ee1Qm",
      fullAddress: "ZxR5vKpyP2W6eT8fVqH8M9sB7cX4aL2nQ5rD1uEe1Qm",
      summary: { available: "7,215.00", locked: "1,400.00", pendingIn: "0.00", pendingOut: "0.00", scan: "Current" },
      assetKeys: [...DEFAULT_FRIENDLY_ASSET_KEYS],
      vouchers: [],
      permissions: [],
      activities: [
        { id: "saving-100", type: "money", direction: "in", titleKey: "history.transferFrom", titleValues: { wallet: "Everyday" }, detailKey: "history.settled", amount: "+ 2,000.00 Z00Z", timeKey: "history.jul3", status: "settled" },
        { id: "saving-101", type: "security", direction: "neutral", titleKey: "history.recoveryCheckCompleted", detailKey: "history.localVerificationPassed", amount: "", timeKey: "history.jun30", status: "settled" }
      ]
    },
    {
      id: "travel",
      name: "Travel",
      chainId: "mainnet",
      initials: "T",
      address: "ZxT8cQ…4Fh2Ns",
      fullAddress: "ZxT8cQy6BvR3sL9wE1mD5hK7pA4Fh2Ns",
      summary: { available: "860.00", locked: "0.00", pendingIn: "125.00", pendingOut: "0.00", scan: "Scanning" },
      assetKeys: [...DEFAULT_FRIENDLY_ASSET_KEYS],
      vouchers: [],
      permissions: [],
      activities: [
        { id: "travel-100", type: "money", direction: "in", titleKey: "history.receivedFrom", titleValues: { sender: "Niko" }, detailKey: "history.waitingToSettle", amount: "+ 125.00 Z00Z", timeKey: "history.minutesAgo", timeValues: { count: 8 }, status: "settling" },
        { id: "travel-101", type: "money", direction: "out", titleKey: "history.paymentTo", titleValues: { recipient: "RailLink" }, detailKey: "history.settled", amount: "− 74.50 Z00Z", timeKey: "history.yesterday", status: "settled" }
      ]
    }
  ]);

  const EMPTY_WALLET_FIXTURE = deepFreeze({
    id: "empty",
    name: "",
    chainId: "mainnet",
    initials: "",
    address: "",
    fullAddress: "",
    summary: { available: "0.00", locked: "0.00", pendingIn: "0.00", pendingOut: "0.00", scan: "Unavailable" },
    assetKeys: [...DEFAULT_FRIENDLY_ASSET_KEYS],
    vouchers: [],
    permissions: [],
    activities: []
  });

  const DEFAULT_WALLET_PREFERENCES = deepFreeze({
    currency: "Z00Z",
    defaultFee: "0.001",
    autoBackup: false,
    backupIntervalHours: "24",
    lockAfterMinutes: "15",
    policyProfile: "Personal Safe · v1.4",
    policyRules: {
      maxTransaction: "2500",
      maxDaily: "5000",
      requireConfirmation: true,
      allowedAssets: "all",
      allowedRecipients: "",
      timeWindow: "any"
    },
    lastMasterKeyRotation: "Never"
  });

  function createInitialWallets() {
    return clone(INITIAL_WALLET_FIXTURES);
  }

  function createEmptyWallet() {
    return clone(EMPTY_WALLET_FIXTURE);
  }

  function createWalletPreferences(autoLockMinutes = DEFAULT_WALLET_PREFERENCES.lockAfterMinutes) {
    return { ...clone(DEFAULT_WALLET_PREFERENCES), lockAfterMinutes: String(autoLockMinutes) };
  }

  function createWalletProfile(existingWallets, name, chainId = "mainnet", scan = "Scanning") {
    const ids = new Set(existingWallets.map((wallet) => wallet.id));
    let index = existingWallets.length + 1;
    while (ids.has(`wallet-${index}`)) index += 1;
    const addressTail = String(2300 + index).padStart(4, "0");
    return {
      id: `wallet-${index}`,
      name,
      chainId,
      initials: name.trim().slice(0, 1).toUpperCase(),
      address: `ZxN${index}q7…${addressTail}Pt`,
      fullAddress: `ZxN${index}q7xA1mP9vR4sT8cQ2wE6hK${addressTail}Pt`,
      summary: { available: "0.00", locked: "0.00", pendingIn: "0.00", pendingOut: "0.00", scan },
      assetKeys: [...DEFAULT_FRIENDLY_ASSET_KEYS],
      vouchers: [],
      permissions: [],
      activities: []
    };
  }

  Object.assign(root.Z00ZDemo ||= {}, {
    INITIAL_WALLET_FIXTURES,
    ASSET_CATALOG,
    ASSET_ICON_LUT,
    DEFAULT_FRIENDLY_ASSET_KEYS,
    createInitialWallets,
    createEmptyWallet,
    createWalletPreferences,
    createWalletProfile
  });
})(typeof window === "undefined" ? globalThis : window);
