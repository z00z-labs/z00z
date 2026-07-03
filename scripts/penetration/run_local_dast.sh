#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/penetration/common.sh
source "$ROOT/scripts/penetration/common.sh"

MODE="standard"
PROFILE="generic"
ARTIFACT_DIR=""
SCOPE_PATH="$ROOT/.security/scope.yaml"
TOOL_STATUS=""
SKIP_REASON=""

HTTPX_PATHS="/,/health,/readyz,/metrics"
FFUF_MATCH_CODES="200,204,301,302,307,401,403,405"

usage() {
  cat <<'EOF'
Usage: run_local_dast.sh --artifact-dir <path> --scope <path> --tool-status <path> [--mode quick|standard|deep] [--profile generic|z00z] [--skip-reason <text>]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --scope)
      SCOPE_PATH="$2"
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
    --skip-reason)
      SKIP_REASON="$2"
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
  "$ARTIFACT_DIR/dast" \
  "$ARTIFACT_DIR/logs" \
  "$ARTIFACT_DIR/raw" \
  "$ARTIFACT_DIR/normalized"

safe_label() {
  local value="$1"
  value="$(printf '%s' "$value" | tr -cs 'A-Za-z0-9' '_')"
  value="${value##_}"
  value="${value%%_}"
  if [[ -z "$value" ]]; then
    value="target"
  fi
  printf '%s\n' "$value"
}

append_status() {
  local status_path="$1"
  local status_name="$2"
  status_files+=("$(pen_rel_path "$ARTIFACT_DIR" "$status_path")")
  status_names+=("$status_name")
}

static_status_name() {
  local exit_code="$1"
  if [[ "$exit_code" -eq 0 ]]; then
    printf '%s\n' "passed"
  else
    printf '%s\n' "failed"
  fi
}

write_passive_status() {
  local status_path="$1"
  local tool_name="$2"
  local status_name="$3"
  local exit_code="$4"
  local summary_text="$5"

  pen_write_status_json \
    "$status_path" \
    "dast" \
    "$tool_name" \
    "$status_name" \
    "$exit_code" \
    "" \
    "" \
    "" \
    "" \
    "" \
    "$summary_text"
  append_status "$status_path" "$status_name"
}

tool_path_for() {
  pen_tool_field "$TOOL_STATUS" "$1" resolved_path
}

tool_state_for() {
  pen_tool_field "$TOOL_STATUS" "$1" status
}

if [[ -n "$SKIP_REASON" ]]; then
  pen_write_json_artifact \
    "$ARTIFACT_DIR/dast/skipped.json" \
    "dast-skip" \
    "skipped" \
    "$SKIP_REASON" \
    "{\"profile\": \"$PROFILE\", \"mode\": \"$MODE\"}"
  pen_write_lane_summary \
    "$ARTIFACT_DIR/dast/summary.json" \
    "dast" \
    "skipped" \
    "$SKIP_REASON" \
    "dast/skipped.json"
  exit 0
fi

set +e
python3 "$ROOT/scripts/penetration/validate_scope.py" \
  "$SCOPE_PATH" \
  --denylist "$ROOT/.security/denied-tools.txt" \
  --require-dast-targets \
  --json >"$ARTIFACT_DIR/dast/scope-validation.json"
scope_exit=$?
set -e

if [[ $scope_exit -eq 3 ]]; then
  pen_write_json_artifact \
    "$ARTIFACT_DIR/dast/skipped.json" \
    "dast-skip" \
    "skipped" \
    "no allowed local DAST target is present" \
    "{\"validation_path\": \"dast/scope-validation.json\"}"
  pen_write_lane_summary \
    "$ARTIFACT_DIR/dast/summary.json" \
    "dast" \
    "skipped" \
    "no allowed local DAST target is present" \
    "dast/scope-validation.json" \
    "dast/skipped.json"
  exit 0
fi

if [[ $scope_exit -ne 0 ]]; then
  pen_write_json_artifact \
    "$ARTIFACT_DIR/dast/validation-failed.json" \
    "dast-validation" \
    "failed" \
    "scope validation failed before any DAST command executed" \
    "{\"validation_path\": \"dast/scope-validation.json\"}"
  exit 1
fi

hosts_file="$ARTIFACT_DIR/dast/hosts.txt"
urls_file="$ARTIFACT_DIR/dast/urls.txt"
pen_scope_values "$ARTIFACT_DIR/dast/scope-validation.json" normalized_hosts >"$hosts_file"
pen_scope_values "$ARTIFACT_DIR/dast/scope-validation.json" normalized_urls >"$urls_file"

request_rate="$(pen_scope_rate_limit "$SCOPE_PATH" requests_per_second 2)"
max_concurrency="$(pen_scope_rate_limit "$SCOPE_PATH" max_concurrency 2)"
timeout_seconds="$(pen_scope_rate_limit "$SCOPE_PATH" timeout_seconds 30)"
service_timeout_seconds=$((timeout_seconds * 6))
if [[ $service_timeout_seconds -lt 60 ]]; then
  service_timeout_seconds=60
fi

status_files=()
status_names=()
has_runnable_tool=0

run_list_tool() {
  local tool_name="$1"
  local raw_name="$2"
  local target_file="$3"
  shift 3

  local tool_path
  local tool_state
  local status_path
  local stdout_path
  local stderr_path
  local exit_path
  local raw_path

  status_path="$ARTIFACT_DIR/normalized/dast.${tool_name}.status.json"
  stdout_path="$ARTIFACT_DIR/logs/${tool_name}.out"
  stderr_path="$ARTIFACT_DIR/logs/${tool_name}.err"
  exit_path="$ARTIFACT_DIR/logs/${tool_name}.exit"
  raw_path="$ARTIFACT_DIR/raw/$raw_name"

  if [[ ! -s "$target_file" ]]; then
    write_passive_status "$status_path" "$tool_name" "skipped" 0 "$tool_name has no scoped target list to scan"
    return
  fi

  tool_path="$(pen_tool_field "$TOOL_STATUS" "$tool_name" resolved_path)"
  tool_state="$(pen_tool_field "$TOOL_STATUS" "$tool_name" status)"

  if [[ -z "$tool_path" || "$tool_state" != "present" ]]; then
    write_passive_status "$status_path" "$tool_name" "missing" 127 "$tool_name is not available in tools/penetration"
    return
  fi

  has_runnable_tool=1
  local tool_exit
  tool_exit="$(pen_capture_command \
    "$ARTIFACT_DIR" \
    "dast" \
    "$tool_name" \
    "$ROOT" \
    "$raw_path" \
    "$stdout_path" \
    "$stderr_path" \
    "$exit_path" \
    "$status_path" \
    "$tool_path" "$@")"
  append_status "$status_path" "$(static_status_name "$tool_exit")"
}

extract_nmap_targets() {
  local gnmap_path="$1"
  python3 - "$gnmap_path" <<'PY'
import re
import sys
from pathlib import Path

gnmap_path = Path(sys.argv[1])
if not gnmap_path.exists():
    raise SystemExit(0)

for line in gnmap_path.read_text(encoding="utf-8", errors="ignore").splitlines():
    if "Ports:" not in line or not line.startswith("Host: "):
        continue
    host = line.split()[1]
    ports = []
    for chunk in line.split("Ports:", 1)[1].split(","):
        match = re.match(r"\s*(\d+)/open/", chunk)
        if match:
            ports.append(match.group(1))
    if ports:
        print(f"{host}\t{','.join(ports)}")
PY
}

write_ffuf_wordlist() {
  local wordlist_path="$1"
  cat >"$wordlist_path" <<'EOF'
admin
login
api
health
metrics
readyz
debug
docs
EOF
}

run_nmap() {
  local tool_name="nmap"
  local status_path="$ARTIFACT_DIR/normalized/dast.nmap.discovery.status.json"
  local stdout_path="$ARTIFACT_DIR/logs/nmap.discovery.out"
  local stderr_path="$ARTIFACT_DIR/logs/nmap.discovery.err"
  local exit_path="$ARTIFACT_DIR/logs/nmap.discovery.exit"
  local raw_prefix="$ARTIFACT_DIR/raw/nmap.discovery"
  local raw_path="${raw_prefix}.gnmap"
  local tool_path
  local tool_state
  local tool_exit
  local found_service_targets=0

  if [[ ! -s "$hosts_file" ]]; then
    write_passive_status "$status_path" "$tool_name" "skipped" 0 "nmap has no scoped host list to scan"
    return
  fi

  tool_path="$(tool_path_for "$tool_name")"
  tool_state="$(tool_state_for "$tool_name")"
  if [[ -z "$tool_path" || "$tool_state" != "present" ]]; then
    write_passive_status "$status_path" "$tool_name" "missing" 127 "nmap is not available in tools/penetration"
    return
  fi

  has_runnable_tool=1
  tool_exit="$(pen_capture_command \
    "$ARTIFACT_DIR" \
    "dast" \
    "$tool_name" \
    "$ROOT" \
    "$raw_path" \
    "$stdout_path" \
    "$stderr_path" \
    "$exit_path" \
    "$status_path" \
    "$tool_path" \
    -n \
    -Pn \
    -sT \
    --top-ports 20 \
    --open \
    -T4 \
    --max-retries 1 \
    --host-timeout "${timeout_seconds}s" \
    -iL "$hosts_file" \
    -oA "$raw_prefix")"
  append_status "$status_path" "$(static_status_name "$tool_exit")"
  if [[ "$tool_exit" -ne 0 ]]; then
    return
  fi

  while IFS=$'\t' read -r host ports; do
    local label
    local service_status_path
    local service_stdout_path
    local service_stderr_path
    local service_exit_path
    local service_prefix
    local service_raw_path
    local service_exit

    [[ -z "$host" || -z "$ports" ]] && continue
    found_service_targets=1
    label="$(safe_label "$host")"
    service_status_path="$ARTIFACT_DIR/normalized/dast.nmap.services.${label}.status.json"
    service_stdout_path="$ARTIFACT_DIR/logs/nmap.services.${label}.out"
    service_stderr_path="$ARTIFACT_DIR/logs/nmap.services.${label}.err"
    service_exit_path="$ARTIFACT_DIR/logs/nmap.services.${label}.exit"
    service_prefix="$ARTIFACT_DIR/raw/nmap.services.${label}"
    service_raw_path="${service_prefix}.gnmap"

    service_exit="$(pen_capture_command \
      "$ARTIFACT_DIR" \
      "dast" \
      "$tool_name" \
      "$ROOT" \
      "$service_raw_path" \
      "$service_stdout_path" \
      "$service_stderr_path" \
      "$service_exit_path" \
      "$service_status_path" \
      "$tool_path" \
      -n \
      -Pn \
      -sT \
      -sV \
      -sC \
      -p "$ports" \
      --script-timeout "${timeout_seconds}s" \
      --host-timeout "${service_timeout_seconds}s" \
      -oA "$service_prefix" \
      "$host")"
    append_status "$service_status_path" "$(static_status_name "$service_exit")"
  done < <(extract_nmap_targets "$raw_path")

  if [[ $found_service_targets -eq 0 ]]; then
    write_passive_status \
      "$ARTIFACT_DIR/normalized/dast.nmap.services.status.json" \
      "$tool_name" \
      "skipped" \
      0 \
      "nmap discovery found no open ports to enrich"
  fi
}

run_ffuf() {
  local tool_name="ffuf"
  local tool_path
  local tool_state
  local wordlist_path="$ARTIFACT_DIR/dast/ffuf-wordlist.txt"
  local index=0

  if [[ ! -s "$urls_file" ]]; then
    write_passive_status \
      "$ARTIFACT_DIR/normalized/dast.ffuf.status.json" \
      "$tool_name" \
      "skipped" \
      0 \
      "ffuf has no scoped URL list to scan"
    return
  fi

  tool_path="$(tool_path_for "$tool_name")"
  tool_state="$(tool_state_for "$tool_name")"
  if [[ -z "$tool_path" || "$tool_state" != "present" ]]; then
    write_passive_status \
      "$ARTIFACT_DIR/normalized/dast.ffuf.status.json" \
      "$tool_name" \
      "missing" \
      127 \
      "ffuf is not available in tools/penetration"
    return
  fi

  has_runnable_tool=1
  write_ffuf_wordlist "$wordlist_path"

  while IFS= read -r url; do
    local target_id
    local fuzz_url
    local status_path
    local stdout_path
    local stderr_path
    local exit_path
    local raw_path
    local tool_exit

    [[ -z "$url" ]] && continue
    index=$((index + 1))
    target_id="$(printf '%02d' "$index")"
    fuzz_url="${url%/}/FUZZ"
    status_path="$ARTIFACT_DIR/normalized/dast.ffuf.target${target_id}.status.json"
    stdout_path="$ARTIFACT_DIR/logs/ffuf.target${target_id}.out"
    stderr_path="$ARTIFACT_DIR/logs/ffuf.target${target_id}.err"
    exit_path="$ARTIFACT_DIR/logs/ffuf.target${target_id}.exit"
    raw_path="$ARTIFACT_DIR/raw/ffuf.target${target_id}.json"

    tool_exit="$(pen_capture_command \
      "$ARTIFACT_DIR" \
      "dast" \
      "$tool_name" \
      "$ROOT" \
      "$raw_path" \
      "$stdout_path" \
      "$stderr_path" \
      "$exit_path" \
      "$status_path" \
      "$tool_path" \
      -w "$wordlist_path" \
      -u "$fuzz_url" \
      -mc "$FFUF_MATCH_CODES" \
      -ac \
      -t "$max_concurrency" \
      -rate "$request_rate" \
      -timeout "$timeout_seconds" \
      -noninteractive \
      -of json \
      -o "$raw_path")"
    append_status "$status_path" "$(static_status_name "$tool_exit")"
  done < "$urls_file"
}

run_nmap
run_list_tool \
  httpx \
  httpx.jsonl \
  "$urls_file" \
  -l "$urls_file" \
  -sc \
  -title \
  -server \
  -td \
  -fr \
  -path "$HTTPX_PATHS" \
  -timeout "$timeout_seconds" \
  -retries 1 \
  -rl "$request_rate" \
  -t "$max_concurrency" \
  -silent \
  -json \
  -o "$ARTIFACT_DIR/raw/httpx.jsonl"
run_list_tool \
  nuclei \
  nuclei.jsonl \
  "$urls_file" \
  -l "$urls_file" \
  -t http/ \
  -tags tech,misconfig,exposure \
  -severity low,medium,high,critical \
  -ni \
  -rl "$request_rate" \
  -c "$max_concurrency" \
  -bs "$max_concurrency" \
  -timeout "$timeout_seconds" \
  -retries 1 \
  -jsonl \
  -o "$ARTIFACT_DIR/raw/nuclei.jsonl"

if [[ "$MODE" != "quick" ]]; then
  run_list_tool \
    katana \
    katana.jsonl \
    "$urls_file" \
    -list "$urls_file" \
    -d 3 \
    -jc \
    -kf robotstxt \
    -c "$max_concurrency" \
    -p "$max_concurrency" \
    -rl "$request_rate" \
    -timeout "$timeout_seconds" \
    -retry 1 \
    -ef png,jpg,jpeg,gif,svg,css,woff,woff2,ttf,eot,map \
    -silent \
    -j \
    -o "$ARTIFACT_DIR/raw/katana.jsonl"
  run_ffuf
else
  write_passive_status \
    "$ARTIFACT_DIR/normalized/dast.katana.status.json" \
    "katana" \
    "skipped" \
    0 \
    "katana is skipped in quick mode"
  write_passive_status \
    "$ARTIFACT_DIR/normalized/dast.ffuf.status.json" \
    "ffuf" \
    "skipped" \
    0 \
    "ffuf is skipped in quick mode"
fi

if [[ $has_runnable_tool -eq 0 ]]; then
  pen_write_json_artifact \
    "$ARTIFACT_DIR/dast/skipped.json" \
    "dast-skip" \
    "skipped" \
    "no runnable bounded local DAST tool is available for the current scope" \
    "{\"validation_path\": \"dast/scope-validation.json\"}"
  status_files+=("dast/skipped.json")
  status_names+=("skipped")
fi

rollup_status="$(pen_status_rollup "${status_names[@]}")"
rollup_summary="bounded local DAST completed"
if [[ "$rollup_status" == "completed-with-missing-tools" ]]; then
  rollup_summary="bounded local DAST completed with missing tools"
elif [[ "$rollup_status" == "completed-with-failures" ]]; then
  rollup_summary="bounded local DAST recorded tool failures"
elif [[ "$rollup_status" == "skipped" ]]; then
  rollup_summary="bounded local DAST was skipped"
fi

pen_write_lane_summary \
  "$ARTIFACT_DIR/dast/summary.json" \
  "dast" \
  "$rollup_status" \
  "$rollup_summary" \
  "${status_files[@]}"
