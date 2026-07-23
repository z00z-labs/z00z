//! Fork-bound local recovery snapshots for the live Nova accumulator.
//!
//! Snapshot files and journal records are local recovery state. They are not
//! proof bodies, network objects, retention tickets, or deletion authority.

use std::{
    collections::BTreeSet,
    fs::File,
    sync::atomic::{AtomicU64, Ordering},
};

use fs2::FileExt;
use z00z_crypto::sha256_256;
use z00z_utils::io::{SecureDir, Write};
use zeroize::Zeroizing;

use super::{
    nova::NovaRecoveryImageV2,
    recursive_measurement::{
        NovaCadenceManifestV2, NOVA_HOT_CAP_BYTES_V2, NOVA_IMAGE_MAX_BYTES_V2,
        NOVA_JOURNAL_MAX_BYTES_V2, NOVA_JOURNAL_MAX_ENTRIES_V2, NOVA_SNAPSHOT_MAX_BYTES_V2,
    },
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, NOVA_SNAPSHOT_DOMAIN_V2,
        RECURSIVE_OBJECT_PREHEADER_BYTES_V2, RECURSIVE_PROFILE_MANIFEST_DIGEST_V2,
        RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
    },
};
use crate::CheckpointError;

const SNAPSHOT_MAGIC_V2: [u8; 4] = *b"ZNAS";
const SNAPSHOT_WIRE_VERSION_V2: u16 = 2;
const SNAPSHOT_DIGEST_LABEL_V2: &str = "registry_framed_snapshot";
const IMAGE_DIGEST_LABEL_V2: &str = "accumulator_image";
const JOURNAL_DIGEST_LABEL_V2: &str = "recovery_journal_record";
const SNAPSHOT_PAYLOAD_BYTES_V2: usize = 4 + 2 + 4 + 4 + 2 + 8 + 8 + 32 + 1 + 32 + 10 * 32 + 8;
const JOURNAL_RECORD_BYTES_V2: usize = 1 + 8 + 32 + 32 + 32;
type NovaRecoveryJournalStateV2 = (BTreeSet<[u8; 32]>, BTreeSet<[u8; 32]>);

/// Bindings captured from the already-verified running accumulator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NovaRecoveryBindingsV2 {
    pub(crate) authority_generation: u64,
    pub(crate) parameter_generation: u32,
    pub(crate) runtime_profile_generation: u16,
    pub(crate) height: u64,
    pub(crate) steps: u64,
    pub(crate) checkpoint_id: [u8; 32],
    pub(crate) predecessor: Option<[u8; 32]>,
    pub(crate) chain_context_digest: [u8; 32],
    pub(crate) config_digest: [u8; 32],
    pub(crate) policy_digest: [u8; 32],
    pub(crate) authority_digest: [u8; 32],
    pub(crate) predicate_digest: [u8; 32],
    pub(crate) profile_digest: [u8; 32],
    pub(crate) verifier_bundle_digest: [u8; 32],
}

/// Local-only, registry-framed recovery image.
#[derive(Clone, PartialEq, Eq)]
pub struct NovaAccumulatorSnapshotV2 {
    bindings: NovaRecoveryBindingsV2,
    cadence_manifest_digest: [u8; 32],
    runtime_profile_manifest_digest: [u8; 32],
    image_digest: [u8; 32],
    framed_bytes: Zeroizing<Vec<u8>>,
}

impl core::fmt::Debug for NovaAccumulatorSnapshotV2 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NovaAccumulatorSnapshotV2")
            .field("bindings", &self.bindings)
            .field("cadence_manifest_digest", &self.cadence_manifest_digest)
            .field(
                "runtime_profile_manifest_digest",
                &self.runtime_profile_manifest_digest,
            )
            .field("image_digest", &self.image_digest)
            .field("framed_bytes", &"<redacted>")
            .field("encoded_len", &self.framed_bytes.len())
            .finish()
    }
}

impl NovaAccumulatorSnapshotV2 {
    pub(crate) fn capture(
        bindings: NovaRecoveryBindingsV2,
        image: NovaRecoveryImageV2,
    ) -> Result<Self, CheckpointError> {
        let cadence = NovaCadenceManifestV2::authority_pinned();
        cadence.validate()?;
        validate_bindings(bindings, cadence)?;
        if image.bytes.is_empty() || image.bytes.len() > NOVA_IMAGE_MAX_BYTES_V2 {
            return Err(CheckpointError::Authority);
        }
        let image_digest = sha256_256(
            NOVA_SNAPSHOT_DOMAIN_V2,
            IMAGE_DIGEST_LABEL_V2,
            &[&image.bytes],
        );
        let cadence_manifest_digest = cadence.digest();
        let runtime_profile_manifest_digest = RECURSIVE_PROFILE_MANIFEST_DIGEST_V2;
        let payload_len = SNAPSHOT_PAYLOAD_BYTES_V2
            .checked_add(image.bytes.len())
            .ok_or(CheckpointError::Overflow)?;
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let header = registry.encode_preheader(
            RecursiveBoundedObjectV2::NovaAccumulatorSnapshot,
            payload_len,
        )?;
        let mut framed_bytes = Zeroizing::new(Vec::with_capacity(header.len() + payload_len));
        framed_bytes.extend_from_slice(&header);
        framed_bytes.extend_from_slice(&SNAPSHOT_MAGIC_V2);
        framed_bytes.extend_from_slice(&SNAPSHOT_WIRE_VERSION_V2.to_le_bytes());
        framed_bytes.extend_from_slice(
            &u32::try_from(bindings.authority_generation)
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        framed_bytes.extend_from_slice(&bindings.parameter_generation.to_le_bytes());
        framed_bytes.extend_from_slice(&bindings.runtime_profile_generation.to_le_bytes());
        framed_bytes.extend_from_slice(&bindings.height.to_le_bytes());
        framed_bytes.extend_from_slice(&bindings.steps.to_le_bytes());
        framed_bytes.extend_from_slice(&bindings.checkpoint_id);
        framed_bytes.push(u8::from(bindings.predecessor.is_some()));
        framed_bytes.extend_from_slice(&bindings.predecessor.unwrap_or([0; 32]));
        for digest in [
            bindings.chain_context_digest,
            bindings.config_digest,
            bindings.policy_digest,
            bindings.authority_digest,
            bindings.predicate_digest,
            bindings.profile_digest,
            bindings.verifier_bundle_digest,
            cadence_manifest_digest,
            runtime_profile_manifest_digest,
            image_digest,
        ] {
            framed_bytes.extend_from_slice(&digest);
        }
        framed_bytes.extend_from_slice(&(image.bytes.len() as u64).to_le_bytes());
        framed_bytes.extend_from_slice(&image.bytes);
        if framed_bytes.len() > NOVA_SNAPSHOT_MAX_BYTES_V2 {
            return Err(CheckpointError::Limit);
        }
        Ok(Self {
            bindings,
            cadence_manifest_digest,
            runtime_profile_manifest_digest,
            image_digest,
            framed_bytes,
        })
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<(Self, NovaRecoveryImageV2), CheckpointError> {
        if bytes.len() > NOVA_SNAPSHOT_MAX_BYTES_V2 {
            return Err(CheckpointError::Limit);
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        let validated = registry
            .validate_preheader(bytes, RecursiveBoundedObjectV2::NovaAccumulatorSnapshot)?;
        if validated.header_len != RECURSIVE_OBJECT_PREHEADER_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let payload = bytes
            .get(validated.header_len..)
            .ok_or(CheckpointError::Canonical)?;
        if payload.len() < SNAPSHOT_PAYLOAD_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let mut reader = SnapshotReaderV2::new(payload);
        if reader.array::<4>()? != SNAPSHOT_MAGIC_V2 || reader.u16()? != SNAPSHOT_WIRE_VERSION_V2 {
            return Err(CheckpointError::Canonical);
        }
        let authority_generation = u64::from(reader.u32()?);
        let parameter_generation = reader.u32()?;
        let runtime_profile_generation = reader.u16()?;
        let height = reader.u64()?;
        let steps = reader.u64()?;
        let checkpoint_id = reader.array()?;
        let predecessor_present = reader.byte()?;
        let predecessor_bytes = reader.array()?;
        let predecessor = match predecessor_present {
            0 if predecessor_bytes == [0; 32] => None,
            1 if predecessor_bytes != [0; 32] => Some(predecessor_bytes),
            _ => return Err(CheckpointError::Canonical),
        };
        let bindings = NovaRecoveryBindingsV2 {
            authority_generation,
            parameter_generation,
            runtime_profile_generation,
            height,
            steps,
            checkpoint_id,
            predecessor,
            chain_context_digest: reader.array()?,
            config_digest: reader.array()?,
            policy_digest: reader.array()?,
            authority_digest: reader.array()?,
            predicate_digest: reader.array()?,
            profile_digest: reader.array()?,
            verifier_bundle_digest: reader.array()?,
        };
        let cadence_manifest_digest = reader.array()?;
        let runtime_profile_manifest_digest = reader.array()?;
        let encoded_image_digest: [u8; 32] = reader.array()?;
        let image_len = usize::try_from(reader.u64()?).map_err(|_| CheckpointError::Limit)?;
        if image_len == 0 || image_len > NOVA_IMAGE_MAX_BYTES_V2 {
            return Err(CheckpointError::Limit);
        }
        let image_bytes = Zeroizing::new(reader.take(image_len)?.to_vec());
        if !reader.done() {
            return Err(CheckpointError::Canonical);
        }
        let cadence = NovaCadenceManifestV2::authority_pinned();
        validate_bindings(bindings, cadence)?;
        if cadence_manifest_digest != cadence.digest()
            || runtime_profile_manifest_digest != RECURSIVE_PROFILE_MANIFEST_DIGEST_V2
        {
            return Err(CheckpointError::Authority);
        }
        let image_digest = sha256_256(
            NOVA_SNAPSHOT_DOMAIN_V2,
            IMAGE_DIGEST_LABEL_V2,
            &[&image_bytes],
        );
        if encoded_image_digest != image_digest {
            return Err(CheckpointError::Canonical);
        }
        let snapshot = Self {
            bindings,
            cadence_manifest_digest,
            runtime_profile_manifest_digest,
            image_digest,
            framed_bytes: Zeroizing::new(bytes.to_vec()),
        };
        Ok((snapshot, NovaRecoveryImageV2 { bytes: image_bytes }))
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        sha256_256(
            NOVA_SNAPSHOT_DOMAIN_V2,
            SNAPSHOT_DIGEST_LABEL_V2,
            &[&self.framed_bytes],
        )
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.bindings.height
    }

    #[must_use]
    pub const fn steps(&self) -> u64 {
        self.bindings.steps
    }

    #[must_use]
    pub const fn checkpoint_id(&self) -> [u8; 32] {
        self.bindings.checkpoint_id
    }

    #[must_use]
    pub const fn predecessor(&self) -> Option<[u8; 32]> {
        self.bindings.predecessor
    }

    #[must_use]
    pub fn encoded_len(&self) -> usize {
        self.framed_bytes.len()
    }

    pub(crate) fn framed_bytes(&self) -> &[u8] {
        &self.framed_bytes
    }

    pub(crate) fn validate_image(&self, bytes: &[u8]) -> Result<(), CheckpointError> {
        if bytes.is_empty()
            || bytes.len() > NOVA_IMAGE_MAX_BYTES_V2
            || sha256_256(NOVA_SNAPSHOT_DOMAIN_V2, IMAGE_DIGEST_LABEL_V2, &[bytes])
                != self.image_digest
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    pub(crate) const fn bindings(&self) -> NovaRecoveryBindingsV2 {
        self.bindings
    }
}

fn validate_bindings(
    bindings: NovaRecoveryBindingsV2,
    cadence: NovaCadenceManifestV2,
) -> Result<(), CheckpointError> {
    if bindings.authority_generation != u64::from(cadence.authority_generation())
        || bindings.parameter_generation != cadence.parameter_generation()
        || bindings.runtime_profile_generation != RECURSIVE_RUNTIME_PROFILE_GENERATION_V2
        || bindings.authority_generation > u64::from(u32::MAX)
        || bindings.height == 0
        || bindings.steps == 0
        || bindings.checkpoint_id == [0; 32]
        || bindings.predecessor == Some([0; 32])
        || bindings.chain_context_digest == [0; 32]
        || bindings.config_digest == [0; 32]
        || bindings.policy_digest == [0; 32]
        || bindings.authority_digest == [0; 32]
        || bindings.predicate_digest == [0; 32]
        || bindings.profile_digest == [0; 32]
        || bindings.verifier_bundle_digest == [0; 32]
    {
        return Err(CheckpointError::Authority);
    }
    Ok(())
}

struct SnapshotReaderV2<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> SnapshotReaderV2<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], CheckpointError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(CheckpointError::Overflow)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(CheckpointError::Canonical)?;
        self.offset = end;
        Ok(value)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], CheckpointError> {
        self.take(N)?
            .try_into()
            .map_err(|_| CheckpointError::Canonical)
    }

    fn byte(&mut self) -> Result<u8, CheckpointError> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, CheckpointError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, CheckpointError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, CheckpointError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn done(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

/// Append-only recovery journal event. None authorizes deletion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NovaRecoveryJournalKindV2 {
    SnapshotCommitted = 1,
    SnapshotQuarantined = 2,
    ForkQuarantined = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NovaRecoveryJournalRecordV2 {
    kind: NovaRecoveryJournalKindV2,
    height: u64,
    snapshot_digest: [u8; 32],
    predecessor_snapshot_digest: [u8; 32],
    digest: [u8; 32],
}

impl NovaRecoveryJournalRecordV2 {
    fn new(
        kind: NovaRecoveryJournalKindV2,
        height: u64,
        snapshot_digest: [u8; 32],
        predecessor_snapshot_digest: [u8; 32],
    ) -> Result<Self, CheckpointError> {
        if height == 0 || snapshot_digest == [0; 32] {
            return Err(CheckpointError::Invariant);
        }
        let mut record = Self {
            kind,
            height,
            snapshot_digest,
            predecessor_snapshot_digest,
            digest: [0; 32],
        };
        record.digest = sha256_256(
            NOVA_SNAPSHOT_DOMAIN_V2,
            JOURNAL_DIGEST_LABEL_V2,
            &[&record.prefix()],
        );
        Ok(record)
    }

    fn prefix(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(JOURNAL_RECORD_BYTES_V2 - 32);
        bytes.push(self.kind as u8);
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.snapshot_digest);
        bytes.extend_from_slice(&self.predecessor_snapshot_digest);
        bytes
    }

    fn bytes(&self) -> Vec<u8> {
        let mut bytes = self.prefix();
        bytes.extend_from_slice(&self.digest);
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() != JOURNAL_RECORD_BYTES_V2 {
            return Err(CheckpointError::Canonical);
        }
        let kind = match bytes[0] {
            1 => NovaRecoveryJournalKindV2::SnapshotCommitted,
            2 => NovaRecoveryJournalKindV2::SnapshotQuarantined,
            3 => NovaRecoveryJournalKindV2::ForkQuarantined,
            _ => return Err(CheckpointError::Canonical),
        };
        let height = u64::from_le_bytes(
            bytes[1..9]
                .try_into()
                .map_err(|_| CheckpointError::Canonical)?,
        );
        let snapshot_digest = bytes[9..41]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?;
        let predecessor_snapshot_digest = bytes[41..73]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?;
        let digest = bytes[73..105]
            .try_into()
            .map_err(|_| CheckpointError::Canonical)?;
        let record = Self {
            kind,
            height,
            snapshot_digest,
            predecessor_snapshot_digest,
            digest,
        };
        if height == 0
            || snapshot_digest == [0; 32]
            || sha256_256(
                NOVA_SNAPSHOT_DOMAIN_V2,
                JOURNAL_DIGEST_LABEL_V2,
                &[&record.prefix()],
            ) != digest
        {
            return Err(CheckpointError::Canonical);
        }
        Ok(record)
    }
}

/// Bounded hot-set accounting after one committed mutation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NovaRecoveryStoreMetricsV2 {
    pub live_snapshot_count: u16,
    pub live_snapshot_bytes: u64,
    pub active_accumulator_bytes: u64,
    pub journal_entries: u16,
    pub journal_bytes: u64,
    pub total_hot_bytes: u64,
}

pub(crate) struct NovaRecoveryStoreV2 {
    snapshots: SecureDir,
    journal: SecureDir,
    quarantine: SecureDir,
    process_lock: File,
}

impl NovaRecoveryStoreV2 {
    pub(crate) fn open(root: impl AsRef<std::path::Path>) -> Result<Self, CheckpointError> {
        let root = SecureDir::ensure_private(root).map_err(|_| CheckpointError::Storage)?;
        let snapshots = root
            .ensure_dir("snapshots")
            .map_err(|_| CheckpointError::Storage)?;
        let journal = root
            .ensure_dir("journal")
            .map_err(|_| CheckpointError::Storage)?;
        let quarantine = root
            .ensure_dir("quarantine")
            .map_err(|_| CheckpointError::Storage)?;
        let process_lock = root
            .open_lock(".recovery.lock")
            .map_err(|_| CheckpointError::Storage)?;
        let store = Self {
            snapshots,
            journal,
            quarantine,
            process_lock,
        };
        store
            .process_lock
            .lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        let result = store.reconcile_locked();
        FileExt::unlock(&store.process_lock).map_err(|_| CheckpointError::Storage)?;
        result?;
        Ok(store)
    }

    pub(crate) fn commit(
        &self,
        snapshot: &NovaAccumulatorSnapshotV2,
    ) -> Result<NovaRecoveryStoreMetricsV2, CheckpointError> {
        self.process_lock
            .lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.commit_locked(snapshot);
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn commit_locked(
        &self,
        snapshot: &NovaAccumulatorSnapshotV2,
    ) -> Result<NovaRecoveryStoreMetricsV2, CheckpointError> {
        let mut live = self.live_snapshots()?;
        if let Some(latest) = live.last() {
            if snapshot.height() <= latest.0.height()
                || snapshot.bindings.chain_context_digest != latest.0.bindings.chain_context_digest
                || snapshot.bindings.config_digest != latest.0.bindings.config_digest
                || snapshot.bindings.policy_digest != latest.0.bindings.policy_digest
                || snapshot.bindings.authority_digest != latest.0.bindings.authority_digest
                || snapshot.bindings.profile_digest != latest.0.bindings.profile_digest
                || snapshot.bindings.verifier_bundle_digest
                    != latest.0.bindings.verifier_bundle_digest
            {
                self.require_journal_capacity(1)?;
                let name = snapshot_name(snapshot);
                write_once(&self.quarantine, &name, snapshot.framed_bytes())?;
                self.append_journal(NovaRecoveryJournalRecordV2::new(
                    NovaRecoveryJournalKindV2::ForkQuarantined,
                    snapshot.height(),
                    snapshot.digest(),
                    latest.0.digest(),
                )?)?;
                return Err(CheckpointError::RecursiveRejected(
                    super::recursive_reject::RecursiveCheckpointRejectReasonV2::StepReordered,
                ));
            }
        }

        self.require_journal_capacity(1 + usize::from(live.len() == 2))?;
        let snapshot_name = snapshot_name(snapshot);
        write_once(&self.snapshots, &snapshot_name, snapshot.framed_bytes())?;
        if live.len() == 2 {
            let oldest = live.remove(0);
            self.append_journal(NovaRecoveryJournalRecordV2::new(
                NovaRecoveryJournalKindV2::SnapshotQuarantined,
                oldest.0.height(),
                oldest.0.digest(),
                [0; 32],
            )?)?;
            let quarantine_name = format!("retired-{}", oldest.1);
            self.snapshots
                .rename_to_no_clobber(&oldest.1, &self.quarantine, &quarantine_name)
                .map_err(|_| CheckpointError::Storage)?;
            self.snapshots
                .sync()
                .and_then(|()| self.quarantine.sync())
                .map_err(|_| CheckpointError::Storage)?;
        }
        let predecessor_snapshot_digest = live.last().map_or([0; 32], |entry| entry.0.digest());
        self.append_journal(NovaRecoveryJournalRecordV2::new(
            NovaRecoveryJournalKindV2::SnapshotCommitted,
            snapshot.height(),
            snapshot.digest(),
            predecessor_snapshot_digest,
        )?)?;
        self.metrics(snapshot.encoded_len() as u64)
    }

    pub(crate) fn latest(
        &self,
    ) -> Result<Option<(NovaAccumulatorSnapshotV2, NovaRecoveryImageV2)>, CheckpointError> {
        self.process_lock
            .lock_shared()
            .map_err(|_| CheckpointError::Storage)?;
        let result = self.live_snapshots().and_then(|mut snapshots| {
            snapshots
                .pop()
                .map(|entry| {
                    let bytes = Zeroizing::new(
                        self.snapshots
                            .read_file_bounded(&entry.1, NOVA_SNAPSHOT_MAX_BYTES_V2 as u64)
                            .map_err(|_| CheckpointError::Storage)?,
                    );
                    NovaAccumulatorSnapshotV2::decode(&bytes)
                })
                .transpose()
        });
        FileExt::unlock(&self.process_lock).map_err(|_| CheckpointError::Storage)?;
        result
    }

    fn live_snapshots(&self) -> Result<Vec<(NovaAccumulatorSnapshotV2, String)>, CheckpointError> {
        let (committed, quarantined) = self.journal_state()?;
        let names = self
            .snapshots
            .read_dir_bounded(3)
            .map_err(|_| CheckpointError::Storage)?;
        let mut snapshots = Vec::with_capacity(names.len());
        for name in names {
            let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
            let bytes = Zeroizing::new(
                self.snapshots
                    .read_file_bounded(&name, NOVA_SNAPSHOT_MAX_BYTES_V2 as u64)
                    .map_err(|_| CheckpointError::Storage)?,
            );
            let (snapshot, _) = NovaAccumulatorSnapshotV2::decode(&bytes)?;
            let digest = snapshot.digest();
            if committed.contains(&digest) && !quarantined.contains(&digest) {
                snapshots.push((snapshot, name));
            }
        }
        snapshots.sort_by_key(|entry| entry.0.height());
        if snapshots.len() > 2 {
            return Err(CheckpointError::Limit);
        }
        Ok(snapshots)
    }

    fn reconcile_locked(&self) -> Result<(), CheckpointError> {
        let (committed, quarantined) = self.journal_state()?;
        let names = self
            .snapshots
            .read_dir_bounded(3)
            .map_err(|_| CheckpointError::Storage)?;
        let mut live_count = 0_usize;
        for name in names {
            let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
            let bytes = Zeroizing::new(
                self.snapshots
                    .read_file_bounded(&name, NOVA_SNAPSHOT_MAX_BYTES_V2 as u64)
                    .map_err(|_| CheckpointError::Storage)?,
            );
            let decoded = NovaAccumulatorSnapshotV2::decode(&bytes);
            let (destination, journal_record) = match decoded {
                Ok((snapshot, _)) => {
                    let digest = snapshot.digest();
                    if committed.contains(&digest) && !quarantined.contains(&digest) {
                        live_count = live_count.checked_add(1).ok_or(CheckpointError::Overflow)?;
                        continue;
                    }
                    let record = if quarantined.contains(&digest) {
                        None
                    } else {
                        Some(NovaRecoveryJournalRecordV2::new(
                            NovaRecoveryJournalKindV2::ForkQuarantined,
                            snapshot.height(),
                            digest,
                            [0; 32],
                        )?)
                    };
                    (format!("orphan-{name}"), record)
                }
                Err(_) => (format!("corrupt-{name}"), None),
            };
            if let Some(record) = journal_record {
                self.append_journal(record)?;
            }
            self.snapshots
                .rename_to_no_clobber(&name, &self.quarantine, &destination)
                .map_err(|_| CheckpointError::Storage)?;
            self.snapshots
                .sync()
                .and_then(|()| self.quarantine.sync())
                .map_err(|_| CheckpointError::Storage)?;
        }
        if live_count > 2 {
            return Err(CheckpointError::Limit);
        }
        Ok(())
    }

    fn journal_state(&self) -> Result<NovaRecoveryJournalStateV2, CheckpointError> {
        let names = self
            .journal
            .read_dir_bounded(NOVA_JOURNAL_MAX_ENTRIES_V2)
            .map_err(|_| CheckpointError::Storage)?;
        let mut committed = BTreeSet::new();
        let mut quarantined = BTreeSet::new();
        for name in names {
            let name = name.into_string().map_err(|_| CheckpointError::Canonical)?;
            let bytes = self
                .journal
                .read_file_bounded(&name, JOURNAL_RECORD_BYTES_V2 as u64)
                .map_err(|_| CheckpointError::Storage)?;
            let record = NovaRecoveryJournalRecordV2::decode(&bytes)?;
            let expected_name = format!(
                "{:020}-{}-{}.journal",
                record.height,
                record.kind as u8,
                lowercase_hex(record.digest)
            );
            if name != expected_name {
                return Err(CheckpointError::Canonical);
            }
            match record.kind {
                NovaRecoveryJournalKindV2::SnapshotCommitted => {
                    committed.insert(record.snapshot_digest);
                }
                NovaRecoveryJournalKindV2::SnapshotQuarantined
                | NovaRecoveryJournalKindV2::ForkQuarantined => {
                    quarantined.insert(record.snapshot_digest);
                }
            }
        }
        Ok((committed, quarantined))
    }

    fn append_journal(&self, record: NovaRecoveryJournalRecordV2) -> Result<(), CheckpointError> {
        self.require_journal_capacity(1)?;
        let bytes = record.bytes();
        let name = format!(
            "{:020}-{}-{}.journal",
            record.height,
            record.kind as u8,
            lowercase_hex(record.digest)
        );
        write_once(&self.journal, &name, &bytes)?;
        Ok(())
    }

    fn require_journal_capacity(&self, additional: usize) -> Result<(), CheckpointError> {
        let entries = self
            .journal
            .read_dir_bounded(NOVA_JOURNAL_MAX_ENTRIES_V2)
            .map_err(|_| CheckpointError::Storage)?;
        let next_entries = entries
            .len()
            .checked_add(additional)
            .ok_or(CheckpointError::Overflow)?;
        let next_bytes = next_entries
            .checked_mul(JOURNAL_RECORD_BYTES_V2)
            .ok_or(CheckpointError::Overflow)?;
        if additional == 0
            || next_entries > NOVA_JOURNAL_MAX_ENTRIES_V2
            || next_bytes > NOVA_JOURNAL_MAX_BYTES_V2
        {
            return Err(CheckpointError::Limit);
        }
        Ok(())
    }

    fn metrics(
        &self,
        active_accumulator_bytes: u64,
    ) -> Result<NovaRecoveryStoreMetricsV2, CheckpointError> {
        let live = self.live_snapshots()?;
        let live_snapshot_bytes = live.iter().try_fold(0_u64, |sum, entry| {
            sum.checked_add(entry.0.encoded_len() as u64)
                .ok_or(CheckpointError::Overflow)
        })?;
        let journal_entries = self
            .journal
            .read_dir_bounded(NOVA_JOURNAL_MAX_ENTRIES_V2)
            .map_err(|_| CheckpointError::Storage)?
            .len();
        let journal_bytes = (journal_entries * JOURNAL_RECORD_BYTES_V2) as u64;
        let total_hot_bytes = active_accumulator_bytes
            .checked_add(live_snapshot_bytes)
            .and_then(|value| value.checked_add(journal_bytes))
            .ok_or(CheckpointError::Overflow)?;
        if live.len() > 2
            || total_hot_bytes > NOVA_HOT_CAP_BYTES_V2 as u64
            || active_accumulator_bytes > NOVA_SNAPSHOT_MAX_BYTES_V2 as u64
        {
            return Err(CheckpointError::Limit);
        }
        Ok(NovaRecoveryStoreMetricsV2 {
            live_snapshot_count: live.len() as u16,
            live_snapshot_bytes,
            active_accumulator_bytes,
            journal_entries: journal_entries as u16,
            journal_bytes,
            total_hot_bytes,
        })
    }
}

fn snapshot_name(snapshot: &NovaAccumulatorSnapshotV2) -> String {
    format!(
        "{:020}-{}.snapshot",
        snapshot.height(),
        lowercase_hex(snapshot.digest())
    )
}

fn write_once(directory: &SecureDir, name: &str, bytes: &[u8]) -> Result<(), CheckpointError> {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    if let Ok(existing) = directory.read_file_bounded(name, bytes.len() as u64) {
        return if existing == bytes {
            Ok(())
        } else {
            Err(CheckpointError::Canonical)
        };
    }
    let mut temporary = None;
    for _ in 0..8 {
        let candidate = format!(
            ".tmp-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        if let Ok(file) = directory.create_file(&candidate) {
            temporary = Some((candidate, file));
            break;
        }
    }
    let (temporary_name, mut file) = temporary.ok_or(CheckpointError::Storage)?;
    if file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        // Best-effort cleanup cannot replace the authoritative write failure.
        let _ = directory.remove_file(&temporary_name);
        return Err(CheckpointError::Storage);
    }
    drop(file);
    if directory.rename_no_clobber(&temporary_name, name).is_err() {
        // Best-effort cleanup cannot replace the authoritative rename failure.
        let _ = directory.remove_file(&temporary_name);
        return Err(CheckpointError::Storage);
    }
    directory.sync().map_err(|_| CheckpointError::Storage)
}

fn lowercase_hex(digest: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings(height: u64) -> NovaRecoveryBindingsV2 {
        NovaRecoveryBindingsV2 {
            authority_generation: 2,
            parameter_generation: super::super::version_registry::RECURSIVE_PARAMETER_GENERATION_V2,
            runtime_profile_generation: RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
            height,
            steps: height,
            checkpoint_id: [height as u8; 32],
            predecessor: Some([(height - 1) as u8; 32]),
            chain_context_digest: [1; 32],
            config_digest: [2; 32],
            policy_digest: [3; 32],
            authority_digest: [4; 32],
            predicate_digest: [5; 32],
            profile_digest: [6; 32],
            verifier_bundle_digest: [7; 32],
        }
    }

    fn snapshot(height: u64) -> NovaAccumulatorSnapshotV2 {
        NovaAccumulatorSnapshotV2::capture(
            bindings(height),
            NovaRecoveryImageV2::new(vec![height as u8; 96]),
        )
        .unwrap()
    }

    #[test]
    fn test_recovery_bytes_zeroize() {
        use zeroize::Zeroize;

        let mut image = NovaRecoveryImageV2::new(vec![0xa5; 96]);
        image.bytes.zeroize();
        assert!(image.bytes.is_empty());

        let mut snapshot = snapshot(100);
        snapshot.framed_bytes.zeroize();
        assert!(snapshot.framed_bytes.is_empty());
    }

    #[test]
    fn test_recovery_debug_redacts() {
        const CANARY: &str = "nova-recovery-secret-canary";
        let image = NovaRecoveryImageV2::new(CANARY.as_bytes().to_vec());
        let snapshot = NovaAccumulatorSnapshotV2::capture(bindings(100), image).unwrap();
        assert!(!format!("{snapshot:?}").contains(CANARY));
    }

    #[test]
    fn test_journal_boundaries() {
        let record = NovaRecoveryJournalRecordV2::new(
            NovaRecoveryJournalKindV2::SnapshotCommitted,
            100,
            [1; 32],
            [2; 32],
        )
        .unwrap();
        assert_eq!(record.bytes().len(), JOURNAL_RECORD_BYTES_V2);
        const {
            assert!(
                NOVA_JOURNAL_MAX_ENTRIES_V2 * JOURNAL_RECORD_BYTES_V2 <= NOVA_JOURNAL_MAX_BYTES_V2
            );
        }
        assert_eq!(
            NovaRecoveryJournalRecordV2::decode(&record.bytes()).unwrap(),
            record
        );
    }

    #[test]
    fn test_snapshot_corruption() {
        let snapshot = snapshot(100);
        let (decoded, image) = NovaAccumulatorSnapshotV2::decode(snapshot.framed_bytes()).unwrap();
        assert_eq!(decoded.digest(), snapshot.digest());
        assert_eq!(image.bytes.as_slice(), &[100; 96]);
        assert!(decoded.validate_image(&image.bytes).is_ok());
        let mut mismatched_image = image.bytes.clone();
        mismatched_image[0] ^= 1;
        assert!(decoded.validate_image(&mismatched_image).is_err());

        let mut corrupt = snapshot.framed_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(NovaAccumulatorSnapshotV2::decode(&corrupt).is_err());

        let mut wrong_preheader = snapshot.framed_bytes().to_vec();
        wrong_preheader[0] ^= 1;
        assert!(NovaAccumulatorSnapshotV2::decode(&wrong_preheader).is_err());

        let checkpoint_id_offset = RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + 4 + 2 + 4 + 4 + 2 + 8 + 8;
        let mut incomplete = snapshot.framed_bytes().to_vec();
        incomplete[checkpoint_id_offset..checkpoint_id_offset + 32].fill(0);
        assert!(NovaAccumulatorSnapshotV2::decode(&incomplete).is_err());

        let mut invalid_predecessor = bindings(100);
        invalid_predecessor.predecessor = Some([0; 32]);
        assert!(matches!(
            NovaAccumulatorSnapshotV2::capture(
                invalid_predecessor,
                NovaRecoveryImageV2::new(vec![100; 96]),
            ),
            Err(CheckpointError::Authority)
        ));
    }

    #[test]
    fn test_snapshot_rotation() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("recovery");
        let store = NovaRecoveryStoreV2::open(&root).unwrap();
        store.commit(&snapshot(100)).unwrap();
        store.commit(&snapshot(200)).unwrap();
        let metrics = store.commit(&snapshot(300)).unwrap();
        assert_eq!(metrics.live_snapshot_count, 2);
        assert!(metrics.total_hot_bytes <= NOVA_HOT_CAP_BYTES_V2 as u64);
        drop(store);

        let reopened = NovaRecoveryStoreV2::open(&root).unwrap();
        assert_eq!(reopened.latest().unwrap().unwrap().0.height(), 300);
        assert_eq!(reopened.live_snapshots().unwrap().len(), 2);
        assert_eq!(reopened.quarantine.read_dir_bounded(2).unwrap().len(), 1);
    }

    #[test]
    fn test_mixed_context_is_quarantined_without_replacing_live_head() {
        let temp = tempfile::tempdir().unwrap();
        let store = NovaRecoveryStoreV2::open(temp.path().join("recovery")).unwrap();
        store.commit(&snapshot(100)).unwrap();

        let mut mixed_bindings = bindings(200);
        mixed_bindings.config_digest[0] ^= 1;
        let mixed = NovaAccumulatorSnapshotV2::capture(
            mixed_bindings,
            NovaRecoveryImageV2::new(vec![200; 96]),
        )
        .unwrap();
        assert!(matches!(
            store.commit(&mixed),
            Err(CheckpointError::RecursiveRejected(
                super::super::recursive_reject::RecursiveCheckpointRejectReasonV2::StepReordered
            ))
        ));
        assert_eq!(store.latest().unwrap().unwrap().0.height(), 100);
        assert_eq!(store.live_snapshots().unwrap().len(), 1);
        assert_eq!(store.quarantine.read_dir_bounded(1).unwrap().len(), 1);
    }

    #[test]
    fn test_corrupt_committed_snapshot_is_quarantined_on_reopen() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("recovery");
        let store = NovaRecoveryStoreV2::open(&root).unwrap();
        let committed = snapshot(100);
        store.commit(&committed).unwrap();
        let name = snapshot_name(&committed);
        store.snapshots.remove_file(&name).unwrap();
        write_once(&store.snapshots, &name, b"corrupt-snapshot").unwrap();
        drop(store);

        let reopened = NovaRecoveryStoreV2::open(&root).unwrap();
        assert!(reopened.latest().unwrap().is_none());
        assert_eq!(reopened.quarantine.read_dir_bounded(1).unwrap().len(), 1);
    }

    #[test]
    fn test_journal_limit_no_mutation() {
        let temp = tempfile::tempdir().unwrap();
        let store = NovaRecoveryStoreV2::open(temp.path().join("recovery")).unwrap();
        store.commit(&snapshot(100)).unwrap();
        store.commit(&snapshot(200)).unwrap();
        for index in 0..NOVA_JOURNAL_MAX_ENTRIES_V2 - 3 {
            store
                .append_journal(
                    NovaRecoveryJournalRecordV2::new(
                        NovaRecoveryJournalKindV2::ForkQuarantined,
                        10_000 + u64::try_from(index).unwrap(),
                        [u8::try_from(index + 1).unwrap(); 32],
                        [8; 32],
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let snapshots_before = store.snapshots.read_dir_bounded(3).unwrap();
        let quarantine_before = store.quarantine.read_dir_bounded(1).unwrap();

        assert!(matches!(
            store.commit(&snapshot(300)),
            Err(CheckpointError::Limit)
        ));
        assert_eq!(
            store.snapshots.read_dir_bounded(3).unwrap(),
            snapshots_before
        );
        assert_eq!(
            store.quarantine.read_dir_bounded(1).unwrap(),
            quarantine_before
        );
        assert_eq!(store.latest().unwrap().unwrap().0.height(), 200);
    }

    #[test]
    fn test_precommit_orphan() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("recovery");
        let store = NovaRecoveryStoreV2::open(&root).unwrap();
        let orphan = snapshot(100);
        write_once(
            &store.snapshots,
            &snapshot_name(&orphan),
            orphan.framed_bytes(),
        )
        .unwrap();
        drop(store);

        let reopened = NovaRecoveryStoreV2::open(&root).unwrap();
        assert!(reopened.latest().unwrap().is_none());
        assert_eq!(reopened.quarantine.read_dir_bounded(1).unwrap().len(), 1);
    }

    #[test]
    fn test_retirement_crash() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("recovery");
        let store = NovaRecoveryStoreV2::open(&root).unwrap();
        let first = snapshot(100);
        let second = snapshot(200);
        let orphan = snapshot(300);
        store.commit(&first).unwrap();
        store.commit(&second).unwrap();
        write_once(
            &store.snapshots,
            &snapshot_name(&orphan),
            orphan.framed_bytes(),
        )
        .unwrap();
        store
            .append_journal(
                NovaRecoveryJournalRecordV2::new(
                    NovaRecoveryJournalKindV2::SnapshotQuarantined,
                    first.height(),
                    first.digest(),
                    [0; 32],
                )
                .unwrap(),
            )
            .unwrap();
        drop(store);

        let reopened = NovaRecoveryStoreV2::open(&root).unwrap();
        assert_eq!(reopened.latest().unwrap().unwrap().0.height(), 200);
        assert_eq!(reopened.live_snapshots().unwrap().len(), 1);
        assert_eq!(reopened.quarantine.read_dir_bounded(2).unwrap().len(), 2);
    }

    #[test]
    fn test_stale_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let store = NovaRecoveryStoreV2::open(temp.path().join("recovery")).unwrap();
        store.commit(&snapshot(200)).unwrap();
        assert!(matches!(
            store.commit(&snapshot(100)),
            Err(CheckpointError::RecursiveRejected(
                super::super::recursive_reject::RecursiveCheckpointRejectReasonV2::StepReordered
            ))
        ));
        assert_eq!(store.latest().unwrap().unwrap().0.height(), 200);
    }
}
