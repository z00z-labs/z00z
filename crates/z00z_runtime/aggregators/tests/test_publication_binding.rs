use z00z_aggregators::{bind_publication_contract, BatchId};
use z00z_storage::{
    checkpoint::{derive_checkpoint_id, CheckpointDraft, CheckpointVersion, CreatedEnt, SpentEnt},
    fixture_support::checkpoint_fixtures,
    settlement::CheckRoot,
    snapshot::PrepSnapshotId,
};

const README_DOC: &str = include_str!("../README.md");
const LIB_SRC: &str = include_str!("../src/lib.rs");
const TYPES_SRC: &str = include_str!("../src/types.rs");
const SERVICE_SRC: &str = include_str!("../src/service.rs");
const VALIDATOR_FLOW_SRC: &str = include_str!("../../validators/src/checkpoint.rs");
const WATCHER_PUBLICATION_SRC: &str = include_str!("../../watchers/src/publication.rs");
const WATCHER_EVIDENCE_SRC: &str = include_str!("../../watchers/src/evidence_export.rs");
const WATCHER_ENGINE_SRC: &str = include_str!("../../watchers/src/engine.rs");

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .unwrap_or_else(|| panic!("missing section start {start:?}"))
        .split(end)
        .next()
        .unwrap_or_else(|| panic!("missing section end {end:?}"))
}

fn alternate_pub_in() -> z00z_storage::checkpoint::CheckpointPubIn {
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        52,
        CheckRoot::new([0x11; 32]),
        CheckRoot::new([0x22; 32]),
        vec![SpentEnt::new([0x31; 32]), SpentEnt::new([0x32; 32])],
        vec![CreatedEnt::new([0x41; 32], [0x51; 32])],
    );
    let proof = draft
        .attest_proof(
            PrepSnapshotId::new([0x61; 32]),
            z00z_storage::checkpoint::CheckpointExecInputId::new([0x71; 32]),
        )
        .expect("proof");
    draft.finalize(proof).expect("artifact").pub_in()
}

#[test]
fn binding_has_one_entry() {
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
    assert!(TYPES_SRC
        .contains("#[derive(Debug, Clone, PartialEq, Eq)]\npub struct PublicationBinding {"));
    assert!(
        !TYPES_SRC.contains(
            "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct PublicationBinding {"
        )
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
        assert!(!publication_block.contains(field));
    }
    assert!(publication_impl.contains("pub(crate) fn new("));
    assert!(!publication_impl.contains("pub fn new("));
    assert!(README_DOC.contains("must not fork a second publication digest"));
}

#[test]
fn binding_reuses_downstream() {
    assert!(VALIDATOR_FLOW_SRC.contains("publication: bind_publication_contract("));
    assert!(TYPES_SRC.contains("pub fn quorum_binding_enabled(&self) -> bool"));
    assert!(TYPES_SRC.contains("pub subject_digest: Option<[u8; 32]>"));
    assert!(TYPES_SRC.contains("pub certificate_digest: Option<[u8; 32]>"));
    assert!(TYPES_SRC.contains("pub theorem_digest: Option<[u8; 32]>"));
    assert!(WATCHER_PUBLICATION_SRC.contains("if !binding.matches_pub_in(&published.pub_in)"));
    assert!(WATCHER_EVIDENCE_SRC.contains(".map(PublicationBinding::binding_digest)"));
    assert!(WATCHER_ENGINE_SRC.contains("publication.publication.binding_digest()"));

    for source in [
        VALIDATOR_FLOW_SRC,
        WATCHER_PUBLICATION_SRC,
        WATCHER_EVIDENCE_SRC,
        WATCHER_ENGINE_SRC,
    ] {
        assert!(!source.contains("PublicationBinding::new("));
        assert!(!source.contains("digest_binding("));
        assert!(!source.contains("digest_pub_in("));
        assert!(!source.contains("Sha256"));
    }
}

#[test]
fn binding_matches_contract() {
    let artifact = checkpoint_fixtures::artifact();
    let checkpoint_id = derive_checkpoint_id(&artifact).expect("checkpoint id");
    let pub_in = artifact.pub_in();
    let batch_id = BatchId::from_bytes([0x81; 32]);
    let route_table_digest = [0x91; 32];

    let left = bind_publication_contract(batch_id, checkpoint_id, route_table_digest, &pub_in);
    let right = bind_publication_contract(batch_id, checkpoint_id, route_table_digest, &pub_in);

    assert_eq!(left.binding_digest(), right.binding_digest());
    assert!(left.matches_pub_in(&pub_in));
    assert!(left.matches_route_table_digest(route_table_digest));
}

#[test]
fn binding_rejects_drift() {
    let artifact = checkpoint_fixtures::artifact();
    let checkpoint_id = derive_checkpoint_id(&artifact).expect("checkpoint id");
    let pub_in = artifact.pub_in();
    let batch_id = BatchId::from_bytes([0x82; 32]);
    let route_table_digest = [0x92; 32];
    let binding = bind_publication_contract(batch_id, checkpoint_id, route_table_digest, &pub_in);
    let moved = bind_publication_contract(batch_id, checkpoint_id, [0x93; 32], &pub_in);

    assert!(!binding.matches_route_table_digest([0x93; 32]));
    assert_ne!(binding.binding_digest(), moved.binding_digest());
    assert!(!binding.matches_pub_in(&alternate_pub_in()));
}
