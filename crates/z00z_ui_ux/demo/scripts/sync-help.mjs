import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { loadHelpLocales, loadHelpSource, parseHelpMarkdown } from "./help-source.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const SOURCE_LOCALE = "en";
const STATE_VERSION = 1;
const HASH_PREFIX = "sha256:";
const statePathFor = (root) => resolve(root, "help/source-state.json");

function canonicalDocument(document) {
  return {
    id: document.id,
    title: document.title,
    summary: document.summary,
    scope: document.scope,
    sections: document.sections.map((section) => ({
      title: section.title,
      target: section.target,
      blocks: section.blocks.map((block) => block.type === "list"
        ? { type: "list", items: [...block.items] }
        : { type: "paragraph", text: block.text })
    }))
  };
}

export function helpSourceHash(document) {
  return `${HASH_PREFIX}${createHash("sha256")
    .update(JSON.stringify(canonicalDocument(document)))
    .digest("hex")}`;
}

export function helpStructure(document) {
  return document.sections.map((section) => ({
    target: section.target,
    blocks: section.blocks.map((block) => block.type === "list"
      ? { type: "list", items: block.items.length }
      : { type: "paragraph" })
  }));
}

export function helpMessages(document) {
  const messages = {
    "document.title": document.title,
    "document.summary": document.summary
  };
  document.sections.forEach((section, sectionIndex) => {
    messages[`sections.${sectionIndex}.title`] = section.title;
    section.blocks.forEach((block, blockIndex) => {
      if (block.type === "paragraph") {
        messages[`sections.${sectionIndex}.blocks.${blockIndex}.text`] = block.text;
        return;
      }
      block.items.forEach((item, itemIndex) => {
        messages[`sections.${sectionIndex}.blocks.${blockIndex}.items.${itemIndex}`] = item;
      });
    });
  });
  return Object.freeze(messages);
}

function translatedValue(messages, key) {
  const value = messages[key];
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(`Translation is missing non-empty message ${key}`);
  }
  if (/[\r\n]/.test(value) || /<\/?[a-z][^>]*>/i.test(value) || /\b(?:javascript|data):/i.test(value)) {
    throw new Error(`Translation message ${key} contains unsupported Markdown content`);
  }
  return value.trim();
}

export function localizeHelpDocument(sourceDocument, translatedMessages) {
  return {
    id: sourceDocument.id,
    title: translatedValue(translatedMessages, "document.title"),
    summary: translatedValue(translatedMessages, "document.summary"),
    scope: sourceDocument.scope,
    sections: sourceDocument.sections.map((section, sectionIndex) => ({
      title: translatedValue(translatedMessages, `sections.${sectionIndex}.title`),
      target: section.target,
      blocks: section.blocks.map((block, blockIndex) => block.type === "list"
        ? {
            type: "list",
            items: block.items.map((_item, itemIndex) => translatedValue(
              translatedMessages,
              `sections.${sectionIndex}.blocks.${blockIndex}.items.${itemIndex}`
            ))
          }
        : {
            type: "paragraph",
            text: translatedValue(translatedMessages, `sections.${sectionIndex}.blocks.${blockIndex}.text`)
          })
    }))
  };
}

export function serializeHelpMarkdown(document) {
  const body = document.sections.map((section) => {
    const heading = `## ${section.title}${section.target ? ` {#${section.target}}` : ""}`;
    const blocks = section.blocks.map((block) => block.type === "list"
      ? block.items.map((item) => `- ${item}`).join("\n")
      : block.text);
    return `${heading}\n${blocks.join("\n\n")}`;
  }).join("\n\n");
  return `---\nid: ${document.id}\ntitle: ${document.title}\nsummary: ${document.summary}\nscope: ${document.scope}\n---\n${body}\n`;
}

async function loadDocument(root, locale, file) {
  const path = resolve(root, "help", locale, `${file}.md`);
  return {
    path,
    document: parseHelpMarkdown(await readFile(path, "utf8"), path)
  };
}

async function loadState(root) {
  const path = statePathFor(root);
  try {
    const state = JSON.parse(await readFile(path, "utf8"));
    if (
      state.version !== STATE_VERSION
      || state.sourceLocale !== SOURCE_LOCALE
      || state.hashAlgorithm !== "sha256"
      || !state.topics
    ) {
      throw new Error(`${path}: unsupported Help source-state schema`);
    }
    return state;
  } catch (error) {
    if (error.code === "ENOENT") {
      return {
        version: STATE_VERSION,
        sourceLocale: SOURCE_LOCALE,
        hashAlgorithm: "sha256",
        topics: {}
      };
    }
    throw error;
  }
}

async function writeState(root, state) {
  const path = statePathFor(root);
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, `${JSON.stringify(state, null, 2)}\n`, "utf8");
}

function translationCommand(command, locale, topic, sourceHash, messages) {
  const translated = spawnSync(command, [locale], {
    shell: false,
    encoding: "utf8",
    input: JSON.stringify({
      contentType: "z00z-help-messages-v1",
      language: locale,
      sourceLanguage: SOURCE_LOCALE,
      topic: topic.id,
      sourceHash,
      messages
    })
  });
  if (translated.error) throw translated.error;
  if (translated.status !== 0) {
    throw new Error(`Local Help translator failed for ${locale}/${topic.id}: ${translated.stderr.trim()}`);
  }
  const parsed = JSON.parse(translated.stdout);
  return parsed.messages && typeof parsed.messages === "object" ? parsed.messages : parsed;
}

function topicState(sourceHash, localeIds) {
  return {
    sourceHash,
    locales: Object.fromEntries(localeIds.map((locale) => [locale, sourceHash]))
  };
}

export async function recordReviewedHelpState(root = demoRoot) {
  const { lut } = await loadHelpSource(root);
  const localeIds = await loadHelpLocales(root);
  const topics = {};
  for (const topic of lut.topics) {
    const source = await loadDocument(root, SOURCE_LOCALE, topic.file);
    const structure = JSON.stringify(helpStructure(source.document));
    for (const locale of localeIds) {
      const localized = await loadDocument(root, locale, topic.file);
      if (JSON.stringify(helpStructure(localized.document)) !== structure) {
        throw new Error(`${localized.path}: structure does not match English source ${source.path}`);
      }
    }
    topics[topic.id] = topicState(helpSourceHash(source.document), localeIds);
  }
  const state = {
    version: STATE_VERSION,
    sourceLocale: SOURCE_LOCALE,
    hashAlgorithm: "sha256",
    topics
  };
  await writeState(root, state);
  return state;
}

export async function assertHelpSynchronized(root = demoRoot) {
  const { lut } = await loadHelpSource(root);
  const localeIds = await loadHelpLocales(root);
  const state = await loadState(root);
  const expectedTopicIds = lut.topics.map(({ id }) => id).sort();
  const stateTopicIds = Object.keys(state.topics).sort();
  if (JSON.stringify(expectedTopicIds) !== JSON.stringify(stateTopicIds)) {
    throw new Error("Help source-state topics are stale; run node scripts/sync-help.mjs");
  }

  for (const topic of lut.topics) {
    const source = await loadDocument(root, SOURCE_LOCALE, topic.file);
    const sourceHash = helpSourceHash(source.document);
    const entry = state.topics[topic.id];
    if (entry.sourceHash !== sourceHash) {
      throw new Error(`English Help changed for ${topic.id}; run node scripts/sync-help.mjs`);
    }
    const synchronizedLocales = Object.keys(entry.locales || {}).sort();
    if (
      JSON.stringify(synchronizedLocales) !== JSON.stringify([...localeIds].sort())
      || localeIds.some((locale) => entry.locales[locale] !== sourceHash)
    ) {
      throw new Error(`Help locale synchronization is stale for ${topic.id}; run node scripts/sync-help.mjs`);
    }
  }
  return state;
}

export async function synchronizeHelp(root = demoRoot, options = {}) {
  const { lut } = await loadHelpSource(root);
  const localeIds = await loadHelpLocales(root);
  const state = await loadState(root);
  const translator = options.translatorCommand ?? process.env.Z00Z_TRANSLATE_COMMAND;
  const changedTopics = [];

  for (const topic of lut.topics) {
    const source = await loadDocument(root, SOURCE_LOCALE, topic.file);
    const sourceHash = helpSourceHash(source.document);
    const previous = state.topics[topic.id];
    const staleLocales = localeIds.filter((locale) => (
      locale !== SOURCE_LOCALE
      && (options.force || previous?.sourceHash !== sourceHash || previous?.locales?.[locale] !== sourceHash)
    ));
    if (!staleLocales.length) {
      state.topics[topic.id] = topicState(sourceHash, localeIds);
      continue;
    }
    if (!translator) {
      throw new Error(
        `English Help changed for ${topic.id}. Set Z00Z_TRANSLATE_COMMAND to the local translation bridge, then rerun node scripts/sync-help.mjs.`
      );
    }

    const messages = helpMessages(source.document);
    for (const locale of staleLocales) {
      const translatedMessages = translationCommand(translator, locale, topic, sourceHash, messages);
      const localized = localizeHelpDocument(source.document, translatedMessages);
      const output = serializeHelpMarkdown(localized);
      parseHelpMarkdown(output, `${locale}/${topic.file}.md`);
      await writeFile(resolve(root, "help", locale, `${topic.file}.md`), output, "utf8");
    }
    state.topics[topic.id] = topicState(sourceHash, localeIds);
    changedTopics.push(topic.id);
  }

  const liveTopicIds = new Set(lut.topics.map(({ id }) => id));
  Object.keys(state.topics).forEach((topicId) => {
    if (!liveTopicIds.has(topicId)) delete state.topics[topicId];
  });
  await writeState(root, state);
  return Object.freeze(changedTopics);
}

async function main() {
  if (process.argv.includes("--record-reviewed")) {
    await recordReviewedHelpState(demoRoot);
    console.log("Recorded reviewed Help translations against the current English source hashes.");
    return;
  }
  const changedTopics = await synchronizeHelp(demoRoot, { force: process.argv.includes("--force") });
  console.log(changedTopics.length
    ? `Synchronized Help translations: ${changedTopics.join(", ")}`
    : "Help translations already match the English source hashes.");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
