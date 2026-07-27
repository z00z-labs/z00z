import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { helpRecords, pageFile } from "./navigation-contract.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "../..");
const manifestPath = resolve(demoRoot, "help/topics.yaml");

function serializeRecord(record) {
  return [
    `  - id: ${record.id}`,
    `    file: ${pageFile(record)}`,
    `    label_key: ${record.labelKey}`,
    `    route: ${record.routeId || "none"}`,
    `    scope: ${record.scope}`,
  ].join("\n");
}

export function serializeNavigationManifest() {
  return `version: 2\nsource_locale: en\ntopics:\n${helpRecords().map(serializeRecord).join("\n")}\n`;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const expected = serializeNavigationManifest();
  const checkOnly = process.argv.includes("--check");

  if (checkOnly) {
    const actual = await readFile(manifestPath, "utf8");
    if (actual !== expected) throw new Error("Help navigation manifest is stale; run node scripts/help/write-navigation-manifest.mjs.");
  } else {
    await writeFile(manifestPath, expected, "utf8");
    console.log(`Updated ${manifestPath}`);
  }
}
