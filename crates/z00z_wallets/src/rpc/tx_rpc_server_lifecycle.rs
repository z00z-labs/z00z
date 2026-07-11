use rand::SeedableRng;

#[derive(Clone)]
enum TxBuildTarget {
    Card(crate::receiver::ReceiverCard),
    Request(crate::receiver::PaymentRequest),
}

fn parse_tx_build_target(recipient: &str) -> Result<TxBuildTarget, ErrorObjectOwned> {
    let pay_data = recipient
        .strip_prefix("z00z:pay?data=")
        .unwrap_or(recipient);

    if let Ok(request) = crate::receiver::request::decode_request_compact(pay_data) {
        return Ok(TxBuildTarget::Request(request));
    }

    if let Ok(record) = crate::chain::ReceiverCardRecord::from_compact(pay_data, None) {
        let card = record.decode_card().map_err(|_| {
            ErrorObjectOwned::owned(-32602, "SEND_RECIPIENT_INVALID".to_string(), None::<()>)
        })?;
        return Ok(TxBuildTarget::Card(card));
    }

    if let Ok(record) = crate::chain::ReceiverCardRecord::from_compact(recipient, None) {
        let card = record.decode_card().map_err(|_| {
            ErrorObjectOwned::owned(-32602, "SEND_RECIPIENT_INVALID".to_string(), None::<()>)
        })?;
        return Ok(TxBuildTarget::Card(card));
    }

    Err(ErrorObjectOwned::owned(
        -32602,
        "SEND_RECIPIENT_INVALID".to_string(),
        None::<()>,
    ))
}

fn tx_request_to_card(request: &crate::receiver::PaymentRequest) -> crate::receiver::ReceiverCard {
    crate::receiver::ReceiverCard {
        version: 1,
        owner_handle: request.owner_handle,
        view_pk: request.view_pk,
        identity_pk: request.identity_pk,
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    }
}

fn inherited_output_serial(asset: &z00z_core::Asset) -> Result<u32, ErrorObjectOwned> {
    let max_serial = asset.definition.serials;
    if max_serial <= 1 {
        return Err(ErrorObjectOwned::owned(
            -32603,
            "selected asset definition does not expose a tx-output serial".to_string(),
            None::<()>,
        ));
    }

    Ok(asset.serial_id.clamp(1, max_serial.saturating_sub(1)))
}

impl TxRpcImpl {
    fn tx_seed_bytes(&self) -> [u8; 32] {
        let mut seed = [0u8; 32];
        self.service.fill_entropy(&mut seed);
        seed
    }

    fn tx_scalar(&self) -> Result<z00z_crypto::Z00ZScalar, ErrorObjectOwned> {
        const MAX_ATTEMPTS: usize = 16;

        for _ in 0..MAX_ATTEMPTS {
            let mut uniform = [0u8; 64];
            self.service.fill_entropy(&mut uniform);
            if let Ok(scalar) = z00z_crypto::Z00ZScalar::try_from_hash(&uniform) {
                return Ok(scalar);
            }
        }

        Err(ErrorObjectOwned::owned(
            -32603,
            "wallet tx entropy source failed to derive a non-zero scalar".to_string(),
            None::<()>,
        ))
    }

    fn tx_wallet_chain_meta(&self) -> Result<(u32, String, String), ErrorObjectOwned> {
        let chain = crate::services::wallet_runtime_config::resolve_wallet_chain_type_checked().map_err(|e| {
            ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>)
        })?;

        let (chain_id, chain_type) = match chain {
            crate::ChainType::Mainnet => (1, "mainnet".to_string()),
            crate::ChainType::Testnet => (2, "testnet".to_string()),
            crate::ChainType::Devnet => (3, "devnet".to_string()),
        };

        Ok((chain_id, chain_type, "z00z".to_string()))
    }

    async fn verify_tx_build_tofu(
        &self,
        wallet_id: &PersistWalletId,
        target: &TxBuildTarget,
    ) -> Result<(), ErrorObjectOwned> {
        match target {
            TxBuildTarget::Card(card) => {
                let result = self
                    .service
                    .tofu_verify_pin(wallet_id, card, None)
                    .await
                    .map_err(|err| {
                        ErrorObjectOwned::owned(
                            -32602,
                            format!("SEND_TOFU_REJECTED:{err}"),
                            None::<()>,
                        )
                    })?;

                if matches!(result, crate::receiver::VerifyResult::Verified | crate::receiver::VerifyResult::NewPin) {
                    self.service
                        .tofu_confirm(wallet_id, &card.owner_handle, &card.view_pk)
                        .await
                        .map_err(|err| {
                            ErrorObjectOwned::owned(
                                -32602,
                                format!("SEND_TOFU_REJECTED:{err}"),
                                None::<()>,
                            )
                        })?;
                }

                match result {
                    crate::receiver::VerifyResult::Verified
                    | crate::receiver::VerifyResult::NewPin => Ok(()),
                    crate::receiver::VerifyResult::ViewKeyChanged { .. }
                    | crate::receiver::VerifyResult::IdentityKeyChanged => Err(
                        ErrorObjectOwned::owned(
                            -32602,
                            "SEND_TOFU_CONFIRM_REQUIRED".to_string(),
                            None::<()>,
                        ),
                    ),
                }
            }
            TxBuildTarget::Request(request) => {
                let mut pins = self.service.load_tofu(wallet_id).await.map_err(|err| {
                    ErrorObjectOwned::owned(
                        -32602,
                        format!("SEND_TOFU_REJECTED:{err}"),
                        None::<()>,
                    )
                })?;
                let (chain_id, _, _) = self.tx_wallet_chain_meta()?;

                let outcome = crate::receiver::ValidatePaymentRequest::validate_all(
                    request,
                    &mut pins,
                    chain_id,
                )
                .map_err(|_| {
                    ErrorObjectOwned::owned(-32602, "SEND_REQUEST_INVALID".to_string(), None::<()>)
                })?;

                self.service.save_tofu(wallet_id, &pins).await.map_err(|err| {
                    ErrorObjectOwned::owned(
                        -32602,
                        format!("SEND_TOFU_REJECTED:{err}"),
                        None::<()>,
                    )
                })?;

                match outcome {
                    crate::receiver::ValidationOutcome::Approved => Ok(()),
                    crate::receiver::ValidationOutcome::RequiresUserConfirmation
                    | crate::receiver::ValidationOutcome::IdentityMismatch => Err(
                        ErrorObjectOwned::owned(
                            -32602,
                            "SEND_TOFU_CONFIRM_REQUIRED".to_string(),
                            None::<()>,
                        ),
                    ),
                }
            }
        }
    }

    async fn broadcast_transaction_impl(
        &self,
        session: SessionToken,
        tx_data: String,
        idempotency_key: Option<IdempotencyKey>,
        timestamp_ms: Option<u64>,
    ) -> RpcResult<RuntimeBroadcastTxResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        if tx_data.trim().is_empty() {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid tx_data: must not be empty".to_string(),
                None::<()>,
            ));
        }

        let (tx_bytes, pkg) = self.parse_tx_pkg(&tx_data).await?;
        let verify = crate::tx::verify_full_tx_package(&tx_bytes).map_err(|error| {
            ErrorObjectOwned::owned(-32602, format!("Invalid tx package: {error}"), None::<()>)
        })?;
        if !verify.valid {
            return Err(ErrorObjectOwned::owned(
                -32602,
                format!("Invalid tx package: {}", verify.errors.join("; ")),
                None::<()>,
            ));
        }

        let tx_hash = pkg.tx_digest_hex.clone();
        let tx_id = PersistTxId::new(format!("tx_{tx_hash}"));
        let summary = tx_runtime_state::tx_package_summary(&tx_bytes).ok_or_else(|| {
            ErrorObjectOwned::owned(-32602, "Invalid tx package summary".to_string(), None::<()>)
        })?;
        let (chain_id, _, _) = self.tx_wallet_chain_meta()?;
        let submitter_role = match self.load_stored_tx_record(&wallet_id, &tx_id).await {
            Ok(record) if record.imported => TxSubmitterRole::Receiver,
            _ => TxSubmitterRole::Sender,
        };

        let (attempts, submit_result) = run_with_retry(TX_BROADCAST_MAX_RETRIES, || Ok(()));

        match submit_result {
            Ok(()) => {
                let admission = self
                    .tx_admitter
                    .admit(tx_rpc_admission::WalletTxAdmissionRequest {
                        tx_id: tx_id.clone(),
                        tx_hash_hex: tx_hash,
                        tx_bytes: tx_bytes.clone(),
                        chain_id,
                        submitter_role,
                        idempotency_key,
                        requested_at: self.now_ms(),
                    })
                    .map_err(|error| {
                        ErrorObjectOwned::owned(
                            -32603,
                            format!("Admission failed: {error}"),
                            None::<()>,
                        )
                    })?;
                let confirmation = self.tx_admitter.confirm(&admission).map_err(|error| {
                    ErrorObjectOwned::owned(
                        -32603,
                        format!("Confirmation evidence failed: {error}"),
                        None::<()>,
                    )
                })?;
                let confirmation_evidence = tx_rpc_admission::confirmation_to_evidence(
                    &confirmation,
                    chain_id,
                );

                self.persist_pending_tx(
                    &wallet_id,
                    &tx_id,
                    tx_bytes,
                    summary.amount,
                    summary.fee,
                    false,
                    timestamp_ms.unwrap_or_else(|| self.now_ms()),
                )
                .await?;
                self.journal_admission(&wallet_id, &tx_id).await?;
                self.store_confirmation_evidence(confirmation_evidence).await;
                let _public_receipt = RuntimeAdmissionReceipt::from(&admission);

                Ok(RuntimeBroadcastTxResponse {
                    status: RuntimeOperationStatus {
                        success: true,
                        message: String::new(),
                    },
                    tx_id,
                })
            }
            Err(err) => Err(ErrorObjectOwned::owned(
                -32603,
                format!("Broadcast failed after {attempts} attempt(s): {err}"),
                None::<()>,
            )),
        }
    }

    async fn build_transaction_impl(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
    ) -> RpcResult<RuntimeBuildTxResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        if amount == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: must be > 0".to_string(),
                None::<()>,
            ));
        }

        self.tx_build_precheck(&wallet_id).await.map_err(
            |(retry_after_seconds, current_count, max_requests)| {
                ErrorObjectOwned::owned(
                    SecurityErrorCode::RateLimitExceeded.code(),
                    "Rate limit exceeded".to_string(),
                    Some(RuntimeRateLimitError {
                        method: "wallet.tx.build_transaction".to_string(),
                        tier: RuntimeRateLimitTier::WRITE_OPS.name.to_string(),
                        retry_after_seconds,
                        current_count,
                        max_requests,
                        window_seconds: tx_rpc_rate_limits::BUILD_TX_RATE_LIMIT_WINDOW,
                    }),
                )
            },
        )?;

        let has_explicit_asset_id = asset_id.is_some();
        let parsed_asset_id = Self::parse_asset_id_hex(asset_id)?;
        self.reject_non_asset_cash_id(&wallet_id, parsed_asset_id).await?;
        self.validate_policy(&wallet_id, parsed_asset_id, &recipient, amount)
            .await?;

        let target = parse_tx_build_target(&recipient)?;
        self.verify_tx_build_tofu(&wallet_id, &target).await?;

        let recv_keys = self
            .service
            .receiver_keys(&wallet_id)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;
        let recv_secret = *recv_keys.reveal_receiver_secret().as_bytes();
        let mut spendable_rows = self
            .service
            .list_spendable_asset_rows(
                &wallet_id,
                has_explicit_asset_id.then_some(parsed_asset_id),
            )
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;
        if spendable_rows.is_empty() {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "No spendable assets available".to_string(),
                None::<()>,
            ));
        }

        if has_explicit_asset_id {
            spendable_rows.retain(|row| row.asset_definition_id == parsed_asset_id);
        } else if let Some(first_asset_id) =
            spendable_rows.first().map(|row| row.asset_definition_id)
        {
            spendable_rows.retain(|row| row.asset_definition_id == first_asset_id);
        }
        let claimed_assets = spendable_rows
            .into_iter()
            .map(|row| {
                row.validate_invariants().map_err(|error| {
                    ErrorObjectOwned::owned(-32603, error.to_string(), None::<()>)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if claimed_assets.is_empty() {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "No spendable assets available".to_string(),
                None::<()>,
            ));
        }
        let selector = crate::tx::AssetSelectorImpl::new(SystemRngProvider);
        let change_card = recv_keys
            .export_receiver_card()
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        let change_tofu = self
            .service
            .tofu_verify_pin(&wallet_id, &change_card, None)
            .await
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        match change_tofu {
            crate::receiver::VerifyResult::Verified
            | crate::receiver::VerifyResult::NewPin => {
                self.service
                    .tofu_confirm(&wallet_id, &change_card.owner_handle, &change_card.view_pk)
                    .await
                    .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
            }
            crate::receiver::VerifyResult::ViewKeyChanged { .. }
            | crate::receiver::VerifyResult::IdentityKeyChanged => {
                return Err(ErrorObjectOwned::owned(
                    -32603,
                    "wallet change receiver card requires TOFU confirmation".to_string(),
                    None::<()>,
                ));
            }
        }

        let (chain_id, chain_type, chain_name) = self.tx_wallet_chain_meta()?;
        let mut pins = self
            .service
            .load_tofu(&wallet_id)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;
        let tx_digest = self.tx_seed_bytes();
        let tx_seed = self.tx_seed_bytes();
        let mut sender_wallet = crate::stealth::SenderWallet::new(tx_seed);
        let mut fee = 0u64;
        let mut selection = crate::tx::AssetSelector::select(
            &selector,
            &claimed_assets,
            amount,
            fee,
            crate::tx::SelectionStrategy::MinInputs,
        )
        .map_err(|err| ErrorObjectOwned::owned(-32602, err.to_string(), None::<()>))?;

        let fee_assembler = crate::tx::TxAssemblerImpl::new();
        for _ in 0..3 {
            let asset_template = selection.inputs.first().ok_or_else(|| {
                ErrorObjectOwned::owned(-32603, "missing selected input".to_string(), None::<()>)
            })?;
            let output_serial = inherited_output_serial(asset_template)?;
            let recipient_serial = output_serial;
            let change_serial = output_serial;
            let fee_serial = output_serial;
            let recipient_probe_blinding = self.tx_scalar()?;
            let recipient_probe = match &target {
                TxBuildTarget::Card(card) => crate::stealth::output::build_card_bundle_rng_checked(
                    recipient.clone(),
                    crate::tx::TxOutRole::Recipient,
                    asset_template.definition.class,
                    card,
                    crate::stealth::BuildCheck {
                        pins: &mut pins,
                        chain_id,
                    },
                    &mut sender_wallet,
                    &tx_digest,
                    0,
                    self.tx_seed_bytes(),
                    amount,
                    recipient_serial,
                    Some(&recipient_probe_blinding),
                ),
                TxBuildTarget::Request(request) => {
                    let card = tx_request_to_card(request);
                    crate::stealth::output::build_tx_bundle_rng_checked(
                        recipient.clone(),
                        crate::tx::TxOutRole::Recipient,
                        asset_template.definition.class,
                        &card,
                        Some(request),
                        crate::stealth::BuildCheck {
                            pins: &mut pins,
                            chain_id,
                        },
                        &mut sender_wallet,
                        &tx_digest,
                        0,
                        self.tx_seed_bytes(),
                        amount,
                        recipient_serial,
                        Some(&recipient_probe_blinding),
                    )
                }
            }
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;

            let mut probe_wires = Vec::new();
            let mut recipient_probe_asset = asset_template.clone();
            recipient_probe_asset.amount = amount;
            recipient_probe_asset.serial_id = recipient_serial;
            recipient_probe_asset.nonce = self.tx_seed_bytes();
            probe_wires.push(
                crate::stealth::bind_stealth_output_wire(
                    z00z_core::AssetWire::from_asset(&recipient_probe_asset),
                    &recipient_probe.leaf,
                )
                .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?,
            );

            if selection.change_amount > 0 {
                let change_probe_blinding = self.tx_scalar()?;
                let change_probe = crate::stealth::output::build_card_bundle_rng_checked(
                    wallet_id.0.clone(),
                    crate::tx::TxOutRole::Change,
                    asset_template.definition.class,
                    &change_card,
                    crate::stealth::BuildCheck {
                        pins: &mut pins,
                        chain_id,
                    },
                    &mut sender_wallet,
                    &tx_digest,
                    1,
                    self.tx_seed_bytes(),
                    1,
                    change_serial,
                    Some(&change_probe_blinding),
                )
                .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
                let mut change_probe_asset = asset_template.clone();
                change_probe_asset.amount = 1;
                change_probe_asset.serial_id = change_serial;
                change_probe_asset.nonce = self.tx_seed_bytes();
                probe_wires.push(
                    crate::stealth::bind_stealth_output_wire(
                        z00z_core::AssetWire::from_asset(&change_probe_asset),
                        &change_probe.leaf,
                    )
                    .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?,
                );
            }

            let fee_probe_blinding = self.tx_scalar()?;
            let fee_probe = crate::stealth::output::build_card_bundle_rng_checked(
                wallet_id.0.clone(),
                crate::tx::TxOutRole::Fee,
                z00z_core::assets::AssetClass::Coin,
                &change_card,
                crate::stealth::BuildCheck {
                    pins: &mut pins,
                    chain_id,
                },
                &mut sender_wallet,
                &tx_digest,
                if selection.change_amount > 0 { 2 } else { 1 },
                self.tx_seed_bytes(),
                1,
                fee_serial,
                Some(&fee_probe_blinding),
            )
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
            let mut fee_probe_asset = asset_template.clone();
            fee_probe_asset.amount = 1;
            fee_probe_asset.serial_id = fee_serial;
            fee_probe_asset.nonce = self.tx_seed_bytes();
            probe_wires.push(
                crate::stealth::bind_stealth_output_wire(
                    z00z_core::AssetWire::from_asset(&fee_probe_asset),
                    &fee_probe.leaf,
                )
                .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?,
            );

            let next_fee = fee_assembler
                .calculate_fee_for_wires(selection.inputs.len(), &probe_wires)
                .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
            if next_fee == fee {
                break;
            }
            fee = next_fee;
            selection = crate::tx::AssetSelector::select(
                &selector,
                &claimed_assets,
                amount,
                fee,
                crate::tx::SelectionStrategy::MinInputs,
            )
            .map_err(|err| ErrorObjectOwned::owned(-32602, err.to_string(), None::<()>))?;
        }

        let resolved_inputs = selection
            .inputs
            .iter()
            .map(|asset| {
                let wire = z00z_core::AssetWire::from_asset(asset);
                let plain = crate::tx::resolve_input_pack(recv_secret, &wire).map_err(|err| {
                    ErrorObjectOwned::owned(-32603, err, None::<()>)
                })?;
                let blinding = z00z_crypto::Z00ZScalar::try_from_bytes(plain.blinding).map_err(|_| {
                    ErrorObjectOwned::owned(
                        -32603,
                        "invalid input blinding bytes".to_string(),
                        None::<()>,
                    )
                })?;
                let bytes = crate::tx::tx_assembler::encode_asm_input_wire(
                    asset.asset_id(),
                    asset.serial_id,
                    plain.value,
                    plain.blinding,
                    wire.commitment.as_bytes().try_into().map_err(|_| {
                        ErrorObjectOwned::owned(
                            -32603,
                            "invalid input commitment bytes".to_string(),
                            None::<()>,
                        )
                    })?,
                )
                .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
                Ok::<(Vec<u8>, z00z_crypto::Z00ZScalar), ErrorObjectOwned>((bytes, blinding))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let input_bytes = resolved_inputs
            .iter()
            .map(|(bytes, _)| bytes.clone())
            .collect::<Vec<_>>();
        let input_blind_sum = resolved_inputs
            .iter()
            .skip(1)
            .fold(resolved_inputs[0].1.dangerous_clone(), |acc, (_, blinding)| &acc + blinding);

        let asset_template = selection.inputs.first().ok_or_else(|| {
            ErrorObjectOwned::owned(-32603, "missing selected input".to_string(), None::<()>)
        })?;
        let output_serial = inherited_output_serial(asset_template)?;
        let recipient_serial = output_serial;
        let change_serial = output_serial;
        let fee_serial = output_serial;
        let fee_blinding = self.tx_scalar()?;
        let recipient_blinding = if selection.change_amount > 0 {
            self.tx_scalar()?
        } else {
            crate::tx::balance_blindings(&input_blind_sum, &[fee_blinding.dangerous_clone()])
        };

        let recipient_bundle = match &target {
            TxBuildTarget::Card(card) => crate::stealth::output::build_card_bundle_rng_checked(
                recipient.clone(),
                crate::tx::TxOutRole::Recipient,
                asset_template.definition.class,
                card,
                crate::stealth::BuildCheck {
                    pins: &mut pins,
                    chain_id,
                },
                &mut sender_wallet,
                &tx_digest,
                0,
                self.tx_seed_bytes(),
                amount,
                recipient_serial,
                Some(&recipient_blinding),
            ),
            TxBuildTarget::Request(request) => {
                let card = tx_request_to_card(request);
                crate::stealth::output::build_tx_bundle_rng_checked(
                    recipient.clone(),
                    crate::tx::TxOutRole::Recipient,
                    asset_template.definition.class,
                    &card,
                    Some(request),
                    crate::stealth::BuildCheck {
                        pins: &mut pins,
                        chain_id,
                    },
                    &mut sender_wallet,
                    &tx_digest,
                    0,
                    self.tx_seed_bytes(),
                    amount,
                    recipient_serial,
                    Some(&recipient_blinding),
                )
            }
        }
        .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;

        let mut tx_outputs_bytes = Vec::new();
        let mut recipient_asset = asset_template.clone();
        recipient_asset.amount = amount;
        recipient_asset.serial_id = recipient_serial;
        recipient_asset.nonce = self.tx_seed_bytes();
        let recipient_wire = crate::stealth::bind_stealth_output_wire(
            z00z_core::AssetWire::from_asset(&recipient_asset),
            &recipient_bundle.leaf,
        )
        .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?;
        tx_outputs_bytes.push(
            z00z_utils::codec::JsonCodec
                .serialize(&crate::tx::TxOutputWire {
                    role: crate::tx::TxOutRole::Recipient,
                    asset_wire: z00z_core::assets::AssetPkgWire::from_wire(&recipient_wire),
                })
                .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?,
        );

        if selection.change_amount > 0 {
            let change_blinding = crate::tx::balance_blindings(
                &input_blind_sum,
                &[
                    recipient_blinding.dangerous_clone(),
                    fee_blinding.dangerous_clone(),
                ],
            );
            let change_bundle = crate::stealth::output::build_card_bundle_rng_checked(
                wallet_id.0.clone(),
                crate::tx::TxOutRole::Change,
                asset_template.definition.class,
                &change_card,
                crate::stealth::BuildCheck {
                    pins: &mut pins,
                    chain_id,
                },
                &mut sender_wallet,
                &tx_digest,
                1,
                self.tx_seed_bytes(),
                selection.change_amount,
                change_serial,
                Some(&change_blinding),
            )
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;

            let mut change_asset = asset_template.clone();
            change_asset.amount = selection.change_amount;
            change_asset.serial_id = change_serial;
            change_asset.nonce = self.tx_seed_bytes();
            let change_wire = crate::stealth::bind_stealth_output_wire(
                z00z_core::AssetWire::from_asset(&change_asset),
                &change_bundle.leaf,
            )
            .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?;
            tx_outputs_bytes.push(
                z00z_utils::codec::JsonCodec
                    .serialize(&crate::tx::TxOutputWire {
                        role: crate::tx::TxOutRole::Change,
                        asset_wire: z00z_core::assets::AssetPkgWire::from_wire(&change_wire),
                    })
                    .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?,
            );
        }

        let fee_bundle = crate::stealth::output::build_card_bundle_rng_checked(
            wallet_id.0.clone(),
            crate::tx::TxOutRole::Fee,
            z00z_core::assets::AssetClass::Coin,
            &change_card,
            crate::stealth::BuildCheck {
                pins: &mut pins,
                chain_id,
            },
            &mut sender_wallet,
            &tx_digest,
            if selection.change_amount > 0 { 2 } else { 1 },
            self.tx_seed_bytes(),
            fee,
            fee_serial,
            Some(&fee_blinding),
        )
        .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        let mut fee_asset = asset_template.clone();
        fee_asset.amount = fee;
        fee_asset.serial_id = fee_serial;
        fee_asset.nonce = self.tx_seed_bytes();
        let fee_wire = crate::stealth::bind_stealth_output_wire(
            z00z_core::AssetWire::from_asset(&fee_asset),
            &fee_bundle.leaf,
        )
        .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?;
        tx_outputs_bytes.push(
            z00z_utils::codec::JsonCodec
                .serialize(&crate::tx::TxOutputWire {
                    role: crate::tx::TxOutRole::Fee,
                    asset_wire: z00z_core::assets::AssetPkgWire::from_wire(&fee_wire),
                })
                .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?,
        );

        let raw_tx_bytes = crate::tx::TxAssembler::assemble(
            &crate::tx::TxAssemblerImpl::new(),
            crate::tx::TxAssemblyParams {
                inputs_bytes: input_bytes,
                tx_outputs_bytes,
                fee,
                chain_id,
                chain_type: chain_type.clone(),
                chain_name: chain_name.clone(),
            },
        )
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        let selected_input_wires = selection
            .inputs
            .iter()
            .map(z00z_core::AssetWire::from_asset)
            .collect::<Vec<_>>();
        let mut package: crate::tx::TxPackage = z00z_utils::codec::JsonCodec
            .deserialize(&raw_tx_bytes)
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        let (prev_root, membership) = crate::tx::prepare_spend_membership_witnesses(
            &selected_input_wires,
            &package.tx.inputs,
        )
        .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?;
        let proof_inputs = crate::tx::prepare_spend_public_inputs(
            chain_id,
            recv_secret,
            &selected_input_wires,
            &package.tx.inputs,
        )
        .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?;
        let input_s_in = selected_input_wires
            .iter()
            .map(|item| crate::tx::resolve_input_pack(recv_secret, item).map(|pack| pack.s_out))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| ErrorObjectOwned::owned(-32603, err, None::<()>))?;
        let receiver_secret = crate::key::ReceiverSecret::from_bytes(recv_secret)
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        let mut auth_rng = rand::rngs::StdRng::from_seed(self.tx_seed_bytes());
        let (proof, auth) = crate::tx::build_spend_contract_with_rng(
            &recv_keys,
            chain_id,
            1,
            &chain_type,
            &chain_name,
            &package.tx,
            prev_root,
            proof_inputs,
            crate::tx::SpendProofWitness {
                receiver_secret,
                input_s_in,
                membership,
            },
            &mut auth_rng,
        )
        .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        package.tx.proof = proof;
        package.tx.auth = auth;
        package.tx_digest_hex = crate::tx::build_tx_package_digest(
            &package.kind,
            &package.package_type,
            package.version,
            package.chain_id,
            &package.chain_type,
            &package.chain_name,
            &package.tx,
        )
        .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        let raw_tx_bytes = z00z_utils::codec::JsonCodec
            .serialize(&package)
            .map_err(|err| ErrorObjectOwned::owned(-32603, err.to_string(), None::<()>))?;
        let raw_tx = String::from_utf8(raw_tx_bytes.clone()).map_err(|err| {
            ErrorObjectOwned::owned(
                -32603,
                format!("assembled tx is not valid utf-8 json: {err}"),
                None::<()>,
            )
        })?;
        let tx_id = PersistTxId::new(format!("tx_{}", package.tx_digest_hex));
        let timestamp_ms = self.now_ms();
        let reserved_asset_ids = selection
            .inputs
            .iter()
            .map(|asset| asset.asset_id())
            .collect::<Vec<_>>();

        self.service
            .reserve_claimed_asset_inputs(&wallet_id, &tx_id, reserved_asset_ids.as_slice())
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;

        if let Err(error) = self
            .persist_pending_tx(
            &wallet_id,
            &tx_id,
            raw_tx_bytes,
            amount,
            fee,
            false,
            timestamp_ms,
        )
        .await
        {
            if let Err(rollback) = self
                .service
                .release_claimed_asset_reservation(&wallet_id, &tx_id)
                .await
            {
                return Err(ErrorObjectOwned::owned(
                    -32603,
                    format!(
                        "pending tx persist failed and reservation rollback failed: {}; rollback: {}",
                        error.message(),
                        rollback
                    ),
                    None::<()>,
                ));
            }

            return Err(error);
        }

        Ok(RuntimeBuildTxResponse {
            tx_id,
            raw_tx,
        })
    }

    async fn verify_transaction_package_impl(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeVerifyTxPkgResponse> {
        self.verify_session(&session).await?;
        let (tx_bytes, pkg) = self.parse_tx_pkg(&tx_data).await?;
        let verify = crate::tx::verify_full_tx_package(&tx_bytes).map_err(|error| {
            crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                format!("Invalid tx package: {error}"),
                vec![crate::rpc::error_mapping::map_message_error_code(
                    &error.to_string(),
                )],
                Some(RuntimeTxLifecycle::Failed),
            )
        })?;

        let owned_outputs = if verify.valid {
            self.scan_pkg_outputs(&session.wallet_id, &pkg).await?
        } else {
            Vec::new()
        };
        let lifecycle = if verify.valid {
            tx_rpc_support::lifecycle_from_package_status(&pkg.status)
        } else {
            RuntimeTxLifecycle::Failed
        };
        let status_import_ready = verify.valid && is_import_ready(&pkg.status);
        let all_owned_spendable =
            !owned_outputs.is_empty() && owned_outputs.iter().all(|output| output.can_spend);
        let mut errors = verify.errors;
        let mut error_codes =
            crate::rpc::error_mapping::map_verify_error_codes(&errors);
        if verify.valid && !status_import_ready {
            errors.push("tx package is not import-ready".to_string());
            if !error_codes.contains(&RuntimeTxErrorCode::NotImportReady) {
                error_codes.push(RuntimeTxErrorCode::NotImportReady);
            }
        }
        if verify.valid && owned_outputs.is_empty() {
            errors.push("tx package has no wallet-owned outputs".to_string());
            if !error_codes.contains(&RuntimeTxErrorCode::NoOwnedOutputs) {
                error_codes.push(RuntimeTxErrorCode::NoOwnedOutputs);
            }
        }

        Ok(RuntimeVerifyTxPkgResponse {
            tx_digest_hex: pkg.tx_digest_hex,
            package_status: pkg.status.clone(),
            is_valid: verify.valid,
            lifecycle,
            import_ready: status_import_ready && !owned_outputs.is_empty(),
            all_owned_spendable,
            owned_outputs,
            errors,
            error_codes,
        })
    }

    async fn cancel_transaction_impl(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeCancelTxResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        let record = self.load_stored_tx_record(&wallet_id, &tx_id).await?;
        if !matches!(record.status, crate::persistence::TxStatus::Pending) {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Transaction is not pending".to_string(),
                None::<()>,
            ));
        }

        if let Some(store) = &self.tx_store {
            {
                let mut store = store.write().await;
                store.record_cancelled(&tx_id.0).map_err(|error| {
                    ErrorObjectOwned::owned(
                        -32603,
                        format!("tx cancel journal failed: {error}"),
                        None::<()>,
                    )
                })?;
            }
        } else {
            let mut store = self.wallet_tx_store(&wallet_id);
            store.record_cancelled(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx cancel journal failed: {error}"),
                    None::<()>,
                )
            })?;
        }

        if let Err(error) = self
            .service
            .release_claimed_asset_reservation(&wallet_id, &tx_id)
            .await
        {
            if let Some(store) = &self.tx_store {
                let mut store = store.write().await;
                if let Err(rollback) =
                    store.update_status(&tx_id.0, crate::persistence::TxStatus::Pending)
                {
                    return Err(ErrorObjectOwned::owned(
                        -32603,
                        format!(
                            "tx cancel asset release failed and tx rollback failed: {}; rollback: {}",
                            error, rollback
                        ),
                        None::<()>,
                    ));
                }
            } else {
                let mut store = self.wallet_tx_store(&wallet_id);
                if let Err(rollback) =
                    store.update_status(&tx_id.0, crate::persistence::TxStatus::Pending)
                {
                    return Err(ErrorObjectOwned::owned(
                        -32603,
                        format!(
                            "tx cancel asset release failed and tx rollback failed: {}; rollback: {}",
                            error, rollback
                        ),
                        None::<()>,
                    ));
                }
            }

            return Err(crate::rpc::error_mapping::map_wallet_error_to_rpc(
                error,
            ));
        }

        let mut pending = self.pending_txs.write().await;
        if let Some(tx) = pending
            .iter_mut()
            .find(|tx| tx.wallet_id == wallet_id && tx.id == tx_id)
        {
            tx.status = TxStatus::Cancelled;
        }

        Ok(RuntimeCancelTxResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            tx_id,
        })
    }

    async fn get_transaction_details_impl(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeTxDetailsResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        if let Some(store) = &self.tx_store {
            let store = store.read().await;
            if let Ok(record) = store.get(&tx_id.0) {
                let latest_kind = self.load_tx_latest_kind(&wallet_id, &record.tx_hash).await?;
                let info = tx_runtime_state::tx_record_to_tx_info(
                    wallet_id.clone(),
                    record.clone(),
                    latest_kind,
                );
                return Ok(tx_runtime_state::tx_info_to_details(
                    wallet_id,
                    info,
                    Some(record.tx_bytes.as_slice()),
                ));
            }
        }

        if self.tx_store.is_none() {
            let store = self.wallet_tx_store(&wallet_id);
            if let Ok(record) = store.get(&tx_id.0) {
                let latest_kind = self.load_tx_latest_kind(&wallet_id, &record.tx_hash).await?;
                let info = tx_runtime_state::tx_record_to_tx_info(
                    wallet_id.clone(),
                    record.clone(),
                    latest_kind,
                );
                return Ok(tx_runtime_state::tx_info_to_details(
                    wallet_id,
                    info,
                    Some(record.tx_bytes.as_slice()),
                ));
            }
        }

        let record = {
            let records = self.pending_txs.read().await;
            records
                .iter()
                .find(|tx| tx.wallet_id == wallet_id && tx.id == tx_id)
                .cloned()
        };

        let record = record.ok_or_else(|| {
            ErrorObjectOwned::owned(-32602, "Unknown tx_id".to_string(), None::<()>)
        })?;

        Ok(tx_runtime_state::tx_info_to_details(wallet_id, record, None))
    }

}
