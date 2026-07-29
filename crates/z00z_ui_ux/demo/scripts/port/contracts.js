"use strict";

((root) => {
  const freezeList = (values) => Object.freeze([...values]);
  const freezeRecord = (entries) => Object.freeze(Object.fromEntries(entries));

  const VIEW_IDS = freezeList([
    "wallet",
    "wallet-send",
    "wallet-receive",
    "wallet-import",
    "wallet-merge-split",
    "activity",
    "swap",
    "staking",
    "wallet-backup",
    "wallet-settings",
    "settings",
    "telemetry",
    "data-storage",
    "about"
  ]);
  const WALLET_SECTION_IDS = freezeList(["assets", "vouchers", "permissions"]);
  const WALLET_SETTINGS_SECTION_IDS = freezeList(["general", "security", "backup", "policies", "advanced"]);
  const WALLET_CHAIN_OPTIONS = Object.freeze([
    Object.freeze({ id: "mainnet", label: "Mainnet", tone: "main" }),
    Object.freeze({ id: "testnet-1", label: "Testnet-1", tone: "test" }),
    Object.freeze({ id: "testnet-2", label: "Testnet-2", tone: "test" }),
    Object.freeze({ id: "devnet-1", label: "Devnet-1", tone: "dev" }),
    Object.freeze({ id: "devnet-2", label: "Devnet-2", tone: "dev" })
  ]);
  const WALLET_CHAIN_IDS = freezeList(WALLET_CHAIN_OPTIONS.map(({ id }) => id));
  const SETTINGS_SECTION_IDS = freezeList(["general", "reticulum", "onionnet", "quic", "notifications", "appearance"]);
  const NETWORK_SECTION_IDS = freezeList(["overview", "reticulum", "onionnet", "quic"]);
  const TELEMETRY_SOURCE_IDS = freezeList(["reticulum", "onionnet", "quic", "aggregators", "watchers", "explorer"]);
  const TELEMETRY_TAB_IDS = Object.freeze({
    reticulum: freezeList(["overview", "node", "interfaces", "radio", "entrypoints", "paths", "probes", "links"]),
    onionnet: freezeList(["overview", "epoch", "privacy", "transport", "queues", "probation", "ingress"]),
    quic: freezeList(["overview", "connections", "paths", "streams", "recovery", "security"]),
    aggregators: freezeList(["overview", "ingress", "planning", "placement", "publication", "recovery"]),
    watchers: freezeList(["overview", "alerts", "publication", "providers", "censorship", "evidence"]),
    explorer: freezeList(["overview", "search", "checkpoints", "batches", "evidence"])
  });
  const WALLET_ROUTE_IDS = freezeList([
    "wallet.assets",
    "wallet.vouchers",
    "wallet.permissions",
    "wallet.quarantine",
    "wallet.send",
    "wallet.receive",
    "wallet.import",
    "wallet.merge-split",
    "wallet.history",
    "wallet.swap",
    "wallet.staking.stake",
    "wallet.staking.unstake",
    "wallet.backup",
    "wallet.settings.general",
    "wallet.settings.security",
    "wallet.settings.backup",
    "wallet.settings.policies",
    "wallet.settings.advanced"
  ]);
  const TELEMETRY_ROUTE_IDS = freezeList([
    "telemetry.reticulum.overview",
    "telemetry.reticulum.node",
    "telemetry.reticulum.interfaces",
    "telemetry.reticulum.radio",
    "telemetry.reticulum.entrypoints",
    "telemetry.reticulum.paths",
    "telemetry.reticulum.probes",
    "telemetry.reticulum.links",
    "telemetry.onionnet.overview",
    "telemetry.onionnet.epoch",
    "telemetry.onionnet.privacy",
    "telemetry.onionnet.transport",
    "telemetry.onionnet.queues",
    "telemetry.onionnet.probation",
    "telemetry.onionnet.ingress",
    "telemetry.quic.overview",
    "telemetry.quic.connections",
    "telemetry.quic.paths",
    "telemetry.quic.streams",
    "telemetry.quic.recovery",
    "telemetry.quic.security",
    "telemetry.aggregators.overview",
    "telemetry.aggregators.ingress",
    "telemetry.aggregators.planning",
    "telemetry.aggregators.placement",
    "telemetry.aggregators.publication",
    "telemetry.aggregators.recovery",
    "telemetry.watchers.overview",
    "telemetry.watchers.alerts",
    "telemetry.watchers.publication",
    "telemetry.watchers.providers",
    "telemetry.watchers.censorship",
    "telemetry.watchers.evidence",
    "telemetry.explorer.overview",
    "telemetry.explorer.search",
    "telemetry.explorer.checkpoints",
    "telemetry.explorer.batches",
    "telemetry.explorer.evidence"
  ]);
  const DAPP_ROUTE_IDS = freezeList([
    "dapps.discover",
    "dapps.pay",
    "dapps.request",
    "dapps.create-voucher",
    "dapps.create-permission",
    "dapps.create-asset",
    "dapps.agents-budget",
    "dapps.wbold-gateway",
    "dapps.subscription",
    "dapps.donation",
    "dapps.escrow",
    "dapps.bounties",
    "dapps.tickets-passes",
    "dapps.service-credits",
    "dapps.digital-goods",
    "dapps.payroll",
    "dapps.private-contract",
    "dapps.assets-locker",
    "dapps.xchain-integration"
  ]);
  const MESSENGER_ROUTE_IDS = freezeList([
    "messenger.inbox",
    "messenger.sent",
    "messenger.conversations"
  ]);
  const CONTACTS_ROUTE_IDS = freezeList(["contacts.list"]);
  const DATA_STORAGE_ROUTE_IDS = freezeList(["data-storage.disk-usage", "data-storage.network-usage"]);
  const APP_SETTINGS_ROUTE_IDS = freezeList([
    "settings.general",
    "settings.reticulum",
    "settings.onionnet",
    "settings.quic",
    "settings.notifications",
    "settings.appearance"
  ]);
  const ABOUT_ROUTE_IDS = freezeList(["about"]);
  const HELP_ROUTE_IDS = freezeList(["help.root"]);
  const APP_ACTION_IDS = freezeList(["lock", "logout"]);
  const ROUTE_NAMESPACE_IDS = freezeList([
    "wallet",
    "reticulum",
    "onionnet",
    "quic",
    "aggregators",
    "watchers",
    "explorer",
    "dapps",
    "messenger",
    "data-storage",
    "contacts",
    "settings",
    "about"
  ]);
  const APP_ROUTE_IDS = freezeList([
    ...WALLET_ROUTE_IDS,
    ...TELEMETRY_ROUTE_IDS,
    ...DAPP_ROUTE_IDS,
    ...MESSENGER_ROUTE_IDS,
    ...DATA_STORAGE_ROUTE_IDS,
    ...CONTACTS_ROUTE_IDS,
    ...APP_SETTINGS_ROUTE_IDS,
    ...ABOUT_ROUTE_IDS
  ]);
  const DEFAULT_ROUTE_BY_NAMESPACE = freezeRecord([
    ["wallet", "wallet.assets"],
    ["reticulum", "telemetry.reticulum.overview"],
    ["onionnet", "telemetry.onionnet.overview"],
    ["quic", "telemetry.quic.overview"],
    ["aggregators", "telemetry.aggregators.overview"],
    ["watchers", "telemetry.watchers.overview"],
    ["explorer", "telemetry.explorer.overview"],
    ["dapps", "dapps.discover"],
    ["messenger", "messenger.inbox"],
    ["data-storage", "data-storage.disk-usage"],
    ["contacts", "contacts.list"],
    ["settings", "settings.general"],
    ["about", "about"]
  ]);
  const DIALOG_HELP_TOPIC_IDS = freezeList([
    "asset.details",
    "dapps.detail",
    "dapps.permission-review",
    "messenger.detail",
    "messenger.request-review",
    "contacts.detail",
    "contacts.identity-review",
    "telemetry.watchers.alert-detail",
    "telemetry.explorer.detail"
  ]);
  const ROUTE_HELP_TOPIC_IDS = APP_ROUTE_IDS.flatMap((routeId) => (
    routeId === "wallet.merge-split" ? ["wallet.merge", "wallet.split"] : [routeId]
  ));
  const HELP_TOPIC_IDS = freezeList(["app", ...ROUTE_HELP_TOPIC_IDS, ...DIALOG_HELP_TOPIC_IDS]);
  const APP_VERSION = "0.1.0";
  const PALETTE_IDS = freezeList(["z00z-default", "z00z-corporate"]);
  const MATURITY_IDS = freezeList(["live", "target", "concept"]);
  const AVAILABILITY_IDS = freezeList(["available", "degraded", "unavailable"]);
  const EVIDENCE_SOURCE_IDS = freezeList(["native", "fixture", "none"]);
  const FRESHNESS_IDS = freezeList(["timestamp", "stale", "unknown", "not_applicable"]);
  const PRESENTATION_MODE_IDS = freezeList(["product", "roadmap_preview"]);
  const TELEMETRY_RESULT_STATE_IDS = freezeList([
    "loading",
    "success",
    "degraded",
    "unavailable",
    "empty",
    "malformed",
    "error"
  ]);
  const LOCALE_IDS = freezeList(["en", "ru", "fr", "de", "es", "pt", "ko", "tr", "ja", "zh-Hans"]);
  const GATEWAY_QUERY_IDS = freezeList([
    "list_wallets",
    "load_wallet",
    "list_assets",
    "list_vouchers",
    "list_permissions",
    "list_activity",
    "get_receiver_card",
    "load_effective_config",
    "load_network_telemetry",
    "reconcile_operation"
  ]);
  const GATEWAY_COMMAND_IDS = freezeList([
    "create_wallet",
    "open_wallet",
    "restore_wallet",
    "remove_wallet_profiles",
    "rename_wallet",
    "change_wallet_password",
    "lock_wallet",
    "create_payment_draft",
    "approve_payment",
    "submit_payment",
    "import_asset",
    "create_voucher",
    "create_permission",
    "transfer_voucher",
    "transfer_permission",
    "create_backup",
    "apply_wallet_policy"
  ]);
  const GATEWAY_ERROR_CODES = freezeList([
    "validation",
    "authentication",
    "authorization",
    "unavailable_capability",
    "conflict",
    "timeout_unknown_outcome",
    "integrity",
    "internal"
  ]);
  const RENDERER_FORBIDDEN_STATE = freezeList([
    "password",
    "seed_phrase",
    "private_key",
    "session_token",
    "raw_signed_package",
    "arbitrary_filesystem_path"
  ]);
  const PRODUCTION_OWNERSHIP = Object.freeze({
    leptosRenderer: freezeList(["views", "ephemeral_ui_state", "focus", "routing", "presentation_models"]),
    tauriBridge: freezeList(["window_lifecycle", "allowlisted_commands", "sanitized_events", "native_capabilities"]),
    nativeGateway: freezeList(["authentication", "authorization", "session_tokens", "config_mutation", "operation_reconciliation"]),
    walletBackend: freezeList(["wallet_files", "keys", "seeds", "signing", "policy_enforcement", "settlement"])
  });

  function allowed(value, values, fallback) {
    return values.includes(value) ? value : fallback;
  }

  function resolveInitialNavigation(search = "") {
    const params = new URLSearchParams(search);
    const view = allowed(params.get("view"), VIEW_IDS, "wallet");
    const walletSection = allowed(params.get("wallet"), WALLET_SECTION_IDS, "assets");
    const walletSettingsSection = allowed(params.get("walletSettings"), WALLET_SETTINGS_SECTION_IDS, "general");
    const networkSection = allowed(params.get("network"), NETWORK_SECTION_IDS, "overview");
    const requestedSettings = params.get("settings");
    const settingsSection = requestedSettings === "network"
      ? (["reticulum", "onionnet", "quic"].includes(networkSection) ? networkSection : "reticulum")
      : allowed(requestedSettings, SETTINGS_SECTION_IDS, "general");
    const telemetrySource = allowed(params.get("telemetry"), TELEMETRY_SOURCE_IDS, "onionnet");

    return Object.freeze({
      view,
      walletSection,
      walletSettingsSection,
      settingsSection,
      networkSection: ["reticulum", "onionnet", "quic"].includes(settingsSection) ? settingsSection : networkSection,
      telemetrySource,
      reticulumTelemetryTab: allowed(params.get("reticulumTab"), TELEMETRY_TAB_IDS.reticulum, "overview"),
      onionnetTelemetryTab: allowed(params.get("onionTab"), TELEMETRY_TAB_IDS.onionnet, "overview"),
      quicTelemetryTab: allowed(params.get("quicTab"), TELEMETRY_TAB_IDS.quic, "overview"),
      aggregatorsTelemetryTab: allowed(params.get("aggregatorsTab"), TELEMETRY_TAB_IDS.aggregators, "overview"),
      watchersTelemetryTab: allowed(params.get("watchersTab"), TELEMETRY_TAB_IDS.watchers, "overview"),
      explorerTelemetryTab: allowed(params.get("explorerTab"), TELEMETRY_TAB_IDS.explorer, "overview")
    });
  }

  const PORT_CONTRACT = Object.freeze({
    version: "1.2.0",
    appVersion: APP_VERSION,
    rendererRuntime: "leptos-csr-wasm",
    packagedHost: "tauri-2",
    browserProduct: false,
    walletBackendRuntime: "native-rust",
    views: VIEW_IDS,
    walletSections: WALLET_SECTION_IDS,
    walletSettingsSections: WALLET_SETTINGS_SECTION_IDS,
    walletChains: WALLET_CHAIN_IDS,
    settingsSections: SETTINGS_SECTION_IDS,
    networkSections: NETWORK_SECTION_IDS,
    telemetrySources: TELEMETRY_SOURCE_IDS,
    telemetryTabs: TELEMETRY_TAB_IDS,
    routes: APP_ROUTE_IDS,
    routeNamespaces: ROUTE_NAMESPACE_IDS,
    walletRoutes: WALLET_ROUTE_IDS,
    telemetryRoutes: TELEMETRY_ROUTE_IDS,
    dappRoutes: DAPP_ROUTE_IDS,
    messengerRoutes: MESSENGER_ROUTE_IDS,
    contactsRoutes: CONTACTS_ROUTE_IDS,
    dataStorageRoutes: DATA_STORAGE_ROUTE_IDS,
    settingsRoutes: APP_SETTINGS_ROUTE_IDS,
    aboutRoutes: ABOUT_ROUTE_IDS,
    helpRoutes: HELP_ROUTE_IDS,
    helpTopics: HELP_TOPIC_IDS,
    actions: APP_ACTION_IDS,
    defaultRouteByNamespace: DEFAULT_ROUTE_BY_NAMESPACE,
    palettes: PALETTE_IDS,
    maturity: MATURITY_IDS,
    availability: AVAILABILITY_IDS,
    evidenceSources: EVIDENCE_SOURCE_IDS,
    freshness: FRESHNESS_IDS,
    presentationModes: PRESENTATION_MODE_IDS,
    telemetryResultStates: TELEMETRY_RESULT_STATE_IDS,
    locales: LOCALE_IDS,
    gatewayQueries: GATEWAY_QUERY_IDS,
    gatewayCommands: GATEWAY_COMMAND_IDS,
    gatewayErrorCodes: GATEWAY_ERROR_CODES,
    rendererForbiddenState: RENDERER_FORBIDDEN_STATE,
    productionOwnership: PRODUCTION_OWNERSHIP,
    desktopTransport: "authenticated-os-ipc",
    iosTransport: "typed-in-process",
    forbiddenTransports: freezeList(["http", "https", "websocket", "tcp", "browser-rpc"]),
    routeDefaults: freezeRecord([
      ["view", "wallet"],
      ["walletSection", "assets"],
      ["walletSettingsSection", "general"],
      ["settingsSection", "general"],
      ["networkSection", "overview"],
      ["telemetrySource", "onionnet"]
    ])
  });

  Object.assign(root.Z00ZDemo ||= {}, {
    APP_VERSION,
    PORT_CONTRACT,
    WALLET_CHAIN_OPTIONS,
    resolveInitialNavigation
  });
})(typeof window === "undefined" ? globalThis : window);
