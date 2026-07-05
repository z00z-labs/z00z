---
name: document-project
description: 'Document brownfield projects for humans and AI context without depending on external workflow files. Use when the user says "document this project" or "generate project docs"'
---

## Purpose

Generate practical project documentation for an existing codebase so future engineers and AI agents can understand the structure, architecture, conventions, entry points, data flow, and operational model.

## When to Use

- The user wants documentation for an existing project.
- A brownfield repository needs onboarding materials.
- A codebase needs structured docs before planning new features.

## Inputs

- Project root.
- Desired depth: quick, deep, or exhaustive.
- Optional focus areas: APIs, data models, architecture, deployment, UI, integrations.
- Optional existing docs to update rather than replace.

## Working Rules

- Match the level of depth to the user's goal.
- Prefer writing incrementally instead of holding all findings in memory.
- Distinguish verified facts from assumptions.
- If local helper files such as `instructions.md`, `checklist.md`, or `documentation-requirements.csv` exist, treat them as optional accelerators, not required dependencies.

## Recommended Modes

1. Initial Scan
	Create a first documentation set for an undocumented or lightly documented project.

2. Full Rescan
	Refresh an existing documentation set after meaningful codebase drift.

3. Deep Dive
	Document one subsystem, feature area, or module in much more detail.

## Recommended Workflow

1. Detect project type and stack.
	Identify manifests, languages, frameworks, build tools, deployment config, tests, and repo layout.

2. Determine scan depth and scope.
	For deep or exhaustive scans, batch by folder or subsystem.

3. Map the codebase.
	Capture entry points, major directories, APIs, data models, shared utilities, tests, CI, deployment, and integration boundaries.

4. Write documentation as you go.
	Suggested outputs:
	- `index.md`
	- `project-overview.md`
	- `source-tree-analysis.md`
	- `architecture.md`
	- `development-guide.md`
	- optional `api-contracts.md`
	- optional `data-models.md`
	- optional `deployment-guide.md`
	- optional `component-inventory.md`
	- optional `integration-architecture.md`

5. Validate quality.
	Ensure the docs are accurate, navigable, and useful for onboarding and future planning.

## Output Expectations

- Clear project summary and stack inventory.
- Accurate source tree explanation.
- Entry points and runtime flows documented.
- Data and API surfaces documented where relevant.
- Build, test, and run instructions captured.
- Explicit note of gaps or unknown areas.

## Quality Bar

- The docs should help someone start modifying the project.
- No placeholder text or fake certainty.
- Architecture descriptions should reflect the actual code, not idealized intent.
- Deep-dive outputs must identify dependencies, dependents, and risky modification areas.
