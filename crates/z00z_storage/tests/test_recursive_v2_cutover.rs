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

const CLEAN_PROCESS_MARKER: &str = "Z00Z_RECURSIVE_V2_CUTOVER_CLEAN_PROCESS";
const CLEAN_PROCESS_PATH: &str = "Z00Z_RECURSIVE_V2_CUTOVER_PATH";

fn authority(store: &SettlementStore) -> RecursiveAuthoritySnapshotV2 {
    z00z_storage::fixture_support::genesis_chain_identity::ensure_test_process_chain_identity()
        .expect("validated canonical devnet genesis identity");
    RecursiveAuthoritySnapshotV2::resolve_active_authority(store)
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
    SettlementRootGenerationCutoverV2::active_authority(
        authority,
        store,
        10,
        opaque,
        opaque_record_digest(opaque),
        expected,
        11,
    )
    .expect("active-authority cutover")
}

#[test]
fn test_cutover_is_exactly_once() {
    let (_guard, temp, mut store) = durable_fixture_store();
    let resolved_authority = authority(&store);
    let expected = store.settlement_root_v2(7).expect("expected root");
    let mut cutover = fixture_cutover(&store, resolved_authority);

    assert_eq!(cutover.mode(), SettlementRootCutoverModeV2::ActiveAuthority);
    assert_eq!(
        cutover
            .install_active_authority(&mut store, 11)
            .expect("durable installation"),
        expected
    );
    assert!(cutover.install_active_authority(&mut store, 11).is_err());

    drop(store);
    let mut reloaded = SettlementStore::load(temp.path()).expect("reload after cutover");
    let reloaded_authority = authority(&reloaded);
    let mut replay = fixture_cutover(&reloaded, reloaded_authority);
    assert!(replay.install_active_authority(&mut reloaded, 11).is_err());
}

#[test]
fn test_cutover_rejects_state_substitution() {
    let (_guard, _temp, mut store) = durable_fixture_store();
    let authority = authority(&store);
    let mut cutover = fixture_cutover(&store, authority);
    let fixture = load_fixture();
    store
        .put_settlement_item(asset_item(&fixture.assets[1]))
        .expect("advance live store");
    assert!(cutover.install_active_authority(&mut store, 11).is_err());
}

#[test]
fn test_cutover_rejects_unpinned_record() {
    let (_guard, _temp, store) = durable_fixture_store();
    let authority = authority(&store);
    let expected = store.settlement_root_v2(7).expect("expected root");
    assert!(SettlementRootGenerationCutoverV2::active_authority(
        authority, &store, 10, [8; 32], [9; 32], expected, 11,
    )
    .is_err());
}

#[test]
fn test_live_hjmt_root_owner() {
    let store = SettlementStore::new();
    let root = store.settlement_root_v2(7).expect("V2 root");
    assert_eq!(root.generation(), RootGeneration::SettlementV2);
    assert!(store.settlement_root_v2(0).is_err());
}

#[test]
fn test_failed_cutover_is_atomic() {
    let (_guard, temp, mut store) = durable_fixture_store();
    let mut committed = fixture_cutover(&store, authority(&store));
    let mut conflicting = fixture_cutover(&store, authority(&store));
    committed
        .install_active_authority(&mut store, 11)
        .expect("first atomic cutover");
    assert!(
        conflicting
            .install_active_authority(&mut store, 11)
            .is_err(),
        "a second valid token must fail inside the durable transaction",
    );

    drop(store);
    let mut reloaded = SettlementStore::load(temp.path()).expect("reload failed transaction");
    let mut replay = fixture_cutover(&reloaded, authority(&reloaded));
    assert!(
        replay.install_active_authority(&mut reloaded, 11).is_err(),
        "the failed duplicate must not remove or replace the committed record",
    );
}

#[test]
fn test_cutover_clean_process_reload() {
    if std::env::var_os(CLEAN_PROCESS_MARKER).is_some() {
        let path = std::env::var_os(CLEAN_PROCESS_PATH).expect("child cutover path");
        let mut store = SettlementStore::load(path).expect("clean-process reload");
        let mut replay = fixture_cutover(&store, authority(&store));
        assert!(
            replay.install_active_authority(&mut store, 11).is_err(),
            "a clean process must observe the already committed cutover"
        );
        return;
    }

    let (_guard, temp, mut store) = durable_fixture_store();
    let mut cutover = fixture_cutover(&store, authority(&store));
    cutover
        .install_active_authority(&mut store, 11)
        .expect("durable parent installation");
    drop(store);

    let current_thread = std::thread::current();
    let test_name = current_thread.name().expect("test harness name");
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture")
        .env(CLEAN_PROCESS_MARKER, "1")
        .env(CLEAN_PROCESS_PATH, temp.path())
        .env("Z00Z_SETTLEMENT_BACKEND_MODE", "hjmt")
        .env("Z00Z_SETTLEMENT_BUCKET_BITS", "2")
        .env("Z00Z_STORAGE_SCHED_CPU", "1")
        .env("Z00Z_STORAGE_SCHED_QUEUE", "1024")
        .output()
        .expect("start clean-process cutover verifier");
    assert!(
        output.status.success(),
        "clean-process cutover verification failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
