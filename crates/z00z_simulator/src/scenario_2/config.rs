use std::path::{Path, PathBuf};

use celestia_types::{consts::appconsts, nmt::Namespace};
use serde::{Deserialize, Serialize};

use super::runner::Scenario2Err;

pub const DEFAULT_CONFIG_PATH: &str = "crates/z00z_simulator/src/scenario_2/scenario_config.yaml";
const CONFIG_MAX_BYTES: u64 = 64 * 1024;
const MAX_CYCLES: u32 = 10;
const MAX_BLOCKS_PER_CYCLE: u32 = 2_000;
const MAX_TXS_PER_BLOCK: u32 = 1_000;
const MAX_TOTAL_TXS: u64 = 20_000_000;
const MAX_WORKER_THREADS: usize = 64;
const PLONKY3_CADENCE_BLOCKS: u64 = 2_000;
const MAX_PLONKY3_WORK_MANIFEST_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PLONKY3_PROOF_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario2Cfg {
    pub scenario: Scenario2Meta,
    pub load: LoadCfg,
    pub runtime: RuntimeCfg,
    pub storage: StorageCfg,
    pub da: DaCfg,
    pub nova: NovaCfg,
    pub plonky3: Plonky3Cfg,
    pub profiling: ProfileCfg,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario2Meta {
    pub id: u32,
    pub name: String,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoadCfg {
    pub cycles: u32,
    pub blocks_per_cycle: u32,
    pub transactions_per_block: u32,
    pub initial_value_per_lane: u64,
    pub wallet_route: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCfg {
    /// Zero selects `available_parallelism`, still capped by the scenario contract.
    pub worker_threads: usize,
    pub chain_id: u32,
    pub chain_type: String,
    pub chain_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageCfg {
    pub output_root: PathBuf,
    pub checkpoint_reload_every_blocks: u32,
    pub hjmt_reload_every_blocks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaCfg {
    pub namespace: String,
    pub max_blob_payload_bytes: usize,
    pub max_ods_width: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NovaCfg {
    pub artifact_dir: PathBuf,
    pub prover_material_file: String,
    pub verifier_bundle_file: String,
    pub max_prover_material_bytes: u64,
    pub max_verifier_bundle_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plonky3Cfg {
    pub cadence_blocks: u64,
    pub max_inflight_chunk_proofs: usize,
    pub max_work_manifest_bytes: u64,
    pub max_proof_artifact_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileCfg {
    pub save_block_records: bool,
    pub directory_scan_entry_cap: usize,
    pub requirement_headroom_percent: u32,
}

impl Scenario2Cfg {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Scenario2Err> {
        let config: Self = z00z_utils::io::load_yaml_bounded(path, CONFIG_MAX_BYTES)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Scenario2Err> {
        if self.scenario.id != 2 || self.scenario.name.trim().is_empty() {
            return Err(Scenario2Err::Config(
                "scenario must have id=2 and a non-empty name".to_string(),
            ));
        }
        if self.load.cycles == 0
            || self.load.cycles > MAX_CYCLES
            || self.load.blocks_per_cycle == 0
            || self.load.blocks_per_cycle > MAX_BLOCKS_PER_CYCLE
            || self.load.transactions_per_block == 0
            || self.load.transactions_per_block > MAX_TXS_PER_BLOCK
        {
            return Err(Scenario2Err::Config(
                "load exceeds the bounded 10 x 2,000 x 1,000 contract".to_string(),
            ));
        }
        let total = self.total_transactions()?;
        if total > MAX_TOTAL_TXS || self.load.initial_value_per_lane == 0 {
            return Err(Scenario2Err::Config(
                "total transactions or initial lane value is invalid".to_string(),
            ));
        }
        let expected_route = ["A", "B", "C", "B", "A"];
        if self
            .load
            .wallet_route
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != expected_route
        {
            return Err(Scenario2Err::Config(
                "wallet_route must be exactly A -> B -> C -> B -> A".to_string(),
            ));
        }
        if self.runtime.chain_id == 0
            || self.runtime.chain_type.trim().is_empty()
            || self.runtime.chain_name.trim().is_empty()
            || self.runtime.worker_threads > MAX_WORKER_THREADS
        {
            return Err(Scenario2Err::Config(
                "runtime chain metadata or worker_threads is invalid".to_string(),
            ));
        }
        Namespace::new_v0(self.da.namespace.as_bytes()).map_err(|error| {
            Scenario2Err::Config(format!("invalid Celestia namespace: {error}"))
        })?;
        let max_blob_bytes = u64::try_from(self.da.max_blob_payload_bytes)
            .map_err(|_| Scenario2Err::Config("Celestia blob bound overflow".to_string()))?;
        if max_blob_bytes == 0
            || max_blob_bytes > appconsts::v6::MAX_TX_SIZE
            || self.da.max_ods_width == 0
            || !self.da.max_ods_width.is_power_of_two()
            || self.da.max_ods_width > appconsts::v6::SQUARE_SIZE_UPPER_BOUND
        {
            return Err(Scenario2Err::Config(
                "Celestia blob or ODS bounds are invalid".to_string(),
            ));
        }
        if self.storage.output_root.as_os_str().is_empty()
            || self.storage.checkpoint_reload_every_blocks == 0
            || self.storage.hjmt_reload_every_blocks == 0
            || self.nova.artifact_dir.as_os_str().is_empty()
            || self.nova.prover_material_file.trim().is_empty()
            || self.nova.verifier_bundle_file.trim().is_empty()
            || self.nova.max_prover_material_bytes == 0
            || self.nova.max_verifier_bundle_bytes == 0
            || self.plonky3.cadence_blocks != PLONKY3_CADENCE_BLOCKS
            || self.plonky3.cadence_blocks != u64::from(self.load.blocks_per_cycle)
            || self.plonky3.max_inflight_chunk_proofs != 1
            || self.plonky3.max_work_manifest_bytes == 0
            || self.plonky3.max_work_manifest_bytes > MAX_PLONKY3_WORK_MANIFEST_BYTES
            || self.plonky3.max_proof_artifact_bytes == 0
            || self.plonky3.max_proof_artifact_bytes > MAX_PLONKY3_PROOF_ARTIFACT_BYTES
            || self.profiling.directory_scan_entry_cap == 0
            || !(10..=100).contains(&self.profiling.requirement_headroom_percent)
        {
            return Err(Scenario2Err::Config(
                "storage, Nova, Plonky3, or profiling bounds are invalid".to_string(),
            ));
        }
        Ok(())
    }

    pub fn total_blocks(&self) -> Result<u64, Scenario2Err> {
        u64::from(self.load.cycles)
            .checked_mul(u64::from(self.load.blocks_per_cycle))
            .ok_or_else(|| Scenario2Err::Config("block count overflow".to_string()))
    }

    pub fn total_transactions(&self) -> Result<u64, Scenario2Err> {
        self.total_blocks()?
            .checked_mul(u64::from(self.load.transactions_per_block))
            .ok_or_else(|| Scenario2Err::Config("transaction count overflow".to_string()))
    }

    pub fn worker_threads(&self) -> usize {
        let detected = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        if self.runtime.worker_threads == 0 {
            detected.clamp(1, MAX_WORKER_THREADS)
        } else {
            self.runtime.worker_threads.clamp(1, MAX_WORKER_THREADS)
        }
    }
}

impl DaCfg {
    pub fn namespace_value(&self) -> Result<Namespace, Scenario2Err> {
        Namespace::new_v0(self.namespace.as_bytes())
            .map_err(|error| Scenario2Err::Config(format!("invalid Celestia namespace: {error}")))
    }

    pub fn max_eds_bytes(&self) -> Result<u64, Scenario2Err> {
        let ods_width = u64::try_from(self.max_ods_width)
            .map_err(|_| Scenario2Err::Config("Celestia ODS width overflow".to_string()))?;
        let share_size = u64::try_from(appconsts::SHARE_SIZE)
            .map_err(|_| Scenario2Err::Config("Celestia share size overflow".to_string()))?;
        let eds_width = ods_width
            .checked_mul(2)
            .ok_or_else(|| Scenario2Err::Config("Celestia EDS width overflow".to_string()))?;
        eds_width
            .checked_mul(eds_width)
            .and_then(|shares| shares.checked_mul(share_size))
            .ok_or_else(|| Scenario2Err::Config("Celestia EDS byte bound overflow".to_string()))
    }
}
