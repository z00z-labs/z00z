---
name: tech-stack-blueprint
description: 'Analyze a repository and generate a technology stack blueprint that documents languages, frameworks, tooling, versions, conventions, integration points, and implementation patterns for consistent future code generation. Use when the user asks for a tech stack map, dependency inventory, framework audit, architecture-by-technology, implementation patterns, or to convert a stack-analysis prompt into a reusable workflow. Trigger words: tech stack, technology stack, stack blueprint, dependency inventory, technology map, framework audit, implementation patterns, coding conventions, architecture map.'
argument-hint: '[project type|auto-detect] [basic|standard|comprehensive|implementation-ready] [markdown|json|yaml|html]'
---

# Technology Stack Blueprint

## Mission

Turn a repository scan or a stack-analysis prompt into a saved technology stack
blueprint that future contributors and AI agents can use as an implementation
guide.

The blueprint must be evidence-driven. Do not invent dependencies, versions,
licenses, conventions, or architecture decisions. If the repository does not
prove a claim, label it as `unknown`, `not detected`, or `inferred`.

## When to Use

- mapping the current technology stack of a repo or monorepo
- building a dependency and tooling inventory
- documenting languages, frameworks, and configuration patterns
- extracting implementation conventions for future code generation
- creating an implementation-ready stack reference before large feature work
- upgrading a one-off stack-analysis prompt into a reusable workflow

## Do Not Use This Skill For

- deep code review for bugs, vulnerabilities, or correctness defects
- full architecture redesign or ADR creation
- package upgrade planning by itself
- dependency license compliance sign-off by itself
- repo-wide API reference generation

## Invocation Signals

Invoke this skill when the user asks for any of the following outcomes:

- a technology stack map for a repository, crate, app, service, or monorepo
- an inventory of languages, frameworks, build tools, test tools, or package managers
- a dependency landscape grouped by role, layer, or purpose
- a coding-conventions map intended to guide future implementation
- an implementation-pattern blueprint for adding new code consistently
- a one-off stack-analysis prompt converted into a reusable workflow

Strong trigger phrases include:

- `analyze the tech stack`
- `map this repository`
- `generate a technology blueprint`
- `document the stack`
- `framework audit`
- `dependency inventory`
- `what technologies are used here`
- `convert this prompt into a skill`

If the request is partly about stack mapping and partly about implementation
planning, this skill may be used first to establish the stack baseline.

## Default Configuration

If the user does not provide explicit settings, use these defaults:

| Setting | Default | Rule |
|---|---|---|
| `project_type` | `Auto-detect` | Scan manifests, config files, lockfiles, and source layout first |
| `depth_level` | `Comprehensive` | Prefer a useful blueprint over a shallow inventory |
| `include_versions` | `true` | Extract only from manifests, lockfiles, or tool configs |
| `include_licenses` | `false` | Enable only when user asks or the repo already tracks license inventory |
| `include_diagrams` | `true` for `Markdown` and `HTML`, else `false` | Use Mermaid when text output supports it |
| `include_usage_patterns` | `true` | Show representative patterns, not random snippets |
| `include_conventions` | `true` | Capture naming, layering, testing, and config norms |
| `output_format` | `Markdown` | Prefer readable, reviewable artifacts |
| `categorization` | `Layer` | Group by runtime role first, then by technology |
| `output_path` | `Technology_Stack_Blueprint.<ext>` | Save in workspace root unless the user specifies another path |

## Required Intake

Resolve these inputs early, explicitly or by sensible default:

1. Project type or whether auto-detection is required.
2. Depth level.
3. Output format.
4. Whether versions, licenses, diagrams, usage patterns, and conventions are needed.
5. Categorization method.
6. Output path if the default file name is not desired.

If the user leaves most of these open, continue with defaults instead of
blocking.

## Clarification Policy

Ask follow-up questions only when one of these conditions is true:

- the user explicitly wants the artifact saved to a particular path or scope and
   that destination is unclear
- the user requests license coverage or version precision, but the acceptable
   evidence source is ambiguous
- the repository contains multiple major stacks and the user wants focus on only
   one or two of them
- the user asks for an implementation-ready artifact, but the target audience is
   unclear, such as contributors, AI agents, or reviewers

Otherwise, proceed with defaults and state the chosen assumptions in the output.

## Depth Modes

Choose the mode once and keep the output aligned to it.

| Mode | Include | Exclude |
|---|---|---|
| `Basic` | languages, frameworks, package managers, major tooling, primary manifests | detailed examples, diagrams, implementation templates |
| `Standard` | `Basic` plus versions, core configs, build/test tooling, high-level conventions | extensive templates and deep integration maps |
| `Comprehensive` | `Standard` plus integration points, relationship maps, representative usage patterns, diagrams, decision context | ready-to-copy file templates unless they are obvious |
| `Implementation-Ready` | `Comprehensive` plus file/class templates, checklists, integration guidance, testing requirements, documentation rules | nothing essential for new feature delivery |

## Project Type Resolution

Use this branching logic:

1. If the user specifies a project type, prioritize that stack while still
   noting adjacent technologies that materially affect implementation.
2. If `Auto-detect` is selected, scan manifests, lockfiles, CI files, build
   scripts, container files, and source tree patterns.
3. If multiple stacks are present, produce a multi-stack blueprint instead of
   forcing a single dominant technology.
4. If detection is weak, say so and continue with the evidence that exists.

Supported focal stacks:

- `.NET`
- `Java`
- `JavaScript`
- `React.js`
- `React Native`
- `Angular`
- `Python`
- `Other`
- `Auto-detect`

## Execution Workflow

### Phase 1: Repository Discovery

Collect facts before drafting the blueprint.

Inspect:

- source tree and language distribution
- manifest files such as `package.json`, `Cargo.toml`, `pyproject.toml`,
  `requirements.txt`, `pom.xml`, `build.gradle`, `.csproj`, and lockfiles
- CI pipelines, Dockerfiles, container compose files, task runners, and build scripts
- formatting, linting, testing, and analysis config
- major entrypoints, modules, packages, apps, and libraries

Capture only what the repository shows.

### Phase 2: Technology Identification

Build the inventory.

For each detected technology, record:

- name and category
- where it is declared
- version if explicitly declared
- purpose in this repository
- scope, such as workspace-wide, service-specific, app-specific, or test-only
- related tooling or configuration files

When `include_licenses=true`, attach license data only if it is explicitly
available from lockfiles, manifest metadata, vendored metadata, or an existing
license report.

### Phase 3: Core Stack Analysis

Analyze stack-specific implementation reality.

#### .NET

- target frameworks and C# language version
- NuGet package groups and purpose
- solution and project organization
- configuration model, options binding, and environment handling
- auth, API, DI, middleware, and data access patterns

#### Java

- JDK level and framework choices
- Maven or Gradle structure
- Spring or Jakarta usage patterns
- annotations, DI, data access, and API style

#### JavaScript / TypeScript

- runtime and module system
- bundler or build toolchain
- package manager and workspace shape
- type system usage, linting, formatting, and test stack

#### React / Angular / React Native

- UI architecture and routing
- component composition style
- state management, forms, styling, and API integration
- testing approach for components and user flows

#### Python

- interpreter version and environment management
- framework, ORM, packaging, and task runner usage
- app layout, API organization, and test approach

#### Other

- language- or framework-specific conventions proven by the repository
- toolchain, runtime, build, and deployment characteristics

### Phase 4: Patterns And Conventions

When `include_conventions=true`, document the working norms that a new
contributor or AI agent must follow.

Focus on:

- naming conventions for files, modules, types, and functions
- code organization and folder boundaries
- dependency direction and shared-library boundaries
- configuration loading patterns
- error handling, logging, validation, and auth patterns
- testing structure and common test harness choices

Use representative patterns, not exhaustive trivia.

### Phase 5: Usage Pattern Extraction

When `include_usage_patterns=true`, extract a small set of high-signal examples.

Prefer examples for:

- endpoint or controller patterns
- service-layer composition
- repository or data-access patterns
- UI component or page structure
- job, worker, or CLI command structure
- dependency injection wiring

Do not dump long code listings. Summarize the pattern and include a compact,
representative snippet only when it clarifies the convention.

### Phase 6: Relationship Map

For `Comprehensive` and `Implementation-Ready`, build the system-level stack map.

Document:

- how major layers connect
- how frontend, backend, storage, messaging, and external services interact
- where auth, config, logging, and observability sit
- what tooling controls quality gates, builds, and delivery
- what runtime or infrastructure constraints shape implementation

If diagrams are enabled, use Mermaid and keep diagrams minimal but informative.

### Phase 7: Implementation Blueprint

Only in `Implementation-Ready`, add concrete delivery guidance:

- common file or class templates by component type
- checklist for adding a new feature end-to-end
- integration touchpoints to update when new code is added
- expected tests for each component type
- documentation requirements for new code
- common pitfalls that would break consistency with the current stack

### Phase 8: Prompt Upgrade

When the input is itself a freeform prompt rather than a direct repository
analysis request:

- extract the stable workflow hidden in the prompt
- convert configurable prompt variables into explicit skill defaults or inputs
- remove decorative wording that does not change execution behavior
- preserve all meaningful decision branches, depth modes, and output contracts
- rewrite the result as reusable operational instructions instead of a single
   one-shot prompt

## Output Contract

The primary deliverable is a saved file named:

- `Technology_Stack_Blueprint.md`
- `Technology_Stack_Blueprint.json`
- `Technology_Stack_Blueprint.yaml`
- `Technology_Stack_Blueprint.html`

Choose the extension from `output_format`.

Unless the user asks for a different destination, save it in the workspace root.

### Markdown Blueprint Shape

For `Markdown`, prefer this section order:

1. `## Scope And Detection Summary`
2. `## Technology Inventory`
3. `## Core Stack Analysis`
4. `## Tooling And Build Pipeline`
5. `## Conventions And Implementation Patterns`
6. `## Integration Points`
7. `## Decision Context And Constraints`
8. `## New Code Blueprint` when depth is `Implementation-Ready`
9. `## Diagrams` when enabled
10. `## Unknowns And Follow-Ups`

### JSON / YAML Shape

For machine-readable formats, use stable top-level keys:

- `metadata`
- `detection_summary`
- `technology_inventory`
- `core_stacks`
- `tooling`
- `conventions`
- `usage_patterns`
- `integration_points`
- `decision_context`
- `implementation_blueprint`
- `unknowns`

## Quality Bar

Do not finalize until all of the following are true:

- every listed technology is backed by repository evidence
- versions are quoted only when explicitly declared
- inferred conclusions are labeled as inferred
- multi-stack repos are represented as multi-stack, not flattened badly
- examples reflect actual repository patterns
- the output is saved to the requested file path
- the summary given to the user highlights the main stack findings and the file path

## Failure Modes To Avoid

- inventing versions from memory or current ecosystem defaults
- treating dev dependencies as production architecture without qualification
- flattening a monorepo into one app when several subprojects exist
- listing every package without explaining purpose or scope
- producing decorative diagrams that add no implementation value
- confusing code style preferences with repository-enforced conventions
- claiming a framework is used based on folder names alone

## Completion Checklist

Before finishing, verify:

- the chosen depth level matches the artifact detail
- optional sections appear only when enabled or justified by depth
- stack-specific analysis is present only for detected or requested stacks
- unknowns, gaps, and ambiguous areas are surfaced explicitly
- the user-facing response does not dump the entire blueprint unless requested

## User Response Contract

When this skill completes, the response to the user should include:

- the detected or targeted stack summary in a few high-signal lines
- the saved output path
- the main assumptions or defaults that were applied
- any major unknowns or confidence limits
- optional next prompts the user can try to extend or narrow the blueprint

## Example Prompts

```text
Create a comprehensive technology stack blueprint for this repo.
```

```text
Auto-detect the stack and generate an implementation-ready blueprint with versions and diagrams.
```

```text
Focus on the React and Python parts of this monorepo and save a markdown technology blueprint.
```

```text
Convert this stack-analysis prompt into a reusable workflow and save the blueprint as YAML.
```