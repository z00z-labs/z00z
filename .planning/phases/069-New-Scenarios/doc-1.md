## Что уже можно и нужно делать в simulation

Сейчас уже можно без devnet: гонять local DA publish/resolve roundtrip с tamper/replay rejection, доказывать route-table codec/migration/tamper, и моделировать same-shard quorum, split-brain, partition, replay, catch-up и standby takeover. Это уже покрыто тестами. (crates/z00z_rollup_node/tests/test_da_local_sim.rs:8) (crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs:57) (crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs:49) (crates/z00z_runtime/aggregators/tests/test_hjmt_dist_journal.rs:33)

Сценарный pipeline тоже частично есть: stage 9 строит `CheckpointExecInput` из `TxPackage`, stage 10 проверяет наличие exec input для publish, stage 11 валидирует committed-state proof перед wallet ownership detection, а `runtime_observability` stage13 уже сверяет `PublicationBinding`, route table digest и public checkpoint proofs. (crates/z00z_simulator/src/scenario_1/stage_9/exec_input_builder.rs:41) (crates/z00z_simulator/src/scenario_1/stage_10/publish_support.rs:18) (crates/z00z_simulator/src/scenario_1/stage_11/jmt_wallet_scan.rs:131) (crates/z00z_simulator/src/scenario_1/runtime_observability.rs:5681)

Следующий правильный шаг в симуляции: сделать один end-to-end harness, который на одном и том же `TxPackage` буквально проходит `IngressBoundary -> BatchPlanner -> ShardPlacementTable -> DistDispatch/Recovery -> LocalDaAdapter -> ResolvedBatch -> ValidatorBoundary`. Это уже не написано как один сквозной live path, но все нужные seams для него в кодовой базе есть.

