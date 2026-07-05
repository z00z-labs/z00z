#!/bin/bash

# Orchestrate Z00Z verification in report, fix, or find-and-fix mode.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$SCRIPT_DIR/profile-lib.sh"
PROFILE_SUMMARY_SCRIPT="$SCRIPT_DIR/summarize-profile-events.py"
RESOURCE_SUMMARY_SCRIPT="$SCRIPT_DIR/summarize-resource-profiles.py"
TOOL_INVENTORY_SCRIPT="$SCRIPT_DIR/inspect-profiler-tools.py"
RUN_FOOTPRINT_SCRIPT="$SCRIPT_DIR/summarize-run-footprint.py"
HJMT_SUMMARY_SCRIPT="$SCRIPT_DIR/summarize-hjmt-artifacts.py"
REPORT_VALIDATOR_SCRIPT="$SCRIPT_DIR/validate-report-format.py"
REPORT_FORMAT_PATH="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/FORMAT.md"
VENDOR_ROOT_REL="${Z00Z_VENDOR_ROOT:-crates/z00z_crypto/tari}"
FEATURE_FLAG="${Z00Z_ALL_FEATURES_FLAG-}"
ARTIFACT_BUILDER_SCRIPT="$ROOT_DIR/.github/skills/z00z-verification-artifact-builder/scripts/bootstrap-artifacts.py"
CODE_TO_LOGIC_SKILL_DIR="$ROOT_DIR/.github/skills/z00z-code-to-logic-gate"

source "$PROFILE_LIB"

MODE="report"
SCOPE_KIND=""
TARGET_INPUT=""
REPORT_INPUT=""
BASE_REF=""
DRY_RUN=0
RUN_ALL=0
LEVELS=""
REPORT_PATH=""
MAX_FIX_PASSES=2

TARGET_ROOT_ABS=""
TARGET_ROOT_REL=""
TARGET_MANIFEST_ABS=""
TARGET_MANIFEST_REL=""
TARGET_PACKAGE_NAME=""
TARGET_LABEL="project"
TARGET_TOUCHES_VENDOR=0
REPORT_LOG_DIR=""
OVERALL_STATUS="PASS"
COVERAGE_MANIFEST_PATH=""
COVERAGE_SUMMARY_PATH=""
COVERAGE_TRACKED_FILES=0
COVERAGE_FAIL_COUNT=0
COVERAGE_SKIPPED_COUNT=0
COVERAGE_UNKNOWN_COUNT=0
COVERAGE_UNMAPPED_COUNT=0
COVERAGE_CRATE_UNMAPPED_COUNT=0
BOOTSTRAP_SUMMARY_PATH=""
RUNTIME_BOOTSTRAP_SUMMARY_PATH=""
BOOTSTRAP_CREATED_COUNT=0
BOOTSTRAP_UPDATED_COUNT=0
BOOTSTRAP_SKIPPED_COUNT=0
PROFILE_EVENTS_PATH=""
PROFILE_SUMMARY_PATH=""
PROFILE_RESOURCE_DIR=""
PROFILE_RESOURCE_SUMMARY_PATH=""
PROFILE_TOOL_SUMMARY_PATH=""
RUN_FOOTPRINT_SUMMARY_PATH=""
HJMT_SUMMARY_PATH=""
REPORT_VALIDATION_SUMMARY_PATH=""
PROFILE_EVENT_COUNT=0
PROFILE_GATE_EVENT_COUNT=0
PROFILE_COMMAND_EVENT_COUNT=0
PROFILE_TOP_N=0
SECURITY_BRAINSTORM_SUMMARY_PATH=""
SECURITY_BRAINSTORM_REPORT_PATH=""
RUN_TIMESTAMP=""
RUN_ROOT=""
RUN_CACHE_ROOT=""
TMP_ROOT=""
SYSTEM_TMP_ROOT=""
CARGO_TARGET_ROOT=""
SPECS_RUNTIME_ROOT=""
VERIFICATION_RUNTIME_ROOT=""
FUZZ_RUNTIME_ROOT=""
RUNTIME_CWD_ROOT=""
RUN_STARTED_EPOCH=""
STALE_RUN_ROOTS_TRASHED=0
EXTERNAL_INTERFERERS_KILLED=0

declare -a REMAINING_ARGS=()
declare -A SELECTED_LEVELS=()
declare -A GATE_STATUS=()
declare -A GATE_LOG=()
declare -A GATE_LABEL=()
declare -A GATE_ELAPSED_SECS=()
declare -A GATE_MODULE=()
declare -A GATE_ARTIFACTS=()
declare -A GATE_LEAKS=()

usage() {
  cat <<'EOF'
Usage:
  orchestrate.sh [report project|report crate <crate>|fix report <report>|fix project|fix crate <crate>|find-and-fix project|find-and-fix crate <crate>] [OPTIONS]

Modes:
  report               Generate a report only. No code edits.
  fix                  Re-validate findings, apply bounded mechanical fixes, rerun gates, update report.
  find-and-fix         Run report -> bounded mechanical fix loop -> rerun gates -> update report.

Scopes:
  project              Whole-project verification. Defaults to all levels unless overridden.
  crate <crate>        Verify one crate root or a path inside a crate. Sub-crate paths resolve up to the owning crate.
  report <report>      Reuse metadata from a previously generated report. Valid only with fix mode.

Options:
  --base <ref>         Compare changed files against a base ref when inferring levels.
  --level <levels>     Comma-separated levels: l0,l1,l2,l3,l4.
  --all                Run every level for the resolved scope.
  --report-path <path> Write or update a report at the given path.
  --max-fix-passes <n> Maximum bounded mechanical fix passes. Default: 2.
  --dry-run            Print selected commands without running them.
  -h, --help           Show this help.

Examples:
  orchestrate.sh report crate crates/z00z_storage
  orchestrate.sh report project --all
  orchestrate.sh fix report reports/z00z-verification-orchestrator-20260616-120000/z00z-verification-report.md
  orchestrate.sh find-and-fix crate crates/z00z_crypto/tari
EOF
}

log() {
  printf '[z00z-verification] %s\n' "$1"
}

warn() {
  printf '[z00z-verification] WARNING: %s\n' "$1" >&2
}

die() {
  printf '[z00z-verification] ERROR: %s\n' "$1" >&2
  exit 1
}

have() {
  command -v "$1" >/dev/null 2>&1
}

run_cmd() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY'
    printf ' %q' "$@"
    printf '\n'
    return 0
  fi
  "$@"
}

safe_write_report() {
  local path="$1"
  local tmp_file="$2"

  mkdir -p "$(dirname "$path")"
  if [[ -f "$path" ]]; then
    cp "$path" "$path.bak"
  fi
  mv "$tmp_file" "$path"
}

sanitize_name() {
  local value="$1"
  value="${value//\//-}"
  value="${value// /-}"
  value="$(printf '%s' "$value" | tr -cd '[:alnum:]._-')"
  printf '%s\n' "${value:-project}"
}

default_report_path() {
  printf '%s/%s\n' "$RUN_ROOT" "z00z-verification-report.md"
}

default_run_timestamp() {
  date -u +%Y%m%d-%H%M%S
}

allocate_run_root() {
  local candidate

  while true; do
    candidate="$ROOT_DIR/reports/z00z-verification-orchestrator-$RUN_TIMESTAMP"
    if [[ ! -e "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
    if [[ -n "${Z00Z_REPORT_TIMESTAMP:-}" ]]; then
      die "run root already exists for explicit timestamp: $candidate"
    fi
    sleep 1
    RUN_TIMESTAMP="$(default_run_timestamp)"
  done
}

cleanup_stale_run_roots() {
  local reports_dir="$ROOT_DIR/reports"
  local path keep_dir base_name
  local -a keep_dirs=()

  [[ "$DRY_RUN" -eq 0 ]] || return 0
  [[ -d "$reports_dir" ]] || return 0
  [[ "${Z00Z_KEEP_PREVIOUS_RUNS:-0}" != "1" ]] || return 0
  [[ -n "$RUN_ROOT" ]] || return 0

  if [[ -n "$REPORT_INPUT" ]]; then
    keep_dirs+=("$(cd "$(dirname "$REPORT_INPUT")" && pwd)")
  fi

  shopt -s nullglob
  for path in \
    "$reports_dir"/z00z-verification-orchestrator-* \
    "$reports_dir"/z00z-verification-orchestrator-preflight-* \
    "$reports_dir"/repro-simcache.* \
    "$reports_dir"/repro-nextest-simcache.*; do
    [[ -d "$path" ]] || continue
    base_name="$(basename "$path")"
    case "$base_name" in
      z00z-verification-orchestrator-[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]-[0-9][0-9][0-9][0-9][0-9][0-9]) ;;
      z00z-verification-orchestrator-preflight-[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]-[0-9][0-9][0-9][0-9][0-9][0-9]) ;;
      repro-simcache.*|repro-nextest-simcache.*) ;;
      *) continue ;;
    esac
    [[ "$path" != "$RUN_ROOT" ]] || continue
    for keep_dir in "${keep_dirs[@]}"; do
      [[ "$path" != "$keep_dir" ]] || continue 2
    done
    if z00z_profile_live_root_owner "$path"; then
      continue
    fi
    if z00z_profile_trash_run_root "$path" >/dev/null 2>&1; then
      STALE_RUN_ROOTS_TRASHED=$((STALE_RUN_ROOTS_TRASHED + 1))
    fi
  done
  shopt -u nullglob
}

normalize_report_path() {
  local requested_path=""
  local basename_path=""

  RUN_TIMESTAMP="${RUN_TIMESTAMP:-$(default_run_timestamp)}"
  RUN_ROOT="$(allocate_run_root)"
  cleanup_stale_run_roots

  if [[ -n "$REPORT_PATH" ]]; then
    requested_path="$REPORT_PATH"
    case "$requested_path" in
      /*) ;;
      *)
        requested_path="$ROOT_DIR/$requested_path"
        ;;
    esac
    basename_path="$(basename "$requested_path")"
  fi

  if [[ -z "$basename_path" || "$basename_path" == "." || "$basename_path" == "/" ]]; then
    REPORT_PATH="$(default_report_path)"
  else
    REPORT_PATH="$RUN_ROOT/$basename_path"
  fi
}

parse_mode_and_scope() {
  if [[ $# -eq 0 ]]; then
    SCOPE_KIND="project"
    REMAINING_ARGS=()
    return 0
  fi

  case "$1" in
    report|fix|find-and-fix)
      MODE="$1"
      shift
      ;;
    --*)
      SCOPE_KIND="project"
      REMAINING_ARGS=("$@")
      return 0
      ;;
    *)
      die "unknown positional mode: $1"
      ;;
  esac

  if [[ $# -eq 0 ]]; then
    SCOPE_KIND="project"
    REMAINING_ARGS=()
    return 0
  fi

  case "$1" in
    project)
      SCOPE_KIND="project"
      shift
      ;;
    crate)
      [[ $# -ge 2 ]] || die "crate scope requires a crate name or path"
      SCOPE_KIND="crate"
      TARGET_INPUT="$2"
      shift 2
      ;;
    report)
      [[ "$MODE" == "fix" ]] || die "report scope is valid only with fix mode"
      [[ $# -ge 2 ]] || die "fix report requires a report path"
      SCOPE_KIND="report"
      REPORT_INPUT="$2"
      shift 2
      ;;
    --*)
      SCOPE_KIND="project"
      ;;
    *)
      die "unknown positional scope: $1"
      ;;
  esac

  REMAINING_ARGS=("$@")
}

parse_flags() {
  local args=("${REMAINING_ARGS[@]}")
  local index=0
  while [[ "$index" -lt "${#args[@]}" ]]; do
    case "${args[$index]}" in
      --base)
        (( index + 1 < ${#args[@]} )) || die "--base requires a ref"
        BASE_REF="${args[$((index + 1))]}"
        index=$((index + 2))
        ;;
      --level)
        (( index + 1 < ${#args[@]} )) || die "--level requires a comma-separated list"
        LEVELS="${args[$((index + 1))]}"
        index=$((index + 2))
        ;;
      --all)
        RUN_ALL=1
        index=$((index + 1))
        ;;
      --report-path)
        (( index + 1 < ${#args[@]} )) || die "--report-path requires a path"
        REPORT_PATH="${args[$((index + 1))]}"
        index=$((index + 2))
        ;;
      --max-fix-passes)
        (( index + 1 < ${#args[@]} )) || die "--max-fix-passes requires a positive integer"
        MAX_FIX_PASSES="${args[$((index + 1))]}"
        [[ "$MAX_FIX_PASSES" =~ ^[1-9][0-9]*$ ]] || die "--max-fix-passes must be a positive integer"
        index=$((index + 2))
        ;;
      --dry-run)
        DRY_RUN=1
        index=$((index + 1))
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        die "unknown option: ${args[$index]}"
        ;;
    esac
  done
}

load_report_metadata() {
  local report_path="$1"
  local meta

  case "$report_path" in
    /*) ;;
    *)
      report_path="$ROOT_DIR/$report_path"
      ;;
  esac

  [[ -f "$report_path" ]] || die "report does not exist: $report_path"
  REPORT_INPUT="$report_path"

  meta="$(python3 - "$report_path" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
scope = target = levels = ""
inside = False
for line in path.read_text(encoding="utf-8").splitlines():
    if line.strip() == "<!-- z00z-orchestrator-report":
        inside = True
        continue
    if inside and line.strip() == "-->":
        break
    if not inside or "=" not in line:
        continue
    key, value = line.split("=", 1)
    key = key.strip()
    value = value.strip()
    if key == "scope":
        scope = value
    elif key == "target":
        target = value
    elif key == "levels":
        levels = value
print(scope)
print(target)
print(levels)
PY
)"

  SCOPE_KIND="$(printf '%s\n' "$meta" | sed -n '1p')"
  TARGET_INPUT="$(printf '%s\n' "$meta" | sed -n '2p')"
  if [[ -z "$LEVELS" ]]; then
    LEVELS="$(printf '%s\n' "$meta" | sed -n '3p')"
  fi
  [[ -n "$SCOPE_KIND" ]] || die "report metadata is missing scope: $report_path"
  REPORT_PATH="${REPORT_PATH:-$(basename "$report_path")}"
}

resolve_target_from_path() {
  local input="$1"
  local candidate=""

  if [[ -e "$input" ]]; then
    candidate="$(cd "$(dirname "$input")" && pwd)/$(basename "$input")"
  elif [[ -e "$ROOT_DIR/$input" ]]; then
    candidate="$(cd "$ROOT_DIR/$(dirname "$input")" && pwd)/$(basename "$input")"
  elif [[ -e "$ROOT_DIR/crates/$input" ]]; then
    candidate="$(cd "$ROOT_DIR/crates/$input" && pwd)"
  elif [[ -e "$ROOT_DIR/crates/$input/Cargo.toml" ]]; then
    candidate="$ROOT_DIR/crates/$input"
  else
    return 1
  fi

  if [[ -f "$candidate" ]]; then
    candidate="$(cd "$(dirname "$candidate")" && pwd)"
  fi

  while [[ "$candidate" != "/" && "$candidate" != "$ROOT_DIR" ]]; do
    if [[ -f "$candidate/Cargo.toml" ]]; then
      TARGET_ROOT_ABS="$candidate"
      TARGET_MANIFEST_ABS="$candidate/Cargo.toml"
      TARGET_ROOT_REL="${candidate#"$ROOT_DIR"/}"
      TARGET_MANIFEST_REL="${TARGET_MANIFEST_ABS#"$ROOT_DIR"/}"
      return 0
    fi
    candidate="$(dirname "$candidate")"
  done

  return 1
}

resolve_target_from_package_name() {
  local input="$1"
  local package_info

  package_info="$(cargo metadata --no-deps --format-version 1 \
    | python3 -c '
import json
import sys

needle = sys.argv[1]
data = json.load(sys.stdin)
for package in data.get("packages", []):
    if package.get("name") != needle:
        continue
    print(package["manifest_path"])
    break
' "$input")"

  [[ -n "$package_info" ]] || return 1
  TARGET_MANIFEST_ABS="$package_info"
  TARGET_ROOT_ABS="$(dirname "$package_info")"
  TARGET_ROOT_REL="${TARGET_ROOT_ABS#"$ROOT_DIR"/}"
  TARGET_MANIFEST_REL="${TARGET_MANIFEST_ABS#"$ROOT_DIR"/}"
  return 0
}

resolve_package_name() {
  [[ -n "$TARGET_MANIFEST_ABS" ]] || return 0
  TARGET_PACKAGE_NAME="$(python3 - "$TARGET_MANIFEST_ABS" <<'PY'
import pathlib
import tomllib
import sys

manifest = pathlib.Path(sys.argv[1])
data = tomllib.loads(manifest.read_text(encoding="utf-8"))
print(data["package"]["name"])
PY
)"
}

resolve_scope() {
  cd "$ROOT_DIR"

  case "$SCOPE_KIND" in
    report)
      load_report_metadata "$REPORT_INPUT"
      resolve_scope
      return 0
      ;;
    project)
      TARGET_ROOT_ABS="$ROOT_DIR"
      TARGET_ROOT_REL="."
      TARGET_LABEL="project"
      ;;
    crate)
      resolve_target_from_path "$TARGET_INPUT" || resolve_target_from_package_name "$TARGET_INPUT" || die "could not resolve crate target: $TARGET_INPUT"
      resolve_package_name
      TARGET_LABEL="${TARGET_ROOT_REL:-$TARGET_INPUT}"
      ;;
    *)
      die "unsupported scope kind: $SCOPE_KIND"
      ;;
  esac

  if [[ -n "$TARGET_INPUT" ]]; then
    local original
    case "$TARGET_INPUT" in
      /*) original="$TARGET_INPUT" ;;
      *) original="$ROOT_DIR/$TARGET_INPUT" ;;
    esac
    if [[ -e "$original" ]]; then
      original="$(cd "$(dirname "$original")" && pwd)/$(basename "$original")"
      case "$original" in
        "$ROOT_DIR/$VENDOR_ROOT_REL"/*|"$ROOT_DIR/$VENDOR_ROOT_REL")
          TARGET_TOUCHES_VENDOR=1
          ;;
      esac
    fi
  fi

  if [[ "$TARGET_ROOT_ABS" == "$ROOT_DIR/$VENDOR_ROOT_REL" || "$TARGET_ROOT_ABS" == "$ROOT_DIR/${VENDOR_ROOT_REL%/*}" ]]; then
    TARGET_TOUCHES_VENDOR=1
  fi
}

changed_files() {
  if [[ -n "$BASE_REF" ]]; then
    git diff --name-only "$BASE_REF"...HEAD
    return 0
  fi

  git diff --name-only
  git diff --cached --name-only
  git status --short | awk '{print $2}'
}

collect_scope_candidates() {
  local have_changes=0

  if [[ "$SCOPE_KIND" == "project" && -z "$LEVELS" && "$RUN_ALL" -eq 0 ]]; then
    RUN_ALL=1
    return 0
  fi

  if [[ "$SCOPE_KIND" == "project" ]]; then
    while IFS= read -r file; do
      [[ -n "$file" ]] || continue
      have_changes=1
      printf '%s\n' "$file"
    done < <(changed_files | sort -u)
    return 0
  fi

  while IFS= read -r file; do
    [[ -n "$file" ]] || continue
    case "$file" in
      "$TARGET_ROOT_REL"/*|"$TARGET_ROOT_REL")
        have_changes=1
        printf '%s\n' "$file"
        ;;
    esac
  done < <(changed_files | sort -u)

  if [[ "$have_changes" -eq 0 ]]; then
    printf '%s\n' "$TARGET_ROOT_REL"
    [[ -n "$TARGET_MANIFEST_REL" ]] && printf '%s\n' "$TARGET_MANIFEST_REL"
  fi
}

select_level() {
  local level="$1"
  SELECTED_LEVELS["$level"]=1
}

infer_levels() {
  declare -gA SELECTED_LEVELS=()

  if [[ "$SCOPE_KIND" == "project" && -z "$LEVELS" && "$RUN_ALL" -eq 0 ]]; then
    RUN_ALL=1
  fi

  if [[ "$RUN_ALL" -eq 1 ]]; then
    for level in l0 l1 l2 l3 l4; do
      select_level "$level"
    done
    return 0
  fi

  if [[ -n "$LEVELS" ]]; then
    IFS=',' read -r -a requested_levels <<< "$LEVELS"
    for level in "${requested_levels[@]}"; do
      [[ -n "$level" ]] || continue
      select_level "$level"
    done
    return 0
  fi

  while IFS= read -r file; do
    [[ -n "$file" ]] || continue
    case "$file" in
      *.md|*.yaml|*.yml|*.toml|docs/*|specs/*)
        select_level l0
        ;;
    esac
    case "$file" in
      specs/tla/*|specs/alloy/*|crates/z00z_storage/*|crates/z00z_runtime/*|*checkpoint*|*settlement*|*voucher*|*rights*|*policy*)
        select_level l1
        ;;
    esac
    case "$file" in
      specs/tamarin/*|specs/proverif/*|specs/crypto/*|crates/z00z_crypto/*|*transcript*|*domain*|*proof*|*stealth*|*inbox*|*payment_request*)
        select_level l2
        ;;
    esac
    case "$file" in
      *.rs|crates/*|tests/*|benches/*|examples/*|Cargo.toml)
        select_level l3
        ;;
    esac
    case "$file" in
      Cargo.lock|Cargo.toml|crates/*/Cargo.toml|fuzz/*|crates/*/fuzz/*|*decode*|*parser*|*serde*|*unsafe*)
        select_level l4
        ;;
    esac
  done < <(collect_scope_candidates | sort -u)

  if [[ "$TARGET_TOUCHES_VENDOR" -eq 1 ]]; then
    select_level l2
    select_level l4
  fi

  if [[ "$SCOPE_KIND" == "crate" ]]; then
    case "$TARGET_ROOT_REL" in
      crates/z00z_storage|crates/z00z_runtime)
        select_level l1
        ;;
    esac

    case "$TARGET_ROOT_REL" in
      crates/z00z_crypto|crates/z00z_crypto/*)
        select_level l2
        ;;
    esac

    if [[ "${#SELECTED_LEVELS[@]}" -eq 0 ]]; then
      select_level l3
    fi
  fi
}

levels_csv() {
  local levels=()
  local level
  for level in l0 l1 l2 l3 l4; do
    [[ "${SELECTED_LEVELS[$level]:-0}" -eq 1 ]] || continue
    levels+=("$level")
  done
  local IFS=,
  printf '%s\n' "${levels[*]}"
}

init_report_state() {
  normalize_report_path
  RUN_ROOT="$(dirname "$REPORT_PATH")"
  REPORT_LOG_DIR="$RUN_ROOT/logs"
  BOOTSTRAP_SUMMARY_PATH="$RUN_ROOT/bootstrap-summary.json"
  RUNTIME_BOOTSTRAP_SUMMARY_PATH="$RUN_ROOT/runtime-bootstrap-summary.json"
  PROFILE_EVENTS_PATH="$RUN_ROOT/profiling/events.tsv"
  PROFILE_SUMMARY_PATH="$RUN_ROOT/profiling/summary.json"
  PROFILE_RESOURCE_DIR="$RUN_ROOT/profiling/resources"
  PROFILE_RESOURCE_SUMMARY_PATH="$RUN_ROOT/profiling/resources-summary.json"
  PROFILE_TOOL_SUMMARY_PATH="$RUN_ROOT/profiling/tool-availability.json"
  RUN_FOOTPRINT_SUMMARY_PATH="$RUN_ROOT/profiling/run-footprint.json"
  HJMT_SUMMARY_PATH="$RUN_ROOT/profiling/hjmt-summary.json"
  REPORT_VALIDATION_SUMMARY_PATH="$RUN_ROOT/report-validation.json"
  SECURITY_BRAINSTORM_SUMMARY_PATH="$RUN_ROOT/security/adversarial-summary.json"
  SECURITY_BRAINSTORM_REPORT_PATH="$RUN_ROOT/security/adversarial-review.md"
  RUN_CACHE_ROOT="$RUN_ROOT/.cache"
  TMP_ROOT="$RUN_ROOT/tmp$RUN_TIMESTAMP"
  CARGO_TARGET_ROOT="$RUN_ROOT/target"
  SPECS_RUNTIME_ROOT="$RUN_ROOT/specs$RUN_TIMESTAMP"
  VERIFICATION_RUNTIME_ROOT="$RUN_ROOT/verification$RUN_TIMESTAMP"
  FUZZ_RUNTIME_ROOT="$RUN_ROOT/fuzz$RUN_TIMESTAMP"
  RUNTIME_CWD_ROOT="$RUN_ROOT/workdir"
  SYSTEM_TMP_ROOT="$TMP_ROOT/system"
  RUN_STARTED_EPOCH="$(date -u +%s)"

  export Z00Z_VERIFICATION_RUN_ROOT="$RUN_ROOT"
  export Z00Z_VERIFICATION_TMPDIR="$TMP_ROOT"
  export Z00Z_REPORT_TIMESTAMP="$RUN_TIMESTAMP"
  export Z00Z_CARGO_PROFILE_ARGS="${Z00Z_CARGO_PROFILE_ARGS:---release}"
  export Z00Z_DISABLE_TIME_LIMITS="${Z00Z_DISABLE_TIME_LIMITS:-1}"
  export Z00Z_PROFILE_EVENTS_FILE="$PROFILE_EVENTS_PATH"
  export Z00Z_SPECS_RUNTIME_ROOT="$SPECS_RUNTIME_ROOT"
  export Z00Z_VERIFICATION_RUNTIME_ROOT="$VERIFICATION_RUNTIME_ROOT"
  export Z00Z_FUZZ_RUNTIME_ROOT="$FUZZ_RUNTIME_ROOT"
  export Z00Z_RUNTIME_CWD_ROOT="$RUNTIME_CWD_ROOT"
  export Z00Z_RUN_CACHE_ROOT="$RUN_CACHE_ROOT"
  export Z00Z_SYSTEM_TMPDIR="$SYSTEM_TMP_ROOT"
  export Z00Z_SIMULATOR_CACHE_ROOT="$RUN_CACHE_ROOT/scenario_1"
  export Z00Z_SIMULATOR_STORAGE_ROOT="$RUN_CACHE_ROOT/storage/scenario_1"
  export CARGO_TARGET_DIR="$CARGO_TARGET_ROOT"
  export Z00Z_SOURCE_CARGO_HOME="${Z00Z_SOURCE_CARGO_HOME:-${CARGO_HOME:-${HOME:-$ROOT_DIR}/.cargo}}"
  export CARGO_HOME="${Z00Z_CARGO_HOME:-$Z00Z_SOURCE_CARGO_HOME}"
  export CARGO_INSTALL_ROOT="$RUN_CACHE_ROOT/cargo-install"
  # Verification runs use an isolated Cargo home; defaulting that home to
  # offline mode turns missing local cache state into false gate failures.
  # Operators can still force offline reruns explicitly via CARGO_NET_OFFLINE.
  export CARGO_NET_OFFLINE="${CARGO_NET_OFFLINE:-false}"
  export TMPDIR="$SYSTEM_TMP_ROOT"
  export TMP="$SYSTEM_TMP_ROOT"
  export TEMP="$SYSTEM_TMP_ROOT"
  export XDG_CACHE_HOME="$RUN_CACHE_ROOT/xdg"
  export XDG_STATE_HOME="$RUN_CACHE_ROOT/xdg-state"
  export PYTHONPYCACHEPREFIX="$RUN_CACHE_ROOT/python/pycache"
  export PYTHONDONTWRITEBYTECODE=1
  export PIP_CACHE_DIR="$RUN_CACHE_ROOT/pip"
  export NPM_CONFIG_CACHE="$RUN_CACHE_ROOT/npm"
  export MYPY_CACHE_DIR="$RUN_CACHE_ROOT/mypy"
  export RUFF_CACHE_DIR="$RUN_CACHE_ROOT/ruff"
  export UV_CACHE_DIR="$RUN_CACHE_ROOT/uv"

  if [[ "$DRY_RUN" -eq 0 ]]; then
    mkdir -p \
      "$RUN_ROOT" \
      "$REPORT_LOG_DIR" \
      "$RUN_ROOT/profiling" \
      "$PROFILE_RESOURCE_DIR" \
      "$RUN_CACHE_ROOT" \
      "$CARGO_INSTALL_ROOT" \
      "$XDG_CACHE_HOME" \
      "$XDG_STATE_HOME" \
      "$PYTHONPYCACHEPREFIX" \
      "$PIP_CACHE_DIR" \
      "$NPM_CONFIG_CACHE" \
      "$MYPY_CACHE_DIR" \
      "$RUFF_CACHE_DIR" \
      "$UV_CACHE_DIR" \
      "$TMP_ROOT" \
      "$SYSTEM_TMP_ROOT" \
      "$CARGO_TARGET_ROOT" \
      "$SPECS_RUNTIME_ROOT" \
      "$VERIFICATION_RUNTIME_ROOT" \
      "$FUZZ_RUNTIME_ROOT" \
      "$RUNTIME_CWD_ROOT" \
      "$Z00Z_SIMULATOR_CACHE_ROOT" \
      "$Z00Z_SIMULATOR_STORAGE_ROOT"
    z00z_profile_init_file
  else
    mkdir -p "$RUN_ROOT" "$TMP_ROOT" "$SYSTEM_TMP_ROOT"
    export TMPDIR="$SYSTEM_TMP_ROOT"
    export TMP="$SYSTEM_TMP_ROOT"
    export TEMP="$SYSTEM_TMP_ROOT"
  fi
}

sanitize_preexisting_root_runtime_leaks() {
  local saved_epoch="${RUN_STARTED_EPOCH:-}"
  kill_external_interferers
  RUN_STARTED_EPOCH=""
  relocate_root_runtime_leaks preflight >/dev/null 2>&1 || true
  RUN_STARTED_EPOCH="$(date -u +%s)"
  if [[ -n "$saved_epoch" && ! "$RUN_STARTED_EPOCH" =~ ^[0-9]+$ ]]; then
    RUN_STARTED_EPOCH="$saved_epoch"
  fi
}

gate_log_path() {
  local gate_id="$1"
  printf '%s/%s.log\n' "$REPORT_LOG_DIR" "$gate_id"
}

success_status_from_log() {
  local log_path="$1"

  python3 - "$log_path" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
lines = []
for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
    line = raw.strip()
    if not line:
        continue
    if line.startswith("warning:") or line.startswith("help:"):
        continue
    lines.append(line)

status_order = (
    "NEEDS_HUMAN_CRYPTO_REVIEW",
    "UNKNOWN",
    "SKIPPED",
    "SECURITY_PROTOCOL_PROVED",
    "FORMALLY_PROVED",
    "MODEL_CHECKED",
    "BOUNDED_VERIFIED",
    "TESTED",
)

for status in status_order:
    needle = f"{status}:"
    if any(needle in line for line in lines):
        print(status)
        break
else:
    print("PASS")
PY
}

gate_runtime_cwd() {
  local gate_id="$1"
  printf '%s/%s\n' "$RUNTIME_CWD_ROOT" "$gate_id"
}

root_output_is_tracked() {
  local rel_path="$1"
  if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    return 1
  fi
  git -C "$ROOT_DIR" ls-files --error-unmatch -- "$rel_path" >/dev/null 2>&1
}

cleanup_system_tmp_root() {
  return 0
}

path_mtime_epoch() {
  local path="$1"
  stat -c '%Y' "$path" 2>/dev/null || stat -f '%m' "$path" 2>/dev/null || printf '0\n'
}

path_created_in_run() {
  local path="$1"
  local mtime_epoch
  [[ -e "$path" ]] || return 1
  [[ -n "$RUN_STARTED_EPOCH" ]] || return 0
  mtime_epoch="$(path_mtime_epoch "$path")"
  [[ "$mtime_epoch" =~ ^[0-9]+$ ]] || return 1
  (( mtime_epoch >= RUN_STARTED_EPOCH ))
}

gate_module_path() {
  local gate_id="$1"
  case "$gate_id" in
    l0-docs) printf '%s\n' ".github/skills/z00z-l0-spec-gate/scripts/check-docs.sh" ;;
    l1-tla) printf '%s\n' ".github/skills/z00z-l1-protocol-model-gate/scripts/run-tla.sh" ;;
    l1-apalache) printf '%s\n' ".github/skills/z00z-l1-protocol-model-gate/scripts/run-apalache.sh" ;;
    l1-alloy) printf '%s\n' ".github/skills/z00z-l1-protocol-model-gate/scripts/run-alloy.sh" ;;
    l2-domain) printf '%s\n' ".github/skills/z00z-l2-crypto-protocol-gate/scripts/check-domain-separation.py" ;;
    l2-transcript) printf '%s\n' ".github/skills/z00z-l2-crypto-protocol-gate/scripts/check-transcript-binding.py" ;;
    l2-proverif) printf '%s\n' ".github/skills/z00z-l2-crypto-protocol-gate/scripts/run-proverif.sh" ;;
    l2-tamarin) printf '%s\n' ".github/skills/z00z-l2-crypto-protocol-gate/scripts/run-tamarin.sh" ;;
    l2-hax) printf '%s\n' ".github/skills/z00z-l2-crypto-protocol-gate/scripts/run-hax.sh" ;;
    l2-refinement-map) printf '%s\n' ".github/skills/z00z-code-to-logic-gate/scripts/check-refinement-map.py" ;;
    l2-cryptol) printf '%s\n' ".github/skills/z00z-code-to-logic-gate/scripts/run-cryptol.sh" ;;
    l2-saw) printf '%s\n' ".github/skills/z00z-code-to-logic-gate/scripts/run-saw.sh" ;;
    l2-crux-mir) printf '%s\n' ".github/skills/z00z-code-to-logic-gate/scripts/run-crux-mir.sh" ;;
    l2-charon) printf '%s\n' ".github/skills/z00z-code-to-logic-gate/scripts/run-charon.sh" ;;
    l2-aeneas) printf '%s\n' ".github/skills/z00z-code-to-logic-gate/scripts/run-aeneas.sh" ;;
    l3-verify-fast) printf '%s\n' ".github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh" ;;
    l3-fmt) printf '%s\n' "cargo fmt --check" ;;
    l3-clippy) printf '%s\n' "cargo clippy" ;;
    l3-nextest|l3-nextest-ignored) printf '%s\n' "cargo nextest run" ;;
    l3-test|l3-test-ignored) printf '%s\n' "cargo test" ;;
    l3-doc) printf '%s\n' "cargo test --doc" ;;
    l3-miri) printf '%s\n' ".github/skills/z00z-l3-rust-implementation-gate/scripts/verify-miri.sh" ;;
    l3-kani) printf '%s\n' ".github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh" ;;
    l3-loom) printf '%s\n' ".github/skills/z00z-l3-rust-implementation-gate/scripts/verify-loom.sh" ;;
    l3-verus) printf '%s\n' ".github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh" ;;
    l3-prusti) printf '%s\n' ".github/skills/z00z-l3-rust-implementation-gate/scripts/verify-prusti.sh" ;;
    l4-supply-chain) printf '%s\n' ".github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh" ;;
    l4-unsafe|l4-vendor-unsafe) printf '%s\n' ".github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh" ;;
    l4-fuzz) printf '%s\n' ".github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-short.sh" ;;
    l4-constant-time) printf '%s\n' ".github/skills/z00z-l4-security-engineering-gate/scripts/run-constant-time.sh" ;;
    l4-adversarial-review) printf '%s\n' ".github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py" ;;
    *) printf '%s\n' "-" ;;
  esac
}

gate_artifact_paths() {
  local gate_id="$1"
  local emitted=0
  case "$gate_id" in
    l0-docs) printf '%s\n' "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}" ;;
    l1-tla) printf '%s\n%s\n' "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}/l1/tla-states" "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}/l1/tla-user" ;;
    l1-apalache) printf '%s\n' "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}/l1/apalache" ;;
    l1-alloy)
      if [[ -d "$SPECS_RUNTIME_ROOT/alloy" ]]; then
        printf '%s\n' "${SPECS_RUNTIME_ROOT#"$ROOT_DIR"/}/alloy"
        emitted=1
      fi
      if [[ -d "$VERIFICATION_RUNTIME_ROOT/l1/alloy" ]]; then
        printf '%s\n' "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}/l1/alloy"
        emitted=1
      fi
      if [[ "$emitted" -eq 0 ]]; then
        printf '%s\n' "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
      fi
      ;;
    l2-domain|l2-transcript) printf '%s\n' "${SPECS_RUNTIME_ROOT#"$ROOT_DIR"/}/crypto" ;;
    l2-proverif) printf '%s\n' "${SPECS_RUNTIME_ROOT#"$ROOT_DIR"/}/proverif" ;;
    l2-tamarin) printf '%s\n%s\n' "${SPECS_RUNTIME_ROOT#"$ROOT_DIR"/}/tamarin" "${TMP_ROOT#"$ROOT_DIR"/}/tamarin" ;;
    l2-hax) printf '%s\n' "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}/l2/hax" ;;
    l2-refinement-map|l2-cryptol|l2-saw|l2-crux-mir|l2-charon|l2-aeneas) printf '%s\n' "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}/code-to-logic" ;;
    l3-verify-fast|l3-fmt|l3-clippy|l3-nextest|l3-nextest-ignored|l3-test|l3-test-ignored|l3-doc|l3-miri|l3-kani|l3-loom|l3-verus|l3-prusti) printf '%s\n' "${CARGO_TARGET_ROOT#"$ROOT_DIR"/}" ;;
    l4-supply-chain) printf '%s\n' "${RUN_ROOT#"$ROOT_DIR"/}/supply-chain" ;;
    l4-unsafe|l4-vendor-unsafe) printf '%s\n%s\n' "${RUN_ROOT#"$ROOT_DIR"/}/vendor/vendor-unsafe.md" "${RUN_ROOT#"$ROOT_DIR"/}/geiger" ;;
    l4-fuzz) printf '%s\n' "${FUZZ_RUNTIME_ROOT#"$ROOT_DIR"/}" ;;
    l4-constant-time)
      if [[ -d "$VERIFICATION_RUNTIME_ROOT/dudect" ]]; then
        printf '%s\n' "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}/dudect"
        emitted=1
      fi
      if [[ -d "$CARGO_TARGET_ROOT/release" ]]; then
        printf '%s\n' "${CARGO_TARGET_ROOT#"$ROOT_DIR"/}/release"
        emitted=1
      fi
      if [[ "$emitted" -eq 0 ]]; then
        printf '%s\n' "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
      fi
      ;;
    l4-adversarial-review) printf '%s\n%s\n' "${SECURITY_BRAINSTORM_SUMMARY_PATH#"$ROOT_DIR"/}" "${SECURITY_BRAINSTORM_REPORT_PATH#"$ROOT_DIR"/}" ;;
    *) printf '%s\n' "-" ;;
  esac
}

gate_existing_artifacts_csv() {
  local gate_id="$1"
  local raw_csv="${GATE_ARTIFACTS[$gate_id]:--}"
  local item=""
  local filtered=()

  if [[ -z "$raw_csv" || "$raw_csv" == "-" ]]; then
    printf '%s\n' "-"
    return 0
  fi

  IFS=',' read -r -a raw_items <<< "$raw_csv"
  for item in "${raw_items[@]}"; do
    item="${item#"${item%%[![:space:]]*}"}"
    item="${item%"${item##*[![:space:]]}"}"
    [[ -n "$item" && "$item" != "-" ]] || continue
    [[ -e "$ROOT_DIR/$item" ]] || continue
    filtered+=("$item")
  done

  if [[ "${#filtered[@]}" -eq 0 ]]; then
    printf '%s\n' "-"
    return 0
  fi

  local IFS=','
  printf '%s\n' "${filtered[*]}"
}

relocate_root_runtime_leaks() {
  local gate_id="$1"
  local leak_dest_root="$RUN_ROOT/leaks/$gate_id"
  local rel_path abs_path dest_path rel_candidate candidate_basename
  local -a leaked=()
  local -a root_runtime_paths=(
    tmp
    cache
    state
    specs
    verification
    fuzz
    supply-chain
    .uv-cache
    .npm-cache
    .pip-cache
    .pycache
    _apalache-out
    states
    .xdg-state
    .ruff_cache
    .pytest_cache
    .mypy_cache
    __pycache__
    scripts/tmp
    scripts/__pycache__
  )

  for rel_path in "${root_runtime_paths[@]}"; do
    abs_path="$ROOT_DIR/$rel_path"
    [[ -e "$abs_path" ]] || continue
    path_created_in_run "$abs_path" || continue
    if [[ -d "$abs_path" ]] && rmdir "$abs_path" 2>/dev/null; then
      leaked+=("$rel_path(empty)")
      continue
    fi
    if root_output_is_tracked "$rel_path"; then
      leaked+=("$rel_path(tracked)")
      continue
    fi

    mkdir -p "$(dirname "$leak_dest_root/$rel_path")"
    dest_path="$leak_dest_root/$rel_path"
    if [[ -e "$dest_path" ]]; then
      dest_path="$dest_path-$RUN_TIMESTAMP"
    fi
    mv "$abs_path" "$dest_path"
    leaked+=("$rel_path")
  done

  shopt -s nullglob dotglob
  for abs_path in "$ROOT_DIR"/reports/bootstrap-sync* "$ROOT_DIR"/reports/.code-to-logic*; do
    [[ -e "$abs_path" ]] || continue
    [[ "$abs_path" == "$RUN_ROOT" ]] && continue
    path_created_in_run "$abs_path" || continue
    candidate_basename="$(basename "$abs_path")"
    rel_candidate="reports/$candidate_basename"
    mkdir -p "$leak_dest_root/reports"
    dest_path="$leak_dest_root/$rel_candidate"
    if [[ -e "$dest_path" ]]; then
      dest_path="$dest_path-$RUN_TIMESTAMP"
    fi
    mv "$abs_path" "$dest_path"
    leaked+=("$rel_candidate")
  done
  shopt -u nullglob dotglob

  if [[ "${#leaked[@]}" -eq 0 ]]; then
    return 1
  fi

  local IFS=', '
  printf '%s\n' "${leaked[*]}"
  return 0
}

external_interferer_pids() {
  local args pid
  while IFS= read -r line; do
    pid="${line%% *}"
    args="${line#* }"
    [[ "$pid" =~ ^[0-9]+$ ]] || continue
    [[ "$pid" != "$$" && "$pid" != "$BASHPID" ]] || continue
    case "$args" in
      *"$ROOT_DIR/target/release/deps/test_hjmt_e2e-"*|*"cargo test -p z00z_simulator --release --test test_hjmt_e2e"*)
        if [[ -n "$RUN_ROOT" && "$args" == *"$RUN_ROOT/target/"* ]]; then
          continue
        fi
        printf '%s\n' "$pid"
        ;;
    esac
  done < <(ps -eo pid=,args=)
}

kill_external_interferers() {
  local -a pids=()
  local pid

  mapfile -t pids < <(external_interferer_pids | sort -u)
  [[ "${#pids[@]}" -gt 0 ]] || return 0

  for pid in "${pids[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
  sleep 1
  for pid in "${pids[@]}"; do
    kill -9 "$pid" 2>/dev/null || true
  done
  EXTERNAL_INTERFERERS_KILLED=$((EXTERNAL_INTERFERERS_KILLED + ${#pids[@]}))
}

snapshot_known_external_root_cache_activity() {
  local gate_id="$1"
  local root_cache="$ROOT_DIR/.cache"
  local dest_dir="$RUN_ROOT/interference/$gate_id"
  local manifest_path="$dest_dir/root-production-cache.tsv"

  [[ -d "$root_cache" ]] || return 1
  if ! find "$root_cache" -maxdepth 6 \( -type d -o -type f \) 2>/dev/null \
    | rg -q 'test_hjmt_e2e-pid-|scenario1_full_stage13_shared_v4'; then
    return 1
  fi

  mkdir -p "$dest_dir"
  if [[ -e "$manifest_path" ]]; then
    manifest_path="$dest_dir/root-production-cache-$RUN_TIMESTAMP.tsv"
  fi

  find "$root_cache" -maxdepth 6 -printf '%y\t%P\t%TY-%Tm-%TdT%TH:%TM:%TS\t%s\n' 2>/dev/null \
    | sort >"$manifest_path"
  printf '%s\n' "${manifest_path#"$ROOT_DIR"/}"
  return 0
}

root_cache_state_digest() {
  emit_root_cache_listing \
    | sha256sum \
    | awk '{print $1}'
}

emit_root_cache_listing() {
  local root_cache="$ROOT_DIR/.cache"

  [[ -d "$root_cache" ]] || {
    printf '%s\n' "absent"
    return 0
  }

  find "$root_cache" -maxdepth 8 -printf '%y\t%P\t%TY-%Tm-%TdT%TH:%TM:%TS\t%s\n' 2>/dev/null | sort
}

write_root_cache_manifest() {
  local gate_id="$1"
  local phase="$2"
  local snapshot_path="${3:-}"
  local dest_dir="$RUN_ROOT/interference/$gate_id"
  local manifest_path="$dest_dir/root-production-cache-$phase.tsv"

  mkdir -p "$dest_dir"
  if [[ -n "$snapshot_path" && -f "$snapshot_path" ]]; then
    cp "$snapshot_path" "$manifest_path"
  else
    emit_root_cache_listing >"$manifest_path"
  fi

  printf '%s\n' "${manifest_path#"$ROOT_DIR"/}"
}

run_gate() {
  local gate_id="$1"
  local label="$2"
  shift 2
  local start_ns end_ns started_at ended_at elapsed_ms gate_cwd leak_paths success_status gate_rc
  local resource_time_bin resource_profile_path
  local root_cache_before_digest root_cache_after_digest root_cache_before_manifest root_cache_after_manifest
  local root_cache_before_snapshot

  GATE_LABEL["$gate_id"]="$label"
  GATE_LOG["$gate_id"]="$(gate_log_path "$gate_id")"
  GATE_MODULE["$gate_id"]="$(gate_module_path "$gate_id")"
  GATE_ARTIFACTS["$gate_id"]="$(gate_artifact_paths "$gate_id" | paste -sd ',' -)"
  GATE_LEAKS["$gate_id"]=""

  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY'
    printf ' %q' "$@"
    printf '\n'
    GATE_STATUS["$gate_id"]="DRY-RUN"
    return 0
  fi

  gate_cwd="$(gate_runtime_cwd "$gate_id")"
  mkdir -p \
    "$RUN_ROOT" \
    "$gate_cwd" \
    "$REPORT_LOG_DIR" \
    "$RUN_ROOT/profiling" \
    "$PROFILE_RESOURCE_DIR" \
    "$TMP_ROOT"
  resource_time_bin="${Z00Z_RESOURCE_TIME_BIN:-/usr/bin/time}"
  resource_profile_path=""
  if [[ -x "$resource_time_bin" ]]; then
    resource_profile_path="$PROFILE_RESOURCE_DIR/$gate_id.time"
  fi
  sanitize_preexisting_root_runtime_leaks
  root_cache_before_snapshot="$TMP_ROOT/root-production-cache-$gate_id-before.tsv"
  emit_root_cache_listing >"$root_cache_before_snapshot"
  root_cache_before_digest="$(root_cache_state_digest)"
  start_ns="$(z00z_profile_now_ns)"
  started_at="$(z00z_profile_now_iso)"
  set +e
  if [[ -n "$resource_profile_path" ]]; then
    "$resource_time_bin" -v -o "$resource_profile_path" env Z00Z_RUNTIME_CWD="$gate_cwd" "$@" >"${GATE_LOG[$gate_id]}" 2>&1
  else
    env Z00Z_RUNTIME_CWD="$gate_cwd" "$@" >"${GATE_LOG[$gate_id]}" 2>&1
  fi
  gate_rc=$?
  set -e
  if [[ "$gate_rc" -eq 0 ]]; then
    success_status="$(success_status_from_log "${GATE_LOG[$gate_id]}")"
    kill_external_interferers
    root_cache_after_digest="$(root_cache_state_digest)"
    if noise_path="$(snapshot_known_external_root_cache_activity "$gate_id")"; then
      printf '[z00z-verification] NOTE: observed external production-cache activity; report-local manifest written: %s\n' "$noise_path" >>"${GATE_LOG[$gate_id]}"
    fi
    if [[ "$root_cache_before_digest" != "$root_cache_after_digest" ]]; then
      root_cache_before_manifest="$(write_root_cache_manifest "$gate_id" before "$root_cache_before_snapshot")"
      root_cache_after_manifest="$(write_root_cache_manifest "$gate_id" after)"
      printf '[z00z-verification] NOTE: repo/.cache changed during gate; manifests written: %s, %s. Treat this as production/dev cache activity, not verifier-owned output.\n' \
        "$root_cache_before_manifest" "$root_cache_after_manifest" >>"${GATE_LOG[$gate_id]}"
      if [[ -n "${GATE_ARTIFACTS[$gate_id]:-}" && "${GATE_ARTIFACTS[$gate_id]}" != "-" ]]; then
        GATE_ARTIFACTS["$gate_id"]+=",${root_cache_before_manifest},${root_cache_after_manifest}"
      else
        GATE_ARTIFACTS["$gate_id"]="${root_cache_before_manifest},${root_cache_after_manifest}"
      fi
    fi
    if leak_paths="$(relocate_root_runtime_leaks "$gate_id")"; then
      GATE_LEAKS["$gate_id"]="$leak_paths"
      printf '[z00z-verification] FAIL: relocated unauthorized runtime output(s) into report-local leaks/: %s\n' "$leak_paths" >>"${GATE_LOG[$gate_id]}"
      GATE_STATUS["$gate_id"]="FAIL"
    else
      GATE_STATUS["$gate_id"]="$success_status"
    fi
    end_ns="$(z00z_profile_now_ns)"
    ended_at="$(z00z_profile_now_iso)"
    elapsed_ms=$(((end_ns - start_ns) / 1000000))
    GATE_ELAPSED_SECS["$gate_id"]="$(printf '%d.%03d' "$((elapsed_ms / 1000))" "$((elapsed_ms % 1000))")"
    if [[ -n "$resource_profile_path" && -f "$resource_profile_path" ]]; then
      if [[ -n "${GATE_ARTIFACTS[$gate_id]:-}" && "${GATE_ARTIFACTS[$gate_id]}" != "-" ]]; then
        GATE_ARTIFACTS["$gate_id"]+=",${resource_profile_path#"$ROOT_DIR"/}"
      else
        GATE_ARTIFACTS["$gate_id"]="${resource_profile_path#"$ROOT_DIR"/}"
      fi
    fi
    z00z_profile_record_event "gate" "$gate_id" "${GATE_STATUS[$gate_id]}" "$start_ns" "$end_ns" "$started_at" "$ended_at" "$(z00z_profile_join_command "$@")"
    return 0
  fi

  GATE_STATUS["$gate_id"]="FAIL"
  kill_external_interferers
  root_cache_after_digest="$(root_cache_state_digest)"
  if noise_path="$(snapshot_known_external_root_cache_activity "$gate_id")"; then
    printf '[z00z-verification] NOTE: observed external production-cache activity; report-local manifest written: %s\n' "$noise_path" >>"${GATE_LOG[$gate_id]}"
  fi
  if [[ "$root_cache_before_digest" != "$root_cache_after_digest" ]]; then
    root_cache_before_manifest="$(write_root_cache_manifest "$gate_id" before "$root_cache_before_snapshot")"
    root_cache_after_manifest="$(write_root_cache_manifest "$gate_id" after)"
    printf '[z00z-verification] NOTE: repo/.cache changed during failed gate; manifests written: %s, %s. Treat this as production/dev cache activity, not verifier-owned output.\n' \
      "$root_cache_before_manifest" "$root_cache_after_manifest" >>"${GATE_LOG[$gate_id]}"
    if [[ -n "${GATE_ARTIFACTS[$gate_id]:-}" && "${GATE_ARTIFACTS[$gate_id]}" != "-" ]]; then
      GATE_ARTIFACTS["$gate_id"]+=",${root_cache_before_manifest},${root_cache_after_manifest}"
    else
      GATE_ARTIFACTS["$gate_id"]="${root_cache_before_manifest},${root_cache_after_manifest}"
    fi
  fi
  if leak_paths="$(relocate_root_runtime_leaks "$gate_id")"; then
    GATE_LEAKS["$gate_id"]="$leak_paths"
    printf '[z00z-verification] NOTE: relocated unauthorized root runtime output(s) after failure: %s\n' "$leak_paths" >>"${GATE_LOG[$gate_id]}"
  fi
  end_ns="$(z00z_profile_now_ns)"
  ended_at="$(z00z_profile_now_iso)"
  elapsed_ms=$(((end_ns - start_ns) / 1000000))
  GATE_ELAPSED_SECS["$gate_id"]="$(printf '%d.%03d' "$((elapsed_ms / 1000))" "$((elapsed_ms % 1000))")"
  if [[ -n "$resource_profile_path" && -f "$resource_profile_path" ]]; then
    if [[ -n "${GATE_ARTIFACTS[$gate_id]:-}" && "${GATE_ARTIFACTS[$gate_id]}" != "-" ]]; then
      GATE_ARTIFACTS["$gate_id"]+=",${resource_profile_path#"$ROOT_DIR"/}"
    else
      GATE_ARTIFACTS["$gate_id"]="${resource_profile_path#"$ROOT_DIR"/}"
    fi
  fi
  z00z_profile_record_event "gate" "$gate_id" "${GATE_STATUS[$gate_id]}" "$start_ns" "$end_ns" "$started_at" "$ended_at" "$(z00z_profile_join_command "$@")"
  return 1
}

feature_args() {
  if [[ -n "$FEATURE_FLAG" ]]; then
    printf '%s\n' "$FEATURE_FLAG"
  fi
}

has_verus_targets() {
  find "$VERIFICATION_RUNTIME_ROOT/verus" -type f -name '*.rs' 2>/dev/null | grep -q .
}

has_prusti_targets() {
  find "$VERIFICATION_RUNTIME_ROOT/prusti" -type f -name '*.rs' 2>/dev/null | grep -q .
}

has_loom_targets() {
  rg -q "loom::|cfg\\(loom\\)|cfg\\([^)]*loom" "$ROOT_DIR/crates" "$ROOT_DIR/tests" 2>/dev/null
}

has_hax_targets() {
  [[ -f "$VERIFICATION_RUNTIME_ROOT/hax/targets.json" ]]
}

has_code_to_logic_targets() {
  [[ -f "$VERIFICATION_RUNTIME_ROOT/code-to-logic/targets.yaml" ]]
}

resolved_verus_root() {
  printf '%s\n' "$VERIFICATION_RUNTIME_ROOT/verus"
}

resolved_hax_targets() {
  printf '%s\n' "$VERIFICATION_RUNTIME_ROOT/hax/targets.json"
}

resolved_code_to_logic_targets() {
  printf '%s\n' "$VERIFICATION_RUNTIME_ROOT/code-to-logic/targets.yaml"
}

resolved_code_to_logic_llbc_dir() {
  printf '%s\n' "$VERIFICATION_RUNTIME_ROOT/code-to-logic/llbc"
}

resolved_code_to_logic_aeneas_dir() {
  printf '%s\n' "$VERIFICATION_RUNTIME_ROOT/code-to-logic/aeneas"
}

resolved_prusti_root() {
  printf '%s\n' "$VERIFICATION_RUNTIME_ROOT/prusti"
}

resolved_specs_root() {
  printf '%s\n' "$SPECS_RUNTIME_ROOT"
}

resolved_dudect_root() {
  printf '%s\n' "$VERIFICATION_RUNTIME_ROOT/dudect"
}

resolved_fuzz_dir() {
  printf '%s\n' "$FUZZ_RUNTIME_ROOT"
}

scope_needs_verus() {
  if [[ "$SCOPE_KIND" == "project" ]]; then
    return 0
  fi

  case "$TARGET_ROOT_REL" in
    crates/z00z_core|crates/z00z_storage|crates/z00z_runtime|crates/z00z_runtime/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

scope_needs_hax() {
  if [[ "$SCOPE_KIND" == "project" ]]; then
    return 0
  fi

  case "$TARGET_ROOT_REL" in
    crates/z00z_crypto|crates/z00z_crypto/*|crates/z00z_wallets|crates/z00z_wallets/*|crates/z00z_storage|crates/z00z_storage/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

scope_needs_code_to_logic() {
  if [[ "$SCOPE_KIND" == "project" ]]; then
    return 0
  fi

  case "$TARGET_ROOT_REL" in
    crates/z00z_core|crates/z00z_core/*|crates/z00z_crypto|crates/z00z_crypto/*|crates/z00z_storage|crates/z00z_storage/*|crates/z00z_wallets|crates/z00z_wallets/*|crates/z00z_runtime/validators|crates/z00z_runtime/validators/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

scope_needs_constant_time() {
  if [[ "$SCOPE_KIND" == "project" ]]; then
    return 0
  fi

  case "$TARGET_ROOT_REL" in
    crates/z00z_wallets|crates/z00z_wallets/*|crates/z00z_crypto|crates/z00z_crypto/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

run_l3_project_gate() {
  local gate_failed=0

  run_gate l3-verify-fast "L3 Rust implementation gate" \
    "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh" || gate_failed=1
  run_gate l3-miri "L3 Miri gate" \
    "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-miri.sh" || gate_failed=1
  run_gate l3-kani "L3 Kani gate" \
    "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh" || gate_failed=1
  if has_loom_targets; then
    run_gate l3-loom "L3 Loom gate" \
      "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-loom.sh" || gate_failed=1
  fi
  if has_verus_targets; then
    run_gate l3-verus "L3 Verus gate" \
      env Z00Z_VERUS_ROOT="$(resolved_verus_root)" \
      "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh" || gate_failed=1
  fi
  if has_prusti_targets || [[ -n "${Z00Z_PRUSTI_PACKAGES:-}" ]]; then
    run_gate l3-prusti "L3 Prusti gate" \
      env Z00Z_PRUSTI_ROOT="$(resolved_prusti_root)" \
      "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-prusti.sh" || gate_failed=1
  fi

  return "$gate_failed"
}

run_l3_crate_gate() {
  local gate_failed=0
  local features=()
  local profile_args=(--release)
  local kani_timeout="${Z00Z_KANI_TIMEOUT_SECS:-0}"
  local nextest_mode="${Z00Z_L3_USE_NEXTEST:-auto}"
  local feature

  while IFS= read -r feature; do
    [[ -n "$feature" ]] || continue
    features+=("$feature")
  done < <(feature_args)

  run_gate l3-fmt "L3 crate cargo fmt" cargo fmt --manifest-path "$TARGET_MANIFEST_ABS" --check || gate_failed=1
  run_gate l3-clippy "L3 crate cargo clippy" cargo clippy -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" --all-targets "${features[@]}" -- -D warnings || gate_failed=1

  if [[ -n "${Z00Z_VERIFICATION_RUN_ROOT:-}" && "$nextest_mode" == "auto" ]]; then
    run_gate l3-test "L3 crate cargo test" cargo test -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" "${features[@]}" || gate_failed=1
    run_gate l3-test-ignored "L3 crate cargo test ignored-only" cargo test -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" "${features[@]}" -- --ignored || gate_failed=1
  elif have cargo-nextest || cargo nextest --version >/dev/null 2>&1; then
    run_gate l3-nextest "L3 crate cargo nextest" cargo nextest run -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" "${features[@]}" || gate_failed=1
    run_gate l3-nextest-ignored "L3 crate cargo nextest ignored-only" cargo nextest run -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" "${features[@]}" --run-ignored ignored-only || gate_failed=1
  else
    run_gate l3-test "L3 crate cargo test" cargo test -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" "${features[@]}" || gate_failed=1
    run_gate l3-test-ignored "L3 crate cargo test ignored-only" cargo test -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" "${features[@]}" -- --ignored || gate_failed=1
  fi

  run_gate l3-doc "L3 crate cargo doc tests" cargo test -p "$TARGET_PACKAGE_NAME" "${profile_args[@]}" "${features[@]}" --doc || gate_failed=1
  run_gate l3-miri "L3 crate Miri gate" \
    env Z00Z_MIRI_PACKAGES="$TARGET_PACKAGE_NAME" \
    "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-miri.sh" || gate_failed=1
  if [[ "$TARGET_PACKAGE_NAME" == "z00z_validators" ]]; then
    kani_timeout="${Z00Z_VALIDATOR_KANI_TIMEOUT_SECS:-0}"
  fi
  run_gate l3-kani "L3 crate Kani gate" \
    env Z00Z_KANI_PACKAGES="$TARGET_PACKAGE_NAME" Z00Z_KANI_TIMEOUT_SECS="$kani_timeout" \
    "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh" || gate_failed=1
  if has_verus_targets && scope_needs_verus; then
    run_gate l3-verus "L3 crate Verus gate" \
      env Z00Z_VERUS_ROOT="$(resolved_verus_root)" \
      "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh" || gate_failed=1
  fi
  if has_prusti_targets || [[ -n "${Z00Z_PRUSTI_PACKAGES:-}" ]]; then
    run_gate l3-prusti "L3 crate Prusti gate" \
      env Z00Z_PRUSTI_ROOT="$(resolved_prusti_root)" \
      "$ROOT_DIR/.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-prusti.sh" || gate_failed=1
  fi
  return "$gate_failed"
}

run_l4_gate() {
  local gate_failed=0
  local vendor_report_path="$RUN_ROOT/vendor/vendor-unsafe.md"
  local supply_chain_prefix="$RUN_ROOT/supply-chain/supply-chain"
  local geiger_target_root="$RUN_ROOT/geiger/target"
  local fuzz_target_dir="$FUZZ_RUNTIME_ROOT/target"
  local fuzz_corpus_root="$FUZZ_RUNTIME_ROOT/corpus"
  local fuzz_artifact_root="$FUZZ_RUNTIME_ROOT/artifacts"

  run_gate l4-supply-chain "L4 supply-chain gate" \
    env Z00Z_SUPPLY_CHAIN_REPORT_PREFIX="$supply_chain_prefix" \
    "$ROOT_DIR/.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh" || gate_failed=1

  if [[ "$TARGET_TOUCHES_VENDOR" -eq 1 ]]; then
    run_gate l4-vendor-unsafe "L4 vendor unsafe report" \
      env Z00Z_VENDOR_ROOT="$VENDOR_ROOT_REL" Z00Z_VENDOR_UNSAFE_REPORT="$vendor_report_path" Z00Z_GEIGER_TARGET_ROOT="$geiger_target_root" \
      "$ROOT_DIR/.github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh" --vendor-report-only "$vendor_report_path" || gate_failed=1
  elif [[ "$SCOPE_KIND" == "project" || "${SELECTED_LEVELS[l2]:-0}" -eq 1 ]]; then
    run_gate l4-unsafe "L4 unsafe scan and vendor report" \
      env Z00Z_VENDOR_ROOT="$VENDOR_ROOT_REL" Z00Z_VENDOR_UNSAFE_REPORT="$vendor_report_path" Z00Z_GEIGER_TARGET_ROOT="$geiger_target_root" \
      "$ROOT_DIR/.github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh" --all || gate_failed=1
  else
    run_gate l4-unsafe "L4 unsafe scan" \
      env Z00Z_GEIGER_TARGET_ROOT="$geiger_target_root" \
      "$ROOT_DIR/.github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh" --project-only || gate_failed=1
  fi

  if [[ "$SCOPE_KIND" == "project" || "$TARGET_ROOT_REL" == crates/z00z_wallets* || "$TARGET_ROOT_REL" == crates/z00z_storage* || "$TARGET_ROOT_REL" == crates/z00z_core* ]]; then
    run_gate l4-fuzz "L4 short fuzz gate" \
      env Z00Z_FUZZ_DIR="$(resolved_fuzz_dir)" Z00Z_FUZZ_TARGET_DIR="$fuzz_target_dir" Z00Z_FUZZ_CORPUS_ROOT="$fuzz_corpus_root" Z00Z_FUZZ_ARTIFACT_ROOT="$fuzz_artifact_root" \
      "$ROOT_DIR/.github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-short.sh" || gate_failed=1
  fi

  if scope_needs_constant_time; then
    run_gate l4-constant-time "L4 constant-time gate" \
      env Z00Z_DUDECT_ROOT="$(resolved_dudect_root)" \
      "$ROOT_DIR/.github/skills/z00z-l4-security-engineering-gate/scripts/run-constant-time.sh" || gate_failed=1
  fi

  run_gate l4-adversarial-review "L4 adversarial security brainstorming" \
    python3 "$SCRIPT_DIR/run-security-brainstorm.py" \
      --root "$ROOT_DIR" \
      --scope-kind "$SCOPE_KIND" \
      --target-root-rel "${TARGET_ROOT_REL:-}" \
      --verification-root "$VERIFICATION_RUNTIME_ROOT" \
      --summary-out "$SECURITY_BRAINSTORM_SUMMARY_PATH" \
      --report-out "$SECURITY_BRAINSTORM_REPORT_PATH" || gate_failed=1

  return "$gate_failed"
}

execute_selected_gates() {
  local level
  local failed=0

  for gate_id in "${!GATE_STATUS[@]}"; do
    unset "GATE_STATUS[$gate_id]"
  done
  for gate_id in "${!GATE_LOG[@]}"; do
    unset "GATE_LOG[$gate_id]"
  done
  for gate_id in "${!GATE_LABEL[@]}"; do
    unset "GATE_LABEL[$gate_id]"
  done
  for gate_id in "${!GATE_ELAPSED_SECS[@]}"; do
    unset "GATE_ELAPSED_SECS[$gate_id]"
  done
  for gate_id in "${!GATE_MODULE[@]}"; do
    unset "GATE_MODULE[$gate_id]"
  done
  for gate_id in "${!GATE_ARTIFACTS[@]}"; do
    unset "GATE_ARTIFACTS[$gate_id]"
  done
  for gate_id in "${!GATE_LEAKS[@]}"; do
    unset "GATE_LEAKS[$gate_id]"
  done

  for level in l0 l1 l2 l3 l4; do
    [[ "${SELECTED_LEVELS[$level]:-0}" -eq 1 ]] || continue
    case "$level" in
      l0)
        run_gate l0-docs "L0 documentation gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" "$ROOT_DIR/.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh" || failed=1
        ;;
      l1)
        run_gate l1-tla "L1 TLA+ gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" Z00Z_REPORT_TIMESTAMP="$RUN_TIMESTAMP" Z00Z_TLA_STATE_ROOT="$VERIFICATION_RUNTIME_ROOT/l1/tla-states" Z00Z_TLA_USER_OUTPUT_ROOT="$VERIFICATION_RUNTIME_ROOT/l1/tla-user" "$ROOT_DIR/.github/skills/z00z-l1-protocol-model-gate/scripts/run-tla.sh" || failed=1
        run_gate l1-apalache "L1 Apalache gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" Z00Z_REPORT_TIMESTAMP="$RUN_TIMESTAMP" Z00Z_APALACHE_OUT_DIR="$VERIFICATION_RUNTIME_ROOT/l1/apalache" Z00Z_APALACHE_RUN_DIR="$VERIFICATION_RUNTIME_ROOT/l1/apalache/runs" "$ROOT_DIR/.github/skills/z00z-l1-protocol-model-gate/scripts/run-apalache.sh" || failed=1
        run_gate l1-alloy "L1 Alloy gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" Z00Z_REPORT_TIMESTAMP="$RUN_TIMESTAMP" Z00Z_ALLOY_OUT_DIR="$VERIFICATION_RUNTIME_ROOT/l1/alloy" "$ROOT_DIR/.github/skills/z00z-l1-protocol-model-gate/scripts/run-alloy.sh" || failed=1
        ;;
      l2)
        run_gate l2-domain "L2 domain separation gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" python3 "$ROOT_DIR/.github/skills/z00z-l2-crypto-protocol-gate/scripts/check-domain-separation.py" || failed=1
        run_gate l2-transcript "L2 transcript binding gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" python3 "$ROOT_DIR/.github/skills/z00z-l2-crypto-protocol-gate/scripts/check-transcript-binding.py" || failed=1
        run_gate l2-proverif "L2 ProVerif gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" "$ROOT_DIR/.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-proverif.sh" || failed=1
        run_gate l2-tamarin "L2 Tamarin gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" Z00Z_REPORT_TIMESTAMP="$RUN_TIMESTAMP" Z00Z_TAMARIN_TMPDIR="$TMP_ROOT/tamarin" "$ROOT_DIR/.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-tamarin.sh" || failed=1
        if scope_needs_hax || scope_needs_code_to_logic; then
          refresh_runtime_artifacts || true
        fi
        if has_hax_targets && scope_needs_hax; then
          run_gate l2-hax "L2 HAX/EasyCrypt extraction gate" env Z00Z_SPECS_ROOT="$(resolved_specs_root)" Z00Z_REPORT_TIMESTAMP="$RUN_TIMESTAMP" Z00Z_HAX_TARGETS="$(resolved_hax_targets)" Z00Z_HAX_OUTPUT_ROOT="$VERIFICATION_RUNTIME_ROOT/l2/hax" Z00Z_HAX_TMPDIR="$VERIFICATION_RUNTIME_ROOT/hax/tmp" "$ROOT_DIR/.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-hax.sh" || failed=1
        fi
        if scope_needs_code_to_logic; then
          refresh_runtime_artifacts || true
          run_gate l2-refinement-map "L2 code-to-logic refinement map gate" \
            env Z00Z_VERIFICATION_RUNTIME_ROOT="$VERIFICATION_RUNTIME_ROOT" \
            python3 "$CODE_TO_LOGIC_SKILL_DIR/scripts/check-refinement-map.py" --targets "$(resolved_code_to_logic_targets)" || failed=1
          run_gate l2-cryptol "L2 Cryptol code-to-logic gate" \
            env Z00Z_SPECS_ROOT="$(resolved_specs_root)" Z00Z_VERIFICATION_RUNTIME_ROOT="$VERIFICATION_RUNTIME_ROOT" \
            "$CODE_TO_LOGIC_SKILL_DIR/scripts/run-cryptol.sh" || failed=1
          run_gate l2-saw "L2 SAW code-to-logic gate" \
            env Z00Z_SPECS_ROOT="$(resolved_specs_root)" Z00Z_VERIFICATION_RUNTIME_ROOT="$VERIFICATION_RUNTIME_ROOT" \
            "$CODE_TO_LOGIC_SKILL_DIR/scripts/run-saw.sh" || failed=1
          if has_code_to_logic_targets; then
            run_gate l2-crux-mir "L2 Crux-MIR code-to-logic gate" \
              env Z00Z_CODE_TO_LOGIC_TARGETS="$(resolved_code_to_logic_targets)" Z00Z_VERIFICATION_RUNTIME_ROOT="$VERIFICATION_RUNTIME_ROOT" \
              "$CODE_TO_LOGIC_SKILL_DIR/scripts/run-crux-mir.sh" || failed=1
            run_gate l2-charon "L2 Charon extraction gate" \
              env Z00Z_CODE_TO_LOGIC_TARGETS="$(resolved_code_to_logic_targets)" Z00Z_VERIFICATION_RUNTIME_ROOT="$VERIFICATION_RUNTIME_ROOT" Z00Z_CODE_TO_LOGIC_CHARON_OUT="$(resolved_code_to_logic_llbc_dir)" \
              "$CODE_TO_LOGIC_SKILL_DIR/scripts/run-charon.sh" || failed=1
            run_gate l2-aeneas "L2 Aeneas translation gate" \
              env Z00Z_VERIFICATION_RUNTIME_ROOT="$VERIFICATION_RUNTIME_ROOT" Z00Z_CODE_TO_LOGIC_CHARON_OUT="$(resolved_code_to_logic_llbc_dir)" Z00Z_CODE_TO_LOGIC_AENEAS_OUT="$(resolved_code_to_logic_aeneas_dir)" \
              "$CODE_TO_LOGIC_SKILL_DIR/scripts/run-aeneas.sh" || failed=1
          fi
        fi
        ;;
      l3)
        if [[ "$SCOPE_KIND" == "crate" ]]; then
          run_l3_crate_gate || failed=1
        else
          run_l3_project_gate || failed=1
        fi
        ;;
      l4)
        run_l4_gate || failed=1
        ;;
    esac
  done

  return "$failed"
}

apply_mechanical_fixes() {
  [[ "$MODE" != "report" ]] || return 0
  [[ "${SELECTED_LEVELS[l3]:-0}" -eq 1 ]] || return 0

  if [[ "$SCOPE_KIND" == "crate" ]]; then
    log "mechanical fix: cargo fmt for $TARGET_PACKAGE_NAME"
    run_cmd cargo fmt --manifest-path "$TARGET_MANIFEST_ABS"
  else
    log "mechanical fix: cargo fmt --all"
    run_cmd cargo fmt --all
  fi

  return 0
}

bootstrap_missing_artifacts() {
  [[ "$MODE" != "report" ]] || return 0

  if [[ ! -f "$ARTIFACT_BUILDER_SCRIPT" ]]; then
    warn "artifact builder script not found: $ARTIFACT_BUILDER_SCRIPT"
    return 0
  fi

  local args=(
    python3
    "$ARTIFACT_BUILDER_SCRIPT"
    --root "$ROOT_DIR"
    --scope-kind "$SCOPE_KIND"
    --target-root-rel "${TARGET_ROOT_REL:-}"
    --specs-runtime-root "$SPECS_RUNTIME_ROOT"
    --verification-runtime-root "$VERIFICATION_RUNTIME_ROOT"
    --fuzz-runtime-root "$FUZZ_RUNTIME_ROOT"
    --runtime-only
    --summary-out "$BOOTSTRAP_SUMMARY_PATH"
  )

  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY'
    printf ' %q' "${args[@]}"
    printf '\n'
    return 0
  fi

  "${args[@]}"
  local summary
  summary="$(python3 - "$BOOTSTRAP_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(len(data.get("created", [])))
print(len(data.get("updated", [])))
print(len(data.get("skipped", [])))
PY
)"

  mapfile -t bootstrap_lines <<< "$summary"
  BOOTSTRAP_CREATED_COUNT="${bootstrap_lines[0]:-0}"
  BOOTSTRAP_UPDATED_COUNT="${bootstrap_lines[1]:-0}"
  BOOTSTRAP_SKIPPED_COUNT="${bootstrap_lines[2]:-0}"
}

prepare_runtime_artifacts() {
  [[ "$MODE" == "report" ]] || return 0
  [[ -f "$ARTIFACT_BUILDER_SCRIPT" ]] || return 0

  local args=(
    python3
    "$ARTIFACT_BUILDER_SCRIPT"
    --root "$ROOT_DIR"
    --scope-kind "$SCOPE_KIND"
    --target-root-rel "${TARGET_ROOT_REL:-}"
    --specs-runtime-root "$SPECS_RUNTIME_ROOT"
    --verification-runtime-root "$VERIFICATION_RUNTIME_ROOT"
    --fuzz-runtime-root "$FUZZ_RUNTIME_ROOT"
    --runtime-only
    --summary-out "$RUNTIME_BOOTSTRAP_SUMMARY_PATH"
  )

  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY'
    printf ' %q' "${args[@]}"
    printf '\n'
    return 0
  fi

  "${args[@]}" >/dev/null
}

refresh_runtime_artifacts() {
  [[ -f "$ARTIFACT_BUILDER_SCRIPT" ]] || return 0

  local summary_path="$RUNTIME_BOOTSTRAP_SUMMARY_PATH"
  if [[ "$MODE" != "report" ]]; then
    summary_path="$BOOTSTRAP_SUMMARY_PATH"
  fi

  local args=(
    python3
    "$ARTIFACT_BUILDER_SCRIPT"
    --root "$ROOT_DIR"
    --scope-kind "$SCOPE_KIND"
    --target-root-rel "${TARGET_ROOT_REL:-}"
    --specs-runtime-root "$SPECS_RUNTIME_ROOT"
    --verification-runtime-root "$VERIFICATION_RUNTIME_ROOT"
    --fuzz-runtime-root "$FUZZ_RUNTIME_ROOT"
    --runtime-only
    --summary-out "$summary_path"
  )

  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY'
    printf ' %q' "${args[@]}"
    printf '\n'
    return 0
  fi

  "${args[@]}" >/dev/null
}

vendor_findings_count() {
  local vendor_report_path="$RUN_ROOT/vendor/vendor-unsafe.md"

  [[ -f "$vendor_report_path" ]] || {
    printf '0\n'
    return 0
  }

  python3 - "$vendor_report_path" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
count = 0
for line in path.read_text(encoding="utf-8").splitlines():
    match = re.search(r"Rust unsafe facts found: `([0-9]+)`", line)
    if match:
        count = int(match.group(1))
        break
print(count)
PY
}

build_coverage_manifest() {
  [[ "$SCOPE_KIND" == "project" ]] || return 0

  COVERAGE_MANIFEST_PATH="$RUN_ROOT/coverage/manifest.tsv"
  COVERAGE_SUMMARY_PATH="$RUN_ROOT/coverage/summary.json"

  local args=(
    python3
    "$SCRIPT_DIR/build-coverage-manifest.py"
    --root "$ROOT_DIR"
    --vendor-root "$ROOT_DIR/$VENDOR_ROOT_REL"
    --manifest-out "$COVERAGE_MANIFEST_PATH"
    --summary-out "$COVERAGE_SUMMARY_PATH"
    --verification-runtime-root "$VERIFICATION_RUNTIME_ROOT"
  )
  local gate_id
  for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
    args+=(--gate-status "$gate_id=${GATE_STATUS[$gate_id]}")
  done

  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY'
    printf ' %q' "${args[@]}"
    printf '\n'
    return 0
  fi

  "${args[@]}"

  local summary
  summary="$(python3 - "$COVERAGE_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
counts = data.get("status_counts", {})
crate_counts = data.get("crate_status_counts", {})
print(data.get("tracked_files", 0))
print(counts.get("FAIL", 0))
print(counts.get("SKIPPED", 0))
print(counts.get("UNKNOWN", 0))
print(counts.get("UNMAPPED", 0))
print(crate_counts.get("UNMAPPED", 0))
PY
)"

  mapfile -t summary_lines <<< "$summary"
  COVERAGE_TRACKED_FILES="${summary_lines[0]:-0}"
  COVERAGE_FAIL_COUNT="${summary_lines[1]:-0}"
  COVERAGE_SKIPPED_COUNT="${summary_lines[2]:-0}"
  COVERAGE_UNKNOWN_COUNT="${summary_lines[3]:-0}"
  COVERAGE_UNMAPPED_COUNT="${summary_lines[4]:-0}"
  COVERAGE_CRATE_UNMAPPED_COUNT="${summary_lines[5]:-0}"
}

collect_profiler_tool_inventory() {
  [[ "$DRY_RUN" -eq 0 ]] || return 0
  python3 "$TOOL_INVENTORY_SCRIPT" \
    --summary-out "$PROFILE_TOOL_SUMMARY_PATH"
}

build_resource_profile_summary() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    return 0
  fi

  python3 "$RESOURCE_SUMMARY_SCRIPT" \
    --profiles-dir "$PROFILE_RESOURCE_DIR" \
    --summary-out "$PROFILE_RESOURCE_SUMMARY_PATH"
}

build_run_footprint_summary() {
  [[ "$DRY_RUN" -eq 0 ]] || return 0

  python3 "$RUN_FOOTPRINT_SCRIPT" \
    --run-root "$RUN_ROOT" \
    --summary-out "$RUN_FOOTPRINT_SUMMARY_PATH"
}

build_hjmt_summary() {
  [[ "$DRY_RUN" -eq 0 ]] || return 0

  python3 "$HJMT_SUMMARY_SCRIPT" \
    --run-root "$RUN_ROOT" \
    --summary-out "$HJMT_SUMMARY_PATH"
}

build_profiling_summary() {
  PROFILE_EVENT_COUNT=0
  PROFILE_GATE_EVENT_COUNT=0
  PROFILE_COMMAND_EVENT_COUNT=0
  PROFILE_TOP_N=0

  if [[ "$DRY_RUN" -eq 1 || ! -f "$PROFILE_EVENTS_PATH" ]]; then
    return 0
  fi

  python3 "$PROFILE_SUMMARY_SCRIPT" \
    --events "$PROFILE_EVENTS_PATH" \
    --summary-out "$PROFILE_SUMMARY_PATH"

  local summary
  summary="$(python3 - "$PROFILE_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(data.get("event_count", 0))
print(data.get("gate_event_count", 0))
print(data.get("command_event_count", 0))
print(data.get("top_n", 0))
PY
)"

  mapfile -t profile_lines <<< "$summary"
  PROFILE_EVENT_COUNT="${profile_lines[0]:-0}"
  PROFILE_GATE_EVENT_COUNT="${profile_lines[1]:-0}"
  PROFILE_COMMAND_EVENT_COUNT="${profile_lines[2]:-0}"
  PROFILE_TOP_N="${profile_lines[3]:-0}"

  build_resource_profile_summary
  build_run_footprint_summary
  build_hjmt_summary
}

compute_overall_status() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    OVERALL_STATUS="DRY-RUN"
    return 0
  fi

  OVERALL_STATUS="PASS"

  local gate_id status
  for gate_id in "${!GATE_STATUS[@]}"; do
    status="${GATE_STATUS[$gate_id]}"
    if [[ "$status" == "FAIL" ]]; then
      OVERALL_STATUS="FAIL"
      return 0
    fi
    if [[ "$status" == "NEEDS_HUMAN_CRYPTO_REVIEW" ]]; then
      OVERALL_STATUS="NEEDS_HUMAN_CRYPTO_REVIEW"
      continue
    fi
    if [[ "$status" == "UNKNOWN" ]]; then
      if [[ "$OVERALL_STATUS" != "NEEDS_HUMAN_CRYPTO_REVIEW" ]]; then
        OVERALL_STATUS="UNKNOWN"
      fi
      continue
    fi
    if [[ "$status" == "SKIPPED" && "$OVERALL_STATUS" == "PASS" ]]; then
      OVERALL_STATUS="SKIPPED"
    fi
  done

  if [[ "$OVERALL_STATUS" != "FAIL" && "$SCOPE_KIND" == "project" ]]; then
    if [[ "$COVERAGE_FAIL_COUNT" -gt 0 ]]; then
      OVERALL_STATUS="FAIL"
      return 0
    fi
    if [[ "$COVERAGE_CRATE_UNMAPPED_COUNT" -gt 0 ]]; then
      OVERALL_STATUS="FAIL"
      return 0
    fi
    if [[ "$COVERAGE_UNKNOWN_COUNT" -gt 0 ]]; then
      OVERALL_STATUS="UNKNOWN"
    elif [[ "$COVERAGE_SKIPPED_COUNT" -gt 0 && "$OVERALL_STATUS" == "PASS" ]]; then
      OVERALL_STATUS="SKIPPED"
    fi
  fi
}

status_validity_ceiling() {
  local status="$1"
  case "$status" in
    PASS) printf '%s\n' "checker ran successfully but did not emit a stronger proof-grade classification" ;;
    TESTED) printf '%s\n' "runtime or executable check passed for the configured artifact; this is not a proof" ;;
    BOUNDED_VERIFIED) printf '%s\n' "bounded symbolic/model search completed successfully for the configured harness bounds" ;;
    MODEL_CHECKED) printf '%s\n' "model checker found no counterexample in the configured abstract model and scope" ;;
    FORMALLY_PROVED) printf '%s\n' "proof-oriented checker discharged the configured artifact, not an unstated larger surface" ;;
    SECURITY_PROTOCOL_PROVED) printf '%s\n' "symbolic protocol proof completed for the configured model and claims" ;;
    UNKNOWN) printf '%s\n' "tool, model, harness, or semantic closure is missing, so no stronger conclusion is valid" ;;
    SKIPPED) printf '%s\n' "gate did not run for the selected scope, so it contributes no positive evidence" ;;
    NEEDS_HUMAN_CRYPTO_REVIEW) printf '%s\n' "machine heuristics found risk hypotheses that remain unproven and require expert cryptographic review" ;;
    FAIL) printf '%s\n' "checker failed or the artifact confinement contract was violated, so the claimed property is not established" ;;
    DRY-RUN) printf '%s\n' "planned commands were rendered only; no evidence was executed" ;;
    *) printf '%s\n' "status is not classified; inspect the log directly before drawing conclusions" ;;
  esac
}

write_report() {
  local tmp_file vendor_count level_list gate_id status generated_at vendor_report_path
  local leak_gate_count fail_gate_count unknown_gate_count human_review_gate_count skipped_gate_count nonblocking_gate_count
  local gate_artifacts doublecheck_coverage_input doublecheck_bootstrap_input doublecheck_security_input
  local root_cache_manifest_count
  if [[ "$DRY_RUN" -eq 0 ]]; then
    tmp_file="$(mktemp "$TMP_ROOT/report.XXXXXX")"
  else
    tmp_file="$(mktemp "${TMPDIR:-/tmp}/z00z-report.XXXXXX")"
  fi
  generated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  level_list="$(levels_csv)"
  vendor_count="$(vendor_findings_count)"
  vendor_report_path="$RUN_ROOT/vendor/vendor-unsafe.md"
  doublecheck_coverage_input="-"
  doublecheck_bootstrap_input="-"
  doublecheck_security_input="-"
  [[ -n "$COVERAGE_SUMMARY_PATH" ]] && doublecheck_coverage_input="${COVERAGE_SUMMARY_PATH#"$ROOT_DIR"/}"
  [[ -n "$RUNTIME_BOOTSTRAP_SUMMARY_PATH" ]] && doublecheck_bootstrap_input="${RUNTIME_BOOTSTRAP_SUMMARY_PATH#"$ROOT_DIR"/}"
  [[ -n "$SECURITY_BRAINSTORM_SUMMARY_PATH" ]] && doublecheck_security_input="${SECURITY_BRAINSTORM_SUMMARY_PATH#"$ROOT_DIR"/}"
  if [[ -d "$RUN_ROOT/interference" ]]; then
    root_cache_manifest_count="$(find "$RUN_ROOT/interference" -type f -name 'root-production-cache-*.tsv' | wc -l | tr -d ' ')"
  else
    root_cache_manifest_count=0
  fi
  collect_profiler_tool_inventory
  build_profiling_summary
  build_coverage_manifest
  compute_overall_status
  leak_gate_count=0
  fail_gate_count=0
  unknown_gate_count=0
  human_review_gate_count=0
  skipped_gate_count=0
  for gate_id in "${!GATE_STATUS[@]}"; do
    status="${GATE_STATUS[$gate_id]}"
    [[ -n "${GATE_LEAKS[$gate_id]:-}" ]] && leak_gate_count=$((leak_gate_count + 1))
    [[ "$status" == "FAIL" ]] && fail_gate_count=$((fail_gate_count + 1))
    [[ "$status" == "UNKNOWN" ]] && unknown_gate_count=$((unknown_gate_count + 1))
    [[ "$status" == "NEEDS_HUMAN_CRYPTO_REVIEW" ]] && human_review_gate_count=$((human_review_gate_count + 1))
    [[ "$status" == "SKIPPED" ]] && skipped_gate_count=$((skipped_gate_count + 1))
  done
  nonblocking_gate_count=$(( ${#GATE_STATUS[@]} - fail_gate_count - unknown_gate_count - human_review_gate_count - skipped_gate_count ))

  {
    printf '<!-- z00z-orchestrator-report\n'
    printf 'scope=%s\n' "$SCOPE_KIND"
    printf 'target=%s\n' "$TARGET_LABEL"
    printf 'levels=%s\n' "$level_list"
    printf 'mode=%s\n' "$MODE"
    printf 'format=%s\n' "${REPORT_FORMAT_PATH#"$ROOT_DIR"/}"
    printf -- '-->\n'
    printf '# Z00Z Verification Orchestrator Report\n\n'
    printf '## 🎯 Executive Verdict\n\n'
    printf -- "- Overall status: \`%s\`\n" "$OVERALL_STATUS"
    printf -- "- Scope: \`%s\`\n" "$SCOPE_KIND"
    printf -- "- Mode: \`%s\`\n" "$MODE"
    printf -- "- Levels: \`%s\`\n" "$level_list"
    printf -- "- Run root: \`%s\`\n" "${RUN_ROOT#"$ROOT_DIR"/}"
    printf -- "- Evidence basis: \`%s\` gates, \`%s\` profiling events, \`%s\` tracked files inventoried\n" \
      "${#GATE_STATUS[@]}" "$PROFILE_EVENT_COUNT" "$COVERAGE_TRACKED_FILES"
    printf -- "- Blocking counts: fail \`%s\`, unknown \`%s\`, human-review \`%s\`, skipped \`%s\`\n" \
      "$fail_gate_count" "$unknown_gate_count" "$human_review_gate_count" "$skipped_gate_count"
    printf -- "- Integration contract: leaked output gates \`%s\`; crate-unmapped files \`%s\`\n\n" \
      "$leak_gate_count" "$COVERAGE_CRATE_UNMAPPED_COUNT"

    printf '```mermaid\n'
    printf 'pie showData\n'
    printf '    title Gate Blocking Distribution\n'
    printf '    "Nonblocking" : %s\n' "$nonblocking_gate_count"
    printf '    "FAIL" : %s\n' "$fail_gate_count"
    printf '    "UNKNOWN" : %s\n' "$unknown_gate_count"
    printf '    "HUMAN_REVIEW" : %s\n' "$human_review_gate_count"
    printf '    "SKIPPED" : %s\n' "$skipped_gate_count"
    printf '```\n\n'

    printf '## 📦 Evidence Provenance\n\n'
    printf -- "- Generated UTC: \`%s\`\n" "$generated_at"
    printf -- "- Run root: \`%s\`\n" "${RUN_ROOT#"$ROOT_DIR"/}"
    printf -- "- Timestamp stamp: \`%s\`\n" "$RUN_TIMESTAMP"
    printf -- "- Report format: \`%s\`\n" "${REPORT_FORMAT_PATH#"$ROOT_DIR"/}"
    printf -- "- Stale verifier run roots compacted before start: \`%s\`\n" "$STALE_RUN_ROOTS_TRASHED"
    printf -- "- External interferer processes killed: \`%s\`\n" "$EXTERNAL_INTERFERERS_KILLED"
    printf -- "- Mode: \`%s\`\n" "$MODE"
    printf -- "- Scope: \`%s\`\n" "$SCOPE_KIND"
    printf -- "- Target: \`%s\`\n" "$TARGET_LABEL"
    printf -- "- Levels: \`%s\`\n" "$level_list"
    printf -- "- Release profile args: \`%s\`\n" "${Z00Z_CARGO_PROFILE_ARGS:-unset}"
    printf -- "- Cache root: \`%s\`\n" "${RUN_CACHE_ROOT#"$ROOT_DIR"/}"
    printf -- "- Cargo home: \`%s\`\n" "${CARGO_HOME#"$ROOT_DIR"/}"
    printf -- "- Cargo install root: \`%s\`\n" "${CARGO_INSTALL_ROOT#"$ROOT_DIR"/}"
    printf -- "- Canonical tmp root: \`%s\`\n" "${TMP_ROOT#"$ROOT_DIR"/}"
    printf -- "- Specs runtime root: \`%s\`\n" "${SPECS_RUNTIME_ROOT#"$ROOT_DIR"/}"
    printf -- "- Verification runtime root: \`%s\`\n" "${VERIFICATION_RUNTIME_ROOT#"$ROOT_DIR"/}"
    printf -- "- Fuzz runtime root: \`%s\`\n" "${FUZZ_RUNTIME_ROOT#"$ROOT_DIR"/}"
    printf -- "- Cargo target dir: \`%s\`\n" "${CARGO_TARGET_ROOT#"$ROOT_DIR"/}"
    printf -- "- Python bytecode writes disabled: \`%s\`\n" "${PYTHONDONTWRITEBYTECODE:-0}"
    printf -- "- Protected vendor path touched: \`%s\`\n" "$TARGET_TOUCHES_VENDOR"
    printf -- "- Core evidence paths: \`%s\`, \`%s\`, \`%s\`, \`%s\`, \`%s\`, \`%s\`, \`%s\`, \`%s\`\n" \
      "${REPORT_LOG_DIR#"$ROOT_DIR"/}" \
      "${PROFILE_EVENTS_PATH#"$ROOT_DIR"/}" \
      "${PROFILE_SUMMARY_PATH#"$ROOT_DIR"/}" \
      "${PROFILE_TOOL_SUMMARY_PATH#"$ROOT_DIR"/}" \
      "${PROFILE_RESOURCE_SUMMARY_PATH#"$ROOT_DIR"/}" \
      "${RUN_FOOTPRINT_SUMMARY_PATH#"$ROOT_DIR"/}" \
      "${HJMT_SUMMARY_PATH#"$ROOT_DIR"/}" \
      "${SECURITY_BRAINSTORM_SUMMARY_PATH#"$ROOT_DIR"/}"
    if [[ "$SCOPE_KIND" == "project" ]]; then
      printf -- "- Coverage evidence paths: \`%s\`, \`%s\`\n" \
        "${COVERAGE_MANIFEST_PATH#"$ROOT_DIR"/}" \
        "${COVERAGE_SUMMARY_PATH#"$ROOT_DIR"/}"
    fi
    if [[ -f "$BOOTSTRAP_SUMMARY_PATH" ]]; then
      printf -- "- Bootstrap summary path: \`%s\`\n" "${BOOTSTRAP_SUMMARY_PATH#"$ROOT_DIR"/}"
    fi
    if [[ -f "$RUNTIME_BOOTSTRAP_SUMMARY_PATH" ]]; then
      printf -- "- Runtime bootstrap summary path: \`%s\`\n" "${RUNTIME_BOOTSTRAP_SUMMARY_PATH#"$ROOT_DIR"/}"
    fi
    printf -- "- Report validation summary: \`%s\`\n\n" "${REPORT_VALIDATION_SUMMARY_PATH#"$ROOT_DIR"/}"

    printf '## 🚦 Gate Matrix\n\n'
    printf '| Gate | Checker module | Status | Elapsed (s) | Log | Primary artifacts |\n'
    printf '| --- | --- | --- | --- | --- | --- |\n'
    for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
      status="${GATE_STATUS[$gate_id]}"
      gate_artifacts="$(gate_existing_artifacts_csv "$gate_id")"
      gate_artifacts="$(printf '%s' "$gate_artifacts" | sed 's/,/; /g')"
      if [[ "$DRY_RUN" -eq 1 ]]; then
        printf "| \`%s\` | \`%s\` | \`%s\` | \`-\` | \`dry-run\` | \`%s\` |\n" \
          "$gate_id" "${GATE_MODULE[$gate_id]:--}" "$status" "$gate_artifacts"
      else
        printf "| \`%s\` | \`%s\` | \`%s\` | \`%s\` | \`%s\` | \`%s\` |\n" \
          "$gate_id" "${GATE_MODULE[$gate_id]:--}" "$status" "${GATE_ELAPSED_SECS[$gate_id]:--}" \
          "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}" "$gate_artifacts"
      fi
    done
    printf '\n'

    printf '## 🧪 Conclusion Ledger\n\n'
    printf '| Gate | Checker module | Machine conclusion | Validity ceiling | Anchoring artifact |\n'
    printf '| --- | --- | --- | --- | --- |\n'
    for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
      status="${GATE_STATUS[$gate_id]}"
      printf "| \`%s\` | \`%s\` | \`%s\` | %s | \`%s\` |\n" \
        "$gate_id" \
        "${GATE_MODULE[$gate_id]:--}" \
        "$status" \
        "$(status_validity_ceiling "$status")" \
        "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
    done
    printf '\n'

    printf '## 🔍 Validity And Doublecheck\n\n'
    printf -- '- The orchestrator does not upgrade conclusions above raw tool evidence. A gate is only stronger than `PASS` when the underlying log emitted `TESTED`, `BOUNDED_VERIFIED`, `MODEL_CHECKED`, `FORMALLY_PROVED`, or `SECURITY_PROTOCOL_PROVED`.\n'
    printf -- '- Missing tools, missing models, missing specs, and non-closed semantic gaps stay `UNKNOWN`.\n'
    printf -- '- High-risk adversarial hypotheses stay `NEEDS_HUMAN_CRYPTO_REVIEW`; they are not treated as proven exploits.\n'
    printf -- '- Artifact traceability lives in this run root only: logs in `logs/`, gate state under `specs*`, `verification*`, `fuzz*`, temp state under `tmp*`, profiling in `profiling/`.\n'
    if [[ "$leak_gate_count" -eq 0 ]]; then
      printf -- '- Canonical artifact contract check: no unauthorized root/runtime leak was detected.\n'
    else
      printf -- '- Canonical artifact contract check: `%s` gate(s) emitted unauthorized paths; moved evidence is preserved under `leaks/` and the run is blocking.\n' "$leak_gate_count"
    fi
    if [[ "$root_cache_manifest_count" == "0" ]]; then
      printf -- '- Production/dev cache observation: no `repo/.cache` mutation manifest was captured during this run.\n'
    else
      printf -- '- Production/dev cache observation: `%s` `repo/.cache` manifest(s) were captured under `interference/`; they are treated as production/dev cache snapshots, not verifier-owned outputs.\n' "$root_cache_manifest_count"
    fi
    if [[ -f "${GATE_LOG[l3-kani]:-}" ]]; then
      python3 - "${GATE_LOG[l3-kani]}" <<'PY'
import pathlib
import re
import sys

text = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
notes = []
if "cannot honor Z00Z_CARGO_PROFILE_ARGS='--release'" in text:
    notes.append(
        "Kani validity note: this run requested `--release`, but `cargo-kani` executed in its supported test-profile flow; treat `l3-kani` as bounded harness evidence, not release-codegen equivalence."
    )
unsupported = []
for needle in ("caller_location", "foreign function", "atomic_singlethreadfence"):
    if needle in text:
        unsupported.append(needle)
if unsupported:
    notes.append(
        "Kani validity note: unsupported or reduced-fidelity constructs were present in the analyzed harnesses (`{}`); atomics/concurrency were not modeled with full runtime semantics.".format(
            ", ".join(unsupported)
        )
    )
for note in notes:
    print(f"- {note}")
PY
    fi
    if [[ -f "${GATE_LOG[l3-miri]:-}" ]]; then
      python3 - "${GATE_LOG[l3-miri]}" <<'PY'
import pathlib
import sys

text = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
if "Miri does not support optimizations" in text:
    print(
        "- Miri validity note: `l3-miri` ran under a release-selected profile, but Miri ignored optimization level; use it as UB/interpreter evidence, not optimized-machine-code equivalence."
    )
PY
    fi
    printf -- '- Doublecheck inputs: `%s`, `%s`, `%s`, `%s`, `%s`.\n\n' \
      "${REPORT_LOG_DIR#"$ROOT_DIR"/}" \
      "${PROFILE_SUMMARY_PATH#"$ROOT_DIR"/}" \
      "$doublecheck_coverage_input" \
      "$doublecheck_bootstrap_input" \
      "$doublecheck_security_input"

    printf '## 🏗️ Bootstrap Artifact Provenance\n\n'
    if [[ "$MODE" == "report" ]]; then
      printf -- '- Report mode did not edit repo-owned verification artifacts.\n'
      printf -- '- Report-local runtime verifier assets may be staged under the active run root.\n'
      if [[ -f "$RUNTIME_BOOTSTRAP_SUMMARY_PATH" ]]; then
        printf -- "- Runtime bootstrap summary: \`%s\`\n" "${RUNTIME_BOOTSTRAP_SUMMARY_PATH#"$ROOT_DIR"/}"
      fi
      printf '\n'
    elif [[ "$DRY_RUN" -eq 1 ]]; then
      printf -- '- Bootstrap artifact generation planned in dry-run mode.\n\n'
    else
      printf -- "- Generated artifacts: \`%s\`\n" "$BOOTSTRAP_CREATED_COUNT"
      printf -- "- Refreshed generated artifacts: \`%s\`\n" "$BOOTSTRAP_UPDATED_COUNT"
      printf -- "- Skipped manual pre-existing artifacts: \`%s\`\n" "$BOOTSTRAP_SKIPPED_COUNT"
      printf -- "- Bootstrap summary: \`%s\`\n\n" "${BOOTSTRAP_SUMMARY_PATH#"$ROOT_DIR"/}"
    fi

    printf '## 📊 Performance And Resource Profiling\n\n'
    if [[ "$DRY_RUN" -eq 1 ]]; then
      printf -- '- Profiling is not recorded during dry-run mode.\n\n'
    else
      if [[ -f "$PROFILE_TOOL_SUMMARY_PATH" ]]; then
        printf -- "- Profiler tool inventory: \`%s\`\n\n" "${PROFILE_TOOL_SUMMARY_PATH#"$ROOT_DIR"/}"
        printf '| Tool | Available | Path | Version |\n'
        printf '| --- | --- | --- | --- |\n'
        python3 - "$PROFILE_TOOL_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for item in summary.get("tools", []):
    version = (item.get("version") or "-").replace("|", "/")
    print(
        "| `{}` | `{}` | `{}` | `{}` |".format(
            item.get("name", "-"),
            "yes" if item.get("available") else "no",
            item.get("path") or "-",
            version,
        )
    )
PY
        printf '\n'
      fi

      printf -- "- Profiling events: \`%s\`\n" "${PROFILE_EVENTS_PATH#"$ROOT_DIR"/}"
      printf -- "- Profiling summary: \`%s\`\n" "${PROFILE_SUMMARY_PATH#"$ROOT_DIR"/}"
      printf -- "- Resource profiles: \`%s\`\n" "${PROFILE_RESOURCE_DIR#"$ROOT_DIR"/}"
      printf -- "- Resource summary: \`%s\`\n" "${PROFILE_RESOURCE_SUMMARY_PATH#"$ROOT_DIR"/}"
      printf -- "- Run-footprint summary: \`%s\`\n" "${RUN_FOOTPRINT_SUMMARY_PATH#"$ROOT_DIR"/}"
      printf -- "- Profiled events: \`%s\` total, \`%s\` gate-level, \`%s\` command-level\n" \
        "$PROFILE_EVENT_COUNT" "$PROFILE_GATE_EVENT_COUNT" "$PROFILE_COMMAND_EVENT_COUNT"
      if [[ "$PROFILE_TOP_N" -gt 0 ]]; then
        python3 - "$PROFILE_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(
    "- Slowest slice reported: top `5%` => `{}` events consuming `{}`s / `{}`s total (`{}%`)".format(
        summary.get("top_n", 0),
        summary.get("slowest_total_secs", 0),
        summary.get("total_elapsed_secs", 0),
        summary.get("slowest_fraction_percent", 0),
    )
)
PY
        python3 - "$PROFILE_SUMMARY_PATH" <<'PY'
import json
import math
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
slowest = summary.get("slowest", [])[:5]
if not slowest:
    raise SystemExit(0)
labels = [str(item.get("label", "-")).replace('"', "'")[:24] for item in slowest]
values = [round(float(item.get("elapsed_secs", 0)), 3) for item in slowest]
upper = max(values) if values else 1
upper = max(1, math.ceil(upper))
print()
print("```mermaid")
print("xychart-beta")
print('    title "Slowest Top 5% Events (seconds)"')
print(f'    x-axis {json.dumps(labels)}')
print(f'    y-axis "seconds" 0 --> {upper}')
print(f'    bar {json.dumps(values)}')
print("```")
PY
        printf '\n'
        printf '| Kind | Label | Status | Elapsed (s) | Command | Recommendation |\n'
        printf '| --- | --- | --- | --- | --- | --- |\n'
        python3 - "$PROFILE_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for item in summary.get("slowest", []):
    command = item.get("command", "")
    if len(command) > 120:
        command = command[:117] + "..."
    recommendations = "; ".join(item.get("recommendations", [])[:2]) or "-"
    if len(recommendations) > 160:
        recommendations = recommendations[:157] + "..."
    print(
        "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |".format(
            item.get("kind", ""),
            item.get("label", ""),
            item.get("status", ""),
            item.get("elapsed_secs", 0),
            command,
            recommendations,
        )
    )
PY
        python3 - "$PROFILE_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
guidance = summary.get("guidance_source")
recommendations = summary.get("aggregate_recommendations", [])
if recommendations:
    print()
    print("Aggregate acceleration candidates:")
    for item in recommendations[:6]:
        print(f"- {item}")
if guidance:
    print()
    print(f"- Profiling guidance source: `{guidance}`")
PY
        printf '\n'
      else
        printf -- '- No profiling events were captured.\n\n'
      fi

      if [[ -f "$PROFILE_RESOURCE_SUMMARY_PATH" ]]; then
        python3 - "$PROFILE_RESOURCE_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if not summary.get("available"):
    print("- GNU time resource profiles were not captured.")
    raise SystemExit(0)
def label(item: dict[str, object]) -> str:
    return str(item.get("label") or item.get("gate_id") or "-")

def roots(items: list[str]) -> str:
    if not items:
        return "-"
    return "<br>".join(f"`{item}`" for item in items[:2])

print("Top CPU-total profiles:")
print()
print("| Profile | Kind | Mode | Wall (s) | CPU total (s) | CPU % | Max RSS (KB) | Exit |")
print("| --- | --- | --- | --- | --- | --- | --- | --- |")
for item in summary.get("top_cpu_total", [])[:5]:
    print(
        "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |".format(
            label(item),
            item.get("profile_kind", "-"),
            item.get("execution_mode", "-"),
            item.get("wall_elapsed_secs", 0),
            item.get("cpu_total_secs", 0),
            item.get("cpu_percent", 0),
            item.get("max_rss_kb", 0),
            item.get("exit_status", 0),
        )
    )
print()
print("Top memory-RSS profiles:")
print()
print("| Profile | Max RSS (KB) | Wall (s) | CPU total (s) | FS in | FS out |")
print("| --- | --- | --- | --- | --- | --- |")
for item in summary.get("top_memory_rss", [])[:5]:
    print(
        "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |".format(
            label(item),
            item.get("max_rss_kb", 0),
            item.get("wall_elapsed_secs", 0),
            item.get("cpu_total_secs", 0),
            item.get("fs_inputs", 0),
            item.get("fs_outputs", 0),
        )
    )
print()
print("Top filesystem-I/O profiles:")
print()
print("| Profile | FS in | FS out | Wall (s) | CPU total (s) | Max RSS (KB) |")
print("| --- | --- | --- | --- | --- | --- |")
for item in summary.get("top_fs_io", [])[:5]:
    print(
        "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |".format(
            label(item),
            item.get("fs_inputs", 0),
            item.get("fs_outputs", 0),
            item.get("wall_elapsed_secs", 0),
            item.get("cpu_total_secs", 0),
            item.get("max_rss_kb", 0),
        )
    )
print()
print("Performance inventory snapshots:")
print()
print("| Profile | Kind | Mode | Targets | Caches | Cleanup (ms) | Reclaimed bytes |")
print("| --- | --- | --- | --- | --- | --- | --- |")
for item in summary.get("top_wall", [])[:8]:
    print(
        "| `{}` | `{}` | `{}` | {} | {} | `{}` | `{}` |".format(
            label(item),
            item.get("profile_kind", "-"),
            item.get("execution_mode", "-"),
            roots(list(item.get("target_roots") or [])),
            roots(list(item.get("cache_roots") or [])),
            item.get("cleanup_elapsed_ms", 0),
            item.get("cleanup_reclaimed_bytes", 0),
        )
    )
PY
        printf '\n'
      fi

      if [[ -f "$RUN_FOOTPRINT_SUMMARY_PATH" ]]; then
        python3 - "$RUN_FOOTPRINT_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

def humanize(value: int) -> str:
    size = float(value)
    for unit in ("B", "KiB", "MiB", "GiB", "TiB"):
        if size < 1024 or unit == "TiB":
            return f"{size:.2f} {unit}"
        size /= 1024
    return f"{value} B"

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(f"- Active run-root disk footprint: `{humanize(int(summary.get('active_total_bytes', 0)))}`")
cleanup = summary.get("cache_cleanup") or {}
if cleanup.get("available"):
    print(
        "- Stale verifier run-root cleanup: `{}` invocation(s), `{}` trashed run root(s), `{}` reclaimed, `{}`s total overhead.".format(
            cleanup.get("invocation_count", 0),
            cleanup.get("total_trimmed_roots", 0),
            humanize(int(cleanup.get("total_reclaimed_bytes", 0))),
            cleanup.get("total_elapsed_secs", 0),
        )
    )
top_level = summary.get("top_level_active", [])
if top_level:
    print()
    print("| Top-level path | Kind | Size |")
    print("| --- | --- | --- |")
    for item in top_level[:8]:
        print(
            "| `{}` | `{}` | `{}` |".format(
                item.get("path", "-"),
                item.get("kind", "-"),
                humanize(int(item.get("bytes", 0))),
            )
        )
largest = summary.get("largest_files", [])
if largest:
    print()
    print("Largest files:")
    for item in largest[:5]:
        print(f"- `{item.get('path', '-')}` => `{humanize(int(item.get('bytes', 0)))}`")
PY
        printf '\n'
      fi
    fi

    printf '## 🌲 HJMT Runtime Evidence\n\n'
    if [[ "$DRY_RUN" -eq 1 ]]; then
      printf -- '- HJMT runtime evidence is not produced during dry-run mode.\n\n'
    elif [[ -f "$HJMT_SUMMARY_PATH" ]]; then
      printf -- "- HJMT summary: \`%s\`\n" "${HJMT_SUMMARY_PATH#"$ROOT_DIR"/}"
      python3 - "$HJMT_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if not summary.get("available"):
    print("- No HJMT artifact pair was found under the active run root.")
    print("- TPS status: not measured in this run because no active-run HJMT or throughput artifact was produced.")
    raise SystemExit(0)
print(f"- Primary metrics artifact: `{summary.get('primary_metrics_path') or '-'}`")
print(f"- Primary proof-size artifact: `{summary.get('primary_proof_path') or '-'}`")
cache = summary.get("cache_scheduler") or {}
if cache:
    print(
        "- Cache metrics: hits `{}`, misses `{}`, hit ratio `{}`, invalidations `{}`, root reuse `{}`, proof-segment reuse `{}`".format(
            cache.get("cache_hit_count", 0),
            cache.get("cache_miss_count", 0),
            cache.get("cache_hit_ratio"),
            cache.get("invalidation_count", 0),
            cache.get("root_reuse_ratio"),
            cache.get("proof_segment_reuse_ratio"),
        )
    )
    print(
        "- Scheduler metrics: queue depth `{}`, max queued `{}`, max active `{}`, backpressure `{}`, reject `{}`, cancel `{}`, deterministic ordering `{}`, last blocking wait us `{}`".format(
            cache.get("scheduler_queue_depth"),
            cache.get("max_queued"),
            cache.get("max_active"),
            cache.get("scheduler_backpressure_count"),
            cache.get("reject_count"),
            cache.get("cancel_count"),
            cache.get("deterministic_parent_ordering"),
            cache.get("last_blocking_wait_us"),
        )
    )
proof = summary.get("proof_examples") or {}
if proof:
    print(
        "- Proof examples: `{}` entries; proof bytes min/median/max = `{}/{}/{}`; verify us min/median/max = `{}/{}/{}`".format(
            proof.get("entry_count", 0),
            (proof.get("proof_size_bytes") or {}).get("min"),
            (proof.get("proof_size_bytes") or {}).get("median"),
            (proof.get("proof_size_bytes") or {}).get("max"),
            (proof.get("verify_time_us") or {}).get("min"),
            (proof.get("verify_time_us") or {}).get("median"),
            (proof.get("verify_time_us") or {}).get("max"),
        )
    )
    print()
    print("| Slowest examples | Backend | Verify us | Proof bytes |")
    print("| --- | --- | --- | --- |")
    for item in proof.get("slowest_examples", [])[:5]:
        print(
            "| `{}` | `{}` | `{}` | `{}` |".format(
                item.get("example_id", "-"),
                item.get("backend_mode", "-"),
                item.get("verify_time_us", "-"),
                item.get("proof_size_bytes", "-"),
            )
        )
    print()
    print("| Largest examples | Backend | Proof bytes | Verify us |")
    print("| --- | --- | --- | --- |")
    for item in proof.get("largest_examples", [])[:5]:
        print(
            "| `{}` | `{}` | `{}` | `{}` |".format(
                item.get("example_id", "-"),
                item.get("backend_mode", "-"),
                item.get("proof_size_bytes", "-"),
                item.get("verify_time_us", "-"),
            )
        )
tps = summary.get("tps") or {}
print()
if tps.get("measured"):
    print(f"- TPS evidence artifacts: `{', '.join(tps.get('artifact_paths', []))}`")
else:
    print(f"- TPS status: not measured in this run. {tps.get('reason', '')}")
PY
      printf '\n'
    else
      printf -- '- No HJMT summary artifact was produced in this pass.\n\n'
    fi

	    if [[ "$SCOPE_KIND" == "project" ]]; then
	      printf '## 🗺️ Coverage Inventory\n\n'
      if [[ "$DRY_RUN" -eq 1 ]]; then
        printf -- "- Coverage manifest: \`dry-run\`\n"
      else
        printf -- "- Tracked files inventoried: \`%s\`\n" "$COVERAGE_TRACKED_FILES"
        printf -- "- Coverage manifest: \`%s\`\n" "${COVERAGE_MANIFEST_PATH#"$ROOT_DIR"/}"
        printf -- "- Coverage summary: \`%s\`\n" "${COVERAGE_SUMMARY_PATH#"$ROOT_DIR"/}"
        printf -- "- Coverage status counts: fail \`%s\`, skipped \`%s\`, unknown \`%s\`, unmapped \`%s\`\n" \
          "$COVERAGE_FAIL_COUNT" "$COVERAGE_SKIPPED_COUNT" "$COVERAGE_UNKNOWN_COUNT" "$COVERAGE_UNMAPPED_COUNT"
        printf -- "- Crate-unmapped tracked files: \`%s\`\n\n" "$COVERAGE_CRATE_UNMAPPED_COUNT"
	      fi
	    fi

	    printf '## 🚨 Risk Register\n\n'
      printf -- '- Severity below is orchestrator triage severity, not exploit-proof severity.\n\n'
      if [[ "$fail_gate_count" -eq 0 && "$unknown_gate_count" -eq 0 && "$human_review_gate_count" -eq 0 && ! -f "$SECURITY_BRAINSTORM_SUMMARY_PATH" ]]; then
        printf -- '- No blocking or security-risk items were synthesized into the risk register in this pass.\n\n'
      else
        printf '| Class | Source | Severity | Rationale | Anchor |\n'
        printf '| --- | --- | --- | --- | --- |\n'
        for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
          status="${GATE_STATUS[$gate_id]}"
          case "$status" in
            FAIL)
              printf "| \`gate-blocker\` | \`%s\` | \`high\` | gate failed or artifact-confinement contract broke | \`%s\` |\n" \
                "$gate_id" "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
              ;;
            NEEDS_HUMAN_CRYPTO_REVIEW)
              printf "| \`expert-review\` | \`%s\` | \`high\` | machine review raised crypto/security hypotheses that remain open | \`%s\` |\n" \
                "$gate_id" "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
              ;;
            UNKNOWN)
              printf "| \`evidence-gap\` | \`%s\` | \`medium\` | tool/model/spec closure is missing, so assurance is incomplete | \`%s\` |\n" \
                "$gate_id" "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
              ;;
          esac
        done
        if [[ -f "$SECURITY_BRAINSTORM_SUMMARY_PATH" ]]; then
          python3 - "$SECURITY_BRAINSTORM_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for item in summary.get("top_findings", [])[:5]:
    evidence = ", ".join(item.get("evidence_files", [])[:2]) or "-"
    print(
        "| `adversarial` | `{}` | `{}` | {} | `{}` |".format(
            item.get("class", "unknown"),
            item.get("severity", "unknown"),
            item.get("title", "").replace("|", "/"),
            evidence,
        )
    )
PY
        fi
        printf '\n'
        printf '### Gate Evidence Highlights\n\n'
	      for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
	        status="${GATE_STATUS[$gate_id]}"
	        case "$status" in
	          FAIL|UNKNOWN|NEEDS_HUMAN_CRYPTO_REVIEW)
	            printf '### %s\n\n' "$gate_id"
	            printf -- '- Status: `%s`\n' "$status"
	            printf -- '- Checker module: `%s`\n' "${GATE_MODULE[$gate_id]:--}"
	            printf -- '- Log: `%s`\n' "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
	            printf -- '- Primary artifacts: `%s`\n' "$(gate_existing_artifacts_csv "$gate_id")"
	            python3 - "${GATE_LOG[$gate_id]}" <<'PY'
import pathlib
import sys

log_path = pathlib.Path(sys.argv[1])
lines = log_path.read_text(encoding="utf-8", errors="replace").splitlines()
interesting = []
needles = (
    "ERROR:",
    "UNKNOWN:",
    "NEEDS_HUMAN_CRYPTO_REVIEW:",
    "Summary:",
    "FAIL:",
    "FAILED",
    "failures:",
    "panicked at ",
    "error: test failed",
)
for raw in lines:
    line = raw.strip()
    if not line:
        continue
    if any(token in line for token in needles):
        if line not in interesting:
            interesting.append(line)
for line in interesting[:8]:
    print(f"- Evidence: `{line}`")
if not interesting:
    tail = [line.strip() for line in lines if line.strip()][-5:]
    for line in tail:
        print(f"- Tail: `{line}`")
PY
	            printf '\n'
	            ;;
	        esac
	      done
	    fi

	    printf '## 🔗 Supply-Chain Highlights\n\n'
	    if [[ -f "$RUN_ROOT/supply-chain/supply-chain-summary.json" ]]; then
	      python3 - "$RUN_ROOT/supply-chain/supply-chain-summary.json" "$RUN_ROOT/supply-chain/supply-chain-project.md" "$RUN_ROOT/supply-chain/supply-chain-vendor.md" "$ROOT_DIR" <<'PY'
import json
import pathlib
import re
import sys

summary_path = pathlib.Path(sys.argv[1])
project_report = pathlib.Path(sys.argv[2])
vendor_report = pathlib.Path(sys.argv[3])
root = pathlib.Path(sys.argv[4]).resolve()
summary = json.loads(summary_path.read_text(encoding="utf-8"))

def rel_or_abs(path: pathlib.Path) -> str:
    try:
        return path.resolve().relative_to(root).as_posix()
    except Exception:
        return str(path)

def headings(path: pathlib.Path):
    if not path.exists():
        return []
    return re.findall(r"^##\s+(.+)$", path.read_text(encoding="utf-8"), flags=re.MULTILINE)

project = summary.get("project", {})
vendor = summary.get("vendor", {})
mixed = summary.get("mixed", {})
print(f"- Summary JSON: `{rel_or_abs(summary_path)}`")
print(f"- Project report: `{rel_or_abs(project_report)}`")
print(f"- Vendor report: `{rel_or_abs(vendor_report)}`")
print(
    f"- Counts: project unreviewed `{project.get('unreviewed', 0)}`, "
    f"project reviewed `{project.get('reviewed', 0)}`, "
    f"vendor unreviewed `{vendor.get('unreviewed', 0)}`, "
    f"vendor reviewed `{vendor.get('reviewed', 0)}`, "
    f"mixed unreviewed `{mixed.get('unreviewed', 0)}`"
)
for title in headings(project_report)[:6]:
    print(f"- Project advisory: `{title}`")
for title in headings(vendor_report)[:4]:
    print(f"- Vendor advisory: `{title}`")
PY
	      printf '\n'
	    else
	      printf -- '- No supply-chain summary artifact was produced in this pass.\n\n'
	    fi

	    printf '## 🛡️ Adversarial Security Review\n\n'
	    if [[ -f "$SECURITY_BRAINSTORM_SUMMARY_PATH" ]]; then
	      python3 - "$SECURITY_BRAINSTORM_SUMMARY_PATH" "$ROOT_DIR" <<'PY'
import json
import pathlib
import sys

summary_path = pathlib.Path(sys.argv[1])
root = pathlib.Path(sys.argv[2]).resolve()
data = json.loads(summary_path.read_text(encoding="utf-8"))

def rel_or_abs(value: str) -> str:
    if not value:
        return "-"
    path = pathlib.Path(value)
    try:
        return path.resolve().relative_to(root).as_posix()
    except Exception:
        return value

print(f"- Summary JSON: `{summary_path.resolve().relative_to(root).as_posix()}`")
print(f"- Code files scanned: `{data.get('files_scanned', 0)}`")
print(
    f"- Prompt sources scanned under `.github/`: `{data.get('prompt_sources_scanned', 0)}` "
    f"with `{data.get('prompt_sources_relevant', 0)}` security-relevant"
)
print(
    f"- Findings: `{data.get('findings_total', 0)}` total; "
    f"high `{data.get('high_risk_count', 0)}`, medium `{data.get('medium_risk_count', 0)}`, "
    f"low `{data.get('low_risk_count', 0)}`"
)
kind_counts = data.get("prompt_sources_by_kind", {})
if kind_counts:
    kinds = ", ".join(f"{kind} `{count}`" for kind, count in sorted(kind_counts.items()))
    print(f"- Prompt corpus kinds: {kinds}")
classes = data.get("findings_by_class", {})
print(
    f"- Classes: file `{classes.get('file', 0)}`, module `{classes.get('module', 0)}`, "
    f"crate `{classes.get('crate', 0)}`, cross-crate `{classes.get('cross-crate', 0)}`"
)
ownership = data.get("ownership_counts", {})
print(
    f"- Ownership: project-owned `{ownership.get('project-owned', 0)}`, "
    f"protected-vendor `{ownership.get('protected-vendor', 0)}`, mixed `{ownership.get('mixed', 0)}`"
)
print(f"- Prompt corpus JSON: `{rel_or_abs(data.get('prompt_corpus_path', ''))}`")
print(f"- Attack-surface registry JSON: `{rel_or_abs(data.get('family_registry_path', ''))}`")
print(f"- Detailed report: `{rel_or_abs(data.get('report_markdown', ''))}`")
print(f"- Summary JSON: `{rel_or_abs(data.get('report_json', ''))}`")
print()
print("| Severity | Class | Ownership | Hypothesis | Evidence anchors |")
print("| --- | --- | --- | --- | --- |")
for finding in data.get("top_findings", [])[:10]:
    evidence = ", ".join(f"`{path}`" for path in finding.get("evidence_files", [])[:3]) or "`-`"
    print(
        "| `{}` | `{}` | `{}` | {} | {} |".format(
            finding.get("severity", "unknown"),
            finding.get("class", "unknown"),
            finding.get("ownership", "unknown"),
            finding.get("title", "").replace("|", "/"),
            evidence,
        )
    )
top_prompt_sources = data.get("top_prompt_sources", [])
if top_prompt_sources:
    print()
    print("- Highest-signal `.github/` prompt sources:")
    for source in top_prompt_sources[:6]:
        categories = ", ".join(source.get("categories", [])[:4]) or "-"
        print(
            f"  - `{source.get('kind', 'other')}` `{source.get('path', '-')}` "
            f"(categories: {categories}; excerpts: `{source.get('excerpt_count', 0)}`)"
        )
top = data.get("top_findings", [])
if top:
    print()
    print("Top hypotheses:")
    for item in top[:6]:
        evidence = ", ".join(f"`{path}`" for path in item.get("evidence_files", [])[:3]) or "`-`"
        print(
            f"- `{item.get('severity', 'unknown').upper()}` `{item.get('class', 'finding')}` "
            f"{item.get('title', 'untitled')} -- evidence: {evidence}"
        )
print()
PY
	      printf '\n'
	    else
	      printf -- '- No adversarial brainstorming summary artifact was produced in this pass.\n\n'
	    fi

	    printf '## 🧰 Project-Owned Fixable Findings\n\n'
    if [[ "$fail_gate_count" -eq 0 && "$unknown_gate_count" -eq 0 && "$human_review_gate_count" -eq 0 ]]; then
      printf -- '- No failing project-owned gates were detected in this pass.\n\n'
    else
      for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
        status="${GATE_STATUS[$gate_id]}"
        [[ "$gate_id" == "l4-vendor-unsafe" ]] && continue
        case "$status" in
          FAIL|UNKNOWN|NEEDS_HUMAN_CRYPTO_REVIEW)
            printf -- '- `%s` via `%s` => `%s`; log `%s`\n' \
              "$gate_id" "${GATE_MODULE[$gate_id]:--}" "$status" "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
            ;;
        esac
      done
      printf '\n'
    fi

    printf '## 📚 Protected Vendor Findings\n\n'
    if [[ "$vendor_count" -gt 0 || -f "$RUN_ROOT/supply-chain/supply-chain-vendor.md" ]]; then
      printf -- "- Vendor unsafe facts found: \`%s\`\n" "$vendor_count"
      printf -- "- Vendor unsafe report: \`%s\`\n" "${vendor_report_path#"$ROOT_DIR"/}"
      if [[ -f "$RUN_ROOT/supply-chain/supply-chain-vendor.md" ]]; then
        printf -- "- Vendor supply-chain report: \`%s\`\n" "${RUN_ROOT#"$ROOT_DIR"/}/supply-chain/supply-chain-vendor.md"
      fi
      printf -- '- Policy: do not auto-edit protected vendor code; only report, wrap, pin, upstream, or isolate.\n\n'
    else
      printf -- '- No protected vendor findings were recorded in this pass.\n\n'
    fi

    printf '## 🧩 Missing Evidence Or Missing Models\n\n'
    if [[ "$unknown_gate_count" -eq 0 && "$skipped_gate_count" -eq 0 ]]; then
      printf -- '- No machine-visible missing-model or missing-tool gaps were left in this pass.\n\n'
    else
      for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
        status="${GATE_STATUS[$gate_id]}"
        case "$status" in
          UNKNOWN|SKIPPED)
            printf -- '- `%s` => `%s`; inspect `%s`\n' "$gate_id" "$status" "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
            ;;
        esac
      done
      printf '\n'
    fi

    printf '## ✅ Recommended Actions\n\n'
    if [[ "$fail_gate_count" -eq 0 && "$unknown_gate_count" -eq 0 && "$human_review_gate_count" -eq 0 && "$leak_gate_count" -eq 0 ]]; then
      printf -- '- No blocker-specific module recommendations were synthesized in this pass.\n\n'
    else
      for gate_id in $(printf '%s\n' "${!GATE_STATUS[@]}" | sort); do
        status="${GATE_STATUS[$gate_id]}"
        case "$status" in
          FAIL|UNKNOWN|NEEDS_HUMAN_CRYPTO_REVIEW)
            printf '### %s\n\n' "$gate_id"
            printf -- '- Checker module: `%s`\n' "${GATE_MODULE[$gate_id]:--}"
            printf -- '- Status: `%s`\n' "$status"
            printf -- '- Validity ceiling: %s\n' "$(status_validity_ceiling "$status")"
            printf -- '- Anchor log: `%s`\n' "${GATE_LOG[$gate_id]#"$ROOT_DIR"/}"
            printf -- '- Anchor artifacts: `%s`\n' "$(gate_existing_artifacts_csv "$gate_id")"
            case "$gate_id" in
              l0-docs)
                printf -- '- Recommendation: format `deny.toml` with `taplo format deny.toml` and rerun the L0 gate so TOML drift stops hiding real semantic doc issues.\n'
                python3 - "${GATE_LOG[$gate_id]}" <<'PY'
import pathlib
import re
import sys

text = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
summary = re.search(r"Summary:\s+(\d+)\s+error\(s\)", text)
zinv = re.search(r"ZINV references:\s+(\d+)", text)
if summary:
    print(
        f"- Recommendation: close the current Markdown lint backlog of `{summary.group(1)}` errors before treating docs/spec traceability as release-grade."
    )
if zinv:
    print(
        f"- Recommendation: current traceability output reports `{zinv.group(1)}` `ZINV` references in the scanned spec roots; add explicit invariant anchors where security-critical docs are meant to justify code claims."
    )
PY
                printf -- '- Recommendation: if the repository intentionally has no canonical mdBook, scope that expectation explicitly in the L0 gate; otherwise add the missing book root so strict L0 runs do not fail on absent doc topology.\n'
                ;;
              l3-verify-fast)
                printf -- '- Recommendation: rerun the exact failing ignored test and fix its statistical assumption before any broader rerun: `cargo test -p z00z_crypto --release --test test_h2scalar -- --ignored --exact test_h2scalar_distribution`.\n'
                printf -- '- Recommendation: inspect `crates/z00z_crypto/tests/test_h2scalar.rs:119` and the associated bucket-threshold math; the current report proves the failure is concrete, not a reporting artifact.\n'
                ;;
              l4-supply-chain)
                if [[ -f "$RUN_ROOT/supply-chain/supply-chain-summary.json" ]]; then
                  python3 - "$RUN_ROOT/supply-chain/supply-chain-summary.json" <<'PY'
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
project = data.get("project", {})
vendor = data.get("vendor", {})
mixed = data.get("mixed", {})
print(
    f"- Recommendation: close `{project.get('unreviewed', 0)}` project-owned unresolved advisories before treating supply-chain review as attestable."
)
if vendor.get("unreviewed", 0) or mixed.get("unreviewed", 0):
    print(
        f"- Recommendation: separately review `{vendor.get('unreviewed', 0)}` vendor and `{mixed.get('unreviewed', 0)}` mixed advisories; wrapper, pin, or upstream actions are acceptable, direct vendor edits are not."
    )
PY
                fi
                printf -- '- Recommendation: shrink or justify every cargo-vet bootstrap exemption in `%s/.cache/supply-chain/cargo-vet` because the gate log shows vet trust is not yet mature enough to treat as settled.\n' "${RUN_ROOT#"$ROOT_DIR"/}"
                ;;
              l4-adversarial-review)
                if [[ -f "$SECURITY_BRAINSTORM_SUMMARY_PATH" ]]; then
                  python3 - "$SECURITY_BRAINSTORM_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
top = data.get("top_findings", [])[:3]
print(
    f"- Recommendation: route `{data.get('high_risk_count', 0)}` high-risk hypotheses into manual crypto review, starting from the top cross-crate scenarios listed below."
)
for item in top:
    evidence = ", ".join(item.get("evidence_files", [])[:3]) or "-"
    print(
        f"- Recommendation: review `{item.get('title', 'untitled')}` against `{evidence}` and either add a formal claim/harness or document why the scenario is impossible."
    )
PY
                fi
                printf -- '- Recommendation: treat this gate as attack-surface generation, not exploit proof; only promote a scenario after a follow-up artifact closes the gap with a concrete claim, harness, or proof.\n'
                ;;
              *)
                if [[ "$status" == "UNKNOWN" ]]; then
                  printf -- '- Recommendation: convert this gate from `UNKNOWN` to evidence by installing the missing tool or adding the missing model/harness/spec referenced by the anchor log.\n'
                elif [[ -n "${GATE_LEAKS[$gate_id]:-}" ]]; then
                  printf -- '- Recommendation: remove the unauthorized runtime output path(s) recorded in the gate log and keep all temporary state under the active run root.\n'
                else
                  printf -- '- Recommendation: use the anchor log and artifacts above to close the exact blocker before rerunning the full project pipeline.\n'
                fi
                ;;
            esac
            printf '\n'
            ;;
        esac
      done
    fi

    printf '### Global Actions\n\n'
    if [[ "$fail_gate_count" -eq 0 && "$unknown_gate_count" -eq 0 && "$human_review_gate_count" -eq 0 && "$leak_gate_count" -eq 0 && "$vendor_count" -eq 0 && "$COVERAGE_CRATE_UNMAPPED_COUNT" -eq 0 ]]; then
      printf -- '- No blocking remediation actions were synthesized from this pass.\n\n'
    else
      if [[ "$fail_gate_count" -gt 0 ]]; then
        printf -- '- Reproduce and close each `FAIL` gate using its checker module row and problem-evidence section before treating this pass as attestable.\n'
      fi
      if [[ "$unknown_gate_count" -gt 0 ]]; then
        printf -- '- Convert each `UNKNOWN` gate into evidence by installing the missing tool, adding the missing model/harness/spec, or explicitly narrowing scope when the gate is not semantically applicable.\n'
      fi
      if [[ "$human_review_gate_count" -gt 0 ]]; then
        printf -- '- Route all `NEEDS_HUMAN_CRYPTO_REVIEW` findings to manual cryptographic review with the referenced adversarial evidence files, not just reruns.\n'
      fi
      if [[ "$leak_gate_count" -gt 0 ]]; then
        printf -- '- Keep tightening artifact confinement until root-level runtime leaks stay at `0`; moved leak evidence is under `leaks/` for exact offender tracing.\n'
      fi
      if [[ "$vendor_count" -gt 0 ]]; then
        printf -- '- Treat protected vendor findings as wrapper/upstream remediation items only; do not auto-edit vendor code.\n'
      fi
      if [[ "$COVERAGE_CRATE_UNMAPPED_COUNT" -gt 0 ]]; then
        printf -- '- Map every crate-owned tracked file to at least one active gate before claiming crate-complete coverage.\n'
      fi
      if [[ "$PROFILE_TOP_N" -gt 0 && -f "$PROFILE_SUMMARY_PATH" ]]; then
        python3 - "$PROFILE_SUMMARY_PATH" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
recommendations = summary.get("aggregate_recommendations", [])
if recommendations:
    print(
        f"- Profiling shows the slowest top 5% of events consume `{summary.get('slowest_fraction_percent', 0)}%` of measured runtime; prioritize these acceleration steps:"
    )
    for item in recommendations[:4]:
        print(f"  - {item}")
PY
      fi
      printf '\n'
    fi

    printf '## 📝 Execution Notes\n\n'
    if [[ "$MODE" == "report" ]]; then
      printf -- '- This mode did not apply code edits.\n'
    else
      printf -- '- This mode applied only bounded mechanical fixes available to the shell orchestrator.\n'
      printf -- '- Deep semantic remediation remains a follow-up for the active coding session when failing gates persist.\n'
    fi
    if [[ "$DRY_RUN" -eq 0 ]]; then
      printf -- "- Report contract validation summary: \`%s\`\n" "${REPORT_VALIDATION_SUMMARY_PATH#"$ROOT_DIR"/}"
    fi
  } >"$tmp_file"

  if [[ "$DRY_RUN" -eq 0 ]]; then
    safe_write_report "$REPORT_PATH" "$tmp_file"
    python3 "$REPORT_VALIDATOR_SCRIPT" \
      --report "$REPORT_PATH" \
      --run-root "$RUN_ROOT" \
      --root "$ROOT_DIR" \
      --scope-kind "$SCOPE_KIND" \
      --format-path "${REPORT_FORMAT_PATH#"$ROOT_DIR"/}" \
      --summary-out "$REPORT_VALIDATION_SUMMARY_PATH"
  else
    cat "$tmp_file"
    rm "$tmp_file"
  fi
}

main() {
  parse_mode_and_scope "$@"
  parse_flags
  resolve_scope
  infer_levels

  if [[ "${#SELECTED_LEVELS[@]}" -eq 0 ]]; then
    die "no levels selected for scope $SCOPE_KIND"
  fi

  init_report_state
  z00z_profile_activate_tool_env "$ROOT_DIR"
  sanitize_preexisting_root_runtime_leaks

  case "$MODE" in
    report)
      prepare_runtime_artifacts || true
      execute_selected_gates || true
      write_report
      ;;
    fix|find-and-fix)
      bootstrap_missing_artifacts || true
      local pass=0
      execute_selected_gates || true
      while [[ "$pass" -lt "$MAX_FIX_PASSES" ]]; do
        local has_fail=0
        local gate_id
        for gate_id in "${!GATE_STATUS[@]}"; do
          if [[ "${GATE_STATUS[$gate_id]}" == "FAIL" ]]; then
            has_fail=1
            break
          fi
        done
        [[ "$has_fail" -eq 1 ]] || break
        apply_mechanical_fixes || true
        pass=$((pass + 1))
        execute_selected_gates || true
      done
      write_report
      ;;
    *)
      die "unsupported mode: $MODE"
      ;;
  esac

  if [[ "$OVERALL_STATUS" == "FAIL" || "$OVERALL_STATUS" == "UNKNOWN" || "$OVERALL_STATUS" == "NEEDS_HUMAN_CRYPTO_REVIEW" || "$OVERALL_STATUS" == "DRY-RUN" ]]; then
    exit 1
  fi
}

main "$@"
