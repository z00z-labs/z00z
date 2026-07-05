#![forbid(unsafe_code)]

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, BatchRoute, PublicationBinding,
    RejectClass as AggregatorRejectClass, ShardQuorumCertificate,
};
use z00z_storage::{
    checkpoint::derive_checkpoint_id,
    settlement::{check_route_binding_v1, PublicationRouteSnapshotV1},
};

use crate::{RejectClass, ResolvedBatch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointFlow {
    pub publication: PublicationBinding,
    pub publication_route: PublicationRouteSnapshotV1,
    pub ordered_route: BatchRoute,
    pub runtime_route: Option<BatchRoute>,
}

impl CheckpointFlow {
    pub fn try_from_resolved(batch: &ResolvedBatch) -> Result<Self, RejectClass> {
        if batch.published.batch_id != batch.ordered.batch_id
            || batch.ordered.batch_id != batch.ordered.planned.batch_id
        {
            return Err(RejectClass::ReconcileInvalid);
        }

        let checkpoint_id =
            derive_checkpoint_id(batch.artifact()).map_err(|_| RejectClass::ArtifactVersion)?;
        if batch.published.checkpoint_id != checkpoint_id {
            return Err(RejectClass::ReconcileInvalid);
        }
        if batch.link().checkpoint_id() != batch.published.checkpoint_id {
            return Err(RejectClass::ReconcileInvalid);
        }

        if batch.published.pub_in != batch.artifact().pub_in() {
            return Err(RejectClass::StateRootMismatch);
        }

        if let Some(exec_ticket) = batch.runtime_exec() {
            if exec_ticket.batch_id != batch.published.batch_id {
                return Err(RejectClass::ReconcileInvalid);
            }
        }

        let runtime_route = batch.runtime_placement().map(|placement| placement.route);
        if let Some(route) = runtime_route {
            if route != batch.ordered.planned.route {
                return Err(RejectClass::ReconcileInvalid);
            }
        }
        check_route_binding_v1(
            &batch.published.publication_route,
            batch.ordered.planned.route_table_digest.into_bytes(),
            Some(batch.published.publication_checkpoint),
            Some((
                batch.ordered.planned.route.shard_id.as_u32(),
                batch.ordered.planned.route.routing_generation,
            )),
        )
        .map_err(|_| RejectClass::ReconcileInvalid)?;

        let flow = Self {
            publication: bind_publication_contract(
                batch.published.batch_id,
                batch.published.checkpoint_id,
                batch.ordered.planned.route_table_digest.into_bytes(),
                &batch.published.pub_in,
            ),
            publication_route: batch.published.publication_route.clone(),
            ordered_route: batch.ordered.planned.route,
            runtime_route,
        };
        if batch.quorum_binding_enabled() {
            flow.verify_quorum_binding(batch)?;
        }
        Ok(flow)
    }

    #[must_use]
    pub const fn binding_digest(&self) -> [u8; 32] {
        self.publication.binding_digest()
    }

    fn verify_quorum_binding(&self, batch: &ResolvedBatch) -> Result<(), RejectClass> {
        let subject = batch.subject.as_ref().ok_or(RejectClass::AuthInvalid)?;
        let certificate = batch.certificate.as_ref().ok_or(RejectClass::AuthInvalid)?;
        let published_subject_digest = batch
            .published
            .subject_digest
            .ok_or(RejectClass::AuthInvalid)?;
        let published_certificate_digest = batch
            .published
            .certificate_digest
            .ok_or(RejectClass::AuthInvalid)?;
        let published_theorem_digest = batch
            .published
            .theorem_digest
            .ok_or(RejectClass::ProofInvalid)?;

        subject
            .verify_binding(&batch.ordered, &self.publication, batch.theorem_digest())
            .map_err(map_aggregator_reject)?;

        if published_subject_digest != subject.digest()
            || certificate.subject_digest != subject.digest()
            || published_certificate_digest != certificate.digest()
        {
            return Err(RejectClass::ReconcileInvalid);
        }
        if published_theorem_digest != batch.theorem_digest()
            || subject.theorem_or_settlement_digest != batch.theorem_digest()
        {
            return Err(RejectClass::ProofInvalid);
        }
        certificate
            .verify_subject(subject)
            .map_err(map_aggregator_reject)?;

        let placement = batch.runtime_placement().ok_or(RejectClass::AuthInvalid)?;
        if placement.route != subject.route()
            || placement.expected_journal_lineage != subject.journal_lineage
        {
            return Err(RejectClass::ReconcileInvalid);
        }

        let ready_secondaries = placement
            .secondaries
            .iter()
            .filter(|secondary| secondary.is_ready)
            .map(|secondary| secondary.aggregator_id)
            .collect::<Vec<_>>();
        let expected_membership = membership_digest_for_voters(
            placement.route,
            placement.primary_id,
            ready_secondaries.iter().copied(),
        );
        if expected_membership != subject.membership_digest {
            return Err(RejectClass::ReconcileInvalid);
        }

        let rebuilt = match certificate.quorum_rule {
            z00z_aggregators::QuorumRule::MajorityCft => ShardQuorumCertificate::new(
                subject,
                placement.primary_id,
                ready_secondaries.clone(),
                &certificate.votes,
            ),
            z00z_aggregators::QuorumRule::BftTwoFPlusOne => ShardQuorumCertificate::new_bft(
                subject,
                placement.primary_id,
                ready_secondaries.clone(),
                &certificate.votes,
            ),
        }
        .map_err(map_aggregator_reject)?;
        if rebuilt != *certificate {
            return Err(RejectClass::ReconcileInvalid);
        }

        Ok(())
    }
}

fn map_aggregator_reject(reject: z00z_aggregators::RejectRecord) -> RejectClass {
    match reject.class {
        AggregatorRejectClass::AuthInvalid => RejectClass::AuthInvalid,
        AggregatorRejectClass::ShapeInvalid => RejectClass::ShapeInvalid,
        AggregatorRejectClass::DeferredRetry => RejectClass::ReconcileInvalid,
        AggregatorRejectClass::ReplayLocal | AggregatorRejectClass::PolicyReject => {
            RejectClass::ReconcileInvalid
        }
        AggregatorRejectClass::ParseInvalid => RejectClass::ArtifactVersion,
    }
}
