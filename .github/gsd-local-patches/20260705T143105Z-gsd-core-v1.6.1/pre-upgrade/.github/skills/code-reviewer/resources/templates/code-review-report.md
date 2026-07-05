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

[If none, write "None identified ✅"]

### Issue Title 🔴

**Location:** `file.ts:line`

**Issue:**
```language
[Code snippet showing the problem]
```

**Why This is Critical:**
[Explanation of the security/functional impact]

**Recommendation:**
```language
[Code snippet showing the fix]
```

**Severity:** 🔴 Critical
**Category:** [Security | Performance | Functionality]
**OWASP/CWE:** [If applicable]

---

## High Priority Issues (Fix Within 48 Hours)

[If none, write "None identified ✅"]

### Issue Title 🟠

**Location:** `file.ts:line`

**Issue:**
[Description]

**Recommendation:**
[Specific fix with code example if needed]

**Severity:** 🟠 High
**Category:** [Security | Performance | Code Quality]

---

## Medium Priority Issues (Fix This Sprint)

[If none, write "None identified ✅"]

### Issue Title 🟡

**Location:** `file.ts:line`

**Issue:**
[Description]

**Recommendation:**
[Specific fix]

**Severity:** 🟡 Medium

---

## Low Priority / Suggestions (Backlog)

[If none, write "None identified ✅"]

### Issue Title 🔵

**Location:** `file.ts:line`

**Suggestion:**
[Description and recommendation]

---

## Strengths & Good Practices

✅ [What was done well - be specific]
✅ [Good patterns observed]
✅ [Security best practices followed]
✅ [Performance optimizations]

---

## Detailed Analysis

### Security Review (OWASP Top 10)
- **A01 - Broken Access Control:** [Status]
- **A02 - Cryptographic Failures:** [Status]
- **A03 - Injection:** [Status]
- **A07 - Authentication Failures:** [Status]
- **A05 - Security Misconfiguration:** [Status]

### Performance Analysis
- **Database Queries:** [No N+1 issues | Issues found]
- **Algorithm Complexity:** [O(n) | Issues found]
- **Memory Usage:** [Efficient | Concerns noted]

### Code Quality
- **TypeScript Compliance:** [Strict mode | Issues]
- **Test Coverage:** [X%]
- **Complexity:** [Functions < 50 lines | Issues]
- **Error Handling:** [Proper | Missing in places]

---

## SAST Tool Results

[If tools were run, include results]

### npm audit
- **Critical:** X
- **High:** Y
- **Medium:** Z

### ESLint
- **Errors:** X
- **Warnings:** Y

### TypeScript
- **Type Errors:** X

---

## Recommendations Summary

**Before Merge:**
1. [Action item with estimated time]
2. [Action item with estimated time]

**After Merge (Follow-up):**
1. [Optional improvement]
2. [Optional refactoring]

**Total Estimated Fix Time:** X hours/minutes

---

## Next Steps

**If Approved (✅):**
1. Merge the PR
2. Monitor deployment
3. Update metrics

**If Approved with Reservations (⚠️):**
1. Fix critical/high issues
2. Re-review changes
3. Merge when fixes confirmed

**If Requires Revision (❌):**
1. Author: Address all critical issues
2. Author: Push fixes
3. Reviewer: Full re-review required

---

## Sign-off

**Status:** [✅ APPROVED | ⚠️ APPROVED WITH RESERVATIONS | ❌ REQUIRES REVISION]

[Final summary paragraph with overall assessment and confidence level]

**Reviewer Signature:** [Your Name]
**Date:** [YYYY-MM-DD]
