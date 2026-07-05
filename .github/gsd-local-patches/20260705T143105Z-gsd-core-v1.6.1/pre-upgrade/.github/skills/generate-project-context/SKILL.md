---
name: generate-project-context
description: 'Create project-context.md with concise AI implementation rules, patterns, and constraints. Use when the user says "generate project context" or "create project context"'
---

## Purpose

Generate a concise `project-context.md` file that captures the non-obvious rules, patterns, commands, and pitfalls that AI agents and new contributors must follow when working in the repository.

## When to Use

- The user wants a `project-context.md` file.
- A repository has repeated conventions that AI agents keep missing.
- Existing instructions are too scattered, noisy, or tool-specific.

## Inputs

- Repository root or target folder.
- Existing docs, instructions, architecture notes, README files, and conventions.
- Optional emphasis: coding rules, architecture boundaries, testing, deployment, naming, security, or review expectations.

## Working Rules

- Keep the file lean. It is a reminder layer, not full documentation.
- Include only details that materially improve implementation quality.
- Prefer unobvious rules over generic best practices.
- If `project-context-template.md` exists locally, use it as the starting scaffold.

## Recommended Workflow

1. Discover source material.
	Read the most relevant docs, instructions, build scripts, manifests, and code conventions.

2. Extract the highest-value context.
	Focus on:
	- stack and versions
	- architectural boundaries
	- naming and file placement rules
	- required commands
	- testing expectations
	- security and compliance constraints
	- known pitfalls and anti-patterns
	- preferred reuse patterns

3. Compress aggressively.
	Convert verbose docs into terse, actionable reminders.

4. Write `project-context.md`.
	Keep it easy for an agent to scan quickly before implementation.

## Suggested Output Structure

- Technology Stack and Versions
- Critical Implementation Rules
- Project Structure and Boundaries
- Required Commands
- Testing and Validation Expectations
- Security and Safety Notes
- Common Mistakes to Avoid

## Quality Bar

- Every item must change implementation behavior in a useful way.
- No duplicate README content unless it is essential and easy to forget.
- The output should be short enough to be read repeatedly.
