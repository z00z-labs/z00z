#![forbid(unsafe_code)]

use z00z_aggregators::{
    BatchId, PublicationBinding, PublicationRecord, PublishedBatch, ShardExecTicket,
    ShardPlacementView, SoftConfirmation,
};
use z00z_storage::checkpoint::{
    CheckpointDaReferenceV1, CheckpointLifecycleV1, CheckpointPublicationEvidenceV1,
};
use z00z_validators::{ObjectRejectCode, Verdict};

use crate::{
    alerts::{AlertKind, AlertSeverity, AlertSubject},
    da_health::ProviderSignal,
    publication::{PublicationWatch, PublicationWatchErr},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub evidence_key: EvidenceKey,
    pub kind: AlertKind,
    pub severity: AlertSeverity,
    pub subject: AlertSubject,
    pub publication: Option<PublicationRecord>,
    pub published: Option<PublishedBatch>,
    pub soft_confirmation: Option<SoftConfirmation>,
    pub placement: Option<ShardPlacementView>,
    pub exec_ticket: Option<ShardExecTicket>,
    pub verdict: Option<Verdict>,
    pub provider_signal: Option<ProviderSignal>,
}

impl EvidenceRecord {
    #[must_use]
    pub fn runtime_placement(&self) -> Option<&ShardPlacementView> {
        self.exec_ticket
            .as_ref()
            .map(|ticket| &ticket.placement)
            .or(self.placement.as_ref())
    }

    #[must_use]
    pub fn runtime_exec(&self) -> Option<&ShardExecTicket> {
        self.exec_ticket.as_ref()
    }

    #[must_use]
    pub fn publication_binding(&self) -> Option<&PublicationBinding> {
        self.verdict
            .as_ref()
            .and_then(|verdict| verdict.publication.as_ref())
    }

    #[must_use]
    pub fn binding_digest(&self) -> Option<[u8; 32]> {
        self.publication_binding()
            .map(PublicationBinding::binding_digest)
    }

    #[must_use]
    pub fn da_reference(&self) -> Option<&CheckpointDaReferenceV1> {
        self.publication
            .as_ref()
            .and_then(|publication| publication.da_reference.as_ref())
    }

    #[must_use]
    pub fn publication_evidence(&self) -> Option<&CheckpointPublicationEvidenceV1> {
        self.publication
            .as_ref()
            .and_then(|publication| publication.publication_evidence.as_ref())
    }

    #[must_use]
    pub fn lifecycle(&self) -> Option<&CheckpointLifecycleV1> {
        self.publication
            .as_ref()
            .and_then(|publication| publication.lifecycle.as_ref())
    }

    #[must_use]
    pub fn object_reject_codes(&self) -> Vec<ObjectRejectCode> {
        self.verdict
            .as_ref()
            .map(Verdict::object_reject_codes)
            .unwrap_or_default()
    }

    pub fn publication_watch(&self) -> Result<PublicationWatch, PublicationWatchErr> {
        let published = self
            .published
            .as_ref()
            .ok_or(PublicationWatchErr::BatchMismatch)?;
        let publication = self
            .publication
            .as_ref()
            .ok_or(PublicationWatchErr::CheckpointMismatch)?;
        PublicationWatch::try_from_runtime(
            published,
            publication,
            self.verdict.as_ref(),
            self.runtime_placement(),
            self.runtime_exec(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvidenceKey {
    pub batch_id: BatchId,
    pub sequence: u64,
}
