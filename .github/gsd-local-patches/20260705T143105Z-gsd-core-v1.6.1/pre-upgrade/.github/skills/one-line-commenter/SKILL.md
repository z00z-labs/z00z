---
name: one-line-commenter
description: 'Write concise one-line comments for self-explanatory functions only when the comment explains why, not what. Use this skill when you want short, practical comments that stay on one line and add intent instead of repeating the code.'
argument-hint: '[path]'
---

# One-Line Commenter

## When to Use

- The user wants terse one-line comments rather than full Rustdoc or verbose inline commentary.
- A function, struct, or small block is already readable but still needs a brief explanation of intent.
- The goal is to explain why something exists or must stay ordered without repeating what the code already says.
- The request is specifically about short practical comments, not large documentation rewrites.

## Mission

Create short, high-signal comments for functions and small code blocks that are already easy to read.

This skill exists to keep comments focused on intent. A good comment explains **why** the code exists, not **what** the code does.

## Core Rules

- Comment only when the code is already self-explanatory.
- Keep every comment to one line.
- Comment only above a function or struct declaration, or above a small block that needs intent.
- Do not add comments inside function bodies or inside struct definitions.
- Explain motivation, tradeoffs, invariants, or safety concerns.
- Do not restate the code.
- Do not narrate obvious control flow.
- Prefer removing a comment over writing a weak comment.
- Use English only.

## When to Add a Comment

Add a one-line comment when the code needs:

- a design reason
- a security reason
- a domain rule
- a performance tradeoff
- a maintenance warning
- a non-obvious constraint
- a short explanation of what the function or struct is for
- a short explanation of why the function or struct exists

## When Not to Add a Comment

Do not add a comment when it would only repeat:

- the function name
- the variable name
- the obvious behavior of a simple statement
- a standard library call that is already clear

## Comment Style

Write comments in the form of a short sentence that answers one of these questions:

- Why is this needed?
- Why is this order important?
- Why is this check here?
- Why is this implementation chosen?
- Why must this not change?
- What is this function or struct for?
- Why does this type exist?

Keep the comment directly above the function, statement, or block it explains.

## Good Examples

```rust
// Keep the original ordering so signature verification sees the exact claimed payload.
fn verify_claim(...) {
    ...
}
```

```rust
// Retry because the upstream index is eventually consistent after block finalization.
if retry_needed {
    ...
}
```

```rust
// Use a stable seed here so test fixtures produce identical results across runs.
let seed = 42;
```

```rust
// Reject early to avoid leaking timing differences on invalid secrets.
if secret.is_empty() {
    ...
}
```

```rust
// Cache this result because the same derivation is reused by multiple callers.
let derived = derive_key(...);
```

```rust
// Represents the minimal state needed to rebuild a wallet from persisted data.
struct WalletSnapshot {
    ...
}
```

## Bad Examples

```rust
// Check if the value is empty.
if value.is_empty() {
    ...
}
```

```rust
// Increment the counter.
counter += 1;
```

```rust
// Call the validation function.
validate(input)?;
```

```rust
// This function processes the request.
fn process_request(...) {
    ...
}
```

```rust
struct WalletSnapshot {
    // Stores the seed.
    seed: [u8; 32],
}
```

## Output Requirements

When using this skill, produce comments that are:

- one line each
- specific to the code context
- written to explain intent, not mechanics
- short enough to read instantly

## Example Invocation
add one-line comments to all functions and structs across the codebase, following the rules above.
- `/one-line-commenter all`


add one-line comments to all functions and structs in the `crates/z00z_core/src/assets` module, following the rules above.
- `/one-line-commenter crates/z00z_core/src/assets`

- `/one-line-commenter crates/z00z_core/src/assets/registry.rs`

- `/one-line-commenter crates/z00z_core/src/assets/registry.rs::verify_claim`

- `/one-line-commenter crates/z00z_core/src/assets/registry.rs::WalletSnapshot`

## Related Prompt Example

If a function is already obvious, prefer no comment at all.
If a comment is needed, keep it narrow and actionable.
