---
name: 'GSD-Upgrade-Version'
agent: agent
description: 'Upgrade GSD from the upstream source or a specific tag while preserving local GSD file modifications and letting the user choose what to restore.'
argument-hint: 'target_ref=main [source_repo=https://github.com/open-gsd/gsd-core] [runtime=--copilot] [live_root=.github/gsd-core] [legacy_root=.github/get-shit-done]'
---

# GSD Upgrade Version

## Mission

Upgrade GSD from the upstream repository at a chosen ref, while preserving local modifications inside GSD-managed files and restoring the selected changes into the new version.

Default to the new `gsd-core` runtime root, but support legacy repositories that still run from `.github/get-shit-done` by migrating them into `.github/gsd-core` during the same upgrade.

The default source is the upstream `main` branch, but the prompt must also support a concrete tag or version ref.

## Inputs

- `target_ref`: `${input:target_ref:main}`
- `source_repo`: `${input:source_repo:https://github.com/open-gsd/gsd-core}`
- `runtime`: `${input:runtime:--copilot}`
- `live_root`: `${input:live_root:.github/gsd-core}`
- `legacy_root`: `${input:legacy_root:.github/get-shit-done}`

Interpret `target_ref` as either:

- the upstream moving branch, such as `main`; or
- a concrete release tag, such as `v1.38.3`.

## Source And Layout Rules

Normalize the upstream repository tree to the local Copilot install layout before comparing or restoring files.

- do not assume the upstream source tree paths are identical to the installed `./.github` paths;
- treat the local Copilot install as the authoritative live layout under `./.github`;
- treat `live_root` as the target runtime root and, if it does not exist yet, treat `legacy_root` as the pre-migration source root that must be mapped into `live_root`;
- preserve the repository's current `.github/copilot-instructions.md` instead of replacing it with the upstream template;
- map upstream `gsd-core/`, `skills/`, `agents/`, `scripts/`, and `hooks/` content into the installed Copilot layout before deciding whether a file is missing, renamed, or drifted;
- map legacy local `get-shit-done/` paths into the new `gsd-core/` destinations before comparing or restoring behavior;
- do not classify packaged omissions such as missing upstream-only assets, sdk, or tests as install failures by default.

## Primary Directive

Use tagged source or manual install logic for the upgrade path.

Do not use the generic npm latest path when performing this upgrade.

Run the upgrade in the Copilot runtime mode using the provided runtime flag.

## Mandatory Preservation Rules

Before the install begins:

- treat the upgrade as a clean reinstall of GSD-managed files;
- assume local modifications inside GSD files will be lost unless they are backed up first;
- create `.github/gsd-local-patches/` and store the user's local GSD-file modifications there before replacement;
- create any backup copies for existing `txt`, `md`, `json`, `yaml`, `yml`, and `csv` files that will be fully replaced during the reinstall inside `.github/gsd-local-patches/` instead of next to the active live root under `.github`;
- preserve enough metadata to compare backup content against the new version after the upgrade;
- record upgrade metadata such as `backed_up_at`, `from_version`, and the exact preserved file list so restore decisions can be audited later;
- show the diff between the backed-up local files and the new upgraded files;
- preserve the current `.github/copilot-instructions.md` exactly as a repo-specific local anchor;
- treat the target scenario as `upstream core + selective restore of local anchors`, not as a blank upstream reset;
- treat restore as a surgical behavior carry-forward, not as a blanket rollback of upgraded files;
- reapply preserved behavior into the new file structure at the narrowest valid insertion points;
- prefer block-level or anchor-level merges over full-file replacement whenever the upgraded file shape changed;
- let the user choose what should be restored into the new version.

If the installer does not create `.github/gsd-local-patches/` automatically, create it manually before proceeding.

If the installer reports a detected local patch set, treat that detected set as the authoritative restore-candidate baseline, then cross-check it against the mandatory anchors below so required local behavior is not silently missed.

## Mandatory Local Restore Anchors

The following local customizations must always be included in the restore review and must not be silently dropped:

- the local `gsd-add-tests` changes;
- the local `gsd-code-review` auto-fix flow at the end of every completed plan;
- the local phase-level `gsd-code-review` auto-fix flow in `execute-phase.md`;
- the repo-local `gsd-graphify` flow;
- the current repository-specific `.github/copilot-instructions.md`;
- the compatibility alias `/gsd-add-phase` when that command surface exists or is explicitly requested.

Treat these files as mandatory anchors when building the carry-forward set:

- `.github/skills/gsd-add-tests/SKILL.md`
- `.github/gsd-core/workflows/execute-plan.md`
- `.github/gsd-core/workflows/execute-phase.md`
- `.github/skills/gsd-graphify/SKILL.md`
- `.github/copilot-instructions.md`
- `.github/skills/gsd-add-phase/SKILL.md` when present in the repo or selected for restore;
- when upgrading from a legacy live root, map these legacy source equivalents into the new destinations before diffing:
  - `.github/get-shit-done/workflows/execute-plan.md` -> `.github/gsd-core/workflows/execute-plan.md`
  - `.github/get-shit-done/workflows/execute-phase.md` -> `.github/gsd-core/workflows/execute-phase.md`
  - `.github/get-shit-done/workflows/add-tests.md` -> `.github/gsd-core/workflows/add-tests.md`
- `execute-phase.md` still contains the `code_review_gate` anchor and automatically invokes `gsd-code-review-fix` for non-clean review results while remaining non-blocking for execution flow;
- `execute-plan.md` still contains the `plan_review_gate` anchor and automatically invokes `/gsd-code-review-fix ${PHASE} --all --auto` for non-clean plan review results while remaining non-blocking for execution flow;
- `.github/skills/gsd-add-tests/SKILL.md` still preserves the intended local behavior from the saved patch set, including phase-local test-spec outputs and commit-on-demand behavior;
- `.github/skills/gsd-graphify/SKILL.md` still preserves the repo-local `.github/gsd-core` invocation flow;
- `.github/copilot-instructions.md` is still the repository's local file, not the upstream template.

Also inspect related support files when needed for preserving those behaviors:

- `.github/gsd-core/workflows/code-review.md`
- `.github/gsd-core/workflows/code-review-fix.md`
- `.github/gsd-core/bin/lib/config.cjs`
- `.github/gsd-core/bin/lib/graphify.cjs`

## Required Upgrade Workflow

1. Inspect the current live GSD tree under `live_root`. If it does not exist, inspect `legacy_root` and classify the run as a root-migration upgrade into `live_root`.
2. Resolve `target_ref` explicitly and verify that the requested branch or tag exists before replacing anything.
3. Map the chosen upstream source into the local Copilot install layout and compare like-for-like installed paths, including legacy-path-to-new-path mapping when the source root is `legacy_root`.
4. Detect overlapping files that differ.
5. Back up local modifications into `.github/gsd-local-patches/` before reinstalling anything.
6. Preserve a baseline or pristine comparison copy when possible so three-way restore remains possible.
7. Perform the upgrade using source or tagged manual install logic with the runtime flag so the live runtime lands under `live_root` (`.github/gsd-core` by default).
8. Verify the live installed version directly from `./.github/gsd-core/VERSION` after install instead of relying only on installer output. When migrating from `legacy_root`, also record the pre-upgrade legacy version from `legacy_root/VERSION` when available.
9. After the new version is installed, diff the backup files against the upgraded files.
10. Present the user with the restore candidates.
11. Explicitly call out the `gsd-add-tests` local changes, the plan-end auto-fix behavior, the phase-level auto-fix behavior, the repo-local graphify flow, the preserved `.github/copilot-instructions.md`, and the compatibility alias when present as mandatory review items.
12. For each selected restore, map the old behavior into the new file's current structure and insert it at the smallest valid anchor points.
13. Only use full-file replacement when the upgraded file shape is materially unchanged or when the user explicitly requests a full rollback for that file.
14. Let the user choose what should be restored.
15. Reapply the selected patches.
16. Re-check that the restored version still contains the intended local behavior without removing unrelated upstream additions.
17. Surface any installer warnings that materially affect Copilot usage, including unreplaced `.claude` path references and any remaining active `.github/get-shit-done` dependencies outside explicit legacy archives/backups.

## Required User Choice Step

Do not blindly restore all local patches.

After the upgrade, present a restore review that includes:

- file path;
- brief description of the local behavior or content at risk;
- diff between backup and new version;
- recommended merge mode: surgical merge or full-file replace;
- whether the new upstream structure changed in a way that makes blanket restore unsafe;
- whether the item is a mandatory local anchor or an optional patch.

The user must be able to choose which changes to restore into the new version.

When this repository matches the known local-anchor profile, recommend restoring this set by default:

- `.github/copilot-instructions.md`
- `.github/skills/gsd-add-tests/SKILL.md`
- `.github/gsd-core/workflows/execute-plan.md` plan review gate
- `.github/gsd-core/workflows/execute-phase.md` code review gate
- `.github/skills/gsd-graphify/SKILL.md`
- `.github/skills/gsd-add-phase/SKILL.md` when the alias is installed or explicitly requested

## Backup Manifest Integrity Gate

Before any restore comparison or survival-count reporting:

- verify that each entry in `preserved_files.txt` has a physically present backup file under `.github/gsd-local-patches/<timestamp>/`;
- classify any missing backup payload as `BACKUP_MISSING` and report it separately from live-tree drift;
- compute restore/survival counts only from backup entries that physically exist;
- do not treat `BACKUP_MISSING` items as live regressions;
- do not finish silently when `BACKUP_MISSING` is non-zero: surface the count and file list in the final report.

## Verification Anchors

Verify these conditions directly in workspace files after the restore step:

- `gsd-add-tests` still exists as a repo-local workflow and skill surface;
- `/gsd-add-phase` compatibility alias still exists when it was part of the selected restore set;
- `execute-plan.md` still contains the plan-end review gate;
- `execute-plan.md` still includes the auto-fix invocation for non-clean review results;
- `execute-plan.md` still contains the concrete `plan_review_gate` anchor;
- `execute-plan.md` still contains the concrete `/gsd-code-review-fix ${PHASE} --all --auto` invocation;
- `execute-phase.md` still contains the concrete `code_review_gate` anchor;
- `execute-phase.md` still automatically invokes `Skill(skill="gsd-code-review-fix", args="${PHASE_NUMBER}")` for non-clean review results;
- `.github/skills/gsd-add-tests/SKILL.md` still matches the intended local behavior from the selected patch set, including phase-local test-spec outputs and commit-on-demand behavior;
- `.github/skills/gsd-graphify/SKILL.md` still preserves the repo-local `.github/gsd-core` graphify invocation flow;
- `.github/copilot-instructions.md` is still the repository's local file and was not replaced by the upstream template;
- `code-review.md` and any required supporting agent surface still preserve the planning-artifact allowance when that local behavior was part of the saved patch set;
- the live `./.github/gsd-core/VERSION` file matches the requested installed version when `target_ref` is a concrete release tag;
- no active GSD skill, agent, or workflow surface still depends on `.github/get-shit-done` except explicitly retained legacy archives/backups;
- the restored local behavior was inserted into the current upgraded structures instead of replacing unrelated upstream content wholesale;
- the preserved local changes are visible in the upgraded tree, not only in the backup folder.

## Output Expectations

When finished, provide a concise result that includes:

- source repository used;
- target ref used;
- confirmation that the upgrade used the Copilot runtime flag;
- whether `legacy_root` was used as the pre-migration baseline;
- the final live version read from `./.github/gsd-core/VERSION`;
- whether installer-detected local patches were used as the restore baseline;
- location of `.github/gsd-local-patches/`;
- location of any pristine comparison copy or metadata file created for the restore;
- diff review summary between backup and new version;
- which changes were offered for restoration;
- which restore mode was recommended for each selected file;
- which changes were actually restored;
- whether `.github/copilot-instructions.md` was preserved verbatim;
- whether phase-level auto-fix customization survived;
- whether the repo-local graphify flow survived;
- whether the `/gsd-add-phase` compatibility alias survived or was intentionally skipped;
- backup manifest integrity summary: `BACKUP_MISSING` count and listed paths (if any);
- any Copilot-relevant installer warnings that remain open after the upgrade;
- whether the `gsd-add-tests` and plan-end auto-fix customizations survived.

## Completion Criteria

Completion requires all of the following:

- GSD was upgraded from the requested upstream source or exact tag;
- the requested ref was resolved successfully before reinstall work started;
- `.github/gsd-local-patches/` exists before destructive replacement happens;
- required backup copies for full overwrite of existing text-like files were created inside `.github/gsd-local-patches/` instead of next to the active live root files under `.github`;
- backup-versus-new diffs were shown;
- the user had a choice over what to restore;
- the `gsd-add-tests` local customization was included in the restore review;
- the plan-end code-review auto-fix customization was included in the restore review;
- the phase-level code-review auto-fix customization was included in the restore review;
- the repo-local graphify customization was included in the restore review;
- the repository-specific `.github/copilot-instructions.md` was preserved;
- backup manifest integrity was validated and any `BACKUP_MISSING` entries were explicitly reported;
- selected changes were restored into the upgraded tree;
- selected changes were merged into the new file structures without pulling unrelated upstream updates backward;
- the final installed version was confirmed from the live `VERSION` file;
- the final active runtime root is `live_root` (`.github/gsd-core` by default), not the legacy root;
- any remaining `.claude` path warnings were surfaced instead of being silently ignored;
- final workspace checks confirmed the required local behaviors survived.

## Non-Negotiable Rules

- Do not use npm latest as the primary upgrade path for this task.
- Do not assume the upstream source tree and the local Copilot install tree use identical paths.
- Do not perform a clean reinstall without first preserving local GSD-file changes.
- Do not create sibling `.bak` files inside the active live root during this upgrade flow.
- Do not store upgrade backup copies anywhere except `.github/gsd-local-patches/` for this repository-local Copilot workflow.
- Do not overwrite `.github/copilot-instructions.md` with the upstream template.
- Do not rely only on installer console output when the live `VERSION` file can be checked directly.
- Do not summarize diffs abstractly when an exact file-level diff can be shown.
- Do not omit the user choice step.
- Do not suppress installer warnings that may leave Copilot-specific paths unresolved.
- Do not restore old files wholesale when the upgraded file gained new structure, new guards, or new upstream logic that is unrelated to the preserved local behavior.
- Do not let preserved local patches drag the upgraded tree backward beyond the minimum code or prompt surface needed to keep the intended functionality.
- Do not leave active GSD skill, agent, or workflow surfaces pointed at `.github/get-shit-done` after a `gsd-core` migration unless the repo explicitly asked to retain legacy execution.
- Do not count or classify `BACKUP_MISSING` entries as live-tree regressions.
- Do not report restore-survival statistics from manifest rows that have no physical backup payload.
- Do not finish while the required local anchors remain only in backup and not in the upgraded live tree.

## Pre-Update Review Checklist

- Confirm the exact `source_repo` and `target_ref` before any install or replacement step.
- Confirm whether the repo is already on `live_root` or still sourcing from `legacy_root`.
- Confirm `.github/gsd-local-patches/` exists and is up to date before destructive actions.
- Confirm required backup copies are in place inside `.github/gsd-local-patches/` for any full overwrite of existing text-like files.
- Confirm `preserved_files.txt` matches the physical backup tree and capture `BACKUP_MISSING` rows before restore decisions.
- Confirm backup-versus-new diffs are ready so restoration can be decided file by file.
- Confirm restore decisions are scoped to minimal required anchors, not wholesale old-file rollback.
- Confirm `.github/copilot-instructions.md`, `gsd-add-tests`, graphify, and the plan/phase auto-fix anchors are explicitly included in restore review.
- Confirm final validation plan includes live `VERSION` check and surfacing unresolved installer warnings.

## Example Invocation

`/GSD-Upgrade-Version target_ref=v1.4.3 source_repo=https://github.com/open-gsd/gsd-core runtime=--copilot live_root=.github/gsd-core legacy_root=.github/get-shit-done`
