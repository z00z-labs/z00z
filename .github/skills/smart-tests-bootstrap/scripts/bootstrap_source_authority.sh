#!/usr/bin/env bash
# Canonical source authority for the Phase-069 bootstrap compile and test graph.

set -euo pipefail

SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
readonly SCRIPT_PATH
ROOT_DIR="$(cd "$(dirname "$SCRIPT_PATH")/../../../.." && pwd)"
readonly ROOT_DIR
readonly ROOT_PACKAGE="z00z_storage"
readonly SOURCE_HASH_PROCESSES=8
readonly SOURCE_HASH_BATCH_SIZE=8

die() {
    printf 'bootstrap source authority: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 ||
        die "required command not found: $1"
}

repo_relative_path() {
    local path="$1" absolute
    absolute="$(realpath -e -- "$path")" ||
        die "source input does not resolve: $path"
    case "$absolute" in
        "$ROOT_DIR"/*)
            printf '%s\n' "${absolute#"$ROOT_DIR"/}"
            ;;
        *)
            die "resolved local source escaped repository root: $absolute"
            ;;
    esac
}

resolved_local_packages() {
    cargo metadata --format-version 1 --locked --offline |
        jq -r --arg root_name "$ROOT_PACKAGE" '
            (.packages | map({key: .id, value: .}) | from_entries) as $packages
            | (.resolve.nodes | map({key: .id, value: .dependencies}) | from_entries) as $edges
            | def closure($ids):
                (($ids + [$ids[] as $id | $edges[$id][]?]) | unique) as $next
                | if $next == $ids then $ids else closure($next) end;
              [
                $packages
                | to_entries[]
                | select(.value.name == $root_name and .value.source == null)
                | .key
              ] as $roots
            | if ($roots | length) != 1 then
                error("expected exactly one local root package named " + $root_name)
              else
                closure($roots)[]
                | $packages[.]
                | select(.source == null)
                | [.name, .manifest_path]
                | @tsv
              end
        ' |
        LC_ALL=C sort -t $'\t' -k2,2
}

emit_required_file() {
    local path="$1"
    [[ -f "$path" ]] || die "required source input missing: $path"
    repo_relative_path "$path"
}

emit_optional_file() {
    local path="$1"
    [[ ! -e "$path" || -f "$path" ]] ||
        die "optional source input is not a regular file: $path"
    if [[ -f "$path" ]]; then
        repo_relative_path "$path"
    fi
}

emit_tree_files() {
    local root="$1" file canonical_root
    [[ ! -e "$root" || -d "$root" ]] ||
        die "source input root is not a directory: $root"
    [[ -d "$root" ]] || return 0
    canonical_root="$(realpath -e -- "$root")" ||
        die "source input root does not resolve: $root"
    case "$canonical_root" in
        "$ROOT_DIR"/*) ;;
        *)
            die "source input root escaped repository root: $canonical_root"
            ;;
    esac
    while IFS= read -r -d '' file; do
        case "$file" in
            "$ROOT_DIR"/*)
                printf '%s\n' "${file#"$ROOT_DIR"/}"
                ;;
            *)
                die "source input escaped repository root: $file"
                ;;
        esac
    done < <(find "$canonical_root" -type f -print0 | sort -z)
}

source_paths() {
    local package_name manifest_path package_root packages
    local -a bootstrap_scripts=(
        ".github/skills/smart-tests-bootstrap/scripts/bootstrap_cache_identity.sh"
        ".github/skills/smart-tests-bootstrap/scripts/bootstrap_source_authority.sh"
        ".github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh"
        ".github/skills/smart-tests-bootstrap/scripts/nova_measurement_worker_authority_v2.txt"
        ".github/skills/smart-tests-bootstrap/scripts/nova_milestone_tests.sh"
        ".github/skills/smart-tests-bootstrap/scripts/nova_verifier_rss_measurement.sh"
        ".github/skills/smart-tests-bootstrap/scripts/plonky3_resource_worker.sh"
    )
    local -a planning_inputs=(
        ".planning/phases/069-Recursive-Proof/069-CONTEXT.md"
        ".planning/phases/069-Recursive-Proof/069-COVERAGE-AUDIT.py"
        ".planning/phases/069-Recursive-Proof/069-COVERAGE.md"
        ".planning/phases/069-Recursive-Proof/069-TEST-SPEC.md"
        ".planning/phases/069-Recursive-Proof/069-TESTS-TASKS.md"
        ".planning/phases/069-Recursive-Proof/069-TODO.md"
        ".planning/phases/069-Recursive-Proof/069-01-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-02-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-03-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-04-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-05-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-06-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-07-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-08-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-09-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-10-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-11-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-12-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-13-PLAN.md"
        ".planning/phases/069-Recursive-Proof/069-051-PLAN.md"
    )

    emit_required_file "Cargo.toml"
    emit_required_file "Cargo.lock"
    emit_required_file ".cargo/config.toml"
    for manifest_path in "${bootstrap_scripts[@]}" "${planning_inputs[@]}"; do
        emit_required_file "$manifest_path"
    done

    packages="$(resolved_local_packages)" ||
        die "failed to resolve the local bootstrap dependency graph"
    [[ -n "$packages" ]] ||
        die "resolved local bootstrap dependency graph is empty"
    while IFS=$'\t' read -r package_name manifest_path; do
        [[ -n "$package_name" && -n "$manifest_path" ]] ||
            die "cargo metadata returned an incomplete local package record"
        manifest_path="$(realpath -e -- "$manifest_path")"
        package_root="$(dirname "$manifest_path")"
        case "$package_root" in
            "$ROOT_DIR"/*) ;;
            *)
                die "local dependency package escaped repository root: $package_root"
                ;;
        esac

        emit_required_file "$manifest_path"
        emit_optional_file "$package_root/build.rs"
        emit_optional_file "$package_root/README.md"
        emit_tree_files "$package_root/src"
        emit_tree_files "$package_root/config"
        emit_tree_files "$package_root/configs"
        emit_tree_files "$package_root/docs"
        emit_tree_files "$package_root/tests/fixtures"

        if [[ "$package_name" == "$ROOT_PACKAGE" ]]; then
            emit_required_file \
                "$package_root/tests/test_recursive_v2_nova_step.rs"
            emit_required_file \
                "$package_root/tests/test_recursive_v2_nova_adversarial.rs"
        fi
    done <<<"$packages"
}

emit_hashed_paths() {
    local record digest path
    local -a paths=("$@")
    ((${#paths[@]} > 0)) || return 0
    {
        while IFS= read -r -d '' record; do
            digest="${record%% *}"
            path="${record#"$digest  "}"
            [[ "$digest" =~ ^[0-9a-f]{64}$ && -n "$path" ]] ||
                die "sha256sum returned an invalid source record"
            printf '%s\t%s\n' "$path" "$digest"
        done < <(
            printf '%s\0' "${paths[@]}" |
                xargs -0 -r -n "$SOURCE_HASH_BATCH_SIZE" \
                    -P "$SOURCE_HASH_PROCESSES" sha256sum -z --
        )
    } | LC_ALL=C sort -t $'\t' -k1,1
}

write_manifest() (
    local path
    local -a paths=()
    mapfile -t paths < <(source_paths | LC_ALL=C sort -u)
    ((${#paths[@]} > 0)) || die "source authority returned no paths"
    for path in "${paths[@]}"; do
        [[ "$path" != *$'\t'* && "$path" != *$'\n'* ]] ||
            die "source path contains a forbidden control character"
        [[ -f "$path" ]] || die "source input disappeared: $path"
    done
    emit_hashed_paths "${paths[@]}"
)

rehash_manifest() {
    local baseline="$1" path
    local -a paths=()
    [[ -f "$baseline" ]] || die "baseline manifest missing: $baseline"
    while IFS=$'\t' read -r path _; do
        [[ -n "$path" ]] || die "baseline manifest contains an empty path"
        case "$path" in
            /* | ../* | */../* | */..)
                die "baseline manifest contains a non-repository path: $path"
                ;;
        esac
        if [[ -f "$path" ]]; then
            paths+=("$path")
        fi
    done <"$baseline"
    emit_hashed_paths "${paths[@]}"
}

manifest_digest() {
    local manifest="$1"
    [[ -f "$manifest" ]] || die "manifest missing: $manifest"
    sha256sum -- "$manifest" | awk '{print $1}'
}

compare_manifests() (
    local before="$1" after="$2" before_digest after_digest changed_tsv changed_json
    [[ -f "$before" ]] || die "before manifest missing: $before"
    [[ -f "$after" ]] || die "after manifest missing: $after"
    before_digest="$(manifest_digest "$before")"
    after_digest="$(manifest_digest "$after")"
    changed_tsv="$(mktemp)"
    trap 'rm -f -- "$changed_tsv"' EXIT

    awk -F '\t' '
        FNR == NR {
            before[$1] = $2
            next
        }
        {
            after[$1] = $2
        }
        END {
            for (path in before) {
                if (!(path in after)) {
                    printf "removed\t%s\t%s\t\n", path, before[path]
                } else if (before[path] != after[path]) {
                    printf "modified\t%s\t%s\t%s\n", path, before[path], after[path]
                }
            }
            for (path in after) {
                if (!(path in before)) {
                    printf "added\t%s\t\t%s\n", path, after[path]
                }
            }
        }
    ' "$before" "$after" |
        LC_ALL=C sort -t $'\t' -k2,2 >"$changed_tsv"

    changed_json="$(
        jq -Rn '
            [
                inputs
                | split("\t")
                | {
                    change: .[0],
                    path: .[1],
                    before_sha256: (
                        if (.[2] // "") == "" then null else .[2] end
                    ),
                    after_sha256: (
                        if (.[3] // "") == "" then null else .[3] end
                    )
                }
            ]
        ' <"$changed_tsv"
    )"
    jq -n -S \
        --arg status "$([[ "$before_digest" == "$after_digest" ]] &&
            printf stable || printf source_drift)" \
        --arg before_digest "$before_digest" \
        --arg after_digest "$after_digest" \
        --argjson changed_paths "$changed_json" \
        '{
            status: $status,
            before_digest: $before_digest,
            after_digest: $after_digest,
            changed_paths: $changed_paths
        }'
    [[ "$before_digest" == "$after_digest" ]] || return 86
)

usage() {
    printf 'usage: %s packages | manifest | digest | rehash <baseline> | compare <before> <after>\n' \
        "${0##*/}"
}

main() {
    local command="${1:-}" command_name
    cd "$ROOT_DIR"
    for command_name in awk cargo find jq realpath sha256sum sort xargs; do
        require_command "$command_name"
    done
    case "$command" in
        packages)
            [[ "$#" == 1 ]] || die "packages takes no arguments"
            resolved_local_packages |
                while IFS=$'\t' read -r package_name manifest_path; do
                    printf '%s\t%s\n' \
                        "$package_name" \
                        "$(repo_relative_path "$manifest_path")"
                done
            ;;
        manifest)
            [[ "$#" == 1 ]] || die "manifest takes no arguments"
            write_manifest
            ;;
        digest)
            [[ "$#" == 1 ]] || die "digest takes no arguments"
            write_manifest | sha256sum | awk '{print $1}'
            ;;
        rehash)
            [[ "$#" == 2 ]] || die "rehash requires a baseline manifest"
            rehash_manifest "$2"
            ;;
        compare)
            [[ "$#" == 3 ]] || die "compare requires before and after manifests"
            compare_manifests "$2" "$3"
            ;;
        *)
            usage >&2
            return 2
            ;;
    esac
}

main "$@"
