const README_DOC: &str = include_str!("../README.md");
const LIB_SRC: &str = include_str!("../src/lib.rs");
const TYPES_SRC: &str = include_str!("../src/types.rs");
const INGRESS_SRC: &str = include_str!("../src/ingress.rs");
const PLANNER_SRC: &str = include_str!("../src/batch_planner.rs");
const PLACEMENT_SRC: &str = include_str!("../src/placement.rs");
const RECOVERY_SRC: &str = include_str!("../src/recovery.rs");
const SERVICE_SRC: &str = include_str!("../src/service.rs");
const VALIDATOR_FLOW_SRC: &str = include_str!("../../validators/src/checkpoint.rs");
const WATCHER_PUBLICATION_SRC: &str = include_str!("../../watchers/src/publication.rs");
const WATCHER_EVIDENCE_SRC: &str = include_str!("../../watchers/src/evidence_export.rs");
const WATCHER_ENGINE_SRC: &str = include_str!("../../watchers/src/engine.rs");
const DIST_DISPATCH_SRC: &str = include_str!("../src/dist_dispatch.rs");
const DIST_SCHEDULER_SRC: &str = include_str!("../src/dist_scheduler.rs");

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .unwrap_or_else(|| panic!("missing section start {start:?}"))
        .split(end)
        .next()
        .unwrap_or_else(|| panic!("missing section end {end:?}"))
}

#[test]
fn planner_items_stay_ingress_only() {
    let work_item_block = TYPES_SRC
        .split("pub struct WorkItem {")
        .nth(1)
        .and_then(|tail| tail.split("impl WorkItem").next())
        .expect("work item block");
    let work_item_impl = TYPES_SRC
        .split("impl WorkItem {")
        .nth(1)
        .and_then(|tail| {
            tail.split("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]")
                .next()
        })
        .expect("work item impl");

    assert!(!work_item_block.contains("pub intake_id"));
    assert!(!work_item_block.contains("pub digest"));
    assert!(!work_item_block.contains("pub payload"));
    assert!(work_item_impl.contains("pub(crate) fn new("));
    assert!(!work_item_impl.contains("pub fn new("));
    assert!(!work_item_impl.contains("pub fn from_tx("));
    assert!(!work_item_impl.contains("pub fn from_claim("));
}

#[test]
fn planner_lane_starts_from_payload() {
    assert!(INGRESS_SRC.contains(
        "pub fn normalize(&self, payload: WorkPayload) -> Result<WorkItem, RejectRecord>"
    ));
    assert!(INGRESS_SRC.contains("build_tx_package_digest"));
    assert!(INGRESS_SRC.contains("build_claim_tx_digest"));
    assert!(SERVICE_SRC
        .contains("fn admit(&mut self, item: WorkPayload) -> Result<WorkItem, RejectRecord>;"));
    assert!(SERVICE_SRC.contains(
        "fn order(&mut self, items: &[WorkItem]) -> Result<OrderedBatch, RejectRecord>;"
    ));
    assert!(SERVICE_SRC.contains("fn bind_exec_handoff("));
    assert!(SERVICE_SRC.contains("batch.exec_handoff(ops, txs)"));
    assert!(README_DOC.contains("emit the only planner-ready `WorkItem` type"));
    assert!(README_DOC.contains("SettlementExecHandoff"));
    assert!(README_DOC.contains("bind_exec_handoff"));
}

#[test]
fn planner_routes_from_verified_key() {
    assert!(PLANNER_SRC.contains("let route_key = item.route_key();"));
    assert!(!PLANNER_SRC.contains("decode_hex32("));
    assert!(PLANNER_SRC.contains("pub struct PlannerAuthority"));
    assert!(README_DOC.contains("Caller-supplied digest strings are never planner authority"));
    assert!(README_DOC.contains(
        "`PlannerAuthority` plus `BatchPlanner` are the only live planner-authority path"
    ));
}

#[test]
fn test_surface_keeps_lineage() {
    assert!(PLACEMENT_SRC.contains("pub expected_journal_lineage: [u8; 32],"));
    assert!(PLACEMENT_SRC.contains("pub fn activate(&self, primary_id: AggregatorId) -> Self"));
    assert!(RECOVERY_SRC.contains("pub enum RecoveryIntent"));
    assert!(RECOVERY_SRC.contains("pub struct ShardRecoveryRecord"));
    assert!(RECOVERY_SRC.contains("current.journal_lineage != record.recovery.journal_lineage"));
    assert!(RECOVERY_SRC
        .contains("wrong lineage: journal lineage does not match committed recovery state"));
    assert!(RECOVERY_SRC
        .contains("stale local root: recovery state root does not match the live durable state"));
    assert!(RECOVERY_SRC.contains("split-brain: live primary owner drifted during recovery"));
    assert!(README_DOC.contains(
        "`RecoveryBoundary` and `ShardRecoveryRecord` are the only runtime recovery export"
    ));
    assert!(
        README_DOC.contains("`ShardPlacement.expected_journal_lineage` binds runtime placement")
    );
    assert!(README_DOC.contains("Same-lineage secondary-aggregator takeover is lawful only when shard id, routing generation, journal lineage, and live local root metadata all match."));
    assert!(README_DOC.contains("Wrong lineage, wrong generation, stale local root, stale restart, secondary aggregator down, and split-brain states reject fail-closed without silent reroute."));
    assert!(
        README_DOC.contains("A shared cross-aggregator WAL is not runtime or protocol truth in v1")
    );
}

#[test]
fn test_binding_has_one_entry() {
    let publication_block = section(
        TYPES_SRC,
        "pub struct PublicationBinding {",
        "impl PublicationBinding {",
    );
    let publication_impl = section(
        TYPES_SRC,
        "impl PublicationBinding {",
        "#[derive(Debug, Clone, PartialEq, Eq)]\npub struct PublicationRecord",
    );

    assert!(LIB_SRC.contains("bind_publication_contract"));
    assert_eq!(
        SERVICE_SRC
            .match_indices("pub fn bind_publication_contract(")
            .count(),
        1
    );
    assert!(SERVICE_SRC
        .contains("PublicationBinding::new(batch_id, checkpoint_id, route_table_digest, pub_in)"));
    assert!(
        TYPES_SRC.contains("#[derive(Debug, Clone, PartialEq, Eq)]\npub struct PublicationBinding {"),
        "PublicationBinding must keep a non-serde derive line so callers cannot deserialize a forged binding"
    );
    assert!(
        !TYPES_SRC.contains(
            "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct PublicationBinding {"
        ),
        "PublicationBinding must not expose serde construction paths outside bind_publication_contract(...)"
    );
    for field in [
        "pub batch_id:",
        "pub checkpoint_id:",
        "pub route_table_digest:",
        "pub prev_settlement_root:",
        "pub new_settlement_root:",
        "pub claim_root:",
        "pub spent_count:",
        "pub created_count:",
        "pub pub_in_digest:",
        "pub binding_digest:",
    ] {
        assert!(
            !publication_block.contains(field),
            "PublicationBinding field {field} must stay private so callers cannot fork a struct-literal binding path"
        );
    }
    assert!(publication_impl.contains("pub(crate) fn new("));
    assert!(!publication_impl.contains("pub fn new("));
    assert!(README_DOC.contains(
        "`bind_publication_contract(...)` is the only runtime-owned path that may derive `PublicationBinding`"
    ));
    assert!(README_DOC
        .contains("must not fork a second publication digest or binding-construction path"));
}

#[test]
fn test_reuses_binding_without_forks() {
    assert!(VALIDATOR_FLOW_SRC.contains("publication: bind_publication_contract("));
    assert!(WATCHER_PUBLICATION_SRC.contains("let binding = verdict"));
    assert!(WATCHER_PUBLICATION_SRC.contains("if !binding.matches_pub_in(&published.pub_in)"));
    assert!(WATCHER_EVIDENCE_SRC.contains(".map(PublicationBinding::binding_digest)"));
    assert!(WATCHER_ENGINE_SRC.contains("publication.publication.binding_digest()"));

    for (label, source) in [
        ("validator checkpoint", VALIDATOR_FLOW_SRC),
        ("watcher publication", WATCHER_PUBLICATION_SRC),
        ("watcher evidence", WATCHER_EVIDENCE_SRC),
        ("watcher engine", WATCHER_ENGINE_SRC),
    ] {
        assert!(
            !source.contains("PublicationBinding::new("),
            "{label} must not construct publication bindings locally"
        );
        assert!(
            !source.contains("digest_binding("),
            "{label} must not define a second binding digest path"
        );
        assert!(
            !source.contains("digest_pub_in("),
            "{label} must not define a second publication-input digest path"
        );
        assert!(
            !source.contains("Sha256"),
            "{label} must not hash publication bindings locally"
        );
    }
}

#[test]
fn test_runtime_has_one_path() {
    assert!(LIB_SRC.contains("mod dist_dispatch;"));
    assert!(LIB_SRC.contains("mod dist_scheduler;"));
    assert!(LIB_SRC.contains("pub use dist_dispatch::{"));
    assert!(LIB_SRC.contains("pub use dist_scheduler::{"));
    assert!(README_DOC.contains("`dist_dispatch`: local route-rollout activation"));
    assert!(README_DOC.contains("`dist_scheduler`: deterministic scheduler waves"));
    assert!(README_DOC.contains("`scheduler` stays a thin facade over `dist_scheduler`"));
    assert!(README_DOC.contains("Runtime notes for rollout, scheduler waves, stalls, freeze, disputes, drift, failover, and storage-lock hazards stay advisory observability only."));
    assert!(DIST_DISPATCH_SRC.contains("pub struct RouteRollout"));
    assert!(DIST_DISPATCH_SRC.contains("pub struct DistDispatch"));
    assert!(DIST_SCHEDULER_SRC.contains("pub struct DistScheduler"));
    assert!(DIST_SCHEDULER_SRC.contains("scheduler wave"));
}
