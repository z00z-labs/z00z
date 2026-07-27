"use strict";

((root) => {
  const registry = root.Z00ZHelpRegistry;
  const i18n = root.Z00ZI18n;
  const localeRegistry = root.Z00ZLocaleRegistry;
  if (!registry || !i18n || !localeRegistry?.length) {
    throw new Error("Standalone Help dependencies are missing.");
  }

  const groupDefinitions = registry.groups();
  const groupOrder = Object.freeze(groupDefinitions.map(({ id }) => id));
  const groupIcons = Object.freeze(Object.fromEntries(groupDefinitions.map(({ id, iconId }) => [id, iconId])));
  const topicList = registry.topics();
  const workspaceDefinitions = Object.freeze([
    Object.freeze({
      id: "wallet.assets-rights",
      group: "wallets",
      labelKey: "navigation.assets",
      defaultTopicId: "wallet.assets",
      matches: (topicId) => ["wallet.assets", "wallet.vouchers", "wallet.permissions"].includes(topicId),
    }),
    Object.freeze({
      id: "wallet.settings",
      group: "wallets",
      labelKey: "navigation.walletSettings",
      defaultTopicId: "wallet.settings.general",
      matches: (topicId) => topicId.startsWith("wallet.settings."),
    }),
    Object.freeze({
      id: "wallet.staking",
      group: "wallets",
      labelKey: "navigation.staking",
      defaultTopicId: "wallet.staking.stake",
      matches: (topicId) => topicId.startsWith("wallet.staking."),
    }),
    ...["reticulum", "onionnet", "aggregators", "watchers", "explorer"].map((component) => Object.freeze({
      id: `telemetry.${component}`,
      group: "telemetry",
      labelKey: `navigation.${component}`,
      defaultTopicId: `telemetry.${component}.overview`,
      matches: (topicId) => topicId.startsWith(`telemetry.${component}.`),
    })),
    Object.freeze({
      id: "dapps",
      group: "dapps",
      labelKey: "navigation.dapps",
      defaultTopicId: "dapps.discover",
      matches: (topicId) => registry.topic(topicId)?.group === "dapps"
        && registry.topic(topicId)?.scope === "context",
    }),
    Object.freeze({
      id: "messenger",
      group: "messenger",
      labelKey: "navigation.messenger",
      defaultTopicId: "messenger.inbox",
      matches: (topicId) => topicId.startsWith("messenger.") && registry.topic(topicId)?.scope === "context",
    }),
    Object.freeze({
      id: "settings",
      group: "settings",
      labelKey: "navigation.settings",
      defaultTopicId: "settings.general",
      matches: (topicId) => topicId.startsWith("settings.") && registry.topic(topicId)?.scope === "context",
    }),
    Object.freeze({
      id: "data-storage",
      group: "data-storage",
      labelKey: "navigation.dataStorage",
      defaultTopicId: "data-storage.disk-usage",
      matches: (topicId) => topicId.startsWith("data-storage.") && registry.topic(topicId)?.scope === "context",
    }),
  ]);
  const topicIds = new Set(topicList.map(({ id }) => id));
  const languageIds = new Set(localeRegistry.map(({ id }) => id));
  const tree = document.querySelector("#help-tree");
  const searchInput = document.querySelector("#help-search");
  const searchClear = document.querySelector("#help-search-clear");
  const searchResults = document.querySelector("#help-search-results");
  const searchStatus = document.querySelector("#help-search-status");
  const contextTabs = document.querySelector("#help-context-tabs");
  const article = document.querySelector("#help-document");
  const main = document.querySelector("#help-main");
  const sidebar = document.querySelector("#help-sidebar");
  const menuButton = document.querySelector("#help-menu-button");
  const closeButton = document.querySelector("#help-sidebar-close");
  const backdrop = document.querySelector("#help-sidebar-backdrop");
  const languagePicker = document.querySelector("[data-help-language-picker]");
  const languageTrigger = document.querySelector("#help-language");
  const languageMenu = document.querySelector("#help-language-options");
  const walletLink = document.querySelector(".help-wallet-link");
  const openGroups = new Set();
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
    language = languageIds.has(parameters.get("lang")) ? parameters.get("lang") : "en";
    activeTopicId = topicIds.has(parameters.get("topic"))
      ? parameters.get("topic")
      : registry.globalTopic();
    sectionTarget = /^[a-z][a-z0-9-]*$/.test(parameters.get("section") || "")
      ? parameters.get("section")
      : "";
    const requestedPalette = parameters.get("palette");
    document.documentElement.dataset.palette = requestedPalette === "z00z-corporate"
      ? "z00z-corporate"
      : "z00z-default";
    const group = registry.topic(activeTopicId)?.group;
    if (group) openGroups.add(group);
  }

  function routeUrl(topicId, nextLanguage = language, target = "") {
    const url = new URL(root.location.href);
    url.hash = "";
    url.searchParams.set("topic", topicId);
    url.searchParams.set("lang", nextLanguage);
    if (target) url.searchParams.set("section", target);
    else url.searchParams.delete("section");
    return url;
  }

  function groupLabel(group) {
    const definition = groupDefinitions.find(({ id }) => id === group);
    return definition ? translate(definition.labelKey) : group;
  }

  function workspaceForTopic(topicId) {
    return workspaceDefinitions.find((workspace) => (
      workspace.group === registry.topic(topicId)?.group && workspace.matches(topicId)
    )) || null;
  }

  function topicsForWorkspace(workspace) {
    return topicList.filter((topic) => topic.group === workspace.group && workspace.matches(topic.id));
  }

  function treeEntries(group) {
    const entries = [];
    const emittedWorkspaces = new Set();
    topicList.filter((topic) => topic.group === group).forEach((topic) => {
      if (topic.scope === "dialog") return;
      const workspace = workspaceForTopic(topic.id);
      if (!workspace) {
        entries.push({
          id: topic.id,
          topicId: topic.id,
          topicIds: [topic.id],
          label: registry.resolveDocument(language, topic.id)?.title || topic.id,
        });
        return;
      }
      if (emittedWorkspaces.has(workspace.id)) return;
      emittedWorkspaces.add(workspace.id);
      const workspaceTopics = topicsForWorkspace(workspace);
      entries.push({
        id: workspace.id,
        topicId: topicIds.has(workspace.defaultTopicId) ? workspace.defaultTopicId : workspaceTopics[0]?.id,
        topicIds: workspaceTopics.map(({ id }) => id),
        label: translate(workspace.labelKey),
      });
    });
    return entries.filter(({ topicId }) => topicId);
  }

  function activeContextTopics() {
    const workspace = workspaceForTopic(activeTopicId);
    return workspace ? topicsForWorkspace(workspace) : [registry.topic(activeTopicId)].filter(Boolean);
  }

  function blockMarkup(block) {
    if (block.type === "list") {
      return `<ul>${block.items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}</ul>`;
    }
    return `<p>${escapeHtml(block.text)}</p>`;
  }

  function renderChrome() {
    const languageMeta = localeRegistry.find(({ id }) => id === language);
    document.documentElement.lang = language;
    document.documentElement.dir = languageMeta?.direction || "ltr";
    document.querySelector("#help-product-label").textContent = translate("help.title");
    document.querySelector("#help-contents-eyebrow").textContent = translate("help.title");
    document.querySelector("#help-contents-title").textContent = translate("help.contents");
    document.querySelector("#help-language-label").textContent = translate("app.language");
    document.querySelector("#help-search-label").textContent = translate("navigation.search");
    document.querySelector("#help-wallet-label").textContent = translate("assets.wallet");
    document.querySelector("#help-home-link").href = routeUrl(registry.globalTopic()).href;
    menuButton.setAttribute("aria-label", translate("app.menu"));
    closeButton.setAttribute("aria-label", translate("common.close"));
    backdrop.setAttribute("aria-label", translate("common.close"));
    languageTrigger.setAttribute("aria-label", translate("app.language"));
    walletLink.setAttribute("aria-label", translate("assets.wallet"));
    tree.setAttribute("aria-label", translate("help.contents"));
    searchInput.setAttribute("placeholder", `${translate("navigation.search")}…`);
    searchInput.setAttribute("aria-label", translate("navigation.search"));
    searchClear.setAttribute("aria-label", translate("common.close"));
    contextTabs.setAttribute("aria-label", translate("help.contents"));
    const selectedLanguage = localeRegistry.find(({ id }) => id === language) || localeRegistry[0];
    languageTrigger.innerHTML = `<span class="help-language-current"><span aria-hidden="true">${escapeHtml(selectedLanguage.flag)}</span><span class="help-language-name">${escapeHtml(selectedLanguage.nativeName)}</span></span>${icon("chevron")}`;
    languageMenu.innerHTML = localeRegistry.map(({ id, nativeName, flag }) => (
      `<button class="help-language-option${id === language ? " is-selected" : ""}" type="button" role="option" aria-selected="${id === language}" tabindex="${id === language ? "0" : "-1"}" data-help-language-option="${escapeHtml(id)}"><span aria-hidden="true">${escapeHtml(flag)}</span><span>${escapeHtml(nativeName)}</span><i aria-hidden="true"></i></button>`
    )).join("");
  }

  function closeLanguagePicker({ restoreFocus = false } = {}) {
    if (!languagePicker.classList.contains("is-open")) return;
    languagePicker.classList.remove("is-open");
    languageMenu.hidden = true;
    languageMenu.removeAttribute("style");
    languageTrigger.setAttribute("aria-expanded", "false");
    if (restoreFocus) languageTrigger.focus();
  }

  function openLanguagePicker() {
    languagePicker.classList.add("is-open");
    languageTrigger.setAttribute("aria-expanded", "true");
    languageMenu.hidden = false;
    root.requestAnimationFrame(() => {
      if (!languagePicker.classList.contains("is-open")) return;
      const triggerRect = languageTrigger.getBoundingClientRect();
      const viewportPadding = 12;
      const menuHeight = Math.min(languageMenu.scrollHeight, 360);
      const spaceAbove = triggerRect.top - viewportPadding;
      const spaceBelow = root.innerHeight - triggerRect.bottom - viewportPadding;
      const opensUpward = spaceBelow < Math.min(menuHeight, 224) && spaceAbove > spaceBelow;
      const availableHeight = Math.max(128, opensUpward ? spaceAbove : spaceBelow);
      const width = Math.min(Math.max(triggerRect.width, 220), root.innerWidth - viewportPadding * 2);
      const left = Math.max(viewportPadding, Math.min(triggerRect.right - width, root.innerWidth - width - viewportPadding));
      languageMenu.style.left = `${Math.round(left)}px`;
      languageMenu.style.width = `${Math.round(width)}px`;
      languageMenu.style.maxHeight = `${Math.floor(availableHeight)}px`;
      if (opensUpward) {
        languageMenu.style.top = "auto";
        languageMenu.style.bottom = `${Math.max(viewportPadding, Math.round(root.innerHeight - triggerRect.top + 6))}px`;
      } else {
        languageMenu.style.top = `${Math.round(triggerRect.bottom + 6)}px`;
        languageMenu.style.bottom = "auto";
      }
    });
  }

  function selectLanguage(languageId) {
    language = i18n.resolveLanguage(languageId);
    root.history.pushState({}, "", routeUrl(activeTopicId, language, sectionTarget));
    render();
    root.requestAnimationFrame(() => languageTrigger.focus());
  }

  function searchableText(topic) {
    const documentData = registry.resolveDocument(language, topic.id);
    if (!documentData) return "";
    return [
      documentData.title,
      documentData.summary,
      ...documentData.sections.flatMap((section) => [
        section.title,
        ...section.blocks.flatMap((block) => block.type === "list" ? block.items : [block.text])
      ])
    ].join(" ").toLocaleLowerCase();
  }

  function renderSearch() {
    const normalized = searchQuery.trim().toLocaleLowerCase();
    const matches = normalized
      ? topicList.filter((topic) => searchableText(topic).includes(normalized))
      : [];
    tree.hidden = Boolean(normalized);
    searchResults.hidden = !normalized;
    searchClear.hidden = !searchQuery;
    searchStatus.textContent = normalized ? `${matches.length} ${translate("navigation.search")}` : "";
    searchResults.innerHTML = matches.length
      ? matches.map((topic) => {
          const documentData = registry.resolveDocument(language, topic.id);
          return `<a class="help-search-result" href="${escapeHtml(routeUrl(topic.id).href)}" data-help-search-topic="${escapeHtml(topic.id)}">
            ${icon(groupIcons[topic.group])}
            <span><strong>${escapeHtml(documentData?.title || topic.id)}</strong><small>${escapeHtml(groupLabel(topic.group))}</small></span>
          </a>`;
        }).join("")
      : `<p class="help-search-empty">${escapeHtml(translate("help.unavailable"))}</p>`;
  }

  function renderTree(focusGroup = "") {
    tree.innerHTML = groupOrder.map((group) => {
      const expanded = openGroups.has(group);
      const entries = treeEntries(group);
      const controlsId = `help-group-${group}`;
      return `<section class="help-tree-group">
        <button class="help-tree-group-button" type="button" data-help-group="${group}" aria-expanded="${expanded}" aria-controls="${controlsId}">
          ${icon(groupIcons[group])}
          <span>${escapeHtml(groupLabel(group))}</span>
          <small>${entries.length}</small>
          ${icon("chevron")}
        </button>
        <div class="help-tree-items" id="${controlsId}"${expanded ? "" : " hidden"}>
          ${entries.map((entry) => {
            const active = entry.topicIds.includes(activeTopicId);
            return `<a class="help-topic-link${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(entry.topicId).href)}" data-help-topic-link="${escapeHtml(entry.topicId)}"${active ? ' aria-current="page"' : ""}>${escapeHtml(entry.label)}</a>`;
          }).join("")}
        </div>
      </section>`;
    }).join("");
    if (focusGroup) {
      requestAnimationFrame(() => tree.querySelector(`[data-help-group="${focusGroup}"]`)?.focus());
    }
  }

  function renderContextTabs() {
    const topics = activeContextTopics();
    contextTabs.hidden = topics.length <= 1;
    main.classList.toggle("has-context-tabs", topics.length > 1);
    contextTabs.innerHTML = topics.map((topic) => {
      const documentData = registry.resolveDocument(language, topic.id);
      const active = topic.id === activeTopicId;
      return `<a class="help-context-tab${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(topic.id).href)}" data-help-context-topic="${escapeHtml(topic.id)}"${active ? ' aria-current="page"' : ""}>${escapeHtml(documentData?.title || topic.id)}</a>`;
    }).join("");
    requestAnimationFrame(() => {
      contextTabs.querySelector(".is-active")?.scrollIntoView({
        block: "nearest",
        inline: "center",
      });
    });
  }

  function renderDocument({ focusHeading = false } = {}) {
    const documentData = registry.resolveDocument(language, activeTopicId);
    if (!documentData) {
      article.innerHTML = `<header class="help-document-header"><h1 id="help-title" tabindex="-1">${escapeHtml(translate("help.unavailable"))}</h1></header>`;
      return;
    }
    const group = registry.topic(activeTopicId)?.group || "app";
    article.innerHTML = `
      <header class="help-document-header">
        <span class="eyebrow">${escapeHtml(groupLabel(group))}</span>
        <h1 id="help-title" tabindex="-1">${escapeHtml(documentData.title)}</h1>
        <p>${escapeHtml(documentData.summary)}</p>
      </header>
      <div class="help-document-sections">
        ${documentData.sections.map((section, index) => {
          const target = section.target || `section-${index + 1}`;
          return `<section class="help-document-section" id="${escapeHtml(target)}">
            <h2>${escapeHtml(section.title)}</h2>
            ${section.blocks.map(blockMarkup).join("")}
          </section>`;
        }).join("")}
      </div>`;
    document.title = `${documentData.title} · Z00Z ${translate("help.title")}`;
    requestAnimationFrame(() => {
      if (focusHeading) {
        article.querySelector("#help-title")?.focus({ preventScroll: true });
        root.scrollTo(0, 0);
      }
      if (sectionTarget) article.querySelector(`#${CSS.escape(sectionTarget)}`)?.scrollIntoView({ block: "start" });
    });
  }

  function render(options) {
    renderChrome();
    renderTree();
    renderSearch();
    renderContextTabs();
    renderDocument(options);
  }

  function setDrawer(open, { restoreFocus = false } = {}) {
    const mobile = root.matchMedia("(max-width: 767px)").matches;
    sidebar.classList.toggle("is-open", open);
    backdrop.hidden = !open;
    menuButton.setAttribute("aria-expanded", String(open));
    document.body.classList.toggle("has-help-drawer", open);
    sidebar.inert = mobile && !open;
    if (mobile && !open) sidebar.setAttribute("aria-hidden", "true");
    else sidebar.removeAttribute("aria-hidden");
    if (open) requestAnimationFrame(() => closeButton.focus());
    else if (restoreFocus) menuButton.focus();
  }

  function openTopic(topicId) {
    activeTopicId = topicId;
    sectionTarget = "";
    openGroups.add(registry.topic(activeTopicId).group);
    searchQuery = "";
    searchInput.value = "";
    root.history.pushState({}, "", routeUrl(activeTopicId));
    root.scrollTo(0, 0);
    render({ focusHeading: true });
    setDrawer(false);
  }

  tree.addEventListener("click", (event) => {
    const groupButton = event.target.closest("[data-help-group]");
    if (groupButton) {
      const group = groupButton.dataset.helpGroup;
      openGroups.has(group) ? openGroups.delete(group) : openGroups.add(group);
      renderTree(group);
      return;
    }
    const topicLink = event.target.closest("[data-help-topic-link]");
    if (!topicLink) return;
    event.preventDefault();
    openTopic(topicLink.dataset.helpTopicLink);
  });

  searchResults.addEventListener("click", (event) => {
    const topicLink = event.target.closest("[data-help-search-topic]");
    if (!topicLink) return;
    event.preventDefault();
    openTopic(topicLink.dataset.helpSearchTopic);
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

  contextTabs.addEventListener("click", (event) => {
    const topicLink = event.target.closest("[data-help-context-topic]");
    if (!topicLink) return;
    event.preventDefault();
    openTopic(topicLink.dataset.helpContextTopic);
  });

  languageTrigger.addEventListener("click", () => {
    if (languagePicker.classList.contains("is-open")) closeLanguagePicker();
    else openLanguagePicker();
  });
  languageMenu.addEventListener("click", (event) => {
    const option = event.target.closest("[data-help-language-option]");
    if (!option) return;
    selectLanguage(option.dataset.helpLanguageOption);
  });
  document.addEventListener("click", (event) => {
    if (!event.target.closest("[data-help-language-picker]")) closeLanguagePicker();
  });

  walletLink.addEventListener("click", (event) => {
    if (!root.opener || root.opener.closed) return;
    event.preventDefault();
    root.opener.focus();
  });
  menuButton.addEventListener("click", () => setDrawer(true));
  closeButton.addEventListener("click", () => setDrawer(false, { restoreFocus: true }));
  backdrop.addEventListener("click", () => setDrawer(false, { restoreFocus: true }));
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && languagePicker.classList.contains("is-open")) {
      event.preventDefault();
      closeLanguagePicker({ restoreFocus: true });
      return;
    }
    if (event.key === "Escape" && sidebar.classList.contains("is-open")) {
      event.preventDefault();
      setDrawer(false, { restoreFocus: true });
    }
  });
  root.addEventListener("popstate", () => {
    readRoute();
    render();
  });
  root.addEventListener("resize", () => {
    closeLanguagePicker();
    const open = sidebar.classList.contains("is-open");
    setDrawer(open);
  });

  readRoute();
  render();
  setDrawer(false);
})(window);
