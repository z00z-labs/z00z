"use strict";

((root) => {
  const demo = root.Z00ZDemo ||= {};
  if (!demo.ICON_NAMES) {
    throw new Error("The canonical icon sprite must load before UI primitives.");
  }

  const VIEWPORT_QUERY_LUT = Object.freeze({
    mobileNavigation: "(max-width: 768px)",
    helpDesktopToc: "(min-width: 1280px)",
    reducedMotion: "(prefers-reduced-motion: reduce)"
  });

  const DRAWER_GESTURE_LUT = Object.freeze({
    edge: 48,
    distance: 56
  });

  const MERMAID_INTERACTION_LUT = Object.freeze({
    keyboardPanDistance: 48
  });

  const FLOATING_PANEL_LUT = Object.freeze({
    control: Object.freeze({
      viewportPadding: 12,
      maxContentHeight: 360,
      openThreshold: 224,
      minAvailableHeight: 128,
      minWidth: 220,
      maxWidth: Number.POSITIVE_INFINITY,
      gap: 6,
      horizontalAlignment: "end"
    }),
    walletDesktop: Object.freeze({
      viewportPadding: 12,
      maxContentHeight: 280,
      openThreshold: 176,
      minAvailableHeight: 156,
      minWidth: 252,
      maxWidth: 288,
      gap: 8,
      horizontalAlignment: "start"
    }),
    walletMobile: Object.freeze({
      viewportPadding: 8,
      maxContentHeight: 280,
      openThreshold: 176,
      minAvailableHeight: 156,
      minWidth: 240,
      maxWidth: 300,
      gap: 8,
      horizontalAlignment: "start"
    })
  });

  const LANGUAGE_PICKER_DOM_LUT = Object.freeze({
    app: Object.freeze({
      controlId: "language-picker-options",
      wrapperAttribute: "data-language-picker",
      triggerAttribute: "data-language-picker-trigger",
      menuAttribute: "data-language-picker-menu",
      optionAttribute: "data-language-picker-option"
    }),
    help: Object.freeze({
      controlId: "help-language-options",
      wrapperAttribute: "data-help-language-picker",
      triggerAttribute: "data-help-language-trigger",
      menuAttribute: "data-help-language-options",
      optionAttribute: "data-help-language-option"
    })
  });

  function escapeHtml(value) {
    return String(value ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#039;");
  }

  function iconMarkup(name, className = "") {
    const resolvedName = demo.ICON_NAMES.includes(name) ? name : "question";
    const classes = ["icon", className].filter(Boolean).join(" ");
    return `<svg class="${classes}" aria-hidden="true"><use href="#i-${resolvedName}"/></svg>`;
  }

  function languagePickerMarkup({
    languages,
    language,
    label,
    variant = "app",
    className = ""
  }) {
    const dom = LANGUAGE_PICKER_DOM_LUT[variant];
    if (!dom) throw new Error(`Unknown language-picker variant: ${variant}`);
    const selected = languages.find(({ id }) => id === language) || languages[0];
    const wrapperClasses = ["language-picker", className].filter(Boolean).join(" ");
    return `<div class="${wrapperClasses}" ${dom.wrapperAttribute}>
      <button class="language-picker-trigger" type="button" ${dom.triggerAttribute} aria-label="${escapeHtml(label)}" aria-haspopup="listbox" aria-expanded="false" aria-controls="${dom.controlId}">
        <span class="language-picker-value"><span aria-hidden="true">${escapeHtml(selected.flag)}</span><span>${escapeHtml(selected.nativeName)}</span></span>
        ${iconMarkup("chevron")}
      </button>
      <div class="language-picker-menu" id="${dom.controlId}" ${dom.menuAttribute} role="listbox" aria-label="${escapeHtml(label)}" hidden>
        ${languages.map(({ id, nativeName, flag }) => `<button class="language-picker-option${id === language ? " is-selected" : ""}" type="button" role="option" aria-selected="${id === language}" tabindex="${id === language ? "0" : "-1"}" ${dom.optionAttribute}="${escapeHtml(id)}"><span aria-hidden="true">${escapeHtml(flag)}</span><span>${escapeHtml(nativeName)}</span>${id === language ? iconMarkup("check") : ""}</button>`).join("")}
      </div>
    </div>`;
  }

  function matchesViewport(name) {
    const query = VIEWPORT_QUERY_LUT[name];
    if (!query) throw new Error(`Unknown viewport query: ${name}`);
    return root.matchMedia(query).matches;
  }

  function positionFloatingPanel(panel, trigger, {
    profile = "control",
    anchor = trigger
  } = {}) {
    const config = FLOATING_PANEL_LUT[profile];
    if (!config) throw new Error(`Unknown floating-panel profile: ${profile}`);
    const triggerRect = trigger.getBoundingClientRect();
    const anchorRect = anchor.getBoundingClientRect();
    const spaceAbove = triggerRect.top - config.viewportPadding;
    const spaceBelow = root.innerHeight - triggerRect.bottom - config.viewportPadding;
    const contentHeight = Math.min(panel.scrollHeight, config.maxContentHeight);
    const opensUpward = spaceBelow < Math.min(contentHeight, config.openThreshold)
      && spaceAbove > spaceBelow;
    const availableHeight = Math.max(
      config.minAvailableHeight,
      opensUpward ? spaceAbove : spaceBelow
    );
    const viewportWidth = root.innerWidth - config.viewportPadding * 2;
    const width = Math.min(
      Math.max(anchorRect.width, config.minWidth),
      config.maxWidth,
      viewportWidth
    );
    const preferredLeft = config.horizontalAlignment === "end"
      ? triggerRect.right - width
      : triggerRect.left;
    const left = Math.max(
      config.viewportPadding,
      Math.min(preferredLeft, root.innerWidth - width - config.viewportPadding)
    );

    panel.style.left = `${Math.round(left)}px`;
    panel.style.width = `${Math.round(width)}px`;
    panel.style.maxHeight = `${Math.floor(availableHeight)}px`;
    panel.style.top = opensUpward ? "auto" : `${Math.round(triggerRect.bottom + config.gap)}px`;
    panel.style.bottom = opensUpward
      ? `${Math.max(config.viewportPadding, Math.round(root.innerHeight - triggerRect.top + config.gap))}px`
      : "auto";
    return Object.freeze({ opensUpward, width, availableHeight });
  }

  Object.assign(demo, {
    VIEWPORT_QUERY_LUT,
    DRAWER_GESTURE_LUT,
    MERMAID_INTERACTION_LUT,
    FLOATING_PANEL_LUT,
    LANGUAGE_PICKER_DOM_LUT,
    escapeHtml,
    iconMarkup,
    languagePickerMarkup,
    matchesViewport,
    positionFloatingPanel
  });
})(typeof window === "undefined" ? globalThis : window);
