---
name: architecture-design-review
description: 'Canonical architecture skill for technical architecture, solution design, existing-system architecture review, and implementation-ready design decisions. Use when the user asks for architecture creation, architecture review, blueprinting, current-state mapping, or one shared architecture direction without duplicated skills.'
---

## Purpose

Create a practical architecture document that aligns requirements, constraints, interfaces, data flow, and implementation decisions so engineers and agents can build consistently.

## When to Use

- The user wants a technical architecture or solution design.
- The user wants a review of an existing architecture, system shape, or service boundary layout.
- A feature or system is large enough to need explicit design decisions.
- Multiple engineers or agents need one shared implementation direction.
- The user wants architecture review discipline and risk classification folded
	into the design output.
- The user wants a blueprint-level map of the current system, but the result
	still needs to stay implementation-ready and pragmatic.
- The user asks for architecture creation, architecture review, blueprinting,
	current-state mapping, or target-state design in one workflow.

## When Not to Use

- The task is a small local code change with no meaningful boundary or system
	design impact.
- The request is purely a code review, syntax cleanup, or one-bug diagnosis.
- There is not enough system context to support defensible architecture
	decisions, trade-offs, or validation steps.

## Inputs

- Problem statement and desired outcomes.
- Functional requirements and non-functional requirements.
- Existing system context, if any.
- Constraints: deadlines, team size, platform, compliance, cost, performance.
- Known risks or open questions.

## Working Rules

- Stay pragmatic. Prefer a design that ships and can evolve.
- Capture trade-offs, not just the chosen path.
- Separate facts, assumptions, and decisions.
- Tie every major choice to operational impact and developer impact.
- Do not call something an architectural flaw without explaining the boundary,
	dependency, or operational consequence that makes it a flaw.
- Distinguish `confirmed risk`, `likely risk`, and `needs validation`.
- Prefer direct evidence from code, config, deployment shape, or design notes
	over pattern-name assumptions.
- Prefer a smaller set of high-confidence decisions and risks over padded,
	generic architecture prose.
- If `architecture-decision-template.md` is present in this folder, use it as the initial skeleton.

## Review Lenses

- Boundary lens: are responsibilities and interfaces placed in the right seams.
- Dependency lens: what depends on what, and how unstable those edges are.
- Failure lens: what breaks, degrades, or cascades when parts fail.
- Scale lens: what assumptions stop holding under growth.
- Change lens: how much system surface must move when requirements change.
- Operator lens: whether deployment, rollout, recovery, and observability are tractable.

## Recommended Workflow

1. Clarify the problem.
	Define scope, goals, exclusions, constraints, and success criteria.

2. Map the current state.
	Identify existing systems, dependencies, boundaries, data sources, and integration points.
	If helpful, produce a lightweight architecture map or ASCII/component view.

3. Enumerate decision areas.
	Cover at least:
	- system boundaries
	- modules or services
	- interfaces and contracts
	- data model and storage
	- data architecture and transformation boundaries
	- security and permissions
	- cross-cutting concerns
	- performance and scaling
	- reliability and failure handling
	- observability
	- rollout and migration
	- testing strategy
	- deployment architecture
	- extension and evolution patterns

4. Evaluate options.
	For each major decision, record viable alternatives, pros and cons, and why one option wins.
	If reviewing an existing design, classify findings by severity and root shape before selecting the change.

5. Define the target architecture.
	Produce the minimum design that is clear enough to implement without invention.
	Preserve implementation readiness; do not import giant prompt scaffolding or variable-template syntax.

6. Record unresolved items.
	Keep open questions and deferred decisions explicit.

7. End with validation steps.
	List what must be benchmarked, prototyped, documented, or tested before the design should be treated as safe to implement.

## Suggested Output Structure

- Context and Goals
- Scope and Constraints
- Assumptions
- Current-State Map or Boundary Summary
- Architecture Overview
- Components and Responsibilities
- Data Flow
- Data Architecture
- Interfaces and Contracts
- Cross-Cutting Concerns
- Key Decisions and Trade-offs
- Findings Ordered by Severity
- Risks and Mitigations
- Testing Architecture
- Rollout and Migration Plan
- Deployment Architecture
- Extension and Evolution Patterns
- Testing and Validation Plan
- Open Questions

## Architecture Risk Shapes

Use these labels when classifying findings so duplicate symptoms collapse into
one root cause where possible:

- boundary violation
- hidden coupling
- under-specified interface
- brittle deployment model
- unclear ownership
- state-consistency hazard
- scaling bottleneck
- poor failure isolation
- observability gap

## Quality Bar

- No decision without rationale.
- No critical interface left ambiguous.
- No hidden dependency on undocumented behavior.
- The document should be useful to an implementer, reviewer, and operator.

## Examples

```text
User: Create a technical architecture for this feature and call out the main risks.
Assistant: Build one implementation-ready architecture document that covers scope,
boundaries, interfaces, trade-offs, findings ordered by severity, and the
validation work required before implementation.
```

```text
User: We need a blueprint of the current backend architecture, but keep it practical.
Assistant: Map the current system from code and configuration, summarize the
boundaries and runtime shape, then convert that into a pragmatic target design,
decision record, and rollout/validation plan.
```
