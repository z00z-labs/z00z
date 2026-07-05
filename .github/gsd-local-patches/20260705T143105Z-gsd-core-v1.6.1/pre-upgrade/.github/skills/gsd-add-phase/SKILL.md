---
name: gsd-add-phase
description: "Compatibility alias for adding a new phase at the end of the current milestone"
argument-hint: "<description>"
allowed-tools: Read, Write, Bash, Glob
---

<objective>
Preserve the legacy `/gsd-add-phase` command name while using the new `gsd-core` add-phase workflow.
</objective>

<execution_context>
@.github/gsd-core/workflows/add-phase.md
</execution_context>

<context>
Phase description: $ARGUMENTS

This is a compatibility alias for users who prefer `/gsd-add-phase` instead of `/gsd-phase`.
Pass all arguments directly to the add-phase workflow.
</context>

<process>
Execute the add-phase workflow end-to-end.
Preserve all validation gates and roadmap/state updates.
</process>
