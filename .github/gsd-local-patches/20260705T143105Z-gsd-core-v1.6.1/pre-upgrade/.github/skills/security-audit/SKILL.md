---
name: security-audit
description: 'Comprehensive security skill for codebases, services, and delivery pipelines. Use when asked to run a security review, security audit, vulnerability scan, trust-boundary review, authentication or authorization audit, dependency audit, static security scan, Semgrep or CodeQL triage, secret scanning setup, push protection bypass remediation, hardcoded-secret detection, pre-commit secret scanning, CI/CD security review, exploitability triage, residual-risk assessment, or zeroization and secure-memory-wipe analysis.'
---

# Security Audit

An AI-powered security scanner that reasons about your codebase the way a human security
researcher would — tracing data flows, understanding component interactions, reviewing
security controls, and catching vulnerabilities that pattern-matching tools miss.

## When to Use This Skill

Use this skill when the request involves:

- Scanning a codebase or file for security vulnerabilities
- Running a security review or vulnerability check
- Checking for SQL injection, XSS, command injection, or other injection flaws
- Finding exposed API keys, hardcoded secrets, or credentials in code
- Auditing dependencies for known CVEs
- Reviewing authentication, authorization, or access control logic
- Detecting insecure cryptography or weak randomness
- Performing a data flow analysis to trace user input to dangerous sinks
- Any request phrasing like "is my code secure?", "scan this file", or "check my repo for vulnerabilities"
- Running `/security-audit` or `/security-audit <path>`

### Structured Audit and Control Review Coverage

- Running a structured security audit, risk assessment, control review, or security
  posture review of a codebase, feature, service, workflow, pipeline, or deployment boundary
- Reviewing data protection, CI/CD security, infrastructure exposure, dependency risk,
  or compliance-relevant control coverage
- Turning architecture, code, control, and operational evidence into prioritized
  remediation and residual-risk decisions

### Static Analysis and SAST Coverage

- Running or planning SAST checks across one or more languages, frameworks, or modules
- Triaging findings from tools such as Semgrep, CodeQL, Bandit, linters, or similar
  static analyzers

### Secret Protection and Push Protection Coverage

- Enabling or configuring secret scanning for a repository or organization
- Setting up push protection to block secrets before they reach the repository
- Defining custom secret patterns with regular expressions
- Resolving a blocked push from the command line
- Triaging, dismissing, or remediating secret scanning alerts
- Configuring delegated bypass for push protection
- Excluding directories from secret scanning via `secret_scanning.yml`
- Enabling validity checks or extended metadata checks
- Scanning local code changes for secrets before committing

### Zeroization and Secret Lifetime Coverage

- Auditing whether passwords, keys, tokens, seeds, session material, plaintext payloads,
  decrypted config, or API credentials are wiped from memory correctly
- Reviewing cleanup on success, error paths, retries, cancellation, and optimized builds

## When Not to Use This Skill

### Audit Boundary Exclusions

- The request is for formal legal advice, certification, or regulator sign-off rather
  than a technical audit
- The request is for intrusive production testing without clear authorization or scope approval

### Static Analysis Boundary Exclusions

- The task is dynamic testing, fuzzing, penetration testing, or exploit validation rather than static analysis
- There is no source code or analyzable build artifact available
- The user only wants architectural threat modeling with no code-level scan

### Zeroization Boundary Exclusions

- The target code does not hold secrets or other sensitive material in process memory
- The task is purely performance tuning or refactoring unrelated to secret cleanup

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

#### GitHub Secret Scanning Operations

Secret scanning automatically detects exposed credentials across:

- entire Git history on all branches
- issue descriptions, comments, and titles
- pull request titles, descriptions, and comments
- discussion content and wiki content

Availability depends on repository ownership and GitHub Secret Protection configuration.

If the request is about repository or organization secret scanning, include these workflows explicitly.

#### Enable Secret Scanning

1. Navigate to repository **Settings** → **Advanced Security**
2. Click **Enable** next to "Secret Protection"
3. Confirm by clicking **Enable Secret Protection**

For organizations, use security configurations to enable at scale.

#### Enable Push Protection

1. Navigate to repository **Settings** → **Advanced Security**
2. Enable "Push protection" under Secret Protection

Push protection blocks secrets in command line pushes, GitHub UI commits,
file uploads, REST API requests, and REST API content creation endpoints.

#### Configure Exclusions

Create `.github/secret_scanning.yml` to auto-close alerts for specific directories:

```yaml
paths-ignore:
  - "docs/**"
  - "test/fixtures/**"
  - "**/*.example"
```

Rules:

- Maximum 1,000 entries in `paths-ignore`
- File must be under 1 MB
- Excluded paths also skip push protection checks

#### Enable Additional Secret Detection Features

- Enable non-provider pattern scanning for private keys and generic connection secrets
- Enable AI detection for unstructured secrets such as passwords
- Enable validity checks to classify detected secrets as `active`, `inactive`, or `unknown`
- Enable extended metadata checks after validity checks when ownership and scope context matters

#### Resolve Blocked Pushes

**If the secret is in the latest commit:**

```bash
# Remove the secret from the file
# Then amend the commit
git commit --amend --all
git push
```

**If the secret is in an earlier commit:**

```bash
# Find the earliest commit containing the secret
git log

# Start interactive rebase before that commit
git rebase -i <COMMIT-ID>~1

# Change 'pick' to 'edit' for the offending commit
# Remove the secret, then:
git add .
git commit --amend
git rebase --continue
git push
```

#### Bypass Push Protection

1. Visit the URL returned in the push error message
2. Select a bypass reason
3. Click **Allow me to push this secret**
4. Re-push within 3 hours

#### Custom Secret Patterns

1. Settings → Advanced Security → Custom patterns → **New pattern**
2. Enter pattern name and regex for secret format
3. Add a sample test string
4. Click **Save and dry run**
5. Review results for false positives
6. Click **Publish pattern**
7. Optionally enable push protection for the pattern

#### Pre-Commit Scanning via AI Coding Agents

For scanning local changes inside an AI coding agent before commit, install the
Advanced Security plugin that provides the `run_secret_scanning` MCP tool.

**GitHub Copilot CLI:**

```bash
/plugin install advanced-security@copilot-plugins
```

**Visual Studio Code:**

- Open **Chat: Plugins** or use `@agentPlugins`
- Install the `advanced-security` plugin
- Run `/secret-scanning` in Copilot Chat when needed

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

#### Injection Flaws

- SQL Injection: raw queries with string interpolation, ORM misuse, second-order SQLi
- XSS: unescaped output, dangerouslySetInnerHTML, innerHTML, template injection
- Command Injection: exec/spawn/system with user input
- LDAP, XPath, Header, Log injection

#### Authentication & Access Control

- Missing authentication on sensitive endpoints
- Broken object-level authorization (BOLA/IDOR)
- JWT weaknesses (alg:none, weak secrets, no expiry validation)
- Session fixation, missing CSRF protection
- Privilege escalation paths
- Mass assignment / parameter pollution

#### Data Handling

- Sensitive data in logs, error messages, or API responses
- Missing encryption at rest or in transit
- Insecure deserialization
- Path traversal / directory traversal
- XXE (XML External Entity) processing
- SSRF (Server-Side Request Forgery)

#### Cryptography

- Use of MD5, SHA1, DES for security purposes
- Hardcoded IVs or salts
- Weak random number generation (Math.random() for tokens)
- Missing TLS certificate validation

#### Business Logic

- Race conditions (TOCTOU)
- Integer overflow in financial calculations
- Missing rate limiting on sensitive endpoints
- Predictable resource identifiers

#### Control Review Areas

Also review these control areas explicitly:

- authentication and session boundaries
- authorization, privilege separation, and administrative controls
- secret handling, encryption expectations, privacy boundaries, and logging exposure
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
4. Downgrade or discard findings that aren't genuine vulnerabilities
5. Assign final severity: CRITICAL / HIGH / MEDIUM / LOW / INFO

#### Finding Confidence States

During self-verification, preserve the distinction between:

- `confirmed issue`
- `likely issue`
- `needs manual review`

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

### Step 10 — Propose Patches

For every CRITICAL and HIGH finding, generate a concrete patch:

- Show the vulnerable code (before)
- Show the fixed code (after)
- Explain what changed and why
- Preserve the original code style, variable names, and structure
- Add a comment explaining the fix inline

Explicitly state: **"Review each patch before applying. Nothing has been changed yet."**

### Step 11 — Optional YOLO Remediation Mode

If and only if the user explicitly requests YOLO mode, convert eligible findings into
applied fixes instead of leaving them as patch proposals only.

YOLO remediation rules:

- Only apply fixes for findings that survived the mandatory `doublecheck` verification gate
- Prefer findings marked `confirmed issue` with High confidence
- Apply the smallest safe change that fully addresses the issue
- Preserve project style, public behavior, and trust boundaries unless the user explicitly asked for a breaking remediation
- Do not auto-apply ambiguous fixes involving broad refactors, policy decisions, production credentials, or uncertain exploit paths
- Skip anything that still needs human judgment and report it as unresolved rather than guessing
- After applying YOLO fixes, run the narrowest reliable validation for the edited scope before continuing

Examples of YOLO-eligible actions:

- removing a clearly hardcoded secret from committed source and switching to environment lookup
- replacing obviously unsafe string-built SQL with parameterized queries
- tightening missing input validation where the safe boundary is already clear in surrounding code
- adding missing secure defaults when the framework contract is explicit and local

Examples that still require human approval:

- rotating live credentials or choosing replacement secret values
- changing authentication policy, authorization model, or production deployment posture
- large cross-module refactors to eliminate deep security design flaws
- remediation where exploitability or compatibility is still uncertain after `doublecheck`

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

## Notes

### Audit Posture Notes

- A security audit is strongest when it combines architecture, code, and control evidence instead of relying on one tool or one checklist
- Re-run the audit when core trust boundaries, deployment paths, or identity flows change materially

### SAST Caveat

- SAST is strongest as an early filter, not as the only security decision
- Static findings become useful only after code-aware triage

### Zeroization Remediation Note

- Prefer root-cause fixes such as reducing copies, shortening lifetime, and using established wipe mechanisms
- When the codebase already has a zeroization abstraction, audit whether all sensitive types consistently go through that abstraction
