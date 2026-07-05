---
name: rust-refactoring
description: Rust-specialized refactoring delegate for `.rs` files, `Cargo.toml`, and Cargo-driven workflows. Use after code-refactoring routes clearly Rust-only work here, or when the request is unambiguously Rust-first and needs Cargo-verified structural cleanup.
---

# Rust Refactoring

Use this skill for Rust-first structural changes. It is intentionally narrower
than the general `code-refactoring` skill and assumes the main validation loop
is Cargo-based.

## When to Use

- The request is clearly Rust-first and mostly touches `.rs` files or `Cargo.toml`.
- `code-refactoring` has already classified the work as Rust-only.
- The task involves module splits, ownership cleanup, error typing, async boundaries, or Cargo-verified structural refactors.
- The user wants Rust refactoring discipline rather than general mixed-language cleanup.

## Entry-Point Relationship

`rust-refactoring` is the specialized Rust route, not the default general
refactoring entry.

Use it when one of these is true:

- `code-refactoring` already classified the request as Rust-only.
- The user explicitly asks for Rust/Cargo/module/ownership refactoring.
- Nearly all touched files are `.rs` files or `Cargo.toml`.

If the task becomes mixed-language or starts as a general cleanup request,
route back through `code-refactoring` so one main entry point remains obvious.

## Auto-Invoke Scope

Prefer this skill when all or most of the following are true:

- The target files are `.rs` files or `Cargo.toml`.
- The user asks to split a module, file, crate, stage, or verification path.
- Ownership, borrowing, lifetimes, clone pressure, error typing, or async
  boundaries are central to the refactor.
- Validation should run through `cargo fmt`, `cargo clippy`, and `cargo test`.

## Do Not Use This Skill

Do not use it for:

- Pure JavaScript, TypeScript, React, or Python refactors.
- Feature work that only happens to touch a small Rust file.
- Generated Rust code or vendored code that should not be edited.

## Default Workflow

1. Establish the baseline.
2. Identify the ownership and module seams.
3. Add or confirm guard tests.
4. Refactor in small Cargo-verifiable steps.
5. Re-check API surface, error behavior, and performance-sensitive paths.

## Baseline Commands

Run the narrowest command set that still proves safety.

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

Use targeted commands when the workspace is large:

```bash
cargo test -p crate_name
cargo test -p crate_name module_name::tests::case_name
```

## Typical Refactor Targets

- Split parse, validate, transform, and persist stages.
- Extract typed error enums with `thiserror`.
- Move protocol or storage behavior behind small traits.
- Replace ad hoc clone chains with clearer ownership boundaries.
- Break oversized modules into `types`, `errors`, `builder`, `verify`, or
  `service` submodules.

## Output Expectations

A completed Rust refactor should leave behind:

- Smaller stage-oriented functions.
- Clearer ownership semantics.
- Focused modules with grouped imports.
- Cargo validation evidence.
- Notes about any public API or test adjustments.

## Reference Files

- Detailed patterns: `REFERENCE.md`
- Templates and checklists: `FORMS.md`

If the task becomes language-mixed, fall back to the broader
`code-refactoring` skill.
