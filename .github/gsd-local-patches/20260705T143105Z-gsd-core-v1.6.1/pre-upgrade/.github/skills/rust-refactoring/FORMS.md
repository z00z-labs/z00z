# Rust Refactoring Forms

Reusable checklists and templates for Rust refactoring work.

## Quick Intake Checklist

- Target file or module is Rust-specific.
- Baseline `cargo fmt`, `cargo clippy`, and `cargo test` status is known.
- Public API stability requirements are known.
- Ownership, error, or module seams have been identified.
- Guard tests exist or will be added first.

## Refactor Plan Template

```markdown
## Rust Refactor Plan

### Target
- File or module:
- Crate:
- Current problem:

### Goal
- Intended structural improvement:
- Behavior that must remain unchanged:

### Seams
- Ownership seam:
- Error seam:
- Module seam:
- Async or blocking seam:

### Steps
1. Add or confirm guard tests.
2. Extract first helper or stage.
3. Move typed errors or traits if needed.
4. Split modules or files.
5. Re-run Cargo validation.

### Validation
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
```

## Cargo Validation Ladder

Use the narrowest safe command first, then widen when the change crosses crate
or workspace boundaries.

```bash
cargo test -p crate_name
cargo test -p crate_name module_name::tests::case_name
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Review Checklist

- Ownership is clearer than before.
- New clones are justified.
- Error variants remain precise.
- Imports stay grouped and minimal.
- Visibility is no wider than required.
- Module names reflect responsibilities.
- Targeted tests and wider Cargo checks passed.

## Quality Checklist

- Reduced unnecessary cloning and memory allocations.
- Improved error handling with a consistent error surface.
- Enhanced type safety through enums, newtypes, or stronger contracts.
- Simplified lifetime management where possible.
- Addressed relevant Clippy findings rather than suppressing them blindly.
- Maintained or improved performance-sensitive paths.
- Re-checked safety invariants for unsafe or FFI-adjacent code.
- Updated docs, examples, or comments that describe the old structure.
- Added tests for edge cases discovered during the refactor.
- Left the code more idiomatic than before.

## Modernization Intake

Use these prompts when the refactor also includes targeted modernization:

- Edition migration needed:
- Runtime migration involved:
- Macro-heavy code involved:
- Coverage gap blocking refactor:
- Unsafe or FFI review required:
- Benchmark before and after required:

## Rollback Triggers

Stop and reassess when:

- The refactor starts changing behavior instead of structure.
- Borrow-checker fixes require broad semantic rewrites.
- Public APIs drift unexpectedly.
- Validation failures stop being local and understandable.
- The change grows into a crate redesign instead of a bounded refactor.
