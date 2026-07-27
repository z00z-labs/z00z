"use strict";

((root) => {
  const MERMAID_KEYBOARD_PAN_DISTANCE = 48;
  const MERMAID_THEME_CONFIG = {
    fontFamily: "Trebuchet MS, Verdana, Arial, sans-serif",
    startOnLoad: false,
    theme: "base",
    themeVariables: {
      activationBkgColor: "#FFF3E0",
      activationBorderColor: "#FB8C00",
      actorBkg: "#E3F2FD",
      actorBorder: "#1E88E5",
      actorLineColor: "#000000",
      actorTextColor: "#0D47A1",
      altSectionBkgColor: "#ECEFF1",
      background: "#FFFFFF",
      clusterBkg: "#ECEFF1",
      clusterBorder: "#546E7A",
      critBkgColor: "#FFE0E0",
      critBorderColor: "#D32F2F",
      doneTaskBkgColor: "#E8F5E9",
      doneTaskBorderColor: "#43A047",
      edgeLabelBackground: "#FFFFFF",
      gridColor: "#D0D7DE",
      labelBoxBkgColor: "#FFFFFF",
      labelTextColor: "#263238",
      lineColor: "#000000",
      mainBkg: "#F3E5F5",
      nodeBorder: "#8E24AA",
      noteBkgColor: "#E8F5E9",
      noteTextColor: "#1B5E20",
      primaryBorderColor: "#8E24AA",
      primaryColor: "#F3E5F5",
      primaryTextColor: "#4A148C",
      secondaryBorderColor: "#1E88E5",
      secondaryColor: "#E3F2FD",
      secondaryTextColor: "#0D47A1",
      sectionBkgColor: "#F3E5F5",
      signalColor: "#000000",
      signalTextColor: "#263238",
      tertiaryBorderColor: "#FB8C00",
      tertiaryColor: "#FFF3E0",
      tertiaryTextColor: "#E65100",
      titleColor: "#263238",
    },
  };
  const mermaidPanzoomBindings = new Map();
  let mermaidSequence = 0;

  function readMermaidSource(node) {
    const value = node.dataset.mermaidDefinition || node.dataset.mermaidSource || node.textContent || "";
    try {
      return decodeURIComponent(value).trim();
    } catch {
      return value.trim();
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
      buttons.forEach((button, index) => {
        button.onclick = () => activate(index);
      });
      activate(Math.max(0, buttons.findIndex((button) => button.classList.contains("tabs-nav-btn-active"))));
    });
  }

  function initializeTableOfContents(container) {
    container.querySelectorAll(".table-of-contents-link[href^='#']").forEach((link) => {
      link.onclick = (event) => {
        const id = link.getAttribute("href")?.slice(1);
        const target = id && container.querySelector(`#${CSS.escape(id)}`);
        if (!target) return;
        event.preventDefault();
        target.scrollIntoView({ block: "start" });
        root.history.replaceState({}, "", `#${encodeURIComponent(id)}`);
        target.focus({ preventScroll: true });
      };
    });
  }

  function cleanupMermaidPanzoom(node) {
    const binding = mermaidPanzoomBindings.get(node);
    if (!binding) return;
    binding.frame.removeEventListener("wheel", binding.handleWheel);
    binding.frame.removeEventListener("dblclick", binding.handleDoubleClick);
    binding.frame.removeEventListener("keydown", binding.handleKeyDown);
    binding.panzoom.destroy();
    binding.panzoom.resetStyle();
    mermaidPanzoomBindings.delete(node);
  }

  function cleanupDetachedMermaidPanzoom() {
    for (const node of mermaidPanzoomBindings.keys()) {
      if (!node.isConnected) cleanupMermaidPanzoom(node);
    }
  }

  function getFrameContentWidth(frame) {
    const style = root.getComputedStyle(frame);
    const leftPadding = Number.parseFloat(style.getPropertyValue("padding-left")) || 0;
    const rightPadding = Number.parseFloat(style.getPropertyValue("padding-right")) || 0;
    return Math.max(1, frame.clientWidth - leftPadding - rightPadding);
  }

  function getSvgLength(value) {
    if (!value) return null;
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
  }

  function ensureMermaidSvgViewport(svg) {
    const existingViewBox = svg.viewBox.baseVal;
    if (existingViewBox?.width > 0 && existingViewBox.height > 0) {
      return { height: existingViewBox.height, width: existingViewBox.width };
    }

    const bboxTarget = svg.querySelector("g") || svg;
    try {
      const bbox = bboxTarget.getBBox();
      if (bbox.width > 0 && bbox.height > 0) {
        const padding = 24;
        const viewBoxX = Math.floor(bbox.x - padding);
        const viewBoxY = Math.floor(bbox.y - padding);
        const viewBoxWidth = Math.ceil(bbox.width + padding * 2);
        const viewBoxHeight = Math.ceil(bbox.height + padding * 2);
        svg.setAttribute("viewBox", `${viewBoxX} ${viewBoxY} ${viewBoxWidth} ${viewBoxHeight}`);
        svg.setAttribute("preserveAspectRatio", "xMidYMid meet");
        if (!getSvgLength(svg.getAttribute("height"))) {
          svg.setAttribute("height", String(viewBoxHeight));
        }
        return { height: viewBoxHeight, width: viewBoxWidth };
      }
    } catch {
      // Fall through to measured dimensions.
    }

    const fallbackWidth = getSvgLength(svg.getAttribute("width")) || svg.getBoundingClientRect().width;
    const fallbackHeight = getSvgLength(svg.getAttribute("height")) || svg.getBoundingClientRect().height;
    return {
      height: fallbackHeight > 0 ? fallbackHeight : 1,
      width: fallbackWidth > 0 ? fallbackWidth : 1,
    };
  }

  function applyMermaidFitLayout(binding) {
    const viewport = ensureMermaidSvgViewport(binding.svg);
    const layoutScale = Math.min(1, getFrameContentWidth(binding.frame) / viewport.width);
    binding.svg.style.width = `${Math.max(1, viewport.width * layoutScale)}px`;
    binding.svg.style.height = `${Math.max(1, viewport.height * layoutScale)}px`;
    binding.svg.style.maxWidth = "none";
    binding.svg.style.marginInline = "auto";
  }

  function fitMermaidPanzoom(binding) {
    if (binding.frame.clientWidth <= 0) return;
    applyMermaidFitLayout(binding);
    binding.panzoom.setOptions({
      panOnlyWhenZoomed: true,
      startScale: 1,
      startX: 0,
      startY: 0,
    });
    binding.panzoom.reset({ animate: false });
  }

  function bindMermaidPanzoom(node) {
    const svg = node.querySelector("svg");
    if (!svg || typeof root.Panzoom !== "function") return;

    cleanupMermaidPanzoom(node);

    const frame = root.document.createElement("div");
    frame.className = "mermaid-panzoom-frame";
    frame.tabIndex = 0;
    frame.setAttribute("role", "region");
    frame.setAttribute(
      "aria-label",
      "Interactive Mermaid diagram. Drag to pan, wheel or pinch to zoom, or use the arrow keys to pan. Press plus or minus to zoom and zero to reset.",
    );
    frame.setAttribute("aria-keyshortcuts", "ArrowUp ArrowDown ArrowLeft ArrowRight + - 0");

    const hint = root.document.createElement("p");
    hint.className = "mermaid-panzoom-hint";
    hint.textContent =
      "Controls: drag to pan, wheel or pinch to zoom, double-click or press 0 to reset. Keyboard: Arrow keys pan; + and − zoom.";

    const svgViewport = ensureMermaidSvgViewport(svg);
    svg.removeAttribute("height");
    svg.style.width = `${svgViewport.width}px`;
    svg.style.height = `${svgViewport.height}px`;
    svg.style.maxWidth = "none";
    svg.style.marginInline = "auto";
    svg.style.display = "block";
    svg.classList.add("mermaid-panzoom-svg");
    node.replaceChildren(frame, hint);
    frame.appendChild(svg);

    const panzoom = root.Panzoom(svg, {
      canvas: true,
      cursor: "grab",
      maxScale: 24,
      minScale: 0.1,
      overflow: "hidden",
      panOnlyWhenZoomed: true,
      roundPixels: true,
      step: 0.15,
      touchAction: "none",
    });
    const binding = {
      frame,
      handleDoubleClick: null,
      handleKeyDown: null,
      handleWheel: null,
      panzoom,
      svg,
    };

    binding.handleWheel = (event) => {
      event.preventDefault();
      panzoom.zoomWithWheel(event);
    };
    binding.handleDoubleClick = () => {
      ensureMermaidSvgViewport(binding.svg);
      fitMermaidPanzoom(binding);
    };
    binding.handleKeyDown = (event) => {
      const pan = (x, y) => panzoom.pan(x, y, { animate: false, force: true, relative: true });
      let handled = true;
      switch (event.key) {
        case "ArrowDown":
          pan(0, -MERMAID_KEYBOARD_PAN_DISTANCE);
          break;
        case "ArrowLeft":
          pan(MERMAID_KEYBOARD_PAN_DISTANCE, 0);
          break;
        case "ArrowRight":
          pan(-MERMAID_KEYBOARD_PAN_DISTANCE, 0);
          break;
        case "ArrowUp":
          pan(0, MERMAID_KEYBOARD_PAN_DISTANCE);
          break;
        case "+":
        case "=":
          panzoom.zoomIn({ animate: false });
          break;
        case "-":
        case "_":
          panzoom.zoomOut({ animate: false });
          break;
        case "0":
          ensureMermaidSvgViewport(binding.svg);
          fitMermaidPanzoom(binding);
          break;
        default:
          handled = false;
      }
      if (handled) event.preventDefault();
    };

    frame.addEventListener("wheel", binding.handleWheel, { passive: false });
    frame.addEventListener("dblclick", binding.handleDoubleClick);
    frame.addEventListener("keydown", binding.handleKeyDown);
    mermaidPanzoomBindings.set(node, binding);
    fitMermaidPanzoom(binding);

    const refreshViewport = () => {
      const currentBinding = mermaidPanzoomBindings.get(node);
      if (!currentBinding) return;
      ensureMermaidSvgViewport(currentBinding.svg);
      fitMermaidPanzoom(currentBinding);
    };
    root.requestAnimationFrame(() => {
      refreshViewport();
      root.requestAnimationFrame(refreshViewport);
    });
  }

  async function waitForDocumentFonts() {
    if (!("fonts" in root.document)) return;
    try {
      await root.document.fonts.ready;
    } catch {
      // Mermaid can still render with fallback font metrics.
    }
  }

  async function initializeMermaid(container) {
    const nodes = [...container.querySelectorAll(".mermaid")];
    if (!nodes.length || !root.mermaid) return;
    cleanupDetachedMermaidPanzoom();
    await waitForDocumentFonts();
    root.mermaid.initialize(MERMAID_THEME_CONFIG);

    for (const node of nodes) {
      const source = readMermaidSource(node);
      cleanupMermaidPanzoom(node);
      if (source) {
        node.dataset.mermaidSource = source;
        node.textContent = source;
      }
      node.removeAttribute("data-processed");
    }

    for (const node of nodes) {
      const source = node.dataset.mermaidSource?.trim();
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

    nodes.forEach(bindMermaidPanzoom);
  }

  root.addEventListener("resize", () => {
    root.requestAnimationFrame(() => {
      mermaidPanzoomBindings.forEach(fitMermaidPanzoom);
    });
  });

  root.Z00ZHelpMarkdownEnhancer = Object.freeze({
    enhance(container) {
      initializeTabs(container);
      initializeTableOfContents(container);
      return initializeMermaid(container);
    },
  });
})(typeof window === "undefined" ? globalThis : window);
