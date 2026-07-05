# Security-Focused Code Review: [Feature/PR Name]

**Reviewed by:** [Your Name]
**Date:** [YYYY-MM-DD]
**Commit/PR:** [#123 or commit hash]
**Review Type:** Security Audit

---

## Executive Summary

**Security Verdict:** ✅ SECURE | ⚠️ MINOR ISSUES | ❌ CRITICAL VULNERABILITIES

**Risk Level:** 🟢 Low | 🟡 Medium | 🔴 High

**Overview:**
[Brief assessment of security posture]

**Critical Vulnerabilities:** X
**High Priority Issues:** Y
**Medium Priority Issues:** Z

---

## OWASP Top 10 2021 Detailed Check

### A01:2021 – Broken Access Control

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Authentication required on protected routes
- [ ] Authorization validates user permissions
- [ ] Direct object references protected (IDOR prevention)
- [ ] Path traversal prevented
- [ ] CORS configured correctly

**Findings:**
[List any issues found with severity and location]

---

### A02:2021 – Cryptographic Failures

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Sensitive data encrypted at rest
- [ ] Sensitive data encrypted in transit (HTTPS)
- [ ] Passwords hashed with bcrypt/argon2
- [ ] API keys/secrets in environment variables
- [ ] No hardcoded credentials

**Findings:**
[List any issues]

---

### A03:2021 – Injection

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Parameterized queries used (no string concatenation)
- [ ] Input validation on all user inputs
- [ ] ORM used correctly (no raw queries with user input)
- [ ] Command injection prevented
- [ ] NoSQL injection prevented

**Findings:**
[List any issues]

---

### A04:2021 – Insecure Design

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Threat modeling performed
- [ ] Secure by default
- [ ] Defense in depth
- [ ] Fail securely
- [ ] Separation of duties

**Findings:**
[List any issues]

---

### A05:2021 – Security Misconfiguration

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Security headers configured (CSP, HSTS, X-Frame-Options)
- [ ] Error messages don't leak information
- [ ] Default credentials changed
- [ ] Unnecessary features disabled
- [ ] Software up to date

**Findings:**
[List any issues]

---

### A06:2021 – Vulnerable and Outdated Components

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] npm audit shows no critical/high vulnerabilities
- [ ] Dependencies up to date
- [ ] Known vulnerable components not used
- [ ] Component versions tracked

**Findings:**
[npm audit results]

---

### A07:2021 – Identification and Authentication Failures

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Multi-factor authentication available
- [ ] Password requirements enforced
- [ ] Session management secure
- [ ] Rate limiting on authentication endpoints
- [ ] Account lockout after failed attempts

**Findings:**
[List any issues]

---

### A08:2021 – Software and Data Integrity Failures

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Code signature verification
- [ ] Dependency integrity checks
- [ ] No unsigned/unverified plugins
- [ ] CI/CD pipeline secured

**Findings:**
[List any issues]

---

### A09:2021 – Security Logging and Monitoring Failures

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] Authentication failures logged
- [ ] Authorization failures logged
- [ ] High-value transactions logged
- [ ] Logs protected from tampering
- [ ] Alerting on suspicious activity

**Findings:**
[List any issues]

---

### A10:2021 – Server-Side Request Forgery (SSRF)

**Status:** ✅ Pass | ⚠️ Minor Issues | ❌ Vulnerabilities Found

**Checks Performed:**
- [ ] URL validation on user-supplied URLs
- [ ] Network segmentation
- [ ] Whitelist of allowed domains
- [ ] Internal IP ranges blocked

**Findings:**
[List any issues]

---

## Threat Modeling

### Attack Surface Analysis

**External Endpoints:**
- [List public-facing endpoints]

**Authentication Methods:**
- [List auth methods used]

**Sensitive Data Handled:**
- [List types of sensitive data]

### Potential Attack Vectors

1. **[Attack Vector Name]**
   - **Likelihood:** High | Medium | Low
   - **Impact:** High | Medium | Low
   - **Mitigation:** [Current controls]

---

## Security Test Results

### Automated Testing
- **SAST Tool:** [Tool name and version]
- **Results:** [Summary]

### Manual Testing
- **Penetration Testing:** [If performed]
- **Code Review:** [Findings]

---

## Compliance Checklist

**Applicable Standards:**
- [ ] PCI-DSS (if handling payments)
- [ ] HIPAA (if handling health data)
- [ ] GDPR (if handling EU citizen data)
- [ ] SOC 2 (if SaaS application)

**Findings:**
[List any compliance issues]

---

## Overall Security Assessment

### Risk Summary

**Critical Risks:** X
- [List critical vulnerabilities]

**High Risks:** Y
- [List high priority issues]

**Medium Risks:** Z
- [List medium issues]

### Recommendations

**Immediate Actions (Critical):**
1. [Action with timeline]
2. [Action with timeline]

**Short-term (Within 7 days):**
1. [Action]
2. [Action]

**Long-term (Within 30 days):**
1. [Action]
2. [Action]

---

## Security Sign-off

**Security Status:** [✅ APPROVED | ⚠️ CONDITIONAL APPROVAL | ❌ REJECTED]

**Conditions (if conditional):**
- [List conditions that must be met]

**Next Security Review:** [Date or trigger]

**Reviewer:** [Your Name]
**Date:** [YYYY-MM-DD]
