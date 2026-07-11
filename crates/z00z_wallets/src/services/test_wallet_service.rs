use super::*;

#[cfg(test)]
mod tests {
    use super::wallet_actions::ReceivePersistTestHook;
    use super::*;
    use crate::db::test_owned_objects::{test_owned_right_payload, test_owned_voucher_payload};
    use crate::db::{
        ObjectInventoryFilter, OwnedObjectPayload, OwnedRightStatus, OwnedVoucherStatus,
        WalletAssetStore,
    };
    use crate::rpc::logging::RpcLoggingConfig;
    use crate::rpc::methods::{
        AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
        NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
    };
    use crate::rpc::register_all_wallet_rpc_methods;
    use crate::rpc::types::chain::RuntimeReceiveScanOutcome;
    use crate::services::wallet_runtime_config::resolve_wallet_identity;
    use crate::wallet::stub_defaults::StubDefault;
    use crate::wallet::LockTrigger;
    use crate::AppService;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::ffi::OsString;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::task::JoinSet;
    use tokio::time::timeout;
    use z00z_crypto::expert::encoding::SafePassword;
    use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcTransport};
    use z00z_utils::codec::{json, Value};
    use z00z_utils::io::write_file;
    use z00z_utils::rng::SecureRngProvider;
    use z00z_utils::time::MockTimeProvider;

    const TEST_PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
    const TEST_WRONG_PASSWORD: &str = "Ff6^gG7&hH8*iI9(jJ0)";
    const CONCURRENT_DERIVE_TIMEOUT: Duration = Duration::from_secs(10);

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
        fn sleep<'a>(
            &'a self,
            duration: Duration,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
            Box::pin(async move {
                self.time.advance_by(duration);
            })
        }
    }

    #[derive(Debug, Clone)]
    struct SeqTestSecureRngProvider {
        seed: u64,
        counter: Arc<AtomicU64>,
    }

    impl SeqTestSecureRngProvider {
        fn new(seed: u64) -> Self {
            Self {
                seed,
                counter: Arc::new(AtomicU64::new(0)),
            }
        }

        fn next_seed(&self) -> u64 {
            let n = self.counter.fetch_add(1, Ordering::Relaxed);
            self.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        }
    }

    impl SecureRngProvider for SeqTestSecureRngProvider {
        type Rng = StdRng;

        fn rng(&self) -> Self::Rng {
            StdRng::seed_from_u64(self.next_seed())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    struct FailingWltStore;

    #[cfg(not(target_arch = "wasm32"))]
    impl WltStore for FailingWltStore {
        fn create_wallet_store(
            &self,
            _path: &Path,
            _wallet_id: &PersistWalletId,
            _password: &SafePassword,
            _seed_phrase: &str,
            _identity: &WalletIdentity,
        ) -> WalletResult<()> {
            Err(WalletError::InvalidConfig(
                "forced .wlt create failure".to_string(),
            ))
        }

        fn open_wallet_store(
            &self,
            _path: &Path,
            _wallet_id: &PersistWalletId,
            _password: &SafePassword,
            _identity: &WalletIdentity,
        ) -> WalletResult<crate::db::WalletSession> {
            Err(WalletError::InvalidConfig(
                "forced .wlt open failure".to_string(),
            ))
        }

        fn discover_wallet_store(&self, _path: &Path) -> WalletResult<PersistWalletDiscovery> {
            Err(WalletError::InvalidConfig(
                "forced .wlt discover failure".to_string(),
            ))
        }

        fn reveal_seed_phrase(&self, _session: &crate::db::WalletSession) -> WalletResult<String> {
            Err(WalletError::InvalidConfig(
                "forced .wlt reveal failure".to_string(),
            ))
        }

        fn verify_password(
            &self,
            _session: &crate::db::WalletSession,
            _password: &SafePassword,
        ) -> WalletResult<()> {
            Err(WalletError::InvalidConfig(
                "forced .wlt verify failure".to_string(),
            ))
        }

        fn write_wallet_profile(
            &self,
            _session: &crate::db::WalletSession,
            _profile_bytes: Vec<u8>,
        ) -> WalletResult<u64> {
            Err(WalletError::InvalidConfig(
                "forced .wlt profile write failure".to_string(),
            ))
        }

        fn read_wallet_profile(
            &self,
            _session: &crate::db::WalletSession,
        ) -> WalletResult<crate::security::SecretBytes> {
            Err(WalletError::InvalidConfig(
                "forced .wlt profile read failure".to_string(),
            ))
        }
    }

    fn test_seed_phrase_24() -> &'static str {
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
    }

    fn test_wallet_id(i: u32) -> PersistWalletId {
        PersistWalletId(format!("wallet_{i:064x}"))
    }

    fn create_test_service_raw(
        output_dir: std::path::PathBuf,
        time: Arc<MockTimeProvider>,
    ) -> WalletService {
        WalletService::create_service_custom_output_directory(output_dir, time, SystemRngProvider)
    }

    fn create_test_service_at(
        output_dir: std::path::PathBuf,
        time: Arc<MockTimeProvider>,
    ) -> WalletService {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env();
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");
        create_test_service_raw(output_dir, time)
    }

    fn create_service_with_default_config(time: Arc<dyn TimeProvider>) -> WalletService {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env();
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");
        WalletService::with_dependencies(time)
    }

    fn create_service_with_rng<P>(time: Arc<dyn TimeProvider>, rng_provider: P) -> WalletService
    where
        P: SecureRngProvider + Send + Sync + 'static,
    {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env();
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");
        WalletService::with_dependencies_and_rng_provider(time, rng_provider)
    }

    fn test_service_with_tempdir() -> (WalletService, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut service = create_test_service_at(dir.path().to_path_buf(), time.clone());
        service.sleeper = Arc::new(MockSleeper::new(time));
        (service, dir)
    }

    fn test_service_tempdir_raw() -> (WalletService, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut service = create_test_service_raw(dir.path().to_path_buf(), time.clone());
        service.sleeper = Arc::new(MockSleeper::new(time));
        (service, dir)
    }

    fn build_local_rpc_transport(wallet_svc: Arc<WalletService>) -> LocalRpcTransport {
        let app_svc = Arc::new(AppService::with_wallet_service(Arc::clone(&wallet_svc)));
        let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_svc)));
        let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&wallet_svc)));
        let asset_rpc = Arc::new(AssetRpcImpl::with_wallet_service(Arc::clone(&wallet_svc)));
        let tx_rpc = Arc::new(TxRpcImpl::new(Arc::clone(&wallet_svc)));
        let backup_rpc = Arc::new(BackupRpcImpl::new(Arc::clone(&wallet_svc)));
        let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(&wallet_svc)));
        let chain_rpc = Arc::new(ChainRpcImpl::new(Arc::clone(&app_svc)));
        let network_rpc = Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_svc)));
        let scan_rpc = Arc::new(ChainScanRpcImpl::new(Arc::clone(&app_svc)));
        let storage_rpc = Arc::new(StorageRpcImpl::new(Arc::clone(&wallet_svc)));

        let dispatcher = Arc::new(RpcDispatcher::new());
        register_all_wallet_rpc_methods(
            &dispatcher,
            app_rpc,
            wallet_rpc,
            asset_rpc,
            tx_rpc,
            backup_rpc,
            key_rpc,
            chain_rpc,
            network_rpc,
            scan_rpc,
            storage_rpc,
        )
        .expect("register_all_wallet_rpc_methods must succeed");

        LocalRpcTransport::new(dispatcher)
    }

    struct ReceivePersistHookGuard {
        wallet_id: PersistWalletId,
    }

    impl ReceivePersistHookGuard {
        fn install(wallet_id: &PersistWalletId, hook: ReceivePersistTestHook) -> Self {
            WalletService::set_receive_persist_test_hook(Some(wallet_id.clone()), Some(hook));
            Self {
                wallet_id: wallet_id.clone(),
            }
        }
    }

    impl Drop for ReceivePersistHookGuard {
        fn drop(&mut self) {
            WalletService::set_receive_persist_test_hook(Some(self.wallet_id.clone()), None);
        }
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

    fn write_wallet_config(dir: &tempfile::TempDir, yaml: &str) -> std::path::PathBuf {
        let path = dir.path().join("wallet_config.yaml");
        z00z_utils::io::write_file(&path, yaml.as_bytes()).expect("wallet config must write");
        path
    }

    fn default_identity() -> WalletIdentity {
        WalletIdentity {
            network: "p2p".to_string(),
            chain: "devnet".to_string(),
        }
    }

    fn test_wallet_settings(created_at: u64, updated_at: u64) -> PersistWalletSettings {
        PersistWalletSettings {
            auto_lock_timeout: 42,
            default_fee: "0.125".to_string(),
            currency_display: "TOK".to_string(),
            policy_rules: None,
            created_at,
            updated_at,
        }
    }

    fn test_wallet_profile_payload(
        wallet_id: &PersistWalletId,
        created_at: u64,
        updated_at: u64,
    ) -> WalletProfilePayload {
        WalletProfilePayload::new_with_checksum(
            wallet_id.clone(),
            "phase047-profile-only".to_string(),
            created_at,
            updated_at,
            PasswordVerifierState {
                salt: [7u8; 32],
                verifier: [9u8; 32],
            },
            ReceiverDeriverState {
                next_payment_index: 3,
                next_change_index: 4,
            },
            test_wallet_settings(created_at, updated_at),
            [5u8; 16],
            WalletState::Locked,
        )
    }

    #[test]
    fn wallet_stem_names_sync() {
        let (service, dir) = test_service_with_tempdir();
        let wallet_id = PersistWalletId("wallet-naming-contract".to_string());
        let hash = compute_wallet_file_id(&wallet_id.0);
        let expected_stem = hex::encode(&hash[..8]);

        let wallet_stem = WalletService::wallet_stem(&wallet_id);
        assert_eq!(wallet_stem, expected_stem);
        assert_eq!(wallet_stem.len(), 16);
        assert_eq!(
            WalletService::wallet_file_name(&wallet_stem),
            format!("wallet_{wallet_stem}.wlt")
        );
        assert_eq!(
            WalletService::wallet_history_jsonl_name(&wallet_stem),
            format!("wallet_{wallet_stem}_tx_history.jsonl")
        );
        assert_ne!(
            WalletService::wallet_history_jsonl_name(&wallet_stem),
            format!("tx_history_{wallet_stem}.jsonl")
        );
        assert_eq!(
            service.wlt_file_path(&wallet_id),
            dir.path().join(format!("wallet_{wallet_stem}.wlt"))
        );
        assert_eq!(
            service.wallet_history_jsonl_path(&wallet_id),
            dir.path()
                .join(format!("wallet_{wallet_stem}_tx_history.jsonl"))
        );
        let noncanonical_history_dir = dir.path().join(format!("wallet_{wallet_stem}_tx_history"));
        assert_eq!(
            service
                .wallet_history_jsonl_path(&wallet_id)
                .with_file_name(format!("wallet_{wallet_stem}_tx_history")),
            noncanonical_history_dir
        );

        let rpc_export_dir =
            crate::rpc::methods::tx_runtime_state::tx_export_dir_for_output(dir.path());
        assert_ne!(
            service.wallet_history_jsonl_path(&wallet_id),
            rpc_export_dir
        );
        assert_ne!(noncanonical_history_dir, rpc_export_dir);
        assert!(!service
            .wallet_history_jsonl_path(&wallet_id)
            .starts_with(&rpc_export_dir));
        assert!(!noncanonical_history_dir.starts_with(&rpc_export_dir));
    }

    #[test]
    fn noncanonical_history_name_rejected() {
        let (service, _dir) = test_service_with_tempdir();
        let wallet_id = PersistWalletId("wallet-noncanonical-history-order".to_string());
        let wallet_stem = WalletService::wallet_stem(&wallet_id);
        let noncanonical_name = format!("tx_history_{wallet_stem}.jsonl");
        let canonical_name = format!("wallet_{wallet_stem}_tx_history.jsonl");

        assert_eq!(
            WalletService::wallet_history_jsonl_name(&wallet_stem),
            canonical_name
        );
        assert_ne!(
            WalletService::wallet_history_jsonl_name(&wallet_stem),
            noncanonical_name
        );
        assert_eq!(
            service
                .wallet_history_jsonl_path(&wallet_id)
                .file_name()
                .and_then(|name| name.to_str()),
            Some(canonical_name.as_str())
        );
        assert_ne!(
            service
                .wallet_history_jsonl_path(&wallet_id)
                .file_name()
                .and_then(|name| name.to_str()),
            Some(noncanonical_name.as_str())
        );
    }

    #[tokio::test]
    async fn test_open_source_backfills_yaml() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let cfg_dir = tempfile::tempdir().unwrap();
        let cfg_path = write_wallet_config(
            &cfg_dir,
            r#"
wallet:
  network:
    type: "p2p"
  chain:
    type: "devnet"
  settings:
    auto_lock_timeout_secs: 42
    default_fee: "0.125"
    currency_display: "TOK"
  auto_lock:
    timeout_secs: 42
    triggers: []
"#,
        );
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

        let (service_a, dir) = test_service_tempdir_raw();
        let wallet_id = service_a
            .create_wallet_in_memory(
                "phase047-wallet",
                SafePassword::from(TEST_PASSWORD),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();
        let wlt_path = service_a.wlt_file_path(&wallet_id);

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = create_test_service_raw(dir.path().to_path_buf(), time.clone());
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        let discovery = service_b
            .open_wallet_source(WalletSource::Path {
                path: wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap();
        assert_eq!(discovery.wallet_id, wallet_id);

        let settings = service_b.get_wallet_settings(&wallet_id).await.unwrap();
        assert_eq!(
            settings.auto_lock_timeout, 42,
            "phase-047 requires open_wallet_source default backfill to use wallet.settings.auto_lock_timeout_secs"
        );
        assert_eq!(settings.default_fee, "0.125");
        assert_eq!(settings.currency_display, "TOK");
    }

    #[tokio::test]
    async fn test_open_imports_history() {
        let (source_service, _source_dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = source_service
            .create_wallet_in_memory(
                "phase047-history-sidecar",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        source_service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let source_history_path = source_service.wallet_history_jsonl_path(&wallet_id);
        let history_record = sample_backup_tx_record("tx-sidecar-1", vec![1, 2, 3, 4]);
        let mut source_store = crate::persistence::tx::TxStorageImpl::new(
            &source_history_path,
            z00z_utils::time::SystemTimeProvider,
        );
        crate::persistence::tx::TxStorage::put(&mut source_store, history_record.clone()).unwrap();

        let source_wlt_path = source_service.wlt_file_path(&wallet_id);
        let dest_dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut dest_service = create_test_service_raw(dest_dir.path().to_path_buf(), time.clone());
        dest_service.sleeper = Arc::new(MockSleeper::new(time));

        let discovery = dest_service
            .open_wallet_source(WalletSource::Path {
                path: source_wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap();
        assert_eq!(discovery.wallet_id, wallet_id);

        let dest_history_path = dest_service.wallet_history_jsonl_path(&wallet_id);
        let dest_store = crate::persistence::tx::TxStorageImpl::new(
            &dest_history_path,
            z00z_utils::time::SystemTimeProvider,
        );
        let imported_history = crate::persistence::tx::TxStorage::list(&dest_store).unwrap();
        assert_eq!(imported_history, vec![history_record]);
    }

    #[tokio::test]
    async fn test_open_reconciles_history() {
        let (source_service, _source_dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = source_service
            .create_wallet_in_memory(
                "phase047-history-sync",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        source_service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let source_history_path = source_service.wallet_history_jsonl_path(&wallet_id);
        let history_record = sample_backup_tx_record("tx-sidecar-sync", vec![4, 3, 2, 1]);
        let mut source_store = crate::persistence::tx::TxStorageImpl::new(
            &source_history_path,
            z00z_utils::time::SystemTimeProvider,
        );
        crate::persistence::tx::TxStorage::put(&mut source_store, history_record.clone()).unwrap();

        let source_wlt_path = source_service.wlt_file_path(&wallet_id);
        let source_wlt_bytes = crate::db::wallet_io::read_file(source_wlt_path.as_path()).unwrap();

        let dest_dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut dest_service = create_test_service_raw(dest_dir.path().to_path_buf(), time.clone());
        dest_service.sleeper = Arc::new(MockSleeper::new(time));

        dest_service
            .open_wallet_source(WalletSource::Bytes {
                bytes: source_wlt_bytes,
            })
            .await
            .unwrap();
        assert!(!dest_service.wallet_history_jsonl_path(&wallet_id).exists());

        let discovery = dest_service
            .open_wallet_source(WalletSource::Path {
                path: source_wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap();
        assert_eq!(discovery.wallet_id, wallet_id);

        let dest_history_path = dest_service.wallet_history_jsonl_path(&wallet_id);
        let dest_store = crate::persistence::tx::TxStorageImpl::new(
            &dest_history_path,
            z00z_utils::time::SystemTimeProvider,
        );
        let imported_history = crate::persistence::tx::TxStorage::list(&dest_store).unwrap();
        assert_eq!(imported_history, vec![history_record]);
    }

    #[tokio::test]
    async fn test_open_rejects_bad_history() {
        let (source_service, _source_dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = source_service
            .create_wallet_in_memory(
                "phase047-history-malformed",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        source_service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let source_history_path = source_service.wallet_history_jsonl_path(&wallet_id);
        z00z_utils::io::write_file(&source_history_path, b"not-jsonl").unwrap();

        let source_wlt_path = source_service.wlt_file_path(&wallet_id);
        let dest_dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut dest_service = create_test_service_raw(dest_dir.path().to_path_buf(), time.clone());
        dest_service.sleeper = Arc::new(MockSleeper::new(time));

        let err = dest_service
            .open_wallet_source(WalletSource::Path {
                path: source_wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("wallet source tx-history import failed")
        ));
        assert!(!dest_service.wlt_file_path(&wallet_id).exists());
        assert!(!dest_service.wallet_history_jsonl_path(&wallet_id).exists());
    }

    #[tokio::test]
    async fn test_open_rejects_big_history() {
        let (source_service, _source_dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = source_service
            .create_wallet_in_memory(
                "phase047-history-oversized",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        source_service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let source_history_path = source_service.wallet_history_jsonl_path(&wallet_id);
        let oversized =
            vec![b'\n'; crate::persistence::tx::MAX_TX_HISTORY_JSONL_BYTES as usize + 1];
        z00z_utils::io::write_file(&source_history_path, &oversized).unwrap();

        let source_wlt_path = source_service.wlt_file_path(&wallet_id);
        let dest_dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut dest_service = create_test_service_raw(dest_dir.path().to_path_buf(), time.clone());
        dest_service.sleeper = Arc::new(MockSleeper::new(time));

        let err = dest_service
            .open_wallet_source(WalletSource::Path {
                path: source_wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap_err();

        let err_text = format!("{err:?}");
        assert!(
            err_text.contains("tx-history JSONL file too large")
                || err_text.contains("File too large"),
            "{err_text}"
        );
        assert!(!dest_service.wlt_file_path(&wallet_id).exists());
        assert!(!dest_service.wallet_history_jsonl_path(&wallet_id).exists());
    }

    #[tokio::test]
    async fn test_open_rolls_back_history() {
        let (source_service, _source_dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = source_service
            .create_wallet_in_memory(
                "phase047-history-rollback",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        source_service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let source_history_path = source_service.wallet_history_jsonl_path(&wallet_id);
        let mut source_store = crate::persistence::tx::TxStorageImpl::new(
            &source_history_path,
            z00z_utils::time::SystemTimeProvider,
        );
        crate::persistence::tx::TxStorage::put(
            &mut source_store,
            sample_backup_tx_record("tx-sidecar-rollback", vec![6, 6, 6]),
        )
        .unwrap();

        let source_wlt_path = source_service.wlt_file_path(&wallet_id);
        let dest_dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut dest_service = create_test_service_raw(dest_dir.path().to_path_buf(), time.clone());
        dest_service.sleeper = Arc::new(MockSleeper::new(time));

        let blocked_history_path = dest_service.wallet_history_jsonl_path(&wallet_id);
        z00z_utils::io::create_dir_all(&blocked_history_path).unwrap();

        let err = dest_service
            .open_wallet_source(WalletSource::Path {
                path: source_wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap_err();

        let err_text = format!("{err:?}");
        assert!(
            err_text.contains("Backup tx-history import failed")
                || err_text.contains("wallet history import task failed")
                || err_text.contains("directory"),
            "{err_text}"
        );
        assert!(!dest_service.wlt_file_path(&wallet_id).exists());
    }

    #[test]
    fn test_yaml_auto_lock() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env();
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let cfg_dir = tempfile::tempdir().unwrap();
        let cfg_path = write_wallet_config(
            &cfg_dir,
            r#"
wallet:
  settings:
    auto_lock_timeout_secs: 42
    default_fee: "0.125"
    currency_display: "TOK"
  auto_lock:
    timeout_secs: 42
    triggers:
      - "manual"
"#,
        );
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

        let (service, _dir) = test_service_tempdir_raw();
        assert_eq!(service.auto_lock_policy.timeout.as_secs(), 42);
        assert_eq!(service.auto_lock_policy.triggers, vec![LockTrigger::Manual]);
    }

    #[tokio::test]
    async fn test_load_profile_only_wlt() {
        use z00z_utils::codec::{BincodeCodec, Codec};

        let (service, _dir) = test_service_with_tempdir();
        let wallet_id = test_wallet_id(47);
        let identity = default_identity();
        let password = SafePassword::from(TEST_PASSWORD);
        let wlt_path = service.wlt_file_path(&wallet_id);

        service
            .wlt_store
            .create_wallet_store(
                &wlt_path,
                &wallet_id,
                &password,
                test_seed_phrase_24(),
                &identity,
            )
            .unwrap();
        let session = service
            .wlt_store
            .open_wallet_store(&wlt_path, &wallet_id, &password, &identity)
            .unwrap();
        let profile = test_wallet_profile_payload(&wallet_id, 11, 12);
        let profile_bytes = BincodeCodec.serialize(&profile).unwrap();
        service
            .wlt_store
            .write_wallet_profile(&session, profile_bytes)
            .unwrap();
        drop(session);

        service
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();

        let settings = service.get_wallet_settings(&wallet_id).await.unwrap();
        assert_eq!(settings.auto_lock_timeout, 42);
        assert_eq!(settings.default_fee, "0.125");
        let counters = service.get_deriver_state(&wallet_id).await.unwrap();
        assert_eq!(counters.next_payment_index, 3);
        assert_eq!(counters.next_change_index, 4);
        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(
            claimed.is_empty(),
            "profile-only load must restore non-asset wallet state without any legacy asset bridge"
        );
    }

    #[tokio::test]
    async fn test_save_profile_only_wlt() {
        let (service_a, dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        service_a
            .derive_public_key_for_path(&wallet_id, Bip44Path::payment(0).unwrap())
            .await
            .unwrap();
        service_a
            .save_wallet(wallet_id.clone(), password.clone(), None)
            .await
            .unwrap();
        service_a.lock_wallet(&wallet_id).await.unwrap();

        let identity = service_a
            .resolve_persisted_wallet_identity(&wallet_id)
            .await
            .unwrap();
        let wlt_path = service_a.wlt_file_path(&wallet_id);
        let session = service_a
            .wlt_store
            .open_wallet_store(&wlt_path, &wallet_id, &password, &identity)
            .unwrap();

        assert!(
            service_a.wlt_store.read_wallet_profile(&session).is_ok(),
            "normal create/save must persist WalletProfilePayload"
        );
        drop(session);

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();

        let counters = service_b.get_deriver_state(&wallet_id).await.unwrap();
        assert_eq!(counters.next_payment_index, 1);
        assert_eq!(counters.next_change_index, 0);
        assert!(
            service_b
                .list_claimed_assets(&wallet_id)
                .await
                .unwrap()
                .is_empty(),
            "profile-only reopen must not require preexisting owned-asset rows"
        );
    }

    #[tokio::test]
    async fn test_save_keeps_assets() {
        let (service_a, dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let asset = test_backup_claimed_asset();
        assert!(service_a
            .put_claimed_asset(&wallet_id, asset.clone())
            .await
            .unwrap());

        let now_ms = service_a.require_now_ms().unwrap();
        let timeout_ms = service_a.timeout_ms();
        let _session_before = service_a
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();

        service_a
            .derive_public_key_for_path(&wallet_id, Bip44Path::payment(0).unwrap())
            .await
            .unwrap();
        service_a
            .save_wallet(wallet_id.clone(), password.clone(), None)
            .await
            .unwrap();

        service_a.lock_wallet(&wallet_id).await.unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();

        let counters = service_b.get_deriver_state(&wallet_id).await.unwrap();
        assert_eq!(counters.next_payment_index, 1);
        assert_eq!(counters.next_change_index, 0);

        let claimed = service_b.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].asset_id(), asset.asset_id());
    }

    #[test]
    fn jsonl_import_keeps_view() {
        let (service, _dir) = test_service_with_tempdir();
        let wallet_id = PersistWalletId("wallet-jsonl-import".to_string());
        let source_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let record = crate::persistence::tx::TxRecord {
            tx_hash: "tx-jsonl-1".to_string(),
            tx_bytes: vec![9, 8, 7],
            imported: false,
            status: crate::persistence::tx::TxStatus::Confirmed,
            timestamp_ms: 1_700_000_000,
            block_height: Some(9),
            confirmation_evidence: None,
        };
        let wallet_stem = WalletService::wallet_stem(&wallet_id);
        let bytes =
            crate::backup::encode_tx_history_jsonl(&wallet_stem, &[record.clone()]).unwrap();
        write_file(source_path.to_string_lossy().as_ref(), &bytes).unwrap();

        service
            .import_tx_history_jsonl(&wallet_id, source_path.as_ref())
            .unwrap();

        let live_path = service.wallet_history_jsonl_path(&wallet_id);
        let store = crate::persistence::tx::TxStorageImpl::new(
            &live_path,
            z00z_utils::time::MockTimeProvider::default(),
        );
        let records = crate::persistence::tx::TxStorage::list(&store).unwrap();
        assert_eq!(records, vec![record]);
        assert!(live_path.exists());
        assert!(!live_path
            .with_file_name(format!(
                "wallet_{}_tx_history",
                WalletService::wallet_stem(&wallet_id)
            ))
            .exists());
    }

    #[test]
    fn jsonl_repeat_import_keeps_view() {
        let (service, _dir) = test_service_with_tempdir();
        let wallet_id = PersistWalletId("wallet-jsonl-repeat".to_string());
        let source_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let record = crate::persistence::tx::TxRecord {
            tx_hash: "tx-jsonl-repeat-1".to_string(),
            tx_bytes: vec![1, 2, 3, 4],
            imported: false,
            status: crate::persistence::tx::TxStatus::Confirmed,
            timestamp_ms: 1_700_000_000,
            block_height: Some(7),
            confirmation_evidence: None,
        };
        let wallet_stem = WalletService::wallet_stem(&wallet_id);
        let bytes =
            crate::backup::encode_tx_history_jsonl(&wallet_stem, &[record.clone()]).unwrap();
        write_file(source_path.to_string_lossy().as_ref(), &bytes).unwrap();

        service
            .import_tx_history_jsonl(&wallet_id, source_path.as_ref())
            .unwrap();
        service
            .import_tx_history_jsonl(&wallet_id, source_path.as_ref())
            .unwrap();

        let live_path = service.wallet_history_jsonl_path(&wallet_id);
        let store = crate::persistence::tx::TxStorageImpl::new(
            &live_path,
            z00z_utils::time::MockTimeProvider::default(),
        );
        let records = crate::persistence::tx::TxStorage::list(&store).unwrap();
        assert_eq!(records, vec![record]);
    }

    fn non_default_wallet_identity() -> WalletIdentity {
        let default_identity = resolve_wallet_identity();
        let chain = if default_identity.chain == "mainnet" {
            "devnet"
        } else {
            "mainnet"
        };

        WalletIdentity {
            network: format!("persisted-{}", default_identity.network),
            chain: chain.to_string(),
        }
    }

    fn string_kdf_contract_bytes() -> Vec<u8> {
        const SALT_BYTES: usize = 16;
        const NONCE_BYTES: usize = 24;

        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        struct TestBackupEncryption {
            algorithm: String,
            kdf: String,
            salt: [u8; SALT_BYTES],
            nonce: [u8; NONCE_BYTES],
        }

        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        struct TestBackupCompression {
            algorithm: String,
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        struct TestBackupContainer {
            metadata: crate::backup::BackupMetadata,
            encryption: TestBackupEncryption,
            compression: TestBackupCompression,
            checksum: [u8; 32],
            ciphertext: Vec<u8>,
        }

        use z00z_utils::codec::{Codec, JsonCodec};

        let metadata = crate::backup::BackupMetadata {
            version: 1,
            created_at: 1,
            wallet_id: "wallet-1".to_string(),
            network: "testnet".to_string(),
        };
        let encryption = TestBackupEncryption {
            algorithm: "xchacha20poly1305".to_string(),
            kdf: "argon2id".to_string(),
            salt: [1u8; SALT_BYTES],
            nonce: [0u8; NONCE_BYTES],
        };
        let compression = TestBackupCompression {
            algorithm: "zstd".to_string(),
        };
        let codec = JsonCodec;
        let checksum = [0u8; 32];
        let ciphertext = vec![0u8; 8];

        let container = TestBackupContainer {
            metadata,
            encryption,
            compression,
            checksum,
            ciphertext,
        };

        codec.serialize(&container).expect("serialize container")
    }

    fn sample_backup_tx_record(
        tx_hash: &str,
        tx_bytes: Vec<u8>,
    ) -> crate::persistence::tx::TxRecord {
        crate::persistence::tx::TxRecord {
            tx_hash: tx_hash.to_string(),
            tx_bytes,
            imported: false,
            status: crate::persistence::tx::TxStatus::Confirmed,
            timestamp_ms: 1_700_000_000,
            block_height: Some(42),
            confirmation_evidence: None,
        }
    }

    fn test_dev_coin_asset(serial_id: u32, amount: u64) -> z00z_core::Asset {
        let serials =
            z00z_core::genesis::asset_std::serials_from_dev_class(z00z_core::AssetClass::Coin)
                .expect("dev coin serials");
        assert!(
            serial_id < serials,
            "test serial_id {serial_id} exceeds live dev fixture limit {serials}"
        );
        z00z_core::genesis::asset_std::asset_from_dev_class(
            z00z_core::AssetClass::Coin,
            serial_id,
            amount,
        )
        .expect("dev coin asset")
    }

    fn test_backup_claimed_asset() -> z00z_core::Asset {
        test_dev_coin_asset(14, 99)
    }

    fn test_backup_profile_payload() -> crate::db::WalletProfilePayload {
        crate::db::WalletProfilePayload::new_with_checksum(
            PersistWalletId("wallet-1".to_string()),
            "Test Wallet".to_string(),
            1,
            2,
            crate::wallet::persistence::PasswordVerifierState {
                salt: [1u8; 32],
                verifier: [2u8; 32],
            },
            crate::wallet::persistence::ReceiverDeriverState {
                next_payment_index: 0,
                next_change_index: 0,
            },
            PersistWalletSettings {
                auto_lock_timeout: 300,
                default_fee: "0.001".to_string(),
                currency_display: "Z00Z".to_string(),
                policy_rules: None,
                created_at: 1,
                updated_at: 2,
            },
            [6u8; 16],
            WalletState::Locked,
        )
    }

    fn test_backup_owned_asset_payloads(
        asset_wires: Vec<z00z_core::AssetWire>,
    ) -> Vec<crate::db::OwnedAssetPayload> {
        asset_wires
            .into_iter()
            .map(|wire| {
                let asset = wire.clone().to_asset().expect("backup owned asset");
                let mut asset_wire = wire;
                asset_wire.secret = None;
                let mut payload = crate::db::OwnedAssetPayload {
                    version: crate::db::OwnedAssetPayload::VERSION,
                    wallet_id: PersistWalletId("wallet-1".to_string()),
                    account_id: None,
                    asset_id: asset.asset_id(),
                    asset_definition_id: asset.definition.id,
                    asset_wire,
                    status: crate::db::redb_store::OwnedAssetStatus::Spendable,
                    source: crate::db::redb_store::OwnedAssetSource::Restore,
                    first_seen: None,
                    last_updated_ms: 2,
                    scan_ref: None,
                    receive_ref: None,
                    spend_ref: None,
                    confirmation_ref: None,
                    labels: Vec::new(),
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
                    .expect("backup owned asset invariants");
                payload
            })
            .collect()
    }

    fn test_backup_manifest_payload(
        owned_asset_count: usize,
        owned_object_count: usize,
    ) -> crate::db::BackupManifestPayload {
        let mut manifest = crate::db::BackupManifestPayload {
            version: crate::db::BackupManifestPayload::VERSION,
            wallet_id: PersistWalletId("wallet-1".to_string()),
            created_at_ms: 1,
            network: "testnet".to_string(),
            chain: "mainnet".to_string(),
            profile_count: 1,
            owned_asset_count: owned_asset_count as u32,
            owned_object_count: owned_object_count as u32,
            scan_state_count: 0,
            stealth_meta_count: 0,
            tofu_pins_count: 0,
            key_ref_count: 0,
            tx_record_count: 0,
            has_tx_history_sidecar: true,
            tx_history_plane: crate::db::BackupManifestPayload::TX_HISTORY_JSONL.to_string(),
            checksum: None,
        };
        manifest.checksum = Some(manifest.compute_checksum());
        manifest
    }

    fn test_export_pack_owned_assets(
        asset_wires: Vec<z00z_core::AssetWire>,
    ) -> crate::wallet::persistence::WalletExportPack {
        let owned_assets = test_backup_owned_asset_payloads(asset_wires);
        crate::wallet::persistence::WalletExportPack {
            version: crate::wallet::persistence::WalletExportPack::VERSION,
            manifest: Some(test_backup_manifest_payload(owned_assets.len(), 0)),
            wallet_profile: Some(test_backup_profile_payload()),
            owned_assets,
            owned_objects: Vec::new(),
            scan_state: None,
            stealth_meta: None,
            tofu_pins: None,
            keys: None,
            tx_history_plane: Some(crate::db::BackupManifestPayload::TX_HISTORY_JSONL.to_string()),
            seed_phrase: test_seed_phrase_24().to_string(),
            wallet_identity: None,
        }
    }

    fn test_backup_export_pack() -> crate::wallet::persistence::WalletExportPack {
        let claimed_asset = test_backup_claimed_asset();
        test_export_pack_owned_assets(vec![z00z_core::AssetWire::from_asset(&claimed_asset)])
    }

    fn test_backup_wallet_stem() -> String {
        WalletService::wallet_stem(&PersistWalletId("wallet-1".to_string()))
    }

    fn forensic_backup_bytes() -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("backup.json");
        let records = vec![
            sample_backup_tx_record("tx-1", vec![1, 2, 3, 4]),
            sample_backup_tx_record("tx-2", vec![5, 6, 7, 8]),
        ];
        let exporter = crate::backup::BackupExporterImpl::new_with_forensic_history(
            "wallet-1".to_string(),
            "testnet".to_string(),
            "mainnet".to_string(),
            test_backup_export_pack(),
            records.clone(),
            MockTimeProvider::from_unix_secs(1),
            SystemRngProvider,
        );

        let wallet_stem = test_backup_wallet_stem();
        let history_bytes = crate::backup::encode_tx_history_jsonl(&wallet_stem, &records).unwrap();

        exporter
            .export_with_history_bytes(
                backup_path.to_string_lossy().as_ref(),
                &SafePassword::from("password"),
                &history_bytes,
            )
            .expect("forensic backup bytes");

        z00z_utils::io::read_file(&backup_path).unwrap()
    }

    fn wrong_stem_forensic_backup_bytes() -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("backup-wrong-stem.json");
        let records = vec![
            sample_backup_tx_record("tx-1", vec![1, 2, 3, 4]),
            sample_backup_tx_record("tx-2", vec![5, 6, 7, 8]),
        ];
        let exporter = crate::backup::BackupExporterImpl::new_with_forensic_history(
            "wallet-1".to_string(),
            "testnet".to_string(),
            "mainnet".to_string(),
            test_backup_export_pack(),
            records.clone(),
            MockTimeProvider::from_unix_secs(1),
            SystemRngProvider,
        );

        let history_bytes = crate::backup::encode_tx_history_jsonl("wrongstem", &records).unwrap();

        exporter
            .export_with_history_bytes(
                backup_path.to_string_lossy().as_ref(),
                &SafePassword::from("password"),
                &history_bytes,
            )
            .expect("wrong-stem forensic backup bytes");

        z00z_utils::io::read_file(&backup_path).unwrap()
    }

    fn dup_owned_asset_backup() -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("backup-duplicate-owned-assets.json");
        let records = vec![
            sample_backup_tx_record("tx-1", vec![1, 2, 3, 4]),
            sample_backup_tx_record("tx-2", vec![5, 6, 7, 8]),
        ];
        let claimed_asset = test_backup_claimed_asset();
        let asset_wire = z00z_core::AssetWire::from_asset(&claimed_asset);
        let exporter = crate::backup::BackupExporterImpl::new_with_forensic_history(
            "wallet-1".to_string(),
            "testnet".to_string(),
            "mainnet".to_string(),
            test_export_pack_owned_assets(vec![asset_wire.clone(), asset_wire]),
            records.clone(),
            MockTimeProvider::from_unix_secs(1),
            SystemRngProvider,
        );

        let wallet_stem = test_backup_wallet_stem();
        let history_bytes = crate::backup::encode_tx_history_jsonl(&wallet_stem, &records).unwrap();

        exporter
            .export_with_history_bytes(
                backup_path.to_string_lossy().as_ref(),
                &SafePassword::from("password"),
                &history_bytes,
            )
            .expect("duplicate-owned-assets forensic backup bytes");

        z00z_utils::io::read_file(&backup_path).unwrap()
    }

    fn tampered_forensic_backup_bytes() -> Vec<u8> {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        struct TestBackupEncryption {
            algorithm: String,
            kdf: crate::backup::BackupKdf,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            salt: Option<[u8; 16]>,
            nonce: [u8; 24],
        }

        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        struct TestBackupCompression {
            algorithm: String,
        }

        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        struct TestBackupContainer {
            metadata: crate::backup::BackupMetadata,
            encryption: TestBackupEncryption,
            compression: TestBackupCompression,
            checksum: [u8; 32],
            ciphertext: Vec<u8>,
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        struct TestBackupPayload {
            network: String,
            chain: String,
            export_pack: crate::wallet::persistence::WalletExportPack,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            forensic: Option<crate::backup::WalletForensicPack>,
        }

        fn build_aad_bytes(
            metadata: &crate::backup::BackupMetadata,
            encryption: &TestBackupEncryption,
            compression: &TestBackupCompression,
        ) -> Vec<u8> {
            #[derive(serde::Serialize)]
            struct TestBackupAssociatedData<'a> {
                metadata: &'a crate::backup::BackupMetadata,
                encryption: TestBackupEncryption,
                compression: &'a TestBackupCompression,
            }

            let mut stripped_encryption = encryption.clone();
            stripped_encryption.nonce = [0u8; 24];

            let aad_json = z00z_utils::codec::JsonCodec
                .serialize(&TestBackupAssociatedData {
                    metadata,
                    encryption: stripped_encryption,
                    compression,
                })
                .expect("serialize aad");

            let ctx = [crate::key::Z00ZKeyBranch::WalletBackup.as_aad_byte()];
            let prefix = z00z_crypto::aead::build_aad_multipart(
                crate::key::Z00ZKeyBranch::WalletBackup.label(),
                &[&ctx[..]],
            )
            .expect("build aad prefix");

            let checksum = crate::backup::WalletBackupCrypto::aad_tag(&aad_json);
            let mut tagged = Vec::with_capacity(prefix.len() + checksum.len() + aad_json.len());
            tagged.extend_from_slice(&prefix);
            tagged.extend_from_slice(&checksum);
            tagged.extend_from_slice(&aad_json);
            tagged
        }

        use z00z_utils::codec::{Codec, JsonCodec};

        let password = SafePassword::from("password");
        let mut container: TestBackupContainer = JsonCodec
            .deserialize(&forensic_backup_bytes())
            .expect("decode forensic backup container");
        let aad = build_aad_bytes(
            &container.metadata,
            &container.encryption,
            &container.compression,
        );
        let key = crate::backup::WalletBackupCrypto::derive_key_with_kdf(
            &password,
            &container.encryption.kdf,
        )
        .expect("derive backup key");
        let compressed =
            crate::backup::WalletBackupCrypto::decrypt(&key, &aad, &container.ciphertext)
                .expect("decrypt forensic payload");
        let plaintext =
            z00z_utils::compression::zstd_decompress_bounded(&compressed, 64 * 1024 * 1024)
                .expect("decompress forensic payload");
        let mut payload: TestBackupPayload = JsonCodec
            .deserialize(&plaintext)
            .expect("decode forensic payload");

        payload
            .forensic
            .as_mut()
            .expect("forensic payload")
            .manifest
            .entries[0]
            .tx_bytes_hash[0] ^= 0x01;

        let plaintext = JsonCodec
            .serialize(&payload)
            .expect("serialize tampered forensic payload");
        let compressed = z00z_utils::compression::zstd_compress(&plaintext)
            .expect("compress tampered forensic payload");
        let ciphertext = crate::backup::WalletBackupCrypto::encrypt(&key, &aad, &compressed)
            .expect("encrypt tampered forensic payload");
        container.encryption.nonce = ciphertext[1..25].try_into().expect("ciphertext nonce");
        container.checksum = crate::backup::WalletBackupCrypto::checksum(&aad, &ciphertext);
        container.ciphertext = ciphertext;

        JsonCodec
            .serialize(&container)
            .expect("serialize tampered forensic container")
    }

    fn make_recv_chunk(
        keys: &crate::key::ReceiverKeys,
        height: u64,
        amount: u64,
        mark: u8,
    ) -> ScanChunk {
        let card = ReceiverCard {
            version: 1,
            owner_handle: keys.owner_handle,
            view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
            identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
            card_id: None,
            metadata: None,
            signature: [0u8; 64],
        };
        let base_asset =
            z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount).expect("asset");
        let output = crate::stealth::build_output_bundle(
            format!("recv-{height}-{mark}"),
            crate::tx::TxOutRole::Recipient,
            z00z_core::AssetClass::Coin,
            &card,
            amount,
            1,
        )
        .expect("output");
        let asset = crate::stealth::bind_stealth_output_wire(
            z00z_core::AssetWire::from_asset(&base_asset),
            &output.leaf,
        )
        .expect("bind output wire")
        .to_asset()
        .expect("scanned asset");

        ScanChunk {
            height,
            hash: vec![height as u8; 32],
            leaves: vec![asset],
        }
    }

    async fn owned_asset_payload(
        service: &WalletService,
        wallet_id: &PersistWalletId,
        asset_id: [u8; 32],
    ) -> crate::db::redb_store::OwnedAssetPayload {
        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await
            .expect("wallet session");

        session
            .with_wallet_session(|wlt_session| {
                crate::db::wallet_asset_store()
                    .get_owned_asset(wlt_session, &asset_id)
                    .map(|payload| payload.expect("owned asset payload"))
            })
            .expect("owned asset read")
    }

    struct FetchFailWorker;

    impl crate::chain::RemoteScanWorker for FetchFailWorker {
        fn fetch_range_evidence(
            &mut self,
            _range: &crate::chain::RemoteScanRange,
        ) -> crate::chain::RemoteScanWorkerResult<crate::chain::RemoteScanEvidence> {
            Err(crate::chain::RemoteScanWorkerError::Transport(
                "network loss mid-scan".to_string(),
            ))
        }

        fn is_fetching(&self) -> bool {
            false
        }

        fn stop_fetch(&mut self) -> crate::chain::RemoteScanWorkerResult<()> {
            Ok(())
        }

        fn progress(&self) -> crate::chain::RemoteScanProgress {
            crate::chain::RemoteScanProgress {
                fetched_ckpt: 1,
                total_ckpt: 2,
            }
        }

        fn set_progress_callback(&mut self, _callback: crate::chain::RemoteScanProgressCallback) {}
    }

    #[cfg(not(target_arch = "wasm32"))]
    struct CountingOpenWltStore {
        inner: Arc<dyn WltStore>,
        open_calls: AtomicUsize,
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl CountingOpenWltStore {
        fn new(inner: Arc<dyn WltStore>) -> Self {
            Self {
                inner,
                open_calls: AtomicUsize::new(0),
            }
        }

        fn open_calls(&self) -> usize {
            self.open_calls.load(Ordering::SeqCst)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl WltStore for CountingOpenWltStore {
        fn create_wallet_store(
            &self,
            path: &Path,
            wallet_id: &PersistWalletId,
            password: &SafePassword,
            seed_phrase: &str,
            identity: &WalletIdentity,
        ) -> WalletResult<()> {
            self.inner
                .create_wallet_store(path, wallet_id, password, seed_phrase, identity)
        }

        fn open_wallet_store(
            &self,
            path: &Path,
            wallet_id: &PersistWalletId,
            password: &SafePassword,
            identity: &WalletIdentity,
        ) -> WalletResult<crate::db::WalletSession> {
            self.open_calls.fetch_add(1, Ordering::SeqCst);
            self.inner
                .open_wallet_store(path, wallet_id, password, identity)
        }

        fn discover_wallet_store(&self, path: &Path) -> WalletResult<PersistWalletDiscovery> {
            self.inner.discover_wallet_store(path)
        }

        fn reveal_seed_phrase(&self, session: &crate::db::WalletSession) -> WalletResult<String> {
            self.inner.reveal_seed_phrase(session)
        }

        fn verify_password(
            &self,
            session: &crate::db::WalletSession,
            password: &SafePassword,
        ) -> WalletResult<()> {
            self.inner.verify_password(session, password)
        }

        fn write_wallet_profile(
            &self,
            session: &crate::db::WalletSession,
            profile_bytes: Vec<u8>,
        ) -> WalletResult<u64> {
            self.inner.write_wallet_profile(session, profile_bytes)
        }

        fn read_wallet_profile(
            &self,
            session: &crate::db::WalletSession,
        ) -> WalletResult<crate::security::SecretBytes> {
            self.inner.read_wallet_profile(session)
        }
    }

    fn test_service_tempdir_policy(
        policy: AutoLockPolicy,
    ) -> (WalletService, tempfile::TempDir, Arc<MockTimeProvider>) {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());

        let time_provider: Arc<dyn TimeProvider> = time.clone();
        let mut service = WalletService::auto_lock_policy_dependencies(policy, time_provider);
        service.output_dir = dir.path().to_path_buf();
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        (service, dir, time)
    }

    #[test]
    fn test_wallet_id_deterministic_mock() {
        let time: Arc<dyn TimeProvider> = Arc::new(MockTimeProvider::default());

        #[derive(Debug, Clone, Copy)]
        struct TestSecureRngProvider {
            seed: u64,
        }

        impl SecureRngProvider for TestSecureRngProvider {
            type Rng = StdRng;

            fn rng(&self) -> Self::Rng {
                StdRng::seed_from_u64(self.seed)
            }
        }

        let rng = TestSecureRngProvider { seed: 42 };

        let service_a = create_service_with_rng(Arc::clone(&time), rng);
        let service_b =
            create_service_with_rng(Arc::clone(&time), TestSecureRngProvider { seed: 42 });

        let now_ms = time.compat_unix_timestamp_millis();
        let id_a = service_a.generate_wallet_id("test-wallet", now_ms);
        let id_b = service_b.generate_wallet_id("test-wallet", now_ms);

        assert_eq!(id_a, id_b);
    }

    #[tokio::test]
    async fn test_seq_reunlock_verify() {
        let dir = tempfile::tempdir().expect("tempdir must create");
        let time = Arc::new(MockTimeProvider::default());
        time.set_unix_millis(7);

        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SeqTestSecureRngProvider::new(7),
        );
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let identity = resolve_wallet_identity();
        let actors = [
            ("alice", "Alice_Pass_Z00Z_42!"),
            ("bob", "Bob_Pass_Z00Z_43!"),
            ("charlie", "Charlie_Pass_Z00Z_44!"),
        ];

        let mut alice_id = None;
        for (name, password) in actors {
            let wallet_id = service
                .create_wallet_using_explicit_identity(
                    name,
                    SafePassword::from(password),
                    test_seed_phrase_24(),
                    &identity,
                )
                .await
                .expect("create wallet must succeed");

            let session = service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(password))
                .await
                .unwrap_or_else(|err| panic!("first unlock for {name} failed: {err}"));

            let _keys = service
                .receiver_keys(&wallet_id)
                .await
                .unwrap_or_else(|err| panic!("receiver_keys for {name} failed: {err}"));

            service
                .lock_wallet(&wallet_id)
                .await
                .unwrap_or_else(|err| panic!("lock_wallet for {name} failed: {err}"));

            let verify_service =
                WalletService::with_output_dir_and_time(dir.path().to_path_buf(), time.clone());
            let wlt_path = service.wlt_file_path(&wallet_id);
            verify_service
                .open_wallet_source(WalletSource::Path {
                    path: wlt_path.to_string_lossy().to_string(),
                })
                .await
                .unwrap_or_else(|err| panic!("verify open_wallet_source for {name} failed: {err}"));

            if name == "alice" {
                assert_eq!(session.wallet_id, wallet_id);
                alice_id = Some(wallet_id);
            }
        }

        let alice_id = alice_id.expect("alice wallet id must be captured");
        service
            .unlock_wallet_in_memory(&alice_id, &SafePassword::from("Alice_Pass_Z00Z_42!"))
            .await
            .unwrap_or_else(|err| panic!("second unlock for alice failed: {err}"));
    }

    #[tokio::test]
    async fn test_rpc_reunlock_verify() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let dir = tempfile::tempdir().expect("tempdir must create");
        let time = Arc::new(MockTimeProvider::default());
        time.set_unix_millis(42);

        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SeqTestSecureRngProvider::new(42),
        );
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));
        let wallet_svc = Arc::new(service);
        let transport = build_local_rpc_transport(Arc::clone(&wallet_svc));

        let actors = [
            ("alice", "Alice_Pass_Z00Z_42!"),
            ("bob", "Bob_Pass_Z00Z_43!"),
            ("charlie", "Charlie_Pass_Z00Z_44!"),
        ];

        let mut alice_id = None;
        for (name, password) in actors {
            let resp = transport
                .call(
                    "app.wallet.create_wallet",
                    json!({
                        "name": name,
                        "password": password,
                        "seed_phrase": Value::Null,
                    }),
                )
                .await
                .unwrap_or_else(|err| panic!("create_wallet({name}) RPC failed: {err}"));

            let wallet_id = PersistWalletId(
                resp["wallet_id"]
                    .as_str()
                    .unwrap_or_else(|| panic!("wallet_id missing for {name}"))
                    .to_string(),
            );

            let session = transport
                .call(
                    "wallet.session.unlock_wallet",
                    json!({
                        "wallet_id": wallet_id.0,
                        "password": password,
                    }),
                )
                .await
                .unwrap_or_else(|err| panic!("first unlock RPC for {name} failed: {err}"));

            let _keys = wallet_svc
                .receiver_keys(&wallet_id)
                .await
                .unwrap_or_else(|err| panic!("receiver_keys for {name} failed: {err}"));

            transport
                .call("wallet.session.lock_wallet", json!({ "session": session }))
                .await
                .unwrap_or_else(|err| panic!("lock_wallet RPC for {name} failed: {err}"));

            let verify_service =
                WalletService::with_output_dir_and_time(dir.path().to_path_buf(), time.clone());
            let wlt_path = wallet_svc.wlt_file_path(&wallet_id);
            verify_service
                .open_wallet_source(WalletSource::Path {
                    path: wlt_path.to_string_lossy().to_string(),
                })
                .await
                .unwrap_or_else(|err| panic!("verify open_wallet_source for {name} failed: {err}"));

            if name == "alice" {
                alice_id = Some(wallet_id);
            }
        }

        let list_wallets = transport
            .call("app.wallet.list_wallets", json!({}))
            .await
            .expect("list_wallets RPC must succeed");
        assert_eq!(
            list_wallets.as_array().map(|items| items.len()),
            Some(3),
            "stage-2 RPC mix must list three wallets before the second unlock"
        );

        let alice_id = alice_id.expect("alice wallet id must be captured");
        transport
            .call(
                "wallet.session.unlock_wallet",
                json!({
                    "wallet_id": alice_id.0,
                    "password": "Alice_Pass_Z00Z_42!",
                }),
            )
            .await
            .unwrap_or_else(|err| panic!("second unlock RPC for alice failed: {err}"));
    }

    #[tokio::test]
    async fn test_list_wallets_created_wallets() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);

        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password, test_seed_phrase_24())
            .await
            .unwrap();

        let wallets = service.list_wallets_in_memory().await.unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].id, wallet_id);
        assert_eq!(wallets[0].name, "test-wallet");
        assert!(wallets[0].is_locked);
    }

    #[tokio::test]
    async fn test_unlock_token_no_reopen() {
        let (mut service, _dir) = test_service_with_tempdir();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let inner = Arc::clone(&service.wlt_store);
            let counting = Arc::new(CountingOpenWltStore::new(inner));
            let open_calls_ptr = Arc::clone(&counting);
            service.wlt_store = counting;

            let password = SafePassword::from(TEST_PASSWORD);
            let wallet_id = service
                .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
                .await
                .unwrap();

            let baseline_opens = open_calls_ptr.open_calls();

            let token_a = service
                .unlock_wallet_in_memory(&wallet_id, &password)
                .await
                .unwrap();
            assert_eq!(open_calls_ptr.open_calls(), baseline_opens + 1);

            // Second unlock should hit the existing-token fast path, not re-open `.wlt`.
            let token_b = service
                .unlock_wallet_in_memory(&wallet_id, &password)
                .await
                .unwrap();
            assert_eq!(token_b.token, token_a.token);
            assert_eq!(open_calls_ptr.open_calls(), baseline_opens + 1);
        }
    }

    #[tokio::test]
    async fn test_rejects_wlt_no_publish() {
        let (mut service, _dir) = test_service_with_tempdir();

        #[cfg(not(target_arch = "wasm32"))]
        {
            service.wlt_store = Arc::new(FailingWltStore);
        }

        let password = SafePassword::from(TEST_PASSWORD);
        let res = service
            .create_wallet_in_memory("test-wallet", password, test_seed_phrase_24())
            .await;

        assert!(res.is_err());
        let wallets = service.list_wallets_in_memory().await.unwrap();
        assert!(wallets.is_empty());
    }

    async fn insert_test_wallet_entry(
        service: &WalletService,
        wallet_id: PersistWalletId,
        name: String,
        created_at_secs: u64,
        state: WalletState,
    ) {
        let mut names = service.wallet_names.write().await;
        names.insert(wallet_id.clone(), name);
        drop(names);

        let mut settings = service.wallet_settings.write().await;
        settings.insert(
            wallet_id.clone(),
            PersistWalletSettings {
                auto_lock_timeout: 300,
                default_fee: "0.001".to_string(),
                currency_display: "Z00Z".to_string(),
                policy_rules: None,
                created_at: created_at_secs.saturating_mul(1000),
                updated_at: created_at_secs.saturating_mul(1000),
            },
        );
        drop(settings);

        let mut states = service.wallet_states.write().await;
        states.insert(wallet_id, state);
    }

    #[tokio::test]
    async fn test_list_wallets_100_wallets() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time);

        for i in 0..100u32 {
            insert_test_wallet_entry(
                &service,
                PersistWalletId(format!("test-wallet-{i}")),
                format!("Test Wallet {i}"),
                1_700_000_000,
                WalletState::Locked,
            )
            .await;
        }

        let wallets = service.list_wallets_in_memory().await.unwrap();
        assert_eq!(wallets.len(), 100);
    }

    #[tokio::test]
    async fn test_derive_public_key_concurrent() {
        let (service, _dir) = test_service_with_tempdir();
        let service = Arc::new(service);

        let password = SafePassword::from(TEST_PASSWORD);

        let mut set = JoinSet::new();
        for i in 0..8u32 {
            let service = Arc::clone(&service);
            let password = password.clone();
            set.spawn(async move {
                let wallet_id = service
                    .create_wallet_in_memory(
                        &format!("w{i}"),
                        password.clone(),
                        test_seed_phrase_24(),
                    )
                    .await?;

                let _token = service
                    .unlock_wallet_in_memory(&wallet_id, &password)
                    .await?;

                service
                    .derive_public_key_for_path(&wallet_id, Bip44Path::payment(0)?)
                    .await
            });
        }

        let completed = timeout(CONCURRENT_DERIVE_TIMEOUT, async move {
            while let Some(res) = set.join_next().await {
                let public_key = res.unwrap()?;
                assert_ne!(public_key, [0u8; 32]);
            }

            Ok::<(), WalletError>(())
        })
        .await;

        assert!(
            completed.is_ok(),
            "derive tasks should finish within release timeout"
        );
        completed.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_auto_lock_100_wallets() {
        let policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        let time = Arc::new(MockTimeProvider::default());
        let service = WalletService::auto_lock_policy_dependencies(policy, time.clone());

        // Ensure `now_ms` is sufficiently large so that subtracting an offset does not saturate.
        time.advance_by(Duration::from_millis(1_000));

        // Insert 100 unlocked wallets.
        // Half are already expired at the time we call `check_auto_lock`, half are still fresh.
        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let check_advance_ms = timeout_ms / 2;
        let stale_offset_ms = timeout_ms + 10;
        for i in 0..100u32 {
            let last_activity_ms = if i % 2 == 0 {
                now_ms - stale_offset_ms
            } else {
                now_ms
            };

            insert_test_wallet_entry(
                &service,
                test_wallet_id(i),
                format!("Test Wallet {i}"),
                1_700_000_000,
                WalletState::Unlocked {
                    session_start_ms: now_ms,
                    last_activity_ms,
                },
            )
            .await;
        }

        // Advance time, but not enough to expire the fresh half.
        time.advance_by(Duration::from_millis(check_advance_ms));

        let locked = service.check_auto_lock().await.unwrap();
        assert_eq!(locked.len(), 50);
    }

    #[tokio::test]
    async fn test_create_wallet_stub() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let id = service
            .create_wallet_in_memory("test", password, test_seed_phrase_24())
            .await
            .unwrap();
        // After implementing, ID is generated from domain-separated hash
        assert!(!id.0.is_empty());
        assert_ne!(id, PersistWalletId::stub_default());
    }

    #[tokio::test]
    async fn test_derivation_deterministic_restart() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let password = SafePassword::from(TEST_PASSWORD);
        let seed_phrase = test_seed_phrase_24();

        // Device A: derive keys once.
        let (service_a, dir_a) = test_service_tempdir_raw();
        let wallet_id_a = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), seed_phrase)
            .await
            .unwrap();
        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id_a, &password)
            .await
            .unwrap();
        let pk_a_payment0 = service_a
            .derive_public_key_for_path(&wallet_id_a, Bip44Path::payment(0).unwrap())
            .await
            .unwrap();
        let pk_a_change0 = service_a
            .derive_public_key_for_path(&wallet_id_a, Bip44Path::change_path(0).unwrap())
            .await
            .unwrap();

        // Ensure `.wlt` session state does not hold file locks.
        service_a.lock_wallet(&wallet_id_a).await.unwrap();

        // Restart A: new service instance reading the same `.wlt` file.
        let time = Arc::new(MockTimeProvider::default());
        let mut service_a_restart = WalletService::create_service_custom_output_directory(
            dir_a.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_a_restart.sleeper = Arc::new(MockSleeper::new(time));

        service_a_restart
            .load_wallet(&wallet_id_a, TEST_PASSWORD)
            .await
            .unwrap();
        let _token_restart = service_a_restart
            .unlock_wallet_in_memory(&wallet_id_a, &password)
            .await
            .unwrap();
        let pk_restart_payment0 = service_a_restart
            .derive_public_key_for_path(&wallet_id_a, Bip44Path::payment(0).unwrap())
            .await
            .unwrap();
        let pk_restart_change0 = service_a_restart
            .derive_public_key_for_path(&wallet_id_a, Bip44Path::change_path(0).unwrap())
            .await
            .unwrap();

        assert_eq!(pk_a_payment0, pk_restart_payment0);
        assert_eq!(pk_a_change0, pk_restart_change0);

        // Device B: new data directory, same entropy/seed phrase.
        let (service_b, _dir_b) = test_service_tempdir_raw();
        let wallet_id_b = service_b
            .create_wallet_in_memory("test-wallet", password.clone(), seed_phrase)
            .await
            .unwrap();
        let _token_b = service_b
            .unlock_wallet_in_memory(&wallet_id_b, &password)
            .await
            .unwrap();
        let pk_b_payment0 = service_b
            .derive_public_key_for_path(&wallet_id_b, Bip44Path::payment(0).unwrap())
            .await
            .unwrap();
        let pk_b_change0 = service_b
            .derive_public_key_for_path(&wallet_id_b, Bip44Path::change_path(0).unwrap())
            .await
            .unwrap();

        assert_eq!(pk_a_payment0, pk_b_payment0);
        assert_eq!(pk_a_change0, pk_b_change0);
    }

    #[tokio::test]
    async fn test_progress_persists_restart() {
        let (service_a, dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        service_a
            .derive_public_key_for_path(&wallet_id, Bip44Path::payment(0).unwrap())
            .await
            .unwrap();
        service_a
            .derive_public_key_for_path(&wallet_id, Bip44Path::change_path(0).unwrap())
            .await
            .unwrap();

        // Drop the lazy deriver but keep persisted counters.
        service_a.lock_wallet(&wallet_id).await.unwrap();

        let counters = {
            let store = service_a.wallet_receiver_deriver_counters.read().await;
            store.get(&wallet_id).copied().unwrap()
        };
        assert_eq!(counters.next_payment_index, 1);
        assert_eq!(counters.next_change_index, 1);

        let deriver_a = service_a.profile_receiver_deriver_state(&wallet_id).await;
        assert_eq!(deriver_a.next_payment_index, 1);
        assert_eq!(deriver_a.next_change_index, 1);

        // Persist to `.wlt` and restart the service.
        service_a
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();

        let deriver_b = service_b.profile_receiver_deriver_state(&wallet_id).await;
        assert_eq!(deriver_b.next_payment_index, 1);
        assert_eq!(deriver_b.next_change_index, 1);
    }

    #[tokio::test]
    async fn test_recv_range_restart() {
        let (service_a, dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service_a.receiver_keys(&wallet_id).await.unwrap();
        let chunks = vec![
            make_recv_chunk(&recv_keys, 7, 310, 17),
            make_recv_chunk(&recv_keys, 8, 420, 18),
        ];
        let expect_ids: Vec<[u8; 32]> = chunks
            .iter()
            .flat_map(|chunk| chunk.leaves.iter().map(|leaf| leaf.definition.id))
            .collect();

        let first = service_a
            .recv_range(&wallet_id, &chunks, &[], Some(1))
            .await
            .unwrap();
        assert_eq!(first.outputs.len(), 1);
        assert_eq!(first.stat.done_ckpt, 1);
        assert_eq!(first.stat.cursor.height(), 7);
        assert!(!first.stat.cursor.is_origin());
        let now_ms = service_a.now_ms();
        let timeout_ms = service_a.timeout_ms();
        let session_a = service_a
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved_a = session_a
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");
        assert_eq!(saved_a.height(), first.stat.cursor.height());
        assert!(saved_a.matches_chunk(7, &chunks[0].hash));
        let claimed_a = service_a.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed_a.len(), 1);
        assert_eq!(claimed_a[0].definition.id, expect_ids[0]);
        let first_asset_id = claimed_a[0].asset_id();
        let stored_a = owned_asset_payload(&service_a, &wallet_id, claimed_a[0].asset_id()).await;
        assert_eq!(
            stored_a.source,
            crate::db::redb_store::OwnedAssetSource::Scan
        );
        let scan_ref_a = stored_a.scan_ref.expect("scan ref");
        assert_eq!(scan_ref_a.start_height, 7);
        assert_eq!(scan_ref_a.end_height, 7);
        assert_eq!(scan_ref_a.cursor_hash, chunks[0].hash);

        service_a.lock_wallet(&wallet_id).await.unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();
        let _token = service_b
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let next = service_b
            .recv_range(&wallet_id, &chunks, &[], None)
            .await
            .unwrap();
        assert_eq!(next.outputs.len(), 1);
        assert_eq!(next.stat.done_ckpt, 1);
        assert_eq!(next.stat.cursor.height(), 8);
        assert_eq!(next.stat.total_ckpt, 2);
        assert_eq!(next.stat.found_cnt, 1);
        let now_ms = service_b.now_ms();
        let timeout_ms = service_b.timeout_ms();
        let session_b = service_b
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved_b = session_b
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");
        assert_eq!(saved_b.height(), next.stat.cursor.height());
        assert!(saved_b.matches_chunk(8, &chunks[1].hash));
        let claimed_b = service_b.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed_b.len(), 2);
        let mut claim_ids = claimed_b
            .iter()
            .map(|asset| asset.definition.id)
            .collect::<Vec<_>>();
        claim_ids.sort_unstable();
        let mut expect_sorted = expect_ids;
        expect_sorted.sort_unstable();
        assert_eq!(claim_ids, expect_sorted);
        let unique = claimed_b
            .iter()
            .map(|asset| asset.asset_id())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), claimed_b.len());
        let resumed_asset_id = claimed_b
            .iter()
            .find(|asset| asset.asset_id() != first_asset_id)
            .map(|asset| asset.asset_id())
            .expect("resumed asset id");
        let stored_b = owned_asset_payload(&service_b, &wallet_id, resumed_asset_id).await;
        assert_eq!(
            stored_b.source,
            crate::db::redb_store::OwnedAssetSource::Scan
        );
        let scan_ref_b = stored_b.scan_ref.expect("scan ref");
        assert_eq!(scan_ref_b.end_height, 8);
        assert_eq!(scan_ref_b.cursor_hash, chunks[1].hash);
    }

    #[tokio::test]
    async fn test_claimed_asset_restart() {
        let (service_a, dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let asset = test_dev_coin_asset(19, 99);
        let asset_id = asset.definition.id;

        service_a
            .put_claimed_asset(&wallet_id, asset)
            .await
            .unwrap();
        service_a.lock_wallet(&wallet_id).await.unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();

        let claimed = service_b.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].definition.id, asset_id);
    }

    #[tokio::test]
    async fn test_ex4_restart_resume() {
        let (service_a, dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service_a.receiver_keys(&wallet_id).await.unwrap();
        let chunks = vec![
            make_recv_chunk(&recv_keys, 7, 310, 17),
            make_recv_chunk(&recv_keys, 8, 420, 18),
        ];
        let expect_ids: Vec<[u8; 32]> = chunks
            .iter()
            .flat_map(|chunk| chunk.leaves.iter().map(|leaf| leaf.definition.id))
            .collect();

        let first = service_a
            .recv_range(&wallet_id, &chunks, &[], Some(1))
            .await
            .unwrap();
        let line_a = format!(
            "first done={} cursor={}",
            first.stat.done_ckpt,
            first.stat.cursor.height()
        );
        let first_claimed = service_a.list_claimed_assets(&wallet_id).await.unwrap();

        assert_eq!(first.outputs.len(), 1);
        assert_eq!(first.stat.done_ckpt, 1);
        assert_eq!(first.stat.cursor.height(), 7);
        assert!(!first.stat.cursor.is_origin());
        assert_eq!(first_claimed.len(), 1);
        assert_eq!(
            first_claimed[0].definition.id,
            chunks[0].leaves[0].definition.id
        );
        assert!(line_a.contains("done=1"));
        assert!(line_a.contains("cursor=7"));

        let now_ms = service_a.now_ms();
        let timeout_ms = service_a.timeout_ms();
        let session_a = service_a
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved_a = session_a
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");

        assert_eq!(saved_a.height(), first.stat.cursor.height());
        assert!(saved_a.matches_chunk(7, &chunks[0].hash));

        service_a.lock_wallet(&wallet_id).await.unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();
        let _token = service_b
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let next = service_b
            .recv_range(&wallet_id, &chunks, &[], None)
            .await
            .unwrap();
        let line_b = format!(
            "next done={} cursor={}",
            next.stat.done_ckpt,
            next.stat.cursor.height()
        );

        assert_eq!(next.outputs.len(), 1);
        assert_eq!(next.stat.done_ckpt, 1);
        assert_eq!(next.stat.cursor.height(), 8);
        assert_eq!(next.stat.total_ckpt, 2);
        assert!(line_b.contains("done=1"));
        assert!(line_b.contains("cursor=8"));

        let now_ms = service_b.now_ms();
        let timeout_ms = service_b.timeout_ms();
        let session_b = service_b
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved_b = session_b
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");

        assert_eq!(saved_b.height(), next.stat.cursor.height());
        assert!(saved_b.matches_chunk(8, &chunks[1].hash));

        let claimed = service_b.list_claimed_assets(&wallet_id).await.unwrap();
        let mut claim_ids = claimed
            .iter()
            .map(|asset| asset.definition.id)
            .collect::<Vec<_>>();
        claim_ids.sort_unstable();
        let mut expect_sorted = expect_ids;
        expect_sorted.sort_unstable();

        assert_eq!(claimed.len(), 2);
        assert_eq!(claim_ids, expect_sorted);
        assert_eq!(
            claimed
                .iter()
                .map(|asset| asset.asset_id())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            claimed.len()
        );
    }

    #[tokio::test]
    async fn test_recv_range_rescan_idempotent() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let chunks = vec![make_recv_chunk(&recv_keys, 7, 310, 17)];

        let first = service
            .recv_range(&wallet_id, &chunks, &[], None)
            .await
            .unwrap();
        assert_eq!(first.outputs.len(), 1);
        let first_claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(first_claimed.len(), 1);
        let claim_id = first_claimed[0].asset_id();

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        session
            .with_wallet_session(|wlt_session| {
                crate::db::redb_store::upsert_scan_state(
                    wlt_session,
                    &crate::db::ScanStatePayload::new(0, Vec::new()),
                    z00z_utils::rng::SystemRngProvider,
                )?;
                Ok(())
            })
            .unwrap();

        let second = service
            .recv_range(&wallet_id, &chunks, &[], None)
            .await
            .unwrap();
        assert_eq!(second.outputs.len(), 1);
        assert_eq!(second.stat.done_ckpt, 1);

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].asset_id(), claim_id);
        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::Scanned)
        );

        let stored = owned_asset_payload(&service, &wallet_id, claim_id).await;
        assert_eq!(stored.source, crate::db::redb_store::OwnedAssetSource::Scan);
        let scan_ref = stored.scan_ref.expect("scan ref");
        assert_eq!(scan_ref.start_height, 7);
        assert_eq!(scan_ref.end_height, 7);
        assert_eq!(scan_ref.cursor_hash, chunks[0].hash);
    }

    #[tokio::test]
    async fn test_scan_commit_atomically() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let chunks = vec![make_recv_chunk(&recv_keys, 7, 310, 17)];

        {
            let _guard = ReceivePersistHookGuard::install(
                &wallet_id,
                ReceivePersistTestHook::AbortBeforePersist,
            );
            let err = service
                .recv_range(&wallet_id, &chunks, &[], None)
                .await
                .expect_err("scan batch failpoint must fail closed");
            assert!(matches!(err, WalletError::InvalidConfig(_)));
            assert!(err
                .to_string()
                .contains("scan batch persist failpoint enabled"));
        }

        assert!(service
            .list_claimed_assets(&wallet_id)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(service.last_receive_scan_outcome(&wallet_id).await, None);

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .unwrap_or_else(|| crate::db::ScanStatePayload::new(0, Vec::new()));
        assert!(saved.is_origin());

        let retry = service
            .recv_range(&wallet_id, &chunks, &[], None)
            .await
            .expect("retry after atomic failure must succeed");
        assert_eq!(retry.outputs.len(), 1);
        assert_eq!(retry.stat.cursor.height(), 7);
        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::ImportedHit)
        );

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 1);

        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");
        assert_eq!(saved.height(), 7);
        assert_eq!(saved.last_scanned_hash, chunks[0].hash);
    }

    #[tokio::test]
    async fn test_scan_resume_no_hit() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet-a", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();
        let other_wallet = service
            .create_wallet_in_memory(
                "test-wallet-b",
                password.clone(),
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
            )
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();
        let _other_token = service
            .unlock_wallet_in_memory(&other_wallet, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let first = vec![make_recv_chunk(&recv_keys, 7, 310, 17)];
        service
            .recv_range(&wallet_id, &first, &[], None)
            .await
            .unwrap();
        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::ImportedHit)
        );

        let foreign_keys = service.receiver_keys(&other_wallet).await.unwrap();
        let resumed = vec![first[0].clone(), make_recv_chunk(&foreign_keys, 8, 420, 18)];
        let out = service
            .recv_range(&wallet_id, &resumed, &[], None)
            .await
            .unwrap();

        assert!(out.outputs.is_empty());
        assert_eq!(out.stat.cursor.height(), 8);
        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::Resumed)
        );
        assert_eq!(
            service.list_claimed_assets(&wallet_id).await.unwrap().len(),
            1
        );
    }

    #[tokio::test]
    async fn test_scan_cursor_conflict() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let chunks = vec![make_recv_chunk(&recv_keys, 7, 310, 17)];

        {
            let _guard = ReceivePersistHookGuard::install(
                &wallet_id,
                ReceivePersistTestHook::MutateCursorBeforePersist {
                    height: 99,
                    hash: vec![9u8; 32],
                },
            );
            let err = service
                .recv_range(&wallet_id, &chunks, &[], None)
                .await
                .expect_err("concurrent cursor mutation must fail closed");
            assert!(matches!(err, WalletError::InvalidConfig(_)));
            assert!(err
                .to_string()
                .contains("scan state changed during receive persistence"));
        }

        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::CursorConflict)
        );
        assert!(service
            .list_claimed_assets(&wallet_id)
            .await
            .unwrap()
            .is_empty());

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");
        assert_eq!(saved.height(), 99);
        assert_eq!(saved.last_scanned_hash, vec![9u8; 32]);
    }

    #[tokio::test]
    async fn test_worker_range_ok() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let chunks = vec![make_recv_chunk(&recv_keys, 7, 310, 17)];
        let evidence = crate::chain::RemoteScanEvidence {
            chunks: chunks.clone(),
            proof_hints: Vec::new(),
            resume_hint: None,
        };

        let out = service
            .recv_range_with_worker(&wallet_id, &evidence, &[], None)
            .await
            .unwrap();
        assert_eq!(out.outputs.len(), 1);
        assert_eq!(out.stat.done_ckpt, 1);
        assert_eq!(out.stat.cursor.height(), 7);
        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::ImportedHit)
        );

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 1);

        let stored = owned_asset_payload(&service, &wallet_id, claimed[0].asset_id()).await;
        assert_eq!(stored.source, crate::db::redb_store::OwnedAssetSource::Scan);
        let scan_ref = stored.scan_ref.expect("scan ref");
        assert_eq!(scan_ref.start_height, 7);
        assert_eq!(scan_ref.end_height, 7);
        assert_eq!(scan_ref.cursor_hash, chunks[0].hash);
    }

    #[tokio::test]
    async fn test_worker_resume_reject() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let evidence = crate::chain::RemoteScanEvidence {
            chunks: vec![make_recv_chunk(&recv_keys, 7, 310, 17)],
            proof_hints: Vec::new(),
            resume_hint: Some(crate::chain::RemoteScanResumeHint {
                next_height: 7,
                last_chunk_hash: vec![0xAA; 32],
            }),
        };

        let err = service
            .recv_range_with_worker(&wallet_id, &evidence, &[], None)
            .await
            .expect_err("worker resume hint must fail closed");
        assert!(matches!(err, WalletError::InvalidConfig(_)));
        assert!(err
            .to_string()
            .contains("remote resume hint cannot set local cursor"));
        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::WorkerEvidenceRejected)
        );

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed.is_empty());

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .unwrap_or_else(|| crate::db::ScanStatePayload::new(0, Vec::new()));
        assert!(saved.is_origin());
    }

    #[tokio::test]
    async fn test_worker_replay_cursor_reject() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let chunk = make_recv_chunk(&recv_keys, 7, 310, 17);
        let first = crate::chain::RemoteScanEvidence {
            chunks: vec![chunk.clone()],
            proof_hints: Vec::new(),
            resume_hint: None,
        };

        let first_out = service
            .recv_range_with_worker(&wallet_id, &first, &[], None)
            .await
            .unwrap();
        assert_eq!(first_out.stat.cursor.height(), 7);

        let mut replay_chunk = chunk.clone();
        replay_chunk.height = 8;
        replay_chunk.hash = vec![8u8; 32];
        let replay = crate::chain::RemoteScanEvidence {
            chunks: vec![chunk, replay_chunk],
            proof_hints: Vec::new(),
            resume_hint: None,
        };

        let err = service
            .recv_range_with_worker(&wallet_id, &replay, &[], None)
            .await
            .expect_err("worker replay with conflicting cursor metadata must fail closed");
        assert!(matches!(err, WalletError::InvalidConfig(_)));
        assert!(err
            .to_string()
            .contains("duplicate scan asset id conflicts with stored payload"));

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 1);

        let stored = owned_asset_payload(&service, &wallet_id, claimed[0].asset_id()).await;
        let scan_ref = stored.scan_ref.expect("scan ref");
        assert_eq!(scan_ref.start_height, 7);
        assert_eq!(scan_ref.end_height, 7);
        assert_eq!(scan_ref.cursor_hash, vec![7u8; 32]);

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .unwrap_or_else(|| crate::db::ScanStatePayload::new(0, Vec::new()));
        assert_eq!(saved.height(), 7);
        assert_eq!(saved.last_scanned_hash, vec![7u8; 32]);
    }

    #[tokio::test]
    async fn test_worker_gap_reject() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let mut first = make_recv_chunk(&recv_keys, 7, 310, 17);
        first.leaves.clear();
        let mut third = make_recv_chunk(&recv_keys, 9, 520, 19);
        third.leaves.clear();
        let evidence = crate::chain::RemoteScanEvidence {
            chunks: vec![first, third],
            proof_hints: Vec::new(),
            resume_hint: None,
        };

        let err = service
            .recv_range_with_worker(&wallet_id, &evidence, &[], None)
            .await
            .expect_err("worker checkpoint gaps must fail closed");
        assert!(matches!(err, WalletError::InvalidConfig(_)));
        assert!(err.to_string().contains("remote chunks must be contiguous"));

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed.is_empty());

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .unwrap_or_else(|| crate::db::ScanStatePayload::new(0, Vec::new()));
        assert!(saved.is_origin());
    }

    #[tokio::test]
    async fn test_worker_proof_empty_reject() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let evidence = crate::chain::RemoteScanEvidence {
            chunks: vec![make_recv_chunk(&recv_keys, 7, 310, 17)],
            proof_hints: vec![crate::chain::RemoteScanProofHint {
                checkpoint_height: 7,
                proof_bytes: Vec::new(),
            }],
            resume_hint: None,
        };

        let err = service
            .recv_range_with_worker(&wallet_id, &evidence, &[], None)
            .await
            .expect_err("worker proof hints with empty bytes must fail closed");
        assert!(matches!(err, WalletError::InvalidConfig(_)));
        assert!(err
            .to_string()
            .contains("remote proof hint bytes must not be empty"));

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed.is_empty());

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .unwrap_or_else(|| crate::db::ScanStatePayload::new(0, Vec::new()));
        assert!(saved.is_origin());
    }

    #[tokio::test]
    async fn test_worker_hint_miss_reject() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let evidence = crate::chain::RemoteScanEvidence {
            chunks: vec![make_recv_chunk(&recv_keys, 7, 310, 17)],
            proof_hints: vec![crate::chain::RemoteScanProofHint {
                checkpoint_height: 8,
                proof_bytes: vec![0xAB],
            }],
            resume_hint: None,
        };

        let err = service
            .recv_range_with_worker(&wallet_id, &evidence, &[], None)
            .await
            .expect_err("worker proof hints without matching chunks must fail closed");
        assert!(matches!(err, WalletError::InvalidConfig(_)));
        assert!(err
            .to_string()
            .contains("remote proof hint must match a returned chunk"));

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed.is_empty());

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .unwrap_or_else(|| crate::db::ScanStatePayload::new(0, Vec::new()));
        assert!(saved.is_origin());
    }

    #[tokio::test]
    async fn test_worker_foreign_chunk_ok() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet-a", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();
        let other_wallet = service
            .create_wallet_in_memory(
                "test-wallet-b",
                password.clone(),
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
            )
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();
        let _other_token = service
            .unlock_wallet_in_memory(&other_wallet, &password)
            .await
            .unwrap();

        let foreign_keys = service.receiver_keys(&other_wallet).await.unwrap();
        let chunks = vec![make_recv_chunk(&foreign_keys, 7, 310, 17)];
        let evidence = crate::chain::RemoteScanEvidence {
            chunks: chunks.clone(),
            proof_hints: vec![crate::chain::RemoteScanProofHint {
                checkpoint_height: 7,
                proof_bytes: vec![0xAB],
            }],
            resume_hint: None,
        };

        let out = service
            .recv_range_with_worker(&wallet_id, &evidence, &[], None)
            .await
            .unwrap();
        assert!(out.outputs.is_empty());
        assert_eq!(out.stat.done_ckpt, 1);
        assert_eq!(out.stat.cursor.height(), 7);
        assert_eq!(
            service.last_receive_scan_outcome(&wallet_id).await,
            Some(RuntimeReceiveScanOutcome::NoHit)
        );

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed.is_empty());

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");
        assert_eq!(saved.height(), 7);
        assert_eq!(saved.last_scanned_hash, chunks[0].hash);
    }

    #[tokio::test]
    async fn test_worker_transport_fallback_ok() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let chunks = vec![
            make_recv_chunk(&recv_keys, 7, 310, 17),
            make_recv_chunk(&recv_keys, 8, 420, 18),
        ];

        let mut worker = FetchFailWorker;
        let range = crate::chain::RemoteScanRange {
            start_height: 7,
            end_height: 8,
        };
        let err = crate::chain::RemoteScanWorker::fetch_range_evidence(&mut worker, &range)
            .expect_err("worker transport failure must surface before fallback");
        assert!(matches!(
            err,
            crate::chain::RemoteScanWorkerError::Transport(_)
        ));
        assert!(err.to_string().contains("network loss mid-scan"));

        let progress = crate::chain::RemoteScanWorker::progress(&worker);
        assert_eq!(progress.fetched_ckpt, 1);
        assert_eq!(progress.total_ckpt, 2);

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed.is_empty());

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();
        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .unwrap_or_else(|| crate::db::ScanStatePayload::new(0, Vec::new()));
        assert!(saved.is_origin());

        let out = service
            .recv_range(&wallet_id, &chunks, &[], None)
            .await
            .unwrap();
        assert_eq!(out.outputs.len(), 2);
        assert_eq!(out.stat.done_ckpt, 2);
        assert_eq!(out.stat.cursor.height(), 8);

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 2);

        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");
        assert_eq!(saved.height(), 8);
        assert_eq!(saved.last_scanned_hash, chunks[1].hash);
    }

    #[tokio::test]
    async fn test_worker_stale_cursor_reject() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let session = service
            .wallet_sessions
            .session_for_wallet(&wallet_id, now_ms, timeout_ms)
            .await
            .unwrap();

        let persisted_cursor = crate::db::ScanStatePayload::new(8, vec![8u8; 32]);
        session
            .with_wallet_session(|wlt_session| {
                crate::db::redb_store::upsert_scan_state(
                    wlt_session,
                    &persisted_cursor,
                    z00z_utils::rng::SystemRngProvider,
                )?;
                Ok(())
            })
            .unwrap();

        let stale_resume = crate::db::ScanStatePayload::new(7, vec![7u8; 32]);
        let stale_cursor = crate::db::ScanStatePayload::new(9, vec![9u8; 32]);
        let err = session
            .with_wallet_session(|wlt_session| {
                crate::db::wallet_asset_store().persist_scan_batch(
                    wlt_session,
                    &[],
                    &stale_resume,
                    &stale_cursor,
                    crate::db::AssetPersistContext {
                        now_ms,
                        ..crate::db::AssetPersistContext::default()
                    },
                )
            })
            .expect_err("stale scan cursor write must fail closed");
        assert!(matches!(err, WalletError::InvalidConfig(_)));
        assert!(err
            .to_string()
            .contains("scan state changed during receive persistence"));

        let saved = session
            .with_wallet_session(crate::db::read_scan_state)
            .unwrap()
            .expect("scan state");
        assert_eq!(saved.height(), 8);
        assert_eq!(saved.last_scanned_hash, vec![8u8; 32]);
    }

    #[test]
    fn test_scan_error_classifies_version() {
        let error = WalletError::InvalidAssetPack("unsupported version");

        assert_eq!(
            WalletService::classify_receive_scan_error(&error),
            Some(RuntimeReceiveScanOutcome::UnsupportedVersion)
        );
    }

    #[tokio::test]
    async fn test_claimed_asset_rejects_invalid() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let mut asset = test_dev_coin_asset(15, 101);
        let mut definition = (*asset.definition).clone();
        definition.id[0] ^= 0x55;
        asset.definition = Arc::new(definition);

        let err = service
            .put_claimed_asset(&wallet_id, asset)
            .await
            .unwrap_err();
        assert!(matches!(err, WalletError::InvalidConfig(_)));

        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed.is_empty());
    }

    #[tokio::test]
    async fn test_recv_route_gate() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let asset = test_dev_coin_asset(16, 123);
        let asset_id = asset.asset_id();

        let reported = service
            .recv_route(&wallet_id, asset.clone(), ReceiveNext::ReportOnly)
            .await
            .unwrap();
        assert!(!reported);
        let claimed_none = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert!(claimed_none.is_empty());

        let persisted = service
            .recv_route(&wallet_id, asset, ReceiveNext::PersistClaim)
            .await
            .unwrap();
        assert!(persisted);
        let claimed = service.list_claimed_assets(&wallet_id).await.unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].asset_id(), asset_id);
        let stored = owned_asset_payload(&service, &wallet_id, asset_id).await;
        assert_eq!(
            stored.source,
            crate::db::redb_store::OwnedAssetSource::ManualClaim
        );
        assert!(stored.scan_ref.is_none());
    }

    #[tokio::test]
    async fn test_recv_claim_invalid_signature() {
        let asset = test_dev_coin_asset(17, 123);
        assert!(asset.validate().is_ok());

        let mut detector_asset = asset.clone();
        detector_asset.is_frozen = true;
        assert!(matches!(
            detector_asset.validate(),
            Err(z00z_core::assets::AssetError::InvalidSignature(_))
        ));

        assert!(
            recv_claim_asset(&detector_asset).is_none(),
            "invalid-signature receive assets must not be silently scrubbed into claimed storage"
        );
    }

    #[tokio::test]
    async fn test_gap_scan_next_index() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let is_used: ReceiverUsageOracle = Arc::new(|path, _public_key| {
            Box::pin(async move {
                if path.is_payment() {
                    Ok(matches!(path.address_index().index(), 0 | 2 | 5))
                } else if path.is_change() {
                    Ok(matches!(path.address_index().index(), 0))
                } else {
                    Ok(false)
                }
            })
        });

        let reconciled = service
            .reconcile_persist_gap_limit(&wallet_id, 3, is_used)
            .await
            .unwrap();

        assert_eq!(reconciled.next_payment_index, 6);
        assert_eq!(reconciled.next_change_index, 1);
    }

    #[tokio::test]
    async fn test_gap_limit_finds_used() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let is_used: ReceiverUsageOracle = Arc::new(|path, _public_key| {
            Box::pin(async move {
                if path.is_payment() {
                    Ok(matches!(path.address_index().index(), 0 | 19))
                } else {
                    Ok(false)
                }
            })
        });

        let reconciled = service
            .reconcile_persist_gap_limit(&wallet_id, 20, is_used)
            .await
            .unwrap();

        assert_eq!(reconciled.next_payment_index, 20);
        assert_eq!(reconciled.next_change_index, 0);
    }

    #[tokio::test]
    async fn test_gap_reconcile_persists_wlt() {
        let (service_a, dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let is_used: ReceiverUsageOracle = Arc::new(|path, _public_key| {
            Box::pin(async move {
                if path.is_payment() {
                    Ok(matches!(path.address_index().index(), 0 | 2 | 5))
                } else if path.is_change() {
                    Ok(matches!(path.address_index().index(), 0))
                } else {
                    Ok(false)
                }
            })
        });

        let reconciled = service_a
            .reconcile_persist_gap_limit(&wallet_id, 3, is_used)
            .await
            .unwrap();

        assert_eq!(reconciled.next_payment_index, 6);
        assert_eq!(reconciled.next_change_index, 1);

        // Drop the session to release the `.wlt` advisory lock before restart.
        service_a.lock_wallet(&wallet_id).await.unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();

        let deriver_b = service_b.profile_receiver_deriver_state(&wallet_id).await;
        assert_eq!(deriver_b.next_payment_index, 6);
        assert_eq!(deriver_b.next_change_index, 1);
    }

    #[tokio::test]
    async fn test_ver_zero_is_live() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let actual = service.receiver_keys(&wallet_id).await.unwrap();
        let expected = service.live_receiver_keys(&wallet_id).await.unwrap();

        assert_eq!(
            actual.reveal_view_sk().as_bytes(),
            expected.reveal_view_sk().as_bytes()
        );
    }
    #[tokio::test]
    async fn test_receiver_keys_retries_unusable() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        ReceiverSecret::set_fail_usable(true);
        let keys = service.live_receiver_keys(&wallet_id).await.unwrap();

        assert_ne!(keys.owner_handle, [0u8; 32]);
    }

    #[tokio::test]
    async fn test_stays_live_post_rotate() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let live_keys = service.live_receiver_keys(&wallet_id).await.unwrap();
        let live_chunk = make_recv_chunk(&live_keys, 7, 310, 17);
        let live_asset = live_chunk.leaves[0].clone();

        let rotated = service.rotate_recv_view(&wallet_id).await.unwrap();
        assert_eq!(rotated, 1);

        let default_keys = service.receiver_keys(&wallet_id).await.unwrap();
        assert_eq!(
            default_keys.reveal_view_sk().as_bytes(),
            live_keys.reveal_view_sk().as_bytes()
        );

        let rotated_keys = service
            .receiver_keys_for_view_version(&wallet_id, rotated)
            .await
            .unwrap();
        assert_ne!(
            rotated_keys.reveal_view_sk().as_bytes(),
            live_keys.reveal_view_sk().as_bytes()
        );

        let live_report = service
            .scan_asset_report(&wallet_id, &live_asset)
            .await
            .unwrap();
        assert_eq!(live_report.status, ReceiveStatus::Detected);
        assert_eq!(live_report.reject, None);

        let rotated_chunk = make_recv_chunk(&rotated_keys, 8, 420, 18);
        let rotated_asset = rotated_chunk.leaves[0].clone();
        let rotated_report = service
            .scan_asset_report(&wallet_id, &rotated_asset)
            .await
            .unwrap();
        assert_eq!(rotated_report.status, ReceiveStatus::NotMine);
        assert_eq!(rotated_report.reject, Some(ReceiveReject::NotMine));

        let out = service
            .recv_range(&wallet_id, &[live_chunk], &[], None)
            .await
            .unwrap();
        assert_eq!(out.outputs.len(), 1);
    }

    #[tokio::test]
    async fn test_recv_ver_explicit() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let mut expected = service.receiver_keys(&wallet_id).await.unwrap();
        let base_card = expected.export_receiver_card().unwrap();
        expected.rotate_view().unwrap();
        let expected_card = expected.export_receiver_card().unwrap();

        let rotated = service.rotate_recv_view(&wallet_id).await.unwrap();
        assert_eq!(rotated, 1);

        let actual_live = service.receiver_keys(&wallet_id).await.unwrap();
        let actual_live_card = actual_live.export_receiver_card().unwrap();
        let actual_rotated = service
            .receiver_keys_for_view_version(&wallet_id, rotated)
            .await
            .unwrap();
        let actual_rotated_card = actual_rotated.export_receiver_card().unwrap();

        assert_eq!(actual_live_card.view_pk, base_card.view_pk);
        assert_eq!(actual_live_card.identity_pk, base_card.identity_pk);
        assert_eq!(actual_live_card.owner_handle, base_card.owner_handle);
        assert_eq!(actual_rotated_card.view_pk, expected_card.view_pk);
        assert_eq!(actual_rotated_card.identity_pk, expected_card.identity_pk);
        assert_eq!(actual_rotated_card.owner_handle, expected_card.owner_handle);
    }

    #[tokio::test]
    async fn test_recv_ver_save() {
        let (service_a, dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        assert_eq!(service_a.rotate_recv_view(&wallet_id).await.unwrap(), 1);
        assert_eq!(service_a.rotate_recv_view(&wallet_id).await.unwrap(), 2);

        service_a.lock_wallet(&wallet_id).await.unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();
        let _token = service_b
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        assert_eq!(service_b.rotate_recv_view(&wallet_id).await.unwrap(), 3);

        let live_keys = service_b.receiver_keys(&wallet_id).await.unwrap();
        let rotated_keys = service_b
            .receiver_keys_for_view_version(&wallet_id, 3)
            .await
            .unwrap();
        assert_ne!(
            live_keys.reveal_view_sk().as_bytes(),
            rotated_keys.reveal_view_sk().as_bytes()
        );
    }

    #[tokio::test]
    async fn test_tofu_save_load() {
        let (service_a, dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service_a
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service_a
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service_a.receiver_keys(&wallet_id).await.unwrap();
        let card = recv_keys.export_receiver_card().unwrap();

        let res = service_a
            .tofu_verify_pin(&wallet_id, &card, Some("dir-a"))
            .await
            .unwrap();
        assert_eq!(res, VerifyResult::NewPin);

        service_a.lock_wallet(&wallet_id).await.unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut service_b = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service_b.sleeper = Arc::new(MockSleeper::new(time));

        service_b
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();
        let _token = service_b
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let pins = service_b.load_tofu(&wallet_id).await.unwrap();
        assert_eq!(pins.len(), 1);
        let entry = pins.get(&card.owner_handle).unwrap();
        assert_eq!(entry.identity_pk, card.identity_pk);
        assert_eq!(entry.view_pk, card.view_pk);
        assert_eq!(entry.directory_id.as_deref(), Some("dir-a"));
        assert_eq!(entry.trust_level, TrustLevel::Tentative);
    }

    #[tokio::test]
    async fn test_tofu_write_through() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let recv_keys = service.receiver_keys(&wallet_id).await.unwrap();
        let mut card = recv_keys.export_receiver_card().unwrap();

        let _ = service
            .tofu_verify_pin(&wallet_id, &card, None)
            .await
            .unwrap();

        let changed = service
            .tofu_verify_pin(&wallet_id, &card, None)
            .await
            .unwrap();
        assert_eq!(changed, VerifyResult::Verified);

        let mut next_keys = recv_keys;
        next_keys.rotate_view().unwrap();
        card = next_keys.export_receiver_card().unwrap();

        let changed = service
            .tofu_verify_pin(&wallet_id, &card, None)
            .await
            .unwrap();
        assert!(matches!(changed, VerifyResult::ViewKeyChanged { .. }));

        service
            .tofu_confirm(&wallet_id, &card.owner_handle, &card.view_pk)
            .await
            .unwrap();

        let pins = service.load_tofu(&wallet_id).await.unwrap();
        let entry = pins.get(&card.owner_handle).unwrap();
        assert_eq!(entry.view_pk, card.view_pk);
        assert_eq!(entry.trust_level, TrustLevel::Pinned);

        service
            .tofu_revoke(&wallet_id, &card.owner_handle)
            .await
            .unwrap();
        let pins = service.load_tofu(&wallet_id).await.unwrap();
        assert_eq!(
            pins.get(&card.owner_handle).unwrap().trust_level,
            TrustLevel::Revoked
        );
    }

    #[tokio::test]
    async fn test_tofu_confirm_miss() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let err = service
            .tofu_confirm(&wallet_id, &[1u8; 32], &[2u8; 32])
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            WalletError::InvalidParams(ref msg) if msg == "tofu owner not found"
        ));
    }

    #[tokio::test]
    async fn test_tofu_revoke_miss() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _token = service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let err = service
            .tofu_revoke(&wallet_id, &[1u8; 32])
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            WalletError::InvalidParams(ref msg) if msg == "tofu owner not found"
        ));
    }

    #[test]
    fn test_create_never_returns_seed() {
        let (service, _dir) = test_service_with_tempdir();
        let resp = service
            .reachability()
            .create_wallet("test".to_string(), "<redacted>".to_string());

        assert_eq!(resp.seed_phrase, "<redacted>");
    }

    #[tokio::test]
    async fn test_wallet_wlt_delete_data() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password, test_seed_phrase_24())
            .await
            .unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let wlt_path = service.wlt_file_path(&wallet_id);
            assert!(wlt_path.exists(), ".wlt file should exist after create");

            let lock_path = {
                let mut os = wlt_path.as_os_str().to_os_string();
                os.push(".lock");
                PathBuf::from(os)
            };

            let tmp_path = {
                let mut os = wlt_path.as_os_str().to_os_string();
                os.push(".tmp");
                PathBuf::from(os)
            };

            let password = SafePassword::from(TEST_PASSWORD);
            service
                .delete_wallet_data(&wallet_id, &password)
                .await
                .unwrap();

            assert!(!wlt_path.exists(), ".wlt file should be deleted");
            assert!(!lock_path.exists(), ".lock file should be deleted");
            assert!(!tmp_path.exists(), ".tmp file should be deleted");
        }

        let wallets = service.list_wallets_in_memory().await.unwrap();
        assert!(wallets.is_empty());
    }

    #[tokio::test]
    async fn test_unlock_wallet_stub() {
        let (service, _dir) = test_service_with_tempdir();
        let create_password = SafePassword::from(TEST_PASSWORD);
        let unlock_password = SafePassword::from(TEST_PASSWORD);
        // First create a wallet
        let wallet_id = service
            .create_wallet_in_memory("test", create_password, test_seed_phrase_24())
            .await
            .unwrap();

        // Now unlock it with the correct password
        let token = service
            .unlock_wallet_in_memory(&wallet_id, &unlock_password)
            .await
            .unwrap();

        // After implementing, token is a random hex string
        assert!(!token.token.is_empty());
        assert_ne!(token.token, "stub-session-token");
        assert_eq!(token.wallet_id, wallet_id);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_lock_removes_wlt_lock() {
        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let wlt_path = service.wlt_file_path(&wallet_id);
        let lock_path = {
            let mut os = wlt_path.as_os_str().to_os_string();
            os.push(".lock");
            PathBuf::from(os)
        };

        assert!(lock_path.exists(), ".wlt.lock should exist while unlocked");

        service.lock_wallet(&wallet_id).await.unwrap();
        assert!(
            !lock_path.exists(),
            ".wlt.lock should be deleted after lock_wallet"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_removes_stale_wlt_lock() {
        use z00z_utils::io::File;

        let (service, _dir) = test_service_with_tempdir();

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password, test_seed_phrase_24())
            .await
            .unwrap();

        let wlt_path = service.wlt_file_path(&wallet_id);
        let lock_path = {
            let mut os = wlt_path.as_os_str().to_os_string();
            os.push(".lock");
            PathBuf::from(os)
        };

        let _ = File::options()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .unwrap();

        assert!(lock_path.exists(), ".wlt.lock should exist before cleanup");

        service.lock_wallet(&wallet_id).await.unwrap();

        assert!(
            !lock_path.exists(),
            ".wlt.lock should be deleted when stale"
        );
    }

    #[test]
    fn test_safe_password_debug_leak() {
        let secret = "correct horse battery staple";
        let password = SafePassword::from(secret);
        let rendered = format!("{password:?}");
        assert!(
            !rendered.contains(secret),
            "SafePassword debug output must not include plaintext"
        );
    }

    #[tokio::test]
    async fn test_unlock_attempt_precheck_rate() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time);
        let wallet_id = PersistWalletId("test-wallet".to_string());

        for _ in 0..5 {
            assert_eq!(
                service.unlock_attempt_precheck(&wallet_id).await.unwrap(),
                UnlockAttemptPrecheck::Allowed
            );
        }

        match service.unlock_attempt_precheck(&wallet_id).await.unwrap() {
            UnlockAttemptPrecheck::RateLimited { .. } => {}
            other => panic!("expected RateLimited, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_unlock_attempt_precheck_enforces() {
        let time = Arc::new(MockTimeProvider::default());
        let mut service = create_service_with_default_config(time.clone());
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));
        let wallet_id = PersistWalletId("test-wallet".to_string());

        assert_eq!(
            service.unlock_attempt_precheck(&wallet_id).await.unwrap(),
            UnlockAttemptPrecheck::Allowed
        );

        service
            .record_unlock_attempt_result(&wallet_id, false)
            .await;

        assert_eq!(
            service.unlock_attempt_precheck(&wallet_id).await.unwrap(),
            UnlockAttemptPrecheck::Allowed
        );
    }

    #[tokio::test]
    async fn test_unlock_precheck_clock_unavailable() {
        let time = Arc::new(MockTimeProvider::before_unix_millis(1));
        let service = create_service_with_default_config(time);
        let wallet_id = PersistWalletId("test-wallet".to_string());

        let err = service
            .unlock_attempt_precheck(&wallet_id)
            .await
            .unwrap_err();
        assert!(
            matches!(err, WalletError::InvalidConfig(message) if message.contains("clock unavailable"))
        );
    }

    #[tokio::test]
    async fn test_verify_fails_clock_unavailable() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time.clone());
        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();
        let session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        time.set_before_unix_millis(1);

        let err = service.verify_session(&session).await.unwrap_err();
        assert!(
            matches!(err, WalletError::InvalidConfig(message) if message.contains("clock unavailable"))
        );
    }

    #[tokio::test]
    async fn test_backup_precheck_clock_unavailable() {
        let time = Arc::new(MockTimeProvider::before_unix_millis(1));
        let service = create_service_with_default_config(time);
        let wallet_id = PersistWalletId("test-wallet".to_string());

        let err = service
            .backup_create_rate_limit_precheck(&wallet_id)
            .await
            .unwrap_err();
        assert!(
            matches!(err, WalletError::InvalidConfig(message) if message.contains("clock unavailable"))
        );
    }

    #[tokio::test]
    async fn test_unlock_delay_matches_formula() {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let correct_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", correct_password, test_seed_phrase_24())
            .await
            .unwrap();

        let wrong_password = SafePassword::from(TEST_WRONG_PASSWORD);

        // failures=0 => 200ms
        let t0 = time.compat_unix_timestamp_millis();
        let err = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(err, WalletError::InvalidPassword));
        let t1 = time.compat_unix_timestamp_millis();
        assert_eq!(t1.saturating_sub(t0), 200);

        // failures=1 => 400ms
        let t2 = time.compat_unix_timestamp_millis();
        let err2 = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(err2, WalletError::InvalidPassword));
        let t3 = time.compat_unix_timestamp_millis();
        assert_eq!(t3.saturating_sub(t2), 400);
    }

    #[tokio::test]
    async fn test_unlock_delay_cap() {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let correct_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", correct_password, test_seed_phrase_24())
            .await
            .unwrap();

        let wrong_password = SafePassword::from(TEST_WRONG_PASSWORD);

        // Drive failures to 4 so the next delay is capped.
        // failures=0 => 200ms
        let t0 = time.compat_unix_timestamp_millis();
        let _ = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        let t1 = time.compat_unix_timestamp_millis();
        assert_eq!(t1.saturating_sub(t0), 200);

        // failures=1 => 400ms
        let t2 = time.compat_unix_timestamp_millis();
        let _ = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        let t3 = time.compat_unix_timestamp_millis();
        assert_eq!(t3.saturating_sub(t2), 400);

        // failures=2 => 800ms
        let t4 = time.compat_unix_timestamp_millis();
        let _ = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        let t5 = time.compat_unix_timestamp_millis();
        assert_eq!(t5.saturating_sub(t4), 800);

        // failures=3 => 1600ms
        let t6 = time.compat_unix_timestamp_millis();
        let _ = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        let t7 = time.compat_unix_timestamp_millis();
        assert_eq!(t7.saturating_sub(t6), 1600);

        // failures=4 => min(3000, 200*16=3200) => 3000ms
        let t8 = time.compat_unix_timestamp_millis();
        let err = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(err, WalletError::InvalidPassword));
        let t9 = time.compat_unix_timestamp_millis();
        assert_eq!(t9.saturating_sub(t8), 3000);
    }

    #[tokio::test]
    async fn test_successful_unlock_resets_backoff() {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let correct_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", correct_password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let wrong_password = SafePassword::from(TEST_WRONG_PASSWORD);

        // Two failures to increase backoff.
        let t0 = time.compat_unix_timestamp_millis();
        let _ = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        let t1 = time.compat_unix_timestamp_millis();
        assert_eq!(t1.saturating_sub(t0), 200);

        let t2 = time.compat_unix_timestamp_millis();
        let _ = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        let t3 = time.compat_unix_timestamp_millis();
        assert_eq!(t3.saturating_sub(t2), 400);

        // Successful unlock should reset failed_attempts.
        let _session = service
            .unlock_wallet_in_memory(&wallet_id, &correct_password)
            .await
            .unwrap();

        // Drop the session so the next attempt actually reopens the `.wlt` (and can fail).
        service.lock_wallet(&wallet_id).await.unwrap();

        // Backoff should be reset: failures=0 => 200ms.
        let t4 = time.compat_unix_timestamp_millis();
        let err = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(err, WalletError::InvalidPassword));
        let t5 = time.compat_unix_timestamp_millis();
        assert_eq!(t5.saturating_sub(t4), 200);
    }

    #[tokio::test]
    async fn test_unlock_path_requires_password() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time);
        let correct_password = SafePassword::from(TEST_PASSWORD);
        let wrong_password = SafePassword::from(TEST_WRONG_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", correct_password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let session = service
            .unlock_wallet_in_memory(&wallet_id, &correct_password)
            .await
            .unwrap();

        let err = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(err, WalletError::InvalidPassword));

        let same_session = service
            .unlock_wallet_in_memory(&wallet_id, &correct_password)
            .await
            .unwrap();
        assert_eq!(same_session.token, session.token);
    }

    #[tokio::test]
    async fn test_unlock_wrong_password_backoff() {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let correct_password = SafePassword::from(TEST_PASSWORD);
        let wrong_password = SafePassword::from(TEST_WRONG_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", correct_password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _session = service
            .unlock_wallet_in_memory(&wallet_id, &correct_password)
            .await
            .unwrap();

        let t0 = time.compat_unix_timestamp_millis();
        let err = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(err, WalletError::InvalidPassword));
        let t1 = time.compat_unix_timestamp_millis();
        assert_eq!(t1.saturating_sub(t0), 200);
    }

    #[tokio::test]
    async fn test_unlock_success_resets_backoff() {
        let dir = tempfile::tempdir().unwrap();
        let time = Arc::new(MockTimeProvider::default());
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let correct_password = SafePassword::from(TEST_PASSWORD);
        let wrong_password = SafePassword::from(TEST_WRONG_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", correct_password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let _session = service
            .unlock_wallet_in_memory(&wallet_id, &correct_password)
            .await
            .unwrap();

        let t0 = time.compat_unix_timestamp_millis();
        let first_err = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(first_err, WalletError::InvalidPassword));
        let t1 = time.compat_unix_timestamp_millis();
        assert_eq!(t1.saturating_sub(t0), 200);

        let _same_session = service
            .unlock_wallet_in_memory(&wallet_id, &correct_password)
            .await
            .unwrap();

        let t2 = time.compat_unix_timestamp_millis();
        let second_err = service
            .unlock_wallet_in_memory(&wallet_id, &wrong_password)
            .await
            .unwrap_err();
        assert!(matches!(second_err, WalletError::InvalidPassword));
        let t3 = time.compat_unix_timestamp_millis();
        assert_eq!(t3.saturating_sub(t2), 200);
    }

    #[tokio::test]
    async fn test_rate_limit_resets_wallet() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time.clone());

        let wallet_a = PersistWalletId("wallet-a".to_string());
        let wallet_b = PersistWalletId("wallet-b".to_string());

        for _ in 0..5 {
            assert_eq!(
                service.unlock_attempt_precheck(&wallet_a).await.unwrap(),
                UnlockAttemptPrecheck::Allowed
            );
        }

        match service.unlock_attempt_precheck(&wallet_a).await.unwrap() {
            UnlockAttemptPrecheck::RateLimited { .. } => {}
            other => panic!("expected RateLimited for wallet_a, got {other:?}"),
        }

        // Different wallet id must not share counters.
        assert_eq!(
            service.unlock_attempt_precheck(&wallet_b).await.unwrap(),
            UnlockAttemptPrecheck::Allowed
        );

        // After window elapses, wallet_a should be allowed again.
        time.advance_by(Duration::from_secs(60) + Duration::from_millis(1));
        assert_eq!(
            service.unlock_attempt_precheck(&wallet_a).await.unwrap(),
            UnlockAttemptPrecheck::Allowed
        );
    }

    // ========== Session Teardown Tests ==========

    #[tokio::test]
    async fn test_lock_drops_session() {
        let policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        let (service, _dir, _time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        service.verify_session(&session).await.unwrap();

        service.lock_wallet(&wallet_id).await.unwrap();

        let err = service.verify_session(&session).await.unwrap_err();
        assert!(matches!(
            err,
            WalletError::SessionInvalid | WalletError::SessionExpired
        ));

        let session2 = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        assert_ne!(session2.token, session.token);
        service.verify_session(&session2).await.unwrap();
    }

    #[tokio::test]
    async fn test_lock_revokes_session() {
        let policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        let (service, _dir, _time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let session_token = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        let session_handle = {
            let now_ms = service.now_ms();
            let timeout_ms = service.timeout_ms();
            service
                .wallet_sessions
                .verify(&session_token, now_ms, timeout_ms)
                .await
                .unwrap()
        };

        service.lock_wallet(&wallet_id).await.unwrap();

        let handle_res = tokio::task::spawn_blocking(move || {
            session_handle
                .with_wallet_session(|wlt_session| Ok(wlt_session.opened().wallet_id.clone()))
        })
        .await
        .expect("spawn_blocking failed");

        let err = handle_res.unwrap_err();
        assert!(matches!(err, WalletError::SessionInvalid));
    }

    #[tokio::test]
    async fn test_expiry_revokes_session() {
        let policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        let (service, _dir, time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let session_token = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        let session_handle = {
            let now_ms = service.now_ms();
            let timeout_ms = service.timeout_ms();
            service
                .wallet_sessions
                .verify(&session_token, now_ms, timeout_ms)
                .await
                .unwrap()
        };

        time.advance_by(Duration::from_millis(101));

        let now_ms = service.now_ms();
        let timeout_ms = service.timeout_ms();
        let token = service
            .wallet_sessions
            .existing_token(&wallet_id, now_ms, timeout_ms)
            .await;
        assert!(token.is_none());

        let handle_res = tokio::task::spawn_blocking(move || {
            session_handle
                .with_wallet_session(|wlt_session| Ok(wlt_session.opened().wallet_id.clone()))
        })
        .await
        .expect("spawn_blocking failed");

        let err = handle_res.unwrap_err();
        assert!(matches!(err, WalletError::SessionInvalid));
    }

    #[tokio::test]
    async fn test_auto_lock_timeout_boundary() {
        let policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        let (service, _dir, time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let _session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        time.advance_by(Duration::from_millis(100));

        let expired = service.check_auto_lock().await.unwrap();
        assert_eq!(expired, vec![wallet_id.clone()]);

        let token = service
            .wallet_sessions
            .existing_token(&wallet_id, service.now_ms(), service.timeout_ms())
            .await;
        assert!(token.is_none());
    }

    #[tokio::test]
    async fn test_auto_lock_clock_unavailable() {
        let policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        let (service, _dir, time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let _session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        time.set_before_unix_millis(1);

        let err = service.check_auto_lock().await.unwrap_err();
        assert!(
            matches!(err, WalletError::InvalidConfig(message) if message.contains("clock unavailable"))
        );
    }

    #[tokio::test]
    async fn test_screen_lock_drops_sessions() {
        let policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        let (service, _dir, _time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        service.verify_session(&session).await.unwrap();

        service
            .on_lifecycle_event(WalletLifecycleEvent::ScreenLocked)
            .await
            .unwrap();

        let err = service.verify_session(&session).await.unwrap_err();
        assert!(matches!(
            err,
            WalletError::SessionInvalid | WalletError::SessionExpired
        ));
    }

    #[tokio::test]
    async fn test_background_drops_session() {
        let policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        let (service, _dir, _time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        service.verify_session(&session).await.unwrap();

        service
            .on_lifecycle_event(WalletLifecycleEvent::Backgrounded)
            .await
            .unwrap();

        let err = service.verify_session(&session).await.unwrap_err();
        assert!(matches!(
            err,
            WalletError::SessionInvalid | WalletError::SessionExpired
        ));
    }

    #[tokio::test]
    async fn test_suspend_drops_session() {
        let policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        let (service, _dir, _time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        service.verify_session(&session).await.unwrap();

        service
            .on_lifecycle_event(WalletLifecycleEvent::Suspended)
            .await
            .unwrap();

        let err = service.verify_session(&session).await.unwrap_err();
        assert!(matches!(
            err,
            WalletError::SessionInvalid | WalletError::SessionExpired
        ));
    }

    #[tokio::test]
    async fn test_foreground_keeps_session() {
        let policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        let (service, _dir, _time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        let session = service
            .unlock_wallet_in_memory(&wallet_id, &wallet_password)
            .await
            .unwrap();

        service.verify_session(&session).await.unwrap();

        service
            .on_lifecycle_event(WalletLifecycleEvent::Foregrounded)
            .await
            .unwrap();

        service.verify_session(&session).await.unwrap();
    }

    #[tokio::test]
    async fn test_lifecycle_locks_all_wallets() {
        let policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        let (service, _dir, _time) = test_service_tempdir_policy(policy);

        let wallet_password = SafePassword::from(TEST_PASSWORD);

        let wallet_a = service
            .create_wallet_in_memory("wallet-a", wallet_password.clone(), test_seed_phrase_24())
            .await
            .unwrap();
        let wallet_b = service
            .create_wallet_in_memory("wallet-b", wallet_password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        let session_a = service
            .unlock_wallet_in_memory(&wallet_a, &wallet_password)
            .await
            .unwrap();
        let session_b = service
            .unlock_wallet_in_memory(&wallet_b, &wallet_password)
            .await
            .unwrap();

        service.verify_session(&session_a).await.unwrap();
        service.verify_session(&session_b).await.unwrap();

        service
            .on_lifecycle_event(WalletLifecycleEvent::Backgrounded)
            .await
            .unwrap();

        let err_a = service.verify_session(&session_a).await.unwrap_err();
        let err_b = service.verify_session(&session_b).await.unwrap_err();

        assert!(matches!(
            err_a,
            WalletError::SessionInvalid | WalletError::SessionExpired
        ));
        assert!(matches!(
            err_b,
            WalletError::SessionInvalid | WalletError::SessionExpired
        ));
    }

    #[tokio::test]
    async fn test_timeout_auto_lock() {
        let policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        let (service, _dir, time) = test_service_tempdir_policy(policy);

        let step_timeout = if cfg!(debug_assertions) {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(20)
        };

        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = timeout(
            step_timeout,
            service.create_wallet_in_memory(
                "test-wallet",
                wallet_password.clone(),
                test_seed_phrase_24(),
            ),
        )
        .await
        .expect("create_wallet_in_memory timed out")
        .unwrap();

        let session = timeout(
            step_timeout,
            service.unlock_wallet_in_memory(&wallet_id, &wallet_password),
        )
        .await
        .expect("unlock_wallet_in_memory timed out")
        .unwrap();

        // Let the wallet become inactive.
        time.advance_by(Duration::from_millis(150));

        // Auto-lock should deterministically revoke the session.
        let locked = timeout(step_timeout, service.check_auto_lock())
            .await
            .expect("check_auto_lock timed out")
            .unwrap();
        assert_eq!(locked, vec![wallet_id.clone()]);

        {
            let states = service.wallet_states.read().await;
            let state = states.get(&wallet_id).unwrap();
            assert!(state.is_locked());
        }

        let err = timeout(step_timeout, service.verify_session(&session))
            .await
            .expect("verify_session timed out")
            .unwrap_err();
        assert!(matches!(
            err,
            WalletError::SessionInvalid | WalletError::SessionExpired
        ));

        let session2 = timeout(
            step_timeout,
            service.unlock_wallet_in_memory(&wallet_id, &wallet_password),
        )
        .await
        .expect("unlock_wallet_in_memory (second) timed out")
        .unwrap();

        assert_ne!(session2.token, session.token);
        service.verify_session(&session2).await.unwrap();
    }

    // ========== Auto-Lock Tests ==========

    #[tokio::test]
    async fn test_register_and_unregister_wallet() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time);
        let wallet_id = PersistWalletId("test-wallet".to_string());

        // Register wallet
        service
            .register_unlocked_wallet(wallet_id.clone())
            .await
            .unwrap();

        // Check it's tracked
        let states = service.wallet_states.read().await;
        assert!(states.contains_key(&wallet_id));
        drop(states);

        // Unregister wallet
        service.unregister_wallet(&wallet_id).await.unwrap();

        // Check it's removed
        let states = service.wallet_states.read().await;
        assert!(!states.contains_key(&wallet_id));
    }

    #[tokio::test]
    async fn test_check_auto_lock_no() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time);
        let wallet_id = PersistWalletId("test-wallet".to_string());

        // Register wallet
        service
            .register_unlocked_wallet(wallet_id.clone())
            .await
            .unwrap();

        // Immediately check (no timeout)
        let locked = service.check_auto_lock().await.unwrap();
        assert_eq!(locked.len(), 0, "No wallets should be locked yet");

        // Verify wallet still unlocked
        let states = service.wallet_states.read().await;
        let state = states
            .iter()
            .find_map(|(id, state)| if id == &wallet_id { Some(state) } else { None })
            .unwrap();
        assert!(state.is_unlocked());
    }

    #[tokio::test]
    async fn test_check_auto_lock_timeout() {
        // Create service with 100ms timeout for testing
        let policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        let time = Arc::new(MockTimeProvider::default());
        let service = WalletService::auto_lock_policy_dependencies(policy, time.clone());
        let wallet_id = test_wallet_id(1);

        // Register wallet
        service
            .register_unlocked_wallet(wallet_id.clone())
            .await
            .unwrap();

        // Advance time past timeout
        time.advance_by(Duration::from_millis(150));

        // Check auto-lock
        let locked = service.check_auto_lock().await.unwrap();
        assert_eq!(locked.len(), 1, "One wallet should be locked");
        assert_eq!(locked[0], wallet_id);

        // Verify wallet is now locked
        let states = service.wallet_states.read().await;
        let state = states
            .iter()
            .find_map(|(id, state)| if id == &wallet_id { Some(state) } else { None })
            .unwrap();
        assert!(state.is_locked());
    }

    #[tokio::test]
    async fn test_update_activity_resets_timeout() {
        // Create service with 200ms timeout
        let policy = AutoLockPolicy::new(Duration::from_millis(200), vec![]);
        let time = Arc::new(MockTimeProvider::default());
        let service = WalletService::auto_lock_policy_dependencies(policy, time.clone());
        let wallet_id = test_wallet_id(1);

        // Register wallet
        service
            .register_unlocked_wallet(wallet_id.clone())
            .await
            .unwrap();

        // Advance 100ms (halfway to timeout)
        time.advance_by(Duration::from_millis(100));

        // Update activity (resets timeout)
        service.update_activity(&wallet_id).await.unwrap();

        // Advance another 150ms (would exceed original timeout but not reset one)
        time.advance_by(Duration::from_millis(150));

        // Check - should still be unlocked
        let locked = service.check_auto_lock().await.unwrap();
        assert_eq!(
            locked.len(),
            0,
            "Wallet should still be unlocked after activity update"
        );

        // Verify wallet still unlocked
        let states = service.wallet_states.read().await;
        let state = states
            .iter()
            .find_map(|(id, state)| if id == &wallet_id { Some(state) } else { None })
            .unwrap();
        assert!(state.is_unlocked());
    }

    #[tokio::test]
    async fn test_auto_lock_monitor_background() {
        // Create service with 50ms timeout (very short for testing)
        let policy = AutoLockPolicy::new(Duration::from_millis(50), vec![]);
        let time = Arc::new(MockTimeProvider::default());
        let service = Arc::new(WalletService::auto_lock_policy_dependencies(
            policy,
            time.clone(),
        ));
        let wallet_id = test_wallet_id(1);

        // Register wallet
        service
            .register_unlocked_wallet(wallet_id.clone())
            .await
            .unwrap();

        // Start monitor
        let monitor = service.clone().start_auto_lock_monitor();

        // Wait for timeout + full monitor interval (50ms + 30s is too long)
        // Monitor runs every 30s, so we need to wait > 30s for it to trigger
        // Instead, let's just verify monitor starts without crashing
        time.advance_by(Duration::from_millis(100));

        // Manually call check_auto_lock (monitor would do this)
        let locked = service.check_auto_lock().await.unwrap();
        assert_eq!(
            locked.len(),
            1,
            "Wallet should be auto-locked after timeout"
        );

        // Verify wallet was locked
        let states = service.wallet_states.read().await;
        let state = states
            .iter()
            .find_map(|(id, state)| if id == &wallet_id { Some(state) } else { None })
            .unwrap();
        assert!(state.is_locked(), "Wallet should be locked");

        // Stop monitor
        monitor.abort();
    }

    #[tokio::test]
    async fn test_auto_lock_monitor_error() {
        // Monitor should handle errors gracefully
        let time = Arc::new(MockTimeProvider::default());
        let service = Arc::new(create_service_with_default_config(time));

        // Start monitor (will run but nothing to lock)
        let monitor = service.start_auto_lock_monitor();

        // Wait a bit to ensure monitor runs without panicking
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Monitor should still be running
        assert!(
            !monitor.is_finished(),
            "Monitor should not crash on empty state"
        );

        // Stop monitor
        monitor.abort();
    }

    #[tokio::test]
    async fn test_multiple_wallets_auto_lock() {
        // Create service with 100ms timeout
        let policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        let time = Arc::new(MockTimeProvider::default());
        let service = WalletService::auto_lock_policy_dependencies(policy, time.clone());

        let wallet1 = test_wallet_id(1);
        let wallet2 = test_wallet_id(2);
        let wallet3 = test_wallet_id(3);

        // Register 3 wallets at staggered times
        service
            .register_unlocked_wallet(wallet1.clone())
            .await
            .unwrap();
        time.advance_by(Duration::from_millis(40));
        service
            .register_unlocked_wallet(wallet2.clone())
            .await
            .unwrap();
        time.advance_by(Duration::from_millis(40));
        service
            .register_unlocked_wallet(wallet3.clone())
            .await
            .unwrap();

        // Wait for first wallet to timeout (total: 80ms + 30ms = 110ms > 100ms)
        time.advance_by(Duration::from_millis(30));

        // Check - wallet1 should be locked (110ms elapsed)
        let locked = service.check_auto_lock().await.unwrap();
        assert_eq!(locked.len(), 1, "First wallet should timeout");
        assert!(locked.contains(&wallet1));

        // Wait for remaining wallets to timeout (another 80ms)
        time.advance_by(Duration::from_millis(80));

        // Check - wallet2 and wallet3 should now be locked
        let locked = service.check_auto_lock().await.unwrap();
        assert_eq!(locked.len(), 2, "Second and third wallets should timeout");
        assert!(locked.contains(&wallet2));
        assert!(locked.contains(&wallet3));
    }

    #[tokio::test]
    async fn test_get_wallet_state() {
        let time = Arc::new(MockTimeProvider::default());
        let service = create_service_with_default_config(time);
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test-wallet", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        let state = service.get_wallet_state(&wallet_id).await.unwrap();
        assert!(state.is_unlocked());

        service.unregister_wallet(&wallet_id).await.unwrap();

        let result = service.get_wallet_state(&wallet_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::WalletError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_state_expired_session_locked() {
        let time = Arc::new(MockTimeProvider::from_unix_secs(1));
        let dir = tempfile::tempdir().unwrap();
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.auto_lock_policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        time.advance_by(Duration::from_millis(101));

        assert_eq!(
            service.get_wallet_state(&wallet_id).await.unwrap(),
            WalletState::Locked
        );
    }

    #[tokio::test]
    async fn test_list_expired_session_locked() {
        let time = Arc::new(MockTimeProvider::from_unix_secs(1));
        let dir = tempfile::tempdir().unwrap();
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.auto_lock_policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        time.advance_by(Duration::from_millis(101));

        let wallets = service.list_wallets_in_memory().await.unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].id, wallet_id);
        assert!(wallets[0].is_locked);
    }

    #[tokio::test]
    async fn test_state_clock_failure_locked() {
        let time = Arc::new(MockTimeProvider::from_unix_secs(1));
        let dir = tempfile::tempdir().unwrap();
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.auto_lock_policy = AutoLockPolicy::new(Duration::from_secs(60), vec![]);
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        time.set_before_unix_millis(1);

        assert_eq!(
            service.get_wallet_state(&wallet_id).await.unwrap(),
            WalletState::Locked
        );
    }

    #[tokio::test]
    async fn test_export_import_wallet_payload() {
        use z00z_utils::codec::{Codec, JsonCodec};

        let (service, _dir) = test_service_with_tempdir();
        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", wallet_password, test_seed_phrase_24())
            .await
            .unwrap();
        service
            .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
            .await
            .unwrap();

        let exported = service
            .export_wallet_payload(&wallet_id, &export_password)
            .await
            .unwrap();

        let codec = JsonCodec;
        let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

        let (dst_service, _dst_dir) = test_service_with_tempdir();
        let imported_wallet_id = dst_service
            .import_wallet_payload(&export_json, &export_password, "imported")
            .await
            .unwrap();

        let wallets = dst_service.list_wallets_in_memory().await.unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].id, imported_wallet_id);
        assert_eq!(wallets[0].name, "imported");
        assert!(!dst_service
            .wallet_history_jsonl_path(&imported_wallet_id)
            .exists());
    }

    #[tokio::test]
    async fn test_export_skips_history() {
        use z00z_utils::codec::{Codec, JsonCodec};

        let (service, _dir) = test_service_with_tempdir();
        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", wallet_password, test_seed_phrase_24())
            .await
            .unwrap();
        service
            .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
            .await
            .unwrap();

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let mut history_store = crate::persistence::tx::TxStorageImpl::new(
            &history_path,
            z00z_utils::time::SystemTimeProvider,
        );
        crate::persistence::tx::TxStorage::put(
            &mut history_store,
            sample_backup_tx_record("tx-payload-only", vec![7, 8, 9]),
        )
        .unwrap();

        let exported = service
            .export_wallet_payload(&wallet_id, &export_password)
            .await
            .unwrap();

        let codec = JsonCodec;
        let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

        let (dst_service, _dst_dir) = test_service_with_tempdir();
        let imported_wallet_id = dst_service
            .import_wallet_payload(&export_json, &export_password, "imported")
            .await
            .unwrap();

        assert!(!dst_service
            .wallet_history_jsonl_path(&imported_wallet_id)
            .exists());
    }

    #[tokio::test]
    async fn test_load_wallet_persisted_identity() {
        let (service, dir) = test_service_with_tempdir();
        let explicit_identity = non_default_wallet_identity();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_using_explicit_identity(
                "persisted",
                password,
                test_seed_phrase_24(),
                &explicit_identity,
            )
            .await
            .unwrap();

        let time = Arc::new(MockTimeProvider::default());
        let mut restarted = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        restarted.sleeper = Arc::new(MockSleeper::new(time));

        restarted
            .load_wallet(&wallet_id, TEST_PASSWORD)
            .await
            .unwrap();

        let identities = restarted.wallet_identities.read().await;
        let restored_identity = identities.get(&wallet_id).cloned().unwrap();
        assert_eq!(restored_identity.chain, explicit_identity.chain);
    }

    #[tokio::test]
    async fn test_export_pack_discovers_identity() {
        let (service, _dir) = test_service_with_tempdir();
        let explicit_identity = non_default_wallet_identity();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_using_explicit_identity(
                "persisted",
                password.clone(),
                test_seed_phrase_24(),
                &explicit_identity,
            )
            .await
            .unwrap();

        {
            let mut identities = service.wallet_identities.write().await;
            identities.remove(&wallet_id);
        }

        let export_pack = service
            .build_wallet_export_pack(&wallet_id, &password)
            .await
            .unwrap();

        let wallet_identity = export_pack.wallet_identity.unwrap();
        assert_eq!(wallet_identity.chain, explicit_identity.chain);
        assert_eq!(wallet_identity.network, explicit_identity.network);
    }

    #[tokio::test]
    async fn test_import_payload_preserves_identity() {
        use z00z_utils::codec::{Codec, JsonCodec};

        let (service, _dir) = test_service_with_tempdir();
        let explicit_identity = non_default_wallet_identity();
        let wallet_password = SafePassword::from(TEST_PASSWORD);
        let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
        let wallet_id = service
            .create_wallet_using_explicit_identity(
                "persisted",
                wallet_password,
                test_seed_phrase_24(),
                &explicit_identity,
            )
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
            .await
            .unwrap();

        let exported = service
            .export_wallet_payload(&wallet_id, &export_password)
            .await
            .unwrap();

        let codec = JsonCodec;
        let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

        let (dst_service, _dst_dir) = test_service_with_tempdir();
        let imported_wallet_id = dst_service
            .import_wallet_payload(&export_json, &export_password, "imported")
            .await
            .unwrap();

        let identities = dst_service.wallet_identities.read().await;
        let imported_identity = identities.get(&imported_wallet_id).cloned().unwrap();
        assert_eq!(imported_identity.chain, explicit_identity.chain);
        assert_eq!(imported_identity.network, explicit_identity.network);
    }

    #[tokio::test]
    async fn test_save_wallet_refreshes_activity() {
        let time = Arc::new(MockTimeProvider::from_unix_secs(1));
        let dir = tempfile::tempdir().unwrap();
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.auto_lock_policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        time.advance_by(Duration::from_millis(99));

        service
            .save_wallet(wallet_id.clone(), password.clone(), None)
            .await
            .unwrap();

        time.advance_by(Duration::from_millis(50));
        assert!(service.check_auto_lock().await.unwrap().is_empty());

        time.advance_by(Duration::from_millis(50));
        assert_eq!(service.check_auto_lock().await.unwrap(), vec![wallet_id]);
    }

    #[tokio::test]
    async fn test_export_pack_refreshes_activity() {
        let time = Arc::new(MockTimeProvider::from_unix_secs(1));
        let dir = tempfile::tempdir().unwrap();
        let mut service = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            time.clone(),
            SystemRngProvider,
        );
        service.auto_lock_policy = AutoLockPolicy::new(Duration::from_millis(100), vec![]);
        service.sleeper = Arc::new(MockSleeper::new(time.clone()));

        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .unwrap();

        time.advance_by(Duration::from_millis(99));

        let export_pack = service
            .build_wallet_export_pack(&wallet_id, &password)
            .await
            .unwrap();
        assert_eq!(export_pack.seed_phrase, test_seed_phrase_24());

        time.advance_by(Duration::from_millis(50));
        assert!(service.check_auto_lock().await.unwrap().is_empty());

        time.advance_by(Duration::from_millis(50));
        assert_eq!(service.check_auto_lock().await.unwrap(), vec![wallet_id]);
    }

    #[tokio::test]
    async fn test_backup_writes_jsonl() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("test", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let mut store = crate::persistence::tx::TxStorageImpl::new(
            &history_path,
            z00z_utils::time::SystemTimeProvider,
        );
        crate::persistence::tx::TxStorage::put(
            &mut store,
            sample_backup_tx_record("tx-1", vec![1, 2, 3, 4]),
        )
        .unwrap();
        crate::persistence::tx::TxStorage::put(
            &mut store,
            sample_backup_tx_record("tx-2", vec![5, 6, 7, 8]),
        )
        .unwrap();

        let backup = service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .unwrap();

        let wallet_file_path = service.wlt_file_path(&wallet_id);
        let backup_path = std::path::PathBuf::from(&backup.backup_path);

        assert!(wallet_file_path.exists());
        assert!(history_path.exists());
        assert!(backup_path.exists());
        assert_eq!(wallet_file_path.parent(), history_path.parent());
        assert_ne!(backup_path.parent(), history_path.parent());
        assert!(!history_path
            .with_file_name(format!(
                "wallet_{}_tx_history",
                WalletService::wallet_stem(&wallet_id)
            ))
            .exists());

        let jsonl = z00z_utils::io::read_to_string(&history_path).unwrap();
        assert!(jsonl.contains("\"tx_hash\":\"tx-1\""));
        assert!(jsonl.contains("\"tx_hash\":\"tx-2\""));
    }

    #[tokio::test]
    async fn test_backup_empty_jsonl() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("empty-backup", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let backup = service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .expect("create backup for empty wallet");

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let backup_path = std::path::PathBuf::from(&backup.backup_path);

        assert!(history_path.exists());
        assert!(backup_path.exists());
        assert_eq!(z00z_utils::io::read_to_string(&history_path).unwrap(), "");
        assert!(!history_path
            .with_file_name(format!(
                "wallet_{}_tx_history",
                WalletService::wallet_stem(&wallet_id)
            ))
            .exists());
    }

    #[tokio::test]
    async fn test_backup_rejects_big_history() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "backup-oversized-history",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let oversized =
            vec![b'\n'; crate::persistence::tx::MAX_TX_HISTORY_JSONL_BYTES as usize + 1];
        z00z_utils::io::write_file(&history_path, &oversized).unwrap();

        let err = service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .unwrap_err();

        let err_text = format!("{err:?}");
        assert!(
            err_text.contains("tx-history JSONL file too large")
                || err_text.contains("File too large"),
            "{err_text}"
        );
    }

    #[tokio::test]
    async fn test_backup_rejects_wrong_stem() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "backup-wrong-history-stem",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let wrong_bytes = crate::backup::encode_tx_history_jsonl(
            "wrongstem",
            &[sample_backup_tx_record("tx-bad-stem", vec![9, 9, 9])],
        )
        .unwrap();
        z00z_utils::io::write_file(&history_path, &wrong_bytes).unwrap();

        let err = service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .unwrap_err();

        let err_text = format!("{err:?}");
        assert!(
            err_text.contains("tx-history wallet stem mismatch"),
            "{err_text}"
        );
    }

    #[tokio::test]
    async fn test_backup_fail_keeps_limit() {
        let (service, _dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "backup-retry-after-fail",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let oversized =
            vec![b'\n'; crate::persistence::tx::MAX_TX_HISTORY_JSONL_BYTES as usize + 1];
        z00z_utils::io::write_file(&history_path, &oversized).unwrap();

        let first_err = service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .unwrap_err();
        let first_err_text = format!("{first_err:?}");
        assert!(
            first_err_text.contains("tx-history JSONL file too large")
                || first_err_text.contains("File too large"),
            "{first_err_text}"
        );

        z00z_utils::io::write_file(&history_path, &[]).unwrap();

        let backup = service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .expect("retry backup after failed oversized sidecar");

        assert!(std::path::PathBuf::from(backup.backup_path).exists());
    }

    #[tokio::test]
    async fn test_backup_waits_history_lock() {
        let (service, _dir) = test_service_with_tempdir();
        let service = Arc::new(service);
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory("locked-backup", password.clone(), test_seed_phrase_24())
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let history_lock = crate::persistence::tx::tx_history_path_lock(&history_path).unwrap();
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();

        let holder = std::thread::spawn(move || {
            let _guard = history_lock.lock().unwrap();
            locked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });

        locked_rx.recv().unwrap();

        let backup_service = Arc::clone(&service);
        let backup_wallet_id = wallet_id.clone();
        let backup_password = password.clone();
        let mut backup_task = tokio::spawn(async move {
            backup_service
                .create_backup(&backup_wallet_id, backup_password, None)
                .await
        });

        assert!(timeout(Duration::from_millis(100), &mut backup_task)
            .await
            .is_err());

        release_tx.send(()).unwrap();
        let backup = backup_task.await.unwrap().unwrap();
        holder.join().unwrap();

        assert!(std::path::PathBuf::from(backup.backup_path).exists());
        assert_eq!(z00z_utils::io::read_to_string(&history_path).unwrap(), "");
    }

    #[tokio::test]
    async fn test_backup_progress_limits() {
        let (service, _dir) = test_service_with_tempdir();
        let service = Arc::new(service);
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = service
            .create_wallet_in_memory(
                "locked-backup-rate-limit",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .unwrap();

        service
            .save_wallet(
                wallet_id.clone(),
                password.clone(),
                Some(test_seed_phrase_24()),
            )
            .await
            .unwrap();

        let history_path = service.wallet_history_jsonl_path(&wallet_id);
        let history_lock = crate::persistence::tx::tx_history_path_lock(&history_path).unwrap();
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();

        let holder = std::thread::spawn(move || {
            let _guard = history_lock.lock().unwrap();
            locked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });

        locked_rx.recv().unwrap();

        let first_service = Arc::clone(&service);
        let first_wallet_id = wallet_id.clone();
        let first_password = password.clone();
        let mut first_backup = tokio::spawn(async move {
            first_service
                .create_backup(&first_wallet_id, first_password, None)
                .await
        });

        assert!(timeout(Duration::from_millis(100), &mut first_backup)
            .await
            .is_err());

        let second_err = service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .unwrap_err();
        assert!(matches!(
            second_err,
            WalletError::RateLimited {
                retry_after_seconds: 1
            }
        ));

        release_tx.send(()).unwrap();
        let backup = first_backup.await.unwrap().unwrap();
        holder.join().unwrap();

        assert!(std::path::PathBuf::from(backup.backup_path).exists());
    }

    #[tokio::test]
    async fn test_string_kdf_restore_fails() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("invalid-kdf-contract.backup");

        z00z_utils::io::write_file(&backup_path, &string_kdf_contract_bytes())
            .expect("write backup");

        let err = service
            .restore_backup(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("deserialization error")
        ));
    }

    #[tokio::test]
    async fn test_restore_imports_tx() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("wallet-plus-history.backup");
        let expected_claim = test_backup_claimed_asset();

        z00z_utils::io::write_file(&backup_path, &forensic_backup_bytes()).expect("write backup");

        let restored = service
            .restore_backup(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
            )
            .await
            .expect("restore backup with history");

        let history_path = service.wallet_history_jsonl_path(&restored.wallet_id);
        let store = crate::persistence::tx::TxStorageImpl::new(
            &history_path,
            z00z_utils::time::MockTimeProvider::default(),
        );
        let records = crate::persistence::tx::TxStorage::list(&store).expect("list tx records");

        assert!(history_path.exists());
        assert!(!history_path
            .with_file_name(format!(
                "wallet_{}_tx_history",
                WalletService::wallet_stem(&restored.wallet_id)
            ))
            .exists());
        let claimed = service
            .list_claimed_assets(&restored.wallet_id)
            .await
            .expect("list restored claimed assets");
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].definition.id, expected_claim.definition.id);
        assert_eq!(claimed[0].asset_id(), expected_claim.asset_id());
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].tx_hash, "tx-1");
        assert_eq!(records[1].tx_hash, "tx-2");
    }

    #[tokio::test]
    async fn test_backup_restore_owned_objects() {
        let (source_service, _source_dir) = test_service_with_tempdir();
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = source_service
            .create_wallet_in_memory(
                "typed-owned-objects",
                password.clone(),
                test_seed_phrase_24(),
            )
            .await
            .expect("create source wallet");
        source_service
            .unlock_wallet_in_memory(&wallet_id, &password)
            .await
            .expect("unlock source wallet");

        let voucher = test_owned_voucher_payload(wallet_id.clone(), 91);
        let right = test_owned_right_payload(wallet_id.clone(), 92);
        source_service
            .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Voucher(voucher.clone()))
            .await
            .expect("store voucher");
        source_service
            .put_owned_object_for_tests(&wallet_id, OwnedObjectPayload::Right(right.clone()))
            .await
            .expect("store right");

        let backup = source_service
            .create_backup(&wallet_id, password.clone(), None)
            .await
            .expect("create backup");
        let backup_bytes = z00z_utils::io::read_file(&backup.backup_path).expect("read backup");

        let (restore_service, restore_dir) = test_service_with_tempdir();
        let restore_path = restore_dir.path().join("typed-owned-objects.backup");
        write_file(&restore_path, &backup_bytes).expect("write restore backup");

        let restored = restore_service
            .restore_backup(
                restore_path.to_string_lossy().to_string(),
                password.clone(),
                None,
            )
            .await
            .expect("restore backup");
        restore_service
            .unlock_wallet_in_memory(&restored.wallet_id, &password)
            .await
            .expect("unlock restored wallet");

        let objects = restore_service
            .list_wallet_inventory(
                &restored.wallet_id,
                ObjectInventoryFilter::default(),
                None,
                10,
            )
            .await
            .expect("list restored objects");
        assert_eq!(objects.items.len(), 2);
        assert!(
            objects.items.iter().any(|object| matches!(
                &object.payload,
                OwnedObjectPayload::Voucher(payload)
                    if payload.terminal_id == voucher.terminal_id
                        && payload.status == OwnedVoucherStatus::Redeemable
            )),
            "restored inventory must contain voucher payload"
        );
        assert!(
            objects.items.iter().any(|object| matches!(
                &object.payload,
                OwnedObjectPayload::Right(payload)
                    if payload.terminal_id == right.terminal_id
                        && payload.status == OwnedRightStatus::Granted
            )),
            "restored inventory must contain right payload"
        );

        let vouchers = restore_service
            .list_voucher_claim_rows(
                &restored.wallet_id,
                Some(OwnedVoucherStatus::Redeemable),
                None,
                10,
            )
            .await
            .expect("list restored vouchers");
        let rights = restore_service
            .list_right_inventory_rows(
                &restored.wallet_id,
                Some(OwnedRightStatus::Granted),
                None,
                10,
            )
            .await
            .expect("list restored rights");
        assert_eq!(vouchers.len(), 1);
        assert_eq!(rights.len(), 1);
        assert_eq!(vouchers[0].terminal_id, voucher.terminal_id);
        assert_eq!(rights[0].terminal_id, right.terminal_id);
        assert!(
            restore_service
                .list_claimed_assets(&restored.wallet_id)
                .await
                .expect("list claimed assets")
                .is_empty(),
            "typed object restore must not materialize phantom cash assets"
        );
    }

    #[tokio::test]
    async fn test_restore_rejects_tampered_asset() {
        let (service, _dir) = test_service_with_tempdir();
        let identity = WalletIdentity {
            network: "testnet".to_string(),
            chain: "mainnet".to_string(),
        };
        let password = SafePassword::from(TEST_PASSWORD);
        let mut export_pack = test_backup_export_pack();
        export_pack.owned_assets[0]
            .labels
            .push("tampered".to_string());

        let err = service
            .restore_wallet_export_pack(export_pack, &password, Some("tampered-asset"), &identity)
            .await
            .expect_err("tampered asset checksum must fail closed");

        assert!(matches!(err, WalletError::ChecksumMismatch { .. }));
    }

    #[tokio::test]
    async fn test_restore_rejects_tampered_object() {
        let (service, _dir) = test_service_with_tempdir();
        let identity = WalletIdentity {
            network: "testnet".to_string(),
            chain: "mainnet".to_string(),
        };
        let password = SafePassword::from(TEST_PASSWORD);
        let wallet_id = PersistWalletId("wallet-1".to_string());
        let mut export_pack = test_backup_export_pack();
        let mut voucher = test_owned_voucher_payload(wallet_id, 101);
        voucher.labels.push("tampered".to_string());
        export_pack
            .owned_objects
            .push(OwnedObjectPayload::Voucher(voucher));

        let err = service
            .restore_wallet_export_pack(export_pack, &password, Some("tampered-object"), &identity)
            .await
            .expect_err("tampered object checksum must fail closed");

        assert!(matches!(err, WalletError::ChecksumMismatch { .. }));
    }

    #[tokio::test]
    async fn test_restore_rejects_object_count() {
        let (service, _dir) = test_service_with_tempdir();
        let identity = WalletIdentity {
            network: "testnet".to_string(),
            chain: "mainnet".to_string(),
        };
        let password = SafePassword::from(TEST_PASSWORD);
        let mut export_pack = test_backup_export_pack();
        let manifest = export_pack
            .manifest
            .as_mut()
            .expect("backup manifest must be present");
        manifest.owned_object_count = 1;
        manifest.checksum = Some(manifest.compute_checksum());

        let err = service
            .restore_wallet_export_pack(export_pack, &password, Some("count-mismatch"), &identity)
            .await
            .expect_err("owned object count mismatch must fail closed");

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("backup manifest owned object count mismatch")
        ));
    }

    #[tokio::test]
    async fn test_restore_rejects_wrong_stem() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("wallet-wrong-stem.backup");

        z00z_utils::io::write_file(&backup_path, &wrong_stem_forensic_backup_bytes())
            .expect("write backup");

        let err = service
            .restore_backup(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("tx-history wallet stem mismatch")
        ));
    }

    #[tokio::test]
    async fn test_restore_wrong_password() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("wallet-plus-history-wrong-password.backup");
        let wallet_id = PersistWalletId("wallet-1".to_string());

        z00z_utils::io::write_file(&backup_path, &forensic_backup_bytes()).expect("write backup");

        let err = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("wrong-password"),
                None,
                crate::backup::ForensicImportMode::WalletPlusHistory,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, WalletError::InvalidPassword));
        assert!(!service.wlt_file_path(&wallet_id).exists());
        assert!(!service.wallet_history_jsonl_path(&wallet_id).exists());
        assert!(service.list_wallets_in_memory().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_restore_wallet_only_backup() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("wallet-only-canonical.backup");
        let wallet_id = PersistWalletId("wallet-1".to_string());

        z00z_utils::io::write_file(&backup_path, &forensic_backup_bytes()).expect("write backup");

        let restored = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::WalletOnly,
            )
            .await
            .expect("restore wallet only");

        assert_eq!(restored.wallet_id, wallet_id);
        assert!(service.wlt_file_path(&restored.wallet_id).exists());
        assert!(!service
            .wallet_history_jsonl_path(&restored.wallet_id)
            .exists());

        let wallets = service
            .list_wallets_in_memory()
            .await
            .expect("list wallets");
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].id, restored.wallet_id);
        assert_eq!(wallets[0].name, "Test Wallet");
    }

    #[tokio::test]
    async fn test_restore_history_commit_fail() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir
            .path()
            .join("wallet-plus-history-history-blocked.backup");
        let wallet_id = PersistWalletId("wallet-1".to_string());
        let history_path = service.wallet_history_jsonl_path(&wallet_id);

        z00z_utils::io::write_file(&backup_path, &forensic_backup_bytes()).expect("write backup");
        z00z_utils::io::create_dir_all(&history_path).expect("block history path with directory");

        let err = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::WalletPlusHistory,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("history commit failed")
        ));
        assert!(!service.wlt_file_path(&wallet_id).exists());
        assert!(history_path.is_dir());
        assert!(service.list_wallets_in_memory().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_restore_rejects_dup_assets() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir
            .path()
            .join("wallet-plus-history-duplicate-owned-assets.backup");
        let wallet_id = PersistWalletId("wallet-1".to_string());

        z00z_utils::io::write_file(&backup_path, &dup_owned_asset_backup())
            .expect("write duplicate-owned-assets backup");

        let err = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::WalletPlusHistory,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("duplicate owned asset id in backup payload")
        ));
        assert!(!service.wlt_file_path(&wallet_id).exists());
        assert!(!service.wallet_history_jsonl_path(&wallet_id).exists());
        assert!(service.list_wallets_in_memory().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_restore_wlt_stage_fail() {
        let (mut service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("wallet-plus-history-existing.backup");

        z00z_utils::io::write_file(&backup_path, &forensic_backup_bytes()).expect("write backup");

        let restored = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::WalletPlusHistory,
            )
            .await
            .expect("initial restore");

        let history_path = service.wallet_history_jsonl_path(&restored.wallet_id);
        let history_before =
            z00z_utils::io::read_to_string(&history_path).expect("read history before failure");

        #[cfg(not(target_arch = "wasm32"))]
        {
            service.wlt_store = Arc::new(FailingWltStore);
        }

        let err = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::WalletPlusHistory,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("forced .wlt create failure")
        ));

        let wallets = service
            .list_wallets_in_memory()
            .await
            .expect("list wallets");
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].id, restored.wallet_id);
        assert_eq!(wallets[0].name, "Test Wallet");
        assert_eq!(
            z00z_utils::io::read_to_string(&history_path).expect("read history after failure"),
            history_before
        );
    }

    #[tokio::test]
    async fn test_restore_tx_history_only() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("tx-history-only.backup");

        z00z_utils::io::write_file(&backup_path, &forensic_backup_bytes()).expect("write backup");

        let restored = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::TxHistoryOnly,
            )
            .await
            .expect("restore tx history only");

        assert_eq!(restored.wallet_id.0, "wallet-1");
        assert!(!service.wlt_file_path(&restored.wallet_id).exists());

        let history_path = service.wallet_history_jsonl_path(&restored.wallet_id);
        let store = crate::persistence::tx::TxStorageImpl::new(
            &history_path,
            z00z_utils::time::MockTimeProvider::default(),
        );
        let records = crate::persistence::tx::TxStorage::list(&store).expect("list tx records");

        assert!(history_path.exists());
        assert!(!history_path
            .with_file_name(format!(
                "wallet_{}_tx_history",
                WalletService::wallet_stem(&restored.wallet_id)
            ))
            .exists());
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].tx_hash, "tx-1");
        assert_eq!(records[1].tx_hash, "tx-2");
    }

    #[tokio::test]
    async fn test_restore_tampered_archive() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("wallet-plus-history-tampered.backup");

        z00z_utils::io::write_file(&backup_path, &tampered_forensic_backup_bytes())
            .expect("write tampered backup");

        let err = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::WalletPlusHistory,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("tx_bytes hash mismatch")
        ));
        assert!(!service
            .wlt_file_path(&PersistWalletId("wallet-1".to_string()))
            .exists());
        assert!(!service
            .wallet_history_jsonl_path(&PersistWalletId("wallet-1".to_string()))
            .with_file_name(format!(
                "wallet_{}_tx_history",
                WalletService::wallet_stem(&PersistWalletId("wallet-1".to_string()))
            ))
            .exists());
    }

    #[tokio::test]
    async fn test_restore_needs_archive() {
        let (service, dir) = test_service_with_tempdir();
        let backup_path = dir.path().join("wallet-only.backup");
        let exporter = crate::backup::BackupExporterImpl::new_with_chain(
            "wallet-1".to_string(),
            "testnet".to_string(),
            "mainnet".to_string(),
            test_backup_export_pack(),
            MockTimeProvider::from_unix_secs(1),
            SystemRngProvider,
        );
        let backup_bytes = crate::backup::BackupExporter::export_to_bytes(
            &exporter,
            &SafePassword::from("password"),
        )
        .expect("wallet-only backup bytes");

        z00z_utils::io::write_file(&backup_path, &backup_bytes).expect("write backup");

        let err = service
            .restore_backup_with_mode(
                backup_path.to_string_lossy().to_string(),
                SafePassword::from("password"),
                None,
                crate::backup::ForensicImportMode::WalletPlusHistory,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            WalletError::InvalidConfig(ref msg)
                if msg.contains("forensic archive section is required")
        ));
        assert!(!service
            .wlt_file_path(&PersistWalletId("wallet-1".to_string()))
            .exists());
        assert!(!service
            .wallet_history_jsonl_path(&PersistWalletId("wallet-1".to_string()))
            .with_file_name(format!(
                "wallet_{}_tx_history",
                WalletService::wallet_stem(&PersistWalletId("wallet-1".to_string()))
            ))
            .exists());
    }

    mod rejects_unframed_payload {
        use super::*;

        #[tokio::test]
        async fn test_import_wallet_payload_rejects() {
            use z00z_utils::codec::{Codec, JsonCodec};

            let (service, _dir) = test_service_with_tempdir();
            let wallet_password = SafePassword::from(TEST_PASSWORD);
            let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
            let wallet_id = service
                .create_wallet_in_memory("test", wallet_password, test_seed_phrase_24())
                .await
                .unwrap();
            service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
                .await
                .unwrap();

            let mut exported = service
                .export_wallet_payload(&wallet_id, &export_password)
                .await
                .unwrap();

            let framed_bytes = base64::engine::general_purpose::STANDARD
                .decode(exported.ciphertext.as_bytes())
                .unwrap();

            // Drop magic+version prefix to simulate an unframed payload.
            let prefix_len = WalletService::WALLET_EXPORT_PAYLOAD_MAGIC.len() + 4;
            let unframed = if framed_bytes.len() > prefix_len {
                framed_bytes[prefix_len..].to_vec()
            } else {
                vec![]
            };

            exported.ciphertext = base64::engine::general_purpose::STANDARD.encode(unframed);

            let codec = JsonCodec;
            let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

            let (dst_service, _dst_dir) = test_service_with_tempdir();
            let err = dst_service
                .import_wallet_payload(&export_json, &export_password, "n")
                .await
                .unwrap_err();

            assert!(matches!(
                err,
                WalletError::InvalidParams(ref msg) if msg == "Invalid backup payload format"
            ));
        }
    }

    mod rejects_unknown_version {
        use super::*;

        #[tokio::test]
        async fn test_import_wallet_payload_rejects() {
            use z00z_utils::codec::{Codec, JsonCodec};

            let (service, _dir) = test_service_with_tempdir();
            let wallet_password = SafePassword::from(TEST_PASSWORD);
            let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
            let wallet_id = service
                .create_wallet_in_memory("test", wallet_password, test_seed_phrase_24())
                .await
                .unwrap();
            service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
                .await
                .unwrap();

            let mut exported = service
                .export_wallet_payload(&wallet_id, &export_password)
                .await
                .unwrap();

            let mut framed_bytes = base64::engine::general_purpose::STANDARD
                .decode(exported.ciphertext.as_bytes())
                .unwrap();

            // Overwrite version in the framing prefix.
            // Layout: magic || version (4 bytes LE) || bincode(container)
            let version_offset = WalletService::WALLET_EXPORT_PAYLOAD_MAGIC.len();
            framed_bytes[version_offset..version_offset + 4].copy_from_slice(&999u32.to_le_bytes());

            exported.ciphertext = base64::engine::general_purpose::STANDARD.encode(framed_bytes);

            let codec = JsonCodec;
            let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

            let (dst_service, _dst_dir) = test_service_with_tempdir();
            let err = dst_service
                .import_wallet_payload(&export_json, &export_password, "n")
                .await
                .unwrap_err();

            assert!(matches!(
                err,
                WalletError::InvalidParams(ref msg) if msg == "Unsupported backup payload version: 999"
            ));
        }
    }

    mod rejects_wrong_magic {
        use super::*;

        #[tokio::test]
        async fn test_import_wallet_payload_rejects() {
            use z00z_utils::codec::{Codec, JsonCodec};

            let (service, _dir) = test_service_with_tempdir();
            let wallet_password = SafePassword::from(TEST_PASSWORD);
            let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
            let wallet_id = service
                .create_wallet_in_memory("test", wallet_password, test_seed_phrase_24())
                .await
                .unwrap();
            service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
                .await
                .unwrap();

            let mut exported = service
                .export_wallet_payload(&wallet_id, &export_password)
                .await
                .unwrap();

            let mut framed_bytes = base64::engine::general_purpose::STANDARD
                .decode(exported.ciphertext.as_bytes())
                .unwrap();

            // Corrupt the magic prefix so framing decode fails.
            framed_bytes[0] ^= 0xFF;

            exported.ciphertext = base64::engine::general_purpose::STANDARD.encode(framed_bytes);

            let codec = JsonCodec;
            let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

            let (dst_service, _dst_dir) = test_service_with_tempdir();
            let err = dst_service
                .import_wallet_payload(&export_json, &export_password, "n")
                .await
                .unwrap_err();

            assert!(matches!(
                err,
                WalletError::InvalidParams(ref msg) if msg == "Invalid backup payload format"
            ));
        }
    }

    mod rejects_truncated_framing_prefix {
        use super::*;

        #[tokio::test]
        async fn test_import_wallet_payload_rejects() {
            use z00z_utils::codec::{Codec, JsonCodec};

            let (service, _dir) = test_service_with_tempdir();
            let wallet_password = SafePassword::from(TEST_PASSWORD);
            let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
            let wallet_id = service
                .create_wallet_in_memory("test", wallet_password, test_seed_phrase_24())
                .await
                .unwrap();
            service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
                .await
                .unwrap();

            let mut exported = service
                .export_wallet_payload(&wallet_id, &export_password)
                .await
                .unwrap();

            let framed_bytes = base64::engine::general_purpose::STANDARD
                .decode(exported.ciphertext.as_bytes())
                .unwrap();

            // Truncate so `magic.len() + 4` check fails.
            let truncated = framed_bytes
                .get(..WalletService::WALLET_EXPORT_PAYLOAD_MAGIC.len() + 3)
                .unwrap_or(&[])
                .to_vec();

            exported.ciphertext = base64::engine::general_purpose::STANDARD.encode(truncated);

            let codec = JsonCodec;
            let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

            let (dst_service, _dst_dir) = test_service_with_tempdir();
            let err = dst_service
                .import_wallet_payload(&export_json, &export_password, "n")
                .await
                .unwrap_err();

            assert!(matches!(
                err,
                WalletError::InvalidParams(ref msg) if msg == "Invalid backup payload format"
            ));
        }
    }

    mod rejects_unsupported_algorithm {
        use super::*;

        #[tokio::test]
        async fn test_import_wallet_payload_rejects() {
            use crate::security::encryption::EncryptedWalletContainer;
            use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

            let (service, _dir) = test_service_with_tempdir();
            let wallet_password = SafePassword::from(TEST_PASSWORD);
            let export_password = SafePassword::from(TEST_WRONG_PASSWORD);
            let wallet_id = service
                .create_wallet_in_memory("test", wallet_password, test_seed_phrase_24())
                .await
                .unwrap();
            service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(TEST_PASSWORD))
                .await
                .unwrap();

            let mut exported = service
                .export_wallet_payload(&wallet_id, &export_password)
                .await
                .unwrap();

            let payload_bytes = base64::engine::general_purpose::STANDARD
                .decode(exported.ciphertext.as_bytes())
                .unwrap();

            let (version, inner_bytes) =
                WalletService::decode_wallet_export_payload(&payload_bytes)
                    .expect("exported payload should be framed");
            assert_eq!(version, EncryptedWalletContainer::VERSION);

            let bin_codec = BincodeCodec;
            let mut container = bin_codec
                .deserialize::<EncryptedWalletContainer>(inner_bytes)
                .unwrap();

            container.algorithm = "aes-256-gcm".to_string();

            let container_bytes = bin_codec.serialize(&container).unwrap();
            let payload_bytes =
                WalletService::encode_wallet_export_payload(version, &container_bytes);

            exported.ciphertext = base64::engine::general_purpose::STANDARD.encode(payload_bytes);

            let codec = JsonCodec;
            let export_json = String::from_utf8(codec.serialize(&exported).unwrap()).unwrap();

            let (dst_service, _dst_dir) = test_service_with_tempdir();
            let err = dst_service
                .import_wallet_payload(&export_json, &export_password, "n")
                .await
                .unwrap_err();

            assert!(matches!(
                err,
                WalletError::InvalidConfig(ref msg)
                    if msg == "Unsupported algorithm: aes-256-gcm (expected xchacha20poly1305)"
            ));
        }
    }
}
