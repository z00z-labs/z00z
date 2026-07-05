use tempfile::tempdir;
use z00z_aggregators::{AggregatorId, BatchRoute, ShardId};
use z00z_rollup_node::{NodeConfig, ShardMapping};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io,
};

#[path = "support/test_hjmt_home.rs"]
mod hjmt_test_home;

use hjmt_test_home::{agg, repo_hjmt_home, write_hjmt_home, write_hjmt_home_with_mapping};

const REGEN_CMD: &str = "Z00Z_REGEN_DUMP=1 cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology test_grid57_matches_contract -- --exact --nocapture";
const TEST_CMD: &str = "cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology test_grid57_matches_contract -- --nocapture";
const EVIDENCE_PTR: &str =
    "crates/z00z_rollup_node/tests/test_hjmt_topology.rs::test_grid57_matches_contract";
const REGEN_CMD_77: &str = "Z00Z_REGEN_DUMP=1 cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology test_grid77_matches_contract -- --exact --nocapture";
const TEST_CMD_77: &str = "cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology test_grid77_matches_contract -- --nocapture";
const EVIDENCE_PTR_77: &str =
    "crates/z00z_rollup_node/tests/test_hjmt_topology.rs::test_grid77_matches_contract";

#[test]
fn test_grid57_loads_topology() {
    let cfg = NodeConfig::from_hjmt_home(repo_hjmt_home()).expect("load repo SIM-5A7S home");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    assert_eq!(hjmt.profile, "SIM-5A7S");
    assert_eq!(hjmt.agg_count(), 5);
    assert_eq!(hjmt.shard_count(), 7);
    assert_eq!(hjmt.shard_mapping(), ShardMapping::AggregatorOwned);
    assert!(hjmt.has_dual_primary());
    assert_eq!(
        hjmt.planner.route.table_path.as_deref(),
        Some(std::path::Path::new(
            "shard_route_tables/route-table-v1.canon.hex"
        ))
    );
    assert_eq!(
        hjmt.planner.route.expected_digest.as_deref(),
        Some("000c78634c31e624c5e194378e6c7613e916e1975ca901e5d6416325c1d617e1")
    );

    let table = cfg.placement_table().expect("placement table");
    for shard in 0..7 {
        let route = BatchRoute {
            shard_id: ShardId::new(shard),
            routing_generation: 1,
        };
        let placement = table.placement(route).expect("placement row");
        assert!(!placement.secondaries.is_empty());
        assert_eq!(placement.expected_journal_lineage, [0u8; 32]);
    }

    for aggregator in 0..5 {
        assert!(hjmt.proc(AggregatorId::new(aggregator)).is_some());
    }
}

#[test]
fn test_grid57_matches_contract() {
    let expected: SimFixtureManifest = JsonCodec
        .deserialize(&io::read_file(repo_hjmt_home().join("manifest.json")).expect("manifest"))
        .expect("manifest json");
    let live = live_manifest(
        &repo_hjmt_home(),
        "quorum, term, membership, split-brain stay simulator-proven locally",
        "SIM-5A7S-PUB",
        REGEN_CMD,
        TEST_CMD,
        EVIDENCE_PTR,
    );
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let json = JsonCodec.serialize_pretty(&live).expect("manifest json");
        println!("{}", String::from_utf8(json).expect("manifest utf8"));
    }
    assert_eq!(expected, live);
}

#[test]
fn test_grid77_loads_topology() {
    let home = repo_hjmt_home_77();
    let cfg = NodeConfig::from_hjmt_home(&home).expect("load repo SIM-7A7S home");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    assert_eq!(hjmt.profile, "SIM-7A7S");
    assert_eq!(hjmt.agg_count(), 7);
    assert_eq!(hjmt.shard_count(), 7);
    assert_eq!(hjmt.shard_mapping(), ShardMapping::AggregatorOwned);

    let table = cfg.placement_table().expect("placement table");
    for shard in 0..7 {
        let route = BatchRoute {
            shard_id: ShardId::new(shard),
            routing_generation: 1,
        };
        let placement = table.placement(route).expect("placement row");
        assert_eq!(placement.primary_id.as_u16(), shard);
        assert_eq!(placement.secondaries.len(), 6);
        assert!(placement
            .secondaries
            .iter()
            .all(|secondary| secondary.is_ready));
    }

    for aggregator in 0..7 {
        assert!(hjmt.proc(AggregatorId::new(aggregator)).is_some());
    }
}

#[test]
fn test_grid77_matches_contract() {
    let home = repo_hjmt_home_77();
    let expected: SimFixtureManifest = JsonCodec
        .deserialize(&io::read_file(home.join("manifest.json")).expect("manifest"))
        .expect("manifest json");
    let live = live_manifest(
        &home,
        "3f+1 membership and 2f+1 quorum stay simulator-proven locally",
        "SIM-7A7S-BFT",
        REGEN_CMD_77,
        TEST_CMD_77,
        EVIDENCE_PTR_77,
    );
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let json = JsonCodec.serialize_pretty(&live).expect("manifest json");
        println!("{}", String::from_utf8(json).expect("manifest utf8"));
    }
    assert_eq!(expected, live);
}

#[test]
fn test_accepts_non57_topology() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_3a3s");
    write_hjmt_home(
        &home,
        1,
        &[
            agg(0, 7200, &[(0, &[1, 2]), (1, &[2])]),
            agg(1, 7201, &[]),
            agg(2, 7202, &[(2, &[0])]),
        ],
    );

    let cfg = NodeConfig::from_hjmt_home(&home).expect("load non-5x7 topology");
    let stat = cfg.node_stat().expect("node stat");

    assert_eq!(stat.profile, "SIM-3A3S");
    assert_eq!(stat.agg_count, 3);
    assert_eq!(stat.shard_count, 3);
    assert_eq!(stat.routing_generation, 1);
}

#[test]
fn test_accepts_shard_mapping() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_2a2s_shard_process");
    write_hjmt_home_with_mapping(
        &home,
        1,
        &[agg(0, 7250, &[(0, &[1])]), agg(1, 7251, &[(1, &[0])])],
        Some("shard_process"),
    );

    let cfg = NodeConfig::from_hjmt_home(&home).expect("valid shard_process topology");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    assert_eq!(hjmt.shard_mapping(), ShardMapping::ShardProcess);
    assert!(hjmt.aggs.iter().all(|agg| agg.shards.len() <= 1));
}

#[test]
fn duplicate_primary_owner_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_bad");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7300, &[(0, &[1])]), agg(1, 7301, &[(0, &[0])])],
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("duplicate primary must reject");
    let msg = err.to_string();

    assert!(msg.contains("duplicate primary owner"));
}

#[test]
fn mixed_shard_mapping_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_mixed_mapping");
    write_hjmt_home_with_mapping(
        &home,
        1,
        &[agg(0, 7320, &[(0, &[1])]), agg(1, 7321, &[(1, &[0])])],
        Some("aggregator_owned"),
    );

    let agg_cfg = home.join("aggregators/agg-1/aggregator-config.yaml");
    let body = String::from_utf8(io::read_file(&agg_cfg).expect("read agg cfg")).expect("utf8");
    let body = body.replace("aggregator_owned", "shard_process");
    io::write_file(&agg_cfg, body.as_bytes()).expect("agg cfg");

    let err = NodeConfig::from_hjmt_home(&home).expect_err("mixed mapping must reject");
    assert!(err
        .to_string()
        .contains("must share one execution.shard_mapping"));
}

#[test]
fn test_rejects_mixed_lineage() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_bad_lineage");
    write_hjmt_home(
        &home,
        1,
        &[agg(0, 7350, &[(0, &[1]), (1, &[1])]), agg(1, 7351, &[])],
    );

    let cfg_path = home.join("aggregators/agg-0/aggregator-config.yaml");
    let body = String::from_utf8(io::read_file(&cfg_path).expect("read agg cfg")).expect("utf8");
    let mut seen = 0usize;
    let mut out = String::with_capacity(body.len());
    for line in body.lines() {
        if line.contains("expected_journal_lineage:") {
            if seen == 1 {
                out.push_str(
                    "    expected_journal_lineage: \"1111111111111111111111111111111111111111111111111111111111111111\"\n",
                );
            } else {
                out.push_str(line);
                out.push('\n');
            }
            seen += 1;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    io::write_file(&cfg_path, out.as_bytes()).expect("write agg cfg");

    let err = NodeConfig::from_hjmt_home(&home).expect_err("mixed lineage must reject");
    assert!(err
        .to_string()
        .contains("must share one expected_journal_lineage"));
}

#[test]
fn multi_shard_shard_process_rejects() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_multi_shard_process");
    write_hjmt_home_with_mapping(
        &home,
        1,
        &[agg(0, 7360, &[(0, &[1]), (1, &[1])]), agg(1, 7361, &[])],
        Some("shard_process"),
    );

    let err =
        NodeConfig::from_hjmt_home(&home).expect_err("multi-primary shard_process must reject");
    assert!(err
        .to_string()
        .contains("shard_process mapping allows at most one primary shard per process"));
}

#[test]
fn test_grid57_requires_aggregators() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_5a7s");
    write_hjmt_home(
        &home,
        1,
        &[
            agg(1, 7401, &[(0, &[2, 3]), (5, &[3, 5])]),
            agg(2, 7402, &[(1, &[1, 4])]),
            agg(3, 7403, &[(2, &[1, 5])]),
            agg(4, 7404, &[(3, &[2, 5])]),
            agg(5, 7405, &[(4, &[3, 4]), (6, &[1, 3])]),
        ],
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("SIM-5A7S agg ids must reject");
    assert!(err
        .to_string()
        .contains("SIM-5A7S must declare AggregatorId(0)..AggregatorId(4)"));
}

#[test]
fn test_grid57_requires_shards() {
    let temp = tempdir().expect("tempdir");
    let home = temp.path().join("sim_5a7s");
    write_hjmt_home(
        &home,
        1,
        &[
            agg(0, 7500, &[(1, &[1, 2]), (5, &[2, 4])]),
            agg(1, 7501, &[(2, &[0, 3])]),
            agg(2, 7502, &[(3, &[0, 4])]),
            agg(3, 7503, &[(4, &[1, 4])]),
            agg(4, 7504, &[(6, &[2, 3]), (7, &[0, 2])]),
        ],
    );

    let err = NodeConfig::from_hjmt_home(&home).expect_err("SIM-5A7S shard ids must reject");
    assert!(err
        .to_string()
        .contains("SIM-5A7S must declare ShardId(0)..ShardId(6)"));
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct SimFixtureManifest {
    version: u32,
    profile: String,
    expected_verdict: String,
    process_model: String,
    shard_mapping: String,
    route_table_digest: String,
    route_table_rel_path: String,
    runtime_fixture_home: String,
    runtime_fixture_note: String,
    wallet_boundary: String,
    scope_authority: String,
    dist_sim_mode: String,
    dist_sim_truth: String,
    consensus_truth: String,
    route_rollout_truth: String,
    scheduler_truth: String,
    dispatch_truth: String,
    observability_truth: String,
    adapter_only_register: String,
    planner_config_path: String,
    storage_config_path: String,
    agg_ids: Vec<u16>,
    shard_ids: Vec<u16>,
    placement_rows: Vec<SimPlacementRow>,
    aggregators: Vec<SimAggregatorRow>,
    publication: SimPublicationRow,
    regen_command: String,
    test_command: String,
    evidence_pointer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct SimPlacementRow {
    shard_id: u16,
    primary_aggregator_id: u16,
    secondary_ids: Vec<u16>,
    expected_journal_lineage_hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct SimAggregatorRow {
    aggregator_id: u16,
    process_id: String,
    cfg_path: String,
    listen_addr: String,
    data_dir: String,
    journal_path: String,
    log_path: String,
    start_cmd: String,
    restart_cmd: String,
    shard_ids: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct SimPublicationRow {
    acceptance_profile: String,
    inherits_profile: String,
    topology_status: String,
    public_leaf_count: u16,
    leaf_trace_file: String,
    proof_trace_file: String,
    digest_trace_file: String,
}

fn live_manifest(
    home: &std::path::Path,
    consensus_truth: &str,
    acceptance_profile: &str,
    regen_cmd: &str,
    test_cmd: &str,
    evidence_ptr: &str,
) -> SimFixtureManifest {
    let cfg = NodeConfig::from_hjmt_home(home).expect("load repo HJMT home");
    let hjmt = cfg.hjmt.as_ref().expect("hjmt config");

    let mut shard_ids = hjmt
        .aggs
        .iter()
        .flat_map(|agg| agg.shards.iter().map(|shard| shard.shard_id.as_u16()))
        .collect::<Vec<_>>();
    shard_ids.sort_unstable();
    shard_ids.dedup();

    let mut placement_rows = hjmt
        .aggs
        .iter()
        .flat_map(|agg| {
            agg.shards.iter().map(move |shard| SimPlacementRow {
                shard_id: shard.shard_id.as_u16(),
                primary_aggregator_id: agg.aggregator_id.as_u16(),
                secondary_ids: shard
                    .secondary_ids
                    .iter()
                    .map(|secondary| secondary.as_u16())
                    .collect(),
                expected_journal_lineage_hex: hex::encode(shard.expected_journal_lineage),
            })
        })
        .collect::<Vec<_>>();
    placement_rows.sort_by_key(|row| row.shard_id);

    let mut aggregators = hjmt
        .aggs
        .iter()
        .map(|agg| SimAggregatorRow {
            aggregator_id: agg.aggregator_id.as_u16(),
            process_id: format!("agg-{}", agg.aggregator_id.as_u16()),
            cfg_path: repo_rel(&agg.cfg_path),
            listen_addr: agg.network.listen_addr.clone(),
            data_dir: agg.paths.data_dir.display().to_string(),
            journal_path: agg.paths.journal_path.display().to_string(),
            log_path: agg.paths.log_path.display().to_string(),
            start_cmd: agg.lifecycle.start_cmd.clone(),
            restart_cmd: agg.lifecycle.restart_cmd.clone(),
            shard_ids: agg
                .shards
                .iter()
                .map(|shard| shard.shard_id.as_u16())
                .collect(),
        })
        .collect::<Vec<_>>();
    aggregators.sort_by_key(|agg| agg.aggregator_id);

    SimFixtureManifest {
        version: 1,
        profile: hjmt.profile.clone(),
        expected_verdict: "accept".to_string(),
        process_model: "os_process".to_string(),
        shard_mapping: hjmt.shard_mapping().as_str().to_string(),
        route_table_digest: hjmt
            .planner
            .route
            .expected_digest
            .clone()
            .expect("route digest"),
        route_table_rel_path: hjmt
            .planner
            .route
            .table_path
            .as_ref()
            .expect("route table path")
            .display()
            .to_string(),
        runtime_fixture_home: "config/hjmt_runtime".to_string(),
        runtime_fixture_note: "config/hjmt_runtime stays runtime-owned fixture home".to_string(),
        wallet_boundary: "wallet sees public proofs/API only".to_string(),
        scope_authority: "storage-created scopes stay storage-owned semantic truth".to_string(),
        dist_sim_mode: "deterministic local-network simulator".to_string(),
        dist_sim_truth: "real planner/storage/journal/proof primitives".to_string(),
        consensus_truth: consensus_truth.to_string(),
        route_rollout_truth:
            "checkpoint and process acks, mixed-generation drift, stale digest, and late joiners stay simulator-proven locally".to_string(),
        scheduler_truth:
            "scheduler waves stay shard-owner bound locally and do not overclaim durable throughput before root publication".to_string(),
        dispatch_truth:
            "remote owner delivery, duplicate or reordered frames, restart fencing, and cross-shard fail-closed stay simulator-proven locally".to_string(),
        observability_truth:
            "stalls, freeze, disputes, drift, failover, and storage-lock hazards stay advisory notes and never become proof truth".to_string(),
        adapter_only_register:
            "real transport and chain-network adapters stay adapter-only exclusions".to_string(),
        planner_config_path: repo_rel(&hjmt.planner.cfg_path),
        storage_config_path: repo_rel(&hjmt.storage.cfg_path),
        agg_ids: hjmt
            .aggs
            .iter()
            .map(|agg| agg.aggregator_id.as_u16())
            .collect(),
        shard_ids,
        placement_rows,
        aggregators,
        publication: SimPublicationRow {
            acceptance_profile: acceptance_profile.to_string(),
            inherits_profile: hjmt.profile.clone(),
            topology_status: "canonical_acceptance_fixture".to_string(),
            public_leaf_count: 7,
            leaf_trace_file: "leaf_flow.json".to_string(),
            proof_trace_file: "proof_flow.json".to_string(),
            digest_trace_file: "pub_flow.json".to_string(),
        },
        regen_command: regen_cmd.to_string(),
        test_command: test_cmd.to_string(),
        evidence_pointer: evidence_ptr.to_string(),
    }
}

fn repo_hjmt_home_77() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/hjmt_runtime/sim_7a7s")
}

fn repo_rel(path: &std::path::Path) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root");
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    canon
        .strip_prefix(root)
        .unwrap_or(&canon)
        .display()
        .to_string()
}
