# Refactoring Decision Flowchart

Visual guide for when to refactor.

---

## 🔀 Main Decision Flowchart

```mermaid
flowchart TD
    Start([About to edit file]) --> Check{Check file size}

    Check -->|<150 lines| Good[✅ Proceed with edit]
    Check -->|150-200 lines| Warning{Recently changed?}
    Check -->|200-300 lines| Alert{Critical feature?}
    Check -->|>300 lines| Stop[🛑 MUST refactor first]

    Warning -->|Yes| Plan[⚠️ Plan refactoring]
    Warning -->|No| Proceed[✅ Proceed with caution]

    Alert -->|Yes| Plan
    Alert -->|No| Consider[⚠️ Consider refactoring]

    Stop --> Analysis[Run refactoring analysis]
    Plan --> Analysis

    Analysis --> Present[Present refactoring plan]
    Present --> Approval{User approves?}

    Approval -->|Yes| Execute[Execute refactoring]
    Approval -->|No| Skip[Skip refactoring]
    Approval -->|Later| Save[Save plan for later]

    Execute --> Verify{Tests pass?}
    Verify -->|Yes| Success[✅ Refactoring complete]
    Verify -->|No| Rollback[❌ Rollback changes]

    Success --> Original[Continue with original edit]
    Skip --> Original
    Rollback --> Original

    Good --> Original
    Proceed --> Original
    Consider --> Original
```

---

## 📏 File Size Decision Tree

```mermaid
flowchart TD
    FileSize[Check file size] --> Size{Current size?}

    Size -->|<100 lines| Status1[✅ Healthy - Continue]
    Size -->|100-150 lines| Status2[✅ Good - Monitor]
    Size -->|150-200 lines| Status3[⚠️ Warning - Plan extraction]
    Size -->|200-300 lines| Status4[🚨 Alert - Refactor soon]
    Size -->|>300 lines| Status5[🛑 Critical - Refactor now]

    Status3 --> Check3{Complexity?}
    Check3 -->|Low| Monitor3[Monitor, extract at 200]
    Check3 -->|High| Extract3[Extract now]

    Status4 --> Check4{Change frequency?}
    Check4 -->|Low| Opportunistic[Refactor when editing]
    Check4 -->|High| Immediate[Refactor immediately]

    Status5 --> Emergency[Stop all edits, refactor first]
```

---

## 🎯 What to Extract Decision Tree

```mermaid
flowchart TD
    Extract[What to extract?] --> Type{File type?}

    Type -->|React/TypeScript| React[React component]
    Type -->|Python| Python[Python module]
    Type -->|General| General[Any language]

    React --> ReactCheck{What's large?}
    ReactCheck -->|Data arrays| DataFile[Extract to data file]
    ReactCheck -->|Modal/dialog| SubComponent[Extract sub-component]
    ReactCheck -->|4+ hooks| CustomHook[Extract custom hook]
    ReactCheck -->|Complex form| FormComponent[Extract form component]

    Python --> PythonCheck{What's large?}
    PythonCheck -->|Config vars| ConfigFile[Extract to config file]
    PythonCheck -->|Class >300 lines| SplitClass[Split into multiple classes]
    PythonCheck -->|Function >50 lines| SplitFunction[Break into smaller functions]
    PythonCheck -->|Module >400 lines| Package[Convert to package]

    General --> GeneralCheck{Pattern?}
    GeneralCheck -->|Repeated logic| Utility[Extract to utility function]
    GeneralCheck -->|Complex conditional| Strategy[Use strategy pattern/dispatch table]
    GeneralCheck -->|Deep nesting| Flatten[Use early returns/guard clauses]
    GeneralCheck -->|Large data| DataFile
```

---

## 🚨 Priority Decision Matrix

```mermaid
flowchart TD
    Prioritize[Prioritize refactoring] --> Factors[Calculate factors]

    Factors --> Size[Size Factor = Lines / 100]
    Factors --> Freq[Change Freq = Commits/month × 2]
    Factors --> Impact[Business Impact = Rating(1-3) × 3]
    Factors --> Risk[Risk Factor = Complexity(1-3) × 1.5]

    Size --> Score[Total Score = Sum of factors]
    Freq --> Score
    Impact --> Score
    Risk --> Score

    Score --> Level{Score level?}

    Level -->|≥25| P0[P0 - Critical: Fix in 1 week]
    Level -->|15-24| P1[P1 - High: Fix in 1 month]
    Level -->|10-14| P2[P2 - Medium: Fix in quarter]
    Level -->|<10| P3[P3 - Low: Fix when convenient]
```

---

## 🔄 Execution Workflow

```mermaid
flowchart TD
    Start([User approves refactoring]) --> Backup[Create backup commit]

    Backup --> VerifyStart{Tests pass?}
    VerifyStart -->|No| FailStart[❌ Cannot start]
    VerifyStart -->|Yes| Step1[Step 1: Extract data/component]

    Step1 --> Test1{Tests pass?}
    Test1 -->|No| Rollback[Rollback all changes]
    Test1 -->|Yes| Commit1[Commit step 1]

    Commit1 --> Step2[Step 2: Extract next piece]
    Step2 --> Test2{Tests pass?}
    Test2 -->|No| Rollback
    Test2 -->|Yes| Commit2[Commit step 2]

    Commit2 --> StepN[Step N: Final extraction]
    StepN --> TestN{Tests pass?}
    TestN -->|No| Rollback
    TestN -->|Yes| CommitN[Commit step N]

    CommitN --> Final{Full test suite?}
    Final -->|Pass| Success[✅ Refactoring complete]
    Final -->|Fail| Rollback

    Rollback --> Report[Report failure to user]
    FailStart --> Report

    Success --> Continue[Continue with original request]
    Report --> Options[User chooses: Retry/Skip/Manual]
```

---

## 📊 Language-Specific Triggers

### JavaScript/TypeScript/React

```mermaid
flowchart LR
    Trigger[Auto-invoke when:] --> T1[File reaches 150 lines]
    Trigger --> T2[4+ hooks detected]
    Trigger --> T3[Large data array >20 lines]
    Trigger --> T4[Modal/dialog code found]
    Trigger --> T5[Complex form >10 fields]

    T1 --> Action[Invoke refactoring skill]
    T2 --> Action
    T3 --> Action
    T4 --> Action
    T5 --> Action
```

### Python

```mermaid
flowchart LR
    Trigger[Auto-invoke when:] --> T1[Class reaches 250 lines]
    Trigger --> T2[Function reaches 40 lines]
    Trigger --> T3[Module reaches 350 lines]
    Trigger --> T4[10+ methods in class]
    Trigger --> T5[Complex __init__ >20 lines]

    T1 --> Action[Invoke refactoring skill]
    T2 --> Action
    T3 --> Action
    T4 --> Action
    T5 --> Action
```

---

## 💡 Quick Reference: When NOT to Refactor

```mermaid
flowchart TD
    Question([Should I refactor?]) --> Check{Conditions?}

    Check -->|Code works perfectly| Maybe1{Is it changing?}
    Check -->|No tests exist| No1[❌ Write tests first]
    Check -->|Production incident| No2[❌ Fix incident first]
    Check -->|Tight deadline| No3[❌ Defer to later]
    Check -->|Learning new codebase| No4[❌ Understand first]

    Maybe1 -->|Rarely| No5[❌ Leave it alone]
    Maybe1 -->|Frequently| Yes1[✅ Refactor before next change]

    No1 --> Defer[Defer refactoring]
    No2 --> Defer
    No3 --> Defer
    No4 --> Defer
    No5 --> Defer
```

---

## 🎯 Success Criteria Checklist

```mermaid
flowchart TD
    Done([Refactoring finished]) --> Q1{Main file <200 lines?}

    Q1 -->|Yes| Q2{All tests pass?}
    Q1 -->|No| Fail[❌ Not complete]

    Q2 -->|Yes| Q3{No performance regression?}
    Q2 -->|No| Fail

    Q3 -->|Yes| Q4{Documentation updated?}
    Q3 -->|No| Fail

    Q4 -->|Yes| Q5{Code review approved?}
    Q4 -->|No| Fail

    Q5 -->|Yes| Success[✅ Refactoring successful!]
    Q5 -->|No| Fail
```

---

## 📖 How to Read These Diagrams

**Shapes:**
- `([Start/End])` - Start or end point
- `[Action]` - Action to take
- `{Decision?}` - Decision point
- `[[Process]]` - Sub-process

**Colors (in Mermaid renderers):**
- Green - Success/good state
- Yellow - Warning/caution
- Red - Error/stop
- Blue - Process/action

**Arrows:**
- `-->` - Flow direction
- `|Label|` - Condition or note

---

## 🔗 Usage

**Copy these diagrams into:**
- GitHub/GitLab wikis (supports Mermaid)
- Documentation sites (most support Mermaid)
- Presentations (render as images)
- README files (GitHub supports Mermaid)

**Online editors:**
- https://mermaid.live/ - Test and export diagrams
- https://www.mermaidchart.com/ - Create and share

---

**Note:** These flowcharts complement the detailed instructions in SKILL.md and REFERENCE.md. Use them for quick decision-making at a glance.
