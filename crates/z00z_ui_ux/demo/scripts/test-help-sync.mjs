import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import {
  assertHelpSynchronized,
  recordReviewedHelpState,
  synchronizeHelp
} from "./sync-help.mjs";

const fixtureRoot = await mkdtemp(resolve(tmpdir(), "z00z-help-sync-"));
const helpDocument = (title, sections) => `---
id: app
title: ${title}
summary: Local Help summary.
scope: global
---
${sections}
`;

try {
  await mkdir(resolve(fixtureRoot, "scripts/port"), { recursive: true });
  await mkdir(resolve(fixtureRoot, "help/en"), { recursive: true });
  await mkdir(resolve(fixtureRoot, "help/de"), { recursive: true });
  await writeFile(
    resolve(fixtureRoot, "scripts/port/locale-registry.js"),
    '"use strict"; window.Z00ZLocaleRegistry = Object.freeze([{ id: "en" }, { id: "de" }]);\n',
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/topics.yaml"),
    "version: 1\ntopics:\n  - id: app\n    file: app\n    scope: global\n    match: global\n",
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/en/app.md"),
    helpDocument("Application help", "## Existing section\n- Existing guidance."),
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/de/app.md"),
    helpDocument("Anwendungshilfe", "## Bestehender Abschnitt\n- Bestehender Hinweis."),
    "utf8"
  );
  await recordReviewedHelpState(fixtureRoot);
  await assertHelpSynchronized(fixtureRoot);

  await writeFile(
    resolve(fixtureRoot, "help/en/app.md"),
    helpDocument("Application help", "## Existing section\n- Existing guidance.\n\n## New section\n- New guidance."),
    "utf8"
  );
  await assert.rejects(
    synchronizeHelp(fixtureRoot, { translatorCommand: "" }),
    /English Help changed for app/
  );

  const translatorPath = resolve(fixtureRoot, "translate.mjs");
  await writeFile(translatorPath, `#!/usr/bin/env node
let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { input += chunk; });
process.stdin.on("end", () => {
  const request = JSON.parse(input);
  process.stdout.write(JSON.stringify(Object.fromEntries(
    Object.entries(request.messages).map(([key, value]) => [key, "[de] " + value])
  )));
});
`, "utf8");
  await chmod(translatorPath, 0o755);

  const changed = await synchronizeHelp(fixtureRoot, { translatorCommand: translatorPath });
  assert.deepEqual([...changed], ["app"]);
  await assertHelpSynchronized(fixtureRoot);
  const german = await readFile(resolve(fixtureRoot, "help/de/app.md"), "utf8");
  assert.match(german, /## \[de\] New section/);
  assert.match(german, /- \[de\] New guidance\./);
  console.log("Help hash synchronization test passed.");
} finally {
  await rm(fixtureRoot, { recursive: true, force: true });
}
