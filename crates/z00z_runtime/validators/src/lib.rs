#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

#[allow(unexpected_cfgs)]
#[cfg(kani)]
extern crate self as z00z_validators;

#[allow(unexpected_cfgs)]
#[cfg(kani)]
#[path = "../tests/generated_kani_validator_checkpoint_flow.rs"]
mod generated_kani_validator_checkpoint_flow;

mod artifact;
mod checkpoint;
mod claim_verify;
mod engine;
mod nullifier;
mod reconcile;
mod spend;
mod tx_verify;
mod verdict;

pub use artifact::ArtifactDecode;
pub use checkpoint::CheckpointFlow;
pub use claim_verify::ClaimPkgVerify;
pub use engine::{ValidatorBoundary, ValidatorService};
pub use nullifier::ClaimNulls;
pub use reconcile::ReconcileRules;
pub use spend::SpendRules;
pub use tx_verify::TxPkgVerify;
#[allow(unexpected_cfgs)]
#[cfg(kani)]
pub use verdict::{kani_runtime_exec_parts, kani_runtime_placement_parts};
pub use verdict::{
    verify_settlement_theorem, RejectClass, ResolvedBatch, SettlementError, SettlementTheorem,
    SettlementTheoremBundle, Verdict, VerdictKind,
};
pub use z00z_storage::settlement::{
    inspect_object_package, ObjectPolicyRegistryV1, ObjectRejectCode, ObjectValidatorVerdict,
};
