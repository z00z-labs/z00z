import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(demoRoot, "../../..");
const reviewRoot = resolve(
  process.argv[2]
    || resolve(repoRoot, "crates/z00z_storage/outputs/checkpoint/phase-110/ui-help-review"),
);
const readJson = async (path) => JSON.parse(await readFile(path, "utf8"));

const baselineRoot = resolve(demoRoot, "evidence/phase-0");
const baseline = await readJson(resolve(baselineRoot, "baseline-manifest.json"));
const defaultSnapshot = await readJson(resolve(baselineRoot, "z00z-default-tokens.json"));
const colorSource = await readFile(resolve(demoRoot, "styles/colors.css"), "utf8");
const escapeRegExp = (value) => value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

assert.deepEqual(
  baseline.failedRouteViewportPairs.map(({ viewport, route, issueTypes }) => ({
    viewport,
    route,
    issueTypes,
  })),
  [{
    viewport: "desktop-1024",
    route: "wallet-assets",
    issueTypes: ["element-outside-viewport", "viewport-overflow"],
  }],
  "Phase 0 debt record must remain explicit",
);

for (const [token, value] of Object.entries(defaultSnapshot.tokens)) {
  const sourceName = `--lut-z00z-dark-${token}`;
  assert.match(
    colorSource,
    new RegExp(`${escapeRegExp(sourceName)}:\\s*${escapeRegExp(value)};`, "i"),
    `Z00Z Default token ${token} drifted from the frozen baseline`,
  );
}

const responsiveAudit = await readJson(resolve(reviewRoot, "responsive-layout-audit.json"));
const phase6Audit = await readJson(resolve(reviewRoot, "phase-6-responsive-layout-audit.json"));
const phase7Audit = await readJson(resolve(reviewRoot, "phase-7-help-responsive-audit.json"));
for (const [name, audit] of [
  ["responsive", responsiveAudit],
  ["Messenger/Contacts", phase6Audit],
  ["Help", phase7Audit],
]) {
  assert.deepEqual(
    audit.filter(({ issues }) => issues.length > 0),
    [],
    `${name} visual audit contains a regression`,
  );
}

const contractSource = await readFile(resolve(demoRoot, "scripts/port/contracts.js"), "utf8");
const context = vm.createContext({ URLSearchParams, window: {} });
vm.runInContext(contractSource, context, { filename: "scripts/port/contracts.js" });
const routes = context.window.Z00ZDemo.PORT_CONTRACT.routes;
const viewports = ["desktop-1280", "desktop-1024", "tablet-768", "mobile-390", "mobile-320"];
const palettes = ["default", "corporate"];
const expectedScreenshots = [];
for (const viewport of viewports) {
  for (const palette of palettes) {
    for (const routeId of routes) {
      expectedScreenshots.push(
        `${viewport}-${palette}-${routeId.replaceAll(".", "-")}.png`,
      );
    }
  }
}
for (const screenshot of expectedScreenshots) {
  await access(resolve(reviewRoot, screenshot));
}
for (const requiredState of [
  "desktop-1280-default-wallet-assets-zoom-200.png",
  "mobile-320-default-wallet-assets-zoom-200.png",
  "desktop-1280-default-watchers-reduced-motion.png",
  "mobile-320-default-watchers-reduced-motion.png",
  "mobile-320-wallet-telemetry-multi-open-lower-tree.png",
]) {
  await access(resolve(reviewRoot, requiredState));
}

const repairedAssetAudit = responsiveAudit.find(({ viewport, route }) =>
  viewport === "desktop-1024" && route === "default-wallet-assets"
);
assert.ok(repairedAssetAudit, "Current audit must include the frozen 1024px Assets debt");
assert.deepEqual(repairedAssetAudit.issues, [], "The frozen 1024px Assets overflow regressed");

console.log(
  `Phase 9 regression gate passed: ${routes.length} routes × ${viewports.length} viewports × ${palettes.length} palettes; Default tokens unchanged`,
);
