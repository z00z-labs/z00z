# Code Refactoring Templates & Checklists

**Ready-to-use templates for refactoring sessions.**

---

## Table of Contents

1. [Technical Debt Prioritization Framework](#technical-debt-prioritization-framework)
2. [Legacy Codebase Refactoring Roadmap](#legacy-codebase-refactoring-roadmap)
3. [Refactoring Analysis Template](#refactoring-analysis-template)
4. [Pre-Refactoring Checklist](#pre-refactoring-checklist)
5. [Refactoring Execution Plan](#refactoring-execution-plan)
6. [Post-Refactoring Verification](#post-refactoring-verification)
7. [Rust Refactoring Handoff](#rust-refactoring-handoff)
8. [Language-Specific Checklists](#language-specific-checklists)

---

## Technical Debt Prioritization Framework

**Use this framework when you have multiple files needing refactoring and must decide which to tackle first.**

### Priority Scoring Matrix

Calculate priority score for each file using this formula:

```
Priority Score = (Size Factor) + (Change Frequency Factor) + (Business Impact Factor) + (Risk Factor)

Where:
- Size Factor = (Current Lines / 100) × 1.0
- Change Frequency Factor = Commits per Month × 2.0
- Business Impact Factor = Impact Rating (1-3) × 3.0
- Risk Factor = Complexity Rating (1-3) × 1.5
```

### Scoring Template

```markdown
# Technical Debt Priority Assessment

**Assessment Date:** [Date]
**Codebase:** [Project Name]

---

## Files Under Assessment

### File 1: [path/to/file.tsx]

**Size Metrics:**
- Current lines: 450
- Size factor: 450 / 100 = 4.5

**Change Frequency:**
- Commits (last 3 months): 24
- Commits per month: 8
- Change frequency factor: 8 × 2.0 = 16.0

**Business Impact:**
- [ ] Low (1): Minor feature, few users affected
- [ ] Medium (2): Important feature, significant users
- [X] High (3): Critical feature, all users affected
- Business impact factor: 3 × 3.0 = 9.0

**Risk/Complexity:**
- [ ] Low (1): Simple logic, good test coverage
- [ ] Medium (2): Moderate complexity, some tests
- [X] High (3): Complex logic, poor test coverage
- Risk factor: 3 × 1.5 = 4.5

**Total Priority Score:** 4.5 + 16.0 + 9.0 + 4.5 = **34.0**
**Priority Level:** P0 (Critical)

---

### File 2: [path/to/file2.py]

**Size Metrics:**
- Current lines: 280
- Size factor: 280 / 100 = 2.8

**Change Frequency:**
- Commits (last 3 months): 6
- Commits per month: 2
- Change frequency factor: 2 × 2.0 = 4.0

**Business Impact:**
- [ ] Low (1)
- [X] Medium (2)
- [ ] High (3)
- Business impact factor: 2 × 3.0 = 6.0

**Risk/Complexity:**
- [ ] Low (1)
- [X] Medium (2)
- [ ] High (3)
- Risk factor: 2 × 1.5 = 3.0

**Total Priority Score:** 2.8 + 4.0 + 6.0 + 3.0 = **15.8**
**Priority Level:** P1 (High)

---

[Continue for all files...]

---

## Priority Level Guidelines

**P0 - Critical (Score ≥ 25)**
- **Action:** Fix within 1 week
- **Rationale:** High size + high change frequency + critical impact
- **Approach:** Dedicated sprint, full team focus

**P1 - High (Score 15-24)**
- **Action:** Fix within 1 month
- **Rationale:** Moderate to high concern in multiple areas
- **Approach:** Regular sprint work, pair programming

**P2 - Medium (Score 10-14)**
- **Action:** Fix within quarter
- **Rationale:** Some concern, but manageable
- **Approach:** Opportunistic refactoring when making changes

**P3 - Low (Score <10)**
- **Action:** Fix when convenient
- **Rationale:** Low concern, stable code
- **Approach:** Include in larger refactoring initiatives

---

## Prioritized Refactoring List

**Based on scores above:**

| Rank | File | Score | Priority | Timeline |
|------|------|-------|----------|----------|
| 1 | file.tsx | 34.0 | P0 | Week 1 |
| 2 | file2.py | 15.8 | P1 | Week 3 |
| ... | ... | ... | ... | ... |

---

## Resource Allocation

**P0 Files:**
- Developer allocation: 2 developers
- Estimated time: [X] hours
- Sprint allocation: Sprint [X]

**P1 Files:**
- Developer allocation: 1 developer
- Estimated time: [X] hours
- Sprint allocation: Sprint [X-Y]

**P2 Files:**
- Developer allocation: As available
- Estimated time: [X] hours
- Sprint allocation: Ongoing

---

```

### Quick Prioritization Guide

**When you don't have time for full scoring:**

1. **Is file >400 lines AND changed this month?** → P0
2. **Is file >300 lines AND critical business feature?** → P0
3. **Is file >300 lines OR changed weekly?** → P1
4. **Is file >200 lines AND changed monthly?** → P1
5. **Is file >200 lines OR changed quarterly?** → P2
6. **Everything else** → P3

---

## Legacy Codebase Refactoring Roadmap

**Use this template when planning refactoring for an inherited or long-neglected codebase.**

```markdown
# Legacy Codebase Refactoring Roadmap

**Project:** [Project Name]
**Assessment Date:** [Date]
**Target Completion:** [Date, typically 3-6 months out]
**Document Owner:** [Name]

---

## Current State Assessment

### Codebase Health Metrics

| Metric | Current | Industry Standard | Gap |
|--------|---------|-------------------|-----|
| Average file size | [X] lines | <150 lines | +[X] lines |
| Files >300 lines | [X] files | 0 files | [X] files |
| Test coverage | [X]% | >80% | -[X]% |
| Technical debt ratio | [X]% | <5% | +[X]% |

### Problem Summary

**Total files scanned:** [X]
**Files needing refactoring:** [X] ([X]%)

- 🛑 Critical (>300 lines): [X] files
- 🚨 High (200-300 lines): [X] files
- ⚠️ Medium (150-200 lines): [X] files

**Estimated total refactoring effort:** [X] hours ([X] weeks)

---

## Goals & Success Criteria

### 3-Month Goals

- [ ] **Primary:** All critical files (<300 lines) refactored
- [ ] **Secondary:** 50% of high-priority files refactored
- [ ] **Tertiary:** No new files created >200 lines

### Success Metrics

| Metric | Baseline | 3-Month Target | 6-Month Target |
|--------|----------|----------------|----------------|
| Critical files | [X] | 0 | 0 |
| High priority files | [X] | [X/2] | 0 |
| Average file size | [X] | <200 | <150 |
| Test coverage | [X]% | [X+10]% | >80% |

---

## Phase 1: Discovery & Planning (Weeks 1-2)

### Week 1: Assessment

- [ ] Run codebase audit script
- [ ] Generate priority scores for all files
- [ ] Identify patterns and anti-patterns
- [ ] Document current architecture

**Deliverables:**
- Technical debt assessment report
- Prioritized refactoring list
- Architecture diagram (current state)

### Week 2: Planning

- [ ] Create refactoring plans for P0 files
- [ ] Estimate time for each refactor
- [ ] Identify dependencies between files
- [ ] Plan test strategy

**Deliverables:**
- Detailed refactoring plans (use Refactoring Execution Plan template)
- Sprint allocation plan
- Risk assessment

---

## Phase 2: Critical Files (Weeks 3-6)

### Sprint 1 (Weeks 3-4): Top P0 Files

**Target:** Top [X] critical files

| File | Current Lines | Target Lines | Estimated Time | Assignee |
|------|---------------|--------------|----------------|----------|
| file1.tsx | 450 | <200 | 8 hours | [Name] |
| file2.py | 420 | <200 | 6 hours | [Name] |
| ... | ... | ... | ... | ... |

**Approach:**
1. Extract data/configuration files
2. Split into logical sub-components/modules
3. Add missing tests
4. Verify functionality unchanged

**Success Criteria:**
- [ ] All targeted files <300 lines
- [ ] Test coverage improved by [X]%
- [ ] No regressions introduced

### Sprint 2 (Weeks 5-6): Remaining P0 Files

[Repeat structure from Sprint 1]

---

## Phase 3: High Priority Files (Weeks 7-12)

### Sprint 3-4: P1 Files (Month 2)

**Target:** [X] high-priority files

**Approach:** Opportunistic refactoring
- Refactor files when making feature changes
- Batch similar refactorings together
- Focus on highest-scoring P1 files first

---

## Phase 4: Medium Priority & Prevention (Weeks 13+)

### Ongoing Activities

**Refactoring:**
- [ ] Address P2 files opportunistically
- [ ] Refactor any P3 files if quick wins available

**Prevention:**
- [ ] Enable code-refactoring skill for all developers
- [ ] Add file size limits to CI/CD
- [ ] Implement pre-commit hooks
- [ ] Update code review checklist

**Monitoring:**
- [ ] Monthly codebase audits
- [ ] Track refactoring metrics
- [ ] Celebrate successes

---

## Risk Management

### Identified Risks

**Risk 1: Breaking changes during refactoring**
- **Likelihood:** Medium
- **Impact:** High
- **Mitigation:**
  - Require 80% test coverage before refactoring
  - Use feature flags for major changes
  - Incremental refactoring with frequent commits
- **Contingency:** Have rollback plan for each file

**Risk 2: Timeline overruns**
- **Likelihood:** High
- **Impact:** Medium
- **Mitigation:**
  - Build 25% buffer into estimates
  - Prioritize ruthlessly (P0 only if needed)
  - Pair programming for complex refactors
- **Contingency:** Extend timeline, reduce scope

**Risk 3: New technical debt created**
- **Likelihood:** Medium
- **Impact:** Medium
- **Mitigation:**
  - Code review all refactors
  - Enforce file size limits
  - Regular monitoring
- **Contingency:** Immediate remediation of new issues

---

## Resource Plan

### Team Allocation

**Full-time equivalent (FTE) needed:** [X] FTE

| Phase | Duration | FTE Required | Team Members |
|-------|----------|--------------|--------------|
| Phase 1: Planning | 2 weeks | 0.5 FTE | [Names] |
| Phase 2: Critical | 4 weeks | 1.0 FTE | [Names] |
| Phase 3: High Priority | 6 weeks | 0.5 FTE | [Names] |
| Phase 4: Ongoing | Ongoing | 0.25 FTE | [Names] |

### Budget

**Estimated total cost:** $[X] ([X] hours × $[Y]/hour)

- Phase 1: $[X]
- Phase 2: $[X]
- Phase 3: $[X]
- Phase 4: $[X]/month

---

## Communication Plan

### Stakeholder Updates

**Weekly:** Team standup on refactoring progress
**Biweekly:** Management summary (metrics + blockers)
**Monthly:** Detailed report with trends

### Reporting Template

```
## Refactoring Update: Week [X]

**Progress:**
- Files refactored this week: [X]
- Total files refactored: [X] of [Y] ([X]%)
- Lines of code reduced: [X]

**Metrics:**
- Critical files remaining: [X]
- Average file size: [X] lines (was [Y])
- Test coverage: [X]% (was [Y]%)

**Blockers:**
- [Issue 1 and mitigation]

**Next week:**
- Target files: [list]
```

---

## Lessons Learned (Update at end of each phase)

### What Went Well
- [Success 1]
- [Success 2]

### What Could Be Improved
- [Challenge 1 and solution]
- [Challenge 2 and solution]

### Best Practices Identified
- [Best practice 1]
- [Best practice 2]

---

## Approval & Sign-off

**Roadmap Approved By:**
- [ ] Engineering Manager: __________ Date: ______
- [ ] Tech Lead: __________ Date: ______
- [ ] Product Owner: __________ Date: ______

**Phase Completion Sign-offs:**

**Phase 1:** __________ Date: ______
**Phase 2:** __________ Date: ______
**Phase 3:** __________ Date: ______
**Phase 4:** __________ Date: ______

---

```

---

## Refactoring Analysis Template

**Use this when analyzing a file for refactoring:**

```markdown
# Refactoring Analysis: [Filename]

**Date:** [Current Date]
**Analyst:** [Your Name / AI]
**Language:** [JavaScript/TypeScript/Python/Other]

---

## Current State

### File Information
- **Path:** `[full/path/to/file]`
- **Size:** [X] lines
- **Type:** [Component/Module/Class/Service]
- **Status:** [✅ Good / ⚠️ Warning / 🛑 Critical]

### Size Breakdown
| Category | Lines | Percentage |
|----------|-------|------------|
| Main logic | [X] | [X]% |
| Data/Config | [X] | [X]% |
| Helper functions | [X] | [X]% |
| Types/Interfaces | [X] | [X]% |
| Imports/Exports | [X] | [X]% |
| Comments/Docs | [X] | [X]% |
| **Total** | **[X]** | **100%** |

---

## Identified Issues

### Critical Issues (🛑 Must Fix)
- [ ] File exceeds 300 lines
- [ ] Multiple responsibilities detected
- [ ] Complex nested logic (>3 levels)
- [ ] [Other critical issue]

### Warnings (⚠️ Should Fix)
- [ ] File exceeds 200 lines
- [ ] Large data structures inline
- [ ] Repeated code patterns
- [ ] [Other warning]

### Improvements (💡 Nice to Have)
- [ ] Better naming conventions
- [ ] Add documentation
- [ ] Performance optimizations
- [ ] [Other improvement]

---

## Refactoring Recommendations

### Priority 1: [Most Critical Action]

**Recommendation:** Extract [component/class/function/data]

**Justification:** [Why this is priority 1]

**Target Structure:**
```
[proposed file structure]
```

**Estimated Impact:**
- Lines reduced in main file: [X] lines
- New files created: [count]
- Estimated time: [X] hours/minutes
- Risk level: [Low/Medium/High]

### Priority 2: [Second Priority]

**Recommendation:** [description]

**Justification:** [Why this matters]

**Target Structure:**
```
[proposed structure]
```

**Estimated Impact:**
- Lines reduced: [X] lines
- Files created: [count]
- Estimated time: [X] hours/minutes
- Risk level: [Low/Medium/High]

### Priority 3: [Third Priority]

[Continue as needed...]

---

## Expected Results

### After Refactoring
- **Main file:** [X] lines (from [Y] lines)
- **Extracted files:** [list with sizes]
- **Total lines:** [X] lines (distributed across files)
- **Maintainability:** [Improved/Same/Degraded]

### Benefits
- ✅ [Benefit 1]
- ✅ [Benefit 2]
- ✅ [Benefit 3]

### Risks
- ⚠️ [Risk 1 and mitigation]
- ⚠️ [Risk 2 and mitigation]

---

## Decision

**Proceed with refactoring?** [YES / NO / PARTIALLY]

**Rationale:** [Explanation of decision]

**Selected priorities:** [Which priorities to implement]

**Timeline:** [When to execute]

---

## Sign-off

**Reviewed by:** [Name]
**Date:** [Date]
**Approved:** [YES / NO / PENDING]
```

---

## Pre-Refactoring Checklist

**Complete BEFORE starting refactoring:**

```markdown
## Pre-Refactoring Checklist: [Filename]

### Preparation
- [ ] Current code is committed to version control
- [ ] All tests are passing
- [ ] No pending changes in working directory
- [ ] Refactoring plan documented
- [ ] Team notified (if working in team)

### Understanding
- [ ] Read and understood entire file
- [ ] Identified all dependencies
- [ ] Mapped data flow
- [ ] Understood business logic
- [ ] Reviewed related files

### Testing
- [ ] Test suite exists for this file
- [ ] Test coverage is adequate (>70%)
- [ ] Tests are passing
- [ ] Performance baseline established
- [ ] Edge cases documented

### Backup
- [ ] Created backup branch
- [ ] Tagged current state
- [ ] Documented current behavior
- [ ] Captured current metrics

### Tools
- [ ] Linter configured
- [ ] Type checker enabled
- [ ] Test runner ready
- [ ] Code review tool available

### Risk Assessment
- [ ] Identified breaking change potential: [Low/Medium/High]
- [ ] Impact on other files: [List files]
- [ ] Rollback plan documented
- [ ] Stakeholders aware of changes

---

**Ready to proceed?** [YES / NO]

**If NO, what's missing?** [List blockers]
```

---

## Refactoring Execution Plan

**Use this to execute refactoring step-by-step:**

```markdown
## Refactoring Execution Plan: [Filename]

**Date Started:** [Date]
**Target Completion:** [Date]
**Language:** [JavaScript/TypeScript/Python/Other]

---

### Step 1: [First Action]

**Action:** Extract [component/function/data]

**From:** `[original-file]`
**To:** `[new-file-path]`

**Procedure:**
1. Create new file: `[new-file-path]`
2. Copy relevant code (lines [X]-[Y])
3. Update imports in original file
4. Update imports in new file
5. Run tests
6. Commit with message: "refactor: extract [description]"

**Verification:**
- [ ] Code extracted successfully
- [ ] All imports resolved
- [ ] Tests passing
- [ ] Linter happy
- [ ] Type checker passing
- [ ] No functionality broken

**Estimated Time:** [X] minutes
**Actual Time:** [X] minutes
**Status:** [✅ Complete / 🚧 In Progress / ⏸️ Blocked]
**Notes:** [Any issues or observations]

---

### Step 2: [Second Action]

**Action:** [description]

**From:** `[source]`
**To:** `[destination]`

**Procedure:**
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Verification:**
- [ ] [Check 1]
- [ ] [Check 2]
- [ ] [Check 3]

**Estimated Time:** [X] minutes
**Actual Time:** [X] minutes
**Status:** [Status]
**Notes:** [Notes]

---

[Continue for all steps...]

---

### Final Integration

**Action:** Integrate all refactored components

**Procedure:**
1. Review all changes
2. Run full test suite
3. Check performance metrics
4. Update documentation
5. Final commit

**Verification:**
- [ ] All tests passing
- [ ] No performance regression
- [ ] Documentation updated
- [ ] Code review completed
- [ ] Ready for merge

---

### Summary

**Total Steps:** [X]
**Completed:** [X]
**Time Estimated:** [X] hours
**Time Actual:** [X] hours
**Success:** [YES / PARTIAL / NO]

**Lessons Learned:**
- [Lesson 1]
- [Lesson 2]
- [Lesson 3]
```

---

## Post-Refactoring Verification

**Complete AFTER refactoring:**

```markdown
## Post-Refactoring Verification: [Filename]

**Date Completed:** [Date]
**Refactored by:** [Name]

---

### Code Quality Checks

#### Structure
- [ ] Main file within size target (<200 lines for most)
- [ ] All extracted files within size targets
- [ ] Clear separation of concerns
- [ ] No circular dependencies
- [ ] Proper file organization

#### Code Standards
- [ ] Follows naming conventions
- [ ] No linter errors
- [ ] No type errors
- [ ] Consistent formatting
- [ ] Comments/documentation updated

#### Functionality
- [ ] All tests passing
- [ ] No new bugs introduced
- [ ] Edge cases still handled
- [ ] Error handling preserved
- [ ] Business logic unchanged

#### Performance
- [ ] No performance regression
- [ ] Bundle size acceptable
- [ ] Load time maintained or improved
- [ ] Memory usage acceptable

---

### Testing Verification

#### Unit Tests
- [ ] All existing tests pass
- [ ] New tests added for extracted code
- [ ] Test coverage maintained or improved
- [ ] Edge cases tested

#### Integration Tests
- [ ] Integration tests pass
- [ ] No breaking changes detected
- [ ] Dependencies work correctly

#### Manual Testing
- [ ] Manually tested main flow
- [ ] Tested error scenarios
- [ ] Tested edge cases
- [ ] UI/UX unchanged (if applicable)

---

### Documentation Updates

- [ ] Code comments updated
- [ ] README updated (if needed)
- [ ] Architecture docs updated
- [ ] API documentation updated
- [ ] Changelog entry added

---

### Metrics Comparison

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Main file size | [X] lines | [Y] lines | [+/-Z] |
| Total lines | [X] lines | [Y] lines | [+/-Z] |
| Test coverage | [X]% | [Y]% | [+/-Z]% |
| Performance | [X]ms | [Y]ms | [+/-Z]ms |
| Bundle size | [X]KB | [Y]KB | [+/-Z]KB |

---

### Success Criteria

- [x] Main file <200 lines: [YES / NO]
- [x] All tests passing: [YES / NO]
- [x] No performance regression: [YES / NO]
- [x] Documentation updated: [YES / NO]
- [x] Code review approved: [YES / NO]

---

### Final Assessment

**Overall Success:** [✅ Success / ⚠️ Partial / ❌ Failed]

**Benefits Achieved:**
- [Benefit 1]
- [Benefit 2]
- [Benefit 3]

**Issues Encountered:**
- [Issue 1 and resolution]
- [Issue 2 and resolution]

**Recommendations:**
- [Future improvement 1]
- [Future improvement 2]

---

**Signed Off By:** [Name]
**Date:** [Date]
```

---

## Rust Refactoring Handoff

Rust-specific forms and checklists now live in the dedicated
`../rust-refactoring/FORMS.md` file.

Use the Rust-specific package when the task centers on:

- `.rs` files or `Cargo.toml`
- ownership or clone pressure
- `thiserror` or typed error cleanup
- async `Send` or lock-boundary issues
- stage or module splits inside a crate

---

## Language-Specific Checklists

### JavaScript/TypeScript/React Checklist

```markdown
## React Component Refactoring Checklist

### Component Size
- [ ] Component <200 lines
- [ ] Each sub-component <150 lines
- [ ] Data extracted to separate files

### React Best Practices
- [ ] React.memo() used for components with animations
- [ ] useMemo() for expensive computations
- [ ] useCallback() for callbacks passed to children
- [ ] No inline function definitions in JSX
- [ ] Keys used properly in lists

### Performance
- [ ] No unnecessary re-renders
- [ ] Lazy loading for heavy components
- [ ] Code splitting at route boundaries
- [ ] Images optimized (if applicable)

### Imports
- [ ] No file extensions in imports
- [ ] Absolute imports used (@/ prefix)
- [ ] Imports organized (React, libraries, local, styles)
- [ ] No unused imports

### TypeScript
- [ ] Proper types/interfaces defined
- [ ] No 'any' types (unless justified)
- [ ] Props interface defined
- [ ] Return types specified for functions

### Hooks
- [ ] Custom hooks extracted (if 4+ hooks)
- [ ] Hook dependencies correct
- [ ] No hooks inside conditionals
- [ ] Cleanup in useEffect when needed

### Testing
- [ ] Component tests exist
- [ ] Hook tests exist (if custom hooks)
- [ ] Snapshot tests updated
- [ ] Interaction tests pass
```

### Python Checklist

```markdown
## Python Module/Class Refactoring Checklist

### File Size
- [ ] Module <400 lines (or split into package)
- [ ] Class <300 lines
- [ ] Function <50 lines
- [ ] No method >30 lines

### Python Best Practices
- [ ] PEP 8 compliant
- [ ] Type hints used (Python 3.6+)
- [ ] Docstrings for all public methods
- [ ] No mutable default arguments
- [ ] Context managers for resources

### Class Design
- [ ] Single Responsibility Principle
- [ ] No God classes (too many methods)
- [ ] Composition over inheritance
- [ ] __init__ not too complex (<20 lines)
- [ ] Private methods prefixed with _

### Configuration
- [ ] No magic numbers
- [ ] Constants extracted to config file
- [ ] Environment variables for settings
- [ ] Configuration validated

### Imports
- [ ] Standard library imports first
- [ ] Third-party imports second
- [ ] Local imports last
- [ ] No circular imports

### Error Handling
- [ ] Specific exceptions caught
- [ ] No bare except clauses
- [ ] Proper error messages
- [ ] Logging in place

### Testing
- [ ] Unit tests exist
- [ ] Edge cases tested
- [ ] Mocks used appropriately
- [ ] Test coverage >80%
```

### Rust Checklist

Use the dedicated Rust checklist in `../rust-refactoring/FORMS.md`.

---

## Quick Reference: When to Extract

### Extract to Separate File When

**JavaScript/TypeScript:**

- ✅ 5+ data items in component
- ✅ Array >20 lines
- ✅ 4+ hooks
- ✅ Modal/dialog code
- ✅ Complex form

**Python:**

- ✅ 5+ configuration variables
- ✅ Class >300 lines
- ✅ Function >50 lines
- ✅ Module >400 lines
- ✅ Multiple responsibilities

**Universal:**

- ✅ Repeated code across files
- ✅ Complex conditionals (>5 branches)
- ✅ Deeply nested logic (>3 levels)
- ✅ Large data structures (>20 lines)

---

## Refactoring Time Estimates

**Use these as rough guidelines:**

| Task | Time Estimate |
| --- | --- |
| Extract data to file | 10-15 minutes |
| Extract sub-component | 20-30 minutes |
| Extract custom hook | 30-45 minutes |
| Split class into 2 classes | 45-60 minutes |
| Convert module to package | 1-2 hours |
| Full feature refactor | 2-4 hours |

**Always add 50% buffer for testing and fixes!**

---

## Rollback Plan Template

```markdown
## Rollback Plan: [Refactoring]

### If Refactoring Fails

**Trigger Conditions:**
- Tests fail and can't be fixed in 30 minutes
- Performance regression >20%
- Breaking changes discovered
- Critical bugs introduced

**Rollback Procedure:**
1. Stop all changes immediately
2. Stash or discard uncommitted changes
3. Revert to backup branch: `git checkout [backup-branch]`
4. Verify tests pass on backup branch
5. Deploy backup branch if necessary
6. Document what went wrong
7. Create post-mortem

**Backup Locations:**
- Branch: `[backup-branch-name]`
- Tag: `[pre-refactor-tag]`
- Commit: `[commit-hash]`

**Notification List:**
- [Person/team to notify]
- [Person/team to notify]

**Recovery Time Objective:** [X] minutes
```

---

## Additional Resources

- See `SKILL.md` for when to invoke refactoring
- See `REFERENCE.md` for detailed patterns and examples
- Use `scripts/check-size.sh` for quick file size checks
