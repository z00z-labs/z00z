#[path = "support/test_theorem_fixture.rs"]
mod theorem_fixture;

use std::{
    ffi::OsString,
    sync::{Arc, Mutex, OnceLock},
};

use z00z_core::assets::{Asset, AssetClass, AssetDefinition, AssetPkgWire, AssetWire};
use z00z_rollup_node::{
    verify_settlement_theorem, DaAdapter, LocalDaAdapter, SettlementError, SettlementTheorem,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, CheckpointDraft,
        CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointId, CheckpointInRef, CheckpointLink, CheckpointLinkVersion, CheckpointVersion,
    },
    settlement::{CheckRoot, DefinitionId, SerialId},
    snapshot::PrepSnapshotId,
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_validators::{ObjectPolicyRegistryV1, RejectClass, ValidatorBoundary, VerdictKind};
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

fn checkpoint_bundle(
    exec_input: &CheckpointExecInput,
) -> (z00z_storage::checkpoint::CheckpointArtifact, CheckpointLink) {
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

fn settlement_fixture() -> (
    TxPackage,
    CheckpointExecInput,
    z00z_storage::checkpoint::CheckpointArtifact,
    CheckpointLink,
) {
    let (package, prev_root) = package_fixture();
    let exec_input = exec_input_from_package(&package, prev_root);
    let (artifact, link) = checkpoint_bundle(&exec_input);
    (package, exec_input, artifact, link)
}

#[test]
fn validator_rejects_detached_certificate_binding() {
    let request = theorem_fixture::publication_request([0x91; 32], "theorem-detached-cert");
    let mut adapter = LocalDaAdapter::new("local-theorem");
    let published = adapter.publish(request).expect("publish");
    let mut resolved = adapter.resolve(&published).expect("resolve");
    resolved.certificate = None;

    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::AuthInvalid));
}

#[test]
fn validator_rejects_detached_publication_binding() {
    let request = theorem_fixture::publication_request([0x92; 32], "theorem-detached-pub");
    let mut adapter = LocalDaAdapter::new("local-theorem");
    let published = adapter.publish(request).expect("publish");
    let mut resolved = adapter.resolve(&published).expect("resolve");
    resolved
        .subject
        .as_mut()
        .expect("subject")
        .publication_binding_digest = [0xA4; 32];

    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::ReconcileInvalid));
}

fn exec_input_with_prev_root(
    exec_input: &CheckpointExecInput,
    prev_root: CheckRoot,
) -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        exec_input.prep_snapshot_id(),
        prev_root,
        exec_input.txs().to_vec(),
    )
    .expect("exec input with root")
}

fn exec_input_with_bad_proof(exec_input: &CheckpointExecInput) -> CheckpointExecInput {
    let tx_row = exec_input.txs().first().expect("exec tx");
    let bad_tx = CheckpointExecTx::new(
        tx_row.input_refs().to_vec(),
        tx_row.outputs().to_vec(),
        vec![0xAA],
    )
    .expect("bad exec tx");
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        exec_input.prep_snapshot_id(),
        exec_input.prev_root(),
        vec![bad_tx],
    )
    .expect("bad exec input")
}

fn artifact_value(artifact: &z00z_storage::checkpoint::CheckpointArtifact) -> serde_json::Value {
    serde_json::from_slice(&JsonCodec.serialize(artifact).expect("artifact json bytes"))
        .expect("artifact json value")
}

fn artifact_from_value(value: serde_json::Value) -> z00z_storage::checkpoint::CheckpointArtifact {
    JsonCodec
        .deserialize(&serde_json::to_vec(&value).expect("artifact json encode"))
        .expect("artifact from json")
}

#[test]
fn test_settlement_accepts_bundle() {
    let (package, exec_input, artifact, link) = settlement_fixture();

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &exec_input,
        link: &link,
    });

    assert!(
        result.is_ok(),
        "settlement theorem must accept canonical bundle: {result:?}"
    );
}

#[test]
fn test_settlement_rejects_checkpoint_replay() {
    let (package, exec_input, artifact, link) = settlement_fixture();
    let exec_input = exec_input_with_bad_proof(&exec_input);

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &exec_input,
        link: &link,
    });

    assert!(
        matches!(result, Err(SettlementError::CheckpointReplay)),
        "expected checkpoint replay failure, got {result:?}"
    );
}

#[test]
fn test_settlement_rejects_tx_missing() {
    let (package, exec_input, _, _) = settlement_fixture();
    let (other_package, _) = package_fixture();
    let other_exec = exec_input_from_package(&other_package, exec_input.prev_root());
    let (artifact, link) = checkpoint_bundle(&other_exec);

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &other_exec,
        link: &link,
    });

    assert!(
        matches!(result, Err(SettlementError::TxMissing)),
        "expected missing tx failure, got {result:?}"
    );
}

#[test]
fn test_settlement_rejects_root_mismatch() {
    let (package, exec_input, _, _) = settlement_fixture();
    let bad_exec = exec_input_with_prev_root(&exec_input, CheckRoot::new([0x99; 32]));
    let (artifact, link) = checkpoint_bundle(&bad_exec);

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &bad_exec,
        link: &link,
    });

    assert!(
        matches!(result, Err(SettlementError::CheckpointRoot)),
        "expected checkpoint root failure, got {result:?}"
    );
}

#[test]
fn test_settlement_rejects_bad_link() {
    let (package, exec_input, artifact, _) = settlement_fixture();
    let exec_bytes = encode_exec_bin(&exec_input).expect("exec encode");
    let exec_id = derive_exec_id(&exec_bytes);
    let bad_link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new([0x99; 32]),
        SNAP_ID,
        exec_id,
    )
    .expect("bad link builds");

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &exec_input,
        link: &bad_link,
    });

    assert!(
        matches!(result, Err(SettlementError::CheckpointLink)),
        "expected checkpoint link failure, got {result:?}"
    );
}

#[test]
fn test_settlement_rejects_detached_statement() {
    let (package, exec_input, artifact, link) = settlement_fixture();
    let mut value = artifact_value(&artifact);
    value["prep_snapshot_id"] = serde_json::Value::Null;
    value["exec_input_id"] = serde_json::Value::Null;
    let artifact = artifact_from_value(value);

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &exec_input,
        link: &link,
    });

    assert!(
        matches!(result, Err(SettlementError::CheckpointStatement)),
        "expected detached statement failure, got {result:?}"
    );
}

#[test]
fn test_settlement_rejects_checkpoint_proof() {
    let (package, exec_input, artifact, link) = settlement_fixture();
    let mut value = artifact_value(&artifact);
    value["cp_proof"] = serde_json::json!([170, 187, 204]);
    let artifact = artifact_from_value(value);

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &exec_input,
        link: &link,
    });

    assert!(
        matches!(result, Err(SettlementError::CheckpointProof)),
        "expected checkpoint proof failure, got {result:?}"
    );
}

#[test]
fn test_settlement_rejects_prep_mismatch() {
    let (package, exec_input, artifact, link) = settlement_fixture();
    let mut value = artifact_value(&artifact);
    value["prep_snapshot_id"] = serde_json::json!(vec![0x55u8; 32]);
    let artifact = artifact_from_value(value);

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &exec_input,
        link: &link,
    });

    assert!(
        matches!(result, Err(SettlementError::CheckpointLink)),
        "expected prep mismatch failure, got {result:?}"
    );
}

#[test]
fn test_settlement_rejects_bad_package() {
    let (mut package, exec_input, artifact, link) = settlement_fixture();
    package
        .tx
        .proof
        .spend
        .as_mut()
        .expect("spend proof")
        .proof_hex = hex::encode([0xAB; 32]);

    let result = verify_settlement_theorem(&SettlementTheorem {
        tx_package: &package,
        artifact: &artifact,
        exec_input: &exec_input,
        link: &link,
    });

    assert!(matches!(result, Err(SettlementError::TxTheorem(_))));
}
