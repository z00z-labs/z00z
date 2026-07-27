import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { renderHelpMarkdown } from "./help/markdown-renderer.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const websiteRoot = resolve(process.env.Z00Z_WEBSITE_ROOT || resolve(demoRoot, "../../../../z00z-website"));
const playwrightPath = pathToFileURL(resolve(websiteRoot, "node_modules/playwright/index.mjs")).href;
const { chromium } = await import(playwrightPath);
const contentPath = resolve(websiteRoot, "content/docs/learn/what-is-z00z.md");
const evidenceRoot = resolve(
  process.env.Z00Z_MARKDOWN_EVIDENCE_DIR
    || resolve(demoRoot, "../../z00z_storage/outputs/checkpoint/phase-110/markdown-parity-review"),
);
const helpUrl = process.env.Z00Z_HELP_URL || "http://127.0.0.1:4173/help.html";
const liveUrl = "https://www.z00z.io/docs/learn/what-is-z00z";
const executablePath = process.env.Z00Z_PLAYWRIGHT_EXECUTABLE_PATH || "/usr/bin/chromium";
const source = (await readFile(contentPath, "utf8")).replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/u, "");
const helpHtml = renderHelpMarkdown(source, contentPath);
const pluginFixtureHtml = renderHelpMarkdown(`
# Markdown extension matrix

[TOC]

*[HTML]: HyperText Markup Language

HTML linkify: https://z00z.io/website -- "typography".

> [!NOTE]
> Note alert

> [!IMPORTANT]
> Important alert

> [!TIP]
> Tip alert

> [!WARNING]
> Warning alert

> [!CAUTION]
> Caution alert

::: center
Aligned content
:::

## Attributes {#fixture-attributes .accent}

Term
: Definition

Footnote reference[^note].

[^note]: Footnote text.

++inserted++ ==marked== !!spoiler!! H~2~O x^2^ $x^2$

::: tabs
@tab First #first
First panel
@tab Second #second
Second panel
:::

- [x] Checked task

\`\`\`mermaid
flowchart LR
  Parse --> Render --> Enhance
\`\`\`

+++ Details
Hidden detail
+++

| Control | Purpose |
| --- | --- |
| Parser | Produces sanitized HTML |
| Enhancer | Adds tabs and Mermaid Panzoom |

\`\`\`js
const parity = true;
\`\`\`
`, contentPath);

await mkdir(evidenceRoot, { recursive: true });

const browser = await chromium.launch({ executablePath, headless: true });
const audit = {
  capturedAt: new Date().toISOString(),
  evidenceRoot,
  fontChanged: false,
  helpUrl,
  liveUrl,
  pages: {},
};

async function verifyPanzoom(page, selector) {
  const frame = page.locator(selector).first();
  const svg = frame.locator("svg");
  await frame.waitFor({ state: "visible", timeout: 20_000 });
  await frame.focus();
  const initial = await svg.evaluate((element) => element.style.transform);
  await page.keyboard.press("Equal");
  await page.waitForFunction(
    ({ frameSelector, transform }) =>
      document.querySelector(`${frameSelector} svg`)?.style.transform !== transform,
    { frameSelector: selector, transform: initial },
  );
  const zoomed = await svg.evaluate((element) => element.style.transform);
  await page.keyboard.press("ArrowRight");
  await page.waitForFunction(
    ({ frameSelector, transform }) =>
      document.querySelector(`${frameSelector} svg`)?.style.transform !== transform,
    { frameSelector: selector, transform: zoomed },
  );
  const panned = await svg.evaluate((element) => element.style.transform);
  await page.keyboard.press("Minus");
  const zoomedOut = await svg.evaluate((element) => element.style.transform);
  await page.keyboard.press("0");
  await page.waitForFunction(
    ({ frameSelector, transform }) =>
      document.querySelector(`${frameSelector} svg`)?.style.transform === transform,
    { frameSelector: selector, transform: initial },
  );
  return {
    ariaKeyshortcuts: await frame.getAttribute("aria-keyshortcuts"),
    frameBox: await frame.evaluate((element) => element.getBoundingClientRect().toJSON()),
    initial,
    panned,
    reset: await svg.evaluate((element) => element.style.transform),
    source: await frame.evaluate((element) => element.parentElement?.dataset.mermaidSource),
    svgAttributes: await svg.evaluate((element) => ({
      height: element.getAttribute("height"),
      style: element.getAttribute("style"),
      viewBox: element.getAttribute("viewBox"),
      width: element.getAttribute("width"),
    })),
    svgStructure: await svg.evaluate((element) => element.outerHTML.slice(0, 2400)),
    svgBox: await svg.evaluate((element) => element.getBoundingClientRect().toJSON()),
    firstLabel: await svg.evaluate((element) => {
      const label = element.querySelector("foreignObject div, .nodeLabel");
      if (!label) return null;
      const style = getComputedStyle(label);
      return {
        box: label.getBoundingClientRect().toJSON(),
        fontFamily: style.fontFamily,
        fontSize: style.fontSize,
        lineHeight: style.lineHeight,
        margin: style.margin,
        text: label.textContent,
      };
    }),
    zoomed,
    zoomedOut,
  };
}

async function captureLive(name, viewport) {
  const page = await browser.newPage({ viewport });
  const errors = [];
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(liveUrl, { waitUntil: "networkidle", timeout: 60_000 });
  const panzoom = await verifyPanzoom(page, ".docs-prose .mermaid-panzoom-frame");
  await page.evaluate(() => window.scrollTo(0, 0));
  const fullPagePath = resolve(evidenceRoot, `live-what-is-z00z-${name}.png`);
  await page.screenshot({ path: fullPagePath, fullPage: true });
  const mermaidPath = resolve(evidenceRoot, `live-mermaid-${name}.png`);
  await page.locator(".docs-prose .mermaid").first().screenshot({ path: mermaidPath });
  const tablePath = resolve(evidenceRoot, `live-table-${name}.png`);
  await page.locator(".docs-prose table").first().screenshot({ path: tablePath });
  const result = {
    errors,
    fullPagePath,
    heading: await page.locator("#page-title").textContent(),
    mermaidPath,
    panzoom,
    tablePath,
    viewport,
  };
  await page.close();
  return result;
}

async function captureHelp(name, viewport) {
  const page = await browser.newPage({ viewport });
  const errors = [];
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(helpUrl, { waitUntil: "networkidle", timeout: 60_000 });
  await page.locator("#help-document").waitFor({ state: "visible" });
  await page.evaluate(async (html) => {
    const article = document.createElement("article");
    article.id = "website-markdown-parity-document";
    article.className = "help-markdown";
    article.innerHTML = html;
    document.querySelector("#help-document").replaceChildren(article);
    await window.Z00ZHelpMarkdownEnhancer.enhance(article);
    document.querySelector("#help-main").scrollTop = 0;
    window.scrollTo(0, 0);
  }, helpHtml);
  const panzoom = await verifyPanzoom(page, "#website-markdown-parity-document .mermaid-panzoom-frame");
  await page.evaluate(() => {
    document.querySelector("#help-main").scrollTop = 0;
    window.scrollTo(0, 0);
  });
  const fullPagePath = resolve(evidenceRoot, `help-what-is-z00z-${name}.png`);
  await page.screenshot({ path: fullPagePath, fullPage: true });
  const mermaidPath = resolve(evidenceRoot, `help-mermaid-${name}.png`);
  await page.locator("#website-markdown-parity-document .mermaid").first().screenshot({ path: mermaidPath });
  const tablePath = resolve(evidenceRoot, `help-table-${name}.png`);
  await page.locator("#website-markdown-parity-document table").first().screenshot({ path: tablePath });
  const rawMermaid = await page.evaluate(async () => {
    const node = document.querySelector("#website-markdown-parity-document .mermaid");
    const source = node.dataset.mermaidSource;
    const { svg } = await window.mermaid.render(`z00z-help-mermaid-audit-${Date.now()}`, source);
    const holder = document.createElement("div");
    holder.innerHTML = svg;
    const element = holder.querySelector("svg");
    return {
      height: element.getAttribute("height"),
      style: element.getAttribute("style"),
      viewBox: element.getAttribute("viewBox"),
      width: element.getAttribute("width"),
    };
  });
  const result = {
    errors,
    fullPagePath,
    heading: await page.locator("#website-markdown-parity-document h1").first().textContent(),
    mermaidPath,
    panzoom,
    rawMermaid,
    tablePath,
    viewport,
  };
  await page.close();
  return result;
}

async function capturePluginFixture(name, viewport) {
  const page = await browser.newPage({ viewport });
  const errors = [];
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(helpUrl, { waitUntil: "networkidle", timeout: 60_000 });
  await page.locator("#help-document").waitFor({ state: "visible" });
  await page.evaluate(async (html) => {
    const article = document.createElement("article");
    article.id = "website-markdown-plugin-fixture";
    article.className = "help-markdown";
    article.innerHTML = html;
    document.querySelector("#help-document").replaceChildren(article);
    await window.Z00ZHelpMarkdownEnhancer.enhance(article);
    document.querySelector("#help-main").scrollTop = 0;
    window.scrollTo(0, 0);
  }, pluginFixtureHtml);
  await page.locator("#website-markdown-plugin-fixture .mermaid-panzoom-frame").waitFor({ state: "visible" });
  await page.locator("#website-markdown-plugin-fixture .tabs-nav-btn", { hasText: "Second" }).click();
  await page.locator("#website-markdown-plugin-fixture details summary").click();
  const selectors = await page.locator("#website-markdown-plugin-fixture").evaluate((article) => {
    const expected = [
      ".table-of-contents",
      "abbr",
      ".markdown-alert-note",
      ".markdown-alert-important",
      ".markdown-alert-tip",
      ".markdown-alert-warning",
      ".markdown-alert-caution",
      "[style*='text-align']",
      "#fixture-attributes.accent",
      "dl",
      ".footnote-ref",
      "ins",
      "mark",
      ".spoiler",
      "sub",
      "sup",
      ".katex",
      ".tabs-block",
      ".task-list-item-checkbox",
      ".mermaid-panzoom-frame",
      "details[open]",
      "table",
      "code.hljs",
    ];
    return Object.fromEntries(expected.map((selector) => [selector, article.querySelectorAll(selector).length]));
  });
  const screenshotPath = resolve(evidenceRoot, `help-plugin-matrix-${name}.png`);
  await page.addStyleTag({
    content: `
      .help-site-header, .help-sidebar, .help-sidebar-backdrop { display: none !important; }
      .help-page-shell { display: block !important; }
      .help-main { margin-inline: auto !important; }
    `,
  });
  await page.locator("#website-markdown-plugin-fixture").screenshot({ path: screenshotPath });
  await page.close();
  return { errors, screenshotPath, selectors, viewport };
}

try {
  for (const [name, viewport] of Object.entries({
    desktop: { width: 1440, height: 1000 },
    mobile: { width: 390, height: 844 },
  })) {
    audit.pages[`live-${name}`] = await captureLive(name, viewport);
    audit.pages[`help-${name}`] = await captureHelp(name, viewport);
    audit.pages[`help-plugin-matrix-${name}`] = await capturePluginFixture(name, viewport);
  }
} finally {
  await browser.close();
}

await writeFile(resolve(evidenceRoot, "audit.json"), `${JSON.stringify(audit, null, 2)}\n`, "utf8");
console.log(`Markdown visual parity evidence written to ${evidenceRoot}`);
