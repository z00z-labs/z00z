#!/usr/bin/env bash

pen_root() {
  cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd
}

pen_now_iso() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

pen_now_run_id() {
  date -u +"%Y%m%dT%H%M%SZ"
}

pen_rel_path() {
  local artifact_dir="$1"
  local path="$2"
  if [[ -z "$path" ]]; then
    printf '%s' ""
    return
  fi
  if [[ "$path" == "$artifact_dir/"* ]]; then
    printf '%s' "${path#$artifact_dir/}"
    return
  fi
  printf '%s' "$path"
}

pen_tool_field() {
  local tool_status_json="$1"
  local tool_name="$2"
  local field_name="$3"
  python3 - "$tool_status_json" "$tool_name" "$field_name" <<'PY'
import json
import sys
from pathlib import Path

tool_status_path = Path(sys.argv[1])
tool_name = sys.argv[2]
field_name = sys.argv[3]

if not tool_status_path.exists():
    raise SystemExit(0)

payload = json.loads(tool_status_path.read_text(encoding="utf-8"))
for item in payload.get("tools", []):
    if item.get("name") != tool_name:
        continue
    value = item.get(field_name, "")
    if isinstance(value, bool):
        print("true" if value else "false")
    elif value is None:
        print("")
    else:
        print(str(value))
    break
PY
}

pen_scope_values() {
  local scope_json="$1"
  local field_name="$2"
  python3 - "$scope_json" "$field_name" <<'PY'
import json
import sys
from pathlib import Path

scope_path = Path(sys.argv[1])
field_name = sys.argv[2]

if not scope_path.exists():
    raise SystemExit(0)

payload = json.loads(scope_path.read_text(encoding="utf-8"))
for item in payload.get(field_name, []):
    print(item)
PY
}

pen_scope_paths() {
  local scope_json="$1"
  local repo_root="$2"
  python3 - "$scope_json" "$repo_root" <<'PY'
import json
import sys
from pathlib import Path

scope_path = Path(sys.argv[1])
repo_root = Path(sys.argv[2]).resolve()

if not scope_path.exists():
    raise SystemExit(0)

payload = json.loads(scope_path.read_text(encoding="utf-8"))
for raw_path in payload.get("normalized_paths", []):
    candidate = (repo_root / raw_path).resolve()
    print(candidate.as_posix())
PY
}

pen_scope_rate_limit() {
  local scope_yaml="$1"
  local field_name="$2"
  local default_value="$3"
  python3 - "$scope_yaml" "$field_name" "$default_value" <<'PY'
import sys
from pathlib import Path

import yaml

scope_path = Path(sys.argv[1])
field_name = sys.argv[2]
default_value = sys.argv[3]

if not scope_path.exists():
    print(default_value)
    raise SystemExit(0)

payload = yaml.safe_load(scope_path.read_text(encoding="utf-8")) or {}
rate_limits = payload.get("rate_limits", {})
value = rate_limits.get(field_name, default_value)
print(value)
PY
}

pen_status_rollup() {
  local has_passed=0
  local has_failed=0
  local has_missing=0
  local has_skipped=0
  local status

  for status in "$@"; do
    case "$status" in
      passed|completed)
        has_passed=1
        ;;
      failed|completed-with-failures)
        has_failed=1
        ;;
      missing|completed-with-missing-tools)
        has_missing=1
        ;;
      skipped)
        has_skipped=1
        ;;
    esac
  done

  if [[ $has_failed -eq 1 ]]; then
    printf '%s\n' "completed-with-failures"
    return
  fi
  if [[ $has_missing -eq 1 ]]; then
    printf '%s\n' "completed-with-missing-tools"
    return
  fi
  if [[ $has_passed -eq 0 && $has_skipped -eq 1 ]]; then
    printf '%s\n' "skipped"
    return
  fi
  printf '%s\n' "completed"
}

pen_write_status_json() {
  local target_path="$1"
  local lane_name="$2"
  local tool_name="$3"
  local status_name="$4"
  local exit_code="$5"
  local command_display="$6"
  local stdout_rel="$7"
  local stderr_rel="$8"
  local exit_rel="$9"
  local raw_rel="${10}"
  local summary_text="${11}"

  python3 - "$target_path" "$lane_name" "$tool_name" "$status_name" "$exit_code" "$command_display" "$stdout_rel" "$stderr_rel" "$exit_rel" "$raw_rel" "$summary_text" <<'PY'
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

target = Path(sys.argv[1])
payload = {
    "version": 1,
    "lane": sys.argv[2],
    "tool": sys.argv[3],
    "status": sys.argv[4],
    "exit_code": int(sys.argv[5]),
    "command_display": sys.argv[6],
    "stdout_path": sys.argv[7],
    "stderr_path": sys.argv[8],
    "exit_path": sys.argv[9],
    "raw_output_path": sys.argv[10],
    "summary": sys.argv[11],
    "produced_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}
target.parent.mkdir(parents=True, exist_ok=True)
if target.exists():
    shutil.copy2(target, target.with_suffix(target.suffix + ".bak"))
target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

pen_write_lane_summary() {
  local target_path="$1"
  local lane_name="$2"
  local status_name="$3"
  local summary_text="$4"
  shift 4
  python3 - "$target_path" "$lane_name" "$status_name" "$summary_text" "$@" <<'PY'
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

target = Path(sys.argv[1])
payload = {
    "version": 1,
    "lane": sys.argv[2],
    "status": sys.argv[3],
    "summary": sys.argv[4],
    "status_files": list(sys.argv[5:]),
    "produced_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}
target.parent.mkdir(parents=True, exist_ok=True)
if target.exists():
    shutil.copy2(target, target.with_suffix(target.suffix + ".bak"))
target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

pen_write_json_artifact() {
  local target_path="$1"
  local artifact_kind="$2"
  local status_name="$3"
  local summary_text="$4"
  local extra_json="${5:-}"
  if [[ -z "$extra_json" ]]; then
    extra_json='{}'
  fi
  python3 - "$target_path" "$artifact_kind" "$status_name" "$summary_text" "$extra_json" <<'PY'
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

target = Path(sys.argv[1])
payload = {
    "version": 1,
    "artifact": sys.argv[2],
    "status": sys.argv[3],
    "summary": sys.argv[4],
    "produced_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}
payload.update(json.loads(sys.argv[5]))
target.parent.mkdir(parents=True, exist_ok=True)
if target.exists():
    shutil.copy2(target, target.with_suffix(target.suffix + ".bak"))
target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

pen_capture_command() {
  local artifact_dir="$1"
  local lane_name="$2"
  local tool_name="$3"
  local working_dir="$4"
  local raw_path="$5"
  local stdout_path="$6"
  local stderr_path="$7"
  local exit_path="$8"
  local status_path="$9"
  shift 9

  local -a command=("$@")
  local command_display
  local exit_code
  local tool_status="passed"
  local summary_text="command completed"
  local timeout_seconds="${PEN_TIMEOUT_SECONDS:-}"
  local -a run_command=("${command[@]}")

  printf -v command_display '%q ' "${command[@]}"
  command_display="${command_display% }"
  if [[ -n "$timeout_seconds" ]] && command -v timeout >/dev/null 2>&1; then
    run_command=(timeout --signal=TERM --kill-after=10s "${timeout_seconds}s" "${command[@]}")
    printf -v command_display '%q ' "${run_command[@]}"
    command_display="${command_display% }"
  fi

  set +e
  (
    cd "$working_dir"
    "${run_command[@]}" >"$stdout_path" 2>"$stderr_path"
  )
  exit_code=$?
  set -e

  printf '%s\n' "$exit_code" >"$exit_path"

  if [[ $exit_code -eq 124 || $exit_code -eq 137 ]]; then
    tool_status="failed"
    if [[ -n "$timeout_seconds" ]]; then
      summary_text="$tool_name timed out after ${timeout_seconds}s"
    else
      summary_text="$tool_name timed out"
    fi
  elif [[ $exit_code -ne 0 ]]; then
    tool_status="failed"
    summary_text="$tool_name returned a non-zero exit status"
  fi

  pen_write_status_json \
    "$status_path" \
    "$lane_name" \
    "$tool_name" \
    "$tool_status" \
    "$exit_code" \
    "$command_display" \
    "$(pen_rel_path "$artifact_dir" "$stdout_path")" \
    "$(pen_rel_path "$artifact_dir" "$stderr_path")" \
    "$(pen_rel_path "$artifact_dir" "$exit_path")" \
    "$(pen_rel_path "$artifact_dir" "$raw_path")" \
    "$summary_text"

  printf '%s\n' "$exit_code"
}
