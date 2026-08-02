#![cfg(not(target_arch = "wasm32"))]

#[path = "test_thin_support.rs"]
mod thin_test_support;

use z00z_wallets::{
    key::generate_identity_keypair,
    rpc::methods::TxRpcServer,
    tx::{
        ThinFallbackReason, ThinIndexError, ThinIndexStore, ThinSnapshot, ThinSnapshotCache,
        ThinSnapshotPin, ThinTransportMode,
    },
};

use thin_test_support::{context_for_entry, fixture_entry, tx_json, ThinRpcEnv};

#[tokio::test]
async fn test_uncertainty_forces_thick() {
    let entry = fixture_entry().await;
    let (old_sk, _) = generate_identity_keypair().unwrap();
    let valid_snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 50, 10, 1_000),
        vec![entry.clone()],
        &old_sk,
    )
    .expect("valid snapshot");
    let valid_pin = ThinSnapshotPin::new(&valid_snapshot, 100).expect("valid pin");

    let mut store = ThinIndexStore::new();
    store
        .publish_snapshot(valid_snapshot.clone())
        .expect("publish valid snapshot");

    let (new_sk, _) = generate_identity_keypair().unwrap();
    let missing_snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 51, 20, 1_000),
        vec![entry.clone()],
        &new_sk,
    )
    .expect("missing snapshot fixture");
    let missing_pin = ThinSnapshotPin::new(&missing_snapshot, 100).expect("missing pin");

    let mut cache = ThinSnapshotCache::new();
    cache.remember_pin(valid_pin);
    cache.remember_pin(missing_pin);

    let fallback = cache
        .build_transport(&store, &entry.tx_bytes, 100)
        .expect("fallback transport");
    assert_eq!(fallback.mode, ThinTransportMode::Thick);
    assert_eq!(fallback.payload_json, tx_json(&entry));
    assert!(matches!(
        fallback.fallback_reason,
        Some(ThinFallbackReason::CacheUnavailable(
            ThinIndexError::SnapshotMissing(ref digest)
        )) if digest == &missing_snapshot.snapshot_digest_hex
    ));

    let post_clear = cache
        .build_transport(&store, &entry.tx_bytes, 100)
        .expect("post-clear transport");
    assert_eq!(post_clear.mode, ThinTransportMode::Thick);
    assert_eq!(
        post_clear.fallback_reason,
        Some(ThinFallbackReason::NoPinnedSnapshot)
    );

    cache
        .pin_snapshot(&store, &valid_snapshot.snapshot_digest_hex, 100)
        .expect("explicit repin");
    assert!(cache
        .build_transport(&store, &entry.tx_bytes, 100)
        .expect("thin after repin")
        .is_thin());
}

#[tokio::test]
async fn test_resubmit_keeps_meaning() {
    let entry = fixture_entry().await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let valid_snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 60, 10, 1_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("valid snapshot");
    let missing_snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 61, 20, 1_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("missing snapshot");

    let mut store = ThinIndexStore::new();
    store
        .publish_snapshot(valid_snapshot.clone())
        .expect("publish valid snapshot");

    let mut cache = ThinSnapshotCache::new();
    cache.remember_pin(ThinSnapshotPin::new(&valid_snapshot, 100).expect("valid pin"));
    cache.remember_pin(ThinSnapshotPin::new(&missing_snapshot, 100).expect("missing pin"));
    let fallback = cache
        .build_transport(&store, &entry.tx_bytes, 100)
        .expect("fallback transport");
    assert_eq!(fallback.mode, ThinTransportMode::Thick);

    let thick_env = ThinRpcEnv::new("thin-fallback-thick", 100).await;
    let fallback_env = ThinRpcEnv::new("thin-fallback-resubmit", 100).await;

    let thick_verify = thick_env
        .rpc
        .verify_transaction_package(thick_env.session.clone(), tx_json(&entry))
        .await
        .expect("verify original thick package");
    let fallback_verify = fallback_env
        .rpc
        .verify_transaction_package(fallback_env.session.clone(), fallback.payload_json.clone())
        .await
        .expect("verify fallback thick package");
    assert_eq!(fallback_verify.tx_digest_hex, thick_verify.tx_digest_hex);
    assert_eq!(fallback_verify.package_status, thick_verify.package_status);
    assert_eq!(fallback_verify.error_codes, thick_verify.error_codes);

    let thick_broadcast = thick_env
        .rpc
        .broadcast_transaction(thick_env.session.clone(), tx_json(&entry))
        .await
        .expect("broadcast original thick package");
    let fallback_broadcast = fallback_env
        .rpc
        .broadcast_transaction(fallback_env.session.clone(), fallback.payload_json)
        .await
        .expect("broadcast fallback thick package");
    assert_eq!(fallback_broadcast.tx_id, thick_broadcast.tx_id);
}
