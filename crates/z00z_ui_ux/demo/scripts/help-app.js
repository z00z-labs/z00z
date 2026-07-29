"use strict";

((root) => {
  const registry = root.Z00ZHelpRegistry;
  const i18n = root.Z00ZI18n;
  const demo = root.Z00ZDemo;
  if (!registry || !i18n || !demo?.navigationChildren || !demo.createNavigationSession) {
    throw new Error("Standalone Help dependencies are missing.");
  }

  const topicIds = new Set(registry.topics().map(({ id }) => id));
  const tree = document.querySelector("#help-tree");
  const navigationTerminal = document.querySelector("#help-navigation-terminal");
  const searchTrigger = document.querySelector("#help-search-trigger");
  const searchTriggerLabel = document.querySelector("#help-search-trigger-label");
  const searchShortcut = document.querySelector("#help-search-shortcut");
  const searchOverlay = document.querySelector("#help-search-overlay");
  const searchBackdrop = document.querySelector("#help-search-backdrop");
  const searchDialog = document.querySelector("#help-search-dialog");
  const searchDialogTitle = document.querySelector("#help-search-dialog-title");
  const searchInput = document.querySelector("#help-search");
  const searchClose = document.querySelector("#help-search-close");
  const searchResults = document.querySelector("#help-search-results");
  const searchStatus = document.querySelector("#help-search-status");
  const article = document.querySelector("#help-document");
  const main = document.querySelector("#help-main");
  const sidebar = document.querySelector("#help-sidebar");
  const navigationScrollRegion = sidebar.querySelector(".help-navigation-scroll-region");
  const siteHeader = document.querySelector(".help-site-header");
  const mobileTopbarContext = document.querySelector("#help-mobile-topbar-context");
  const menuButton = document.querySelector("#help-menu-button");
  const backdrop = document.querySelector("#help-sidebar-backdrop");
  const homeLink = document.querySelector("#help-home-link");
  const languagePicker = document.querySelector("#help-language-picker");
  let language = "en";
  let activeTopicId = registry.globalTopic();
  let sectionTarget = "";
  let searchQuery = "";
  let mobileNavigationLayout = root.matchMedia("(max-width: 768px)").matches;
  const navigationSession = demo.createNavigationSession("help");
  const restoredNavigationSnapshot = navigationSession.read();
  const navigationScrollPositions = {
    desktop: restoredNavigationSnapshot?.scrollPositions.desktop || 0,
    mobile: restoredNavigationSnapshot?.scrollPositions.mobile || 0
  };
  const expandedBranchIds = new Set(restoredNavigationSnapshot?.expandedBranchIds || ["wallet"]);
  let navigationSessionReady = false;
  const terminalNodeIds = new Set(["settings", "help", "about", "logout"]);
  const excludedNodeIds = new Set(["help", "about", "logout"]);
  const mobileDrawerSwipe = {
    pointerId: null,
    source: "",
    startX: 0,
    startY: 0,
    direction: ""
  };
  const mobileDrawerSwipeEdge = 48;
  const mobileDrawerSwipeDistance = 56;

  root.name = "z00z-help";

  const escapeHtml = (value) => String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
  const icon = (name, extraClass = "") => `<svg class="icon${extraClass ? ` ${extraClass}` : ""}" aria-hidden="true"><use href="#i-${name}"/></svg>`;
  const navigationIcon = (node, extraClass = "") => `<svg class="icon navigation-tree-icon help-tree-navigation-icon${extraClass ? ` ${extraClass}` : ""}" aria-hidden="true" data-help-navigation-icon="${escapeHtml(node.id)}"><use href="#i-${node.iconId}"/></svg>`;
  const translate = (key, values) => i18n.translate(language, key, values);

  function persistNavigationState() {
    if (!navigationSessionReady) return;
    navigationSession.write({
      activeRoute: registry.topic(activeTopicId)?.routeId || null,
      expandedBranchIds: [...expandedBranchIds],
      scrollPositions: navigationScrollPositions,
      drawerOpen: Boolean(mobileNavigationLayout && !sidebar.hidden && sidebar.classList.contains("is-open"))
    });
  }

  function captureNavigationScrollPosition() {
    const layout = mobileNavigationLayout ? "mobile" : "desktop";
    navigationScrollPositions[layout] = Math.max(0, Math.round(navigationScrollRegion.scrollTop));
    persistNavigationState();
  }

  function restoreNavigationScrollPosition(layout) {
    navigationScrollRegion.scrollTop = navigationScrollPositions[layout] || 0;
  }

  function languageMetadata() {
    return i18n.languages().find(({ id }) => id === language) || i18n.languages()[0];
  }

  function languagePickerMarkup() {
    const selected = languageMetadata();
    const label = translate("app.language");
    return `<div class="language-picker help-language-picker" data-help-language-picker>
      <button class="language-picker-trigger" type="button" data-help-language-trigger aria-label="${escapeHtml(label)}" aria-haspopup="listbox" aria-expanded="false" aria-controls="help-language-options">
        <span class="language-picker-value"><span aria-hidden="true">${escapeHtml(selected.flag)}</span><span>${escapeHtml(selected.nativeName)}</span></span>
        ${icon("chevron")}
      </button>
      <div class="language-picker-menu" id="help-language-options" data-help-language-options role="listbox" aria-label="${escapeHtml(label)}" hidden>
        ${i18n.languages().map(({ id, nativeName, flag }) => `<button class="language-picker-option${id === language ? " is-selected" : ""}" type="button" role="option" aria-selected="${id === language}" tabindex="${id === language ? "0" : "-1"}" data-help-language-option="${escapeHtml(id)}"><span aria-hidden="true">${escapeHtml(flag)}</span><span>${escapeHtml(nativeName)}</span>${id === language ? icon("check") : ""}</button>`).join("")}
      </div>
    </div>`;
  }

  function readRoute({ preserveExpansion = false } = {}) {
    const parameters = new URLSearchParams(root.location.search);
    language = i18n.resolveLanguage(parameters.get("lang"));
    const requestedTopicId = parameters.get("topic");
    const restoredTopicId = restoredNavigationSnapshot?.activeRoute
      ? registry.topics().find(({ routeId }) => routeId === restoredNavigationSnapshot.activeRoute)?.id
      : null;
    activeTopicId = topicIds.has(requestedTopicId)
      ? requestedTopicId
      : topicIds.has(restoredTopicId) ? restoredTopicId : registry.globalTopic();
    sectionTarget = /^[a-z][a-z0-9-]*$/u.test(parameters.get("section") || "") ? parameters.get("section") : "";
    document.documentElement.dataset.palette = parameters.get("palette") === "z00z-corporate" ? "z00z-corporate" : "z00z-default";
    if (!preserveExpansion) expandActiveBranch();
  }

  function routeUrl(topicId, target = "") {
    const url = new URL(root.location.href);
    url.hash = "";
    url.searchParams.set("topic", topicId);
    url.searchParams.set("lang", language);
    if (target) url.searchParams.set("section", target);
    else url.searchParams.delete("section");
    return url;
  }

  function nodeLabel(node) {
    return translate(node.labelKey);
  }

  function topicIdForNode(node) {
    if (node.target.kind === "help") return registry.globalTopic();
    return node.helpTopicId;
  }

  function activeRouteNode() {
    const record = registry.topic(activeTopicId);
    return record?.routeId ? demo.navigationNodeForRoute(record.routeId) : null;
  }

  function hasActiveDescendant(node) {
    const routeNode = activeRouteNode();
    return demo.ancestorContainerIdsForNode(routeNode?.id || "").includes(node.id);
  }

  function isActiveNavigationNode(node) {
    const routeNode = activeRouteNode();
    if (!routeNode || !["route", "workspace"].includes(node.target.kind)) return false;
    return node.target.routeId === routeNode.target.routeId
      || (node.target.kind === "workspace" && hasActiveDescendant(node));
  }

  function navigationNodeMarkup(node, { prefix, depth = 0, terminal = false } = {}) {
    const label = escapeHtml(nodeLabel(node));
    const depthClass = `is-depth-${depth}`;
    const sectionBreakClass = node.sectionBreakBefore ? " navigation-tree-section-break" : "";
    const activeDescendant = hasActiveDescendant(node);
    if (node.target.kind === "branch") {
      const expanded = expandedBranchIds.has(node.id);
      const controlId = `${prefix}-${node.id.replaceAll(".", "-")}-toggle`;
      const panelId = `${prefix}-${node.id.replaceAll(".", "-")}-children`;
      return `<section class="navigation-tree-branch ${depthClass}${sectionBreakClass}${expanded ? " is-expanded" : ""}${activeDescendant ? " has-active-descendant" : ""}" data-help-navigation-node="${escapeHtml(node.id)}">
        <button id="${controlId}" class="navigation-tree-item navigation-tree-branch-toggle" type="button" data-help-navigation-branch="${escapeHtml(node.id)}" aria-expanded="${expanded}" aria-controls="${panelId}">
          ${navigationIcon(node)}
          <span class="navigation-tree-label">${label}</span>
          ${icon("chevron", "navigation-tree-chevron")}
        </button>
        <div id="${panelId}" class="navigation-tree-children" role="group" aria-labelledby="${controlId}"${expanded ? "" : " hidden"}>
          ${demo.navigationChildren(node.id).map((child) => navigationNodeMarkup(child, { prefix, depth: depth + 1 })).join("")}
        </div>
      </section>`;
    }
    if (node.target.kind === "group") {
      const groupId = `${prefix}-${node.id.replaceAll(".", "-")}-group`;
      return `<section class="navigation-tree-group ${depthClass}${sectionBreakClass}${activeDescendant ? " has-active-descendant" : ""}" data-help-navigation-node="${escapeHtml(node.id)}" aria-labelledby="${groupId}">
        <p id="${groupId}" class="navigation-tree-group-label">
          ${navigationIcon(node)}
          <span class="navigation-tree-label">${label}</span>
        </p>
        <div class="navigation-tree-group-children" role="group" aria-labelledby="${groupId}">
          ${demo.navigationChildren(node.id).map((child) => navigationNodeMarkup(child, { prefix, depth: depth + 1 })).join("")}
        </div>
      </section>`;
    }
    const topicId = topicIdForNode(node);
    if (!topicId || !registry.hasTopic(topicId)) return "";
    const active = isActiveNavigationNode(node);
    return `<a class="navigation-tree-item navigation-tree-leaf${terminal ? " navigation-tree-terminal" : ""} ${depthClass}${sectionBreakClass}${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(topicId).href)}" data-help-navigation-node="${escapeHtml(node.id)}" data-help-topic-link="${escapeHtml(topicId)}"${node.target.kind === "workspace" ? ` data-navigation-workspace="${escapeHtml(node.id)}"` : ""}${active ? ' aria-current="page"' : ""}>
      ${navigationIcon(node)}
      <span class="navigation-tree-label">${label}</span>
    </a>`;
  }

  function expandActiveBranch() {
    const routeNode = activeRouteNode();
    demo.ancestorBranchIdsForNode(routeNode?.id || "").forEach((nodeId) => expandedBranchIds.add(nodeId));
  }

  function renderTree() {
    const rootNodes = demo.navigationChildren().filter((node) => !excludedNodeIds.has(node.id));
    tree.innerHTML = rootNodes
      .filter((node) => !terminalNodeIds.has(node.id))
      .map((node) => navigationNodeMarkup(node, { prefix: "help-navigation" }))
      .join("");
    navigationTerminal.innerHTML = rootNodes
      .filter((node) => terminalNodeIds.has(node.id))
      .map((node) => navigationNodeMarkup(node, { prefix: "help-terminal", terminal: true }))
      .join("")
      + `<p class="app-version">Version ${escapeHtml(demo.APP_VERSION)}</p>`;
  }

  function activeWorkspaceNode() {
    let node = activeRouteNode();
    while (node) {
      if (node.target.kind === "workspace") return node;
      node = node.parentId ? demo.navigationNode(node.parentId) : null;
    }
    return null;
  }

  function workspaceDestinations(workspace, { includeHidden = false } = {}) {
    return demo.workspaceLocalDestinations(workspace.id, { includeHidden })
      .map(({ nodeId, routeId, labelKey, iconId }) => {
        const node = demo.navigationNode(nodeId) || demo.navigationNodeForRoute(routeId);
        const topicId = node ? topicIdForNode(node) : "";
        return topicId && registry.hasTopic(topicId) ? { iconId, labelKey, routeId, topicId } : null;
      })
      .filter(Boolean);
  }

  function walletAssetsContext(workspace) {
    const sections = [
      { id: "assets", topicId: "wallet.assets", routeId: "wallet.assets", labelKey: "assets.sectionAssets", iconId: "assets" },
      { id: "vouchers", topicId: "wallet.vouchers", routeId: "wallet.vouchers", labelKey: "assets.sectionVouchers", iconId: "voucher-list" },
      { id: "permissions", topicId: "wallet.permissions", routeId: "wallet.permissions", labelKey: "assets.sectionPermissions", iconId: "permission-list" }
    ];
    const destinations = sections.filter(({ topicId }) => registry.hasTopic(topicId));
    if (!destinations.length) return null;
    const navigation = `<nav class="context-nav context-tab-list" role="tablist" aria-label="${escapeHtml(translate("assets.sections"))}">${destinations.map((destination) => {
      const active = destination.topicId === activeTopicId;
      return `<a id="help-wallet-section-${destination.id}" class="context-nav-item${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(destination.topicId).href)}" role="tab" aria-selected="${active}" aria-controls="help-document" tabindex="${active ? "0" : "-1"}" data-wallet-section="${destination.id}" data-help-context-topic="${escapeHtml(destination.topicId)}"${active ? ' aria-current="page"' : ""}>${icon(destination.iconId)}<span><strong>${escapeHtml(translate(destination.labelKey))}</strong></span></a>`;
    }).join("")}</nav>`;
    return { workspace, layoutClass: "wallet-assets-layout", navigation, usesWorkspaceAttribute: false, frame: "workspace" };
  }

  function walletSettingsContext(workspace) {
    const sections = [
      { id: "general", topicId: "wallet.settings.general", labelKey: "navigation.general", iconId: "settings" },
      { id: "security", topicId: "wallet.settings.security", labelKey: "navigation.security", iconId: "shield" },
      { id: "backup", topicId: "wallet.settings.backup", labelKey: "navigation.backup", iconId: "backup" },
      { id: "policies", topicId: "wallet.settings.policies", labelKey: "navigation.policies", iconId: "permission" },
      { id: "advanced", topicId: "wallet.settings.advanced", labelKey: "navigation.advanced", iconId: "advanced" }
    ];
    const destinations = sections.filter(({ topicId }) => registry.hasTopic(topicId));
    if (!destinations.length) return null;
    const navigation = `<nav class="context-nav context-tab-list wallet-settings-context" aria-label="${escapeHtml(translate("navigation.walletSettings"))}">${destinations.map((destination) => {
      const active = destination.topicId === activeTopicId;
      return `<a class="context-nav-item${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(destination.topicId).href)}" data-wallet-settings-section="${destination.id}" data-help-context-topic="${escapeHtml(destination.topicId)}"${active ? ' aria-current="page"' : ""}>${icon(destination.iconId)}<span><strong>${escapeHtml(translate(destination.labelKey))}</strong></span></a>`;
    }).join("")}</nav>`;
    return { workspace, navigation, usesWorkspaceAttribute: false, frame: "wallet-settings" };
  }

  function settingsContext(workspace) {
    const destinations = workspaceDestinations(workspace, { includeHidden: true });
    if (!destinations.length) return null;
    const navigation = `<nav class="context-nav context-tab-list settings-network-tabs help-settings-network-tabs" role="tablist" aria-label="${escapeHtml(translate("navigation.network"))}">${destinations.map((destination) => {
      const active = destination.topicId === activeTopicId;
      return `<a class="context-nav-item settings-network-tab${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(destination.topicId).href)}" role="tab" aria-selected="${active}" tabindex="${active ? "0" : "-1"}" data-help-context-topic="${escapeHtml(destination.topicId)}">${icon(destination.iconId)}<span><strong>${escapeHtml(translate(destination.labelKey))}</strong></span></a>`;
    }).join("")}</nav>`;
    return {
      workspace,
      navigation,
      usesWorkspaceAttribute: true,
      frame: "settings-network"
    };
  }

  function workspaceContext() {
    const workspace = activeWorkspaceNode();
    if (workspace?.id === "wallet.assets-rights") return walletAssetsContext(workspace);
    if (workspace?.id === "wallet.settings") return walletSettingsContext(workspace);
    if (workspace?.id === "settings.network") return settingsContext(workspace);
    const destinations = workspace ? workspaceDestinations(workspace) : [];
    if (!workspace || !destinations.length) return null;
    const workspaceClass = workspace.id.startsWith("telemetry.") ? " telemetry-workspace-context" : "";
    const label = escapeHtml(nodeLabel(workspace));
    const navigation = `<nav class="context-nav context-tab-list workspace-local-context${workspaceClass}" aria-label="${label}">${destinations.map((destination) => {
      const active = destination.topicId === activeTopicId;
      return `<a class="context-nav-item${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(destination.topicId).href)}" data-workspace-route="${escapeHtml(destination.routeId)}" data-help-context-topic="${escapeHtml(destination.topicId)}"${active ? ' aria-current="page"' : ""}>${icon(destination.iconId)}<span><strong>${escapeHtml(translate(destination.labelKey))}</strong></span></a>`;
    }).join("")}</nav>`;
    return { workspace, layoutClass: `workspace-local-layout${workspace.id.startsWith("telemetry.") ? " telemetry-workspace-layout" : ""}`, navigation, usesWorkspaceAttribute: true, frame: "workspace" };
  }

  function renderMobileTopbarContext() {
    mobileTopbarContext.replaceChildren();
    const source = article.querySelector(".workspace-layout > .context-rail > .context-nav");
    const mobile = root.matchMedia("(max-width: 768px)").matches;
    if (!mobile || !source) {
      mobileTopbarContext.hidden = true;
      siteHeader.classList.remove("has-mobile-context");
      document.body.classList.remove("has-help-mobile-context");
      return;
    }
    mobileTopbarContext.append(source);
    mobileTopbarContext.hidden = false;
    siteHeader.classList.add("has-mobile-context");
    document.body.classList.add("has-help-mobile-context");
    requestAnimationFrame(() => {
      mobileTopbarContext.querySelector(".context-nav-item.is-active")
        ?.scrollIntoView({ block: "nearest", inline: "center" });
      article.querySelector(".settings-network-rail .context-nav-item.is-active")
        ?.scrollIntoView({ block: "nearest", inline: "center" });
    });
  }

  function normalizeSearchValue(value) {
    return String(value || "")
      .normalize("NFKC")
      .replace(/\s+/gu, " ")
      .trim();
  }

  function normalizeSearchText(value) {
    return normalizeSearchValue(value).toLocaleLowerCase(language);
  }

  function searchRecord(topic) {
    const documentData = registry.resolveDocument(language, topic.id);
    return {
      body: normalizeSearchValue(documentData?.text),
      documentData,
      path: topic.pagePath || topic.id,
      title: normalizeSearchValue(documentData?.title || topic.id),
      topic
    };
  }

  function scoreSearchRecord(record, query) {
    if (!query) {
      return { ...record, exactPhrase: false, score: Number.MAX_SAFE_INTEGER };
    }
    const title = normalizeSearchText(record.title);
    const body = normalizeSearchText(record.body);
    const tokens = query.split(/\s+/u).filter(Boolean);
    const haystack = `${title} ${body}`.trim();
    if (!tokens.every((token) => haystack.includes(token))) return null;

    const exactPhrase = title.includes(query) || body.includes(query);
    let score = 80;
    if (title === query) score = 0;
    else if (title.startsWith(query)) score = 10;
    else if (title.includes(query)) score = 20;
    else if (tokens.every((token) => title.includes(token))) score = 30;
    else if (body.includes(query)) score = 70;
    return { ...record, exactPhrase, score };
  }

  function searchTopics(queryValue) {
    const query = normalizeSearchText(queryValue);
    const ranked = registry.topics()
      .map(searchRecord)
      .map((record) => scoreSearchRecord(record, query))
      .filter(Boolean);
    const matches = query.includes(" ")
      ? ranked.filter(({ exactPhrase }) => exactPhrase)
      : ranked;
    return matches
      .sort((left, right) => left.score - right.score
        || left.title.localeCompare(right.title, language)
        || left.topic.id.localeCompare(right.topic.id))
      .slice(0, 10);
  }

  function searchExcerpt(record, queryValue) {
    const body = normalizeSearchValue(record.body);
    const query = normalizeSearchText(queryValue);
    const matchIndex = query ? normalizeSearchText(body).indexOf(query) : -1;
    const start = matchIndex === -1 ? 0 : Math.max(0, matchIndex - 72);
    const end = matchIndex === -1
      ? Math.min(body.length, 240)
      : Math.min(body.length, matchIndex + query.length + 144);
    const excerpt = `${start > 0 ? "…" : ""}${body.slice(start, end).trim()}${end < body.length ? "…" : ""}`;
    return excerpt || translate("help.unavailable");
  }

  function renderSearch() {
    const matches = searchTopics(searchQuery);
    const resultLabel = translate("navigation.search");
    searchStatus.textContent = `${matches.length} ${resultLabel}`;
    searchResults.innerHTML = matches.map((record) => `
      <a class="help-search-result" href="${escapeHtml(routeUrl(record.topic.id).href)}" data-help-search-topic="${escapeHtml(record.topic.id)}">
        <span class="help-search-result-content">
          <strong>${escapeHtml(record.title)}</strong>
          <small>${escapeHtml(searchExcerpt(record, searchQuery))}</small>
        </span>
        <span class="help-search-result-path">${escapeHtml(record.path)}</span>
      </a>`).join("") || `<p class="help-search-empty">${escapeHtml(translate("help.unavailable"))}</p>`;
  }

  function searchIsOpen() {
    return !searchOverlay.hidden;
  }

  function setSearchBackgroundInert(inert) {
    siteHeader.inert = inert;
    main.inert = inert;
    sidebar.inert = inert;
    backdrop.inert = inert;
  }

  function openSearch() {
    if (searchIsOpen()) {
      searchInput.focus();
      return;
    }
    closeSidebar();
    closeLanguagePicker();
    searchOverlay.hidden = false;
    document.body.classList.add("has-help-search");
    searchTrigger.setAttribute("aria-expanded", "true");
    setSearchBackgroundInert(true);
    renderSearch();
    requestAnimationFrame(() => searchInput.focus());
  }

  function closeSearch({ restoreFocus = false } = {}) {
    const wasOpen = searchIsOpen();
    searchOverlay.hidden = true;
    document.body.classList.remove("has-help-search");
    searchTrigger.setAttribute("aria-expanded", "false");
    searchQuery = "";
    searchInput.value = "";
    renderSearch();
    setSearchBackgroundInert(false);
    if (restoreFocus && wasOpen) searchTrigger.focus();
  }

  function closeLanguagePicker({ restoreFocus = false } = {}) {
    const picker = languagePicker.querySelector("[data-help-language-picker]");
    if (!picker?.classList.contains("is-open")) return;
    picker.classList.remove("is-open");
    const menu = picker.querySelector("[data-help-language-options]");
    menu.hidden = true;
    menu.removeAttribute("style");
    const trigger = picker.querySelector("[data-help-language-trigger]");
    trigger.setAttribute("aria-expanded", "false");
    if (restoreFocus) trigger.focus();
  }

  function openLanguagePicker() {
    const picker = languagePicker.querySelector("[data-help-language-picker]");
    const trigger = picker?.querySelector("[data-help-language-trigger]");
    const menu = picker?.querySelector("[data-help-language-options]");
    if (!picker || !trigger || !menu) return;
    picker.classList.add("is-open");
    trigger.setAttribute("aria-expanded", "true");
    menu.hidden = false;
    requestAnimationFrame(() => {
      if (!picker.classList.contains("is-open")) return;
      const triggerRect = trigger.getBoundingClientRect();
      const viewportPadding = 12;
      const menuHeight = Math.min(menu.scrollHeight, 360);
      const spaceAbove = triggerRect.top - viewportPadding;
      const spaceBelow = root.innerHeight - triggerRect.bottom - viewportPadding;
      const opensUpward = spaceBelow < Math.min(menuHeight, 224) && spaceAbove > spaceBelow;
      const width = Math.min(Math.max(triggerRect.width, 220), root.innerWidth - viewportPadding * 2);
      const left = Math.max(viewportPadding, Math.min(triggerRect.right - width, root.innerWidth - width - viewportPadding));
      menu.style.left = `${Math.round(left)}px`;
      menu.style.width = `${Math.round(width)}px`;
      menu.style.maxHeight = `${Math.floor(Math.max(128, opensUpward ? spaceAbove : spaceBelow))}px`;
      menu.style.top = opensUpward ? "auto" : `${Math.round(triggerRect.bottom + 6)}px`;
      menu.style.bottom = opensUpward ? `${Math.max(viewportPadding, Math.round(root.innerHeight - triggerRect.top + 6))}px` : "auto";
    });
  }

  function selectLanguage(languageId) {
    const nextLanguage = i18n.resolveLanguage(languageId);
    if (nextLanguage === language) {
      closeLanguagePicker({ restoreFocus: true });
      return;
    }
    language = nextLanguage;
    root.history.replaceState({}, "", routeUrl(activeTopicId, sectionTarget).href);
    render({ focusHeading: true });
    requestAnimationFrame(() => languagePicker.querySelector("[data-help-language-trigger]")?.focus());
  }

  function renderDocument(focusHeading = false) {
    const documentData = registry.resolveDocument(language, activeTopicId);
    if (!documentData) {
      article.innerHTML = `<header class="help-document-header"><h1 id="help-title" tabindex="-1">${escapeHtml(translate("help.unavailable"))}</h1></header>`;
      renderMobileTopbarContext();
      return;
    }

    const context = workspaceContext();
    if (context) {
      const panel = `<div class="workspace-panel help-markdown">${documentData.html}</div>`;
      if (context.frame === "wallet-settings") {
        article.innerHTML = `<div class="view-enter settings-view wallet-settings-view"><div class="workspace-layout settings-layout"><aside class="context-rail help-context-rail">${context.navigation}</aside>${panel}</div></div>`;
      } else if (context.frame === "settings-network") {
        article.innerHTML = `<div class="view-enter workspace-layout workspace-local-layout settings-network-workspace" data-workspace-id="${escapeHtml(context.workspace.id)}"><aside class="context-rail settings-network-rail help-context-rail">${context.navigation}</aside>${panel}</div>`;
      } else {
        const workspaceAttribute = context.usesWorkspaceAttribute ? ` data-workspace-id="${escapeHtml(context.workspace.id)}"` : "";
        article.innerHTML = `<div class="view-enter workspace-layout ${context.layoutClass}"${workspaceAttribute}><aside class="context-rail help-context-rail">${context.navigation}</aside>${panel}</div>`;
      }
    } else {
      article.innerHTML = `<div class="help-markdown">${documentData.html}</div>`;
    }
    renderMobileTopbarContext();
    article.querySelector("h1")?.setAttribute("id", "help-title");
    article.querySelector("h1")?.setAttribute("tabindex", "-1");
    article.querySelectorAll(".table-of-contents").forEach((tableOfContents) => {
      const label = translate("help.contents");
      tableOfContents.dataset.title = label;
      tableOfContents.setAttribute("aria-label", label);
    });
    root.Z00ZHelpMarkdownEnhancer?.enhance(article);
    document.title = `${documentData.title} · Z00Z Help`;
    const target = sectionTarget ? article.querySelector(`#${CSS.escape(sectionTarget)}`) : undefined;
    if (target) {
      target.setAttribute("tabindex", "-1");
      target.scrollIntoView({ block: "start" });
      target.focus({ preventScroll: true });
    } else if (focusHeading) {
      article.querySelector("#help-title")?.focus();
      main.scrollTo({ top: 0, behavior: "auto" });
    }
  }

  function renderChrome() {
    const searchLabel = translate("navigation.search");
    document.documentElement.lang = language;
    document.documentElement.dir = languageMetadata().direction || "ltr";
    document.querySelector("#help-product-label").textContent = translate("navigation.help");
    document.querySelector("#help-mobile-menu-title").textContent = translate("help.drawerTitle");
    document.querySelector("#help-search-label").textContent = searchLabel;
    searchDialogTitle.textContent = searchLabel;
    searchTriggerLabel.textContent = searchLabel;
    searchTrigger.setAttribute("aria-label", searchLabel);
    searchShortcut.textContent = navigator.platform.toLocaleLowerCase().includes("mac") ? "⌘K" : "Ctrl K";
    menuButton.setAttribute("aria-label", translate("app.menu"));
    backdrop.setAttribute("aria-label", translate("common.close"));
    searchClose.setAttribute("aria-label", translate("common.close"));
    searchInput.placeholder = `${searchLabel}…`;
    searchInput.setAttribute("aria-label", searchLabel);
    tree.setAttribute("aria-label", translate("help.contents"));
    navigationTerminal.setAttribute("aria-label", translate("navigation.settings"));
    languagePicker.innerHTML = languagePickerMarkup();
  }

  function render(options = {}) {
    renderChrome();
    renderTree();
    renderSearch();
    renderDocument(options.focusHeading);
    requestAnimationFrame(() => {
      restoreNavigationScrollPosition(mobileNavigationLayout ? "mobile" : "desktop");
    });
  }

  function openTopic(topicId) {
    if (!registry.hasTopic(topicId)) return;
    activeTopicId = topicId;
    sectionTarget = "";
    expandActiveBranch();
    persistNavigationState();
    root.history.pushState({}, "", routeUrl(topicId).href);
    render({ focusHeading: true });
    closeSidebar();
  }

  function openSidebar() {
    if (!mobileNavigationLayout) return;
    if (!sidebar.hidden) {
      closeSidebar({ restoreFocus: true });
      return;
    }
    closeLanguagePicker();
    sidebar.dataset.popupType = "menu";
    sidebar.setAttribute("role", "dialog");
    sidebar.setAttribute("aria-modal", "true");
    sidebar.classList.add("is-open");
    sidebar.hidden = false;
    backdrop.hidden = false;
    document.body.classList.add("has-mobile-drawer");
    main.inert = true;
    menuButton.setAttribute("aria-expanded", "true");
    persistNavigationState();
    requestAnimationFrame(() => {
      const focusTarget = sidebar.querySelector("[aria-current='page']")
        || sidebar.querySelector('button:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])');
      const restoredScrollTop = navigationScrollPositions.mobile || 0;
      if (restoredScrollTop > 0) {
        restoreNavigationScrollPosition("mobile");
      } else {
        focusTarget?.scrollIntoView({ block: "nearest" });
      }
      focusTarget?.focus({ preventScroll: restoredScrollTop > 0 });
    });
  }

  function closeSidebar({ restoreFocus = false } = {}) {
    const wasOpen = !sidebar.hidden && sidebar.classList.contains("is-open");
    if (wasOpen) captureNavigationScrollPosition();
    sidebar.classList.remove("is-open");
    backdrop.hidden = true;
    document.body.classList.remove("has-mobile-drawer");
    main.inert = searchIsOpen();
    menuButton.setAttribute("aria-expanded", "false");
    sidebar.removeAttribute("aria-modal");
    if (mobileNavigationLayout) {
      sidebar.dataset.popupType = "menu";
      sidebar.setAttribute("role", "dialog");
      sidebar.hidden = true;
    } else {
      sidebar.removeAttribute("data-popup-type");
      sidebar.removeAttribute("role");
      sidebar.hidden = false;
    }
    persistNavigationState();
    if (restoreFocus && wasOpen) menuButton.focus();
  }

  function resetMobileDrawerSwipe() {
    mobileDrawerSwipe.pointerId = null;
    mobileDrawerSwipe.source = "";
    mobileDrawerSwipe.direction = "";
  }

  function beginMobileDrawerSwipe({ source, pointerId, clientX, clientY, target }) {
    if (!mobileNavigationLayout) return;
    if (searchIsOpen()) return;
    const touchReplacesPointer = source === "touch" && mobileDrawerSwipe.source === "pointer";
    if (mobileDrawerSwipe.pointerId !== null && !touchReplacesPointer) return;
    if (document.querySelector("[data-help-language-picker].is-open")) return;

    const drawerIsOpen = !sidebar.hidden;
    const startsInDrawer = target instanceof Element && Boolean(target.closest("#help-sidebar"));
    if ((!drawerIsOpen && clientX > mobileDrawerSwipeEdge) || (drawerIsOpen && !startsInDrawer)) return;

    mobileDrawerSwipe.pointerId = pointerId;
    mobileDrawerSwipe.source = source;
    mobileDrawerSwipe.startX = clientX;
    mobileDrawerSwipe.startY = clientY;
    mobileDrawerSwipe.direction = drawerIsOpen ? "close" : "open";
  }

  function completeMobileDrawerSwipe({ source, pointerId, clientX, clientY }) {
    if (source !== mobileDrawerSwipe.source || pointerId !== mobileDrawerSwipe.pointerId) return;
    const { startX, startY, direction } = mobileDrawerSwipe;
    resetMobileDrawerSwipe();

    const deltaX = clientX - startX;
    const deltaY = clientY - startY;
    const isHorizontalSwipe = Math.abs(deltaX) >= mobileDrawerSwipeDistance
      && Math.abs(deltaX) > Math.abs(deltaY) * 1.25;
    if (!isHorizontalSwipe) return;

    if (direction === "open" && deltaX > 0 && sidebar.hidden) {
      openSidebar();
    } else if (direction === "close" && deltaX < 0 && !sidebar.hidden) {
      closeSidebar({ restoreFocus: true });
    }
  }

  function cancelMobileDrawerSwipe({ source, pointerId }) {
    if (source !== mobileDrawerSwipe.source || pointerId !== mobileDrawerSwipe.pointerId) return;
    resetMobileDrawerSwipe();
  }

  tree.addEventListener("click", (event) => {
    const branch = event.target.closest("[data-help-navigation-branch]");
    if (branch) {
      const nodeId = branch.dataset.helpNavigationBranch;
      if (expandedBranchIds.has(nodeId)) expandedBranchIds.delete(nodeId);
      else expandedBranchIds.add(nodeId);
      persistNavigationState();
      renderTree();
      requestAnimationFrame(() => tree.querySelector(`[data-help-navigation-branch="${CSS.escape(nodeId)}"]`)?.focus({ preventScroll: true }));
      return;
    }
    const link = event.target.closest("[data-help-topic-link]");
    if (link) {
      event.preventDefault();
      openTopic(link.dataset.helpTopicLink);
    }
  });

  navigationTerminal.addEventListener("click", (event) => {
    const branch = event.target.closest("[data-help-navigation-branch]");
    if (branch) {
      const nodeId = branch.dataset.helpNavigationBranch;
      if (expandedBranchIds.has(nodeId)) expandedBranchIds.delete(nodeId);
      else expandedBranchIds.add(nodeId);
      persistNavigationState();
      renderTree();
      requestAnimationFrame(() => navigationTerminal.querySelector(`[data-help-navigation-branch="${CSS.escape(nodeId)}"]`)?.focus({ preventScroll: true }));
      return;
    }
    const link = event.target.closest("[data-help-topic-link]");
    if (link) {
      event.preventDefault();
      openTopic(link.dataset.helpTopicLink);
    }
  });

  function openContextTopic(event) {
    const link = event.target.closest("[data-help-context-topic]");
    if (!link) return;
    event.preventDefault();
    openTopic(link.dataset.helpContextTopic);
  }

  article.addEventListener("click", openContextTopic);
  mobileTopbarContext.addEventListener("click", openContextTopic);

  languagePicker.addEventListener("click", (event) => {
    const option = event.target.closest("[data-help-language-option]");
    if (option) {
      event.preventDefault();
      selectLanguage(option.dataset.helpLanguageOption);
      return;
    }
    const trigger = event.target.closest("[data-help-language-trigger]");
    if (!trigger) return;
    event.preventDefault();
    const picker = trigger.closest("[data-help-language-picker]");
    if (picker?.classList.contains("is-open")) closeLanguagePicker();
    else openLanguagePicker();
  });

  searchResults.addEventListener("click", (event) => {
    const link = event.target.closest("[data-help-search-topic]");
    if (!link) return;
    event.preventDefault();
    closeSearch();
    openTopic(link.dataset.helpSearchTopic);
  });

  searchTrigger.addEventListener("click", openSearch);
  searchBackdrop.addEventListener("pointerdown", () => closeSearch({ restoreFocus: true }));
  searchClose.addEventListener("click", () => closeSearch({ restoreFocus: true }));
  searchInput.addEventListener("input", () => {
    searchQuery = searchInput.value;
    renderSearch();
  });
  navigationScrollRegion.addEventListener("scroll", captureNavigationScrollPosition, { passive: true });
  menuButton.addEventListener("click", openSidebar);
  backdrop.addEventListener("click", () => closeSidebar({ restoreFocus: true }));
  homeLink.addEventListener("click", (event) => {
    event.preventDefault();
    openTopic(registry.globalTopic());
  });
  root.addEventListener("popstate", () => {
    closeSearch();
    closeSidebar();
    readRoute();
    render();
  });
  root.addEventListener("resize", () => {
    const mobile = root.matchMedia("(max-width: 768px)").matches;
    if (mobile === mobileNavigationLayout) return;
    mobileNavigationLayout = mobile;
    closeSidebar();
    render();
  });
  document.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLocaleLowerCase() === "k") {
      event.preventDefault();
      openSearch();
      return;
    }
    if (searchIsOpen()) {
      if (event.key === "Escape") {
        event.preventDefault();
        closeSearch({ restoreFocus: true });
        return;
      }
      if (event.key === "Tab") {
        const focusable = [...searchDialog.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])')]
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
    if (event.key === "Escape" && !sidebar.hidden) {
      event.preventDefault();
      closeSidebar({ restoreFocus: true });
      return;
    }
    if (event.key === "Tab" && !sidebar.hidden) {
      const focusable = [...sidebar.querySelectorAll('a[href], button:not([disabled]), input:not([disabled])')]
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
      return;
    }
    if (event.key === "Escape") {
      closeLanguagePicker({ restoreFocus: true });
    }
  });
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
  document.addEventListener("click", (event) => {
    if (!event.target.closest("[data-help-language-picker]")) closeLanguagePicker();
  });

  readRoute({ preserveExpansion: Boolean(restoredNavigationSnapshot) });
  closeSidebar();
  navigationSessionReady = true;
  render();
  persistNavigationState();
  if (restoredNavigationSnapshot?.drawerOpen && mobileNavigationLayout) {
    requestAnimationFrame(openSidebar);
  }
})(typeof window === "undefined" ? globalThis : window);
