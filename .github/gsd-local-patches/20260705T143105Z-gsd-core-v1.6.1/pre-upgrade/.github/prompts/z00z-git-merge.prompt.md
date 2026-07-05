---
name: "z00z-git-merge"
agent: agent
description: "Perform a direct branch merge in the z00z repository with no pull request, using safe direct-push rules and branch defaults."
argument-hint: "source_branch=<optional> target_branch=<optional>"
---

# Z00Z Git Merge

## Mission

Perform a direct branch merge in the z00z repository, with no pull request.

## Inputs

- `source_branch` (optional): source branch to merge from.
- `target_branch` (optional): target branch to merge into.

## Defaults

- If `source_branch` is omitted, empty, or not provided, use the branch that is currently checked out before the operation starts.
- If `target_branch` is omitted, empty, or not provided, use `main`.

## Requirements

1. Goal: merge `source_branch` -> `target_branch`.
2. After completion, you must return to the branch that was checked out before starting the operation.
3. Do not create a pull request. Do not use any pull request workflow.
4. Do not restore files deleted from git unless I explicitly ask for that.
5. Respect existing deletions in the worktree.
6. If the operation cannot be continued safely because of conflicts or other blocking conditions, stop and show the exact reason. Do not guess conflict resolution.
7. If a force push is required, use only `--force-with-lease`, never plain `--force`.
8. Do not use the version-manager workflow for version bumps; this is a normal branch merge only.
9. If switching to `target_branch` is blocked only by local tracked changes on the current branch that must be preserved and should be included in `target_branch`, you may create one normal commit on the current branch containing only those blocking files, push that branch, and then continue the merge flow. Do not use stash, reset, restore, or clean.
10. Do not restore deleted files and do not discard local work.

## Workflow

First, save the current branch in a variable, then resolve effective source and target branches.

Preferred sequence:

```bash
PREV_BRANCH="$(git branch --show-current)"
SOURCE_BRANCH="${source_branch:-$PREV_BRANCH}"
TARGET_BRANCH="${target_branch:-main}"

git fetch origin &&
git fetch origin "$SOURCE_BRANCH" &&
test "$(git rev-parse "$SOURCE_BRANCH")" = "$(git rev-parse "origin/$SOURCE_BRANCH")" &&
git switch "$TARGET_BRANCH" &&
git pull --ff-only origin "$TARGET_BRANCH" &&
git merge --no-ff --no-edit "$SOURCE_BRANCH" &&
git push --force-with-lease origin "$TARGET_BRANCH" &&
git fetch origin "$TARGET_BRANCH" &&
test "$(git rev-parse "$TARGET_BRANCH")" = "$(git rev-parse "origin/$TARGET_BRANCH")" &&
test "$(git merge-base "$SOURCE_BRANCH" "$TARGET_BRANCH")" = "$(git rev-parse "$SOURCE_BRANCH")" &&
git switch "$PREV_BRANCH"
```

After the merge:

- If a normal push works without force, you may use a normal push.
- If a forced direct push is required, use only:

```bash
git push --force-with-lease origin "$TARGET_BRANCH"
```

## Additional Rules

- If the merge stops because of conflicts, do not attempt to guess the resolution. Stop and show the exact reason and the list of conflicted files.
- If switching to `target_branch` is blocked by local tracked uncommitted changes that must be preserved and should be included in `target_branch`, you may:
  1. commit only the blocking files on `PREV_BRANCH` with a normal git commit,
  2. push `PREV_BRANCH`,
  3. continue with the merge sequence.
- If switching to `target_branch` is blocked by local tracked uncommitted changes that should NOT be included in `target_branch`, stop and show the exact reason and the blocking files. Do not restore anything.
- If `source_branch` and `target_branch` resolve to the same branch, stop and report that exact reason.
- Before switching to `target_branch`, verify that local `source_branch` matches `origin/source_branch`. If it does not, stop and show the exact reason unless you intentionally pushed `source_branch` first.
- If a normal push fails and a forced direct push is required, use only `git push --force-with-lease origin "$TARGET_BRANCH"`, then restore the original branch.

## Completion Confirmation

At the end, confirm all of the following:

1. the merge was completed into `target_branch`,
2. no pull request was created,
3. deleted files were not restored,
4. the original branch was restored.

## Example Invocations

- `/z00z-git-merge`
- `/z00z-git-merge source_branch=z00z-dev`
- `/z00z-git-merge source_branch=z00z-dev target_branch=main`
- `/z00z-git-merge source_branch=release-candidate target_branch=main`