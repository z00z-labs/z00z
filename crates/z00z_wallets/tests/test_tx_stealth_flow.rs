#![allow(deprecated)]

use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_core::Asset;
use z00z_crypto::{
    commitment::{commit_value, verify_opening},
    create_range_proof,
    hash::poseidon2_hash,
    hash_to_scalar_domain, verify_range_proof, Z00ZCommitment, Z00ZScalar,
};
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    receiver::{receiver_scan_leaf, receiver_scan_report},
    receiver::{ScanResult, StealthOutputScanner},
    stealth::ecdh::{receiver_derive_dh, sender_derive_dh_with_r},
    stealth::kdf::{compute_leaf_ad, compute_owner_tag, compute_tag16, derive_k_dh},
    stealth::zkpack::ZkPack,
};

struct TestCase {
    sender_dh: [u8; 32],
    sender_r: z00z_crypto::Z00ZRistrettoPoint,
    bob_keys: ReceiverKeys,
    bob_handle: [u8; 32],
    bob_view_sk: Z00ZScalar,
    asset_id: [u8; 32],
    serial_id: u32,
    amount: u64,
    expected_pack: AssetPackPlain,
    leaf_ad: [u8; 32],
    leaf: AssetLeaf,
}

fn commit_bytes(commitment: &Z00ZCommitment) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = commitment.as_bytes();
    let size = out.len().min(bytes.len());
    out[..size].copy_from_slice(&bytes[..size]);
    out
}

fn build_case() -> TestCase {
    let mut rng = SystemRngProvider.rng();
    let bob_secret = ReceiverSecret::from_bytes([0x22u8; 32]).expect("receiver secret");
    let bob_keys = ReceiverKeys::from_receiver_secret(bob_secret).expect("receiver keys");
    let bob_view_sk = bob_keys.reveal_view_sk().dangerous_clone();
    let bob_handle = bob_keys.owner_handle;
    let bob_view_pk = bob_keys.view_pk.clone();
    let _bob_card = ReceiverCard {
        version: 1,
        owner_handle: bob_handle,
        view_pk: bob_view_pk.to_bytes(),
        identity_pk: [3u8; 32],
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };

    let asset_id = [0x02u8; 32];
    let serial_id = 1u32;
    let amount = 1000u64;

    let sender = sender_derive_dh_with_r(&bob_view_pk, &Z00ZScalar::random(&mut rng).unwrap())
        .expect("sender derive failed");
    let k_dh = derive_k_dh(&sender.dh.to_bytes());

    let blinding = Z00ZScalar::random(&mut rng).unwrap();
    let commitment = commit_value(amount, &blinding);
    let range_proof =
        create_range_proof(amount, &blinding, 64, 0).expect("range proof generation failed");

    let owner_tag = compute_owner_tag(&bob_handle, &k_dh);
    let r_pub_bytes = sender.r_pub.to_bytes();
    let c_amount = commit_bytes(&commitment);
    let leaf_ad = compute_leaf_ad(&asset_id, serial_id, &r_pub_bytes, &owner_tag, &c_amount);

    let expected_pack = AssetPackPlain {
        value: amount,
        blinding: blinding.to_bytes(),
        s_out: [0xDEu8; 32],
    };
    let plain = expected_pack.to_bytes();

    let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub_bytes, &asset_id, serial_id, &plain);
    let tag16 = compute_tag16(&k_dh, &leaf_ad);

    let leaf = AssetLeaf {
        asset_id,
        serial_id,
        r_pub: r_pub_bytes,
        owner_tag,
        c_amount,
        enc_pack: enc,
        range_proof,
        tag16,
    };

    TestCase {
        sender_dh: k_dh,
        sender_r: sender.r_pub,
        bob_keys,
        bob_handle,
        bob_view_sk,
        asset_id,
        serial_id,
        amount,
        expected_pack,
        leaf_ad,
        leaf,
    }
}

fn leaf_commit(leaf: &AssetLeaf) -> Z00ZCommitment {
    z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
        .expect("invalid leaf commitment")
        .0
}

#[test]
fn test_stage4_asset_to_bob() {
    let case = build_case();

    let dh_bob =
        receiver_derive_dh(&case.bob_view_sk, &case.sender_r).expect("receiver derive failed");
    let k_dh_bob = derive_k_dh(&dh_bob.to_bytes());
    assert_eq!(case.sender_dh, k_dh_bob, "ECDH must agree");

    let exp_tag = compute_owner_tag(&case.bob_handle, &k_dh_bob);
    assert_eq!(exp_tag, case.leaf.owner_tag, "Bob must find his asset");

    let dec = ZkPack::decrypt(
        &k_dh_bob,
        &case.leaf_ad,
        &case.leaf.r_pub,
        &case.asset_id,
        case.serial_id,
        &case.leaf.enc_pack,
    )
    .expect("decrypt failed");

    let decrypted = AssetPackPlain::from_bytes(&dec).expect("asset pack decode failed");
    assert_eq!(decrypted.value, case.amount, "Amount must match");

    let commitment = leaf_commit(&case.leaf);
    let recov_blinding = Z00ZScalar::try_from_bytes(decrypted.blinding).expect("invalid blinding");
    assert!(
        verify_opening(&commitment, case.amount, &recov_blinding),
        "Commitment must verify"
    );

    let proof_ok = verify_range_proof(&case.leaf.range_proof, &commitment, 64, 1, 0).is_ok();
    assert!(proof_ok, "Range proof must verify");
}

#[test]
fn test_stage4_misses_bob_asset() {
    let case = build_case();
    let carol_secret = [0x33u8; 32];
    let carol_view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&carol_secret]);
    let carol_handle = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&carol_secret]);

    let dh_carol = receiver_derive_dh(&carol_view_sk, &case.sender_r).expect("carol derive failed");
    let k_dh_carol = derive_k_dh(&dh_carol.to_bytes());
    let carol_tag = compute_owner_tag(&carol_handle, &k_dh_carol);

    assert_ne!(
        carol_tag, case.leaf.owner_tag,
        "Carol must not match owner tag"
    );

    let result = ZkPack::decrypt(
        &k_dh_carol,
        &case.leaf_ad,
        &case.leaf.r_pub,
        &case.asset_id,
        case.serial_id,
        &case.leaf.enc_pack,
    );
    assert_eq!(result, None, "Carol must not decrypt Bob pack");
}

#[test]
fn test_stage4_tag16_scan_accelerator() {
    let case = build_case();
    let tag16_leaf = case.leaf.tag16;

    let dh_bob =
        receiver_derive_dh(&case.bob_view_sk, &case.sender_r).expect("receiver derive failed");
    let k_dh_bob = derive_k_dh(&dh_bob.to_bytes());
    let tag16_bob = compute_tag16(&k_dh_bob, &case.leaf_ad);
    assert_eq!(tag16_bob, tag16_leaf, "Bob tag16 must match");

    let mut found = false;
    for tweak in 0u8..=u8::MAX {
        let mut carol_secret = [0x33u8; 32];
        carol_secret[31] ^= tweak;
        let carol_view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&carol_secret]);
        let dh_carol =
            receiver_derive_dh(&carol_view_sk, &case.sender_r).expect("carol derive failed");
        let k_dh_carol = derive_k_dh(&dh_carol.to_bytes());
        let carol_tag16 = compute_tag16(&k_dh_carol, &case.leaf_ad);
        if carol_tag16 != tag16_leaf {
            found = true;
            break;
        }
    }

    assert!(found, "No non-matching Carol tag16 found");
}

#[test]
fn test_stage4_sender_receiver_roundtrip() {
    let case = build_case();
    let decrypted = receiver_scan_leaf(&case.bob_keys, &case.leaf)
        .expect("receiver scan failed")
        .expect("owned leaf must decrypt");

    assert_eq!(decrypted.value, case.amount, "amount must match");
    assert_eq!(decrypted.s_out, [0xDEu8; 32], "s_out must match");
}

#[test]
fn test_stage4_workflow_sender_receiver() {
    let case = build_case();
    let decrypted = receiver_scan_leaf(&case.bob_keys, &case.leaf)
        .expect("receiver scan failed")
        .expect("owned leaf must decrypt");

    assert_eq!(decrypted, case.expected_pack);
}

#[test]
fn test_stage4_runtime_roundtrip() {
    let case = build_case();
    let scanner = StealthOutputScanner::from_keys(&case.bob_keys);
    let asset = make_runtime_asset(&case);

    match scanner.scan_leaf(&asset) {
        ScanResult::Mine { wallet_output } => {
            assert_eq!(wallet_output.amount, case.amount);
            assert_eq!(wallet_output.asset_id, asset.asset_id());
        }
        other => panic!("expected Mine, got {other:?}"),
    }
}

#[test]
fn test_stage4_path_parity() {
    let case = build_case();
    let scanner = StealthOutputScanner::from_keys(&case.bob_keys);
    let asset = make_runtime_asset(&case);

    let leaf_report =
        receiver_scan_report(&case.bob_keys, &case.leaf).expect("receiver report failed");
    let leaf_pack = receiver_scan_leaf(&case.bob_keys, &case.leaf)
        .expect("receiver scan failed")
        .expect("owned leaf must decrypt");
    let runtime = scanner.scan_leaf(&asset);
    let runtime_report = runtime.recv_report();
    let ScanResult::Mine { wallet_output } = runtime else {
        panic!("expected Mine, got {runtime:?}");
    };

    assert_eq!(leaf_report, runtime_report);
    assert_eq!(leaf_pack.value, case.amount);
    assert_eq!(wallet_output.amount, case.amount);
    assert_eq!(wallet_output.asset_id, asset.asset_id());

    let mut bad_leaf = case.leaf.clone();
    bad_leaf.asset_id[0] ^= 1;
    let mut bad_asset = make_runtime_asset(&case);
    bad_asset.leaf_ad_id = Some(bad_leaf.asset_id);

    let bad_report =
        receiver_scan_report(&case.bob_keys, &bad_leaf).expect("receiver reject report failed");
    let leaf_reject =
        receiver_scan_leaf(&case.bob_keys, &bad_leaf).expect("receiver reject scan failed");
    assert!(leaf_reject.is_none());

    assert_eq!(bad_report, scanner.scan_leaf(&bad_asset).recv_report());
}

fn make_runtime_asset(case: &TestCase) -> Asset {
    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, case.amount).expect("asset");
    asset.leaf_ad_id = Some(case.asset_id);
    asset.serial_id = case.serial_id;
    asset.r_pub = Some(case.leaf.r_pub);
    asset.owner_tag = Some(case.leaf.owner_tag);
    asset.commitment = z00z_crypto::Commitment::from_bytes(&case.leaf.c_amount)
        .expect("commitment")
        .0;
    asset.enc_pack = Some(case.leaf.enc_pack.clone());
    asset.tag16 = Some(case.leaf.tag16);
    asset
}
