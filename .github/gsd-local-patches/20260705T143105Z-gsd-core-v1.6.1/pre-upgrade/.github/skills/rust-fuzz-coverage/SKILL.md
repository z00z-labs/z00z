---
name: rust-fuzz-coverage
description: 'Analyze Rust code for hidden defects with cargo-fuzz and measure test coverage with cargo-llvm-cov. Use when the user asks for fuzzing, coverage, crash reproduction, corpus maintenance, sanitizer checks, uncovered paths, or stronger pre-merge quality gates. Trigger words: fuzz, fuzzing, cargo-fuzz, libFuzzer, coverage, cargo-llvm-cov, llvm-cov, crash reproducer, corpus, sanitizer, uncovered branches.'
argument-hint: '[setup|fuzz|coverage|regression|report] [crate|path|target]'
---

# Rust Fuzz And Coverage

## Mission

Use fuzzing and coverage as a practical defect-finding workflow for Rust code.
This skill must help the user:

- find crashers, panics, UB-adjacent behavior, and brittle parsing logic
- measure what tests actually execute
- turn crashes into stable regression tests
- identify high-value gaps instead of chasing vanity coverage

This is a developer-quality workflow. The repository must keep working without
`cargo-fuzz` or `cargo-llvm-cov` installed.

## When to Use

- adding or maintaining fuzz targets for parsers, verifiers, codecs, and input
  boundaries
- running smoke fuzzing before merge on high-risk code
- doing longer fuzz runs during hardening or triage
- measuring line and branch coverage for a crate or workspace
- turning crash artifacts into minimized repro inputs and regression tests
- auditing uncovered paths and proposing the next best tests to add
- converting a one-off fuzz or coverage request into a repeatable workflow

## Do Not Use This Skill For

- generic benchmarking or micro-performance tuning by itself
- formal security review without actual fuzz or coverage execution
- vendor-code edits inside `crates/z00z_crypto/tari/`
- broad code review when the user did not ask for fuzzing or coverage

## Invocation Signals

Invoke this skill when the user asks for outcomes like:

- `add fuzzing for this parser`
- `run coverage on this crate`
- `why does this parser crash on weird input`
- `set up cargo-fuzz`
- `measure uncovered code paths`
- `turn this crash into a regression test`
- `check sanitizer coverage on this input boundary`
- `show me what tests are missing here`

Strong technical triggers:

- `cargo-fuzz`
- `cargo llvm-cov`
- `llvm-cov`
- `libFuzzer`
- `corpus`
- `sanitizer`
- `coverage delta`
- `crash reproducer`

## Repository-Specific Conventions

Prefer repository evidence over assumptions.

Current workspace reality:

- crate-local fuzz harnesses already exist under
  `crates/z00z_core/fuzz/` and `crates/z00z_crypto/fuzz/`
- some vendored subtrees also contain fuzz or coverage helpers, but vendor code
  remains read-only
- coverage HTML reports conventionally land under `target/llvm-cov/html/`

Use these rules:

1. If the owning crate already has a `fuzz/` directory, extend that harness.
2. If no harness exists, create a crate-local `fuzz/` directory unless the repo
   already standardizes on a workspace-root fuzz package.
3. Never modify vendor code under `crates/z00z_crypto/tari/`. If a vendor-backed
   API needs fuzz coverage, target the exported wrapper or public integration
   surface instead.
4. Keep one fuzz target focused on one narrow function or entrypoint.

## Mode Selection

Choose one primary mode from the request, then execute it fully.

| Mode | Use when | Primary outcome |
|---|---|---|
| `setup` | tools or harnesses are missing | working local fuzz and coverage setup |
| `fuzz` | user wants crash finding or hardening | smoke or extended fuzz execution |
| `coverage` | user wants gaps, percentages, or report | coverage report plus gap analysis |
| `regression` | a crash or artifact already exists | minimized repro and stable regression test |
| `report` | user wants summary only | concise defect-risk and gap report |

If the request mixes modes, use this priority order:

1. `regression`
2. `fuzz`
3. `coverage`
4. `setup`
5. `report`

## Core Decision Logic

### Step 1: Identify the target surface

Choose the narrowest meaningful surface:

- parser
- verifier
- deserializer
- decoder
- reducer
- boundary adapter for untrusted bytes or messages

Good candidates:

- `try_from_bytes`
- `decode_*`
- `parse_*`
- `verify_*`
- `from_slice`
- `deserialize_*`

Avoid broad multi-subsystem targets unless no narrow boundary exists.

### Step 2: Choose harness location

Use this branch:

```text
existing crate-local fuzz harness?
  -> yes: extend that harness
  -> no: create crate-local fuzz package for the owning crate

user only wants coverage?
  -> no fuzz package changes unless necessary
```

### Step 3: Choose execution depth

Use these defaults unless the user specifies stronger gates:

- smoke fuzz: 30 seconds or 10k runs for a single target
- extended fuzz: minutes to hours, only when the user explicitly wants it
- focused coverage: one crate during iteration
- workspace coverage: before broader quality review or when the change spans
  multiple crates

### Step 4: Choose outputs

Always produce the smallest useful set:

- target file changes only when a harness or regression is needed
- command list actually used
- result summary with crashes, corpus additions, and coverage gaps
- specific next tests, not generic advice

## Execution Workflow

### Phase 1: Inspect And Confirm

Before editing or running heavy commands:

- identify the owning crate and target function
- inspect whether a fuzz harness already exists
- inspect whether a regression test already covers the discovered input
- decide whether coverage should be crate-scoped or workspace-scoped

Do not create duplicate fuzz targets for the same behavior.

### Phase 2: Verify Tool Availability

Check whether the required tools exist:

```bash
command -v cargo-fuzz
command -v cargo-llvm-cov
rustup component list --installed | grep llvm-tools-preview
```

If tooling is missing:

- explain exactly what is missing
- install only if the user asked for setup or approved installation
- keep the workflow usable even when the tools are absent by reporting the next
  exact commands the user can run later

Recommended setup:

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

### Phase 3: Add Or Refresh Fuzz Target

When a new target is needed:

1. Pick one clean byte-oriented or structured-input entrypoint.
2. Name the target by behavior, not by module size.
3. Add early returns for tiny or obviously invalid inputs when that helps reach
   deeper states faster.
4. Keep the body minimal. Prefer one call path plus safe post-parse checks.

Target naming rules:

- `parse_*`
- `verify_*`
- `serde_*`
- `decode_*`
- `reduce_*`

Example skeleton:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    let _ = my_crate::parser::parse_message(data);
});
```

### Phase 4: Run Fuzzing

Use smoke fuzzing first.

For crate-local harnesses, run from the harness directory:

```bash
cd crates/<crate>/fuzz
cargo fuzz run <target> -max_total_time=30
```

Alternative quick run:

```bash
cargo fuzz run <target> -runs=10000 -max_total_time=30
```

If a crash occurs:

- keep the exact artifact under `fuzz/artifacts/<target>/`
- minimize or preserve the smallest stable reproducer available
- move the minimized input into `fuzz/corpus/<target>/` if it is useful seed data
- add a regression test in the owning crate

### Phase 5: Run Coverage

Prefer the smallest relevant coverage scope during iteration.

Focused crate run:

```bash
cargo llvm-cov -p <crate> --all-features --no-report
cargo llvm-cov report --html
```

Workspace run:

```bash
cargo llvm-cov --workspace --all-features --no-report
cargo llvm-cov report --html
```

Open HTML only when it helps the user inspect the report interactively.

### Phase 6: Triage Gaps

Treat uncovered code as a prioritization problem, not a vanity metric problem.

Prioritize gaps in this order:

1. public parsing and decoding APIs
2. verification and proof-validation branches
3. serialization round-trips and malformed input handling
4. panic-prone edge cases and length-boundary logic
5. low-risk glue code

Recommended test type by gap:

| Gap type | Preferred test |
|---|---|
| public API example path | doctest or example |
| malformed edge input | unit test |
| input space invariant | property test |
| crash artifact | regression unit or integration test |
| parser hardening | fuzz target plus corpus seeds |

### Phase 7: Report Results

Every final report must say:

- what target or crate was analyzed
- what commands ran
- whether tooling was missing or available
- whether crashes occurred
- whether new fuzz targets, corpus seeds, or regression tests were added
- the important coverage result or gap summary
- the next highest-value test or fuzz action

## Quality Gates

Use these as default local gates unless the repo or user provides stricter ones.

### Fuzz Gate

- at least one smoke run for every new high-risk parser or verifier
- no untriaged crash artifacts left unexplained
- every reproducible crash must become either a regression test or a documented
  tracked follow-up

### Coverage Gate

- coverage report generated successfully for the requested scope
- uncovered critical branches are listed explicitly
- new public APIs should have either examples, doctests, or direct tests

### Recommended Thresholds

Use these as recommendations, not fabricated hard policy:

- aim for about 80% line coverage on critical parsing, proof, and serde-heavy
  modules
- require branch-focused tests where a false positive or false negative would be
  expensive

If the real project threshold differs, use the project threshold instead.

## Regression Workflow

When the user already has a crash artifact or failing input:

1. reproduce the failure with the same command, toolchain, and feature set
2. minimize the input if the current artifact is noisy or huge
3. identify the owning public or internal boundary
4. write the narrowest regression test that proves the bug stays fixed
5. keep the artifact only if it adds ongoing corpus value

Do not stop at `confirmed crash`. The workflow is incomplete until the crash is
either converted into a regression or explicitly blocked by missing information.

## Anti-Patterns

| Anti-pattern | Why it is bad | Corrective action |
|---|---|---|
| one giant fuzz target | poor diagnosis and low signal | split by behavior or boundary |
| root-only harness by habit | wrong ownership and messy maintenance | keep harness near owning crate |
| coverage run with no triage | percentages without action | list concrete uncovered branches |
| crash report without regression | bug can silently return | add stable test or tracked follow-up |
| editing vendor code for fuzzability | breaks boundary and maintainability | fuzz wrapper or exported surface |
| chasing 100% coverage | wasted effort and distorted priorities | focus on risk-heavy code |

## Troubleshooting

- `cargo fuzz run` exits immediately:
  target likely panics on tiny buffers or setup is wrong; add early guards and
  verify you are in the correct `fuzz/` directory.
- `cargo-llvm-cov` report is empty:
  ensure `llvm-tools-preview` is installed and that tests were launched via
  `cargo llvm-cov`, not plain `cargo test`.
- fuzzing is too slow:
  reduce input size, bound loops, seed with valid corpora, and shrink the target
  surface.
- crash is not reproducible:
  pin toolchain, features, and exact command; keep the original artifact path in
  the report.
- coverage is broad but not useful:
  switch from workspace scope to one crate and inspect branch-heavy files first.

## Minimal Command Set

```bash
# Smoke fuzz when a crate-local harness already exists
cd crates/<crate>/fuzz
cargo fuzz run <target> -max_total_time=30

# Focused crate coverage
cargo llvm-cov -p <crate> --all-features --no-report
cargo llvm-cov report --html

# Workspace coverage
cargo llvm-cov --workspace --all-features --no-report
cargo llvm-cov report --html
```

## Completion Checklist

Do not finalize until all relevant items are true:

- the chosen target boundary is explicit
- harness location matches crate ownership
- commands are reproducible and concrete
- crashes are either absent or converted into actionable regressions
- coverage output is paired with gap analysis, not just a percentage
- any created artifacts are named and located clearly
- vendor boundaries remain intact

## User Response Contract

When this skill completes, respond with:

- the analyzed crate, path, or fuzz target
- the concrete outcome in two to five lines
- file paths for any new target, regression test, or artifact location
- missing-tool blockers if they prevented execution
- one or two precise next actions only when they are genuinely useful

## Example Prompts

```text
Add a cargo-fuzz target for this parser and do a 30-second smoke run.
```

```text
Run cargo-llvm-cov for z00z_core and tell me which verifier branches are still uncovered.
```

```text
Take this crash artifact and turn it into a regression test.
```

```text
Audit the existing fuzz harnesses and tell me which high-risk input boundaries still have no target.
```

```text
Set up cargo-fuzz and cargo-llvm-cov for the crate I am editing.
```