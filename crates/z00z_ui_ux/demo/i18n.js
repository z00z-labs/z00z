"use strict";

(() => {
  const fallbackLanguage = "en";
  const catalogues = new Map();
  const localeRegistry = window.Z00ZLocaleRegistry;
  if (!localeRegistry?.length) throw new Error("Z00Z locale registry must load before i18n.");
  const languageMeta = new Map(localeRegistry.map(({ id, locale, nativeName, direction }) => [
    id,
    Object.freeze({ locale, nativeName, direction })
  ]));

  const pluralForms = new Set(["zero", "one", "two", "few", "many", "other"]);

  function flatten(source, prefix = "", result = {}) {
    Object.entries(source).forEach(([key, value]) => {
      const path = prefix ? prefix + "." + key : key;
      const isPluralMessage = value && typeof value === "object" && Object.keys(value).every((entry) => pluralForms.has(entry));
      if (value && typeof value === "object" && !Array.isArray(value) && !isPluralMessage) flatten(value, path, result);
      else result[path] = value;
    });
    return result;
  }

  function registerLocale(language, messages) {
    if (!languageMeta.has(language)) throw new Error("Unsupported UI language: " + language);
    catalogues.set(language, flatten(messages));
  }

  function resolveLanguage(language) {
    return languageMeta.has(language) ? language : fallbackLanguage;
  }

  function resolveLocale(language, locale) {
    if (locale && Intl.NumberFormat.supportedLocalesOf([locale]).length) return locale;
    return languageMeta.get(resolveLanguage(language)).locale;
  }

  function messageFor(language, key) {
    const selected = catalogues.get(resolveLanguage(language));
    const fallback = catalogues.get(fallbackLanguage);
    return selected?.[key] ?? fallback?.[key] ?? "[" + key + "]";
  }

  function interpolate(template, values = {}) {
    return String(template).replace(/\{(\w+)\}/g, (_, key) => String(values[key] ?? "{" + key + "}"));
  }

  function translate(language, key, values = {}) {
    const message = messageFor(language, key);
    if (message && typeof message === "object") {
      const count = Number(values.count ?? 0);
      const plural = new Intl.PluralRules(resolveLocale(language)).select(count);
      return interpolate(message[plural] ?? message.other ?? "", { ...values, count });
    }
    return interpolate(message, values);
  }

  function formatNumber(value, language, locale, options = {}) {
    return new Intl.NumberFormat(resolveLocale(language, locale), options).format(value);
  }

  function formatDateTime(value, language, locale, timeZone, options = {}) {
    return new Intl.DateTimeFormat(resolveLocale(language, locale), {
      dateStyle: "medium",
      timeStyle: "short",
      timeZone,
      ...options
    }).format(value);
  }

  function formatBitrate(bitsPerSecond, language, locale) {
    const value = Number(bitsPerSecond);
    const [amount, unitKey] = value >= 1_000_000
      ? [value / 1_000_000, "units.megabitPerSecond"]
      : value >= 1_000
        ? [value / 1_000, "units.kilobitPerSecond"]
        : [value, "units.bitPerSecond"];
    return translate(language, unitKey, {
      value: formatNumber(amount, language, locale, { maximumFractionDigits: 1 })
    });
  }

  function auditCatalogues() {
    const source = catalogues.get(fallbackLanguage) ?? {};
    const sourceKeys = Object.keys(source).sort();
    return [...languageMeta.keys()].map((language) => {
      const messages = catalogues.get(language) ?? {};
      const missing = sourceKeys.filter((key) => !(key in messages));
      const extra = Object.keys(messages).filter((key) => !(key in source)).sort();
      return { language, missing, extra, ready: missing.length === 0 && extra.length === 0 };
    });
  }

  function catalogue(language) {
    return { ...(catalogues.get(resolveLanguage(language)) ?? {}) };
  }

  window.Z00ZI18n = Object.freeze({
    fallbackLanguage,
    registerLocale,
    resolveLanguage,
    resolveLocale,
    translate,
    formatNumber,
    formatDateTime,
    formatBitrate,
    auditCatalogues,
    catalogue,
    languages: () => [...languageMeta.entries()].map(([id, meta]) => ({ id, ...meta }))
  });
})();
