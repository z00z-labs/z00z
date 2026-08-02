use std::collections::BTreeMap;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use z00z_aggregators::{
    BatchId, BatchPlanner, IngressBoundary, OrderedBatch, WorkItem, WorkPayload,
};
use z00z_core::{assets::AssetClass, assets::AssetPkgWire, AssetWire};
use z00z_crypto::{protocol::ecdh::derive_dh_key, Hidden, Z00ZRistrettoPoint, Z00ZScalar};
use z00z_storage::{
    checkpoint::{CheckpointExecOut, CheckpointExecTx, CheckpointInRef},
    settlement::{
        CheckRoot, DefinitionId, ProofBlob, SerialId, SettlementExecHandoff, SettlementPath,
        StoreItem, StoreOp,
    },
};
use z00z_utils::{codec::Codec, codec::JsonCodec, rng::DeterministicRngProvider};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    stealth::{ecdh::sender_derive_dh_with_r, kdf::derive_s_out},
    tx::{
        asset_wire_to_leaf, build_spend_contract_with_rng, build_tx_package_digest,
        decode_output_pack, prepare_spend_public_inputs, resolve_input_pack,
        verify_commitment_balance_gate, verify_full_tx_package, verify_plaintext_balance_with_fee,
        verify_self_decrypt, verify_spend_witness_gate_membership, OutputBundle,
        SpendMembershipWitness, SpendProofWitness, TxAssemblerImpl, TxInputWire, TxOutRole,
        TxOutputWire, TxPackage, TxWire,
    },
};

use super::{
    config::Scenario2Cfg,
    runner::Scenario2Err,
    types::OwnedCoin,
    wallets::{owned_coin, tx_seed, WalletRing, WalletSpec},
};

const BATCH_DIGEST_LABEL: &[u8] = b"z00z.simulator.scenario-2.batch.v1";

pub(super) struct BuiltTx {
    package: TxPackage,
    input_path: SettlementPath,
    next_coin: OwnedCoin,
    output_items: Vec<StoreItem>,
    exec_tx: CheckpointExecTx,
}

struct TxEffects {
    input_path: SettlementPath,
    next_coin: OwnedCoin,
    output_items: Vec<StoreItem>,
    exec_tx: CheckpointExecTx,
}

pub(super) struct PreparedBlock {
    pub ordered: OrderedBatch,
    pub handoff: SettlementExecHandoff,
    pub next_coins: Vec<OwnedCoin>,
    pub spent_paths: Vec<SettlementPath>,
    pub output_paths: Vec<SettlementPath>,
}

pub(super) fn build_block(
    pool: &rayon::ThreadPool,
    config: &Scenario2Cfg,
    wallets: &WalletRing,
    height: u64,
    coins: &[OwnedCoin],
    proofs: &[ProofBlob],
    prev_root: CheckRoot,
) -> Result<Vec<BuiltTx>, Scenario2Err> {
    if coins.len() != proofs.len()
        || coins.len()
            != usize::try_from(config.load.transactions_per_block)
                .map_err(|_| Scenario2Err::Config("tx count conversion failed".to_string()))?
    {
        return Err(Scenario2Err::Invariant(
            "coin and membership cardinality drift".to_string(),
        ));
    }
    let (sender, recipient) = wallets.edge(height);
    let fee = wallets.fee();
    pool.install(|| {
        coins
            .par_iter()
            .zip(proofs.par_iter())
            .map(|(coin, proof)| {
                build_tx(
                    config,
                    height,
                    coin.clone(),
                    proof.clone(),
                    prev_root,
                    sender,
                    recipient,
                    fee,
                )
            })
            .collect()
    })
}

pub(super) fn order_block(height: u64, built: Vec<BuiltTx>) -> Result<PreparedBlock, Scenario2Err> {
    let batch_id = batch_id(height, &built);
    let ingress = IngressBoundary;
    let mut items = Vec::<WorkItem>::with_capacity(built.len());
    let mut effects = BTreeMap::<String, TxEffects>::new();
    for transaction in built {
        let digest = transaction.package.tx_digest_hex.clone();
        let item = ingress
            .normalize(WorkPayload::Tx(Box::new(transaction.package)))
            .map_err(|error| Scenario2Err::Aggregator(format!("{error:?}")))?;
        if effects
            .insert(
                digest,
                TxEffects {
                    input_path: transaction.input_path,
                    next_coin: transaction.next_coin,
                    output_items: transaction.output_items,
                    exec_tx: transaction.exec_tx,
                },
            )
            .is_some()
        {
            return Err(Scenario2Err::Invariant(
                "duplicate transaction digest in block".to_string(),
            ));
        }
        items.push(item);
    }

    let ordered = BatchPlanner::default()
        .make_batch(batch_id, &items)
        .map_err(|error| Scenario2Err::Aggregator(format!("{error:?}")))?;
    let mut ops = Vec::with_capacity(ordered.items.len().saturating_mul(3));
    let mut exec_txs = Vec::with_capacity(ordered.items.len());
    let mut next_coins = Vec::with_capacity(ordered.items.len());
    let mut spent_paths = Vec::with_capacity(ordered.items.len());
    let mut output_paths = Vec::with_capacity(ordered.items.len().saturating_mul(2));
    for item in &ordered.items {
        let effect = effects.remove(item.digest_hex()).ok_or_else(|| {
            Scenario2Err::Invariant("ordered batch lost transaction effects".to_string())
        })?;
        spent_paths.push(effect.input_path);
        ops.push(StoreOp::Delete(effect.input_path));
        for output in effect.output_items {
            output_paths.push(output.path());
            ops.push(StoreOp::Put(Box::new(output)));
        }
        exec_txs.push(effect.exec_tx);
        next_coins.push(effect.next_coin);
    }
    if !effects.is_empty() {
        return Err(Scenario2Err::Invariant(
            "unordered transaction effects remain".to_string(),
        ));
    }
    next_coins.sort_by_key(|coin| coin.lane);
    let handoff = ordered.exec_handoff(ops, exec_txs);
    Ok(PreparedBlock {
        ordered,
        handoff,
        next_coins,
        spent_paths,
        output_paths,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_tx(
    config: &Scenario2Cfg,
    height: u64,
    input: OwnedCoin,
    proof: ProofBlob,
    prev_root: CheckRoot,
    sender: &WalletSpec,
    recipient: &WalletSpec,
    fee_wallet: &WalletSpec,
) -> Result<BuiltTx, Scenario2Err> {
    let tx_input = TxInputWire {
        asset_id_hex: hex::encode(input.path.terminal_id().as_bytes()),
        serial_id: input.path.serial_id.get(),
    };
    let shape_seed = tx_seed(config.scenario.seed, height, input.lane, b"shape");
    let mut shape_rng = DeterministicRngProvider::from_seed(shape_seed).rng();
    let shape = make_outputs(
        &input.wire,
        sender.secret,
        recipient,
        fee_wallet,
        input.wire.amount.saturating_sub(1),
        1,
        &mut shape_rng,
    )?;
    let shape_wires = output_wires(&shape)?;
    let fee = calculate_fee(&shape_wires)?;
    let recipient_value = input
        .wire
        .amount
        .checked_sub(fee)
        .filter(|value| *value > 0)
        .ok_or_else(|| Scenario2Err::Wallet("lane value exhausted by fees".to_string()))?;

    let final_seed = tx_seed(config.scenario.seed, height, input.lane, b"final");
    let mut rng = DeterministicRngProvider::from_seed(final_seed).rng();
    let outputs = make_outputs(
        &input.wire,
        sender.secret,
        recipient,
        fee_wallet,
        recipient_value,
        fee,
        &mut rng,
    )?;
    let tx_outputs = output_wires(&outputs)?;
    if calculate_fee(&tx_outputs)? != fee {
        return Err(Scenario2Err::Invariant(
            "fee changed after final output construction".to_string(),
        ));
    }
    verify_plaintext_balance_with_fee(std::slice::from_ref(&input.wire), &outputs, fee)
        .map_err(Scenario2Err::Wallet)?;
    let input_commits = [input.wire.commitment.clone()];
    let output_commits = outputs
        .iter()
        .map(|output| {
            z00z_crypto::Commitment::from_bytes(&output.leaf.c_amount)
                .map(|commitment| commitment.as_commitment().clone())
                .map_err(|error| Scenario2Err::Wallet(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    verify_commitment_balance_gate(&input_commits, &output_commits, 0)
        .map_err(Scenario2Err::Wallet)?;
    for output in &outputs {
        verify_self_decrypt(output).map_err(Scenario2Err::Wallet)?;
    }

    let membership = membership_witness(input.path, proof)?;
    let proof_inputs = prepare_spend_public_inputs(
        config.runtime.chain_id,
        sender.secret,
        std::slice::from_ref(&input.wire),
        std::slice::from_ref(&tx_input),
    )
    .map_err(Scenario2Err::Wallet)?;
    let input_pack =
        resolve_input_pack(sender.secret, &input.wire).map_err(Scenario2Err::Wallet)?;
    let nonce = height
        .checked_mul(u64::from(config.load.transactions_per_block))
        .and_then(|value| value.checked_add(u64::from(input.lane)))
        .ok_or_else(|| Scenario2Err::Invariant("transaction nonce overflow".to_string()))?;
    let mut tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![tx_input],
        outputs: tx_outputs.clone(),
        fee,
        nonce,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let receiver_keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(sender.secret)
            .map_err(|error| Scenario2Err::Wallet(error.to_string()))?,
    )
    .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let witness = SpendProofWitness::new(
        ReceiverSecret::from_bytes(sender.secret)
            .map_err(|error| Scenario2Err::Wallet(error.to_string()))?,
        vec![input_pack.s_out],
        vec![membership.clone()],
    );
    let (spend_proof, spend_auth) = build_spend_contract_with_rng(
        &receiver_keys,
        config.runtime.chain_id,
        1,
        &config.runtime.chain_type,
        &config.runtime.chain_name,
        &tx,
        prev_root,
        proof_inputs,
        witness,
        &mut rng,
    )
    .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    tx.proof = spend_proof;
    tx.auth = spend_auth;

    verify_spend_witness_gate_membership(
        config.runtime.chain_id,
        sender.secret,
        std::slice::from_ref(&input.wire),
        &outputs,
        prev_root,
        vec![membership],
    )
    .map_err(Scenario2Err::Wallet)?;
    let digest = build_tx_package_digest(
        "TxPackage",
        "regular_tx",
        1,
        config.runtime.chain_id,
        &config.runtime.chain_type,
        &config.runtime.chain_name,
        &tx,
    )
    .map_err(Scenario2Err::Wallet)?;
    let package = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: config.runtime.chain_id,
        chain_type: config.runtime.chain_type.clone(),
        chain_name: config.runtime.chain_name.clone(),
        tx,
        tx_digest_hex: digest,
        status: "aggregator_ready".to_string(),
    };
    let package_bytes = JsonCodec
        .serialize(&package)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let report = verify_full_tx_package(&package_bytes)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    if !report.valid {
        return Err(Scenario2Err::Wallet(format!(
            "full package verification failed: {}",
            report.errors.join("; ")
        )));
    }

    let recipient_wire = tx_outputs
        .iter()
        .find(|output| output.role == TxOutRole::Recipient)
        .ok_or_else(|| Scenario2Err::Invariant("recipient output missing".to_string()))?
        .asset_wire
        .clone()
        .to_wire()
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let next_coin = owned_coin(input.lane, recipient_wire)?;
    let mut output_items = Vec::with_capacity(tx_outputs.len());
    let mut exec_outputs = Vec::with_capacity(tx_outputs.len());
    for output in &tx_outputs {
        let wire = output
            .asset_wire
            .clone()
            .to_wire()
            .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
        let leaf = asset_wire_to_leaf(&wire).map_err(Scenario2Err::Wallet)?;
        let definition = DefinitionId::new(wire.definition.id);
        let path = z00z_storage::settlement::SettlementPath::new(
            definition,
            SerialId::new(wire.serial_id),
            leaf.terminal_id(),
        );
        output_items.push(
            StoreItem::new(path, leaf.clone())
                .map_err(|error| Scenario2Err::Storage(error.to_string()))?,
        );
        exec_outputs.push(
            CheckpointExecOut::new(definition, leaf)
                .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?,
        );
    }
    let tx_proof = JsonCodec
        .serialize(&package.tx.proof)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let exec_tx = CheckpointExecTx::new(
        vec![CheckpointInRef::new(
            input.path.terminal_id(),
            input.path.serial_id,
        )],
        exec_outputs,
        tx_proof,
    )
    .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    Ok(BuiltTx {
        package,
        input_path: input.path,
        next_coin,
        output_items,
        exec_tx,
    })
}

fn make_outputs<R: rand::CryptoRng + rand::RngCore>(
    input: &AssetWire,
    sender_secret: [u8; 32],
    recipient: &WalletSpec,
    fee_wallet: &WalletSpec,
    recipient_value: u64,
    fee_value: u64,
    rng: &mut R,
) -> Result<Vec<OutputBundle>, Scenario2Err> {
    if recipient_value == 0 || fee_value == 0 {
        return Err(Scenario2Err::Wallet(
            "recipient and fee outputs must be positive".to_string(),
        ));
    }
    let recipient_output = z00z_wallets::build_output_bundle_with_rng(
        recipient.name.clone(),
        TxOutRole::Recipient,
        AssetClass::Coin,
        &recipient.card,
        recipient_value,
        input.serial_id,
        rng,
    )
    .map_err(Scenario2Err::Wallet)?;
    let input_pack = resolve_input_pack(sender_secret, input).map_err(Scenario2Err::Wallet)?;
    let input_blind = Z00ZScalar::try_from_bytes(input_pack.blinding)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let recipient_pack = decode_output_pack(&recipient_output).map_err(Scenario2Err::Wallet)?;
    let recipient_blind = Z00ZScalar::try_from_bytes(recipient_pack.blinding)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let fee_blind = &input_blind - &recipient_blind;
    let fee_output = balanced_output(
        fee_wallet.name.clone(),
        &fee_wallet.card,
        fee_value,
        input.serial_id,
        fee_blind,
        rng,
    )?;
    Ok(vec![recipient_output, fee_output])
}

fn balanced_output<R: rand::CryptoRng + rand::RngCore>(
    receiver: String,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
    blinding: Z00ZScalar,
    rng: &mut R,
) -> Result<OutputBundle, Scenario2Err> {
    let view_pk = Z00ZRistrettoPoint::try_from_bytes(card.view_pk)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let r = Z00ZScalar::random(rng).map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let sender = sender_derive_dh_with_r(&view_pk, &r)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let r_pub = sender.r_pub.to_bytes();
    let k_dh = derive_dh_key(&sender.dh);
    let s_out = derive_s_out(&k_dh, &r_pub, serial_id);
    let hidden = Hidden::hide(blinding);
    let leaf = z00z_wallets::build_stealth_leaf_with_rng(
        &k_dh,
        &r_pub,
        &card.owner_handle,
        value,
        serial_id,
        s_out,
        &hidden,
        rng,
    )
    .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    Ok(OutputBundle {
        receiver,
        role: TxOutRole::Fee,
        class: AssetClass::Coin,
        value,
        leaf,
        k_dh,
        s_out,
    })
}

pub(super) fn output_wire(
    output: &OutputBundle,
    index: usize,
) -> Result<TxOutputWire, Scenario2Err> {
    let mut asset = z00z_core::genesis::asset_std::asset_from_dev_class(
        output.class,
        output.leaf.serial_id,
        output.value,
    )
    .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    asset.nonce = z00z_wallets::tx::derive_tx_output_nonce(&output.leaf, index);
    let wire = z00z_wallets::bind_stealth_output_wire(AssetWire::from_asset(&asset), &output.leaf)
        .map_err(Scenario2Err::Wallet)?;
    Ok(TxOutputWire {
        role: output.role,
        asset_wire: AssetPkgWire::from_wire(&wire),
    })
}

fn output_wires(outputs: &[OutputBundle]) -> Result<Vec<TxOutputWire>, Scenario2Err> {
    outputs
        .iter()
        .enumerate()
        .map(|(index, output)| output_wire(output, index))
        .collect()
}

fn calculate_fee(outputs: &[TxOutputWire]) -> Result<u64, Scenario2Err> {
    let wires = outputs
        .iter()
        .map(|output| output.asset_wire.clone().to_wire())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    TxAssemblerImpl::new()
        .calculate_fee_for_wires(1, &wires)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))
}

fn membership_witness(
    path: SettlementPath,
    proof: ProofBlob,
) -> Result<SpendMembershipWitness, Scenario2Err> {
    if proof.item().path() != path {
        return Err(Scenario2Err::Invariant(
            "membership proof path drift".to_string(),
        ));
    }
    let leaf = proof
        .item()
        .terminal_leaf()
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?
        .clone();
    let proof_item = proof.item().clone();
    let bytes = proof
        .encode()
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    SpendMembershipWitness::new(path, leaf, bytes, proof_item)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))
}

fn batch_id(height: u64, built: &[BuiltTx]) -> BatchId {
    let mut hasher = Sha256::new();
    hasher.update(BATCH_DIGEST_LABEL);
    hasher.update(height.to_le_bytes());
    hasher.update((built.len() as u64).to_le_bytes());
    for transaction in built {
        hasher.update(transaction.package.tx_digest_hex.as_bytes());
    }
    BatchId::from_bytes(hasher.finalize().into())
}
