import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFile(resolve(demoRoot, path), "utf8");
const context = vm.createContext({ URLSearchParams, structuredClone, window: {} });

for (const modulePath of [
  "scripts/port/contracts.js",
  "scripts/port/icon-registry.js",
  "scripts/port/locale-registry.js"
]) {
  vm.runInContext(await read(modulePath), context, { filename: modulePath });
}

const demo = context.window.Z00ZDemo;
const localeRegistry = context.window.Z00ZLocaleRegistry;
const index = await read("index.html");
const scriptSources = [...index.matchAll(/<script\s+src="([^"]+)"/g)].map((match) => match[1]);
const expectedScripts = [
  "scripts/port/locale-registry.js",
  "i18n.js",
  ...localeRegistry.map(({ catalogue }) => catalogue),
  "scripts/port/contracts.js",
  "scripts/port/fixtures.js",
  "scripts/port/presentation-state.js",
  "scripts/port/mock-wallet-gateway.js",
  "scripts/port/icon-registry.js",
  "app.js"
];
assert.deepEqual(scriptSources, expectedScripts, "index.html script order must follow the canonical registries and bootstrap contract");
assert.equal(/<(?:script|link)\b[^>]*(?:src|href)="https?:\/\//i.test(index), false, "runtime scripts and styles must be local");
const staticResourceUrls = [...index.matchAll(/<(?:script|link|img|source|video|audio)\b[^>]*\b(?:src|href)="([^"]+)"/gi)].map((match) => match[1]);
for (const resourceUrl of staticResourceUrls) {
  assert.equal(/^(?:https?:)?\/\//i.test(resourceUrl), false, `static resource ${resourceUrl} must be bundled locally`);
}

assert.match(index, /<link rel="manifest" href="manifest\.webmanifest\?v=2">/, "index.html must expose the versioned local app manifest");
assert.match(index, /<link rel="apple-touch-icon" sizes="180x180" href="assets\/logo\/z00z-apple-touch-icon-v2-180\.png">/, "index.html must expose the local Apple touch icon");
assert.match(index, /<link rel="icon" type="image\/png" href="assets\/logo\/z00z-logo-gold-circle\.png\?v=2">/, "index.html must use the canonical PNG app brand as its favicon");
assert.equal(
  [...index.matchAll(/<img class="brand-mark" src="([^"]+)"/g)].every((match) => match[1] === "assets/logo/z00z-logo-gold-circle.png"),
  true,
  "every visible app brand must use the canonical PNG source"
);
const appManifest = JSON.parse(await read("manifest.webmanifest"));
assert.equal(appManifest.start_url, "./");
assert.equal(appManifest.scope, "./");
assert.equal(appManifest.display, "standalone");
assert.deepEqual(
  appManifest.icons.map(({ src, sizes, type, purpose }) => ({ src, sizes, type, purpose })),
  [
    { src: "assets/logo/z00z-app-icon-v2-192.png", sizes: "192x192", type: "image/png", purpose: "any" },
    { src: "assets/logo/z00z-app-icon-v2-512.png", sizes: "512x512", type: "image/png", purpose: "any" },
    { src: "assets/logo/z00z-app-icon-v2-maskable-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" }
  ]
);
for (const appIcon of [
  "assets/logo/z00z-logo-gold-circle.png",
  "assets/logo/z00z-app-icon-v2-192.png",
  "assets/logo/z00z-app-icon-v2-512.png",
  "assets/logo/z00z-app-icon-v2-maskable-512.png",
  "assets/logo/z00z-apple-touch-icon-v2-180.png"
]) {
  const iconInfo = await stat(resolve(demoRoot, appIcon));
  assert.ok(iconInfo.size > 0, `${appIcon} must exist and be non-empty`);
}

const symbolBlocks = [...index.matchAll(/<symbol\s+id="i-([^"]+)"\s+viewBox="([^"]+)"[^>]*>([\s\S]*?)<\/symbol>/g)];
const symbolNames = symbolBlocks.map((match) => match[1]);
assert.deepEqual(symbolNames, Array.from(demo.ICON_NAMES), "inline SVG symbols must match the canonical icon registry order");
for (const [, name, viewBox, body] of symbolBlocks) {
  assert.equal(viewBox, "0 0 24 24", `icon ${name} must use the normalized viewBox`);
  assert.equal(/fill="currentColor"/i.test(body), false, `icon ${name} must use the shared outline stroke contract`);
}

for (const family of Object.values(demo.OBJECT_TYPE_ICON_LUT)) {
  for (const definition of Object.values(family)) {
    if (definition.iconName) {
      assert.ok(symbolNames.includes(definition.iconName), `object icon ${definition.iconName} must exist in the sprite`);
      continue;
    }
    assert.equal(definition.mode, "image", `object icon ${definition.iconSrc} must declare image mode`);
    const iconInfo = await stat(resolve(demoRoot, definition.iconSrc));
    assert.ok(iconInfo.size > 0, `${definition.iconSrc} must exist and be non-empty`);
  }
}
for (const definition of Object.values(demo.OBJECT_FAMILY_ICON_LUT)) {
  assert.ok(["image", "mask"].includes(definition.mode), `object family icon ${definition.iconSrc} must declare a supported mode`);
  const iconInfo = await stat(resolve(demoRoot, definition.iconSrc));
  assert.ok(iconInfo.size > 0, `${definition.iconSrc} must exist and be non-empty`);
}
for (const lightBackgroundCoinIcon of [
  "assets/z00z-friendly/Coins/algorand-algo-logo-z00z.svg",
  "assets/z00z-friendly/Coins/cardano-ada-logo-z00z.svg",
  "assets/z00z-friendly/Coins/ethereum-eth-logo-z00z.svg",
  "assets/z00z-friendly/Coins/hyperliquid-hype-logo-z00z.svg"
]) {
  const iconBody = await read(lightBackgroundCoinIcon);
  assert.match(
    iconBody,
    /<circle id="coin-background" cx="500" cy="500" r="439" fill="#FFFFFF"\/>/,
    `${lightBackgroundCoinIcon} must provide an opaque white background inside the gold ring`
  );
  assert.doesNotMatch(
    iconBody,
    /<rect width="1000" height="1000" fill="#FFFFFF"\/>/,
    `${lightBackgroundCoinIcon} must remain transparent outside the gold ring`
  );
}

const runtimeFiles = [
  "app.js",
  "i18n.js",
  "scripts/port/contracts.js",
  "scripts/port/fixtures.js",
  "scripts/port/presentation-state.js",
  "scripts/port/mock-wallet-gateway.js",
  "scripts/port/icon-registry.js",
  "scripts/port/locale-registry.js",
  ...localeRegistry.map(({ catalogue }) => catalogue)
];
const forbiddenRuntimePatterns = [
  ["fetch", /\bfetch\s*\(/],
  ["XMLHttpRequest", /\bXMLHttpRequest\b/],
  ["WebSocket", /\b(?:new\s+)?WebSocket\s*\(/],
  ["EventSource", /\b(?:new\s+)?EventSource\s*\(/],
  ["sendBeacon", /\b(?:navigator\s*\.\s*)?sendBeacon\s*\(/],
  ["localStorage", /\blocalStorage\b/],
  ["sessionStorage", /\bsessionStorage\b/],
  ["IndexedDB", /\bindexedDB\b/],
  ["service worker", /\bserviceWorker\b/],
  ["generic RPC dispatcher", /\brpc\s*\.\s*call\s*\(/i]
];
for (const runtimeFile of runtimeFiles) {
  const source = await read(runtimeFile);
  for (const [label, pattern] of forbiddenRuntimePatterns) {
    assert.equal(pattern.test(source), false, `${runtimeFile} must not use ${label}`);
  }
}

const styleEntry = await read("styles.css");
assert.equal(styleEntry.trim(), '@import url("styles/colors.css");\n@import url("styles/foundation.css");\n@import url("styles/components.css");');
assert.deepEqual(
  [...index.matchAll(/<link\s+rel="stylesheet"\s+href="([^"]+)"/g)].map((match) => match[1]),
  ["styles.css"],
  "index.html must load the stable CSS entry point only"
);
for (const styleFile of ["styles.css", "styles/foundation.css", "styles/components.css"]) {
  const source = await read(styleFile);
  assert.equal(/@import\s+url\(["']?https?:\/\//i.test(source), false, `${styleFile} must not import remote CSS`);
  assert.equal(/url\(["']?https?:\/\//i.test(source), false, `${styleFile} must not load remote assets`);
  assert.equal(/#[0-9a-f]{3,8}\b|\brgba?\s*\(|\bhsla?\s*\(/i.test(source), false, `${styleFile} must consume semantic colour tokens only`);
}
const colorSource = await read("styles/colors.css");
assert.equal(/@import\s+url\(["']?https?:\/\//i.test(colorSource), false, "styles/colors.css must not import remote CSS");
assert.equal(/url\(["']?https?:\/\//i.test(colorSource), false, "styles/colors.css must not load remote assets");
assert.ok(colorSource.includes("--lut-z00z-dark-brand"), "styles/colors.css must expose the canonical colour LUT");

for (const fontFile of [
  "assets/fonts/geist/Geist-Variable.woff2",
  "assets/fonts/geist/GeistMono-Variable.woff2",
  "assets/fonts/geist/OFL.txt"
]) {
  const info = await stat(resolve(demoRoot, fontFile));
  assert.ok(info.size > 1000, `${fontFile} must be vendored and non-empty`);
}

const porting = await read("PORTING.md");
for (const requiredStatement of [
  "Leptos CSR/WASM",
  "not a browser product",
  "WalletGateway",
  "native Rust",
  "Windows/Linux",
  "iOS",
  "must never be imported by production"
]) {
  assert.ok(porting.includes(requiredStatement), `PORTING.md must declare: ${requiredStatement}`);
}

assert.equal(demo.PORT_CONTRACT.browserProduct, false);
assert.equal(demo.PORT_CONTRACT.walletBackendRuntime, "native-rust");
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.rendererForbiddenState),
  ["password", "seed_phrase", "private_key", "session_token", "raw_signed_package", "arbitrary_filesystem_path"]
);

console.log("Production-port readiness check passed.");
