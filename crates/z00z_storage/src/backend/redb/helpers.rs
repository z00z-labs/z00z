use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use redb::{Database, Durability, ReadableTable};
#[cfg(not(test))]
use z00z_utils::io::path_exists;
use z00z_utils::io::prepare_managed_root;
#[cfg(not(test))]
use z00z_utils::time::{SystemTimeProvider, TimeProvider};
use z00z_utils::{
    codec::{BincodeCodec, Codec},
    io::{create_dir_all, hash_root_inputs},
};

use super::{
    state::{LoadState, RecursiveV2CutoverManifestV2, StateMeta},
    validate, RedbBackend, AST_ROOT_TABLE, AST_ROW_TABLE, CLAIM_NULL_TABLE, DB_FILE,
    DEF_ROOT_TABLE, DEF_ROW_TABLE, FEE_REPLAY_TABLE, KEY_ACTIVE, KEY_STATE, META_TABLE,
    OBJECT_DELTA_TABLE, PATH_ROW_TABLE, RECURSIVE_V2_CUTOVER_TABLE, SER_ROOT_TABLE, SER_ROW_TABLE,
};
#[cfg(not(test))]
use crate::settlement::hjmt_config::env_opt;
use crate::{
    backend::error::StoreBackendError,
    settlement::{
        ClaimNullRec, ClaimNullifier, DefinitionId, FeeReplayKey, FeeReplayRec, SerialId,
        SettlementStateRoot,
    },
};
#[cfg(not(test))]
const ROOT_ENV: &str = "Z00Z_STORAGE_REDB_ROOT";
#[cfg(not(test))]
const ROOT_BASE_ENV: &str = "Z00Z_STORAGE_REDB_ROOT_BASE";
const ROOT_HASH_SCHEMA: &str = "storage-redb-root-v1";

impl RedbBackend {
    pub(super) fn new(root: PathBuf) -> Self {
        Self {
            root: Some(root),
            db: OnceLock::new(),
        }
    }

    pub(super) fn off() -> Self {
        Self {
            root: None,
            db: OnceLock::new(),
        }
    }

    pub(super) fn is_on(&self) -> bool {
        self.root.is_some()
    }

    pub(super) fn db(&self) -> Result<&Database, StoreBackendError> {
        db(self)
    }

    pub(super) fn load_state(&self) -> Result<Option<LoadState>, StoreBackendError> {
        let Some(_root) = &self.root else {
            return Ok(None);
        };

        let db = self.db()?;
        let read = db
            .begin_read()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let meta_table = match read.open_table(META_TABLE) {
            Ok(table) => table,
            Err(_) => return Ok(None),
        };
        let Some(meta_bytes) = meta_table
            .get(KEY_STATE)
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        else {
            return Ok(None);
        };

        let codec = BincodeCodec;
        let meta: StateMeta = codec.deserialize(meta_bytes.value())?;
        validate::validate_checkpoint_meta(&read, &meta)?;
        let hjmt_journal = super::hjmt::load_journal(&read, meta.version)?;
        if hjmt_journal.is_none() {
            let message = if legacy_generation_present(&read)? {
                "legacy simple-jmt rows are unsupported by live hjmt reload"
            } else {
                "persisted settlement state is missing live hjmt generation metadata"
            };
            return Err(StoreBackendError::UnsupportedGeneration(
                message.to_string(),
            ));
        }
        let hjmt_terminal_rows = super::hjmt::load_terminal_rows(&read, meta.version)?;
        let hjmt_settlement_path_rows =
            super::hjmt::load_settlement_path_rows(&read, meta.version)?;

        let claim_null_table = read
            .open_table(CLAIM_NULL_TABLE)
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let mut claim_null_rows = Vec::new();
        for entry in claim_null_table
            .iter()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        {
            let (key, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            let Some(key_null) = decode_claim_null_key(key.value(), meta.version) else {
                continue;
            };
            let row = decode_claim_null_row(value.value())?;
            if row.nullifier != key_null {
                return Err(StoreBackendError::Tx(
                    "claim nullifier row does not match persisted replay key".to_owned(),
                ));
            }
            claim_null_rows.push(row);
        }

        let fee_replay_table = match read.open_table(FEE_REPLAY_TABLE) {
            Ok(table) => table,
            Err(_) => {
                if meta.fee_replay_count != 0 || meta.fee_replay_digest != [0u8; 32] {
                    return Err(StoreBackendError::Tx(
                        "fee replay table missing for persisted replay metadata".to_owned(),
                    ));
                }
                let hjmt_root_rows = super::hjmt::load_root_rows(&read, meta.version)?;
                if let Some(journal) = &hjmt_journal {
                    super::hjmt::validate_loaded(
                        journal,
                        meta.version,
                        SettlementStateRoot::settlement_v1(meta.state_root),
                        &hjmt_terminal_rows,
                        &hjmt_settlement_path_rows,
                        &claim_null_rows,
                        &[],
                        &hjmt_root_rows,
                    )?;
                }
                super::hjmt::validate_fee_meta(&meta, &[])?;

                return Ok(Some(LoadState {
                    version: meta.version,
                    state_root: SettlementStateRoot::settlement_v1(meta.state_root),
                    flat_root: meta.flat_root,
                    hjmt_terminal_rows,
                    hjmt_settlement_path_rows,
                    claim_null_rows,
                    fee_replay_rows: Vec::new(),
                    object_delta: load_object_delta(&read, meta.version)?,
                    hjmt_journal,
                }));
            }
        };
        let mut fee_replay_rows = Vec::new();
        for entry in fee_replay_table
            .iter()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        {
            let (key, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            let Some(key_replay) = decode_fee_replay_key(key.value(), meta.version) else {
                continue;
            };
            let row = decode_fee_replay_row(value.value())?;
            if row.replay_key != key_replay {
                return Err(StoreBackendError::Tx(
                    "fee replay row does not match persisted replay key".to_owned(),
                ));
            }
            fee_replay_rows.push(row);
        }

        super::hjmt::validate_fee_meta(&meta, &fee_replay_rows)?;

        let hjmt_root_rows = super::hjmt::load_root_rows(&read, meta.version)?;
        if let Some(journal) = &hjmt_journal {
            super::hjmt::validate_loaded(
                journal,
                meta.version,
                SettlementStateRoot::settlement_v1(meta.state_root),
                &hjmt_terminal_rows,
                &hjmt_settlement_path_rows,
                &claim_null_rows,
                &fee_replay_rows,
                &hjmt_root_rows,
            )?;
        }

        Ok(Some(LoadState {
            version: meta.version,
            state_root: SettlementStateRoot::settlement_v1(meta.state_root),
            flat_root: meta.flat_root,
            hjmt_terminal_rows,
            hjmt_settlement_path_rows,
            claim_null_rows,
            fee_replay_rows,
            object_delta: load_object_delta(&read, meta.version)?,
            hjmt_journal,
        }))
    }

    pub(super) fn install_recursive_v2_cutover(
        &self,
        manifest: &RecursiveV2CutoverManifestV2,
    ) -> Result<(), StoreBackendError> {
        if self.root.is_none() {
            return Err(StoreBackendError::Open(
                "recursive V2 cutover requires durable storage".to_string(),
            ));
        }
        manifest.validate()?;

        let codec = BincodeCodec;
        let manifest_bytes = codec.serialize(manifest)?;
        let db = self.db()?;
        let mut write = db
            .begin_write()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        // This commit is the restart boundary. It must not stay in redb's
        // deferred queue after the API reports a successful cutover.
        write.set_durability(Durability::Immediate);

        let meta_table = write
            .open_table(META_TABLE)
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let meta_bytes = meta_table
            .get(KEY_STATE)
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?
            .ok_or_else(|| {
                StoreBackendError::Tx(
                    "recursive V2 cutover has no persisted active state".to_string(),
                )
            })?;
        let meta: StateMeta = codec.deserialize(meta_bytes.value())?;
        let active = meta_table
            .get(KEY_ACTIVE)
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?
            .ok_or_else(|| {
                StoreBackendError::Tx(
                    "recursive V2 cutover has no persisted active generation".to_string(),
                )
            })?;
        if active.value().len() != 8 {
            return Err(StoreBackendError::Tx(
                "recursive V2 cutover active generation has invalid length".to_string(),
            ));
        }
        let mut active_bytes = [0_u8; 8];
        active_bytes.copy_from_slice(active.value());
        let active_generation = u64::from_be_bytes(active_bytes);
        if meta.version != manifest.storage_generation
            || active_generation != manifest.storage_generation
            || meta.def_root != Some(manifest.expected_definition_root)
        {
            return Err(StoreBackendError::Tx(
                "recursive V2 cutover compare-and-swap mismatch".to_string(),
            ));
        }
        drop(active);
        drop(meta_bytes);
        drop(meta_table);

        let mut cutover_table = write
            .open_table(RECURSIVE_V2_CUTOVER_TABLE)
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        if cutover_table
            .get(&b"installed"[..])
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?
            .is_some()
        {
            return Err(StoreBackendError::Tx(
                "recursive V2 cutover already installed".to_string(),
            ));
        }
        cutover_table
            .insert(&b"installed"[..], manifest_bytes.as_slice())
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        drop(cutover_table);
        write
            .commit()
            .map_err(|err| StoreBackendError::Commit(err.to_string()))
    }

    pub(super) fn load_recursive_v2_cutover(
        &self,
    ) -> Result<Option<RecursiveV2CutoverManifestV2>, StoreBackendError> {
        if self.root.is_none() {
            return Ok(None);
        }
        let read = self
            .db()?
            .begin_read()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let table = match read.open_table(RECURSIVE_V2_CUTOVER_TABLE) {
            Ok(table) => table,
            Err(_) => return Ok(None),
        };
        let Some(bytes) = table
            .get(&b"installed"[..])
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        else {
            return Ok(None);
        };
        let codec = BincodeCodec;
        let manifest: RecursiveV2CutoverManifestV2 = codec.deserialize(bytes.value())?;
        manifest.validate()?;
        Ok(Some(manifest))
    }
}

pub(super) fn fee_replay_digest(rows: &[FeeReplayRec]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut rows = rows.to_vec();
    rows.sort_by_key(|row| row.replay_key);

    let mut hasher = Sha256::new();
    hasher.update(u64::try_from(rows.len()).unwrap_or(u64::MAX).to_be_bytes());
    for row in rows {
        hasher.update(row.replay_key.as_bytes());
        hasher.update(row.replay_digest);
        hasher.update(row.nonce);
        hasher.update(row.transition_id);
        hasher.update(row.domain_id);
        hasher.update(row.payer_commitment);
        hasher.update(row.sponsor_commitment);
        hasher.update(row.budget_units.to_be_bytes());
        hasher.update(row.budget_commitment);
        hasher.update([u8::from(row.support_ref.is_some())]);
        hasher.update(row.support_ref.unwrap_or([0u8; 32]));
        hasher.update(row.failure_policy_id);
        hasher.update(row.expires_at.to_be_bytes());
        hasher.update(row.accepted_at_seq.to_be_bytes());
    }

    hasher.finalize().into()
}

fn legacy_generation_present(read: &redb::ReadTransaction) -> Result<bool, StoreBackendError> {
    for table_def in [
        DEF_ROW_TABLE,
        SER_ROW_TABLE,
        AST_ROW_TABLE,
        PATH_ROW_TABLE,
        DEF_ROOT_TABLE,
        SER_ROOT_TABLE,
        AST_ROOT_TABLE,
    ] {
        if table_has_rows(read, table_def)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn table_has_rows(
    read: &redb::ReadTransaction,
    table_def: redb::TableDefinition<&[u8], &[u8]>,
) -> Result<bool, StoreBackendError> {
    let table = match read.open_table(table_def) {
        Ok(table) => table,
        Err(_) => return Ok(false),
    };
    let mut rows = table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    if let Some(entry) = rows.next() {
        let _ = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        return Ok(true);
    }
    Ok(false)
}

fn load_object_delta(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Option<crate::settlement::ObjectDeltaSetV1>, StoreBackendError> {
    let table = match read.open_table(OBJECT_DELTA_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(None),
    };
    let Some(bytes) = table
        .get(version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    else {
        return Ok(None);
    };
    let codec = BincodeCodec;
    codec
        .deserialize(bytes.value())
        .map(Some)
        .map_err(Into::into)
}

impl Default for RedbBackend {
    fn default() -> Self {
        default_backend()
    }
}

#[cfg(test)]
pub(super) fn managed_default_backend() -> RedbBackend {
    RedbBackend::off()
}

#[cfg(not(test))]
pub(super) fn managed_default_backend() -> RedbBackend {
    RedbBackend::new(managed_default_root())
}

pub(super) fn default_backend() -> RedbBackend {
    #[cfg(test)]
    {
        RedbBackend::off()
    }

    #[cfg(not(test))]
    RedbBackend::new(default_root())
}

pub(super) fn db(backend: &RedbBackend) -> Result<&Database, StoreBackendError> {
    let Some(root) = &backend.root else {
        return Err(StoreBackendError::Open("backend off".to_string()));
    };

    if let Some(db) = backend.db.get() {
        return Ok(db);
    }

    create_dir_all(root)?;
    let db =
        Database::create(db_path(root)).map_err(|err| StoreBackendError::Open(err.to_string()))?;
    let _ = backend.db.set(db);
    backend
        .db
        .get()
        .ok_or_else(|| StoreBackendError::Open("backend init lost".to_string()))
}

fn db_path(root: &Path) -> PathBuf {
    root.join(DB_FILE)
}

#[cfg(not(test))]
fn default_root() -> PathBuf {
    if let Some(root) = env_opt(ROOT_ENV) {
        return PathBuf::from(root);
    }

    create_managed_root(managed_root_base_from_env())
}

#[cfg(not(test))]
fn managed_default_root() -> PathBuf {
    create_managed_root(repo_managed_root_base())
}

#[cfg(not(test))]
fn managed_root_base_from_env() -> PathBuf {
    env_opt(ROOT_BASE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(repo_managed_root_base)
}

#[cfg(not(test))]
fn repo_managed_root_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs")
        .join(".z00z-storage-redb")
}

#[cfg(not(test))]
fn create_managed_root(base: PathBuf) -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let _ = prepare_managed_root(&base, &managed_root_fingerprint())
        .or_else(|_| create_dir_all(&base).map(|_| true));

    let pid = std::process::id();
    let stamp = SystemTimeProvider.try_unix_timestamp_micros().unwrap_or(0);

    for _ in 0..1_000 {
        let next_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = base.join(format!("run-{pid}-{stamp}-{next_id}"));
        let exists = path_exists(&candidate).unwrap_or(true);
        if !exists && create_dir_all(&candidate).is_ok() {
            return candidate;
        }
    }

    base.join(format!("run-{pid}-{stamp}-fallback"))
}

fn managed_root_fingerprint() -> String {
    static VALUE: OnceLock<String> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            hash_root_inputs(
                ROOT_HASH_SCHEMA,
                &[
                    root.join("Cargo.toml"),
                    root.join("Cargo.lock"),
                    root.join(".cargo/config.toml"),
                    root.join("crates/z00z_core/Cargo.toml"),
                    root.join("crates/z00z_crypto/Cargo.toml"),
                    root.join("crates/z00z_storage/Cargo.toml"),
                    root.join("crates/z00z_utils/Cargo.toml"),
                ],
                &[
                    root.join("crates/z00z_core/src"),
                    root.join("crates/z00z_crypto/src"),
                    root.join("crates/z00z_storage/src"),
                    root.join("crates/z00z_utils/src"),
                ],
            )
            .expect("hash storage redb root")
        })
        .clone()
}

pub(super) fn map_store(err: impl std::fmt::Display) -> StoreBackendError {
    StoreBackendError::Tx(err.to_string())
}

pub(super) fn ver_key(version: u64) -> [u8; 8] {
    version.to_be_bytes()
}

pub(super) fn hjmt_terminal_row_key(
    version: u64,
    path: crate::settlement::SettlementPath,
    bucket_id: crate::settlement::BucketId,
) -> Vec<u8> {
    let mut key = Vec::with_capacity(108);
    key.extend_from_slice(&ver_key(version));
    key.extend_from_slice(path.definition_id.as_bytes());
    key.extend_from_slice(&path.serial_id.get().to_be_bytes());
    key.extend_from_slice(bucket_id.as_bytes());
    key.extend_from_slice(path.terminal_id.as_bytes());
    key
}

pub(super) fn hjmt_settlement_path_key(
    version: u64,
    terminal_id: crate::settlement::TerminalId,
) -> Vec<u8> {
    let mut key = Vec::with_capacity(40);
    key.extend_from_slice(&ver_key(version));
    key.extend_from_slice(terminal_id.as_bytes());
    key
}

pub(super) fn claim_null_key(version: u64, nullifier: ClaimNullifier) -> Vec<u8> {
    let mut key = Vec::with_capacity(40);
    key.extend_from_slice(&ver_key(version));
    key.extend_from_slice(nullifier.as_bytes());
    key
}

pub(super) fn fee_replay_key(version: u64, replay_key: FeeReplayKey) -> Vec<u8> {
    let mut key = Vec::with_capacity(40);
    key.extend_from_slice(&ver_key(version));
    key.extend_from_slice(replay_key.as_bytes());
    key
}

pub(super) fn def_root_key(version: u64) -> Vec<u8> {
    ver_key(version).to_vec()
}

pub(super) fn decode_claim_null_key(key: &[u8], version: u64) -> Option<ClaimNullifier> {
    if key.len() <= 8 {
        return None;
    }

    let mut ver = [0u8; 8];
    ver.copy_from_slice(&key[..8]);
    if u64::from_be_bytes(ver) != version {
        return None;
    }

    let raw = &key[8..];
    if raw.len() == 32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(raw);
        return Some(ClaimNullifier::new(bytes));
    }

    None
}

pub(super) fn decode_fee_replay_key(key: &[u8], version: u64) -> Option<FeeReplayKey> {
    if key.len() <= 8 {
        return None;
    }

    let mut ver = [0u8; 8];
    ver.copy_from_slice(&key[..8]);
    if u64::from_be_bytes(ver) != version {
        return None;
    }

    let raw = &key[8..];
    if raw.len() == 32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(raw);
        return Some(FeeReplayKey::new(bytes));
    }

    None
}

pub(super) fn decode_hjmt_terminal_key(
    key: &[u8],
    version: u64,
) -> Option<(
    crate::settlement::SettlementPath,
    crate::settlement::BucketId,
)> {
    if key.len() != 108 {
        return None;
    }

    let mut ver = [0u8; 8];
    ver.copy_from_slice(&key[..8]);
    if u64::from_be_bytes(ver) != version {
        return None;
    }

    let mut def = [0u8; 32];
    def.copy_from_slice(&key[8..40]);
    let mut ser = [0u8; 4];
    ser.copy_from_slice(&key[40..44]);
    let mut bucket = [0u8; 32];
    bucket.copy_from_slice(&key[44..76]);
    let mut terminal = [0u8; 32];
    terminal.copy_from_slice(&key[76..108]);

    Some((
        crate::settlement::SettlementPath::new(
            DefinitionId::new(def),
            SerialId::new(u32::from_be_bytes(ser)),
            crate::settlement::TerminalId::new(terminal),
        ),
        crate::settlement::BucketId::new(bucket),
    ))
}

pub(super) fn decode_hjmt_settlement_path_key(
    key: &[u8],
    version: u64,
) -> Option<crate::settlement::TerminalId> {
    if key.len() != 40 {
        return None;
    }

    let mut ver = [0u8; 8];
    ver.copy_from_slice(&key[..8]);
    if u64::from_be_bytes(ver) != version {
        return None;
    }

    let mut terminal = [0u8; 32];
    terminal.copy_from_slice(&key[8..40]);
    Some(crate::settlement::TerminalId::new(terminal))
}

pub(super) fn decode_claim_null_row(bytes: &[u8]) -> Result<ClaimNullRec, StoreBackendError> {
    let codec = BincodeCodec;
    codec.deserialize(bytes).map_err(StoreBackendError::from)
}

pub(super) fn decode_fee_replay_row(bytes: &[u8]) -> Result<FeeReplayRec, StoreBackendError> {
    let codec = BincodeCodec;
    codec.deserialize(bytes).map_err(StoreBackendError::from)
}

pub(super) fn map_check(err: impl std::fmt::Display) -> StoreBackendError {
    StoreBackendError::Tx(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use z00z_utils::io::{read_to_string, write_file};

    const SRC: &str = include_str!("helpers.rs");

    #[test]
    fn storage_root_fingerprint_scope_contract() {
        let prod_src = SRC
            .split("#[cfg(test)]\nmod tests {")
            .next()
            .expect("production helpers source");
        for needle in [
            "const ROOT_HASH_SCHEMA: &str = \"storage-redb-root-v1\";",
            "\"Cargo.toml\"",
            "\"Cargo.lock\"",
            "\".cargo/config.toml\"",
            "\"crates/z00z_core/src\"",
            "\"crates/z00z_crypto/src\"",
            "\"crates/z00z_storage/src\"",
            "\"crates/z00z_utils/src\"",
            "prepare_managed_root(&base, &managed_root_fingerprint())",
        ] {
            assert!(
                prod_src.contains(needle),
                "storage root fingerprint contract must include {needle}"
            );
        }
        assert!(
            !prod_src.contains("reset_managed_root_once(&base"),
            "default storage root must not clear sibling run directories on each process start"
        );
    }

    #[test]
    fn managed_cleanup_on_hash_drift() {
        let dir = TempDir::new().expect("temp dir");
        let root = dir.path().join(".z00z-storage-redb");
        create_dir_all(&root).expect("create root");
        write_file(root.join(".managed-root-fingerprint"), b"stale").expect("write stale mark");
        write_file(root.join("old.redb"), b"old").expect("write stale payload");

        let cleared =
            prepare_managed_root(&root, &managed_root_fingerprint()).expect("prepare root");

        assert!(cleared, "storage root must clear on fingerprint drift");
        assert!(
            !root.join("old.redb").exists(),
            "stale payload must be removed"
        );
        assert_eq!(
            read_to_string(root.join(".managed-root-fingerprint")).expect("read mark"),
            managed_root_fingerprint()
        );
    }
}
