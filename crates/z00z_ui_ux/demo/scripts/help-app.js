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
  const searchInput = document.querySelector("#help-search");
  const searchClear = document.querySelector("#help-search-clear");
  const searchResults = document.querySelector("#help-search-results");
  const searchStatus = document.querySelector("#help-search-status");
  const article = document.querySelector("#help-document");
  const main = document.querySelector("#help-main");
  const sidebar = document.querySelector("#help-sidebar");
  const menuButton = document.querySelector("#help-menu-button");
  const closeButton = document.querySelector("#help-sidebar-close");
  const backdrop = document.querySelector("#help-sidebar-backdrop");
  const homeLink = document.querySelector("#help-home-link");
  const openNodes = new Set();
  let language = "en";
  let activeTopicId = registry.globalTopic();
  let sectionTarget = "";
  let searchQuery = "";

  root.name = "z00z-help";

  const escapeHtml = (value) => String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
  const icon = (name) => `<svg class="icon" aria-hidden="true"><use href="#i-${name}"/></svg>`;
  const translate = (key, values) => i18n.translate(language, key, values);

  function readRoute() {
    const parameters = new URLSearchParams(root.location.search);
    language = "en";
    activeTopicId = topicIds.has(parameters.get("topic")) ? parameters.get("topic") : registry.globalTopic();
    sectionTarget = /^[a-z][a-z0-9-]*$/u.test(parameters.get("section") || "") ? parameters.get("section") : "";
    document.documentElement.dataset.palette = parameters.get("palette") === "z00z-corporate" ? "z00z-corporate" : "z00z-default";
    expandActivePath();
  }

  function routeUrl(topicId, target = "") {
    const url = new URL(root.location.href);
    url.hash = "";
    url.searchParams.set("topic", topicId);
    url.searchParams.delete("lang");
    if (target) url.searchParams.set("section", target);
    else url.searchParams.delete("section");
    return url;
  }

  function expandActivePath() {
    const record = registry.topic(activeTopicId);
    const node = record?.routeId ? demo.navigationNodeForRoute(record.routeId) : demo.navigationNode("help");
    let current = node;
    while (current?.parentId) {
      current = demo.navigationNode(current.parentId);
      if (current?.target.kind === "branch") openNodes.add(current.id);
    }
  }

  function nodeLabel(node) {
    return translate(node.labelKey);
  }

  function isActiveNode(node) {
    const record = registry.topic(activeTopicId);
    return node.helpTopicId === activeTopicId || Boolean(record?.routeId && node.target.routeId === record.routeId);
  }

  function nodeMarkup(node, depth = 0) {
    if (node.target.kind === "action") return "";
    const children = demo.navigationChildren(node.id).filter((child) => child.target.kind !== "action");
    const hasChildren = children.length > 0;
    const active = isActiveNode(node);
    const label = escapeHtml(nodeLabel(node));
    const depthClass = `is-depth-${depth}`;
    const controlId = `help-node-${node.id.replaceAll(".", "-")}`;
    const childId = `${controlId}-children`;
    const expanded = openNodes.has(node.id);
    const targetId = node.target.kind === "help" ? registry.globalTopic() : node.helpTopicId;
    const destination = targetId && registry.hasTopic(targetId)
      ? `<a class="help-tree-link${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(targetId).href)}" data-help-topic-link="${escapeHtml(targetId)}"${active ? ' aria-current="page"' : ""}>${icon(node.iconId)}<span>${label}</span></a>`
      : `<span class="help-tree-label">${icon(node.iconId)}<span>${label}</span></span>`;

    if (!hasChildren) return `<div class="help-tree-node ${depthClass}">${destination}</div>`;

    const opener = node.target.kind === "branch"
      ? `<button class="help-tree-toggle" type="button" data-help-tree-toggle="${escapeHtml(node.id)}" aria-expanded="${expanded}" aria-controls="${childId}">${icon(node.iconId)}<span>${label}</span>${icon("chevron")}</button>`
      : `<div class="help-tree-node-head">${destination}<button class="help-tree-child-toggle" type="button" data-help-tree-toggle="${escapeHtml(node.id)}" aria-expanded="${expanded}" aria-controls="${childId}" aria-label="${label}">${icon("chevron")}</button></div>`;

    return `<section class="help-tree-node help-tree-branch ${depthClass}${expanded ? " is-expanded" : ""}${active ? " is-active" : ""}">${opener}<div class="help-tree-children" id="${childId}"${expanded ? "" : " hidden"}>${children.map((child) => nodeMarkup(child, depth + 1)).join("")}</div></section>`;
  }

  function renderTree(focusId = "") {
    tree.innerHTML = demo.navigationChildren()
      .filter((node) => node.target.kind !== "action")
      .map((node) => nodeMarkup(node))
      .join("");
    if (focusId) requestAnimationFrame(() => tree.querySelector(`[data-help-tree-toggle="${focusId}"]`)?.focus());
  }

  function searchableText(topic) {
    const documentData = registry.resolveDocument(language, topic.id);
    return `${documentData?.title || ""} ${documentData?.text || ""}`.toLocaleLowerCase(language);
  }

  function renderSearch() {
    const query = searchQuery.trim().toLocaleLowerCase(language);
    const matches = query ? registry.topics().filter((topic) => searchableText(topic).includes(query)) : [];
    tree.hidden = Boolean(query);
    searchClear.hidden = !searchQuery;
    searchResults.hidden = !query;
    searchStatus.textContent = query ? `${matches.length} ${translate("navigation.search")}` : "";
    searchResults.innerHTML = query
      ? matches.map((topic) => {
        const documentData = registry.resolveDocument(language, topic.id);
        return `<a class="help-search-result" href="${escapeHtml(routeUrl(topic.id).href)}" data-help-search-topic="${escapeHtml(topic.id)}"><span>${escapeHtml(documentData?.title || topic.id)}</span></a>`;
      }).join("") || `<p class="help-search-empty">${escapeHtml(translate("help.unavailable"))}</p>`
      : "";
  }

  function renderDocument(focusHeading = false) {
    const documentData = registry.resolveDocument(language, activeTopicId);
    if (!documentData) {
      article.innerHTML = `<header class="help-document-header"><h1 id="help-title" tabindex="-1">${escapeHtml(translate("help.unavailable"))}</h1></header>`;
      return;
    }

    article.innerHTML = `<div class="help-markdown">${documentData.html}</div>`;
    article.querySelector("h1")?.setAttribute("id", "help-title");
    article.querySelector("h1")?.setAttribute("tabindex", "-1");
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
    document.querySelector("#help-product-label").textContent = translate("navigation.help");
    document.querySelector("#help-contents-eyebrow").textContent = translate("navigation.help");
    document.querySelector("#help-contents-title").textContent = translate("help.contents");
    document.querySelector("#help-search-label").textContent = translate("navigation.search");
    searchInput.placeholder = `${translate("navigation.search")}…`;
    searchInput.setAttribute("aria-label", translate("navigation.search"));
    tree.setAttribute("aria-label", translate("help.contents"));
  }

  function render(options = {}) {
    renderChrome();
    renderTree(options.focusNode);
    renderSearch();
    renderDocument(options.focusHeading);
  }

  function openTopic(topicId) {
    if (!registry.hasTopic(topicId)) return;
    activeTopicId = topicId;
    sectionTarget = "";
    root.history.pushState({}, "", routeUrl(topicId).href);
    expandActivePath();
    render({ focusHeading: true });
    closeSidebar();
  }

  function openSidebar() {
    sidebar.classList.add("is-open");
    backdrop.hidden = false;
    menuButton.setAttribute("aria-expanded", "true");
  }

  function closeSidebar() {
    sidebar.classList.remove("is-open");
    backdrop.hidden = true;
    menuButton.setAttribute("aria-expanded", "false");
  }

  tree.addEventListener("click", (event) => {
    const toggle = event.target.closest("[data-help-tree-toggle]");
    if (toggle) {
      const nodeId = toggle.dataset.helpTreeToggle;
      if (openNodes.has(nodeId)) openNodes.delete(nodeId);
      else openNodes.add(nodeId);
      render({ focusNode: nodeId });
      return;
    }
    const link = event.target.closest("[data-help-topic-link]");
    if (link) {
      event.preventDefault();
      openTopic(link.dataset.helpTopicLink);
    }
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
  closeButton.addEventListener("click", closeSidebar);
  backdrop.addEventListener("click", closeSidebar);
  homeLink.addEventListener("click", (event) => {
    event.preventDefault();
    openTopic(registry.globalTopic());
  });
  root.addEventListener("popstate", () => {
    readRoute();
    render();
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeSidebar();
    }
  });

  readRoute();
  render();
})(typeof window === "undefined" ? globalThis : window);
