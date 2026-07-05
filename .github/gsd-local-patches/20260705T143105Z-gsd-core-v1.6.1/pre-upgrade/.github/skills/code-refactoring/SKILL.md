---
name: code-refactoring
description: Primary refactoring entry point for general and mixed-language requests. Proactively monitors file sizes, handles "refactor this" and maintainability cleanup requests, and explicitly delegates Rust-only workflows to rust-refactoring when Rust triggers fire.
---

<!--
Created by: Madina Gbotoe (https://madinagbotoe.com/)
Version: 2.0.0
Created: October 30, 2025
Last Updated: October 31, 2025
License: Creative Commons Attribution 4.0 International (CC BY 4.0)
Attribution Required: Yes - Include author name and link when sharing/modifying

Purpose: Universal code refactoring skill - Works across multiple languages (JavaScript/TypeScript/React, Python, and general patterns). Proactively prevents code complexity by catching size and pattern issues before they become problems.

NEW in v2.0: Path-based thresholds! The watcher now intelligently adjusts thresholds based on file location (pages can be larger than utilities, data files get different limits than logic files).

Language Support:
- JavaScript/TypeScript/React: Component splitting, hook extraction, data files
- Python: Class/module splitting, function extraction, configuration management
- General: Universal patterns that work across all languages
-->

# Code Refactoring Skill

## 🚪 Primary Entry Point

Use `code-refactoring` as the default refactoring skill when the request is
general, ambiguous, or spans more than one language.

This skill owns the first-pass routing for prompts such as:

- "refactor this"
- "clean up this code"
- "improve maintainability"
- "split this file"
- "reduce complexity before adding features"

If the target is clearly Rust-first, this skill should hand the task off to the
dedicated `rust-refactoring` skill instead of letting two general-purpose
entries compete.

## When to Use

Use this skill when the request is about structural cleanup, maintainability,
safe file splitting, or complexity reduction without changing behavior.

Typical triggers:

- The user says "refactor this", "clean this up", or "improve structure"
- A file is already large and more edits are planned
- The code has repeated logic, deep nesting, or too many responsibilities
- The task spans multiple languages or is still ambiguous at first glance
- A Rust-only request must be triaged and then handed off cleanly to
  `rust-refactoring`

## 🎯 Purpose: Prevent Complexity Before It Becomes a Problem

**Problem this solves:** "I rarely know when to refactor, typically get caught by accident"

**Solution:** AI proactively monitors file sizes and complexity patterns, alerting you BEFORE making changes that would worsen the problem.

**Works with:** JavaScript/TypeScript (React, Node), Python, and general codebases.


## 🔔 CRITICAL FIRST STEP: Check for File Watcher Alerts

**BEFORE doing anything else when this skill is invoked, ALWAYS check for unread alerts from the background file watcher:**

```bash
node .github/skills/code-refactoring/scripts/check-alerts.js
```

**Why this is critical:**
- The file watcher runs in the background monitoring code file sizes
- It writes alerts to `watcher-alerts.json` when files are edited and exceed thresholds
- These alerts tell you which files the user just edited that need refactoring attention
- **You must check these alerts FIRST** to provide real-time feedback

**If alerts exist:**
1. Display the alerts using the check-alerts.js script output
2. Ask the user if they want help refactoring the alerted files NOW or LATER
3. Then proceed with the rest of the skill workflow

**If no alerts:**
- Continue with normal skill workflow (checking file sizes manually, etc.)

**User can also manually check alerts with:** `/check-refactor-alerts`

---

## 🤖 AUTO-INVOKE CONDITIONS (AI Should Check These)

### 0. General Refactor Requests (PRIMARY ROUTE)

Use this skill first when the user asks for refactoring in general terms, even
before size thresholds are checked.

Common triggers:

- "refactor this"
- "clean up this code"
- "make this easier to maintain"
- "split this file/module"
- "improve structure without changing behavior"

Then decide whether to stay in this skill or delegate to `rust-refactoring`.

### 1. Before Editing Files (PREVENTIVE)

**When AI is about to edit a file, check size first:**

```bash
# Check file size before editing
wc -l [target-file]

# If result:
# 150-200 lines → ⚠️ WARN: Getting large, plan extraction
# 200-300 lines → 🚨 ALERT: Should split before adding more
# 300+ lines   → 🛑 STOP: Must refactor before editing
```

**Example workflow:**
```
User: "Add new feature to UserProfile.tsx"
AI: *Before editing, checks file size*
AI: *Sees 280 lines*
AI: *Auto-invokes code-refactoring skill*
Skill: "🚨 UserProfile.tsx is 280 lines. Before adding features:
       - Extract data to separate file
       - Split modal into separate component
       - Then add new feature safely"
```

Use the same escalation pattern during file creation and after reading an
oversized file: pause, surface the threshold breach, and propose extraction
before adding more logic.

### 2. During File Creation (PROACTIVE)

**While creating new code file, track size:**

- At 100 lines → ✅ Good, continue
- At 150 lines → ⚠️ Pause, suggest structure
- At 200 lines → 🚨 Stop, extract now

### 3. After Reading Files (REACTIVE)

**When AI reads a file and discovers size issues:**

```bash
# After reading, if file >300 lines, suggest refactor
wc -l filename

# If >300 lines, proactively suggest refactor BEFORE user asks for changes
```

### 4. Pattern Detection (AUTOMATIC)

**AI should automatically detect these patterns and invoke skill:**

#### Universal Patterns (All Languages)
- ✅ Large data structures (>20 lines) → Suggest separate config/data file
- ✅ Complex conditionals (5+ branches) → Suggest strategy pattern or dispatch table
- ✅ Deeply nested logic (>3 levels) → Suggest extraction
- ✅ Repeated code blocks → Suggest function/method extraction
- ✅ Long parameter lists (>5 params) → Suggest config object or builder pattern

#### JavaScript/TypeScript/React Patterns
- ✅ 5+ data items hardcoded → Suggest data file extraction
- ✅ 4+ useState/useEffect hooks → Suggest custom hook extraction
- ✅ Modal/Dialog code → Suggest separate component
- ✅ Complex form (10+ fields) → Suggest separate form component
- ✅ Large switch statements → Suggest lookup table or component map

#### Python Patterns
- ✅ Class >300 lines → Suggest splitting into multiple classes
- ✅ Function >50 lines → Suggest breaking into smaller functions
- ✅ Module >500 lines → Suggest splitting into package
- ✅ Too many methods (>15) → Suggest composition or mixins
- ✅ Complex __init__ (>20 lines) → Suggest factory method or builder

---

## 📏 File Size Guidelines (Language-Specific)

### JavaScript/TypeScript/React Components
- **100-200 lines**: ✅ Perfect for most components
- **200-300 lines**: ⚠️ Good for complex components, watch closely
- **300+ lines**: 🛑 MUST split into logical sub-components

### Python Modules
- **150-250 lines**: ✅ Good module size
- **250-400 lines**: ⚠️ Consider splitting by responsibility
- **400+ lines**: 🛑 Split into multiple modules or package

### Python Classes
- **100-200 lines**: ✅ Well-focused class
- **200-300 lines**: ⚠️ Check single responsibility principle
- **300+ lines**: 🛑 Split into multiple classes or use composition

### General Files (Any Language)
- **150-250 lines**: ✅ Maintainable size
- **250-400 lines**: ⚠️ Getting complex
- **400+ lines**: 🛑 Refactor needed

---

## 🔍 Refactoring Analysis Template

**When skill is invoked, provide this analysis:**

```markdown
## Code Refactoring Analysis: [Filename]

**Current State:** [X] lines - [✅ Good / ⚠️ Warning / 🛑 Critical]

**Identified Issues:**
- File size issues (>200 or >300 lines)
- Complex nested logic / too many responsibilities
- Language-specific patterns (hooks, data arrays, classes, etc.)

**Refactoring Recommendations:**
1. Extract [what] to [new-file] (saves [X] lines, low/med/high scope)
2. Extract [what] to [new-file] (saves [X] lines, low/med/high scope)

**Result:** Main file → [target] lines | Risk: [Low/Med/High] | Scope: [Small/Medium/Large]

**Proceed?** [Yes/No/Later/Review]
```

**For complete template with all fields, see `FORMS.md`**

---

## 🚨 Automatic Splitting Triggers

**When ANY of these conditions occur, AUTO-INVOKE this skill:**

### Universal Triggers (All Languages)
1. ✅ File reaches 150 lines → Suggest extraction points
2. ✅ File reaches 200 lines → Alert and plan refactor
3. ✅ File reaches 300 lines → Stop and refactor first
4. ✅ Complex nested logic detected
5. ✅ Repeated code patterns found

### JavaScript/TypeScript Triggers
1. ✅ Component reaches 150 lines → Extract next section
2. ✅ 3+ useState hooks → Extract to custom hook
3. ✅ Large data array → Move to data file immediately
4. ✅ Modal/Dialog code → Separate component from start
5. ✅ Complex form → Separate form component immediately

### Python Triggers
1. ✅ Class reaches 250 lines → Plan class split
2. ✅ Function reaches 40 lines → Suggest breaking down
3. ✅ Module reaches 350 lines → Plan module split
4. ✅ 10+ methods in class → Consider composition
5. ✅ Complex __init__ → Suggest factory pattern

---

## 📋 Quick Decision Matrix

**When to extract what (Quick Reference):**

**JavaScript/TypeScript/React:**
- **Data file**: 5+ items, arrays >20 lines, config objects
- **Sub-component**: Modal/dialog, complex list items, distinct UI sections
- **Custom hook**: 4+ useState, complex logic, reusable state

**Python:**
- **Config file**: 5+ variables, large data (>20 lines), magic numbers
- **New class**: Class >300 lines, multiple responsibilities, 10+ methods
- **Utility function**: Repeated logic, pure functions, generic operations
- **New module**: File >400 lines, distinct responsibilities, testable independently

## 🦀 Rust Routing Block

**Switch to the dedicated `rust-refactoring` skill when ANY of these are true:**
- Target files are `.rs`, `Cargo.toml`, or Rust module trees
- User mentions ownership, borrowing, lifetimes, `thiserror`, `Send`/`Sync`, async boundaries, `clippy`, `cargo`, or Rust stage/module splits
- The refactor touches multi-stage Rust files, oversized impl blocks, or crate-local public re-exports

**Critical Rust triggers:**
- File or module >400 lines, or a stage/module file owns multiple logical flows
- Function >50 lines, or one function mixes load/validate/transform/persist phases
- Repeated `clone()`, `to_owned()`, `Arc<Mutex<_>>`, or similar borrow-checker escape hatches
- Existing or new `unwrap()`/`expect()`/`String`-based errors in production paths
- Async code keeps lock guards or non-`Send` state alive across `.await`
- Domain state is encoded as strings instead of enums or newtypes

**Default Rust validation commands:**
```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

**Escalate Rust validation when needed:**
- Use targeted `cargo test -p <crate> <target>` for crate-local refactors
- Use `cargo test --release` when release-only behavior or hot paths matter
- Use `cargo miri test` for unsafe-sensitive or aliasing-sensitive refactors
- Add benchmarks before/after when clone pressure or allocation churn is part of the change

**When Rust triggers fire:**
- Hand off detailed Rust-specific guidance to `../rust-refactoring/SKILL.md`
- Use `../rust-refactoring/REFERENCE.md` for patterns and examples
- Use `../rust-refactoring/FORMS.md` for checklists and plan templates

---

## 💡 Proactive Suggestions

**AI should auto-suggest at these milestones:**
- **New file**: Monitor size, alert at 150 lines
- **100 lines**: Looking good, will alert at 150
- **150 lines**: ⚠️ Check if should extract before continuing
- **200 lines**: 🚨 Recommend refactoring now
- **Reading large file**: Offer refactoring plan before changes

---

## 🔧 EXECUTION PHASE (After User Approval)

**IMPORTANT:** This skill can EXECUTE refactoring, not just suggest it!

### Step 1: Present Plan and Get User Approval

After analyzing file size and creating refactoring plan, **always ask for approval:**

```markdown
## Refactoring Plan Ready

**File:** [filename] ([X] lines)
**Target:** Reduce to <[target] lines

### Proposed Changes:
1. Extract [data/component/class] to [new-file] (saves [X] lines)
2. Extract [component/function] to [new-file] (saves [X] lines)
3. Extract [hook/utility] to [new-file] (saves [X] lines)

**Result:** Main file reduced to ~[X] lines
**Execution scope:** [Small/Medium/Large]
**Risk level:** Low (all changes are git-committed incrementally)

### Options:

**✅ Yes** - Execute refactoring now (recommended)
  - I'll split the file step-by-step
  - Each step will be tested and committed
  - You can rollback if needed
  - Then I'll continue with your original request

**❌ No** - Skip refactoring
  - I'll continue with your edit
  - Warning: File will grow larger
  - May become harder to maintain

**⏰ Later** - Save plan for later
  - I'll save the plan to `refactoring-plan-[filename].md`
  - You can execute it manually later
  - I'll continue with your edit now

**📋 Review** - Show detailed step-by-step execution plan
  - I'll explain exactly what will happen
  - Then you can choose Yes/No/Later

**Your choice?**
```

---

### Step 2: Execute Refactoring (Only if User Says "Yes")

**When user approves, execute refactoring incrementally:**

1. **Create backup commit** (safety first)
2. **Verify starting state** (lint, type-check, test must pass)
3. **Execute step-by-step** (extract → verify → commit for each change)
4. **Final verification** (full test suite)
5. **Rollback automatically** if any step fails

**Key execution principles:**
- ✅ Each step is atomic and committed separately
- ✅ Each step is verified before proceeding
- ✅ Automatic rollback on any failure
- ✅ User can cancel at any time

**For detailed execution procedures, safety measures, rollback procedures, and examples, see `REFERENCE.md` → "EXECUTION PHASE - Detailed Procedures"**

---

### Step 3: Continue With Original Request

**After execution (success or skip):**

```
Refactoring: ✅ Complete (or ⏭️ Skipped)
Original request: "[user's request]"

Now proceeding with your request...
```

---

## 🔄 Integration with Other Skills

**Works with:**
- `frontend-ui` - Check file sizes during UI audit and implementation work
- `technical-writing` - Document refactored structure
- `qa-testing` - Ensure tests cover refactored code

**Triggers:**
- Before major edits to large files
- During file creation
- After reading files >200 lines
- When detecting size/complexity patterns

---

## Example Invocations

### Direct skill calls

```text
/code-refactoring refactor src/components/UserProfile.tsx before adding a new settings tab
```

```text
/code-refactoring scan analytics.py and propose a safe split plan before editing
```

```text
/code-refactoring audit src for oversized TSX files and rank the top 5 refactor targets
```

```text
/code-refactoring review this service module for repeated conditionals and extraction points
```

```text
/code-refactoring check whether DashboardService.py is too large before I add export logic
```

```text
/code-refactoring inspect this Rust module and hand off to rust-refactoring if Rust triggers fire
```

### Natural-language requests that should trigger this skill

```text
Before you add the new modal, check whether this React component should be split first.
```

```text
Take a look at this Python module and tell me if it is getting too large to maintain.
```

```text
Find large files in the frontend and give me a refactoring roadmap.
```

```text
I need to add one more workflow to this service, but do a size and complexity check first.
```

```text
Audit the codebase for technical debt caused by oversized files and repeated logic.
```

```text
This class feels bloated. Suggest extraction points before making any feature changes.
```

### Good use-case coverage

- preventive checks before adding new code to an already large file
- proactive file-splitting guidance during component, module, or service creation
- audit mode for finding the worst refactor hotspots in an existing codebase
- pattern-based suggestions when repeated logic, nested branching, or long parameter lists appear
- Rust routing when the request is really about ownership, module splits, typed errors, or async boundaries

---

## 🔍 CODEBASE AUDIT MODE (For Existing Technical Debt)

**Use this mode when user asks to:**
- "Audit the codebase" or "find large files"
- Check for "technical debt" or "existing problems"
- "Scan for refactoring opportunities"
- Review legacy/inherited codebase
- Plan major refactoring initiative

**Audit Workflow (Brief):**

1. **Scan for oversized files** by language/type
2. **Categorize by severity** (Critical >300, High 200-300, Medium 150-200, Good <150)
3. **Prioritize using formula**: (Size/100) + (Change Frequency × 2) + (Business Impact × 3)
4. **Create refactoring roadmap** with priority and execution scope
5. **Execute incrementally** (one file per sprint)

**Quick scan commands:**
```bash
# Find oversized files by type
find src -name "*.tsx" -exec wc -l {} \; | awk '$1 > 200' | sort -rn

# Use helper script
bash scripts/check-size.sh src/ '*.tsx'
```

**For detailed audit procedures, report templates, prioritization matrices, and batch analysis commands, see `REFERENCE.md` → "CODEBASE AUDIT MODE - Detailed Procedures"**

---

## 📚 Language-Specific Details

**For detailed refactoring patterns specific to each language, see:**
- `REFERENCE.md` - Comprehensive language-specific guides
- `FORMS.md` - Refactoring templates and checklists
- Rust mode covers ownership pressure, typed errors, async boundaries, type-driven refactors, and stage/module/function split guidance

**To check file size quickly:**
```bash
# Use the helper script
bash .github/skills/code-refactoring/scripts/check-size.sh [filename]
```

---

## ✅ Success Criteria

**Refactoring is successful when:**

1. ✅ Main file within target size for language
2. ✅ All extracted files well-organized
3. ✅ No broken imports or references
4. ✅ All tests pass
5. ✅ Functionality unchanged
6. ✅ Code more maintainable and readable
7. ✅ Clear separation of concerns
8. ✅ Documentation updated

---

## Remember: Prevention > Cure

**This skill prevents "caught by accident" scenarios by:**
- Checking BEFORE editing (preventive)
- Alerting DURING creation (proactive)
- Suggesting AFTER reading (reactive)
- Enforcing patterns automatically

**The goal:** Never again discover an oversized, complex file by accident.

**Multi-language support:** Works across JavaScript/TypeScript, Python, and general codebases with language-specific patterns.
