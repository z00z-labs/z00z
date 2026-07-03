---
phase: 066-Strix
plan: 066-11
status: complete
completed_at: 2026-07-03
next_plan: 066-12
summary_artifact_for: .planning/phases/066-Strix/066-11-PLAN.md
requirements_completed:
  - REQ-016
  - REQ-017
  - REQ-018
  - REQ-019
  - REQ-020
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-11 Summary: Docker Decision And Isolation Model

## Outcome

`066-11` is complete.

Phase 066 now has an executable Docker-isolation contract on the same
repository-local pentest path. The canonical entrypoint stays
`./z00z_penetration_tests.sh`; the Docker lane remains optional, archive-driven,
attached to the host terminal, non-privileged by default, and report-exporting
back to the host under the existing `.security-artifacts/<timestamp>/` and
`reports/z00z-pentests_report-<timestamp>/` roots.

The lane does not introduce a second orchestration surface. It hardens the
existing Docker wrapper with a non-root container user, read-only rootfs,
read-only archive mount, explicit evidence fields in `docker-run.json`, an
operator README, an optional minimal Dockerfile, and regression checks that
prove no default privileged mode, Docker socket mount, formal-verification
execution, public recon image content, or detached-only logging path was added.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-11-SUMMARY.md`
- `tools/penetration/docker/README.md`
- `tools/penetration/docker/Dockerfile`
- `tools/penetration/docker/run_pentest_container.sh`
- `scripts/penetration/validate_pentest_docker_scope.py`
- `tests/penetration/test_docker_scope.py`
- `z00z_penetration_tests.sh`

## Landed Changes

- Docker wrapper hardening
  - `tools/penetration/docker/run_pentest_container.sh` now runs with a
    non-root numeric container user, `--read-only`, `--cap-drop=ALL`,
    `no-new-privileges`, and writable `tmpfs` mounts only for `/tmp` and
    `/workspace`.
  - The wrapper keeps the packed archive read-only, keeps stdout or stderr
    attached to the invoking terminal, exports host artifacts and reports, and
    records the Docker isolation evidence in `docker-run.json`.
  - The wrapper now invokes the extracted top-level entrypoint through `bash`
    so packed execution does not depend on the archive preserving the entry
    script's executable bit.
  - Copy-back is conditional, so an early inner failure no longer gets masked
    by a secondary `cp` error when no artifact or report tree exists yet.
- Sandbox correctness
  - The wrapper now propagates `Z00Z_SANDBOX_MODE=1` into the inner restore.
  - The writable tmpfs mounts are explicitly `exec`, which restores the
    intended `check_pentest_tools.sh` hook path inside `--skip-formal-verification`
    instead of silently degrading to a "missing hook" warning.
- Docker operator artifacts
  - Added `tools/penetration/docker/README.md` as the operator-facing
    contract: when to use Docker, when not to use it, archive-only semantics,
    host export paths, and the default prohibition set.
  - Added `tools/penetration/docker/Dockerfile` as a minimal optional image
    that preinstalls `PyYAML`, keeps the package surface small, and defaults to
    a non-root `pentest` user.
- Regression coverage
  - `scripts/penetration/validate_pentest_docker_scope.py` now scans the
    Dockerfile in addition to shell entrypoints.
  - `tests/penetration/test_docker_scope.py` now proves the wrapper contract,
    the operator README contract, and the optional Dockerfile denylist on top
    of the existing formal-verification-path validator checks.
  - `z00z_penetration_tests.sh` help text now states that the Docker lane never
    scans the live checkout directly.

## Validation

Commands and evidence used for `066-11` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `bash -n z00z_penetration_tests.sh tools/penetration/docker/run_pentest_container.sh unpack_z00z_project.sh pack_z00z_project.sh scripts/penetration/check_pentest_tools.sh`
- `python3 -m py_compile scripts/penetration/validate_pentest_docker_scope.py tests/penetration/test_docker_scope.py`
- `python3 -m unittest tests.penetration.test_docker_scope tests.penetration.test_packaging_portability tests.penetration.test_codex_surface_integration`
- `python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration`
- `./z00z_penetration_tests.sh --mode quick --static-only --check-only --artifact-dir /tmp/z00z-pentest-local-check-ws11`
- `./pack_z00z_project.sh --output /tmp/z00z-pentest-ws11.tar.gz`
- `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-ws11.tar.gz --mode check-only`
- `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-ws11.tar.gz --mode check-only >/tmp/z00z-ws11-docker.log 2>&1`
- `./z00z_penetration_tests.sh --docker-sandbox --mode quick --static-only --check-only`
- `docker build -t z00z-pentest:local -f tools/penetration/docker/Dockerfile tools/penetration/docker`
- `./z00z_penetration_tests.sh --docker-sandbox --docker-image z00z-pentest:local --archive /tmp/z00z-pentest-ws11.tar.gz --mode quick --static-only --check-only`
- `git diff --check`

Observed proof points:

- The mandatory `bootstrap_tests.sh` gate completed green before `066-11`
  closeout and again on the final `066-11` tree after the last Docker-wrapper
  fixes; no Rust runtime changes were introduced during the Docker-only slice.
- The local non-Docker quick static check completed green and still produced
  the canonical host report root.
- The Docker scope validator passed green on the live repository tree and now
  scans `12` files, including the Dockerfile.
- The direct Docker wrapper succeeded on the default `python:3.12-slim` image
  with a non-root container user, read-only rootfs, attached logs, the
  penetration-tool hook restored inside the sandbox, host artifact export, and
  host report export.
- The latest default-image `docker-run.json` records
  `archive_mount=/input/archive.tar.gz:ro`, `container_user=1000:1000`,
  `log_mode=attached`, and `rootfs_mode=read-only`.
- The auto-pack top-level Docker journey completed green and the paired
  `docker-run.json` records the generated host archive path
  `/tmp/z00z-pentest-20260702T213835Z.tar.gz`.
- The optional Dockerfile built successfully into `z00z-pentest:local`, and
  the top-level Docker journey completed green against that pinned local image.
- The captured live Docker log at `/tmp/z00z-ws11-docker.log` contains no
  `install-verification-tools`, `tools/formal_verification`, `full_verify.sh`,
  or `verification-orchestrator` execution.
- Direct grep audit on the wrapper and Dockerfile is clean for default
  `--privileged` or Docker socket usage.

`cargo test --release` was not rerun as a separate top-level command for
`066-11` because no Rust code changed; the mandatory bootstrap gate already
reran green on the final tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a usable automated prompt-execution
path for this slice.

- Attempt 1
  - `./.github/prompts/gsd-review-tasks-execution.prompt.md current_spec=.planning/phases/066-Strix/066-11-PLAN.md current_task="WS-11" --yolo`
  - Result: failed with `Permission denied` because the prompt file is not an
    executable entrypoint.
- Attempt 2
  - `timeout 60s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-11-PLAN.md current_task="WS-11" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 3
  - `timeout 60s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-11-PLAN.md current_task="WS-11" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83627 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-11-PLAN.md`, the `WS-11` row in `066-TODO.md`, the Docker
    wrapper, the top-level entrypoint, the new Docker README or Dockerfile, the
    validator, and the Docker regression tests.
  - Landed the missing non-root and read-only isolation contract, the evidence
    fields in `docker-run.json`, the operator documentation, the optional local
    image, and the Docker denylist or wrapper assertions.
- Pass 2
  - Ran the direct default-image Docker wrapper and found two real runtime
    issues: the extracted top-level entrypoint was invoked as an executable
    file and failed with `Permission denied`, and host copy-back masked that
    primary failure when no artifact or report tree existed yet.
  - Fixed both issues, reran `bash -n`, reran `test_docker_scope`, and reran
    the direct wrapper path clean.
- Pass 3
  - Re-ran the direct wrapper with log capture and found one more runtime drift:
    the `/workspace` tmpfs did not satisfy the intended executable hook path
    and the inner restore lacked `Z00Z_SANDBOX_MODE=1`, which produced a
    host-style duration hint and skipped the pentest tool hook.
  - Fixed both issues, reran `test_docker_scope`, reran the direct wrapper log
    audit clean, reran the auto-pack top-level journey clean, and reran the
    optional `z00z-pentest:local` image journey clean.

Passes 2 and 3 ended with consecutive clean reruns on the final `066-11` tree.

## Closeout

`066-11` closes `WS-11` by proving that Docker is an optional pentest-only
isolation layer on the same canonical entrypoint, not a second orchestration
stack. The Docker lane is archive-driven, host-report-exporting, attached to
the host terminal, non-root, read-only by default where it matters, and
explicitly separated from formal-verification flows, public-recon defaults, and
credential-attack tooling.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-12-PLAN.md`.
