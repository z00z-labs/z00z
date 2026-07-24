"use strict";

((root) => {
  const catalogue = root.Z00ZHelpCatalog;
  if (!catalogue?.topics || !catalogue?.catalogues) {
    throw new Error("Z00Z Help catalogue must load before the Help registry.");
  }

  const topicsById = new Map(catalogue.topics.map((topic) => [topic.id, topic]));

  function matchesState(topic, state) {
    if (topic.match.global === "true") return false;
    return Object.entries(topic.match).every(([key, value]) => String(state?.[key] ?? "") === value);
  }

  function resolveTopicId(state, explicitTopicId = "") {
    if (explicitTopicId && topicsById.has(explicitTopicId)) return explicitTopicId;
    return catalogue.topics.find((topic) => topic.scope === "context" && matchesState(topic, state))?.id || "";
  }

  function resolveDocument(language, topicId) {
    const selectedLanguage = catalogue.locales.includes(language) ? language : "en";
    return catalogue.catalogues[selectedLanguage]?.[topicId]
      || catalogue.catalogues.en?.[topicId]
      || null;
  }

  function globalTopic() {
    return "app";
  }

  root.Z00ZHelpRegistry = Object.freeze({
    globalTopic,
    resolveTopicId,
    resolveDocument,
    hasTopic: (topicId) => topicsById.has(topicId)
  });
})(typeof window === "undefined" ? globalThis : window);
