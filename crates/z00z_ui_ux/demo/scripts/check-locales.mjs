import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const context = vm.createContext({ Intl, window: {} });
const registrySource = await readFile(resolve(demoRoot, "scripts/port/locale-registry.js"), "utf8");
vm.runInContext(registrySource, context, { filename: "scripts/port/locale-registry.js" });
const localeRegistry = context.window.Z00ZLocaleRegistry;
const resources = ["i18n.js", ...localeRegistry.map(({ catalogue }) => catalogue)];

for (const resource of resources) {
  const source = await readFile(resolve(demoRoot, resource), "utf8");
  vm.runInContext(source, context, { filename: resource });
}

const [appSource, markupSource] = await Promise.all([
  readFile(resolve(demoRoot, "app.js"), "utf8"),
  readFile(resolve(demoRoot, "index.html"), "utf8")
]);
const referencedKeys = new Set([
  ...appSource.matchAll(/\bt\(\s*["']([\w.-]+)["']/g),
  ...markupSource.matchAll(/\bdata-i18n=["']([\w.-]+)["']/g)
].map((match) => match[1]));
const english = context.window.Z00ZI18n.catalogue("en");
const undefinedKeys = [...referencedKeys].filter((key) => !(key in english));
const failures = context.window.Z00ZI18n.auditCatalogues().filter((entry) => !entry.ready);
if (failures.length || undefinedKeys.length) {
  for (const entry of failures) {
    console.error(entry.language + ": missing [" + entry.missing.join(", ") + "], extra [" + entry.extra.join(", ") + "]");
  }
  if (undefinedKeys.length) console.error("UI references absent from English source catalogue: " + undefinedKeys.join(", "));
  process.exitCode = 1;
} else {
  console.log(`Locale catalogue check passed: ${localeRegistry.length}/${localeRegistry.length} language packs match English source keys and ${referencedKeys.size} static UI keys resolve.`);
}
