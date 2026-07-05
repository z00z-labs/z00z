#!/usr/bin/env bash
set -uo pipefail

scope="${1:-.}"
mode="${2:-fast}"

have() {
  command -v "$1" >/dev/null 2>&1
}

print_section() {
  printf '\n## %s\n\n' "$1"
}

line_search() {
  local pattern="$1"
  if have rg; then
    rg -n "$pattern" "$scope" 2>/dev/null
  else
    grep -RInE "$pattern" "$scope" 2>/dev/null
  fi
}

print_limited() {
  local label="$1"
  local limit="${2:-250}"
  local content
  content="$(cat)"
  if [ -z "$content" ]; then
    printf 'No matches for %s.\n' "$label"
    return
  fi
  local count
  count="$(printf '%s\n' "$content" | wc -l | tr -d ' ')"
  printf -- '- %s line count: `%s`\n\n' "$label" "$count"
  printf '%s\n' "$content" | head -n "$limit"
  if [ "$count" -gt "$limit" ]; then
    printf '\nOutput truncated after %s lines. Re-run the focused command for full detail.\n' "$limit"
  fi
}

printf '# Rust Security Snapshot\n\n'
printf -- '- Scope: `%s`\n' "$scope"
printf -- '- Mode: `%s`\n' "$mode"

print_section "Tool Availability"
for tool in cargo cargo-audit cargo-deny cargo-geiger cargo-nextest cargo-llvm-cov rustc rustdoc rust-analyzer rg; do
  if have "$tool"; then
    printf -- '- `%s`: available\n' "$tool"
  else
    printf -- '- `%s`: missing\n' "$tool"
  fi
done

print_section "Dependency And Advisory Snapshot"
if have cargo && [ -f Cargo.toml ]; then
  cargo tree --duplicates 2>/dev/null | print_limited "duplicate dependency tree" 250 \
    || printf 'cargo tree --duplicates failed or no duplicates were reported.\n'
else
  printf 'cargo dependency snapshot skipped: cargo missing or current directory has no Cargo.toml.\n'
fi

if have cargo-audit && [ -f Cargo.lock ]; then
  cargo audit 2>/dev/null | print_limited "cargo audit" 250 || true
else
  printf 'cargo audit skipped: cargo-audit missing or Cargo.lock absent.\n'
fi

if have cargo-deny && [ -f Cargo.toml ]; then
  cargo deny check advisories bans licenses sources 2>/dev/null | print_limited "cargo deny" 250 || true
else
  printf 'cargo deny skipped: cargo-deny missing or Cargo.toml absent.\n'
fi

print_section "Unsafe, FFI, Global State"
line_search '\b(unsafe|unsafe impl|unsafe trait|extern "C"|static mut|MaybeUninit|transmute|from_raw|into_raw|NonNull|UnsafeCell)\b' | print_limited "unsafe/ffi/global-state hotspots" 250 || true

print_section "Panic And Fail-Open Hotspots"
line_search '\b(unwrap\(|expect\(|panic!|todo!|unimplemented!|unreachable!|assert!|debug_assert!)' | print_limited "panic/fail-open hotspots" 250 || true

print_section "Parser, Serializer, And Boundary Hotspots"
line_search '\b(parse|parser|deserialize|serialize|from_bytes|to_bytes|decode|encode|canonical|version|schema|validate|verify|check)\b' | print_limited "parser/serializer/boundary hotspots" 250 || true

print_section "Crypto, Proof, Secret, Replay Hotspots"
line_search '\b(secret|private|key|nonce|random|rng|proof|commit|commitment|signature|sign|verify|hash|domain|transcript|nullifier|replay|spent|unique|AEAD|encrypt|decrypt)\b' | print_limited "crypto/proof/secret/replay hotspots" 250 || true

print_section "Async, Concurrency, And Resource Hotspots"
line_search '\b(async|await|spawn|select!|Mutex|RwLock|Atomic|channel|mpsc|oneshot|timeout|sleep|loop|while true|recursion|Vec::with_capacity|reserve\()' | print_limited "async/concurrency/resource hotspots" 250 || true

if [ "$mode" = "deep" ]; then
  print_section "Deep Optional Checks"
  if have cargo-geiger && [ -f Cargo.toml ]; then
    cargo geiger 2>/dev/null | print_limited "cargo geiger" 250 || true
  else
    printf 'cargo geiger skipped.\n'
  fi

  if have cargo && [ -f Cargo.toml ]; then
    cargo clippy --all-targets --all-features --message-format short 2>/dev/null | print_limited "cargo clippy" 250 || true
  else
    printf 'cargo clippy skipped.\n'
  fi
else
  print_section "Deep Optional Checks"
  printf 'Skipped by default. Re-run with: `%s %s deep`\n' "$0" "$scope"
fi
