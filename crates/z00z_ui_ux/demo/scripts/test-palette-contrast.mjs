import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const colors = await readFile(resolve(demoRoot, "styles/colors.css"), "utf8");

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
  ["Corporate primary text", "#181a2a", "#ffffff", 4.5],
  ["Corporate secondary text", "#61738d", "#ffffff", 4.5],
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
  "--brand: var(--lut-z00z-corporate-primary-contrast);",
  "--success: var(--lut-z00z-corporate-success-contrast);",
  "--warning: var(--lut-z00z-corporate-warning-contrast);",
  "--danger: var(--lut-z00z-corporate-error-contrast);"
]) {
  assert.ok(colors.includes(requiredSemanticMapping), `${requiredSemanticMapping} must remain an accessible Corporate semantic mapping`);
}

console.log("Palette contrast checks passed.");
