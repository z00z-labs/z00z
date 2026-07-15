use z00z_crypto::{sha256_256_role, CheckpointShaRole};
use z00z_storage::{
    checkpoint::recursive_v2::{
        RecursiveAuthoritySnapshotV2, SettlementRootCutoverModeV2,
        SettlementRootGenerationCutoverV2,
    },
    fixture_support::settlement_corpus::{
        asset_item, load_fixture, redb_store_with_bits, HjmtEnvGuard,
    },
    settlement::{RootGeneration, SettlementStore},
};

fn authority(store: &SettlementStore) -> RecursiveAuthoritySnapshotV2 {
    RecursiveAuthoritySnapshotV2::resolve_repository_local_fixture(store)
        .expect("repository-local authority capability")
}

fn opaque_record_digest(record: [u8; 32]) -> [u8; 32] {
    sha256_256_role(
        CheckpointShaRole::Link,
        &[b"z00z.recursive.v2.opaque-last-root-record", &record],
    )
}

fn durable_fixture_store() -> (HjmtEnvGuard, tempfile::TempDir, SettlementStore) {
    let (guard, temp, mut store) = redb_store_with_bits(Some("2")).expect("durable fixture");
    let fixture = load_fixture();
    store
        .put_settlement_item(asset_item(&fixture.assets[0]))
        .expect("persist fixture root");
    (guard, temp, store)
}

fn fixture_cutover(
    store: &SettlementStore,
    authority: RecursiveAuthoritySnapshotV2,
) -> SettlementRootGenerationCutoverV2 {
    let expected = store.settlement_root_v2(7).expect("expected V2 root");
    let opaque = [8; 32];
    SettlementRootGenerationCutoverV2::repository_local_fixture(
        authority,
        store,
        10,
        opaque,
        opaque_record_digest(opaque),
        expected,
        11,
    )
    .expect("fixture cutover")
}

#[test]
fn repository_fixture_cutover_is_durable_exactly_once_and_never_live_authority() {
    let (_guard, temp, mut store) = durable_fixture_store();
    let resolved_authority = authority(&store);
    let expected = store.settlement_root_v2(7).expect("expected root");
    let mut cutover = fixture_cutover(&store, resolved_authority);

    assert_eq!(
        cutover.mode(),
        SettlementRootCutoverModeV2::RepositoryLocalFixture
    );
    assert_eq!(
        cutover
            .install_repository_fixture(&mut store, 11)
            .expect("durable installation"),
        expected
    );
    assert!(cutover.install_repository_fixture(&mut store, 11).is_err());

    drop(store);
    let mut reloaded = SettlementStore::load(temp.path()).expect("reload after cutover");
    let reloaded_authority = authority(&reloaded);
    let mut replay = fixture_cutover(&reloaded, reloaded_authority);
    assert!(replay
        .install_repository_fixture(&mut reloaded, 11)
        .is_err());
}

#[test]
fn repository_fixture_cutover_rejects_snapshot_or_root_substitution() {
    let (_guard, _temp, mut store) = durable_fixture_store();
    let authority = authority(&store);
    let mut cutover = fixture_cutover(&store, authority);
    let fixture = load_fixture();
    store
        .put_settlement_item(asset_item(&fixture.assets[1]))
        .expect("advance live store");
    assert!(cutover.install_repository_fixture(&mut store, 11).is_err());
}

#[test]
fn repository_fixture_cutover_rejects_unpinned_opaque_record() {
    let (_guard, _temp, store) = durable_fixture_store();
    let authority = authority(&store);
    let expected = store.settlement_root_v2(7).expect("expected root");
    assert!(SettlementRootGenerationCutoverV2::repository_local_fixture(
        authority, &store, 10, [8; 32], [9; 32], expected, 11,
    )
    .is_err());
}

#[test]
fn recursive_v2_root_uses_the_live_hjmt_definition_root_owner() {
    let store = SettlementStore::new();
    let root = store.settlement_root_v2(7).expect("V2 root");
    assert_eq!(root.generation(), RootGeneration::SettlementV2);
    assert!(store.settlement_root_v2(0).is_err());
}
