use std::sync::{Mutex, OnceLock};

use redb::ReadableTable;
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::checkpoint::{
    decode_art_bin, decode_draft_bin, derive_draft_id, CheckpointExecInputId, CheckpointExecOut,
    CheckpointExecTx, CheckpointId, CheckpointInRef, CheckpointLink, CheckpointLinkVersion,
    CheckpointProofSystem, CheckpointStatement, CheckpointVersion, CreatedEnt, SpentEnt,
};
use z00z_storage::settlement::{
    CheckRoot, ClaimSourceRoot, DefinitionId, FeeActorCtx, FeeEnvelope, FeeReplayKey, FeeReplayRec,
    FeeSupportCtx, SerialId, SettlementPath, SettlementStateRoot, SettlementStore, StoreItem,
    StoreOp, TerminalId, TerminalLeaf,
};
use z00z_utils::codec::{BincodeCodec, Codec};

const CHECK_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_checkpoints");
const DRAFT_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_cp_drafts");
const EXEC_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_cp_execs");
const FEE_REPLAY_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_fee_replays");
const HJMT_JOURNAL_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_hjmt_journal");
const HJMT_PENDING_META_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_hjmt_pending_meta");
const LINK_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_cp_links");
const META_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_meta");
const DB_FILE: &str = "settlement_state.redb";
const HJMT_INJ_STAGE_ENV: &str = "Z00Z_STORAGE_HJMT_INJ_STAGE";
const KEY_ACTIVE: &[u8] = b"active_version";
const KEY_STATE: &[u8] = b"state_meta";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct StateMetaWire {
    version: u64,
    state_root: [u8; 32],
    flat_root: [u8; 32],
    snap_id: [u8; 32],
    draft_id: [u8; 32],
    check_id: [u8; 32],
    exec_id: [u8; 32],
    def_root: Option<[u8; 32]>,
    #[serde(default)]
    fee_replay_count: u64,
    #[serde(default)]
    fee_replay_digest: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CheckArtWire {
    version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
    prep_snapshot_id: Option<z00z_storage::snapshot::PrepSnapshotId>,
    exec_input_id: Option<CheckpointExecInputId>,
    statement_core: Option<z00z_storage::checkpoint::CheckpointTransitionStatementCoreV1>,
    da_ref: Option<[u8; 32]>,
    proof_sys: CheckpointProofSystem,
    cp_proof: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DraftWire {
    version: CheckpointVersion,
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    prev_settlement_root: SettlementStateRoot,
    new_settlement_root: SettlementStateRoot,
    claim_root: Option<ClaimSourceRoot>,
    spent_delta: Vec<SpentEnt>,
    created_delta: Vec<CreatedEnt>,
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(definition)),
        SerialId::new(serial),
        TerminalId::new(bytes(asset)),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: bytes(3),
        s_out: bytes(4),
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: bytes(1),
        owner_tag: bytes(2),
        c_amount: bytes(5),
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("store item")
}

fn settlement_path(path: SettlementPath) -> SettlementPath {
    path
}

fn fee_envelope(mark: u8, support: FeeSupportCtx) -> FeeEnvelope {
    let budget_units = support.required_units;
    let support_ref = Some(bytes(mark.wrapping_add(8)));
    FeeEnvelope {
        version: 1,
        payer_commitment: bytes(mark),
        sponsor_commitment: bytes(mark.wrapping_add(1)),
        budget_units,
        budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: support.domain_id,
        expires_at: 90,
        nonce: bytes(mark.wrapping_add(4)),
        transition_id: support.transition_id,
        replay_key: bytes(mark.wrapping_add(6)),
        support_ref,
        failure_policy_id: bytes(mark.wrapping_add(7)),
    }
}

fn actor(mark: u8) -> FeeActorCtx {
    FeeActorCtx {
        now: 12,
        payer_commitment: Some(bytes(mark)),
        sponsor_commitment: Some(bytes(mark.wrapping_add(1))),
    }
}

fn exec_tx(path: SettlementPath, value: u64, proof: &[u8]) -> CheckpointExecTx {
    CheckpointExecTx::new(
        vec![CheckpointInRef::new(
            path.terminal_id().into_bytes(),
            path.serial_id,
        )],
        vec![CheckpointExecOut::new(path.definition_id, leaf(path, value)).expect("exec out")],
        proof.to_vec(),
    )
    .expect("exec tx")
}

fn remove_fee_row(root: &std::path::Path, version: u64, replay_key: FeeReplayKey) {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write
            .open_table(FEE_REPLAY_TABLE)
            .expect("fee replay table");
        let mut key = Vec::with_capacity(40);
        key.extend_from_slice(&version.to_be_bytes());
        key.extend_from_slice(replay_key.as_bytes());
        table.remove(key.as_slice()).expect("remove fee row");
    }
    write.commit().expect("commit fee row removal");
}

fn tamper_fee_row(root: &std::path::Path, version: u64, replay_key: FeeReplayKey) {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write
            .open_table(FEE_REPLAY_TABLE)
            .expect("fee replay table");
        let mut key = Vec::with_capacity(40);
        key.extend_from_slice(&version.to_be_bytes());
        key.extend_from_slice(replay_key.as_bytes());
        let row_bytes = table
            .get(key.as_slice())
            .expect("fee replay get")
            .expect("fee replay row")
            .value()
            .to_vec();
        let codec = BincodeCodec;
        let mut row: FeeReplayRec = codec.deserialize(&row_bytes).expect("fee replay decode");
        row.nonce = bytes(199);
        let row_bytes = codec.serialize(&row).expect("fee replay encode");
        table
            .insert(key.as_slice(), row_bytes.as_slice())
            .expect("fee replay update");
    }
    write.commit().expect("commit fee row tamper");
}

fn has_pending_meta(root: &std::path::Path, version: u64) -> bool {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let read = db.begin_read().expect("begin read");
    let table = read
        .open_table(HJMT_PENDING_META_TABLE)
        .expect("pending meta table");
    table
        .get(version.to_be_bytes().as_slice())
        .expect("pending meta get")
        .is_some()
}

fn pending_meta(root: &std::path::Path, version: u64) -> StateMetaWire {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let read = db.begin_read().expect("begin read");
    let table = read
        .open_table(HJMT_PENDING_META_TABLE)
        .expect("pending meta table");
    let bytes = table
        .get(version.to_be_bytes().as_slice())
        .expect("pending meta get")
        .expect("pending meta row")
        .value()
        .to_vec();
    let codec = BincodeCodec;
    codec.deserialize(&bytes).expect("pending meta decode")
}

fn active_version(root: &std::path::Path) -> u64 {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(META_TABLE).expect("meta table");
    let bytes = table
        .get(KEY_ACTIVE)
        .expect("active version get")
        .expect("active version row");
    let mut raw = [0u8; 8];
    raw.copy_from_slice(bytes.value());
    u64::from_be_bytes(raw)
}

fn tamper_pending_meta_fee_digest(root: &std::path::Path, version: u64) {
    let mut meta = pending_meta(root, version);
    meta.fee_replay_digest = bytes(201);

    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let write = db.begin_write().expect("begin write");
    {
        let mut table = write
            .open_table(HJMT_PENDING_META_TABLE)
            .expect("pending meta table");
        let meta_bytes = BincodeCodec.serialize(&meta).expect("pending meta encode");
        table
            .insert(version.to_be_bytes().as_slice(), meta_bytes.as_slice())
            .expect("pending meta update");
    }
    write.commit().expect("commit pending meta drift");
}

fn tamper_active_meta_fee_digest(root: &std::path::Path) {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(META_TABLE).expect("meta table");
    let meta_bytes = table
        .get(KEY_STATE)
        .expect("state meta get")
        .expect("state meta row")
        .value()
        .to_vec();
    drop(table);
    drop(read);

    let mut meta: StateMetaWire = BincodeCodec
        .deserialize(&meta_bytes)
        .expect("state meta decode");
    meta.fee_replay_digest = bytes(202);

    let write = db.begin_write().expect("begin write");
    {
        let mut table = write.open_table(META_TABLE).expect("meta table");
        let meta_bytes = BincodeCodec.serialize(&meta).expect("state meta encode");
        table
            .insert(KEY_STATE, meta_bytes.as_slice())
            .expect("state meta update");
    }
    write.commit().expect("commit state meta drift");
}

fn table_bytes(
    root: &std::path::Path,
    table_def: redb::TableDefinition<&[u8], &[u8]>,
    key: [u8; 32],
) -> Vec<u8> {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(table_def).expect("table");
    table
        .get(&key[..])
        .expect("table get")
        .expect("table row")
        .value()
        .to_vec()
}

fn sha256_row_hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn check_wire(artifact: &z00z_storage::checkpoint::CheckpointArtifact) -> CheckArtWire {
    let (prep_snapshot_id, exec_input_id) = match artifact.statement() {
        CheckpointStatement::V1(stmt) => {
            (Some(stmt.prep_snapshot_id()), Some(stmt.exec_input_id()))
        }
        CheckpointStatement::Detached => {
            panic!("persisted checkpoint artifact must stay statement-bound")
        }
    };

    CheckArtWire {
        version: artifact.version(),
        height: artifact.height(),
        prev_root: artifact.prev_root(),
        new_root: artifact.new_root(),
        prev_settlement_root: artifact.prev_settlement_root(),
        new_settlement_root: artifact.new_settlement_root(),
        claim_root: artifact.claim_root(),
        spent_delta: artifact.spent_delta().to_vec(),
        created_delta: artifact.created_delta().to_vec(),
        prep_snapshot_id,
        exec_input_id,
        statement_core: artifact.statement_core(),
        da_ref: artifact.da_ref(),
        proof_sys: artifact.proof_sys(),
        cp_proof: artifact.cp_proof().to_vec(),
    }
}

fn draft_wire(draft: &z00z_storage::checkpoint::CheckpointDraft) -> DraftWire {
    DraftWire {
        version: draft.version(),
        height: draft.height(),
        prev_root: draft.prev_root(),
        new_root: draft.new_root(),
        prev_settlement_root: draft.prev_settlement_root(),
        new_settlement_root: draft.new_settlement_root(),
        claim_root: draft.claim_root(),
        spent_delta: draft.spent_delta().to_vec(),
        created_delta: draft.created_delta().to_vec(),
    }
}

fn table_versions(
    root: &std::path::Path,
    table_def: redb::TableDefinition<&[u8], &[u8]>,
) -> Vec<u64> {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(table_def).expect("table");
    table
        .iter()
        .expect("table iter")
        .map(|entry| {
            let (key, _) = entry.expect("table entry");
            let mut version = [0u8; 8];
            version.copy_from_slice(&key.value()[..8]);
            u64::from_be_bytes(version)
        })
        .collect()
}

fn assert_fee_rejects_reuse(err: impl ToString) {
    let err = err.to_string();
    assert!(
        err.contains("replay binding is invalid")
            || err.contains("domain binding mismatch")
            || err.contains("transition binding mismatch"),
        "unexpected fee rejection: {err}"
    );
}

#[test]
fn test_rejects_post_reload() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let root = store.put_settlement_item(item(path(9, 1, 9), 900))?;
    let ops = vec![StoreOp::Put(Box::new(item(path(9, 1, 9), 901)))];
    let envelope = fee_envelope(60, store.fee_support_ctx(&ops)?);

    let next_root = store.apply_fee_ops(ops.clone(), envelope, actor(60))?;
    assert_ne!(next_root.into_bytes(), root.into_bytes());
    drop(store);

    let mut reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(
        reloaded.settlement_root()?.into_bytes(),
        next_root.into_bytes()
    );
    assert!(reloaded
        .fee_replay_rec(&FeeReplayKey::new(bytes(66)))?
        .is_some());

    let err = reloaded
        .apply_fee_ops(ops, envelope, actor(60))
        .expect_err("replayed fee support must reject");
    assert_fee_rejects_reuse(err);
    assert_eq!(
        reloaded.settlement_root()?.into_bytes(),
        next_root.into_bytes()
    );
    Ok(())
}

#[test]
fn test_row_tamper_rejects_reload() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let ops = vec![StoreOp::Put(Box::new(item(path(10, 1, 9), 1_001)))];
    store.put_settlement_item(item(path(10, 1, 9), 1_000))?;
    let envelope = fee_envelope(70, store.fee_support_ctx(&ops)?);
    let _ = store.apply_fee_ops(ops, envelope, actor(70))?;
    drop(store);

    remove_fee_row(temp.path(), 2, FeeReplayKey::new(bytes(76)));
    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("tampered fee replay row must reject reload"),
        Err(err) => err,
    };
    assert!(err
        .to_string()
        .contains("fee replay rows do not match persisted replay metadata"));
    Ok(())
}

#[test]
fn test_content_tamper_rejects_reload() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let ops = vec![StoreOp::Put(Box::new(item(path(15, 1, 9), 1_501)))];
    store.put_settlement_item(item(path(15, 1, 9), 1_500))?;
    let envelope = fee_envelope(74, store.fee_support_ctx(&ops)?);
    let _ = store.apply_fee_ops(ops, envelope, actor(74))?;
    drop(store);

    tamper_fee_row(temp.path(), 2, FeeReplayKey::new(bytes(80)));
    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("content-tampered fee replay row must reject reload"),
        Err(err) => err,
    };
    assert!(err
        .to_string()
        .contains("fee replay rows do not match persisted replay metadata"));
    Ok(())
}

#[test]
fn test_metadata_tamper_rejects_reload() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let ops = vec![StoreOp::Put(Box::new(item(path(15, 2, 9), 1_551)))];
    store.put_settlement_item(item(path(15, 2, 9), 1_550))?;
    let envelope = fee_envelope(75, store.fee_support_ctx(&ops)?);
    let _ = store.apply_fee_ops(ops, envelope, actor(75))?;
    drop(store);

    tamper_active_meta_fee_digest(temp.path());
    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("fee replay metadata drift must reject reload"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("fee replay rows do not match persisted replay metadata"),
        "{err}"
    );
    Ok(())
}

#[test]
fn test_post_parent_stage_hjmt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);

    let temp = tempdir()?;
    let target = path(16, 1, 9);
    let target_settlement = settlement_path(target);

    let mut store = SettlementStore::load(temp.path())?;
    store.put_settlement_item(item(target, 1_600))?;
    let ops = vec![StoreOp::Put(Box::new(item(target, 1_601)))];
    let txs = vec![exec_tx(target, 1_601, b"fee-attested-proof")];
    let envelope = fee_envelope(82, store.fee_support_exec_ctx(&ops, &txs)?);

    std::env::set_var(HJMT_INJ_STAGE_ENV, "parents");
    let err = store
        .apply_attested_fee_ops(ops.clone(), txs, envelope, actor(82))
        .expect_err("parents-stage injection must fail before publication");
    assert!(err.to_string().contains("hjmt journal injection"), "{err}");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    drop(store);
    assert!(
        has_pending_meta(temp.path(), 2),
        "pending meta versions: {:?}, journal versions: {:?}",
        table_versions(temp.path(), HJMT_PENDING_META_TABLE),
        table_versions(temp.path(), HJMT_JOURNAL_TABLE)
    );

    let reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(
        reloaded.get_settlement_item(&target_settlement)?,
        Some(item(target, 1_601))
    );
    assert!(reloaded
        .fee_replay_rec(&FeeReplayKey::new(bytes(88)))?
        .is_some());
    Ok(())
}

#[test]
fn test_fee_metadata_drift_hjmt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let target = path(16, 1, 11);
    let mut store = SettlementStore::load(temp.path())?;
    store.put_settlement_item(item(target, 1_610))?;
    let ops = vec![StoreOp::Put(Box::new(item(target, 1_611)))];
    let envelope = fee_envelope(86, store.fee_support_ctx(&ops)?);

    std::env::set_var(HJMT_INJ_STAGE_ENV, "parents");
    let err = store
        .apply_fee_ops(ops, envelope, actor(86))
        .expect_err("parents-stage injection must fail before publication");
    assert!(err.to_string().contains("hjmt journal injection"), "{err}");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    drop(store);

    tamper_pending_meta_fee_digest(temp.path(), 2);
    assert_eq!(active_version(temp.path()), 1);

    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("pending fee metadata drift must reject"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("fee replay rows do not match persisted replay metadata"),
        "{err}"
    );
    assert_eq!(active_version(temp.path()), 1);
    Ok(())
}

#[test]
fn test_ckpt_proof_drift_hjmt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let target = path(17, 1, 9);
    let mut store = SettlementStore::load(temp.path())?;
    store.put_settlement_item(item(target, 1_700))?;
    let ops = vec![StoreOp::Put(Box::new(item(target, 1_701)))];
    let txs = vec![exec_tx(target, 1_701, b"fee-proof-drift")];
    let envelope = fee_envelope(83, store.fee_support_exec_ctx(&ops, &txs)?);

    std::env::set_var(HJMT_INJ_STAGE_ENV, "parents");
    let err = store
        .apply_attested_fee_ops(ops, txs, envelope, actor(83))
        .expect_err("parents-stage injection must fail before publication");
    assert!(err.to_string().contains("hjmt journal injection"), "{err}");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    drop(store);

    let pending = pending_meta(temp.path(), 2);
    let check_bytes = table_bytes(temp.path(), CHECK_TABLE, pending.check_id);
    let checkpoint = decode_art_bin(&check_bytes)?;
    let mut wire = check_wire(&checkpoint);
    wire.cp_proof.push(0x55);

    let db = redb::Database::create(temp.path().join(DB_FILE)).expect("open db");
    let write = db.begin_write().expect("begin write");
    {
        let mut checks = write.open_table(CHECK_TABLE).expect("check table");
        checks.insert(
            &pending.check_id[..],
            BincodeCodec.serialize(&wire)?.as_slice(),
        )?;
    }
    write.commit().expect("commit proof drift");
    drop(db);

    assert_eq!(active_version(temp.path()), 1);
    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("tampered proof bytes must reject"),
        Err(err) => err,
    };
    let err = err.to_string();
    assert!(
        err.contains("checkpoint proof mismatch")
            || err.contains("checkpoint proof bytes do not match persisted exec id and state root"),
        "{err}"
    );
    assert_eq!(active_version(temp.path()), 1);
    Ok(())
}

#[test]
fn test_ckpt_draft_drift_hjmt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let target = path(18, 1, 9);
    let mut store = SettlementStore::load(temp.path())?;
    store.put_settlement_item(item(target, 1_800))?;
    let ops = vec![StoreOp::Put(Box::new(item(target, 1_801)))];
    let txs = vec![exec_tx(target, 1_801, b"fee-draft-drift")];
    let envelope = fee_envelope(84, store.fee_support_exec_ctx(&ops, &txs)?);

    std::env::set_var(HJMT_INJ_STAGE_ENV, "parents");
    let err = store
        .apply_attested_fee_ops(ops, txs, envelope, actor(84))
        .expect_err("parents-stage injection must fail before publication");
    assert!(err.to_string().contains("hjmt journal injection"), "{err}");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    drop(store);

    let pending = pending_meta(temp.path(), 2);
    let draft_bytes = table_bytes(temp.path(), DRAFT_TABLE, pending.draft_id);
    let draft = decode_draft_bin(&draft_bytes)?;
    let mut wire = draft_wire(&draft);
    wire.new_root = CheckRoot::new([0x44u8; 32]);
    wire.new_settlement_root = SettlementStateRoot::settlement_v1(wire.new_root.into_bytes());
    let wire_bytes = BincodeCodec.serialize(&wire)?;
    let drift_draft = decode_draft_bin(&wire_bytes)?;
    let drift_draft_id = derive_draft_id(&drift_draft)?.into_bytes();
    let mut drift_meta = pending.clone();
    drift_meta.draft_id = drift_draft_id;

    let db = redb::Database::create(temp.path().join(DB_FILE)).expect("open db");
    let write = db.begin_write().expect("begin write");
    {
        let mut drafts = write.open_table(DRAFT_TABLE).expect("draft table");
        drafts
            .remove(&pending.draft_id[..])?
            .expect("remove current draft");
        drafts.insert(&drift_draft_id[..], wire_bytes.as_slice())?;
    }
    {
        let mut pending_meta_table = write
            .open_table(HJMT_PENDING_META_TABLE)
            .expect("pending meta table");
        let pending_meta_bytes = BincodeCodec.serialize(&drift_meta)?;
        pending_meta_table.insert(
            drift_meta.version.to_be_bytes().as_slice(),
            pending_meta_bytes.as_slice(),
        )?;
    }
    write.commit().expect("commit draft drift");
    drop(db);

    assert_eq!(active_version(temp.path()), 1);
    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("draft drift must reject"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("checkpoint artifact statement does not match persisted draft boundary"),
        "{err}"
    );
    assert_eq!(active_version(temp.path()), 1);
    Ok(())
}

#[test]
fn test_stmtless_link_bundle_hjmt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    let temp = tempdir()?;

    let target = path(19, 1, 9);
    let mut store = SettlementStore::load(temp.path())?;
    store.put_settlement_item(item(target, 1_900))?;
    let ops = vec![StoreOp::Put(Box::new(item(target, 1_901)))];
    let txs = vec![exec_tx(target, 1_901, b"fee-link-drift")];
    let envelope = fee_envelope(85, store.fee_support_exec_ctx(&ops, &txs)?);

    std::env::set_var(HJMT_INJ_STAGE_ENV, "parents");
    let err = store
        .apply_attested_fee_ops(ops, txs, envelope, actor(85))
        .expect_err("parents-stage injection must fail before publication");
    assert!(err.to_string().contains("hjmt journal injection"), "{err}");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    drop(store);

    let pending = pending_meta(temp.path(), 2);
    let draft_bytes = table_bytes(temp.path(), DRAFT_TABLE, pending.draft_id);
    let exec_bytes = table_bytes(temp.path(), EXEC_TABLE, pending.exec_id);
    let draft = decode_draft_bin(&draft_bytes)?;
    let statementless_check_bytes = BincodeCodec.serialize(&CheckArtWire {
        version: CheckpointVersion::CURRENT,
        height: draft.height(),
        prev_root: draft.prev_root(),
        new_root: draft.new_root(),
        prev_settlement_root: draft.prev_settlement_root(),
        new_settlement_root: draft.new_settlement_root(),
        claim_root: draft.claim_root(),
        spent_delta: draft.spent_delta().to_vec(),
        created_delta: draft.created_delta().to_vec(),
        prep_snapshot_id: None,
        exec_input_id: None,
        statement_core: None,
        da_ref: None,
        proof_sys: CheckpointProofSystem::OPAQUE_ATTEST,
        cp_proof: vec![9u8],
    })?;
    let alternate_exec_id = sha256_row_hash(&exec_bytes);
    let alternate_draft_id = sha256_row_hash(&draft_bytes);
    let alternate_check_id = sha256_row_hash(&statementless_check_bytes);
    let unbound_link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new(alternate_check_id),
        z00z_storage::snapshot::PrepSnapshotId::new(pending.snap_id),
        CheckpointExecInputId::new(alternate_exec_id),
    )?;
    let unbound_link_bytes = BincodeCodec.serialize(&unbound_link)?;
    let mut drift_meta = pending.clone();
    drift_meta.draft_id = alternate_draft_id;
    drift_meta.check_id = alternate_check_id;
    drift_meta.exec_id = alternate_exec_id;

    let db = redb::Database::create(temp.path().join(DB_FILE)).expect("open db");
    let write = db.begin_write().expect("begin write");
    {
        let mut execs = write.open_table(EXEC_TABLE).expect("exec table");
        let raw = execs
            .remove(&pending.exec_id[..])?
            .expect("remove current exec")
            .value()
            .to_vec();
        execs.insert(&alternate_exec_id[..], raw.as_slice())?;
    }
    {
        let mut drafts = write.open_table(DRAFT_TABLE).expect("draft table");
        let raw = drafts
            .remove(&pending.draft_id[..])?
            .expect("remove current draft")
            .value()
            .to_vec();
        drafts.insert(&alternate_draft_id[..], raw.as_slice())?;
    }
    {
        let mut checks = write.open_table(CHECK_TABLE).expect("check table");
        checks
            .remove(&pending.check_id[..])?
            .expect("remove current check");
        checks.insert(
            &alternate_check_id[..],
            statementless_check_bytes.as_slice(),
        )?;
    }
    {
        let mut links = write.open_table(LINK_TABLE).expect("link table");
        links
            .remove(&pending.check_id[..])?
            .expect("remove current link");
        links.insert(&alternate_check_id[..], unbound_link_bytes.as_slice())?;
    }
    {
        let mut pending_meta_table = write
            .open_table(HJMT_PENDING_META_TABLE)
            .expect("pending meta table");
        let pending_meta_bytes = BincodeCodec.serialize(&drift_meta)?;
        pending_meta_table.insert(
            drift_meta.version.to_be_bytes().as_slice(),
            pending_meta_bytes.as_slice(),
        )?;
    }
    write.commit().expect("commit statementless link bundle");
    drop(db);

    assert_eq!(active_version(temp.path()), 1);
    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("statementless link bundle must reject"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains(
            "checkpoint artifacts missing statement ids cannot carry persisted link metadata"
        ),
        "{err}"
    );
    assert_eq!(active_version(temp.path()), 1);
    Ok(())
}
