import { access, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { HELP_GROUP_DEFINITIONS } from "./help-topics.mjs";

const META_KEYS = new Set(["title", "icon", "order"]);
const INDEX_KEYS = new Set(["title", "description", "difficulty", "icon", "toc"]);
const INDEX_ICON = "mdi:alphabet-a-box-outline";
const SUPPLEMENTAL_SECTIONS = Object.freeze({
  privacy: Object.freeze({
    icon: "shield",
    order: Object.freeze(["index", "privacy-statement"])
  })
});
const NESTED_SECTION_ICONS = Object.freeze({
  "wallets/assets-rights": "assets",
  "wallets/staking": "staking",
  "wallets/settings": "settings",
  "telemetry/reticulum": "reticulum-node",
  "telemetry/onionnet": "shield",
  "telemetry/aggregators": "staking",
  "telemetry/watchers": "eye",
  "telemetry/explorer": "search"
});

function parseScalar(value) {
  const trimmed = value.trim();
  if (
    trimmed.length >= 2
    && ((trimmed.startsWith('"') && trimmed.endsWith('"'))
      || (trimmed.startsWith("'") && trimmed.endsWith("'")))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

function parseKeyValue(line, sourceName) {
  const separator = line.indexOf(":");
  if (separator < 1) throw new Error(`${sourceName}: malformed YAML line: ${line}`);
  return [line.slice(0, separator).trim(), parseScalar(line.slice(separator + 1))];
}

export function parseHelpFolderMeta(source, sourceName = "_meta.yaml") {
  const meta = {};
  let readingOrder = false;

  for (const rawLine of source.replaceAll("\r\n", "\n").split("\n")) {
    if (!rawLine.trim() || rawLine.trimStart().startsWith("#")) continue;
    if (/^  - /.test(rawLine) && readingOrder) {
      const item = parseScalar(rawLine.slice(4));
      if (!/^[a-z0-9][a-z0-9-]*$/.test(item)) {
        throw new Error(`${sourceName}: unsafe order entry ${item}`);
      }
      meta.order.push(item);
      continue;
    }
    if (/^\S/.test(rawLine)) {
      const [key, value] = parseKeyValue(rawLine, sourceName);
      if (!META_KEYS.has(key)) throw new Error(`${sourceName}: unknown metadata key ${key}`);
      if (Object.hasOwn(meta, key)) throw new Error(`${sourceName}: duplicate metadata key ${key}`);
      if (key === "order") {
        if (value) throw new Error(`${sourceName}: order must be a YAML list`);
        meta.order = [];
        readingOrder = true;
      } else {
        if (!value) throw new Error(`${sourceName}: empty metadata value ${key}`);
        meta[key] = value;
        readingOrder = false;
      }
      continue;
    }
    throw new Error(`${sourceName}: unsupported YAML syntax: ${rawLine}`);
  }

  if (!meta.title || !Array.isArray(meta.order) || !meta.order.length) {
    throw new Error(`${sourceName}: title and non-empty order are required`);
  }
  if (new Set(meta.order).size !== meta.order.length) {
    throw new Error(`${sourceName}: order entries must be unique`);
  }
  return Object.freeze({
    ...meta,
    order: Object.freeze(meta.order)
  });
}

export function parseHelpLandingPage(source, sourceName = "index.md") {
  const lines = source.replaceAll("\r\n", "\n").split("\n");
  if (lines[0] !== "---") throw new Error(`${sourceName}: missing YAML front matter`);
  const end = lines.indexOf("---", 1);
  if (end < 2) throw new Error(`${sourceName}: unterminated YAML front matter`);
  if (lines.slice(end + 1).some((line) => line.trim())) {
    throw new Error(`${sourceName}: landing pages must contain only YAML front matter`);
  }

  const frontmatter = {};
  for (const line of lines.slice(1, end)) {
    if (!line.trim()) continue;
    const [key, value] = parseKeyValue(line, sourceName);
    if (!INDEX_KEYS.has(key)) throw new Error(`${sourceName}: unknown front matter key ${key}`);
    if (Object.hasOwn(frontmatter, key)) throw new Error(`${sourceName}: duplicate front matter key ${key}`);
    if (!value) throw new Error(`${sourceName}: empty front matter value ${key}`);
    frontmatter[key] = value;
  }
  for (const key of INDEX_KEYS) {
    if (!Object.hasOwn(frontmatter, key)) {
      throw new Error(`${sourceName}: missing front matter key ${key}`);
    }
  }
  if (frontmatter.difficulty !== "basic") {
    throw new Error(`${sourceName}: landing-page difficulty must be basic`);
  }
  if (frontmatter.icon !== INDEX_ICON) {
    throw new Error(`${sourceName}: basic landing pages must use ${INDEX_ICON}`);
  }
  if (frontmatter.toc !== "true") {
    throw new Error(`${sourceName}: toc must be true`);
  }
  return Object.freeze(frontmatter);
}

async function assertOrderTargets(directory, meta, sourceName) {
  for (const entry of meta.order) {
    const markdown = resolve(directory, `${entry}.md`);
    const section = resolve(directory, entry);
    try {
      await access(markdown);
      continue;
    } catch {}
    try {
      await access(section);
      continue;
    } catch {}
    throw new Error(`${sourceName}: order entry ${entry} has no matching Markdown page or section`);
  }
}

async function validateSection(helpRoot, section, expectedOrder, expectedIcon) {
  const directory = resolve(helpRoot, section);
  const metaPath = resolve(directory, "_meta.yaml");
  const indexPath = resolve(directory, "index.md");
  const meta = parseHelpFolderMeta(await readFile(metaPath, "utf8"), metaPath);
  const landing = parseHelpLandingPage(await readFile(indexPath, "utf8"), indexPath);

  if (meta.title !== landing.title) {
    throw new Error(`${metaPath}: title must match ${indexPath}`);
  }
  if (meta.icon !== expectedIcon) {
    throw new Error(`${metaPath}: section icon must be ${expectedIcon}`);
  }
  if (JSON.stringify(meta.order) !== JSON.stringify(expectedOrder)) {
    throw new Error(`${metaPath}: order does not match the canonical Help topic order`);
  }
  await assertOrderTargets(directory, meta, metaPath);
}

function immediateEntries(topics, prefix = "") {
  return [...new Set(topics
    .map(({ file }) => prefix ? file.slice(prefix.length + 1) : file)
    .filter((file) => file && !file.startsWith("../"))
    .map((file) => file.split("/")[0]))];
}

export async function validateHelpContentMaps(demoRoot, lut, locales) {
  const runtimeSections = [...new Set(lut.topics.map(({ group }) => group))];
  const rootArticles = lut.topics
    .filter(({ source }) => source === "root")
    .map(({ file }) => file);
  const rootOrder = ["index", ...rootArticles, ...runtimeSections, ...Object.keys(SUPPLEMENTAL_SECTIONS)];
  const groupIcons = Object.fromEntries(HELP_GROUP_DEFINITIONS.map(({ id, iconId }) => [id, iconId]));

  for (const locale of locales) {
    const helpRoot = resolve(demoRoot, "help", locale);
    const rootMetaPath = resolve(helpRoot, "_meta.yaml");
    const rootIndexPath = resolve(helpRoot, "index.md");
    const rootMeta = parseHelpFolderMeta(await readFile(rootMetaPath, "utf8"), rootMetaPath);
    const rootLanding = parseHelpLandingPage(await readFile(rootIndexPath, "utf8"), rootIndexPath);

    if (rootMeta.title !== rootLanding.title) {
      throw new Error(`${rootMetaPath}: title must match ${rootIndexPath}`);
    }
    if (rootMeta.icon !== undefined) {
      throw new Error(`${rootMetaPath}: the domain root must not override its structural icon`);
    }
    if (JSON.stringify(rootMeta.order) !== JSON.stringify(rootOrder)) {
      throw new Error(`${rootMetaPath}: order does not match the canonical Help section order`);
    }
    await assertOrderTargets(helpRoot, rootMeta, rootMetaPath);

    for (const section of runtimeSections) {
      const sectionTopics = lut.topics.filter(({ group, source }) => group === section && source !== "root");
      const groupEntries = immediateEntries(sectionTopics);
      await validateSection(helpRoot, section, ["index", ...groupEntries], groupIcons[section]);
      for (const nested of groupEntries.filter((entry) => sectionTopics.some(({ file }) => file.startsWith(`${entry}/`)))) {
        const nestedPath = `${section}/${nested}`;
        await validateSection(
          helpRoot,
          nestedPath,
          ["index", ...immediateEntries(sectionTopics.filter(({ file }) => file.startsWith(`${nested}/`)), nested)],
          NESTED_SECTION_ICONS[nestedPath]
        );
      }
    }
    for (const [section, contract] of Object.entries(SUPPLEMENTAL_SECTIONS)) {
      await validateSection(helpRoot, section, contract.order, contract.icon);
    }
  }
}

export const validateEnglishHelpContentMap = validateHelpContentMaps;
