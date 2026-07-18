#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
mapfile -t wiki_docs < <(
  find "$repo_root/wiki" \
    -path "$repo_root/wiki/.vitepress" -prune -o \
    -path "$repo_root/wiki/.inbox" -prune -o \
    -path "$repo_root/wiki/node_modules" -prune -o \
    -type f -print | LC_ALL=C sort
)

if [[ "${#wiki_docs[@]}" -eq 0 ]]; then
  printf 'No wiki files found under %s/wiki\n' "$repo_root" >&2
  exit 1
fi

hits="$(rg -n --no-heading 'https://github\.com/[^[:space:])]+/blob' "${wiki_docs[@]}" || true)"
if [[ -n "$hits" ]]; then
  printf 'Wiki corpus must use local-path source refs, not GitHub blob links:\n%s\n' "$hits" >&2
  exit 1
fi

printf 'Local docs-link audit passed for the wiki corpus.\n'
