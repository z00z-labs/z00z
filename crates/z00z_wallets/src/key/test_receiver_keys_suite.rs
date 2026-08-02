use std::collections::HashSet;

use z00z_utils::io::create_dir_all;
use zeroize::Zeroize;

use super::*;
use crate::domains::WalletViewKeyHashProdDomain;

fn derive_owner_handle_ident(receiver_secret: &ReceiverSecret) -> [u8; 32] {
    derive_owner_handle(receiver_secret)
}

fn test_file(name: &str) -> tempfile::NamedTempFile {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test-tmp");
    create_dir_all(&root).expect("temp root");
    tempfile::Builder::new()
        .prefix(name)
        .rand_bytes(6)
        .tempfile_in(&root)
        .expect("tempfile")
}

#[test]
fn test_receiver_secret_zeroize() {
    let mut secret = ReceiverSecret::from_test_bytes([7u8; 32]).expect("secret");
    secret.zeroize();
    assert_eq!(secret.as_bytes(), &[0u8; 32]);
}

#[test]
fn test_receiver_secret_not_zero() {
    let secret = ReceiverSecret::generate().expect("generated");
    assert_ne!(secret.as_bytes(), &[0u8; 32]);
}

#[test]
fn test_receiver_secret_entropy() {
    let mut seen = HashSet::new();
    for _ in 0..16 {
        let secret = ReceiverSecret::generate().expect("generated");
        seen.insert(secret.as_bytes().to_vec());
    }
    assert!(seen.len() > 1);
}

#[test]
fn test_receiver_secret_debug_redaction() {
    let secret = ReceiverSecret::from_test_bytes([0xAB; 32]).expect("secret");
    let hidden = Hidden::hide(secret);
    let text = format!("{hidden:?}");
    assert!(text.contains("REDACTED"));
    assert!(!text.contains("AB"));
}

#[test]
fn test_owner_handle_deterministic() {
    let secret = ReceiverSecret::from_test_bytes([1u8; 32]).expect("secret");
    let h1 = derive_owner_handle(&secret);
    let h2 = derive_owner_handle(&secret);
    assert_eq!(h1, h2);
}

#[test]
fn test_eq_owner_handle() {
    let secret = ReceiverSecret::from_test_bytes([31u8; 32]).expect("secret");
    let handle_a = derive_owner_handle(&secret);
    let handle_b = derive_owner_handle_ident(&secret);
    assert_eq!(handle_a, handle_b);
}

#[test]
fn test_owner_handle_domain_separation() {
    let secret = ReceiverSecret::from_test_bytes([2u8; 32]).expect("secret");
    let owner = derive_owner_handle(&secret);
    let other = hash_zk::<WalletViewKeyHashProdDomain>("RID", &[secret.as_bytes()]);
    assert_ne!(owner, other);
}

#[test]
fn test_owner_handle_collision_resistance() {
    let mut seen = HashSet::new();
    for _ in 0..32 {
        let secret = ReceiverSecret::generate().expect("generated");
        seen.insert(derive_owner_handle(&secret).to_vec());
    }
    assert_eq!(seen.len(), 32);
}

#[test]
fn test_view_key_deterministic() {
    let secret = ReceiverSecret::from_test_bytes([3u8; 32]).expect("secret");
    let k1 = derive_view_secret_key(&secret).expect("view key");
    let k2 = derive_view_secret_key(&secret).expect("view key");
    assert_eq!(k1.as_bytes(), k2.as_bytes());
}

#[test]
fn test_view_key_not_zero() {
    let secret = ReceiverSecret::from_test_bytes([4u8; 32]).expect("secret");
    let view = derive_view_secret_key(&secret).expect("view key");
    assert_ne!(view.as_bytes(), [0u8; 32]);
}

#[test]
fn test_view_pk_not_identity() {
    let secret = ReceiverSecret::from_test_bytes([5u8; 32]).expect("secret");
    let view_sk = derive_view_secret_key(&secret).expect("view key");
    let view_pk = derive_view_public_key(&view_sk).expect("view pk");
    assert_ne!(view_pk.as_bytes(), [0u8; 32]);
}

#[test]
fn test_view_key_encoding_roundtrip() {
    let secret = ReceiverSecret::from_test_bytes([6u8; 32]).expect("secret");
    let view_sk = derive_view_secret_key(&secret).expect("view key");
    let decoded = Z00ZScalar::from_canonical_bytes(view_sk.as_bytes()).expect("decode");
    assert_eq!(decoded.as_bytes(), view_sk.as_bytes());
}

#[test]
fn test_view_key_version_rotation() {
    let secret = ReceiverSecret::from_test_bytes([8u8; 32]).expect("secret");
    let key_0 = derive_rotated_view_secret_key(&secret, 0).expect("v0");
    let key_1 = derive_rotated_view_secret_key(&secret, 1).expect("v1");

    assert_ne!(key_0.as_bytes(), key_1.as_bytes());

    let hash_0 = hash_zk::<WalletViewKeyHashProdDomain>("VIEW_META", &[key_0.as_bytes()]);
    let ver = make_view_key_version(1, Some(hash_0));
    assert_eq!(ver.version, 1);
    assert!(ver.prev_hash.is_some());
}

#[test]
fn test_rotated_view_key() {
    let secret = ReceiverSecret::from_test_bytes([14u8; 32]).expect("secret");
    let key_a = derive_rotated_view_secret_key(&secret, 9).expect("a");
    let key_b = derive_rotated_view_secret_key(&secret, 9).expect("b");
    let key_c = derive_rotated_view_secret_key(&secret, 10).expect("c");

    assert_eq!(key_a.as_bytes(), key_b.as_bytes());
    assert_ne!(key_a.as_bytes(), key_c.as_bytes());
}

#[test]
fn test_identity_key_versioning() {
    let secret = ReceiverSecret::from_test_bytes([9u8; 32]).expect("secret");
    let k0 = derive_identity_secret_key(&secret, 0).expect("v0");
    let k1 = derive_identity_secret_key(&secret, 1).expect("v1");
    assert_ne!(k0.as_bytes(), k1.as_bytes());
}

#[test]
fn test_identity_signature_roundtrip() {
    let secret = ReceiverSecret::from_test_bytes([10u8; 32]).expect("secret");
    let sk = derive_identity_secret_key(&secret, 0).expect("sk");
    let pk = derive_identity_public_key(&sk).expect("pk");
    let sig = sign_identity(&sk, b"msg", b"ctx").expect("sign");
    verify_identity(&pk, b"msg", b"ctx", &sig).expect("verify");
}

#[test]
fn test_identity_signature_wrong_context() {
    let secret = ReceiverSecret::from_test_bytes([11u8; 32]).expect("secret");
    let sk = derive_identity_secret_key(&secret, 0).expect("sk");
    let pk = derive_identity_public_key(&sk).expect("pk");
    let sig = sign_identity(&sk, b"msg", b"ctx-a").expect("sign");
    let check = verify_identity(&pk, b"msg", b"ctx-b", &sig);
    assert!(check.is_err());
}

#[test]
fn test_identity_signature_invalid() {
    let secret_a = ReceiverSecret::from_test_bytes([12u8; 32]).expect("secret");
    let secret_b = ReceiverSecret::from_test_bytes([13u8; 32]).expect("secret");
    let sk = derive_identity_secret_key(&secret_a, 0).expect("sk");
    let pk = derive_identity_public_key(&derive_identity_secret_key(&secret_b, 0).expect("sk"))
        .expect("pk");
    let sig = sign_identity(&sk, b"msg", b"ctx").expect("sign");
    let check = verify_identity(&pk, b"msg", b"ctx", &sig);
    assert!(check.is_err());
}

#[test]
fn test_generate_identity_keypair() {
    let (sk, pk) = generate_identity_keypair().unwrap();
    let expected = Z00ZRistrettoPoint::from_secret_key(&sk);
    assert_eq!(pk.as_bytes(), expected.as_bytes());
    assert_ne!(pk.as_bytes(), [0u8; 32]);
}

#[test]
fn test_wallet_id_no_rand() {
    reset_id_gen_count();

    let secret = ReceiverSecret::from_test_bytes([77u8; 32]).expect("secret");
    let keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
    let card = keys.export_receiver_card().expect("card");

    assert_eq!(card.version, 1);
    assert_eq!(id_gen_count(), 0);
}

#[test]
fn test_receiver_keys_derivation() {
    let secret = ReceiverSecret::from_test_bytes([20u8; 32]).expect("secret");
    let keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
    assert_ne!(keys.owner_handle, [0u8; 32]);
    assert_ne!(keys.view_pk.as_bytes(), [0u8; 32]);
    assert_ne!(keys.identity_pk.as_bytes(), [0u8; 32]);
}

#[test]
fn test_receiver_keys_all_unique() {
    let secret = ReceiverSecret::from_test_bytes([21u8; 32]).expect("secret");
    let keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
    assert_ne!(keys.view_pk.as_bytes(), keys.identity_pk.as_bytes());
    assert_ne!(keys.owner_handle, [0u8; 32]);
}

#[test]
fn test_receiver_keys_zeroize_drop() {
    let secret = ReceiverSecret::from_test_bytes([22u8; 32]).expect("secret");
    let mut keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
    keys.receiver_secret.zeroize();
    keys.view_sk.zeroize();
    keys.identity_sk.zeroize();

    assert_eq!(keys.receiver_secret.reveal().as_bytes(), &[0u8; 32]);
    assert_eq!(keys.view_sk.reveal().as_bytes(), [0u8; 32]);
    assert_eq!(keys.identity_sk.reveal().as_bytes(), [0u8; 32]);
}

#[test]
fn test_export_receiver_card_signature() {
    let secret = ReceiverSecret::from_test_bytes([23u8; 32]).expect("secret");
    let keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
    let card = keys.export_receiver_card().expect("card");

    card.verify().expect("verify");
}

#[test]
fn test_export_receiver_card_fields() {
    let secret = ReceiverSecret::from_test_bytes([24u8; 32]).expect("secret");
    let keys = ReceiverKeys::from_receiver_secret(secret).expect("keys");
    let card = keys.export_receiver_card().expect("card");
    let view_pk = pk_bytes(&keys.view_pk).expect("view pk");
    let identity_pk = pk_bytes(&keys.identity_pk).expect("identity pk");

    assert_eq!(card.version, 1);
    assert_eq!(card.owner_handle, keys.owner_handle);
    assert_eq!(card.view_pk, view_pk);
    assert_eq!(card.identity_pk, identity_pk);
    assert_eq!(card.card_id, None);
    assert_eq!(card.metadata, None);
}

#[test]
fn test_receiver_secret_encrypt_roundtrip() {
    let secret = ReceiverSecret::from_test_bytes([31u8; 32]).expect("secret");
    let encrypted = secret.to_encrypted(b"password").expect("encrypt");
    let recovered = ReceiverSecret::from_encrypted(&encrypted, b"password").expect("decrypt");
    assert_eq!(secret.as_bytes(), recovered.as_bytes());
}

#[test]
fn test_receiver_secret_store_load() {
    let secret = ReceiverSecret::from_test_bytes([41u8; 32]).expect("secret");
    let file = test_file("receiver-secret-store-load");
    let path = file.path().to_path_buf();

    secret.store(&path, b"password").expect("store");
    let loaded = ReceiverSecret::load(&path, b"password").expect("load");
    assert_eq!(secret.as_bytes(), loaded.as_bytes());
}

#[test]
fn test_receiver_secret_wrong_password() {
    let secret = ReceiverSecret::from_test_bytes([42u8; 32]).expect("secret");
    let file = test_file("receiver-secret-wrong-pw");
    let path = file.path().to_path_buf();

    secret.store(&path, b"password-a").expect("store");
    let loaded = ReceiverSecret::load(&path, b"password-b");
    assert!(loaded.is_err());
}

#[test]
fn test_receiver_secret_corrupted_data() {
    let secret = ReceiverSecret::from_test_bytes([43u8; 32]).expect("secret");
    let file = test_file("receiver-secret-corrupt");
    let path = file.path().to_path_buf();

    secret.store(&path, b"password").expect("store");
    let mut encrypted = read_file(&path).expect("read");
    let len = encrypted.len();
    if len > 0 {
        encrypted[len - 1] ^= 0xFF;
    }
    write_file(&path, &encrypted).expect("write");

    let loaded = ReceiverSecret::load(&path, b"password");
    assert!(loaded.is_err());
}
