#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use z00z_storage::settlement::SettlementRecoveryState;

use crate::{
    consensus_store::{ConsensusStore, ConsensusStoreRecord},
    placement::{AggregatorId, ShardPlacementTable, ShardPlacementView},
    shard_exec::{ShardExecState, ShardExecTicket},
    types::{BatchId, BatchRoute, PublicationRecord, PublicationState, RejectClass, RejectRecord},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RecoveryBoundary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryIntent {
    RestartPrimary,
    TakeoverSecondary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardRecoveryRecord {
    pub batch_id: BatchId,
    pub placement: ShardPlacementView,
    pub checkpoint_id: Option<z00z_storage::checkpoint::CheckpointId>,
    pub publication_state: PublicationState,
    pub recovery: SettlementRecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedConsensusRestart {
    pub ticket: ShardExecTicket,
    pub record: ConsensusStoreRecord,
}

impl RecoveryBoundary {
    pub fn mark_handed_off(&self, batch_id: BatchId) -> PublicationRecord {
        PublicationRecord {
            batch_id,
            checkpoint_id: None,
            state: PublicationState::HandedOff,
        }
    }

    pub fn capture(
        &self,
        ticket: &ShardExecTicket,
        publication: &PublicationRecord,
        recovery: SettlementRecoveryState,
    ) -> Result<ShardRecoveryRecord, RejectRecord> {
        if publication.batch_id != ticket.batch_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "recovery capture batch id does not match the shard execution ticket",
            ));
        }

        Ok(ShardRecoveryRecord {
            batch_id: ticket.batch_id,
            placement: ticket.placement.clone(),
            checkpoint_id: publication.checkpoint_id,
            publication_state: publication.state.clone(),
            recovery,
        })
    }

    pub fn resume(
        &self,
        requester: AggregatorId,
        placement_table: &ShardPlacementTable,
        record: &ShardRecoveryRecord,
        current: &SettlementRecoveryState,
        intent: RecoveryIntent,
    ) -> Result<ShardExecTicket, RejectRecord> {
        let Some(live_placement) =
            placement_table.placement_for_shard(record.placement.route.shard_id)
        else {
            return Err(reject(
                RejectClass::PolicyReject,
                "recovery route is not owned by the current placement table",
            ));
        };

        if live_placement.route.routing_generation != record.placement.route.routing_generation {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: placement routing_generation drifted during recovery",
            ));
        }

        if live_placement.primary_id != record.placement.primary_id {
            return Err(reject(
                RejectClass::PolicyReject,
                "split-brain: live primary owner drifted during recovery",
            ));
        }

        if live_placement.expected_journal_lineage != record.placement.expected_journal_lineage
            || live_placement.expected_journal_lineage != current.journal_lineage
            || current.journal_lineage != record.recovery.journal_lineage
        {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong lineage: journal lineage does not match committed recovery state",
            ));
        }

        if current.version != 0 || record.recovery.version != 0 {
            let current_route = current.route.ok_or_else(|| {
                reject(
                    RejectClass::PolicyReject,
                    "wrong shard: live durable recovery state is missing route identity",
                )
            })?;
            let recorded_route = record.recovery.route.ok_or_else(|| {
                reject(
                    RejectClass::PolicyReject,
                    "wrong shard: committed recovery state is missing route identity",
                )
            })?;
            let expected_shard_id = record.placement.route.shard_id.as_u32();
            let expected_generation = record.placement.route.routing_generation;

            if current_route.shard_id() != expected_shard_id
                || current_route.routing_generation() != expected_generation
                || recorded_route.shard_id() != expected_shard_id
                || recorded_route.routing_generation() != expected_generation
            {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "wrong shard: durable recovery route does not match shard placement",
                ));
            }

            if current_route.route_table_digest() != recorded_route.route_table_digest() {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "wrong route digest: durable recovery route digest drifted",
                ));
            }

            if current_route.batch_id() != record.batch_id.into_bytes()
                || recorded_route.batch_id() != record.batch_id.into_bytes()
            {
                return Err(reject(
                    RejectClass::PolicyReject,
                    "stale replay: durable recovery batch id does not match the recovery record",
                ));
            }
        }

        if current.version != record.recovery.version {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale restart: recovery version does not match the live durable state",
            ));
        }

        if current.state_root != record.recovery.state_root {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale local root: recovery state root does not match the live durable state",
            ));
        }

        if current.root_generation != record.recovery.root_generation
            || current.proof_version != record.recovery.proof_version
            || current.bucket_policy_generation != record.recovery.bucket_policy_generation
            || current.bucket_policy_id != record.recovery.bucket_policy_id
        {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale restart: backend generation metadata drifted during recovery",
            ));
        }

        let placement = match intent {
            RecoveryIntent::RestartPrimary => {
                if requester != live_placement.primary_id {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "split-brain: primary restart requester is not the live primary owner",
                    ));
                }
                live_placement.view()
            }
            RecoveryIntent::TakeoverSecondary => {
                if requester == live_placement.primary_id {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "split-brain: takeover cannot keep the live primary executor active",
                    ));
                }

                let Some(secondary) = live_placement.secondary(requester) else {
                    return Err(reject(
                        RejectClass::PolicyReject,
                        "split-brain: takeover requester is not a lawful secondary aggregator",
                    ));
                };
                if !secondary.is_ready {
                    return Err(reject(
                        RejectClass::DeferredRetry,
                        "secondary aggregator down: takeover requester is not ready",
                    ));
                }
                live_placement.view().activate(requester)
            }
        };

        Ok(ShardExecTicket {
            batch_id: record.batch_id,
            placement,
            state: ShardExecState::RecoveryPending,
        })
    }

    pub fn resume_from_store(
        &self,
        requester: AggregatorId,
        placement_table: &ShardPlacementTable,
        current: &SettlementRecoveryState,
        store: &ConsensusStore,
        route: BatchRoute,
        intent: RecoveryIntent,
    ) -> Result<PersistedConsensusRestart, RejectRecord> {
        let record = store.load_route(route).map_err(|err| err.to_reject())?;
        let ticket = self.resume(
            requester,
            placement_table,
            &record.recovery_record,
            current,
            intent,
        )?;
        Ok(PersistedConsensusRestart { ticket, record })
    }
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
