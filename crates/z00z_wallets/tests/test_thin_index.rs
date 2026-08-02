#![cfg(not(target_arch = "wasm32"))]

#[path = "test_thin_support.rs"]
mod thin_test_support;

use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::{
    key::generate_identity_keypair,
    rpc::methods::TxRpcServer,
    tx::{
        ThinAssetPathRef, ThinIndexEntry, ThinIndexError, ThinIndexStore, ThinSnapshot,
        ThinWalletTxPackage,
    },
};

use thin_test_support::{
    assert_runtime_tx_error_codes, context_for_entry, expected_package, fixture_entry, ThinRpcEnv,
};

#[tokio::test]
async fn test_roundtrip_resolves_package() {
    let entry = fixture_entry().await;
    let expected_pkg = expected_package(&entry);
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 7, 10, 1_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("signed snapshot");

    let mut store = ThinIndexStore::new();
    store
        .publish_snapshot(snapshot.clone())
        .expect("publish snapshot");
    let fetched = store
        .snapshot(&snapshot.snapshot_digest_hex)
        .expect("fetch snapshot");
    assert_eq!(fetched.snapshot_digest_hex, snapshot.snapshot_digest_hex);

    let pin = store
        .pin_snapshot(&snapshot.snapshot_digest_hex, 100)
        .expect("pin snapshot");
    let thin = ThinWalletTxPackage::new(&pin, &entry).expect("build thin wrapper");
    let (resolved_bytes, resolved_pkg) = store
        .resolve_package(&thin, 100)
        .expect("resolve canonical package");

    assert_eq!(thin.package_kind, "TxPackage");
    assert_eq!(thin.package_type, expected_pkg.package_type);
    assert_eq!(resolved_bytes, entry.tx_bytes);
    assert_eq!(resolved_pkg, expected_pkg);
}

#[tokio::test]
async fn test_rejects_generation_drift() {
    let entry = fixture_entry().await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();

    let stale_snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 3, 10, 25),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("stale snapshot");
    let mut stale_store = ThinIndexStore::new();
    stale_store
        .publish_snapshot(stale_snapshot.clone())
        .expect("publish stale snapshot");
    let stale = stale_store
        .pin_snapshot(&stale_snapshot.snapshot_digest_hex, 50)
        .expect_err("expired snapshot must fail closed");
    assert!(matches!(
        stale,
        ThinIndexError::SnapshotExpired {
            expires_at_ms: 25,
            now_ms: 50
        }
    ));

    let fresh_snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 7, 10, 1_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("fresh snapshot");
    let mut fresh_store = ThinIndexStore::new();
    fresh_store
        .publish_snapshot(fresh_snapshot.clone())
        .expect("publish fresh snapshot");
    let pin = fresh_store
        .pin_snapshot(&fresh_snapshot.snapshot_digest_hex, 100)
        .expect("pin fresh snapshot");
    let mut drift = ThinWalletTxPackage::new(&pin, &entry).expect("thin wrapper");
    drift.compatibility_generation += 1;
    drift
        .refresh_metadata_hash()
        .expect("refresh drift metadata hash");

    let err = fresh_store
        .resolve_package(&drift, 100)
        .expect_err("generation drift must fail closed");
    assert!(matches!(
        err,
        ThinIndexError::SnapshotGenerationMismatch {
            expected: 7,
            actual: 8
        }
    ));
}

#[tokio::test]
async fn test_rejects_ref_drift() {
    let entry = fixture_entry().await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 9, 10, 1_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("signed snapshot");

    let mut store = ThinIndexStore::new();
    store
        .publish_snapshot(snapshot.clone())
        .expect("publish snapshot");
    let pin = store
        .pin_snapshot(&snapshot.snapshot_digest_hex, 100)
        .expect("pin snapshot");
    let thin = ThinWalletTxPackage::new(&pin, &entry).expect("thin wrapper");

    let mut wrong_refs = thin.clone();
    wrong_refs.input_refs.push(ThinAssetPathRef {
        asset_id_hex: "11".repeat(32),
        serial_id: 1,
    });
    wrong_refs
        .refresh_metadata_hash()
        .expect("refresh wrong-ref metadata");
    assert!(matches!(
        store.resolve_package(&wrong_refs, 100),
        Err(ThinIndexError::InputRefMismatch)
    ));

    let mut missing_entry = thin.clone();
    missing_entry.snapshot_entry_id_hex = "aa".repeat(32);
    missing_entry
        .refresh_metadata_hash()
        .expect("refresh missing-entry metadata");
    assert!(matches!(
        store.resolve_package(&missing_entry, 100),
        Err(ThinIndexError::EntryMissing(_))
    ));

    let mut alt_pkg = expected_package(&entry);
    alt_pkg.status = "thin_retry".to_string();
    let alt_entry = ThinIndexEntry::from_tx_bytes(
        JsonCodec
            .serialize(&alt_pkg)
            .expect("serialize alternate package"),
    )
    .expect("alternate entry");
    let (second_sk, _) = generate_identity_keypair().unwrap();
    let equivocated = ThinSnapshot::new_signed(
        context_for_entry(&entry, 9, 10, 1_000),
        vec![alt_entry],
        &second_sk,
    )
    .expect("equivocated snapshot");
    assert!(matches!(
        store.publish_snapshot(equivocated),
        Err(ThinIndexError::SnapshotConflict { .. })
    ));
}

#[tokio::test]
async fn test_verify_uses_thin_path() {
    let env = ThinRpcEnv::new("thin-wallet", 100).await;
    let entry = fixture_entry().await;
    let expected_pkg = expected_package(&entry);
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 11, 10, 200_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("signed thin snapshot");
    env.rpc
        .publish_thin_snapshot(snapshot.clone())
        .await
        .expect("publish thin snapshot");
    let pin = env
        .rpc
        .pin_thin_snapshot(&snapshot.snapshot_digest_hex)
        .await
        .expect("pin thin snapshot");
    let thin = ThinWalletTxPackage::new(&pin, &entry).expect("build thin wrapper");
    let thin_json = String::from_utf8(JsonCodec.serialize(&thin).expect("serialize thin wrapper"))
        .expect("utf8 thin wrapper");

    let response = env
        .rpc
        .verify_transaction_package(env.session, thin_json)
        .await
        .expect("verify thin transaction package");

    assert!(response.is_valid);
    assert_eq!(response.tx_digest_hex, expected_pkg.tx_digest_hex);
    assert_eq!(response.package_status, expected_pkg.status);
}

#[tokio::test]
async fn test_reports_wrapper_errors() {
    let env = ThinRpcEnv::new("thin-wallet-errors", 100).await;
    let entry = fixture_entry().await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 16, 10, 200_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("signed thin snapshot");
    env.rpc
        .publish_thin_snapshot(snapshot.clone())
        .await
        .expect("publish thin snapshot");
    let pin = env
        .rpc
        .pin_thin_snapshot(&snapshot.snapshot_digest_hex)
        .await
        .expect("pin thin snapshot");
    let thin = ThinWalletTxPackage::new(&pin, &entry).expect("build thin wrapper");

    let mut invalid_metadata = thin.clone();
    invalid_metadata.metadata_hash_hex = "00".repeat(32);
    let invalid_metadata_json = String::from_utf8(
        JsonCodec
            .serialize(&invalid_metadata)
            .expect("serialize invalid metadata wrapper"),
    )
    .expect("utf8 invalid metadata wrapper");
    let invalid_metadata_error = env
        .rpc
        .verify_transaction_package(env.session.clone(), invalid_metadata_json)
        .await
        .expect_err("metadata drift must fail closed");
    assert_runtime_tx_error_codes(&invalid_metadata_error, &["invalid_digest"]);

    let mut missing_snapshot = thin.clone();
    missing_snapshot.snapshot_digest_hex = "aa".repeat(32);
    missing_snapshot
        .refresh_metadata_hash()
        .expect("refresh missing-snapshot metadata");
    let missing_snapshot_json = String::from_utf8(
        JsonCodec
            .serialize(&missing_snapshot)
            .expect("serialize missing snapshot wrapper"),
    )
    .expect("utf8 missing snapshot wrapper");
    let missing_snapshot_error = env
        .rpc
        .verify_transaction_package(env.session.clone(), missing_snapshot_json)
        .await
        .expect_err("missing snapshot must fail closed");
    assert_runtime_tx_error_codes(&missing_snapshot_error, &["thin_snapshot_missing"]);

    let mut wrong_refs = thin.clone();
    wrong_refs.input_refs.push(ThinAssetPathRef {
        asset_id_hex: "22".repeat(32),
        serial_id: 2,
    });
    wrong_refs
        .refresh_metadata_hash()
        .expect("refresh wrong-ref metadata");
    let wrong_refs_json = String::from_utf8(
        JsonCodec
            .serialize(&wrong_refs)
            .expect("serialize wrong-ref wrapper"),
    )
    .expect("utf8 wrong-ref wrapper");
    let wrong_refs_error = env
        .rpc
        .verify_transaction_package(env.session.clone(), wrong_refs_json)
        .await
        .expect_err("input ref drift must fail closed");
    assert_runtime_tx_error_codes(&wrong_refs_error, &["thin_snapshot_conflict"]);

    env.time.set_unix_millis(200_001);
    let thin_json = String::from_utf8(
        JsonCodec
            .serialize(&thin)
            .expect("serialize thin wrapper for stale"),
    )
    .expect("utf8 stale thin wrapper");
    let stale_error = env
        .rpc
        .verify_transaction_package(env.session.clone(), thin_json)
        .await
        .expect_err("expired snapshot must fail closed");
    assert_runtime_tx_error_codes(&stale_error, &["thin_snapshot_stale"]);
}
