use serde_json::Value;
use z00z_core::{
    assets::{AssetClass, AssetPkgWire},
    genesis::asset_std::asset_from_dev_class,
};
use z00z_crypto::{
    domains::ReceiverIdDomain,
    domains::ViewKeyDomain,
    hash::{hash_to_scalar_zk, hash_zk},
};
use z00z_wallets::{
    key::{derive_owner_handle, derive_view_secret_key, ReceiverSecret},
    stealth::kdf::compute_tag16,
    tx::{
        build_tx_package_digest, TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire,
        TxPackage, TxProofWire, TxVerifier, TxVerifierImpl, TxWire,
    },
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";
const WALLET_README: &str = include_str!("../README.md");
const WALLET_GUIDE: &str = include_str!("../docs/WALLET-GUIDE.md");
const APP_KERNEL: &str = include_str!("../src/app/app_kernel.rs");
const APP_CHAIN_NETWORK: &str = include_str!("../src/services/app_chain_network.rs");
const WALLET_ZKPACK: &str = include_str!("../src/stealth/zkpack.rs");

fn rid_ctx(sec: &[u8; 32], ctx: &'static str) -> [u8; 32] {
    hash_zk::<ReceiverIdDomain>(ctx, &[sec])
}

fn view_ctx(sec: &[u8; 32], ctx: &'static str) -> [u8; 32] {
    hash_to_scalar_zk::<ViewKeyDomain>(ctx, &[sec])
        .expect("view ctx")
        .to_bytes()
}

fn mk_pkg() -> Vec<u8> {
    let asset = asset_from_dev_class(AssetClass::Coin, 1, 100).expect("asset");
    let tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([1u8; 32]),
            serial_id: 1,
        }],
        outputs: vec![TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_wire(&z00z_core::assets::AssetWire::from_asset(&asset)),
        }],
        fee: 0,
        nonce: 0,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let payload = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: CHAIN_ID,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx: tx.clone(),
        tx_digest_hex: build_tx_package_digest(
            "TxPackage",
            "regular_tx",
            1,
            CHAIN_ID,
            CHAIN_TYPE,
            CHAIN_NAME,
            &tx,
        )
        .expect("digest"),
        status: "prepared".to_string(),
    };

    serde_json::to_vec(&payload).expect("json")
}

#[test]
fn test_rid_domain_parity() {
    let sec = [0x22u8; 32];
    let recv = ReceiverSecret::from_bytes(sec).expect("receiver secret");
    let wallet_hash = derive_owner_handle(&recv);
    let consensus_hash = hash_zk::<ReceiverIdDomain>("", &[&sec]);
    assert_eq!(wallet_hash, consensus_hash);
}

#[test]
fn test_view_domain_parity() {
    let sec = [0x22u8; 32];
    let recv = ReceiverSecret::from_bytes(sec).expect("receiver secret");
    let wallet_sk = derive_view_secret_key(&recv).expect("wallet sk");
    let consensus_sk = hash_to_scalar_zk::<ViewKeyDomain>("", &[&sec]).expect("consensus sk");
    assert_eq!(wallet_sk.to_bytes(), consensus_sk.to_bytes());
}

#[test]
fn test_rid_drift_fail() {
    let sec = [0x22u8; 32];
    let recv = ReceiverSecret::from_bytes(sec).expect("receiver secret");
    let wallet_hash = derive_owner_handle(&recv);

    let good = rid_ctx(&sec, "");
    let drift = rid_ctx(&sec, "RID");

    assert_eq!(wallet_hash, good);
    assert_ne!(wallet_hash, drift);
}

#[test]
fn test_view_drift_fail() {
    let sec = [0x22u8; 32];
    let recv = ReceiverSecret::from_bytes(sec).expect("receiver secret");
    let wallet = derive_view_secret_key(&recv).expect("wallet sk").to_bytes();

    let good = view_ctx(&sec, "");
    let drift = view_ctx(&sec, "VIEW");

    assert_eq!(wallet, good);
    assert_ne!(wallet, drift);
}

#[test]
fn test_domain_base_live() {
    let sec = [0x31u8; 32];

    let base = [rid_ctx(&sec, ""), view_ctx(&sec, "")];
    let drift = [rid_ctx(&sec, "RID"), view_ctx(&sec, "VIEW")];

    assert_ne!(base, drift);
}

#[test]
fn test_tag_order_gap() {
    let k_dh = [0x44u8; 32];
    let leaf_ad = [0x55u8; 32];
    let norm = compute_tag16(&k_dh, &leaf_ad);
    let swap = compute_tag16(&leaf_ad, &k_dh);
    assert_ne!(norm, swap);
}

#[test]
fn test_tx_no_state_mix() {
    let verifier = TxVerifierImpl::new();
    let bytes = mk_pkg();
    assert!(verifier.verify_structure(&bytes).is_ok());

    let mut payload: Value = serde_json::from_slice(&bytes).expect("json");
    let top = payload.as_object_mut().expect("obj");
    let tx = top
        .get_mut("tx")
        .and_then(Value::as_object_mut)
        .expect("tx");
    tx.insert(
        "prev_root".to_string(),
        Value::Array(vec![Value::from(0); 32]),
    );

    let mixed = serde_json::to_vec(&payload).expect("json");
    assert!(verifier.verify_structure(&mixed).is_err());
}

#[test]
fn test_future_terms_stay_bounded() {
    assert!(
        WALLET_README.contains("### 📋 Planned") && WALLET_README.contains("OnionNet, P2P"),
        "wallet README must keep OnionNet under planned scope"
    );
    assert!(
        WALLET_GUIDE.contains("Phase 062 does not claim live transport anonymity."),
        "wallet guide must keep transport anonymity out of live claims"
    );
    assert!(
        WALLET_GUIDE.contains("Excluded scope: external chain trust tiers, linked liability, live"),
        "wallet guide must keep linked-liability and live cross-chain claims excluded"
    );
    assert!(
        APP_KERNEL.contains("OnionNet transport is not represented by `ChainType`"),
        "app kernel must keep OnionNet outside the ChainType transport model"
    );
    assert!(
        APP_KERNEL.contains("local fallback chain selection."),
        "app kernel must keep OnionNet bounded to the current local fallback path"
    );
    assert!(
        !APP_KERNEL.contains("deterministic placeholder"),
        "app kernel must not describe OnionNet as a placeholder contract anymore"
    );
    assert!(
        !APP_KERNEL.contains("Core app stub:"),
        "app kernel must not advertise core app controls as stubs"
    );
    assert!(
        APP_CHAIN_NETWORK.contains("Phase 1: deterministic placeholder that reaches the core app."),
        "app network service must keep OnionNet as a placeholder seam"
    );
    assert!(
        WALLET_ZKPACK.contains("no alternate field-native or Poseidon2 wire is live on this path."),
        "zkpack docs must keep field-native or Poseidon2 parity non-live"
    );
    assert!(
        !WALLET_README.contains("OnionNet transport is live"),
        "wallet README must not promote OnionNet as a live transport"
    );
    assert!(
        !WALLET_GUIDE.contains("live OnionNet transport anonymity"),
        "wallet guide must not claim live OnionNet anonymity"
    );
}
