#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::ShardEvidenceDomain;

use crate::{
    commit_subject::{
        digest_bytes, push_bytes32, push_len_prefixed, push_shard_id, push_u64, push_u8,
    },
    placement::AggregatorId,
    shard_vote::{ShardVote, ShardVoteKind},
    signature::VoteSignatureScheme,
    transport::VoteTransportEnvelope,
    types::{RejectClass, RejectRecord, ShardId},
};

const SHARD_EVIDENCE_TAG: &[u8] = b"z00z.shard_evidence";
const EVIDENCE_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Equivocation,
    PayloadWithholding,
    MissingBlob,
    WrongRoot,
    WrongRouteDigest,
    StaleMember,
    SplitBrain,
    TransportFault,
}

impl EvidenceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Equivocation => "equivocation",
            Self::PayloadWithholding => "payload_withholding",
            Self::MissingBlob => "missing_blob",
            Self::WrongRoot => "wrong_root",
            Self::WrongRouteDigest => "wrong_route_digest",
            Self::StaleMember => "stale_member",
            Self::SplitBrain => "split_brain",
            Self::TransportFault => "transport_fault",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Vote,
    Subject,
    Membership,
    Payload,
    BlobCommitment,
    Certificate,
    RouteDigest,
    StateRoot,
    TransportMessage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub kind: ArtifactKind,
    pub digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteArtifactRef {
    pub vote_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub vote_kind: ShardVoteKind,
    pub signature_scheme: VoteSignatureScheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteEvidenceKind {
    Equivocation,
    PayloadWithholding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportFaultEvidenceKind {
    Delay,
    Reorder,
    Duplicate,
    Replay,
    Drop,
    PartitionDeferred,
    Heal,
    Restart,
    Reconnect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivocationEvidence {
    pub version: u8,
    pub voter_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub membership_digest: [u8; 32],
    pub first_vote: VoteArtifactRef,
    pub second_vote: VoteArtifactRef,
    pub evidence_digest: [u8; 32],
}

impl EquivocationEvidence {
    pub fn new(first_vote: ShardVote, second_vote: ShardVote) -> Result<Self, RejectRecord> {
        if first_vote.voter_id != second_vote.voter_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "equivocation evidence requires one voter identity",
            ));
        }
        if first_vote.shard_id != second_vote.shard_id || first_vote.term != second_vote.term {
            return Err(reject(
                RejectClass::PolicyReject,
                "equivocation evidence requires one shard and one term",
            ));
        }
        if first_vote.membership_digest != second_vote.membership_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "equivocation evidence requires one membership digest",
            ));
        }
        if first_vote.subject_digest == second_vote.subject_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "equivocation evidence requires conflicting subject digests",
            ));
        }

        let (first_vote, second_vote) = canonical_vote_pair(first_vote, second_vote);
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            voter_id: first_vote.voter_id,
            shard_id: first_vote.shard_id,
            term: first_vote.term,
            membership_digest: first_vote.membership_digest,
            first_vote: vote_artifact_ref(&first_vote),
            second_vote: vote_artifact_ref(&second_vote),
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest =
            digest_bytes::<ShardEvidenceDomain>("equivocation", &evidence.encode_without_digest());
        Ok(evidence)
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(512);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, VoteEvidenceKind::Equivocation.code());
        push_u64(&mut out, u64::from(self.voter_id.as_u16()));
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.membership_digest);
        push_bytes32(&mut out, self.first_vote.vote_digest);
        push_bytes32(&mut out, self.first_vote.subject_digest);
        push_u8(
            &mut out,
            match self.first_vote.vote_kind {
                ShardVoteKind::Prepare => 1,
                ShardVoteKind::Commit => 2,
                ShardVoteKind::LocalCommit => 3,
            },
        );
        push_u8(&mut out, self.first_vote.signature_scheme.code());
        push_bytes32(&mut out, self.second_vote.vote_digest);
        push_bytes32(&mut out, self.second_vote.subject_digest);
        push_u8(
            &mut out,
            match self.second_vote.vote_kind {
                ShardVoteKind::Prepare => 1,
                ShardVoteKind::Commit => 2,
                ShardVoteKind::LocalCommit => 3,
            },
        );
        push_u8(&mut out, self.second_vote.signature_scheme.code());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadWithholdingEvidence {
    pub version: u8,
    pub reporter_id: AggregatorId,
    pub accused_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub membership_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub payload_digest: [u8; 32],
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl PayloadWithholdingEvidence {
    pub fn new(
        reporter_id: AggregatorId,
        accused_id: AggregatorId,
        shard_id: ShardId,
        term: u64,
        membership_digest: [u8; 32],
        subject_digest: [u8; 32],
        payload_digest: [u8; 32],
        detail: impl Into<String>,
    ) -> Result<Self, RejectRecord> {
        require_nonzero_digest(
            membership_digest,
            "payload withholding evidence requires a membership digest",
        )?;
        require_nonzero_digest(
            subject_digest,
            "payload withholding evidence requires a subject digest",
        )?;
        require_nonzero_digest(
            payload_digest,
            "payload withholding evidence requires a payload digest",
        )?;
        let detail = detail.into();
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            reporter_id,
            accused_id,
            shard_id,
            term,
            membership_digest,
            subject_digest,
            payload_digest,
            detail,
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest = digest_bytes::<ShardEvidenceDomain>(
            "payload_withholding",
            &evidence.encode_without_digest(),
        );
        Ok(evidence)
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(256);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, VoteEvidenceKind::PayloadWithholding.code());
        push_u64(&mut out, u64::from(self.reporter_id.as_u16()));
        push_u64(&mut out, u64::from(self.accused_id.as_u16()));
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.membership_digest);
        push_bytes32(&mut out, self.subject_digest);
        push_bytes32(&mut out, self.payload_digest);
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingBlobEvidence {
    pub version: u8,
    pub reporter_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub membership_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub namespace: [u8; 8],
    pub blob_commitment: [u8; 32],
    pub certificate_digest: [u8; 32],
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl MissingBlobEvidence {
    pub fn new(
        reporter_id: AggregatorId,
        shard_id: ShardId,
        term: u64,
        membership_digest: [u8; 32],
        subject_digest: [u8; 32],
        namespace: [u8; 8],
        blob_commitment: [u8; 32],
        certificate_digest: [u8; 32],
        detail: impl Into<String>,
    ) -> Result<Self, RejectRecord> {
        require_nonzero_digest(
            membership_digest,
            "missing blob evidence requires a membership digest",
        )?;
        require_nonzero_digest(
            subject_digest,
            "missing blob evidence requires a subject digest",
        )?;
        require_namespace(namespace, "missing blob evidence requires a namespace")?;
        require_nonzero_digest(
            blob_commitment,
            "missing blob evidence requires a blob commitment",
        )?;
        require_nonzero_digest(
            certificate_digest,
            "missing blob evidence requires a certificate digest",
        )?;
        let detail = detail.into();
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            reporter_id,
            shard_id,
            term,
            membership_digest,
            subject_digest,
            namespace,
            blob_commitment,
            certificate_digest,
            detail,
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest =
            digest_bytes::<ShardEvidenceDomain>("missing_blob", &evidence.encode_without_digest());
        Ok(evidence)
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(256);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 12);
        push_u64(&mut out, u64::from(self.reporter_id.as_u16()));
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.membership_digest);
        push_bytes32(&mut out, self.subject_digest);
        push_len_prefixed(&mut out, &self.namespace);
        push_bytes32(&mut out, self.blob_commitment);
        push_bytes32(&mut out, self.certificate_digest);
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrongRootEvidence {
    pub version: u8,
    pub reporter_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub subject_digest: [u8; 32],
    pub expected_root: [u8; 32],
    pub claimed_root: [u8; 32],
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl WrongRootEvidence {
    pub fn new(
        reporter_id: AggregatorId,
        shard_id: ShardId,
        term: u64,
        subject_digest: [u8; 32],
        expected_root: [u8; 32],
        claimed_root: [u8; 32],
        detail: impl Into<String>,
    ) -> Result<Self, RejectRecord> {
        require_nonzero_digest(
            subject_digest,
            "wrong root evidence requires a subject digest",
        )?;
        require_nonzero_digest(
            expected_root,
            "wrong root evidence requires an expected root",
        )?;
        require_nonzero_digest(claimed_root, "wrong root evidence requires a claimed root")?;
        require_distinct_digests(
            expected_root,
            claimed_root,
            "wrong root evidence requires conflicting root digests",
        )?;
        let detail = detail.into();
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            reporter_id,
            shard_id,
            term,
            subject_digest,
            expected_root,
            claimed_root,
            detail,
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest =
            digest_bytes::<ShardEvidenceDomain>("wrong_root", &evidence.encode_without_digest());
        Ok(evidence)
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(192);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 13);
        push_u64(&mut out, u64::from(self.reporter_id.as_u16()));
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.subject_digest);
        push_bytes32(&mut out, self.expected_root);
        push_bytes32(&mut out, self.claimed_root);
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrongRouteDigestEvidence {
    pub version: u8,
    pub reporter_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub subject_digest: [u8; 32],
    pub expected_route_digest: [u8; 32],
    pub claimed_route_digest: [u8; 32],
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl WrongRouteDigestEvidence {
    pub fn new(
        reporter_id: AggregatorId,
        shard_id: ShardId,
        term: u64,
        subject_digest: [u8; 32],
        expected_route_digest: [u8; 32],
        claimed_route_digest: [u8; 32],
        detail: impl Into<String>,
    ) -> Result<Self, RejectRecord> {
        require_nonzero_digest(
            subject_digest,
            "wrong route digest evidence requires a subject digest",
        )?;
        require_nonzero_digest(
            expected_route_digest,
            "wrong route digest evidence requires an expected route digest",
        )?;
        require_nonzero_digest(
            claimed_route_digest,
            "wrong route digest evidence requires a claimed route digest",
        )?;
        require_distinct_digests(
            expected_route_digest,
            claimed_route_digest,
            "wrong route digest evidence requires conflicting route digests",
        )?;
        let detail = detail.into();
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            reporter_id,
            shard_id,
            term,
            subject_digest,
            expected_route_digest,
            claimed_route_digest,
            detail,
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest = digest_bytes::<ShardEvidenceDomain>(
            "wrong_route_digest",
            &evidence.encode_without_digest(),
        );
        Ok(evidence)
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(192);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 14);
        push_u64(&mut out, u64::from(self.reporter_id.as_u16()));
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.subject_digest);
        push_bytes32(&mut out, self.expected_route_digest);
        push_bytes32(&mut out, self.claimed_route_digest);
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaleMemberEvidence {
    pub version: u8,
    pub member_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub expected_membership_digest: [u8; 32],
    pub claimed_membership_digest: [u8; 32],
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl StaleMemberEvidence {
    pub fn new(
        member_id: AggregatorId,
        shard_id: ShardId,
        term: u64,
        expected_membership_digest: [u8; 32],
        claimed_membership_digest: [u8; 32],
        detail: impl Into<String>,
    ) -> Result<Self, RejectRecord> {
        require_nonzero_digest(
            expected_membership_digest,
            "stale member evidence requires an expected membership digest",
        )?;
        require_nonzero_digest(
            claimed_membership_digest,
            "stale member evidence requires a claimed membership digest",
        )?;
        require_distinct_digests(
            expected_membership_digest,
            claimed_membership_digest,
            "stale member evidence requires conflicting membership digests",
        )?;
        let detail = detail.into();
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            member_id,
            shard_id,
            term,
            expected_membership_digest,
            claimed_membership_digest,
            detail,
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest =
            digest_bytes::<ShardEvidenceDomain>("stale_member", &evidence.encode_without_digest());
        Ok(evidence)
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(192);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 15);
        push_u64(&mut out, u64::from(self.member_id.as_u16()));
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.expected_membership_digest);
        push_bytes32(&mut out, self.claimed_membership_digest);
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitBrainEvidence {
    pub version: u8,
    pub reporter_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub membership_digest: [u8; 32],
    pub first_subject_digest: [u8; 32],
    pub second_subject_digest: [u8; 32],
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl SplitBrainEvidence {
    pub fn new(
        reporter_id: AggregatorId,
        shard_id: ShardId,
        term: u64,
        membership_digest: [u8; 32],
        first_subject_digest: [u8; 32],
        second_subject_digest: [u8; 32],
        detail: impl Into<String>,
    ) -> Result<Self, RejectRecord> {
        require_nonzero_digest(
            membership_digest,
            "split brain evidence requires a membership digest",
        )?;
        require_nonzero_digest(
            first_subject_digest,
            "split brain evidence requires a first subject digest",
        )?;
        require_nonzero_digest(
            second_subject_digest,
            "split brain evidence requires a second subject digest",
        )?;
        require_distinct_digests(
            first_subject_digest,
            second_subject_digest,
            "split brain evidence requires conflicting subject digests",
        )?;
        let detail = detail.into();
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            reporter_id,
            shard_id,
            term,
            membership_digest,
            first_subject_digest,
            second_subject_digest,
            detail,
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest =
            digest_bytes::<ShardEvidenceDomain>("split_brain", &evidence.encode_without_digest());
        Ok(evidence)
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(192);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, 16);
        push_u64(&mut out, u64::from(self.reporter_id.as_u16()));
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.term);
        push_bytes32(&mut out, self.membership_digest);
        push_bytes32(&mut out, self.first_subject_digest);
        push_bytes32(&mut out, self.second_subject_digest);
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteEvidence {
    Equivocation(EquivocationEvidence),
    PayloadWithholding(PayloadWithholdingEvidence),
}

impl VoteEvidence {
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        match self {
            Self::Equivocation(evidence) => evidence.evidence_digest,
            Self::PayloadWithholding(evidence) => evidence.evidence_digest,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> VoteEvidenceKind {
        match self {
            Self::Equivocation(_) => VoteEvidenceKind::Equivocation,
            Self::PayloadWithholding(_) => VoteEvidenceKind::PayloadWithholding,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceRecord {
    Equivocation(EquivocationEvidence),
    PayloadWithholding(PayloadWithholdingEvidence),
    MissingBlob(MissingBlobEvidence),
    WrongRoot(WrongRootEvidence),
    WrongRouteDigest(WrongRouteDigestEvidence),
    StaleMember(StaleMemberEvidence),
    SplitBrain(SplitBrainEvidence),
    TransportFault(TransportFaultEvidence),
}

impl EvidenceRecord {
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        match self {
            Self::Equivocation(evidence) => evidence.evidence_digest,
            Self::PayloadWithholding(evidence) => evidence.evidence_digest,
            Self::MissingBlob(evidence) => evidence.evidence_digest,
            Self::WrongRoot(evidence) => evidence.evidence_digest,
            Self::WrongRouteDigest(evidence) => evidence.evidence_digest,
            Self::StaleMember(evidence) => evidence.evidence_digest,
            Self::SplitBrain(evidence) => evidence.evidence_digest,
            Self::TransportFault(evidence) => evidence.evidence_digest,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> EvidenceKind {
        match self {
            Self::Equivocation(_) => EvidenceKind::Equivocation,
            Self::PayloadWithholding(_) => EvidenceKind::PayloadWithholding,
            Self::MissingBlob(_) => EvidenceKind::MissingBlob,
            Self::WrongRoot(_) => EvidenceKind::WrongRoot,
            Self::WrongRouteDigest(_) => EvidenceKind::WrongRouteDigest,
            Self::StaleMember(_) => EvidenceKind::StaleMember,
            Self::SplitBrain(_) => EvidenceKind::SplitBrain,
            Self::TransportFault(_) => EvidenceKind::TransportFault,
        }
    }

    #[must_use]
    pub fn artifact_refs(&self) -> Vec<ArtifactRef> {
        match self {
            Self::Equivocation(evidence) => vec![
                artifact_ref(ArtifactKind::Vote, evidence.first_vote.vote_digest),
                artifact_ref(ArtifactKind::Vote, evidence.second_vote.vote_digest),
                artifact_ref(ArtifactKind::Subject, evidence.first_vote.subject_digest),
                artifact_ref(ArtifactKind::Subject, evidence.second_vote.subject_digest),
                artifact_ref(ArtifactKind::Membership, evidence.membership_digest),
            ],
            Self::PayloadWithholding(evidence) => vec![
                artifact_ref(ArtifactKind::Subject, evidence.subject_digest),
                artifact_ref(ArtifactKind::Payload, evidence.payload_digest),
                artifact_ref(ArtifactKind::Membership, evidence.membership_digest),
            ],
            Self::MissingBlob(evidence) => vec![
                artifact_ref(ArtifactKind::Subject, evidence.subject_digest),
                artifact_ref(ArtifactKind::BlobCommitment, evidence.blob_commitment),
                artifact_ref(ArtifactKind::Certificate, evidence.certificate_digest),
                artifact_ref(ArtifactKind::Membership, evidence.membership_digest),
            ],
            Self::WrongRoot(evidence) => vec![
                artifact_ref(ArtifactKind::Subject, evidence.subject_digest),
                artifact_ref(ArtifactKind::StateRoot, evidence.expected_root),
                artifact_ref(ArtifactKind::StateRoot, evidence.claimed_root),
            ],
            Self::WrongRouteDigest(evidence) => vec![
                artifact_ref(ArtifactKind::Subject, evidence.subject_digest),
                artifact_ref(ArtifactKind::RouteDigest, evidence.expected_route_digest),
                artifact_ref(ArtifactKind::RouteDigest, evidence.claimed_route_digest),
            ],
            Self::StaleMember(evidence) => vec![
                artifact_ref(
                    ArtifactKind::Membership,
                    evidence.expected_membership_digest,
                ),
                artifact_ref(ArtifactKind::Membership, evidence.claimed_membership_digest),
            ],
            Self::SplitBrain(evidence) => vec![
                artifact_ref(ArtifactKind::Membership, evidence.membership_digest),
                artifact_ref(ArtifactKind::Subject, evidence.first_subject_digest),
                artifact_ref(ArtifactKind::Subject, evidence.second_subject_digest),
            ],
            Self::TransportFault(evidence) => {
                let mut refs = Vec::new();
                if evidence.message_id != [0u8; 32] {
                    refs.push(artifact_ref(
                        ArtifactKind::TransportMessage,
                        evidence.message_id,
                    ));
                }
                refs
            }
        }
    }

    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Equivocation(_) => None,
            Self::PayloadWithholding(evidence) => Some(&evidence.detail),
            Self::MissingBlob(evidence) => Some(&evidence.detail),
            Self::WrongRoot(evidence) => Some(&evidence.detail),
            Self::WrongRouteDigest(evidence) => Some(&evidence.detail),
            Self::StaleMember(evidence) => Some(&evidence.detail),
            Self::SplitBrain(evidence) => Some(&evidence.detail),
            Self::TransportFault(evidence) => Some(&evidence.detail),
        }
    }
}

impl From<VoteEvidence> for EvidenceRecord {
    fn from(value: VoteEvidence) -> Self {
        match value {
            VoteEvidence::Equivocation(evidence) => Self::Equivocation(evidence),
            VoteEvidence::PayloadWithholding(evidence) => Self::PayloadWithholding(evidence),
        }
    }
}

impl From<Box<VoteEvidence>> for EvidenceRecord {
    fn from(value: Box<VoteEvidence>) -> Self {
        Self::from(*value)
    }
}

impl From<TransportFaultEvidence> for EvidenceRecord {
    fn from(value: TransportFaultEvidence) -> Self {
        Self::TransportFault(value)
    }
}

impl VoteEvidenceKind {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Equivocation => 1,
            Self::PayloadWithholding => 2,
        }
    }
}

impl TransportFaultEvidenceKind {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Delay => 3,
            Self::Reorder => 4,
            Self::Duplicate => 5,
            Self::Replay => 6,
            Self::Drop => 7,
            Self::PartitionDeferred => 8,
            Self::Heal => 9,
            Self::Restart => 10,
            Self::Reconnect => 11,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportFaultEvidence {
    pub version: u8,
    pub kind: TransportFaultEvidenceKind,
    pub tick: u64,
    pub message_id: [u8; 32],
    pub from_id: AggregatorId,
    pub to_id: AggregatorId,
    pub detail: String,
    pub evidence_digest: [u8; 32],
}

impl TransportFaultEvidence {
    #[must_use]
    pub fn for_envelope(
        kind: TransportFaultEvidenceKind,
        tick: u64,
        envelope: &VoteTransportEnvelope,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(
            kind,
            tick,
            envelope.message_id,
            envelope.from_id,
            envelope.to_id,
            detail.into(),
        )
    }

    #[must_use]
    pub fn for_peer(
        kind: TransportFaultEvidenceKind,
        tick: u64,
        aggregator_id: AggregatorId,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(
            kind,
            tick,
            [0u8; 32],
            aggregator_id,
            aggregator_id,
            detail.into(),
        )
    }

    fn new(
        kind: TransportFaultEvidenceKind,
        tick: u64,
        message_id: [u8; 32],
        from_id: AggregatorId,
        to_id: AggregatorId,
        detail: String,
    ) -> Self {
        let mut evidence = Self {
            version: EVIDENCE_VERSION,
            kind,
            tick,
            message_id,
            from_id,
            to_id,
            detail,
            evidence_digest: [0u8; 32],
        };
        evidence.evidence_digest = digest_bytes::<ShardEvidenceDomain>(
            "transport_fault",
            &evidence.encode_without_digest(),
        );
        evidence
    }

    #[must_use]
    pub fn encode_without_digest(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(192);
        out.extend_from_slice(SHARD_EVIDENCE_TAG);
        push_u8(&mut out, self.version);
        push_u8(&mut out, self.kind.code());
        push_u64(&mut out, self.tick);
        push_bytes32(&mut out, self.message_id);
        push_u64(&mut out, u64::from(self.from_id.as_u16()));
        push_u64(&mut out, u64::from(self.to_id.as_u16()));
        push_len_prefixed(&mut out, self.detail.as_bytes());
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct VoteConflictKey {
    voter_id: AggregatorId,
    shard_id: ShardId,
    term: u64,
    membership_digest: [u8; 32],
}

impl VoteConflictKey {
    fn new(vote: &ShardVote) -> Self {
        Self {
            voter_id: vote.voter_id,
            shard_id: vote.shard_id,
            term: vote.term,
            membership_digest: vote.membership_digest,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VoteEvidenceTracker {
    recorded_votes: BTreeMap<VoteConflictKey, ShardVote>,
    records: Vec<VoteEvidence>,
}

impl VoteEvidenceTracker {
    pub fn record_vote(&mut self, vote: ShardVote) -> Result<(), Box<VoteEvidence>> {
        let key = VoteConflictKey::new(&vote);
        if let Some(existing) = self.recorded_votes.get(&key) {
            if existing.subject_digest == vote.subject_digest {
                return Ok(());
            }

            let evidence = VoteEvidence::Equivocation(
                EquivocationEvidence::new(existing.clone(), vote)
                    .expect("conflicting votes produce canonical evidence"),
            );
            self.records.push(evidence.clone());
            return Err(Box::new(evidence));
        }

        self.recorded_votes.insert(key, vote);
        Ok(())
    }

    #[must_use]
    pub fn record_payload_withholding(
        &mut self,
        evidence: PayloadWithholdingEvidence,
    ) -> VoteEvidence {
        let record = VoteEvidence::PayloadWithholding(evidence);
        self.records.push(record.clone());
        record
    }

    #[must_use]
    pub fn records(&self) -> &[VoteEvidence] {
        &self.records
    }
}

fn canonical_vote_pair(first_vote: ShardVote, second_vote: ShardVote) -> (ShardVote, ShardVote) {
    if second_vote.subject_digest < first_vote.subject_digest {
        (second_vote, first_vote)
    } else {
        (first_vote, second_vote)
    }
}

fn vote_artifact_ref(vote: &ShardVote) -> VoteArtifactRef {
    VoteArtifactRef {
        vote_digest: vote.digest(),
        subject_digest: vote.subject_digest,
        vote_kind: vote.vote_kind,
        signature_scheme: vote.signature_scheme(),
    }
}

fn artifact_ref(kind: ArtifactKind, digest: [u8; 32]) -> ArtifactRef {
    ArtifactRef { kind, digest }
}

fn require_nonzero_digest(digest: [u8; 32], detail: &str) -> Result<(), RejectRecord> {
    if digest == [0u8; 32] {
        return Err(reject(RejectClass::PolicyReject, detail));
    }
    Ok(())
}

fn require_distinct_digests(
    first: [u8; 32],
    second: [u8; 32],
    detail: &str,
) -> Result<(), RejectRecord> {
    if first == second {
        return Err(reject(RejectClass::PolicyReject, detail));
    }
    Ok(())
}

fn require_namespace(namespace: [u8; 8], detail: &str) -> Result<(), RejectRecord> {
    if namespace == [0u8; 8] {
        return Err(reject(RejectClass::PolicyReject, detail));
    }
    Ok(())
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
