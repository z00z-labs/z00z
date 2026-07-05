use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tempfile::{tempdir, TempDir};
use z00z_rollup_node::{
    hjmt_process_ready_path, hjmt_process_stale_marker_path, hjmt_process_state_path,
    AggProc, AggRunArgs, HjmtProcessPersistedState, HjmtProcessReadyEvidence, NodeConfig,
    HJMT_PROCESS_HOLD_SECS_ENV, HJMT_PROCESS_MODE_ENV, HJMT_PROCESS_MODE_HOLD,
    HJMT_PROCESS_REJECT_STALE_ENV, HJMT_PROCESS_RUN_DIR_ENV, HJMT_PROCESS_RUN_ID_ENV,
    HJMT_PROCESS_STOP_ALL_FILE,
};

#[allow(dead_code)]
#[path = "support/test_hjmt_home.rs"]
mod hjmt_test_home;

use hjmt_test_home::{agg, write_hjmt_home_with_mapping};

const DEVNET_ARTIFACT_ROOT_ENV: &str = "Z00Z_HJMT_DEVNET_ARTIFACT_ROOT";
const DEVNET_TIMEOUT_SECS_ENV: &str = "Z00Z_HJMT_DEVNET_TIMEOUT_SECS";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RestartReport {
    aggregator_id: u16,
    first_boot_count: u64,
    second_boot_count: u64,
    persisted_state_reused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SmokeReport {
    profile: String,
    claim_level: String,
    process_model: String,
    run_id: String,
    home: PathBuf,
    manifest_path: PathBuf,
    binary_path: PathBuf,
    timeout_secs: u64,
    process_count: usize,
    processes: Vec<HjmtProcessReadyEvidence>,
    restart: RestartReport,
    note: String,
}

#[test]
fn sim_5a7s_process_devnet_smoke() {
    let output = output_root("process-smoke");
    let home = output.root.join("sim_5a7s");
    write_sim_5a7s_home(&home);
    let cfg = NodeConfig::from_hjmt_home(&home).expect("load cloned sim_5a7s home");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");
    let binary = binary_path();
    let timeout_secs = devnet_timeout_secs();
    let run_id = format!("sim-5a7s-process-{}", std::process::id());
    let run_root = output.root.join("run");
    fs::create_dir_all(&run_root).expect("create run root");

    let mut guard = ProcessGuard::default();
    for agg in &hjmt.aggs {
        guard.insert(spawn_process(&binary, agg, &run_root, Some(&run_id), timeout_secs, false));
    }

    let mut ready_by_id = BTreeMap::<u16, HjmtProcessReadyEvidence>::new();
    for agg in &hjmt.aggs {
        let ready = guard
            .child_mut(agg.aggregator_id.as_u16())
            .wait_for_ready(&run_root, 1, Duration::from_secs(10))
            .expect("wait for process ready");
        ready_by_id.insert(agg.aggregator_id.as_u16(), ready);
    }

    let mut listen_addrs = BTreeSet::new();
    let mut data_dirs = BTreeSet::new();
    let mut log_paths = BTreeSet::new();
    for ready in ready_by_id.values() {
        assert!(listen_addrs.insert(ready.listen_addr.clone()));
        assert!(data_dirs.insert(ready.data_dir.clone()));
        assert!(log_paths.insert(ready.log_path.clone()));
        assert_eq!(ready.profile, "SIM-5A7S");
        assert_eq!(ready.run_id, run_id);
        assert!(ready.ready_file.is_file());
        assert!(ready.state_file.is_file());
    }

    let first_ready = ready_by_id
        .get(&0)
        .cloned()
        .expect("agg-0 ready evidence");
    let killed = guard
        .child_mut(0)
        .kill_and_collect()
        .expect("kill agg-0 before restart");
    assert!(!killed.status.success());

    let restarted = spawn_process(
        &binary,
        hjmt.proc(z00z_aggregators::AggregatorId::new(0))
            .expect("agg-0 config"),
        &run_root,
        Some(&run_id),
        timeout_secs,
        false,
    );
    guard.insert(restarted);
    let second_ready = guard
        .child_mut(0)
        .wait_for_ready(&run_root, 2, Duration::from_secs(10))
        .expect("wait for restarted agg-0");
    assert_eq!(second_ready.boot_count, 2);
    assert!(second_ready.restarted_from_persisted_state);

    fs::write(run_root.join(HJMT_PROCESS_STOP_ALL_FILE), b"stop-all").expect("write stop-all");
    let mut processes = ready_by_id.into_values().collect::<Vec<_>>();
    processes.sort_by_key(|item| item.aggregator_id);
    for child in guard.children.values_mut() {
        child.wait_success().expect("stop child cleanly");
    }
    if let Some(entry) = processes.iter_mut().find(|item| item.aggregator_id == 0) {
        *entry = second_ready.clone();
    }

    let report = SmokeReport {
        profile: "SIM-5A7S".to_string(),
        claim_level: "local simulated-full".to_string(),
        process_model: "os_process".to_string(),
        run_id,
        home: home.clone(),
        manifest_path: home.join("manifest.json"),
        binary_path: binary,
        timeout_secs,
        process_count: processes.len(),
        processes,
        restart: RestartReport {
            aggregator_id: 0,
            first_boot_count: first_ready.boot_count,
            second_boot_count: second_ready.boot_count,
            persisted_state_reused: second_ready.restarted_from_persisted_state,
        },
        note: "OS-process identity, restart, and cleanup are live here; quorum and partition truth stay bound to scenario_11 deterministic evidence.".to_string(),
    };
    let report_path = output.root.join("process-devnet-smoke.json");
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("serialize smoke report"),
    )
    .expect("write smoke report");

    assert_eq!(report.process_count, 5);
    assert_eq!(report.restart.first_boot_count, 1);
    assert_eq!(report.restart.second_boot_count, 2);
    assert!(report.restart.persisted_state_reused);
    assert!(report.manifest_path.is_file());
}

#[test]
fn duplicate_ports_reject_before_devnet() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_duplicate_ports");
    write_hjmt_home_with_mapping(
        &home,
        1,
        &[
            agg(0, 7300, &[(0, &[1])]),
            agg(1, 7300, &[(1, &[0])]),
        ],
        Some("aggregator_owned"),
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("duplicate listen addr must reject");
    assert!(err.to_string().contains("duplicate listen addr"));
}

#[test]
fn missing_binary_rejects_before_devnet_claim() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_missing_binary");
    write_two_agg_home(&home);
    let cfg = NodeConfig::from_hjmt_home(&home).expect("load test home");
    let agg = cfg
        .hjmt
        .as_ref()
        .expect("hjmt config")
        .proc(z00z_aggregators::AggregatorId::new(0))
        .expect("agg-0");
    let run_root = temp.path().join("run");
    let err = spawn_process_with_binary(
        Path::new("/definitely/missing/z00z_rollup_node"),
        agg,
        &run_root,
        Some("missing-binary"),
        3,
        false,
    )
    .expect_err("missing binary must reject before claim");
    assert!(err.contains("No such file") || err.contains("os error 2"));
}

#[test]
fn stale_process_data_dir_rejects_on_restart() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_stale_restart");
    write_two_agg_home(&home);
    let cfg = NodeConfig::from_hjmt_home(&home).expect("load test home");
    let agg = cfg
        .hjmt
        .as_ref()
        .expect("hjmt config")
        .proc(z00z_aggregators::AggregatorId::new(0))
        .expect("agg-0")
        .clone();
    let run_root = temp.path().join("run");
    fs::create_dir_all(&run_root).expect("create run root");

    let mut first = spawn_process(&binary_path(), &agg, &run_root, Some("stale-restart"), 4, false);
    first
        .wait_for_ready(&run_root, 1, Duration::from_secs(10))
        .expect("wait for ready");
    fs::write(run_root.join(HJMT_PROCESS_STOP_ALL_FILE), b"stop-all").expect("stop-all");
    first.wait_success().expect("first stop cleanly");

    fs::write(hjmt_process_stale_marker_path(&agg.paths.data_dir), b"stale").expect("stale marker");
    let mut second = spawn_process(&binary_path(), &agg, &run_root, Some("stale-restart"), 4, true);
    let output = second.wait_output().expect("wait for stale restart output");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(stderr.contains("stale process data dir rejects on restart"));
}

#[test]
fn missing_run_id_observability_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_missing_run_id");
    write_two_agg_home(&home);
    let cfg = NodeConfig::from_hjmt_home(&home).expect("load test home");
    let agg = cfg
        .hjmt
        .as_ref()
        .expect("hjmt config")
        .proc(z00z_aggregators::AggregatorId::new(0))
        .expect("agg-0");
    let mut child = spawn_process_with_binary(
        &binary_path(),
        agg,
        &temp.path().join("run"),
        None,
        4,
        false,
    )
    .expect("spawn without run id");
    let output = child.wait_output().expect("wait output");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(stderr.contains(HJMT_PROCESS_RUN_ID_ENV));
}

fn write_sim_5a7s_home(home: &Path) {
    write_hjmt_home_with_mapping(
        home,
        1,
        &[
            agg(0, 7100, &[(0, &[1, 2]), (5, &[2, 4])]),
            agg(1, 7101, &[(1, &[0, 3])]),
            agg(2, 7102, &[(2, &[0, 4])]),
            agg(3, 7103, &[(3, &[1, 4])]),
            agg(4, 7104, &[(4, &[2, 3]), (6, &[0, 2])]),
        ],
        Some("aggregator_owned"),
    );
    write_manifest(home);
}

fn write_two_agg_home(home: &Path) {
    write_hjmt_home_with_mapping(
        home,
        1,
        &[agg(0, 7200, &[(0, &[1])]), agg(1, 7201, &[(1, &[0])])],
        Some("aggregator_owned"),
    );
    write_manifest(home);
}

fn write_manifest(home: &Path) {
    let cfg = NodeConfig::from_hjmt_home(home).expect("load generated home for manifest");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");
    let placement_rows = hjmt
        .aggs
        .iter()
        .flat_map(|agg| {
            agg.shards.iter().map(|shard| {
                serde_json::json!({
                    "shard_id": shard.shard_id.as_u16(),
                    "primary_aggregator_id": agg.aggregator_id.as_u16(),
                    "secondary_ids": shard.secondary_ids.iter().map(|item| item.as_u16()).collect::<Vec<_>>(),
                    "expected_journal_lineage_hex": hex::encode(shard.expected_journal_lineage),
                })
            })
        })
        .collect::<Vec<_>>();
    let aggregators = hjmt
        .aggs
        .iter()
        .map(|agg| {
            serde_json::json!({
                "aggregator_id": agg.aggregator_id.as_u16(),
                "process_id": format!("agg-{}", agg.aggregator_id.as_u16()),
                "cfg_path": agg.cfg_path,
                "listen_addr": agg.network.listen_addr,
                "data_dir": agg.paths.data_dir,
                "journal_path": agg.paths.journal_path,
                "log_path": agg.paths.log_path,
                "start_cmd": agg.lifecycle.start_cmd,
                "restart_cmd": agg.lifecycle.restart_cmd,
                "shard_ids": agg.shards.iter().map(|shard| shard.shard_id.as_u16()).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    let manifest = serde_json::json!({
        "version": 1,
        "profile": hjmt.profile,
        "process_model": "os_process",
        "shard_mapping": hjmt.shard_mapping().as_str(),
        "planner_config_path": hjmt.planner.cfg_path,
        "storage_config_path": hjmt.storage.cfg_path,
        "placement_rows": placement_rows,
        "aggregators": aggregators,
    });
    fs::write(
        home.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
}

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_z00z_rollup_node"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn devnet_timeout_secs() -> u64 {
    std::env::var(DEVNET_TIMEOUT_SECS_ENV)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(8)
}

fn spawn_process(
    binary: &Path,
    agg: &AggProc,
    run_root: &Path,
    run_id: Option<&str>,
    timeout_secs: u64,
    reject_stale: bool,
) -> ManagedChild {
    spawn_process_with_binary(binary, agg, run_root, run_id, timeout_secs, reject_stale)
        .expect("spawn process")
}

fn spawn_process_with_binary(
    binary: &Path,
    agg: &AggProc,
    run_root: &Path,
    run_id: Option<&str>,
    timeout_secs: u64,
    reject_stale: bool,
) -> Result<ManagedChild, String> {
    fs::create_dir_all(run_root).map_err(|err| format!("create {}: {err}", run_root.display()))?;
    let parsed = AggRunArgs::parse_life_cmd(&agg.lifecycle.start_cmd)
        .map_err(|err| format!("parse lifecycle command: {err}"))?;
    let mut cmd = Command::new(binary);
    cmd.current_dir(repo_root())
        .arg("--mode")
        .arg("aggregator")
        .arg("--aggregator-config")
        .arg(&parsed.aggregator_cfg)
        .arg("--planner-config")
        .arg(&parsed.planner_cfg)
        .arg("--storage-config")
        .arg(&parsed.storage_cfg)
        .env(HJMT_PROCESS_MODE_ENV, HJMT_PROCESS_MODE_HOLD)
        .env(HJMT_PROCESS_RUN_DIR_ENV, run_root)
        .env(HJMT_PROCESS_HOLD_SECS_ENV, timeout_secs.to_string())
        .env(HJMT_PROCESS_REJECT_STALE_ENV, if reject_stale { "1" } else { "0" })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(run_id) = run_id {
        cmd.env(HJMT_PROCESS_RUN_ID_ENV, run_id);
    }
    let child = cmd
        .spawn()
        .map_err(|err| format!("spawn {}: {err}", binary.display()))?;
    Ok(ManagedChild {
        aggregator_id: agg.aggregator_id.as_u16(),
        child: Some(child),
    })
}

struct OutputRoot {
    root: PathBuf,
    _temp: Option<TempDir>,
}

fn output_root(label: &str) -> OutputRoot {
    if let Ok(raw) = std::env::var(DEVNET_ARTIFACT_ROOT_ENV) {
        let root = PathBuf::from(raw);
        fs::create_dir_all(&root).expect("create artifact root");
        return OutputRoot { root, _temp: None };
    }
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join(label);
    fs::create_dir_all(&root).expect("create temp artifact root");
    OutputRoot {
        root,
        _temp: Some(temp),
    }
}

#[derive(Default)]
struct ProcessGuard {
    children: BTreeMap<u16, ManagedChild>,
}

impl ProcessGuard {
    fn insert(&mut self, child: ManagedChild) {
        self.children.insert(child.aggregator_id, child);
    }

    fn child_mut(&mut self, aggregator_id: u16) -> &mut ManagedChild {
        self.children
            .get_mut(&aggregator_id)
            .expect("managed child")
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        for child in self.children.values_mut() {
            child.kill_if_running();
        }
    }
}

#[derive(Debug)]
struct ManagedChild {
    aggregator_id: u16,
    child: Option<Child>,
}

impl ManagedChild {
    fn wait_for_ready(
        &mut self,
        run_root: &Path,
        min_boot_count: u64,
        timeout: Duration,
    ) -> Result<HjmtProcessReadyEvidence, String> {
        let ready_path = hjmt_process_ready_path(run_root, self.aggregator_id);
        let started = Instant::now();
        loop {
            if ready_path.is_file() {
                let bytes = fs::read(&ready_path)
                    .map_err(|err| format!("read {}: {err}", ready_path.display()))?;
                if let Ok(ready) = serde_json::from_slice::<HjmtProcessReadyEvidence>(&bytes) {
                    if ready.boot_count >= min_boot_count {
                        return Ok(ready);
                    }
                }
            }
            if started.elapsed() > timeout {
                return Err(format!(
                    "timed out waiting for aggregator {} ready file {}",
                    self.aggregator_id,
                    ready_path.display()
                ));
            }
            let Some(child) = self.child.as_mut() else {
                return Err(format!("aggregator {} child already consumed", self.aggregator_id));
            };
            if let Some(status) = child
                .try_wait()
                .map_err(|err| format!("poll child {}: {err}", self.aggregator_id))?
            {
                let output = collect_output(self.child.take().expect("child"), status)?;
                return Err(format!(
                    "aggregator {} exited before ready: status={} stderr={}",
                    self.aggregator_id,
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn kill_and_collect(&mut self) -> Result<Output, String> {
        let mut child = self.child.take().expect("child");
        child
            .kill()
            .map_err(|err| format!("kill child {}: {err}", self.aggregator_id))?;
        let status = child
            .wait()
            .map_err(|err| format!("wait child {}: {err}", self.aggregator_id))?;
        collect_output(child, status)
    }

    fn wait_success(&mut self) -> Result<(), String> {
        let output = self.wait_output()?;
        if !output.status.success() {
            return Err(format!(
                "aggregator {} exited unsuccessfully: {}",
                self.aggregator_id,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }

    fn wait_output(&mut self) -> Result<Output, String> {
        let child = self.child.take().expect("child");
        let output = child
            .wait_with_output()
            .map_err(|err| format!("wait_with_output child {}: {err}", self.aggregator_id))?;
        Ok(output)
    }

    fn kill_if_running(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.wait();
            let mut sink = String::new();
            if let Some(mut stdout) = child.stdout.take() {
                let _ = stdout.read_to_string(&mut sink);
            }
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_string(&mut sink);
            }
        }
    }
}

fn collect_output(mut child: Child, status: std::process::ExitStatus) -> Result<Output, String> {
    let mut stdout = Vec::new();
    if let Some(mut pipe) = child.stdout.take() {
        pipe.read_to_end(&mut stdout)
            .map_err(|err| format!("read stdout: {err}"))?;
    }
    let mut stderr = Vec::new();
    if let Some(mut pipe) = child.stderr.take() {
        pipe.read_to_end(&mut stderr)
            .map_err(|err| format!("read stderr: {err}"))?;
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

#[test]
fn persisted_state_file_tracks_boot_count() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_state");
    write_two_agg_home(&home);
    let cfg = NodeConfig::from_hjmt_home(&home).expect("load test home");
    let agg = cfg
        .hjmt
        .as_ref()
        .expect("hjmt config")
        .proc(z00z_aggregators::AggregatorId::new(0))
        .expect("agg-0")
        .clone();
    let run_root = temp.path().join("run");
    fs::create_dir_all(&run_root).expect("run root");

    let mut first = spawn_process(&binary_path(), &agg, &run_root, Some("state-track"), 4, false);
    first
        .wait_for_ready(&run_root, 1, Duration::from_secs(10))
        .expect("ready");
    fs::write(run_root.join(HJMT_PROCESS_STOP_ALL_FILE), b"stop-all").expect("stop-all");
    first.wait_success().expect("first clean stop");

    let state_path = hjmt_process_state_path(&agg.paths.data_dir);
    let first_state: HjmtProcessPersistedState =
        serde_json::from_slice(&fs::read(&state_path).expect("read state")).expect("state json");
    assert_eq!(first_state.boot_count, 1);

    let run_root2 = temp.path().join("run-2");
    let mut second = spawn_process(&binary_path(), &agg, &run_root2, Some("state-track"), 4, false);
    second
        .wait_for_ready(&run_root2, 2, Duration::from_secs(10))
        .expect("restart ready");
    fs::write(run_root2.join(HJMT_PROCESS_STOP_ALL_FILE), b"stop-all").expect("stop-all");
    second.wait_success().expect("second clean stop");

    let second_state: HjmtProcessPersistedState =
        serde_json::from_slice(&fs::read(&state_path).expect("read state")).expect("state json");
    assert_eq!(second_state.boot_count, 2);
}
