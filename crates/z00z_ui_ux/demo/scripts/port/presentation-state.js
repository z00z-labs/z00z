"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.createInitialWallets) {
    throw new Error("Z00Z demo contracts and fixtures must load before presentation state.");
  }

  const PALETTE_OPTIONS = Object.freeze([
    Object.freeze({ id: "z00z-default", label: "Z00Z Default", description: "Current private-wallet palette" }),
    Object.freeze({ id: "black-gold-elegance", label: "Black & Gold", description: "Black, navy, and restrained gold" }),
    Object.freeze({ id: "moonlit-stroll", label: "Moonlit Stroll", description: "Moonlit teal and navy with restrained gold" }),
    Object.freeze({ id: "walking-at-night", label: "Walking at Night", description: "Blue-charcoal streets and warm stone" })
  ]);

  const CODE_THEME_OPTIONS = Object.freeze([
    Object.freeze({ id: "atom-one-light", label: "One Light", description: "Bright technical surface with magenta, amber, violet, and green syntax.", mode: "light" }),
    Object.freeze({ id: "xcode", label: "Xcode", description: "Light Apple-style syntax with green comments and crisp blue numerics.", mode: "light" }),
    Object.freeze({ id: "atom-one-dark", label: "One Dark", description: "Deep blue-black surface with Monokai pink, amber, violet, and green syntax.", mode: "dark" }),
    Object.freeze({ id: "night-owl", label: "Night Owl", description: "Deep dark technical surface with muted violet, sand, and orange tokens.", mode: "dark" })
  ]);

  function createInitialState({ search = "", brand = "", rail = "" } = {}) {
    const navigation = demo.resolveInitialNavigation(search);
    return {
      view: navigation.view,
      balanceHidden: false,
      expertDetails: false,
      activityFilter: "all",
      assetFilter: "all",
      walletSection: navigation.walletSection,
      walletSettingsSection: navigation.walletSettingsSection,
      settingsSection: navigation.settingsSection,
      networkSection: navigation.networkSection,
      telemetrySource: navigation.telemetrySource,
      reticulumTelemetryTab: navigation.reticulumTelemetryTab,
      onionnetTelemetryTab: navigation.onionnetTelemetryTab,
      aggregatorsTelemetryTab: navigation.aggregatorsTelemetryTab,
      isNetworkOpen: ["reticulum", "onionnet"].includes(navigation.settingsSection),
      theme: "dark",
      palette: "z00z-default",
      language: "en",
      regionalLocale: "en-US",
      timeZone: "UTC",
      networkUnits: "decimal-bps",
      notifications: true,
      autoLockMinutes: "15",
      textScale: "100",
      reducedMotion: false,
      codeTheme: "atom-one-dark",
      configView: "yaml",
      configDraft: "",
      walletSettingsConfigDraft: "",
      configStatus: "Local draft is in sync with the visible controls.",
      hasCustomAppearance: false,
      customAppearance: { brand, rail },
      walletPreferences: {},
      sendDrafts: {},
      exchangeDrafts: {},
      locked: false,
      flow: null,
      lastDialogTrigger: null,
      selectedWalletId: "everyday",
      wallets: demo.createInitialWallets()
    };
  }

  function activeWallet(state) {
    return state.wallets.find((wallet) => wallet.id === state.selectedWalletId)
      || state.wallets[0]
      || demo.createEmptyWallet();
  }

  function ensureWalletPreferences(state, wallet = activeWallet(state)) {
    if (!state.walletPreferences[wallet.id]) {
      state.walletPreferences[wallet.id] = demo.createWalletPreferences(state.autoLockMinutes);
    }
    return state.walletPreferences[wallet.id];
  }

  Object.assign(root.Z00ZDemo, {
    PALETTE_OPTIONS,
    CODE_THEME_OPTIONS,
    createInitialState,
    activeWallet,
    ensureWalletPreferences
  });
})(typeof window === "undefined" ? globalThis : window);
