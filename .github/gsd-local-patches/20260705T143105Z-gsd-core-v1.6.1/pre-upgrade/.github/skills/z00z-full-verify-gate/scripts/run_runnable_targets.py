#!/usr/bin/env python3
import argparse
import json
import os
import signal
import shlex
import subprocess
import sys
import tomllib
from functools import lru_cache
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parents[3]


def parse_args():
    parser = argparse.ArgumentParser(
        description="Run whitelisted cargo bins/examples from a manifest."
    )
    parser.add_argument(
        "--manifest",
        default=str(SCRIPT_DIR / "runnable_targets.toml"),
        help="Path to the TOML manifest.",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Include disabled entries from the manifest.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List manifest entries without running them.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print commands without executing them.",
    )
    parser.add_argument(
        "--id",
        action="append",
        dest="ids",
        default=[],
        help="Run only the given manifest id. May be repeated.",
    )
    parser.add_argument(
        "--features",
        default=None,
        help="Override features as a comma-separated list.",
    )
    parser.add_argument(
        "--all-features",
        action="store_true",
        help="Force --all-features for all selected entries.",
    )
    parser.add_argument(
        "--no-all-features",
        action="store_true",
        help="Force all selected entries to skip --all-features.",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Use debug builds instead of --release.",
    )
    parser.add_argument(
        "--reuse-build",
        action="store_true",
        help="Execute prebuilt artifacts directly instead of cargo run when possible.",
    )
    parser.add_argument(
        "--prebuilt-only",
        action="store_true",
        help="With --reuse-build, fail if an artifact is missing instead of building it.",
    )
    return parser.parse_args()


def load_manifest(path):
    with path.open("rb") as handle:
        data = tomllib.load(handle)
    defaults = data.get("defaults", {})
    targets = data.get("target", [])
    if not isinstance(targets, list):
        raise ValueError("manifest target section must be an array of tables")
    return defaults, targets


def parse_features(raw):
    if raw is None:
        return None
    items = [item.strip() for item in raw.split(",")]
    return [item for item in items if item]


def pick_entries(entries, args):
    selected = []
    wanted = set(args.ids)
    for entry in entries:
        entry_id = entry["id"]
        if wanted and entry_id not in wanted:
            continue
        if not args.all and not entry.get("enabled", False):
            continue
        selected.append(entry)
    return selected


def join_features(features):
    return ",".join(features)


def pick_runner(entry, defaults):
    return entry.get("runner", defaults.get("runner", []))


def prefix_cmd(cmd, prefix):
    if not prefix:
        return list(cmd)
    return [*prefix, *cmd]


def pick_release(entry, defaults, args):
    if args.debug:
        return False
    return entry.get("release", defaults.get("release", True))


def pick_all_features(entry, defaults, args):
    if args.all_features:
        return True
    if args.no_all_features:
        return False
    return entry.get("all_features", defaults.get("all_features", False))


def pick_features(entry, defaults, args):
    override = parse_features(args.features)
    if override is not None:
        return override
    return entry.get("features", defaults.get("features", []))


def build_cmd(entry, defaults, args):
    kind = entry["kind"]
    if kind not in {"bin", "example"}:
        raise ValueError(f"unsupported kind for {entry['id']}: {kind}")

    cmd = ["cargo", "run"]
    if pick_release(entry, defaults, args):
        cmd.append("--release")
    if pick_all_features(entry, defaults, args):
        cmd.append("--all-features")

    features = pick_features(entry, defaults, args)
    if features:
        cmd.extend(["--features", join_features(features)])

    cmd.extend(["-p", entry["package"]])
    cmd.extend([f"--{kind}", entry["name"]])

    target_args = entry.get("args", [])
    if target_args:
        cmd.append("--")
        cmd.extend(target_args)
    return prefix_cmd(cmd, pick_runner(entry, defaults))


@lru_cache(maxsize=None)
def cargo_target_dir(root):
    raw = subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=root,
        text=True,
    )
    return Path(json.loads(raw)["target_directory"])


def direct_exec_path(entry, defaults, args, target_dir):
    profile = "debug" if args.debug else "release"
    profile_dir = target_dir / profile
    if entry["kind"] == "bin":
        return profile_dir / entry["name"]
    return profile_dir / "examples" / entry["name"]


def group_build_missing(entries, defaults, args, root, target_dir):
    missing_groups = {}

    for entry in entries:
        exec_path = direct_exec_path(entry, defaults, args, target_dir)
        if exec_path.exists():
            continue
        key = (
            pick_release(entry, defaults, args),
            pick_all_features(entry, defaults, args),
            tuple(pick_features(entry, defaults, args) or []),
        )
        missing_groups.setdefault(key, []).append(entry)

    for (use_release, use_all_features, features), group_entries in missing_groups.items():
        cmd = ["cargo", "build"]
        if use_release:
            cmd.append("--release")
        if use_all_features:
            cmd.append("--all-features")
        elif features:
            cmd.extend(["--features", join_features(list(features))])

        for entry in group_entries:
            cmd.extend(["-p", entry["package"], f"--{entry['kind']}", entry["name"]])

        result = subprocess.run(cmd, cwd=root, check=False)
        if result.returncode != 0:
            raise RuntimeError(f"group build failed: {shlex.join(cmd)}")


def build_env(entry):
    env = os.environ.copy()
    for key, value in entry.get("env", {}).items():
        env[str(key)] = str(value)
    return env


def display_is_usable(env):
    display = env.get("DISPLAY", "").strip()
    if not display:
        return False

    try:
        result = subprocess.run(
            ["xdpyinfo"],
            cwd=ROOT_DIR,
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=5,
            check=False,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False

    return result.returncode == 0


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


def run_smoke(cmd, env, root, stop_sec, allowed):
    proc = subprocess.Popen(
        cmd,
        cwd=root,
        env=env,
        start_new_session=True,
    )

    try:
        code = proc.wait(timeout=stop_sec)
    except subprocess.TimeoutExpired:
        stop_proc(proc)
        return True, f"auto-stopped after {stop_sec}s"

    if code not in allowed:
        return False, f"exited with {code}, expected {allowed}"
    return True, f"exit {code} before auto-stop"


def print_entry(entry):
    flag = "on " if entry.get("enabled", False) else "off"
    note = entry.get("note", "")
    print(f"{entry['id']}: [{flag}] {entry['package']} {entry['kind']} {entry['name']}")
    if note:
        print(f"  note: {note}")


def run_entry(entry, defaults, args, root):
    has_explicit_features = bool(entry.get("features")) or bool(entry.get("all_features"))
    use_direct_exec = (args.reuse_build or args.prebuilt_only) and not has_explicit_features
    cmd = build_cmd(entry, defaults, args)
    timeout = entry.get("timeout_sec", defaults.get("timeout_sec", 60))
    allowed = entry.get(
        "allowed_exit_codes", defaults.get("allowed_exit_codes", [0])
    )
    stop_sec = entry.get("stop_after_sec", defaults.get("stop_after_sec"))

    if use_direct_exec:
        target_dir = cargo_target_dir(root)
        exec_path = direct_exec_path(entry, defaults, args, target_dir)
        if not exec_path.exists():
            if args.prebuilt_only:
                print(
                    f"[runner] missing prebuilt artifact for {entry['id']}: {exec_path}",
                    file=sys.stderr,
                )
                return False
            group_build_missing([entry], defaults, args, root, target_dir)
        cmd = [str(exec_path)]
        cmd.extend(entry.get("args", []))
        cmd = prefix_cmd(cmd, pick_runner(entry, defaults))

    print(f"[runner] {entry['id']}")
    print(f"[runner] cmd: {shlex.join(cmd)}")
    print(f"[runner] timeout: {timeout}s")
    if stop_sec is not None:
        print(f"[runner] auto-stop: {stop_sec}s")

    if args.dry_run:
        return True

    env = build_env(entry)
    if entry.get("headless_fallback") and not display_is_usable(env):
        env["Z00Z_EGUI_SMOKE_HEADLESS"] = "1"
        print("[runner] display unavailable; using headless smoke fallback")

    if stop_sec is not None:
        ok, msg = run_smoke(cmd, env, root, stop_sec, allowed)
        if not ok:
            print(f"[runner] fail: {entry['id']} {msg}", file=sys.stderr)
            return False
        print(f"[runner] pass: {entry['id']} {msg}")
        return True

    try:
        result = subprocess.run(
            cmd,
            cwd=root,
            env=env,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired:
        print(f"[runner] timeout: {entry['id']} exceeded {timeout}s", file=sys.stderr)
        return False

    if result.returncode not in allowed:
        print(
            f"[runner] fail: {entry['id']} exited with {result.returncode}, expected {allowed}",
            file=sys.stderr,
        )
        return False

    print(f"[runner] pass: {entry['id']} exit {result.returncode}")
    return True


def main():
    args = parse_args()
    if args.prebuilt_only:
        args.reuse_build = True
    root = ROOT_DIR
    manifest_path = Path(args.manifest)
    manifest = manifest_path.resolve() if manifest_path.is_absolute() else (root / manifest_path).resolve()
    defaults, entries = load_manifest(manifest)
    selected = pick_entries(entries, args)

    if args.list:
        for entry in selected:
            print_entry(entry)
        if not selected:
            print("No matching manifest entries.")
        return 0

    if not selected:
        print("No runnable entries selected.")
        return 0

    target_dir = None
    if args.reuse_build:
        target_dir = cargo_target_dir(root)
        if not args.prebuilt_only:
            group_build_missing(selected, defaults, args, root, target_dir)

    failures = []
    for entry in selected:
        if not run_entry(entry, defaults, args, root):
            failures.append(entry["id"])

    if failures:
        print("[runner] failed ids: " + ", ".join(failures), file=sys.stderr)
        return 1

    print(f"[runner] all {len(selected)} selected entries passed")
    return 0


def entrypoint():
    try:
        return main()
    except KeyboardInterrupt:
        print("[runner] interrupted by user", file=sys.stderr)
        return 130
    except FileNotFoundError as err:
        missing = err.filename or "unknown-command"
        print(f"[runner] command not found: {missing}", file=sys.stderr)
        return 127


if __name__ == "__main__":
    sys.exit(entrypoint())
