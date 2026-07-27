import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { MARKDOWN_PIPELINE } from "./markdown-pipeline.mjs";
import { renderMarkdown } from "./website-markdown.ts";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const sourcePath = resolve(scriptDirectory, "website-markdown.ts");

export function renderHelpMarkdown(source, filePath) {
  if (!existsSync(sourcePath)) {
    throw new Error("Website Markdown snapshot is missing; run node scripts/help/sync-website-markdown.mjs.");
  }

  return renderMarkdown(source, { filePath, pipeline: MARKDOWN_PIPELINE });
}
