#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::ShardEvidenceDomain;

use crate::{
    commit_subject::{
        digest_bytes, push_bytes32, push_len_prefixed, push_shard_id, push_u64, push_u8,
    },
    placement::AggregatorId,
    shard_vote::ShardVote,
    types::{RejectClass, RejectRecord, ShardId},
};

const SHARD_EVIDENCE_TAG: &[u8] = b"z00z.shard_evidence";
const EVIDENCE_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteEvidenceKind {
    Equivocation,
    PayloadWithholding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivocationEvidence {
    pub version: u8,
    pub voter_id: AggregatorId,
    pub shard_id: ShardId,
    pub term: u64,
    pub membership_digest: [u8; 32],
    pub first_vote: ShardVote,
    pub second_vote: ShardVote,
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
            first_vote,
            second_vote,
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
        push_len_prefixed(&mut out, &self.first_vote.encode());
        push_len_prefixed(&mut out, &self.second_vote.encode());
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
    #[must_use]
    pub fn new(
        reporter_id: AggregatorId,
        accused_id: AggregatorId,
        shard_id: ShardId,
        term: u64,
        membership_digest: [u8; 32],
        subject_digest: [u8; 32],
        payload_digest: [u8; 32],
        detail: impl Into<String>,
    ) -> Self {
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
        evidence
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

impl VoteEvidenceKind {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Equivocation => 1,
            Self::PayloadWithholding => 2,
        }
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
    pub fn record_vote(&mut self, vote: ShardVote) -> Result<(), VoteEvidence> {
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
            return Err(evidence);
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

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
