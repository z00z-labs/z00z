#!/bin/bash

# Report unsafe Rust usage in project-owned code and dependencies.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROFILE_LIB="$ROOT_DIR/.github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh"
FEATURE_FLAG="${Z00Z_ALL_FEATURES_FLAG---all-features}"
PROFILE_ARGS_TEXT="${Z00Z_CARGO_PROFILE_ARGS:---release}"
INCLUDE_VENDOR="${Z00Z_INCLUDE_VENDOR:-0}"
INCLUDE_DEPENDENCIES="${Z00Z_UNSAFE_INCLUDE_DEPENDENCIES:-0}"
GEIGER_INCLUDE_VENDOR="${Z00Z_GEIGER_INCLUDE_VENDOR:-0}"
WRITE_VENDOR_REPORT=0
SKIP_INLINE_SCAN=0
VENDOR_ROOT="${Z00Z_VENDOR_ROOT:-crates/z00z_crypto/tari}"
REPORT_STAMP="${Z00Z_REPORT_TIMESTAMP:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_ROOT="${Z00Z_VERIFICATION_RUN_ROOT:-$ROOT_DIR/reports/z00z-verification-orchestrator-$REPORT_STAMP}"
DEFAULT_VENDOR_REPORT_PATH="$RUN_ROOT/vendor/vendor-unsafe.md"
DEFAULT_GEIGER_TARGET_ROOT="$RUN_ROOT/geiger/target"
VENDOR_REPORT_PATH="${Z00Z_VENDOR_UNSAFE_REPORT:-$DEFAULT_VENDOR_REPORT_PATH}"
GEIGER_TARGET_ROOT="${Z00Z_GEIGER_TARGET_ROOT:-$DEFAULT_GEIGER_TARGET_ROOT}"
GEIGER_PACKAGES="${Z00Z_GEIGER_PACKAGES:-z00z_core z00z_crypto z00z_storage z00z_wallets z00z_validators}"

source "$PROFILE_LIB"
z00z_profile_activate_tool_env "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage: unsafe-report.sh [OPTIONS]

Options:
  --project-only          Scan project-owned Rust code only (default).
  --include-vendor        Include vendored Rust code in the inline unsafe scan.
  --include-dependencies  Run cargo-geiger dependency scan when installed.
  --vendor-report [path]  Generate a fact-based vendor unsafe Markdown report.
  --vendor-report-only [path]
                          Generate only the vendor unsafe Markdown report.
  --all                   Include vendor, dependencies, and vendor report.
  -h, --help              Show this help.

Environment:
  Z00Z_VENDOR_ROOT                 Vendor root. Default: crates/z00z_crypto/tari.
  Z00Z_VENDOR_UNSAFE_REPORT        Vendor report path. Default: reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/vendor/vendor-unsafe.md.
  Z00Z_INCLUDE_VENDOR=1            Include vendor in inline scan.
  Z00Z_UNSAFE_INCLUDE_DEPENDENCIES=1 Include dependency scan.
  Z00Z_GEIGER_INCLUDE_VENDOR=1     Include vendor packages in cargo-geiger package selection.
  Z00Z_CARGO_PROFILE_ARGS          Preferred cargo profile args. Default: --release when supported.
EOF
}

resolve_root_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --project-only)
      INCLUDE_VENDOR=0
      INCLUDE_DEPENDENCIES=0
      shift
      ;;
    --include-vendor)
      INCLUDE_VENDOR=1
      shift
      ;;
    --include-dependencies)
      INCLUDE_DEPENDENCIES=1
      shift
      ;;
    --vendor-report)
      WRITE_VENDOR_REPORT=1
      if [[ $# -gt 1 && "${2:0:1}" != "-" ]]; then
        VENDOR_REPORT_PATH="$2"
        shift 2
      else
        shift
      fi
      ;;
    --vendor-report-only)
      WRITE_VENDOR_REPORT=1
      SKIP_INLINE_SCAN=1
      if [[ $# -gt 1 && "${2:0:1}" != "-" ]]; then
        VENDOR_REPORT_PATH="$2"
        shift 2
      else
        shift
      fi
      ;;
    --all)
      INCLUDE_VENDOR=1
      INCLUDE_DEPENDENCIES=1
      WRITE_VENDOR_REPORT=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

cd "$ROOT_DIR"
VENDOR_REPORT_PATH="$(resolve_root_path "$VENDOR_REPORT_PATH")"
GEIGER_TARGET_ROOT="$(resolve_root_path "$GEIGER_TARGET_ROOT")"

gate_failed=0

log() {
  printf '[z00z-l4:unsafe] %s\n' "$1"
}

feature_args=()
if [[ -n "$FEATURE_FLAG" ]]; then
  feature_args+=("$FEATURE_FLAG")
fi

profile_args=()
if [[ -n "$PROFILE_ARGS_TEXT" ]]; then
  read -r -a profile_args <<<"$PROFILE_ARGS_TEXT"
fi

geiger_profile_args=()
if cargo geiger --help 2>/dev/null | grep -Eq -- '--release|--profile'; then
  geiger_profile_args=("${profile_args[@]}")
fi

collect_geiger_packages() {
  if [[ "$GEIGER_PACKAGES" != "workspace" ]]; then
    python3 -c '
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1]).resolve()
vendor = pathlib.Path(sys.argv[2])
if not vendor.is_absolute():
    vendor = (root / vendor).resolve()
include_vendor = sys.argv[3] == "1"
wanted = {item for item in sys.argv[4].split() if item}
data = json.load(sys.stdin)
for package in sorted(data.get("packages", []), key=lambda item: item["name"]):
    if package["name"] not in wanted:
        continue
    manifest = pathlib.Path(package["manifest_path"]).resolve()
    if not include_vendor:
        try:
            manifest.relative_to(vendor)
        except ValueError:
            pass
        else:
            continue
    print("{}\t{}".format(package["name"], manifest))
 ' "$ROOT_DIR" "$VENDOR_ROOT" "$GEIGER_INCLUDE_VENDOR" "$GEIGER_PACKAGES"
    return 0
  fi

  python3 -c '
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1]).resolve()
vendor = pathlib.Path(sys.argv[2])
if not vendor.is_absolute():
    vendor = (root / vendor).resolve()
include_vendor = sys.argv[3] == "1"
data = json.load(sys.stdin)
for package in sorted(data.get("packages", []), key=lambda item: item["name"]):
    manifest = pathlib.Path(package["manifest_path"]).resolve()
    if not include_vendor:
        try:
            manifest.relative_to(vendor)
        except ValueError:
            pass
        else:
            continue
    print("{}\t{}".format(package["name"], manifest))
 ' "$ROOT_DIR" "$VENDOR_ROOT" "$GEIGER_INCLUDE_VENDOR"
}

sanitize_target_name() {
  printf '%s\n' "$1" | tr '/[:space:]' '__' | tr -cd '[:alnum:]_.-'
}

if [[ "$SKIP_INLINE_SCAN" != "1" && "$INCLUDE_DEPENDENCIES" == "1" ]]; then
  if command -v cargo-geiger >/dev/null 2>&1 || cargo geiger --version >/dev/null 2>&1; then
    log "cargo geiger per workspace package"
    mkdir -p "$GEIGER_TARGET_ROOT"
    mapfile -t geiger_packages < <(cargo metadata --no-deps --format-version 1 | collect_geiger_packages)
    if [[ "${#geiger_packages[@]}" -eq 0 ]]; then
      log "UNKNOWN: no workspace packages resolved for cargo geiger"
    fi
    for package_info in "${geiger_packages[@]}"; do
      IFS=$'\t' read -r package_name package_manifest <<< "$package_info"
      log "cargo geiger package $package_name"
      geiger_log="$(mktemp "${TMPDIR:-/tmp}/z00z-geiger-$(sanitize_target_name "$package_name").XXXXXX")"
      geiger_target_dir="$GEIGER_TARGET_ROOT/$(sanitize_target_name "$package_name")"
      if z00z_profile_run_command command "geiger:$package_name" env CARGO_TARGET_DIR="$geiger_target_dir" cargo geiger --manifest-path "$package_manifest" "${geiger_profile_args[@]}" "${feature_args[@]}" >"$geiger_log" 2>&1; then
        if grep -Eq 'Failed to parse file|Failed to match|warning:' "$geiger_log"; then
          sed -n '1,40p' "$geiger_log"
          log "NOTE: cargo geiger reported warnings or parser limits for $package_name"
        else
          geiger_summary="$(grep -E '^[0-9]+/[0-9]+' "$geiger_log" | tail -n 1 || true)"
          if [[ -n "$geiger_summary" ]]; then
            log "cargo geiger summary: $geiger_summary"
          else
            log "cargo geiger completed for $package_name"
          fi
        fi
      else
        sed -n '1,40p' "$geiger_log"
        log "NOTE: cargo geiger returned non-zero for $package_name"
      fi
      rm -f "$geiger_log"
    done
  else
    log "NOTE: cargo-geiger not installed; dependency unsafe coverage unavailable; continuing with inline Rust unsafe scan"
  fi
fi

if [[ "$SKIP_INLINE_SCAN" != "1" ]]; then
  if [[ "$INCLUDE_DEPENDENCIES" == "1" ]]; then
    log "inline Rust unsafe scan after dependency gate"
  elif [[ "$INCLUDE_VENDOR" == "1" ]]; then
    log "inline Rust unsafe scan including vendor"
  else
    log "project-owned unsafe scan"
  fi

  if [[ "$INCLUDE_VENDOR" == "1" ]]; then
    rg -n "\\bunsafe\\b" crates -g '*.rs' || true
  else
    vendor_glob="$VENDOR_ROOT"
    case "$vendor_glob" in
      "$ROOT_DIR"/*)
        vendor_glob="${vendor_glob#"$ROOT_DIR"/}"
        ;;
    esac
    vendor_glob="${vendor_glob%/}/**"
    rg -n "\\bunsafe\\b" crates -g '*.rs' -g "!$vendor_glob" || true
  fi
fi

if [[ "$WRITE_VENDOR_REPORT" == "1" ]]; then
  if ! z00z_profile_run_command command "vendor-unsafe-report" python3 "$SCRIPT_DIR/vendor-unsafe-report.py" \
    --vendor-root "$VENDOR_ROOT" \
    --output "$VENDOR_REPORT_PATH"; then
    gate_failed=1
  fi
fi

if [[ "$gate_failed" -eq 0 ]]; then
  log "TESTED: unsafe inventory completed successfully"
fi

exit "$gate_failed"
