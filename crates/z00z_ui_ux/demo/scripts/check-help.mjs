import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { compileHelp } from "./compile-help.mjs";
import { loadNavigationHelp } from "./help/navigation-help.mjs";
import { helpRecords, pageFile } from "./help/navigation-contract.mjs";
import { serializeNavigationManifest } from "./help/write-navigation-manifest.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const helpRoot = resolve(demoRoot, "help");
const manifestPath = resolve(helpRoot, "topics.yaml");
const cataloguePath = resolve(demoRoot, "scripts/generated/help-catalog.js");

function svgSymbols(source) {
  return new Map(
    [...source.matchAll(/<symbol id="(i-[^"]+)"[^>]*>[\s\S]*?<\/symbol>/gu)]
      .map((match) => [match[1], match[0]]),
  );
}

async function markdownFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await markdownFiles(path));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(path);
    }
  }
  return files;
}

async function emptyContentDirectories(directory) {
  const emptyDirectories = [];
  const entries = (await readdir(directory, { withFileTypes: true }))
    .filter((entry) => !entry.name.startsWith("_") && !entry.name.startsWith("."));
  if (entries.length === 0) {
    emptyDirectories.push(directory);
  }
  for (const entry of entries.filter((entry) => entry.isDirectory())) {
    emptyDirectories.push(...await emptyContentDirectories(resolve(directory, entry.name)));
  }
  return emptyDirectories;
}

function contentFingerprint(source) {
  return createHash("sha256")
    .update(source
      .replace(/^---[\s\S]*?\n---\n/u, "")
      .replace(/<!--[\s\S]*?-->/gu, "")
      .replace(/\s+/gu, " ")
      .trim())
    .digest("hex");
}

function markdownStructure(source) {
  return {
    bulletItems: [...source.matchAll(/^-\s+\S/gmu)].length,
    images: [...source.matchAll(/^!\[[^\]]*\]\([^)]+\)\s*$/gmu)].length,
    orderedSteps: [...source.matchAll(/^\d+\.\s+\S/gmu)].length,
    tableLines: [...source.matchAll(/^\|.*\|\s*$/gmu)].length,
  };
}

const expectedManifest = serializeNavigationManifest();
const actualManifest = await readFile(manifestPath, "utf8");
assert.equal(actualManifest, expectedManifest, "Help navigation manifest is stale.");

const topLevelHelpFiles = (await readdir(helpRoot, { withFileTypes: true }))
  .filter((entry) => entry.isFile())
  .map(({ name }) => name)
  .sort();
assert.deepEqual(
  topLevelHelpFiles,
  ["topics.yaml"],
  "Help root contains obsolete archives, templates, state, backups, or other non-runtime files.",
);

const requiredRecords = helpRecords();
const { documents, navigation, records } = await loadNavigationHelp(demoRoot);
const supportedLocales = globalThis.Z00ZLocaleRegistry.map(({ id }) => id);
const contexts = records.filter(({ scope }) => scope === "context");
const dialogs = records.filter(({ scope }) => scope === "dialog");
const guides = records.filter(({ scope }) => scope === "guide");
const articles = records.filter(({ scope }) => scope === "article");

assert.equal(contexts.length, 89, "Help must cover every current Demo route and the separate Split context page.");
assert.ok(
  contexts.some(({ id, routeId }) => id === "wallet.split" && routeId === "wallet.merge-split"),
  "Help must expose Wallet: Split as a separate page for the Merge/Split route.",
);
assert.equal(dialogs.length, 9, "Help must cover every supported dialog view.");
assert.ok(guides.some(({ id }) => id === "dapps.security-model"), "Help must contain the dApps security guide.");
for (const articleId of [
  "help.how-to",
  "help.tips-and-tricks",
  "help.video-tutorials",
  "help.faq",
  "help.report-issues",
]) {
  assert.ok(articles.some(({ id }) => id === articleId), `Help general guide is missing: ${articleId}.`);
}
assert.equal(documents.length, records.length, "Every Help navigation record requires one English Markdown page.");
assert.equal(new Set(documents.map(({ id }) => id)).size, records.length, "Help page IDs must be unique.");
for (const requiredRecord of requiredRecords) {
  assert.ok(records.some(({ id }) => id === requiredRecord.id), `Required App Help topic is missing: ${requiredRecord.id}.`);
}

const primaryItems = navigation.items;
const primarySections = primaryItems.filter(({ type }) => type === "section");
const navigationTopicIds = (items) => items.flatMap((item) => [
  ...(item.topicId ? [item.topicId] : []),
  ...navigationTopicIds(item.children || []),
]);
const navigationItems = (items) => items.flatMap((item) => [
  item,
  ...navigationItems(item.children || []),
]);
const exposedAdditionalTopicIds = new Set([
  ...navigationTopicIds(primaryItems),
  ...Object.values(navigation.contexts).flatMap((items) => navigationTopicIds(items)),
]);
assert.ok(
  primarySections.every(({ children }) => children.every(({ type }) => type === "article")),
  "Help primary navbar may contain only one nested menu level.",
);
for (const record of records.filter(({ nodeId, scope }) => (
  !nodeId && ["article", "guide"].includes(scope)
))) {
  assert.ok(
    exposedAdditionalTopicIds.has(record.id),
    `Additional Help article is not exposed by its primary or Main View navigation: ${record.id}.`,
  );
}
for (const item of navigationItems(primaryItems).filter(({ contextId }) => contextId)) {
  const directory = navigation.directories[item.directoryPath];
  assert.ok(directory, `Help content folder is missing for Main View navigation: ${item.directoryPath}.`);
  const expectedTopics = directory.entries.flatMap((entryName) => {
    const record = records.find((candidate) => (
      candidate.pagePath.slice(0, -1).join("/") === item.directoryPath
      && candidate.pagePath.at(-1) === entryName
    ));
    return record ? [record.id] : [];
  });
  const actualTopics = (navigation.contexts[item.contextId] || []).map(({ topicId }) => topicId);
  if (expectedTopics.length > 1) {
    assert.deepEqual(
      actualTopics,
      expectedTopics,
      `Main View navigation must follow ${item.directoryPath}/_meta.yaml.`,
    );
  }
}
assert.deepEqual(
  navigation.contexts["wallet.assets-rights"].map(({ topicId }) => topicId),
  ["wallet.assets", "wallet.vouchers", "wallet.permissions", "wallet.quarantine", "asset.details"],
  "The Assets folder must expose every ordered article in Main View navigation.",
);
const expectedAppRootIds = globalThis.Z00ZDemo.navigationChildren()
  .filter(({ id }) => !["help", "logout"].includes(id))
  .map(({ id }) => id);
assert.deepEqual(
  primaryItems.filter(({ id }) => id !== "guides").map(({ id }) => id),
  expectedAppRootIds,
  "Help primary navbar must preserve the Demo App root navigation.",
);
for (const node of globalThis.Z00ZDemo.navigationChildren()) {
  if (node.target.kind !== "branch" || ["help", "logout"].includes(node.id)) continue;
  const section = primarySections.find(({ id }) => id === node.id);
  assert.ok(section, `Help primary navbar branch is missing: ${node.id}.`);
  const expectedChildren = globalThis.Z00ZDemo.navigationChildren(node.id).map(({ id }) => id);
  assert.deepEqual(
    section.children.filter(({ id }) => expectedChildren.includes(id)).map(({ id }) => id),
    expectedChildren,
    `Help primary navbar differs from the Demo App branch: ${node.id}.`,
  );
  for (const workspace of globalThis.Z00ZDemo.navigationChildren(node.id).filter(({ target }) => target.kind === "workspace")) {
    const workspaceChildIds = globalThis.Z00ZDemo.navigationChildren(workspace.id).map(({ id }) => id);
    assert.deepEqual(
      section.children.filter(({ id }) => workspaceChildIds.includes(id)),
      [],
      `Workspace-local routes leaked into the Help primary navbar: ${workspace.id}.`,
    );
  }
}

for (const document of documents) {
  const record = records.find(({ id }) => id === document.id);
  assert.ok(record, `Unknown Help document: ${document.id}`);
  assert.equal(document.pagePath, pageFile(record), `Help path drift: ${document.id}`);
  assert.ok(document.title.trim(), `Help title is missing: ${document.id}`);
  if (record.scope === "article") {
    assert.match(document.html, /<h2 id="overview">Overview<\/h2>/u, `Overview is missing: ${document.id}`);
    assert.match(document.html, /<h2 id="how-to-use-this-guide">How to use this guide<\/h2>/u, `Guide workflow is missing: ${document.id}`);
  } else {
    assert.match(document.html, /<h2 id="current-view">App View<\/h2>/u, `App View is missing: ${document.id}`);
    assert.match(document.html, /<h2 id="terms-and-controls">Terms and controls<\/h2>/u, `Terms section is missing: ${document.id}`);
  }
  assert.doesNotMatch(document.html, /help-sync:source/u, `Sync provenance leaked into Help: ${document.id}`);
}

const localizedDocuments = await Promise.all(supportedLocales.map(async (language) => [
  language,
  (await loadNavigationHelp(demoRoot, language, records)).documents,
]));
for (const [language, localeDocuments] of localizedDocuments) {
  assert.equal(localeDocuments.length, records.length, `${language}: localized Help is incomplete.`);
  assert.equal(new Set(localeDocuments.map(({ id }) => id)).size, records.length, `${language}: localized Help IDs must be unique.`);
  for (const document of localeDocuments) {
    const record = records.find(({ id }) => id === document.id);
    if (record.scope !== "article") {
      assert.match(document.html, /<h2 id="current-view">/u, `${language}: App View is missing: ${document.id}`);
      assert.match(document.html, /<img src="help\/assets\/en\//u, `${language}: App View screenshot is missing: ${document.id}`);
    }
    assert.doesNotMatch(document.html, /help-sync:source/u, `${language}: sync provenance leaked into Help: ${document.id}`);
  }
  for (const [topicId, englishTitle, screenshot] of [
    ["wallet.merge", "Wallet: Merge", "help/assets/en/wallet-merge.png"],
    ["wallet.split", "Wallet: Split", "help/assets/en/wallet-split.png"],
  ]) {
    const document = localeDocuments.find(({ id }) => id === topicId);
    assert.ok(document, `${language}: ${topicId} Help is missing.`);
    assert.deepEqual(
      [...document.html.matchAll(/<h2 id="([^"]+)"/gu)].map((match) => match[1]),
      ["current-view", "overview", "how-to-use-this-view", "terms-and-controls", "safety-and-limits"],
      `${language}: ${topicId} sections differ from English Help.`,
    );
    assert.match(
      document.html,
      new RegExp(`<img src="${screenshot.replaceAll("/", "\\/")}"`, "u"),
      `${language}: ${topicId} App View screenshot is missing.`,
    );
    if (language !== "en") {
      assert.notEqual(document.title, englishTitle, `${language}: ${topicId} title is not translated.`);
      const record = records.find(({ id }) => id === topicId);
      const source = await readFile(resolve(helpRoot, language, pageFile(record)), "utf8");
      const englishSource = await readFile(resolve(helpRoot, "en", pageFile(record)), "utf8");
      assert.doesNotMatch(
        source,
        /^## (?:App View|Overview|How to use this view|Terms and controls|Safety and limits)(?:\s+\{#[^}]+\})?\s*$/mu,
        `${language}: ${topicId} still contains an English section heading.`,
      );
      assert.deepEqual(
        markdownStructure(source),
        markdownStructure(englishSource),
        `${language}: ${topicId} workflow, table, image, or safety structure differs from English Help.`,
      );
      for (const technicalToken of [
        "definition_id",
        "serial_id",
        topicId === "wallet.merge" ? "wallet.asset.merge_assets" : "wallet.asset.split_asset",
      ]) {
        assert.ok(source.includes(technicalToken), `${language}: ${topicId} is missing ${technicalToken}.`);
      }
    }
  }
  const localeRoot = resolve(helpRoot, language);
  const localeMarkdown = await markdownFiles(localeRoot);
  assert.equal(localeMarkdown.length, records.length, `${language}: one Markdown file per topic is required.`);
  assert.deepEqual(
    localeMarkdown.filter((path) => /(?:^|\/)(?:\.temp|_drafts)(?:\/|$)|-draft-\d{8}/u.test(path)),
    [],
    `${language}: draft or temporary Markdown is forbidden.`,
  );
  const bodiesByFingerprint = new Map();
  for (const path of localeMarkdown) {
    const source = await readFile(path, "utf8");
    const fingerprint = contentFingerprint(source);
    bodiesByFingerprint.set(
      fingerprint,
      [...(bodiesByFingerprint.get(fingerprint) || []), relative(localeRoot, path).split(sep).join("/")],
    );
  }
  const duplicateBodies = [...bodiesByFingerprint.values()].filter((paths) => paths.length > 1);
  assert.deepEqual(duplicateBodies, [], `${language}: duplicate Help article bodies are forbidden.`);
  assert.deepEqual(
    (await emptyContentDirectories(localeRoot)).map((path) => relative(localeRoot, path).split(sep).join("/")),
    [],
    `${language}: empty Help content directories are forbidden.`,
  );
}

const expectedCatalogue = await compileHelp(demoRoot);
const actualCatalogue = await readFile(cataloguePath, "utf8");
assert.equal(actualCatalogue, expectedCatalogue, "Generated Help catalogue is stale.");

const [demoDocument, helpDocument] = await Promise.all([
  readFile(resolve(demoRoot, "index.html"), "utf8"),
  readFile(resolve(demoRoot, "help.html"), "utf8"),
]);
const demoSymbols = svgSymbols(demoDocument);
const helpSymbols = svgSymbols(helpDocument);
const visibleHelpNodes = globalThis.Z00ZDemo.NAVIGATION_NODES
  .filter((node) => node.isVisible && node.target.kind !== "action");

for (const node of visibleHelpNodes) {
  const iconId = `i-${node.iconId}`;
  assert.equal(
    helpSymbols.get(iconId),
    demoSymbols.get(iconId),
    `Help icon differs from the Demo navigation: ${node.id}`,
  );
}

console.log(`Metadata Help ready: ${contexts.length} context pages, ${dialogs.length} dialog views, ${guides.length} security guide, ${articles.length} general guides, ${documents.length} pages in ${supportedLocales.length} locales.`);
