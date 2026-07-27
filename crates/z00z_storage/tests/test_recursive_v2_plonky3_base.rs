use std::sync::Once;
use tracing_subscriber::fmt::format::FmtSpan;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::{sha256_256, ZkPackEncrypted};
use z00z_storage::{
    checkpoint::recursive_v2::{
        CanonicalCheckpointTransitionV2, CheckpointVersionRegistryV2, Plonky3BaseAdapterV2,
        Plonky3BaseProofV2, RecursiveBoundedObjectV2, RecursiveCheckpointRejectReasonV2,
        RecursiveCircuitProfileV2, RecursiveSecurityBudgetManifestV2, RegistryLifecycleV2,
        PLONKY3_PUBLISH_BYTES_V2, PLONKY3_TARGET_BYTES_V2, RECURSIVE_INGRESS_BYTES_V2,
        RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
    },
    checkpoint::{
        CheckpointDraft, CheckpointExecInput, CheckpointExecOut, CheckpointExecTx,
        CheckpointExecVersion, CheckpointFsStore, CheckpointId, CheckpointInRef, CheckpointStore,
        CheckpointVersion, CreatedEnt, SpentEnt,
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
                "p3_batch_stark=info,p3_circuit_prover=info,p3_fri=info,p3_dft=info",
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
    let draft = CheckpointDraft::new_settlement(
        CheckpointVersion::CURRENT,
        1,
        pre_settlement_root,
        post_settlement_root,
        vec![SpentEnt::new([0x51; 32])],
        vec![CreatedEnt::new([0x52; 32], [0x53; 32])],
    );
    let (snapshot, snapshot_id) =
        build_snapshot_v2(pre_settlement_root, Vec::new()).expect("prep snapshot");
    let mut prep_store = PrepFsStore::new(root);
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
    let mut checkpoint_store = CheckpointFsStore::new(root);
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
    (checkpoint_store, prep_store, link.checkpoint_id())
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

#[test]
#[ignore = "real Plonky3 prover acceptance runs only through plonky3_resource_worker.sh"]
fn predicate_differential() {
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
fn transcript_and_actual_proof_mutations_reject() {
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
    let mut mutated = None;
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
        if let Ok(decoded) = Plonky3BaseProofV2::decode_local_with_source(&candidate, &proof) {
            mutated = Some(decoded);
            break;
        }
    }
    let mutated = mutated.expect("canonical recursive-root mutation candidate");
    resource_phase("verifying");
    assert!(
        Plonky3BaseAdapterV2::verify(&mut transition, &store, &mutated).is_err(),
        "the actual verifier must reject a proof mutation with a repaired outer digest"
    );
    resource_phase("verify_complete");
}

#[test]
fn security_budget_mutations_reject() {
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
fn test_base_proof_size_budget_rejects_before_decode() {
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
fn base_proof_lifecycle_is_local_only() {
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
