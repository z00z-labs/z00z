You are a skill-builder. Create a universal local skill named:

```
drafts-to-spec
```

Target output:

```
.github/skills/drafts-to-spec/SKILL.md
```

## Purpose

Create a reusable skill that generates full, self-contained, architecture-grade specification documents from a folder or set of planning documents.

The skill MUST be universal.

It MUST NOT be hardcoded to any single project, phase number, architecture, crate name, module name, blockchain, storage model, consensus model, or previous document.

Reference documents provided by the user are FORMAT EXAMPLES only. They may be used to learn document style, depth, structure, rigor, and quality level, but their project-specific content MUST NOT be copied into unrelated specs unless the current source docs or repository evidence justify it.

## What the skill does

The skill transforms scattered notes, TODO files, design drafts, code references, configs, and tests into one canonical single-source-of-truth specification.

The generated spec MUST:

1. Be self-contained.
2. Preserve all relevant ideas from the input documents.
3. Resolve outdated signatures, stale terminology, and conflicting assumptions.
4. Separate current implementation truth from proposed future architecture.
5. Define module/crate/package ownership.
6. Define complete end-to-end flow, not isolated point decisions.
7. Include a real YAML config gate for architecture parameters, paths, feature flags, limits, and enforcement gates.
8. Actively use that YAML config throughout the spec.
9. Include positive, negative, integration, e2e, simulation, property, fuzz, and regression tests where applicable.
10. Include fallback, recovery, migration, and failure behavior.
11. Include security, data integrity, cryptography, validation, and trust-boundary analysis where applicable.
12. Recommend dependencies only after checking the current project stack.
13. Avoid dependency sprawl.
14. Use MUST / SHOULD / MAY / MUST NOT / FAIL CLOSED language.
15. Include diagrams using available diagram skills or standard Mermaid.
16. End with a self-audit and coverage matrix.

## Skill inputs

The skill MUST accept these generic variables:

```text
SOURCE_DIR=<directory containing planning docs or drafts>
OUTPUT_SPEC_PATH=<where to write the generated spec>
SPEC_SCOPE=<feature|module|protocol|storage|runtime|api|security|testing|deployment|general>
OUTPUT_MODE=<create|overwrite|review-only>
```

Minimum required inputs:

```text
SOURCE_DIR
OUTPUT_SPEC_PATH
```

If required inputs are missing, the skill MAY ask for clarification.

If required inputs are present, the skill MUST proceed without asking unnecessary questions.

## Universal behavior rule

The skill MUST infer architecture from:

1. User request.
2. Source documents.
3. Repository evidence.
4. Existing configs.
5. Existing tests.
6. Reference specs only as style examples.

The skill MUST NOT hardcode:

- phase numbers;
- feature names;
- blockchain names;
- crate/module/package names;
- consensus models;
- storage engines;
- proof systems;
- specific external services;
- specific test scenario names;
- project-specific paths;
- previous user decisions.

Any specific architecture term may appear in the generated spec only if it is found in the current source docs, codebase evidence, config, or explicit user request.

## Required workflow

### Step 1 — Resolve input context

The skill MUST identify:

- source docs;
- target output path;
- project root;
- available reference specs;
- available style guides;
- code evidence paths;
- config evidence paths;
- test evidence paths;
- language/dependency ecosystem;
- target scope.

The skill MUST record these in the generated spec header.

### Step 2 — Read source documents

The skill MUST recursively read the source directory.

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
- implementation notes
- review notes
- generated artifacts
- scenario docs
- test plans

The skill MUST treat the source directory as the primary concept source.

### Step 3 — Read reference specs only for format

If `REFERENCE_SPECS` are provided, the skill MAY inspect them for:

- document depth;
- section structure;
- table style;
- self-audit style;
- config-gate style;
- test-matrix style;
- diagram style;
- normative language style.

The skill MUST NOT copy project-specific architecture from reference specs unless the current source docs also support that architecture.

The generated skill MUST explicitly warn future agents:

```text
REFERENCE_SPECS are style examples, not architecture authority.
```

### Step 4 — Inspect repository evidence

The skill MUST inspect repository evidence before making implementation claims.

Generic commands:

```bash
find "$PROJECT_ROOT" -maxdepth 4 -type f | sort
rg -n "<key terms from source docs>" "$PROJECT_ROOT" -S
```

For Rust projects:

```bash
cargo metadata --no-deps --format-version 1
cargo tree -e features
```

For Python projects:

```bash
find . -maxdepth 4 \( -name "pyproject.toml" -o -name "requirements*.txt" -o -name "setup.py" \) -print
```

For TypeScript projects:

```bash
find . -maxdepth 4 \( -name "package.json" -o -name "pnpm-lock.yaml" -o -name "yarn.lock" -o -name "package-lock.json" \) -print
```

The skill MUST distinguish:

| Evidence type   | Meaning                                       |
| --------------- | --------------------------------------------- |
| Existing code   | Current implementation truth.                 |
| Existing tests  | Current behavior evidence.                    |
| Existing config | Current operational or planned gate evidence. |
| Source docs     | Design intent.                                |
| Reference specs | Style only.                                   |
| Missing code    | Proposed work only.                           |

The skill MUST NOT claim that something exists if it was only proposed in notes.

### Step 5 — Extract and normalize concepts

The skill MUST create an internal extraction matrix.

The final spec SHOULD include a version of this matrix:

| Source | Extracted concept | Keep / Modify / Reject | Reason | Target spec section |
| ------ | ----------------- | ---------------------- | ------ | ------------------- |
|        |                   |                        |        |                     |

The skill MUST identify:

- core goals;
- non-goals;
- key terms;
- architecture objects;
- state transitions;
- module ownership;
- data ownership;
- security assumptions;
- trust boundaries;
- configs;
- paths;
- feature flags;
- test requirements;
- stale signatures;
- outdated names;
- conflicting assumptions;
- future/deferred work.

### Step 6 — Resolve stale or conflicting material

The skill MUST include a section:

```markdown
## Stale Material And Conflict Resolution
```

It MUST explain:

| Old / conflicting item | Resolution | Reason | Spec impact |
| ---------------------- | ---------- | ------ | ----------- |
|                        |            |        |             |

Rules:

- Preserve concepts where useful.
- Update signatures and module names if code evidence shows newer reality.
- Reject unsafe or obsolete ideas explicitly.
- Do not silently drop ideas.
- Do not silently merge contradictory architectures.
- If uncertain, mark the issue as an open risk instead of pretending it is solved.

### Step 7 — Decide architecture ownership

The skill MUST produce an ownership matrix.

Generic format:

| Subsystem / object / flow | Owner module/package/crate | Consumer | MUST own | MUST NOT own |
| ------------------------- | -------------------------- | -------- | -------- | ------------ |
|                           |                            |          |          |              |

The skill MUST infer ownership from the current project.

If the project has no clear module structure, the skill MUST propose one and mark it as proposed.

The skill MUST distinguish:

- implementation owner;
- data owner;
- validation owner;
- config owner;
- test owner;
- runtime owner;
- persistence owner;
- external adapter owner.

### Step 8 — Create YAML config gate

Every generated spec MUST include a YAML config gate.

The YAML MUST be adapted to the current spec subject.

It MUST include at least:

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

The skill MUST NOT use meaningless placeholder values in the final spec.

If a value is unknown, it MUST mark it explicitly:

```yaml
value: TBD
decision_required: true
```

The spec MUST explain every YAML section.

The YAML MUST be treated as a real implementation contract:

- startup gate;
- config validation gate;
- runtime feature gate;
- test fixture source;
- path and limits authority;
- compatibility gate;
- security policy gate.

### Step 9 — Write full end-to-end flow

The skill MUST include a complete flow section.

It MUST describe:

1. Input creation.
2. Input normalization.
3. Validation/admission.
4. Routing or ownership decision, if relevant.
5. State transition or processing.
6. Storage/persistence.
7. Publication/export/API response, if relevant.
8. Validation/verdict.
9. Recovery/fallback.
10. Observability/evidence.
11. Final acceptance or rejection.

Use this format:

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

The actual names MUST be adapted to the current project.

### Step 10 — Diagrams

The skill MUST include Mermaid diagrams.

Minimum required diagrams:

1. C4/container or high-level architecture diagram.
2. Component/module diagram.
3. End-to-end sequence diagram.
4. Lifecycle/state diagram.
5. Data model or storage model diagram.
6. Requirement or coverage diagram.

If diagram style skills are provided in `STYLE_GUIDES`, the skill MUST read them before producing diagrams.

If no diagram skill is available, the skill MUST use standard Mermaid syntax.

The diagrams MUST be valid Mermaid.

The diagrams MUST match the spec content.

The skill MUST NOT include diagrams that claim future systems already exist.

### Step 11 — Security and correctness

The generated spec MUST include a section:

```markdown
## Security, Correctness, And Trust Boundaries
```

It MUST cover relevant items for the current spec, such as:

- canonical encoding;
- digest/hash stability;
- signature scope;
- replay protection;
- input validation;
- authorization;
- privacy;
- data retention;
- auditability;
- rollback;
- challenge/dispute window;
- version compatibility;
- mixed-version failure;
- unsafe config rejection;
- external provider trust boundaries;
- adapter boundaries;
- fail-closed behavior.

If cryptography is relevant, the spec MUST include:

| Primitive / proof / signature / hash | Purpose | Current status | Risk | Test requirement |
| ------------------------------------ | ------- | -------------- | ---- | ---------------- |
|                                      |         |                |      |                  |

The skill MUST NOT claim cryptographic safety without implementation evidence, test vectors, and review status.

### Step 12 — Failure, fallback, and recovery

The generated spec MUST include:

```markdown
## Failure Model
## Fallback And Recovery
```

Required tables:

| Failure | Detection | Required response | Test |
| ------- | --------- | ----------------- | ---- |
|         |           |                   |      |

| Fallback | When allowed | Owner | MUST preserve | MUST NOT bypass |
| -------- | ------------ | ----- | ------------- | --------------- |
|          |              |       |               |                 |

The skill MUST define what happens when:

- config is missing or invalid;
- input is malformed;
- validation fails;
- dependency is unavailable;
- storage write fails;
- external provider is unavailable;
- state is stale;
- version is incompatible;
- retry is safe;
- retry is unsafe;
- partial progress exists;
- recovery resumes after crash.

### Step 13 — Tests

The generated spec MUST include full testing requirements.

Required sections:

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

Each test SHOULD be represented as:

| Test | Owner | Type | Purpose | Required assertion |
| ---- | ----- | ---- | ------- | ------------------ |
|      |       |      |         |                    |

The skill MUST include negative tests for every gate and major invariant.

The skill MUST include positive tests for the happy path.

The skill MUST include stale-version and mixed-version tests where applicable.

The skill MUST include deterministic replay or idempotency tests where applicable.

### Step 14 — Dependency recommendations

The skill MUST recommend dependencies only after inspecting the current ecosystem.

The dependency section MUST be generic and ecosystem-aware.

For each dependency:

| Role | Existing dependency? | Recommended dependency | Use now / later / avoid | Reason | Risk |
| ---- | -------------------- | ---------------------- | ----------------------- | ------ | ---- |
|      |                      |                        |                         |        |      |

Rules:

- Prefer existing project dependencies.
- Prefer standard library or existing internal modules when sufficient.
- Avoid adding many dependencies.
- Avoid adding external services as hard requirements unless the source docs require them.
- Mark security-critical dependencies for audit.
- Mark future dependencies as deferred.
- Recheck current versions at implementation time.
- Do not paste outdated version pins unless verified from the local lockfile or current package metadata.

### Step 15 — Implementation phases

The generated spec MUST include implementation phases.

Generic format:

| Phase | Goal | Work | Exit gate |
| ----- | ---- | ---- | --------- |
|       |      |      |           |

Phases SHOULD progress from:

1. Spec and config gate.
2. Type/object model.
3. Core validation.
4. Storage/persistence.
5. Integration with existing flow.
6. Negative tests and failure handling.
7. E2E/simulation.
8. External adapters, if any.
9. Production hardening.

The exact phases MUST be adapted to the current spec.

### Step 16 — Acceptance gates

The generated spec MUST include clear acceptance gates.

Format:

| Gate | Required evidence | Blocks release if missing |
| ---- | ----------------- | ------------------------- |
|      |                   |                           |

Acceptance gates MUST include:

- config validates;
- core objects have stable encoding;
- happy path passes;
- negative tests fail closed;
- integration path works;
- docs/spec matches code;
- observability exists;
- recovery works;
- security review items are tracked.

### Step 17 — Non-negotiable rejections

The generated spec MUST include:

```markdown
## Non-Negotiable Rejections
```

This section MUST list designs or implementations that MUST NOT be accepted.

Examples of generic rejection patterns:

- bypassing config gates;
- treating future design as current implementation;
- weakening validation to pass tests;
- using debug strings as canonical bytes;
- relying on local paths as authoritative commitments;
- skipping negative tests;
- skipping retention policy;
- bypassing adapter boundaries;
- adding external dependency sprawl;
- silently accepting mixed-version artifacts;
- claiming security properties without evidence.

### Step 18 — Architecture doublecheck

The generated spec MUST end with a self-audit.

Required matrix:

| Requirement | Covered by section | Doublecheck result |
| ----------- | ------------------ | ------------------ |
|             |                    |                    |

The skill MUST verify:

- every user requirement appears in the spec;
- every source concept was kept, modified, or rejected;
- every module owner is named;
- every config field is explained;
- every major flow has tests;
- every failure mode has a response;
- every future item is marked future/deferred;
- every security claim is evidence-bound;
- every diagram matches the text.

## Required generated spec structure

The skill SHOULD use this default structure unless the source docs clearly require another structure:

```markdown
# <SPEC_TITLE>

Status:
Date:
Scope:
Canonical artifact:
Source directory:
Output path:
Reference specs:
Owner modules/packages/crates:

## 0. Executive Summary
## 1. Reader Contract
## 2. Current Implementation Truth
## 3. Source Assimilation Matrix
## 4. Stale Material And Conflict Resolution
## 5. Glossary
## 6. Goals And Non-Goals
## 7. Core Architecture Decisions
## 8. Module / Package / Crate Ownership
## 9. Config Gate YAML
## 10. Config Field Contract
## 11. End-To-End Flow
## 12. Object Model
## 13. Lifecycle And State Machine
## 14. Storage / Persistence / Retention
## 15. External Interfaces And Adapter Boundaries
## 16. Security, Correctness, And Trust Boundaries
## 17. Failure Model
## 18. Fallback And Recovery
## 19. Observability And Evidence
## 20. Positive Tests
## 21. Negative Tests
## 22. Property, Fuzz, Simulation, And E2E Tests
## 23. Integration With Existing Codebase
## 24. Dependency Recommendations
## 25. Implementation Phases
## 26. Acceptance Gates
## 27. Non-Negotiable Rejections
## 28. Architecture Doublecheck
## 29. Open Risks And Deferred Work
## 30. Bottom Line
```

## Skill output behavior

When invoked, the skill MUST produce or update the target specification file.

It MUST also report:

- source files read;
- reference specs used for style only;
- code/config/test evidence inspected;
- output file written;
- major architectural decisions made;
- unresolved risks;
- next recommended implementation steps.

The skill MUST NOT ask for confirmation when `SOURCE_DIR`, `OUTPUT_SPEC_PATH`, and `SPEC_TITLE` are provided.

## Generic invocation example

```text
Use drafts-to-spec with:
SOURCE_DIR=.planning/phases/<PHASE-ID>-<PHASE-NAME>
OUTPUT_SPEC_PATH=.planning/phases/<PHASE-ID>-<PHASE-NAME>/<PHASE-ID>-<PHASE-NAME>-Spec.md
```

## Non-negotiable behavior of the skill itself

The skill MUST NOT:

- hardcode any specific project;
- hardcode any specific phase number;
- hardcode any specific blockchain, consensus, storage, proof, or network model;
- treat reference specs as architecture authority;
- produce a shallow summary;
- leave important source ideas outside the generated spec;
- invent code paths without evidence;
- hide stale assumptions;
- skip config YAML;
- skip diagrams;
- skip test matrices;
- skip negative tests;
- skip security and trust-boundary analysis;
- skip fallback and recovery;
- recommend dependencies without checking the current project ecosystem;
- claim future/deferred systems are currently implemented;
- ask unnecessary questions when required inputs are present.

The generated skill must make `drafts-to-spec` a universal SSoT-specification builder for any feature, phase, module, protocol, storage layer, API, security model, or deployment design.