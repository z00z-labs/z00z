import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { MARKDOWN_PIPELINE } from "./help/markdown-pipeline.mjs";
import { renderHelpMarkdown } from "./help/markdown-renderer.mjs";
import { WEBSITE_MARKDOWN_PIPELINE } from "./help/website-markdown-pipeline.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const rendererPath = resolve(scriptDirectory, "help/website-markdown.ts");
const pipelineSnapshotPath = resolve(scriptDirectory, "help/website-markdown-pipeline.mjs");
const upstreamPath = resolve(scriptDirectory, "../../../../../../z00z-website/src/lib/content/markdown.ts");
const upstreamPipelinePath = resolve(scriptDirectory, "../../../../../../z00z-website/config/content-pipeline.yaml");
const fixturePath = "help/en/app/index.md";
const rendererSource = await readFile(rendererPath, "utf8");
const pipelineSnapshot = await readFile(pipelineSnapshotPath, "utf8");
const header = rendererSource.match(/^\/\/ Generated from .+ \(sha256:([a-f0-9]{64})\)\.\n/u);
const pipelineHeader = pipelineSnapshot.match(/^\/\/ Generated from .+ \(sha256:([a-f0-9]{64})\)\.\n/u);

function parseMarkdownPipeline(source) {
  const section = source.match(/^markdown:\n((?:  [a-z_]+: (?:true|false)\n?)+)/mu)?.[1];
  assert.ok(section, "Website content pipeline must include a Markdown boolean configuration section.");
  return Object.fromEntries(
    [...section.matchAll(/^  ([a-z_]+): (true|false)$/gmu)].map(([, key, value]) => [key, value === "true"]),
  );
}

assert.ok(header, "Website Markdown snapshot must contain its upstream SHA-256 header.");
assert.ok(pipelineHeader, "Website Markdown pipeline snapshot must contain its upstream SHA-256 header.");

if (existsSync(upstreamPath)) {
  const upstream = await readFile(upstreamPath, "utf8");
  const normalizedUpstream = upstream
    .replace('import { slugifyHeading } from "@/lib/content/docs-route-contract";', 'import { slugifyHeading } from "./markdown-route-contract.mjs";')
    .replace('import { isExternalLink } from "@/lib/content/link-policy";', 'import { isExternalLink } from "./markdown-link-policy.mjs";');
  assert.equal(header[1], createHash("sha256").update(upstream).digest("hex"));
  assert.equal(rendererSource.slice(header[0].length), normalizedUpstream);
}

if (existsSync(upstreamPipelinePath)) {
  const upstreamPipeline = await readFile(upstreamPipelinePath, "utf8");
  assert.equal(pipelineHeader[1], createHash("sha256").update(upstreamPipeline).digest("hex"));
  assert.deepEqual(WEBSITE_MARKDOWN_PIPELINE, parseMarkdownPipeline(upstreamPipeline));
}

for (const [name, enabled] of Object.entries(MARKDOWN_PIPELINE.markdown)) {
  assert.equal(typeof enabled, "boolean", `Website Markdown pipeline value ${name} must be explicit.`);
}
assert.deepEqual(MARKDOWN_PIPELINE.markdown, WEBSITE_MARKDOWN_PIPELINE);
assert.equal(MARKDOWN_PIPELINE.markdown.include, false);
assert.equal(MARKDOWN_PIPELINE.markdown.snippet, false);

const rendered = renderHelpMarkdown(`
# Extension matrix

[TOC]

*[HTML]: HyperText Markup Language

HTML

Visit https://z00z.io/website -- "Smart typography".

> [!NOTE]
> Alert text

::: center
Aligned text
:::

## Heading attributes {#heading-attrs .accent}

::: warning
Container warning
:::

Term
: Definition

{% youtube dQw4w9WgXcQ %}

![Sized image =120x80](./image.png)

Footnote reference[^note].

[^note]: Footnote text.

++inserted++ ==marked== !!secret!! H~2~O x^2^ $x^2$ ==brand: stylized text==

::: tabs
@tab First #first
First panel
@tab Second #second
Second panel
:::

- [x] Checked task

\`\`\`mermaid
flowchart LR
  A --> B
\`\`\`

+++ Details
Hidden detail
+++

<script>alert("blocked")</script>

Use [Z00Z](https://z00z.io) and [local Help](./send.md).

| Control | Purpose |
| --- | --- |
| Send | Opens a payment draft |

\`\`\`js
const amount = 42;
\`\`\`
`, fixturePath);

assert.match(rendered, /<h1 id="extension-matrix">Extension matrix<\/h1>/u);
assert.match(rendered, /<nav class="table-of-contents">/u);
assert.match(rendered, /<abbr title="HyperText Markup Language">HTML<\/abbr>/u);
assert.match(rendered, /href="https:\/\/z00z\.io\/website" target="_blank" rel="noopener noreferrer"/u);
assert.match(rendered, /– “Smart typography”\./u);
assert.match(rendered, /class="markdown-alert markdown-alert-note"/u);
assert.match(rendered, /style="text-align:center"/u);
assert.match(rendered, /<h2 id="heading-attrs" class="accent">Heading attributes<\/h2>/u);
assert.match(rendered, /<div class="warning">/u);
assert.match(rendered, /<dl>[\s\S]*<dt>Term<\/dt>[\s\S]*<dd>Definition<\/dd>/u);
assert.match(rendered, /<div class="video-embed"><iframe src="https:\/\/www\.youtube\.com\/embed\/dQw4w9WgXcQ"/u);
assert.match(rendered, /<figure><img src="\.\/image\.png" alt="Sized image" width="120" height="80" tabindex="0" loading="lazy">/u);
assert.match(rendered, /class="footnote-ref"/u);
assert.match(rendered, /<ins>inserted<\/ins>/u);
assert.match(rendered, /<mark>marked<\/mark>/u);
assert.match(rendered, /<span class="spoiler" tabindex="-1">secret<\/span>/u);
assert.match(rendered, /H<sub>2<\/sub>O x<sup>2<\/sup>/u);
assert.match(rendered, /class="katex"/u);
assert.match(rendered, /class="token-inline-accent"/u);
assert.match(rendered, /class="tabs-block"[\s\S]*data-tab-target="first"[\s\S]*data-tab-target="second"/u);
assert.match(rendered, /class="task-list-item-checkbox"/u);
assert.match(rendered, /class="mermaid" data-mermaid-definition="flowchart%20LR/u);
assert.match(rendered, /<details>[\s\S]*<summary>/u);
assert.match(rendered, /<table>/u);
assert.match(rendered, /<code class="hljs language-js">/u);
assert.match(rendered, /href="https:\/\/z00z\.io" target="_blank" rel="noopener noreferrer"/u);
assert.match(rendered, /href="\.\/send\.md"/u);
assert.doesNotMatch(rendered, /<script>|javascript:/iu);

console.log("Website Markdown parser and extension parity contract passed.");
