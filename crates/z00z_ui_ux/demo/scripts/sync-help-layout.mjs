import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import {
  parseHelpFolderMeta,
  parseHelpLandingPage
} from "./help-content-map.mjs";
import { loadHelpLocales } from "./help-source.mjs";

const SOURCE_LOCALE = "en";

function yamlString(value) {
  return `"${String(value).replaceAll("\\", "\\\\").replaceAll('"', '\\"')}"`;
}

function serializeMeta(source, localized) {
  return [
    `title: ${yamlString(localized.title)}`,
    ...(source.icon ? [`icon: ${source.icon}`] : []),
    "order:",
    ...source.order.map((entry) => `  - ${entry}`),
    ""
  ].join("\n");
}

function serializeLanding(source, localized) {
  return [
    "---",
    `title: ${yamlString(localized.title)}`,
    `description: ${yamlString(localized.description)}`,
    `difficulty: ${source.difficulty}`,
    `icon: ${source.icon}`,
    `toc: ${source.toc}`,
    "---",
    ""
  ].join("\n");
}

async function readOptional(path) {
  try {
    return await readFile(path, "utf8");
  } catch (error) {
    if (error.code === "ENOENT") return "";
    throw error;
  }
}

async function collectLayout(directory, root = directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const directories = [relative(root, directory).replaceAll("\\", "/")];
  const files = [];
  for (const entry of entries) {
    if (entry.name === "_drafts") continue;
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      const nested = await collectLayout(path, root);
      directories.push(...nested.directories);
      files.push(...nested.files);
      continue;
    }
    if (entry.isFile() && ["_meta.yaml", "index.md"].includes(entry.name)) {
      files.push(relative(root, path).replaceAll("\\", "/"));
    }
  }
  return { directories, files };
}

export async function synchronizeHelpLayout(root) {
  const locales = await loadHelpLocales(root);
  const sourceRoot = resolve(root, "help", SOURCE_LOCALE);
  const layout = await collectLayout(sourceRoot);
  const changedPaths = [];

  for (const locale of locales) {
    if (locale === SOURCE_LOCALE) continue;
    const localeRoot = resolve(root, "help", locale);
    for (const directory of layout.directories) {
      await mkdir(resolve(localeRoot, directory), { recursive: true });
    }
    for (const relativePath of layout.files) {
      const sourcePath = resolve(sourceRoot, relativePath);
      const outputPath = resolve(localeRoot, relativePath);
      const sourceText = await readFile(sourcePath, "utf8");
      const currentText = await readOptional(outputPath);
      let output;
      if (relativePath.endsWith("_meta.yaml")) {
        const source = parseHelpFolderMeta(sourceText, sourcePath);
        const localized = currentText
          ? parseHelpFolderMeta(currentText, outputPath)
          : source;
        output = serializeMeta(source, localized);
      } else {
        const source = parseHelpLandingPage(sourceText, sourcePath);
        const localized = currentText
          ? parseHelpLandingPage(currentText, outputPath)
          : source;
        output = serializeLanding(source, localized);
      }
      if (output === currentText) continue;
      await mkdir(dirname(outputPath), { recursive: true });
      await writeFile(outputPath, output, "utf8");
      changedPaths.push(relative(root, outputPath).replaceAll("\\", "/"));
    }
  }

  return Object.freeze(changedPaths);
}
