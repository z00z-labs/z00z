#[path = "test_inc/test_proof_blob_input_case.inc"]
mod proof_blob_input_case;

use std::collections::BTreeMap;

use z00z_core::assets::{AssetClass, AssetLeaf, AssetPkgWire, AssetWire};
use z00z_core::genesis::asset_std::asset_from_dev_class;
use z00z_crypto::{create_commitment, Hidden, Z00ZScalar};
use z00z_storage::settlement::{CheckRoot, TerminalId, TerminalLeaf};
use z00z_utils::rng::MockRngProvider;
use z00z_wallets::tx::fee_estimator::calculate_fee_for_wires;
use z00z_wallets::tx::{
    apply_batch_checkpoint, build_tx_package_digest, prepare_tx_sum, InputResolver, Prover,
    ProverImpl, ResolvedInput, SettlementState, SpentIndex, SpentIndexError, StateError,
    TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire, TxPackage, TxPkgSum,
    TxProofError, TxProofVerifier, TxProofWire, TxVerifier, TxVerifierImpl, TxWire,
};

use self::proof_blob_input_case::wit_input_case;

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

fn mk_wire(class: AssetClass, amount: u64) -> AssetWire {
    let mut asset = asset_from_dev_class(class, 1, amount).expect("asset");
    asset.owner_pub = None;
    asset.owner_signature = None;
    AssetWire::from_asset(&asset)
}

fn calc_fee(outputs: &[AssetWire]) -> u64 {
    calculate_fee_for_wires(1, outputs).expect("fee")
}

fn mk_pkg(outputs: Vec<(TxOutRole, AssetWire)>, fee: u64) -> Vec<u8> {
    let tx_wire = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([1u8; 32]),
            serial_id: 1,
        }],
        outputs: outputs
            .into_iter()
            .map(|(role, asset_wire)| TxOutputWire {
                role,
                asset_wire: AssetPkgWire::from_wire(&asset_wire),
            })
            .collect(),
        fee,
        nonce: 0,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let tx = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: CHAIN_ID,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx: tx_wire.clone(),
        tx_digest_hex: build_tx_package_digest(
            "TxPackage",
            "regular_tx",
            1,
            CHAIN_ID,
            CHAIN_TYPE,
            CHAIN_NAME,
            &tx_wire,
        )
        .expect("digest"),
        status: "prepared".to_string(),
    };

    serde_json::to_vec(&tx).expect("json")
}

#[test]
fn test_class_field() {
    let mut coin = asset_from_dev_class(AssetClass::Coin, 1, 10).expect("coin");
    coin.owner_pub = None;
    coin.owner_signature = None;
    let mut coin_wire = AssetWire::from_asset(&coin);
    coin_wire.amount = 0;

    let mut nft = asset_from_dev_class(AssetClass::Nft, 2, 0).expect("nft");
    nft.owner_pub = None;
    nft.owner_signature = None;
    let nft_wire = AssetWire::from_asset(&nft);

    let mut fee_coin = asset_from_dev_class(AssetClass::Coin, 3, 1_000_000).expect("fee coin");
    fee_coin.owner_pub = None;
    fee_coin.owner_signature = None;

    let verifier = TxVerifierImpl::new();

    let bad_fee = calc_fee(&[coin_wire.clone()]);
    let bad_coin_pkg = mk_pkg(vec![(TxOutRole::Recipient, coin_wire)], bad_fee);
    assert!(verifier.verify_balance(&bad_coin_pkg).is_err());

    let fee_seed = mk_wire(AssetClass::Coin, 1);
    let ok_fee = calc_fee(&[nft_wire.clone(), fee_seed]);
    let fee_coin_wire = mk_wire(AssetClass::Coin, ok_fee);
    let ok_nft_pkg = mk_pkg(
        vec![
            (TxOutRole::Recipient, nft_wire),
            (TxOutRole::Fee, fee_coin_wire),
        ],
        ok_fee,
    );
    let ok_nft = verifier.verify_balance(&ok_nft_pkg);
    assert!(ok_nft.is_ok(), "ok_nft={ok_nft:?}");
}

#[test]
fn test_id_stable() {
    let asset = asset_from_dev_class(AssetClass::Coin, 2, 77).expect("asset");
    let id_a = asset.asset_id();
    let wire = AssetWire::from_asset(&asset);
    let back = wire.to_asset().expect("to_asset");
    let id_b = back.asset_id();
    assert_eq!(id_a, id_b);
}

#[test]
fn test_bind_commit() {
    let prover = ProverImpl::new().expect("prover");
    let mut rng = MockRngProvider::with_u64_seed(24).rng();

    let blind_a = Hidden::hide(Z00ZScalar::random(&mut rng).unwrap());
    let blind_b = Hidden::hide(Z00ZScalar::random(&mut rng).unwrap());

    let proof_a = prover.create_proof(9, &blind_a).expect("proof");
    let comm_b = create_commitment(11, blind_b.reveal()).expect("commit");

    let bad = prover.verify_proof(&proof_a, comm_b.as_bytes());
    assert!(bad.is_err() || matches!(bad, Ok(false)));
}

struct TestState {
    root: [u8; 32],
    map: BTreeMap<[u8; 32], TerminalLeaf>,
}

impl SettlementState for TestState {
    fn root(&self) -> CheckRoot {
        self.root.into()
    }

    fn get_leaf(&self, id: &TerminalId) -> Result<Option<TerminalLeaf>, StateError> {
        Ok(self.map.get(id.as_bytes()).cloned())
    }

    fn del_leaf(&mut self, id: &TerminalId) -> Result<(), StateError> {
        self.map.remove(id.as_bytes());
        Ok(())
    }

    fn put_leaf(&mut self, leaf: TerminalLeaf) -> Result<(), StateError> {
        self.map.insert(leaf.asset_id, leaf);
        Ok(())
    }

    fn leaf_hash(&self, leaf: &TerminalLeaf) -> Result<[u8; 32], StateError> {
        Ok(leaf.asset_id)
    }
}

fn out_leaf(asset_id: [u8; 32], serial_id: u32) -> TerminalLeaf {
    TerminalLeaf::from(AssetLeaf {
        asset_id,
        serial_id,
        ..AssetLeaf::default()
    })
}

struct OkProof;
impl TxProofVerifier for OkProof {
    fn verify_tx(&self, _tx: &TxPkgSum) -> Result<(), TxProofError> {
        Ok(())
    }
}

struct HitSpent;
impl SpentIndex for HitSpent {
    fn is_spent(
        &self,
        _prev: CheckRoot,
        _curr: CheckRoot,
        _id: &TerminalId,
    ) -> Result<bool, SpentIndexError> {
        Ok(true)
    }
}

struct PathMixResolver;

impl InputResolver for PathMixResolver {
    fn resolve(
        &self,
        _prev_root: CheckRoot,
        terminal_id: TerminalId,
        _serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        Ok(wit_input_case([0x42; 32], 2, terminal_id.into_bytes()).input)
    }
}

#[test]
fn test_prev_root() {
    let id = [3u8; 32];
    let case = wit_input_case([0x41; 32], 0, id);
    let mut state = TestState {
        root: case.root.into_bytes(),
        map: BTreeMap::new(),
    };
    state.map.insert(id, case.input.leaf().clone());

    let tx = TxPkgSum {
        prev_root: [2u8; 32].into(),
        resolved_inputs: vec![case.input],
        outputs: vec![TerminalLeaf::default()],
        tx_proof: vec![1u8],
    };

    let err = apply_batch_checkpoint(1, &mut state, &[tx], &OkProof, &HitSpent).expect_err("err");
    assert_eq!(err, StateError::PrevRoot);
}

#[test]
fn test_spent_gate() {
    let id = [4u8; 32];
    let case = wit_input_case([0x41; 32], 0, id);
    let mut state = TestState {
        root: case.root.into_bytes(),
        map: BTreeMap::new(),
    };
    state.map.insert(id, case.input.leaf().clone());

    let tx = TxPkgSum {
        prev_root: case.root,
        resolved_inputs: vec![case.input],
        outputs: vec![TerminalLeaf::default()],
        tx_proof: vec![1u8],
    };

    let err = apply_batch_checkpoint(1, &mut state, &[tx], &OkProof, &HitSpent).expect_err("err");
    assert_eq!(err, StateError::SpentAfter);
}

#[test]
fn test_prep_path_mix() {
    let id = [5u8; 32];
    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &PathMixResolver,
        &[TxInputWire {
            asset_id_hex: hex::encode(id),
            serial_id: 1,
        }],
        &[out_leaf([0u8; 32], 0)],
        &[1u8],
    )
    .expect_err("path mix");

    assert_eq!(
        err,
        StateError::LeafMatch,
        "prepare_tx_sum must reject path-mixed resolution before checkpoint apply"
    );
}
