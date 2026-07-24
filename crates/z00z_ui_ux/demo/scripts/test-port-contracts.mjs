import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const context = vm.createContext({
  URLSearchParams,
  structuredClone,
  window: {}
});
const modules = [
  "scripts/port/contracts.js",
  "scripts/port/exchange-catalog.js",
  "scripts/port/fixtures.js",
  "scripts/port/presentation-state.js",
  "scripts/port/mock-wallet-gateway.js",
  "scripts/port/icon-registry.js",
  "scripts/port/locale-registry.js"
];

for (const modulePath of modules) {
  const source = await readFile(resolve(demoRoot, modulePath), "utf8");
  vm.runInContext(source, context, { filename: modulePath });
}

const demo = context.window.Z00ZDemo;
const locales = context.window.Z00ZLocaleRegistry;

assert.equal(demo.PORT_CONTRACT.rendererRuntime, "leptos-csr-wasm");
assert.equal(demo.PORT_CONTRACT.packagedHost, "tauri-2");
assert.equal(demo.PORT_CONTRACT.browserProduct, false);
assert.equal(demo.PORT_CONTRACT.walletBackendRuntime, "native-rust");
assert.ok(demo.PORT_CONTRACT.rendererForbiddenState.includes("session_token"));
assert.ok(demo.PORT_CONTRACT.forbiddenTransports.includes("websocket"));
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.walletChains),
  ["mainnet", "testnet-1", "testnet-2", "devnet-1", "devnet-2"]
);
assert.deepEqual(Object.keys(demo.EXCHANGE_PROVIDER_LUT), ["hyperliquid", "near-intents"]);
assert.equal(demo.exchangeProvider("unknown").id, "near-intents");
assert.deepEqual(Array.from(demo.exchangeDestinations("hyperliquid"), ({ id }) => id), [
  "hyperliquid-usdc",
  "hyperliquid-hype",
  "hyperliquid-btc"
]);
assert.ok(Object.isFrozen(demo.EXCHANGE_PROVIDER_LUT));

const defaults = demo.resolveInitialNavigation("?view=unknown&wallet=everything&settings=invalid");
assert.equal(defaults.view, "wallet");
assert.equal(defaults.walletSection, "assets");
assert.equal(defaults.settingsSection, "general");
const allowed = demo.resolveInitialNavigation("?view=wallet-send&wallet=permissions&walletSettings=advanced&settings=onionnet&onionTab=queues");
assert.equal(allowed.view, "wallet-send");
assert.equal(allowed.walletSection, "permissions");
assert.equal(allowed.walletSettingsSection, "advanced");
assert.equal(allowed.settingsSection, "onionnet");
assert.equal(allowed.onionnetTelemetryTab, "queues");

const firstClone = demo.createInitialWallets();
const secondClone = demo.createInitialWallets();
firstClone[0].name = "Changed only here";
firstClone[0].activities.length = 0;
assert.equal(secondClone[0].name, "Everyday");
assert.ok(secondClone[0].activities.length > 0);
assert.equal(demo.INITIAL_WALLET_FIXTURES[0].name, "Everyday");
assert.ok(demo.INITIAL_WALLET_FIXTURES.every(({ chainId }) => chainId === "mainnet"));
const deterministicProfile = demo.createWalletProfile(
  [...secondClone, { id: "wallet-4" }],
  "Field wallet",
  "testnet-2"
);
assert.equal(deterministicProfile.id, "wallet-5");
assert.equal(deterministicProfile.address, "ZxN5q7…2305Pt");
assert.equal(deterministicProfile.chainId, "testnet-2");
assert.equal(demo.createEmptyWallet().summary.scan, "Unavailable");
const friendlyAssetKeys = Object.keys(demo.ASSET_ICON_LUT);
assert.deepEqual(Array.from(demo.DEFAULT_FRIENDLY_ASSET_KEYS), friendlyAssetKeys);
assert.equal(new Set(friendlyAssetKeys).size, 16);
assert.equal(demo.ASSET_CATALOG.length, 16);
for (const iconPath of Object.values(demo.ASSET_ICON_LUT)) {
  const iconInfo = await stat(resolve(demoRoot, iconPath));
  assert.ok(iconInfo.size > 0, `${iconPath} must exist and be non-empty`);
}
for (const wallet of demo.INITIAL_WALLET_FIXTURES) {
  assert.equal(wallet.assetKeys.length, 16);
  assert.ok(friendlyAssetKeys.every((key) => wallet.assetKeys.includes(key)));
}
assert.deepEqual(Array.from(deterministicProfile.assetKeys), friendlyAssetKeys);
assert.deepEqual(Array.from(demo.createEmptyWallet().assetKeys), friendlyAssetKeys);

const state = demo.createInitialState({ search: "?view=activity", brand: "brand-token", rail: "rail-token" });
assert.equal(state.view, "activity");
assert.equal(state.wallets.length, 3);
assert.equal(demo.activeWallet(state).id, "everyday");
const preferences = demo.ensureWalletPreferences(state);
assert.equal(preferences.defaultFee, "0.001");
assert.equal(preferences.lockAfterMinutes, "15");

const gateway = demo.createMockWalletGateway(state);
assert.equal(gateway.contractVersion, demo.PORT_CONTRACT.version);
assert.equal(gateway.createProfile({ name: "x" }).error.code, "validation");
assert.equal(gateway.createProfile({ name: "Valid wallet", chainId: "unknown" }).error.code, "validation");
assert.equal(gateway.removeProfiles({ walletIds: [] }).error.code, "validation");
assert.equal(gateway.removeProfiles({ walletIds: ["missing"] }).error.code, "validation");
assert.equal(gateway.renameWallet({ walletId: "missing", name: "Valid name" }).error.code, "validation");
assert.equal(gateway.renameWallet({ walletId: "everyday", name: "x" }).error.code, "validation");
assert.equal(gateway.changePassword({ walletId: "missing", currentPassword: "old-value", newPassword: "new-value" }).error.code, "validation");
assert.equal(gateway.changePassword({ walletId: "everyday", currentPassword: "same-value", newPassword: "same-value" }).error.code, "validation");
const created = gateway.createProfile({ name: "Field wallet", chainId: "devnet-2", scan: "Scanning" });
assert.equal(created.ok, true);
assert.equal(state.wallets.at(-1).name, "Field wallet");
assert.equal(state.wallets.at(-1).chainId, "devnet-2");
const renamed = gateway.renameWallet({ walletId: created.data.wallet.id, name: "Field savings" });
assert.equal(renamed.ok, true);
assert.equal(state.wallets.at(-1).initials, "F");

const currentSecret = "current-password-value";
const newSecret = "new-password-value";
const changed = gateway.changePassword({
  walletId: created.data.wallet.id,
  currentPassword: currentSecret,
  newPassword: newSecret
});
assert.equal(changed.ok, true);
const serializedState = JSON.stringify(state);
assert.equal(serializedState.includes(currentSecret), false);
assert.equal(serializedState.includes(newSecret), false);

demo.ensureWalletPreferences(state, state.wallets.find(({ id }) => id === "savings"));
assert.ok(state.walletPreferences.savings);
const preservedSelection = gateway.removeProfiles({
  walletIds: ["savings"],
  selectedWalletId: "everyday"
});
assert.equal(preservedSelection.ok, true);
assert.equal(preservedSelection.data.selectedWalletId, "everyday");
assert.equal(state.walletPreferences.savings, undefined);

const allIds = state.wallets.map(({ id }) => id);
const removed = gateway.removeProfiles({ walletIds: allIds, selectedWalletId: state.selectedWalletId });
assert.equal(removed.ok, true);
assert.equal(state.wallets.length, 0);
assert.equal(removed.data.selectedWalletId, null);

assert.equal(locales.length, 10);
assert.deepEqual(
  Array.from(locales, ({ id }) => id),
  ["en", "ru", "fr", "de", "es", "pt", "ko", "tr", "ja", "zh-Hans"]
);
assert.equal(new Set(locales.map(({ catalogue }) => catalogue)).size, locales.length);

assert.equal(new Set(demo.ICON_NAMES).size, demo.ICON_NAMES.length);
for (const family of Object.values(demo.OBJECT_TYPE_ICON_LUT)) {
  for (const definition of Object.values(family)) {
    if (definition.iconName) {
      assert.ok(demo.ICON_NAMES.includes(definition.iconName));
      continue;
    }
    assert.equal(definition.mode, "image");
    const iconInfo = await stat(resolve(demoRoot, definition.iconSrc));
    assert.ok(iconInfo.size > 0, `${definition.iconSrc} must exist and be non-empty`);
  }
}
for (const definition of Object.values(demo.OBJECT_FAMILY_ICON_LUT)) {
  assert.ok(["image", "mask"].includes(definition.mode));
  const iconInfo = await stat(resolve(demoRoot, definition.iconSrc));
  assert.ok(iconInfo.size > 0, `${definition.iconSrc} must exist and be non-empty`);
}
assert.equal(Object.keys(demo.VOUCHER_ICON_LUT).length, 8);
assert.equal(Object.keys(demo.PERMISSION_ICON_LUT).length, 8);
for (const voucher of demo.INITIAL_WALLET_FIXTURES[0].vouchers) assert.ok(demo.VOUCHER_ICON_LUT[voucher.kind]);
for (const permission of demo.INITIAL_WALLET_FIXTURES[0].permissions) assert.ok(demo.PERMISSION_ICON_LUT[permission.kind]);

console.log("Production-port contract tests passed.");
