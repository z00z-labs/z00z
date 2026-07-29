import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFile(resolve(demoRoot, path), "utf8");
const MENU_ICON_STANDARD = Object.freeze({
  viewBox: "0 0 24 24",
  renderedSize: 18,
  minimumWeight: 1.5,
  maximumWeight: 1.8
});
const HELP_OMITTED_NODE_IDS = new Set(["help", "about", "logout"]);

const context = vm.createContext({ URLSearchParams, structuredClone, window: {} });
for (const modulePath of [
  "scripts/port/contracts.js",
  "scripts/port/icon-registry.js",
  "scripts/port/navigation-model.js",
  "scripts/port/dapp-catalog.js"
]) {
  vm.runInContext(await read(modulePath), context, { filename: modulePath });
}

const demo = context.window.Z00ZDemo;
const sources = new Map([
  ["index.html", await read("index.html")],
  ["help.html", await read("help.html")]
]);
const componentsCss = await read("styles/components.css");
const baseIconRule = componentsCss.match(/\.icon\s*\{([\s\S]*?)\}/)?.[1] ?? "";
const baseIconStrokeWidth = Number(baseIconRule.match(/\bstroke-width:\s*([0-9.]+)\s*;/)?.[1]);
assert.equal(
  baseIconStrokeWidth,
  MENU_ICON_STANDARD.maximumWeight,
  "ordinary outline icons must inherit the shared 1.8 light stroke"
);

function attributes(source) {
  return Object.fromEntries(
    [...source.matchAll(/([:\w-]+)="([^"]*)"/g)].map(([, name, value]) => [name, value])
  );
}

function symbols(source) {
  return new Map(
    [...source.matchAll(/<symbol\b([^>]*\bid="i-([^"]+)"[^>]*)>([\s\S]*?)<\/symbol>/g)]
      .map(([, openingAttributes, name, body]) => [
        name,
        {
          attributes: attributes(openingAttributes),
          body,
          paths: [...body.matchAll(/<path\b[^>]*\bd="([^"]+)"/g)].map(([, path]) => path)
        }
      ])
  );
}

function scaleFor(body) {
  const matches = [...body.matchAll(/\bscale\(\s*([0-9.]+)(?:[\s,]+([0-9.]+))?\s*\)/g)];
  return matches.reduce((scale, [, x, y]) => scale * Math.min(Number(x), Number(y ?? x)), 1);
}

function explicitStrokeWidth(body) {
  const value = body.match(/\bstroke-width="([0-9.]+)"/)?.[1];
  return value === undefined ? null : Number(value);
}

function normalizedDefinition(symbol) {
  const normalizedAttributes = Object.entries(symbol.attributes)
    .filter(([name]) => name !== "id")
    .sort(([left], [right]) => left.localeCompare(right));
  return JSON.stringify({
    attributes: normalizedAttributes,
    body: symbol.body.replace(/\s+/g, " ").trim()
  });
}

const voucherListSource = await read("assets/z00z-friendly/Vauchers/vaucher-orange.svg");
const permissionListSource = await read("assets/z00z-friendly/Permissions/permission-blue.svg");
const voucherListPath = voucherListSource.match(/<path[^>]+d="([^"]+)"/)?.[1];
const permissionListPaths = [...permissionListSource.matchAll(/<path[^>]+d="([^"]+)"/g)]
  .map(([, path]) => path);

const ADAPTED_ICON_CONTRACTS = Object.freeze({
  earn: Object.freeze({
    source: "material-symbols-light:money-bag-outline",
    mode: "outline",
    weight: 1.5,
    paths: [
      "M8.6 20h6.8a4.6 4.6 0 0 0 3.5-7.6L15.4 8H8.6l-3.5 4.4A4.6 4.6 0 0 0 8.6 20Z",
      "M9.5 8 7.6 4h8.8l-1.9 4M9.5 12h5M12 10.5v5"
    ]
  }),
  "dapp-request": Object.freeze({
    source: "mdi-light:pin",
    mode: "normalized-fill",
    baseWeight: 1,
    weight: 1.5,
    paths: [
      "M14 12.41V5h1V4H8v1h1v7.41l-2 2V15h9v-.59zM17 14v2h-5v4.5l-.5 1.5l-.5-1.5V16H6v-2l2-2V6H7V3h9v3h-1v6z"
    ]
  }),
  "voucher-list": Object.freeze({
    source: "wallet-assets:voucher-list",
    mode: "source-fill",
    weight: 1.5,
    paths: [voucherListPath]
  }),
  "permission-list": Object.freeze({
    source: "wallet-assets:permission-list",
    mode: "outline",
    weight: 1.8,
    paths: permissionListPaths
  }),
  "dapp-private-contract": Object.freeze({
    source: "et:document",
    mode: "source-fill",
    weight: 1.5,
    paths: [
      "M1.5 32h21c.827 0 1.5-.673 1.5-1.5v-21c0-.017-.008-.031-.009-.047q-.004-.033-.013-.065a.5.5 0 0 0-.09-.191c-.007-.009-.006-.02-.013-.029l-8-9-.01-.006a.5.5 0 0 0-.223-.134q-.027-.008-.056-.011C15.557.012 15.53 0 15.5 0h-14C.673 0 0 .673 0 1.5v29c0 .827.673 1.5 1.5 1.5M16 1.815L22.387 9H16.5c-.22 0-.5-.42-.5-.75zM1 1.5a.5.5 0 0 1 .5-.5H15v7.25c0 .809.655 1.75 1.5 1.75H23v20.5a.5.5 0 0 1-.5.5h-21c-.28 0-.5-.22-.5-.5z",
      "M5.5 14h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1m0 4h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1m0-8h6a.5.5 0 0 0 0-1h-6a.5.5 0 0 0 0 1m0 12h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1m0 4h13a.5.5 0 0 0 0-1h-13a.5.5 0 0 0 0 1"
    ]
  }),
  "dapp-assets-locker": Object.freeze({
    source: "material-symbols-light:lock-outline",
    mode: "normalized-fill",
    baseWeight: 1,
    weight: 1.5,
    paths: [
      "M6.616 21q-.672 0-1.144-.472T5 19.385v-8.77q0-.67.472-1.143Q5.944 9 6.616 9H8V7q0-1.671 1.165-2.835Q10.329 3 12 3t2.836 1.165T16 7v2h1.385q.67 0 1.143.472q.472.472.472 1.144v8.769q0 .67-.472 1.143q-.472.472-1.143.472zm0-1h10.769q.269 0 .442-.173t.173-.442v-8.77q0-.269-.173-.442T17.385 10H6.615q-.269 0-.442.173T6 10.616v8.769q0 .269.173.442t.443.173m6.45-3.934q.434-.433.434-1.066t-.434-1.066T12 13.5t-1.066.434Q10.5 14.367 10.5 15t.434 1.066q.433.434 1.066.434t1.066-.434M9 9h6V7q0-1.25-.875-2.125T12 4t-2.125.875T9 7zM6 20V10z"
    ]
  }),
  "dapp-xchain-integration": Object.freeze({
    source: "mdi-light:link-variant",
    mode: "normalized-fill",
    baseWeight: 1,
    weight: 1.5,
    paths: [
      "M10.73 14.97c.27.11.36.41.24.66s-.41.37-.66.24h-.01c-.46-.21-.89-.51-1.27-.9a4.49 4.49 0 0 1 0-6.36l3.53-3.53a4.49 4.49 0 0 1 6.36 0a4.49 4.49 0 0 1 0 6.36l-1.63 1.63l-.15-1.26l1.08-1.08a3.513 3.513 0 0 0 0-4.95a3.513 3.513 0 0 0-4.95 0L9.73 9.32a3.513 3.513 0 0 0 0 4.95c.3.3.64.53 1 .7m-6.65 4.95a4.49 4.49 0 0 1 0-6.36l1.63-1.63l.15 1.26l-1.08 1.08a3.513 3.513 0 0 0 0 4.95a3.513 3.513 0 0 0 4.95 0l3.54-3.54a3.513 3.513 0 0 0 0-4.95c-.3-.3-.64-.53-1-.7v.01a.49.49 0 0 1-.24-.67c.12-.25.41-.37.66-.24h.01c.46.21.89.51 1.27.9a4.49 4.49 0 0 1 0 6.36l-3.53 3.53a4.49 4.49 0 0 1-6.36 0"
    ]
  }),
  import: Object.freeze({
    source: "system-uicons:import",
    mode: "outline",
    weight: 1.8,
    paths: [
      "M9.5 3.5h-4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-10",
      "m13.5 10.5-3 3-3-3",
      "M17.5 3.5h-4a3 3 0 0 0-3 3v7"
    ]
  }),
  "merge-split": Object.freeze({
    source: "mdi-light:sitemap",
    mode: "source-fill",
    weight: 1.5,
    paths: [
      "M9 3h5v5h-2v4h5a3 3 0 0 1 3 3v2h2v5h-5v-5h2v-2a2 2 0 0 0-2-2h-5v4h2v5H9v-5h2v-4H6a2 2 0 0 0-2 2v2h2v5H1v-5h2v-2a3 3 0 0 1 3-3h5V8H9zm4 4V4h-3v3zM5 21v-3H2v3zm8 0v-3h-3v3zm8 0v-3h-3v3z"
    ]
  })
});

const spriteByFile = new Map(
  [...sources].map(([file, source]) => [file, symbols(source)])
);
const demoMenuIcons = new Set(
  demo.NAVIGATION_NODES
    .filter(({ isVisible }) => isVisible)
    .map(({ iconId }) => iconId)
);
const helpMenuIcons = new Set(
  demo.NAVIGATION_NODES
    .filter(({ id, isVisible }) => isVisible && !HELP_OMITTED_NODE_IDS.has(id))
    .map(({ iconId }) => iconId)
);

for (const [file, menuIcons] of [
  ["index.html", demoMenuIcons],
  ["help.html", helpMenuIcons]
]) {
  const sprite = spriteByFile.get(file);
  for (const iconName of menuIcons) {
    const symbol = sprite.get(iconName);
    assert.ok(symbol, `${file} must define the menu icon i-${iconName}`);
    assert.equal(
      symbol.attributes.viewBox,
      MENU_ICON_STANDARD.viewBox,
      `${file} i-${iconName} must use the shared 24px canvas`
    );
    assert.doesNotMatch(symbol.body, /\bfilter=/i, `${file} i-${iconName} must not use a rendering filter`);
    assert.doesNotMatch(
      symbol.body,
      /(?:fill|stroke)="(?:#|rgb|hsl)/i,
      `${file} i-${iconName} must inherit the neutral menu color`
    );
    const mode = symbol.attributes["data-menu-icon-mode"];
    const isFilled = /(?:class="icon-fill"|fill="currentColor")/.test(symbol.body);
    if (!isFilled && mode !== "normalized-fill" && mode !== "source-fill") {
      const declaredStroke = explicitStrokeWidth(symbol.body);
      const renderedWeight = declaredStroke === null
        ? baseIconStrokeWidth
        : declaredStroke * scaleFor(symbol.body);
      assert.ok(
        renderedWeight >= MENU_ICON_STANDARD.minimumWeight
          && renderedWeight <= MENU_ICON_STANDARD.maximumWeight,
        `${file} i-${iconName} effective stroke ${renderedWeight} must stay inside the shared light range`
      );
    }
  }
}

for (const iconName of helpMenuIcons) {
  assert.equal(
    normalizedDefinition(spriteByFile.get("help.html").get(iconName)),
    normalizedDefinition(spriteByFile.get("index.html").get(iconName)),
    `Demo and Help must use the same i-${iconName} geometry and adaptation contract`
  );
}

for (const [file, sprite] of spriteByFile) {
  const adaptedNames = [...sprite]
    .filter(([, symbol]) => symbol.attributes["data-menu-icon-mode"])
    .map(([name]) => name)
    .filter((name) => (file === "index.html" ? demoMenuIcons : helpMenuIcons).has(name));
  assert.deepEqual(
    adaptedNames.sort(),
    Object.keys(ADAPTED_ICON_CONTRACTS).sort(),
    `${file} changed menu icons must be registered in the unified adaptation contract`
  );

  for (const [iconName, contract] of Object.entries(ADAPTED_ICON_CONTRACTS)) {
    const symbol = sprite.get(iconName);
    assert.ok(symbol, `${file} must define adapted menu icon i-${iconName}`);
    assert.equal(
      symbol.attributes["data-iconify"] ?? symbol.attributes["data-menu-icon-source"],
      contract.source,
      `${file} i-${iconName} must identify its canonical source`
    );
    assert.equal(symbol.attributes["data-menu-icon-mode"], contract.mode);
    assert.equal(Number(symbol.attributes["data-menu-icon-weight"]), contract.weight);
    assert.ok(
      contract.weight >= MENU_ICON_STANDARD.minimumWeight
        && contract.weight <= MENU_ICON_STANDARD.maximumWeight,
      `${file} i-${iconName} weight must remain inside the shared light range`
    );
    assert.deepEqual(symbol.paths, contract.paths, `${file} i-${iconName} source geometry changed`);

    if (contract.mode === "outline") {
      const renderedWeight = explicitStrokeWidth(symbol.body) * scaleFor(symbol.body);
      assert.ok(Number.isFinite(renderedWeight), `${file} i-${iconName} must declare an outline stroke`);
      assert.ok(
        Math.abs(renderedWeight - contract.weight) < 0.0001,
        `${file} i-${iconName} effective stroke ${renderedWeight} must equal ${contract.weight}`
      );
      assert.match(symbol.body, /\bstroke="currentColor"/, `${file} i-${iconName} must inherit menu color`);
    }

    if (contract.mode === "normalized-fill") {
      const baseWeight = Number(symbol.attributes["data-menu-icon-base-weight"]);
      const addedStroke = explicitStrokeWidth(symbol.body);
      assert.equal(baseWeight, contract.baseWeight);
      assert.ok(
        Math.abs(baseWeight + addedStroke - contract.weight) < 0.0001,
        `${file} i-${iconName} base and normalization stroke must equal ${contract.weight}`
      );
      assert.match(symbol.body, /\bfill="currentColor"/);
      assert.match(symbol.body, /\bstroke="currentColor"/);
    }

    if (contract.mode === "source-fill") {
      assert.match(
        symbol.body,
        /(?:class="icon-fill"|fill="currentColor")/,
        `${file} i-${iconName} must use the shared currentColor fill path`
      );
    }
  }
}

for (const { iconName } of demo.DAPP_CATALOG) {
  if (ADAPTED_ICON_CONTRACTS[iconName]) continue;
  for (const [file, sprite] of spriteByFile) {
    const symbol = sprite.get(iconName);
    assert.ok(symbol, `${file} must define dApp icon i-${iconName}`);
    assert.equal(
      explicitStrokeWidth(symbol.body) * scaleFor(symbol.body),
      MENU_ICON_STANDARD.minimumWeight,
      `${file} i-${iconName} must use the standard light outline weight`
    );
    assert.doesNotMatch(symbol.body, /\bclass="icon-fill"/);
  }
}

const navigationIconRule = componentsCss.match(/\.navigation-tree-icon\s*\{([\s\S]*?)\}/)?.[1] ?? "";
assert.match(
  navigationIconRule,
  new RegExp(`\\bwidth:\\s*${MENU_ICON_STANDARD.renderedSize}px\\s*;`),
  "menu icons must use the shared rendered width"
);
assert.match(
  navigationIconRule,
  new RegExp(`\\bheight:\\s*${MENU_ICON_STANDARD.renderedSize}px\\s*;`),
  "menu icons must use the shared rendered height"
);

console.log(
  `Menu icon contract check passed: ${demoMenuIcons.size} Demo icons, `
  + `${helpMenuIcons.size} Help icons, ${Object.keys(ADAPTED_ICON_CONTRACTS).length} source-adapted icons, `
  + `${MENU_ICON_STANDARD.renderedSize}px at ${MENU_ICON_STANDARD.minimumWeight}-${MENU_ICON_STANDARD.maximumWeight} light weight.`
);
