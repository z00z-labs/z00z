import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";

import { getContentPipelineConfig } from "../../../../../../z00z-website/src/lib/config/site.ts";
import { renderMarkdown } from "../../../../../../z00z-website/src/lib/content/markdown.ts";

const websiteRoot = process.cwd();
const contentRoot = resolve(websiteRoot, "content");
const pipeline = getContentPipelineConfig();

async function listMarkdownFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map(async (entry) => {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) return listMarkdownFiles(path);
    return entry.isFile() && entry.name.endsWith(".md") ? [path] : [];
  }));
  return nested.flat().sort();
}

function withoutFrontmatter(source) {
  return source.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/u, "");
}

const results = {};
for (const filePath of await listMarkdownFiles(contentRoot)) {
  const source = withoutFrontmatter(await readFile(filePath, "utf8"));
  const html = renderMarkdown(source, { filePath, pipeline });
  results[filePath.slice(websiteRoot.length + 1)] = createHash("sha256").update(html).digest("hex");
}

process.stdout.write(`${JSON.stringify(results)}\n`);
