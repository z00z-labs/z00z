#!/usr/bin/env bash
# Build a deterministic bootstrap cache identity from existing Cargo JSON artifacts.

set -euo pipefail

die() {
    printf 'bootstrap cache identity: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 ||
        die "required command not found: $1"
}

usage() {
    printf 'usage: %s capture <normal-jsonl> <lib-test-jsonl> <context-json> <output-json> [expected-digest]\n' \
        "${0##*/}"
}

capture_identity() {
    local normal_messages="$1" test_messages="$2" context_file="$3"
    local output_file="$4" expected_digest="${5:-}"
    local context_json root_name repo_root checkpoint_root cache_authority_root source_digest
    local normal_units test_units
    local normal_hashes test_library_hashes test_executable_hashes
    local normal_hash test_library_hash test_executable_hash
    local normal_target test_target normal_size test_size
    local checkpoint_real cache_authority_real target_real evidence_path evidence_real output_parent
    local base_file final_file identity_digest expected_json

    [[ -s "$normal_messages" ]] ||
        die "normal Cargo unit messages are missing: $normal_messages"
    [[ -s "$test_messages" ]] ||
        die "lib-test Cargo unit messages are missing: $test_messages"
    [[ -s "$context_file" ]] ||
        die "cache context is missing: $context_file"
    if [[ -n "$expected_digest" && ! "$expected_digest" =~ ^[0-9a-f]{64}$ ]]; then
        die "expected cache digest must be empty or lowercase SHA-256"
    fi
    jq -e '.schema == "z00z.phase069.bootstrap-cache-context.v3"' \
        "$context_file" >/dev/null ||
        die "cache context schema is invalid"

    context_json="$(<"$context_file")"
    root_name="$(jq -er '.root_package_name' <<<"$context_json")"
    repo_root="$(jq -er '.repo_root' <<<"$context_json")"
    checkpoint_root="$(jq -er '.checkpoint_output_root' <<<"$context_json")"
    cache_authority_root="$(jq -er '.cache_authority_root' <<<"$context_json")"
    source_digest="$(jq -er '.source_authority_digest' <<<"$context_json")"
    [[ "$source_digest" =~ ^[0-9a-f]{64}$ ]] ||
        die "cache context source authority digest is invalid"
    normal_target="$(jq -er '.target_dirs.normal_library' <<<"$context_json")"
    test_target="$(jq -er '.target_dirs.library_test' <<<"$context_json")"
    [[ "$repo_root" == /* && "$checkpoint_root" == /* &&
        "$cache_authority_root" == /* &&
        "$normal_target" == /* && "$test_target" == /* ]] ||
        die "cache context paths must be absolute"
    checkpoint_real="$(realpath -e -- "$checkpoint_root")" ||
        die "checkpoint output authority is unavailable"
    [[ "$checkpoint_real" == "$checkpoint_root" && ! -L "$checkpoint_root" ]] ||
        die "checkpoint output authority resolved through a symlink"
    cache_authority_real="$(realpath -e -- "$cache_authority_root")" ||
        die "repository cache authority is unavailable"
    [[ "$cache_authority_real" == "$cache_authority_root" &&
        "$cache_authority_root" == "$repo_root/.cache" &&
        ! -L "$cache_authority_root" ]] ||
        die "repository cache authority must be canonical .cache"
    for target_real in "$normal_target" "$test_target"; do
        [[ -d "$target_real" && ! -L "$target_real" ]] ||
            die "cache target is not a real directory: $target_real"
        [[ "$(realpath -e -- "$target_real")" == "$target_real" ]] ||
            die "cache target resolved through a symlink: $target_real"
        case "$target_real" in
            "$cache_authority_real/"*) ;;
            *)
                die "cache target escaped repository .cache authority: $target_real"
                ;;
        esac
        case "$target_real" in
            "$repo_root" | "$repo_root/" | \
                "$repo_root/target" | "$repo_root/target/"* | \
                "$repo_root/third_party" | "$repo_root/third_party/"*)
                die "cache target uses a forbidden repository path: $target_real"
                ;;
        esac
    done
    for evidence_path in "$normal_messages" "$test_messages" "$context_file"; do
        evidence_real="$(realpath -e -- "$evidence_path")" ||
            die "cache identity input is unavailable: $evidence_path"
        case "$evidence_real" in
            "$checkpoint_real/"*) ;;
            *)
                die "cache identity input escaped checkpoint output authority"
                ;;
        esac
    done
    output_parent="$(realpath -e -- "$(dirname "$output_file")")" ||
        die "cache identity output parent is unavailable"
    case "$output_parent" in
        "$checkpoint_real" | "$checkpoint_real/"*) ;;
        *)
            die "cache identity output escaped checkpoint output authority"
            ;;
    esac
    [[ ! -L "$output_file" ]] ||
        die "cache identity output must not be a symlink"

    normal_units="$(
        jq -s \
            --arg root_name "$root_name" \
            --arg root "$repo_root/" \
            '
                [
                    .[]
                    | select(
                        .reason == "compiler-artifact"
                        and .target.name == $root_name
                    )
                    | {
                        target: {
                            name: .target.name,
                            kind: .target.kind,
                            crate_types: .target.crate_types
                        },
                        profile: .profile,
                        features: (.features | sort),
                        filenames: (
                            .filenames
                            | map(ltrimstr($root))
                            | sort
                        ),
                        executable: (
                            .executable
                            | if . == null then null else ltrimstr($root) end
                        )
                    }
                ]
                | sort_by(.target.name, .profile.test, (.executable // ""))
            ' "$normal_messages"
    )"
    test_units="$(
        jq -s \
            --arg root_name "$root_name" \
            --arg root "$repo_root/" \
            '
                [
                    .[]
                    | select(
                        .reason == "compiler-artifact"
                        and .target.name == $root_name
                    )
                    | {
                        target: {
                            name: .target.name,
                            kind: .target.kind,
                            crate_types: .target.crate_types
                        },
                        profile: .profile,
                        features: (.features | sort),
                        filenames: (
                            .filenames
                            | map(ltrimstr($root))
                            | sort
                        ),
                        executable: (
                            .executable
                            | if . == null then null else ltrimstr($root) end
                        )
                    }
                ]
                | sort_by(.target.name, .profile.test, (.executable // ""))
            ' "$test_messages"
    )"

    normal_hashes="$(
        jq -c '
            [
                .[]
                | select(.profile.test == false)
                | .filenames[]
                | try capture(
                    "(^|/)libz00z_storage-(?<hash>[0-9a-f]{16,64})\\.(rlib|rmeta)$"
                  ).hash
            ]
            | unique
        ' <<<"$normal_units"
    )"
    test_library_hashes="$(
        jq -c '
            [
                .[]
                | select(.profile.test == false)
                | .filenames[]
                | try capture(
                    "(^|/)libz00z_storage-(?<hash>[0-9a-f]{16,64})\\.(rlib|rmeta)$"
                  ).hash
            ]
            | unique
        ' <<<"$test_units"
    )"
    test_executable_hashes="$(
        jq -c '
            [
                .[]
                | select(.profile.test == true)
                | .executable // empty
                | try capture(
                    "(^|/)z00z_storage-(?<hash>[0-9a-f]{16,64})$"
                  ).hash
            ]
            | unique
        ' <<<"$test_units"
    )"
    [[ "$(jq 'length' <<<"$normal_hashes")" == 1 ]] ||
        die "normal unit graph must expose exactly one z00z_storage library hash"
    [[ "$(jq 'length' <<<"$test_library_hashes")" == 1 ]] ||
        die "lib-test graph must expose exactly one z00z_storage dependency-library hash"
    [[ "$(jq 'length' <<<"$test_executable_hashes")" == 1 ]] ||
        die "lib-test graph must expose exactly one z00z_storage test-executable hash"
    normal_hash="$(jq -r '.[0]' <<<"$normal_hashes")"
    test_library_hash="$(jq -r '.[0]' <<<"$test_library_hashes")"
    test_executable_hash="$(jq -r '.[0]' <<<"$test_executable_hashes")"

    normal_size="$(du -sb -- "$normal_target" | awk '{print $1}')"
    test_size="$(du -sb -- "$test_target" | awk '{print $1}')"
    base_file="$(mktemp "${output_file}.base.XXXXXX")"
    final_file="$(mktemp "${output_file}.final.XXXXXX")"
    trap 'rm -f -- "$base_file" "$final_file"' RETURN
    jq -n -S \
        --argjson context "$context_json" \
        --argjson normal_units "$normal_units" \
        --argjson test_units "$test_units" \
        --arg normal_hash "$normal_hash" \
        --arg test_library_hash "$test_library_hash" \
        --arg test_executable_hash "$test_executable_hash" \
        '{
            schema: "z00z.phase069.bootstrap-cache-identity.v3",
            execution_scope: $context.execution_scope,
            repo_root: $context.repo_root,
            checkpoint_output_root: $context.checkpoint_output_root,
            cache_authority_root: $context.cache_authority_root,
            source_authority_digest: $context.source_authority_digest,
            profile: $context.profile,
            rustflags: $context.rustflags,
            toolchain: $context.toolchain,
            compile_environment: $context.compile_environment,
            resolved_local_packages: $context.resolved_local_packages,
            target_dirs: {
                normal_library: {
                    path: $context.target_dirs.normal_library
                },
                library_test: {
                    path: $context.target_dirs.library_test
                }
            },
            retention: {
                schema: $context.retention.schema,
                strategy: $context.retention.strategy,
                max_target_roots: $context.retention.max_target_roots,
                automatic_deletion: $context.retention.automatic_deletion
            },
            compile_contract: $context.compile_contract,
            unit_graph: {
                normal_library: $normal_units,
                library_test: $test_units
            },
            unit_hashes: {
                normal_library: $normal_hash,
                library_test_dependency_library: $test_library_hash,
                library_test_executable: $test_executable_hash
            },
            digest_scope: [
                "execution_scope",
                "repo_root",
                "checkpoint_output_root",
                "cache_authority_root",
                "source_authority_digest",
                "profile",
                "rustflags",
                "toolchain",
                "compile_environment",
                "resolved_local_packages",
                "target_dirs.paths",
                "retention",
                "compile_contract",
                "unit_graph",
                "unit_hashes"
            ]
        }' >"$base_file"
    identity_digest="$(sha256sum "$base_file" | awk '{print $1}')"
    if [[ -n "$expected_digest" ]]; then
        expected_json="$(jq -n --arg value "$expected_digest" '$value')"
    else
        expected_json=null
    fi
    jq -S \
        --arg digest "$identity_digest" \
        --argjson expected_digest "$expected_json" \
        --argjson normal_size "$normal_size" \
        --argjson test_size "$test_size" \
        '.target_dirs.normal_library.observed_size_bytes = $normal_size
        | .target_dirs.library_test.observed_size_bytes = $test_size
        | . + {
            digest: $digest,
            expected_digest: $expected_digest,
            matches_expected: (
                if $expected_digest == null then null
                else $digest == $expected_digest
                end
            )
        }' "$base_file" >"$final_file"
    mv -- "$final_file" "$output_file"
    trap - RETURN
    rm -f -- "$base_file"
}

main() {
    local command="${1:-}" required
    for required in awk dirname du jq mktemp mv realpath sha256sum; do
        require_command "$required"
    done
    case "$command" in
        capture)
            [[ "$#" -ge 5 && "$#" -le 6 ]] ||
                die "capture requires four paths and an optional expected digest"
            capture_identity "$2" "$3" "$4" "$5" "${6:-}"
            ;;
        *)
            usage >&2
            return 2
            ;;
    esac
}

main "$@"
