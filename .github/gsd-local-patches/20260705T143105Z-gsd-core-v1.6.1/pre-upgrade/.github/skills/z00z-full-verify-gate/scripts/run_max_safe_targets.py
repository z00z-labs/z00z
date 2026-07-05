#!/usr/bin/env python3
import argparse
import json
import os
import shlex
import signal
import subprocess
import sys
import tomllib
from functools import lru_cache
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parents[3]


SAFE_HELP_KEYS = ("cli", "validate")
SKIP_KEYS = (
    "scenario",
    "egui",
    "gui",
    "attack",
    "perf",
    "stress",
    "wallet_creation",
    "rpc_complete",
)


def parse_args():
    parser = argparse.ArgumentParser(
        description="Run a broad but safety-filtered target sweep across workspace crates."
    )
    add_manifest_args(parser)
    add_exec_args(parser)
    add_run_args(parser)
    return parser.parse_args()


def add_manifest_args(parser):
    parser.add_argument(
        "--manifest",
        default=str(SCRIPT_DIR / "runnable_targets.toml"),
        help="Path to the curated runnable target manifest.",
    )
    parser.add_argument(
        "--features",
        default=None,
        help="Override features as a comma-separated list.",
    )
    parser.add_argument(
        "--all-features",
        action="store_true",
        help="Use --all-features for cargo commands.",
    )


def add_exec_args(parser):
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Use debug builds instead of --release.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print commands without executing them.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List planned tasks without running them.",
    )
    parser.add_argument(
        "--keep-going",
        action="store_true",
        help="Continue after failures and report them at the end.",
    )
    parser.add_argument(
        "--reuse-build",
        action="store_true",
        help="Reuse prebuilt artifacts instead of invoking per-target cargo commands.",
    )
    parser.add_argument(
        "--prebuilt-only",
        action="store_true",
        help="With --reuse-build, fail when an artifact is missing instead of building it.",
    )


def add_run_args(parser):
    parser.add_argument(
        "--start-at",
        default=None,
        help="Start execution from the task with this exact id.",
    )
    parser.add_argument(
        "--test-threads",
        type=int,
        default=1,
        help="Rust test threads per cargo test task.",
    )
    parser.add_argument(
        "--bench-timeout",
        type=int,
        default=180,
        help="Timeout in seconds for a bench --no-run task.",
    )
    parser.add_argument(
        "--test-timeout",
        type=int,
        default=1800,
        help="Timeout in seconds for a cargo test task.",
    )
    parser.add_argument(
        "--exec-timeout",
        type=int,
        default=120,
        help="Timeout in seconds for a safe bin/example execution.",
    )
    return parser.parse_args()


def parse_features(raw):
    if raw is None:
        return []
    return [item.strip() for item in raw.split(",") if item.strip()]


def prefix_cmd(cmd, prefix):
    if not prefix:
        return list(cmd)
    return [*prefix, *cmd]


def load_manifest(path):
    with path.open("rb") as handle:
        data = tomllib.load(handle)
    defaults = data.get("defaults", {})
    items = {}
    for entry in data.get("target", []):
        key = (entry["package"], entry["kind"], entry["name"])
        items[key] = entry
    return defaults, items


@lru_cache(maxsize=None)
def load_meta(root):
    raw = subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=root,
        text=True,
    )
    return json.loads(raw)


def is_repo_pkg(pkg):
    path = pkg.get("manifest_path", "")
    return "/crates/" in path and "/tari/" not in path


def pick_kind(target):
    kinds = target.get("kind", [])
    for kind in ("lib", "bin", "example", "test", "bench"):
        if kind in kinds:
            return kind
    return None


def base_cmd(args):
    cmd = []
    if not args.debug:
        cmd.append("--release")
    if args.all_features:
        cmd.append("--all-features")
    features = parse_features(args.features)
    if features:
        cmd.extend(["--features", ",".join(features)])
    return cmd


def add_test_task(tasks, args, pkg_name, pkg_id, kind, target_name):
    cmd = ["cargo", "test", "-p", pkg_name]
    cmd.extend(base_cmd(args))
    if kind == "lib":
        cmd.append("--lib")
    else:
        cmd.extend([f"--{kind}", target_name])
    cmd.extend(["--", "--test-threads", str(args.test_threads)])
    tasks.append(
        {
            "id": f"{pkg_name}:{kind}:{target_name}:test",
            "group": "test",
            "cmd": cmd,
            "timeout": args.test_timeout,
            "package": pkg_name,
            "package_id": pkg_id,
            "kind": kind,
            "name": target_name,
        }
    )


def add_bench_task(tasks, args, pkg_name, target_name):
    cmd = ["cargo", "bench", "-p", pkg_name]
    if args.all_features:
        cmd.append("--all-features")
    features = parse_features(args.features)
    if features:
        cmd.extend(["--features", ",".join(features)])
    cmd.extend(["--bench", target_name, "--no-run"])
    tasks.append(
        {
            "id": f"{pkg_name}:bench:{target_name}:build",
            "group": "bench",
            "cmd": cmd,
            "timeout": args.bench_timeout,
            "package": pkg_name,
            "kind": "bench",
            "name": target_name,
        }
    )


def auto_exec(target):
    name = target["name"].lower()
    src_path = target.get("src_path", "").lower()
    hay = f"{name} {src_path}"
    if any(key in hay for key in SKIP_KEYS):
        return None, "auto-skip by safety pattern"
    if any(key in name for key in SAFE_HELP_KEYS):
        return {"args": ["--help"], "allowed_exit_codes": [0, 2]}, None
    return None, "unreviewed execution target"


def pick_exec_spec(manifest, defaults, target):
    key = (target["package"], target["kind"], target["name"])
    entry = manifest.get(key)
    if entry is None:
        return auto_exec(target)
    if not entry.get("enabled", False):
        return None, entry.get("note", "manifest disabled")
    return {
        "args": entry.get("args", defaults.get("args", [])),
        "env": {**defaults.get("env", {}), **entry.get("env", {})},
        "allowed_exit_codes": entry.get(
            "allowed_exit_codes", defaults.get("allowed_exit_codes", [0])
        ),
        "timeout_sec": entry.get("timeout_sec", defaults.get("timeout_sec")),
        "stop_after_sec": entry.get(
            "stop_after_sec", defaults.get("stop_after_sec")
        ),
        "features": entry.get("features", defaults.get("features")),
        "all_features": entry.get("all_features", defaults.get("all_features")),
        "release": entry.get("release", defaults.get("release")),
        "runner": entry.get("runner", defaults.get("runner", [])),
    }, None


def build_exec_cmd(args, target, spec):
    cmd = ["cargo", "run"]
    use_release = spec.get("release")
    if use_release is None:
        use_release = not args.debug
    if use_release:
        cmd.append("--release")

    use_all = spec.get("all_features")
    if use_all is None:
        use_all = args.all_features
    if use_all:
        cmd.append("--all-features")

    feat_list = spec.get("features")
    if feat_list is None:
        feat_list = parse_features(args.features)
    if feat_list:
        cmd.extend(["--features", ",".join(feat_list)])

    cmd.extend(["-p", target["package"], f"--{target['kind']}", target["name"]])
    run_args = spec.get("args", [])
    if run_args:
        cmd.append("--")
        cmd.extend(run_args)
    return prefix_cmd(cmd, spec.get("runner", []))


def add_exec_task(tasks, args, manifest, defaults, target, skips):
    spec, reason = pick_exec_spec(manifest, defaults, target)
    if spec is None:
        skips.append(
            {
                "id": f"{target['package']}:{target['kind']}:{target['name']}:run",
                "reason": reason,
            }
        )
        return

    timeout_sec = spec.get("timeout_sec")
    if timeout_sec is None:
        timeout_sec = args.exec_timeout

    tasks.append(
        {
            "id": f"{target['package']}:{target['kind']}:{target['name']}:run",
            "group": "exec",
            "cmd": build_exec_cmd(args, target, spec),
            "timeout": timeout_sec,
            "allowed": spec.get("allowed_exit_codes", [0]),
            "env": spec.get("env", {}),
            "stop_sec": spec.get("stop_after_sec"),
            "package": target["package"],
            "kind": target["kind"],
            "name": target["name"],
            "run_args": spec.get("args", []),
            "release": spec.get("release"),
            "all_features": spec.get("all_features"),
            "features": spec.get("features"),
            "runner": spec.get("runner", []),
        }
    )


def collect_tasks(args, root, manifest, defaults):
    meta = load_meta(root)
    tasks = []
    skips = []

    for pkg in meta.get("packages", []):
        if not is_repo_pkg(pkg):
            continue
        pkg_name = pkg["name"]
        pkg_id = pkg["id"]
        pkg_dir = str(Path(pkg["manifest_path"]).resolve().parent)
        for target in pkg.get("targets", []):
            kind = pick_kind(target)
            if kind is None:
                continue
            item = {
                "package": pkg_name,
                "package_id": pkg_id,
                "package_dir": pkg_dir,
                "kind": kind,
                "name": target["name"],
                "src_path": target.get("src_path", ""),
            }
            if kind in {"lib", "bin", "example", "test"}:
                add_test_task(tasks, args, pkg_name, pkg_id, kind, target["name"])
                tasks[-1]["cwd"] = pkg_dir
            if kind == "bench":
                add_bench_task(tasks, args, pkg_name, target["name"])
            if kind in {"bin", "example"}:
                add_exec_task(tasks, args, manifest, defaults, item, skips)

    return tasks, skips


def apply_start_at(tasks, start_at):
    if start_at is None:
        return tasks

    for index, task in enumerate(tasks):
        if task["id"] == start_at:
            return tasks[index:]

    raise ValueError(f"Unknown task id for --start-at: {start_at}")


def print_task(task):
    print(f"[task] {task['id']}")
    print(f"[task] cmd: {shlex.join(task['cmd'])}")
    print(f"[task] timeout: {task['timeout']}s")
    stop_sec = task.get("stop_sec")
    if stop_sec is not None:
        print(f"[task] auto-stop: {stop_sec}s")


def cargo_target_dir(root):
    return Path(load_meta(root)["target_directory"])


def direct_exec_path(target_dir, release, kind, name):
    profile = "release" if release else "debug"
    profile_dir = target_dir / profile
    if kind == "bin":
        return profile_dir / name
    if kind == "example":
        return profile_dir / "examples" / name
    raise ValueError(f"unsupported direct exec kind: {kind}")


def test_artifact_patterns(name):
    patterns = [name]
    normalized = name.replace("-", "_")
    if normalized != name:
        patterns.append(normalized)
    return patterns


@lru_cache(maxsize=None)
def is_test_harness(path_str):
    path = Path(path_str)
    try:
        result = subprocess.run(
            [str(path), "--list"],
            text=True,
            capture_output=True,
            timeout=5,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return False

    if result.returncode != 0:
        return False

    combined = f"{result.stdout}\n{result.stderr}".lower()
    return "test" in combined or "benchmark" in combined


def build_test_artifacts(args, root):
    cmd = ["cargo", "test", "--workspace"]
    cmd.extend(base_cmd(args))
    cmd.extend(["--all-targets", "--no-run", "--message-format=json-render-diagnostics"])
    result = subprocess.run(cmd, cwd=root, text=True, capture_output=True, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"batched test prebuild failed: {shlex.join(cmd)}")

    artifacts = {}
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-artifact":
            continue
        executable = message.get("executable")
        target = message.get("target", {})
        package_id = message.get("package_id")
        if not executable or not package_id:
            continue
        kinds = target.get("kind", [])
        kind = next((item for item in ("lib", "bin", "example", "test") if item in kinds), None)
        name = target.get("name")
        if kind is None or not name:
            continue
        package_name = package_id.split(" ", 1)[0]
        artifacts[(package_id, kind, name)] = executable
        artifacts[(package_name, kind, name)] = executable
    return artifacts


def probe_test_artifacts(tasks, args, target_dir):
    artifacts = {}
    for task in tasks:
        use_release = task.get("release")
        if use_release is None:
            use_release = not args.debug
        profile = "release" if use_release else "debug"
        name = task["name"]
        search_dirs = [target_dir / profile / "deps"]
        if task["kind"] == "example":
            search_dirs.append(target_dir / profile / "examples")
        matches = []
        for search_dir in search_dirs:
            for pattern in test_artifact_patterns(name):
                matches.extend(
                    path
                    for path in search_dir.glob(f"{pattern}-*")
                    if path.is_file() and os.access(path, os.X_OK)
                )
        if not matches:
            continue
        harnesses = [path for path in matches if is_test_harness(str(path))]
        if not harnesses:
            continue
        executable = max(harnesses, key=lambda path: path.stat().st_mtime_ns)
        package_name = task["package"]
        package_id = task.get("package_id")
        if package_id:
            artifacts[(package_id, task["kind"], name)] = str(executable)
        artifacts[(package_name, task["kind"], name)] = str(executable)
    return artifacts


def has_test_artifact(test_artifacts, task):
    key = (task.get("package_id"), task["kind"], task["name"])
    if key in test_artifacts:
        return True
    fallback_key = (task["package"], task["kind"], task["name"])
    return fallback_key in test_artifacts


def prebuild_benches(args, root):
    cmd = ["cargo", "bench", "--workspace"]
    if args.all_features:
        cmd.append("--all-features")
    features = parse_features(args.features)
    if features:
        cmd.extend(["--features", ",".join(features)])
    cmd.append("--no-run")
    result = subprocess.run(cmd, cwd=root, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"batched bench prebuild failed: {shlex.join(cmd)}")


def build_missing_exec_tasks(tasks, args, root, target_dir):
    groups = {}
    for task in tasks:
        use_release = task["release"]
        if use_release is None:
            use_release = not args.debug
        exec_path = direct_exec_path(target_dir, use_release, task["kind"], task["name"])
        if exec_path.exists():
            continue
        use_all_features = task["all_features"]
        if use_all_features is None:
            use_all_features = args.all_features
        features = task["features"]
        if features is None:
            features = parse_features(args.features)
        key = (use_release, use_all_features, tuple(features or []))
        groups.setdefault(key, []).append(task)

    for (use_release, use_all_features, features), group_tasks in groups.items():
        cmd = ["cargo", "build"]
        if use_release:
            cmd.append("--release")
        if use_all_features:
            cmd.append("--all-features")
        elif features:
            cmd.extend(["--features", ",".join(features)])
        for task in group_tasks:
            cmd.extend(["-p", task["package"], f"--{task['kind']}", task["name"]])
        result = subprocess.run(cmd, cwd=root, check=False)
        if result.returncode != 0:
            raise RuntimeError(f"batched exec prebuild failed: {shlex.join(cmd)}")


def prepare_reused_tasks(tasks, args, root):
    if not args.reuse_build or args.dry_run:
        return

    test_tasks = [task for task in tasks if task["group"] == "test"]
    bench_tasks = [task for task in tasks if task["group"] == "bench"]
    exec_tasks = [task for task in tasks if task["group"] == "exec"]

    target_dir = cargo_target_dir(root) if (test_tasks or exec_tasks) else None

    test_artifacts = {}
    if test_tasks:
        test_artifacts = probe_test_artifacts(test_tasks, args, target_dir)
        missing_test_artifacts = [
            task for task in test_tasks if not has_test_artifact(test_artifacts, task)
        ]
        if missing_test_artifacts:
            if args.prebuilt_only:
                for task in missing_test_artifacts:
                    task["prebuilt_missing"] = True
            else:
                test_artifacts = build_test_artifacts(args, root)

    if bench_tasks and not args.prebuilt_only:
        prebuild_benches(args, root)

    if exec_tasks and not args.prebuilt_only:
        build_missing_exec_tasks(exec_tasks, args, root, target_dir)

    for task in test_tasks:
        executable = test_artifacts.get((task["package_id"], task["kind"], task["name"]))
        if executable is None:
            executable = test_artifacts.get((task["package"], task["kind"], task["name"]))
        if executable is None:
            if args.prebuilt_only:
                task["prebuilt_missing"] = True
            continue
        task["cmd"] = [executable, "--test-threads", str(args.test_threads)]

    for task in bench_tasks:
        task["skip_exec"] = True

    for task in exec_tasks:
        has_explicit_features = bool(task.get("features")) or bool(task.get("all_features"))
        if has_explicit_features:
            continue
        use_release = task["release"]
        if use_release is None:
            use_release = not args.debug
        executable = direct_exec_path(target_dir, use_release, task["kind"], task["name"])
        if not executable.exists():
            if args.prebuilt_only:
                task["prebuilt_missing"] = True
            continue
        task["cmd"] = prefix_cmd(
            [str(executable), *task.get("run_args", [])],
            task.get("runner", []),
        )


def stop_proc(proc):
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return

    try:
        proc.wait(timeout=2)
        return
    except subprocess.TimeoutExpired:
        pass

    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    proc.wait()


def run_smoke(task, root):
    env = os.environ.copy()
    env.update(task.get("env", {}))
    stop_sec = task["stop_sec"]
    proc = subprocess.Popen(
        task["cmd"],
        cwd=task.get("cwd", root),
        env=env,
        start_new_session=True,
    )

    try:
        code = proc.wait(timeout=stop_sec)
    except subprocess.TimeoutExpired:
        stop_proc(proc)
        print(f"[task] pass: {task['id']} auto-stopped after {stop_sec}s")
        return True

    allowed = task.get("allowed", [0])
    if code not in allowed:
        print(
            f"[task] fail: {task['id']} exited with {code}, expected {allowed}",
            file=sys.stderr,
        )
        return False

    print(f"[task] pass: {task['id']} exit {code} before auto-stop")
    return True


def run_task(task, root, dry_run):
    if task.get("prebuilt_missing"):
        print(f"[task] missing prebuilt artifact: {task['id']}", file=sys.stderr)
        return False

    if task.get("skip_exec"):
        print(f"[task] {task['id']}")
        print("[task] satisfied by batched prebuild")
        return True

    print_task(task)
    if dry_run:
        return True

    if task.get("stop_sec") is not None:
        return run_smoke(task, root)

    env = os.environ.copy()
    env.update(task.get("env", {}))
    task_cwd = task.get("cwd", root)
    try:
        result = subprocess.run(
            task["cmd"],
            cwd=task_cwd,
            env=env,
            timeout=task["timeout"],
            check=False,
        )
    except subprocess.TimeoutExpired:
        print(f"[task] timeout: {task['id']}", file=sys.stderr)
        return False

    allowed = task.get("allowed", [0])
    if result.returncode not in allowed:
        print(
            f"[task] fail: {task['id']} exited with {result.returncode}, expected {allowed}",
            file=sys.stderr,
        )
        return False

    print(f"[task] pass: {task['id']} exit {result.returncode}")
    return True


def run_sweep(args):
    return run_sweep_impl(args)


def run_sweep_impl(args):
    root = ROOT_DIR
    manifest_path = Path(args.manifest)
    manifest_file = manifest_path.resolve() if manifest_path.is_absolute() else (root / manifest_path).resolve()
    defaults, manifest = load_manifest(manifest_file)
    tasks, skips = collect_tasks(args, root, manifest, defaults)
    tasks = apply_start_at(tasks, args.start_at)

    if args.list:
        print_plan(tasks, skips)
        return 0

    prepare_reused_tasks(tasks, args, root)

    fails = execute_tasks(tasks, root, args.dry_run, args.keep_going)

    print_summary(tasks, skips, fails)

    if fails:
        print("[summary] failed ids: " + ", ".join(fails), file=sys.stderr)
        return 1
    return 0


def print_plan(tasks, skips):
    for task in tasks:
        print_task(task)
    for skip in skips:
        print(f"[skip] {skip['id']} :: {skip['reason']}")


def execute_tasks(tasks, root, dry_run, keep_going):
    fails = []
    for task in tasks:
        ok = run_task(task, root, dry_run)
        if ok:
            continue
        fails.append(task["id"])
        if not keep_going:
            break
    return fails


def print_summary(tasks, skips, fails):
    print(f"[summary] planned={len(tasks)} skipped={len(skips)} failed={len(fails)}")
    for skip in skips:
        print(f"[skip] {skip['id']} :: {skip['reason']}")


def main():
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    args = parse_args()
    if args.prebuilt_only:
        args.reuse_build = True
    return run_sweep(args)


def entrypoint():
    try:
        return main()
    except KeyboardInterrupt:
        print("[summary] interrupted by user", file=sys.stderr)
        return 130
    except FileNotFoundError as err:
        missing = err.filename or "unknown-command"
        print(f"[summary] command not found: {missing}", file=sys.stderr)
        return 127


if __name__ == "__main__":
    sys.exit(entrypoint())
