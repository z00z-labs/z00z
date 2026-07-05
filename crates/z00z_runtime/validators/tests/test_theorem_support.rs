use std::{
    ffi::OsString,
    sync::{Arc, Mutex, OnceLock},
};

use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, AggregatorId, BatchId, BatchPlanned,
    BatchRoute, CommitSubject, IngressBoundary, JournalCandidate, OrderedBatch, PlanDigest,
    PublishedBatch, SecondaryState, ShardId, ShardPlacementView, ShardQuorumCertificate, ShardVote,
    ShardVoteKind, ShardVoteRole, WorkPayload,
};
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, AssetPkgWire, AssetWire};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, CheckpointArtifact, CheckpointDraft,
        CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointInRef, CheckpointLink, CheckpointLinkVersion, CheckpointVersion,
    },
    settlement::{CheckRoot, DefinitionId, SerialId},
    snapshot::PrepSnapshotId,
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_validators::SettlementTheoremBundle;
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

fn checkpoint_bundle(exec_input: &CheckpointExecInput) -> (CheckpointArtifact, CheckpointLink) {
    let exec_bytes = encode_exec_bin(exec_input).expect("exec encode");
    let exec_id = derive_exec_id(&exec_bytes);
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        11,
        exec_input.prev_root(),
        CheckRoot::new([0x88; 32]),
        Vec::new(),
        Vec::new(),
    );
    let proof = draft
        .attest_proof(exec_input.prep_snapshot_id(), exec_id)
        .expect("checkpoint proof");
    let artifact = draft.finalize(proof).expect("checkpoint artifact");
    let checkpoint_id = derive_checkpoint_id(&artifact).expect("checkpoint id");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        exec_input.prep_snapshot_id(),
        exec_id,
    )
    .expect("checkpoint link");
    (artifact, link)
}

pub fn settlement_fixture() -> (
    TxPackage,
    CheckpointExecInput,
    CheckpointArtifact,
    CheckpointLink,
) {
    let (package, prev_root) = package_fixture();
    let exec_input = exec_input_from_package(&package, prev_root);
    let (artifact, link) = checkpoint_bundle(&exec_input);
    (package, exec_input, artifact, link)
}

pub fn theorem_bundle() -> SettlementTheoremBundle {
    let (tx_package, exec_input, artifact, link) = settlement_fixture();
    SettlementTheoremBundle::new(tx_package, artifact, exec_input, link).expect("theorem bundle")
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuorumFixture {
    pub theorem: SettlementTheoremBundle,
    pub ordered: OrderedBatch,
    pub placement: ShardPlacementView,
    pub subject: CommitSubject,
    pub certificate: ShardQuorumCertificate,
    pub published: PublishedBatch,
}

#[allow(dead_code)]
pub fn quorum_fixture(batch_id: BatchId) -> QuorumFixture {
    let theorem = theorem_bundle();
    let item = IngressBoundary
        .normalize(WorkPayload::Tx(Box::new(theorem.tx_package().clone())))
        .expect("ingress normalize");
    let ordered = OrderedBatch {
        batch_id,
        items: vec![item.clone()],
        created_leaves: Vec::new(),
        planned: BatchPlanned {
            batch_id,
            route: BatchRoute {
                shard_id: ShardId::new(1),
                routing_generation: 7,
            },
            route_table_digest: PlanDigest::new([0x51; 32]),
            intake_ids: vec![item.intake_id().clone()],
            op_count: 1,
            plan_digest: PlanDigest::new([0x52; 32]),
        },
    };
    let placement = ShardPlacementView {
        route: ordered.planned.route,
        primary_id: AggregatorId::new(3),
        secondaries: vec![
            SecondaryState::ready(AggregatorId::new(4)),
            SecondaryState::ready(AggregatorId::new(5)),
        ],
        expected_journal_lineage: [0x61; 32],
    };
    let checkpoint_id = derive_checkpoint_id(theorem.artifact()).expect("checkpoint id");
    let publication = bind_publication_contract(
        batch_id,
        checkpoint_id,
        ordered.planned.route_table_digest.into_bytes(),
        &theorem.artifact().pub_in(),
    );
    let candidate = JournalCandidate {
        batch_id,
        route: ordered.planned.route,
        state_root: theorem.artifact().pub_in().new_settlement_root(),
        journal_lineage: placement.expected_journal_lineage,
        version: 0,
        root_generation: 0,
        proof_version: 0,
        bucket_policy_generation: 0,
        bucket_policy_id: [0x71; 32],
    };
    let subject = CommitSubject::from_runtime(
        7,
        membership_digest_for_voters(
            ordered.planned.route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        ),
        &ordered,
        &candidate,
        &publication,
        theorem.theorem_digest(),
        None,
    )
    .expect("commit subject");
    let votes = [
        ShardVote::new_local(
            placement.primary_id,
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
    ];
    let certificate = ShardQuorumCertificate::new(
        &subject,
        placement.primary_id,
        placement
            .secondaries
            .iter()
            .filter(|secondary| secondary.is_ready)
            .map(|secondary| secondary.aggregator_id),
        &votes,
    )
    .expect("quorum certificate");
    let published = PublishedBatch {
        batch_id,
        checkpoint_id,
        publication_checkpoint: 11,
        publication_route: z00z_storage::settlement::PublicationRouteSnapshotV1::new(
            7,
            [0x51; 32],
            10,
            vec![1],
        ),
        pub_in: theorem.artifact().pub_in(),
        subject_digest: Some(subject.digest()),
        certificate_digest: Some(certificate.digest()),
        theorem_digest: Some(theorem.theorem_digest()),
        da_provider: "local-da".to_string(),
        blob_ref: "blob://hjmt-publication".to_string(),
    };

    QuorumFixture {
        theorem,
        ordered,
        placement,
        subject,
        certificate,
        published,
    }
}
