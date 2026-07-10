use std::{
    ffi::OsString,
    sync::{Arc, Mutex, OnceLock},
};

use sha2::{Digest, Sha256};
use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchId, BatchPlanned,
    BatchRoute, CommitSubject, IngressBoundary, JournalCandidate, PlanDigest, PublicationRequest,
    SecondaryState, ShardId, ShardPlacementView, ShardQuorumCertificate, ShardVote, ShardVoteKind,
    ShardVoteRole, WorkPayload,
};
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, AssetPkgWire, AssetWire};
use z00z_crypto::{expert::traits::DomainSeparation, DomainHasher256};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, encode_link_bin,
        ArchiveManifestVersion, CheckpointArchiveEncodingKindV1, CheckpointArchiveEntryKindV1,
        CheckpointArchiveEntryV1, CheckpointArchiveEntryVersion, CheckpointArchiveManifestV1,
        CheckpointArchiveRetentionClassV1, CheckpointArtifact, CheckpointDaLocatorKind,
        CheckpointDaProviderFamily, CheckpointDaReferenceV1, CheckpointDaReferenceVersion,
        CheckpointDraft, CheckpointExecInput, CheckpointExecOut, CheckpointExecTx,
        CheckpointExecVersion, CheckpointId, CheckpointInRef, CheckpointLink,
        CheckpointLinkVersion, CheckpointTransitionStatementCoreV1,
        CheckpointTransitionStatementFinalV1, CheckpointTransitionStatementV1, CheckpointVersion,
    },
    settlement::{
        CheckRoot, ClaimNullifier, DefinitionId, PublicationRouteSnapshotV1, SerialId,
        SettlementPath, SettlementStore, SnapItem, StoreItem, TerminalId,
    },
    snapshot::{build_snapshot, PrepSnapshot, PrepSnapshotId},
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::{bind_stealth_output_wire, build_card_stealth_leaf},
    tx::{
        asset_wire_to_leaf, build_public_spend_contract, build_tx_package_digest,
        prepare_spend_membership_witnesses, prepare_spend_public_inputs, resolve_input_pack,
        verify_package_public_spend_contract, SpendProofWitness, TxInputWire, TxOutRole,
        TxOutputWire, TxPackage, TxWire,
    },
};

const CHAIN_ID: u32 = 3;

#[derive(Clone)]
struct PreviewPublicationContract {
    artifact: CheckpointArtifact,
    checkpoint_id: CheckpointId,
}

fn publication_height_for_request(
    draft: &CheckpointDraft,
    publication_route: &PublicationRouteSnapshotV1,
) -> u64 {
    draft
        .height()
        .max(publication_route.activation_checkpoint)
        .max(1)
}

#[allow(clippy::too_many_arguments)]
fn preview_publication_contract_parts(
    batch_id: BatchId,
    replay_id: &str,
    publication_route: &PublicationRouteSnapshotV1,
    draft: &CheckpointDraft,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    nullifiers: &[ClaimNullifier],
    provider_family: CheckpointDaProviderFamily,
) -> Result<PreviewPublicationContract, Box<dyn std::error::Error>> {
    let payload_commitment = payload_commitment_from_parts(
        batch_id,
        replay_id,
        publication_route,
        draft,
        tx_package,
        exec_input,
        nullifiers,
    )?;
    let publication_height = publication_height_for_request(draft, publication_route);
    let statement_core = statement_core_from_parts(
        batch_id,
        publication_route,
        tx_package,
        exec_input,
        nullifiers,
    )?;
    let exec_bytes = encode_exec_bin(exec_input)?;
    let exec_input_id = derive_exec_id(&exec_bytes);
    let archive_manifest = archive_manifest_from_parts(
        batch_id,
        publication_route,
        draft,
        tx_package,
        exec_input,
        exec_input_id,
        payload_commitment,
        statement_core,
    )?;
    let da_reference = CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        provider_family,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        canonical_locator_value(
            batch_id,
            provider_family,
            publication_height,
            payload_commitment,
        ),
        payload_commitment,
        archive_manifest.statement_core_digest(),
        archive_manifest.archive_manifest_root(),
        publication_height,
    )?;
    let proof = draft.attest_proof(exec_input.prep_snapshot_id(), exec_input_id)?;
    let artifact = draft.finalize(proof)?.bind_canonical_v1(
        statement_core,
        CheckpointTransitionStatementFinalV1::new(da_reference.da_ref()),
    )?;
    let checkpoint_id = derive_checkpoint_id(&artifact)?;
    Ok(PreviewPublicationContract {
        artifact,
        checkpoint_id,
    })
}

#[allow(clippy::too_many_arguments)]
fn payload_commitment_from_parts(
    batch_id: BatchId,
    replay_id: &str,
    publication_route: &PublicationRouteSnapshotV1,
    draft: &CheckpointDraft,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    nullifiers: &[ClaimNullifier],
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let pub_in = draft.pub_in();
    let pub_in_bytes = JsonCodec.serialize(&pub_in)?;
    let publication_route_bytes = JsonCodec.serialize(publication_route)?;
    let nullifier_bytes = nullifier_bytes(nullifiers);
    let tx_package_bytes = JsonCodec.serialize(tx_package)?;
    let exec_input_bytes = encode_exec_bin(exec_input)?;
    Ok(hash_parts(
        b"z00z.rollup.local-da.payload.v1",
        &[
            &batch_id.into_bytes(),
            replay_id.as_bytes(),
            &publication_route_bytes,
            &pub_in_bytes,
            &tx_package_bytes,
            &exec_input_bytes,
            &nullifier_bytes,
        ],
    ))
}

fn statement_core_from_parts(
    batch_id: BatchId,
    publication_route: &PublicationRouteSnapshotV1,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    nullifiers: &[ClaimNullifier],
) -> Result<CheckpointTransitionStatementCoreV1, Box<dyn std::error::Error>> {
    let exec_bytes = encode_exec_bin(exec_input)?;
    let route_bytes = JsonCodec.serialize(publication_route)?;
    let tx_package_bytes = JsonCodec.serialize(tx_package)?;
    let nullifier_bytes = nullifier_bytes(nullifiers);
    let delta_root = hash_parts(
        b"z00z.rollup.checkpoint.delta-root.v1",
        &[&batch_id.into_bytes(), &route_bytes, &exec_bytes],
    );
    let witness_root = hash_parts(
        b"z00z.rollup.checkpoint.witness-root.v1",
        &[&exec_bytes, &tx_package_bytes],
    );
    let journal_digest = hash_parts(
        b"z00z.rollup.checkpoint.journal-digest.v1",
        &[&batch_id.into_bytes(), &nullifier_bytes, &route_bytes],
    );
    Ok(CheckpointTransitionStatementCoreV1::from_exec(
        exec_input,
        delta_root,
        witness_root,
        journal_digest,
    ))
}

#[allow(clippy::too_many_arguments)]
fn archive_manifest_from_parts(
    batch_id: BatchId,
    publication_route: &PublicationRouteSnapshotV1,
    draft: &CheckpointDraft,
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
    exec_input_id: z00z_storage::checkpoint::CheckpointExecInputId,
    payload_commitment: [u8; 32],
    statement_core: CheckpointTransitionStatementCoreV1,
) -> Result<CheckpointArchiveManifestV1, Box<dyn std::error::Error>> {
    let route_bytes = JsonCodec.serialize(publication_route)?;
    let tx_package_bytes = JsonCodec.serialize(tx_package)?;
    let exec_bytes = encode_exec_bin(exec_input)?;
    let proof_bytes = exact_tx_proof_bytes(exec_input);
    let statement_core_digest = CheckpointTransitionStatementV1::from_draft(
        draft,
        exec_input.prep_snapshot_id(),
        exec_input_id,
    )
    .statement_core_digest_v1(&statement_core);
    let epoch_manifest_root = hash_parts(
        b"z00z.rollup.checkpoint.epoch-manifest-root.v1",
        &[&batch_id.into_bytes(), &route_bytes, &payload_commitment],
    );
    let raw_tx_package_root = hash_parts(
        b"z00z.rollup.checkpoint.raw-tx-package-root.v1",
        &[&tx_package_bytes],
    );
    let exact_tx_proof_bytes_root = hash_parts(
        b"z00z.rollup.checkpoint.exact-tx-proof-root.v1",
        &[&proof_bytes],
    );
    let witness_archive_root = hash_parts(
        b"z00z.rollup.checkpoint.witness-archive-root.v1",
        &[&statement_core.witness_root(), &exec_bytes],
    );
    let delta_journal_root = hash_parts(
        b"z00z.rollup.checkpoint.delta-journal-root.v1",
        &[
            &statement_core.delta_root(),
            &statement_core.journal_digest(),
        ],
    );
    let archive_provider_receipt_root = hash_parts(
        b"z00z.rollup.checkpoint.archive-provider-receipt-root.v1",
        &[&payload_commitment, &route_bytes],
    );
    let retrieval_audit_root = hash_parts(
        b"z00z.rollup.checkpoint.retrieval-audit-root.v1",
        &[&payload_commitment, &batch_id.into_bytes()],
    );
    let content_address_root = hash_parts(
        b"z00z.rollup.checkpoint.content-address-root.v1",
        &[&payload_commitment, &tx_package_bytes],
    );
    let entries = vec![
        archive_entry(
            CheckpointArchiveEntryKindV1::RawTxPackage,
            0,
            raw_tx_package_root,
            tx_package_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::ArchiveRequired,
            CheckpointArchiveEncodingKindV1::CanonicalJsonV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ExactTxProofBytes,
            1,
            exact_tx_proof_bytes_root,
            proof_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::WitnessArchive,
            2,
            witness_archive_root,
            exec_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::DeltaJournal,
            3,
            delta_journal_root,
            route_bytes.len() as u64,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalJsonV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ArchiveProviderReceipt,
            4,
            archive_provider_receipt_root,
            payload_commitment.len() as u64,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::RetrievalAudit,
            5,
            retrieval_audit_root,
            payload_commitment.len() as u64,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ContentAddressIndex,
            6,
            content_address_root,
            payload_commitment.len() as u64,
            CheckpointArchiveRetentionClassV1::ArchiveRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
    ];
    Ok(CheckpointArchiveManifestV1::new(
        ArchiveManifestVersion::CURRENT,
        statement_core_digest,
        exec_input_id,
        exec_input.prep_snapshot_id(),
        statement_core.tx_data_root(),
        statement_core.delta_root(),
        statement_core.witness_root(),
        statement_core.journal_digest(),
        epoch_manifest_root,
        raw_tx_package_root,
        exact_tx_proof_bytes_root,
        witness_archive_root,
        delta_journal_root,
        payload_commitment,
        archive_provider_receipt_root,
        retrieval_audit_root,
        content_address_root,
        entries,
        3,
    )?)
}

fn archive_entry(
    entry_kind: CheckpointArchiveEntryKindV1,
    ordinal: u32,
    content_digest: [u8; 32],
    byte_length: u64,
    retention_class: CheckpointArchiveRetentionClassV1,
    encoding_kind: CheckpointArchiveEncodingKindV1,
) -> Result<CheckpointArchiveEntryV1, Box<dyn std::error::Error>> {
    Ok(CheckpointArchiveEntryV1::new(
        CheckpointArchiveEntryVersion::CURRENT,
        entry_kind,
        ordinal,
        content_digest,
        byte_length,
        retention_class,
        encoding_kind,
    )?)
}

fn exact_tx_proof_bytes(exec_input: &CheckpointExecInput) -> Vec<u8> {
    let mut out = Vec::new();
    for tx in exec_input.txs() {
        out.extend_from_slice(&(tx.tx_proof().len() as u64).to_le_bytes());
        out.extend_from_slice(tx.tx_proof());
    }
    out
}

fn canonical_locator_value(
    batch_id: BatchId,
    provider_family: CheckpointDaProviderFamily,
    publication_height: u64,
    payload_commitment: [u8; 32],
) -> String {
    format!(
        "checkpoint-da://{}/{}/{}/{}",
        provider_family.as_str(),
        hex::encode(batch_id.into_bytes()),
        publication_height,
        hex::encode(payload_commitment)
    )
}

fn nullifier_bytes(nullifiers: &[ClaimNullifier]) -> Vec<u8> {
    let mut out = Vec::with_capacity(nullifiers.len() * 32);
    for nullifier in nullifiers {
        out.extend_from_slice(nullifier.as_bytes());
    }
    out
}

fn hash_parts(label: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    for part in parts {
        hasher.update((*part).len().to_le_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}
const TX_VERSION: u8 = 1;
const CHAIN_TYPE: &str = "rollup_settlement";
const CHAIN_NAME: &str = "rollup-settlement";
const RECEIVER_SECRET: [u8; 32] = [0x11; 32];
const SETTLEMENT_THEOREM_DIGEST_TAG: &[u8] = b"z00z.settlement_theorem.bundle";

struct SettlementTheoremDigestDomain;

impl DomainSeparation for SettlementTheoremDigestDomain {
    fn version() -> u8 {
        1
    }

    fn domain() -> &'static str {
        "z00z.settlement_theorem.digest"
    }
}

struct RangeProofEnvGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
    prev: Option<OsString>,
}

impl RangeProofEnvGuard {
    fn new() -> Self {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let guard = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let prev = std::env::var_os("Z00Z_ALLOW_DEBUG_RANGE_PROOF");
        std::env::set_var("Z00Z_ALLOW_DEBUG_RANGE_PROOF", "1");
        Self {
            _guard: guard,
            prev,
        }
    }
}

impl Drop for RangeProofEnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.prev.take() {
            std::env::set_var("Z00Z_ALLOW_DEBUG_RANGE_PROOF", value);
        } else {
            std::env::remove_var("Z00Z_ALLOW_DEBUG_RANGE_PROOF");
        }
    }
}

fn receiver_keys() -> ReceiverKeys {
    ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(RECEIVER_SECRET).expect("receiver secret"),
    )
    .expect("receiver keys")
}

fn asset_fixture(serial_id: u32, amount: u64) -> Asset {
    let definition = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Rollup Settlement Coin".to_string(),
        "RSC".to_string(),
        8,
        1024,
        100_000_000,
        "rollup.settlement.test".to_string(),
        1,
        1,
        0,
        None,
    )
    .expect("asset definition");
    Asset::new_confidential(
        Arc::new(definition),
        serial_id,
        amount,
        [serial_id as u8; 32],
    )
    .expect("asset")
    .0
}

fn tx_inputs_for_wires(inputs: &[AssetWire]) -> Vec<TxInputWire> {
    inputs
        .iter()
        .map(|wire| TxInputWire {
            asset_id_hex: hex::encode(asset_wire_to_leaf(wire).expect("input leaf").asset_id),
            serial_id: wire.serial_id,
        })
        .collect()
}

fn snapshot_fixture(
    input_wire: &AssetWire,
    tx_input: &TxInputWire,
    prev_root: CheckRoot,
) -> (PrepSnapshot, PrepSnapshotId) {
    let asset_id: [u8; 32] = hex::decode(&tx_input.asset_id_hex)
        .expect("asset id hex")
        .try_into()
        .expect("asset id bytes");
    let mut leaf = asset_wire_to_leaf(input_wire).expect("snapshot leaf");
    leaf.set_terminal_id(TerminalId::new(asset_id));
    let path = SettlementPath::new(
        DefinitionId::new(input_wire.definition.id),
        SerialId::new(input_wire.serial_id),
        TerminalId::new(asset_id),
    );
    let mut store = SettlementStore::try_new().expect("settlement store");
    store
        .put_settlement_item(StoreItem::new(path, leaf.clone()).expect("store item"))
        .expect("put settlement item");
    assert_eq!(
        CheckRoot::from(store.settlement_root().expect("settlement root")),
        prev_root
    );
    let witness = store
        .settlement_proof_blob(&path)
        .expect("proof blob")
        .encode()
        .expect("encode proof blob");
    let entry = SnapItem::new(path, leaf, witness).expect("snap item");
    build_snapshot(prev_root, vec![entry]).expect("snapshot")
}

fn package_fixture() -> (TxPackage, CheckRoot, PrepSnapshot, PrepSnapshotId) {
    let _guard = RangeProofEnvGuard::new();
    let keys = receiver_keys();
    let card = keys.export_receiver_card().expect("receiver card");
    let input_asset = asset_fixture(7, 55);
    let input_leaf = build_card_stealth_leaf(&card, input_asset.amount, input_asset.serial_id)
        .expect("input leaf");
    let input_wire = bind_stealth_output_wire(AssetWire::from_asset(&input_asset), &input_leaf)
        .expect("input wire");
    let mut output_wire = input_wire.clone();
    output_wire.nonce[0] ^= 0x55;
    output_wire.leaf_ad_id = Some([0x77; 32]);

    let tx_input = tx_inputs_for_wires(std::slice::from_ref(&input_wire))
        .pop()
        .expect("tx input");
    let tx_output = TxOutputWire {
        role: TxOutRole::Recipient,
        asset_wire: AssetPkgWire::from_wire(&output_wire),
    };
    let proof_inputs = prepare_spend_public_inputs(
        CHAIN_ID,
        RECEIVER_SECRET,
        std::slice::from_ref(&input_wire),
        std::slice::from_ref(&tx_input),
    )
    .expect("proof inputs");
    let (prev_root, membership) = prepare_spend_membership_witnesses(
        std::slice::from_ref(&input_wire),
        std::slice::from_ref(&tx_input),
    )
    .expect("membership witnesses");
    let (snapshot, snapshot_id) = snapshot_fixture(&input_wire, &tx_input, prev_root);
    let mut tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![tx_input],
        outputs: vec![tx_output],
        fee: 0,
        nonce: 0,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let (proof, auth) = build_public_spend_contract(
        &keys,
        CHAIN_ID,
        TX_VERSION,
        CHAIN_TYPE,
        CHAIN_NAME,
        &tx,
        prev_root,
        proof_inputs,
        SpendProofWitness {
            receiver_secret: ReceiverSecret::from_bytes(RECEIVER_SECRET).expect("receiver secret"),
            input_s_in: vec![
                resolve_input_pack(RECEIVER_SECRET, &input_wire)
                    .expect("input pack")
                    .s_out,
            ],
            membership,
        },
    )
    .expect("public spend contract");
    tx.proof = proof;
    tx.auth = auth;
    let digest = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        TX_VERSION,
        CHAIN_ID,
        CHAIN_TYPE,
        CHAIN_NAME,
        &tx,
    )
    .expect("package digest");
    let package = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: TX_VERSION,
        chain_id: CHAIN_ID,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx,
        tx_digest_hex: digest,
        status: "prepared".to_string(),
    };
    verify_package_public_spend_contract(&package).expect("package theorem");
    (package, prev_root, snapshot, snapshot_id)
}

fn exec_out_from_output(output: &TxOutputWire) -> CheckpointExecOut {
    let wire = output.asset_wire.clone().to_wire().expect("output wire");
    let leaf = asset_wire_to_leaf(&wire).expect("output leaf");
    CheckpointExecOut::new(DefinitionId::new(wire.definition.id), leaf).expect("exec output")
}

fn exec_input_from_package(
    snapshot_id: PrepSnapshotId,
    package: &TxPackage,
    prev_root: CheckRoot,
) -> CheckpointExecInput {
    let input_refs = package
        .tx
        .inputs
        .iter()
        .map(|input| {
            let asset_id: [u8; 32] = hex::decode(&input.asset_id_hex)
                .expect("asset id hex")
                .try_into()
                .expect("asset id bytes");
            CheckpointInRef::new(asset_id, SerialId::new(input.serial_id))
        })
        .collect::<Vec<_>>();
    let outputs = package
        .tx
        .outputs
        .iter()
        .map(exec_out_from_output)
        .collect::<Vec<_>>();
    let tx_proof = JsonCodec
        .serialize(&package.tx.proof)
        .expect("tx proof encode");
    let tx = CheckpointExecTx::new(input_refs, outputs, tx_proof).expect("exec tx");
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        prev_root,
        vec![tx],
    )
    .expect("exec input")
}

#[allow(dead_code)]
pub fn publication_request(batch_bytes: [u8; 32], replay_id: &str) -> PublicationRequest {
    publication_case_with_route_and_provider(
        batch_bytes,
        replay_id,
        PublicationRouteSnapshotV1::new(1, [0x51; 32], 11, vec![1]),
        CheckpointDaProviderFamily::LocalArchive,
    )
    .0
}

#[allow(dead_code)]
pub fn publication_request_with_provider(
    batch_bytes: [u8; 32],
    replay_id: &str,
    provider_family: CheckpointDaProviderFamily,
) -> PublicationRequest {
    publication_case_with_route_and_provider(
        batch_bytes,
        replay_id,
        PublicationRouteSnapshotV1::new(1, [0x51; 32], 11, vec![1]),
        provider_family,
    )
    .0
}

#[allow(dead_code)]
pub fn canonical_artifact_for_request(
    request: &PublicationRequest,
    provider_family: CheckpointDaProviderFamily,
) -> CheckpointArtifact {
    preview_publication_contract_parts(
        request.batch_id,
        &request.idempotency_key,
        &request.publication_route,
        &request.draft,
        &request.tx_package,
        &request.exec_input,
        &request.nullifiers,
        provider_family,
    )
    .expect("preview publication contract")
    .artifact
}

#[allow(dead_code)]
pub fn publication_request_with_route(
    batch_bytes: [u8; 32],
    replay_id: &str,
    publication_route: PublicationRouteSnapshotV1,
) -> PublicationRequest {
    publication_case_with_route_and_provider(
        batch_bytes,
        replay_id,
        publication_route,
        CheckpointDaProviderFamily::LocalArchive,
    )
    .0
}

#[allow(dead_code)]
pub fn publication_case(
    batch_bytes: [u8; 32],
    replay_id: &str,
) -> (PublicationRequest, PrepSnapshot) {
    publication_case_with_route_and_provider(
        batch_bytes,
        replay_id,
        PublicationRouteSnapshotV1::new(1, [0x51; 32], 11, vec![1]),
        CheckpointDaProviderFamily::LocalArchive,
    )
}

#[allow(dead_code)]
fn publication_case_with_route(
    batch_bytes: [u8; 32],
    replay_id: &str,
    publication_route: PublicationRouteSnapshotV1,
) -> (PublicationRequest, PrepSnapshot) {
    publication_case_with_route_and_provider(
        batch_bytes,
        replay_id,
        publication_route,
        CheckpointDaProviderFamily::LocalArchive,
    )
}

fn publication_case_with_route_and_provider(
    batch_bytes: [u8; 32],
    replay_id: &str,
    publication_route: PublicationRouteSnapshotV1,
    provider_family: CheckpointDaProviderFamily,
) -> (PublicationRequest, PrepSnapshot) {
    let (tx_package, prev_root, snapshot, snapshot_id) = package_fixture();
    let exec_input = exec_input_from_package(snapshot_id, &tx_package, prev_root);
    let exec_bytes = encode_exec_bin(&exec_input).expect("exec encode");
    let exec_id = derive_exec_id(&exec_bytes);
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        publication_route.activation_checkpoint.max(11),
        exec_input.prev_root(),
        CheckRoot::new([0x88; 32]),
        Vec::new(),
        Vec::new(),
    );
    let batch_id = BatchId::from_bytes(batch_bytes);
    let ordered_batch = ordered_batch_fixture(batch_id, &publication_route, &tx_package);
    let nullifiers = vec![ClaimNullifier::new([batch_bytes[0].wrapping_add(0x40); 32])];
    let preview = preview_publication_contract_parts(
        batch_id,
        replay_id,
        &publication_route,
        &draft,
        &tx_package,
        &exec_input,
        &nullifiers,
        provider_family,
    )
    .expect("preview publication contract");
    let artifact = preview.artifact.clone();
    let checkpoint_id = preview.checkpoint_id;
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        exec_input.prep_snapshot_id(),
        exec_id,
    )
    .expect("checkpoint link");
    let placement = placement_view(ordered_batch.planned.route);
    let candidate = JournalCandidate {
        batch_id: ordered_batch.batch_id,
        route: ordered_batch.planned.route,
        state_root: artifact.pub_in().new_settlement_root(),
        journal_lineage: placement.expected_journal_lineage,
        version: 0,
        root_generation: 0,
        proof_version: 0,
        bucket_policy_generation: 0,
        bucket_policy_id: [0x63; 32],
    };
    let publication_binding = bind_publication_contract(
        ordered_batch.batch_id,
        checkpoint_id,
        ordered_batch.planned.route_table_digest.into_bytes(),
        &artifact.pub_in(),
    );
    let subject = CommitSubject::from_runtime(
        7,
        membership_digest_for_voters(
            ordered_batch.planned.route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        ),
        &ordered_batch,
        &candidate,
        &publication_binding,
        theorem_digest(&tx_package, &artifact, &exec_input, &link),
        None,
    )
    .expect("commit subject");
    let votes = quorum_votes(&subject);
    let certificate = ShardQuorumCertificate::new(
        &subject,
        placement.primary_id,
        placement
            .secondaries
            .iter()
            .filter(|secondary| secondary.is_ready)
            .map(|secondary| secondary.aggregator_id),
        &votes[..2],
    )
    .expect("quorum certificate");
    (
        PublicationRequest {
            batch_id,
            ordered_batch,
            publication_route,
            draft,
            subject,
            certificate,
            tx_package,
            exec_input,
            link,
            nullifiers,
            idempotency_key: replay_id.to_string(),
        },
        snapshot,
    )
}

fn ordered_batch_fixture(
    batch_id: BatchId,
    publication_route: &PublicationRouteSnapshotV1,
    tx_package: &TxPackage,
) -> z00z_aggregators::OrderedBatch {
    let item = IngressBoundary
        .normalize(WorkPayload::Tx(Box::new(tx_package.clone())))
        .expect("ingress normalize");
    z00z_aggregators::OrderedBatch {
        batch_id,
        items: vec![item.clone()],
        created_leaves: Vec::new(),
        planned: BatchPlanned {
            batch_id,
            route: BatchRoute {
                shard_id: ShardId::new(
                    publication_route.shard_ids.first().copied().unwrap_or(1) as u16
                ),
                routing_generation: publication_route.routing_generation,
            },
            route_table_digest: PlanDigest::new(publication_route.route_table_digest),
            intake_ids: vec![item.intake_id().clone()],
            op_count: 1,
            plan_digest: PlanDigest::new([0x72; 32]),
        },
    }
}

fn placement_view(route: BatchRoute) -> ShardPlacementView {
    ShardPlacementView {
        route,
        primary_id: AggregatorId::new(3),
        secondaries: vec![
            SecondaryState::ready(AggregatorId::new(4)),
            SecondaryState::ready(AggregatorId::new(5)),
        ],
        expected_journal_lineage: [0x62; 32],
    }
}

pub fn quorum_votes(subject: &CommitSubject) -> Vec<ShardVote> {
    vec![
        ShardVote::new_local(
            AggregatorId::new(3),
            ShardVoteRole::Primary,
            subject.shard_id,
            subject.term,
            subject.membership_digest,
            subject.digest(),
            ShardVoteKind::LocalCommit,
        ),
        ShardVote::new_local(
            AggregatorId::new(4),
            ShardVoteRole::Secondary,
            subject.shard_id,
            subject.term,
            subject.membership_digest,
            subject.digest(),
            ShardVoteKind::LocalCommit,
        ),
        ShardVote::new_local(
            AggregatorId::new(5),
            ShardVoteRole::Secondary,
            subject.shard_id,
            subject.term,
            subject.membership_digest,
            subject.digest(),
            ShardVoteKind::LocalCommit,
        ),
    ]
}

fn theorem_digest(
    tx_package: &TxPackage,
    artifact: &z00z_storage::checkpoint::CheckpointArtifact,
    exec_input: &CheckpointExecInput,
    link: &CheckpointLink,
) -> [u8; 32] {
    let tx_digest: [u8; 32] = hex::decode(&tx_package.tx_digest_hex)
        .expect("fixture tx digest must decode")
        .try_into()
        .expect("fixture tx digest must stay 32 bytes");
    let checkpoint_id = derive_checkpoint_id(artifact).expect("fixture checkpoint id");
    let exec_bytes = encode_exec_bin(exec_input).expect("fixture exec input encode");
    let exec_id = derive_exec_id(&exec_bytes);
    let link_bytes = encode_link_bin(link).expect("fixture link encode");

    let mut bytes = Vec::with_capacity(192 + link_bytes.len());
    bytes.extend_from_slice(SETTLEMENT_THEOREM_DIGEST_TAG);
    bytes.push(1);
    push_len_prefixed(&mut bytes, &tx_digest);
    bytes.extend_from_slice(checkpoint_id.as_bytes());
    bytes.extend_from_slice(exec_id.as_bytes());
    push_len_prefixed(&mut bytes, &link_bytes);

    let digest = DomainHasher256::<SettlementTheoremDigestDomain>::new_with_label("digest")
        .chain(bytes)
        .finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(digest.as_ref());
    out
}

fn push_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(bytes);
}
