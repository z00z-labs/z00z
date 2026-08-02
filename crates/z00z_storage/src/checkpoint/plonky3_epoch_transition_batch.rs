//! Proof-bound transition and typed-event tables for one bounded epoch chunk.
//!
//! This artifact is the first canonical multi-table Batch-STARK in the streamed
//! epoch path. The transition table supplies the expected typed commitments,
//! while independently materialized typed-event rows supply and consume the
//! matching LogUp multisets. Private event bytes remain non-public while their
//! exact canonical source is SHA-bound inside each closed proof group.

use p3_circuit::tables::{
    AluTrace, ConstTrace, NonPrimitiveTrace, PublicTrace, Traces, WitnessTrace,
};
use p3_koala_bear::KoalaBear;
use z00z_crypto::sha256_256;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    OneShotResourceSnapshotV2, OneShotResourceTelemetrySinkV2, TablePacking,
};
use z00z_plonky3_circuit_prover::{BatchStarkProof, BatchStarkProver};

use super::super::{
    epoch_frontier::EpochFrontierAuthorityV2,
    epoch_prover::{
        EPOCH_CHUNK_BYTES_V2, EPOCH_CHUNK_GRAMMAR_GENERATION_V2, EPOCH_DIRECT_AIR_GENERATION_V2,
        EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2, EPOCH_TRANSITION_BINDING_BYTES_V2,
    },
    epoch_range::EpochCodecReaderV2,
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
    },
};
use super::plonky3_epoch_event_source_columns::{
    npo_type as event_source_npo_type, EventSourceAirRoleV2, EventSourceTraceV2,
};
use super::plonky3_epoch_event_source_table::EventSourceProverV2;
use super::plonky3_epoch_event_source_witness as event_source_witness;
use super::plonky3_epoch_jmt_air::{jmt_chunk_npo_type, JmtChunkTraceV2};
use super::plonky3_epoch_jmt_table::JmtChunkProverV2;
use super::plonky3_epoch_jmt_witness as jmt_witness;
use super::plonky3_epoch_semantic_source_air::{
    SemanticSourceAirRoleV2, SemanticSourceProverV2, SemanticSourceTraceV2,
};
use super::plonky3_epoch_semantic_source_witness as semantic_source_witness;
use super::plonky3_epoch_sha256::ShaProverV2;
use super::plonky3_epoch_sha256_columns::{ShaAirRoleV2, ShaTraceV2};
use super::plonky3_epoch_sha256_witness as sha_witness;
use super::plonky3_epoch_trace_framing as trace_framing;
use super::plonky3_epoch_trace_framing_air::{
    TraceFramingAirRoleV2, TraceFramingProverV2, TraceFramingTraceV2,
    ROWS_V2 as TRACE_FRAMING_ROWS_V2,
};
use super::plonky3_epoch_transition_air::{
    npo_type as transition_npo_type, TransitionAirRoleV2, TransitionProverV2, TransitionTraceV2,
    ROWS_V2 as TRANSITION_ROWS_V2,
};
use super::plonky3_epoch_transition_witness as transition_witness;
use super::plonky3_epoch_typed_commitment as typed_commitment;
use super::plonky3_epoch_typed_commitment_air::{
    TypedCommitmentAirRoleV2, TypedCommitmentProverV2, TypedCommitmentTraceV2,
    COMMITMENTS_PER_TRANSITION_V2, ROWS_V2 as TYPED_ROWS_V2,
};
use super::plonky3_epoch_uniqueness_air::{UniquenessAirRoleV2, UniquenessProverV2, ROLE_COUNT_V2};
use super::plonky3_epoch_uniqueness_range::{
    self as uniqueness_range, UniquenessRangeProverV2, UniquenessRangeTraceV2,
};
use super::plonky3_epoch_uniqueness_slice::EpochUniquenessSliceV2;
use super::plonky3_epoch_uniqueness_witness::{
    self as uniqueness_witness, ParsedUniquenessWitnessV2, UniquenessAirWitnessV2,
};
use super::{
    decode_internal_canonical_batch_proof_v2, encode_internal_canonical_batch_proof_v2,
    hardened_koala_bear_config, EpochAirTableV2, EpochPreparedTransitionV2,
    EpochTraceChunkInputsV2, EpochTraceChunkV2, EpochTransitionBindingV2, Plonky3StarkConfigV2,
    RecursiveCheckpointRejectReasonV2,
};
use crate::CheckpointError;

const RECEIPT_DOMAIN_V2: &str = "z00z.storage.checkpoint.plonky3.epoch-transition-batch-receipt.v2";
const RECEIPT_LABEL_V2: &str = "actual_verified_trace_chunk";
const CHUNK_PROOF_MAGIC_V2: [u8; 8] = *b"Z00ZECP6";
const CHUNK_PROOF_WIRE_VERSION_V2: u16 = 6;
const CHUNK_PROOF_STATEMENT_COUNT_V2: u16 = 6;
const CHUNK_PROOF_MIN_GROUP_COUNT_V2: u16 = 6;
const CHUNK_PROOF_MAX_GROUP_COUNT_V2: u16 = 7;
const CHUNK_PROOF_GROUP_FIXED_BYTES_V2: usize = 1 + 4 + 32;
const CHUNK_PROOF_UNIQUENESS_METADATA_BYTES_V2: usize = 2;
const CHUNK_PROOF_FIXED_BYTES_V2: usize = 8
    + 2 * 5
    + 8
    + 4
    + EPOCH_CHUNK_BYTES_V2 * 6
    + CHUNK_PROOF_GROUP_FIXED_BYTES_V2 * CHUNK_PROOF_MIN_GROUP_COUNT_V2 as usize
    + CHUNK_PROOF_UNIQUENESS_METADATA_BYTES_V2;
const TRANSITION_CORE_GROUP_TABLE_COUNT_V2: usize = 2;
const TRANSITION_TYPED_GROUP_TABLE_COUNT_V2: usize = 5;
const TRANSITION_JMT_GROUP_TABLE_COUNT_V2: usize = 5;
const TRANSITION_FLOW_GROUP_TABLE_COUNT_V2: usize = 4;
const HASH_GROUP_TABLE_COUNT_V2: usize = 2;
const UNIQUENESS_GROUP_TABLE_COUNT_V2: usize = ROLE_COUNT_V2 + 4;
const TABLE_COUNT_V2: usize = TRANSITION_CORE_GROUP_TABLE_COUNT_V2
    + TRANSITION_TYPED_GROUP_TABLE_COUNT_V2
    + 3
    + 2
    + HASH_GROUP_TABLE_COUNT_V2
    + UNIQUENESS_GROUP_TABLE_COUNT_V2;
// Frontier admission is enabled only for the closed twenty-three-table theorem.
// The transition and uniqueness semantic-source AIRs bind the canonical event
// bytes to the complete uniqueness transcript/Close relation, while their
// linked SHA tables derive every structural event identifier from those same
// bytes. Group verification and the exact uniqueness partition are mandatory
// before this boundary can issue an admission receipt.
const FRONTIER_ADMISSION_COMPLETE_SEMANTIC_COVERAGE_V2: bool = true;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum EpochChunkProofGroupV2 {
    Transition = 1,
    TransitionTyped = 2,
    TransitionJmt = 3,
    TransitionFlow = 4,
    Hash = 5,
    UniquenessLower = 6,
    UniquenessUpper = 7,
}

impl EpochChunkProofGroupV2 {
    const fn decode(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Transition),
            2 => Some(Self::TransitionTyped),
            3 => Some(Self::TransitionJmt),
            4 => Some(Self::TransitionFlow),
            5 => Some(Self::Hash),
            6 => Some(Self::UniquenessLower),
            7 => Some(Self::UniquenessUpper),
            _ => None,
        }
    }

    fn expected(binding_count: usize) -> Result<Vec<Self>, CheckpointError> {
        let mut groups = vec![
            Self::Transition,
            Self::TransitionTyped,
            Self::TransitionJmt,
            Self::TransitionFlow,
            Self::Hash,
            Self::UniquenessLower,
        ];
        if EpochUniquenessSliceV2::canonical(binding_count)?.len() == 2 {
            groups.push(Self::UniquenessUpper);
        }
        Ok(groups)
    }

    const fn is_uniqueness(self) -> bool {
        matches!(self, Self::UniquenessLower | Self::UniquenessUpper)
    }

    const fn table_count(self) -> usize {
        match self {
            Self::Transition => TRANSITION_CORE_GROUP_TABLE_COUNT_V2,
            Self::TransitionTyped => TRANSITION_TYPED_GROUP_TABLE_COUNT_V2,
            Self::TransitionJmt => TRANSITION_JMT_GROUP_TABLE_COUNT_V2,
            Self::TransitionFlow => TRANSITION_FLOW_GROUP_TABLE_COUNT_V2,
            Self::Hash => HASH_GROUP_TABLE_COUNT_V2,
            Self::UniquenessLower | Self::UniquenessUpper => UNIQUENESS_GROUP_TABLE_COUNT_V2,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Transition => "transition",
            Self::TransitionTyped => "transition-typed",
            Self::TransitionJmt => "transition-jmt",
            Self::TransitionFlow => "transition-flow",
            Self::Hash => "hash",
            Self::UniquenessLower => "uniqueness-lower",
            Self::UniquenessUpper => "uniqueness-upper",
        }
    }
}

const ONE_SHOT_RESOURCE_PREFIX_V2: &str = "Z00Z_PLONKY3_ONE_SHOT_RESOURCE_V2 ";
const ONE_SHOT_RESOURCE_SCHEMA_V2: &str = "z00z.plonky3.one-shot-resource.v2";
const ONE_SHOT_RESOURCE_DOMAIN_V2: &str = "z00z/plonky3/one-shot-resource-lifetime/v2";

fn resource_telemetry_enabled_v2() -> bool {
    std::env::var_os("Z00Z_PLONKY3_RESOURCE_TELEMETRY").is_some()
}

fn process_memory_bytes_v2() -> (Option<u64>, Option<u64>) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (None, None);
    };
    let mut rss_bytes = None;
    let mut hwm_bytes = None;
    for line in status.lines() {
        let mut fields = line.split_ascii_whitespace();
        match fields.next() {
            Some("VmRSS:") => {
                rss_bytes = fields
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .and_then(|kib| kib.checked_mul(1024));
            }
            Some("VmHWM:") => {
                hwm_bytes = fields
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .and_then(|kib| kib.checked_mul(1024));
            }
            _ => {}
        }
    }
    (rss_bytes, hwm_bytes)
}

fn json_optional_u64_v2(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn one_shot_resource_json_line_v2(
    group: EpochChunkProofGroupV2,
    snapshot: OneShotResourceSnapshotV2,
    rss_bytes: Option<u64>,
    hwm_bytes: Option<u64>,
) -> String {
    let air_index = snapshot
        .air_index
        .map_or_else(|| "null".to_owned(), |value| value.to_string());
    let buffers = snapshot.visible_buffers;
    format!(
        concat!(
            "{}",
            "{{\"schema\":\"{}\",",
            "\"domain\":\"{}\",",
            "\"proof_group\":\"{}\",\"stage\":\"{}\",\"air_index\":{},",
            "\"process\":{{\"rss_bytes\":{},\"hwm_bytes\":{}}},",
            "\"visible_buffers\":{{",
            "\"main_trace\":{{\"len_bytes\":{},\"capacity_bytes\":{}}},",
            "\"permutation_trace\":{{\"len_bytes\":{},\"capacity_bytes\":{}}},",
            "\"quotient_lde\":{{\"len_bytes\":{},\"capacity_bytes\":{}}}}}}}"
        ),
        ONE_SHOT_RESOURCE_PREFIX_V2,
        ONE_SHOT_RESOURCE_SCHEMA_V2,
        ONE_SHOT_RESOURCE_DOMAIN_V2,
        group.name(),
        snapshot.stage.name(),
        air_index,
        json_optional_u64_v2(rss_bytes),
        json_optional_u64_v2(hwm_bytes),
        buffers.main_trace.len_bytes,
        buffers.main_trace.capacity_bytes,
        buffers.permutation_trace.len_bytes,
        buffers.permutation_trace.capacity_bytes,
        buffers.quotient_lde.len_bytes,
        buffers.quotient_lde.capacity_bytes,
    )
}

struct EpochOneShotResourceSinkV2 {
    group: EpochChunkProofGroupV2,
}

impl OneShotResourceTelemetrySinkV2 for EpochOneShotResourceSinkV2 {
    fn record(&mut self, snapshot: OneShotResourceSnapshotV2) {
        let (rss_bytes, hwm_bytes) = process_memory_bytes_v2();
        eprintln!(
            "{}",
            one_shot_resource_json_line_v2(self.group, snapshot, rss_bytes, hwm_bytes)
        );
    }
}

#[cfg(test)]
mod one_shot_resource_telemetry_tests {
    use super::*;
    use z00z_plonky3_circuit_prover::batch_stark_prover::{
        OneShotBufferBytesV2, OneShotResourceStageV2, OneShotVisibleBuffersV2,
    };

    #[test]
    fn one_shot_resource_json_line_v2_is_canonical_and_public_size_only() {
        let line = one_shot_resource_json_line_v2(
            EpochChunkProofGroupV2::Transition,
            OneShotResourceSnapshotV2 {
                stage: OneShotResourceStageV2::PostQuotientAir,
                air_index: Some(7),
                visible_buffers: OneShotVisibleBuffersV2 {
                    main_trace: OneShotBufferBytesV2 {
                        len_bytes: 11,
                        capacity_bytes: 13,
                    },
                    permutation_trace: OneShotBufferBytesV2 {
                        len_bytes: 17,
                        capacity_bytes: 19,
                    },
                    quotient_lde: OneShotBufferBytesV2 {
                        len_bytes: 23,
                        capacity_bytes: 29,
                    },
                },
            },
            Some(31),
            Some(37),
        );
        let payload = line
            .strip_prefix(ONE_SHOT_RESOURCE_PREFIX_V2)
            .expect("canonical telemetry prefix");
        let parsed: serde_json::Value =
            serde_json::from_str(payload).expect("canonical telemetry JSON");

        assert_eq!(parsed["schema"], ONE_SHOT_RESOURCE_SCHEMA_V2);
        assert_eq!(parsed["domain"], ONE_SHOT_RESOURCE_DOMAIN_V2);
        assert_eq!(parsed["proof_group"], "transition");
        assert_eq!(parsed["stage"], "post_quotient_air");
        assert_eq!(parsed["air_index"], 7);
        assert_eq!(parsed["process"]["rss_bytes"], 31);
        assert_eq!(parsed["process"]["hwm_bytes"], 37);
        assert_eq!(
            parsed["visible_buffers"]["quotient_lde"]["capacity_bytes"],
            29
        );
        assert!(!payload.contains("witness"));
        assert!(!payload.contains("private"));
    }

    #[test]
    fn one_shot_resource_telemetry_has_one_env_read_and_no_constraint_hooks() {
        let source = include_str!("plonky3_epoch_transition_batch.rs");
        assert_eq!(
            source.matches(concat!("std::env::", "var_os")).count(),
            1,
            "epoch-chunk orchestration must read the telemetry environment once",
        );
        for hook in [
            concat!("::check_", "constraints("),
            concat!("::check_trace_", "constraints("),
            concat!("::check_constraints_for_chunk_", "trace("),
        ] {
            assert!(
                !source.contains(hook),
                "resource telemetry must not duplicate semantic work through {hook}",
            );
        }
    }
}

#[cfg(test)]
mod slot_sliced_codec_tests {
    use super::*;

    #[test]
    fn group_schedule_is_exact_and_v5_is_fail_closed() {
        assert_eq!(CHUNK_PROOF_MAGIC_V2, *b"Z00ZECP6");
        assert_eq!(CHUNK_PROOF_WIRE_VERSION_V2, 6);
        assert_eq!(
            EpochChunkProofGroupV2::expected(8).expect("eight-slot schedule"),
            vec![
                EpochChunkProofGroupV2::Transition,
                EpochChunkProofGroupV2::TransitionTyped,
                EpochChunkProofGroupV2::TransitionJmt,
                EpochChunkProofGroupV2::TransitionFlow,
                EpochChunkProofGroupV2::Hash,
                EpochChunkProofGroupV2::UniquenessLower,
                EpochChunkProofGroupV2::UniquenessUpper,
            ],
        );
        assert_eq!(
            EpochChunkProofGroupV2::expected(4).expect("four-slot schedule"),
            vec![
                EpochChunkProofGroupV2::Transition,
                EpochChunkProofGroupV2::TransitionTyped,
                EpochChunkProofGroupV2::TransitionJmt,
                EpochChunkProofGroupV2::TransitionFlow,
                EpochChunkProofGroupV2::Hash,
                EpochChunkProofGroupV2::UniquenessLower,
            ],
        );
        assert_ne!(CHUNK_PROOF_MAGIC_V2, *b"Z00ZECP5");
        assert_ne!(CHUNK_PROOF_WIRE_VERSION_V2, 5);
        assert!(EpochChunkProofGroupV2::decode(8).is_none());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EpochChunkGroupProofV2 {
    group: EpochChunkProofGroupV2,
    uniqueness: Option<UniquenessGroupMetadataV2>,
    proof_digest: [u8; 32],
    proof_bytes: Vec<u8>,
}

/// Public metadata carried by every sliced uniqueness proof. The full chunk
/// statements remain in the proof's public values; this descriptor prevents a
/// proof for one local lane range from being substituted for another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UniquenessGroupMetadataV2 {
    slice: EpochUniquenessSliceV2,
}

/// Private capability created only after the complete linked-table proof has
/// passed the pinned Batch-STARK verifier.
pub(in crate::checkpoint) struct VerifiedEpochTraceChunkAdmissionV2 {
    pub(in crate::checkpoint) transition_statement: EpochTraceChunkV2,
    pub(in crate::checkpoint) bindings: Vec<EpochTransitionBindingV2>,
    pub(in crate::checkpoint) proof_digest: [u8; 32],
    pub(in crate::checkpoint) verification_receipt_digest: [u8; 32],
    pub(in crate::checkpoint) proof_bytes: Vec<u8>,
}

/// Canonical actual-verifiable direct-AIR proof for one bounded epoch chunk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plonky3EpochChunkProofV2 {
    transition_statement: EpochTraceChunkV2,
    trace_framing_statement: EpochTraceChunkV2,
    packed_statement: EpochTraceChunkV2,
    typed_statement: EpochTraceChunkV2,
    jmt_statement: EpochTraceChunkV2,
    uniqueness_statement: EpochTraceChunkV2,
    uniqueness_range_query_count: u64,
    bindings: Vec<EpochTransitionBindingV2>,
    group_proofs: Vec<EpochChunkGroupProofV2>,
    canonical_bytes: Vec<u8>,
}

impl Plonky3EpochChunkProofV2 {
    pub fn decode_canonical(
        authority: &EpochFrontierAuthorityV2,
        bytes: &[u8],
    ) -> Result<Self, CheckpointError> {
        if bytes.len()
            < RECURSIVE_OBJECT_PREHEADER_BYTES_V2
                + CHUNK_PROOF_FIXED_BYTES_V2
                + EPOCH_TRANSITION_BINDING_BYTES_V2
            || bytes.len() > super::RECURSIVE_INGRESS_BYTES_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        if registry.digest() != authority.registry_digest() {
            return Err(CheckpointError::Authority);
        }
        let preheader =
            registry.validate_preheader(bytes, RecursiveBoundedObjectV2::Plonky3EpochChunkProof)?;
        let payload = bytes
            .get(preheader.header_len..)
            .ok_or(CheckpointError::Canonical)?;
        let mut reader = EpochCodecReaderV2::new(payload);
        if reader.array::<8>()? != CHUNK_PROOF_MAGIC_V2
            || reader.u16()? != CHUNK_PROOF_WIRE_VERSION_V2
            || reader.u16()? != EPOCH_DIRECT_AIR_GENERATION_V2
            || reader.u16()? != EPOCH_CHUNK_GRAMMAR_GENERATION_V2
            || reader.u16()? != CHUNK_PROOF_STATEMENT_COUNT_V2
        {
            return Err(CheckpointError::Canonical);
        }
        let encoded_group_count = reader.u16()?;
        if !(CHUNK_PROOF_MIN_GROUP_COUNT_V2..=CHUNK_PROOF_MAX_GROUP_COUNT_V2)
            .contains(&encoded_group_count)
        {
            return Err(CheckpointError::Canonical);
        }
        let uniqueness_range_query_count = reader.u64()?;
        let binding_count = usize::try_from(reader.u32()?).map_err(|_| CheckpointError::Limit)?;
        if binding_count == 0
            || binding_count
                > usize::try_from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2)
                    .map_err(|_| CheckpointError::Limit)?
        {
            return Err(CheckpointError::Canonical);
        }
        let mut bindings = Vec::with_capacity(binding_count);
        for _ in 0..binding_count {
            bindings.push(EpochTransitionBindingV2::decode_canonical(
                reader.take(EPOCH_TRANSITION_BINDING_BYTES_V2)?,
            )?);
        }
        let transition_statement = EpochTraceChunkV2::decode_canonical(
            authority,
            &bindings,
            reader.take(EPOCH_CHUNK_BYTES_V2)?,
        )?;
        let trace_framing_statement = EpochTraceChunkV2::decode_canonical(
            authority,
            &bindings,
            reader.take(EPOCH_CHUNK_BYTES_V2)?,
        )?;
        let packed_statement = EpochTraceChunkV2::decode_canonical(
            authority,
            &bindings,
            reader.take(EPOCH_CHUNK_BYTES_V2)?,
        )?;
        let typed_statement = EpochTraceChunkV2::decode_canonical(
            authority,
            &bindings,
            reader.take(EPOCH_CHUNK_BYTES_V2)?,
        )?;
        let jmt_statement = EpochTraceChunkV2::decode_canonical(
            authority,
            &bindings,
            reader.take(EPOCH_CHUNK_BYTES_V2)?,
        )?;
        let uniqueness_statement = EpochTraceChunkV2::decode_canonical(
            authority,
            &bindings,
            reader.take(EPOCH_CHUNK_BYTES_V2)?,
        )?;
        let expected_groups = EpochChunkProofGroupV2::expected(binding_count)?;
        if usize::from(encoded_group_count) != expected_groups.len() {
            return Err(CheckpointError::Canonical);
        }
        let mut decoded_group_proofs = Vec::with_capacity(expected_groups.len());
        for expected_group in expected_groups {
            let group = EpochChunkProofGroupV2::decode(reader.u8()?)
                .filter(|group| *group == expected_group)
                .ok_or(CheckpointError::Canonical)?;
            let uniqueness = if group.is_uniqueness() {
                let start = usize::from(reader.u8()?);
                let len = usize::from(reader.u8()?);
                Some(UniquenessGroupMetadataV2 {
                    slice: EpochUniquenessSliceV2::from_wire(binding_count, start, len)?,
                })
            } else {
                None
            };
            let proof_len = usize::try_from(reader.u32()?).map_err(|_| CheckpointError::Limit)?;
            if proof_len == 0 || proof_len > super::RECURSIVE_INGRESS_BYTES_V2 {
                return Err(CheckpointError::Canonical);
            }
            let proof_bytes = reader.take(proof_len)?.to_vec();
            let proof_digest = reader.array()?;
            if proof_digest == [0; 32] || proof_digest != super::plonky3_proof_digest(&proof_bytes)
            {
                return Err(CheckpointError::Canonical);
            }
            decoded_group_proofs.push(EpochChunkGroupProofV2 {
                group,
                uniqueness,
                proof_digest,
                proof_bytes,
            });
        }
        if !reader.is_done() {
            return Err(CheckpointError::Canonical);
        }
        let artifact = Self {
            transition_statement,
            trace_framing_statement,
            packed_statement,
            typed_statement,
            jmt_statement,
            uniqueness_statement,
            uniqueness_range_query_count,
            bindings,
            group_proofs: decoded_group_proofs,
            canonical_bytes: bytes.to_vec(),
        };
        artifact.verify()?;
        Ok(artifact)
    }

    #[must_use]
    pub const fn transition_statement(&self) -> &EpochTraceChunkV2 {
        &self.transition_statement
    }

    #[must_use]
    pub const fn trace_framing_statement(&self) -> &EpochTraceChunkV2 {
        &self.trace_framing_statement
    }

    #[must_use]
    pub const fn packed_statement(&self) -> &EpochTraceChunkV2 {
        &self.packed_statement
    }

    #[must_use]
    pub const fn typed_statement(&self) -> &EpochTraceChunkV2 {
        &self.typed_statement
    }

    #[must_use]
    pub const fn jmt_statement(&self) -> &EpochTraceChunkV2 {
        &self.jmt_statement
    }

    #[must_use]
    pub const fn uniqueness_statement(&self) -> &EpochTraceChunkV2 {
        &self.uniqueness_statement
    }

    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    #[must_use]
    pub fn bindings(&self) -> &[EpochTransitionBindingV2] {
        &self.bindings
    }

    #[must_use]
    pub fn proof_digest(&self) -> [u8; 32] {
        super::plonky3_proof_digest(&self.canonical_bytes)
    }

    #[must_use]
    pub fn internal_proof_bundle_len(&self) -> usize {
        self.group_proofs
            .iter()
            .map(|group| group.proof_bytes.len())
            .sum()
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn trace_row_count(&self) -> Result<usize, CheckpointError> {
        self.group_proofs.iter().try_fold(0_usize, |total, group| {
            decode_internal_canonical_batch_proof_v2(&group.proof_bytes)?
                .non_primitives
                .iter()
                .try_fold(total, |subtotal, entry| {
                    subtotal
                        .checked_add(entry.rows)
                        .ok_or(CheckpointError::Overflow)
                })
        })
    }

    #[must_use]
    pub const fn table_count(&self) -> usize {
        TABLE_COUNT_V2
    }

    pub(super) fn decode_group_proof(
        &self,
        group: EpochChunkProofGroupV2,
    ) -> Result<BatchStarkProof<Plonky3StarkConfigV2>, CheckpointError> {
        let encoded = self
            .group_proofs
            .iter()
            .find(|encoded| encoded.group == group)
            .ok_or(CheckpointError::Canonical)?;
        if encoded.proof_digest != super::plonky3_proof_digest(&encoded.proof_bytes) {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch,
            ));
        }
        decode_internal_canonical_batch_proof_v2(&encoded.proof_bytes)
    }

    fn uniqueness_metadata(
        &self,
        group: EpochChunkProofGroupV2,
    ) -> Result<UniquenessGroupMetadataV2, CheckpointError> {
        self.group_proofs
            .iter()
            .find(|encoded| encoded.group == group)
            .and_then(|encoded| encoded.uniqueness)
            .ok_or(CheckpointError::Canonical)
    }

    fn validate_uniqueness_partition(&self) -> Result<(), CheckpointError> {
        let expected_groups = EpochChunkProofGroupV2::expected(self.bindings.len())?;
        if self.group_proofs.len() != expected_groups.len()
            || self
                .group_proofs
                .iter()
                .map(|group| group.group)
                .ne(expected_groups.into_iter())
        {
            return Err(CheckpointError::Canonical);
        }
        let metadata = self
            .group_proofs
            .iter()
            .filter(|group| group.group.is_uniqueness())
            .map(|group| group.uniqueness.ok_or(CheckpointError::Canonical))
            .collect::<Result<Vec<_>, _>>()?;
        let slices = metadata
            .iter()
            .map(|metadata| metadata.slice)
            .collect::<Vec<_>>();
        EpochUniquenessSliceV2::validate_partition(self.bindings.len(), &slices)?;
        let semantic_rows = slices.iter().try_fold(0_u64, |total, slice| {
            let end = slice.end()?;
            self.bindings
                .get(slice.start()..end)
                .ok_or(CheckpointError::Canonical)?
                .iter()
                .try_fold(total, |sum, binding| {
                    sum.checked_add(binding.inputs().uniqueness_row_count)
                        .ok_or(CheckpointError::Overflow)
                })
        })?;
        if semantic_rows != self.uniqueness_statement.inputs().row_count {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    pub fn verify(&self) -> Result<(), CheckpointError> {
        validate_statements(
            &self.transition_statement,
            &self.trace_framing_statement,
            &self.packed_statement,
            &self.typed_statement,
            &self.jmt_statement,
            &self.uniqueness_statement,
            &self.bindings,
        )?;
        self.validate_uniqueness_partition()?;
        if self.canonical_bytes != encode_chunk_proof(self)? {
            return Err(CheckpointError::Canonical);
        }
        let event_bytes = binding_event_bytes(&self.bindings)?;
        let transition_public =
            transition_witness::public_values(&self.transition_statement, &self.bindings)?;
        verify_transition_core_group_proof(
            &self.decode_group_proof(EpochChunkProofGroupV2::Transition)?,
            &transition_public,
            &trace_framing::public_values(
                &self.trace_framing_statement,
                &self.bindings,
                event_bytes,
            )?,
        )?;
        let typed_public = typed_public_values(&self.typed_statement, &self.bindings)?;
        let jmt_public = jmt_witness::chunk_public_values(&self.jmt_statement, &self.bindings)?;
        let jmt_sha_public = sha_witness::jmt_linked_public_values(&self.jmt_statement)?;
        let semantic_event_source_public = event_source_witness::public_values_for_slice(
            &self.packed_statement,
            &self.bindings,
            EpochUniquenessSliceV2::full(self.bindings.len())?,
        )?;
        let semantic_source_public = semantic_source_witness::public_values_for_slice(
            &self.packed_statement,
            &self.bindings,
            EpochUniquenessSliceV2::full(self.bindings.len())?,
        )?;
        let semantic_sha_public = sha_witness::chain_public_values_for_slice(
            &self.transition_statement,
            &self.bindings,
            EpochUniquenessSliceV2::full(self.bindings.len())?,
        )?;
        verify_transition_typed_group_proof(
            &self.decode_group_proof(EpochChunkProofGroupV2::TransitionTyped)?,
            &transition_public,
            &typed_public,
            &semantic_event_source_public,
            &semantic_source_public,
            &semantic_sha_public,
        )?;
        verify_transition_jmt_group_proof(
            &self.decode_group_proof(EpochChunkProofGroupV2::TransitionJmt)?,
            &jmt_public,
            &jmt_sha_public,
            &semantic_event_source_public,
            &semantic_source_public,
            &semantic_sha_public,
        )?;
        verify_transition_flow_group_proof(
            &self.decode_group_proof(EpochChunkProofGroupV2::TransitionFlow)?,
            &transition_public,
            &semantic_event_source_public,
            &semantic_source_public,
            &semantic_sha_public,
        )?;
        verify_hash_group_proof(
            &self.decode_group_proof(EpochChunkProofGroupV2::Hash)?,
            &event_source_witness::public_values_for_slice(
                &self.packed_statement,
                &self.bindings,
                EpochUniquenessSliceV2::full(self.bindings.len())?,
            )?,
            &sha_witness::chain_public_values_for_slice(
                &self.transition_statement,
                &self.bindings,
                EpochUniquenessSliceV2::full(self.bindings.len())?,
            )?,
        )?;
        let mut verified_range_count = 0_u64;
        for group in [
            EpochChunkProofGroupV2::UniquenessLower,
            EpochChunkProofGroupV2::UniquenessUpper,
        ] {
            let Ok(metadata) = self.uniqueness_metadata(group) else {
                if group == EpochChunkProofGroupV2::UniquenessUpper && self.bindings.len() <= 4 {
                    continue;
                }
                return Err(CheckpointError::Canonical);
            };
            let end = metadata.slice.end()?;
            let slice_bindings = self
                .bindings
                .get(metadata.slice.start()..end)
                .ok_or(CheckpointError::Canonical)?;
            let semantic_row_count = slice_bindings.iter().try_fold(0_u64, |total, binding| {
                total
                    .checked_add(binding.inputs().uniqueness_row_count)
                    .ok_or(CheckpointError::Overflow)
            })?;
            let range_count = verify_uniqueness_group_proof(
                group,
                &self.decode_group_proof(group)?,
                &uniqueness_witness::public_values_for_slice(
                    &self.uniqueness_statement,
                    metadata.slice,
                    semantic_row_count,
                )?,
                &uniqueness_range::public_prefix_for_slice(
                    &self.uniqueness_statement,
                    metadata.slice,
                )?,
                &event_source_witness::public_values_for_slice(
                    &self.packed_statement,
                    &self.bindings,
                    metadata.slice,
                )?,
                &semantic_source_witness::public_values_for_slice(
                    &self.packed_statement,
                    &self.bindings,
                    metadata.slice,
                )?,
                &sha_witness::chain_public_values_for_slice(
                    &self.transition_statement,
                    &self.bindings,
                    metadata.slice,
                )?,
            )?;
            verified_range_count = verified_range_count
                .checked_add(range_count)
                .ok_or(CheckpointError::Overflow)?;
        }
        if verified_range_count != self.uniqueness_range_query_count {
            return Err(CheckpointError::Canonical);
        }
        Ok(())
    }

    pub(in crate::checkpoint) fn verified_frontier_admission(
        &self,
    ) -> Result<VerifiedEpochTraceChunkAdmissionV2, CheckpointError> {
        if !FRONTIER_ADMISSION_COMPLETE_SEMANTIC_COVERAGE_V2 {
            return Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
            ));
        }
        self.verify()?;
        let binding_count =
            u32::try_from(self.bindings.len()).map_err(|_| CheckpointError::Limit)?;
        let verification_receipt_digest = sha256_256(
            RECEIPT_DOMAIN_V2,
            RECEIPT_LABEL_V2,
            &[
                &self.transition_statement.digest(),
                &self.proof_digest(),
                &binding_count.to_le_bytes(),
            ],
        );
        Ok(VerifiedEpochTraceChunkAdmissionV2 {
            transition_statement: self.transition_statement.clone(),
            bindings: self.bindings.clone(),
            proof_digest: self.proof_digest(),
            verification_receipt_digest,
            proof_bytes: self.canonical_bytes.clone(),
        })
    }
}

fn encode_chunk_proof(proof: &Plonky3EpochChunkProofV2) -> Result<Vec<u8>, CheckpointError> {
    let binding_bytes = proof
        .bindings
        .len()
        .checked_mul(EPOCH_TRANSITION_BINDING_BYTES_V2)
        .ok_or(CheckpointError::Overflow)?;
    let mut payload = Vec::with_capacity(
        CHUNK_PROOF_FIXED_BYTES_V2
            .checked_add(binding_bytes)
            .and_then(|len| {
                proof.group_proofs.iter().try_fold(len, |total, group| {
                    total.checked_add(group.proof_bytes.len())
                })
            })
            .ok_or(CheckpointError::Overflow)?,
    );
    payload.extend_from_slice(&CHUNK_PROOF_MAGIC_V2);
    payload.extend_from_slice(&CHUNK_PROOF_WIRE_VERSION_V2.to_le_bytes());
    payload.extend_from_slice(&EPOCH_DIRECT_AIR_GENERATION_V2.to_le_bytes());
    payload.extend_from_slice(&EPOCH_CHUNK_GRAMMAR_GENERATION_V2.to_le_bytes());
    payload.extend_from_slice(&CHUNK_PROOF_STATEMENT_COUNT_V2.to_le_bytes());
    let expected_groups = EpochChunkProofGroupV2::expected(proof.bindings.len())?;
    if proof.group_proofs.len() != expected_groups.len() {
        return Err(CheckpointError::Canonical);
    }
    payload.extend_from_slice(
        &u16::try_from(expected_groups.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(&proof.uniqueness_range_query_count.to_le_bytes());
    payload.extend_from_slice(
        &u32::try_from(proof.bindings.len())
            .map_err(|_| CheckpointError::Limit)?
            .to_le_bytes(),
    );
    for binding in &proof.bindings {
        payload.extend_from_slice(&binding.encode_canonical());
    }
    for statement in [
        &proof.transition_statement,
        &proof.trace_framing_statement,
        &proof.packed_statement,
        &proof.typed_statement,
        &proof.jmt_statement,
        &proof.uniqueness_statement,
    ] {
        payload.extend_from_slice(statement.canonical_bytes());
    }
    for (expected_group, group) in expected_groups.into_iter().zip(&proof.group_proofs) {
        if group.group != expected_group
            || group.proof_digest == [0; 32]
            || group.proof_digest != super::plonky3_proof_digest(&group.proof_bytes)
        {
            return Err(CheckpointError::Canonical);
        }
        payload.push(group.group as u8);
        match (group.group.is_uniqueness(), group.uniqueness) {
            (true, Some(metadata)) => {
                payload.push(
                    u8::try_from(metadata.slice.start()).map_err(|_| CheckpointError::Limit)?,
                );
                payload
                    .push(u8::try_from(metadata.slice.len()).map_err(|_| CheckpointError::Limit)?);
            }
            (false, None) => {}
            _ => return Err(CheckpointError::Canonical),
        }
        payload.extend_from_slice(
            &u32::try_from(group.proof_bytes.len())
                .map_err(|_| CheckpointError::Limit)?
                .to_le_bytes(),
        );
        payload.extend_from_slice(&group.proof_bytes);
        payload.extend_from_slice(&group.proof_digest);
    }
    let registry = CheckpointVersionRegistryV2::authority_pinned()?;
    let preheader = registry.encode_preheader(
        RecursiveBoundedObjectV2::Plonky3EpochChunkProof,
        payload.len(),
    )?;
    let mut bytes = Vec::with_capacity(
        RECURSIVE_OBJECT_PREHEADER_BYTES_V2
            .checked_add(payload.len())
            .ok_or(CheckpointError::Overflow)?,
    );
    bytes.extend_from_slice(&preheader);
    bytes.extend_from_slice(&payload);
    if bytes.len() > super::RECURSIVE_INGRESS_BYTES_V2 {
        return Err(CheckpointError::Limit);
    }
    Ok(bytes)
}

fn typed_public_values(
    statement: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<Vec<KoalaBear>, CheckpointError> {
    let commitments = bindings
        .iter()
        .map(EpochTransitionBindingV2::typed_commitment_digests)
        .collect::<Vec<_>>();
    typed_commitment::public_values(statement, &commitments)
}

fn validate_statements(
    transition: &EpochTraceChunkV2,
    trace_framing: &EpochTraceChunkV2,
    packed: &EpochTraceChunkV2,
    typed: &EpochTraceChunkV2,
    jmt: &EpochTraceChunkV2,
    uniqueness: &EpochTraceChunkV2,
    bindings: &[EpochTransitionBindingV2],
) -> Result<(), CheckpointError> {
    let binding_count = u32::try_from(bindings.len()).map_err(|_| CheckpointError::Limit)?;
    if binding_count == 0 || binding_count > super::EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let transition_inputs = transition.inputs();
    let first = bindings
        .first()
        .copied()
        .ok_or(CheckpointError::Canonical)?
        .ordinal();
    let last = bindings
        .last()
        .copied()
        .ok_or(CheckpointError::Canonical)?
        .ordinal();
    let transition_count = u64::from(binding_count);
    let typed_row_start = u64::from(first)
        .checked_mul(COMMITMENTS_PER_TRANSITION_V2 as u64)
        .ok_or(CheckpointError::Overflow)?;
    let typed_row_count = transition_count
        .checked_mul(COMMITMENTS_PER_TRANSITION_V2 as u64)
        .ok_or(CheckpointError::Overflow)?;
    if transition_inputs.table != EpochAirTableV2::Transition
        || transition_inputs.replica != 0
        || transition_inputs.first_transition != first
        || transition_inputs.last_transition != last
        || transition_inputs.row_start != u64::from(first)
        || transition_inputs.row_count != transition_count
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_trace_framing_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::TraceFraming,
        row_start: u64::from(first),
        row_count: transition_count,
        ..transition_inputs
    };
    if trace_framing.inputs() != expected_trace_framing_inputs {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let event_bytes = binding_event_bytes(bindings)?;
    let expected_packed_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::PackedRange,
        row_start: 0,
        row_count: event_bytes,
        ..transition_inputs
    };
    if packed.inputs() != expected_packed_inputs {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_typed_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::TypedCommitment,
        row_start: typed_row_start,
        row_count: typed_row_count,
        ..transition_inputs
    };
    if typed.inputs() != expected_typed_inputs {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_jmt_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::JmtUpdate,
        row_start: 0,
        row_count: bindings.iter().try_fold(0_u64, |total, binding| {
            total
                .checked_add(binding.inputs().jmt_record_count)
                .ok_or(CheckpointError::Overflow)
        })?,
        ..transition_inputs
    };
    if jmt.inputs() != expected_jmt_inputs {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    let expected_uniqueness_inputs = EpochTraceChunkInputsV2 {
        table: EpochAirTableV2::Uniqueness,
        row_start: transition_inputs.event_start,
        row_count: bindings.iter().try_fold(0_u64, |total, binding| {
            total
                .checked_add(binding.inputs().uniqueness_row_count)
                .ok_or(CheckpointError::Overflow)
        })?,
        ..transition_inputs
    };
    if uniqueness.inputs() != expected_uniqueness_inputs {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn binding_event_bytes(bindings: &[EpochTransitionBindingV2]) -> Result<u64, CheckpointError> {
    bindings.iter().try_fold(0_u64, |total, binding| {
        total
            .checked_add(binding.inputs().event_bytes)
            .ok_or(CheckpointError::Overflow)
    })
}

fn empty_traces(
    non_primitive_traces: Vec<(
        p3_circuit::ops::NpoTypeId,
        Box<dyn NonPrimitiveTrace<KoalaBear>>,
    )>,
) -> Traces<KoalaBear> {
    Traces {
        witness_trace: WitnessTrace::new(Vec::new()),
        const_trace: ConstTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        public_trace: PublicTrace {
            index: Vec::new(),
            values: Vec::new(),
        },
        alu_trace: AluTrace::from_records(Vec::new()),
        non_primitive_traces: non_primitive_traces.into_iter().collect(),
        tag_to_witness: Default::default(),
    }
}

fn transition_core_group_traces(
    transition_rows: Vec<super::plonky3_epoch_transition_air::TransitionRowV2>,
    trace_framing_rows: Vec<super::plonky3_epoch_trace_framing_air::TraceFramingRowV2>,
) -> Traces<KoalaBear> {
    empty_traces(vec![
        (
            TransitionAirRoleV2::Core.npo_type(),
            Box::new(TransitionTraceV2 {
                role: TransitionAirRoleV2::Core,
                rows: transition_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            TraceFramingAirRoleV2::LinkedConsumer.npo_type(),
            Box::new(TraceFramingTraceV2 {
                role: TraceFramingAirRoleV2::LinkedConsumer,
                rows: trace_framing_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
    ])
}

fn transition_typed_group_traces(
    transition_rows: Vec<super::plonky3_epoch_transition_air::TransitionRowV2>,
    typed_rows: Vec<super::plonky3_epoch_typed_commitment_air::TypedCommitmentRowV2>,
    event_source: EventSourceTraceV2,
    semantic_source: SemanticSourceTraceV2,
    semantic_sha_trace: ShaTraceV2,
) -> Traces<KoalaBear> {
    empty_traces(vec![
        (
            TransitionAirRoleV2::SemanticTyped.npo_type(),
            Box::new(TransitionTraceV2 {
                role: TransitionAirRoleV2::SemanticTyped,
                rows: transition_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            TypedCommitmentAirRoleV2::LinkedConsumer.npo_type(),
            Box::new(TypedCommitmentTraceV2 {
                role: TypedCommitmentAirRoleV2::LinkedConsumer,
                rows: typed_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            EventSourceAirRoleV2::SemanticTransition.npo_type(),
            Box::new(event_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            SemanticSourceAirRoleV2::TransitionTyped.npo_type(),
            Box::new(semantic_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            ShaAirRoleV2::SemanticTransitionChain.npo_type(),
            Box::new(semantic_sha_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
    ])
}

fn transition_jmt_group_traces(
    jmt_trace: JmtChunkTraceV2,
    jmt_sha_trace: ShaTraceV2,
    event_source: EventSourceTraceV2,
    semantic_source: SemanticSourceTraceV2,
    semantic_sha_trace: ShaTraceV2,
) -> Traces<KoalaBear> {
    empty_traces(vec![
        (
            jmt_chunk_npo_type(),
            Box::new(jmt_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            ShaAirRoleV2::JmtLinked.npo_type(),
            Box::new(jmt_sha_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            EventSourceAirRoleV2::SemanticTransition.npo_type(),
            Box::new(event_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            SemanticSourceAirRoleV2::TransitionJmt.npo_type(),
            Box::new(semantic_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            ShaAirRoleV2::SemanticTransitionChain.npo_type(),
            Box::new(semantic_sha_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
    ])
}

fn transition_flow_group_traces(
    transition_rows: Vec<super::plonky3_epoch_transition_air::TransitionRowV2>,
    event_source: EventSourceTraceV2,
    semantic_source: SemanticSourceTraceV2,
    semantic_sha_trace: ShaTraceV2,
) -> Traces<KoalaBear> {
    empty_traces(vec![
        (
            TransitionAirRoleV2::SemanticFlow.npo_type(),
            Box::new(TransitionTraceV2 {
                role: TransitionAirRoleV2::SemanticFlow,
                rows: transition_rows,
            }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            EventSourceAirRoleV2::SemanticTransition.npo_type(),
            Box::new(event_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            SemanticSourceAirRoleV2::TransitionFlow.npo_type(),
            Box::new(semantic_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            ShaAirRoleV2::SemanticTransitionChain.npo_type(),
            Box::new(semantic_sha_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
    ])
}

fn hash_group_traces(event_source: EventSourceTraceV2, sha_trace: ShaTraceV2) -> Traces<KoalaBear> {
    empty_traces(vec![
        (
            event_source_npo_type(),
            Box::new(event_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
        (
            ShaAirRoleV2::Chain.npo_type(),
            Box::new(sha_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
        ),
    ])
}

fn uniqueness_group_traces(
    uniqueness: UniquenessAirWitnessV2,
    uniqueness_range_rows: Vec<uniqueness_range::UniquenessRangeRowV2>,
    event_source: EventSourceTraceV2,
    semantic_source: SemanticSourceTraceV2,
    semantic_sha_trace: ShaTraceV2,
) -> Traces<KoalaBear> {
    let mut traces = uniqueness
        .traces
        .into_iter()
        .map(|trace| {
            (
                trace.role.npo_type(),
                Box::new(trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
            )
        })
        .collect::<Vec<_>>();
    traces.push((
        uniqueness_range::npo_type(),
        Box::new(UniquenessRangeTraceV2 {
            rows: uniqueness_range_rows,
        }) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
    ));
    traces.push((
        EventSourceAirRoleV2::SemanticUniqueness.npo_type(),
        Box::new(event_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
    ));
    traces.push((
        SemanticSourceAirRoleV2::Uniqueness.npo_type(),
        Box::new(semantic_source) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
    ));
    traces.push((
        ShaAirRoleV2::SemanticUniquenessChain.npo_type(),
        Box::new(semantic_sha_trace) as Box<dyn NonPrimitiveTrace<KoalaBear>>,
    ));
    empty_traces(traces)
}

fn transition_core_group_table_provers(
) -> Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> {
    vec![
        Box::new(TransitionProverV2::new(TransitionAirRoleV2::Core)),
        Box::new(TraceFramingProverV2::new(
            TraceFramingAirRoleV2::LinkedConsumer,
        )),
    ]
}

fn transition_typed_group_table_provers(
) -> Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> {
    vec![
        Box::new(TransitionProverV2::new(TransitionAirRoleV2::SemanticTyped)),
        Box::new(TypedCommitmentProverV2::new(
            TypedCommitmentAirRoleV2::LinkedConsumer,
        )),
        Box::new(EventSourceProverV2::new(
            EventSourceAirRoleV2::SemanticTransition,
        )),
        Box::new(SemanticSourceProverV2::new(
            SemanticSourceAirRoleV2::TransitionTyped,
        )),
        Box::new(ShaProverV2::new(ShaAirRoleV2::SemanticTransitionChain)),
    ]
}

fn transition_jmt_group_table_provers(
) -> Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> {
    vec![
        Box::new(JmtChunkProverV2),
        Box::new(ShaProverV2::new(ShaAirRoleV2::JmtLinked)),
        Box::new(EventSourceProverV2::new(
            EventSourceAirRoleV2::SemanticTransition,
        )),
        Box::new(SemanticSourceProverV2::new(
            SemanticSourceAirRoleV2::TransitionJmt,
        )),
        Box::new(ShaProverV2::new(ShaAirRoleV2::SemanticTransitionChain)),
    ]
}

fn transition_flow_group_table_provers(
) -> Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> {
    vec![
        Box::new(TransitionProverV2::new(TransitionAirRoleV2::SemanticFlow)),
        Box::new(EventSourceProverV2::new(
            EventSourceAirRoleV2::SemanticTransition,
        )),
        Box::new(SemanticSourceProverV2::new(
            SemanticSourceAirRoleV2::TransitionFlow,
        )),
        Box::new(ShaProverV2::new(ShaAirRoleV2::SemanticTransitionChain)),
    ]
}

fn hash_group_table_provers(
) -> Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> {
    vec![
        Box::new(EventSourceProverV2::new(EventSourceAirRoleV2::Hash)),
        Box::new(ShaProverV2::new(ShaAirRoleV2::Chain)),
    ]
}

fn uniqueness_group_table_provers(
) -> Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> {
    let mut provers: Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> =
        Vec::with_capacity(UNIQUENESS_GROUP_TABLE_COUNT_V2);
    for role in UniquenessAirRoleV2::ALL {
        provers.push(Box::new(UniquenessProverV2::new(role)));
    }
    provers.push(Box::new(UniquenessRangeProverV2));
    provers.push(Box::new(EventSourceProverV2::new(
        EventSourceAirRoleV2::SemanticUniqueness,
    )));
    provers.push(Box::new(SemanticSourceProverV2::new(
        SemanticSourceAirRoleV2::Uniqueness,
    )));
    provers.push(Box::new(ShaProverV2::new(
        ShaAirRoleV2::SemanticUniquenessChain,
    )));
    provers
}

pub(super) fn epoch_chunk_table_provers(
) -> Vec<Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>> {
    let mut provers = transition_core_group_table_provers();
    provers.extend(transition_typed_group_table_provers());
    provers.extend([
        Box::new(JmtChunkProverV2)
            as Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>,
        Box::new(ShaProverV2::new(ShaAirRoleV2::JmtLinked)),
        Box::new(SemanticSourceProverV2::new(
            SemanticSourceAirRoleV2::TransitionJmt,
        )),
    ]);
    provers.extend([
        Box::new(TransitionProverV2::new(TransitionAirRoleV2::SemanticFlow))
            as Box<dyn z00z_plonky3_circuit_prover::TableProver<Plonky3StarkConfigV2>>,
        Box::new(SemanticSourceProverV2::new(
            SemanticSourceAirRoleV2::TransitionFlow,
        )),
    ]);
    provers.extend(hash_group_table_provers());
    provers.extend(uniqueness_group_table_provers());
    debug_assert_eq!(provers.len(), TABLE_COUNT_V2);
    provers
}

fn configured_group_prover(
    group: EpochChunkProofGroupV2,
    table_packing: TablePacking,
) -> BatchStarkProver<Plonky3StarkConfigV2> {
    let mut prover = BatchStarkProver::new(hardened_koala_bear_config())
        .with_table_packing(table_packing)
        .with_one_shot_lde_capacity_bits(usize::from(super::PLONKY3_FRI_LOG_BLOWUP_V2))
        .with_one_shot_pre_commit_reclaimer(super::trim_prover_heap)
        .with_debug_lookups();
    let table_provers = match group {
        EpochChunkProofGroupV2::Transition => transition_core_group_table_provers(),
        EpochChunkProofGroupV2::TransitionTyped => transition_typed_group_table_provers(),
        EpochChunkProofGroupV2::TransitionJmt => transition_jmt_group_table_provers(),
        EpochChunkProofGroupV2::TransitionFlow => transition_flow_group_table_provers(),
        EpochChunkProofGroupV2::Hash => hash_group_table_provers(),
        EpochChunkProofGroupV2::UniquenessLower | EpochChunkProofGroupV2::UniquenessUpper => {
            uniqueness_group_table_provers()
        }
    };
    debug_assert_eq!(table_provers.len(), group.table_count());
    for table_prover in table_provers {
        prover.register_table_prover(table_prover);
    }
    prover
}

fn exact_public_values<'a>(
    proof: &'a BatchStarkProof<Plonky3StarkConfigV2>,
    op_type: p3_circuit::ops::NpoTypeId,
) -> Result<&'a [KoalaBear], CheckpointError> {
    let mut entries = proof
        .non_primitives
        .iter()
        .filter(|entry| entry.op_type == op_type);
    let values = entries
        .next()
        .map(|entry| entry.public_values.as_slice())
        .ok_or(CheckpointError::BackendVerificationFailed)?;
    if entries.next().is_some() {
        return Err(CheckpointError::BackendVerificationFailed);
    }
    Ok(values)
}

fn verify_group_tables(
    group: EpochChunkProofGroupV2,
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
) -> Result<(), CheckpointError> {
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(TRACE_FRAMING_ROWS_V2);
    configured_group_prover(group, table_packing)
        .verify_all_tables::<KoalaBear>(proof)
        .map_err(|error| {
            CheckpointError::Backend(format!(
                "Plonky3 epoch {:?} group verifier rejected proof: {error}",
                group
            ))
        })?;
    if proof.non_primitives.len() != group.table_count() {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn verify_transition_core_group_proof(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_transition_public: &[KoalaBear],
    expected_trace_framing_public: &[KoalaBear],
) -> Result<(), CheckpointError> {
    verify_group_tables(EpochChunkProofGroupV2::Transition, proof)?;
    if exact_public_values(proof, transition_npo_type())? != expected_transition_public
        || exact_public_values(proof, TraceFramingAirRoleV2::LinkedConsumer.npo_type())?
            != expected_trace_framing_public
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn verify_transition_typed_group_proof(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_transition_public: &[KoalaBear],
    expected_typed_public: &[KoalaBear],
    expected_event_source_public: &[KoalaBear],
    expected_semantic_source_public: &[KoalaBear],
    expected_semantic_sha_public: &[KoalaBear],
) -> Result<(), CheckpointError> {
    verify_group_tables(EpochChunkProofGroupV2::TransitionTyped, proof)?;
    if exact_public_values(proof, TransitionAirRoleV2::SemanticTyped.npo_type())?
        != expected_transition_public
        || exact_public_values(proof, TypedCommitmentAirRoleV2::LinkedConsumer.npo_type())?
            != expected_typed_public
        || exact_public_values(proof, EventSourceAirRoleV2::SemanticTransition.npo_type())?
            != expected_event_source_public
        || exact_public_values(proof, SemanticSourceAirRoleV2::TransitionTyped.npo_type())?
            != expected_semantic_source_public
        || exact_public_values(proof, ShaAirRoleV2::SemanticTransitionChain.npo_type())?
            != expected_semantic_sha_public
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn verify_transition_jmt_group_proof(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_jmt_public: &[KoalaBear],
    expected_jmt_sha_public: &[KoalaBear],
    expected_event_source_public: &[KoalaBear],
    expected_semantic_source_public: &[KoalaBear],
    expected_semantic_sha_public: &[KoalaBear],
) -> Result<(), CheckpointError> {
    verify_group_tables(EpochChunkProofGroupV2::TransitionJmt, proof)?;
    if exact_public_values(proof, jmt_chunk_npo_type())? != expected_jmt_public
        || exact_public_values(proof, ShaAirRoleV2::JmtLinked.npo_type())?
            != expected_jmt_sha_public
        || exact_public_values(proof, EventSourceAirRoleV2::SemanticTransition.npo_type())?
            != expected_event_source_public
        || exact_public_values(proof, SemanticSourceAirRoleV2::TransitionJmt.npo_type())?
            != expected_semantic_source_public
        || exact_public_values(proof, ShaAirRoleV2::SemanticTransitionChain.npo_type())?
            != expected_semantic_sha_public
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn verify_transition_flow_group_proof(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_transition_public: &[KoalaBear],
    expected_event_source_public: &[KoalaBear],
    expected_semantic_source_public: &[KoalaBear],
    expected_semantic_sha_public: &[KoalaBear],
) -> Result<(), CheckpointError> {
    verify_group_tables(EpochChunkProofGroupV2::TransitionFlow, proof)?;
    if exact_public_values(proof, TransitionAirRoleV2::SemanticFlow.npo_type())?
        != expected_transition_public
        || exact_public_values(proof, EventSourceAirRoleV2::SemanticTransition.npo_type())?
            != expected_event_source_public
        || exact_public_values(proof, SemanticSourceAirRoleV2::TransitionFlow.npo_type())?
            != expected_semantic_source_public
        || exact_public_values(proof, ShaAirRoleV2::SemanticTransitionChain.npo_type())?
            != expected_semantic_sha_public
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn verify_hash_group_proof(
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_event_source_public: &[KoalaBear],
    expected_sha_public: &[KoalaBear],
) -> Result<(), CheckpointError> {
    verify_group_tables(EpochChunkProofGroupV2::Hash, proof)?;
    if exact_public_values(proof, event_source_npo_type())? != expected_event_source_public
        || exact_public_values(proof, ShaAirRoleV2::Chain.npo_type())? != expected_sha_public
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(())
}

fn verify_uniqueness_group_proof(
    group: EpochChunkProofGroupV2,
    proof: &BatchStarkProof<Plonky3StarkConfigV2>,
    expected_uniqueness_public: &[KoalaBear],
    expected_uniqueness_range_public: &[KoalaBear],
    expected_event_source_public: &[KoalaBear],
    expected_semantic_source_public: &[KoalaBear],
    expected_semantic_sha_public: &[KoalaBear],
) -> Result<u64, CheckpointError> {
    if !group.is_uniqueness() {
        return Err(CheckpointError::Canonical);
    }
    verify_group_tables(group, proof)?;
    let uniqueness_public_matches = UniquenessAirRoleV2::ALL.iter().try_fold(
        true,
        |matches, role| -> Result<bool, CheckpointError> {
            Ok(matches
                && exact_public_values(proof, role.npo_type())? == expected_uniqueness_public)
        },
    )?;
    let range_public = exact_public_values(proof, uniqueness_range::npo_type())?;
    let range_query_count = uniqueness_range::query_count_from_verified_public(
        range_public,
        expected_uniqueness_range_public,
    )?;
    if !uniqueness_public_matches
        || exact_public_values(proof, EventSourceAirRoleV2::SemanticUniqueness.npo_type())?
            != expected_event_source_public
        || exact_public_values(proof, SemanticSourceAirRoleV2::Uniqueness.npo_type())?
            != expected_semantic_source_public
        || exact_public_values(proof, ShaAirRoleV2::SemanticUniquenessChain.npo_type())?
            != expected_semantic_sha_public
    {
        return Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3AirBindingMismatch,
        ));
    }
    Ok(range_query_count)
}

fn prove_group(
    group: EpochChunkProofGroupV2,
    traces: Traces<KoalaBear>,
    resource_telemetry_enabled: bool,
) -> Result<EpochChunkGroupProofV2, CheckpointError> {
    let table_packing = TablePacking::new(1, 1).with_min_trace_height(TRACE_FRAMING_ROWS_V2);
    let prover = configured_group_prover(group, table_packing);
    let proof_result = if resource_telemetry_enabled {
        let mut telemetry = EpochOneShotResourceSinkV2 { group };
        prover.prove_direct_tables_one_shot_with_resource_telemetry(traces, &mut telemetry)
    } else {
        prover.prove_direct_tables_one_shot(traces)
    };
    let proof = proof_result.map_err(|error| {
        CheckpointError::Backend(format!(
            "Plonky3 epoch {:?} group proving failed: {error}",
            group
        ))
    })?;
    let proof_bytes = encode_internal_canonical_batch_proof_v2(&proof)?;
    let proof_digest = super::plonky3_proof_digest(&proof_bytes);
    Ok(EpochChunkGroupProofV2 {
        group,
        uniqueness: None,
        proof_digest,
        proof_bytes,
    })
}

pub(super) fn prove_epoch_chunk(
    transition_statement: EpochTraceChunkV2,
    trace_framing_statement: EpochTraceChunkV2,
    packed_statement: EpochTraceChunkV2,
    typed_statement: EpochTraceChunkV2,
    jmt_statement: EpochTraceChunkV2,
    uniqueness_statement: EpochTraceChunkV2,
    bindings: Vec<EpochTransitionBindingV2>,
    prepared: &[EpochPreparedTransitionV2],
    parsed_uniqueness: ParsedUniquenessWitnessV2,
) -> Result<Plonky3EpochChunkProofV2, CheckpointError> {
    let resource_telemetry_enabled = resource_telemetry_enabled_v2();
    validate_statements(
        &transition_statement,
        &trace_framing_statement,
        &packed_statement,
        &typed_statement,
        &jmt_statement,
        &uniqueness_statement,
        &bindings,
    )?;
    if bindings.len() != prepared.len() {
        return Err(CheckpointError::Invariant);
    }
    let transition_public = transition_witness::public_values(&transition_statement, &bindings)?;
    let event_bytes = binding_event_bytes(&bindings)?;
    let trace_framing_public =
        trace_framing::public_values(&trace_framing_statement, &bindings, event_bytes)?;
    let event_source_public = event_source_witness::public_values(&packed_statement, &bindings)?;
    let semantic_source_public =
        semantic_source_witness::public_values(&packed_statement, &bindings)?;
    let typed_public = typed_public_values(&typed_statement, &bindings)?;
    let transition_rows = transition_witness::rows(&transition_statement, &bindings)?;
    let trace_framing_rows = trace_framing::rows(&trace_framing_statement, &bindings, event_bytes)?;
    let transition_core_group_traces =
        transition_core_group_traces(transition_rows, trace_framing_rows);
    let transition_group = prove_group(
        EpochChunkProofGroupV2::Transition,
        transition_core_group_traces,
        resource_telemetry_enabled,
    )?;
    let transition_group_proof =
        decode_internal_canonical_batch_proof_v2(&transition_group.proof_bytes)?;
    verify_transition_core_group_proof(
        &transition_group_proof,
        &transition_public,
        &trace_framing_public,
    )?;
    drop(transition_group_proof);

    let transition_semantic_sha_public = sha_witness::chain_public_values_for_slice(
        &transition_statement,
        &bindings,
        EpochUniquenessSliceV2::full(bindings.len())?,
    )?;

    let typed_event_source = event_source_witness::semantic_trace(
        EventSourceAirRoleV2::SemanticTransition,
        &packed_statement,
        &bindings,
        prepared,
    )?;
    let typed_semantic_source = semantic_source_witness::witness(
        SemanticSourceAirRoleV2::TransitionTyped,
        &packed_statement,
        &bindings,
        prepared,
    )?;
    let (typed_semantic_sha, _) = sha_witness::semantic_chain_trace(
        ShaAirRoleV2::SemanticTransitionChain,
        &transition_statement,
        &bindings,
        prepared,
    )?;
    let transition_typed_group_traces = transition_typed_group_traces(
        transition_witness::rows(&transition_statement, &bindings)?,
        typed_commitment::rows(&typed_statement, &bindings, prepared)?,
        typed_event_source,
        typed_semantic_source.trace,
        typed_semantic_sha,
    );
    let transition_typed_group = prove_group(
        EpochChunkProofGroupV2::TransitionTyped,
        transition_typed_group_traces,
        resource_telemetry_enabled,
    )?;
    let transition_typed_group_proof =
        decode_internal_canonical_batch_proof_v2(&transition_typed_group.proof_bytes)?;
    verify_transition_typed_group_proof(
        &transition_typed_group_proof,
        &transition_public,
        &typed_public,
        &event_source_public,
        &semantic_source_public,
        &transition_semantic_sha_public,
    )?;
    drop(transition_typed_group_proof);

    let jmt_public = jmt_witness::chunk_public_values(&jmt_statement, &bindings)?;
    let jmt_sha_public = sha_witness::jmt_linked_public_values(&jmt_statement)?;
    let jmt_event_source = event_source_witness::semantic_trace(
        EventSourceAirRoleV2::SemanticTransition,
        &packed_statement,
        &bindings,
        prepared,
    )?;
    let jmt_semantic_source = semantic_source_witness::witness(
        SemanticSourceAirRoleV2::TransitionJmt,
        &packed_statement,
        &bindings,
        prepared,
    )?;
    let (jmt_semantic_sha, _) = sha_witness::semantic_chain_trace(
        ShaAirRoleV2::SemanticTransitionChain,
        &transition_statement,
        &bindings,
        prepared,
    )?;
    let transition_jmt_group = prove_group(
        EpochChunkProofGroupV2::TransitionJmt,
        transition_jmt_group_traces(
            jmt_witness::chunk_trace(&jmt_statement, &bindings, prepared)?,
            sha_witness::jmt_linked_trace(&jmt_statement, &bindings, prepared)?,
            jmt_event_source,
            jmt_semantic_source.trace,
            jmt_semantic_sha,
        ),
        resource_telemetry_enabled,
    )?;
    let transition_jmt_group_proof =
        decode_internal_canonical_batch_proof_v2(&transition_jmt_group.proof_bytes)?;
    verify_transition_jmt_group_proof(
        &transition_jmt_group_proof,
        &jmt_public,
        &jmt_sha_public,
        &event_source_public,
        &semantic_source_public,
        &transition_semantic_sha_public,
    )?;
    drop(transition_jmt_group_proof);

    let flow_event_source = event_source_witness::semantic_trace(
        EventSourceAirRoleV2::SemanticTransition,
        &packed_statement,
        &bindings,
        prepared,
    )?;
    let flow_semantic_source = semantic_source_witness::witness(
        SemanticSourceAirRoleV2::TransitionFlow,
        &packed_statement,
        &bindings,
        prepared,
    )?;
    let (flow_semantic_sha, _) = sha_witness::semantic_chain_trace(
        ShaAirRoleV2::SemanticTransitionChain,
        &transition_statement,
        &bindings,
        prepared,
    )?;
    let transition_flow_group = prove_group(
        EpochChunkProofGroupV2::TransitionFlow,
        transition_flow_group_traces(
            transition_witness::rows(&transition_statement, &bindings)?,
            flow_event_source,
            flow_semantic_source.trace,
            flow_semantic_sha,
        ),
        resource_telemetry_enabled,
    )?;
    let transition_flow_group_proof =
        decode_internal_canonical_batch_proof_v2(&transition_flow_group.proof_bytes)?;
    verify_transition_flow_group_proof(
        &transition_flow_group_proof,
        &transition_public,
        &event_source_public,
        &semantic_source_public,
        &transition_semantic_sha_public,
    )?;
    drop(transition_flow_group_proof);

    let (sha_trace, _sha_block_count) =
        sha_witness::chain_trace(&transition_statement, &bindings, prepared)?;
    let sha_public = sha_trace.public_values.clone();
    let event_source = event_source_witness::trace(&packed_statement, &bindings, prepared)?;
    let hash_group_traces = hash_group_traces(event_source, sha_trace);
    let hash_group = prove_group(
        EpochChunkProofGroupV2::Hash,
        hash_group_traces,
        resource_telemetry_enabled,
    )?;
    let hash_group_proof = decode_internal_canonical_batch_proof_v2(&hash_group.proof_bytes)?;
    verify_hash_group_proof(&hash_group_proof, &event_source_public, &sha_public)?;
    drop(hash_group_proof);

    let mut uniqueness_groups = Vec::new();
    let mut uniqueness_range_query_count = 0_u64;
    for (slice_index, slice) in EpochUniquenessSliceV2::canonical(bindings.len())?
        .into_iter()
        .enumerate()
    {
        let group = match slice_index {
            0 => EpochChunkProofGroupV2::UniquenessLower,
            1 => EpochChunkProofGroupV2::UniquenessUpper,
            _ => return Err(CheckpointError::Invariant),
        };
        let end = slice.end()?;
        let slice_bindings = bindings
            .get(slice.start()..end)
            .ok_or(CheckpointError::Canonical)?;
        let semantic_row_count = slice_bindings.iter().try_fold(0_u64, |total, binding| {
            total
                .checked_add(binding.inputs().uniqueness_row_count)
                .ok_or(CheckpointError::Overflow)
        })?;
        // This scope is deliberately one slice-wide. Each direct proof owns a
        // fresh Batch-STARK prover and all large witness buffers die before the
        // next slice begins.
        let mut uniqueness = uniqueness_witness::air_witness_for_slice(
            &uniqueness_statement,
            &parsed_uniqueness,
            slice,
        )?;
        let uniqueness_semantic_source = semantic_source_witness::witness_for_slice(
            SemanticSourceAirRoleV2::Uniqueness,
            &packed_statement,
            &bindings,
            prepared,
            slice,
        )?;
        uniqueness
            .range_queries
            .extend(uniqueness_semantic_source.range_queries.iter().copied());
        let uniqueness_range_rows =
            uniqueness_range::rows(&uniqueness_statement, &uniqueness.range_queries, slice)?;
        let uniqueness_event_source = event_source_witness::semantic_trace_for_slice(
            EventSourceAirRoleV2::SemanticUniqueness,
            &packed_statement,
            &bindings,
            prepared,
            slice,
        )?;
        let (uniqueness_semantic_sha, _) = sha_witness::semantic_chain_trace_for_slice(
            ShaAirRoleV2::SemanticUniquenessChain,
            &transition_statement,
            &bindings,
            prepared,
            slice,
        )?;
        let uniqueness_group_traces = uniqueness_group_traces(
            uniqueness,
            uniqueness_range_rows,
            uniqueness_event_source,
            uniqueness_semantic_source.trace,
            uniqueness_semantic_sha,
        );
        let mut uniqueness_group =
            prove_group(group, uniqueness_group_traces, resource_telemetry_enabled)?;
        let uniqueness_group_proof =
            decode_internal_canonical_batch_proof_v2(&uniqueness_group.proof_bytes)?;
        let verified_range_count = verify_uniqueness_group_proof(
            group,
            &uniqueness_group_proof,
            &uniqueness_witness::public_values_for_slice(
                &uniqueness_statement,
                slice,
                semantic_row_count,
            )?,
            &uniqueness_range::public_prefix_for_slice(&uniqueness_statement, slice)?,
            &event_source_witness::public_values_for_slice(&packed_statement, &bindings, slice)?,
            &semantic_source_witness::public_values_for_slice(&packed_statement, &bindings, slice)?,
            &sha_witness::chain_public_values_for_slice(&transition_statement, &bindings, slice)?,
        )?;
        drop(uniqueness_group_proof);
        uniqueness_range_query_count = uniqueness_range_query_count
            .checked_add(verified_range_count)
            .ok_or(CheckpointError::Overflow)?;
        uniqueness_group.uniqueness = Some(UniquenessGroupMetadataV2 { slice });
        uniqueness_groups.push(uniqueness_group);
    }

    let mut artifact = Plonky3EpochChunkProofV2 {
        transition_statement,
        trace_framing_statement,
        packed_statement,
        typed_statement,
        jmt_statement,
        uniqueness_statement,
        uniqueness_range_query_count,
        bindings,
        group_proofs: {
            let mut groups = vec![
                transition_group,
                transition_typed_group,
                transition_jmt_group,
                transition_flow_group,
                hash_group,
            ];
            groups.extend(uniqueness_groups);
            groups
        },
        canonical_bytes: Vec::new(),
    };
    artifact.canonical_bytes = encode_chunk_proof(&artifact)?;
    artifact.verify()?;
    Ok(artifact)
}

const _: () = assert!(TRANSITION_ROWS_V2 <= TYPED_ROWS_V2);
const _: () = assert!(TRACE_FRAMING_ROWS_V2 <= TRANSITION_ROWS_V2);
