#!/usr/bin/env bash
# Test-only authority fixture: the first manifest is stable, then one path drifts.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
readonly ROOT_DIR
readonly REAL_AUTHORITY="$ROOT_DIR/.github/skills/smart-tests-bootstrap/scripts/bootstrap_source_authority.sh"
readonly STATE_FILE="${Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STATE:?fixture state path is required}"
readonly STABLE_MODE="${Z00Z_BOOTSTRAP_AUTHORITY_FIXTURE_STABLE:-false}"

case "${1:-}" in
    packages)
        printf 'z00z_storage\tcrates/z00z_storage/Cargo.toml\n'
        ;;
    manifest)
        count=0
        if [[ -f "$STATE_FILE" ]]; then
            count="$(<"$STATE_FILE")"
        fi
        if [[ "$STABLE_MODE" == true ]] || (( count == 0 )); then
            printf 'fixture/source.rs\t%s\n' \
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
        else
            printf 'fixture/source.rs\t%s\n' \
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        fi
        printf '%s\n' "$((count + 1))" >"$STATE_FILE"
        ;;
    compare | rehash)
        exec "$REAL_AUTHORITY" "$@"
        ;;
    *)
        printf 'unsupported fixture command: %s\n' "${1:-missing}" >&2
        exit 2
        ;;
esac
