import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { synchronizeHelp } from "./sync-help.mjs";
import { loadHelpLocales, loadHelpSource, parseHelpMarkdown } from "./help-source.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const outputPath = resolve(demoRoot, "scripts/generated/help-catalog.js");

export async function compileHelp(root = demoRoot) {
  const { lut } = await loadHelpSource(root);
  const localeIds = await loadHelpLocales(root);
  const catalogues = {};

  for (const locale of localeIds) {
    catalogues[locale] = {};
    for (const topic of lut.topics) {
      const path = resolve(root, "help", locale, `${topic.file}.md`);
      const document = parseHelpMarkdown(await readFile(path, "utf8"), path);
      if (document.id !== topic.id) throw new Error(`${path}: expected id ${topic.id}`);
      if (document.scope !== topic.scope) throw new Error(`${path}: expected scope ${topic.scope}`);
      catalogues[locale][topic.id] = document;
    }
  }

  const payload = { version: lut.version, locales: localeIds, topics: lut.topics, catalogues };
  return `"use strict";\n((root) => {\n  const deepFreeze = (value) => {\n    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;\n    Object.values(value).forEach(deepFreeze);\n    return Object.freeze(value);\n  };\n  root.Z00ZHelpCatalog = deepFreeze(${JSON.stringify(payload, null, 2)});\n})(typeof window === "undefined" ? globalThis : window);\n`;
}

async function main() {
  await synchronizeHelp(demoRoot);
  const output = await compileHelp(demoRoot);
  await mkdir(dirname(outputPath), { recursive: true });
  await writeFile(outputPath, output, "utf8");
  console.log(`Compiled Help catalogue: ${outputPath}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
