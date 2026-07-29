import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import { readFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFile(resolve(demoRoot, path), "utf8");
const context = vm.createContext({ URLSearchParams, structuredClone, window: {} });

for (const modulePath of [
  "scripts/port/contracts.js",
  "scripts/port/icon-sprite.js",
  "scripts/port/ui-primitives.js",
  "scripts/port/icon-registry.js",
  "scripts/port/locale-registry.js"
]) {
  vm.runInContext(await read(modulePath), context, { filename: modulePath });
}

const demo = context.window.Z00ZDemo;
const localeRegistry = context.window.Z00ZLocaleRegistry;
const index = await read("index.html");
const helpPage = await read("help.html");
const scriptSources = [...index.matchAll(/<script\s+src="([^"]+)"/g)].map((match) => match[1]);
const expectedScripts = [
  "scripts/port/locale-registry.js",
  "i18n.js",
  ...localeRegistry.map(({ catalogue }) => catalogue),
  "locales/send-exchange.js",
  "locales/navigation.js",
  "locales/demo-plan-2.js",
  "scripts/generated/help-catalog.js",
  "scripts/port/help-registry.js",
  "scripts/help-controller.js",
  "scripts/port/contracts.js",
  "scripts/port/icon-sprite.js",
  "scripts/port/ui-primitives.js",
  "scripts/port/icon-registry.js",
  "scripts/port/navigation-model.js",
  "scripts/port/navigation-session.js",
  "scripts/port/exchange-catalog.js",
  "scripts/port/dapp-catalog.js",
  "scripts/port/messenger-catalog.js",
  "scripts/port/contacts-catalog.js",
  "scripts/port/fixtures.js",
  "scripts/port/presentation-state.js",
  "scripts/port/mock-wallet-gateway.js",
  "scripts/port/mock-telemetry-gateway.js",
  "scripts/port/mock-dapp-gateway.js",
  "scripts/port/mock-messenger-gateway.js",
  "scripts/port/mock-contacts-gateway.js",
  "app.js"
];
assert.deepEqual(scriptSources, expectedScripts, "index.html script order must follow the canonical registries and bootstrap contract");
assert.deepEqual(
  [...helpPage.matchAll(/<script\s+src="([^"]+)"/g)].map((match) => match[1]),
  [
    "scripts/port/locale-registry.js",
    "i18n.js",
    ...localeRegistry.map(({ catalogue }) => catalogue),
    "locales/navigation.js",
    "scripts/port/contracts.js",
    "scripts/port/icon-sprite.js",
    "scripts/port/ui-primitives.js",
    "scripts/port/navigation-model.js",
    "scripts/port/navigation-session.js",
    "scripts/generated/help-catalog.js",
    "scripts/port/help-registry.js",
    "scripts/vendor/markdown/mermaid.min.js",
    "scripts/vendor/markdown/panzoom.min.js",
    "scripts/help-markdown-enhancer.js",
    "scripts/help-app.js"
  ],
  "help.html script order must follow the canonical locale and Help registries"
);
assert.doesNotMatch(helpPage, /help-wallet-link|help-wallet-label/u, "Help must not expose a redundant wallet placeholder");
assert.equal(/<(?:script|link)\b[^>]*(?:src|href)="https?:\/\//i.test(index), false, "runtime scripts and styles must be local");
assert.equal(/<(?:script|link)\b[^>]*(?:src|href)="https?:\/\//i.test(helpPage), false, "Help runtime scripts and styles must be local");
const staticResourceUrls = [index, helpPage].flatMap((source) => (
  [...source.matchAll(/<(?:script|link|img|source|video|audio)\b[^>]*\b(?:src|href)="([^"]+)"/gi)].map((match) => match[1])
));
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

assert.doesNotMatch(index, /<symbol\b/, "index.html must not duplicate the canonical icon sprite");
assert.doesNotMatch(helpPage, /<symbol\b/, "help.html must not duplicate the canonical icon sprite");
const symbolBlocks = [...demo.ICON_SPRITE_MARKUP.matchAll(/<symbol\s+id="i-([^"]+)"\s+viewBox="([^"]+)"[^>]*>([\s\S]*?)<\/symbol>/g)];
const symbolNames = symbolBlocks.map((match) => match[1]);
assert.deepEqual(symbolNames, Array.from(demo.ICON_NAMES), "icon names must derive from the canonical SVG sprite");
for (const [, name, viewBox] of symbolBlocks) {
  assert.equal(viewBox, "0 0 24 24", `icon ${name} must use the normalized viewBox`);
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
  "assets/z00z-friendly/Coins/curve-usd-logo-z00z.svg",
  "assets/z00z-friendly/Coins/ethereum-eth-logo-z00z.svg"
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
const curveSource = await read("assets/z00z-friendly/Coins/curve-usd-logo.svg");
const curveWrapped = await read("assets/z00z-friendly/Coins/curve-usd-logo-z00z.svg");
const ethereumWrapped = await read("assets/z00z-friendly/Coins/ethereum-eth-logo-z00z.svg");
const embeddedCurveSource = curveWrapped.match(/href="data:image\/svg\+xml;base64,([^"]+)"/);
assert.ok(embeddedCurveSource, "Curve USD Z00Z variant must embed the original local SVG");
assert.equal(
  Buffer.from(embeddedCurveSource[1], "base64").toString("utf8"),
  curveSource,
  "Curve USD Z00Z variant must preserve the original SVG byte-for-byte"
);
assert.match(
  curveWrapped,
  /<image x="210"\s+y="210"\s+width="580"\s+height="580"/,
  "Curve USD source must be centered and reduced enough to fit completely inside the gold ring"
);
const normalizedRing = (source) => (
  source.match(/<g id="z00z-cross-chain-ring"[\s\S]*?<\/g>/)?.[0].replace(/\s+/g, " ").trim()
);
assert.equal(
  normalizedRing(curveWrapped),
  normalizedRing(ethereumWrapped),
  "Curve USD and Ethereum Z00Z variants must use the same gold ring"
);

const runtimeFiles = [
  "app.js",
  "i18n.js",
  "scripts/port/contracts.js",
  "scripts/port/icon-sprite.js",
  "scripts/port/ui-primitives.js",
  "scripts/port/navigation-model.js",
  "scripts/port/navigation-session.js",
  "scripts/port/exchange-catalog.js",
  "scripts/port/dapp-catalog.js",
  "scripts/port/messenger-catalog.js",
  "scripts/port/contacts-catalog.js",
  "scripts/port/fixtures.js",
  "scripts/port/presentation-state.js",
  "scripts/port/mock-wallet-gateway.js",
  "scripts/port/mock-telemetry-gateway.js",
  "scripts/port/mock-dapp-gateway.js",
  "scripts/port/mock-messenger-gateway.js",
  "scripts/port/mock-contacts-gateway.js",
  "scripts/port/icon-registry.js",
  "scripts/port/locale-registry.js",
  "scripts/generated/help-catalog.js",
  "scripts/port/help-registry.js",
  "scripts/help-controller.js",
  "scripts/help-app.js",
  "scripts/help-markdown-enhancer.js",
  "locales/send-exchange.js",
  "locales/navigation.js",
  "locales/demo-plan-2.js",
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
const presentationStorageExceptions = new Map([
  ["sessionStorage", new Set(["scripts/port/navigation-session.js"])],
]);
for (const runtimeFile of runtimeFiles) {
  const source = await read(runtimeFile);
  for (const [label, pattern] of forbiddenRuntimePatterns) {
    if (presentationStorageExceptions.get(label)?.has(runtimeFile)) continue;
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
const helpStyle = await read("styles/help.css");
assert.equal(/@import\s+url\(["']?https?:\/\//i.test(helpStyle), false, "styles/help.css must not import remote CSS");
assert.equal(/url\(["']?https?:\/\//i.test(helpStyle), false, "styles/help.css must not load remote assets");
assert.equal(/#[0-9a-f]{3,8}\b|\brgba?\s*\(|\bhsla?\s*\(/i.test(helpStyle), false, "styles/help.css must consume semantic colour tokens only");
assert.deepEqual(
  [...helpPage.matchAll(/<link\s+rel="stylesheet"\s+href="([^"]+)"/g)].map((match) => match[1]),
  ["styles.css", "styles/help.css"],
  "help.html must load the shared foundation before its page-specific styles"
);
const colorSource = await read("styles/colors.css");
assert.equal(/@import\s+url\(["']?https?:\/\//i.test(colorSource), false, "styles/colors.css must not import remote CSS");
assert.equal(/url\(["']?https?:\/\//i.test(colorSource), false, "styles/colors.css must not load remote assets");
assert.ok(colorSource.includes("--lut-z00z-dark-brand"), "styles/colors.css must expose the canonical colour LUT");
assert.ok(colorSource.includes("--lut-z00z-corporate-primary"), "styles/colors.css must expose the local Corporate source mapping");
assert.deepEqual(
  [...colorSource.matchAll(/html\[data-palette="([^"]+)"\]/g)].map((match) => match[1]),
  ["z00z-corporate"],
  "colors.css must map only the Corporate palette beyond the Default root mapping"
);
assert.doesNotMatch(colorSource, /data-theme/, "colors.css must derive colour scheme from PaletteId");
assert.doesNotMatch(index, /data-theme=/, "the app shell must derive colour scheme from PaletteId");
assert.doesNotMatch(helpPage, /data-theme=/, "Help must derive colour scheme from PaletteId");

for (const fontFile of [
  "assets/fonts/geist/Geist-Variable.woff2",
  "assets/fonts/geist/GeistMono-Variable.woff2",
  "assets/fonts/geist/OFL.txt"
]) {
  const info = await stat(resolve(demoRoot, fontFile));
  assert.ok(info.size > 1000, `${fontFile} must be vendored and non-empty`);
}

const porting = await read("RUST-PORTING.md");
for (const requiredStatement of [
  "Leptos CSR/WASM",
  "not a browser product",
  "WalletGateway",
  "native Rust",
  "Windows/Linux",
  "iOS",
  "must never be imported by production"
]) {
  assert.ok(porting.includes(requiredStatement), `RUST-PORTING.md must declare: ${requiredStatement}`);
}

assert.equal(demo.PORT_CONTRACT.browserProduct, false);
assert.equal(demo.PORT_CONTRACT.walletBackendRuntime, "native-rust");
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.rendererForbiddenState),
  ["password", "seed_phrase", "private_key", "session_token", "raw_signed_package", "arbitrary_filesystem_path"]
);

console.log("Production-port readiness check passed.");
