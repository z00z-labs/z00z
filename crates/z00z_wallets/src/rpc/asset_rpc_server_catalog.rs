impl AssetRpcImpl {
    async fn list_assets_impl(
        &self,
        wallet_id: PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeAssetListFilter>,
    ) -> RpcResult<RuntimeListAssetsResponse> {
        let _ = self
            .service
            .list_assets(&wallet_id, limit, cursor.clone(), filter.clone());

        let limit = limit.unwrap_or(ASSET_LIST_DEFAULT_LIMIT);
        if limit == 0 || limit > ASSET_LIST_MAX_LIMIT {
            return Err(jsonrpsee::types::ErrorObjectOwned::owned(
                -32602,
                format!(
                    "Invalid limit: {} (must be 1..={})",
                    limit, ASSET_LIST_MAX_LIMIT
                ),
                None::<()>,
            ));
        }

        let start = match cursor {
            None => 0usize,
            Some(c) if c.is_empty() => 0usize,
            Some(c) => c.parse::<usize>().map_err(|_| {
                jsonrpsee::types::ErrorObjectOwned::owned(
                    -32602,
                    "Invalid cursor".to_string(),
                    None::<()>,
                )
            })?,
        };

        let skip = self.quarantine_ids(&wallet_id).await;
        let stored = self.load_assets_from_storage(&wallet_id).await?;
        let assets: Vec<AssetWire> = stored
            .into_iter()
            .filter(|asset| !skip.iter().any(|id| id == &asset.asset_id()))
            .map(|asset| Self::to_asset_wire(&asset))
            .collect();

        let filter = filter.unwrap_or(RuntimeAssetListFilter {
            asset_class: None,
            min_balance: None,
        });

        let mut filtered = assets;
        if let Some(class) = filter.asset_class {
            filtered.retain(|a| a.definition.class == class);
        }
        if let Some(min_balance) = filter.min_balance {
            filtered.retain(|a| a.amount >= min_balance);
        }

        let total_count = filtered.len();
        if start >= total_count {
            return Ok(RuntimeListAssetsResponse {
                items: Vec::new(),
                next_cursor: None,
                has_more: false,
                total_count: Some(total_count),
            });
        }

        let end = (start + limit).min(total_count);
        let page = filtered[start..end].to_vec();
        let has_more = end < total_count;
        let next_cursor = has_more.then_some(end.to_string());

        Ok(RuntimeListAssetsResponse {
            items: page,
            next_cursor,
            has_more,
            total_count: Some(total_count),
        })
    }

    async fn add_asset_impl(
        &self,
        session: SessionToken,
        asset_data: String,
    ) -> RpcResult<RuntimeAddAssetResponse> {
        self.verify_session(&session).await?;
        let codec = JsonCodec;
        let wire: DefinitionWire = codec.deserialize(asset_data.as_bytes()).map_err(|_| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32602,
                "Invalid asset_data JSON".to_string(),
                None::<()>,
            )
        })?;

        let def: z00z_core::assets::AssetDefinition = wire.try_into().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32602,
                format!("Invalid asset definition: {e}"),
                None::<()>,
            )
        })?;
        def.validate().map_err(|e| {
            jsonrpsee::types::ErrorObjectOwned::owned(
                -32602,
                format!("Invalid asset definition: {e}"),
                None::<()>,
            )
        })?;

        let asset = RuntimeAssetRef {
            asset_id: def.id,
            serial_id: 0,
            symbol: def.symbol,
            class: def.class,
        };

        Ok(RuntimeAddAssetResponse {
            asset,
            status: RuntimeOperationStatus {
                success: true,
                message: "ASSET_DEFINITION_ACCEPTED".to_string(),
            },
        })
    }

    async fn get_asset_balance_impl(
        &self,
        wallet_id: PersistWalletId,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetBalanceResponse> {
        self.reject_non_asset_alias(&wallet_id, asset_id).await?;
        let stored = self.load_assets_from_storage(&wallet_id).await?;
        let skip = self.quarantine_ids(&wallet_id).await;
        let matching: Vec<_> = stored
            .iter()
            .filter(|a| Self::asset_matches_query_id(a, asset_id))
            .filter(|a| !skip.iter().any(|id| id == &a.asset_id()))
            .collect();

        if matching.is_empty() {
            return Err(Self::unknown_wallet_asset_err(
                "Unknown asset_id for this wallet",
            ));
        }

        let first = matching[0];
        let total = matching.iter().try_fold(0u64, |acc, a| {
            acc.checked_add(a.amount).ok_or_else(|| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Invalid amount: overflow while summing wallet balance".to_string(),
                    None::<()>,
                )
            })
        })?;
        let reserved_asset_ids = self.load_wallet_reserved_asset_ids(&wallet_id).await?;
        let pending_reserved = matching.iter().try_fold(0u64, |acc, asset| {
            if !reserved_asset_ids.contains(&asset.asset_id()) {
                return Ok(acc);
            }

            acc.checked_add(asset.amount).ok_or_else(|| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Invalid amount: overflow while summing pending wallet balance".to_string(),
                    None::<()>,
                )
            })
        })?;

        Ok(RuntimeAssetBalanceResponse {
            asset: Self::runtime_asset_ref(first),
            total,
            available: total.saturating_sub(pending_reserved),
            pending: pending_reserved,
            decimals: first.definition.decimals,
        })
    }

    async fn get_asset_details_impl(
        &self,
        wallet_id: PersistWalletId,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetDetailsResponse> {
        self.reject_non_asset_alias(&wallet_id, asset_id).await?;
        let asset = self
            .load_wallet_asset(&wallet_id, asset_id, "Unknown asset_id for this wallet")
            .await?;

        Self::build_details_from_asset(&asset)
    }

    async fn import_asset_impl(
        &self,
        session: SessionToken,
        asset_data: String,
    ) -> RpcResult<RuntimeImportAssetResponse> {
        self.verify_session(&session).await.map_err(|_| {
            self.import_err(
                ImportRejectReason::SessionInvalid,
                &session.wallet_id,
                None,
                "session verification failed",
            )
        })?;
        let wallet_id = session.wallet_id.clone();

        let codec = JsonCodec;
        let json_value: z00z_utils::codec::Value =
            codec.deserialize(asset_data.as_bytes()).map_err(|_| {
                self.import_err(
                    ImportRejectReason::MalformedJson,
                    &wallet_id,
                    None,
                    "invalid asset_data json",
                )
            })?;

        let has_secret = json_value
            .as_object()
            .map(|root| root.contains_key("secret"))
            .unwrap_or(false);

        if !has_secret {
            let _ = payload_has_secret_field(asset_data.as_bytes()).map_err(|_| {
                self.import_err(
                    ImportRejectReason::MalformedJson,
                    &wallet_id,
                    None,
                    "invalid asset_data json",
                )
            })?;
        }

        if has_secret {
            return Err(self.import_err(
                ImportRejectReason::SecretFieldForbidden,
                &wallet_id,
                None,
                "incoming dto must not carry secret",
            ));
        }

        let dto = decode_asset_pkg_json(asset_data.as_bytes()).map_err(|error| {
            self.import_err(
                dto_reason(&error),
                &wallet_id,
                None,
                "invalid asset_data json",
            )
        })?;
        let validated = dto.to_asset().map_err(|e| {
            let reason = if matches!(e, AssetError::InvalidStealth(_)) {
                ImportRejectReason::StealthInconsistent
            } else {
                ImportRejectReason::CryptoVerifyFailed
            };

            self.import_err(reason, &wallet_id, None, format!("invalid asset: {e}"))
        })?;

        let asset_ref = Self::runtime_asset_ref(&validated);

        validated.verify_complete().map_err(|e| {
            self.import_err(
                ImportRejectReason::CryptoVerifyFailed,
                &wallet_id,
                Some(validated.asset_id()),
                format!("verify_complete failed: {e}"),
            )
        })?;
        self.verify_complete_calls.fetch_add(1, Ordering::Relaxed);

        self.check_owner(&wallet_id, &validated).await?;

        let asset_id = validated.asset_id();
        let (receipt, claim_sig) = self.sign_claim(&wallet_id, asset_id).await?;
        let reservation = self.claim_reserve(&wallet_id, &receipt, &claim_sig)?;

        let (is_inserted, need_finalize_retry) = match self
            .service
            .recv_route(&wallet_id, validated, ReceiveNext::PersistClaim)
            .await
        {
            Ok(inserted) => {
                if inserted {
                    (true, false)
                } else {
                    (
                        false,
                        claim_registry::has_pending_owner(&wallet_id.0, asset_id),
                    )
                }
            }
            Err(err) => {
                self.claim_release(&reservation);
                return Err(self.import_err(
                    ImportRejectReason::ConservationViolation,
                    &wallet_id,
                    Some(asset_id),
                    format!("wallet-native put failed: {err}"),
                ));
            }
        };

        if is_inserted || need_finalize_retry {
            if let Err(err) = self.claim_finalize(&wallet_id, &reservation) {
                self.mark_quarantine(&wallet_id, asset_id).await;
                return Err(err);
            }
        }

        let status_key = if is_inserted {
            "IMPORT_ACCEPTED_NEW"
        } else {
            ImportRejectReason::AlreadyExists.code()
        };

        if !is_inserted && !need_finalize_retry {
            self.claim_release(&reservation);
        }

        let aid = hex::encode(asset_id);
        Logger::info(
            &TracingLogger,
            &format!(
                "wallet_id={} asset_id={} action=import_accepted is_new={} status={} asset import completed",
                wallet_id.0,
                aid,
                is_inserted,
                status_key
            ),
        );

        Ok(RuntimeImportAssetResponse {
            asset: asset_ref,
            status: RuntimeOperationStatus {
                success: true,
                message: if is_inserted {
                    "asset_imported".to_string()
                } else {
                    "asset_already_exists".to_string()
                },
            },
            is_inserted,
            asset_already_exists: !is_inserted,
        })
    }

    async fn merge_assets_impl(
        &self,
        session: SessionToken,
        asset_ids: Vec<AssetId>,
    ) -> RpcResult<RuntimeMergeAssetsResponse> {
        self.verify_session(&session).await?;
        let wallet_id = session.wallet_id.clone();

        if asset_ids.len() < 2 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid asset_ids: must provide at least 2 asset ids".to_string(),
                None::<()>,
            ));
        }

        for asset_id in &asset_ids {
            self.reject_non_asset_alias(&wallet_id, *asset_id).await?;
        }

        let candidates = self
            .load_wallet_assets_by_ids(&wallet_id, &asset_ids)
            .await?;

        if candidates.len() < 2 {
            return Err(Self::unknown_wallet_asset_err(
                "Invalid asset_ids: not enough matching assets in wallet",
            ));
        }

        let merge_definition = candidates
            .first()
            .map(|asset| asset.definition.clone())
            .ok_or_else(|| {
                Self::unknown_wallet_asset_err(
                    "Invalid asset_ids: not enough matching assets in wallet",
                )
            })?;

        if candidates
            .iter()
            .any(|asset| asset.definition.id != merge_definition.id)
        {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid asset_ids: merged assets must share one definition".to_string(),
                None::<()>,
            ));
        }

        let target_amount = candidates.iter().try_fold(0u64, |acc, asset| {
            acc.checked_add(asset.amount).ok_or_else(|| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Invalid amount: overflow while aggregating assets".to_string(),
                    None::<()>,
                )
            })
        })?;

        let selector = AssetSelectorImpl::new(SystemRngProvider);
        let selection = selector
            .select(&candidates, target_amount, 0, SelectionStrategy::MinInputs)
            .map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("Asset selection failed: {e}"), None::<()>)
            })?;

        let next_serial = selection
            .inputs
            .iter()
            .map(|asset| asset.serial_id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let merged_output = self.build_local_mutation_output(
            &wallet_id,
            "merge_assets",
            merge_definition,
            next_serial,
            selection.total_amount,
            0,
        )?;
        let tx_id = self
            .local_mutation_exec(
                &wallet_id,
                "merge_assets",
                &selection.inputs,
                &[merged_output.clone()],
            )
            .submit()?;

        Ok(RuntimeMergeAssetsResponse {
            asset: Self::runtime_asset_ref(&merged_output),
            merged_count: selection.inputs.len(),
            total_amount: selection.total_amount,
            tx_id: Some(tx_id),
        })
    }

    async fn get_asset_metadata_impl(
        &self,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeAssetMetadataResponse> {
        let _ = self.service.get_asset_metadata(asset_id);
        self.get_cached_metadata(&asset_id).await
    }
}
