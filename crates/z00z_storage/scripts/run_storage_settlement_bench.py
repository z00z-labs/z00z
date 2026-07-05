#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import math
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
OUT_DIR = ROOT / "crates" / "z00z_storage" / "outputs" / "settlement"
CRITERION_DIR = ROOT / "target" / "criterion"
SCENARIO_OUT_DIR = ROOT / "crates" / "z00z_simulator" / "outputs" / "scenario_1"
TIME_BIN = Path("/usr/bin/time")
OUT_KEEP_ENV = "Z00Z_STORAGE_SETTLEMENT_BENCH_KEEP"
PROOF_NOTE_SCOPE_ENV = "Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE"
PROOF_NOTE_COMMAND_ENV = "Z00Z_SETTLEMENT_PROOF_NOTE_COMMAND"
PROOF_NOTE_FILTER_ENV = "Z00Z_SETTLEMENT_PROOF_NOTE_FILTER"
SETTLEMENT_TIME_OUT_ENV = "Z00Z_SETTLEMENT_TIME_OUT"
SETTLEMENT_TIME_RUN_ENV = "Z00Z_SETTLEMENT_TIME_RUN"
DEFAULT_SCENARIO_CFG = (
    ROOT / "crates" / "z00z_simulator" / "src" / "scenario_1" / "scenario_config.yaml"
)
DEFAULT_SCENARIO_DESIGN = (
    ROOT / "crates" / "z00z_simulator" / "src" / "scenario_1" / "scenario_design.yaml"
)
DEFAULT_HJMT_HOME = ROOT / "config" / "hjmt_runtime" / "sim_5a7s"
VARIANT_ROOT = ROOT / "target" / "settlement_hjmt_variants"
LANE_CRITERION = "criterion_closure_timing"
LANE_COMMAND = "whole_command_resource"
LANE_SCENARIO = "scenario_stage_runtime"
LANE_THROUGHPUT = "user_facing_throughput"
SCENARIO_EVIDENCE_STAGE_IDS = {13}
SCENARIO_EVIDENCE_STAGE_NAMES = {"hjmt_settlement_examples"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run storage settlement benches and store logs and markdown reports in crates/z00z_storage/outputs/settlement.",
    )
    parser.add_argument(
        "--bench",
        required=True,
        choices=[
            "settlement_shard",
            "settlement_nested",
            "settlement_hjmt",
            "settlement_proofs",
            "adaptive_policy_bench",
            "scenario_1",
            "hjmt_mapping_ab",
        ],
    )
    parser.add_argument("--backend-mode", choices=["hjmt"])
    parser.add_argument("--bucket-bits")
    parser.add_argument("--bench-mode")
    parser.add_argument("--root-mode")
    parser.add_argument(
        "--shard-mapping",
        choices=["aggregator_owned", "shard_process"],
    )
    parser.add_argument("--baseline")
    parser.add_argument("--log-base")
    parser.add_argument("--no-run", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("criterion_args", nargs=argparse.REMAINDER)
    return parser.parse_args()


def pick_base(args: argparse.Namespace) -> str:
    if args.log_base:
        return args.log_base
    if args.baseline:
        return args.baseline
    if args.bench == "hjmt_mapping_ab":
        return "hjmt_mapping_ab"
    if args.bench == "scenario_1":
        mapping = f"_{args.shard_mapping}" if args.shard_mapping else ""
        return f"scenario_1_hjmt_workload{mapping}"
    parts = [args.bench]
    if args.backend_mode:
        parts.append(args.backend_mode)
    if args.bucket_bits:
        parts.append(f"bucket_bits_{args.bucket_bits}")
    if args.bench_mode:
        parts.append(args.bench_mode)
    if args.root_mode:
        parts.append(args.root_mode)
    if args.no_run:
        parts.append("no_run")
    return "_".join(parts)


def build_cmd(args: argparse.Namespace) -> list[str]:
    if args.bench == "hjmt_mapping_ab":
        return []
    if args.bench == "scenario_1":
        return [
            "cargo",
            "run",
            "--release",
            "-p",
            "z00z_simulator",
            "--bin",
            "scenario_1",
            "--features",
            "test-params-fast",
            "--features",
            "wallet_debug_tools",
            "--",
        ]
    cmd = ["cargo", "bench", "-p", "z00z_storage", "--bench", args.bench]
    extra = criterion_args(args)
    if args.no_run:
        cmd.append("--no-run")
    elif args.baseline:
        cmd.extend(["--", "--save-baseline", args.baseline, *extra])
    elif extra:
        cmd.extend(["--", *extra])
    return cmd


def criterion_args(args: argparse.Namespace) -> list[str]:
    extra = list(args.criterion_args)
    if extra and extra[0] == "--":
        extra = extra[1:]
    validate_quick_args(extra)
    return extra


def validate_quick_args(extra: list[str]) -> None:
    if "--quick" not in extra:
        return
    if "--sample-size" in extra:
        raise SystemExit(
            "--quick cannot be combined with --sample-size; remove one of them so the measured command stays exact."
        )


def criterion_filter_from(extra: list[str]) -> str | None:
    if extra and not extra[0].startswith("-"):
        return extra[0]
    return None


def criterion_filter(args: argparse.Namespace) -> str | None:
    return criterion_filter_from(criterion_args(args))


def is_batch_note_source_filter(filter_arg: str | None) -> bool:
    if not filter_arg:
        return False
    return filter_arg == "hjmt_batch_" or filter_arg.startswith("hjmt_batch_proof_bytes")


def proof_note_scope(args: argparse.Namespace) -> str | None:
    if args.bench != "settlement_proofs":
        return None
    filter_arg = criterion_filter(args)
    if filter_arg is None:
        return "full"
    if is_batch_note_source_filter(filter_arg):
        return "batch_only"
    return "skip"


def validate_settlement_proofs_scope(args: argparse.Namespace) -> None:
    if args.bench != "settlement_proofs":
        return
    scope = proof_note_scope(args)
    if args.log_base == "settlement_proofs_batch" and scope != "batch_only":
        raise SystemExit(
            "settlement_proofs_batch requires a filter that includes hjmt_batch_proof_bytes lanes; use `hjmt_batch_` or `hjmt_batch_proof_bytes/...`."
        )


def timing_trace_path(base: str) -> Path:
    return OUT_DIR / f"{base}.timing.tsv"


def bench_env(args: argparse.Namespace, base: str) -> dict[str, str]:
    env = os.environ.copy()
    if args.backend_mode:
        env["Z00Z_SETTLEMENT_BACKEND_MODE"] = args.backend_mode
    if args.bucket_bits:
        env["Z00Z_SETTLEMENT_BUCKET_BITS"] = args.bucket_bits
    if args.bench_mode:
        env["Z00Z_SETTLEMENT_BENCH_MODE"] = args.bench_mode
    if args.root_mode:
        env["Z00Z_SETTLEMENT_ROOT_MODE"] = args.root_mode
    if args.baseline:
        env["Z00Z_SETTLEMENT_BASELINE"] = args.baseline
    proof_scope = proof_note_scope(args)
    if proof_scope:
        env[PROOF_NOTE_SCOPE_ENV] = proof_scope
    if args.bench != "scenario_1" and not args.no_run:
        env[SETTLEMENT_TIME_OUT_ENV] = str(timing_trace_path(base))
        env[SETTLEMENT_TIME_RUN_ENV] = base
    return env


def merge_keep_prefixes(existing: str, prefixes: list[str]) -> str:
    merged: list[str] = []
    for value in [existing, *prefixes]:
        for chunk in value.replace("\n", ";").split(";"):
            text = chunk.strip()
            if text and text not in merged:
                merged.append(text)
    return ";".join(merged)


def prepare_out_dir() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    keep = _keep_prefixes(OUT_KEEP_ENV)
    _clear_dir_contents(OUT_DIR, Path("."), keep)


def _keep_prefixes(env_key: str) -> list[Path]:
    raw = os.environ.get(env_key, "")
    prefixes: list[Path] = []
    for chunk in raw.replace("\n", ";").split(";"):
        text = chunk.strip()
        if not text:
            continue
        path = Path(text)
        if path.is_absolute() or any(part == ".." for part in path.parts):
            raise SystemExit(f"{env_key} must contain only relative output prefixes: {text}")
        normalized = Path(*[part for part in path.parts if part not in ("", ".")])
        if not normalized.parts:
            raise SystemExit(f"{env_key} must not contain empty prefixes")
        prefixes.append(normalized)
    return prefixes


def _clear_dir_contents(root: Path, rel_dir: Path, keep: list[Path]) -> None:
    for entry in root.iterdir():
        rel_path = entry.name if rel_dir == Path(".") else (rel_dir / entry.name).as_posix()
        rel = Path(rel_path)
        if any(_starts_with(rel, prefix) for prefix in keep):
            continue
        if entry.is_dir() and any(_starts_with(prefix, rel) for prefix in keep):
            _clear_dir_contents(entry, rel, keep)
            if not any(entry.iterdir()):
                entry.rmdir()
            continue
        if entry.is_dir():
            for nested in sorted(
                entry.rglob("*"), key=lambda path: path.as_posix(), reverse=True
            ):
                if nested.is_file() or nested.is_symlink():
                    nested.unlink()
                elif nested.is_dir():
                    nested.rmdir()
            entry.rmdir()
        else:
            entry.unlink()


def _starts_with(path: Path, prefix: Path) -> bool:
    try:
        path.relative_to(prefix)
        return True
    except ValueError:
        return False


def repo_rel(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def mapping_scope(args: argparse.Namespace) -> str:
    if not args.shard_mapping:
        return "unset"
    if args.bench == "scenario_1":
        return "runtime_config_variant"
    if args.bench == "hjmt_mapping_ab":
        return "ab_summary"
    return "report_label_only_storage_local_path"


def report_measurement_lanes(
    args: argparse.Namespace,
    criterion_results: list[dict[str, object]],
    time_metrics: dict[str, str],
    scenario_runtime_split: dict[str, object] | None,
) -> list[tuple[str, str]]:
    lanes: list[tuple[str, str]] = []
    if args.bench != "scenario_1" and criterion_results:
        lanes.append(
            (
                LANE_CRITERION,
                "Criterion repeated-loop closure timings for one benchmark lane.",
            )
        )
    if time_metrics:
        lanes.append(
            (
                LANE_COMMAND,
                "Whole-command `/usr/bin/time -v` resource metrics for the exact executed command.",
            )
        )
    if scenario_runtime_split is not None:
        lanes.append(
            (
                LANE_SCENARIO,
                "Scenario stage-runtime split from live stage.profile instrumentation.",
            )
        )
    if args.bench in {"settlement_shard", "hjmt_mapping_ab"}:
        lanes.append(
            (
                LANE_THROUGHPUT,
                "User-facing throughput claims backed only by `durable_root_published_tps` artifacts.",
            )
        )
    return lanes


def variant_root(base: str, mapping: str) -> Path:
    return VARIANT_ROOT / base / mapping


def clear_path(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def write_text(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


def copy_home_variant(base: str, mapping: str) -> tuple[Path, Path]:
    root = variant_root(base, mapping)
    clear_path(root)
    shutil.copytree(DEFAULT_HJMT_HOME, root, dirs_exist_ok=True)
    if mapping == "shard_process":
        rewrite_shard_process_home(root)
    rewrite_variant_home_paths(root)
    manifest_path = root / "manifest.json"
    return root, manifest_path


def rewrite_shard_process_home(home: Path) -> None:
    manifest_path = home / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    placement_rows = sorted(manifest["placement_rows"], key=lambda row: int(row["shard_id"]))
    source_aggs = {int(agg["aggregator_id"]): agg for agg in manifest["aggregators"]}
    shard_rows_by_new_agg: dict[int, list[dict[str, object]]] = {}
    split_meta: list[tuple[int, int, int]] = []
    next_agg_id = max(source_aggs) + 1
    primary_counts: dict[int, int] = {}

    for row in placement_rows:
        source_id = int(row["primary_aggregator_id"])
        prior_count = primary_counts.get(source_id, 0)
        if prior_count == 0:
            new_agg_id = source_id
        else:
            new_agg_id = next_agg_id
            next_agg_id += 1
        primary_counts[source_id] = prior_count + 1
        row["primary_aggregator_id"] = new_agg_id
        shard_rows_by_new_agg.setdefault(new_agg_id, []).append(row)
        split_meta.append((new_agg_id, source_id, int(row["shard_id"])))

    agg_root = home / "aggregators"
    if agg_root.exists():
        shutil.rmtree(agg_root)
    agg_root.mkdir(parents=True, exist_ok=True)

    aggregators = []
    for new_agg_id, source_id, shard_id in split_meta:
        row = shard_rows_by_new_agg[new_agg_id][0]
        listen_port = 7100 + new_agg_id
        cfg_rel = f"aggregators/agg-{new_agg_id}/aggregator-config.yaml"
        data_dir = f"var/hjmt_runtime/sim_5a7s/agg-{new_agg_id}/data"
        journal_path = f"var/hjmt_runtime/sim_5a7s/agg-{new_agg_id}/journal.redb"
        log_path = f"var/hjmt_runtime/sim_5a7s/agg-{new_agg_id}/aggregator.log"
        evidence_dir = f"var/hjmt_runtime/sim_5a7s/agg-{new_agg_id}/evidence"
        route_rel = "shard_route_tables/route-table-v1.canon.hex"
        route_digest = manifest["route_table_digest"]
        cfg_body = (
            f'aggregator_id: {new_agg_id}\n'
            'role: "aggregator"\n'
            "routing_generation: 1\n"
            "execution:\n"
            '  shard_mapping: "shard_process"\n'
            "shards:\n"
            f'  - shard_id: {shard_id}\n'
            f'    secondary_ids: [{", ".join(str(value) for value in row["secondary_ids"])}]\n'
            f'    expected_journal_lineage: "{row["expected_journal_lineage_hex"]}"\n'
            "network:\n"
            f'  listen_addr: "127.0.0.1:{listen_port}"\n'
            "paths:\n"
            f'  data_dir: "{data_dir}"\n'
            f'  journal_path: "{journal_path}"\n'
            f'  log_path: "{log_path}"\n'
            "lifecycle:\n"
            f'  start_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config {repo_rel(home / cfg_rel)} --planner-config {repo_rel(home / "planner" / "planner-config.yaml")} --storage-config {repo_rel(home / "storage" / "storage-config.yaml")}"\n'
            f'  restart_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config {repo_rel(home / cfg_rel)} --planner-config {repo_rel(home / "planner" / "planner-config.yaml")} --storage-config {repo_rel(home / "storage" / "storage-config.yaml")}"\n'
            "route:\n"
            f'  table_path: "{route_rel}"\n'
            f'  expected_digest: "{route_digest}"\n'
            "startup:\n"
            "  route_codec: true\n"
            "  placement: true\n"
            "  journal_lineage: true\n"
            "  backend_generation: true\n"
            "  proof_codec: true\n"
            "  handoff_ready: true\n"
            "  crypto_tags: true\n"
            "evidence:\n"
            f'  config_digest_file: "{evidence_dir}/config-digests.json"\n'
            f'  preflight_report_file: "{evidence_dir}/preflight-report.json"\n'
            "limits:\n"
            "  max_batch_ops: 128\n"
            "  max_inflight: 16\n"
        )
        write_text(home / cfg_rel, cfg_body)
        aggregators.append(
            {
                "aggregator_id": new_agg_id,
                "process_id": f"agg-{new_agg_id}",
                "cfg_path": repo_rel(home / cfg_rel),
                "listen_addr": f"127.0.0.1:{listen_port}",
                "data_dir": data_dir,
                "journal_path": journal_path,
                "log_path": log_path,
                "start_cmd": (
                    f"cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config "
                    f"{repo_rel(home / cfg_rel)} --planner-config {repo_rel(home / 'planner' / 'planner-config.yaml')} "
                    f"--storage-config {repo_rel(home / 'storage' / 'storage-config.yaml')}"
                ),
                "restart_cmd": (
                    f"cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config "
                    f"{repo_rel(home / cfg_rel)} --planner-config {repo_rel(home / 'planner' / 'planner-config.yaml')} "
                    f"--storage-config {repo_rel(home / 'storage' / 'storage-config.yaml')}"
                ),
                "shard_ids": [shard_id],
            }
        )

    manifest["shard_mapping"] = "shard_process"
    manifest["agg_ids"] = [agg["aggregator_id"] for agg in sorted(aggregators, key=lambda item: item["aggregator_id"])]
    manifest["aggregators"] = sorted(aggregators, key=lambda item: item["aggregator_id"])
    manifest["placement_rows"] = placement_rows
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def rewrite_variant_home_paths(home: Path) -> None:
    planner_cfg = home / "planner" / "planner-config.yaml"
    storage_cfg = home / "storage" / "storage-config.yaml"
    for cfg_path in sorted(home.glob("aggregators/agg-*/aggregator-config.yaml")):
        body = cfg_path.read_text(encoding="utf-8")
        start_cmd = (
            f'  start_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config {cfg_path} '
            f'--planner-config {planner_cfg} --storage-config {storage_cfg}"'
        )
        restart_cmd = (
            f'  restart_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config {cfg_path} '
            f'--planner-config {planner_cfg} --storage-config {storage_cfg}"'
        )
        lines = []
        for line in body.splitlines():
            if line.startswith("  start_cmd: "):
                lines.append(start_cmd)
            elif line.startswith("  restart_cmd: "):
                lines.append(restart_cmd)
            else:
                lines.append(line)
        write_text(cfg_path, "\n".join(lines) + "\n")

    manifest_path = home / "manifest.json"
    if not manifest_path.exists():
        return
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["planner_config_path"] = str(planner_cfg)
    manifest["storage_config_path"] = str(storage_cfg)
    for agg in manifest.get("aggregators", []):
        agg_id = int(agg["aggregator_id"])
        cfg_path = home / "aggregators" / f"agg-{agg_id}" / "aggregator-config.yaml"
        agg["cfg_path"] = str(cfg_path)
        agg["start_cmd"] = (
            f"cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config {cfg_path} "
            f"--planner-config {planner_cfg} --storage-config {storage_cfg}"
        )
        agg["restart_cmd"] = agg["start_cmd"]
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def prepare_scenario_variant(base: str, mapping: str) -> tuple[Path, Path, Path]:
    home, manifest_path = copy_home_variant(base, mapping)
    cfg_root = variant_root(base, mapping) / "scenario"
    cfg_root.mkdir(parents=True, exist_ok=True)
    cfg_path = cfg_root / "scenario_config.yaml"
    source = DEFAULT_SCENARIO_CFG.read_text(encoding="utf-8")
    replaced = source.replace(
        '  config_root: "config/hjmt_runtime/sim_5a7s"',
        f'  config_root: "{home}"',
    )
    if replaced == source:
        raise SystemExit("failed to rewrite scenario_config.yaml hjmt_runtime.config_root")
    write_text(cfg_path, replaced)
    return cfg_path, DEFAULT_SCENARIO_DESIGN, manifest_path


def pull_times(lines: list[str]) -> list[str]:
    return [line.rstrip() for line in lines if "time:" in line]


def snapshot_criterion_state() -> dict[Path, int]:
    state: dict[Path, int] = {}
    if not CRITERION_DIR.exists():
        return state
    for benchmark_path in CRITERION_DIR.rglob("benchmark.json"):
        if benchmark_path.parent.name != "new":
            continue
        state[benchmark_path] = benchmark_path.stat().st_mtime_ns
    return state


def percentile(values: list[float], ratio: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    index = (len(ordered) - 1) * ratio
    lower = math.floor(index)
    upper = math.ceil(index)
    if lower == upper:
        return ordered[lower]
    blend = index - lower
    return ordered[lower] * (1.0 - blend) + ordered[upper] * blend


def load_criterion_result(benchmark_path: Path) -> dict[str, object]:
    benchmark = json.loads(benchmark_path.read_text(encoding="utf-8"))
    estimates = json.loads(
        benchmark_path.with_name("estimates.json").read_text(encoding="utf-8")
    )
    sample = json.loads(benchmark_path.with_name("sample.json").read_text(encoding="utf-8"))
    per_iter_ns = [
        float(total) / float(iters)
        for iters, total in zip(sample.get("iters", []), sample.get("times", []))
        if float(iters) > 0.0
    ]
    median_ns = float(estimates["median"]["point_estimate"])
    return {
        "full_id": benchmark["full_id"],
        "directory_name": benchmark["directory_name"],
        "mean_ns": float(estimates["mean"]["point_estimate"]),
        "median_ns": median_ns,
        "p50_ns": percentile(per_iter_ns, 0.50),
        "p95_ns": percentile(per_iter_ns, 0.95),
        "p99_ns": percentile(per_iter_ns, 0.99),
        "throughput_ops_s": (1_000_000_000.0 / median_ns) if median_ns > 0 else None,
    }


def collect_criterion_results(before: dict[Path, int]) -> list[dict[str, object]]:
    after = snapshot_criterion_state()
    changed_paths = [path for path, stamp in after.items() if before.get(path) != stamp]
    return sorted(
        [load_criterion_result(path) for path in changed_paths],
        key=lambda item: str(item["full_id"]),
    )


def format_ns(value: float | None) -> str:
    if value is None:
        return "n/a"
    if value < 1_000.0:
        return f"{value:.0f} ns"
    if value < 1_000_000.0:
        return f"{value / 1_000.0:.3f} us"
    if value < 1_000_000_000.0:
        return f"{value / 1_000_000.0:.3f} ms"
    return f"{value / 1_000_000_000.0:.3f} s"


def format_ops(value: float | None) -> str:
    if value is None:
        return "n/a"
    if value >= 1_000.0:
        return f"{value:,.0f} ops/s"
    return f"{value:.2f} ops/s"


def parse_time_metrics(lines: list[str]) -> dict[str, str]:
    labels = {
        "User time (seconds)": "user_time_s",
        "System time (seconds)": "system_time_s",
        "Percent of CPU this job got": "cpu_percent",
        "Elapsed (wall clock) time": "elapsed",
        "Maximum resident set size (kbytes)": "max_rss_kb",
        "File system inputs": "fs_inputs",
        "File system outputs": "fs_outputs",
        "Major (requiring I/O) page faults": "major_page_faults",
        "Minor (reclaiming a frame) page faults": "minor_page_faults",
        "Voluntary context switches": "voluntary_context_switches",
        "Involuntary context switches": "involuntary_context_switches",
    }
    metrics: dict[str, str] = {}
    for line in lines:
        text = line.strip()
        for label, key in labels.items():
            prefix = f"{label}:"
            if text.startswith(prefix):
                metrics[key] = text.split(":", 1)[1].strip()
                break
    return metrics


def parse_csv_kv(text: str) -> dict[str, str]:
    fields: dict[str, str] = {}
    for chunk in text.split(","):
        if "=" not in chunk:
            continue
        key, value = chunk.split("=", 1)
        fields[key.strip()] = value.strip()
    return fields


def is_evidence_stage(profile: dict[str, object]) -> bool:
    stage_id = profile.get("stage_id")
    stage_name = profile.get("stage_name")
    return stage_id in SCENARIO_EVIDENCE_STAGE_IDS or stage_name in SCENARIO_EVIDENCE_STAGE_NAMES


def parse_scenario_runtime_split(lines: list[str]) -> dict[str, object] | None:
    profiles: list[dict[str, object]] = []
    total_stage_elapsed_ms: int | None = None
    for line in lines:
        text = line.strip()
        if "stage.profile:" in text:
            payload = text.split("stage.profile:", 1)[1].strip()
            fields = parse_csv_kv(payload)
            try:
                stage_id = int(fields["id"])
                elapsed_ms = int(fields["elapsed_ms"])
            except (KeyError, ValueError):
                continue
            profiles.append(
                {
                    "stage_id": stage_id,
                    "stage_name": fields.get("name", "unknown"),
                    "elapsed_ms": elapsed_ms,
                    "result": fields.get("result", "unknown"),
                }
            )
            continue
        if "scenario.profile_total:" in text:
            payload = text.split("scenario.profile_total:", 1)[1].strip()
            fields = parse_csv_kv(payload)
            try:
                total_stage_elapsed_ms = int(fields["stage_elapsed_ms"])
            except (KeyError, ValueError):
                continue

    if not profiles and total_stage_elapsed_ms is None:
        return None

    evidence_profiles = [profile for profile in profiles if is_evidence_stage(profile)]
    protocol_profiles = [profile for profile in profiles if not is_evidence_stage(profile)]
    summed_stage_elapsed_ms = sum(
        int(profile["elapsed_ms"]) for profile in profiles if isinstance(profile.get("elapsed_ms"), int)
    )
    evidence_elapsed_ms = sum(
        int(profile["elapsed_ms"])
        for profile in evidence_profiles
        if isinstance(profile.get("elapsed_ms"), int)
    )
    live_protocol_elapsed_ms = sum(
        int(profile["elapsed_ms"])
        for profile in protocol_profiles
        if isinstance(profile.get("elapsed_ms"), int)
    )
    total = total_stage_elapsed_ms if total_stage_elapsed_ms is not None else summed_stage_elapsed_ms
    return {
        "total_stage_elapsed_ms": total,
        "summed_stage_elapsed_ms": summed_stage_elapsed_ms,
        "live_protocol_stage_elapsed_ms": live_protocol_elapsed_ms,
        "evidence_stage_elapsed_ms": evidence_elapsed_ms,
        "evidence_stage_ids": [profile["stage_id"] for profile in evidence_profiles],
        "evidence_stage_names": [profile["stage_name"] for profile in evidence_profiles],
        "live_protocol_stage_ids": [profile["stage_id"] for profile in protocol_profiles],
        "live_protocol_stage_names": [profile["stage_name"] for profile in protocol_profiles],
        "evidence_stage_share": (evidence_elapsed_ms / total) if total > 0 else None,
    }


def parse_internal_stage_timings(path: Path) -> tuple[list[dict[str, object]], list[str]]:
    if not path.exists():
        return [], []
    raw_lines = path.read_text(encoding="utf-8").splitlines()
    grouped: dict[str, list[float]] = {}
    for line in raw_lines:
        fields = {}
        for part in line.split("\t"):
            if "=" not in part:
                continue
            key, value = part.split("=", 1)
            fields[key] = value
        stage = fields.get("stage")
        ns_text = fields.get("ns")
        if not stage or not ns_text:
            continue
        try:
            ns = float(ns_text)
        except ValueError:
            continue
        grouped.setdefault(stage, []).append(ns)
    summary = [
        {
            "stage": stage,
            "samples": len(values),
            "min_ns": min(values),
            "p50_ns": percentile(values, 0.50),
            "p95_ns": percentile(values, 0.95),
            "max_ns": max(values),
        }
        for stage, values in grouped.items()
    ]
    return summary, raw_lines


def note_files(args: argparse.Namespace) -> list[str]:
    if args.bench == "settlement_hjmt":
        return ["settlement_hjmt_diag.md"]
    if args.bench == "settlement_proofs":
        return ["settlement_proof_sizes.md"]
    if args.bench == "settlement_nested":
        return ["settlement_nested_reload.md"]
    if args.bench == "settlement_shard":
        return ["settlement_shard_recovery.md"]
    return []


def read_note_sections(args: argparse.Namespace) -> list[tuple[str, str]]:
    sections: list[tuple[str, str]] = []
    for note in note_files(args):
        note_path = OUT_DIR / note
        if note_path.exists():
            sections.append((note, note_path.read_text(encoding="utf-8").rstrip()))
    return sections


def scenario_artifacts() -> list[tuple[str, Path]]:
    return [
        (
            "hjmt_settlement_examples.json",
            SCENARIO_OUT_DIR / "hjmt" / "hjmt_settlement_examples.json",
        ),
        (
            "hjmt_proof_size_report.json",
            SCENARIO_OUT_DIR / "hjmt" / "hjmt_proof_size_report.json",
        ),
        (
            "hjmt_cache_scheduler_metrics.json",
            SCENARIO_OUT_DIR / "hjmt" / "hjmt_cache_scheduler_metrics.json",
        ),
        (
            "hjmt_replay_roots.json",
            SCENARIO_OUT_DIR / "hjmt" / "hjmt_replay_roots.json",
        ),
        (
            "hjmt_tamper_report.json",
            SCENARIO_OUT_DIR / "hjmt" / "hjmt_tamper_report.json",
        ),
    ]


def snapshot_paths(paths: list[Path]) -> dict[Path, tuple[bool, int, int]]:
    snapshot: dict[Path, tuple[bool, int, int]] = {}
    for path in paths:
        if path.exists():
            stat = path.stat()
            snapshot[path] = (True, stat.st_mtime_ns, stat.st_size)
        else:
            snapshot[path] = (False, 0, 0)
    return snapshot


def summarize_json(prefix: str, value: object) -> list[str]:
    lines: list[str] = []
    if isinstance(value, dict):
        for key, nested in value.items():
            nested_prefix = f"{prefix}{key}"
            if isinstance(nested, (str, int, float, bool)) or nested is None:
                lines.append(f"- {nested_prefix}: `{nested}`")
            elif isinstance(nested, list):
                lines.append(f"- {nested_prefix}_count: `{len(nested)}`")
            elif isinstance(nested, dict):
                for inner_key, inner_value in nested.items():
                    if isinstance(inner_value, (str, int, float, bool)) or inner_value is None:
                        lines.append(f"- {nested_prefix}.{inner_key}: `{inner_value}`")
        return lines[:24]
    return [f"- {prefix.rstrip('.')}: `{value}`"]


def summarize_stage13_artifact(name: str, data: object) -> list[str]:
    if not isinstance(data, dict):
        return summarize_json("", data)

    lines = summarize_json("", data)
    if name == "hjmt_settlement_examples.json":
        comparison_rows = data.get("comparison_rows")
        if isinstance(comparison_rows, list):
            proof_surfaces = sorted(
                {
                    row.get("proof_surface")
                    for row in comparison_rows
                    if isinstance(row, dict) and isinstance(row.get("proof_surface"), str)
                }
            )
            batch_counts = sorted(
                {
                    row.get("path_count")
                    for row in comparison_rows
                    if isinstance(row, dict)
                    and row.get("proof_surface") == "batch_proof_v1"
                    and isinstance(row.get("path_count"), int)
                }
            )
            lines.append(f"- comparison_rows_count: `{len(comparison_rows)}`")
            lines.append(f"- comparison_proof_surfaces: `{proof_surfaces}`")
            lines.append(f"- batch_path_counts: `{batch_counts}`")
    if name == "hjmt_proof_size_report.json":
        comparison_rows = data.get("comparison_rows")
        if isinstance(comparison_rows, list):
            lines.append(f"- comparison_rows_count: `{len(comparison_rows)}`")
    if name == "hjmt_tamper_report.json":
        cases = data.get("cases")
        if isinstance(cases, list):
            case_ids = sorted(
                [
                    case.get("case_id")
                    for case in cases
                    if isinstance(case, dict) and isinstance(case.get("case_id"), str)
                ]
            )
            lines.append(f"- case_ids: `{case_ids}`")
    return lines[:32]


def read_scenario_sections(
    before: dict[Path, tuple[bool, int, int]],
) -> tuple[list[tuple[str, str, list[str]]], bool]:
    sections: list[tuple[str, str, list[str]]] = []
    fresh = True
    for name, path in scenario_artifacts():
        if not path.exists():
            sections.append((name, path.relative_to(ROOT).as_posix(), ["- missing: `true`"]))
            fresh = False
            continue
        stat = path.stat()
        existed_before, prev_mtime_ns, prev_size = before.get(path, (False, 0, 0))
        refreshed = (not existed_before) or stat.st_mtime_ns > prev_mtime_ns or stat.st_size != prev_size
        if not refreshed:
            fresh = False
        data = json.loads(path.read_text(encoding="utf-8"))
        summary = summarize_stage13_artifact(name, data)
        summary.insert(0, f"- refreshed_in_run: `{str(refreshed).lower()}`")
        sections.append(
            (
                name,
                path.relative_to(ROOT).as_posix(),
                summary,
            )
        )
    return sections, fresh


def write_report(
    args: argparse.Namespace,
    cmd: list[str],
    log_path: Path,
    report_path: Path,
    lines: list[str],
    criterion_results: list[dict[str, object]],
    internal_stage_timings: list[dict[str, object]],
    timing_lines: list[str],
    timing_path: Path | None,
    scenario_sections: list[tuple[str, str, list[str]]] | None = None,
    scenario_fresh: bool | None = None,
    scenario_runtime_split: dict[str, object] | None = None,
    scenario_runtime_split_path: Path | None = None,
    runtime_manifest_path: Path | None = None,
) -> None:
    time_metrics = parse_time_metrics(lines)
    measurement_lanes = report_measurement_lanes(
        args,
        criterion_results,
        time_metrics,
        scenario_runtime_split,
    )
    time_lines = pull_times(lines)
    report = [
        "# Storage Settlement Bench Run",
        "",
        f"- bench: `{args.bench}`",
        f"- output_dir: `crates/z00z_storage/outputs/settlement`",
        f"- log_file: `{log_path.relative_to(ROOT).as_posix()}`",
        f"- shard_mapping: `{args.shard_mapping or 'unset'}`",
        f"- shard_mapping_scope: `{mapping_scope(args)}`",
        (
            f"- runtime_manifest: `{repo_rel(runtime_manifest_path)}`"
            if runtime_manifest_path is not None
            else "- runtime_manifest: `unset`"
        ),
        f"- backend_mode: `{args.backend_mode or 'unset'}`",
        f"- bucket_bits: `{args.bucket_bits or 'unset'}`",
        f"- bench_mode: `{args.bench_mode or 'unset'}`",
        f"- root_mode: `{args.root_mode or 'unset'}`",
        f"- baseline: `{args.baseline or 'unset'}`",
        f"- no_run: `{str(args.no_run).lower()}`",
        f"- profiler: `{'/usr/bin/time -v' if TIME_BIN.exists() else 'unset'}`",
        "",
        "## Measurement Lanes",
        "",
    ]
    if measurement_lanes:
        report.extend(
            [
                "| lane | semantics |",
                "| --- | --- |",
            ]
        )
        for lane, semantics in measurement_lanes:
            report.append(f"| `{lane}` | {semantics} |")
    else:
        report.append("No measurement lanes were activated.")
    report.extend(
        [
            "",
        "## Command",
        "",
        "```bash",
        env_line(args),
        shlex.join(cmd),
        "```",
        "",
        "## Criterion Results",
        "",
        ]
    )
    if criterion_results:
        report.extend(
            [
                "| lane | mean | median | p50 | p95 | p99 | throughput |",
                "| --- | --- | --- | --- | --- | --- | --- |",
            ]
        )
        for result in criterion_results:
            report.append(
                "| {lane} | {mean} | {median} | {p50} | {p95} | {p99} | {throughput} |".format(
                    lane=result["full_id"],
                    mean=format_ns(result["mean_ns"]),
                    median=format_ns(result["median_ns"]),
                    p50=format_ns(result["p50_ns"]),
                    p95=format_ns(result["p95_ns"]),
                    p99=format_ns(result["p99_ns"]),
                    throughput=format_ops(result["throughput_ops_s"]),
                )
            )
    else:
        report.append("No Criterion JSON artifacts were updated.")
    report.extend(["", "## Resource Metrics", ""])
    if time_metrics:
        for key, value in time_metrics.items():
            report.append(f"- {key}: `{value}`")
    else:
        report.append("- time_metrics: `missing`")
    report.extend(["", "## Raw Timing Lines", "", "```text"])
    if time_lines:
        report.extend(time_lines)
    else:
        report.append("No timing lines were captured.")
    report.extend(["```", ""])
    report.extend(["## Internal Stage Timing Slices", ""])
    report.append(
        f"- timing_trace_file: `{timing_path.relative_to(ROOT).as_posix()}`"
        if timing_path is not None
        else "- timing_trace_file: `unset`"
    )
    if internal_stage_timings:
        report.extend(
            [
                "",
                "| stage | samples | min | p50 | p95 | max |",
                "| --- | --- | --- | --- | --- | --- |",
            ]
        )
        for row in internal_stage_timings:
            report.append(
                "| {stage} | {samples} | {min_v} | {p50_v} | {p95_v} | {max_v} |".format(
                    stage=row["stage"],
                    samples=row["samples"],
                    min_v=format_ns(row["min_ns"]),
                    p50_v=format_ns(row["p50_ns"]),
                    p95_v=format_ns(row["p95_ns"]),
                    max_v=format_ns(row["max_ns"]),
                )
            )
    else:
        report.append("")
        report.append("No internal stage timing rows were captured.")
    if timing_lines:
        report.extend(["", f"- timing_row_count: `{len(timing_lines)}`", ""])
    else:
        report.extend(["", "- timing_row_count: `0`", ""])
    note_sections = read_note_sections(args)
    if note_sections:
        report.extend(["## Note Files", ""])
        for note_name, content in note_sections:
            report.extend([f"### `{note_name}`", "", "```markdown", content, "```", ""])
    if args.bench == "scenario_1":
        report.extend(["## Scenario Provenance", ""])
        report.append(
            f"- artifacts_refreshed_in_run: `{str(bool(scenario_fresh)).lower()}`"
        )
        report.append(
            "- artifact_scope: `Stage 13 summaries stay evidence-only; live_protocol_stage_elapsed_ms excludes the Stage 13 evidence lane.`"
        )
        report.append("")
        report.extend(["## Scenario Runtime Split", ""])
        if scenario_runtime_split is not None:
            if scenario_runtime_split_path is not None:
                report.append(
                    f"- runtime_split_file: `{scenario_runtime_split_path.relative_to(ROOT).as_posix()}`"
                )
            report.append(
                f"- total_stage_elapsed_ms: `{scenario_runtime_split['total_stage_elapsed_ms']}`"
            )
            report.append(
                f"- live_protocol_stage_elapsed_ms: `{scenario_runtime_split['live_protocol_stage_elapsed_ms']}`"
            )
            report.append(
                f"- evidence_stage_elapsed_ms: `{scenario_runtime_split['evidence_stage_elapsed_ms']}`"
            )
            report.append(
                f"- evidence_stage_ids: `{scenario_runtime_split['evidence_stage_ids']}`"
            )
            report.append(
                f"- evidence_stage_names: `{scenario_runtime_split['evidence_stage_names']}`"
            )
            report.append(
                f"- live_protocol_stage_ids: `{scenario_runtime_split['live_protocol_stage_ids']}`"
            )
            report.append(
                f"- evidence_stage_share: `{scenario_runtime_split['evidence_stage_share']}`"
            )
        else:
            report.append("- scenario_runtime_split: `missing`")
        report.append("")
        report.extend(["## Scenario Artifacts", ""])
        for artifact_name, artifact_path, summary in scenario_sections or []:
            report.append(f"### `{artifact_name}`")
            report.append("")
            report.append(f"- artifact_path: `{artifact_path}`")
            report.extend(summary)
            report.append("")
    elif note_sections:
        report.append("")
    report_path.write_text("\n".join(report), encoding="utf-8")


def env_line(args: argparse.Namespace, base: str | None = None) -> str:
    parts: list[str] = []
    if args.backend_mode:
        parts.append(f"Z00Z_SETTLEMENT_BACKEND_MODE={shlex.quote(args.backend_mode)}")
    if args.bucket_bits:
        parts.append(f"Z00Z_SETTLEMENT_BUCKET_BITS={shlex.quote(args.bucket_bits)}")
    if args.bench_mode:
        parts.append(f"Z00Z_SETTLEMENT_BENCH_MODE={shlex.quote(args.bench_mode)}")
    if args.root_mode:
        parts.append(f"Z00Z_SETTLEMENT_ROOT_MODE={shlex.quote(args.root_mode)}")
    proof_scope = proof_note_scope(args)
    if proof_scope:
        parts.append(f"{PROOF_NOTE_SCOPE_ENV}={shlex.quote(proof_scope)}")
    if base is not None and args.bench != "scenario_1" and not args.no_run:
        parts.append(f"{SETTLEMENT_TIME_OUT_ENV}={shlex.quote(timing_trace_path(base).relative_to(ROOT).as_posix())}")
        parts.append(f"{SETTLEMENT_TIME_RUN_ENV}={shlex.quote(base)}")
    return " ".join(parts)


def parse_report_metric(path: Path, key: str) -> str | None:
    prefix = f"- {key}: `"
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith(prefix) and line.endswith("`"):
            return line[len(prefix) : -1]
    return None


def parse_stage_timing_metric(timing_path: Path, stage: str, field: str) -> str | None:
    rows, _ = parse_internal_stage_timings(timing_path)
    for row in rows:
        if row["stage"] == stage:
            value = row.get(field)
            if isinstance(value, (float, int)):
                return format_ns(float(value))
    return None


def parse_scaling_row(note_path: Path, lane: str) -> dict[str, str] | None:
    lines = note_path.read_text(encoding="utf-8").splitlines()
    header: list[str] | None = None
    for line in lines:
        if line.startswith("| lane | shard_count |"):
            header = [part.strip() for part in line.strip("|").split("|")]
            continue
        if header is None or not line.startswith(f"| {lane} |"):
            continue
        values = [part.strip() for part in line.strip("|").split("|")]
        if len(values) != len(header):
            continue
        return dict(zip(header, values))
    return None


def parse_required_ab_metrics(base: str, mapping: str) -> dict[str, str]:
    shard_report = OUT_DIR / f"settlement_shard_{mapping}.md"
    scenario_report = OUT_DIR / f"scenario_1_hjmt_workload_{mapping}.md"
    timing_path = OUT_DIR / f"settlement_shard_{mapping}.timing.tsv"
    note_path = OUT_DIR / f"settlement_shard_recovery_{mapping}.md"
    scaling_row = parse_scaling_row(note_path, "sim_5a7s") if note_path.exists() else None

    return {
        "mapping": mapping,
        "measurement_note": (
            "storage-local shard bench is process-model invariant; mapping-dependent numbers come from scenario whole-command resource runs."
        ),
        "durable_root_published_tps": (
            scaling_row.get("durable_root_published_tps", "not_measured")
            if scaling_row
            else "not_measured"
        ),
        "worker_local_tps": (
            scaling_row.get("worker_local_tps", "not_measured") if scaling_row else "not_measured"
        ),
        "publication_latency_us": (
            scaling_row.get("publication_latency_us", "not_measured")
            if scaling_row
            else "not_measured"
        ),
        "blocked_time_us": (
            scaling_row.get("blocked_time_us", "not_measured") if scaling_row else "not_measured"
        ),
        "hjmt_journal_sync_p50": (
            parse_stage_timing_metric(timing_path, "hjmt_journal_sync", "p50_ns")
            if timing_path.exists()
            else None
        )
        or "not_measured",
        "cpu_percent": parse_report_metric(scenario_report, "cpu_percent") or "not_measured",
        "max_rss_kb": parse_report_metric(scenario_report, "max_rss_kb") or "not_measured",
        "total_stage_elapsed_ms": (
            parse_report_metric(scenario_report, "total_stage_elapsed_ms") or "not_measured"
        ),
        "live_protocol_stage_elapsed_ms": (
            parse_report_metric(scenario_report, "live_protocol_stage_elapsed_ms")
            or "not_measured"
        ),
        "restart_time_us": parse_report_metric(note_path, "reload_time_us") or "not_measured",
        "failover_recovery_time_us": "not_measured",
        "runtime_manifest": parse_report_metric(scenario_report, "runtime_manifest") or "unset",
        "mapping_scope": parse_report_metric(scenario_report, "shard_mapping_scope") or "unset",
    }


def ab_subrun_args(
    bench: str,
    log_base: str,
    shard_mapping: str,
) -> argparse.Namespace:
    return argparse.Namespace(
        bench=bench,
        backend_mode=None,
        bucket_bits=None,
        bench_mode=None,
        root_mode=None,
        shard_mapping=shard_mapping,
        baseline=None,
        log_base=log_base,
        no_run=False,
        dry_run=False,
        criterion_args=(
            ["--sample-size", "10", "--warm-up-time", "0.01", "--measurement-time", "0.02"]
            if bench == "settlement_shard"
            else []
        ),
    )


def copy_recovery_note(mapping: str) -> Path:
    src = OUT_DIR / "settlement_shard_recovery.md"
    dst = OUT_DIR / f"settlement_shard_recovery_{mapping}.md"
    shutil.copyfile(src, dst)
    return dst


def write_ab_report(base: str, metrics: list[dict[str, str]]) -> None:
    report_path = OUT_DIR / f"{base}.md"
    json_path = OUT_DIR / f"{base}.json"
    verdict = (
        "keep `aggregator_owned` as production default: the current repository packet does not "
        "produce mapping-sensitive durable-throughput or failover-recovery evidence that justifies promotion."
    )
    report = [
        "# HJMT Shard Mapping A/B Packet",
        "",
        "- measurement_lane_primary: `user_facing_throughput`",
        "- measurement_lane_supporting: `whole_command_resource`, `scenario_stage_runtime`, `criterion_closure_timing`",
        "- fairness_contract: `same hardware, release profile, shard count, cache mode, persistence mode, and route generation; only shard_mapping changes across the two scenario runs.`",
        "- production_default_verdict: " + verdict,
        "- missing_metric_guard: `failover_recovery_time_us stays not_measured in this packet, so no production-default promotion is allowed.`",
        "",
        "| mapping | durable_root_published_tps | worker_local_tps | hjmt_journal_sync_p50 | publication_latency_us | blocked_time_us | cpu_percent | max_rss_kb | total_stage_elapsed_ms | restart_time_us | failover_recovery_time_us | runtime_manifest |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for item in metrics:
        report.append(
            "| {mapping} | {durable_root_published_tps} | {worker_local_tps} | {hjmt_journal_sync_p50} | {publication_latency_us} | {blocked_time_us} | {cpu_percent} | {max_rss_kb} | {total_stage_elapsed_ms} | {restart_time_us} | {failover_recovery_time_us} | `{runtime_manifest}` |".format(
                **item
            )
        )
    report.extend(
        [
            "",
            "## Notes",
            "",
            "- `settlement_shard_*` provides the durable throughput and `hjmt_journal_sync` slices, but its storage-local path does not load HJMT process topology; treat those numbers as throughput evidence, not as direct proof of process-placement benefit.",
            "- `scenario_1_hjmt_workload_*` provides whole-command CPU or RSS and stage-runtime splits with the selected runtime manifest captured in the report metadata.",
            "- Because `failover_recovery_time_us` is still `not_measured`, the packet is valid only as a no-promotion guard.",
        ]
    )
    write_text(report_path, "\n".join(report) + "\n")
    write_text(json_path, json.dumps({"verdict": verdict, "rows": metrics}, indent=2) + "\n")


def run_hjmt_mapping_ab(args: argparse.Namespace) -> int:
    prior_keep = os.environ.get(OUT_KEEP_ENV, "")
    keep_entries: list[str] = []
    try:
        for mapping in ["aggregator_owned", "shard_process"]:
            for bench in ["settlement_shard", "scenario_1"]:
                log_base = (
                    f"settlement_shard_{mapping}"
                    if bench == "settlement_shard"
                    else f"scenario_1_hjmt_workload_{mapping}"
                )
                os.environ[OUT_KEEP_ENV] = merge_keep_prefixes(prior_keep, keep_entries)
                code = run_bench(ab_subrun_args(bench, log_base, mapping))
                if code != 0:
                    return code
                keep_entries.extend(
                    [
                        f"{log_base}.log",
                        f"{log_base}.md",
                        f"{log_base}.timing.tsv",
                        f"{log_base}.runtime_split.json",
                    ]
                )
                if bench == "settlement_shard":
                    keep_entries.append(copy_recovery_note(mapping).name)
        metrics = [parse_required_ab_metrics(args.log_base or "hjmt_mapping_ab", mapping) for mapping in ["aggregator_owned", "shard_process"]]
        write_ab_report(args.log_base or "hjmt_mapping_ab", metrics)
        return 0
    finally:
        if prior_keep:
            os.environ[OUT_KEEP_ENV] = prior_keep
        else:
            os.environ.pop(OUT_KEEP_ENV, None)


def run_bench(args: argparse.Namespace) -> int:
    if args.bench == "hjmt_mapping_ab":
        return run_hjmt_mapping_ab(args)
    if args.bench == "scenario_1" and args.no_run:
        raise SystemExit("scenario_1 does not support --no-run")
    if args.bench == "scenario_1" and criterion_args(args):
        raise SystemExit("scenario_1 does not accept Criterion arguments")
    if args.bench == "settlement_proofs":
        validate_settlement_proofs_scope(args)
    base = pick_base(args)
    log_path = OUT_DIR / f"{base}.log"
    report_path = OUT_DIR / f"{base}.md"
    timing_path = timing_trace_path(base)
    scenario_runtime_split_path = OUT_DIR / f"{base}.runtime_split.json"
    cmd = build_cmd(args)
    runtime_manifest_path: Path | None = None
    if args.bench == "scenario_1":
        if args.shard_mapping:
            cfg_path, design_path, runtime_manifest_path = prepare_scenario_variant(
                base,
                args.shard_mapping,
            )
            cmd.extend(
                [
                    "--config",
                    str(cfg_path),
                    "--design",
                    str(design_path),
                ]
            )
        else:
            runtime_manifest_path = DEFAULT_HJMT_HOME / "manifest.json"
    env = bench_env(args, base)
    if args.bench == "settlement_proofs":
        env[PROOF_NOTE_COMMAND_ENV] = shlex.join(cmd)
        filter_arg = criterion_filter(args)
        if filter_arg:
            env[PROOF_NOTE_FILTER_ENV] = filter_arg
    env[OUT_KEEP_ENV] = merge_keep_prefixes(
        env.get(OUT_KEEP_ENV, ""),
        [
            log_path.name,
            report_path.name,
            timing_path.name,
            scenario_runtime_split_path.name,
        ],
    )
    criterion_before = (
        snapshot_criterion_state()
        if args.bench != "scenario_1" and not args.no_run
        else {}
    )
    scenario_before = (
        snapshot_paths([path for _, path in scenario_artifacts()])
        if args.bench == "scenario_1" and not args.no_run
        else {}
    )
    exec_cmd = [str(TIME_BIN), "-v", *cmd] if TIME_BIN.exists() else cmd

    if args.dry_run:
        print(f"log: {repo_rel(log_path)}")
        print(f"report: {repo_rel(report_path)}")
        print(f"proof_note_scope: {proof_note_scope(args) or 'unset'}")
        print(f"proof_note_filter: {criterion_filter(args) or 'unset'}")
        print(f"shard_mapping: {args.shard_mapping or 'unset'}")
        print(
            f"runtime_manifest: {repo_rel(runtime_manifest_path) if runtime_manifest_path is not None else 'unset'}"
        )
        env_text = env_line(args, base)
        if env_text:
            print(env_text)
        print(shlex.join(cmd))
        return 0

    prepare_out_dir()
    if timing_path.exists():
        timing_path.unlink()

    lines: list[str] = []
    with log_path.open("w", encoding="utf-8") as log_file:
        proc = subprocess.Popen(
            exec_cmd,
            cwd=ROOT,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        assert proc.stdout is not None
        for line in proc.stdout:
            sys.stdout.write(line)
            log_file.write(line)
            lines.append(line)
        code = proc.wait()

    criterion_results = []
    if args.bench != "scenario_1" and not args.no_run:
        criterion_results = collect_criterion_results(criterion_before)
    scenario_sections: list[tuple[str, str, list[str]]] | None = None
    scenario_fresh: bool | None = None
    scenario_runtime_split: dict[str, object] | None = None
    if args.bench == "scenario_1":
        scenario_sections, scenario_fresh = read_scenario_sections(scenario_before)
        scenario_runtime_split = parse_scenario_runtime_split(lines)
        if scenario_runtime_split is not None:
            scenario_runtime_split_path.write_text(
                json.dumps(scenario_runtime_split, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
    internal_stage_timings: list[dict[str, object]] = []
    timing_lines: list[str] = []
    report_timing_path: Path | None = None
    if args.bench != "scenario_1" and not args.no_run:
        internal_stage_timings, timing_lines = parse_internal_stage_timings(timing_path)
        report_timing_path = timing_path
    write_report(
        args,
        cmd,
        log_path,
        report_path,
        lines,
        criterion_results,
        internal_stage_timings,
        timing_lines,
        report_timing_path,
        scenario_sections,
        scenario_fresh,
        scenario_runtime_split,
        scenario_runtime_split_path if args.bench == "scenario_1" else None,
        runtime_manifest_path,
    )
    if args.bench == "scenario_1" and scenario_fresh is False:
        return 1
    return code


def main() -> int:
    args = parse_args()
    return run_bench(args)


if __name__ == "__main__":
    raise SystemExit(main())
