---
name: code-commenter
description: Produce minimal, high-signal Rust comments and complete Rustdoc for public APIs, with strict focus on why, contracts, safety, and executable examples.
argument-hint: '[path or symbol]'
---

# Code Commenter

## When to Use

- The user wants Rust comments or Rustdoc focused on intent, contracts, safety, or examples.
- Public APIs need documentation without turning the codebase into comment spam.
- Unsafe, async, or contract-heavy Rust code needs high-signal documentation review.
- The request is about documentation quality, not behavioral refactoring.

## Mission

Create the smallest amount of Rust documentation that still makes the code easier to maintain.

Default to self-documenting code first. Add comments only when they explain intent, invariants, safety, lifecycle constraints, or API contracts that the code alone does not make obvious.

**INPUT:** `<<INPUT_TEXT>> AND/OR <<ATTACHED_FILES>>`

## Core Principle

**Write code that speaks for itself. Document WHY, contracts, and safety. Avoid narrating WHAT.**

Rust already carries meaning through types, traits, enums, ownership, and errors. Good documentation in Rust should amplify that signal, not duplicate it.

## Scope

Use this skill for:

- inline comments above non-obvious code
- Rustdoc for public functions, structs, enums, traits, modules, and macros
- unsafe code documentation
- async API documentation
- documentation cleanup when comments are stale, noisy, or incomplete

## Operating Modes

### 1. Comment Pass

Use regular comments only when they add intent that is hard to infer from code.

- Prefer a single short comment above the item or block.
- Keep comments one or two short lines unless `SAFETY:` requires more precision.
- Explain motivation, tradeoffs, ordering, invariants, or maintenance traps.
- Prefer deleting a weak comment over rewriting it into fluff.

### 2. Rustdoc Pass

Use Rustdoc for public APIs and documented module boundaries.

- Use `///` for public items.
- Use `//!` for module- and crate-level documentation.
- Include `# Examples` when examples improve usage clarity.
- Include `# Errors`, `# Panics`, and `# Safety` when applicable.
- Keep examples executable whenever possible.

## Decision Framework

Before writing any documentation, ask these in order:

1. Can better naming, stronger types, or smaller functions remove the need for a comment?
2. Does Rust already express the behavior through the type system or API shape?
3. Is the missing information about intent, contract, safety, async behavior, or domain rules?
4. Will this help the next maintainer make a safer change?

If the answer to all four is no, do not add documentation.

## What To Document

### Public APIs

For public items, prefer concise Rustdoc with these sections when relevant:

- one-line summary of purpose
- key arguments only when names are not enough
- return value semantics when not obvious
- `# Errors` for meaningful failure modes
- `# Panics` only when panic is possible or constrained by invariants
- `# Examples` with compiling snippets

### Traits

Document trait contracts, implementation obligations, and caller expectations.

- define semantic guarantees, not just method names
- call out idempotency, determinism, ownership, cancellation, or ordering requirements
- explain what implementors must preserve

### Unsafe Code

Unsafe code must always document safety invariants.

- use `// SAFETY:` directly above unsafe blocks
- use `# Safety` in Rustdoc for unsafe functions, traits, or contracts
- state what the caller must guarantee
- state what this code has already validated

### Async Code

Document the async behavior that readers cannot infer from `.await` alone.

- blocking vs non-blocking behavior
- timeout expectations
- cancellation safety
- retry semantics
- executor assumptions such as `spawn_blocking`

### Macros, Enums, And Modules

- macros: explain expansion intent, required inputs, and panic/safety caveats
- enums: document non-obvious variants and state semantics
- modules: use `//!` to explain boundaries, purpose, and usage patterns

## Commenting Best Practices

### Good Inline Comment Targets

- unusual ownership or borrowing choices
- interior mutability with a clear reason
- ordering that protects correctness
- performance tradeoffs backed by workload assumptions
- FFI boundaries and preconditions
- domain-specific invariants

### Good Examples

```rust
// Keep this ordering so signature verification sees the original claimed payload.
fn verify_claim(...) {
    ...
}
```

```rust
// Use BTreeMap to preserve deterministic iteration for audit output and tests.
let users = BTreeMap::new();
```

```rust
// SAFETY: The mutex lock guarantees exclusive access and the buffer length was checked above.
unsafe {
    ptr::copy_nonoverlapping(src, dest, len);
}
```

```rust
// Use spawn_blocking because this derivation is CPU-bound and would stall the async executor.
let result = tokio::task::spawn_blocking(move || derive(data)).await?;
```

### Bad Inline Comment Examples

```rust
// Increment the counter.
counter += 1;
```

```rust
// Create a new User instance.
let user = User::new();
```

```rust
// Return the user's name.
fn get_name(user: &User) -> &str {
    &user.name
}
```

## Rustdoc Best Practices

### Public Function Template

```rust
/// Calculates compound interest using the standard formula.
///
/// # Arguments
/// * `principal` - Initial amount invested; must be positive.
/// * `rate` - Annual interest rate as a decimal such as `0.05`.
/// * `time` - Time period in years.
/// * `compound_frequency` - Number of compounding periods per year.
///
/// # Errors
/// Returns `InterestError::NegativePrincipal` when `principal` is negative.
///
/// # Examples
/// ```
/// let amount = calculate_compound_interest(1000.0, 0.05, 1.0, Some(1))?;
/// assert_eq!(amount, 1050.0);
/// # Ok::<(), InterestError>(())
/// ```
pub fn calculate_compound_interest(
    principal: f64,
    rate: f64,
    time: f64,
    compound_frequency: Option<u32>,
) -> Result<f64, InterestError> {
    // Implementation...
}
```

### Module Template

```rust
//! Payment processing primitives.
//!
//! This module provides idempotent payment execution with gateway-specific adapters.
//!
//! ## Usage
//!
//! ```
//! use payment_processor::payments;
//!
//! # async fn demo() -> Result<(), payment_processor::PaymentError> {
//! let result = payments::process_payment("idemp-123", payment).await?;
//! # Ok(())
//! # }
//! ```
```

### Trait Contract Template

```rust
/// Payment processor interface.
///
/// Implementors must preserve idempotency for the same idempotency key and
/// return errors with enough context for retry decisions.
pub trait PaymentProcessor {
    /// Execute a payment transaction.
    ///
    /// # Errors
    /// Returns `PaymentError` when validation, gateway communication, or
    /// downstream settlement fails.
    async fn execute(&self, payment: Payment) -> Result<PaymentResult, PaymentError>;
}
```

### Unsafe Function Template

```rust
/// Rebuilds a string from raw parts.
///
/// # Safety
/// The caller must ensure that `ptr`, `length`, and `capacity` come from a
/// previous `String::into_raw_parts` call for the same allocation.
unsafe fn from_raw_parts(ptr: *mut u8, length: usize, capacity: usize) -> String {
    // Implementation...
}
```

## Anti-Patterns To Avoid

### Weak Comments

- comments that restate the function name
- comments that narrate obvious control flow
- comments that duplicate type information
- comments that describe old behavior after refactoring

### Weak Rustdoc

- public APIs with no examples when usage is non-obvious
- doctests that do not compile
- missing `# Safety` sections for unsafe surfaces
- async docs that ignore cancellation or blocking behavior
- trait docs that never define semantic guarantees

### Bad Rustdoc Examples

```rust
/// Processes a payment.
pub async fn process_payment(...) -> Result<PaymentResult, PaymentError> {
    ...
}
```

```rust
/// Splits a string by whitespace.
///
/// # Examples
/// ```
/// let words = split_whitespace("hello world")
/// assert_eq!(words, vec!["hello", "world"]);
/// ```
pub fn split_whitespace(input: &str) -> Vec<&str> {
    input.split_whitespace().collect()
}
```

## Output Rules

When using this skill:

- prefer fewer, better comments
- convert broad explanatory comments into precise Rustdoc when the item is public
- use hidden setup lines in doctests with `#` when needed
- keep prose concrete and specific to the codebase
- write in English only
- remove stale or redundant comments instead of preserving them

## Validation Checklist

- [ ] Public items have complete Rustdoc where needed.
- [ ] Inline comments explain intent, not mechanics.
- [ ] Unsafe blocks or APIs include explicit safety reasoning.
- [ ] Async APIs document blocking, retry, timeout, or cancellation behavior when relevant.
- [ ] Examples compile and run, or are clearly marked when they are illustrative only.
- [ ] Documentation matches the current implementation.
- [ ] `cargo doc --no-deps` succeeds for the touched crate when practical.
- [ ] `cargo test --doc` passes for the touched crate when doctests were added or changed.

## Example Invocation

- `/code-commenter crates/z00z_core/src/assets/registry.rs`
- `/code-commenter crates/z00z_core/src/assets/registry.rs::verify_claim`
- `/code-commenter crates/z00z_core/src/assets/mod.rs`

## Final Rule

In Rust, the compiler is your first documentation layer. Use this skill to add only the documentation that the compiler, names, and types cannot already say for you.