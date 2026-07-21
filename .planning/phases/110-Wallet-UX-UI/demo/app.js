"use strict";

const main = document.querySelector("#main-content");
const pageTitle = document.querySelector("#page-title");
const pageContext = document.querySelector("#page-context");
const copyWalletAddress = document.querySelector("#copy-wallet-address");
const walletNav = document.querySelector("#wallet-nav");
const networkNav = document.querySelector("#network-nav");
const walletTabs = document.querySelector("#wallet-tabs");
const walletIdentity = document.querySelector("#wallet-identity");
const walletStatusbar = document.querySelector("#wallet-statusbar");
const lockWalletLabel = document.querySelector("#lock-wallet-label");
const dialog = document.querySelector("#flow-dialog");
const dialogContent = document.querySelector("#dialog-content");
const appShell = document.querySelector("#app-shell");
const lockScreen = document.querySelector("#lock-screen");
const demoParams = new URLSearchParams(window.location.search);
const requestedView = ["home", "wallet", "activity", "swap", "exchange", "staking", "wallet-backup", "wallet-settings", "settings", "add-wallet"].includes(demoParams.get("view")) ? demoParams.get("view") : "wallet";
const requestedWalletSection = ["assets", "vouchers", "permissions"].includes(demoParams.get("wallet")) ? demoParams.get("wallet") : "assets";
const requestedWalletSettingsSection = ["general", "security", "backup", "policies", "advanced"].includes(demoParams.get("walletSettings")) ? demoParams.get("walletSettings") : "general";
const requestedSettingsSection = ["general", "security", "network", "policies", "backup", "appearance", "advanced"].includes(demoParams.get("settings")) ? demoParams.get("settings") : "general";
const requestedNetworkSection = ["overview", "reticulum", "onionnet", "carriers"].includes(demoParams.get("network")) ? demoParams.get("network") : "overview";

const paletteOptions = [
  { id: "z00z-default", label: "Z00Z Default", description: "Current private-wallet palette", swatches: ["#081019", "#101D29", "#E3B341", "#32A9E8", "#F5F7F8"] },
  { id: "black-gold-elegance", label: "Black & Gold", description: "Black, navy, and restrained gold", swatches: ["#000000", "#14213D", "#FCA311", "#E5E5E5", "#FFFFFF"] },
  { id: "deep-blue-sea", label: "Deep Blue Sea", description: "Layered blue with cool neutrals", swatches: ["#134074", "#13315C", "#0B2545", "#8DA9C4", "#EEF4ED"] },
  { id: "golden-twilight", label: "Golden Twilight", description: "Near-black, deep blue, and gold", swatches: ["#000814", "#001D3D", "#003566", "#FFC300", "#FFD60A"] },
  { id: "midnight-sky", label: "Midnight Sky", description: "Midnight blue with luminous gold", swatches: ["#00296B", "#003F88", "#00509D", "#FDC500", "#FFD500"] }
];

const codeThemeOptions = [
  { id: "atom-one-light", label: "One Light", description: "Bright technical surface with magenta, amber, violet, and green syntax.", mode: "light", background: "#FAFAFA", border: "#D6D6D6", foreground: "#0D161B", comment: "#64560D", keyword: "#F23173", string: "#BD5200", number: "#CF4DFF", title: "#007400" },
  { id: "xcode", label: "Xcode", description: "Light Apple-style syntax with green comments and crisp blue numerics.", mode: "light", background: "#FAFAFA", border: "#D6D6D6", foreground: "#000000", comment: "#007400", keyword: "#AA0D91", string: "#C41A16", number: "#1C00CF", title: "#1C00CF" },
  { id: "atom-one-dark", label: "One Dark", description: "Deep blue-black surface with Monokai pink, amber, violet, and green syntax.", mode: "dark", background: "#0D161B", border: "#181A1F", foreground: "#CACED6", comment: "#75715E", keyword: "#F92672", string: "#E5C07B", number: "#C678DD", title: "#98C379" },
  { id: "night-owl", label: "Night Owl", description: "Deep dark technical surface with muted violet, sand, and orange tokens.", mode: "dark", background: "#011627", border: "#13344A", foreground: "#D6DEEB", comment: "#637777", keyword: "#C792EA", string: "#ECC48D", number: "#F78C6C", title: "#DCDCAA" }
];

const state = {
  view: requestedView,
  balanceHidden: false,
  expertDetails: false,
  activityFilter: "all",
  walletSection: requestedWalletSection,
  walletSettingsSection: requestedWalletSettingsSection,
  settingsSection: requestedSettingsSection,
  networkSection: requestedNetworkSection,
  isNetworkOpen: requestedSettingsSection === "network",
  theme: "dark",
  palette: "z00z-default",
  codeTheme: "atom-one-dark",
  language: "English",
  notifications: true,
  autoLockMinutes: "15",
  textScale: "100",
  reducedMotion: false,
  compactLists: false,
  configView: "yaml",
  configDraft: "",
  walletSettingsConfigDraft: "",
  configStatus: "Local draft is in sync with the visible controls.",
  hasCustomAppearance: false,
  customAppearance: { brand: "#e3b341", rail: "#32a9e8" },
  walletPreferences: {},
  locked: false,
  flow: null,
  lastDialogTrigger: null,
  selectedWalletId: "everyday",
  wallets: [
    {
      id: "everyday",
      name: "Everyday",
      initials: "E",
      address: "ZxChpo…2Mj8Pt",
      fullAddress: "ZxChpoioBEFR1PRJPamJxh5aWdEb94ek8J52PmT8PYAEa8RKVtSs9X3UPgaSaHvMMZKcQoiyVFhEE256vcyGPeFV23d2Mj8Pt",
      summary: { available: "12,480.75", locked: "0.00", pendingIn: "960.00", pendingOut: "240.00", scan: "Current" },
      activities: [
        { id: "tx-7f31", type: "money", direction: "out", title: "Payment to Mira", detail: "Sent · waiting to settle", amount: "− 240.00 Z00Z", time: "2 min", status: "settling" },
        { id: "claim-014", type: "asset", direction: "in", title: "Allocation claimed", detail: "Verified claim · waiting to settle", amount: "+ 86.00 Z00Z", time: "18 min", status: "settling" },
        { id: "tx-7e88", type: "money", direction: "in", title: "Received from Niko", detail: "Settled", amount: "+ 1,200.00 Z00Z", time: "Yesterday", status: "settled" },
        { id: "voucher-221", type: "voucher", direction: "neutral", title: "Travel refund voucher", detail: "Offered · review before 21 Jul", amount: "86.00 Z00Z", time: "Yesterday", status: "attention" },
        { id: "right-221", type: "permission", direction: "neutral", title: "Delivery receipt access", detail: "Data access · 2 of 5 uses remain", amount: "2 uses", time: "Yesterday", status: "active" },
        { id: "tx-7d12", type: "money", direction: "out", title: "Payment to Coffee Lab", detail: "Settled", amount: "− 18.50 Z00Z", time: "12 Jul", status: "settled" },
        { id: "security-4", type: "security", direction: "neutral", title: "Local backup created", detail: "Integrity check passed", amount: "", time: "10 Jul", status: "settled" }
      ]
    },
    {
      id: "savings",
      name: "Savings",
      initials: "S",
      address: "ZxR5vK…8Ee1Qm",
      fullAddress: "ZxR5vKpyP2W6eT8fVqH8M9sB7cX4aL2nQ5rD1uEe1Qm",
      summary: { available: "7,215.00", locked: "1,400.00", pendingIn: "0.00", pendingOut: "0.00", scan: "Current" },
      activities: [
        { id: "saving-100", type: "money", direction: "in", title: "Transfer from Everyday", detail: "Settled", amount: "+ 2,000.00 Z00Z", time: "3 Jul", status: "settled" },
        { id: "saving-101", type: "security", direction: "neutral", title: "Recovery check completed", detail: "Local verification passed", amount: "", time: "30 Jun", status: "settled" }
      ]
    },
    {
      id: "travel",
      name: "Travel",
      initials: "T",
      address: "ZxT8cQ…4Fh2Ns",
      fullAddress: "ZxT8cQy6BvR3sL9wE1mD5hK7pA4Fh2Ns",
      summary: { available: "860.00", locked: "0.00", pendingIn: "125.00", pendingOut: "0.00", scan: "Scanning" },
      activities: [
        { id: "travel-100", type: "money", direction: "in", title: "Received from Niko", detail: "Waiting to settle", amount: "+ 125.00 Z00Z", time: "8 min", status: "settling" },
        { id: "travel-101", type: "money", direction: "out", title: "Payment to RailLink", detail: "Settled", amount: "− 74.50 Z00Z", time: "Yesterday", status: "settled" }
      ]
    }
  ]
};

const headings = {
  home: ["Home", "Your private money at a glance"],
  wallet: ["Wallet", "Assets, vouchers, and permissions stay distinct"],
  activity: ["History", "Asset, voucher, permission, policy, and security events"],
  swap: ["Swap", "Move value between assets in this wallet"],
  exchange: ["Exchange", "Compare external exchange routes for this wallet"],
  staking: ["Staking", "Put selected wallet value to work with clear terms"],
  "wallet-backup": ["Backup", "Protect the selected wallet with a verified local backup"],
  "wallet-settings": ["Wallet settings", "Configure this wallet without changing other local profiles"],
  settings: ["Settings", "Common controls and advanced configuration"],
  "add-wallet": ["Add wallet", "Create, open, or restore a local wallet"]
};

function activeWallet() {
  return state.wallets.find((wallet) => wallet.id === state.selectedWalletId) || state.wallets[0] || {
    id: "empty",
    name: "",
    initials: "",
    address: "",
    fullAddress: "",
    summary: { available: "0.00", locked: "0.00", pendingIn: "0.00", pendingOut: "0.00", scan: "Unavailable" },
    activities: []
  };
}

function activeWalletPreferences() {
  const wallet = activeWallet();
  if (!state.walletPreferences[wallet.id]) {
    state.walletPreferences[wallet.id] = {
      currency: "Z00Z",
      defaultFee: "0.001",
      autoBackup: false,
      backupIntervalHours: "24",
      lockAfterMinutes: state.autoLockMinutes,
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
    };
  }
  return state.walletPreferences[wallet.id];
}

function yamlScalar(value) {
  return String(value).replaceAll('"', '\\"');
}

function effectiveDemoConfigYaml() {
  const wallet = activeWallet();
  const walletPreferences = activeWalletPreferences();
  return [
    "schema_version: 1",
    "",
    "app:",
    "  general:",
    `    language: \"${yamlScalar(state.language)}\"`,
    `    notifications: ${state.notifications}`,
    "  appearance:",
    `    theme: ${state.theme}`,
    `    palette: ${state.palette}`,
    `    code_theme: ${state.codeTheme}`,
    `    custom_enabled: ${state.hasCustomAppearance}`,
    `    custom_brand: "${state.customAppearance.brand}"`,
    `    custom_rail: "${state.customAppearance.rail}"`,
    `    text_scale: ${state.textScale}`,
    `    reduced_motion: ${state.reducedMotion}`,
    `    compact_desktop_lists: ${state.compactLists}`,
    "  security:",
    `    lock_after_minutes: ${state.autoLockMinutes}`,
    "",
    "wallet:",
    `  id: \"${yamlScalar(wallet.id)}\"`,
    "  display:",
    `    name: \"${yamlScalar(wallet.name)}\"`,
    `    currency: ${walletPreferences.currency}`,
    "  transactions:",
    `    default_fee: \"${yamlScalar(walletPreferences.defaultFee)}\"`,
    "  security:",
    `    lock_after_minutes: ${walletPreferences.lockAfterMinutes}`,
    "  backup:",
    `    auto_backup: ${walletPreferences.autoBackup}`,
    `    interval_hours: ${walletPreferences.backupIntervalHours}`,
    "    encrypt: true",
    "  policy_rules:",
    `    max_transaction: \"${yamlScalar(walletPreferences.policyRules.maxTransaction)}\"`,
    `    max_daily: \"${yamlScalar(walletPreferences.policyRules.maxDaily)}\"`,
    `    require_confirmation: ${walletPreferences.policyRules.requireConfirmation}`,
    `    allowed_assets: ${walletPreferences.policyRules.allowedAssets}`,
    `    allowed_recipients: \"${yamlScalar(walletPreferences.policyRules.allowedRecipients || "any")}\"`,
    `    time_restrictions: ${walletPreferences.policyRules.timeWindow}`,
    "  compliance_profile:",
    `    preview: \"${yamlScalar(walletPreferences.policyProfile)}\"`,
    "  privacy:",
    `    hide_sensitive_amounts: ${state.balanceHidden}`,
    "  advanced:",
    `    expert_details: ${state.expertDetails}`,
    "",
    "# Secrets, local paths, session tokens, and receiver material are excluded."
  ].join("\n");
}

function syncConfigDraftFromState() {
  state.configDraft = effectiveDemoConfigYaml();
  state.configStatus = "Local draft is in sync with the visible controls.";
}

function applyAppearancePreferences() {
  const root = document.documentElement;
  const effectiveTheme = state.theme === "system"
    ? (window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark")
    : state.theme;
  root.dataset.theme = effectiveTheme;
  root.dataset.palette = state.palette;
  root.dataset.codeTheme = state.codeTheme;
  root.dataset.textScale = state.textScale;
  root.dataset.reducedMotion = String(state.reducedMotion);
  root.dataset.compactLists = String(state.compactLists);
  if (state.hasCustomAppearance) {
    root.style.setProperty("--brand", state.customAppearance.brand);
    root.style.setProperty("--rail", state.customAppearance.rail);
  } else {
    root.style.removeProperty("--brand");
    root.style.removeProperty("--rail");
  }
  const themeColor = getComputedStyle(root).getPropertyValue("--bg-canvas").trim() || (effectiveTheme === "dark" ? "#081019" : "#F4F7FA");
  document.querySelector('meta[name="theme-color"]').content = themeColor;
}

function hexToRgb(value) {
  const normalized = value.replace("#", "");
  if (!/^[0-9a-f]{6}$/i.test(normalized)) return null;
  return [0, 2, 4].map((index) => Number.parseInt(normalized.slice(index, index + 2), 16) / 255);
}

function relativeLuminance(value) {
  const rgb = hexToRgb(value);
  if (!rgb) return null;
  const channels = rgb.map((channel) => (channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4));
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function hasSafeControlContrast(value) {
  const background = getComputedStyle(document.documentElement).getPropertyValue("--bg-canvas").trim();
  const foregroundLum = relativeLuminance(value);
  const backgroundLum = relativeLuminance(background);
  if (foregroundLum === null || backgroundLum === null) return false;
  return (Math.max(foregroundLum, backgroundLum) + 0.05) / (Math.min(foregroundLum, backgroundLum) + 0.05) >= 3;
}

function readYamlScalar(source, key) {
  const match = source.match(new RegExp(`^\\s*${key}:\\s*(?:\\"([^\\"]*)\\"|([^#\\n]+))`, "m"));
  return match ? (match[1] ?? match[2]).trim() : null;
}

function validateAndApplyDemoConfig(source, apply = false) {
  const forbidden = /(^|\n)\s*(password|seed|private_key|session_token|receiver_secret|path):/i;
  if (!/^schema_version:\s*1\s*$/m.test(source)) return { valid: false, message: "Use schema_version: 1." };
  if (!/^app:\s*$/m.test(source) || !/^wallet:\s*$/m.test(source)) return { valid: false, message: "App and wallet sections are required." };
  if (forbidden.test(source)) return { valid: false, message: "Secrets and local paths are not allowed in this configuration." };

  const theme = readYamlScalar(source, "theme");
  const palette = readYamlScalar(source, "palette");
  const codeTheme = readYamlScalar(source, "code_theme");
  const language = readYamlScalar(source, "language");
  const textScale = readYamlScalar(source, "text_scale");
  const notifications = readYamlScalar(source, "notifications");
  const reducedMotion = readYamlScalar(source, "reduced_motion");
  const compactLists = readYamlScalar(source, "compact_desktop_lists");
  const appLockAfter = readYamlScalar(source, "lock_after_minutes");
  const defaultFee = readYamlScalar(source, "default_fee");
  const customEnabled = readYamlScalar(source, "custom_enabled");
  const customBrand = readYamlScalar(source, "custom_brand");
  const customRail = readYamlScalar(source, "custom_rail");
  const hideSensitive = readYamlScalar(source, "hide_sensitive_amounts");
  const expertDetails = readYamlScalar(source, "expert_details");

  if (theme && !["system", "dark", "light"].includes(theme)) return { valid: false, message: "Theme must be system, dark, or light." };
  if (palette && !paletteOptions.some((entry) => entry.id === palette)) return { valid: false, message: "Palette must use one of the listed preset IDs." };
  if (codeTheme && !codeThemeOptions.some((entry) => entry.id === codeTheme)) return { valid: false, message: "code_theme must use one of the listed preset IDs." };
  if (textScale && !["100", "110", "125"].includes(textScale)) return { valid: false, message: "text_scale must be 100, 110, or 125." };
  if (notifications && !["true", "false"].includes(notifications)) return { valid: false, message: "notifications must be true or false." };
  if (reducedMotion && !["true", "false"].includes(reducedMotion)) return { valid: false, message: "reduced_motion must be true or false." };
  if (compactLists && !["true", "false"].includes(compactLists)) return { valid: false, message: "compact_desktop_lists must be true or false." };
  if (defaultFee && !/^\d+(?:\.\d+)?$/.test(defaultFee)) return { valid: false, message: "default_fee must be a non-negative decimal." };
  if (customEnabled && !["true", "false"].includes(customEnabled)) return { valid: false, message: "custom_enabled must be true or false." };
  if (customBrand && !hexToRgb(customBrand)) return { valid: false, message: "custom_brand must be a six-digit hex color." };
  if (customRail && !hexToRgb(customRail)) return { valid: false, message: "custom_rail must be a six-digit hex color." };
  if (hideSensitive && !["true", "false"].includes(hideSensitive)) return { valid: false, message: "hide_sensitive_amounts must be true or false." };
  if (expertDetails && !["true", "false"].includes(expertDetails)) return { valid: false, message: "expert_details must be true or false." };
  if (appLockAfter && !["5", "15", "30", "never"].includes(appLockAfter.toLowerCase())) return { valid: false, message: "lock_after_minutes must be 5, 15, 30, or never." };

  if (apply) {
    if (theme) state.theme = theme;
    if (palette) state.palette = palette;
    if (codeTheme) state.codeTheme = codeTheme;
    if (language) state.language = language;
    if (textScale) state.textScale = textScale;
    if (notifications) state.notifications = notifications === "true";
    if (reducedMotion) state.reducedMotion = reducedMotion === "true";
    if (compactLists) state.compactLists = compactLists === "true";
    if (appLockAfter) state.autoLockMinutes = appLockAfter.toLowerCase();
    if (defaultFee) activeWalletPreferences().defaultFee = defaultFee;
    if (customEnabled) state.hasCustomAppearance = customEnabled === "true";
    if (customBrand) state.customAppearance.brand = customBrand;
    if (customRail) state.customAppearance.rail = customRail;
    if (hideSensitive) state.balanceHidden = hideSensitive === "true";
    if (expertDetails) state.expertDetails = expertDetails === "true";
    applyAppearancePreferences();
  }

  return { valid: true, message: apply ? "Local concept draft applied. Runtime YAML write/watch is still unavailable." : "YAML draft is valid for this concept schema." };
}

function paletteCard(palette) {
  const isActive = state.palette === palette.id;
  return `<button class="palette-card${isActive ? " is-active" : ""}" type="button" data-palette="${palette.id}" aria-pressed="${isActive}">
    <span class="palette-swatches" aria-hidden="true">${palette.swatches.map((swatch) => `<i style="--swatch:${swatch}"></i>`).join("")}</span>
    <span><strong>${palette.label}</strong><small>${palette.description}</small></span>
  </button>`;
}

function codeThemeCard(theme) {
  const isActive = state.codeTheme === theme.id;
  const vars = `--code-preview-bg:${theme.background};--code-preview-border:${theme.border};--code-preview-fg:${theme.foreground};--code-preview-comment:${theme.comment};--code-preview-keyword:${theme.keyword};--code-preview-string:${theme.string};--code-preview-number:${theme.number};--code-preview-title:${theme.title}`;
  return `<button class="code-theme-card${isActive ? " is-active" : ""}" type="button" data-code-theme="${theme.id}" aria-pressed="${isActive}">
    <span class="code-theme-card-heading"><strong>${theme.label}</strong>${isActive ? "<em>Active</em>" : ""}</span>
    <span class="code-theme-preview" aria-hidden="true" style="${vars}">
      <span class="code-theme-preview-dots"><i></i><i></i><i></i><i></i></span>
      <span><b>// z00z preview</b></span>
      <span><strong>theme</strong><span> = </span><em>"demo"</em></span>
      <span><strong>epoch</strong><span> = </span><u>42</u></span>
    </span>
  </button>`;
}

function yamlCommentIndex(value) {
  let isQuoted = false;
  let isEscaped = false;
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index];
    if (char === '"' && !isEscaped) isQuoted = !isQuoted;
    if (char === "#" && !isQuoted && (index === 0 || /\s/.test(value[index - 1]))) return index;
    isEscaped = char === "\\" && !isEscaped;
    if (char !== "\\") isEscaped = false;
  }
  return -1;
}

function yamlHighlightValue(value) {
  const commentIndex = yamlCommentIndex(value);
  const scalar = commentIndex === -1 ? value : value.slice(0, commentIndex);
  const comment = commentIndex === -1 ? "" : value.slice(commentIndex);
  const trailing = scalar.match(/\s*$/)?.[0] || "";
  const core = scalar.slice(0, scalar.length - trailing.length);
  let rendered = escapeHtml(core);
  if (/^"(?:[^"\\]|\\.)*"$/.test(core)) rendered = `<span class="yaml-token-string">${escapeHtml(core)}</span>`;
  else if (/^(?:true|false|null|~)$/i.test(core)) rendered = `<span class="yaml-token-number">${escapeHtml(core)}</span>`;
  else if (/^-?\d+(?:\.\d+)?$/.test(core)) rendered = `<span class="yaml-token-number">${escapeHtml(core)}</span>`;
  return `${rendered}${escapeHtml(trailing)}${comment ? `<span class="yaml-token-comment">${escapeHtml(comment)}</span>` : ""}`;
}

function yamlSyntaxHighlight(source) {
  return source.split("\n").map((line) => {
    const match = line.match(/^(\s*)([A-Za-z][A-Za-z0-9_-]*)(:)(\s*)(.*)$/);
    if (!match) return line.trimStart().startsWith("#") ? `<span class="yaml-token-comment">${escapeHtml(line)}</span>` : escapeHtml(line);
    return `${escapeHtml(match[1])}<span class="yaml-token-key">${escapeHtml(match[2])}</span><span class="yaml-token-punctuation">${match[3]}</span>${escapeHtml(match[4])}${yamlHighlightValue(match[5])}`;
  }).join("\n");
}

function yamlEditorMarkup(id, source, label, describedBy = "") {
  return `<label class="yaml-field"><span class="visually-hidden">${label}</span><span class="yaml-editor-shell"><pre class="yaml-highlight" aria-hidden="true">${yamlSyntaxHighlight(source)}</pre><textarea id="${id}" class="yaml-editor" spellcheck="false"${describedBy ? ` aria-describedby="${describedBy}"` : ""}>${escapeHtml(source)}</textarea></span></label>`;
}

function syncYamlHighlight(textarea) {
  const highlight = textarea.closest(".yaml-editor-shell")?.querySelector(".yaml-highlight");
  if (!highlight) return;
  highlight.innerHTML = yamlSyntaxHighlight(textarea.value);
  highlight.scrollTop = textarea.scrollTop;
  highlight.scrollLeft = textarea.scrollLeft;
}

function advancedConfigContent() {
  const hasYamlView = state.configView === "yaml";
  const hasFormView = state.configView === "form";
  const hasDiffView = state.configView === "diff";
  const walletPreferences = activeWalletPreferences();
  const source = state.configDraft || effectiveDemoConfigYaml();
  const formContent = `
    <div class="config-form-grid">
      <label><span>Language</span><select data-config-control="language"><option${state.language === "English" ? " selected" : ""}>English</option><option${state.language === "Русский" ? " selected" : ""}>Русский</option><option${state.language === "עברית" ? " selected" : ""}>עברית</option></select></label>
      <label><span>Palette</span><select data-config-control="palette">${paletteOptions.map((palette) => `<option value="${palette.id}"${state.palette === palette.id ? " selected" : ""}>${palette.label}</option>`).join("")}</select></label>
      <label><span>Code highlighting</span><select data-config-control="code-theme">${codeThemeOptions.map((theme) => `<option value="${theme.id}"${state.codeTheme === theme.id ? " selected" : ""}>${theme.label}</option>`).join("")}</select></label>
      <label><span>Text scale</span><select data-config-control="text-scale"><option value="100"${state.textScale === "100" ? " selected" : ""}>100%</option><option value="110"${state.textScale === "110" ? " selected" : ""}>110%</option><option value="125"${state.textScale === "125" ? " selected" : ""}>125%</option></select></label>
      <label><span>Default fee</span><input data-config-control="default-fee" inputmode="decimal" value="${escapeHtml(walletPreferences.defaultFee)}" aria-label="Default fee"></label>
    </div>`;
  const diffContent = `
    <div class="config-diff" aria-label="Visible controls and YAML mapping">
      <div><span>UI</span><strong>Appearance and wallet controls</strong></div><div>${icon("chevron")}</div><div><span>YAML</span><strong class="mono">app.* / wallet.*</strong></div>
      <p>Changes remain inside this browser concept. A future runtime integration must provide revisioned read, validate, write, and watch capabilities before local files can change.</p>
    </div>`;
  return `
    <div class="settings-heading"><div><p class="eyebrow">Advanced configuration</p><h2>YAML & diagnostics</h2><p>Visible controls and the local concept YAML describe the same safe settings. Secrets and local paths are excluded.</p></div><span class="config-source">Concept-local</span></div>
    <div class="choice-strip config-view-choices" role="tablist" aria-label="Configuration view">
      ${["yaml", "form", "diff"].map((view) => `<button class="choice-chip${state.configView === view ? " is-active" : ""}" type="button" role="tab" aria-selected="${state.configView === view}" data-config-view="${view}">${view === "yaml" ? "YAML" : view === "form" ? "Form" : "Mapping"}</button>`).join("")}
    </div>
    <div class="yaml-toolbar"><span><strong class="mono">wallet_config.yaml</strong><small>${escapeHtml(state.configStatus)}</small></span><div><button class="button" type="button" data-demo-action="config-validate">Validate</button><button class="button button-primary" type="button" data-demo-action="config-apply">Apply locally</button></div></div>
    <div role="tabpanel" class="config-panel">
      ${hasYamlView ? yamlEditorMarkup("config-yaml", source, "Concept configuration YAML", "config-capability-note") : ""}
      ${hasFormView ? formContent : ""}
      ${hasDiffView ? diffContent : ""}
    </div>
    <div class="config-foot"><span>${icon("shield")} No secrets or local paths</span><span>${icon("activity")} Local concept only</span><span>${icon("backup")} Runtime sync unavailable</span></div>
    <div class="capability-note" id="config-capability-note">${icon("alert")} <span><strong>Runtime integration boundary</strong><small>Apply locally updates this demo only. The runtime currently has no configuration write, watch, or revision RPC, so it cannot update a real wallet configuration.</small></span></div>
    <div class="setting-group"><div class="setting-line"><span class="setting-line-copy"><strong>Expert details</strong><small>Show identifiers, receipts, and lifecycle events</small></span><button class="toggle" type="button" aria-pressed="${state.expertDetails}" aria-label="Show expert details" data-demo-action="expert"></button></div><div class="setting-line"><span class="setting-line-copy"><strong>Sanitized diagnostics</strong><small>RPC, configuration, route, and synchronization events</small></span><button class="button" type="button" data-demo-action="diagnostics">Open</button></div></div>`;
}

function isWalletView() {
  return ["wallet", "activity", "swap", "exchange", "staking", "wallet-backup", "wallet-settings"].includes(state.view);
}

function hasSelectedWalletContext() {
  return Boolean(state.selectedWalletId) && !["settings", "add-wallet"].includes(state.view);
}

function addWalletProfile(name, scan = "Scanning") {
  const index = state.wallets.length + 1;
  const id = `wallet-${index}`;
  const addressTail = String(2300 + index).padStart(4, "0");
  const wallet = {
    id,
    name,
    initials: name.trim().slice(0, 1).toUpperCase(),
    address: `ZxN${index}q7…${addressTail}Pt`,
    fullAddress: `ZxN${index}q7xA1mP9vR4sT8cQ2wE6hK${addressTail}Pt`,
    summary: { available: "0.00", locked: "0.00", pendingIn: "0.00", pendingOut: "0.00", scan },
    activities: []
  };
  state.wallets.push(wallet);
  return wallet;
}

function sidebarActiveTarget() {
  if (state.view === "settings") {
    if (state.settingsSection === "network") {
      return { group: "network", id: state.networkSection };
    }
    return { group: "settings", id: "settings" };
  }

  if (state.view === "add-wallet") {
    return { group: "add-wallet", id: "add-wallet" };
  }

  return state.selectedWalletId
    ? { group: "wallet", id: state.selectedWalletId }
    : { group: null, id: null };
}

function renderWalletShell() {
  const wallet = activeWallet();
  const summary = wallet.summary;
  const sidebarTarget = sidebarActiveTarget();
  walletNav.innerHTML = `${state.wallets.map((entry) => `
    <button class="wallet-nav-item${sidebarTarget.group === "wallet" && entry.id === sidebarTarget.id ? " is-active" : ""}" type="button" ${sidebarTarget.group === "wallet" && entry.id === sidebarTarget.id ? 'aria-current="page"' : ""} data-wallet-id="${escapeHtml(entry.id)}">
      <span class="wallet-avatar" aria-hidden="true">${escapeHtml(entry.initials)}</span>
      <span class="wallet-nav-copy"><strong>${escapeHtml(entry.name)}</strong><small><span class="mono">${sensitive(`${entry.summary.available} Z00Z`)}</span> available</small></span>
      <span class="wallet-nav-state${entry.summary.scan === "Scanning" ? " is-scanning" : ""}" aria-label="${escapeHtml(entry.summary.scan)}"></span>
    </button>`).join("")}
    <div class="wallet-nav-actions" id="wallet-nav-actions">
      <button class="nav-item nav-item-primary" type="button" data-view="add-wallet">${icon("plus")}<span>Add wallet</span></button>
      <button class="nav-item nav-item-danger" type="button" data-demo-action="remove-wallet"${state.wallets.length === 0 ? " disabled" : ""}>${icon("remove")}<span>Remove wallet</span></button>
    </div>`;
  const networkEntries = [
    { key: "onionnet", label: "OnionNet", initials: "O", helper: "Target overlay" },
    { key: "reticulum", label: "Reticulum", initials: "R", helper: "Target carrier" },
    { key: "overview", label: "TBD", initials: "T", helper: "Additional route" }
  ];
  networkNav.innerHTML = networkEntries.map((entry) => {
    const isActive = sidebarTarget.group === "network" && sidebarTarget.id === entry.key;
    return `<button class="network-nav-item${isActive ? " is-active" : ""}" type="button" ${isActive ? 'aria-current="page"' : ""} data-network-section="${entry.key}" title="Open ${entry.label} network settings">
      <span class="network-avatar" aria-hidden="true">${entry.initials}</span>
      <span class="network-nav-copy"><strong>${entry.label}</strong><small>${entry.helper}</small></span>
      <span class="network-nav-state" aria-hidden="true"></span>
    </button>`;
  }).join("");
  walletIdentity.innerHTML = `<span class="wallet-avatar" aria-hidden="true">${escapeHtml(wallet.initials)}</span><span><strong>${escapeHtml(wallet.name)}</strong><small class="mono">${escapeHtml(wallet.address)}</small></span>`;
  walletIdentity.setAttribute("aria-label", `Switch wallet. Current wallet: ${wallet.name}`);
  lockWalletLabel.innerHTML = `${escapeHtml(wallet.name)} wallet <span aria-hidden="true">·</span> <span class="mono">${escapeHtml(wallet.address)}</span>`;
  copyWalletAddress.setAttribute("aria-label", `Copy full address for ${wallet.name} wallet`);
  copyWalletAddress.setAttribute("title", wallet.fullAddress);
  walletTabs.innerHTML = [
    { view: "wallet", label: "Assets", iconName: "wallet" },
    { view: "activity", label: "History", iconName: "activity" },
    { view: "swap", label: "Swap", iconName: "swap", title: "Compatibility preview — no canonical execution route" },
    { view: "exchange", label: "Exchange", iconName: "exchange", title: "Unavailable — no verified exchange provider or route", disabled: true },
    { view: "staking", label: "Staking", iconName: "staking", title: "Compatibility preview — validator and lock terms required" },
    { view: "wallet-backup", label: "Backup", iconName: "backup" },
    { view: "wallet-settings", label: "Settings", iconName: "settings" }
  ].map(({ view, label, iconName, title = "", disabled = false }) => `<button class="wallet-tab${state.view === view ? " is-active" : ""}${disabled ? " is-unavailable" : ""}" type="button" ${state.view === view ? 'aria-current="page"' : ""}${disabled ? " disabled" : ""}${title ? ` title="${escapeHtml(title)}"` : ""} data-view="${view}">${icon(iconName)}<span>${label}</span>${disabled ? '<span class="sr-only">Unavailable</span>' : ""}</button>`).join("");
  walletStatusbar.innerHTML = `
    <span><small>Available</small><strong>${sensitive(`${summary.available} Z00Z`)}</strong></span>
    <span><small>Locked</small><strong>${sensitive(`${summary.locked} Z00Z`)}</strong></span>
    <span><small>Pending in</small><strong>${sensitive(`${summary.pendingIn} Z00Z`)}</strong></span>
    <span><small>Pending out</small><strong>${sensitive(`${summary.pendingOut} Z00Z`)}</strong></span>
    <span class="statusbar-telemetry"><small>Route telemetry</small><strong><span class="statusbar-state-dot" aria-hidden="true"></span>Unavailable</strong></span>`;
  walletTabs.hidden = !isWalletView();
  walletStatusbar.hidden = !hasSelectedWalletContext();
  document.querySelector(".bottom-nav").hidden = state.view === "add-wallet";
}

function icon(name, className = "") {
  return `<svg class="icon ${className}" aria-hidden="true"><use href="#i-${name}"/></svg>`;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function sensitive(value) {
  return `<span class="sensitive${state.balanceHidden ? " is-hidden" : ""}"${state.balanceHidden ? ' aria-label="Amount hidden"' : ""}>${state.balanceHidden ? "Hidden" : escapeHtml(value)}</span>`;
}

function walletAssetEntries() {
  const wallet = activeWallet();
  return [
    {
      key: "z00z", label: "Z00Z", ticker: "Z00Z", unit: "Z00Z", kind: "Coin", balance: wallet.summary.available, balanceLabel: `${wallet.summary.available} Z00Z`, value: "—", price: "Unavailable", priceNote: "No market feed", divisible: true, logo: "Z", owner: "Protocol-native asset", assetId: "z00z:main:coin", currentSupply: "18,450,000 Z00Z", maxSupply: "21,000,000 Z00Z", note: "Native asset · trusted catalog", tone: "native"
    },
    {
      key: "acme", label: "Acme Credits", ticker: "ACME", unit: "ACME", kind: "Token", balance: "240.00", balanceLabel: "240.00 ACME", value: "—", price: "Unavailable", priceNote: "No market feed", divisible: true, logo: "A", owner: "acme.example issuer", assetId: "asset:acme:8f31…c20e", currentSupply: "2,400,000 ACME", maxSupply: "10,000,000 ACME", note: "Declared domain · review needed", tone: "token"
    },
    {
      key: "founders", label: "Founders Pass #014", ticker: "PASS-014", unit: "pass", kind: "Collectible", balance: "1", balanceLabel: "1 pass", value: "—", price: "Unavailable", priceNote: "No market feed", divisible: false, logo: "◇", owner: wallet.fullAddress || wallet.address, assetId: "nft:founders:014", currentSupply: "1 pass", maxSupply: "100 passes", note: "Unique collectible · metadata available", tone: "collectible"
    }
  ];
}

function supportedAsset(assetKey = "z00z") {
  const assets = walletAssetEntries();
  return assets.find((asset) => asset.key === assetKey) || assets[0];
}

function flowAsset(data = state.flow?.data) {
  return supportedAsset(data?.assetKey);
}

function assetOptions(selectedKey = "z00z") {
  return walletAssetEntries().map((asset) => {
    return `<option value="${asset.key}"${asset.key === selectedKey ? " selected" : ""}>${asset.label} · ${asset.kind}</option>`;
  }).join("");
}

function quickAction(type, label, helper, iconName) {
  return `
    <button class="quick-action" type="button" data-open-flow="${type}">
      <span class="quick-action-icon">${icon(iconName)}</span>
      <span><strong>${label}</strong><small>${helper}</small></span>
    </button>`;
}

function homeView() {
  const wallet = activeWallet();
  const summary = wallet.summary;
  return `
    <div class="view-enter">
      <section class="dashboard-grid" aria-label="Wallet overview">
        <article class="card balance-card">
          <div class="balance-card-top">
            <span class="balance-label">${icon("shield")} Available privately</span>
            <span class="asset-chip">Z00Z</span>
          </div>
          <p class="balance-amount">${sensitive(summary.available)} <span class="mono">Z00Z</span></p>
          <p class="balance-pending"><strong>${sensitive(`+ ${summary.pendingIn}`)}</strong> receiving · <strong>${sensitive(`− ${summary.pendingOut}`)}</strong> sending</p>
        </article>

        <article class="card privacy-card">
          <div class="privacy-card-header">
            <span class="shield-mark">${icon("shield")}</span>
            <span class="status-badge is-ready">Target simulation</span>
          </div>
          <h2>Private route model</h2>
          <p>Target layering is shown without pretending current RPC telemetry exists.</p>
          <div class="privacy-lines">
            <div class="privacy-line"><span>Privacy overlay</span><strong>OnionNet · target</strong></div>
            <div class="privacy-line"><span>Primary carrier</span><strong>Reticulum · target</strong></div>
            <div class="privacy-line"><span>Wallet scan</span><strong>${escapeHtml(summary.scan)}</strong></div>
            <div class="privacy-line"><span>Live route telemetry</span><strong>Unavailable</strong></div>
          </div>
        </article>
      </section>

      <section class="quick-section" aria-labelledby="quick-title">
        <div class="section-heading">
          <div><h2 id="quick-title">What would you like to do?</h2><p>Private actions with safe defaults</p></div>
        </div>
        <div class="quick-pairs">
          <div class="quick-pair">
            ${quickAction("pay", "Send", "Send any supported asset", "send")}
            ${quickAction("receive", "Receive", "Request any supported asset", "receive")}
          </div>
          <div class="quick-pair">
            ${quickAction("asset-claim", "Claim", "Claim an asset allocation", "claim")}
            ${quickAction("permission", "Give permission", "Delegate a bounded right", "permission")}
          </div>
        </div>
      </section>

      <section class="home-lower">
        <article class="card panel" aria-labelledby="attention-title">
          <div class="section-heading">
            <div><h2 id="attention-title">Needs your attention</h2><p>Two items</p></div>
            <button class="section-link" type="button" data-view="activity">Review history ${icon("chevron")}</button>
          </div>
          <div class="attention-list">
            <button class="attention-item" type="button" data-open-flow="voucher-review">
              <span class="list-icon is-claim">${icon("claim")}</span>
              <span class="list-copy"><strong>Travel refund voucher</strong><small>Offered by Northwind Travel · review required</small></span>
              <span class="list-meta"><strong>86.00 Z00Z</strong><small>Ends in 2 days</small></span>
            </button>
            <button class="attention-item" type="button" data-open-flow="permission-detail">
              <span class="list-icon is-warning">${icon("alert")}</span>
              <span class="list-copy"><strong>Delivery receipt access</strong><small>Data access · cannot delegate</small></span>
              <span class="list-meta"><strong>2 of 5 uses</strong><small>Ends 31 Jul</small></span>
            </button>
          </div>
        </article>

        <article class="card panel" aria-labelledby="recent-title">
          <div class="section-heading">
            <div><h2 id="recent-title">Recent history</h2><p>Latest wallet events</p></div>
            <button class="section-link" type="button" data-view="activity">View history ${icon("chevron")}</button>
          </div>
          <div class="activity-list">
            ${activityRows(wallet.activities.slice(0, 3), true)}
          </div>
        </article>
      </section>
    </div>`;
}

function moneyView() {
  const summary = activeWallet().summary;
  const assets = walletAssetEntries();
  return `
    <div class="view-enter">
      <div class="page-intro">
        <div><p class="eyebrow">Asset family</p><h2>Your assets</h2><p>Native cash, issued tokens, and collectibles stay distinguishable. Only spendable native cash enters Available.</p></div>
        <button class="button button-primary" type="button" data-open-flow="pay">${icon("send")} Send</button>
      </div>
      <section class="money-summary" aria-label="Money totals">
        <article class="card metric-card"><span>Available</span><strong>${sensitive(`${summary.available} Z00Z`)}</strong><small>Ready to use</small></article>
        <article class="card metric-card"><span>Receiving</span><strong>${sensitive(summary.pendingIn)}</strong><small>Waiting to settle</small></article>
        <article class="card metric-card"><span>Sending</span><strong>${sensitive(summary.pendingOut)}</strong><small>Waiting to settle</small></article>
      </section>
      <div class="choice-strip" aria-label="Asset filters"><button class="choice-chip is-active" type="button">All</button><button class="choice-chip" type="button">Coin</button><button class="choice-chip" type="button">Token</button><button class="choice-chip" type="button">Collectible</button><button class="choice-chip" type="button">Needs review</button></div>
      <div class="section-heading"><div><h2>Owned assets</h2><p>Class, trust, and spendability are explicit</p></div></div>
      <section class="asset-list" aria-label="Owned assets">
        <div class="asset-table-head" aria-hidden="true"><span>Name</span><span>Balance</span><span>Value</span><span>Price</span><span></span></div>
        ${assets.map((asset) => `
          <article class="card asset-row">
            <button class="asset-identity-button" type="button" data-open-flow="asset-detail" data-asset-key="${escapeHtml(asset.key)}" aria-label="View details for ${escapeHtml(asset.label)}">
              <span class="asset-logo${asset.tone === "token" ? " is-token" : asset.tone === "collectible" ? " is-collectible" : ""}" aria-hidden="true">${escapeHtml(asset.logo)}</span>
              <span class="asset-info"><strong>${escapeHtml(asset.label)} <span class="object-kind">${escapeHtml(asset.kind)}</span></strong><small class="${asset.tone === "native" ? "trust-label" : asset.tone === "token" ? "review-label" : ""}">${asset.tone === "native" ? `${icon("shield")} ` : ""}${escapeHtml(asset.note)}</small></span>
            </button>
            <div class="asset-number"><strong>${sensitive(asset.balanceLabel)}</strong><small>Balance</small></div>
            <div class="asset-number"><strong>${asset.value === "—" ? asset.value : sensitive(asset.value)}</strong><small>Value</small></div>
            <div class="asset-number"><strong>${escapeHtml(asset.price)}</strong><small>${escapeHtml(asset.priceNote)}</small></div>
            <div class="asset-actions">
              <button class="button" type="button" data-open-flow="receive" data-asset-key="${escapeHtml(asset.key)}" aria-label="Receive ${escapeHtml(asset.label)}">${icon("receive")}<span>Receive</span></button>
              <button class="button button-primary" type="button" data-open-flow="pay" data-asset-key="${escapeHtml(asset.key)}" aria-label="Send ${escapeHtml(asset.label)}">${icon("send")}<span>Send</span></button>
            </div>
          </article>`).join("")}
      </section>
      <div class="notice">${icon("shield")} Vouchers, permissions, quarantined objects, non-native tokens, collectibles, and experimental compatibility assets are excluded from Available.</div>
    </div>`;
}

const walletSections = [
  ["assets", "Assets", "Spendable and owned value"],
  ["vouchers", "Vouchers", "Conditional value"],
  ["permissions", "Permissions", "Bounded authority"]
];

function walletContextNav() {
  return `<nav class="context-nav" aria-label="Wallet sections">${walletSections.map(([key, label, helper]) => `
    <button class="context-nav-item${state.walletSection === key ? " is-active" : ""}" type="button" ${state.walletSection === key ? 'aria-current="page"' : ""} data-wallet-section="${key}">
      <span><strong>${label}</strong><small>${helper}</small></span>${key === "vouchers" ? '<span class="nav-count">1</span>' : ""}
    </button>`).join("")}</nav>`;
}

function vouchersPanel() {
  return `
    <div class="page-intro compact-intro">
      <div><p class="eyebrow">Voucher family</p><h2>Vouchers</h2><p>Conditional value has its own acceptance, redemption, transfer, refund, and expiry lifecycle.</p></div>
    </div>
    <div class="choice-strip" aria-label="Voucher filters"><button class="choice-chip is-active" type="button">Needs action</button><button class="choice-chip" type="button">Redeemable</button><button class="choice-chip" type="button">History</button><button class="choice-chip" type="button">Quarantined</button></div>
    <section class="card action-panel">
      <div class="action-panel-top"><div class="action-title"><span class="list-icon is-claim">${icon("claim")}</span><div><h2>Ready for your decision</h2><p>Backing and restrictions are checked before any action</p></div></div><span class="status-badge is-ready">1 ready</span></div>
      <div class="claim-list">
        <button class="claim-row" type="button" data-open-flow="voucher-review"><span class="list-icon is-claim">${icon("claim")}</span><span class="list-copy"><strong>Travel refund voucher</strong><small>Northwind Travel · consumed-asset backing · acceptance required · refund allowed</small></span><span class="list-meta"><strong>86.00 Z00Z</strong><small class="status-badge is-ready">Offered</small></span></button>
        <button class="claim-row" type="button" data-open-flow="voucher-settled"><span class="list-icon">${icon("check")}</span><span class="list-copy"><strong>Event deposit return</strong><small>Riverside Events · redeemed and settled 12 Jul</small></span><span class="list-meta"><strong>150.00 Z00Z</strong><small class="status-badge is-settled">Redeemed</small></span></button>
      </div>
    </section>
    <div class="notice">${icon("shield")} Imported vouchers with unknown policy, invalid signatures, or unsupported schema go to Quarantine and never enter Available.</div>`;
}

function permissionsPanel() {
  return `
    <div class="page-intro compact-intro">
      <div><p class="eyebrow">Right family</p><h2>Permissions</h2><p>Zero-value authority with explicit action, scope, uses, expiry, and delegation rules.</p></div>
      <button class="button button-primary" type="button" data-open-flow="permission">${icon("permission")} Give permission</button>
    </div>
    <div class="choice-strip" aria-label="Permission filters"><button class="choice-chip is-active" type="button">Held</button><button class="choice-chip" type="button">Delegated</button><button class="choice-chip" type="button">Used</button><button class="choice-chip" type="button">Needs review</button></div>
    <section class="card action-panel">
      <div class="action-panel-top"><div class="action-title"><span class="list-icon is-warning">${icon("permission")}</span><div><h2>Held permissions</h2><p>Class, action, scope, uses, expiry, and delegation are visible</p></div></div><span class="status-badge is-active">2 held</span></div>
      <div class="permission-list">
        <button class="permission-row" type="button" data-open-flow="permission-detail"><span class="list-icon is-warning">${icon("permission")}</span><span class="list-copy"><strong>Delivery receipt access</strong><small>Data access · view · receipts.example · cannot delegate</small></span><span class="list-meta"><strong>2 of 5 uses</strong><small class="status-badge is-active">Held</small></span></button>
        <div class="permission-row"><span class="list-icon">${icon("shield")}</span><span class="list-copy"><strong>Deploy to staging</strong><small>Machine capability · deploy · staging.example · attenuation only</small></span><span class="list-meta"><strong>1 use</strong><small class="status-badge is-active">Held</small></span></div>
      </div>
    </section>
    <div class="notice">${icon("spark")} A permission is zero-value. “Give permission” delegates a narrower held right; monetary budgets require a separate future composition and are not projected here.</div>`;
}

function walletView() {
  const panel = state.walletSection === "assets" ? moneyView() : state.walletSection === "vouchers" ? vouchersPanel() : permissionsPanel();
  return `<div class="view-enter workspace-layout"><aside class="context-rail"><p class="context-rail-label">Wallet</p>${walletContextNav()}</aside><div class="workspace-panel">${panel}</div></div>`;
}

function statusText(status) {
  return {
    settling: "Settling",
    settled: "Settled",
    active: "Active",
    attention: "Needs attention"
  }[status] || "Ready";
}

function activityRows(items, compact = false) {
  if (!items.length) {
    return `<div class="empty-state"><span class="list-icon">${icon("search")}</span><h3>No matching history</h3><p>Try another filter or search term.</p></div>`;
  }

  return items.map((item) => {
    const iconName = item.type === "voucher" || item.id.startsWith("claim-") ? "claim" : item.type === "permission" ? "permission" : item.type === "security" ? "backup" : item.direction === "in" ? "receive" : "send";
    const iconClass = item.direction === "in" ? "is-incoming" : item.direction === "out" ? "is-outgoing" : "";
    const amountClass = item.direction === "in" ? "positive" : item.direction === "out" ? "negative" : "";
    return `
      <button class="activity-row" type="button" data-open-activity="${escapeHtml(item.id)}">
        <span class="activity-icon ${iconClass}">${icon(iconName)}</span>
        <span class="activity-copy"><strong>${escapeHtml(item.title)}</strong><small>${escapeHtml(item.detail)}${compact ? ` · ${escapeHtml(item.time)}` : ` · <span class="status-badge is-${escapeHtml(item.status)}">${statusText(item.status)}</span>`}</small></span>
        <span class="activity-value"><strong class="${amountClass}">${escapeHtml(item.amount)}</strong><small>${escapeHtml(item.time)}</small></span>
      </button>`;
  }).join("");
}

function matchesActivityFilter(item, filter) {
  if (filter === "all") return true;
  if (filter === "asset") return item.type === "asset" || item.type === "money";
  if (filter === "attention") return item.status === "attention" || item.status === "settling";
  return item.type === filter;
}

function activityView() {
  const visible = activeWallet().activities.filter((item) => matchesActivityFilter(item, state.activityFilter));

  const filters = [
    ["all", "All"], ["asset", "Assets"], ["voucher", "Vouchers"], ["permission", "Permissions"], ["security", "System"], ["attention", "Needs attention"]
  ].map(([value, label]) => `<button class="choice-chip${state.activityFilter === value ? " is-active" : ""}" type="button" data-filter="${value}">${label}</button>`).join("");

  return `
    <div class="view-enter">
      <div class="page-intro"><div><p class="eyebrow">Honest settlement</p><h2>Everything that changed</h2><p>Submission and final settlement are shown as different states. Open an item for its receipt and technical timeline.</p></div></div>
      <div class="filter-bar choice-strip" aria-label="History filters">
        ${filters}
        <label class="search-wrap"><span class="sr-only">Search history</span>${icon("search")}<input id="activity-search" type="search" placeholder="Search history" autocomplete="off"></label>
      </div>
      <section class="card activity-panel" id="activity-results" aria-label="History results">
        ${activityRows(visible)}
      </section>
    </div>`;
}

function swapView() {
  const wallet = activeWallet();
  const asset = supportedAsset("z00z");
  return `
    <div class="view-enter wallet-tool-view">
      <div class="page-intro"><div><p class="eyebrow">Wallet swap</p><h2>Swap assets privately</h2><p>Move between compatible assets within ${escapeHtml(wallet.name)}. The preview labels route availability honestly.</p></div><span class="status-badge is-ready">${escapeHtml(wallet.name)} wallet</span></div>
      <section class="wallet-tool-grid">
        <article class="card wallet-tool-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("swap")}</span><div><h3>Build a swap</h3><p>Choose the assets before you request a quote.</p></div></div>
          <div class="form-grid">
            <div class="field-group"><label class="field-label" for="swap-from">From</label><select id="swap-from">${assetOptions("z00z")}</select><p class="field-hint">Available: ${sensitive(`${asset.balance} ${asset.unit}`)}</p></div>
            <div class="field-group"><label class="field-label" for="swap-amount">Amount</label><div class="input-with-affix"><input id="swap-amount" type="number" min="0.01" max="${escapeHtml(asset.balance.replaceAll(",", ""))}" step="0.01" inputmode="decimal" placeholder="0.00"><span class="input-affix">Z00Z</span></div></div>
            <div class="field-group"><label class="field-label" for="swap-to">To</label><select id="swap-to">${assetOptions("acme")}</select></div>
            <button class="button button-primary" type="button" data-demo-action="preview-swap">${icon("swap")} Preview swap</button>
          </div>
        </article>
        <aside class="card wallet-tool-card wallet-tool-summary">
          <p class="eyebrow">Route status</p>
          <div class="summary-row"><span>Wallet</span><strong>${escapeHtml(wallet.name)}</strong></div>
          <div class="summary-row"><span>Privacy route</span><strong>Target preview</strong></div>
          <div class="summary-row"><span>Execution</span><strong>Not submitted</strong></div>
          <div class="notice">${icon("shield")} A swap cannot use vouchers, permissions, quarantined items, or unsupported assets.</div>
        </aside>
      </section>
    </div>`;
}

function exchangeView() {
  const wallet = activeWallet();
  const asset = supportedAsset("z00z");
  return `
    <div class="view-enter wallet-tool-view">
      <div class="page-intro"><div><p class="eyebrow">External exchange</p><h2>Compare exchange routes</h2><p>Exchange remains separate from an in-wallet swap, so provider, rate, and settlement responsibility stay visible.</p></div><span class="status-badge">No route selected</span></div>
      <section class="wallet-tool-grid">
        <article class="card wallet-tool-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("exchange")}</span><div><h3>Request a quote</h3><p>Only assets held by ${escapeHtml(wallet.name)} can be prepared.</p></div></div>
          <div class="form-grid">
            <div class="field-group"><label class="field-label" for="exchange-asset">Asset to exchange</label><select id="exchange-asset">${assetOptions("z00z")}</select><p class="field-hint">${sensitive(`${asset.balance} ${asset.unit}`)} available in this wallet.</p></div>
            <div class="field-group"><label class="field-label" for="exchange-destination">Receive</label><select id="exchange-destination"><option>USDC · Token</option><option>EURC · Token</option><option>BTC · Coin</option></select></div>
            <button class="button button-primary" type="button" data-demo-action="request-exchange-quote">${icon("exchange")} Request quote</button>
          </div>
        </article>
        <aside class="card wallet-tool-card wallet-tool-summary">
          <p class="eyebrow">Before exchange</p>
          <div class="summary-row"><span>Provider</span><strong>Not selected</strong></div>
          <div class="summary-row"><span>Rate</span><strong>Unavailable</strong></div>
          <div class="summary-row"><span>Settlement</span><strong>Not started</strong></div>
          <div class="capability-note">${icon("alert")} <span><strong>Concept-only route</strong><small>Production requires a verified provider and an authoritative quote before enabling exchange.</small></span></div>
        </aside>
      </section>
    </div>`;
}

function stakingView() {
  const wallet = activeWallet();
  const summary = wallet.summary;
  return `
    <div class="view-enter wallet-tool-view">
      <div class="page-intro"><div><p class="eyebrow">Wallet staking</p><h2>Stake from ${escapeHtml(wallet.name)}</h2><p>Staking terms and pending value remain distinct from the wallet's spendable Available balance.</p></div><span class="status-badge is-ready">Main · concept</span></div>
      <section class="money-summary" aria-label="Staking totals">
        <article class="card metric-card"><span>Available to stake</span><strong>${sensitive(`${summary.available} Z00Z`)}</strong><small>Wallet value before a stake is submitted</small></article>
        <article class="card metric-card"><span>Staked</span><strong>0.00 Z00Z</strong><small>Nothing delegated in this concept</small></article>
        <article class="card metric-card"><span>Rewards</span><strong>0.00 Z00Z</strong><small>Accrual is not simulated</small></article>
      </section>
      <section class="wallet-tool-grid">
        <article class="card wallet-tool-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("staking")}</span><div><h3>Prepare a stake</h3><p>Choose the amount and inspect the validator terms before authorization.</p></div></div>
          <div class="form-grid">
            <div class="field-group"><label class="field-label" for="stake-amount">Amount</label><div class="input-with-affix"><input id="stake-amount" type="number" min="0.01" max="${escapeHtml(summary.available.replaceAll(",", ""))}" step="0.01" inputmode="decimal" placeholder="0.00"><span class="input-affix">Z00Z</span></div><p class="field-hint">Available: ${sensitive(`${summary.available} Z00Z`)}</p></div>
            <div class="field-group"><label class="field-label" for="stake-validator">Validator</label><select id="stake-validator"><option>Choose after chain verification</option></select></div>
            <button class="button button-primary" type="button" data-demo-action="prepare-stake">${icon("staking")} Review stake</button>
          </div>
        </article>
        <aside class="card wallet-tool-card wallet-tool-summary">
          <p class="eyebrow">Staking safeguards</p>
          <div class="summary-row"><span>Validator status</span><strong>Unavailable</strong></div>
          <div class="summary-row"><span>Unlock period</span><strong>Not selected</strong></div>
          <div class="summary-row"><span>Rewards</span><strong>Not projected</strong></div>
          <div class="notice">${icon("shield")} The concept never estimates yield or hides lock-up risk. A stake remains pending until chain settlement.</div>
        </aside>
      </section>
    </div>`;
}

function walletBackupView() {
  const wallet = activeWallet();
  return `
    <div class="view-enter wallet-tool-view">
      <div class="page-intro"><div><p class="eyebrow">Selected wallet backup</p><h2>Back up ${escapeHtml(wallet.name)}</h2><p>Create and verify a recoverable local backup for this wallet profile without changing any other wallet.</p></div><span class="status-badge is-ready">Local only</span></div>
      <section class="wallet-tool-grid">
        <article class="card wallet-tool-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("backup")}</span><div><h3>Backup status</h3><p>Backup material stays distinct from the live wallet and recovery phrase.</p></div></div>
          <div class="review-card"><div class="summary-row"><span>Latest backup</span><strong>10 Jul 2026 · 09:42</strong></div><div class="summary-row"><span>Integrity</span><strong class="trust-label">${icon("shield")} Verified</strong></div><div class="summary-row"><span>Destination</span><strong>Encrypted local file</strong></div></div>
          <button class="button button-primary button-full" type="button" data-demo-action="backup">${icon("backup")} Create fresh backup</button>
        </article>
        <aside class="card wallet-tool-card wallet-tool-summary">
          <p class="eyebrow">Recovery guardrails</p>
          <div class="summary-row"><span>Wallet</span><strong>${escapeHtml(wallet.name)}</strong></div>
          <div class="summary-row"><span>Address</span><strong class="mono">${escapeHtml(wallet.address)}</strong></div>
          <div class="summary-row"><span>Restore test</span><strong>Not run today</strong></div>
          <div class="notice">${icon("shield")} Restoring validates integrity before any live wallet data is replaced.</div>
        </aside>
      </section>
    </div>`;
}

const walletSettingsMeta = {
  general: ["General", "Name, display currency, and transaction defaults"],
  security: ["Security", "Locking, recovery phrase, and key material"],
  backup: ["Backup", "Recovery scope and encrypted local copies"],
  policies: ["Policies", "Local spend rules and profile preview"],
  advanced: ["Advanced", "Safe YAML view of this wallet's visible controls"]
};

function walletSettingsContextNav() {
  const item = (key) => {
    const [label, helper] = walletSettingsMeta[key];
    const active = state.walletSettingsSection === key;
    return `<button class="context-nav-item${active ? " is-active" : ""}" type="button" ${active ? 'aria-current="page"' : ""} data-wallet-settings-section="${key}"><span><strong>${label}</strong><small>${helper}</small></span></button>`;
  };
  return `<nav class="context-nav wallet-settings-context" aria-label="Selected wallet settings">
    <p class="context-group-label">Wallet</p>${item("general")}${item("security")}${item("backup")}
    <p class="context-group-label">Rules</p>${item("policies")}
    <p class="context-group-label">Expert</p>${item("advanced")}
  </nav>`;
}

function walletSettingsYaml() {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  return [
    "schema_version: 1",
    "wallet:",
    `  id: \"${yamlScalar(wallet.id)}\"`,
    "  display:",
    `    name: \"${yamlScalar(wallet.name)}\"`,
    `    currency: ${preferences.currency}`,
    "  transactions:",
    `    default_fee: \"${yamlScalar(preferences.defaultFee)}\"`,
    "  security:",
    `    lock_after_minutes: ${preferences.lockAfterMinutes}`,
    "  backup:",
    `    auto_backup: ${preferences.autoBackup}`,
    `    interval_hours: ${preferences.backupIntervalHours}`,
    "    encrypt: true",
    "  policy_rules:",
    `    max_transaction: \"${yamlScalar(preferences.policyRules.maxTransaction)}\"`,
    `    max_daily: \"${yamlScalar(preferences.policyRules.maxDaily)}\"`,
    `    require_confirmation: ${preferences.policyRules.requireConfirmation}`,
    `    allowed_assets: ${preferences.policyRules.allowedAssets}`,
    `    allowed_recipients: \"${yamlScalar(preferences.policyRules.allowedRecipients || "any")}\"`,
    `    time_restrictions: ${preferences.policyRules.timeWindow}`,
    "  compliance_profile:",
    `    preview: \"${yamlScalar(preferences.policyProfile)}\"`,
    "# Secrets, paths, session tokens, and receiver material are excluded."
  ].join("\n");
}

function walletSettingsGeneralDetail() {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  return `
    <div class="settings-heading"><div><p class="eyebrow">Selected wallet</p><h2>Wallet details</h2><p>Name and transaction defaults apply only to ${escapeHtml(wallet.name)}.</p></div><span class="config-source">Local profile</span></div>
    <div class="setting-group">
      <div class="setting-line"><span class="setting-line-copy"><strong>Wallet name</strong><small>${escapeHtml(wallet.name)} · local display label</small></span><button class="button" type="button" data-open-flow="wallet-rename">Rename wallet</button></div>
      <div class="setting-line"><span class="setting-line-copy"><strong>Wallet ID</strong><small class="mono">${escapeHtml(wallet.id)}</small></span><span class="status-badge">Read-only</span></div>
      <div class="setting-line"><label class="setting-line-copy" for="wallet-currency"><strong>Display currency</strong><small>Presentation only; it never changes asset units.</small></label><select id="wallet-currency" data-wallet-settings-control="currency"><option${preferences.currency === "Z00Z" ? " selected" : ""}>Z00Z</option><option${preferences.currency === "USD" ? " selected" : ""}>USD</option><option${preferences.currency === "EUR" ? " selected" : ""}>EUR</option></select></div>
      <div class="setting-line"><label class="setting-line-copy" for="wallet-default-fee"><strong>Default fee</strong><small>Used as a local draft default; final fee remains visible before authorization.</small></label><input id="wallet-default-fee" data-wallet-settings-control="default-fee" inputmode="decimal" value="${escapeHtml(preferences.defaultFee)}" aria-label="Default fee"></div>
    </div>
    <div class="capability-note">${icon("alert")} <span><strong>Runtime boundary</strong><small>${icon("shield")} WalletService stores these fields, but a public wallet-settings write route is not registered yet. This demo keeps the change local and does not imply a real wallet write.</small></span></div>`;
}

function walletSettingsSecurityDetail() {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  return `
    <div class="settings-heading"><div><p class="eyebrow">Private authority</p><h2>Security</h2><p>Use a fresh password check before a sensitive operation. Private keys are never displayed.</p></div><span class="status-badge is-active">Unlocked</span></div>
    <div class="setting-group">
      <div class="setting-line"><label class="setting-line-copy" for="wallet-lock-after"><strong>Lock app after</strong><small>Per-wallet inactivity preference for this local profile.</small></label><select id="wallet-lock-after" data-wallet-settings-control="lock-after"><option value="5"${preferences.lockAfterMinutes === "5" ? " selected" : ""}>5 minutes</option><option value="15"${preferences.lockAfterMinutes === "15" ? " selected" : ""}>15 minutes</option><option value="30"${preferences.lockAfterMinutes === "30" ? " selected" : ""}>30 minutes</option><option value="never"${preferences.lockAfterMinutes === "never" ? " selected" : ""}>Never</option></select></div>
      <div class="setting-line"><span class="setting-line-copy"><strong>Lock now</strong><small>Clears sensitive presentation and closes the wallet session.</small></span><button class="button" type="button" data-demo-action="lock">Lock now</button></div>
    </div>
    <div class="setting-group wallet-key-settings">
      <div class="setting-line"><span class="setting-line-copy"><strong>Recovery phrase</strong><small>Requires the wallet password and the exact confirmation phrase. The renderer clears it when the dialog closes.</small></span><button class="button" type="button" data-open-flow="wallet-seed-reveal">View phrase</button></div>
      <div class="setting-line"><span class="setting-line-copy"><strong>Public keys</strong><small>Prepares encrypted public material after a password check; private keys are never shown.</small></span><button class="button" type="button" data-open-flow="wallet-public-export">View public keys</button></div>
      <div class="setting-line"><span class="setting-line-copy"><strong>Master key</strong><small>Last rotation: ${escapeHtml(preferences.lastMasterKeyRotation)}. Rotation rewrites protected wallet records.</small></span><button class="button button-primary" type="button" data-open-flow="wallet-key-rotation">Rotate master key</button></div>
    </div>
    <div class="confirmation-note">${icon("alert")} Seed reveal and master-key rotation are critical operations: they require password plus an explicit typed confirmation and are rate-limited in the wallet service.</div>`;
}

function walletSettingsBackupDetail() {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  return `
    <div class="settings-heading"><div><p class="eyebrow">Recovery state</p><h2>Backup</h2><p>A full backup can preserve more than a recovery phrase: local labels, scan context, receiver history, and encrypted wallet state.</p></div><span class="status-badge is-ready">Local only</span></div>
    <div class="review-card wallet-settings-summary"><div class="summary-row"><span>Latest backup</span><strong>10 Jul 2026 · 09:42</strong></div><div class="summary-row"><span>Integrity</span><strong class="trust-label">${icon("shield")} Verified</strong></div><div class="summary-row"><span>Encryption</span><strong>Enabled</strong></div><div class="summary-row"><span>Wallet</span><strong>${escapeHtml(wallet.name)}</strong></div></div>
    <div class="setting-group"><div class="setting-line"><span class="setting-line-copy"><strong>Automatic backup</strong><small>Create encrypted local recovery points for this wallet profile.</small></span><button class="toggle" type="button" aria-pressed="${preferences.autoBackup}" aria-label="Automatic wallet backup" data-demo-action="wallet-auto-backup"></button></div><div class="setting-line"><label class="setting-line-copy" for="wallet-backup-interval"><strong>Backup interval</strong><small>Cadence is stored with the selected wallet; its platform location is never exposed as YAML.</small></label><select id="wallet-backup-interval" data-wallet-settings-control="backup-interval"><option value="6"${preferences.backupIntervalHours === "6" ? " selected" : ""}>Every 6 hours</option><option value="24"${preferences.backupIntervalHours === "24" ? " selected" : ""}>Every 24 hours</option><option value="72"${preferences.backupIntervalHours === "72" ? " selected" : ""}>Every 3 days</option></select></div><div class="setting-line"><span class="setting-line-copy"><strong>Create fresh backup</strong><small>Choose a platform destination; no path is exposed in the demo.</small></span><button class="button button-primary" type="button" data-demo-action="backup">Create backup</button></div><div class="setting-line"><span class="setting-line-copy"><strong>Restore backup</strong><small>Always validates integrity before any local state is replaced.</small></span><button class="button" type="button" data-demo-action="restore">Restore backup</button></div></div>
    <div class="notice">${icon("shield")} A seed-only recovery may not restore labels, history, receiver context, or scoped disclosure artifacts. The network cannot reconstruct private state it never received.</div>`;
}

function walletSettingsPoliciesDetail() {
  const preferences = activeWalletPreferences();
  const rules = preferences.policyRules;
  return `
    <div class="settings-heading"><div><p class="eyebrow">Bounded authority</p><h2>Policies</h2><p>Wallet rules may narrow spend behavior. They cannot expand protocol authority or prove legal compliance.</p></div><button class="button" type="button" data-open-flow="wallet-policy-profile">Profile preview</button></div>
    <div class="setting-group"><div class="setting-line"><span class="setting-line-copy"><strong>Profile preview</strong><small>${escapeHtml(preferences.policyProfile)} · user-configured jurisdiction profile, not a compliance certificate.</small></span><span class="status-badge is-ready">Target</span></div><div class="setting-line"><span class="setting-line-copy"><strong>Local spend rules</strong><small>Maximum spend, daily limit, and confirmation gate map to current <code>PolicyRules</code> fields.</small></span><button class="button button-primary" type="button" data-open-flow="wallet-policy-apply">Review rules</button></div></div>
    <div class="policy-stack" aria-label="Effective wallet spend rules"><div class="policy-layer is-locked"><span>1</span><div><strong>Protocol rules</strong><small>Immutable; never editable in the wallet.</small></div><span class="status-badge">Locked</span></div><div class="policy-layer is-active"><span>2</span><div><strong>Local policy rules</strong><small>Max ${escapeHtml(rules.maxTransaction)} Z00Z · daily ${escapeHtml(rules.maxDaily)} Z00Z · ${rules.allowedAssets === "all" ? "all assets" : "native asset only"} · ${rules.allowedRecipients ? "recipient allowlist" : "all recipients"} · ${rules.timeWindow === "any" ? "any time" : "business hours UTC"} · ${rules.requireConfirmation ? "confirmation required" : "no confirmation gate"}</small></div><span class="status-badge is-active">Local</span></div><div class="policy-layer"><span>3</span><div><strong>Compliance profile</strong><small>Signed profile load/apply is unavailable in the current RPC; preview only.</small></div><span class="status-badge is-ready">Target</span></div></div>
    <div class="capability-note">${icon("alert")} <span><strong>Honest profile boundary</strong><small>Current code validates local spend rules, but has no signed compliance-profile loader, signature verifier, apply, disable, or persistence route. This page never reports “compliant”.</small></span></div>`;
}

function walletSettingsAdvancedDetail() {
  const source = state.walletSettingsConfigDraft || walletSettingsYaml();
  return `
    <div class="settings-heading"><div><p class="eyebrow">Wallet configuration</p><h2>Advanced</h2><p>Safe visible controls and this YAML are two views of the selected wallet configuration.</p></div><span class="config-source">Concept-local</span></div>
    <div class="yaml-toolbar"><span><strong class="mono">wallet_settings.yaml</strong><small>Secrets, local paths, session tokens, and receiver material are excluded.</small></span><div><button class="button" type="button" data-demo-action="wallet-config-validate">Validate</button><button class="button button-primary" type="button" data-demo-action="wallet-config-apply">Apply locally</button></div></div>
    ${yamlEditorMarkup("wallet-settings-yaml", source, "Selected wallet settings YAML")}
    <div class="config-foot"><span>${icon("shield")} No secrets or paths</span><span>${icon("activity")} Selected wallet only</span><span>${icon("settings")} ${escapeHtml(state.configStatus)}</span></div>
    <div class="capability-note">${icon("alert")} <span><strong>Local concept only</strong><small>Validation keeps UI and YAML aligned in this demo. A production bridge needs revisioned wallet-settings read/write capabilities before it can change durable wallet state.</small></span></div>`;
}

function walletSettingsDetail() {
  if (state.walletSettingsSection === "security") return walletSettingsSecurityDetail();
  if (state.walletSettingsSection === "backup") return walletSettingsBackupDetail();
  if (state.walletSettingsSection === "policies") return walletSettingsPoliciesDetail();
  if (state.walletSettingsSection === "advanced") return walletSettingsAdvancedDetail();
  return walletSettingsGeneralDetail();
}

function walletSettingsView() {
  return `<div class="view-enter settings-view wallet-settings-view"><div class="workspace-layout settings-layout"><aside class="context-rail">${walletSettingsContextNav()}</aside><article class="card settings-detail">${walletSettingsDetail()}</article></div></div>`;
}

const settingsMeta = {
  general: ["General", "Common wallet behavior", "settings"],
  security: ["Security", "Lock, sensitive information, and wallet keys", "shield"],
  network: ["Network & privacy", "Private route, chain, and synchronization", "network"],
  policies: ["Policies", "Safety and managed restrictions", "shield"],
  backup: ["Backups", "Create and verify recoverable local backups", "backup"],
  appearance: ["Appearance", "Theme, density, and motion", "sun"],
  advanced: ["Advanced", "YAML configuration and diagnostic tools", "settings"]
};

function settingsContextNav() {
  const item = (key) => {
    const [label, helper, iconName] = settingsMeta[key];
    const isNetworkBranch = key === "network";
    const isCurrent = state.settingsSection === key && (!isNetworkBranch || state.networkSection === "overview");
    const disclosure = isNetworkBranch
      ? `<span class="context-disclosure${state.isNetworkOpen ? " is-open" : ""}" aria-hidden="true">${icon("chevron")}</span>`
      : "";
    const expanded = isNetworkBranch ? ` aria-expanded="${state.isNetworkOpen}" aria-controls="network-sections"` : "";
    return `<button class="context-nav-item${isCurrent ? " is-active" : ""}${isNetworkBranch && state.isNetworkOpen ? " is-open" : ""}" type="button" ${isCurrent ? 'aria-current="page"' : ""}${expanded} title="${helper}" data-settings-section="${key}">${icon(iconName)}<span><strong>${label}</strong><small>${helper}</small></span>${disclosure}</button>`;
  };
  const networkChildren = [["overview", "Overview"], ["reticulum", "Reticulum"], ["onionnet", "OnionNet"], ["carriers", "Carriers"]]
    .map(([key, label]) => `<button class="context-nav-child${state.settingsSection === "network" && state.networkSection === key ? " is-active" : ""}" type="button" ${state.settingsSection === "network" && state.networkSection === key ? 'aria-current="page"' : ""} data-network-section="${key}">${label}</button>`).join("");
  return `<nav class="context-nav settings-context" aria-label="Settings sections">
    <p class="context-group-label">Wallet</p>${item("general")}${item("security")}${item("backup")}
    <p class="context-group-label">Connectivity</p>${item("network")}${state.isNetworkOpen ? `<div id="network-sections" class="context-nav-children" aria-label="Network sections">${networkChildren}</div>` : ""}
    <p class="context-group-label">Rules & interface</p>${item("policies")}${item("appearance")}${item("advanced")}
  </nav>`;
}

function networkDetail() {
  if (state.networkSection === "reticulum") return `
    <div class="connection-options">
      <div class="connection-option"><span class="health-orb"></span><span><strong>Reticulum service</strong><small>Target service example · no live wallet API</small></span><span class="status-badge is-ready">Target</span></div>
      <div class="connection-option"><span class="list-icon">${icon("network")}</span><span><strong>Interfaces</strong><small>Auto · TCP client + local mesh discovery</small></span><button class="button" type="button" data-demo-action="config-stage">Configure</button></div>
      <div class="connection-option"><span class="list-icon">${icon("shield")}</span><span><strong>Network identity</strong><small class="mono">RNS 6A3E…91B2 · independent from wallet seed</small></span><span class="status-badge is-active">Separate</span></div>
    </div><div class="notice">${icon("settings")} Raw Reticulum interface definitions live in Advanced YAML. Service/runtime changes may require restart.</div>`;

  if (state.networkSection === "onionnet") return `
    <div class="connection-options">
      <div class="connection-option"><span class="health-orb"></span><span><strong>Privacy route</strong><small>Target example · 3 hops · epoch 1842</small></span><span class="status-badge is-ready">Target floor</span></div>
      <div class="connection-option"><span class="list-icon">${icon("shield")}</span><span><strong>Membership & replay checks</strong><small>Target telemetry · unavailable in current RPC</small></span><span class="status-badge is-ready">Target</span></div>
      <div class="connection-option"><span class="list-icon">${icon("activity")}</span><span><strong>Route age</strong><small>12 minutes · rebuilt automatically by policy</small></span><button class="button" type="button" data-demo-action="rebuild-route">Rebuild</button></div>
    </div><div class="capability-note">${icon("alert")} <span><strong>Target Phase 080 simulation</strong><small>The current live network RPC is stubbed; all route details on this screen are illustrative until an authoritative status capability exists.</small></span></div><div class="notice">${icon("shield")} This reports concrete route properties. It does not claim that the user is “anonymous” or “untraceable.”</div>`;

  if (state.networkSection === "carriers") return `
    <div class="confirmation-note">${icon("alert")} Carrier priority affects availability. Private mode never falls back to a non-OnionNet direct path.</div>
    <div class="connection-options">
      <div class="connection-option"><span class="health-orb is-good"></span><span><strong>1 · Reticulum</strong><small>Primary resilient carrier · in use</small></span><span class="status-badge is-settled">Allowed</span></div>
      <div class="connection-option"><span class="health-orb"></span><span><strong>2 · QUIC/TLS</strong><small>Private carrier fallback</small></span><span class="status-badge is-active">Allowed</span></div>
      <div class="connection-option"><span class="health-orb"></span><span><strong>Tor compatibility</strong><small>Optional carrier · disabled in this profile</small></span><span class="status-badge">Off</span></div>
    </div>`;

  return `
    <div class="network-summary-grid">
      <article><span>Mode</span><strong>Private</strong><small>No direct fallback</small></article>
      <article><span>Privacy overlay</span><strong>OnionNet</strong><small>Verified · 3 hops</small></article>
      <article><span>Active carrier</span><strong>Reticulum</strong><small>Direct underlay</small></article>
      <article><span>Chain & scan</span><strong>Main · current</strong><small>Checked just now</small></article>
    </div>
    <div class="capability-note">${icon("alert")} <span><strong>Target Phase 080 simulation</strong><small>The current network RPC is stubbed. Production must show “capability unavailable” until these properties are authoritative.</small></span></div>`;
}

function settingsDetail() {
  if (state.settingsSection === "general") {
    return `
      <h2>General</h2>
      <div class="setting-group settings-first-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Language</strong><small>Language used throughout this wallet application</small></span><select aria-label="Language" data-config-control="language"><option${state.language === "English" ? " selected" : ""}>English</option><option${state.language === "Русский" ? " selected" : ""}>Русский</option><option${state.language === "עברית" ? " selected" : ""}>עברית</option></select></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Notifications</strong><small>Show wallet updates and required actions</small></span><button class="toggle" type="button" data-demo-action="general-notifications" aria-pressed="${state.notifications}" aria-label="Notifications ${state.notifications ? "on" : "off"}"></button></div>
      </div>`;
  }

  if (state.settingsSection === "security") {
    return `
      <h2>Security</h2><p>Keep private material out of sight and end sessions automatically.</p>
      <div class="setting-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Lock app after</strong><small>Automatically lock the wallet after inactivity</small></span><select aria-label="Lock app after" data-config-control="lock-after"><option value="5"${state.autoLockMinutes === "5" ? " selected" : ""}>5 minutes</option><option value="15"${state.autoLockMinutes === "15" ? " selected" : ""}>15 minutes</option><option value="30"${state.autoLockMinutes === "30" ? " selected" : ""}>30 minutes</option><option value="never"${state.autoLockMinutes === "never" ? " selected" : ""}>Never</option></select></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Lock now</strong><small>End the in-memory wallet session and hide all wallet content</small></span><button class="button" type="button" data-demo-action="lock">${icon("lock")} Lock now</button></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Hide sensitive amounts</strong><small>Mask balances and transaction values</small></span><button class="toggle" type="button" data-demo-action="toggle-balance" aria-pressed="${state.balanceHidden}" aria-label="Hide sensitive amounts"></button></div>
      </div>
      <div class="setting-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Recovery phrase</strong><small>Requires password and a private display check</small></span><button class="button" type="button" data-demo-action="seed-warning">View phrase</button></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Master key</strong><small>Last rotated when wallet was created</small></span><button class="button" type="button" data-demo-action="key-rotation">Rotate</button></div>
      </div>`;
  }

  if (state.settingsSection === "backup") {
    return `
      <h2>Backups</h2><p>Backups are local unless you explicitly choose a configured provider.</p>
      <div class="review-card">
        <div class="summary-row"><span>Latest backup</span><strong>10 Jul 2026 · 09:42</strong></div>
        <div class="summary-row"><span>Integrity</span><strong class="trust-label">${icon("shield")} Verified</strong></div>
        <div class="summary-row"><span>Destination</span><strong>Encrypted local file</strong></div>
      </div>
      <div class="setting-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Create a fresh backup</strong><small>Choose a destination, authenticate, then verify integrity</small></span><button class="button button-primary" type="button" data-demo-action="backup">${icon("backup")} Create backup</button></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Restore a backup</strong><small>Validate a backup before replacing wallet state</small></span><button class="button" type="button" data-demo-action="restore">Restore</button></div>
      </div>`;
  }

  if (state.settingsSection === "network") {
    return `
      <div class="settings-heading"><div><p class="eyebrow">Overlay, carrier, chain</p><h2>Network & privacy</h2><p>OnionNet protects the route; Reticulum carries it. Chain remains separate.</p></div><select aria-label="Network mode"><option>Private · no direct fallback</option><option>Auto</option><option>Resilient</option><option>Direct · warning</option></select></div>
      ${networkDetail()}`;
  }

  if (state.settingsSection === "policies") {
    return `
      <div class="settings-heading"><div><p class="eyebrow">Target profile preview</p><h2>Safety & policy profiles</h2><p>Profiles can narrow behavior. They cannot change protocol rules or expand your authority.</p></div><button class="button button-primary" type="button" data-demo-action="load-policy">${icon("backup")} Preview profile</button></div>
      <div class="policy-stack" aria-label="Policy precedence">
        <div class="policy-layer is-locked"><span>1</span><div><strong>Protocol rules</strong><small>Native cash conservation · immutable in wallet</small></div><span class="status-badge">Locked</span></div>
        <div class="policy-layer"><span>2</span><div><strong>Organization</strong><small>No managed profile · signed profiles only</small></div><button class="button" type="button" data-demo-action="load-policy">Load</button></div>
        <div class="policy-layer is-active"><span>3</span><div><strong>Personal Safe · v1.4</strong><small>Target example · max payment 2,500 · daily 5,000 · confirmation required</small></div><span class="status-badge is-ready">Preview</span></div>
        <div class="policy-layer"><span>4</span><div><strong>Per-action attenuation</strong><small>May only make the current action narrower</small></div><span class="status-badge">As needed</span></div>
      </div>
      <button class="why-blocked" type="button" data-demo-action="why-blocked">${icon("alert")}<span><strong>Why a 3,200 Z00Z payment would be blocked</strong><small>Target Personal Safe preview → maximum transaction is 2,500 Z00Z</small></span>${icon("chevron")}</button>
      <div class="notice">${icon("shield")} A loaded profile is not proof of legal compliance. Invalid signatures, expired schemas, and ambiguous conflicts fail closed and go to quarantine.</div>`;
  }

  if (state.settingsSection === "appearance") {
    return `
      <div class="settings-heading"><div><p class="eyebrow">Protected semantics</p><h2>Appearance</h2><p>Personalize brand surfaces while safety, privacy, and environment colors stay unambiguous.</p></div><span class="config-source">Source · YAML</span></div>
      <div class="setting-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Theme</strong><small>System follows the operating system</small></span><div class="segmented" aria-label="Theme"><button type="button" data-theme="system" class="${state.theme === "system" ? "is-active" : ""}>System</button><button type="button" data-theme="dark" class="${state.theme === "dark" ? "is-active" : ""}">${icon("moon")} Dark</button><button type="button" data-theme="light" class="${state.theme === "light" ? "is-active" : ""}">${icon("sun")} Light</button></div></div>
        <div class="setting-line palette-setting"><span class="setting-line-copy"><strong>Palette</strong><small>Changes decorative and primary-action colors; safety colors remain semantic.</small></span><div class="palette-grid" aria-label="Palette presets">${paletteOptions.map(paletteCard).join("")}</div></div>
        <div class="setting-line palette-setting code-theme-setting"><span class="setting-line-copy"><strong>Code highlighting</strong><small>Changes YAML syntax colours only. It does not affect wallet safety colours or runtime data.</small></span><div class="code-theme-sections" aria-label="YAML code highlighting theme"><section><p class="code-theme-group-label">Light</p><div class="code-theme-grid">${codeThemeOptions.filter((theme) => theme.mode === "light").map(codeThemeCard).join("")}</div></section><section><p class="code-theme-group-label">Dark</p><div class="code-theme-grid">${codeThemeOptions.filter((theme) => theme.mode === "dark").map(codeThemeCard).join("")}</div></section></div></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Custom accents</strong><small>Fine-tune brand and privacy rail only. Safety, warning, failure, and focus colours stay protected.</small></span><div class="custom-color-controls"><label>Brand<input type="color" data-config-control="custom-brand" value="${state.customAppearance.brand}" aria-label="Custom brand color"></label><label>Privacy rail<input type="color" data-config-control="custom-rail" value="${state.customAppearance.rail}" aria-label="Custom privacy rail color"></label></div></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Text scale</strong><small>Interface reflows without hiding content</small></span><select aria-label="Text scale" data-config-control="text-scale"><option value="100"${state.textScale === "100" ? " selected" : ""}>100%</option><option value="110"${state.textScale === "110" ? " selected" : ""}>110%</option><option value="125"${state.textScale === "125" ? " selected" : ""}>125%</option></select></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Reduced motion</strong><small>Reduce interface motion in addition to operating system preferences</small></span><button class="toggle" type="button" aria-pressed="${state.reducedMotion}" aria-label="Use reduced motion" data-demo-action="motion"></button></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Compact desktop lists</strong><small>Keep touch targets at least 44 pixels</small></span><button class="toggle" type="button" aria-pressed="${state.compactLists}" aria-label="Use compact desktop lists" data-demo-action="compact"></button></div>
      </div>`;
  }

  return advancedConfigContent();
}

function settingsView() {
  return `
    <div class="view-enter settings-view">
      <div class="workspace-layout settings-layout">
        <aside class="context-rail">${settingsContextNav()}</aside>
        <article class="card settings-detail">${settingsDetail()}</article>
      </div>
    </div>`;
}

function addWalletView() {
  return `
    <section class="view-enter add-wallet-view" aria-labelledby="add-wallet-heading">
      <div class="add-wallet-stage">
        <div class="add-wallet-card">
          <span class="add-wallet-mark" aria-hidden="true">${icon("wallet")}</span>
          <p class="eyebrow">Local wallet profiles</p>
          <h2 id="add-wallet-heading">Bring a wallet to this device</h2>
          <p class="add-wallet-copy">Choose how to begin. Wallet keys, passwords, and recovery words remain local to this device in this concept.</p>
          <div class="add-wallet-actions" aria-label="Add wallet options">
            <button class="button add-wallet-choice is-primary" type="button" data-demo-action="create-wallet">${icon("plus")} Create new wallet</button>
            <button class="button add-wallet-choice is-primary" type="button" data-demo-action="open-existing-wallet">${icon("wallet")} Open existing wallet</button>
            <button class="button add-wallet-choice" type="button" data-demo-action="restore-wallet">${icon("backup")} Restore from backup</button>
          </div>
          <button class="button button-quiet add-wallet-back" type="button" data-view="home">Cancel</button>
        </div>
      </div>
    </section>`;
}

function render(options = {}) {
  applyAppearancePreferences();
  renderWalletShell();
  const sidebarTarget = sidebarActiveTarget();
  const walletScreen = hasSelectedWalletContext();
  const wallet = activeWallet();
  const [title, context] = headings[state.view];
  pageTitle.textContent = walletScreen ? wallet.address : title;
  pageContext.textContent = walletScreen ? `${wallet.name} wallet` : context;
  pageTitle.classList.toggle("is-wallet-address", walletScreen);
  copyWalletAddress.hidden = !walletScreen;
  walletIdentity.hidden = !walletScreen;

  document.querySelectorAll("[data-view]").forEach((button) => {
    const active = button.closest(".system-nav")
      ? sidebarTarget.group === "settings"
      : button.dataset.view === state.view;
    button.classList.toggle("is-active", active);
    if (button.closest("nav")) {
      active ? button.setAttribute("aria-current", "page") : button.removeAttribute("aria-current");
    }
  });

  main.innerHTML = {
    home: homeView,
    wallet: walletView,
    activity: activityView,
    swap: swapView,
    exchange: exchangeView,
    staking: stakingView,
    "wallet-backup": walletBackupView,
    "wallet-settings": walletSettingsView,
    settings: settingsView,
    "add-wallet": addWalletView
  }[state.view]();

  syncBalanceButtons();
  requestAnimationFrame(() => {
    walletTabs.querySelector(".wallet-tab.is-active")?.scrollIntoView({ block: "nearest", inline: "center" });
    const activeContext = main.querySelector(".context-nav-child.is-active") || main.querySelector(".context-nav-item.is-active");
    activeContext?.scrollIntoView({ block: "nearest", inline: "center" });
  });
  if (options.focusMain) {
    main.focus({ preventScroll: true });
    window.scrollTo({ top: 0, behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
  }
}

function syncBalanceButtons() {
  document.querySelectorAll('[data-demo-action="toggle-balance"]').forEach((button) => {
    const label = state.balanceHidden ? "Show sensitive amounts" : "Hide sensitive amounts";
    button.setAttribute("aria-label", label);
    button.setAttribute("title", label);
    if (button.classList.contains("toggle")) button.setAttribute("aria-pressed", String(state.balanceHidden));
    const use = button.querySelector("use");
    if (use) use.setAttribute("href", state.balanceHidden ? "#i-eye-off" : "#i-eye");
  });
}

function validateAndApplyWalletSettingsYaml(source, apply = false) {
  const forbidden = /(^|\n)\s*(password|seed|private_key|session_token|receiver_secret|path):/i;
  if (!/^schema_version:\s*1\s*$/m.test(source) || !/^wallet:\s*$/m.test(source)) return { valid: false, message: "Use schema_version: 1 and a wallet section." };
  if (forbidden.test(source)) return { valid: false, message: "Secrets and local paths are not allowed in wallet settings YAML." };

  const name = readYamlScalar(source, "name");
  const currency = readYamlScalar(source, "currency");
  const defaultFee = readYamlScalar(source, "default_fee");
  const lockAfter = readYamlScalar(source, "lock_after_minutes");
  const backupInterval = readYamlScalar(source, "interval_hours");
  const maxTransaction = readYamlScalar(source, "max_transaction");
  const maxDaily = readYamlScalar(source, "max_daily");
  const requireConfirmation = readYamlScalar(source, "require_confirmation");
  const allowedAssets = readYamlScalar(source, "allowed_assets");
  const allowedRecipients = readYamlScalar(source, "allowed_recipients");
  const timeRestrictions = readYamlScalar(source, "time_restrictions");

  if (name && (name.length < 2 || name.length > 32)) return { valid: false, message: "Wallet name must contain 2–32 characters." };
  if (currency && !["Z00Z", "USD", "EUR"].includes(currency)) return { valid: false, message: "currency must be Z00Z, USD, or EUR." };
  if (defaultFee && !/^\d+(?:\.\d+)?$/.test(defaultFee)) return { valid: false, message: "default_fee must be a non-negative decimal." };
  if (lockAfter && !["5", "15", "30", "never"].includes(lockAfter.toLowerCase())) return { valid: false, message: "lock_after_minutes must be 5, 15, 30, or never." };
  if (backupInterval && !["6", "24", "72"].includes(backupInterval)) return { valid: false, message: "interval_hours must be 6, 24, or 72." };
  if (maxTransaction && !/^\d+(?:\.\d+)?$/.test(maxTransaction)) return { valid: false, message: "max_transaction must be a non-negative decimal." };
  if (maxDaily && !/^\d+(?:\.\d+)?$/.test(maxDaily)) return { valid: false, message: "max_daily must be a non-negative decimal." };
  if (requireConfirmation && !["true", "false"].includes(requireConfirmation)) return { valid: false, message: "require_confirmation must be true or false." };
  if (allowedAssets && !["all", "native"].includes(allowedAssets)) return { valid: false, message: "allowed_assets must be all or native." };
  if (timeRestrictions && !["any", "business-hours"].includes(timeRestrictions)) return { valid: false, message: "time_restrictions must be any or business-hours." };

  if (apply) {
    const wallet = activeWallet();
    const preferences = activeWalletPreferences();
    if (name) {
      wallet.name = name;
      wallet.initials = name.slice(0, 1).toUpperCase();
    }
    if (currency) preferences.currency = currency;
    if (defaultFee) preferences.defaultFee = defaultFee;
    if (lockAfter) preferences.lockAfterMinutes = lockAfter.toLowerCase();
    if (backupInterval) preferences.backupIntervalHours = backupInterval;
    if (maxTransaction) preferences.policyRules.maxTransaction = maxTransaction;
    if (maxDaily) preferences.policyRules.maxDaily = maxDaily;
    if (requireConfirmation) preferences.policyRules.requireConfirmation = requireConfirmation === "true";
    if (allowedAssets) preferences.policyRules.allowedAssets = allowedAssets;
    if (allowedRecipients) preferences.policyRules.allowedRecipients = allowedRecipients === "any" ? "" : allowedRecipients;
    if (timeRestrictions) preferences.policyRules.timeWindow = timeRestrictions;
    state.walletSettingsConfigDraft = "";
    syncConfigDraftFromState();
  }
  return { valid: true, message: apply ? "Selected wallet settings applied locally in this concept." : "Selected wallet YAML is valid for the concept schema." };
}

function sensitiveWalletDialog(type) {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  if (type === "wallet-policy-profile") {
    return dialogFrame({
      title: "Compliance profile preview",
      subtitle: "User-configured policy, not a certificate",
      body: `<div class="confirmation-note">${icon("alert")} A profile can guide local restrictions and scoped disclosure choices. It cannot prove legal status, override protocol rules, or expand authority.</div><div class="review-card"><div class="summary-row"><span>Profile</span><strong>${escapeHtml(preferences.policyProfile)}</strong></div><div class="summary-row"><span>Scope</span><strong>${escapeHtml(wallet.name)} only</strong></div><div class="summary-row"><span>Signature / apply route</span><strong>Unavailable in current RPC</strong></div></div><div class="policy-stack"><div class="policy-layer is-locked"><span>1</span><div><strong>Protocol rules</strong><small>Always enforced and not editable.</small></div><span class="status-badge">Locked</span></div><div class="policy-layer"><span>2</span><div><strong>Jurisdiction profile</strong><small>Target preview; no managed claim or legal certification.</small></div><span class="status-badge is-ready">Target</span></div><div class="policy-layer is-active"><span>3</span><div><strong>Local <code>PolicyRules</code></strong><small>Spend limits and confirmation preferences can narrow this wallet.</small></div><span class="status-badge is-active">Local</span></div></div>`,
      footer: `<button class="button button-primary" type="button" data-dialog-close>Close</button>`
    });
  }

  const definitions = {
    "wallet-rename": {
      title: "Rename wallet",
      subtitle: "Confirm with the wallet password",
      confirmation: null,
      body: `<div class="field-group"><label class="field-label" for="wallet-rename-name">Wallet name</label><input id="wallet-rename-name" name="name" maxlength="32" value="${escapeHtml(wallet.name)}" autocomplete="off" required><p class="field-hint">This local label does not change the wallet address or key material.</p></div>`,
      actionLabel: "Save wallet name"
    },
    "wallet-seed-reveal": {
      title: "View recovery phrase",
      subtitle: "Private display only · critical operation",
      confirmation: "SHOW SEED",
      body: `<div class="confirmation-note">${icon("alert")} Never share recovery words with support, a website, or a remote-access session. Close the dialog to clear them from this renderer.</div>`,
      actionLabel: "Reveal demonstration phrase"
    },
    "wallet-public-export": {
      title: "Prepare public-material export",
      subtitle: "Encrypted export after password check",
      confirmation: null,
      body: `<div class="notice">${icon("shield")} The wallet route exports encrypted public material. It does not expose a private key in the interface.</div>`,
      actionLabel: "Prepare encrypted export"
    },
    "wallet-key-rotation": {
      title: "Rotate master key",
      subtitle: "Rewrap protected wallet records",
      confirmation: "ROTATE",
      body: `<div class="confirmation-note">${icon("alert")} This critical operation re-encrypts protected local records. Keep a verified backup before continuing. The wallet service rate-limits successful rotation.</div>`,
      actionLabel: "Rotate master key"
    },
    "wallet-policy-apply": {
      title: "Review local spend rules",
      subtitle: "Narrow this wallet's behavior",
      confirmation: "APPLY",
      body: `<div class="form-grid policy-rule-form"><div class="field-group"><label class="field-label" for="wallet-policy-max-tx">Maximum transaction</label><div class="input-with-affix"><input id="wallet-policy-max-tx" name="maxTransaction" inputmode="decimal" value="${escapeHtml(preferences.policyRules.maxTransaction)}" required><span class="input-affix">Z00Z</span></div></div><div class="field-group"><label class="field-label" for="wallet-policy-max-daily">Maximum daily total</label><div class="input-with-affix"><input id="wallet-policy-max-daily" name="maxDaily" inputmode="decimal" value="${escapeHtml(preferences.policyRules.maxDaily)}" required><span class="input-affix">Z00Z</span></div></div><div class="field-group"><label class="field-label" for="wallet-policy-assets">Allowed assets</label><select id="wallet-policy-assets" name="allowedAssets"><option value="all"${preferences.policyRules.allowedAssets === "all" ? " selected" : ""}>All supported assets</option><option value="native"${preferences.policyRules.allowedAssets === "native" ? " selected" : ""}>Native Z00Z only</option></select></div><div class="field-group"><label class="field-label" for="wallet-policy-time">Time restrictions</label><select id="wallet-policy-time" name="timeWindow"><option value="any"${preferences.policyRules.timeWindow === "any" ? " selected" : ""}>Any time</option><option value="business-hours"${preferences.policyRules.timeWindow === "business-hours" ? " selected" : ""}>Business hours UTC</option></select></div><div class="field-group policy-rule-recipient"><label class="field-label" for="wallet-policy-recipient">Allowed recipients <span class="muted">(optional)</span></label><input id="wallet-policy-recipient" name="allowedRecipients" maxlength="160" value="${escapeHtml(preferences.policyRules.allowedRecipients)}" placeholder="Leave blank to allow all recipients"><p class="field-hint">A target integration must parse and validate each receiver identifier before save.</p></div><label class="checkbox-line"><input name="requireConfirmation" type="checkbox"${preferences.policyRules.requireConfirmation ? " checked" : ""}> <span><strong>Require settlement confirmation</strong><small>Block another local spend while a prior one awaits settlement.</small></span></label></div><div class="notice">${icon("shield")} Rules remain local to this concept. Signed profile application is a target capability, not part of this action.</div>`,
      actionLabel: "Apply local rules"
    }
  };
  const definition = definitions[type];
  if (state.flow.step === 1) {
    const result = type === "wallet-seed-reveal"
      ? `<div class="confirmation-note">${icon("alert")} Demonstration words only. Never copy recovery words to a shared clipboard.</div><ol class="seed-grid" aria-label="Demonstration recovery phrase">${demoSeedWords.map((word, index) => `<li><span>${index + 1}</span><strong>${word}</strong></li>`).join("")}</ol><p class="seed-demo-label">DEMONSTRATION WORDS · NOT A REAL WALLET SEED</p>`
      : type === "wallet-public-export"
        ? `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Encrypted export prepared</h3><p>Only encrypted public material is represented. It is not placed on the clipboard.</p></div><code class="request-code">z00z-public-export:encrypted:${escapeHtml(wallet.id)}:account-0</code>`
        : type === "wallet-key-rotation"
          ? `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Master key rotated</h3><p>Protected local records were rewrapped in this concept. A production UI would show the returned fingerprint and record count.</p></div>`
          : type === "wallet-policy-apply"
            ? `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Local spend rules updated</h3><p>They narrow ${escapeHtml(wallet.name)} only and never claim regulatory compliance.</p></div>`
            : `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Wallet name updated</h3><p>${escapeHtml(wallet.name)} remains the same local wallet with the same address and keys.</p></div>`;
    return dialogFrame({ title: definition.title, subtitle: "Local concept result", body: result, footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>` });
  }
  const passwordId = `${type}-password`;
  const confirmationMarkup = definition.confirmation ? `<div class="field-group"><label class="field-label" for="${type}-confirmation">Type ${definition.confirmation}</label><input id="${type}-confirmation" name="confirmation" autocomplete="off" autocapitalize="characters" spellcheck="false" required><p class="field-hint">This exact phrase prevents accidental execution.</p></div>` : "";
  return dialogFrame({
    title: definition.title,
    subtitle: definition.subtitle,
    body: `<form class="form-grid" id="${type}-entry" novalidate>${definition.body}<div class="field-group"><label class="field-label" for="${passwordId}">Wallet password</label><input id="${passwordId}" name="password" type="password" minlength="8" autocomplete="current-password" required><p class="field-hint">This concept validates locally and clears the value immediately after use.</p><p class="field-error" id="${type}-error" role="alert"></p></div>${confirmationMarkup}</form>`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="${type}-entry">${definition.actionLabel}</button>`
  });
}

function dialogFrame({ title, subtitle, body, footer = "", steps = 0, activeStep = 0, closeLabel = "Close" }) {
  const indicators = steps > 1
    ? `<div class="step-indicator" aria-label="Step ${activeStep + 1} of ${steps}">${Array.from({ length: steps }, (_, index) => `<span class="${index < activeStep ? "is-done" : index === activeStep ? "is-active" : ""}"></span>`).join("")}</div>`
    : "";
  return `
    <div class="dialog-shell">
      <header class="dialog-header">
        <div class="dialog-header-copy"><h2 id="dialog-title">${title}</h2><p>${subtitle}</p></div>
        ${indicators}
        <button class="icon-button" type="button" data-dialog-close aria-label="${closeLabel}">${icon("close")}</button>
      </header>
      <div class="dialog-body">${body}</div>
      ${footer ? `<footer class="dialog-footer">${footer}</footer>` : ""}
    </div>`;
}

function payDialog() {
  const data = state.flow.data;
  const asset = flowAsset(data);
  const wallet = activeWallet();
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Send privately",
      subtitle: "Recipient, asset, and amount",
      steps: 3,
      activeStep: 0,
      body: `
        <form class="form-grid" id="pay-entry" novalidate>
          <div class="field-group"><label class="field-label" for="pay-recipient">Recipient or private request</label><input id="pay-recipient" name="recipient" value="${escapeHtml(data.recipient)}" placeholder="Paste or scan a private request" autocomplete="off" aria-describedby="pay-recipient-hint pay-recipient-error" required><p class="field-hint" id="pay-recipient-hint">The wallet validates the receiver, asset, and network before review.</p><p class="field-error" id="pay-recipient-error"></p></div>
          <div class="field-group"><label class="field-label" for="pay-asset">Asset</label><select id="pay-asset" name="assetKey">${assetOptions(asset.key)}</select><p class="field-hint">${escapeHtml(asset.kind)} held by ${escapeHtml(wallet.name)}.</p></div>
          <div class="field-group"><label class="field-label" for="pay-amount">Amount</label><div class="input-with-affix"><input id="pay-amount" name="amount" type="number" min="${asset.divisible ? "0.01" : "1"}" max="${escapeHtml(asset.balance.replaceAll(",", ""))}" step="${asset.divisible ? "0.01" : "1"}" inputmode="decimal" value="${escapeHtml(data.amount)}" placeholder="${asset.divisible ? "0.00" : "1"}" aria-describedby="pay-amount-hint pay-amount-error" required><span class="input-affix">${escapeHtml(asset.unit)}</span></div><p class="field-hint" id="pay-amount-hint">Available: ${sensitive(`${asset.balance} ${asset.unit}`)} · fee shown before authorization</p><p class="field-error" id="pay-amount-error"></p></div>
          <div class="field-group"><label class="field-label" for="pay-memo">Private note <span class="muted">(optional)</span></label><input id="pay-memo" name="memo" value="${escapeHtml(data.memo)}" maxlength="80" placeholder="What is this for?"></div>
        </form>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="pay-entry">Review send ${icon("chevron")}</button>`
    });
  }

  if (state.flow.step === 1) {
    return dialogFrame({
      title: "Review send",
      subtitle: "Check before authorizing",
      steps: 3,
      activeStep: 1,
      body: `
        <div class="review-card review-hero"><span class="list-icon">${icon("send")}</span><strong>${escapeHtml(data.amount)} ${escapeHtml(asset.unit)}</strong><span>${escapeHtml(asset.label)} to ${escapeHtml(data.recipientLabel)}</span></div>
        <div class="review-card">
          <div class="summary-row"><span>Asset</span><strong>${escapeHtml(asset.label)} · ${escapeHtml(asset.kind)}</strong></div>
          <div class="summary-row"><span>Recipient</span><strong>${escapeHtml(data.recipientLabel)} · <span class="mono">7D3B…9A40</span></strong></div>
          <div class="summary-row"><span>From</span><strong>${escapeHtml(wallet.name)} wallet</strong></div>
          <div class="summary-row"><span>Fee</span><strong>Included</strong></div>
          <div class="summary-row"><span>Privacy route</span><strong>OnionNet · target simulation</strong></div>
          <div class="summary-row"><span>Carrier</span><strong>Reticulum · target</strong></div>
          <div class="summary-row"><span>Network</span><strong><span class="environment-tag is-main">MAIN</span></strong></div>
          ${data.memo ? `<div class="summary-row"><span>Note</span><strong>${escapeHtml(data.memo)}</strong></div>` : ""}
        </div>
        <div class="confirmation-note">${icon("shield")} Sending authorizes this asset transfer once. It will appear as settling until the wallet confirms final state.</div>`,
      footer: `<button class="button" type="button" data-dialog-action="pay-back">Back</button><button class="button button-primary" type="button" data-dialog-action="pay-submit">Send asset</button>`
    });
  }

  return dialogFrame({
    title: "Asset sent",
    subtitle: "Waiting for final settlement",
    steps: 3,
    activeStep: 2,
    body: `
      <div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Sent · settling</h3><p>${escapeHtml(asset.label)} was accepted for processing for ${escapeHtml(data.recipientLabel)}. It is not final yet.</p><div class="receipt-ref mono">Reference TX-8A42 · idempotency protected</div></div>
      <div class="review-card"><div class="summary-row"><span>Amount</span><strong>${escapeHtml(data.amount)} ${escapeHtml(asset.unit)}</strong></div><div class="summary-row"><span>Fee</span><strong>Shown before authorization</strong></div><div class="summary-row"><span>Next update</span><strong>Automatic</strong></div></div>`,
    footer: `<button class="button" type="button" data-dialog-action="view-activity">View history</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function receiveDialog() {
  const data = state.flow.data;
  const asset = flowAsset(data);
  const wallet = activeWallet();
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Receive privately",
      subtitle: "Create an asset request",
      steps: 2,
      activeStep: 0,
      body: `
        <form class="form-grid" id="receive-entry" novalidate>
          <div class="field-group"><label class="field-label" for="receive-asset">Asset</label><select id="receive-asset" name="assetKey">${assetOptions(asset.key)}</select><p class="field-hint">Create a request for a coin, token, or collectible.</p></div>
          <div class="field-group"><label class="field-label" for="receive-amount">Requested amount <span class="muted">(optional)</span></label><div class="input-with-affix"><input id="receive-amount" name="amount" type="number" min="${asset.divisible ? "0.01" : "1"}" step="${asset.divisible ? "0.01" : "1"}" inputmode="decimal" value="${escapeHtml(data.amount)}" placeholder="${asset.divisible ? "Any amount" : "1"}"><span class="input-affix">${escapeHtml(asset.unit)}</span></div></div>
          <div class="field-group"><label class="field-label" for="receive-note">What is it for? <span class="muted">(optional)</span></label><input id="receive-note" name="note" maxlength="80" value="${escapeHtml(data.note)}" placeholder="Dinner, invoice, refund…"></div>
          <div class="field-group"><label class="field-label" for="receive-expiry">Request expires</label><select id="receive-expiry" name="expiry"><option>In 24 hours</option><option>In 7 days</option><option>Never</option></select><p class="field-hint">Expiry limits how long this request should be trusted.</p></div>
        </form>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="receive-entry">Create request</button>`
    });
  }

  return dialogFrame({
    title: "Private asset request",
    subtitle: "Share the QR or request text",
    steps: 2,
    activeStep: 1,
    body: `
      <div class="qr-layout"><div class="mock-qr" aria-label="Mock asset request QR code">${qrCells()}</div><div><p class="eyebrow">Ready to share</p><h3>${data.amount ? `${escapeHtml(data.amount)} ${escapeHtml(asset.unit)}` : `Any ${escapeHtml(asset.label)} amount`}</h3><p class="muted">${escapeHtml(wallet.name)} wallet · expires in 24 hours</p><code class="request-code">z00z:receive:${escapeHtml(asset.key)}?amount=${escapeHtml(data.amount || "any")}</code><button class="button button-full" type="button" data-demo-action="copy-request">${icon("copy")} Copy request</button></div></div>
      <div class="notice">${icon("shield")} The sender sees “${escapeHtml(wallet.name)} wallet” and an abbreviated receiver. Incoming value appears as settling before it becomes available.</div>`,
    footer: `<button class="button" type="button" data-demo-action="share-request">Share</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function assetClaimDialog() {
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Claim asset allocation",
      subtitle: "One source, one recipient, one replay-safe claim",
      steps: 2,
      activeStep: 0,
      body: `
        <div class="review-card review-hero"><span class="list-icon is-claim">${icon("claim")}</span><strong>86.00 Z00Z</strong><span>Genesis allocation #014</span></div>
        <div class="review-card"><div class="summary-row"><span>Claim source</span><strong>Allocation root · proof present</strong></div><div class="summary-row"><span>Authority</span><strong>Signature present</strong></div><div class="summary-row"><span>Recipient</span><strong>Everyday wallet · bound</strong></div><div class="summary-row"><span>Output</span><strong>Z00Z Coin · 86.00</strong></div><div class="summary-row"><span>Replay protection</span><strong>Chain-bound nullifier</strong></div></div>
        <div class="confirmation-note">${icon("shield")} The claim package is separate from vouchers. A successful claim creates owned Asset output and can be used only once.</div>
        <div class="capability-note">${icon("alert")} <span><strong>Target claim intake</strong><small>Live code verifies ClaimTxPackage, but the current wallet RPC has no dedicated high-level claim intake/build method. Production keeps this action capability-gated.</small></span></div>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="button" data-dialog-action="asset-claim-submit">Verify and claim once</button>`
    });
  }

  return dialogFrame({
    title: "Claim submitted",
    subtitle: "Waiting for final settlement",
    steps: 2,
    activeStep: 1,
    body: `<div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Asset receiving · settling</h3><p>The verified claim output is tracked as an Asset. It is not included in Available until authoritative settlement makes it spendable.</p><div class="receipt-ref mono">Claim CLM-883C · nullifier reserved once</div></div>`,
    footer: `<button class="button" type="button" data-dialog-action="view-activity">View history</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function voucherDialog(settled = false) {
  if (settled) {
    return dialogFrame({
      title: "Event deposit return",
      subtitle: "Voucher history",
      body: `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Redeemed · settled</h3><p>The voucher was redeemed and its resulting asset settled on 12 Jul 2026.</p></div><div class="review-card"><div class="summary-row"><span>Issuer</span><strong>Riverside Events</strong></div><div class="summary-row"><span>Face / remaining</span><strong>150.00 / 0.00 Z00Z</strong></div><div class="summary-row"><span>Lifecycle</span><strong>Redeemed</strong></div></div><details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>Object: voucher_04e9…af31</span><span>Lifecycle: offered → accepted → redeemed</span></div></details>`,
      footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
    });
  }

  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Review voucher",
      subtitle: "Conditional value offered to this wallet",
      steps: 3,
      activeStep: 0,
      body: `<div class="review-card review-hero"><span class="list-icon is-claim">${icon("claim")}</span><strong>86.00 Z00Z</strong><span>Travel refund voucher</span></div><div class="review-card"><div class="summary-row"><span>Issuer</span><strong>Northwind Travel</strong></div><div class="summary-row"><span>Backing</span><strong>Consumed asset reference</strong></div><div class="summary-row"><span>Face / remaining</span><strong>86.00 / 86.00 Z00Z</strong></div><div class="summary-row"><span>Acceptance</span><strong>Required</strong></div><div class="summary-row"><span>Ends</span><strong>21 Jul 2026 · 18:00</strong></div><div class="summary-row"><span>Holder options</span><strong>Accept · Reject</strong></div></div><div class="confirmation-note">${icon("shield")} Accepting changes the voucher lifecycle. It does not directly add 86.00 Z00Z to Available.</div>`,
      footer: `<button class="button button-danger" type="button" data-dialog-action="voucher-reject">Reject voucher</button><button class="button button-primary" type="button" data-dialog-action="voucher-accept">Accept voucher</button>`
    });
  }

  if (state.flow.step === 1) {
    return dialogFrame({
      title: "Voucher accepted",
      subtitle: "Now redeemable",
      steps: 3,
      activeStep: 1,
      body: `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Accepted · redeemable</h3><p>The voucher remains conditional value. Redeem it to request its asset outcome.</p></div><div class="review-card"><div class="summary-row"><span>Remaining value</span><strong>86.00 Z00Z</strong></div><div class="summary-row"><span>Next action</span><strong>Redeem full voucher</strong></div></div>`,
      footer: `<button class="button" type="button" data-dialog-close>Later</button><button class="button button-primary" type="button" data-dialog-action="voucher-redeem">Redeem voucher</button>`
    });
  }

  return dialogFrame({
    title: "Voucher redeemed",
    subtitle: "Asset outcome is settling",
    steps: 3,
    activeStep: 2,
    body: `<div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Redeemed · receiving</h3><p>The voucher lifecycle is redeemed. Its asset outcome is waiting for authoritative settlement and is not Available yet.</p></div>`,
    footer: `<button class="button" type="button" data-dialog-action="view-activity">View history</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function permissionDialog() {
  const data = state.flow.data;
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Give permission",
      subtitle: "Delegate a narrower right you already hold",
      steps: 3,
      activeStep: 0,
      body: `
        <form class="form-grid" id="permission-entry" novalidate>
          <div class="field-group"><label class="field-label" for="permission-source">Held authority</label><select id="permission-source" name="source"><option>Deploy to staging · machine capability</option></select><p class="field-hint">Only held, delegable authority is offered. Right creation is a separate issuer capability.</p></div>
          <div class="field-group"><label class="field-label" for="permission-delegate">Delegate</label><input id="permission-delegate" name="delegate" value="${escapeHtml(data.delegate)}" placeholder="Verified service or known identity" required aria-describedby="permission-delegate-error"><p class="field-error" id="permission-delegate-error"></p></div>
          <div class="field-group"><label class="field-label" for="permission-action">Allowed action</label><select id="permission-action" name="action"><option>Deploy release</option><option>View status</option></select></div>
          <div class="field-group"><label class="field-label" for="permission-scope">Scope</label><input id="permission-scope" name="scope" value="${escapeHtml(data.scope)}" readonly></div>
          <div class="field-group"><label class="field-label" for="permission-uses">Maximum uses</label><input id="permission-uses" name="uses" type="number" min="1" max="5" inputmode="numeric" value="${escapeHtml(data.uses)}" required aria-describedby="permission-uses-error"><p class="field-error" id="permission-uses-error"></p></div>
          <div class="field-group"><label class="field-label" for="permission-expiry">Ends</label><input id="permission-expiry" name="expiry" type="date" value="${escapeHtml(data.expiry)}" min="2026-07-20" required></div>
        </form>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="permission-entry">Review permission ${icon("chevron")}</button>`
    });
  }

  if (state.flow.step === 1) {
    return dialogFrame({
      title: "Review permission",
      subtitle: "The delegated right can only become narrower",
      steps: 3,
      activeStep: 1,
      body: `
        <div class="review-card review-hero"><span class="list-icon is-warning">${icon("permission")}</span><strong>${escapeHtml(data.uses)} uses</strong><span>for ${escapeHtml(data.delegate)}</span></div>
        <div class="review-card"><div class="summary-row"><span>Class</span><strong>Machine capability</strong></div><div class="summary-row"><span>Can</span><strong>${escapeHtml(data.action)}</strong></div><div class="summary-row"><span>Only within</span><strong>${escapeHtml(data.scope)}</strong></div><div class="summary-row"><span>Use limit</span><strong>${escapeHtml(data.uses)}</strong></div><div class="summary-row"><span>Ends</span><strong>${escapeHtml(data.expiryLabel)}</strong></div><div class="summary-row"><span>Cannot</span><strong>Sub-delegate or broaden scope</strong></div><div class="summary-row"><span>Monetary value</span><strong>None · Right is zero-value</strong></div></div>
        <div class="confirmation-note">${icon("alert")} Delegation transfers bounded authority. Revocation cannot be described as cancelling work already accepted by the protocol.</div>`,
      footer: `<button class="button" type="button" data-dialog-action="permission-back">Back</button><button class="button button-primary" type="button" data-dialog-action="permission-submit">Give permission</button>`
    });
  }

  return dialogFrame({
    title: "Permission delegated",
    subtitle: "Bounded authority is being tracked",
    steps: 3,
    activeStep: 2,
    body: `<div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Delegating · settling</h3><p>${escapeHtml(data.delegate)} may ${escapeHtml(data.action).toLowerCase()} within ${escapeHtml(data.scope)} up to ${escapeHtml(data.uses)} times, ending ${escapeHtml(data.expiryLabel)}.</p><div class="receipt-ref mono">Right RGT-40A1 · zero-value · attenuation only</div></div>`,
    footer: `<button class="button" type="button" data-dialog-action="go-actions">View permissions</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function permissionDetailDialog() {
  return dialogFrame({
    title: "Delivery receipt access",
    subtitle: "Held data-access permission",
    body: `
      <div class="review-card review-hero"><span class="list-icon is-warning">${icon("permission")}</span><strong>2 of 5 uses</strong><span>remaining</span></div>
      <div class="review-card"><div class="summary-row"><span>Class</span><strong>Data access</strong></div><div class="summary-row"><span>Allowed action</span><strong>View receipt</strong></div><div class="summary-row"><span>Scope</span><strong>receipts.example</strong></div><div class="summary-row"><span>Delegation</span><strong>Forbidden</strong></div><div class="summary-row"><span>Ends</span><strong>31 Jul 2026</strong></div><div class="summary-row"><span>Monetary value</span><strong>None</strong></div><div class="summary-row"><span>Status</span><strong><span class="status-badge is-active">Held</span></strong></div></div>
      <details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>Right: right_54ac…1f88</span><span>Class: data_access</span><span>Lifecycle: granted → held</span></div></details>`,
    footer: `<button class="button button-danger" type="button" data-dialog-action="permission-revoke">Revoke permission</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function activityDialog(item) {
  const lifecycle = item.status === "settling" ? "created → submitted → admitted" : "created → submitted → admitted → confirmed";
  return dialogFrame({
    title: item.title,
    subtitle: "History details",
    body: `
      <div class="review-card review-hero"><span class="list-icon ${item.direction === "in" ? "is-claim" : ""}">${icon(item.direction === "in" ? "receive" : item.direction === "out" ? "send" : "activity")}</span><strong>${escapeHtml(item.amount || statusText(item.status))}</strong><span>${escapeHtml(item.detail)}</span></div>
      <div class="review-card"><div class="summary-row"><span>Status</span><strong><span class="status-badge is-${escapeHtml(item.status)}">${statusText(item.status)}</span></strong></div><div class="summary-row"><span>When</span><strong>${escapeHtml(item.time)}</strong></div><div class="summary-row"><span>Fee</span><strong>${item.type === "money" ? "Included" : "Not applicable"}</strong></div><div class="summary-row"><span>Privacy</span><strong>Target simulation · not live telemetry</strong></div><div class="summary-row"><span>Carrier & chain</span><strong>Reticulum target · Main mock</strong></div></div>
      <details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>ID: ${escapeHtml(item.id)}-b4c9…8e20</span><span>Lifecycle: ${lifecycle}</span><span>Receipt: public_4a92…c71e</span></div></details>`,
    footer: `<button class="button" type="button" data-demo-action="copy-receipt">${icon("copy")} Copy receipt</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function assetDetailDialog() {
  const asset = supportedAsset(state.flow.data.assetKey);
  const rows = [
    ["Asset name", asset.label],
    ["Ticker", asset.ticker],
    ["Owner", asset.owner],
    ["Asset ID", asset.assetId],
    ["Current supply", asset.currentSupply],
    ["Max supply", asset.maxSupply]
  ];
  return dialogFrame({
    title: "Asset details",
    subtitle: `${asset.label} · ${asset.kind}`,
    body: `<div class="asset-detail-table">${rows.map(([label, value]) => `<div class="asset-detail-row"><span>${escapeHtml(label)}</span><strong class="${["Owner", "Asset ID"].includes(label) ? "mono" : ""}">${escapeHtml(value)}</strong></div>`).join("")}</div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>OK</button>`
  });
}

function connectionDialog() {
  return dialogFrame({
    title: "Network & privacy",
    subtitle: "Overlay, carrier, and chain are separate",
    body: `
      <p class="eyebrow">Privacy mode · target simulation</p>
      <div class="connection-options"><div class="connection-option"><span class="health-orb"></span><span><strong>OnionNet</strong><small>Target overlay example · 3 hops</small></span><span class="status-badge is-ready">Target</span></div><div class="connection-option"><span class="health-orb"></span><span><strong>Reticulum</strong><small>Target primary resilient carrier</small></span><span class="status-badge is-ready">Target</span></div><div class="connection-option"><span class="health-orb"></span><span><strong>Tor</strong><small>Current switch method is a placeholder</small></span><span class="status-badge">Stub</span></div></div>
      <p class="eyebrow" style="margin-top:22px">Chain</p>
      <div class="connection-options"><div class="connection-option"><span class="environment-tag is-main">MAIN</span><span><strong>Main</strong><small>Real private value</small></span><span class="status-badge is-settled">In use</span></div><button class="connection-option" type="button" data-demo-action="test-network"><span class="environment-tag is-test">TEST</span><span><strong>Test</strong><small>Test value only · persistent blue label</small></span>${icon("chevron")}</button><button class="connection-option" type="button" data-demo-action="dev-network"><span class="environment-tag is-dev">DEV</span><span><strong>Dev</strong><small>Development value · persistent amber label</small></span>${icon("chevron")}</button></div>
      <div class="capability-note">${icon("alert")} <span><strong>Phase 080 target</strong><small>Current network RPC is stubbed; production must not show these properties until authoritative.</small></span></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function notificationsDialog() {
  return dialogFrame({
    title: "Notifications",
    subtitle: "One item needs attention",
    body: `<div class="attention-list"><button class="attention-item" type="button" data-dialog-action="notification-voucher"><span class="list-icon is-claim">${icon("claim")}</span><span class="list-copy"><strong>Travel refund voucher expires soon</strong><small>Review 86.00 Z00Z from Northwind Travel</small></span>${icon("chevron")}</button><div class="attention-item"><span class="list-icon">${icon("backup")}</span><span class="list-copy"><strong>Backup verified</strong><small>Your 10 Jul local backup passed integrity checks</small></span><span class="status-badge is-settled">Done</span></div></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

const demoSeedWords = [
  "canvas", "orbit", "maple", "velvet", "harbor", "copper", "quiet", "meadow",
  "lamp", "river", "winter", "piano", "forest", "amber", "window", "salt",
  "comet", "paper", "garden", "silver", "cloud", "stone", "echo", "north"
];

function walletsDialog() {
  const selected = activeWallet();
  return dialogFrame({
    title: "Your wallets",
    subtitle: "Local profiles on this device",
    body: `
      <div class="wallet-list">
        ${state.wallets.map((wallet) => `<button class="wallet-choice${wallet.id === selected.id ? " is-current" : ""}" type="button" data-dialog-action="select-wallet" data-wallet-id="${escapeHtml(wallet.id)}">
          <span class="wallet-avatar" aria-hidden="true">${escapeHtml(wallet.initials)}</span><span><strong>${escapeHtml(wallet.name)}</strong><small class="mono">${escapeHtml(wallet.address)} · Main</small></span><span class="status-badge${wallet.id === selected.id ? " is-active" : ""}">${wallet.id === selected.id ? "Open" : "Select"}</span>
        </button>`).join("")}
      </div>
      <div class="notice">${icon("shield")} Wallet profiles are local. Switching never sends a seed or password to another service.</div>`,
    footer: `<button class="button" type="button" data-dialog-action="add-wallet">${icon("plus")} Add wallet</button><button class="button button-primary" type="button" data-dialog-close>Close</button>`
  });
}

function removeWalletDialog() {
  const selectedIds = new Set(state.flow.data.walletIds || []);
  const selectedCount = selectedIds.size;
  const canRemove = selectedCount > 0;
  return dialogFrame({
    title: "Remove wallet profiles",
    subtitle: "Remove local demo profiles from this concept. Wallet files are not deleted.",
    body: `
      <fieldset class="wallet-remove-list" aria-describedby="wallet-remove-summary">
        <legend class="sr-only">Wallet profiles to remove</legend>
        ${state.wallets.map((wallet) => {
          const checked = selectedIds.has(wallet.id);
          return `<label class="wallet-remove-choice${checked ? " is-selected" : ""}">
            <input type="checkbox" data-remove-wallet-id="${escapeHtml(wallet.id)}"${checked ? " checked" : ""}>
            <span class="wallet-avatar" aria-hidden="true">${escapeHtml(wallet.initials)}</span>
            <span class="wallet-remove-copy"><strong>${escapeHtml(wallet.name)}</strong><small class="mono">${escapeHtml(wallet.address)} · ${escapeHtml(wallet.summary.available)} Z00Z</small></span>
          </label>`;
        }).join("")}
      </fieldset>
      <p class="remove-selection-summary" id="wallet-remove-summary">${selectedCount} of ${state.wallets.length} selected. This removes concept profiles only.</p>
      ${selectedCount === state.wallets.length ? `<p class="field-error">All concept profiles will be removed. You can add a wallet again afterward.</p>` : ""}`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-danger" type="button" data-dialog-action="confirm-remove-wallet"${canRemove ? "" : " disabled"}>${icon("remove")} Remove profiles${selectedCount ? ` (${selectedCount})` : ""}</button>`
  });
}

function createWalletDialog() {
  const data = state.flow.data;
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Create wallet",
      subtitle: "A private local wallet",
      steps: 4,
      activeStep: 0,
      body: `
        <form class="form-grid" id="create-wallet-entry" novalidate>
          <div class="field-group"><label class="field-label" for="create-name">Wallet name</label><input id="create-name" name="name" value="${escapeHtml(data.name)}" maxlength="32" placeholder="Everyday wallet" autocomplete="off" required aria-describedby="create-name-error"><p class="field-error" id="create-name-error"></p></div>
          <div class="field-group"><label class="field-label" for="create-password">Wallet password</label><input id="create-password" name="password" type="password" minlength="8" autocomplete="new-password" required aria-describedby="create-password-hint create-password-error"><p class="field-hint" id="create-password-hint">Use at least 8 characters. This concept never stores the value.</p><p class="field-error" id="create-password-error"></p></div>
          <div class="field-group"><label class="field-label" for="create-confirm">Confirm password</label><input id="create-confirm" name="confirm" type="password" minlength="8" autocomplete="new-password" required aria-describedby="create-confirm-error"><p class="field-error" id="create-confirm-error"></p></div>
          <div class="review-card"><div class="summary-row"><span>Chain</span><strong><span class="environment-tag is-main">MAIN</span></strong></div><div class="summary-row"><span>Storage</span><strong>Encrypted on this device</strong></div></div>
        </form>`,
      footer: `<button class="button" type="button" data-dialog-action="create-back-wallets">Back</button><button class="button button-primary" type="submit" form="create-wallet-entry">Create securely</button>`
    });
  }

  if (state.flow.step === 1) {
    return dialogFrame({
      title: "Save your recovery phrase",
      subtitle: "Shown once · demonstration words only",
      steps: 4,
      activeStep: 1,
      closeLabel: "Close and clear recovery phrase",
      body: `
        <div class="confirmation-note">${icon("alert")} Anyone with these 24 words can control the wallet. In production, check your surroundings and keep them offline.</div>
        <ol class="seed-grid" aria-label="Demonstration 24-word recovery phrase">${demoSeedWords.map((word, index) => `<li><span>${index + 1}</span><strong>${word}</strong></li>`).join("")}</ol>
        <p class="seed-demo-label">DEMONSTRATION WORDS · NOT A REAL WALLET SEED</p>
        <button class="button button-full" type="button" data-demo-action="copy-seed-warning">${icon("copy")} Copy requires an extra warning</button>`,
      footer: `<button class="button button-primary" type="button" data-dialog-action="create-seed-saved">I've saved these words</button>`
    });
  }

  if (state.flow.step === 2) {
    return dialogFrame({
      title: "Check your backup",
      subtitle: "Confirm two words before continuing",
      steps: 4,
      activeStep: 2,
      body: `
        <form class="form-grid" id="create-wallet-verify" novalidate>
          <div class="field-group"><label class="field-label" for="seed-word-4">Word 4</label><select id="seed-word-4" name="word4" required><option value="">Choose word</option><option>harbor</option><option>velvet</option><option>meadow</option></select></div>
          <div class="field-group"><label class="field-label" for="seed-word-17">Word 17</label><select id="seed-word-17" name="word17" required><option value="">Choose word</option><option>paper</option><option>comet</option><option>silver</option></select></div>
          <p class="field-error" id="seed-verify-error" role="alert"></p>
        </form>`,
      footer: `<button class="button" type="button" data-dialog-action="create-seed-back">View words again</button><button class="button button-primary" type="submit" form="create-wallet-verify">Finish setup</button>`
    });
  }

  return dialogFrame({
    title: "Wallet ready",
    subtitle: "Recovery check completed",
    steps: 4,
    activeStep: 3,
    body: `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>${escapeHtml(data.name || "New wallet")} is ready</h3><p>The wallet is encrypted on this device. The demonstration phrase has been cleared from the view.</p></div><div class="review-card"><div class="summary-row"><span>Network</span><strong>Main</strong></div><div class="summary-row"><span>Backup check</span><strong class="trust-label">${icon("shield")} Completed</strong></div></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-action="create-finish">Open wallet</button>`
  });
}

function recoverWalletDialog() {
  const data = state.flow.data;
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Recover wallet",
      subtitle: "Enter the same 24 English words twice",
      steps: 2,
      activeStep: 0,
      body: `
        <div class="confirmation-note">${icon("shield")} Recovery is validated locally. Never enter your words into a website or support chat.</div>
        <form class="form-grid" id="recover-wallet-entry" novalidate>
          <div class="field-group"><label class="field-label" for="recover-name">Wallet name</label><input id="recover-name" name="name" value="${escapeHtml(data.name || "Recovered wallet")}" maxlength="32" placeholder="Recovered wallet" autocomplete="off" required aria-describedby="recover-name-error"><p class="field-error" id="recover-name-error"></p></div>
          <div class="field-group"><label class="field-label" for="recover-phrase-a">Recovery phrase</label><textarea class="seed-entry" id="recover-phrase-a" name="phraseA" rows="4" autocomplete="off" autocapitalize="none" spellcheck="false" placeholder="Enter 24 English words" required></textarea><p class="field-hint">0 of 24 words</p></div>
          <div class="field-group"><label class="field-label" for="recover-phrase-b">Enter it again</label><textarea class="seed-entry" id="recover-phrase-b" name="phraseB" rows="4" autocomplete="off" autocapitalize="none" spellcheck="false" placeholder="Repeat the same 24 words" required></textarea><p class="field-hint">0 of 24 words</p></div>
          <p class="field-error" id="recover-phrase-error" role="alert"></p>
          <button class="text-button" type="button" data-demo-action="fill-demo-seed">Fill demonstration words</button>
        </form>`,
      footer: `<button class="button" type="button" data-dialog-action="recover-back-wallets">Back</button><button class="button button-primary" type="submit" form="recover-wallet-entry">Validate and recover</button>`
    });
  }

  return dialogFrame({
    title: "Wallet recovered",
    subtitle: "Updating private wallet state",
    steps: 2,
    activeStep: 1,
    body: `<div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Recovery complete · scanning</h3><p>Your keys are available locally. Money and history will appear as the wallet scan catches up.</p></div><div class="review-card"><div class="summary-row"><span>Wallet scan</span><strong>42%</strong></div><div class="progress-track"><div class="progress-bar" style="width:42%"></div></div><div class="summary-row"><span>Safe to close</span><strong>Yes · resumes automatically</strong></div></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-action="recover-finish">Open wallet</button>`
  });
}

function openWalletDialog() {
  const data = state.flow.data;
  return dialogFrame({
    title: "Open existing wallet",
    subtitle: "Add a local encrypted wallet profile",
    body: `
      <form class="form-grid" id="open-wallet-entry" novalidate>
        <div class="field-group"><label class="field-label" for="open-wallet-name">Wallet name</label><input id="open-wallet-name" name="name" value="${escapeHtml(data.name || "Existing wallet")}" maxlength="32" placeholder="Existing wallet" autocomplete="off" required aria-describedby="open-wallet-error"><p class="field-error" id="open-wallet-error" role="alert"></p></div>
        <div class="review-card"><div class="summary-row"><span>Storage</span><strong>Encrypted local profile</strong></div><div class="summary-row"><span>After opening</span><strong>Wallet scan begins</strong></div></div>
        <div class="notice">${icon("shield")} This concept does not ask for, access, or upload a wallet file path.</div>
      </form>`,
    footer: `<button class="button" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="open-wallet-entry">Open wallet</button>`
  });
}

function renderDialog() {
  if (!state.flow) return;
  const type = state.flow.type;
  const content = type === "pay" ? payDialog()
    : type === "receive" ? receiveDialog()
    : type === "asset-claim" ? assetClaimDialog()
    : type === "voucher-review" ? voucherDialog(false)
    : type === "voucher-settled" ? voucherDialog(true)
    : type === "permission" ? permissionDialog()
    : type === "permission-detail" ? permissionDetailDialog()
    : type === "activity" ? activityDialog(state.flow.data.item)
    : type === "asset-detail" ? assetDetailDialog()
    : type === "connection" ? connectionDialog()
    : type === "wallets" ? walletsDialog()
    : type === "remove-wallet" ? removeWalletDialog()
    : type === "create-wallet" ? createWalletDialog()
    : type === "open-wallet" ? openWalletDialog()
    : type === "recover-wallet" ? recoverWalletDialog()
    : ["wallet-rename", "wallet-seed-reveal", "wallet-public-export", "wallet-key-rotation", "wallet-policy-apply", "wallet-policy-profile"].includes(type) ? sensitiveWalletDialog(type)
    : notificationsDialog();
  dialogContent.innerHTML = content;
}

function defaultFlowData(type) {
  if (type === "pay") return { recipient: "", recipientLabel: "", amount: "", memo: "", assetKey: "z00z" };
  if (type === "receive") return { amount: "", note: "", assetKey: "z00z" };
  if (type === "permission") return { delegate: "", action: "Deploy release", scope: "staging.example", uses: "1", expiry: "2026-08-19", expiryLabel: "19 Aug 2026" };
  if (type === "create-wallet") return { name: "" };
  if (type === "open-wallet") return { name: "Existing wallet" };
  if (type === "recover-wallet") return { name: "Recovered wallet" };
  if (type === "remove-wallet") return { walletIds: [] };
  return {};
}

function openFlow(type, trigger = document.activeElement, extraData = {}) {
  state.lastDialogTrigger = trigger;
  state.flow = { type, step: 0, data: { ...defaultFlowData(type), ...extraData } };
  renderDialog();
  if (!dialog.open) dialog.showModal();
  requestAnimationFrame(() => {
    const target = dialog.querySelector("input:not([type='hidden']), select, button:not([data-dialog-close])");
    target?.focus();
  });
}

function closeDialog() {
  if (dialog.open) dialog.close();
}

function showToast(message, iconName = "check") {
  const region = document.querySelector("#toast-region");
  const toast = document.createElement("div");
  toast.className = "toast";
  toast.innerHTML = `${icon(iconName)}<span>${escapeHtml(message)}</span><button type="button" aria-label="Dismiss notification">${icon("close")}</button>`;
  toast.querySelector("button").addEventListener("click", () => toast.remove());
  region.append(toast);
  window.setTimeout(() => toast.remove(), 4200);
}

function qrCells() {
  const fixed = new Set();
  function square(startX, startY) {
    for (let y = 0; y < 5; y += 1) {
      for (let x = 0; x < 5; x += 1) {
        if (x === 0 || y === 0 || x === 4 || y === 4 || (x >= 2 && x <= 2 && y >= 2 && y <= 2)) fixed.add((startY + y) * 13 + startX + x);
      }
    }
  }
  square(0, 0); square(8, 0); square(0, 8);
  return Array.from({ length: 169 }, (_, index) => {
    const pseudo = ((index * 17 + Math.floor(index / 13) * 11 + 7) % 9) < 4;
    return `<span class="${fixed.has(index) || pseudo ? "is-dark" : ""}"></span>`;
  }).join("");
}

function validatePay(form) {
  const recipient = form.elements.recipient;
  const amount = form.elements.amount;
  const asset = supportedAsset(form.elements.assetKey.value);
  let valid = true;
  document.querySelector("#pay-recipient-error").textContent = "";
  document.querySelector("#pay-amount-error").textContent = "";
  recipient.removeAttribute("aria-invalid");
  amount.removeAttribute("aria-invalid");

  if (recipient.value.trim().length < 3) {
    document.querySelector("#pay-recipient-error").textContent = "Enter or scan a valid recipient request.";
    recipient.setAttribute("aria-invalid", "true");
    valid = false;
  }
  const number = Number(amount.value);
  if (!Number.isFinite(number) || number <= 0 || number > Number(asset.balance.replaceAll(",", "")) || (!asset.divisible && !Number.isInteger(number))) {
    const minimum = asset.divisible ? "0.01" : "1";
    document.querySelector("#pay-amount-error").textContent = `Enter ${asset.divisible ? "an amount" : "a whole unit"} between ${minimum} and ${asset.balance} ${asset.unit}.`;
    amount.setAttribute("aria-invalid", "true");
    valid = false;
  }
  if (!valid) {
    form.querySelector('[aria-invalid="true"]')?.focus();
    return;
  }

  state.flow.data = {
    ...state.flow.data,
    recipient: recipient.value.trim(),
    recipientLabel: recipient.value.trim().startsWith("z00z:") ? "Verified asset request" : recipient.value.trim(),
    amount: asset.divisible ? number.toFixed(2) : String(number),
    memo: form.elements.memo.value.trim(),
    assetKey: asset.key
  };
  state.flow.step = 1;
  renderDialog();
}

function validatePermission(form) {
  const delegate = form.elements.delegate;
  const uses = form.elements.uses;
  let valid = true;
  document.querySelector("#permission-delegate-error").textContent = "";
  document.querySelector("#permission-uses-error").textContent = "";
  delegate.removeAttribute("aria-invalid");
  uses.removeAttribute("aria-invalid");

  if (delegate.value.trim().length < 3) {
    document.querySelector("#permission-delegate-error").textContent = "Choose a verified service or known person.";
    delegate.setAttribute("aria-invalid", "true");
    valid = false;
  }
  const useCount = Number(uses.value);
  if (!Number.isInteger(useCount) || useCount < 1 || useCount > 5) {
    document.querySelector("#permission-uses-error").textContent = "Choose between 1 and 5 uses, within the held authority.";
    uses.setAttribute("aria-invalid", "true");
    valid = false;
  }
  if (!valid) {
    form.querySelector('[aria-invalid="true"]')?.focus();
    return;
  }

  const expiry = new Date(`${form.elements.expiry.value}T12:00:00`);
  state.flow.data = {
    delegate: delegate.value.trim(),
    action: form.elements.action.value,
    scope: form.elements.scope.value,
    uses: String(useCount),
    expiry: form.elements.expiry.value,
    expiryLabel: new Intl.DateTimeFormat("en", { day: "2-digit", month: "short", year: "numeric" }).format(expiry)
  };
  state.flow.step = 1;
  renderDialog();
}

function validateWalletSettingsAction(form) {
  const type = state.flow?.type;
  const error = form.querySelector(".field-error");
  const password = form.elements.password;
  if (error) error.textContent = "";
  password?.removeAttribute("aria-invalid");
  if (!password || password.value.length < 8) {
    if (error) error.textContent = "Enter at least 8 characters for this concept password check.";
    password?.setAttribute("aria-invalid", "true");
    password?.focus();
    return;
  }

  const requiredConfirmation = {
    "wallet-seed-reveal": "SHOW SEED",
    "wallet-key-rotation": "ROTATE",
    "wallet-policy-apply": "APPLY"
  }[type];
  if (requiredConfirmation && form.elements.confirmation?.value.trim() !== requiredConfirmation) {
    if (error) error.textContent = `Type ${requiredConfirmation} to continue.`;
    form.elements.confirmation?.setAttribute("aria-invalid", "true");
    form.elements.confirmation?.focus();
    return;
  }

  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  if (type === "wallet-rename") {
    const name = form.elements.name.value.trim();
    if (name.length < 2 || name.length > 32) {
      if (error) error.textContent = "Wallet name must contain 2–32 characters.";
      form.elements.name.setAttribute("aria-invalid", "true");
      form.elements.name.focus();
      return;
    }
    wallet.name = name;
    wallet.initials = name.slice(0, 1).toUpperCase();
  }
  if (type === "wallet-policy-apply") {
    const maxTransaction = form.elements.maxTransaction.value.trim();
    const maxDaily = form.elements.maxDaily.value.trim();
    if (!/^\d+(?:\.\d+)?$/.test(maxTransaction) || !/^\d+(?:\.\d+)?$/.test(maxDaily)) {
      if (error) error.textContent = "Spend limits must be non-negative decimals.";
      (!/^\d+(?:\.\d+)?$/.test(maxTransaction) ? form.elements.maxTransaction : form.elements.maxDaily).focus();
      return;
    }
    preferences.policyRules = {
      maxTransaction,
      maxDaily,
      requireConfirmation: form.elements.requireConfirmation.checked,
      allowedAssets: form.elements.allowedAssets.value,
      allowedRecipients: form.elements.allowedRecipients.value.trim(),
      timeWindow: form.elements.timeWindow.value
    };
  }
  if (type === "wallet-key-rotation") preferences.lastMasterKeyRotation = "Just now · concept";
  password.value = "";
  if (form.elements.confirmation) form.elements.confirmation.value = "";
  state.walletSettingsConfigDraft = "";
  syncConfigDraftFromState();
  state.flow.step = 1;
  render();
  renderDialog();
}

function setButtonLoading(button, label) {
  button.disabled = true;
  button.dataset.original = button.innerHTML;
  button.textContent = label;
}

function completePay() {
  const data = state.flow.data;
  const wallet = activeWallet();
  const asset = flowAsset(data);
  wallet.activities.unshift({ id: `tx-${wallet.activities.length + 1}`, type: asset.key === "z00z" ? "money" : "asset", direction: "out", title: `${asset.label} sent`, detail: `Sent to ${data.recipientLabel} · waiting to settle`, amount: `− ${data.amount} ${asset.unit}`, time: "Now", status: "settling" });
  state.flow.step = 2;
  renderDialog();
}

function handleDialogAction(action, button) {
  if (action === "pay-back") {
    state.flow.step = 0;
    renderDialog();
  } else if (action === "pay-submit") {
    setButtonLoading(button, "Sending once…");
    window.setTimeout(completePay, 650);
  } else if (action === "permission-back") {
    state.flow.step = 0;
    renderDialog();
  } else if (action === "permission-submit") {
    setButtonLoading(button, "Delegating…");
    window.setTimeout(() => { state.flow.step = 2; renderDialog(); }, 650);
  } else if (action === "asset-claim-submit") {
    setButtonLoading(button, "Verifying once…");
    window.setTimeout(() => { state.flow.step = 1; renderDialog(); }, 600);
  } else if (action === "voucher-accept") {
    setButtonLoading(button, "Accepting voucher…");
    window.setTimeout(() => { state.flow.step = 1; renderDialog(); }, 600);
  } else if (action === "voucher-redeem") {
    setButtonLoading(button, "Redeeming…");
    window.setTimeout(() => { state.flow.step = 2; renderDialog(); }, 600);
  } else if (action === "voucher-reject") {
    showToast("Rejecting a voucher requires a separate consequence confirmation.", "alert");
  } else if (action === "permission-revoke") {
    showToast("Revocation requires re-authentication and consequence review.", "alert");
  } else if (action === "view-activity") {
    closeDialog();
    state.view = "activity";
    state.activityFilter = "all";
    render({ focusMain: true });
  } else if (action === "go-actions") {
    closeDialog();
    state.view = "wallet";
    state.walletSection = "permissions";
    render({ focusMain: true });
  } else if (action === "notification-voucher") {
    closeDialog();
    window.setTimeout(() => openFlow("voucher-review", button), 0);
  } else if (action === "select-wallet") {
    closeDialog();
    state.selectedWalletId = button.dataset.walletId;
    state.view = "wallet";
    state.activityFilter = "all";
    render({ focusMain: true });
    showToast(`${activeWallet().name} wallet opened in concept mode.`);
  } else if (action === "confirm-remove-wallet") {
    const selectedIds = new Set(state.flow?.data.walletIds || []);
    const walletsToRemove = state.wallets.filter((wallet) => selectedIds.has(wallet.id));
    if (walletsToRemove.length === 0) {
      showToast("Select one or more wallets to remove.", "alert");
      return;
    }
    const selectedIndex = state.wallets.findIndex((wallet) => wallet.id === state.selectedWalletId);
    const remainingWallets = state.wallets.filter((wallet) => !selectedIds.has(wallet.id));
    state.wallets = remainingWallets;
    if (remainingWallets.length === 0) {
      state.selectedWalletId = null;
      state.view = "add-wallet";
    } else if (selectedIds.has(state.selectedWalletId)) {
      state.selectedWalletId = remainingWallets[Math.min(selectedIndex, remainingWallets.length - 1)].id;
      state.view = "wallet";
    } else {
      state.view = "wallet";
    }
    state.activityFilter = "all";
    closeDialog();
    render({ focusMain: true });
    showToast(remainingWallets.length === 0 ? "All wallet profiles removed. Add a wallet to continue." : `${walletsToRemove.length} wallet${walletsToRemove.length === 1 ? "" : "s"} removed from this concept.`);
  } else if (action === "add-wallet") {
    closeDialog();
    state.view = "add-wallet";
    render({ focusMain: true });
  } else if (["start-create", "start-recover"].includes(action)) {
    closeDialog();
    state.view = "add-wallet";
    render({ focusMain: true });
  } else if (["create-back-wallets", "recover-back-wallets"].includes(action)) {
    closeDialog();
    state.view = "add-wallet";
    render({ focusMain: true });
  } else if (action === "create-seed-saved") {
    state.flow.step = 2;
    renderDialog();
  } else if (action === "create-seed-back") {
    state.flow.step = 1;
    renderDialog();
  } else if (action === "create-finish" || action === "recover-finish") {
    const recovered = action === "recover-finish";
    const wallet = addWalletProfile(
      state.flow.data.name || (recovered ? "Recovered wallet" : "New wallet"),
      "Scanning"
    );
    state.selectedWalletId = wallet.id;
    state.view = "wallet";
    state.activityFilter = "all";
    closeDialog();
    if (state.locked) {
      state.locked = false;
      lockScreen.hidden = true;
      appShell.hidden = false;
      appShell.inert = false;
    }
    render();
    showToast(recovered ? "Recovered wallet opened; scan continues." : "New wallet opened in concept mode.");
  }
}

function handleDemoAction(action, button) {
  if (action === "toggle-balance") {
    state.balanceHidden = !state.balanceHidden;
    syncConfigDraftFromState();
    render();
    showToast(state.balanceHidden ? "Sensitive amounts hidden." : "Sensitive amounts visible.");
  } else if (["lock", "logout"].includes(action)) {
    closeDialog();
    state.locked = true;
    appShell.hidden = true;
    appShell.inert = true;
    lockScreen.hidden = false;
    document.querySelector("#unlock-password").value = "";
    document.querySelector("#unlock-error").textContent = "";
    document.querySelector("#unlock-password").focus();
    if (action === "logout") showToast("Wallet session ended.");
  } else if (action === "add-wallet") {
    state.view = "add-wallet";
    render({ focusMain: true });
  } else if (action === "remove-wallet") {
    if (state.wallets.length === 0) {
      state.view = "add-wallet";
      render({ focusMain: true });
      return;
    }
    openFlow("remove-wallet", button);
  } else if (action === "create-wallet") {
    openFlow("create-wallet", button);
  } else if (action === "open-existing-wallet") {
    openFlow("open-wallet", button);
  } else if (action === "restore-wallet") {
    openFlow("recover-wallet", button);
  } else if (action === "switch-wallet") {
    openFlow("wallets", button);
  } else if (action === "notifications") {
    openFlow("notifications", button);
  } else if (["copy-request", "copy-receipt", "copy-wallet-address"].includes(action)) {
    const messages = {
      "copy-request": "Asset request copied.",
      "copy-receipt": "Public receipt copied.",
      "copy-wallet-address": "Wallet address copied."
    };
    showToast(messages[action]);
  } else if (action === "share-request") {
    showToast("Native share sheet would open on this device.");
  } else if (action === "wallet-auto-backup") {
    const preferences = activeWalletPreferences();
    preferences.autoBackup = !preferences.autoBackup;
    state.walletSettingsConfigDraft = "";
    syncConfigDraftFromState();
    render();
    showToast(`Automatic backup ${preferences.autoBackup ? "enabled" : "disabled"} for ${activeWallet().name}.`);
  } else if (action === "wallet-config-validate") {
    const source = document.querySelector("#wallet-settings-yaml")?.value ?? state.walletSettingsConfigDraft;
    state.walletSettingsConfigDraft = source;
    const result = validateAndApplyWalletSettingsYaml(source);
    state.configStatus = result.message;
    render();
    showToast(result.message, result.valid ? "check" : "alert");
  } else if (action === "wallet-config-apply") {
    const source = document.querySelector("#wallet-settings-yaml")?.value ?? state.walletSettingsConfigDraft;
    state.walletSettingsConfigDraft = source;
    const result = validateAndApplyWalletSettingsYaml(source, true);
    state.configStatus = result.message;
    if (result.valid) state.walletSettingsConfigDraft = "";
    render();
    showToast(result.message, result.valid ? "check" : "alert");
  } else if (action === "seed-warning") {
    showToast("Seed reveal requires re-authentication and a private display check.", "alert");
  } else if (action === "key-rotation") {
    showToast("Key rotation requires re-authentication and a fresh backup.", "alert");
  } else if (action === "backup") {
    showToast("Backup destination selection would open next.");
  } else if (action === "restore") {
    showToast("Restore validates integrity before any replacement.", "alert");
  } else if (action === "preview-swap") {
    showToast(`${activeWallet().name} wallet needs a verified quote before a swap can be reviewed.`);
  } else if (action === "request-exchange-quote") {
    showToast("An exchange quote requires a verified provider and an authoritative route.");
  } else if (action === "prepare-stake") {
    showToast(`${activeWallet().name} wallet needs validator and lock-up terms before staking can be reviewed.`);
  } else if (action === "asset-review") {
    showToast("Declared domain and metadata are not the same as an authoritative trust verdict.", "alert");
  } else if (action === "general-notifications") {
    state.notifications = !state.notifications;
    syncConfigDraftFromState();
    render();
    showToast(`Notifications ${state.notifications ? "enabled" : "disabled"}.`);
  } else if (action === "motion") {
    state.reducedMotion = !state.reducedMotion;
    syncConfigDraftFromState();
    render();
    showToast(`Reduced motion ${state.reducedMotion ? "enabled" : "disabled"}.`);
  } else if (action === "compact") {
    state.compactLists = !state.compactLists;
    syncConfigDraftFromState();
    render();
    showToast(`Compact desktop lists ${state.compactLists ? "enabled" : "disabled"}.`);
  } else if (action === "expert") {
    state.expertDetails = !state.expertDetails;
    syncConfigDraftFromState();
    render();
    showToast(`Expert details ${state.expertDetails ? "enabled" : "disabled"}.`);
  } else if (action === "diagnostics") {
    showToast("Diagnostics would open sanitized RPC and route records.");
  } else if (action === "load-policy") {
    showToast("Profile would be parsed, signature-checked, capability-checked, and previewed before Apply.");
  } else if (action === "why-blocked") {
    showToast("Target preview: Personal Safe v1.4 would block this above its 2,500 Z00Z maximum.", "alert");
  } else if (action === "config-validate") {
    const source = document.querySelector("#config-yaml")?.value ?? state.configDraft;
    state.configDraft = source;
    const result = validateAndApplyDemoConfig(source);
    state.configStatus = result.message;
    render();
    showToast(result.message, result.valid ? "check" : "alert");
  } else if (action === "config-apply") {
    const source = document.querySelector("#config-yaml")?.value ?? state.configDraft;
    state.configDraft = source;
    const result = validateAndApplyDemoConfig(source, true);
    state.configStatus = result.message;
    if (result.valid) syncConfigDraftFromState();
    render();
    showToast(result.message, result.valid ? "check" : "alert");
  } else if (action === "config-stage") {
    showToast("Reticulum interface changes would be staged in YAML; restart required.");
  } else if (action === "rebuild-route") {
    showToast("OnionNet would build and verify a new route before cutover.");
  } else if (action === "route-onion") {
    showToast("Route switch requires a live connectivity check.");
  } else if (["test-network", "dev-network"].includes(action)) {
    showToast("Chain switch requires confirmation and persistent environment labeling.", "alert");
  } else if (action === "copy-seed-warning") {
    showToast("Production copy requires a second warning and timed clipboard clearing.", "alert");
  } else if (action === "fill-demo-seed") {
    const phrase = demoSeedWords.join(" ");
    const first = document.querySelector("#recover-phrase-a");
    const second = document.querySelector("#recover-phrase-b");
    if (first && second) {
      first.value = phrase;
      second.value = phrase;
      first.dispatchEvent(new Event("input", { bubbles: true }));
      second.dispatchEvent(new Event("input", { bubbles: true }));
      showToast("Demonstration words filled; they are not a real seed.");
    }
  }
}

document.addEventListener("click", (event) => {
  const viewButton = event.target.closest("[data-view]");
  if (viewButton) {
    const view = viewButton.dataset.view;
    if (view === "settings" && viewButton.closest(".system-nav")) {
      state.settingsSection = "general";
      state.networkSection = "overview";
      state.isNetworkOpen = false;
    }
    state.view = view;
    render({ focusMain: true });
    return;
  }

  const walletSectionButton = event.target.closest("[data-wallet-section]");
  if (walletSectionButton) {
    state.view = "wallet";
    state.walletSection = walletSectionButton.dataset.walletSection;
    render({ focusMain: true });
    return;
  }

  const walletSettingsSectionButton = event.target.closest("[data-wallet-settings-section]");
  if (walletSettingsSectionButton) {
    state.view = "wallet-settings";
    state.walletSettingsSection = walletSettingsSectionButton.dataset.walletSettingsSection;
    render({ focusMain: true });
    return;
  }

  const flowButton = event.target.closest("[data-open-flow]");
  if (flowButton) {
    openFlow(flowButton.dataset.openFlow, flowButton, flowButton.dataset.assetKey ? { assetKey: flowButton.dataset.assetKey } : {});
    return;
  }

  const activityButton = event.target.closest("[data-open-activity]");
  if (activityButton) {
    const item = activeWallet().activities.find((entry) => entry.id === activityButton.dataset.openActivity);
    if (item) openFlow("activity", activityButton, { item });
    return;
  }

  const walletButton = event.target.closest("[data-wallet-id]");
  if (walletButton && !walletButton.dataset.dialogAction) {
    state.selectedWalletId = walletButton.dataset.walletId;
    state.view = "wallet";
    state.activityFilter = "all";
    render({ focusMain: true });
    return;
  }

  const filterButton = event.target.closest("[data-filter]");
  if (filterButton) {
    state.activityFilter = filterButton.dataset.filter;
    render();
    return;
  }

  const settingButton = event.target.closest("[data-settings-section]");
  if (settingButton) {
    const section = settingButton.dataset.settingsSection;
    if (section === "network") {
      if (state.settingsSection === "network" && state.isNetworkOpen) {
        state.isNetworkOpen = false;
        state.networkSection = "overview";
      } else {
        state.isNetworkOpen = true;
        state.networkSection = "overview";
      }
      state.settingsSection = "network";
    } else {
      state.settingsSection = section;
      state.isNetworkOpen = false;
    }
    render();
    return;
  }

  const networkButton = event.target.closest("[data-network-section]");
  if (networkButton) {
    state.view = "settings";
    state.settingsSection = "network";
    state.networkSection = networkButton.dataset.networkSection;
    state.isNetworkOpen = true;
    render();
    return;
  }

  const themeButton = event.target.closest("[data-theme]");
  if (themeButton && themeButton.tagName === "BUTTON") {
    state.theme = themeButton.dataset.theme;
    syncConfigDraftFromState();
    applyAppearancePreferences();
    render();
    showToast(`${state.theme === "system" ? "System" : state.theme === "dark" ? "Dark" : "Light"} theme applied locally.`);
    return;
  }

  const paletteButton = event.target.closest("[data-palette]");
  if (paletteButton && paletteButton.tagName === "BUTTON") {
    state.palette = paletteButton.dataset.palette;
    state.hasCustomAppearance = false;
    syncConfigDraftFromState();
    applyAppearancePreferences();
    render();
    showToast(`${paletteOptions.find((palette) => palette.id === state.palette)?.label || "Palette"} applied locally.`);
    return;
  }

  const codeThemeButton = event.target.closest("[data-code-theme]");
  if (codeThemeButton && codeThemeButton.tagName === "BUTTON") {
    state.codeTheme = codeThemeButton.dataset.codeTheme;
    syncConfigDraftFromState();
    applyAppearancePreferences();
    render();
    showToast(`${codeThemeOptions.find((theme) => theme.id === state.codeTheme)?.label || "Code"} highlighting applied locally.`);
    return;
  }

  const configViewButton = event.target.closest("[data-config-view]");
  if (configViewButton) {
    state.configView = configViewButton.dataset.configView;
    render();
    return;
  }

  const closeButton = event.target.closest("[data-dialog-close]");
  if (closeButton) {
    closeDialog();
    return;
  }

  const dialogAction = event.target.closest("[data-dialog-action]");
  if (dialogAction) {
    handleDialogAction(dialogAction.dataset.dialogAction, dialogAction);
    return;
  }

  const demoAction = event.target.closest("[data-demo-action]");
  if (demoAction) handleDemoAction(demoAction.dataset.demoAction, demoAction);
});

document.addEventListener("submit", (event) => {
  event.preventDefault();
  if (["wallet-rename-entry", "wallet-seed-reveal-entry", "wallet-public-export-entry", "wallet-key-rotation-entry", "wallet-policy-apply-entry"].includes(event.target.id)) {
    validateWalletSettingsAction(event.target);
  } else if (event.target.id === "pay-entry") {
    validatePay(event.target);
  } else if (event.target.id === "receive-entry") {
    const asset = supportedAsset(event.target.elements.assetKey.value);
    state.flow.data = {
      ...state.flow.data,
      assetKey: asset.key,
      amount: event.target.elements.amount.value ? (asset.divisible ? Number(event.target.elements.amount.value).toFixed(2) : String(Number(event.target.elements.amount.value))) : "",
      note: event.target.elements.note.value.trim()
    };
    state.flow.step = 1;
    renderDialog();
  } else if (event.target.id === "permission-entry") {
    validatePermission(event.target);
  } else if (event.target.id === "create-wallet-entry") {
    const name = event.target.elements.name;
    const password = event.target.elements.password;
    const confirm = event.target.elements.confirm;
    let valid = true;
    document.querySelector("#create-name-error").textContent = "";
    document.querySelector("#create-password-error").textContent = "";
    document.querySelector("#create-confirm-error").textContent = "";
    [name, password, confirm].forEach((field) => field.removeAttribute("aria-invalid"));
    if (name.value.trim().length < 2) {
      document.querySelector("#create-name-error").textContent = "Enter a recognizable wallet name.";
      name.setAttribute("aria-invalid", "true");
      valid = false;
    }
    if (password.value.length < 8) {
      document.querySelector("#create-password-error").textContent = "Use at least 8 characters.";
      password.setAttribute("aria-invalid", "true");
      valid = false;
    }
    if (confirm.value !== password.value) {
      document.querySelector("#create-confirm-error").textContent = "Passwords do not match.";
      confirm.setAttribute("aria-invalid", "true");
      valid = false;
    }
    if (!valid) {
      event.target.querySelector('[aria-invalid="true"]')?.focus();
      return;
    }
    state.flow.data.name = name.value.trim();
    state.flow.step = 1;
    renderDialog();
  } else if (event.target.id === "create-wallet-verify") {
    const correct = event.target.elements.word4.value === "velvet" && event.target.elements.word17.value === "comet";
    if (!correct) {
      document.querySelector("#seed-verify-error").textContent = "Choose the words shown at positions 4 and 17.";
      event.target.elements.word4.focus();
      return;
    }
    state.flow.step = 3;
    renderDialog();
  } else if (event.target.id === "open-wallet-entry") {
    const name = event.target.elements.name;
    const error = document.querySelector("#open-wallet-error");
    error.textContent = "";
    name.removeAttribute("aria-invalid");
    if (name.value.trim().length < 2) {
      error.textContent = "Enter a recognizable wallet name.";
      name.setAttribute("aria-invalid", "true");
      name.focus();
      return;
    }
    const wallet = addWalletProfile(name.value.trim(), "Scanning");
    state.selectedWalletId = wallet.id;
    state.view = "wallet";
    state.activityFilter = "all";
    closeDialog();
    render({ focusMain: true });
    showToast("Existing wallet opened; scan continues.");
  } else if (event.target.id === "recover-wallet-entry") {
    const name = event.target.elements.name;
    const phraseA = event.target.elements.phraseA.value.trim().split(/\s+/).filter(Boolean);
    const phraseB = event.target.elements.phraseB.value.trim().split(/\s+/).filter(Boolean);
    const error = document.querySelector("#recover-phrase-error");
    const nameError = document.querySelector("#recover-name-error");
    error.textContent = "";
    nameError.textContent = "";
    name.removeAttribute("aria-invalid");
    if (name.value.trim().length < 2) {
      nameError.textContent = "Enter a recognizable wallet name.";
      name.setAttribute("aria-invalid", "true");
      name.focus();
      return;
    }
    if (phraseA.length !== 24 || phraseB.length !== 24) {
      error.textContent = "Both entries must contain exactly 24 words.";
      event.target.elements.phraseA.focus();
      return;
    }
    if (phraseA.join(" ") !== phraseB.join(" ")) {
      error.textContent = "The two recovery phrase entries do not match.";
      event.target.elements.phraseB.focus();
      return;
    }
    event.target.elements.phraseA.value = "";
    event.target.elements.phraseB.value = "";
    state.flow.data.name = name.value.trim();
    state.flow.step = 1;
    renderDialog();
  } else if (event.target.id === "unlock-form") {
    const input = document.querySelector("#unlock-password");
    if (input.value.length < 4) {
      document.querySelector("#unlock-error").textContent = "Enter at least four characters for this concept.";
      input.setAttribute("aria-invalid", "true");
      input.focus();
      return;
    }
    input.removeAttribute("aria-invalid");
    input.value = "";
    state.locked = false;
    lockScreen.hidden = true;
    appShell.hidden = false;
    appShell.inert = false;
    render();
    document.querySelector('[data-demo-action="lock"]')?.focus();
    showToast("Wallet unlocked for this concept.");
  }
});

document.addEventListener("input", (event) => {
  if (event.target.id === "activity-search") {
    const term = event.target.value.trim().toLowerCase();
    const items = activeWallet().activities.filter((item) => {
      const matchesFilter = matchesActivityFilter(item, state.activityFilter);
      return matchesFilter && `${item.title} ${item.detail} ${item.id}`.toLowerCase().includes(term);
    });
    document.querySelector("#activity-results").innerHTML = activityRows(items);
  } else if (event.target.classList.contains("seed-entry")) {
    const count = event.target.value.trim() ? event.target.value.trim().split(/\s+/).length : 0;
    const hint = event.target.closest(".field-group")?.querySelector(".field-hint");
    if (hint) hint.textContent = `${count} of 24 words`;
  } else if (event.target.id === "config-yaml") {
    state.configDraft = event.target.value;
    const result = validateAndApplyDemoConfig(state.configDraft);
    state.configStatus = result.message;
    event.target.setAttribute("aria-invalid", String(!result.valid));
    syncYamlHighlight(event.target);
  } else if (event.target.id === "wallet-settings-yaml") {
    state.walletSettingsConfigDraft = event.target.value;
    const result = validateAndApplyWalletSettingsYaml(state.walletSettingsConfigDraft);
    state.configStatus = result.message;
    event.target.setAttribute("aria-invalid", String(!result.valid));
    syncYamlHighlight(event.target);
  }
});

document.addEventListener("scroll", (event) => {
  if (event.target instanceof HTMLTextAreaElement && event.target.classList.contains("yaml-editor")) syncYamlHighlight(event.target);
}, true);

document.addEventListener("change", (event) => {
  if (event.target.matches("[data-remove-wallet-id]")) {
    const walletId = event.target.dataset.removeWalletId;
    const selectedIds = new Set(state.flow?.data.walletIds || []);
    if (event.target.checked) selectedIds.add(walletId);
    else selectedIds.delete(walletId);
    state.flow.data.walletIds = [...selectedIds];
    renderDialog();
    document.querySelector(`[data-remove-wallet-id="${walletId}"]`)?.focus();
    return;
  }
  const walletSettingsControl = event.target.dataset.walletSettingsControl;
  if (walletSettingsControl) {
    const preferences = activeWalletPreferences();
    if (walletSettingsControl === "currency") preferences.currency = event.target.value;
    if (walletSettingsControl === "default-fee") {
      if (!/^\d+(?:\.\d+)?$/.test(event.target.value.trim())) {
        showToast("Default fee must be a non-negative decimal.", "alert");
        render();
        return;
      }
      preferences.defaultFee = event.target.value.trim();
    }
    if (walletSettingsControl === "lock-after") preferences.lockAfterMinutes = event.target.value;
    if (walletSettingsControl === "backup-interval") preferences.backupIntervalHours = event.target.value;
    state.walletSettingsConfigDraft = "";
    syncConfigDraftFromState();
    render();
    showToast(`${activeWallet().name} wallet setting updated locally.`);
    return;
  }
  const configControl = event.target.dataset.configControl;
  if (configControl) {
    if (configControl === "language") state.language = event.target.value;
    if (configControl === "palette") state.palette = event.target.value;
    if (configControl === "code-theme") state.codeTheme = event.target.value;
    if (configControl === "text-scale") state.textScale = event.target.value;
    if (configControl === "lock-after") {
      state.autoLockMinutes = event.target.value;
      activeWalletPreferences().lockAfterMinutes = event.target.value;
    }
    if (configControl === "default-fee") activeWalletPreferences().defaultFee = event.target.value.trim();
    if (["custom-brand", "custom-rail"].includes(configControl)) {
      if (!hasSafeControlContrast(event.target.value)) {
        showToast("Choose a colour with at least 3:1 contrast against the current canvas.", "alert");
        render();
        return;
      }
      state.customAppearance[configControl === "custom-brand" ? "brand" : "rail"] = event.target.value;
      state.hasCustomAppearance = true;
    }
    syncConfigDraftFromState();
    applyAppearancePreferences();
    render();
    return;
  }
  if (["pay-asset", "receive-asset"].includes(event.target.id)) {
    state.flow.data.assetKey = event.target.value;
    state.flow.data.amount = "";
    renderDialog();
    document.querySelector(`#${event.target.id}`)?.focus();
  }
});

document.addEventListener("click", (event) => {
  const toggle = event.target.closest("[data-toggle-password]");
  if (!toggle) return;
  const input = document.querySelector("#unlock-password");
  const visible = input.type === "text";
  input.type = visible ? "password" : "text";
  toggle.setAttribute("aria-label", visible ? "Show password" : "Hide password");
  toggle.querySelector("use").setAttribute("href", visible ? "#i-eye" : "#i-eye-off");
});

dialog.addEventListener("click", (event) => {
  if (event.target !== dialog) return;
  const rect = dialog.getBoundingClientRect();
  const inside = event.clientX >= rect.left && event.clientX <= rect.right && event.clientY >= rect.top && event.clientY <= rect.bottom;
  if (!inside) closeDialog();
});

dialog.addEventListener("close", () => {
  state.flow = null;
  const trigger = state.lastDialogTrigger;
  state.lastDialogTrigger = null;
  if (trigger?.isConnected) trigger.focus();
});

syncConfigDraftFromState();
applyAppearancePreferences();
render();
