// Generated from /home/vadim/Projects/z00z-website/src/lib/content/markdown.ts (sha256:2fc992113376e7abf56c4c17f92523e54b3d028d36b32fee289318445d85e92e).
import path from "node:path";

import { abbr } from "@mdit/plugin-abbr";
import { alert } from "@mdit/plugin-alert";
import { align } from "@mdit/plugin-align";
import { anchor } from "@mdit/plugin-anchor";
import { attrs } from "@mdit/plugin-attrs";
import { container } from "@mdit/plugin-container";
import { dl } from "@mdit/plugin-dl";
import { embed } from "@mdit/plugin-embed";
import { figure } from "@mdit/plugin-figure";
import { footnote } from "@mdit/plugin-footnote";
import { imgLazyload } from "@mdit/plugin-img-lazyload";
import { imgSize } from "@mdit/plugin-img-size";
import { include } from "@mdit/plugin-include";
import { ins } from "@mdit/plugin-ins";
import { katex } from "@mdit/plugin-katex";
import { mark } from "@mdit/plugin-mark";
import { snippet } from "@mdit/plugin-snippet";
import { spoiler } from "@mdit/plugin-spoiler";
import { stylize } from "@mdit/plugin-stylize";
import { sub } from "@mdit/plugin-sub";
import { sup } from "@mdit/plugin-sup";
import { tab } from "@mdit/plugin-tab";
import type { MarkdownItTabData, MarkdownItTabInfo } from "@mdit/plugin-tab";
import { tasklist } from "@mdit/plugin-tasklist";
import { uml } from "@mdit/plugin-uml";
import { tocPlugin } from "@mdit-vue/plugin-toc";
import { load } from "cheerio";
import hljs from "highlight.js";
import MarkdownIt from "markdown-it";
import collapsible from "markdown-it-collapsible";

import type { ContentPipelineConfig } from "@/lib/config/site";
import { slugifyHeading } from "./markdown-route-contract.mjs";
import { isExternalLink } from "./markdown-link-policy.mjs";

type RenderMarkdownOptions = {
  filePath: string;
  pipeline: ContentPipelineConfig;
};

type MermaidRole =
  | "public"
  | "domain"
  | "runtime"
  | "external"
  | "danger"
  | "support"
  | "crypto"
  | "storage";

type MermaidPaletteEntry = {
  box: string;
  fill: string;
  label: string;
  stroke: string;
  text: string;
};

type C4Node = {
  description: string;
  external: boolean;
  id: string;
  internal: boolean;
  name: string;
  person: boolean;
  role: MermaidRole;
  technology: string | null;
};

type C4Edge = {
  from: string;
  label: string;
  to: string;
};

const MERMAID_PALETTE: Record<MermaidRole, MermaidPaletteEntry> = {
  public: {
    box: "rgb(227,242,253)",
    fill: "#E3F2FD",
    label: "Public API / User",
    stroke: "#1E88E5",
    text: "#0D47A1",
  },
  domain: {
    box: "rgb(243,229,245)",
    fill: "#F3E5F5",
    label: "Domain logic",
    stroke: "#8E24AA",
    text: "#4A148C",
  },
  runtime: {
    box: "rgb(255,243,224)",
    fill: "#FFF3E0",
    label: "Infrastructure / Runtime",
    stroke: "#FB8C00",
    text: "#E65100",
  },
  external: {
    box: "rgb(232,245,233)",
    fill: "#E8F5E9",
    label: "External / Cross-system",
    stroke: "#43A047",
    text: "#1B5E20",
  },
  danger: {
    box: "rgb(255,224,224)",
    fill: "#FFE0E0",
    label: "Danger / Failure / Attack",
    stroke: "#D32F2F",
    text: "#B71C1C",
  },
  support: {
    box: "rgb(236,239,241)",
    fill: "#ECEFF1",
    label: "Neutral / Support",
    stroke: "#546E7A",
    text: "#263238",
  },
  crypto: {
    box: "rgb(237,231,246)",
    fill: "#EDE7F6",
    label: "Crypto / Proof",
    stroke: "#5E35B1",
    text: "#311B92",
  },
  storage: {
    box: "rgb(255,224,178)",
    fill: "#FFE0B2",
    label: "Storage / DA layer",
    stroke: "#F57C00",
    text: "#263238",
  },
};

const UNSAFE_URL_ATTRIBUTES = new Set([
  "action",
  "cite",
  "formaction",
  "href",
  "poster",
  "src",
  "xlink:href",
]);

function isUnsafeProtocol(value: string, attributeName: string): boolean {
  const normalized = value.replace(/[\u0000-\u001f\u007f\s]+/gu, "").trim().toLowerCase();

  if (!normalized) {
    return false;
  }

  if (normalized.startsWith("javascript:") || normalized.startsWith("vbscript:")) {
    return true;
  }

  if (!normalized.startsWith("data:")) {
    return false;
  }

  return attributeName !== "src" || !normalized.startsWith("data:image/");
}

function sanitizeInlineStyle(styleValue: string): string | null {
  if (/(expression\s*\(|javascript:|vbscript:|behavior\s*:)/iu.test(styleValue)) {
    return null;
  }

  return styleValue;
}

function sanitizeRenderedMarkdown(html: string): string {
  const $ = load(`<div data-z00z-markdown-root="">${html}</div>`);
  const root = $("div[data-z00z-markdown-root]");

  root.find("script, base, meta[http-equiv='refresh']").remove();

  root.find("*").each((_, element) => {
    const attributes =
      "attribs" in element && element.attribs
        ? Object.entries(element.attribs)
        : [];

    for (const [name, rawValue] of attributes) {
      const attributeName = name.toLowerCase();
      const attributeValue = rawValue ?? "";

      if (attributeName.startsWith("on") || attributeName === "srcdoc") {
        $(element).removeAttr(name);
        continue;
      }

      if (attributeName === "style") {
        const safeStyle = sanitizeInlineStyle(attributeValue);

        if (safeStyle === null) {
          $(element).removeAttr(name);
        }

        continue;
      }

      if (UNSAFE_URL_ATTRIBUTES.has(attributeName) && isUnsafeProtocol(attributeValue, attributeName)) {
        $(element).removeAttr(name);
      }
    }
  });

  root.find("a[href]").each((_, element) => {
    const href = $(element).attr("href") ?? "";

    if (!isExternalLink(href)) {
      return;
    }

    $(element).attr("target", "_blank");
    $(element).attr("rel", "noopener noreferrer");
  });

  return root.html() ?? "";
}

const MERMAID_ALLOWED_HEX = new Set(
  [
    "#FFFFFF",
    "#D0D7DE",
    ...Object.values(MERMAID_PALETTE).flatMap((entry) => [entry.fill, entry.stroke, entry.text]),
  ].map((color) => color.toUpperCase()),
);

const FLOW_NODE_PATTERN =
  /([A-Za-z][A-Za-z0-9_]*)\s*(\[\([^\n]*?\)\]|\[\[[^\n]*?\]\]|\[[^\n]*?\]|\(\([^\n]*?\)\)|\([^\n]*?\)|\{\{[^\n]*?\}\})/g;

const STATE_TRANSITION_PATTERN = /([A-Za-z][A-Za-z0-9_]*)\s*-->\s*([A-Za-z][A-Za-z0-9_]*)/g;
const STATE_DECLARATION_PATTERN = /^\s*([A-Za-z][A-Za-z0-9_]*)\s*:/u;
const SEQUENCE_PARTICIPANT_PATTERN =
  /^\s*participant\s+([A-Za-z][A-Za-z0-9_]*)\s*(?:as\s+(.+))?$/u;

function encodeMermaidSource(source: string): string {
  return encodeURIComponent(source);
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function isMermaidFenceLanguage(language: string | undefined): boolean {
  return language === "mermaid" || language === "mermaid-spectrum" || language === "mermaid-c4";
}

function normalizeHexColor(color: string): string {
  return color.toUpperCase();
}

function hasCanonicalMermaidPalette(source: string): boolean {
  for (const color of MERMAID_ALLOWED_HEX) {
    if (source.includes(color) || source.includes(color.toLowerCase())) {
      return true;
    }
  }

  return false;
}

function hasOffPaletteHexColors(source: string): boolean {
  const matches = source.match(/#(?:[0-9A-Fa-f]{6})\b/g) ?? [];
  return matches.some((color) => !MERMAID_ALLOWED_HEX.has(normalizeHexColor(color)));
}

function stripMermaidLabelWrapper(value: string): string {
  return value
    .replace(/^\[\(/u, "")
    .replace(/\)\]$/u, "")
    .replace(/^\[\[/u, "")
    .replace(/\]\]$/u, "")
    .replace(/^\(\(/u, "")
    .replace(/\)\)$/u, "")
    .replace(/^\{\{/u, "")
    .replace(/\}\}$/u, "")
    .replace(/^[\[(]/u, "")
    .replace(/[\])]$/u, "")
    .replace(/^"(.*)"$/u, "$1")
    .replace(/^'(.*)'$/u, "$1")
    .replace(/<br\s*\/?>/giu, " ")
    .replace(/\s+/gu, " ")
    .trim();
}

function classifyMermaidRole(id: string, label: string): MermaidRole {
  const normalized = `${id} ${label}`.toLowerCase();

  if (
    /\b(question|questions|risk|risky|danger|attack|failure|fail|failed|reject|rejected|quarantine|conflict|fraud|slash|locked|challenge|expired|unsupported|unsafe)\b/u.test(
      normalized,
    )
  ) {
    return "danger";
  }

  if (
    /\b(da|data availability|database|db|storage|store|queue|redis|postgres|archive|archived|blob|bytes|index|artifact lineage)\b/u.test(
      normalized,
    )
  ) {
    return "storage";
  }

  if (
    /\b(crypto|proof|signature|commitment|nullifier|cipher|decrypt|encrypt|receiver material|zk|stealth)\b/u.test(
      normalized,
    )
  ) {
    return "crypto";
  }

  if (
    /\b(user|wallet|client|customer|sender|receiver|contributor|developer|reader|operator|support agent)\b/u.test(
      normalized,
    )
  ) {
    return "public";
  }

  if (
    /\b(external|issuer|merchant|bridge|auditor|reviewer|regulator|provider|integrator|counterparty|service operator|explorer|status)\b/u.test(
      normalized,
    )
  ) {
    return "external";
  }

  if (
    /\b(publication|aggregator|runtime|node|network|relay|route|ingress|egress|pipeline|batch|validator|watcher|watchers|publication path|soft confirmation)\b/u.test(
      normalized,
    )
  ) {
    return "runtime";
  }

  if (
    /\b(protocol|checkpoint|settlement|governance|policy|asset|voucher|right|claim|authority|theorem|core)\b/u.test(
      normalized,
    )
  ) {
    return "domain";
  }

  return "support";
}

function buildMermaidStyleLine(id: string, role: MermaidRole): string {
  const palette = MERMAID_PALETTE[role];
  return `style ${id} fill:${palette.fill},stroke:${palette.stroke},stroke-width:1px,color:${palette.text}`;
}

function buildMermaidClassDef(role: MermaidRole): string {
  const palette = MERMAID_PALETTE[role];
  return `  classDef ${role} fill:${palette.fill},stroke:${palette.stroke},stroke-width:1px,color:${palette.text}`;
}

function appendMermaidLines(source: string, lines: string[]): string {
  return `${source.trimEnd()}\n\n${lines.join("\n")}`;
}

function applyFlowchartPalette(source: string): string {
  const nodeLabels = new Map<string, string>();

  for (const match of source.matchAll(FLOW_NODE_PATTERN)) {
    const [, id, label] = match;

    if (!nodeLabels.has(id)) {
      nodeLabels.set(id, stripMermaidLabelWrapper(label));
    }
  }

  if (nodeLabels.size === 0) {
    return source;
  }

  const styles = Array.from(nodeLabels.entries()).map(([id, label]) =>
    buildMermaidStyleLine(id, classifyMermaidRole(id, label)),
  );

  return appendMermaidLines(source, styles);
}

function applyStatePalette(source: string): string {
  const roles = new Map<string, MermaidRole>();

  for (const line of source.split(/\r?\n/u)) {
    const declarationMatch = line.match(STATE_DECLARATION_PATTERN);

    if (declarationMatch) {
      const state = declarationMatch[1];
      roles.set(state, classifyMermaidRole(state, state));
    }

    for (const match of line.matchAll(STATE_TRANSITION_PATTERN)) {
      const [, from, to] = match;
      if (from !== "[*]") {
        roles.set(from, classifyMermaidRole(from, from));
      }
      if (to !== "[*]") {
        roles.set(to, classifyMermaidRole(to, to));
      }
    }
  }

  if (roles.size === 0) {
    return source;
  }

  const usedRoles = new Set(roles.values());
  const classDefs = Array.from(usedRoles).map((role) => buildMermaidClassDef(role));
  const classLines = Array.from(roles.entries()).map(([state, role]) => `  class ${state} ${role}`);

  return appendMermaidLines(source, [...classDefs, "", ...classLines]);
}

function applySequencePalette(source: string): string {
  const lines = source.trim().split(/\r?\n/u);
  const [header, ...rest] = lines;
  const participants: Array<{ definition: string; role: MermaidRole }> = [];
  const body: string[] = [];

  for (const line of rest) {
    const participantMatch = line.match(SEQUENCE_PARTICIPANT_PATTERN);

    if (!participantMatch) {
      body.push(line);
      continue;
    }

    const [, id, alias] = participantMatch;
    const label = alias?.trim() ?? id;
    participants.push({
      definition: line.trim(),
      role: classifyMermaidRole(id, label),
    });
  }

  if (participants.length === 0) {
    return source;
  }

  const boxedParticipants = participants.flatMap(({ definition, role }) => {
    const palette = MERMAID_PALETTE[role];
    return [
      `  box ${palette.box} ${palette.label}`,
      `    ${definition}`,
      "  end",
    ];
  });

  return [header, ...boxedParticipants, "", ...body].join("\n").trimEnd();
}

function parseC4Arguments(argumentList: string): string[] {
  const values: string[] = [];
  let current = "";
  let inQuote = false;

  for (let index = 0; index < argumentList.length; index += 1) {
    const character = argumentList[index];

    if (character === '"' && argumentList[index - 1] !== "\\") {
      inQuote = !inQuote;
      current += character;
      continue;
    }

    if (character === "," && !inQuote) {
      values.push(current.trim());
      current = "";
      continue;
    }

    current += character;
  }

  if (current.trim().length > 0) {
    values.push(current.trim());
  }

  return values.map((value) => value.replace(/^"(.*)"$/u, "$1").trim());
}

function normalizeC4LabelPart(value: string | null): string {
  return value?.replace(/"/gu, "&quot;").trim() ?? "";
}

function convertPseudoC4(source: string): string {
  const lines = source
    .trim()
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter(Boolean);

  const diagramKind = lines[0];
  let title = "System Boundary";
  const nodes: C4Node[] = [];
  const edges: C4Edge[] = [];

  for (const line of lines.slice(1)) {
    if (line.startsWith("title ")) {
      title = line.slice("title ".length).trim();
      continue;
    }

    const relationMatch = line.match(/^Rel\((.+)\)$/u);

    if (relationMatch) {
      const [from, to, label] = parseC4Arguments(relationMatch[1]);
      edges.push({ from, label, to });
      continue;
    }

    const nodeMatch = line.match(/^([A-Za-z_]+)\((.+)\)$/u);

    if (!nodeMatch) {
      continue;
    }

    const [, kind, rawArgs] = nodeMatch;
    const args = parseC4Arguments(rawArgs);
    const [id, name, thirdArg, fourthArg] = args;
    const person = kind === "Person";
    const external = kind.endsWith("_Ext");
    const internal = !person && !external;
    const technology = kind.startsWith("Container") ? thirdArg ?? null : null;
    const description = kind.startsWith("Container") ? fourthArg ?? "" : thirdArg ?? "";
    const role = person
      ? "public"
      : external
        ? "external"
        : classifyMermaidRole(id, [name, technology ?? "", description].join(" "));

    nodes.push({
      description,
      external,
      id,
      internal,
      name,
      person,
      role,
      technology,
    });
  }

  const internalNodes = nodes.filter((node) => node.internal);
  const externalNodes = nodes.filter((node) => !node.internal);
  const output = ["flowchart LR"];

  for (const node of externalNodes.filter((entry) => entry.person)) {
    const label = [normalizeC4LabelPart(node.name), normalizeC4LabelPart(node.description)]
      .filter(Boolean)
      .join("<br/>");
    output.push(`  ${node.id}["${label}"]`);
  }

  if (diagramKind === "C4Container" && internalNodes.length > 0) {
    output.push(`  subgraph SystemBoundary[${normalizeC4LabelPart(title)}]`);
    for (const node of internalNodes) {
      const label = [
        normalizeC4LabelPart(node.name),
        normalizeC4LabelPart(node.technology),
        normalizeC4LabelPart(node.description),
      ]
        .filter(Boolean)
        .join("<br/>");
      output.push(`    ${node.id}["${label}"]`);
    }
    output.push("  end");
  } else {
    for (const node of internalNodes) {
      const label = [
        normalizeC4LabelPart(node.name),
        normalizeC4LabelPart(node.description),
      ]
        .filter(Boolean)
        .join("<br/>");
      output.push(`  ${node.id}["${label}"]`);
    }
  }

  for (const node of externalNodes.filter((entry) => !entry.person)) {
    const label = [normalizeC4LabelPart(node.name), normalizeC4LabelPart(node.description)]
      .filter(Boolean)
      .join("<br/>");
    output.push(`  ${node.id}["${label}"]`);
  }

  output.push("");

  for (const edge of edges) {
    output.push(`  ${edge.from} -->|${normalizeC4LabelPart(edge.label)}| ${edge.to}`);
  }

  output.push("");

  if (diagramKind === "C4Container" && internalNodes.length > 0) {
    output.push(buildMermaidStyleLine("SystemBoundary", "support"));
  }

  for (const node of nodes) {
    output.push(buildMermaidStyleLine(node.id, node.role));
  }

  return output.join("\n").trimEnd();
}

export function normalizeMermaidSource(source: string): string {
  const content = source.trim();

  if (content.length === 0) {
    return content;
  }

  const firstLine = content.split(/\r?\n/u).find((line) => line.trim().length > 0)?.trim() ?? "";

  if (/^C4(?:Context|Container)\b/u.test(firstLine)) {
    return convertPseudoC4(content);
  }

  const hasPalette = hasCanonicalMermaidPalette(content);

  if ((firstLine.startsWith("flowchart") || firstLine.startsWith("graph")) && !hasPalette) {
    return applyFlowchartPalette(content);
  }

  if (firstLine.startsWith("stateDiagram-v2") && !hasPalette) {
    return applyStatePalette(content);
  }

  if (firstLine.startsWith("sequenceDiagram") && !/box rgb\(/u.test(content)) {
    return applySequencePalette(content);
  }

  if (hasOffPaletteHexColors(content)) {
    return content;
  }

  return content;
}

function renderMermaidBlock(source: string): string {
  const content = normalizeMermaidSource(source);

  return `<div class="mermaid" data-mermaid-definition="${encodeMermaidSource(content)}">${escapeHtml(content)}</div>`;
}

function resolveCurrentPath(env: unknown, filePath: string): string {
  if (
    env &&
    typeof env === "object" &&
    "filePath" in env &&
    typeof (env as { filePath?: unknown }).filePath === "string"
  ) {
    return path.dirname((env as { filePath: string }).filePath);
  }

  return path.dirname(filePath);
}

export function renderMarkdown(source: string, options: RenderMarkdownOptions): string {
  const { filePath, pipeline } = options;

  const md = new MarkdownIt({
    html: pipeline.markdown.html,
    linkify: pipeline.markdown.linkify,
    typographer: pipeline.markdown.typographer,
    highlight(code, language) {
      if (language && hljs.getLanguage(language)) {
        return `<pre class="code-block"><code class="hljs language-${language}">${hljs.highlight(code, {
          language,
          ignoreIllegals: true,
        }).value}</code></pre>`;
      }

      const autoDetected = hljs.highlightAuto(code);

      return `<pre class="code-block"><code class="hljs language-${autoDetected.language ?? "plaintext"}">${autoDetected.value}</code></pre>`;
    },
  });

  const defaultFenceRenderer = md.renderer.rules.fence?.bind(md.renderer.rules);

  md.renderer.rules.fence = (tokens, index, rendererOptions, env, self) => {
    const token = tokens[index];
    const language = token.info.trim().split(/\s+/u)[0]?.toLowerCase();

    if (pipeline.markdown.mermaid && isMermaidFenceLanguage(language)) {
      return renderMermaidBlock(token.content);
    }

    if (defaultFenceRenderer) {
      return defaultFenceRenderer(tokens, index, rendererOptions, env, self);
    }

    return self.renderToken(tokens, index, rendererOptions);
  };

  if (pipeline.markdown.abbreviation) {
    md.use(abbr);
  }

  if (pipeline.markdown.alerts) {
    md.use(alert, { deep: true });
  }

  if (pipeline.markdown.align) {
    md.use(align);
  }

  if (pipeline.markdown.anchor) {
    md.use(anchor, {
      slugify: slugifyHeading,
      tabIndex: false,
    });
  }

  if (pipeline.markdown.toc) {
    md.use(tocPlugin, {
      pattern: /^\[TOC\]$/i,
      slugify: slugifyHeading,
      level: [2, 3],
      containerTag: "nav",
      containerClass: "table-of-contents",
      listClass: "table-of-contents-list",
      itemClass: "table-of-contents-item",
      linkClass: "table-of-contents-link",
    });
  }

  if (pipeline.markdown.attrs) {
    md.use(attrs);
  }

  if (pipeline.markdown.containers) {
    md.use(container, {
      name: "warning",
    });
  }

  if (pipeline.markdown.definition_lists) {
    md.use(dl);
  }

  if (pipeline.markdown.embeds) {
    md.use(embed, {
      config: [
        {
          name: "youtube",
          setup: (id: string) =>
            `<div class="video-embed"><iframe src="https://www.youtube.com/embed/${id}" title="YouTube video" loading="lazy" allowfullscreen></iframe></div>`,
        },
      ],
    });
  }

  if (pipeline.markdown.figures) {
    md.use(figure);
  }

  if (pipeline.markdown.footnotes) {
    md.use(footnote);
  }

  if (pipeline.markdown.image_lazyload) {
    md.use(imgLazyload);
  }

  if (pipeline.markdown.image_size) {
    md.use(imgSize);
  }

  if (pipeline.markdown.include) {
    md.use(include, {
      currentPath: (env) => resolveCurrentPath(env, filePath),
      deep: true,
    });
  }

  if (pipeline.markdown.insertions) {
    md.use(ins);
  }

  if (pipeline.markdown.katex) {
    md.use(katex);
  }

  if (pipeline.markdown.mark) {
    md.use(mark);
  }

  if (pipeline.markdown.snippet) {
    md.use(snippet, {
      currentPath: (env) => resolveCurrentPath(env, filePath),
    });
  }

  if (pipeline.markdown.spoiler) {
    md.use(spoiler);
  }

  if (pipeline.markdown.stylize) {
    md.use(stylize, {
      config: [
        {
          matcher: /^(?!bg-)([#a-zA-Z0-9-]+):(.*)$/,
          replacer: ({ tag, content }) => {
            if (tag !== "mark") {
              return;
            }

            const match = content.match(/^(?!bg-)([#a-zA-Z0-9-]+):(.*)$/);

            if (!match) {
              return;
            }

            const [, color, text] = match;

            return {
              tag: "span",
              attrs: {
                class: "token-inline-accent",
                style: color.startsWith("#") ? `color: ${color};` : undefined,
                "data-color-token": color.startsWith("#") ? undefined : color,
              },
              content: text.trim(),
            };
          },
        },
      ],
    });
  }

  if (pipeline.markdown.sub_sup) {
    md.use(sub);
    md.use(sup);
  }

  if (pipeline.markdown.tabs) {
    const tabActiveIndexStack: number[] = [];

    md.use(tab, {
      openRender(info: MarkdownItTabInfo) {
        const activeIndex = info.active >= 0 ? info.active : 0;
        tabActiveIndexStack.push(activeIndex);
        const navigation = info.data
          .map(
            (item, index) => `
              <button
                type="button"
                class="tabs-nav-btn ${index === activeIndex ? "tabs-nav-btn-active" : "tabs-nav-btn-inactive"}"
                aria-selected="${index === activeIndex}"
                data-tab-target="${item.id}"
              >
                ${item.title}
              </button>`,
          )
          .join("");

        return `<div class="tabs-block"><div class="tabs-nav" role="tablist">${navigation}</div>`;
      },
      closeRender: () => {
        tabActiveIndexStack.pop();
        return "</div>";
      },
      tabOpenRender(data: MarkdownItTabData) {
        const activeIndex = tabActiveIndexStack.at(-1) ?? 0;
        const isActive = data.index === activeIndex;
        return `<div class="tabs-panel ${isActive ? "tabs-panel-active" : "tabs-panel-hidden"}" id="${data.id}" role="tabpanel" aria-expanded="${isActive}">`;
      },
      tabCloseRender: () => "</div>",
    });
  }

  if (pipeline.markdown.tasklist) {
    md.use(tasklist);
  }

  if (pipeline.markdown.mermaid) {
    md.use(uml, {
      name: "mermaid",
      open: "mermaidstart",
      close: "mermaidend",
      render(tokens, index) {
        return renderMermaidBlock(tokens[index].content);
      },
    });
  }

  if (pipeline.markdown.collapsible) {
    md.use(collapsible);
  }

  return sanitizeRenderedMarkdown(md.render(source, { filePath }));
}
