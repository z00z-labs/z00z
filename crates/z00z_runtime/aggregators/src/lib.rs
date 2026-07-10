#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod batch_planner;
mod bft_committee;
mod bft_engine;
mod commit_subject;
mod consensus_adapter;
mod consensus_store;
mod dist_dispatch;
mod dist_scheduler;
mod dist_sim;
mod evidence;
mod hotstuff_local;
mod ingress;
mod ordering;
mod placement;
mod recovery;
mod scheduler;
mod secondary_replay;
mod service;
mod shard_exec;
mod shard_quorum_certificate;
mod shard_vote;
mod signature;
mod transport;
mod types;

pub use batch_planner::{
    BatchPlanner, PlannerAuthority, RouteErr, RouteRangeRule, ShardRouteTable,
};
pub use bft_committee::{BftCommittee, BftThresholds};
pub use bft_engine::{BftCommit, BftEngine};
pub use commit_subject::{CommitSubject, JournalCandidate};
pub use consensus_adapter::{ConsensusAdapter, ConsensusCommit, MembershipChange};
pub use consensus_store::{
    ConsensusStore, ConsensusStoreBinding, ConsensusStoreError, ConsensusStorePublication,
    ConsensusStorePublishedBatch, ConsensusStoreRecord, ConsensusStoreRouteCursor,
    ConsensusValidatorDecision, CONSENSUS_STORE_BACKEND, CONSENSUS_STORE_SCHEMA_VERSION,
};
pub use dist_dispatch::{
    DispatchStage, DispatchVerdict, DistDispatch, DistLevel, DistNote, DistNoteKind, RouteRollout,
};
pub use dist_scheduler::{BatchWave, DistScheduler, ScheduledBatch, SchedulerWave};
pub use dist_sim::{DistNode, DistSim, FrameStage, FrameVerdict, JournalFrame};
pub use evidence::{
    ArtifactKind, ArtifactRef, EquivocationEvidence, EvidenceKind, EvidenceRecord,
    MissingBlobEvidence, PayloadWithholdingEvidence, SplitBrainEvidence, StaleMemberEvidence,
    TransportFaultEvidence, TransportFaultEvidenceKind, VoteArtifactRef, VoteEvidence,
    VoteEvidenceKind, VoteEvidenceTracker, WrongRootEvidence, WrongRouteDigestEvidence,
};
pub use hotstuff_local::{
    HotstuffCommit, HotstuffLeaderConflict, HotstuffLocal, HotstuffProposal, HotstuffQc,
    HotstuffTimeout, HotstuffViewChange,
};
pub use ingress::IngressBoundary;
pub use ordering::OrderingBoundary;
pub use placement::{
    AggregatorId, SecondaryState, ShardPlacement, ShardPlacementTable, ShardPlacementView,
};
pub use recovery::{
    PersistedConsensusRestart, RecoveryBoundary, RecoveryIntent, ShardRecoveryRecord,
};
pub use scheduler::SchedulerBoundary;
pub use secondary_replay::{
    SecondaryReplayAccept, SecondaryReplayReject, SecondaryReplayRejectCode,
    SecondaryReplayRequest, SecondaryReplayVerdict, SecondaryReplayVerifier,
};
pub use service::{
    bind_publication_contract, persist_consensus_commit, persist_consensus_publication,
    persist_validator_decision, publication_record_for_published, validator_decision_snapshot,
    AggregatorIngress, AggregatorOrdering, AggregatorRecovery, AggregatorService,
    ReplayVerifiedVoteService, VoteExchangeContext, VoteExchangeOutcome, VoteExchangeResult,
};
pub use shard_exec::{ShardExecState, ShardExecTicket, ShardExecutor};
pub use shard_quorum_certificate::{
    membership_digest_for_voters, QuorumRule, ShardQuorumCertificate,
};
pub use shard_vote::{ShardVote, ShardVoteKind, ShardVoteRole};
pub use signature::{
    verify_vote_signature, DeterministicLocalVoteSigner, VoteSignature, VoteSignatureScheme,
    VoteSignatureVerifier, VoteSigner,
};
pub use transport::{
    InMemoryVoteTransport, TransportDeliveryPlan, TransportPayloadStatus, VoteTransport,
    VoteTransportEnvelope,
};
pub use types::{
    BatchId, BatchPlanned, BatchRoute, IntakeId, ObjectWitnessBundleV1, OrderedBatch, PlanDigest,
    PlannerMode, PublicationBinding, PublicationReadinessErr, PublicationRecord,
    PublicationRequest, PublicationState, PublishedBatch, RejectClass, RejectRecord,
    RightWitnessRefV1, RightWitnessStateV1, RuntimeObjectPackageV1, ShardId, SoftConfirmation,
    WorkItem, WorkPayload,
};
