use std::path::PathBuf;

use z00z_rollup_node::{NodeConfig, ProcModel, ShardMapping};

#[test]
fn test_projects_topology() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo SIM-5A7S home");
    let stat = cfg.node_stat().expect("node stat");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    assert_eq!(stat.profile, "SIM-5A7S");
    assert_eq!(stat.proc_model, ProcModel::OsProcess);
    assert_eq!(stat.agg_count, 5);
    assert_eq!(stat.shard_count, 7);
    assert_eq!(stat.routing_generation, 1);
    assert_eq!(hjmt.shard_mapping(), ShardMapping::AggregatorOwned);
    assert_eq!(cfg.mode, z00z_rollup_node::NodeMode::Aggregator);
    assert!(!cfg.rpc_enabled);
    assert_eq!(cfg.da_provider, "not_configured");
    assert!(cfg.placement_table().is_some());

    for agg in &hjmt.aggs {
        assert!(agg
            .lifecycle
            .start_cmd
            .contains("cargo run --release -p z00z_rollup_node"));
        assert!(agg.lifecycle.start_cmd.contains("aggregator-config.yaml"));
        assert!(agg
            .lifecycle
            .restart_cmd
            .contains("cargo run --release -p z00z_rollup_node"));
        assert!(agg.lifecycle.restart_cmd.contains("aggregator-config.yaml"));
    }
}

fn repo_hjmt_home() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/hjmt_runtime/sim_5a7s")
}
