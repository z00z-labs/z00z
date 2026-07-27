import { access, copyFile, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import {
  helpDocumentPath,
  loadHelpLocales,
  loadHelpSource
} from "./help-source.mjs";
import {
  HELP_GROUP_DEFINITIONS,
  helpTopicDefinitions
} from "./help-topics.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const helpRoot = resolve(demoRoot, "help");
const manifestPath = resolve(helpRoot, "preserved-sources.json");
const INDEX_ICON = "mdi:alphabet-a-box-outline";

const FOLDER_LAYOUT = Object.freeze([
  { path: "app", labelKey: "help.title", icon: "question", order: ["index", "app", "about"] },
  { path: "wallets", labelKey: "app.wallets", icon: "wallet", order: ["index", "assets-rights", "quarantine", "send", "receive", "history", "staking", "backup", "settings"] },
  { path: "wallets/assets-rights", labelKey: "navigation.assets", icon: "assets", order: ["index", "assets", "vouchers", "permissions", "asset-details"] },
  { path: "wallets/staking", labelKey: "navigation.staking", icon: "staking", order: ["index", "stake", "unstake"] },
  { path: "wallets/settings", labelKey: "navigation.walletSettings", icon: "settings", order: ["index", "general", "security", "backup", "policies", "advanced"] },
  { path: "telemetry", labelKey: "navigation.telemetry", icon: "network", order: ["index", "reticulum", "onionnet", "aggregators", "watchers", "explorer"] },
  { path: "telemetry/reticulum", labelKey: "navigation.reticulum", icon: "reticulum-node", order: ["index", "overview", "node", "interfaces", "radio", "entrypoints", "paths", "probes", "links"] },
  { path: "telemetry/onionnet", labelKey: "navigation.onionnet", icon: "shield", order: ["index", "overview", "epoch", "privacy", "transport", "queues", "probation", "ingress"] },
  { path: "telemetry/aggregators", labelKey: "navigation.aggregators", icon: "staking", order: ["index", "overview", "ingress", "planning", "placement", "publication", "recovery"] },
  { path: "telemetry/watchers", labelKey: "navigation.watchers", icon: "eye", order: ["index", "overview", "alerts", "publication", "providers", "censorship", "evidence", "alert-detail"] },
  { path: "telemetry/explorer", labelKey: "navigation.explorer", icon: "search", order: ["index", "overview", "search", "checkpoints", "batches", "evidence", "detail"] },
  { path: "dapps", labelKey: "navigation.dapps", icon: "spark", order: ["index", "discover", "installed", "connections", "permissions", "swap", "exchange", "detail", "permission-review"] },
  { path: "messenger", labelKey: "navigation.messenger", icon: "message", order: ["index", "inbox", "sent", "conversations", "detail", "request-review"] },
  { path: "contacts", labelKey: "navigation.contacts", icon: "user", order: ["index", "contacts", "detail", "identity-review"] },
  { path: "data-storage", labelKey: "navigation.dataStorage", icon: "storage", order: ["index", "disk-usage", "network-usage"] },
  { path: "settings", labelKey: "navigation.settings", icon: "settings", order: ["index", "general", "notifications", "appearance"] },
  { path: "privacy", labelKey: "navigation.privacy", icon: "shield", order: ["index", "privacy-statement"] }
]);

const SUPPLEMENTAL_MARKDOWN = Object.freeze([
  "about.md",
  "faq.md",
  "how-to.md",
  "report-issues.md",
  "tips-and-tricks.md",
  "video-tutorials.md",
  "privacy/privacy-statement.md"
]);

function yamlString(value) {
  return `"${String(value).replaceAll("\\", "\\\\").replaceAll('"', '\\"')}"`;
}

async function exists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function loadI18n() {
  const context = vm.createContext({ window: {}, Intl });
  for (const relativePath of [
    "scripts/port/locale-registry.js",
    "i18n.js",
    "locales/en.js",
    "locales/ru.js",
    "locales/fr.js",
    "locales/de.js",
    "locales/es.js",
    "locales/pt.js",
    "locales/ko.js",
    "locales/tr.js",
    "locales/ja.js",
    "locales/zh-Hans.js",
    "locales/navigation.js"
  ]) {
    const sourcePath = resolve(demoRoot, relativePath);
    vm.runInContext(await readFile(sourcePath, "utf8"), context, { filename: sourcePath });
  }
  return context.window.Z00ZI18n;
}

async function loadContract() {
  const context = vm.createContext({ URLSearchParams, window: {} });
  const sourcePath = resolve(demoRoot, "scripts/port/contracts.js");
  vm.runInContext(await readFile(sourcePath, "utf8"), context, { filename: sourcePath });
  return context.window.Z00ZDemo.PORT_CONTRACT;
}

async function mirrorSupplementalMarkdown(locales) {
  for (const relativePath of SUPPLEMENTAL_MARKDOWN) {
    const source = resolve(helpRoot, "en", relativePath);
    if (!await exists(source)) continue;
    for (const locale of locales) {
      if (locale === "en") continue;
      const destination = resolve(helpRoot, locale, relativePath);
      if (await exists(destination)) continue;
      await mkdir(dirname(destination), { recursive: true });
      await copyFile(source, destination);
    }
  }
}

async function migrateRuntimeTopics(locales, oldTopics, newTopics) {
  const oldById = new Map(oldTopics.map((topic) => [topic.id, topic]));
  const preserved = [];
  for (const topic of newTopics) {
    const oldTopic = oldById.get(topic.id);
    if (!oldTopic) continue;
    const oldRelative = `${oldTopic.group}/${oldTopic.file}.md`;
    const newRelative = `${topic.group}/${topic.file}.md`;
    if (oldRelative === newRelative) continue;
    preserved.push(oldRelative);
    for (const locale of locales) {
      const source = helpDocumentPath(demoRoot, locale, oldTopic);
      const destination = helpDocumentPath(demoRoot, locale, topic);
      if (await exists(destination)) continue;
      await mkdir(dirname(destination), { recursive: true });
      await copyFile(source, destination);
    }
  }
  return preserved;
}

async function writeFolderMaps(locales, i18n) {
  const groupOrder = HELP_GROUP_DEFINITIONS.map(({ id }) => id);
  const rootOrder = ["index", ...groupOrder, "privacy"];
  for (const locale of locales) {
    const localeRoot = resolve(helpRoot, locale);
    const helpTitle = `Z00Z ${i18n.translate(locale, "help.title")}`;
    await writeFile(resolve(localeRoot, "index.md"), `---\ntitle: ${yamlString(helpTitle)}\ndescription: ${yamlString(i18n.translate(locale, "help.contents"))}\ndifficulty: basic\nicon: ${INDEX_ICON}\ntoc: true\n---\n`, "utf8");
    await writeFile(resolve(localeRoot, "_meta.yaml"), `title: ${yamlString(helpTitle)}\norder:\n${rootOrder.map((entry) => `  - ${entry}`).join("\n")}\n`, "utf8");

    for (const folder of FOLDER_LAYOUT) {
      const directory = resolve(localeRoot, folder.path);
      const title = i18n.translate(locale, folder.labelKey);
      await mkdir(directory, { recursive: true });
      await writeFile(resolve(directory, "index.md"), `---\ntitle: ${yamlString(title)}\ndescription: ${yamlString(`${title} · ${i18n.translate(locale, "help.contents")}`)}\ndifficulty: basic\nicon: ${INDEX_ICON}\ntoc: true\n---\n`, "utf8");
      await writeFile(resolve(directory, "_meta.yaml"), `title: ${yamlString(title)}\nicon: ${folder.icon}\norder:\n${folder.order.map((entry) => `  - ${entry}`).join("\n")}\n`, "utf8");
    }
  }
}

async function updatePreservedManifest(preservedRuntimePaths) {
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const landingPages = ["index.md", ...FOLDER_LAYOUT.map(({ path }) => `${path}/index.md`)];
  const paths = [...new Set([
    ...manifest.paths,
    ...preservedRuntimePaths,
    ...SUPPLEMENTAL_MARKDOWN,
    ...landingPages
  ])].sort();
  await writeFile(manifestPath, `${JSON.stringify({
    ...manifest,
    purpose: "Original tracked and superseded flat Help Markdown retained for provenance; these paths are not runtime catalogue entries.",
    paths,
    optionalLocalePaths: {}
  }, null, 2)}\n`, "utf8");
}

const locales = await loadHelpLocales(demoRoot);
const { lut } = await loadHelpSource(demoRoot);
const contract = await loadContract();
const newTopics = helpTopicDefinitions(contract);
const i18n = await loadI18n();
const preservedRuntimePaths = await migrateRuntimeTopics(locales, lut.topics, newTopics);
await mirrorSupplementalMarkdown(locales);
await writeFolderMaps(locales, i18n);
await updatePreservedManifest(preservedRuntimePaths);

const relativeFolders = (await readdir(resolve(helpRoot, "en"), { withFileTypes: true }))
  .filter((entry) => entry.isDirectory())
  .map((entry) => relative(helpRoot, resolve(helpRoot, "en", entry.name)));
console.log(`Migrated Help layout for ${locales.length} locales; preserved ${preservedRuntimePaths.length} flat runtime paths; English sections: ${relativeFolders.join(", ")}`);
