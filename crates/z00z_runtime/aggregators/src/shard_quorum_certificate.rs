#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::{ShardMembershipDomain, ShardQuorumCertificateDomain};

use crate::{
    bft_committee::BftThresholds,
    commit_subject::{
        digest_bytes, push_batch_route, push_bytes32, push_len_prefixed, push_shard_id, push_u64,
        push_u8, push_usize, CommitSubject, COMMIT_SUBJECT_VERSION,
    },
    placement::AggregatorId,
    shard_vote::{ShardVote, ShardVoteRole},
    types::{RejectClass, RejectRecord},
};

const SHARD_QC_TAG: &[u8] = b"z00z.shard_qc";
const SHARD_MEMBERSHIP_TAG: &[u8] = b"z00z.shard_membership";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuorumRule {
    MajorityCft,
    BftTwoFPlusOne,
}

impl QuorumRule {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MajorityCft => "majority_cft",
            Self::BftTwoFPlusOne => "bft_two_f_plus_one",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardQuorumCertificate {
    pub version: u8,
    pub shard_id: crate::types::ShardId,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub term: u64,
    pub quorum_rule: QuorumRule,
    pub membership_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub votes: Vec<ShardVote>,
    pub aggregate_digest: [u8; 32],
    pub evidence_refs: Vec<[u8; 32]>,
}

impl ShardQuorumCertificate {
    pub fn new(
        subject: &CommitSubject,
        primary_id: AggregatorId,
        active_secondaries: impl IntoIterator<Item = AggregatorId>,
        votes: &[ShardVote],
    ) -> Result<Self, RejectRecord> {
        Self::new_with_rule(
            subject,
            primary_id,
            active_secondaries,
            votes,
            QuorumRule::MajorityCft,
        )
    }

    pub fn new_bft(
        subject: &CommitSubject,
        primary_id: AggregatorId,
        active_secondaries: impl IntoIterator<Item = AggregatorId>,
        votes: &[ShardVote],
    ) -> Result<Self, RejectRecord> {
        Self::new_with_rule(
            subject,
            primary_id,
            active_secondaries,
            votes,
            QuorumRule::BftTwoFPlusOne,
        )
    }

    fn new_with_rule(
        subject: &CommitSubject,
        primary_id: AggregatorId,
        active_secondaries: impl IntoIterator<Item = AggregatorId>,
        votes: &[ShardVote],
        quorum_rule: QuorumRule,
    ) -> Result<Self, RejectRecord> {
        let active_secondaries = active_secondaries.into_iter().collect::<BTreeSet<_>>();
        let expected_membership = membership_digest_for_voters(
            subject.route(),
            primary_id,
            active_secondaries.iter().copied(),
        );
        if subject.membership_digest != expected_membership {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: commit subject membership digest does not match active placement members",
            ));
        }

        let subject_digest = subject.digest();
        let mut unique_ids = BTreeSet::new();
        let mut canonical_votes = Vec::with_capacity(votes.len());
        let mut vote_kind = None;
        for vote in votes {
            if vote.shard_id != subject.shard_id {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "wrong shard: quorum vote drifted from the committed subject",
                ));
            }
            if vote.term != subject.term {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "mixed terms: quorum vote set contains different terms",
                ));
            }
            if vote.membership_digest != subject.membership_digest {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "mixed membership digests: quorum vote set drifted from active placement membership",
                ));
            }
            if vote.subject_digest != subject_digest {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "mixed subject digests: quorum vote set drifted from the committed subject",
                ));
            }
            if !vote.has_valid_signature() {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "invalid vote signature seam: vote signature bytes do not match vote content",
                ));
            }
            if !unique_ids.insert(vote.voter_id) {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "duplicate voter ids: quorum vote set must contain each voter once",
                ));
            }
            let expected_role = if vote.voter_id == primary_id {
                ShardVoteRole::Primary
            } else if active_secondaries.contains(&vote.voter_id) {
                ShardVoteRole::Secondary
            } else {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "inactive voter ids: quorum vote referenced a non-member",
                ));
            };
            if vote.voter_role != expected_role {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "wrong voter role: quorum vote role drifted from the active placement role",
                ));
            }
            if let Some(kind) = vote_kind {
                if kind != vote.vote_kind {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "mixed vote kinds: quorum vote set must use one canonical vote kind",
                    ));
                }
            } else {
                vote_kind = Some(vote.vote_kind);
            }
            canonical_votes.push(vote.clone());
        }

        let member_count = active_secondaries.len() + 1;
        let quorum_threshold = match quorum_rule {
            QuorumRule::MajorityCft => (member_count / 2) + 1,
            QuorumRule::BftTwoFPlusOne => BftThresholds::new(member_count)?.quorum_threshold,
        };
        if canonical_votes.len() < quorum_threshold {
            return Err(reject(
                RejectClass::DeferredRetry,
                match quorum_rule {
                    QuorumRule::MajorityCft => {
                        "below quorum: quorum vote set does not meet the majority threshold"
                    }
                    QuorumRule::BftTwoFPlusOne => {
                        "below quorum: quorum vote set does not meet the 2f+1 threshold"
                    }
                },
            ));
        }

        canonical_votes.sort_by_key(|vote| vote.voter_id);
        let evidence_refs = Vec::new();
        let mut certificate = Self {
            version: COMMIT_SUBJECT_VERSION,
            shard_id: subject.shard_id,
            routing_generation: subject.routing_generation,
            route_table_digest: subject.route_table_digest,
            term: subject.term,
            quorum_rule,
            membership_digest: subject.membership_digest,
            subject_digest,
            votes: canonical_votes,
            aggregate_digest: [0u8; 32],
            evidence_refs,
        };
        certificate.aggregate_digest = digest_bytes::<ShardQuorumCertificateDomain>(
            "aggregate_digest",
            &certificate.encode_without_aggregate(),
        );
        Ok(certificate)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.encode_without_aggregate();
        push_bytes32(&mut out, self.aggregate_digest);
        out
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.aggregate_digest
    }

    pub fn verify_subject(&self, subject: &CommitSubject) -> Result<(), RejectRecord> {
        if self.version != COMMIT_SUBJECT_VERSION {
            return Err(reject(
                RejectClass::PolicyReject,
                "quorum certificate version drifted from the live contract",
            ));
        }
        if self.shard_id != subject.shard_id
            || self.routing_generation != subject.routing_generation
            || self.route_table_digest != subject.route_table_digest
            || self.term != subject.term
            || self.membership_digest != subject.membership_digest
        {
            return Err(reject(
                RejectClass::PolicyReject,
                "quorum certificate metadata drifted from the committed subject",
            ));
        }

        let subject_digest = subject.digest();
        if self.subject_digest != subject_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "quorum certificate subject digest drifted from the committed subject",
            ));
        }

        let mut unique_ids = BTreeSet::new();
        let mut vote_kind = None;
        for vote in &self.votes {
            if vote.shard_id != subject.shard_id
                || vote.term != subject.term
                || vote.membership_digest != subject.membership_digest
                || vote.subject_digest != subject_digest
            {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "quorum certificate vote set drifted from the committed subject",
                ));
            }
            if !vote.has_valid_signature() {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "invalid vote signature seam: vote signature bytes do not match vote content",
                ));
            }
            if !unique_ids.insert(vote.voter_id) {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "duplicate voter ids: quorum vote set must contain each voter once",
                ));
            }
            if let Some(kind) = vote_kind {
                if kind != vote.vote_kind {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "mixed vote kinds: quorum vote set must use one canonical vote kind",
                    ));
                }
            } else {
                vote_kind = Some(vote.vote_kind);
            }
        }

        let expected_digest = digest_bytes::<ShardQuorumCertificateDomain>(
            "aggregate_digest",
            &self.encode_without_aggregate(),
        );
        if self.aggregate_digest != expected_digest {
            return Err(reject(
                RejectClass::PolicyReject,
                "quorum certificate aggregate digest drifted from its canonical encoding",
            ));
        }

        Ok(())
    }

    fn encode_without_aggregate(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(512);
        out.extend_from_slice(SHARD_QC_TAG);
        push_u8(&mut out, self.version);
        push_shard_id(&mut out, self.shard_id);
        push_u64(&mut out, self.routing_generation);
        push_bytes32(&mut out, self.route_table_digest);
        push_u64(&mut out, self.term);
        push_u8(
            &mut out,
            match self.quorum_rule {
                QuorumRule::MajorityCft => 1,
                QuorumRule::BftTwoFPlusOne => 2,
            },
        );
        push_bytes32(&mut out, self.membership_digest);
        push_bytes32(&mut out, self.subject_digest);
        push_usize(&mut out, self.votes.len());
        for vote in &self.votes {
            push_len_prefixed(&mut out, &vote.encode());
        }
        push_usize(&mut out, self.evidence_refs.len());
        for evidence in &self.evidence_refs {
            push_bytes32(&mut out, *evidence);
        }
        out
    }
}

#[must_use]
pub fn membership_digest_for_voters(
    route: crate::types::BatchRoute,
    primary_id: AggregatorId,
    active_secondaries: impl IntoIterator<Item = AggregatorId>,
) -> [u8; 32] {
    let active_secondaries = active_secondaries.into_iter().collect::<BTreeSet<_>>();
    let mut bytes = Vec::with_capacity(128);
    bytes.extend_from_slice(SHARD_MEMBERSHIP_TAG);
    push_u8(&mut bytes, COMMIT_SUBJECT_VERSION);
    push_batch_route(&mut bytes, route);
    push_u64(&mut bytes, u64::from(primary_id.as_u16()));
    push_usize(&mut bytes, active_secondaries.len());
    for secondary_id in active_secondaries {
        push_u64(&mut bytes, u64::from(secondary_id.as_u16()));
    }
    digest_bytes::<ShardMembershipDomain>("membership_digest", &bytes)
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
