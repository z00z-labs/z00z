use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use z00z_crypto::{expert::hash_domain, frame_bytes, hash_zk::hash_zk};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::proof::{
    chk_blob_settlement, hjmt_default_child_commitment, hjmt_default_value_commitment,
    HjmtProofFamily, ProofBlob, ProofChkErr, HJMT_DEFAULT_COMMITMENT_VERSION,
};
use super::{
    BucketPolicy, RootGeneration, SettlementLeaf, SettlementLeafFamily, SettlementPath,
    SettlementStateRoot,
};

hash_domain!(StorBatchProofDom, "z00z.storage.batch.proof", 1);
hash_domain!(StorProofBindDom, "z00z.storage.proof.bind", 1);
hash_domain!(StorPolicySetDom, "z00z.hjmt.policy-set.v1", 1);
hash_domain!(StorShardRootLeafDom, "z00z.hjmt.shard-root-leaf.v1", 1);
hash_domain!(
    StorCheckpointPublicationDom,
    "z00z.hjmt.checkpoint-publication.v1",
    1
);

pub const BATCH_PROOF_ENCODING_VERSION: u8 = 1;
const OPENING_VERSION_V1: u8 = 1;
const PRIOR_CTX_VERSION_V1: u8 = 1;
const DELETION_FACT_VERSION_V1: u8 = 1;
const ROOT_BIND_VER: u8 = 1;
const WITNESS_CHUNK_LABEL: &str = "checkpoint_witness_chunk_v1";
const WITNESS_PAYLOAD_LABEL: &str = "checkpoint_witness_payload_v1";
const WITNESS_ROOT_LABEL: &str = "checkpoint_witness_root_v1";
const WITNESS_CHUNK_VER: u8 = 1;
const WITNESS_CHUNK_BATCH: u8 = 1;
static BATCH_PROOF_TRANSCRIPT_DOMAIN: OnceLock<[u8; 32]> = OnceLock::new();

#[must_use]
pub fn batch_proof_transcript_domain_v1() -> [u8; 32] {
    *BATCH_PROOF_TRANSCRIPT_DOMAIN.get_or_init(|| {
        hash_zk::<StorBatchProofDom>(
            "proof_batch_transcript_domain_v1",
            &[&[BATCH_PROOF_ENCODING_VERSION]],
        )
    })
}

#[must_use]
pub(crate) fn batch_proof_accept_transcript_v1(bytes: &[u8]) -> [u8; 32] {
    hash_zk::<StorBatchProofDom>("proof_batch_accept_transcript_v1", &[bytes])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum BatchProofFamilyTagV1 {
    Inclusion = 0x01,
    Deletion = 0x02,
    NonExistence = 0x03,
}

impl BatchProofFamilyTagV1 {
    #[must_use]
    pub const fn from_live(family: HjmtProofFamily) -> Self {
        match family {
            HjmtProofFamily::Inclusion => Self::Inclusion,
            HjmtProofFamily::Deletion => Self::Deletion,
            HjmtProofFamily::NonExistence => Self::NonExistence,
        }
    }

    #[must_use]
    pub const fn into_live(self) -> HjmtProofFamily {
        match self {
            Self::Inclusion => HjmtProofFamily::Inclusion,
            Self::Deletion => HjmtProofFamily::Deletion,
            Self::NonExistence => HjmtProofFamily::NonExistence,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Inclusion),
            0x02 => Ok(Self::Deletion),
            0x03 => Ok(Self::NonExistence),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum TerminalFamilyTagV1 {
    Asset = 0x01,
    Right = 0x02,
    Voucher = 0x03,
}

impl TerminalFamilyTagV1 {
    #[must_use]
    pub const fn from_live(family: SettlementLeafFamily) -> Self {
        match family {
            SettlementLeafFamily::Terminal => Self::Asset,
            SettlementLeafFamily::Right => Self::Right,
            SettlementLeafFamily::Voucher => Self::Voucher,
        }
    }

    #[must_use]
    pub const fn into_live(self) -> SettlementLeafFamily {
        match self {
            Self::Asset => SettlementLeafFamily::Terminal,
            Self::Right => SettlementLeafFamily::Right,
            Self::Voucher => SettlementLeafFamily::Voucher,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Asset),
            0x02 => Ok(Self::Right),
            0x03 => Ok(Self::Voucher),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum LeafFamilyTagV1 {
    Asset = 0x01,
    Right = 0x02,
    Voucher = 0x03,
}

impl LeafFamilyTagV1 {
    #[must_use]
    pub const fn from_live(family: SettlementLeafFamily) -> Self {
        match family {
            SettlementLeafFamily::Terminal => Self::Asset,
            SettlementLeafFamily::Right => Self::Right,
            SettlementLeafFamily::Voucher => Self::Voucher,
        }
    }

    #[must_use]
    pub const fn into_live(self) -> SettlementLeafFamily {
        match self {
            Self::Asset => SettlementLeafFamily::Terminal,
            Self::Right => SettlementLeafFamily::Right,
            Self::Voucher => SettlementLeafFamily::Voucher,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Asset),
            0x02 => Ok(Self::Right),
            0x03 => Ok(Self::Voucher),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum RootGenerationTagV1 {
    RootGeneration0 = 0x00,
    RootGeneration1 = 0x01,
}

impl RootGenerationTagV1 {
    #[must_use]
    pub const fn from_live(root: SettlementStateRoot) -> Self {
        match root.generation() {
            RootGeneration::SettlementV1 => Self::RootGeneration1,
        }
    }

    #[must_use]
    pub const fn into_version(self) -> u8 {
        match self {
            Self::RootGeneration0 => 0,
            Self::RootGeneration1 => 1,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x00 => Ok(Self::RootGeneration0),
            0x01 => Ok(Self::RootGeneration1),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeDomainTagV1 {
    Terminal = 0x01,
    Bucket = 0x02,
    Serial = 0x03,
    Definition = 0x04,
    Shard = 0x05,
    Global = 0x06,
}

impl NodeDomainTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Terminal),
            0x02 => Ok(Self::Bucket),
            0x03 => Ok(Self::Serial),
            0x04 => Ok(Self::Definition),
            0x05 => Ok(Self::Shard),
            0x06 => Ok(Self::Global),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum SiblingSideTagV1 {
    LeftSibling = 0x00,
    RightSibling = 0x01,
}

impl SiblingSideTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x00 => Ok(Self::LeftSibling),
            0x01 => Ok(Self::RightSibling),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpeningKindTagV1 {
    InclusionLeaf = 0x01,
    DeletionFact = 0x02,
    AbsenceOpening = 0x03,
}

impl OpeningKindTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::InclusionLeaf),
            0x02 => Ok(Self::DeletionFact),
            0x03 => Ok(Self::AbsenceOpening),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProofLimits {
    pub max_path_count: u32,
    pub max_witness_node_count: u32,
    pub max_opening_count: u32,
    pub max_reference_count: u32,
    pub max_total_bytes: u32,
}

impl BatchProofLimits {
    #[must_use]
    pub const fn v1() -> Self {
        Self {
            max_path_count: 1_024,
            max_witness_node_count: 16_384,
            max_opening_count: 1_024,
            max_reference_count: 1_024,
            max_total_bytes: 8 * 1024 * 1024,
        }
    }

    #[must_use]
    pub const fn from_policy(policy: BucketPolicy) -> Self {
        let base = Self::v1();
        let _ = policy;
        Self {
            max_path_count: base.max_path_count,
            ..base
        }
    }

    #[must_use]
    pub(crate) const fn contains(self, other: Self) -> bool {
        other.max_path_count <= self.max_path_count
            && other.max_witness_node_count <= self.max_witness_node_count
            && other.max_opening_count <= self.max_opening_count
            && other.max_reference_count <= self.max_reference_count
            && other.max_total_bytes <= self.max_total_bytes
    }

    pub(crate) fn encode(self, out: &mut Vec<u8>) {
        put_u32(out, self.max_path_count);
        put_u32(out, self.max_witness_node_count);
        put_u32(out, self.max_opening_count);
        put_u32(out, self.max_reference_count);
        put_u32(out, self.max_total_bytes);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        Ok(Self {
            max_path_count: take_u32(bytes, pos)?,
            max_witness_node_count: take_u32(bytes, pos)?,
            max_opening_count: take_u32(bytes, pos)?,
            max_reference_count: take_u32(bytes, pos)?,
            max_total_bytes: take_u32(bytes, pos)?,
        })
    }
}

impl Default for BatchProofLimits {
    fn default() -> Self {
        Self::v1()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum PublicationModeTagV1 {
    Synchronous = 0x00,
    CheckpointWindow = 0x01,
    EmergencyFreeze = 0x02,
}

impl PublicationModeTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x00 => Ok(Self::Synchronous),
            0x01 => Ok(Self::CheckpointWindow),
            0x02 => Ok(Self::EmergencyFreeze),
            _ => Err(ProofChkErr::PublicationModeMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySetMemberV1 {
    pub policy_generation: u64,
    pub bucket_policy_digest: [u8; 32],
    pub activation_checkpoint: u64,
    pub retirement_checkpoint: Option<u64>,
}

impl PolicySetMemberV1 {
    #[must_use]
    pub const fn new(
        policy_generation: u64,
        bucket_policy_digest: [u8; 32],
        activation_checkpoint: u64,
        retirement_checkpoint: Option<u64>,
    ) -> Self {
        Self {
            policy_generation,
            bucket_policy_digest,
            activation_checkpoint,
            retirement_checkpoint,
        }
    }

    #[must_use]
    pub const fn policy_digest(self) -> [u8; 32] {
        self.bucket_policy_digest
    }

    #[must_use]
    pub const fn is_active_at(self, checkpoint: u64) -> bool {
        self.activation_checkpoint <= checkpoint
            && match self.retirement_checkpoint {
                Some(retire_at) => checkpoint < retire_at,
                None => true,
            }
    }

    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_u64(out, self.policy_generation);
        out.extend_from_slice(&self.bucket_policy_digest);
        put_u64(out, self.activation_checkpoint);
        put_opt_u64(out, self.retirement_checkpoint);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        Ok(Self {
            policy_generation: take_pub_u64(bytes, pos)?,
            bucket_policy_digest: take_pub_32(bytes, pos)?,
            activation_checkpoint: take_pub_u64(bytes, pos)?,
            retirement_checkpoint: take_pub_opt_u64(bytes, pos)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySetCommitmentV1 {
    pub members: Vec<PolicySetMemberV1>,
}

impl PolicySetCommitmentV1 {
    #[must_use]
    pub fn new(members: Vec<PolicySetMemberV1>) -> Self {
        Self { members }
    }

    #[must_use]
    pub fn singleton_live(
        policy_generation: u64,
        bucket_policy_digest: [u8; 32],
        activation_checkpoint: u64,
    ) -> Self {
        Self::new(vec![PolicySetMemberV1::new(
            policy_generation,
            bucket_policy_digest,
            activation_checkpoint,
            None,
        )])
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.members.is_empty() {
            return Err(ProofChkErr::PublicationPolicyMix);
        }
        let mut prev: Option<PolicySetMemberV1> = None;
        for member in &self.members {
            if member
                .retirement_checkpoint
                .is_some_and(|retire_at| retire_at <= member.activation_checkpoint)
            {
                return Err(ProofChkErr::PublicationPolicyMix);
            }
            if let Some(last) = prev {
                let last_key = (
                    last.policy_generation,
                    last.bucket_policy_digest,
                    last.activation_checkpoint,
                );
                let next_key = (
                    member.policy_generation,
                    member.bucket_policy_digest,
                    member.activation_checkpoint,
                );
                if next_key <= last_key {
                    return Err(ProofChkErr::PublicationPolicyMix);
                }
                if member.policy_generation == last.policy_generation
                    && member.bucket_policy_digest == last.bucket_policy_digest
                    && member.activation_checkpoint < last.retirement_checkpoint.unwrap_or(u64::MAX)
                {
                    return Err(ProofChkErr::PublicationPolicyMix);
                }
            }
            prev = Some(*member);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        self.check_contract_v1()?;
        let mut out = Vec::new();
        for member in &self.members {
            member.encode(&mut out);
        }
        Ok(out)
    }

    pub fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0usize;
        let mut members = Vec::new();
        while pos < bytes.len() {
            members.push(PolicySetMemberV1::decode(bytes, &mut pos)?);
        }
        let out = Self { members };
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn digest(&self) -> Result<[u8; 32], ProofChkErr> {
        Ok(hash_zk::<StorPolicySetDom>("", &[&self.canonical_bytes()?]))
    }

    pub fn verify_member(
        &self,
        policy_generation: u64,
        policy_digest: [u8; 32],
        proof_checkpoint: u64,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        let found = self.members.iter().any(|member| {
            member.policy_generation == policy_generation
                && member.policy_digest() == policy_digest
                && member.is_active_at(proof_checkpoint)
        });
        if !found {
            return Err(ProofChkErr::PublicationPolicyMix);
        }
        Ok(())
    }
}

// Storage owns the committed publication route snapshot. Wallet-facing code
// verifies this public proof surface and must not read raw backend or journal
// internals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationRouteSnapshotV1 {
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub activation_checkpoint: u64,
    pub shard_ids: Vec<u32>,
}

impl PublicationRouteSnapshotV1 {
    #[must_use]
    pub fn new(
        routing_generation: u64,
        route_table_digest: [u8; 32],
        activation_checkpoint: u64,
        shard_ids: Vec<u32>,
    ) -> Self {
        Self {
            routing_generation,
            route_table_digest,
            activation_checkpoint,
            shard_ids,
        }
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.shard_ids.is_empty() {
            return Err(ProofChkErr::PublicationCountMix);
        }
        if self.shard_ids.len() > u32::MAX as usize {
            return Err(ProofChkErr::PublicationCountMix);
        }
        let mut prev = None;
        for shard_id in &self.shard_ids {
            if let Some(last) = prev {
                if *shard_id <= last {
                    return if *shard_id == last {
                        Err(ProofChkErr::PublicationDupShard)
                    } else {
                        Err(ProofChkErr::PublicationOrderMix)
                    };
                }
            }
            prev = Some(*shard_id);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationHandoffRowV1 {
    pub shard_id: u32,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub checkpoint_id: [u8; 32],
}

impl PublicationHandoffRowV1 {
    #[must_use]
    pub const fn new(
        shard_id: u32,
        routing_generation: u64,
        route_table_digest: [u8; 32],
        checkpoint_id: [u8; 32],
    ) -> Self {
        Self {
            shard_id,
            routing_generation,
            route_table_digest,
            checkpoint_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardRootLeafV1 {
    pub shard_id: u32,
    pub shard_root: [u8; 32],
    pub shard_epoch: u64,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub policy_set_digest: [u8; 32],
    pub journal_checkpoint: u64,
    pub local_sequence: u64,
    pub transition_flags: u32,
}

impl ShardRootLeafV1 {
    #[must_use]
    pub const fn new(
        shard_id: u32,
        shard_root: [u8; 32],
        shard_epoch: u64,
        routing_generation: u64,
        route_table_digest: [u8; 32],
        policy_set_digest: [u8; 32],
        journal_checkpoint: u64,
        local_sequence: u64,
        transition_flags: u32,
    ) -> Self {
        Self {
            shard_id,
            shard_root,
            shard_epoch,
            routing_generation,
            route_table_digest,
            policy_set_digest,
            journal_checkpoint,
            local_sequence,
            transition_flags,
        }
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.transition_flags & !0b111 != 0 {
            return Err(ProofChkErr::PublicationFlagMix);
        }
        Ok(())
    }

    pub fn check_route_binding_v1(
        &self,
        expected_route_table_digest: [u8; 32],
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.route_table_digest != expected_route_table_digest {
            return Err(ProofChkErr::PublicationRouteMix);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        self.check_contract_v1()?;
        let mut out = Vec::with_capacity(136);
        put_u32(&mut out, self.shard_id);
        out.extend_from_slice(&self.shard_root);
        put_u64(&mut out, self.shard_epoch);
        put_u64(&mut out, self.routing_generation);
        out.extend_from_slice(&self.route_table_digest);
        out.extend_from_slice(&self.policy_set_digest);
        put_u64(&mut out, self.journal_checkpoint);
        put_u64(&mut out, self.local_sequence);
        put_u32(&mut out, self.transition_flags);
        Ok(out)
    }

    pub fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0usize;
        let out = Self {
            shard_id: take_pub_u32(bytes, &mut pos)?,
            shard_root: take_pub_32(bytes, &mut pos)?,
            shard_epoch: take_pub_u64(bytes, &mut pos)?,
            routing_generation: take_pub_u64(bytes, &mut pos)?,
            route_table_digest: take_pub_32(bytes, &mut pos)?,
            policy_set_digest: take_pub_32(bytes, &mut pos)?,
            journal_checkpoint: take_pub_u64(bytes, &mut pos)?,
            local_sequence: take_pub_u64(bytes, &mut pos)?,
            transition_flags: take_pub_u32(bytes, &mut pos)?,
        };
        if pos != bytes.len() {
            return Err(ProofChkErr::PublicationTrailingBytes);
        }
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn digest(&self) -> Result<[u8; 32], ProofChkErr> {
        Ok(hash_zk::<StorShardRootLeafDom>(
            "",
            &[&self.canonical_bytes()?],
        ))
    }

    pub fn verify_policy_member(
        &self,
        policy_set: &PolicySetCommitmentV1,
        policy_generation: u64,
        policy_digest: [u8; 32],
        proof_checkpoint: u64,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.policy_set_digest != policy_set.digest()? {
            return Err(ProofChkErr::PublicationPolicyMix);
        }
        policy_set.verify_member(policy_generation, policy_digest, proof_checkpoint)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointPublicationV1 {
    pub root_generation: RootGenerationTagV1,
    pub publication_mode: PublicationModeTagV1,
    pub publication_checkpoint: u64,
    pub route_table_digest: [u8; 32],
    pub prior_public_root: SettlementStateRoot,
    pub shard_leaves: Vec<ShardRootLeafV1>,
}

impl CheckpointPublicationV1 {
    #[must_use]
    pub fn new(
        root_generation: RootGenerationTagV1,
        publication_mode: PublicationModeTagV1,
        publication_checkpoint: u64,
        route_table_digest: [u8; 32],
        prior_public_root: SettlementStateRoot,
        shard_leaves: Vec<ShardRootLeafV1>,
    ) -> Self {
        Self {
            root_generation,
            publication_mode,
            publication_checkpoint,
            route_table_digest,
            prior_public_root,
            shard_leaves,
        }
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.root_generation != RootGenerationTagV1::RootGeneration1 {
            return Err(ProofChkErr::PublicationRootGenerationMix);
        }
        if self.shard_leaves.is_empty() {
            return Err(ProofChkErr::PublicationCountMix);
        }
        if self.shard_leaves.len() > u32::MAX as usize {
            return Err(ProofChkErr::PublicationCountMix);
        }
        let mut prev = None;
        for leaf in &self.shard_leaves {
            leaf.check_contract_v1()?;
            if leaf.route_table_digest != self.route_table_digest {
                return Err(ProofChkErr::PublicationRouteMix);
            }
            if let Some(last) = prev {
                if leaf.shard_id <= last {
                    return if leaf.shard_id == last {
                        Err(ProofChkErr::PublicationDupShard)
                    } else {
                        Err(ProofChkErr::PublicationOrderMix)
                    };
                }
            }
            prev = Some(leaf.shard_id);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        self.check_contract_v1()?;
        let mut out = Vec::new();
        out.push(self.root_generation as u8);
        out.push(self.publication_mode as u8);
        put_u64(&mut out, self.publication_checkpoint);
        out.extend_from_slice(&self.route_table_digest);
        put_state_root(&mut out, self.prior_public_root);
        put_u32(&mut out, self.shard_leaves.len() as u32);
        for leaf in &self.shard_leaves {
            out.extend_from_slice(&leaf.canonical_bytes()?);
        }
        Ok(out)
    }

    pub fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0usize;
        let root_generation = RootGenerationTagV1::decode(take_pub_u8(bytes, &mut pos)?)?;
        let publication_mode = PublicationModeTagV1::decode(take_pub_u8(bytes, &mut pos)?)?;
        let publication_checkpoint = take_pub_u64(bytes, &mut pos)?;
        let route_table_digest = take_pub_32(bytes, &mut pos)?;
        let prior_public_root = take_pub_state_root(bytes, &mut pos)?;
        let leaf_count = take_pub_u32(bytes, &mut pos)? as usize;
        let mut shard_leaves = Vec::with_capacity(leaf_count);
        for _ in 0..leaf_count {
            let start = pos;
            pos = pos.saturating_add(136);
            if pos > bytes.len() {
                return Err(ProofChkErr::PublicationTrunc);
            }
            shard_leaves.push(ShardRootLeafV1::from_canon(&bytes[start..pos])?);
        }
        if pos != bytes.len() {
            return Err(ProofChkErr::PublicationTrailingBytes);
        }
        let out = Self {
            root_generation,
            publication_mode,
            publication_checkpoint,
            route_table_digest,
            prior_public_root,
            shard_leaves,
        };
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn digest(&self) -> Result<[u8; 32], ProofChkErr> {
        Ok(hash_zk::<StorCheckpointPublicationDom>(
            "",
            &[&self.canonical_bytes()?],
        ))
    }

    pub fn public_root_v1(&self) -> Result<SettlementStateRoot, ProofChkErr> {
        Ok(SettlementStateRoot::settlement_v1(self.digest()?))
    }

    pub fn check_prior_root_v1(
        &self,
        expected_prior_root: SettlementStateRoot,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.prior_public_root != expected_prior_root {
            return Err(ProofChkErr::PublicationPriorRootMix);
        }
        Ok(())
    }

    pub fn check_monotonic_successor_v1(
        &self,
        prev: &CheckpointPublicationV1,
    ) -> Result<(), ProofChkErr> {
        prev.check_contract_v1()?;
        self.check_contract_v1()?;
        if self.publication_checkpoint <= prev.publication_checkpoint {
            return Err(ProofChkErr::PublicationCheckpointMix);
        }
        let same_route = self.route_table_digest == prev.route_table_digest;
        if same_route && self.shard_leaves.len() != prev.shard_leaves.len() {
            return Err(ProofChkErr::PublicationCountMix);
        }

        let mut last_by_shard = std::collections::BTreeMap::new();
        for leaf in &prev.shard_leaves {
            last_by_shard.insert(leaf.shard_id, *leaf);
        }
        for leaf in &self.shard_leaves {
            let Some(prev_leaf) = last_by_shard.get(&leaf.shard_id).copied() else {
                if same_route {
                    return Err(ProofChkErr::PublicationCountMix);
                }
                continue;
            };
            if leaf.routing_generation < prev_leaf.routing_generation {
                return Err(ProofChkErr::PublicationMonotonicityMix);
            }
            if leaf.journal_checkpoint < prev_leaf.journal_checkpoint {
                return Err(ProofChkErr::PublicationMonotonicityMix);
            }
            if leaf.routing_generation == prev_leaf.routing_generation {
                let prev_key = (
                    prev_leaf.shard_epoch,
                    prev_leaf.local_sequence,
                    prev_leaf.journal_checkpoint,
                );
                let next_key = (
                    leaf.shard_epoch,
                    leaf.local_sequence,
                    leaf.journal_checkpoint,
                );
                if next_key < prev_key {
                    return Err(ProofChkErr::PublicationMonotonicityMix);
                }
                if leaf != &prev_leaf && next_key <= prev_key {
                    return Err(ProofChkErr::PublicationMonotonicityMix);
                }
            }
        }
        if same_route {
            for prev_leaf in &prev.shard_leaves {
                if !self
                    .shard_leaves
                    .iter()
                    .any(|leaf| leaf.shard_id == prev_leaf.shard_id)
                {
                    return Err(ProofChkErr::PublicationCountMix);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardProofContextV1 {
    pub shard_id: u32,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub policy_generation: u64,
    pub bucket_policy_digest: [u8; 32],
    pub proof_family: HjmtProofFamily,
    pub leaf_family: SettlementLeafFamily,
}

impl ShardProofContextV1 {
    #[must_use]
    pub const fn new(
        shard_id: u32,
        routing_generation: u64,
        route_table_digest: [u8; 32],
        policy_generation: u64,
        bucket_policy_digest: [u8; 32],
        proof_family: HjmtProofFamily,
        leaf_family: SettlementLeafFamily,
    ) -> Self {
        Self {
            shard_id,
            routing_generation,
            route_table_digest,
            policy_generation,
            bucket_policy_digest,
            proof_family,
            leaf_family,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointPublicationProofV1 {
    pub root_generation: RootGenerationTagV1,
    pub public_root: SettlementStateRoot,
    pub publication: CheckpointPublicationV1,
    pub shard_leaf_index: u32,
    pub shard_context: ShardProofContextV1,
    pub policy_set: PolicySetCommitmentV1,
    pub shard_proof: ProofBlob,
}

impl CheckpointPublicationProofV1 {
    #[must_use]
    pub fn new(
        root_generation: RootGenerationTagV1,
        public_root: SettlementStateRoot,
        publication: CheckpointPublicationV1,
        shard_leaf_index: u32,
        shard_context: ShardProofContextV1,
        policy_set: PolicySetCommitmentV1,
        shard_proof: ProofBlob,
    ) -> Self {
        Self {
            root_generation,
            public_root,
            publication,
            shard_leaf_index,
            shard_context,
            policy_set,
            shard_proof,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProofChkErr> {
        Ok(BincodeCodec.serialize(self)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let out: Self = BincodeCodec.deserialize(bytes)?;
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn shard_leaf_v1(&self) -> Result<ShardRootLeafV1, ProofChkErr> {
        self.publication
            .shard_leaves
            .get(self.shard_leaf_index as usize)
            .copied()
            .ok_or(ProofChkErr::PublicationProofIndexMix)
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        self.publication.check_contract_v1()?;
        self.policy_set.check_contract_v1()?;
        if self.root_generation != RootGenerationTagV1::RootGeneration1
            || self.root_generation != self.publication.root_generation
            || self.public_root.generation_version() != self.root_generation.into_version()
        {
            return Err(ProofChkErr::PublicationProofGenerationMix);
        }
        if self.publication.public_root_v1()? != self.public_root {
            return Err(ProofChkErr::RootMix);
        }

        let shard_leaf = self.shard_leaf_v1()?;
        if shard_leaf.shard_id != self.shard_context.shard_id {
            return Err(ProofChkErr::PublicationProofShardMix);
        }
        if shard_leaf.routing_generation != self.shard_context.routing_generation
            || shard_leaf.route_table_digest != self.shard_context.route_table_digest
            || self.publication.route_table_digest != self.shard_context.route_table_digest
        {
            return Err(ProofChkErr::PublicationProofRouteMix);
        }
        if shard_leaf.policy_set_digest != self.policy_set.digest()? {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        }

        let proof_item = self.shard_proof.item();
        if proof_item.settlement_root().into_bytes() != shard_leaf.shard_root {
            return Err(ProofChkErr::RootMix);
        }
        if self.shard_proof.hjmt_proof_family() != Some(self.shard_context.proof_family) {
            return Err(ProofChkErr::ProofFamilyMix);
        }
        if self.shard_proof.hjmt_leaf_family() != Some(self.shard_context.leaf_family) {
            return Err(ProofChkErr::LeafMix);
        }

        let Some(journal_checkpoint) = self.shard_proof.hjmt_journal_checkpoint() else {
            return Err(ProofChkErr::PublicationProofCheckpointMix);
        };
        if journal_checkpoint != shard_leaf.journal_checkpoint {
            return Err(ProofChkErr::PublicationProofCheckpointMix);
        }

        let Some(bucket_policy) = self.shard_proof.hjmt_bucket_policy() else {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        };
        if u64::from(bucket_policy.compatibility_generation())
            != self.shard_context.policy_generation
            || bucket_policy.bucket_policy_id() != self.shard_context.bucket_policy_digest
        {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        }

        shard_leaf.verify_policy_member(
            &self.policy_set,
            self.shard_context.policy_generation,
            self.shard_context.bucket_policy_digest,
            journal_checkpoint,
        )?;
        Ok(())
    }

    pub fn check_public_root_v1(
        &self,
        expected_public_root: SettlementStateRoot,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.public_root != expected_public_root {
            return Err(ProofChkErr::RootMix);
        }
        Ok(())
    }

    pub fn verify_v1(&self) -> Result<ProofBlob, ProofChkErr> {
        self.verify_against_public_root_v1(self.public_root)
    }

    pub fn verify_against_public_root_v1(
        &self,
        expected_public_root: SettlementStateRoot,
    ) -> Result<ProofBlob, ProofChkErr> {
        self.check_public_root_v1(expected_public_root)?;
        let shard_leaf = self.shard_leaf_v1()?;
        let proof_item = self.shard_proof.item().clone();
        let checked = chk_blob_settlement(
            &self.shard_proof.encode()?,
            SettlementStateRoot::settlement_v1(shard_leaf.shard_root),
            &proof_item.path(),
            proof_item.def_leaf(),
            proof_item.ser_leaf(),
            proof_item.leaf().clone(),
        )?;

        if checked.hjmt_journal_checkpoint() != Some(shard_leaf.journal_checkpoint) {
            return Err(ProofChkErr::PublicationProofCheckpointMix);
        }
        let Some(bucket_policy) = checked.hjmt_bucket_policy() else {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        };
        if u64::from(bucket_policy.compatibility_generation())
            != self.shard_context.policy_generation
            || bucket_policy.bucket_policy_id() != self.shard_context.bucket_policy_digest
        {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        }
        Ok(checked)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProofHeaderV1 {
    pub encoding_version: u8,
    pub transcript_domain: [u8; 32],
    pub proof_family: BatchProofFamilyTagV1,
    pub root_generation: RootGenerationTagV1,
    pub settlement_root: SettlementStateRoot,
    pub backend_root: [u8; 32],
    pub root_bind_version: u8,
    pub root_bind: [u8; 32],
    pub leaf_family_set: Vec<LeafFamilyTagV1>,
    pub bucket_bits: u8,
    pub bucket_id_width: u8,
    pub min_bucket_count: u32,
    pub max_target_leaf_count: u32,
    pub policy_generation: u64,
    pub bucket_policy_digest: [u8; 32],
    pub journal_checkpoint: Option<u64>,
    pub journal_digest: [u8; 32],
    pub checkpoint_bind: [u8; 32],
    pub batch_limits: BatchProofLimits,
}

impl BatchProofHeaderV1 {
    #[must_use]
    pub fn new(
        proof_family: BatchProofFamilyTagV1,
        settlement_root: SettlementStateRoot,
        backend_root: [u8; 32],
        leaf_family_set: Vec<LeafFamilyTagV1>,
        policy: BucketPolicy,
        journal_checkpoint: Option<u64>,
        journal_digest: [u8; 32],
        batch_limits: BatchProofLimits,
    ) -> Self {
        let checkpoint_bind = journal_checkpoint
            .map(|checkpoint| {
                checkpoint_bind_v1(settlement_root, backend_root, checkpoint, journal_digest)
            })
            .unwrap_or([0u8; 32]);
        Self {
            encoding_version: BATCH_PROOF_ENCODING_VERSION,
            transcript_domain: batch_proof_transcript_domain_v1(),
            proof_family,
            root_generation: RootGenerationTagV1::from_live(settlement_root),
            settlement_root,
            backend_root,
            root_bind_version: ROOT_BIND_VER,
            root_bind: root_bind_v1(settlement_root, backend_root),
            leaf_family_set,
            bucket_bits: policy.bucket_bits(),
            bucket_id_width: policy.bucket_id_width(),
            min_bucket_count: policy.min_bucket_count(),
            max_target_leaf_count: policy.max_target_leaf_count(),
            policy_generation: u64::from(policy.compatibility_generation()),
            bucket_policy_digest: policy.bucket_policy_id(),
            journal_checkpoint,
            journal_digest,
            checkpoint_bind,
            batch_limits,
        }
    }

    #[must_use]
    pub fn from_policy(
        proof_family: BatchProofFamilyTagV1,
        settlement_root: SettlementStateRoot,
        backend_root: [u8; 32],
        leaf_family_set: Vec<LeafFamilyTagV1>,
        policy: BucketPolicy,
        journal_checkpoint: Option<u64>,
        journal_digest: [u8; 32],
    ) -> Self {
        Self::new(
            proof_family,
            settlement_root,
            backend_root,
            leaf_family_set,
            policy,
            journal_checkpoint,
            journal_digest,
            BatchProofLimits::from_policy(policy),
        )
    }

    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.encoding_version);
        out.extend_from_slice(&self.transcript_domain);
        out.push(self.proof_family as u8);
        out.push(self.root_generation as u8);
        put_state_root(out, self.settlement_root);
        out.extend_from_slice(&self.backend_root);
        out.push(self.root_bind_version);
        out.extend_from_slice(&self.root_bind);
        put_u32(out, self.leaf_family_set.len() as u32);
        for family in &self.leaf_family_set {
            out.push(*family as u8);
        }
        out.push(self.bucket_bits);
        out.push(self.bucket_id_width);
        put_u32(out, self.min_bucket_count);
        put_u32(out, self.max_target_leaf_count);
        put_u64(out, self.policy_generation);
        out.extend_from_slice(&self.bucket_policy_digest);
        put_opt_u64(out, self.journal_checkpoint);
        out.extend_from_slice(&self.journal_digest);
        out.extend_from_slice(&self.checkpoint_bind);
        self.batch_limits.encode(out);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        let encoding_version = take_u8(bytes, pos)?;
        let transcript_domain = take_32(bytes, pos)?;
        let proof_family = BatchProofFamilyTagV1::decode(take_u8(bytes, pos)?)?;
        let root_generation = RootGenerationTagV1::decode(take_u8(bytes, pos)?)?;
        let settlement_root = take_state_root(bytes, pos)?;
        let backend_root = take_32(bytes, pos)?;
        let root_bind_version = take_u8(bytes, pos)?;
        let root_bind = take_32(bytes, pos)?;
        let leaf_count = take_count(bytes, pos, 2)?;
        let mut leaf_family_set = Vec::with_capacity(leaf_count);
        for _ in 0..leaf_count {
            leaf_family_set.push(LeafFamilyTagV1::decode(take_u8(bytes, pos)?)?);
        }
        let bucket_bits = take_u8(bytes, pos)?;
        let bucket_id_width = take_u8(bytes, pos)?;
        let min_bucket_count = take_u32(bytes, pos)?;
        let max_target_leaf_count = take_u32(bytes, pos)?;
        let policy_generation = take_u64(bytes, pos)?;
        let bucket_policy_digest = take_32(bytes, pos)?;
        let journal_checkpoint = take_opt_u64(bytes, pos)?;
        let journal_digest = take_32(bytes, pos)?;
        let checkpoint_bind = take_32(bytes, pos)?;
        let batch_limits = BatchProofLimits::decode(bytes, pos)?;
        Ok(Self {
            encoding_version,
            transcript_domain,
            proof_family,
            root_generation,
            settlement_root,
            backend_root,
            root_bind_version,
            root_bind,
            leaf_family_set,
            bucket_bits,
            bucket_id_width,
            min_bucket_count,
            max_target_leaf_count,
            policy_generation,
            bucket_policy_digest,
            journal_checkpoint,
            journal_digest,
            checkpoint_bind,
            batch_limits,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPathEntryV1 {
    pub path: SettlementPath,
    pub terminal_family: TerminalFamilyTagV1,
    pub leaf_family: LeafFamilyTagV1,
    pub shard_id: Option<u32>,
    pub routing_generation: Option<u64>,
    pub opening_index: u32,
    pub reference_index: u32,
}

impl BatchPathEntryV1 {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_path(out, self.path);
        out.push(self.terminal_family as u8);
        out.push(self.leaf_family as u8);
        put_opt_u32(out, self.shard_id);
        put_opt_u64(out, self.routing_generation);
        put_u32(out, self.opening_index);
        put_u32(out, self.reference_index);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        Ok(Self {
            path: take_path(bytes, pos)?,
            terminal_family: TerminalFamilyTagV1::decode(take_u8(bytes, pos)?)?,
            leaf_family: LeafFamilyTagV1::decode(take_u8(bytes, pos)?)?,
            shard_id: take_opt_u32(bytes, pos)?,
            routing_generation: take_opt_u64(bytes, pos)?,
            opening_index: take_u32(bytes, pos)?,
            reference_index: take_u32(bytes, pos)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessNodeV1 {
    pub tree_level: u16,
    pub node_domain: NodeDomainTagV1,
    pub child_index: u8,
    pub sibling_side: SiblingSideTagV1,
    pub subtree_marker: bool,
    pub hash_material: Vec<[u8; 32]>,
}

impl WitnessNodeV1 {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_u16(out, self.tree_level);
        out.push(self.node_domain as u8);
        out.push(self.child_index);
        out.push(self.sibling_side as u8);
        put_bool(out, self.subtree_marker);
        put_u32(out, self.hash_material.len() as u32);
        for hash in &self.hash_material {
            out.extend_from_slice(hash);
        }
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        let tree_level = take_u16(bytes, pos)?;
        let node_domain = NodeDomainTagV1::decode(take_u8(bytes, pos)?)?;
        let child_index = take_u8(bytes, pos)?;
        let sibling_side = SiblingSideTagV1::decode(take_u8(bytes, pos)?)?;
        let subtree_marker = take_bool(bytes, pos)?;
        let hash_material_count = take_u32(bytes, pos)?;
        if hash_material_count != 1 {
            return Err(ProofChkErr::BatchHashCountMix);
        }
        let mut hash_material = Vec::with_capacity(hash_material_count as usize);
        for _ in 0..hash_material_count {
            hash_material.push(take_32(bytes, pos)?);
        }
        Ok(Self {
            tree_level,
            node_domain,
            child_index,
            sibling_side,
            subtree_marker,
            hash_material,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpeningEntryV1 {
    pub opening_kind: OpeningKindTagV1,
    pub payload: Vec<u8>,
}

impl OpeningEntryV1 {
    #[must_use]
    pub fn from_inclusion(opening: InclusionOpeningV1) -> Self {
        Self {
            opening_kind: OpeningKindTagV1::InclusionLeaf,
            payload: opening.encode(),
        }
    }

    #[must_use]
    pub fn from_nonexistence(opening: NonExistenceOpeningV1) -> Self {
        Self {
            opening_kind: OpeningKindTagV1::AbsenceOpening,
            payload: opening.encode(),
        }
    }

    #[must_use]
    pub fn from_deletion(opening: DeletionFactV1) -> Self {
        Self {
            opening_kind: OpeningKindTagV1::DeletionFact,
            payload: opening.encode(),
        }
    }

    pub fn decode_inclusion(&self) -> Result<InclusionOpeningV1, ProofChkErr> {
        if self.opening_kind != OpeningKindTagV1::InclusionLeaf {
            return Err(ProofChkErr::BatchOpeningKindMix);
        }
        InclusionOpeningV1::decode(&self.payload)
    }

    pub fn decode_nonexistence(&self) -> Result<NonExistenceOpeningV1, ProofChkErr> {
        if self.opening_kind != OpeningKindTagV1::AbsenceOpening {
            return Err(ProofChkErr::BatchOpeningKindMix);
        }
        NonExistenceOpeningV1::decode(&self.payload)
    }

    pub fn decode_deletion(&self) -> Result<DeletionFactV1, ProofChkErr> {
        if self.opening_kind != OpeningKindTagV1::DeletionFact {
            return Err(ProofChkErr::BatchOpeningKindMix);
        }
        DeletionFactV1::decode(&self.payload)
    }

    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.opening_kind as u8);
        put_bytes(out, &self.payload);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        let opening_kind = OpeningKindTagV1::decode(take_u8(bytes, pos)?)?;
        let payload = take_bytes(bytes, pos)?;
        Ok(Self {
            opening_kind,
            payload,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InclusionOpeningV1 {
    pub version: u8,
    pub leaf_bytes: Vec<u8>,
}

impl InclusionOpeningV1 {
    pub fn new(leaf: &SettlementLeaf) -> Result<Self, ProofChkErr> {
        Ok(Self {
            version: OPENING_VERSION_V1,
            leaf_bytes: leaf.encode()?,
        })
    }

    pub fn decode_leaf(&self) -> Result<SettlementLeaf, ProofChkErr> {
        Ok(SettlementLeaf::decode(&self.leaf_bytes)?)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_bytes(&mut out, &self.leaf_bytes);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let leaf_bytes = take_bytes(bytes, &mut pos)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != OPENING_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            leaf_bytes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonExistenceOpeningV1 {
    pub version: u8,
    pub marker_leaf_bytes: Vec<u8>,
    pub default_commitment_version: u8,
    pub default_commitment: [u8; 32],
    pub default_child_commitment: [u8; 32],
}

impl NonExistenceOpeningV1 {
    pub fn new(marker_leaf: &SettlementLeaf) -> Result<Self, ProofChkErr> {
        Ok(Self {
            version: OPENING_VERSION_V1,
            marker_leaf_bytes: marker_leaf.encode()?,
            default_commitment_version: HJMT_DEFAULT_COMMITMENT_VERSION,
            default_commitment: hjmt_default_value_commitment(),
            default_child_commitment: hjmt_default_child_commitment(),
        })
    }

    pub fn decode_marker_leaf(&self) -> Result<SettlementLeaf, ProofChkErr> {
        Ok(SettlementLeaf::decode(&self.marker_leaf_bytes)?)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_bytes(&mut out, &self.marker_leaf_bytes);
        out.push(self.default_commitment_version);
        out.extend_from_slice(&self.default_commitment);
        out.extend_from_slice(&self.default_child_commitment);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let marker_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let default_commitment_version = take_u8(bytes, &mut pos)?;
        let default_commitment = take_32(bytes, &mut pos)?;
        let default_child_commitment = take_32(bytes, &mut pos)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != OPENING_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            marker_leaf_bytes,
            default_commitment_version,
            default_commitment,
            default_child_commitment,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorProofContextV1 {
    pub version: u8,
    pub prior_hjmt_version: u64,
    pub prior_settlement_root: SettlementStateRoot,
    pub prior_backend_root: [u8; 32],
    pub prior_root_bind_version: u8,
    pub prior_root_bind: [u8; 32],
    pub prior_journal_digest: [u8; 32],
    pub prior_checkpoint_bind: [u8; 32],
    pub definition_root_leaf_bytes: Vec<u8>,
    pub serial_root_leaf_bytes: Vec<u8>,
    pub bucket_root_leaf_bytes: Vec<u8>,
    pub definition_proof_bytes: Vec<u8>,
    pub serial_proof_bytes: Vec<u8>,
    pub bucket_proof_bytes: Vec<u8>,
    pub prior_terminal_proof_bytes: Vec<u8>,
}

impl PriorProofContextV1 {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_u64(&mut out, self.prior_hjmt_version);
        put_state_root(&mut out, self.prior_settlement_root);
        out.extend_from_slice(&self.prior_backend_root);
        out.push(self.prior_root_bind_version);
        out.extend_from_slice(&self.prior_root_bind);
        out.extend_from_slice(&self.prior_journal_digest);
        out.extend_from_slice(&self.prior_checkpoint_bind);
        put_bytes(&mut out, &self.definition_root_leaf_bytes);
        put_bytes(&mut out, &self.serial_root_leaf_bytes);
        put_bytes(&mut out, &self.bucket_root_leaf_bytes);
        put_bytes(&mut out, &self.definition_proof_bytes);
        put_bytes(&mut out, &self.serial_proof_bytes);
        put_bytes(&mut out, &self.bucket_proof_bytes);
        put_bytes(&mut out, &self.prior_terminal_proof_bytes);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let prior_hjmt_version = take_u64(bytes, &mut pos)?;
        let prior_settlement_root = take_state_root(bytes, &mut pos)?;
        let prior_backend_root = take_32(bytes, &mut pos)?;
        let prior_root_bind_version = take_u8(bytes, &mut pos)?;
        let prior_root_bind = take_32(bytes, &mut pos)?;
        let prior_journal_digest = take_32(bytes, &mut pos)?;
        let prior_checkpoint_bind = take_32(bytes, &mut pos)?;
        let definition_root_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let serial_root_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let bucket_root_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let definition_proof_bytes = take_bytes(bytes, &mut pos)?;
        let serial_proof_bytes = take_bytes(bytes, &mut pos)?;
        let bucket_proof_bytes = take_bytes(bytes, &mut pos)?;
        let prior_terminal_proof_bytes = take_bytes(bytes, &mut pos)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != PRIOR_CTX_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            prior_hjmt_version,
            prior_settlement_root,
            prior_backend_root,
            prior_root_bind_version,
            prior_root_bind,
            prior_journal_digest,
            prior_checkpoint_bind,
            definition_root_leaf_bytes,
            serial_root_leaf_bytes,
            bucket_root_leaf_bytes,
            definition_proof_bytes,
            serial_proof_bytes,
            bucket_proof_bytes,
            prior_terminal_proof_bytes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionFactV1 {
    pub version: u8,
    pub deleted_leaf_bytes: Vec<u8>,
    pub default_commitment_version: u8,
    pub default_child_commitment: [u8; 32],
    pub prior_context: PriorProofContextV1,
}

impl DeletionFactV1 {
    pub fn new(
        deleted_leaf: &SettlementLeaf,
        prior_context: PriorProofContextV1,
    ) -> Result<Self, ProofChkErr> {
        Ok(Self {
            version: DELETION_FACT_VERSION_V1,
            deleted_leaf_bytes: deleted_leaf.encode()?,
            default_commitment_version: HJMT_DEFAULT_COMMITMENT_VERSION,
            default_child_commitment: hjmt_default_child_commitment(),
            prior_context,
        })
    }

    pub fn decode_deleted_leaf(&self) -> Result<SettlementLeaf, ProofChkErr> {
        Ok(SettlementLeaf::decode(&self.deleted_leaf_bytes)?)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_bytes(&mut out, &self.deleted_leaf_bytes);
        out.push(self.default_commitment_version);
        out.extend_from_slice(&self.default_child_commitment);
        let prior = self.prior_context.encode();
        put_bytes(&mut out, &prior);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let deleted_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let default_commitment_version = take_u8(bytes, &mut pos)?;
        let default_child_commitment = take_32(bytes, &mut pos)?;
        let prior_bytes = take_bytes(bytes, &mut pos)?;
        let prior_context = PriorProofContextV1::decode(&prior_bytes)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != DELETION_FACT_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            deleted_leaf_bytes,
            default_commitment_version,
            default_child_commitment,
            prior_context,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathWitnessRefV1 {
    pub witness_indexes: Vec<u32>,
}

impl PathWitnessRefV1 {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_u32(out, self.witness_indexes.len() as u32);
        for index in &self.witness_indexes {
            put_u32(out, *index);
        }
    }

    pub(crate) fn decode(
        bytes: &[u8],
        pos: &mut usize,
        max_count: u32,
    ) -> Result<Self, ProofChkErr> {
        let count = take_count(bytes, pos, max_count)?;
        let mut witness_indexes = Vec::with_capacity(count);
        for _ in 0..count {
            witness_indexes.push(take_u32(bytes, pos)?);
        }
        Ok(Self { witness_indexes })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProofBlobV1 {
    pub header: BatchProofHeaderV1,
    pub path_table: Vec<BatchPathEntryV1>,
    pub witness_dag: Vec<WitnessNodeV1>,
    pub opening_table: Vec<OpeningEntryV1>,
    pub reference_table: Vec<PathWitnessRefV1>,
}

impl BatchProofBlobV1 {
    #[must_use]
    pub fn new(
        header: BatchProofHeaderV1,
        path_table: Vec<BatchPathEntryV1>,
        witness_dag: Vec<WitnessNodeV1>,
        opening_table: Vec<OpeningEntryV1>,
        reference_table: Vec<PathWitnessRefV1>,
    ) -> Self {
        Self {
            header,
            path_table,
            witness_dag,
            opening_table,
            reference_table,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProofChkErr> {
        let mut out = Vec::new();
        self.header.encode(&mut out);
        put_u32(&mut out, self.path_table.len() as u32);
        for entry in &self.path_table {
            entry.encode(&mut out);
        }
        put_u32(&mut out, self.witness_dag.len() as u32);
        for node in &self.witness_dag {
            node.encode(&mut out);
        }
        put_u32(&mut out, self.opening_table.len() as u32);
        for entry in &self.opening_table {
            entry.encode(&mut out);
        }
        put_u32(&mut out, self.reference_table.len() as u32);
        for entry in &self.reference_table {
            entry.encode(&mut out);
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        Self::decode_with_limits(bytes, BatchProofLimits::v1())
    }

    pub fn decode_with_limits(bytes: &[u8], ceil: BatchProofLimits) -> Result<Self, ProofChkErr> {
        if bytes.len() > ceil.max_total_bytes as usize {
            return Err(ProofChkErr::BatchLimitMix);
        }
        let mut pos = 0;
        let header = BatchProofHeaderV1::decode(bytes, &mut pos)?;
        if !ceil.contains(header.batch_limits) {
            return Err(ProofChkErr::BatchLimitMix);
        }
        if bytes.len() > header.batch_limits.max_total_bytes as usize {
            return Err(ProofChkErr::BatchLimitMix);
        }
        let path_cap = header.batch_limits.max_path_count.min(ceil.max_path_count);
        let witness_cap = header
            .batch_limits
            .max_witness_node_count
            .min(ceil.max_witness_node_count);
        let opening_cap = header
            .batch_limits
            .max_opening_count
            .min(ceil.max_opening_count);
        let ref_cap = header
            .batch_limits
            .max_reference_count
            .min(ceil.max_reference_count);

        let path_count = take_count(bytes, &mut pos, path_cap)?;
        let mut path_table = Vec::with_capacity(path_count);
        for _ in 0..path_count {
            path_table.push(BatchPathEntryV1::decode(bytes, &mut pos)?);
        }

        let witness_count = take_count(bytes, &mut pos, witness_cap)?;
        let mut witness_dag = Vec::with_capacity(witness_count);
        for _ in 0..witness_count {
            witness_dag.push(WitnessNodeV1::decode(bytes, &mut pos)?);
        }

        let opening_count = take_count(bytes, &mut pos, opening_cap)?;
        let mut opening_table = Vec::with_capacity(opening_count);
        for _ in 0..opening_count {
            opening_table.push(OpeningEntryV1::decode(bytes, &mut pos)?);
        }

        let ref_count = take_count(bytes, &mut pos, ref_cap)?;
        let mut reference_table = Vec::with_capacity(ref_count);
        for _ in 0..ref_count {
            reference_table.push(PathWitnessRefV1::decode(bytes, &mut pos, witness_cap)?);
        }

        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }

        let batch = Self {
            header,
            path_table,
            witness_dag,
            opening_table,
            reference_table,
        };
        batch.check_contract_v1()?;
        Ok(batch)
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        super::proof_batch_verify::check_batch_contract_v1(self)
    }
}

#[derive(Serialize)]
struct CheckpointWitnessChunkV1 {
    version: u8,
    ordinal: u32,
    chunk_kind: u8,
    encoding_version: u8,
    byte_length: u32,
    content_digest: [u8; 32],
    linked_terminal_ids: Vec<super::TerminalId>,
}

impl CheckpointWitnessChunkV1 {
    fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&frame_bytes(&[self.version]));
        bytes.extend_from_slice(&frame_bytes(&self.ordinal.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&[self.chunk_kind]));
        bytes.extend_from_slice(&frame_bytes(&[self.encoding_version]));
        bytes.extend_from_slice(&frame_bytes(&self.byte_length.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&self.content_digest));
        let linked_count = u32::try_from(self.linked_terminal_ids.len())
            .map_err(|_| ProofChkErr::BatchLimitMix)?;
        bytes.extend_from_slice(&frame_bytes(&linked_count.to_le_bytes()));
        for terminal_id in &self.linked_terminal_ids {
            bytes.extend_from_slice(&frame_bytes(terminal_id.as_bytes()));
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy)]
struct BatchCtxV1 {
    settlement_root: SettlementStateRoot,
    backend_root: [u8; 32],
    journal_checkpoint: u64,
    journal_digest: [u8; 32],
}

fn batch_ctx_v1(batches: &[BatchProofBlobV1]) -> Result<BatchCtxV1, ProofChkErr> {
    let mut ctx: Option<BatchCtxV1> = None;
    for batch in batches {
        batch.check_contract_v1()?;
        let checkpoint = batch
            .header
            .journal_checkpoint
            .ok_or(ProofChkErr::BatchCheckpointMix)?;
        let candidate = BatchCtxV1 {
            settlement_root: batch.header.settlement_root,
            backend_root: batch.header.backend_root,
            journal_checkpoint: checkpoint,
            journal_digest: batch.header.journal_digest,
        };
        if let Some(expect) = ctx {
            if candidate.settlement_root != expect.settlement_root
                || candidate.backend_root != expect.backend_root
            {
                return Err(ProofChkErr::BatchRootMix);
            }
            if candidate.journal_checkpoint != expect.journal_checkpoint
                || candidate.journal_digest != expect.journal_digest
            {
                return Err(ProofChkErr::BatchCheckpointMix);
            }
        } else {
            ctx = Some(candidate);
        }
    }
    ctx.ok_or(ProofChkErr::BatchLimitMix)
}

pub fn derive_journal_digest_v1(batches: &[BatchProofBlobV1]) -> Result<[u8; 32], ProofChkErr> {
    Ok(batch_ctx_v1(batches)?.journal_digest)
}

pub fn derive_witness_root_v1(batches: &[BatchProofBlobV1]) -> Result<[u8; 32], ProofChkErr> {
    batch_ctx_v1(batches)?;
    let batch_count = u32::try_from(batches.len()).map_err(|_| ProofChkErr::BatchLimitMix)?;
    let mut root_bytes = frame_bytes(&batch_count.to_le_bytes());
    for (ordinal, batch) in batches.iter().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| ProofChkErr::BatchLimitMix)?;
        let batch_bytes = batch.encode()?;
        let byte_length =
            u32::try_from(batch_bytes.len()).map_err(|_| ProofChkErr::BatchLimitMix)?;
        let content_digest =
            hash_zk::<StorBatchProofDom>(WITNESS_PAYLOAD_LABEL, &[batch_bytes.as_slice()]);
        let chunk = CheckpointWitnessChunkV1 {
            version: WITNESS_CHUNK_VER,
            ordinal,
            chunk_kind: WITNESS_CHUNK_BATCH,
            encoding_version: batch.header.encoding_version,
            byte_length,
            content_digest,
            linked_terminal_ids: batch
                .path_table
                .iter()
                .map(|entry| entry.path.terminal_id())
                .collect(),
        };
        let chunk_bytes = chunk.canonical_bytes()?;
        let chunk_hash =
            hash_zk::<StorBatchProofDom>(WITNESS_CHUNK_LABEL, &[chunk_bytes.as_slice()]);
        root_bytes.extend_from_slice(&frame_bytes(&chunk_hash));
    }
    Ok(hash_zk::<StorBatchProofDom>(
        WITNESS_ROOT_LABEL,
        &[root_bytes.as_slice()],
    ))
}

fn root_bind_v1(root: SettlementStateRoot, backend_root: [u8; 32]) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_root_bind_v1",
        &[&generation, &root_bytes, &backend_root],
    )
}

fn checkpoint_bind_v1(
    root: SettlementStateRoot,
    backend_root: [u8; 32],
    checkpoint: u64,
    journal_digest: [u8; 32],
) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    let checkpoint_bytes = checkpoint.to_be_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_hjmt_checkpoint_bind_v1",
        &[
            &generation,
            &root_bytes,
            &backend_root,
            &checkpoint_bytes,
            &journal_digest,
        ],
    )
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_bool(out: &mut Vec<u8>, value: bool) {
    out.push(u8::from(value));
}

fn put_opt_u32(out: &mut Vec<u8>, value: Option<u32>) {
    match value {
        Some(value) => {
            out.push(0x01);
            put_u32(out, value);
        }
        None => out.push(0x00),
    }
}

fn put_opt_u64(out: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            out.push(0x01);
            put_u64(out, value);
        }
        None => out.push(0x00),
    }
}

fn put_bytes(out: &mut Vec<u8>, value: &[u8]) {
    put_u32(out, value.len() as u32);
    out.extend_from_slice(value);
}

fn put_state_root(out: &mut Vec<u8>, root: SettlementStateRoot) {
    out.push(root.generation_version());
    out.extend_from_slice(root.as_bytes());
}

fn put_path(out: &mut Vec<u8>, path: SettlementPath) {
    out.extend_from_slice(path.definition_id.as_bytes());
    put_u32(out, path.serial_id.get());
    out.extend_from_slice(path.terminal_id.as_bytes());
}

fn take_u8(bytes: &[u8], pos: &mut usize) -> Result<u8, ProofChkErr> {
    let Some(value) = bytes.get(*pos).copied() else {
        return Err(ProofChkErr::BatchTrunc);
    };
    *pos += 1;
    Ok(value)
}

fn take_bool(bytes: &[u8], pos: &mut usize) -> Result<bool, ProofChkErr> {
    match take_u8(bytes, pos)? {
        0x00 => Ok(false),
        0x01 => Ok(true),
        _ => Err(ProofChkErr::BatchBoolMix),
    }
}

fn take_u16(bytes: &[u8], pos: &mut usize) -> Result<u16, ProofChkErr> {
    let end = pos.saturating_add(2);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 2];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u16::from_be_bytes(raw))
}

fn take_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, ProofChkErr> {
    let end = pos.saturating_add(4);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u32::from_be_bytes(raw))
}

fn take_u64(bytes: &[u8], pos: &mut usize) -> Result<u64, ProofChkErr> {
    let end = pos.saturating_add(8);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u64::from_be_bytes(raw))
}

fn take_opt_u32(bytes: &[u8], pos: &mut usize) -> Result<Option<u32>, ProofChkErr> {
    match take_u8(bytes, pos)? {
        0x00 => Ok(None),
        0x01 => Ok(Some(take_u32(bytes, pos)?)),
        _ => Err(ProofChkErr::BatchBoolMix),
    }
}

fn take_opt_u64(bytes: &[u8], pos: &mut usize) -> Result<Option<u64>, ProofChkErr> {
    match take_u8(bytes, pos)? {
        0x00 => Ok(None),
        0x01 => Ok(Some(take_u64(bytes, pos)?)),
        _ => Err(ProofChkErr::BatchBoolMix),
    }
}

fn take_32(bytes: &[u8], pos: &mut usize) -> Result<[u8; 32], ProofChkErr> {
    let end = pos.saturating_add(32);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 32];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(raw)
}

fn take_count(bytes: &[u8], pos: &mut usize, max: u32) -> Result<usize, ProofChkErr> {
    let count = take_u32(bytes, pos)?;
    if count > max {
        return Err(ProofChkErr::BatchLimitMix);
    }
    Ok(count as usize)
}

fn take_bytes(bytes: &[u8], pos: &mut usize) -> Result<Vec<u8>, ProofChkErr> {
    let len = take_u32(bytes, pos)? as usize;
    let end = pos.saturating_add(len);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let out = bytes[*pos..end].to_vec();
    *pos = end;
    Ok(out)
}

fn take_state_root(bytes: &[u8], pos: &mut usize) -> Result<SettlementStateRoot, ProofChkErr> {
    let version = take_u8(bytes, pos)?;
    let root = take_32(bytes, pos)?;
    SettlementStateRoot::from_version(version, root).ok_or(ProofChkErr::BatchRootGenerationMix)
}

fn take_path(bytes: &[u8], pos: &mut usize) -> Result<SettlementPath, ProofChkErr> {
    let definition_id = super::DefinitionId::new(take_32(bytes, pos)?);
    let serial_id = super::SerialId::new(take_u32(bytes, pos)?);
    let terminal_id = super::TerminalId::new(take_32(bytes, pos)?);
    Ok(SettlementPath::new(definition_id, serial_id, terminal_id))
}

fn take_pub_u8(bytes: &[u8], pos: &mut usize) -> Result<u8, ProofChkErr> {
    let Some(value) = bytes.get(*pos).copied() else {
        return Err(ProofChkErr::PublicationTrunc);
    };
    *pos += 1;
    Ok(value)
}

fn take_pub_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, ProofChkErr> {
    let end = pos.saturating_add(4);
    if end > bytes.len() {
        return Err(ProofChkErr::PublicationTrunc);
    }
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u32::from_be_bytes(raw))
}

fn take_pub_u64(bytes: &[u8], pos: &mut usize) -> Result<u64, ProofChkErr> {
    let end = pos.saturating_add(8);
    if end > bytes.len() {
        return Err(ProofChkErr::PublicationTrunc);
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u64::from_be_bytes(raw))
}

fn take_pub_opt_u64(bytes: &[u8], pos: &mut usize) -> Result<Option<u64>, ProofChkErr> {
    match take_pub_u8(bytes, pos)? {
        0x00 => Ok(None),
        0x01 => Ok(Some(take_pub_u64(bytes, pos)?)),
        _ => Err(ProofChkErr::PublicationPolicyMix),
    }
}

fn take_pub_32(bytes: &[u8], pos: &mut usize) -> Result<[u8; 32], ProofChkErr> {
    let end = pos.saturating_add(32);
    if end > bytes.len() {
        return Err(ProofChkErr::PublicationTrunc);
    }
    let mut raw = [0u8; 32];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(raw)
}

fn take_pub_state_root(bytes: &[u8], pos: &mut usize) -> Result<SettlementStateRoot, ProofChkErr> {
    let version = take_pub_u8(bytes, pos)?;
    let root = take_pub_32(bytes, pos)?;
    SettlementStateRoot::from_version(version, root)
        .ok_or(ProofChkErr::PublicationRootGenerationMix)
}
