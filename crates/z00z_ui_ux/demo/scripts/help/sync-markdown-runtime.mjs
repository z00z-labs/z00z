import { cp, copyFile, mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "../..");
const outputRoot = resolve(process.argv[2] || demoRoot, "scripts/vendor/markdown");
const nodeModules = resolve(demoRoot, "node_modules");

await mkdir(outputRoot, { recursive: true });
await copyFile(resolve(nodeModules, "mermaid/dist/mermaid.min.js"), resolve(outputRoot, "mermaid.min.js"));
await copyFile(
  resolve(nodeModules, "@panzoom/panzoom/dist/panzoom.min.js"),
  resolve(outputRoot, "panzoom.min.js"),
);
await copyFile(resolve(nodeModules, "katex/dist/katex.min.css"), resolve(outputRoot, "katex.min.css"));
await cp(resolve(nodeModules, "katex/dist/fonts"), resolve(outputRoot, "fonts"), { recursive: true, force: true });
await writeFile(
  resolve(outputRoot, "runtime.json"),
  `${JSON.stringify({ mermaid: "11.16.0", panzoom: "4.6.2", katex: "0.17.0" }, null, 2)}\n`,
  "utf8",
);
console.log(`Prepared static Markdown runtime assets in ${outputRoot}`);
