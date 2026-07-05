# Code Reviewer Skill

**Research-backed code review with OWASP 2025, SAST integration, and DevSecOps best practices.**

Version: 1.0
Created: November 3, 2025
Research-Backed: OWASP, CWE, Google, Microsoft, IEEE

---

## 🎯 Quick Start

### Run a Code Review

```bash
# Quick security audit (2 minutes)
bash scripts/quick-audit.sh       # Linux/Mac
scripts\quick-audit.bat            # Windows

# Or invoke the skill directly
"Review this code for security issues"
"Check for bugs and vulnerabilities"
"Analyze this PR"
```

### What This Skill Does

- ✅ **OWASP Top 10 Security Checks** - Identifies injection, XSS, authentication issues
- ✅ **Architecture And Reliability Review** - Checks layering, modularity, invariants, observability, and domain fit
- ✅ **Performance Analysis** - Detects N+1 queries, O(n²) algorithms, memory leaks
- ✅ **Code Quality Standards** - TypeScript, ESLint, naming conventions, complexity
- ✅ **SAST Integration** - Works with SonarQube, CodeQL, Snyk, npm audit
- ✅ **Structured Reports** - Clear severity classification, topic ids, and actionable recommendations

---

## 📁 File Structure

```
code-reviewer/
├── SKILL.md                    # Main skill instructions (~2,600 tokens)
├── EXAMPLES.md                 # Complete review examples (good, bad, security)
├── REFERENCE.md                # Complete OWASP Top 10, CWE Top 25 (~15,000 tokens)
├── FORMS.md                    # Template overview and guidance
├── README.md                   # This file - Quick start guide
├── scripts/
│   ├── quick-audit.sh         # Quick security audit (Linux/Mac)
│   └── quick-audit.bat        # Quick security audit (Windows)
└── resources/
    ├── templates/
    │   ├── code-review-report.md           # Comprehensive review template
    │   ├── security-review-template.md     # Security-focused audit template
    │   ├── performance-review-template.md  # Performance analysis template
    │   └── quick-checklist.md              # 3-minute rapid review checklist
    └── examples/
        ├── good-review-example.md          # What a thorough review looks like
        └── bad-review-example.md           # What to avoid (rubber stamps)
```

**Progressive Disclosure:** Core skill loads ~2,600 tokens. Templates, reference docs, and examples loaded on-demand when needed.

---

## 🔍 Auto-Invoke Triggers

This skill automatically activates when you mention:
- "review this code"
- "check for bugs"
- "security audit"
- "analyze this PR"
- "code review"
- "check code quality"
- "find vulnerabilities"
- "performance check"

---

## 🛡️ Research Backing

### OWASP Standards
- **OWASP Code Review Guide 2025** - Official review methodology
- **OWASP Top 10 2021** - Current security standard
- **CWE Top 25 2024** - Most dangerous software weaknesses

### Industry Research
- **Google Research (2018):** 9M code reviews analyzed
  - Median review latency: < 4 hours
  - 200-400 LOC optimal for catching defects
- **Microsoft Research (2013):** 900+ developers surveyed
  - Code review finds 60-70% of defects
  - Best defect detection rate at 200-400 LOC
- **Empirical Study (2024):** 135,560 code review comments analyzed
  - Reviewers caught security issues in 35/40 weakness categories
  - Most missed: Memory errors, resource management

### Key Finding
Teams with **continuous code review** fix vulnerabilities **92% faster** than batch reviews.

---

## 🎯 Review Focus (Balanced Quality + Security)

- **50% Security** - OWASP Top 10, vulnerabilities, authentication
- **30% Code Quality** - Maintainability, standards, duplication
- **20% Performance** - N+1 queries, algorithm complexity, bundle size

The skill also supports a broader review matrix for deeper code reviews:

- `A` Architecture and design
- `B` Modularity and responsibility
- `C` Complexity and readability
- `D` Performance and optimization
- `E` Parallelism and concurrency
- `F` Error handling and edge cases
- `G` Security
- `H` Testability and tests
- `I` Observability and logging
- `J` Maintainability and evolution
- `K` Domain correctness
- `L` Documentation and comments
- `M` Style and consistency
- `N` Dependencies and environment

Use the matrix as an internal checklist, then report only concrete issues actually found in code.

## 🧭 Structured Findings

For rigorous reviews, findings can be emitted with a strict contract:

- `severity`
- `location`
- `category`
- `topic_id`
- `related_questions`
- `description`
- `rationale`
- `suggestion`

See `FORMS.md` for ready-to-use structured finding templates and `REFERENCE.md` for the complete topic checklist.

---

## 🔧 SAST Tool Integration

### Supported Tools

**Always Available:**
- ✅ npm audit (dependency vulnerabilities)
- ✅ ESLint (code quality)
- ✅ Prettier (code formatting)
- ✅ TypeScript type checking

**Advanced (If Installed):**
- ✅ **SonarQube** - Comprehensive quality + security
  - Quality Gates, Code Smells, Duplications
- ✅ **CodeQL** - Semantic analysis (88% accuracy)
  - SQL injection, XSS, command injection detection
- ✅ **Snyk** - Developer-friendly security (85% accuracy)
  - Dependency vulnerabilities, real-time feedback
- ✅ **Semgrep** - Custom security rules (82% accuracy)
  - Policy-as-code, organization-specific patterns

### Tool Accuracy Benchmarks (2025 Research)

| Tool | Accuracy | False Positive Rate | Best Use Case |
|------|----------|---------------------|---------------|
| CodeQL | 88% | 5% | Semantic analysis, SQL injection |
| Snyk | 85% | 8% | Dependencies, real-time IDE feedback |
| Semgrep | 82% | 12% | Custom rules, policy enforcement |
| SonarQube | ~80% | 8-10% | Comprehensive quality + security |

---

## 📊 Severity Classification

### 🔴 Critical (Blocks Deployment)
- SQL injection vulnerabilities
- XSS vulnerabilities
- Authentication bypass
- Hardcoded secrets/API keys
- Remote code execution risks

**Action:** STOP. Must fix immediately.

### 🟠 High (Fix Within 48 Hours)
- Missing authentication checks
- Insecure session management
- CSRF vulnerabilities
- N+1 query problems in critical paths

**Action:** Create blocker ticket. Fix before next deployment.

### 🟡 Medium (Fix This Sprint)
- Missing input validation
- Inefficient algorithms (O(n²) on small datasets)
- Code duplication
- Missing error handling

**Action:** Create ticket. Fix within current sprint.

### 🔵 Low (Backlog)
- Code style violations
- Minor performance optimizations
- Refactoring opportunities
- Documentation improvements

**Action:** Optional. Add to backlog.

---

## 📋 Quick Review Checklist (3 Minutes)

**Security (30 seconds):**
- [ ] No hardcoded secrets/API keys
- [ ] User input sanitized/validated
- [ ] Authentication on protected routes
- [ ] No SQL injection (parameterized queries)
- [ ] Passwords hashed (bcrypt/argon2)

**Performance (30 seconds):**
- [ ] No N+1 query patterns
- [ ] No nested loops over large datasets
- [ ] Database indexes present
- [ ] No synchronous file operations

**Code Quality (60 seconds):**
- [ ] TypeScript strict mode (no `any`)
- [ ] Functions < 50 lines
- [ ] No commented-out code
- [ ] Proper error handling
- [ ] No console.log statements

**Tests (30 seconds):**
- [ ] Unit tests present
- [ ] Test coverage > 80%
- [ ] Edge cases tested
- [ ] Tests pass locally

**Documentation (30 seconds):**
- [ ] Complex logic has comments
- [ ] README updated
- [ ] Public APIs have JSDoc

**Total Time: 3 minutes**

---

## 💡 Usage Examples

### Example 1: Quick PR Review
```
User: "Review this PR for security issues"

Skill Output:
✅ Runs quick-audit.sh
✅ Checks OWASP Top 10
✅ Analyzes performance patterns
✅ Generates structured report with severity classification
```

### Example 2: Pre-Deployment Audit
```
User: "Security audit before deployment"

Skill Output:
✅ Comprehensive security review
✅ SAST tool integration (SonarQube, Snyk, CodeQL)
✅ Threat modeling
✅ Manual testing checklist
✅ Deployment recommendation (APPROVED/REJECTED)
```

### Example 3: Performance Review
```
User: "Check performance of the user list endpoint"

Skill Output:
✅ N+1 query detection
✅ Algorithm complexity analysis
✅ Database index recommendations
✅ Memory leak detection
✅ Bundle size impact
```

---

## 🚀 Key Features

### 1. Research-Backed Standards
Every checklist item backed by OWASP, CWE, or academic research.

### 2. Progressive Disclosure
Core skill loads fast (4,500 tokens). Detailed references loaded on-demand.

### 3. Multi-Tool Integration
Works with SonarQube, CodeQL, Snyk, or just npm audit + ESLint.

### 4. Structured Output
Clear reports with severity classification, code snippets, and specific recommendations.

### 5. Automation Scripts
Quick-audit scripts run all basic checks in 2 minutes.

---

## 📖 Documentation

**For detailed information, see:**
- **EXAMPLES.md** - Complete walkthrough examples (start here if new to code review!)
- **SKILL.md** - Main review workflow and procedures
- **REFERENCE.md** - Complete OWASP Top 10, CWE Top 25, performance patterns
- **FORMS.md** - Template overview and usage guide
- **resources/templates/** - Ready-to-use review templates
- **resources/examples/** - Real-world good/bad review examples

**Research Citations:**
- OWASP Code Review Guide: https://owasp.org/www-project-code-review-guide/
- OWASP Top 10 2021: https://owasp.org/Top10/
- CWE Top 25: https://cwe.mitre.org/top25/
- Empirical Study (2024): https://arxiv.org/html/2311.16396v2

---

## 🔄 Integration with Other Skills

**Works well with:**
- **qa-testing** - Code review identifies issues, QA testing verifies fixes
- **feature-orchestrator** - Reviews features during implementation phase
- **critic-agent** - Code-reviewer for quick checks, critic-agent for deep audits
- **devops-deployment** - Pre-deployment security validation

---

## ⚙️ Customization

### Adjust Severity Thresholds

Edit `SKILL.md` to change what's considered critical vs high priority for your project.

### Add Custom Security Rules

Create files in `resources/` directory with project-specific patterns to check.

### Integrate Additional Tools

Add tool-specific scripts to `scripts/` directory and update quick-audit scripts.

---

## 📊 Metrics to Track

**From 2025 Research:**

1. **Mean Time to Remediate (MTTR)**
   - Target: < 7 days for high severity
   - Critical issues: < 24 hours

2. **Defect Density**
   - Formula: (# of bugs) / (1000 lines of code)
   - Target: < 1.0 defects per 1000 LOC

3. **Review Coverage**
   - Target: 100% of changed lines reviewed

4. **False Positive Rate**
   - Track your project's rate to calibrate trust in automated tools

---

## 🤝 Contributing

Found a new security pattern? Improved a checklist? Submit updates to:

- Your shared skills repository or internal documentation source
- License: CC BY 4.0 (attribution required)

---

## 📜 License

### Creative Commons Attribution 4.0 International (CC BY 4.0)

Created by: [Madina Gbotoe](https://madinagbotoe.com/)

Attribution Required: Yes - Include author name and link when sharing/modifying

---

**Remember:** Code review is not about finding fault—it's about ensuring quality, security, and maintainability. Be constructive, specific, and always suggest solutions alongside identifying problems.
