//! Spending pipeline abstractions.
//!
//! This module groups the components involved in building and validating outgoing
//! transactions (selection, fee estimation, assembly, signing, proving, verification).
//! Use `crate::tx` for tx-specific assembly and verification flows,
//! and use `crate::stealth` for public sender-output construction instead of
//! reaching into split implementation modules such as `claim_tx`, `state_update`,
//! `tx_verifier`, or `spend_events`.
//!
//! Breaking change: deprecated public construction paths under `tx::test_output_builder::*`
//! and `tx::tx_output::*` are no longer part of the public caller surface.
//! Public sender construction now lives under the crate-root stealth re-exports
//! and `crate::stealth`.

pub mod asset_selector;
/// Algebra and commitment balance validation helpers.
pub mod balance;
mod claim_auth;
mod claim_errors;
pub mod claim_tx;
mod claim_tx_digest;
mod claim_tx_wire;
mod commit_audit;
pub mod fee_estimator;
/// Stage-4 lifecycle projection helpers.
pub mod lifecycle;
pub mod multi_io;
pub mod pay_ref;
pub mod prover;
pub mod signer;
pub mod spend_events;
mod spend_proof_backend;
mod spend_rules;
mod spend_verification;
mod state_checkpoint;
mod state_errors;
mod state_resolved_input;
mod state_traits;
pub mod state_update;
mod state_witness;
/// Output builder internals for confidential leaves.
mod test_output_builder;
/// Thin helper transport builders and fallback metadata.
pub mod thin_builder;
/// Wallet-local thin snapshot cache and thick fallback selection.
pub mod thin_cache;
/// Thin helper signed-index store and resolution APIs.
pub mod thin_index;
/// Thin helper snapshot verification helpers.
pub mod thin_snapshot;
/// Thin helper transport DTOs.
pub mod thin_types;
pub mod tx_assembler;
mod tx_digest;
pub mod tx_id;
/// Stage-4 output flow helpers.
mod tx_output;
pub mod tx_verifier;
mod tx_verifier_errors;
mod tx_wire;
/// Stage-4 spend witness gate helpers.
pub mod witness_gate;
pub use asset_selector::{
    build_selection_fixture, check_batch, check_statement, derive_output_id, is_low_link,
    link_score, InRef, MultiErr, MultiStmt, OutRef, SelCase, SelFix,
};
pub use asset_selector::{
    AssetSelection, AssetSelector, AssetSelectorError, AssetSelectorImpl, AssetSelectorResult,
    SelectionStrategy,
};
pub use balance::{
    balance_blindings, verify_blind_balance, verify_tx_balance, verify_tx_balance_meta, TxBalErr,
};
pub use fee_estimator::{
    FeeEstimate, FeeEstimator, FeeEstimatorError, FeeEstimatorImpl, FeeEstimatorResult, TxWeight,
};
pub use lifecycle::{
    build_confirm_rows, build_pending_rows, validate_confirm_rows, ConfirmEnt, PendingEnt,
};
pub use multi_io::{
    pick_input_rows, pick_output_serials, split_output_amounts, AssetSelCfg, BobOutCfg, MultiIoErr,
};
pub use tx_output::{
    decode_output_pack, derive_balance_commitment, derive_tx_output_nonce,
    verify_commitment_balance_gate, verify_fee_commitment_opening, verify_fee_opening_eq,
    verify_plaintext_balance_with_fee, verify_self_decrypt, OutputBundle,
};

pub use self::tx_verifier::{
    build_tx_package_digest, verify_full_tx_package, verify_package_public_spend_contract,
    TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire, TxPackage, TxProofWire,
    TxVerifier, TxVerifierError, TxVerifierImpl, TxVerifierResult, TxWire, VerificationResult,
};
pub use self::tx_verifier::{
    TxVerifier as LocalVerifier, TxVerifierError as LocalVerifierError,
    TxVerifierImpl as LocalVerifierImpl, TxVerifierResult as LocalVerifierResult,
};
pub use pay_ref::{
    format_compact as format_payref_compact, format_full as format_payref_full,
    format_short as format_payref_short, generate as generate_payref,
    parse_hex as parse_payref_hex, verify as verify_payref, PAY_REF_DOMAIN, PAY_REF_SIZE,
};
pub use spend_events::{
    apply_batch, replay_events, validate_batch, CreatedRec, EventBatch, EventCom, EventErr,
    EventSpent, EventState,
};
pub use tx_id::{generate_mac_key, Z00ZTxId, TX_ID_DOMAIN};
pub use tx_wire::{SpendAuthWire, SpendInputProofWire, SpendProofWire};
pub use witness_gate::{
    asset_wire_to_leaf, prepare_spend_membership_witnesses, prepare_spend_public_inputs,
    resolve_input_pack, resolve_input_secret, verify_spend_witness_gate,
    verify_spend_witness_gate_membership, wire_decrypt_leaf,
};

pub use commit_audit::{
    audit_asset_class_outcome, audit_asset_class_total, AssetClassAuditEntry, AssetClassAuditErr,
    AssetClassAuditMismatchClass, AssetClassAuditOutcome, AssetClassAuditReport,
    AssetClassAuditStatus, AssetClassAuditTarget,
};
pub use prover::{Prover, ProverError, ProverImpl, ProverResult};
pub use signer::{Signer, SignerError, SignerImpl, SignerResult};
pub use spend_proof_backend::{
    default_spend_proof_backend, CanonicalSpendProofBackend, SpendMembershipWitness,
    SpendProofArtifact, SpendProofBackend, SpendProofBackendError, SpendProofStmt,
    SpendProofWitness,
};
pub use spend_rules::{
    derive_spend_nullifier, has_validator_mandate_lock_profile, spend_order, spend_triplets,
    validator_mandate_lock_matches_asset, validator_mandate_lock_payload_commitment,
    validator_mandate_lock_unlock_ready, verify_spend_rules, SpendIn, SpendRule, SpendRuleErr,
    SpendRuleTriplet, SpendStmt, VALIDATOR_MANDATE_LOCK_PROFILE_ID,
};
pub use spend_verification::{
    build_public_spend_contract, build_spend_assets, build_spend_contract_with_rng,
    build_spend_input_proof, verify_tx_public_spend_contract, SpendBuildErr, SpendInputLeaf,
    SpendInputRef, SpendPlan, SpendProofApi, SpendProofErr, SpendPublicErr, SpendWitness,
};

#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
pub use claim_tx::sign_claim_auth;
pub use claim_tx::{
    build_claim_stmt, build_claim_tx_digest, build_owner_attest_msg, claim_auth_pk,
    compute_claim_scope_hash, derive_output_nonce, sign_owner_attest, ClaimAuthWire,
    ClaimContextWire, ClaimInputWire, ClaimOutputWire, ClaimProofWire, ClaimScopeKey, ClaimTxError,
    ClaimTxPackage, ClaimTxVerifier, ClaimTxVerifierImpl, ClaimTxVerifyReport, ClaimTxWire,
    ClaimVerifyResult, CLAIM_PKG,
};
pub use state_update::{
    apply_batch_checkpoint, build_cp_draft, prepare_tx_sum, Checkpoint, CheckpointPubIn,
    CreatedEnt, InputResolver, MemberIndex, MemberWit, ResolvedInput, SettlementState, SpentEnt,
    SpentIndex, SpentIndexError, StateError, TxPkgSum, TxProofError, TxProofVerifier,
};
pub use thin_builder::{
    build_thick_transport_payload, build_thin_transport_payload, ThinFallbackReason,
    ThinTransportMode, ThinTransportPayload,
};
pub use thin_cache::ThinSnapshotCache;
pub use thin_index::ThinIndexStore;
pub use thin_types::{
    ThinAssetPathRef, ThinIndexEntry, ThinIndexError, ThinSnapshot, ThinSnapshotContext,
    ThinSnapshotPin, ThinWalletTxPackage, THIN_SNAPSHOT_VERSION, THIN_TX_PACKAGE_VERSION,
};
pub use tx_assembler::{
    TxAssembler, TxAssemblerError, TxAssemblerImpl, TxAssemblerResult, TxAssemblyParams,
};
