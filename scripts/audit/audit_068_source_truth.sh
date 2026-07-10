#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

if ! command -v rg >/dev/null 2>&1; then
  echo "ripgrep (rg) is required" >&2
  exit 1
fi

cd "$REPO_ROOT"

scenario_doc=".planning/phases/090-New-Scenarios/080-Recursive-Ready-Checkpoint-Scenarios.md"
phase_dir=".planning/phases/068-Checkpoint-Contract"
fixture_dir="$SCRIPT_DIR/fixtures/068-source-truth"
positive_fixture="$fixture_dir/pass_current_code_only.md"
negative_fixtures=(
  "$fixture_dir/fail_live_external_da.md"
  "$fixture_dir/fail_live_pq_recursive_security.md"
  "$fixture_dir/fail_recursive_authority.md"
  "$fixture_dir/fail_nova_pq_authority.md"
  "$fixture_dir/fail_plonky3_nova_only.md"
  "$fixture_dir/fail_watcher_settlement.md"
  "$fixture_dir/fail_da_state_validity.md"
  "$fixture_dir/fail_phase_068_dependency_pins.md"
  "$fixture_dir/fail_provider_native_authority.md"
  "$fixture_dir/fail_second_checkpoint_theorem.md"
)

required_files=(
  "$phase_dir/068-TODO.md"
  "$phase_dir/068-CONTEXT.md"
  "$phase_dir/068-COVERAGE.md"
  "$phase_dir/068-TEST-SPEC.md"
  "$phase_dir/068-TESTS-TASKS.md"
  ".planning/phases/069-Recursive-Proof/069-TODO.md"
  "$scenario_doc"
  "crates/z00z_storage/src/checkpoint/mod.rs"
  "crates/z00z_rollup_node/src/da.rs"
  "$positive_fixture"
)

for idx in $(seq -w 1 16); do
  required_files+=("$phase_dir/068-${idx}-PLAN.md")
done

for fixture in "${negative_fixtures[@]}"; do
  required_files+=("$fixture")
done

failures=()

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    failures+=("missing required artifact: $file")
  fi
done

assert_contains() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if ! rg -q -F "$pattern" "$file"; then
    failures+=("$label missing in $file :: $pattern")
  fi
}

assert_contains "$phase_dir/068-TODO.md" "$scenario_doc" "scenario source ref"
assert_contains "$phase_dir/068-CONTEXT.md" "$scenario_doc" "context scenario ref"
assert_contains "$phase_dir/068-TEST-SPEC.md" "$scenario_doc" "test spec scenario ref"
assert_contains "$phase_dir/068-TESTS-TASKS.md" "$scenario_doc" "tests tasks scenario ref"
assert_contains "$phase_dir/068-CONTEXT.md" ".planning/phases/069-Recursive-Proof/069-TODO.md" "successor compatibility ref"
assert_contains "$phase_dir/068-TEST-SPEC.md" "TS-01" "test slice lower bound"
assert_contains "$phase_dir/068-TEST-SPEC.md" "TS-16" "test slice upper bound"
assert_contains "$phase_dir/068-TESTS-TASKS.md" "TT-01" "test task lower bound"
assert_contains "$phase_dir/068-TESTS-TASKS.md" "TT-16" "test task upper bound"
assert_contains "$scenario_doc" "MUST NOT claim live external DA" "scenario guardrail"
assert_contains "$scenario_doc" "MUST NOT claim live PQ security" "scenario guardrail"
assert_contains "$scenario_doc" "MUST NOT claim production recursive validity" "scenario guardrail"
assert_contains "$scenario_doc" "MUST preserve the Nova/Plonky3 implementation track required by Phase 069." "scenario guardrail"
assert_contains "$scenario_doc" "MUST NOT pin Nova, Plonky3, or IPFS/Kubo client versions inside Phase 068;" "scenario guardrail"
assert_contains "$scenario_doc" "Phase 069 remains the dependency authority for those pins." "scenario guardrail"
assert_contains "$scenario_doc" 'Any future `z00z_recursive_proofs` crate inherits Phase 069 dependency pins' "scenario guardrail"
assert_contains "$scenario_doc" "Local DA publication readiness is availability evidence only; it does not" "scenario guardrail"
assert_contains "$scenario_doc" "watchers remain advisory only" "scenario guardrail"
assert_contains "$scenario_doc" "cargo test --release -p z00z_storage --doc" "scenario verification anchor"
assert_contains "$scenario_doc" "bash scripts/audit/audit_068_source_truth.sh" "scenario verification anchor"
assert_contains "$scenario_doc" "cargo test --release -p z00z_rollup_node --test test_da_local_sim --test test_da_local_quorum_binding -- --nocapture" "scenario verification anchor"
assert_contains "$scenario_doc" "cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture" "scenario verification anchor"
assert_contains "$scenario_doc" "cargo test --release -p z00z_watchers --test test_hjmt_publication_contract -- --nocapture" "scenario verification anchor"
assert_contains "$scenario_doc" "cargo test --release -p z00z_simulator --test scenario_1 test_checkpoint_acceptance -- --nocapture" "scenario verification anchor"
assert_contains "$scenario_doc" "cargo test --release" "scenario verification anchor"
assert_contains ".planning/phases/069-Recursive-Proof/069-TODO.md" "authority for Nova, Plonky3, and IPFS/Kubo integration." "phase 069 dependency authority"
assert_contains "crates/z00z_storage/src/checkpoint/mod.rs" "Validators and watchers consume storage-owned checkpoint artifacts" "checkpoint rustdoc boundary"
assert_contains "crates/z00z_rollup_node/src/da.rs" "Publication readiness is availability evidence only; it does not prove state validity" "rollup da rustdoc boundary"

claim_labels=(
  "live external da"
  "live pq recursive security"
  "recursive authority"
  "nova pq authority"
  "plonky3 nova-only authority"
  "watcher settlement authority"
  "da state validity"
  "phase 068 dependency pinning"
  "provider-native authority"
  "second checkpoint theorem"
)

claim_patterns=(
  "live[[:space:]-]+external[[:space:]-]+da"
  "live[[:space:]-]+pq[[:space:]-]+recursive[[:space:]-]+security|production[[:space:]-]+recursive[[:space:]-]+validity"
  "recursive[[:space:]-]+sidecar.*(canonical admission|is authoritative|are authoritative|acts as authority|acts as canonical admission)"
  "nova.*(pq-safe|post-quantum|quantum-safe)"
  "plonky3.*only.*nova.*verifier"
  "watcher(s)? .*validate(s)? settlement|watcher(s)? .*settlement authority"
  "(^|[^[:alnum:]])da .*prove(s)? state validity|data-availability .*prove(s)? state validity"
  "phase 068 .*pin(s|ned)? .*nova|phase 068 .*pin(s|ned)? .*plonky3|phase 068 .*pin(s|ned)? .*(ipfs|kubo)"
  "provider-native authority|sdk-native .*storage-owned authority|provider sdk-native types as storage-owned authority"
  "second[[:space:]-]+checkpoint[[:space:]-]+theorem|validator duplicate theorem|validator(s)? .*own(s)? .*checkpoint theorem"
)

guard_scan_file() {
  local file="$1"
  local lowered
  lowered="$(tr '[:upper:]' '[:lower:]' < "$file")"
  local idx
  for idx in "${!claim_patterns[@]}"; do
    if printf '%s\n' "$lowered" | grep -Eiq -- "${claim_patterns[$idx]}"; then
      printf '%s\n' "${claim_labels[$idx]}"
      return 1
    fi
  done
  return 0
}

expect_guard_pass() {
  local file="$1"
  local reason
  if ! reason="$(guard_scan_file "$file")"; then
    failures+=("allowed source-truth fixture rejected for ${reason}: $file")
  fi
}

expect_guard_fail() {
  local file="$1"
  local reason
  if reason="$(guard_scan_file "$file")"; then
    failures+=("forbidden source-truth fixture passed: $file")
  fi
}

expect_guard_pass "$positive_fixture"
for fixture in "${negative_fixtures[@]}"; do
  expect_guard_fail "$fixture"
done

if [[ ${#failures[@]} -eq 0 ]]; then
  printf 'phase 068 source truth audit passed\n'
  exit 0
fi

printf 'phase 068 source truth audit failed:\n' >&2
for failure in "${failures[@]}"; do
  printf '  - %s\n' "$failure" >&2
done
exit 1
