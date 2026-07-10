use super::initial_objects::create_initial_objects;
use super::{
    encode_bincode, encode_encrypted_object_record, encode_object_id_be, encrypt_secret_record,
    generate_16_bytes, object_id_to_be_bytes, store_required_meta, store_wallet_save_seq, to_hex,
    try_lock_wallet_file, update_wallet_integrity, wallet_tmp_path, zstd_encode_to_writer, Arc,
    Database, EncryptedObjectRecord, Hidden, KdfParams, MasterKeyRecord, MnemonicLanguage, Path,
    PersistWalletId, RngCoreExt, SafePassword, SecretsKind, SecretsRecord, SecureRngProvider,
    SeedMainEntropyPayload, SeedMainMnemonicLanguage, SeedPhrase24, SystemRngProvider,
    SystemTimeProvider, TimeProvider, WalletError, WalletIdentity, WalletIo, WalletRedbKeyManager,
    WalletResult, Z00ZWalletIo, META_APP_OBJECT_ID, META_CHAIN_OBJECT_ID,
    META_DERIVATION_STATE_OBJECT_ID, META_KEYS_OBJECT_ID, META_SCAN_STATE_OBJECT_ID,
    META_STEALTH_META_OBJECT_ID, META_TABLE, META_TOFU_PINS_OBJECT_ID, OBJECTS_TABLE,
    SECRETS_MASTER_KEY, SECRETS_SEED_MAIN, SECRETS_TABLE, WLT_ZSTD_LEVEL,
};

#[cfg(test)]
use super::super::{
    create_take_wlt_failpoint_db, inc_create_wlt_commit_ct, take_create_wlt_fp_commit,
    take_create_wlt_fp_meta, take_create_wlt_fp_secrets,
};

pub(super) fn store_master_key_record(
    secrets: &mut redb::Table<'_, &str, &[u8]>,
    record: &MasterKeyRecord,
) -> WalletResult<()> {
    secrets
        .insert(SECRETS_MASTER_KEY, encode_bincode(record)?.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets insert failed: {e}")))?;
    Ok(())
}

pub(super) fn store_seed_secret(
    secrets: &mut redb::Table<'_, &str, &[u8]>,
    rng: &mut impl rand::RngCore,
    wallet_id: &PersistWalletId,
    master_key: &[u8; 32],
    seed_phrase: &str,
) -> WalletResult<()> {
    let seed_phrase = SeedPhrase24::parse_in(MnemonicLanguage::English, seed_phrase)
        .map_err(|_| WalletError::InvalidParams("Invalid seed phrase".to_string()))?;

    let mut entropy_bytes = seed_phrase
        .to_bip39_entropy_bytes()
        .map_err(|_| WalletError::InvalidParams("Invalid seed phrase".to_string()))?;

    if entropy_bytes.len() != 32 {
        entropy_bytes.fill(0);
        return Err(WalletError::InvalidParams(
            "Seed phrase must produce 32 bytes of entropy".to_string(),
        ));
    }

    let mut payload = SeedMainEntropyPayload {
        entropy_bytes,
        mnemonic_language: SeedMainMnemonicLanguage::English,
    };

    let mut plaintext = encode_bincode(&payload)?;
    payload.entropy_bytes.fill(0);

    let envelope =
        encrypt_secret_record(rng, wallet_id, SECRETS_SEED_MAIN, master_key, &plaintext)?;
    plaintext.fill(0);

    let record = SecretsRecord {
        kind: SecretsKind::Seed,
        label: "main".to_string(),
        version: 1,
        envelope,
    };

    secrets
        .insert(SECRETS_SEED_MAIN, encode_bincode(&record)?.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets insert failed: {e}")))?;
    Ok(())
}

pub(crate) fn store_object(
    objects: &mut redb::Table<'_, &[u8], &[u8]>,
    object_id: u128,
    record: &EncryptedObjectRecord,
) -> WalletResult<()> {
    let key = encode_object_id_be(object_id);
    let value = encode_encrypted_object_record(record)?;

    objects
        .insert(key.as_slice(), value.as_slice())
        .map_err(|e| WalletError::InvalidConfig(format!("redb objects insert failed: {e}")))?;
    Ok(())
}

pub fn create_wallet_store<R: SecureRngProvider + Clone>(
    path: &Path,
    wallet_id: &PersistWalletId,
    password: &SafePassword,
    seed_phrase: &str,
    identity: &WalletIdentity,
    rng_provider: R,
) -> WalletResult<()> {
    let time_provider = SystemTimeProvider;
    let io = Arc::new(Z00ZWalletIo);

    create_wlt_with_deps(
        path,
        wallet_id,
        password,
        seed_phrase,
        identity,
        rng_provider,
        &time_provider,
        io,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_wlt_with_deps<R: SecureRngProvider + Clone>(
    path: &Path,
    wallet_id: &PersistWalletId,
    password: &SafePassword,
    seed_phrase: &str,
    identity: &WalletIdentity,
    rng_provider: R,
    time_provider: &dyn TimeProvider,
    io: Arc<dyn WalletIo>,
) -> WalletResult<()> {
    // The wallet-file lock lives next to the target `.wlt`, so the parent
    // directory must exist before we try to create `<wallet>.wlt.lock`.
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            io.create_dir_all(parent)?;
        }
    }

    // Creation is exclusive: ensure only one creator/opener exists per wallet path.
    let _create_lock = try_lock_wallet_file(path, time_provider, io.clone())?;

    let now_ms = time_provider.compat_unix_timestamp_millis();

    if io.path_exists(path)? {
        return Err(WalletError::WalletAlreadyExists);
    }

    // IMPORTANT: Use a single RNG stream for all random values created during wallet init.
    // MockRngProvider resets its RNG on every call, so re-calling `rng_provider.rng()`
    // would otherwise yield repeated IDs/nonces and overwrite objects.
    let mut rng = rng_provider.rng();

    // Best-effort cleanup of stale temp file from a previous crash.
    let stale_tmp_path = wallet_tmp_path(path);
    io.remove_file_best_effort(&stale_tmp_path);

    // Create the RedB file in tmpfs to avoid writing an uncompressed `.wlt` to disk.
    // The final `.wlt` is written as zstd bytes using an atomic replace.
    let shm_dir = Path::new("/dev/shm");
    if !io.path_exists(shm_dir)? {
        return Err(WalletError::InvalidConfig(
            "/dev/shm is required to create zstd .wlt without writing plaintext to disk"
                .to_string(),
        ));
    }

    let mut tmp_rng = SystemRngProvider.rng();
    let tmp_id = generate_16_bytes(&mut tmp_rng);
    let work_name = format!("wallet-init.work.{}", to_hex(&tmp_id));
    let work_path = shm_dir.join(work_name);
    io.remove_file_best_effort(&work_path);

    let result = (|| {
        let km = WalletRedbKeyManager::new();

        // Determinism (tests): all randomness must come from the injected RNG stream.
        // Production callers pass `SystemRngProvider`, so this remains secure by default.
        let mut kdf_salt = vec![0u8; 32];
        rng.fill_bytes_ext(&mut kdf_salt);
        let kdf_params = KdfParams::default_argon2id_with_salt(kdf_salt);

        let mut master_key_bytes = [0u8; 32];
        rng.fill_bytes_ext(&mut master_key_bytes);
        let master_key: Hidden<[u8; 32]> = Hidden::hide(master_key_bytes);
        let master_key_record = km
            .wrap_master_key(wallet_id, password, &master_key, &kdf_params)
            .map_err(|_| WalletError::InvalidConfig("master key wrap failed".to_string()))?;

        let derived = km
            .derive_wallet_keys(&master_key)
            .map_err(|_| WalletError::InvalidConfig("key derivation failed".to_string()))?;

        let initial_objects =
            create_initial_objects(&mut rng, wallet_id, &derived, identity, now_ms)?;

        // Crash-safe create: build the RedB file in tmpfs first.
        let db = Database::create(&work_path)
            .map_err(|_| WalletError::InvalidConfig("wallet create failed".to_string()))?;

        #[cfg(test)]
        {
            if create_take_wlt_failpoint_db() {
                return Err(WalletError::InvalidConfig(
                    "injected create_wallet_store failure".to_string(),
                ));
            }
        }

        let write_txn = db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        {
            let mut meta = write_txn
                .open_table(META_TABLE)
                .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;

            store_required_meta(&mut meta, wallet_id, &kdf_params, identity, now_ms)?;

            meta.insert(
                META_DERIVATION_STATE_OBJECT_ID,
                object_id_to_be_bytes(initial_objects.derivation_state_id).as_slice(),
            )
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
            meta.insert(
                META_SCAN_STATE_OBJECT_ID,
                object_id_to_be_bytes(initial_objects.scan_state_id).as_slice(),
            )
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
            meta.insert(
                META_APP_OBJECT_ID,
                object_id_to_be_bytes(initial_objects.app_id).as_slice(),
            )
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
            meta.insert(
                META_CHAIN_OBJECT_ID,
                object_id_to_be_bytes(initial_objects.chain_id).as_slice(),
            )
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
            meta.insert(
                META_KEYS_OBJECT_ID,
                object_id_to_be_bytes(initial_objects.keys_id).as_slice(),
            )
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
            meta.insert(
                META_STEALTH_META_OBJECT_ID,
                object_id_to_be_bytes(initial_objects.stealth_meta_id).as_slice(),
            )
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
            meta.insert(
                META_TOFU_PINS_OBJECT_ID,
                object_id_to_be_bytes(initial_objects.tofu_pins_id).as_slice(),
            )
            .map_err(|e| WalletError::InvalidConfig(format!("redb meta insert failed: {e}")))?;
        }

        #[cfg(test)]
        {
            if take_create_wlt_fp_meta() {
                return Err(WalletError::InvalidConfig(
                    "injected create_wallet_store failure".to_string(),
                ));
            }
        }

        {
            let mut secrets = write_txn.open_table(SECRETS_TABLE).map_err(|e| {
                WalletError::InvalidConfig(format!("redb open secrets failed: {e}"))
            })?;
            store_master_key_record(&mut secrets, &master_key_record)?;
            store_seed_secret(
                &mut secrets,
                &mut rng,
                wallet_id,
                master_key.reveal(),
                seed_phrase,
            )?;
        }

        #[cfg(test)]
        {
            if take_create_wlt_fp_secrets() {
                return Err(WalletError::InvalidConfig(
                    "injected create_wallet_store failure".to_string(),
                ));
            }
        }

        {
            let mut objects = write_txn.open_table(OBJECTS_TABLE).map_err(|e| {
                WalletError::InvalidConfig(format!("redb open objects failed: {e}"))
            })?;

            store_object(
                &mut objects,
                initial_objects.wallet_root_id,
                &initial_objects.wallet_root_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.main_account_id,
                &initial_objects.account_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.derivation_state_id,
                &initial_objects.derivation_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.scan_state_id,
                &initial_objects.scan_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.app_id,
                &initial_objects.app_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.chain_id,
                &initial_objects.chain_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.keys_id,
                &initial_objects.keys_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.stealth_meta_id,
                &initial_objects.stealth_meta_record,
            )?;
            store_object(
                &mut objects,
                initial_objects.tofu_pins_id,
                &initial_objects.tofu_pins_record,
            )?;
        }

        {
            let mut meta = write_txn
                .open_table(META_TABLE)
                .map_err(|e| WalletError::InvalidConfig(format!("redb open meta failed: {e}")))?;
            store_wallet_save_seq(&mut meta, 0)?;
            update_wallet_integrity(&mut meta, 0)?;
        }

        #[cfg(test)]
        {
            if take_create_wlt_fp_commit() {
                return Err(WalletError::InvalidConfig(
                    "injected create_wallet_store failure".to_string(),
                ));
            }
        }

        write_txn
            .commit()
            .map_err(|e| WalletError::InvalidConfig(format!("redb commit failed: {e}")))?;

        #[cfg(test)]
        inc_create_wlt_commit_ct();

        // Close the database before reading its bytes.
        drop(db);

        // Stream-compress the whole wallet file and persist it atomically.
        // This avoids allocating full Vec<u8> for compressed data.
        {
            let work_file = std::fs::File::open(&work_path).map_err(|e| {
                WalletError::InvalidConfig(format!("failed to open work file: {e}"))
            })?;

            io.atomic_write_file_streaming(path, &mut |mut out| {
                zstd_encode_to_writer(&mut &work_file, &mut out, WLT_ZSTD_LEVEL)
                    .map_err(std::io::Error::other)
            })
            .map_err(|e| {
                WalletError::InvalidConfig(format!("wallet zstd compression failed: {e}"))
            })?;
        }

        Ok(())
    })();

    if result.is_err() {
        // Best-effort cleanup of the final path (should not exist yet) and temp artifacts.
        io.remove_file_best_effort(path);
        io.remove_file_best_effort(&work_path);
    } else {
        // Best-effort cleanup of tmpfs work file.
        io.remove_file_best_effort(&work_path);
    }

    result
}
