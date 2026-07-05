#!/bin/bash

# Run Rust dependency and supply-chain checks.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
STRICT="${Z00Z_L4_STRICT:-0}"
VENDOR_ROOT_REL="${Z00Z_VENDOR_ROOT:-crates/z00z_crypto/tari}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
DEFAULT_SUPPLY_CHAIN_REPORT_PREFIX="$RUN_ROOT/supply-chain/supply-chain"
DEFAULT_SUPPLY_CHAIN_REPO_DIR="$ROOT_DIR/.reviews"
SUPPLY_CHAIN_REPORT_PREFIX="${Z00Z_SUPPLY_CHAIN_REPORT_PREFIX:-$DEFAULT_SUPPLY_CHAIN_REPORT_PREFIX}"
SUPPLY_CHAIN_SUMMARY_PATH="${Z00Z_SUPPLY_CHAIN_SUMMARY_PATH:-${SUPPLY_CHAIN_REPORT_PREFIX}-summary.json}"
SUPPLY_CHAIN_PROJECT_REPORT="${Z00Z_SUPPLY_CHAIN_PROJECT_REPORT:-${SUPPLY_CHAIN_REPORT_PREFIX}-project.md}"
SUPPLY_CHAIN_VENDOR_REPORT="${Z00Z_SUPPLY_CHAIN_VENDOR_REPORT:-${SUPPLY_CHAIN_REPORT_PREFIX}-vendor.md}"
SUPPLY_CHAIN_DUPLICATES_REPORT="${Z00Z_SUPPLY_CHAIN_DUPLICATES_REPORT:-${SUPPLY_CHAIN_REPORT_PREFIX}-duplicates.txt}"
SUPPLY_CHAIN_REVIEW_FILE="${Z00Z_SUPPLY_CHAIN_REVIEW_FILE:-$DEFAULT_SUPPLY_CHAIN_REPO_DIR/reviewed-advisories.toml}"
SUPPLY_CHAIN_VET_STORE="${Z00Z_SUPPLY_CHAIN_VET_STORE:-$DEFAULT_SUPPLY_CHAIN_REPO_DIR}"
Z00Z_RUN_CACHE_ROOT="${Z00Z_RUN_CACHE_ROOT:-$RUN_ROOT/.cache}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

cd "$ROOT_DIR"
FAILED=0

cleanup_root_tmp_dir() {
  # shellcheck disable=SC2317
  if [[ -d "$ROOT_DIR/tmp" ]]; then
    rmdir "$ROOT_DIR/tmp" 2>/dev/null || true
  fi
}

trap cleanup_root_tmp_dir EXIT

log() {
  printf '[z00z-l4:supply] %s\n' "$1"
}

is_semver_network_error() {
  local log_path="$1"
  rg -q 'HTTP2 framing layer|curl failed|failed to update registry|failed to get .* as a dependency|download of .* failed' "$log_path"
}

sanitize_semver_ref() {
  printf '%s' "$1" | tr '/:@ ' '_' | tr -c '[:alnum:]_-' '_'
}

is_semver_baseline_compile_error() {
  local log_path="$1"
  local semver_base="$2"
  local baseline_tag
  baseline_tag="$(sanitize_semver_ref "$semver_base")"

  rg -q "target/semver-checks/git-${baseline_tag}/" "$log_path" &&
    rg -q 'error\[E[0-9]{4}\]|error: could not compile' "$log_path"
}

run_semver_checks() {
  local semver_base="$1"
  local semver_log
  semver_log="$(mktemp "${TMPDIR:-/tmp}/z00z-semver.XXXXXX")"

  if z00z_profile_run_command command "supply:semver:$semver_base" \
    env \
    CARGO_HTTP_MULTIPLEXING="${CARGO_HTTP_MULTIPLEXING:-false}" \
    CARGO_NET_RETRY="${CARGO_NET_RETRY:-10}" \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL="${CARGO_REGISTRIES_CRATES_IO_PROTOCOL:-sparse}" \
    cargo semver-checks check-release --baseline-rev "$semver_base" >"$semver_log" 2>&1; then
    cat "$semver_log"
    rm -f "$semver_log"
    return 0
  fi

  cat "$semver_log"
  if is_semver_network_error "$semver_log"; then
    log "retrying cargo semver-checks after network transport failure"
    if z00z_profile_run_command command "supply:semver-retry:$semver_base" \
      env \
      CARGO_HTTP_MULTIPLEXING=false \
      CARGO_NET_RETRY=20 \
      CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
      cargo semver-checks check-release --baseline-rev "$semver_base"; then
      rm -f "$semver_log"
      return 0
    fi
  fi

  if is_semver_baseline_compile_error "$semver_log" "$semver_base"; then
    log "UNKNOWN: semver baseline $semver_base failed inside the cloned baseline tree; current-branch API comparison is blocked by baseline compile or rustdoc errors"
    rm -f "$semver_log"
    return 0
  fi

  rm -f "$semver_log"
  return 1
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    exit 1
  fi
  log "UNKNOWN: $message"
}

resolve_semver_base() {
  local candidate

  if [[ -n "${Z00Z_SEMVER_BASE:-}" ]]; then
    printf '%s\n' "$Z00Z_SEMVER_BASE"
    return 0
  fi

  for candidate in origin/main main; do
    if git rev-parse --verify "$candidate" >/dev/null 2>&1; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
}

ensure_review_file() {
  if [[ -f "$SUPPLY_CHAIN_REVIEW_FILE" ]]; then
    return 0
  fi
  mkdir -p "$(dirname "$SUPPLY_CHAIN_REVIEW_FILE")"
  cat >"$SUPPLY_CHAIN_REVIEW_FILE" <<'EOF'
reviewed = []
EOF
}

resolve_bootstrap_vet_store() {
  if [[ -n "$SUPPLY_CHAIN_VET_STORE" ]]; then
    printf '%s\n' "$SUPPLY_CHAIN_VET_STORE"
    return 0
  fi

  printf '%s\n' "$(dirname "$SUPPLY_CHAIN_REPORT_PREFIX")/cargo-vet"
}

resolve_vet_store_path() {
  local store="$1"
  case "$store" in
    /*) printf '%s\n' "$store" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$store" ;;
  esac
}

ensure_review_file

if command -v cargo-audit >/dev/null 2>&1 || cargo audit --version >/dev/null 2>&1; then
  log "cargo audit reviewed classification"
  mkdir -p "$(dirname "$SUPPLY_CHAIN_SUMMARY_PATH")"
  if ! z00z_profile_run_command command "supply:review-advisories" python3 "$SCRIPT_DIR/review-advisories.py" \
    --root "$ROOT_DIR" \
    --vendor-root "$ROOT_DIR/$VENDOR_ROOT_REL" \
    --review-file "$SUPPLY_CHAIN_REVIEW_FILE" \
    --summary-out "$SUPPLY_CHAIN_SUMMARY_PATH" \
    --project-report "$SUPPLY_CHAIN_PROJECT_REPORT" \
    --vendor-report "$SUPPLY_CHAIN_VENDOR_REPORT"; then
    echo "ERROR: cargo audit review pass failed. If this is a RustSec database parse error, update cargo-audit with scripts/verification-tools/install-verification-tools.sh --install --profile all --upgrade" >&2
    FAILED=1
  else
    summary="$(
      python3 - "$SUPPLY_CHAIN_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
project = data.get("project", {})
vendor = data.get("vendor", {})
mixed = data.get("mixed", {})
print(project.get("unreviewed", 0))
print(project.get("reviewed", 0))
print(vendor.get("unreviewed", 0))
print(vendor.get("reviewed", 0))
print(mixed.get("unreviewed", 0))
print(mixed.get("reviewed", 0))
PY
)"
    mapfile -t summary_lines <<< "$summary"
    project_unreviewed="${summary_lines[0]:-0}"
    project_reviewed="${summary_lines[1]:-0}"
    vendor_unreviewed="${summary_lines[2]:-0}"
    vendor_reviewed="${summary_lines[3]:-0}"
    mixed_unreviewed="${summary_lines[4]:-0}"
    mixed_reviewed="${summary_lines[5]:-0}"
    log "supply-chain project report -> ${SUPPLY_CHAIN_PROJECT_REPORT#"$ROOT_DIR"/}"
    log "supply-chain vendor report -> ${SUPPLY_CHAIN_VENDOR_REPORT#"$ROOT_DIR"/}"

    if [[ "$project_unreviewed" -gt 0 || "$mixed_unreviewed" -gt 0 ]]; then
      echo "ERROR: unresolved project-owned supply-chain advisories remain; see ${SUPPLY_CHAIN_PROJECT_REPORT#"$ROOT_DIR"/}" >&2
      FAILED=1
    fi

    if [[ "$project_reviewed" -gt 0 || "$vendor_unreviewed" -gt 0 || "$vendor_reviewed" -gt 0 || "$mixed_reviewed" -gt 0 ]]; then
      log "NEEDS_HUMAN_CRYPTO_REVIEW: reviewed or vendor-scoped supply-chain exceptions remain; see ${SUPPLY_CHAIN_PROJECT_REPORT#"$ROOT_DIR"/} and ${SUPPLY_CHAIN_VENDOR_REPORT#"$ROOT_DIR"/}"
    fi
  fi
else
  unknown_or_fail "cargo-audit is not installed"
fi

if command -v cargo-deny >/dev/null 2>&1 || cargo deny --version >/dev/null 2>&1; then
  if [[ -f "deny.toml" || -f ".cargo/deny.toml" ]]; then
    log "cargo deny check bans licenses sources"
    deny_log="$(mktemp "${TMPDIR:-/tmp}/z00z-deny.XXXXXX")"
    if ! z00z_profile_run_command command "supply:deny" cargo deny check bans licenses sources >"$deny_log" 2>&1; then
      deny_output="$(cat "$deny_log")"
      printf '%s\n' "$deny_output"
      if grep -Fq "unsupported CVSS version: 4.0" <<<"$deny_output"; then
        unknown_or_fail "cargo-deny is too old for the current RustSec advisory DB; upgrade to >= 0.19.0"
      else
        FAILED=1
      fi
    else
      deny_output="$(cat "$deny_log")"
    fi
    rm -f "$deny_log"
    printf '%s\n' "$deny_output"
  else
    log "UNKNOWN: no deny.toml found"
  fi
else
  unknown_or_fail "cargo-deny is not installed"
fi

if command -v cargo-vet >/dev/null 2>&1 || cargo vet --version >/dev/null 2>&1; then
  repo_vet_store="$(resolve_vet_store_path "$SUPPLY_CHAIN_VET_STORE")"
  if [[ -f "$repo_vet_store/config.toml" ]]; then
    log "cargo vet check -> ${repo_vet_store#"$ROOT_DIR"/}"
    if ! z00z_profile_run_command command "supply:vet" cargo vet check --store-path "$repo_vet_store"; then
      FAILED=1
    elif rg -q '^\[\[exemptions\.' "$repo_vet_store/config.toml"; then
      log "NEEDS_HUMAN_CRYPTO_REVIEW: configured cargo-vet store at ${repo_vet_store#"$ROOT_DIR"/} still carries explicit exemptions; trust is repository-owned by path, but not yet mature"
    fi
  else
    bootstrap_vet_store="$(resolve_vet_store_path "$(resolve_bootstrap_vet_store)")"
    mkdir -p "$(dirname "$bootstrap_vet_store")"
    if [[ ! -f "$bootstrap_vet_store/config.toml" ]]; then
      log "cargo vet bootstrap init -> ${bootstrap_vet_store#"$ROOT_DIR"/}"
      if ! z00z_profile_run_command command "supply:vet:init" cargo vet init --locked --store-path "$bootstrap_vet_store"; then
        unknown_or_fail "cargo vet bootstrap init failed"
        bootstrap_vet_store=""
      fi
    fi

    if [[ -n "$bootstrap_vet_store" && -f "$bootstrap_vet_store/config.toml" ]]; then
      log "cargo vet bootstrap check -> ${bootstrap_vet_store#"$ROOT_DIR"/}"
      if ! z00z_profile_run_command command "supply:vet:bootstrap" cargo vet check --store-path "$bootstrap_vet_store"; then
        FAILED=1
      else
        log "NEEDS_HUMAN_CRYPTO_REVIEW: cargo-vet bootstrap exemptions are active at ${bootstrap_vet_store#"$ROOT_DIR"/}; review and shrink them before treating vet coverage as trusted"
      fi
    fi
  fi
else
  unknown_or_fail "cargo-vet is not installed"
fi

mkdir -p "$(dirname "$SUPPLY_CHAIN_DUPLICATES_REPORT")"
log "duplicate dependency report -> ${SUPPLY_CHAIN_DUPLICATES_REPORT#"$ROOT_DIR"/}"
if z00z_profile_run_command command "supply:duplicates" cargo tree -d >"$SUPPLY_CHAIN_DUPLICATES_REPORT"; then
  duplicate_count="$(grep -Ec '^[[:alnum:]_-]+ v[0-9]' "$SUPPLY_CHAIN_DUPLICATES_REPORT" || true)"
  log "duplicate dependency roots: $duplicate_count"
else
  log "duplicate dependency report generation failed"
fi

if command -v cargo-semver-checks >/dev/null 2>&1 || cargo semver-checks --version >/dev/null 2>&1; then
  semver_base="$(resolve_semver_base || true)"
  if [[ -n "$semver_base" ]]; then
    log "cargo semver-checks check-release --baseline-rev $semver_base"
    if ! run_semver_checks "$semver_base"; then
      FAILED=1
    fi
  else
    log "UNKNOWN: could not resolve a semver baseline ref"
  fi
else
  log "UNKNOWN: cargo-semver-checks is not installed"
fi

exit "$FAILED"
