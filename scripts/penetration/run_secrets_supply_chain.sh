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
Usage: run_secrets_supply_chain.sh --artifact-dir <path> --tool-status <path> [--mode quick|standard|deep] [--profile generic|z00z]
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
  "$ARTIFACT_DIR/secrets" \
  "$ARTIFACT_DIR/logs" \
  "$ARTIFACT_DIR/raw/secrets" \
  "$ARTIFACT_DIR/raw/supply-chain" \
  "$ARTIFACT_DIR/normalized"

tool_timeout_seconds() {
  case "$MODE" in
    quick) printf '%s\n' "120" ;;
    standard) printf '%s\n' "300" ;;
    deep) printf '%s\n' "900" ;;
  esac
}

TOOL_TIMEOUT_SECONDS="$(tool_timeout_seconds)"

write_missing_tool() {
  local tool_name="$1"
  pen_write_status_json \
    "$ARTIFACT_DIR/normalized/secrets.${tool_name}.status.json" \
    "secrets" \
    "$tool_name" \
    "missing" \
    127 \
    "" \
    "" \
    "" \
    "" \
    "" \
    "$tool_name is not available in tools/penetration or PATH"
}

gitleaks_path="$(pen_tool_field "$TOOL_STATUS" gitleaks resolved_path)"
gitleaks_state="$(pen_tool_field "$TOOL_STATUS" gitleaks status)"
if [[ -z "$gitleaks_path" || "$gitleaks_state" != "present" ]]; then
  write_missing_tool "gitleaks"
else
  raw_path="$ARTIFACT_DIR/raw/secrets/secrets.gitleaks.json"
  PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
    "$ARTIFACT_DIR" \
    "secrets" \
    "gitleaks" \
    "$ROOT" \
    "$raw_path" \
    "$ARTIFACT_DIR/logs/secrets.gitleaks.out" \
    "$ARTIFACT_DIR/logs/secrets.gitleaks.err" \
    "$ARTIFACT_DIR/logs/secrets.gitleaks.exit" \
    "$ARTIFACT_DIR/normalized/secrets.gitleaks.status.json" \
    "$gitleaks_path" detect --no-banner --exit-code 0 --source "$ROOT" --report-format json --report-path "$raw_path" >/dev/null
fi

trufflehog_path="$(pen_tool_field "$TOOL_STATUS" trufflehog resolved_path)"
trufflehog_state="$(pen_tool_field "$TOOL_STATUS" trufflehog status)"
if [[ -z "$trufflehog_path" || "$trufflehog_state" != "present" ]]; then
  write_missing_tool "trufflehog"
else
  raw_path="$ARTIFACT_DIR/raw/secrets/secrets.trufflehog.json"
  PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
    "$ARTIFACT_DIR" \
    "secrets" \
    "trufflehog" \
    "$ROOT" \
    "$raw_path" \
    "$raw_path" \
    "$ARTIFACT_DIR/logs/secrets.trufflehog.err" \
    "$ARTIFACT_DIR/logs/secrets.trufflehog.exit" \
    "$ARTIFACT_DIR/normalized/secrets.trufflehog.status.json" \
    "$trufflehog_path" filesystem --no-update --json --no-verification "$ROOT" >/dev/null
fi

trivy_path="$(pen_tool_field "$TOOL_STATUS" trivy resolved_path)"
trivy_state="$(pen_tool_field "$TOOL_STATUS" trivy status)"
if [[ -z "$trivy_path" || "$trivy_state" != "present" ]]; then
  write_missing_tool "trivy"
else
  raw_path="$ARTIFACT_DIR/raw/supply-chain/secrets.trivy.json"
  PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
    "$ARTIFACT_DIR" \
    "secrets" \
    "trivy" \
    "$ROOT" \
    "$raw_path" \
    "$ARTIFACT_DIR/logs/secrets.trivy.out" \
    "$ARTIFACT_DIR/logs/secrets.trivy.err" \
    "$ARTIFACT_DIR/logs/secrets.trivy.exit" \
    "$ARTIFACT_DIR/normalized/secrets.trivy.status.json" \
    "$trivy_path" fs --quiet --format json --output "$raw_path" "$ROOT" >/dev/null
fi

mapfile -t secrets_statuses < <(python3 - "$ARTIFACT_DIR/normalized" <<'PY'
import json
import sys
from pathlib import Path

for path in sorted(Path(sys.argv[1]).glob("secrets.*.status.json")):
    payload = json.loads(path.read_text(encoding="utf-8"))
    print(payload.get("status", "failed"))
PY
)

rollup_status="$(pen_status_rollup "${secrets_statuses[@]}")"
rollup_summary="secrets and supply-chain commands completed"
if [[ "$rollup_status" == "completed-with-missing-tools" ]]; then
  rollup_summary="secrets and supply-chain completed with missing tools"
elif [[ "$rollup_status" == "completed-with-failures" ]]; then
  rollup_summary="secrets and supply-chain recorded tool failures"
fi

pen_write_lane_summary \
  "$ARTIFACT_DIR/secrets/summary.json" \
  "secrets" \
  "$rollup_status" \
  "$rollup_summary" \
  "normalized/secrets.gitleaks.status.json" \
  "normalized/secrets.trufflehog.status.json" \
  "normalized/secrets.trivy.status.json"
