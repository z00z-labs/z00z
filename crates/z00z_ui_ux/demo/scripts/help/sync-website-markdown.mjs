import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const websiteRoot = resolve(scriptDirectory, "../../../../../../z00z-website");
const defaultSource = resolve(websiteRoot, "src/lib/content/markdown.ts");
const defaultPipeline = resolve(websiteRoot, "config/content-pipeline.yaml");
const targetPath = resolve(scriptDirectory, "website-markdown.ts");
const pipelineTargetPath = resolve(scriptDirectory, "website-markdown-pipeline.mjs");
const sourcePath = resolve(process.env.Z00Z_WEBSITE_MARKDOWN ?? defaultSource);
const pipelinePath = resolve(process.env.Z00Z_WEBSITE_CONTENT_PIPELINE ?? defaultPipeline);

const source = await readFile(sourcePath, "utf8");
const snapshot = source
  .replace('import { slugifyHeading } from "@/lib/content/docs-route-contract";', 'import { slugifyHeading } from "./markdown-route-contract.mjs";')
  .replace('import { isExternalLink } from "@/lib/content/link-policy";', 'import { isExternalLink } from "./markdown-link-policy.mjs";');

if (snapshot.includes("@/lib/content/")) {
  throw new Error("Website Markdown snapshot still contains an unresolved Website alias import.");
}

const upstreamHash = createHash("sha256").update(source).digest("hex");
await writeFile(targetPath, `// Generated from ${sourcePath} (sha256:${upstreamHash}).\n${snapshot}`, "utf8");
const pipelineSource = await readFile(pipelinePath, "utf8");
const markdownSection = pipelineSource.match(/^markdown:\n((?:  [a-z_]+: (?:true|false)\n?)+)/mu)?.[1];

if (!markdownSection) {
  throw new Error(`Unable to read the markdown section from ${pipelinePath}.`);
}

const markdown = Object.fromEntries(
  [...markdownSection.matchAll(/^  ([a-z_]+): (true|false)$/gmu)].map(([, key, value]) => [key, value === "true"]),
);
const pipelineHash = createHash("sha256").update(pipelineSource).digest("hex");
await writeFile(
  pipelineTargetPath,
  `// Generated from ${pipelinePath} (sha256:${pipelineHash}).\nexport const WEBSITE_MARKDOWN_PIPELINE = Object.freeze(${JSON.stringify(markdown, null, 2)});\n`,
  "utf8",
);
console.log(`Updated ${targetPath}`);
console.log(`Updated ${pipelineTargetPath}`);
