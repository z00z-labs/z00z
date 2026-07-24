import { spawnSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const context = vm.createContext({ Intl, window: {} });
const registrySource = await readFile(resolve(demoRoot, "scripts/port/locale-registry.js"), "utf8");
vm.runInContext(registrySource, context, { filename: "scripts/port/locale-registry.js" });
const localeRegistry = context.window.Z00ZLocaleRegistry;
const localeFiles = Object.fromEntries(localeRegistry.map(({ id, catalogue }) => [id, catalogue]));

for (const resource of ["i18n.js", ...Object.values(localeFiles), "locales/send-exchange.js"]) {
  const source = await readFile(resolve(demoRoot, resource), "utf8");
  vm.runInContext(source, context, { filename: resource });
}

const i18n = context.window.Z00ZI18n;
const translator = process.env.Z00Z_TRANSLATE_COMMAND;
const reports = i18n.auditCatalogues().filter((entry) => entry.language !== "en" && entry.missing.length);

if (!reports.length) {
  console.log("No locale keys require machine translation.");
  process.exit(0);
}

if (!translator) {
  console.error("Set Z00Z_TRANSLATE_COMMAND to a local translation bridge before syncing locale drafts.");
  for (const report of reports) console.error(report.language + ": " + report.missing.join(", "));
  process.exit(1);
}

const english = i18n.catalogue("en");
for (const { language, missing } of reports) {
  const sourceMessages = Object.fromEntries(missing.map((key) => [key, english[key]]));
  const translated = spawnSync(translator, [language], {
    shell: false,
    encoding: "utf8",
    input: JSON.stringify({ language, sourceLanguage: "en", messages: sourceMessages })
  });
  if (translated.status !== 0) throw new Error("Local translator failed for " + language + ": " + translated.stderr);

  const draft = JSON.parse(translated.stdout);
  const unresolved = missing.filter((key) => typeof draft[key] !== "string" || !draft[key].trim());
  if (unresolved.length) throw new Error("Local translator left " + language + " keys unresolved: " + unresolved.join(", "));

  const merged = { ...i18n.catalogue(language), ...draft };
  const body = JSON.stringify(merged, null, 2);
  const output = '"use strict";\n\nwindow.Z00ZI18n.registerLocale("' + language + '", ' + body + ');\n';
  await writeFile(resolve(demoRoot, localeFiles[language]), output, "utf8");
}

console.log("Machine-translation drafts synchronized. Run node scripts/check-locales.mjs before committing.");
