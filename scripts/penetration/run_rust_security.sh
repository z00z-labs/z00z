#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/penetration/common.sh
source "$ROOT/scripts/penetration/common.sh"

MODE="standard"
ARTIFACT_DIR=""
TOOL_STATUS=""
PROFILE="generic"

usage() {
  cat <<'EOF'
Usage: run_rust_security.sh --artifact-dir <path> --tool-status <path> [--mode quick|standard|deep] [--profile generic|z00z]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --tool-status)
      TOOL_STATUS="$2"
      shift 2
      ;;
    --mode)
      MODE="$2"
      shift 2
      ;;
    --profile)
      PROFILE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$ARTIFACT_DIR" || -z "$TOOL_STATUS" ]]; then
  echo "ERROR: --artifact-dir and --tool-status are required" >&2
  exit 1
fi

case "$MODE" in
  quick|standard|deep) ;;
  *)
    echo "ERROR: unsupported mode: $MODE" >&2
    exit 1
    ;;
esac

case "$PROFILE" in
  generic|z00z) ;;
  *)
    echo "ERROR: unsupported profile: $PROFILE" >&2
    exit 1
    ;;
esac

mkdir -p \
  "$ARTIFACT_DIR/rust" \
  "$ARTIFACT_DIR/logs" \
  "$ARTIFACT_DIR/raw/rust" \
  "$ARTIFACT_DIR/normalized"

tool_timeout_seconds() {
  case "$MODE" in
    quick) printf '%s\n' "120" ;;
    standard) printf '%s\n' "300" ;;
    deep) printf '%s\n' "900" ;;
  esac
}

TOOL_TIMEOUT_SECONDS="$(tool_timeout_seconds)"

run_rust_tool() {
  local tool_name="$1"
  local raw_name="$2"
  shift 2
  local tool_path
  local tool_state
  local status_path
  local stdout_path
  local stderr_path
  local exit_path
  local raw_path

  tool_path="$(pen_tool_field "$TOOL_STATUS" "$tool_name" resolved_path)"
  tool_state="$(pen_tool_field "$TOOL_STATUS" "$tool_name" status)"
  status_path="$ARTIFACT_DIR/normalized/rust.${tool_name}.status.json"
  stdout_path="$ARTIFACT_DIR/logs/${tool_name}.out"
  stderr_path="$ARTIFACT_DIR/logs/${tool_name}.err"
  exit_path="$ARTIFACT_DIR/logs/${tool_name}.exit"
  raw_path="$ARTIFACT_DIR/raw/rust/$raw_name"

  if [[ -z "$tool_path" || "$tool_state" != "present" ]]; then
    pen_write_status_json \
      "$status_path" \
      "rust" \
      "$tool_name" \
      "missing" \
      127 \
      "" \
      "" \
      "" \
      "" \
      "" \
      "$tool_name is not available in tools/penetration or PATH"
    return 0
  fi

  PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
    "$ARTIFACT_DIR" \
    "rust" \
    "$tool_name" \
    "$ROOT" \
    "$raw_path" \
    "$stdout_path" \
    "$stderr_path" \
    "$exit_path" \
    "$status_path" \
    "$tool_path" "$@" >/dev/null
}

run_rust_tool cargo-audit cargo-audit.json audit --json
run_rust_tool cargo-deny cargo-deny.txt check advisories bans sources licenses

if [[ "$MODE" != "quick" ]]; then
  run_rust_tool cargo-geiger cargo-geiger.txt -q
else
  pen_write_status_json \
    "$ARTIFACT_DIR/normalized/rust.cargo-geiger.status.json" \
    "rust" \
    "cargo-geiger" \
    "skipped" \
    0 \
    "" \
    "" \
    "" \
    "" \
    "" \
    "cargo-geiger is skipped in quick mode"
fi

mapfile -t rust_statuses < <(python3 - "$ARTIFACT_DIR/normalized" <<'PY'
import json
import sys
from pathlib import Path

for path in sorted(Path(sys.argv[1]).glob("rust.*.status.json")):
    payload = json.loads(path.read_text(encoding="utf-8"))
    print(payload.get("status", "failed"))
PY
)

rollup_status="$(pen_status_rollup "${rust_statuses[@]}")"
rollup_summary="Rust security commands completed"
if [[ "$rollup_status" == "completed-with-missing-tools" ]]; then
  rollup_summary="Rust security completed with missing tools"
elif [[ "$rollup_status" == "completed-with-failures" ]]; then
  rollup_summary="Rust security recorded tool failures"
fi

pen_write_lane_summary \
  "$ARTIFACT_DIR/rust/summary.json" \
  "rust" \
  "$rollup_status" \
  "$rollup_summary" \
  "normalized/rust.cargo-audit.status.json" \
  "normalized/rust.cargo-deny.status.json" \
  "normalized/rust.cargo-geiger.status.json"
