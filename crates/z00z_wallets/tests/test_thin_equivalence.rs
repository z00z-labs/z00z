#![cfg(not(target_arch = "wasm32"))]

#[path = "test_thin_support.rs"]
mod thin_test_support;

use z00z_wallets::{
    key::generate_identity_keypair,
    rpc::methods::TxRpcServer,
    tx::{ThinSnapshot, ThinTransportMode},
};

use thin_test_support::{context_for_entry, fixture_entry, tx_json, ThinRpcEnv};

#[tokio::test]
async fn test_verify_matches_checkpoint() {
    let entry = fixture_entry().await;
    let env = ThinRpcEnv::new("thin-equivalence-verify", 100).await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 41, 99_000, 200_000),
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
    assert_eq!(thin.mode, ThinTransportMode::Thin);

    let thick_response = env
        .rpc
        .verify_transaction_package(env.session.clone(), tx_json(&entry))
        .await
        .expect("verify thick package");
    let thin_response = env
        .rpc
        .verify_transaction_package(env.session.clone(), thin.payload_json)
        .await
        .expect("verify thin package");

    assert_eq!(thin_response.tx_digest_hex, thick_response.tx_digest_hex);
    assert_eq!(thin_response.package_status, thick_response.package_status);
    assert_eq!(thin_response.is_valid, thick_response.is_valid);
    assert_eq!(thin_response.lifecycle, thick_response.lifecycle);
    assert_eq!(thin_response.import_ready, thick_response.import_ready);
    assert_eq!(
        thin_response.all_owned_spendable,
        thick_response.all_owned_spendable
    );
    assert_eq!(
        thin_response.owned_outputs.len(),
        thick_response.owned_outputs.len()
    );
    assert_eq!(thin_response.errors, thick_response.errors);
    assert_eq!(thin_response.error_codes, thick_response.error_codes);
}

#[tokio::test]
async fn test_broadcast_keeps_tx_id() {
    let entry = fixture_entry().await;
    let thick_env = ThinRpcEnv::new("thin-equivalence-broadcast-thick", 100).await;
    let thin_env = ThinRpcEnv::new("thin-equivalence-broadcast-thin", 100).await;
    let (identity_sk, _) = generate_identity_keypair().unwrap();
    let snapshot = ThinSnapshot::new_signed(
        context_for_entry(&entry, 42, 99_000, 200_000),
        vec![entry.clone()],
        &identity_sk,
    )
    .expect("signed snapshot");
    thin_env
        .rpc
        .publish_thin_snapshot(snapshot.clone())
        .await
        .expect("publish snapshot");
    thin_env
        .rpc
        .pin_thin_snapshot(&snapshot.snapshot_digest_hex)
        .await
        .expect("pin snapshot");
    let thin = thin_env
        .rpc
        .build_cached_tx_transport(&entry.tx_bytes)
        .await
        .expect("build thin transport");
    assert!(thin.is_thin());

    let thick_response = thick_env
        .rpc
        .broadcast_transaction(thick_env.session.clone(), tx_json(&entry))
        .await
        .expect("broadcast thick package");
    let thin_response = thin_env
        .rpc
        .broadcast_transaction(thin_env.session.clone(), thin.payload_json)
        .await
        .expect("broadcast thin package");

    assert_eq!(thin_response.tx_id, thick_response.tx_id);
}
