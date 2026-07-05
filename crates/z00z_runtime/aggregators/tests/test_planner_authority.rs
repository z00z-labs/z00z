mod test_common;

use sha2::{Digest, Sha256};
use z00z_aggregators::{
    BatchPlanner, PlanDigest, PlannerAuthority, PlannerMode, RejectClass, ShardId,
};

use self::test_common::{batch_id, claim_item, span_table, tx_item};

fn cfg_digest(seed: &str) -> PlanDigest {
    PlanDigest::new(Sha256::digest(seed.as_bytes()).into())
}

#[test]
fn identical_authority_recomputes_identical_plan() {
    let items = vec![
        tx_item("authority-a"),
        claim_item("authority-b"),
        tx_item("authority-c"),
    ];
    let table = span_table(&items, ShardId::new(1));
    let batch_id = batch_id("authority-match");
    let planner_cfg_digest = cfg_digest("planner.mode=central");
    let authority = PlannerAuthority::bind(PlannerMode::Central, &table, planner_cfg_digest);
    let claimed = BatchPlanner::new(table.clone())
        .plan_batch(batch_id, &items)
        .expect("primary planned batch");

    let verified = authority
        .verify_batch(
            PlannerMode::Central,
            &table,
            planner_cfg_digest,
            batch_id,
            &items,
            &claimed,
        )
        .expect("authority verified");

    assert_eq!(verified, claimed);
    assert_eq!(
        authority.digest(),
        PlannerAuthority::bind(PlannerMode::Central, &table, planner_cfg_digest).digest()
    );
}

#[test]
fn planner_config_drift_rejects() {
    let items = vec![tx_item("config-a"), claim_item("config-b")];
    let table = span_table(&items, ShardId::new(1));
    let batch_id = batch_id("config-drift");
    let planner_cfg_digest = cfg_digest("planner.mode=central");
    let authority = PlannerAuthority::bind(PlannerMode::Central, &table, planner_cfg_digest);
    let claimed = BatchPlanner::new(table.clone())
        .plan_batch(batch_id, &items)
        .expect("primary planned batch");

    let err = authority
        .verify_batch(
            PlannerMode::Central,
            &table,
            cfg_digest("planner.mode=per_agg"),
            batch_id,
            &items,
            &claimed,
        )
        .expect_err("planner config drift must reject");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("planner config drift"));
}

#[test]
fn stale_route_digest_rejects() {
    let items = vec![tx_item("route-a"), claim_item("route-b")];
    let table = span_table(&items, ShardId::new(1));
    let batch_id = batch_id("route-drift");
    let planner_cfg_digest = cfg_digest("planner.mode=central");
    let authority = PlannerAuthority::bind(PlannerMode::Central, &table, planner_cfg_digest);
    let claimed = BatchPlanner::new(table.clone())
        .plan_batch(batch_id, &items)
        .expect("primary planned batch");

    let mut drifted_table = table.clone();
    drifted_table.activation_checkpoint += 1;
    drifted_table.validate().expect("drifted table stays valid");

    let err = authority
        .verify_batch(
            PlannerMode::Central,
            &drifted_table,
            planner_cfg_digest,
            batch_id,
            &items,
            &claimed,
        )
        .expect_err("stale route digest must reject");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("stale route-table digest"));
}

#[test]
fn mixed_planner_generation_rejects() {
    let items = vec![tx_item("generation-a"), claim_item("generation-b")];
    let table = span_table(&items, ShardId::new(1));
    let batch_id = batch_id("generation-drift");
    let planner_cfg_digest = cfg_digest("planner.mode=central");
    let authority = PlannerAuthority::bind(PlannerMode::Central, &table, planner_cfg_digest);
    let claimed = BatchPlanner::new(table.clone())
        .plan_batch(batch_id, &items)
        .expect("primary planned batch");

    let mut drifted_table = table.clone();
    drifted_table.routing_generation += 1;
    drifted_table.previous_generation_digest = Some(table.digest());
    drifted_table.activation_checkpoint += 1;
    drifted_table.validate().expect("drifted table stays valid");

    let err = authority
        .verify_batch(
            PlannerMode::Central,
            &drifted_table,
            planner_cfg_digest,
            batch_id,
            &items,
            &claimed,
        )
        .expect_err("mixed planner generation must reject");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("mixed planner generation"));
}

#[test]
fn copied_primary_plan_bytes_reject() {
    let items = vec![tx_item("copied-a"), claim_item("copied-b")];
    let table = span_table(&items, ShardId::new(1));
    let batch_id = batch_id("copied-plan");
    let planner_cfg_digest = cfg_digest("planner.mode=central");
    let authority = PlannerAuthority::bind(PlannerMode::Central, &table, planner_cfg_digest);
    let mut claimed = BatchPlanner::new(table.clone())
        .plan_batch(batch_id, &items)
        .expect("primary planned batch");
    claimed.plan_digest = PlanDigest::new([0x44; 32]);

    let err = authority
        .verify_batch(
            PlannerMode::Central,
            &table,
            planner_cfg_digest,
            batch_id,
            &items,
            &claimed,
        )
        .expect_err("copied primary plan bytes must reject");

    assert_eq!(err.class, RejectClass::PolicyReject);
    assert!(err.detail.contains("planner digest drift"));
}
