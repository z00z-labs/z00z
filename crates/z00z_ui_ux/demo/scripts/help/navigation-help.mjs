import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

import { helpRecords, pageFile } from "./navigation-contract.mjs";
import { renderHelpMarkdown } from "./markdown-renderer.mjs";

const REQUIRED_HEADINGS = Object.freeze([
  "## App View {#current-view}",
  "## Overview",
  "## How to use this view",
  "## Terms and controls",
  "## Safety and limits",
]);

function parseFrontmatter(source, sourceName) {
  if (!source.startsWith("---\n")) throw new Error(`${sourceName}: missing YAML front matter.`);
  const closing = source.indexOf("\n---\n", 4);
  if (closing < 0) throw new Error(`${sourceName}: unterminated YAML front matter.`);

  const frontmatter = Object.fromEntries(source.slice(4, closing).split("\n").map((line) => {
    const separator = line.indexOf(": ");
    if (separator < 1) throw new Error(`${sourceName}: malformed front matter line.`);
    return [line.slice(0, separator), line.slice(separator + 2)];
  }));
  return { body: source.slice(closing + 5), frontmatter };
}

function sourceMarker(source, sourceName) {
  const match = source.match(/<!-- help-sync:source (\{.+\}) -->/u);
  if (!match) throw new Error(`${sourceName}: missing Help sync provenance.`);

  try {
    return JSON.parse(match[1]);
  } catch {
    throw new Error(`${sourceName}: invalid Help sync provenance.`);
  }
}

function plainText(html) {
  return html
    .replace(/<[^>]+>/gu, " ")
    .replace(/&(?:amp|lt|gt|quot|#39);/gu, " ")
    .replace(/\s+/gu, " ")
    .trim();
}

export function parseHelpPage(source, record, sourceName) {
  const { body, frontmatter } = parseFrontmatter(source, sourceName);
  const expectedPath = pageFile(record);

  for (const heading of REQUIRED_HEADINGS) {
    if (!body.includes(heading)) throw new Error(`${sourceName}: missing required section ${heading}.`);
  }
  if (frontmatter.id !== record.id) throw new Error(`${sourceName}: topic ID does not match navigation.`);
  if (frontmatter.route !== (record.routeId || "none")) throw new Error(`${sourceName}: route does not match navigation.`);
  if (frontmatter.scope !== record.scope) throw new Error(`${sourceName}: scope does not match navigation.`);
  if (!frontmatter.title) throw new Error(`${sourceName}: title is required.`);

  const provenance = sourceMarker(body, sourceName);
  if (
    provenance.topic_id !== record.id
    || provenance.route_id !== (record.routeId || "none")
    || provenance.page_path !== expectedPath
  ) {
    throw new Error(`${sourceName}: Help sync provenance does not match navigation.`);
  }

  const renderedSource = body.replace(/<!-- help-sync:source \{.+\} -->/u, "");
  const html = renderHelpMarkdown(renderedSource, sourceName);
  return Object.freeze({
    html,
    id: record.id,
    pagePath: expectedPath,
    routeId: record.routeId,
    scope: record.scope,
    text: plainText(html),
    title: frontmatter.title,
  });
}

export async function loadNavigationHelp(root) {
  const records = helpRecords();
  const documents = await Promise.all(records.map(async (record) => {
    const path = resolve(root, "help", "en", pageFile(record));
    const source = await readFile(path, "utf8");
    return parseHelpPage(source, record, path);
  }));
  return Object.freeze({ documents: Object.freeze(documents), records });
}

export async function compileNavigationHelp(root) {
  const { documents, records } = await loadNavigationHelp(root);
  const documentsById = Object.fromEntries(documents.map((document) => [document.id, document]));
  const payload = {
    catalogues: { en: documentsById },
    locales: ["en"],
    records,
    version: 2,
  };
  return `"use strict";\n\n((root) => {\n  root.Z00ZHelpCatalog = Object.freeze(${JSON.stringify(payload, null, 2)});\n})(typeof window === "undefined" ? globalThis : window);\n`;
}
