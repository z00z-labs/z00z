"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.createInitialWallets || !demo.canonicalRouteFromLegacyNavigation) {
    throw new Error("Z00Z contracts, navigation model, and fixtures must load before presentation state.");
  }

  const PALETTE_OPTIONS = Object.freeze([
    Object.freeze({
      id: "z00z-default",
      label: "Z00Z Default",
      colorScheme: "dark"
    }),
    Object.freeze({
      id: "z00z-corporate",
      label: "Z00Z Corporate",
      colorScheme: "light"
    })
  ]);

  const CODE_THEME_OPTIONS = Object.freeze([
    Object.freeze({ id: "atom-one-light", label: "One Light", description: "Bright technical surface with magenta, amber, violet, and green syntax.", mode: "light" }),
    Object.freeze({ id: "xcode", label: "Xcode", description: "Light Apple-style syntax with green comments and crisp blue numerics.", mode: "light" }),
    Object.freeze({ id: "atom-one-dark", label: "One Dark", description: "Deep blue-black surface with Monokai pink, amber, violet, and green syntax.", mode: "dark" }),
    Object.freeze({ id: "night-owl", label: "Night Owl", description: "Deep dark technical surface with muted violet, sand, and orange tokens.", mode: "dark" })
  ]);

  const SHELL_ACTION_TYPES = Object.freeze([
    "toggle_branch",
    "select_leaf",
    "restore_route",
    "switch_wallet",
    "set_drawer",
    "set_palette",
    "begin_request",
    "cancel_request",
    "lock",
    "logout"
  ]);

  function sortBranchIds(branchIds) {
    return [...new Set(branchIds)].sort();
  }

  function paletteOption(paletteId) {
    return PALETTE_OPTIONS.find(({ id }) => id === paletteId) || PALETTE_OPTIONS[0];
  }

  function resolvePalettePreference({ palette, theme } = {}) {
    if (palette === "z00z-corporate") return "z00z-corporate";
    if (theme === "light") return "z00z-corporate";
    return "z00z-default";
  }

  function resolveInitialPalettePreference(search = "") {
    const params = new URLSearchParams(search);
    return resolvePalettePreference({
      palette: params.get("palette"),
      theme: params.get("theme")
    });
  }

  function defaultShellState(navigation, activeWalletId = "everyday", palette = "z00z-default") {
    const activeRoute = demo.canonicalRouteFromLegacyNavigation(navigation);
    const routeNode = demo.navigationNodeForRoute(activeRoute);
    return {
      activeRoute,
      expandedBranchIds: sortBranchIds(routeNode ? demo.ancestorBranchIdsForNode(routeNode.id) : ["wallet"]),
      drawerOpen: false,
      activeWalletId,
      palette: resolvePalettePreference({ palette }),
      requestGenerations: {},
      cancelledRequestKeys: [],
      shellPreferences: {
        language: "en",
        textScale: "100",
        reducedMotion: false,
        codeTheme: "atom-one-dark",
        sensitiveValuesHidden: false
      },
      locked: false
    };
  }

  function normalizeShellState(shell) {
    const fallback = defaultShellState({});
    const activeRoute = demo.PORT_CONTRACT.routes.includes(shell?.activeRoute)
      ? shell.activeRoute
      : fallback.activeRoute;
    return {
      ...fallback,
      ...shell,
      activeRoute,
      expandedBranchIds: sortBranchIds(shell?.expandedBranchIds || fallback.expandedBranchIds),
      drawerOpen: Boolean(shell?.drawerOpen),
      palette: demo.PORT_CONTRACT.palettes.includes(shell?.palette) ? shell.palette : fallback.palette,
      requestGenerations: { ...(shell?.requestGenerations || {}) },
      cancelledRequestKeys: sortBranchIds(shell?.cancelledRequestKeys || []),
      shellPreferences: { ...fallback.shellPreferences, ...(shell?.shellPreferences || {}) },
      locked: Boolean(shell?.locked)
    };
  }

  function reduceShellState(shell, action = {}) {
    const current = normalizeShellState(shell);
    const next = {
      ...current,
      requestGenerations: { ...current.requestGenerations },
      cancelledRequestKeys: [...current.cancelledRequestKeys],
      shellPreferences: { ...current.shellPreferences }
    };
    const type = action.type;
    if (!SHELL_ACTION_TYPES.includes(type)) return next;

    if (type === "toggle_branch") {
      const branch = demo.navigationNode(action.nodeId);
      if (branch?.target.kind !== "branch" || branch.parentId !== null) return next;
      const expanded = new Set(next.expandedBranchIds);
      if (expanded.has(branch.id)) expanded.delete(branch.id);
      else expanded.add(branch.id);
      next.expandedBranchIds = sortBranchIds(expanded);
      return next;
    }

    if (type === "select_leaf" || type === "restore_route") {
      const routeId = type === "select_leaf"
        ? demo.navigationNode(action.nodeId)?.target.routeId
        : action.routeId;
      const routeNode = demo.navigationNodeForRoute(routeId);
      if (!routeNode) return next;
      next.activeRoute = routeId;
      next.expandedBranchIds = sortBranchIds([
        ...next.expandedBranchIds,
        ...demo.ancestorBranchIdsForNode(routeNode.id)
      ]);
      if (type === "select_leaf" || type === "restore_route") next.drawerOpen = false;
      return next;
    }

    if (type === "switch_wallet") {
      if (!action.walletId) return next;
      next.activeWalletId = action.walletId;
      if (!demo.isWalletRoute(next.activeRoute) || action.walletRouteCompatible === false) {
        next.activeRoute = demo.PORT_CONTRACT.defaultRouteByNamespace.wallet;
        next.expandedBranchIds = sortBranchIds([
          ...next.expandedBranchIds,
          ...demo.ancestorBranchIdsForNode("wallet.assets-rights")
        ]);
      }
      return next;
    }

    if (type === "set_drawer") {
      next.drawerOpen = Boolean(action.open);
      return next;
    }

    if (type === "set_palette") {
      if (demo.PORT_CONTRACT.palettes.includes(action.palette)) next.palette = action.palette;
      return next;
    }

    if (type === "begin_request") {
      const requestKey = String(action.requestKey || "");
      if (!requestKey) return next;
      next.requestGenerations[requestKey] = Number(next.requestGenerations[requestKey] || 0) + 1;
      next.cancelledRequestKeys = next.cancelledRequestKeys.filter((key) => key !== requestKey);
      return next;
    }

    if (type === "cancel_request") {
      const requestKey = String(action.requestKey || "");
      if (!requestKey) return next;
      next.cancelledRequestKeys = sortBranchIds([...next.cancelledRequestKeys, requestKey]);
      return next;
    }

    if (type === "lock") {
      next.drawerOpen = false;
      next.locked = true;
      return next;
    }

    if (type === "logout") {
      const reset = defaultShellState({}, null);
      return {
        ...reset,
        palette: next.palette,
        shellPreferences: { ...next.shellPreferences },
        locked: true
      };
    }

    return next;
  }

  function createRequestKey({
    domain,
    routeId,
    walletId = "none",
    scope = "default"
  } = {}) {
    const values = [domain, routeId, walletId, scope].map((value) => String(value || ""));
    if (!values[0] || !demo.PORT_CONTRACT.routes.includes(values[1])) {
      throw new TypeError("Request keys require a domain and canonical route.");
    }
    if (values.some((value) => !/^[A-Za-z0-9._-]+$/.test(value))) {
      throw new TypeError("Request key fields must be bounded identifiers.");
    }
    return values.join("|");
  }

  function requestResultIsCurrent(shell, {
    requestKey,
    generation,
    routeId,
    walletId = "none"
  } = {}) {
    const current = normalizeShellState(shell);
    return !current.locked
      && current.activeRoute === routeId
      && String(current.activeWalletId || "none") === String(walletId || "none")
      && !current.cancelledRequestKeys.includes(requestKey)
      && Number.isSafeInteger(generation)
      && generation === Number(current.requestGenerations[requestKey] || 0);
  }

  function createInitialState({ search = "" } = {}) {
    const navigation = demo.resolveInitialNavigation(search);
    const palette = resolveInitialPalettePreference(search);
    const shell = defaultShellState(navigation, "everyday", palette);
    const params = new URLSearchParams(search);
    const requestedOperationScenario = params.get("operationScenario");
    const requestedWatcherScenario = params.get("watcherScenario");
    const requestedWatcherSource = params.get("watcherSource");
    const requestedExplorerScenario = params.get("explorerScenario");
    const requestedMessengerRelayScenario = params.get("messengerRelay");
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
      quicTelemetryTab: navigation.quicTelemetryTab,
      aggregatorsTelemetryTab: navigation.aggregatorsTelemetryTab,
      watchersTelemetryTab: navigation.watchersTelemetryTab,
      explorerTelemetryTab: navigation.explorerTelemetryTab,
      watcherScenario: demo.PORT_CONTRACT.telemetryResultStates.includes(requestedWatcherScenario)
        ? requestedWatcherScenario
        : "success",
      watcherSourceId: ["runtime_projection", "evidence_archive"].includes(requestedWatcherSource)
        ? requestedWatcherSource
        : "runtime_projection",
      watcherSeverityFilter: "all",
      watcherKindFilter: "all",
      watcherSelectedAlertId: null,
      watcherExportEnvelope: null,
      explorerScenario: demo.PORT_CONTRACT.telemetryResultStates.includes(requestedExplorerScenario)
        ? requestedExplorerScenario
        : "success",
      explorerQuery: "",
      explorerSearchResult: null,
      explorerSelectedPublicId: null,
      explorerEvidenceKindFilter: "all",
      explorerDetailMode: "summary",
      dappSection: "discover",
      dappScreen: "list",
      dappSelectedId: null,
      dappReviewConnectionId: null,
      dappReviewDecision: null,
      dappReviewValidationError: null,
      dappReviewAcknowledgements: {
        scopeConfirmed: false,
        reauthAcknowledged: false
      },
      dappLastOutcome: null,
      dappWalletReviewHandoff: null,
      messengerSection: "inbox",
      messengerScreen: "list",
      messengerQuery: "",
      messengerSelectedMessageId: null,
      messengerReviewDecision: null,
      messengerReviewError: null,
      messengerLastOutcome: null,
      messengerWalletReviewHandoff: null,
      messengerDeletedIds: [],
      messengerAcknowledgedIds: [],
      messengerReportedIds: [],
      messengerBlockedSenders: [],
      messengerRelayScenario: demo.MESSENGER_RELAY_STATES.some(({ id }) => id === requestedMessengerRelayScenario)
        ? requestedMessengerRelayScenario
        : "available",
      contacts: demo.createInitialContacts(),
      contactsQuery: "",
      contactsStatus: "all",
      contactsSort: "nickname",
      contactsScreen: "list",
      contactsSelectedId: null,
      contactsImportSourceId: "receiver_card",
      contactsFormError: null,
      contactsLastOutcome: null,
      contactActionHandoff: null,
      isNetworkOpen: ["reticulum", "onionnet", "quic"].includes(navigation.settingsSection),
      palette: shell.palette,
      activeRoute: shell.activeRoute,
      expandedBranchIds: shell.expandedBranchIds,
      drawerOpen: shell.drawerOpen,
      activeWalletId: shell.activeWalletId,
      requestGenerations: shell.requestGenerations,
      cancelledRequestKeys: shell.cancelledRequestKeys,
      shellPreferences: shell.shellPreferences,
      language: "en",
      regionalLocale: "en-US",
      valuationCurrency: "USD",
      timeZone: "UTC",
      networkUnits: "decimal-bps",
      reticulumInterface: "automatic",
      reticulumDiscoveryScope: "link",
      reticulumPeerDiscovery: true,
      reticulumLinkStrategy: "persistent-ingress",
      onionnetPrivacyMode: "private",
      onionnetRouteRotation: "session",
      onionnetCoverTraffic: "adaptive",
      onionnetFailClosed: true,
      quicEndpointPolicy: "automatic",
      quicPathMigration: true,
      quicIdleTimeout: "30",
      quicKeepAlive: "15",
      notifications: true,
      vibrate: "messages-and-alerts",
      ringtone: "z00z-pulse",
      autoLockMinutes: "15",
      textScale: "100",
      reducedMotion: false,
      codeTheme: "atom-one-dark",
      configView: "yaml",
      configDraft: "",
      walletSettingsConfigDraft: "",
      configStatus: "Local draft is in sync with the visible controls.",
      walletPreferences: {},
      sendDrafts: {},
      assetImport: {
        walletId: "",
        status: "idle",
        fileName: "",
        fileSize: 0,
        reviewToken: "",
        preview: null,
        error: null,
        result: null
      },
      assetMergeSplit: {
        walletId: "",
        mode: "merge",
        selectedMergeIds: [],
        selectedSplitId: "",
        splitAmounts: ["", ""],
        preview: null,
        error: ""
      },
      exchangeDrafts: {},
      demoOperationScenario: requestedOperationScenario === "timeout_unknown_outcome"
        ? requestedOperationScenario
        : "success",
      locked: false,
      dataStorageSection: "disk-usage",
      updateCheckStatus: "idle",
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
    SHELL_ACTION_TYPES,
    paletteOption,
    resolvePalettePreference,
    resolveInitialPalettePreference,
    createInitialState,
    defaultShellState,
    activeWallet,
    ensureWalletPreferences,
    reduceShellState,
    createRequestKey,
    requestResultIsCurrent
  });
})(typeof window === "undefined" ? globalThis : window);
