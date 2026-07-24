import { readFile, readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { compileHelp } from "./compile-help.mjs";
import { assertHelpSynchronized, helpStructure } from "./sync-help.mjs";
import { loadHelpLocales, loadHelpSource, parseHelpMarkdown } from "./help-source.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const genericHelpCopy = /inspect or change|проверять и изменять|consultar o cambiar|consultar ou alterar|consulter ou modifier|prüfen oder zu ändern|確認または変更|확인하거나 변경|查看或更改|incelemek veya değiştirmek/i;

async function loadPortContract() {
  const source = await readFile(resolve(demoRoot, "scripts/port/contracts.js"), "utf8");
  const sandbox = { URLSearchParams };
  sandbox.globalThis = sandbox;
  vm.runInNewContext(source, sandbox, { filename: "scripts/port/contracts.js" });
  return sandbox.Z00ZDemo.PORT_CONTRACT;
}

function routedHelpStates(contract) {
  return contract.views.flatMap((view) => {
    if (view === "wallet") {
      return contract.walletSections.map((walletSection) => ({ view, walletSection }));
    }
    if (view === "wallet-settings") {
      return contract.walletSettingsSections.map((walletSettingsSection) => ({ view, walletSettingsSection }));
    }
    if (view === "settings") {
      return contract.settingsSections.map((settingsSection) => ({ view, settingsSection }));
    }
    if (view === "telemetry") {
      return contract.telemetrySources.flatMap((telemetrySource) => contract.telemetryTabs[telemetrySource].map((tab) => ({
        view,
        telemetrySource,
        [`${telemetrySource}TelemetryTab`]: tab
      })));
    }
    return [{ view }];
  });
}

function matchesState(topic, state) {
  return Object.entries(topic.match).every(([key, value]) => String(state[key] ?? "") === value);
}

function checkRouteCoverage(lut, contract) {
  const contextTopics = lut.topics.filter(({ scope }) => scope === "context");
  const routedStates = routedHelpStates(contract);

  for (const state of routedStates) {
    const matches = contextTopics.filter((topic) => matchesState(topic, state));
    if (matches.length !== 1) {
      throw new Error(`Help route coverage for ${JSON.stringify(state)} resolved ${matches.length} topics: ${matches.map(({ id }) => id).join(", ") || "none"}`);
    }
  }

  for (const topic of contextTopics) {
    if (!routedStates.some((state) => matchesState(topic, state))) {
      throw new Error(`Help topic ${topic.id} does not match any PORT_CONTRACT routed state`);
    }
  }

  const globalTopics = lut.topics.filter(({ scope }) => scope === "global");
  if (globalTopics.length !== 1 || globalTopics[0].id !== "app") {
    throw new Error("Help must expose exactly one global app topic");
  }
  return routedStates.length;
}

async function main() {
  const { lut } = await loadHelpSource(demoRoot);
  const localeIds = await loadHelpLocales(demoRoot);
  await assertHelpSynchronized(demoRoot);
  const routeCount = checkRouteCoverage(lut, await loadPortContract());
  const expectedFiles = lut.topics.map(({ file }) => `${file}.md`).sort();
  const englishStructures = Object.fromEntries(await Promise.all(lut.topics.map(async (topic) => {
    const path = resolve(demoRoot, "help", "en", `${topic.file}.md`);
    return [topic.id, JSON.stringify(helpStructure(parseHelpMarkdown(await readFile(path, "utf8"), path)))];
  })));

  for (const locale of localeIds) {
    const localeRoot = resolve(demoRoot, "help", locale);
    const actualFiles = (await readdir(localeRoot)).filter((file) => file.endsWith(".md")).sort();
    if (JSON.stringify(actualFiles) !== JSON.stringify(expectedFiles)) {
      throw new Error(`${localeRoot}: Help topic files do not match topics.yaml`);
    }
    for (const topic of lut.topics) {
      const path = resolve(localeRoot, `${topic.file}.md`);
      const document = parseHelpMarkdown(await readFile(path, "utf8"), path);
      if (document.id !== topic.id || document.scope !== topic.scope) {
        throw new Error(`${path}: metadata does not match topics.yaml`);
      }
      if (JSON.stringify(helpStructure(document)) !== englishStructures[topic.id]) {
        throw new Error(`${path}: section and block structure does not match the English source`);
      }
      if (document.sections.some(({ blocks }) => blocks.length === 0)) {
        throw new Error(`${path}: every Help section must contain content`);
      }
      if (topic.scope !== "global" && !document.sections.some(({ target }) => target === "current-view")) {
        throw new Error(`${path}: contextual and dialog Help must declare the current-view target`);
      }
      if (/\bTODO\b|\[translate\]/i.test(`${document.title} ${document.summary} ${JSON.stringify(document.sections)}`)) {
        throw new Error(`${path}: incomplete translation marker`);
      }
      if (genericHelpCopy.test(JSON.stringify(document.sections))) {
        throw new Error(`${path}: generic control-inspection boilerplate must be replaced with view-specific guidance`);
      }
    }
  }

  const generatedPath = resolve(demoRoot, "scripts/generated/help-catalog.js");
  const expected = await compileHelp(demoRoot);
  const actual = await readFile(generatedPath, "utf8");
  if (actual !== expected) throw new Error("Generated Help catalogue is stale; run node scripts/compile-help.mjs");
  console.log(`Help coverage ready: ${routeCount} routed states, ${lut.topics.length} topics × ${localeIds.length} locales`);
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
