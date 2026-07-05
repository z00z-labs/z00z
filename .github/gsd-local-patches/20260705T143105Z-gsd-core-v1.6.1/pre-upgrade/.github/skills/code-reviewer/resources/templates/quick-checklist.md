# Quick Code Review Checklist (3 Minutes)

Use this for rapid reviews of small changes or pre-commit checks.

---

## Security (30 seconds) 🔒

- [ ] **No hardcoded secrets/API keys** - Check for `api_key`, `password`, `secret` in code
- [ ] **User input sanitized/validated** - All user inputs go through validation
- [ ] **Authentication on protected routes** - Middleware present on all protected endpoints
- [ ] **No SQL injection** - Parameterized queries only, no string concatenation
- [ ] **Passwords hashed** - bcrypt/argon2 used (not MD5/SHA1)

**Quick command:**
```bash
git diff --staged | grep -i "api_key\|password\|secret"
```

---

## Performance (30 seconds) ⚡

- [ ] **No N+1 query patterns** - No loops containing database queries
- [ ] **No nested loops over large datasets** - Check for O(n²) algorithms
- [ ] **Database indexes present** - Queried columns have indexes
- [ ] **No synchronous file operations** - Use async/await for I/O

**Quick check:**
Look for:
- `forEach` or `map` containing `await db.query`
- Nested `for` loops
- `fs.readFileSync` in Node.js

---

## Code Quality (60 seconds) ✨

- [ ] **TypeScript strict mode** - No `any` types without justification
- [ ] **Functions < 50 lines** - Extract larger functions
- [ ] **No commented-out code** - Remove dead code
- [ ] **Proper error handling** - No empty `catch` blocks
- [ ] **No console.log statements** - Use proper logging library

**Quick command:**
```bash
# Find console.logs
git diff --staged | grep "console.log"

# Find any types
git diff --staged | grep ": any"
```

---

## Tests (30 seconds) 🧪

- [ ] **Unit tests present** - New code has corresponding tests
- [ ] **Test coverage > 80%** - Check coverage report
- [ ] **Edge cases tested** - Not just happy path
- [ ] **Tests pass locally** - Run test suite before review

**Quick command:**
```bash
npm test
npm run test:coverage
```

---

## Documentation (30 seconds) 📚

- [ ] **Complex logic has comments** - Explain the "why", not the "what"
- [ ] **README updated** - If behavior changed or new features added
- [ ] **Public APIs have JSDoc** - Functions exported have documentation

**Quick check:**
- Are there functions > 20 lines without any comments?
- Did public API change? Is it documented?

---

## Total Time: 3 minutes ⏱️

---

## Quick Pass/Fail Criteria

### ❌ Immediate Fail (Stop Review)

If you find ANY of these:
- Hardcoded API keys or passwords
- SQL injection vulnerability (string concatenation in queries)
- XSS vulnerability (unescaped user input in HTML)
- Authentication bypass
- Plain text password storage

**Action:** Reject immediately. Security takes priority.

---

### ⚠️ Conditional Pass (Requires Fixes)

If you find:
- Missing tests for new code
- console.log statements
- `any` types
- No error handling
- Functions > 50 lines

**Action:** Request changes before merge.

---

### ✅ Pass

If:
- All checklist items pass
- No security vulnerabilities
- Tests present and passing
- Code is readable and maintainable

**Action:** Approve and merge.

---

## When to Do a Full Review

Use the **full code review process** (15-30 minutes) instead of this quick checklist if:

- Changes affect authentication or authorization
- Changes handle sensitive data (PII, payment info, health data)
- Changes affect critical user flows
- Changes are > 400 lines
- Changes touch database migrations
- Pre-deployment security audit

For these cases, see:
- `code-review-report.md` for comprehensive reviews
- `security-review-template.md` for security audits
- `performance-review-template.md` for performance reviews

---

## Tips for Fast Reviews

1. **Use git diff** - Review only what changed
2. **Run automated tools first** - Let tools catch obvious issues
3. **Focus on critical paths** - Authentication, authorization, data handling
4. **Trust but verify** - Good test coverage lets you move faster
5. **Know when to go deep** - Some changes deserve more time

---

**Remember:** This is a screening tool, not a replacement for thorough code review. Use your judgment on when to go deeper.
