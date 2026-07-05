# Code Refactoring Reference Guide

Detailed patterns, workflows, and checklists for refactoring across JavaScript,
TypeScript, React, Python, Rust, and language-agnostic code.

## Table of Contents

1. JavaScript, TypeScript, and React Patterns
2. Python Patterns
3. Rust Patterns
4. General Patterns
5. Performance Considerations
6. Common Anti-Patterns
7. File Naming Conventions
8. Additional Resources
9. Execution Phase Procedures
10. Codebase Audit Mode
11. When Not to Refactor
12. Decision Matrix and Alternatives
13. TDD Integration

## JavaScript, TypeScript, and React Patterns

### Data Extraction

Use this when static arrays, config objects, or long option lists dominate the
component and make UI logic hard to scan.

```typescript
// Before: large data block inside the component
export function UserProfile() {
  const badges = [
    { id: 1, name: 'Expert', icon: 'trophy' },
    { id: 2, name: 'Leader', icon: 'crown' },
    { id: 3, name: 'Mentor', icon: 'star' },
  ];

  return <BadgeList badges={badges} />;
}
```

```typescript
// After: data moved to a dedicated module
export const userBadges = [
  { id: 1, name: 'Expert', icon: 'trophy' },
  { id: 2, name: 'Leader', icon: 'crown' },
  { id: 3, name: 'Mentor', icon: 'star' },
];
```

```typescript
import { userBadges } from './user-profile.data';

export function UserProfile() {
  return <BadgeList badges={userBadges} />;
}
```

Prefer a `.data.ts` or `.config.ts` file when the extracted content has no JSX.
Use `.tsx` only when the data structure itself embeds JSX.

### Sub-Component Extraction

Use this when a component contains visually distinct regions, modal markup,
large table rows, or conditional sections with their own state and handlers.

```tsx
// Before
export function Dashboard() {
  return (
    <div>
      <Header />
      <section>{/* 100 lines of chart rendering */}</section>
      <aside>{/* 80 lines of filter UI */}</aside>
      <footer>{/* 40 lines of summary actions */}</footer>
    </div>
  );
}
```

```tsx
// After
import { DashboardChart } from './DashboardChart';
import { DashboardFilters } from './DashboardFilters';
import { DashboardSummary } from './DashboardSummary';

export function Dashboard() {
  return (
    <div>
      <Header />
      <DashboardChart />
      <DashboardFilters />
      <DashboardSummary />
    </div>
  );
}
```

Extract a child component only if the child gains a clear responsibility. Do
not extract trivial wrappers that only forward props without simplifying logic.

### Custom Hook Extraction

Use this when the component manages multiple related `useState` fields, async
loading, derived state, or imperative event coordination.

```tsx
// Before
export function UserForm() {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSaving, setIsSaving] = useState(false);

  async function submit() {
    // validation, request, toast, reset
  }

  return <form>{/* form fields */}</form>;
}
```

```tsx
// After
function useUserForm() {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSaving, setIsSaving] = useState(false);

  async function submit() {
    // validation, request, toast, reset
  }

  return {
    name,
    email,
    errors,
    isSaving,
    setName,
    setEmail,
    submit,
  };
}
```

Keep the hook local when it serves exactly one component. Promote it to a
shared hook only after a second real consumer appears.

### Service Extraction

Use this when API calls, response shaping, retries, or storage access are mixed
into UI files.

```typescript
// Before
export async function loadOrders() {
  const response = await fetch('/api/orders');
  const body = await response.json();
  return body.items.map(normalizeOrder);
}
```

```typescript
// After
import { orderApi } from './order-api';

export async function loadOrders() {
  const items = await orderApi.fetchOrders();
  return items.map(normalizeOrder);
}
```

Service extraction should move side effects and transport details, not merely
rename a fetch call.

### Reducer Promotion

Use `useReducer` or a state machine when several state variables change
 together and event handling starts looking like implicit transitions.

```tsx
type State = {
  step: 'idle' | 'editing' | 'saving' | 'done' | 'error';
  draft: DraftProfile;
  error: string | null;
};

type Action =
  | { type: 'edit'; draft: DraftProfile }
  | { type: 'save' }
  | { type: 'saved' }
  | { type: 'failed'; message: string };
```

Reducers help most when they encode transition rules that were otherwise spread
across click handlers and effects.

### Frontend Refactoring Heuristics

- Split by responsibility before splitting by file count.
- Keep data shaping close to the data boundary.
- Prefer explicit prop names over broad `config` objects.
- Move heavy computations out of render paths.
- Resist creating a shared abstraction after a single use.

## Python Patterns

### Function Extraction

Use this when a single function performs parsing, validation, transformation,
and I/O in one pass.

```python
def import_users(csv_path: str) -> list[User]:
    rows = load_rows(csv_path)
    valid_rows = validate_rows(rows)
    records = normalize_rows(valid_rows)
    return persist_users(records)
```

Extract steps in the order data flows through the system. That keeps function
names concrete and makes targeted tests easier to write.

### Class Decomposition

Use this when one class mixes transport, orchestration, domain rules, and
serialization.

```python
class ReportService:
    def build_report(self, request: ReportRequest) -> Report:
        data = self._fetch_data(request)
        validated = self._validate_data(data)
        summary = self._summarize(validated)
        return self._render(summary)
```

If the helper methods can evolve independently, they probably deserve separate
collaborators rather than staying as private methods forever.

### Module Split

Split a Python file by role when it accumulates unrelated constants, Pydantic
models, validators, adapters, and orchestration functions.

Preferred layout:

```text
billing/
  __init__.py
  models.py
  validators.py
  service.py
  repository.py
  errors.py
```

Keep import direction simple. Models should not import repositories, and low
level adapters should not depend on API handlers.

### Boundary Extraction

Separate side effects from pure logic.

```python
def calculate_invoice_total(lines: list[InvoiceLine]) -> Decimal:
    return sum(line.quantity * line.price for line in lines)


def store_invoice(repository: InvoiceRepository, invoice: Invoice) -> None:
    repository.save(invoice)
```

This makes business logic unit-testable without mocks and narrows the surface
that needs integration tests.

### Error Surface Cleanup

Use typed exceptions that match domain failures instead of raising broad
`ValueError` or raw library exceptions across layers.

```python
class InvalidPayoutWindowError(Exception):
    pass


def validate_payout_window(start: date, end: date) -> None:
    if end < start:
        raise InvalidPayoutWindowError('end date must be after start date')
```

### Python Refactoring Heuristics

- Prefer dataclasses or Pydantic models for structured data.
- Keep modules focused around one domain concept.
- Avoid circular imports by pushing shared types downward.
- Replace long parameter lists with explicit objects when parameters travel
  together.
- Add characterization tests before touching legacy control flow.

## Rust Patterns

Rust-specific refactoring guidance now lives in the dedicated
`rust-refactoring` skill package.

Use it when the change centers on:

- `.rs` files or `Cargo.toml`
- ownership, borrowing, lifetimes, or clone pressure
- typed errors with `thiserror`
- async `Send` or lock-boundary cleanup
- module or stage splits in a Rust crate

For the detailed Rust playbook, use:

- `../rust-refactoring/SKILL.md`
- `../rust-refactoring/REFERENCE.md`
- `../rust-refactoring/FORMS.md`

## General Patterns

### Split by Axis of Change

Split code where reasons to change differ.

If pricing rules change with policy updates, persistence changes with storage
migrations, and rendering changes with UX work, keep those concerns apart.

### Extract Adapters at Boundaries

When refactoring legacy or vendor-driven systems, prefer adapters over invasive
rewrites.

```text
legacy input -> adapter -> clean domain model -> application logic
```

This lets new code stabilize while keeping risky legacy changes local.

### Prefer Explicit Data Flow

Reduce hidden coupling by passing the exact values a function needs instead of a
large context object.

### Replace Boolean Control Flags

If a function takes flags like `is_preview`, `should_retry`, or `with_cache`, it
often signals conflated behaviors. Consider separate entry points or a small
options type with clear semantics.

### Consolidate Duplicate Decision Logic

If the same branching logic appears in several files, extract the decision once
and let callers consume the result.

## Performance Considerations

Refactoring should improve structure first, but it must not quietly introduce
runtime regressions.

### Watch Allocation Pressure

- Avoid cloning collections just to satisfy signatures.
- Prefer iterators or borrowed views for read-only paths.
- Re-check hot loops after extraction.

### Preserve Batch Semantics

When splitting functions, keep batching behavior explicit so the code does not
accidentally become N+1 I/O.

### Measure Before Claiming Improvement

Use profiling or benchmarks for performance-sensitive paths. A cleaner design
can still be slower if it adds repeated parsing, allocation, or dispatch.

## Common Anti-Patterns

### Cosmetic Refactoring

Do not churn names, files, or layers when there is no clarity or maintenance
gain.

### Abstraction Inflation

Avoid creating a helper, trait, interface, or base class after only one call
site unless a clear seam already exists.

### Parallel Structural and Behavioral Changes

Do not mix a large refactor with a feature change unless the user explicitly
asked for both and the review surface remains manageable.

### Premature Shared Utilities

Duplicated code in two places is often still cheaper than a wrong shared
abstraction that many modules must depend on.

### Refactoring Without Verification

If you cannot prove behavior stability through tests, builds, or strong manual
checks, reduce scope or stop.

## File Naming Conventions

Use file names that reveal the extracted responsibility.

- React data modules: `feature-name.data.ts`
- React hooks: `use-feature-name.ts`
- Python modules: `service.py`, `models.py`, `validators.py`
- Rust modules: `types.rs`, `errors.rs`, `verify.rs`, `builder.rs`

Do not name files after vague actions such as `helpers`, `utils2`, or
`miscellaneous` unless the repository already enforces such a pattern.

## Additional Resources

- For templates and checklists, see `FORMS.md`.
- For Rust-only workflows, see `../rust-refactoring/REFERENCE.md`.
- For quick file-size checks, see `scripts/check-size.sh` when present.

## Execution Phase Procedures

This section describes the detailed execution workflow after the high-level
decision to refactor has already been made.

### Phase 1: Preparation

Start only from a known-good state.

```bash
git add [target-files]
git commit -m "backup: before refactoring"

npm run lint
npm run type-check
npm run test
```

If any baseline check fails, stop. Refactoring on top of a broken baseline makes
rollback and diagnosis much harder.

### Phase 2: Incremental Refactoring

Use a tight loop:

1. Extract one seam.
2. Update imports or references.
3. Run validation.
4. Commit the atomic step.

Example execution log:

```text
Step 1/3: Extract DashboardChart
- Created DashboardChart.tsx
- Updated Dashboard.tsx imports
- Lint passes
- Tests pass
- Commit created
```

Repeat until the main file is below the target size or the structural problem is
resolved.

### Phase 3: Final Verification

Run the full validation set that matches the repository:

```bash
npm run lint
npm run type-check
npm run test
npm run build
```

For Python or Rust repositories, substitute the native toolchain commands rather
than forcing JavaScript tooling.

### Rollback Procedure

If a refactor step fails and cannot be repaired quickly, roll back to the last
known-good refactor commit or the pre-refactor backup commit.

```bash
git log --oneline
git reset --hard HEAD~[count]
```

Use rollback sparingly and only when explicitly allowed by the repository or the
user. In many repositories, reverting with a new commit is safer than resetting
history.

### Safety Checklist

Before execution:

- User approval is explicit when the refactor is optional.
- The baseline build is green.
- A rollback path exists.
- The working tree is understood.

During execution:

- One logical change per step.
- Validation after each structural move.
- Immediate stop on unexplained failures.

After execution:

- Full validation has passed.
- Documentation and imports are current.
- Any user-facing API drift is called out.

## Codebase Audit Mode

Use audit mode when the user asks for a broad scan of oversized or high-risk
files rather than an immediate edit.

### When to Use Audit Mode

Use it for requests such as:

- Audit the codebase.
- Find large files.
- Identify refactoring opportunities.
- Build a technical debt roadmap.

### Audit Workflow

#### Step 1: Scan for Oversized Files

```bash
find src -name "*.tsx" -o -name "*.ts" | xargs wc -l | sort -rn | head -20
find src -name "*.py" | xargs wc -l | sort -rn | head -20
find crates -name "*.rs" | xargs wc -l | sort -rn | head -20
```

#### Step 2: Categorize by Severity

- Critical: over 300 lines.
- High: 200 to 300 lines.
- Medium: 150 to 200 lines.
- Healthy: under 150 lines.

Thresholds are heuristics, not laws. A dense parser or generated bindings may be
acceptable at a larger size if the responsibility is still clear.

#### Step 3: Prioritize

Score files by three factors:

- Size.
- Change frequency.
- Business impact.

Example matrix:

| File | Size | Change Frequency | Business Impact | Priority |
| --- | --- | --- | --- | --- |
| dashboard.tsx | 450 | Weekly | Critical | P0 |
| report.py | 320 | Monthly | Medium | P1 |
| tx_verify.rs | 280 | Rare | High | P1 |

#### Step 4: Produce a Roadmap

Structure the roadmap by phases or sprints instead of listing every file as an
undifferentiated backlog.

```text
Phase 1: P0 files used in weekly feature work
Phase 2: P1 files with repeated review churn
Phase 3: P2 files touched opportunistically
```

#### Step 5: Execute Gradually

Do not start broad codebase surgery all at once. Pick one file or one module per
phase and keep the review surface tractable.

### Audit Report Template

Use this shape for audit summaries:

```markdown
## Code Refactoring Audit Report

### Summary Statistics

| Metric | Count |
| --- | --- |
| Critical files | 2 |
| High-priority files | 5 |
| Medium-priority files | 7 |
| Healthy files | 41 |

### Top Findings

1. `dashboard.tsx` mixes layout, data fetching, and chart transforms.
2. `billing.py` combines API parsing with domain calculation.
3. `tx_verify.rs` contains three verification stages in one function.

### Recommended Roadmap

1. Refactor P0 files first.
2. Add missing tests around critical behavior.
3. Re-run the audit after each phase.
```

## When Not to Refactor

Knowing when to skip refactoring is as important as knowing how to perform it.

### No Test Coverage

If the code has little or no test coverage, do not start with structural edits.
Write characterization tests first.

### Frozen or Maintenance-Only Code

If the code will not change again, the business return may be too low to justify
the risk.

### Production Incident Window

During an outage or severe incident, fix the incident first. Structural cleanup
belongs in the follow-up phase.

### Tight Deadline or Active Concurrency

If several developers are landing changes in the same area under deadline
pressure, defer unless the refactor is the smallest safe path to delivery.

### Over-Sized Refactor Plan

If the planned refactor takes more than a day or two for one person, it is
usually too large. Break it down or postpone.

### Missing Domain Context

Ugly code sometimes encodes real business or compliance rules. If intent is not
clear, investigate before cleaning it up.

### Third-Party or Generated Code

Do not refactor code that will be overwritten by generators or upstream vendor
updates. Wrap it instead.

## Decision Matrix and Alternatives

### Quick Decision Matrix

| Factor | Refactor Now | Defer | Skip |
| --- | --- | --- | --- |
| Test coverage | Strong | Partial | Minimal |
| Code activity | Frequent | Moderate | Rare |
| Business value | Clear | Potential | None |
| Deadline pressure | Low | Medium | High |
| Domain understanding | Strong | Partial | Weak |

If most signals fall into the left column, proceed. If the right column wins,
skip or re-scope.

### Alternatives to Refactoring

- Use a facade around messy code.
- Add documentation instead of structural churn.
- Introduce feature flags for risk isolation.
- Create a technical debt ticket and schedule it.
- Use a strangler approach for larger legacy systems.

### Pragmatic Rule

Refactoring is not a goal by itself. The goal is to deliver safer change with
less pain. If a refactor does not improve delivery, clarity, or defect rate,
skip it.

## TDD Integration

TDD provides the safest operational rhythm for refactoring.

### Red, Green, Refactor

```text
RED -> write a failing test
GREEN -> make the test pass with minimal code
REFACTOR -> improve structure while tests stay green
```

### Where This Skill Fits

- During red: stay focused on the failing test.
- During green: get the behavior working first.
- During refactor: use this skill to split files, extract helpers, and clean up
  design while the tests protect behavior.

### Characterization Tests for Legacy Code

If the code was not built with TDD, add characterization tests that capture the
current behavior before changing structure.

```javascript
it('returns zero for negative prices in the current implementation', () => {
  expect(calculateDiscount(-100)).toBe(0);
});
```

The test may describe awkward behavior. That is fine. The point is to create a
baseline before deciding whether behavior itself should later change.

### TDD Benefits During Refactoring

- Immediate regression detection.
- Safer extraction of helpers and modules.
- Better public API design.
- Executable documentation for future contributors.

Keep the test suite green throughout the refactor loop. If the suite goes red
for unclear reasons, shrink the step size or stop and diagnose before going on.
