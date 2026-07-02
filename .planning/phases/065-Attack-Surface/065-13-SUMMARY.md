---
phase: 065-Attack-Surface
plan: 065-13
status: complete
completed_at: 2026-07-02
next_plan: complete
summary_artifact_for: .planning/phases/065-Attack-Surface/065-13-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 065-13 Summary: Payment Request And Stealth Binding Closure

## 🎯 Outcome

`065-13` is complete.

`VR-13` closes the repeated request or stealth or wallet branch of
`l4-adversarial-review` on the project-owned path without editing
`crates/z00z_crypto/tari/**`. The current tree already had the required
request, receiver-card, stealth-output, and sensitive-RPC guardrails on the
canonical public paths, including an explicitly noncanonical raw stealth
builder lane and strong receive or replay coverage.

The live remaining drift was narrower:

- asset-import claim receipts were signed with `claim_scope_hash("dev-chain")`
  instead of the persisted wallet chain;
- asset RPC chain metadata existed on more than one local mapping path;
- the hash-policy proof for `z00z.payment.request.v1` and
  `z00z.receiver.card.v1` was implicit in code but not locked by explicit
  release tests.

After the fix:

- claim receipts bind to the persisted wallet chain, even if runtime chain env
  values drift after wallet creation;
- asset RPC chain metadata now uses one canonical helper path;
- payment-request and receiver-card domain-policy proofs are explicit;
- the required targeted release suites are green;
- the broad `cargo test --release` workspace gate ran through green;
- the protected vendor subtree stayed untouched.

## 📦 Files Changed

- `.planning/phases/065-Attack-Surface/065-13-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_crypto/tests/test_hash_policy.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_support_claims.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_support_state.rs`
- `crates/z00z_wallets/src/rpc/test_asset_impl.rs`

## 🔧 Landed Changes

- Persisted chain binding for claim receipts
  - `crates/z00z_wallets/src/rpc/asset_rpc_support_claims.rs` now resolves the
    persisted wallet chain via
    `resolve_persisted_wallet_chain_type(wallet_id)` and derives
    `claim_scope_hash(...)` from that chain label instead of the hardcoded
    `dev-chain` literal.
- Canonical chain metadata path
  - `crates/z00z_wallets/src/rpc/asset_rpc_impl.rs` now owns
    `chain_meta(...)` and `chain_meta_from_id(...)`.
  - `crates/z00z_wallets/src/rpc/asset_rpc_support_state.rs` now consumes that
    canonical helper and no longer carries a duplicate local mapping.
- Explicit regression coverage
  - `crates/z00z_wallets/src/rpc/test_asset_impl.rs` adds
    `test_claim_scope_chain`, proving that a wallet created on `testnet` keeps
    its claim scope on `testnet` even after the runtime env is switched to
    `mainnet`.
  - `crates/z00z_crypto/tests/test_hash_policy.rs` adds explicit request and
    receiver-card domain-policy tests to pin both domains to `Poseidon2`.
- Boundary preserved
  - No source under `crates/z00z_crypto/tari/**` was modified.
  - The existing raw stealth builder remains visibly noncanonical, and the
    public validated path continues to carry the production meaning.

## ✅ Validation

Commands and evidence used for `065-13` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_crypto --test test_hash_policy`
- `cargo test --release -p z00z_wallets test_claim_scope_chain -- --nocapture`
- `cargo test --release -p z00z_crypto`
- `cargo test --release -p z00z_wallets --test test_payment_request`
- `cargo test --release -p z00z_wallets --test test_asset_replay_protection`
- `cargo test --release -p z00z_wallets --test test_e2e_req_flow`
- `cargo test --release -p z00z_wallets --test test_stealth_output`
- `cargo test --release -p z00z_wallets --test test_view_key_contract`
- `cargo test --release -p z00z_wallets --test test_adversarial`
- `cargo test --release -p z00z_wallets --test test_rpc_route_coverage`
- `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session`
- `cargo test --release -p z00z_wallets --test test_import_error_taxonomy`
- `cargo fmt --all --check`
- `cargo test --release`

Observed proof points:

- `bootstrap_tests.sh` completed green before broader verification.
- The new targeted request or claim or hash-policy regressions completed green.
- The required focused `z00z_crypto` and `z00z_wallets` release suites
  completed green after the fix.
- `cargo fmt --all --check` completed green.
- The broad `cargo test --release` workspace run completed through the full
  suite with no failing output and no surviving `cargo test --release`
  process afterward.
- The protected vendor subtree remained untouched for this slice.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a usable automated review path for
this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-13-PLAN.md current_task="Payment Request And Stealth Binding Closure" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-13-PLAN.md current_task="Payment Request And Stealth Binding Closure" --yolo'`
  - Result: exited with code `1` and reported `Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-13-PLAN.md current_task="Payment Request And Stealth Binding Closure" --yolo'`
  - Result: exited with code `1` and reported `Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `065-13-PLAN.md`, `065-TODO.md`, `065-CONTEXT.md`, the residual
    verification-report sections for `VR-13`, the in-scope crypto or wallet
    anchors, and the targeted release tests.
  - Result: found one live project-owned drift in claim-scope chain binding and
    one duplicated asset-RPC chain mapping path.
- Pass 2
  - Re-checked the final patch and surrounding code paths to confirm that claim
    scope now binds to persisted wallet chain state, asset RPC chain metadata
    is centralized on one helper path, and no protected vendor code was edited.
  - Result: clean for the in-scope request or stealth or wallet hypotheses
    after the fix.
- Pass 3
  - Re-ran the targeted release suites, `cargo fmt --all --check`, and the
    broad `cargo test --release` gate.
  - Result: clean for the `065-13` scope. No remaining material project-owned
    drift was found.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
fix.

## 🧾 Closeout

`065-13` closes `VR-13` by binding claim receipts to persisted wallet chain
state, proving explicit request and receiver-card hash-policy coverage, and
showing that the project-owned request, stealth, and sensitive-RPC surfaces
already carry the required fail-closed behavior without vendor edits.

Phase `065` is now complete on the existing
`.planning/phases/065-Attack-Surface/` folder only. `065-TODO.md` remains the
normative human-readable authority, and its linked design or whitepaper corpus
remains live mandatory scope for the closed implementation.
