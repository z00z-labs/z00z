use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    sync::Once,
};
use tracing_subscriber::fmt::format::FmtSpan;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::{sha256_256, ZkPackEncrypted};
use z00z_storage::{
    checkpoint::recursive_v2::{
        composed_history_error_exponent_v2, CanonicalCheckpointTransitionV2,
        CheckpointVersionRegistryV2, EpochCadenceClassV2, EpochFrontierAuthorityInputsV2,
        EpochFrontierAuthorityV2, EpochProofFrontierV2, EpochTransitionStreamV2,
        HistoryAccumulatorInputsV2, HistoryAccumulatorStatementV2, HistoryBranchV2,
        Plonky3BaseAdapterV2, Plonky3BaseProofV2, Plonky3BaseRangeBindingV2, Plonky3EpochAdapterV2,
        Plonky3EpochChunkWorkerV2, Plonky3EpochProofV2, Plonky3HistoryAdapterV2,
        Plonky3HistoryAuthorityResolverV2, Plonky3HistoryProofV2, RecursiveBoundedObjectV2,
        RecursiveCheckpointRejectReasonV2, RecursiveCircuitProfileV2,
        RecursiveSecurityBudgetManifestV2, RegistryLifecycleV2,
        EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2, PLONKY3_PUBLISH_BYTES_V2, PLONKY3_TARGET_BYTES_V2,
        RECURSIVE_INGRESS_BYTES_V2, RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
    },
    checkpoint::{
        CheckpointConfigResolverV3, CheckpointDraft, CheckpointExecInput, CheckpointExecOut,
        CheckpointExecTx, CheckpointExecVersion, CheckpointFsStore, CheckpointId, CheckpointInRef,
        CheckpointStore, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    fixture_support::{
        checkpoint_fixtures, genesis_chain_identity::ensure_test_process_chain_identity,
    },
    settlement::{
        DefinitionId, SerialId, SettlementExecHandoff, SettlementPath, SettlementRouteCtx,
        SettlementStateRoot, SettlementStore, StoreItem, StoreOp, TerminalId, TerminalLeaf,
    },
    snapshot::{build_snapshot_v2, PrepFsStore, PrepSnapshotStore},
    CheckpointError,
};

fn digest_hex(digest: [u8; 32]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(64);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("write digest hex");
    }
    encoded
}

fn resource_phase(phase: &str) {
    init_resource_tracing();
    println!("Z00Z_PLONKY3_PHASE_V1 {phase}");
}

fn init_resource_tracing() {
    static INIT: Once = Once::new();
    if std::env::var_os("Z00Z_PLONKY3_RESOURCE_TELEMETRY").is_none() {
        return;
    }
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_env_filter(tracing_subscriber::EnvFilter::new(
                "p3_batch_stark=info,z00z_plonky3_circuit_prover=info,p3_fri=info,p3_dft=info",
            ))
            .try_init()
            .expect("install bounded Plonky3 resource tracing subscriber");
    });
}

fn resource_telemetry(proof: &Plonky3BaseProofV2) {
    let dimensions = proof
        .trace_dimensions()
        .expect("locally produced proof trace dimensions");
    println!(
        concat!(
            "Z00Z_PLONKY3_TELEMETRY_V1 ",
            "{{\"parameter_digest\":\"{}\",\"air_binding_digest\":\"{}\",",
            "\"canonical_proof_bytes\":{},\"size_status\":\"{}\",",
            "\"trace_dimensions\":{{\"chunk_count\":{},\"predicate_words\":{},",
            "\"event_vector_bytes\":{},",
            "\"circuit_witnesses\":{},\"circuit_operations\":{},\"private_inputs\":{},",
            "\"witness_rows\":{},\"constant_rows\":{},\"public_rows\":{},",
            "\"alu_rows\":{},\"non_primitive_tables\":{},\"non_primitive_rows\":{},",
            "\"max_chunk_witnesses\":{},\"max_chunk_operations\":{},",
            "\"max_chunk_alu_rows\":{},\"max_chunk_npo_rows\":{}}}}}"
        ),
        digest_hex(proof.parameter_digest()),
        digest_hex(proof.air_binding_digest()),
        proof.canonical_bytes().len(),
        proof.size_status().name(),
        dimensions.chunk_count(),
        dimensions.predicate_words(),
        dimensions.event_vector_bytes(),
        dimensions.circuit_witnesses(),
        dimensions.circuit_operations(),
        dimensions.private_inputs(),
        dimensions.witness_rows(),
        dimensions.constant_rows(),
        dimensions.public_rows(),
        dimensions.alu_rows(),
        dimensions.non_primitive_tables(),
        dimensions.non_primitive_rows(),
        dimensions.max_chunk_witnesses(),
        dimensions.max_chunk_operations(),
        dimensions.max_chunk_alu_rows(),
        dimensions.max_chunk_npo_rows(),
    );
}

fn profile() -> RecursiveCircuitProfileV2 {
    ensure_test_process_chain_identity().expect("canonical test process chain identity");
    RecursiveCircuitProfileV2::authority_pinned()
}

fn path(definition: u8, serial: u32, terminal: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([definition; 32]),
        SerialId::new(serial),
        TerminalId::new([terminal; 32]),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: [3; 32],
        s_out: [4; 32],
    }
    .to_bytes();
    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1; 32],
        owner_tag: [2; 32],
        c_amount: [5; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0; 16],
        },
        range_proof: vec![9; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("terminal item")
}

fn handoff(input: SettlementPath, output: StoreItem) -> SettlementExecHandoff {
    let tx = CheckpointExecTx::new(
        vec![CheckpointInRef::new(input.terminal_id(), input.serial_id)],
        vec![CheckpointExecOut::new(
            output.path().definition_id,
            output.terminal_leaf().expect("terminal output").clone(),
        )
        .expect("canonical output")],
        vec![8],
    )
    .expect("canonical transaction row");
    SettlementExecHandoff::new(
        SettlementRouteCtx::new([9; 32], 1, 1, [10; 32]),
        vec![StoreOp::Delete(input), StoreOp::Put(Box::new(output))],
        vec![tx],
    )
}

fn expected_post_root(input: SettlementPath, output: StoreItem) -> SettlementStateRoot {
    let mut expected = SettlementStore::new();
    expected
        .put_settlement_item(item(input, 10))
        .expect("seed expected pre-state");
    expected
        .apply_exec_handoff(handoff(input, output))
        .expect("apply canonical expected handoff");
    expected
        .settlement_root_v2(7)
        .expect("expected V2 post-state root")
}

fn canonical_checkpoint(
    root: &std::path::Path,
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
    handoff: &SettlementExecHandoff,
) -> (CheckpointFsStore, PrepFsStore, CheckpointId) {
    let mut checkpoint_store = CheckpointFsStore::new(root);
    let mut prep_store = PrepFsStore::new(root);
    let checkpoint_id = canonical_checkpoint_at_height(
        &mut checkpoint_store,
        &mut prep_store,
        1,
        pre_settlement_root,
        post_settlement_root,
        handoff,
    );
    (checkpoint_store, prep_store, checkpoint_id)
}

fn canonical_checkpoint_at_height(
    checkpoint_store: &mut CheckpointFsStore,
    prep_store: &mut PrepFsStore,
    height: u64,
    pre_settlement_root: SettlementStateRoot,
    post_settlement_root: SettlementStateRoot,
    handoff: &SettlementExecHandoff,
) -> CheckpointId {
    let marker = sha256_256(
        "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
        "checkpoint-delta",
        &[&height.to_le_bytes()],
    );
    let draft = CheckpointDraft::new_settlement(
        CheckpointVersion::CURRENT,
        height,
        pre_settlement_root,
        post_settlement_root,
        vec![SpentEnt::new(marker)],
        vec![CreatedEnt::new(
            sha256_256(
                "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
                "created-id",
                &[&height.to_le_bytes()],
            ),
            sha256_256(
                "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
                "created-owner",
                &[&height.to_le_bytes()],
            ),
        )],
    );
    let (snapshot, snapshot_id) =
        build_snapshot_v2(pre_settlement_root, Vec::new()).expect("prep snapshot");
    assert_eq!(
        prep_store
            .save_snapshot(&snapshot)
            .expect("persist prep snapshot"),
        snapshot_id
    );
    let exec = CheckpointExecInput::new_settlement(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        pre_settlement_root,
        handoff.txs().to_vec(),
    )
    .expect("canonical execution input");
    let exec_id = checkpoint_store
        .save_exec_input(&exec)
        .expect("persist execution input");
    let manifest = checkpoint_fixtures::archive_manifest(&draft, &exec, exec_id);
    let da_reference = checkpoint_fixtures::da_reference(&manifest);
    let statement_core = checkpoint_fixtures::statement_core(&exec);
    checkpoint_store
        .stage_publication_contract(exec_id, &statement_core, &manifest, &da_reference)
        .expect("stage canonical checkpoint evidence");
    let link = checkpoint_store
        .seal_artifact(
            &draft,
            draft
                .attest_proof(snapshot_id, exec_id)
                .expect("attested checkpoint proof"),
            snapshot_id,
            exec_id,
        )
        .expect("persist canonical checkpoint artifact and link");
    link.checkpoint_id()
}

fn fixture() -> (
    tempfile::TempDir,
    SettlementStore,
    CheckpointFsStore,
    PrepFsStore,
    CheckpointId,
    SettlementExecHandoff,
) {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let mut store = SettlementStore::new();
    let input = path(1, 1, 1);
    store
        .put_settlement_item(item(input, 10))
        .expect("seed pre-state");
    let pre_root = store.settlement_root_v2(7).expect("V2 pre-state root");
    let output = item(path(2, 2, 2), 20);
    let post_root = expected_post_root(input, output.clone());
    let handoff = handoff(input, output);
    let (checkpoint_store, prep_store, checkpoint_id) =
        canonical_checkpoint(temp.path(), pre_root, post_root, &handoff);
    (
        temp,
        store,
        checkpoint_store,
        prep_store,
        checkpoint_id,
        handoff,
    )
}

fn transition<'a>(
    temp: &'a tempfile::TempDir,
    store: &mut SettlementStore,
    checkpoint_store: &'a CheckpointFsStore,
    prep_store: &'a PrepFsStore,
    checkpoint_id: CheckpointId,
    handoff: SettlementExecHandoff,
) -> CanonicalCheckpointTransitionV2 {
    CanonicalCheckpointTransitionV2::from_exec(
        temp.path(),
        profile(),
        checkpoint_store,
        prep_store,
        checkpoint_id,
        store,
        handoff,
    )
    .expect("canonical V2 transition")
}

const EXACT_EPOCH_LEAVES: u64 = 2_000;
const EXACT_EPOCH_CACHE_DIR: &str = "production-epoch-2000-frontier-g6-c12-v4";
const EXACT_EPOCH_AUTHORITY_MAGIC: [u8; 8] = *b"Z00Z2K02";
const EXACT_EPOCH_AUTHORITY_VERSION: u16 = 2;
const EXACT_EPOCH_AUTHORITY_BYTES: usize = 8 + 2 + 32 * 5 + 32;

#[derive(Clone, Copy)]
struct ExactEpochAuthoritySeed {
    start_root: [u8; 32],
    chain_context_digest: [u8; 32],
    predicate_digest: [u8; 32],
    parameter_digest: [u8; 32],
    verifier_bundle_digest: [u8; 32],
}

impl ExactEpochAuthoritySeed {
    fn from_range(range: Plonky3BaseRangeBindingV2) -> Self {
        Self {
            start_root: range.pre_settlement_root,
            chain_context_digest: range.chain_context_digest,
            predicate_digest: range.predicate_digest,
            parameter_digest: range.parameter_digest,
            verifier_bundle_digest: range.verifier_bundle_digest,
        }
    }

    fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EXACT_EPOCH_AUTHORITY_BYTES);
        bytes.extend_from_slice(&EXACT_EPOCH_AUTHORITY_MAGIC);
        bytes.extend_from_slice(&EXACT_EPOCH_AUTHORITY_VERSION.to_le_bytes());
        for digest in [
            self.start_root,
            self.chain_context_digest,
            self.predicate_digest,
            self.parameter_digest,
            self.verifier_bundle_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        let checksum = sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "authority-seed",
            &[&bytes],
        );
        bytes.extend_from_slice(&checksum);
        assert_eq!(bytes.len(), EXACT_EPOCH_AUTHORITY_BYTES);
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() != EXACT_EPOCH_AUTHORITY_BYTES
            || bytes[..8] != EXACT_EPOCH_AUTHORITY_MAGIC
            || u16::from_le_bytes(
                bytes[8..10]
                    .try_into()
                    .map_err(|_| CheckpointError::Canonical)?,
            ) != EXACT_EPOCH_AUTHORITY_VERSION
        {
            return Err(CheckpointError::Canonical);
        }
        let checksum_offset = EXACT_EPOCH_AUTHORITY_BYTES - 32;
        let expected = sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "authority-seed",
            &[&bytes[..checksum_offset]],
        );
        if bytes[checksum_offset..] != expected {
            return Err(CheckpointError::Canonical);
        }
        let digest = |index: usize| -> Result<[u8; 32], CheckpointError> {
            let start = 10 + index * 32;
            bytes[start..start + 32]
                .try_into()
                .map_err(|_| CheckpointError::Canonical)
        };
        Ok(Self {
            start_root: digest(0)?,
            chain_context_digest: digest(1)?,
            predicate_digest: digest(2)?,
            parameter_digest: digest(3)?,
            verifier_bundle_digest: digest(4)?,
        })
    }

    fn validate_range(self, range: Plonky3BaseRangeBindingV2) {
        assert_eq!(range.chain_context_digest, self.chain_context_digest);
        assert_eq!(range.predicate_digest, self.predicate_digest);
        assert_eq!(range.parameter_digest, self.parameter_digest);
        assert_eq!(range.verifier_bundle_digest, self.verifier_bundle_digest);
        assert_eq!(range.pre_settlement_root, self.start_root);
    }
}

fn exact_epoch_cache_root() -> PathBuf {
    let base = std::env::var_os("Z00Z_PLONKY3_CHUNK_CACHE_DIR")
        .map(PathBuf::from)
        .expect("resource worker must supply the Phase-069 checkpoint cache root");
    let root = base.join(EXACT_EPOCH_CACHE_DIR);
    fs::create_dir_all(&root).expect("create exact-epoch cache root");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("secure exact-epoch cache permissions");
    }
    root
}

fn exact_epoch_authority_seed_path(root: &Path) -> PathBuf {
    root.join("authority-seed-v1.bin")
}

fn load_exact_epoch_authority_seed(root: &Path) -> Option<ExactEpochAuthoritySeed> {
    let path = exact_epoch_authority_seed_path(root);
    match fs::read(path) {
        Ok(bytes) => Some(
            ExactEpochAuthoritySeed::decode(&bytes).expect("canonical exact-epoch authority seed"),
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("read exact-epoch authority seed: {error}"),
    }
}

fn persist_exact_epoch_authority_seed(root: &Path, seed: ExactEpochAuthoritySeed) {
    let path = exact_epoch_authority_seed_path(root);
    if let Some(existing) = load_exact_epoch_authority_seed(root) {
        assert_eq!(existing.encode(), seed.encode());
        return;
    }
    let temporary = root.join(format!(".authority-seed-v1.{}.tmp", std::process::id()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .expect("create exact-epoch authority seed");
    file.write_all(&seed.encode())
        .expect("write exact-epoch authority seed");
    file.sync_all().expect("sync exact-epoch authority seed");
    fs::rename(&temporary, &path).expect("publish exact-epoch authority seed");
    fs::File::open(root)
        .and_then(|directory| directory.sync_all())
        .expect("sync exact-epoch cache directory");
}

fn exact_epoch_authority(seed: ExactEpochAuthoritySeed) -> EpochFrontierAuthorityV2 {
    EpochFrontierAuthorityV2::new(EpochFrontierAuthorityInputsV2 {
        cadence_class: EpochCadenceClassV2::Production,
        epoch_index: 0,
        cadence_blocks: EXACT_EPOCH_LEAVES,
        start_root: seed.start_root,
        chain_context_digest: seed.chain_context_digest,
        predicate_digest: seed.predicate_digest,
        parameter_digest: seed.parameter_digest,
        verifier_bundle_digest: seed.verifier_bundle_digest,
    })
    .expect("production exact-epoch authority")
}

fn exact_epoch_path(index: u64) -> SettlementPath {
    let index_bytes = index.to_le_bytes();
    SettlementPath::new(
        DefinitionId::new(sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "definition",
            &[&index_bytes],
        )),
        SerialId::new(u32::try_from(index + 1).expect("fixture serial fits")),
        TerminalId::new(sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "terminal",
            &[&index_bytes],
        )),
    )
}

fn exact_epoch_leaf(path: SettlementPath, height: u64) -> TerminalLeaf {
    // The base helper creates a canonical terminal leaf. The height-derived
    // amount makes every deterministic transition distinct.
    leaf(path, height + 10)
}

fn exact_epoch_item(index: u64) -> StoreItem {
    let path = exact_epoch_path(index);
    StoreItem::new(path, exact_epoch_leaf(path, index)).expect("exact-epoch terminal item")
}

fn exact_epoch_handoff(
    height: u64,
    input: SettlementPath,
    output: StoreItem,
) -> SettlementExecHandoff {
    let tx = CheckpointExecTx::new(
        vec![CheckpointInRef::new(input.terminal_id(), input.serial_id)],
        vec![CheckpointExecOut::new(
            output.path().definition_id,
            output.terminal_leaf().expect("terminal output").clone(),
        )
        .expect("canonical exact-epoch output")],
        vec![
            0x08,
            u8::try_from(height % 251).expect("bounded fixture tag"),
        ],
    )
    .expect("canonical exact-epoch transaction");
    SettlementExecHandoff::new(
        SettlementRouteCtx::new(
            sha256_256(
                "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
                "batch",
                &[&height.to_le_bytes()],
            ),
            u32::try_from(height % 16).expect("bounded fixture shard"),
            height,
            sha256_256(
                "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
                "route",
                &[&height.to_le_bytes()],
            ),
        ),
        vec![StoreOp::Delete(input), StoreOp::Put(Box::new(output))],
        vec![tx],
    )
}

fn exact_epoch_transition_fixture(
    target_height: u64,
) -> (
    tempfile::TempDir,
    SettlementStore,
    CheckpointFsStore,
    PrepFsStore,
    CheckpointId,
    SettlementExecHandoff,
) {
    assert!((1..=EXACT_EPOCH_LEAVES).contains(&target_height));
    let temp = tempfile::TempDir::new().expect("exact-epoch temporary witness root");
    let mut store = SettlementStore::new();
    let mut preview = SettlementStore::new();
    let genesis = exact_epoch_item(0);
    store
        .put_settlement_item(genesis.clone())
        .expect("seed exact-epoch pre-state");
    preview
        .put_settlement_item(genesis)
        .expect("seed exact-epoch preview state");
    let mut checkpoint_store = CheckpointFsStore::new(temp.path());
    let mut prep_store = PrepFsStore::new(temp.path());

    for height in 1..=target_height {
        let input = exact_epoch_path(height - 1);
        let output = exact_epoch_item(height);
        let handoff = exact_epoch_handoff(height, input, output);
        let pre_root = store
            .settlement_root_v2(7)
            .expect("exact-epoch pre-state root");
        preview
            .apply_exec_handoff(handoff.clone())
            .expect("preview exact-epoch transition");
        let post_root = preview
            .settlement_root_v2(7)
            .expect("exact-epoch post-state root");
        let checkpoint_id = canonical_checkpoint_at_height(
            &mut checkpoint_store,
            &mut prep_store,
            height,
            pre_root,
            post_root,
            &handoff,
        );
        if height == target_height {
            return (
                temp,
                store,
                checkpoint_store,
                prep_store,
                checkpoint_id,
                handoff,
            );
        }
        store
            .apply_exec_handoff(handoff)
            .expect("advance exact-epoch canonical state");
        assert_eq!(
            store
                .settlement_root_v2(7)
                .expect("advanced exact-epoch root"),
            post_root,
        );
    }
    unreachable!("bounded target height always returns");
}

fn exact_epoch_first_trace_chunk() -> z00z_storage::checkpoint::recursive_v2::EpochTraceChunkWorkV2
{
    let transition_count = u64::from(EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2);
    let temp = tempfile::TempDir::new().expect("trace-chunk temporary witness root");
    let mut store = SettlementStore::new();
    let mut preview = SettlementStore::new();
    let genesis = exact_epoch_item(0);
    store
        .put_settlement_item(genesis.clone())
        .expect("seed trace-chunk pre-state");
    preview
        .put_settlement_item(genesis)
        .expect("seed trace-chunk preview state");
    let mut checkpoint_store = CheckpointFsStore::new(temp.path());
    let mut prep_store = PrepFsStore::new(temp.path());
    let mut stream = EpochTransitionStreamV2::resolve_active(
        &store,
        EpochCadenceClassV2::BoundedSimulation,
        0,
        transition_count,
    )
    .expect("resolve exact bounded trace-chunk stream");
    let mut completed = None;

    for height in 1..=transition_count {
        let input = exact_epoch_path(height - 1);
        let output = exact_epoch_item(height);
        let handoff = exact_epoch_handoff(height, input, output);
        let pre_root = store
            .settlement_root_v2(7)
            .expect("trace-chunk pre-state root");
        preview
            .apply_exec_handoff(handoff.clone())
            .expect("preview trace-chunk transition");
        let post_root = preview
            .settlement_root_v2(7)
            .expect("trace-chunk post-state root");
        let checkpoint_id = canonical_checkpoint_at_height(
            &mut checkpoint_store,
            &mut prep_store,
            height,
            pre_root,
            post_root,
            &handoff,
        );
        let mut canonical = transition(
            &temp,
            &mut store,
            &checkpoint_store,
            &prep_store,
            checkpoint_id,
            handoff,
        );
        let emitted = stream
            .append(&mut canonical, &store)
            .expect("append exact trace-chunk transition");
        drop(canonical);
        assert_eq!(
            store
                .settlement_root_v2(7)
                .expect("advanced trace-chunk root"),
            post_root,
        );
        if height == transition_count {
            completed = emitted;
        } else {
            assert!(emitted.is_none());
        }
    }

    assert_eq!(
        stream.transition_count(),
        EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize
    );
    assert_eq!(stream.emitted_chunk_count(), 1);
    completed.expect("fixed transition span emits exactly one trace chunk")
}

fn exact_history_base_statement(epoch: &Plonky3EpochProofV2) -> HistoryAccumulatorStatementV2 {
    let active = CheckpointConfigResolverV3::resolve_active().expect("active checkpoint config");
    let config_identity = active.identity();
    let authority = Plonky3HistoryAuthorityResolverV2::resolve_active()
        .expect("active Plonky3 history authority");
    let identity = authority.identity();
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned()
        .expect("active recursive security budget");
    let epoch_inputs = epoch.statement().inputs();
    let inherited = security
        .inherited_error_exponent()
        .expect("inherited history error");
    let epoch_anchor_mmr_root =
        Plonky3HistoryAdapterV2::derive_epoch_anchor_mmr_root(None, epoch, None)
            .expect("base epoch-anchor MMR root");
    HistoryAccumulatorStatementV2::new(HistoryAccumulatorInputsV2 {
        branch: HistoryBranchV2::Base,
        first_epoch: 0,
        last_epoch: 0,
        first_height: 1,
        last_height: EXACT_EPOCH_LEAVES,
        cadence_blocks: EXACT_EPOCH_LEAVES,
        history_length: 1,
        accepted_epoch_count: 1,
        config_generation: config_identity.config_generation,
        authority_generation: config_identity.authority_generation,
        activation_height: config_identity.activation_height,
        rollback_floor: config_identity.rollback_floor,
        parameter_generation: identity.parameter_generation,
        runtime_profile_generation: config_identity.runtime_profile_generation,
        composition_rule_generation: security.composition_rule_generation(),
        per_proof_error_exponent: security.per_proof_error_exponent(),
        inherited_error_exponent: inherited,
        cumulative_error_exponent: composed_history_error_exponent_v2(
            security.per_proof_error_exponent(),
            1,
            inherited,
        )
        .expect("base history composition"),
        minimum_residual_bits: security.minimum_residual_bits(),
        chain_context_digest: epoch_inputs.chain_context_digest,
        genesis_trust_anchor_digest: sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "genesis-trust",
            &[],
        ),
        genesis_state_root: epoch_inputs.start_root,
        previous_terminal_state_root: epoch_inputs.start_root,
        current_terminal_state_root: epoch_inputs.end_root,
        previous_epoch_anchor_root: sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "genesis-epoch-anchor",
            &[],
        ),
        current_epoch_anchor_root: epoch_inputs.epoch_close_anchor_digest,
        exact_epoch_statement_digest: epoch.statement().digest(),
        predicate_digest: epoch_inputs.predicate_digest,
        verifier_parameter_digest: identity.verifier_parameter_digest,
        security_budget_digest: identity.security_budget_digest,
        config_digest: config_identity.config_digest,
        registry_digest: config_identity.registry_digest,
        runtime_profile_manifest_digest: config_identity.runtime_profile_manifest_digest,
        authority_bundle_digest: config_identity.history_authority_bundle_digest,
        verifier_bundle_digest: identity.verifier_bundle_digest,
        epoch_anchor_mmr_root,
        predecessor_statement_digest: None,
    })
    .expect("production base-history statement")
}

fn emit_exact_epoch_progress(
    admitted_leaves: u32,
    active_ranges: u32,
    merged_parents: usize,
    completed: bool,
    final_envelope_bytes: Option<usize>,
) {
    println!(
        concat!(
            "Z00Z_PLONKY3_EPOCH_PROGRESS_V1 ",
            "{{\"admitted_leaves\":{},\"total_leaves\":2000,",
            "\"active_ranges\":{},\"merged_parents\":{},",
            "\"completed\":{},\"final_envelope_bytes\":{}}}"
        ),
        admitted_leaves,
        active_ranges,
        merged_parents,
        completed,
        final_envelope_bytes.map_or_else(|| "null".to_owned(), |value| value.to_string()),
    );
}

fn verify_or_build_exact_epoch_final(
    root: &Path,
    frontier: &EpochProofFrontierV2,
    seed: ExactEpochAuthoritySeed,
) {
    let final_path = root.join("history-base-final-v2.bin");
    if let Ok(bytes) = fs::read(&final_path) {
        let history =
            Plonky3HistoryProofV2::decode_local(&bytes).expect("cached final history proof");
        Plonky3HistoryAdapterV2::verify(&history).expect("actual-verify cached history proof");
        assert_eq!(history.statement().last_height(), EXACT_EPOCH_LEAVES);
        assert!(history.canonical_bytes().len() <= PLONKY3_TARGET_BYTES_V2);
        emit_exact_epoch_progress(
            u32::try_from(EXACT_EPOCH_LEAVES).expect("exact cadence fits"),
            u32::try_from(frontier.active_range_count().expect("active range count"))
                .expect("active range count fits"),
            0,
            true,
            Some(history.canonical_bytes().len()),
        );
        println!(
            concat!(
                "Z00Z_PLONKY3_TELEMETRY_V1 ",
                "{{\"parameter_digest\":\"{}\",\"canonical_proof_bytes\":{},",
                "\"size_status\":\"{}\",\"trace_dimensions\":",
                "{{\"epoch_leaf_count\":2000,\"actual_verifier\":true}}}}"
            ),
            digest_hex(seed.parameter_digest),
            history.canonical_bytes().len(),
            history.size_status().name(),
        );
        return;
    }

    let epoch = Plonky3EpochAdapterV2::seal(
        frontier,
        sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "close-anchor",
            &[],
        ),
        Some(sha256_256(
            "z00z.storage.checkpoint.plonky3.epoch-2000.fixture.v2",
            "nova-chain-root",
            &[],
        )),
    )
    .expect("seal exact 2000-leaf epoch proof");
    Plonky3EpochAdapterV2::verify(&epoch).expect("actual-verify exact epoch proof");
    let history = Plonky3HistoryAdapterV2::prove_base(exact_history_base_statement(&epoch), &epoch)
        .expect("prove exact epoch history base");
    Plonky3HistoryAdapterV2::verify(&history).expect("actual-verify exact history base");
    assert_eq!(history.statement().last_height(), EXACT_EPOCH_LEAVES);
    assert!(history.canonical_bytes().len() <= PLONKY3_TARGET_BYTES_V2);

    let temporary = root.join(format!(".history-base-final-v2.{}.tmp", std::process::id()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .expect("create exact history proof");
    file.write_all(history.canonical_bytes())
        .expect("write exact history proof");
    file.sync_all().expect("sync exact history proof");
    fs::rename(&temporary, &final_path).expect("publish exact history proof");
    fs::File::open(root)
        .and_then(|directory| directory.sync_all())
        .expect("sync final exact-epoch cache");

    emit_exact_epoch_progress(
        u32::try_from(EXACT_EPOCH_LEAVES).expect("exact cadence fits"),
        u32::try_from(frontier.active_range_count().expect("active range count"))
            .expect("active range count fits"),
        0,
        true,
        Some(history.canonical_bytes().len()),
    );
    println!(
        concat!(
            "Z00Z_PLONKY3_TELEMETRY_V1 ",
            "{{\"parameter_digest\":\"{}\",\"canonical_proof_bytes\":{},",
            "\"size_status\":\"{}\",\"trace_dimensions\":",
            "{{\"epoch_leaf_count\":2000,\"actual_verifier\":true}}}}"
        ),
        digest_hex(seed.parameter_digest),
        history.canonical_bytes().len(),
        history.size_status().name(),
    );
}

#[test]
#[ignore = "real Plonky3 proving must run through plonky3_resource_worker.sh"]
fn test_direct_typed_commitment_actual_roundtrip() {
    ensure_test_process_chain_identity().expect("canonical test process chain identity");
    let (temp, mut store, checkpoint_store, prep_store, checkpoint_id, handoff) = fixture();
    let mut stream = EpochTransitionStreamV2::resolve_active(
        &store,
        EpochCadenceClassV2::BoundedSimulation,
        0,
        1,
    )
    .expect("one-transition typed-commitment stream");
    let mut transition = transition(
        &temp,
        &mut store,
        &checkpoint_store,
        &prep_store,
        checkpoint_id,
        handoff,
    );
    let work = stream
        .append(&mut transition, &store)
        .expect("append canonical transition")
        .expect("one-transition cadence emits one bounded work item");

    resource_phase("fixture_ready");
    resource_phase("proving");
    let artifact = Plonky3EpochChunkWorkerV2::prove_typed_commitments(work)
        .expect("real direct typed-commitment proof");
    resource_phase("proof_ready");
    resource_phase("verifying");
    artifact
        .verify()
        .expect("actual pinned verifier accepts typed commitments");
    resource_phase("verify_complete");
    println!(
        concat!(
            "Z00Z_PLONKY3_TELEMETRY_V1 ",
            "{{\"parameter_digest\":\"{}\",\"canonical_proof_bytes\":{},",
            "\"size_status\":\"internal_only\",\"trace_dimensions\":",
            "{{\"table\":\"typed_commitment\",\"trace_rows\":32,",
            "\"input_items\":{},\"table_count\":1,\"actual_verifier\":true}}}}"
        ),
        digest_hex(artifact.statement().inputs().parameter_digest),
        artifact.local_proof_bytes().len(),
        artifact.binding_count(),
    );
}

#[test]
#[ignore = "real Plonky3 proving must run through plonky3_resource_worker.sh"]
fn test_direct_transition_batch_actual_roundtrip() {
    ensure_test_process_chain_identity().expect("canonical test process chain identity");
    let (temp, mut store, checkpoint_store, prep_store, checkpoint_id, handoff) = fixture();
    let mut stream = EpochTransitionStreamV2::resolve_active(
        &store,
        EpochCadenceClassV2::BoundedSimulation,
        0,
        1,
    )
    .expect("one-transition linked-table stream");
    let mut transition = transition(
        &temp,
        &mut store,
        &checkpoint_store,
        &prep_store,
        checkpoint_id,
        handoff,
    );
    let work = stream
        .append(&mut transition, &store)
        .expect("append canonical transition")
        .expect("one-transition cadence emits one bounded work item");

    resource_phase("fixture_ready");
    resource_phase("proving");
    let artifact = Plonky3EpochChunkWorkerV2::prove_transition_batch(work)
        .expect("real proof-bound transition Batch-STARK");
    resource_phase("proof_ready");
    resource_phase("verifying");
    artifact
        .verify()
        .expect("actual pinned verifier accepts all linked tables");
    resource_phase("verify_complete");
    println!(
        concat!(
            "Z00Z_PLONKY3_TELEMETRY_V1 ",
            "{{\"parameter_digest\":\"{}\",\"canonical_proof_bytes\":{},",
            "\"size_status\":\"internal_only\",\"trace_dimensions\":",
            "{{\"table\":\"transition_batch\",\"trace_rows\":{},",
            "\"input_items\":{},\"table_count\":{},\"actual_verifier\":true}}}}"
        ),
        digest_hex(artifact.transition_statement().inputs().parameter_digest),
        artifact.local_proof_bytes().len(),
        artifact
            .trace_row_count()
            .expect("canonical linked-table trace rows"),
        artifact.binding_count(),
        artifact.table_count(),
    );
}

#[test]
#[ignore = "real Plonky3 proving must run through plonky3_resource_worker.sh"]
fn test_direct_transition_batch_actual_eight_transition_roundtrip() {
    ensure_test_process_chain_identity().expect("canonical test process chain identity");
    resource_phase("fixture_ready");
    let work = exact_epoch_first_trace_chunk();
    let input_items = work.bindings().len();
    let event_bytes = work.event_bytes();

    resource_phase("proving");
    let artifact = Plonky3EpochChunkWorkerV2::prove_transition_batch(work)
        .expect("real proof-bound eight-transition Batch-STARK");
    resource_phase("proof_ready");
    resource_phase("verifying");
    artifact
        .verify()
        .expect("actual pinned verifier accepts the eight-transition linked tables");
    resource_phase("verify_complete");
    println!(
        concat!(
            "Z00Z_PLONKY3_TELEMETRY_V1 ",
            "{{\"parameter_digest\":\"{}\",\"canonical_proof_bytes\":{},",
            "\"size_status\":\"internal_only\",\"trace_dimensions\":",
            "{{\"table\":\"transition_batch\",\"trace_rows\":{},",
            "\"event_vector_bytes\":{},\"input_items\":{},\"table_count\":{},",
            "\"actual_verifier\":true}}}}"
        ),
        digest_hex(artifact.transition_statement().inputs().parameter_digest),
        artifact.local_proof_bytes().len(),
        artifact
            .trace_row_count()
            .expect("canonical linked-table trace rows"),
        event_bytes,
        input_items,
        artifact.table_count(),
    );
}

#[test]
#[ignore = "restartable exact-cadence prover runs only through plonky3_resource_worker.sh"]
fn test_production_epoch_2000_actual_recursion_step() {
    ensure_test_process_chain_identity().expect("canonical test process chain identity");
    let cache_root = exact_epoch_cache_root();
    let frontier_root = cache_root.join("frontier");
    let mut seed = load_exact_epoch_authority_seed(&cache_root);

    if let Some(authority_seed) = seed {
        let authority = exact_epoch_authority(authority_seed);
        let frontier =
            EpochProofFrontierV2::open(&frontier_root, authority).expect("recover exact frontier");
        let progress = frontier.progress().expect("recover exact progress");
        if progress.all_leaves_admitted() {
            resource_phase("aggregation");
            let _merged =
                Plonky3EpochAdapterV2::merge_ready(&frontier).expect("finish ready exact merges");
            let final_progress = frontier.progress().expect("final exact progress");
            assert!(final_progress.all_leaves_admitted());
            verify_or_build_exact_epoch_final(&cache_root, &frontier, authority_seed);
            return;
        }
    }

    let target_height = match seed {
        Some(authority_seed) => {
            let authority = exact_epoch_authority(authority_seed);
            let frontier = EpochProofFrontierV2::open(&frontier_root, authority)
                .expect("recover exact frontier for next leaf");
            frontier
                .progress()
                .expect("exact frontier progress")
                .next_missing_height()
                .expect("incomplete frontier has a missing canonical height")
        }
        None => 1,
    };
    resource_phase("fixture_ready");
    let (temp, mut store, checkpoint_store, prep_store, checkpoint_id, handoff) =
        exact_epoch_transition_fixture(target_height);
    let mut transition = transition(
        &temp,
        &mut store,
        &checkpoint_store,
        &prep_store,
        checkpoint_id,
        handoff,
    );
    resource_phase("proving");
    let (proof, receipt) = Plonky3BaseAdapterV2::prove_and_verify(&mut transition, &store)
        .expect("real exact-epoch base proof and actual verifier");
    resource_phase("proof_ready");
    let range = proof
        .range_binding()
        .expect("actual-verified exact-epoch range binding");
    assert_eq!(range.height, target_height);
    resource_telemetry(&proof);

    let authority_seed = match seed {
        Some(existing) => {
            existing.validate_range(range);
            existing
        }
        None => {
            let created = ExactEpochAuthoritySeed::from_range(range);
            persist_exact_epoch_authority_seed(&cache_root, created);
            seed = Some(created);
            created
        }
    };
    let authority = exact_epoch_authority(authority_seed);
    let frontier =
        EpochProofFrontierV2::open(&frontier_root, authority).expect("open exact frontier");
    let before = frontier.progress().expect("pre-admission exact progress");
    assert_eq!(before.next_missing_height(), Some(target_height));
    frontier
        .admit_verified_base(&proof, &receipt)
        .expect("admit actual-verified exact base proof");
    drop(receipt);
    drop(proof);
    drop(transition);
    drop(store);
    drop(prep_store);
    drop(checkpoint_store);
    drop(temp);

    resource_phase("aggregation");
    let merged =
        Plonky3EpochAdapterV2::merge_ready(&frontier).expect("merge every ready exact range");
    let progress = frontier.progress().expect("post-merge exact progress");
    assert_eq!(
        progress.admitted_leaf_count(),
        u32::try_from(target_height).expect("exact target fits"),
    );
    assert_eq!(progress.total_leaf_count(), EXACT_EPOCH_LEAVES as u32);
    if progress.all_leaves_admitted() {
        verify_or_build_exact_epoch_final(
            &cache_root,
            &frontier,
            seed.expect("persisted exact authority seed"),
        );
    } else {
        emit_exact_epoch_progress(
            progress.admitted_leaf_count(),
            progress.active_range_count(),
            merged,
            false,
            None,
        );
    }
}

#[test]
#[ignore = "real Plonky3 prover acceptance runs only through plonky3_resource_worker.sh"]
fn test_predicate_differential() {
    let (temp, mut store, checkpoint_store, prep_store, checkpoint_id, handoff) = fixture();
    resource_phase("fixture_ready");
    let mut transition = transition(
        &temp,
        &mut store,
        &checkpoint_store,
        &prep_store,
        checkpoint_id,
        handoff,
    );
    resource_phase("proving");
    let proof =
        Plonky3BaseAdapterV2::prove(&mut transition, &store).expect("real Plonky3 base proof");
    resource_phase("proof_ready");
    resource_telemetry(&proof);
    assert!(
        proof.canonical_bytes().len() <= PLONKY3_TARGET_BYTES_V2,
        "complete canonical proof envelope is {} bytes and exceeds the 2 MiB production target",
        proof.canonical_bytes().len(),
    );
    resource_phase("verifying");
    let receipt = Plonky3BaseAdapterV2::verify(&mut transition, &store, &proof)
        .expect("real Plonky3 base verifier");
    resource_phase("verify_complete");
    assert_eq!(receipt.height(), 1);
    assert_eq!(receipt.statement_digest(), proof.statement().digest());
    assert_eq!(receipt.proof_digest(), proof.proof_digest());
    assert_ne!(receipt.receipt_digest(), [0; 32]);
    let registry = CheckpointVersionRegistryV2::authority_pinned().expect("pinned registry");
    let proof_header = registry
        .validate_preheader(
            proof.canonical_bytes(),
            RecursiveBoundedObjectV2::Plonky3BaseProof,
        )
        .expect("typed proof preheader");
    let receipt_header = registry
        .validate_preheader(
            receipt.canonical_bytes(),
            RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt,
        )
        .expect("typed receipt preheader");
    assert_eq!(
        proof_header.object,
        RecursiveBoundedObjectV2::Plonky3BaseProof
    );
    assert_eq!(
        receipt_header.object,
        RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt
    );

    let decoded =
        Plonky3BaseProofV2::decode_local(proof.canonical_bytes()).expect("canonical local proof");
    assert_eq!(decoded.canonical_bytes(), proof.canonical_bytes());
    let second_receipt = Plonky3BaseAdapterV2::verify(&mut transition, &store, &proof)
        .expect("unchanged actual verifier");
    assert_eq!(second_receipt, receipt);
}

#[test]
#[ignore = "real Plonky3 prover acceptance runs only through plonky3_resource_worker.sh"]
fn test_transcript_mutations_reject() {
    let (temp, mut store, checkpoint_store, prep_store, checkpoint_id, handoff) = fixture();
    resource_phase("fixture_ready");
    let mut transition = transition(
        &temp,
        &mut store,
        &checkpoint_store,
        &prep_store,
        checkpoint_id,
        handoff,
    );
    resource_phase("proving");
    let proof =
        Plonky3BaseAdapterV2::prove(&mut transition, &store).expect("real Plonky3 base proof");
    resource_phase("proof_ready");
    resource_telemetry(&proof);
    let original = proof.canonical_bytes();
    let payload_start = RECURSIVE_OBJECT_PREHEADER_BYTES_V2;
    let statement_len = usize::try_from(u32::from_le_bytes(
        original[payload_start + 10..payload_start + 14]
            .try_into()
            .expect("statement length"),
    ))
    .expect("statement length fits");
    let statement_start = payload_start + 14;
    let digest_block_start = statement_start + statement_len;
    let proof_len_offset = digest_block_start + 32 * 5;
    let proof_start = proof_len_offset + 4;

    let mut transcript_offsets = vec![
        4,
        8,
        10,
        12,
        14,
        16,
        18,
        22,
        26,
        28,
        40,
        statement_start + 8,
        statement_start + 10 + 32 * 11,
        statement_start + 10 + 32 * 11 + 8 + 32,
        statement_start + statement_len - 1,
        digest_block_start,
        digest_block_start + 32,
        digest_block_start + 64,
        digest_block_start + 96,
        digest_block_start + 128,
    ];
    transcript_offsets.extend((0..11).map(|index| statement_start + 10 + 32 * index));
    for offset in transcript_offsets {
        let mut mutated = original.to_vec();
        mutated[offset] ^= 1;
        assert!(
            Plonky3BaseProofV2::decode_local(&mutated).is_err(),
            "transcript family at byte {offset} must reject"
        );
    }
    assert!(Plonky3BaseProofV2::decode_local(&original[..original.len() - 1]).is_err());
    let mut trailing = original.to_vec();
    trailing.push(0);
    assert!(Plonky3BaseProofV2::decode_local(&trailing).is_err());

    let first_statement_digest = statement_start + 10;
    let second_statement_digest = first_statement_digest + 32;
    let mut reordered = original.to_vec();
    let first: [u8; 32] = reordered[first_statement_digest..second_statement_digest]
        .try_into()
        .expect("first statement digest");
    let second: [u8; 32] = reordered[second_statement_digest..second_statement_digest + 32]
        .try_into()
        .expect("second statement digest");
    reordered[first_statement_digest..second_statement_digest].copy_from_slice(&second);
    reordered[second_statement_digest..second_statement_digest + 32].copy_from_slice(&first);
    let reordered_statement_digest = sha256_256(
        "z00z.storage.checkpoint.plonky3.base-statement.v2",
        "statement",
        &[&reordered[statement_start..statement_start + statement_len]],
    );
    reordered[digest_block_start..digest_block_start + 32]
        .copy_from_slice(&reordered_statement_digest);
    assert!(Plonky3BaseProofV2::decode_local(&reordered).is_ok());
    assert!(
        Plonky3BaseProofV2::decode_local_with_source(&reordered, &proof).is_err(),
        "reordered authority roles must reject at verifier ingress"
    );

    let proof_len = usize::try_from(u32::from_le_bytes(
        original[proof_len_offset..proof_start]
            .try_into()
            .expect("proof length"),
    ))
    .expect("proof length fits");
    assert_eq!(proof_start + proof_len, original.len());
    const ROOT_ENVELOPE_HEADER_BYTES: usize = 8 + 2 + 1 + 1 + 32;
    const ROOT_ENTRY_HEADER_BYTES: usize = 1 + 2 + 2 + 4;
    let first_root_len_offset = proof_start + ROOT_ENVELOPE_HEADER_BYTES + 1 + 2 + 2;
    let first_root_start = proof_start + ROOT_ENVELOPE_HEADER_BYTES + ROOT_ENTRY_HEADER_BYTES;
    let first_root_len = usize::try_from(u32::from_le_bytes(
        original[first_root_len_offset..first_root_start]
            .try_into()
            .expect("first root length"),
    ))
    .expect("first root length fits");
    let proof_digest_offset = digest_block_start + 32 * 4;
    let mut canonical_raw_mutation = false;
    for relative in [32_usize, 33, 34, 35, 64, 65, 96, 128, 192, 256, 512, 1_024] {
        if relative >= first_root_len {
            continue;
        }
        let mut candidate = original.to_vec();
        candidate[first_root_start + relative] ^= 1;
        let mutated_proof_digest = sha256_256(
            "z00z.storage.checkpoint.plonky3.base-proof.v2",
            "proof",
            &[&candidate[proof_start..proof_start + proof_len]],
        );
        candidate[proof_digest_offset..proof_digest_offset + 32]
            .copy_from_slice(&mutated_proof_digest);
        if Plonky3BaseProofV2::decode_local(&candidate).is_ok() {
            canonical_raw_mutation = true;
            break;
        }
    }
    assert!(
        !canonical_raw_mutation,
        "raw recursive-root byte mutation must not bypass canonical ingress"
    );
    resource_phase("verifying");
    checkpoint_fixtures::verify_root_opening_mutation(&proof)
        .expect("the actual verifier must reject a typed canonical root-opening mutation");
    resource_phase("verify_complete");
}

#[test]
fn test_security_budget_rejects_mutations() {
    let manifest =
        RecursiveSecurityBudgetManifestV2::authority_pinned().expect("pinned security budget");
    let bytes = manifest.canonical_bytes();
    let registry = CheckpointVersionRegistryV2::authority_pinned().expect("pinned registry");
    let header = registry
        .validate_preheader(
            &bytes,
            RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest,
        )
        .expect("typed security-budget preheader");
    assert_eq!(
        header.object,
        RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest
    );
    assert_eq!(
        registry
            .row(RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest)
            .expect("security-budget row")
            .lifecycle,
        RegistryLifecycleV2::LocalOnly
    );
    assert_eq!(
        RecursiveSecurityBudgetManifestV2::decode_canonical(&bytes).expect("canonical manifest"),
        manifest
    );
    for relative_offset in [
        8_usize, 10, 14, 16, 17, 18, 20, 21, 22, 23, 25, 27, 29, 31, 33, 35, 37, 39, 41, 43, 45,
        49, 53, 55, 57, 65, 67, 69,
    ] {
        let offset = RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + relative_offset;
        let mut mutated = bytes.clone();
        mutated[offset] ^= 1;
        assert!(
            RecursiveSecurityBudgetManifestV2::decode_canonical(&mutated).is_err(),
            "security derivation field at byte {offset} must reject"
        );
    }
    assert!(
        RecursiveSecurityBudgetManifestV2::decode_canonical(&bytes[..bytes.len() - 1]).is_err()
    );
}

#[test]
fn test_proof_budget_rejects_early() {
    let registry = CheckpointVersionRegistryV2::authority_pinned().expect("pinned registry");
    let row = registry
        .row(RecursiveBoundedObjectV2::Plonky3BaseProof)
        .expect("Plonky3 proof registry row");
    assert_eq!(
        usize::try_from(row.max_encoded_len).expect("registry cap fits usize")
            + RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
        PLONKY3_PUBLISH_BYTES_V2
    );

    let target_miss = vec![0_u8; PLONKY3_TARGET_BYTES_V2 + 1];
    assert!(!matches!(
        Plonky3BaseProofV2::decode_local(&target_miss),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded
        ))
    ));
    drop(target_miss);

    let over_production = vec![0_u8; PLONKY3_PUBLISH_BYTES_V2 + 1];
    assert!(matches!(
        Plonky3BaseProofV2::decode_local(&over_production),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded
        ))
    ));
    drop(over_production);

    let over_ingress = vec![0_u8; RECURSIVE_INGRESS_BYTES_V2 + 1];
    assert!(matches!(
        Plonky3BaseProofV2::decode_local(&over_ingress),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofBytesTooLarge
        ))
    ));
}

#[test]
fn test_base_proof_lifecycle_local() {
    const PLONKY3_OWNER: &str = include_str!("../src/checkpoint/plonky3.rs");
    const RECURSIVE_FACADE: &str = include_str!("../src/checkpoint/recursive_v2.rs");
    const SIDECAR: &str = include_str!("../src/checkpoint/sidecar.rs");
    const CHECKPOINT_CODEC: &str = include_str!("../src/checkpoint/codec.rs");
    const AUTHORITY: &str = include_str!("../src/checkpoint/authority_artifacts.rs");
    const SOURCE_REVISION: &str = "b36339709a7a67ee9760fb578b3d4339fd983709";
    assert!(PLONKY3_OWNER.contains("Local-only real Plonky3 base proof"));
    assert!(RECURSIVE_FACADE.contains("Plonky3BaseProofV2"));
    assert!(!SIDECAR.contains("Plonky3BaseProofV2"));
    assert!(!CHECKPOINT_CODEC.contains("Plonky3BaseProofV2"));
    assert!(!PLONKY3_OWNER.contains("impl serde::Serialize for Plonky3BaseProofV2"));
    assert!(!RECURSIVE_FACADE.contains("p3_"));
    assert!(!PLONKY3_OWNER.contains("super::nova::"));
    assert!(!PLONKY3_OWNER.contains("NovaProofEnvelopeV2"));
    assert!(!PLONKY3_OWNER.contains("verify_compressed"));
    assert_eq!(PLONKY3_OWNER.matches(SOURCE_REVISION).count(), 0);
    assert_eq!(AUTHORITY.matches(SOURCE_REVISION).count(), 1);
}
