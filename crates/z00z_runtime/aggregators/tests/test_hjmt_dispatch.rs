mod test_common;

use tempfile::tempdir;
use z00z_aggregators::{
    AggregatorId, BatchPlanner, DispatchStage, DistDispatch, DistNoteKind, DistScheduler, ShardId,
    ShardRouteTable, WorkItem,
};

use self::test_common::{batch_id, claim_item, tx_item};

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;

use hjmt_topology_support::{canonical_five_by_seven, load_cfg, read_route_table, write_home};

#[test]
fn test_waves_follow_topology() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let home = temp.path().join("sim_5a7s");
    write_home(&home, 1, &canonical_five_by_seven(9700));

    let cfg = load_cfg(&home);
    let table = read_route_table(&home);
    let placement = cfg.placement_table().expect("placement table");
    let items = vec![
        route_item("wave-0", &table, 0),
        route_item("wave-1", &table, 1),
        route_item("wave-4", &table, 4),
        route_item("wave-5", &table, 5),
    ];

    let waves = runtime(DistScheduler::new(table, placement).plan_waves(&items))?;
    assert_eq!(waves.len(), 2);
    assert_eq!(waves[0].index, 1);
    assert_eq!(
        wave_owners(&waves[0]),
        vec![
            AggregatorId::new(0),
            AggregatorId::new(1),
            AggregatorId::new(4)
        ]
    );
    assert_eq!(wave_shards(&waves[0]), vec![0, 1, 4]);
    assert_eq!(wave_owners(&waves[1]), vec![AggregatorId::new(0)]);
    assert_eq!(wave_shards(&waves[1]), vec![5]);
    assert_eq!(waves[0].notes[0].kind, DistNoteKind::SchedulerWave);
    assert!(!waves[0].notes[0].proof_truth);

    Ok(())
}

#[test]
fn test_rejects_dispatch_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let home = temp.path().join("sim_5a7s");
    write_home(&home, 1, &canonical_five_by_seven(9710));

    let cfg = load_cfg(&home);
    let table = read_route_table(&home);
    let placement = cfg.placement_table().expect("placement table");
    let lock_path = cfg
        .hjmt
        .as_ref()
        .expect("hjmt config")
        .storage
        .paths
        .lock_path
        .display()
        .to_string();
    let mut dispatch = runtime(DistDispatch::new(table.clone(), placement))?;

    let shard0_a = route_item("dispatch-0a", &table, 0);
    let shard0_b = route_item("dispatch-0b", &table, 0);
    let shard0_c = route_item("dispatch-0c", &table, 0);
    let shard1 = route_item("dispatch-1", &table, 1);

    let wrong_owner = dispatch
        .dispatch_batch(
            batch_id("wrong-owner"),
            std::slice::from_ref(&shard0_a),
            AggregatorId::new(1),
            1,
            1,
            lock_path.clone(),
        )
        .expect_err("wrong owner must reject");
    assert!(wrong_owner.detail.contains("wrong owner"));

    runtime(dispatch.partition(AggregatorId::new(0)))?;
    let unavailable = runtime(dispatch.dispatch_batch(
        batch_id("owner-down"),
        std::slice::from_ref(&shard0_a),
        AggregatorId::new(0),
        1,
        1,
        lock_path.clone(),
    ))?;
    assert_eq!(unavailable.stage, DispatchStage::Deferred);
    assert!(unavailable.detail.contains("owner unavailable"));
    runtime(dispatch.heal(AggregatorId::new(0)))?;

    let mut stale_table = table.clone();
    stale_table.activation_checkpoint += 1;
    stale_table.validate().expect("stale table stays valid");
    let stale_planned = BatchPlanner::new(stale_table)
        .plan_batch(
            batch_id("stale-route-digest"),
            std::slice::from_ref(&shard0_a),
        )
        .expect("stale planned batch");
    let stale_route = dispatch
        .dispatch_planned(stale_planned, AggregatorId::new(0), 1, 1, lock_path.clone())
        .expect_err("stale route digest must reject");
    assert!(stale_route.detail.contains("wrong route digest"));

    let delivered = runtime(dispatch.dispatch_batch(
        batch_id("deliver-1"),
        std::slice::from_ref(&shard0_a),
        AggregatorId::new(0),
        1,
        1,
        lock_path.clone(),
    ))?;
    assert_eq!(delivered.stage, DispatchStage::Delivered);

    let duplicate = runtime(dispatch.dispatch_batch(
        batch_id("deliver-1"),
        std::slice::from_ref(&shard0_a),
        AggregatorId::new(0),
        1,
        1,
        lock_path.clone(),
    ))?;
    assert_eq!(duplicate.stage, DispatchStage::Duplicate);

    let reorder = runtime(dispatch.dispatch_batch(
        batch_id("deliver-2"),
        std::slice::from_ref(&shard0_b),
        AggregatorId::new(0),
        3,
        1,
        lock_path.clone(),
    ))?;
    assert_eq!(reorder.stage, DispatchStage::Deferred);
    assert!(reorder.detail.contains("reorder"));

    let delivered_two = runtime(dispatch.dispatch_batch(
        batch_id("deliver-2"),
        std::slice::from_ref(&shard0_b),
        AggregatorId::new(0),
        2,
        1,
        lock_path.clone(),
    ))?;
    assert_eq!(delivered_two.stage, DispatchStage::Delivered);

    runtime(dispatch.restart(AggregatorId::new(0), 2))?;
    let stale_owner = dispatch
        .dispatch_batch(
            batch_id("stale-owner"),
            std::slice::from_ref(&shard0_c),
            AggregatorId::new(0),
            1,
            1,
            lock_path.clone(),
        )
        .expect_err("stale owner must reject");
    assert!(stale_owner.detail.contains("stale owner"));

    let restarted = runtime(dispatch.dispatch_batch(
        batch_id("deliver-3"),
        std::slice::from_ref(&shard0_c),
        AggregatorId::new(0),
        1,
        2,
        lock_path.clone(),
    ))?;
    assert_eq!(restarted.stage, DispatchStage::Delivered);

    let cross_shard = dispatch
        .dispatch_batch(
            batch_id("cross-shard"),
            &[shard0_a, shard1],
            AggregatorId::new(0),
            2,
            2,
            lock_path,
        )
        .expect_err("cross-shard dispatch must reject");
    assert!(cross_shard
        .detail
        .contains("multi-shard batch admission stays closed"));

    let notes = dispatch.take_notes();
    assert!(notes
        .iter()
        .any(|note| note.kind == DistNoteKind::ShardStall));
    assert!(notes
        .iter()
        .any(|note| note.kind == DistNoteKind::DispatchDispute));
    assert!(notes
        .iter()
        .any(|note| note.kind == DistNoteKind::FailoverState));

    Ok(())
}

#[test]
fn test_rejects_writer_root_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let home = temp.path().join("sim_5a7s");
    write_home(&home, 1, &canonical_five_by_seven(9720));

    let cfg = load_cfg(&home);
    let table = read_route_table(&home);
    let placement = cfg.placement_table().expect("placement table");
    let lock_path = cfg
        .hjmt
        .as_ref()
        .expect("hjmt config")
        .storage
        .paths
        .lock_path
        .display()
        .to_string();
    let mut dispatch = runtime(DistDispatch::new(table.clone(), placement))?;

    let shard0 = route_item("lock-0", &table, 0);
    let shard1 = route_item("lock-1", &table, 1);
    let shard5 = route_item("lock-5", &table, 5);

    let first = runtime(dispatch.dispatch_batch(
        batch_id("lock-a"),
        std::slice::from_ref(&shard0),
        AggregatorId::new(0),
        1,
        1,
        lock_path.clone(),
    ))?;
    assert_eq!(first.stage, DispatchStage::Delivered);

    let concurrent = dispatch
        .dispatch_batch(
            batch_id("lock-b"),
            std::slice::from_ref(&shard1),
            AggregatorId::new(1),
            1,
            1,
            lock_path.clone(),
        )
        .expect_err("concurrent writer must reject");
    assert!(concurrent.detail.contains("concurrent writer"));

    runtime(dispatch.restart(AggregatorId::new(0), 2))?;

    let stale = dispatch
        .dispatch_batch(
            batch_id("lock-c"),
            std::slice::from_ref(&shard0),
            AggregatorId::new(0),
            1,
            1,
            lock_path.clone(),
        )
        .expect_err("stale owner must reject");
    assert!(stale.detail.contains("stale owner"));

    let reacquired = runtime(dispatch.dispatch_batch(
        batch_id("lock-d"),
        std::slice::from_ref(&shard0),
        AggregatorId::new(0),
        1,
        2,
        lock_path.clone(),
    ))?;
    assert_eq!(reacquired.stage, DispatchStage::Delivered);

    let shared_root = dispatch
        .dispatch_batch(
            batch_id("lock-e"),
            std::slice::from_ref(&shard5),
            AggregatorId::new(0),
            2,
            2,
            lock_path,
        )
        .expect_err("shared root hazard must reject");
    assert!(shared_root.detail.contains("shared root hazard"));

    let notes = dispatch.take_notes();
    assert!(notes
        .iter()
        .any(|note| note.kind == DistNoteKind::StorageLockHazard));

    Ok(())
}

fn wave_owners(wave: &z00z_aggregators::SchedulerWave) -> Vec<AggregatorId> {
    wave.batches.iter().map(|batch| batch.owner_id).collect()
}

fn wave_shards(wave: &z00z_aggregators::SchedulerWave) -> Vec<u16> {
    wave.batches
        .iter()
        .map(|batch| batch.planned.route.shard_id.as_u16())
        .collect()
}

fn route_item(label: &str, table: &ShardRouteTable, shard_id: u16) -> WorkItem {
    let target = ShardId::new(shard_id);
    for index in 0..20_000u32 {
        let seed = format!("{label}-{index}");
        let item = if index % 2 == 0 {
            tx_item(&seed)
        } else {
            claim_item(&seed)
        };
        let route_key = route_key(&item);
        if table.lookup(route_key).expect("route lookup") == target {
            return item;
        }
    }
    panic!("missing route item for shard {shard_id}");
}

fn route_key(item: &WorkItem) -> [u8; 32] {
    let raw = hex::decode(item.digest_hex()).expect("digest hex");
    let mut out = [0u8; 32];
    out.copy_from_slice(&raw);
    out
}

fn runtime<T>(
    result: Result<T, z00z_aggregators::RejectRecord>,
) -> Result<T, Box<dyn std::error::Error>> {
    result.map_err(|err| std::io::Error::other(err.detail).into())
}
