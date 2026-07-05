#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use z00z_storage::{
    checkpoint::{CheckpointExecTx, CheckpointId, CheckpointPubIn},
    settlement::{SettlementExecHandoff, StoreOp},
};

use crate::types::{
    BatchId, IntakeId, OrderedBatch, PublicationBinding, PublicationRecord, PublicationRequest,
    PublicationState, PublishedBatch, RejectRecord, SoftConfirmation, WorkItem, WorkPayload,
};
use crate::{
    consensus_store::{ConsensusStore, ConsensusStoreRecord, ConsensusValidatorDecision},
    evidence::{PayloadWithholdingEvidence, VoteEvidence, VoteEvidenceTracker},
    placement::AggregatorId,
    recovery::ShardRecoveryRecord,
    secondary_replay::{SecondaryReplayReject, SecondaryReplayRequest, SecondaryReplayVerifier},
    shard_quorum_certificate::ShardQuorumCertificate,
    shard_vote::{ShardVote, ShardVoteKind, ShardVoteRole},
    signature::{DeterministicLocalVoteSigner, VoteSigner},
    transport::{TransportPayloadStatus, VoteTransportEnvelope},
    CommitSubject,
};

pub trait AggregatorIngress {
    fn admit(&mut self, item: WorkPayload) -> Result<WorkItem, RejectRecord>;
}

pub trait AggregatorOrdering {
    fn order(&mut self, items: &[WorkItem]) -> Result<OrderedBatch, RejectRecord>;
}

pub trait AggregatorRecovery {
    fn build_publication(&mut self, batch: OrderedBatch) -> PublicationRequest;

    fn record_publication(&mut self, batch: PublishedBatch) -> PublicationRecord;
}

pub trait AggregatorService: AggregatorIngress + AggregatorOrdering + AggregatorRecovery {
    fn emit_soft_confirmation(&self, intake_id: &IntakeId, batch_id: &BatchId) -> SoftConfirmation;

    fn bind_exec_handoff(
        &self,
        batch: &OrderedBatch,
        ops: Vec<StoreOp>,
        txs: Vec<CheckpointExecTx>,
    ) -> SettlementExecHandoff {
        batch.exec_handoff(ops, txs)
    }
}

#[must_use]
pub fn bind_publication_contract(
    batch_id: BatchId,
    checkpoint_id: CheckpointId,
    route_table_digest: [u8; 32],
    pub_in: &CheckpointPubIn,
) -> PublicationBinding {
    PublicationBinding::new(batch_id, checkpoint_id, route_table_digest, pub_in)
}

#[must_use]
pub fn publication_record_for_published(
    published: &PublishedBatch,
    state: PublicationState,
) -> PublicationRecord {
    PublicationRecord {
        batch_id: published.batch_id,
        checkpoint_id: Some(published.checkpoint_id),
        state,
    }
}

#[must_use]
pub fn validator_decision_snapshot(
    verdict_kind: impl Into<String>,
    reject_class: Option<String>,
    batch_id: BatchId,
    subject: &CommitSubject,
    certificate: &ShardQuorumCertificate,
    theorem_digest: [u8; 32],
    checkpoint_id: Option<CheckpointId>,
    publication_binding: Option<&PublicationBinding>,
) -> ConsensusValidatorDecision {
    ConsensusValidatorDecision::new(
        verdict_kind,
        reject_class,
        checkpoint_id,
        publication_binding.map(PublicationBinding::binding_digest),
        theorem_digest,
        batch_id,
        subject.digest(),
        certificate.digest(),
    )
}

pub fn persist_consensus_commit(
    store: &ConsensusStore,
    recovery_record: &ShardRecoveryRecord,
    subject: &CommitSubject,
    votes: &[ShardVote],
    certificate: &ShardQuorumCertificate,
) -> Result<ConsensusStoreRecord, RejectRecord> {
    store
        .persist_commit(recovery_record, subject, votes, certificate)
        .map_err(|err| err.to_reject())
}

pub fn persist_consensus_publication(
    store: &ConsensusStore,
    batch_id: BatchId,
    publication_record: PublicationRecord,
    publication_binding: &PublicationBinding,
    published: &PublishedBatch,
) -> Result<ConsensusStoreRecord, RejectRecord> {
    store
        .persist_publication(batch_id, publication_record, publication_binding, published)
        .map_err(|err| err.to_reject())
}

pub fn persist_validator_decision(
    store: &ConsensusStore,
    batch_id: BatchId,
    decision: ConsensusValidatorDecision,
) -> Result<ConsensusStoreRecord, RejectRecord> {
    store
        .persist_validator_decision(batch_id, decision)
        .map_err(|err| err.to_reject())
}

#[derive(Debug, Clone, Copy)]
pub struct VoteExchangeContext<'a> {
    pub voter_role: ShardVoteRole,
    pub replay_request: SecondaryReplayRequest<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteExchangeOutcome {
    Vote(ShardVote),
    ReplayRejected(SecondaryReplayReject),
    Evidence(VoteEvidence),
    DuplicateMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteExchangeResult {
    pub message_id: [u8; 32],
    pub outcome: VoteExchangeOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayVerifiedVoteService<S> {
    signer: S,
    verifier: SecondaryReplayVerifier,
    seen_message_ids: BTreeSet<[u8; 32]>,
    evidence_tracker: VoteEvidenceTracker,
}

impl ReplayVerifiedVoteService<DeterministicLocalVoteSigner> {
    #[must_use]
    pub fn local() -> Self {
        Self::new(DeterministicLocalVoteSigner)
    }
}

impl Default for ReplayVerifiedVoteService<DeterministicLocalVoteSigner> {
    fn default() -> Self {
        Self::local()
    }
}

impl<S: VoteSigner> ReplayVerifiedVoteService<S> {
    #[must_use]
    pub fn new(signer: S) -> Self {
        Self {
            signer,
            verifier: SecondaryReplayVerifier,
            seen_message_ids: BTreeSet::new(),
            evidence_tracker: VoteEvidenceTracker::default(),
        }
    }

    #[must_use]
    pub fn evidence_records(&self) -> &[VoteEvidence] {
        self.evidence_tracker.records()
    }

    #[must_use]
    pub fn process_envelope(
        &mut self,
        envelope: &VoteTransportEnvelope,
        context: VoteExchangeContext<'_>,
    ) -> VoteExchangeResult {
        if self.seen_message_ids.contains(&envelope.message_id) {
            return VoteExchangeResult {
                message_id: envelope.message_id,
                outcome: VoteExchangeOutcome::DuplicateMessage,
            };
        }

        if envelope.to_id != context.replay_request.voter_id {
            self.seen_message_ids.insert(envelope.message_id);
            return VoteExchangeResult {
                message_id: envelope.message_id,
                outcome: VoteExchangeOutcome::ReplayRejected(replay_reject(
                    "membership drift: transport envelope recipient drifted from replay voter",
                )),
            };
        }

        if let TransportPayloadStatus::Missing { detail } = &envelope.payload_status {
            self.seen_message_ids.insert(envelope.message_id);
            let evidence =
                self.evidence_tracker
                    .record_payload_withholding(PayloadWithholdingEvidence::new(
                        envelope.to_id,
                        envelope.from_id,
                        envelope.subject.shard_id,
                        envelope.subject.term,
                        envelope.subject.membership_digest,
                        envelope.subject.digest(),
                        envelope.subject.payload_digest,
                        detail.clone(),
                    ));
            return VoteExchangeResult {
                message_id: envelope.message_id,
                outcome: VoteExchangeOutcome::Evidence(evidence),
            };
        }

        match self
            .verifier
            .verify(&envelope.subject, &context.replay_request)
        {
            crate::secondary_replay::SecondaryReplayVerdict::Accept(accept) => {
                self.seen_message_ids.insert(envelope.message_id);
                let vote = build_vote(
                    &self.signer,
                    &accept.subject,
                    context.replay_request.voter_id,
                    context.voter_role,
                    envelope.vote_kind,
                );
                if let Err(evidence) = self.evidence_tracker.record_vote(vote.clone()) {
                    return VoteExchangeResult {
                        message_id: envelope.message_id,
                        outcome: VoteExchangeOutcome::Evidence(evidence),
                    };
                }
                VoteExchangeResult {
                    message_id: envelope.message_id,
                    outcome: VoteExchangeOutcome::Vote(vote),
                }
            }
            crate::secondary_replay::SecondaryReplayVerdict::Reject(reject)
                if reject.class == crate::types::RejectClass::DeferredRetry =>
            {
                VoteExchangeResult {
                    message_id: envelope.message_id,
                    outcome: VoteExchangeOutcome::ReplayRejected(reject),
                }
            }
            crate::secondary_replay::SecondaryReplayVerdict::Reject(reject) => {
                self.seen_message_ids.insert(envelope.message_id);
                VoteExchangeResult {
                    message_id: envelope.message_id,
                    outcome: VoteExchangeOutcome::ReplayRejected(reject),
                }
            }
        }
    }
}

fn build_vote(
    signer: &impl VoteSigner,
    subject: &CommitSubject,
    voter_id: AggregatorId,
    voter_role: ShardVoteRole,
    vote_kind: ShardVoteKind,
) -> ShardVote {
    ShardVote::new(
        voter_id,
        voter_role,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        vote_kind,
        signer,
    )
}

fn replay_reject(detail: &str) -> SecondaryReplayReject {
    SecondaryReplayReject {
        code: crate::secondary_replay::SecondaryReplayRejectCode::MembershipDrift,
        class: crate::types::RejectClass::PolicyReject,
        detail: detail.to_string(),
    }
}
