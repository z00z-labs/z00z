#![allow(deprecated)]

use std::time::Instant;

use z00z_core::assets::AssetLeaf;
use z00z_crypto::{
    domains::ViewKeyDomain, hash::hash_to_scalar_zk, Z00ZRistrettoPoint, Z00ZScalar,
};
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::{
    receiver::receiver_scan_leaf, stealth::ecdh::sender_derive_dh_with_r, stealth::zkpack::ZkPack,
};

#[test]
#[ignore]
fn test_perf_ecdh() {
    let view_sk = hash_to_scalar_zk::<ViewKeyDomain>("", &[&[0x22u8; 32]]).expect("view sk");
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    const ITERS: u64 = 10_000;
    let mut rng = SystemRngProvider.rng();
    let start = Instant::now();
    for _ in 0..ITERS {
        let r = Z00ZScalar::random(&mut rng).unwrap();
        let _ = sender_derive_dh_with_r(&view_pk, &r).expect("sender derive failed");
    }

    let avg_us = start.elapsed().as_micros() as u64 / ITERS;
    println!("ECDH avg: {} us", avg_us);
    assert!(avg_us < 200, "ECDH must be < 200 us, got {}", avg_us);
}

#[test]
#[ignore]
fn test_perf_zkpack_cycle() {
    let k_dh = [0x42u8; 32];
    let leaf_ad = [0xAAu8; 32];
    let r_pub = [0x01u8; 32];
    let asset_id = [0x02u8; 32];
    let plaintext = [0x42u8; 64];

    const ITERS: u64 = 10_000;
    let start = Instant::now();
    for _ in 0..ITERS {
        let enc = ZkPack::encrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, &plaintext);
        let _ = ZkPack::decrypt(&k_dh, &leaf_ad, &r_pub, &asset_id, 0, &enc);
    }

    let avg_us = start.elapsed().as_micros() as u64 / ITERS;
    println!("ZkPack cycle avg: {} us", avg_us);
    assert!(
        avg_us < 500,
        "ZkPack cycle must be < 500 us, got {}",
        avg_us
    );
}

#[test]
#[ignore]
fn test_perf_scan_throughput() {
    let recv_secret =
        z00z_wallets::key::ReceiverSecret::from_bytes([0x22u8; 32]).expect("receiver secret");
    let keys =
        z00z_wallets::key::ReceiverKeys::from_receiver_secret(recv_secret).expect("receiver keys");
    const N: usize = 10_000;

    let leaves: Vec<AssetLeaf> = (0..N)
        .map(|idx| AssetLeaf::dummy_for_scan(idx as u32))
        .collect();
    let start = Instant::now();
    for leaf in &leaves {
        let _ = receiver_scan_leaf(&keys, leaf);
    }

    let throughput = N as f64 / start.elapsed().as_secs_f64();
    println!("Scan throughput: {:.0} leaves/sec", throughput);
    assert!(
        throughput >= 1000.0,
        "Scan must be >= 1000 leaves/sec, got {:.0}",
        throughput
    );
}
