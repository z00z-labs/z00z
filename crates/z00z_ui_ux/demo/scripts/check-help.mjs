import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { compileHelp } from "./compile-help.mjs";
import { loadNavigationHelp } from "./help/navigation-help.mjs";
import { helpRecords, pageFile } from "./help/navigation-contract.mjs";
import { serializeNavigationManifest } from "./help/write-navigation-manifest.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const manifestPath = resolve(demoRoot, "help/topics.yaml");
const cataloguePath = resolve(demoRoot, "scripts/generated/help-catalog.js");

const expectedManifest = serializeNavigationManifest();
const actualManifest = await readFile(manifestPath, "utf8");
assert.equal(actualManifest, expectedManifest, "Help navigation manifest is stale.");

const records = helpRecords();
const { documents } = await loadNavigationHelp(demoRoot);
const contexts = records.filter(({ scope }) => scope === "context");
const dialogs = records.filter(({ scope }) => scope === "dialog");

assert.equal(contexts.length, 63, "Help must cover every current Demo route.");
assert.equal(dialogs.length, 9, "Help must cover every supported dialog view.");
assert.equal(documents.length, records.length, "Every Help navigation record requires one English Markdown page.");
assert.equal(new Set(documents.map(({ id }) => id)).size, records.length, "Help page IDs must be unique.");

for (const document of documents) {
  const record = records.find(({ id }) => id === document.id);
  assert.ok(record, `Unknown Help document: ${document.id}`);
  assert.equal(document.pagePath, pageFile(record), `Help path drift: ${document.id}`);
  assert.match(document.html, /<h2 id="current-view">App View<\/h2>/u, `App View is missing: ${document.id}`);
  assert.match(document.html, /<h2 id="terms-and-controls">Terms and controls<\/h2>/u, `Terms section is missing: ${document.id}`);
  assert.doesNotMatch(document.html, /help-sync:source/u, `Sync provenance leaked into Help: ${document.id}`);
}

const expectedCatalogue = await compileHelp(demoRoot);
const actualCatalogue = await readFile(cataloguePath, "utf8");
assert.equal(actualCatalogue, expectedCatalogue, "Generated Help catalogue is stale.");

console.log(`Navigation Help ready: ${contexts.length} routed views, ${dialogs.length} dialog views, ${documents.length} English pages.`);
