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
Usage: run_source_sast.sh --artifact-dir <path> --scope-json <path> --tool-status <path> [--mode quick|standard|deep] [--profile generic|z00z]
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
  "$ARTIFACT_DIR/sast" \
  "$ARTIFACT_DIR/logs" \
  "$ARTIFACT_DIR/raw/semgrep" \
  "$ARTIFACT_DIR/raw/ast" \
  "$ARTIFACT_DIR/raw/secrets" \
  "$ARTIFACT_DIR/normalized"

tool_timeout_seconds() {
  case "$MODE" in
    quick) printf '%s\n' "120" ;;
    standard) printf '%s\n' "300" ;;
    deep) printf '%s\n' "900" ;;
  esac
}

TOOL_TIMEOUT_SECONDS="$(tool_timeout_seconds)"

SEMGREP_STATUS_PATH="$ARTIFACT_DIR/normalized/sast.semgrep.status.json"
SEMGREP_STDOUT_PATH="$ARTIFACT_DIR/raw/semgrep/semgrep.json"
SEMGREP_STDERR_PATH="$ARTIFACT_DIR/logs/semgrep.err"
SEMGREP_EXIT_PATH="$ARTIFACT_DIR/logs/semgrep.exit"
SAST_SUMMARY_PATH="$ARTIFACT_DIR/sast/summary.json"
AST_TARGETS_PATH="$ARTIFACT_DIR/raw/ast/sg-targets.txt"

write_passive_status() {
  local status_path="$1"
  local tool_name="$2"
  local status_name="$3"
  local exit_code="$4"
  local summary_text="$5"

  pen_write_status_json \
    "$status_path" \
    "sast" \
    "$tool_name" \
    "$status_name" \
    "$exit_code" \
    "" \
    "" \
    "" \
    "" \
    "" \
    "$summary_text"
}

append_status() {
  local status_path="$1"
  local status_name="$2"
  status_files+=("$(pen_rel_path "$ARTIFACT_DIR" "$status_path")")
  status_names+=("$status_name")
}

tool_path_for() {
  pen_tool_field "$TOOL_STATUS" "$1" resolved_path
}

tool_state_for() {
  pen_tool_field "$TOOL_STATUS" "$1" status
}

derive_ast_targets() {
  python3 - "$SEMGREP_STDOUT_PATH" "$SCOPE_JSON" "$ROOT" "$AST_TARGETS_PATH" <<'PY'
import json
import sys
from pathlib import Path

semgrep_path = Path(sys.argv[1])
scope_json_path = Path(sys.argv[2])
root = Path(sys.argv[3]).resolve()
target_path = Path(sys.argv[4])
code_suffixes = {
    ".bash",
    ".c",
    ".cc",
    ".cpp",
    ".go",
    ".h",
    ".hpp",
    ".java",
    ".js",
    ".json",
    ".kt",
    ".md",
    ".py",
    ".rb",
    ".rs",
    ".sh",
    ".toml",
    ".ts",
    ".tsx",
    ".yaml",
    ".yml",
}


def to_rel(candidate: Path) -> str | None:
    try:
        return candidate.resolve(strict=False).relative_to(root).as_posix()
    except ValueError:
        return None


def add_candidate(seen: list[str], known: set[str], candidate: Path) -> None:
    if candidate.is_file():
        rel = to_rel(candidate)
        if rel and rel not in known:
            known.add(rel)
            seen.append(rel)
        return
    if not candidate.is_dir():
        return

    for child in sorted(candidate.rglob("*")):
        if not child.is_file():
            continue
        if child.suffix and child.suffix not in code_suffixes:
            continue
        rel = to_rel(child)
        if rel and rel not in known:
            known.add(rel)
            seen.append(rel)
        if len(seen) >= 512:
            return


targets: list[str] = []
known: set[str] = set()

if semgrep_path.exists():
    try:
        payload = json.loads(semgrep_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        payload = {}
    for raw_path in payload.get("paths", {}).get("scanned", []):
        candidate = Path(raw_path)
        if not candidate.is_absolute():
            candidate = root / candidate
        add_candidate(targets, known, candidate)
        if len(targets) >= 512:
            break
    if len(targets) < 512:
        for result in payload.get("results", []):
            raw_path = result.get("path")
            if not raw_path:
                continue
            candidate = Path(raw_path)
            if not candidate.is_absolute():
                candidate = root / candidate
            add_candidate(targets, known, candidate)
            if len(targets) >= 512:
                break

if len(targets) < 512 and scope_json_path.exists():
    payload = json.loads(scope_json_path.read_text(encoding="utf-8"))
    for raw_path in payload.get("normalized_paths", []):
        add_candidate(targets, known, root / raw_path)
        if len(targets) >= 512:
            break

target_path.parent.mkdir(parents=True, exist_ok=True)
target_path.write_text(
    "".join(f"{item}\n" for item in targets[:512]),
    encoding="utf-8",
)
print(min(len(targets), 512))
PY
}

run_manual_status() {
  local tool_name="$1"
  local status_path="$2"
  local stdout_path="$3"
  local stderr_path="$4"
  local exit_path="$5"
  local raw_path="$6"
  local summary_text="$7"
  shift 7

  local -a command=("$@")
  local command_display
  local exit_code
  local status_name="passed"

  if command -v timeout >/dev/null 2>&1 && [[ -n "$TOOL_TIMEOUT_SECONDS" ]]; then
    command=(timeout --signal=TERM --kill-after=10s "${TOOL_TIMEOUT_SECONDS}s" "${command[@]}")
  fi

  printf -v command_display '%q ' "${command[@]}"
  command_display="${command_display% }"

  set +e
  (
    cd "$ROOT"
    "${command[@]}" >"$stdout_path" 2>"$stderr_path"
  )
  exit_code=$?
  set -e

  printf '%s\n' "$exit_code" >"$exit_path"
  if [[ $exit_code -eq 124 || $exit_code -eq 137 ]]; then
    status_name="failed"
    summary_text="$tool_name timed out after ${TOOL_TIMEOUT_SECONDS}s"
  elif [[ $exit_code -ne 0 ]]; then
    status_name="failed"
    summary_text="$tool_name returned a non-zero exit status"
  fi

  pen_write_status_json \
    "$status_path" \
    "sast" \
    "$tool_name" \
    "$status_name" \
    "$exit_code" \
    "$command_display" \
    "$(pen_rel_path "$ARTIFACT_DIR" "$stdout_path")" \
    "$(pen_rel_path "$ARTIFACT_DIR" "$stderr_path")" \
    "$(pen_rel_path "$ARTIFACT_DIR" "$exit_path")" \
    "$(pen_rel_path "$ARTIFACT_DIR" "$raw_path")" \
    "$summary_text"

  append_status "$status_path" "$status_name"
}

semgrep_path="$(pen_tool_field "$TOOL_STATUS" semgrep resolved_path)"
semgrep_status="$(pen_tool_field "$TOOL_STATUS" semgrep status)"
status_files=()
status_names=()

if [[ -z "$semgrep_path" || "$semgrep_status" != "present" ]]; then
  pen_write_status_json \
    "$SEMGREP_STATUS_PATH" \
    "sast" \
    "semgrep" \
    "missing" \
    127 \
    "" \
    "" \
    "" \
    "" \
    "" \
    "semgrep is not available in tools/penetration or PATH"
  append_status "$SEMGREP_STATUS_PATH" "missing"
  pen_write_lane_summary \
    "$SAST_SUMMARY_PATH" \
    "sast" \
    "completed-with-missing-tools" \
    "source SAST completed with missing tools" \
    "$(pen_rel_path "$ARTIFACT_DIR" "$SEMGREP_STATUS_PATH")"
  exit 0
fi

mapfile -t scan_paths < <(pen_scope_paths "$SCOPE_JSON" "$ROOT")
if [[ ${#scan_paths[@]} -eq 0 ]]; then
  scan_paths=("$ROOT")
fi

command_exit="$(PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
  "$ARTIFACT_DIR" \
  "sast" \
  "semgrep" \
  "$ROOT" \
  "$SEMGREP_STDOUT_PATH" \
  "$SEMGREP_STDOUT_PATH" \
  "$SEMGREP_STDERR_PATH" \
  "$SEMGREP_EXIT_PATH" \
  "$SEMGREP_STATUS_PATH" \
  "$semgrep_path" scan --metrics=off --json --config p/owasp-top-ten --config p/r2c-security-audit "${scan_paths[@]}")"
if [[ "$command_exit" -eq 0 ]]; then
  append_status "$SEMGREP_STATUS_PATH" "passed"
else
  append_status "$SEMGREP_STATUS_PATH" "failed"
fi

derive_ast_targets >/dev/null

sg_path="$(tool_path_for sg)"
sg_state="$(tool_state_for sg)"
tree_sitter_path="$(tool_path_for tree-sitter)"
tree_sitter_state="$(tool_state_for tree-sitter)"
sg_status_path="$ARTIFACT_DIR/normalized/sast.sg.status.json"
tree_sitter_status_path="$ARTIFACT_DIR/normalized/sast.tree-sitter.status.json"

if [[ ! -s "$AST_TARGETS_PATH" ]]; then
  write_passive_status "$sg_status_path" "sg" "skipped" 0 "no deterministic AST targets were derived from scope"
  append_status "$sg_status_path" "skipped"
  write_passive_status "$tree_sitter_status_path" "tree-sitter" "skipped" 0 "no deterministic AST targets were derived from scope"
  append_status "$tree_sitter_status_path" "skipped"
elif [[ -n "$sg_path" && "$sg_state" == "present" ]]; then
  sg_pattern='$F($$$ARGS)'
  run_manual_status \
    "sg" \
    "$sg_status_path" \
    "$ARTIFACT_DIR/raw/ast/sg.jsonl" \
    "$ARTIFACT_DIR/logs/sg.err" \
    "$ARTIFACT_DIR/logs/sg.exit" \
    "$ARTIFACT_DIR/raw/ast/sg.jsonl" \
    "AST structural scan completed" \
    bash -lc 'xargs -r -d "\n" -n 200 "$0" run --pattern "$2" --json=stream < "$1"' "$sg_path" "$AST_TARGETS_PATH" "$sg_pattern"
  write_passive_status "$tree_sitter_status_path" "tree-sitter" "skipped" 0 "tree-sitter was not needed because sg satisfied the AST structural lane"
  append_status "$tree_sitter_status_path" "skipped"
elif [[ -n "$tree_sitter_path" && "$tree_sitter_state" == "present" ]]; then
  run_manual_status \
    "tree-sitter" \
    "$tree_sitter_status_path" \
    "$ARTIFACT_DIR/raw/ast/tree-sitter.txt" \
    "$ARTIFACT_DIR/logs/tree-sitter.err" \
    "$ARTIFACT_DIR/logs/tree-sitter.exit" \
    "$ARTIFACT_DIR/raw/ast/tree-sitter.txt" \
    "tree-sitter structural parse completed" \
    bash -lc '
set -euo pipefail
: >"$2"
while IFS= read -r target; do
  [[ -n "$target" ]] || continue
  "$0" parse -q "$target" >>"$2"
done < "$1"
' "$tree_sitter_path" "$AST_TARGETS_PATH" "$ARTIFACT_DIR/raw/ast/tree-sitter.txt"
  write_passive_status "$sg_status_path" "sg" "skipped" 0 "sg was not needed because tree-sitter satisfied the AST structural lane"
  append_status "$sg_status_path" "skipped"
else
  write_passive_status "$sg_status_path" "sg" "missing" 127 "sg is not available in tools/penetration or PATH"
  append_status "$sg_status_path" "missing"
  write_passive_status "$tree_sitter_status_path" "tree-sitter" "missing" 127 "tree-sitter is not available in tools/penetration or PATH"
  append_status "$tree_sitter_status_path" "missing"
fi

for secret_tool in gitleaks trufflehog trivy; do
  tool_path="$(tool_path_for "$secret_tool")"
  tool_state="$(tool_state_for "$secret_tool")"
  status_path="$ARTIFACT_DIR/normalized/sast.${secret_tool}.status.json"

  if [[ -z "$tool_path" || "$tool_state" != "present" ]]; then
    write_passive_status "$status_path" "$secret_tool" "missing" 127 "$secret_tool is not available in tools/penetration or PATH"
    append_status "$status_path" "missing"
    continue
  fi

  case "$secret_tool" in
    gitleaks)
      raw_path="$ARTIFACT_DIR/raw/secrets/sast.gitleaks.json"
      exit_code="$(PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
        "$ARTIFACT_DIR" \
        "sast" \
        "$secret_tool" \
        "$ROOT" \
        "$raw_path" \
        "$ARTIFACT_DIR/logs/sast.gitleaks.out" \
        "$ARTIFACT_DIR/logs/sast.gitleaks.err" \
        "$ARTIFACT_DIR/logs/sast.gitleaks.exit" \
        "$status_path" \
        "$tool_path" detect --no-banner --exit-code 0 --source "$ROOT" --report-format json --report-path "$raw_path")"
      ;;
    trufflehog)
      raw_path="$ARTIFACT_DIR/raw/secrets/sast.trufflehog.json"
      exit_code="$(PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
        "$ARTIFACT_DIR" \
        "sast" \
        "$secret_tool" \
        "$ROOT" \
        "$raw_path" \
        "$raw_path" \
        "$ARTIFACT_DIR/logs/sast.trufflehog.err" \
        "$ARTIFACT_DIR/logs/sast.trufflehog.exit" \
        "$status_path" \
        "$tool_path" filesystem --no-update --json --no-verification "$ROOT")"
      ;;
    trivy)
      raw_path="$ARTIFACT_DIR/raw/secrets/sast.trivy.json"
      exit_code="$(PEN_TIMEOUT_SECONDS="$TOOL_TIMEOUT_SECONDS" pen_capture_command \
        "$ARTIFACT_DIR" \
        "sast" \
        "$secret_tool" \
        "$ROOT" \
        "$raw_path" \
        "$ARTIFACT_DIR/logs/sast.trivy.out" \
        "$ARTIFACT_DIR/logs/sast.trivy.err" \
        "$ARTIFACT_DIR/logs/sast.trivy.exit" \
        "$status_path" \
        "$tool_path" fs --quiet --format json --output "$raw_path" "$ROOT")"
      ;;
  esac

  if [[ "$exit_code" -eq 0 ]]; then
    append_status "$status_path" "passed"
  else
    append_status "$status_path" "failed"
  fi
done

rollup_status="completed"
rollup_summary="source SAST completed successfully"
rollup_status="$(pen_status_rollup "${status_names[@]}")"
if [[ "$rollup_status" == "completed-with-missing-tools" ]]; then
  rollup_summary="source SAST completed with missing tools"
elif [[ "$rollup_status" == "completed-with-failures" ]]; then
  rollup_summary="source SAST recorded tool failures"
fi

pen_write_lane_summary \
  "$SAST_SUMMARY_PATH" \
  "sast" \
  "$rollup_status" \
  "$rollup_summary" \
  "${status_files[@]}"
