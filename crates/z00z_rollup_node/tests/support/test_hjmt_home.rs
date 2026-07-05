use std::path::{Path, PathBuf};

use z00z_aggregators::{RouteRangeRule, ShardId, ShardRouteTable};
use z00z_rollup_node::canonical_run_cmd;
use z00z_utils::io;

const ZERO_LINEAGE: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const ROUTE_FILE: &str = "shard_route_tables/route-table-v1.canon.hex";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestAgg {
    id: u16,
    port: u16,
    shards: Vec<TestShard>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TestShard {
    shard_id: u16,
    secondary_ids: Vec<u16>,
}

pub fn agg(id: u16, port: u16, shards: &[(u16, &[u16])]) -> TestAgg {
    TestAgg {
        id,
        port,
        shards: shards
            .iter()
            .map(|(shard_id, secondary_ids)| TestShard {
                shard_id: *shard_id,
                secondary_ids: secondary_ids.to_vec(),
            })
            .collect(),
    }
}

pub fn repo_hjmt_home() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/hjmt_runtime/sim_5a7s")
}

pub fn write_hjmt_home(home: &Path, routing_generation: u64, aggs: &[TestAgg]) {
    write_hjmt_home_with_mapping(home, routing_generation, aggs, None);
}

pub fn write_hjmt_home_with_mapping(
    home: &Path,
    routing_generation: u64,
    aggs: &[TestAgg],
    shard_mapping: Option<&str>,
) {
    let planner_dir = home.join("planner");
    let storage_dir = home.join("storage");
    let agg_root = home.join("aggregators");
    let route_dir = home.join("shard_route_tables");
    io::create_dir_all(&planner_dir).expect("planner dir");
    io::create_dir_all(&storage_dir).expect("storage dir");
    io::create_dir_all(&agg_root).expect("agg root");
    io::create_dir_all(&route_dir).expect("route dir");

    let table = build_route_table(routing_generation, aggs);
    let route_hex = hex::encode(table.canonical_bytes());
    let route_digest = hex::encode(table.digest().as_bytes());
    let route_path = route_dir.join("route-table-v1.canon.hex");
    io::write_file(&route_path, route_hex.as_bytes()).expect("route table");

    let planner_cfg = planner_dir.join("planner-config.yaml");
    let storage_cfg = storage_dir.join("storage-config.yaml");
    io::write_file(
        &planner_cfg,
        format!(
            "mode: central\nrouting_generation: {routing_generation}\nroute:\n  table_path: \"{ROUTE_FILE}\"\n  expected_digest: \"{route_digest}\"\npolicy:\n  shard_local_only: true\n  reject_cross_shard: true\n  cadence_ms: 250\nlimits:\n  max_batch_ops: 128\n  max_batch_bytes: 1048576\npaths:\n  plan_dir: \"{}\"\n  evidence_dir: \"{}\"\n",
            home.join("var/planner/runtime").display(),
            home.join("var/planner/evidence").display(),
        )
        .as_bytes(),
    )
    .expect("planner cfg");
    io::write_file(
        &storage_cfg,
        format!(
            "backend: hjmt\ngeneration: 1\npaths:\n  data_dir: \"{}\"\n  journal_dir: \"{}\"\n  export_dir: \"{}\"\n  import_dir: \"{}\"\n  lock_path: \"{}\"\nsettings:\n  flush_each_batch: true\n  sync_mode: full\n  compression: none\n  cache_capacity: 1024\n  lock_timeout_ms: 1000\n",
            home.join("var/storage/data").display(),
            home.join("var/storage/journal").display(),
            home.join("var/storage/export").display(),
            home.join("var/storage/import").display(),
            home.join("var/storage/storage.lock").display(),
        )
        .as_bytes(),
    )
    .expect("storage cfg");

    for agg in aggs {
        let agg_dir = agg_root.join(format!("agg-{}", agg.id));
        io::create_dir_all(&agg_dir).expect("agg dir");
        let agg_cfg = agg_dir.join("aggregator-config.yaml");
        let shards = agg
            .shards
            .iter()
            .map(|shard| {
                format!(
                    "  - shard_id: {}\n    secondary_ids: [{}]\n    expected_journal_lineage: \"{ZERO_LINEAGE}\"\n",
                    shard.shard_id,
                    shard
                        .secondary_ids
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect::<String>();
        let execution_block = shard_mapping.map_or_else(String::new, |mapping| {
            format!("execution:\n  shard_mapping: \"{mapping}\"\n")
        });
        io::write_file(
            &agg_cfg,
            format!(
                "aggregator_id: {}\nrole: \"aggregator\"\nrouting_generation: {routing_generation}\n{}shards:\n{}network:\n  listen_addr: \"127.0.0.1:{}\"\npaths:\n  data_dir: \"{}\"\n  journal_path: \"{}\"\n  log_path: \"{}\"\nlifecycle:\n  start_cmd: \"{}\"\n  restart_cmd: \"{}\"\nroute:\n  table_path: \"{ROUTE_FILE}\"\n  expected_digest: \"{route_digest}\"\nstartup:\n  route_codec: true\n  placement: true\n  journal_lineage: true\n  backend_generation: true\n  proof_codec: true\n  handoff_ready: true\n  crypto_tags: true\nevidence:\n  config_digest_file: \"{}\"\n  preflight_report_file: \"{}\"\nlimits:\n  max_batch_ops: 128\n  max_inflight: 16\n",
                agg.id,
                execution_block,
                shards,
                agg.port,
                home.join(format!("var/agg-{}/data", agg.id)).display(),
                home.join(format!("var/agg-{}/journal.redb", agg.id)).display(),
                home.join(format!("var/agg-{}/aggregator.log", agg.id)).display(),
                canonical_run_cmd(&agg_cfg, &planner_cfg, &storage_cfg),
                canonical_run_cmd(&agg_cfg, &planner_cfg, &storage_cfg),
                home.join(format!("var/agg-{}/evidence/config-digests.json", agg.id))
                    .display(),
                home.join(format!("var/agg-{}/evidence/preflight-report.json", agg.id))
                    .display(),
            )
            .as_bytes(),
        )
        .expect("agg cfg");
    }
}

fn build_route_table(routing_generation: u64, aggs: &[TestAgg]) -> ShardRouteTable {
    let mut shard_set = aggs
        .iter()
        .flat_map(|agg| agg.shards.iter().map(|shard| ShardId::new(shard.shard_id)))
        .collect::<Vec<_>>();
    shard_set.sort();
    shard_set.dedup();
    assert!(
        !shard_set.is_empty(),
        "route table requires at least one shard"
    );

    let shard_count = shard_set.len() as u16;
    let mut rules = Vec::with_capacity(shard_set.len());
    for (index, shard_id) in shard_set.iter().copied().enumerate() {
        let start_byte = ((index as u16) * 256 / shard_count) as u8;
        let end_byte = (((index as u16 + 1) * 256) / shard_count - 1) as u8;
        let mut start = [0u8; 32];
        let mut end = [0xffu8; 32];
        start[0] = start_byte;
        end[0] = end_byte;
        rules.push(RouteRangeRule::new(start, end, shard_id));
    }

    let previous_generation_digest =
        (routing_generation > 0).then_some(ShardRouteTable::default().digest());
    let table = ShardRouteTable {
        routing_generation,
        shard_set,
        rules,
        previous_generation_digest,
        activation_checkpoint: 0,
    };
    table.validate().expect("route table");
    table
}
