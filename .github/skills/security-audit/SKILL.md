---
name: security-audit
description: 'Comprehensive security skill for codebases, services, clients, native components, AI/LLM systems, and delivery pipelines. Use when asked to run a security review, security audit, vulnerability scan, trust-boundary review, authentication or authorization audit, dependency audit, static security scan, Semgrep or CodeQL triage, secret scanning setup, push protection bypass remediation, hardcoded-secret detection, pre-commit secret scanning, CI/CD security review, exploitability triage, residual-risk assessment, or zeroization and secure-memory-wipe analysis.'
---

# Security Audit

An AI-powered security scanner that reasons about your codebase the way a human security
researcher would — tracing data flows, understanding component interactions, reviewing
security controls, and catching vulnerabilities that pattern-matching tools miss.

## When to Use and Scope Boundaries

- Use for source-backed vulnerability, control, dependency, secret, pipeline,
  identity, data-flow, and secure-memory reviews.
- Perform technical, evidence-backed review of source, configuration, workflows,
  dependencies, build artifacts, and explicitly in-scope runtime behavior.
- Do not represent the result as legal advice, certification, regulator sign-off,
  or proof that no vulnerability exists.
- Do not perform intrusive production testing without explicit authorization and scope.
- Treat DAST, fuzzing, and exploit validation as separate branches that require
  suitable authorization and tooling.
- Run the zeroization branch only when the target handles secrets or confidential
  material in process memory.

## How This Skill Works

Unlike traditional static analysis tools that match patterns, this skill:

1. **Reads code like a security researcher** — understanding context, intent, and data flow
2. **Traces across files** — following how user input moves through your application
3. **Self-verifies findings** — re-examines each result to filter false positives
4. **Assigns severity ratings** — CRITICAL / HIGH / MEDIUM / LOW / INFO
5. **Proposes targeted patches** — every finding includes a concrete fix
6. **Requires human approval** — nothing is auto-applied; you always review first

### Extended Capabilities

- Build an evidence surface first using code, config, workflows, manifests,
  scanner output, and incident context before making security claims
- Map assets and trust boundaries so findings are prioritized by real attack exposure
- Report residual risk, not only defects, plus unresolved assumptions and next verification steps
- Build architectural context before deep review by mapping technology stack,
  entry points, components, and trust boundaries
- Treat scanner output as leads, not verdicts, and triage tool findings
  against real code paths
- Audit secret lifetime explicitly when needed, including copies, derived material,
  cleanup paths, and optimization-sensitive wipe behavior
- Select target-specific hunting packs for AI/LLM systems, browser clients,
  HTTP protocol and identity flows, and native or memory-unsafe components
- Require a concrete attacker, boundary crossing, entrypoint-to-sink trace,
  and observable impact before promoting a candidate into a finding

## Audit Lenses

### Security Audit Lenses

- **Scope lens**: what system boundary is actually being audited
- **Asset lens**: what matters if compromised
- **Control lens**: what exists, what is partial, and what is missing
- **Attack-path lens**: how an attacker would chain weaknesses into impact
- **Evidence lens**: what is proven by code or configuration versus assumed
- **Residual-risk lens**: what remains even after proposed fixes

## Execution Workflow

Follow these steps **in order** every time:

### Step 0 — Reconnaissance & Architecture Context

#### Reconnaissance Branch

Before scanning for vulnerabilities, build architectural understanding:

- Inventory README files, architecture docs, security docs, changelogs, and CI/CD files
- Identify the build system and dependency manifests
- Check test layout and repository automation
- Detect language and framework mix
- Build a technology map
- Identify directory structure, entry points, config files, and high-level components
- Map trust boundaries before making vulnerability claims
- Inventory prior audit reports and unresolved leads when available; use them to
  avoid duplicate work and to target unreviewed surfaces, but re-verify any reused claim

Use these reconnaissance patterns when needed:

```bash
# Check for documentation
ls -la README* ARCHITECTURE* SECURITY* CHANGELOG* docs/

# Identify build system
ls package.json Cargo.toml go.mod pyproject.toml Makefile

# Check for tests
ls -la test* spec* *_test* __tests__/

# Identify CI/CD
ls -la .github/workflows/ .gitlab-ci.yml Jenkinsfile .circleci/
```

```bash
# Top-level structure
tree -L 2 -d

# Identify entry points
find . -name "main.py" -o -name "app.py" -o -name "index.ts" -o -name "main.go"

# Identify config
find . -name "config*" -o -name "settings*" -o -name ".env*"
```

For deeper recon, use the bundled support material in this skill folder:

- `resources/recon-checklist.md`
- `resources/question-bank.md`
- `templates/context-document.md`
- `docs/advanced-techniques.md`
- `examples/fastapi-example.md`

### Step 1 — Scope Resolution

Determine what to scan:

- If a path was provided (`/security-audit src/auth/`), scan only that scope
- If no path given, scan the **entire project** starting from the root
- Identify the language(s) and framework(s) in use (check package.json, requirements.txt,
  go.mod, Cargo.toml, pom.xml, Gemfile, composer.json, etc.)
- Read `references/language-patterns.md` to load language-specific vulnerability patterns

#### Audit Goal and Constraints

- State the audit goal explicitly: broad security posture review, pre-release gate,
  control validation, incident-driven review, or compliance-readiness pass
- Record hard constraints such as no intrusive testing, no production exploitation,
  or limited runtime access

### Step 2 — Dependency Audit

Before scanning source code, audit dependencies first (fast wins):

- **Node.js**: Check `package.json` + `package-lock.json` for known vulnerable packages
- **Python**: Check `requirements.txt` / `pyproject.toml` / `Pipfile`
- **Java**: Check `pom.xml` / `build.gradle`
- **Ruby**: Check `Gemfile.lock`
- **Rust**: Check `Cargo.toml`
- **Go**: Check `go.sum`
- Flag packages with known CVEs, deprecated crypto libs, or suspiciously old pinned versions
- Read `references/vulnerable-packages.md` for a curated watchlist

### Step 3 — Secrets & Exposure Scan

Scan ALL files (including config, env, CI/CD, Dockerfiles, IaC) for:

- Hardcoded API keys, tokens, passwords, private keys
- `.env` files accidentally committed
- Secrets in comments or debug logs
- Cloud credentials (AWS, GCP, Azure, Stripe, Twilio, etc.)
- Database connection strings with credentials embedded
- Read `references/secret-patterns.md` for regex patterns and entropy heuristics to apply

#### GitHub Secret Protection Branch

For repository or organization secret protection, load:

- `references/push-protection.md` for prevention, blocked pushes, and bypass governance
- `references/custom-patterns.md` for scoped patterns and dry runs
- `references/alerts-and-remediation.md` for alert triage, validity, rotation,
  history cleanup, and metadata

Verify current product availability and repository settings before making claims.
Treat exclusions and bypasses as trust-boundary changes: require a narrow reason,
owner, review path, and evidence that the excluded material cannot contain live secrets.

### Step 4 — Static Security Scan / SAST

#### SAST Workflow

Before the deeper manual vulnerability pass, run or reconstruct a static scan surface:

1. **Define the scan scope**
   - Identify target files, modules, languages, frameworks, and trust boundaries
   - Record whether the goal is baseline scanning, targeted review, CI gate prep,
     or interpretation of existing findings

2. **Choose the right static analysis surface**
   - Match scanners and rule sets to the actual language and framework mix
   - Distinguish code-pattern scanning, taint analysis, and framework-specific checks

3. **Run or reconstruct the static scan**
   - If tools are available, execute them and capture raw results
   - If not, inspect code manually using SAST thinking: sources, sinks, sanitization,
     access control decisions, dangerous APIs, insecure defaults

4. **Classify the findings by security shape**
   - injection
   - traversal
   - insecure deserialization
   - unsafe command execution
   - secret exposure
   - auth and authorization flaws
   - insecure crypto
   - trust-boundary violations
   - denial-of-service enablers

5. **Triage for real exploitability**
   - Check whether the path is reachable and attacker-influenced
   - Down-rank false positives caused by dead code, safe wrappers, constants,
     or non-user-controlled data
   - Up-rank findings that cross trust boundaries or touch secrets, auth, persistence,
     or command execution

### Step 5 — Vulnerability Deep Scan

This is the core scan. Reason about the code — don't just pattern-match.
Read `references/vuln-categories.md` for full details on each category.
Read `references/hunting-methodology.md` before promoting any candidate into a finding.

Load only the target-specific packs that apply:

- `references/ai-llm-patterns.md` for RAG, tool-calling agents, MCP, model output,
  or any untrusted-text-to-capability flow
- `references/client-side-patterns.md` for browser, SPA, extension, WebView,
  DOM, cross-window messaging, CORS, or WebSocket surfaces
- `references/http-auth-patterns.md` for proxies, caches, custom HTTP parsing,
  JWT, OAuth/OIDC, SAML, sessions, or account recovery
- `references/memory-safety-patterns.md` for C/C++, Rust `unsafe`, FFI,
  parsers, binary formats, kernels, firmware, or privileged native interfaces

Always cover the applicable core classes:

- injection, unsafe sinks, traversal, SSRF, and deserialization
- authentication, authorization, session, and administrative boundaries
- sensitive data, secret handling, privacy, logging, and cryptography
- business logic, ordering, concurrency, numeric, replay, and resource-abuse paths
- pipeline security, artifact integrity, rollout safety, and environment assumptions

### Step 6 — Cross-File Data Flow Analysis

After the per-file scan, perform a **holistic review**:

- Trace user-controlled input from entry points (HTTP params, headers, body, file uploads)
  all the way to sinks (DB queries, exec calls, HTML output, file writes)
- Identify vulnerabilities that only appear when looking at multiple files together
- Check for insecure trust boundaries between services or modules

#### Attack-Path Tracing

- Trace realistic attack paths from entry point to impact for high-value assets or weak controls
- Focus on reachable paths, attacker influence, trust-boundary crossings,
  privilege transitions, and persistence or execution sinks

#### Exploit Evidence Contract

For every candidate finding, record:

1. attacker identity and starting privileges
2. exact input, request, file, call sequence, or race schedule
3. line-referenced trace from entrypoint through propagation to the sink or state change
4. the security boundary crossed and the concrete impact
5. framework, middleware, library, deployment, and downstream controls checked
6. observable result and the narrowest safe reproduction when local execution is feasible

If exploitability depends on a proxy, cache, identity provider, runtime default,
parser behavior, production configuration, or other evidence outside the audited
tree, label it `requires deployment testing`. Do not count it as a confirmed
severity-rated finding until that dependency is verified.

### Step 7 — Zeroization & Secret Lifetime Audit

#### Zeroization Branch

If the code handles cryptography, wallets, authentication, signing, decryption,
or confidential state, run this branch explicitly:

1. **Define the secret inventory**
   - private keys, seeds, passwords, tokens, plaintext payloads, decrypted config,
     API credentials, and derived secret material
   - record every representation: stack value, heap buffer, string, vector,
     struct field, temporary clone, serialized form, encoded copy

2. **Map the secret lifetime**
   - trace where each secret is created, copied, moved, borrowed, transformed,
     cached, logged, serialized, or returned
   - inspect helper functions, conversions, error formatting, and test utilities

3. **Find the intended wipe mechanism**
   - locate the API, wrapper, destructor, or cleanup path that should clear the secret
   - treat a plain zeroing write as insufficient if the build may optimize it away

4. **Check path coverage**
   - success
   - early return
   - error handling
   - retries
   - cancellation
   - panic-like unwind paths
   - shutdown paths

5. **Check copies, aliases, and derived material**
   - clones
   - formatted strings
   - temporary allocations
   - register spills
   - stack slots
   - derived buffers
   - serialization, hex, base64, JSON, debug output, string interpolation

6. **Classify findings clearly**
   - `missing_zeroize`
   - `partial_wipe`
   - `path_gap`
   - `secret_copy`
   - `optimized_away`
   - `retention_risk`

### Step 8 — Self-Verification Pass

For EACH finding:

1. Re-read the relevant code with fresh eyes
2. Ask: "Is this actually exploitable, or is there sanitization I missed?"
3. Check if a framework or middleware already handles this upstream
4. Try to disprove the claimed data flow, boundary crossing, and impact
5. Reproduce parser-, runtime-, race-, or memory-behavior claims when safely feasible
6. Downgrade or discard findings that aren't genuine vulnerabilities
7. Assign final severity from both likelihood and impact: CRITICAL / HIGH / MEDIUM / LOW / INFO

#### Finding Confidence States

During self-verification, preserve the distinction between:

- `confirmed issue`
- `likely issue`
- `needs manual review`

Only `confirmed issue` entries belong in the severity findings table. Put
evidence-backed `likely issue` entries under `needs manual review`, and put
defense-in-depth gaps under `hardening note`. Put candidates blocked on external
or deployment evidence under `requires deployment testing`; do not include any
of these three classes in vulnerability counts.

### Mandatory Gate — `doublecheck` Verification

Before the review is considered complete, run a mandatory `doublecheck` verification pass
against the draft findings set.

- Re-verify the main claims, exploitability statements, and remediation logic with `doublecheck`
- Use it to detect contradictions, unsupported claims, weak evidence, and hallucinated attack paths
- Tighten or remove findings that fail `doublecheck` validation
- If `doublecheck` is unavailable in the current environment, explicitly report that the
  verification gate could not run and treat that as an unresolved validation gap

### Step 9 — Generate Security Report

Output the full report in the format defined in `references/report-format.md`.

#### Required Report Additions

The review output must include:

- Scope and constraints
- Assets and trust boundaries
- Evidence reviewed
- Findings ordered by severity
- Attack-path or control-gap notes for each finding
- Recommended remediation
- Residual risk and next verification step

#### Machine-Readable Findings

When the audit is being delivered as files or the user requests structured output:

1. Write `findings.json` as an array conforming to
   `references/findings-schema.json`
2. Include confirmed and rejected candidates so the verification trail is explicit
3. Run:

```bash
node scripts/validate-findings.mjs findings.json
```

Run the command from this skill directory, or pass absolute paths. Structural
validation does not prove exploitability; the mandatory `doublecheck` gate still applies.

### Step 10 — Propose Patches

For every CRITICAL and HIGH finding, generate a concrete patch:

- Show the vulnerable code (before)
- Show the fixed code (after)
- Explain what changed and why
- Preserve the original code style, variable names, and structure
- Add a comment explaining the fix inline

Explicitly state: **"Review each patch before applying. Nothing has been changed yet."**

### Step 11 — Optional YOLO Remediation Mode

Only enter this mode when the user explicitly requests YOLO remediation. Read
`references/remediation-mode.md`, apply only eligible fixes that survived
`doublecheck`, and validate the edited scope. Leave policy decisions, live
credential rotation, ambiguous exploit paths, and broad security redesigns unresolved.

### Step 12 — Final `doublecheck` Closeout

Before declaring the audit complete, run `doublecheck` one more time on the final state:

- if no code was changed, run it against the final findings and remediation summary
- if YOLO remediation changed code, run it against the applied-fixes summary plus the residual findings set
- remove or downgrade any claim that no longer holds after edits or validation
- if this final `doublecheck` pass cannot run, report the audit as having an open verification gap rather than silently succeeding

## Severity Guide

| Severity | Meaning | Example |
| -------- | ------- | ------- |
| 🔴 CRITICAL | Immediate exploitation risk, data breach likely | SQLi, RCE, auth bypass |
| 🟠 HIGH | Serious vulnerability, exploit path exists | XSS, IDOR, hardcoded secrets |
| 🟡 MEDIUM | Exploitable with conditions or chaining | CSRF, open redirect, weak crypto |
| 🔵 LOW | Best practice violation, low direct risk | Verbose errors, missing headers |
| ⚪ INFO | Observation worth noting, not a vulnerability | Outdated dependency (no CVE) |

## Output Rules

- **Always** produce a findings summary table first (counts by severity)
- **Always** run `doublecheck` before finalizing the findings set or explicitly report that the gate was unavailable
- **Never** auto-apply any patch unless the user explicitly requested YOLO mode and the fix passed the skill's remediation gates
- **Always** include a confidence rating per finding (High / Medium / Low)
- **Group findings** by category, not by file
- **Be specific** — include file path, line number, and the exact vulnerable code snippet
- **Explain the risk** in plain English — what could an attacker do with this?
- **Separate** confirmed/likely findings, hardening notes, rejected candidates,
  and deployment-test leads; do not inflate vulnerability counts with the latter groups
- If the codebase is clean, say so clearly: "No vulnerabilities found" with what was scanned

### Evidence and Confidence Rules

- Do not present a compliance or security claim as fact without supporting code,
  configuration, workflow, or runtime evidence
- Do not keep a finding in the final report if it fails the mandatory `doublecheck` verification pass
- Do not claim YOLO remediation completed successfully until the final `doublecheck` closeout and post-fix validation both pass
- Do not let scanner output replace reasoning about reachability, authorization,
  trust boundaries, or exploitability
- Prefer a smaller set of well-supported findings over a bloated report with low confidence
- Record residual risk and unresolved assumptions explicitly

### Optimized-Away Evidence Rule

- If cleanup claims depend on compiler behavior, do not claim `optimized_away`
  without concrete artifact-based evidence

## Reference Files

For detailed detection guidance, load the following reference files as needed:

- `references/vuln-categories.md` — Deep reference for every vulnerability category with detection signals, safe patterns, and escalation checkers
  - Search patterns: `SQL injection`, `XSS`, `command injection`, `SSRF`, `BOLA`, `IDOR`, `JWT`, `CSRF`, `secrets`, `cryptography`, `race condition`, `path traversal`
- `references/secret-patterns.md` — Regex patterns, entropy-based detection, and CI/CD secret risks
  - Search patterns: `API key`, `token`, `private key`, `connection string`, `entropy`, `.env`, `GitHub Actions`, `Docker`, `Terraform`
- `references/language-patterns.md` — Framework-specific vulnerability patterns for JavaScript, Python, Java, PHP, Go, Ruby, and Rust
  - Search patterns: `Express`, `React`, `Next.js`, `Django`, `Flask`, `FastAPI`, `Spring Boot`, `PHP`, `Go`, `Rails`, `Rust`
- `references/vulnerable-packages.md` — Curated CVE watchlist for npm, pip, Maven, Rubygems, Cargo, and Go modules
  - Search patterns: `lodash`, `axios`, `jsonwebtoken`, `Pillow`, `log4j`, `nokogiri`, `CVE`
- `references/report-format.md` — Structured output template for security reports with finding cards, dependency audit, secrets scan, and patch proposal formatting
  - Search patterns: `report`, `format`, `template`, `finding`, `patch`, `summary`, `confidence`
- `references/hunting-methodology.md` — Attacker-led hunting and the mandatory exploit evidence contract
  - Search patterns: `sad path`, `boundary`, `parser`, `replay`, `evidence`, `hardening`, `deployment testing`
- `references/ai-llm-patterns.md` — AI/LLM, RAG, agent, MCP, tool authority, and output-handling checks
  - Search patterns: `prompt injection`, `tool`, `agency`, `tenant`, `model output`, `MCP`
- `references/client-side-patterns.md` — Browser-only source-to-sink, messaging, WebSocket, CORS, UI-redress, and prototype-pollution checks
  - Search patterns: `DOM`, `postMessage`, `WebSocket`, `CORS`, `clickjacking`, `prototype`
- `references/http-auth-patterns.md` — HTTP framing/cache differentials and authentication-protocol checks
  - Search patterns: `smuggling`, `cache`, `Host`, `JWT`, `OAuth`, `SAML`, `session`, `recovery`
- `references/memory-safety-patterns.md` — Native memory-safety, binary parser, FFI, kernel, and privileged-interface checks
  - Search patterns: `out-of-bounds`, `use-after-free`, `double-fetch`, `type confusion`, `unsafe`, `FFI`
- `references/findings-schema.json` — Strict schema for machine-readable confirmed and rejected findings
- `scripts/validate-findings.mjs` — Zero-dependency structural and trace-order validator for `findings.json`
- `references/remediation-mode.md` — Eligibility and safety rules for explicitly requested YOLO remediation

### Additional Secret Protection Reference Files

- `references/push-protection.md` — Push protection mechanics, bypass workflow, delegated bypass, user push protection
  - Search patterns: `bypass`, `delegated`, `bypass request`, `command line`, `REST API`, `user push protection`
- `references/custom-patterns.md` — Custom pattern creation, regex syntax, dry runs, Copilot regex generation, scopes
  - Search patterns: `custom pattern`, `regex`, `dry run`, `publish`, `organization`, `enterprise`, `Copilot`
- `references/alerts-and-remediation.md` — Alert types, validity checks, extended metadata, generic alerts, secret removal, REST API
  - Search patterns: `user alert`, `partner alert`, `validity`, `metadata`, `generic`, `remediation`, `git history`, `REST API`

### Architecture-Context Reminder

- Use recon-style entry point, component, trust-boundary, and technology-map discovery
  before asserting broad architectural security conclusions
