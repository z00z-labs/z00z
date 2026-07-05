---
name: gsd-add-tests
description: "Generate tests for a completed phase based on UAT criteria and implementation"
argument-hint: "<phase> [additional instructions]"
allowed-tools: Read, Write, Edit, Bash, Glob, Grep, Agent, AskUserQuestion
---

<objective>
Generate unit and E2E tests for a completed phase, using prefixed or unprefixed SUMMARY, CONTEXT, TODO, VERIFICATION or VALIDATION artifacts together with every `*-PLAN.md` as specifications.

Analyzes implementation files, classifies them into TDD (unit), E2E (browser), or Skip categories, presents a test plan for user approval, then generates tests following RED-GREEN conventions.

Output: phase-local `NNN-TEST-SPEC.md`, `NNN-TESTS-TASKS.md`, optional compatibility `NNN-TEST-PLAN.md`, and optionally committed test files with message `test(phase-{N}): add unit and E2E tests from add-tests command` when the user explicitly wants the commit
</objective>

<execution_context>
@.github/gsd-core/workflows/add-tests.md
</execution_context>

<context>
Phase: $ARGUMENTS

@.planning/STATE.md
@.planning/ROADMAP.md
</context>

<process>
Execute the add-tests workflow from @.github/gsd-core/workflows/add-tests.md end-to-end.
Preserve all workflow gates (classification approval, test plan approval, RED-GREEN verification, gap reporting).
</process>
