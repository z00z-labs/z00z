use std::path::{Path, PathBuf};

use serde::Serialize;
use z00z_crypto::sha256_256;
use z00z_storage::{
    checkpoint::{
        recursive_v2::{
            composed_history_error_exponent_v2, epoch_ordered_digest_root_v2, EpochCadenceClassV2,
            EpochFrontierAuthorityV2, EpochManifestInputsV2, EpochManifestV2, EpochProofFrontierV2,
            EpochProofWorkManifestV2, EpochTraceChunkWorkV2, EpochTransitionStreamV2,
            HistoryAccumulatorInputsV2, HistoryAccumulatorStatementV2, HistoryBranchV2,
            Plonky3EpochAdapterV2, Plonky3EpochChunkWorkerV2, Plonky3EpochProofV2,
            Plonky3HistoryAdapterV2, Plonky3HistoryAuthorityResolverV2, Plonky3HistoryProofV2,
            RecursiveSecurityBudgetManifestV2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2,
        },
        CheckpointConfigResolverV3,
    },
    settlement::{SettlementStateRoot, SettlementStore},
};
use z00z_utils::io::{atomic_write_file_private, read_file_bounded, save_json};

use super::{
    checkpoint::SealedCheckpoint, config::Scenario2Cfg, da::DaCommit, runner::Scenario2Err,
};

const CHECKPOINT_ID_ROOT_DOMAIN: &str = "z00z.simulator.scenario-2.plonky3.checkpoint-id-root.v1";
const DA_ROOT_DOMAIN: &str = "z00z.simulator.scenario-2.plonky3.da-root.v1";
const ARCHIVE_ROOT_DOMAIN: &str = "z00z.simulator.scenario-2.plonky3.archive-availability-root.v1";
const CLOSE_DOMAIN: &str = "z00z.simulator.scenario-2.plonky3.epoch-close.v1";
const TRUST_DOMAIN: &str = "z00z.simulator.scenario-2.plonky3.trust.v1";

#[derive(Clone, Debug, Serialize)]
pub(super) struct Plonky3EpochOutcome {
    pub epoch_index: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub transition_count: u32,
    pub trace_chunk_count: u32,
    pub merged_parent_count: usize,
    pub epoch_statement_digest_hex: String,
    pub epoch_proof_digest_hex: String,
    pub history_statement_digest_hex: String,
    pub history_proof_digest_hex: String,
    pub epoch_manifest_digest_hex: String,
    pub work_manifest_bytes: u64,
    pub epoch_proof_bytes: u64,
    pub history_proof_bytes: u64,
    pub epoch_manifest_bytes: u64,
}

pub(super) struct Plonky3Pipeline {
    root: PathBuf,
    cadence_blocks: u64,
    max_work_manifest_bytes: u64,
    max_proof_artifact_bytes: u64,
    genesis_trust_anchor_digest: [u8; 32],
    genesis_state_root: [u8; 32],
    genesis_epoch_anchor_root: [u8; 32],
    predecessor: Option<Plonky3HistoryProofV2>,
    completed_epochs: u32,
}

pub(super) struct Plonky3Cycle {
    cycle: u32,
    epoch_index: u64,
    cadence_blocks: u64,
    authority: EpochFrontierAuthorityV2,
    stream: EpochTransitionStreamV2,
    frontier: EpochProofFrontierV2,
    frontier_path: PathBuf,
    checkpoint_ids: Vec<[u8; 32]>,
    da_commitments: Vec<[u8; 32]>,
    archive_manifest_roots: Vec<[u8; 32]>,
    verified_chunks: u32,
    merged_parents: usize,
}

pub(super) struct Plonky3ChunkObservation {
    pub ordinal: u32,
    pub binding_count: u64,
    pub proof_bytes: u64,
    pub trace_rows: u64,
    pub table_count: u64,
}

pub(super) struct Plonky3MergeObservation {
    pub merged: usize,
    pub verified_chunks: u32,
    pub active_ranges: u32,
}

pub(super) struct ClosedPlonky3Epoch {
    epoch_dir: PathBuf,
    authority: EpochFrontierAuthorityV2,
    frontier: EpochProofFrontierV2,
    work_manifest: EpochProofWorkManifestV2,
    archive_availability_manifest_root: [u8; 32],
    checkpoint_id_root: [u8; 32],
    da_commitment_root: [u8; 32],
    merged_parents: usize,
}

pub(super) struct SealedPlonky3Epoch {
    epoch_dir: PathBuf,
    authority: EpochFrontierAuthorityV2,
    work_manifest: EpochProofWorkManifestV2,
    epoch_proof: Plonky3EpochProofV2,
    archive_availability_manifest_root: [u8; 32],
    checkpoint_id_root: [u8; 32],
    da_commitment_root: [u8; 32],
    merged_parents: usize,
}

#[derive(Serialize)]
struct AvailabilityEvidence {
    epoch_index: u64,
    start_height: u64,
    end_height: u64,
    block_count: u64,
    checkpoint_id_root_hex: String,
    da_commitment_root_hex: String,
    archive_availability_manifest_root_hex: String,
    nova_chain_root_hex: String,
    epoch_close_anchor_digest_hex: String,
}

impl Plonky3Pipeline {
    pub fn new(
        run_dir: &Path,
        config: &Scenario2Cfg,
        genesis_state_root: SettlementStateRoot,
    ) -> Result<Self, Scenario2Err> {
        let active = CheckpointConfigResolverV3::resolve_active().map_err(plonky3_error)?;
        let authority_cadence = active.config().branches.plonky3_epoch.cadence_blocks;
        if authority_cadence != config.plonky3.cadence_blocks
            || authority_cadence != u64::from(config.load.blocks_per_cycle)
            || config.plonky3.max_inflight_chunk_proofs != 1
        {
            return Err(Scenario2Err::Config(
                "scenario Plonky3 cadence or in-flight proof bound differs from authority"
                    .to_string(),
            ));
        }
        let chain_id = config.runtime.chain_id.to_le_bytes();
        let genesis_trust_anchor_digest = sha256_256(
            TRUST_DOMAIN,
            "genesis-trust-anchor",
            &[
                &chain_id,
                config.runtime.chain_type.as_bytes(),
                config.runtime.chain_name.as_bytes(),
                genesis_state_root.as_bytes(),
            ],
        );
        let genesis_epoch_anchor_root = sha256_256(
            TRUST_DOMAIN,
            "genesis-epoch-anchor",
            &[&genesis_trust_anchor_digest, genesis_state_root.as_bytes()],
        );
        Ok(Self {
            root: run_dir.join("plonky3"),
            cadence_blocks: authority_cadence,
            max_work_manifest_bytes: config.plonky3.max_work_manifest_bytes,
            max_proof_artifact_bytes: config.plonky3.max_proof_artifact_bytes,
            genesis_trust_anchor_digest,
            genesis_state_root: *genesis_state_root.as_bytes(),
            genesis_epoch_anchor_root,
            predecessor: None,
            completed_epochs: 0,
        })
    }

    pub fn begin_cycle(
        &self,
        cycle: u32,
        store: &SettlementStore,
    ) -> Result<Plonky3Cycle, Scenario2Err> {
        let epoch_index = u64::from(cycle)
            .checked_sub(1)
            .ok_or_else(|| Scenario2Err::Invariant("Plonky3 cycle starts at one".to_string()))?;
        if u64::from(self.completed_epochs) != epoch_index {
            return Err(Scenario2Err::Invariant(
                "Plonky3 epoch lineage is not sequential".to_string(),
            ));
        }
        let stream = EpochTransitionStreamV2::resolve_active(
            store,
            EpochCadenceClassV2::Production,
            epoch_index,
            self.cadence_blocks,
        )
        .map_err(plonky3_error)?;
        let authority = stream.authority();
        let expected_chunks = u32::try_from(
            self.cadence_blocks
                .checked_div(u64::from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2))
                .ok_or_else(|| {
                    Scenario2Err::Invariant("Plonky3 chunk division failed".to_string())
                })?,
        )
        .map_err(|_| Scenario2Err::Invariant("Plonky3 chunk count overflow".to_string()))?;
        if !self
            .cadence_blocks
            .is_multiple_of(u64::from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2))
            || authority.cadence_blocks() != self.cadence_blocks
            || authority.chunk_count() != expected_chunks
        {
            return Err(Scenario2Err::Invariant(
                "Plonky3 production epoch authority has unexpected dimensions".to_string(),
            ));
        }
        let epoch_dir = self.root.join(format!("epoch-{epoch_index:04}"));
        let frontier_path = epoch_dir.join("frontier");
        let frontier =
            EpochProofFrontierV2::open(&frontier_path, authority).map_err(plonky3_error)?;
        if frontier
            .progress()
            .map_err(plonky3_error)?
            .verified_chunk_count()
            != 0
        {
            return Err(Scenario2Err::Invariant(
                "new scenario run opened a non-empty Plonky3 frontier".to_string(),
            ));
        }
        let capacity = usize::try_from(self.cadence_blocks)
            .map_err(|_| Scenario2Err::Invariant("Plonky3 cadence overflow".to_string()))?;
        Ok(Plonky3Cycle {
            cycle,
            epoch_index,
            cadence_blocks: self.cadence_blocks,
            authority,
            stream,
            frontier,
            frontier_path,
            checkpoint_ids: Vec::with_capacity(capacity),
            da_commitments: Vec::with_capacity(capacity),
            archive_manifest_roots: Vec::with_capacity(capacity),
            verified_chunks: 0,
            merged_parents: 0,
        })
    }

    pub fn prove_history(
        &self,
        epoch: &SealedPlonky3Epoch,
    ) -> Result<Plonky3HistoryProofV2, Scenario2Err> {
        let statement = self.history_statement(&epoch.epoch_proof)?;
        let proof = match self.predecessor.as_ref() {
            None => Plonky3HistoryAdapterV2::prove_base(statement, &epoch.epoch_proof),
            Some(predecessor) => {
                Plonky3HistoryAdapterV2::prove_successor(statement, predecessor, &epoch.epoch_proof)
            }
        }
        .map_err(plonky3_error)?;
        Plonky3HistoryAdapterV2::verify(&proof).map_err(plonky3_error)?;
        Ok(proof)
    }

    pub fn publish_and_reload(
        &mut self,
        epoch: SealedPlonky3Epoch,
        history: Plonky3HistoryProofV2,
    ) -> Result<Plonky3EpochOutcome, Scenario2Err> {
        let epoch_inputs = epoch.epoch_proof.statement().inputs();
        let publishable = EpochManifestV2::new(
            EpochManifestInputsV2 {
                checkpoint_artifact_root: epoch_inputs.checkpoint_artifact_root,
                archive_availability_manifest_root: epoch.archive_availability_manifest_root,
            },
            epoch.epoch_proof.statement().clone(),
            history.clone(),
        )
        .map_err(plonky3_error)?;

        enforce_len(
            epoch.work_manifest.canonical_bytes(),
            self.max_work_manifest_bytes,
            "Plonky3 work manifest",
        )?;
        for (label, bytes) in [
            ("Plonky3 epoch proof", epoch.epoch_proof.canonical_bytes()),
            ("Plonky3 history proof", history.canonical_bytes()),
            ("Plonky3 epoch manifest", publishable.canonical_bytes()),
        ] {
            enforce_len(bytes, self.max_proof_artifact_bytes, label)?;
        }

        write_private(
            epoch.epoch_dir.join("work-manifest.bin"),
            epoch.work_manifest.canonical_bytes(),
        )?;
        write_private(
            epoch.epoch_dir.join("epoch-proof.bin"),
            epoch.epoch_proof.canonical_bytes(),
        )?;
        write_private(
            epoch.epoch_dir.join("history-proof.bin"),
            history.canonical_bytes(),
        )?;
        write_private(
            epoch.epoch_dir.join("epoch-manifest.bin"),
            publishable.canonical_bytes(),
        )?;
        save_json(
            epoch.epoch_dir.join("availability-evidence.json"),
            &AvailabilityEvidence {
                epoch_index: epoch.authority.epoch_index(),
                start_height: epoch.authority.start_height(),
                end_height: epoch.authority.end_height(),
                block_count: epoch.authority.cadence_blocks(),
                checkpoint_id_root_hex: hex::encode(epoch.checkpoint_id_root),
                da_commitment_root_hex: hex::encode(epoch.da_commitment_root),
                archive_availability_manifest_root_hex: hex::encode(
                    epoch.archive_availability_manifest_root,
                ),
                nova_chain_root_hex: hex::encode(
                    epoch.work_manifest.nova_chain_root().ok_or_else(|| {
                        Scenario2Err::Invariant(
                            "Plonky3 work manifest omitted Nova root".to_string(),
                        )
                    })?,
                ),
                epoch_close_anchor_digest_hex: hex::encode(
                    epoch.work_manifest.epoch_close_anchor_digest(),
                ),
            },
        )?;

        let reloaded_work = EpochProofWorkManifestV2::decode_canonical(&read_file_bounded(
            epoch.epoch_dir.join("work-manifest.bin"),
            self.max_work_manifest_bytes,
        )?)
        .map_err(plonky3_error)?;
        let reloaded_epoch = Plonky3EpochProofV2::decode_local(&read_file_bounded(
            epoch.epoch_dir.join("epoch-proof.bin"),
            self.max_proof_artifact_bytes,
        )?)
        .map_err(plonky3_error)?;
        Plonky3EpochAdapterV2::verify(&reloaded_epoch).map_err(plonky3_error)?;
        let reloaded_history = Plonky3HistoryProofV2::decode_local(&read_file_bounded(
            epoch.epoch_dir.join("history-proof.bin"),
            self.max_proof_artifact_bytes,
        )?)
        .map_err(plonky3_error)?;
        Plonky3HistoryAdapterV2::verify(&reloaded_history).map_err(plonky3_error)?;
        let reloaded_publishable = EpochManifestV2::decode_canonical(&read_file_bounded(
            epoch.epoch_dir.join("epoch-manifest.bin"),
            self.max_proof_artifact_bytes,
        )?)
        .map_err(plonky3_error)?;
        if reloaded_work != epoch.work_manifest
            || reloaded_epoch != epoch.epoch_proof
            || reloaded_history != history
            || reloaded_publishable != publishable
            || reloaded_publishable.epoch_statement_digest() != reloaded_epoch.statement().digest()
            || reloaded_publishable.plonky3_history_proof_digest()
                != reloaded_history.proof_digest()
        {
            return Err(Scenario2Err::Invariant(
                "reloaded Plonky3 epoch artifacts changed identity".to_string(),
            ));
        }

        self.predecessor = Some(reloaded_publishable.history_proof().clone());
        self.completed_epochs = self
            .completed_epochs
            .checked_add(1)
            .ok_or_else(|| Scenario2Err::Invariant("Plonky3 epoch count overflow".to_string()))?;
        Ok(Plonky3EpochOutcome {
            epoch_index: epoch.authority.epoch_index(),
            start_height: epoch.authority.start_height(),
            end_height: epoch.authority.end_height(),
            transition_count: epoch.authority.transition_count(),
            trace_chunk_count: epoch.authority.chunk_count(),
            merged_parent_count: epoch.merged_parents,
            epoch_statement_digest_hex: hex::encode(reloaded_epoch.statement().digest()),
            epoch_proof_digest_hex: hex::encode(reloaded_epoch.proof_digest()),
            history_statement_digest_hex: hex::encode(reloaded_history.statement().digest()),
            history_proof_digest_hex: hex::encode(reloaded_history.proof_digest()),
            epoch_manifest_digest_hex: hex::encode(reloaded_publishable.digest()),
            work_manifest_bytes: bytes_len(reloaded_work.canonical_bytes())?,
            epoch_proof_bytes: bytes_len(reloaded_epoch.canonical_bytes())?,
            history_proof_bytes: bytes_len(reloaded_history.canonical_bytes())?,
            epoch_manifest_bytes: bytes_len(reloaded_publishable.canonical_bytes())?,
        })
    }

    #[must_use]
    pub const fn completed_epochs(&self) -> u32 {
        self.completed_epochs
    }

    fn history_statement(
        &self,
        epoch: &Plonky3EpochProofV2,
    ) -> Result<HistoryAccumulatorStatementV2, Scenario2Err> {
        let active = CheckpointConfigResolverV3::resolve_active().map_err(plonky3_error)?;
        let config_identity = active.identity();
        let authority =
            Plonky3HistoryAuthorityResolverV2::resolve_active().map_err(plonky3_error)?;
        let identity = authority.identity();
        let security =
            RecursiveSecurityBudgetManifestV2::authority_pinned().map_err(plonky3_error)?;
        let inherited = security.inherited_error_exponent().ok_or_else(|| {
            Scenario2Err::Plonky3(
                "active Plonky3 security budget has no inherited error exponent".to_string(),
            )
        })?;
        let epoch_inputs = epoch.statement().inputs();
        let epoch_anchor_mmr_root = Plonky3HistoryAdapterV2::derive_epoch_anchor_mmr_root(
            self.predecessor.as_ref(),
            epoch,
            None,
        )
        .map_err(plonky3_error)?;
        let (
            branch,
            first_epoch,
            first_height,
            history_length,
            accepted_epoch_count,
            genesis_trust_anchor_digest,
            genesis_state_root,
            previous_terminal_state_root,
            previous_epoch_anchor_root,
            predecessor_statement_digest,
        ) = match self.predecessor.as_ref() {
            None => (
                HistoryBranchV2::Base,
                epoch_inputs.epoch_index,
                epoch_inputs.start_height,
                1,
                1,
                self.genesis_trust_anchor_digest,
                self.genesis_state_root,
                self.genesis_state_root,
                self.genesis_epoch_anchor_root,
                None,
            ),
            Some(predecessor) => {
                let previous = predecessor.statement().inputs();
                (
                    HistoryBranchV2::Successor,
                    previous.first_epoch,
                    previous.first_height,
                    previous.history_length.checked_add(1).ok_or_else(|| {
                        Scenario2Err::Invariant("Plonky3 history length overflow".to_string())
                    })?,
                    previous
                        .accepted_epoch_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            Scenario2Err::Invariant(
                                "Plonky3 accepted epoch count overflow".to_string(),
                            )
                        })?,
                    previous.genesis_trust_anchor_digest,
                    previous.genesis_state_root,
                    previous.current_terminal_state_root,
                    previous.current_epoch_anchor_root,
                    Some(predecessor.statement().digest()),
                )
            }
        };
        let cumulative_error_exponent = composed_history_error_exponent_v2(
            security.per_proof_error_exponent(),
            accepted_epoch_count,
            inherited,
        )
        .map_err(plonky3_error)?;
        HistoryAccumulatorStatementV2::new(HistoryAccumulatorInputsV2 {
            branch,
            first_epoch,
            last_epoch: epoch_inputs.epoch_index,
            first_height,
            last_height: epoch_inputs.end_height,
            cadence_blocks: epoch_inputs.cadence_blocks,
            history_length,
            accepted_epoch_count,
            config_generation: config_identity.config_generation,
            authority_generation: config_identity.authority_generation,
            activation_height: config_identity.activation_height,
            rollback_floor: config_identity.rollback_floor,
            parameter_generation: identity.parameter_generation,
            runtime_profile_generation: config_identity.runtime_profile_generation,
            composition_rule_generation: security.composition_rule_generation(),
            per_proof_error_exponent: security.per_proof_error_exponent(),
            inherited_error_exponent: inherited,
            cumulative_error_exponent,
            minimum_residual_bits: security.minimum_residual_bits(),
            chain_context_digest: epoch_inputs.chain_context_digest,
            genesis_trust_anchor_digest,
            genesis_state_root,
            previous_terminal_state_root,
            current_terminal_state_root: epoch_inputs.end_root,
            previous_epoch_anchor_root,
            current_epoch_anchor_root: epoch_inputs.epoch_close_anchor_digest,
            exact_epoch_statement_digest: epoch.statement().digest(),
            predicate_digest: epoch_inputs.predicate_digest,
            verifier_parameter_digest: identity.verifier_parameter_digest,
            security_budget_digest: identity.security_budget_digest,
            config_digest: config_identity.config_digest,
            registry_digest: config_identity.registry_digest,
            runtime_profile_manifest_digest: config_identity.runtime_profile_manifest_digest,
            authority_bundle_digest: config_identity.history_authority_bundle_digest,
            verifier_bundle_digest: identity.verifier_bundle_digest,
            epoch_anchor_mmr_root,
            predecessor_statement_digest,
        })
        .map_err(plonky3_error)
    }
}

impl Plonky3Cycle {
    pub fn stream(&self) -> &EpochTransitionStreamV2 {
        &self.stream
    }

    pub fn stream_mut(&mut self) -> &mut EpochTransitionStreamV2 {
        &mut self.stream
    }

    pub fn record_block(
        &mut self,
        height: u64,
        checkpoint: &SealedCheckpoint,
        da: &DaCommit,
    ) -> Result<(), Scenario2Err> {
        let ordinal = u64::try_from(self.checkpoint_ids.len())
            .map_err(|_| Scenario2Err::Invariant("Plonky3 block ordinal overflow".to_string()))?;
        let expected_height = self
            .authority
            .start_height()
            .checked_add(ordinal)
            .ok_or_else(|| Scenario2Err::Invariant("Plonky3 height overflow".to_string()))?;
        if height != expected_height || ordinal >= self.cadence_blocks {
            return Err(Scenario2Err::Invariant(
                "Plonky3 availability evidence is out of epoch order".to_string(),
            ));
        }
        self.checkpoint_ids
            .push(*checkpoint.checkpoint_id.as_bytes());
        self.da_commitments.push(da.payload_commitment);
        self.archive_manifest_roots
            .push(checkpoint.archive_manifest_root);
        Ok(())
    }

    pub fn prove_and_admit(
        &mut self,
        work: EpochTraceChunkWorkV2,
    ) -> Result<Plonky3ChunkObservation, Scenario2Err> {
        let expected_ordinal = self.verified_chunks;
        if work.chunk_ordinal() != expected_ordinal {
            return Err(Scenario2Err::Invariant(
                "Plonky3 trace chunk arrived out of order".to_string(),
            ));
        }
        let proof = Plonky3EpochChunkWorkerV2::prove_chunk(work).map_err(plonky3_error)?;
        proof.verify().map_err(plonky3_error)?;
        let inputs = proof.transition_statement().inputs();
        if inputs.chunk_ordinal != expected_ordinal
            || inputs.chunk_count != self.authority.chunk_count()
        {
            return Err(Scenario2Err::Invariant(
                "Plonky3 proof statement has unexpected chunk authority".to_string(),
            ));
        }
        let observation = Plonky3ChunkObservation {
            ordinal: inputs.chunk_ordinal,
            binding_count: u64::try_from(proof.binding_count()).map_err(|_| {
                Scenario2Err::Invariant("Plonky3 binding count overflow".to_string())
            })?,
            proof_bytes: u64::try_from(proof.canonical_bytes().len()).map_err(|_| {
                Scenario2Err::Invariant("Plonky3 proof byte count overflow".to_string())
            })?,
            trace_rows: u64::try_from(proof.trace_row_count().map_err(plonky3_error)?).map_err(
                |_| Scenario2Err::Invariant("Plonky3 trace row count overflow".to_string()),
            )?,
            table_count: u64::try_from(proof.table_count())
                .map_err(|_| Scenario2Err::Invariant("Plonky3 table count overflow".to_string()))?,
        };
        self.frontier
            .admit_verified_chunk(&proof)
            .map_err(plonky3_error)?;
        self.verified_chunks = self
            .verified_chunks
            .checked_add(1)
            .ok_or_else(|| Scenario2Err::Invariant("Plonky3 chunk count overflow".to_string()))?;
        Ok(observation)
    }

    pub fn merge_ready(&mut self) -> Result<Plonky3MergeObservation, Scenario2Err> {
        let merged = Plonky3EpochAdapterV2::merge_ready(&self.frontier).map_err(plonky3_error)?;
        self.merged_parents = self
            .merged_parents
            .checked_add(merged)
            .ok_or_else(|| Scenario2Err::Invariant("Plonky3 merge count overflow".to_string()))?;
        let progress = self.frontier.progress().map_err(plonky3_error)?;
        Ok(Plonky3MergeObservation {
            merged,
            verified_chunks: progress.verified_chunk_count(),
            active_ranges: progress.active_range_count(),
        })
    }

    pub fn close(self, nova_chain_root: [u8; 32]) -> Result<ClosedPlonky3Epoch, Scenario2Err> {
        let expected_blocks = usize::try_from(self.cadence_blocks)
            .map_err(|_| Scenario2Err::Invariant("Plonky3 cadence overflow".to_string()))?;
        let progress = self.frontier.progress().map_err(plonky3_error)?;
        if self.stream.transition_count() != expected_blocks
            || self.stream.emitted_chunk_count() != self.stream.total_chunk_count()
            || self.checkpoint_ids.len() != expected_blocks
            || self.da_commitments.len() != expected_blocks
            || self.archive_manifest_roots.len() != expected_blocks
            || !progress.all_chunks_verified()
            || progress.verified_chunk_count() != self.authority.chunk_count()
            || self.verified_chunks != self.authority.chunk_count()
        {
            return Err(Scenario2Err::Invariant(
                "Plonky3 epoch closed before all transitions and chunks completed".to_string(),
            ));
        }
        let checkpoint_id_root =
            epoch_ordered_digest_root_v2(CHECKPOINT_ID_ROOT_DOMAIN, &self.checkpoint_ids)
                .map_err(plonky3_error)?;
        let da_commitment_root = epoch_ordered_digest_root_v2(DA_ROOT_DOMAIN, &self.da_commitments)
            .map_err(plonky3_error)?;
        let archive_availability_manifest_root =
            epoch_ordered_digest_root_v2(ARCHIVE_ROOT_DOMAIN, &self.archive_manifest_roots)
                .map_err(plonky3_error)?;
        let cycle = self.cycle.to_le_bytes();
        let epoch_index = self.epoch_index.to_le_bytes();
        let cadence = self.cadence_blocks.to_le_bytes();
        let epoch_close_anchor_digest = sha256_256(
            CLOSE_DOMAIN,
            "close-anchor",
            &[
                &cycle,
                &epoch_index,
                &cadence,
                &checkpoint_id_root,
                &da_commitment_root,
                &archive_availability_manifest_root,
                &nova_chain_root,
            ],
        );
        let work_manifest = self
            .stream
            .close(epoch_close_anchor_digest, Some(nova_chain_root))
            .map_err(plonky3_error)?;
        let authority = self.authority;
        let frontier_path = self.frontier_path;
        let merged_parents = self.merged_parents;
        drop(self.frontier);
        let frontier =
            EpochProofFrontierV2::open(&frontier_path, authority).map_err(plonky3_error)?;
        let final_merges = Plonky3EpochAdapterV2::merge_ready(&frontier).map_err(plonky3_error)?;
        let merged_parents = merged_parents
            .checked_add(final_merges)
            .ok_or_else(|| Scenario2Err::Invariant("Plonky3 merge count overflow".to_string()))?;
        frontier
            .validate_closed_manifest(&work_manifest)
            .map_err(plonky3_error)?;
        let reopened = frontier.progress().map_err(plonky3_error)?;
        if !reopened.all_chunks_verified()
            || reopened.verified_chunk_count() != authority.chunk_count()
        {
            return Err(Scenario2Err::Invariant(
                "reopened Plonky3 frontier lost verified chunks".to_string(),
            ));
        }
        let epoch_dir = frontier_path.parent().ok_or_else(|| {
            Scenario2Err::Invariant("Plonky3 frontier has no epoch directory".to_string())
        })?;
        Ok(ClosedPlonky3Epoch {
            epoch_dir: epoch_dir.to_path_buf(),
            authority,
            frontier,
            work_manifest,
            archive_availability_manifest_root,
            checkpoint_id_root,
            da_commitment_root,
            merged_parents,
        })
    }
}

impl ClosedPlonky3Epoch {
    pub fn seal_and_verify(self) -> Result<SealedPlonky3Epoch, Scenario2Err> {
        let epoch_proof = Plonky3EpochAdapterV2::seal(&self.frontier, &self.work_manifest)
            .map_err(plonky3_error)?;
        Plonky3EpochAdapterV2::verify(&epoch_proof).map_err(plonky3_error)?;
        if epoch_proof.is_nova_only()
            || epoch_proof.statement().transition_count() != self.authority.transition_count()
            || epoch_proof.statement().epoch_work_manifest_digest() != self.work_manifest.digest()
        {
            return Err(Scenario2Err::Invariant(
                "sealed Plonky3 epoch proof does not bind the exact work manifest".to_string(),
            ));
        }
        Ok(SealedPlonky3Epoch {
            epoch_dir: self.epoch_dir,
            authority: self.authority,
            work_manifest: self.work_manifest,
            epoch_proof,
            archive_availability_manifest_root: self.archive_availability_manifest_root,
            checkpoint_id_root: self.checkpoint_id_root,
            da_commitment_root: self.da_commitment_root,
            merged_parents: self.merged_parents,
        })
    }

    #[must_use]
    pub fn logical_bytes(&self) -> u64 {
        u64::try_from(self.work_manifest.canonical_bytes().len()).unwrap_or(u64::MAX)
    }
}

impl SealedPlonky3Epoch {
    #[must_use]
    pub fn epoch_proof_bytes(&self) -> u64 {
        u64::try_from(self.epoch_proof.canonical_bytes().len()).unwrap_or(u64::MAX)
    }
}

fn enforce_len(bytes: &[u8], cap: u64, label: &str) -> Result<(), Scenario2Err> {
    let len = bytes_len(bytes)?;
    if len == 0 || len > cap {
        return Err(Scenario2Err::Plonky3(format!(
            "{label} size {len} exceeds configured cap {cap}"
        )));
    }
    Ok(())
}

fn bytes_len(bytes: &[u8]) -> Result<u64, Scenario2Err> {
    u64::try_from(bytes.len())
        .map_err(|_| Scenario2Err::Invariant("Plonky3 byte count overflow".to_string()))
}

fn write_private(path: PathBuf, bytes: &[u8]) -> Result<(), Scenario2Err> {
    atomic_write_file_private(path, bytes).map_err(Scenario2Err::Io)
}

fn plonky3_error(error: z00z_storage::CheckpointError) -> Scenario2Err {
    Scenario2Err::Plonky3(error.to_string())
}
