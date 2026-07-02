---
name: z00z-git-versioning
description: "Handle Z00Z git versioning, minor release commits, direct GitHub sync, and same-branch force-push workflows through the repository-owned version manager. Use when the user asks for git commit, github commit, sync, version bump, minor release, stage-all release commit, or branch-local force push without a pull request. Trigger words: git_commit, github_commit, sync with GitHub, version-manager, minor release, stage-all, same branch, force push, no PR."
argument-hint: '[patch|minor|major|crate|sync|status] [message or crate/version details]'
---

# Z00Z Git Versioning

Use this skill for the Z00Z repository's canonical git and versioning workflow.
The repository-owned scripts for this workflow live in `.github/skills/z00z-git-versioning/scripts/`.

## When to Use

Invoke this skill when the user asks to:

- create a versioned git commit
- do a `minor` release-style commit
- stage all current work and sync it to GitHub
- push directly to the current branch without opening a PR
- use the Z00Z version manager instead of raw git commands
- keep deleted files deleted and avoid restoring them from git

## Rules

1. Always use `.github/skills/z00z-git-versioning/scripts/version-manager.sh` for version-managed git commit and sync flows.
2. Keep commit, sync, and GitHub push on the same currently checked out branch only.
3. For the combined `git_commit minor + github_commit + sync` workflow, prefer `minor --stage-all` and force-push on the same branch.
4. Do not create a pull request for this workflow.
5. Do not restore files deleted from git unless the user explicitly asks.
6. Respect existing user deletions in the worktree.
7. Keep `versions.yaml` internally consistent at all times: if `total_version.version` is `X.Y.Z`, then `total_version.last_git_tag` must be exactly `vX.Y.Z`.
8. Do not create a repository release tag from the `crate` command. Crate-only version updates must not mutate `total_version.version` or `total_version.last_git_tag`.
9. Do not put files larger than `50 MiB` into git by default.
10. To allow a large file intentionally, pass an explicit per-run override flag with the maximum allowed single-file size: `--allow-large-files-up-to-mb <MB>`.

## Core Workflow

### Minor Commit And GitHub Sync

Use this as the default combined workflow when the user asks for a minor git commit plus GitHub sync:

```bash
CURRENT_BRANCH="$(git branch --show-current)"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor --stage-all -f -b "$CURRENT_BRANCH" -m "<message>"
```

This performs all of the following in the same branch only:

- stages tracked, untracked, and deleted changes
- updates `versions.yaml` so `total_version.version == X.Y.Z` and `total_version.last_git_tag == vX.Y.Z`
- creates the git commit and tag
- force-pushes commits and tag to GitHub

### Sync Existing Commit

If the commit already exists and only GitHub sync is needed:

```bash
CURRENT_BRANCH="$(git branch --show-current)"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh sync -f -b "$CURRENT_BRANCH"
```

### Other Version Bumps

```bash
CURRENT_BRANCH="$(git branch --show-current)"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh patch -f -b "$CURRENT_BRANCH" -m "<message>"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh major -f -b "$CURRENT_BRANCH" -m "<message>"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh crate <crate-name> <version> --stage-all -f -b "$CURRENT_BRANCH" -m "<message>"
```

## Decision Logic

1. If the user explicitly asks for `minor` and wants commit plus GitHub sync, run the combined `minor --stage-all -f` flow on the current branch.
2. If the user asks only for sync, use `sync -f -b "$CURRENT_BRANCH"`.
3. If the user asks for another bump type, substitute `patch`, `major`, or `crate`.
4. If the current branch does not match the requested branch, stop and use the checked out branch unless the user explicitly wants a different branch.
5. If deleted files are present, leave them deleted. Never restore them automatically.
6. If `versions.yaml` is inconsistent, fix or block the workflow before creating a release commit or sync.
7. For `crate` updates, commit and push the branch only; do not create a repository release tag unless the user separately requests a repo release bump.
8. If staged or to-be-pushed git content contains a file larger than `50 MiB`, block the workflow unless the run explicitly supplies `--allow-large-files-up-to-mb <MB>`.

## Verification

Before reporting success:

- confirm the command used the current branch
- confirm no PR flow was introduced
- confirm deleted files were not restored
- confirm the workflow used the repository-owned script path under `.github/skills/z00z-git-versioning/scripts/`
- confirm `versions.yaml` ended with `total_version.version == X.Y.Z` and `total_version.last_git_tag == vX.Y.Z`
- confirm no file above the default `50 MiB` limit entered git without an explicit `--allow-large-files-up-to-mb <MB>` override

## Examples

```text
/z00z-git-versioning do a minor git commit and GitHub sync on this branch
```

```text
/z00z-git-versioning stage everything, version it, and force-push to the same branch only
```

```text
/z00z-git-versioning sync the current branch without creating a PR
```

```text
/z00z-git-versioning bump z00z_core to 0.2.0 and push on the same branch
```
