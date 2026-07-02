#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use z00z_crypto::expert::{encoding::SafePassword, traits::DomainSeparation};
use z00z_utils::time::MockTimeProvider;

use z00z_utils::codec::{BincodeCodec, Codec};
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::{
    domains::AeadEnvelopeDomain,
    key::Z00ZKeyBranch,
    security::encryption::{EncryptedWalletContainer, WalletEncryption},
    wallet::persistence::WalletExportPack,
};

const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
const EXPORT_MAGIC: &[u8] = b"z00z-wexp\0";

fn decode_export_seed_salt(
    export: &z00z_wallets::rpc::types::common::RuntimeEncryptedResponse,
    password: &SafePassword,
) -> [u8; 16] {
    let context = [Z00ZKeyBranch::WalletBackup.as_aad_byte()];
    let aad = z00z_crypto::aead::build_aad_multipart(AeadEnvelopeDomain::domain(), &[&context[..]])
        .expect("wallet export aad");
    let payload = base64::engine::general_purpose::STANDARD
        .decode(export.ciphertext.as_bytes())
        .expect("payload base64");
    let prefix_len = EXPORT_MAGIC.len();
    assert!(payload.len() > prefix_len + 4, "framed export payload");

    let container: EncryptedWalletContainer = BincodeCodec
        .deserialize(&payload[prefix_len + 4..])
        .expect("container decode");
    let plaintext =
        WalletEncryption::decrypt_wallet(password, &aad, &container).expect("decrypt export");
    let pack = BincodeCodec
        .deserialize::<WalletExportPack>(plaintext.as_ref())
        .expect("export pack decode");
    pack.wallet_profile
        .as_ref()
        .and_then(|profile| profile.seed_salt)
        .expect("seed salt present")
}

#[tokio::test]
async fn test_stub_wallet_creation_stub() {
    let time = Arc::new(MockTimeProvider::default());
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let service = WalletService::with_output_dir_and_time(output_dir, time.clone());
    let app = AppService::with_wallet_service(Arc::new(service));

    let response = app
        .create_wallet(
            "Test".to_string(),
            PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .unwrap();
    let wallet_id = response.wallet_id;

    // After implementing real logic, wallet ID is generated from domain-separated hash
    assert!(!wallet_id.0.is_empty());
    assert_ne!(
        wallet_id,
        z00z_wallets::rpc::types::common::PersistWalletId::default()
    );
}

#[tokio::test]
async fn test_stub_wallet_unlock_stub() {
    let time = Arc::new(MockTimeProvider::default());
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let service = WalletService::with_output_dir_and_time(output_dir, time.clone());
    let service = Arc::new(service);
    let app = AppService::with_wallet_service(Arc::clone(&service));

    let unlock_password = SafePassword::from(PASSWORD);

    // First create a wallet with a known password
    let wallet_id = app
        .create_wallet(
            "Test".to_string(),
            PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .unwrap()
        .wallet_id;

    // Now unlock with the correct password
    let wrong_password = SafePassword::from("wrong-password");
    let err = service
        .unlock_wallet_in_memory(&wallet_id, &wrong_password)
        .await
        .expect_err("wrong password must fail");
    assert!(matches!(err, z00z_wallets::WalletError::InvalidPassword));

    // Backoff is enforced after invalid attempts; advance mocked time to allow retry.
    time.advance_by(Duration::from_secs(2));

    let token = service
        .unlock_wallet_in_memory(&wallet_id, &unlock_password)
        .await
        .unwrap();

    // After implementing real logic, session token is randomly generated
    assert!(!token.token.is_empty());
    assert_eq!(token.wallet_id, wallet_id);

    assert_ne!(token.token, "stub-session-token");

    // Subsequent secret ops require a valid session token.
    let shown = service
        .show_seed_phrase(&token, unlock_password.clone(), "I understand".to_string())
        .await
        .expect("show_seed_phrase must succeed with active session");

    let export = service
        .export_wallet_payload(&wallet_id, &unlock_password)
        .await
        .expect("export wallet");
    let salt = decode_export_seed_salt(&export, &unlock_password);

    let nonce_hex = shown
        .encrypted_payload
        .metadata
        .nonce
        .strip_prefix("0x")
        .unwrap_or(&shown.encrypted_payload.metadata.nonce);
    let nonce_bytes = hex::decode(nonce_hex).expect("nonce hex");
    assert_eq!(nonce_bytes.len(), z00z_crypto::aead::XCHACHA_NONCE_SIZE);
    let mut nonce = [0u8; z00z_crypto::aead::XCHACHA_NONCE_SIZE];
    nonce.copy_from_slice(&nonce_bytes);

    let aad = z00z_crypto::aead::build_aad_multipart(
        "wallet.seed_phrase_response",
        &[wallet_id.0.as_bytes()],
    )
    .unwrap();
    let mut key =
        z00z_wallets::security::encryption::WalletEncryption::derive_key(&unlock_password, &salt)
            .expect("derive key");

    let mut envelope = Vec::new();
    envelope.push(z00z_crypto::aead::XCHACHA20_POLY1305_ID);
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&hex::decode(&shown.encrypted_payload.ciphertext).unwrap());
    let recovered = z00z_crypto::aead::open(&key, &aad, &envelope).expect("decrypt");
    key.fill(0);
    assert_eq!(String::from_utf8(recovered).unwrap(), TEST_SEED_PHRASE_24);
}
