import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { loadHelpLocales, loadHelpSource } from "./help-source.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
async function main() {
  const requestedId = process.argv[2];
  if (!requestedId) throw new Error("Usage: node scripts/scaffold-help.mjs <topic-id>");
  const { lut } = await loadHelpSource(demoRoot);
  const localeIds = await loadHelpLocales(demoRoot);
  const topic = lut.topics.find(({ id }) => id === requestedId);
  if (!topic) throw new Error(`Add ${requestedId} to help/topics.yaml before scaffolding it.`);

  for (const locale of localeIds) {
    const path = resolve(demoRoot, "help", locale, `${topic.file}.md`);
    try {
      await readFile(path, "utf8");
      console.log(`Kept existing ${path}`);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
      await mkdir(dirname(path), { recursive: true });
      await writeFile(path, `---\nid: ${topic.id}\ntitle: TODO [translate]\nsummary: TODO [translate]\nscope: ${topic.scope}\n---\n## TODO [translate] {#current-view}\n- TODO [translate]\n`, "utf8");
      console.log(`Created ${path}`);
    }
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
