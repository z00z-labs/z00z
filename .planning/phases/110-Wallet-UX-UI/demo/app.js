"use strict";

const main = document.querySelector("#main-content");
const pageTitle = document.querySelector("#page-title");
const pageContext = document.querySelector("#page-context");
const dialog = document.querySelector("#flow-dialog");
const dialogContent = document.querySelector("#dialog-content");
const appShell = document.querySelector("#app-shell");
const lockScreen = document.querySelector("#lock-screen");
const demoParams = new URLSearchParams(window.location.search);
const requestedView = ["home", "wallet", "activity", "settings"].includes(demoParams.get("view")) ? demoParams.get("view") : "home";
const requestedWalletSection = ["assets", "claims", "permissions"].includes(demoParams.get("wallet")) ? demoParams.get("wallet") : "assets";
const requestedSettingsSection = ["general", "security", "network", "policies", "backup", "appearance", "advanced"].includes(demoParams.get("settings")) ? demoParams.get("settings") : "general";
const requestedNetworkSection = ["overview", "reticulum", "onionnet", "carriers"].includes(demoParams.get("network")) ? demoParams.get("network") : "overview";

const state = {
  view: requestedView,
  balanceHidden: false,
  activityFilter: "all",
  walletSection: requestedWalletSection,
  settingsSection: requestedSettingsSection,
  networkSection: requestedNetworkSection,
  theme: "dark",
  locked: false,
  flow: null,
  lastDialogTrigger: null,
  activities: [
    { id: "tx-7f31", type: "money", direction: "out", title: "Payment to Mira", detail: "Sent · waiting to settle", amount: "− 240.00 Z00Z", time: "2 min", status: "settling" },
    { id: "claim-014", type: "claim", direction: "in", title: "Travel refund", detail: "Claimed · waiting to settle", amount: "+ 86.00 Z00Z", time: "18 min", status: "settling" },
    { id: "tx-7e88", type: "money", direction: "in", title: "Received from Niko", detail: "Settled", amount: "+ 1,200.00 Z00Z", time: "Yesterday", status: "settled" },
    { id: "budget-221", type: "budget", direction: "neutral", title: "Design services budget", detail: "Used 120 of 500 Z00Z", amount: "380.00 left", time: "Yesterday", status: "active" },
    { id: "tx-7d12", type: "money", direction: "out", title: "Payment to Coffee Lab", detail: "Settled", amount: "− 18.50 Z00Z", time: "12 Jul", status: "settled" },
    { id: "security-4", type: "security", direction: "neutral", title: "Local backup created", detail: "Integrity check passed", amount: "", time: "10 Jul", status: "settled" }
  ]
};

const headings = {
  home: ["Home", "Your private money at a glance"],
  wallet: ["Wallet", "Assets, claims, and permissions stay distinct"],
  activity: ["Activity", "Assets, claims, permissions, policies, and security events"],
  settings: ["Settings", "Common controls and fully synchronized advanced configuration"]
};

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

function quickAction(type, label, helper, iconName) {
  return `
    <button class="quick-action" type="button" data-open-flow="${type}">
      <span class="quick-action-icon">${icon(iconName)}</span>
      <span><strong>${label}</strong><small>${helper}</small></span>
    </button>`;
}

function homeView() {
  return `
    <div class="view-enter">
      <section class="dashboard-grid" aria-label="Wallet overview">
        <article class="card balance-card">
          <div class="balance-card-top">
            <span class="balance-label">${icon("shield")} Available privately</span>
            <span class="asset-chip">Z00Z</span>
          </div>
          <p class="balance-amount">${sensitive("12,480.75")} <span class="mono">Z00Z</span></p>
          <p class="balance-pending"><strong>${sensitive("+ 960.00")}</strong> receiving · <strong>${sensitive("− 240.00")}</strong> sending</p>
        </article>

        <article class="card privacy-card">
          <div class="privacy-card-header">
            <span class="shield-mark">${icon("shield")}</span>
            <span class="status-badge is-settled">Healthy</span>
          </div>
          <h2>Onion route verified</h2>
          <p>OnionNet protects the route. Reticulum carries it on Main.</p>
          <div class="privacy-lines">
            <div class="privacy-line"><span>Privacy</span><strong>OnionNet · 3 hops</strong></div>
            <div class="privacy-line"><span>Carrier</span><strong>Reticulum</strong></div>
            <div class="privacy-line"><span>Wallet scan</span><strong>Up to date</strong></div>
            <div class="privacy-line"><span>Last checked</span><strong>Just now</strong></div>
          </div>
        </article>
      </section>

      <section class="quick-section" aria-labelledby="quick-title">
        <div class="section-heading">
          <div><h2 id="quick-title">What would you like to do?</h2><p>Private actions with safe defaults</p></div>
        </div>
        <div class="quick-grid">
          ${quickAction("pay", "Pay", "Send private money", "send")}
          ${quickAction("receive", "Receive", "Create a private request", "receive")}
          ${quickAction("claim", "Claim", "Review something offered", "claim")}
          ${quickAction("budget", "Give permission", "Start with a safe budget", "budget")}
        </div>
      </section>

      <section class="home-lower">
        <article class="card panel" aria-labelledby="attention-title">
          <div class="section-heading">
            <div><h2 id="attention-title">Needs your attention</h2><p>Two items</p></div>
            <button class="section-link" type="button" data-wallet-section="claims">Open claims ${icon("chevron")}</button>
          </div>
          <div class="attention-list">
            <button class="attention-item" type="button" data-open-flow="claim">
              <span class="list-icon is-claim">${icon("claim")}</span>
              <span class="list-copy"><strong>Travel refund</strong><small>Verified offer from Northwind Travel</small></span>
              <span class="list-meta"><strong>86.00 Z00Z</strong><small>Ends in 2 days</small></span>
            </button>
            <button class="attention-item" type="button" data-open-flow="budget-detail">
              <span class="list-icon is-warning">${icon("alert")}</span>
              <span class="list-copy"><strong>Design services permission</strong><small>Budget recipe · 76% remaining</small></span>
              <span class="list-meta"><strong>380.00 left</strong><small>Ends 31 Jul</small></span>
            </button>
          </div>
        </article>

        <article class="card panel" aria-labelledby="recent-title">
          <div class="section-heading">
            <div><h2 id="recent-title">Recent activity</h2><p>Latest wallet events</p></div>
            <button class="section-link" type="button" data-view="activity">View all ${icon("chevron")}</button>
          </div>
          <div class="activity-list">
            ${activityRows(state.activities.slice(0, 3), true)}
          </div>
        </article>
      </section>
    </div>`;
}

function moneyView() {
  return `
    <div class="view-enter">
      <div class="page-intro">
        <div><p class="eyebrow">Cash only</p><h2>Your private money</h2><p>Only spendable cash is counted here. Claims and delegated limits remain separate.</p></div>
        <button class="button button-primary" type="button" data-open-flow="pay">${icon("send")} Pay</button>
      </div>
      <section class="money-summary" aria-label="Money totals">
        <article class="card metric-card"><span>Available</span><strong>${sensitive("12,480.75 Z00Z")}</strong><small>Ready to use</small></article>
        <article class="card metric-card"><span>Receiving</span><strong>${sensitive("960.00")}</strong><small>Waiting to settle</small></article>
        <article class="card metric-card"><span>Sending</span><strong>${sensitive("240.00")}</strong><small>Waiting to settle</small></article>
      </section>
      <div class="section-heading"><div><h2>Assets</h2><p>Verified cash projections from this wallet</p></div></div>
      <section class="asset-list" aria-label="Cash assets">
        <article class="card asset-row">
          <span class="asset-logo" aria-hidden="true">Z</span>
          <div class="asset-info"><strong>Z00Z</strong><small class="trust-label">${icon("shield")} Native · verified</small></div>
          <div class="asset-number"><strong>${sensitive("12,480.75")}</strong><small>Available</small></div>
          <div class="asset-number"><strong>${sensitive("720.00")}</strong><small>Net settling</small></div>
          <div class="asset-actions">
            <button class="button" type="button" data-open-flow="receive" aria-label="Receive Z00Z">${icon("receive")}<span>Receive</span></button>
            <button class="button button-primary" type="button" data-open-flow="pay" aria-label="Pay Z00Z">${icon("send")}<span>Pay</span></button>
          </div>
        </article>
      </section>
      <div class="notice">${icon("shield")} Claims, permissions, quarantined objects, and experimental compatibility assets are intentionally excluded from your spendable total.</div>
    </div>`;
}

const walletTabs = [
  ["assets", "Assets", "Spendable and owned value"],
  ["claims", "Claims", "Voucher outcomes"],
  ["permissions", "Permissions", "Bounded rights"]
];

function walletTabBar() {
  return `<nav class="workspace-tabs" role="tablist" aria-label="Wallet object families">${walletTabs.map(([key, label, helper]) => `
    <button class="workspace-tab${state.walletSection === key ? " is-active" : ""}" type="button" role="tab" aria-selected="${state.walletSection === key}" data-wallet-section="${key}">
      <strong>${label}</strong><small>${helper}</small>${key === "claims" ? '<span class="tab-count">1</span>' : ""}
    </button>`).join("")}</nav>`;
}

function claimsPanel() {
  return `
    <div class="page-intro compact-intro">
      <div><p class="eyebrow">Voucher family</p><h2>Claims</h2><p>Conditional value stays outside Available until its outcome settles.</p></div>
    </div>
    <section class="card action-panel" role="tabpanel">
      <div class="action-panel-top"><div class="action-title"><span class="list-icon is-claim">${icon("claim")}</span><div><h2>Ready for your decision</h2><p>Backing and restrictions are checked before any action</p></div></div><span class="status-badge is-ready">1 ready</span></div>
      <div class="claim-list">
        <button class="claim-row" type="button" data-open-flow="claim"><span class="list-icon is-claim">${icon("claim")}</span><span class="list-copy"><strong>Travel refund</strong><small>Northwind Travel · cash-backed · one-time · refund allowed</small></span><span class="list-meta"><strong>86.00 Z00Z</strong><small class="status-badge is-ready">Ready to claim</small></span></button>
        <button class="claim-row" type="button" data-open-flow="claim-settled"><span class="list-icon">${icon("check")}</span><span class="list-copy"><strong>Event deposit return</strong><small>Riverside Events · redeemed and settled 12 Jul</small></span><span class="list-meta"><strong>150.00 Z00Z</strong><small class="status-badge is-settled">Settled</small></span></button>
      </div>
    </section>
    <div class="notice">${icon("shield")} Imported vouchers with unknown policy, invalid signatures, or unsupported schema go to Quarantine and never enter Available.</div>`;
}

function permissionsPanel() {
  return `
    <div class="page-intro compact-intro">
      <div><p class="eyebrow">Right family</p><h2>Permissions</h2><p>Zero-value authority with explicit action, scope, uses, expiry, and delegation rules.</p></div>
      <button class="button button-primary" type="button" data-open-flow="budget">${icon("budget")} Give permission</button>
    </div>
    <section class="card action-panel" role="tabpanel">
      <div class="action-panel-top"><div class="action-title"><span class="list-icon is-warning">${icon("budget")}</span><div><h2>Active permissions</h2><p>Budget is one safe recipe, not every permission</p></div></div><span class="status-badge is-active">2 active</span></div>
      <div class="budget-list">
        <button class="budget-row" type="button" data-open-flow="budget-detail"><span class="list-icon is-warning">${icon("budget")}</span><span class="list-copy"><strong>Design services</strong><small>Budget recipe · pay Studio North · ends 31 Jul</small><span class="budget-progress"><span class="progress-track"><span class="progress-bar" style="width:24%"></span></span></span></span><span class="list-meta"><strong>380.00 left</strong><small class="status-badge is-active">Active</small></span></button>
        <div class="budget-row"><span class="list-icon">${icon("shield")}</span><span class="list-copy"><strong>Verify delivery receipt</strong><small>Scoped action · receipts.example · 2 of 5 uses remain · cannot delegate</small></span><span class="list-meta"><strong>2 uses</strong><small class="status-badge is-active">Active</small></span></div>
      </div>
    </section>
    <div class="notice">${icon("spark")} A delegated permission can only become narrower. The wallet never creates authority broader than the right you hold.</div>`;
}

function walletView() {
  const panel = state.walletSection === "assets" ? moneyView() : state.walletSection === "claims" ? claimsPanel() : permissionsPanel();
  return `<div class="view-enter">${walletTabBar()}<div class="workspace-panel">${panel}</div></div>`;
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
    return `<div class="empty-state"><span class="list-icon">${icon("search")}</span><h3>No matching activity</h3><p>Try another filter or search term.</p></div>`;
  }

  return items.map((item) => {
    const iconName = item.type === "claim" ? "claim" : item.type === "budget" ? "budget" : item.type === "security" ? "backup" : item.direction === "in" ? "receive" : "send";
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

function activityView() {
  const visible = state.activityFilter === "all"
    ? state.activities
    : state.activityFilter === "attention"
      ? state.activities.filter((item) => item.status === "attention" || item.status === "settling")
      : state.activities.filter((item) => item.type === state.activityFilter);

  const filters = [
    ["all", "All"], ["money", "Assets"], ["claim", "Claims"], ["budget", "Permissions"], ["attention", "Needs attention"]
  ].map(([value, label]) => `<button class="filter-chip${state.activityFilter === value ? " is-active" : ""}" type="button" data-filter="${value}">${label}</button>`).join("");

  return `
    <div class="view-enter">
      <div class="page-intro"><div><p class="eyebrow">Honest settlement</p><h2>Everything that changed</h2><p>Submission and final settlement are shown as different states. Open an item for its receipt and technical timeline.</p></div></div>
      <div class="filter-bar" aria-label="Activity filters">
        ${filters}
        <label class="search-wrap"><span class="sr-only">Search activity</span>${icon("search")}<input id="activity-search" type="search" placeholder="Search activity" autocomplete="off"></label>
      </div>
      <section class="card activity-panel" id="activity-results" aria-label="Activity results">
        ${activityRows(visible)}
      </section>
    </div>`;
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

function settingsMenu() {
  return Object.entries(settingsMeta).map(([key, [label, helper, iconName]]) => `
    <button class="workspace-tab${state.settingsSection === key ? " is-active" : ""}" type="button" role="tab" aria-selected="${state.settingsSection === key}" title="${helper}" data-settings-section="${key}">
      ${icon(iconName)}<strong>${label}</strong>
    </button>`).join("");
}

function networkTabs() {
  return `<nav class="subtabs" role="tablist" aria-label="Network settings">${[
    ["overview", "Overview"], ["reticulum", "Reticulum"], ["onionnet", "OnionNet"], ["carriers", "Carriers"]
  ].map(([key, label]) => `<button type="button" role="tab" aria-selected="${state.networkSection === key}" class="${state.networkSection === key ? "is-active" : ""}" data-network-section="${key}">${label}</button>`).join("")}</nav>`;
}

function networkDetail() {
  if (state.networkSection === "reticulum") return `
    <div class="connection-options">
      <div class="connection-option"><span class="health-orb is-good"></span><span><strong>Reticulum service</strong><small>Connected · direct underlay · target simulation</small></span><span class="status-badge is-settled">Healthy</span></div>
      <div class="connection-option"><span class="list-icon">${icon("network")}</span><span><strong>Interfaces</strong><small>Auto · TCP client + local mesh discovery</small></span><button class="button" type="button" data-demo-action="config-stage">Configure</button></div>
      <div class="connection-option"><span class="list-icon">${icon("shield")}</span><span><strong>Network identity</strong><small class="mono">RNS 6A3E…91B2 · independent from wallet seed</small></span><span class="status-badge is-active">Separate</span></div>
    </div><div class="notice">${icon("settings")} Raw Reticulum interface definitions live in Advanced YAML. Service/runtime changes may require restart.</div>`;

  if (state.networkSection === "onionnet") return `
    <div class="connection-options">
      <div class="connection-option"><span class="health-orb is-good"></span><span><strong>Privacy route</strong><small>Verified · 3 hops · epoch 1842</small></span><span class="status-badge is-settled">Standard floor</span></div>
      <div class="connection-option"><span class="list-icon">${icon("shield")}</span><span><strong>Membership & replay checks</strong><small>Healthy · fixed packet geometry active</small></span><span class="status-badge is-settled">Verified</span></div>
      <div class="connection-option"><span class="list-icon">${icon("activity")}</span><span><strong>Route age</strong><small>12 minutes · rebuilt automatically by policy</small></span><button class="button" type="button" data-demo-action="rebuild-route">Rebuild</button></div>
    </div><div class="notice">${icon("shield")} This reports concrete route properties. It does not claim that the user is “anonymous” or “untraceable.”</div>`;

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
      <div class="settings-heading"><div><p class="eyebrow">Effective configuration</p><h2>General</h2><p>Common controls write validated patches to the same YAML-backed configuration.</p></div><span class="status-badge is-settled">Synced</span></div>
      <div class="config-status-grid">
        <div><span>Source</span><strong class="mono">wallet_config.yaml</strong></div><div><span>Schema</span><strong>v2 target</strong></div><div><span>Revision</span><strong class="mono">8f31c2</strong></div><div><span>Last valid load</span><strong>Just now</strong></div>
      </div>
      <div class="setting-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Wallet name</strong><small>Everyday · source: YAML</small></span><input class="short-input" value="Everyday" aria-label="Wallet name"></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Default fee</strong><small>Explained in reviews · source: Default</small></span><select aria-label="Default fee"><option>Safe default · 1,000</option></select></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Configuration file</strong><small>External editors are supported; invalid changes keep the last known good settings</small></span><button class="button" type="button" data-settings-section="advanced">Open YAML</button></div>
      </div>`;
  }

  if (state.settingsSection === "security") {
    return `
      <h2>Security</h2><p>Keep private material out of sight and end sessions automatically.</p>
      <div class="setting-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Lock wallet</strong><small>Clear session data and hide all wallet content</small></span><button class="button" type="button" data-demo-action="lock">${icon("lock")} Lock now</button></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Auto-lock</strong><small>After inactivity on this device</small></span><select aria-label="Auto-lock duration"><option>5 minutes</option><option>15 minutes</option><option>30 minutes</option></select></div>
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
      ${networkTabs()}${networkDetail()}`;
  }

  if (state.settingsSection === "policies") {
    return `
      <div class="settings-heading"><div><p class="eyebrow">Effective restrictions</p><h2>Safety & policy profiles</h2><p>Profiles can narrow behavior. They cannot change protocol rules or expand your authority.</p></div><button class="button button-primary" type="button" data-demo-action="load-policy">${icon("backup")} Load profile</button></div>
      <div class="policy-stack" aria-label="Policy precedence">
        <div class="policy-layer is-locked"><span>1</span><div><strong>Protocol rules</strong><small>Native cash conservation · immutable in wallet</small></div><span class="status-badge">Locked</span></div>
        <div class="policy-layer"><span>2</span><div><strong>Organization</strong><small>No managed profile · signed profiles only</small></div><button class="button" type="button" data-demo-action="load-policy">Load</button></div>
        <div class="policy-layer is-active"><span>3</span><div><strong>Personal Safe · v1.4</strong><small>Max payment 2,500 · daily 5,000 · confirmation required</small></div><span class="status-badge is-settled">Applied</span></div>
        <div class="policy-layer"><span>4</span><div><strong>Per-action attenuation</strong><small>May only make the current action narrower</small></div><span class="status-badge">As needed</span></div>
      </div>
      <button class="why-blocked" type="button" data-demo-action="why-blocked">${icon("alert")}<span><strong>Why a 3,200 Z00Z payment would be blocked</strong><small>Personal Safe → maximum transaction is 2,500 Z00Z</small></span>${icon("chevron")}</button>
      <div class="notice">${icon("shield")} A loaded profile is not proof of legal compliance. Invalid signatures, expired schemas, and ambiguous conflicts fail closed and go to quarantine.</div>`;
  }

  if (state.settingsSection === "appearance") {
    return `
      <div class="settings-heading"><div><p class="eyebrow">Protected semantics</p><h2>Appearance</h2><p>Personalize brand surfaces while safety, privacy, and environment colors stay unambiguous.</p></div><span class="config-source">Source · YAML</span></div>
      <div class="setting-group">
        <div class="setting-line"><span class="setting-line-copy"><strong>Theme</strong><small>System follows the operating system</small></span><div class="segmented" aria-label="Theme"><button type="button" data-theme="system" class="${state.theme === "system" ? "is-active" : ""}>System</button><button type="button" data-theme="dark" class="${state.theme === "dark" ? "is-active" : ""}">${icon("moon")} Dark</button><button type="button" data-theme="light" class="${state.theme === "light" ? "is-active" : ""}">${icon("sun")} Light</button></div></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Accent</strong><small>Decorative and primary-action color · semantic colors protected</small></span><div class="accent-options" aria-label="Accent color"><button class="accent-swatch is-active" style="--swatch:#D9A441" aria-label="Z00Z Gold" title="Z00Z Gold"></button><button class="accent-swatch" style="--swatch:#37B6D7" aria-label="Private Cyan" title="Private Cyan"></button><button class="accent-swatch" style="--swatch:#8C98A7" aria-label="Neutral" title="Neutral"></button><button class="accent-swatch accent-custom" aria-label="Custom validated accent" title="Custom validated accent">+</button></div></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Text scale</strong><small>Interface reflows without hiding content</small></span><select aria-label="Text scale"><option>100%</option><option>110%</option><option>125%</option></select></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Reduced motion</strong><small>The wallet already follows your operating system preference</small></span><button class="toggle" type="button" aria-pressed="false" aria-label="Use reduced motion" data-demo-action="motion"></button></div>
        <div class="setting-line"><span class="setting-line-copy"><strong>Compact desktop lists</strong><small>Touch targets stay at least 44 pixels</small></span><button class="toggle" type="button" aria-pressed="false" aria-label="Use compact desktop lists" data-demo-action="compact"></button></div>
      </div>`;
  }

  return `
    <div class="settings-heading"><div><p class="eyebrow">Advanced configuration</p><h2>YAML & diagnostics</h2><p>UI controls and YAML edit one effective configuration. Secrets are never stored here.</p></div><span class="status-badge is-settled">Synced · 8f31c2</span></div>
    <nav class="subtabs config-tabs" role="tablist" aria-label="Configuration views"><button class="is-active" role="tab" aria-selected="true">YAML</button><button role="tab" aria-selected="false">Form</button><button role="tab" aria-selected="false">Diff</button></nav>
    <div class="yaml-toolbar"><span><strong class="mono">wallet_config.yaml</strong><small>Schema v2 target · last valid just now</small></span><div><button class="button" type="button" data-demo-action="config-validate">Validate</button><button class="button button-primary" type="button" data-demo-action="config-apply">Apply</button></div></div>
    <pre class="yaml-editor" aria-label="Read-only target YAML configuration preview"><code><span class="yaml-key">schema_version:</span> 2
<span class="yaml-key">wallet:</span>
  <span class="yaml-key">appearance:</span>
    <span class="yaml-key">theme:</span> system        <span class="yaml-comment"># UI ↔ YAML</span>
    <span class="yaml-key">accent:</span> z00z-gold
  <span class="yaml-key">network:</span>
    <span class="yaml-key">mode:</span> private
    <span class="yaml-key">privacy_overlay:</span> onionnet
    <span class="yaml-key">carriers:</span> [reticulum, quic_tls]
  <span class="yaml-key">policy_profiles:</span>
    <span class="yaml-key">user:</span> personal-safe-v1</code></pre>
    <div class="config-foot"><span>${icon("shield")} Unknown keys and comments are preserved</span><span>${icon("activity")} External edits watched</span><span>${icon("backup")} Last known good retained</span></div>
    <div class="capability-note">${icon("alert")} <span><strong>Target configuration service</strong><small>Current runtime reads YAML but has no write/watch/revision RPC. These controls must stay disabled in production until that service exists.</small></span></div>
    <div class="setting-group"><div class="setting-line"><span class="setting-line-copy"><strong>Expert details</strong><small>Show identifiers, receipts, and lifecycle events</small></span><button class="toggle" type="button" aria-pressed="false" aria-label="Show expert details" data-demo-action="expert"></button></div><div class="setting-line"><span class="setting-line-copy"><strong>Sanitized diagnostics</strong><small>RPC, configuration, route, and synchronization events</small></span><button class="button" type="button" data-demo-action="diagnostics">Open</button></div></div>`;
}

function settingsView() {
  return `
    <div class="view-enter">
      <div class="page-intro"><div><p class="eyebrow">Your wallet, your device</p><h2>Wallet settings</h2><p>Security actions require the right level of review and authentication.</p></div></div>
      <section class="settings-workspace">
        <nav class="workspace-tabs settings-menu" role="tablist" aria-label="Settings sections">${settingsMenu()}</nav>
        <article class="card settings-detail">${settingsDetail()}</article>
      </section>
    </div>`;
}

function render(options = {}) {
  const [title, context] = headings[state.view];
  pageTitle.textContent = title;
  pageContext.textContent = context;

  document.querySelectorAll("[data-view]").forEach((button) => {
    const active = button.dataset.view === state.view;
    button.classList.toggle("is-active", active);
    if (button.closest("nav")) {
      active ? button.setAttribute("aria-current", "page") : button.removeAttribute("aria-current");
    }
  });

  main.innerHTML = {
    home: homeView,
    wallet: walletView,
    activity: activityView,
    settings: settingsView
  }[state.view]();

  syncBalanceButtons();
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
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Pay privately",
      subtitle: "Recipient and amount",
      steps: 3,
      activeStep: 0,
      body: `
        <form class="form-grid" id="pay-entry" novalidate>
          <div class="field-group"><label class="field-label" for="pay-recipient">Recipient or payment request</label><input id="pay-recipient" name="recipient" value="${escapeHtml(data.recipient)}" placeholder="Paste or scan a private request" autocomplete="off" aria-describedby="pay-recipient-hint pay-recipient-error" required><p class="field-hint" id="pay-recipient-hint">The wallet validates the receiver and network before review.</p><p class="field-error" id="pay-recipient-error"></p></div>
          <div class="field-group"><label class="field-label" for="pay-amount">Amount</label><div class="input-with-affix"><input id="pay-amount" name="amount" type="number" min="0.01" max="12480.75" step="0.01" inputmode="decimal" value="${escapeHtml(data.amount)}" placeholder="0.00" aria-describedby="pay-amount-hint pay-amount-error" required><span class="input-affix">Z00Z</span></div><p class="field-hint" id="pay-amount-hint">Available: 12,480.75 Z00Z · fee included</p><p class="field-error" id="pay-amount-error"></p></div>
          <div class="field-group"><label class="field-label" for="pay-memo">Private note <span class="muted">(optional)</span></label><input id="pay-memo" name="memo" value="${escapeHtml(data.memo)}" maxlength="80" placeholder="What is this for?"></div>
        </form>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="pay-entry">Review payment ${icon("chevron")}</button>`
    });
  }

  if (state.flow.step === 1) {
    return dialogFrame({
      title: "Review payment",
      subtitle: "Check before authorizing",
      steps: 3,
      activeStep: 1,
      body: `
        <div class="review-card review-hero"><span class="list-icon">${icon("send")}</span><strong>${escapeHtml(data.amount)} Z00Z</strong><span>to ${escapeHtml(data.recipientLabel)}</span></div>
        <div class="review-card">
          <div class="summary-row"><span>Recipient</span><strong>${escapeHtml(data.recipientLabel)} · <span class="mono">7D3B…9A40</span></strong></div>
          <div class="summary-row"><span>From</span><strong>Everyday wallet</strong></div>
          <div class="summary-row"><span>Fee</span><strong>Included</strong></div>
          <div class="summary-row"><span>Privacy route</span><strong>OnionNet · verified · 3 hops</strong></div>
          <div class="summary-row"><span>Carrier</span><strong>Reticulum</strong></div>
          <div class="summary-row"><span>Network</span><strong><span class="environment-tag is-main">MAIN</span></strong></div>
          ${data.memo ? `<div class="summary-row"><span>Note</span><strong>${escapeHtml(data.memo)}</strong></div>` : ""}
        </div>
        <div class="confirmation-note">${icon("shield")} Sending authorizes this payment once. It will appear as settling until the wallet confirms final state.</div>`,
      footer: `<button class="button" type="button" data-dialog-action="pay-back">Back</button><button class="button button-primary" type="button" data-dialog-action="pay-submit">Send payment</button>`
    });
  }

  return dialogFrame({
    title: "Payment sent",
    subtitle: "Waiting for final settlement",
    steps: 3,
    activeStep: 2,
    body: `
      <div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Sent · settling</h3><p>Your payment to ${escapeHtml(data.recipientLabel)} was accepted for processing. It is not final yet.</p><div class="receipt-ref mono">Reference TX-8A42 · idempotency protected</div></div>
      <div class="review-card"><div class="summary-row"><span>Amount</span><strong>${escapeHtml(data.amount)} Z00Z</strong></div><div class="summary-row"><span>Fee</span><strong>Included</strong></div><div class="summary-row"><span>Next update</span><strong>Automatic</strong></div></div>`,
    footer: `<button class="button" type="button" data-dialog-action="view-activity">View activity</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function receiveDialog() {
  const data = state.flow.data;
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Receive privately",
      subtitle: "Create a payment request",
      steps: 2,
      activeStep: 0,
      body: `
        <form class="form-grid" id="receive-entry" novalidate>
          <div class="field-group"><label class="field-label" for="receive-amount">Requested amount <span class="muted">(optional)</span></label><div class="input-with-affix"><input id="receive-amount" name="amount" type="number" min="0.01" step="0.01" inputmode="decimal" value="${escapeHtml(data.amount)}" placeholder="Any amount"><span class="input-affix">Z00Z</span></div></div>
          <div class="field-group"><label class="field-label" for="receive-note">What is it for? <span class="muted">(optional)</span></label><input id="receive-note" name="note" maxlength="80" value="${escapeHtml(data.note)}" placeholder="Dinner, invoice, refund…"></div>
          <div class="field-group"><label class="field-label" for="receive-expiry">Request expires</label><select id="receive-expiry" name="expiry"><option>In 24 hours</option><option>In 7 days</option><option>Never</option></select><p class="field-hint">Expiry limits how long this request should be trusted.</p></div>
        </form>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="receive-entry">Create request</button>`
    });
  }

  return dialogFrame({
    title: "Private payment request",
    subtitle: "Share the QR or request text",
    steps: 2,
    activeStep: 1,
    body: `
      <div class="qr-layout"><div class="mock-qr" aria-label="Mock payment request QR code">${qrCells()}</div><div><p class="eyebrow">Ready to share</p><h3>${data.amount ? `${escapeHtml(data.amount)} Z00Z` : "Any Z00Z amount"}</h3><p class="muted">Everyday wallet · expires in 24 hours</p><code class="request-code">z00z:pay:rcv_8f2a71c0?amount=${escapeHtml(data.amount || "any")}</code><button class="button button-full" type="button" data-demo-action="copy-request">${icon("copy")} Copy request</button></div></div>
      <div class="notice">${icon("shield")} The payer sees “Everyday wallet” and an abbreviated receiver. Incoming money appears as settling before it becomes available.</div>`,
    footer: `<button class="button" type="button" data-demo-action="share-request">Share</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function claimDialog(settled = false) {
  if (settled) {
    return dialogFrame({
      title: "Event deposit return",
      subtitle: "Claim details",
      body: `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Settled</h3><p>150.00 Z00Z was added to your available private money on 12 Jul 2026.</p></div><div class="review-card"><div class="summary-row"><span>Issuer</span><strong>Riverside Events · verified</strong></div><div class="summary-row"><span>Receipt</span><strong class="mono">RCPT-14B9…C201</strong></div></div><details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>Object: voucher_04e9…af31</span><span>Lifecycle: accepted → redeemed → confirmed</span></div></details>`,
      footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
    });
  }

  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Review claim",
      subtitle: "Offer verified by your wallet",
      steps: 2,
      activeStep: 0,
      body: `
        <div class="review-card review-hero"><span class="list-icon is-claim">${icon("claim")}</span><strong>86.00 Z00Z</strong><span>Travel refund</span></div>
        <div class="review-card"><div class="summary-row"><span>From</span><strong class="trust-label">${icon("shield")} Northwind Travel</strong></div><div class="summary-row"><span>You receive</span><strong>86.00 Z00Z</strong></div><div class="summary-row"><span>Ends</span><strong>21 Jul 2026 · 18:00</strong></div><div class="summary-row"><span>Terms</span><strong>One-time refund · no payment required</strong></div></div>
        <div class="confirmation-note">${icon("shield")} Accepting reveals only the information required for this refund. It does not grant future spending permission.</div>
        <details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>Preview: voucher_883c…204a</span><span>Package verified · Main chain</span></div></details>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-action="claim-reject">Reject</button><button class="button button-primary" type="button" data-dialog-action="claim-accept">Accept claim</button>`
    });
  }

  return dialogFrame({
    title: "Claim accepted",
    subtitle: "Waiting for final settlement",
    steps: 2,
    activeStep: 1,
    body: `<div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Claimed · settling</h3><p>The refund was accepted and is waiting for final settlement. It is not included in your available balance yet.</p><div class="receipt-ref mono">Claim CLM-883C · result tracked automatically</div></div>`,
    footer: `<button class="button" type="button" data-dialog-action="view-activity">View activity</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function budgetDialog() {
  const data = state.flow.data;
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Create a budget",
      subtitle: "A bounded permission you can revoke",
      steps: 3,
      activeStep: 0,
      body: `
        <form class="form-grid" id="budget-entry" novalidate>
          <div class="field-group"><label class="field-label" for="budget-recipe">Budget recipe</label><select id="budget-recipe" name="recipe"><option>Service allowance</option><option>Recurring allowance</option><option>Single-use permission</option></select><p class="field-hint">Recipes prevent accidentally broad permissions.</p></div>
          <div class="field-group"><label class="field-label" for="budget-delegate">Service or person</label><input id="budget-delegate" name="delegate" value="${escapeHtml(data.delegate)}" placeholder="Verified request or known identity" required aria-describedby="budget-delegate-error"><p class="field-error" id="budget-delegate-error"></p></div>
          <div class="field-group"><label class="field-label" for="budget-limit">Maximum total</label><div class="input-with-affix"><input id="budget-limit" name="limit" type="number" min="1" max="12480.75" step="0.01" inputmode="decimal" value="${escapeHtml(data.limit)}" placeholder="0.00" required aria-describedby="budget-limit-error"><span class="input-affix">Z00Z</span></div><p class="field-error" id="budget-limit-error"></p></div>
          <div class="field-group"><label class="field-label" for="budget-purpose">Allowed purpose</label><select id="budget-purpose" name="purpose"><option>Design services</option><option>Compute services</option><option>Travel services</option><option>Single purchase</option></select></div>
          <div class="field-group"><label class="field-label" for="budget-expiry">Ends</label><input id="budget-expiry" name="expiry" type="date" value="2026-08-19" min="2026-07-20" required></div>
        </form>`,
      footer: `<button class="button button-quiet" type="button" data-dialog-close>Cancel</button><button class="button button-primary" type="submit" form="budget-entry">Review budget ${icon("chevron")}</button>`
    });
  }

  if (state.flow.step === 1) {
    return dialogFrame({
      title: "Review budget",
      subtitle: "Check what is and is not allowed",
      steps: 3,
      activeStep: 1,
      body: `
        <div class="review-card review-hero"><span class="list-icon is-warning">${icon("budget")}</span><strong>${escapeHtml(data.limit)} Z00Z</strong><span>maximum for ${escapeHtml(data.delegate)}</span></div>
        <div class="review-card"><div class="summary-row"><span>Can use</span><strong>Up to ${escapeHtml(data.limit)} Z00Z total</strong></div><div class="summary-row"><span>Only for</span><strong>${escapeHtml(data.purpose)}</strong></div><div class="summary-row"><span>Ends</span><strong>${escapeHtml(data.expiryLabel)}</strong></div><div class="summary-row"><span>Cannot</span><strong>Transfer permission or exceed limit</strong></div><div class="summary-row"><span>Your control</span><strong>Revoke from this wallet</strong></div></div>
        <div class="confirmation-note">${icon("alert")} Already-authorized or in-flight use may still settle after revocation. Final behavior follows the live protocol state.</div>`,
      footer: `<button class="button" type="button" data-dialog-action="budget-back">Back</button><button class="button button-primary" type="button" data-dialog-action="budget-submit">Create budget</button>`
    });
  }

  return dialogFrame({
    title: "Budget created",
    subtitle: "Bounded permission is active",
    steps: 3,
    activeStep: 2,
    body: `<div class="result-state"><span class="result-icon">${icon("check")}</span><h3>Budget active</h3><p>${escapeHtml(data.delegate)} can use up to ${escapeHtml(data.limit)} Z00Z for ${escapeHtml(data.purpose).toLowerCase()} until ${escapeHtml(data.expiryLabel)}.</p><div class="receipt-ref mono">Budget BGT-40A1 · revocable from this wallet</div></div>`,
    footer: `<button class="button" type="button" data-dialog-action="go-actions">View budgets</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function budgetDetailDialog() {
  return dialogFrame({
    title: "Design services",
    subtitle: "Active budget for Studio North",
    body: `
      <div class="review-card review-hero"><span class="list-icon is-warning">${icon("budget")}</span><strong>380.00 Z00Z</strong><span>remaining of 500.00 Z00Z</span><span class="budget-progress"><span class="progress-track"><span class="progress-bar" style="width:24%"></span></span></span></div>
      <div class="review-card"><div class="summary-row"><span>Allowed purpose</span><strong>Design services</strong></div><div class="summary-row"><span>Used</span><strong>120.00 Z00Z</strong></div><div class="summary-row"><span>Ends</span><strong>31 Jul 2026</strong></div><div class="summary-row"><span>Status</span><strong><span class="status-badge is-active">Active</span></strong></div></div>
      <details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>Right: right_54ac…1f88</span><span>Scope: provider/studio-north/design</span><span>Lifecycle: delegated → active</span></div></details>`,
    footer: `<button class="button button-danger" type="button" data-dialog-action="budget-revoke">Revoke budget</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function activityDialog(item) {
  const lifecycle = item.status === "settling" ? "created → submitted → admitted" : "created → submitted → admitted → confirmed";
  return dialogFrame({
    title: item.title,
    subtitle: "Activity details",
    body: `
      <div class="review-card review-hero"><span class="list-icon ${item.direction === "in" ? "is-claim" : ""}">${icon(item.direction === "in" ? "receive" : item.direction === "out" ? "send" : "activity")}</span><strong>${escapeHtml(item.amount || statusText(item.status))}</strong><span>${escapeHtml(item.detail)}</span></div>
      <div class="review-card"><div class="summary-row"><span>Status</span><strong><span class="status-badge is-${escapeHtml(item.status)}">${statusText(item.status)}</span></strong></div><div class="summary-row"><span>When</span><strong>${escapeHtml(item.time)}</strong></div><div class="summary-row"><span>Fee</span><strong>${item.type === "money" ? "Included" : "Not applicable"}</strong></div><div class="summary-row"><span>Privacy</span><strong>OnionNet verified</strong></div><div class="summary-row"><span>Carrier & chain</span><strong>Reticulum · Main</strong></div></div>
      <details class="technical"><summary>Technical details</summary><div class="technical-content mono"><span>ID: ${escapeHtml(item.id)}-b4c9…8e20</span><span>Lifecycle: ${lifecycle}</span><span>Receipt: public_4a92…c71e</span></div></details>`,
    footer: `<button class="button" type="button" data-demo-action="copy-receipt">${icon("copy")} Copy receipt</button><button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

function connectionDialog() {
  return dialogFrame({
    title: "Network & privacy",
    subtitle: "Overlay, carrier, and chain are separate",
    body: `
      <p class="eyebrow">Privacy mode · target simulation</p>
      <div class="connection-options"><div class="connection-option"><span class="health-orb is-good"></span><span><strong>OnionNet</strong><small>Route verified · standard floor · 3 hops</small></span><span class="status-badge is-settled">Privacy overlay</span></div><div class="connection-option"><span class="health-orb is-good"></span><span><strong>Reticulum</strong><small>Primary resilient carrier · direct underlay</small></span><span class="status-badge is-active">Carrier</span></div><div class="connection-option"><span class="health-orb"></span><span><strong>Tor</strong><small>Optional compatibility carrier · disabled</small></span><span class="status-badge">Off</span></div></div>
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
    body: `<div class="attention-list"><button class="attention-item" type="button" data-dialog-action="notification-claim"><span class="list-icon is-claim">${icon("claim")}</span><span class="list-copy"><strong>Travel refund expires soon</strong><small>Review 86.00 Z00Z from Northwind Travel</small></span>${icon("chevron")}</button><div class="attention-item"><span class="list-icon">${icon("backup")}</span><span class="list-copy"><strong>Backup verified</strong><small>Your 10 Jul local backup passed integrity checks</small></span><span class="status-badge is-settled">Done</span></div></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-close>Done</button>`
  });
}

const demoSeedWords = [
  "canvas", "orbit", "maple", "velvet", "harbor", "copper", "quiet", "meadow",
  "lamp", "river", "winter", "piano", "forest", "amber", "window", "salt",
  "comet", "paper", "garden", "silver", "cloud", "stone", "echo", "north"
];

function walletsDialog() {
  return dialogFrame({
    title: "Your wallets",
    subtitle: "Local profiles on this device",
    body: `
      <div class="wallet-list">
        <button class="wallet-choice is-current" type="button" data-dialog-action="select-wallet">
          <span class="wallet-avatar" aria-hidden="true">E</span><span><strong>Everyday</strong><small class="mono">8F2A…71C0 · Main</small></span><span class="status-badge is-active">Open</span>
        </button>
        <button class="wallet-choice" type="button" data-dialog-action="select-wallet">
          <span class="wallet-avatar" aria-hidden="true">S</span><span><strong>Savings</strong><small class="mono">0B44…8EE1 · Main</small></span><span class="status-badge">Locked</span>
        </button>
      </div>
      <div class="notice">${icon("shield")} Wallet profiles are local. Switching never sends a seed or password to another service.</div>`,
    footer: `<button class="button" type="button" data-dialog-action="start-recover">Recover wallet</button><button class="button button-primary" type="button" data-dialog-action="start-create">Create wallet</button>`
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
  if (state.flow.step === 0) {
    return dialogFrame({
      title: "Recover wallet",
      subtitle: "Enter the same 24 English words twice",
      steps: 2,
      activeStep: 0,
      body: `
        <div class="confirmation-note">${icon("shield")} Recovery is validated locally. Never enter your words into a website or support chat.</div>
        <form class="form-grid" id="recover-wallet-entry" novalidate>
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
    body: `<div class="result-state"><span class="result-icon is-settling">${icon("activity")}</span><h3>Recovery complete · scanning</h3><p>Your keys are available locally. Money and activity will appear as the wallet scan catches up.</p></div><div class="review-card"><div class="summary-row"><span>Wallet scan</span><strong>42%</strong></div><div class="progress-track"><div class="progress-bar" style="width:42%"></div></div><div class="summary-row"><span>Safe to close</span><strong>Yes · resumes automatically</strong></div></div>`,
    footer: `<button class="button button-primary" type="button" data-dialog-action="recover-finish">Open wallet</button>`
  });
}

function renderDialog() {
  if (!state.flow) return;
  const type = state.flow.type;
  const content = type === "pay" ? payDialog()
    : type === "receive" ? receiveDialog()
    : type === "claim" ? claimDialog(false)
    : type === "claim-settled" ? claimDialog(true)
    : type === "budget" ? budgetDialog()
    : type === "budget-detail" ? budgetDetailDialog()
    : type === "activity" ? activityDialog(state.flow.data.item)
    : type === "connection" ? connectionDialog()
    : type === "wallets" ? walletsDialog()
    : type === "create-wallet" ? createWalletDialog()
    : type === "recover-wallet" ? recoverWalletDialog()
    : notificationsDialog();
  dialogContent.innerHTML = content;
}

function defaultFlowData(type) {
  if (type === "pay") return { recipient: "", recipientLabel: "", amount: "", memo: "" };
  if (type === "receive") return { amount: "", note: "" };
  if (type === "budget") return { delegate: "", limit: "", purpose: "Design services", expiry: "2026-08-19", expiryLabel: "19 Aug 2026" };
  if (type === "create-wallet") return { name: "" };
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
  if (!Number.isFinite(number) || number <= 0 || number > 12480.75) {
    document.querySelector("#pay-amount-error").textContent = "Enter an amount between 0.01 and 12,480.75 Z00Z.";
    amount.setAttribute("aria-invalid", "true");
    valid = false;
  }
  if (!valid) {
    form.querySelector('[aria-invalid="true"]')?.focus();
    return;
  }

  state.flow.data = {
    recipient: recipient.value.trim(),
    recipientLabel: recipient.value.trim().startsWith("z00z:") ? "Verified payment request" : recipient.value.trim(),
    amount: number.toFixed(2),
    memo: form.elements.memo.value.trim()
  };
  state.flow.step = 1;
  renderDialog();
}

function validateBudget(form) {
  const delegate = form.elements.delegate;
  const limit = form.elements.limit;
  let valid = true;
  document.querySelector("#budget-delegate-error").textContent = "";
  document.querySelector("#budget-limit-error").textContent = "";
  delegate.removeAttribute("aria-invalid");
  limit.removeAttribute("aria-invalid");

  if (delegate.value.trim().length < 3) {
    document.querySelector("#budget-delegate-error").textContent = "Choose a verified service or known person.";
    delegate.setAttribute("aria-invalid", "true");
    valid = false;
  }
  const number = Number(limit.value);
  if (!Number.isFinite(number) || number < 1 || number > 12480.75) {
    document.querySelector("#budget-limit-error").textContent = "Set a maximum between 1.00 and 12,480.75 Z00Z.";
    limit.setAttribute("aria-invalid", "true");
    valid = false;
  }
  if (!valid) {
    form.querySelector('[aria-invalid="true"]')?.focus();
    return;
  }

  const expiry = new Date(`${form.elements.expiry.value}T12:00:00`);
  state.flow.data = {
    delegate: delegate.value.trim(),
    limit: number.toFixed(2),
    purpose: form.elements.purpose.value,
    expiry: form.elements.expiry.value,
    expiryLabel: new Intl.DateTimeFormat("en", { day: "2-digit", month: "short", year: "numeric" }).format(expiry)
  };
  state.flow.step = 1;
  renderDialog();
}

function setButtonLoading(button, label) {
  button.disabled = true;
  button.dataset.original = button.innerHTML;
  button.textContent = label;
}

function completePay() {
  const data = state.flow.data;
  state.activities.unshift({ id: "tx-8a42", type: "money", direction: "out", title: `Payment to ${data.recipientLabel}`, detail: "Sent · waiting to settle", amount: `− ${data.amount} Z00Z`, time: "Now", status: "settling" });
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
  } else if (action === "budget-back") {
    state.flow.step = 0;
    renderDialog();
  } else if (action === "budget-submit") {
    setButtonLoading(button, "Creating…");
    window.setTimeout(() => { state.flow.step = 2; renderDialog(); }, 650);
  } else if (action === "claim-accept") {
    setButtonLoading(button, "Accepting…");
    window.setTimeout(() => { state.flow.step = 1; renderDialog(); }, 600);
  } else if (action === "claim-reject") {
    showToast("Reject is a separate destructive confirmation in production.", "alert");
  } else if (action === "budget-revoke") {
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
  } else if (action === "notification-claim") {
    closeDialog();
    window.setTimeout(() => openFlow("claim", button), 0);
  } else if (action === "select-wallet") {
    closeDialog();
    showToast("Everyday wallet remains open in this concept.");
  } else if (action === "start-create") {
    state.flow = { type: "create-wallet", step: 0, data: defaultFlowData("create-wallet") };
    renderDialog();
  } else if (action === "start-recover") {
    state.flow = { type: "recover-wallet", step: 0, data: {} };
    renderDialog();
  } else if (["create-back-wallets", "recover-back-wallets"].includes(action)) {
    state.flow = { type: "wallets", step: 0, data: {} };
    renderDialog();
  } else if (action === "create-seed-saved") {
    state.flow.step = 2;
    renderDialog();
  } else if (action === "create-seed-back") {
    state.flow.step = 1;
    renderDialog();
  } else if (action === "create-finish" || action === "recover-finish") {
    closeDialog();
    if (state.locked) {
      state.locked = false;
      lockScreen.hidden = true;
      appShell.hidden = false;
      appShell.inert = false;
    }
    render();
    showToast(action === "create-finish" ? "New wallet opened in concept mode." : "Recovered wallet opened; scan continues.");
  }
}

function handleDemoAction(action, button) {
  if (action === "toggle-balance") {
    state.balanceHidden = !state.balanceHidden;
    render();
    showToast(state.balanceHidden ? "Sensitive amounts hidden." : "Sensitive amounts visible.");
  } else if (action === "lock") {
    closeDialog();
    state.locked = true;
    appShell.hidden = true;
    appShell.inert = true;
    lockScreen.hidden = false;
    document.querySelector("#unlock-password").value = "";
    document.querySelector("#unlock-error").textContent = "";
    document.querySelector("#unlock-password").focus();
  } else if (action === "switch-wallet") {
    openFlow("wallets", button);
  } else if (action === "notifications") {
    openFlow("notifications", button);
  } else if (["copy-request", "copy-receipt"].includes(action)) {
    showToast(action === "copy-request" ? "Payment request copied." : "Public receipt copied.");
  } else if (action === "share-request") {
    showToast("Native share sheet would open on this device.");
  } else if (action === "seed-warning") {
    showToast("Seed reveal requires re-authentication and a private display check.", "alert");
  } else if (action === "key-rotation") {
    showToast("Key rotation requires re-authentication and a fresh backup.", "alert");
  } else if (action === "backup") {
    showToast("Backup destination selection would open next.");
  } else if (action === "restore") {
    showToast("Restore validates integrity before any replacement.", "alert");
  } else if (["motion", "compact", "expert"].includes(action)) {
    const pressed = button.getAttribute("aria-pressed") === "true";
    button.setAttribute("aria-pressed", String(!pressed));
    showToast(`${action[0].toUpperCase()}${action.slice(1)} preference ${pressed ? "disabled" : "enabled"}.`);
  } else if (action === "diagnostics") {
    showToast("Diagnostics would open sanitized RPC and route records.");
  } else if (action === "load-policy") {
    showToast("Profile would be parsed, signature-checked, capability-checked, and previewed before Apply.");
  } else if (action === "why-blocked") {
    showToast("Blocked by Personal Safe v1.4: maximum transaction is 2,500 Z00Z.", "alert");
  } else if (action === "config-validate") {
    showToast("Target document is valid; no secret paths detected.");
  } else if (action === "config-apply") {
    showToast("Target service would apply atomically after revision check.");
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
    state.view = viewButton.dataset.view;
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

  const flowButton = event.target.closest("[data-open-flow]");
  if (flowButton) {
    openFlow(flowButton.dataset.openFlow, flowButton);
    return;
  }

  const activityButton = event.target.closest("[data-open-activity]");
  if (activityButton) {
    const item = state.activities.find((entry) => entry.id === activityButton.dataset.openActivity);
    if (item) openFlow("activity", activityButton, { item });
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
    state.settingsSection = settingButton.dataset.settingsSection;
    render();
    return;
  }

  const networkButton = event.target.closest("[data-network-section]");
  if (networkButton) {
    state.networkSection = networkButton.dataset.networkSection;
    render();
    return;
  }

  const themeButton = event.target.closest("[data-theme]");
  if (themeButton && themeButton.tagName === "BUTTON") {
    state.theme = themeButton.dataset.theme;
    const effectiveTheme = state.theme === "system" ? (window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark") : state.theme;
    document.documentElement.dataset.theme = effectiveTheme;
    document.querySelector('meta[name="theme-color"]').content = effectiveTheme === "dark" ? "#081019" : "#F4F7FA";
    render();
    showToast(`${state.theme === "system" ? "System" : state.theme === "dark" ? "Dark" : "Light"} theme applied and ready to sync.`);
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
  if (event.target.id === "pay-entry") {
    validatePay(event.target);
  } else if (event.target.id === "receive-entry") {
    state.flow.data = {
      amount: event.target.elements.amount.value ? Number(event.target.elements.amount.value).toFixed(2) : "",
      note: event.target.elements.note.value.trim()
    };
    state.flow.step = 1;
    renderDialog();
  } else if (event.target.id === "budget-entry") {
    validateBudget(event.target);
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
  } else if (event.target.id === "recover-wallet-entry") {
    const phraseA = event.target.elements.phraseA.value.trim().split(/\s+/).filter(Boolean);
    const phraseB = event.target.elements.phraseB.value.trim().split(/\s+/).filter(Boolean);
    const error = document.querySelector("#recover-phrase-error");
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
    const items = state.activities.filter((item) => {
      const matchesFilter = state.activityFilter === "all" || item.type === state.activityFilter || (state.activityFilter === "attention" && ["settling", "attention"].includes(item.status));
      return matchesFilter && `${item.title} ${item.detail} ${item.id}`.toLowerCase().includes(term);
    });
    document.querySelector("#activity-results").innerHTML = activityRows(items);
  } else if (event.target.classList.contains("seed-entry")) {
    const count = event.target.value.trim() ? event.target.value.trim().split(/\s+/).length : 0;
    const hint = event.target.closest(".field-group")?.querySelector(".field-hint");
    if (hint) hint.textContent = `${count} of 24 words`;
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

render();
