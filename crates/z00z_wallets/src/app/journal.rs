//! Durable idempotency and reconciliation journal.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use z00z_app_api::{AppError, BoundedId, OperationProjection, OperationState, ReconciliationState};
use z00z_utils::{
    codec::{AppWireCodec, AppWireEnvelope, AppWireField},
    io::{atomic_write_file_private, SecureDir},
};

use super::redaction::internal_error;

const MAX_RECORD_BYTES: u64 = 4_096;
const MAX_JOURNAL_RECORDS: usize = 4_096;

/// One persisted effect identity and terminal/reconciliation state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalRecord {
    /// Server-owned operation identifier.
    pub id: BoundedId,
    /// Digest of the canonical request that created the operation.
    pub request_digest: [u8; 32],
    /// Persisted monotonic deadline.
    pub deadline: u64,
    /// Current public operation state.
    pub state: OperationState,
    /// Current reconciliation state.
    pub reconciliation: ReconciliationState,
    /// Monotonic journal revision.
    pub revision: u64,
    /// Number of committed domain effects.
    pub effect_count: u32,
}

impl JournalRecord {
    /// Convert the durable record into the sanitized API projection.
    #[must_use]
    pub fn projection(&self) -> OperationProjection {
        OperationProjection {
            id: self.id.clone(),
            state: self.state,
            reconciliation: self.reconciliation,
            revision: self.revision,
            effect_count: self.effect_count,
        }
    }
}

/// Filesystem-backed journal. Every transition is atomically persisted.
pub struct DurableJournal {
    root: PathBuf,
    directory: SecureDir,
    records: BTreeMap<String, JournalRecord>,
}

impl DurableJournal {
    /// Open the journal and verify every existing record before use.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AppError> {
        let root = root.as_ref().to_path_buf();
        let directory =
            SecureDir::ensure_private(&root).map_err(|_| internal_error("journal-create"))?;
        let mut journal = Self {
            root,
            directory,
            records: BTreeMap::new(),
        };
        journal.reload()?;
        Ok(journal)
    }

    /// Persist operation identity and request digest before any domain effect.
    pub fn persist_identity(
        &mut self,
        id: BoundedId,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        if request_digest == [0; 32] || deadline == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        if let Some(existing) = self.records.get(id.as_str()) {
            if existing.request_digest != request_digest || existing.deadline != deadline {
                return Err(AppError::Conflict);
            }
            return Ok(existing.projection());
        }
        let record = JournalRecord {
            id,
            request_digest,
            deadline,
            state: OperationState::Pending,
            reconciliation: ReconciliationState::Settling,
            revision: 1,
            effect_count: 0,
        };
        self.persist(record)
    }

    /// Persist the single effect boundary for an operation.
    pub fn mark_effect(&mut self, id: &BoundedId) -> Result<OperationProjection, AppError> {
        let record = self.records.get(id.as_str()).ok_or(AppError::NotFound)?;
        if record.state != OperationState::Pending || record.effect_count != 0 {
            return Err(AppError::Conflict);
        }
        self.transition(
            id,
            OperationState::Running,
            ReconciliationState::Settling,
            1,
        )
    }

    /// Persist an ambiguous post-effect response state.
    pub fn mark_unknown(&mut self, id: &BoundedId) -> Result<OperationProjection, AppError> {
        let record = self.records.get(id.as_str()).ok_or(AppError::NotFound)?;
        if record.state != OperationState::Running || record.effect_count != 1 {
            return Err(AppError::Conflict);
        }
        self.transition(
            id,
            OperationState::UnknownOutcome,
            ReconciliationState::Settling,
            1,
        )
    }

    /// Reconcile only after the final owner proves exactly one durable effect.
    pub fn reconcile(
        &mut self,
        id: &BoundedId,
        owner_effect_count: u32,
    ) -> Result<OperationProjection, AppError> {
        let record = self.records.get(id.as_str()).ok_or(AppError::NotFound)?;
        if !matches!(
            record.state,
            OperationState::Running | OperationState::UnknownOutcome
        ) || record.effect_count != 1
            || owner_effect_count != 1
        {
            return Err(AppError::Conflict);
        }
        self.transition(
            id,
            OperationState::Succeeded,
            ReconciliationState::Settled,
            1,
        )
    }

    /// Cancel only when both journal and final owner prove zero effect.
    pub fn cancel(
        &mut self,
        id: &BoundedId,
        owner_effect_count: u32,
    ) -> Result<OperationProjection, AppError> {
        let record = self.records.get(id.as_str()).ok_or(AppError::NotFound)?;
        if owner_effect_count != 0
            || record.effect_count != 0
            || record.state != OperationState::Pending
        {
            return Err(AppError::Conflict);
        }
        self.transition(
            id,
            OperationState::Cancelled,
            ReconciliationState::Settled,
            0,
        )
    }

    /// Read one sanitized operation projection.
    pub fn get(&self, id: &BoundedId) -> Result<OperationProjection, AppError> {
        self.records
            .get(id.as_str())
            .map(JournalRecord::projection)
            .ok_or(AppError::NotFound)
    }

    /// Read the durable identity used to query the accountable final owner.
    pub fn record(&self, id: &BoundedId) -> Result<JournalRecord, AppError> {
        self.records
            .get(id.as_str())
            .cloned()
            .ok_or(AppError::NotFound)
    }

    /// Persist a projection returned by another durable final owner.
    pub fn persist_projection(
        &mut self,
        projection: OperationProjection,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        projection.validate()?;
        if request_digest == [0; 32] || deadline == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        if let Some(existing) = self.records.get(projection.id.as_str()) {
            if existing.request_digest != request_digest {
                return Err(AppError::Conflict);
            }
            return Ok(existing.projection());
        }
        self.persist(JournalRecord {
            id: projection.id,
            request_digest,
            deadline,
            state: projection.state,
            reconciliation: projection.reconciliation,
            revision: projection.revision,
            effect_count: projection.effect_count,
        })
    }

    /// Update a lifecycle projection while retaining its original request identity.
    pub fn update_projection(
        &mut self,
        projection: OperationProjection,
    ) -> Result<OperationProjection, AppError> {
        projection.validate()?;
        let mut record = self
            .records
            .get(projection.id.as_str())
            .cloned()
            .ok_or(AppError::NotFound)?;
        if matches!(
            record.state,
            OperationState::Succeeded | OperationState::Failed | OperationState::Cancelled
        ) && projection != record.projection()
        {
            return Err(AppError::Conflict);
        }
        if projection.revision < record.revision || projection.effect_count < record.effect_count {
            return Err(AppError::Conflict);
        }
        record.state = projection.state;
        record.reconciliation = projection.reconciliation;
        record.revision = projection.revision;
        record.effect_count = projection.effect_count;
        self.persist(record)
    }

    /// Reload and recover nonterminal records from final-owner observations.
    pub fn recover_with<F>(
        &mut self,
        mut observe_owner: F,
    ) -> Result<Vec<OperationProjection>, AppError>
    where
        F: FnMut(&JournalRecord) -> Result<Option<u32>, AppError>,
    {
        self.reload()?;
        let records = self
            .records
            .values()
            .filter(|record| {
                !matches!(
                    record.state,
                    OperationState::Succeeded | OperationState::Failed | OperationState::Cancelled
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut recovered = Vec::with_capacity(records.len());
        for record in records {
            let Some(owner_effect_count) = observe_owner(&record)? else {
                continue;
            };
            match (record.state, owner_effect_count) {
                (OperationState::Pending, 0) => {}
                (OperationState::Pending, 1) => {
                    self.mark_effect(&record.id)?;
                    recovered.push(self.reconcile(&record.id, 1)?);
                }
                (OperationState::Running | OperationState::UnknownOutcome, 1) => {
                    recovered.push(self.reconcile(&record.id, 1)?);
                }
                _ => return Err(AppError::Conflict),
            }
        }
        Ok(recovered)
    }

    fn transition(
        &mut self,
        id: &BoundedId,
        state: OperationState,
        reconciliation: ReconciliationState,
        effect_count: u32,
    ) -> Result<OperationProjection, AppError> {
        let mut record = self
            .records
            .get(id.as_str())
            .cloned()
            .ok_or(AppError::NotFound)?;
        if effect_count < record.effect_count || effect_count > 1 {
            return Err(AppError::Conflict);
        }
        record.state = state;
        record.reconciliation = reconciliation;
        record.effect_count = effect_count;
        record.revision = record.revision.checked_add(1).ok_or(AppError::Conflict)?;
        self.persist(record)
    }

    fn persist(&mut self, record: JournalRecord) -> Result<OperationProjection, AppError> {
        let path = self.record_path(&record.id);
        let bytes = encode_record(&record)?;
        atomic_write_file_private(path, &bytes).map_err(|_| internal_error("journal-write"))?;
        self.directory
            .sync()
            .map_err(|_| internal_error("journal-sync"))?;
        let projection = record.projection();
        self.records.insert(record.id.as_str().to_owned(), record);
        Ok(projection)
    }

    fn reload(&mut self) -> Result<(), AppError> {
        let mut loaded = BTreeMap::new();
        let names = self
            .directory
            .read_dir_bounded(MAX_JOURNAL_RECORDS)
            .map_err(|_| internal_error("journal-read-dir"))?;
        for name in names {
            let name_path = Path::new(&name);
            if name_path.extension().and_then(|value| value.to_str()) != Some("record") {
                return Err(AppError::IntegrityFailure);
            }
            let bytes = self
                .directory
                .read_file_bounded(name_path, MAX_RECORD_BYTES)
                .map_err(|_| AppError::IntegrityFailure)?;
            let record = decode_record(&bytes)?;
            if loaded
                .insert(record.id.as_str().to_owned(), record)
                .is_some()
            {
                return Err(AppError::Conflict);
            }
        }
        self.records = loaded;
        Ok(())
    }

    fn record_path(&self, id: &BoundedId) -> PathBuf {
        self.root.join(format!("{}.record", id.as_str()))
    }
}

fn encode_record(record: &JournalRecord) -> Result<Vec<u8>, AppError> {
    AppWireCodec
        .encode(&AppWireEnvelope::v1(vec![
            AppWireField::new(1, record.id.as_str().as_bytes().to_vec()),
            AppWireField::new(2, record.request_digest.to_vec()),
            AppWireField::new(3, record.deadline.to_be_bytes().to_vec()),
            AppWireField::new(4, vec![state_code(record.state)]),
            AppWireField::new(5, vec![reconciliation_code(record.reconciliation)]),
            AppWireField::new(6, record.revision.to_be_bytes().to_vec()),
            AppWireField::new(7, record.effect_count.to_be_bytes().to_vec()),
        ]))
        .map_err(|_| AppError::IntegrityFailure)
}

fn decode_record(bytes: &[u8]) -> Result<JournalRecord, AppError> {
    let envelope = AppWireCodec
        .decode(bytes)
        .map_err(|_| AppError::IntegrityFailure)?;
    if envelope.fields.len() != 7
        || envelope
            .fields
            .iter()
            .enumerate()
            .any(|(index, field)| field.id != u16::try_from(index + 1).unwrap_or(0))
    {
        return Err(AppError::IntegrityFailure);
    }
    let id = BoundedId::new(
        std::str::from_utf8(&envelope.fields[0].value)
            .map_err(|_| AppError::IntegrityFailure)?
            .to_owned(),
    )?;
    let request_digest = envelope.fields[1]
        .value
        .as_slice()
        .try_into()
        .map_err(|_| AppError::IntegrityFailure)?;
    let deadline = u64::from_be_bytes(
        envelope.fields[2]
            .value
            .as_slice()
            .try_into()
            .map_err(|_| AppError::IntegrityFailure)?,
    );
    let state = parse_state(envelope.fields[3].value.as_slice())?;
    let reconciliation = parse_reconciliation(envelope.fields[4].value.as_slice())?;
    let revision = u64::from_be_bytes(
        envelope.fields[5]
            .value
            .as_slice()
            .try_into()
            .map_err(|_| AppError::IntegrityFailure)?,
    );
    let effect_count = u32::from_be_bytes(
        envelope.fields[6]
            .value
            .as_slice()
            .try_into()
            .map_err(|_| AppError::IntegrityFailure)?,
    );
    if request_digest == [0; 32] || deadline == 0 || effect_count > 1 {
        return Err(AppError::IntegrityFailure);
    }
    let record = JournalRecord {
        id,
        request_digest,
        deadline,
        state,
        reconciliation,
        revision,
        effect_count,
    };
    record.projection().validate()?;
    Ok(record)
}

const fn state_code(state: OperationState) -> u8 {
    match state {
        OperationState::Pending => 1,
        OperationState::Running => 2,
        OperationState::Succeeded => 3,
        OperationState::Failed => 4,
        OperationState::Cancelled => 5,
        OperationState::UnknownOutcome => 6,
    }
}

fn parse_state(bytes: &[u8]) -> Result<OperationState, AppError> {
    match bytes {
        [1] => Ok(OperationState::Pending),
        [2] => Ok(OperationState::Running),
        [3] => Ok(OperationState::Succeeded),
        [4] => Ok(OperationState::Failed),
        [5] => Ok(OperationState::Cancelled),
        [6] => Ok(OperationState::UnknownOutcome),
        _ => Err(AppError::IntegrityFailure),
    }
}

const fn reconciliation_code(state: ReconciliationState) -> u8 {
    match state {
        ReconciliationState::Settling => 1,
        ReconciliationState::Settled => 2,
        ReconciliationState::Failed => 3,
        ReconciliationState::Conflict => 4,
        ReconciliationState::NeedsAttention => 5,
    }
}

fn parse_reconciliation(bytes: &[u8]) -> Result<ReconciliationState, AppError> {
    match bytes {
        [1] => Ok(ReconciliationState::Settling),
        [2] => Ok(ReconciliationState::Settled),
        [3] => Ok(ReconciliationState::Failed),
        [4] => Ok(ReconciliationState::Conflict),
        [5] => Ok(ReconciliationState::NeedsAttention),
        _ => Err(AppError::IntegrityFailure),
    }
}
