"use strict";

((root) => {
  const registry = [
    { id: "en", locale: "en-US", nativeName: "English", direction: "ltr", catalogue: "locales/en.js" },
    { id: "ru", locale: "ru-RU", nativeName: "Русский", direction: "ltr", catalogue: "locales/ru.js" },
    { id: "fr", locale: "fr-FR", nativeName: "Français", direction: "ltr", catalogue: "locales/fr.js" },
    { id: "de", locale: "de-DE", nativeName: "Deutsch", direction: "ltr", catalogue: "locales/de.js" },
    { id: "es", locale: "es-ES", nativeName: "Español", direction: "ltr", catalogue: "locales/es.js" },
    { id: "pt", locale: "pt-PT", nativeName: "Português", direction: "ltr", catalogue: "locales/pt.js" },
    { id: "ko", locale: "ko-KR", nativeName: "한국어", direction: "ltr", catalogue: "locales/ko.js" },
    { id: "tr", locale: "tr-TR", nativeName: "Türkçe", direction: "ltr", catalogue: "locales/tr.js" },
    { id: "ja", locale: "ja-JP", nativeName: "日本語", direction: "ltr", catalogue: "locales/ja.js" },
    { id: "zh-Hans", locale: "zh-CN", nativeName: "简体中文", direction: "ltr", catalogue: "locales/zh-Hans.js" }
  ].map((entry) => Object.freeze(entry));

  root.Z00ZLocaleRegistry = Object.freeze(registry);
})(typeof window === "undefined" ? globalThis : window);
