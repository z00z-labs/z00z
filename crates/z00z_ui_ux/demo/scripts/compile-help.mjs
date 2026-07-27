import { writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { compileNavigationHelp } from "./help/navigation-help.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");

export async function compileHelp(root = demoRoot) {
  return compileNavigationHelp(root);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await writeFile(resolve(demoRoot, "scripts/generated/help-catalog.js"), await compileHelp(demoRoot), "utf8");
  console.log("Compiled navigation-derived English Help catalogue.");
}
