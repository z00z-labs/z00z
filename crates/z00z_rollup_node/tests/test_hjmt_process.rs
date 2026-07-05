use std::{collections::BTreeSet, process::Command};

use tempfile::tempdir;
use z00z_rollup_node::{canonical_run_cmd, AggRunArgs, NodeConfig, ShardMapping};
use z00z_utils::io;

#[path = "support/test_hjmt_home.rs"]
mod hjmt_test_home;

use hjmt_test_home::{agg, repo_hjmt_home, write_hjmt_home, write_hjmt_home_with_mapping};

#[test]
fn test_keeps_explicit_paths() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo SIM-5A7S home");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    assert_eq!(hjmt.shard_mapping(), ShardMapping::AggregatorOwned);

    let mut cfg_paths = BTreeSet::new();
    let mut data_dirs = BTreeSet::new();
    let mut journal_paths = BTreeSet::new();
    let mut log_paths = BTreeSet::new();
    let mut listen_addrs = BTreeSet::new();

    for agg in &hjmt.aggs {
        assert!(cfg_paths.insert(agg.cfg_path.clone()));
        assert!(data_dirs.insert(agg.paths.data_dir.clone()));
        assert!(journal_paths.insert(agg.paths.journal_path.clone()));
        assert!(log_paths.insert(agg.paths.log_path.clone()));
        assert!(listen_addrs.insert(agg.network.listen_addr.clone()));
        assert_eq!(agg.role, "aggregator");
        assert!(agg.startup.all_enabled());
        assert!(agg.route.table_path.is_some());
        assert!(agg.route.expected_digest.is_some());
        assert!(agg
            .evidence
            .config_digest_file
            .to_string_lossy()
            .contains("config-digests.json"));
        assert!(agg
            .evidence
            .preflight_report_file
            .to_string_lossy()
            .contains("preflight-report.json"));
        let expect_cmd = canonical_run_cmd(
            &agg.cfg_path,
            &hjmt.planner.cfg_path,
            &hjmt.storage.cfg_path,
        );
        assert_eq!(agg.lifecycle.start_cmd, expect_cmd);
        assert_eq!(agg.lifecycle.restart_cmd, expect_cmd);
        let parsed = AggRunArgs::parse_life_cmd(&agg.lifecycle.start_cmd).expect("parse start cmd");
        let launch = NodeConfig::from_agg_run_args(&parsed).expect("launch args");
        assert_eq!(launch.aggregator_id, agg.aggregator_id);
    }

    assert_eq!(cfg_paths.len(), hjmt.agg_count());
    assert_eq!(data_dirs.len(), hjmt.agg_count());
    assert_eq!(journal_paths.len(), hjmt.agg_count());
    assert_eq!(log_paths.len(), hjmt.agg_count());
    assert_eq!(listen_addrs.len(), hjmt.agg_count());
    assert_eq!(
        hjmt.planner
            .cfg_path
            .file_name()
            .and_then(|item| item.to_str()),
        Some("planner-config.yaml")
    );
    assert_eq!(
        hjmt.storage
            .cfg_path
            .file_name()
            .and_then(|item| item.to_str()),
        Some("storage-config.yaml")
    );
    assert!(hjmt
        .storage
        .paths
        .import_dir
        .to_string_lossy()
        .contains("import"));
    assert!(hjmt
        .storage
        .paths
        .lock_path
        .to_string_lossy()
        .contains("lock"));
    assert!(hjmt.storage.settings.cache_capacity > 0);
    assert!(hjmt.storage.settings.lock_timeout_ms > 0);
}

#[test]
fn test_defaults_owned_mapping() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_default_mapping");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7150, &[(0, &[1])]), agg(1, 7151, &[(1, &[0])])],
    );

    let cfg = NodeConfig::from_hjmt_home(&home).expect("default mapping");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    assert_eq!(hjmt.shard_mapping(), ShardMapping::AggregatorOwned);
}

#[test]
fn test_accepts_owned_mapping() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_explicit_owned");
    write_hjmt_home_with_mapping(
        &home,
        1,
        &[agg(0, 7160, &[(0, &[1])]), agg(1, 7161, &[(1, &[0])])],
        Some("aggregator_owned"),
    );

    let cfg = NodeConfig::from_hjmt_home(&home).expect("explicit owned mapping");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    assert_eq!(hjmt.shard_mapping(), ShardMapping::AggregatorOwned);
}

#[test]
fn unknown_shard_mapping_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_bad_mapping");
    write_hjmt_home_with_mapping(
        &home,
        1,
        &[agg(0, 7170, &[(0, &[1])]), agg(1, 7171, &[(1, &[0])])],
        Some("aggregator_owned"),
    );

    let agg_cfg = home.join("aggregators/agg-0/aggregator-config.yaml");
    let body = String::from_utf8(io::read_file(&agg_cfg).expect("read agg cfg")).expect("utf8");
    let body = body.replace("aggregator_owned", "invalid_mapping");
    io::write_file(&agg_cfg, body.as_bytes()).expect("agg cfg");

    let err = NodeConfig::from_hjmt_home(&home).expect_err("unknown mapping must reject");
    assert!(err.to_string().contains("unknown variant"));
}

#[test]
fn test_requires_all_refs() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_1a1s");
    write_home(
        &home,
        "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config agg-0/aggregator-config.yaml --storage-config storage/storage-config.yaml",
        "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config agg-0/aggregator-config.yaml --planner-config planner/planner-config.yaml",
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("missing config refs must reject");
    let msg = err.to_string();
    assert!(
        msg.contains("--planner-config must be present")
            || msg.contains("--storage-config must be present")
    );
}

#[test]
fn test_rejects_shadow_paths() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_1a1s_shadow");
    write_home(
        &home,
        "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config shadow/agg-0/aggregator-config.yaml --planner-config shadow/planner/planner-config.yaml --storage-config shadow/storage/storage-config.yaml",
        "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config shadow/agg-0/aggregator-config.yaml --planner-config shadow/planner/planner-config.yaml --storage-config shadow/storage/storage-config.yaml",
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("shadow cfg paths must reject");
    let msg = err.to_string();
    assert!(msg.contains("canonical aggregator/planner/storage config paths"));
}

#[test]
fn test_accepts_canonical_paths() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_1a1s_norm");
    write_home(
        &home,
        "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config aggregators/agg-0/../agg-0/./aggregator-config.yaml --planner-config planner/./planner-config.yaml --storage-config storage/../storage/storage-config.yaml",
        "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config aggregators/agg-0/./aggregator-config.yaml --planner-config planner/plan/../planner-config.yaml --storage-config ./storage/storage-config.yaml",
    );

    NodeConfig::from_hjmt_home(&home).expect("normalized canonical cfg paths must load");
}

#[test]
fn test_binary_help_runs() {
    let out = Command::new(env!("CARGO_BIN_EXE_z00z_rollup_node"))
        .arg("--help")
        .output()
        .expect("run help");

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("help utf8");
    assert!(stdout.contains("--mode"));
    assert!(stdout.contains("--aggregator-config"));
    assert!(stdout.contains("--planner-config"));
    assert!(stdout.contains("--storage-config"));
}

#[test]
fn test_repo_manifest_commands_run_binary_contract() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo SIM-5A7S home");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    for agg in &hjmt.aggs {
        assert_manifest_cmd_runs(&agg.lifecycle.start_cmd);
        assert_manifest_cmd_runs(&agg.lifecycle.restart_cmd);
    }
}

fn write_home(home: &std::path::Path, start_cmd: &str, restart_cmd: &str) {
    write_hjmt_home(home, 1, &[agg(0, 7100, &[(0, &[1])]), agg(1, 7101, &[])]);
    let agg_cfg = home.join("aggregators/agg-0/aggregator-config.yaml");
    let body = String::from_utf8(io::read_file(&agg_cfg).expect("read agg cfg")).expect("utf8");
    let body = body
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("start_cmd: ") {
                format!("  start_cmd: \"{start_cmd}\"")
            } else if line.trim_start().starts_with("restart_cmd: ") {
                format!("  restart_cmd: \"{restart_cmd}\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    io::write_file(&agg_cfg, body.as_bytes()).expect("agg cfg");
}

fn assert_manifest_cmd_runs(cmd: &str) {
    let argv = shell_words::split(cmd).expect("split manifest cmd");
    let sep = argv
        .iter()
        .position(|item| item == "--")
        .expect("cargo delimiter");
    let out = Command::new(env!("CARGO_BIN_EXE_z00z_rollup_node"))
        .current_dir(repo_root())
        .args(argv.iter().skip(sep + 1))
        .output()
        .expect("run binary contract");

    assert!(
        out.status.success(),
        "binary contract failed for `{cmd}`: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
