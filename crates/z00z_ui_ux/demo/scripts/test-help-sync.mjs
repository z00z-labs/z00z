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
  await mkdir(resolve(fixtureRoot, "help/en/app"), { recursive: true });
  await mkdir(resolve(fixtureRoot, "help/de/app"), { recursive: true });
  await writeFile(
    resolve(fixtureRoot, "scripts/port/locale-registry.js"),
    '"use strict"; window.Z00ZLocaleRegistry = Object.freeze([{ id: "en" }, { id: "de" }]);\n',
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/topics.yaml"),
    "version: 1\ntopics:\n  - id: app\n    group: app\n    file: app\n    scope: global\n    match: global\n",
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/en/app/app.md"),
    helpDocument("Application help", "## Existing section\n- Existing guidance."),
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/de/app/app.md"),
    helpDocument("Anwendungshilfe", "## Bestehender Abschnitt\n- Bestehender Hinweis."),
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/en/app/_meta.yaml"),
    'title: "Application"\nicon: question\norder:\n  - app\n',
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/de/app/_meta.yaml"),
    'title: "Anwendung"\nicon: wallet\norder:\n  - app\n',
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/en/app/index.md"),
    '---\ntitle: "Application"\ndescription: "Application Help"\ndifficulty: basic\nicon: mdi:alphabet-a-box-outline\ntoc: true\n---\n',
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/de/app/index.md"),
    '---\ntitle: "Anwendung"\ndescription: "Anwendungshilfe"\ndifficulty: basic\nicon: mdi:alphabet-a-box-outline\ntoc: true\n---\n',
    "utf8"
  );
  await recordReviewedHelpState(fixtureRoot);
  await assertHelpSynchronized(fixtureRoot);

  await writeFile(
    resolve(fixtureRoot, "help/topics.yaml"),
    "version: 1\ntopics:\n  - id: app\n    group: app\n    file: app\n    source: root\n    scope: global\n    match: global\n",
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/en/app.md"),
    helpDocument("Application help", "## Existing section\n- Existing guidance."),
    "utf8"
  );
  const relocated = await synchronizeHelp(fixtureRoot);
  assert.deepEqual([...relocated], ["app"]);
  assert.match(await readFile(resolve(fixtureRoot, "help/de/app.md"), "utf8"), /Application help/);
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

  const bundledChanged = await synchronizeHelp(fixtureRoot);
  assert.deepEqual([...bundledChanged], ["app"]);
  assert.equal(bundledChanged.fallbacks.length, 1);
  assert.deepEqual(
    [...bundledChanged.fallbacks[0].keys],
    [
      "sections.1.title",
      "sections.1.blocks.0.items.0"
    ]
  );
  await assertHelpSynchronized(fixtureRoot);
  const bundledGerman = await readFile(resolve(fixtureRoot, "help/de/app.md"), "utf8");
  const bundledGermanMeta = await readFile(resolve(fixtureRoot, "help/de/app/_meta.yaml"), "utf8");
  assert.match(bundledGerman, /## Existing section/);
  assert.match(bundledGerman, /## New section/);
  assert.match(bundledGerman, /- New guidance\./);
  assert.match(bundledGermanMeta, /title: "Anwendung"/);
  assert.match(bundledGermanMeta, /icon: question/);

  await writeFile(
    resolve(fixtureRoot, "help/en/app.md"),
    helpDocument("Application help", "## Existing section\n- Existing guidance.\n\n## New section\n- Revised guidance."),
    "utf8"
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
  assert.match(german, /- \[de\] Revised guidance\./);

  await writeFile(
    resolve(fixtureRoot, "help/topics.yaml"),
    "version: 1\ntopics:\n  - id: app\n    group: app\n    file: app\n    source: root\n    scope: global\n    match: global\n  - id: app.new\n    group: app\n    file: new\n    source: root\n    scope: article\n    match: article=new\n",
    "utf8"
  );
  await writeFile(
    resolve(fixtureRoot, "help/en/new.md"),
    helpDocument("New topic", "## New topic section\n- New topic guidance.").replace("id: app", "id: app.new").replace("scope: global", "scope: article"),
    "utf8"
  );
  const addedTopic = await synchronizeHelp(fixtureRoot, { translatorCommand: translatorPath });
  assert.deepEqual([...addedTopic], ["app.new"]);
  const addedGerman = await readFile(resolve(fixtureRoot, "help/de/new.md"), "utf8");
  assert.match(addedGerman, /title: \[de\] New topic/);
  await assertHelpSynchronized(fixtureRoot);
  console.log("Help hash synchronization test passed.");
} finally {
  await rm(fixtureRoot, { recursive: true, force: true });
}
