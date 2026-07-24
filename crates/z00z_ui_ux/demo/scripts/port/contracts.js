"use strict";

((root) => {
  const freezeList = (values) => Object.freeze([...values]);
  const freezeRecord = (entries) => Object.freeze(Object.fromEntries(entries));

  const VIEW_IDS = freezeList([
    "home",
    "wallet",
    "wallet-send",
    "wallet-receive",
    "activity",
    "swap",
    "exchange",
    "staking",
    "wallet-backup",
    "wallet-settings",
    "settings",
    "telemetry"
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
const SETTINGS_SECTION_IDS = freezeList(["general", "reticulum", "onionnet", "appearance"]);
  const NETWORK_SECTION_IDS = freezeList(["overview", "reticulum", "onionnet"]);
  const TELEMETRY_SOURCE_IDS = freezeList(["onionnet", "reticulum", "aggregators"]);
  const TELEMETRY_TAB_IDS = Object.freeze({
    reticulum: freezeList(["overview", "node", "interfaces", "radio", "entrypoints", "paths", "probes", "links"]),
    onionnet: freezeList(["overview", "epoch", "privacy", "transport", "queues", "probation", "ingress"]),
    aggregators: freezeList(["overview"])
  });
  const CAPABILITY_STATES = freezeList(["live", "target", "unavailable", "degraded"]);
  const GATEWAY_QUERY_IDS = freezeList([
    "load_home",
    "list_wallets",
    "load_wallet",
    "list_assets",
    "list_vouchers",
    "list_permissions",
    "list_activity",
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
      ? (networkSection === "onionnet" ? "onionnet" : "reticulum")
      : allowed(requestedSettings, SETTINGS_SECTION_IDS, "general");
    const telemetrySource = allowed(params.get("telemetry"), TELEMETRY_SOURCE_IDS, "onionnet");

    return Object.freeze({
      view,
      walletSection,
      walletSettingsSection,
      settingsSection,
      networkSection: ["reticulum", "onionnet"].includes(settingsSection) ? settingsSection : networkSection,
      telemetrySource,
      reticulumTelemetryTab: allowed(params.get("reticulumTab"), TELEMETRY_TAB_IDS.reticulum, "overview"),
      onionnetTelemetryTab: allowed(params.get("onionTab"), TELEMETRY_TAB_IDS.onionnet, "overview"),
      aggregatorsTelemetryTab: allowed(params.get("aggregatorsTab"), TELEMETRY_TAB_IDS.aggregators, "overview")
    });
  }

  const PORT_CONTRACT = Object.freeze({
    version: "1.2.0",
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
    capabilityStates: CAPABILITY_STATES,
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
    PORT_CONTRACT,
    WALLET_CHAIN_OPTIONS,
    resolveInitialNavigation
  });
})(typeof window === "undefined" ? globalThis : window);
