#![cfg(not(target_arch = "wasm32"))]

#[path = "test_thin_support.rs"]
mod thin_test_support;

use z00z_wallets::{
    key::generate_identity_keypair,
    tx::{ThinFallbackReason, ThinIndexError, ThinSnapshot, ThinTransportMode},
};

use thin_test_support::{context_for_entry, fixture_entry, tx_json, ThinRpcEnv};

#[tokio::test]
async fn test_defaults_without_pin() {
    let entry = fixture_entry().await;
    let env = ThinRpcEnv::new("thin-cache-default", 100).await;

    let transport = env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("build cached tx transport");

    assert_eq!(transport.mode, ThinTransportMode::Thick);
    assert_eq!(transport.tx_digest_hex, entry.tx_hash_hex);
    assert_eq!(transport.payload_json, tx_json(&entry));
    assert_eq!(
        transport.fallback_reason,
        Some(ThinFallbackReason::NoPinnedSnapshot)
    );
}

#[tokio::test]
async fn test_evicts_stale_pin() {
    let entry = fixture_entry().await;
    let env = ThinRpcEnv::new("thin-cache-stale", 100).await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 12, 99_000, 101_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("signed snapshot");
    env.rpc
        .publish_thin_snapshot(snapshot.clone())
        .await
        .expect("publish snapshot");
    env.rpc
        .pin_thin_snapshot(&snapshot.snapshot_digest_hex)
        .await
        .expect("pin snapshot");

    let thin = env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("build thin transport");
    assert!(thin.is_thin());

    env.time.set_unix_millis(102_000);
    let thick = env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("build thick fallback");
    assert_eq!(thick.mode, ThinTransportMode::Thick);
    assert_eq!(thick.payload_json, tx_json(&entry));
    assert!(matches!(
        thick.fallback_reason,
        Some(ThinFallbackReason::CacheUnavailable(
            ThinIndexError::SnapshotExpired {
                expires_at_ms: 101_000,
                now_ms: 102_000,
            }
        ))
    ));

    let thick_again = env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("build post-eviction fallback");
    assert_eq!(thick_again.mode, ThinTransportMode::Thick);
    assert_eq!(
        thick_again.fallback_reason,
        Some(ThinFallbackReason::NoPinnedSnapshot)
    );
}

#[tokio::test]
async fn test_refresh_restores_thin() {
    let entry = fixture_entry().await;
    let env = ThinRpcEnv::new("thin-cache-refresh", 100).await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let initial = ThinSnapshot::new_signed(
        context_for_entry(&entry, 13, 99_000, 200_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("initial snapshot");
    env.rpc
        .publish_thin_snapshot(initial.clone())
        .await
        .expect("publish initial snapshot");
    env.rpc
        .pin_thin_snapshot(&initial.snapshot_digest_hex)
        .await
        .expect("pin initial snapshot");
    assert!(env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("thin transport")
        .is_thin());

    env.rpc.clear_thin_snapshot_cache().await;
    let fallback = env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("thick fallback after cache reset");
    assert_eq!(fallback.mode, ThinTransportMode::Thick);
    assert_eq!(
        fallback.fallback_reason,
        Some(ThinFallbackReason::NoPinnedSnapshot)
    );

    let refreshed = ThinSnapshot::new_signed(
        context_for_entry(&entry, 14, 99_000, 250_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("refreshed snapshot");
    env.rpc
        .refresh_thin_snapshot(refreshed)
        .await
        .expect("refresh snapshot");
    assert!(env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("thin transport after refresh")
        .is_thin());
}
