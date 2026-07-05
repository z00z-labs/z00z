---
description: 'Operational GitHub Copilot rules for the Z00Z repository'
applyTo: '**'
---

**Version:** 2.2  
**Last Updated:** 2026-04-16  
**Project:** Z00Z - Privacy-focused blockchain with confidential transactions

---

## 🎯 Core Principles (@MUST reference)

### 🚫 1. English-Only Policy

**MANDATORY:** All code, comments, documentation, commit messages, and technical content MUST be written exclusively in English.

**Applies to:**
- ✅ Source code and inline comments
- ✅ API documentation and README files
- ✅ Commit messages and pull request descriptions
- ✅ Technical specifications and architecture documents
- ✅ Error messages and logging output
- ✅ Configuration files and environment variables

**Exception:** In the chat terminal, write responses to User in Russian using Cyrillic.

---

### ⛔ 2. Safe File Operations

**CRITICAL:** NEVER use destructive deletion commands without explicit user confirmation.

**On Linux:**
- ✅ Use `trash-put <path>` (trash-cli package)
- ✅ Use `gio trash <path>` (GNOME utilities)
- ❌ NEVER use `rm -rf`
- ❌ NEVER use `rm -r`
- ❌ NEVER use `rm` with wildcards (`rm *.tmp`) without confirmation

**If trash utilities are unavailable:** Ask the user for a preferred safe-delete method.

**Before full file reset or overwrite:** If an existing file with extension `txt`, `md`, `json`, `yaml`, `yml`, or `csv` is going to be rewritten from scratch as a whole file, first create a sibling backup with the `.bak` suffix before truncating, clearing, or replacing its content.

**This backup rule applies only to reset-style rewrites:**
- ✅ Create `<file>.bak` before zeroing or replacing the full content of an existing text-like file
- ✅ Create `<file>.bak` before discarding the current content and writing a new version from scratch
- ❌ No `.bak` is required for normal in-place edits, patch-based edits, or small targeted updates
- ❌ No `.bak` is required for simple file deletion; this rule is only for content reset or full overwrite flows

---

### 🔑 3. Version Management

**MANDATORY:** Use `./.github/skills/z00z-git-versioning/scripts/version-manager.sh` for all version updates and git release flows.

**Supported flows:**
```bash
./.github/skills/z00z-git-versioning/scripts/version-manager.sh patch -m "Bug fix description"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor -m "New feature description"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh major -m "Breaking change description"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh crate <name> <version> -m "Update description"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh sync -f -b <current-branch>
```

**Reference:** `.github/skills/z00z-git-versioning/scripts/VERSION_MANAGEMENT.md`

---

### 🛑 4. Protected Directories

**CRITICAL:** The `z00z_crypto/tari/` directory is read-only vendor code.

- ✅ Can compile and use Tari libraries
- ✅ Can export new functionality through `z00z_crypto/src/lib.rs`
- ❌ NEVER modify source files in `tari/` subdirectories

---

### 🔔 5. User Interaction Signals

**MANDATORY:** At the end of each Copilot cycle, when user interaction is expected, execute:

```bash
./scripts/play_tone.sh
```

---

### ⭐ 6. Documentation Standards

**MANDATORY:** Use one leading emoji on H2-H4 headings when it clarifies structure. Choose emoji semantically for the concrete section instead of defaulting to `📌` everywhere.

**Preferred emoji set:** `📌 🎯 ⏰ 💥 ⚙️ 🔑 ♨️ ⭐ 👍 ☢️ 🚫 💯 👁️‍🗨️ 🚨 🛑 🔔 🚩 ⚠️ ⛔ ✅ ❌ ‼️ ❓ 🐞`

**Date format:** ISO 8601 (`YYYY-MM-DD`)

---

## ⚙️ Architecture Guidelines

This file is an operational rulebook, not a second architecture manual. Canonical architecture and detailed policy belong in the Design Foundation and the focused instruction files.

### 🔑 Canonical Sources

- **ONE SOURCE OF TRUTH:** Read Section 1 in `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` before touching file I/O, serialization, config, time, logging, metrics, or RNG boundaries.
- **Rust structure and split rules:** Use `.github/instructions/rust.instructions.md` plus the Rust Implementation Standards and Split Policy sections in the Design Foundation.
- **Bounded-code rules:** Follow the NASA Rules section in the Design Foundation for all new code and substantial modifications.
- **API and naming rules:** Use the canonical Error Handling, Public API Design, Naming Conventions, and Identifier Length Rule sections in the Design Foundation.

### 🚨 Local Red Flags

- Business-logic crates MUST NOT bypass `z00z_utils` where a project abstraction already exists.
- When this file and the Design Foundation overlap, follow the stricter rule and keep examples only in the canonical document.
- If a rule starts expanding into tutorial-style explanation, move that content into the canonical document instead of growing this file.

---

## Copilot Token Discipline

MANDATORY use `Token Discipline Skill` -->  `skills/token-discipline`

### Output policy

- Do not repeat the user's request.
- No generic preambles. Answer the task directly.
- Prefer exact code, commands, paths, and diffs.
- Keep explanations brief but sufficient.
- For code edits, show changed files and tests, not long prose.
- For reviews, use a compact findings table.
- For architecture, give one recommendation first, then trade-offs.
- Do not add generic introductions.
- Do not add generic conclusions.
- Prefer exact code, commands, file paths, diffs, tables, YAML, or JSON.
- Do not paste full files unless requested.
- Use one strong recommendation instead of many weak alternatives.
- Keep reasoning visible only when it changes implementation decisions.
- NEVER omit critical warnings, failing tests, security issues, data-loss risks, or breaking changes.

### Budgets

- Simple answer: <= 3 lines.
- Normal coding answer: <= 5 bullets.
- Review: table + max 5 remediation bullets.
- Long answer only when explicitly requested.

### Quality rule

NEVER omit security, correctness, data-loss, breaking-change, or test-failure information to save tokens.



## 🔑 Tari Crypto Integration

### ✅ Approved Surface

- Use Tari-backed exports through `z00z_crypto`, including `PedersenCommitmentFactory`, `CommitmentSignature`, `RistrettoSchnorr`, `BulletproofsPlusService`, `DhKeyExchange`, `DerivedKeyDomain`, `Hidden<T>`, `SafePassword`, `to_hex()`, `from_hex()`, and `ByteArray`.
- Never add local tutorial examples here; keep API walkthroughs in the crypto reference documents.

### 🔑 Canonical References

- `.github/requirements/Tari-Crypto-Integration-Z00Z.md`
- `.github/requirements/Tari-Crypto-Components-Cookbook.md`

---

## ⚙️ Rust Standards

This section keeps only Z00Z-specific Rust deltas. General Rust development guidance lives in `.github/instructions/rust.instructions.md`.

### 🔑 Canonical Rust Sources

- Use `.github/instructions/rust.instructions.md` for general Rust coding, ownership, async, documentation, error handling, and testing guidance.
- Use `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` for Z00Z architecture, split policy, NASA Rules, public API boundaries, and naming rules.

### ✅ Rust Delivery Gate

- Run `cargo fmt` before commits.
- Run `cargo clippy --all-targets --all-features` with zero warnings.
- Run `cargo test --all`; all tests must pass.
- Run `cargo doc --no-deps` when public Rust APIs or docs change.
- Apply the NASA Rules with the strictest enforcement on safety-critical and high-assurance code paths.

### 🚫 Codacy

- Do not use Codacy in this repository.
- Use `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` and the Rust delivery gate above as the canonical verification path.
- For Markdown and planning docs, use editor diagnostics and markdownlint conventions instead of Codacy.

### 🔑 Local Rust Rules

- Types and structs: `PascalCase` nouns.
- Functions and methods: `snake_case` verbs.
- Booleans: `is_*` or `has_*` prefixes.
- Constants: `SCREAMING_SNAKE_CASE`.
- Modules: `snake_case` nouns.
- No identifiers longer than 5 words.
- Count words across `_`, `-`, and camelCase or PascalCase transitions.
- Apply the identifier limit to functions, methods, constants, failpoint IDs, metric IDs, and other signature-like identifiers.
- If violations already exist, report them and schedule or apply a rename.
- Recommended renames must stay within project style and match real behavior.
- Group all `use` imports from the same crate or module into a single `use` statement with braces.

---

## ✅ Operational Checklist

### ✅ Before Finalizing Work

- Follow Z00Z_DESIGN_FOUNDATION.md and keep project abstractions intact.
- Add or update tests when behavior changes.
- Add or update documentation when public behavior or public APIs change.
- Validate inputs, avoid logging secrets, and keep secrets out of error messages.
- Use audited Tari crypto primitives; do not implement custom cryptography.
- Keep public APIs stable unless a breaking change is explicitly intended and documented.

### 🚫 Avoid These Patterns

- Mixing transport concerns into core business logic.
- Bypassing `z00z_utils` abstractions inside business-logic crates.
- Exposing dependency-specific types in public APIs when project wrappers already exist.
- Overgeneralized APIs that hide real domain intent.
- `unwrap()`, `expect()`, or `panic!()` for production error handling.
- Logging or serializing secrets in plaintext.

---

## 👍 Usage Notes

### ✅ For Copilot

1. Read and internalize these rules before making changes.
2. Read the Design Foundation before architecture, crypto, or boundary changes.
3. Search for existing patterns in the repository before introducing new ones.
4. Verify compliance after changes instead of assuming correctness.
5. Prefer small, focused changes over large speculative refactors.

### 🔔 For Users

- Reference this file with `use .github/copilot-instructions.md;`.
- Reference key anchors with `@must`, `@rust.instructions`, and `@Z00Z_DESIGN_FOUNDATION`.
- Use the crypto references for Tari integration details and the utils references for `z00z_utils` APIs.

---

## 🔑 Canonical References

- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
- `.github/instructions/rust.instructions.md`
- `.github/requirements/Tari-Crypto-Integration-Z00Z.md`
- `.github/requirements/Tari-Crypto-Components-Cookbook.md`
- `crates/z00z_utils/Z00Z_UTILS_MODULE_MAP.md`
- `crates/z00z_utils/Z00Z_UTILS_QUICK_REFERENCE.md`
- `.github/skills/z00z-git-versioning/scripts/VERSION_MANAGEMENT.md`

---

**Maintainers:** Z00Z Development Team  
**License:** Internal Use Only

<!-- GSD Configuration — managed by gsd-core installer -->
# Instructions for GSD

- Use the gsd-core skill when the user asks for GSD or uses a `gsd-*` command.
- Treat `/gsd-...` or `gsd-...` as command invocations and load the matching file from `.github/skills/gsd-*`.
- When a command says to spawn a subagent, prefer a matching custom agent from `.github/agents`.
- Do not apply GSD workflows unless the user explicitly asks for them.
- After completing any `gsd-*` command (or any deliverable it triggers: feature, bug fix, tests, docs, etc.), ALWAYS: (1) offer the user the next step by prompting via `ask_user`; repeat this feedback loop until the user explicitly indicates they are done.
<!-- /GSD Configuration -->

## graphify

Before answering architecture or codebase questions, read `.graphify/GRAPH_REPORT.md` if it exists.
If `.graphify/wiki/index.md` exists, navigate it for deep questions.
Type `/graphify` in Copilot Chat to build or update the knowledge graph.
