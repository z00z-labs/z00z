---
name: code-reviewer
description: Research-backed code review skill with OWASP Top 10 security checks, SAST tool integration (SonarQube, CodeQL, Snyk), architecture and reliability review, performance pattern detection, and structured issue reporting. Auto-invoked for code review, security audit, PR analysis, bug checking, maintainability review, and design-level code inspection. Implements 2025 best practices with 92% faster vulnerability remediation.
---

<!--
Created by: Madina Gbotoe (https://madinagbotoe.com/)
Version: 1.0
Created: November 3, 2025
License: Creative Commons Attribution 4.0 International (CC BY 4.0)
Attribution Required: Yes - Include author name and link when sharing/modifying

Purpose: Code Reviewer Skill - Research-backed code review with OWASP 2025, SAST integration, and DevSecOps best practices

Research Backing:
- OWASP Code Review Guide 2025
- OWASP Top 10 2021 (Current Standard)
- CWE Top 25 (2024)
- Empirical Study: arxiv.org/html/2311.16396v2 (135,560 code reviews analyzed)
- NIST Secure Software Development Framework
- DevSecOps automation research (92% faster remediation with continuous review)
-->

# Code Reviewer Skill

Comprehensive code review skill implementing **2025 research-backed best practices** with automated security checks, performance analysis, and quality standards enforcement.

## Core Philosophy

**Balanced Quality + Security Approach:**
- 50% Security focus (OWASP Top 10, vulnerabilities, authentication)
- 30% Code quality (maintainability, standards, duplication)
- 20% Performance (N+1 queries, algorithm complexity, bundle size)

**Research Finding:** Teams with continuous code review fix vulnerabilities **92% faster** than traditional batch reviews.

**Execution Principle:** Use a silent checklist internally, then report only concrete issues with evidence. Avoid speculative warnings, avoid repeating the same issue across multiple categories, and prioritize high-impact findings first.

---

## When to Use This Skill

**Auto-invoked when user mentions:**
- "review this code"
- "check for bugs"
- "security audit"
- "analyze this PR"
- "code review"
- "check code quality"
- "review my changes"
- "find vulnerabilities"
- "performance check"

**Manual invocation:**
- Before committing critical changes
- Pre-deployment validation
- After implementing security-sensitive features
- When integrating SAST tool results

---

## Core Review Workflow

**NEW TO CODE REVIEW?** See `EXAMPLES.md` for complete walkthrough examples showing good vs bad reviews.

### Review Mode

Use this execution mode by default:

1. Run automated checks first when available.
2. Scan the code against the topic matrix below as an internal checklist.
3. Report only issues that are real, specific, and evidenced in code.
4. Keep findings short, precise, and ordered by severity.
5. If a topic is clean and the user wants a full review, say so explicitly instead of padding with theory.

**Default limits:**
- Prefer the top 5 highest-impact issues when the review surface is large.
- Avoid more than 5 findings per topic unless the user asked for exhaustive output.
- Do not invent risks like "there may be a race" without a concrete path, shared state, or code location.

### Phase 1: Automated Analysis (Run First)

**Step 1: Identify Code to Review**
- If reviewing specific files: Read those files
- If reviewing PR/changes: Use `git diff` to see changes
- If reviewing entire feature: Identify affected files via grep/glob

**Step 2: Run Automated SAST Tools (If Available)**

Use the scripts in `scripts/` directory:
```bash
# Quick security audit
bash scripts/quick-audit.sh

# Or run individual tools:
npm audit                    # Dependency vulnerabilities
npm run lint                 # ESLint code quality
npx prettier --check .       # Code formatting

# Advanced (if installed):
sonar-scanner               # SonarQube
codeql database analyze     # CodeQL
snyk test                   # Snyk security
```

**Step 3: Parse SAST Results**
- Categorize findings by severity (Critical/High/Medium/Low)
- Filter false positives (document reasoning)
- Cross-reference with manual checks

### Phase 2: Manual Security Analysis

**Check OWASP Top 10 2021 (Current Standard):**

1. **A01:2021 – Broken Access Control**
   - Are authentication checks on all protected routes?
   - Is user input validated before authorization decisions?
   - Are direct object references protected (IDOR prevention)?

2. **A03:2021 – Injection (SQL, NoSQL, Command)**
   - Are parameterized queries used instead of string concatenation?
   - Is user input sanitized before database queries?
   - Are ORMs used correctly (no raw queries with user input)?

3. **A03:2021 – Cross-Site Scripting (XSS)**
   - Is user input escaped before rendering in HTML?
   - Are Content Security Policy headers configured?
   - Is `dangerouslySetInnerHTML` avoided or properly sanitized?

4. **A07:2021 – Identification and Authentication Failures**
   - Are passwords hashed with bcrypt/argon2 (not MD5/SHA1)?
   - Is session management secure (httpOnly cookies, CSRF tokens)?
   - Is rate limiting implemented on login endpoints?

5. **A02:2021 – Cryptographic Failures**
   - Is sensitive data encrypted at rest and in transit?
   - Are API keys/secrets stored in environment variables (not hardcoded)?
   - Is HTTPS enforced for all external communication?

**See REFERENCE.md for complete OWASP Top 10 checklist**

### Phase 3: Performance Pattern Detection

**Check for Common Performance Issues:**

1. **N+1 Query Problem**
   ```typescript
   // 🔴 BAD: N+1 queries
   users.forEach(user => {
     db.query("SELECT * FROM posts WHERE user_id = ?", user.id)
   })

   // ✅ GOOD: Single query with JOIN
   db.query("SELECT * FROM users LEFT JOIN posts ON users.id = posts.user_id")
   ```

2. **O(n²) or Worse Algorithms**
   - Nested loops over large datasets
   - Inefficient sorting/searching
   - Recursive functions without memoization

3. **Missing Database Indexes**
   - Queries on unindexed columns
   - WHERE clauses without supporting indexes
   - JOIN operations on unindexed foreign keys

4. **Memory Leaks**
   - Event listeners not cleaned up
   - Closures holding large objects
   - Unbounded caches

5. **Large Bundle Sizes**
   - Importing entire libraries instead of specific functions
   - Unoptimized images
   - Missing code splitting

**See REFERENCE.md for performance pattern catalog**

### Phase 4: Code Quality Standards

**Check TypeScript/JavaScript Standards:**
- [ ] No `any` types without justification comment
- [ ] Proper error handling (no empty catch blocks)
- [ ] No `console.log` statements (use proper logging)
- [ ] Functions < 50 lines (extract if larger)
- [ ] Cyclomatic complexity < 10
- [ ] No commented-out code
- [ ] Import order follows convention (React → Third-party → Internal → Relative)

**Check Naming Conventions:**
- Files: `kebab-case` (user-service.ts)
- Components: `PascalCase` (UserProfile.tsx)
- Functions/Variables: `camelCase` (getUserData)
- Constants: `UPPER_SNAKE_CASE` (MAX_RETRIES)

**See REFERENCE.md for complete standards checklist**

### Phase 4.5: Architecture And Reliability Matrix

Use these topics as the internal review matrix after automation. They extend the original security-focused workflow so the review can cover architecture, domain fit, and operational quality without becoming vague.

| Topic | What to Inspect | Primary Category |
| --- | --- | --- |
| A | Architecture and design boundaries, layering, hidden coupling, god modules | architecture |
| B | Modularity, single responsibility, reusable seams, helper sprawl | modularity |
| C | Complexity, naming clarity, nesting, overly clever code | complexity |
| D | Algorithms, data structures, clones/allocations, hot paths | optimization |
| E | Parallelism, async boundaries, race/deadlock/cancellation risks | parallelization |
| F | Error handling, edge cases, failure cascades, invariant boundaries | reliability |
| G | Security, input trust, secrets, crypto use, DoS controls | security |
| H | Testability, determinism, dependency seams, coverage realism | testability |
| I | Logging, metrics, traces, reconstructability, privacy-safe observability | observability |
| J | Maintainability, extensibility, technical debt, compatibility drift | maintainability |
| K | Domain correctness, invariants, contract fidelity, state transition safety | domain_correctness |
| L | Documentation, comments, contract notes, spec references | documentation |
| M | Style consistency, naming consistency, pattern uniformity | style_consistency |
| N | Dependencies, features, environment assumptions, config centralization | dependencies_and_environment |

**How to use the matrix:**
- Use the matrix to discover issues, not to force 14 separate essay sections.
- If multiple topics match one issue, report it once under the strongest category.
- For security-sensitive or crypto-sensitive projects, explicitly overlay project policies before making style-only suggestions.
- If a codebase has strong project rules, prefer those rules over generic best practices.

### Phase 5: Generate Review Report

Use templates from `resources/templates/` to create structured output:
- **Comprehensive Review:** Use `resources/templates/code-review-report.md`
- **Security Audit:** Use `resources/templates/security-review-template.md`
- **Performance Review:** Use `resources/templates/performance-review-template.md`
- **Quick Check:** Use `resources/templates/quick-checklist.md`

**Example report format:**

```markdown
# Code Review Report

**Verdict:** ✅ APPROVED | ⚠️ APPROVED WITH RESERVATIONS | ❌ REQUIRES REVISION
**Files Reviewed:** [List]
**Review Date:** [ISO Date]

## Critical Issues (Must Fix Before Merge)
[None or list with code snippets]

## High Priority Issues (Fix Within 48h)
[None or list with recommendations]

## Medium Priority Issues (Fix This Sprint)
[None or list]

## Low Priority / Suggestions
[Optional improvements]

## Strengths & Good Practices
[What was done well]

## Metrics
- **Lines Changed:** X
- **Files Modified:** Y
- **Estimated Risk:** Low/Medium/High
- **Test Coverage:** Z%
```

### Structured Finding Contract

When the user wants a rigorous review, format each finding with these fields:

- `severity`: `high`, `medium`, or `low`
- `location`: file + function/method + line range when available
- `category`: one of the review focus areas
- `topic_id`: one of `A` through `N`
- `related_questions`: checklist question indices when useful
- `description`: what is wrong
- `rationale`: why it matters
- `suggestion`: concrete improvement

Use a short summary after the findings:
- top high-impact issues
- quick wins
- future risks
- clean topics worth noting

---

## Integration with SAST Tools

### SonarQube Integration

**If SonarQube results available:**
1. Read `sonar-report.json` or access SonarQube API
2. Focus manual review on issues SonarQube missed:
   - Business logic vulnerabilities
   - Context-specific security issues
   - Authorization logic
3. Validate SonarQube findings (check false positives)

**Key SonarQube Metrics to Review:**
- **Security Hotspots:** Require manual validation
- **Code Smells:** Maintainability issues (threshold: Grade A or B)
- **Duplications:** Keep < 3%
- **Coverage:** Target > 80%

### CodeQL Integration

**If CodeQL scan available:**
1. Review CodeQL alerts in GitHub Security tab
2. Prioritize alerts by severity:
   - **Critical/High:** Must fix before merge
   - **Medium:** Fix within sprint
   - **Low:** Backlog
3. Use CodeQL's suggested fixes when available
4. Manual review for:
   - Authentication flows
   - Authorization decisions
   - Cryptographic operations

**CodeQL Strengths (88% accuracy):**
- SQL injection detection
- Path traversal vulnerabilities
- Command injection
- Insecure deserialization

### Snyk Integration

**If Snyk results available:**
1. Run `snyk test` for dependency vulnerabilities
2. Run `snyk code test` for code-level issues
3. Prioritize by:
   - **Critical:** Fix immediately
   - **High:** Fix within 7 days
   - **Medium:** Fix within 30 days
4. Check for available patches: `snyk wizard`

**Snyk Strengths:**
- Dependency vulnerabilities (real-time CVE database)
- License compliance
- Container security
- Infrastructure as Code (IaC) scanning

### ESLint + Prettier + npm audit

**Basic Security Stack (Always Run):**
```bash
# Run these three commands ALWAYS
npm audit --audit-level=high    # Dependency vulnerabilities
npm run lint                    # Code quality (ESLint)
npx prettier --check .          # Code formatting
```

**Interpretation:**
- **npm audit:** Fix all high/critical vulnerabilities
- **ESLint:** Must pass with 0 errors (warnings acceptable if documented)
- **Prettier:** Auto-fix with `npx prettier --write .`

---

## Severity Classification

Use this framework to categorize all findings:

### 🔴 Critical (Blocks Deployment)
- SQL injection vulnerabilities
- XSS vulnerabilities (unescaped user input)
- Authentication bypass
- Hardcoded secrets/API keys
- Remote code execution (RCE) risks
- Sensitive data logged in plain text

**Action:** STOP. Must fix immediately before proceeding.

### 🟠 High (Fix Within 48 Hours)
- Missing authentication checks
- Insecure session management
- CSRF vulnerabilities
- Missing rate limiting on sensitive endpoints
- Weak cryptography (MD5, SHA1)
- N+1 query problems in critical paths
- Memory leaks in production code

**Action:** Create blocker ticket. Fix before next deployment.

### 🟡 Medium (Fix This Sprint)
- Missing input validation (non-critical fields)
- Inefficient algorithms (O(n²) on small datasets)
- Missing database indexes (< 1000 rows)
- Code duplication (> 5 occurrences)
- Missing error handling
- Accessibility violations (WCAG AA)

**Action:** Create ticket. Fix within current sprint.

### 🔵 Low (Backlog / Nice-to-Have)
- Code style violations (if not enforced by linter)
- Missing comments on complex code
- Minor performance optimizations
- Refactoring opportunities
- Documentation improvements

**Action:** Optional. Add to backlog for future improvement.

---

## Key Metrics to Track

**From 2025 Research:**

1. **Mean Time to Remediate (MTTR)**
   - Target: < 7 days for high severity issues
   - Critical issues: < 24 hours

2. **Defect Density**
   - Formula: (# of bugs) / (1000 lines of code)
   - Target: < 1.0 defects per 1000 LOC

3. **Review Coverage**
   - Target: 100% of changed lines reviewed
   - Critical paths: 100% manual review (not just automated)

4. **False Positive Rate**
   - CodeQL: ~5% (best in class)
   - SonarQube: ~8-10%
   - Snyk: ~8%
   - Track your project's rate to calibrate trust

---

## Common Pitfalls to Avoid

1. **Over-reliance on Automation**
   - Automated tools catch 60-70% of security issues
   - Manual review essential for business logic, authorization, context-specific issues

2. **Ignoring Performance for Security**
   - A secure but unusable app is not secure (DoS via performance)
   - Balance security checks with performance impact

3. **Blocking Every Minor Issue**
   - Use severity classification to prioritize
   - Don't let perfection block progress

4. **Missing the Forest for the Trees**
   - Step back and review overall architecture
   - Check if the approach is fundamentally sound

5. **Not Checking Test Coverage**
   - Untested code = unreviewed code
   - Require tests for all security-critical paths

6. **Turning The Review Into A Questionnaire**
   - The checklist is for inspection, not for dumping every question into the output
   - Final review comments should contain findings, not a transcript of the audit process

7. **Missing Project-Specific Rules**
   - Crypto, storage, compliance, and API-contract projects often have stricter invariants than generic app code
   - Respect repository policies before recommending "cleanups" that weaken guarantees

---

## Quick Reference Checklist

Before approving any code review:

**Security:**
- [ ] No SQL injection vulnerabilities (parameterized queries)
- [ ] No XSS vulnerabilities (user input escaped)
- [ ] Authentication checks on protected routes
- [ ] Secrets in environment variables (not hardcoded)
- [ ] HTTPS enforced for external APIs
- [ ] CSRF protection on state-changing endpoints

**Performance:**
- [ ] No N+1 query patterns
- [ ] No O(n²) or worse algorithms on large datasets
- [ ] Database indexes present for queried columns
- [ ] No memory leaks (event listeners cleaned up)
- [ ] Images optimized and lazy-loaded

**Code Quality:**
- [ ] TypeScript strict mode compliance (no `any` without justification)
- [ ] Error handling present (no empty catch blocks)
- [ ] No console.log statements
- [ ] Functions < 50 lines
- [ ] Test coverage > 80%
- [ ] No commented-out code

**Documentation:**
- [ ] Complex logic has explanatory comments
- [ ] Public APIs have JSDoc comments
- [ ] README updated if behavior changed
- [ ] Environment variables documented

---

## Next Steps After Review

**If APPROVED (✅):**
1. Merge the PR
2. Monitor deployment for issues
3. Update test coverage metrics

**If APPROVED WITH RESERVATIONS (⚠️):**
1. Create tickets for medium/low priority issues
2. Merge if critical/high issues are fixed
3. Schedule follow-up review

**If REQUIRES REVISION (❌):**
1. Provide detailed feedback with code snippets
2. Block merge until critical issues resolved
3. Offer to pair-program on fixes if needed

---

## Supporting Files

**For detailed information, see:**
- **EXAMPLES.md** - Complete end-to-end code review examples (good, bad, security-focused)
- **REFERENCE.md** - Complete OWASP Top 10 checklist, performance patterns, CWE references
- **FORMS.md** - Review report templates overview and guidance
- **resources/templates/** - Ready-to-use review templates
  - `code-review-report.md` - Comprehensive review template
  - `security-review-template.md` - Security-focused audit template
  - `performance-review-template.md` - Performance analysis template
  - `quick-checklist.md` - 3-minute rapid review checklist
- **resources/examples/** - Real-world review examples
  - `good-review-example.md` - What a thorough review looks like
  - `bad-review-example.md` - What to avoid (rubber stamp reviews)
- **scripts/** - Automated SAST tool integration scripts
  - `quick-audit.sh` - Quick security audit (Linux/Mac)
  - `quick-audit.bat` - Quick security audit (Windows)

**Research Citations:**
- OWASP Code Review Guide: https://owasp.org/www-project-code-review-guide/
- OWASP Top 10 2021: https://owasp.org/Top10/
- Empirical Study (2024): https://arxiv.org/html/2311.16396v2
- CWE Top 25: https://cwe.mitre.org/top25/

---

**Remember:** Code review is not about finding fault—it's about ensuring quality, security, and maintainability. Be constructive, specific, and always suggest solutions alongside identifying problems.
