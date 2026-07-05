# Testing And Demo

Use these commands to prove the skill works locally without external model keys.

## Validate The Skill Structure

```bash
python .github/skills/skill-builder/scripts/validate-skill.py .github/skills/token-discipline
```

Expected signal:

- `VALIDATION PASSED` or `VALIDATION PASSED WITH WARNINGS`

## Run The Offline Promptfoo Eval

```bash
.github/skills/token-discipline/scripts/run_eval.sh
```

Expected signal:

- promptfoo completes without API-key errors
- promptfoo runs under a compatible local Node runtime even if the system Node is slightly older
- all bundled test cases pass

## Run The End-To-End Demo

```bash
.github/skills/token-discipline/scripts/run_demo.sh
```

Expected signal:

- the raw sample triggers verbose-phrase audit hits
- the compacted sample passes the audit and budget guard
- `llmlingua_compress.py` returns a compressed prompt payload with a lower token count on the long demo prompt
- the bundled promptfoo eval passes

## Direct Helper Examples

Count tokens:

```bash
python .github/skills/token-discipline/scripts/token_count.py README.md --json
```

Check a draft against the compact budget:

```bash
python .github/skills/token-discipline/scripts/budget_guard.py draft.md --mode compact --json
```

Audit a draft for filler:

```bash
python .github/skills/token-discipline/scripts/prompt_audit.py draft.md --json
```

Compact Markdown in place:

```bash
python .github/skills/token-discipline/scripts/compact_markdown.py draft.md --in-place
```

Compress a prompt with llmlingua:

```bash
python .github/skills/token-discipline/scripts/llmlingua_compress.py prompt.txt \
  --instruction 'Compress without losing setup steps.' \
  --question 'What is the minimum correct answer?' \
  --target-token 80 \
  --json
```

Warm the local cache once if needed:

```bash
python .github/skills/token-discipline/scripts/llmlingua_compress.py prompt.txt \
  --allow-download \
  --json
```

## Notes

- The llmlingua script defaults to `sshleifer/tiny-gpt2` and now uses cache-only loading by default, so repeat demo runs stay quiet and offline after the first warmup.
- The promptfoo runner uses a compatible local `node@22.22.0` runtime via `npx`, so the latest promptfoo can run without engine warnings even when the system Node is slightly older.
