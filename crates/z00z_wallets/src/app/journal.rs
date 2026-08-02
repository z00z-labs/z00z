//! Durable idempotency and reconciliation journal.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use z00z_app_api::{AppError, BoundedId, OperationProjection, OperationState, ReconciliationState};
use z00z_crypto::hash::sha256_256;
use z00z_utils::{
    codec::{AppWireCodec, AppWireEnvelope, AppWireField},
    io::{atomic_write_file_private, to_lower_hex, SecureDir},
};

use super::redaction::internal_error;

const MAX_RECORD_BYTES: u64 = 4_096;
const MAX_JOURNAL_RECORDS: usize = 4_096;
const MAX_JOURNAL_RECORDS_PER_CLIENT: usize = 256;
const MAX_JOURNAL_SCAN_ENTRIES: usize = MAX_JOURNAL_RECORDS + 1;
const RETAIN_TERMINAL_RECORDS_GLOBAL: usize = 2_048;
const RETAIN_TERMINAL_RECORDS_PER_CLIENT: usize = 64;
const BINDING_DOMAIN: &str = "z00z.app.outer-journal.binding.v2";
const BINDING_LABEL: &str = "canonical-identity";

/// One persisted effect identity and terminal/reconciliation state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalRecord {
    /// Server-owned operation identifier.
    pub id: BoundedId,
    /// Authenticated client that caused the effect.
    pub client_id: BoundedId,
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct JournalBinding {
    operation_id: BoundedId,
    client_id: BoundedId,
    request_digest: [u8; 32],
    deadline: u64,
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
    bindings: BTreeMap<String, JournalBinding>,
    default_client_id: BoundedId,
    #[cfg(test)]
    fail_next_record_write: bool,
}

impl DurableJournal {
    /// Open the journal and verify every existing record before use.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AppError> {
        Self::open_for_client(root, BoundedId::new("journal-default-client")?)
    }

    /// Open the journal with the authenticated client used for new records.
    pub fn open_for_client(
        root: impl AsRef<Path>,
        default_client_id: BoundedId,
    ) -> Result<Self, AppError> {
        let root = root.as_ref().to_path_buf();
        let directory =
            SecureDir::ensure_private(&root).map_err(|_| internal_error("journal-create"))?;
        let mut journal = Self {
            root,
            directory,
            records: BTreeMap::new(),
            bindings: BTreeMap::new(),
            default_client_id,
            #[cfg(test)]
            fail_next_record_write: false,
        };
        journal.reload()?;
        journal.enforce_reopen_policy()?;
        Ok(journal)
    }

    /// Persist operation identity and request digest before any domain effect.
    pub fn persist_identity(
        &mut self,
        id: BoundedId,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        let client_id = self.default_client_id.clone();
        self.persist_identity_for_client(client_id, id, request_digest, deadline)
    }

    /// Persist identity for an explicit authenticated client.
    pub fn persist_identity_for_client(
        &mut self,
        client_id: BoundedId,
        id: BoundedId,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        if request_digest == [0; 32] || deadline == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        if let Some(existing) = self.records.get(id.as_str()) {
            if existing.client_id != client_id
                || existing.request_digest != request_digest
                || existing.deadline != deadline
            {
                return Err(AppError::Conflict);
            }
            return Ok(existing.projection());
        }
        let record = JournalRecord {
            id,
            client_id,
            request_digest,
            deadline,
            state: OperationState::Pending,
            reconciliation: ReconciliationState::Settling,
            revision: 1,
            effect_count: 0,
        };
        self.persist(record)
    }

    /// Durably bind an original request identity before invoking its final owner.
    pub fn bind_request(
        &mut self,
        operation_id: BoundedId,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<(), AppError> {
        let client_id = self.default_client_id.clone();
        self.bind_request_for_client(client_id, operation_id, request_digest, deadline)
    }

    /// Durably bind an original request for an explicit authenticated client.
    pub fn bind_request_for_client(
        &mut self,
        client_id: BoundedId,
        operation_id: BoundedId,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<(), AppError> {
        let binding = JournalBinding {
            operation_id,
            client_id,
            request_digest,
            deadline,
        };
        validate_binding(&binding)?;
        let key = binding_key(&binding);
        if self.bindings.contains_key(&key)
            || self
                .bindings
                .values()
                .any(|existing| existing.operation_id == binding.operation_id)
            || self.records.contains_key(binding.operation_id.as_str())
        {
            return Err(AppError::Conflict);
        }
        self.reclaim_for_admission(&binding.client_id)?;
        if self.total_entries() >= MAX_JOURNAL_RECORDS
            || self.client_entry_count(&binding.client_id) >= MAX_JOURNAL_RECORDS_PER_CLIENT
        {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        atomic_write_file_private(self.binding_path(&key), &encode_binding(&binding)?)
            .map_err(|_| internal_error("journal-binding-write"))?;
        self.directory
            .sync()
            .map_err(|_| internal_error("journal-binding-sync"))?;
        self.bindings.insert(key, binding);
        Ok(())
    }

    /// Remove a pre-effect binding after a known-not-submitted owner failure.
    pub fn discard_binding(
        &mut self,
        operation_id: &BoundedId,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<(), AppError> {
        let binding = JournalBinding {
            operation_id: operation_id.clone(),
            client_id: self.default_client_id.clone(),
            request_digest,
            deadline,
        };
        let key = binding_key(&binding);
        if self.bindings.contains_key(&key) {
            self.directory
                .remove_file(format!("{key}.binding"))
                .map_err(|_| internal_error("journal-binding-remove"))?;
            self.directory
                .sync()
                .map_err(|_| internal_error("journal-binding-sync"))?;
            self.bindings.remove(&key);
        }
        Ok(())
    }

    /// Return the exact unresolved owner operation for an original request retry.
    pub fn bound_operation(
        &self,
        request_digest: [u8; 32],
    ) -> Result<Option<(BoundedId, u64)>, AppError> {
        let mut matches = self.bindings.values().filter(|binding| {
            binding.client_id == self.default_client_id && binding.request_digest == request_digest
        });
        let result = matches
            .next()
            .map(|binding| (binding.operation_id.clone(), binding.deadline));
        if matches.next().is_some() {
            return Err(AppError::IntegrityFailure);
        }
        Ok(result)
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
        let client_id = self.default_client_id.clone();
        self.persist_projection_for_client(client_id, projection, request_digest, deadline)
    }

    /// Persist another owner's projection for an explicit authenticated client.
    pub fn persist_projection_for_client(
        &mut self,
        client_id: BoundedId,
        projection: OperationProjection,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        projection.validate()?;
        if request_digest == [0; 32] || deadline == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        if let Some(existing) = self.records.get(projection.id.as_str()) {
            if existing.client_id != client_id
                || existing.request_digest != request_digest
                || existing.deadline != deadline
            {
                return Err(AppError::Conflict);
            }
            return Ok(existing.projection());
        }
        self.persist(JournalRecord {
            id: projection.id,
            client_id,
            request_digest,
            deadline,
            state: projection.state,
            reconciliation: projection.reconciliation,
            revision: projection.revision,
            effect_count: projection.effect_count,
        })
    }

    /// Recover a missing first journal record only from its durable pre-effect binding.
    pub fn persist_projection_from_binding(
        &mut self,
        projection: OperationProjection,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        let binding = JournalBinding {
            operation_id: projection.id.clone(),
            client_id: self.default_client_id.clone(),
            request_digest,
            deadline,
        };
        let key = binding_key(&binding);
        if self.bindings.get(&key) != Some(&binding) {
            return Err(AppError::IntegrityFailure);
        }
        // A binding and its resulting record are one logical admission slot. Remove
        // it from the in-memory count while the durable binding remains as recovery
        // authority until the record and directory entry are synced.
        let retained = self
            .bindings
            .remove(&key)
            .ok_or(AppError::IntegrityFailure)?;
        let result = self.persist_projection_for_client(
            binding.client_id.clone(),
            projection,
            request_digest,
            deadline,
        );
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                self.bindings.insert(key, retained);
                return Err(error);
            }
        };
        self.directory
            .remove_file(format!("{key}.binding"))
            .map_err(|_| internal_error("journal-binding-remove"))?;
        self.directory
            .sync()
            .map_err(|_| internal_error("journal-binding-sync"))?;
        self.bindings.remove(&key);
        Ok(result)
    }

    /// Update a lifecycle projection while retaining its original request identity.
    pub fn update_projection(
        &mut self,
        projection: OperationProjection,
        request_digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        projection.validate()?;
        let mut record = self
            .records
            .get(projection.id.as_str())
            .cloned()
            .ok_or(AppError::NotFound)?;
        if record.request_digest != request_digest || record.deadline != deadline {
            return Err(AppError::Conflict);
        }
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
        if !valid_record(&record) {
            return Err(AppError::IntegrityFailure);
        }
        let is_new = !self.records.contains_key(record.id.as_str());
        if is_new {
            self.reclaim_for_admission(&record.client_id)?;
            if self.total_entries() >= MAX_JOURNAL_RECORDS
                || self.client_entry_count(&record.client_id) >= MAX_JOURNAL_RECORDS_PER_CLIENT
            {
                return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
            }
        }
        let path = self.record_path(&record.id);
        let bytes = encode_record(&record)?;
        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_record_write) {
            return Err(internal_error("journal-injected-record-write"));
        }
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
        let mut bindings = BTreeMap::new();
        let names = self
            .directory
            .read_dir_bounded(MAX_JOURNAL_SCAN_ENTRIES)
            .map_err(|_| internal_error("journal-read-dir"))?;
        for name in names {
            let name_path = Path::new(&name);
            let extension = name_path.extension().and_then(|value| value.to_str());
            match extension {
                Some("record") => {
                    let stem = name_path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .ok_or(AppError::IntegrityFailure)?;
                    let bytes = self
                        .directory
                        .read_file_bounded(name_path, MAX_RECORD_BYTES)
                        .map_err(|_| AppError::IntegrityFailure)?;
                    let record = decode_record(&bytes)?;
                    if stem != record.id.as_str()
                        || loaded
                            .insert(record.id.as_str().to_owned(), record)
                            .is_some()
                    {
                        return Err(AppError::IntegrityFailure);
                    }
                }
                Some("binding") => {
                    let stem = name_path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .ok_or(AppError::IntegrityFailure)?;
                    let bytes = self
                        .directory
                        .read_file_bounded(name_path, MAX_RECORD_BYTES)
                        .map_err(|_| AppError::IntegrityFailure)?;
                    let binding = decode_binding(&bytes)?;
                    let key = binding_key(&binding);
                    if stem != key || bindings.insert(key, binding).is_some() {
                        return Err(AppError::IntegrityFailure);
                    }
                }
                _ => return Err(AppError::IntegrityFailure),
            }
        }
        let redundant = bindings
            .iter()
            .filter(|(_, binding)| {
                loaded.values().any(|record| {
                    record.id == binding.operation_id
                        && record.client_id == binding.client_id
                        && record.request_digest == binding.request_digest
                        && record.deadline == binding.deadline
                })
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in &redundant {
            self.directory
                .remove_file(format!("{key}.binding"))
                .map_err(|_| internal_error("journal-binding-finalize"))?;
            bindings.remove(key);
        }
        if !redundant.is_empty() {
            self.directory
                .sync()
                .map_err(|_| internal_error("journal-binding-sync"))?;
        }
        self.records = loaded;
        self.bindings = bindings;
        Ok(())
    }

    fn enforce_reopen_policy(&mut self) -> Result<(), AppError> {
        self.compact_terminal_records()?;
        let clients = self
            .records
            .values()
            .map(|record| record.client_id.clone())
            .chain(
                self.bindings
                    .values()
                    .map(|binding| binding.client_id.clone()),
            )
            .collect::<Vec<_>>();
        for client_id in clients {
            while self.client_entry_count(&client_id) > MAX_JOURNAL_RECORDS_PER_CLIENT {
                let Some(id) = self
                    .records
                    .values()
                    .find(|record| record.client_id == client_id && is_terminal(record.state))
                    .map(|record| record.id.as_str().to_owned())
                else {
                    break;
                };
                self.remove_terminal_records(&[id])?;
            }
        }
        while self.total_entries() > MAX_JOURNAL_RECORDS {
            let Some(id) = self
                .records
                .values()
                .find(|record| is_terminal(record.state))
                .map(|record| record.id.as_str().to_owned())
            else {
                break;
            };
            self.remove_terminal_records(&[id])?;
        }
        if self.total_entries() > MAX_JOURNAL_RECORDS {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        let mut counts = BTreeMap::<String, usize>::new();
        for client_id in self
            .records
            .values()
            .map(|record| &record.client_id)
            .chain(self.bindings.values().map(|binding| &binding.client_id))
        {
            let count = counts.entry(client_id.as_str().to_owned()).or_default();
            *count = count.saturating_add(1);
            if *count > MAX_JOURNAL_RECORDS_PER_CLIENT {
                return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
            }
        }
        Ok(())
    }

    fn reclaim_for_admission(&mut self, client_id: &BoundedId) -> Result<(), AppError> {
        self.compact_terminal_records()?;
        while self.client_entry_count(client_id) >= MAX_JOURNAL_RECORDS_PER_CLIENT {
            let Some(id) = self
                .records
                .values()
                .find(|record| &record.client_id == client_id && is_terminal(record.state))
                .map(|record| record.id.as_str().to_owned())
            else {
                break;
            };
            self.remove_terminal_records(&[id])?;
        }
        while self.total_entries() >= MAX_JOURNAL_RECORDS {
            let Some(id) = self
                .records
                .values()
                .find(|record| is_terminal(record.state))
                .map(|record| record.id.as_str().to_owned())
            else {
                break;
            };
            self.remove_terminal_records(&[id])?;
        }
        Ok(())
    }

    fn compact_terminal_records(&mut self) -> Result<(), AppError> {
        let mut per_client = BTreeMap::<String, Vec<String>>::new();
        for record in self
            .records
            .values()
            .filter(|record| is_terminal(record.state))
        {
            per_client
                .entry(record.client_id.as_str().to_owned())
                .or_default()
                .push(record.id.as_str().to_owned());
        }
        let mut removals = Vec::new();
        for ids in per_client.values() {
            let excess = ids.len().saturating_sub(RETAIN_TERMINAL_RECORDS_PER_CLIENT);
            removals.extend(ids.iter().take(excess).cloned());
        }
        let retained_terminal = self
            .records
            .values()
            .filter(|record| is_terminal(record.state))
            .count()
            .saturating_sub(removals.len());
        let global_excess = retained_terminal.saturating_sub(RETAIN_TERMINAL_RECORDS_GLOBAL);
        if global_excess > 0 {
            let extra = self
                .records
                .values()
                .filter(|record| {
                    is_terminal(record.state) && !removals.iter().any(|id| id == record.id.as_str())
                })
                .take(global_excess)
                .map(|record| record.id.as_str().to_owned())
                .collect::<Vec<_>>();
            removals.extend(extra);
        }
        self.remove_terminal_records(&removals)
    }

    fn remove_terminal_records(&mut self, ids: &[String]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        for id in ids {
            let record = self.records.get(id).ok_or(AppError::IntegrityFailure)?;
            if !is_terminal(record.state) {
                return Err(AppError::IntegrityFailure);
            }
            self.directory
                .remove_file(format!("{}.record", record.id.as_str()))
                .map_err(|_| internal_error("journal-compact-remove"))?;
            self.records.remove(id);
        }
        self.directory
            .sync()
            .map_err(|_| internal_error("journal-compact-sync"))
    }

    fn total_entries(&self) -> usize {
        self.records.len().saturating_add(self.bindings.len())
    }

    fn client_entry_count(&self, client_id: &BoundedId) -> usize {
        self.records
            .values()
            .filter(|record| &record.client_id == client_id)
            .count()
            .saturating_add(
                self.bindings
                    .values()
                    .filter(|binding| &binding.client_id == client_id)
                    .count(),
            )
    }

    fn record_path(&self, id: &BoundedId) -> PathBuf {
        self.root.join(format!("{}.record", id.as_str()))
    }

    fn binding_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.binding"))
    }

    #[cfg(test)]
    pub(crate) fn fail_next_record_write_for_test(&mut self) {
        self.fail_next_record_write = true;
    }
}

fn encode_record(record: &JournalRecord) -> Result<Vec<u8>, AppError> {
    AppWireCodec
        .encode(&AppWireEnvelope::v1(vec![
            AppWireField::new(1, record.id.as_str().as_bytes().to_vec()),
            AppWireField::new(2, record.client_id.as_str().as_bytes().to_vec()),
            AppWireField::new(3, record.request_digest.to_vec()),
            AppWireField::new(4, record.deadline.to_be_bytes().to_vec()),
            AppWireField::new(5, vec![state_code(record.state)]),
            AppWireField::new(6, vec![reconciliation_code(record.reconciliation)]),
            AppWireField::new(7, record.revision.to_be_bytes().to_vec()),
            AppWireField::new(8, record.effect_count.to_be_bytes().to_vec()),
        ]))
        .map_err(|_| AppError::IntegrityFailure)
}

fn decode_record(bytes: &[u8]) -> Result<JournalRecord, AppError> {
    let envelope = AppWireCodec
        .decode(bytes)
        .map_err(|_| AppError::IntegrityFailure)?;
    if envelope.fields.len() != 8
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
    let client_id = BoundedId::new(
        std::str::from_utf8(&envelope.fields[1].value)
            .map_err(|_| AppError::IntegrityFailure)?
            .to_owned(),
    )?;
    let request_digest = envelope.fields[2]
        .value
        .as_slice()
        .try_into()
        .map_err(|_| AppError::IntegrityFailure)?;
    let deadline = u64::from_be_bytes(
        envelope.fields[3]
            .value
            .as_slice()
            .try_into()
            .map_err(|_| AppError::IntegrityFailure)?,
    );
    let state = parse_state(envelope.fields[4].value.as_slice())?;
    let reconciliation = parse_reconciliation(envelope.fields[5].value.as_slice())?;
    let revision = u64::from_be_bytes(
        envelope.fields[6]
            .value
            .as_slice()
            .try_into()
            .map_err(|_| AppError::IntegrityFailure)?,
    );
    let effect_count = u32::from_be_bytes(
        envelope.fields[7]
            .value
            .as_slice()
            .try_into()
            .map_err(|_| AppError::IntegrityFailure)?,
    );
    let record = JournalRecord {
        id,
        client_id,
        request_digest,
        deadline,
        state,
        reconciliation,
        revision,
        effect_count,
    };
    if !valid_record(&record) {
        return Err(AppError::IntegrityFailure);
    }
    Ok(record)
}

fn encode_binding(binding: &JournalBinding) -> Result<Vec<u8>, AppError> {
    AppWireCodec
        .encode(&AppWireEnvelope::v1(vec![
            AppWireField::new(1, binding.operation_id.as_str().as_bytes().to_vec()),
            AppWireField::new(2, binding.client_id.as_str().as_bytes().to_vec()),
            AppWireField::new(3, binding.request_digest.to_vec()),
            AppWireField::new(4, binding.deadline.to_be_bytes().to_vec()),
        ]))
        .map_err(|_| AppError::IntegrityFailure)
}

fn decode_binding(bytes: &[u8]) -> Result<JournalBinding, AppError> {
    let envelope = AppWireCodec
        .decode(bytes)
        .map_err(|_| AppError::IntegrityFailure)?;
    if envelope.fields.len() != 4
        || envelope
            .fields
            .iter()
            .enumerate()
            .any(|(index, field)| field.id != u16::try_from(index + 1).unwrap_or(0))
    {
        return Err(AppError::IntegrityFailure);
    }
    let binding = JournalBinding {
        operation_id: BoundedId::new(
            std::str::from_utf8(&envelope.fields[0].value)
                .map_err(|_| AppError::IntegrityFailure)?
                .to_owned(),
        )?,
        client_id: BoundedId::new(
            std::str::from_utf8(&envelope.fields[1].value)
                .map_err(|_| AppError::IntegrityFailure)?
                .to_owned(),
        )?,
        request_digest: envelope.fields[2]
            .value
            .as_slice()
            .try_into()
            .map_err(|_| AppError::IntegrityFailure)?,
        deadline: u64::from_be_bytes(
            envelope.fields[3]
                .value
                .as_slice()
                .try_into()
                .map_err(|_| AppError::IntegrityFailure)?,
        ),
    };
    validate_binding(&binding)?;
    Ok(binding)
}

fn validate_binding(binding: &JournalBinding) -> Result<(), AppError> {
    if binding.request_digest == [0; 32] || binding.deadline == 0 {
        return Err(AppError::IntegrityFailure);
    }
    Ok(())
}

fn binding_key(binding: &JournalBinding) -> String {
    let deadline = binding.deadline.to_be_bytes();
    format!(
        "binding-{}",
        to_lower_hex(&sha256_256(
            BINDING_DOMAIN,
            BINDING_LABEL,
            &[
                binding.operation_id.as_str().as_bytes(),
                binding.client_id.as_str().as_bytes(),
                &binding.request_digest,
                &deadline,
            ],
        ))
    )
}

fn valid_record(record: &JournalRecord) -> bool {
    if record.request_digest == [0; 32] || record.deadline == 0 {
        return false;
    }
    if record.projection().validate().is_err() {
        return false;
    }
    match record.state {
        OperationState::Pending => {
            record.reconciliation == ReconciliationState::Settling
                && record.revision == 1
                && record.effect_count == 0
        }
        OperationState::Running => {
            record.reconciliation == ReconciliationState::Settling
                && record.revision == 2
                && record.effect_count == 1
        }
        OperationState::Succeeded => {
            record.reconciliation == ReconciliationState::Settled
                && matches!(record.revision, 2..=4)
                && record.effect_count == 1
        }
        OperationState::Failed => {
            record.reconciliation == ReconciliationState::Failed
                && matches!(record.revision, 2 | 3)
                && record.effect_count == 0
        }
        OperationState::Cancelled => {
            record.reconciliation == ReconciliationState::Settled
                && record.revision == 2
                && record.effect_count == 0
        }
        OperationState::UnknownOutcome => {
            record.reconciliation == ReconciliationState::Settling
                && matches!((record.revision, record.effect_count), (2, 0) | (3, 1))
        }
    }
}

const fn is_terminal(state: OperationState) -> bool {
    matches!(
        state,
        OperationState::Succeeded | OperationState::Failed | OperationState::Cancelled
    )
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

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt, path::Path};

    use z00z_utils::io::TemporaryDirectory;

    use super::*;

    fn id(value: impl Into<String>) -> BoundedId {
        BoundedId::new(value.into()).expect("bounded test id")
    }

    fn record(
        operation: impl Into<String>,
        client: impl Into<String>,
        state: OperationState,
        reconciliation: ReconciliationState,
        revision: u64,
        effect_count: u32,
    ) -> JournalRecord {
        JournalRecord {
            id: id(operation),
            client_id: id(client),
            request_digest: [7; 32],
            deadline: 100,
            state,
            reconciliation,
            revision,
            effect_count,
        }
    }

    fn pending(operation: impl Into<String>, client: impl Into<String>) -> JournalRecord {
        record(
            operation,
            client,
            OperationState::Pending,
            ReconciliationState::Settling,
            1,
            0,
        )
    }

    fn succeeded(operation: impl Into<String>, client: impl Into<String>) -> JournalRecord {
        record(
            operation,
            client,
            OperationState::Succeeded,
            ReconciliationState::Settled,
            2,
            1,
        )
    }

    fn write_record(root: &Path, record: &JournalRecord) {
        let path = root.join(format!("{}.record", record.id.as_str()));
        fs::write(&path, encode_record(record).expect("encode journal record"))
            .expect("write journal record");
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .expect("set private record permissions");
    }

    fn assert_decode_rejected(record: JournalRecord) {
        let bytes = encode_record(&record).expect("encode mutated record");
        assert_eq!(decode_record(&bytes), Err(AppError::IntegrityFailure));
    }

    #[test]
    fn decoder_rejects_full_state_revision_effect_schema_mutations() {
        let mut pending_revision = pending("schema-pending-revision", "schema-client");
        pending_revision.revision = 2;
        assert_decode_rejected(pending_revision);

        let mut pending_effect = pending("schema-pending-effect", "schema-client");
        pending_effect.effect_count = 1;
        assert_decode_rejected(pending_effect);

        let mut running_revision = record(
            "schema-running-revision",
            "schema-client",
            OperationState::Running,
            ReconciliationState::Settling,
            2,
            1,
        );
        running_revision.revision = 3;
        assert_decode_rejected(running_revision);

        let mut running_effect = record(
            "schema-running-effect",
            "schema-client",
            OperationState::Running,
            ReconciliationState::Settling,
            2,
            1,
        );
        running_effect.effect_count = 0;
        assert_decode_rejected(running_effect);

        let mut succeeded_effect = succeeded("schema-succeeded-effect", "schema-client");
        succeeded_effect.effect_count = 0;
        assert_decode_rejected(succeeded_effect);

        let mut failed_effect = record(
            "schema-failed-effect",
            "schema-client",
            OperationState::Failed,
            ReconciliationState::Failed,
            2,
            0,
        );
        failed_effect.effect_count = 1;
        assert_decode_rejected(failed_effect);

        let mut cancelled_effect = record(
            "schema-cancelled-effect",
            "schema-client",
            OperationState::Cancelled,
            ReconciliationState::Settled,
            2,
            0,
        );
        cancelled_effect.effect_count = 1;
        assert_decode_rejected(cancelled_effect);

        let mut mismatched_reconciliation = succeeded("schema-reconciliation", "schema-client");
        mismatched_reconciliation.reconciliation = ReconciliationState::Settling;
        assert_decode_rejected(mismatched_reconciliation);

        let mut zero_revision = succeeded("schema-zero-revision", "schema-client");
        zero_revision.revision = 0;
        assert_decode_rejected(zero_revision);

        let mut succeeded_future_revision = succeeded("schema-succeeded-future", "schema-client");
        succeeded_future_revision.revision = 5;
        assert_decode_rejected(succeeded_future_revision);

        let mut failed_conflict = record(
            "schema-failed-conflict",
            "schema-client",
            OperationState::Failed,
            ReconciliationState::Conflict,
            2,
            0,
        );
        assert_decode_rejected(failed_conflict.clone());
        failed_conflict.reconciliation = ReconciliationState::NeedsAttention;
        assert_decode_rejected(failed_conflict);

        let mut unknown_conflict = record(
            "schema-unknown-conflict",
            "schema-client",
            OperationState::UnknownOutcome,
            ReconciliationState::Conflict,
            2,
            0,
        );
        assert_decode_rejected(unknown_conflict.clone());
        unknown_conflict.reconciliation = ReconciliationState::NeedsAttention;
        assert_decode_rejected(unknown_conflict);

        let unknown_wrong_pair = record(
            "schema-unknown-pair",
            "schema-client",
            OperationState::UnknownOutcome,
            ReconciliationState::Settling,
            3,
            0,
        );
        assert_decode_rejected(unknown_wrong_pair);
    }

    #[test]
    fn reopen_rejects_record_and_binding_filename_identity_mutations() {
        let root = TemporaryDirectory::new().expect("temporary root");
        let journal_root = root.path().join("record-name");
        let journal = DurableJournal::open(&journal_root).expect("create journal");
        let fixture = pending("embedded-operation", "filename-client");
        atomic_write_file_private(
            journal_root.join("different-operation.record"),
            &encode_record(&fixture).expect("encode fixture"),
        )
        .expect("write mismatched record");
        drop(journal);
        assert!(matches!(
            DurableJournal::open(&journal_root),
            Err(AppError::IntegrityFailure)
        ));

        let binding_root = root.path().join("binding-name");
        let mut journal =
            DurableJournal::open_for_client(&binding_root, id("filename-binding-client"))
                .expect("create binding journal");
        journal
            .bind_request(id("filename-binding-operation"), [8; 32], 100)
            .expect("bind request");
        let original = fs::read_dir(&binding_root)
            .expect("read binding journal")
            .next()
            .expect("binding entry")
            .expect("binding dir entry")
            .path();
        fs::rename(original, binding_root.join("binding-wrong.binding")).expect("rename binding");
        drop(journal);
        assert!(matches!(
            DurableJournal::open_for_client(&binding_root, id("filename-binding-client")),
            Err(AppError::IntegrityFailure)
        ));
    }

    #[test]
    fn failed_first_record_write_retains_binding_for_reopen_recovery() {
        let root = TemporaryDirectory::new().expect("temporary root");
        let client = id("binding-recovery-client");
        let projection = OperationProjection {
            id: id("binding-recovery-operation"),
            state: OperationState::Succeeded,
            reconciliation: ReconciliationState::Settled,
            revision: 2,
            effect_count: 1,
        };
        let mut journal =
            DurableJournal::open_for_client(root.path(), client.clone()).expect("open journal");
        journal
            .bind_request(projection.id.clone(), [9; 32], 100)
            .expect("bind request");
        assert_eq!(
            journal.bind_request(projection.id.clone(), [9; 32], 101),
            Err(AppError::Conflict)
        );
        journal.fail_next_record_write_for_test();
        assert!(journal
            .persist_projection_from_binding(projection.clone(), [9; 32], 100)
            .is_err());
        drop(journal);

        let mut reopened =
            DurableJournal::open_for_client(root.path(), client.clone()).expect("reopen binding");
        assert_eq!(
            reopened
                .persist_projection_from_binding(projection.clone(), [9; 32], 100)
                .expect("recover projection"),
            projection
        );
        drop(reopened);
        assert_eq!(
            DurableJournal::open_for_client(root.path(), client)
                .expect("reopen recovered record")
                .get(&projection.id)
                .expect("read recovered record"),
            projection
        );
    }

    #[test]
    fn repeated_identity_recovers_only_the_exact_bound_operation() {
        let root = TemporaryDirectory::new().expect("temporary exact binding root");
        let client = id("exact-binding-client");
        let first = OperationProjection {
            id: id("exact-binding-first"),
            state: OperationState::Succeeded,
            reconciliation: ReconciliationState::Settled,
            revision: 2,
            effect_count: 1,
        };
        let second = OperationProjection {
            id: id("exact-binding-second"),
            ..first.clone()
        };
        let mut journal = DurableJournal::open_for_client(root.path(), client.clone())
            .expect("open exact binding journal");
        journal
            .bind_request(first.id.clone(), [12; 32], 100)
            .expect("bind first identical request");
        journal
            .persist_projection_from_binding(first.clone(), [12; 32], 100)
            .expect("persist first identical request");
        journal
            .bind_request(second.id.clone(), [12; 32], 100)
            .expect("bind second identical request");
        drop(journal);

        let mut reopened = DurableJournal::open_for_client(root.path(), client)
            .expect("reopen exact binding journal");
        assert_eq!(
            reopened
                .bound_operation([12; 32])
                .expect("read exact binding"),
            Some((second.id.clone(), 100))
        );
        assert_eq!(
            reopened.persist_projection_from_binding(first, [12; 32], 100),
            Err(AppError::IntegrityFailure)
        );
        assert_eq!(
            reopened
                .persist_projection_from_binding(second.clone(), [12; 32], 100)
                .expect("persist exact second operation"),
            second
        );
    }

    #[test]
    fn per_client_admission_reclaims_only_terminal_records() {
        let root = TemporaryDirectory::new().expect("temporary root");
        let client = id("admission-client");
        {
            let journal = DurableJournal::open_for_client(root.path(), client.clone())
                .expect("create journal");
            for index in 0..RETAIN_TERMINAL_RECORDS_PER_CLIENT {
                write_record(
                    root.path(),
                    &succeeded(format!("terminal-{index:03}"), client.as_str()),
                );
            }
            for index in 0..(MAX_JOURNAL_RECORDS_PER_CLIENT - RETAIN_TERMINAL_RECORDS_PER_CLIENT) {
                write_record(
                    root.path(),
                    &pending(format!("unresolved-{index:03}"), client.as_str()),
                );
            }
            drop(journal);
        }
        let mut journal =
            DurableJournal::open_for_client(root.path(), client.clone()).expect("reopen journal");
        journal
            .bind_request(id("admission-binding"), [10; 32], 100)
            .expect("admit binding");
        assert_eq!(
            journal.client_entry_count(&client),
            MAX_JOURNAL_RECORDS_PER_CLIENT
        );
        for index in 0..(MAX_JOURNAL_RECORDS_PER_CLIENT - RETAIN_TERMINAL_RECORDS_PER_CLIENT) {
            assert!(journal
                .records
                .contains_key(&format!("unresolved-{index:03}")));
        }
        assert_eq!(
            journal
                .records
                .values()
                .filter(|record| is_terminal(record.state))
                .count(),
            RETAIN_TERMINAL_RECORDS_PER_CLIENT - 1
        );
    }

    #[test]
    fn reopen_4096_accepts_and_4097_compacts_terminal_or_rejects_unresolved() {
        let root = TemporaryDirectory::new().expect("temporary root");
        let journal = DurableJournal::open(root.path()).expect("create journal");
        for client_index in 0..16 {
            for operation_index in 0..MAX_JOURNAL_RECORDS_PER_CLIENT {
                write_record(
                    root.path(),
                    &pending(
                        format!("global-{client_index:02}-{operation_index:03}"),
                        format!("global-client-{client_index:02}"),
                    ),
                );
            }
        }
        drop(journal);
        let exact = DurableJournal::open(root.path()).expect("reopen exactly 4096");
        assert_eq!(exact.total_entries(), MAX_JOURNAL_RECORDS);
        drop(exact);

        let overflow = succeeded("overflow-terminal", "overflow-client");
        write_record(root.path(), &overflow);
        let compacted = DurableJournal::open(root.path()).expect("compact 4097th terminal");
        assert_eq!(compacted.total_entries(), MAX_JOURNAL_RECORDS);
        assert!(compacted
            .records
            .values()
            .all(|record| !is_terminal(record.state)));
        drop(compacted);

        write_record(
            root.path(),
            &pending("overflow-unresolved", "overflow-client"),
        );
        assert!(matches!(
            DurableJournal::open(root.path()),
            Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds))
        ));
    }
}
