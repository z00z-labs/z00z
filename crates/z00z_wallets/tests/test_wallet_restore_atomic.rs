#![cfg(not(target_arch = "wasm32"))]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::{
    io::{path_exists, read_file, write_file},
    rng::SystemRngProvider,
    time::MockTimeProvider,
};
use z00z_wallets::{
    backup::{decode_tx_history_jsonl, encode_tx_history_jsonl, BackupExporterImpl},
    db::{BackupManifestPayload, WalletProfilePayload},
    domains::hashing::compute_wallet_file_id,
    persistence::{TxRecord, TxStatus},
    rpc::types::{common::PersistWalletId, wallet::PersistWalletSettings},
    services::{AppService, WalletService},
    wallet::{
        persistence::{
            PasswordVerifierState, ReceiverDeriverState, WalletExportIdentity, WalletExportPack,
        },
        WalletState,
    },
};

#[path = "test_inc/test_wallet_env.inc"]
mod test_common;

const RESTORE_ATOMIC_FAILPOINT_ENV: &str = "Z00Z_TEST_RESTORE_ATOMIC_FAILPOINT";
const TEST_PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn restore_failpoint_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct RestoreFailpointGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
    prev_value: Option<OsString>,
}

impl RestoreFailpointGuard {
    fn set(failpoint: &str) -> Self {
        let guard = restore_failpoint_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let prev_value = std::env::var_os(RESTORE_ATOMIC_FAILPOINT_ENV);
        std::env::set_var(RESTORE_ATOMIC_FAILPOINT_ENV, failpoint);
        Self {
            _guard: guard,
            prev_value,
        }
    }
}

impl Drop for RestoreFailpointGuard {
    fn drop(&mut self) {
        match &self.prev_value {
            Some(value) => std::env::set_var(RESTORE_ATOMIC_FAILPOINT_ENV, value),
            None => std::env::remove_var(RESTORE_ATOMIC_FAILPOINT_ENV),
        }
    }
}

fn wallet_stem(wallet_id: &PersistWalletId) -> String {
    let hash = compute_wallet_file_id(&wallet_id.0);
    hex::encode(&hash[..8])
}

fn wlt_path(root: &Path, wallet_id: &PersistWalletId) -> PathBuf {
    root.join(format!("wallet_{}.wlt", wallet_stem(wallet_id)))
}

fn history_path(root: &Path, wallet_id: &PersistWalletId) -> PathBuf {
    root.join(format!(
        "wallet_{}_tx_history.jsonl",
        wallet_stem(wallet_id)
    ))
}

fn staged_restore_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("wallet");
    path.with_file_name(format!("{file_name}.restore.tmp"))
}

fn history_backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("wallet");
    path.with_file_name(format!("{file_name}.bak"))
}

fn wlt_backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("wallet");
    path.with_file_name(format!("{file_name}.bak"))
}

fn restore_mark_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("wallet");
    path.with_file_name(format!("{file_name}.restore.json"))
}

fn export_pack(wallet_id: PersistWalletId, name: &str) -> WalletExportPack {
    let profile = WalletProfilePayload::new_with_checksum(
        wallet_id.clone(),
        name.to_string(),
        10,
        20,
        PasswordVerifierState {
            salt: [0x11; 32],
            verifier: [0x22; 32],
        },
        ReceiverDeriverState {
            next_payment_index: 3,
            next_change_index: 5,
        },
        PersistWalletSettings {
            auto_lock_timeout: 300,
            default_fee: "0.001".to_string(),
            currency_display: "Z00Z".to_string(),
            policy_rules: None,
            created_at: 10,
            updated_at: 20,
        },
        [0x33; 16],
        WalletState::Locked,
    );
    let mut manifest = BackupManifestPayload {
        version: BackupManifestPayload::VERSION,
        wallet_id,
        created_at_ms: 42,
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
        profile_count: 1,
        owned_asset_count: 0,
        owned_object_count: 0,
        scan_state_count: 0,
        stealth_meta_count: 0,
        tofu_pins_count: 0,
        key_ref_count: 0,
        tx_record_count: 1,
        has_tx_history_sidecar: true,
        tx_history_plane: BackupManifestPayload::TX_HISTORY_JSONL.to_string(),
        checksum: None,
    };
    manifest.checksum = Some(manifest.compute_checksum());

    WalletExportPack {
        version: WalletExportPack::VERSION,
        manifest: Some(manifest),
        wallet_profile: Some(profile),
        owned_assets: Vec::new(),
        owned_objects: Vec::new(),
        scan_state: None,
        stealth_meta: None,
        tofu_pins: None,
        keys: None,
        tx_history_plane: Some(BackupManifestPayload::TX_HISTORY_JSONL.to_string()),
        seed_phrase: TEST_SEED_PHRASE_24.to_string(),
        wallet_identity: Some(WalletExportIdentity {
            network: "p2p".to_string(),
            chain: "devnet".to_string(),
        }),
    }
}

fn history_records(prefix: &str, timestamp_ms: u64) -> Vec<TxRecord> {
    vec![TxRecord {
        tx_hash: format!("{prefix}-tx-1"),
        tx_bytes: format!("{prefix}-payload").into_bytes(),
        imported: false,
        status: TxStatus::Pending,
        timestamp_ms,
        block_height: None,
        confirmation_evidence: None,
    }]
}

fn build_backup(
    backup_path: &Path,
    wallet_id: &PersistWalletId,
    history_rows: &[TxRecord],
) -> Vec<u8> {
    let wallet_history = encode_tx_history_jsonl(&wallet_stem(wallet_id), history_rows)
        .expect("encode tx-history jsonl");
    let exporter = BackupExporterImpl::new_with_forensic_history(
        wallet_id.0.clone(),
        "p2p".to_string(),
        "devnet".to_string(),
        export_pack(wallet_id.clone(), "restored-profile"),
        history_rows.to_vec(),
        MockTimeProvider::new(SystemTime::UNIX_EPOCH + Duration::from_secs(5)),
        SystemRngProvider,
    );
    exporter
        .export_with_history_bytes(
            backup_path.to_string_lossy().as_ref(),
            &SafePassword::from(TEST_PASSWORD),
            &wallet_history,
        )
        .expect("export backup with history bytes");
    wallet_history
}

async fn make_existing_wallet(
    output_dir: &Path,
) -> (Arc<WalletService>, PersistWalletId, PathBuf, PathBuf) {
    let wallets = Arc::new(WalletService::with_output_dir(output_dir.to_path_buf()));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));
    let wallet_id = app
        .create_wallet(
            "restore-atomic-existing".to_string(),
            TEST_PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .expect("create existing wallet")
        .wallet_id;
    let wlt = wlt_path(output_dir, &wallet_id);
    let history = history_path(output_dir, &wallet_id);
    (wallets, wallet_id, wlt, history)
}

fn assert_temp_restore_files_cleared(wlt: &Path, history: &Path) {
    for path in [
        staged_restore_path(wlt),
        wlt_backup_path(wlt),
        restore_mark_path(wlt),
        staged_restore_path(history),
        history_backup_path(history),
    ] {
        assert!(
            !path_exists(&path).expect("check temp path existence"),
            "restore temp path must be removed: {}",
            path.display()
        );
    }
}

async fn run_restore_failure_case(failpoint: &str) {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
    let _failpoint = RestoreFailpointGuard::set(failpoint);
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let backup_dir = temp.path().join("backups");
    let backup_path = backup_dir.join(format!("{failpoint}.backup"));

    let (wallets, wallet_id, wlt, history) = make_existing_wallet(&output_dir).await;
    let original_history =
        encode_tx_history_jsonl(&wallet_stem(&wallet_id), &history_records("orig", 11))
            .expect("encode original history");
    write_file(&history, &original_history).expect("write original history");
    let original_wlt = read_file(&wlt).expect("read original wlt");

    let incoming_history = history_records("incoming", 99);
    build_backup(&backup_path, &wallet_id, &incoming_history);

    let err = wallets
        .restore_backup(
            backup_path.to_string_lossy().to_string(),
            SafePassword::from(TEST_PASSWORD),
            Some("restored-wallet".to_string()),
        )
        .await
        .expect_err("restore failpoint must abort restore");

    let err_text = err.to_string();
    assert!(
        err_text.contains(failpoint),
        "restore error must name failpoint, got: {err_text}"
    );
    assert_eq!(read_file(&wlt).expect("read final wlt"), original_wlt);
    assert_eq!(
        read_file(&history).expect("read final history"),
        original_history
    );
    assert_temp_restore_files_cleared(&wlt, &history);
}

async fn run_restore_retry_case(failpoint: &str) {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let backup_dir = temp.path().join("backups");
    let backup_path = backup_dir.join(format!("{failpoint}.backup"));

    let (wallets, wallet_id, wlt, history) = make_existing_wallet(&output_dir).await;
    let original_history =
        encode_tx_history_jsonl(&wallet_stem(&wallet_id), &history_records("orig", 11))
            .expect("encode original history");
    write_file(&history, &original_history).expect("write original history");
    let original_wlt = read_file(&wlt).expect("read original wlt");

    let incoming_history = history_records("incoming", 99);
    let incoming_history_bytes = build_backup(&backup_path, &wallet_id, &incoming_history);

    let err = {
        let _failpoint = RestoreFailpointGuard::set(failpoint);
        wallets
            .restore_backup(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from(TEST_PASSWORD),
                Some("restored-wallet".to_string()),
            )
            .await
            .expect_err("restore crash failpoint must abort restore")
    };
    let err_text = err.to_string();
    assert!(
        err_text.contains(failpoint),
        "restore error must name failpoint, got: {err_text}"
    );
    assert!(
        path_exists(restore_mark_path(&wlt)).expect("mark path exists"),
        "restore mark must survive crash-style interruption"
    );
    assert!(
        path_exists(wlt_backup_path(&wlt)).expect("wlt backup path exists"),
        "wlt backup must survive crash-style interruption"
    );

    let restored = wallets
        .restore_backup(
            backup_path.to_string_lossy().to_string(),
            SafePassword::from(TEST_PASSWORD),
            Some("restored-wallet".to_string()),
        )
        .await
        .expect("restore retry must succeed");

    assert_eq!(restored.wallet_id, wallet_id);
    assert_ne!(read_file(&wlt).expect("read updated wlt"), original_wlt);
    assert_eq!(
        read_file(&history).expect("read updated history"),
        incoming_history_bytes
    );
    assert_eq!(
        decode_tx_history_jsonl(&incoming_history_bytes).expect("decode incoming history"),
        incoming_history
    );
    assert_temp_restore_files_cleared(&wlt, &history);
}

#[tokio::test]
async fn restore_rolls_back_history_commit_failure() {
    run_restore_failure_case("history_commit").await;
}

#[tokio::test]
async fn restore_rolls_back_wlt_commit_failure() {
    run_restore_failure_case("wlt_commit").await;
}

#[tokio::test]
async fn restore_rolls_back_publish_failure() {
    run_restore_failure_case("publish").await;
}

#[tokio::test]
async fn retry_after_history_crash() {
    run_restore_retry_case("crash_after_history").await;
}

#[tokio::test]
async fn retry_after_wlt_crash() {
    run_restore_retry_case("crash_after_wlt").await;
}

#[tokio::test]
async fn restore_replaces_wallet_and_history_when_no_failpoint_is_set() {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let backup_dir = temp.path().join("backups");
    let backup_path = backup_dir.join("success.backup");

    let (wallets, wallet_id, wlt, history) = make_existing_wallet(&output_dir).await;
    let original_history =
        encode_tx_history_jsonl(&wallet_stem(&wallet_id), &history_records("orig", 11))
            .expect("encode original history");
    write_file(&history, &original_history).expect("write original history");
    let original_wlt = read_file(&wlt).expect("read original wlt");

    let incoming_history = history_records("incoming", 99);
    let incoming_history_bytes = build_backup(&backup_path, &wallet_id, &incoming_history);

    let restored = wallets
        .restore_backup(
            backup_path.to_string_lossy().to_string(),
            SafePassword::from(TEST_PASSWORD),
            Some("restored-wallet".to_string()),
        )
        .await
        .expect("restore backup");

    assert_eq!(restored.wallet_id, wallet_id);
    assert_ne!(read_file(&wlt).expect("read updated wlt"), original_wlt);
    assert_eq!(
        read_file(&history).expect("read updated history"),
        incoming_history_bytes
    );
    assert_eq!(
        decode_tx_history_jsonl(&incoming_history_bytes).expect("decode incoming history"),
        incoming_history
    );
    assert_temp_restore_files_cleared(&wlt, &history);
}
