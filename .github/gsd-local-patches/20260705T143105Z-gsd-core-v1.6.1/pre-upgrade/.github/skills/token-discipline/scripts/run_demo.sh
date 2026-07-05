#!/bin/bash

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly INPUT_FILE="$(mktemp /tmp/token-discipline-demo-input-XXXX.md)"
readonly OUTPUT_FILE="$(mktemp /tmp/token-discipline-demo-output-XXXX.md)"
readonly PROMPT_FILE="$(mktemp /tmp/token-discipline-demo-prompt-XXXX.txt)"
readonly LONG_PROMPT_FILE="$(mktemp /tmp/token-discipline-demo-long-prompt-XXXX.txt)"

cleanup() {
  rm -f "${INPUT_FILE}" "${OUTPUT_FILE}" "${PROMPT_FILE}" "${LONG_PROMPT_FILE}"
}

trap cleanup EXIT

PYTHON_BIN="${PYTHON:-python3}"

cat > "${INPUT_FILE}" <<'EOF'
Sure.

Here is a comprehensive explanation of how to add AGENTS.md and SKILL.md to a repository.

Use AGENTS.md at repo root and keep reusable workflows in .github/skills.

In conclusion, keep the rules short.
EOF

cat > "${PROMPT_FILE}" <<'EOF'
Add AGENTS.md and SKILL.md to a repository. Keep the answer short, mention validation, and preserve the key setup steps.
EOF

cat > "${LONG_PROMPT_FILE}" <<'EOF'
You are documenting repository setup rules for a coding agent. Produce a compact answer that preserves the essential setup and validation steps without dropping guardrails.

Requirements to preserve:
- Put AGENTS.md at repository root for always-loaded rules.
- Put reusable workflows in .github/skills/<name>/SKILL.md.
- Keep instructions in English.
- Avoid filler, long introductions, and repeated summaries.
- Mention that the skill should be validated after edits.
- Mention that a local offline eval path is preferred when possible.
- Mention that scripts should stay runnable from the repository root.
- Mention that troubleshooting should point to the first failing local check.

Output goal:
- Keep the final answer short.
- Preserve all required setup and validation points.
- Remove repetition and softener phrases.
EOF

echo '== Raw audit =='
"${PYTHON_BIN}" "${SCRIPT_DIR}/prompt_audit.py" "${INPUT_FILE}" --json || true

echo '== Raw token count =='
"${PYTHON_BIN}" "${SCRIPT_DIR}/token_count.py" "${INPUT_FILE}" --json

echo '== Compact markdown =='
"${PYTHON_BIN}" "${SCRIPT_DIR}/compact_markdown.py" "${INPUT_FILE}" --out "${OUTPUT_FILE}"

echo '== Compacted file =='
cat "${OUTPUT_FILE}"

echo '== Compacted audit =='
"${PYTHON_BIN}" "${SCRIPT_DIR}/prompt_audit.py" "${OUTPUT_FILE}" --json

echo '== Compact budget guard =='
"${PYTHON_BIN}" "${SCRIPT_DIR}/budget_guard.py" "${OUTPUT_FILE}" --mode compact --json

echo '== llmlingua compression =='
"${PYTHON_BIN}" "${SCRIPT_DIR}/llmlingua_compress.py" "${LONG_PROMPT_FILE}" \
  --instruction 'Compress without losing setup and validation steps.' \
  --question 'What is the minimum correct answer?' \
  --target-token 60 \
  --json

echo '== promptfoo offline eval =='
"${SCRIPT_DIR}/run_eval.sh"