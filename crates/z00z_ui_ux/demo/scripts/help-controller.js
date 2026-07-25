"use strict";

((root) => {
  const registry = root.Z00ZHelpRegistry;
  const i18n = root.Z00ZI18n;
  const contextHelpHost = document.querySelector("#context-help-host");
  const walletStatusbar = document.querySelector("#wallet-statusbar");
  if (!registry || !i18n || !contextHelpHost) {
    throw new Error("Help launcher dependencies are missing.");
  }

  let language = "en";
  let theme = "dark";
  let palette = "z00z-default";

  if (!root.name) root.name = "z00z-wallet";

  function helpUrl(topicId) {
    const current = new URL(root.location.href);
    const url = new URL("help.html", current);
    url.search = "";
    url.hash = "";
    url.searchParams.set("topic", topicId);
    url.searchParams.set("lang", language);
    url.searchParams.set("theme", theme);
    url.searchParams.set("palette", palette);
    if (topicId !== registry.globalTopic()) url.searchParams.set("section", "current-view");
    const releaseVersion = current.searchParams.get("v");
    if (releaseVersion) url.searchParams.set("v", releaseVersion);
    return url.href;
  }

  function open(topicId) {
    const resolved = registry.hasTopic(topicId) ? topicId : registry.globalTopic();
    document.dispatchEvent(new CustomEvent("z00z:help-opening"));
    const helpWindow = root.open(helpUrl(resolved), "z00z-help");
    helpWindow?.focus();
    return Boolean(helpWindow);
  }

  function configure(options = {}) {
    language = i18n.resolveLanguage(options.language || language);
    theme = ["dark", "light"].includes(options.theme) ? options.theme : theme;
    palette = /^[a-z0-9-]+$/.test(options.palette || "") ? options.palette : palette;
  }

  function mountContextButton(state, viewRoot) {
    contextHelpHost.replaceChildren();
    const topicId = registry.resolveTopicId(state);
    if (!topicId || !viewRoot) return;
    const button = document.createElement("button");
    button.type = "button";
    button.className = "icon-button context-help-button";
    button.dataset.helpTopic = topicId;
    button.dataset.contextHelpButton = "";
    button.setAttribute("aria-label", i18n.translate(language, "help.openContext"));
    button.setAttribute("title", i18n.translate(language, "help.openContext"));
    button.innerHTML = '<svg class="icon"><use href="#i-question"/></svg>';
    contextHelpHost.classList.toggle("has-statusbar", !walletStatusbar?.hidden);
    contextHelpHost.append(button);
  }

  document.addEventListener("click", (event) => {
    const trigger = event.target.closest("[data-help-topic]");
    if (!trigger) return;
    event.preventDefault();
    open(trigger.dataset.helpTopic);
  });

  root.Z00ZHelp = Object.freeze({
    configure,
    open,
    close: () => {},
    mountContextButton,
    resolveTopicId: registry.resolveTopicId,
    urlFor: (topicId) => helpUrl(registry.hasTopic(topicId) ? topicId : registry.globalTopic())
  });
})(window);
