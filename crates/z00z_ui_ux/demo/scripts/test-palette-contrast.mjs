import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const colors = await readFile(resolve(demoRoot, "styles/colors.css"), "utf8");
const help = await readFile(resolve(demoRoot, "styles/help.css"), "utf8");

const corporateSource = Object.freeze({
  "base-100": "#ffffff",
  "base-200": "#e8e8e8",
  "base-300": "#d1d1d1",
  "base-content": "#181a2a",
  primary: "#0082c4",
  secondary: "#61738d",
  accent: "#009588",
  success: "#00a242",
  warning: "#f7c800",
  error: "#ff6266"
});

for (const [name, value] of Object.entries(corporateSource)) {
  assert.ok(
    colors.includes(`--lut-z00z-corporate-${name}: ${value};`),
    `Corporate source token ${name} must remain traceable in colors.css`
  );
}

const corporateReadingSource = Object.freeze({
  "content-82": "rgb(24 26 42 / 82%)",
  "content-78": "rgb(24 26 42 / 78%)",
  "content-72": "rgb(24 26 42 / 72%)",
  "content-60": "rgb(24 26 42 / 60%)",
  "table-head": "#eeeeee",
  "table-row-alt": "#f3f4f6",
  "table-code-bg": "#f5f6f7",
  "table-code-border": "#d9dce1",
  "table-code-fg": "#1f2937"
});

for (const [name, value] of Object.entries(corporateReadingSource)) {
  assert.ok(
    colors.includes(`--lut-z00z-corporate-${name}: ${value};`),
    `Corporate reading token ${name} must remain traceable in colors.css`
  );
}

function luminance(hex) {
  const channels = [1, 3, 5].map((index) => Number.parseInt(hex.slice(index, index + 2), 16) / 255);
  const linear = channels.map((channel) => channel <= 0.04045
    ? channel / 12.92
    : ((channel + 0.055) / 1.055) ** 2.4);
  return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
}

function contrast(first, second) {
  const [light, dark] = [luminance(first), luminance(second)].sort((a, b) => b - a);
  return (light + 0.05) / (dark + 0.05);
}

const pairs = Object.freeze([
  ["Default primary text", "#f5f7f8", "#101d29", 4.5],
  ["Default primary control", "#1e1704", "#fca311", 4.5],
  ["Default focus", "#7ccbff", "#081019", 3],
  ["Default table header", "#f5f7f8", "#283746", 4.5],
  ["Default table canvas row", "#a9b6c2", "#081019", 4.5],
  ["Default table raised row", "#a9b6c2", "#162635", 4.5],
  ["Corporate primary text", "#181a2a", "#ffffff", 4.5],
  ["Corporate secondary text", "#4b4c59", "#ffffff", 4.5],
  ["Corporate tertiary text", "#74767f", "#ffffff", 4.5],
  ["Corporate primary control", "#ffffff", "#006da3", 4.5],
  ["Corporate success text", "#007c35", "#ffffff", 4.5],
  ["Corporate warning text", "#8a6400", "#ffffff", 4.5],
  ["Corporate error text", "#b23b45", "#ffffff", 4.5],
  ["Corporate focus", "#006da3", "#ffffff", 3]
]);

for (const [label, foreground, background, minimum] of pairs) {
  const ratio = contrast(foreground, background);
  assert.ok(ratio >= minimum, `${label} contrast ${ratio.toFixed(2)}:1 is below ${minimum}:1`);
}

for (const requiredSemanticMapping of [
  "--help-table-header-bg: color-mix(in srgb, var(--bg-raised) 88%, var(--text-secondary) 12%);",
  "--help-table-row-bg: var(--bg-canvas);",
  "--help-table-row-alt-bg: var(--bg-raised);",
  "--help-table-border: var(--border);",
  "--text-secondary: var(--lut-z00z-corporate-content-78);",
  "--text-tertiary: var(--lut-z00z-corporate-content-60);",
  "--brand: var(--lut-z00z-corporate-primary-contrast);",
  "--brand-strong: var(--lut-z00z-corporate-primary-contrast);",
  "--button-primary-bg: var(--lut-z00z-corporate-primary-contrast);",
  "--button-primary-bg-strong: var(--lut-z00z-corporate-primary-contrast);",
  "--success: var(--lut-z00z-corporate-success-contrast);",
  "--warning: var(--lut-z00z-corporate-warning-contrast);",
  "--danger: var(--lut-z00z-corporate-error-contrast);"
]) {
  assert.ok(colors.includes(requiredSemanticMapping), `${requiredSemanticMapping} must remain an accessible Corporate semantic mapping`);
}

for (const requiredHelpMapping of [
  "color: var(--lut-z00z-corporate-content-82);",
  "background: var(--help-table-header-bg);",
  "background: var(--help-table-row-bg);",
  "background: var(--help-table-row-alt-bg);"
]) {
  assert.ok(help.includes(requiredHelpMapping), `${requiredHelpMapping} must remain in the Corporate Help reading surface`);
}

for (const requiredCorporateTableMapping of [
  "--help-table-header-bg: var(--lut-z00z-corporate-table-head);",
  "--help-table-row-bg: var(--lut-z00z-corporate-base-100);",
  "--help-table-row-alt-bg: var(--lut-z00z-corporate-table-row-alt);"
]) {
  assert.ok(colors.includes(requiredCorporateTableMapping), `${requiredCorporateTableMapping} must remain in the Corporate palette mapping`);
}

console.log("Palette contrast checks passed.");
