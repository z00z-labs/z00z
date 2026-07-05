#![forbid(unsafe_code)]

use z00z_aggregators::{
    AggregatorService, PublicationRecord, PublishedBatch, ShardExecTicket, ShardExecutor,
    ShardPlacementTable, ShardPlacementView,
};
use z00z_validators::ValidatorService;
use z00z_watchers::{ObservationSnapshot, ProviderSignal, WatcherService};

use crate::{
    config::NodeConfig,
    da::DaAdapter,
    mode::NodeMode,
    status::{ServiceBinding, ServiceBindings, StatusSnapshot},
};

pub struct NodeRuntime<A, V, W, D>
where
    A: AggregatorService,
    V: ValidatorService,
    W: WatcherService,
    D: DaAdapter,
{
    pub config: NodeConfig,
    pub aggregator: Option<A>,
    pub placement_table: Option<ShardPlacementTable>,
    pub shard_executor: Option<ShardExecutor>,
    pub validator: Option<V>,
    pub watcher: Option<W>,
    pub da: D,
    pub last_publication: Option<PublicationRecord>,
    pub last_published: Option<PublishedBatch>,
    pub last_placement: Option<ShardPlacementView>,
    pub last_exec_ticket: Option<ShardExecTicket>,
    pub last_verdict: Option<z00z_validators::Verdict>,
    pub last_provider_signal: Option<ProviderSignal>,
    pub last_observation: Option<ObservationSnapshot>,
}

impl<A, V, W, D> NodeRuntime<A, V, W, D>
where
    A: AggregatorService,
    V: ValidatorService,
    W: WatcherService,
    D: DaAdapter,
{
    pub fn mode(&self) -> NodeMode {
        self.config.mode
    }

    pub fn status(&self) -> StatusSnapshot {
        StatusSnapshot {
            mode: self.mode(),
            topology: self.config.node_stat(),
            services: ServiceBindings {
                aggregator: if self.aggregator.is_some() {
                    ServiceBinding::Attached
                } else {
                    ServiceBinding::Detached
                },
                validator: if self.validator.is_some() {
                    ServiceBinding::Attached
                } else {
                    ServiceBinding::Detached
                },
                watcher: if self.watcher.is_some() {
                    ServiceBinding::Attached
                } else {
                    ServiceBinding::Detached
                },
            },
            publication: self.last_publication.clone(),
            published: self.last_published.clone(),
            placement: canonical_placement(
                self.last_placement.as_ref(),
                self.last_exec_ticket.as_ref(),
            ),
            exec_ticket: self.last_exec_ticket.clone(),
            verdict: self.last_verdict.clone(),
            provider_signal: self.last_provider_signal.clone(),
            observation: self.last_observation.clone(),
        }
    }
}

fn canonical_placement(
    placement: Option<&ShardPlacementView>,
    exec_ticket: Option<&ShardExecTicket>,
) -> Option<ShardPlacementView> {
    exec_ticket
        .map(|ticket| ticket.placement.clone())
        .or_else(|| placement.cloned())
}

#[cfg(test)]
mod tests {
    use z00z_storage::checkpoint::CheckpointDraftId;

    use super::{canonical_placement, NodeRuntime};
    use crate::{NodeConfig, NodeMode};
    use z00z_aggregators::{
        AggregatorId, BatchId, BatchRoute, ShardExecState, ShardExecTicket, ShardId,
        ShardPlacementView,
    };
    use z00z_watchers::{ProviderOutcome, ProviderSignal, ProviderStage};

    #[test]
    fn test_placement_prefers_exec_ticket() {
        let stale = placement_view(1, 3, 7);
        let ticket = ShardExecTicket {
            batch_id: BatchId::new(CheckpointDraftId::new([9u8; 32])),
            placement: placement_view(2, 4, 8),
            state: ShardExecState::Running,
        };

        let placement = canonical_placement(Some(&stale), Some(&ticket)).expect("placement");

        assert_eq!(placement, ticket.placement);
    }

    #[test]
    fn test_last_view_no_ticket() {
        let current = placement_view(3, 5, 9);

        let placement = canonical_placement(Some(&current), None).expect("placement");

        assert_eq!(placement, current);
    }

    #[test]
    fn test_status_projects_topology() {
        let route = crate::config::RouteRef {
            table_path: Some(std::path::PathBuf::from(
                "shard_route_tables/route-table-v1.canon.hex",
            )),
            expected_digest: Some(
                "000c78634c31e624c5e194378e6c7613e916e1975ca901e5d6416325c1d617e1".to_string(),
            ),
        };
        let runtime = NodeRuntime::<DummyAgg, DummyVal, DummyWatch, DummyDa> {
            config: NodeConfig {
                mode: NodeMode::Aggregator,
                da_provider: "local".to_string(),
                rpc_enabled: false,
                hjmt: Some(crate::config::HjmtCfg {
                    home: std::path::PathBuf::from("config/hjmt_runtime/sim_5a7s"),
                    profile: "SIM-2A3S".to_string(),
                    proc_model: crate::config::ProcModel::OsProcess,
                    planner: crate::config::PlanCfg {
                        cfg_path: std::path::PathBuf::from(
                            "config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml",
                        ),
                        mode: crate::config::PlannerMode::Central,
                        routing_generation: 1,
                        route: route.clone(),
                        policy: crate::config::PlanPolicy {
                            shard_local_only: true,
                            reject_cross_shard: true,
                            cadence_ms: 250,
                        },
                        limits: crate::config::PlanLimits {
                            max_batch_ops: 128,
                            max_batch_bytes: 1 << 20,
                        },
                        paths: crate::config::PlanPaths {
                            plan_dir: std::path::PathBuf::from("var/hjmt_runtime/plan"),
                            evidence_dir: std::path::PathBuf::from("var/hjmt_runtime/evidence"),
                        },
                    },
                    storage: crate::config::StoreCfg {
                        cfg_path: std::path::PathBuf::from(
                            "config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml",
                        ),
                        backend: "hjmt".to_string(),
                        generation: 1,
                        paths: crate::config::StorePaths {
                            data_dir: std::path::PathBuf::from("var/hjmt_runtime/store"),
                            journal_dir: std::path::PathBuf::from("var/hjmt_runtime/journal"),
                            export_dir: std::path::PathBuf::from("var/hjmt_runtime/export"),
                            import_dir: std::path::PathBuf::from("var/hjmt_runtime/import"),
                            lock_path: std::path::PathBuf::from("var/hjmt_runtime/store.lock"),
                        },
                        settings: crate::config::StoreSet {
                            flush_each_batch: true,
                            sync_mode: "full".to_string(),
                            compression: "none".to_string(),
                            cache_capacity: 128,
                            lock_timeout_ms: 1000,
                        },
                    },
                    aggs: vec![
                        crate::config::AggProc {
                            cfg_path: std::path::PathBuf::from(
                                "config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml",
                            ),
                            aggregator_id: AggregatorId::new(0),
                            role: "aggregator".to_string(),
                            routing_generation: 1,
                            execution: crate::config::AggExecutionCfg::default(),
                            shards: vec![
                                crate::config::ShardOwn {
                                    shard_id: ShardId::new(0),
                                    secondary_ids: vec![AggregatorId::new(1)],
                                    expected_journal_lineage: [0u8; 32],
                                },
                                crate::config::ShardOwn {
                                    shard_id: ShardId::new(1),
                                    secondary_ids: vec![AggregatorId::new(2)],
                                    expected_journal_lineage: [0u8; 32],
                                },
                            ],
                            network: crate::config::NetCfg {
                                listen_addr: "127.0.0.1:7100".to_string(),
                            },
                            paths: crate::config::AggPaths {
                                data_dir: std::path::PathBuf::from("var/hjmt_runtime/agg-0/data"),
                                journal_path: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-0/journal.redb",
                                ),
                                log_path: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-0/aggregator.log",
                                ),
                            },
                            lifecycle: crate::config::LifeCfg {
                                start_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml --planner-config config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml --storage-config config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml".to_string(),
                                restart_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml --planner-config config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml --storage-config config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml".to_string(),
                            },
                            route: route.clone(),
                            startup: crate::config::StartupCheckCfg {
                                route_codec: true,
                                placement: true,
                                journal_lineage: true,
                                backend_generation: true,
                                proof_codec: true,
                                handoff_ready: true,
                                crypto_tags: true,
                            },
                            evidence: crate::config::EvidenceCfg {
                                config_digest_file: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-0/evidence/config-digests.json",
                                ),
                                preflight_report_file: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-0/evidence/preflight-report.json",
                                ),
                            },
                            limits: crate::config::AggLimits::default(),
                        },
                        crate::config::AggProc {
                            cfg_path: std::path::PathBuf::from(
                                "config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml",
                            ),
                            aggregator_id: AggregatorId::new(1),
                            role: "aggregator".to_string(),
                            routing_generation: 1,
                            execution: crate::config::AggExecutionCfg::default(),
                            shards: vec![crate::config::ShardOwn {
                                shard_id: ShardId::new(2),
                                secondary_ids: vec![AggregatorId::new(0)],
                                expected_journal_lineage: [0u8; 32],
                            }],
                            network: crate::config::NetCfg {
                                listen_addr: "127.0.0.1:7101".to_string(),
                            },
                            paths: crate::config::AggPaths {
                                data_dir: std::path::PathBuf::from("var/hjmt_runtime/agg-1/data"),
                                journal_path: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-1/journal.redb",
                                ),
                                log_path: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-1/aggregator.log",
                                ),
                            },
                            lifecycle: crate::config::LifeCfg {
                                start_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml --planner-config config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml --storage-config config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml".to_string(),
                                restart_cmd: "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config config/hjmt_runtime/sim_5a7s/aggregators/agg-1/aggregator-config.yaml --planner-config config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml --storage-config config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml".to_string(),
                            },
                            route,
                            startup: crate::config::StartupCheckCfg {
                                route_codec: true,
                                placement: true,
                                journal_lineage: true,
                                backend_generation: true,
                                proof_codec: true,
                                handoff_ready: true,
                                crypto_tags: true,
                            },
                            evidence: crate::config::EvidenceCfg {
                                config_digest_file: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-1/evidence/config-digests.json",
                                ),
                                preflight_report_file: std::path::PathBuf::from(
                                    "var/hjmt_runtime/agg-1/evidence/preflight-report.json",
                                ),
                            },
                            limits: crate::config::AggLimits::default(),
                        },
                    ],
                }),
            },
            aggregator: None,
            placement_table: None,
            shard_executor: None,
            validator: None,
            watcher: None,
            da: DummyDa,
            last_publication: None,
            last_published: None,
            last_placement: None,
            last_exec_ticket: None,
            last_verdict: None,
            last_provider_signal: None,
            last_observation: None,
        };

        let status = runtime.status();
        let topology = status.topology.expect("topology");

        assert_eq!(topology.profile, "SIM-2A3S");
        assert_eq!(topology.proc_model, crate::config::ProcModel::OsProcess);
        assert_eq!(topology.agg_count, 2);
        assert_eq!(topology.shard_count, 3);
        assert_eq!(topology.routing_generation, 1);
    }

    #[test]
    fn test_status_keeps_provider_signal() {
        let batch_id = BatchId::from_bytes([0x41; 32]);
        let signal = ProviderSignal {
            provider_name: "local-bridge".to_string(),
            batch_id,
            stage: ProviderStage::Resolve,
            outcome: ProviderOutcome::Success,
            blob_ref: Some("local-da://local-bridge/abcd".to_string()),
        };
        let runtime = NodeRuntime::<DummyAgg, DummyVal, DummyWatch, DummyDa> {
            config: NodeConfig {
                mode: NodeMode::Aggregator,
                da_provider: "local-bridge".to_string(),
                rpc_enabled: false,
                hjmt: None,
            },
            aggregator: None,
            placement_table: None,
            shard_executor: None,
            validator: None,
            watcher: None,
            da: DummyDa,
            last_publication: None,
            last_published: None,
            last_placement: None,
            last_exec_ticket: None,
            last_verdict: None,
            last_provider_signal: Some(signal.clone()),
            last_observation: None,
        };

        let status = runtime.status();

        assert_eq!(status.provider_signal, Some(signal));
    }

    struct DummyAgg;

    impl z00z_aggregators::AggregatorIngress for DummyAgg {
        fn admit(
            &mut self,
            _item: z00z_aggregators::WorkPayload,
        ) -> Result<z00z_aggregators::WorkItem, z00z_aggregators::RejectRecord> {
            unreachable!("not used in status projection")
        }
    }

    impl z00z_aggregators::AggregatorOrdering for DummyAgg {
        fn order(
            &mut self,
            _items: &[z00z_aggregators::WorkItem],
        ) -> Result<z00z_aggregators::OrderedBatch, z00z_aggregators::RejectRecord> {
            unreachable!("not used in status projection")
        }
    }

    impl z00z_aggregators::AggregatorRecovery for DummyAgg {
        fn build_publication(
            &mut self,
            _batch: z00z_aggregators::OrderedBatch,
        ) -> z00z_aggregators::PublicationRequest {
            unreachable!("not used in status projection")
        }

        fn record_publication(
            &mut self,
            _batch: z00z_aggregators::PublishedBatch,
        ) -> z00z_aggregators::PublicationRecord {
            unreachable!("not used in status projection")
        }
    }

    impl z00z_aggregators::AggregatorService for DummyAgg {
        fn emit_soft_confirmation(
            &self,
            _intake_id: &z00z_aggregators::IntakeId,
            _batch_id: &z00z_aggregators::BatchId,
        ) -> z00z_aggregators::SoftConfirmation {
            unreachable!("not used in status projection")
        }
    }

    struct DummyVal;

    impl z00z_validators::ValidatorService for DummyVal {
        fn validate(&mut self, _batch: z00z_validators::ResolvedBatch) -> z00z_validators::Verdict {
            unreachable!("not used in status projection")
        }
    }

    struct DummyWatch;

    impl z00z_watchers::WatcherService for DummyWatch {
        fn observe(
            &mut self,
            _input: z00z_watchers::WatcherInput,
        ) -> z00z_watchers::ObservationSnapshot {
            unreachable!("not used in status projection")
        }

        fn alerts(&self) -> &[z00z_watchers::WatcherAlert] {
            unreachable!("not used in status projection")
        }
    }

    struct DummyDa;

    impl crate::da::DaAdapter for DummyDa {
        fn publish(
            &mut self,
            _request: z00z_aggregators::PublicationRequest,
        ) -> Result<z00z_aggregators::PublishedBatch, crate::da::DaError> {
            unreachable!("not used in status projection")
        }

        fn resolve(
            &mut self,
            _batch: &z00z_aggregators::PublishedBatch,
        ) -> Result<z00z_validators::ResolvedBatch, crate::da::DaError> {
            unreachable!("not used in status projection")
        }
    }

    fn placement_view(shard: u16, generation: u64, aggregator: u16) -> ShardPlacementView {
        ShardPlacementView {
            route: BatchRoute {
                shard_id: ShardId::new(shard),
                routing_generation: generation,
            },
            primary_id: AggregatorId::new(aggregator),
            secondaries: Vec::new(),
            expected_journal_lineage: [0u8; 32],
        }
    }
}
