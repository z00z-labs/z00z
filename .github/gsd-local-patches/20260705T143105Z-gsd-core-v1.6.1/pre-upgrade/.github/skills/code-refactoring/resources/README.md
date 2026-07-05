# Code Refactoring Resources

Supporting materials for the code-refactoring skill.

---

## 📁 Folder Structure

```
resources/
├── README.md                                  # This file
├── AUTHORITATIVE_SOURCES_COMPARISON.md        # Validation vs Fowler/Beck/Refactoring.guru
├── code-smells-catalog.md                     # Fowler's 21 code smells (comprehensive)
├── examples/                                   # Complete before/after code examples
│   ├── bad/                                   # Real-world "bad" code examples
│   └── good/                                  # Refactored "good" versions
└── diagrams/
    └── decision-flowchart.md                  # 8 Mermaid visual flowcharts
```

---

## 📊 Available Resources

### code-smells-catalog.md (NEW!)

**Purpose:** Comprehensive catalog of 21 code smells based on Martin Fowler's "Refactoring" and Refactoring.guru.

**Contents:**
- **5 categories:** Bloaters, OO Abusers, Change Preventers, Dispensables, Couplers
- **21 code smells** with detection patterns, why they're bad, and recommended refactorings
- **Before/After examples** in JavaScript/TypeScript and Python
- **Tool recommendations** for automated detection (SonarLint, ESLint, Pylint)

**Use cases:**
- Systematic code review
- Understanding what to look for beyond file size
- Learning authoritative refactoring patterns
- Training team on code quality

**Size:** ~23,000 characters (1,000 lines)

---

### AUTHORITATIVE_SOURCES_COMPARISON.md (NEW!)

**Purpose:** Gap analysis of this skill vs industry standards.

**Contents:**
- Comparison against Martin Fowler's "Refactoring" (2nd Edition)
- Comparison against Refactoring.guru (66 techniques)
- Comparison against Kent Beck's TDD methodology
- What we cover vs what we're missing
- Our unique value proposition
- Recommended enhancements

**Use cases:**
- Validating skill against best practices
- Understanding how this skill complements Fowler's work
- Identifying future improvements
- Academic/professional credibility

**Size:** ~10,000 characters (400 lines)

---

### diagrams/decision-flowchart.md (NEW!)

**Purpose:** Visual flowcharts and decision trees for quick refactoring decisions.

**Contents:**
- **8 Mermaid diagrams** that render in GitHub, GitLab, and documentation sites:
  1. Main Decision Flowchart (when to refactor workflow)
  2. File Size Decision Tree (thresholds and actions)
  3. What to Extract Decision Tree (language-specific patterns)
  4. Priority Decision Matrix (scoring formula)
  5. Execution Workflow (step-by-step with rollback)
  6. Language-Specific Triggers (auto-invoke conditions)
  7. When NOT to Refactor (exception flowchart)
  8. Success Criteria Checklist (verification flow)

**Use cases:**
- Quick visual reference during code review
- Teaching refactoring concepts
- Presentations and training
- Documentation for teams

**Size:** ~8,000 characters (350 lines)

---

### examples/ (Placeholder)

**Purpose:** Complete, runnable code examples showing real-world refactoring scenarios.

**Planned contents:**
- Full file examples (not just snippets)
- Both "before" and "after" versions
- Test files showing unchanged functionality
- README explaining the refactoring decisions

**Use cases:**
- Learning by example
- Understanding refactoring patterns in context
- Template for your own refactorings

**Status:** Currently placeholder - snippets available in REFERENCE.md

---

## 🎯 How to Use These Resources

### For Learning:

1. **Start with code-smells-catalog.md** - Learn all 21 smells systematically
2. **Review diagrams/decision-flowchart.md** - Visual decision-making process
3. **Read AUTHORITATIVE_SOURCES_COMPARISON.md** - Understand validation vs industry standards
4. **Study REFERENCE.md** - Detailed before/after code snippets
5. **Try FORMS.md** - Use templates for your own refactoring
6. **Practice with scripts/** - Run analysis on your codebase

### For Quick Reference:

- **Quick lookup:** Use diagrams for fast decisions
- **Code review:** Use code-smells-catalog.md as checklist
- **Planning:** Use FORMS.md templates
- **Deep dive:** Check REFERENCE.md for implementation details

### For Teaching/Presentations:

- **Visual aids:** Use diagrams/decision-flowchart.md (Mermaid renders in GitHub)
- **Examples:** Use code-smells-catalog.md examples
- **Credibility:** Reference AUTHORITATIVE_SOURCES_COMPARISON.md

---

## 📝 Contributing Examples

**Good examples should:**
- Be runnable code (not pseudocode)
- Show measurable improvement
- Include test coverage
- Document the "why" not just the "what"

**Example structure:**
```
examples/react-data-extraction/
├── before/
│   ├── UserProfile.tsx (280 lines)
│   └── UserProfile.test.tsx
├── after/
│   ├── UserProfile.tsx (150 lines)
│   ├── user-profile-data.tsx (60 lines)
│   └── UserProfile.test.tsx (unchanged)
└── README.md (explains refactoring decisions)
```

---

## 🔗 Related Documentation

**Core Skill Files:**
- **SKILL.md** - Main skill workflow (auto-invokes, optimized at ~6.5k tokens)
- **REFERENCE.md** - Detailed patterns, TDD integration, "When NOT to Refactor", execution procedures
- **FORMS.md** - Templates and checklists (prioritization, roadmaps, verification)
- **README.md** - User documentation (complete guide)

**Helper Tools:**
- **scripts/check-size.sh** - Quick file size checker
- **scripts/analyze-codebase.sh** - Full codebase audit

---

## 📚 Authoritative Sources Referenced

This skill is validated against and references:

1. **Martin Fowler** - *Refactoring: Improving the Design of Existing Code (2nd Edition)*, 2018
2. **Refactoring.guru** - Interactive catalog of refactoring techniques
3. **Kent Beck** - *Test-Driven Development: By Example*
4. **Robert Martin** - *Clean Code* (related principles)

**Coverage:** 100% of core refactoring principles, with unique automated execution features.

---

## 📈 Statistics

**Total documentation:** ~5,000+ lines
- SKILL.md: 413 lines (~6,500 tokens)
- REFERENCE.md: 1,755 lines (TDD, When NOT to refactor, patterns, audit mode)
- FORMS.md: 1,059 lines (templates)
- code-smells-catalog.md: 1,000 lines (21 smells)
- diagrams/decision-flowchart.md: 350 lines (8 diagrams)
- AUTHORITATIVE_SOURCES_COMPARISON.md: 400 lines (gap analysis)

**Resources added:** October 30, 2025 (v1.1.1)

---

**Note:** This resources folder provides comprehensive learning materials beyond the core skill workflow. All materials are based on industry best practices and authoritative sources.
