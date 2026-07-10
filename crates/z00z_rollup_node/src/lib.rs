//! Z00Z Rollup Node
//!
//! Proof verification and rollup-facing limits for Z00Z output handling.

#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod celestia_local;
mod config;
mod da;
mod mode;
mod process_devnet;
mod rpc;
mod runtime;
mod status;

use z00z_crypto::{
    batch_verify_range_proofs, verify_range_proof, CryptoError, Z00ZCommitment, AGGREGATION_FACTOR,
    MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};

pub use celestia_local::{CelestiaLocalAdapter, CelestiaLocalRecord};
pub use config::{
    canonical_run_cmd, AggExecutionCfg, AggLaunch, AggLimits, AggPaths, AggProc, AggRunArgs,
    ConfigDigestRecord, EvidenceCfg, HjmtCfg, LifeCfg, NetCfg, NodeCfgErr, NodeConfig, NodeStat,
    PlanCfg, PlanLimits, PlanPaths, PlanPolicy, PlannerMode, PreflightCheck, ProcModel,
    PublicationHandoffMeta, RouteRef, ShardMapping, ShardOwn, StartupCheckCfg,
    StartupPreflightInput, StartupPreflightReport, StoreCfg, StorePaths, StoreSet,
};
pub use da::{
    preview_publication_contract, preview_publication_contract_parts,
    publication_height_for_request, DaAdapter, DaError, LocalAdapterRecord, LocalDaAdapter,
    LocalResolveState, PreSealPublicationContract, PublicationReadyInput, PublicationReadyRecord,
};
pub use mode::NodeMode;
pub use process_devnet::{
    hjmt_process_event_path, hjmt_process_heartbeat_path, hjmt_process_ready_path,
    hjmt_process_root, hjmt_process_stale_marker_path, hjmt_process_state_path,
    hjmt_process_stop_path, maybe_run_hjmt_process_devnet, HjmtProcessPersistedState,
    HjmtProcessReadyEvidence, HJMT_PROCESS_EVENTS_FILE, HJMT_PROCESS_HEARTBEAT_FILE,
    HJMT_PROCESS_HEARTBEAT_MS_ENV, HJMT_PROCESS_HOLD_SECS_ENV, HJMT_PROCESS_MODE_ENV,
    HJMT_PROCESS_MODE_HOLD, HJMT_PROCESS_READY_FILE, HJMT_PROCESS_REJECT_STALE_ENV,
    HJMT_PROCESS_RUN_DIR_ENV, HJMT_PROCESS_RUN_ID_ENV, HJMT_PROCESS_STALE_MARKER_FILE,
    HJMT_PROCESS_STATE_FILE, HJMT_PROCESS_STOP_ALL_FILE, HJMT_PROCESS_STOP_FILE,
};
pub use rpc::RpcState;
pub use runtime::NodeRuntime;
pub use status::{ServiceBinding, ServiceBindings, StatusSnapshot};
pub use z00z_validators::{
    verify_settlement_theorem, SettlementError, SettlementTheorem, SettlementTheoremBundle,
};

/// Maximum zero-out outputs per block.
pub const MAX_ZERO_OUT_BLOCK: usize = 10;
/// Maximum zero-out outputs per transaction.
pub const MAX_ZERO_OUT_TX: usize = 2;

/// Effective zero-out block limit.
pub const MAX_ZERO_OUT_PER_BLOCK: usize = MAX_ZERO_OUT_BLOCK;
/// Effective zero-out transaction limit.
pub const MAX_ZERO_OUT_PER_TX: usize = MAX_ZERO_OUT_TX;

/// Validate zero-out limits for a block and transaction.
pub fn validate_zero_out_limits(zero_tx: usize, zero_block: usize) -> bool {
    zero_tx <= MAX_ZERO_OUT_PER_TX && zero_block <= MAX_ZERO_OUT_PER_BLOCK
}

/// Output proof bundle used by rollup verification helpers.
pub struct OutputProof {
    /// Output commitment.
    pub commitment: Z00ZCommitment,
    /// Range proof bytes.
    pub range_proof: Option<Vec<u8>>,
}

/// Verify a single output proof.
pub fn verify_output_proof(output: &OutputProof) -> Result<(), CryptoError> {
    let proof = output
        .range_proof
        .as_ref()
        .ok_or(CryptoError::ProofVerificationFailed)?;

    verify_range_proof(
        proof,
        &output.commitment,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    )
}

/// Verify a batch of output proofs.
pub fn verify_batch_proofs(outputs: &[OutputProof]) -> Result<(), CryptoError> {
    if outputs.is_empty() {
        return Ok(());
    }

    let mut proofs: Vec<&Vec<u8>> = Vec::with_capacity(outputs.len());
    let mut commitments: Vec<&Z00ZCommitment> = Vec::with_capacity(outputs.len());

    for output in outputs {
        let proof = output
            .range_proof
            .as_ref()
            .ok_or(CryptoError::ProofVerificationFailed)?;
        proofs.push(proof);
        commitments.push(&output.commitment);
    }

    let mins = vec![MIN_VALUE_PROMISE; proofs.len()];
    batch_verify_range_proofs(
        &proofs,
        &commitments,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        &mins,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        validate_zero_out_limits, verify_batch_proofs, verify_output_proof, OutputProof,
        MAX_ZERO_OUT_PER_BLOCK, MAX_ZERO_OUT_PER_TX,
    };
    use z00z_crypto::{create_commitment, create_range_proof, Z00ZScalar};

    fn scalar(seed: u64) -> Z00ZScalar {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&seed.to_le_bytes());
        Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
    }

    fn make_output(value: u64, seed: u64) -> OutputProof {
        let blind = scalar(seed);
        let commitment = create_commitment(value, &blind).expect("commitment");
        let proof = create_range_proof(
            value,
            &blind,
            super::RANGE_PROOF_BITS,
            super::MIN_VALUE_PROMISE,
        )
        .expect("proof");
        OutputProof {
            commitment,
            range_proof: Some(proof),
        }
    }

    #[test]
    fn test_zero_out_limits_ok() {
        assert!(validate_zero_out_limits(
            MAX_ZERO_OUT_PER_TX,
            MAX_ZERO_OUT_PER_BLOCK,
        ));
    }

    #[test]
    fn test_zero_out_reject_tx() {
        assert!(!validate_zero_out_limits(
            MAX_ZERO_OUT_PER_TX + 1,
            MAX_ZERO_OUT_PER_BLOCK,
        ));
    }

    #[test]
    fn test_zero_out_reject_block() {
        assert!(!validate_zero_out_limits(
            MAX_ZERO_OUT_PER_TX,
            MAX_ZERO_OUT_PER_BLOCK + 1,
        ));
    }

    #[test]
    fn test_valid_proof_passes() {
        let output = make_output(1000, 11);
        assert!(verify_output_proof(&output).is_ok());
    }

    #[test]
    fn test_tampered_proof_fails() {
        let mut output = make_output(1000, 12);
        if let Some(ref mut proof) = output.range_proof {
            proof[0] ^= 0xAA;
        }
        assert!(verify_output_proof(&output).is_err());
    }

    #[test]
    fn test_wrong_commitment_fails() {
        let mut output = make_output(1000, 13);
        let wrong = make_output(999, 14);
        output.commitment = wrong.commitment;
        assert!(verify_output_proof(&output).is_err());
    }

    #[test]
    fn test_batch_verify_multiple() {
        let outputs = vec![
            make_output(100, 20),
            make_output(200, 21),
            make_output(300, 22),
        ];
        assert!(verify_batch_proofs(&outputs).is_ok());
    }

    #[test]
    fn test_batch_verify_one_bad() {
        let mut outputs = vec![
            make_output(100, 30),
            make_output(200, 31),
            make_output(300, 32),
        ];
        if let Some(ref mut proof) = outputs[1].range_proof {
            proof[0] ^= 0xFF;
        }
        assert!(verify_batch_proofs(&outputs).is_err());
    }

    #[test]
    fn test_batch_empty_ok() {
        let outputs: Vec<OutputProof> = Vec::new();
        assert!(verify_batch_proofs(&outputs).is_ok());
    }

    #[test]
    fn test_missing_proof_fails() {
        let mut output = make_output(1000, 15);
        output.range_proof = None;
        assert!(verify_output_proof(&output).is_err());
    }
}
