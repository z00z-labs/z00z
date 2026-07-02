use super::*;
use crate::chain::ReceiverCardRecord;
use crate::key::ReceiverKeys;
use crate::receiver::receiver_card::encode_card_compact;
use crate::receiver::request::encode_request_compact;
use crate::receiver::{
    receiver_scan_leaf, receiver_scan_report, DetectedAssetPack, PaymentRequest, ReceiveNext,
    ReceiveReject, ReceiveStatus, ReceiverCard, RequestParams, ScanResult, StealthOutputScanner,
};
use crate::services::wallet_service::Sleeper;
use crate::stealth::ecdh::{compute_dh_receiver, decode_r_pub};
use crate::stealth::kdf::{compute_leaf_ad, derive_k_dh};
use crate::stealth::zkpack::ZkPack;
use crate::stealth::{build_tx_output_unchecked, SenderWallet};
use crate::tx::wire_decrypt_leaf;
use crate::WalletError;
use std::ffi::OsString;
use std::future::Future;
use std::pin::Pin;
use tempfile::TempDir;
use z00z_core::assets::{decode_asset_pkg_json, encode_asset_pkg_json, AssetLeaf, AssetPkgWire};
use z00z_core::Asset;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::{create_range_proof, Z00ZScalar};

#[derive(Debug, Clone)]
struct MockSleeper {
    time: Arc<MockTimeProvider>,
}

impl MockSleeper {
    fn new(time: Arc<MockTimeProvider>) -> Self {
        Self { time }
    }
}

impl Sleeper for MockSleeper {
    fn sleep<'a>(&'a self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.time.advance_by(duration);
        })
    }
}

fn flip_sig_byte(node: &mut Value) -> bool {
    match node {
        Value::Array(items) => {
            for item in items {
                if flip_sig_byte(item) {
                    return true;
                }
            }
            false
        }
        Value::Object(map) => {
            for value in map.values_mut() {
                if flip_sig_byte(value) {
                    return true;
                }
            }
            false
        }
        Value::Number(num) => {
            if let Some(val) = num.as_u64() {
                *node = Value::from((val ^ 1) & 0xff);
                return true;
            }
            false
        }
        Value::String(text) => {
            if text.is_empty() {
                return false;
            }
            let mut bytes = text.as_bytes().to_vec();
            bytes[0] = if bytes[0] == b'a' { b'b' } else { b'a' };
            *text = String::from_utf8_lossy(&bytes).to_string();
            true
        }
        _ => false,
    }
}

fn reset_claims() {
    claim_registry::clear_rows();
}

struct WalletConfigEnvRestore {
    prev_path: Option<OsString>,
    prev_network: Option<OsString>,
    prev_chain: Option<OsString>,
}

impl WalletConfigEnvRestore {
    fn capture() -> Self {
        Self {
            prev_path: std::env::var_os("Z00Z_WALLET_CONFIG_PATH"),
            prev_network: std::env::var_os("Z00Z_WALLET_NETWORK"),
            prev_chain: std::env::var_os("Z00Z_WALLET_CHAIN"),
        }
    }
}

impl Drop for WalletConfigEnvRestore {
    fn drop(&mut self) {
        match &self.prev_path {
            Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
        match &self.prev_network {
            Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }
        match &self.prev_chain {
            Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
    }
}

fn mk_rpc() -> AssetRpcImpl {
    let time = Arc::new(MockTimeProvider::default());
    AssetRpcImpl::with_dependencies(time)
}

fn mk_rpc_with_disk() -> (AssetRpcImpl, TempDir, Arc<MockTimeProvider>) {
    let dir = TempDir::new().expect("tempdir");
    let time = Arc::new(MockTimeProvider::default());
    let mut service = WalletService::create_service_custom_output_directory(
        dir.path().to_path_buf(),
        time.clone(),
        SystemRngProvider,
    );
    service.set_sleeper(Arc::new(MockSleeper::new(time.clone())));
    let service = Arc::new(service);
    let rpc = AssetRpcImpl::with_dependencies_and_wallet_service(time.clone(), service);
    (rpc, dir, time)
}

fn mk_recv_card(keys: &ReceiverKeys) -> ReceiverCard {
    ReceiverCard {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
        identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    }
}

async fn mk_owned_stealth_wire(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    serial_id: u32,
) -> AssetWire {
    let asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, 10)
        .expect("valid std asset");
    mk_owned_wire_from_asset(
        rpc,
        wallet_id,
        asset,
        serial_id,
        [41u8; 32],
        [serial_id as u8; 32],
    )
    .await
}

async fn mk_owned_wire_from_asset(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    mut asset: z00z_core::assets::Asset,
    _serial_id: u32,
    tx_seed: [u8; 32],
    tx_dig: [u8; 32],
) -> AssetWire {
    let keys = rpc
        .test_receiver_keys(wallet_id)
        .await
        .expect("receiver keys");

    let card = mk_recv_card(&keys);
    let mut sender_wallet = SenderWallet::new(tx_seed);
    let output = build_tx_output_unchecked(
        &card,
        None,
        &mut sender_wallet,
        &tx_dig,
        asset.serial_id,
        asset.amount,
        &asset.definition.id,
    )
    .expect("stealth output");

    let tag16 = output.tag16.expect("tag16");

    asset.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
        .expect("commitment")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(output.r_pub);
    asset.owner_tag = Some(output.owner_tag);
    asset.enc_pack = Some(output.enc_pack);
    asset.tag16 = Some(tag16);
    asset.leaf_ad_id = Some(asset.definition.id);

    let scanner = StealthOutputScanner::from_keys(&keys);
    if let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) {
        if let Some(blinding_bytes) = wallet_output.blinding.as_ref().copied() {
            let blinding = Z00ZScalar::try_from_bytes(blinding_bytes).expect("blinding scalar");
            asset.range_proof =
                Some(create_range_proof(asset.amount, &blinding, 64, 0).expect("proof"));
        }
    }

    let mut wire = AssetWire::from_asset(&asset);
    wire.leaf_ad_id = Some(asset.definition.id);
    wire.secret = None;
    wire
}

async fn mk_recv_card_compact(rpc: &AssetRpcImpl, wallet_id: &PersistWalletId) -> String {
    let keys = rpc
        .test_receiver_keys(wallet_id)
        .await
        .expect("receiver keys");
    let card = keys.export_receiver_card().expect("receiver card");
    ReceiverCardRecord::new(&card, card.canonical_encoding(), 0)
        .expect("receiver record")
        .to_compact()
        .expect("receiver compact")
}

fn derive_wallet_mark_seed(wallet_id: &PersistWalletId, mark: u8) -> [u8; 32] {
    let mut seed = [mark; 32];
    for (slot, byte) in seed.iter_mut().zip(wallet_id.0.as_bytes().iter().copied()) {
        *slot ^= byte;
    }
    seed
}

async fn mk_req_compact(rpc: &AssetRpcImpl, wallet_id: &PersistWalletId, amount: u64) -> String {
    let keys = rpc
        .test_receiver_keys(wallet_id)
        .await
        .expect("receiver keys");

    let request = PaymentRequest::generate(
        &keys,
        RequestParams {
            amount: Some(amount),
            expiry_seconds: 3600,
            memo: Some("send-flow".to_string()),
            payment_id: None,
        },
        wallet_chain_id().expect("wallet chain id"),
    )
    .expect("request");

    encode_request_compact(&request)
}

async fn seed_assets(rpc: &AssetRpcImpl, session: &SessionToken) {
    let fixtures = [
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Token, 2, 100)
            .expect("valid std asset"),
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Nft, 3, 1)
            .expect("valid std asset"),
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 4, 15)
            .expect("valid std asset"),
    ];

    for asset in &fixtures {
        let _ = rpc
            .service
            .put_claimed_asset(&session.wallet_id, asset.clone())
            .await
            .expect("seed put_claimed_asset must succeed");
    }

    let assets = fixtures
        .into_iter()
        .map(|asset| AssetWire::from_asset(&asset))
        .collect::<Vec<_>>();

    let now_ms = rpc.now_ms();
    {
        let mut list_cache = rpc.asset_list_cache.write().await;
        list_cache.insert(
            session.wallet_id.clone(),
            AssetListCacheValue {
                created_at: now_ms,
                assets,
            },
        );
    }

    let _ = session;
}

async fn ensure_seed_assets(rpc: &AssetRpcImpl, session: &SessionToken) {
    let claimed = rpc
        .service
        .list_claimed_assets_live_cache(&session.wallet_id)
        .await
        .expect("claimed list must load");
    if claimed.len() >= 4 {
        return;
    }

    seed_assets(rpc, session).await;
}

async fn seed_wire_cache(rpc: &AssetRpcImpl, session: &SessionToken, wire: AssetWire) {
    let now_ms = rpc.now_ms();
    let mut list_cache = rpc.asset_list_cache.write().await;
    list_cache.insert(
        session.wallet_id.clone(),
        AssetListCacheValue {
            created_at: now_ms,
            assets: vec![wire],
        },
    );
}

fn encode_test_dto_json(wire: &AssetWire) -> String {
    let dto = z00z_core::assets::AssetPkgWire::from_wire(wire);
    let bytes = z00z_core::assets::encode_asset_pkg_json(&dto).expect("encode dto json");
    String::from_utf8(bytes).expect("dto utf8")
}

fn decode_json_value(input: &str) -> Value {
    JsonCodec.deserialize(input.as_bytes()).expect("json parse")
}

fn encode_json_value(value: &Value) -> String {
    String::from_utf8(JsonCodec.serialize(value).expect("json encode")).expect("json utf8")
}

fn encode_dto_secret(wire: &AssetWire, secret: [u8; 32]) -> String {
    let mut root = decode_json_value(&encode_test_dto_json(wire));
    let object = root.as_object_mut().expect("dto object");
    object.insert("secret".to_string(), Value::String(hex::encode(secret)));
    encode_json_value(&root)
}

fn encode_secret_null(wire: &AssetWire) -> String {
    let mut root = decode_json_value(&encode_test_dto_json(wire));
    let object = root.as_object_mut().expect("dto object");
    object.insert("secret".to_string(), Value::Null);
    encode_json_value(&root)
}

struct RecvBase {
    keys: ReceiverKeys,
    wire: AssetWire,
    asset: Asset,
    leaf: AssetLeaf,
    pack: DetectedAssetPack,
}

fn dto_asset(wire: &AssetWire) -> Asset {
    let dto = AssetPkgWire::from_wire(wire);
    let json = encode_asset_pkg_json(&dto).expect("encode dto");
    decode_asset_pkg_json(&json)
        .expect("decode dto")
        .to_asset()
        .expect("asset")
}

fn wire_asset(wire: &AssetWire) -> Asset {
    wire.clone().to_asset().expect("asset")
}

fn use_leaf(wire: &AssetWire, leaf: &AssetLeaf) -> AssetWire {
    crate::stealth::bind_stealth_output_wire(wire.clone(), leaf).expect("bind output wire")
}

fn leaf_key(keys: &ReceiverKeys, leaf: &AssetLeaf) -> [u8; 32] {
    let r_pub = decode_r_pub(&leaf.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    derive_k_dh(&dh)
}

fn short_leaf(keys: &ReceiverKeys, leaf: &AssetLeaf) -> AssetLeaf {
    let mut bad = leaf.clone();
    let pack = receiver_scan_leaf(keys, leaf)
        .expect("scan")
        .expect("owned pack");
    let mut bytes = pack.to_bytes().expect("pack bytes");
    let _ = bytes.pop().expect("pop");
    let k_dh = leaf_key(keys, leaf);
    let leaf_ad = compute_leaf_ad(
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amount,
    );
    bad.enc_pack = ZkPack::encrypt(
        &k_dh,
        &leaf_ad,
        &leaf.r_pub,
        &leaf.asset_id,
        leaf.serial_id,
        &bytes,
    );
    bad
}

fn bad_open(keys: &ReceiverKeys, leaf: &AssetLeaf) -> AssetLeaf {
    let mut bad = leaf.clone();
    let mut pack = receiver_scan_leaf(keys, leaf)
        .expect("scan")
        .expect("owned pack");
    let k_dh = leaf_key(keys, leaf);
    let leaf_ad = compute_leaf_ad(
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amount,
    );
    pack.value = pack.value.saturating_add(1);
    let bytes = pack.to_bytes().expect("pack bytes");
    bad.enc_pack = ZkPack::encrypt(
        &k_dh,
        &leaf_ad,
        &leaf.r_pub,
        &leaf.asset_id,
        leaf.serial_id,
        &bytes,
    );
    bad
}

fn bad_enc(leaf: &AssetLeaf) -> AssetLeaf {
    let mut bad = leaf.clone();
    bad.enc_pack.ciphertext[0] ^= 1;
    bad
}

async fn claims_empty(rpc: &AssetRpcImpl, wallet_id: &PersistWalletId) {
    let claimed = rpc
        .service
        .list_claimed_assets(wallet_id)
        .await
        .expect("claimed list");
    assert!(
        claimed.is_empty(),
        "tampered receive must not persist claims"
    );
}

async fn assert_rpc_msg(rpc: &AssetRpcImpl, session: &SessionToken, wire: AssetWire, want: &str) {
    assert_eq!(recv_msg(rpc, session, wire).await, want);
}

async fn assert_rpc_bad(
    rpc: &AssetRpcImpl,
    session: &SessionToken,
    wallet_id: &PersistWalletId,
    wire: AssetWire,
) {
    assert_rpc_msg(rpc, session, wire, ReceiveStatus::InvalidProof.rpc_code()).await;
    claims_empty(rpc, wallet_id).await;
}

async fn assert_rpc_ok(rpc: &AssetRpcImpl, session: &SessionToken, wire: AssetWire, want: &str) {
    let asset_id = wire.clone().to_asset().expect("wire asset").asset_id();
    seed_wire_cache(rpc, session, wire).await;
    let recv = rpc
        .receive_asset(session.clone(), asset_id)
        .await
        .expect("receive");
    assert_eq!(recv.status, want);
}

async fn assert_parse_bad(
    rpc: &AssetRpcImpl,
    session: &SessionToken,
    wallet_id: &PersistWalletId,
    scanner: &StealthOutputScanner,
    keys: &ReceiverKeys,
    wire: &AssetWire,
    leaf: &AssetLeaf,
) {
    assert!(matches!(
        receiver_scan_leaf(keys, leaf),
        Err(WalletError::InvalidAssetPack("wrong length"))
    ));
    assert!(matches!(
        receiver_scan_report(keys, leaf),
        Err(WalletError::InvalidAssetPack("wrong length"))
    ));

    let scan = scanner.scan_leaf(&wire_asset(&use_leaf(wire, leaf)));
    assert!(!matches!(scan, ScanResult::Mine { .. }));
    assert_eq!(scan.recv_report().status, ReceiveStatus::InvalidProof);

    assert_rpc_bad(rpc, session, wallet_id, use_leaf(wire, leaf)).await;
}

async fn assert_open_bad(
    rpc: &AssetRpcImpl,
    session: &SessionToken,
    wallet_id: &PersistWalletId,
    scanner: &StealthOutputScanner,
    keys: &ReceiverKeys,
    wire: &AssetWire,
    leaf: &AssetLeaf,
) {
    assert!(matches!(
        receiver_scan_leaf(keys, leaf),
        Err(WalletError::CommitmentMismatch)
    ));
    assert!(matches!(
        receiver_scan_report(keys, leaf),
        Err(WalletError::CommitmentMismatch)
    ));

    let scan = scanner.scan_leaf(&wire_asset(&use_leaf(wire, leaf)));
    assert!(!matches!(scan, ScanResult::Mine { .. }));
    assert_eq!(scan.recv_report().status, ReceiveStatus::InvalidProof);

    assert_rpc_bad(rpc, session, wallet_id, use_leaf(wire, leaf)).await;
}

async fn assert_bad_leaf(
    rpc: &AssetRpcImpl,
    session: &SessionToken,
    wallet_id: &PersistWalletId,
    scanner: &StealthOutputScanner,
    keys: &ReceiverKeys,
    wire: &AssetWire,
    leaf: &AssetLeaf,
) {
    assert!(receiver_scan_leaf(keys, leaf).expect("scan").is_none());
    let report = receiver_scan_report(keys, leaf).expect("report");
    assert_eq!(report.status, ReceiveStatus::InvalidProof);
    assert_eq!(report.reject, Some(ReceiveReject::InvalidProof));

    let scan = scanner.scan_leaf(&wire_asset(&use_leaf(wire, leaf)));
    assert!(!matches!(scan, ScanResult::Mine { .. }));
    assert_eq!(scan.recv_report(), report);

    assert_rpc_bad(rpc, session, wallet_id, use_leaf(wire, leaf)).await;
}

fn report_code(report: &crate::receiver::ReceiveReport) -> &'static str {
    WalletService::recv_code(report.status)
}

async fn build_recv_base(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    serial_id: u32,
) -> RecvBase {
    let wire = mk_owned_stealth_wire(rpc, wallet_id, serial_id).await;
    let asset = dto_asset(&wire);
    let leaf = wire_decrypt_leaf(&wire).expect("leaf");
    let keys = rpc
        .test_receiver_keys(wallet_id)
        .await
        .expect("receiver keys");
    let pack = receiver_scan_leaf(&keys, &leaf)
        .expect("scan")
        .expect("owned pack");

    RecvBase {
        keys,
        wire,
        asset,
        leaf: leaf.into(),
        pack,
    }
}

async fn recv_msg(rpc: &AssetRpcImpl, session: &SessionToken, wire: AssetWire) -> String {
    let asset_id = wire.clone().to_asset().expect("wire asset").asset_id();
    seed_wire_cache(rpc, session, wire).await;
    rpc.receive_asset(session.clone(), asset_id)
        .await
        .expect_err("receive err")
        .message()
        .to_string()
}

async fn cached_asset_ids(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    session: &SessionToken,
) -> Vec<AssetId> {
    ensure_seed_assets(rpc, session).await;

    let resp = rpc
        .list_assets(wallet_id.clone(), Some(ASSET_LIST_MAX_LIMIT), None, None)
        .await
        .unwrap();

    let mut ids = Vec::new();
    for item in resp.items {
        let asset_id = item.clone().to_asset().expect("cached asset").asset_id();
        if !ids.contains(&asset_id) {
            ids.push(asset_id);
        }
    }
    ids
}

async fn cached_asset_id_by_class(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    session: &SessionToken,
    class: AssetClass,
) -> AssetId {
    cached_asset_ids_by_class(rpc, wallet_id, session, class)
        .await
        .into_iter()
        .next()
        .expect("filtered asset must exist")
}

async fn cached_asset_ids_by_class(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    session: &SessionToken,
    class: AssetClass,
) -> Vec<AssetId> {
    ensure_seed_assets(rpc, session).await;

    let resp = rpc
        .list_assets(
            wallet_id.clone(),
            Some(ASSET_LIST_MAX_LIMIT),
            None,
            Some(RuntimeAssetListFilter {
                asset_class: Some(class),
                min_balance: None,
            }),
        )
        .await
        .unwrap();

    resp.items
        .into_iter()
        .map(|asset| asset.to_asset().expect("filtered asset").asset_id())
        .collect()
}

async fn cached_definition_ids(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    session: &SessionToken,
) -> Vec<AssetId> {
    ensure_seed_assets(rpc, session).await;

    rpc.list_assets(wallet_id.clone(), Some(ASSET_LIST_MAX_LIMIT), None, None)
        .await
        .unwrap()
        .items
        .into_iter()
        .map(|item| item.definition.id)
        .collect()
}
use crate::db::test_owned_objects::{test_owned_right_payload, test_owned_voucher_payload};
use crate::db::{OwnedObjectFamily, OwnedObjectPayload, OwnedRightStatus, OwnedVoucherStatus};
use crate::rpc::types::security::SecurityErrorCode;
use crate::rpc::types::wallet::PersistWalletSettings;
use crate::services::WalletService;
use crate::wallet::policy::PolicyRules;
use std::collections::BTreeSet;
use std::time::Duration;
use z00z_core::{
    actions::{
        ActionDescriptorV1, ActionPoolDescriptorV1, LifecycleEffectV1, RequiredSignatureV1,
        WitnessRequirementV1,
    },
    assets::ObjectFamily,
    policies::{
        ConservationContributionV1, ExpiryRuleV1, PolicyDescriptorV1, ReplayProtectionV1,
        UnknownPolicyHandlingV1,
    },
    rights::{RightActionV1, RightRequirementV1, RightScopeV1},
    vouchers::VoucherLifecycleV1,
};
use z00z_storage::settlement::{
    DefinitionId, ObjectDeltaSetV1, ObjectRejectCode, ObjectWitnessBundleV1, RightAction,
    RightWitnessRefV1, RightWitnessStateV1, SerialId, SettlementActionV1, SettlementLeaf,
    SettlementObjectDeltaV1, SettlementPath, SettlementStateRoot, TerminalId, TerminalLeaf,
    VoucherAction, VoucherActionCtx,
};
use z00z_utils::codec::{Codec, JsonCodec, Value};
use z00z_utils::time::MockTimeProvider;

use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn next_test_wallet_name() -> String {
    static TEST_WALLET_SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = TEST_WALLET_SEQ.fetch_add(1, AtomicOrdering::Relaxed);
    format!("test-wallet-{seq}")
}

async fn create_unlocked_session(rpc: &AssetRpcImpl) -> (PersistWalletId, SessionToken) {
    let _lock = crate::rpc::logging::RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let _restore = WalletConfigEnvRestore::capture();
    std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
    std::env::remove_var("Z00Z_WALLET_NETWORK");
    std::env::remove_var("Z00Z_WALLET_CHAIN");

    let password = "test-password".to_string();
    let wallet_name = next_test_wallet_name();
    let wallet_id = rpc
        .service
        .create_wallet_in_memory(
            &wallet_name,
            SafePassword::from(password.clone()),
            TEST_SEED_PHRASE_24,
        )
        .await
        .expect("create_wallet_in_memory must succeed");

    let safe_password = SafePassword::from(password);
    let session = rpc
        .service
        .unlock_wallet_in_memory(&wallet_id, &safe_password)
        .await
        .expect("unlock_wallet_in_memory must succeed");

    (wallet_id, session)
}

async fn create_unlocked_session_chain(
    rpc: &AssetRpcImpl,
    chain: &str,
) -> (PersistWalletId, SessionToken) {
    let _lock = crate::rpc::logging::RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let _restore = WalletConfigEnvRestore::capture();
    std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
    std::env::set_var("Z00Z_WALLET_NETWORK", "p2p");
    std::env::set_var("Z00Z_WALLET_CHAIN", chain);

    let password = "test-password".to_string();
    let wallet_name = next_test_wallet_name();
    let wallet_id = rpc
        .service
        .create_wallet_in_memory(
            &wallet_name,
            SafePassword::from(password.clone()),
            TEST_SEED_PHRASE_24,
        )
        .await
        .expect("create_wallet_in_memory must succeed");

    let safe_password = SafePassword::from(password);
    let session = rpc
        .service
        .unlock_wallet_in_memory(&wallet_id, &safe_password)
        .await
        .expect("unlock_wallet_in_memory must succeed");

    (wallet_id, session)
}

fn object_test_root(seed: u8) -> SettlementStateRoot {
    SettlementStateRoot::settlement_v1([seed; 32])
}

fn object_asset_leaf(path: SettlementPath) -> TerminalLeaf {
    let mut leaf = TerminalLeaf::dummy_for_scan(path.serial_id.get());
    leaf.asset_id = path.terminal_id.into_bytes();
    leaf.serial_id = path.serial_id.get();
    leaf
}

fn refund_asset_path(
    voucher: &crate::db::OwnedVoucherPayload,
    fallback_serial: u32,
) -> SettlementPath {
    let (definition_id, serial_id) = match voucher.voucher_leaf.backing {
        z00z_storage::settlement::VoucherBackingRef::ConsumedAsset {
            definition_id,
            serial_id,
        } => (definition_id, serial_id),
        z00z_storage::settlement::VoucherBackingRef::ReserveCommitment(backing)
        | z00z_storage::settlement::VoucherBackingRef::GenesisReserve(backing) => {
            (backing, fallback_serial)
        }
    };
    SettlementPath::new(
        DefinitionId::new(definition_id),
        SerialId::new(serial_id),
        TerminalId::new(voucher.voucher_leaf.refund_target_commitment),
    )
}

fn voucher_redeem_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "voucher_redeem_full".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        lifecycle_effect: LifecycleEffectV1::Redeem,
        witness_requirements: BTreeSet::from([
            WitnessRequirementV1::Signature(RequiredSignatureV1::Holder),
            WitnessRequirementV1::AcceptanceProof,
            WitnessRequirementV1::ReplayNonce,
            WitnessRequirementV1::PriorStateRoot,
            WitnessRequirementV1::RightReference("kyc_v1".to_string()),
        ]),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "voucher_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "voucher_policy_v1".to_string(),
        domain_name: "z00z.core.policies.voucher_redeem.test.v1".to_string(),
        primary_family: ObjectFamily::Voucher,
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::from([RightRequirementV1 {
            right_policy: "kyc_v1".to_string(),
            allowed_actions: BTreeSet::from([RightActionV1::Use]),
            scope: RightScopeV1::ObjectFamily(ObjectFamily::Voucher),
            max_uses: Some(1),
            delegation_allowed: false,
            attenuation_only: true,
        }]),
        required_signatures: BTreeSet::from([RequiredSignatureV1::Holder]),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::NonceAndRoot,
        conservation: ConservationContributionV1::ConditionalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn voucher_reject_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "voucher_reject".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        lifecycle_effect: LifecycleEffectV1::Refund,
        witness_requirements: BTreeSet::new(),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "voucher_reject_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "voucher_reject_policy_v1".to_string(),
        domain_name: "z00z.core.policies.voucher_reject.test.v1".to_string(),
        primary_family: ObjectFamily::Voucher,
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::new(),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::None,
        conservation: ConservationContributionV1::ConditionalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn voucher_issue_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "voucher_issue".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Voucher]),
        lifecycle_effect: LifecycleEffectV1::Offer,
        witness_requirements: BTreeSet::new(),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "voucher_issue_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "voucher_issue_policy_v1".to_string(),
        domain_name: "z00z.core.policies.voucher_issue.test.v1".to_string(),
        primary_family: ObjectFamily::Voucher,
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Voucher]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::new(),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::None,
        replay_protection: ReplayProtectionV1::None,
        conservation: ConservationContributionV1::ConditionalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn right_create_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "right_create".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        lifecycle_effect: LifecycleEffectV1::Grant,
        witness_requirements: BTreeSet::new(),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "right_create_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "right_create_policy_v1".to_string(),
        domain_name: "z00z.core.policies.right_create.test.v1".to_string(),
        primary_family: ObjectFamily::Right,
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::new(),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::None,
        replay_protection: ReplayProtectionV1::None,
        conservation: ConservationContributionV1::ZeroValueAuthority,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn issue_asset_payload(
    wallet_id: &PersistWalletId,
    seed: u8,
    prior_root: SettlementStateRoot,
) -> crate::db::OwnedAssetPayload {
    let serial_id = u32::from(seed % 10);
    let asset = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        serial_id,
        25,
    )
    .expect("issue asset");
    let mut asset_wire = z00z_core::AssetWire::from_asset(&asset);
    asset_wire.secret = None;
    let mut payload = crate::db::OwnedAssetPayload {
        version: crate::db::OwnedAssetPayload::VERSION,
        wallet_id: wallet_id.clone(),
        account_id: Some(seed as u128),
        asset_id: asset.asset_id(),
        asset_definition_id: asset.definition.id,
        asset_wire,
        status: crate::db::redb_store::OwnedAssetStatus::Spendable,
        source: crate::db::redb_store::OwnedAssetSource::Import,
        first_seen: None,
        last_updated_ms: 1_000 + u64::from(seed),
        scan_ref: None,
        receive_ref: None,
        spend_ref: None,
        confirmation_ref: Some(crate::db::redb_store::ConfirmRef {
            checkpoint_id_hex: Some(format!("issue-cp-{seed}")),
            state_root_hex: Some(hex::encode(prior_root.into_bytes())),
            evidence_id: Some(format!("issue-ev-{seed}")),
        }),
        labels: vec![format!("issue-{seed}")],
        policy: crate::db::redb_store::OwnedAssetPolicy {
            frozen: false,
            manual_review: false,
            quarantine_reason: None,
        },
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    let _ = payload
        .validate_invariants()
        .expect("issue asset invariants");
    payload
}

fn right_consume_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "right_consume".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        lifecycle_effect: LifecycleEffectV1::Use,
        witness_requirements: BTreeSet::new(),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "right_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "right_policy_v1".to_string(),
        domain_name: "z00z.core.policies.right_consume.test.v1".to_string(),
        primary_family: ObjectFamily::Right,
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::new(),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::None,
        replay_protection: ReplayProtectionV1::None,
        conservation: ConservationContributionV1::ZeroValueAuthority,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn right_delegate_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "right_delegate".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        lifecycle_effect: LifecycleEffectV1::Delegate,
        witness_requirements: BTreeSet::new(),
        receiver_must_accept: false,
        preserves_beneficiary: false,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "right_delegate_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "right_delegate_policy_v1".to_string(),
        domain_name: "z00z.core.policies.right_delegate.test.v1".to_string(),
        primary_family: ObjectFamily::Right,
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::new(),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::None,
        replay_protection: ReplayProtectionV1::None,
        conservation: ConservationContributionV1::ZeroValueAuthority,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn voucher_redeem_request(
    voucher: &mut crate::db::OwnedVoucherPayload,
    seed: u8,
) -> (
    crate::rpc::types::object::RuntimeObjectPackageRequest,
    [u8; 32],
) {
    let prior_root = object_test_root(seed);
    let next_root = object_test_root(seed.wrapping_add(1));
    let (policy, action_pool, action_id) = voucher_redeem_policy_contract();
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    voucher.voucher_leaf.policy_id = policy_hash;
    voucher.voucher_leaf.action_pool_id = action_pool_id;
    voucher.policy.policy_id = Some(policy_hash);
    if let Some(confirm) = voucher.confirmation_ref.as_mut() {
        confirm.state_root_hex = Some(hex::encode(prior_root.into_bytes()));
    }
    voucher.checksum = Some(voucher.compute_checksum());

    let path = SettlementPath::new(
        DefinitionId::new([seed; 32]),
        SerialId::new(u32::from(seed) + 1),
        voucher.terminal_id,
    );
    let asset_path = refund_asset_path(voucher, u32::from(seed) + 2);
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::RedeemFull),
        policy_hash,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(voucher.voucher_leaf.holder_commitment),
            expected_beneficiary: Some(voucher.voucher_leaf.beneficiary_commitment),
            expected_refund_target: Some(voucher.voucher_leaf.refund_target_commitment),
            acceptance_confirmed: true,
            ..VoucherActionCtx::default()
        }),
        vec![SettlementObjectDeltaV1::deleted(
            path,
            SettlementLeaf::Voucher(voucher.voucher_leaf.clone()),
            None,
        )],
        vec![SettlementObjectDeltaV1::created(
            asset_path,
            SettlementLeaf::Terminal(object_asset_leaf(asset_path)),
            Some(voucher.voucher_leaf.remaining_value),
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    (
        crate::rpc::types::object::RuntimeObjectPackageRequest {
            stable_id_hex: Some(hex::encode(voucher.terminal_id.as_bytes())),
            issue_asset_id_hex: None,
            issue_reserve_hex: None,
            create_terminal_id_hex: None,
            selected_action: Some(SettlementActionV1::Voucher(VoucherAction::RedeemFull)),
            action_label: "voucher_redeem_full".to_string(),
            policy_descriptor: policy,
            action_pool,
            required_rights: vec![RightWitnessRefV1 {
                right_policy: "kyc_v1".to_string(),
                witness_state: RightWitnessStateV1::Present,
            }],
            object_witnesses: ObjectWitnessBundleV1 {
                signatures: BTreeSet::from([RequiredSignatureV1::Holder]),
                attestation_labels: BTreeSet::new(),
                has_acceptance_proof: true,
                has_replay_nonce: true,
                has_prior_root_binding: true,
                has_disclosure_commitment: false,
            },
            delta_set,
        },
        action_id,
    )
}

fn voucher_reject_request(
    voucher: &mut crate::db::OwnedVoucherPayload,
    seed: u8,
) -> (
    crate::rpc::types::object::RuntimeObjectPackageRequest,
    [u8; 32],
) {
    let prior_root = object_test_root(seed);
    let next_root = object_test_root(seed.wrapping_add(1));
    let (policy, action_pool, action_id) = voucher_reject_policy_contract();
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    voucher.status = OwnedVoucherStatus::PendingAccept;
    voucher.voucher_leaf.lifecycle = VoucherLifecycleV1::PendingAcceptance;
    voucher.voucher_leaf.policy_id = policy_hash;
    voucher.voucher_leaf.action_pool_id = action_pool_id;
    voucher.policy.policy_id = Some(policy_hash);
    if let Some(confirm) = voucher.confirmation_ref.as_mut() {
        confirm.state_root_hex = Some(hex::encode(prior_root.into_bytes()));
    }
    voucher.checksum = Some(voucher.compute_checksum());

    let path = SettlementPath::new(
        DefinitionId::new([seed; 32]),
        SerialId::new(u32::from(seed) + 1),
        voucher.terminal_id,
    );
    let asset_path = refund_asset_path(voucher, u32::from(seed) + 2);
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Reject),
        policy_hash,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(voucher.voucher_leaf.holder_commitment),
            expected_beneficiary: Some(voucher.voucher_leaf.beneficiary_commitment),
            expected_refund_target: Some(voucher.voucher_leaf.refund_target_commitment),
            policy_allows_reject: true,
            ..VoucherActionCtx::default()
        }),
        vec![SettlementObjectDeltaV1::deleted(
            path,
            SettlementLeaf::Voucher(voucher.voucher_leaf.clone()),
            None,
        )],
        vec![SettlementObjectDeltaV1::created(
            asset_path,
            SettlementLeaf::Terminal(object_asset_leaf(asset_path)),
            Some(voucher.voucher_leaf.remaining_value),
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    (
        crate::rpc::types::object::RuntimeObjectPackageRequest {
            stable_id_hex: Some(hex::encode(voucher.terminal_id.as_bytes())),
            issue_asset_id_hex: None,
            issue_reserve_hex: None,
            create_terminal_id_hex: None,
            selected_action: None,
            action_label: "voucher_reject".to_string(),
            policy_descriptor: policy,
            action_pool,
            required_rights: Vec::new(),
            object_witnesses: ObjectWitnessBundleV1 {
                signatures: BTreeSet::new(),
                attestation_labels: BTreeSet::new(),
                has_acceptance_proof: false,
                has_replay_nonce: false,
                has_prior_root_binding: false,
                has_disclosure_commitment: false,
            },
            delta_set,
        },
        action_id,
    )
}

fn right_consume_request(
    right: &mut crate::db::OwnedRightPayload,
    seed: u8,
) -> (
    crate::rpc::types::object::RuntimeObjectPackageRequest,
    [u8; 32],
) {
    let prior_root = object_test_root(seed);
    let next_root = object_test_root(seed.wrapping_add(1));
    let (policy, action_pool, action_id) = right_consume_policy_contract();
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    right.right_leaf.transition_policy_id = policy_hash;
    right.policy.policy_id = Some(policy_hash);
    if let Some(confirm) = right.confirmation_ref.as_mut() {
        confirm.state_root_hex = Some(hex::encode(prior_root.into_bytes()));
    }
    right.checksum = Some(right.compute_checksum());

    let path = SettlementPath::new(
        DefinitionId::new([seed; 32]),
        SerialId::new(u32::from(seed) + 1),
        right.terminal_id,
    );
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Right(RightAction::Consume),
        policy_hash,
        None,
        vec![SettlementObjectDeltaV1::deleted(
            path,
            SettlementLeaf::Right(right.right_leaf),
            None,
        )],
        Vec::new(),
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    (
        crate::rpc::types::object::RuntimeObjectPackageRequest {
            stable_id_hex: Some(hex::encode(right.terminal_id.as_bytes())),
            issue_asset_id_hex: None,
            issue_reserve_hex: None,
            create_terminal_id_hex: None,
            selected_action: None,
            action_label: "right_consume".to_string(),
            policy_descriptor: policy,
            action_pool,
            required_rights: Vec::new(),
            object_witnesses: ObjectWitnessBundleV1 {
                signatures: BTreeSet::new(),
                attestation_labels: BTreeSet::new(),
                has_acceptance_proof: false,
                has_replay_nonce: false,
                has_prior_root_binding: false,
                has_disclosure_commitment: false,
            },
            delta_set,
        },
        action_id,
    )
}

fn empty_object_witnesses() -> ObjectWitnessBundleV1 {
    ObjectWitnessBundleV1 {
        signatures: BTreeSet::new(),
        attestation_labels: BTreeSet::new(),
        has_acceptance_proof: false,
        has_replay_nonce: false,
        has_prior_root_binding: false,
        has_disclosure_commitment: false,
    }
}

fn voucher_issue_asset_request(
    wallet_id: &PersistWalletId,
    seed: u8,
) -> (
    crate::db::OwnedAssetPayload,
    crate::db::OwnedVoucherPayload,
    crate::rpc::types::object::RuntimeObjectPackageRequest,
    [u8; 32],
) {
    let prior_root = object_test_root(seed);
    let next_root = object_test_root(seed.wrapping_add(1));
    let source = issue_asset_payload(wallet_id, seed, prior_root);
    let source_asset_id_hex = hex::encode(source.asset_id);
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), seed.wrapping_add(40));
    let (policy, action_pool, action_id) = voucher_issue_policy_contract();
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    voucher.voucher_leaf.policy_id = policy_hash;
    voucher.voucher_leaf.action_pool_id = action_pool_id;
    voucher.voucher_leaf.backing = z00z_storage::settlement::VoucherBackingRef::ConsumedAsset {
        definition_id: source.asset_definition_id,
        serial_id: source.asset_wire.serial_id,
    };
    voucher.policy.policy_id = Some(policy_hash);
    voucher.checksum = Some(voucher.compute_checksum());

    let source_path = SettlementPath::new(
        DefinitionId::new(source.asset_definition_id),
        SerialId::new(source.asset_wire.serial_id),
        TerminalId::new(source.asset_id),
    );
    let voucher_path = SettlementPath::new(
        DefinitionId::new(source.asset_definition_id),
        SerialId::new(source.asset_wire.serial_id),
        voucher.terminal_id,
    );
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Issue),
        policy_hash,
        Some(VoucherActionCtx::default()),
        vec![SettlementObjectDeltaV1::deleted(
            source_path,
            SettlementLeaf::Terminal(object_asset_leaf(source_path)),
            Some(voucher.voucher_leaf.face_value),
        )],
        vec![SettlementObjectDeltaV1::created(
            voucher_path,
            SettlementLeaf::Voucher(voucher.voucher_leaf.clone()),
            None,
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    (
        source,
        voucher.clone(),
        crate::rpc::types::object::RuntimeObjectPackageRequest {
            stable_id_hex: None,
            issue_asset_id_hex: Some(source_asset_id_hex),
            issue_reserve_hex: None,
            create_terminal_id_hex: None,
            selected_action: Some(SettlementActionV1::Voucher(VoucherAction::Issue)),
            action_label: "voucher_issue".to_string(),
            policy_descriptor: policy,
            action_pool,
            required_rights: Vec::new(),
            object_witnesses: empty_object_witnesses(),
            delta_set,
        },
        action_id,
    )
}

fn voucher_issue_reserve_request(
    wallet_id: &PersistWalletId,
    seed: u8,
) -> (
    crate::db::OwnedVoucherPayload,
    crate::rpc::types::object::RuntimeObjectPackageRequest,
    [u8; 32],
) {
    let prior_root = object_test_root(seed);
    let next_root = object_test_root(seed.wrapping_add(1));
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), seed.wrapping_add(50));
    let (policy, action_pool, action_id) = voucher_issue_policy_contract();
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    voucher.voucher_leaf.policy_id = policy_hash;
    voucher.voucher_leaf.action_pool_id = action_pool_id;
    voucher.policy.policy_id = Some(policy_hash);
    voucher.checksum = Some(voucher.compute_checksum());
    let reserve = match voucher.voucher_leaf.backing {
        z00z_storage::settlement::VoucherBackingRef::ReserveCommitment(backing)
        | z00z_storage::settlement::VoucherBackingRef::GenesisReserve(backing) => backing,
        z00z_storage::settlement::VoucherBackingRef::ConsumedAsset { .. } => unreachable!(),
    };
    let voucher_path = SettlementPath::new(
        DefinitionId::new(reserve),
        SerialId::new(u32::from(seed) + 1),
        voucher.terminal_id,
    );
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Issue),
        policy_hash,
        Some(VoucherActionCtx::default()),
        Vec::new(),
        vec![SettlementObjectDeltaV1::created(
            voucher_path,
            SettlementLeaf::Voucher(voucher.voucher_leaf.clone()),
            None,
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    (
        voucher.clone(),
        crate::rpc::types::object::RuntimeObjectPackageRequest {
            stable_id_hex: None,
            issue_asset_id_hex: None,
            issue_reserve_hex: Some(hex::encode(reserve)),
            create_terminal_id_hex: None,
            selected_action: Some(SettlementActionV1::Voucher(VoucherAction::Issue)),
            action_label: "voucher_issue".to_string(),
            policy_descriptor: policy,
            action_pool,
            required_rights: Vec::new(),
            object_witnesses: empty_object_witnesses(),
            delta_set,
        },
        action_id,
    )
}

fn right_create_request(
    wallet_id: &PersistWalletId,
    seed: u8,
) -> (
    crate::db::OwnedRightPayload,
    crate::rpc::types::object::RuntimeObjectPackageRequest,
    [u8; 32],
) {
    let prior_root = object_test_root(seed);
    let next_root = object_test_root(seed.wrapping_add(1));
    let mut right = test_owned_right_payload(wallet_id.clone(), seed.wrapping_add(60));
    let (policy, action_pool, action_id) = right_create_policy_contract();
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    right.right_leaf.transition_policy_id = policy_hash;
    right.policy.policy_id = Some(policy_hash);
    right.checksum = Some(right.compute_checksum());
    let path = SettlementPath::new(
        DefinitionId::new([seed.wrapping_add(1); 32]),
        SerialId::new(u32::from(seed) + 1),
        right.terminal_id,
    );
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Right(RightAction::Create),
        policy_hash,
        None,
        Vec::new(),
        vec![SettlementObjectDeltaV1::created(
            path,
            SettlementLeaf::Right(right.right_leaf),
            None,
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    (
        right,
        crate::rpc::types::object::RuntimeObjectPackageRequest {
            stable_id_hex: None,
            issue_asset_id_hex: None,
            issue_reserve_hex: None,
            create_terminal_id_hex: Some(hex::encode(path.terminal_id.into_bytes())),
            selected_action: Some(SettlementActionV1::Right(RightAction::Create)),
            action_label: "right_create".to_string(),
            policy_descriptor: policy,
            action_pool,
            required_rights: Vec::new(),
            object_witnesses: empty_object_witnesses(),
            delta_set,
        },
        action_id,
    )
}

fn right_delegate_request(
    right: &mut crate::db::OwnedRightPayload,
    seed: u8,
) -> (
    crate::rpc::types::object::RuntimeObjectPackageRequest,
    [u8; 32],
) {
    let prior_root = object_test_root(seed);
    let next_root = object_test_root(seed.wrapping_add(1));
    let (policy, action_pool, action_id) = right_delegate_policy_contract();
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    right.right_leaf.transition_policy_id = policy_hash;
    right.right_leaf.revocation_policy_id = [seed.wrapping_add(91); 32];
    right.right_leaf.challenge_policy_id = [seed.wrapping_add(92); 32];
    right.policy.policy_id = Some(policy_hash);
    if let Some(confirm) = right.confirmation_ref.as_mut() {
        confirm.state_root_hex = Some(hex::encode(prior_root.into_bytes()));
    }
    right.checksum = Some(right.compute_checksum());

    let mut delegated = right.right_leaf;
    delegated.holder_commitment = [seed.wrapping_add(70); 32];
    delegated.control_commitment = [seed.wrapping_add(71); 32];
    delegated.beneficiary_commitment = [seed.wrapping_add(72); 32];
    delegated.use_nonce = [seed.wrapping_add(73); 32];
    delegated.valid_from = right.right_leaf.valid_from + 1;
    delegated.valid_until = right.right_leaf.valid_until - 1;
    delegated.challenge_from = right.right_leaf.challenge_from + 1;
    delegated.challenge_until = right.right_leaf.challenge_until - 1;

    let path = SettlementPath::new(
        DefinitionId::new([seed; 32]),
        SerialId::new(u32::from(seed) + 1),
        right.terminal_id,
    );
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Right(RightAction::Transfer),
        policy_hash,
        None,
        Vec::new(),
        Vec::new(),
        vec![SettlementObjectDeltaV1::updated(
            path,
            SettlementLeaf::Right(right.right_leaf),
            SettlementLeaf::Right(delegated),
            None,
        )],
        None,
        prior_root,
        next_root,
    );

    (
        crate::rpc::types::object::RuntimeObjectPackageRequest {
            stable_id_hex: Some(hex::encode(right.terminal_id.as_bytes())),
            issue_asset_id_hex: None,
            issue_reserve_hex: None,
            create_terminal_id_hex: None,
            selected_action: Some(SettlementActionV1::Right(RightAction::Transfer)),
            action_label: "right_delegate".to_string(),
            policy_descriptor: policy,
            action_pool,
            required_rights: Vec::new(),
            object_witnesses: empty_object_witnesses(),
            delta_set,
        },
        action_id,
    )
}

#[tokio::test]
async fn test_asset_list_with_cursor() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    seed_assets(&rpc, &session).await;

    let page1 = rpc
        .list_assets(wallet_id.clone(), Some(1), None, None)
        .await
        .unwrap();
    assert_eq!(page1.items.len(), 1);
    assert!(page1.has_more);
    assert!(page1.next_cursor.is_some());

    let page2 = rpc
        .list_assets(wallet_id.clone(), Some(2), page1.next_cursor, None)
        .await
        .unwrap();
    assert_eq!(page2.items.len(), 2);
    assert!(page2.has_more);
    assert!(page2.next_cursor.is_some());

    let page3 = rpc
        .list_assets(wallet_id, Some(2), page2.next_cursor, None)
        .await
        .unwrap();
    assert_eq!(page3.items.len(), 1);
    assert!(!page3.has_more);
    assert!(page3.next_cursor.is_none());
}

#[tokio::test]
async fn test_asset_list_page_limit() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    seed_assets(&rpc, &session).await;

    let page = rpc
        .list_assets(wallet_id, Some(50), None, None)
        .await
        .unwrap();

    assert_eq!(page.items.len(), 4);
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());
}

#[tokio::test]
async fn test_asset_list_filter_excludes() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    seed_assets(&rpc, &session).await;

    let filter = RuntimeAssetListFilter {
        asset_class: None,
        min_balance: Some(u64::MAX),
    };

    let page = rpc
        .list_assets(wallet_id, Some(50), None, Some(filter))
        .await
        .unwrap();

    assert!(page.items.is_empty());
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());
}

#[tokio::test]
async fn test_asset_list_past_end() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    seed_assets(&rpc, &session).await;

    let page = rpc
        .list_assets(wallet_id, Some(50), Some("999".to_string()), None)
        .await
        .unwrap();

    assert!(page.items.is_empty());
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());
}

#[tokio::test]
async fn test_asset_list_class_min() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    seed_assets(&rpc, &session).await;

    let filter = RuntimeAssetListFilter {
        asset_class: Some(AssetClass::Token),
        min_balance: Some(50),
    };

    let resp = rpc
        .list_assets(wallet_id, None, None, Some(filter))
        .await
        .unwrap();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.items[0].definition.class, AssetClass::Token);
    assert!(resp.items[0].amount >= 50);
}

#[tokio::test]
async fn test_asset_list_cached_50ms() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time.clone());

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    seed_assets(&rpc, &session).await;

    let r1 = rpc
        .list_assets(wallet_id.clone(), None, None, None)
        .await
        .unwrap();
    let ids1: Vec<_> = r1.items.iter().map(|a| a.definition.id).collect();

    time.advance_by(Duration::from_millis(10));
    let r2 = rpc
        .list_assets(wallet_id.clone(), None, None, None)
        .await
        .unwrap();
    let ids2: Vec<_> = r2.items.iter().map(|a| a.definition.id).collect();
    assert_eq!(ids1, ids2);

    time.advance_by(Duration::from_millis(60));
    let r3 = rpc.list_assets(wallet_id, None, None, None).await.unwrap();
    let ids3: Vec<_> = r3.items.iter().map(|a| a.definition.id).collect();
    assert_eq!(ids1, ids3);
}

#[tokio::test]
async fn test_asset_list_bounded_evicts() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time.clone());

    for i in 0..(ASSET_LIST_CACHE_MAX_WALLETS + 10) {
        let wallet_id = PersistWalletId(format!("w{i}"));
        let _ = rpc.list_assets(wallet_id, None, None, None).await.unwrap();
        time.advance_by(Duration::from_millis(1));
    }

    let cache = rpc.asset_list_cache.read().await;
    assert!(cache.len() <= ASSET_LIST_CACHE_MAX_WALLETS);
    assert!(!cache.contains_key(&PersistWalletId("w0".to_string())));
}

#[tokio::test]
async fn test_asset_list_concurrent_requests() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc_obj = AssetRpcImpl::with_dependencies(time);
    let (wallet_id, session) = create_unlocked_session(&rpc_obj).await;
    seed_assets(&rpc_obj, &session).await;
    let rpc = Arc::new(rpc_obj);

    let mut set = tokio::task::JoinSet::new();

    for _ in 0..32 {
        let rpc = rpc.clone();
        let wallet_id = wallet_id.clone();
        set.spawn(async move { rpc.list_assets(wallet_id, None, None, None).await });
    }

    while let Some(res) = set.join_next().await {
        let resp = res.unwrap().unwrap();
        assert!(!resp.items.is_empty());
    }
}

#[tokio::test]
async fn test_asset_balance_cached_1s() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time.clone());

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let asset_id = cached_asset_ids(&rpc, &wallet_id, &session).await[0];

    let b1 = rpc
        .get_asset_balance(wallet_id.clone(), asset_id)
        .await
        .unwrap();
    let total1 = b1.total;

    time.advance_by(Duration::from_millis(10));
    let b2 = rpc
        .get_asset_balance(wallet_id.clone(), asset_id)
        .await
        .unwrap();
    assert_eq!(total1, b2.total);

    time.advance_by(Duration::from_millis(1_100));
    let b3 = rpc.get_asset_balance(wallet_id, asset_id).await.unwrap();
    assert_eq!(total1, b3.total);
}

#[tokio::test]
async fn test_asset_metadata_cached_24h() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time.clone());

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let asset_id = cached_asset_ids(&rpc, &wallet_id, &session).await[0];

    let m1 = rpc.get_asset_metadata(asset_id).await.unwrap();
    let name1 = m1.name.clone();
    let symbol1 = m1.asset.symbol.clone();

    time.advance_by(Duration::from_secs(1));
    let m2 = rpc.get_asset_metadata(asset_id).await.unwrap();
    let name2 = m2.name.clone();
    let symbol2 = m2.asset.symbol.clone();

    assert_eq!(name1, name2);
    assert_eq!(symbol1, symbol2);

    time.advance_by(Duration::from_millis(ASSET_METADATA_CACHE_TTL_MS + 1));
    let m3 = rpc.get_asset_metadata(asset_id).await.unwrap();
    let name3 = m3.name.clone();
    let symbol3 = m3.asset.symbol.clone();

    assert_eq!(name1, name3);
    assert_eq!(symbol1, symbol3);
}

#[tokio::test]
async fn test_asset_metadata_unknown_asset() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let err = rpc.get_asset_metadata([9u8; 32]).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_metadata_id_query() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let def_id = cached_definition_ids(&rpc, &wallet_id, &session).await[0];
    let asset_id = cached_asset_ids(&rpc, &wallet_id, &session).await[0];
    assert_ne!(def_id, asset_id);

    let err = rpc.get_asset_metadata(def_id).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_details_cached_asset() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let asset_id = cached_asset_ids(&rpc, &wallet_id, &session).await[0];

    let resp = rpc
        .get_asset_details(wallet_id.clone(), asset_id)
        .await
        .unwrap();

    assert_eq!(resp.asset.asset_id, asset_id);
    assert_eq!(resp.total_serials, resp.definition.serials);
    assert_eq!(resp.nominal_per_serial, resp.definition.nominal);
    assert_eq!(
        resp.total_supply,
        u64::from(resp.total_serials) * resp.nominal_per_serial
    );
}

#[tokio::test]
async fn test_asset_details_unknown_asset() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let wallet_id = PersistWalletId("w1".to_string());
    let err = rpc
        .get_asset_details(wallet_id, [4u8; 32])
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_details_id_query() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let def_id = cached_definition_ids(&rpc, &wallet_id, &session).await[0];
    let asset_id = cached_asset_ids(&rpc, &wallet_id, &session).await[0];
    assert_ne!(def_id, asset_id);

    let err = rpc.get_asset_details(wallet_id, def_id).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_balance_unknown_asset() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let wallet_id = PersistWalletId("w1".to_string());
    let asset_id: AssetId = [7u8; 32];

    let err = rpc
        .get_asset_balance(wallet_id, asset_id)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_balance_id_query() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let def_id = cached_definition_ids(&rpc, &wallet_id, &session).await[0];
    let asset_id = cached_asset_ids(&rpc, &wallet_id, &session).await[0];
    assert_ne!(def_id, asset_id);

    let err = rpc.get_asset_balance(wallet_id, def_id).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

fn valid_definition_wire() -> DefinitionWire {
    let definition = z00z_core::AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Test Asset".to_string(),
        "TST".to_string(),
        8,
        1,
        1,
        "test.local".to_string(),
        1,
        1,
        0,
        None,
    )
    .expect("valid test definition");

    DefinitionWire::from(&definition)
}

#[tokio::test]
async fn test_asset_add_invalid_definition() {
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut bad = valid_definition_wire();
    bad.serials = 0;

    let codec = JsonCodec;
    let asset_data = String::from_utf8(codec.serialize(&bad).unwrap()).unwrap();

    let err = rpc.add_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_add_invalid_asset() {
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let asset_data = r#"{\
            \"id\": [1,1,1],\
            \"class\": \"Coin\",\
            \"name\": \"Test Asset\",\
            \"symbol\": \"TST\",\
            \"decimals\": 8,\
            \"serials\": 1,\
            \"nominal\": 1,\
            \"domain_name\": \"test.local\",\
            \"version\": 1,\
            \"crypto_version\": 1,\
            \"policy_flags\": 0,\
            \"metadata\": null\
        }"#
    .to_string();

    let err = rpc.add_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_add_unknown_asset() {
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let asset_data = r#"{\
            \"id\": [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],\
            \"class\": \"NotARealClass\",\
            \"name\": \"Test Asset\",\
            \"symbol\": \"TST\",\
            \"decimals\": 8,\
            \"serials\": 1,\
            \"nominal\": 1,\
            \"domain_name\": \"test.local\",\
            \"version\": 1,\
            \"crypto_version\": 1,\
            \"policy_flags\": 0,\
            \"metadata\": null\
        }"#
    .to_string();

    let err = rpc.add_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_add_valid_definition() {
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let good = valid_definition_wire();

    let codec = JsonCodec;
    let asset_data = String::from_utf8(codec.serialize(&good).unwrap()).unwrap();

    let _ = rpc.add_asset(session, asset_data).await.unwrap();
}

#[tokio::test]
async fn test_asset_import_invalid_json() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let err = rpc
        .import_asset(session, "not json".to_string())
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_import_invalid_asset() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut bad = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );
    bad.serial_id = bad.definition.serials;

    let asset_data = encode_test_dto_json(&bad);

    let err = rpc.import_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_import_tampered_sig() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );

    let asset_data = encode_test_dto_json(&wire);

    let mut root = decode_json_value(&asset_data);
    let sig = root
        .get_mut("owner_signature")
        .expect("owner_signature present");
    assert!(flip_sig_byte(sig), "failed to tamper signature bytes");

    let tampered = encode_json_value(&root);
    let err = rpc.import_asset(session, tampered).await.unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "IMPORT_CRYPTO_VERIFY_FAILED");
}

#[tokio::test]
async fn test_asset_import_tampered_commitment() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );

    let asset_data = encode_test_dto_json(&wire);

    let mut root = decode_json_value(&asset_data);
    let commitment = root.get_mut("commitment").expect("commitment present");
    assert!(
        flip_sig_byte(commitment),
        "failed to tamper commitment bytes"
    );

    let tampered = encode_json_value(&root);
    let err = rpc.import_asset(session, tampered).await.unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "IMPORT_CRYPTO_VERIFY_FAILED");
}

#[tokio::test]
async fn test_asset_import_range_proof() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );
    let proof = wire.range_proof.as_mut().expect("range_proof present");
    proof[0] ^= 1;

    let asset_data = encode_test_dto_json(&wire);

    let err = rpc.import_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "IMPORT_CRYPTO_VERIFY_FAILED");
}

#[tokio::test]
async fn test_asset_import_valid_asset() {
    reset_claims();
    let rpc = mk_rpc();

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let good = mk_owned_stealth_wire(&rpc, &wallet_id, 1).await;

    let asset_data = encode_test_dto_json(&good);

    let _ = rpc.import_asset(session, asset_data).await.unwrap();
}

#[tokio::test]
async fn test_asset_import_owner_mismatch() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 12, 10)
            .expect("valid std asset"),
    );

    let asset_data = encode_test_dto_json(&wire);

    let err = rpc.import_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "IMPORT_OWNER_MISMATCH");
}

#[tokio::test]
async fn test_asset_import_rejects_secret() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );

    let asset_data = encode_dto_secret(&wire, [7u8; 32]);

    let err = rpc.import_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "IMPORT_SECRET_FIELD_FORBIDDEN");
}

#[tokio::test]
async fn test_asset_import_secret_null() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = AssetWire::from_asset(
        &z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("valid std asset"),
    );

    let asset_data = encode_secret_null(&wire);

    let err = rpc.import_asset(session, asset_data).await.unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "IMPORT_SECRET_FIELD_FORBIDDEN");
}

#[tokio::test]
async fn test_asset_import_bad_json() {
    reset_claims();
    let rpc = AssetRpcImpl::new();

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;
    let err = rpc
        .import_asset(session, "not-json".to_string())
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "IMPORT_MALFORMED_JSON");
}

#[tokio::test]
async fn test_asset_import_claim_conflict() {
    reset_claims();
    let rpc = mk_rpc();

    let (wallet_a, session_a) = create_unlocked_session(&rpc).await;
    let rpc_b = mk_rpc();
    let (_wallet_b, session_b) = create_unlocked_session(&rpc_b).await;

    let wire = mk_owned_stealth_wire(&rpc, &wallet_a, 2).await;
    let asset_data = encode_test_dto_json(&wire);

    let ok = rpc
        .import_asset(session_a, asset_data.clone())
        .await
        .expect("wallet A import");
    assert!(ok.status.success);

    let err = rpc_b.import_asset(session_b, asset_data).await.unwrap_err();
    assert!(err.code() == -32603 || err.code() == -32602);
}

#[tokio::test]
async fn test_asset_claim_reserve_conflict() {
    reset_claims();
    let rpc = AssetRpcImpl::new();
    let (wallet_a, _session_a) = create_unlocked_session(&rpc).await;
    let (wallet_b, _session_b) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &wallet_a, 12).await;
    let asset_id = wire.to_asset().expect("asset").asset_id();

    let (receipt_a, sig_a) = rpc.sign_claim(&wallet_a, asset_id).await.expect("sign A");
    let _res_a = rpc
        .claim_reserve(&wallet_a, &receipt_a, &sig_a)
        .expect("reserve A");

    let (receipt_b, sig_b) = rpc.sign_claim(&wallet_b, asset_id).await.expect("sign B");
    let err = rpc
        .claim_reserve(&wallet_b, &receipt_b, &sig_b)
        .expect_err("reserve B must conflict");

    assert_eq!(err.code(), -32603);
    assert_eq!(err.message(), "IMPORT_CLAIM_CONFLICT");
}

#[tokio::test]
async fn test_claim_scope_chain() {
    reset_claims();
    let rpc = AssetRpcImpl::new();
    let (wallet_id, _session) = create_unlocked_session_chain(&rpc, "testnet").await;

    std::env::set_var("Z00Z_WALLET_NETWORK", "p2p");
    std::env::set_var("Z00Z_WALLET_CHAIN", "mainnet");

    let (receipt, sig) = rpc.sign_claim(&wallet_id, [0xAB; 32]).await.expect("sign");
    verify_claim_receipt(&receipt, &sig).expect("verify");
    assert_eq!(receipt.claim_scope, claim_scope_hash("testnet"));
}

#[tokio::test]
async fn test_asset_import_claim_succeeds() {
    reset_claims();
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &wallet_id, 11).await;
    let asset_id = wire.clone().to_asset().expect("asset").asset_id();

    let asset_data = encode_test_dto_json(&wire);

    let resp = rpc.import_asset(session, asset_data).await.expect("import");
    assert!(resp.status.success);
    if resp.is_inserted {
        assert_eq!(resp.status.message, "asset_imported");
    } else {
        assert_eq!(resp.status.message, "asset_already_exists");
    }
    assert!(!rpc.is_claim_pending(&wallet_id, asset_id));
}

#[tokio::test]
async fn test_asset_send_policy_max() {
    let time = Arc::new(MockTimeProvider::default());
    let service = Arc::new(WalletService::with_dependencies(time.clone()));

    let rules = PolicyRules {
        max_tx_amount: Some(10),
        max_daily_amount: None,
        allowed_assets: None,
        allowed_recipients: None,
        require_confirmation: false,
        time_restrictions: None,
    };

    let mut settings = PersistWalletSettings {
        auto_lock_timeout: 0,
        default_fee: "0".to_string(),
        currency_display: "Z00Z".to_string(),
        policy_rules: None,
        created_at: 0,
        updated_at: 0,
    };
    settings.policy_rules = Some(rules);

    let rpc = AssetRpcImpl::with_dependencies_and_wallet_service(time, Arc::clone(&service));
    let (wallet_id, session) = create_unlocked_session(&rpc).await;

    service
        .set_wallet_settings(wallet_id.clone(), settings)
        .await
        .unwrap();

    let recipient = mk_recv_card_compact(&rpc, &wallet_id).await;
    let err = rpc
        .send_asset(session, [1u8; 32], recipient, 11)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_send_limits_10() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time.clone());

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let recipient = mk_recv_card_compact(&rpc, &wallet_id).await;

    for _ in 0..10 {
        let resp = rpc
            .send_asset(session.clone(), [1u8; 32], recipient.clone(), 1)
            .await
            .unwrap();
        assert!(!resp.tx_id.0.is_empty());
    }

    let err = rpc
        .send_asset(session.clone(), [1u8; 32], recipient.clone(), 1)
        .await
        .unwrap_err();

    assert_eq!(err.code(), SecurityErrorCode::RateLimitExceeded.code());

    time.advance_by(Duration::from_secs(60));
    let _ = rpc
        .send_asset(session, [1u8; 32], recipient, 1)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_asset_send_bad_recipient() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);
    let (_wallet_id, session) = create_unlocked_session(&rpc).await;

    let err = rpc
        .send_asset(session, [1u8; 32], "alice".to_string(), 1)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "SEND_RECIPIENT_INVALID");
}

#[tokio::test]
async fn test_asset_send_card_text() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);
    let (_wallet_id, session) = create_unlocked_session(&rpc).await;

    let err = rpc
        .send_asset(session, [1u8; 32], "z00z1compactdefaultpath".to_string(), 1)
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "SEND_RECIPIENT_INVALID");
}

#[tokio::test]
async fn test_asset_send_compact_card() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);
    let (wallet_id, session) = create_unlocked_session(&rpc).await;

    let keys = rpc
        .test_receiver_keys(&wallet_id)
        .await
        .expect("receiver keys");
    let card = keys.export_receiver_card().expect("receiver card");
    let raw_compact = encode_card_compact(&card);

    let err = rpc
        .send_asset(session, [1u8; 32], raw_compact, 1)
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "SEND_RECIPIENT_INVALID");
}

#[tokio::test]
async fn test_asset_send_from_request() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let recipient_card = mk_recv_card_compact(&rpc, &wallet_id).await;

    let _ = rpc
        .send_asset(session.clone(), [1u8; 32], recipient_card, 7)
        .await
        .unwrap();

    let recipient = mk_req_compact(&rpc, &wallet_id, 7).await;

    let resp = rpc
        .send_asset(session, [1u8; 32], recipient.clone(), 7)
        .await
        .unwrap();

    assert_eq!(resp.status, "stealth_submitted");
    assert_eq!(resp.recipient, recipient);
    assert_eq!(resp.amount, 7);
    assert_eq!(resp.owner_handle.len(), 64);
}

#[tokio::test]
async fn test_asset_send_first_seen() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let recipient = mk_req_compact(&rpc, &wallet_id, 3).await;

    let err = rpc
        .send_asset(session, [1u8; 32], recipient, 3)
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "SEND_TOFU_CONFIRM_REQUIRED");
}

#[test]
fn test_asset_send_uses_builder() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/rpc/asset_rpc_server_transfer.rs"
    ));
    let card_branch = source
        .split("SendTarget::Card(card) => {")
        .nth(1)
        .and_then(|tail| tail.split("SendTarget::Request(request) => {").next())
        .expect("card branch");
    let request_branch = source
        .split("SendTarget::Request(request) => {")
        .nth(1)
        .expect("request branch");

    assert!(card_branch.contains("build_card_stealth_output_validated("));
    assert!(!card_branch.contains("build_tx_stealth_output_validated("));
    assert!(request_branch.contains("build_tx_stealth_output_validated("));
    assert!(source.contains("BuildCheck {"));
    assert!(!source.contains("let stealth = build_tx_output_unchecked("));
}

#[tokio::test]
async fn test_asset_receive_scans_mine() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &wallet_id, 11).await;
    let asset_id = wire.clone().to_asset().expect("asset").asset_id();
    let keys = rpc
        .test_receiver_keys(&wallet_id)
        .await
        .expect("receiver keys");
    let scanner = StealthOutputScanner::from_keys(&keys);
    let pre = scanner.scan_leaf(&wire.clone().to_asset().expect("leaf"));
    assert!(matches!(pre, ScanResult::Mine { .. }));

    let asset_data = encode_test_dto_json(&wire);
    let _ = rpc
        .import_asset(session.clone(), asset_data)
        .await
        .expect("import");

    let recv = rpc.receive_asset(session, asset_id).await.expect("receive");

    assert_eq!(recv.asset.asset_id, asset_id);
    assert_eq!(recv.status, "RECEIVE_DETECTED");
    assert_eq!(recv.owner_handle, hex::encode(keys.owner_handle));
}

#[tokio::test]
async fn test_asset_receive_no_claim() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &wallet_id, 19).await;
    let asset_id = wire.clone().to_asset().expect("asset").asset_id();

    seed_wire_cache(&rpc, &session, wire).await;

    let recv = rpc
        .receive_asset(session.clone(), asset_id)
        .await
        .expect("receive");
    assert_eq!(recv.asset.asset_id, asset_id);
    assert_eq!(recv.status, "RECEIVE_DETECTED");

    let claimed = rpc
        .service
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claimed list");
    assert!(
        claimed.is_empty(),
        "receive must not claim or persist asset"
    );
}

#[tokio::test]
async fn test_asset_receive_not_mine() {
    let rpc = mk_rpc();
    let (owner_wallet, owner_session) = create_unlocked_session(&rpc).await;
    let (_other_wallet, other_session) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &owner_wallet, 34).await;
    let asset_id = wire.clone().to_asset().expect("asset").asset_id();
    seed_wire_cache(&rpc, &other_session, wire).await;
    let _ = owner_session;

    let err = rpc
        .receive_asset(other_session, asset_id)
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "RECEIVE_NOT_MINE");
}

#[tokio::test]
async fn test_asset_receive_tampered_tag16() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut wire = mk_owned_stealth_wire(&rpc, &wallet_id, 27).await;
    let asset_id = wire.clone().to_asset().expect("asset").asset_id();
    wire.tag16 = wire.tag16.map(|tag16| tag16 ^ 1);

    seed_wire_cache(&rpc, &session, wire).await;

    let err = rpc.receive_asset(session, asset_id).await.unwrap_err();
    assert_eq!(err.message(), "RECEIVE_INVALID_PROOF");
}

#[tokio::test]
async fn test_asset_receive_view_ok() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &wallet_id, 35).await;
    let asset_id = wire.clone().to_asset().expect("asset").asset_id();

    let asset_data = encode_test_dto_json(&wire);
    let _ = rpc
        .import_asset(session.clone(), asset_data)
        .await
        .expect("import");

    let _ = rpc
        .service
        .rotate_recv_view(&wallet_id)
        .await
        .expect("rotate");

    let recv = rpc.receive_asset(session, asset_id).await.expect("receive");
    assert_eq!(recv.asset.asset_id, asset_id);
    assert_eq!(recv.status, "RECEIVE_DETECTED");
}

#[tokio::test]
async fn test_asset_receive_status_sync() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &wallet_id, 41).await;
    let asset_id = wire.clone().to_asset().expect("asset").asset_id();

    seed_wire_cache(&rpc, &session, wire).await;

    let recv = rpc
        .receive_asset(session.clone(), asset_id)
        .await
        .expect("receive");

    assert_eq!(
        recv.status,
        WalletService::recv_code(crate::receiver::ReceiveStatus::Detected)
    );
}

#[tokio::test]
async fn test_asset_receive_definition_collision() {
    let rpc = mk_rpc();
    let (alice_id, _alice_session) = create_unlocked_session(&rpc).await;
    let (bob_id, bob_session) = create_unlocked_session(&rpc).await;

    let bob_src = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, 15)
        .expect("valid bob asset");
    let shared_def = Arc::clone(&bob_src.definition);
    let bob_wire = mk_owned_wire_from_asset(
        &rpc,
        &bob_id,
        bob_src,
        67,
        derive_wallet_mark_seed(&alice_id, 0x11),
        derive_wallet_mark_seed(&alice_id, 0x21),
    )
    .await;
    let bob_asset = dto_asset(&bob_wire);
    let exact_id = bob_asset.asset_id();

    let mut foreign_src =
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, 16)
            .expect("valid foreign asset");
    foreign_src.definition = shared_def;
    let foreign_wire = mk_owned_wire_from_asset(
        &rpc,
        &alice_id,
        foreign_src,
        68,
        derive_wallet_mark_seed(&bob_id, 0x31),
        derive_wallet_mark_seed(&bob_id, 0x41),
    )
    .await;

    seed_wire_cache(&rpc, &bob_session, foreign_wire).await;
    seed_wire_cache(&rpc, &bob_session, bob_wire).await;

    let recv = rpc
        .receive_asset(bob_session, exact_id)
        .await
        .expect("receive exact asset id");

    assert_eq!(recv.asset.asset_id, exact_id);
    assert_eq!(recv.status, "RECEIVE_DETECTED");
}

#[tokio::test]
async fn test_asset_receive_not_claim() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let base = build_recv_base(&rpc, &wallet_id, 47).await;
    let claim_id = base.asset.asset_id();

    assert_rpc_ok(&rpc, &session, base.wire.clone(), "RECEIVE_DETECTED").await;

    let claimed_none = rpc
        .service
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claimed list");
    assert!(
        claimed_none.is_empty(),
        "report-only receive must not claim asset"
    );

    let persisted = rpc
        .service
        .recv_route(&wallet_id, base.asset.clone(), ReceiveNext::PersistClaim)
        .await
        .expect("persist claim");
    assert!(
        persisted,
        "explicit claim boundary must persist exactly once"
    );

    let claimed = rpc
        .service
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claimed list");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].asset_id(), claim_id);
    assert_eq!(claimed[0].definition.id, base.wire.definition.id);

    let again = rpc
        .service
        .recv_route(&wallet_id, base.asset, ReceiveNext::PersistClaim)
        .await
        .expect("dedupe claim");
    assert!(!again, "second claim must not duplicate asset persistence");

    let claimed_again = rpc
        .service
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claimed list");
    assert_eq!(claimed_again.len(), 1);
    assert_eq!(claimed_again[0].asset_id(), claim_id);
    assert_eq!(claimed_again[0].definition.id, base.wire.definition.id);
}

#[tokio::test]
async fn test_ex3_recv_claim() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let base = build_recv_base(&rpc, &wallet_id, 59).await;
    let claim_id = base.asset.asset_id();

    seed_wire_cache(&rpc, &session, base.wire.clone()).await;

    let recv = rpc
        .receive_asset(session.clone(), claim_id)
        .await
        .expect("receive");
    let claimed_none = rpc
        .service
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claimed list");
    let phase_one = format!(
        "report status={} claimed={}",
        recv.status,
        claimed_none.len()
    );

    assert_eq!(recv.status, "RECEIVE_DETECTED");
    assert!(claimed_none.is_empty());
    assert!(phase_one.contains("RECEIVE_DETECTED"));
    assert!(phase_one.contains("claimed=0"));

    let persisted = rpc
        .service
        .recv_route(&wallet_id, base.asset.clone(), ReceiveNext::PersistClaim)
        .await
        .expect("persist claim");
    let claimed = rpc
        .service
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claimed list");
    let phase_two = format!("claim persisted={} claimed={}", persisted, claimed.len());

    assert!(persisted);
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].asset_id(), claim_id);
    assert_eq!(claimed[0].definition.id, base.wire.definition.id);
    assert!(phase_two.contains("persisted=true"));
    assert!(phase_two.contains("claimed=1"));
}

#[tokio::test]
async fn test_asset_receive_api_sync() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let base = build_recv_base(&rpc, &wallet_id, 49).await;

    let owned = rpc
        .service
        .scan_asset_report(&wallet_id, &base.asset)
        .await
        .expect("owned scan");
    assert_eq!(report_code(&owned), "RECEIVE_DETECTED");
    assert_rpc_ok(&rpc, &session, base.wire.clone(), "RECEIVE_DETECTED").await;

    let (other_wallet, other_session) = create_unlocked_session(&rpc).await;
    let not_mine = rpc
        .service
        .scan_asset_report(&other_wallet, &base.asset)
        .await
        .expect("foreign scan");
    assert_eq!(report_code(&not_mine), "RECEIVE_NOT_MINE");
    assert_rpc_msg(&rpc, &other_session, base.wire.clone(), "RECEIVE_NOT_MINE").await;

    let mut bad_wire = base.wire.clone();
    bad_wire.tag16 = bad_wire.tag16.map(|tag16| tag16 ^ 1);
    let bad_asset = wire_asset(&bad_wire);
    let invalid = rpc
        .service
        .scan_asset_report(&wallet_id, &bad_asset)
        .await
        .expect("proof-invalid scan");
    assert_eq!(report_code(&invalid), "RECEIVE_INVALID_PROOF");
    assert_eq!(invalid.reject, Some(ReceiveReject::InvalidProof));
    assert_rpc_bad(&rpc, &session, &wallet_id, bad_wire).await;

    let mut bad_tuple = base.asset.clone();
    bad_tuple.tag16 = None;
    let malformed = rpc.service.scan_asset_report(&wallet_id, &bad_tuple).await;
    assert!(matches!(malformed, Err(ReceiveReject::InvalidInput)));

    let mut bad_wire = base.wire.clone();
    bad_wire.tag16 = None;
    seed_wire_cache(&rpc, &session, bad_wire).await;
    let malformed_rpc = rpc
        .receive_asset(session, base.leaf.asset_id)
        .await
        .unwrap_err();
    assert_eq!(malformed_rpc.message(), "RECEIVE_INVALID_INPUT");
}

#[tokio::test]
async fn test_asset_receive_path_parity() {
    let rpc = mk_rpc();
    let (wallet_id, _session) = create_unlocked_session(&rpc).await;
    let base = build_recv_base(&rpc, &wallet_id, 53).await;
    let scanner = StealthOutputScanner::from_keys(&base.keys);

    let runtime_leaf = wire_decrypt_leaf(&z00z_core::AssetWire::from_asset(&base.asset))
        .expect("runtime decrypt leaf");
    assert_eq!(runtime_leaf.asset_id, base.leaf.asset_id);
    assert_ne!(base.asset.asset_id(), base.leaf.asset_id);
    assert_eq!(base.asset.serial_id, base.leaf.serial_id);
    assert_eq!(base.asset.r_pub, Some(base.leaf.r_pub));
    assert_eq!(base.asset.owner_tag, Some(base.leaf.owner_tag));
    assert_eq!(base.asset.tag16, Some(base.leaf.tag16));
    assert_eq!(base.asset.enc_pack, Some(base.leaf.enc_pack.clone()));
    assert_eq!(base.asset.commitment.as_bytes(), &base.leaf.c_amount);

    let pack = receiver_scan_leaf(&base.keys, &base.leaf)
        .expect("scan")
        .expect("owned pack");
    let canon = receiver_scan_report(&base.keys, &base.leaf).expect("report");
    assert_eq!(pack.value, base.asset.amount);
    assert_eq!(canon.status, ReceiveStatus::Detected);
    assert_eq!(canon.next, ReceiveNext::ReportOnly);
    assert!(!canon.next.should_persist());
    assert_eq!(canon.status.rpc_code(), "RECEIVE_DETECTED");

    let scan = scanner.scan_leaf(&base.asset);
    let runtime = scan.recv_report();
    assert_eq!(canon, runtime);
    assert_eq!(runtime.status.rpc_code(), "RECEIVE_DETECTED");

    let ScanResult::Mine { wallet_output } = scan else {
        panic!("expected Mine, got {scan:?}");
    };

    assert_eq!(pack, base.pack);
    assert_eq!(wallet_output.amount, pack.value);
    assert_eq!(wallet_output.asset_id, base.asset.asset_id());
    assert_ne!(wallet_output.asset_id, base.leaf.asset_id);
    assert_eq!(wallet_output.serial_id, base.leaf.serial_id);
    assert_eq!(wallet_output.r_pub, base.leaf.r_pub);
    assert_eq!(wallet_output.owner_tag, base.leaf.owner_tag);

    let claimed = rpc
        .service
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claimed list");
    assert!(claimed.is_empty(), "detect path must not persist claims");
}

#[tokio::test]
async fn test_asset_receive_journey() {
    let (rpc_a, dir, time) = mk_rpc_with_disk();
    let (alice_id, _alice_session) = create_unlocked_session(&rpc_a).await;
    let (bob_id, bob_session) = create_unlocked_session(&rpc_a).await;
    let amount = 10u64;
    let bob_card = mk_recv_card_compact(&rpc_a, &bob_id).await;
    assert!(!bob_card.is_empty());

    let asset_src =
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 0, amount)
            .expect("valid std asset");
    let wire = mk_owned_wire_from_asset(
        &rpc_a,
        &bob_id,
        asset_src,
        57,
        derive_wallet_mark_seed(&alice_id, 0x41),
        derive_wallet_mark_seed(&alice_id, 0x57),
    )
    .await;
    let asset = dto_asset(&wire);
    let claim_id = asset.asset_id();

    assert_ne!(alice_id, bob_id);

    seed_wire_cache(&rpc_a, &bob_session, wire.clone()).await;

    let recv = rpc_a
        .receive_asset(bob_session.clone(), claim_id)
        .await
        .expect("receive");
    assert_eq!(recv.asset.asset_id, claim_id);
    assert_eq!(recv.status, "RECEIVE_DETECTED");

    let claimed_none = rpc_a
        .service
        .list_claimed_assets(&bob_id)
        .await
        .expect("claimed list");
    assert!(
        claimed_none.is_empty(),
        "report-only receive must not claim asset"
    );

    let persisted = rpc_a
        .service
        .recv_route(&bob_id, asset.clone(), ReceiveNext::PersistClaim)
        .await
        .expect("persist claim");
    assert!(persisted, "explicit claim boundary must persist asset");

    let claimed_a = rpc_a
        .service
        .list_claimed_assets(&bob_id)
        .await
        .expect("claimed list");
    assert_eq!(claimed_a.len(), 1);
    assert_eq!(claimed_a[0].asset_id(), claim_id);
    assert_eq!(claimed_a[0].definition.id, wire.definition.id);

    rpc_a.service.lock_wallet(&bob_id).await.expect("lock");

    let mut service_b = WalletService::create_service_custom_output_directory(
        dir.path().to_path_buf(),
        time.clone(),
        SystemRngProvider,
    );
    service_b.set_sleeper(Arc::new(MockSleeper::new(time)));
    service_b
        .load_wallet(&bob_id, "test-password")
        .await
        .expect("load wallet");
    let password = SafePassword::from("test-password");
    let _bob_session_b = service_b
        .unlock_wallet_in_memory(&bob_id, &password)
        .await
        .expect("unlock");

    let claimed_b = service_b
        .list_claimed_assets(&bob_id)
        .await
        .expect("claimed list");
    assert_eq!(claimed_b.len(), 1);
    assert_eq!(claimed_b[0].asset_id(), claim_id);
    assert_eq!(claimed_b[0].definition.id, wire.definition.id);

    let persisted_again = service_b
        .recv_route(&bob_id, asset, ReceiveNext::PersistClaim)
        .await
        .expect("dedupe claim");
    assert!(!persisted_again, "restart must preserve dedupe state");

    let claimed_final = service_b
        .list_claimed_assets(&bob_id)
        .await
        .expect("claimed list");
    assert_eq!(claimed_final.len(), 1);
    assert_eq!(claimed_final[0].asset_id(), claim_id);
    assert_eq!(claimed_final[0].definition.id, wire.definition.id);
}

#[tokio::test]
async fn test_asset_receive_reject_proof() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let base = build_recv_base(&rpc, &wallet_id, 61).await;
    let scanner = StealthOutputScanner::from_keys(&base.keys);

    let mut drift = base.leaf.clone();
    drift.asset_id[0] ^= 1;
    assert_bad_leaf(
        &rpc, &session, &wallet_id, &scanner, &base.keys, &base.wire, &drift,
    )
    .await;

    let mut bad_tag = base.leaf.clone();
    bad_tag.tag16 ^= 1;
    assert_bad_leaf(
        &rpc, &session, &wallet_id, &scanner, &base.keys, &base.wire, &bad_tag,
    )
    .await;

    let bad_enc = bad_enc(&base.leaf);
    assert_bad_leaf(
        &rpc, &session, &wallet_id, &scanner, &base.keys, &base.wire, &bad_enc,
    )
    .await;
}

#[tokio::test]
async fn test_asset_receive_reject_types() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let base = build_recv_base(&rpc, &wallet_id, 61).await;
    let scanner = StealthOutputScanner::from_keys(&base.keys);

    let short = short_leaf(&base.keys, &base.leaf);
    assert_parse_bad(
        &rpc, &session, &wallet_id, &scanner, &base.keys, &base.wire, &short,
    )
    .await;

    let bad_open = bad_open(&base.keys, &base.leaf);
    assert_open_bad(
        &rpc, &session, &wallet_id, &scanner, &base.keys, &base.wire, &bad_open,
    )
    .await;

    let mut bad_tuple = base.asset.clone();
    bad_tuple.tag16 = None;
    let tuple = rpc.service.scan_asset_report(&wallet_id, &bad_tuple).await;
    assert!(matches!(tuple, Err(ReceiveReject::InvalidInput)));
    claims_empty(&rpc, &wallet_id).await;
}

#[tokio::test]
async fn test_asset_definition_id_query() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let base = build_recv_base(&rpc, &wallet_id, 73).await;

    seed_wire_cache(&rpc, &session, base.wire.clone()).await;

    let err = rpc
        .receive_asset(session, base.wire.definition.id)
        .await
        .unwrap_err();
    assert_eq!(err.message(), "RECEIVE_INVALID_INPUT");
}

#[tokio::test]
async fn test_asset_receive_without_cache() {
    let rpc = mk_rpc();
    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let wire = mk_owned_stealth_wire(&rpc, &wallet_id, 74).await;
    let asset_data = encode_test_dto_json(&wire);

    rpc.import_asset(session.clone(), asset_data)
        .await
        .expect("import");

    let err = rpc
        .receive_asset(session, wire.definition.id)
        .await
        .unwrap_err();
    assert_eq!(err.message(), "RECEIVE_INVALID_INPUT");
}

#[test]
fn test_asset_receive_err_map() {
    let malformed = AssetRpcImpl::recv_err(ReceiveReject::InvalidInput);
    assert_eq!(malformed.code(), -32602);
    assert_eq!(malformed.message(), "RECEIVE_INVALID_INPUT");

    let internal = AssetRpcImpl::recv_err(ReceiveReject::RuntimeFail);
    assert_eq!(internal.code(), -32603);
    assert_eq!(internal.message(), "RECEIVE_INVALID_PROOF");
}

#[tokio::test]
async fn test_asset_merge_assets_total() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let coin_ids = cached_asset_ids_by_class(&rpc, &wallet_id, &session, AssetClass::Coin).await;
    assert!(coin_ids.len() >= 2);
    let resp = rpc
        .merge_assets(session, coin_ids[..2].to_vec())
        .await
        .unwrap();

    assert_eq!(resp.merged_count, 2);
    assert_eq!(resp.total_amount, 25);
    assert!(resp
        .tx_id
        .as_ref()
        .is_some_and(|tx_id| tx_id.0.starts_with("tx_") && !tx_id.0.contains("stub_tx_")));
}

#[tokio::test]
async fn test_asset_merge_rejects_mixed_definitions() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let coin_id = cached_asset_id_by_class(&rpc, &wallet_id, &session, AssetClass::Coin).await;
    let token_id = cached_asset_id_by_class(&rpc, &wallet_id, &session, AssetClass::Token).await;

    let err = rpc
        .merge_assets(session, vec![coin_id, token_id])
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(
        err.message(),
        "Invalid asset_ids: merged assets must share one definition"
    );
}

#[tokio::test]
async fn test_asset_merge_insufficient_asset() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let ids = cached_asset_ids(&rpc, &wallet_id, &session).await;

    let err = rpc.merge_assets(session, vec![ids[0]]).await.unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_split_sum_equals() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let token_id = cached_asset_id_by_class(&rpc, &wallet_id, &session, AssetClass::Token).await;

    let err = rpc
        .split_asset(session.clone(), token_id, vec![40, 50])
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);

    let ok = rpc
        .split_asset(session, token_id, vec![40, 60])
        .await
        .unwrap();
    assert_eq!(
        ok.splits.iter().map(|s| s.amount).collect::<Vec<_>>(),
        vec![40, 60]
    );
    assert_eq!(ok.splits.len(), 2);
    assert!(ok
        .tx_id
        .as_ref()
        .is_some_and(|tx_id| tx_id.0.starts_with("tx_") && !tx_id.0.contains("stub_tx_")));
}

#[tokio::test]
async fn test_asset_stake_echo_id() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let token_id = cached_asset_id_by_class(&rpc, &wallet_id, &session, AssetClass::Token).await;

    let resp = rpc.stake_assets(session, token_id, 10).await.unwrap();
    assert!(resp.stake_id.starts_with("stake_"));
    assert_eq!(resp.asset.asset_id, token_id);
    assert_eq!(resp.amount, 10);
    assert!(resp.end_time >= resp.start_time);
}

#[tokio::test]
async fn test_asset_unstake_unknown() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (_wallet_id, session) = create_unlocked_session(&rpc).await;

    let err = rpc
        .unstake_assets(session, "missing".to_string())
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_asset_unstake_roundtrip_surface() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let token_id = cached_asset_id_by_class(&rpc, &wallet_id, &session, AssetClass::Token).await;

    let stake = rpc
        .stake_assets(session.clone(), token_id, 10)
        .await
        .unwrap();

    let stake_id = stake.stake_id.clone();
    let ok = rpc.unstake_assets(session, stake_id.clone()).await.unwrap();
    assert_eq!(ok.stake_id, stake_id);
    assert_eq!(ok.asset.asset_id, token_id);
    assert_eq!(ok.amount, 10);
}

#[tokio::test]
async fn test_asset_swap_inputs_live_tx() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let ids = cached_asset_ids(&rpc, &wallet_id, &session).await;
    let from_id = ids[1];
    let to_id = ids[0];

    let err = rpc
        .swap_assets(session.clone(), from_id, from_id, 1)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);

    let ok = rpc.swap_assets(session, from_id, to_id, 1).await.unwrap();
    assert!(ok.tx_id.0.starts_with("tx_"));
    assert!(!ok.tx_id.0.contains("stub_tx_"));
    assert_eq!(ok.from_amount, 1);
    assert_eq!(ok.to_amount, 1);
}

#[tokio::test]
async fn test_asset_swap_target_asset() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let ids = cached_asset_ids(&rpc, &wallet_id, &session).await;
    let from_id = ids[0];

    let err = rpc
        .swap_assets(session, from_id, [0x99; 32], 1)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "Unknown to_asset_id for this wallet");
}

#[tokio::test]
async fn test_object_rpc_lists_inventory() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, _session) = create_unlocked_session(&rpc).await;
    let voucher = test_owned_voucher_payload(wallet_id.clone(), 61);
    let right = test_owned_right_payload(wallet_id.clone(), 62);

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher.clone()))
        .await
        .expect("voucher insert must succeed");
    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Right(right.clone()))
        .await
        .expect("right insert must succeed");

    let page1 = crate::rpc::methods::ObjectRpcServer::list_objects(
        &rpc,
        wallet_id.clone(),
        Some(1),
        None,
        None,
    )
    .await
    .expect("first object page");
    assert_eq!(page1.items.len(), 1);
    assert!(page1.has_more);
    assert!(page1.next_cursor.is_some());

    let page2 = crate::rpc::methods::ObjectRpcServer::list_objects(
        &rpc,
        wallet_id.clone(),
        Some(10),
        page1.next_cursor.clone(),
        None,
    )
    .await
    .expect("second object page");
    assert_eq!(page2.items.len(), 1);
    assert!(!page2.has_more);

    let mut families = page1
        .items
        .iter()
        .chain(page2.items.iter())
        .map(|item| item.family)
        .collect::<Vec<_>>();
    families.sort_by_key(|family| match family {
        OwnedObjectFamily::Asset => 0u8,
        OwnedObjectFamily::Voucher => 1u8,
        OwnedObjectFamily::Right => 2u8,
    });
    assert_eq!(
        families,
        vec![OwnedObjectFamily::Voucher, OwnedObjectFamily::Right]
    );

    let voucher_rows = crate::rpc::methods::ObjectRpcServer::list_vouchers(
        &rpc,
        wallet_id.clone(),
        Some(10),
        None,
        Some(OwnedVoucherStatus::Redeemable),
    )
    .await
    .expect("voucher inventory");
    assert_eq!(voucher_rows.items.len(), 1);
    assert_eq!(
        voucher_rows.items[0].stable_id_hex,
        hex::encode(voucher.terminal_id.as_bytes())
    );
    assert!(voucher_rows.items[0].voucher.is_some());
    assert!(voucher_rows.items[0].policy.is_some());

    let right_rows = crate::rpc::methods::ObjectRpcServer::list_rights(
        &rpc,
        wallet_id.clone(),
        Some(10),
        None,
        Some(OwnedRightStatus::Granted),
    )
    .await
    .expect("right inventory");
    assert_eq!(right_rows.items.len(), 1);
    assert_eq!(
        right_rows.items[0].stable_id_hex,
        hex::encode(right.terminal_id.as_bytes())
    );
    assert!(right_rows.items[0].right.is_some());
    assert!(right_rows.items[0].policy.is_some());

    let cash_assets = rpc
        .list_assets(wallet_id, Some(10), None, None)
        .await
        .expect("cash asset inventory");
    assert!(
        cash_assets.items.is_empty(),
        "typed objects must not bleed into asset cash inventory"
    );
}

#[tokio::test]
async fn test_asset_rpc_rejects_ids() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let voucher = test_owned_voucher_payload(wallet_id.clone(), 71);
    let right = test_owned_right_payload(wallet_id.clone(), 72);

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher.clone()))
        .await
        .expect("voucher insert must succeed");
    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Right(right.clone()))
        .await
        .expect("right insert must succeed");

    let voucher_err = crate::rpc::methods::AssetRpcServer::get_asset_balance(
        &rpc,
        wallet_id.clone(),
        voucher.terminal_id.into_bytes(),
    )
    .await
    .unwrap_err();
    assert_eq!(voucher_err.code(), -32602);
    assert!(voucher_err.message().contains("voucher inventory"));

    let recipient = mk_recv_card_compact(&rpc, &wallet_id).await;
    let right_err = crate::rpc::methods::AssetRpcServer::send_asset(
        &rpc,
        session,
        right.terminal_id.into_bytes(),
        recipient,
        1,
    )
    .await
    .unwrap_err();
    assert_eq!(right_err.code(), -32602);
    assert!(right_err.message().contains("right inventory"));
}

#[tokio::test]
async fn test_object_issue_asset() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (source, voucher, request, action_id) = voucher_issue_asset_request(&wallet_id, 88);

    rpc.service
        .put_test_asset_payload(&wallet_id, source)
        .await
        .expect("asset insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(preview.verdict.reject, None);
    assert_eq!(preview.family, OwnedObjectFamily::Voucher);
    assert_eq!(
        preview.stable_id_hex,
        hex::encode(voucher.terminal_id.as_bytes())
    );
    assert_eq!(
        preview.package.selected_action,
        SettlementActionV1::Voucher(VoucherAction::Issue)
    );
    assert_eq!(preview.package.selected_action_id, action_id);

    let build = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .expect("build must succeed");
    assert_eq!(build.family, OwnedObjectFamily::Voucher);
    assert_eq!(
        build.stable_id_hex,
        hex::encode(voucher.terminal_id.as_bytes())
    );
    assert_eq!(build.package.selected_action_id, action_id);
}

#[tokio::test]
async fn test_object_issue_reserve() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (voucher, request, action_id) = voucher_issue_reserve_request(&wallet_id, 89);

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(preview.verdict.reject, None);
    assert_eq!(preview.family, OwnedObjectFamily::Voucher);
    assert_eq!(
        preview.stable_id_hex,
        hex::encode(voucher.terminal_id.as_bytes())
    );
    assert_eq!(preview.package.selected_action_id, action_id);

    let build = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .expect("build must succeed");
    assert_eq!(
        build.stable_id_hex,
        hex::encode(voucher.terminal_id.as_bytes())
    );
    assert_eq!(build.package.selected_action_id, action_id);
}

#[tokio::test]
async fn test_object_issue_mix() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (_, _, mut request, _) = voucher_issue_asset_request(&wallet_id, 90);
    request.issue_reserve_hex = Some(hex::encode([0x90; 32]));

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_TARGET_AMBIGUOUS");
}

#[tokio::test]
async fn test_object_issue_missing() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (_, mut request, _) = voucher_issue_reserve_request(&wallet_id, 91);
    request.issue_reserve_hex = None;

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_ISSUE_SOURCE_REQUIRED");
}

#[tokio::test]
async fn test_object_issue_stale() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (mut source, _, request, _) = voucher_issue_asset_request(&wallet_id, 92);
    if let Some(confirm) = source.confirmation_ref.as_mut() {
        confirm.state_root_hex = Some(hex::encode(object_test_root(12).into_bytes()));
    }
    source.checksum = Some(source.compute_checksum());

    rpc.service
        .put_test_asset_payload(&wallet_id, source)
        .await
        .expect("asset insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(preview.verdict.reject, Some(ObjectRejectCode::StaleRoot));

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_STALE_ROOT");
}

#[tokio::test]
async fn test_object_issue_reserve_bad() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (_, mut request, _) = voucher_issue_reserve_request(&wallet_id, 93);
    request.issue_reserve_hex = Some(hex::encode([0x94; 32]));

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(
        preview.verdict.reject,
        Some(ObjectRejectCode::InvalidBacking)
    );

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_INVALID_BACKING");
}

#[tokio::test]
async fn test_object_create_right() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (right, request, action_id) = right_create_request(&wallet_id, 94);

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(preview.verdict.reject, None);
    assert_eq!(preview.family, OwnedObjectFamily::Right);
    assert_eq!(
        preview.stable_id_hex,
        hex::encode(right.terminal_id.as_bytes())
    );
    assert_eq!(
        preview.package.selected_action,
        SettlementActionV1::Right(RightAction::Create)
    );
    assert_eq!(preview.package.selected_action_id, action_id);

    let build = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .expect("build must succeed");
    assert_eq!(build.family, OwnedObjectFamily::Right);
    assert_eq!(
        build.stable_id_hex,
        hex::encode(right.terminal_id.as_bytes())
    );
    assert_eq!(build.package.selected_action_id, action_id);
}

#[tokio::test]
async fn test_object_create_missing() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let (_, mut request, _) = right_create_request(&wallet_id, 95);
    request.create_terminal_id_hex = None;

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_CREATE_CONTEXT_REQUIRED");
}

#[tokio::test]
async fn test_object_delegate_rejects_widening() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut right = test_owned_right_payload(wallet_id.clone(), 96);
    let (mut request, _) = right_delegate_request(&mut right, 96);
    let SettlementLeaf::Right(next) = request.delta_set.updated_objects[0]
        .next_leaf
        .as_mut()
        .expect("delegated right")
    else {
        panic!("next leaf must be right");
    };
    next.valid_until = right.right_leaf.valid_until + 1;

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Right(right))
        .await
        .expect("right insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(
        preview.verdict.reject,
        Some(ObjectRejectCode::WrongFamilyProof)
    );

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_WRONG_FAMILY_PROOF");
}

#[tokio::test]
async fn test_object_delegate_scope_drift() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut right = test_owned_right_payload(wallet_id.clone(), 97);
    let (mut request, _) = right_delegate_request(&mut right, 97);
    let SettlementLeaf::Right(next) = request.delta_set.updated_objects[0]
        .next_leaf
        .as_mut()
        .expect("delegated right")
    else {
        panic!("next leaf must be right");
    };
    next.provider_scope = [0xF7; 32];

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Right(right))
        .await
        .expect("right insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(
        preview.verdict.reject,
        Some(ObjectRejectCode::WrongFamilyProof)
    );

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_WRONG_FAMILY_PROOF");
}

#[tokio::test]
async fn test_object_builds_redeem_voucher() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 81);
    let (request, action_id) = voucher_redeem_request(&mut voucher, 81);

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(preview.verdict.reject, None);
    assert_eq!(
        preview.package.selected_action,
        SettlementActionV1::Voucher(VoucherAction::RedeemFull)
    );
    assert_eq!(preview.package.selected_action_id, action_id);
    assert_eq!(preview.family, OwnedObjectFamily::Voucher);

    let build = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .expect("build must succeed");
    assert_eq!(
        build.package.selected_action,
        SettlementActionV1::Voucher(VoucherAction::RedeemFull)
    );
    assert_eq!(build.package.selected_action_id, action_id);
}

#[tokio::test]
async fn test_build_rejects_missing_right() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 82);
    let (mut request, _) = voucher_redeem_request(&mut voucher, 82);
    request.required_rights = vec![RightWitnessRefV1 {
        right_policy: "kyc_v1".to_string(),
        witness_state: RightWitnessStateV1::Missing,
    }];

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(preview.verdict.reject, Some(ObjectRejectCode::MissingRight));

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_MISSING_RIGHT");
}

#[tokio::test]
async fn test_build_rejects_value_mismatch() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 84);
    let (mut request, _) = voucher_redeem_request(&mut voucher, 84);
    request.delta_set.created_objects[0].declared_value_units =
        Some(voucher.voucher_leaf.remaining_value.saturating_sub(1));

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(
        preview.verdict.reject,
        Some(ObjectRejectCode::ResidualMismatch)
    );

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_RESIDUAL_MISMATCH");
}

#[tokio::test]
async fn test_build_rejects_bad_fee() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 85);
    let (mut request, _) = voucher_redeem_request(&mut voucher, 85);
    let support_ref = Some([0xE5; 32]);
    request.delta_set.fee_envelope = Some(z00z_storage::settlement::FeeEnvelope {
        version: 1,
        payer_commitment: [0xA1; 32],
        sponsor_commitment: [0xA2; 32],
        budget_units: 3,
        budget_commitment: z00z_storage::settlement::FeeEnvelope::budget_bind(4, support_ref),
        domain_id: [0xA3; 32],
        expires_at: 100,
        nonce: [0xA4; 32],
        transition_id: [0xA5; 32],
        replay_key: [0xA6; 32],
        support_ref,
        failure_policy_id: [0xA7; 32],
    });

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(preview.verdict.reject, Some(ObjectRejectCode::FeeBoundary));

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_FEE_BOUNDARY");
}

#[tokio::test]
async fn test_build_rejects_refund_target() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 86);
    let (mut request, _) = voucher_reject_request(&mut voucher, 86);
    request.selected_action = Some(SettlementActionV1::Voucher(VoucherAction::Reject));
    request.delta_set.created_objects[0].path = SettlementPath::new(
        request.delta_set.created_objects[0].path.definition_id,
        request.delta_set.created_objects[0].path.serial_id,
        TerminalId::new([0xF6; 32]),
    );
    request.delta_set.created_objects[0].next_leaf = Some(SettlementLeaf::Terminal(
        object_asset_leaf(request.delta_set.created_objects[0].path),
    ));

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(
        preview.verdict.reject,
        Some(ObjectRejectCode::WrongFamilyProof)
    );

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_WRONG_FAMILY_PROOF");
}

#[tokio::test]
async fn test_build_rejects_refund_ctx() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 87);
    voucher.voucher_leaf.backing = z00z_storage::settlement::VoucherBackingRef::ConsumedAsset {
        definition_id: [0x87; 32],
        serial_id: 9,
    };
    let (mut request, _) = voucher_reject_request(&mut voucher, 87);
    request.selected_action = Some(SettlementActionV1::Voucher(VoucherAction::Reject));
    request.delta_set.created_objects[0].path = SettlementPath::new(
        DefinitionId::new([0x88; 32]),
        SerialId::new(9),
        TerminalId::new(voucher.voucher_leaf.refund_target_commitment),
    );
    request.delta_set.created_objects[0].next_leaf = Some(SettlementLeaf::Terminal(
        object_asset_leaf(request.delta_set.created_objects[0].path),
    ));

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(
        preview.verdict.reject,
        Some(ObjectRejectCode::WrongFamilyProof)
    );

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_WRONG_FAMILY_PROOF");
}

#[tokio::test]
async fn test_build_rejects_reserve_ctx() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 88);
    let (mut request, _) = voucher_reject_request(&mut voucher, 88);
    request.selected_action = Some(SettlementActionV1::Voucher(VoucherAction::Reject));
    request.delta_set.created_objects[0].path = SettlementPath::new(
        DefinitionId::new([0x89; 32]),
        SerialId::new(90),
        TerminalId::new(voucher.voucher_leaf.refund_target_commitment),
    );
    request.delta_set.created_objects[0].next_leaf = Some(SettlementLeaf::Terminal(
        object_asset_leaf(request.delta_set.created_objects[0].path),
    ));

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let preview = crate::rpc::methods::ObjectRpcServer::preview_package(
        &rpc,
        session.clone(),
        request.clone(),
    )
    .await
    .expect("preview must succeed");
    assert_eq!(
        preview.verdict.reject,
        Some(ObjectRejectCode::InvalidBacking)
    );

    let err = crate::rpc::methods::ObjectRpcServer::build_package(&rpc, session, request)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32602);
    assert_eq!(err.message(), "OBJECT_INVALID_BACKING");
}

#[tokio::test]
async fn test_reject_wrapper_maps_refund() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut voucher = test_owned_voucher_payload(wallet_id.clone(), 83);
    let (request, action_id) = voucher_reject_request(&mut voucher, 83);

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher))
        .await
        .expect("voucher insert must succeed");

    let build = crate::rpc::methods::ObjectRpcServer::reject_voucher(&rpc, session, request)
        .await
        .expect("reject wrapper must succeed");
    assert_eq!(
        build.package.selected_action,
        SettlementActionV1::Voucher(VoucherAction::Reject)
    );
    assert_eq!(build.package.selected_action_id, action_id);
}

#[tokio::test]
async fn test_consume_right_builds_package() {
    let time = Arc::new(MockTimeProvider::default());
    let rpc = AssetRpcImpl::with_dependencies(time);

    let (wallet_id, session) = create_unlocked_session(&rpc).await;
    let mut right = test_owned_right_payload(wallet_id.clone(), 84);
    let (request, action_id) = right_consume_request(&mut right, 84);

    rpc.service
        .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Right(right))
        .await
        .expect("right insert must succeed");

    let build = crate::rpc::methods::ObjectRpcServer::consume_right(&rpc, session, request)
        .await
        .expect("consume wrapper must succeed");
    assert_eq!(
        build.package.selected_action,
        SettlementActionV1::Right(RightAction::Consume)
    );
    assert_eq!(build.package.selected_action_id, action_id);
    assert_eq!(build.family, OwnedObjectFamily::Right);
}
