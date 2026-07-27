"use strict";

((root) => {
  const registry = root.Z00ZHelpRegistry;
  const i18n = root.Z00ZI18n;
  const demo = root.Z00ZDemo;
  if (!registry || !i18n || !demo?.navigationChildren) {
    throw new Error("Standalone Help dependencies are missing.");
  }

  const topicIds = new Set(registry.topics().map(({ id }) => id));
  const tree = document.querySelector("#help-tree");
  const navigationTerminal = document.querySelector("#help-navigation-terminal");
  const searchInput = document.querySelector("#help-search");
  const searchClear = document.querySelector("#help-search-clear");
  const searchResults = document.querySelector("#help-search-results");
  const searchStatus = document.querySelector("#help-search-status");
  const article = document.querySelector("#help-document");
  const main = document.querySelector("#help-main");
  const sidebar = document.querySelector("#help-sidebar");
  const siteHeader = document.querySelector(".help-site-header");
  const mobileTopbarContext = document.querySelector("#help-mobile-topbar-context");
  const menuButton = document.querySelector("#help-menu-button");
  const closeButton = document.querySelector("#help-sidebar-close");
  const backdrop = document.querySelector("#help-sidebar-backdrop");
  const homeLink = document.querySelector("#help-home-link");
  const languagePicker = document.querySelector("#help-language-picker");
  let language = "en";
  let activeTopicId = registry.globalTopic();
  let sectionTarget = "";
  let searchQuery = "";
  let mobileNavigationLayout = root.matchMedia("(max-width: 768px)").matches;
  const expandedBranchIds = new Set(["wallet"]);
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

  function readRoute() {
    const parameters = new URLSearchParams(root.location.search);
    language = i18n.resolveLanguage(parameters.get("lang"));
    activeTopicId = topicIds.has(parameters.get("topic")) ? parameters.get("topic") : registry.globalTopic();
    sectionTarget = /^[a-z][a-z0-9-]*$/u.test(parameters.get("section") || "") ? parameters.get("section") : "";
    document.documentElement.dataset.palette = parameters.get("palette") === "z00z-corporate" ? "z00z-corporate" : "z00z-default";
    expandActiveBranch();
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
    const activeDescendant = hasActiveDescendant(node);
    if (node.target.kind === "branch") {
      const expanded = expandedBranchIds.has(node.id);
      const controlId = `${prefix}-${node.id.replaceAll(".", "-")}-toggle`;
      const panelId = `${prefix}-${node.id.replaceAll(".", "-")}-children`;
      return `<section class="navigation-tree-branch ${depthClass}${expanded ? " is-expanded" : ""}${activeDescendant ? " has-active-descendant" : ""}" data-help-navigation-node="${escapeHtml(node.id)}">
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
      return `<section class="navigation-tree-group ${depthClass}${activeDescendant ? " has-active-descendant" : ""}" data-help-navigation-node="${escapeHtml(node.id)}" aria-labelledby="${groupId}">
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
    return `<a class="navigation-tree-item navigation-tree-leaf${terminal ? " navigation-tree-terminal" : ""} ${depthClass}${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(topicId).href)}" data-help-navigation-node="${escapeHtml(node.id)}" data-help-topic-link="${escapeHtml(topicId)}"${node.target.kind === "workspace" ? ` data-navigation-workspace="${escapeHtml(node.id)}"` : ""}${active ? ' aria-current="page"' : ""}>
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

  function workspaceDestinations(workspace) {
    return demo.workspaceLocalDestinations(workspace.id)
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
      { id: "vouchers", topicId: "wallet.vouchers", routeId: "wallet.vouchers", labelKey: "assets.sectionVouchers", iconId: "voucher" },
      { id: "permissions", topicId: "wallet.permissions", routeId: "wallet.permissions", labelKey: "assets.sectionPermissions", iconId: "permission" }
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

  function workspaceContext() {
    const workspace = activeWorkspaceNode();
    if (workspace?.id === "wallet.assets-rights") return walletAssetsContext(workspace);
    if (workspace?.id === "wallet.settings") return walletSettingsContext(workspace);
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
  }

  function searchableText(topic) {
    const documentData = registry.resolveDocument(language, topic.id);
    return `${documentData?.title || ""} ${documentData?.text || ""}`.toLocaleLowerCase(language);
  }

  function renderSearch() {
    const query = searchQuery.trim().toLocaleLowerCase(language);
    const matches = query ? registry.topics().filter((topic) => searchableText(topic).includes(query)) : [];
    tree.hidden = !mobileNavigationLayout && Boolean(query);
    searchClear.hidden = !searchQuery;
    searchResults.hidden = mobileNavigationLayout || !query;
    searchStatus.textContent = query ? `${matches.length} ${translate("navigation.search")}` : "";
    searchResults.innerHTML = query
      ? matches.map((topic) => {
        const documentData = registry.resolveDocument(language, topic.id);
        return `<a class="help-search-result" href="${escapeHtml(routeUrl(topic.id).href)}" data-help-search-topic="${escapeHtml(topic.id)}"><span>${escapeHtml(documentData?.title || topic.id)}</span></a>`;
      }).join("") || `<p class="help-search-empty">${escapeHtml(translate("help.unavailable"))}</p>`
      : "";
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
    document.documentElement.lang = language;
    document.documentElement.dir = languageMetadata().direction || "ltr";
    document.querySelector("#help-product-label").textContent = translate("navigation.help");
    document.querySelector("#help-contents-eyebrow").textContent = translate("navigation.help");
    document.querySelector("#help-contents-title").textContent = translate("help.contents");
    document.querySelector("#help-mobile-menu-title").textContent = translate("app.menu");
    document.querySelector("#help-search-label").textContent = translate("navigation.search");
    menuButton.setAttribute("aria-label", translate("app.menu"));
    closeButton.setAttribute("aria-label", translate("common.close"));
    backdrop.setAttribute("aria-label", translate("common.close"));
    searchInput.placeholder = `${translate("navigation.search")}…`;
    searchInput.setAttribute("aria-label", translate("navigation.search"));
    tree.setAttribute("aria-label", translate("help.contents"));
    navigationTerminal.setAttribute("aria-label", translate("navigation.settings"));
    languagePicker.innerHTML = languagePickerMarkup();
  }

  function render(options = {}) {
    renderChrome();
    renderTree();
    renderSearch();
    renderDocument(options.focusHeading);
  }

  function openTopic(topicId) {
    if (!registry.hasTopic(topicId)) return;
    activeTopicId = topicId;
    sectionTarget = "";
    expandActiveBranch();
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
    requestAnimationFrame(() => {
      sidebar.querySelector("[aria-current='page']")?.scrollIntoView({ block: "nearest" });
      closeButton.focus();
    });
  }

  function closeSidebar({ restoreFocus = false } = {}) {
    const wasOpen = !sidebar.hidden && sidebar.classList.contains("is-open");
    sidebar.classList.remove("is-open");
    backdrop.hidden = true;
    document.body.classList.remove("has-mobile-drawer");
    main.inert = false;
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
    if (restoreFocus && wasOpen) menuButton.focus();
  }

  function resetMobileDrawerSwipe() {
    mobileDrawerSwipe.pointerId = null;
    mobileDrawerSwipe.source = "";
    mobileDrawerSwipe.direction = "";
  }

  function beginMobileDrawerSwipe({ source, pointerId, clientX, clientY, target }) {
    if (!mobileNavigationLayout) return;
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
      renderTree();
      requestAnimationFrame(() => tree.querySelector(`[data-help-navigation-branch="${CSS.escape(nodeId)}"]`)?.focus());
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
      renderTree();
      requestAnimationFrame(() => navigationTerminal.querySelector(`[data-help-navigation-branch="${CSS.escape(nodeId)}"]`)?.focus());
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
    openTopic(link.dataset.helpSearchTopic);
  });

  searchInput.addEventListener("input", () => {
    searchQuery = searchInput.value;
    renderSearch();
  });
  searchClear.addEventListener("click", () => {
    searchQuery = "";
    searchInput.value = "";
    renderSearch();
    searchInput.focus();
  });
  menuButton.addEventListener("click", openSidebar);
  closeButton.addEventListener("click", () => closeSidebar({ restoreFocus: true }));
  backdrop.addEventListener("click", () => closeSidebar({ restoreFocus: true }));
  homeLink.addEventListener("click", (event) => {
    event.preventDefault();
    openTopic(registry.globalTopic());
  });
  root.addEventListener("popstate", () => {
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

  readRoute();
  closeSidebar();
  render();
})(typeof window === "undefined" ? globalThis : window);
