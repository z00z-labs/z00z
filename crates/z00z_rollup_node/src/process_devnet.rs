#![forbid(unsafe_code)]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{AggLaunch, AggProc};

pub const HJMT_PROCESS_MODE_ENV: &str = "Z00Z_HJMT_PROCESS_MODE";
pub const HJMT_PROCESS_MODE_HOLD: &str = "hold";
pub const HJMT_PROCESS_RUN_DIR_ENV: &str = "Z00Z_HJMT_RUN_DIR";
pub const HJMT_PROCESS_RUN_ID_ENV: &str = "Z00Z_HJMT_RUN_ID";
pub const HJMT_PROCESS_HOLD_SECS_ENV: &str = "Z00Z_HJMT_HOLD_SECS";
pub const HJMT_PROCESS_HEARTBEAT_MS_ENV: &str = "Z00Z_HJMT_HEARTBEAT_MS";
pub const HJMT_PROCESS_REJECT_STALE_ENV: &str = "Z00Z_HJMT_REJECT_STALE_RESTART";
pub const HJMT_PROCESS_READY_FILE: &str = "process-ready.json";
pub const HJMT_PROCESS_HEARTBEAT_FILE: &str = "process-heartbeat.json";
pub const HJMT_PROCESS_EVENTS_FILE: &str = "process-events.jsonl";
pub const HJMT_PROCESS_STATE_FILE: &str = "process-state.json";
pub const HJMT_PROCESS_STOP_FILE: &str = "stop";
pub const HJMT_PROCESS_STOP_ALL_FILE: &str = "stop-all";
pub const HJMT_PROCESS_STALE_MARKER_FILE: &str = "stale-process-dir.marker";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HjmtProcessPersistedState {
    pub profile: String,
    pub run_id: String,
    pub aggregator_id: u16,
    pub boot_count: u64,
    pub pid: u32,
    pub listen_addr: String,
    pub data_dir: PathBuf,
    pub journal_path: PathBuf,
    pub log_path: PathBuf,
    pub state_file: PathBuf,
    pub last_started_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HjmtProcessReadyEvidence {
    pub profile: String,
    pub run_id: String,
    pub evidence_scope: String,
    pub aggregator_id: u16,
    pub pid: u32,
    pub boot_count: u64,
    pub restarted_from_persisted_state: bool,
    pub listen_addr: String,
    pub data_dir: PathBuf,
    pub journal_path: PathBuf,
    pub log_path: PathBuf,
    pub state_file: PathBuf,
    pub ready_file: PathBuf,
    pub heartbeat_file: PathBuf,
    pub event_file: PathBuf,
    pub observed_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HjmtProcessHeartbeat {
    run_id: String,
    aggregator_id: u16,
    pid: u32,
    state: String,
    observed_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HjmtProcessEvent {
    run_id: String,
    aggregator_id: u16,
    pid: u32,
    boot_count: u64,
    event: String,
    observed_unix_ms: u64,
    detail: String,
}

pub fn hjmt_process_root(run_root: &Path, aggregator_id: u16) -> PathBuf {
    run_root.join(format!("agg-{aggregator_id}"))
}

pub fn hjmt_process_ready_path(run_root: &Path, aggregator_id: u16) -> PathBuf {
    hjmt_process_root(run_root, aggregator_id).join(HJMT_PROCESS_READY_FILE)
}

pub fn hjmt_process_heartbeat_path(run_root: &Path, aggregator_id: u16) -> PathBuf {
    hjmt_process_root(run_root, aggregator_id).join(HJMT_PROCESS_HEARTBEAT_FILE)
}

pub fn hjmt_process_event_path(run_root: &Path, aggregator_id: u16) -> PathBuf {
    hjmt_process_root(run_root, aggregator_id).join(HJMT_PROCESS_EVENTS_FILE)
}

pub fn hjmt_process_stop_path(run_root: &Path, aggregator_id: u16) -> PathBuf {
    hjmt_process_root(run_root, aggregator_id).join(HJMT_PROCESS_STOP_FILE)
}

pub fn hjmt_process_state_path(data_dir: &Path) -> PathBuf {
    data_dir.join(HJMT_PROCESS_STATE_FILE)
}

pub fn hjmt_process_stale_marker_path(data_dir: &Path) -> PathBuf {
    data_dir.join(HJMT_PROCESS_STALE_MARKER_FILE)
}

pub fn maybe_run_hjmt_process_devnet(launch: &AggLaunch) -> Result<bool, String> {
    let Some(mode) = env_value(HJMT_PROCESS_MODE_ENV) else {
        return Ok(false);
    };
    if mode != HJMT_PROCESS_MODE_HOLD {
        return Err(format!(
            "{HJMT_PROCESS_MODE_ENV} must stay `{HJMT_PROCESS_MODE_HOLD}` when process devnet mode is enabled; got `{mode}`"
        ));
    }

    let hjmt = launch.config.hjmt.as_ref().ok_or_else(|| {
        "hjmt config must be loaded for the live process devnet contract".to_string()
    })?;
    let agg = hjmt
        .proc(launch.aggregator_id)
        .ok_or_else(|| format!("unknown aggregator id {}", launch.aggregator_id.as_u16()))?;
    let run_id = required_env(HJMT_PROCESS_RUN_ID_ENV)?;
    let run_dir = PathBuf::from(required_env(HJMT_PROCESS_RUN_DIR_ENV)?);
    let hold_secs = parse_env_u64(HJMT_PROCESS_HOLD_SECS_ENV, 60)?;
    let heartbeat_ms = parse_env_u64(HJMT_PROCESS_HEARTBEAT_MS_ENV, 200)?;

    run_process_devnet(
        &hjmt.profile,
        agg,
        &run_dir,
        &run_id,
        hold_secs.max(1),
        heartbeat_ms.clamp(50, 1_000),
    )?;
    Ok(true)
}

fn run_process_devnet(
    profile: &str,
    agg: &AggProc,
    run_root: &Path,
    run_id: &str,
    hold_secs: u64,
    heartbeat_ms: u64,
) -> Result<(), String> {
    create_dir(run_root)?;
    create_dir(&agg.paths.data_dir)?;
    create_parent(&agg.paths.journal_path)?;
    create_parent(&agg.paths.log_path)?;

    let proc_root = hjmt_process_root(run_root, agg.aggregator_id.as_u16());
    create_dir(&proc_root)?;

    let state_path = hjmt_process_state_path(&agg.paths.data_dir);
    let stale_marker = hjmt_process_stale_marker_path(&agg.paths.data_dir);
    if state_path.is_file()
        && stale_marker.is_file()
        && env_value(HJMT_PROCESS_REJECT_STALE_ENV).as_deref() != Some("0")
    {
        return Err(format!(
            "stale process data dir rejects on restart for aggregator {}: {}",
            agg.aggregator_id.as_u16(),
            stale_marker.display(),
        ));
    }

    let previous = load_json::<HjmtProcessPersistedState>(&state_path)?;
    let boot_count = previous.as_ref().map_or(1, |state| state.boot_count + 1);
    let restarted_from_persisted_state = previous.is_some();
    let pid = std::process::id();
    let now_ms = unix_ms()?;

    let persisted = HjmtProcessPersistedState {
        profile: profile.to_string(),
        run_id: run_id.to_string(),
        aggregator_id: agg.aggregator_id.as_u16(),
        boot_count,
        pid,
        listen_addr: agg.network.listen_addr.clone(),
        data_dir: agg.paths.data_dir.clone(),
        journal_path: agg.paths.journal_path.clone(),
        log_path: agg.paths.log_path.clone(),
        state_file: state_path.clone(),
        last_started_unix_ms: now_ms,
    };
    save_json(&state_path, &persisted)?;

    let ready_path = hjmt_process_ready_path(run_root, agg.aggregator_id.as_u16());
    let heartbeat_path = hjmt_process_heartbeat_path(run_root, agg.aggregator_id.as_u16());
    let event_path = hjmt_process_event_path(run_root, agg.aggregator_id.as_u16());
    let ready = HjmtProcessReadyEvidence {
        profile: profile.to_string(),
        run_id: run_id.to_string(),
        evidence_scope: "local simulated-full process contract".to_string(),
        aggregator_id: agg.aggregator_id.as_u16(),
        pid,
        boot_count,
        restarted_from_persisted_state,
        listen_addr: agg.network.listen_addr.clone(),
        data_dir: agg.paths.data_dir.clone(),
        journal_path: agg.paths.journal_path.clone(),
        log_path: agg.paths.log_path.clone(),
        state_file: state_path.clone(),
        ready_file: ready_path.clone(),
        heartbeat_file: heartbeat_path.clone(),
        event_file: event_path.clone(),
        observed_unix_ms: now_ms,
    };
    save_json(&ready_path, &ready)?;
    write_heartbeat(&heartbeat_path, run_id, agg.aggregator_id.as_u16(), pid, "running")?;
    append_event(
        &event_path,
        run_id,
        agg.aggregator_id.as_u16(),
        pid,
        boot_count,
        "started",
        format!(
            "listen_addr={} boot_count={} restarted_from_persisted_state={}",
            agg.network.listen_addr, boot_count, restarted_from_persisted_state
        ),
    )?;
    append_log_line(
        &agg.paths.log_path,
        &format!(
            "run_id={run_id} aggregator_id={} event=started pid={pid} boot_count={boot_count} listen_addr={}",
            agg.aggregator_id.as_u16(),
            agg.network.listen_addr,
        ),
    )?;

    let stop_all = run_root.join(HJMT_PROCESS_STOP_ALL_FILE);
    let stop_one = hjmt_process_stop_path(run_root, agg.aggregator_id.as_u16());
    let deadline = Instant::now() + Duration::from_secs(hold_secs);
    let stop_reason = loop {
        if stop_all.is_file() {
            break "stop_all";
        }
        if stop_one.is_file() {
            break "stop";
        }
        if Instant::now() >= deadline {
            break "timeout";
        }
        thread::sleep(Duration::from_millis(heartbeat_ms));
        write_heartbeat(&heartbeat_path, run_id, agg.aggregator_id.as_u16(), pid, "running")?;
    };

    write_heartbeat(
        &heartbeat_path,
        run_id,
        agg.aggregator_id.as_u16(),
        pid,
        stop_reason,
    )?;
    append_event(
        &event_path,
        run_id,
        agg.aggregator_id.as_u16(),
        pid,
        boot_count,
        "stopped",
        format!("reason={stop_reason}"),
    )?;
    append_log_line(
        &agg.paths.log_path,
        &format!(
            "run_id={run_id} aggregator_id={} event=stopped pid={pid} boot_count={boot_count} reason={stop_reason}",
            agg.aggregator_id.as_u16(),
        ),
    )?;
    Ok(())
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn required_env(name: &str) -> Result<String, String> {
    env_value(name).ok_or_else(|| format!("{name} must be present for process devnet mode"))
}

fn parse_env_u64(name: &str, default: u64) -> Result<u64, String> {
    match env_value(name) {
        Some(raw) => raw
            .parse::<u64>()
            .map_err(|err| format!("{name} must parse as u64: {err}")),
        None => Ok(default),
    }
}

fn create_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|err| format!("failed to create {}: {err}", path.display()))
}

fn create_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("missing parent for {}", path.display()))?;
    create_dir(parent)
}

fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    create_parent(path)?;
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| format!("failed to serialize {}: {err}", path.display()))?;
    fs::write(path, bytes).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|err| format!("failed to decode {}: {err}", path.display()))
}

fn write_heartbeat(
    path: &Path,
    run_id: &str,
    aggregator_id: u16,
    pid: u32,
    state: &str,
) -> Result<(), String> {
    save_json(
        path,
        &HjmtProcessHeartbeat {
            run_id: run_id.to_string(),
            aggregator_id,
            pid,
            state: state.to_string(),
            observed_unix_ms: unix_ms()?,
        },
    )
}

fn append_event(
    path: &Path,
    run_id: &str,
    aggregator_id: u16,
    pid: u32,
    boot_count: u64,
    event: &str,
    detail: String,
) -> Result<(), String> {
    create_parent(path)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| format!("failed to open {}: {err}", path.display()))?;
    let line = serde_json::to_string(&HjmtProcessEvent {
        run_id: run_id.to_string(),
        aggregator_id,
        pid,
        boot_count,
        event: event.to_string(),
        observed_unix_ms: unix_ms()?,
        detail,
    })
    .map_err(|err| format!("failed to serialize {}: {err}", path.display()))?;
    writeln!(file, "{line}").map_err(|err| format!("failed to append {}: {err}", path.display()))
}

fn append_log_line(path: &Path, line: &str) -> Result<(), String> {
    create_parent(path)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| format!("failed to open {}: {err}", path.display()))?;
    writeln!(file, "{line}").map_err(|err| format!("failed to append {}: {err}", path.display()))
}

fn unix_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .map_err(|err| format!("system clock drifted before unix epoch: {err}"))
}
