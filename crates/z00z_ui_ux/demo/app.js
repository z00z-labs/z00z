"use strict";

const main = document.querySelector("#main-content");
const pageTitle = document.querySelector("#page-title");
const pageContext = document.querySelector("#page-context");
const topbarAddressGroup = document.querySelector(".topbar-address-group");
const mobileTopbarContext = document.querySelector("#mobile-topbar-context");
const mobileActiveWallet = document.querySelector("#mobile-active-wallet");
const routeBreadcrumb = document.querySelector("#route-breadcrumb");
const walletNav = document.querySelector("#wallet-nav");
const walletNavViewport = document.querySelector(".wallet-nav-viewport");
const sidebarWalletsLabel = document.querySelector(".sidebar-label");
const sidebarNavigationScrollRegion = document.querySelector(".sidebar-navigation-scroll-region");
const navigationTree = document.querySelector("#app-navigation-tree");
const navigationTerminal = document.querySelector("#app-navigation-terminal");
const walletIdentity = document.querySelector("#wallet-identity");
const walletStatusbar = document.querySelector("#wallet-statusbar");
const lockWalletLabel = document.querySelector("#lock-wallet-label");
const mobileMenuButton = document.querySelector("#mobile-menu-button");
const mobileMenuBackdrop = document.querySelector("#mobile-menu-backdrop");
const mobilePopupMenu = document.querySelector("#mobile-popup-menu");
const walletPickerPopup = document.querySelector("#wallet-picker-popup");
const menuSearchTrigger = document.querySelector("#menu-search-trigger");
const menuSearchOverlay = document.querySelector("#menu-search-overlay");
const menuSearchBackdrop = document.querySelector("#menu-search-backdrop");
const menuSearchDialog = document.querySelector("#menu-search-dialog");
const menuSearchTitle = document.querySelector("#menu-search-title");
const menuSearchLabel = document.querySelector("#menu-search-label");
const menuSearchInput = document.querySelector("#menu-search-input");
const menuSearchClose = document.querySelector("#menu-search-close");
const menuSearchResults = document.querySelector("#menu-search-results");
const menuSearchStatus = document.querySelector("#menu-search-status");
const appBody = document.querySelector("#app-body");
let mobilePopupType = "";
let mobilePopupTrigger = null;
let desktopWalletPickerTrigger = null;
let menuSearchQuery = "";
let mobileNavigationLayout = window.matchMedia("(max-width: 768px)").matches;
const mobileDrawerSwipe = {
  pointerId: null,
  source: "",
  startX: 0,
  startY: 0,
  direction: "",
  isDragging: false,
  offsetX: 0,
  opacity: 0
};
const mobileDrawerSwipeEdge = 48;
const mobileDrawerSwipeDistance = 56;
let mobileDrawerAnimations = [];
let mobileDrawerMotionId = 0;

const dialog = document.querySelector("#flow-dialog");
const dialogContent = document.querySelector("#dialog-content");
let dialogHistoryActive = false;
let dialogHistoryClosing = false;
const appShell = document.querySelector("#app-shell");
const lockScreen = document.querySelector("#lock-screen");
const i18n = window.Z00ZI18n;
if (!i18n) throw new Error("Z00Z i18n must load before the wallet demo.");
const help = window.Z00ZHelp;
if (!help) throw new Error("Z00Z Help must load before the wallet demo.");
const demoRuntime = window.Z00ZDemo;
if (!demoRuntime?.APP_VERSION || !demoRuntime.PORT_CONTRACT || !demoRuntime.WALLET_CHAIN_OPTIONS || !demoRuntime.ASSET_CATALOG || !demoRuntime.DAPP_CATALOG || !demoRuntime.MESSENGER_MESSAGES || !demoRuntime.CONTACT_FIXTURES || !demoRuntime.createInitialState || !demoRuntime.createNavigationSession || !demoRuntime.createMockWalletGateway || !demoRuntime.createMockTelemetryGateway || !demoRuntime.createMockDappGateway || !demoRuntime.createMockMessengerGateway || !demoRuntime.createMockContactsGateway) {
  throw new Error("Z00Z production-port modules must load before the wallet demo.");
}
const uiLanguages = i18n.languages();
const paletteOptions = demoRuntime.PALETTE_OPTIONS;
const codeThemeOptions = demoRuntime.CODE_THEME_OPTIONS;
const walletChainOptions = demoRuntime.WALLET_CHAIN_OPTIONS;
const valuationCurrencyOptions = Object.freeze([
  Object.freeze({ id: "USD", flags: "🇺🇸" }),
  Object.freeze({ id: "GBP", languageIds: ["en"] }),
  Object.freeze({ id: "EUR", languageIds: ["fr", "de", "es", "pt"] }),
  Object.freeze({ id: "RUB", languageIds: ["ru"] }),
  Object.freeze({ id: "KRW", languageIds: ["ko"] }),
  Object.freeze({ id: "TRY", languageIds: ["tr"] }),
  Object.freeze({ id: "JPY", languageIds: ["ja"] }),
  Object.freeze({ id: "CNY", languageIds: ["zh-Hans"] })
].map((entry) => Object.freeze({
  ...entry,
  flags: entry.flags || entry.languageIds
    .map((languageId) => uiLanguages.find(({ id }) => id === languageId)?.flag)
    .filter(Boolean)
    .join(" ")
})));
const navigationSession = demoRuntime.createNavigationSession("app");
const restoredNavigationSnapshot = navigationSession.read();
const navigationScrollPositions = {
  desktop: restoredNavigationSnapshot?.scrollPositions.desktop || 0,
  mobile: restoredNavigationSnapshot?.scrollPositions.mobile || 0
};
let navigationSessionReady = false;

function synchronizePalettePreference(paletteId) {
  const url = new URL(window.location.href);
  url.searchParams.set("palette", paletteId);
  url.searchParams.delete("theme");
  window.history.replaceState(window.history.state, "", url);
}

const state = demoRuntime.createInitialState({
  search: window.location.search
});
const walletGateway = demoRuntime.createMockWalletGateway(state);
const telemetryGateway = demoRuntime.createMockTelemetryGateway();
const dappGateway = demoRuntime.createMockDappGateway();
const messengerGateway = demoRuntime.createMockMessengerGateway();
const contactsGateway = demoRuntime.createMockContactsGateway(state);

function persistNavigationState() {
  if (!navigationSessionReady) return;
  navigationSession.write({
    activeRoute: state.activeRoute,
    expandedBranchIds: state.expandedBranchIds,
    scrollPositions: navigationScrollPositions,
    drawerOpen: Boolean(state.drawerOpen && isMobileNavigation())
  });
}

function captureNavigationScrollPosition(layout, scrollTop) {
  const value = Number(scrollTop);
  if (!Number.isFinite(value) || value < 0) return;
  navigationScrollPositions[layout] = Math.round(value);
  persistNavigationState();
}

function restoreNavigationScrollPosition(element, layout) {
  if (!element) return;
  element.scrollTop = navigationScrollPositions[layout] || 0;
}

function clearExternalReviewHandoffs() {
  state.dappWalletReviewHandoff = null;
  state.messengerWalletReviewHandoff = null;
  state.contactActionHandoff = null;
}

const passwordManagerIgnoreAttributeMap = Object.freeze({
  "data-form-type": "other",
  "data-1p-ignore": "true",
  "data-lpignore": "true",
  "data-bwignore": "true",
  "data-protonpass-ignore": "true"
});
const passwordManagerIgnoreAttributes = Object.entries(passwordManagerIgnoreAttributeMap)
  .map(([name, value]) => `${name}="${value}"`)
  .join(" ");

function secureEntryAttributes(section = "wallet") {
  return `type="text" class="secure-entry" data-secure-entry data-port-control="secure-entry" inputmode="text" autocomplete="section-z00z-${section} one-time-code" autocapitalize="none" autocorrect="off" spellcheck="false" ${passwordManagerIgnoreAttributes}`;
}

function suppressPasswordManagerUI(root = document) {
  const forms = root.querySelectorAll("form");
  const fields = root.querySelectorAll("input, textarea, select");
  const applyIgnoreAttributes = (element) => {
    Object.entries(passwordManagerIgnoreAttributeMap).forEach(([name, value]) => element.setAttribute(name, value));
  };

  forms.forEach((form) => {
    form.setAttribute("autocomplete", "off");
    applyIgnoreAttributes(form);
  });
  fields.forEach((field) => {
    applyIgnoreAttributes(field);
    if (field.matches('input[type="password"], input[data-secure-entry]')) {
      field.type = "text";
      field.classList.add("secure-entry");
      field.setAttribute("data-secure-entry", "");
      field.setAttribute("data-port-control", "secure-entry");
      field.setAttribute("inputmode", "text");
      if (!/one-time-code$/.test(field.autocomplete)) {
        field.setAttribute("autocomplete", "section-z00z-private one-time-code");
      }
      field.setAttribute("autocapitalize", "none");
      field.setAttribute("autocorrect", "off");
      field.setAttribute("spellcheck", "false");
    } else if (!field.hasAttribute("autocomplete")) {
      field.setAttribute("autocomplete", "off");
    }
  });
}

const headings = {
  wallet: ["Wallet", "Assets, vouchers, and permissions stay distinct"],
  "wallet-send": ["assets.send", "Assets, vouchers, and permissions stay distinct"],
  "wallet-receive": ["assets.receive", "Assets, vouchers, and permissions stay distinct"],
  "wallet-import": ["navigation.import", "Claim a verified public asset package from disk"],
  "wallet-merge-split": ["navigation.mergeSplit", "Recompose fragments without changing their base series"],
  activity: ["History", "Asset, voucher, permission, policy, and security events"],
  swap: ["Swap", "Move value between assets in this wallet"],
  staking: ["Earn", "Put selected wallet value to work with clear terms"],
  "wallet-backup": ["Backup", "Protect the selected wallet with a verified local backup"],
  "wallet-settings": ["Wallet settings", "Configure this wallet without changing other local profiles"],
  settings: ["app.settings", "app.settingsContext"],
  telemetry: ["Telemetry", "navigation.telemetryContext"],
  "data-storage": ["navigation.dataStorage", "Local concept usage without wallet secrets"],
  about: ["navigation.about", "plan2.about.context"]
};

const telemetryTopbar = {
  onionnet: ["OnionNet", "network.routeTelemetry"],
  reticulum: ["Reticulum", "network.carrierTelemetry"],
  aggregators: ["Aggregators", "network.publicationTelemetry"]
};

const networkEntries = [
  { key: "reticulum", label: "Reticulum", initials: "R", helperKey: "network.carrierTelemetry" },
  { key: "onionnet", label: "OnionNet", initials: "O", helperKey: "network.routeTelemetry" },
  { key: "aggregators", label: "Aggregators", initials: "A", helperKey: "network.publicationTelemetry" }
];

function t(key, values) {
  return i18n.translate(state.language, key, values);
}

function walletChain(chainId) {
  return walletChainOptions.find(({ id }) => id === chainId) || walletChainOptions[0];
}

function walletChainOptionsMarkup(selectedChainId = "mainnet") {
  return walletChainOptions.map(({ id, label }) => `<option value="${escapeHtml(id)}"${id === selectedChainId ? " selected" : ""}>${escapeHtml(label)}</option>`).join("");
}

function walletChainBadgeMarkup(chainId) {
  const chain = walletChain(chainId);
  return `<span class="environment-tag is-${chain.tone}" title="${escapeHtml(t("common.readOnly"))}">${escapeHtml(chain.label)}</span>`;
}

function languagePickerMarkup(className = "") {
  const selected = uiLanguages.find(({ id }) => id === state.language) || uiLanguages[0];
  const label = t("app.language");
  return `<div class="language-picker${className ? ` ${className}` : ""}" data-language-picker>
    <button class="language-picker-trigger" type="button" data-language-picker-trigger aria-label="${escapeHtml(label)}" aria-haspopup="listbox" aria-expanded="false" aria-controls="language-picker-options">
      <span class="language-picker-value"><span aria-hidden="true">${escapeHtml(selected.flag)}</span><span>${escapeHtml(selected.nativeName)}</span></span>
      ${icon("chevron")}
    </button>
    <div class="language-picker-menu" id="language-picker-options" data-language-picker-menu role="listbox" aria-label="${escapeHtml(label)}" hidden>
      ${uiLanguages.map(({ id, nativeName, flag }) => `<button class="language-picker-option${id === state.language ? " is-selected" : ""}" type="button" role="option" aria-selected="${id === state.language}" tabindex="${id === state.language ? "0" : "-1"}" data-language-picker-option="${escapeHtml(id)}"><span aria-hidden="true">${escapeHtml(flag)}</span><span>${escapeHtml(nativeName)}</span>${id === state.language ? icon("check") : ""}</button>`).join("")}
    </div>
  </div>`;
}

function closeLanguagePicker(picker, { restoreFocus = false } = {}) {
  if (!picker?.classList.contains("is-open")) return;
  picker.classList.remove("is-open");
  const menu = picker.querySelector("[data-language-picker-menu]");
  if (menu) {
    menu.hidden = true;
    menu.removeAttribute("style");
  }
  const trigger = picker.querySelector("[data-language-picker-trigger]");
  trigger?.setAttribute("aria-expanded", "false");
  if (restoreFocus) trigger?.focus();
}

function closeLanguagePickers({ restoreFocus = false } = {}) {
  document.querySelectorAll("[data-language-picker].is-open").forEach((picker) => {
    closeLanguagePicker(picker, { restoreFocus });
  });
}

function openLanguagePicker(picker) {
  closeLanguagePickers();
  const trigger = picker.querySelector("[data-language-picker-trigger]");
  const menu = picker.querySelector("[data-language-picker-menu]");
  if (!trigger || !menu) return;
  picker.classList.add("is-open");
  trigger.setAttribute("aria-expanded", "true");
  menu.hidden = false;
  requestAnimationFrame(() => {
    if (!picker.classList.contains("is-open")) return;
    const triggerRect = trigger.getBoundingClientRect();
    const viewportPadding = 12;
    const menuHeight = Math.min(menu.scrollHeight, 360);
    const spaceAbove = triggerRect.top - viewportPadding;
    const spaceBelow = window.innerHeight - triggerRect.bottom - viewportPadding;
    const opensUpward = spaceBelow < Math.min(menuHeight, 224) && spaceAbove > spaceBelow;
    const availableHeight = Math.max(128, opensUpward ? spaceAbove : spaceBelow);
    const width = Math.min(Math.max(triggerRect.width, 220), window.innerWidth - viewportPadding * 2);
    const left = Math.max(viewportPadding, Math.min(triggerRect.right - width, window.innerWidth - width - viewportPadding));
    menu.style.left = `${Math.round(left)}px`;
    menu.style.width = `${Math.round(width)}px`;
    menu.style.maxHeight = `${Math.floor(availableHeight)}px`;
    if (opensUpward) {
      menu.style.top = "auto";
      menu.style.bottom = `${Math.max(viewportPadding, Math.round(window.innerHeight - triggerRect.top + 6))}px`;
    } else {
      menu.style.top = `${Math.round(triggerRect.bottom + 6)}px`;
      menu.style.bottom = "auto";
    }
  });
}

let selectPickerSequence = 0;

function selectPickerLabel(select) {
  const explicitLabel = select.getAttribute("aria-label")?.trim();
  if (explicitLabel) return explicitLabel;
  const label = select.labels?.[0];
  const labelText = label?.querySelector(".field-label, :scope > span, :scope > strong")?.textContent?.trim();
  return labelText || select.name || "Choose an option";
}

function selectOptionLabel(option) {
  return option.label?.trim() || option.textContent?.trim() || option.value;
}

function syncSelectPicker(select) {
  const picker = select.closest("[data-select-picker]");
  if (!picker) return;
  const trigger = picker.querySelector("[data-select-picker-trigger]");
  const menu = picker.querySelector("[data-select-picker-menu]");
  const selected = select.selectedOptions[0] || select.options[0];
  if (!trigger || !menu || !selected) return;
  trigger.disabled = select.disabled;
  trigger.innerHTML = `<span>${escapeHtml(selectOptionLabel(selected))}</span>${icon("chevron")}`;
  menu.replaceChildren(...[...select.options].map((option, index) => {
    const optionButton = document.createElement("button");
    optionButton.className = `select-picker-option${option.selected ? " is-selected" : ""}`;
    optionButton.type = "button";
    optionButton.role = "option";
    optionButton.disabled = option.disabled;
    optionButton.tabIndex = option.selected ? 0 : -1;
    optionButton.dataset.selectPickerIndex = String(index);
    optionButton.setAttribute("aria-selected", String(option.selected));
    optionButton.textContent = selectOptionLabel(option);
    return optionButton;
  }));
}

function closeSelectPicker(picker, { restoreFocus = false } = {}) {
  if (!picker?.classList.contains("is-open")) return;
  picker.classList.remove("is-open");
  const menu = picker.querySelector("[data-select-picker-menu]");
  if (menu) {
    menu.hidden = true;
    menu.removeAttribute("style");
  }
  const trigger = picker.querySelector("[data-select-picker-trigger]");
  trigger?.setAttribute("aria-expanded", "false");
  if (restoreFocus) trigger?.focus();
}

function closeSelectPickers({ restoreFocus = false } = {}) {
  document.querySelectorAll("[data-select-picker].is-open").forEach((picker) => {
    closeSelectPicker(picker, { restoreFocus });
  });
}

function openSelectPicker(picker) {
  closeLanguagePickers();
  closeSelectPickers();
  const trigger = picker.querySelector("[data-select-picker-trigger]");
  const menu = picker.querySelector("[data-select-picker-menu]");
  if (!trigger || !menu || trigger.disabled) return;
  picker.classList.add("is-open");
  trigger.setAttribute("aria-expanded", "true");
  menu.hidden = false;
  requestAnimationFrame(() => {
    if (!picker.classList.contains("is-open")) return;
    const triggerRect = trigger.getBoundingClientRect();
    const viewportPadding = 12;
    const menuHeight = Math.min(menu.scrollHeight, 360);
    const spaceAbove = triggerRect.top - viewportPadding;
    const spaceBelow = window.innerHeight - triggerRect.bottom - viewportPadding;
    const opensUpward = spaceBelow < Math.min(menuHeight, 224) && spaceAbove > spaceBelow;
    const availableHeight = Math.max(128, opensUpward ? spaceAbove : spaceBelow);
    const width = Math.min(Math.max(triggerRect.width, 220), window.innerWidth - viewportPadding * 2);
    const left = Math.max(viewportPadding, Math.min(triggerRect.right - width, window.innerWidth - width - viewportPadding));
    menu.style.left = `${Math.round(left)}px`;
    menu.style.width = `${Math.round(width)}px`;
    menu.style.maxHeight = `${Math.floor(availableHeight)}px`;
    if (opensUpward) {
      menu.style.top = "auto";
      menu.style.bottom = `${Math.max(viewportPadding, Math.round(window.innerHeight - triggerRect.top + 6))}px`;
    } else {
      menu.style.top = `${Math.round(triggerRect.bottom + 6)}px`;
      menu.style.bottom = "auto";
    }
    menu.querySelector(".select-picker-option.is-selected:not([disabled])")?.focus();
  });
}

function enhanceNativeSelects(scope = document) {
  const selects = scope instanceof HTMLSelectElement
    ? [scope]
    : [...scope.querySelectorAll("select:not([multiple]):not([data-select-picker-native])")];
  selects.forEach((select) => {
    if (select.size > 1 || select.closest("[data-language-picker]")) return;
    const picker = document.createElement("span");
    const pickerId = `select-picker-${select.id || ++selectPickerSequence}`;
    picker.className = `select-picker ${select.className}`.trim();
    picker.dataset.selectPicker = "";
    picker.innerHTML = `<button class="select-picker-trigger" type="button" data-select-picker-trigger aria-haspopup="listbox" aria-expanded="false" aria-controls="${pickerId}"></button><span class="select-picker-menu" id="${pickerId}" data-select-picker-menu role="listbox" aria-label="${escapeHtml(selectPickerLabel(select))}" hidden></span>`;
    select.parentNode.insertBefore(picker, select);
    picker.prepend(select);
    select.classList.add("select-picker-native");
    select.tabIndex = -1;
    select.setAttribute("aria-hidden", "true");
    select.addEventListener("change", () => syncSelectPicker(select));
    select.addEventListener("focus", () => picker.querySelector("[data-select-picker-trigger]")?.focus({ preventScroll: true }));
    syncSelectPicker(select);
  });
}

function selectLanguage(languageId) {
  const nextLanguage = i18n.resolveLanguage(languageId);
  const languageChanged = state.language !== nextLanguage;
  state.language = nextLanguage;
  syncConfigDraftFromState();
  applyAppearancePreferences();
  render();
  if (languageChanged) showToast(t("app.languageChanged"));
  requestAnimationFrame(() => document.querySelector("[data-language-picker-trigger]")?.focus());
}

function regionalLocaleOptionsMarkup() {
  return uiLanguages.map(({ locale, nativeName }) => `<option value="${locale}"${state.regionalLocale === locale ? " selected" : ""}>${nativeName} · ${locale}</option>`).join("");
}

function valuationCurrencyName(currencyId) {
  try {
    return new Intl.DisplayNames([state.regionalLocale], { type: "currency" }).of(currencyId) || currencyId;
  } catch {
    return currencyId;
  }
}

function valuationCurrencyOptionsMarkup() {
  return valuationCurrencyOptions.map(({ id, flags }) => {
    const label = `${flags} ${id} · ${valuationCurrencyName(id)}`;
    return `<option value="${id}"${state.valuationCurrency === id ? " selected" : ""}>${escapeHtml(label)}</option>`;
  }).join("");
}

function applyDocumentTranslations() {
  document.documentElement.lang = state.language;
  document.documentElement.dir = uiLanguages.find((language) => language.id === state.language)?.direction ?? "ltr";
  document.title = t("app.documentTitle");
  document.querySelectorAll("[data-i18n]").forEach((element) => {
    element.textContent = t(element.dataset.i18n);
  });
}

function formatLocalizedNumber(value, options) {
  return i18n.formatNumber(value, state.language, state.regionalLocale, options);
}

function formatLocalizedDateTime(value, options) {
  return i18n.formatDateTime(value, state.language, state.regionalLocale, state.timeZone, options);
}

function formatLocalizedBitrate(bitsPerSecond) {
  return i18n.formatBitrate(bitsPerSecond, state.language, state.regionalLocale);
}

function formatValuation(value) {
  const amount = Number(value);
  if (!Number.isFinite(amount)) return "—";
  return formatLocalizedNumber(amount, {
    style: "currency",
    currency: state.valuationCurrency,
    currencyDisplay: "code"
  });
}

function activeWallet() {
  return demoRuntime.activeWallet(state);
}

function activeWalletPreferences() {
  return demoRuntime.ensureWalletPreferences(state, activeWallet());
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
    `    regional_locale: \"${yamlScalar(state.regionalLocale)}\"`,
    `    valuation_currency: ${state.valuationCurrency}`,
    `    time_zone: \"${yamlScalar(state.timeZone)}\"`,
    `    network_units: ${state.networkUnits}`,
    `    notifications: ${state.notifications}`,
    "  appearance:",
    `    palette: ${state.palette}`,
    `    text_scale: ${state.textScale}`,
    `    reduced_motion: ${state.reducedMotion}`,
    `    code_theme: ${state.codeTheme}`,
    "",
    "wallet:",
    `  id: \"${yamlScalar(wallet.id)}\"`,
    `  chain: \"${yamlScalar(wallet.chainId)}\"`,
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

function paletteForId(paletteId) {
  return demoRuntime.paletteOption(paletteId);
}

function paletteName(palette) {
  return t(palette.id === "z00z-corporate"
    ? "plan2.palette.corporateName"
    : "plan2.palette.defaultName");
}

function applyPalette(paletteId) {
  const palette = paletteForId(paletteId);
  state.palette = palette.id;
  mergeShellState({ type: "set_palette", palette: palette.id });
  synchronizePalettePreference(palette.id);
  return palette;
}

function applyAppearancePreferences() {
  const root = document.documentElement;
  const palette = paletteForId(state.palette);
  root.dataset.palette = palette.id;
  delete root.dataset.theme;
  root.dataset.codeTheme = state.codeTheme;
  root.dataset.textScale = state.textScale;
  root.dataset.reducedMotion = String(state.reducedMotion);
  applyDocumentTranslations();
  const themeColor = getComputedStyle(root).getPropertyValue("--bg-canvas").trim();
  if (themeColor) document.querySelector('meta[name="theme-color"]').content = themeColor;
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
  const language = readYamlScalar(source, "language");
  const regionalLocale = readYamlScalar(source, "regional_locale");
  const valuationCurrency = readYamlScalar(source, "valuation_currency");
  const timeZone = readYamlScalar(source, "time_zone");
  const networkUnits = readYamlScalar(source, "network_units");
  const textScale = readYamlScalar(source, "text_scale");
  const notifications = readYamlScalar(source, "notifications");
  const reducedMotion = readYamlScalar(source, "reduced_motion");
  const codeTheme = readYamlScalar(source, "code_theme");
  const chainId = readYamlScalar(source, "chain");
  const appLockAfter = readYamlScalar(source, "lock_after_minutes");
  const defaultFee = readYamlScalar(source, "default_fee");
  const hideSensitive = readYamlScalar(source, "hide_sensitive_amounts");
  const expertDetails = readYamlScalar(source, "expert_details");

  if (language && !uiLanguages.some((entry) => entry.id === language)) return { valid: false, message: "language must be a supported UI language code." };
  if (regionalLocale && !uiLanguages.some((entry) => entry.locale === regionalLocale)) return { valid: false, message: "regional_locale must use a supported locale." };
  if (valuationCurrency && !valuationCurrencyOptions.some((entry) => entry.id === valuationCurrency)) return { valid: false, message: "valuation_currency must use a supported ISO 4217 code." };
  if (timeZone && !["UTC", "Asia/Jerusalem", "Europe/Berlin", "America/New_York", "Asia/Tokyo", "Asia/Shanghai"].includes(timeZone)) return { valid: false, message: "time_zone must use a supported IANA identifier." };
  if (networkUnits && networkUnits !== "decimal-bps") return { valid: false, message: "network_units must be decimal-bps." };
  if (textScale && !["100", "110", "125"].includes(textScale)) return { valid: false, message: "text_scale must be 100, 110, or 125." };
  if (notifications && !["true", "false"].includes(notifications)) return { valid: false, message: "notifications must be true or false." };
  if (reducedMotion && !["true", "false"].includes(reducedMotion)) return { valid: false, message: "reduced_motion must be true or false." };
  if (codeTheme && !codeThemeOptions.some((entry) => entry.id === codeTheme)) return { valid: false, message: "code_theme must use one of the listed preset IDs." };
  if (chainId !== activeWallet().chainId) return { valid: false, message: `chain is read-only and must remain ${activeWallet().chainId}.` };
  if (defaultFee && !/^\d+(?:\.\d+)?$/.test(defaultFee)) return { valid: false, message: "default_fee must be a non-negative decimal." };
  if (hideSensitive && !["true", "false"].includes(hideSensitive)) return { valid: false, message: "hide_sensitive_amounts must be true or false." };
  if (expertDetails && !["true", "false"].includes(expertDetails)) return { valid: false, message: "expert_details must be true or false." };
  if (appLockAfter && !["5", "15", "30", "never"].includes(appLockAfter.toLowerCase())) return { valid: false, message: "lock_after_minutes must be 5, 15, 30, or never." };

  if (apply) {
    if (theme || palette) applyPalette(demoRuntime.resolvePalettePreference({ palette, theme }));
    if (language) state.language = language;
    if (regionalLocale) state.regionalLocale = regionalLocale;
    if (valuationCurrency) state.valuationCurrency = valuationCurrency;
    if (timeZone) state.timeZone = timeZone;
    if (networkUnits) state.networkUnits = networkUnits;
    if (textScale) state.textScale = textScale;
    if (notifications) state.notifications = notifications === "true";
    if (reducedMotion) state.reducedMotion = reducedMotion === "true";
    if (codeTheme) state.codeTheme = codeTheme;
    if (appLockAfter) state.autoLockMinutes = appLockAfter.toLowerCase();
    if (defaultFee) activeWalletPreferences().defaultFee = defaultFee;
    if (hideSensitive) state.balanceHidden = hideSensitive === "true";
    if (expertDetails) state.expertDetails = expertDetails === "true";
    applyAppearancePreferences();
  }

  return { valid: true, message: apply ? "Local concept draft applied. Runtime YAML write/watch is still unavailable." : "YAML draft is valid for this concept schema." };
}

function paletteCard(palette) {
  const isActive = state.palette === palette.id;
  return `<button class="palette-card${isActive ? " is-active" : ""}" type="button" data-palette="${palette.id}" aria-pressed="${isActive}">
    <span class="palette-swatches" aria-hidden="true"><i></i><i></i><i></i><i></i><i></i></span>
    <span class="palette-card-copy"><span class="palette-card-heading"><strong>${escapeHtml(paletteName(palette))}</strong>${isActive ? "<em>Active</em>" : ""}</span></span>
  </button>`;
}

function codeThemeCard(theme) {
  const isActive = state.codeTheme === theme.id;
  return `<button class="code-theme-card${isActive ? " is-active" : ""}" type="button" data-code-theme="${theme.id}" aria-pressed="${isActive}">
    <span class="code-theme-card-heading"><strong>${theme.label}</strong>${isActive ? "<em>Active</em>" : ""}</span>
    <span class="code-theme-preview" aria-hidden="true">
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
      <div class="config-field"><span>${t("app.language")}</span>${languagePickerMarkup()}</div>
      <label><span>${t("app.regionalFormat")}</span><select data-config-control="regional-locale">${regionalLocaleOptionsMarkup()}</select></label>
      <label><span>${t("app.currency")}</span><select aria-label="${escapeHtml(t("app.currency"))}" title="${escapeHtml(t("app.currencyHelp"))}" data-config-control="valuation-currency">${valuationCurrencyOptionsMarkup()}</select></label>
      <label><span>${t("app.timeZone")}</span><select data-config-control="time-zone"><option value="UTC"${state.timeZone === "UTC" ? " selected" : ""}>UTC</option><option value="Asia/Jerusalem"${state.timeZone === "Asia/Jerusalem" ? " selected" : ""}>Asia/Jerusalem</option><option value="Europe/Berlin"${state.timeZone === "Europe/Berlin" ? " selected" : ""}>Europe/Berlin</option><option value="America/New_York"${state.timeZone === "America/New_York" ? " selected" : ""}>America/New_York</option><option value="Asia/Tokyo"${state.timeZone === "Asia/Tokyo" ? " selected" : ""}>Asia/Tokyo</option><option value="Asia/Shanghai"${state.timeZone === "Asia/Shanghai" ? " selected" : ""}>Asia/Shanghai</option></select></label>
      <label><span>${escapeHtml(t("plan2.palette.label"))}</span><select data-config-control="palette">${paletteOptions.map((palette) => `<option value="${palette.id}"${state.palette === palette.id ? " selected" : ""}>${escapeHtml(paletteName(palette))}</option>`).join("")}</select></label>
      <label><span>Text scale</span><select data-config-control="text-scale"><option value="100"${state.textScale === "100" ? " selected" : ""}>100%</option><option value="110"${state.textScale === "110" ? " selected" : ""}>110%</option><option value="125"${state.textScale === "125" ? " selected" : ""}>125%</option></select></label>
      <label><span>Code highlighting</span><select data-config-control="code-theme">${codeThemeOptions.map((theme) => `<option value="${theme.id}"${state.codeTheme === theme.id ? " selected" : ""}>${theme.label}</option>`).join("")}</select></label>
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
  return ["wallet", "wallet-send", "wallet-receive", "wallet-import", "wallet-merge-split", "activity", "swap", "staking", "wallet-backup", "wallet-settings"].includes(state.view);
}

function hasSelectedWalletContext() {
  return Boolean(state.selectedWalletId)
    && !["settings", "telemetry", "dapps", "messenger", "contacts", "data-storage", "about"].includes(state.view);
}

function addWalletProfile(name, chainId = "mainnet", scan = "Scanning") {
  const result = walletGateway.createProfile({ name, chainId, scan });
  if (!result.ok) throw new Error(result.error.message);
  return result.data.wallet;
}

function mergeShellState(action) {
  Object.assign(state, demoRuntime.reduceShellState(state, action));
  if (state.activeWalletId) state.selectedWalletId = state.activeWalletId;
  persistNavigationState();
}

function routeFromCurrentLegacyState() {
  if (["dapps", "messenger", "contacts", "data-storage", "about"].includes(state.view)
    && state.activeRoute?.startsWith(`${state.view}.`)) {
    return state.activeRoute;
  }
  if (state.view === "staking" && state.activeRoute?.startsWith("wallet.staking.")) {
    return state.activeRoute;
  }
  if (state.view === "about" && state.activeRoute === "about") return "about";
  return state.view === "route-preview" && demoRuntime.PORT_CONTRACT.routes.includes(state.previewRoute)
    ? state.previewRoute
    : demoRuntime.canonicalRouteFromLegacyNavigation(state);
}

function synchronizeShellRoute() {
  const routeId = routeFromCurrentLegacyState();
  if (state.activeRoute !== routeId) mergeShellState({ type: "restore_route", routeId });
  if (state.activeWalletId !== state.selectedWalletId && state.selectedWalletId) {
    mergeShellState({ type: "switch_wallet", walletId: state.selectedWalletId, walletRouteCompatible: true });
  }
}

function legacyStateForRoute(routeId) {
  if (["wallet.assets", "wallet.vouchers", "wallet.permissions"].includes(routeId)) {
    return { view: "wallet", walletSection: routeId.split(".").at(-1) };
  }
  if (routeId === "wallet.quarantine") return { view: "wallet", walletSection: "permissions" };
  if (routeId === "wallet.send") return { view: "wallet-send" };
  if (routeId === "wallet.receive") return { view: "wallet-receive" };
  if (routeId === "wallet.import") return { view: "wallet-import" };
  if (routeId === "wallet.merge-split") return { view: "wallet-merge-split" };
  if (routeId === "wallet.history") return { view: "activity" };
  if (routeId === "wallet.swap") return { view: "swap" };
  if (routeId.startsWith("wallet.staking.")) return { view: "staking" };
  if (routeId === "wallet.backup") return { view: "wallet-backup" };
  if (routeId.startsWith("wallet.settings.")) return { view: "wallet-settings", walletSettingsSection: routeId.split(".").at(-1) };
  if (routeId.startsWith("telemetry.reticulum.") || routeId.startsWith("telemetry.onionnet.") || routeId.startsWith("telemetry.aggregators.") || routeId.startsWith("telemetry.watchers.") || routeId.startsWith("telemetry.explorer.")) {
    const [, source, tab] = routeId.split(".");
    return { view: "telemetry", telemetrySource: source, [`${source}TelemetryTab`]: tab };
  }
  if (routeId.startsWith("dapps.")) {
    return {
      view: "dapps",
      dappSection: routeId.split(".").at(-1),
      dappScreen: "list",
      dappSelectedId: null,
      dappReviewConnectionId: null,
      dappReviewValidationError: null,
      dappReviewAcknowledgements: {
        scopeConfirmed: false,
        reauthAcknowledged: false
      }
    };
  }
  if (routeId.startsWith("messenger.")) {
    return {
      view: "messenger",
      messengerSection: routeId.split(".").at(-1),
      messengerScreen: "list",
      messengerSelectedMessageId: null
    };
  }
  if (routeId === "contacts.list") {
    return {
      view: "contacts",
      contactsScreen: "list",
      contactsSelectedId: null
    };
  }
  if (routeId.startsWith("data-storage.")) {
    return { view: "data-storage", dataStorageSection: routeId.split(".").at(-1) };
  }
  if (["settings.general", "settings.notifications", "settings.appearance"].includes(routeId)) {
    return { view: "settings", settingsSection: routeId.split(".").at(-1) };
  }
  if (routeId === "about") return { view: "about" };
  return { view: "route-preview", previewRoute: routeId };
}

function selectCanonicalRoute(routeId, { pushHistory = true } = {}) {
  const node = demoRuntime.navigationNodeForRoute(routeId);
  if (!node) return;
  mergeShellState({ type: "select_leaf", nodeId: node.id });
  Object.assign(state, legacyStateForRoute(routeId));
  if (pushHistory) {
    const url = new URL(window.location.href);
    url.searchParams.set("route", routeId);
    window.history.pushState({ z00zRoute: routeId }, "", url);
  }
}

function navigationLabel(node) {
  return t(node.labelKey);
}

function normalizeMenuSearchValue(value) {
  return String(value || "")
    .normalize("NFKC")
    .replace(/\s+/gu, " ")
    .trim();
}

function normalizeMenuSearchText(value) {
  return normalizeMenuSearchValue(value).toLocaleLowerCase(state.language);
}

function menuSearchRecords() {
  return demoRuntime.NAVIGATION_NODES
    .filter((node) => node.isVisible)
    .map((node, index) => {
      const nodeLabel = navigationLabel(node);
      const titleKey = node.target.kind === "workspace"
        ? node.target.defaultLabelKey
        : node.labelKey;
      const title = normalizeMenuSearchValue(t(titleKey));
      const ancestorLabels = demoRuntime.ancestorContainerIdsForNode(node.id)
        .map((nodeId) => demoRuntime.navigationNode(nodeId))
        .filter(Boolean)
        .map(navigationLabel);
      const pathLabels = node.target.kind === "workspace"
        ? [...ancestorLabels, nodeLabel]
        : ancestorLabels;
      const path = normalizeMenuSearchValue(pathLabels.join(" › ") || t("app.menu"));
      return {
        index,
        node,
        path,
        searchText: normalizeMenuSearchText([title, nodeLabel, ...pathLabels].join(" ")),
        title
      };
    });
}

function searchMenu(queryValue) {
  const query = normalizeMenuSearchText(queryValue);
  return menuSearchRecords()
    .map((record) => {
      if (!query) return { ...record, score: Number.MAX_SAFE_INTEGER };
      const title = normalizeMenuSearchText(record.title);
      const path = normalizeMenuSearchText(record.path);
      const tokens = query.split(/\s+/u).filter(Boolean);
      if (!tokens.every((token) => record.searchText.includes(token))) return null;
      let score = 60;
      if (title === query) score = 0;
      else if (title.startsWith(query)) score = 10;
      else if (title.includes(query)) score = 20;
      else if (path.includes(query)) score = 40;
      return { ...record, score };
    })
    .filter(Boolean)
    .sort((left, right) => left.score - right.score
      || left.index - right.index
      || left.node.id.localeCompare(right.node.id))
    .slice(0, 10);
}

function renderMenuSearch() {
  const matches = searchMenu(menuSearchQuery);
  const searchLabel = t("navigation.search");
  menuSearchStatus.textContent = `${matches.length} ${searchLabel}`;
  menuSearchResults.innerHTML = matches.map((record) => {
    const isActive = ["route", "workspace"].includes(record.node.target.kind)
      && record.node.target.routeId === state.activeRoute;
    return `<button class="menu-search-result${isActive ? " is-active" : ""}" type="button" data-menu-search-node="${escapeHtml(record.node.id)}"${isActive ? ' aria-current="page"' : ""}>
      ${icon(record.node.iconId, "menu-search-result-icon")}
      <span class="menu-search-result-content">
        <strong>${escapeHtml(record.title)}</strong>
        <small>${escapeHtml(record.path)}</small>
      </span>
      <span class="menu-search-result-path">${escapeHtml(record.path)}</span>
    </button>`;
  }).join("") || `<p class="menu-search-empty">${escapeHtml(t("common.unavailable"))}</p>`;
}

function renderMenuSearchChrome() {
  const searchLabel = t("navigation.search");
  menuSearchTitle.textContent = searchLabel;
  menuSearchLabel.textContent = searchLabel;
  menuSearchInput.placeholder = `${searchLabel}…`;
  menuSearchInput.setAttribute("aria-label", searchLabel);
  menuSearchTrigger.setAttribute("aria-label", searchLabel);
  menuSearchTrigger.setAttribute("title", searchLabel);
  menuSearchClose.setAttribute("aria-label", t("common.close"));
  renderMenuSearch();
}

function menuSearchIsOpen() {
  return !menuSearchOverlay.hidden;
}

function openMenuSearch() {
  if (menuSearchIsOpen()) {
    menuSearchInput.focus();
    return;
  }
  if (dialog.open || state.locked) return;
  closeMobilePopup();
  closeDesktopWalletPicker();
  closeLanguagePickers();
  closeSelectPickers();
  menuSearchOverlay.hidden = false;
  document.body.classList.add("has-menu-search");
  menuSearchTrigger.setAttribute("aria-expanded", "true");
  appShell.inert = true;
  renderMenuSearch();
  requestAnimationFrame(() => menuSearchInput.focus());
}

function closeMenuSearch({ restoreFocus = false } = {}) {
  const wasOpen = menuSearchIsOpen();
  menuSearchOverlay.hidden = true;
  document.body.classList.remove("has-menu-search");
  menuSearchTrigger.setAttribute("aria-expanded", "false");
  menuSearchQuery = "";
  menuSearchInput.value = "";
  renderMenuSearch();
  appShell.inert = state.locked;
  if (restoreFocus && wasOpen && menuSearchTrigger.getClientRects().length > 0) {
    menuSearchTrigger.focus();
  }
}

function activateMenuSearchNode(nodeId) {
  const node = demoRuntime.navigationNode(nodeId);
  if (!node) return;
  closeMenuSearch();
  if (["route", "workspace"].includes(node.target.kind)) {
    selectCanonicalRoute(node.target.routeId);
    render({ focusMain: true });
    return;
  }
  if (node.target.kind === "branch") {
    if (!state.expandedBranchIds.includes(node.id)) {
      mergeShellState({ type: "toggle_branch", nodeId: node.id });
    }
    render();
    requestAnimationFrame(() => {
      const branch = navigationTree.querySelector(`[data-navigation-branch="${CSS.escape(node.id)}"]`);
      branch?.scrollIntoView({ block: "nearest" });
      branch?.focus({ preventScroll: true });
    });
    return;
  }
  if (node.target.kind === "help") {
    help.open(node.helpTopicId);
    return;
  }
  if (node.target.kind === "action") {
    handleDemoAction(node.target.actionId, menuSearchTrigger);
  }
}

function navigationNodeMarkup(node, { prefix, depth = 0, terminal = false } = {}) {
  const nodeLabel = escapeHtml(navigationLabel(node));
  const selectedRouteNode = demoRuntime.navigationNodeForRoute(state.activeRoute);
  const activeRouteNode = ["route", "workspace"].includes(node.target.kind) && (
    node.target.routeId === state.activeRoute
    || (node.target.kind === "workspace"
      && demoRuntime.ancestorContainerIdsForNode(selectedRouteNode?.id || "").includes(node.id))
  );
  const activeBranch = ["branch", "group"].includes(node.target.kind)
    && demoRuntime.ancestorContainerIdsForNode(selectedRouteNode?.id || "").includes(node.id);
  const depthClass = `is-depth-${depth}`;
  const sectionBreakClass = node.sectionBreakBefore ? " navigation-tree-section-break" : "";
  if (node.target.kind === "branch") {
    const expanded = state.expandedBranchIds.includes(node.id);
    const controlId = `${prefix}-${node.id.replaceAll(".", "-")}-toggle`;
    const panelId = `${prefix}-${node.id.replaceAll(".", "-")}-children`;
    return `<section class="navigation-tree-branch ${depthClass}${sectionBreakClass}${expanded ? " is-expanded" : ""}${activeBranch ? " has-active-descendant" : ""}">
      <button id="${controlId}" class="navigation-tree-item navigation-tree-branch-toggle" type="button" data-navigation-branch="${escapeHtml(node.id)}" aria-expanded="${expanded}" aria-controls="${panelId}">
        ${icon(node.iconId, "navigation-tree-icon")}
        <span class="navigation-tree-label">${nodeLabel}</span>
        ${icon("chevron", "navigation-tree-chevron")}
      </button>
      <div id="${panelId}" class="navigation-tree-children" role="group" aria-labelledby="${controlId}"${expanded ? "" : " hidden"}>
        ${demoRuntime.navigationChildren(node.id).map((child) => navigationNodeMarkup(child, { prefix, depth: depth + 1 })).join("")}
      </div>
    </section>`;
  }
  if (node.target.kind === "group") {
    const groupId = `${prefix}-${node.id.replaceAll(".", "-")}-group`;
    return `<section class="navigation-tree-group ${depthClass}${sectionBreakClass}${activeBranch ? " has-active-descendant" : ""}" data-navigation-group="${escapeHtml(node.id)}" aria-labelledby="${groupId}">
      <p id="${groupId}" class="navigation-tree-group-label">
        ${icon(node.iconId, "navigation-tree-icon")}
        <span class="navigation-tree-label">${nodeLabel}</span>
      </p>
      <div class="navigation-tree-group-children" role="group" aria-labelledby="${groupId}">
        ${demoRuntime.navigationChildren(node.id).map((child) => navigationNodeMarkup(child, { prefix, depth: depth + 1 })).join("")}
      </div>
    </section>`;
  }
  if (["route", "workspace"].includes(node.target.kind)) {
    return `<button class="navigation-tree-item navigation-tree-leaf${terminal ? " navigation-tree-terminal" : ""} ${depthClass}${sectionBreakClass}${activeRouteNode ? " is-active" : ""}" type="button" data-navigation-route="${escapeHtml(node.target.routeId)}"${node.target.kind === "workspace" ? ` data-navigation-workspace="${escapeHtml(node.id)}"` : ""}${activeRouteNode ? ' aria-current="page"' : ""}>
      ${icon(node.iconId, "navigation-tree-icon")}
      <span class="navigation-tree-label">${nodeLabel}</span>
    </button>`;
  }
  const attributes = node.target.kind === "help"
    ? `data-help-topic="${escapeHtml(node.helpTopicId)}"`
    : `data-demo-action="${escapeHtml(node.target.actionId)}"`;
  return `<button class="navigation-tree-item navigation-tree-terminal ${depthClass}" type="button" ${attributes}>
    ${icon(node.iconId, "navigation-tree-icon")}
    <span class="navigation-tree-label">${nodeLabel}</span>
  </button>`;
}

function renderNavigationTree() {
  const rootNodes = demoRuntime.navigationChildren();
  navigationTree.innerHTML = rootNodes
    .filter((node) => !["settings", "help", "about", "logout"].includes(node.id))
    .map((node) => navigationNodeMarkup(node, { prefix: "desktop-navigation" }))
    .join("");
  navigationTerminal.innerHTML = rootNodes
    .filter((node) => ["settings", "help", "about", "logout"].includes(node.id))
    .map((node) => navigationNodeMarkup(node, { prefix: "desktop-terminal", terminal: true }))
    .join("")
    + `<p class="app-version">Version ${escapeHtml(demoRuntime.APP_VERSION)}</p>`;
}

function walletPickerListMarkup() {
  return `<div class="wallet-picker-list" role="group" aria-label="Wallets">${state.wallets.map((wallet) => {
    const chain = walletChain(wallet.chainId);
    return `<button class="wallet-picker-choice${wallet.id === state.selectedWalletId ? " is-active" : ""}" type="button" role="menuitemradio" data-wallet-picker-id="${escapeHtml(wallet.id)}" data-wallet-chain="${escapeHtml(chain.id)}" aria-checked="${wallet.id === state.selectedWalletId}">
      <span class="wallet-avatar" aria-hidden="true">${escapeHtml(wallet.initials)}</span>
      <span class="wallet-picker-copy"><strong>${escapeHtml(wallet.name)}</strong><small>${t("walletShell.balanceAvailable", { value: `<span class="mono">${sensitive(`${wallet.summary.available} Z00Z`)}</span>` })}</small></span>
      <span class="wallet-nav-state is-${escapeHtml(chain.tone)}" role="img" aria-label="${escapeHtml(chain.label)}"></span>
    </button>`;
  }).join("")}</div>`;
}

function walletAddActionMarkup({ className = "", hasMenuRole = false } = {}) {
  return `<button class="wallet-picker-action nav-item nav-item-primary${className ? ` ${escapeHtml(className)}` : ""}" type="button"${hasMenuRole ? ' role="menuitem"' : ""} data-wallet-picker-action="add-wallet">${icon("plus")}<span>${escapeHtml(t("app.addWallet"))}</span></button>`;
}

function walletPickerPopupMarkup() {
  return `${state.wallets.length ? walletPickerListMarkup() : ""}
  <div class="wallet-picker-actions">
    ${walletAddActionMarkup({ hasMenuRole: true })}
    ${state.wallets.length ? `<button class="wallet-picker-action nav-item nav-item-danger" type="button" role="menuitem" data-wallet-picker-action="remove-wallet">${icon("remove")}<span>${escapeHtml(t("app.removeWallet"))}</span></button>` : ""}
  </div>`;
}

function walletPickerTriggerMarkup(wallet, className) {
  const chain = walletChain(wallet.chainId);
  return `<button class="wallet-picker-trigger ${escapeHtml(className)}" type="button" data-wallet-picker-trigger aria-haspopup="menu" aria-expanded="false" aria-controls="wallet-picker-popup">
    <span class="wallet-avatar" aria-hidden="true">${escapeHtml(wallet.initials)}</span>
    <span class="wallet-picker-trigger-copy"><strong>${escapeHtml(wallet.name)}</strong><small>${t("walletShell.balanceAvailable", { value: `<span class="mono">${sensitive(`${wallet.summary.available} Z00Z`)}</span>` })}</small></span>
    <span class="wallet-nav-state is-${escapeHtml(chain.tone)}" role="img" aria-label="${escapeHtml(chain.label)}"></span>
    ${icon("chevron", "wallet-picker-trigger-chevron")}
  </button>`;
}

function mobileNavigationDrawerMarkup() {
  const rootNodes = demoRuntime.navigationChildren();
  const walletControl = state.wallets.length
    ? walletPickerTriggerMarkup(activeWallet(), "mobile-wallet-picker-trigger")
    : walletAddActionMarkup({ className: "wallet-empty-action" });
  const walletSelector = `<section class="mobile-wallet-selector" aria-label="${escapeHtml(t("app.wallets"))}">
      <p>${escapeHtml(t("app.wallets"))}</p>
      ${walletControl}
    </section>`;
  return `${walletSelector}
    <div class="mobile-navigation-scroll-region">
      <nav class="mobile-navigation-tree" aria-label="${escapeHtml(t("app.menu"))}">
      ${rootNodes.filter((node) => !["settings", "help", "about", "logout"].includes(node.id)).map((node) => navigationNodeMarkup(node, { prefix: "mobile-navigation" })).join("")}
      </nav>
      <nav class="mobile-navigation-terminal" aria-label="${escapeHtml(t("app.settings"))}">
        ${rootNodes.filter((node) => ["settings", "help", "about", "logout"].includes(node.id)).map((node) => navigationNodeMarkup(node, { prefix: "mobile-terminal", terminal: true })).join("")}
        <p class="app-version">Version ${escapeHtml(demoRuntime.APP_VERSION)}</p>
      </nav>
    </div>`;
}

function renderWalletShell() {
  sidebarWalletsLabel.hidden = false;
  walletNavViewport.hidden = false;
  if (state.wallets.length === 0) {
    walletNav.innerHTML = walletAddActionMarkup({ className: "wallet-empty-action" });
    walletIdentity.replaceChildren();
    walletIdentity.removeAttribute("aria-label");
    lockWalletLabel.textContent = "";
    renderNavigationTree();
    walletStatusbar.replaceChildren();
    walletStatusbar.hidden = true;
    return;
  }
  const wallet = activeWallet();
  const summary = wallet.summary;
  walletNav.innerHTML = walletPickerTriggerMarkup(wallet, "wallet-nav-item wallet-picker-sidebar-trigger is-active");
  const walletName = wallet.name;
  const copyLabel = t("walletShell.copyAddress", { wallet: walletName });
  walletIdentity.innerHTML = `
    <div class="wallet-identity-heading">
      <div class="wallet-identity-address-row">
        <strong class="wallet-identity-address" title="${escapeHtml(wallet.fullAddress || wallet.address)}">${escapeHtml(wallet.address)}</strong>
        <button class="icon-button wallet-identity-copy" type="button" data-demo-action="copy-wallet-address" aria-label="${escapeHtml(copyLabel)}" title="${escapeHtml(wallet.fullAddress || wallet.address)}">${icon("copy")}</button>
      </div>
      <div class="wallet-identity-meta">
        <p class="wallet-identity-name">${escapeHtml(t("walletShell.lockLabel", { wallet: walletName }))}</p>
        ${walletChainBadgeMarkup(wallet.chainId)}
      </div>
    </div>
  `;
  walletIdentity.setAttribute("aria-label", t("walletShell.identityAria", { wallet: walletName }));
  lockWalletLabel.innerHTML = `${escapeHtml(t("walletShell.lockLabel", { wallet: walletName }))} <span aria-hidden="true">·</span> <span class="mono">${escapeHtml(wallet.address)}</span>`;
  renderNavigationTree();
  walletStatusbar.innerHTML = `
    <span><small>${t("walletShell.available")}</small><strong>${sensitive(`${summary.available} Z00Z`)}</strong></span>
    <span><small>${t("walletShell.locked")}</small><strong>${sensitive(`${summary.locked} Z00Z`)}</strong></span>
    <span><small>${t("walletShell.pendingIn")}</small><strong>${sensitive(`${summary.pendingIn} Z00Z`)}</strong></span>
    <span><small>${t("walletShell.pendingOut")}</small><strong>${sensitive(`${summary.pendingOut} Z00Z`)}</strong></span>
    <span class="statusbar-telemetry"><small>${t("walletShell.routeTelemetry")}</small><strong><span class="statusbar-state-dot" aria-hidden="true"></span>${t("common.unavailable")}</strong></span>`;
  walletStatusbar.hidden = !hasSelectedWalletContext();
}

function renderMobileActiveWallet(wallet) {
  const hasActiveWallet = Boolean(state.selectedWalletId && wallet);
  mobileActiveWallet.hidden = !hasActiveWallet;
  if (!hasActiveWallet) {
    mobileActiveWallet.innerHTML = "";
    return;
  }
  const copyLabel = t("walletShell.copyAddress", { wallet: wallet.name });
  mobileActiveWallet.innerHTML = `
    <div class="mobile-active-wallet-details">
      <div class="mobile-active-wallet-address-row">
        <span class="mobile-active-wallet-address mono" title="${escapeHtml(wallet.fullAddress || wallet.address)}">${escapeHtml(wallet.address)}</span>
      </div>
      <span class="mobile-active-wallet-name">${escapeHtml(t("walletShell.lockLabel", { wallet: wallet.name }))}</span>
    </div>
    <div class="mobile-active-wallet-actions">
      <button class="icon-button mobile-active-wallet-copy" type="button" data-demo-action="copy-wallet-address" aria-label="${escapeHtml(copyLabel)}" title="${escapeHtml(wallet.fullAddress || wallet.address)}">${icon("copy")}</button>
      ${walletChainBadgeMarkup(wallet.chainId)}
    </div>`;
}

function renderMobileTopbarContext() {
  mobileTopbarContext.replaceChildren();
  if (!isMobileNavigation()) {
    mobileTopbarContext.hidden = true;
    appShell.classList.remove("has-mobile-topbar-context");
    return;
  }
  const source = main.querySelector(".workspace-layout > .context-rail > .context-nav");
  if (!source) {
    mobileTopbarContext.hidden = true;
    appShell.classList.remove("has-mobile-topbar-context");
    return;
  }
  mobileTopbarContext.append(source);
  mobileTopbarContext.hidden = false;
  appShell.classList.add("has-mobile-topbar-context");
}

function icon(name, className = "") {
  return `<svg class="icon ${className}" aria-hidden="true"><use href="#i-${name}"/></svg>`;
}

function objectIconDefinition(definition, className = "") {
  if (!definition) return "";
  const classes = `icon object-family-glyph is-${definition.mode}${className ? ` ${className}` : ""}`;
  if (definition.mode === "mask") {
    const resolvedSource = new URL(definition.iconSrc, document.baseURI).href;
    return `<span class="${classes}" style="--object-family-source:url(${escapeHtml(resolvedSource)})" aria-hidden="true"></span>`;
  }
  return `<img class="${classes}" src="${escapeHtml(definition.iconSrc)}" alt="" decoding="async" draggable="false">`;
}

function objectFamilyIcon(family, className = "") {
  return objectIconDefinition(demoRuntime.OBJECT_FAMILY_ICON_LUT[family], className);
}

function objectTypeIcon(family, type, className = "") {
  const definition = demoRuntime.OBJECT_TYPE_ICON_LUT[family]?.[type] || demoRuntime.OBJECT_FAMILY_ICON_LUT[family];
  if (!definition) return "";
  const glyph = definition.iconSrc ? objectIconDefinition(definition) : icon(definition.iconName);
  return `<span class="object-type-icon ${definition.className}${className ? ` ${className}` : ""}" aria-hidden="true">${glyph}</span>`;
}

function assetIcon(asset, className = "") {
  if (!asset.iconSrc) return objectTypeIcon("asset", asset.type, className);
  return `<span class="object-type-icon is-${escapeHtml(asset.type)} has-brand-icon${className ? ` ${className}` : ""}" aria-hidden="true"><img src="${escapeHtml(asset.iconSrc)}" alt="" decoding="async" draggable="false"></span>`;
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
  const assetKeys = new Set(wallet.assetKeys || ["z00z"]);
  const kindKeys = { coin: "assets.kindCoin", token: "assets.kindToken", nft: "assets.kindCollectible" };
  const kinds = { coin: "Coin", token: "Token", nft: "NFT" };
  return demoRuntime.ASSET_CATALOG
    .filter((asset) => assetKeys.has(asset.key))
    .map((asset) => {
      const balance = asset.key === "z00z" ? wallet.summary.available : asset.demoBalance || "0.00";
      return {
        ...asset,
        kind: kinds[asset.type],
        kindKey: kindKeys[asset.type],
        balance,
        balanceLabel: `${balance} ${asset.unit}`,
        value: "0.00",
        priceKey: "common.unavailable",
        priceNoteKey: "assets.noMarketFeed",
        owner: asset.owner || wallet.fullAddress || wallet.address,
        currentSupply: asset.currentSupply || "Unavailable",
        maxSupply: asset.maxSupply || "Unavailable"
      };
    });
}

function supportedAsset(assetKey = "z00z") {
  const assets = walletAssetEntries();
  return assets.find((asset) => asset.key === assetKey) || assets[0];
}

function walletObjectEntry(family, objectId) {
  const wallet = activeWallet();
  const entries = family === "voucher" ? wallet.vouchers : family === "permission" ? wallet.permissions : [];
  return (entries || []).find((entry) => entry.id === objectId) || null;
}

const sendFamilies = Object.freeze([
  Object.freeze({ id: "asset", labelKey: "assets.sectionAssets", iconName: "assets", createFlow: "" }),
  Object.freeze({ id: "voucher", labelKey: "assets.sectionVouchers", iconName: "voucher-list", createFlow: "create-voucher" }),
  Object.freeze({ id: "permission", labelKey: "assets.sectionPermissions", iconName: "permission-list", createFlow: "create-permission" })
]);

function sendOptionEntries(family = "") {
  const wallet = activeWallet();
  const assets = walletAssetEntries().map((asset) => ({
    key: asset.key,
    family: "asset",
    label: asset.label,
    kindLabel: t(asset.kindKey),
    meta: asset.balanceLabel,
    asset
  }));
  const vouchers = (wallet.vouchers || [])
    .filter((voucher) => voucher.transferable)
    .map((voucher) => ({
      key: `voucher:${voucher.id}`,
      family: "voucher",
      label: voucher.title,
      kindLabel: t("assets.sectionVouchers"),
      meta: voucher.value,
      entry: voucher
    }));
  const permissions = (wallet.permissions || [])
    .filter((permission) => permission.transferable)
    .map((permission) => ({
      key: `permission:${permission.id}`,
      family: "permission",
      label: permission.title,
      kindLabel: t("assets.sectionPermissions"),
      meta: permission.remaining,
      entry: permission
    }));
  const entries = [...assets, ...vouchers, ...permissions];
  return family ? entries.filter((entry) => entry.family === family) : entries;
}

function defaultSendDraft() {
  return {
    family: "asset",
    step: 0,
    recipient: "",
    recipientLabel: "",
    amount: "",
    memo: "",
    itemKey: "z00z",
    reviewedItem: null,
    completed: null,
    idempotencyKey: "",
    operationId: null,
    operationStatus: null,
    operationError: null,
    requestGeneration: 0
  };
}

function activeSendDraft() {
  const wallet = activeWallet();
  state.sendDrafts ||= {};
  state.sendDrafts[wallet.id] ||= defaultSendDraft();
  const draft = state.sendDrafts[wallet.id];
  if (!sendFamilies.some(({ id }) => id === draft.family)) draft.family = "asset";
  const options = sendOptionEntries(draft.family);
  if (draft.step === 0 && !options.some((entry) => entry.key === draft.itemKey)) {
    draft.itemKey = options[0]?.key || "";
  }
  return draft;
}

function resetActiveSendDraft() {
  state.sendDrafts[activeWallet().id] = defaultSendDraft();
  return state.sendDrafts[activeWallet().id];
}

function selectedSendOption(draft = activeSendDraft()) {
  const options = sendOptionEntries(draft.family);
  return options.find((entry) => entry.key === draft.itemKey)
    || (draft.step === 0 ? options[0] : draft.reviewedItem);
}

function sendOptionsMarkup(family, selectedKey) {
  return sendOptionEntries(family).map((entry) => `<option value="${escapeHtml(entry.key)}"${entry.key === selectedKey ? " selected" : ""}>${escapeHtml(entry.label)} · ${escapeHtml(entry.kindLabel)}</option>`).join("");
}

function assetOptions(selectedKey = "z00z") {
  return walletAssetEntries().map((asset) => `<option value="${escapeHtml(asset.key)}"${asset.key === selectedKey ? " selected" : ""}>${escapeHtml(asset.label)} · ${t(asset.kindKey)}</option>`).join("");
}

function moneyView() {
  const assets = walletAssetEntries();
  const assetFilters = [
    ["all", "assets.all"],
    ["coin", "assets.filterCoins"],
    ["token", "assets.filterTokens"],
    ["nft", "assets.filterNfts"]
  ];
  const filteredAssets = state.assetFilter === "all"
    ? assets
    : assets.filter((asset) => asset.type === state.assetFilter);
  return `
    <div class="view-enter">
      <div class="choice-strip" aria-label="${t("assets.filters")}">${assetFilters.map(([value, labelKey]) => `<button class="choice-chip${state.assetFilter === value ? " is-active" : ""}" type="button" data-asset-filter="${value}" aria-pressed="${state.assetFilter === value}">${t(labelKey)}</button>`).join("")}</div>
      <div class="asset-list" role="table" aria-label="${t("nav.assets")}">
        <div class="asset-table-head" role="row" aria-hidden="true"><span>${t("assets.name")}</span><span>${t("assets.balance")}</span><span>${t("assets.value")}</span><span>${t("assets.price")}</span></div>
        ${filteredAssets.map((asset) => `
          <article class="card asset-row" role="row">
            <button class="asset-identity-button" type="button" data-open-flow="asset-detail" data-asset-key="${escapeHtml(asset.key)}" aria-label="${t("assets.viewDetails", { asset: asset.label })}">
              ${assetIcon(asset, "asset-logo")}
              <span class="asset-info"><strong><span class="object-label">${escapeHtml(asset.label)}</span><span class="object-kind">${t(asset.kindKey)}</span></strong></span>
            </button>
            <div class="asset-number" role="cell"><small class="asset-number-label">${t("assets.balance")}</small><strong>${sensitive(asset.balanceLabel)}</strong></div>
            <div class="asset-number" role="cell"><small class="asset-number-label">${t("assets.value")}</small><strong>${asset.value === "—" ? asset.value : sensitive(formatValuation(asset.value))}</strong></div>
            <div class="asset-number" role="cell"><small class="asset-number-label">${t("assets.price")}</small><strong>${t(asset.priceKey)}</strong></div>
          </article>`).join("")}
      </div>
    </div>`;
}

function sendItemIcon(item, className = "") {
  if (item.family === "asset") return assetIcon(item.asset, className);
  return objectTypeIcon(item.family === "voucher" ? "voucher" : "right", item.entry.kind, className);
}

function sendStepIndicator(activeStep) {
  return `<div class="step-indicator" aria-label="Step ${activeStep + 1} of 3">${Array.from({ length: 3 }, (_, index) => `<span class="${index < activeStep ? "is-done" : index === activeStep ? "is-active" : ""}"></span>`).join("")}</div>`;
}

function sendPanelFrame({ title, subtitle, step, body, footer, panelClass = "" }) {
  return `<section class="send-panel${panelClass ? ` ${panelClass}` : ""}" aria-labelledby="send-panel-title">
    <header class="wallet-action-header send-panel-header">
      <div><h2 id="send-panel-title">${escapeHtml(title)}</h2><p>${escapeHtml(subtitle)}</p></div>
      ${sendStepIndicator(step)}
    </header>
    <div class="send-panel-body">${body}</div>
    <footer class="send-panel-footer">${footer}</footer>
  </section>`;
}

function sendFamilyContextNav(activeFamily) {
  return `<nav class="context-nav context-tab-list" aria-label="${t("send.familyNavigation")}">${sendFamilies.map(({ id, labelKey, iconName }) => `
    <button class="context-nav-item${activeFamily === id ? " is-active" : ""}" type="button" ${activeFamily === id ? 'aria-current="page"' : ""} data-send-family="${id}">
      ${icon(iconName)}<span><strong>${t(labelKey)}</strong></span>
    </button>`).join("")}</nav>`;
}

function sendFamilyFacts(item) {
  if (!item) return "";
  const facts = item.family === "asset"
    ? [
      [t("send.objectType"), item.kindLabel],
      [t("send.spendableBalance"), item.asset.balanceLabel],
      [t("send.unit"), item.asset.unit],
      [t("common.chain"), walletChain(activeWallet().chainId).label]
    ]
    : item.family === "voucher"
      ? [
        [t("send.conditionalValue"), item.entry.value],
        [t("send.lifecycle"), item.entry.status],
        [t("send.expires"), item.entry.expiry || t("common.unavailable")],
        [t("send.receiverAcceptance"), t("send.required")]
      ]
      : [
        [t("send.authorityValue"), t("send.zeroValue")],
        [t("send.action"), item.entry.action],
        [t("send.scope"), item.entry.scope],
        [t("send.remainingUses"), item.entry.remaining],
        [t("send.expires"), item.entry.expiry],
        [t("send.delegation"), item.entry.delegation]
      ];
  return `<dl class="send-family-facts">${facts.map(([label, value]) => `<div><dt>${escapeHtml(label)}</dt><dd>${escapeHtml(value)}</dd></div>`).join("")}</dl>`;
}

function sendEmptyPanel(draft) {
  const family = sendFamilies.find(({ id }) => id === draft.family) || sendFamilies[0];
  const action = family.createFlow
    ? `<button class="button button-primary" type="button" data-open-flow="${family.createFlow}">${icon("plus")} ${t(family.id === "voucher" ? "assets.createVoucher" : "assets.createPermission")}</button>`
    : "";
  return sendPanelFrame({
    title: t("send.title"),
    subtitle: t("send.subtitle"),
    step: 0,
    body: `<div class="object-empty-state"><h2>${t(`send.empty.${family.id}`)}</h2>${action}</div>`,
    footer: `<button class="button button-quiet" type="button" data-send-action="cancel">${t("common.back")}</button>`
  });
}

function dappWalletReviewNotice() {
  const handoff = state.dappWalletReviewHandoff;
  if (!handoff || handoff.target.routeId !== state.activeRoute) return "";
  const descriptor = demoRuntime.dappDescriptor(handoff.source.descriptorId);
  return `<div class="capability-note dapp-wallet-handoff" data-dapp-wallet-handoff="${escapeHtml(handoff.handoffId)}">${icon("shield")}<span><strong>Prepared from ${escapeHtml(descriptor?.label || "bounded dApp intent")}</strong><small>Only a typed, immutable prefill crossed into Wallet. Confirm the recipient, amount, item, and fee here; no wallet object changed during handoff.</small></span></div>`;
}

function messengerWalletReviewNotice() {
  const handoff = state.messengerWalletReviewHandoff;
  if (!handoff || handoff.target.routeId !== state.activeRoute) return "";
  return `<div class="capability-note messenger-wallet-handoff" data-messenger-wallet-handoff="${escapeHtml(handoff.handoffId)}">${icon("shield")}<span><strong>Revalidated Messenger payment request</strong><small>Wallet accepted only the typed amount and asset prefill. The recipient remains blank and must be confirmed here; message state has no settlement authority.</small></span></div>`;
}

function contactWalletReviewNotice() {
  const handoff = state.contactActionHandoff;
  if (!handoff || handoff.action !== "pay" || handoff.target.routeId !== state.activeRoute) return "";
  return `<div class="capability-note contact-wallet-handoff" data-contact-wallet-handoff="${escapeHtml(handoff.handoffId)}">${icon("shield")}<span><strong>Revalidated Pay action for ${escapeHtml(handoff.label)}</strong><small>The Contact supplied a typed Wallet-recipient reference, not a browser address. Confirm the recipient and every value field inside Wallet.</small></span></div>`;
}

function walletSendView() {
  const wallet = activeWallet();
  const draft = activeSendDraft();
  const item = selectedSendOption(draft);
  const frame = !item && ![2, 3].includes(draft.step)
    ? sendEmptyPanel(draft)
    : draft.step === 0
      ? (() => {
        const amountField = item.family === "asset"
          ? `<div class="field-group"><label class="field-label" for="send-amount">${t("send.amount")}</label><div class="input-with-affix"><input id="send-amount" name="amount" type="number" min="${item.asset.divisible ? "0.01" : "1"}" max="${escapeHtml(item.asset.balance.replaceAll(",", ""))}" step="${item.asset.divisible ? "0.01" : "1"}" inputmode="decimal" value="${escapeHtml(draft.amount)}" placeholder="${item.asset.divisible ? "0.00" : "1"}" aria-describedby="send-amount-hint send-amount-error" required><span class="input-affix">${escapeHtml(item.asset.unit)}</span></div><p class="field-hint" id="send-amount-hint">${t("send.available", { value: sensitive(`${item.asset.balance} ${item.asset.unit}`) })}</p><p class="field-error" id="send-amount-error" role="alert"></p></div>`
          : `<div class="field-group"><span class="field-label">${t("send.transferObject")}</span><div class="send-object-value">${sendItemIcon(item, "send-object-icon")}<span><strong>${escapeHtml(item.label)}</strong><small>${escapeHtml(item.meta)}</small></span></div><p class="field-error" id="send-amount-error" role="alert"></p></div>`;
        return sendPanelFrame({
          title: t("send.title"),
          subtitle: t("send.subtitle"),
          step: 0,
          body: `<form class="form-grid" id="send-entry" autocomplete="off" novalidate>
            <div class="field-group"><label class="field-label" for="send-recipient">${t("send.recipient")}</label><input id="send-recipient" name="recipient" value="${escapeHtml(draft.recipient)}" placeholder="${t("send.recipientPlaceholder")}" autocomplete="off" aria-describedby="send-recipient-error" required><p class="field-error" id="send-recipient-error" role="alert"></p></div>
            <div class="field-group"><label class="field-label" for="send-item">${t(`send.itemLabel.${item.family}`)}</label><select id="send-item" name="itemKey">${sendOptionsMarkup(item.family, item.key)}</select></div>
            ${sendFamilyFacts(item)}
            ${amountField}
            <div class="field-group"><label class="field-label" for="send-memo">${t("send.note")} <span class="muted">(${t("send.optional")})</span></label><input id="send-memo" name="memo" value="${escapeHtml(draft.memo)}" maxlength="80" placeholder="${t("send.notePlaceholder")}" autocomplete="off"></div>
          </form>`,
          footer: `<button class="button button-quiet" type="button" data-send-action="cancel">${t("send.cancel")}</button><button class="button button-primary" type="submit" form="send-entry">${t("send.review")} ${icon("chevron")}</button>`
        });
      })()
      : draft.step === 1
        ? (() => {
          const amountLabel = item.family === "asset" ? `${draft.amount} ${item.asset.unit}` : item.meta;
          return sendPanelFrame({
            title: t("send.reviewTitle"),
            subtitle: t("send.reviewSubtitle"),
            step: 1,
            panelClass: "send-review-panel",
            body: `<div class="review-card review-hero send-review-hero">${sendItemIcon(item, "list-icon")}<strong>${escapeHtml(amountLabel)}</strong><span>${escapeHtml(item.label)} · ${escapeHtml(draft.recipientLabel)}</span></div>
              <div class="review-card send-review-facts">
                <div class="summary-row"><span>${t("send.family")}</span><strong>${t(`send.familyName.${item.family}`)}</strong></div>
                <div class="summary-row"><span>${t("send.item")}</span><strong>${escapeHtml(item.label)}</strong></div>
                <div class="summary-row"><span>${t("send.recipientShort")}</span><strong>${escapeHtml(draft.recipientLabel)}</strong></div>
                <div class="summary-row"><span>${t("send.from")}</span><strong>${escapeHtml(wallet.name)}</strong></div>
                <div class="summary-row"><span>${t("send.fee")}</span><strong>${item.family === "asset" ? t("send.feeAtAuthorization") : t("send.notApplicable")}</strong></div>
                ${draft.memo ? `<div class="summary-row"><span>${t("send.noteShort")}</span><strong>${escapeHtml(draft.memo)}</strong></div>` : ""}
              </div>
              <div class="confirmation-note send-review-confirmation">${icon("shield")} ${t(`send.confirmation.${item.family}`)}</div>`,
            footer: `<button class="button" type="button" data-send-action="back">${t("common.back")}</button><button class="button button-primary" type="button" data-send-action="submit">${t(`send.submit.${item.family}`)}</button>`
          });
        })()
        : draft.step === 2
          ? (() => {
            if (draft.operationError) {
              const operationId = draft.operationId || t("common.unavailable");
              return sendPanelFrame({
                title: "Submission outcome unknown",
                subtitle: "Reconcile before any retry",
                step: 2,
                body: `<div class="operation-error-state" role="alert">${icon("alert")}<div><h3>${escapeHtml(draft.operationError.message)}</h3><p>The native boundary may already have accepted this intent. Re-submitting without reconciliation could duplicate the operation.</p></div></div>
                  <div class="review-card"><div class="summary-row"><span>Operation</span><strong class="mono">${escapeHtml(operationId)}</strong></div><div class="summary-row"><span>Error class</span><strong>${escapeHtml(draft.operationError.code)}</strong></div><div class="summary-row"><span>Draft</span><strong>Preserved for this wallet</strong></div></div>
                  <details class="technical"><summary>Recovery details</summary><div class="technical-content"><span>Reconcile by operation identity through the native gateway.</span><span>Do not infer failure from a timeout and do not generate a new idempotency key.</span></div></details>`,
                footer: `<button class="button" type="button" data-send-action="back">${t("common.back")}</button><button class="button button-primary" type="button" data-send-action="reconcile">Reconcile status</button>`
              });
            }
            const reconciling = draft.operationStatus === "reconciling";
            return sendPanelFrame({
              title: reconciling ? "Reconciling operation" : "Submitting payment",
              subtitle: reconciling ? "Checking the native operation journal" : "One idempotent native intent",
              step: 2,
              body: `<div class="operation-progress-state" role="status" aria-live="polite">
                <div class="result-icon is-settling">${icon("activity")}</div>
                <h3>${reconciling ? "Resolving the recorded outcome…" : "Processing the reviewed intent…"}</h3>
                <ol class="operation-progress-list">
                  <li class="is-done">${icon("check")}<span>Revalidate wallet, item, and authority</span></li>
                  <li class="${reconciling ? "is-done" : "is-active"}">${icon(reconciling ? "check" : "activity")}<span>Submit once with an idempotency key</span></li>
                  <li class="${reconciling ? "is-active" : ""}">${icon("activity")}<span>Record and reconcile the operation identity</span></li>
                </ol>
                <div class="progress-track" aria-hidden="true"><div class="progress-bar" style="width:${reconciling ? "82" : "58"}%"></div></div>
              </div>`,
              footer: `<span class="send-operation-note">${reconciling ? "Safe to leave; the operation journal remains authoritative." : "Closing this view does not imply cancellation."}</span>`
            });
          })()
          : (() => {
          const completed = draft.completed || { family: item.family, label: item.label, amountLabel: item.meta, recipientLabel: draft.recipientLabel };
          return sendPanelFrame({
            title: t(`send.sent.${completed.family}`),
            subtitle: t("send.settlementPending"),
            step: 2,
            body: `<div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>${t("send.settling")}</h3><p>${escapeHtml(completed.label)} · ${escapeHtml(completed.recipientLabel)}</p><div class="receipt-ref mono">${escapeHtml(draft.operationId || "Operation unavailable")}</div></div><div class="review-card"><div class="summary-row"><span>${t("send.value")}</span><strong>${escapeHtml(completed.amountLabel)}</strong></div><div class="summary-row"><span>Native outcome</span><strong>Submitted · pending confirmation</strong></div><div class="summary-row"><span>${t("send.nextUpdate")}</span><strong>${t("send.automatic")}</strong></div></div>`,
            footer: `<button class="button" type="button" data-send-action="history">${t("send.viewHistory")}</button><button class="button button-primary" type="button" data-send-action="done">${t("history.done")}</button>`
          });
        })();

  return `<div class="view-enter workspace-layout send-workspace-layout"><aside class="context-rail">${sendFamilyContextNav(draft.family)}</aside><div class="workspace-panel send-view">${dappWalletReviewNotice()}${messengerWalletReviewNotice()}${contactWalletReviewNotice()}${frame}</div></div>`;
}

function walletReceiveView() {
  const wallet = activeWallet();
  const response = walletGateway.getReceiverCard({ walletId: wallet.id });
  if (!response.ok) {
    return `<div class="view-enter receiver-view">
      <section class="receiver-card receiver-card-unavailable" aria-labelledby="receiver-card-title">
        <header class="wallet-action-header receiver-card-header">
          <div><h2 id="receiver-card-title">${escapeHtml(t("receive.title"))}</h2><p>${escapeHtml(t("receive.subtitle"))}</p></div>
        </header>
        <div class="receiver-card-error" role="status">${icon("alert")}<strong>${escapeHtml(t("receive.unavailable"))}</strong></div>
      </section>
    </div>`;
  }

  const card = response.data;
  const shortValue = (value, start = 12, end = 10) => (
    value.length > start + end + 1 ? `${value.slice(0, start)}…${value.slice(-end)}` : value
  );
  const cardLabel = shortValue(card.card_compact, 14, 10);
  const cardFields = [
    [t("receive.receiverHandle"), card.owner_handle_display],
    [t("receive.ownerHandle"), card.owner_handle],
    [t("receive.viewKey"), card.view_key],
    [t("receive.identityKey"), card.identity_key],
    [t("receive.registryEntry"), card.registry_entry_id],
    [t("receive.cardEpoch"), String(card.card_epoch)],
    [t("receive.signature"), card.signature]
  ];

  return `
    <div class="view-enter receiver-view">
      <section class="receiver-card" aria-labelledby="receiver-card-title">
        <header class="wallet-action-header receiver-card-header">
          <div>
            <h2 id="receiver-card-title">${escapeHtml(t("receive.title"))}</h2>
            <p>${escapeHtml(t("receive.subtitle"))}</p>
          </div>
        </header>
        <div class="receiver-card-body">
          <div class="receiver-card-share">
            <div class="mock-qr receiver-card-qr" data-qr-payload="${escapeHtml(card.card_compact)}" aria-label="${escapeHtml(`${t("receive.verifiedCard")} QR`)}">${qrCells(card.card_compact)}</div>
            <div class="receiver-card-verification">
              <span>${icon("shield")}</span>
              <span><strong>${escapeHtml(t("receive.verifiedCard"))}</strong><small>${escapeHtml(t("receive.shareHint"))}</small></span>
              <span class="status-badge is-ready">${escapeHtml(t("receive.verified"))}</span>
            </div>
          </div>
          <div class="receiver-address-control" title="${escapeHtml(card.card_compact)}">
            <span class="receiver-card-address">
              <small>${escapeHtml(t("receive.compactRecord"))}</small>
              <code>${escapeHtml(cardLabel)}</code>
            </span>
            <button class="icon-button receiver-card-copy" type="button" data-demo-action="copy-receiver-card" data-copy-value="${escapeHtml(card.card_compact)}" aria-label="${escapeHtml(t("receive.copyCard"))}" title="${escapeHtml(t("receive.copyCard"))}">${icon("copy")}</button>
          </div>
          <dl class="receiver-card-fields">
            ${cardFields.map(([label, value]) => `<div><dt>${escapeHtml(label)}</dt><dd><code title="${escapeHtml(value)}">${escapeHtml(shortValue(value))}</code></dd></div>`).join("")}
          </dl>
        </div>
      </section>
    </div>`;
}

function resetAssetImportState() {
  state.assetImport = {
    walletId: activeWallet().id,
    status: "idle",
    fileName: "",
    fileSize: 0,
    reviewToken: "",
    preview: null,
    error: null,
    result: null
  };
  return state.assetImport;
}

function activeAssetImportState() {
  const wallet = activeWallet();
  if (!state.assetImport || state.assetImport.walletId !== wallet.id) return resetAssetImportState();
  return state.assetImport;
}

function assetImportFlagMarkup(label, active, danger = false) {
  const tone = active ? (danger ? "is-attention" : "is-ready") : "";
  return `<span class="status-badge ${tone}">${escapeHtml(label)}: ${active ? "Yes" : "No"}</span>`;
}

function assetImportPanel({ body, footer = "" }) {
  return `<section class="asset-import-panel" aria-labelledby="asset-import-title">
    <header class="wallet-action-header asset-import-header">
      <span class="asset-import-header-icon">${icon("import")}</span>
      <div>
        <h2 id="asset-import-title">Import asset</h2>
        <p>Claim a public asset package from disk into ${escapeHtml(activeWallet().name)}.</p>
      </div>
    </header>
    <div class="asset-import-body">${body}</div>
    ${footer ? `<footer class="asset-import-footer">${footer}</footer>` : ""}
  </section>`;
}

function assetImportIdleView(importState) {
  const error = importState.error
    ? `<div class="asset-import-error" role="alert">${icon("alert")}<span><strong>Package rejected</strong><small>${escapeHtml(importState.error.message)}</small><code>${escapeHtml(importState.error.reason || "IMPORT_MALFORMED_JSON")}</code></span></div>`
    : "";
  return assetImportPanel({
    body: `${error}
      <label class="asset-import-picker" for="asset-import-file">
        <input class="visually-hidden" id="asset-import-file" name="assetPackage" type="file" accept=".json,application/json">
        <span class="asset-import-picker-icon">${icon("backup")}</span>
        <span><strong>Choose asset package</strong><small>AssetPkgWire JSON · maximum 64 KiB</small></span>
        <span class="button button-primary" aria-hidden="true">Choose file</span>
      </label>
      <div class="asset-import-target">
        <span>${icon("wallet")}</span>
        <span><small>Claim target</small><strong>${escapeHtml(activeWallet().name)}</strong></span>
        ${walletChainBadgeMarkup(activeWallet().chainId)}
      </div>
      <ul class="asset-import-boundaries">
        <li>${icon("check")} Public package only; a top-level <code>secret</code> field is rejected.</li>
        <li>${icon("check")} The wallet verifies cryptography, ownership, replay, and claim conflicts.</li>
        <li>${icon("check")} The local file path is never stored or sent to the wallet RPC.</li>
      </ul>`
  });
}

function assetImportReviewView(importState) {
  const { preview } = importState;
  const { asset, ownership, cryptography } = preview;
  const amount = Number.isSafeInteger(asset.amount)
    ? `${asset.amount} atomic unit${asset.amount === 1 ? "" : "s"}`
    : "Exact u64 value checked by native wallet";
  const nominal = Number.isSafeInteger(asset.nominal)
    ? `${asset.nominal} atomic unit${asset.nominal === 1 ? "" : "s"}`
    : "Exact u64 value checked by native wallet";
  const lockHeight = asset.lockHeight === null
    ? "None"
    : Number.isSafeInteger(asset.lockHeight)
      ? asset.lockHeight
      : "Exact u64 value checked by native wallet";
  const stateFlags = [
    assetImportFlagMarkup("Burned", asset.flags.burned, true),
    assetImportFlagMarkup("Frozen", asset.flags.frozen, true),
    assetImportFlagMarkup("Slashed", asset.flags.slashed, true)
  ].join("");

  return assetImportPanel({
    body: `<div class="asset-import-file-row">
        <span>${icon("import")}</span>
        <span><small>Selected package</small><strong>${escapeHtml(preview.file.name)}</strong><small>${escapeHtml(`${preview.file.bytes.toLocaleString("en-US")} bytes`)}</small></span>
        <button class="button button-quiet" type="button" data-asset-import-action="reset">Change</button>
      </div>
      <div class="asset-import-hero">
        <span class="asset-import-class-icon">${icon(asset.class === "Coin" ? "coin" : asset.class === "Token" ? "token" : asset.class === "Nft" ? "nft" : "claim")}</span>
        <span><small>${escapeHtml(asset.class)}</small><strong>${escapeHtml(asset.name)}</strong><code>${escapeHtml(asset.symbol)}</code></span>
        <span class="status-badge is-attention">Review</span>
      </div>
      <dl class="asset-import-facts">
        <div><dt>Amount</dt><dd>${escapeHtml(amount)}</dd></div>
        <div><dt>Decimals</dt><dd>${escapeHtml(asset.decimals)}</dd></div>
        <div><dt>Serial ID</dt><dd>${escapeHtml(asset.serialId)}</dd></div>
        <div><dt>Domain</dt><dd title="${escapeHtml(asset.domainName)}">${escapeHtml(asset.domainName)}</dd></div>
        <div><dt>Definition ID</dt><dd class="mono">${escapeHtml(asset.definitionId)}</dd></div>
        <div><dt>Lock height</dt><dd>${escapeHtml(lockHeight)}</dd></div>
      </dl>
      <div class="asset-import-flags" aria-label="Asset state flags">${stateFlags}</div>
      <div class="asset-import-checks">
        <h3>Wallet verification</h3>
        <div><span>${icon("check")}</span><span><strong>Public DTO schema</strong><small>Known fields, JSON types, size limit, and no secret field.</small></span></div>
        <div><span>${icon("shield")}</span><span><strong>Cryptography and ownership</strong><small>${escapeHtml(ownership.mode)} · verified only by the native wallet.</small></span></div>
        <div><span>${icon("activity")}</span><span><strong>Claim and replay state</strong><small>Reserve nullifier, persist claim, then finalize or quarantine.</small></span></div>
      </div>
      <details class="technical asset-import-technical">
        <summary>Technical package fields</summary>
        <div class="technical-content">
          <span><strong>Commitment</strong><code>${escapeHtml(cryptography.commitment)}</code></span>
          <span><strong>Nonce</strong><code>${escapeHtml(cryptography.nonce)}</code></span>
          <span><strong>Range proof</strong><code>${cryptography.rangeProofPresent ? "Present" : "Not present"}</code></span>
          <span><strong>Owner reference</strong><code>${escapeHtml(ownership.ownerReference)}</code></span>
          ${ownership.leafAdId ? `<span><strong>Leaf AD ID</strong><code>${escapeHtml(ownership.leafAdId)}</code></span>` : ""}
          <span><strong>Declared serials</strong><code>${escapeHtml(asset.serials)}</code></span>
          <span><strong>Nominal amount</strong><code>${escapeHtml(nominal)}</code></span>
          <span><strong>Metadata entries</strong><code>${escapeHtml(asset.metadataEntryCount)}</code></span>
          <span><strong>Tag 16</strong><code>${asset.tag16 === null ? "None" : escapeHtml(asset.tag16)}</code></span>
          <span><strong>Versions</strong><code>definition ${escapeHtml(cryptography.definitionVersion)} · crypto ${escapeHtml(cryptography.cryptoVersion)}</code></span>
          <span><strong>Policy flags</strong><code>${escapeHtml(cryptography.policyFlags)}</code></span>
        </div>
      </details>`,
    footer: `<button class="button" type="button" data-asset-import-action="reset">Cancel</button>
      <button class="button button-primary" type="button" data-asset-import-action="prepare">${icon("import")} Import asset</button>`
  });
}

function assetImportPreparedView(importState) {
  return assetImportPanel({
    body: `<div class="asset-import-result" role="status">
        <span class="result-icon is-settling">${icon("shield")}</span>
        <h3>Ready for native verification</h3>
        <p>The public package is prepared for <code>wallet.asset.import_asset</code>. This JavaScript design demo does not sign or mutate wallet state.</p>
      </div>
      <dl class="asset-import-result-fields">
        <div><dt>Native result</dt><dd><code>asset_id</code>, <code>serial_id</code>, <code>symbol</code>, <code>class</code></dd></div>
        <div><dt>Status</dt><dd><code>success</code> and <code>message</code></dd></div>
        <div><dt>Claim outcome</dt><dd><code>is_inserted</code> or <code>asset_already_exists</code></dd></div>
        <div><dt>Rejection</dt><dd>Explicit <code>IMPORT_*</code> reason; no partial success.</dd></div>
      </dl>`,
    footer: `<button class="button button-primary" type="button" data-asset-import-action="reset">${icon("backup")} Choose another package</button>`
  });
}

function walletImportView() {
  const importState = activeAssetImportState();
  const panel = importState.status === "ready" && importState.preview
    ? assetImportReviewView(importState)
    : importState.status === "prepared"
      ? assetImportPreparedView(importState)
      : assetImportIdleView(importState);
  return `<div class="view-enter asset-import-view">${panel}</div>`;
}

const MERGE_SPLIT_ASSET_FIXTURES = Object.freeze([
  Object.freeze({ id: "z42-a18f", definitionId: "z00z:main:coin", serialId: 42, amountAtomic: 1850, decimals: 2, symbol: "Z00Z", label: "Z00Z", status: "available" }),
  Object.freeze({ id: "z42-b72c", definitionId: "z00z:main:coin", serialId: 42, amountAtomic: 3225, decimals: 2, symbol: "Z00Z", label: "Z00Z", status: "available" }),
  Object.freeze({ id: "z42-c09d", definitionId: "z00z:main:coin", serialId: 42, amountAtomic: 475, decimals: 2, symbol: "Z00Z", label: "Z00Z", status: "available" }),
  Object.freeze({ id: "z42-lock", definitionId: "z00z:main:coin", serialId: 42, amountAtomic: 900, decimals: 2, symbol: "Z00Z", label: "Z00Z", status: "locked" }),
  Object.freeze({ id: "z43-f11a", definitionId: "z00z:main:coin", serialId: 43, amountAtomic: 12500, decimals: 2, symbol: "Z00Z", label: "Z00Z", status: "available" }),
  Object.freeze({ id: "b7-a410", definitionId: "external:ethereum:bold", serialId: 7, amountAtomic: 4000, decimals: 2, symbol: "BOLD", label: "wBOLD", status: "available" }),
  Object.freeze({ id: "b7-d299", definitionId: "external:ethereum:bold", serialId: 7, amountAtomic: 6000, decimals: 2, symbol: "BOLD", label: "wBOLD", status: "available" })
]);

function formatAtomicAmount(amountAtomic, decimals = 2, symbol = "") {
  const factor = 10 ** decimals;
  const whole = Math.floor(amountAtomic / factor);
  const fraction = String(amountAtomic % factor).padStart(decimals, "0");
  return `${whole.toLocaleString("en-US")}.${fraction}${symbol ? ` ${symbol}` : ""}`;
}

function parseAtomicAmount(value, decimals = 2) {
  const normalized = String(value || "").trim();
  const match = normalized.match(new RegExp(`^(\\d+)(?:\\.(\\d{0,${decimals}}))?$`));
  if (!match) return null;
  const factor = 10 ** decimals;
  const whole = Number(match[1]);
  const fraction = Number(String(match[2] || "").padEnd(decimals, "0"));
  const amount = whole * factor + fraction;
  return Number.isSafeInteger(amount) ? amount : null;
}

function mergeSplitGroups() {
  const groups = new Map();
  MERGE_SPLIT_ASSET_FIXTURES.forEach((asset) => {
    const key = `${asset.definitionId}:${asset.serialId}`;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(asset);
  });
  return [...groups.entries()]
    .map(([key, assets]) => ({
      key,
      assets,
      available: assets.filter(({ status }) => status === "available")
    }))
    .filter(({ available }) => available.length >= 2);
}

function resetMergeSplitState(mode = "merge") {
  const firstGroup = mergeSplitGroups()[0];
  const splitSource = MERGE_SPLIT_ASSET_FIXTURES.find(({ id }) => id === "z43-f11a")
    || MERGE_SPLIT_ASSET_FIXTURES.find(({ status }) => status === "available");
  state.assetMergeSplit = {
    walletId: activeWallet().id,
    mode,
    selectedMergeIds: firstGroup?.available.slice(0, 2).map(({ id }) => id) || [],
    selectedSplitId: splitSource?.id || "",
    splitAmounts: splitSource
      ? [
          formatAtomicAmount(Math.floor(splitSource.amountAtomic / 2), splitSource.decimals),
          formatAtomicAmount(splitSource.amountAtomic - Math.floor(splitSource.amountAtomic / 2), splitSource.decimals)
        ]
      : ["", ""],
    preview: null,
    error: ""
  };
  return state.assetMergeSplit;
}

function activeMergeSplitState() {
  if (!state.assetMergeSplit || state.assetMergeSplit.walletId !== activeWallet().id) {
    return resetMergeSplitState();
  }
  return state.assetMergeSplit;
}

function mergeSplitTabs(mode) {
  return `<div class="merge-split-tabs" role="tablist" aria-label="Merge or split assets">
    <button class="merge-split-tab${mode === "merge" ? " is-active" : ""}" type="button" role="tab" aria-selected="${mode === "merge"}" data-merge-split-mode="merge">${icon("merge-split")} <span><strong>Merge</strong><small>Many fragments → one</small></span></button>
    <button class="merge-split-tab${mode === "split" ? " is-active" : ""}" type="button" role="tab" aria-selected="${mode === "split"}" data-merge-split-mode="split">${icon("merge-split")} <span><strong>Split</strong><small>One fragment → many</small></span></button>
  </div>`;
}

function mergeSplitCompatibilityKey(asset) {
  return `<span class="merge-split-key"><span><small>Definition</small><code title="${escapeHtml(asset.definitionId)}">${escapeHtml(asset.definitionId)}</code></span><span><small>Serial ID</small><code>${asset.serialId}</code></span></span>`;
}

function mergeSplitPreviewMarkup(mergeSplitState) {
  const preview = mergeSplitState.preview;
  if (!preview) return "";
  const direction = preview.mode === "merge"
    ? `${preview.inputs.length} inputs → 1 output`
    : `1 input → ${preview.outputs.length} outputs`;
  const rows = preview.mode === "merge"
    ? preview.inputs.map((asset) => `<li><code>${escapeHtml(asset.id)}</code><strong>${escapeHtml(formatAtomicAmount(asset.amountAtomic, asset.decimals, asset.symbol))}</strong></li>`).join("")
    : preview.outputs.map((amountAtomic, index) => `<li><code>Output ${index + 1}</code><strong>${escapeHtml(formatAtomicAmount(amountAtomic, preview.asset.decimals, preview.asset.symbol))}</strong></li>`).join("");
  const asset = preview.mode === "merge" ? preview.inputs[0] : preview.asset;
  return `<section class="merge-split-preview" aria-labelledby="merge-split-preview-title">
    <div class="merge-split-preview-icon">${icon("shield")}</div>
    <p class="eyebrow">Wallet review</p>
    <h3 id="merge-split-preview-title">${preview.mode === "merge" ? "Merge" : "Split"} preview ready</h3>
    <p>${escapeHtml(direction)} · total ${escapeHtml(formatAtomicAmount(preview.totalAtomic, asset.decimals, asset.symbol))}</p>
    ${mergeSplitCompatibilityKey(asset)}
    <ul class="merge-split-preview-rows">${rows}</ul>
    <div class="confirmation-note">${icon("alert")} This demo prepares intent only. A native wallet must re-check ownership, availability, conservation, fees, authorization, submission, and reconciliation.</div>
    <footer class="merge-split-actions">
      <button class="button" type="button" data-merge-split-action="edit">Back</button>
      <button class="button button-primary" type="button" disabled aria-disabled="true">Native confirmation unavailable</button>
    </footer>
  </section>`;
}

function mergeModeMarkup(mergeSplitState) {
  const groups = mergeSplitGroups();
  const selectedAssets = MERGE_SPLIT_ASSET_FIXTURES.filter(({ id }) => mergeSplitState.selectedMergeIds.includes(id));
  const compatible = selectedAssets.length >= 2
    && selectedAssets.every(({ definitionId, serialId, status }) => (
      definitionId === selectedAssets[0].definitionId
      && serialId === selectedAssets[0].serialId
      && status === "available"
    ));
  const totalAtomic = selectedAssets.reduce((sum, { amountAtomic }) => sum + amountAtomic, 0);
  return `<section class="merge-split-mode-panel" role="tabpanel" aria-labelledby="merge-mode-title">
    <div class="merge-split-mode-heading">
      <div><p class="eyebrow">Compatible fragments</p><h3 id="merge-mode-title">Choose at least two inputs</h3><p>Only wallet-owned, available fragments with the same definition and base serial can be combined.</p></div>
      <span class="status-badge is-ready">${groups.length} groups</span>
    </div>
    <div class="merge-group-list">
      ${groups.map(({ key, assets, available }) => {
        const lead = available[0];
        return `<fieldset class="merge-group" data-merge-group="${escapeHtml(key)}">
          <legend><span><strong>${escapeHtml(lead.label)}</strong><small>${available.length} available fragment${available.length === 1 ? "" : "s"}</small></span>${mergeSplitCompatibilityKey(lead)}</legend>
          <div class="merge-fragment-list">
            ${assets.map((asset) => {
              const unavailable = asset.status !== "available";
              const checked = mergeSplitState.selectedMergeIds.includes(asset.id);
              return `<label class="merge-fragment${unavailable ? " is-unavailable" : ""}">
                <input type="checkbox" data-merge-fragment-id="${escapeHtml(asset.id)}"${checked ? " checked" : ""}${unavailable ? " disabled" : ""}>
                <span class="merge-fragment-check">${icon("check")}</span>
                <span><strong>${escapeHtml(formatAtomicAmount(asset.amountAtomic, asset.decimals, asset.symbol))}</strong><code>${escapeHtml(asset.id)}</code></span>
                <small>${unavailable ? "Locked" : "Available"}</small>
              </label>`;
            }).join("")}
          </div>
        </fieldset>`;
      }).join("")}
    </div>
    <div class="merge-split-summary" aria-live="polite">
      <span><small>Selected</small><strong>${selectedAssets.length} input${selectedAssets.length === 1 ? "" : "s"}</strong></span>
      <span><small>Total output</small><strong>${selectedAssets[0] ? escapeHtml(formatAtomicAmount(totalAtomic, selectedAssets[0].decimals, selectedAssets[0].symbol)) : "—"}</strong></span>
      <span><small>Series</small><strong>${compatible ? `#${selectedAssets[0].serialId} preserved` : "Select one compatible group"}</strong></span>
    </div>
    ${mergeSplitState.error ? `<p class="merge-split-error" role="alert">${icon("alert")} ${escapeHtml(mergeSplitState.error)}</p>` : ""}
    <div class="merge-split-actions"><button class="button button-primary" type="button" data-merge-split-action="preview-merge"${compatible ? "" : " disabled"}>Preview merge</button></div>
  </section>`;
}

function splitModeMarkup(mergeSplitState) {
  const sources = MERGE_SPLIT_ASSET_FIXTURES.filter(({ status, amountAtomic }) => status === "available" && amountAtomic > 1);
  const source = sources.find(({ id }) => id === mergeSplitState.selectedSplitId) || sources[0];
  const parsed = source
    ? mergeSplitState.splitAmounts.map((value) => parseAtomicAmount(value, source.decimals))
    : [];
  const validParts = parsed.length >= 2 && parsed.every((value) => Number.isSafeInteger(value) && value > 0);
  const sum = validParts ? parsed.reduce((total, value) => total + value, 0) : 0;
  const exact = Boolean(source && validParts && sum === source.amountAtomic);
  return `<section class="merge-split-mode-panel" role="tabpanel" aria-labelledby="split-mode-title">
    <div class="merge-split-mode-heading">
      <div><p class="eyebrow">Source fragment</p><h3 id="split-mode-title">Allocate the full amount</h3><p>Every output keeps the source definition and base serial. Amounts must be positive and sum exactly to the input.</p></div>
    </div>
    <label class="split-source-control"><span>Source asset</span><select data-split-source aria-label="Source asset">${sources.map((asset) => `<option value="${escapeHtml(asset.id)}"${asset.id === source?.id ? " selected" : ""}>${escapeHtml(`${asset.label} · serial #${asset.serialId} · ${formatAtomicAmount(asset.amountAtomic, asset.decimals, asset.symbol)}`)}</option>`).join("")}</select></label>
    ${source ? `<div class="split-source-card"><span class="merge-split-asset-icon">${icon("coin")}</span><span><small>Available input</small><strong>${escapeHtml(formatAtomicAmount(source.amountAtomic, source.decimals, source.symbol))}</strong><code>${escapeHtml(source.id)}</code></span>${mergeSplitCompatibilityKey(source)}</div>` : ""}
    <div class="split-allocation-heading"><span><strong>Outputs</strong><small>2–8 positive amounts</small></span><button class="button button-quiet" type="button" data-merge-split-action="add-output"${mergeSplitState.splitAmounts.length >= 8 ? " disabled" : ""}>${icon("plus")} Add output</button></div>
    <div class="split-allocation-list">
      ${mergeSplitState.splitAmounts.map((value, index) => `<label class="split-allocation"><span>Output ${index + 1}</span><span class="amount-input"><input inputmode="decimal" autocomplete="off" value="${escapeHtml(value)}" data-split-amount-index="${index}" aria-label="Output ${index + 1} amount"><span>${escapeHtml(source?.symbol || "")}</span></span><button class="icon-button" type="button" aria-label="Remove output ${index + 1}" data-merge-split-action="remove-output" data-output-index="${index}"${mergeSplitState.splitAmounts.length <= 2 ? " disabled" : ""}>${icon("close")}</button></label>`).join("")}
    </div>
    <div class="merge-split-summary" aria-live="polite">
      <span><small>Input</small><strong>${source ? escapeHtml(formatAtomicAmount(source.amountAtomic, source.decimals, source.symbol)) : "—"}</strong></span>
      <span><small>Outputs</small><strong>${validParts && source ? escapeHtml(formatAtomicAmount(sum, source.decimals, source.symbol)) : "Check amounts"}</strong></span>
      <span><small>Conservation</small><strong class="${exact ? "is-valid" : "is-invalid"}">${exact ? "Exact" : "Must equal input"}</strong></span>
    </div>
    ${mergeSplitState.error ? `<p class="merge-split-error" role="alert">${icon("alert")} ${escapeHtml(mergeSplitState.error)}</p>` : ""}
    <div class="merge-split-actions"><button class="button button-primary" type="button" data-merge-split-action="preview-split"${exact ? "" : " disabled"}>Preview split</button></div>
  </section>`;
}

function walletMergeSplitView() {
  const mergeSplitState = activeMergeSplitState();
  const content = mergeSplitState.preview
    ? mergeSplitPreviewMarkup(mergeSplitState)
    : `${mergeSplitTabs(mergeSplitState.mode)}${mergeSplitState.mode === "merge" ? mergeModeMarkup(mergeSplitState) : splitModeMarkup(mergeSplitState)}`;
  return `<div class="view-enter merge-split-view">
    <section class="merge-split-shell" aria-labelledby="merge-split-title">
      <header class="wallet-action-header merge-split-header">
        <span class="asset-import-header-icon">${icon("merge-split")}</span>
        <div><h2 id="merge-split-title">Merge/Split assets</h2><p>Recompose spendable fragments in ${escapeHtml(activeWallet().name)} without changing their definition or base serial.</p></div>
      </header>
      <div class="merge-split-body">${content}</div>
      <div class="capability-note merge-split-boundary">${icon("alert")} <span><strong>Compatibility boundary</strong><small>The current <code>wallet.asset.merge_assets</code> and <code>split_asset</code> helpers do not claim canonical ledger reconciliation authority. This screen stops at review.</small></span></div>
    </section>
  </div>`;
}

const walletSections = [
  { key: "assets", labelKey: "assets.sectionAssets", iconName: "assets" },
  { key: "vouchers", labelKey: "assets.sectionVouchers", iconName: "voucher-list" },
  { key: "permissions", labelKey: "assets.sectionPermissions", iconName: "permission-list" }
];

function walletContextNav() {
  return `<nav class="context-nav context-tab-list" role="tablist" aria-label="${t("assets.sections")}">${walletSections.map(({ key, labelKey, iconName, objectFamily }) => {
    const active = state.walletSection === key;
    return `
    <button id="wallet-section-${key}" class="context-nav-item${active ? " is-active" : ""}" type="button" role="tab" aria-selected="${active}" aria-controls="wallet-sections-panel" tabindex="${active ? "0" : "-1"}" ${active ? 'aria-current="page"' : ""} data-wallet-section="${key}">
      ${objectFamily ? objectFamilyIcon(objectFamily) : icon(iconName)}<span><strong>${t(labelKey)}</strong></span>
    </button>`;
  }).join("")}</nav>`;
}

function vouchersPanel() {
  const vouchers = activeWallet().vouchers || [];
  if (!vouchers.length) {
    return `<div class="object-empty-state">
      <span class="list-icon is-voucher">${objectFamilyIcon("voucher")}</span>
      <h2>${t("assets.noVouchers")}</h2>
      <button class="button button-primary" type="button" data-open-flow="create-voucher">${icon("plus")} ${t("assets.createVoucher")}</button>
    </div>`;
  }
  return `
    <div class="choice-strip" aria-label="Voucher filters"><button class="choice-chip is-active" type="button">Needs action</button><button class="choice-chip" type="button">Redeemable</button><button class="choice-chip" type="button">History</button><button class="choice-chip" type="button">Quarantined</button></div>
    <div class="claim-list">
      ${vouchers.map((voucher) => `<button class="claim-row" type="button" data-open-flow="${escapeHtml(voucher.detailFlow || "voucher-detail")}" data-object-id="${escapeHtml(voucher.id)}">${objectTypeIcon("voucher", voucher.kind, "list-icon")}<span class="list-copy"><strong>${escapeHtml(voucher.title)}</strong></span><span class="list-meta"><strong>${escapeHtml(voucher.value)}</strong><small class="status-badge is-${escapeHtml(voucher.tone)}">${escapeHtml(voucher.status)}</small></span></button>`).join("")}
    </div>`;
}

const permissionDetails = Object.freeze({
  receipt: Object.freeze({
    title: "Delivery receipt access",
    subtitle: "Held data-access permission",
    remaining: "2 of 5 uses",
    classLabel: "Data access",
    action: "View receipt",
    scope: "receipts.example",
    delegation: "Forbidden",
    expiry: "31 Jul 2026",
    rightId: "right_54ac…1f88",
    kind: "data_access"
  }),
  deploy: Object.freeze({
    title: "Deploy to staging",
    subtitle: "Held machine-capability permission",
    remaining: "1 use",
    classLabel: "Machine capability",
    action: "Deploy",
    scope: "staging.example",
    delegation: "Attenuation only",
    expiry: "19 Aug 2026",
    rightId: "right_8d9e…4a62",
    kind: "machine_capability"
  })
});

function permissionsPanel() {
  const permissions = activeWallet().permissions || [];
  if (!permissions.length) {
    return `<div class="object-empty-state">
      <span class="list-icon is-right">${objectFamilyIcon("right")}</span>
      <h2>${t("assets.noPermissions")}</h2>
      <button class="button button-primary" type="button" data-open-flow="create-permission">${icon("plus")} ${t("assets.createPermission")}</button>
    </div>`;
  }
  return `
    <div class="choice-strip" aria-label="Permission filters"><button class="choice-chip is-active" type="button">Held</button><button class="choice-chip" type="button">Delegated</button><button class="choice-chip" type="button">Used</button></div>
    <div class="permission-list">
      ${permissions.map((permission) => `<button class="permission-row" type="button" data-open-flow="permission-detail" data-permission-id="${escapeHtml(permission.id)}">${objectTypeIcon("right", permission.kind, "list-icon")}<span class="list-copy"><strong>${escapeHtml(permission.title)}</strong></span><span class="list-meta"><strong>${escapeHtml(permission.remaining)}</strong><small class="status-badge is-${escapeHtml(permission.tone)}">${escapeHtml(permission.status)}</small></span></button>`).join("")}
    </div>`;
}

function walletView() {
  const panel = state.walletSection === "assets" ? moneyView() : state.walletSection === "vouchers" ? vouchersPanel() : permissionsPanel();
  return `<div class="view-enter workspace-layout wallet-assets-layout"><aside class="context-rail">${walletContextNav()}</aside><div id="wallet-sections-panel" class="workspace-panel" role="tabpanel" aria-labelledby="wallet-section-${state.walletSection}">${dappWalletReviewNotice()}${panel}</div></div>`;
}

function statusText(status) {
  const key = {
    settling: "history.settling",
    settled: "history.settled",
    active: "history.active",
    attention: "history.needsAttention"
  }[status] || "history.ready";
  return t(key);
}

function activityText(item, field) {
  const key = item[`${field}Key`];
  if (!key) return item[field] || "";
  const values = { ...item[`${field}Values`] };
  Object.entries(item[`${field}ValueKeys`] || {}).forEach(([name, valueKey]) => {
    values[name] = t(valueKey);
  });
  return t(key, values);
}

function activityRows(items, compact = false) {
  if (!items.length) {
    return `<div class="empty-state"><span class="list-icon">${icon("search")}</span><h3>${t("history.noMatching")}</h3><p>${t("history.tryAnother")}</p></div>`;
  }

  return items.map((item) => {
    const iconName = item.id.startsWith("claim-") ? "claim" : item.type === "security" ? "backup" : item.direction === "in" ? "receive" : "send";
    const iconMarkup = item.type === "voucher" ? objectFamilyIcon("voucher") : item.type === "permission" ? objectFamilyIcon("right") : icon(iconName);
    const iconClass = item.direction === "in" ? "is-incoming" : item.direction === "out" ? "is-outgoing" : "";
    const amountClass = item.direction === "in" ? "positive" : item.direction === "out" ? "negative" : "";
    return `
      <button class="activity-row" type="button" data-open-activity="${escapeHtml(item.id)}">
        <span class="activity-icon ${iconClass}">${iconMarkup}</span>
        <span class="activity-copy"><strong>${escapeHtml(activityText(item, "title"))}</strong><small><span class="activity-detail">${escapeHtml(activityText(item, "detail"))}${compact ? ` · ${escapeHtml(activityText(item, "time"))}` : ""}</span>${compact ? "" : `<span class="status-badge is-${escapeHtml(item.status)}">${statusText(item.status)}</span>`}</small></span>
        <span class="activity-value"><strong class="${amountClass}">${escapeHtml(activityText(item, "amount"))}</strong><small>${escapeHtml(activityText(item, "time"))}</small></span>
      </button>`;
  }).join("");
}

function matchesActivityFilter(item, filter) {
  if (filter === "all") return true;
  if (filter === "asset") return item.type === "asset" || item.type === "money";
  return item.type === filter;
}

function activityView() {
  const visible = activeWallet().activities.filter((item) => matchesActivityFilter(item, state.activityFilter));

  const filters = [
    ["all", "history.all"], ["asset", "history.assets"], ["voucher", "history.vouchers"], ["permission", "history.permissions"], ["security", "history.system"]
  ].map(([value, labelKey]) => `<button class="choice-chip${state.activityFilter === value ? " is-active" : ""}" type="button" data-filter="${value}">${t(labelKey)}</button>`).join("");

  return `
    <div class="view-enter">
      <div class="filter-bar choice-strip" aria-label="${t("history.filters")}">
        ${filters}
        <label class="search-wrap"><span class="sr-only">${t("history.search")}</span>${icon("search")}<input id="activity-search" type="search" placeholder="${t("history.search")}" autocomplete="off"></label>
      </div>
      <section class="activity-card-list" id="activity-results" aria-label="${t("history.results")}">
        ${activityRows(visible)}
      </section>
    </div>`;
}

function swapView() {
  const asset = supportedAsset("z00z");
  return `
    <div class="view-enter wallet-tool-view">
      <section class="wallet-tool-grid wallet-tool-grid-single wallet-tool-grid-centered">
        <article class="card wallet-tool-card swap-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("swap")}</span><div><h2>Build a swap</h2></div></div>
          <div class="form-grid">
            <div class="field-group"><label class="field-label" for="swap-from">From</label><select id="swap-from">${assetOptions("z00z")}</select><p class="field-hint">Available: ${sensitive(`${asset.balance} ${asset.unit}`)}</p></div>
            <div class="field-group"><label class="field-label" for="swap-amount">Amount</label><div class="input-with-affix"><input id="swap-amount" type="number" min="0.01" max="${escapeHtml(asset.balance.replaceAll(",", ""))}" step="0.01" inputmode="decimal" placeholder="0.00"><span class="input-affix">Z00Z</span></div></div>
            <div class="field-group"><label class="field-label" for="swap-to">To</label><select id="swap-to">${assetOptions("dai")}</select></div>
            <button class="button button-primary" type="button" data-demo-action="preview-swap">${icon("swap")} Preview experimental recipe</button>
          </div>
        </article>
      </section>
    </div>`;
}

function defaultExchangeDraft() {
  return {
    step: 0,
    providerId: "near-intents",
    sourceAssetKey: "z00z",
    amount: "",
    destinationId: demoRuntime.exchangeProvider("near-intents").defaultDestination,
    orderType: "market",
    limitPrice: "",
    recipient: "",
    refundAddress: "",
    slippageBps: "100",
    deadlineMinutes: "5"
  };
}

function activeExchangeDraft() {
  const wallet = activeWallet();
  state.exchangeDrafts ||= {};
  state.exchangeDrafts[wallet.id] ||= defaultExchangeDraft();
  const draft = state.exchangeDrafts[wallet.id];
  const provider = demoRuntime.exchangeProvider(draft.providerId);
  if (!walletAssetEntries().some(({ key }) => key === draft.sourceAssetKey)) draft.sourceAssetKey = walletAssetEntries()[0]?.key || "z00z";
  if (!demoRuntime.exchangeDestinations(provider.id).some(({ id }) => id === draft.destinationId)) {
    draft.destinationId = provider.defaultDestination;
  }
  return draft;
}

function exchangeProviderContextNav(draft) {
  return `<nav class="context-nav context-tab-list exchange-provider-context" role="radiogroup" aria-label="${t("exchange.executionModel")}">${Object.values(demoRuntime.EXCHANGE_PROVIDER_LUT).map((provider) => `
    <button class="context-nav-item${draft.providerId === provider.id ? " is-active" : ""}" type="button" role="radio" aria-checked="${draft.providerId === provider.id}" data-exchange-provider="${provider.id}">
      ${icon(provider.iconName)}<span><strong>${t(provider.labelKey)}</strong></span>
    </button>`).join("")}</nav>`;
}

function exchangeDestinationOptions(draft) {
  return demoRuntime.exchangeDestinations(draft.providerId).map((destination) => `<option value="${escapeHtml(destination.id)}"${draft.destinationId === destination.id ? " selected" : ""}>${escapeHtml(destination.label)} · ${escapeHtml(destination.network)}</option>`).join("");
}

function exchangeProviderFields(draft) {
  if (draft.providerId === "hyperliquid") {
    return `
      <div class="field-group"><label class="field-label" for="exchange-order-type">${t("exchange.orderType")}</label><select id="exchange-order-type" name="orderType"><option value="market"${draft.orderType === "market" ? " selected" : ""}>${t("exchange.market")}</option><option value="limit"${draft.orderType === "limit" ? " selected" : ""}>${t("exchange.limit")}</option></select></div>
      ${draft.orderType === "limit" ? `<div class="field-group"><label class="field-label" for="exchange-limit-price">${t("exchange.limitPrice")}</label><input id="exchange-limit-price" name="limitPrice" type="number" min="0" step="any" inputmode="decimal" value="${escapeHtml(draft.limitPrice)}" placeholder="${t("exchange.limitPricePlaceholder")}"></div>` : ""}`;
  }
  return `
    <div class="field-group"><label class="field-label" for="exchange-recipient">${t("exchange.recipient")}</label><input id="exchange-recipient" name="recipient" value="${escapeHtml(draft.recipient)}" placeholder="${t("exchange.recipientPlaceholder")}" autocomplete="off"></div>
    <div class="field-group"><label class="field-label" for="exchange-refund">${t("exchange.refundAddress")}</label><input id="exchange-refund" name="refundAddress" value="${escapeHtml(draft.refundAddress)}" placeholder="${t("exchange.refundPlaceholder")}" autocomplete="off"></div>
    <div class="exchange-control-grid">
      <div class="field-group"><label class="field-label" for="exchange-slippage">${t("exchange.slippage")}</label><select id="exchange-slippage" name="slippageBps"><option value="50"${draft.slippageBps === "50" ? " selected" : ""}>0.5%</option><option value="100"${draft.slippageBps === "100" ? " selected" : ""}>1%</option><option value="200"${draft.slippageBps === "200" ? " selected" : ""}>2%</option></select></div>
      <div class="field-group"><label class="field-label" for="exchange-deadline">${t("exchange.deadline")}</label><select id="exchange-deadline" name="deadlineMinutes"><option value="3"${draft.deadlineMinutes === "3" ? " selected" : ""}>3 min</option><option value="5"${draft.deadlineMinutes === "5" ? " selected" : ""}>5 min</option><option value="10"${draft.deadlineMinutes === "10" ? " selected" : ""}>10 min</option></select></div>
    </div>`;
}

function captureExchangeDraft(form) {
  const draft = activeExchangeDraft();
  const fields = ["sourceAssetKey", "amount", "destinationId", "orderType", "limitPrice", "recipient", "refundAddress", "slippageBps", "deadlineMinutes"];
  fields.forEach((name) => {
    if (form.elements[name]) draft[name] = form.elements[name].value.trim();
  });
  return draft;
}

function exchangeReview(draft) {
  const provider = demoRuntime.exchangeProvider(draft.providerId);
  const source = supportedAsset(draft.sourceAssetKey);
  const destination = demoRuntime.EXCHANGE_DESTINATION_LUT[draft.destinationId];
  const providerRows = draft.providerId === "hyperliquid"
    ? `<div class="summary-row"><span>${t("exchange.pair")}</span><strong>${escapeHtml(source.unit)}/${escapeHtml(destination.unit)}</strong></div>
       <div class="summary-row"><span>${t("exchange.orderType")}</span><strong>${t(`exchange.${draft.orderType}`)}</strong></div>
       ${draft.orderType === "limit" ? `<div class="summary-row"><span>${t("exchange.limitPrice")}</span><strong>${escapeHtml(draft.limitPrice)}</strong></div>` : ""}`
    : `<div class="summary-row"><span>${t("exchange.route")}</span><strong>${escapeHtml(walletChain(activeWallet().chainId).label)} → ${escapeHtml(destination.network)}</strong></div>
       <div class="summary-row"><span>${t("exchange.mode")}</span><strong>${t("exchange.exactInput")}</strong></div>
       <div class="summary-row"><span>${t("exchange.recipient")}</span><strong>${escapeHtml(draft.recipient)}</strong></div>
       <div class="summary-row"><span>${t("exchange.refundAddress")}</span><strong>${escapeHtml(draft.refundAddress)}</strong></div>
       <div class="summary-row"><span>${t("exchange.slippage")}</span><strong>${Number(draft.slippageBps) / 100}%</strong></div>
       <div class="summary-row"><span>${t("exchange.deadline")}</span><strong>${escapeHtml(draft.deadlineMinutes)} min</strong></div>`;
  return `<article class="card wallet-tool-card exchange-card">
    <div class="tool-card-heading"><span class="list-icon">${icon(provider.iconName)}</span><div><h2>${t("exchange.reviewTitle")}</h2></div></div>
    <div class="review-card">
      <div class="summary-row"><span>${t("exchange.executionModel")}</span><strong>${t(provider.labelKey)}</strong></div>
      <div class="summary-row"><span>${t("exchange.from")}</span><strong>${escapeHtml(draft.amount)} ${escapeHtml(source.unit)}</strong></div>
      <div class="summary-row"><span>${t("exchange.to")}</span><strong>${escapeHtml(destination.label)} · ${escapeHtml(destination.network)}</strong></div>
      ${providerRows}
    </div>
    <div class="exchange-unavailable-grid">
      <div><span>${t("exchange.rate")}</span><strong>${t("common.unavailable")}</strong></div>
      <div><span>${t("exchange.expectedOutput")}</span><strong>${t("common.unavailable")}</strong></div>
      <div><span>${t("exchange.minimumReceived")}</span><strong>${t("common.unavailable")}</strong></div>
      <div><span>${t("exchange.fee")}</span><strong>${t("common.unavailable")}</strong></div>
      <div><span>${t("exchange.eta")}</span><strong>${t("common.unavailable")}</strong></div>
      ${draft.providerId === "near-intents"
        ? `<div><span>${t("exchange.depositAddress")}</span><strong>${t("common.unavailable")}</strong></div><div><span>${t("exchange.executionStatus")}</span><strong>${t("common.unavailable")}</strong></div>`
        : `<div><span>${t("exchange.marketState")}</span><strong>${t("common.unavailable")}</strong></div>`}
    </div>
    <div class="confirmation-note">${icon("shield")} ${t("exchange.connectorBoundary")}</div>
    <div class="wallet-tool-actions"><button class="button" type="button" data-exchange-action="back">${t("common.back")}</button><button class="button button-primary" type="button" data-exchange-action="new">${t("exchange.editRequest")}</button></div>
  </article>`;
}

function exchangeView() {
  const draft = activeExchangeDraft();
  const asset = supportedAsset(draft.sourceAssetKey);
  const panel = draft.step === 1
    ? `<section class="wallet-tool-grid wallet-tool-grid-single wallet-tool-grid-centered">${exchangeReview(draft)}</section>`
    : `<section class="wallet-tool-grid wallet-tool-grid-single wallet-tool-grid-centered">
        <article class="card wallet-tool-card exchange-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("exchange")}</span><div><h2>${t("exchange.title")}</h2></div></div>
          <form class="form-grid" id="exchange-entry" autocomplete="off" novalidate>
            <div class="field-group"><label class="field-label" for="exchange-source">${t("exchange.sourceAsset")}</label><select id="exchange-source" name="sourceAssetKey">${assetOptions(draft.sourceAssetKey)}</select><p class="field-hint">${t("send.available", { value: sensitive(`${asset.balance} ${asset.unit}`) })}</p></div>
            <div class="field-group"><label class="field-label" for="exchange-amount">${t("exchange.amount")}</label><div class="input-with-affix"><input id="exchange-amount" name="amount" type="number" min="${asset.divisible ? "0.01" : "1"}" max="${escapeHtml(asset.balance.replaceAll(",", ""))}" step="${asset.divisible ? "0.01" : "1"}" inputmode="decimal" value="${escapeHtml(draft.amount)}" placeholder="0.00" aria-describedby="exchange-error" required><span class="input-affix">${escapeHtml(asset.unit)}</span></div></div>
            <div class="field-group"><label class="field-label" for="exchange-destination">${t("exchange.destinationAsset")}</label><select id="exchange-destination" name="destinationId">${exchangeDestinationOptions(draft)}</select></div>
            ${exchangeProviderFields(draft)}
            <p class="field-error" id="exchange-error" role="alert"></p>
            <button class="button button-primary" type="submit">${icon("exchange")} Review target request</button>
          </form>
        </article>
      </section>`;
  return `<div class="view-enter workspace-layout exchange-workspace-layout">
    <aside class="context-rail">${exchangeProviderContextNav(draft)}</aside>
    <div class="workspace-panel wallet-tool-view">
      ${panel}
    </div>
  </div>`;
}

function stakingView() {
  const wallet = activeWallet();
  const summary = wallet.summary;
  const isUnstake = state.activeRoute === "wallet.staking.unstake";
  const panel = isUnstake
    ? `
      <section class="wallet-tool-grid wallet-tool-grid-single wallet-tool-grid-centered">
        <article class="card wallet-tool-card staking-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("restore")}</span><div><h2>${t("navigation.unstake")}</h2></div></div>
          <div class="staking-summary" aria-label="${t("staking.totals")}">
            <div class="staking-metric"><span>${t("staking.staked")}</span><strong>${t("common.unavailable")}</strong></div>
            <div class="staking-metric"><span>${t("staking.rewards")}</span><strong>${t("common.unavailable")}</strong></div>
          </div>
          <div class="form-grid">
            <div class="field-group"><label class="field-label" for="unstake-position">${t("staking.staked")}</label><select id="unstake-position"><option>${t("staking.nothingDelegated")}</option></select></div>
            <div class="field-group"><label class="field-label" for="unstake-amount">${t("staking.amount")}</label><div class="input-with-affix"><input id="unstake-amount" type="number" min="0.01" step="0.01" inputmode="decimal" placeholder="0.00"><span class="input-affix">Z00Z</span></div><p class="field-hint">${t("staking.notice")}</p></div>
            <button class="button button-primary" type="button" data-demo-action="prepare-unstake">${icon("restore")} ${t("navigation.unstake")}</button>
          </div>
        </article>
      </section>`
    : `
      <section class="wallet-tool-grid wallet-tool-grid-single wallet-tool-grid-centered">
        <article class="card wallet-tool-card staking-card">
          <div class="tool-card-heading"><span class="list-icon">${icon("earn")}</span><div><h2>${t("staking.prepare")}</h2></div></div>
          <div class="staking-summary" aria-label="${t("staking.totals")}">
            <div class="staking-metric"><span>${t("staking.staked")}</span><strong>${t("common.unavailable")}</strong></div>
            <div class="staking-metric"><span>${t("staking.rewards")}</span><strong>${t("common.unavailable")}</strong></div>
          </div>
          <div class="form-grid">
            <div class="field-group"><label class="field-label" for="stake-amount">${t("staking.amount")}</label><div class="input-with-affix"><input id="stake-amount" type="number" min="0.01" max="${escapeHtml(summary.available.replaceAll(",", ""))}" step="0.01" inputmode="decimal" placeholder="0.00"><span class="input-affix">Z00Z</span></div><p class="field-hint">${t("staking.availableBalance", { value: sensitive(`${summary.available} Z00Z`) })}</p></div>
            <div class="field-group"><label class="field-label" for="stake-validator">${t("staking.validator")}</label><select id="stake-validator"><option>${t("staking.validatorPlaceholder")}</option></select></div>
            <button class="button button-primary" type="button" data-demo-action="prepare-stake">${icon("earn")} ${t("staking.review")}</button>
          </div>
        </article>
      </section>`;
  return workspaceFrame("wallet.staking", `<div class="wallet-tool-view">
    ${panel}
  </div>`);
}

function walletBackupView() {
  return `
    <div class="view-enter wallet-tool-view">
      <section class="wallet-tool-grid wallet-tool-grid-single wallet-tool-grid-centered">
        <article class="card wallet-tool-card backup-card">
          <div class="tool-card-heading backup-card-heading"><span class="list-icon">${icon("backup")}</span><div><h2>Backup status</h2></div></div>
          <div class="review-card backup-summary"><div class="summary-row"><span>Latest backup</span><strong>10 Jul 2026 · 09:42</strong></div><div class="summary-row"><span>Integrity</span><strong class="trust-label">${icon("shield")} Verified</strong></div><div class="summary-row"><span>Destination</span><strong>Encrypted local file</strong></div></div>
          <div class="wallet-backup-actions">
            <button class="button button-primary button-full wallet-backup-action" type="button" data-demo-action="backup">${icon("backup")} Create fresh backup</button>
            <button class="button button-full wallet-backup-recovery" type="button" data-demo-action="restore">${icon("restore")} Recover from backup</button>
          </div>
        </article>
      </section>
    </div>`;
}

const walletSettingsMeta = {
  general: ["General", "settings"],
  security: ["Security", "shield"],
  backup: ["Backup", "backup"],
  policies: ["Policies", "permission"],
  advanced: ["Advanced", "advanced"]
};

function isMobileNavigation() {
  return window.matchMedia("(max-width: 768px)").matches;
}

function closeDesktopWalletPicker({ restoreFocus = false } = {}) {
  if (!walletPickerPopup || walletPickerPopup.hidden) return;
  const trigger = desktopWalletPickerTrigger;
  walletPickerPopup.hidden = true;
  walletPickerPopup.replaceChildren();
  walletPickerPopup.removeAttribute("style");
  desktopWalletPickerTrigger = null;
  trigger?.setAttribute("aria-expanded", "false");
  if (restoreFocus && trigger?.isConnected) trigger.focus();
}

function closeMobilePopup({ restoreFocus = false } = {}) {
  if (!mobilePopupMenu || mobilePopupMenu.hidden) return;
  const navigationScrollRegion = mobilePopupMenu.querySelector(".mobile-navigation-scroll-region");
  if (navigationScrollRegion) {
    captureNavigationScrollPosition("mobile", navigationScrollRegion.scrollTop);
  }
  clearMobileDrawerMotion();
  const trigger = mobilePopupTrigger;
  mobilePopupMenu.hidden = true;
  mobilePopupMenu.innerHTML = "";
  mobilePopupMenu.removeAttribute("data-popup-type");
  mobilePopupMenu.setAttribute("role", "dialog");
  mobilePopupMenu.removeAttribute("aria-modal");
  mobileMenuBackdrop.hidden = true;
  document.body.classList.remove("has-mobile-drawer");
  appBody.inert = false;
  mergeShellState({ type: "set_drawer", open: false });
  mobilePopupType = "";
  mobilePopupTrigger = null;
  trigger?.setAttribute("aria-expanded", "false");
  mobileMenuButton.setAttribute("aria-expanded", "false");
  if (restoreFocus && trigger?.isConnected) trigger.focus();
}

function openDesktopWalletPicker(trigger) {
  if (!walletPickerPopup.hidden && desktopWalletPickerTrigger === trigger) {
    closeDesktopWalletPicker({ restoreFocus: true });
    return;
  }
  closeDesktopWalletPicker();
  if (!isMobileNavigation()) closeMobilePopup();
  desktopWalletPickerTrigger = trigger;
  walletPickerPopup.innerHTML = walletPickerPopupMarkup();
  walletPickerPopup.hidden = false;
  trigger.setAttribute("aria-expanded", "true");
  requestAnimationFrame(() => {
    const triggerRect = trigger.getBoundingClientRect();
    const anchorRect = trigger.closest(".mobile-wallet-selector, .wallet-nav-viewport")?.getBoundingClientRect() || triggerRect;
    const viewportPadding = isMobileNavigation() ? 8 : 12;
    const maxWidth = isMobileNavigation() ? 300 : 288;
    const minWidth = isMobileNavigation() ? 240 : 252;
    const width = Math.min(Math.max(anchorRect.width, minWidth), maxWidth, window.innerWidth - viewportPadding * 2);
    const left = Math.max(viewportPadding, Math.min(triggerRect.left, window.innerWidth - width - viewportPadding));
    const spaceBelow = window.innerHeight - triggerRect.bottom - viewportPadding;
    const spaceAbove = triggerRect.top - viewportPadding;
    const popupHeight = Math.min(walletPickerPopup.scrollHeight, 280);
    const opensUpward = spaceBelow < Math.min(popupHeight, 176) && spaceAbove > spaceBelow;
    const availableHeight = Math.max(156, opensUpward ? spaceAbove : spaceBelow);
    walletPickerPopup.style.left = `${Math.round(left)}px`;
    walletPickerPopup.style.width = `${Math.round(width)}px`;
    walletPickerPopup.style.maxHeight = `${Math.floor(availableHeight)}px`;
    if (opensUpward) {
      walletPickerPopup.style.top = "auto";
      walletPickerPopup.style.bottom = `${Math.max(viewportPadding, Math.round(window.innerHeight - triggerRect.top + 8))}px`;
    } else {
      walletPickerPopup.style.top = `${Math.round(triggerRect.bottom + 8)}px`;
      walletPickerPopup.style.bottom = "auto";
    }
    walletPickerPopup.querySelector(".wallet-picker-choice.is-active")?.focus();
  });
}

function openWalletPicker(trigger) {
  closeLanguagePickers();
  closeSelectPickers();
  openDesktopWalletPicker(trigger);
}

function selectWalletFromPicker(walletId) {
  const walletRouteCompatible = demoRuntime.isWalletRoute(state.activeRoute);
  clearExternalReviewHandoffs();
  state.selectedWalletId = walletId;
  mergeShellState({ type: "switch_wallet", walletId, walletRouteCompatible });
  Object.assign(state, legacyStateForRoute(state.activeRoute));
  closeDesktopWalletPicker();
  render({ focusMain: true });
}

function focusMobileDrawer({ preventScroll = false } = {}) {
  mobilePopupMenu.querySelector(
    "[data-wallet-picker-trigger], [data-wallet-picker-action='add-wallet'], button:not([disabled])"
  )?.focus({ preventScroll });
}

function openMobilePopup(trigger = mobileMenuButton, { isSwipePreview = false } = {}) {
  if (!isMobileNavigation()) return;
  if (!isSwipePreview && !mobilePopupMenu.hidden && mobilePopupType === "menu" && mobilePopupTrigger === trigger) {
    closeMobilePopup({ restoreFocus: true });
    return;
  }
  if (!mobilePopupMenu.hidden) closeMobilePopup();
  clearMobileDrawerMotion();
  closeDesktopWalletPicker();
  mobilePopupType = "menu";
  mobilePopupTrigger = trigger;
  mobilePopupMenu.innerHTML = mobileNavigationDrawerMarkup();
  enhanceNativeSelects(mobilePopupMenu);
  mobilePopupMenu.dataset.popupType = "menu";
  mobilePopupMenu.setAttribute("role", "dialog");
  mobilePopupMenu.setAttribute("aria-modal", "true");
  mobileMenuBackdrop.hidden = false;
  document.body.classList.add("has-mobile-drawer");
  appBody.inert = true;
  mergeShellState({ type: "set_drawer", open: true });
  mobilePopupMenu.hidden = false;
  trigger.setAttribute("aria-expanded", "true");
  if (isSwipePreview) {
    mobilePopupMenu.classList.add("is-swipe-dragging");
    mobileMenuBackdrop.classList.add("is-swipe-dragging");
    return;
  }
  requestAnimationFrame(() => {
    const navigationScrollRegion = mobilePopupMenu.querySelector(".mobile-navigation-scroll-region");
    const restoredScrollTop = navigationScrollPositions.mobile || 0;
    if (restoredScrollTop > 0) {
      restoreNavigationScrollPosition(navigationScrollRegion, "mobile");
    } else {
      mobilePopupMenu.querySelector(".mobile-wallet-picker-trigger")?.scrollIntoView({ block: "nearest" });
      mobilePopupMenu.querySelector(".mobile-navigation-tree [aria-current='page']")?.scrollIntoView({ block: "nearest" });
    }
    focusMobileDrawer({ preventScroll: restoredScrollTop > 0 });
  });
}

function walletSettingsContextNav() {
  const item = (key) => {
    const [label, iconName] = walletSettingsMeta[key];
    const active = state.walletSettingsSection === key;
    return `<button class="context-nav-item${active ? " is-active" : ""}" type="button" ${active ? 'aria-current="page"' : ""} data-wallet-settings-section="${key}">${icon(iconName)}<span><strong>${label}</strong></span></button>`;
  };
  return `<nav class="context-nav context-tab-list wallet-settings-context" aria-label="Selected wallet settings">${item("general")}${item("security")}${item("backup")}${item("policies")}${item("advanced")}</nav>`;
}

function walletSettingsYaml() {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  return [
    "schema_version: 1",
    "wallet:",
    `  id: \"${yamlScalar(wallet.id)}\"`,
    `  chain: \"${yamlScalar(wallet.chainId)}\"`,
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
  return `
    <div class="setting-group settings-first-group">
      <div class="setting-line compact-row" data-help-anchor="wallet-name"><strong class="compact-row-label">Wallet name</strong><span class="compact-value" title="${escapeHtml(wallet.name)}">${escapeHtml(wallet.name)}</span><button class="button compact-action" type="button" data-open-flow="wallet-rename">Rename</button></div>
      <div class="setting-line compact-row" data-help-anchor="wallet-id"><strong class="compact-row-label">Wallet ID</strong><span class="compact-value mono" title="${escapeHtml(wallet.id)}">${escapeHtml(wallet.id)}</span><span class="status-badge compact-action">Read-only</span></div>
      <div class="setting-line compact-row" data-help-anchor="wallet-chain" data-wallet-chain-readonly aria-label="${escapeHtml(t("common.chain"))}: ${escapeHtml(walletChain(wallet.chainId).label)}. ${escapeHtml(t("common.readOnly"))}."><strong class="compact-row-label">${t("common.chain")}</strong><span class="compact-value"></span><span class="compact-action">${walletChainBadgeMarkup(wallet.chainId)}</span></div>
    </div>`;
}

function walletSettingsSecurityDetail() {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  return `
    <div class="setting-group settings-first-group">
      <div class="setting-line compact-row" data-help-anchor="wallet-lock"><label class="compact-row-label" for="wallet-lock-after">Lock app after</label><select class="compact-value" id="wallet-lock-after" data-wallet-settings-control="lock-after"><option value="5"${preferences.lockAfterMinutes === "5" ? " selected" : ""}>5 minutes</option><option value="15"${preferences.lockAfterMinutes === "15" ? " selected" : ""}>15 minutes</option><option value="30"${preferences.lockAfterMinutes === "30" ? " selected" : ""}>30 minutes</option><option value="never"${preferences.lockAfterMinutes === "never" ? " selected" : ""}>Never</option></select><span class="compact-action"></span></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">Lock now</strong><span class="compact-value"></span><button class="button compact-action" type="button" data-demo-action="lock">Lock now</button></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">${t("walletSettings.password")}</strong><span class="compact-value"></span><button class="button compact-action" type="button" data-open-flow="wallet-password-change">${t("walletSettings.changePassword")}</button></div>
    </div>
    <div class="setting-group wallet-key-settings">
      <div class="setting-line compact-row" data-help-anchor="recovery-phrase"><strong class="compact-row-label">Recovery phrase</strong><span class="compact-value"></span><button class="button compact-action" type="button" data-open-flow="wallet-seed-reveal">View phrase</button></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">Public keys</strong><span class="compact-value"></span><button class="button compact-action" type="button" data-open-flow="wallet-public-export">View keys</button></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">Master key</strong><span class="compact-value" title="${escapeHtml(preferences.lastMasterKeyRotation)}">${escapeHtml(preferences.lastMasterKeyRotation)}</span><button class="button button-primary compact-action" type="button" data-open-flow="wallet-key-rotation">Rotate</button></div>
    </div>`;
}

function walletSettingsBackupDetail() {
  const wallet = activeWallet();
  const preferences = activeWalletPreferences();
  return `
    <div class="review-card wallet-settings-summary"><div class="summary-row"><span>Latest backup</span><strong>10 Jul 2026 · 09:42</strong></div><div class="summary-row"><span>Integrity</span><strong class="trust-label">${icon("shield")} Verified</strong></div><div class="summary-row"><span>Encryption</span><strong>Enabled</strong></div><div class="summary-row"><span>Wallet</span><strong>${escapeHtml(wallet.name)}</strong></div></div>
    <div class="setting-group"><div class="setting-line compact-row" data-help-anchor="automatic-backup"><strong class="compact-row-label">Automatic backup</strong><span class="compact-value"></span><button class="toggle compact-action" type="button" aria-pressed="${preferences.autoBackup}" aria-label="Automatic wallet backup" data-demo-action="wallet-auto-backup"></button></div><div class="setting-line compact-row"><label class="compact-row-label" for="wallet-backup-interval">Backup interval</label><select class="compact-value" id="wallet-backup-interval" data-wallet-settings-control="backup-interval"><option value="6"${preferences.backupIntervalHours === "6" ? " selected" : ""}>Every 6 hours</option><option value="24"${preferences.backupIntervalHours === "24" ? " selected" : ""}>Every 24 hours</option><option value="72"${preferences.backupIntervalHours === "72" ? " selected" : ""}>Every 3 days</option></select><span class="compact-action"></span></div></div>`;
}

function walletSettingsPoliciesDetail() {
  const preferences = activeWalletPreferences();
  const rules = preferences.policyRules;
  return `
    <div class="setting-group settings-first-group">
      <div class="setting-line compact-row" data-help-anchor="policy-profile"><strong class="compact-row-label">Profile preview</strong><span class="compact-value" title="${escapeHtml(preferences.policyProfile)}">${escapeHtml(preferences.policyProfile)}</span><span class="status-badge is-ready compact-action">Target</span></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">Local spend rules</strong><span class="compact-value mono">${escapeHtml(rules.maxTransaction)} / ${escapeHtml(rules.maxDaily)} Z00Z</span><button class="button button-primary compact-action" type="button" data-open-flow="wallet-policy-apply">Review</button></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">Protocol rules</strong><span class="compact-value"></span><span class="status-badge compact-action">Locked</span></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">Local policy rules</strong><span class="compact-value">${rules.requireConfirmation ? "Confirmation" : "No confirmation"}</span><span class="status-badge is-active compact-action">Local</span></div>
      <div class="setting-line compact-row"><strong class="compact-row-label">Compliance profile</strong><span class="compact-value">${t("common.unavailable")}</span><span class="status-badge is-ready compact-action">Target</span></div>
    </div>`;
}

function walletSettingsAdvancedDetail() {
  const source = state.walletSettingsConfigDraft || walletSettingsYaml();
  return `
    <div class="yaml-toolbar"><span><strong class="mono">wallet_settings.yaml</strong></span><div><button class="button" type="button" data-demo-action="wallet-config-validate">Validate</button><button class="button button-primary" type="button" data-demo-action="wallet-config-apply">Apply locally</button></div></div>
    ${yamlEditorMarkup("wallet-settings-yaml", source, "Selected wallet settings YAML")}
    <div class="config-foot"><span>${icon("shield")} No secrets or paths</span><span>${icon("activity")} Selected wallet only</span><span>${icon("settings")} ${escapeHtml(state.configStatus)}</span></div>`;
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

function networkDetail() {
  if (state.settingsSection === "reticulum") return `
    <div class="connection-options">
      <div class="connection-option"><span class="health-orb"></span><span><strong>Reticulum service</strong><small>Target service example · no live wallet API</small></span><span class="status-badge is-ready">Target</span></div>
      <div class="connection-option"><span class="list-icon">${icon("network")}</span><span><strong>Interfaces</strong><small>Auto · TCP client + local mesh discovery</small></span><button class="button" type="button" data-demo-action="config-stage">Configure</button></div>
      <div class="connection-option"><span class="list-icon">${icon("shield")}</span><span><strong>Network identity</strong><small class="mono">RNS 6A3E…91B2 · independent from wallet seed</small></span><span class="status-badge is-active">Separate</span></div>
    </div><div class="notice">${icon("settings")} Raw Reticulum interface definitions require a future runtime configuration route. Service/runtime changes may require restart.</div>`;

  if (state.settingsSection === "onionnet") return `
    <div class="connection-options">
      <div class="connection-option"><span class="health-orb"></span><span><strong>Privacy route</strong><small>Target example · 3 hops · epoch 1842</small></span><span class="status-badge is-ready">Target floor</span></div>
      <div class="connection-option"><span class="list-icon">${icon("shield")}</span><span><strong>Membership & replay checks</strong><small>Target telemetry · unavailable in current RPC</small></span><span class="status-badge is-ready">Target</span></div>
      <div class="connection-option"><span class="list-icon">${icon("activity")}</span><span><strong>Route age</strong><small>12 minutes · rebuilt automatically by policy</small></span><button class="button" type="button" data-demo-action="rebuild-route">Rebuild</button></div>
    </div><div class="capability-note">${icon("alert")} <span><strong>Target Phase 080 simulation</strong><small>The current live network RPC is stubbed; all route details on this screen are illustrative until an authoritative status capability exists.</small></span></div><div class="notice">${icon("shield")} This reports concrete route properties. It does not claim that the user is “anonymous” or “untraceable.”</div>`;

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
      <div class="setting-group settings-first-group">
        <div class="setting-line compact-row app-select-setting" data-help-anchor="language"><strong class="compact-row-label">${t("app.language")}</strong>${languagePickerMarkup("compact-value")}<span class="compact-action"></span></div>
        <div class="setting-line compact-row app-select-setting"><strong class="compact-row-label">${t("app.regionalFormat")}</strong><select class="compact-value" aria-label="${t("app.regionalFormat")}" data-config-control="regional-locale">${regionalLocaleOptionsMarkup()}</select><span class="compact-action"></span></div>
        <div class="setting-line compact-row app-select-setting" title="${escapeHtml(t("app.currencyHelp"))}"><strong class="compact-row-label">${t("app.currency")}</strong><select class="compact-value" aria-label="${escapeHtml(t("app.currency"))}" data-config-control="valuation-currency">${valuationCurrencyOptionsMarkup()}</select><span class="compact-action"></span></div>
        <div class="setting-line compact-row app-select-setting"><strong class="compact-row-label">${t("app.timeZone")}</strong><select class="compact-value" aria-label="${t("app.timeZone")}" data-config-control="time-zone"><option value="UTC"${state.timeZone === "UTC" ? " selected" : ""}>UTC</option><option value="Asia/Jerusalem"${state.timeZone === "Asia/Jerusalem" ? " selected" : ""}>Asia/Jerusalem</option><option value="Europe/Berlin"${state.timeZone === "Europe/Berlin" ? " selected" : ""}>Europe/Berlin</option><option value="America/New_York"${state.timeZone === "America/New_York" ? " selected" : ""}>America/New_York</option><option value="Asia/Tokyo"${state.timeZone === "Asia/Tokyo" ? " selected" : ""}>Asia/Tokyo</option><option value="Asia/Shanghai"${state.timeZone === "Asia/Shanghai" ? " selected" : ""}>Asia/Shanghai</option></select><span class="compact-action"></span></div>
      </div>`;
  }

  if (state.settingsSection === "notifications") {
    return `
      <div class="settings-heading"><div><p class="eyebrow">${escapeHtml(t("navigation.notifications"))}</p><h2>${escapeHtml(t("navigation.notifications"))}</h2><p>Choose how this device announces messages and important wallet events.</p></div></div>
      <div class="setting-group settings-first-group">
        <div class="setting-line compact-row notification-master-setting"><span class="setting-line-copy compact-row-label"><strong>${escapeHtml(t("app.notifications"))}</strong><small>Master notification control for this device</small></span><span class="compact-value"></span><button class="toggle compact-action" type="button" data-demo-action="general-notifications" aria-pressed="${state.notifications}" aria-label="${escapeHtml(t("app.notifications"))} ${state.notifications ? escapeHtml(t("common.on")) : escapeHtml(t("common.off"))}"></button></div>
        <div class="setting-line compact-row app-select-setting notification-choice-setting"><span class="setting-line-copy compact-row-label"><strong>Vibrate</strong><small>Haptic feedback policy</small></span><select class="compact-value" aria-label="Vibrate" data-config-control="vibrate"${state.notifications ? "" : " disabled"}><option value="messages-and-alerts"${state.vibrate === "messages-and-alerts" ? " selected" : ""}>Messages and alerts</option><option value="alerts-only"${state.vibrate === "alerts-only" ? " selected" : ""}>Important alerts only</option><option value="never"${state.vibrate === "never" ? " selected" : ""}>Never</option></select><span class="compact-action"></span></div>
        <div class="setting-line compact-row app-select-setting notification-choice-setting"><span class="setting-line-copy compact-row-label"><strong>Ringtone</strong><small>Sound used for new messages</small></span><select class="compact-value" aria-label="Ringtone" data-config-control="ringtone"${state.notifications ? "" : " disabled"}><option value="z00z-pulse"${state.ringtone === "z00z-pulse" ? " selected" : ""}>Z00Z Pulse</option><option value="soft-chime"${state.ringtone === "soft-chime" ? " selected" : ""}>Soft Chime</option><option value="none"${state.ringtone === "none" ? " selected" : ""}>None</option></select><span class="compact-action"></span></div>
      </div>
      <div class="notice">${icon("bell")} These choices are local demo preferences. A packaged app must request the operating-system permissions before using sound or vibration.</div>`;
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

  if (["reticulum", "onionnet"].includes(state.settingsSection)) {
    const isOnionNet = state.settingsSection === "onionnet";
    return `
      <div class="settings-heading"><div><p class="eyebrow">${isOnionNet ? "Private overlay" : "Local carrier"}</p><h2>${isOnionNet ? "OnionNet" : "Reticulum"}</h2><p>${isOnionNet ? "Route privacy and admission controls remain distinct from the carrier." : "Carrier configuration remains local and separate from wallet keys and route policy."}</p></div>${isOnionNet ? '<select aria-label="Network mode"><option>Private · no direct fallback</option><option>Auto</option><option>Resilient</option><option>Direct · warning</option></select>' : ""}</div>
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
      <div class="setting-group settings-first-group">
        <div class="setting-line palette-setting" data-help-anchor="appearance">
          <span class="setting-line-copy"><strong>${escapeHtml(t("plan2.palette.label"))}</strong><small>${escapeHtml(t("plan2.palette.help"))}</small></span>
          <div class="palette-choice">
            <div class="palette-grid" aria-label="${escapeHtml(t("plan2.palette.previews"))}">${paletteOptions.map(paletteCard).join("")}</div>
          </div>
        </div>
        <div class="setting-line palette-setting code-theme-setting"><span class="setting-line-copy"><strong>Code highlighting</strong></span><div class="code-theme-sections" aria-label="YAML code highlighting theme"><section><p class="code-theme-group-label">Light</p><div class="code-theme-grid">${codeThemeOptions.filter((theme) => theme.mode === "light").map(codeThemeCard).join("")}</div></section><section><p class="code-theme-group-label">Dark</p><div class="code-theme-grid">${codeThemeOptions.filter((theme) => theme.mode === "dark").map(codeThemeCard).join("")}</div></section></div></div>
        <div class="setting-line compact-row"><strong class="compact-row-label">Text scale</strong><select class="compact-value" aria-label="Text scale" data-config-control="text-scale"><option value="100"${state.textScale === "100" ? " selected" : ""}>100%</option><option value="110"${state.textScale === "110" ? " selected" : ""}>110%</option><option value="125"${state.textScale === "125" ? " selected" : ""}>125%</option></select><span class="compact-action"></span></div>
        <div class="setting-line compact-row"><strong class="compact-row-label">Reduced motion</strong><span class="compact-value"></span><button class="toggle compact-action" type="button" aria-pressed="${state.reducedMotion}" aria-label="Use reduced motion" data-demo-action="motion"></button></div>
      </div>`;
  }

  return advancedConfigContent();
}

function settingsView() {
  return `
    <div class="view-enter settings-view">
      <div class="settings-layout settings-layout--full">
        <article class="card settings-detail">${settingsDetail()}</article>
      </div>
    </div>`;
}

function usageMeter(label, value, total, detail) {
  const percent = Math.min(100, Math.max(0, Math.round((value / total) * 100)));
  return `<article class="usage-meter">
    <div><span>${escapeHtml(label)}</span><strong>${escapeHtml(detail)}</strong></div>
    <div class="progress-track" aria-label="${escapeHtml(label)}: ${percent}%"><div class="progress-bar" style="width:${percent}%"></div></div>
  </article>`;
}

function dataStorageView() {
  if (state.dataStorageSection === "network-usage") {
    return `<section class="view-enter data-storage-view">
      <div class="settings-layout settings-layout--full">
        <article class="card settings-detail">
          <div class="settings-heading"><div><p class="eyebrow">${escapeHtml(t("navigation.dataStorage"))}</p><h2>${escapeHtml(t("navigation.networkUsage"))}</h2><p>Local demonstration counters. No wallet address, contact, message, or route detail is exported.</p></div></div>
          <div class="network-summary-grid">
            <article><span>Received today</span><strong>184 MB</strong><small>Local fixture</small></article>
            <article><span>Sent today</span><strong>42 MB</strong><small>Local fixture</small></article>
            <article><span>Reticulum</span><strong>138 MB</strong><small>Transport total</small></article>
            <article><span>OnionNet</span><strong>88 MB</strong><small>Private overlay total</small></article>
          </div>
          <div class="notice">${icon("shield")} Production counters must remain aggregate and must not expose contacts, destinations, message content, or wallet activity.</div>
        </article>
      </div>
    </section>`;
  }
  return `<section class="view-enter data-storage-view">
    <div class="settings-layout settings-layout--full">
      <article class="card settings-detail">
        <div class="settings-heading"><div><p class="eyebrow">${escapeHtml(t("navigation.dataStorage"))}</p><h2>${escapeHtml(t("navigation.diskUsage"))}</h2><p>Understand local space without exposing private wallet material.</p></div><strong class="usage-total">286 MB</strong></div>
        <div class="usage-meter-list">
          ${usageMeter("Wallet indexes", 164, 512, "164 MB")}
          ${usageMeter("Network cache", 78, 512, "78 MB")}
          ${usageMeter("Help content", 31, 512, "31 MB")}
          ${usageMeter("Sanitized logs", 13, 512, "13 MB")}
        </div>
        <div class="notice">${icon("storage")} Values are deterministic demo fixtures. Seed phrases, keys, passwords, and arbitrary file paths are never shown.</div>
      </article>
    </div>
  </section>`;
}

function aboutView() {
  const checked = state.updateCheckStatus === "current";
  return `<section class="view-enter about-view">
    <div class="about-surface">
      <div class="about-identity">
        <img src="assets/logo/z00z-logo-gold-circle.png" alt="">
        <h2>${escapeHtml(t("plan2.about.productVersion", { version: demoRuntime.APP_VERSION }))}</h2>
        <p>${escapeHtml(t("plan2.about.summary"))}</p>
      </div>
      <nav class="about-links" aria-label="${escapeHtml(t("plan2.about.linksLabel"))}">
        <a href="https://z00z.io/docs/legal/privacy" target="_blank" rel="noopener noreferrer">${escapeHtml(t("plan2.about.privacyPolicy"))}</a>
        <a href="https://z00z.io/docs/legal/terms" target="_blank" rel="noopener noreferrer">${escapeHtml(t("plan2.about.termsOfUse"))}</a>
        <a href="https://github.com/z00z-labs/z00z" target="_blank" rel="noopener noreferrer">${escapeHtml(t("plan2.about.visitGitHub"))}</a>
      </nav>
      <p class="update-check-status" role="status"${checked ? "" : " hidden"}>${checked ? escapeHtml(t("plan2.about.currentVersion", { version: demoRuntime.APP_VERSION })) : ""}</p>
      <button class="button button-primary about-update-button" type="button" data-demo-action="check-for-updates">${icon("activity")} ${escapeHtml(t("plan2.about.checkForUpdates"))}</button>
    </div>
  </section>`;
}

function telemetryValue(label, value, helper) {
  return `<article><span>${label}</span><strong>${value}</strong><small>${helper}</small></article>`;
}

function telemetryLine(iconName, title, detail) {
  return `<div class="connection-option"><span class="list-icon">${icon(iconName)}</span><span><strong>${title}</strong><small>${detail}</small></span><span class="status-badge">${t("common.unavailable")}</span></div>`;
}

const RETICULUM_TAB_ICON_LUT = Object.freeze({
  overview: "overview",
  node: "reticulum-node",
  interfaces: "reticulum-interface",
  radio: "network",
  entrypoints: "entry",
  paths: "reticulum-paths",
  probes: "probe",
  links: "reticulum-link"
});

const reticulumTelemetryTabs = [
  {
    id: "overview",
    labelKey: "reticulum.tabs.overview",
    iconName: RETICULUM_TAB_ICON_LUT.overview,
    metrics: [
      ["Managed-node state", "Unavailable", "No local Reticulum status bridge"],
      ["Interface availability", "Unavailable", "No local interface summary"],
      ["Path and link evidence", "Unavailable", "No managed transport summary"],
      ["Probe coverage", "Unavailable", "No controlled-destination probe results"]
    ]
  },
  {
    id: "node",
    label: "Node",
    labelKey: "reticulum.tabs.node",
    iconName: RETICULUM_TAB_ICON_LUT.node,
    metrics: [
      ["RNS instance", "Unavailable", "No local rnstatus bridge"],
      ["Transport role", "Unavailable", "No managed node snapshot"],
      ["Uptime", "Unavailable", "No local process observation"],
      ["Aggregate RX / TX", "Unavailable", "No local traffic counters"]
    ]
  },
  {
    id: "interfaces",
    label: "Interfaces",
    labelKey: "reticulum.tabs.interfaces",
    iconName: RETICULUM_TAB_ICON_LUT.interfaces,
    metrics: [
      ["Interfaces up / total", "Unavailable", "No local interface snapshot"],
      ["Mode", "Unavailable", "No interface-mode snapshot"],
      ["Nominal bitrate", "Unavailable", "No configured-rate snapshot"],
      ["Current RX / TX", "Unavailable", "No local traffic-rate snapshot"]
    ]
  },
  {
    id: "radio",
    label: "Radio",
    labelKey: "reticulum.tabs.radio",
    iconName: RETICULUM_TAB_ICON_LUT.radio,
    metrics: [
      ["Frequency", "Unavailable", "No RNode interface snapshot"],
      ["Channel configuration", "Unavailable", "No bandwidth / SF / CR metadata"],
      ["RF health", "Unavailable", "No noise / RSSI / SNR observation"],
      ["Airtime / channel load", "Unavailable", "No short or long-window observation"]
    ]
  },
  {
    id: "entrypoints",
    label: "Entry points",
    labelKey: "reticulum.tabs.entrypoints",
    iconName: RETICULUM_TAB_ICON_LUT.entrypoints,
    metrics: [
      ["Discovery state", "Unavailable", "No local discovery snapshot"],
      ["Available entry points", "Unavailable", "No trusted-entrypoint count"],
      ["Last heard", "Unavailable", "No discovery freshness signal"],
      ["Trust scope", "Unavailable", "No managed discovery policy"]
    ]
  },
  {
    id: "paths",
    label: "Paths",
    labelKey: "reticulum.tabs.paths",
    iconName: RETICULUM_TAB_ICON_LUT.paths,
    metrics: [
      ["Known paths", "Unavailable", "No local path-table summary"],
      ["Path freshness", "Unavailable", "No local update observation"],
      ["Path churn", "Unavailable", "No managed change-rate summary"],
      ["Announce pressure", "Unavailable", "No announce-rate or hold-state summary"]
    ]
  },
  {
    id: "probes",
    label: "Probes",
    labelKey: "reticulum.tabs.probes",
    iconName: RETICULUM_TAB_ICON_LUT.probes,
    metrics: [
      ["Probe availability", "Unavailable", "No managed-destination probe results"],
      ["RTT", "Unavailable", "No local latency sample"],
      ["Loss / jitter", "Unavailable", "No probe series"],
      ["Consecutive failures", "Unavailable", "No local failure count"]
    ]
  },
  {
    id: "links",
    label: "Links",
    labelKey: "reticulum.tabs.links",
    iconName: RETICULUM_TAB_ICON_LUT.links,
    metrics: [
      ["Active links", "Unavailable", "No local link summary"],
      ["Receipt delivery", "Unavailable", "No local receipt observation"],
      ["Expected / establish rate", "Unavailable", "No application link-rate summary"],
      ["Measured goodput", "Unavailable", "No controlled resource transfer"]
    ]
  }
];

const aggregatorsTelemetryTabs = [
  {
    id: "overview",
    labelKey: "aggregators.tabs.overview",
    iconName: "overview",
    summary: "The runtime composes admission, deterministic ordering, publication, and recovery boundaries; this renderer observes none of them directly.",
    authority: "AggregatorService: AggregatorIngress + AggregatorOrdering + AggregatorRecovery",
    metrics: [
      ["Service bindings", "Unavailable", "No wallet-to-node status bridge"],
      ["Publication", "Unavailable", "No latest publication record"],
      ["Placement", "Unavailable", "No batch placement observation"],
      ["Verdict", "Unavailable", "No validation or lifecycle evidence"]
    ],
    fields: [
      ["Admission", "WorkPayload → WorkItem | RejectRecord"],
      ["Ordering", "&[WorkItem] → OrderedBatch | RejectRecord"],
      ["Publication", "PublicationRequest → PublishedBatch"],
      ["Recovery", "ShardRecoveryRecord → ShardExecTicket"]
    ],
    guards: [
      "No wallet-to-node bridge is registered in this demo.",
      "Storage owns settlement roots, proofs, lifecycle evidence, and recovery truth.",
      "Wallet-private identifiers never enter this workspace."
    ]
  },
  {
    id: "ingress",
    labelKey: "navigation.ingress",
    iconName: "entry",
    summary: "Admission normalizes a transaction or claim payload and returns a digest-bound WorkItem or a typed rejection.",
    authority: "AggregatorIngress::admit(WorkPayload) → Result<WorkItem, RejectRecord>",
    metrics: [
      ["Payload class", "Unavailable", "Contract permits Tx or Claim"],
      ["Intake identity", "Unavailable", "Derived from the admission digest"],
      ["Object binding", "Unavailable", "Optional runtime object package binding"],
      ["Admission outcome", "Unavailable", "No WorkItem or RejectRecord snapshot"]
    ],
    fields: [
      ["Input", "WorkPayload::Tx | WorkPayload::Claim"],
      ["Accepted", "WorkItem { intake_id, payload, object_package }"],
      ["Rejected", "RejectRecord { intake_id?, class, detail }"],
      ["Reject classes", "parse · auth · shape · replay · policy · deferred"]
    ],
    guards: [
      "A bound object package changes the admission digest and intake identity.",
      "No raw payload, receiver, memo, or wallet-local route is exposed.",
      "Unavailable is not interpreted as admitted or rejected."
    ]
  },
  {
    id: "planning",
    labelKey: "navigation.planning",
    iconName: "advanced",
    summary: "Planning binds admitted work to one shard route and produces a deterministic batch plan without settlement authority.",
    authority: "BatchPlanner + PlannerAuthority → BatchPlanned",
    metrics: [
      ["Planner mode", "Unavailable", "Runtime supports central or per_agg"],
      ["Batch route", "Unavailable", "Shard ID plus routing generation"],
      ["Work binding", "Unavailable", "Intake IDs and operation count"],
      ["Plan evidence", "Unavailable", "Route-table and plan digests"]
    ],
    fields: [
      ["Authority", "mode · routing_generation · route_table_digest"],
      ["Route", "BatchRoute { shard_id, routing_generation }"],
      ["Plan", "BatchPlanned { batch_id, intake_ids, op_count }"],
      ["Digests", "planner config · route table · deterministic plan"]
    ],
    guards: [
      "Planner mode, configuration, generation, and route-table digest must match.",
      "A claimed plan is accepted only after deterministic local recomputation.",
      "Planning does not finalize settlement or publication."
    ]
  },
  {
    id: "placement",
    labelKey: "navigation.placement",
    iconName: "reticulum-interface",
    summary: "Placement is an operational view of which aggregator owns a planned shard generation and which secondaries are ready.",
    authority: "ShardPlacementTable::view(BatchPlanned) → ShardPlacementView",
    metrics: [
      ["Shard route", "Unavailable", "No observed shard or generation"],
      ["Primary owner", "Unavailable", "No AggregatorId snapshot"],
      ["Secondaries", "Unavailable", "No bounded readiness set"],
      ["Journal lineage", "Unavailable", "No expected lineage digest"]
    ],
    fields: [
      ["Route", "BatchRoute { shard_id, routing_generation }"],
      ["Primary", "AggregatorId"],
      ["Secondaries", "SecondaryState { aggregator_id, is_ready }"],
      ["Continuity", "expected_journal_lineage"]
    ],
    guards: [
      "The placement table must own the exact shard and routing generation.",
      "Primary and secondary IDs are operational runtime data, not wallet identities.",
      "This concept never invents endpoints, topology, or global network health."
    ]
  },
  {
    id: "publication",
    labelKey: "navigation.publication",
    iconName: "send",
    summary: "Publication binds one ordered batch to checkpoint, quorum, data-availability, and lifecycle evidence.",
    authority: "PublicationRequest → PublishedBatch → PublicationRecord",
    metrics: [
      ["Publication state", "Unavailable", "No current lifecycle snapshot"],
      ["Operation identity", "Unavailable", "No idempotency key or batch ID"],
      ["Checkpoint binding", "Unavailable", "No root or delta evidence"],
      ["DA evidence", "Unavailable", "No provider reference or evidence bundle"]
    ],
    fields: [
      ["Request", "batch · route · subject · certificate · idempotency_key"],
      ["Published", "checkpoint · provider · blob_ref · quorum digests"],
      ["Record", "state · DA reference · publication evidence · lifecycle"],
      ["Binding", "route table · roots · spent/created counts · pub_in digest"]
    ],
    guards: [
      "Partial evidence bundles fail readiness validation.",
      "Provider, height, manifest, payload, statement, and evidence roots must agree.",
      "Storage remains authoritative for checkpoint roots, proofs, and lifecycle."
    ]
  },
  {
    id: "recovery",
    labelKey: "navigation.recovery",
    iconName: "restore",
    summary: "Recovery resumes a committed shard execution only when route ownership, generation, primary, and journal lineage still agree.",
    authority: "RecoveryBoundary::capture / resume → ShardRecoveryRecord | ShardExecTicket",
    metrics: [
      ["Recovery intent", "Unavailable", "RestartPrimary or TakeoverSecondary"],
      ["Committed record", "Unavailable", "No checkpoint or publication state"],
      ["Durable lineage", "Unavailable", "No storage recovery snapshot"],
      ["Execution ticket", "Unavailable", "No routed or recovery-pending state"]
    ],
    fields: [
      ["Record", "batch · placement · checkpoint? · publication_state"],
      ["Durable state", "version · state_root · generation · lineage · route"],
      ["Intent", "restart_primary | takeover_secondary"],
      ["Ticket", "batch_id · placement · ShardExecState"]
    ],
    guards: [
      "Wrong generation, primary, shard, batch, or journal lineage fails closed.",
      "A secondary takeover requires a ready secondary and committed recovery state.",
      "The renderer cannot initiate failover or mutate storage recovery truth."
    ]
  }
];

function aggregatorContractFields(screen) {
  return screen.fields.map(([label, contract]) => `<div><dt>${escapeHtml(label)}</dt><dd><code>${escapeHtml(contract)}</code></dd></div>`).join("");
}

function aggregatorGuardList(screen) {
  return screen.guards.map((guard) => `<li>${icon("shield")}<span>${escapeHtml(guard)}</span></li>`).join("");
}

function aggregatorTelemetryPanel(screen) {
  const label = t(screen.labelKey);
  return `<section class="telemetry-view aggregator-concept" data-aggregator-screen="${escapeHtml(screen.id)}" aria-labelledby="aggregator-screen-title">
    <section class="telemetry-tab-detail" aria-label="${escapeHtml(label)} runtime contract">
      <div class="telemetry-tab-heading"><div><h3 id="aggregator-screen-title">${escapeHtml(label)}</h3><p>${escapeHtml(screen.summary)}</p></div><span class="status-badge">${t("common.unavailable")}</span></div>
      <section class="network-summary-grid telemetry-summary" aria-label="${escapeHtml(label)} observation status">${screen.metrics.map(([metric, value, helper]) => telemetryValue(escapeHtml(metric), escapeHtml(value), escapeHtml(helper))).join("")}</section>
      <div class="aggregator-contract-grid">
        <section class="aggregator-contract-card" aria-labelledby="aggregator-fields-title">
          <div class="aggregator-contract-heading">${icon("advanced")}<h4 id="aggregator-fields-title">Runtime contract fields</h4></div>
          <dl>${aggregatorContractFields(screen)}</dl>
        </section>
        <section class="aggregator-contract-card" aria-labelledby="aggregator-guards-title">
          <div class="aggregator-contract-heading">${icon("shield")}<h4 id="aggregator-guards-title">Fail-closed boundaries</h4></div>
          <ul>${aggregatorGuardList(screen)}</ul>
        </section>
      </div>
    </section>
  </section>`;
}

const onionnetTelemetryTabs = [
  {
    id: "overview",
    labelKey: "onionnet.tabs.overview",
    iconName: "overview",
    metrics: [
      ["Public epoch data", "Unavailable", "No verified registry or policy snapshot"],
      ["Local route evidence", "Unavailable", "No wallet or SDK status bridge"],
      ["Synthetic health", "Unavailable", "No aggregate probe feed"],
      ["Protected fields", "Hidden", "No paths, endpoints, or session IDs"]
    ]
  },
  {
    id: "epoch",
    labelKey: "onionnet.tabs.epoch",
    iconName: "activity",
    metrics: [
      ["Epoch ID", "Unavailable", "No fresh verified epoch view"],
      ["Independent derivation", "Unavailable", "No observer agreement snapshot"],
      ["Registry / policy freshness", "Unavailable", "No verified roots or snapshot age"],
      ["Lane contract expiry", "Unavailable", "No active contract snapshot"]
    ]
  },
  {
    id: "privacy",
    labelKey: "onionnet.tabs.privacy",
    iconName: "shield",
    metrics: [
      ["Privacy", "Unavailable", "No active profile evaluation"],
      ["Active lanes", "Unavailable", "No lane-contract snapshot"],
      ["Minimum bucket population", "Unavailable", "No bucket aggregate"],
      ["Compliant route floor", "Unavailable", "No policy-bound route count"]
    ]
  },
  {
    id: "transport",
    labelKey: "onionnet.tabs.transport",
    iconName: "network",
    metrics: [
      ["Carrier availability", "Unavailable", "No local carrier snapshot"],
      ["Aggregate RTT / loss", "Unavailable", "No aggregate measurement window"],
      ["Carrier distribution", "Unavailable", "No coarse traffic-class aggregate"],
      ["Geometry compliance", "Unavailable", "No packet-class validation aggregate"]
    ]
  },
  {
    id: "queues",
    labelKey: "onionnet.tabs.queues",
    iconName: "queue",
    metrics: [
      ["Queue utilization", "Unavailable", "No local bounded-queue aggregate"],
      ["Replay ledger", "Unavailable", "No durable replay snapshot"],
      ["Backpressure actions", "Unavailable", "No aggregated reason counts"],
      ["Forwarding invariant", "Unavailable", "No verified replay-before-forward proof"]
    ]
  },
  {
    id: "probation",
    labelKey: "onionnet.tabs.probation",
    iconName: "probe",
    metrics: [
      ["Probation population", "Unavailable", "No lifecycle aggregate"],
      ["Shadow-probe coverage", "Unavailable", "No aggregate probe results"],
      ["Reserve activation", "Unavailable", "No activation-time observation"],
      ["Challenge outcomes", "Unavailable", "No bounded outcome summary"]
    ]
  },
  {
    id: "ingress",
    labelKey: "onionnet.tabs.ingress",
    iconName: "entry",
    metrics: [
      ["Exit boundary", "Unavailable", "No opaque-handoff aggregate"],
      ["Inner decrypt", "Unavailable", "No result-count aggregate"],
      ["Recipient-key lifecycle", "Unavailable", "No key-age or rotation snapshot"],
      ["Runtime admission", "Unavailable", "No WorkItem admission aggregate"]
    ]
  }
];

function workspaceContextNav(workspaceId) {
  const workspace = demoRuntime.navigationNode(workspaceId);
  return `<nav class="context-nav context-tab-list workspace-local-context${workspaceId.startsWith("telemetry.") ? " telemetry-workspace-context" : ""}" aria-label="${escapeHtml(navigationLabel(workspace))}">${demoRuntime.workspaceLocalDestinations(workspaceId).map(({ routeId, labelKey, iconId }) => {
    const active = state.activeRoute === routeId;
    return `<button class="context-nav-item${active ? " is-active" : ""}" type="button" ${active ? 'aria-current="page"' : ""} data-workspace-route="${escapeHtml(routeId)}">
      ${icon(iconId)}<span><strong>${t(labelKey)}</strong></span>
    </button>`;
  }).join("")}</nav>`;
}

function workspaceFrame(workspaceId, panel, helpTopicOverride = "") {
  return `<div class="view-enter workspace-layout workspace-local-layout${workspaceId.startsWith("telemetry.") ? " telemetry-workspace-layout" : ""}" data-workspace-id="${escapeHtml(workspaceId)}"${helpTopicOverride ? ` data-help-topic-override="${escapeHtml(helpTopicOverride)}"` : ""}>
    <aside class="context-rail">${workspaceContextNav(workspaceId)}</aside>
    <div class="workspace-panel">${panel}</div>
  </div>`;
}

function telemetryTabbedView({ source, tabs, selectedTabId, titleKey, localCapabilityKey }) {
  const activeTab = tabs.find((tab) => tab.id === selectedTabId) || tabs[0];
  const tabLabel = t(activeTab.labelKey);
  const panel = `<section class="telemetry-view ${source}-telemetry-view" aria-labelledby="telemetry-heading">
    <h2 class="sr-only" id="telemetry-heading">${t(titleKey)}</h2>
    <div class="capability-note capability-note-compact">${icon("alert")}<span><strong>${t(localCapabilityKey)}</strong></span><span class="status-badge">${t("common.readOnly")}</span></div>
    <section id="${source}-panel-${activeTab.id}" class="telemetry-tab-detail" role="tabpanel" aria-label="${tabLabel}" aria-labelledby="${source}-tab-${activeTab.id}">
      <div class="telemetry-tab-heading"><div><h3>${tabLabel}</h3></div><span class="status-badge">${t("common.unavailable")}</span></div>
      <section class="network-summary-grid telemetry-summary" aria-label="${tabLabel} parameters">${activeTab.metrics.map(([label, value, helper]) => telemetryValue(label, value, helper)).join("")}</section>
    </section>
  </section>`;
  return workspaceFrame(`telemetry.${source}`, panel);
}

function reticulumTelemetryView() {
  return telemetryTabbedView({
    source: "reticulum",
    tabs: reticulumTelemetryTabs,
    selectedTabId: state.reticulumTelemetryTab,
    titleKey: "reticulum.title",
    localCapabilityKey: "reticulum.localCapability"
  });
}

function onionnetTelemetryView() {
  return telemetryTabbedView({
    source: "onionnet",
    tabs: onionnetTelemetryTabs,
    selectedTabId: state.onionnetTelemetryTab,
    titleKey: "onionnet.title",
    localCapabilityKey: "onionnet.localCapability"
  });
}

function aggregatorsTelemetryView() {
  const screen = aggregatorsTelemetryTabs.find(({ id }) => id === state.aggregatorsTelemetryTab)
    || aggregatorsTelemetryTabs[0];
  return workspaceFrame("telemetry.aggregators", aggregatorTelemetryPanel(screen));
}

const watcherScenarioLabels = Object.freeze({
  loading: "Loading",
  success: "Useful fixture",
  degraded: "Degraded",
  unavailable: "Unavailable",
  empty: "Empty",
  malformed: "Malformed",
  error: "Error"
});

const watcherKindOptions = Object.freeze([
  ["all", "All kinds"],
  ["PublicationLag", "Publication lag"],
  ["MissingBlob", "Missing blob"],
  ["RouteRollout", "Route rollout"]
]);

function watcherObservation() {
  const routeId = state.activeRoute.startsWith("telemetry.watchers.")
    ? state.activeRoute
    : "telemetry.watchers.overview";
  return telemetryGateway.readWatcherView({
    routeId,
    scenario: state.watcherScenario,
    sourceId: state.watcherSourceId,
    generation: Number(state.requestGenerations[`telemetry:${routeId}`] || 0),
    filters: {
      severity: state.watcherSeverityFilter,
      kind: state.watcherKindFilter
    }
  });
}

function watcherControls() {
  const showFilters = ["alerts", "evidence"].includes(state.watchersTelemetryTab);
  return `<section class="watcher-toolbar" aria-label="${escapeHtml(t("plan2.aria.watcherControls"))}">
    <label><span>Observation source</span><select data-watcher-control="source" aria-label="Watchers observation source">${telemetryGateway.watcherSources.map((source) => `<option value="${escapeHtml(source.id)}"${source.id === state.watcherSourceId ? " selected" : ""}>${escapeHtml(source.label)}</option>`).join("")}</select></label>
    <label><span>Scenario</span><select data-watcher-control="scenario" aria-label="Watchers scenario">${telemetryGateway.scenarioIds.map((scenario) => `<option value="${escapeHtml(scenario)}"${scenario === state.watcherScenario ? " selected" : ""}>${escapeHtml(watcherScenarioLabels[scenario])}</option>`).join("")}</select></label>
    ${showFilters ? `<label><span>Severity</span><select data-watcher-control="severity" aria-label="Filter Watchers by severity">${[
      ["all", "All severities"],
      ["info", "Info"],
      ["warn", "Warning"],
      ["critical", "Critical"]
    ].map(([id, label]) => `<option value="${id}"${id === state.watcherSeverityFilter ? " selected" : ""}>${label}</option>`).join("")}</select></label>
    <label><span>Kind</span><select data-watcher-control="kind" aria-label="Filter Watchers by kind">${watcherKindOptions.map(([id, label]) => `<option value="${id}"${id === state.watcherKindFilter ? " selected" : ""}>${label}</option>`).join("")}</select></label>` : ""}
  </section>`;
}

function watcherStateNotice(observation) {
  if (observation.status === "success" && observation.data?.total > 0) return "";
  if (observation.status === "degraded" && observation.data?.total > 0) {
    return `<div class="watcher-state-notice is-warning" role="status">${icon("alert")}<span><strong>Degraded fixture observation</strong><small>${escapeHtml(observation.issue.message)} ${escapeHtml(observation.issue.recoveryAction)}</small></span><button class="button" type="button" data-watcher-action="recover">Refresh fixture</button></div>`;
  }
  if (observation.status === "loading") {
    return `<div class="watcher-state-panel" data-watcher-state="loading" role="status" aria-live="polite"><div class="watcher-loading-copy"><span class="watcher-loading-indicator" aria-hidden="true"></span><span><strong>Loading sanitized observation</strong><p>No previous route result is promoted while this deterministic request is pending.</p></span></div><button class="button" type="button" data-watcher-action="recover">Complete fixture request</button></div>`;
  }
  const filteredEmpty = observation.status === "success" && observation.data?.total === 0;
  const title = filteredEmpty ? "No matching observations" : {
    unavailable: "Telemetry source unavailable",
    empty: "No observations in this fixture",
    malformed: "Malformed source payload rejected",
    error: "Telemetry query failed"
  }[observation.status] || "No observation";
  const message = filteredEmpty
    ? "The selected filters exclude every sanitized record."
    : observation.issue?.message || "No authoritative observation is available.";
  const action = filteredEmpty || observation.status === "empty"
    ? `<button class="button" type="button" data-watcher-action="clear-filters">Clear filters</button>`
    : `<button class="button button-primary" type="button" data-watcher-action="recover">Retry with fixture</button>`;
  return `<div class="watcher-state-panel${["malformed", "error"].includes(observation.status) ? " is-danger" : ""}" data-watcher-state="${escapeHtml(filteredEmpty ? "empty" : observation.status)}" role="status"><div>${icon(["malformed", "error"].includes(observation.status) ? "alert" : "eye-off")}<strong>${escapeHtml(title)}</strong><p>${escapeHtml(message)}</p>${observation.issue?.recoveryAction ? `<small>${escapeHtml(observation.issue.recoveryAction)}</small>` : ""}</div>${action}</div>`;
}

function watcherOverview(records) {
  const snapshot = records[0];
  if (!snapshot) return "";
  return `<section class="network-summary-grid telemetry-summary watcher-summary" aria-label="Watcher observation summary">
    ${telemetryValue("Publication state", snapshot.publicationState, "PublicationRecord state from the deterministic witness")}
    ${telemetryValue("Provider outcome", snapshot.providerOutcome, `${snapshot.providerStage} stage · ProviderSignal`)}
    ${telemetryValue("Alert counts", `${snapshot.alertCounts.critical} critical · ${snapshot.alertCounts.warn} warning`, `${snapshot.alertCounts.info} informational fixture alert`)}
    ${telemetryValue("Runtime authority", snapshot.runtimeTruth ? "Authoritative" : "Observational only", "Watcher runtime notes do not become protocol truth")}
  </section>
  <section class="watcher-mapping-card"><div>${icon("advanced")}<span><strong>Current Rust mapping</strong><small><code>WatcherBoundary::project_snapshot</code><span>produces</span><code>ObservationSnapshot</code></small></span></div><p>Snapshot fields bind batch, publication state, placement, execution, verdict, provider stage/outcome, runtime notes, and alert counts. Storage and validators remain authoritative.</p></section>`;
}

function watcherSeverityLabel(severity) {
  return severity === "critical" ? "Critical" : severity === "warn" ? "Warning" : "Info";
}

function watcherAlertCard(alert) {
  const selected = state.watcherSelectedAlertId === alert.id;
  return `<button class="watcher-alert-card severity-${escapeHtml(alert.severity)}${selected ? " is-selected" : ""}" type="button" data-watcher-alert="${escapeHtml(alert.id)}"${selected ? ' aria-current="true"' : ""}>
    <span class="watcher-alert-icon">${icon(alert.severity === "critical" ? "alert" : "eye")}</span>
    <span class="watcher-alert-copy"><span><strong>${escapeHtml(alert.kind)}</strong><em>${escapeHtml(watcherSeverityLabel(alert.severity))}</em></span><small>${escapeHtml(alert.summary)}</small><code>${escapeHtml(alert.subject.publicId)}</code></span>
  </button>`;
}

function watcherAlertDetail(alert) {
  if (!alert) return `<section class="watcher-detail-empty"><strong>Select a typed alert</strong><p>Open an alert to inspect kind, severity, public subject, observation time, provenance, affected public IDs, and its safe next action.</p></section>`;
  return `<section class="watcher-alert-detail" aria-labelledby="watcher-alert-detail-title">
    <div class="watcher-detail-heading"><div><p class="eyebrow">Typed WatcherAlert</p><h4 id="watcher-alert-detail-title">${escapeHtml(alert.kind)}</h4></div><span class="status-badge">${escapeHtml(watcherSeverityLabel(alert.severity))}</span></div>
    <p>${escapeHtml(alert.summary)}</p>
    <dl>
      <div><dt>Subject</dt><dd><code>${escapeHtml(alert.subject.kind)} · ${escapeHtml(alert.subject.publicId)}</code></dd></div>
      <div><dt>Observed</dt><dd>${escapeHtml(alert.observedAt)}</dd></div>
      <div><dt>Provenance</dt><dd><code>${escapeHtml(alert.provenance.module)}</code> · ${escapeHtml(alert.provenance.evidence)}</dd></div>
      <div><dt>Affected public IDs</dt><dd>${alert.affectedPublicIds.map((id) => `<code>${escapeHtml(id)}</code>`).join(" ")}</dd></div>
    </dl>
    <div class="watcher-detail-actions">
      <button class="button button-primary" type="button" data-watcher-action="inspect-evidence" data-alert-id="${escapeHtml(alert.id)}">${icon("search")} ${escapeHtml(alert.nextAction.label)}</button>
      <button class="button button-quiet" type="button" data-watcher-action="open-explorer" data-public-id="${escapeHtml(alert.explorerAction.publicId)}">${icon("copy")} ${escapeHtml(alert.explorerAction.label)}</button>
    </div>
  </section>`;
}

function watcherAlerts(records) {
  const selected = records.find(({ id }) => id === state.watcherSelectedAlertId);
  return `<div class="watcher-split-layout">
    <section class="watcher-alert-list" aria-label="Typed Watcher alerts">${records.map(watcherAlertCard).join("")}</section>
    ${watcherAlertDetail(selected)}
  </div>`;
}

function watcherPublication(records) {
  return `<section class="watcher-record-grid" aria-label="Publication checks">${records.map((record) => `<article class="watcher-record-card"><div>${icon(record.status === "matched" ? "check" : "alert")}<span><strong>${escapeHtml(record.check)}</strong><small>${escapeHtml(record.status.replaceAll("_", " "))}</small></span></div><p>${escapeHtml(record.detail)}</p><code>${escapeHtml(record.mapping)}</code></article>`).join("")}</section>
    <p class="watcher-boundary-note">${icon("shield")} A matched Watcher check is local evidence only. It cannot finalize a checkpoint or override storage readiness.</p>`;
}

function watcherProviders(records) {
  return `<section class="watcher-record-grid" aria-label="DA provider signals">${records.map((record) => `<article class="watcher-record-card"><div>${icon("network")}<span><strong>${escapeHtml(record.providerName)}</strong><small>${escapeHtml(record.stage)} · ${escapeHtml(record.outcome)}</small></span></div><dl><div><dt>Batch</dt><dd><code>${escapeHtml(record.batchId)}</code></dd></div><div><dt>Opaque DA ref</dt><dd><code>${escapeHtml(record.blobRef)}</code></dd></div></dl><code>${escapeHtml(record.mapping)}</code></article>`).join("")}</section>
    <p class="watcher-boundary-note">${icon("alert")} <code>ProviderCompare</code> is currently a marker type; this preview compares typed fixture rows without inventing a provider ranking or semantic verdict.</p>`;
}

function watcherCensorship(records) {
  return `<section class="watcher-record-grid" aria-label="Censorship signals">${records.map((record) => `<article class="watcher-record-card watcher-censorship-card"><div>${icon("eye-off")}<span><strong>${escapeHtml(record.signalKind)}</strong><small>${escapeHtml(record.status.replaceAll("_", " "))}</small></span></div><p>${escapeHtml(record.detail)}</p><dl><div><dt>Fixture window</dt><dd>${escapeHtml(record.observationWindow)}</dd></div></dl><code>${escapeHtml(record.mapping)}</code></article>`).join("")}</section>
    <p class="watcher-boundary-note">${icon("alert")} No global censorship score is inferred from a local or deterministic observation window.</p>`;
}

function watcherEvidence(records) {
  return `<section class="watcher-evidence-list" aria-label="Sanitized Watcher evidence">${records.map((record) => `<article class="watcher-evidence-card${record.alertId === state.watcherSelectedAlertId ? " is-selected" : ""}">
    <div class="watcher-detail-heading"><div><p class="eyebrow">EvidenceKey · sequence ${record.sequence}</p><h4>${escapeHtml(record.alertKind)}</h4></div><span class="status-badge">${escapeHtml(watcherSeverityLabel(record.severity))}</span></div>
    <dl><div><dt>Batch</dt><dd><code>${escapeHtml(record.batchId)}</code></dd></div><div><dt>Checkpoint</dt><dd><code>${escapeHtml(record.checkpointId)}</code></dd></div><div><dt>DA reference</dt><dd><code>${escapeHtml(record.providerRef || "None")}</code></dd></div><div><dt>Bindings</dt><dd>${record.bindings.map((binding) => `<code>${escapeHtml(binding)}</code>`).join(" ")}</dd></div></dl>
    <button class="button button-primary" type="button" data-watcher-action="export-evidence" data-alert-id="${escapeHtml(record.alertId)}">${icon("backup")} Prepare sanitized export</button>
  </article>`).join("")}</section>
  ${state.watcherExportEnvelope ? `<section class="watcher-export-result" aria-labelledby="watcher-export-title"><div><span>${icon("check")}</span><span><strong id="watcher-export-title">Sanitized fixture envelope prepared</strong><small>No wallet-private or secret field is present. Production save/share remains a native boundary.</small></span></div><pre>${escapeHtml(JSON.stringify(state.watcherExportEnvelope, null, 2))}</pre></section>` : ""}`;
}

function watcherContent(tabId, records) {
  if (tabId === "overview") return watcherOverview(records);
  if (tabId === "alerts") return watcherAlerts(records);
  if (tabId === "publication") return watcherPublication(records);
  if (tabId === "providers") return watcherProviders(records);
  if (tabId === "censorship") return watcherCensorship(records);
  return watcherEvidence(records);
}

function watchersTelemetryView() {
  const observation = watcherObservation();
  const routeNode = demoRuntime.navigationNodeForRoute(state.activeRoute);
  const tabLabel = routeNode ? navigationLabel(routeNode) : "Overview";
  const records = observation.data?.records || [];
  const stateNotice = watcherStateNotice(observation);
  const content = ["success", "degraded"].includes(observation.status) && records.length
    ? watcherContent(state.watchersTelemetryTab, records)
    : "";
  const panel = `<section class="telemetry-view watcher-roadmap" data-watcher-screen="${escapeHtml(state.watchersTelemetryTab)}" data-watcher-result="${escapeHtml(observation.status)}" aria-labelledby="watcher-screen-title">
    ${watcherControls()}
    <section class="telemetry-tab-detail watcher-tab-detail">
      <div class="telemetry-tab-heading"><div><h3 id="watcher-screen-title">${escapeHtml(tabLabel)}</h3><p>Deterministic, privacy-safe observation and evidence workflow.</p></div><span class="status-badge">${escapeHtml(watcherScenarioLabels[observation.status])}</span></div>
      ${stateNotice}
      ${content}
    </section>
  </section>`;
  const helpTopicOverride = state.watchersTelemetryTab === "alerts" && state.watcherSelectedAlertId
    ? "telemetry.watchers.alert-detail"
    : "";
  return workspaceFrame("telemetry.watchers", panel, helpTopicOverride);
}

const explorerEvidenceKinds = Object.freeze([
  ["all", "All public evidence"],
  ["publication", "Publications"],
  ["proof", "Proof envelopes"],
  ["da_reference", "Opaque DA references"]
]);

function explorerObservation() {
  const routeId = state.activeRoute.startsWith("telemetry.explorer.")
    ? state.activeRoute
    : "telemetry.explorer.overview";
  return telemetryGateway.readExplorerView({
    routeId,
    scenario: state.explorerScenario,
    generation: Number(state.requestGenerations[`telemetry:${routeId}`] || 0),
    filters: { kind: state.explorerEvidenceKindFilter }
  });
}

function explorerControls() {
  const showEvidenceFilter = state.explorerTelemetryTab === "evidence";
  return `<section class="watcher-toolbar explorer-toolbar" aria-label="${escapeHtml(t("plan2.aria.explorerControls"))}">
    <label><span>Scenario</span><select data-explorer-control="scenario" aria-label="Explorer scenario">${telemetryGateway.scenarioIds.map((scenario) => `<option value="${escapeHtml(scenario)}"${scenario === state.explorerScenario ? " selected" : ""}>${escapeHtml(watcherScenarioLabels[scenario])}</option>`).join("")}</select></label>
    ${showEvidenceFilter ? `<label><span>Evidence kind</span><select data-explorer-control="kind" aria-label="Filter public evidence">${explorerEvidenceKinds.map(([id, label]) => `<option value="${id}"${id === state.explorerEvidenceKindFilter ? " selected" : ""}>${label}</option>`).join("")}</select></label>` : ""}
  </section>`;
}

function explorerStateNotice(observation) {
  const isSearch = state.explorerTelemetryTab === "search";
  if (observation.status === "success" && (isSearch || observation.data?.total > 0)) return "";
  if (observation.status === "degraded" && observation.data?.total > 0) {
    return `<div class="watcher-state-notice is-warning" role="status">${icon("alert")}<span><strong>Stale public fixture withheld from search</strong><small>${escapeHtml(observation.issue.message)} Current visible rows are partial context only.</small></span><button class="button" type="button" data-explorer-action="recover">Refresh fixture</button></div>`;
  }
  if (observation.status === "loading") {
    return `<div class="watcher-state-panel" data-explorer-state="loading" role="status" aria-live="polite"><div class="watcher-loading-copy"><span class="watcher-loading-indicator" aria-hidden="true"></span><span><strong>Loading public evidence</strong><p>No cached or private fallback is promoted while the fixture request is pending.</p></span></div><button class="button" type="button" data-explorer-action="recover">Complete fixture request</button></div>`;
  }
  const filteredEmpty = observation.status === "success" && observation.data?.total === 0;
  const title = filteredEmpty ? "No matching public evidence" : {
    unavailable: "Public evidence source unavailable",
    empty: "No public evidence in this fixture",
    malformed: "Malformed evidence payload rejected",
    error: "Public evidence query failed",
    degraded: "Current public evidence withheld"
  }[observation.status] || "No public evidence";
  const message = filteredEmpty
    ? "The selected evidence-kind filter excludes every public fixture record."
    : observation.issue?.message || "No validated public observation is available.";
  const action = filteredEmpty || observation.status === "empty"
    ? `<button class="button" type="button" data-explorer-action="clear-filter">Clear filter</button>`
    : `<button class="button button-primary" type="button" data-explorer-action="recover">Retry with fixture</button>`;
  return `<div class="watcher-state-panel${["malformed", "error"].includes(observation.status) ? " is-danger" : ""}" data-explorer-state="${escapeHtml(filteredEmpty ? "empty" : observation.status)}" role="status"><div>${icon(["malformed", "error"].includes(observation.status) ? "alert" : "eye-off")}<strong>${escapeHtml(title)}</strong><p>${escapeHtml(message)}</p>${observation.issue?.recoveryAction ? `<small>${escapeHtml(observation.issue.recoveryAction)}</small>` : ""}</div>${action}</div>`;
}

function explorerOverview(records) {
  const summary = records[0];
  if (!summary) return "";
  return `<section class="network-summary-grid telemetry-summary explorer-summary" aria-label="Explorer public scope">
    ${telemetryValue("Public checkpoints", String(summary.checkpointCount), "Lifecycle and publication evidence only")}
    ${telemetryValue("Published batches", String(summary.batchCount), "Public batch/checkpoint relationships")}
    ${telemetryValue("Evidence records", String(summary.evidenceCount), "Proof, publication, route, and opaque DA references")}
    ${telemetryValue("Wallet-local fields", "Never accepted", "No labels, balances, messages, paths, receivers, or memos")}
  </section>
  <section class="watcher-mapping-card explorer-scope-card"><div>${icon("shield")}<span><strong>Narrow public scope</strong><small><code>check_public_checkpoint_v1</code><span>and</span><code>check_publication_route_v1</code></small></span></div><p>Search is allowlisted by typed public-ID family. Unknown, private-looking, malformed, unsupported, and stale identifiers fail closed without echoing the rejected input.</p></section>`;
}

function explorerRecordLabel(record) {
  if (record.recordType === "checkpoint") return `Checkpoint · ${record.lifecycleStatus.replaceAll("_", " ")}`;
  if (record.recordType === "batch") return `Batch · checkpoint ${record.publicationCheckpoint}`;
  if (record.recordType === "publication") return `Publication · ${record.state.replaceAll("_", " ")}`;
  if (record.recordType === "proof") return `Proof envelope · ${record.proofFamily}`;
  return `Opaque DA reference · ${record.providerFamily}`;
}

function explorerRecordCard(record) {
  const selected = state.explorerSelectedPublicId === record.publicId;
  const helper = record.recordType === "checkpoint"
    ? `${record.batchIds.length} public batch relationship`
    : record.recordType === "batch"
      ? `${record.checkpointId} · ${record.relationship}`
      : record.mapping;
  return `<button class="explorer-record-card${selected ? " is-selected" : ""}" type="button" data-explorer-record="${escapeHtml(record.publicId)}"${selected ? ' aria-current="true"' : ""}>
    <span class="explorer-record-icon">${icon(record.recordType === "proof" ? "shield" : record.recordType === "da_reference" ? "network" : "copy")}</span>
    <span><strong>${escapeHtml(explorerRecordLabel(record))}</strong><code>${escapeHtml(record.publicId)}</code><small>${escapeHtml(helper)}</small></span>
  </button>`;
}

function explorerRelatedIds(record) {
  return [...new Set([
    ...(record.batchIds || []),
    ...(record.publicEvidenceIds || []),
    ...(record.checkpointIds || []),
    record.checkpointId,
    record.batchId,
    record.publicationId,
    record.proofId,
    record.daReferenceId
  ].filter((id) => id && id !== record.publicId))];
}

function explorerTechnicalDto(record) {
  if (record.recordType === "checkpoint") return {
    publicId: record.publicId,
    lifecycleStatus: record.lifecycleStatus,
    publicRoot: record.publicRoot,
    priorPublicRoot: record.priorPublicRoot,
    publicationEvidenceRoot: record.publicationEvidenceRoot,
    publicationHeight: record.publicationHeight,
    challengeWindowStartHeight: record.challengeWindowStartHeight,
    mapping: record.mapping
  };
  if (record.recordType === "batch") return {
    publicId: record.publicId,
    checkpointId: record.checkpointId,
    publicationId: record.publicationId,
    proofId: record.proofId,
    daReferenceId: record.daReferenceId,
    publicationCheckpoint: record.publicationCheckpoint,
    routeGeneration: record.routeGeneration,
    shardIds: record.shardIds,
    mapping: record.mapping
  };
  if (record.recordType === "publication") return {
    publicId: record.publicId,
    checkpointId: record.checkpointId,
    batchId: record.batchId,
    state: record.state,
    publicRoot: record.publicRoot,
    routeSnapshot: record.routeSnapshot,
    daReferenceId: record.daReferenceId,
    mapping: record.mapping
  };
  if (record.recordType === "proof") return {
    publicId: record.publicId,
    checkpointId: record.checkpointId,
    publicationId: record.publicationId,
    publicRoot: record.publicRoot,
    rootGeneration: record.rootGeneration,
    proofFamily: record.proofFamily,
    shardLeafIndex: record.shardLeafIndex,
    verificationBoundary: record.verificationBoundary,
    mapping: record.mapping
  };
  return {
    publicId: record.publicId,
    checkpointIds: record.checkpointIds,
    providerFamily: record.providerFamily,
    locatorKind: record.locatorKind,
    opaqueProviderRef: record.opaqueProviderRef,
    publishedHeight: record.publishedHeight,
    payloadCommitment: record.payloadCommitment,
    archiveManifestRoot: record.archiveManifestRoot,
    mapping: record.mapping
  };
}

function explorerRecordDetail(record) {
  if (!record) return `<section class="watcher-detail-empty explorer-detail-empty"><strong>Select public evidence</strong><p>Choose a record to inspect only its allowlisted public summary and technical DTO.</p></section>`;
  const related = explorerRelatedIds(record);
  const summaryRows = record.recordType === "checkpoint"
    ? [
        ["Lifecycle", record.lifecycleStatus.replaceAll("_", " ")],
        ["Public root", record.publicRoot],
        ["Publication evidence", record.publicationState.replaceAll("_", " ")],
        ["Observed", record.observedAt]
      ]
    : record.recordType === "batch"
      ? [
          ["Checkpoint", record.checkpointId],
          ["Publication checkpoint", String(record.publicationCheckpoint)],
          ["Route generation", String(record.routeGeneration)],
          ["Relationship", record.relationship]
        ]
      : record.recordType === "publication"
        ? [
            ["Checkpoint", record.checkpointId],
            ["Public root", record.publicRoot],
            ["Route generation", String(record.routeSnapshot.routingGeneration)],
            ["Shard IDs", record.routeSnapshot.shardIds.join(", ")]
          ]
        : record.recordType === "proof"
          ? [
              ["Checkpoint", record.checkpointId],
              ["Public root", record.publicRoot],
              ["Proof family", record.proofFamily],
              ["Verification", record.verificationBoundary]
            ]
          : [
              ["Provider family", record.providerFamily],
              ["Locator kind", record.locatorKind],
              ["Opaque provider ref", record.opaqueProviderRef],
              ["Published height", String(record.publishedHeight)]
            ];
  return `<section class="watcher-alert-detail explorer-record-detail" data-explorer-detail="${escapeHtml(record.publicId)}" aria-labelledby="explorer-record-detail-title">
    <div class="watcher-detail-heading"><div><p class="eyebrow">Public ${escapeHtml(record.recordType.replaceAll("_", " "))}</p><h4 id="explorer-record-detail-title"><code>${escapeHtml(record.publicId)}</code></h4></div><span class="status-badge">Fixture</span></div>
    <div class="explorer-detail-toggle" role="group" aria-label="${escapeHtml(t("plan2.aria.explorerDetail"))}">
      <button class="button${state.explorerDetailMode === "summary" ? " is-selected" : ""}" type="button" data-explorer-action="summary" aria-pressed="${state.explorerDetailMode === "summary"}">Summary</button>
      <button class="button${state.explorerDetailMode === "technical" ? " is-selected" : ""}" type="button" data-explorer-action="technical" aria-pressed="${state.explorerDetailMode === "technical"}">Technical details</button>
    </div>
    ${state.explorerDetailMode === "technical"
      ? `<pre class="explorer-technical-json">${escapeHtml(JSON.stringify(explorerTechnicalDto(record), null, 2))}</pre>`
      : `<dl>${summaryRows.map(([label, value]) => `<div><dt>${escapeHtml(label)}</dt><dd>${String(value).includes("_") || String(value).includes("root") || String(value).startsWith("check_") ? `<code>${escapeHtml(value)}</code>` : escapeHtml(value)}</dd></div>`).join("")}</dl>`}
    ${related.length ? `<div class="explorer-related"><strong>Related public IDs</strong><div>${related.map((id) => `<button class="button button-quiet" type="button" data-explorer-open-id="${escapeHtml(id)}"><code>${escapeHtml(id)}</code></button>`).join("")}</div></div>` : ""}
    <p class="watcher-boundary-note">${icon("shield")} This detail is built from an explicit public DTO; wallet state is not an input.</p>
  </section>`;
}

function explorerRecordBrowser(records, label) {
  const selected = records.find(({ publicId }) => publicId === state.explorerSelectedPublicId);
  return `<div class="watcher-split-layout explorer-split-layout">
    <section class="explorer-record-list" aria-label="${escapeHtml(label)}">${records.map(explorerRecordCard).join("")}</section>
    ${explorerRecordDetail(selected)}
  </div>`;
}

function explorerSearchResult() {
  const result = state.explorerSearchResult;
  if (!result) return `<section class="watcher-detail-empty explorer-search-empty"><strong>Search the public proof surface</strong><p>The input is validated locally before any deterministic lookup. Rejected private-looking input is neither queried nor echoed.</p></section>`;
  if (result.status === "found") return explorerRecordDetail(result.record);
  const title = {
    private: "Private identifier rejected",
    malformed: "Malformed public identifier",
    unsupported: "Unsupported identifier family",
    unknown: "Public identifier not found",
    stale: "Stale public evidence withheld",
    degraded: "Public source degraded",
    unavailable: "Public source unavailable",
    loading: "Public source is loading",
    error: "Public search failed"
  }[result.status] || "Public search rejected";
  return `<section class="watcher-state-panel${["private", "malformed", "error"].includes(result.status) ? " is-danger" : ""}" data-explorer-search-status="${escapeHtml(result.status)}" role="status"><div>${icon(["private", "malformed", "error"].includes(result.status) ? "alert" : "eye-off")}<strong>${escapeHtml(title)}</strong><p>${escapeHtml(result.issue?.message || "No public result is available.")}</p><small>${escapeHtml(result.issue?.recoveryAction || "Use a supported public identifier.")}</small></div><button class="button" type="button" data-explorer-action="clear-search">Clear search</button></section>`;
}

function explorerSearch() {
  const examples = ["checkpoint_000184", "batch_4f91c7a0", "publication_6f840184", "proof_92840184", "da_ref_72be91"];
  return `<section class="explorer-search-workflow" aria-labelledby="explorer-search-heading">
    <form class="explorer-search-form" id="explorer-public-search" autocomplete="off" novalidate>
      <div class="field-group"><label class="field-label" for="explorer-public-id">Supported public ID</label><div class="explorer-search-row"><input id="explorer-public-id" name="publicId" value="${escapeHtml(state.explorerQuery)}" maxlength="80" autocapitalize="none" spellcheck="false" placeholder="checkpoint_000184" aria-describedby="explorer-search-hint"><button class="button button-primary" type="submit">${icon("search")} Search</button></div><p class="field-hint" id="explorer-search-hint">Checkpoint, batch, publication, proof, or opaque DA-reference IDs only.</p></div>
      <div class="explorer-search-examples" aria-label="Public ID examples">${examples.map((id) => `<button class="button button-quiet" type="button" data-explorer-example-id="${id}"><code>${id}</code></button>`).join("")}</div>
    </form>
    ${explorerSearchResult()}
  </section>`;
}

function explorerContent(tabId, records) {
  if (tabId === "overview") return explorerOverview(records);
  if (tabId === "search") return explorerSearch();
  if (tabId === "checkpoints") return explorerRecordBrowser(records, "Public checkpoints");
  if (tabId === "batches") return explorerRecordBrowser(records, "Published batches");
  return explorerRecordBrowser(records, "Public proof and publication evidence");
}

function explorerTelemetryView() {
  const observation = explorerObservation();
  const routeNode = demoRuntime.navigationNodeForRoute(state.activeRoute);
  const tabLabel = routeNode ? navigationLabel(routeNode) : "Overview";
  const records = observation.data?.records || [];
  const stateNotice = explorerStateNotice(observation);
  const canRenderContent = observation.status === "success"
    || (observation.status === "degraded" && records.length > 0 && state.explorerTelemetryTab !== "search");
  const panel = `<section class="telemetry-view explorer-roadmap" data-explorer-screen="${escapeHtml(state.explorerTelemetryTab)}" data-explorer-result="${escapeHtml(observation.status)}" aria-labelledby="explorer-screen-title">
    ${explorerControls()}
    <section class="telemetry-tab-detail watcher-tab-detail explorer-tab-detail">
      <div class="telemetry-tab-heading"><div><h3 id="explorer-screen-title">${escapeHtml(tabLabel)}</h3><p>Deterministic, privacy-restricted public evidence workflow.</p></div><span class="status-badge">${escapeHtml(watcherScenarioLabels[observation.status])}</span></div>
      ${stateNotice}
      ${canRenderContent ? explorerContent(state.explorerTelemetryTab, records) : ""}
    </section>
  </section>`;
  return workspaceFrame(
    "telemetry.explorer",
    panel,
    state.explorerSelectedPublicId ? "telemetry.explorer.detail" : ""
  );
}

function telemetryView() {
  const source = state.telemetrySource;
  if (source === "reticulum") return reticulumTelemetryView();
  if (source === "onionnet") return onionnetTelemetryView();
  if (source === "aggregators") return aggregatorsTelemetryView();
  if (source === "watchers") return watchersTelemetryView();
  return explorerTelemetryView();
}

function dappTitleCase(value) {
  return String(value || "")
    .replaceAll("_", " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function dappDateTime(value) {
  return formatLocalizedDateTime(new Date(value), { dateStyle: "medium", timeStyle: "short" });
}

function dappStatusBadge(status) {
  const className = {
    active: "is-ready",
    approved: "is-ready",
    accepted: "is-ready",
    expiring: "is-warning",
    pending: "is-warning",
    expired: "",
    rejected: "is-error",
    revoked: "is-error"
  }[status] || "";
  const stateKeys = new Set([
    "active",
    "approved",
    "accepted",
    "expiring",
    "pending",
    "expired",
    "rejected",
    "revoked"
  ]);
  const label = stateKeys.has(status) ? t(`plan2.states.${status}`) : dappTitleCase(status);
  return `<span class="status-badge ${className}">${escapeHtml(label)}</span>`;
}

function dappObjectFamilyChips(families) {
  return `<div class="dapp-chip-list">${families.map((family) => `<span class="dapp-chip">${escapeHtml(dappTitleCase(family))}</span>`).join("")}</div>`;
}

function dappCard(entry) {
  return `<article class="dapp-card" data-dapp-card="${escapeHtml(entry.id)}">
    <div class="dapp-card-heading">
      <span class="dapp-card-icon" aria-hidden="true">${icon(entry.iconName)}</span>
      <div><p class="eyebrow">${escapeHtml(dappTitleCase(entry.useCaseFamily))}</p><h3>${escapeHtml(entry.label)}</h3></div>
    </div>
    <p>${escapeHtml(entry.summary)}</p>
    <dl class="dapp-card-metadata">
      <div><dt>Maturity</dt><dd>${escapeHtml(dappTitleCase(entry.maturity))}</dd></div>
      <div><dt>Availability</dt><dd>${escapeHtml(dappTitleCase(entry.availability))}</dd></div>
      <div><dt>Publisher</dt><dd>${escapeHtml(entry.publisher.label)} · Unverified</dd></div>
    </dl>
    <div class="dapp-card-section"><strong>Requested objects</strong>${dappObjectFamilyChips(entry.requestedObjectFamilies)}</div>
    <div class="dapp-card-section"><strong>Offline behavior</strong><p>${escapeHtml(entry.offlineBehavior.summary)}</p></div>
    <div class="dapp-card-section"><strong>Data disclosed</strong><p>${escapeHtml(entry.disclosures.map(dappTitleCase).join(" · "))}</p></div>
    <div class="dapp-card-actions">
      <button class="button button-primary" type="button" data-dapp-action="route" data-dapp-route="${escapeHtml(entry.routeId)}">Open interface</button>
      <button class="button button-quiet" type="button" data-dapp-action="open" data-dapp-id="${escapeHtml(entry.id)}">${escapeHtml(t("plan2.actions.openDetails"))}</button>
      <button class="button button-quiet" type="button" data-help-topic="${escapeHtml(entry.helpTopicId)}">Help ↗</button>
    </div>
  </article>`;
}

function dappScreenHeading(title, copy, meta = "") {
  return `<header class="dapp-screen-heading"><div><p class="eyebrow">Local fixture workflow</p><h2>${escapeHtml(title)}</h2><p>${escapeHtml(copy)}</p></div>${meta}</header>`;
}

function dappDiscoverScreen() {
  return `${dappScreenHeading("Discover dApps", "Seventeen curated Z00Z typed-action interfaces. Every card is a bundled local descriptor, never remotely executed application code.", `<span class="status-badge">${demoRuntime.DAPP_CATALOG.length} descriptors</span>`)}
    <section class="dapp-catalog-grid" aria-label="${escapeHtml(t("plan2.aria.dappCatalogue"))}">${demoRuntime.DAPP_CATALOG.map((entry) => dappCard(entry)).join("")}</section>`;
}

function dappProposalField(entry, field) {
  const controlId = `dapp-${entry.id}-${field.id}`;
  const required = field.required ? " required" : "";
  const describedBy = field.suffix ? ` aria-describedby="${escapeHtml(`${controlId}-hint`)}"` : "";
  const control = field.type === "select"
    ? `<select id="${escapeHtml(controlId)}" name="${escapeHtml(field.id)}"${describedBy}${required}>${field.options.map((option) => `<option>${escapeHtml(option)}</option>`).join("")}</select>`
    : `<input id="${escapeHtml(controlId)}" name="${escapeHtml(field.id)}" type="${field.type === "number" ? "number" : "text"}"${field.type === "number" ? ` min="${escapeHtml(field.min ?? "0")}" step="${escapeHtml(field.step || "any")}" inputmode="${field.integer ? "numeric" : "decimal"}"` : ""}${field.placeholder ? ` placeholder="${escapeHtml(field.placeholder)}"` : ""}${describedBy}${required}>`;
  return `<div class="field-group"><label class="field-label" for="${escapeHtml(controlId)}">${escapeHtml(field.label)}</label>${control}${field.suffix ? `<p class="field-hint" id="${escapeHtml(`${controlId}-hint`)}">${escapeHtml(field.suffix)}</p>` : ""}</div>`;
}

function dappProposalFormValues(form) {
  return Object.fromEntries([...new FormData(form).entries()].map(([key, value]) => [key, String(value).trim()]));
}

function validateDappProposalForm(form, descriptor) {
  const error = form.querySelector("#dapp-proposal-error");
  const controls = [...form.elements].filter((control) => control instanceof HTMLInputElement || control instanceof HTMLSelectElement);
  controls.forEach((control) => control.setCustomValidity(""));
  if (error) error.textContent = "";

  const invalidControl = controls.find((control) => control.willValidate && !control.checkValidity());
  if (invalidControl) {
    invalidControl.focus();
    invalidControl.reportValidity();
    return false;
  }

  const values = dappProposalFormValues(form);
  const fail = (fieldId, message) => {
    const control = form.elements.namedItem(fieldId);
    if (control instanceof HTMLInputElement || control instanceof HTMLSelectElement) {
      control.setCustomValidity(message);
      control.focus();
      control.reportValidity();
    }
    if (error) error.textContent = message;
    return false;
  };

  if (descriptor.id === "agents-budget") {
    const periodLimit = Number(values["period-limit"]);
    const actionLimit = Number(values["action-limit"]);
    const approvalThreshold = Number(values.approval);
    if (actionLimit > periodLimit) {
      return fail("action-limit", "Maximum per action cannot exceed the daily budget.");
    }
    if (approvalThreshold > actionLimit) {
      return fail("approval", "Human approval threshold cannot exceed the per-action ceiling.");
    }
  }
  if (descriptor.id === "create-asset"
    && values.class === "NFT"
    && Number(values.decimals) !== 0) {
    return fail("decimals", "NFT definitions must use zero decimals.");
  }
  if (descriptor.id === "wbold-gateway"
    && values.direction.startsWith("Redeem")
    && !values["external-recipient"]) {
    return fail("external-recipient", "External recipient is required for a wBOLD redemption.");
  }
  if (descriptor.id === "assets-locker"
    && values.action.startsWith("Consume")
    && !values["external-recipient"]) {
    return fail("external-recipient", "External recipient is required when consuming a right to redeem.");
  }
  return true;
}

function dappProposalScreen(entry) {
  const stages = [
    ["Typed proposal", entry.intentType],
    ["Scope check", entry.walletChecks.join(" · ")],
    ["Package build", "Wallet selects eligible objects and constructs the package"],
    ["Confirmation", "Wallet shows value, fee, disclosure, and settlement assumptions"],
    ["Settlement path", entry.settlementPath]
  ];
  return `<section class="dapp-proposal" data-dapp-proposal="${escapeHtml(entry.id)}" data-intent-type="${escapeHtml(entry.intentType)}">
    <header class="dapp-detail-heading">
      <span class="dapp-card-icon is-large">${icon(entry.iconName)}</span>
      <div><p class="eyebrow">${escapeHtml(dappTitleCase(entry.useCaseFamily))}</p><h2>${escapeHtml(entry.label)}</h2><p>${escapeHtml(entry.summary)}</p></div>
      ${dappStatusBadge(entry.maturity)}
    </header>
    <div class="dapp-architecture-boundary">${icon("shield")}<span><strong>dApp proposes; Wallet decides</strong><small>A Z00Z dApp does not control the wallet. It proposes a typed action. Wallet checks scope, builds the package, requests confirmation, and only then passes it to the settlement path.</small></span></div>
    <div class="dapp-proposal-layout">
      <form class="dapp-proposal-form" id="dapp-action-proposal-form" data-dapp-id="${escapeHtml(entry.id)}" autocomplete="off" novalidate>
        <div class="dapp-proposal-form-heading"><div><p class="eyebrow">Typed action</p><h3>Prepare proposal</h3></div><code>${escapeHtml(entry.intentType)}</code></div>
        <dl class="dapp-creation-summary">
          <div><dt>What this creates</dt><dd>${escapeHtml(entry.createdArtifact)}</dd></div>
          <div><dt>Why create it</dt><dd>${escapeHtml(entry.purpose)}</dd></div>
        </dl>
        <div class="dapp-proposal-fields">${entry.proposalFields.map((field) => dappProposalField(entry, field)).join("")}</div>
        <p class="field-error dapp-proposal-error" id="dapp-proposal-error" role="alert"></p>
        <div class="capability-note">${icon("alert")}<span><strong>Review boundary</strong><small>${escapeHtml(entry.reviewBoundary)}</small></span></div>
        <button class="button button-primary dapp-proposal-submit" type="submit">${icon(entry.iconName)} ${escapeHtml(entry.actionLabel)}</button>
      </form>
      <aside class="dapp-proposal-review" aria-label="Wallet review boundary">
        <div><p class="eyebrow">Wallet boundary</p><h3>What happens next</h3></div>
        <ol class="dapp-proposal-stages">${stages.map(([label, detail], index) => `<li><span>${index + 1}</span><div><strong>${escapeHtml(label)}</strong><small>${escapeHtml(detail)}</small></div></li>`).join("")}</ol>
        <dl class="dapp-proposal-output"><div><dt>Requested objects</dt><dd>${escapeHtml(entry.requestedObjectFamilies.map(dappTitleCase).join(", "))}</dd></div><div><dt>Evidence output</dt><dd>${escapeHtml(entry.evidenceOutput)}</dd></div><div><dt>Remote code</dt><dd>Never loaded</dd></div></dl>
        <button class="button button-quiet" type="button" data-help-topic="${escapeHtml(entry.helpTopicId)}">Open Help ↗</button>
      </aside>
    </div>
  </section>`;
}

function dappDetailScreen() {
  const entry = demoRuntime.dappDescriptor(state.dappSelectedId);
  if (!entry) {
    state.dappScreen = "list";
    return dappDiscoverScreen();
  }
  const reviewableConnection = demoRuntime.DAPP_CONNECTION_FIXTURES.find(({ descriptorId, status }) => descriptorId === entry.id && status === "pending");
  return `<section class="dapp-detail" data-dapp-detail="${escapeHtml(entry.id)}">
    <button class="button button-quiet dapp-back-button" type="button" data-dapp-action="back">← Back to ${escapeHtml(dappTitleCase(state.dappSection))}</button>
    <div class="dapp-detail-heading"><span class="dapp-card-icon is-large">${icon(entry.iconName)}</span><div><p class="eyebrow">${escapeHtml(dappTitleCase(entry.useCaseFamily))}</p><h2>${escapeHtml(entry.label)}</h2><p>${escapeHtml(entry.summary)}</p></div></div>
    <div class="capability-note">${icon("alert")}<span><strong>${escapeHtml(dappTitleCase(entry.maturity))} · ${escapeHtml(dappTitleCase(entry.availability))}</strong><small>${escapeHtml(entry.reviewBoundary)}</small></span></div>
    <div class="dapp-detail-grid">
      <section class="dapp-detail-panel"><h3>Trust and execution</h3><dl class="dapp-detail-list"><div><dt>Publisher</dt><dd>${escapeHtml(entry.publisher.label)}</dd></div><div><dt>Provenance</dt><dd>${escapeHtml(dappTitleCase(entry.publisher.provenance))}</dd></div><div><dt>Verified</dt><dd>No</dd></div><div><dt>Execution</dt><dd>Typed intent only · no wallet bridge</dd></div></dl></section>
      <section class="dapp-detail-panel"><h3>Requested capability</h3><dl class="dapp-detail-list"><div><dt>Intent</dt><dd><code>${escapeHtml(entry.intentType)}</code></dd></div><div><dt>Objects</dt><dd>${escapeHtml(entry.requestedObjectFamilies.map(dappTitleCase).join(", "))}</dd></div><div><dt>Value path</dt><dd>${escapeHtml(dappTitleCase(entry.valuePath))}</dd></div><div><dt>Fee path</dt><dd>${escapeHtml(dappTitleCase(entry.feePath))}</dd></div></dl></section>
      <section class="dapp-detail-panel"><h3>Offline behavior</h3><p>${escapeHtml(entry.offlineBehavior.summary)}</p><p class="dapp-detail-key">${escapeHtml(dappTitleCase(entry.offlineBehavior.mode))}</p></section>
      <section class="dapp-detail-panel"><h3>Data disclosed</h3>${dappObjectFamilyChips(entry.disclosures)}<p>No raw wallet object, seed, key, session, or arbitrary local path is shared.</p></section>
    </div>
    <div class="dapp-card-actions">${reviewableConnection ? `<button class="button button-primary" type="button" data-dapp-action="review" data-connection-id="${escapeHtml(reviewableConnection.id)}">Review pending request</button>` : ""}<button class="button button-quiet" type="button" data-help-topic="${escapeHtml(entry.helpTopicId)}">Help ↗</button></div>
  </section>`;
}

function dappReviewScreen() {
  const result = dappGateway.readPermissionReview({ connectionId: state.dappReviewConnectionId });
  const review = result.ok ? result.data : null;
  const descriptor = review ? demoRuntime.dappDescriptor(review.descriptorId) : null;
  if (!review || !descriptor) {
    state.dappScreen = "list";
    return dappDiscoverScreen();
  }
  const acknowledgements = state.dappReviewAcknowledgements || {
    scopeConfirmed: false,
    reauthAcknowledged: false
  };
  return `<section class="dapp-review" data-dapp-review="${escapeHtml(review.connectionId)}" data-dapp-review-id="${escapeHtml(review.reviewId)}">
    <button class="button button-quiet dapp-back-button" type="button" data-dapp-action="back">← Back</button>
    <header class="dapp-detail-heading"><span class="dapp-card-icon is-large">${icon(descriptor.iconName)}</span><div><p class="eyebrow">Permission review</p><h2>${escapeHtml(review.appIdentity.label)}</h2><p>${escapeHtml(review.intent.humanReadable)}</p></div></header>
    <div class="confirmation-note">${icon("shield")} Accepting this fixture creates only an app-level decision. It cannot sign, settle, transfer ownership, or mutate a Wallet object.</div>
    <form id="dapp-permission-review-form" class="dapp-review-form" autocomplete="off" novalidate>
      <dl class="dapp-review-grid">
        <div><dt>${escapeHtml(t("plan2.permission.appIdentity"))}</dt><dd>${escapeHtml(review.appIdentity.label)} · ${escapeHtml(review.appIdentity.publisher)} · unverified</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.action"))}</dt><dd>${escapeHtml(review.intent.action)}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.objectFamily"))}</dt><dd>${escapeHtml(dappTitleCase(review.permission.objectFamily))}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.exactScope"))}</dt><dd>${escapeHtml(review.permission.exactScope)}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.uses"))}</dt><dd>${escapeHtml(review.permission.uses)}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.expiry"))}</dt><dd>${escapeHtml(dappDateTime(review.permission.expiry))}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.delegation"))}</dt><dd>${escapeHtml(review.permission.delegation)}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.value"))}</dt><dd>${escapeHtml(review.value.display)}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.feePath"))}</dt><dd>${escapeHtml(review.fee.display)} · ${escapeHtml(review.fee.path)}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.dataDisclosed"))}</dt><dd>${escapeHtml(review.disclosures.join(", "))}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.revokeBehavior"))}</dt><dd>${escapeHtml(review.revoke.behavior)}</dd></div>
        <div><dt>${escapeHtml(t("plan2.permission.reauth"))}</dt><dd>${escapeHtml(review.reauth.behavior)}</dd></div>
      </dl>
      <section class="dapp-review-confirmations" aria-label="${escapeHtml(t("plan2.permission.confirmations"))}">
        <label class="checkbox-line"><input name="scopeConfirmed" type="checkbox"${acknowledgements.scopeConfirmed ? " checked" : ""}><span><strong>${escapeHtml(t("plan2.permission.confirmScope"))}</strong><small>The app cannot broaden objects, value, uses, delegation, or expiry after this review.</small></span></label>
        <label class="checkbox-line"><input name="reauthAcknowledged" type="checkbox"${acknowledgements.reauthAcknowledged ? " checked" : ""}><span><strong>${escapeHtml(t("plan2.permission.acknowledgeReauth"))}</strong><small>${review.reauth.required ? "A later value or fee path requires fresh authentication inside Wallet review." : "This intent has no value-bearing Wallet re-auth path."} dApps never collects the Wallet credential.</small></span></label>
      </section>
      <p class="field-error dapp-review-error" id="dapp-review-error" role="alert">${escapeHtml(state.dappReviewValidationError || "")}</p>
      <div class="dapp-review-actions"><button class="button button-quiet" type="button" data-dapp-action="decide" data-decision="rejected">${escapeHtml(t("plan2.actions.reject"))}</button><button class="button button-primary" type="submit">${escapeHtml(t("plan2.permission.acceptIntent"))}</button></div>
    </form>
  </section>`;
}

function dappOutcomeScreen() {
  const outcome = state.dappLastOutcome;
  if (!outcome) {
    state.dappScreen = "list";
    return dappDiscoverScreen();
  }
  const accepted = ["intent_accepted", "intent_proposed"].includes(outcome.kind);
  const iconName = accepted ? "check" : "close";
  const walletReviewAction = accepted && state.dappReviewDecision?.decision === "accepted"
    ? `<button class="button button-primary" type="button" data-dapp-action="wallet-review">Continue in Wallet review</button>`
    : "";
  return `<section class="dapp-outcome ${accepted ? "is-positive" : "is-negative"}" data-dapp-outcome-route="${escapeHtml(outcome.kind)}">
    <span class="result-icon">${icon(iconName)}</span>
    <p class="eyebrow">Deterministic local outcome</p>
    <h2>${escapeHtml(outcome.label)}</h2>
    <p>${escapeHtml(outcome.summary)}</p>
    <div class="capability-note">${icon("shield")}<span><strong>Wallet state unchanged</strong><small>This outcome stores presentation state only. An accepted typed intent can continue into a separate Wallet review without granting the dApp mutation authority.</small></span></div>
    <div class="dapp-card-actions">${walletReviewAction}<button class="button ${accepted ? "" : "button-primary"}" type="button" data-dapp-action="outcome-back" data-return-route="${escapeHtml(outcome.returnRoute)}">Back to ${escapeHtml(dappTitleCase(outcome.returnRoute.split(".").at(-1)))}</button></div>
  </section>`;
}

function dappsView() {
  const routeSection = state.activeRoute.startsWith("dapps.") ? state.activeRoute.split(".").at(-1) : state.dappSection;
  state.dappSection = routeSection;
  let content;
  if (state.dappScreen === "detail") content = dappDetailScreen();
  else if (state.dappScreen === "review") content = dappReviewScreen();
  else if (state.dappScreen === "outcome") content = dappOutcomeScreen();
  else if (demoRuntime.dappDescriptor(routeSection)) content = dappProposalScreen(demoRuntime.dappDescriptor(routeSection));
  else content = dappDiscoverScreen();
  const helpTopicOverride = state.dappScreen === "detail"
    ? "dapps.detail"
    : state.dappScreen === "review" ? "dapps.permission-review" : "";
  return `<section class="view-enter dapp-roadmap" data-dapp-screen="${escapeHtml(state.dappScreen)}" data-dapp-section="${escapeHtml(routeSection)}"${helpTopicOverride ? ` data-help-topic-override="${helpTopicOverride}"` : ""}>${content}</section>`;
}

function messengerRuntimeControls() {
  const relayResult = messengerGateway.readRelayState({ scenario: state.messengerRelayScenario });
  const relay = relayResult.ok ? relayResult.data : {
    availability: "unavailable",
    summary: relayResult.error.message
  };
  const contactHandoff = state.contactActionHandoff?.target?.routeId === state.activeRoute
    ? `<div class="capability-note messenger-contact-handoff" data-contact-messenger-handoff="${escapeHtml(state.contactActionHandoff.handoffId)}">${icon("shield")}<span><strong>Prepared for ${escapeHtml(state.contactActionHandoff.label)}</strong><small>The Contact supplied only a typed domain-specific reference. Messenger must revalidate it before any future compose or relay action.</small></span></div>`
    : "";
  return `<div class="messenger-relay-control" data-messenger-relay="${escapeHtml(state.messengerRelayScenario)}">
      <span>${icon(relay.availability === "unavailable" ? "alert" : "activity")}<span><strong>Relay ${escapeHtml(relay.availability)}</strong><small>${escapeHtml(relay.summary)}</small></span></span>
      <div>
        <button class="button button-quiet" type="button" data-messenger-action="relay-unavailable">Show unavailable</button>
        <button class="button" type="button" data-messenger-action="relay-recover">Retry locally</button>
      </div>
    </div>
    ${contactHandoff}`;
}

function messengerMessageState(message) {
  if (state.messengerDeletedIds.includes(message.id)) return "deleted";
  if (state.messengerReportedIds.includes(message.id)) return "reported";
  if (state.messengerAcknowledgedIds.includes(message.id)) return "acknowledged";
  return message.deliveryState;
}

function messengerMessageCard(message) {
  const status = messengerMessageState(message);
  const isBlocked = state.messengerBlockedSenders.includes(message.senderLabel);
  return `<article class="messenger-message-card${message.severity === "danger" ? " is-danger" : ""}" data-messenger-message="${escapeHtml(message.id)}">
    <button class="messenger-message-open" type="button" data-messenger-action="open" data-message-id="${escapeHtml(message.id)}">
      <span class="messenger-avatar" aria-hidden="true">${icon(message.kind === "request" ? "receive" : message.kind === "abuse" ? "alert" : "message")}</span>
      <span class="messenger-message-copy">
        <span><strong>${escapeHtml(message.senderLabel)}</strong><small>${escapeHtml(dappDateTime(message.createdAt))}</small></span>
        <b>${escapeHtml(message.subject)}</b>
        <small>${escapeHtml(message.preview)}</small>
      </span>
      ${dappStatusBadge(isBlocked ? "blocked" : status)}
    </button>
  </article>`;
}

function messengerFolderScreen(folder) {
  const result = messengerGateway.listMessages({
    folder,
    deletedIds: state.messengerDeletedIds,
    blockedSenders: state.messengerBlockedSenders
  });
  const items = result.ok ? result.data.items : [];
  const title = folder === "requests" ? "Requests" : "Inbox";
  const copy = folder === "requests"
    ? "Typed advisory proposals remain read-only until an explicit review and separate Wallet handoff."
    : "Short-lived local advisory items. Delivery and acknowledgement never imply payment settlement.";
  return `${dappScreenHeading(title, copy, `<span class="status-badge">${items.length} local items</span>`)}
    ${items.length
      ? `<section class="messenger-message-list" aria-label="${title}">${items.map(messengerMessageCard).join("")}</section>`
      : `<section class="empty-state messenger-empty-state"><h3>No visible local items</h3><p>Deleted and blocked items stay out of this presentation list. Wallet state is unchanged.</p></section>`}`;
}

function messengerConversationScreen() {
  return `${dappScreenHeading("Conversations", "Short-lived off-chain thread previews. Search and retention remain local concepts.", `<span class="status-badge">${demoRuntime.MESSENGER_CONVERSATIONS.length} threads</span>`)}
    <form class="messenger-local-search" id="messenger-conversation-search" autocomplete="off">
      <label class="field-label" for="messenger-search">Search local labels and safe previews</label>
      <div><input id="messenger-search" name="query" value="${escapeHtml(state.messengerQuery || "")}" maxlength="48" placeholder="Search conversations"><button class="button" type="submit">${icon("search")} Search</button></div>
    </form>
    <section class="messenger-record-grid">${demoRuntime.MESSENGER_CONVERSATIONS
      .filter((entry) => !state.messengerQuery || `${entry.label} ${entry.preview}`.toLocaleLowerCase().includes(state.messengerQuery.toLocaleLowerCase()))
      .map((entry) => `<article class="messenger-record-card" data-messenger-conversation="${escapeHtml(entry.id)}"><div><span class="messenger-avatar">${icon("message")}</span><span><strong>${escapeHtml(entry.label)}</strong><small>${escapeHtml(dappDateTime(entry.updatedAt))}</small></span></div><p>${escapeHtml(entry.preview)}</p><dl><div><dt>Retention</dt><dd>${escapeHtml(entry.retention)}</dd></div><div><dt>Items</dt><dd>${entry.messageCount}</dd></div></dl><div class="capability-note">${icon("shield")} <span><strong>Concept thread</strong><small>No durable mailbox, public presence, or live relay is connected.</small></span></div></article>`).join("")}</section>`;
}

function messengerSentScreen() {
  return `${dappScreenHeading("Sent", "Sent and transport states are advisory. A failed relay cannot roll back or confirm Wallet state.", `<span class="status-badge">${demoRuntime.MESSENGER_SENT.length} local items</span>`)}
    <section class="messenger-record-grid">${demoRuntime.MESSENGER_SENT.map((entry) => `<article class="messenger-record-card" data-messenger-sent="${escapeHtml(entry.id)}"><div><span class="messenger-avatar">${icon("send")}</span><span><strong>${escapeHtml(entry.subject)}</strong><small>${escapeHtml(dappDateTime(entry.updatedAt))}</small></span>${dappStatusBadge(entry.state)}</div><p>${escapeHtml(entry.summary)}</p></article>`).join("")}</section>`;
}

function messengerMessageDetail() {
  const result = messengerGateway.readMessage({ messageId: state.messengerSelectedMessageId });
  if (!result.ok) {
    state.messengerScreen = "list";
    return messengerFolderScreen(state.messengerSection);
  }
  const message = result.data.message;
  const request = message.request;
  return `<section class="messenger-detail" data-messenger-detail="${escapeHtml(message.id)}">
    <button class="button button-quiet" type="button" data-messenger-action="back">← Back</button>
    <header class="dapp-detail-heading"><span class="dapp-card-icon is-large">${icon(message.kind === "request" ? "receive" : message.kind === "abuse" ? "alert" : "message")}</span><div><p class="eyebrow">${escapeHtml(dappTitleCase(message.kind))}</p><h2>${escapeHtml(message.subject)}</h2><p>${escapeHtml(message.preview)}</p></div>${dappStatusBadge(messengerMessageState(message))}</header>
    <dl class="messenger-detail-grid">
      <div><dt>From</dt><dd>${escapeHtml(message.senderLabel)}</dd></div>
      <div><dt>Received</dt><dd>${escapeHtml(dappDateTime(message.createdAt))}</dd></div>
      <div><dt>Expires</dt><dd>${escapeHtml(dappDateTime(message.expiresAt))}</dd></div>
      <div><dt>Delivery</dt><dd>${escapeHtml(dappTitleCase(message.deliveryState))} · advisory only</dd></div>
      ${request ? `<div><dt>Request type</dt><dd>${escapeHtml(dappTitleCase(request.type))}</dd></div><div><dt>Exact scope</dt><dd>${escapeHtml(request.exactScope)}</dd></div>` : ""}
    </dl>
    <div class="confirmation-note">${icon("shield")} Opening this item recorded only local presentation state. No Wallet object, balance, ownership, or settlement status changed.</div>
    <div class="messenger-detail-actions">
      ${request ? `<button class="button button-primary" type="button" data-messenger-action="review" data-message-id="${escapeHtml(message.id)}">Review request</button>` : ""}
      <button class="button" type="button" data-messenger-action="acknowledge" data-message-id="${escapeHtml(message.id)}">Acknowledge locally</button>
      <button class="button" type="button" data-messenger-action="delete" data-message-id="${escapeHtml(message.id)}">Delete locally</button>
      <button class="button" type="button" data-messenger-action="block" data-message-id="${escapeHtml(message.id)}">Block sender</button>
      <button class="button button-danger" type="button" data-messenger-action="report" data-message-id="${escapeHtml(message.id)}">Report abuse</button>
    </div>
  </section>`;
}

function messengerRequestReview() {
  const result = messengerGateway.readRequestReview({ messageId: state.messengerSelectedMessageId });
  if (!result.ok) {
    state.messengerScreen = "detail";
    return messengerMessageDetail();
  }
  const review = result.data;
  return `<section class="messenger-review" data-messenger-review="${escapeHtml(review.reviewId)}">
    <button class="button button-quiet" type="button" data-messenger-action="detail">← Message</button>
    <header class="dapp-detail-heading"><span class="dapp-card-icon is-large">${icon("receive")}</span><div><p class="eyebrow">Advisory request review</p><h2>${escapeHtml(review.subject)}</h2><p>${escapeHtml(review.senderLabel)}</p></div>${dappStatusBadge(review.expired ? "expired" : "pending")}</header>
    <div class="confirmation-note">${icon("shield")} Accepting creates a typed Messenger decision only. Wallet revalidates the immutable handoff and owns every later mutation.</div>
    <dl class="dapp-review-grid">
      <div><dt>Type</dt><dd>${escapeHtml(dappTitleCase(review.request.type))}</dd></div>
      <div><dt>Object family</dt><dd>${escapeHtml(dappTitleCase(review.request.objectFamily))}</dd></div>
      <div><dt>Action</dt><dd>${escapeHtml(review.request.action)}</dd></div>
      <div><dt>Exact scope</dt><dd>${escapeHtml(review.request.exactScope)}</dd></div>
      <div><dt>Value</dt><dd>${escapeHtml(review.request.value)}</dd></div>
      <div><dt>Fee</dt><dd>${escapeHtml(review.request.fee)}</dd></div>
      <div><dt>Expires</dt><dd>${escapeHtml(dappDateTime(review.expiresAt))}</dd></div>
      <div><dt>Recipient</dt><dd>Withheld until Wallet review</dd></div>
    </dl>
    <p class="field-error messenger-review-error" role="alert">${escapeHtml(state.messengerReviewError || "")}</p>
    <div class="messenger-detail-actions"><button class="button" type="button" data-messenger-action="reject-request">${escapeHtml(t("plan2.actions.reject"))}</button><button class="button button-primary" type="button" data-messenger-action="accept-request"${review.expired ? " disabled" : ""}>${escapeHtml(t("plan2.actions.accept"))} for Wallet review</button></div>
  </section>`;
}

function messengerOutcomeScreen() {
  const outcome = state.messengerLastOutcome;
  if (!outcome) {
    state.messengerScreen = "list";
    return messengerFolderScreen(state.messengerSection);
  }
  const accepted = outcome.kind === "accepted";
  return `<section class="dapp-outcome${accepted ? "" : " is-negative"} messenger-outcome" data-messenger-outcome="${escapeHtml(outcome.kind)}">
    <span class="result-icon">${icon(accepted ? "check" : "close")}</span>
    <p class="eyebrow">Local advisory decision</p>
    <h2>${escapeHtml(outcome.title)}</h2>
    <p>${escapeHtml(outcome.summary)}</p>
    <div class="capability-note">${icon("shield")}<span><strong>Wallet state unchanged</strong><small>The message decision is presentation state. Only a separately revalidated Wallet review may prepare a wallet operation.</small></span></div>
    <div class="messenger-detail-actions">${accepted ? `<button class="button button-primary" type="button" data-messenger-action="wallet-review">Continue in Wallet review</button>` : ""}<button class="button" type="button" data-messenger-action="back">Back to requests</button></div>
  </section>`;
}

function messengerView() {
  const section = state.activeRoute.startsWith("messenger.") ? state.activeRoute.split(".").at(-1) : state.messengerSection;
  state.messengerSection = section;
  let content;
  if (state.messengerScreen === "detail") content = messengerMessageDetail();
  else if (state.messengerScreen === "review") content = messengerRequestReview();
  else if (state.messengerScreen === "outcome") content = messengerOutcomeScreen();
  else if (section === "sent") content = messengerSentScreen();
  else if (section === "conversations") content = messengerConversationScreen();
  else content = messengerFolderScreen("inbox");
  const helpTopicOverride = state.messengerScreen === "detail"
    ? "messenger.detail"
    : state.messengerScreen === "review" ? "messenger.request-review" : "";
  return `<section class="view-enter messenger-roadmap" data-messenger-screen="${escapeHtml(state.messengerScreen)}" data-messenger-section="${escapeHtml(section)}"${helpTopicOverride ? ` data-help-topic-override="${helpTopicOverride}"` : ""}>${messengerRuntimeControls()}${content}</section>`;
}

function contactStatusBadge(status) {
  const className = {
    known_locally: "is-ready",
    needs_confirmation: "is-warning",
    identity_changed: "is-warning",
    expired: "",
    revoked: "is-error"
  }[status] || "";
  return `<span class="status-badge ${className}">${escapeHtml(dappTitleCase(status))}</span>`;
}

function contactCard(contact) {
  return `<article class="contact-book-entry" data-contact="${escapeHtml(contact.id)}"><button class="contact-book-row" type="button" data-contact-action="open" data-contact-id="${escapeHtml(contact.id)}" aria-label="Open ${escapeHtml(contact.label)} contact">
      <span class="contact-avatar" aria-hidden="true">${escapeHtml(contact.initials)}</span>
      <span class="contact-book-copy"><strong>${escapeHtml(contact.label)}</strong><small>Last used ${escapeHtml(dappDateTime(contact.lastLocalUseAt))}</small></span>
      <span class="contact-book-state">${contactStatusBadge(contact.status)}${icon("chevron")}</span>
    </button></article>`;
}

function contactsListScreen() {
  const result = contactsGateway.listContacts({
    query: state.contactsQuery,
    status: state.contactsStatus,
    sort: state.contactsSort
  });
  const contacts = result.ok ? result.data.items : [];
  return `${dappScreenHeading("Address book", "Choose a nickname to open that contact's local details. No presence lookup or contact upload occurs.", `<button class="button button-primary" type="button" data-contact-action="add">${icon("plus")} Add contact</button>`)}
    <form class="contacts-toolbar" id="contacts-search-form" autocomplete="off">
      <div class="field-group"><label class="field-label" for="contacts-query">Search nicknames</label><div class="contacts-search-row"><input id="contacts-query" name="query" value="${escapeHtml(state.contactsQuery)}" maxlength="48" placeholder="Nickname"><button class="button" type="submit">${icon("search")} ${escapeHtml(t("plan2.actions.search"))}</button></div></div>
      <div class="field-group"><label class="field-label" for="contacts-status">Local status</label><select id="contacts-status" name="status" data-contact-status-filter><option value="all"${state.contactsStatus === "all" ? " selected" : ""}>All statuses</option>${demoRuntime.CONTACT_STATUS_IDS.map((status) => `<option value="${escapeHtml(status)}"${state.contactsStatus === status ? " selected" : ""}>${escapeHtml(dappTitleCase(status))}</option>`).join("")}</select></div>
      <div class="field-group"><label class="field-label" for="contacts-sort">Sort by</label><select id="contacts-sort" name="sort" data-contact-sort><option value="nickname"${state.contactsSort === "nickname" ? " selected" : ""}>Nickname</option><option value="date"${state.contactsSort === "date" ? " selected" : ""}>Date</option></select></div>
    </form>
    ${contacts.length
      ? `<section class="contact-list contact-book-list" aria-label="Address book">${contacts.map(contactCard).join("")}</section>`
      : `<section class="empty-state"><h3>No local contacts match</h3><p>Clear the local search or choose another status. No network search was attempted.</p></section>`}`;
}

function contactDetailScreen() {
  const result = contactsGateway.readContact({ contactId: state.contactsSelectedId });
  if (!result.ok) {
    state.contactsScreen = "list";
    return contactsListScreen();
  }
  const contact = result.data.contact;
  const requiresReview = contact.status === "identity_changed";
  const unusable = ["expired", "revoked"].includes(contact.status);
  return `<section class="contact-detail" data-contact-detail="${escapeHtml(contact.id)}">
    <button class="button button-quiet" type="button" data-contact-action="back">← Contacts</button>
    <header class="contact-detail-heading"><span class="contact-avatar is-large">${escapeHtml(contact.initials)}</span><div><p class="eyebrow">Wallet-local record</p><h2>${escapeHtml(contact.label)}</h2><p>${escapeHtml(contact.safeNote)}</p></div>${contactStatusBadge(contact.status)}</header>
    <dl class="contact-detail-grid">
      <div><dt>Abbreviated fingerprint</dt><dd>${escapeHtml(contact.fingerprint)}</dd></div>
      <div><dt>Source</dt><dd>${escapeHtml(contact.source)}</dd></div>
      <div><dt>Last local use</dt><dd>${escapeHtml(dappDateTime(contact.lastLocalUseAt))}</dd></div>
      <div><dt>Compatibility</dt><dd>${escapeHtml(contact.compatibility)}</dd></div>
      <div><dt>Expiry</dt><dd>${escapeHtml(dappDateTime(contact.expiresAt))}</dd></div>
      <div><dt>Tags</dt><dd>${escapeHtml(contact.tags.join(", "))}</dd></div>
      <div><dt>Identity domains</dt><dd>Contact · Reticulum · Wallet recipient remain separate</dd></div>
      <div><dt>Verification</dt><dd>Known locally only; no public trust claim</dd></div>
    </dl>
    ${requiresReview ? `<div class="notice">${icon("alert")} Receiver identity changed. Review it before Pay, Request, or Message.</div>` : ""}
    ${unusable ? `<div class="notice">${icon("alert")} This receiver material is ${escapeHtml(contact.status)}. Export remains available for local review; value and messaging actions fail closed.</div>` : ""}
    <div class="contact-detail-actions">
      <button class="button button-primary" type="button" data-contact-action="pay" data-contact-id="${escapeHtml(contact.id)}"${requiresReview || unusable ? " disabled" : ""}>Pay</button>
      <button class="button" type="button" data-contact-action="request" data-contact-id="${escapeHtml(contact.id)}"${requiresReview || unusable ? " disabled" : ""}>Request</button>
      <button class="button" type="button" data-contact-action="message" data-contact-id="${escapeHtml(contact.id)}"${requiresReview || unusable ? " disabled" : ""}>Message</button>
      <button class="button" type="button" data-contact-action="edit" data-contact-id="${escapeHtml(contact.id)}">Edit label</button>
      <button class="button" type="button" data-contact-action="export" data-contact-id="${escapeHtml(contact.id)}">Export public material</button>
      ${requiresReview ? `<button class="button" type="button" data-contact-action="identity-review" data-contact-id="${escapeHtml(contact.id)}">Review identity change</button>` : ""}
      <button class="button button-danger" type="button" data-contact-action="remove" data-contact-id="${escapeHtml(contact.id)}">Remove locally</button>
    </div>
  </section>`;
}

function contactImportScreen() {
  const source = demoRuntime.CONTACT_IMPORT_PREVIEWS.find(({ id }) => id === state.contactsImportSourceId) || demoRuntime.CONTACT_IMPORT_PREVIEWS[0];
  const nativeRequired = ["qr_scan", "native_share"].includes(source.id);
  return `<section class="contact-import" data-contact-import="${escapeHtml(source.id)}">
    <button class="button button-quiet" type="button" data-contact-action="back">← Contacts</button>
    ${dappScreenHeading("Add contact", "Choose one reviewed local source. Browser camera, arbitrary URL, remote file, and secret import are unavailable.")}
    <div class="contact-import-options" role="group" aria-label="Contact source">${demoRuntime.CONTACT_IMPORT_PREVIEWS.map((entry) => `<button class="contact-import-option${entry.id === source.id ? " is-selected" : ""}" type="button" data-contact-action="import-source" data-source-id="${escapeHtml(entry.id)}" aria-pressed="${entry.id === source.id}"><span>${icon(entry.iconName)}</span><strong>${escapeHtml(entry.label)}</strong><small>${escapeHtml(entry.summary)}</small></button>`).join("")}</div>
    ${nativeRequired
      ? `<section class="watcher-state-panel contact-native-boundary" role="status">${icon("alert")}<div><strong>Native boundary unavailable in browser demo</strong><p>${escapeHtml(source.summary)}</p><small>A Tauri command must mediate permission, parsing, cancellation, and sanitized errors.</small></div></section>`
      : `<form class="contact-import-form" id="contact-import-form" autocomplete="off" novalidate><div class="field-group"><label class="field-label" for="contact-import-label">Local label</label><input id="contact-import-label" name="label" minlength="2" maxlength="40" required></div><div class="field-group"><label class="field-label" for="contact-import-note">Safe local note <span class="muted">(optional)</span></label><input id="contact-import-note" name="safeNote" maxlength="80"></div><p class="field-error" id="contact-import-error" role="alert">${escapeHtml(state.contactsFormError || "")}</p><div class="contact-detail-actions"><button class="button button-primary" type="submit">Review and save locally</button></div></form>`}
  </section>`;
}

function contactEditScreen() {
  const result = contactsGateway.readContact({ contactId: state.contactsSelectedId });
  if (!result.ok) {
    state.contactsScreen = "list";
    return contactsListScreen();
  }
  const contact = result.data.contact;
  return `<section class="contact-edit" data-contact-edit="${escapeHtml(contact.id)}"><button class="button button-quiet" type="button" data-contact-action="detail">← Contact</button>${dappScreenHeading("Edit local label", "This changes only local presentation metadata, never receiver material or counterparty state.")}
    <form class="contact-import-form" id="contact-edit-form" autocomplete="off" novalidate><div class="field-group"><label class="field-label" for="contact-edit-label">Local label</label><input id="contact-edit-label" name="label" value="${escapeHtml(contact.label)}" minlength="2" maxlength="40" required></div><div class="field-group"><label class="field-label" for="contact-edit-note">Safe local note</label><input id="contact-edit-note" name="safeNote" value="${escapeHtml(contact.safeNote)}" maxlength="80"></div><p class="field-error" id="contact-edit-error" role="alert">${escapeHtml(state.contactsFormError || "")}</p><div class="contact-detail-actions"><button class="button button-primary" type="submit">Save local changes</button></div></form></section>`;
}

function contactIdentityReviewScreen() {
  const result = contactsGateway.readContact({ contactId: state.contactsSelectedId });
  if (!result.ok) {
    state.contactsScreen = "list";
    return contactsListScreen();
  }
  const contact = result.data.contact;
  return `<section class="messenger-review contact-identity-review" data-contact-identity-review="${escapeHtml(contact.id)}"><button class="button button-quiet" type="button" data-contact-action="detail">← ${escapeHtml(t("navigation.contacts"))}</button><header class="contact-detail-heading"><span class="contact-avatar is-large">${escapeHtml(contact.initials)}</span><div><p class="eyebrow">Identity change review</p><h2>${escapeHtml(contact.label)}</h2><p>Compare reviewed receiver material outside this demo before accepting.</p></div>${contactStatusBadge(contact.status)}</header><dl class="contact-detail-grid"><div><dt>Displayed change</dt><dd>${escapeHtml(contact.fingerprint)}</dd></div><div><dt>Source</dt><dd>${escapeHtml(contact.source)}</dd></div><div><dt>Compatibility</dt><dd>${escapeHtml(contact.compatibility)}</dd></div><div><dt>Trust effect</dt><dd>None; local confirmation only</dd></div></dl><div class="confirmation-note">${icon("shield")} Accepting updates local compatibility only. It does not create public trust, upload the contact, or mutate Wallet value.</div><div class="contact-detail-actions"><button class="button" type="button" data-contact-action="identity-reject">${escapeHtml(t("plan2.actions.reject"))}</button><button class="button button-primary" type="button" data-contact-action="identity-accept">${escapeHtml(t("plan2.actions.accept"))}</button></div></section>`;
}

function contactOutcomeScreen() {
  const outcome = state.contactsLastOutcome;
  if (!outcome) {
    state.contactsScreen = "list";
    return contactsListScreen();
  }
  return `<section class="dapp-outcome contact-outcome" data-contact-outcome="${escapeHtml(outcome.kind)}"><span class="result-icon">${icon(outcome.kind === "removed" ? "remove" : "check")}</span><p class="eyebrow">Local contact outcome</p><h2>${escapeHtml(outcome.title)}</h2><p>${escapeHtml(outcome.summary)}</p><div class="capability-note">${icon("shield")}<span><strong>No remote side effect</strong><small>No contact upload, public presence, implicit trust, Wallet mutation, settlement mutation, or protocol revocation occurred.</small></span></div><button class="button button-primary" type="button" data-contact-action="back">Back to contacts</button></section>`;
}

function contactsView() {
  let content;
  if (state.contactsScreen === "detail") content = contactDetailScreen();
  else if (state.contactsScreen === "import") content = contactImportScreen();
  else if (state.contactsScreen === "edit") content = contactEditScreen();
  else if (state.contactsScreen === "identity-review") content = contactIdentityReviewScreen();
  else if (state.contactsScreen === "outcome") content = contactOutcomeScreen();
  else content = contactsListScreen();
  const helpTopicOverride = state.contactsScreen === "detail"
    ? "contacts.detail"
    : state.contactsScreen === "identity-review" ? "contacts.identity-review" : "";
  return `<section class="view-enter contacts-roadmap" data-contact-screen="${escapeHtml(state.contactsScreen)}"${helpTopicOverride ? ` data-help-topic-override="${helpTopicOverride}"` : ""}>${content}</section>`;
}

function completeDappPermissionReview(decision, acknowledgements = {}) {
  const reviewResult = dappGateway.readPermissionReview({
    connectionId: state.dappReviewConnectionId
  });
  if (!reviewResult.ok) {
    state.dappReviewValidationError = reviewResult.error.message;
    render();
    requestAnimationFrame(() => document.querySelector("#dapp-review-error")?.focus?.());
    return false;
  }

  const result = dappGateway.decidePermissionReview({
    reviewId: reviewResult.data.reviewId,
    decision,
    scopeConfirmed: Boolean(acknowledgements.scopeConfirmed),
    reauthAcknowledged: Boolean(acknowledgements.reauthAcknowledged)
  });
  if (!result.ok) {
    state.dappReviewValidationError = result.error.message;
    state.dappReviewAcknowledgements = {
      scopeConfirmed: Boolean(acknowledgements.scopeConfirmed),
      reauthAcknowledged: Boolean(acknowledgements.reauthAcknowledged)
    };
    render();
    requestAnimationFrame(() => document.querySelector("#dapp-review-error")?.focus?.());
    return false;
  }

  state.dappReviewValidationError = null;
  state.dappReviewDecision = result.data;
  state.dappLastOutcome = {
    kind: decision === "accepted" ? "intent_accepted" : "intent_rejected",
    label: decision === "accepted" ? "Bounded intent accepted" : "Connection request rejected",
    summary: decision === "accepted"
      ? "The typed request passed app-level review. No Wallet operation was created and no wallet object changed."
      : "The local request was rejected before any Wallet review, signing, object mutation, or settlement path.",
    returnRoute: "dapps.discover",
    descriptorId: result.data.descriptorId
  };
  state.dappScreen = "outcome";
  render({ focusMain: true });
  showToast(decision === "accepted" ? "Bounded local intent accepted." : "Local connection request rejected.");
  return true;
}

function routePreviewView() {
  const routeId = state.previewRoute;
  const node = demoRuntime.navigationNodeForRoute(routeId);
  const capability = node?.capabilityId ? demoRuntime.capabilityProfile(node.capabilityId) : null;
  const label = node ? navigationLabel(node) : routeId;
  const capabilityLabel = capability?.presentationMode === "roadmap_preview" ? t("navigation.roadmap") : "Concept";
  const panel = `<section class="route-preview" aria-labelledby="route-preview-title">
    <p class="eyebrow">${escapeHtml(capabilityLabel)}</p>
    <h2 id="route-preview-title">${escapeHtml(label)}</h2>
    <p>This route is deliberately visible in the navigation model, but it does not claim a protocol implementation or live network capability.</p>
    <dl class="route-preview-metadata">
      <div><dt>Presentation</dt><dd>${escapeHtml(capability?.presentationMode || "product")}</dd></div>
      <div><dt>Maturity</dt><dd>${escapeHtml(capability?.maturity || "target")}</dd></div>
      <div><dt>Availability</dt><dd>${escapeHtml(capability?.availability || "unavailable")}</dd></div>
      <div><dt>Evidence</dt><dd>${escapeHtml(capability?.evidenceSource || "none")}</dd></div>
    </dl>
  </section>`;
  const workspaceId = node?.target.kind === "workspace"
    ? node.id
    : demoRuntime.ancestorContainerIdsForNode(node?.id || "")
      .find((containerId) => demoRuntime.navigationNode(containerId)?.target.kind === "workspace");
  return workspaceId
    ? workspaceFrame(workspaceId, panel)
    : panel;
}

function renderActiveWorkspace(renderer) {
  try {
    const injectedFailure = new URLSearchParams(window.location.search).get("workspaceFailure");
    if (injectedFailure === state.activeRoute) {
      throw new Error("Deterministic workspace failure");
    }
    return renderer();
  } catch {
    return `<section class="view-enter workspace-error-boundary" role="alert" data-workspace-error="${escapeHtml(state.activeRoute)}">
      <p class="eyebrow">Workspace unavailable</p>
      <h2>This section could not be rendered</h2>
      <p>The failure was isolated from the Z00Z shell. Navigation, Lock, Help, and every other workspace remain available.</p>
      <button class="button button-primary" type="button" data-demo-action="retry-workspace">Retry this section</button>
    </section>`;
  }
}

function render(options = {}) {
  const mobileNavigationScrollTop = !mobilePopupMenu.hidden && mobilePopupType === "menu"
    ? mobilePopupMenu.querySelector(".mobile-navigation-scroll-region")?.scrollTop
    : null;
  closeSelectPickers();
  synchronizeShellRoute();
  applyAppearancePreferences();
  renderWalletShell();
  const mobileMenuLabel = t("app.menu");
  mobileMenuButton.setAttribute("aria-label", mobileMenuLabel);
  mobileMenuButton.setAttribute("title", mobileMenuLabel);
  renderMenuSearchChrome();
  const walletScreen = hasSelectedWalletContext();
  const wallet = activeWallet();
  renderMobileActiveWallet(wallet);
  const routeNode = demoRuntime.navigationNodeForRoute(state.activeRoute);
  const [legacyTitle = ""] = headings[state.view] || [];
  const ancestorLabels = routeNode
    ? demoRuntime.ancestorContainerIdsForNode(routeNode.id).map((containerId) => navigationLabel(demoRuntime.navigationNode(containerId)))
    : [];
  routeBreadcrumb.textContent = routeNode ? [...ancestorLabels, navigationLabel(routeNode)].join(" / ") : "";
  routeBreadcrumb.hidden = !routeNode;
  const telemetryOverview = /^telemetry\.(reticulum|onionnet|aggregators)\.overview$/.exec(state.activeRoute);
  const [topbarTitle] = telemetryOverview
    ? telemetryTopbar[telemetryOverview[1]]
    : [routeNode ? navigationLabel(routeNode) : t(legacyTitle), null];
  pageTitle.textContent = topbarTitle;
  pageContext.textContent = "";
  pageContext.hidden = true;
  pageTitle.classList.remove("is-wallet-address", "is-telemetry-title", "is-settings-title");
  topbarAddressGroup.classList.remove("has-wallet-address");
  walletIdentity.hidden = !Boolean(state.selectedWalletId && wallet);
  const activeRenderer = {
    wallet: walletView,
    "wallet-send": walletSendView,
    "wallet-receive": walletReceiveView,
    "wallet-import": walletImportView,
    "wallet-merge-split": walletMergeSplitView,
    activity: activityView,
    swap: swapView,
    staking: stakingView,
    "wallet-backup": walletBackupView,
    "wallet-settings": walletSettingsView,
    settings: settingsView,
    telemetry: telemetryView,
    dapps: dappsView,
    messenger: messengerView,
    contacts: contactsView,
    "data-storage": dataStorageView,
    about: aboutView,
    "route-preview": routePreviewView
  }[state.view];
  main.dataset.mountedRoute = state.activeRoute;
  main.innerHTML = renderActiveWorkspace(activeRenderer);
  renderMobileTopbarContext();
  help.configure({ language: state.language, palette: state.palette });
  help.mountContextButton(state, main.firstElementChild);
  suppressPasswordManagerUI(document);
  enhanceNativeSelects(main);

  syncBalanceButtons();
  if (!mobilePopupMenu.hidden && mobilePopupType === "menu") {
    mobilePopupMenu.innerHTML = mobileNavigationDrawerMarkup();
    enhanceNativeSelects(mobilePopupMenu);
    if (Number.isFinite(mobileNavigationScrollTop)) {
      mobilePopupMenu.querySelector(".mobile-navigation-scroll-region").scrollTop = mobileNavigationScrollTop;
      captureNavigationScrollPosition("mobile", mobileNavigationScrollTop);
    }
  }
  if (!walletPickerPopup.hidden) {
    walletPickerPopup.innerHTML = walletPickerPopupMarkup();
  }
  requestAnimationFrame(() => {
    restoreNavigationScrollPosition(sidebarNavigationScrollRegion, "desktop");
    const activeContext = isMobileNavigation()
      ? mobileTopbarContext.querySelector(".context-nav-child.is-active, .context-nav-item.is-active")
      : main.querySelector(".context-nav-child.is-active, .context-nav-item.is-active");
    activeContext?.scrollIntoView({ block: "nearest", inline: "center" });
  });
  if (options.focusMain) {
    main.focus({ preventScroll: true });
    window.scrollTo({ top: 0, behavior: state.reducedMotion || window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
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
  const chainId = readYamlScalar(source, "chain");

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
  if (chainId !== activeWallet().chainId) return { valid: false, message: `chain is read-only and must remain ${activeWallet().chainId}.` };

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
  if (type === "wallet-password-change") return walletPasswordChangeDialog();
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
      body: `<div class="field-group"><label class="field-label" for="wallet-rename-name">Wallet name</label><input id="wallet-rename-name" name="walletLabel" maxlength="32" value="${escapeHtml(wallet.name)}" autocomplete="section-z00z-wallet nickname" ${passwordManagerIgnoreAttributes} required><p class="field-hint">This local label does not change the wallet address or key material.</p></div>`,
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
    body: `<form class="form-grid" id="${type}-entry" autocomplete="off" ${passwordManagerIgnoreAttributes} novalidate>${definition.body}<div class="field-group"><label class="field-label" for="${passwordId}">Wallet password</label><input id="${passwordId}" name="walletSecret" ${secureEntryAttributes("wallet-action")} minlength="8" required><p class="field-hint">This concept validates locally and clears the value immediately after use.</p><p class="field-error" id="${type}-error" role="alert"></p></div>${confirmationMarkup}</form>`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="${type}-entry">${definition.actionLabel}</button>`
  });
}

function walletPasswordChangeDialog() {
  if (state.flow.step === 1) {
    return dialogFrame({
      title: t("walletSettings.passwordChangedTitle"),
      subtitle: "Local concept result",
      body: `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>${t("walletSettings.passwordChangedTitle")}</h3><p>${t("walletSettings.passwordChangedResult")}</p></div>`,
      footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
    });
  }

  return dialogFrame({
    title: t("walletSettings.changePasswordTitle"),
    subtitle: t("walletSettings.changePasswordSubtitle"),
    body: `<form class="form-grid" id="wallet-password-change-entry" autocomplete="off" ${passwordManagerIgnoreAttributes} novalidate>
      <div class="field-group"><label class="field-label" for="wallet-current-password">${t("walletSettings.currentPassword")}</label><input id="wallet-current-password" name="currentWalletSecret" ${secureEntryAttributes("current-wallet-secret")} minlength="8" required></div>
      <div class="field-group"><label class="field-label" for="wallet-new-password">${t("walletSettings.newPassword")}</label><input id="wallet-new-password" name="newWalletSecret" ${secureEntryAttributes("new-wallet-secret")} minlength="8" required><p class="field-hint">${t("walletSettings.passwordChangeHint")}</p></div>
      <div class="field-group"><label class="field-label" for="wallet-confirm-new-password">${t("walletSettings.confirmNewPassword")}</label><input id="wallet-confirm-new-password" name="confirmNewWalletSecret" ${secureEntryAttributes("new-wallet-secret-confirmation")} minlength="8" required></div>
      <p class="field-error" id="wallet-password-change-error" role="alert"></p>
    </form>`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="wallet-password-change-entry">${t("walletSettings.changePasswordSubmit")}</button>`
  });
}

function dialogFrame({ title, subtitle, body, footer = "", steps = 0, activeStep = 0, closeLabel, headerLeading = "", helpTopic = "" }) {
  const resolvedCloseLabel = closeLabel || t("common.close");
  const indicators = steps > 1
    ? `<div class="step-indicator" aria-label="Step ${activeStep + 1} of ${steps}">${Array.from({ length: steps }, (_, index) => `<span class="${index < activeStep ? "is-done" : index === activeStep ? "is-active" : ""}"></span>`).join("")}</div>`
    : "";
  return `
    <div class="dialog-shell">
      <header class="dialog-header${headerLeading ? " has-leading-visual" : ""}">
        ${headerLeading}
        <div class="dialog-header-copy"><h2 id="dialog-title">${title}</h2><p>${subtitle}</p></div>
        ${indicators}
        ${helpTopic ? `<button class="icon-button dialog-help-button" type="button" data-help-topic="${escapeHtml(helpTopic)}" aria-label="${escapeHtml(t("help.openContext"))}" title="${escapeHtml(t("help.title"))}">${icon("question")}</button>` : ""}
        <button class="icon-button" type="button" data-dialog-close aria-label="${escapeHtml(resolvedCloseLabel)}">${icon("close")}</button>
      </header>
      <div class="dialog-body">${body}</div>
      ${footer ? `<footer class="dialog-footer">${footer}</footer>` : ""}
    </div>`;
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
      body: `<div class="review-card review-hero"><span class="list-icon is-voucher">${objectFamilyIcon("voucher")}</span><strong>86.00 Z00Z</strong><span>Travel refund voucher</span></div><div class="review-card"><div class="summary-row"><span>Issuer</span><strong>Northwind Travel</strong></div><div class="summary-row"><span>Backing</span><strong>Consumed asset reference</strong></div><div class="summary-row"><span>Face / remaining</span><strong>86.00 / 86.00 Z00Z</strong></div><div class="summary-row"><span>Acceptance</span><strong>Required</strong></div><div class="summary-row"><span>Ends</span><strong>21 Jul 2026 · 18:00</strong></div><div class="summary-row"><span>Holder options</span><strong>Accept · Reject</strong></div></div><div class="confirmation-note">${icon("shield")} Accepting changes the voucher lifecycle. It does not directly add 86.00 Z00Z to Available.</div>`,
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

function createVoucherDialog() {
  return dialogFrame({
    title: "Create voucher",
    subtitle: "Create conditional value in this wallet",
    body: `
      <form class="form-grid" id="create-voucher-entry" novalidate>
        <div class="field-group"><label class="field-label" for="voucher-create-name">Voucher name</label><input id="voucher-create-name" name="title" maxlength="48" value="Gift voucher" autocomplete="off" required aria-describedby="voucher-create-error"><p class="field-error" id="voucher-create-error" role="alert"></p></div>
        <div class="field-group"><label class="field-label" for="voucher-create-amount">Value</label><div class="input-with-affix"><input id="voucher-create-amount" name="amount" type="number" min="0.01" step="0.01" value="10.00" inputmode="decimal" required><span class="input-affix">Z00Z</span></div></div>
        <div class="field-group"><label class="field-label" for="voucher-create-expiry">Expires</label><input id="voucher-create-expiry" name="expiry" type="date" value="2026-12-31" min="2026-07-22" required></div>
        <div class="confirmation-note">${icon("shield")} The voucher remains a distinct wallet object. Its value is not added to Available.</div>
      </form>`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="create-voucher-entry">Create voucher</button>`
  });
}

function voucherDetailDialog() {
  const voucher = walletObjectEntry("voucher", state.flow.data.objectId);
  if (!voucher) return dialogFrame({ title: "Voucher unavailable", subtitle: "The wallet state changed", body: `<div class="empty-state"><p>This voucher is no longer available.</p></div>`, footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>` });
  return dialogFrame({
    title: voucher.title,
    subtitle: "Wallet voucher",
    body: `<div class="review-card review-hero"><span class="list-icon is-voucher">${objectFamilyIcon("voucher")}</span><strong>${escapeHtml(voucher.value)}</strong><span>${escapeHtml(voucher.status)}</span></div><div class="review-card"><div class="summary-row"><span>Expires</span><strong>${escapeHtml(voucher.expiry)}</strong></div><div class="summary-row"><span>Transfer</span><strong>${voucher.transferable ? "Ready" : escapeHtml(voucher.status)}</strong></div><div class="summary-row"><span>Wallet</span><strong>${escapeHtml(activeWallet().name)}</strong></div></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function createPermissionDialog() {
  return dialogFrame({
    title: "Create permission",
    subtitle: "Define bounded authority held by this wallet",
    body: `
      <form class="form-grid" id="create-permission-entry" novalidate>
        <div class="field-group"><label class="field-label" for="permission-create-name">Permission name</label><input id="permission-create-name" name="title" maxlength="48" value="Service access" autocomplete="off" required aria-describedby="permission-create-error"><p class="field-error" id="permission-create-error" role="alert"></p></div>
        <div class="field-group"><label class="field-label" for="permission-create-action">Allowed action</label><select id="permission-create-action" name="action"><option>View status</option><option>Deploy release</option><option>Read receipt</option></select></div>
        <div class="field-group"><label class="field-label" for="permission-create-scope">Scope</label><input id="permission-create-scope" name="scope" value="service.example" autocomplete="off" required></div>
        <div class="field-group"><label class="field-label" for="permission-create-uses">Maximum uses</label><input id="permission-create-uses" name="uses" type="number" min="1" max="100" value="1" inputmode="numeric" required></div>
        <div class="field-group"><label class="field-label" for="permission-create-expiry">Expires</label><input id="permission-create-expiry" name="expiry" type="date" value="2026-12-31" min="2026-07-22" required></div>
        <div class="confirmation-note">${icon("shield")} A permission carries bounded authority and no monetary value.</div>
      </form>`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="create-permission-entry">Create permission</button>`
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
        <div class="review-card review-hero"><span class="list-icon is-right">${objectFamilyIcon("right")}</span><strong>${escapeHtml(data.uses)} uses</strong><span>for ${escapeHtml(data.delegate)}</span></div>
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
  const permission = (activeWallet().permissions || []).find((entry) => entry.id === state.flow.data.permissionId) || permissionDetails[state.flow.data.permissionId] || permissionDetails.receipt;
  return dialogFrame({
    title: permission.title,
    subtitle: permission.subtitle || "Held bounded permission",
    body: `
      <div class="review-card review-hero"><span class="list-icon is-right">${objectFamilyIcon("right")}</span><strong>${permission.remaining}</strong><span>remaining</span></div>
      <div class="review-card"><div class="summary-row"><span>Class</span><strong>${permission.classLabel}</strong></div><div class="summary-row"><span>Allowed action</span><strong>${permission.action}</strong></div><div class="summary-row"><span>Scope</span><strong>${permission.scope}</strong></div><div class="summary-row"><span>Delegation</span><strong>${permission.delegation}</strong></div><div class="summary-row"><span>Ends</span><strong>${permission.expiry}</strong></div><div class="summary-row"><span>Monetary value</span><strong>None</strong></div><div class="summary-row"><span>Status</span><strong><span class="status-badge is-${escapeHtml(permission.tone || "active")}">${escapeHtml(permission.status || "Held")}</span></strong></div></div>
      <details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>Right: ${permission.rightId}</span><span>Class: ${permission.typeLabel || permission.kind}</span><span>Lifecycle: granted → held</span></div></details>`,
    footer: `<button class="button button-danger" type="button" data-dialog-action="permission-revoke">Revoke permission</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function activityDialog(item) {
  const lifecycle = t(item.status === "settling" ? "history.lifecyclePending" : "history.lifecycleConfirmed");
  return dialogFrame({
    title: activityText(item, "title"),
    subtitle: t("history.details"),
    body: `
      <div class="review-card review-hero"><span class="list-icon ${item.direction === "in" ? "is-claim" : ""}">${icon(item.direction === "in" ? "receive" : item.direction === "out" ? "send" : "activity")}</span><strong>${escapeHtml(activityText(item, "amount") || statusText(item.status))}</strong><span>${escapeHtml(activityText(item, "detail"))}</span></div>
      <div class="review-card"><div class="summary-row"><span>${t("history.status")}</span><strong><span class="status-badge is-${escapeHtml(item.status)}">${statusText(item.status)}</span></strong></div><div class="summary-row"><span>${t("history.when")}</span><strong>${escapeHtml(activityText(item, "time"))}</strong></div><div class="summary-row"><span>${t("history.fee")}</span><strong>${t(item.type === "money" ? "history.feeIncluded" : "history.feeNotApplicable")}</strong></div><div class="summary-row"><span>${t("history.privacy")}</span><strong>${t("history.privacyValue")}</strong></div><div class="summary-row"><span>${t("history.carrierChain")}</span><strong>${t("history.carrierChainValue")}</strong></div></div>
      <details class="technical"><summary>${t("history.technicalDetails")}</summary><div class="technical-content mono"><span>${t("history.idLabel")}: ${escapeHtml(item.id)}-b4c9…8e20</span><span>${t("history.lifecycleLabel")}: ${lifecycle}</span><span>${t("history.receiptLabel")}: public_4a92…c71e</span></div></details>`,
    footer: `<button class="button" type="button" data-demo-action="copy-receipt">${icon("copy")} ${t("history.copyReceipt")}</button><button class="button button-primary" type="button" data-dialog-close>${t("history.done")}</button>`
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
    headerLeading: assetIcon(asset, "asset-detail-logo"),
    helpTopic: "asset.details",
    body: `<div class="asset-detail-table">${rows.map(([label, value]) => `<div class="asset-detail-row"><span>${escapeHtml(label)}</span><strong class="${["Owner", "Asset ID"].includes(label) ? "mono" : ""}" title="${escapeHtml(value)}">${escapeHtml(value)}</strong></div>`).join("")}</div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>OK</button>`
  });
}

function connectionDialog() {
  const chain = activeWallet().chainId;
  return dialogFrame({
    title: "Network",
    subtitle: "Overlay, carrier, and chain are separate",
    body: `
      <p class="eyebrow">Privacy mode · target simulation</p>
      <div class="connection-options"><div class="connection-option"><span class="health-orb"></span><span><strong>OnionNet</strong><small>Target overlay example · 3 hops</small></span><span class="status-badge is-ready">Target</span></div><div class="connection-option"><span class="health-orb"></span><span><strong>Reticulum</strong><small>Target primary resilient carrier</small></span><span class="status-badge is-ready">Target</span></div><div class="connection-option"><span class="health-orb"></span><span><strong>Tor</strong><small>Current switch method is a placeholder</small></span><span class="status-badge">Stub</span></div></div>
      <p class="eyebrow" style="margin-top:22px">Chain</p>
      <div class="connection-options"><div class="connection-option">${walletChainBadgeMarkup(chain)}<span><strong>${escapeHtml(walletChain(chain).label)}</strong><small>Bound when this wallet profile was created</small></span><span class="status-badge">Read-only</span></div></div>
      <div class="capability-note">${icon("alert")} <span><strong>Phase 080 target</strong><small>Current network RPC is stubbed; production must not show these properties until authoritative.</small></span></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function notificationsDialog() {
  return dialogFrame({
    title: "Notifications",
    subtitle: "One item needs attention",
    body: `<div class="attention-list"><button class="attention-item" type="button" data-dialog-action="notification-voucher"><span class="list-icon is-voucher">${objectFamilyIcon("voucher")}</span><span class="list-copy"><strong>Travel refund voucher expires soon</strong><small>Review 86.00 Z00Z from Northwind Travel</small></span>${icon("chevron")}</button><div class="attention-item"><span class="list-icon">${icon("backup")}</span><span class="list-copy"><strong>Backup verified</strong><small>Your 10 Jul local backup passed integrity checks</small></span><span class="status-badge is-settled">Done</span></div></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

const demoSeedWords = [
  "canvas", "orbit", "maple", "velvet", "harbor", "copper", "quiet", "meadow",
  "lamp", "river", "winter", "piano", "forest", "amber", "window", "salt",
  "comet", "paper", "garden", "silver", "cloud", "stone", "echo", "north"
];

function randomIndex(upperBound) {
  if (globalThis.crypto?.getRandomValues) {
    const range = 0x100000000;
    const limit = Math.floor(range / upperBound) * upperBound;
    const value = new Uint32Array(1);
    do globalThis.crypto.getRandomValues(value); while (value[0] >= limit);
    return value[0] % upperBound;
  }
  return Math.floor(Math.random() * upperBound);
}

function randomSeedVerificationIndexes(previousIndexes = []) {
  const previous = new Set(previousIndexes);
  const candidates = demoSeedWords.map((_, index) => index).filter((index) => !previous.has(index));
  for (let index = candidates.length - 1; index > 0; index -= 1) {
    const swapIndex = randomIndex(index + 1);
    [candidates[index], candidates[swapIndex]] = [candidates[swapIndex], candidates[index]];
  }
  return candidates.slice(0, 4).sort((left, right) => left - right);
}

function seedVerificationOptions(seedIndex) {
  const choices = [
    seedIndex,
    (seedIndex + 7) % demoSeedWords.length,
    (seedIndex + 13) % demoSeedWords.length
  ];
  const rotation = seedIndex % choices.length;
  return [...choices.slice(rotation), ...choices.slice(0, rotation)]
    .map((index) => `<option value="${escapeHtml(demoSeedWords[index])}">${escapeHtml(demoSeedWords[index])}</option>`)
    .join("");
}

function seedVerificationPositionList(indexes) {
  const positions = indexes.map((index) => index + 1);
  return `${positions.slice(0, -1).join(", ")}, and ${positions.at(-1)}`;
}

function walletsDialog() {
  const selected = activeWallet();
  return dialogFrame({
    title: "Your wallets",
    subtitle: "Local profiles on this device",
    body: `
      <div class="wallet-list">
        ${state.wallets.map((wallet) => `<button class="wallet-choice${wallet.id === selected.id ? " is-current" : ""}" type="button" data-dialog-action="select-wallet" data-wallet-id="${escapeHtml(wallet.id)}">
          <span class="wallet-avatar" aria-hidden="true">${escapeHtml(wallet.initials)}</span><span><strong>${escapeHtml(wallet.name)}</strong><small class="mono">${escapeHtml(wallet.address)} · ${escapeHtml(walletChain(wallet.chainId).label)}</small></span><span class="status-badge${wallet.id === selected.id ? " is-active" : ""}">${wallet.id === selected.id ? "Open" : "Select"}</span>
        </button>`).join("")}
      </div>
      <div class="notice">${icon("shield")} Wallet profiles are local. Switching never sends a seed or password to another service.</div>`,
    footer: `<button class="button" type="button" data-dialog-action="add-wallet">${icon("plus")} Add wallet</button><button class="button button-primary" type="button" data-dialog-close>Close</button>`
  });
}

function networksDialog() {
  const selectedNetwork = state.view === "telemetry" ? state.telemetrySource : "";
  return dialogFrame({
    title: t("app.network"),
    subtitle: t("settings.networkPrivacyHelp"),
    body: `
      <div class="wallet-list network-picker-list">
        ${networkEntries.map((entry) => `<button class="wallet-choice${entry.key === selectedNetwork ? " is-current" : ""}" type="button"${entry.key === selectedNetwork ? ' aria-current="page"' : ""} data-dialog-action="select-network" data-network-section="${entry.key}">
          <span class="network-avatar" aria-hidden="true">${entry.initials}</span>
          <span><strong>${entry.label}</strong><small>${t(entry.helperKey)}</small></span>
          ${entry.key === selectedNetwork ? `<span class="status-badge is-active">${t("walletShell.current")}</span>` : ""}
        </button>`).join("")}
      </div>`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>${t("common.close")}</button>`
  });
}

function removeWalletDialog() {
  const selectedIds = new Set(state.flow.data.walletIds || []);
  const selectedCount = selectedIds.size;
  const canRemove = selectedCount > 0;
  return dialogFrame({
    title: "Remove Wallet(s)",
    subtitle: "Remove local wallets from this concept. Wallet files are not deleted.",
    body: `
      <fieldset class="wallet-remove-list" aria-describedby="wallet-remove-summary">
        <legend class="sr-only">Wallets to remove</legend>
        ${state.wallets.map((wallet) => {
          const checked = selectedIds.has(wallet.id);
          return `<label class="wallet-remove-choice${checked ? " is-selected" : ""}">
            <input type="checkbox" data-remove-wallet-id="${escapeHtml(wallet.id)}"${checked ? " checked" : ""}>
            <span class="wallet-avatar" aria-hidden="true">${escapeHtml(wallet.initials)}</span>
            <span class="wallet-remove-copy"><strong>${escapeHtml(wallet.name)}</strong><small class="mono">${escapeHtml(wallet.address)} · ${escapeHtml(wallet.summary.available)} Z00Z</small></span>
          </label>`;
        }).join("")}
      </fieldset>
      <p class="remove-selection-summary" id="wallet-remove-summary">${selectedCount} of ${state.wallets.length} selected. This removes local wallets only.</p>
      ${selectedCount === state.wallets.length ? `<p class="field-error">All local wallets will be removed. You can add a wallet again afterward.</p>` : ""}`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-danger" type="button" data-dialog-action="confirm-remove-wallet"${canRemove ? "" : " disabled"}>${icon("remove")} Remove Wallet(s)${selectedCount ? ` (${selectedCount})` : ""}</button>`
  });
}

function addWalletDialog() {
  return dialogFrame({
    title: "Add wallet",
    subtitle: "Create, open, or restore a local wallet",
    body: `
      <div class="add-wallet-dialog-options" aria-label="Add wallet options">
        <p class="add-wallet-copy">Wallet keys, passwords, and recovery words remain local to this device in this concept.</p>
        <button class="button add-wallet-choice is-primary" type="button" data-demo-action="create-wallet">${icon("plus")} Create new wallet</button>
        <button class="button add-wallet-choice is-primary" type="button" data-demo-action="open-existing-wallet">${icon("wallet")} Open existing wallet</button>
        <button class="button add-wallet-choice" type="button" data-demo-action="restore-wallet">${icon("backup")} Restore from backup</button>
      </div>`,
    footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button>`
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
        <form class="form-grid" id="create-wallet-entry" autocomplete="off" ${passwordManagerIgnoreAttributes} novalidate>
          <div class="field-group"><label class="field-label" for="create-name">Wallet name</label><input id="create-name" name="walletLabel" value="${escapeHtml(data.name)}" maxlength="32" placeholder="Everyday wallet" autocomplete="section-z00z-new-wallet nickname" ${passwordManagerIgnoreAttributes} required aria-describedby="create-name-error"><p class="field-error" id="create-name-error"></p></div>
          <div class="field-group"><label class="field-label" for="create-chain">${t("common.chain")}</label><select id="create-chain" name="chainId" aria-describedby="create-chain-hint create-chain-error">${walletChainOptionsMarkup(data.chainId)}</select><p class="field-hint" id="create-chain-hint">${t("common.chainLocked")}</p><p class="field-error" id="create-chain-error"></p></div>
          <div class="field-group"><label class="field-label" for="create-password">Wallet password</label><input id="create-password" name="newWalletSecret" ${secureEntryAttributes("new-wallet-secret")} minlength="8" required aria-describedby="create-password-hint create-password-error"><p class="field-hint" id="create-password-hint">Use at least 8 characters. This concept never stores the value.</p><p class="field-error" id="create-password-error"></p></div>
          <div class="field-group"><label class="field-label" for="create-confirm">Confirm password</label><input id="create-confirm" name="confirmWalletSecret" ${secureEntryAttributes("new-wallet-secret-confirmation")} minlength="8" required aria-describedby="create-confirm-error"><p class="field-error" id="create-confirm-error"></p></div>
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
    const verificationIndexes = state.flow.data.verificationIndexes || randomSeedVerificationIndexes();
    state.flow.data.verificationIndexes = verificationIndexes;
    return dialogFrame({
      title: "Check your backup",
      subtitle: "Confirm four random words before continuing",
      steps: 4,
      activeStep: 2,
      body: `
        <form class="form-grid" id="create-wallet-verify" novalidate>
          ${verificationIndexes.map((seedIndex) => `<div class="field-group"><label class="field-label" for="seed-word-${seedIndex + 1}">Word ${seedIndex + 1}</label><select id="seed-word-${seedIndex + 1}" name="word${seedIndex + 1}" data-seed-index="${seedIndex}" required><option value="">Choose word</option>${seedVerificationOptions(seedIndex)}</select></div>`).join("")}
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
    body: `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>${escapeHtml(data.name || "New wallet")} is ready</h3><p>The wallet is encrypted on this device. The demonstration phrase has been cleared from the view.</p></div><div class="review-card"><div class="summary-row"><span>${t("common.chain")}</span><strong>${walletChainBadgeMarkup(data.chainId)}</strong></div><div class="summary-row"><span>Backup check</span><strong class="trust-label">${icon("shield")} Completed</strong></div></div>`,
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
  closeSelectPickers();
  const type = state.flow.type;
  dialog.dataset.flowType = type;
  const content = type === "asset-claim" ? assetClaimDialog()
    : type === "create-voucher" ? createVoucherDialog()
    : type === "voucher-detail" ? voucherDetailDialog()
    : type === "voucher-review" ? voucherDialog(false)
    : type === "voucher-settled" ? voucherDialog(true)
    : type === "create-permission" ? createPermissionDialog()
    : type === "permission" ? permissionDialog()
    : type === "permission-detail" ? permissionDetailDialog()
    : type === "activity" ? activityDialog(state.flow.data.item)
    : type === "asset-detail" ? assetDetailDialog()
    : type === "connection" ? connectionDialog()
    : type === "wallets" ? walletsDialog()
    : type === "networks" ? networksDialog()
    : type === "remove-wallet" ? removeWalletDialog()
    : type === "add-wallet" ? addWalletDialog()
    : type === "create-wallet" ? createWalletDialog()
    : type === "open-wallet" ? openWalletDialog()
    : type === "recover-wallet" ? recoverWalletDialog()
    : ["wallet-rename", "wallet-password-change", "wallet-seed-reveal", "wallet-public-export", "wallet-key-rotation", "wallet-policy-apply", "wallet-policy-profile"].includes(type) ? sensitiveWalletDialog(type)
    : notificationsDialog();
  dialogContent.innerHTML = content;
  suppressPasswordManagerUI(dialogContent);
  enhanceNativeSelects(dialogContent);
  persistDialogHistoryState();
}

function defaultFlowData(type) {
  if (type === "permission") return { delegate: "", action: "Deploy release", scope: "staging.example", uses: "1", expiry: "2026-08-19", expiryLabel: "19 Aug 2026" };
  if (type === "create-wallet") return { name: "", chainId: "mainnet", verificationIndexes: randomSeedVerificationIndexes() };
  if (type === "open-wallet") return { name: "Existing wallet" };
  if (type === "recover-wallet") return { name: "Recovered wallet" };
  if (type === "remove-wallet") return { walletIds: [] };
  return {};
}

function cloneFlowForHistory(flow = state.flow) {
  return flow ? JSON.parse(JSON.stringify(flow)) : null;
}

function persistDialogHistoryState() {
  if (!dialogHistoryActive || !state.flow) return;
  window.history.replaceState({
    ...(window.history.state || {}),
    z00zRoute: state.activeRoute,
    z00zOverlay: "flow-dialog",
    z00zFlow: cloneFlowForHistory()
  }, "", window.location.href);
}

function focusDialogPrimaryControl() {
  requestAnimationFrame(() => {
    const target = dialog.querySelector("input:not([type='hidden']), [data-select-picker-trigger], button:not([data-dialog-close])");
    target?.focus();
  });
}

function openFlow(type, trigger = document.activeElement, extraData = {}) {
  const isOpening = !dialog.open;
  if (isOpening) state.lastDialogTrigger = trigger;
  state.flow = { type, step: 0, data: { ...defaultFlowData(type), ...extraData } };
  renderDialog();
  if (isOpening) {
    window.history.pushState({
      ...(window.history.state || {}),
      z00zRoute: state.activeRoute,
      z00zOverlay: "flow-dialog",
      z00zFlow: cloneFlowForHistory()
    }, "", window.location.href);
    dialogHistoryActive = true;
    dialogHistoryClosing = false;
    dialog.showModal();
  }
  focusDialogPrimaryControl();
}

function closeDialog({ fromHistory = false } = {}) {
  if (!dialog.open) return;
  if (dialogHistoryActive && !fromHistory) {
    if (dialogHistoryClosing) return;
    persistDialogHistoryState();
    dialogHistoryClosing = true;
    window.history.back();
    return;
  }
  dialogHistoryActive = false;
  dialogHistoryClosing = false;
  dialog.close();
}

function restoreDialogFromHistory(flow) {
  if (!flow) return;
  state.flow = cloneFlowForHistory(flow);
  state.lastDialogTrigger = null;
  dialogHistoryActive = true;
  dialogHistoryClosing = false;
  renderDialog();
  if (!dialog.open) dialog.showModal();
  focusDialogPrimaryControl();
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

function qrCells(seed = "") {
  const size = 21;
  const fixedDark = new Set();
  const fixedLight = new Set();
  const seedValue = Array.from(seed).reduce((total, character, index) => total + character.charCodeAt(0) * (index + 1), 0);
  function square(startX, startY) {
    for (let y = 0; y < 7; y += 1) {
      for (let x = 0; x < 7; x += 1) {
        const index = (startY + y) * size + startX + x;
        const isDark = x === 0 || y === 0 || x === 6 || y === 6 || (x >= 2 && x <= 4 && y >= 2 && y <= 4);
        (isDark ? fixedDark : fixedLight).add(index);
      }
    }
  }
  square(0, 0); square(14, 0); square(0, 14);
  return Array.from({ length: size * size }, (_, index) => {
    let hash = (seedValue ^ Math.imul(index + 1, 0x45d9f3b)) >>> 0;
    hash = Math.imul(hash ^ (hash >>> 16), 0x45d9f3b);
    hash = Math.imul(hash ^ (hash >>> 16), 0x45d9f3b);
    const pseudo = ((hash ^ (hash >>> 16)) & 1) === 1;
    const isDark = fixedDark.has(index) || (!fixedLight.has(index) && pseudo);
    return `<span class="${isDark ? "is-dark" : ""}"></span>`;
  }).join("");
}

function validateSend(form) {
  const draft = activeSendDraft();
  const recipient = form.elements.recipient;
  const amount = form.elements.amount || null;
  const item = sendOptionEntries(draft.family).find((entry) => entry.key === form.elements.itemKey.value);
  let valid = true;
  document.querySelector("#send-recipient-error").textContent = "";
  document.querySelector("#send-amount-error").textContent = "";
  recipient.removeAttribute("aria-invalid");
  amount?.removeAttribute("aria-invalid");

  if (recipient.value.trim().length < 3) {
    document.querySelector("#send-recipient-error").textContent = "Enter or scan a valid recipient request.";
    recipient.setAttribute("aria-invalid", "true");
    valid = false;
  }
  if (!item) {
    document.querySelector("#send-amount-error").textContent = "Choose an available wallet item.";
    valid = false;
  }
  let normalizedAmount = "";
  if (item?.family === "asset") {
    const number = Number(amount?.value);
    if (!Number.isFinite(number) || number <= 0 || number > Number(item.asset.balance.replaceAll(",", "")) || (!item.asset.divisible && !Number.isInteger(number))) {
      const minimum = item.asset.divisible ? "0.01" : "1";
      document.querySelector("#send-amount-error").textContent = `Enter ${item.asset.divisible ? "an amount" : "a whole unit"} between ${minimum} and ${item.asset.balance} ${item.asset.unit}.`;
      amount?.setAttribute("aria-invalid", "true");
      valid = false;
    } else {
      normalizedAmount = item.asset.divisible ? number.toFixed(2) : String(number);
    }
  }
  if (!valid) {
    form.querySelector('[aria-invalid="true"]')?.focus();
    return;
  }

  Object.assign(draft, {
    recipient: recipient.value.trim(),
    recipientLabel: recipient.value.trim().startsWith("z00z:") ? "Verified asset request" : recipient.value.trim(),
    amount: normalizedAmount,
    memo: form.elements.memo.value.trim(),
    itemKey: item.key,
    reviewedItem: structuredClone(item),
    step: 1,
    completed: null
  });
  render({ focusMain: true });
}

function validateExchange(form) {
  const draft = captureExchangeDraft(form);
  const source = supportedAsset(draft.sourceAssetKey);
  const amount = Number(draft.amount);
  const error = document.querySelector("#exchange-error");
  const amountInput = form.elements.amount;
  error.textContent = "";
  amountInput.removeAttribute("aria-invalid");

  if (!Number.isFinite(amount) || amount <= 0 || amount > Number(source.balance.replaceAll(",", "")) || (!source.divisible && !Number.isInteger(amount))) {
    error.textContent = t("exchange.amountError", { balance: source.balance, unit: source.unit });
    amountInput.setAttribute("aria-invalid", "true");
    amountInput.focus();
    return;
  }
  if (draft.providerId === "hyperliquid" && draft.orderType === "limit" && !(Number(draft.limitPrice) > 0)) {
    error.textContent = t("exchange.limitPriceError");
    form.elements.limitPrice?.setAttribute("aria-invalid", "true");
    form.elements.limitPrice?.focus();
    return;
  }
  if (draft.providerId === "near-intents" && (draft.recipient.length < 3 || draft.refundAddress.length < 3)) {
    error.textContent = t("exchange.addressError");
    (draft.recipient.length < 3 ? form.elements.recipient : form.elements.refundAddress)?.setAttribute("aria-invalid", "true");
    (draft.recipient.length < 3 ? form.elements.recipient : form.elements.refundAddress)?.focus();
    return;
  }
  draft.amount = source.divisible ? amount.toFixed(2) : String(amount);
  draft.step = 1;
  render({ focusMain: true });
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
  const password = form.elements.walletSecret;
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
    const name = form.elements.walletLabel.value.trim();
    if (name.length < 2 || name.length > 32) {
      if (error) error.textContent = "Wallet name must contain 2–32 characters.";
      form.elements.walletLabel.setAttribute("aria-invalid", "true");
      form.elements.walletLabel.focus();
      return;
    }
    const result = walletGateway.renameWallet({ walletId: wallet.id, name });
    if (!result.ok) {
      if (error) error.textContent = result.error.message;
      form.elements.walletLabel.setAttribute("aria-invalid", "true");
      form.elements.walletLabel.focus();
      return;
    }
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

function validateWalletPasswordChange(form) {
  const currentPassword = form.elements.currentWalletSecret;
  const newPassword = form.elements.newWalletSecret;
  const confirmNewPassword = form.elements.confirmNewWalletSecret;
  const error = form.querySelector(".field-error");
  const fields = [currentPassword, newPassword, confirmNewPassword];
  if (error) error.textContent = "";
  fields.forEach((field) => field.removeAttribute("aria-invalid"));

  if (currentPassword.value.length < 8) {
    error.textContent = t("walletSettings.passwordCurrentError");
    currentPassword.setAttribute("aria-invalid", "true");
    currentPassword.focus();
    return;
  }
  if (newPassword.value.length < 8) {
    error.textContent = t("walletSettings.passwordNewError");
    newPassword.setAttribute("aria-invalid", "true");
    newPassword.focus();
    return;
  }
  if (newPassword.value === currentPassword.value) {
    error.textContent = t("walletSettings.passwordSameError");
    newPassword.setAttribute("aria-invalid", "true");
    newPassword.focus();
    return;
  }
  if (confirmNewPassword.value !== newPassword.value) {
    error.textContent = t("walletSettings.passwordMismatchError");
    confirmNewPassword.setAttribute("aria-invalid", "true");
    confirmNewPassword.focus();
    return;
  }

  const result = walletGateway.changePassword({
    walletId: activeWallet().id,
    currentPassword: currentPassword.value,
    newPassword: newPassword.value
  });
  if (!result.ok) {
    error.textContent = result.error.message;
    currentPassword.focus();
    return;
  }
  fields.forEach((field) => { field.value = ""; });
  state.flow.step = 1;
  renderDialog();
}

function setButtonLoading(button, label) {
  button.disabled = true;
  button.dataset.original = button.innerHTML;
  button.textContent = label;
}

function renderSendOperationIfCurrent(walletId, requestGeneration) {
  const draft = state.sendDrafts[walletId];
  if (state.selectedWalletId === walletId
    && state.activeRoute === "wallet.send"
    && draft?.requestGeneration === requestGeneration) {
    render({ focusMain: true });
  }
}

function beginSendOperation() {
  const draft = activeSendDraft();
  const wallet = activeWallet();
  const item = selectedSendOption(draft);
  if (!item) {
    draft.operationError = { code: "conflict", message: "This wallet item is no longer available." };
    draft.step = 2;
    render({ focusMain: true });
    return;
  }

  draft.requestGeneration += 1;
  const requestGeneration = draft.requestGeneration;
  draft.idempotencyKey ||= `demo-payment-${wallet.id}-${Date.now().toString(36)}`;
  draft.operationError = null;
  draft.operationStatus = "submitting";
  draft.step = 2;
  render({ focusMain: true });

  window.setTimeout(() => {
    const currentDraft = state.sendDrafts[wallet.id];
    if (!currentDraft || currentDraft.requestGeneration !== requestGeneration) return;
    const result = walletGateway.submitPayment({
      walletId: wallet.id,
      family: currentDraft.family,
      itemKey: currentDraft.itemKey,
      amount: currentDraft.amount,
      recipient: currentDraft.recipient,
      idempotencyKey: currentDraft.idempotencyKey,
      scenario: state.demoOperationScenario
    });
    if (!result.ok) {
      currentDraft.operationId = result.error.operationId || null;
      currentDraft.operationStatus = result.error.code === "timeout_unknown_outcome" ? "unknown_outcome" : "failed";
      currentDraft.operationError = { code: result.error.code, message: result.error.message };
      renderSendOperationIfCurrent(wallet.id, requestGeneration);
      return;
    }
    currentDraft.operationId = result.data.operationId;
    currentDraft.operationStatus = result.data.status;
    currentDraft.operationError = null;
    currentDraft.completed = { ...result.data.completed };
    currentDraft.step = 3;
    renderSendOperationIfCurrent(wallet.id, requestGeneration);
  }, 650);
}

function reconcileSendOperation() {
  const draft = activeSendDraft();
  const walletId = activeWallet().id;
  if (!draft.operationId) return;
  draft.requestGeneration += 1;
  const requestGeneration = draft.requestGeneration;
  draft.operationStatus = "reconciling";
  draft.operationError = null;
  draft.step = 2;
  render({ focusMain: true });

  window.setTimeout(() => {
    const currentDraft = state.sendDrafts[walletId];
    if (!currentDraft || currentDraft.requestGeneration !== requestGeneration) return;
    const result = walletGateway.reconcileOperation({ operationId: currentDraft.operationId });
    if (!result.ok) {
      currentDraft.operationStatus = "failed";
      currentDraft.operationError = { code: result.error.code, message: result.error.message };
    } else {
      currentDraft.operationStatus = result.data.status;
      currentDraft.completed = { ...result.data.completed };
      currentDraft.step = 3;
    }
    renderSendOperationIfCurrent(walletId, requestGeneration);
  }, 450);
}

function handleDialogAction(action, button) {
  if (action === "permission-back") {
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
    openFlow("voucher-review", button);
  } else if (action === "select-wallet") {
    closeDialog();
    state.selectedWalletId = button.dataset.walletId;
    state.view = "wallet";
    state.activityFilter = "all";
    state.assetFilter = "all";
    render({ focusMain: true });
    showToast(`${activeWallet().name} wallet opened in concept mode.`);
  } else if (action === "select-network") {
    closeDialog();
    state.view = "telemetry";
    state.telemetrySource = button.dataset.networkSection;
    state.isNetworkOpen = false;
    render({ focusMain: true });
  } else if (action === "confirm-remove-wallet") {
    const selectedIds = new Set(state.flow?.data.walletIds || []);
    const result = walletGateway.removeProfiles({
      walletIds: selectedIds,
      selectedWalletId: state.selectedWalletId
    });
    if (!result.ok) {
      showToast(result.error.message, "alert");
      return;
    }
    const { removed: walletsToRemove, selectedWalletId } = result.data;
    const needsWalletSetup = state.wallets.length === 0;
    if (needsWalletSetup) {
      state.selectedWalletId = null;
      state.view = "wallet";
    } else {
      state.selectedWalletId = selectedWalletId;
      state.view = "wallet";
    }
    state.activityFilter = "all";
    render({ focusMain: true });
    showToast(state.wallets.length === 0 ? "All wallets removed. Add a wallet to continue." : `${walletsToRemove.length} wallet${walletsToRemove.length === 1 ? "" : "s"} removed from this concept.`);
    if (needsWalletSetup) openFlow("add-wallet", button);
    else closeDialog();
  } else if (action === "add-wallet") {
    openFlow("add-wallet", button);
  } else if (["start-create", "start-recover"].includes(action)) {
    openFlow("add-wallet", button);
  } else if (["create-back-wallets", "recover-back-wallets"].includes(action)) {
    openFlow("add-wallet", button);
  } else if (action === "create-seed-saved") {
    state.flow.step = 2;
    renderDialog();
  } else if (action === "create-seed-back") {
    state.flow.data.verificationIndexes = randomSeedVerificationIndexes(state.flow.data.verificationIndexes);
    state.flow.step = 1;
    renderDialog();
  } else if (action === "create-finish" || action === "recover-finish") {
    const recovered = action === "recover-finish";
    const wallet = addWalletProfile(
      state.flow.data.name || (recovered ? "Recovered wallet" : "New wallet"),
      recovered ? "mainnet" : state.flow.data.chainId,
      "Scanning"
    );
    state.selectedWalletId = wallet.id;
    state.view = "wallet";
    state.activityFilter = "all";
    state.assetFilter = "all";
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
  if (action === "retry-workspace") {
    const url = new URL(window.location.href);
    url.searchParams.delete("workspaceFailure");
    history.replaceState(history.state, "", url);
    render({ focusMain: true });
  } else if (action === "toggle-balance") {
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
    openFlow("add-wallet", button);
  } else if (action === "remove-wallet") {
    if (state.wallets.length === 0) {
      openFlow("add-wallet", button);
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
  } else if (action === "network-picker") {
    openFlow("networks", button);
  } else if (action === "notifications") {
    openFlow("notifications", button);
  } else if (["copy-receipt", "copy-wallet-address", "copy-receiver-card"].includes(action)) {
    const messages = {
      "copy-receipt": "Public receipt copied.",
      "copy-wallet-address": "Wallet address copied.",
      "copy-receiver-card": "Receiver Card copied."
    };
    if (action === "copy-receiver-card" && button.dataset.copyValue) {
      navigator.clipboard?.writeText(button.dataset.copyValue).catch(() => {});
    }
    showToast(messages[action]);
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
  } else if (action === "prepare-unstake") {
    showToast(`${activeWallet().name} wallet needs an authoritative staked balance and unlock terms before unstaking can be reviewed.`);
  } else if (action === "asset-review") {
    showToast("Declared domain and metadata are not the same as an authoritative trust verdict.", "alert");
  } else if (action === "general-notifications") {
    state.notifications = !state.notifications;
    syncConfigDraftFromState();
    render();
    showToast(`Notifications ${state.notifications ? "enabled" : "disabled"}.`);
  } else if (action === "check-for-updates") {
    state.updateCheckStatus = "current";
    render();
    showToast(t("plan2.about.updateToast", { version: demoRuntime.APP_VERSION }));
  } else if (action === "motion") {
    state.reducedMotion = !state.reducedMotion;
    syncConfigDraftFromState();
    render();
    showToast(`Reduced motion ${state.reducedMotion ? "enabled" : "disabled"}.`);
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

document.addEventListener("z00z:help-opening", () => {
  closeMenuSearch();
  closeMobilePopup();
  closeDesktopWalletPicker();
});

function clearMobileDrawerMotion() {
  mobileDrawerMotionId += 1;
  mobileDrawerAnimations.forEach((animation) => animation.cancel());
  mobileDrawerAnimations = [];
  mobilePopupMenu.classList.remove("is-swipe-dragging", "is-swipe-settled");
  mobileMenuBackdrop.classList.remove("is-swipe-dragging", "is-swipe-settled");
  mobilePopupMenu.style.removeProperty("transform");
  mobileMenuBackdrop.style.removeProperty("opacity");
}

function settleMobileDrawerSwipe(shouldOpen, { offsetX, opacity }) {
  if (mobilePopupMenu.hidden) return;
  const drawerWidth = Math.max(1, mobilePopupMenu.getBoundingClientRect().width);
  const targetX = shouldOpen ? 0 : -drawerWidth;
  const targetOpacity = shouldOpen ? 1 : 0;
  const remaining = Math.min(1, Math.abs(targetX - offsetX) / drawerWidth);
  const reduceMotion = state.reducedMotion || window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const duration = reduceMotion ? 1 : Math.max(90, Math.round(220 * remaining));
  const motionId = ++mobileDrawerMotionId;

  mobileDrawerAnimations.forEach((animation) => animation.cancel());
  mobilePopupMenu.classList.add("is-swipe-dragging");
  mobileMenuBackdrop.classList.add("is-swipe-dragging");
  const drawerAnimation = mobilePopupMenu.animate([
    { transform: `translate3d(${offsetX}px, 0, 0)` },
    { transform: `translate3d(${targetX}px, 0, 0)` }
  ], {
    duration,
    easing: "cubic-bezier(0.22, 1, 0.36, 1)",
    fill: "forwards"
  });
  const backdropAnimation = mobileMenuBackdrop.animate([
    { opacity },
    { opacity: targetOpacity }
  ], {
    duration,
    easing: "ease-out",
    fill: "forwards"
  });
  mobileDrawerAnimations = [drawerAnimation, backdropAnimation];

  Promise.all([drawerAnimation.finished, backdropAnimation.finished]).then(() => {
    if (motionId !== mobileDrawerMotionId) return;
    mobileDrawerAnimations.forEach((animation) => animation.cancel());
    mobileDrawerAnimations = [];
    mobilePopupMenu.classList.remove("is-swipe-dragging");
    mobileMenuBackdrop.classList.remove("is-swipe-dragging");
    mobilePopupMenu.style.removeProperty("transform");
    mobileMenuBackdrop.style.removeProperty("opacity");
    if (shouldOpen) {
      mobilePopupMenu.classList.add("is-swipe-settled");
      mobileMenuBackdrop.classList.add("is-swipe-settled");
      focusMobileDrawer();
    } else {
      closeMobilePopup({ restoreFocus: true });
    }
  }).catch(() => {});
}

function resetMobileDrawerSwipe() {
  mobileDrawerSwipe.pointerId = null;
  mobileDrawerSwipe.source = "";
  mobileDrawerSwipe.direction = "";
  mobileDrawerSwipe.isDragging = false;
  mobileDrawerSwipe.offsetX = 0;
  mobileDrawerSwipe.opacity = 0;
}

function beginMobileDrawerSwipe({ source, pointerId, clientX, clientY, target }) {
  if (!isMobileNavigation()) return;
  const touchReplacesPointer = source === "touch" && mobileDrawerSwipe.source === "pointer";
  if (mobileDrawerSwipe.pointerId !== null && !touchReplacesPointer) return;
  if (dialog.open || document.querySelector("[data-select-picker].is-open, [data-language-picker].is-open")) return;

  const drawerIsOpen = !mobilePopupMenu.hidden && mobilePopupType === "menu";
  const startsInDrawer = target instanceof Element && Boolean(target.closest("#mobile-popup-menu"));
  if ((!drawerIsOpen && (!mobilePopupMenu.hidden || clientX > mobileDrawerSwipeEdge))
    || (drawerIsOpen && !startsInDrawer)) return;

  mobileDrawerSwipe.pointerId = pointerId;
  mobileDrawerSwipe.source = source;
  mobileDrawerSwipe.startX = clientX;
  mobileDrawerSwipe.startY = clientY;
  mobileDrawerSwipe.direction = drawerIsOpen ? "close" : "open";
  mobileDrawerSwipe.isDragging = false;
  mobileDrawerSwipe.offsetX = drawerIsOpen ? 0 : -window.innerWidth;
  mobileDrawerSwipe.opacity = drawerIsOpen ? 1 : 0;
}

function updateMobileDrawerSwipe({ source, pointerId, clientX, clientY }) {
  if (source !== mobileDrawerSwipe.source || pointerId !== mobileDrawerSwipe.pointerId) return false;
  const deltaX = clientX - mobileDrawerSwipe.startX;
  const deltaY = clientY - mobileDrawerSwipe.startY;
  const { direction } = mobileDrawerSwipe;

  if (!mobileDrawerSwipe.isDragging) {
    if (Math.abs(deltaX) < 8 && Math.abs(deltaY) < 8) return false;
    const isHorizontal = Math.abs(deltaX) > Math.abs(deltaY) * 1.2;
    const movesInDirection = direction === "open" ? deltaX > 0 : deltaX < 0;
    if (!isHorizontal || !movesInDirection) {
      mobileDrawerSwipe.direction = "";
      return false;
    }
    if (direction === "open" && mobilePopupMenu.hidden) {
      openMobilePopup(mobileMenuButton, { isSwipePreview: true });
    }
    mobileDrawerSwipe.isDragging = true;
    mobilePopupMenu.classList.add("is-swipe-dragging");
    mobileMenuBackdrop.classList.add("is-swipe-dragging");
  }

  const drawerWidth = Math.max(1, mobilePopupMenu.getBoundingClientRect().width);
  const offsetX = direction === "open"
    ? Math.min(0, -drawerWidth + Math.max(0, deltaX))
    : Math.max(-drawerWidth, Math.min(0, deltaX));
  const opacity = Math.max(0, Math.min(1, 1 + offsetX / drawerWidth));
  mobileDrawerSwipe.offsetX = offsetX;
  mobileDrawerSwipe.opacity = opacity;
  mobilePopupMenu.style.transform = `translate3d(${offsetX}px, 0, 0)`;
  mobileMenuBackdrop.style.opacity = String(opacity);
  return true;
}

function completeMobileDrawerSwipe({ source, pointerId, clientX, clientY }) {
  if (source !== mobileDrawerSwipe.source || pointerId !== mobileDrawerSwipe.pointerId) return;
  const {
    startX,
    startY,
    direction,
    isDragging,
    offsetX,
    opacity
  } = mobileDrawerSwipe;
  resetMobileDrawerSwipe();

  const deltaX = clientX - startX;
  const deltaY = clientY - startY;
  if (isDragging) {
    const commitsOpen = direction === "open"
      ? deltaX >= mobileDrawerSwipeDistance || opacity >= 0.35
      : !(Math.abs(deltaX) >= mobileDrawerSwipeDistance || opacity <= 0.65);
    settleMobileDrawerSwipe(commitsOpen, { offsetX, opacity });
    return;
  }
  const isHorizontalSwipe = Math.abs(deltaX) >= mobileDrawerSwipeDistance
    && Math.abs(deltaX) > Math.abs(deltaY) * 1.25;
  if (!isHorizontalSwipe) return;

  if (direction === "open" && deltaX > 0 && mobilePopupMenu.hidden) {
    openMobilePopup(mobileMenuButton);
  } else if (direction === "close" && deltaX < 0 && !mobilePopupMenu.hidden && mobilePopupType === "menu") {
    closeMobilePopup({ restoreFocus: true });
  }
}

function cancelMobileDrawerSwipe({ source, pointerId }) {
  if (source !== mobileDrawerSwipe.source || pointerId !== mobileDrawerSwipe.pointerId) return;
  const {
    direction,
    isDragging,
    offsetX,
    opacity
  } = mobileDrawerSwipe;
  resetMobileDrawerSwipe();
  if (isDragging) {
    settleMobileDrawerSwipe(direction === "close", { offsetX, opacity });
  }
}

document.addEventListener("pointerdown", (event) => {
  if (event.pointerType !== "touch" || event.isPrimary === false) return;
  beginMobileDrawerSwipe({
    source: "pointer",
    pointerId: event.pointerId,
    clientX: event.clientX,
    clientY: event.clientY,
    target: event.target
  });
});

document.addEventListener("pointermove", (event) => {
  if (event.pointerType !== "touch" || event.isPrimary === false) return;
  if (updateMobileDrawerSwipe({
    source: "pointer",
    pointerId: event.pointerId,
    clientX: event.clientX,
    clientY: event.clientY
  }) && event.cancelable) event.preventDefault();
});

document.addEventListener("pointerup", (event) => {
  completeMobileDrawerSwipe({
    source: "pointer",
    pointerId: event.pointerId,
    clientX: event.clientX,
    clientY: event.clientY
  });
});

document.addEventListener("pointercancel", (event) => {
  cancelMobileDrawerSwipe({ source: "pointer", pointerId: event.pointerId });
});

document.addEventListener("touchstart", (event) => {
  const touch = event.changedTouches[0];
  if (!touch) return;
  beginMobileDrawerSwipe({
    source: "touch",
    pointerId: touch.identifier,
    clientX: touch.clientX,
    clientY: touch.clientY,
    target: event.target
  });
}, { passive: true });

document.addEventListener("touchmove", (event) => {
  const touch = event.changedTouches[0];
  if (!touch) return;
  if (updateMobileDrawerSwipe({
    source: "touch",
    pointerId: touch.identifier,
    clientX: touch.clientX,
    clientY: touch.clientY
  }) && event.cancelable) event.preventDefault();
}, { passive: false });

document.addEventListener("touchend", (event) => {
  const touch = event.changedTouches[0];
  if (!touch) return;
  completeMobileDrawerSwipe({
    source: "touch",
    pointerId: touch.identifier,
    clientX: touch.clientX,
    clientY: touch.clientY
  });
}, { passive: true });

document.addEventListener("touchcancel", (event) => {
  const touch = event.changedTouches[0];
  if (!touch) return;
  cancelMobileDrawerSwipe({ source: "touch", pointerId: touch.identifier });
}, { passive: true });

document.addEventListener("contextmenu", (event) => {
  if (event.target.closest("[data-wallet-section]")) event.preventDefault();
});

menuSearchTrigger.addEventListener("click", openMenuSearch);
menuSearchBackdrop.addEventListener("pointerdown", () => closeMenuSearch({ restoreFocus: true }));
menuSearchClose.addEventListener("click", () => closeMenuSearch({ restoreFocus: true }));
menuSearchInput.addEventListener("input", () => {
  menuSearchQuery = menuSearchInput.value;
  renderMenuSearch();
});
menuSearchResults.addEventListener("click", (event) => {
  const result = event.target.closest("[data-menu-search-node]");
  if (!result) return;
  activateMenuSearchNode(result.dataset.menuSearchNode);
});

document.addEventListener("click", (event) => {
  if (event.target.closest("#mobile-menu-backdrop")) {
    closeMobilePopup({ restoreFocus: true });
    return;
  }

  const walletPickerTrigger = event.target.closest("[data-wallet-picker-trigger]");
  if (walletPickerTrigger) {
    openWalletPicker(walletPickerTrigger);
    return;
  }

  const walletPickerAction = event.target.closest("[data-wallet-picker-action]");
  if (walletPickerAction) {
    const trigger = desktopWalletPickerTrigger;
    closeDesktopWalletPicker();
    closeMobilePopup();
    handleDemoAction(walletPickerAction.dataset.walletPickerAction, trigger || walletPickerAction);
    return;
  }

  const walletPickerChoice = event.target.closest("[data-wallet-picker-id]");
  if (walletPickerChoice) {
    selectWalletFromPicker(walletPickerChoice.dataset.walletPickerId);
    return;
  }

  const mobileMenuToggle = event.target.closest("#mobile-menu-button");
  if (mobileMenuToggle) {
    openMobilePopup(mobileMenuToggle);
    return;
  }

  const navigationBranch = event.target.closest("[data-navigation-branch]");
  if (navigationBranch) {
    const nodeId = navigationBranch.dataset.navigationBranch;
    const branchWasMobile = Boolean(navigationBranch.closest("#mobile-popup-menu"));
    mergeShellState({ type: "toggle_branch", nodeId });
    render();
    requestAnimationFrame(() => document.querySelectorAll(`[data-navigation-branch="${CSS.escape(nodeId)}"]`).forEach((button) => {
      if (Boolean(button.closest("#mobile-popup-menu")) === branchWasMobile) {
        button.focus({ preventScroll: true });
      }
    }));
    return;
  }

  const navigationRoute = event.target.closest("[data-navigation-route]");
  if (navigationRoute) {
    selectCanonicalRoute(navigationRoute.dataset.navigationRoute);
    closeMobilePopup();
    render({ focusMain: true });
    return;
  }

  if (!mobilePopupMenu.hidden
    && !event.target.closest("#mobile-popup-menu")
    && !event.target.closest("#wallet-picker-popup")
    && !event.target.closest("#mobile-menu-button")
    && !event.target.closest("[data-wallet-picker-trigger]")) {
    closeMobilePopup();
  }

  if (!walletPickerPopup.hidden
    && !event.target.closest("#wallet-picker-popup")
    && !event.target.closest("[data-wallet-picker-trigger]")) {
    closeDesktopWalletPicker();
  }

  const viewButton = event.target.closest("[data-view]");
  if (viewButton) {
    const view = viewButton.dataset.view;
    state.view = view;
    render({ focusMain: true });
    return;
  }

  const sendFamilyButton = event.target.closest("[data-send-family]");
  if (sendFamilyButton) {
    const draft = activeSendDraft();
    clearExternalReviewHandoffs();
    draft.family = sendFamilyButton.dataset.sendFamily;
    draft.itemKey = sendOptionEntries(draft.family)[0]?.key || "";
    draft.amount = "";
    draft.step = 0;
    draft.reviewedItem = null;
    draft.completed = null;
    draft.idempotencyKey = "";
    draft.operationId = null;
    draft.operationStatus = null;
    draft.operationError = null;
    draft.requestGeneration += 1;
    render({ focusMain: true });
    return;
  }

  const exchangeProviderButton = event.target.closest("[data-exchange-provider]");
  if (exchangeProviderButton) {
    const form = document.querySelector("#exchange-entry");
    const draft = form ? captureExchangeDraft(form) : activeExchangeDraft();
    draft.providerId = exchangeProviderButton.dataset.exchangeProvider;
    draft.destinationId = demoRuntime.exchangeProvider(draft.providerId).defaultDestination;
    draft.orderType = "market";
    draft.limitPrice = "";
    draft.step = 0;
    render({ focusMain: true });
    requestAnimationFrame(() => document.querySelector(`[data-exchange-provider="${draft.providerId}"]`)?.focus());
    return;
  }

  const exchangeActionButton = event.target.closest("[data-exchange-action]");
  if (exchangeActionButton) {
    const draft = activeExchangeDraft();
    draft.step = 0;
    render({ focusMain: true });
    return;
  }

  const sendActionButton = event.target.closest("[data-send-action]");
  if (sendActionButton) {
    const action = sendActionButton.dataset.sendAction;
    const draft = activeSendDraft();
    if (action === "cancel") {
      clearExternalReviewHandoffs();
      resetActiveSendDraft();
      state.view = "wallet";
      render({ focusMain: true });
    } else if (action === "back") {
      draft.step = draft.operationError ? 1 : 0;
      draft.completed = null;
      draft.operationError = null;
      render({ focusMain: true });
    } else if (action === "submit") {
      beginSendOperation();
    } else if (action === "reconcile") {
      reconcileSendOperation();
    } else if (action === "history") {
      clearExternalReviewHandoffs();
      resetActiveSendDraft();
      selectCanonicalRoute("wallet.history");
      render({ focusMain: true });
    } else if (action === "done") {
      clearExternalReviewHandoffs();
      resetActiveSendDraft();
      render({ focusMain: true });
    }
    return;
  }

  const assetImportActionButton = event.target.closest("[data-asset-import-action]");
  if (assetImportActionButton) {
    const action = assetImportActionButton.dataset.assetImportAction;
    if (action === "reset") {
      resetAssetImportState();
      render({ focusMain: true });
      requestAnimationFrame(() => document.querySelector("#asset-import-file")?.focus());
    } else if (action === "prepare") {
      const importState = activeAssetImportState();
      const result = walletGateway.prepareAssetImport({
        walletId: activeWallet().id,
        reviewToken: importState.reviewToken
      });
      if (!result.ok) {
        importState.status = "idle";
        importState.preview = null;
        importState.reviewToken = "";
        importState.error = result.error;
        render({ focusMain: true });
        return;
      }
      importState.status = "prepared";
      importState.result = result.data;
      importState.error = null;
      render({ focusMain: true });
      showToast("Asset package is ready for native wallet verification.");
    }
    return;
  }

  const mergeSplitModeButton = event.target.closest("[data-merge-split-mode]");
  if (mergeSplitModeButton) {
    const mergeSplitState = activeMergeSplitState();
    mergeSplitState.mode = mergeSplitModeButton.dataset.mergeSplitMode === "split" ? "split" : "merge";
    mergeSplitState.preview = null;
    mergeSplitState.error = "";
    render({ focusMain: true });
    requestAnimationFrame(() => document.querySelector(`[data-merge-split-mode="${mergeSplitState.mode}"]`)?.focus({ preventScroll: true }));
    return;
  }

  const mergeSplitActionButton = event.target.closest("[data-merge-split-action]");
  if (mergeSplitActionButton) {
    const action = mergeSplitActionButton.dataset.mergeSplitAction;
    const mergeSplitState = activeMergeSplitState();
    mergeSplitState.error = "";
    if (action === "edit") {
      mergeSplitState.preview = null;
    } else if (action === "add-output" && mergeSplitState.splitAmounts.length < 8) {
      mergeSplitState.splitAmounts.push("");
    } else if (action === "remove-output" && mergeSplitState.splitAmounts.length > 2) {
      const index = Number(mergeSplitActionButton.dataset.outputIndex);
      if (Number.isInteger(index) && index >= 0 && index < mergeSplitState.splitAmounts.length) {
        mergeSplitState.splitAmounts.splice(index, 1);
      }
    } else if (action === "preview-merge") {
      const inputs = MERGE_SPLIT_ASSET_FIXTURES.filter(({ id }) => mergeSplitState.selectedMergeIds.includes(id));
      const compatible = inputs.length >= 2
        && inputs.every(({ definitionId, serialId, status }) => (
          definitionId === inputs[0].definitionId
          && serialId === inputs[0].serialId
          && status === "available"
        ));
      if (!compatible) {
        mergeSplitState.error = "Select at least two available fragments from one definition and one serial.";
      } else {
        mergeSplitState.preview = {
          mode: "merge",
          inputs,
          totalAtomic: inputs.reduce((sum, { amountAtomic }) => sum + amountAtomic, 0)
        };
      }
    } else if (action === "preview-split") {
      const source = MERGE_SPLIT_ASSET_FIXTURES.find(({ id, status }) => id === mergeSplitState.selectedSplitId && status === "available");
      const outputs = source
        ? mergeSplitState.splitAmounts.map((value) => parseAtomicAmount(value, source.decimals))
        : [];
      const valid = Boolean(source)
        && outputs.length >= 2
        && outputs.every((value) => Number.isSafeInteger(value) && value > 0)
        && outputs.reduce((sum, value) => sum + value, 0) === source.amountAtomic;
      if (!valid) {
        mergeSplitState.error = "Every output must be positive and the output sum must equal the source amount exactly.";
      } else {
        mergeSplitState.preview = {
          mode: "split",
          asset: source,
          outputs,
          totalAtomic: source.amountAtomic
        };
      }
    }
    render({ focusMain: true });
    return;
  }

  const walletSectionButton = event.target.closest("[data-wallet-section]");
  if (walletSectionButton) {
    selectCanonicalRoute(`wallet.${walletSectionButton.dataset.walletSection}`);
    render({ focusMain: true });
    return;
  }

  const walletSettingsSectionButton = event.target.closest("[data-wallet-settings-section]");
  if (walletSettingsSectionButton) {
    selectCanonicalRoute(`wallet.settings.${walletSettingsSectionButton.dataset.walletSettingsSection}`);
    render({ focusMain: true });
    return;
  }

  const flowButton = event.target.closest("[data-open-flow]");
  if (flowButton) {
    const flowData = flowButton.dataset.assetKey
      ? { assetKey: flowButton.dataset.assetKey }
      : flowButton.dataset.objectKind
        ? { objectKind: flowButton.dataset.objectKind, objectId: flowButton.dataset.objectId }
        : flowButton.dataset.objectId
          ? { objectId: flowButton.dataset.objectId }
      : flowButton.dataset.permissionId
        ? { permissionId: flowButton.dataset.permissionId }
        : {};
    openFlow(flowButton.dataset.openFlow, flowButton, flowData);
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
    clearExternalReviewHandoffs();
    state.selectedWalletId = walletButton.dataset.walletId;
    state.view = "wallet";
    state.activityFilter = "all";
    state.assetFilter = "all";
    render({ focusMain: true });
    return;
  }

  const filterButton = event.target.closest("[data-filter]");
  if (filterButton) {
    state.activityFilter = filterButton.dataset.filter;
    render();
    return;
  }

  const assetFilterButton = event.target.closest("[data-asset-filter]");
  if (assetFilterButton) {
    state.assetFilter = assetFilterButton.dataset.assetFilter;
    render();
    return;
  }

  const settingButton = event.target.closest("[data-settings-section]");
  if (settingButton) {
    const section = settingButton.dataset.settingsSection;
    state.settingsSection = section;
    state.isNetworkOpen = ["reticulum", "onionnet"].includes(section);
    if (state.isNetworkOpen) state.networkSection = section;
    render();
    return;
  }

  const workspaceRouteButton = event.target.closest("[data-workspace-route]");
  if (workspaceRouteButton) {
    selectCanonicalRoute(workspaceRouteButton.dataset.workspaceRoute);
    render({ focusMain: true });
    return;
  }

  const watcherAlertButton = event.target.closest("[data-watcher-alert]");
  if (watcherAlertButton) {
    state.watcherSelectedAlertId = watcherAlertButton.dataset.watcherAlert;
    state.watcherExportEnvelope = null;
    render();
    requestAnimationFrame(() => document.querySelector(`[data-watcher-alert="${CSS.escape(state.watcherSelectedAlertId)}"]`)?.focus());
    return;
  }

  const watcherActionButton = event.target.closest("[data-watcher-action]");
  if (watcherActionButton) {
    const action = watcherActionButton.dataset.watcherAction;
    if (action === "open-explorer") {
      const deepLink = telemetryGateway.resolveExplorerDeepLink({
        publicId: watcherActionButton.dataset.publicId
      });
      if (!deepLink.ok) {
        showToast(deepLink.error.message, "alert");
        return;
      }
      state.explorerScenario = "success";
      state.explorerSelectedPublicId = deepLink.publicId;
      state.explorerSearchResult = null;
      state.explorerQuery = "";
      state.explorerEvidenceKindFilter = "all";
      state.explorerDetailMode = "summary";
      selectCanonicalRoute(deepLink.routeId);
      render({ focusMain: true });
      showToast(`Opened ${deepLink.publicKind.replaceAll("_", " ")} public evidence.`);
    } else if (action === "inspect-evidence") {
      state.watcherSelectedAlertId = watcherActionButton.dataset.alertId;
      state.watcherSeverityFilter = "all";
      state.watcherKindFilter = "all";
      state.watcherExportEnvelope = null;
      selectCanonicalRoute("telemetry.watchers.evidence");
      render({ focusMain: true });
    } else if (action === "export-evidence") {
      state.watcherSelectedAlertId = watcherActionButton.dataset.alertId;
      state.watcherExportEnvelope = telemetryGateway.prepareWatcherEvidenceExport({
        alertId: state.watcherSelectedAlertId,
        sourceId: state.watcherSourceId
      });
      render();
      showToast("Sanitized Watcher evidence fixture prepared.");
    } else if (action === "clear-filters") {
      state.watcherSeverityFilter = "all";
      state.watcherKindFilter = "all";
      state.watcherScenario = "success";
      state.watcherExportEnvelope = null;
      render();
    } else if (action === "recover") {
      mergeShellState({ type: "begin_request", requestKey: `telemetry:${state.activeRoute}` });
      state.watcherScenario = "success";
      state.watcherExportEnvelope = null;
      render();
      showToast("Deterministic Watchers fixture refreshed.");
    }
    return;
  }

  const explorerRecordButton = event.target.closest("[data-explorer-record]");
  if (explorerRecordButton) {
    state.explorerSelectedPublicId = explorerRecordButton.dataset.explorerRecord;
    state.explorerDetailMode = "summary";
    render();
    requestAnimationFrame(() => document.querySelector(`[data-explorer-record="${CSS.escape(state.explorerSelectedPublicId)}"]`)?.focus());
    return;
  }

  const explorerExampleButton = event.target.closest("[data-explorer-example-id]");
  if (explorerExampleButton) {
    state.explorerQuery = explorerExampleButton.dataset.explorerExampleId;
    state.explorerSearchResult = telemetryGateway.searchExplorerPublicId({
      query: state.explorerQuery,
      scenario: state.explorerScenario,
      generation: Number(state.requestGenerations["telemetry:telemetry.explorer.search"] || 0)
    });
    state.explorerSelectedPublicId = state.explorerSearchResult.status === "found"
      ? state.explorerSearchResult.publicId
      : null;
    state.explorerDetailMode = "summary";
    render();
    requestAnimationFrame(() => document.querySelector("#explorer-public-id")?.focus());
    return;
  }

  const explorerRelatedButton = event.target.closest("[data-explorer-open-id]");
  if (explorerRelatedButton) {
    const publicId = explorerRelatedButton.dataset.explorerOpenId;
    const result = telemetryGateway.searchExplorerPublicId({ query: publicId, scenario: "success" });
    if (result.status !== "found") {
      state.explorerSearchResult = result;
      state.explorerQuery = "";
      selectCanonicalRoute("telemetry.explorer.search");
    } else {
      state.explorerSelectedPublicId = result.publicId;
      state.explorerDetailMode = "summary";
      const routeId = result.record.recordType === "checkpoint"
        ? "telemetry.explorer.checkpoints"
        : result.record.recordType === "batch"
          ? "telemetry.explorer.batches"
          : "telemetry.explorer.evidence";
      selectCanonicalRoute(routeId);
    }
    render({ focusMain: true });
    return;
  }

  const explorerActionButton = event.target.closest("[data-explorer-action]");
  if (explorerActionButton) {
    const action = explorerActionButton.dataset.explorerAction;
    if (action === "summary" || action === "technical") {
      state.explorerDetailMode = action;
      render();
    } else if (action === "clear-search") {
      state.explorerQuery = "";
      state.explorerSearchResult = null;
      state.explorerSelectedPublicId = null;
      state.explorerDetailMode = "summary";
      render();
      requestAnimationFrame(() => document.querySelector("#explorer-public-id")?.focus());
    } else if (action === "clear-filter") {
      state.explorerEvidenceKindFilter = "all";
      state.explorerScenario = "success";
      state.explorerSelectedPublicId = null;
      render();
    } else if (action === "recover") {
      mergeShellState({ type: "begin_request", requestKey: `telemetry:${state.activeRoute}` });
      state.explorerScenario = "success";
      state.explorerSearchResult = null;
      state.explorerSelectedPublicId = null;
      state.explorerDetailMode = "summary";
      render();
      showToast("Deterministic public evidence fixture refreshed.");
    }
    return;
  }

  const dappActionButton = event.target.closest("[data-dapp-action]");
  if (dappActionButton) {
    const action = dappActionButton.dataset.dappAction;
    if (action === "route") {
      const routeId = dappActionButton.dataset.dappRoute;
      if (!demoRuntime.PORT_CONTRACT.dappRoutes.includes(routeId)) {
        showToast("Unknown local dApp route.", "alert");
        return;
      }
      selectCanonicalRoute(routeId);
      render({ focusMain: true });
    } else if (action === "open") {
      const descriptor = demoRuntime.dappDescriptor(dappActionButton.dataset.dappId);
      if (!descriptor) {
        showToast("Unknown local dApp descriptor.", "alert");
        return;
      }
      state.dappSelectedId = descriptor.id;
      state.dappReviewConnectionId = null;
      state.dappReviewValidationError = null;
      state.dappScreen = "detail";
      render({ focusMain: true });
    } else if (action === "back") {
      state.dappScreen = "list";
      state.dappSelectedId = null;
      state.dappReviewConnectionId = null;
      state.dappReviewValidationError = null;
      state.dappReviewAcknowledgements = {
        scopeConfirmed: false,
        reauthAcknowledged: false
      };
      render({ focusMain: true });
    } else if (action === "review") {
      const connection = demoRuntime.DAPP_CONNECTION_FIXTURES.find(({ id }) => id === dappActionButton.dataset.connectionId);
      if (!connection) {
        showToast("Unknown local connection fixture.", "alert");
        return;
      }
      clearExternalReviewHandoffs();
      state.dappSelectedId = connection.descriptorId;
      state.dappReviewConnectionId = connection.id;
      state.dappReviewValidationError = null;
      state.dappReviewAcknowledgements = {
        scopeConfirmed: false,
        reauthAcknowledged: false
      };
      state.dappScreen = "review";
      render({ focusMain: true });
    } else if (action === "decide") {
      const decision = dappActionButton.dataset.decision;
      if (decision !== "rejected") {
        showToast("The local review decision failed closed.", "alert");
        return;
      }
      clearExternalReviewHandoffs();
      completeDappPermissionReview("rejected");
    } else if (action === "wallet-review") {
      const handoffResult = dappGateway.prepareWalletReview({
        decision: state.dappReviewDecision
      });
      if (!handoffResult.ok) {
        showToast(handoffResult.error.message, "alert");
        return;
      }
      const handoff = handoffResult.data;
      clearExternalReviewHandoffs();
      state.dappWalletReviewHandoff = handoff;
      selectCanonicalRoute(handoff.target.routeId);
      if (handoff.target.flow === "send") {
        const draft = resetActiveSendDraft();
        Object.assign(draft, {
          ...handoff.draft,
          step: 0,
          recipientLabel: "",
          reviewedItem: null,
          completed: null,
          idempotencyKey: "",
          operationId: null,
          operationStatus: null,
          operationError: null
        });
      }
      render({ focusMain: true });
      showToast("Typed dApp intent opened in Wallet review.");
    } else if (action === "outcome-back") {
      selectCanonicalRoute(dappActionButton.dataset.returnRoute);
      render({ focusMain: true });
    }
    return;
  }

  const messengerActionButton = event.target.closest("[data-messenger-action]");
  if (messengerActionButton) {
    const action = messengerActionButton.dataset.messengerAction;
    const messageId = messengerActionButton.dataset.messageId || state.messengerSelectedMessageId;
    if (action === "open") {
      const opened = messengerGateway.advisoryAction({ messageId, action: "opened" });
      if (!opened.ok) {
        showToast(opened.error.message, "alert");
        return;
      }
      state.messengerSelectedMessageId = messageId;
      state.messengerReviewError = null;
      state.messengerScreen = "detail";
      render({ focusMain: true });
    } else if (action === "back") {
      state.messengerScreen = "list";
      state.messengerSelectedMessageId = null;
      state.messengerReviewDecision = null;
      state.messengerReviewError = null;
      state.messengerLastOutcome = null;
      render({ focusMain: true });
    } else if (action === "detail") {
      state.messengerReviewError = null;
      state.messengerScreen = "detail";
      render({ focusMain: true });
    } else if (action === "review") {
      const review = messengerGateway.readRequestReview({ messageId });
      if (!review.ok) {
        showToast(review.error.message, "alert");
        return;
      }
      state.messengerSelectedMessageId = messageId;
      state.messengerReviewError = null;
      state.messengerScreen = "review";
      render({ focusMain: true });
    } else if (action === "accept-request" || action === "reject-request") {
      const review = messengerGateway.readRequestReview({ messageId });
      const decision = action === "accept-request" ? "accepted" : "rejected";
      const result = review.ok
        ? messengerGateway.decideRequest({ reviewId: review.data.reviewId, decision })
        : review;
      if (!result.ok) {
        state.messengerReviewError = result.error.message;
        render();
        return;
      }
      state.messengerReviewDecision = result.data;
      state.messengerLastOutcome = {
        kind: decision,
        title: decision === "accepted" ? "Request accepted for Wallet review" : "Advisory request rejected",
        summary: decision === "accepted"
          ? "A typed local decision is ready for separate Wallet validation. No Wallet operation or settlement change exists yet."
          : "The request ended locally before any Wallet review, signing, value mutation, or settlement path."
      };
      state.messengerReviewError = null;
      state.messengerScreen = "outcome";
      render({ focusMain: true });
      showToast(decision === "accepted" ? "Request accepted locally." : "Request rejected locally.");
    } else if (action === "wallet-review") {
      const handoffResult = messengerGateway.prepareWalletReview({
        decision: state.messengerReviewDecision
      });
      if (!handoffResult.ok) {
        showToast(handoffResult.error.message, "alert");
        return;
      }
      const validation = walletGateway.revalidateExternalReviewHandoff({
        walletId: activeWallet().id,
        handoff: handoffResult.data
      });
      if (!validation.ok) {
        showToast(validation.error.message, "alert");
        return;
      }
      const handoff = handoffResult.data;
      clearExternalReviewHandoffs();
      state.messengerWalletReviewHandoff = handoff;
      selectCanonicalRoute(validation.data.target.routeId);
      if (validation.data.target.flow === "send") {
        const draft = resetActiveSendDraft();
        Object.assign(draft, {
          ...validation.data.draft,
          step: 0,
          recipientLabel: "",
          reviewedItem: null,
          completed: null,
          idempotencyKey: "",
          operationId: null,
          operationStatus: null,
          operationError: null
        });
      }
      render({ focusMain: true });
      showToast("Messenger request revalidated by Wallet.");
    } else if (["acknowledge", "delete", "block", "report"].includes(action)) {
      const gatewayAction = {
        acknowledge: "acknowledged",
        delete: "deleted",
        block: "blocked",
        report: "reported"
      }[action];
      const result = messengerGateway.advisoryAction({ messageId, action: gatewayAction });
      if (!result.ok) {
        showToast(result.error.message, "alert");
        return;
      }
      if (action === "acknowledge") state.messengerAcknowledgedIds = [...new Set([...state.messengerAcknowledgedIds, messageId])];
      if (action === "delete") state.messengerDeletedIds = [...new Set([...state.messengerDeletedIds, messageId])];
      if (action === "report") state.messengerReportedIds = [...new Set([...state.messengerReportedIds, messageId])];
      if (action === "block") state.messengerBlockedSenders = [...new Set([...state.messengerBlockedSenders, result.data.senderLabel])];
      if (["delete", "block"].includes(action)) {
        state.messengerScreen = "list";
        state.messengerSelectedMessageId = null;
      }
      render({ focusMain: true });
      showToast(`${dappTitleCase(gatewayAction)} locally; Wallet state unchanged.`);
    } else if (action === "relay-unavailable") {
      state.messengerRelayScenario = "unavailable";
      render();
      showToast("Unavailable relay fixture selected.");
    } else if (action === "relay-recover") {
      state.messengerRelayScenario = state.messengerRelayScenario === "recovering" ? "available" : "recovering";
      render();
      showToast(state.messengerRelayScenario === "recovering" ? "Local recovery check started." : "Local relay fixture restored.");
    }
    return;
  }

  const contactActionButton = event.target.closest("[data-contact-action]");
  if (contactActionButton) {
    const action = contactActionButton.dataset.contactAction;
    const contactId = contactActionButton.dataset.contactId || state.contactsSelectedId;
    if (action === "open") {
      state.contactsSelectedId = contactId;
      state.contactsFormError = null;
      state.contactsScreen = "detail";
      render({ focusMain: true });
    } else if (action === "back") {
      state.contactsSelectedId = null;
      state.contactsFormError = null;
      state.contactsLastOutcome = null;
      state.contactsScreen = "list";
      render({ focusMain: true });
    } else if (action === "detail") {
      state.contactsFormError = null;
      state.contactsScreen = "detail";
      render({ focusMain: true });
    } else if (action === "add") {
      state.contactsImportSourceId = "receiver_card";
      state.contactsFormError = null;
      state.contactsScreen = "import";
      render({ focusMain: true });
    } else if (action === "import-source") {
      const preview = contactsGateway.createImportPreview({
        sourceId: contactActionButton.dataset.sourceId
      });
      if (!preview.ok) {
        showToast(preview.error.message, "alert");
        return;
      }
      state.contactsImportSourceId = preview.data.source.id;
      state.contactsFormError = null;
      render();
    } else if (action === "edit") {
      state.contactsSelectedId = contactId;
      state.contactsFormError = null;
      state.contactsScreen = "edit";
      render({ focusMain: true });
    } else if (action === "identity-review") {
      state.contactsSelectedId = contactId;
      state.contactsScreen = "identity-review";
      render({ focusMain: true });
    } else if (action === "identity-accept" || action === "identity-reject") {
      const decision = action === "identity-accept" ? "accepted" : "rejected";
      const result = contactsGateway.reviewIdentityChange({
        contactId,
        decision
      });
      if (!result.ok) {
        showToast(result.error.message, "alert");
        return;
      }
      state.contactsLastOutcome = {
        kind: `identity_${decision}`,
        title: decision === "accepted" ? "Identity change accepted locally" : "Changed identity remains blocked",
        summary: decision === "accepted"
          ? "Local compatibility was updated after explicit review; no public trust claim was created."
          : "The previous local block remains. No receiver reference or counterparty state changed."
      };
      state.contactsScreen = "outcome";
      render({ focusMain: true });
    } else if (action === "remove") {
      const result = contactsGateway.removeContact({ contactId });
      if (!result.ok) {
        showToast(result.error.message, "alert");
        return;
      }
      state.contactsLastOutcome = {
        kind: "removed",
        title: "Contact removed locally",
        summary: "The local label was removed. Protocol objects, counterparty history, and remote state were not revoked or erased."
      };
      state.contactsSelectedId = null;
      state.contactsScreen = "outcome";
      render({ focusMain: true });
    } else if (["pay", "request", "message", "export"].includes(action)) {
      const result = contactsGateway.prepareAction({ contactId, action });
      if (!result.ok) {
        showToast(result.error.message, "alert");
        return;
      }
      const handoff = result.data;
      if (action === "export") {
        state.contactActionHandoff = handoff;
        state.contactsLastOutcome = {
          kind: "export_prepared",
          title: "Public-material export prepared",
          summary: "The typed Contact identity reference is ready for a future native export review. It was not copied or uploaded."
        };
        state.contactsScreen = "outcome";
        render({ focusMain: true });
        return;
      }
      if (action === "pay") {
        const validation = walletGateway.revalidateExternalReviewHandoff({
          walletId: activeWallet().id,
          handoff
        });
        if (!validation.ok) {
          showToast(validation.error.message, "alert");
          return;
        }
        clearExternalReviewHandoffs();
        state.contactActionHandoff = handoff;
        selectCanonicalRoute(validation.data.target.routeId);
        const draft = resetActiveSendDraft();
        draft.step = 0;
        render({ focusMain: true });
        showToast("Contact Pay action revalidated by Wallet.");
        return;
      }
      clearExternalReviewHandoffs();
      state.contactActionHandoff = handoff;
      selectCanonicalRoute(handoff.target.routeId);
      render({ focusMain: true });
      showToast(`${dappTitleCase(action)} opened as a typed Messenger concept.`);
    }
    return;
  }

  const paletteButton = event.target.closest("[data-palette]");
  if (paletteButton && paletteButton.tagName === "BUTTON") {
    applyPalette(paletteButton.dataset.palette);
    syncConfigDraftFromState();
    applyAppearancePreferences();
    render();
    return;
  }

  const codeThemeButton = event.target.closest("[data-code-theme]");
  if (codeThemeButton && codeThemeButton.tagName === "BUTTON") {
    state.codeTheme = codeThemeButton.dataset.codeTheme;
    syncConfigDraftFromState();
    applyAppearancePreferences();
    render();
    showToast(`${codeThemeOptions.find((theme) => theme.id === state.codeTheme)?.label || "Code"} highlighting applied across the application.`);
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

document.addEventListener("keydown", (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLocaleLowerCase() === "k") {
    event.preventDefault();
    openMenuSearch();
    return;
  }
  if (menuSearchIsOpen()) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeMenuSearch({ restoreFocus: true });
      return;
    }
    if (event.key === "Tab") {
      const focusable = [...menuSearchDialog.querySelectorAll('button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])')]
        .filter((element) => element.getClientRects().length > 0);
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable.at(-1);
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
    return;
  }
  if (event.key === "Escape" && document.querySelector("[data-select-picker].is-open")) {
    event.preventDefault();
    closeSelectPickers({ restoreFocus: true });
    return;
  }
  if (event.key === "Escape" && document.querySelector("[data-language-picker].is-open")) {
    event.preventDefault();
    closeLanguagePickers({ restoreFocus: true });
    return;
  }
  if (event.key === "Escape" && !walletPickerPopup.hidden) {
    event.preventDefault();
    closeDesktopWalletPicker({ restoreFocus: true });
    return;
  }
  if (event.key === "Escape" && !mobilePopupMenu.hidden) {
    event.preventDefault();
    closeMobilePopup({ restoreFocus: true });
    return;
  }
  if (event.key === "Tab" && !mobilePopupMenu.hidden) {
    const focusable = [...mobilePopupMenu.querySelectorAll("button:not([disabled])")];
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable.at(-1);
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
});

window.addEventListener("resize", () => {
  closeLanguagePickers();
  closeSelectPickers();
  closeDesktopWalletPicker();
  const mobileNavigation = isMobileNavigation();
  if (!mobileNavigation) {
    closeMobilePopup();
  }
  if (mobileNavigation === mobileNavigationLayout) return;
  mobileNavigationLayout = mobileNavigation;
  render();
});

window.addEventListener("popstate", (event) => {
  closeMenuSearch();
  if (event.state?.z00zOverlay === "flow-dialog" && event.state.z00zFlow) {
    restoreDialogFromHistory(event.state.z00zFlow);
    return;
  }
  if (dialog.open && (dialogHistoryActive || dialogHistoryClosing)) {
    closeDialog({ fromHistory: true });
    return;
  }
  const requestedRoute = event.state?.z00zRoute || new URLSearchParams(window.location.search).get("route");
  const routeId = requestedRoute === "wallet.staking" ? "wallet.staking.stake" : requestedRoute;
  if (!demoRuntime.PORT_CONTRACT.routes.includes(routeId)) return;
  selectCanonicalRoute(routeId, { pushHistory: false });
  closeMobilePopup();
  render({ focusMain: true });
});

document.addEventListener("submit", (event) => {
  event.preventDefault();
  if (event.target.id === "dapp-action-proposal-form") {
    const descriptor = demoRuntime.dappDescriptor(event.target.dataset.dappId);
    if (!descriptor) {
      showToast("Unknown local dApp descriptor.", "alert");
      return;
    }
    if (!validateDappProposalForm(event.target, descriptor)) return;
    state.dappSelectedId = descriptor.id;
    state.dappLastOutcome = {
      kind: "intent_proposed",
      label: `${descriptor.label} proposal prepared`,
      summary: "The typed proposal is local presentation state only. Wallet has not selected objects, built or signed a package, charged a fee, or submitted anything for settlement.",
      returnRoute: descriptor.routeId,
      descriptorId: descriptor.id
    };
    state.dappScreen = "outcome";
    render({ focusMain: true });
    showToast("Typed proposal prepared for Wallet review.");
  } else if (event.target.id === "dapp-permission-review-form") {
    const acknowledgements = {
      scopeConfirmed: Boolean(event.target.elements.scopeConfirmed.checked),
      reauthAcknowledged: Boolean(event.target.elements.reauthAcknowledged.checked)
    };
    state.dappReviewAcknowledgements = acknowledgements;
    completeDappPermissionReview("accepted", acknowledgements);
  } else if (event.target.id === "explorer-public-search") {
    const query = event.target.elements.publicId.value;
    mergeShellState({ type: "begin_request", requestKey: "telemetry:telemetry.explorer.search" });
    const result = telemetryGateway.searchExplorerPublicId({
      query,
      scenario: state.explorerScenario,
      generation: Number(state.requestGenerations["telemetry:telemetry.explorer.search"] || 0)
    });
    state.explorerSearchResult = result;
    state.explorerSelectedPublicId = result.status === "found" ? result.publicId : null;
    state.explorerQuery = result.status === "found" ? result.publicId : "";
    state.explorerDetailMode = "summary";
    render();
    requestAnimationFrame(() => document.querySelector(result.status === "found" ? "[data-explorer-detail]" : "#explorer-public-id")?.focus?.());
  } else if (event.target.id === "messenger-conversation-search") {
    state.messengerQuery = event.target.elements.query.value.trim();
    render();
    requestAnimationFrame(() => document.querySelector("#messenger-search")?.focus());
  } else if (event.target.id === "contacts-search-form") {
    state.contactsQuery = event.target.elements.query.value.trim();
    state.contactsStatus = event.target.elements.status.value;
    state.contactsSort = event.target.elements.sort.value;
    render();
    requestAnimationFrame(() => document.querySelector("#contacts-query")?.focus());
  } else if (event.target.id === "contact-import-form") {
    const result = contactsGateway.addContact({
      sourceId: state.contactsImportSourceId,
      label: event.target.elements.label.value,
      safeNote: event.target.elements.safeNote.value
    });
    if (!result.ok) {
      state.contactsFormError = result.error.message;
      render();
      requestAnimationFrame(() => document.querySelector("#contact-import-label")?.focus());
      return;
    }
    state.contactsSelectedId = result.data.contact.id;
    state.contactsFormError = null;
    state.contactsLastOutcome = {
      kind: "added",
      title: "Contact saved locally",
      summary: "A private local label and reviewed reference domains were created without a contact upload or public presence lookup."
    };
    state.contactsScreen = "outcome";
    render({ focusMain: true });
  } else if (event.target.id === "contact-edit-form") {
    const result = contactsGateway.editLabel({
      contactId: state.contactsSelectedId,
      label: event.target.elements.label.value,
      safeNote: event.target.elements.safeNote.value
    });
    if (!result.ok) {
      state.contactsFormError = result.error.message;
      render();
      requestAnimationFrame(() => document.querySelector("#contact-edit-label")?.focus());
      return;
    }
    state.contactsFormError = null;
    state.contactsLastOutcome = {
      kind: "edited",
      title: "Local label updated",
      summary: "Only local presentation metadata changed. Receiver material and all remote state remained unchanged."
    };
    state.contactsScreen = "outcome";
    render({ focusMain: true });
  } else if (["wallet-rename-entry", "wallet-seed-reveal-entry", "wallet-public-export-entry", "wallet-key-rotation-entry", "wallet-policy-apply-entry"].includes(event.target.id)) {
    validateWalletSettingsAction(event.target);
  } else if (event.target.id === "wallet-password-change-entry") {
    validateWalletPasswordChange(event.target);
  } else if (event.target.id === "create-voucher-entry") {
    const form = event.target;
    const result = walletGateway.createVoucher({
      walletId: activeWallet().id,
      title: form.elements.title.value,
      amount: form.elements.amount.value,
      expiry: form.elements.expiry.value
    });
    const error = document.querySelector("#voucher-create-error");
    if (!result.ok) {
      error.textContent = result.error.message;
      form.elements.title.focus();
      return;
    }
    const wallet = activeWallet();
    wallet.activities.unshift({ id: `voucher-create-${wallet.activities.length + 1}`, type: "voucher", direction: "neutral", title: `${result.data.voucher.title} created`, detail: "Ready to transfer", amount: result.data.voucher.value, time: "Now", status: "active" });
    closeDialog();
    render({ focusMain: true });
    showToast("Voucher created and available in Send.");
  } else if (event.target.id === "create-permission-entry") {
    const form = event.target;
    const result = walletGateway.createPermission({
      walletId: activeWallet().id,
      title: form.elements.title.value,
      action: form.elements.action.value,
      scope: form.elements.scope.value,
      uses: form.elements.uses.value,
      expiry: form.elements.expiry.value
    });
    const error = document.querySelector("#permission-create-error");
    if (!result.ok) {
      error.textContent = result.error.message;
      form.elements.title.focus();
      return;
    }
    const wallet = activeWallet();
    wallet.activities.unshift({ id: `permission-create-${wallet.activities.length + 1}`, type: "permission", direction: "neutral", title: `${result.data.permission.title} created`, detail: "Held · ready to transfer", amount: "", time: "Now", status: "active" });
    closeDialog();
    render({ focusMain: true });
    showToast("Permission created and available in Send.");
  } else if (event.target.id === "send-entry") {
    validateSend(event.target);
  } else if (event.target.id === "exchange-entry") {
    validateExchange(event.target);
  } else if (event.target.id === "permission-entry") {
    validatePermission(event.target);
  } else if (event.target.id === "create-wallet-entry") {
    const name = event.target.elements.walletLabel;
    const chainId = event.target.elements.chainId;
    const password = event.target.elements.newWalletSecret;
    const confirm = event.target.elements.confirmWalletSecret;
    let valid = true;
    document.querySelector("#create-name-error").textContent = "";
    document.querySelector("#create-chain-error").textContent = "";
    document.querySelector("#create-password-error").textContent = "";
    document.querySelector("#create-confirm-error").textContent = "";
    [name, chainId, password, confirm].forEach((field) => field.removeAttribute("aria-invalid"));
    if (name.value.trim().length < 2) {
      document.querySelector("#create-name-error").textContent = "Enter a recognizable wallet name.";
      name.setAttribute("aria-invalid", "true");
      valid = false;
    }
    if (!walletChainOptions.some(({ id }) => id === chainId.value)) {
      document.querySelector("#create-chain-error").textContent = "Choose a supported wallet chain.";
      chainId.setAttribute("aria-invalid", "true");
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
    state.flow.data.chainId = chainId.value;
    state.flow.step = 1;
    renderDialog();
  } else if (event.target.id === "create-wallet-verify") {
    const verificationIndexes = state.flow.data.verificationIndexes;
    const wordSelectors = [...event.target.querySelectorAll("select[data-seed-index]")];
    const firstIncorrect = wordSelectors.find((select) => select.value !== demoSeedWords[Number(select.dataset.seedIndex)]);
    if (wordSelectors.length !== 4 || firstIncorrect) {
      document.querySelector("#seed-verify-error").textContent = `Choose the words shown at positions ${seedVerificationPositionList(verificationIndexes)}.`;
      firstIncorrect?.focus();
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
    const wallet = addWalletProfile(name.value.trim(), "mainnet", "Scanning");
    state.selectedWalletId = wallet.id;
    state.view = "wallet";
    state.activityFilter = "all";
    state.assetFilter = "all";
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
  if (["send-recipient", "send-amount", "send-memo"].includes(event.target.id)) {
    const draft = activeSendDraft();
    if (event.target.id === "send-recipient") draft.recipient = event.target.value;
    if (event.target.id === "send-amount") draft.amount = event.target.value;
    if (event.target.id === "send-memo") draft.memo = event.target.value;
  } else if (event.target.closest("#exchange-entry")) {
    captureExchangeDraft(event.target.form);
  } else if (event.target.matches("[data-split-amount-index]")) {
    const index = Number(event.target.dataset.splitAmountIndex);
    const mergeSplitState = activeMergeSplitState();
    if (Number.isInteger(index) && index >= 0 && index < mergeSplitState.splitAmounts.length) {
      mergeSplitState.splitAmounts[index] = event.target.value;
      mergeSplitState.preview = null;
      mergeSplitState.error = "";
    }
  } else if (event.target.id === "activity-search") {
    const term = event.target.value.trim().toLowerCase();
    const items = activeWallet().activities.filter((item) => {
      const matchesFilter = matchesActivityFilter(item, state.activityFilter);
      return matchesFilter && `${activityText(item, "title")} ${activityText(item, "detail")} ${item.id}`.toLowerCase().includes(term);
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
  if (event.target === sidebarNavigationScrollRegion) {
    captureNavigationScrollPosition("desktop", event.target.scrollTop);
  } else if (event.target instanceof Element
    && event.target.matches("#mobile-popup-menu .mobile-navigation-scroll-region")) {
    captureNavigationScrollPosition("mobile", event.target.scrollTop);
  }
}, true);

document.addEventListener("change", async (event) => {
  if (event.target.matches("[data-merge-fragment-id]")) {
    const mergeSplitState = activeMergeSplitState();
    const assetId = event.target.dataset.mergeFragmentId;
    const asset = MERGE_SPLIT_ASSET_FIXTURES.find(({ id }) => id === assetId);
    if (asset?.status === "available") {
      if (event.target.checked) {
        mergeSplitState.selectedMergeIds = [
          ...mergeSplitState.selectedMergeIds.filter((id) => {
            const selected = MERGE_SPLIT_ASSET_FIXTURES.find((candidate) => candidate.id === id);
            return selected?.definitionId === asset.definitionId && selected?.serialId === asset.serialId;
          }),
          asset.id
        ].filter((id, index, values) => values.indexOf(id) === index);
      } else {
        mergeSplitState.selectedMergeIds = mergeSplitState.selectedMergeIds.filter((id) => id !== asset.id);
      }
    }
    mergeSplitState.preview = null;
    mergeSplitState.error = "";
    render();
    return;
  }
  if (event.target.matches("[data-split-source]")) {
    const mergeSplitState = activeMergeSplitState();
    const source = MERGE_SPLIT_ASSET_FIXTURES.find(({ id, status }) => id === event.target.value && status === "available");
    if (source) {
      const first = Math.floor(source.amountAtomic / 2);
      mergeSplitState.selectedSplitId = source.id;
      mergeSplitState.splitAmounts = [
        formatAtomicAmount(first, source.decimals),
        formatAtomicAmount(source.amountAtomic - first, source.decimals)
      ];
      mergeSplitState.preview = null;
      mergeSplitState.error = "";
    }
    render();
    requestAnimationFrame(() => document.querySelector("[data-split-source]")?.focus({ preventScroll: true }));
    return;
  }
  if (event.target.matches("[data-split-amount-index]")) {
    const index = Number(event.target.dataset.splitAmountIndex);
    const mergeSplitState = activeMergeSplitState();
    if (Number.isInteger(index) && index >= 0 && index < mergeSplitState.splitAmounts.length) {
      mergeSplitState.splitAmounts[index] = event.target.value;
      mergeSplitState.preview = null;
      mergeSplitState.error = "";
    }
    render();
    requestAnimationFrame(() => document.querySelector(`[data-split-amount-index="${index}"]`)?.focus({ preventScroll: true }));
    return;
  }
  if (event.target.id === "asset-import-file") {
    const file = event.target.files?.[0];
    if (!file) return;
    const walletId = activeWallet().id;
    let result;
    if (file.size > 64 * 1024) {
      result = {
        ok: false,
        error: {
          code: "validation",
          reason: "IMPORT_MALFORMED_JSON",
          message: `Asset package exceeds the 64 KiB public JSON limit (${file.size} bytes).`
        }
      };
    } else {
      try {
        result = walletGateway.inspectAssetPackage({
          walletId,
          fileName: file.name,
          assetData: await file.text()
        });
      } catch {
        result = {
          ok: false,
          error: {
            code: "validation",
            reason: "IMPORT_MALFORMED_JSON",
            message: "The selected asset package could not be read."
          }
        };
      }
    }
    if (activeWallet().id !== walletId) return;
    const importState = resetAssetImportState();
    importState.fileName = file.name;
    importState.fileSize = file.size;
    if (!result.ok) {
      importState.error = result.error;
      render({ focusMain: true });
      return;
    }
    importState.status = "ready";
    importState.reviewToken = result.data.reviewToken;
    importState.preview = result.data.preview;
    render({ focusMain: true });
    requestAnimationFrame(() => document.querySelector('[data-asset-import-action="prepare"]')?.focus({ preventScroll: true }));
    return;
  }
  if (event.target.matches("[data-contact-sort]")) {
    state.contactsSort = event.target.value;
    render();
    requestAnimationFrame(() => document.querySelector("[data-contact-sort]")?.focus());
    return;
  }
  if (event.target.matches("[data-contact-status-filter]")) {
    state.contactsStatus = event.target.value;
    render();
    requestAnimationFrame(() => document.querySelector("[data-contact-status-filter]")?.focus());
    return;
  }
  const watcherControl = event.target.dataset.watcherControl;
  if (watcherControl) {
    if (watcherControl === "source") state.watcherSourceId = event.target.value;
    if (watcherControl === "scenario") state.watcherScenario = event.target.value;
    if (watcherControl === "severity") state.watcherSeverityFilter = event.target.value;
    if (watcherControl === "kind") state.watcherKindFilter = event.target.value;
    state.watcherExportEnvelope = null;
    mergeShellState({ type: "begin_request", requestKey: `telemetry:${state.activeRoute}` });
    render();
    requestAnimationFrame(() => document.querySelector(`[data-watcher-control="${CSS.escape(watcherControl)}"]`)?.focus());
    return;
  }
  const explorerControl = event.target.dataset.explorerControl;
  if (explorerControl) {
    if (explorerControl === "scenario") state.explorerScenario = event.target.value;
    if (explorerControl === "kind") state.explorerEvidenceKindFilter = event.target.value;
    state.explorerSearchResult = null;
    state.explorerSelectedPublicId = null;
    state.explorerDetailMode = "summary";
    mergeShellState({ type: "begin_request", requestKey: `telemetry:${state.activeRoute}` });
    render();
    requestAnimationFrame(() => document.querySelector(`[data-explorer-control="${CSS.escape(explorerControl)}"]`)?.focus());
    return;
  }
  if (event.target.id === "send-item") {
    const form = event.target.form;
    const draft = activeSendDraft();
    draft.recipient = form.elements.recipient.value;
    draft.memo = form.elements.memo.value;
    draft.itemKey = event.target.value;
    draft.amount = "";
    render();
    requestAnimationFrame(() => document.querySelector("#send-item")?.focus());
    return;
  }
  if (event.target.closest("#exchange-entry")) {
    const draft = captureExchangeDraft(event.target.form);
    if (["exchange-source", "exchange-order-type"].includes(event.target.id)) {
      render();
      requestAnimationFrame(() => document.querySelector(`#${event.target.id}`)?.focus());
    } else {
      state.exchangeDrafts[activeWallet().id] = draft;
    }
    return;
  }
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
    if (configControl === "language") {
      selectLanguage(event.target.value);
      return;
    }
    if (configControl === "regional-locale") state.regionalLocale = event.target.value;
    if (configControl === "valuation-currency") state.valuationCurrency = event.target.value;
    if (configControl === "time-zone") state.timeZone = event.target.value;
    if (configControl === "network-units") state.networkUnits = event.target.value;
    if (configControl === "vibrate") state.vibrate = event.target.value;
    if (configControl === "ringtone") state.ringtone = event.target.value;
    if (configControl === "palette") applyPalette(event.target.value);
    if (configControl === "text-scale") state.textScale = event.target.value;
    if (configControl === "code-theme") state.codeTheme = event.target.value;
    if (configControl === "lock-after") {
      state.autoLockMinutes = event.target.value;
      activeWalletPreferences().lockAfterMinutes = event.target.value;
    }
    if (configControl === "default-fee") activeWalletPreferences().defaultFee = event.target.value.trim();
    syncConfigDraftFromState();
    applyAppearancePreferences();
    render();
    return;
  }
});

document.addEventListener("click", (event) => {
  const selectPickerOption = event.target.closest("[data-select-picker-index]");
  if (selectPickerOption) {
    event.preventDefault();
    const picker = selectPickerOption.closest("[data-select-picker]");
    const select = picker?.querySelector("select");
    const optionIndex = Number(selectPickerOption.dataset.selectPickerIndex);
    const option = Number.isInteger(optionIndex) ? select?.options[optionIndex] : null;
    if (!select || !option || option.disabled) return;
    closeSelectPicker(picker);
    select.selectedIndex = optionIndex;
    select.dispatchEvent(new Event("input", { bubbles: true }));
    select.dispatchEvent(new Event("change", { bubbles: true }));
    return;
  }

  const selectPickerTrigger = event.target.closest("[data-select-picker-trigger]");
  if (selectPickerTrigger) {
    event.preventDefault();
    const picker = selectPickerTrigger.closest("[data-select-picker]");
    if (picker?.classList.contains("is-open")) closeSelectPicker(picker);
    else if (picker) openSelectPicker(picker);
    return;
  }

  if (!event.target.closest("[data-select-picker]")) closeSelectPickers();

  const languageOption = event.target.closest("[data-language-picker-option]");
  if (languageOption) {
    event.preventDefault();
    selectLanguage(languageOption.dataset.languagePickerOption);
    return;
  }

  const languageTrigger = event.target.closest("[data-language-picker-trigger]");
  if (languageTrigger) {
    event.preventDefault();
    const picker = languageTrigger.closest("[data-language-picker]");
    if (picker?.classList.contains("is-open")) closeLanguagePicker(picker);
    else if (picker) openLanguagePicker(picker);
    return;
  }

  if (!event.target.closest("[data-language-picker]")) closeLanguagePickers();

  const toggle = event.target.closest("[data-toggle-password]");
  if (!toggle) return;
  const input = document.querySelector("#unlock-password");
  const visible = input.classList.toggle("is-revealed");
  toggle.setAttribute("aria-label", visible ? "Hide password" : "Show password");
  toggle.querySelector("use").setAttribute("href", visible ? "#i-eye-off" : "#i-eye");
});

dialog.addEventListener("click", (event) => {
  if (event.target !== dialog) return;
  const rect = dialog.getBoundingClientRect();
  const inside = event.clientX >= rect.left && event.clientX <= rect.right && event.clientY >= rect.top && event.clientY <= rect.bottom;
  if (!inside) closeDialog();
});

dialog.addEventListener("cancel", (event) => {
  event.preventDefault();
  closeDialog();
});

dialog.addEventListener("close", () => {
  state.flow = null;
  const trigger = state.lastDialogTrigger;
  state.lastDialogTrigger = null;
  if (trigger?.isConnected) trigger.focus();
});

syncConfigDraftFromState();
const requestedInitialRoute = new URLSearchParams(window.location.search).get("route");
const initialRouteCandidate = requestedInitialRoute || restoredNavigationSnapshot?.activeRoute;
const initialCanonicalRoute = initialRouteCandidate === "wallet.staking"
  ? "wallet.staking.stake"
  : initialRouteCandidate;
if (demoRuntime.PORT_CONTRACT.routes.includes(initialCanonicalRoute)) {
  selectCanonicalRoute(initialCanonicalRoute, { pushHistory: false });
}
if (restoredNavigationSnapshot) {
  state.expandedBranchIds = [...restoredNavigationSnapshot.expandedBranchIds];
  state.drawerOpen = Boolean(restoredNavigationSnapshot.drawerOpen && isMobileNavigation());
}
navigationSessionReady = true;
applyAppearancePreferences();
render();
persistNavigationState();
if (state.drawerOpen) {
  requestAnimationFrame(() => openMobilePopup(mobileMenuButton));
}
