#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use z00z_aggregators::{
    AggregatorId, BatchRoute, SecondaryState, ShardId, ShardPlacement, ShardPlacementTable,
    ShardRouteTable,
};
use z00z_storage::{
    checkpoint::CheckpointId,
    settlement::{
        check_batch_contract_v1, check_handoff_route_v1, check_live_startup_contract,
        BatchProofBlobV1, ProofChkErr, PublicationHandoffRowV1, PublicationRouteSnapshotV1,
        RootGeneration, SettlementRecoveryState, HJMT_PROOF_ENVELOPE_VERSION,
    },
};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io,
};

use crate::mode::NodeMode;

pub use z00z_aggregators::PlannerMode;

const AGG_CFG_FILE: &str = "aggregator-config.yaml";
const PLAN_CFG_FILE: &str = "planner-config.yaml";
const STORE_CFG_FILE: &str = "storage-config.yaml";
const MANIFEST_FILE: &str = "manifest.json";
const ROUTE_LABEL: &str = "route-source";
const SCENARIO_LABEL: &str = "scenario-config";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub mode: NodeMode,
    pub da_provider: String,
    pub rpc_enabled: bool,
    pub hjmt: Option<HjmtCfg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcModel {
    OsProcess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShardMapping {
    #[default]
    AggregatorOwned,
    ShardProcess,
}

impl ShardMapping {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AggregatorOwned => "aggregator_owned",
            Self::ShardProcess => "shard_process",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeStat {
    pub profile: String,
    pub proc_model: ProcModel,
    pub agg_count: usize,
    pub shard_count: usize,
    pub routing_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HjmtCfg {
    pub home: PathBuf,
    pub profile: String,
    pub proc_model: ProcModel,
    pub planner: PlanCfg,
    pub storage: StoreCfg,
    pub aggs: Vec<AggProc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanCfg {
    pub cfg_path: PathBuf,
    pub mode: PlannerMode,
    pub routing_generation: u64,
    pub route: RouteRef,
    pub policy: PlanPolicy,
    pub limits: PlanLimits,
    pub paths: PlanPaths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreCfg {
    pub cfg_path: PathBuf,
    pub backend: String,
    pub generation: u64,
    pub paths: StorePaths,
    pub settings: StoreSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggProc {
    pub cfg_path: PathBuf,
    pub aggregator_id: AggregatorId,
    pub role: String,
    pub routing_generation: u64,
    pub execution: AggExecutionCfg,
    pub shards: Vec<ShardOwn>,
    pub network: NetCfg,
    pub paths: AggPaths,
    pub lifecycle: LifeCfg,
    pub route: RouteRef,
    pub startup: StartupCheckCfg,
    pub evidence: EvidenceCfg,
    pub limits: AggLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardOwn {
    pub shard_id: ShardId,
    pub secondary_ids: Vec<AggregatorId>,
    pub expected_journal_lineage: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AggExecutionCfg {
    #[serde(default)]
    pub shard_mapping: ShardMapping,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RouteRef {
    #[serde(default)]
    pub table_path: Option<PathBuf>,
    #[serde(default)]
    pub expected_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetCfg {
    pub listen_addr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggPaths {
    pub data_dir: PathBuf,
    pub journal_path: PathBuf,
    pub log_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifeCfg {
    pub start_cmd: String,
    pub restart_cmd: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupCheckCfg {
    pub route_codec: bool,
    pub placement: bool,
    pub journal_lineage: bool,
    pub backend_generation: bool,
    pub proof_codec: bool,
    pub handoff_ready: bool,
    pub crypto_tags: bool,
}

impl StartupCheckCfg {
    #[must_use]
    pub const fn all_enabled(&self) -> bool {
        self.route_codec
            && self.placement
            && self.journal_lineage
            && self.backend_generation
            && self.proof_codec
            && self.handoff_ready
            && self.crypto_tags
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceCfg {
    pub config_digest_file: PathBuf,
    pub preflight_report_file: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggLimits {
    pub max_batch_ops: usize,
    pub max_inflight: usize,
}

impl Default for AggLimits {
    fn default() -> Self {
        Self {
            max_batch_ops: 128,
            max_inflight: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanPolicy {
    pub shard_local_only: bool,
    pub reject_cross_shard: bool,
    pub cadence_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanLimits {
    pub max_batch_ops: usize,
    pub max_batch_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanPaths {
    pub plan_dir: PathBuf,
    pub evidence_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorePaths {
    pub data_dir: PathBuf,
    pub journal_dir: PathBuf,
    pub export_dir: PathBuf,
    pub import_dir: PathBuf,
    pub lock_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreSet {
    pub flush_each_batch: bool,
    pub sync_mode: String,
    pub compression: String,
    pub cache_capacity: usize,
    pub lock_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigDigestRecord {
    pub label: String,
    pub path: PathBuf,
    pub digest_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggRunArgs {
    pub aggregator_cfg: PathBuf,
    pub planner_cfg: PathBuf,
    pub storage_cfg: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggLaunch {
    pub config: NodeConfig,
    pub aggregator_id: AggregatorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PublicationHandoffMeta {
    pub shard_id: ShardId,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub checkpoint_id: CheckpointId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupPreflightInput<'a> {
    pub recovery: &'a SettlementRecoveryState,
    pub proof_bytes: &'a [u8],
    pub handoff: &'a [PublicationHandoffMeta],
    pub scenario_cfg_path: Option<&'a Path>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreflightCheck {
    pub name: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StartupPreflightReport {
    pub aggregator_id: AggregatorId,
    pub route_table_path: PathBuf,
    pub route_table_digest: String,
    pub config_digests: Vec<ConfigDigestRecord>,
    pub checks: Vec<PreflightCheck>,
    pub evidence: EvidenceCfg,
}

#[derive(Debug, Error)]
pub enum NodeCfgErr {
    #[error("failed to read HJMT config home: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to read HJMT config bytes: {0}")]
    File(#[from] z00z_utils::io::IoError),
    #[error("failed to decode YAML at {path}: {detail}")]
    Decode { path: PathBuf, detail: String },
    #[error("invalid HJMT config home at {path}: {detail}")]
    Invalid { path: PathBuf, detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct AggFile {
    aggregator_id: u16,
    role: String,
    routing_generation: u64,
    #[serde(default)]
    execution: AggExecutionCfg,
    shards: Vec<ShardFile>,
    network: NetCfg,
    paths: AggPaths,
    lifecycle: LifeCfg,
    route: RouteRef,
    startup: StartupCheckCfg,
    evidence: EvidenceCfg,
    limits: AggLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShardFile {
    shard_id: u16,
    secondary_ids: Vec<u16>,
    expected_journal_lineage: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanFile {
    mode: PlannerMode,
    routing_generation: u64,
    route: RouteRef,
    policy: PlanPolicy,
    limits: PlanLimits,
    paths: PlanPaths,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreFile {
    backend: String,
    generation: u64,
    paths: StorePaths,
    settings: StoreSet,
}

#[derive(Debug)]
struct RouteLive {
    path: PathBuf,
    table: ShardRouteTable,
    digest_hex: String,
}

impl NodeConfig {
    // `config/hjmt_runtime` is a runtime-orchestration fixture home for local
    // topology and preflight evidence. It must not become storage semantic
    // authority.
    pub fn from_hjmt_home(root: impl AsRef<Path>) -> Result<Self, NodeCfgErr> {
        let root = root.as_ref();
        let planner = load_plan_cfg(root)?;
        let storage = load_store_cfg(root)?;
        let aggs = load_aggs(root)?;
        let hjmt = HjmtCfg {
            home: root.to_path_buf(),
            profile: profile_name(root),
            proc_model: ProcModel::OsProcess,
            planner,
            storage,
            aggs,
        };
        validate_hjmt(&hjmt)?;
        Ok(Self {
            mode: NodeMode::Aggregator,
            da_provider: "not_configured".to_string(),
            rpc_enabled: false,
            hjmt: Some(hjmt),
        })
    }

    pub fn config_digests(&self) -> Result<Vec<ConfigDigestRecord>, NodeCfgErr> {
        let hjmt = self.hjmt_cfg()?;
        let route_live = load_live_route(&hjmt.home, &hjmt.planner.route)?;
        let mut records = Vec::with_capacity(hjmt.aggs.len() + 3);
        records.push(file_digest_record(
            "planner-config",
            hjmt.planner.cfg_path.clone(),
        )?);
        records.push(file_digest_record(
            "storage-config",
            hjmt.storage.cfg_path.clone(),
        )?);
        records.push(file_digest_record(ROUTE_LABEL, route_live.path)?);
        let manifest_path = hjmt.home.join(MANIFEST_FILE);
        if io::path_exists(&manifest_path)? {
            records.push(file_digest_record("runtime-manifest", manifest_path)?);
        }
        for agg in sorted_aggs(&hjmt.aggs) {
            records.push(file_digest_record(
                &format!("aggregator-config-{}", agg.aggregator_id.as_u16()),
                agg.cfg_path.clone(),
            )?);
        }
        Ok(records)
    }

    pub fn startup_preflight(
        &self,
        aggregator_id: AggregatorId,
        input: StartupPreflightInput<'_>,
    ) -> Result<StartupPreflightReport, NodeCfgErr> {
        let hjmt = self.hjmt_cfg()?;
        let agg = hjmt
            .proc(aggregator_id)
            .ok_or_else(|| NodeCfgErr::Invalid {
                path: hjmt.home.clone(),
                detail: format!("unknown aggregator id {}", aggregator_id.as_u16()),
            })?;
        let route_live = load_live_route(&hjmt.home, &hjmt.planner.route)?;
        check_route_contract(hjmt, &route_live.table)?;

        let mut checks = Vec::with_capacity(7);
        checks.push(PreflightCheck {
            name: "route_codec".to_string(),
            detail: format!("{} => {}", route_live.path.display(), route_live.digest_hex),
        });

        check_preflight_placement(hjmt, aggregator_id, &route_live.table)?;
        checks.push(PreflightCheck {
            name: "placement".to_string(),
            detail: format!(
                "aggregator {} placement matches routing_generation {}",
                aggregator_id.as_u16(),
                route_live.table.routing_generation
            ),
        });

        check_preflight_lineage(agg, &route_live.table, input.recovery)?;
        checks.push(PreflightCheck {
            name: "journal_lineage".to_string(),
            detail: format!(
                "aggregator {} expected lineage matched recovery state",
                aggregator_id.as_u16()
            ),
        });

        check_live_startup_contract(
            hjmt.storage.backend.as_str(),
            hjmt.storage.generation,
            input.recovery.root_generation,
            input.recovery.proof_version,
        )
        .map_err(|err| NodeCfgErr::Invalid {
            path: hjmt.storage.cfg_path.clone(),
            detail: err.to_string(),
        })?;
        checks.push(PreflightCheck {
            name: "backend_generation".to_string(),
            detail: format!(
                "backend {} generation {} is supported",
                hjmt.storage.backend, hjmt.storage.generation
            ),
        });

        check_preflight_proof(hjmt, input.proof_bytes)?;
        checks.push(PreflightCheck {
            name: "proof_codec".to_string(),
            detail: format!(
                "proof bytes re-encode and verify under envelope v{HJMT_PROOF_ENVELOPE_VERSION}"
            ),
        });

        check_preflight_handoff(hjmt, input.handoff, &route_live.table)?;
        checks.push(PreflightCheck {
            name: "handoff_ready".to_string(),
            detail: format!(
                "{} shard handoff rows are ordered and digest-bound",
                input.handoff.len()
            ),
        });

        check_crypto_tags(hjmt, input.recovery)?;
        checks.push(PreflightCheck {
            name: "crypto_tags".to_string(),
            detail: format!(
                "root generation {} and proof version {} match compiled expectations",
                input.recovery.root_generation, input.recovery.proof_version
            ),
        });

        let mut config_digests = self.config_digests()?;
        if let Some(path) = input.scenario_cfg_path {
            config_digests.push(file_digest_record(SCENARIO_LABEL, path.to_path_buf())?);
        }

        Ok(StartupPreflightReport {
            aggregator_id,
            route_table_path: route_live.path,
            route_table_digest: route_live.digest_hex,
            config_digests,
            checks,
            evidence: agg.evidence.clone(),
        })
    }

    #[must_use]
    pub fn placement_table(&self) -> Option<ShardPlacementTable> {
        self.hjmt.as_ref().map(HjmtCfg::placement_table)
    }

    #[must_use]
    pub fn node_stat(&self) -> Option<NodeStat> {
        self.hjmt.as_ref().map(HjmtCfg::node_stat)
    }

    fn hjmt_cfg(&self) -> Result<&HjmtCfg, NodeCfgErr> {
        self.hjmt.as_ref().ok_or_else(|| NodeCfgErr::Invalid {
            path: PathBuf::from("<node-config>"),
            detail: "hjmt config is not loaded".to_string(),
        })
    }

    pub fn from_agg_run_args(args: &AggRunArgs) -> Result<AggLaunch, NodeCfgErr> {
        let home = agg_home_from_cfg(&args.aggregator_cfg)?;
        let config = Self::from_hjmt_home(&home)?;
        let hjmt = config.hjmt_cfg()?;
        let agg = hjmt
            .aggs
            .iter()
            .find(|item| cmd_path_eq(&hjmt.home, &args.aggregator_cfg, &item.cfg_path))
            .ok_or_else(|| NodeCfgErr::Invalid {
                path: args.aggregator_cfg.clone(),
                detail: format!(
                    "aggregator config path is not owned by runtime home {}",
                    hjmt.home.display()
                ),
            })?;
        if !cmd_path_eq(&hjmt.home, &args.planner_cfg, &hjmt.planner.cfg_path) {
            return invalid(
                args.planner_cfg.clone(),
                format!(
                    "planner config path must stay {}",
                    hjmt.planner.cfg_path.display()
                ),
            );
        }
        if !cmd_path_eq(&hjmt.home, &args.storage_cfg, &hjmt.storage.cfg_path) {
            return invalid(
                args.storage_cfg.clone(),
                format!(
                    "storage config path must stay {}",
                    hjmt.storage.cfg_path.display()
                ),
            );
        }
        let aggregator_id = agg.aggregator_id;
        Ok(AggLaunch {
            config,
            aggregator_id,
        })
    }
}

impl HjmtCfg {
    #[must_use]
    pub fn agg_count(&self) -> usize {
        self.aggs.len()
    }

    #[must_use]
    pub fn shard_count(&self) -> usize {
        self.aggs.iter().map(|agg| agg.shards.len()).sum()
    }

    #[must_use]
    pub fn routing_generation(&self) -> u64 {
        self.planner.routing_generation
    }

    #[must_use]
    pub fn shard_mapping(&self) -> ShardMapping {
        self.aggs
            .first()
            .map(|agg| agg.execution.shard_mapping)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn has_dual_primary(&self) -> bool {
        self.primary_counts().values().any(|count| *count >= 2)
    }

    #[must_use]
    pub fn primary_counts(&self) -> BTreeMap<AggregatorId, usize> {
        let mut counts = BTreeMap::new();
        for agg in &self.aggs {
            let entry = counts.entry(agg.aggregator_id).or_insert(0);
            *entry += agg.shards.len();
        }
        counts
    }

    #[must_use]
    pub fn proc(&self, aggregator_id: AggregatorId) -> Option<&AggProc> {
        self.aggs
            .iter()
            .find(|agg| agg.aggregator_id == aggregator_id)
    }

    #[must_use]
    pub fn placement_table(&self) -> ShardPlacementTable {
        let mut table = ShardPlacementTable::default();
        for agg in &self.aggs {
            for shard in &agg.shards {
                let route = BatchRoute {
                    shard_id: shard.shard_id,
                    routing_generation: agg.routing_generation,
                };
                let secondary = shard
                    .secondary_ids
                    .iter()
                    .copied()
                    .map(SecondaryState::ready)
                    .collect();
                table.insert(ShardPlacement::new(
                    route,
                    agg.aggregator_id,
                    secondary,
                    shard.expected_journal_lineage,
                ));
            }
        }
        table
    }

    #[must_use]
    pub fn node_stat(&self) -> NodeStat {
        NodeStat {
            profile: self.profile.clone(),
            proc_model: self.proc_model,
            agg_count: self.agg_count(),
            shard_count: self.shard_count(),
            routing_generation: self.routing_generation(),
        }
    }

    pub fn check_life_cmd(
        &self,
        aggregator_id: AggregatorId,
        cmd: &str,
        kind: &str,
    ) -> Result<(), NodeCfgErr> {
        let agg = self
            .proc(aggregator_id)
            .ok_or_else(|| NodeCfgErr::Invalid {
                path: self.home.clone(),
                detail: format!("unknown aggregator id {}", aggregator_id.as_u16()),
            })?;
        let got = AggRunArgs::parse_life_cmd(cmd).map_err(|detail| NodeCfgErr::Invalid {
            path: agg.cfg_path.clone(),
            detail,
        })?;
        let want_agg = norm_path(&agg.cfg_path);
        let want_plan = norm_path(&self.planner.cfg_path);
        let want_store = norm_path(&self.storage.cfg_path);
        let agg_ok = cmd_path_eq(&self.home, &got.aggregator_cfg, &want_agg);
        let plan_ok = cmd_path_eq(&self.home, &got.planner_cfg, &want_plan);
        let store_ok = cmd_path_eq(&self.home, &got.storage_cfg, &want_store);
        if !agg_ok || !plan_ok || !store_ok {
            return invalid(
                agg.cfg_path.clone(),
                format!(
                    "{kind} command must reference canonical aggregator/planner/storage config paths for aggregator {}: got agg={}, planner={}, storage={}; want agg={}, planner={}, storage={}",
                    aggregator_id.as_u16(),
                    got.aggregator_cfg.display(),
                    got.planner_cfg.display(),
                    got.storage_cfg.display(),
                    want_agg.display(),
                    want_plan.display(),
                    want_store.display(),
                ),
            );
        }
        Ok(())
    }
}

impl AggRunArgs {
    pub fn parse_cli_argv(argv: &[String]) -> Result<Self, String> {
        let mut mode = None;
        let mut aggregator_cfg = None;
        let mut planner_cfg = None;
        let mut storage_cfg = None;
        let mut idx = 0usize;
        while idx < argv.len() {
            let arg = &argv[idx];
            if let Some(value) = arg.strip_prefix("--mode=") {
                take_cli_value(&mut mode, value, "--mode")?;
            } else if let Some(value) = arg.strip_prefix("--aggregator-config=") {
                take_path_arg(&mut aggregator_cfg, value, "--aggregator-config")?;
            } else if let Some(value) = arg.strip_prefix("--planner-config=") {
                take_path_arg(&mut planner_cfg, value, "--planner-config")?;
            } else if let Some(value) = arg.strip_prefix("--storage-config=") {
                take_path_arg(&mut storage_cfg, value, "--storage-config")?;
            } else if arg == "--mode"
                || arg == "--aggregator-config"
                || arg == "--planner-config"
                || arg == "--storage-config"
            {
                let flag = arg.as_str();
                let value = argv
                    .get(idx + 1)
                    .ok_or_else(|| format!("{flag} must include one value"))?;
                match flag {
                    "--mode" => take_cli_value(&mut mode, value, flag)?,
                    "--aggregator-config" => {
                        take_path_arg(&mut aggregator_cfg, value, flag)?;
                    }
                    "--planner-config" => {
                        take_path_arg(&mut planner_cfg, value, flag)?;
                    }
                    "--storage-config" => {
                        take_path_arg(&mut storage_cfg, value, flag)?;
                    }
                    _ => unreachable!("flag filtered above"),
                }
                idx += 1;
            } else {
                return Err(format!("unsupported rollup-node argument `{arg}`"));
            }
            idx += 1;
        }

        let mode = mode.ok_or_else(|| "--mode must be present".to_string())?;
        if mode != "aggregator" {
            return Err(format!(
                "--mode must stay aggregator for the live process contract; got `{mode}`"
            ));
        }

        Ok(Self {
            aggregator_cfg: aggregator_cfg
                .ok_or_else(|| "--aggregator-config must be present".to_string())?,
            planner_cfg: planner_cfg
                .ok_or_else(|| "--planner-config must be present".to_string())?,
            storage_cfg: storage_cfg
                .ok_or_else(|| "--storage-config must be present".to_string())?,
        })
    }

    pub fn parse_life_cmd(cmd: &str) -> Result<Self, String> {
        let argv = shell_words::split(cmd)
            .map_err(|err| format!("lifecycle command parse failed: {err}"))?;
        let sep = argv.iter().position(|arg| arg == "--").ok_or_else(|| {
            "lifecycle command must separate cargo arguments from rollup-node arguments with `--`"
                .to_string()
        })?;
        check_cargo_run_argv(&argv[..sep])?;
        Self::parse_cli_argv(&argv[sep + 1..])
    }
}

pub fn canonical_run_cmd(agg_cfg: &Path, plan_cfg: &Path, store_cfg: &Path) -> String {
    format!(
        "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config {} --planner-config {} --storage-config {}",
        cmd_render_path(agg_cfg).display(),
        cmd_render_path(plan_cfg).display(),
        cmd_render_path(store_cfg).display(),
    )
}

fn load_plan_cfg(root: &Path) -> Result<PlanCfg, NodeCfgErr> {
    let cfg_path = root.join("planner").join(PLAN_CFG_FILE);
    let raw: PlanFile = load_yaml(&cfg_path)?;
    Ok(PlanCfg {
        cfg_path,
        mode: raw.mode,
        routing_generation: raw.routing_generation,
        route: raw.route,
        policy: raw.policy,
        limits: raw.limits,
        paths: raw.paths,
    })
}

fn load_store_cfg(root: &Path) -> Result<StoreCfg, NodeCfgErr> {
    let cfg_path = root.join("storage").join(STORE_CFG_FILE);
    let raw: StoreFile = load_yaml(&cfg_path)?;
    Ok(StoreCfg {
        cfg_path,
        backend: raw.backend,
        generation: raw.generation,
        paths: raw.paths,
        settings: raw.settings,
    })
}

fn load_aggs(root: &Path) -> Result<Vec<AggProc>, NodeCfgErr> {
    let agg_root = root.join("aggregators");
    let mut entries = io::read_dir(&agg_root)?
        .into_iter()
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    entries.sort();

    let mut aggs = Vec::with_capacity(entries.len());
    for path in entries {
        let cfg_path = path.join(AGG_CFG_FILE);
        let raw: AggFile = load_yaml(&cfg_path)?;
        let mut shards = Vec::with_capacity(raw.shards.len());
        for item in raw.shards {
            shards.push(ShardOwn {
                shard_id: ShardId::new(item.shard_id),
                secondary_ids: item
                    .secondary_ids
                    .into_iter()
                    .map(AggregatorId::new)
                    .collect(),
                expected_journal_lineage: decode_hex32(
                    &cfg_path,
                    "expected_journal_lineage",
                    &item.expected_journal_lineage,
                )?,
            });
        }
        aggs.push(AggProc {
            cfg_path,
            aggregator_id: AggregatorId::new(raw.aggregator_id),
            role: raw.role,
            routing_generation: raw.routing_generation,
            execution: raw.execution,
            shards,
            network: raw.network,
            paths: raw.paths,
            lifecycle: raw.lifecycle,
            route: raw.route,
            startup: raw.startup,
            evidence: raw.evidence,
            limits: raw.limits,
        });
    }
    aggs.sort_by_key(|agg| agg.aggregator_id.as_u16());
    Ok(aggs)
}

fn load_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, NodeCfgErr> {
    let bytes = io::read_file(path)?;
    let codec = YamlCodec;
    codec.deserialize(&bytes).map_err(|err| NodeCfgErr::Decode {
        path: path.to_path_buf(),
        detail: err.to_string(),
    })
}

fn validate_hjmt(cfg: &HjmtCfg) -> Result<(), NodeCfgErr> {
    let path = cfg.home.clone();
    if cfg.proc_model != ProcModel::OsProcess {
        return invalid(path, "process model must stay os-process");
    }
    if cfg.aggs.is_empty() {
        return invalid(path, "aggregator set must not be empty");
    }
    if !is_named_file(&cfg.planner.cfg_path, PLAN_CFG_FILE) {
        return invalid(
            path,
            format!("planner config path must end with {PLAN_CFG_FILE}"),
        );
    }
    if !is_named_file(&cfg.storage.cfg_path, STORE_CFG_FILE) {
        return invalid(
            path,
            format!("storage config path must end with {STORE_CFG_FILE}"),
        );
    }
    if cfg.planner.policy.shard_local_only != cfg.planner.policy.reject_cross_shard {
        return invalid(
            path,
            "planner policy must keep shard-local-only and cross-shard reject aligned",
        );
    }
    if cfg.planner.policy.cadence_ms == 0 {
        return invalid(path, "planner cadence_ms must be positive");
    }
    if cfg.planner.limits.max_batch_ops == 0 || cfg.planner.limits.max_batch_bytes == 0 {
        return invalid(path, "planner limits must stay positive");
    }
    if cfg.storage.settings.cache_capacity == 0 {
        return invalid(path, "storage cache_capacity must be positive");
    }
    if cfg.storage.settings.lock_timeout_ms == 0 {
        return invalid(path, "storage lock_timeout_ms must be positive");
    }
    if is_blank_path(&cfg.storage.paths.import_dir) || is_blank_path(&cfg.storage.paths.lock_path) {
        return invalid(path, "storage import_dir and lock_path must not be empty");
    }
    check_live_startup_contract(
        cfg.storage.backend.as_str(),
        cfg.storage.generation,
        RootGeneration::SettlementV1.version(),
        HJMT_PROOF_ENVELOPE_VERSION as u16,
    )
    .map_err(|err| NodeCfgErr::Invalid {
        path: cfg.storage.cfg_path.clone(),
        detail: err.to_string(),
    })?;

    let mut agg_ids = BTreeSet::new();
    let mut cfg_paths = BTreeSet::new();
    let mut data_dirs = BTreeSet::new();
    let mut journal_paths = BTreeSet::new();
    let mut log_paths = BTreeSet::new();
    let mut listen_addrs = BTreeSet::new();
    let mut digest_files = BTreeSet::new();
    let mut report_files = BTreeSet::new();
    let mut shard_ids = BTreeSet::new();
    let mut generations = BTreeSet::new();
    let mut shard_mappings = BTreeSet::new();

    for agg in &cfg.aggs {
        if !agg_ids.insert(agg.aggregator_id) {
            return invalid(
                path.clone(),
                format!("duplicate aggregator id {}", agg.aggregator_id.as_u16()),
            );
        }
        if !cfg_paths.insert(agg.cfg_path.clone()) {
            return invalid(
                path.clone(),
                format!(
                    "duplicate aggregator config path {}",
                    agg.cfg_path.display()
                ),
            );
        }
        if !data_dirs.insert(agg.paths.data_dir.clone()) {
            return invalid(
                path.clone(),
                format!("duplicate data dir {}", agg.paths.data_dir.display()),
            );
        }
        if !journal_paths.insert(agg.paths.journal_path.clone()) {
            return invalid(
                path.clone(),
                format!(
                    "duplicate journal path {}",
                    agg.paths.journal_path.display()
                ),
            );
        }
        if !log_paths.insert(agg.paths.log_path.clone()) {
            return invalid(
                path.clone(),
                format!("duplicate log path {}", agg.paths.log_path.display()),
            );
        }
        if !listen_addrs.insert(agg.network.listen_addr.clone()) {
            return invalid(
                path.clone(),
                format!("duplicate listen addr {}", agg.network.listen_addr),
            );
        }
        if !digest_files.insert(agg.evidence.config_digest_file.clone()) {
            return invalid(
                path.clone(),
                format!(
                    "duplicate config digest evidence path {}",
                    agg.evidence.config_digest_file.display()
                ),
            );
        }
        if !report_files.insert(agg.evidence.preflight_report_file.clone()) {
            return invalid(
                path.clone(),
                format!(
                    "duplicate preflight report path {}",
                    agg.evidence.preflight_report_file.display()
                ),
            );
        }
        if agg.role.trim() != "aggregator" {
            return invalid(
                path.clone(),
                format!(
                    "aggregator {} role must stay aggregator",
                    agg.aggregator_id.as_u16()
                ),
            );
        }
        if !agg.startup.all_enabled() {
            return invalid(
                path.clone(),
                format!(
                    "aggregator {} must keep every startup preflight check enabled",
                    agg.aggregator_id.as_u16()
                ),
            );
        }
        if agg.lifecycle.start_cmd.trim().is_empty() || agg.lifecycle.restart_cmd.trim().is_empty()
        {
            return invalid(path.clone(), "lifecycle commands must not be empty");
        }
        cfg.check_life_cmd(agg.aggregator_id, &agg.lifecycle.start_cmd, "start")?;
        cfg.check_life_cmd(agg.aggregator_id, &agg.lifecycle.restart_cmd, "restart")?;
        if agg.limits.max_batch_ops == 0 || agg.limits.max_inflight == 0 {
            return invalid(
                path.clone(),
                format!(
                    "aggregator {} limits must stay positive",
                    agg.aggregator_id.as_u16()
                ),
            );
        }
        shard_mappings.insert(agg.execution.shard_mapping);
        match agg.execution.shard_mapping {
            ShardMapping::AggregatorOwned => {
                if let Some(first_lineage) = agg
                    .shards
                    .first()
                    .map(|shard| shard.expected_journal_lineage)
                {
                    for shard in &agg.shards {
                        if shard.expected_journal_lineage != first_lineage {
                            return invalid(
                                agg.cfg_path.clone(),
                                format!(
                                    "aggregator {} shards must share one expected_journal_lineage because journal_path is process-scoped under aggregator_owned",
                                    agg.aggregator_id.as_u16()
                                ),
                            );
                        }
                    }
                }
            }
            ShardMapping::ShardProcess => {
                if agg.shards.len() > 1 {
                    return invalid(
                        agg.cfg_path.clone(),
                        format!(
                            "aggregator {} shard_process mapping allows at most one primary shard per process",
                            agg.aggregator_id.as_u16()
                        ),
                    );
                }
            }
        }
        generations.insert(agg.routing_generation);
        for shard in &agg.shards {
            if !shard_ids.insert(shard.shard_id) {
                return invalid(
                    path.clone(),
                    format!(
                        "duplicate primary owner for shard {}",
                        shard.shard_id.as_u16()
                    ),
                );
            }
            if shard.secondary_ids.is_empty() {
                return invalid(
                    path.clone(),
                    format!(
                        "shard {} must have at least one secondary",
                        shard.shard_id.as_u16()
                    ),
                );
            }
            let mut secondary_ids = BTreeSet::new();
            for secondary in &shard.secondary_ids {
                if !secondary_ids.insert(*secondary) {
                    return invalid(
                        path.clone(),
                        format!(
                            "shard {} secondary set must stay unique",
                            shard.shard_id.as_u16()
                        ),
                    );
                }
            }
            if shard.secondary_ids.contains(&agg.aggregator_id) {
                return invalid(
                    path.clone(),
                    format!(
                        "shard {} secondary set must not include primary {}",
                        shard.shard_id.as_u16(),
                        agg.aggregator_id.as_u16()
                    ),
                );
            }
        }
    }

    if shard_ids.is_empty() {
        return invalid(path.clone(), "topology must own at least one shard");
    }
    if generations.len() != 1 {
        return invalid(
            path.clone(),
            "all aggregators must share one routing_generation",
        );
    }
    if shard_mappings.len() != 1 {
        return invalid(
            path.clone(),
            "all aggregators in one HJMT home must share one execution.shard_mapping",
        );
    }
    if cfg.planner.routing_generation != *generations.iter().next().expect("routing_generation") {
        return invalid(
            path.clone(),
            "planner routing_generation must match aggregator routing_generation",
        );
    }
    for agg in &cfg.aggs {
        for shard in &agg.shards {
            for secondary in &shard.secondary_ids {
                if !agg_ids.contains(secondary) {
                    return invalid(
                        path.clone(),
                        format!(
                            "shard {} secondary {} is not a declared aggregator",
                            shard.shard_id.as_u16(),
                            secondary.as_u16()
                        ),
                    );
                }
            }
        }
    }

    let live_route = load_live_route(&cfg.home, &cfg.planner.route)?;
    check_route_contract(cfg, &live_route.table)?;

    if cfg.profile == "SIM-5A7S" {
        let expect_aggs = (0..5).map(AggregatorId::new).collect::<BTreeSet<_>>();
        if agg_ids != expect_aggs {
            return invalid(
                path.clone(),
                "SIM-5A7S must declare AggregatorId(0)..AggregatorId(4)",
            );
        }
        let expect_shards = (0..7).map(ShardId::new).collect::<BTreeSet<_>>();
        if shard_ids != expect_shards {
            return invalid(path.clone(), "SIM-5A7S must declare ShardId(0)..ShardId(6)");
        }
        if cfg.agg_count() != 5 {
            return invalid(path.clone(), "SIM-5A7S must declare five aggregators");
        }
        if cfg.shard_count() != 7 {
            return invalid(path.clone(), "SIM-5A7S must declare seven shards");
        }
        if !cfg.has_dual_primary() {
            return invalid(
                path,
                "SIM-5A7S must include at least one dual-primary owner",
            );
        }
    } else if cfg.profile == "SIM-7A7S" {
        let expect_aggs = (0..7).map(AggregatorId::new).collect::<BTreeSet<_>>();
        if agg_ids != expect_aggs {
            return invalid(
                path.clone(),
                "SIM-7A7S must declare AggregatorId(0)..AggregatorId(6)",
            );
        }
        let expect_shards = (0..7).map(ShardId::new).collect::<BTreeSet<_>>();
        if shard_ids != expect_shards {
            return invalid(path.clone(), "SIM-7A7S must declare ShardId(0)..ShardId(6)");
        }
        if cfg.agg_count() != 7 {
            return invalid(path.clone(), "SIM-7A7S must declare seven aggregators");
        }
        if cfg.shard_count() != 7 {
            return invalid(path.clone(), "SIM-7A7S must declare seven shards");
        }
        for agg in &cfg.aggs {
            if agg.shards.len() != 1 {
                return invalid(
                    agg.cfg_path.clone(),
                    "SIM-7A7S must assign exactly one primary shard per aggregator",
                );
            }
            let shard = &agg.shards[0];
            let expected_secondaries = expect_aggs
                .iter()
                .copied()
                .filter(|candidate| *candidate != agg.aggregator_id)
                .collect::<BTreeSet<_>>();
            let actual_secondaries = shard.secondary_ids.iter().copied().collect::<BTreeSet<_>>();
            if actual_secondaries != expected_secondaries {
                return invalid(
                    agg.cfg_path.clone(),
                    "SIM-7A7S secondaries must cover every non-owner aggregator",
                );
            }
        }
    }

    Ok(())
}

fn check_route_contract(cfg: &HjmtCfg, table: &ShardRouteTable) -> Result<(), NodeCfgErr> {
    let route = route_ref_parts(&cfg.home, &cfg.planner.route)?;
    for agg in &cfg.aggs {
        if agg.route != cfg.planner.route {
            return invalid(
                agg.cfg_path.clone(),
                "aggregator route reference must exactly match planner route reference",
            );
        }
        if agg.routing_generation != table.routing_generation {
            return invalid(
                agg.cfg_path.clone(),
                format!(
                    "aggregator {} routing_generation must match route table generation",
                    agg.aggregator_id.as_u16()
                ),
            );
        }
    }
    if cfg.planner.routing_generation != table.routing_generation {
        return invalid(
            cfg.planner.cfg_path.clone(),
            "planner routing_generation must match route table generation",
        );
    }
    let shard_ids = cfg
        .aggs
        .iter()
        .flat_map(|agg| agg.shards.iter().map(|shard| shard.shard_id))
        .collect::<BTreeSet<_>>();
    let route_shards = table.shard_set.iter().copied().collect::<BTreeSet<_>>();
    if route_shards != shard_ids {
        return invalid(
            route.0,
            "route table shard_set must exactly match declared topology shard ids",
        );
    }
    Ok(())
}

fn check_preflight_placement(
    cfg: &HjmtCfg,
    aggregator_id: AggregatorId,
    table: &ShardRouteTable,
) -> Result<(), NodeCfgErr> {
    let placement = cfg.placement_table();
    let agg_ids = cfg
        .aggs
        .iter()
        .map(|agg| agg.aggregator_id)
        .collect::<BTreeSet<_>>();
    let route_shards = table.shard_set.iter().copied().collect::<BTreeSet<_>>();
    let placement_shards = cfg
        .aggs
        .iter()
        .flat_map(|agg| agg.shards.iter().map(|shard| shard.shard_id))
        .collect::<BTreeSet<_>>();
    if placement_shards != route_shards {
        return invalid(
            cfg.home.clone(),
            "placement table must exactly cover the route table shard set",
        );
    }
    for shard_id in route_shards {
        let route = BatchRoute {
            shard_id,
            routing_generation: table.routing_generation,
        };
        let row = placement
            .placement(route)
            .ok_or_else(|| NodeCfgErr::Invalid {
                path: cfg.home.clone(),
                detail: format!(
                    "placement row missing for shard {} generation {}",
                    shard_id.as_u16(),
                    table.routing_generation
                ),
            })?;
        if !agg_ids.contains(&row.primary_id) {
            return invalid(
                cfg.home.clone(),
                format!(
                    "shard {} primary references unknown aggregator",
                    shard_id.as_u16()
                ),
            );
        }
        for secondary in &row.secondaries {
            if !agg_ids.contains(&secondary.aggregator_id) {
                return invalid(
                    cfg.home.clone(),
                    format!(
                        "shard {} secondary references unknown aggregator",
                        shard_id.as_u16()
                    ),
                );
            }
            if secondary.aggregator_id == row.primary_id {
                return invalid(
                    cfg.home.clone(),
                    format!(
                        "shard {} secondary must not equal the primary owner",
                        shard_id.as_u16()
                    ),
                );
            }
        }
    }
    cfg.proc(aggregator_id).ok_or_else(|| NodeCfgErr::Invalid {
        path: cfg.home.clone(),
        detail: format!(
            "aggregator {} is not declared in placement",
            aggregator_id.as_u16()
        ),
    })?;
    Ok(())
}

fn check_preflight_lineage(
    agg: &AggProc,
    table: &ShardRouteTable,
    recovery: &SettlementRecoveryState,
) -> Result<(), NodeCfgErr> {
    for shard in &agg.shards {
        if shard.expected_journal_lineage != recovery.journal_lineage {
            return invalid(
                agg.cfg_path.clone(),
                format!(
                    "shard {} expected journal lineage does not match recovery state",
                    shard.shard_id.as_u16()
                ),
            );
        }
    }

    if recovery.version == 0 && recovery.route.is_none() {
        return Ok(());
    }

    let route = recovery.route.ok_or_else(|| NodeCfgErr::Invalid {
        path: agg.cfg_path.clone(),
        detail: "recovery state must export route identity once durable version is non-zero"
            .to_string(),
    })?;

    if route.routing_generation() != table.routing_generation {
        return invalid(
            agg.cfg_path.clone(),
            "recovery state routing_generation does not match the live route table",
        );
    }

    if route.route_table_digest() != table.digest().into_bytes() {
        return invalid(
            agg.cfg_path.clone(),
            "recovery state route table digest does not match the live route table",
        );
    }

    if !agg
        .shards
        .iter()
        .any(|shard| shard.shard_id.as_u32() == route.shard_id())
    {
        return invalid(
            agg.cfg_path.clone(),
            "recovery state shard is not owned by the current aggregator",
        );
    }

    Ok(())
}

fn check_preflight_proof(cfg: &HjmtCfg, proof_bytes: &[u8]) -> Result<(), NodeCfgErr> {
    let batch = BatchProofBlobV1::decode(proof_bytes).map_err(|err| NodeCfgErr::Invalid {
        path: cfg.home.clone(),
        detail: format!("batch proof decode failed: {err}"),
    })?;
    let reencoded = batch.encode().map_err(|err| NodeCfgErr::Invalid {
        path: cfg.home.clone(),
        detail: format!("batch proof encode failed: {err}"),
    })?;
    if reencoded != proof_bytes {
        return invalid(
            cfg.home.clone(),
            "batch proof canonical re-encode changed bytes",
        );
    }
    check_batch_contract_v1(&batch).map_err(|err| NodeCfgErr::Invalid {
        path: cfg.home.clone(),
        detail: format!("batch proof contract failed: {err}"),
    })?;
    Ok(())
}

fn check_preflight_handoff(
    cfg: &HjmtCfg,
    handoff: &[PublicationHandoffMeta],
    table: &ShardRouteTable,
) -> Result<(), NodeCfgErr> {
    let route = publication_route(table);
    let rows = handoff
        .iter()
        .map(|item| {
            PublicationHandoffRowV1::new(
                item.shard_id.as_u32(),
                item.routing_generation,
                item.route_table_digest,
                item.checkpoint_id.into_bytes(),
            )
        })
        .collect::<Vec<_>>();
    check_handoff_route_v1(&rows, &route).map_err(|err| NodeCfgErr::Invalid {
        path: cfg.home.clone(),
        detail: handoff_err(err),
    })
}

fn publication_route(table: &ShardRouteTable) -> PublicationRouteSnapshotV1 {
    PublicationRouteSnapshotV1::new(
        table.routing_generation,
        table.digest().into_bytes(),
        table.activation_checkpoint,
        table
            .shard_set
            .iter()
            .map(|shard_id| shard_id.as_u32())
            .collect(),
    )
}

fn handoff_err(err: ProofChkErr) -> String {
    match err {
        ProofChkErr::PublicationOrderMix => {
            "publication handoff shard rows must stay in strict ascending order".to_string()
        }
        ProofChkErr::PublicationDupShard => {
            "publication handoff shard rows must stay unique".to_string()
        }
        ProofChkErr::PublicationCheckpointMix => {
            "publication handoff checkpoint ids must stay unique".to_string()
        }
        ProofChkErr::PublicationCountMix => {
            "publication handoff metadata must cover every active route-table shard exactly once"
                .to_string()
        }
        ProofChkErr::PublicationRouteMix => {
            "publication handoff rows must match the committed route snapshot".to_string()
        }
        other => format!("publication handoff contract failed: {other}"),
    }
}

fn check_crypto_tags(cfg: &HjmtCfg, recovery: &SettlementRecoveryState) -> Result<(), NodeCfgErr> {
    if RootGeneration::from_version(recovery.root_generation) != Some(RootGeneration::SettlementV1)
    {
        return invalid(
            cfg.home.clone(),
            format!(
                "unsupported recovery root_generation {}",
                recovery.root_generation
            ),
        );
    }
    if recovery.state_root.generation() != RootGeneration::SettlementV1 {
        return invalid(
            cfg.home.clone(),
            "recovery state_root generation does not match the compiled SettlementV1 tag",
        );
    }
    if recovery.state_root.generation().version() != recovery.root_generation {
        return invalid(
            cfg.home.clone(),
            "recovery state_root generation does not match recovery root_generation metadata",
        );
    }
    if recovery.proof_version != HJMT_PROOF_ENVELOPE_VERSION as u16 {
        return invalid(
            cfg.home.clone(),
            format!(
                "unsupported recovery proof_version {}",
                recovery.proof_version
            ),
        );
    }
    Ok(())
}

fn load_live_route(root: &Path, route: &RouteRef) -> Result<RouteLive, NodeCfgErr> {
    let (path, expected_digest) = route_ref_parts(root, route)?;
    let bytes = io::read_file(&path)?;
    let text = String::from_utf8(bytes).map_err(|err| NodeCfgErr::Invalid {
        path: path.clone(),
        detail: format!("route table source must be UTF-8 lowercase hex: {err}"),
    })?;
    let text = text.trim();
    if text.is_empty() {
        return invalid(path, "route table source must not be empty");
    }
    let canon = hex::decode(text).map_err(|err| NodeCfgErr::Invalid {
        path: path.clone(),
        detail: format!("route table source must stay lowercase hex: {err}"),
    })?;
    if hex::encode(&canon) != text {
        return invalid(
            path,
            "route table source must stay lowercase hex with canonical byte pairs",
        );
    }
    let table = ShardRouteTable::from_canon(&canon).map_err(|err| NodeCfgErr::Invalid {
        path: path.clone(),
        detail: format!("route table canonical bytes are invalid: {err}"),
    })?;
    if table.canonical_bytes() != canon {
        return invalid(path, "route table canonical re-encode changed bytes");
    }
    let digest_hex = hex::encode(table.digest().as_bytes());
    if digest_hex != expected_digest {
        return invalid(path, "route table digest mismatch");
    }
    Ok(RouteLive {
        path,
        table,
        digest_hex,
    })
}

fn route_ref_parts(root: &Path, route: &RouteRef) -> Result<(PathBuf, String), NodeCfgErr> {
    let table_path = route
        .table_path
        .as_ref()
        .filter(|path| !is_blank_path(path))
        .ok_or_else(|| NodeCfgErr::Invalid {
            path: root.to_path_buf(),
            detail: "route table_path must be configured".to_string(),
        })?;
    let expected_digest = route
        .expected_digest
        .as_ref()
        .map(|digest| digest.trim().to_string())
        .filter(|digest| !digest.is_empty())
        .ok_or_else(|| NodeCfgErr::Invalid {
            path: root.to_path_buf(),
            detail: "route expected_digest must be configured".to_string(),
        })?;
    let digest = decode_hex32(root, "route.expected_digest", &expected_digest)?;
    let resolved_path = resolve_path(root, table_path);
    Ok((resolved_path, hex::encode(digest)))
}

fn decode_hex32(path: &Path, label: &str, raw: &str) -> Result<[u8; 32], NodeCfgErr> {
    let bytes = hex::decode(raw.trim()).map_err(|err| NodeCfgErr::Invalid {
        path: path.to_path_buf(),
        detail: format!("{label} must stay lowercase hex: {err}"),
    })?;
    if bytes.len() != 32 {
        return invalid(
            path.to_path_buf(),
            format!("{label} must decode to exactly 32 bytes"),
        );
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    if hex::encode(out) != raw.trim() {
        return invalid(
            path.to_path_buf(),
            format!("{label} must stay lowercase canonical hex"),
        );
    }
    Ok(out)
}

fn file_digest_record(label: &str, path: PathBuf) -> Result<ConfigDigestRecord, NodeCfgErr> {
    let bytes = io::read_file(&path)?;
    Ok(ConfigDigestRecord {
        label: label.to_string(),
        path,
        digest_hex: sha256_hex(&bytes),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sorted_aggs(aggs: &[AggProc]) -> Vec<&AggProc> {
    let mut items = aggs.iter().collect::<Vec<_>>();
    items.sort_by_key(|agg| agg.aggregator_id.as_u16());
    items
}

fn resolve_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn norm_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn is_named_file(path: &Path, file_name: &str) -> bool {
    path.file_name()
        .and_then(|item| item.to_str())
        .is_some_and(|item| item == file_name)
}

fn is_blank_path(path: &Path) -> bool {
    path.as_os_str().is_empty()
}

fn take_path_arg(slot: &mut Option<PathBuf>, raw: &str, flag: &str) -> Result<(), String> {
    if raw.trim().is_empty() {
        return Err(format!("{flag} must not be blank"));
    }
    if slot.is_some() {
        return Err(format!("{flag} must appear exactly once"));
    }
    *slot = Some(PathBuf::from(raw));
    Ok(())
}

fn take_cli_value(slot: &mut Option<String>, raw: &str, flag: &str) -> Result<(), String> {
    if raw.trim().is_empty() {
        return Err(format!("{flag} must not be blank"));
    }
    if slot.is_some() {
        return Err(format!("{flag} must appear exactly once"));
    }
    *slot = Some(raw.to_string());
    Ok(())
}

fn check_cargo_run_argv(argv: &[String]) -> Result<(), String> {
    let want = ["cargo", "run", "--release", "-p", "z00z_rollup_node"];
    if argv.len() != want.len() || !argv.iter().zip(want.iter()).all(|(got, want)| got == want) {
        return Err(format!(
            "lifecycle command must stay `{}` before `--`; got `{}`",
            want.join(" "),
            argv.join(" "),
        ));
    }
    Ok(())
}

fn agg_home_from_cfg(path: &Path) -> Result<PathBuf, NodeCfgErr> {
    let path = resolve_run_path(path);
    if !is_named_file(&path, AGG_CFG_FILE) {
        return invalid(
            path.clone(),
            format!("aggregator config path must end with {AGG_CFG_FILE}"),
        );
    }
    let agg_dir = path.parent().ok_or_else(|| NodeCfgErr::Invalid {
        path: path.clone(),
        detail: "aggregator config path must have one parent directory".to_string(),
    })?;
    let agg_root = agg_dir.parent().ok_or_else(|| NodeCfgErr::Invalid {
        path: path.clone(),
        detail: "aggregator config path must stay under aggregators/<agg-id>/".to_string(),
    })?;
    let home = agg_root.parent().ok_or_else(|| NodeCfgErr::Invalid {
        path: path.clone(),
        detail: "aggregator config path must stay under one HJMT runtime home".to_string(),
    })?;
    if agg_root.file_name().and_then(|item| item.to_str()) != Some("aggregators") {
        return invalid(
            path,
            "aggregator config path must stay under aggregators/<agg-id>/".to_string(),
        );
    }
    Ok(norm_path(home))
}

fn resolve_run_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return norm_path(path);
    }
    let path = norm_path(path);
    if path.starts_with("config/hjmt_runtime") {
        return norm_path(&workspace_root().join(path));
    }
    path
}

fn cmd_render_path(path: &Path) -> PathBuf {
    let path = norm_path(path);
    repo_root_from_runtime_path(&path)
        .and_then(|root| path.strip_prefix(&root).ok().map(Path::to_path_buf))
        .unwrap_or(path)
}

fn workspace_root() -> PathBuf {
    norm_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn repo_root_from_runtime_path(path: &Path) -> Option<PathBuf> {
    let parts = path.components().collect::<Vec<_>>();
    for idx in 0..parts.len().saturating_sub(1) {
        if parts[idx].as_os_str() == "config" && parts[idx + 1].as_os_str() == "hjmt_runtime" {
            let mut root = PathBuf::new();
            for part in &parts[..idx] {
                root.push(part.as_os_str());
            }
            return Some(root);
        }
    }
    None
}

fn cmd_path_eq(home: &Path, raw: &Path, want: &Path) -> bool {
    let want = norm_path(want);
    cmd_path_bases(home)
        .into_iter()
        .map(|base| norm_path(&resolve_path(&base, raw)))
        .any(|got| got == want)
}

fn cmd_path_bases(home: &Path) -> Vec<PathBuf> {
    let mut bases = vec![home.to_path_buf()];
    if let Some(base) = repo_cfg_base(home) {
        bases.push(base);
    }
    bases
}

fn repo_cfg_base(home: &Path) -> Option<PathBuf> {
    let runtime_dir = home.parent()?;
    let cfg_dir = runtime_dir.parent()?;
    let runtime_name = runtime_dir.file_name()?.to_str()?;
    let cfg_name = cfg_dir.file_name()?.to_str()?;
    if runtime_name == "hjmt_runtime" && cfg_name == "config" {
        cfg_dir.parent().map(Path::to_path_buf)
    } else {
        None
    }
}

fn profile_name(root: &Path) -> String {
    root.file_name()
        .and_then(|item| item.to_str())
        .map(|item| item.replace('_', "-").to_ascii_uppercase())
        .unwrap_or_else(|| "HJMT".to_string())
}

fn invalid<T>(path: impl Into<PathBuf>, detail: impl Into<String>) -> Result<T, NodeCfgErr> {
    Err(NodeCfgErr::Invalid {
        path: path.into(),
        detail: detail.into(),
    })
}
