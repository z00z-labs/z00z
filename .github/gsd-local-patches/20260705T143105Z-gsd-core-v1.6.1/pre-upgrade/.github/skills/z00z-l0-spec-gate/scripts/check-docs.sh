#!/bin/bash

# Run lightweight document and structured-file checks for Z00Z.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
STRICT="${Z00Z_L0_STRICT:-0}"
SPECS_ROOT="${Z00Z_SPECS_ROOT:-$ROOT_DIR/specs}"
TOOLS_DIR="${Z00Z_VERIFY_TOOLS_DIR:-$ROOT_DIR/tools/formal_verification}"
MARKDOWNLINT_DISABLE="${Z00Z_MARKDOWNLINT_DISABLE:-MD013}"
MDBOOK_REQUIRED="${Z00Z_REQUIRE_MDBOOK:-0}"

# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/verify-env.sh"

PATH="$TOOLS_DIR/bin:$TOOLS_DIR/cargo/bin:$TOOLS_DIR/node/bin:$TOOLS_DIR/opam/bin:$PATH"
export PATH

cd "$ROOT_DIR"
FAILURES=0

log() {
  printf '[z00z-l0] %s\n' "$1"
}

note_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    FAILURES=$((FAILURES + 1))
    return 0
  fi
  log "NOTE: $message"
}

unknown_or_fail() {
  local message="$1"
  if [[ "$STRICT" == "1" ]]; then
    echo "ERROR: $message" >&2
    FAILURES=$((FAILURES + 1))
    return 0
  fi
  log "UNKNOWN: $message"
}

run_if_available() {
  local tool="$1"
  shift
  if command -v "$tool" >/dev/null 2>&1; then
    "$tool" "$@"
  else
    unknown_or_fail "$tool is not installed"
  fi
}

resolve_repo_path() {
  local path="$1"
  case "$path" in
    /*) printf '%s\n' "$path" ;;
    *) printf '%s/%s\n' "$ROOT_DIR" "$path" ;;
  esac
}

is_generated_reports_path() {
  local path="$1"
  case "$path" in
    "$ROOT_DIR"/reports/*) return 0 ;;
    *) return 1 ;;
  esac
}

find_latest_generated_specs_root() {
  find "$ROOT_DIR/reports" -maxdepth 3 -type d -name 'specs20*' 2>/dev/null \
    | sort \
    | tail -n 1
}

SPECS_ROOT="$(resolve_repo_path "$SPECS_ROOT")"
if [[ ! -d "$SPECS_ROOT" ]]; then
  latest_generated_specs_root="$(find_latest_generated_specs_root)"
  if [[ -n "$latest_generated_specs_root" ]]; then
    log "NOTE: using generated specs root ${latest_generated_specs_root#"$ROOT_DIR"/} because ${SPECS_ROOT#"$ROOT_DIR"/} is absent"
    SPECS_ROOT="$latest_generated_specs_root"
  fi
fi

if [[ -f "$SPECS_ROOT/book/book.toml" ]]; then
  log "mdBook build for ${SPECS_ROOT#"$ROOT_DIR"/}/book"
  run_if_available mdbook build "$SPECS_ROOT/book"
elif [[ -f "book.toml" ]]; then
  log "mdBook build for repository book"
  run_if_available mdbook build
elif [[ "$MDBOOK_REQUIRED" == "1" ]]; then
  note_or_fail "mdBook was explicitly required but no book.toml was found"
else
  log "NOTE: no mdBook book.toml found; repository has not opted into mdBook"
fi

if command -v lychee >/dev/null 2>&1; then
  log "Offline Markdown link check"
  mapfile -t markdown_files < <(find README.md docs "$SPECS_ROOT" .github/requirements -type f -name '*.md' 2>/dev/null | sort)
  if [[ "${#markdown_files[@]}" -gt 0 ]]; then
    lychee --offline --root-dir "$ROOT_DIR" "${markdown_files[@]}"
  else
    note_or_fail "no Markdown files found in checked doc roots"
  fi
else
  unknown_or_fail "lychee is not installed"
fi

if command -v taplo >/dev/null 2>&1; then
  log "TOML format check"
  mapfile -t toml_files < <(
    git ls-files '*.toml' \
      | rg -v '^(crates/z00z_crypto/tari/|tools/formal_verification/)' \
      | sort
  )
  if [[ "${#toml_files[@]}" -gt 0 ]]; then
    taplo_log="$(mktemp "${TMPDIR:-/tmp}/z00z-taplo.XXXXXX")"
    if ! taplo fmt --check "${toml_files[@]}" >"$taplo_log" 2>&1; then
      sed -n '1,40p' "$taplo_log"
      taplo_error_count="$(grep -c '^ERROR' "$taplo_log" || true)"
      log "UNKNOWN: tracked TOML files have formatting drift"
      log "UNKNOWN: taplo reported ${taplo_error_count:-0} formatting issue(s); output truncated"
    fi
    rm -f "$taplo_log"
  fi
else
  unknown_or_fail "taplo is not installed"
fi

if command -v markdownlint-cli2 >/dev/null 2>&1; then
  log "Markdown lint"
  markdownlint_log="$(mktemp "${TMPDIR:-/tmp}/z00z-markdownlint.XXXXXX.log")"
  markdownlint_config="$(mktemp "${TMPDIR:-/tmp}/z00z-markdownlint-config.XXXXXX.json")"
  markdownlint_args=(
    "README.md"
    "docs/*.md"
    "docs/tech-papers/**/*.md"
  )
  if is_generated_reports_path "$SPECS_ROOT"; then
    log "NOTE: skipping markdownlint on generated specs root ${SPECS_ROOT#"$ROOT_DIR"/}"
  else
    markdownlint_args+=("$SPECS_ROOT/**/*.md")
  fi
  if [[ -n "$MARKDOWNLINT_DISABLE" ]]; then
    python3 - "$MARKDOWNLINT_DISABLE" >"$markdownlint_config" <<'PY'
import json
import sys

rules = {}
for raw in sys.argv[1].replace(",", " ").split():
    rule = raw.strip()
    if rule:
        rules[rule] = False

print(json.dumps(rules))
PY
  fi
  if ! markdownlint-cli2 --config "$markdownlint_config" "${markdownlint_args[@]}" >"$markdownlint_log" 2>&1; then
    sed -n '1,40p' "$markdownlint_log"
    markdownlint_summary="$(grep -m1 '^Summary:' "$markdownlint_log" || true)"
    note_or_fail "repository Markdown has style-lint backlog"
    [[ -n "$markdownlint_summary" ]] && note_or_fail "$markdownlint_summary"
  fi
  rm -f "$markdownlint_log"
  rm -f "$markdownlint_config"
else
  note_or_fail "markdownlint-cli2 is not installed"
fi

log "Traceability check"
traceability_json="$(mktemp "${TMPDIR:-/tmp}/z00z-traceability.XXXXXX.json")"
traceability_status=0
if env Z00Z_SPECS_ROOT="$SPECS_ROOT" python3 "$SCRIPT_DIR/check-traceability.py" --json >"$traceability_json"; then
  :
else
  traceability_status=$?
fi

doc_traceability_status=0
if Z00Z_L0_STRICT="$STRICT" python3 - "$ROOT_DIR" "$SPECS_ROOT" "$traceability_json" <<'PY'
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1]).resolve()
SPECS_ROOT = Path(sys.argv[2]).resolve()
TRACEABILITY_JSON = Path(sys.argv[3])
STRICT = os.environ.get("Z00Z_L0_STRICT", "0") == "1"
DOC_ZINV_RE = re.compile(r"\bZINV[:-]\s*([A-Z][A-Z0-9_-]+)\b")
KEY_RE = re.compile(r"^([A-Z][A-Z0-9_-]+):\s*$")


def collect_invariant_ids(root: Path) -> set[str]:
    ids: set[str] = set()
    invariant_dir = root / "invariants"
    if not invariant_dir.exists():
        return ids
    for path in sorted(invariant_dir.rglob("*")):
        if path.suffix not in {".yaml", ".yml"}:
            continue
        for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
            match = KEY_RE.match(line)
            if match:
                ids.add(match.group(1))
    return ids


def iter_markdown_files() -> list[Path]:
    files: list[Path] = []
    roots = [
        ROOT / "README.md",
        ROOT / "docs",
        SPECS_ROOT,
        ROOT / ".github/requirements",
        ROOT / ".github/skills/z00z-l0-spec-gate",
    ]
    for root in roots:
        if root.is_file() and root.suffix == ".md":
            files.append(root)
            continue
        if root.is_dir():
            files.extend(sorted(path for path in root.rglob("*.md") if path.is_file()))
    return files


traceability = json.loads(TRACEABILITY_JSON.read_text(encoding="utf-8"))
invariant_ids = collect_invariant_ids(SPECS_ROOT)
doc_references: list[tuple[str, str]] = []
missing_doc_ids: dict[str, list[str]] = {}

for path in iter_markdown_files():
    rel = path.relative_to(ROOT).as_posix()
    for lineno, line in enumerate(
        path.read_text(encoding="utf-8", errors="replace").splitlines(),
        start=1,
    ):
        for invariant_id in DOC_ZINV_RE.findall(line):
            doc_references.append((rel, invariant_id))
            if invariant_ids and invariant_id not in invariant_ids:
                missing_doc_ids.setdefault(invariant_id, []).append(f"{rel}:{lineno}")

code_missing_ids = sorted(traceability.get("missing_references", {}).keys())
critical_without_zinv = traceability.get("critical_without_zinv", [])
code_reference_count = int(traceability.get("reference_count", 0))
doc_reference_count = len(doc_references)
total_reference_count = code_reference_count + doc_reference_count

print(f"[z00z-l0] invariants: {traceability.get('invariant_count', len(invariant_ids))}")
print(f"[z00z-l0] code ZINV references: {code_reference_count}")
print(f"[z00z-l0] doc ZINV references: {doc_reference_count}")
print(f"[z00z-l0] ZINV references: {total_reference_count}")
if code_missing_ids:
    print(f"[z00z-l0] code missing invariant IDs: {', '.join(code_missing_ids)}")
if missing_doc_ids:
    print(f"[z00z-l0] doc missing invariant IDs: {', '.join(sorted(missing_doc_ids))}")
if critical_without_zinv:
    print(f"[z00z-l0] critical files without ZINV: {len(critical_without_zinv)}")

exit_code = 0
if missing_doc_ids:
    print("ERROR: documentation references unknown invariant IDs", file=sys.stderr)
    exit_code = 1
if STRICT and doc_reference_count == 0:
    print("ERROR: strict docs mode requires at least one documentation ZINV anchor", file=sys.stderr)
    exit_code = 1
sys.exit(exit_code)
PY
then
  :
else
  doc_traceability_status=$?
fi
rm -f "$traceability_json"

if [[ "$traceability_status" -ne 0 ]]; then
  note_or_fail "Rust/code traceability reported missing or invalid ZINV bindings"
fi
if [[ "$doc_traceability_status" -ne 0 ]]; then
  note_or_fail "documentation traceability reported missing or invalid ZINV anchors"
fi

if [[ "$STRICT" == "1" && "$FAILURES" -gt 0 ]]; then
  exit 1
fi
