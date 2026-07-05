---
name: mermaid-c4
description: Generate C4-style Mermaid architecture diagrams from prompts, docs, or code references. Use when the user wants a system context diagram, container diagram, component diagram, code diagram, system landscape, dynamic diagram, or deployment diagram, or asks to visualize software architecture, service boundaries, subsystem relationships, context maps, or C4 model views in Mermaid. Keeps the same semantic color palette as mermaid-spectrum.
argument-hint: 'source=<prompt|file|doc|code> scope=<context|container|component|code|landscape|dynamic|deployment|auto> [target=<system-or-container>] [audience=onboarding|review|audit]'
---

# Mermaid C4

Create the smallest useful C4-style Mermaid output that correctly reflects the
requested architecture level.

Do not flatten the C4 model into a generic flowchart when the user is asking
for a specific architecture view. Keep the C4 abstractions explicit, keep the
view hierarchy clean, and reuse the semantic color palette from
`mermaid-spectrum` exactly as documented in `REFERENCE.md`.

## When to Use

Use this skill when:

- The user asks for a C4 diagram or a C4-style Mermaid diagram.
- The user wants a system context, container, component, code, dynamic,
  deployment, or system landscape view.
- The user asks to visualize architecture boundaries, service relationships,
  subsystem structure, or runtime topology in Mermaid.
- The user has prose, code, docs, or notes that need to become a C4 model
  visualization.
- The user wants the same semantic color palette already used by
  `mermaid-spectrum`, but applied to C4 views.

Do not use this skill when:

- The user wants a non-C4 Mermaid diagram and does not care about C4
  abstractions.
- The task is only to render a finished Mermaid block with no selection or
  modeling work.
- The task is pure code editing with no visualization outcome.

## Core Outcome

Produce one of these outputs:

1. A single C4 view when one level fully answers the request.
2. A paired C4 set when one static view and one supporting view are both needed.
3. A compact C4 pack when the user needs multiple levels or a static plus
   supporting view combination.

Every substantive output must include:

- a short explanation of which C4 view or views were selected
- Mermaid code blocks in deliberate reading order
- the same semantic color palette used by `mermaid-spectrum`
- a short note about assumptions or what was intentionally omitted

## How It Works

### Step 1: Identify the C4 abstraction level

Model the request using the official C4 hierarchy:

- people use software systems
- software systems contain containers
- containers contain components
- components are implemented by code elements

Also detect whether the user is asking for one of the three supporting views:

- system landscape
- dynamic
- deployment

If the user does not name the level directly, infer it from the nouns they use:

- product, platform, users, external systems -> system context or system landscape
- services, apps, APIs, databases, queues -> container
- modules, subsystems, adapters, handlers, stores inside one service -> component
- classes, traits, interfaces, files, functions -> code
- scenario, request path, message order -> dynamic
- node, region, cluster, environment, runtime instance -> deployment

### Step 2: Choose the smallest complete C4 view set

Select only the levels that add meaning.

- Use `system context` for people, the system of interest, and external systems.
- Use `container` for deployable applications and data stores inside one system.
- Use `component` for internal building blocks inside one container.
- Use `code` only when the user explicitly needs code-level structure or the
  discussion is already at class, trait, or function level.
- Use `system landscape` when multiple software systems must be compared or
  positioned together.
- Use `dynamic` when interaction order matters at a chosen abstraction level.
- Use `deployment` when runtime nodes or environments matter.

Follow the C4 guidance that you do not need all levels every time. System
context and container are often enough unless the prompt demands deeper detail.

### Step 3: Map the C4 view to Mermaid notation

Use Mermaid as the rendering notation, while keeping the C4 abstraction rules.

- `system context` -> `flowchart LR` or `graph LR`
- `system landscape` -> `flowchart LR` or `graph LR`
- `container` -> `flowchart LR`; use `architecture-beta` only when it improves
  runtime or service topology clarity
- `component` -> `flowchart LR` with clear container boundaries
- `code` -> `classDiagram` when code relationships matter, otherwise `graph LR`
- `dynamic` -> `sequenceDiagram`
- `deployment` -> `architecture-beta` when supported, otherwise `flowchart LR`
  with deployment-node subgraphs

Use Mermaid `subgraph` boundaries to preserve the system or container scope
when a view has nested ownership.

### Step 4: Apply C4 labeling and palette discipline

For every static C4 view:

- keep one primary abstraction level per diagram
- label each element with a name and short responsibility
- include technology labels for containers when the prompt provides them
- write relationship labels as short verb phrases
- clearly distinguish internal versus external elements
- use the palette in `REFERENCE.md` without inventing new colors

For supporting views:

- keep dynamic diagrams at the same abstraction level as the paired static view
- keep deployment diagrams focused on runtime placement, not on component logic
- if `architecture-beta` styling becomes brittle, fall back to a styled
  `flowchart` so the palette stays visible and the C4 meaning stays clear

### Step 5: Emit the output contract

For every response, use this compact structure:

```text
Diagram plan:
- <C4 view + Mermaid type>: <why it is needed>

Inferences:
- <only when major structure was inferred>

Mermaid:
```mermaid
...
```

Not shown:
- <what was intentionally omitted>
```

If the user explicitly asks for raw Mermaid only, omit the prose and return the
code blocks only.

## Decision Rules

- If the user says `C4` without naming a level and the system of interest is
  clear, start with `system context` and add `container` only when internals
  are needed.
- If the prompt is already scoped inside one application or service, start with
  `container` or `component`, not `system context`.
- If the prompt names modules inside one service, prefer `component` and add a
  `dynamic` view only if scenario order matters.
- If the prompt names files, classes, traits, or functions, use `code` only if
  a code-level view adds value beyond the component view.
- If the prompt mentions multiple peer systems or a portfolio boundary, use
  `system landscape`.
- If the prompt mentions clusters, nodes, regions, pods, or environments, add a
  `deployment` view.
- If the prompt describes a scenario, use case, or request path, add a
  `dynamic` diagram at the same abstraction level as the selected static view.
- If two static levels seem plausible, choose the higher level first and only
  add the lower level when it changes interpretation.
- If the user explicitly asks for one C4 level, honor that request unless a
  small supporting view is needed to prevent ambiguity.

## Examples

### Example 1: Context only

User: `Create a C4 context diagram in Mermaid for this payment platform.`

Expected response shape:

```text
Diagram plan:
- system context + flowchart LR: the ask is about users, the platform, and external systems.
```

### Example 2: Container plus dynamic

User: `Show the containers for the notification service and the message flow for sending an email.`

Expected response shape:

```text
Diagram plan:
- container + flowchart LR: shows deployable applications and data stores.
- dynamic + sequenceDiagram: shows the send-email scenario at container level.
```

### Example 3: Component deep dive

User: `Turn this service module note into a C4 component diagram and include the request path.`

Expected response shape:

```text
Diagram plan:
- component + flowchart LR: shows internal building blocks inside one container.
- dynamic + sequenceDiagram: shows how those components collaborate for one scenario.
```

### Example 4: Deployment view

User: `Generate a C4 deployment diagram in Mermaid for the API across cloud region, cluster, and database.`

Expected response shape:

```text
Diagram plan:
- deployment + architecture-beta: best fit for runtime placement and node boundaries.
```

## Validation Checklist

- The selected C4 level matches the user's question.
- Static diagrams do not mix multiple abstraction levels without clear intent.
- Supporting diagrams stay tied to the chosen static level.
- Mermaid notation is chosen for clarity, not novelty.
- The palette matches `mermaid-spectrum` semantics exactly.
- The answer explains why these C4 views were selected.

## Supporting File

- `REFERENCE.md`: C4 view selection matrix, bundle recipes, Mermaid mapping,
  and the shared semantic palette copied from `mermaid-spectrum`

## Invocation Examples

Use requests like these to trigger the skill clearly:

```text
Create a C4 Mermaid diagram for this service architecture.
```

```text
Show a system context and container view in Mermaid using the C4 model.
```

```text
Turn this code and README into a C4 component diagram with the same palette as mermaid-spectrum.
```