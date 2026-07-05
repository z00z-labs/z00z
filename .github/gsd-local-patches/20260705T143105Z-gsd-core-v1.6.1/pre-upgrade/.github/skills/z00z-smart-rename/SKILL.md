---
name: z00z-smart-rename
description: 'Smart symbol rename audit for Rust code in the z00z workspace. Scans *.rs files for naming violations — abbreviations, wrong test_ prefix, identifier-length overflows, case-convention mismatches, and semantic mismatches. Builds a full inventory, runs an internal pros/cons evaluation for every rename candidate, and emits one unambiguous rename table with exact file and line coordinates. Use before any commit/PR, when cleaning up identifier quality, enforcing test_ naming, or doing a crate-scoped clarity pass. Trigger words: rename, smart rename, naming issues, abbreviations, test_ prefix, identifier quality, symbol clarity.'
argument-hint: '[path]​'
---

# Smart Rename

## When to Use

- The user wants a Rust naming audit before a commit or PR.
- Identifier quality, abbreviation expansion, `test_` naming, or semantic clarity must be reviewed systematically.
- The task is to produce an explicit rename table with file and line coordinates, not ad hoc rename suggestions.
- The request focuses on symbol clarity in `*.rs` files rather than generic refactoring.

## Mission

Act as a precision Symbol Rename Architect for the z00z Rust workspace.

For every **`*.rs` file** in the requested scope:

1. Build a complete symbol inventory (every function, method, struct, enum, variant, trait,
   type alias, const, static, macro, test file name, test function).
2. Detect naming violations against the rules below.
3. For each violating symbol, **internally** evaluate 2–3 candidate names via pros/cons and
   select exactly one winner. The user sees only the final winner.
4. Emit one unambiguous **rename table** — old name, new name, file, line, rationale.

**Scope locked to `*.rs` files only.**  
File renames are permitted *only* for test-only files that violate the `test_` prefix rule.  
No crate or module path renames (module identifiers are symbols and are in scope).

---

## Naming Rules (what to enforce)

### R1 · NO Ambiguous or Audit-Unsafe Abbreviations

Do **not** treat every abbreviation as a violation. The goal is not mechanical expansion.
The goal is to remove abbreviations that are **ambiguous, audit-hostile, or semantically
unsafe**, especially in cryptographic and protocol code.

Use this decision rule:

- **Keep** an abbreviation if it is widely understood in the current Rust or domain context,
  has low ambiguity, and expanding it would not materially improve readability.
- **Rename** an abbreviation if a competent reviewer cannot understand it without reading the
  implementation, or if it can plausibly mean multiple security-relevant things.
- **Escalate severity** for abbreviations that hide crypto intent, serialization boundaries,
  identity domains, or protocol state.

**Golden rule:** if a symbol cannot be understood with high confidence from its name alone,
the name is poor.

#### R1.1 · Keep by default

These are usually acceptable and should **not** be renamed just because they are shortened,
provided the full symbol remains readable and semantically obvious:

| Category | Usually acceptable |
|----------|--------------------|
| Rust / systems | `io`, `len`, `idx`, `cfg`, `err`, `ptr`, `str`, `num` |
| Crypto / security | `rng`, `sig`, `zkp`, `hmac`, `sha`, `aes` |
| Blockchain | `tx`, `bip`, `xpub`, `xprv` |

#### R1.1a · Prefix and suffix rule

Community-accepted abbreviations are generally acceptable as **prefixes or suffixes** when
they improve readability and the full symbol still communicates its role.

Good examples:

- `tx_hash`
- `sig_bytes`
- `rng_seed`
- `aes_cipher`
- `wallet_io_error`

Do **not** treat such affixes as violations when the composed name remains clear.

#### R1.1b · Crypto-audit naming rule

In audit-critical crypto and protocol code, accepted abbreviations like `pk`, `sk`, `vk`,
`sig`, `tx`, `rng`, `sha`, `aes`, and similar shorthand may be used as **readable affixes**,
but should generally **not** appear as complete standalone symbol names or full exported
identifiers.

Preferred:

- `transaction_signature`
- `account_public_key`
- `ephemeral_secret_key`
- `transaction_hash`
- `rng_provider`

Discouraged as full symbol names in shared or audit-facing code:

- `sig`
- `pk`
- `sk`
- `tx`
- `rng`

The shorter form may remain acceptable in very tight local scope, but for cross-module,
public, exported, or audit-facing symbols, prefer a name that spells out the role.

#### R1.2 · Context-dependent abbreviations

These are **not automatic violations**. Keep them only if the surrounding scope makes the
meaning obvious and there is no realistic alternate interpretation.

Apply the same general rule as in R1.1: context-dependent abbreviations are safest when
used as **readable prefixes or suffixes**, or inside a very tight local scope. They should
generally **not** stand alone as full symbol names, exported identifiers, or public-facing
signatures unless the meaning is unquestionably obvious to a reviewer.

Good forms:

- `previous_epoch`
- `command_handler`
- `service_config`
- `block_store`

Discouraged forms in shared code:

- `prev`
- `cmd`
- `svc`
- `blk`

| Abbreviation | Keep only when meaning is obvious | Prefer rename when ambiguous |
|--------------|-----------------------------------|------------------------------|
| `prev`, `next` | linked-list, iterator, cursor, ordered history code, especially as part of a fuller name | when the object is unclear: `previous_block`, `next_epoch` |
| `util` | module-level helper namespace with a clear bounded purpose, especially as an affix rather than a standalone exported name | vague catch-all symbol names |
| `cmd` | CLI, command bus, shell adapter code, especially in names like `command_handler` or `cmd_args` | domain logic where `command` is clearer |
| `svc` | service layer with established naming pattern, preferably as part of a longer readable identifier | when it could mean server, servant, or provider |
| `proc` | process-related OS code or procedural context with no ambiguity, preferably as an affix rather than the whole symbol name | when it could mean process, processor, or procedure |
| `hlpr`, `hndl` | almost never preferred, but may survive in legacy narrow local scope | rename in public API, tests, shared code, and audit-critical code |
| `blk` | explicit blockchain storage or consensus scope, especially in fuller names like `block_store` | anywhere it may be confused with black, block size, or generic blob |
| `pk`, `sk`, `vk` | tightly scoped crypto modules where the key role is already explicit and the abbreviation is not the full audit-facing name | when the exact role matters: `account_public_key`, `ephemeral_secret_key` |

#### R1.3 · Always suspect these names

Treat these as strong rename candidates because they routinely hide intent or create
security review failures.

| Problem class | Dangerous names | Why they are risky | Preferred style |
|---------------|-----------------|--------------------|-----------------|
| Meaningless placeholders | `tmp`, `data`, `val`, `res` | hide what is transformed or returned | `message_hash`, `signature`, `validation_result` |
| Raw byte ambiguity | `buf`, `bytes` | unclear whether bytes are key, nonce, hash, ciphertext, payload | `ciphertext`, `public_key_bytes`, `payload_bytes` |
| Single-letter crypto vars | `k`, `h`, `r`, `s` | may collapse key, nonce, hash, challenge, scalar, signature part | `session_key`, `challenge_hash`, `signature_r` |
| Identity ambiguity | `id` | unclear which identity domain it belongs to | `user_id`, `transaction_id`, `asset_id` |
| Message ambiguity | `msg` | unclear whether raw, encoded, transcript, or prehashed | `message`, `encoded_message`, `message_hash` |
| I/O role ambiguity | `input`, `out` | no semantics of what enters or leaves | `ciphertext`, `plaintext`, `encoded_claim` |
| Crypto material ambiguity | `key`, `nonce`, `iv`, `seed`, `secret` | may hide different trust and reuse properties | `session_key`, `aead_nonce`, `master_seed`, `shared_secret` |
| Protocol ambiguity | `ctx`, `state` | unclear whether domain separator, transcript, ratchet, handshake, runtime state | `protocol_context`, `domain_separator`, `handshake_state` |

#### R1.4 · Bad abbreviations are not only short

Flag an abbreviation even if it looks familiar when it has one of these traits:

- It can map to more than one domain concept.
- It hides whether the value is secret, public, ephemeral, persistent, hashed, encoded,
  or domain-separated.
- It forces the reviewer to read the implementation to recover the meaning.
- It is non-standard even among abbreviations, such as `mgr` instead of clearer forms like
  `manager` or an established local pattern.
- It uses an otherwise accepted crypto abbreviation as the **entire** symbol name instead of
  a readable role-bearing name.
- It appears in a public API, exported type, test name, or cross-module symbol where local
  context cannot rescue readability.

#### R1.5 · Rename strategy under the 5-word limit

If expanding an abbreviation would push the identifier above 5 words, do **not** keep the
bad abbreviation by default. Rephrase to a shorter, clearer name instead.

Examples:

- `wallet_id_is_deterministic_with_mock_rng` → shorten semantically, not cryptically.
- `process_buf` → `process_ciphertext` or `process_payload` depending on real meaning.
- `derive_k` → `derive_session_key` if that is the actual role.

### R2 · `test_` Prefix Mandatory for Test Symbols

| Target | Rule | Example |
|--------|------|---------|
| `#[test]` function | name MUST start with `test_` | `verify_balance` → `test_verify_balance` |
| `#[tokio::test]` / `#[async_std::test]` fn | name MUST start with `test_` | `async_flow` → `test_async_flow` |
| Test-only `.rs` file (file contains only test code, or lives in `tests/` dir, or ends in `_tests.rs` / `_test.rs`) | filename MUST start with `test_` | `nullifier_store_tests.rs` → `test_nullifier_store.rs` |

**Test-only file definition:** A file is "test-only" if:
- It lives under `tests/` (integration tests), OR  
- Its entire top-level code is wrapped in `#[cfg(test)]`, OR  
- Its filename ends with `_tests.rs` or `_test.rs`.

When renaming a test file, also update the `mod` declaration in the parent module (record
both rows in the rename table).

### R3 · Identifier Length ≤ 5 Words

Word count is measured by splitting on `_`, `-`, and PascalCase transitions.

> `wallet_id_deterministic_with_mock` = 5 words ✅  
> `wallet_id_is_deterministic_with_mock_rng` = 7 words ❌

For violations, the replacement name MUST have ≤ 5 words AND preserve semantic intent.

### R4 · Rust Idiom Compliance

| Kind | Required casing | Semantic pattern |
|------|-----------------|-----------------|
| Struct / Enum / Trait / TypeAlias | `PascalCase` | Noun |
| Enum variant | `PascalCase` | Noun or NounAdjective |
| Function / Method / Module | `snake_case` | Verb phrase (`compute_fee`, not `fee_computation`) |
| Boolean-returning predicate | `snake_case` with `is_` or `has_` prefix | `is_ready`, `has_capacity` |
| Constant / Static | `SCREAMING_SNAKE_CASE` | Noun |
| Macro (call site) | `snake_case!` | Verb or noun |

### R5 · Function Semantics

| Smell | Fix |
|-------|-----|
| Vague verb: `process`, `run`, `do`, `handle`, `compute`, `execute` | Replace with the precise action: `validate_fee`, `seal_output`, `derive_key` |
| Noun used as function name: `fee_calculation` | Rename to verb phrase: `calculate_fee` |
| Asymmetric pair: `encode` / `deserialize` | Align: `encode` / `decode` or `serialize` / `deserialize` |
| Leaking layer: `save_to_db`, `read_from_vec` | Remove the how: `persist`, `read` |
| Constructor inconsistency: `new`, `create`, `build`, `make` for same pattern | Standardize: `new` or `try_new`; builders get `::builder()` |

### R6 · Visibility Hygiene (do not rename — annotate)

Items that are `pub` but only referenced within the same crate should be noted as
candidates for `pub(crate)`. Record these as **INFO** rows in the rename table (no rename
needed, just visibility tightening — the implementor decides).

---

## Internal Decision Process (Pros & Cons)

> **This section describes what the AI does inside its reasoning before writing any row
> of the rename table. It is NOT surfaced verbatim to the user — only the final winning
> name is recorded.**

For every symbol that requires a rename, generate **2–3 candidate names**, then score each
on these five criteria (1 = weak, 3 = strong):

| Criterion | Guiding question |
|-----------|-----------------|
| **Semantic precision** | Does the name fully capture the symbol's role without over- or under-specifying? |
| **Brevity** | Is it within the 5-word limit while still being self-documenting? |
| **Rust idiom fit** | Does it match the correct case convention and semantic pattern for its kind? |
| **Codebase consistency** | Does it follow the naming patterns already established in the same module/crate? |
| **Ambiguity-free** | Does it avoid ambiguous or audit-unsafe abbreviations per R1 while keeping accepted domain shorthand when clear? |

Sum the five scores (max 15). The candidate with the **highest total** is the winner.
If two candidates tie, prefer shorter. Record only the winner in the final table.

**Document the winning rationale** in the "Rationale" column of the rename table using
a one-line explanation that references the winning criterion(s), e.g.:
`"Function named as noun; corrected to verb phrase; 5-word limit respected."`

---

## Execution Workflow

### Step 1 — Scope Resolution

| Argument type | Scope |
|---------------|-------|
| `path/to/file.rs` | Audit that single file only |
| Crate name (e.g. `z00z_core`) | Audit `crates/<name>/src/**/*.rs` |
| `"all"` or omitted | Audit `crates/*/src/**/*.rs` and `tests/**/*.rs`; exclude `z00z_crypto/tari/` |

Read all `.rs` files in scope. For large scopes use `grep_search` to locate violation
candidates before deep reads.

### Step 2 — Symbol Inventory

For each `.rs` file in scope, extract every symbol. Use `grep_search` with the pattern:

```
(pub |pub\(crate\) |pub\(super\) )?(struct|enum|trait|type|fn|const|static|macro_rules!|mod)\b
```

Supplement with patterns for `#[test]`, `#[tokio::test]`, `#[async_std::test]` to catch
all test functions.

For each symbol record: **Name · Kind · Visibility · Module path · File · Line**.

Also record every **test-only `.rs` filename** that does not start with `test_`.

### Step 3 — Violation Detection

Run all symbols through R1–R5.  Mark each finding with one of:

| Tag | Meaning |
|-----|---------|
| `RENAME` | Name violates a rule; a new name must be proposed |
| `INFO` | Visibility or structural note; no rename required |
| `OK` | Name passes all rules; no change |

Discard all `OK` symbols from further processing.

### Step 4 — Internal Pros & Cons (per violating symbol)

For each `RENAME` finding, apply the internal decision process described above.
Select the winning name. Record it.

> Do not surface the candidate list or scores to the user. Only the final winner and its
> one-line rationale appear in the output.

### Step 5 — Emit the Rename Table

Produce the table as described in the Output section below.

### Step 6 — Emit File-Rename Rows (if any)

If any test-only `.rs` files violate R2, add one row per rename to the same table, tagged
`file` in the Kind column. Add a second row for the corresponding `mod` declaration update.

---

## Output Format

### Primary Output — Rename Table

Emit **one flat table** covering all proposed renames, sorted by file path then line.

```markdown
## Rename Plan — <scope> — <YYYY-MM-DD>

| # | Kind | Old Name | File | Line | New Name | Violation | Rationale |
|---|------|----------|------|------|----------|-----------|-----------|
| 1 | fn | `process_data` | `crates/z00z_core/src/tx/mod.rs` | 42 | `validate_transaction` | R5 vague verb | Verb "process" replaced with precise "validate"; object scoped to "transaction" |
| 2 | test fn | `should_pass` | `crates/z00z_core/src/tx/mod.rs` | 100 | `test_should_pass` | R2 missing test_ | All #[test] functions must start with `test_` |
| 3 | file | `nullifier_store_tests.rs` | `crates/z00z_wallets/src/core/claim/` | — | `test_nullifier_store.rs` | R2 file prefix | Test-only file must start with `test_` |
| 4 (file-ref) | mod decl | `nullifier_store_tests` | `crates/z00z_wallets/src/core/claim/mod.rs` | 7 | `test_nullifier_store` | R2 file prefix | mod declaration mirrors the file rename |
| 5 | struct | `TxMgr` | `crates/z00z_core/src/pool.rs` | 15 | `TransactionManager` | R1 abbrev + R4 PascalCase noun | `Tx` → `Transaction` (R1); `Mgr` → `Manager` (R1) |
| 6 | const | `MAX_BUF_SZ` | `crates/z00z_utils/src/io/mod.rs` | 3 | `MAX_BUFFER_SIZE` | R1 abbrev | `BUF` → `BUFFER`, `SZ` → `SIZE`; SCREAMING_SNAKE kept |
```

**Rules:**
- Every row has a non-empty `File` and `Line` (use `—` only for file rename rows where
  line is not applicable).
- `File` path is relative to workspace root.
- `Kind` values: `fn`, `test fn`, `async test fn`, `struct`, `enum`, `enum variant`,
  `trait`, `type alias`, `const`, `static`, `macro`, `mod`, `file`, `mod decl` (for
  mod declaration updates caused by file renames).
- `Violation` cites the rule ID(s): `R1`, `R2`, `R3`, `R4`, `R5`, `R6`.
- `Rationale` is one sentence referencing the winning criterion(s) from the pros/cons
  evaluation.

### Secondary Output — Summary

After the table, emit a brief summary block:

```markdown
## Summary

- Files scanned: N
- Symbols inventoried: N
- Rename proposals: N  (breakdown: R1=N, R2=N, R3=N, R4=N, R5=N)
- INFO notes: N
- Files to rename: N
```

### Optional: Symbol Inventory (on request)

If the user asks for the full inventory (`--inventory` flag or explicit request), emit a
second table listing ALL symbols (including `OK` ones):

| # | Module Path | Symbol | Kind | Vis | Role (1 line) | File | Line | Status |
|---|-------------|--------|------|-----|---------------|------|------|--------|
| … | … | … | … | … | … | … | … | OK / RENAME / INFO |

---

## Quality Assurance Checklist (self-check before emitting output)

- [ ] Every `RENAME` row has a non-empty `New Name` that has been through internal pros/cons.
- [ ] No winning name is > 5 words (counted with `_`, `-`, PascalCase splitting).
- [ ] No winning name contains a non-standard abbreviation per R1.
- [ ] All `#[test]` / `#[tokio::test]` functions in the scope have `test_` prefix (or are
      listed for rename).
- [ ] All test-only `*.rs` filenames start with `test_` (or are listed for rename).
- [ ] File rename rows are always paired with their `mod decl` row.
- [ ] File and module paths in the table are verified against the actual workspace tree
      (use `file_search` or `grep_search` if uncertain).
- [ ] No files outside `*.rs` are touched or mentioned.
- [ ] `z00z_crypto/tari/` is excluded from scope entirely.
- [ ] Public renames carry a note about deprecation shim if the symbol is exported from
      a `lib.rs` facade.

---

## Prohibited Actions

- Renaming crate names or workspace members.
- Renaming non-`*.rs` files other than test-only `.rs` files with wrong prefix.
- Proposing churny style renames where the current name already passes all rules.
- Showing the full pros/cons candidate list to the user — only the winner goes in the table.
- Creating or modifying any source file — this skill only produces the rename plan.
  Implementation is done by the developer (or a separate execution step).
- Touching any file in `z00z_crypto/tari/`.

---

## Tooling Commands

```bash
# Find all symbols quickly
rg -n "^\s*(pub(\(crate\)|\(super\))?\s+)?(fn|struct|enum|trait|type|const|static|macro_rules!|mod)\b" \
  crates/z00z_core/src/ --type rust

# Find all test functions
rg -n "#\[(test|tokio::test|async_std::test)\]" crates/ --type rust -A 1

# Find test-only files violating prefix rule
find crates/ tests/ -name "*.rs" | grep -E "_tests?\.rs$"

# Verify no old names remain after apply
rg "OLD_NAME" crates/ --type rust
```

## Example Invocations

### Single file


`/z00z-smart-rename crates/z00z_wallets/src/core/claim/nullifier_store.rs`


### Single crate


`/z00z-smart-rename z00z_wallets`


### Whole workspace

`/z00z-smart-rename all`

```text
Audit all *.rs files in the workspace and include the full symbol inventory.
```
