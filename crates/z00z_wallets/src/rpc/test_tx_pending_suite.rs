#[tokio::test]
async fn test_tx_list_paginates_cursor() {
    let time = mock_time_with_offset(2000);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 200_000, 7).await;
    let recipient_a = mk_recv_card_compact(&ctx).await;
    let recipient_b = mk_recv_card_compact(&ctx).await;
    let recipient_c = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let tx1 = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_a,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(2000))),
        )
        .await
        .unwrap()
        .tx_id;
    time.advance_by(Duration::from_secs(1));

    let tx2 = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_b,
            20,
            None,
            None,
            Some(IdempotencyKey(
                "cccccccccccccccccccccccccccccccc".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(2001))),
        )
        .await
        .unwrap()
        .tx_id;
    time.advance_by(Duration::from_secs(1));

    let tx3 = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_c,
            30,
            None,
            None,
            Some(IdempotencyKey(
                "dddddddddddddddddddddddddddddddd".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(2002))),
        )
        .await
        .unwrap()
        .tx_id;

    let rows = tx_history_rows(&ctx);
    assert_eq!(rows.len(), 9);
    assert_eq!(
        rows.iter().map(|row| row.entry_kind).collect::<Vec<_>>(),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
        ]
    );
    assert_eq!(rows[0].sequence, 1);
    assert_eq!(rows[8].sequence, 9);
    for row in &rows {
        assert!(!row.record.tx_bytes.is_empty());
        JsonCodec
            .deserialize::<crate::tx::TxPackage>(&row.record.tx_bytes)
            .expect("stored tx_bytes must be canonical tx package bytes");
    }

    let page1 = rpc
        .list_pending_transactions(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(2),
                cursor: None,
                include_total: Some(true),
            },
        )
        .await
        .unwrap();

    assert_eq!(page1.items.len(), 2);
    assert!(page1.has_more);
    assert_eq!(page1.total_count, Some(3));
    assert_eq!(page1.items[0].id, tx3);
    assert_eq!(page1.items[1].id, tx2);
    assert!(page1
        .items
        .iter()
        .all(|item| matches!(item.status, TxStatus::Pending)));
    assert!(page1
        .items
        .iter()
        .all(|item| matches!(item.lifecycle, RuntimeTxLifecycle::Admitted)));

    let cursor = page1.next_cursor.clone().expect("next_cursor");
    assert_eq!(cursor, tx2.0);

    let page2 = rpc
        .list_pending_transactions(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(999),
                cursor: Some(cursor),
                include_total: Some(false),
            },
        )
        .await
        .unwrap();

    assert_eq!(page2.items.len(), 1);
    assert!(!page2.has_more);
    assert_eq!(page2.items[0].id, tx1);
}

#[tokio::test]
async fn test_tx_estimate_is_dynamic() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let short = rpc
        .estimate_transaction_fee(ctx.session.clone(), "a".to_string(), 10, None)
        .await
        .unwrap();

    let long = rpc
        .estimate_transaction_fee(
            ctx.session.clone(),
            "this-is-a-longer-recipient".to_string(),
            10,
            None,
        )
        .await
        .unwrap();

    assert_eq!(short.fee_per_byte, 1);
    assert_eq!(long.fee_per_byte, 1);
    assert!(long.estimated_fee > short.estimated_fee);
}

async fn claimed_asset_ids(ctx: &TestSessionCtx) -> Vec<String> {
    let mut ids = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("list claimed assets")
        .into_iter()
        .map(|asset| hex::encode(asset.asset_id()))
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn assert_tx_error_payload(
    err: &jsonrpsee::types::ErrorObjectOwned,
    expected_codes: &[crate::rpc::types::tx::RuntimeTxErrorCode],
    expected_lifecycle: Option<RuntimeTxLifecycle>,
) {
    let data = err.data().expect("typed tx error payload");
    let parsed: crate::rpc::error_mapping::RuntimeTxRpcErrorData =
        JsonCodec
            .deserialize(data.get().as_bytes())
            .expect("typed tx error payload json");
    assert_eq!(parsed.error_codes, expected_codes);
    assert_eq!(parsed.lifecycle, expected_lifecycle);
}

#[tokio::test]
async fn test_tx_export_portable_json() {
    let time = mock_time_with_offset(3000);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3000))),
        )
        .await
        .unwrap()
        .tx_id;

    let resp = rpc
        .export_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();

    assert!(resp.success);
    let path = resp.export_path.expect("export_path");

    let hash = compute_wallet_file_id(&ctx.session.wallet_id.0);
    let wallet_id_hex = hex::encode(&hash[..8]);
    let expected = tx_runtime_state::tx_export_dir_for_output(ctx.service.output_dir())
        .join(format!("tx_{wallet_id_hex}.json"));
    assert_eq!(std::path::PathBuf::from(&path), expected);

    let contents = z00z_utils::io::read_to_string(&path).unwrap();
    let portable: PortableWalletTxPackage = JsonCodec
        .deserialize(contents.as_bytes())
        .expect("portable tx package");
    assert_eq!(format!("tx_{}", portable.tx_hash_hex), tx_id.0);
    assert_eq!(portable.package_version, 1);
    assert!(!portable.tx_bytes.is_empty());
    let exported_pkg: crate::tx::TxPackage = JsonCodec
        .deserialize(&portable.tx_bytes)
        .expect("exported tx package");
    assert_eq!(exported_pkg.status, "admitted");
    assert!(crate::tx::verify_full_tx_package(&portable.tx_bytes)
        .expect("verify exported package")
        .valid);
    assert_eq!(
        tx_history_kinds(&ctx),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
            WalletTxHistoryEntryKind::Exported,
        ]
    );
}

#[tokio::test]
async fn test_tx_import_reconcile_portable() {
    let time = mock_time_with_offset(3001);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let original_asset_id = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets before send")
        .into_iter()
        .next()
        .expect("seeded asset")
        .asset_id();
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "abababababababababababababababab".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3001))),
        )
        .await
        .unwrap()
        .tx_id;

    let export = rpc
        .export_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    let path = export.export_path.expect("export path");
    let contents = z00z_utils::io::read_to_string(&path).unwrap();

    let raw_import_err = rpc
        .import_transaction(ctx.session.clone(), portable_tx_bytes_from_export(&contents))
        .await
        .unwrap_err();
    assert_eq!(raw_import_err.code(), -32602);
    assert_tx_error_payload(
        &raw_import_err,
        &[crate::rpc::types::tx::RuntimeTxErrorCode::InvalidEncoding],
        Some(RuntimeTxLifecycle::Failed),
    );

    let imported = rpc
        .import_transaction(ctx.session.clone(), contents.clone())
        .await
        .unwrap();
    assert_eq!(imported.tx_id, tx_id);
    assert!(matches!(imported.status, TxStatus::Pending));
    assert_eq!(imported.lifecycle, RuntimeTxLifecycle::Imported);
    assert!(imported.error_codes.is_empty());
    assert!(!imported.imported_outputs.is_empty());

    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let store = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(Arc::new(MockTimeProvider::from_unix_secs(BASE_TIME_SECS.saturating_add(3001)))),
    );
    let stored = crate::persistence::TxStorage::get(&store, &tx_id.0)
        .expect("imported tx must be persisted");
    assert!(stored.imported);

    let mut portable: PortableWalletTxPackage = JsonCodec
        .deserialize(contents.as_bytes())
        .expect("portable tx package");
    let claimed_before_failed_imports = claimed_asset_ids(&ctx).await;
    let history_before_failed_imports = tx_history_kinds(&ctx);
    portable.metadata_hash_hex = hex::encode([1u8; 32]);
    let tampered = String::from_utf8(JsonCodec.serialize(&portable).unwrap()).unwrap();
    let err = rpc
        .import_transaction(ctx.session.clone(), tampered)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_tx_error_payload(
        &err,
        &[crate::rpc::types::tx::RuntimeTxErrorCode::InvalidDigest],
        Some(RuntimeTxLifecycle::Failed),
    );
    assert_eq!(
        claimed_asset_ids(&ctx).await,
        claimed_before_failed_imports,
        "tampered import must not mutate claimed-asset state",
    );
    assert_eq!(
        tx_history_kinds(&ctx),
        history_before_failed_imports,
        "tampered import must not append tx-history rows",
    );

    let mut wrong_version: PortableWalletTxPackage = JsonCodec
        .deserialize(contents.as_bytes())
        .expect("portable tx package");
    wrong_version.package_version = 2;
    let wrong_version_bytes = wrong_version.package_version.to_le_bytes();
    wrong_version.metadata_hash_hex = hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.wallet.portable.metadata.v1",
        &[
            &wrong_version_bytes,
            wrong_version.chain_id.as_bytes(),
            wrong_version.tx_hash_hex.as_bytes(),
        ],
    ));
    let wrong_version = String::from_utf8(JsonCodec.serialize(&wrong_version).unwrap()).unwrap();
    let err = rpc
        .import_transaction(ctx.session.clone(), wrong_version)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_tx_error_payload(
        &err,
        &[crate::rpc::types::tx::RuntimeTxErrorCode::UnsupportedPackageVersion],
        Some(RuntimeTxLifecycle::Failed),
    );
    assert_eq!(
        claimed_asset_ids(&ctx).await,
        claimed_before_failed_imports,
        "wrong-version import must not mutate claimed-asset state",
    );
    assert_eq!(
        tx_history_kinds(&ctx),
        history_before_failed_imports,
        "wrong-version import must not append tx-history rows",
    );

    let mut wrong_chain: PortableWalletTxPackage = JsonCodec
        .deserialize(contents.as_bytes())
        .expect("portable tx package");
    wrong_chain.chain_id = "999".to_string();
    let wrong_chain_bytes = wrong_chain.package_version.to_le_bytes();
    wrong_chain.metadata_hash_hex = hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.wallet.portable.metadata.v1",
        &[
            &wrong_chain_bytes,
            wrong_chain.chain_id.as_bytes(),
            wrong_chain.tx_hash_hex.as_bytes(),
        ],
    ));
    let wrong_chain = String::from_utf8(JsonCodec.serialize(&wrong_chain).unwrap()).unwrap();
    let err = rpc
        .import_transaction(ctx.session.clone(), wrong_chain)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_tx_error_payload(
        &err,
        &[crate::rpc::types::tx::RuntimeTxErrorCode::WrongChain],
        Some(RuntimeTxLifecycle::Failed),
    );
    assert_eq!(
        claimed_asset_ids(&ctx).await,
        claimed_before_failed_imports,
        "wrong-chain import must not mutate claimed-asset state",
    );
    assert_eq!(
        tx_history_kinds(&ctx),
        history_before_failed_imports,
        "wrong-chain import must not append tx-history rows",
    );

    let reconciled = rpc
        .reconcile_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    assert_eq!(reconciled.tx_id, tx_id);
    assert!(matches!(reconciled.status, TxStatus::Confirmed));
    assert_eq!(reconciled.lifecycle, RuntimeTxLifecycle::Confirmed);
    assert!(reconciled.confirmation.verified);

    let details = rpc
        .get_transaction_details(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    assert!(matches!(details.status, TxStatus::Confirmed));
    assert_eq!(details.lifecycle, RuntimeTxLifecycle::Confirmed);
    assert!(details.receipt_verified);
    let receipt = details.receipt.expect("confirmed receipt");
    assert_eq!(receipt.block_hash, reconciled.confirmation.checkpoint_id_hex);
    assert!(receipt.merkle_proof.is_none());
    let rows = tx_history_rows(&ctx);
    let confirmed = rows.last().expect("confirmed row");
    let evidence = confirmed
        .record
        .confirmation_evidence
        .as_ref()
        .expect("confirmed row evidence");
    assert_eq!(evidence.checkpoint_id_hex, reconciled.confirmation.checkpoint_id_hex);
    assert_eq!(evidence.prev_root_hex, reconciled.confirmation.prev_root_hex);
    assert_eq!(evidence.new_root_hex, reconciled.confirmation.new_root_hex);
    let claimed_after = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets after reconcile");
    assert!(!claimed_after
        .iter()
        .any(|asset| asset.asset_id() == original_asset_id));
    for output in &imported.imported_outputs {
        let output_id: [u8; 32] = hex::decode(&output.asset_id_hex)
            .expect("decode imported output id")
            .try_into()
            .expect("imported output asset id shape");
        assert!(claimed_after
            .iter()
            .any(|asset| asset.asset_id() == output_id));
    }
    assert_eq!(
        tx_history_kinds(&ctx),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
            WalletTxHistoryEntryKind::Exported,
            WalletTxHistoryEntryKind::Imported,
            WalletTxHistoryEntryKind::Confirmed,
        ]
    );

    let second = rpc
        .reconcile_transaction(ctx.session.clone(), tx_id)
        .await
        .unwrap();
    assert!(matches!(second.status, TxStatus::Confirmed));
    assert_eq!(tx_history_kinds(&ctx).len(), 6);
}

#[tokio::test]
async fn test_verify_report_maps_errors() {
    let time = mock_time_with_offset(3003);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "01010101010101010101010101010101".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3003))),
        )
        .await
        .unwrap()
        .tx_id;

    let export = rpc
        .export_transaction(ctx.session.clone(), tx_id)
        .await
        .unwrap();
    let path = export.export_path.expect("export path");
    let contents = z00z_utils::io::read_to_string(&path).unwrap();
    let portable: PortableWalletTxPackage = JsonCodec
        .deserialize(contents.as_bytes())
        .expect("portable tx package");
    let admitted_text = String::from_utf8(portable.tx_bytes.clone()).expect("tx package utf8");

    let admitted = rpc
        .verify_transaction_package(ctx.session.clone(), admitted_text.clone())
        .await
        .unwrap();
    assert!(admitted.is_valid);
    assert!(admitted.import_ready);
    assert_eq!(admitted.lifecycle, RuntimeTxLifecycle::Admitted);
    assert!(admitted.error_codes.is_empty(), "{:?}", admitted.error_codes);
    assert!(!admitted.owned_outputs.is_empty());

    let mut prepared_pkg: crate::tx::TxPackage = JsonCodec
        .deserialize(admitted_text.as_bytes())
        .expect("tx package");
    prepared_pkg.status = "prepared".to_string();
    let prepared_text = String::from_utf8(JsonCodec.serialize(&prepared_pkg).unwrap())
        .expect("prepared tx utf8");
    let prepared = rpc
        .verify_transaction_package(ctx.session.clone(), prepared_text)
        .await
        .unwrap();
    assert!(prepared.is_valid);
    assert!(!prepared.import_ready);
    assert_eq!(prepared.lifecycle, RuntimeTxLifecycle::Created);
    assert_eq!(
        prepared.error_codes,
        vec![crate::rpc::types::tx::RuntimeTxErrorCode::NotImportReady]
    );
    assert!(prepared
        .errors
        .iter()
        .any(|error| error.contains("not import-ready")));

    let mut bad_digest_pkg: crate::tx::TxPackage = JsonCodec
        .deserialize(admitted_text.as_bytes())
        .expect("tx package");
    bad_digest_pkg.tx_digest_hex = "00".repeat(32);
    let bad_digest = String::from_utf8(JsonCodec.serialize(&bad_digest_pkg).unwrap())
        .expect("bad-digest tx utf8");
    let digest_report = rpc
        .verify_transaction_package(ctx.session.clone(), bad_digest)
        .await
        .unwrap();
    assert!(!digest_report.is_valid);
    assert!(!digest_report.import_ready);
    assert_eq!(digest_report.lifecycle, RuntimeTxLifecycle::Failed);
    assert_eq!(
        digest_report.error_codes,
        vec![crate::rpc::types::tx::RuntimeTxErrorCode::InvalidDigest]
    );
}

#[tokio::test]
async fn test_rejects_spent_input() {
    let time = mock_time_with_offset(3002);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "02020202020202020202020202020202".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3002))),
        )
        .await
        .unwrap()
        .tx_id;

    let export = rpc
        .export_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    let path = export.export_path.expect("export path");
    let contents = z00z_utils::io::read_to_string(&path).unwrap();
    let portable: PortableWalletTxPackage = JsonCodec
        .deserialize(contents.as_bytes())
        .expect("portable tx package");
    let summary = tx_runtime_state::tx_package_summary(&portable.tx_bytes)
        .expect("tx package summary");
    let input_id = *summary.inputs.first().expect("package input asset id");

    ctx.service
        .release_claimed_asset_reservation(&ctx.wallet_id, &tx_id)
        .await
        .expect("release original reservation");
    ctx.service
        .reserve_claimed_asset_inputs(
            &ctx.wallet_id,
            &PersistTxId::new("tx_foreign_already_spent".to_string()),
            &[input_id],
        )
        .await
        .expect("reserve conflicting input");
    let claimed_before = claimed_asset_ids(&ctx).await;

    let err = rpc
        .import_transaction(ctx.session.clone(), contents)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_tx_error_payload(
        &err,
        &[crate::rpc::types::tx::RuntimeTxErrorCode::AlreadySpent],
        Some(RuntimeTxLifecycle::AlreadySpent),
    );
    assert_eq!(
        claimed_asset_ids(&ctx).await,
        claimed_before,
        "already-spent import must not mutate claimed-asset state",
    );
    assert_eq!(
        tx_history_kinds(&ctx),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
            WalletTxHistoryEntryKind::Exported,
            WalletTxHistoryEntryKind::AlreadySpent,
        ]
    );
}

#[tokio::test]
async fn test_tx_import_adds_outputs() {
    let time = mock_time_with_offset(3004);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let original_asset_id = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets before send")
        .into_iter()
        .next()
        .expect("seeded asset")
        .asset_id();
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "ddddccccbbbbaaaaeeeeffff11112222".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3004))),
        )
        .await
        .unwrap()
        .tx_id;

    let export = rpc
        .export_transaction(ctx.session.clone(), tx_id)
        .await
        .unwrap();
    let path = export.export_path.expect("export path");
    let contents = z00z_utils::io::read_to_string(&path).unwrap();
    let imported = rpc
        .import_transaction(ctx.session.clone(), contents)
        .await
        .unwrap();
    let claimed_after = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets after import");

    assert!(claimed_after
        .iter()
        .any(|asset| asset.asset_id() == original_asset_id));
    for output in &imported.imported_outputs {
        let output_id: [u8; 32] = hex::decode(&output.asset_id_hex)
            .expect("decode imported output id")
            .try_into()
            .expect("imported output asset id shape");
        assert!(claimed_after
            .iter()
            .any(|asset| asset.asset_id() == output_id));
    }
}

#[tokio::test]
async fn test_import_is_idempotent() {
    let time = mock_time_with_offset(3006);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "12121212121212121212121212121212".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3006))),
        )
        .await
        .unwrap()
        .tx_id;

    let export = rpc
        .export_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    let path = export.export_path.expect("export path");
    let contents = z00z_utils::io::read_to_string(&path).unwrap();

    let first = rpc
        .import_transaction(ctx.session.clone(), contents.clone())
        .await
        .unwrap();
    assert_eq!(first.tx_id, tx_id);
    assert!(matches!(first.status, TxStatus::Pending));
    assert_eq!(first.lifecycle, RuntimeTxLifecycle::Imported);
    assert!(first.error_codes.is_empty());
    assert!(!first.imported_outputs.is_empty());

    let claimed_after_first = claimed_asset_ids(&ctx).await;
    let history_after_first = tx_history_kinds(&ctx);
    let imported_row_count = history_after_first
        .iter()
        .filter(|kind| matches!(kind, WalletTxHistoryEntryKind::Imported))
        .count();
    let mut first_output_ids = first
        .imported_outputs
        .iter()
        .map(|output| output.asset_id_hex.clone())
        .collect::<Vec<_>>();
    first_output_ids.sort();

    let second = rpc
        .import_transaction(ctx.session.clone(), contents)
        .await
        .unwrap();
    assert_eq!(second.tx_id, tx_id);
    assert!(matches!(second.status, TxStatus::Pending));
    assert_eq!(second.lifecycle, RuntimeTxLifecycle::Imported);
    assert!(second.error_codes.is_empty());
    let mut second_output_ids = second
        .imported_outputs
        .iter()
        .map(|output| output.asset_id_hex.clone())
        .collect::<Vec<_>>();
    second_output_ids.sort();
    assert_eq!(second_output_ids, first_output_ids);
    assert_eq!(
        claimed_asset_ids(&ctx).await,
        claimed_after_first,
        "duplicate import must not duplicate claimed assets",
    );
    assert_eq!(
        tx_history_kinds(&ctx),
        history_after_first,
        "duplicate import must not append tx-history rows",
    );
    assert_eq!(
        tx_history_kinds(&ctx)
            .iter()
            .filter(|kind| matches!(kind, WalletTxHistoryEntryKind::Imported))
            .count(),
        imported_row_count,
    );
}

#[tokio::test]
async fn test_failpoint_restores_tx_state() {
    let time = mock_time_with_offset(3005);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "11112222333344445555666677778888".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3005))),
        )
        .await
        .unwrap()
        .tx_id;

    let export = rpc
        .export_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    let path = export.export_path.expect("export path");
    let contents = z00z_utils::io::read_to_string(&path).unwrap();
    let claimed_before = claimed_asset_ids(&ctx).await;

    let _env_guard = set_test_env_var("Z00Z_FAIL_ASSET_SAVE", &ctx.wallet_id.0);
    let err = rpc
        .import_transaction(ctx.session.clone(), contents)
        .await
        .unwrap_err();
    assert!(matches!(err.code(), -32019 | -32603));

    assert_eq!(claimed_asset_ids(&ctx).await, claimed_before);

    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let store = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(time),
    );
    let current = crate::persistence::TxStorage::get(&store, &tx_id.0)
        .expect("tx record must remain addressable after rollback");
    assert!(!current.imported, "failed import must restore imported flag");
    assert!(matches!(current.status, crate::persistence::TxStatus::Pending));
    assert!(current.confirmation_evidence.is_none());
}

#[tokio::test]
async fn test_ops_reject_stale_session() {
    let time = mock_time_with_offset(3100);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let build = rpc
        .build_transaction(ctx.session.clone(), recipient.clone(), 10, None)
        .await
        .unwrap();
    let tx_id = build.tx_id.clone();
    let raw_tx = build.raw_tx.clone();

    let export = rpc
        .export_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    let export_path = export.export_path.expect("export path");
    let portable_tx = z00z_utils::io::read_to_string(&export_path).unwrap();

    time.advance_by(AutoLockPolicy::default().timeout + Duration::from_millis(1));

    assert_session_guard_error(
        rpc.build_transaction(ctx.session.clone(), recipient, 10, None)
            .await
            .unwrap_err(),
    );
    assert_session_guard_error(
        rpc.broadcast_transaction(ctx.session.clone(), raw_tx)
            .await
            .unwrap_err(),
    );
    assert_session_guard_error(
        rpc.cancel_transaction(ctx.session.clone(), tx_id.clone())
            .await
            .unwrap_err(),
    );
    assert_session_guard_error(
        rpc.reconcile_transaction(ctx.session.clone(), tx_id.clone())
            .await
            .unwrap_err(),
    );
    assert_session_guard_error(
        rpc.import_transaction(ctx.session.clone(), portable_tx)
            .await
            .unwrap_err(),
    );
    assert_session_guard_error(
        rpc.export_transaction(ctx.session.clone(), tx_id)
            .await
            .unwrap_err(),
    );
}

#[tokio::test]
async fn test_tx_reconcile_confirmation_evidence() {
    let time = mock_time_with_offset(3002);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let build = rpc
        .build_transaction(ctx.session.clone(), recipient, 10, None)
        .await
        .unwrap();
    let tx_bytes = build.raw_tx.as_bytes().to_vec();
    let package: crate::tx::TxPackage = JsonCodec
        .deserialize(&tx_bytes)
        .expect("tx package");
    let tx_id = PersistTxId::new(format!("tx_{}", package.tx_digest_hex));
    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let mut store = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(time),
    );
    crate::persistence::TxStorage::record_submitted(&mut store, &tx_id.0).unwrap();
    crate::persistence::TxStorage::record_admitted(&mut store, &tx_id.0).unwrap();
    let claimed_before = claimed_asset_ids(&ctx).await;

    let err = rpc
        .reconcile_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(
        claimed_asset_ids(&ctx).await,
        claimed_before,
        "reconcile without confirmation evidence must not mutate claimed assets",
    );

    let details = rpc
        .get_transaction_details(ctx.session.clone(), tx_id)
        .await
        .unwrap();
    assert!(matches!(details.status, TxStatus::Pending));
    assert_eq!(details.lifecycle, RuntimeTxLifecycle::Admitted);
    assert!(details.receipt.is_none());
    assert_eq!(
        tx_history_kinds(&ctx),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
        ]
    );
}

#[tokio::test]
async fn test_reconcile_rejects_mismatched_evidence() {
    let time = mock_time_with_offset(3003);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3003))),
        )
        .await
        .unwrap()
        .tx_id;
    let claimed_before = claimed_asset_ids(&ctx).await;

    {
        let mut evidence = rpc.confirmation_evidence.write().await;
        evidence[0].created_asset_ids_hex.push(hex::encode([9u8; 32]));
    }

    let err = rpc
        .reconcile_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(
        claimed_asset_ids(&ctx).await,
        claimed_before,
        "mismatched evidence must leave claimed-asset state unchanged",
    );

    let details = rpc
        .get_transaction_details(ctx.session.clone(), tx_id)
        .await
        .unwrap();
    assert!(matches!(details.status, TxStatus::Pending));
    assert_eq!(details.lifecycle, RuntimeTxLifecycle::Admitted);
    assert_eq!(
        tx_history_kinds(&ctx),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
        ]
    );
}

#[tokio::test]
async fn test_failpoint_restores_pending_state() {
    let time = mock_time_with_offset(3006);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let original_asset_id = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets before send")
        .into_iter()
        .next()
        .expect("seeded asset")
        .asset_id();
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "99990000aaaabbbbccccddddeeeeffff".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(3006))),
        )
        .await
        .unwrap()
        .tx_id;
    let claimed_before = claimed_asset_ids(&ctx).await;

    let _env_guard = set_test_env_var("Z00Z_FAIL_ASSET_SAVE", &ctx.wallet_id.0);
    let err = rpc
        .reconcile_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap_err();
    assert!(matches!(err.code(), -32019 | -32603));

    let details = rpc
        .get_transaction_details(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    assert!(matches!(details.status, TxStatus::Pending));
    assert_eq!(details.lifecycle, RuntimeTxLifecycle::Admitted);
    assert!(details.receipt.is_none());
    assert_eq!(claimed_asset_ids(&ctx).await, claimed_before);

    let claimed_after = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets after failed reconcile");
    assert!(claimed_after
        .iter()
        .any(|asset| asset.asset_id() == original_asset_id));

    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let store = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(time),
    );
    let current = crate::persistence::TxStorage::get(&store, &tx_id.0)
        .expect("tx record must remain addressable after rollback");
    assert!(matches!(current.status, crate::persistence::TxStatus::Pending));
    assert!(current.confirmation_evidence.is_none());
}

#[tokio::test]
async fn test_tx_list_reflects_cancel() {
    let time = mock_time_with_offset(4000);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 100_000, 7).await;
    let recipient_a = mk_recv_card_compact(&ctx).await;
    let recipient_b = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_a = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_a,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "ffffffffffffffffffffffffffffffff".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(4000))),
        )
        .await
        .unwrap()
        .tx_id;

    let tx_b = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_b,
            20,
            None,
            None,
            Some(IdempotencyKey(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(4000))),
        )
        .await
        .unwrap()
        .tx_id;

    let _ = rpc
        .cancel_transaction(ctx.session.clone(), tx_b.clone())
        .await
        .unwrap();

    let pending = rpc
        .list_pending_transactions(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
        )
        .await
        .unwrap();

    assert_eq!(pending.items.len(), 1);
    assert!(pending.items.iter().all(|tx| tx.id != tx_b));

    let cancelled = rpc
        .get_transaction_details(ctx.session.clone(), tx_b)
        .await
        .unwrap();
    assert!(matches!(cancelled.status, TxStatus::Cancelled));
    assert_eq!(cancelled.lifecycle, RuntimeTxLifecycle::Cancelled);

    let details = rpc
        .get_transaction_details(ctx.session.clone(), tx_a)
        .await
        .unwrap();
    assert!(matches!(details.status, TxStatus::Pending));
    assert_eq!(details.lifecycle, RuntimeTxLifecycle::Admitted);
}

#[tokio::test]
async fn test_lifecycle_projection_survives_restart() {
    let time = mock_time_with_offset(5000);
    let ctx = setup_session(time.clone()).await;
    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let time_provider: Arc<dyn z00z_utils::time::TimeProvider> = time.clone();
    let mut store = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(time_provider),
    );

    let mk_record = |tx_hash: &str, imported: bool| crate::persistence::TxRecord {
        tx_hash: tx_hash.to_string(),
        tx_bytes: Vec::new(),
        imported,
        status: crate::persistence::TxStatus::Pending,
        timestamp_ms: time.compat_unix_timestamp_millis(),
        block_height: None,
        confirmation_evidence: None,
    };

    store.put(mk_record("tx_created", false)).unwrap();
    store.record_imported(mk_record("tx_imported", true)).unwrap();
    store.put(mk_record("tx_exported", false)).unwrap();
    store.record_exported("tx_exported").unwrap();
    store.put(mk_record("tx_admitted", false)).unwrap();
    store.record_submitted("tx_admitted").unwrap();
    store.record_admitted("tx_admitted").unwrap();
    store.put(mk_record("tx_confirmed", false)).unwrap();
    store.record_submitted("tx_confirmed").unwrap();
    store.record_admitted("tx_confirmed").unwrap();
    store.record_confirmed("tx_confirmed", 42).unwrap();
    store.put(mk_record("tx_cancelled", false)).unwrap();
    store.record_cancelled("tx_cancelled").unwrap();
    store.put(mk_record("tx_failed", false)).unwrap();
    store.record_failed("tx_failed").unwrap();
    store.put(mk_record("tx_conflicted", false)).unwrap();
    store.record_conflicted("tx_conflicted").unwrap();
    store.put(mk_record("tx_already_spent", false)).unwrap();
    store.record_already_spent("tx_already_spent").unwrap();
    drop(store);

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let history = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            None,
            None,
        )
        .await
        .unwrap();

    let expect_lifecycle = |tx_hash: &str, lifecycle: RuntimeTxLifecycle, status: TxStatus| {
        let tx_id = PersistTxId::new(tx_hash.to_string());
        let item = history
            .items
            .iter()
            .find(|item| item.id == tx_id)
            .unwrap_or_else(|| panic!("missing tx history item: {tx_hash}"));
        assert_eq!(item.lifecycle, lifecycle, "unexpected lifecycle for {tx_hash}");
        assert!(std::mem::discriminant(&item.status) == std::mem::discriminant(&status));
    };

    expect_lifecycle("tx_created", RuntimeTxLifecycle::Created, TxStatus::Pending);
    expect_lifecycle("tx_imported", RuntimeTxLifecycle::Imported, TxStatus::Pending);
    expect_lifecycle("tx_exported", RuntimeTxLifecycle::Exported, TxStatus::Pending);
    expect_lifecycle("tx_admitted", RuntimeTxLifecycle::Admitted, TxStatus::Pending);
    expect_lifecycle("tx_confirmed", RuntimeTxLifecycle::Confirmed, TxStatus::Confirmed);
    expect_lifecycle("tx_cancelled", RuntimeTxLifecycle::Cancelled, TxStatus::Cancelled);
    expect_lifecycle("tx_failed", RuntimeTxLifecycle::Failed, TxStatus::Failed);
    expect_lifecycle(
        "tx_conflicted",
        RuntimeTxLifecycle::Conflicted,
        TxStatus::Failed,
    );
    expect_lifecycle(
        "tx_already_spent",
        RuntimeTxLifecycle::AlreadySpent,
        TxStatus::Failed,
    );
}

fn portable_tx_bytes_from_export(contents: &str) -> String {
    let portable: PortableWalletTxPackage = JsonCodec
        .deserialize(contents.as_bytes())
        .expect("portable tx package");
    String::from_utf8(portable.tx_bytes).expect("tx package utf8")
}
