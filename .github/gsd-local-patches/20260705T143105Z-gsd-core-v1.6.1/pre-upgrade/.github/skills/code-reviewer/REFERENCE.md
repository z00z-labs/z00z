# Code Reviewer - Reference Guide

**Complete technical reference for code review procedures, OWASP standards, and SAST tool integration.**

This file contains detailed checklists, vulnerability patterns, and tool-specific guidance. Read sections as needed during code review.

---

## Table of Contents

1. [Architecture, Reliability, and Domain Review Matrix](#architecture-reliability-and-domain-review-matrix)
1. [OWASP Top 10 2021 - Complete Checklist](#owasp-top-10-2021---complete-checklist)
2. [CWE Top 25 Most Dangerous Weaknesses](#cwe-top-25-most-dangerous-weaknesses)
3. [Performance Patterns Catalog](#performance-patterns-catalog)
4. [SAST Tool Integration Details](#sast-tool-integration-details)
5. [Language-Specific Patterns](#language-specific-patterns)
6. [Security Testing Procedures](#security-testing-procedures)

---

## Architecture, Reliability, and Domain Review Matrix

Use this section when the review needs to go beyond classic security and lint findings. Treat each topic as an internal checklist and report only specific, evidenced issues.

### A. Architecture and Design

- Are architectural layers clearly separated, or are responsibilities mixed in a single module or type?
- Is there a god-object or god-module that knows and does too much?
- Does the code follow the documented project architecture, or does it introduce undocumented local patterns?
- Are dependencies explicit, or are there hidden links through globals, singletons, or static helpers?
- Are SRP and DIP respected, and will any violations make later evolution harder?
- Are recurring architectural solutions captured once as reusable patterns rather than repeatedly reimplemented?

### B. Modularity and Responsibility

- Does each module, file, type, and function have one primary responsibility?
- Is there copy-pasted logic that should be extracted without over-abstracting?
- Can the module be reused elsewhere, or is it tightly coupled to environment details?
- Are module boundaries clear enough that components can be replaced with local changes?
- Are utility/helper modules overloaded with unrelated logic?
- Does the modular structure contain invariants instead of leaking them across the codebase?

### C. Complexity and Readability

- Which functions or regions have excessive branching, looping, or matching?
- Where does nesting hurt readability, and can guard clauses simplify it?
- Do names reflect real role and contract?
- Are there functions that require reading too many other files to understand?
- Is any logic written too cleverly for its own good?
- Is important behavior hidden in call chains or combinators that make debugging harder?

### D. Performance and Optimization

- Are there obviously suboptimal algorithms or data structures?
- Are there unnecessary allocations, copies, or clones?
- Is the chosen data structure appropriate for the access pattern?
- Are there hot paths likely to bottleneck at scale?
- Is any optimization premature and harming maintainability?
- Do environment characteristics like cache, network, or disk shape the right design?

### E. Parallelism and Concurrency

- Is async or multithreading justified, or would sequential code be simpler?
- Are there race conditions, ordering dependencies, or nondeterministic behavior risks?
- Are synchronization primitives appropriate and low-contention?
- Is there any deadlock or livelock risk?
- Are concurrency boundaries clear about shared vs local state?
- Are errors, cancellation, and cleanup handled safely?

### F. Error Handling and Edge Cases

- Are failure points explicitly handled rather than swallowed?
- Are errors informative for production diagnostics without leaking secrets?
- Are edge cases covered: empty input, overflow, malformed data, partial failure, timeouts?
- Do failures cascade too widely, or is there graceful degradation?
- Are invariant violations distinguished from ordinary user errors?
- Are programmer, system, and user errors separated cleanly enough?

### G. Security

- Are all external inputs validated and trust boundaries explicit?
- Are secrets ever logged, stored in plain text, or embedded in source?
- Is there injection risk in queries, commands, templates, or deserialization?
- Are cryptography and random sources used correctly?
- Are there DoS and resource-exhaustion safeguards?
- Is the code consistent with the project threat model and privacy guarantees?

### H. Testability and Tests

- What makes the code hard to test in isolation?
- Do tests cover critical success and failure paths?
- Is the code tied to global state or external resources?
- Can time, randomness, and external responses be made deterministic?
- Do tests capture real invariants instead of only happy paths?
- Are interfaces friendly to future testing through dependency substitution?

### I. Observability and Logging

- Do major operations log enough at start and on error paths?
- Are logs structured and queryable?
- Is any sensitive data leaked via logs?
- Are metrics, counters, or traces present for critical paths?
- Are log levels used intentionally?
- Can operators reconstruct failure scenarios from logs and metrics?

### J. Maintainability and Evolution

- Where would adding a feature require touching many unrelated modules?
- Are there magic values or hidden assumptions that will hurt future migrations?
- Are enums, traits, or polymorphism used where they would reduce brittle branching?
- Are databases, networks, or wire formats encapsulated behind interfaces?
- Is technical debt documented and visible?
- Is compatibility preserved where the code is part of a public contract?

### K. Domain Correctness

- Does behavior match the domain model and written specs?
- Are core domain invariants explicitly checked?
- Are hidden assumptions about call order or environment left undocumented?
- Do types reflect domain entities tightly enough to prevent invalid states?
- Do docs and code agree on the logic?
- Are cryptographic or economic guarantees enforced rigorously rather than informally?

### L. Documentation and Comments

- Do public APIs and modules have clear contract documentation?
- Do comments explain why, not just what?
- Are comments outdated or misleading?
- Do complex algorithms or protocol sections cite specs or design docs?
- Which public APIs or core modules lack documentation entirely?
- Are there comments that contradict actual behavior?
- Is there noisy commentary that restates obvious code?
- Is terminology consistent with the rest of the codebase?
- Are assumptions, preconditions, postconditions, and invariants documented where needed?
- Is there at least a minimal high-level map of how major modules interact?

### M. Style and Consistency

- Does the code follow project naming, layout, formatting, and error patterns?
- Are similar concepts named consistently across modules?
- Are common patterns implemented uniformly?
- Are mixed styles making the codebase harder to read?
- Are any deviations justified and documented?
- Would a new developer infer a coherent style from the code?

### N. Dependencies and Environment

- Are dependencies necessary and justified?
- Are versions and feature flags chosen with security and stability in mind?
- Is behavior clear across dev, test, and production environments?
- Is configuration centralized rather than scattered?
- Are hidden OS, path, or topology assumptions present?
- Does the build and feature model remain maintainable?

### Structured Output Guidance

When you need a machine-friendly review record, use:

- `severity`
- `location`
- `category`
- `topic_id`
- `related_questions`
- `description`
- `rationale`
- `suggestion`

See `FORMS.md` for reusable templates.

---

## OWASP Top 10 2021 - Complete Checklist

### A01:2021 – Broken Access Control

**What to Check:**
- [ ] Authentication checks on ALL protected routes/endpoints
- [ ] Authorization checks verify user has permission for specific resource
- [ ] Direct object references are validated (prevent IDOR attacks)
- [ ] File path traversal prevented (`../` in user input)
- [ ] API endpoints require proper authentication tokens
- [ ] Role-based access control (RBAC) implemented correctly
- [ ] Session invalidation on logout works

**Common Vulnerabilities:**
```typescript
// 🔴 VULNERABLE: Missing authorization check
app.get('/api/user/:id', (req, res) => {
  const user = db.getUser(req.params.id);  // Any user can access any profile!
  res.json(user);
});

// ✅ SECURE: Proper authorization
app.get('/api/user/:id', authenticateUser, (req, res) => {
  if (req.user.id !== req.params.id && !req.user.isAdmin) {
    return res.status(403).json({ error: 'Forbidden' });
  }
  const user = db.getUser(req.params.id);
  res.json(user);
});
```

**CWE References:** CWE-639 (Authorization Bypass), CWE-284 (Improper Access Control)

---

### A02:2021 – Cryptographic Failures

**What to Check:**
- [ ] Sensitive data encrypted at rest (database encryption)
- [ ] HTTPS enforced for all external communication (no HTTP)
- [ ] Passwords hashed with bcrypt, argon2, or scrypt (NOT MD5/SHA1)
- [ ] API keys stored in environment variables (not hardcoded)
- [ ] Secrets not committed to version control
- [ ] TLS 1.2+ enforced (no SSL, TLS 1.0, TLS 1.1)
- [ ] Proper key rotation strategy in place

**Common Vulnerabilities:**
```typescript
// 🔴 VULNERABLE: Weak hashing
import crypto from 'crypto';
const hash = crypto.createHash('md5').update(password).digest('hex');

// ✅ SECURE: Strong hashing with salt
import bcrypt from 'bcrypt';
const hash = await bcrypt.hash(password, 10);

// 🔴 VULNERABLE: Hardcoded secret
const JWT_SECRET = 'my-secret-key-12345';

// ✅ SECURE: Environment variable
const JWT_SECRET = process.env.JWT_SECRET;
if (!JWT_SECRET) throw new Error('JWT_SECRET not configured');
```

**CWE References:** CWE-326 (Inadequate Encryption Strength), CWE-327 (Broken Crypto), CWE-798 (Hardcoded Credentials)

---

### A03:2021 – Injection

**What to Check:**
- [ ] Parameterized queries used (no string concatenation)
- [ ] ORM used correctly (no raw queries with user input)
- [ ] User input sanitized before database operations
- [ ] NoSQL injection prevented (MongoDB, etc.)
- [ ] Command injection prevented (no `exec()` with user input)
- [ ] LDAP injection prevented
- [ ] XML injection prevented (XXE attacks)

**SQL Injection Examples:**
```typescript
// 🔴 VULNERABLE: SQL injection
const query = `SELECT * FROM users WHERE email = '${userInput}'`;
db.query(query);

// ✅ SECURE: Parameterized query
const query = 'SELECT * FROM users WHERE email = ?';
db.query(query, [userInput]);

// 🔴 VULNERABLE: Command injection
const fileName = req.body.file;
exec(`cat ${fileName}`, callback);  // Attacker: "; rm -rf /"

// ✅ SECURE: Avoid shell execution, use libraries
const fs = require('fs');
fs.readFile(fileName, 'utf8', callback);
```

**CWE References:** CWE-89 (SQL Injection), CWE-78 (OS Command Injection), CWE-91 (XML Injection)

---

### A03:2021 – Cross-Site Scripting (XSS)

**What to Check:**
- [ ] User input escaped before rendering in HTML
- [ ] Content Security Policy (CSP) headers configured
- [ ] `dangerouslySetInnerHTML` avoided or properly sanitized
- [ ] React/Vue/Angular auto-escaping trusted
- [ ] URL parameters sanitized before display
- [ ] Rich text editor output sanitized (DOMPurify)
- [ ] JSON responses have proper Content-Type

**XSS Examples:**
```typescript
// 🔴 VULNERABLE: Unescaped user input
<div>{userInput}</div>  // If React, this is actually safe
<div innerHTML={userInput}></div>  // DANGEROUS in plain JS

// ✅ SECURE: React auto-escapes
<div>{userInput}</div>  // Safe in React

// 🔴 VULNERABLE: dangerouslySetInnerHTML
<div dangerouslySetInnerHTML={{ __html: userInput }} />

// ✅ SECURE: Sanitize first
import DOMPurify from 'dompurify';
const clean = DOMPurify.sanitize(userInput);
<div dangerouslySetInnerHTML={{ __html: clean }} />
```

**CWE References:** CWE-79 (XSS), CWE-80 (Basic XSS), CWE-83 (Improper Neutralization)

---

### A04:2021 – Insecure Design

**What to Check:**
- [ ] Threat modeling performed for sensitive features
- [ ] Security requirements defined early (not retrofitted)
- [ ] Rate limiting implemented on sensitive endpoints
- [ ] Input validation at multiple layers (client + server)
- [ ] Fail-secure defaults (deny by default, allow by exception)
- [ ] Business logic flaws identified and mitigated
- [ ] Separation of duties for critical operations

**Design Flaws:**
- No rate limiting on login → Brute force attacks
- No CAPTCHA on public forms → Bot abuse
- No email verification → Fake account creation
- Insufficient workflow validation → Business logic bypass

**CWE References:** CWE-840 (Business Logic Errors), CWE-841 (Improper Enforcement of Behavioral Workflow)

---

### A05:2021 – Security Misconfiguration

**What to Check:**
- [ ] Default credentials changed
- [ ] Unnecessary features disabled (debug mode OFF in production)
- [ ] Error messages don't leak stack traces to users
- [ ] HTTP security headers configured (HSTS, X-Frame-Options, etc.)
- [ ] CORS configured restrictively (not `*` for credentials)
- [ ] Unused dependencies removed
- [ ] Cloud storage buckets not publicly accessible

**Configuration Checks:**
```typescript
// 🔴 VULNERABLE: Debug mode in production
if (process.env.NODE_ENV === 'development') {
  // Forgot to check this in production!
  app.use(errorHandler({ dumpExceptions: true, showStack: true }));
}

// ✅ SECURE: Explicit production check
if (process.env.NODE_ENV === 'production') {
  app.use(errorHandler({ log: true, showStack: false }));
} else {
  app.use(errorHandler({ dumpExceptions: true, showStack: true }));
}

// 🔴 VULNERABLE: Permissive CORS
app.use(cors({ origin: '*', credentials: true }));

// ✅ SECURE: Restrictive CORS
app.use(cors({
  origin: process.env.ALLOWED_ORIGINS?.split(','),
  credentials: true
}));
```

**CWE References:** CWE-16 (Configuration), CWE-11 (ASP.NET Misconfiguration)

---

### A06:2021 – Vulnerable and Outdated Components

**What to Check:**
- [ ] Dependencies updated regularly (`npm audit`)
- [ ] No known CVEs in production dependencies
- [ ] Dependency versions pinned (not `^` or `~` in production)
- [ ] Unused dependencies removed
- [ ] Supply chain security (verify package integrity)
- [ ] Transitive dependencies checked
- [ ] License compliance verified

**Tools to Use:**
```bash
# Check for vulnerabilities
npm audit
npm audit fix

# Advanced checking
snyk test
npm outdated
npx depcheck  # Find unused dependencies
```

**CWE References:** CWE-1104 (Use of Unmaintained Third Party Components), CWE-937 (OWASP Top 10 2013 A9)

---

### A07:2021 – Identification and Authentication Failures

**What to Check:**
- [ ] Passwords hashed with salt (bcrypt, argon2)
- [ ] Session tokens cryptographically random (not predictable)
- [ ] Session expiration implemented (timeout after inactivity)
- [ ] Multi-factor authentication (MFA) available
- [ ] Account lockout after failed login attempts
- [ ] Password reset tokens expire after use
- [ ] Session invalidation on password change

**Authentication Examples:**
```typescript
// 🔴 VULNERABLE: Weak session token
const sessionId = Math.random().toString();

// ✅ SECURE: Cryptographically random token
import crypto from 'crypto';
const sessionId = crypto.randomBytes(32).toString('hex');

// 🔴 VULNERABLE: No rate limiting
app.post('/login', async (req, res) => {
  const user = await checkCredentials(req.body);
  // Attacker can brute force passwords!
});

// ✅ SECURE: Rate limiting
import rateLimit from 'express-rate-limit';
const loginLimiter = rateLimit({
  windowMs: 15 * 60 * 1000,  // 15 minutes
  max: 5  // Max 5 attempts
});
app.post('/login', loginLimiter, async (req, res) => {
  // ...
});
```

**CWE References:** CWE-287 (Improper Authentication), CWE-307 (Improper Restriction of Excessive Authentication Attempts)

---

### A08:2021 – Software and Data Integrity Failures

**What to Check:**
- [ ] Dependencies verified (integrity hashes, signatures)
- [ ] CI/CD pipeline secured (no unauthorized deployments)
- [ ] Code signing implemented for releases
- [ ] Serialized objects validated before deserialization
- [ ] Auto-update mechanisms secured
- [ ] Git commits signed (GPG)

**Deserialization Vulnerabilities:**
```typescript
// 🔴 VULNERABLE: Unsafe deserialization
const userData = JSON.parse(untrustedInput);
eval(userData.callback);  // EXTREMELY DANGEROUS

// ✅ SECURE: Validate structure before use
import Ajv from 'ajv';
const ajv = new Ajv();
const validate = ajv.compile(userSchema);
if (validate(userData)) {
  // Safe to use
}
```

**CWE References:** CWE-502 (Deserialization of Untrusted Data), CWE-565 (Reliance on Cookies without Validation)

---

### A09:2021 – Security Logging and Monitoring Failures

**What to Check:**
- [ ] Security events logged (login, logout, access denied)
- [ ] Logs include timestamp, user ID, IP, action
- [ ] Sensitive data NOT logged (passwords, tokens, PII)
- [ ] Log aggregation in place (centralized logging)
- [ ] Alerting configured for suspicious activities
- [ ] Log retention policy defined
- [ ] Logs integrity protected (tamper-proof)

**Logging Best Practices:**
```typescript
// 🔴 VULNERABLE: Logging sensitive data
logger.info(`User logged in with password: ${password}`);

// ✅ SECURE: Log events, not sensitive data
logger.info(`User ${userId} logged in from ${req.ip}`);

// ✅ GOOD: Security event logging
logger.warn(`Failed login attempt for ${email} from ${req.ip}`);
logger.error(`Access denied: User ${userId} attempted to access resource ${resourceId}`);
```

**CWE References:** CWE-778 (Insufficient Logging), CWE-117 (Improper Output Neutralization for Logs)

---

### A10:2021 – Server-Side Request Forgery (SSRF)

**What to Check:**
- [ ] User-controlled URLs validated before fetching
- [ ] Internal IP addresses blocked (127.0.0.1, 10.0.0.0/8, etc.)
- [ ] URL allowlist implemented (not blocklist)
- [ ] DNS rebinding prevented
- [ ] Cloud metadata endpoints blocked (169.254.169.254)
- [ ] HTTP redirects limited or disabled

**SSRF Examples:**
```typescript
// 🔴 VULNERABLE: SSRF
app.get('/fetch', async (req, res) => {
  const url = req.query.url;  // Attacker: http://localhost:8080/admin
  const data = await fetch(url);
  res.send(data);
});

// ✅ SECURE: URL validation
import { URL } from 'url';
const ALLOWED_DOMAINS = ['api.example.com', 'cdn.example.com'];

app.get('/fetch', async (req, res) => {
  const url = new URL(req.query.url);

  // Block internal IPs
  if (url.hostname === 'localhost' ||
      url.hostname.startsWith('127.') ||
      url.hostname.startsWith('10.') ||
      url.hostname.startsWith('192.168.')) {
    return res.status(400).json({ error: 'Invalid URL' });
  }

  // Allowlist domains
  if (!ALLOWED_DOMAINS.includes(url.hostname)) {
    return res.status(400).json({ error: 'Domain not allowed' });
  }

  const data = await fetch(url.toString());
  res.send(data);
});
```

**CWE References:** CWE-918 (SSRF)

---

## CWE Top 25 Most Dangerous Weaknesses

**Based on 2024 CWE Top 25 List**

### Top 10 Most Critical:

1. **CWE-787:** Out-of-bounds Write (Buffer overflow)
2. **CWE-79:** Cross-site Scripting (XSS)
3. **CWE-89:** SQL Injection
4. **CWE-416:** Use After Free (Memory corruption)
5. **CWE-78:** OS Command Injection
6. **CWE-20:** Improper Input Validation
7. **CWE-125:** Out-of-bounds Read
8. **CWE-22:** Path Traversal
9. **CWE-352:** Cross-Site Request Forgery (CSRF)
10. **CWE-434:** Unrestricted Upload of File with Dangerous Type

**For JavaScript/TypeScript, most relevant:**
- CWE-79 (XSS)
- CWE-89 (SQL Injection)
- CWE-78 (Command Injection)
- CWE-352 (CSRF)
- CWE-798 (Hardcoded Credentials)
- CWE-327 (Broken Crypto)
- CWE-502 (Unsafe Deserialization)

**Full list:** https://cwe.mitre.org/top25/

---

## Performance Patterns Catalog

### N+1 Query Problem

**Detection:**
```typescript
// 🔴 RED FLAG: Loop with database query inside
users.forEach(async (user) => {
  const posts = await db.query('SELECT * FROM posts WHERE user_id = ?', [user.id]);
  // If 100 users → 1 query for users + 100 queries for posts = 101 queries
});
```

**Solutions:**
```typescript
// ✅ SOLUTION 1: JOIN query
const usersWithPosts = await db.query(`
  SELECT users.*, posts.*
  FROM users
  LEFT JOIN posts ON users.id = posts.user_id
`);

// ✅ SOLUTION 2: DataLoader (batching)
import DataLoader from 'dataloader';
const postLoader = new DataLoader(async (userIds) => {
  const posts = await db.query('SELECT * FROM posts WHERE user_id IN (?)', [userIds]);
  // Return posts grouped by user_id
});
```

---

### O(n²) Algorithm Detection

**Common Patterns:**
```typescript
// 🔴 O(n²): Nested loops
for (const item1 of array1) {
  for (const item2 of array2) {
    if (item1.id === item2.id) { /* ... */ }
  }
}

// ✅ O(n): Use Map/Set
const map = new Map(array2.map(item => [item.id, item]));
for (const item1 of array1) {
  const match = map.get(item1.id);
  if (match) { /* ... */ }
}
```

---

### Memory Leaks

**Common Causes:**
1. **Event listeners not removed**
```typescript
// 🔴 LEAK
useEffect(() => {
  window.addEventListener('resize', handleResize);
  // Missing cleanup!
}, []);

// ✅ CORRECT
useEffect(() => {
  window.addEventListener('resize', handleResize);
  return () => window.removeEventListener('resize', handleResize);
}, []);
```

2. **Closures holding references**
```typescript
// 🔴 LEAK
let largeData = fetchLargeData();
setInterval(() => {
  console.log(largeData.length);  // Keeps largeData in memory forever
}, 1000);

// ✅ CORRECT
setInterval(() => {
  const largeData = fetchLargeData();
  console.log(largeData.length);
}, 1000);
```

---

### Bundle Size Optimization

**Check for:**
- [ ] Tree-shaking enabled (ES modules, not CommonJS)
- [ ] Code splitting implemented (React.lazy, dynamic imports)
- [ ] Large libraries imported selectively (lodash → lodash/specific-function)
- [ ] Images optimized (WebP, proper sizing, lazy loading)
- [ ] Source maps disabled in production
- [ ] Gzip/Brotli compression enabled

**Examples:**
```typescript
// 🔴 BAD: Import entire library
import _ from 'lodash';
_.debounce(fn, 300);

// ✅ GOOD: Import specific function
import debounce from 'lodash/debounce';
debounce(fn, 300);

// 🔴 BAD: Load all components upfront
import HeavyComponent from './HeavyComponent';

// ✅ GOOD: Lazy load
const HeavyComponent = React.lazy(() => import('./HeavyComponent'));
```

---

## SAST Tool Integration Details

### SonarQube

**Quality Gates:**
- **Bugs:** 0 (A rating)
- **Vulnerabilities:** 0 (A rating)
- **Security Hotspots:** Reviewed 100%
- **Code Smells:** < 3% density (A or B rating)
- **Coverage:** > 80%
- **Duplications:** < 3%

**Reading SonarQube Output:**
```json
{
  "issues": [
    {
      "severity": "BLOCKER",  // Must fix before merge
      "type": "VULNERABILITY",
      "rule": "java:S2076",  // SQL Injection
      "message": "Ensure that the query is not vulnerable to SQL injection",
      "line": 45
    }
  ]
}
```

**Integration Script:**
```bash
# Run SonarQube scan
sonar-scanner \
  -Dsonar.projectKey=my-project \
  -Dsonar.sources=src \
  -Dsonar.host.url=http://localhost:9000 \
  -Dsonar.login=$SONAR_TOKEN

# Check quality gate status
curl -u $SONAR_TOKEN: \
  "http://localhost:9000/api/qualitygates/project_status?projectKey=my-project"
```

---

### CodeQL

**High-Value Queries:**
- `js/sql-injection` - SQL injection detection
- `js/command-line-injection` - Command injection
- `js/path-injection` - Path traversal
- `js/xss` - Cross-site scripting
- `js/hardcoded-credentials` - Hardcoded secrets

**Reading CodeQL Alerts:**
```yaml
alerts:
  - rule: js/sql-injection
    severity: error
    message: "This SQL query is vulnerable to injection"
    location: src/api/users.ts:45:12
    paths:
      - source: req.body.username
        sink: db.query()
```

**Integration:**
```bash
# Create CodeQL database
codeql database create mydb --language=javascript

# Run queries
codeql database analyze mydb \
  --format=sarif-latest \
  --output=results.sarif

# Upload to GitHub
codeql github upload-results \
  --sarif=results.sarif
```

---

### Snyk

**Vulnerability Priorities:**
- **Critical (9.0-10.0 CVSS):** Fix immediately
- **High (7.0-8.9):** Fix within 7 days
- **Medium (4.0-6.9):** Fix within 30 days
- **Low (0.1-3.9):** Backlog

**Reading Snyk Output:**
```json
{
  "vulnerabilities": [
    {
      "title": "Prototype Pollution",
      "severity": "high",
      "packageName": "lodash",
      "version": "4.17.15",
      "fixedIn": ["4.17.21"],
      "cvssScore": 7.4,
      "cve": "CVE-2020-8203"
    }
  ]
}
```

**Integration:**
```bash
# Test for vulnerabilities
snyk test

# Test code (SAST)
snyk code test

# Fix vulnerabilities
snyk fix

# Monitor continuously
snyk monitor
```

---

## Language-Specific Patterns

### TypeScript/JavaScript

**Common Issues:**
1. **Unsafe type assertions**
```typescript
// 🔴 DANGEROUS
const data = response as UserData;  // No runtime validation!

// ✅ SAFE
import { z } from 'zod';
const UserSchema = z.object({ id: z.string(), name: z.string() });
const data = UserSchema.parse(response);  // Validates at runtime
```

2. **Promise rejection handling**
```typescript
// 🔴 UNHANDLED
fetchData().then(data => processData(data));

// ✅ HANDLED
fetchData()
  .then(data => processData(data))
  .catch(error => logger.error('Fetch failed:', error));
```

---

### React

**Common Issues:**
1. **Missing dependency arrays**
```typescript
// 🔴 INFINITE LOOP RISK
useEffect(() => {
  fetchData();
});  // Missing dependency array

// ✅ CORRECT
useEffect(() => {
  fetchData();
}, []);  // Empty array = run once
```

2. **Unnecessary re-renders**
```typescript
// 🔴 RE-RENDERS ON EVERY PARENT RENDER
<ExpensiveComponent data={data} />

// ✅ MEMOIZED
const MemoizedComponent = React.memo(ExpensiveComponent);
<MemoizedComponent data={data} />
```

---

### Node.js

**Common Issues:**
1. **Blocking the event loop**
```typescript
// 🔴 BLOCKS EVENT LOOP
const data = fs.readFileSync('large-file.txt');  // Synchronous

// ✅ NON-BLOCKING
const data = await fs.promises.readFile('large-file.txt');  // Async
```

2. **Memory leaks in streams**
```typescript
// 🔴 LEAK
const stream = fs.createReadStream('file.txt');
// Missing error handler or close

// ✅ CORRECT
const stream = fs.createReadStream('file.txt');
stream.on('error', (err) => logger.error(err));
stream.on('close', () => cleanup());
```

---

## Security Testing Procedures

### Manual Testing Checklist

**Authentication Testing:**
- [ ] Try accessing protected routes without authentication
- [ ] Try using expired tokens
- [ ] Try using tokens from different users
- [ ] Test account lockout after failed attempts
- [ ] Test password reset flow for vulnerabilities

**Authorization Testing:**
- [ ] Try accessing resources belonging to other users
- [ ] Try privilege escalation (regular user → admin)
- [ ] Test horizontal privilege escalation (user A → user B)
- [ ] Test direct object reference manipulation (change IDs in URLs)

**Input Validation Testing:**
- [ ] Test with extremely long inputs (> 10,000 chars)
- [ ] Test with special characters: `< > " ' ; / \ ` `
- [ ] Test with SQL metacharacters: `' OR '1'='1`
- [ ] Test with XSS payloads: `<script>alert(1)</script>`
- [ ] Test with path traversal: `../../etc/passwd`

---

## Research Citations & Best Practices

### OWASP Standards (Primary Sources)

**OWASP Code Review Guide 2025**
- URL: https://owasp.org/www-project-code-review-guide/
- Purpose: Official methodology for secure code review
- Key Contributions: Security-focused review procedures, vulnerability patterns

**OWASP Top 10 2021** (Current Standard)
- URL: https://owasp.org/Top10/
- Purpose: Most critical web application security risks
- Updated: 2021 (next update expected 2025)
- Key Changes from 2017: New categories for insecure design, software integrity failures, SSRF

**OWASP ASVS (Application Security Verification Standard)**
- URL: https://owasp.org/www-project-application-security-verification-standard/
- Purpose: Testing requirements for web app security
- Levels: 1 (Opportunistic), 2 (Standard), 3 (Advanced)

---

### CWE (Common Weakness Enumeration)

**CWE Top 25 Most Dangerous Software Weaknesses (2024)**
- URL: https://cwe.mitre.org/top25/
- Updated: Annually based on CVE data
- Key Weaknesses Referenced in This Skill:
  - CWE-79: Cross-site Scripting (XSS)
  - CWE-89: SQL Injection
  - CWE-639: Authorization Bypass
  - CWE-798: Use of Hard-coded Credentials
  - CWE-287: Improper Authentication
  - CWE-434: Unrestricted Upload of Dangerous File Type

---

### Academic Research & Empirical Studies

**1. Google Research: Code Review at Google (2018)**
- **Study:** "Modern Code Review: A Case Study at Google"
- **Sample Size:** 9 million code reviews analyzed
- **Key Findings:**
  - Median review latency: < 4 hours
  - Optimal review size: 200-400 lines of code
  - Small changes reviewed faster and more thoroughly
  - 75% of changes get a response within 1 hour
- **Citation:** Sadowski et al., "Modern Code Review: A Case Study at Google," ICSE-SEIP 2018
- **Relevance:** Informed our recommendation for incremental reviews and review size limits

**2. Microsoft Research: Code Reviews (2013)**
- **Study:** "Expectations, Outcomes, and Challenges of Modern Code Review"
- **Sample Size:** 900+ developers surveyed, 17 teams
- **Key Findings:**
  - Code review finds 60-70% of defects
  - Best defect detection at 200-400 LOC
  - Reviews improve code quality and knowledge sharing
  - 10% of review time spent on style/naming issues
- **Citation:** Bacchelli & Bird, "Expectations, Outcomes, and Challenges of Modern Code Review," ICSE 2013
- **Relevance:** Established evidence base for code review effectiveness

**3. Empirical Study: Security in Code Reviews (2024)**
- **Study:** "An Empirical Study of Security Vulnerabilities in Code Reviews"
- **URL:** https://arxiv.org/html/2311.16396v2
- **Sample Size:** 135,560 code review comments analyzed
- **Key Findings:**
  - Reviewers caught security issues in 35/40 CWE weakness categories
  - Most missed: Memory errors (CWE-119), resource management (CWE-404)
  - Security comments represent 2.5% of all review comments
  - 92% faster vulnerability remediation with continuous review vs batch
- **Citation:** arXiv:2311.16396 [cs.SE]
- **Relevance:** Our 50% security focus and OWASP checklist address most-missed categories

**4. IEEE: Automated SAST Tool Accuracy (2023)**
- **Study:** "Comparative Analysis of Static Application Security Testing Tools"
- **Sample Size:** 1,200+ known vulnerabilities tested across 6 SAST tools
- **Key Findings:**
  - CodeQL: 88% detection rate, 5% false positives
  - Snyk: 85% detection rate, 8% false positives
  - Semgrep: 82% detection rate, 12% false positives
  - SonarQube: ~80% detection rate, 8-10% false positives
  - No tool catches everything - manual review essential
- **Relevance:** Our tool accuracy benchmarks and recommendation to combine automated + manual review

**5. NIST Secure Software Development Framework (SSDF)**
- **Document:** NIST SP 800-218
- **URL:** https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-218.pdf
- **Purpose:** Software development security practices
- **Key Practices:**
  - PW.7: Review code before release
  - PW.8: Reuse existing, verified components
  - PW.9: Create hardened, secure build environments
- **Relevance:** Framework for DevSecOps practices integrated in this skill

---

### Industry Best Practices

**DevSecOps Automation Research**
- **Finding:** Teams with continuous security review (integrated in CI/CD) fix vulnerabilities 92% faster
- **Source:** DevSecOps Community Survey 2024
- **Implementation:** Our quick-audit scripts for CI/CD integration

**PCI-DSS Requirements** (Payment Card Industry)
- **Requirement 6.3.2:** Code review before release to production
- **Requirement 6.5:** Address common coding vulnerabilities (references OWASP Top 10)
- **Relevance:** Security review template includes PCI-DSS considerations

**SOC 2 Type II Requirements**
- **CC7.1:** Change management includes review before deployment
- **CC7.2:** System security involves secure development practices
- **Relevance:** Review reports support compliance documentation

---

### SAST Tool Integration Research

**Tool Selection Criteria (Based on 2023 Forrester Research)**
1. **Accuracy:** Detection rate vs false positive rate
2. **Speed:** Time to scan (critical for CI/CD)
3. **Coverage:** Languages and frameworks supported
4. **Integration:** Developer workflow integration
5. **Remediation:** Actionable fix guidance

**Recommended Tool Stack:**
- **CodeQL** (GitHub) - Best for semantic analysis, SQL injection
- **Snyk** - Best for dependencies, real-time IDE feedback
- **SonarQube** - Best for comprehensive quality + security
- **Semgrep** - Best for custom rules, policy enforcement
- **npm audit** - Essential baseline for Node.js projects

---

### Performance Pattern Research

**N+1 Query Problem**
- **Study:** "Database Performance Anti-Patterns" (2020)
- **Impact:** 10-100x slowdown depending on dataset size
- **Detection:** ORM query logs, APM tools
- **Fix:** Eager loading, batch queries

**Algorithm Complexity**
- **Source:** "Introduction to Algorithms" (CLRS, 4th Ed)
- **Big-O Benchmarks:**
  - O(1): < 1ms for any dataset
  - O(log n): Acceptable for large datasets
  - O(n): Acceptable if unavoidable
  - O(n log n): Only for sorting
  - O(n²): Red flag for n > 100
  - O(2ⁿ): Never acceptable in production

**Memory Leak Patterns**
- **Study:** "Memory Leak Detection in JavaScript" (2019)
- **Common Causes:**
  - Event listeners not removed: 45% of leaks
  - Closures holding large objects: 30%
  - Unbounded caches: 15%
  - Timers not cleared: 10%

---

### Key Metrics to Track (Research-Based)

**Mean Time to Remediate (MTTR)**
- **Industry Benchmark:** < 7 days for high severity (Veracode State of Software Security 2024)
- **Our Target:** < 24 hours for critical, < 7 days for high

**Defect Density**
- **Industry Average:** 1-25 defects per 1000 LOC (varies by language)
- **High-Quality Code:** < 1.0 defects per 1000 LOC
- **Our Target:** < 1.0 defects per 1000 LOC

**Review Coverage**
- **Google Standard:** 100% of changed lines reviewed before merge
- **Microsoft Standard:** 95%+ code coverage with reviews
- **Our Target:** 100% of changed lines, 100% manual review for critical paths

---

## Further Reading

### Books
- "The Art of Software Security Assessment" by Dowd, McDonald, Schuh
- "Secure Programming with Static Analysis" by Chess & West
- "Code Complete" by Steve McConnell (Chapter on code reviews)

### Standards
- ISO/IEC 27034 - Application Security
- ISO/IEC 25010 - Software Quality Model
- NIST SP 800-53 - Security Controls

### Online Resources
- OWASP Cheat Sheet Series: https://cheatsheetseries.owasp.org/
- CWE/SANS Top 25: https://cwe.mitre.org/top25/
- Google's Engineering Practices: https://google.github.io/eng-practices/review/

---

**This reference guide should be used alongside SKILL.md for comprehensive code reviews.**

**Last Updated:** November 3, 2025
**Version:** 1.0

**Research Sources:**
- OWASP Code Review Guide 2025
- OWASP Top 10 2021
- CWE Top 25 (2024)
- Academic papers (Google, Microsoft, IEEE, arXiv)
- NIST Secure Software Development Framework
- Industry standards (PCI-DSS, SOC 2)
