# Bad Code Review Example (What NOT to Do)

This example shows a superficial, unhelpful code review that misses critical issues and provides no value.

---

## Context

**PR Title:** Add user profile update endpoint
**Files Changed:** 2
**Lines Changed:** +120/-15
**Author:** @developer
**Reviewer:** @careless-reviewer

**NOTE:** This is the SAME PR as the good review example. Compare how differently it's handled!

---

## ❌ The Bad Review

### Reviewer's Comment

> Looks good! LGTM 👍
>
> **Verdict:** ✅ APPROVED

---

## 🚫 Why This Review is Terrible

Let me count the ways this review fails...

---

### ❌ Problem 1: No Specificity

**What the reviewer said:**
> "Looks good!"

**What's wrong:**
- Doesn't say WHAT looks good
- Doesn't say WHAT was checked
- Provides zero evidence of actual review

**What SHOULD have been said:**
> "I reviewed the authentication logic, input validation, and test coverage. The authorization check on line 45 properly prevents users from updating other profiles, and the 88% test coverage is excellent."

---

### ❌ Problem 2: Missed Critical Security Issue

**What the code has:**
```typescript
router.put('/profile/:userId', authenticateUser, async (req, res) => {
  // NO RATE LIMITING!
  const updated = await updateUserProfile(req.params.userId, req.body);
  res.json(updated);
});
```

**What the reviewer said:**
Nothing. Didn't catch it.

**Impact of Missing This:**
- Attackers can spam profile updates
- DoS vulnerability
- No protection against abuse
- Will cause production issues

**What SHOULD have been caught:**
"Missing rate limiting on this endpoint. Add rate limiter to prevent abuse (5 updates per 15 minutes recommended)."

---

### ❌ Problem 3: Missed Performance Issue

**What the code has:**
```typescript
async getUserWithPreferences(userId: string) {
  const user = await User.findOne({ where: { id: userId } });
  const preferences = await Preferences.find({ where: { userId } });
  // N+1 query pattern!
  return { ...user, preferences };
}
```

**What the reviewer said:**
Nothing. Didn't notice.

**Impact of Missing This:**
- Slow API responses
- Database load increases
- Will get worse as users grow
- Could cause scaling issues

**What SHOULD have been caught:**
"N+1 query detected. Use TypeORM relations to fetch in single query for 40% performance improvement."

---

### ❌ Problem 4: No Evidence of Testing

**What the reviewer SHOULD have done:**
```bash
# Pull the branch
git checkout feature/profile-update

# Run tests
npm test

# Check coverage
npm run test:coverage

# Try the endpoint
curl -X PUT http://localhost:3000/api/profile/123 \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"email":"test@example.com"}'
```

**What the reviewer actually did:**
🤷‍♂️ Unknown. Probably just glanced at the code.

---

### ❌ Problem 5: No Actionable Feedback

**What was provided:**
- "LGTM" (Looks Good To Me)
- A thumbs up emoji

**What's missing:**
- No specific comments
- No improvement suggestions
- No educational value
- Nothing for the author to learn from

**Compare to a good review:**
- "Great work on the authorization logic!"
- "Consider adding DTOs for validation"
- "Fix the N+1 query before merge"

---

### ❌ Problem 6: Rubber Stamp Approval

**What happened:**
Reviewer approved without actually reviewing.

**Signs of rubber stamping:**
- ✅ Instant approval (< 2 minutes for 120 lines)
- ✅ Generic praise ("looks good")
- ✅ No specific comments
- ✅ No questions asked
- ✅ No code snippets referenced

**Impact:**
- Bugs ship to production
- Security vulnerabilities not caught
- Team loses trust in code review process
- Technical debt accumulates

---

## 🎯 Direct Comparison

### Same PR, Two Reviews

| Aspect | Bad Review | Good Review |
|--------|-----------|-------------|
| **Time spent** | < 2 minutes | 12 minutes |
| **Issues found** | 0 | 2 medium, 2 low |
| **Security checks** | None | OWASP Top 10 |
| **Performance analysis** | None | Found N+1 query |
| **Specificity** | None | File:line numbers |
| **Code snippets** | None | Problem + solution |
| **Educational value** | Zero | High |
| **Author learns** | Nothing | Multiple best practices |
| **Verdict** | ✅ APPROVED | ⚠️ APPROVED WITH RESERVATIONS |
| **Production risk** | HIGH | LOW (after fixes) |

---

## 💣 Real-World Consequences of Bad Reviews

### What Happened After This Bad Review

**Week 1:**
```
Production deployed ✅
No immediate issues 😊
```

**Week 2:**
```
⚠️ API response times increasing
⚠️ Database load spiking
⚠️ Users complaining about slow profile updates
```

**Week 3:**
```
🔴 Attacker discovers no rate limiting
🔴 Spam attack: 10,000 profile updates in 5 minutes
🔴 Database overloaded
🔴 Site goes down for 2 hours
```

**Week 4:**
```
💰 Incident post-mortem
💰 Lost revenue: $50,000
💰 Engineer time: 40 hours debugging
💰 Customer support: 200 tickets
😞 Customer trust damaged
```

**Total cost of "LGTM" review:** > $75,000

**Time to add rate limiting:** 10 minutes

---

## 🛡️ How to Avoid Being This Reviewer

### Minimum Review Checklist

**Before approving ANY PR, you MUST:**

1. **Pull and run the code**
   ```bash
   git checkout feature-branch
   npm install
   npm test
   ```

2. **Run automated tools**
   ```bash
   npm audit
   npm run lint
   npm run type-check
   ```

3. **Check security (2 minutes)**
   - Authentication/authorization present?
   - User input validated?
   - No hardcoded secrets?

4. **Check performance (2 minutes)**
   - Any N+1 queries?
   - Nested loops over large data?
   - Database indexes present?

5. **Check code quality (2 minutes)**
   - Tests present and passing?
   - No console.log statements?
   - TypeScript strict mode?

6. **Provide specific feedback**
   - File:line references
   - Code snippets
   - Specific suggestions

**Total time:** 8-15 minutes for typical PR

**Value delivered:** Prevents production issues, educates team, improves code quality

---

## 🎓 Learning Exercise

### Try This:

1. Read the [Good Review Example](./good-review-example.md)
2. Note what the good reviewer found
3. Look at this bad review again
4. See how much value was lost?

### Ask Yourself:

- Would I want my name on this review?
- Would I trust this reviewer with my production code?
- Am I sometimes this reviewer?

**Be honest:** We've all done quick "LGTM" reviews. The goal is to do better.

---

## ✅ The Fix: How to Do Better

### Time Investment

**Bad review:** 2 minutes → $75,000 production incident

**Good review:** 12 minutes → Issues caught before production

**ROI:** 6x time investment, infinite value

---

### Quick Improvement Plan

**Week 1:** Commit to running automated tools
```bash
npm audit && npm run lint && npm test
```

**Week 2:** Add security checklist
- Check for hardcoded secrets
- Verify authentication/authorization
- Review input validation

**Week 3:** Add performance checklist
- Look for N+1 queries
- Check algorithm complexity
- Review database queries

**Week 4:** Provide specific feedback
- Use file:line format
- Include code snippets
- Suggest improvements

---

## 💡 Remember

**Bad reviews are worse than no reviews.**

Why?
- Creates false confidence ("it was reviewed")
- Issues slip through
- Author doesn't learn
- Team culture suffers

**Good reviews:**
- Catch issues early
- Educate the team
- Improve code quality
- Build trust

---

## 🎯 Key Takeaways

### ❌ DON'T:
- Rubber stamp with "LGTM"
- Approve without running code
- Skip security checks
- Provide vague feedback
- Rush through reviews

### ✅ DO:
- Run automated tools
- Check OWASP Top 10
- Analyze performance
- Provide specific feedback
- Take time to do it right

---

**The Bottom Line:**

If you don't have time to review properly, say so:

> "I don't have bandwidth for a thorough review right now. Can someone else take this, or should we wait until tomorrow when I can give it proper attention?"

This is **infinitely better** than a rushed, bad review.

---

**Remember:** Your name on an approval means "I've verified this is safe and ready for production." Own it. 🛡️
