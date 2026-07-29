"use strict";

((root) => {
  const catalogue = root.Z00ZHelpCatalog;
  if (!catalogue?.records || !catalogue?.catalogues?.en || !catalogue?.navigations?.en) {
    throw new Error("Navigation-derived Help catalogue must load before the Help registry.");
  }

  const topicsById = new Map(catalogue.records.map((topic) => [topic.id, topic]));

  function resolveTopicId(state, explicitTopicId = "") {
    if (explicitTopicId && topicsById.has(explicitTopicId)) return explicitTopicId;
    if (state?.activeRoute) return catalogue.records.find((topic) => topic.routeId === state.activeRoute)?.id || "";
    if (state?.dialog) return catalogue.records.find((topic) => topic.dialog === state.dialog)?.id || "";
    return "";
  }

  function resolveDocument(language, topicId) {
    const resolvedLanguage = root.Z00ZI18n?.resolveLanguage(language) || "en";
    return catalogue.catalogues[resolvedLanguage]?.[topicId]
      || catalogue.catalogues.en[topicId]
      || null;
  }

  root.Z00ZHelpRegistry = Object.freeze({
    globalTopic: () => "app",
    hasTopic: (topicId) => topicsById.has(topicId),
    navigation: (language) => {
      const resolvedLanguage = root.Z00ZI18n?.resolveLanguage(language) || "en";
      return catalogue.navigations[resolvedLanguage] || catalogue.navigations.en;
    },
    resolveDocument,
    resolveTopicId,
    topic: (topicId) => topicsById.get(topicId) || null,
    topics: () => [...catalogue.records],
  });
})(typeof window === "undefined" ? globalThis : window);
