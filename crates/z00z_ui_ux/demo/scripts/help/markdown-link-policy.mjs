const EXTERNAL_PROTOCOLS = new Set(["http:", "https:"]);

export function isExternalLink(value) {
  const normalized = value.trim();
  if (normalized.startsWith("//")) return true;

  try {
    return EXTERNAL_PROTOCOLS.has(new URL(normalized).protocol);
  } catch {
    return false;
  }
}
