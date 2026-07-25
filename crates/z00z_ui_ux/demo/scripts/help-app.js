"use strict";

((root) => {
  const registry = root.Z00ZHelpRegistry;
  const i18n = root.Z00ZI18n;
  const localeRegistry = root.Z00ZLocaleRegistry;
  if (!registry || !i18n || !localeRegistry?.length) {
    throw new Error("Standalone Help dependencies are missing.");
  }

  const groupOrder = Object.freeze(["app", "wallets", "network", "settings"]);
  const groupIcons = Object.freeze({
    app: "question",
    wallets: "wallet",
    network: "network",
    settings: "settings"
  });
  const topicList = registry.topics();
  const topicIds = new Set(topicList.map(({ id }) => id));
  const languageIds = new Set(localeRegistry.map(({ id }) => id));
  const tree = document.querySelector("#help-tree");
  const article = document.querySelector("#help-document");
  const main = document.querySelector("#help-main");
  const sidebar = document.querySelector("#help-sidebar");
  const menuButton = document.querySelector("#help-menu-button");
  const closeButton = document.querySelector("#help-sidebar-close");
  const backdrop = document.querySelector("#help-sidebar-backdrop");
  const languageSelect = document.querySelector("#help-language");
  const walletLink = document.querySelector(".help-wallet-link");
  const openGroups = new Set();
  let language = "en";
  let activeTopicId = registry.globalTopic();
  let sectionTarget = "";

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
    document.documentElement.dataset.theme = ["dark", "light"].includes(parameters.get("theme"))
      ? parameters.get("theme")
      : "dark";
    document.documentElement.dataset.palette = /^[a-z0-9-]+$/.test(parameters.get("palette") || "")
      ? parameters.get("palette")
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
    if (group === "app") {
      return registry.resolveDocument(language, registry.globalTopic())?.title || translate("help.title");
    }
    if (group === "wallets") return translate("app.wallets");
    if (group === "network") return translate("app.network");
    return translate("app.settings");
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
    document.querySelector("#help-wallet-label").textContent = translate("assets.wallet");
    document.querySelector("#help-home-link").href = routeUrl(registry.globalTopic()).href;
    menuButton.setAttribute("aria-label", translate("app.menu"));
    closeButton.setAttribute("aria-label", translate("common.close"));
    backdrop.setAttribute("aria-label", translate("common.close"));
    languageSelect.setAttribute("aria-label", translate("app.language"));
    walletLink.setAttribute("aria-label", translate("assets.wallet"));
    tree.setAttribute("aria-label", translate("help.contents"));
    languageSelect.innerHTML = localeRegistry.map(({ id, nativeName }) => (
      `<option value="${escapeHtml(id)}"${id === language ? " selected" : ""}>${escapeHtml(nativeName)}</option>`
    )).join("");
  }

  function renderTree(focusGroup = "") {
    tree.innerHTML = groupOrder.map((group) => {
      const expanded = openGroups.has(group);
      const topics = topicList.filter((topic) => topic.group === group);
      const controlsId = `help-group-${group}`;
      return `<section class="help-tree-group">
        <button class="help-tree-group-button" type="button" data-help-group="${group}" aria-expanded="${expanded}" aria-controls="${controlsId}">
          ${icon(groupIcons[group])}
          <span>${escapeHtml(groupLabel(group))}</span>
          <small>${topics.length}</small>
          ${icon("chevron")}
        </button>
        <div class="help-tree-items" id="${controlsId}"${expanded ? "" : " hidden"}>
          ${topics.map((topic) => {
            const documentData = registry.resolveDocument(language, topic.id);
            const active = topic.id === activeTopicId;
            return `<a class="help-topic-link${active ? " is-active" : ""}" href="${escapeHtml(routeUrl(topic.id).href)}" data-help-topic-link="${escapeHtml(topic.id)}"${active ? ' aria-current="page"' : ""}>${escapeHtml(documentData?.title || topic.id)}</a>`;
          }).join("")}
        </div>
      </section>`;
    }).join("");
    if (focusGroup) {
      requestAnimationFrame(() => tree.querySelector(`[data-help-group="${focusGroup}"]`)?.focus());
    }
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
      if (focusHeading) article.querySelector("#help-title")?.focus();
      if (sectionTarget) article.querySelector(`#${CSS.escape(sectionTarget)}`)?.scrollIntoView({ block: "start" });
    });
  }

  function render(options) {
    renderChrome();
    renderTree();
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
    activeTopicId = topicLink.dataset.helpTopicLink;
    sectionTarget = "";
    openGroups.add(registry.topic(activeTopicId).group);
    root.history.pushState({}, "", routeUrl(activeTopicId));
    render({ focusHeading: true });
    setDrawer(false);
  });

  languageSelect.addEventListener("change", () => {
    language = i18n.resolveLanguage(languageSelect.value);
    root.history.pushState({}, "", routeUrl(activeTopicId, language, sectionTarget));
    render();
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
    const open = sidebar.classList.contains("is-open");
    setDrawer(open);
  });

  readRoute();
  render();
  setDrawer(false);
})(window);
