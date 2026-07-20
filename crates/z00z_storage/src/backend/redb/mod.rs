use std::{path::PathBuf, sync::OnceLock};

use redb::{Database, ReadableTable, TableDefinition};

mod helpers;
pub(crate) mod hjmt;
pub(crate) mod state;
mod validate;

use crate::backend::error::StoreBackendError;
use crate::backend::JournalBackend;

const META_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_meta");
const DEF_ROW_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_def_rows");
const SER_ROW_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_ser_rows");
const AST_ROW_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_rows");
const PATH_ROW_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_paths");
const HJMT_TERMINAL_ROW_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_terminal_rows");
const HJMT_SETTLEMENT_PATH_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_settlement_path_rows");
const DEF_ROOT_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_def_roots");
const SER_ROOT_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_ser_roots");
const AST_ROOT_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_tree_roots");
const CLAIM_NULL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_claim_nulls");
const FEE_REPLAY_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_fee_replays");
const OBJECT_DELTA_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_object_deltas");
const SNAP_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_prep_snapshots");
const DRAFT_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_drafts");
const CHECK_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_checkpoints");
const EXEC_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_execs");
const LINK_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("settlement_cp_links");
const RECURSIVE_V2_CUTOVER_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_recursive_v2_cutover");

const KEY_ACTIVE: &[u8] = b"active_version";
const KEY_STATE: &[u8] = b"state_meta";
const KEY_STATE_ROOT: &[u8] = b"state_root";
const KEY_SNAP_ID: &[u8] = b"snap_id";
const KEY_DRAFT_ID: &[u8] = b"draft_id";
const KEY_CHECK_ID: &[u8] = b"check_id";
const KEY_EXEC_ID: &[u8] = b"exec_id";
const DB_FILE: &str = "settlement_state.redb";

#[cfg(test)]
pub(crate) fn recursive_v2_cutover_crash_point(stage: &str) {
    const STAGE_ENV: &str = "Z00Z_RECURSIVE_V2_CUTOVER_CRASH_STAGE";
    if std::env::var(STAGE_ENV).as_deref() == Ok(stage) {
        // `process::exit` intentionally skips Rust destructors. The parent
        // process must establish restart state from redb recovery, not from an
        // orderly transaction or store drop.
        std::process::exit(86);
    }
}

struct RedbBackend {
    root: Option<PathBuf>,
    db: OnceLock<Database>,
}

#[derive(Default)]
pub(crate) struct StoragePlane {
    inner: RedbBackend,
}

impl StoragePlane {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            inner: RedbBackend::new(root),
        }
    }

    pub(crate) fn managed_default() -> Self {
        Self {
            inner: helpers::managed_default_backend(),
        }
    }

    pub(crate) fn off() -> Self {
        Self {
            inner: RedbBackend::off(),
        }
    }

    pub(crate) fn is_on(&self) -> bool {
        self.inner.is_on()
    }

    pub(crate) fn load_state(&self) -> Result<Option<state::LoadState>, StoreBackendError> {
        self.inner.load_state()
    }

    pub(crate) fn load_hjmt_state_at(
        &self,
        version: u64,
    ) -> Result<Option<state::LoadState>, StoreBackendError> {
        self.inner.load_hjmt_state_at(version)
    }

    pub(crate) fn sync_hjmt_work(
        &self,
        work: hjmt::HjmtPersistWork,
    ) -> Result<(), StoreBackendError> {
        self.inner.sync_hjmt_work(work)
    }

    pub(crate) fn hjmt_last_version_for_path(
        &self,
        path: crate::settlement::SettlementPath,
    ) -> Result<Option<u64>, StoreBackendError> {
        self.inner.hjmt_last_version_for_path(path)
    }

    /// Atomically compare the active HJMT generation and install the sole V2
    /// cutover manifest. A cutover is a once-only authority transition, not a
    /// replayable state update.
    pub(crate) fn install_recursive_v2_cutover(
        &self,
        manifest: &state::RecursiveV2CutoverManifestV2,
    ) -> Result<(), StoreBackendError> {
        self.inner.install_recursive_v2_cutover(manifest)
    }

    pub(crate) fn load_recursive_v2_cutover(
        &self,
    ) -> Result<Option<state::RecursiveV2CutoverManifestV2>, StoreBackendError> {
        self.inner.load_recursive_v2_cutover()
    }
}

impl JournalBackend for StoragePlane {
    type Error = StoreBackendError;

    fn recover_journal(&self) -> Result<(), Self::Error> {
        self.inner.recover_hjmt_journals()
    }
}

fn guard_write_head(write: &redb::WriteTransaction, version: u64) -> Result<(), StoreBackendError> {
    let meta_table = write
        .open_table(META_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let active = match meta_table
        .get(KEY_ACTIVE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        Some(bytes) => {
            let raw = bytes.value();
            if raw.len() != 8 {
                return Err(StoreBackendError::Tx(
                    "active version metadata has invalid length".to_string(),
                ));
            }
            let mut ver = [0u8; 8];
            ver.copy_from_slice(raw);
            Some(u64::from_be_bytes(ver))
        }
        None => None,
    };
    drop(meta_table);

    if version == 1 {
        if active.is_none() {
            return Ok(());
        }
    } else if active == Some(version.saturating_sub(1)) {
        return Ok(());
    }

    Err(StoreBackendError::Tx(format!(
        "durable version head mismatch: expected previous version {}, got {:?}",
        version.saturating_sub(1),
        active
    )))
}
