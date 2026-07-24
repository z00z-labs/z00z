import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import vm from "node:vm";

const TOPIC_KEYS = new Set(["id", "file", "scope", "match"]);
const FRONT_MATTER_KEYS = new Set(["id", "title", "summary", "scope"]);
const SAFE_ID = /^[a-z0-9][a-z0-9.-]*$/;
const SAFE_FILE = /^[a-z0-9][a-z0-9-]*$/;
const SAFE_TARGET = /^[a-z][a-z0-9-]*$/;
const SAFE_LOCALE = /^[A-Za-z0-9]+(?:-[A-Za-z0-9]+)*$/;

function parseScalar(line, sourceName) {
  const separator = line.indexOf(":");
  if (separator < 1) throw new Error(`${sourceName}: malformed YAML line: ${line}`);
  return [line.slice(0, separator).trim(), line.slice(separator + 1).trim()];
}

export function parseTopicLut(source, sourceName = "topics.yaml") {
  const lines = source.replaceAll("\r\n", "\n").split("\n");
  const topics = [];
  let version = null;
  let current = null;

  for (const rawLine of lines) {
    if (!rawLine.trim() || rawLine.trimStart().startsWith("#")) continue;
    if (/^version:\s*/.test(rawLine)) {
      version = Number(parseScalar(rawLine, sourceName)[1]);
      continue;
    }
    if (rawLine === "topics:") continue;
    if (/^  - /.test(rawLine)) {
      if (current) topics.push(current);
      current = {};
      const [key, value] = parseScalar(rawLine.slice(4), sourceName);
      current[key] = value;
      continue;
    }
    if (/^    [a-z]+:/.test(rawLine) && current) {
      const [key, value] = parseScalar(rawLine.trim(), sourceName);
      if (!TOPIC_KEYS.has(key)) throw new Error(`${sourceName}: unknown topic key ${key}`);
      current[key] = value;
      continue;
    }
    throw new Error(`${sourceName}: unsupported YAML syntax: ${rawLine}`);
  }
  if (current) topics.push(current);
  if (version !== 1) throw new Error(`${sourceName}: version must be 1`);
  if (!topics.length) throw new Error(`${sourceName}: no topics`);

  const ids = new Set();
  const files = new Set();
  for (const topic of topics) {
    for (const key of TOPIC_KEYS) {
      if (!topic[key]) throw new Error(`${sourceName}: topic is missing ${key}`);
    }
    if (!SAFE_ID.test(topic.id)) throw new Error(`${sourceName}: unsafe topic id ${topic.id}`);
    if (!SAFE_FILE.test(topic.file)) throw new Error(`${sourceName}: unsafe topic file ${topic.file}`);
    if (!["global", "context", "dialog"].includes(topic.scope)) throw new Error(`${sourceName}: invalid scope ${topic.scope}`);
    if (ids.has(topic.id)) throw new Error(`${sourceName}: duplicate id ${topic.id}`);
    if (files.has(topic.file)) throw new Error(`${sourceName}: duplicate file ${topic.file}`);
    ids.add(topic.id);
    files.add(topic.file);
    topic.match = topic.match === "global"
      ? Object.freeze({ global: "true" })
      : Object.freeze(Object.fromEntries(topic.match.split(";").map((entry) => {
          const [key, value] = entry.split("=");
          if (!key || !value) throw new Error(`${sourceName}: invalid match ${topic.match}`);
          return [key, value];
        })));
    Object.freeze(topic);
  }
  return Object.freeze({ version, topics: Object.freeze(topics) });
}

function parseFrontMatter(lines, sourceName) {
  if (lines[0] !== "---") throw new Error(`${sourceName}: missing YAML front matter`);
  const end = lines.indexOf("---", 1);
  if (end < 2) throw new Error(`${sourceName}: unterminated YAML front matter`);
  const meta = {};
  for (const line of lines.slice(1, end)) {
    const [key, value] = parseScalar(line, sourceName);
    if (!FRONT_MATTER_KEYS.has(key)) throw new Error(`${sourceName}: unknown front matter key ${key}`);
    if (meta[key]) throw new Error(`${sourceName}: duplicate front matter key ${key}`);
    if (!value) throw new Error(`${sourceName}: empty front matter value ${key}`);
    meta[key] = value;
  }
  for (const key of FRONT_MATTER_KEYS) {
    if (!meta[key]) throw new Error(`${sourceName}: missing front matter key ${key}`);
  }
  return { meta, bodyStart: end + 1 };
}

export function parseHelpMarkdown(source, sourceName = "help.md") {
  if (/<\/?[a-z][^>]*>/i.test(source) || /\b(?:javascript|data):/i.test(source)) {
    throw new Error(`${sourceName}: raw HTML and unsafe URLs are not supported`);
  }
  const lines = source.replaceAll("\r\n", "\n").split("\n");
  const { meta, bodyStart } = parseFrontMatter(lines, sourceName);
  if (!SAFE_ID.test(meta.id)) throw new Error(`${sourceName}: unsafe id ${meta.id}`);
  if (!["global", "context", "dialog"].includes(meta.scope)) throw new Error(`${sourceName}: invalid scope ${meta.scope}`);

  const sections = [];
  let current = null;
  for (const rawLine of lines.slice(bodyStart)) {
    const line = rawLine.trim();
    if (!line) continue;
    const heading = /^## ([^{}]+?)(?: \{#([a-z][a-z0-9-]*)\})?$/.exec(line);
    if (heading) {
      if (current) sections.push(current);
      current = { title: heading[1].trim(), target: heading[2] || "", blocks: [] };
      if (current.target && !SAFE_TARGET.test(current.target)) throw new Error(`${sourceName}: invalid target ${current.target}`);
      continue;
    }
    if (/^#{1,6}\s/.test(line)) throw new Error(`${sourceName}: only level-two section headings are supported`);
    if (!current) throw new Error(`${sourceName}: content must follow a level-two heading`);
    if (/^[-*] /.test(line)) {
      const text = line.slice(2).trim();
      const previous = current.blocks.at(-1);
      if (previous?.type === "list") previous.items.push(text);
      else current.blocks.push({ type: "list", items: [text] });
    } else {
      const previous = current.blocks.at(-1);
      if (previous?.type === "paragraph") previous.text += ` ${line}`;
      else current.blocks.push({ type: "paragraph", text: line });
    }
  }
  if (current) sections.push(current);
  if (!sections.length) throw new Error(`${sourceName}: at least one section is required`);
  return Object.freeze({
    ...meta,
    sections: Object.freeze(sections.map((section) => Object.freeze({
      ...section,
      blocks: Object.freeze(section.blocks.map((block) => Object.freeze(block.type === "list"
        ? { type: "list", items: Object.freeze(block.items) }
        : block)))
    })))
  });
}

export async function loadHelpSource(root) {
  const topicPath = resolve(root, "help/topics.yaml");
  const lut = parseTopicLut(await readFile(topicPath, "utf8"), topicPath);
  return { lut, topicPath };
}

export async function loadHelpLocales(root) {
  const sourcePath = resolve(root, "scripts/port/locale-registry.js");
  const context = { window: {} };
  vm.runInNewContext(await readFile(sourcePath, "utf8"), context, { filename: sourcePath });
  const registry = context.window.Z00ZLocaleRegistry;
  if (!Array.isArray(registry) || !registry.length) {
    throw new Error(`${sourcePath}: locale registry is empty`);
  }
  const ids = registry.map(({ id }) => id);
  if (new Set(ids).size !== ids.length || ids.some((id) => !SAFE_LOCALE.test(id))) {
    throw new Error(`${sourcePath}: locale IDs must be unique and safe`);
  }
  return Object.freeze(ids);
}
