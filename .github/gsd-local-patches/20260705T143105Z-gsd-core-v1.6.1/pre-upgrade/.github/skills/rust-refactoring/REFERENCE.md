# Rust Refactoring Reference

Focused patterns and operational guidance for Rust refactors driven by Cargo
validation.

## Core Goals

Use Rust refactoring to improve structure without changing intended behavior.
The strongest outcomes are smaller stage boundaries, clearer ownership, and more
precise error surfaces.

## High-Value Patterns

### Ownership Boundary Cleanup

Prefer signatures that reveal intent.

```rust
pub fn summarize(blocks: &[Block]) -> Summary {
    blocks.iter().fold(Summary::default(), Summary::from_block)
}
```

Use borrowed inputs for read-only flows, owned values for transfer boundaries,
and iterators when the caller should control allocation.

### Stage Split

Break long functions into named steps.

```rust
pub fn execute_claim(input: &[u8]) -> Result<ClaimReport, ClaimErr> {
    let raw = parse_claim(input)?;
    let checked = validate_claim(raw)?;
    let staged = stage_claim(checked)?;
    persist_claim(staged)
}
```

Stage splits reduce lifetime complexity and make targeted testing easier.

### Typed Error Surfaces

Promote failure classes into explicit enums.

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClaimErr {
    #[error("invalid amount")]
    InvalidAmount,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

Typed errors are especially valuable when refactoring verification pipelines,
because they preserve behavior semantics better than flattened strings.

### Trait Extraction

Extract traits when orchestration depends on behavior but not on a concrete
backend.

```rust
pub trait SnapshotStore {
    fn load(&self, id: SnapshotId) -> Result<Snapshot, StoreErr>;
    fn save(&self, snapshot: &Snapshot) -> Result<(), StoreErr>;
}
```

Avoid mock-only trait extraction. Traits should reflect a real seam.

### Async Boundary Isolation

Keep blocking or CPU-heavy work outside async orchestration where possible.

```rust
pub async fn sync_snapshot(client: &Client, repo: &Repo) -> Result<(), SyncErr> {
    let bytes = client.fetch_snapshot().await?;
    let snapshot = decode_snapshot(&bytes)?;
    repo.store(&snapshot)?;
    Ok(())
}
```

## Module Patterns

Split modules by responsibility, not just by line count.

```text
ledger/
  mod.rs
  types.rs
  errors.rs
  verify.rs
  builder.rs
  tests.rs
```

Recommended module roles:

- `types.rs` for public structs and enums.
- `errors.rs` for typed failures.
- `verify.rs` for rule evaluation.
- `builder.rs` for assembly or authoring flows.
- `tests.rs` for local unit coverage where the crate style permits it.

## Tooling

Use the default toolchain as part of the refactor loop.

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

Escalate when needed:

- `rust-analyzer` for interactive rename, extract, and ownership inspection.
- `cargo fix --edition` for edition migrations.
- `cargo expand` for macro-heavy code paths.
- `cargo metrics` when module complexity is part of the decision.
- `cargo nextest run` for larger suites.
- `cargo tarpaulin` when coverage confidence is the blocker.
- `cargo miri test` for unsafe or aliasing-sensitive code.
- `cargo bench` or Criterion when performance claims matter.

## Workflow

### Pre-Refactor Assessment

- Confirm the current branch is green.
- Identify public APIs and re-exports that must stay stable.
- Map ownership hot spots and clone-heavy paths.
- Decide whether the refactor is intra-module, crate-local, or cross-crate.

### Refactoring Loop

1. Add or verify guard tests.
2. Extract pure helpers first.
3. Change one ownership or interface seam at a time.
4. Run Cargo validation after each step.
5. Re-check visibility, errors, and docs before stopping.

### Post-Refactor Checks

- Public API drift.
- Error variant drift.
- Async blocking drift.
- Clone or allocation drift.
- Missing doc or example updates.

## Modernization Strategies

Use targeted modernization only when it directly supports the refactor goal.

### Edition Upgrades

- 2015 to 2018: remove `extern crate`, replace `try!`, simplify modules.
- 2018 to 2021: align async usage, exploit better lifetime inference, clean old
  compatibility shims.
- Prefer `cargo fix --edition` before manual cleanup.

### Async Runtime Migration

```rust
// Before
use async_std::task;

task::spawn(async {
    // work
});

// After
use tokio::task;

task::spawn(async {
    // work
});
```

### Error Handling Evolution

```rust
// Before
fn process() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// After
#[derive(thiserror::Error, Debug)]
pub enum ProcessingErr {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

fn process() -> Result<(), ProcessingErr> {
    Ok(())
}
```

### Macro Modernization

```rust
#[proc_macro_derive(MyTrait)]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let generated = impl_my_trait(&ast);
    generated.into()
}
```

### FFI Safety Improvements

If the refactor touches unsafe or FFI edges, preserve the invariant comments,
shrink unsafe scope, and re-validate aliasing and ownership assumptions.

## Failure Modes

Watch for these regressions:

- Hidden allocation growth after signature cleanup.
- `pub` visibility spreading farther than intended.
- Borrow-checker workarounds that change ordering or lifetime semantics.
- Blocking work moved into async call paths.
- Error flattening that loses domain meaning.

## Anti-Patterns to Avoid

### Arc Mutex by Default

```rust
// Bad
struct Service {
    db: Arc<Mutex<Database>>,
    cache: Arc<Mutex<Cache>>,
    config: Arc<Mutex<Config>>,
}

// Better
struct Service {
    db: Database,
    cache: Option<Arc<Cache>>,
    config: Arc<Config>,
}
```

### Complex Lifetime Signatures

```rust
// Bad
fn process<'a, 'b, 'c>(data: &'a Data<'b>, config: &'c Config) -> Result<&'a str, Error>
where
    'b: 'a,
    'c: 'a,
{
    todo!()
}

// Better
fn process(data: &Data, config: &Config) -> Result<String, Error> {
    todo!()
}
```

## Worked Examples

### Eliminating Unnecessary Cloning

```rust
fn process_user(user: &User) -> Result<(), UserErr> {
    validate_user(user)?;
    transform_user(user)?;
    Ok(())
}
```

This is preferable to cloning the same struct repeatedly just to satisfy helper
signatures.

### Improving Async Safety

```rust
async fn handle_request(req: Request) -> Response {
    let profile = {
        let db = Database::new();
        let user = db.get_user(req.user_id).await;
        generate_profile(&user)
    };
    Response::json(profile)
}
```

Keep non-`Send` state and guards scoped so they do not cross the async boundary.

### Type-Safe State Management

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    fn can_cancel(&self) -> bool {
        matches!(self, OrderStatus::Created | OrderStatus::Processing)
    }
}
```

Use enums and newtypes when stringly-typed state is the real source of bugs.

## Best Practices

- Refactor from leaf dependencies inward.
- Prefer simple concrete types over ambitious generic rewrites.
- Keep lifetime complexity local.
- Use `pub(crate)` by default.
- Stop when the design becomes legible; do not keep extracting forever.
