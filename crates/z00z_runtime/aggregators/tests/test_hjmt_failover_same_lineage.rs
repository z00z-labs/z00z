mod test_recovery_common;

use tempfile::tempdir;
use z00z_aggregators::{
    AggregatorId, PublicationRecord, PublicationState, RecoveryBoundary, RecoveryIntent,
    SecondaryState, ShardExecState, ShardExecTicket, ShardPlacement,
};
use z00z_storage::checkpoint::CheckpointId;
use z00z_storage::settlement::SettlementRecoveryState;
use z00z_utils::codec::{Codec, JsonCodec};

#[path = "test_hjmt_topology_support.rs"]
mod hjmt_topology_support;

use self::test_recovery_common::{
    batch_id, carry_forward_publication_case, live_failover_manifest, placement_table,
    recovery_record, route, route_bound_recovery_state, FailoverManifest,
};
use hjmt_topology_support::{
    bind_previous_generation, load_cfg, placement_row, read_route_table, set_activation_checkpoint,
    staged_three_by_seven, staged_two_by_seven, write_home,
};

const FOV_001: &str = "FOV-001";
const FAILOVER_FIXTURE: &str = "Failover fixture";

fn manifest_case<'a>(
    manifest: &'a FailoverManifest,
    fixture_id: &str,
    kind: &str,
) -> &'a test_recovery_common::FailoverCase {
    manifest
        .cases
        .iter()
        .find(|case| case.fixture_id == fixture_id && case.kind == kind)
        .expect("manifest case")
}

#[test]
fn test_manifest_matches_contract() -> Result<(), Box<dyn std::error::Error>> {
    let expected: FailoverManifest =
        JsonCodec.deserialize(include_bytes!("fixtures/failover_v1/manifest.json"))?;
    let live = live_failover_manifest()?;
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        let json = JsonCodec.serialize_pretty(&live)?;
        println!("{}", String::from_utf8(json).expect("manifest utf8"));
    }
    assert_eq!(expected, live);
    Ok(())
}

#[test]
fn test_same_lineage_takeover() -> Result<(), Box<dyn std::error::Error>> {
    let route = route(3, 9);
    let recovery = route_bound_recovery_state(0x71, batch_id(FOV_001), route, [0x31; 32])?;
    let primary = AggregatorId::new(7);
    let secondary = SecondaryState::ready(AggregatorId::new(8));
    let table = placement_table(route, primary, vec![secondary], recovery.journal_lineage);
    let record = recovery_record(FOV_001, route, primary, vec![secondary], recovery.clone());
    let encoded = JsonCodec.serialize_pretty(&record)?;
    let decoded: z00z_aggregators::ShardRecoveryRecord = JsonCodec.deserialize(&encoded)?;
    assert_eq!(decoded, record);
    assert_eq!(FAILOVER_FIXTURE, "Failover fixture");

    let resumed = RecoveryBoundary
        .resume(
            AggregatorId::new(8),
            &table,
            &record,
            &recovery,
            RecoveryIntent::TakeoverSecondary,
        )
        .expect("same-lineage takeover must resume");

    assert_eq!(resumed.state, ShardExecState::RecoveryPending);
    assert_eq!(resumed.placement.route, route);
    assert_eq!(resumed.placement.primary_id, AggregatorId::new(8));
    assert_eq!(
        resumed.placement.expected_journal_lineage,
        recovery.journal_lineage
    );
    Ok(())
}

#[test]
fn test_restart_keeps_lineage() -> Result<(), Box<dyn std::error::Error>> {
    let route = route(4, 10);
    let recovery =
        route_bound_recovery_state(0x72, batch_id("primary-restart"), route, [0x32; 32])?;
    let primary = AggregatorId::new(17);
    let secondary = SecondaryState::ready(AggregatorId::new(18));
    let table = placement_table(route, primary, vec![secondary], recovery.journal_lineage);
    let record = recovery_record(
        "primary-restart",
        route,
        primary,
        vec![secondary],
        recovery.clone(),
    );

    let resumed = RecoveryBoundary
        .resume(
            primary,
            &table,
            &record,
            &recovery,
            RecoveryIntent::RestartPrimary,
        )
        .expect("primary restart must resume");

    assert_eq!(resumed.state, ShardExecState::RecoveryPending);
    assert_eq!(resumed.placement.primary_id, primary);
    assert_eq!(
        resumed.placement.expected_journal_lineage,
        recovery.journal_lineage
    );
    Ok(())
}

#[test]
fn test_keeps_takeover_lawful() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let old_home = temp.path().join("old_3a7s");
    let new_home = temp.path().join("new_2a7s");
    write_home(&old_home, 1, &staged_three_by_seven(8700));
    write_home(&new_home, 2, &staged_two_by_seven(8800));
    set_activation_checkpoint(&old_home, 11);
    bind_previous_generation(&new_home, &read_route_table(&old_home));
    set_activation_checkpoint(&new_home, 42);

    let old_table = read_route_table(&old_home);
    let new_table = read_route_table(&new_home);
    new_table
        .validate_migration(&old_table)
        .expect("decommissioned topology must stay generation bound");

    let new_cfg = load_cfg(&new_home);
    let row = placement_row(&new_cfg, 5, 2);
    let secondary = row.secondaries[0];
    let recovery = route_bound_recovery_state(
        0x74,
        batch_id("decommissioned-topology"),
        row.route,
        [0x74; 32],
    )?;
    let recovery = SettlementRecoveryState::new(
        recovery.version,
        recovery.state_root,
        recovery.root_generation,
        recovery.proof_version,
        recovery.bucket_policy_generation,
        recovery.bucket_policy_id,
        row.expected_journal_lineage,
    )
    .with_route(recovery.route.expect("route-bound recovery"));
    let record = recovery_record(
        "decommissioned-topology",
        row.route,
        row.primary_id,
        row.secondaries.clone(),
        recovery.clone(),
    );

    let resumed = RecoveryBoundary
        .resume(
            secondary.aggregator_id,
            &new_cfg.placement_table().expect("placement table"),
            &record,
            &recovery,
            RecoveryIntent::TakeoverSecondary,
        )
        .expect("decommissioned topology secondary-aggregator takeover must stay lawful");

    assert_eq!(resumed.state, ShardExecState::RecoveryPending);
    assert_eq!(resumed.placement.route, row.route);
    assert_eq!(resumed.placement.primary_id, secondary.aggregator_id);
    assert_eq!(
        resumed.placement.expected_journal_lineage,
        row.expected_journal_lineage
    );

    Ok(())
}

#[test]
fn test_publication_handoff_metadata_roundtrips() -> Result<(), Box<dyn std::error::Error>> {
    let route = route(6, 11);
    let recovery =
        route_bound_recovery_state(0x73, batch_id("publication-handoff"), route, [0x33; 32])?;
    let primary = AggregatorId::new(27);
    let secondary = SecondaryState::ready(AggregatorId::new(28));
    let checkpoint_id = CheckpointId::new([0xA5; 32]);
    let placement = ShardPlacement::new(route, primary, vec![secondary], recovery.journal_lineage);
    let ticket = ShardExecTicket {
        batch_id: batch_id("publication-handoff"),
        placement: placement.view(),
        state: ShardExecState::Routed,
    };
    let publication = PublicationRecord {
        batch_id: ticket.batch_id,
        checkpoint_id: Some(checkpoint_id),
        state: PublicationState::Posted,
        da_reference: None,
        publication_evidence: None,
        lifecycle: None,
    };
    let record = RecoveryBoundary
        .capture(&ticket, &publication, recovery.clone())
        .expect("publication handoff capture");
    let encoded = JsonCodec.serialize_pretty(&record)?;
    let decoded: z00z_aggregators::ShardRecoveryRecord = JsonCodec.deserialize(&encoded)?;

    assert_eq!(decoded.checkpoint_id, Some(checkpoint_id));
    assert_eq!(decoded.publication_state, PublicationState::Posted);

    let resumed = RecoveryBoundary
        .resume(
            secondary.aggregator_id,
            &placement_table(route, primary, vec![secondary], recovery.journal_lineage),
            &record,
            &recovery,
            RecoveryIntent::TakeoverSecondary,
        )
        .expect("publication handoff metadata must not block lawful takeover");

    assert_eq!(resumed.batch_id, ticket.batch_id);
    assert_eq!(resumed.state, ShardExecState::RecoveryPending);
    Ok(())
}

#[test]
fn test_fov_g002_carry_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let case = carry_forward_publication_case()?;
    case.later
        .check_monotonic_successor_v1(&case.prior)
        .expect("carry-forward publication must stay monotonic");

    let prior_carried = case
        .prior
        .shard_leaves
        .iter()
        .find(|leaf| leaf.shard_id == case.carried_forward_leaf.shard_id)
        .expect("prior carried-forward leaf");
    let later_carried = case
        .later
        .shard_leaves
        .iter()
        .find(|leaf| leaf.shard_id == case.carried_forward_leaf.shard_id)
        .expect("later carried-forward leaf");

    assert_eq!(
        prior_carried.canonical_bytes()?,
        later_carried.canonical_bytes()?,
        "failed shard leaf must carry forward byte-for-byte"
    );

    let prior_advanced = case
        .prior
        .shard_leaves
        .iter()
        .find(|leaf| leaf.shard_id != case.carried_forward_leaf.shard_id)
        .expect("prior advanced leaf");
    let later_advanced = case
        .later
        .shard_leaves
        .iter()
        .find(|leaf| leaf.shard_id != case.carried_forward_leaf.shard_id)
        .expect("later advanced leaf");
    assert_ne!(
        prior_advanced.canonical_bytes()?,
        later_advanced.canonical_bytes()?,
        "unaffected shard evidence must advance when publication continues"
    );

    let manifest = live_failover_manifest()?;
    let row = manifest_case(
        &manifest,
        "FOV-G-002",
        "failed-shard carry-forward publication accepts",
    );
    let carried_leaf_hex = hex::encode(later_carried.canonical_bytes()?);
    assert_eq!(
        row.carried_forward_leaf_hex.as_deref(),
        Some(carried_leaf_hex.as_str())
    );
    assert_eq!(
        row.expected_public_root_hexes,
        vec![hex::encode(case.later.public_root_v1()?.into_bytes())]
    );

    Ok(())
}
