#![forbid(unsafe_code)]

use z00z_aggregators::{
    BatchRoute, PublicationBinding, PublicationReadinessErr, PublicationRecord, PublishedBatch,
    ShardExecTicket, ShardPlacementView,
};
use z00z_storage::checkpoint::{
    CheckpointDaReferenceV1, CheckpointLifecycleV1, CheckpointPublicationEvidenceV1,
};
use z00z_storage::settlement::{check_route_binding_v1, PublicationRouteSnapshotV1};
use z00z_validators::{Verdict, VerdictKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationWatchErr {
    MissingVerdict,
    MissingBinding,
    BatchMismatch,
    CheckpointMismatch,
    BindingMismatch,
    RouteMismatch,
    ExecMismatch,
    ReadinessMismatch,
}

impl PublicationWatchErr {
    #[must_use]
    pub const fn is_validator_incomplete(&self) -> bool {
        matches!(self, Self::MissingVerdict | Self::MissingBinding)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationWatch {
    /// Runtime-owned publication binding reused by watcher-side local evidence.
    pub publication: PublicationBinding,
    /// Storage-owned route snapshot; watchers do not invent a second route truth.
    pub publication_route: PublicationRouteSnapshotV1,
    pub da_reference: Option<CheckpointDaReferenceV1>,
    pub publication_evidence: Option<CheckpointPublicationEvidenceV1>,
    pub lifecycle: Option<CheckpointLifecycleV1>,
    pub publication_state: z00z_aggregators::PublicationState,
    pub verdict_kind: VerdictKind,
    pub runtime_route: Option<BatchRoute>,
}

impl PublicationWatch {
    /// Build one canonical local-publication witness from runtime, validator,
    /// and storage surfaces without upgrading the DA adapter boundary into
    /// semantic authority.
    pub fn try_from_runtime(
        published: &PublishedBatch,
        publication: &PublicationRecord,
        verdict: Option<&Verdict>,
        placement: Option<&ShardPlacementView>,
        exec_ticket: Option<&ShardExecTicket>,
    ) -> Result<Self, PublicationWatchErr> {
        let verdict = verdict.ok_or(PublicationWatchErr::MissingVerdict)?;
        let binding = verdict
            .publication
            .as_ref()
            .ok_or(PublicationWatchErr::MissingBinding)?;
        if verdict.batch_id != published.batch_id
            || publication.batch_id != published.batch_id
            || binding.batch_id() != published.batch_id
        {
            return Err(PublicationWatchErr::BatchMismatch);
        }
        if verdict.checkpoint_id != Some(published.checkpoint_id)
            || publication.checkpoint_id != Some(published.checkpoint_id)
            || binding.checkpoint_id() != published.checkpoint_id
        {
            return Err(PublicationWatchErr::CheckpointMismatch);
        }
        if !binding.matches_pub_in(&published.pub_in) {
            return Err(PublicationWatchErr::BindingMismatch);
        }
        if let Some(exec_ticket) = exec_ticket {
            if exec_ticket.batch_id != published.batch_id {
                return Err(PublicationWatchErr::ExecMismatch);
            }
        }
        publication
            .validate_readiness_bundle(published.checkpoint_id)
            .map_err(map_readiness_err)?;
        let runtime_route = exec_ticket
            .map(|ticket| ticket.placement.route)
            .or_else(|| placement.map(|item| item.route));
        check_route_binding_v1(
            &published.publication_route,
            binding.route_table_digest(),
            Some(published.publication_checkpoint),
            runtime_route.map(|route| (route.shard_id.as_u32(), route.routing_generation)),
        )
        .map_err(|_| PublicationWatchErr::RouteMismatch)?;

        Ok(Self {
            publication: binding.clone(),
            publication_route: published.publication_route.clone(),
            da_reference: publication.da_reference.clone(),
            publication_evidence: publication.publication_evidence.clone(),
            lifecycle: publication.lifecycle.clone(),
            publication_state: publication.state.clone(),
            verdict_kind: verdict.kind.clone(),
            runtime_route,
        })
    }
}

fn map_readiness_err(_err: PublicationReadinessErr) -> PublicationWatchErr {
    PublicationWatchErr::ReadinessMismatch
}
