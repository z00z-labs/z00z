"use strict";

((root) => {
  const registry = root.Z00ZHelpRegistry;
  const i18n = root.Z00ZI18n;
  const host = document.querySelector("#help-host");
  const contextHelpHost = document.querySelector("#context-help-host");
  const appShell = document.querySelector("#app-shell");
  const walletStatusbar = document.querySelector("#wallet-statusbar");
  const dialogContent = document.querySelector("#dialog-content");
  if (!registry || !i18n || !host || !contextHelpHost) {
    throw new Error("Help controller dependencies are missing.");
  }

  const hostParent = host.parentNode;
  const hostNextSibling = host.nextSibling;
  let language = "en";
  let activeTopicId = "";
  let activeSectionIndex = 0;
  let returnFocus = null;
  let highlightedTarget = null;
  let highlightScope = document;
  let appShellWasInert = false;
  let dialogContentWasInert = false;

  const escapeHtml = (value) => String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");

  const translate = (key, values) => i18n.translate(language, key, values);

  function clearHighlight() {
    highlightedTarget?.classList.remove("is-help-highlighted");
    highlightedTarget = null;
  }

  function restoreHost() {
    if (host.parentNode === hostParent) return;
    hostParent.insertBefore(host, hostNextSibling);
  }

  function setBackgroundInert(inert) {
    if (inert) {
      appShellWasInert = Boolean(appShell?.inert);
      dialogContentWasInert = Boolean(dialogContent?.inert);
      if (appShell) appShell.inert = true;
      if (dialogContent) dialogContent.inert = true;
      return;
    }
    if (appShell) appShell.inert = appShellWasInert;
    if (dialogContent) dialogContent.inert = dialogContentWasInert;
  }

  function focusableElements() {
    return [...host.querySelectorAll('button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])')]
      .filter((element) => element.tabIndex >= 0 && !element.hidden && element.getClientRects().length > 0);
  }

  function highlight(targetId) {
    clearHighlight();
    if (!targetId) return;
    highlightedTarget = highlightScope.querySelector(`[data-help-anchor="${CSS.escape(targetId)}"]`);
    if (!highlightedTarget) return;
    highlightedTarget.classList.add("is-help-highlighted");
  }

  function blockMarkup(block) {
    if (block.type === "list") {
      return `<ul>${block.items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}</ul>`;
    }
    return `<p>${escapeHtml(block.text)}</p>`;
  }

  function render() {
    const documentData = registry.resolveDocument(language, activeTopicId);
    if (!documentData) {
      host.innerHTML = `<button class="help-backdrop" type="button" data-help-close tabindex="-1" aria-label="${escapeHtml(translate("help.close"))}"></button>
        <section class="help-panel" role="dialog" aria-modal="true" aria-labelledby="help-title">
          <header class="help-header"><div><span class="eyebrow">${escapeHtml(translate("help.title"))}</span><h2 id="help-title">${escapeHtml(translate("help.unavailable"))}</h2></div><button class="icon-button" type="button" data-help-close aria-label="${escapeHtml(translate("help.close"))}"><svg class="icon"><use href="#i-close"/></svg></button></header>
        </section>`;
      return;
    }

    const sections = documentData.sections;
    const active = sections[Math.min(activeSectionIndex, sections.length - 1)];
    activeSectionIndex = sections.indexOf(active);
    host.innerHTML = `
      <button class="help-backdrop" type="button" data-help-close tabindex="-1" aria-label="${escapeHtml(translate("help.close"))}"></button>
      <section class="help-panel" role="dialog" aria-modal="true" aria-labelledby="help-title" aria-describedby="help-summary">
        <header class="help-header">
          <div><span class="eyebrow">${escapeHtml(translate("help.title"))}</span><h2 id="help-title">${escapeHtml(documentData.title)}</h2><p id="help-summary">${escapeHtml(documentData.summary)}</p></div>
          <button class="icon-button" type="button" data-help-close aria-label="${escapeHtml(translate("help.close"))}"><svg class="icon"><use href="#i-close"/></svg></button>
        </header>
        <div class="help-body">
          <nav class="help-contents" aria-label="${escapeHtml(translate("help.contents"))}">
            ${sections.map((section, index) => `<button class="${index === activeSectionIndex ? "is-active" : ""}" type="button" data-help-section="${index}" ${index === activeSectionIndex ? 'aria-current="true"' : ""}>${escapeHtml(section.title)}</button>`).join("")}
          </nav>
          <article class="help-article" aria-live="polite">
            <span class="help-section-count">${escapeHtml(translate("help.section", { current: activeSectionIndex + 1, total: sections.length }))}</span>
            <h3>${escapeHtml(active.title)}</h3>
            ${active.blocks.map(blockMarkup).join("")}
          </article>
        </div>
      </section>`;
    highlight(active.target);
  }

  function close({ restoreFocus = true } = {}) {
    if (!activeTopicId) return;
    activeTopicId = "";
    activeSectionIndex = 0;
    host.hidden = true;
    host.innerHTML = "";
    document.body.classList.remove("has-help-open");
    clearHighlight();
    highlightScope = document;
    setBackgroundInert(false);
    restoreHost();
    if (restoreFocus && returnFocus?.isConnected) returnFocus.focus();
    returnFocus = null;
  }

  function open(topicId, trigger = document.activeElement) {
    const resolved = registry.hasTopic(topicId) ? topicId : registry.globalTopic();
    document.dispatchEvent(new CustomEvent("z00z:help-opening"));
    activeTopicId = resolved;
    activeSectionIndex = 0;
    returnFocus = trigger instanceof HTMLElement ? trigger : null;
    const owningDialog = returnFocus?.closest("dialog[open]");
    highlightScope = owningDialog || document;
    if (owningDialog) owningDialog.append(host);
    else restoreHost();
    setBackgroundInert(true);
    host.hidden = false;
    document.body.classList.add("has-help-open");
    render();
    requestAnimationFrame(() => host.querySelector(".help-panel [data-help-close]")?.focus());
  }

  function configure(options = {}) {
    language = i18n.resolveLanguage(options.language || language);
    if (activeTopicId) render();
  }

  function mountContextButton(state, viewRoot) {
    contextHelpHost?.replaceChildren();
    document.querySelectorAll("[data-help-context-root]").forEach((root) => {
      root.removeAttribute("data-help-context-root");
    });
    const topicId = registry.resolveTopicId(state);
    if (!topicId || !viewRoot) return;
    viewRoot.setAttribute("data-help-anchor", "current-view");
    viewRoot.setAttribute("data-help-context-root", "");
    const button = document.createElement("button");
    button.type = "button";
    button.className = "icon-button context-help-button";
    button.dataset.helpTopic = topicId;
    button.dataset.contextHelpButton = "";
    button.setAttribute("aria-label", translate("help.openContext"));
    button.setAttribute("title", translate("help.openContext"));
    button.innerHTML = '<svg class="icon"><use href="#i-question"/></svg>';
    contextHelpHost?.classList.toggle("has-statusbar", !walletStatusbar?.hidden);
    contextHelpHost?.append(button);
  }

  host.addEventListener("click", (event) => {
    if (event.target.closest("[data-help-close]")) {
      close();
      return;
    }
    const sectionButton = event.target.closest("[data-help-section]");
    if (sectionButton) {
      activeSectionIndex = Number(sectionButton.dataset.helpSection);
      render();
      requestAnimationFrame(() => host.querySelector(`[data-help-section="${activeSectionIndex}"]`)?.focus());
    }
  });

  document.addEventListener("click", (event) => {
    const trigger = event.target.closest("[data-help-topic]");
    if (!trigger) return;
    event.preventDefault();
    open(trigger.dataset.helpTopic, trigger);
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && activeTopicId) {
      event.preventDefault();
      close();
      return;
    }
    if (event.key === "Tab" && activeTopicId) {
      const focusable = focusableElements();
      if (!focusable.length) return;
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

  root.Z00ZHelp = Object.freeze({
    configure,
    open,
    close,
    mountContextButton,
    resolveTopicId: registry.resolveTopicId
  });
})(window);
