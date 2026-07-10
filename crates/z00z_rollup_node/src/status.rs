#![forbid(unsafe_code)]

use z00z_aggregators::{PublicationRecord, PublishedBatch, ShardExecTicket, ShardPlacementView};
use z00z_storage::checkpoint::CheckpointLifecycleV1;
use z00z_validators::{ObjectRejectCode, ObjectValidatorVerdict, Verdict};
use z00z_watchers::{ObservationSnapshot, ProviderSignal};

use crate::config::NodeStat;
use crate::mode::NodeMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub mode: NodeMode,
    pub topology: Option<NodeStat>,
    pub services: ServiceBindings,
    pub publication: Option<PublicationRecord>,
    pub published: Option<PublishedBatch>,
    pub placement: Option<ShardPlacementView>,
    pub exec_ticket: Option<ShardExecTicket>,
    pub verdict: Option<Verdict>,
    pub lifecycle: Option<CheckpointLifecycleV1>,
    pub provider_signal: Option<ProviderSignal>,
    pub observation: Option<ObservationSnapshot>,
}

impl StatusSnapshot {
    #[must_use]
    pub fn object_verdicts(&self) -> Option<&[ObjectValidatorVerdict]> {
        self.verdict
            .as_ref()
            .map(|item| item.object_verdicts.as_slice())
    }

    #[must_use]
    pub fn object_reject_codes(&self) -> Vec<ObjectRejectCode> {
        self.verdict
            .as_ref()
            .map(Verdict::object_reject_codes)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceBindings {
    pub aggregator: ServiceBinding,
    pub validator: ServiceBinding,
    pub watcher: ServiceBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceBinding {
    Attached,
    Detached,
}

#[cfg(test)]
mod tests {
    use super::{ServiceBinding, ServiceBindings, StatusSnapshot};
    use crate::mode::NodeMode;
    use z00z_aggregators::BatchId;
    use z00z_core::assets::ObjectFamily;
    use z00z_storage::settlement::{SettlementActionV1, SettlementStateRoot, VoucherAction};
    use z00z_validators::{
        ObjectRejectCode, ObjectValidatorVerdict, RejectClass, Verdict, VerdictKind,
    };

    #[test]
    fn status_projects_object_reject_codes() {
        let snapshot = StatusSnapshot {
            mode: NodeMode::Validator,
            topology: None,
            services: ServiceBindings {
                aggregator: ServiceBinding::Detached,
                validator: ServiceBinding::Attached,
                watcher: ServiceBinding::Detached,
            },
            publication: None,
            published: None,
            placement: None,
            exec_ticket: None,
            verdict: Some(Verdict {
                batch_id: BatchId::from_bytes([0x41; 32]),
                checkpoint_id: None,
                publication: None,
                kind: VerdictKind::Rejected,
                reject: Some(RejectClass::ProofInvalid),
                object_verdicts: vec![ObjectValidatorVerdict {
                    family: ObjectFamily::Voucher,
                    selected_action: SettlementActionV1::Voucher(VoucherAction::RedeemFull),
                    policy_descriptor_hash: [0x11; 32],
                    action_pool_id: [0x12; 32],
                    selected_action_id: [0x13; 32],
                    prior_root: SettlementStateRoot::settlement_v1([0x21; 32]),
                    expected_new_root: SettlementStateRoot::settlement_v1([0x22; 32]),
                    reject: Some(ObjectRejectCode::FeeBoundary),
                }],
            }),
            lifecycle: None,
            provider_signal: None,
            observation: None,
        };

        assert_eq!(
            snapshot.object_reject_codes(),
            vec![ObjectRejectCode::FeeBoundary]
        );
        assert_eq!(snapshot.object_verdicts().map(|items| items.len()), Some(1));
    }
}
