use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use z00z_core::AssetWire;
use z00z_storage::settlement::SettlementPath;

#[derive(Clone, Debug)]
pub(super) struct OwnedCoin {
    pub lane: u32,
    pub wire: AssetWire,
    pub path: SettlementPath,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scenario2Summary {
    pub run_dir: PathBuf,
    pub blocks: u64,
    pub transactions: u64,
    pub final_checkpoint_id_hex: String,
    pub final_recursive_digest_hex: String,
    pub final_settlement_root_hex: String,
    pub plonky3_cadence_blocks: u64,
    pub completed_plonky3_epochs: u32,
    pub final_plonky3_epoch_statement_digest_hex: String,
    pub final_plonky3_history_proof_digest_hex: String,
    pub final_plonky3_epoch_manifest_digest_hex: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct BlockOutcome {
    pub cycle: u32,
    pub height: u64,
    pub tx_count: u32,
    pub sender: String,
    pub recipient: String,
    pub checkpoint_id_hex: String,
    pub recursive_digest_hex: String,
    pub settlement_root_hex: String,
    pub nova_action: String,
    pub nova_cumulative_steps: u64,
    pub nova_verifier_attempts: Option<u64>,
    pub recovery_snapshot_bytes: Option<u64>,
    pub da_bytes: u64,
    pub plonky3_chunk_ordinal: Option<u32>,
    pub plonky3_chunk_proof_bytes: Option<u64>,
    pub plonky3_trace_rows: Option<u64>,
    pub plonky3_table_count: Option<u64>,
    pub plonky3_merged_parents: Option<usize>,
    pub plonky3_verified_chunks: Option<u32>,
    pub plonky3_active_ranges: Option<u32>,
}
