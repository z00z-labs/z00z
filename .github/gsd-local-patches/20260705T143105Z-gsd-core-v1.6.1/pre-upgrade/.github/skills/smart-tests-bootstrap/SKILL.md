---
name: smart-tests-bootstrap
description: 'Use the self-contained Z00Z fast-fail bootstrap suite that lives inside this skill. Runs a representative test subset (~25% of the workspace), skips known-slow paths, uses the test-params-fast feature, and keeps the canonical script colocated with the skill. Use when: debugging quickly, pre-commit sanity check, CI branch triage, or when you need the bootstrap gate without generating files elsewhere. Trigger words: fast tests, quick tests, test subset, bootstrap tests, smart tests, fast-fail, debug tests, quick check.'
argument-hint: '[run|show|refresh]'
---

# Smart Tests Bootstrap

## When to Use

- The user wants a fast-fail sanity test pass instead of the full workspace suite.
- The task is quick debugging, pre-commit validation, CI triage, or high-signal regression checking.
- The bootstrap gate should run from the canonical colocated script rather than generating a new helper.
- The user asks for smart tests, quick tests, bootstrap tests, or a representative subset run.

## Mission

Use the bootstrap script that lives inside this skill:
`./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

This skill is self-contained. It does not generate a script in `scripts/` and it does not depend on external skill assets.

The script covers unit tests, high-signal integration tests, benches (compile-only), and examples (compile-only).

Total test inventory: **~3 373** `#[test]` annotations across 11 crates (7 with non-zero tests).
Target budget: **~850 tests (~25%)**: foundational lib (766) + wallet integration (84) + compile checks.
Expected runtime: **~8–11 min** versus roughly **~30 min** for the full suite.

---

## Primary Output

When this skill is invoked, run or inspect the script at:
`./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

Supported behaviors:

1. `run` or no argument: execute the script.
2. `show`: print the script path and summarize what it covers.
3. `refresh`: update the colocated script in place when test selection rules change.

If the user asks to run it, use:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

---

## Canonical Script

The script content lives at:
`./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

After refreshing the script, keep it executable:

```bash
chmod +x ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```
