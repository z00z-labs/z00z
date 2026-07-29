import { readFile, readdir } from "node:fs/promises";
import { basename, relative, resolve, sep } from "node:path";

import YAML from "yaml";

function portablePath(root, path) {
  return relative(root, path).split(sep).join("/");
}

function titleFromSegment(value) {
  return value
    .replace(/[-_]+/gu, " ")
    .replace(/\b\w/gu, (character) => character.toUpperCase());
}

function parseFrontmatter(source, sourceName) {
  if (!source.startsWith("---\n")) {
    throw new Error(`${sourceName}: missing YAML front matter.`);
  }
  const closing = source.indexOf("\n---\n", 4);
  if (closing < 0) {
    throw new Error(`${sourceName}: unterminated YAML front matter.`);
  }
  const frontmatter = YAML.parse(source.slice(4, closing)) || {};
  if (typeof frontmatter !== "object" || Array.isArray(frontmatter)) {
    throw new Error(`${sourceName}: front matter must be a YAML mapping.`);
  }
  return frontmatter;
}

async function privateMarkdownFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await privateMarkdownFiles(path));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(path);
    }
  }
  return files;
}

async function publicMarkdownFiles(directory, root = directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.name.startsWith("_") || entry.name.startsWith(".")) continue;
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await publicMarkdownFiles(path, root));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(path);
    }
  }
  return files;
}

export async function discoverAdditionalRecords(root, language, baseRecords) {
  const localeRoot = resolve(root, "help", language);
  const baseById = new Map(baseRecords.map((record) => [record.id, record]));
  const baseByPath = new Map(baseRecords.map((record) => [`${record.pagePath.join("/")}.md`, record]));
  const additions = [];
  for (const path of await publicMarkdownFiles(localeRoot)) {
    const pagePath = portablePath(localeRoot, path);
    const source = await readFile(path, "utf8");
    const frontmatter = parseFrontmatter(source, pagePath);
    const baseByTopic = baseById.get(frontmatter.id);
    const baseAtPath = baseByPath.get(pagePath);
    if (baseByTopic || baseAtPath) {
      if (baseByTopic !== baseAtPath) {
        throw new Error(`${pagePath}: App Help topic and canonical path do not match.`);
      }
      continue;
    }
    if (typeof frontmatter.id !== "string" || !frontmatter.id.trim()) {
      throw new Error(`${pagePath}: additional Help page requires an id.`);
    }
    const routeId = frontmatter.route === "none" ? "" : frontmatter.route;
    const contextualOwner = baseRecords.find((record) => (
      record.scope === "context"
      && record.routeId === routeId
      && record.pagePath.slice(0, -1).join("/") === pagePath.split("/").slice(0, -1).join("/")
    ));
    const isStandaloneArticle = ["article", "guide"].includes(frontmatter.scope) && !routeId;
    const isContextualPage = frontmatter.scope === "context" && Boolean(contextualOwner);
    if (!isStandaloneArticle && !isContextualPage) {
      throw new Error(
        `${pagePath}: additional Help content must be a standalone article/guide or a context page beside its canonical routed page.`,
      );
    }
    additions.push(Object.freeze({
      id: frontmatter.id,
      labelKey: "help.title",
      nodeId: "",
      pagePath: Object.freeze(pagePath.replace(/\.md$/u, "").split("/")),
      routeId,
      scope: frontmatter.scope,
    }));
  }
  const ids = additions.map(({ id }) => id);
  if (new Set(ids).size !== ids.length) throw new Error(`${language}: duplicate additional Help topic IDs.`);
  return Object.freeze(additions);
}

async function readFolderMeta(directory, localeRoot) {
  const metaPath = resolve(directory, "_meta.yaml");
  let source = "";
  try {
    source = await readFile(metaPath, "utf8");
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  const meta = source ? YAML.parse(source) || {} : {};
  if (typeof meta !== "object" || Array.isArray(meta)) {
    throw new Error(`${portablePath(localeRoot, metaPath)}: metadata must be a YAML mapping.`);
  }
  if (meta.icon !== undefined && (typeof meta.icon !== "string" || !meta.icon.trim())) {
    throw new Error(`${portablePath(localeRoot, metaPath)}: icon must be a non-empty string.`);
  }
  if (meta.order !== undefined && (
    !Array.isArray(meta.order)
    || meta.order.some((item) => typeof item !== "string" || !item.trim())
  )) {
    throw new Error(`${portablePath(localeRoot, metaPath)}: order must be a string array.`);
  }
  if (meta.order && new Set(meta.order).size !== meta.order.length) {
    throw new Error(`${portablePath(localeRoot, metaPath)}: order contains duplicate entries.`);
  }
  const folderName = basename(directory);
  return Object.freeze({
    icon: meta.icon || "folder",
    order: Object.freeze([...(meta.order || [])]),
    title: typeof meta.title === "string" && meta.title.trim()
      ? meta.title.trim()
      : (directory === localeRoot ? "Z00Z Help" : titleFromSegment(folderName)),
  });
}

function orderedEntries(entries, order) {
  const orderIndex = new Map(order.map((name, index) => [name, index]));
  return [...entries].sort((left, right) => {
    const leftName = left.isDirectory() ? left.name : left.name.replace(/\.md$/u, "");
    const rightName = right.isDirectory() ? right.name : right.name.replace(/\.md$/u, "");
    const leftOrder = orderIndex.get(leftName);
    const rightOrder = orderIndex.get(rightName);
    if (leftOrder !== undefined && rightOrder !== undefined) return leftOrder - rightOrder;
    if (leftOrder !== undefined) return -1;
    if (rightOrder !== undefined) return 1;
    return leftName.localeCompare(rightName);
  });
}

function entryNames(publicEntries) {
  return publicEntries.map((entry) => (
    entry.isDirectory() ? entry.name : entry.name.replace(/\.md$/u, "")
  ));
}

async function loadPage(path, localeRoot, recordsByPath) {
  const pagePath = portablePath(localeRoot, path);
  const record = recordsByPath.get(pagePath);
  if (!record) {
    throw new Error(`${pagePath}: no canonical Help topic uses this Markdown page.`);
  }
  const source = await readFile(path, "utf8");
  const frontmatter = parseFrontmatter(source, pagePath);
  if (frontmatter.id !== record.id) {
    throw new Error(`${pagePath}: topic id ${frontmatter.id || "(missing)"} does not match ${record.id}.`);
  }
  if (typeof frontmatter.title !== "string" || !frontmatter.title.trim()) {
    throw new Error(`${pagePath}: title is required.`);
  }
  return Object.freeze({
    id: record.nodeId || `help.${record.id}`,
    pagePath,
    title: frontmatter.title.trim(),
    topicId: record.id,
    type: "article",
  });
}

async function loadDirectory(directory, localeRoot, recordsByPath, segments = []) {
  const meta = await readFolderMeta(directory, localeRoot);
  const entries = await readdir(directory, { withFileTypes: true });
  const privateEntries = entries.filter((entry) => entry.name !== "_meta.yaml" && (
    entry.name.startsWith("_") || entry.name.startsWith(".")
  ));
  for (const entry of privateEntries.filter((candidate) => candidate.isDirectory())) {
    const markdown = await privateMarkdownFiles(resolve(directory, entry.name));
    if (markdown.length) {
      throw new Error(
        `${portablePath(localeRoot, resolve(directory, entry.name))}: private Help directories cannot contain Markdown.`,
      );
    }
  }
  const publicEntries = entries.filter((entry) => (
    entry.name !== "_meta.yaml"
    && !entry.name.startsWith("_")
    && !entry.name.startsWith(".")
  ));
  const unsupported = publicEntries.filter((entry) => (
    !entry.isDirectory() && !(entry.isFile() && entry.name.endsWith(".md"))
  ));
  if (unsupported.length) {
    throw new Error(
      `${portablePath(localeRoot, directory) || "."}: unsupported public content: ${unsupported.map(({ name }) => name).join(", ")}.`,
    );
  }
  const byOrderName = new Map(orderedEntries(publicEntries, meta.order).map((entry) => [
    entry.isDirectory() ? entry.name : entry.name.replace(/\.md$/u, ""),
    entry,
  ]));
  const items = [];
  let homeTopicId = "";
  const pages = [];
  const directories = [{
    entries: [...byOrderName.keys()],
    iconId: meta.icon,
    path: segments.join("/"),
    title: meta.title,
  }];

  for (const [name, entry] of byOrderName) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      const child = await loadDirectory(path, localeRoot, recordsByPath, [...segments, entry.name]);
      pages.push(...child.pages);
      directories.push(...child.directories);
      if (child.items.length === 1 && child.items[0].type === "article" && child.items[0].pagePath.endsWith("/index.md")) {
        items.push(Object.freeze({
          ...child.items[0],
          iconId: child.meta.icon,
          id: [...segments, entry.name].join("."),
          title: child.meta.title,
        }));
      } else {
        items.push(Object.freeze({
          children: child.items,
          iconId: child.meta.icon,
          id: [...segments, entry.name].join("."),
          title: child.meta.title,
          type: "section",
        }));
      }
      continue;
    }

    const page = await loadPage(path, localeRoot, recordsByPath);
    pages.push(page);
    if (segments.length === 0 && name === "index") {
      homeTopicId = page.topicId;
    } else {
      items.push(page);
    }
  }

  return Object.freeze({
    homeTopicId,
    directories: Object.freeze(directories),
    items: Object.freeze(items),
    meta,
    pages: Object.freeze(pages),
  });
}

export async function loadHelpContent(root, language, records, pageFile) {
  const localeRoot = resolve(root, "help", language);
  const recordsByPath = new Map(records.map((record) => [pageFile(record), record]));
  const content = await loadDirectory(localeRoot, localeRoot, recordsByPath);
  const expectedPaths = [...recordsByPath.keys()].sort();
  const actualPaths = content.pages.map(({ pagePath }) => pagePath).sort();
  if (JSON.stringify(actualPaths) !== JSON.stringify(expectedPaths)) {
    const actual = new Set(actualPaths);
    const expected = new Set(expectedPaths);
    throw new Error(
      `${language}: Help metadata coverage mismatch`
      + `; missing: ${expectedPaths.filter((path) => !actual.has(path)).join(", ") || "none"}`
      + `; unexpected: ${actualPaths.filter((path) => !expected.has(path)).join(", ") || "none"}.`,
    );
  }
  if (content.homeTopicId !== "app") {
    throw new Error(`${language}: root index.md must be the app Help topic.`);
  }
  return Object.freeze({
    homeTopicId: content.homeTopicId,
    directories: Object.freeze(Object.fromEntries(content.directories.map((directory) => [directory.path, directory]))),
    items: content.items,
    pages: content.pages,
    title: content.meta.title,
  });
}
