use serde::{Deserialize, Serialize};
use z00z_crypto::{sha256_256_role, CheckpointShaRole};

use crate::{
    settlement::{
        derive_settlement_root_v2, BucketId, ClaimNullRec, FeeReplayRec, ObjectDeltaSetV1,
        RootGeneration, SettlementPath, SettlementStateRoot,
    },
    snapshot::PrepSnapshotId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StateMeta {
    pub(crate) version: u64,
    pub(crate) state_root: [u8; 32],
    pub(crate) flat_root: [u8; 32],
    pub(crate) snap_id: [u8; 32],
    pub(crate) draft_id: [u8; 32],
    pub(crate) check_id: [u8; 32],
    pub(crate) exec_id: [u8; 32],
    pub(crate) def_root: Option<[u8; 32]>,
    #[serde(default)]
    pub(crate) fee_replay_count: u64,
    #[serde(default)]
    pub(crate) fee_replay_digest: [u8; 32],
}

/// The durable, repository-local-only V2 root-generation transition record.
///
/// The record carries all authority and snapshot fields rather than trusting a
/// caller-provided digest as a substitute. The storage primitive writes it
/// under one immediate-durability transaction after comparing its storage
/// generation and definition root to the active head.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecursiveV2CutoverManifestV2 {
    pub(crate) schema_version: u8,
    pub(crate) authority_digest: [u8; 32],
    pub(crate) network_context: [u8; 32],
    pub(crate) config_digest: [u8; 32],
    pub(crate) policy_digest: [u8; 32],
    pub(crate) layout: u32,
    pub(crate) authority_generation: u64,
    pub(crate) noop_execution_input_version: u8,
    pub(crate) epoch_cadence_blocks: u64,
    pub(crate) snapshot_id: [u8; 32],
    pub(crate) snapshot_digest: [u8; 32],
    pub(crate) snapshot_storage_generation: u64,
    pub(crate) snapshot_root: [u8; 32],
    pub(crate) snapshot_record_count: u64,
    pub(crate) snapshot_byte_count: u64,
    pub(crate) snapshot_content_digest: [u8; 32],
    pub(crate) height: u64,
    pub(crate) opaque_last_root_record: [u8; 32],
    pub(crate) pinned_opaque_record_digest: [u8; 32],
    pub(crate) expected_definition_root: [u8; 32],
    pub(crate) expected_settlement_root: [u8; 32],
    pub(crate) storage_generation: u64,
    pub(crate) atomic_install_generation: u64,
    pub(crate) record_digest: [u8; 32],
}

impl RecursiveV2CutoverManifestV2 {
    pub(crate) const SCHEMA_VERSION: u8 = 2;

    pub(crate) fn validate(&self) -> Result<(), crate::backend::error::StoreBackendError> {
        if self.schema_version != Self::SCHEMA_VERSION
            || self.layout == 0
            || self.authority_generation == 0
            || self.noop_execution_input_version == 0
            || self.epoch_cadence_blocks == 0
            || self.height == 0
            || self.atomic_install_generation == 0
            || self.snapshot_storage_generation != self.storage_generation
            || self.authority_digest == [0; 32]
            || self.snapshot_digest == [0; 32]
            || self.pinned_opaque_record_digest == [0; 32]
            || self.record_digest == [0; 32]
        {
            return Err(crate::backend::error::StoreBackendError::Tx(
                "invalid recursive V2 cutover manifest".to_string(),
            ));
        }
        let authority_generation = self.authority_generation.to_le_bytes();
        let layout = self.layout.to_le_bytes();
        let noop_execution_input_version = [self.noop_execution_input_version];
        let epoch_cadence_blocks = self.epoch_cadence_blocks.to_le_bytes();
        let expected_authority = sha256_256_role(
            CheckpointShaRole::UniquenessContext,
            &[
                &self.network_context,
                &self.config_digest,
                &self.policy_digest,
                &layout,
                &authority_generation,
                &noop_execution_input_version,
                &epoch_cadence_blocks,
            ],
        );
        let storage_generation = self.storage_generation.to_le_bytes();
        let record_count = self.snapshot_record_count.to_le_bytes();
        let byte_count = self.snapshot_byte_count.to_le_bytes();
        let expected_snapshot = sha256_256_role(
            CheckpointShaRole::Content,
            &[
                &self.snapshot_id,
                &storage_generation,
                &self.snapshot_root,
                &self.expected_definition_root,
                &record_count,
                &byte_count,
                &self.snapshot_content_digest,
            ],
        );
        let expected_opaque = sha256_256_role(
            CheckpointShaRole::Link,
            &[
                b"z00z.recursive.v2.opaque-last-root-record",
                &self.opaque_last_root_record,
            ],
        );
        let expected_root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            self.layout,
            self.policy_digest,
            self.expected_definition_root,
        )
        .map_err(|_| {
            crate::backend::error::StoreBackendError::Tx(
                "recursive V2 cutover manifest has an invalid root binding".to_string(),
            )
        })?;
        let height = self.height.to_le_bytes();
        let install_generation = self.atomic_install_generation.to_le_bytes();
        let expected_record = sha256_256_role(
            CheckpointShaRole::Link,
            &[
                b"z00z.recursive.v2.cutover.manifest",
                &expected_authority,
                &expected_snapshot,
                &height,
                &self.opaque_last_root_record,
                &expected_opaque,
                &self.expected_definition_root,
                expected_root.as_bytes(),
                &install_generation,
            ],
        );
        if self.authority_digest != expected_authority
            || self.snapshot_digest != expected_snapshot
            || self.pinned_opaque_record_digest != expected_opaque
            || self.expected_settlement_root != *expected_root.as_bytes()
            || self.record_digest != expected_record
        {
            return Err(crate::backend::error::StoreBackendError::Tx(
                "recursive V2 cutover manifest binding mismatch".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod recursive_v2_cutover_tests {
    use super::RecursiveV2CutoverManifestV2;
    use crate::settlement::{derive_settlement_root_v2, RootGeneration};
    use z00z_crypto::{sha256_256_role, CheckpointShaRole};

    fn manifest() -> RecursiveV2CutoverManifestV2 {
        let network_context = [1; 32];
        let config_digest = [2; 32];
        let policy_digest = [3; 32];
        let layout = 7_u32;
        let authority_generation = 1_u64;
        let noop_execution_input_version = 2_u8;
        let epoch_cadence_blocks = 1_000_u64;
        let authority_digest = sha256_256_role(
            CheckpointShaRole::UniquenessContext,
            &[
                &network_context,
                &config_digest,
                &policy_digest,
                &layout.to_le_bytes(),
                &authority_generation.to_le_bytes(),
                &[noop_execution_input_version],
                &epoch_cadence_blocks.to_le_bytes(),
            ],
        );
        let snapshot_id = [4; 32];
        let storage_generation = 9_u64;
        let snapshot_root = [5; 32];
        let snapshot_record_count = 2_u64;
        let snapshot_byte_count = 96_u64;
        let snapshot_content_digest = [6; 32];
        let expected_definition_root = [8; 32];
        let snapshot_digest = sha256_256_role(
            CheckpointShaRole::Content,
            &[
                &snapshot_id,
                &storage_generation.to_le_bytes(),
                &snapshot_root,
                &expected_definition_root,
                &snapshot_record_count.to_le_bytes(),
                &snapshot_byte_count.to_le_bytes(),
                &snapshot_content_digest,
            ],
        );
        let opaque_last_root_record = [7; 32];
        let pinned_opaque_record_digest = sha256_256_role(
            CheckpointShaRole::Link,
            &[
                b"z00z.recursive.v2.opaque-last-root-record",
                &opaque_last_root_record,
            ],
        );
        let expected_settlement_root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            layout,
            policy_digest,
            expected_definition_root,
        )
        .expect("fixed manifest root")
        .into_bytes();
        let height = 10_u64;
        let atomic_install_generation = 11_u64;
        let record_digest = sha256_256_role(
            CheckpointShaRole::Link,
            &[
                b"z00z.recursive.v2.cutover.manifest",
                &authority_digest,
                &snapshot_digest,
                &height.to_le_bytes(),
                &opaque_last_root_record,
                &pinned_opaque_record_digest,
                &expected_definition_root,
                &expected_settlement_root,
                &atomic_install_generation.to_le_bytes(),
            ],
        );
        RecursiveV2CutoverManifestV2 {
            schema_version: RecursiveV2CutoverManifestV2::SCHEMA_VERSION,
            authority_digest,
            network_context,
            config_digest,
            policy_digest,
            layout,
            authority_generation,
            noop_execution_input_version,
            epoch_cadence_blocks,
            snapshot_id,
            snapshot_digest,
            snapshot_storage_generation: storage_generation,
            snapshot_root,
            snapshot_record_count,
            snapshot_byte_count,
            snapshot_content_digest,
            height,
            opaque_last_root_record,
            pinned_opaque_record_digest,
            expected_definition_root,
            expected_settlement_root,
            storage_generation,
            atomic_install_generation,
            record_digest,
        }
    }

    #[test]
    fn manifest_rejects_mutations() {
        let valid = manifest();
        assert!(valid.validate().is_ok());

        let mutations: [fn(&mut RecursiveV2CutoverManifestV2); 6] = [
            |manifest: &mut RecursiveV2CutoverManifestV2| manifest.authority_digest[0] ^= 1,
            |manifest: &mut RecursiveV2CutoverManifestV2| manifest.epoch_cadence_blocks += 1,
            |manifest: &mut RecursiveV2CutoverManifestV2| manifest.snapshot_digest[0] ^= 1,
            |manifest: &mut RecursiveV2CutoverManifestV2| {
                manifest.pinned_opaque_record_digest[0] ^= 1
            },
            |manifest: &mut RecursiveV2CutoverManifestV2| manifest.expected_settlement_root[0] ^= 1,
            |manifest: &mut RecursiveV2CutoverManifestV2| manifest.record_digest[0] ^= 1,
        ];
        for mutate in mutations {
            let mut mutated = valid.clone();
            mutate(&mut mutated);
            assert!(mutated.validate().is_err());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonExec {
    pub(crate) exec_id: crate::checkpoint::CheckpointExecInputId,
    pub(crate) exec_bytes: Vec<u8>,
}

impl CanonExec {
    pub(crate) fn new(
        exec_id: crate::checkpoint::CheckpointExecInputId,
        exec_bytes: Vec<u8>,
    ) -> Self {
        Self {
            exec_id,
            exec_bytes,
        }
    }
}

pub(crate) struct WriteArts {
    pub(crate) version: u64,
    pub(crate) snap_id: PrepSnapshotId,
    pub(crate) snap_bytes: Vec<u8>,
    pub(crate) canon_exec: Option<CanonExec>,
    pub(crate) spent: Vec<crate::checkpoint::SpentEnt>,
    pub(crate) created: Vec<crate::checkpoint::CreatedEnt>,
}

impl WriteArts {
    pub(crate) fn new(
        version: u64,
        snap_id: PrepSnapshotId,
        snap_bytes: Vec<u8>,
        canon_exec: Option<CanonExec>,
        spent: Vec<crate::checkpoint::SpentEnt>,
        created: Vec<crate::checkpoint::CreatedEnt>,
    ) -> Self {
        Self {
            version,
            snap_id,
            snap_bytes,
            canon_exec,
            spent,
            created,
        }
    }
}

#[derive(Clone)]
pub(crate) struct LoadState {
    pub(crate) version: u64,
    pub(crate) state_root: SettlementStateRoot,
    pub(crate) flat_root: [u8; 32],
    pub(crate) hjmt_terminal_rows: Vec<(SettlementPath, BucketId, Vec<u8>)>,
    pub(crate) hjmt_settlement_path_rows: Vec<(SettlementPath, Vec<u8>)>,
    pub(crate) claim_null_rows: Vec<ClaimNullRec>,
    pub(crate) fee_replay_rows: Vec<FeeReplayRec>,
    pub(crate) object_delta: Option<ObjectDeltaSetV1>,
    pub(crate) hjmt_journal: Option<crate::settlement::hjmt_journal::HjmtCommitJournalEntry>,
}
