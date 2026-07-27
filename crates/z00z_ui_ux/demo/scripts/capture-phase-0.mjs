import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

import { loadHelpLocales, loadHelpSource } from "./help-source.mjs";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(demoRoot, "../../..");
const evidenceRoot = resolve(demoRoot, "evidence/phase-0");
const evidenceDate = "2026-07-26";
const screenshotRoot = resolve(process.argv[2] || join(
  repoRoot,
  "crates/z00z_storage/outputs/checkpoint/phase-110/demo-plan-2-phase-0/current",
));

async function hashFile(filePath) {
  const source = await readFile(filePath);
  return createHash("sha256").update(source).digest("hex");
}

async function writeJson(fileName, value) {
  await mkdir(evidenceRoot, { recursive: true });
  await writeFile(join(evidenceRoot, fileName), `${JSON.stringify(value, undefined, 2)}\n`);
}

function flattenKeys(value, prefix = "") {
  return Object.entries(value).flatMap(([key, entry]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    return entry && typeof entry === "object" && !Array.isArray(entry)
      ? flattenKeys(entry, path)
      : [path];
  });
}

function collectFieldValues(value, field, target = new Set()) {
  if (!value || typeof value !== "object") return target;
  if (typeof value[field] === "string") target.add(value[field]);
  for (const entry of Object.values(value)) collectFieldValues(entry, field, target);
  return target;
}

async function loadDemo() {
  const sandbox = {
    URLSearchParams,
    structuredClone,
  };
  sandbox.globalThis = sandbox;
  for (const fileName of [
    "scripts/port/contracts.js",
    "scripts/port/fixtures.js",
    "scripts/port/presentation-state.js",
    "scripts/port/icon-registry.js",
  ]) {
    const filePath = resolve(demoRoot, fileName);
    vm.runInNewContext(await readFile(filePath, "utf8"), sandbox, { filename: filePath });
  }
  return sandbox.Z00ZDemo;
}

async function loadEnglish() {
  let catalogue;
  const sandbox = {
    window: {
      Z00ZI18n: {
        registerLocale: (_locale, value) => {
          catalogue = value;
        },
      },
    },
  };
  const sourcePath = resolve(demoRoot, "locales/en.js");
  vm.runInNewContext(await readFile(sourcePath, "utf8"), sandbox, { filename: sourcePath });
  return catalogue;
}

function routeStates(contract) {
  return contract.views.flatMap((view) => {
    if (view === "wallet") {
      return contract.walletSections.map((walletSection) => ({ view, walletSection }));
    }
    if (view === "wallet-settings") {
      return contract.walletSettingsSections.map((walletSettingsSection) => ({
        view,
        walletSettingsSection,
      }));
    }
    if (view === "settings") {
      return contract.settingsSections.map((settingsSection) => ({ view, settingsSection }));
    }
    if (view === "telemetry") {
      return contract.telemetrySources.flatMap((telemetrySource) => (
        contract.telemetryTabs[telemetrySource].map((tab) => ({
          view,
          telemetrySource,
          [`${telemetrySource}TelemetryTab`]: tab,
        }))
      ));
    }
    return [{ view }];
  });
}

async function localeInventory(locales, topics) {
  const result = {};
  for (const locale of locales) {
    const localeRoot = resolve(demoRoot, "help", locale);
    const entries = await readdir(localeRoot, { withFileTypes: true });
    const directories = entries.filter((entry) => entry.isDirectory()).map(({ name }) => name).sort();
    const looseMarkdown = entries
      .filter((entry) => entry.isFile() && entry.name.endsWith(".md"))
      .map(({ name }) => name)
      .sort();
    const topicFiles = topics.map(({ group, file }) => `${group}/${file}.md`);
    result[locale] = { directories, looseMarkdown, topicFiles };
  }
  return result;
}

function tokenMap(source, prefix) {
  const expression = new RegExp(`--${prefix}([a-z0-9-]+):\\s*([^;]+);`, "g");
  return Object.fromEntries([...source.matchAll(expression)].map((match) => [
    match[1],
    match[2].trim(),
  ]));
}

async function screenshotManifest() {
  const entries = await readdir(screenshotRoot, { withFileTypes: true });
  const screenshots = [];
  for (const entry of entries.filter(({ name }) => name.endsWith(".png")).sort((a, b) => (
    a.name.localeCompare(b.name)
  ))) {
    const filePath = join(screenshotRoot, entry.name);
    screenshots.push({
      file: relative(repoRoot, filePath),
      sha256: await hashFile(filePath),
    });
  }
  const auditPath = join(screenshotRoot, "responsive-layout-audit.json");
  const audit = JSON.parse(await readFile(auditPath, "utf8"));
  const failed = audit
    .filter(({ issues }) => issues.length > 0)
    .map(({ viewport, route, viewportWidth, documentWidth, issues }) => ({
      viewport,
      route,
      viewportWidth,
      documentWidth,
      issueTypes: [...new Set(issues.map(({ type }) => type))].sort(),
    }));
  return {
    capturedOn: evidenceDate,
    root: relative(repoRoot, screenshotRoot),
    responsiveGeometryGate: {
      status: failed.length === 0 ? "passed" : "failed",
      suppressedFailures: 0,
    },
    screenshotCount: screenshots.length,
    auditedRouteViewportPairs: audit.length,
    failedRouteViewportPairs: failed,
    screenshots,
  };
}

const demo = await loadDemo();
const contract = demo.PORT_CONTRACT;
const english = await loadEnglish();
const locales = await loadHelpLocales(demoRoot);
const { lut } = await loadHelpSource(demoRoot);
const colorsPath = resolve(demoRoot, "styles/colors.css");
const appPath = resolve(demoRoot, "app.js");
const smokePath = resolve(demoRoot, "smoke.spec.js");
const componentPath = resolve(demoRoot, "styles/components.css");
const colorsSource = await readFile(colorsPath, "utf8");
const appSource = await readFile(appPath, "utf8");
const smokeSource = await readFile(smokePath, "utf8");
const componentSource = await readFile(componentPath, "utf8");
const fixtureIds = [...collectFieldValues(demo.INITIAL_WALLET_FIXTURES, "id")]
  .concat([...collectFieldValues(demo.ASSET_CATALOG, "id")])
  .filter((value, index, values) => values.indexOf(value) === index)
  .sort();
const badgeClasses = [...new Set(
  [...componentSource.matchAll(/\.status-badge\.([a-z0-9-]+)/g)].map((match) => match[1]),
)].sort();
const badgeLabels = [...new Set(
  [...appSource.matchAll(/<span class="status-badge[^"]*">([^<$][^<]*)<\/span>/g)]
    .map((match) => match[1].trim())
    .filter(Boolean),
)].sort();
const smokeAssertions = [...smokeSource.matchAll(/^test\("([^"]+)"/gm)].map((match) => match[1]);
const sourceFiles = [
  "app.js",
  "help/topics.yaml",
  "scripts/port/contracts.js",
  "scripts/port/fixtures.js",
  "scripts/port/help-registry.js",
  "scripts/port/icon-registry.js",
  "scripts/port/locale-registry.js",
  "scripts/port/mock-wallet-gateway.js",
  "scripts/port/presentation-state.js",
  "smoke.spec.js",
  "styles/colors.css",
];
const sourceHashes = {};
for (const fileName of sourceFiles) {
  sourceHashes[fileName] = await hashFile(resolve(demoRoot, fileName));
}

await writeJson("current-inventory.json", {
  schemaVersion: 1,
  capturedOn: evidenceDate,
  sources: sourceHashes,
  routes: {
    contractVersion: contract.version,
    viewIds: contract.views,
    walletSectionIds: contract.walletSections,
    walletSettingsSectionIds: contract.walletSettingsSections,
    settingsSectionIds: contract.settingsSections,
    networkSectionIds: contract.networkSections,
    telemetrySourceIds: contract.telemetrySources,
    telemetryTabIds: contract.telemetryTabs,
    resolvedStates: routeStates(contract),
  },
  help: {
    topicCount: lut.topics.length,
    topics: lut.topics,
    localeTrees: await localeInventory(locales, lut.topics),
  },
  locales: {
    ids: locales,
    englishKeyCount: flattenKeys(english).length,
    englishKeys: flattenKeys(english).sort(),
  },
  icons: {
    navigationAndContent: demo.ICON_NAMES,
    objectFamilies: Object.keys(demo.OBJECT_FAMILY_ICON_LUT).sort(),
    objectTypes: Object.fromEntries(Object.entries(demo.OBJECT_TYPE_ICON_LUT).map(
      ([family, entries]) => [family, Object.keys(entries).sort()],
    )),
  },
  appearance: {
    themeIds: ["dark", "light"],
    paletteIds: demo.PALETTE_OPTIONS.map(({ id }) => id),
    codeThemeIds: demo.CODE_THEME_OPTIONS.map(({ id }) => id),
  },
  badges: {
    cssModifiers: badgeClasses,
    literalLabels: badgeLabels,
    fixtureStatuses: [...collectFieldValues(demo.INITIAL_WALLET_FIXTURES, "status")].sort(),
    fixtureTones: [...collectFieldValues(demo.INITIAL_WALLET_FIXTURES, "tone")].sort(),
  },
  gateway: {
    queryIds: contract.gatewayQueries,
    commandIds: contract.gatewayCommands,
    errorCodes: contract.gatewayErrorCodes,
    forbiddenRendererState: contract.rendererForbiddenState,
  },
  fixtures: {
    exportIds: [
      "ASSET_CATALOG",
      "ASSET_ICON_LUT",
      "DEFAULT_FRIENDLY_ASSET_KEYS",
      "INITIAL_WALLET_FIXTURES",
      "createEmptyWallet",
      "createInitialWallets",
      "createWalletPreferences",
      "createWalletProfile",
    ],
    walletCount: demo.INITIAL_WALLET_FIXTURES.length,
    assetCount: Object.keys(demo.ASSET_CATALOG).length,
    ids: fixtureIds,
  },
  smoke: {
    assertionCount: smokeAssertions.length,
    assertions: smokeAssertions,
  },
});

await writeJson("z00z-default-tokens.json", {
  schemaVersion: 1,
  capturedOn: evidenceDate,
  source: relative(repoRoot, colorsPath),
  sourceSha256: await hashFile(colorsPath),
  paletteId: "z00z-default",
  colorScheme: "dark",
  tokens: tokenMap(colorsSource, "lut-z00z-dark-"),
});

await writeJson("z00z-corporate-source.json", {
  schemaVersion: 1,
  capturedOn: evidenceDate,
  sourceUrl: "https://z00z.io/",
  role: "design provenance only",
  runtimeDependency: false,
  paletteId: "z00z-corporate",
  colorScheme: "light",
  sourceTokens: {
    base100: "#FFFFFF",
    base200: "#E8E8E8",
    base300: "#D1D1D1",
    baseContent: "#181A2A",
    primary: "#0082C4",
    secondary: "#61738D",
    accent: "#009588",
    success: "#00A242",
    warning: "#F7C800",
    error: "#FF6266",
  },
  sourceScreenshots: [
    "z00z-corporate-desktop.png",
    "z00z-corporate-mobile.png",
  ],
});

await writeJson("baseline-manifest.json", await screenshotManifest());

console.log(`Phase 0 evidence captured in ${evidenceRoot}`);
console.log(`Screenshot baseline: ${basename(screenshotRoot)} (${(await readdir(screenshotRoot)).length} files)`);
