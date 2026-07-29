import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFile(resolve(demoRoot, path), "utf8");
const CSS_FILES = [
  "styles/colors.css",
  "styles/foundation.css",
  "styles/components.css",
  "styles/help.css"
];
const RUNTIME_FILES = [
  "app.js",
  "scripts/help-app.js",
  "scripts/help-markdown-enhancer.js",
  "scripts/port/contracts.js",
  "scripts/port/icon-sprite.js",
  "scripts/port/ui-primitives.js",
  "scripts/port/icon-registry.js",
  "scripts/port/navigation-model.js",
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
  "scripts/port/mock-contacts-gateway.js"
];

const sources = new Map();
for (const path of [...CSS_FILES, ...RUNTIME_FILES, "styles.css", "index.html", "help.html"]) {
  sources.set(path, await read(path));
}
const designSpec = await read("../../../.planning/phases/110-Wallet-UX-UI/UI-UX-SPEC.md");

assert.equal(
  sources.get("styles.css").trim(),
  '@import url("styles/colors.css");\n@import url("styles/foundation.css");\n@import url("styles/components.css");',
  "the shared CSS entry point must load color, foundation, then component LUTs"
);

const literalColorPattern = /#[0-9a-f]{3,8}\b|\brgba?\s*\(|\bhsla?\s*\(/i;
for (const path of CSS_FILES.filter((path) => path !== "styles/colors.css")) {
  assert.doesNotMatch(
    sources.get(path),
    literalColorPattern,
    `${path} must consume colors.css semantic tokens instead of literal colors`
  );
}
const runtimeLiteralColorPattern = /(?<!&)#(?:[0-9a-f]{6}|[0-9a-f]{8})\b|\brgba?\s*\(|\bhsla?\s*\(/i;
for (const path of RUNTIME_FILES) {
  assert.doesNotMatch(
    sources.get(path),
    runtimeLiteralColorPattern,
    `${path} must not create a second JavaScript color LUT`
  );
}
assert.match(
  sources.get("styles/colors.css"),
  /--mermaid-activation-bg:/,
  "Help diagram colors must live in the canonical color LUT"
);

assert.equal(
  CSS_FILES.filter((path) => /@font-face\b/.test(sources.get(path))).join(","),
  "styles/foundation.css",
  "foundation.css must be the only font-face owner"
);
for (const path of ["styles/components.css", "styles/help.css"]) {
  const declarations = [...sources.get(path).matchAll(/font-family\s*:\s*([^;]+);/g)]
    .map((match) => match[1].trim());
  for (const declaration of declarations) {
    assert.match(
      declaration,
      /^var\(--font-(?:sans|mono|logo)\)$/,
      `${path} font-family ${declaration} must use the typography LUT`
    );
  }
}
assert.doesNotMatch(
  sources.get("scripts/help-markdown-enhancer.js"),
  /Trebuchet|Verdana|Arial/,
  "Mermaid must inherit the canonical UI font"
);

const customPropertyDefinitions = new Set();
const customPropertyUses = new Map();
for (const path of CSS_FILES) {
  const source = sources.get(path);
  for (const match of source.matchAll(/--([\w-]+)\s*:/g)) {
    customPropertyDefinitions.add(match[1]);
  }
  for (const match of source.matchAll(/var\(--([\w-]+)/g)) {
    const files = customPropertyUses.get(match[1]) || new Set();
    files.add(path);
    customPropertyUses.set(match[1], files);
  }
}
const dynamicCustomProperties = new Set(["object-family-source"]);
for (const [name, files] of customPropertyUses) {
  assert.ok(
    customPropertyDefinitions.has(name) || dynamicCustomProperties.has(name),
    `undefined CSS token --${name} is used by ${[...files].join(", ")}`
  );
}
assert.match(
  sources.get("app.js"),
  /--object-family-source:url\(/,
  "the sole dynamic component variable must be authored by the object-icon renderer"
);

const context = vm.createContext({ window: {} });
for (const path of ["scripts/port/icon-sprite.js", "scripts/port/icon-registry.js"]) {
  vm.runInContext(sources.get(path), context, { filename: path });
}
const demo = context.window.Z00ZDemo;
const symbolBlocks = [
  ...demo.ICON_SPRITE_MARKUP.matchAll(
    /<symbol\b([^>]*\bid="i-([^"]+)"[^>]*)>([\s\S]*?)<\/symbol>/g
  )
];
const symbolNames = symbolBlocks.map((match) => match[2]);
assert.equal(new Set(symbolNames).size, symbolNames.length, "canonical icon IDs must be unique");
assert.deepEqual(symbolNames, Array.from(demo.ICON_NAMES), "icon names must derive from SVG geometry");
for (const [, attributes, name] of symbolBlocks) {
  assert.match(attributes, /\bviewBox="0 0 24 24"/, `i-${name} must use the shared 24px canvas`);
}
for (const path of ["index.html", "help.html"]) {
  assert.doesNotMatch(sources.get(path), /<symbol\b/, `${path} must not duplicate icon geometry`);
  assert.match(
    sources.get(path),
    /<script src="scripts\/port\/icon-sprite\.js"><\/script>/,
    `${path} must load the canonical icon sprite`
  );
  assert.match(
    sources.get(path),
    /<script src="scripts\/port\/ui-primitives\.js"><\/script>/,
    `${path} must load shared runtime UI primitives`
  );
}

const iconReferenceFiles = [
  "app.js",
  "scripts/help-app.js",
  "scripts/port/navigation-model.js",
  "scripts/port/exchange-catalog.js",
  "scripts/port/dapp-catalog.js",
  "scripts/port/icon-registry.js"
];
for (const path of iconReferenceFiles) {
  const source = sources.get(path);
  const referencedNames = [
    ...source.matchAll(/\bicon\(\s*"([^"]+)"/g),
    ...source.matchAll(/\biconId:\s*"([^"]+)"/g),
    ...source.matchAll(/\biconName:\s*"([^"]+)"/g)
  ].map((match) => match[1]);
  for (const name of referencedNames) {
    assert.ok(demo.ICON_NAMES.includes(name), `${path} references missing canonical icon i-${name}`);
  }
}

for (const path of ["app.js", "scripts/help-app.js"]) {
  const source = sources.get(path);
  assert.doesNotMatch(source, /\.matchMedia\s*\(/, `${path} must use VIEWPORT_QUERY_LUT`);
  assert.doesNotMatch(
    source,
    /(?:menu|walletPickerPopup)\.style\.(?:left|width|maxHeight|top|bottom)/,
    `${path} must use the shared floating-panel geometry primitive`
  );
  assert.doesNotMatch(
    source,
    /<div class="language-picker/,
    `${path} must use the shared language-picker structure`
  );
}
assert.doesNotMatch(
  sources.get("scripts/help-markdown-enhancer.js"),
  /\.matchMedia\s*\(/,
  "Help Markdown behavior must use shared viewport primitives"
);
assert.match(
  sources.get("scripts/port/ui-primitives.js"),
  /const VIEWPORT_QUERY_LUT = Object\.freeze/,
  "viewport breakpoints must have one runtime LUT"
);
assert.match(
  sources.get("scripts/port/ui-primitives.js"),
  /const FLOATING_PANEL_LUT = Object\.freeze/,
  "floating-panel geometry must have one runtime LUT"
);
assert.match(
  sources.get("scripts/port/ui-primitives.js"),
  /const LANGUAGE_PICKER_DOM_LUT = Object\.freeze/,
  "language-picker structure must have one runtime LUT"
);
for (const token of [
  "space-1",
  "space-2",
  "space-4",
  "space-5",
  "icon-size",
  "icon-control-size",
  "control-min-height"
]) {
  assert.match(
    sources.get("styles/components.css"),
    new RegExp(`var\\(--${token}\\)`),
    `shared components must consume foundation token --${token}`
  );
}
for (const canonicalSource of [
  "demo/styles/colors.css",
  "demo/styles/foundation.css",
  "demo/styles/components.css",
  "demo/scripts/port/icon-sprite.js",
  "demo/scripts/port/icon-registry.js",
  "demo/scripts/port/ui-primitives.js",
  "demo/scripts/port/navigation-model.js",
  "demo/scripts/port/presentation-state.js",
  "demo/scripts/port/locale-registry.js"
]) {
  assert.ok(
    designSpec.includes(canonicalSource),
    `UI-UX-SPEC.md must declare ${canonicalSource} in the single-source ownership contract`
  );
}

console.log(
  `Design-system contract passed: ${customPropertyDefinitions.size} CSS tokens, `
  + `${symbolNames.length} canonical icons, ${RUNTIME_FILES.length} runtime modules.`
);
