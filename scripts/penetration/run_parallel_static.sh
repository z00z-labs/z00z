#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/penetration/common.sh
source "$ROOT/scripts/penetration/common.sh"

MODE="standard"
ARTIFACT_DIR=""
SCOPE_JSON=""
TOOL_STATUS=""
PROFILE="generic"

usage() {
  cat <<'EOF'
Usage: run_parallel_static.sh --artifact-dir <path> --scope-json <path> --tool-status <path> [--mode quick|standard|deep]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --scope-json)
      SCOPE_JSON="$2"
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

if [[ -z "$ARTIFACT_DIR" || -z "$SCOPE_JSON" || -z "$TOOL_STATUS" ]]; then
  echo "ERROR: --artifact-dir, --scope-json, and --tool-status are required" >&2
  exit 1
fi

case "$PROFILE" in
  generic|z00z) ;;
  *)
    echo "ERROR: unsupported profile: $PROFILE" >&2
    exit 1
    ;;
esac

mkdir -p "$ARTIFACT_DIR/normalized"

scripts=(
  "source-sast:$ROOT/scripts/penetration/run_source_sast.sh"
  "rust-security:$ROOT/scripts/penetration/run_rust_security.sh"
  "secrets-supply-chain:$ROOT/scripts/penetration/run_secrets_supply_chain.sh"
)

names=()
pids=()

for entry in "${scripts[@]}"; do
  IFS=':' read -r lane_name lane_script <<<"$entry"
  names+=("$lane_name")
  if [[ "$lane_name" == "source-sast" ]]; then
    bash "$lane_script" --artifact-dir "$ARTIFACT_DIR" --scope-json "$SCOPE_JSON" --tool-status "$TOOL_STATUS" --mode "$MODE" --profile "$PROFILE" &
  else
    bash "$lane_script" --artifact-dir "$ARTIFACT_DIR" --tool-status "$TOOL_STATUS" --mode "$MODE" --profile "$PROFILE" &
  fi
  pids+=("$!")
done

child_args=()
child_failures=0

for index in "${!pids[@]}"; do
  pid="${pids[$index]}"
  lane_name="${names[$index]}"
  set +e
  wait "$pid"
  child_exit=$?
  set -e
  child_args+=("$lane_name=$child_exit")
  if [[ $child_exit -ne 0 ]]; then
    child_failures=1
  fi
done

python3 - "$ARTIFACT_DIR/normalized/static-orchestration.json" "${child_args[@]}" <<'PY'
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

target = Path(sys.argv[1])
children = []
for entry in sys.argv[2:]:
    lane, exit_code = entry.split("=", 1)
    children.append({"lane": lane, "exit_code": int(exit_code)})

payload = {
    "version": 1,
    "lane": "static-orchestration",
    "children": children,
    "produced_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}
target.parent.mkdir(parents=True, exist_ok=True)
if target.exists():
    shutil.copy2(target, target.with_suffix(target.suffix + ".bak"))
target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if [[ $child_failures -ne 0 ]]; then
  exit 1
fi
