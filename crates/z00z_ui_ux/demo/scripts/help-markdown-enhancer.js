"use strict";

((root) => {
  let mermaidSequence = 0;
  let mermaidInitialized = false;

  function readMermaidSource(node) {
    try {
      return decodeURIComponent(node.dataset.mermaidDefinition || node.textContent || "").trim();
    } catch {
      return (node.dataset.mermaidDefinition || node.textContent || "").trim();
    }
  }

  function initializeTabs(container) {
    container.querySelectorAll(".tabs-block").forEach((block) => {
      const buttons = [...block.querySelectorAll(".tabs-nav-btn")];
      const panels = [...block.querySelectorAll(".tabs-panel")];
      if (!buttons.length || buttons.length !== panels.length) return;
      const activate = (activeIndex) => {
        buttons.forEach((button, index) => {
          const active = index === activeIndex;
          button.classList.toggle("tabs-nav-btn-active", active);
          button.classList.toggle("tabs-nav-btn-inactive", !active);
          button.setAttribute("aria-selected", String(active));
        });
        panels.forEach((panel, index) => {
          const active = index === activeIndex;
          panel.classList.toggle("tabs-panel-active", active);
          panel.classList.toggle("tabs-panel-hidden", !active);
          panel.setAttribute("aria-expanded", String(active));
        });
      };
      buttons.forEach((button, index) => button.addEventListener("click", () => activate(index)));
      activate(Math.max(0, buttons.findIndex((button) => button.classList.contains("tabs-nav-btn-active"))));
    });
  }

  function initializeTableOfContents(container) {
    container.querySelectorAll(".table-of-contents-link[href^='#']").forEach((link) => {
      link.addEventListener("click", (event) => {
        const id = link.getAttribute("href")?.slice(1);
        const target = id && container.querySelector(`#${CSS.escape(id)}`);
        if (!target) return;
        event.preventDefault();
        target.scrollIntoView({ block: "start" });
        root.history.replaceState({}, "", `#${encodeURIComponent(id)}`);
        target.focus({ preventScroll: true });
      });
    });
  }

  async function initializeMermaid(container) {
    const nodes = [...container.querySelectorAll(".mermaid")];
    if (!nodes.length || !root.mermaid) return;
    if (!mermaidInitialized) {
      root.mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: "base" });
      mermaidInitialized = true;
    }
    for (const node of nodes) {
      const source = readMermaidSource(node);
      if (!source) continue;
      try {
        const { bindFunctions, svg } = await root.mermaid.render(`z00z-help-mermaid-${mermaidSequence++}`, source);
        node.innerHTML = svg;
        node.dataset.mermaidRendered = "true";
        bindFunctions?.(node);
      } catch {
        node.dataset.mermaidRendered = "fallback";
        node.textContent = source;
      }
    }
  }

  root.Z00ZHelpMarkdownEnhancer = Object.freeze({
    enhance(container) {
      initializeTabs(container);
      initializeTableOfContents(container);
      void initializeMermaid(container);
    },
  });
})(typeof window === "undefined" ? globalThis : window);
