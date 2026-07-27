import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { access, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  helpDocumentPath,
  loadHelpLocales,
  loadHelpSource,
  parseHelpMarkdown
} from "./help-source.mjs";
import { synchronizeHelpLayout } from "./sync-help-layout.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const SOURCE_LOCALE = "en";
const STATE_VERSION = 2;
const HASH_PREFIX = "sha256:";
const bundledTranslator = resolve(scriptDirectory, "local-help-translate.mjs");
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

function messageHash(value) {
  return `${HASH_PREFIX}${createHash("sha256").update(value).digest("hex")}`;
}

export function helpMessageHashes(messages) {
  return Object.freeze(Object.fromEntries(Object.entries(messages)
    .map(([key, value]) => [key, messageHash(value)])));
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

async function loadDocument(root, locale, topic) {
  const path = helpDocumentPath(root, locale, topic);
  return {
    path,
    document: parseHelpMarkdown(await readFile(path, "utf8"), path)
  };
}

async function loadLocalizedDocumentOrSource(root, locale, topic, source) {
  try {
    return await loadDocument(root, locale, topic);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    return {
      path: helpDocumentPath(root, locale, topic),
      document: source.document,
      missing: true
    };
  }
}

async function helpDocumentExists(root, locale, topic) {
  try {
    await access(helpDocumentPath(root, locale, topic));
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

async function loadState(root) {
  const path = statePathFor(root);
  try {
    const state = JSON.parse(await readFile(path, "utf8"));
    if (
      ![1, STATE_VERSION].includes(state.version)
      || state.sourceLocale !== SOURCE_LOCALE
      || state.hashAlgorithm !== "sha256"
      || !state.topics
    ) {
      throw new Error(`${path}: unsupported Help source-state schema`);
    }
    return {
      ...state,
      version: STATE_VERSION,
      topics: Object.fromEntries(Object.entries(state.topics).map(([topicId, entry]) => [
        topicId,
        {
          ...entry,
          sourceMessageHashes: entry.sourceMessageHashes
            || (entry.sourceMessages ? helpMessageHashes(entry.sourceMessages) : null)
        }
      ]))
    };
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

function translationCommand(
  command,
  locale,
  topic,
  sourceHash,
  messages,
  previousMessageHashes,
  currentMessages
) {
  const translated = spawnSync(command, [locale], {
    shell: false,
    encoding: "utf8",
    input: JSON.stringify({
      contentType: "z00z-help-messages-v1",
      language: locale,
      sourceLanguage: SOURCE_LOCALE,
      topic: topic.id,
      sourceHash,
      messages,
      previousMessageHashes,
      currentMessages
    })
  });
  if (translated.error) throw translated.error;
  if (translated.status !== 0) {
    throw new Error(`Local Help translator failed for ${locale}/${topic.id}: ${translated.stderr.trim()}`);
  }
  const parsed = JSON.parse(translated.stdout);
  return {
    messages: parsed.messages && typeof parsed.messages === "object" ? parsed.messages : parsed,
    fallbackKeys: Array.isArray(parsed.fallbackKeys)
      ? parsed.fallbackKeys.filter((key) => typeof key === "string")
      : []
  };
}

function localeSourceHash(entry) {
  return typeof entry === "string" ? entry : entry?.sourceHash;
}

function localeReviewHash(entry) {
  return typeof entry === "object" ? entry?.reviewHash : "";
}

async function topicState(root, topic, sourceHash, sourceMessageHashes, localeIds) {
  const locales = {};
  for (const locale of localeIds) {
    const localized = await loadDocument(root, locale, topic);
    locales[locale] = {
      sourceHash,
      reviewHash: helpSourceHash(localized.document)
    };
  }
  return {
    sourceHash,
    sourceMessageHashes,
    locales
  };
}

export async function recordReviewedHelpState(root = demoRoot) {
  const { lut } = await loadHelpSource(root);
  const localeIds = await loadHelpLocales(root);
  const topics = {};
  for (const topic of lut.topics) {
    const source = await loadDocument(root, SOURCE_LOCALE, topic);
    const structure = JSON.stringify(helpStructure(source.document));
    for (const locale of localeIds) {
      const localized = await loadDocument(root, locale, topic);
      if (JSON.stringify(helpStructure(localized.document)) !== structure) {
        throw new Error(`${localized.path}: structure does not match English source ${source.path}`);
      }
    }
    const sourceHash = helpSourceHash(source.document);
    topics[topic.id] = await topicState(
      root,
      topic,
      sourceHash,
      helpMessageHashes(helpMessages(source.document)),
      localeIds
    );
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
    const source = await loadDocument(root, SOURCE_LOCALE, topic);
    const sourceHash = helpSourceHash(source.document);
    const sourceMessages = helpMessages(source.document);
    const sourceMessageHashes = helpMessageHashes(sourceMessages);
    const entry = state.topics[topic.id];
    if (entry.sourceHash !== sourceHash) {
      throw new Error(`English Help changed for ${topic.id}; run node scripts/sync-help.mjs`);
    }
    if (JSON.stringify(entry.sourceMessageHashes) !== JSON.stringify(sourceMessageHashes)) {
      throw new Error(`English Help message state is stale for ${topic.id}; run node scripts/sync-help.mjs`);
    }
    const synchronizedLocales = Object.keys(entry.locales || {}).sort();
    if (
      JSON.stringify(synchronizedLocales) !== JSON.stringify([...localeIds].sort())
      || localeIds.some((locale) => localeSourceHash(entry.locales[locale]) !== sourceHash)
    ) {
      throw new Error(`Help locale synchronization is stale for ${topic.id}; run node scripts/sync-help.mjs`);
    }
    for (const locale of localeIds) {
      const reviewHash = localeReviewHash(entry.locales[locale]);
      if (!reviewHash) {
        throw new Error(`Help native-language review hash is missing for ${locale}/${topic.id}; run node scripts/sync-help.mjs`);
      }
      const localized = await loadDocument(root, locale, topic);
      if (helpSourceHash(localized.document) !== reviewHash) {
        throw new Error(`Help native-language review hash is stale for ${locale}/${topic.id}; rerun the local review workflow`);
      }
    }
  }
  return state;
}

export async function synchronizeHelp(root = demoRoot, options = {}) {
  const changedPaths = [...await synchronizeHelpLayout(root)];
  const { lut } = await loadHelpSource(root);
  const localeIds = await loadHelpLocales(root);
  const state = await loadState(root);
  const translator = options.translatorCommand !== undefined
    ? options.translatorCommand
    : (process.env.Z00Z_TRANSLATE_COMMAND || bundledTranslator);
  const changedTopics = [];
  const fallbacks = [];

  for (const topic of lut.topics) {
    const source = await loadDocument(root, SOURCE_LOCALE, topic);
    const sourceHash = helpSourceHash(source.document);
    const messages = helpMessages(source.document);
    const messageHashes = helpMessageHashes(messages);
    const previous = state.topics[topic.id];
    const staleLocales = [];
    for (const locale of localeIds) {
      if (
        locale !== SOURCE_LOCALE
        && (
          options.force
          || !await helpDocumentExists(root, locale, topic)
          || previous?.sourceHash !== sourceHash
          || localeSourceHash(previous?.locales?.[locale]) !== sourceHash
        )
      ) {
        staleLocales.push(locale);
      }
    }
    if (!staleLocales.length) {
      state.topics[topic.id] = await topicState(root, topic, sourceHash, messageHashes, localeIds);
      continue;
    }
    if (!translator) {
      throw new Error(
        `English Help changed for ${topic.id}. Set Z00Z_TRANSLATE_COMMAND to the local translation bridge, then rerun node scripts/sync-help.mjs.`
      );
    }

    for (const locale of staleLocales) {
      const current = await loadLocalizedDocumentOrSource(root, locale, topic, source);
      const currentIsSourceFallback = current.missing
        || (
          helpSourceHash(current.document) === sourceHash
          && localeReviewHash(previous?.locales?.[locale]) === sourceHash
        );
      const translation = translationCommand(
        translator,
        locale,
        topic,
        sourceHash,
        messages,
        previous?.sourceMessageHashes,
        currentIsSourceFallback ? {} : helpMessages(current.document)
      );
      const localized = localizeHelpDocument(source.document, translation.messages);
      if (translation.fallbackKeys.length) {
        fallbacks.push(Object.freeze({
          topic: topic.id,
          locale,
          keys: Object.freeze([...translation.fallbackKeys])
        }));
      }
      const output = serializeHelpMarkdown(localized);
      const outputPath = helpDocumentPath(root, locale, topic);
      parseHelpMarkdown(output, `${locale}/${topic.group}/${topic.file}.md`);
      await mkdir(dirname(outputPath), { recursive: true });
      await writeFile(outputPath, output, "utf8");
      changedPaths.push(outputPath.slice(root.length + 1).replaceAll("\\", "/"));
    }
    state.topics[topic.id] = await topicState(root, topic, sourceHash, messageHashes, localeIds);
    changedTopics.push(topic.id);
  }

  const liveTopicIds = new Set(lut.topics.map(({ id }) => id));
  Object.keys(state.topics).forEach((topicId) => {
    if (!liveTopicIds.has(topicId)) delete state.topics[topicId];
  });
  await writeState(root, state);
  Object.defineProperty(changedTopics, "paths", {
    value: Object.freeze(changedPaths),
    enumerable: false
  });
  Object.defineProperty(changedTopics, "fallbacks", {
    value: Object.freeze(fallbacks),
    enumerable: false
  });
  return Object.freeze(changedTopics);
}

async function main() {
  if (process.argv.includes("--record-reviewed")) {
    await recordReviewedHelpState(demoRoot);
    console.log("Recorded reviewed Help translations against the current English source hashes.");
    return;
  }
  const changedTopics = await synchronizeHelp(demoRoot, { force: process.argv.includes("--force") });
  if (process.argv.includes("--json")) {
    console.log(JSON.stringify({
      topics: [...changedTopics],
      paths: changedTopics.paths,
      fallbacks: changedTopics.fallbacks
    }));
    return;
  }
  console.log(changedTopics.length
    ? `Synchronized Help translations: ${changedTopics.join(", ")}`
    : "Help translations already match the English source hashes.");
  if (changedTopics.fallbacks.length) {
    console.warn(
      `English fallback retained for ${changedTopics.fallbacks.length} locale/topic updates; native-language review is required.`
    );
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
