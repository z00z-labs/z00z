import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { serializeHelpTopics } from "./help-topics.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const contractPath = resolve(demoRoot, "scripts/port/contracts.js");
const outputPath = resolve(demoRoot, "help/topics.yaml");
const sandbox = { URLSearchParams };
sandbox.globalThis = sandbox;
vm.runInNewContext(await readFile(contractPath, "utf8"), sandbox, { filename: contractPath });
await writeFile(outputPath, serializeHelpTopics(sandbox.Z00ZDemo.PORT_CONTRACT), "utf8");
console.log(`Generated Help topic map: ${outputPath}`);
