---
name: drafts-to-spec
description: Auto-invoked when user wants to turn scattered notes, TODOs, draft docs, planning folders, or design files into one real specification, single source of truth, or architecture document. Also triggers on create spec, consolidate design, architecture-grade specification, SSoT spec, YAML config gate, Mermaid diagrams, ownership matrix, and test matrix.
argument-hint: 'SOURCE_DIR=<dir> OUTPUT_SPEC_PATH=<path> [SPEC_TITLE=<title>] [SPEC_SCOPE=<feature|module|protocol|storage|runtime|api|security|testing|deployment|general>] [OUTPUT_MODE=<create|overwrite|review-only>] [REFERENCE_SPECS=<path1,path2>] [STYLE_GUIDES=<path1,path2>] [PROJECT_ROOT=<dir>]'
---

# Drafts To Spec

Generate a full, self-contained, architecture-grade specification from a folder of planning documents plus repository evidence.

## 🔍 When to Use

Use this skill when the user wants to:

- turn a planning folder, TODO set, or design draft into one canonical specification
- merge scattered notes into a single source of truth
- create a feature, protocol, storage, runtime, API, security, testing, or deployment spec
- reconcile stale docs against current code, config, and tests
- produce a spec with YAML config gates, Mermaid diagrams, ownership tables, and test matrices

Do not use this skill for:

- shallow summaries
- code-only implementation without a specification artifact
- copying an old spec into a new project without fresh evidence

## ⚙️ Inputs

Required:

- `SOURCE_DIR`
- `OUTPUT_SPEC_PATH`

Optional:

- `SPEC_TITLE`
- `SPEC_SCOPE`
- `OUTPUT_MODE`
- `REFERENCE_SPECS`
- `STYLE_GUIDES`
- `PROJECT_ROOT`

Accepted values:

```text
SOURCE_DIR=<directory containing planning docs or drafts>
OUTPUT_SPEC_PATH=<where to write the generated spec>
SPEC_SCOPE=<feature|module|protocol|storage|runtime|api|security|testing|deployment|general>
OUTPUT_MODE=<create|overwrite|review-only>
REFERENCE_SPECS=<comma-separated paths>
STYLE_GUIDES=<comma-separated paths>
PROJECT_ROOT=<repository root; default: current workspace root>
```

Rules:

- If `SOURCE_DIR` or `OUTPUT_SPEC_PATH` is missing, ask only for the missing value.
- If required inputs are present, proceed without unnecessary questions.
- If `SPEC_TITLE` is missing, derive it from the source docs and target scope.
- If `PROJECT_ROOT` is missing, infer it from the current workspace.

## 🚫 Universal Behavior Rules

- The skill MUST be universal and MUST NOT hardcode any specific project, phase number, blockchain, consensus model, storage engine, proof system, crate name, module name, package name, path convention, or prior decision.
- The skill MUST infer architecture from the current user request, source docs, repository evidence, config, tests, and explicit user instructions.
- REFERENCE_SPECS are style examples, not architecture authority.
- Reference specs may teach structure, depth, rigor, and format, but project-specific content from them MUST NOT be copied unless the current source docs or repository evidence support it.
- Existing code is current implementation truth.
- Existing tests are current behavior evidence.
- Existing config is current operational or planned gate evidence.
- Source docs are design intent.
- Missing code is proposed work only.
- The generated spec MUST explain the real implementation of relevant modules, packages, crates, services, adapters, and tool boundaries rather than only abstract target architecture.
- The generated spec MUST inventory required libraries, packages, tools, and runtime dependencies from repository evidence and MUST state explicitly when something required is missing and needs installation, enablement, or integration work.
- The generated spec MUST NOT treat scaffolds, placeholders, or stubs as completed implementation. If the codebase only contains a scaffold, placeholder, or stub, the spec MUST mark it as incomplete current truth and describe the missing real implementation work.
- The skill MUST NOT claim that something exists if it was only proposed in notes.
- The skill MUST separate current implementation truth from proposed future architecture.
- The skill MUST use normative language where appropriate: `MUST`, `SHOULD`, `MAY`, `MUST NOT`, `FAIL CLOSED`.

## 🧭 Output Mode

Interpret `OUTPUT_MODE` as follows:

- `create`: write the target spec only if it does not already exist; if it exists, report that `overwrite` is required.
- `overwrite`: replace or update the target spec with the new canonical draft.
- `review-only`: inspect the source docs, repository evidence, and any existing target spec, then report gaps, conflicts, and the intended structure without writing the file.

If `OUTPUT_MODE` is omitted, default to `create`.

## 🛠️ Workflow

Follow this workflow in order.

### 1. Resolve input context

Identify and record:

- source docs
- target output path
- project root
- reference specs
- style guides
- code evidence paths
- config evidence paths
- test evidence paths
- language and dependency ecosystem
- target scope

The generated spec header MUST record at least:

```markdown
Status:
Date:
Scope:
Canonical artifact:
Source directory:
Output path:
Reference specs:
Owner modules/packages/crates:
```

### 2. Read source documents first

Treat `SOURCE_DIR` as the primary concept source.

Recommended discovery commands:

```bash
find "$SOURCE_DIR" -type f | sort
```

Prioritize:

- `*.md`
- `*.txt`
- `*.yaml`
- `*.yml`
- `*.json`
- `*.toml`
- TODO files
- design drafts
- review notes
- implementation notes
- scenario docs
- test plans
- generated planning artifacts

### 3. Read reference specs only for style

If `REFERENCE_SPECS` is provided, inspect those files only for:

- section structure
- table style
- self-audit style
- config-gate style
- test-matrix style
- diagram style
- normative wording

Do not import project-specific architecture from reference specs unless the current source docs or repository evidence independently justify it.

### 4. Inspect repository evidence before making claims

Inspect code, config, and tests before asserting implementation details.
Also inspect manifests, lockfiles, setup scripts, CI, container files, and tool wiring before claiming that required libraries or tools are already available.

Generic commands:

```bash
find "$PROJECT_ROOT" -maxdepth 4 -type f | sort
rg -n "<key terms from source docs>" "$PROJECT_ROOT" -S
```

Common dependency and tool evidence surfaces:

```bash
find "$PROJECT_ROOT" -maxdepth 4 \( -name "Cargo.toml" -o -name "Cargo.lock" -o -name "pyproject.toml" -o -name "requirements*.txt" -o -name "setup.py" -o -name "package.json" -o -name "pnpm-lock.yaml" -o -name "yarn.lock" -o -name "package-lock.json" -o -name "Dockerfile" -o -name "docker-compose*.yml" -o -path "*/.github/workflows/*" \) -print | sort
rg -n "TODO|stub|placeholder|unimplemented!|todo!\\(|FIXME|temporary implementation|mock implementation" "$PROJECT_ROOT" -S
```

For Rust projects:

```bash
cargo metadata --no-deps --format-version 1
cargo tree -e features
```

For Python projects:

```bash
find "$PROJECT_ROOT" -maxdepth 4 \( -name "pyproject.toml" -o -name "requirements*.txt" -o -name "setup.py" \) -print
```

For TypeScript projects:

```bash
find "$PROJECT_ROOT" -maxdepth 4 \( -name "package.json" -o -name "pnpm-lock.yaml" -o -name "yarn.lock" -o -name "package-lock.json" \) -print
```

The skill MUST determine, for every major module, flow, library, package, and tool dependency, whether it is:

- present and live
- present but partial
- scaffold-only, placeholder, or stub
- missing and requiring explicit installation, enablement, or integration

If something required is missing, the generated spec MUST say concretely what must be installed or added, where it plugs into the codebase, and why it is needed.

### 5. Extract and normalize concepts

Build an internal extraction matrix covering:

- goals
- non-goals
- key terms
- architecture objects
- state transitions
- module ownership
- data ownership
- security assumptions
- trust boundaries
- config fields
- paths
- feature flags
- test requirements
- required libraries, packages, CLIs, services, and tools
- installation and enablement requirements
- scaffold, placeholder, and stub locations
- stale signatures
- outdated names
- conflicting assumptions
- future or deferred work

The final spec SHOULD include:

| Source | Extracted concept | Keep / Modify / Reject | Reason | Target spec section |
| ------ | ----------------- | ---------------------- | ------ | ------------------- |

The final spec MUST also include:

| Component | Present | Partial | Stub | Missing | Evidence path | Required action |
| --------- | ------- | ------- | ---- | ------- | ------------- | --------------- |

| Required capability | Current evidence | Gap | Why gap matters | Fix path |
| ------------------- | ---------------- | --- | ---------------- | -------- |

### 6. Resolve stale and conflicting material explicitly

The generated spec MUST include:

```markdown
## Stale Material And Conflict Resolution
```

With this table:

| Old / conflicting item | Resolution | Reason | Spec impact |
| ---------------------- | ---------- | ------ | ----------- |

Rules:

- preserve useful concepts
- update stale signatures and names if repository evidence shows newer reality
- reject unsafe or obsolete ideas explicitly
- do not silently drop ideas
- do not silently merge contradictory architectures
- if uncertainty remains, mark it as an open risk

### 7. Decide ownership

The generated spec MUST contain an ownership matrix:

| Subsystem / object / flow | Owner module/package/crate | Consumer | MUST own | MUST NOT own |
| ------------------------- | -------------------------- | -------- | -------- | ------------ |

Distinguish, where applicable:

- implementation owner
- data owner
- validation owner
- config owner
- test owner
- runtime owner
- persistence owner
- external adapter owner

If the project has no clear structure, propose one and mark it as proposed.

### 8. Create a real YAML config gate

Every generated spec MUST include a YAML config gate adapted to the current subject.

Minimum structure:

```yaml
version: 1
profile: "<spec-profile-name>"
architecture_mode: "<mode-name>"

features: {}
modules: {}
paths: {}
limits: {}

gates:
  inputs: {}
  outputs: {}
  artifacts: {}
  conditions: {}
  security: {}
  compatibility: {}

retention: {}
fallbacks: {}
observability: {}

tests:
  require_unit_tests: true
  require_integration_tests: true
  require_negative_tests: true
  require_e2e_tests: true
```

Rules:

- the YAML must use meaningful values derived from the current project
- if a value is unknown, mark it explicitly as:

```yaml
value: TBD
decision_required: true
```

- explain every YAML section in the spec
- treat the YAML as a real contract for startup, validation, feature gates, paths, limits, compatibility, security, and tests
- every gate must be concrete and checkable, not a general recommendation
- input gates must define what MUST already be true before processing starts
- output gates must define what artifact, state, or response MUST exist after successful completion
- artifact gates must define required files, records, schemas, digests, or generated outputs
- condition gates must define explicit predicates, invariants, version checks, feature checks, and fail-closed branches
- every gate SHOULD identify owner, evidence, and failure behavior
- every gate MUST be phrased with normative language such as `MUST`, `SHOULD`, `MUST NOT`, or `FAIL CLOSED`

The generated spec SHOULD include a gate table such as:

| Gate id | Gate class | Condition | Evidence | Owner | Failure behavior |
| ------- | ---------- | --------- | -------- | ----- | ---------------- |

### 9. Write the full end-to-end flow

The spec MUST describe the complete lifecycle, not isolated decisions.

Cover:

1. input creation
2. input normalization
3. validation or admission
4. routing or ownership decision
5. processing or state transition
6. storage or persistence
7. publication, export, or API exposure
8. validation or verdict
9. recovery or fallback
10. observability or evidence
11. final acceptance or rejection

Default flow shape:

```text
Input
  -> Normalize
  -> Validate
  -> Process
  -> Persist
  -> Publish / expose
  -> Verify
  -> Finalize / reject
```

### 10. Include Mermaid diagrams

The generated spec MUST include valid Mermaid diagrams that match the text.

Minimum diagrams:

1. high-level architecture or C4/container view
2. component or module view
3. end-to-end sequence
4. lifecycle or state diagram
5. data or storage model
6. requirement or coverage view

If the repository provides `.github/skills/mermaid-c4/SKILL.md`, use `mermaid-c4` for C4-style system context, container, component, code, landscape, dynamic, or deployment views.
If the repository provides `.github/skills/mermaid-spectrum/SKILL.md`, use `mermaid-spectrum` for non-C4 Mermaid packs, mixed-view packs, and supporting flow, sequence, state, class, ER, requirement, or architecture views.
When both are relevant, use `mermaid-c4` for the architecture-level C4 view and `mermaid-spectrum` for the supporting non-C4 views.
If `STYLE_GUIDES` includes additional diagram guidance, read it after the local Mermaid skills and before producing diagrams.
If no diagram skill or guide is available, use standard Mermaid syntax.
Do not draw future systems as if they already exist.

### 11. Cover security, correctness, and trust boundaries

The generated spec MUST include:

```markdown
## Security, Correctness, And Trust Boundaries
```

Cover the relevant topics for the scope, such as:

- canonical encoding
- digest and hash stability
- signature scope
- replay protection
- input validation
- authorization
- privacy
- retention
- auditability
- rollback
- challenge or dispute windows
- compatibility
- mixed-version failure
- unsafe config rejection
- external provider trust boundaries
- adapter boundaries
- fail-closed behavior

If cryptography is relevant, include:

| Primitive / proof / signature / hash | Purpose | Current status | Risk | Test requirement |
| ------------------------------------ | ------- | -------------- | ---- | ---------------- |

Never claim cryptographic safety without implementation evidence, test vectors, or review status.

### 12. Define failure, fallback, and recovery

The generated spec MUST include:

```markdown
## Failure Model
## Fallback And Recovery
```

Required tables:

| Failure | Detection | Required response | Test |
| ------- | --------- | ----------------- | ---- |

| Fallback | When allowed | Owner | MUST preserve | MUST NOT bypass |
| -------- | ------------ | ----- | ------------- | --------------- |

It must define behavior for missing config, malformed input, validation failure, unavailable dependencies, failed writes, stale state, incompatible versions, safe versus unsafe retry, partial progress, and crash recovery.

### 13. Define testing requirements

The generated spec MUST include all of these sections:

```markdown
## Unit Tests
## Integration Tests
## Negative Tests
## Property Tests
## Fuzz Tests
## End-To-End Tests
## Simulation / Scenario Tests
## Regression Tests
```

Represent tests with:

| Test | Owner | Type | Purpose | Required assertion |
| ---- | ----- | ---- | ------- | ------------------ |

Rules:

- include happy-path tests
- include negative tests for every gate and major invariant
- include stale-version and mixed-version tests where relevant
- include deterministic replay or idempotency tests where relevant

### 14. Recommend dependencies only after ecosystem inspection

The dependency section MUST be ecosystem-aware.

For each dependency, include:

| Role | Existing dependency? | Recommended dependency | Use now / later / avoid | Reason | Risk |
| ---- | -------------------- | ---------------------- | ----------------------- | ------ | ---- |

If there are meaningful alternatives, also include:

| Role | Option | Pros | Cons | Why chosen / not chosen |
| ---- | ------ | ---- | ---- | ----------------------- |

The dependency and tooling section MUST also include:

```markdown
## Installation And Enablement
## Rejected Alternatives
```

`Installation And Enablement` MUST include concrete commands when they are justified by the current stack, for example:

- `cargo add ...`
- `pnpm add ...`
- `npm install ...`
- `pip install ...`
- `apt install ...`
- `docker compose ...`

Do not emit commands blindly. Only emit commands that match repository evidence, package manager choice, and the integration path described in the spec.

Rules:

- prefer existing project dependencies first
- prefer standard library or internal modules when sufficient
- avoid dependency sprawl
- do not add hard external-service requirements unless the source docs require them
- mark security-critical dependencies for audit
- mark future dependencies as deferred
- do not paste stale version pins unless verified from local package metadata or lockfiles
- check whether the dependency or tool is already present in manifests, lockfiles, scripts, CI, or local setup before recommending it
- if a required dependency or tool is missing, state explicitly what must be installed, enabled, configured, or wired into the codebase
- include concrete install or enablement guidance when repository evidence shows that the capability is absent
- when alternative libraries, packages, or tools are viable, explain pros and cons and justify why one is chosen over the others for the current stack, constraints, and maintenance posture
- do not recommend placeholder tooling plans without concrete integration points
- `Rejected Alternatives` MUST name the meaningful alternatives that were considered and explain why they were rejected for the current phase, stack, or operational constraints

### 15. Include implementation phases

The spec MUST include:

| Phase | Goal | Work | Exit gate |
| ----- | ---- | ---- | --------- |

Adapt the phases to the current scope, but a typical sequence is:

1. spec and config gate
2. type or object model
3. core validation
4. storage or persistence
5. integration with existing flow
6. negative tests and failure handling
7. e2e or simulation
8. external adapters
9. production hardening

### 16. Include acceptance gates

The spec MUST include:

| Gate | Required evidence | Blocks release if missing |
| ---- | ----------------- | ------------------------- |

Acceptance gates MUST cover:

- config validation
- stable core object encoding
- happy-path success
- fail-closed negative behavior
- integration-path success
- spec and code alignment
- observability evidence
- recovery proof
- tracked security review items

### 17. Include non-negotiable rejections

The generated spec MUST include:

```markdown
## Non-Negotiable Rejections
```

At minimum reject:

- bypassing config gates
- treating future design as current implementation
- weakening validation to pass tests
- using debug strings as canonical bytes
- relying on local paths as authoritative commitments
- skipping negative tests
- skipping retention policy
- bypassing adapter boundaries
- adding dependency sprawl
- silently accepting mixed-version artifacts
- claiming security properties without evidence

### 18. End with architecture doublecheck

The generated spec MUST end with a self-audit matrix:

| Requirement | Covered by section | Doublecheck result |
| ----------- | ------------------ | ------------------ |

Verify:

- every user requirement appears in the spec
- every source concept was kept, modified, or rejected
- every owner is named
- every config field is explained
- every major flow has tests
- every failure mode has a response
- every future item is marked as future or deferred
- every security claim is evidence-bound
- every diagram matches the text

### 19. Run mandatory doublecheck and auto-fix

Before finishing, run a mandatory `doublecheck`-style verification pass over the generated spec and the closing summary.

Check at least:

- missing required sections
- wrong section numbering or misplaced glossary
- unsupported implementation claims
- stale placeholders that can be resolved from local evidence
- mismatched Mermaid diagrams versus surrounding text
- unexplained config fields
- missing ownership assignments
- missing required tests or acceptance gates
- inconsistent current-versus-future wording
- unresolved conflict items that were silently dropped
- weak wording that should be upgraded to `MUST`, `SHOULD`, `MUST NOT`, or `FAIL CLOSED`
- missing or non-checkable gates for inputs, outputs, artifacts, or conditions
- missing `Component Presence Matrix`, `Implementation Gap Matrix`, `Installation And Enablement`, or `Rejected Alternatives`

Rules:

- auto-fix every issue that can be fixed from current source docs, repository evidence, config, or tests
- do not stop after listing issues if the spec can be corrected in the same run
- if an issue cannot be fixed because evidence is missing or contradictory, leave it explicitly as an open risk, unresolved question, or `decision_required`
- the final user-facing summary MUST say what was doublechecked and what was auto-fixed

## 🧱 Required Spec Structure

Use this structure by default unless the source docs clearly justify a different order:

```markdown
# <SPEC_TITLE>

[TOC]

Status:
Date:
Scope:
Canonical artifact:
Source directory:
Output path:
Reference specs:
Owner modules/packages/crates:

## 0. Executive Summary
## 1. Glossary
## 2. Reader Contract
## 3. Current Implementation Truth
## 4. Source Assimilation Matrix
## 5. Stale Material And Conflict Resolution
## 6. Goals And Non-Goals
## 7. Core Architecture Decisions
## 8. Module / Package / Crate Ownership
## 9. Config Gate YAML
## 10. Config Field Contract
## 11. Component Presence Matrix
## 12. Implementation Gap Matrix
## 13. End-To-End Flow
## 14. Object Model
## 15. Lifecycle And State Machine
## 16. Storage / Persistence / Retention
## 17. External Interfaces And Adapter Boundaries
## 18. Security, Correctness, And Trust Boundaries
## 19. Failure Model
## 20. Fallback And Recovery
## 21. Observability And Evidence
## 22. Positive Tests
## 23. Negative Tests
## 24. Property, Fuzz, Simulation, And E2E Tests
## 25. Integration With Existing Codebase
## 26. Dependency Recommendations
## 27. Installation And Enablement
## 28. Rejected Alternatives
## 29. Implementation Phases
## 30. Acceptance Gates
## 31. Non-Negotiable Rejections
## 32. Architecture Doublecheck
## 33. Open Risks And Deferred Work
## 34. Bottom Line
```

## 📝 Output Contract

When invoked, this skill MUST:

- produce or update the target specification file when `OUTPUT_MODE` allows writing
- report the source files read
- report reference specs used for style only
- report code, config, and test evidence inspected
- report which required implementation components are present, partial, scaffold-only, or missing
- report which libraries, packages, and tools must be installed, enabled, or integrated additionally
- report alternative library, package, or tool options considered and why one option was selected over the others
- report rejected alternatives and why they were rejected
- report the output file written or the reason nothing was written
- report the major architectural decisions made
- report the mandatory doublecheck pass and every auto-fixed issue
- report unresolved risks
- report the next recommended implementation steps

If `SPEC_TITLE`, `SOURCE_DIR`, and `OUTPUT_SPEC_PATH` are present, do not ask for confirmation before proceeding.

## ✅ Quality Gates

Before finishing, verify that the generated artifact:

- is self-contained
- preserves relevant source ideas
- distinguishes current implementation from proposed work
- explains the real module and tool implementation path instead of abstract placeholders
- names ownership explicitly
- uses a real YAML config gate
- uses concrete, checkable gates for inputs, outputs, artifacts, and conditions
- uses normative requirement wording with `MUST`, `SHOULD`, `MUST NOT`, and `FAIL CLOSED` where enforcement matters
- includes Mermaid diagrams
- includes positive, negative, integration, e2e, simulation, property, fuzz, and regression tests where applicable
- defines fallback, recovery, migration, and failure behavior
- binds security claims to evidence
- identifies missing libraries, packages, tools, or services and states concrete installation or integration work where needed
- does not accept scaffolds, placeholders, or stubs as completed implementation
- compares viable alternative libraries, packages, or tools and explains the chosen option
- includes `Component Presence Matrix`, `Implementation Gap Matrix`, `Installation And Enablement`, and `Rejected Alternatives`
- completes a mandatory doublecheck pass and auto-fixes all fixable issues
- avoids dependency sprawl
- ends with a self-audit and coverage matrix

## ❌ Non-Negotiable Behavior Of This Skill

This skill itself MUST NOT:

- hardcode any specific project or phase number
- hardcode a blockchain, consensus, storage, proof, or network model
- treat reference specs as architecture authority
- produce a shallow summary instead of a spec
- leave important source ideas outside the generated spec
- invent code paths without evidence
- hide stale assumptions
- present scaffolds, placeholders, or stubs as if they were real completed implementation
- skip YAML config gates
- skip diagrams
- skip test matrices
- skip negative tests
- skip security or trust-boundary analysis
- skip fallback and recovery
- skip the final doublecheck pass
- leave auto-fixable issues unresolved
- omit required install, enablement, or integration work for missing libraries or tools
- choose one library, package, or tool over meaningful alternatives without explaining the tradeoffs and why it was selected
- use soft or vague wording where enforceable `MUST`, `SHOULD`, `MUST NOT`, or `FAIL CLOSED` requirements are needed
- emit generic, non-checkable gates instead of concrete gates for inputs, outputs, artifacts, or conditions
- recommend dependencies without checking the current ecosystem
- claim future or deferred systems are already implemented
- ask unnecessary questions when required inputs are present

## 💡 Examples

Example invocation:

```text
Use drafts-to-spec with:
SOURCE_DIR=docs/drafts/<topic>
OUTPUT_SPEC_PATH=docs/specs/<topic>-spec.md
```

Example with style references:

```text
Use drafts-to-spec with:
SOURCE_DIR=docs/drafts/runtime
OUTPUT_SPEC_PATH=docs/specs/runtime-spec.md
SPEC_SCOPE=runtime
OUTPUT_MODE=overwrite
REFERENCE_SPECS=docs/specs/reference-a.md,docs/specs/reference-b.md
STYLE_GUIDES=.github/skills/mermaid-c4/SKILL.md,.github/skills/mermaid-spectrum/SKILL.md
```
