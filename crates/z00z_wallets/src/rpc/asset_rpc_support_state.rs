use super::{
    asset_quarantine, asset_rpc_rate_limits, asset_rpc_stakes, claim_registry,
    decode_request_compact, tx_rpc_support, wallet_chain_id, Asset, AssetId, AssetRpcImpl,
    ErrorObjectOwned, ImportRejectReason, Logger, Ordering, PaymentRequest, PersistTxId,
    PersistWalletId, ReceiverCard, ReceiverCardRecord, ReceiverKeys, RngCoreExt, RpcResult,
    SendTarget, SessionToken, SystemRngProvider, TracingLogger, ValidatePaymentRequest,
    ValidationOutcome, VerifyResult, WalletError,
};
use crate::chain::{Broadcast, BroadcastImpl, ChainClientImpl, LocalNodeSim};
use crate::tx::{build_tx_package_digest, TxInputWire, TxOutRole, TxOutputWire, TxPackage, TxWire};
use z00z_core::assets::AssetPkgWire;
use z00z_utils::codec::{Codec, JsonCodec};

pub(super) struct LocalMutationExec<'a> {
    rpc: &'a AssetRpcImpl,
    wallet_id: &'a PersistWalletId,
    operation: &'static str,
    inputs: &'a [Asset],
    outputs: &'a [Asset],
}

impl<'a> LocalMutationExec<'a> {
    fn new(
        rpc: &'a AssetRpcImpl,
        wallet_id: &'a PersistWalletId,
        operation: &'static str,
        inputs: &'a [Asset],
        outputs: &'a [Asset],
    ) -> Self {
        Self {
            rpc,
            wallet_id,
            operation,
            inputs,
            outputs,
        }
    }

    fn digest_seed(&self) -> Vec<u8> {
        let mut digest_seed = Vec::with_capacity(self.operation.len() + self.wallet_id.0.len());
        digest_seed.extend_from_slice(self.operation.as_bytes());
        digest_seed.extend_from_slice(self.wallet_id.0.as_bytes());
        for input in self.inputs {
            digest_seed.extend_from_slice(&input.asset_id());
            digest_seed.extend_from_slice(&input.serial_id.to_le_bytes());
            digest_seed.extend_from_slice(&input.amount.to_le_bytes());
        }
        for output in self.outputs {
            digest_seed.extend_from_slice(&output.asset_id());
            digest_seed.extend_from_slice(&output.serial_id.to_le_bytes());
            digest_seed.extend_from_slice(&output.amount.to_le_bytes());
        }
        digest_seed
    }

    fn build_package(&self) -> RpcResult<TxPackage> {
        let chain_id = wallet_chain_id()?;
        let (_, chain_type, chain_name) = AssetRpcImpl::chain_meta_from_id(chain_id);
        let digest = blake3::hash(&self.digest_seed());
        let mut nonce_bytes = [0u8; 8];
        nonce_bytes.copy_from_slice(&digest.as_bytes()[..8]);
        let nonce = u64::from_le_bytes(nonce_bytes);

        let tx = TxWire {
            tx_type: "regular_tx".to_string(),
            inputs: self
                .inputs
                .iter()
                .map(AssetRpcImpl::tx_input_from_asset)
                .collect(),
            outputs: self
                .outputs
                .iter()
                .map(AssetRpcImpl::tx_output_from_asset)
                .collect(),
            fee: 0,
            nonce,
            context: Default::default(),
            proof: Default::default(),
            auth: Default::default(),
        };
        let tx_digest_hex = build_tx_package_digest(
            "TxPackage",
            "regular_tx",
            1,
            chain_id,
            chain_type,
            chain_name,
            &tx,
        )
        .map_err(|error| {
            ErrorObjectOwned::owned(
                -32603,
                format!("local asset mutation tx digest build failed: {error}"),
                None::<()>,
            )
        })?;

        Ok(TxPackage {
            kind: "TxPackage".to_string(),
            package_type: "regular_tx".to_string(),
            version: 1,
            chain_id,
            chain_type: chain_type.to_string(),
            chain_name: chain_name.to_string(),
            tx,
            tx_digest_hex,
            status: "pending".to_string(),
        })
    }

    pub(super) fn submit(&self) -> RpcResult<PersistTxId> {
        let package = self.build_package()?;
        let tx_id = PersistTxId::new(format!("tx_{}", package.tx_digest_hex));
        let tx_bytes = JsonCodec.serialize(&package).map_err(|error| {
            ErrorObjectOwned::owned(
                -32603,
                format!("local asset mutation tx encoding failed: {error}"),
                None::<()>,
            )
        })?;
        let client = ChainClientImpl::with_local_sim(LocalNodeSim::default());
        let store = self.rpc.wallet_tx_store(self.wallet_id);
        let broadcast = BroadcastImpl::new(
            client,
            store,
            tx_rpc_support::TimeProviderRef(self.rpc.time_provider.clone()),
        );
        let result = broadcast.broadcast(&tx_bytes).map_err(|error| {
            ErrorObjectOwned::owned(
                -32603,
                format!("local asset mutation broadcast failed: {error}"),
                None::<()>,
            )
        })?;

        if result.tx_hash != package.tx_digest_hex && result.tx_hash != tx_id.0 {
            return Err(ErrorObjectOwned::owned(
                -32603,
                format!(
                    "local asset mutation hash drift: expected {}, got {}",
                    package.tx_digest_hex, result.tx_hash
                ),
                None::<()>,
            ));
        }

        Ok(tx_id)
    }
}

impl AssetRpcImpl {
    pub(super) async fn quarantine_ids(&self, wallet_id: &PersistWalletId) -> Vec<[u8; 32]> {
        asset_quarantine::quarantine_ids(&self.quarantined, wallet_id).await
    }

    pub fn set_finalize_fail(&self, is_on: bool) {
        self.finalize_fail_on.store(is_on, Ordering::Relaxed);
    }

    pub async fn is_asset_quarantined(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: [u8; 32],
    ) -> bool {
        let map = self.quarantined.read().await;
        map.get(wallet_id)
            .map(|rows| rows.iter().any(|item| item == &asset_id))
            .unwrap_or(false)
    }

    pub fn is_claim_pending(&self, wallet_id: &PersistWalletId, asset_id: [u8; 32]) -> bool {
        claim_registry::is_pending(&wallet_id.0, asset_id)
    }

    pub fn clear_claim_rows(&self) {
        claim_registry::clear_rows();
    }

    pub fn verify_complete_count(&self) -> u64 {
        self.verify_complete_calls.load(Ordering::Relaxed)
    }

    pub fn reset_verify_complete_count(&self) {
        self.verify_complete_calls.store(0, Ordering::Relaxed);
    }

    pub fn has_claim_row(&self, asset_id: [u8; 32]) -> bool {
        claim_registry::has_row(asset_id)
    }

    pub async fn has_stored_asset(&self, asset_id: [u8; 32]) -> bool {
        self.service
            .list_claimed_all()
            .await
            .is_ok_and(|items| items.into_iter().any(|item| item.asset_id() == asset_id))
    }

    pub async fn test_receiver_keys(
        &self,
        wallet_id: &PersistWalletId,
    ) -> Result<ReceiverKeys, WalletError> {
        self.service.receiver_keys(wallet_id).await
    }

    pub(crate) fn now_ms(&self) -> u64 {
        self.time_provider.compat_unix_timestamp_millis()
    }

    pub(super) async fn next_stake_id(&self, asset_id: AssetId, amount: u64) -> String {
        asset_rpc_stakes::next_stake_id(&self.stake_id_counter, self.now_ms(), asset_id, amount)
            .await
    }

    pub(super) fn local_mutation_exec<'a>(
        &'a self,
        wallet_id: &'a PersistWalletId,
        operation: &'static str,
        inputs: &'a [Asset],
        outputs: &'a [Asset],
    ) -> LocalMutationExec<'a> {
        LocalMutationExec::new(self, wallet_id, operation, inputs, outputs)
    }

    pub(super) fn build_local_mutation_output(
        &self,
        wallet_id: &PersistWalletId,
        operation: &'static str,
        definition: std::sync::Arc<z00z_core::assets::AssetDefinition>,
        serial_id: u32,
        amount: u64,
        output_index: usize,
    ) -> RpcResult<Asset> {
        let mut nonce_seed = Vec::with_capacity(
            operation.len() + wallet_id.0.len() + definition.id.len() + 4 + 8 + 8,
        );
        nonce_seed.extend_from_slice(operation.as_bytes());
        nonce_seed.extend_from_slice(wallet_id.0.as_bytes());
        nonce_seed.extend_from_slice(&definition.id);
        nonce_seed.extend_from_slice(&serial_id.to_le_bytes());
        nonce_seed.extend_from_slice(&amount.to_le_bytes());
        nonce_seed.extend_from_slice(&(output_index as u64).to_le_bytes());

        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(blake3::hash(&nonce_seed).as_bytes());

        let serial_bytes = serial_id.to_le_bytes();
        let amount_bytes = amount.to_le_bytes();
        let output_bytes = (output_index as u64).to_le_bytes();
        let blinding = z00z_crypto::try_hash_to_scalar_domain(
            b"z00z.wallet.local_mutation.output_blinding.v1",
            &[
                operation.as_bytes(),
                wallet_id.0.as_bytes(),
                &definition.id,
                &serial_bytes,
                &amount_bytes,
                &output_bytes,
            ],
        )
        .map_err(|error| {
            ErrorObjectOwned::owned(
                -32603,
                format!("local asset mutation output blinding derive failed: {error}"),
                None::<()>,
            )
        })?;

        Asset::new_confidential_with_blinding(definition, serial_id, amount, nonce, &blinding)
            .map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("local asset mutation output build failed: {error}"),
                    None::<()>,
                )
            })
    }

    fn tx_input_from_asset(asset: &Asset) -> TxInputWire {
        TxInputWire {
            asset_id_hex: hex::encode(asset.asset_id()),
            serial_id: asset.serial_id,
        }
    }

    fn tx_output_from_asset(asset: &Asset) -> TxOutputWire {
        TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_asset(asset),
        }
    }

    pub(super) async fn asset_send_precheck(
        &self,
        wallet_id: &PersistWalletId,
    ) -> Result<(), (u32, u32, u32)> {
        asset_rpc_rate_limits::asset_send_precheck(
            &self.asset_send_rate_limits,
            self.now_ms(),
            wallet_id,
        )
        .await
    }

    pub(super) fn parse_pay_data(recipient: &str) -> &str {
        recipient
            .strip_prefix("z00z:pay?data=")
            .unwrap_or(recipient)
    }

    pub(super) fn parse_send_target(
        &self,
        recipient: &str,
    ) -> Result<SendTarget, ErrorObjectOwned> {
        let pay_data = Self::parse_pay_data(recipient);

        if let Ok(request) = decode_request_compact(pay_data) {
            return Ok(SendTarget::Request(request));
        }

        if let Ok(record) = ReceiverCardRecord::from_compact(pay_data, None) {
            let card = record.decode_card().map_err(|_| {
                ErrorObjectOwned::owned(-32602, "SEND_RECIPIENT_INVALID".to_string(), None::<()>)
            })?;
            return Ok(SendTarget::Card(card));
        }

        if let Ok(record) = ReceiverCardRecord::from_compact(recipient, None) {
            let card = record.decode_card().map_err(|_| {
                ErrorObjectOwned::owned(-32602, "SEND_RECIPIENT_INVALID".to_string(), None::<()>)
            })?;
            return Ok(SendTarget::Card(card));
        }

        Err(ErrorObjectOwned::owned(
            -32602,
            "SEND_RECIPIENT_INVALID".to_string(),
            None::<()>,
        ))
    }

    pub(super) fn request_to_card(&self, request: &PaymentRequest) -> ReceiverCard {
        ReceiverCard {
            version: 1,
            owner_handle: request.owner_handle,
            view_pk: request.view_pk,
            identity_pk: request.identity_pk,
            card_id: None,
            metadata: None,
            signature: [0u8; 64],
        }
    }

    pub(super) fn random_seed(&self) -> [u8; 32] {
        let mut seed = [0u8; 32];
        let mut rng = SystemRngProvider.rng();
        rng.fill_bytes_ext(&mut seed);
        seed
    }

    fn send_tofu_rejected(err: impl std::fmt::Display) -> ErrorObjectOwned {
        ErrorObjectOwned::owned(-32602, format!("SEND_TOFU_REJECTED:{err}"), None::<()>)
    }

    fn send_tofu_confirm_required() -> ErrorObjectOwned {
        ErrorObjectOwned::owned(-32602, "SEND_TOFU_CONFIRM_REQUIRED".to_string(), None::<()>)
    }

    fn validate_verify_result(result: VerifyResult) -> Result<(), ErrorObjectOwned> {
        match result {
            VerifyResult::Verified | VerifyResult::NewPin => Ok(()),
            VerifyResult::ViewKeyChanged { .. } | VerifyResult::IdentityKeyChanged => {
                Err(Self::send_tofu_confirm_required())
            }
        }
    }

    fn validate_request_outcome(outcome: ValidationOutcome) -> Result<(), ErrorObjectOwned> {
        match outcome {
            ValidationOutcome::Approved => Ok(()),
            ValidationOutcome::RequiresUserConfirmation | ValidationOutcome::IdentityMismatch => {
                Err(Self::send_tofu_confirm_required())
            }
        }
    }

    async fn verify_send_tofu_card(
        &self,
        wallet_id: &PersistWalletId,
        card: &ReceiverCard,
    ) -> Result<(), ErrorObjectOwned> {
        let result = self
            .service
            .tofu_verify_pin(wallet_id, card, None)
            .await
            .map_err(Self::send_tofu_rejected)?;

        if matches!(result, VerifyResult::Verified | VerifyResult::NewPin) {
            // A successful card-only send is the wallet-local approval action
            // for this receiver card, so promote the stored pin before the
            // validated card builder enforces `TrustLevel::Pinned`.
            self.service
                .tofu_confirm(wallet_id, &card.owner_handle, &card.view_pk)
                .await
                .map_err(Self::send_tofu_rejected)?;
        }

        Self::validate_verify_result(result)
    }

    async fn verify_send_tofu_request(
        &self,
        wallet_id: &PersistWalletId,
        request: &PaymentRequest,
    ) -> Result<(), ErrorObjectOwned> {
        let mut pins = self
            .service
            .load_tofu(wallet_id)
            .await
            .map_err(Self::send_tofu_rejected)?;

        let flow = request
            .validate_all(&mut pins, wallet_chain_id()?)
            .map_err(|_| {
                ErrorObjectOwned::owned(-32602, "SEND_REQUEST_INVALID".to_string(), None::<()>)
            })?;

        self.service
            .save_tofu(wallet_id, &pins)
            .await
            .map_err(Self::send_tofu_rejected)?;

        Self::validate_request_outcome(flow)
    }

    pub(super) async fn verify_send_tofu(
        &self,
        wallet_id: &PersistWalletId,
        target: &SendTarget,
    ) -> Result<(), ErrorObjectOwned> {
        match target {
            SendTarget::Card(card) => self.verify_send_tofu_card(wallet_id, card).await,
            SendTarget::Request(request) => self.verify_send_tofu_request(wallet_id, request).await,
        }
    }

    pub(crate) async fn verify_session(&self, session: &SessionToken) -> RpcResult<()> {
        #[cfg(all(test, feature = "os_hardening"))]
        {
            let _ = session;
            Ok(())
        }

        #[cfg(not(all(test, feature = "os_hardening")))]
        {
            self.service
                .check_auto_lock()
                .await
                .map_err(super::map_wallet_error_to_rpc)?;

            self.service
                .verify_session(session)
                .await
                .map_err(super::map_wallet_error_to_rpc)?;

            Ok(())
        }
    }

    pub(super) fn import_err(
        &self,
        reason: ImportRejectReason,
        wallet_id: &PersistWalletId,
        asset_id: Option<[u8; 32]>,
        detail: impl Into<String>,
    ) -> ErrorObjectOwned {
        let code = reason.code();
        let msg = detail.into();
        let aid = asset_id
            .map(hex::encode)
            .unwrap_or_else(|| "n/a".to_string());
        Logger::warn(
            &TracingLogger,
            &format!(
                "reason={} wallet_id={} asset_id={} action=import_rejected {}",
                code, wallet_id.0, aid, msg
            ),
        );
        ErrorObjectOwned::owned(reason.rpc_code(), code.to_string(), None::<()>)
    }
}
