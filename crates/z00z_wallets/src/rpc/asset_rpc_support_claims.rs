use super::{
    asset_quarantine, check_stealth_ownership, check_transparent_ownership, claim_registry,
    claim_scope_hash, sign_claim_receipt, verify_claim_receipt, Asset, AssetRpcImpl, ClaimReceipt,
    ErrorObjectOwned, GlobalClaimRegistry, ImportRejectReason, KernelSignature, Ordering,
    OwnershipError, PersistWalletId, ReceiverKeys, RpcResult, WalletError, WalletOwnershipCtx,
};

fn transparent_owner_detail(error: OwnershipError) -> &'static str {
    match error {
        OwnershipError::MissingOwnerPub => "missing owner_pub",
        OwnershipError::MissingOwnerSig => "missing owner_signature",
        OwnershipError::MissingOwnerTag => "missing owner_tag",
        OwnershipError::BadOwnerSig => "owner signature invalid",
        OwnershipError::OwnerNotInSet => "owner pubkey not in wallet key set",
        OwnershipError::OwnerNotMatch => "owner tag mismatch",
        OwnershipError::PolicyOff => "ownership policy disabled",
        OwnershipError::NotStealth | OwnershipError::BadStealthSig => "unexpected ownership mode",
    }
}

fn stealth_owner_detail(error: OwnershipError) -> &'static str {
    match error {
        OwnershipError::NotStealth => "invalid payment type",
        OwnershipError::MissingOwnerTag => "missing owner_tag",
        OwnershipError::OwnerNotMatch => "stealth owner tag mismatch",
        OwnershipError::BadStealthSig => "stealth owner signature invalid",
        OwnershipError::MissingOwnerPub
        | OwnershipError::MissingOwnerSig
        | OwnershipError::BadOwnerSig
        | OwnershipError::OwnerNotInSet
        | OwnershipError::PolicyOff => "unexpected ownership mode",
    }
}

impl AssetRpcImpl {
    pub(super) async fn key_set(
        &self,
        wallet_id: &PersistWalletId,
    ) -> Result<Vec<[u8; 32]>, WalletError> {
        let mut out = Vec::new();

        if let Ok(list) = self.service.list_cached_receivers(wallet_id).await {
            for (_, pk) in list {
                out.push(pk);
            }
        }

        let pay = self
            .service
            .derive_public_key_for_path(wallet_id, crate::key::Bip44Path::payment(0)?)
            .await?;
        out.push(pay);

        let chg = self
            .service
            .derive_public_key_for_path(wallet_id, crate::key::Bip44Path::change_path(0)?)
            .await?;
        out.push(chg);

        out.sort_unstable();
        out.dedup();
        Ok(out)
    }

    pub(super) async fn check_owner(
        &self,
        wallet_id: &PersistWalletId,
        asset: &Asset,
    ) -> RpcResult<()> {
        if asset.is_transparent() {
            return self.check_transparent_owner(wallet_id, asset).await;
        }

        self.check_stealth_owner(wallet_id, asset).await
    }

    async fn check_transparent_owner(
        &self,
        wallet_id: &PersistWalletId,
        asset: &Asset,
    ) -> RpcResult<()> {
        asset.verify_owner_signature().map_err(|e| {
            self.import_err(
                ImportRejectReason::OwnerMismatch,
                wallet_id,
                Some(asset.asset_id()),
                format!("owner signature verify failed: {e}"),
            )
        })?;

        let keys = self.key_set(wallet_id).await.map_err(|e| {
            self.import_err(
                ImportRejectReason::OwnerMismatch,
                wallet_id,
                Some(asset.asset_id()),
                format!("key_set failed: {e}"),
            )
        })?;

        let ctx = WalletOwnershipCtx { key_set: keys };
        check_transparent_ownership(asset, &ctx)
            .map_err(|error| self.transparent_owner_err(wallet_id, asset, error))
    }

    async fn check_stealth_owner(
        &self,
        wallet_id: &PersistWalletId,
        asset: &Asset,
    ) -> RpcResult<()> {
        let recv_keys: ReceiverKeys = self.service.receiver_keys(wallet_id).await.map_err(|e| {
            self.import_err(
                ImportRejectReason::StealthInconsistent,
                wallet_id,
                Some(asset.asset_id()),
                format!("receiver_keys failed: {e}"),
            )
        })?;

        check_stealth_ownership(asset, &recv_keys)
            .map_err(|error| self.stealth_owner_err(wallet_id, asset, error))
    }

    fn transparent_owner_err(
        &self,
        wallet_id: &PersistWalletId,
        asset: &Asset,
        error: OwnershipError,
    ) -> ErrorObjectOwned {
        self.import_err(
            ImportRejectReason::OwnerMismatch,
            wallet_id,
            Some(asset.asset_id()),
            transparent_owner_detail(error),
        )
    }

    fn stealth_owner_err(
        &self,
        wallet_id: &PersistWalletId,
        asset: &Asset,
        error: OwnershipError,
    ) -> ErrorObjectOwned {
        self.import_err(
            ImportRejectReason::StealthInconsistent,
            wallet_id,
            Some(asset.asset_id()),
            stealth_owner_detail(error),
        )
    }

    pub(super) async fn sign_claim(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: [u8; 32],
    ) -> RpcResult<(ClaimReceipt, KernelSignature)> {
        let keys = self.service.receiver_keys(wallet_id).await.map_err(|e| {
            self.import_err(
                ImportRejectReason::ClaimConflict,
                wallet_id,
                Some(asset_id),
                format!("signer init failed: {e}"),
            )
        })?;
        let chain_type = self
            .service
            .resolve_persisted_wallet_chain_type(wallet_id)
            .await
            .map_err(|e| {
                self.import_err(
                    ImportRejectReason::ClaimConflict,
                    wallet_id,
                    Some(asset_id),
                    format!("wallet chain resolve failed: {e}"),
                )
            })?;
        let (_, chain_label, _) = Self::chain_meta(chain_type);

        let receipt = ClaimReceipt {
            schema_ver: 1,
            asset_id,
            wallet_id: wallet_id.0.as_bytes().to_vec(),
            claim_scope: claim_scope_hash(chain_label),
            identity_pk: keys.identity_pk.to_bytes(),
        };

        let sig = sign_claim_receipt(&keys, &receipt).map_err(|e| {
            self.import_err(
                ImportRejectReason::ClaimConflict,
                wallet_id,
                Some(asset_id),
                format!("claim sign failed: {e}"),
            )
        })?;

        verify_claim_receipt(&receipt, &sig).map_err(|e| {
            self.import_err(
                ImportRejectReason::ClaimConflict,
                wallet_id,
                Some(asset_id),
                format!("claim verify failed: {e}"),
            )
        })?;

        Ok((receipt, sig))
    }

    pub(super) fn claim_reserve(
        &self,
        wallet_id: &PersistWalletId,
        receipt: &ClaimReceipt,
        claim_sig: &KernelSignature,
    ) -> Result<claim_registry::ClaimReservation, ErrorObjectOwned> {
        let reg = claim_registry::global_claim_registry();
        match reg.reserve(receipt.asset_id, &wallet_id.0, receipt, claim_sig) {
            Ok(res) => Ok(res),
            Err(claim_registry::ClaimReserveErr::Conflict(conf)) => Err(self.import_err(
                ImportRejectReason::ClaimConflict,
                wallet_id,
                Some(receipt.asset_id),
                format!(
                    "asset already claimed by another wallet: {}",
                    conf.claimed_by
                ),
            )),
            Err(claim_registry::ClaimReserveErr::LockPoison) => Err(self.import_err(
                ImportRejectReason::ClaimConflict,
                wallet_id,
                Some(receipt.asset_id),
                "claim lock poisoned",
            )),
            Err(claim_registry::ClaimReserveErr::InvalidReceipt) => Err(self.import_err(
                ImportRejectReason::ClaimConflict,
                wallet_id,
                Some(receipt.asset_id),
                "claim receipt invalid",
            )),
        }
    }

    pub(super) fn claim_finalize(
        &self,
        wallet_id: &PersistWalletId,
        reservation: &claim_registry::ClaimReservation,
    ) -> Result<(), ErrorObjectOwned> {
        if self.finalize_fail_on.load(Ordering::Relaxed) {
            return Err(self.import_err(
                ImportRejectReason::ConservationViolation,
                wallet_id,
                Some(reservation.asset_id),
                "claim finalize injected failure",
            ));
        }

        let reg = claim_registry::global_claim_registry();
        match reg.finalize(reservation) {
            Ok(()) => Ok(()),
            Err(claim_registry::ClaimFinalizeErr::LockPoison) => Err(self.import_err(
                ImportRejectReason::ConservationViolation,
                wallet_id,
                Some(reservation.asset_id),
                "claim lock poisoned",
            )),
            Err(claim_registry::ClaimFinalizeErr::MissingClaim) => Err(self.import_err(
                ImportRejectReason::ConservationViolation,
                wallet_id,
                Some(reservation.asset_id),
                "missing claim reservation",
            )),
            Err(claim_registry::ClaimFinalizeErr::OwnerMiss) => Err(self.import_err(
                ImportRejectReason::ConservationViolation,
                wallet_id,
                Some(reservation.asset_id),
                "claim owner mismatch",
            )),
        }
    }

    pub(super) fn claim_release(&self, reservation: &claim_registry::ClaimReservation) {
        let reg = claim_registry::global_claim_registry();
        reg.release(reservation);
    }

    pub(super) async fn mark_quarantine(&self, wallet_id: &PersistWalletId, asset_id: [u8; 32]) {
        asset_quarantine::mark_quarantine(&self.quarantined, wallet_id, asset_id).await;
    }
}
