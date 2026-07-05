use std::{
    ffi::OsString,
    sync::{Arc, Mutex, OnceLock},
};

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
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, encode_link_bin, CheckpointDraft,
        CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointInRef, CheckpointLink, CheckpointLinkVersion, CheckpointVersion,
    },
    settlement::{CheckRoot, ClaimNullifier, DefinitionId, PublicationRouteSnapshotV1, SerialId},
    snapshot::PrepSnapshotId,
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
const TX_VERSION: u8 = 1;
const CHAIN_TYPE: &str = "rollup_settlement";
const CHAIN_NAME: &str = "rollup-settlement";
const RECEIVER_SECRET: [u8; 32] = [0x11; 32];
const SNAP_ID: PrepSnapshotId = PrepSnapshotId::new([0x44; 32]);
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

fn package_fixture() -> (TxPackage, CheckRoot) {
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
    (package, prev_root)
}

fn exec_out_from_output(output: &TxOutputWire) -> CheckpointExecOut {
    let wire = output.asset_wire.clone().to_wire().expect("output wire");
    let leaf = asset_wire_to_leaf(&wire).expect("output leaf");
    CheckpointExecOut::new(DefinitionId::new(wire.definition.id), leaf).expect("exec output")
}

fn exec_input_from_package(package: &TxPackage, prev_root: CheckRoot) -> CheckpointExecInput {
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
    CheckpointExecInput::new(CheckpointExecVersion::CURRENT, SNAP_ID, prev_root, vec![tx])
        .expect("exec input")
}

pub fn publication_request(batch_bytes: [u8; 32], replay_id: &str) -> PublicationRequest {
    publication_request_with_route(
        batch_bytes,
        replay_id,
        PublicationRouteSnapshotV1::new(1, [0x51; 32], 11, vec![1]),
    )
}

pub fn publication_request_with_route(
    batch_bytes: [u8; 32],
    replay_id: &str,
    publication_route: PublicationRouteSnapshotV1,
) -> PublicationRequest {
    let (tx_package, prev_root) = package_fixture();
    let exec_input = exec_input_from_package(&tx_package, prev_root);
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
    let proof = draft
        .attest_proof(exec_input.prep_snapshot_id(), exec_id)
        .expect("checkpoint proof");
    let artifact = draft.clone().finalize(proof).expect("checkpoint artifact");
    let checkpoint_id = derive_checkpoint_id(&artifact).expect("checkpoint id");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        exec_input.prep_snapshot_id(),
        exec_id,
    )
    .expect("checkpoint link");
    let ordered_batch = ordered_batch_fixture(
        BatchId::from_bytes(batch_bytes),
        &publication_route,
        &tx_package,
    );
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
    PublicationRequest {
        batch_id: BatchId::from_bytes(batch_bytes),
        ordered_batch,
        publication_route,
        draft,
        subject,
        certificate,
        tx_package,
        exec_input,
        link,
        nullifiers: vec![ClaimNullifier::new([batch_bytes[0].wrapping_add(0x40); 32])],
        idempotency_key: replay_id.to_string(),
    }
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
