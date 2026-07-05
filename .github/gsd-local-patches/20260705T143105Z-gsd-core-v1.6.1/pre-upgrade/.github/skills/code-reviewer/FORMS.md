# Code Reviewer - Templates & Forms

**Ready-to-use templates for code review reports, checklists, and documentation.**

Use these templates to ensure consistent, thorough code reviews across your team.

---

## 📁 Template Files Location

**Full templates are available in `resources/templates/` directory:**

- **`code-review-report.md`** - Comprehensive review template (most common use case)
- **`security-review-template.md`** - Security-focused audit with OWASP Top 10 checklist
- **`performance-review-template.md`** - Performance analysis with metrics and benchmarks
- **`quick-checklist.md`** - 3-minute rapid review checklist for small changes

**Examples are available in `resources/examples/` directory:**

- **`good-review-example.md`** - Complete walkthrough of a thorough, helpful review
- **`bad-review-example.md`** - What NOT to do (rubber stamp reviews)

---

## Table of Contents

1. [Template Usage Guide](#template-usage-guide)
2. [Code Review Report Template (Overview)](#code-review-report-template-overview)
3. [Quick Review Checklist (Overview)](#quick-review-checklist)
4. [Security-Focused Review Template (Overview)](#security-focused-review-template)
5. [Performance-Focused Review Template (Overview)](#performance-focused-review-template)
6. [PR Comment Templates](#pr-comment-templates)
7. [SAST Integration Report](#sast-integration-report)

---

## Template Usage Guide

**Which template should I use?**

| Scenario | Template | Location |
| -------- | -------- | -------- |
| Standard PR review | Code Review Report | `resources/templates/code-review-report.md` |
| Security audit before deployment | Security Review | `resources/templates/security-review-template.md` |
| Performance optimization review | Performance Review | `resources/templates/performance-review-template.md` |
| Quick pre-commit check | Quick Checklist | `resources/templates/quick-checklist.md` |
| Learn by example | Good/Bad Examples | `resources/examples/*.md` |

**How to use:**

1. Copy the appropriate template from `resources/templates/`
2. Fill in the sections as you review
3. Customize severity thresholds for your project
4. Save as part of PR comment or documentation

---

## Code Review Report Template (Overview)

**Full template:** `resources/templates/code-review-report.md`

**Use for:** Comprehensive code reviews, PR reviews, pre-deployment audits

```markdown
# Code Review Report: [Feature/PR Name]

**Reviewed by:** [Your Name]
**Date:** [YYYY-MM-DD]
**Commit/PR:** [#123 or commit hash]
**Review Type:** [Standard | Security Audit | Performance Review | Pre-Deployment]

---

## Executive Summary

**Verdict:** ✅ APPROVED | ⚠️ APPROVED WITH RESERVATIONS | ❌ REQUIRES REVISION

**Overview:**
[2-3 sentence summary of what was reviewed and overall quality]

**Key Metrics:**
- **Files Modified:** X
- **Lines Changed:** +Y/-Z
- **Estimated Risk:** 🟢 Low | 🟡 Medium | 🔴 High
- **Test Coverage:** X% (target: >80%)
- **Review Duration:** X minutes

---

## Critical Issues (Blocking - Must Fix Before Merge)

| # | Severity | Location | Issue | Recommendation |
|---|----------|----------|-------|----------------|
| 1 | 🔴 Critical | `file.ts:45` | SQL injection vulnerability in login query | Use parameterized queries: `db.query(sql, [params])` |
| 2 | 🔴 Critical | `auth.ts:23` | Hardcoded API key in source code | Move to environment variable: `process.env.API_KEY` |

**Total Critical Issues:** X

---

## High Priority Issues (Fix Within 48 Hours)

| # | Severity | Location | Issue | Recommendation |
|---|----------|----------|-------|----------------|
| 1 | 🟠 High | `api.ts:67` | Missing rate limiting on login endpoint | Implement express-rate-limit with 5 attempts/15min |
| 2 | 🟠 High | `user.ts:89` | N+1 query problem in user list | Use JOIN or DataLoader for batching |

**Total High Priority Issues:** X

---

## Medium Priority Issues (Fix This Sprint)

| # | Severity | Location | Issue | Recommendation |
|---|----------|----------|-------|----------------|
| 1 | 🟡 Medium | `utils.ts:34` | Missing input validation on email field | Add Zod schema validation |
| 2 | 🟡 Medium | `dashboard.tsx:120` | Component exceeds 300 lines | Extract subcomponents |

**Total Medium Priority Issues:** X

---

## Low Priority / Suggestions (Backlog)

| # | Severity | Location | Issue | Recommendation |
|---|----------|----------|-------|----------------|
| 1 | 🔵 Low | `types.ts:12` | Consider using branded types for IDs | Implement branded types for type safety |
| 2 | 🔵 Low | `README.md:45` | Typo in documentation | Fix spelling |

**Total Low Priority Issues:** X

---

## Strengths & Good Practices

**What was done well:**
- ✅ Comprehensive test coverage (85%)
- ✅ Clear naming conventions followed
- ✅ Proper error handling throughout
- ✅ Documentation updated alongside code
- ✅ [Add more specific positive observations]

---

## Detailed Analysis

### Security Assessment

**OWASP Top 10 Check:**
- [x] A01: Access Control - ✅ Proper authentication on all routes
- [x] A02: Cryptography - ✅ Secrets in environment variables
- [x] A03: Injection - ⚠️ Found 1 SQL injection vulnerability (see critical issues)
- [x] A03: XSS - ✅ User input properly escaped
- [x] A04: Insecure Design - ✅ Rate limiting implemented
- [x] A05: Security Misconfiguration - ✅ Production config secured
- [x] A06: Vulnerable Components - ✅ No known CVEs in dependencies
- [x] A07: Authentication - ✅ Proper session management
- [x] A08: Data Integrity - ✅ Input validation present
- [x] A09: Logging - ✅ Security events logged (no sensitive data)
- [x] A10: SSRF - ✅ URL validation on external requests

**Overall Security Rating:** 🟢 Good | 🟡 Needs Improvement | 🔴 Critical Issues

---

### Performance Assessment

**Identified Issues:**
- [ ] N+1 query patterns: [Describe]
- [ ] Algorithm complexity > O(n log n): [Describe]
- [ ] Missing database indexes: [Describe]
- [ ] Memory leaks: [None detected]
- [ ] Large bundle size additions: [Describe impact]

**Performance Impact:** 🟢 Negligible | 🟡 Moderate | 🔴 Significant

---

### Code Quality Assessment

**Metrics:**
- **Cyclomatic Complexity:** Average X (target: <10)
- **Function Length:** Average X lines (target: <50)
- **Code Duplication:** X% (target: <3%)
- **TypeScript Coverage:** X% (target: 100%)

**Standards Compliance:**
- [x] TypeScript strict mode enabled
- [x] ESLint passes with 0 errors
- [x] Prettier formatting applied
- [x] No `console.log` statements
- [x] Proper import order
- [x] Naming conventions followed

---

### Test Coverage Analysis

**Current Coverage:** X%

**Missing Tests:**
- [ ] `function1()` - Edge case: empty input
- [ ] `function2()` - Error handling path
- [ ] `ComponentA` - User interaction: button click

**Test Quality:**
- ✅ Unit tests present
- ✅ Integration tests present
- ⚠️ E2E tests missing for critical flow
- ✅ Mock data follows production schema

---

## SAST Tool Results

**Tools Run:**
- ✅ npm audit: 0 high/critical vulnerabilities
- ✅ ESLint: 0 errors, 2 warnings (acceptable)
- ✅ SonarQube: Quality Gate PASSED (Grade A)
- ✅ Snyk: 0 high/critical vulnerabilities

**False Positives Filtered:** X
**Verified Issues:** Y

---

## Recommendations

### Immediate Actions (Before Merge)

1. **[Critical Issue #1]**
   - **File:** `file.ts:45`
   - **Fix:** [Specific code change]
   - **Effort:** X minutes
   - **Example:**
     ```typescript
     // Before (vulnerable)
     const sql = `SELECT * FROM users WHERE id = '${userId}'`;

     // After (secure)
     const sql = 'SELECT * FROM users WHERE id = ?';
     db.query(sql, [userId]);
     ```

2. **[Critical Issue #2]**
   - [Same format]

---

### Short-term Improvements (This Sprint)

1. **[High Priority Issue #1]** - [Brief description]
2. **[Medium Priority Issue #1]** - [Brief description]

---

### Long-term Considerations (Backlog)

1. **Refactoring Opportunities** - [Describe technical debt]
2. **Performance Optimization** - [Describe future improvements]
3. **Architecture Improvements** - [Describe design enhancements]

---

## Next Steps

**For Developer:**
1. [ ] Address all critical issues
2. [ ] Create tickets for high/medium priority issues
3. [ ] Update tests to cover new edge cases
4. [ ] Request re-review after fixes

**For Reviewer:**
1. [ ] Follow up after fixes applied
2. [ ] Verify critical issues resolved
3. [ ] Approve merge if all blockers addressed

---

## Sign-off

**Reviewed by:** [Name]
**Approval Status:** [Approved | Approved with Reservations | Rejected]
**Next Review Required:** [Yes/No - If yes, when?]

---

_Generated using Code Reviewer Skill v1.0_
_Review completed in X minutes_
```

## Structured Findings Form

Use this when the review needs strict, machine-friendly findings instead of a narrative summary.

```markdown
## Findings

### Finding 1
- Severity: high
- Location: src/module.ts :: handleRequest :: lines 40-67
- Category: security
- Topic ID: G
- Related Questions: [1, 3]
- Description: SQL is built with interpolated user input.
- Rationale: This allows injection and bypasses query parameterization guarantees.
- Suggestion: Replace string interpolation with parameterized queries and add a regression test.

### Finding 2
- Severity: medium
- Location: src/cache.ts :: warmCache :: lines 12-48
- Category: optimization
- Topic ID: D
- Related Questions: [1, 4]
- Description: Nested scans over the full item list create quadratic behavior.
- Rationale: This is likely to degrade sharply under real production cardinality.
- Suggestion: Pre-index items by id before the merge loop.
```

## Clean Topics Summary

Use this after the findings when you want to make the coverage explicit without writing filler.

```markdown
## Clean Topics

- A Architecture and design: No significant issues found.
- H Testability: No significant issues found.
- I Observability: No significant issues found.
```

## Topic Matrix Cheat Sheet

Use the topic ids consistently:

- `A` architecture
- `B` modularity
- `C` complexity
- `D` optimization
- `E` parallelization
- `F` reliability
- `G` security
- `H` testability
- `I` observability
- `J` maintainability
- `K` domain_correctness
- `L` documentation
- `M` style_consistency
- `N` dependencies_and_environment

---

## Quick Review Checklist

**Use for:** Fast reviews, pre-commit checks, daily code reviews

```markdown
# Quick Code Review Checklist

**File/Feature:** _______________
**Reviewer:** _______________
**Date:** _______________

## Security (30 seconds)
- [ ] No hardcoded secrets/API keys
- [ ] User input sanitized/validated
- [ ] Authentication present on protected routes
- [ ] No SQL injection (parameterized queries used)
- [ ] Passwords hashed (bcrypt/argon2, not MD5)

## Performance (30 seconds)
- [ ] No N+1 query patterns
- [ ] No nested loops over large datasets
- [ ] Database indexes present for queried columns
- [ ] No synchronous file operations (use async)

## Code Quality (60 seconds)
- [ ] TypeScript strict mode (no `any` without justification)
- [ ] Functions < 50 lines
- [ ] No commented-out code
- [ ] Proper error handling (no empty catch blocks)
- [ ] No console.log statements

## Tests (30 seconds)
- [ ] Unit tests present for new functions
- [ ] Test coverage > 80%
- [ ] Edge cases tested
- [ ] Tests pass locally

## Documentation (30 seconds)
- [ ] Complex logic has comments
- [ ] README updated if needed
- [ ] Public APIs have JSDoc comments

## Total Time: 3 minutes

**Decision:**
- ✅ APPROVED - Ready to merge
- ⚠️ APPROVED WITH NOTES - Create follow-up tickets
- ❌ CHANGES REQUIRED - Address issues before merge

**Notes:**
_______________________________________
_______________________________________
```

---

## Security-Focused Review Template

**Use for:** Security audits, pre-deployment security checks, sensitive features

```markdown
# Security-Focused Code Review

**Feature:** _______________
**Reviewer:** _______________
**Date:** _______________
**Risk Level:** 🟢 Low | 🟡 Medium | 🔴 High | ⚫ Critical

---

## OWASP Top 10 2021 Detailed Check

### A01: Broken Access Control
- [ ] Authentication required on ALL protected endpoints?
- [ ] Authorization checks verify user owns resource?
- [ ] Direct object references validated (IDOR prevention)?
- [ ] File path traversal prevented (`../` blocked)?
- [ ] API rate limiting implemented?

**Findings:** _______________
**Risk:** 🟢 🟡 🔴 ⚫

---

### A02: Cryptographic Failures
- [ ] HTTPS enforced for all external communication?
- [ ] Passwords hashed with bcrypt/argon2 (NOT MD5/SHA1)?
- [ ] Secrets in environment variables (not hardcoded)?
- [ ] Sensitive data encrypted at rest?
- [ ] TLS 1.2+ enforced (no SSL, TLS 1.0, TLS 1.1)?

**Findings:** _______________
**Risk:** 🟢 🟡 🔴 ⚫

---

### A03: Injection
- [ ] Parameterized queries used (no string concatenation)?
- [ ] User input sanitized before DB operations?
- [ ] Command injection prevented (no `exec()` with user input)?
- [ ] NoSQL injection prevented?
- [ ] XML injection prevented (XXE attacks)?

**Findings:** _______________
**Risk:** 🟢 🟡 🔴 ⚫

---

### A03: Cross-Site Scripting (XSS)
- [ ] User input escaped before rendering in HTML?
- [ ] Content Security Policy (CSP) headers configured?
- [ ] `dangerouslySetInnerHTML` avoided or sanitized (DOMPurify)?
- [ ] JSON responses have proper Content-Type?

**Findings:** _______________
**Risk:** 🟢 🟡 🔴 ⚫

---

### A07: Authentication Failures
- [ ] Session tokens cryptographically random?
- [ ] Session expiration implemented (timeout)?
- [ ] Multi-factor authentication available?
- [ ] Account lockout after failed attempts?
- [ ] Password reset tokens expire after use?

**Findings:** _______________
**Risk:** 🟢 🟡 🔴 ⚫

---

### A09: Logging and Monitoring
- [ ] Security events logged (login, access denied)?
- [ ] Sensitive data NOT logged (passwords, tokens)?
- [ ] Logs include timestamp, user ID, IP, action?
- [ ] Alerting configured for suspicious activity?

**Findings:** _______________
**Risk:** 🟢 🟡 🔴 ⚫

---

## Threat Modeling

**Attack Vectors Considered:**
- [ ] SQL Injection
- [ ] XSS
- [ ] CSRF
- [ ] Authentication bypass
- [ ] Authorization bypass
- [ ] Session hijacking
- [ ] Brute force attacks
- [ ] API abuse
- [ ] Data exfiltration

**Highest Risk Attack:** _______________
**Mitigation Status:** _______________

---

## Security Test Results

**Automated Scans:**
- npm audit: ___ vulnerabilities (Critical: ___, High: ___)
- Snyk: ___ vulnerabilities (Critical: ___, High: ___)
- SonarQube Security Rating: ___

**Manual Testing:**
- [ ] Attempted SQL injection - Result: ___
- [ ] Attempted XSS - Result: ___
- [ ] Attempted authentication bypass - Result: ___
- [ ] Attempted IDOR attack - Result: ___

---

## Overall Security Assessment

**Risk Level:** 🟢 Low | 🟡 Medium | 🔴 High | ⚫ Critical

**Recommendation:**
- ✅ APPROVED - Security requirements met
- ⚠️ CONDITIONAL APPROVAL - Minor issues, create tickets
- ❌ REJECTED - Critical security issues, must fix before deployment

**Critical Issues to Address:**
1. _______________
2. _______________
3. _______________
```

---

## Performance-Focused Review Template

**Use for:** Performance reviews, optimization audits, high-traffic features

```markdown
# Performance-Focused Code Review

**Feature:** _______________
**Reviewer:** _______________
**Date:** _______________
**Expected Load:** ___ requests/second

---

## Database Performance

**Query Analysis:**
- [ ] N+1 query patterns identified: ___
- [ ] Missing indexes identified: ___
- [ ] Query complexity analyzed (EXPLAIN PLAN)
- [ ] Connection pooling configured properly

**Findings:**
- Slowest query: ___ ms (target: <100ms)
- Query count per request: ___ (target: <10)
- Recommended indexes: ___

---

## Algorithm Complexity

**Functions Analyzed:**

| Function | Current Complexity | Dataset Size | Acceptable? |
|----------|-------------------|--------------|-------------|
| `function1()` | O(n²) | 1000 items | ❌ |
| `function2()` | O(n log n) | 10000 items | ✅ |

**Optimization Recommendations:**
1. _______________
2. _______________

---

## Memory Usage

**Potential Leaks:**
- [ ] Event listeners cleaned up
- [ ] Closures releasing references
- [ ] Streams properly closed
- [ ] Caches have max size limits

**Findings:** _______________

---

## Bundle Size Impact

**Added to Bundle:**
- New dependencies: ___ KB
- New code: ___ KB
- Total impact: ___ KB (target: <50KB per feature)

**Optimization Opportunities:**
- [ ] Tree-shaking enabled
- [ ] Code splitting implemented
- [ ] Lazy loading used for heavy components
- [ ] Images optimized (WebP, proper sizing)

---

## Frontend Performance

**React-Specific:**
- [ ] Unnecessary re-renders identified
- [ ] Heavy components memoized (React.memo)
- [ ] Expensive calculations memoized (useMemo)
- [ ] Virtual scrolling for long lists

**Findings:** _______________

---

## Load Testing Results

**Metrics:**
- Response time (p50): ___ ms (target: <200ms)
- Response time (p95): ___ ms (target: <500ms)
- Response time (p99): ___ ms (target: <1000ms)
- Throughput: ___ req/s (target: > ___ req/s)
- Error rate: ___% (target: <1%)

**Bottlenecks Identified:**
1. _______________
2. _______________

---

## Overall Performance Assessment

**Performance Grade:** A | B | C | D | F

**Recommendation:**
- ✅ APPROVED - Performance acceptable
- ⚠️ APPROVED WITH MONITORING - Watch metrics in production
- ❌ REQUIRES OPTIMIZATION - Performance unacceptable

**Critical Optimizations Needed:**
1. _______________
2. _______________
```

---

## PR Comment Templates

**Use for:** Inline code review comments on GitHub/GitLab

### Security Issue Template

```markdown
🔴 **Security Issue: [Issue Type]**

**Risk:** Critical | High | Medium | Low
**CWE:** [CWE-XXX](https://cwe.mitre.org/data/definitions/XXX.html)

**Problem:**
[Explain the vulnerability]

**Exploit Scenario:**
[How an attacker could exploit this]

**Recommended Fix:**
\```typescript
// Before (vulnerable)
[vulnerable code]

// After (secure)
[secure code]
\```

**References:**
- [OWASP link]
- [CVE link if applicable]
```

### Performance Issue Template

```markdown
⚠️ **Performance Issue: [Issue Type]**

**Impact:** High | Medium | Low
**Estimated Overhead:** [X ms per request | Y MB memory | Z% CPU]

**Problem:**
[Explain the performance issue]

**Recommendation:**
\```typescript
// Current implementation (O(n²))
[current code]

// Optimized version (O(n))
[optimized code]
\```

**Expected Improvement:** [X% faster | Y MB less memory]
```

### Code Quality Issue Template

```markdown
💡 **Code Quality: [Issue Type]**

**Priority:** High | Medium | Low

**Issue:**
[Describe what's wrong]

**Why It Matters:**
[Explain impact on maintainability, readability, or testing]

**Suggested Improvement:**
\```typescript
// Current
[current code]

// Suggested
[improved code]
\```

**Alternative Approaches:**
- [Option 1]
- [Option 2]
```

### Praise Template

```markdown
✨ **Great Work!**

[Specific thing done well]

This is a great example of [best practice] because [reason]. Keep it up!
```

---

## SAST Integration Report

**Use for:** Consolidating results from multiple SAST tools

```markdown
# SAST Integration Report

**Project:** _______________
**Scan Date:** _______________
**Scanned By:** _______________

---

## Tools Run

- [x] npm audit
- [x] ESLint
- [x] Prettier
- [x] SonarQube
- [x] CodeQL
- [x] Snyk
- [ ] Other: _______________

---

## Consolidated Results

### Critical Findings (Across All Tools)

| Tool | Rule | File:Line | Issue | Status |
|------|------|-----------|-------|--------|
| Snyk | CVE-2021-XXXX | package.json | Vulnerable lodash version | 🔄 In Progress |
| CodeQL | js/sql-injection | api.ts:45 | SQL injection risk | ✅ Fixed |

**Total Critical:** X

---

### High Priority Findings

| Tool | Rule | File:Line | Issue | Status |
|------|------|-----------|-------|--------|
| SonarQube | squid:S2068 | config.ts:12 | Hardcoded password | 🔄 In Progress |

**Total High Priority:** X

---

### False Positives (Validated and Dismissed)

| Tool | Rule | File:Line | Reason for Dismissal |
|------|------|-----------|----------------------|
| CodeQL | js/xss | render.tsx:67 | React auto-escapes, safe by default |

**Total False Positives:** X

---

## Tool-Specific Details

### npm audit
- **Critical:** X
- **High:** X
- **Fixable Automatically:** X
- **Requires Manual Update:** X

**Command to fix:** `npm audit fix`

---

### SonarQube
- **Quality Gate:** PASSED | FAILED
- **Bugs:** X (Grade: A/B/C/D/E)
- **Vulnerabilities:** X (Grade: A/B/C/D/E)
- **Code Smells:** X (Grade: A/B/C/D/E)
- **Coverage:** X% (Target: >80%)
- **Duplication:** X% (Target: <3%)

**Dashboard:** [Link to SonarQube]

---

### CodeQL
- **Alerts:** X
- **Critical:** X
- **High:** X
- **Medium:** X
- **Low:** X

**Most Common Issues:**
1. [Issue type] - X occurrences
2. [Issue type] - X occurrences

---

### Snyk
- **Critical:** X
- **High:** X
- **Medium:** X
- **Low:** X

**Vulnerable Packages:**
1. [package@version] - [CVE-XXXX-XXXX]
2. [package@version] - [CVE-XXXX-XXXX]

**Fix Available:** Yes/No
**Command to fix:** `snyk fix`

---

## Summary

**Overall Security Posture:** 🟢 Good | 🟡 Needs Improvement | 🔴 Critical Issues

**Recommendation:**
- ✅ READY FOR DEPLOYMENT - All critical issues resolved
- ⚠️ DEPLOY WITH MONITORING - Monitor for issues
- ❌ BLOCK DEPLOYMENT - Critical issues must be fixed

**Next Actions:**
1. [ ] Fix critical issues
2. [ ] Create tickets for high/medium issues
3. [ ] Re-run scans after fixes
4. [ ] Update documentation

---

_Report generated by Code Reviewer Skill v1.0_
_Scan completed in X minutes_
```

---

**These templates should be customized based on your team's specific needs and processes.**

**Last Updated:** November 3, 2025
