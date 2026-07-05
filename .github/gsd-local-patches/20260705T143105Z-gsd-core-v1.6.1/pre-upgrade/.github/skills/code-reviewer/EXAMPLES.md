# Code Review Examples

Complete end-to-end examples showing how to use the code-reviewer skill effectively.

---

## Table of Contents

1. [Example 1: Good Comprehensive Code Review](#example-1-good-comprehensive-code-review)
2. [Example 2: Security-Focused Review](#example-2-security-focused-review)
3. [Example 3: Bad Review (What NOT to Do)](#example-3-bad-review-what-not-to-do)

---

## Example 1: Good Comprehensive Code Review

### Context
**Feature:** User authentication API endpoint
**Files Changed:** `src/api/auth.ts`, `src/middleware/validate.ts`
**Lines Changed:** +150/-30
**Review Type:** Pre-merge PR review

### User Request
```
"Review this authentication PR for security, performance, and code quality issues"
```

### Review Process

**Step 1: Run Automated Tools**
```bash
bash scripts/quick-audit.sh
```

**Results:**
- ✅ npm audit: No vulnerabilities
- ✅ ESLint: 0 errors, 2 warnings
- ✅ TypeScript: All type checks pass
- ⚠️ Prettier: 3 files need formatting

**Step 2: Manual Security Review (OWASP Top 10)**

Checked `src/api/auth.ts`:

**A01 - Broken Access Control:**
- ✅ Authentication middleware present on all protected routes
- ✅ Token validation checks user permissions
- ✅ Direct object references prevented with user ID validation

**A02 - Cryptographic Failures:**
- ✅ Passwords hashed with bcrypt (cost factor 12)
- ✅ JWT secrets in environment variables
- ✅ HTTPS enforced in production

**A03 - Injection:**
- ✅ Parameterized queries used (TypeORM)
- ✅ Input validation with Zod schemas
- ✅ No string concatenation in SQL

**A07 - Authentication Failures:**
- ✅ Rate limiting implemented (5 attempts/15 minutes)
- ⚠️ **ISSUE FOUND:** Session timeout set to 7 days (too long)
- ✅ Password requirements enforced (12 chars min)

**Step 3: Performance Analysis**

```typescript
// File: src/api/auth.ts:45
async function getUserWithPosts(userId: string) {
  const user = await User.findOne({ where: { id: userId } });
  const posts = await Post.find({ where: { userId } });  // 🔴 N+1 potential
  return { ...user, posts };
}
```

**🔴 ISSUE FOUND:** Potential N+1 query if called in loop. Should use JOIN.

**Step 4: Code Quality**

- ✅ Functions < 50 lines
- ✅ No `any` types
- ✅ Proper error handling
- ⚠️ **ISSUE FOUND:** `console.log` on line 78 should be removed
- ✅ Test coverage: 85%

### Review Report Generated

```markdown
# Code Review Report: User Authentication API

**Reviewed by:** Automated reviewer (code-reviewer skill)
**Date:** 2025-11-03
**PR:** #123
**Review Type:** Standard Pre-Merge Review

---

## Executive Summary

**Verdict:** ⚠️ APPROVED WITH RESERVATIONS

**Overview:**
Solid authentication implementation with proper security practices. Two medium-priority issues identified that should be fixed before merge.

**Key Metrics:**
- **Files Modified:** 2
- **Lines Changed:** +150/-30
- **Estimated Risk:** 🟡 Medium
- **Test Coverage:** 85%
- **Review Duration:** 8 minutes

---

## Critical Issues (Blocking - Must Fix Before Merge)

**None identified** ✅

---

## High Priority Issues (Fix Within 48 Hours)

**None identified** ✅

---

## Medium Priority Issues (Fix This Sprint)

### 1. Session Timeout Too Long 🟡

**Location:** `src/api/auth.ts:23`

**Issue:**
```typescript
const token = jwt.sign({ userId }, JWT_SECRET, { expiresIn: '7d' });
```

Session timeout of 7 days is excessive and increases risk if token is compromised.

**Recommendation:**
```typescript
const token = jwt.sign({ userId }, JWT_SECRET, { expiresIn: '24h' });
```

Change to 24 hours (or 1 hour for high-security applications).

**Severity:** 🟡 Medium
**Category:** Security - Authentication
**OWASP:** A07:2021 - Identification and Authentication Failures

---

### 2. Potential N+1 Query Pattern 🟡

**Location:** `src/api/auth.ts:45-48`

**Issue:**
```typescript
async function getUserWithPosts(userId: string) {
  const user = await User.findOne({ where: { id: userId } });
  const posts = await Post.find({ where: { userId } });
  return { ...user, posts };
}
```

If this function is called in a loop, it will create N+1 queries.

**Recommendation:**
```typescript
async function getUserWithPosts(userId: string) {
  const user = await User.findOne({
    where: { id: userId },
    relations: ['posts']
  });
  return user;
}
```

Use TypeORM relations to fetch in single query.

**Severity:** 🟡 Medium
**Category:** Performance
**Impact:** Could slow down API if user list grows

---

## Low Priority / Suggestions

### 3. Remove Debug Console Statement 🔵

**Location:** `src/api/auth.ts:78`

```typescript
console.log('User logged in:', user.email);  // Remove this
```

Replace with proper logging or remove.

**Recommendation:**
```typescript
logger.info('User logged in', { userId: user.id });
```

---

## Strengths & Good Practices

✅ **Excellent password hashing** - bcrypt with cost factor 12
✅ **Rate limiting implemented** - Prevents brute force attacks
✅ **Input validation** - Zod schemas used throughout
✅ **Parameterized queries** - No SQL injection risks
✅ **High test coverage** - 85% coverage with edge cases tested
✅ **Type safety** - No `any` types, strict TypeScript

---

## Recommendations Summary

1. **Before Merge:** Fix session timeout (5 minutes)
2. **Before Merge:** Fix N+1 query pattern (10 minutes)
3. **Optional:** Remove console.log and use proper logger

**Estimated Fix Time:** 15 minutes

---

## Sign-off

**Status:** ⚠️ APPROVED WITH RESERVATIONS

This PR demonstrates strong security practices and code quality. The two medium-priority issues are straightforward fixes that should be completed before merge to ensure optimal security and performance.

Once fixes are committed, this PR is ready to merge.

**Next Steps:**
1. Author: Fix session timeout and N+1 query
2. Author: Push fixes to PR
3. Reviewer: Quick re-review of changes
4. Merge to main
```

### Same Review In Structured Findings Mode

```markdown
## Findings

### Finding 1
- Severity: medium
- Location: src/api/auth.ts :: token creation :: lines 23-23
- Category: security
- Topic ID: G
- Related Questions: [2, 5]
- Description: Authentication token lifetime is set to seven days.
- Rationale: Long-lived tokens increase the blast radius of token theft and weaken session security.
- Suggestion: Reduce token lifetime and pair it with refresh-token rotation if long sessions are required.

### Finding 2
- Severity: medium
- Location: src/api/auth.ts :: getUserWithPosts :: lines 45-48
- Category: optimization
- Topic ID: D
- Related Questions: [1, 4]
- Description: User and posts are loaded with separate queries in a pattern that can become N+1 under iteration.
- Rationale: Query count will grow with caller usage and can degrade endpoint latency sharply.
- Suggestion: Fetch posts through a relation or join-based query.

## Clean Topics

- A Architecture and design: No significant issues found.
- H Testability: No significant issues found.
- I Observability: No significant issues found.
```

---

## Example 2: Security-Focused Review

### Context
**Feature:** Payment processing endpoint
**Files Changed:** `src/api/payments.ts`
**Lines Changed:** +200
**Review Type:** Security audit before deployment

### User Request
```
"Security audit for the payment processing code before we deploy to production"
```

### Review Process

**Step 1: OWASP Top 10 Deep Dive**

**A01 - Broken Access Control:**
```typescript
// 🔴 CRITICAL ISSUE FOUND
app.post('/api/payments/:userId', async (req, res) => {
  const { userId } = req.params;
  const payment = await processPayment(userId, req.body);
  res.json(payment);
});
```

**PROBLEM:** Any authenticated user can process payment for ANY user by changing userId in URL!

**FIX REQUIRED:**
```typescript
app.post('/api/payments/:userId', authenticateUser, async (req, res) => {
  const { userId } = req.params;

  // Verify user can only process their own payments
  if (req.user.id !== userId) {
    return res.status(403).json({ error: 'Forbidden' });
  }

  const payment = await processPayment(userId, req.body);
  res.json(payment);
});
```

**A02 - Cryptographic Failures:**
```typescript
// 🔴 CRITICAL ISSUE FOUND
const creditCard = {
  number: req.body.cardNumber,  // Stored in plain text!
  cvv: req.body.cvv,           // CVV stored (PCI-DSS violation!)
  exp: req.body.expiry
};
await db.save('credit_cards', creditCard);
```

**PROBLEMS:**
1. Credit card stored in plain text
2. CVV stored at all (never allowed under PCI-DSS)
3. No encryption

**FIX REQUIRED:**
```typescript
// Use payment processor API instead of storing cards
const stripeToken = await stripe.tokens.create({
  card: {
    number: req.body.cardNumber,
    exp_month: req.body.expMonth,
    exp_year: req.body.expYear,
    cvc: req.body.cvv  // Stripe handles this, we never store it
  }
});

// Store only the token (not the actual card)
await db.save('payment_methods', {
  userId: req.user.id,
  stripeToken: stripeToken.id,
  lastFour: req.body.cardNumber.slice(-4)  // Only last 4 digits
});
```

**A03 - Injection:**
✅ Parameterized queries used
✅ Input validated with Zod

**A04 - Insecure Design:**
```typescript
// 🟠 HIGH PRIORITY ISSUE
async function processPayment(userId, amount) {
  await debitAccount(userId, amount);
  await creditMerchant(amount);  // No transaction wrapper!
}
```

**PROBLEM:** If `creditMerchant` fails, user is charged but merchant isn't paid.

**FIX REQUIRED:**
```typescript
async function processPayment(userId, amount) {
  const transaction = await db.transaction();
  try {
    await debitAccount(userId, amount, transaction);
    await creditMerchant(amount, transaction);
    await transaction.commit();
  } catch (error) {
    await transaction.rollback();
    throw error;
  }
}
```

### Review Report

```markdown
# Security Audit Report: Payment Processing

**Verdict:** ❌ REQUIRES REVISION - DO NOT DEPLOY

**Critical Issues Found:** 2
**High Priority Issues:** 1

---

## 🔴 Critical Issues (BLOCK DEPLOYMENT)

### 1. Broken Access Control - IDOR Vulnerability

**Severity:** 🔴 Critical
**OWASP:** A01:2021 - Broken Access Control
**CWE:** CWE-639 (Authorization Bypass)

Any user can process payments for other users by changing userId in URL.

**Impact:** Financial fraud, unauthorized transactions
**Fix Time:** 10 minutes
**Must Fix:** YES - Before any deployment

---

### 2. PCI-DSS Violation - Credit Card Storage

**Severity:** 🔴 Critical
**OWASP:** A02:2021 - Cryptographic Failures
**Compliance:** PCI-DSS violation

Credit cards stored in plain text, CVV stored (never allowed).

**Impact:**
- PCI-DSS compliance failure
- Legal liability
- Massive fines if breached
- Customer data exposure

**Fix Time:** 2 hours (integrate payment processor)
**Must Fix:** YES - Legal requirement

---

## 🟠 High Priority Issues

### 3. Race Condition in Payment Processing

**Severity:** 🟠 High
**Category:** Insecure Design

No database transaction wrapping payment operations.

**Impact:** User charged but merchant not paid (or vice versa)
**Fix Time:** 30 minutes

---

## Recommendation

**DO NOT DEPLOY THIS CODE TO PRODUCTION**

The critical security issues pose immediate risk:
1. Financial fraud (access control issue)
2. Legal liability (PCI-DSS violation)

**Required Actions:**
1. Fix IDOR vulnerability (10 min)
2. Integrate Stripe/payment processor (2 hours)
3. Add transaction wrapping (30 min)
4. Re-audit after fixes
5. Penetration testing recommended

**Estimated Total Fix Time:** 3 hours

**Status:** ❌ REJECTED - Requires revision before deployment
```

---

## Example 3: Bad Review (What NOT to Do)

### Context
Same authentication PR from Example 1

### User Request
```
"Review this authentication PR"
```

### ❌ Bad Review Example

```markdown
# Code Review

Looks good! 👍

The code seems fine. I didn't find any major issues.

**Verdict:** ✅ APPROVED

Ship it!
```

### ❌ Why This Review is Bad

**Problems:**

1. **No Specificity**
   - "Looks good" - What specifically was checked?
   - "Seems fine" - Based on what criteria?

2. **No Evidence**
   - No mention of tools run
   - No code snippets reviewed
   - No security checklist

3. **Missed Critical Issues**
   - 7-day session timeout not mentioned
   - N+1 query not caught
   - Console.log not flagged

4. **No Actionable Feedback**
   - No recommendations
   - No severity classification
   - No specific files/lines referenced

5. **False Confidence**
   - Approved without thorough review
   - Could ship vulnerable code

### ✅ What a Good Review Should Have

Compare to Example 1:

1. **Ran automated tools** (npm audit, ESLint, TypeScript)
2. **Checked OWASP Top 10** (specific categories)
3. **Analyzed performance** (found N+1 query)
4. **Reviewed code quality** (found console.log)
5. **Provided specific recommendations** (with code snippets)
6. **Severity classification** (Critical/High/Medium/Low)
7. **Actionable next steps** ("Fix X before merge")

---

## Key Takeaways

### Good Code Review Checklist

✅ **Run automated tools first**
- npm audit, ESLint, TypeScript checks
- Review tool output systematically

✅ **Check security (OWASP Top 10)**
- Don't skip this even if tools pass
- Manual review catches business logic issues

✅ **Analyze performance**
- Look for N+1 queries
- Check algorithm complexity
- Review database indexes

✅ **Provide specific feedback**
- File names and line numbers
- Code snippets showing issue
- Code snippets showing fix

✅ **Classify severity**
- Critical: Block deployment
- High: Fix within 48h
- Medium: Fix this sprint
- Low: Nice to have

✅ **Be constructive**
- Suggest solutions, not just problems
- Acknowledge good practices too
- Be specific and helpful

---

**Remember:** The goal of code review is not to find fault, but to ensure quality, security, and maintainability. A thorough review saves time, money, and reputation in the long run.
