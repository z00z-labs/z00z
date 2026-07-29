import { readFile, readdir } from "node:fs/promises";
import { relative, resolve } from "node:path";

import "../port/locale-registry.js";
import { helpRecords, pageFile } from "./navigation-contract.mjs";
import { renderHelpMarkdown } from "./markdown-renderer.mjs";

const REQUIRED_HEADINGS = Object.freeze([
  "## App View {#current-view}",
  "## Overview",
  "## How to use this view",
  "## Terms and controls",
  "## Safety and limits",
]);

const SUPPORTED_LOCALES = Object.freeze(globalThis.Z00ZLocaleRegistry.map(({ id }) => id));

const LOCALIZED_TEMPLATE_COPY = Object.freeze({
  ru: Object.freeze({ appView: "Экран приложения", overview: "Обзор", howToUse: "Как использовать этот экран", terms: "Термины и элементы управления", safety: "Безопасность и ограничения", imageNote: "Этот снимок экрана получен из текущего представления Demo.", term: "Термин или элемент", explanation: "Объяснение", review: "Требуется проверка", reviewNote: "Объясните видимые на этом экране элементы и состояния." }),
  fr: Object.freeze({ appView: "Vue de l’application", overview: "Vue d’ensemble", howToUse: "Utiliser cet écran", terms: "Termes et commandes", safety: "Sécurité et limites", imageNote: "Cette image est capturée depuis la vue Demo actuelle.", term: "Terme ou commande", explanation: "Explication", review: "Révision requise", reviewNote: "Expliquez les commandes et états visibles dans cette vue." }),
  de: Object.freeze({ appView: "App-Ansicht", overview: "Überblick", howToUse: "Diese Ansicht verwenden", terms: "Begriffe und Steuerelemente", safety: "Sicherheit und Grenzen", imageNote: "Dieses Bild wurde aus der aktuellen Demo-Ansicht aufgenommen.", term: "Begriff oder Steuerelement", explanation: "Erklärung", review: "Prüfung erforderlich", reviewNote: "Erläutern Sie die in dieser Ansicht sichtbaren Steuerelemente und Zustände." }),
  es: Object.freeze({ appView: "Vista de la aplicación", overview: "Descripción general", howToUse: "Cómo usar esta vista", terms: "Términos y controles", safety: "Seguridad y límites", imageNote: "Esta imagen se captura desde la vista Demo actual.", term: "Término o control", explanation: "Explicación", review: "Revisión necesaria", reviewNote: "Explique los controles y estados visibles en esta vista." }),
  pt: Object.freeze({ appView: "Vista da aplicação", overview: "Visão geral", howToUse: "Como utilizar esta vista", terms: "Termos e controlos", safety: "Segurança e limites", imageNote: "Esta imagem é capturada a partir da vista Demo atual.", term: "Termo ou controlo", explanation: "Explicação", review: "Revisão necessária", reviewNote: "Explique os controlos e estados visíveis nesta vista." }),
  ko: Object.freeze({ appView: "앱 화면", overview: "개요", howToUse: "이 화면 사용 방법", terms: "용어 및 제어", safety: "안전 및 제한", imageNote: "이 이미지는 현재 Demo 화면에서 캡처되었습니다.", term: "용어 또는 제어", explanation: "설명", review: "검토 필요", reviewNote: "이 화면에 보이는 제어와 상태를 설명하세요." }),
  tr: Object.freeze({ appView: "Uygulama görünümü", overview: "Genel bakış", howToUse: "Bu görünüm nasıl kullanılır", terms: "Terimler ve denetimler", safety: "Güvenlik ve sınırlar", imageNote: "Bu görüntü güncel Demo görünümünden alınmıştır.", term: "Terim veya denetim", explanation: "Açıklama", review: "İnceleme gerekli", reviewNote: "Bu görünümdeki denetimleri ve durumları açıklayın." }),
  ja: Object.freeze({ appView: "アプリ画面", overview: "概要", howToUse: "この画面の使い方", terms: "用語と操作", safety: "安全性と制限", imageNote: "この画像は現在の Demo 画面から取得されています。", term: "用語または操作", explanation: "説明", review: "要レビュー", reviewNote: "この画面に表示される操作と状態を説明してください。" }),
  "zh-Hans": Object.freeze({ appView: "应用视图", overview: "概览", howToUse: "如何使用此视图", terms: "术语和控件", safety: "安全性和限制", imageNote: "此图像来自当前 Demo 视图。", term: "术语或控件", explanation: "说明", review: "需要审核", reviewNote: "请说明此视图中可见的控件和状态。" }),
});

function parseFrontmatter(source, sourceName) {
  if (!source.startsWith("---\n")) throw new Error(`${sourceName}: missing YAML front matter.`);
  const closing = source.indexOf("\n---\n", 4);
  if (closing < 0) throw new Error(`${sourceName}: unterminated YAML front matter.`);

  const frontmatter = Object.fromEntries(source.slice(4, closing).split("\n").map((line) => {
    const separator = line.indexOf(": ");
    if (separator < 1) throw new Error(`${sourceName}: malformed front matter line.`);
    return [line.slice(0, separator), line.slice(separator + 2)];
  }));
  return { body: source.slice(closing + 5), frontmatter };
}

function sourceMarker(source, sourceName) {
  const match = source.match(/<!-- help-sync:source (\{.+\}) -->/u);
  if (!match) throw new Error(`${sourceName}: missing Help sync provenance.`);

  try {
    return JSON.parse(match[1]);
  } catch {
    throw new Error(`${sourceName}: invalid Help sync provenance.`);
  }
}

function plainText(html) {
  return html
    .replace(/<[^>]+>/gu, " ")
    .replace(/&(?:amp|lt|gt|quot|#39);/gu, " ")
    .replace(/\s+/gu, " ")
    .trim();
}

function documentFromMarkdown(markdown, record, sourceName, title) {
  const renderedSource = markdown.replace(/<!-- help-sync:source \{.+\} -->/u, "");
  const html = renderHelpMarkdown(renderedSource, sourceName);
  return Object.freeze({
    html,
    id: record.id,
    pagePath: pageFile(record),
    routeId: record.routeId,
    scope: record.scope,
    text: plainText(html),
    title,
  });
}

export function parseHelpPage(source, record, sourceName) {
  const { body, frontmatter } = parseFrontmatter(source, sourceName);
  const expectedPath = pageFile(record);

  for (const heading of REQUIRED_HEADINGS) {
    if (!body.includes(heading)) throw new Error(`${sourceName}: missing required section ${heading}.`);
  }
  if (frontmatter.id !== record.id) throw new Error(`${sourceName}: topic ID does not match navigation.`);
  if (frontmatter.route !== (record.routeId || "none")) throw new Error(`${sourceName}: route does not match navigation.`);
  if (frontmatter.scope !== record.scope) throw new Error(`${sourceName}: scope does not match navigation.`);
  if (!frontmatter.title) throw new Error(`${sourceName}: title is required.`);

  const provenance = sourceMarker(body, sourceName);
  if (
    provenance.topic_id !== record.id
    || provenance.route_id !== (record.routeId || "none")
    || provenance.page_path !== expectedPath
  ) {
    throw new Error(`${sourceName}: Help sync provenance does not match navigation.`);
  }

  const renderedSource = body.replace(/<!-- help-sync:source \{.+\} -->/u, "");
  return documentFromMarkdown(renderedSource, record, sourceName, frontmatter.title);
}

async function markdownFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map(async (entry) => {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory() && entry.name.startsWith("_")) return [];
    if (entry.isDirectory()) return markdownFiles(path);
    return entry.isFile() && entry.name.endsWith(".md") ? [path] : [];
  }));
  return nested.flat();
}

function splitLegacyBody(body) {
  const currentView = body.match(/^##[^\n]*\{#current-view\}[^\n]*\n?/mu);
  const source = body.trim();
  if (!currentView) return Object.freeze({ howToUse: source, safety: "" });

  const afterCurrentView = source.slice((currentView.index || 0) + currentView[0].length).trim();
  const nextSection = afterCurrentView.search(/^##\s+/mu);
  const demote = (value) => value.trim().replace(/^##\s+/gmu, "### ");
  if (nextSection < 0) return Object.freeze({ howToUse: demote(afterCurrentView), safety: "" });
  return Object.freeze({
    howToUse: demote(afterCurrentView.slice(0, nextSection)),
    safety: demote(afterCurrentView.slice(nextSection)),
  });
}

function localizedLegacyMarkdown(source, record, language, sourceName) {
  const copy = LOCALIZED_TEMPLATE_COPY[language];
  if (!copy) throw new Error(`Unsupported Help locale: ${language}`);
  const { body, frontmatter } = parseFrontmatter(source, sourceName);
  const sections = splitLegacyBody(body.replace(/<!-- help-sync:source \{.+\} -->/u, ""));
  const screenshot = body.match(/<!-- help-sync:source \{.+?"screenshot":"([^"]+)".+?\} -->/u)?.[1]
    || `help/assets/en/${record.id.replaceAll(".", "-")}.png`;
  const provenance = JSON.stringify({
    localized_source: sourceName,
    page_path: pageFile(record),
    route_id: record.routeId || "none",
    source_locale: "en",
    topic_id: record.id,
  });
  return Object.freeze({
    markdown: [
      `# ${frontmatter.title}`,
      "",
      "[TOC]",
      "",
      `## ${copy.appView} {#current-view}`,
      "",
      `![${frontmatter.title} application view](${screenshot})`,
      "",
      copy.imageNote,
      "",
      `## ${copy.overview}`,
      "",
      frontmatter.summary || "",
      "",
      `## ${copy.howToUse}`,
      "",
      sections.howToUse,
      "",
      `## ${copy.terms}`,
      "",
      `| ${copy.term} | ${copy.explanation} |`,
      "| --- | --- |",
      `| ${copy.review} | ${copy.reviewNote} |`,
      "",
      `## ${copy.safety}`,
      "",
      sections.safety || copy.reviewNote,
      "",
      `<!-- help-sync:source ${provenance} -->`,
      "",
    ].join("\n"),
    title: frontmatter.title,
  });
}

async function loadLegacyLocalizedHelp(root, language, records) {
  const localeRoot = resolve(root, "help", language);
  const expectedIds = new Set(records.map(({ id }) => id));
  const sourcesById = new Map();

  for (const path of await markdownFiles(localeRoot)) {
    const source = await readFile(path, "utf8");
    const id = source.match(/^id: ([^\n]+)$/mu)?.[1];
    if (!id || !expectedIds.has(id)) continue;
    const { frontmatter } = parseFrontmatter(source, path);
    sourcesById.set(frontmatter.id, [...(sourcesById.get(frontmatter.id) || []), Object.freeze({ path, source })]);
  }

  return Object.freeze(await Promise.all(records.map(async (record) => {
    const candidates = sourcesById.get(record.id) || [];
    if (!candidates.length) {
      const fallbackPath = resolve(root, "help", "en", pageFile(record));
      const source = await readFile(fallbackPath, "utf8");
      const sourceName = relative(root, fallbackPath);
      const localized = localizedLegacyMarkdown(source, record, language, sourceName);
      return documentFromMarkdown(localized.markdown, record, sourceName, localized.title);
    }
    if (new Set(candidates.map(({ source }) => source)).size !== 1) {
      throw new Error(`${language}: conflicting localized Help sources for ${record.id}.`);
    }
    const candidate = candidates[0];
    const sourceName = relative(root, candidate.path);
    const localized = localizedLegacyMarkdown(candidate.source, record, language, sourceName);
    return documentFromMarkdown(localized.markdown, record, sourceName, localized.title);
  })));
}

export async function loadNavigationHelp(root, language = "en") {
  const records = helpRecords();
  if (language !== "en") {
    return Object.freeze({
      documents: await loadLegacyLocalizedHelp(root, language, records),
      records,
    });
  }
  const documents = await Promise.all(records.map(async (record) => {
    const path = resolve(root, "help", "en", pageFile(record));
    const source = await readFile(path, "utf8");
    return parseHelpPage(source, record, path);
  }));
  return Object.freeze({ documents: Object.freeze(documents), records });
}

export async function compileNavigationHelp(root) {
  const { documents, records } = await loadNavigationHelp(root, "en");
  const catalogues = {
    en: Object.fromEntries(documents.map((document) => [document.id, document])),
  };
  const localizedCatalogues = await Promise.all(SUPPORTED_LOCALES
    .filter((language) => language !== "en")
    .map(async (language) => {
      const localized = await loadNavigationHelp(root, language);
      return [language, Object.fromEntries(localized.documents.map((document) => [document.id, document]))];
    }));
  Object.assign(catalogues, Object.fromEntries(localizedCatalogues));
  const payload = {
    catalogues,
    locales: SUPPORTED_LOCALES,
    records,
    version: 3,
  };
  return `"use strict";\n\n((root) => {\n  root.Z00ZHelpCatalog = Object.freeze(${JSON.stringify(payload, null, 2)});\n})(typeof window === "undefined" ? globalThis : window);\n`;
}
