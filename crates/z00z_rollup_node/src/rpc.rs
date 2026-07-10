#![forbid(unsafe_code)]

use std::net::SocketAddr;

use z00z_validators::ObjectRejectCode;

use crate::{mode::NodeMode, status::StatusSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcState {
    pub mode: NodeMode,
    pub listen_addr: SocketAddr,
    pub last_status: Option<StatusSnapshot>,
}

impl RpcState {
    #[must_use]
    pub fn object_reject_codes(&self) -> Vec<ObjectRejectCode> {
        self.last_status
            .as_ref()
            .map(StatusSnapshot::object_reject_codes)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};

    use super::RpcState;
    use crate::{
        mode::NodeMode,
        status::{ServiceBinding, ServiceBindings, StatusSnapshot},
    };
    use z00z_aggregators::BatchId;
    use z00z_core::assets::ObjectFamily;
    use z00z_storage::settlement::{SettlementActionV1, SettlementStateRoot, VoucherAction};
    use z00z_validators::{
        ObjectRejectCode, ObjectValidatorVerdict, RejectClass, Verdict, VerdictKind,
    };

    #[test]
    fn rpc_projects_reject_codes() {
        let rpc = RpcState {
            mode: NodeMode::Watcher,
            listen_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 9000)),
            last_status: Some(StatusSnapshot {
                mode: NodeMode::Watcher,
                topology: None,
                services: ServiceBindings {
                    aggregator: ServiceBinding::Detached,
                    validator: ServiceBinding::Detached,
                    watcher: ServiceBinding::Attached,
                },
                publication: None,
                published: None,
                placement: None,
                exec_ticket: None,
                verdict: Some(Verdict {
                    batch_id: BatchId::from_bytes([0x51; 32]),
                    checkpoint_id: None,
                    publication: None,
                    kind: VerdictKind::Rejected,
                    reject: Some(RejectClass::PolicyUnknown),
                    object_verdicts: vec![ObjectValidatorVerdict {
                        family: ObjectFamily::Voucher,
                        selected_action: SettlementActionV1::Voucher(VoucherAction::RedeemFull),
                        policy_descriptor_hash: [0x31; 32],
                        action_pool_id: [0x32; 32],
                        selected_action_id: [0x33; 32],
                        prior_root: SettlementStateRoot::settlement_v1([0x41; 32]),
                        expected_new_root: SettlementStateRoot::settlement_v1([0x42; 32]),
                        reject: Some(ObjectRejectCode::UnknownPolicy),
                    }],
                }),
                lifecycle: None,
                provider_signal: None,
                observation: None,
            }),
        };

        assert_eq!(
            rpc.object_reject_codes(),
            vec![ObjectRejectCode::UnknownPolicy]
        );
    }
}
