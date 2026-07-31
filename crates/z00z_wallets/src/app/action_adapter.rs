//! Fail-closed wallet intent boundary until the final effect owner is wired.

use z00z_app_api::{
    AppError, BoundedId, ReviewProjection, ReviewWalletIntentRequest, SubmitWalletIntentRequest,
    WalletIntentProposal,
};

/// Wallet intent operations are registered but intentionally unavailable.
///
/// The action basis proves vocabulary coverage only. Phase 1 has no live
/// policy, fee, publication, validator, watcher, and wallet-effect composition
/// that can safely authorize or commit an intent.
pub struct ActionAdapter;

impl ActionAdapter {
    pub const fn new() -> Self {
        Self
    }

    pub fn propose(
        &self,
        _proposal: WalletIntentProposal,
    ) -> Result<WalletIntentProposal, AppError> {
        unavailable()
    }

    pub fn review(
        &self,
        _request: &ReviewWalletIntentRequest,
    ) -> Result<ReviewProjection, AppError> {
        unavailable()
    }

    pub fn submit(
        &self,
        _request: &SubmitWalletIntentRequest,
    ) -> Result<z00z_app_api::OperationProjection, AppError> {
        unavailable()
    }
}

fn unavailable<T>() -> Result<T, AppError> {
    Err(AppError::CapabilityUnavailable {
        code: BoundedId::new("wallet-intent-effect")?,
    })
}
