import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { compileHelp } from "./compile-help.mjs";
import { loadNavigationHelp } from "./help/navigation-help.mjs";
import { helpRecords, helpTitle, pageFile } from "./help/navigation-contract.mjs";
import { serializeNavigationManifest } from "./help/write-navigation-manifest.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const helpRoot = resolve(demoRoot, "help");
const manifestPath = resolve(helpRoot, "topics.yaml");
const cataloguePath = resolve(demoRoot, "scripts/generated/help-catalog.js");
const englishRoot = resolve(helpRoot, "en");

function svgSymbols(source) {
  return new Map(
    [...source.matchAll(/<symbol id="(i-[^"]+)"[^>]*>[\s\S]*?<\/symbol>/gu)]
      .map((match) => [match[1], match[0]]),
  );
}

async function activeEnglishSources(directory = englishRoot) {
  const sources = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.name === ".temp" || entry.name === "_generated") continue;
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      sources.push(...await activeEnglishSources(path));
    } else if (/\.(?:md|ya?ml)$/u.test(entry.name)) {
      sources.push(relative(englishRoot, path).split(sep).join("/"));
    }
  }
  return sources.sort();
}

async function emptyActiveEnglishDirectories(directory = englishRoot) {
  const emptyDirectories = [];
  const entries = (await readdir(directory, { withFileTypes: true }))
    .filter((entry) => entry.name !== ".temp");
  if (directory !== englishRoot && entries.length === 0) {
    emptyDirectories.push(relative(englishRoot, directory).split(sep).join("/"));
  }
  for (const entry of entries.filter((entry) => entry.isDirectory())) {
    emptyDirectories.push(...await emptyActiveEnglishDirectories(resolve(directory, entry.name)));
  }
  return emptyDirectories.sort();
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
  ["README.md", "TEMPLATE.md", "topics.yaml"],
  "Help root contains obsolete archives, state, backups, or other non-runtime files.",
);
assert.deepEqual(
  await emptyActiveEnglishDirectories(),
  [],
  "Active English Help contains empty legacy directories.",
);

const records = helpRecords();
const { documents } = await loadNavigationHelp(demoRoot);
const supportedLocales = globalThis.Z00ZLocaleRegistry.map(({ id }) => id);
const contexts = records.filter(({ scope }) => scope === "context");
const dialogs = records.filter(({ scope }) => scope === "dialog");

assert.equal(contexts.length, 63, "Help must cover every current Demo route.");
assert.equal(dialogs.length, 9, "Help must cover every supported dialog view.");
assert.equal(documents.length, records.length, "Every Help navigation record requires one English Markdown page.");
assert.equal(new Set(documents.map(({ id }) => id)).size, records.length, "Help page IDs must be unique.");

const canonicalPages = new Set(records.map(pageFile));
const unexpectedSources = (await activeEnglishSources()).filter((source) => {
  if (canonicalPages.has(source)) return false;
  const draft = source.match(/^(.*)-draft-\d{8}(?:-\d+)?\.md$/u);
  return !draft || !canonicalPages.has(`${draft[1]}.md`);
});
assert.deepEqual(
  unexpectedSources,
  [],
  "Active English Help contains legacy or non-canonical source files; keep historical material under help/en/.temp only.",
);

for (const document of documents) {
  const record = records.find(({ id }) => id === document.id);
  assert.ok(record, `Unknown Help document: ${document.id}`);
  assert.equal(document.pagePath, pageFile(record), `Help path drift: ${document.id}`);
  assert.equal(document.title, helpTitle(document.id), `Help title format drift: ${document.id}`);
  assert.match(document.html, /<h2 id="current-view">App View<\/h2>/u, `App View is missing: ${document.id}`);
  assert.match(document.html, /<h2 id="terms-and-controls">Terms and controls<\/h2>/u, `Terms section is missing: ${document.id}`);
  assert.doesNotMatch(document.html, /help-sync:source/u, `Sync provenance leaked into Help: ${document.id}`);
}

const localizedDocuments = await Promise.all(supportedLocales
  .filter((language) => language !== "en")
  .map(async (language) => [language, (await loadNavigationHelp(demoRoot, language)).documents]));
for (const [language, localeDocuments] of localizedDocuments) {
  assert.equal(localeDocuments.length, records.length, `${language}: localized Help is incomplete.`);
  assert.equal(new Set(localeDocuments.map(({ id }) => id)).size, records.length, `${language}: localized Help IDs must be unique.`);
  for (const document of localeDocuments) {
    assert.match(document.html, /<h2 id="current-view">/u, `${language}: App View is missing: ${document.id}`);
    assert.match(document.html, /<img src="help\/assets\/en\//u, `${language}: App View screenshot is missing: ${document.id}`);
    assert.doesNotMatch(document.html, /help-sync:source/u, `${language}: sync provenance leaked into Help: ${document.id}`);
  }
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

console.log(`Navigation Help ready: ${contexts.length} routed views, ${dialogs.length} dialog views, ${documents.length} pages in ${supportedLocales.length} locales.`);
