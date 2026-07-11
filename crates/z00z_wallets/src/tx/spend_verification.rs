// Threat T-5 anchor: verify_tx_public_spend_contract is statement-envelope scope only; it is not a substitute for verify_full_tx_package.
#![allow(missing_docs)]

use std::collections::{BTreeSet, HashSet};

use subtle::ConstantTimeEq;
use thiserror::Error;
use z00z_core::{assets::registry::AssetId, AssetClass};
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::{
    compute_leaf_ad, domains::AssetIdDomain, frame_bytes, Z00ZCommitment, Z00ZRistrettoPoint,
    Z00ZSchnorrSignature,
};
use z00z_storage::settlement::{CheckRoot, TerminalId, TerminalLeaf};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::rng::SystemRngProvider;

use crate::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{decode_card_compact, encode_card_compact, ReceiverCard},
    stealth::{
        ecdh::{compute_dh_receiver, decode_r_pub},
        kdf::{compute_owner_tag, derive_k_dh},
    },
};

use super::{
    prover::{
        sign_spend_authorization_with_rng, verify_spend_authorization, Prover, ProverImpl,
        SPEND_AUTH_CTX,
    },
    spend_proof_backend::{
        default_spend_proof_backend, SpendProofArtifact, SpendProofBackend, SpendProofBackendError,
        SpendProofStmt, SpendProofWitness,
    },
    spend_rules::{derive_spend_nullifier, verify_spend_rules, SpendIn, SpendStmt},
    tx_digest::build_tx_package_digest,
    tx_wire::{
        canonicalize_tx_inputs, decode_tx_input_asset_id, SpendAuthWire, SpendInputProofWire,
        SpendProofWire, TxAuthWire, TxOutputWire, TxProofWire, TxWire, REGULAR_TX_PACKAGE_TYPE,
        SPEND_AUTH_WIRE_VER, SPEND_PROOF_SUITE, SPEND_PROOF_WIRE_VER, TX_PACKAGE_KIND,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendInputRef {
    pub asset_id: AssetId,
    pub serial_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendInputLeaf {
    /// Canonical consumed pre-state key for this input.
    pub asset_id: AssetId,
    /// Serial id paired with the consumed pre-state key.
    pub serial_id: u32,
    /// Canonical `leaf_ad_id` bound to the stealth leaf payload.
    pub leaf_ad_id: [u8; 32],
    pub r_pub: [u8; 32],
    pub owner_tag: [u8; 32],
    pub c_amt: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendPlan {
    pub chain_id: u32,
    pub prev_root: CheckRoot,
    pub inputs: Vec<SpendInputRef>,
    pub leaf_sums: Vec<SpendInputLeaf>,
    pub outputs: Vec<TerminalLeaf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendWitness {
    pub recv_sec: [u8; 32],
    pub s_in_vec: Vec<[u8; 32]>,
}

type OutputLeafAdParts = ([u8; 32], [u8; 32], [u8; 32], [u8; 32]);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpendProofErr {
    #[error("bind root failed")]
    BindRoot,
    #[error("input proof failed at index {idx}")]
    Input { idx: usize },
    #[error("balance proof failed")]
    Balance,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpendBuildErr {
    #[error("inputs must be non-empty")]
    EmptyInputs,
    #[error("outputs must be non-empty")]
    EmptyOutputs,
    #[error("input vector mismatch")]
    InputMismatch,
    #[error("duplicate input reference")]
    DupInput,
    #[error("input id mismatch at index {idx}")]
    InputId { idx: usize },
    #[error("invalid spend witness at index {idx}")]
    BadWitness { idx: usize },
    #[error("invalid input leaf at index {idx}")]
    BadLeaf { idx: usize },
    #[error("bad receiver secret")]
    BadSecret,
    #[error("invalid input commitment at index {idx}")]
    BadComIn { idx: usize },
    #[error("invalid output commitment at index {idx}")]
    BadComOut { idx: usize },
    #[error("spend rules failed")]
    BadRules,
    #[error(transparent)]
    Cs(#[from] SpendProofErr),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpendPublicErr {
    #[error("missing spend proof")]
    MissingProof,
    #[error("missing spend authorization")]
    MissingAuth,
    #[error("invalid spend proof version")]
    BadProofVersion,
    #[error("invalid spend proof suite")]
    BadProofSuite,
    #[error("invalid spend auth version")]
    BadAuthVersion,
    #[error("invalid previous root")]
    BadPrevRoot,
    #[error("input count mismatch")]
    InputCountMismatch,
    #[error("input {idx} mismatches tx input ref")]
    InputRefMismatch { idx: usize },
    #[error("invalid hex field {label}")]
    InvalidHex { label: &'static str },
    #[error("invalid receiver card: {0}")]
    BadCard(String),
    #[error("invalid spend signature: {0}")]
    BadSignature(String),
    #[error("input {idx} leaf_ad hash mismatch")]
    InputLeafAdHashMismatch { idx: usize },
    #[error("duplicate nullifier in one spend contract")]
    DuplicateNullifier,
    #[error("output {idx} missing stealth field {field}")]
    MissingOutputField { idx: usize, field: &'static str },
    #[error("output {idx} leaf_ad relation invalid")]
    BadOutputLeafAd { idx: usize },
    #[error("range proof missing for output {idx}")]
    MissingRangeProof { idx: usize },
    #[error("range proof invalid for output {idx}: {reason}")]
    BadRangeProof { idx: usize, reason: String },
    #[error("balance equation mismatch")]
    BadBalance,
    #[error("duplicate input state key")]
    DuplicateInputRef,
    #[error("duplicate leaf_ad id")]
    DuplicateLeafAdId,
    #[error("overlapping input/output leaf_ad ids")]
    InputOutputLeafOverlap,
    #[error("statement encode failed: {0}")]
    StatementEncode(String),
    #[error("missing spend statement payload")]
    MissingStatement,
    #[error("missing spend proof payload")]
    MissingProofBlob,
    #[error("invalid spend proof blob")]
    BadProofBlob,
    #[error("spend proof blob does not bind the carried statement")]
    ProofBlobStatementMismatch,
    #[error("spend proof theorem verification failed")]
    TheoremProofFailed,
    #[error("carried spend statement mismatches recomputed statement")]
    StatementMismatch,
    #[error("authorization verification failed")]
    AuthorizationFailed,
    #[error("receiver-bound spend input mismatch at index {idx}")]
    ReceiverInputMismatch { idx: usize },
}

fn map_backend_err(err: SpendProofBackendError) -> SpendPublicErr {
    match err {
        SpendProofBackendError::EmptyStatement => {
            SpendPublicErr::StatementEncode("empty spend statement".to_string())
        }
        SpendProofBackendError::StatementShapeMismatch => {
            SpendPublicErr::StatementEncode("typed spend statement shape mismatch".to_string())
        }
        SpendProofBackendError::InvalidProofHex | SpendProofBackendError::InvalidProofPayload => {
            SpendPublicErr::BadProofBlob
        }
        SpendProofBackendError::UnsupportedSuite => SpendPublicErr::BadProofSuite,
        SpendProofBackendError::MissingTheoremProof => SpendPublicErr::BadProofBlob,
        SpendProofBackendError::StatementMismatch | SpendProofBackendError::PublicHashMismatch => {
            SpendPublicErr::ProofBlobStatementMismatch
        }
        SpendProofBackendError::TheoremRelationMismatch => SpendPublicErr::TheoremProofFailed,
        SpendProofBackendError::EmptyWitness => {
            SpendPublicErr::StatementEncode("empty spend proof witness".to_string())
        }
        SpendProofBackendError::WitnessInputMismatch => {
            SpendPublicErr::StatementEncode("spend proof witness input count mismatch".to_string())
        }
        SpendProofBackendError::MissingMembershipWitness => {
            SpendPublicErr::StatementEncode("missing spend membership witness".to_string())
        }
        SpendProofBackendError::MembershipWitnessMismatch => SpendPublicErr::StatementEncode(
            "spend membership witness relation mismatch".to_string(),
        ),
        SpendProofBackendError::WitnessRelationMismatch => {
            SpendPublicErr::StatementEncode("spend proof witness relation mismatch".to_string())
        }
        SpendProofBackendError::RangeRelationMismatch => SpendPublicErr::TheoremProofFailed,
    }
}

fn decode_canonical_hex(value: &str, label: &'static str) -> Result<Vec<u8>, SpendPublicErr> {
    let bytes = hex::decode(value).map_err(|_| SpendPublicErr::InvalidHex { label })?;
    if hex::encode(&bytes) != value {
        return Err(SpendPublicErr::InvalidHex { label });
    }
    Ok(bytes)
}

fn decode_hex32(value: &str, label: &'static str) -> Result<[u8; 32], SpendPublicErr> {
    let bytes = decode_canonical_hex(value, label)?;
    bytes
        .try_into()
        .map_err(|_| SpendPublicErr::InvalidHex { label })
}

fn canonicalize_spend_input_proof(
    proof: &SpendInputProofWire,
) -> Result<SpendInputProofWire, SpendPublicErr> {
    Ok(SpendInputProofWire {
        input_asset_id_hex: hex::encode(decode_hex32(
            &proof.input_asset_id_hex,
            "proof.inputs[].input_asset_id_hex",
        )?),
        serial_id: proof.serial_id,
        nullifier_hex: hex::encode(decode_hex32(
            &proof.nullifier_hex,
            "proof.inputs[].nullifier_hex",
        )?),
        r_pub_hex: hex::encode(decode_hex32(&proof.r_pub_hex, "proof.inputs[].r_pub_hex")?),
        owner_tag_hex: hex::encode(decode_hex32(
            &proof.owner_tag_hex,
            "proof.inputs[].owner_tag_hex",
        )?),
        commitment_hex: hex::encode(decode_hex32(
            &proof.commitment_hex,
            "proof.inputs[].commitment_hex",
        )?),
        leaf_ad_id_hex: hex::encode(decode_hex32(
            &proof.leaf_ad_id_hex,
            "proof.inputs[].leaf_ad_id_hex",
        )?),
        leaf_ad_hash_hex: hex::encode(decode_hex32(
            &proof.leaf_ad_hash_hex,
            "proof.inputs[].leaf_ad_hash_hex",
        )?),
    })
}

fn canonicalize_spend_proof(proof: &SpendProofWire) -> Result<SpendProofWire, SpendPublicErr> {
    Ok(SpendProofWire {
        ver: proof.ver,
        proof_suite: proof.proof_suite.clone(),
        prev_root_hex: hex::encode(decode_hex32(&proof.prev_root_hex, "proof.prev_root_hex")?),
        statement_hex: hex::encode(decode_canonical_hex(
            &proof.statement_hex,
            "proof.statement_hex",
        )?),
        proof_hex: hex::encode(decode_canonical_hex(&proof.proof_hex, "proof.proof_hex")?),
        inputs: proof
            .inputs
            .iter()
            .map(canonicalize_spend_input_proof)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn canonicalize_spend_auth(auth: &SpendAuthWire) -> Result<SpendAuthWire, SpendPublicErr> {
    Ok(SpendAuthWire {
        ver: auth.ver,
        receiver_card_compact: auth.receiver_card_compact.clone(),
        spend_sig_hex: hex::encode(decode_canonical_hex(
            &auth.spend_sig_hex,
            "auth.spend_sig_hex",
        )?),
    })
}

pub(crate) fn canonicalize_regular_spend_tx(tx: &TxWire) -> Result<TxWire, SpendPublicErr> {
    let mut canonical = tx.clone();
    if canonical.tx_type != super::tx_wire::REGULAR_TX_TYPE {
        return Ok(canonical);
    }
    if let Some(spend) = canonical.proof.spend.as_ref() {
        canonical.proof.spend = Some(canonicalize_spend_proof(spend)?);
    }
    if let Some(auth) = canonical.auth.spend.as_ref() {
        canonical.auth.spend = Some(canonicalize_spend_auth(auth)?);
    }
    Ok(canonical)
}

fn sig_to_hex(sig: &Z00ZSchnorrSignature) -> String {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(sig.get_public_nonce().as_bytes());
    bytes[32..].copy_from_slice(sig.get_signature().as_bytes());
    hex::encode(bytes)
}

fn decode_sig_hex(value: &str) -> Result<Z00ZSchnorrSignature, SpendPublicErr> {
    let bytes = decode_canonical_hex(value, "auth.spend_sig_hex").map_err(|_| {
        SpendPublicErr::BadSignature("spend_sig_hex must be 64-byte lowercase hex".to_string())
    })?;
    let bytes: [u8; 64] = bytes
        .try_into()
        .map_err(|_| SpendPublicErr::BadSignature("spend_sig_hex must be 64 bytes".to_string()))?;
    let nonce =
        Z00ZRistrettoPoint::try_from_bytes(bytes[..32].try_into().map_err(|_| {
            SpendPublicErr::BadSignature("invalid signature nonce bytes".to_string())
        })?)
        .map_err(|err| {
            SpendPublicErr::BadSignature(format!("invalid signature nonce bytes: {err}"))
        })?;
    let scalar =
        z00z_crypto::Z00ZScalar::try_from_bytes(bytes[32..].try_into().map_err(|_| {
            SpendPublicErr::BadSignature("invalid signature scalar bytes".to_string())
        })?)
        .map_err(|err| {
            SpendPublicErr::BadSignature(format!("invalid signature scalar bytes: {err}"))
        })?;
    Ok(Z00ZSchnorrSignature::new(
        nonce.reveal().clone(),
        scalar.reveal().clone(),
    ))
}

fn compute_input_leaf_ad_hash(proof: &SpendInputProofWire) -> Result<[u8; 32], SpendPublicErr> {
    let leaf_ad_id = decode_hex32(&proof.leaf_ad_id_hex, "proof.inputs[].leaf_ad_id_hex")?;
    let r_pub = decode_hex32(&proof.r_pub_hex, "proof.inputs[].r_pub_hex")?;
    let owner_tag = decode_hex32(&proof.owner_tag_hex, "proof.inputs[].owner_tag_hex")?;
    let c_amt = decode_hex32(&proof.commitment_hex, "proof.inputs[].commitment_hex")?;
    Ok(compute_leaf_ad(
        &leaf_ad_id,
        proof.serial_id,
        &r_pub,
        &owner_tag,
        &c_amt,
    ))
}

fn validate_receiver_bound_input(
    receiver_keys: &ReceiverKeys,
    idx: usize,
    proof: &SpendInputProofWire,
) -> Result<SpendInputProofWire, SpendPublicErr> {
    let proof = canonicalize_spend_input_proof(proof)?;
    let r_pub_bytes = decode_hex32(&proof.r_pub_hex, "proof.inputs[].r_pub_hex")?;
    let r_pub =
        decode_r_pub(&r_pub_bytes).map_err(|_| SpendPublicErr::ReceiverInputMismatch { idx })?;
    let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub)
        .map_err(|_| SpendPublicErr::ReceiverInputMismatch { idx })?;
    let k_in = derive_k_dh(&dh);
    let owner_tag = decode_hex32(&proof.owner_tag_hex, "proof.inputs[].owner_tag_hex")?;
    let expected_owner_tag = compute_owner_tag(&receiver_keys.owner_handle, &k_in);
    if owner_tag != expected_owner_tag {
        return Err(SpendPublicErr::ReceiverInputMismatch { idx });
    }

    let leaf_ad_hash = compute_input_leaf_ad_hash(&proof)?;
    if hex::encode(leaf_ad_hash) != proof.leaf_ad_hash_hex {
        return Err(SpendPublicErr::InputLeafAdHashMismatch { idx });
    }

    Ok(proof)
}

fn output_leaf_ad_parts(
    idx: usize,
    out: &TxOutputWire,
) -> Result<OutputLeafAdParts, SpendPublicErr> {
    let wire = &out.asset_wire;
    let leaf_ad_id = wire.leaf_ad_id.ok_or(SpendPublicErr::MissingOutputField {
        idx,
        field: "leaf_ad_id",
    })?;
    let r_pub = wire.r_pub.ok_or(SpendPublicErr::MissingOutputField {
        idx,
        field: "r_pub",
    })?;
    let owner_tag = wire.owner_tag.ok_or(SpendPublicErr::MissingOutputField {
        idx,
        field: "owner_tag",
    })?;
    let c_amt =
        wire.commitment
            .as_bytes()
            .try_into()
            .map_err(|_| SpendPublicErr::MissingOutputField {
                idx,
                field: "commitment",
            })?;
    Ok((leaf_ad_id, r_pub, owner_tag, c_amt))
}

fn compute_output_leaf_ad_hash(idx: usize, out: &TxOutputWire) -> Result<[u8; 32], SpendPublicErr> {
    let (leaf_ad_id, r_pub, owner_tag, c_amt) = output_leaf_ad_parts(idx, out)?;
    Ok(compute_leaf_ad(
        &leaf_ad_id,
        out.asset_wire.serial_id,
        &r_pub,
        &owner_tag,
        &c_amt,
    ))
}

fn build_spend_statement_tx(tx: &TxWire, proof: &SpendProofWire) -> TxWire {
    let mut statement_tx = tx.clone();
    statement_tx.proof = TxProofWire {
        spend: Some(SpendProofWire {
            ver: proof.ver,
            proof_suite: proof.proof_suite.clone(),
            prev_root_hex: proof.prev_root_hex.clone(),
            statement_hex: String::new(),
            proof_hex: String::new(),
            inputs: proof.inputs.clone(),
        }),
    };
    statement_tx.auth = TxAuthWire::default();
    statement_tx
}

fn build_spend_proof_stmt(
    chain_id: u32,
    tx_version: u8,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
    card: &ReceiverCard,
    proof: &SpendProofWire,
) -> Result<SpendProofStmt, SpendPublicErr> {
    let statement = encode_spend_statement(
        chain_id, tx_version, chain_type, chain_name, tx, card, proof,
    )?;
    let package_digest_hex = build_tx_package_digest(
        TX_PACKAGE_KIND,
        REGULAR_TX_PACKAGE_TYPE,
        tx_version,
        chain_id,
        chain_type,
        chain_name,
        &build_spend_statement_tx(tx, proof),
    )
    .map_err(SpendPublicErr::StatementEncode)?;
    let input_refs = tx
        .inputs
        .iter()
        .map(|input| {
            Ok(SpendInputRef {
                asset_id: decode_hex32(&input.asset_id_hex, "tx.inputs[].asset_id_hex")?,
                serial_id: input.serial_id,
            })
        })
        .collect::<Result<Vec<_>, SpendPublicErr>>()?;
    let input_leaves = proof
        .inputs
        .iter()
        .map(|input| {
            Ok(SpendInputLeaf {
                asset_id: decode_hex32(
                    &input.input_asset_id_hex,
                    "proof.inputs[].input_asset_id_hex",
                )?,
                serial_id: input.serial_id,
                leaf_ad_id: decode_hex32(&input.leaf_ad_id_hex, "proof.inputs[].leaf_ad_id_hex")?,
                r_pub: decode_hex32(&input.r_pub_hex, "proof.inputs[].r_pub_hex")?,
                owner_tag: decode_hex32(&input.owner_tag_hex, "proof.inputs[].owner_tag_hex")?,
                c_amt: decode_hex32(&input.commitment_hex, "proof.inputs[].commitment_hex")?,
            })
        })
        .collect::<Result<Vec<_>, SpendPublicErr>>()?;
    let output_leaves = tx
        .outputs
        .iter()
        .enumerate()
        .map(|(idx, output)| {
            let (leaf_ad_id, _, _, _) = output_leaf_ad_parts(idx, output)?;
            let wire = output
                .asset_wire
                .clone()
                .to_wire()
                .map_err(|err| SpendPublicErr::StatementEncode(err.to_string()))?;
            let mut leaf =
                crate::tx::asset_wire_to_leaf(&wire).map_err(SpendPublicErr::StatementEncode)?;
            leaf.set_terminal_id(TerminalId::new(leaf_ad_id));
            Ok(leaf)
        })
        .collect::<Result<Vec<_>, SpendPublicErr>>()?;
    let nullifiers = proof
        .inputs
        .iter()
        .map(|input| decode_hex32(&input.nullifier_hex, "proof.inputs[].nullifier_hex"))
        .collect::<Result<Vec<_>, SpendPublicErr>>()?;

    SpendProofStmt::from_parts(
        statement,
        decode_hex32(&package_digest_hex, "statement.package_digest_hex")?,
        CheckRoot::new(decode_hex32(&proof.prev_root_hex, "proof.prev_root_hex")?),
        chain_id,
        tx_version,
        chain_type.to_string(),
        chain_name.to_string(),
        input_refs,
        input_leaves,
        output_leaves,
        nullifiers,
    )
    .map_err(map_backend_err)
}

fn validate_proof_witness(
    receiver_keys: &ReceiverKeys,
    proof_inputs: &[SpendInputProofWire],
    proof_witness: &SpendProofWitness,
) -> Result<(), SpendPublicErr> {
    if proof_witness
        .receiver_secret
        .ct_eq(receiver_keys.reveal_receiver_secret())
        .unwrap_u8()
        == 0
    {
        return Err(SpendPublicErr::StatementEncode(
            "spend proof witness receiver secret mismatch".to_string(),
        ));
    }
    if proof_witness.input_s_in.len() != proof_inputs.len() {
        return Err(SpendPublicErr::StatementEncode(
            "spend proof witness input count mismatch".to_string(),
        ));
    }
    if proof_witness.membership.len() != proof_inputs.len() {
        return Err(SpendPublicErr::StatementEncode(
            "spend membership witness input count mismatch".to_string(),
        ));
    }
    if proof_witness.input_s_in.contains(&[0u8; 32]) {
        return Err(SpendPublicErr::StatementEncode(
            "spend proof witness contains zero input secret".to_string(),
        ));
    }
    Ok(())
}

fn encode_spend_statement(
    chain_id: u32,
    tx_version: u8,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
    card: &ReceiverCard,
    proof: &SpendProofWire,
) -> Result<Vec<u8>, SpendPublicErr> {
    let tx = canonicalize_tx_inputs(tx).map_err(|_| SpendPublicErr::InvalidHex {
        label: "tx.inputs[].asset_id_hex",
    })?;
    let proof = canonicalize_spend_proof(proof)?;
    if proof.inputs.len() != tx.inputs.len() {
        return Err(SpendPublicErr::InputCountMismatch);
    }

    let codec = JsonCodec;
    let mut statement = Vec::new();
    let statement_tx = build_spend_statement_tx(&tx, &proof);
    let package_digest = build_tx_package_digest(
        TX_PACKAGE_KIND,
        REGULAR_TX_PACKAGE_TYPE,
        tx_version,
        chain_id,
        chain_type,
        chain_name,
        &statement_tx,
    )
    .map_err(SpendPublicErr::StatementEncode)?;
    statement.extend_from_slice(&frame_bytes(b"z00z.spend.public.v1"));
    statement.extend_from_slice(&frame_bytes(SPEND_AUTH_CTX));
    statement.extend_from_slice(&frame_bytes(TX_PACKAGE_KIND.as_bytes()));
    statement.extend_from_slice(&frame_bytes(REGULAR_TX_PACKAGE_TYPE.as_bytes()));
    statement.extend_from_slice(&frame_bytes(&chain_id.to_le_bytes()));
    statement.extend_from_slice(&frame_bytes(&[tx_version]));
    statement.extend_from_slice(&frame_bytes(chain_type.as_bytes()));
    statement.extend_from_slice(&frame_bytes(chain_name.as_bytes()));
    statement.extend_from_slice(&frame_bytes(package_digest.as_bytes()));
    statement.extend_from_slice(&frame_bytes(tx.tx_type.as_bytes()));
    statement.extend_from_slice(&frame_bytes(&tx.fee.to_le_bytes()));
    statement.extend_from_slice(&frame_bytes(&tx.nonce.to_le_bytes()));
    statement.extend_from_slice(&frame_bytes(card.owner_handle.as_slice()));
    statement.extend_from_slice(&frame_bytes(card.identity_pk.as_slice()));
    statement.extend_from_slice(&frame_bytes(&[proof.ver]));
    statement.extend_from_slice(&frame_bytes(proof.proof_suite.as_bytes()));
    statement.extend_from_slice(&frame_bytes(proof.prev_root_hex.as_bytes()));

    for (idx, (input, proof_input)) in tx.inputs.iter().zip(proof.inputs.iter()).enumerate() {
        if input.asset_id_hex != proof_input.input_asset_id_hex {
            return Err(SpendPublicErr::InputRefMismatch { idx });
        }
        if input.serial_id != proof_input.serial_id {
            return Err(SpendPublicErr::InputRefMismatch { idx });
        }
        let input_bytes = codec
            .serialize(input)
            .map_err(|err| SpendPublicErr::StatementEncode(err.to_string()))?;
        let proof_bytes = codec
            .serialize(proof_input)
            .map_err(|err| SpendPublicErr::StatementEncode(err.to_string()))?;
        statement.extend_from_slice(&frame_bytes(&input_bytes));
        statement.extend_from_slice(&frame_bytes(&proof_bytes));
    }

    for (idx, output) in tx.outputs.iter().enumerate() {
        let output_bytes = codec
            .serialize(output)
            .map_err(|err| SpendPublicErr::StatementEncode(err.to_string()))?;
        let output_leaf_hash = compute_output_leaf_ad_hash(idx, output)?;
        statement.extend_from_slice(&frame_bytes(&output_bytes));
        statement.extend_from_slice(&frame_bytes(output_leaf_hash.as_slice()));
    }

    Ok(statement)
}

pub fn build_spend_input_proof(
    chain_id: u32,
    input_ref: &SpendInputRef,
    leaf: &SpendInputLeaf,
    s_in: &[u8; 32],
) -> Result<SpendInputProofWire, SpendPublicErr> {
    if input_ref.serial_id != leaf.serial_id || input_ref.asset_id != leaf.asset_id {
        return Err(SpendPublicErr::InputRefMismatch { idx: 0 });
    }
    let theorem_leaf_ad_id = z00z_crypto::hash_zk::hash_zk::<AssetIdDomain>("", &[s_in]);
    let leaf_ad_hash = compute_leaf_ad(
        &theorem_leaf_ad_id,
        input_ref.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amt,
    );
    Ok(SpendInputProofWire {
        input_asset_id_hex: hex::encode(input_ref.asset_id),
        serial_id: input_ref.serial_id,
        nullifier_hex: hex::encode(derive_spend_nullifier(chain_id, s_in)),
        r_pub_hex: hex::encode(leaf.r_pub),
        owner_tag_hex: hex::encode(leaf.owner_tag),
        commitment_hex: hex::encode(leaf.c_amt),
        leaf_ad_id_hex: hex::encode(theorem_leaf_ad_id),
        leaf_ad_hash_hex: hex::encode(leaf_ad_hash),
    })
}

pub fn build_spend_contract_with_rng<R>(
    receiver_keys: &ReceiverKeys,
    chain_id: u32,
    tx_version: u8,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
    prev_root: CheckRoot,
    proof_inputs: Vec<SpendInputProofWire>,
    proof_witness: SpendProofWitness,
    rng: &mut R,
) -> Result<(TxProofWire, TxAuthWire), SpendPublicErr>
where
    R: rand::CryptoRng + rand::RngCore,
{
    let card = receiver_keys
        .export_receiver_card_with_rng(rng)
        .map_err(|err| SpendPublicErr::BadCard(err.to_string()))?;
    let proof_inputs = proof_inputs
        .iter()
        .enumerate()
        .map(|(idx, proof)| validate_receiver_bound_input(receiver_keys, idx, proof))
        .collect::<Result<Vec<_>, _>>()?;
    validate_proof_witness(receiver_keys, &proof_inputs, &proof_witness)?;
    let proof = SpendProofWire {
        ver: SPEND_PROOF_WIRE_VER,
        proof_suite: SPEND_PROOF_SUITE.to_string(),
        prev_root_hex: hex::encode(prev_root.into_bytes()),
        statement_hex: String::new(),
        proof_hex: String::new(),
        inputs: proof_inputs,
    };
    let stmt = build_spend_proof_stmt(
        chain_id, tx_version, chain_type, chain_name, tx, &card, &proof,
    )?;
    let backend = default_spend_proof_backend();
    let artifact = backend
        .prove(&stmt, &proof_witness)
        .map_err(map_backend_err)?;
    let proof = SpendProofWire {
        statement_hex: hex::encode(&stmt.statement),
        proof_hex: artifact.proof_hex,
        ..proof
    };
    let sig =
        sign_spend_authorization_with_rng(receiver_keys.reveal_identity_sk(), &stmt.statement, rng)
            .map_err(|err| SpendPublicErr::BadSignature(err.to_string()))?;
    let auth = SpendAuthWire {
        ver: SPEND_AUTH_WIRE_VER,
        receiver_card_compact: encode_card_compact(&card),
        spend_sig_hex: sig_to_hex(&sig),
    };
    Ok((
        TxProofWire { spend: Some(proof) },
        TxAuthWire { spend: Some(auth) },
    ))
}

pub fn build_public_spend_contract(
    receiver_keys: &ReceiverKeys,
    chain_id: u32,
    tx_version: u8,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
    prev_root: CheckRoot,
    proof_inputs: Vec<SpendInputProofWire>,
    proof_witness: SpendProofWitness,
) -> Result<(TxProofWire, TxAuthWire), SpendPublicErr> {
    let mut rng = SystemRngProvider.rng();
    build_spend_contract_with_rng(
        receiver_keys,
        chain_id,
        tx_version,
        chain_type,
        chain_name,
        tx,
        prev_root,
        proof_inputs,
        proof_witness,
        &mut rng,
    )
}

/// Verify the current public spend statement only.
///
/// Real theft-resistance boundary: this boundary authenticates the delivered
/// persisted public spend contract. Current-stack spend verification is real
/// and fail closed, and the regular public spend contract binds a
/// deterministic nullifier semantics surface for proof-level replay hygiene
/// and duplicate rejection. That field is not the storage authority for
/// consumed state; the
/// checkpoint path still resolves inputs by asset id, verifies membership under
/// `prev_root`, and consumes them by deleting the asset leaf.
/// Deterministic `chain_id || s_in` derivation is enforced in the witness
/// bridge and structural rule layer; this standalone public verifier
/// authenticates the signed field and rejects malformed, duplicate, or
/// post-signature drift on the current proof/auth seam. The strongest shipped
/// `leaf_ad_id` proof stays limited to
/// wallet, scan, report, and spend-witness bridge paths rather than
/// repository-wide total closure. It does not prove the same receiver-secret
/// plus `s_out` wallet-local post-scan exclusivity gate at the validator-facing
/// seam or upgrade that accepted-path gate into a public trustless theorem.
/// The live contract is already live, but it is still narrower than a finished
/// full-ZK spend theorem.
/// Semantically incomplete but structurally plausible artifacts must therefore
/// still fail closed before checkpoint/state mutation.
pub fn verify_tx_public_spend_contract(
    chain_id: u32,
    tx_version: u8,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
) -> Result<(), SpendPublicErr> {
    let proof = tx
        .proof
        .spend
        .as_ref()
        .ok_or(SpendPublicErr::MissingProof)?;
    let auth = tx.auth.spend.as_ref().ok_or(SpendPublicErr::MissingAuth)?;
    let proof = canonicalize_spend_proof(proof)?;
    let auth = canonicalize_spend_auth(auth)?;
    if proof.ver != SPEND_PROOF_WIRE_VER {
        return Err(SpendPublicErr::BadProofVersion);
    }
    if proof.proof_suite != SPEND_PROOF_SUITE {
        return Err(SpendPublicErr::BadProofSuite);
    }
    if auth.ver != SPEND_AUTH_WIRE_VER {
        return Err(SpendPublicErr::BadAuthVersion);
    }
    if proof.inputs.len() != tx.inputs.len() {
        return Err(SpendPublicErr::InputCountMismatch);
    }

    let prev_root = decode_hex32(&proof.prev_root_hex, "proof.prev_root_hex")?;
    if prev_root == [0u8; 32] {
        return Err(SpendPublicErr::BadPrevRoot);
    }

    let card = decode_card_compact(&auth.receiver_card_compact)
        .map_err(|err| SpendPublicErr::BadCard(err.to_string()))?;
    card.verify()
        .map_err(|err| SpendPublicErr::BadCard(err.to_string()))?;

    if proof.statement_hex.is_empty() {
        return Err(SpendPublicErr::MissingStatement);
    }
    if proof.proof_hex.is_empty() {
        return Err(SpendPublicErr::MissingProofBlob);
    }

    let prover = ProverImpl::new().map_err(|err| SpendPublicErr::BadRangeProof {
        idx: 0,
        reason: err.to_string(),
    })?;

    let mut seen_inputs = HashSet::new();
    let mut seen_input_leaf_ids = HashSet::new();
    let mut seen_nullifiers = HashSet::new();
    let mut input_commitments = Vec::with_capacity(proof.inputs.len());
    for (idx, (tx_input, proof_input)) in tx.inputs.iter().zip(proof.inputs.iter()).enumerate() {
        let tx_input_asset_id = decode_tx_input_asset_id(&tx_input.asset_id_hex).map_err(|_| {
            SpendPublicErr::InvalidHex {
                label: "tx.inputs[].asset_id_hex",
            }
        })?;
        if hex::encode(tx_input_asset_id) != proof_input.input_asset_id_hex {
            return Err(SpendPublicErr::InputRefMismatch { idx });
        }
        if tx_input.serial_id != proof_input.serial_id {
            return Err(SpendPublicErr::InputRefMismatch { idx });
        }
        if !seen_inputs.insert(tx_input.asset_id_hex.as_str()) {
            return Err(SpendPublicErr::DuplicateInputRef);
        }
        let leaf_ad_hash = compute_input_leaf_ad_hash(proof_input)?;
        if hex::encode(leaf_ad_hash) != proof_input.leaf_ad_hash_hex {
            return Err(SpendPublicErr::InputLeafAdHashMismatch { idx });
        }
        let nullifier = decode_hex32(&proof_input.nullifier_hex, "proof.inputs[].nullifier_hex")?;
        if !seen_nullifiers.insert(nullifier) {
            return Err(SpendPublicErr::DuplicateNullifier);
        }
        let leaf_ad_id =
            decode_hex32(&proof_input.leaf_ad_id_hex, "proof.inputs[].leaf_ad_id_hex")?;
        // Historical wire-level leaf_ad_id may repeat across claim-owned
        // inputs, but proof.inputs[].leaf_ad_id_hex is theorem-derived from
        // s_in and therefore remains per-input. Keep the set only for later
        // input/output overlap checks.
        seen_input_leaf_ids.insert(leaf_ad_id);
        let commitment_bytes =
            decode_hex32(&proof_input.commitment_hex, "proof.inputs[].commitment_hex")?;
        let commitment = z00z_crypto::Commitment::from_bytes(&commitment_bytes).map_err(|_| {
            SpendPublicErr::InvalidHex {
                label: "proof.inputs[].commitment_hex",
            }
        })?;
        input_commitments.push(commitment.as_commitment().clone());
    }

    let input_leaf_ids = seen_input_leaf_ids;
    let mut output_commitments = Vec::with_capacity(tx.outputs.len());
    let mut output_leaf_ids = HashSet::new();
    for (idx, output) in tx.outputs.iter().enumerate() {
        let wire = &output.asset_wire;
        let leaf_ad_id = wire.leaf_ad_id.ok_or(SpendPublicErr::MissingOutputField {
            idx,
            field: "leaf_ad_id",
        })?;
        if !output_leaf_ids.insert(leaf_ad_id) {
            return Err(SpendPublicErr::DuplicateLeafAdId);
        }
        compute_output_leaf_ad_hash(idx, output)?;

        let commitment_bytes: [u8; 32] = wire.commitment.as_bytes().try_into().map_err(|_| {
            SpendPublicErr::MissingOutputField {
                idx,
                field: "commitment",
            }
        })?;
        let commitment = z00z_crypto::Commitment::from_bytes(&commitment_bytes).map_err(|_| {
            SpendPublicErr::MissingOutputField {
                idx,
                field: "commitment",
            }
        })?;
        output_commitments.push(commitment.as_commitment().clone());

        match wire.definition.class {
            AssetClass::Coin | AssetClass::Token => {
                let proof_bytes = wire
                    .range_proof
                    .as_ref()
                    .ok_or(SpendPublicErr::MissingRangeProof { idx })?;
                let ok = prover
                    .verify_proof(proof_bytes, wire.commitment.as_bytes())
                    .map_err(|err| SpendPublicErr::BadRangeProof {
                        idx,
                        reason: err.to_string(),
                    })?;
                if !ok {
                    return Err(SpendPublicErr::BadRangeProof {
                        idx,
                        reason: "range proof verification returned false".to_string(),
                    });
                }
            }
            AssetClass::Nft | AssetClass::Void => {
                if let Some(proof_bytes) = wire.range_proof.as_ref() {
                    let ok = prover
                        .verify_proof(proof_bytes, wire.commitment.as_bytes())
                        .map_err(|err| SpendPublicErr::BadRangeProof {
                            idx,
                            reason: err.to_string(),
                        })?;
                    if !ok {
                        return Err(SpendPublicErr::BadRangeProof {
                            idx,
                            reason: "range proof verification returned false".to_string(),
                        });
                    }
                }
            }
        }
    }

    if !input_leaf_ids.is_disjoint(&output_leaf_ids) {
        return Err(SpendPublicErr::InputOutputLeafOverlap);
    }

    if input_commitments.is_empty() || output_commitments.is_empty() {
        return Err(SpendPublicErr::BadBalance);
    }

    let input_sum = input_commitments
        .iter()
        .skip(1)
        .fold(input_commitments[0].clone(), |acc, item| &acc + item);
    let output_sum = output_commitments
        .iter()
        .skip(1)
        .fold(output_commitments[0].clone(), |acc, item| &acc + item);
    if input_sum != output_sum {
        return Err(SpendPublicErr::BadBalance);
    }

    let stmt = build_spend_proof_stmt(
        chain_id, tx_version, chain_type, chain_name, tx, &card, &proof,
    )?;
    let carried_statement = decode_canonical_hex(&proof.statement_hex, "proof.statement_hex")?;
    if carried_statement != stmt.statement {
        return Err(SpendPublicErr::StatementMismatch);
    }
    let artifact = SpendProofArtifact::from_wire_hex(&proof.proof_hex).map_err(map_backend_err)?;
    let backend = default_spend_proof_backend();
    backend.verify(&stmt, &artifact).map_err(map_backend_err)?;
    let identity_pk = Z00ZRistrettoPoint::try_from_bytes(card.identity_pk)
        .map_err(|err| SpendPublicErr::BadCard(err.to_string()))?;
    let sig = decode_sig_hex(&auth.spend_sig_hex)?;
    let ok = verify_spend_authorization(&identity_pk, &stmt.statement, &sig)
        .map_err(|err| SpendPublicErr::BadSignature(err.to_string()))?;
    if !ok {
        return Err(SpendPublicErr::AuthorizationFailed);
    }

    Ok(())
}

pub trait SpendProofApi {
    fn bind_root(&mut self, prev_root: CheckRoot) -> Result<(), SpendProofErr>;
    fn prove_input(
        &mut self,
        idx: usize,
        inp: &SpendInputRef,
        leaf: &SpendInputLeaf,
        s_in: [u8; 32],
        recv_sec: [u8; 32],
    ) -> Result<(), SpendProofErr>;
    fn check_balance(
        &mut self,
        c_ins: &[[u8; 32]],
        c_outs: &[[u8; 32]],
    ) -> Result<(), SpendProofErr>;
}

pub fn build_spend_assets(
    cs: &mut impl SpendProofApi,
    st: &SpendPlan,
    wit: &SpendWitness,
) -> Result<(), SpendBuildErr> {
    if st.inputs.is_empty() {
        return Err(SpendBuildErr::EmptyInputs);
    }
    if st.outputs.is_empty() {
        return Err(SpendBuildErr::EmptyOutputs);
    }
    if st.inputs.len() != st.leaf_sums.len() || st.inputs.len() != wit.s_in_vec.len() {
        return Err(SpendBuildErr::InputMismatch);
    }
    if wit.recv_sec == [0u8; 32] {
        return Err(SpendBuildErr::BadWitness { idx: 0 });
    }

    let mut seen = BTreeSet::new();
    for inp in &st.inputs {
        if !seen.insert(inp.asset_id) {
            return Err(SpendBuildErr::DupInput);
        }
    }

    cs.bind_root(st.prev_root)?;

    for (idx, ((inp, leaf), s_in)) in st
        .inputs
        .iter()
        .zip(st.leaf_sums.iter())
        .zip(wit.s_in_vec.iter())
        .enumerate()
    {
        if inp.asset_id != leaf.asset_id || inp.serial_id != leaf.serial_id {
            return Err(SpendBuildErr::InputId { idx });
        }
        if *s_in == [0u8; 32] {
            return Err(SpendBuildErr::BadWitness { idx });
        }
        if leaf.leaf_ad_id == [0u8; 32]
            || leaf.r_pub == [0u8; 32]
            || leaf.owner_tag == [0u8; 32]
            || leaf.c_amt == [0u8; 32]
        {
            return Err(SpendBuildErr::BadLeaf { idx });
        }
        cs.prove_input(idx, inp, leaf, *s_in, wit.recv_sec)?;
    }

    let receiver_secret =
        ReceiverSecret::from_bytes(wit.recv_sec).map_err(|_| SpendBuildErr::BadSecret)?;

    let spend_ins: Vec<SpendIn> = st
        .leaf_sums
        .iter()
        .zip(wit.s_in_vec.iter())
        .enumerate()
        .map(|(idx, (leaf, s_in))| {
            let c_in = z00z_crypto::Commitment::from_bytes(&leaf.c_amt)
                .map_err(|_| SpendBuildErr::BadComIn { idx })?
                .as_commitment()
                .clone();
            Ok(SpendIn {
                chain_id: st.chain_id,
                r_pub_in: leaf.r_pub,
                owner_tag_in: leaf.owner_tag,
                leaf_ad_id_in: leaf.leaf_ad_id,
                nullifier_in: Some(derive_spend_nullifier(st.chain_id, s_in)),
                s_in: *s_in,
                c_in,
            })
        })
        .collect::<Result<Vec<_>, SpendBuildErr>>()?;

    let c_outs: Vec<Z00ZCommitment> = st
        .outputs
        .iter()
        .enumerate()
        .map(|(idx, leaf)| {
            z00z_crypto::Commitment::from_bytes(&leaf.c_amount)
                .map(|commitment| commitment.as_commitment().clone())
                .map_err(|_| SpendBuildErr::BadComOut { idx })
        })
        .collect::<Result<Vec<_>, SpendBuildErr>>()?;

    let rules_stmt = SpendStmt {
        receiver_secret,
        spend_ins,
        c_outs,
        range_ok: true,
    };
    verify_spend_rules(&rules_stmt).map_err(|_| SpendBuildErr::BadRules)?;

    let c_ins: Vec<[u8; 32]> = st.leaf_sums.iter().map(|x| x.c_amt).collect();
    let c_outs: Vec<[u8; 32]> = st.outputs.iter().map(|x| x.c_amount).collect();
    cs.check_balance(&c_ins, &c_outs)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tx_output::compute_tx_digest_from_wire;
    use super::{
        build_public_spend_contract, build_spend_assets, verify_tx_public_spend_contract,
        SpendBuildErr, SpendInputLeaf, SpendInputRef, SpendPlan, SpendProofApi, SpendProofErr,
        SpendProofWitness, SpendPublicErr, SpendWitness,
    };
    use z00z_core::{
        assets::AssetPkgWire, genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire,
    };
    use z00z_crypto::{
        create_commitment, domains::AssetIdDomain, hash_zk::hash_zk, Z00ZRistrettoPoint, Z00ZScalar,
    };
    use z00z_storage::settlement::{
        CheckRoot, DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, StoreOp,
        TerminalId,
    };

    use crate::{
        key::{derive_owner_handle, derive_view_secret_key},
        stealth::{
            ecdh::compute_dh_sender,
            kdf::{compute_owner_tag, derive_k_dh},
        },
        tx::{SpendMembershipWitness, TxInputWire, TxOutRole, TxOutputWire, TxWire},
    };
    use z00z_storage::settlement::TerminalLeaf;

    struct TestCs;

    impl SpendProofApi for TestCs {
        fn bind_root(&mut self, prev_root: CheckRoot) -> Result<(), SpendProofErr> {
            if prev_root == [0u8; 32].into() {
                return Err(SpendProofErr::BindRoot);
            }
            Ok(())
        }

        fn prove_input(
            &mut self,
            idx: usize,
            _inp: &SpendInputRef,
            _leaf: &SpendInputLeaf,
            s_in: [u8; 32],
            recv_sec: [u8; 32],
        ) -> Result<(), SpendProofErr> {
            if s_in == [0u8; 32] || recv_sec == [0u8; 32] {
                return Err(SpendProofErr::Input { idx });
            }
            Ok(())
        }

        fn check_balance(
            &mut self,
            c_ins: &[[u8; 32]],
            c_outs: &[[u8; 32]],
        ) -> Result<(), SpendProofErr> {
            if c_ins.is_empty() || c_outs.is_empty() {
                return Err(SpendProofErr::Balance);
            }
            Ok(())
        }
    }

    fn test_scalar(seed: u64) -> Z00ZScalar {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&seed.to_le_bytes());
        Z00ZScalar::try_from_bytes(bytes).expect("scalar")
    }

    fn one_stmt() -> SpendPlan {
        let recv_sec = [2u8; 32];
        let recv_secret =
            crate::key::ReceiverSecret::from_bytes(recv_sec).expect("receiver secret");
        let view_sk = derive_view_secret_key(&recv_secret).expect("view");
        let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

        let r = test_scalar(77);
        let r_pub = Z00ZRistrettoPoint::from_secret_key(&r).to_bytes();
        let dh = compute_dh_sender(&r, &view_pk).expect("dh");
        let k_in = derive_k_dh(&dh);
        let owner_handle = derive_owner_handle(&recv_secret);
        let owner_tag = compute_owner_tag(&owner_handle, &k_in);

        let s_in = [7u8; 32];
        let asset_id = hash_zk::<AssetIdDomain>("", &[&s_in]);

        let blind = test_scalar(41);
        let c_in = create_commitment(12, &blind).expect("c_in");
        let c_in_bytes: [u8; 32] = c_in.as_bytes().try_into().expect("commitment bytes");

        let out = TerminalLeaf::from(z00z_core::assets::AssetLeaf {
            c_amount: c_in_bytes,
            ..z00z_core::assets::AssetLeaf::default()
        });
        SpendPlan {
            chain_id: 3,
            prev_root: [1u8; 32].into(),
            inputs: vec![SpendInputRef {
                asset_id,
                serial_id: 1,
            }],
            leaf_sums: vec![SpendInputLeaf {
                asset_id,
                serial_id: 1,
                leaf_ad_id: asset_id,
                r_pub,
                owner_tag,
                c_amt: c_in_bytes,
            }],
            outputs: vec![out],
        }
    }

    fn recv_sec() -> [u8; 32] {
        [0x11u8; 32]
    }

    fn membership_for_wires(
        wires: &[AssetWire],
        tx_inputs: &[TxInputWire],
    ) -> (CheckRoot, Vec<SpendMembershipWitness>) {
        let items = wires
            .iter()
            .zip(tx_inputs.iter())
            .map(|(wire, input)| {
                let asset_id: [u8; 32] = hex::decode(&input.asset_id_hex)
                    .expect("input asset id hex")
                    .try_into()
                    .expect("input asset id bytes");
                let mut leaf = crate::tx::asset_wire_to_leaf(wire).expect("input leaf");
                leaf.set_terminal_id(TerminalId::new(asset_id));
                let path = SettlementPath::new(
                    DefinitionId::new(wire.definition.id),
                    SerialId::new(input.serial_id),
                    TerminalId::new(asset_id),
                );
                (path, leaf)
            })
            .collect::<Vec<_>>();
        let mut store = SettlementStore::new();
        let ops = items
            .iter()
            .map(|(path, leaf)| {
                StoreOp::Put(Box::new(
                    StoreItem::new(*path, leaf.clone()).expect("store item"),
                ))
            })
            .collect::<Vec<_>>();
        let root = CheckRoot::from(
            store
                .apply_settlement_ops(ops)
                .expect("apply input store ops"),
        );
        let membership = items
            .into_iter()
            .map(|(path, leaf)| {
                let proof_item = store.settlement_proof_item(&path).expect("proof item");
                let proof = store
                    .settlement_proof_blob(&path)
                    .expect("proof blob")
                    .encode()
                    .expect("proof bytes");
                SpendMembershipWitness::new(path, leaf, proof, proof_item)
                    .expect("membership witness")
            })
            .collect::<Vec<_>>();
        (root, membership)
    }

    fn make_public_contract_tx(chain_id: u32) -> TxWire {
        let keys = crate::key::ReceiverKeys::from_receiver_secret(
            crate::key::ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
        )
        .expect("receiver keys");
        let card = keys.export_receiver_card().expect("card");

        let input_asset = asset_from_dev_class(AssetClass::Coin, 7, 55).expect("input asset");
        let input_leaf = crate::stealth::build_card_stealth_leaf(
            &card,
            input_asset.amount,
            input_asset.serial_id,
        )
        .expect("input leaf");
        let input_wire = crate::stealth::bind_stealth_output_wire(
            AssetWire::from_asset(&input_asset),
            &input_leaf,
        )
        .expect("input wire");
        let mut output_wire = input_wire.clone();
        output_wire.leaf_ad_id = Some([0x77; 32]);

        let tx_input = TxInputWire {
            asset_id_hex: hex::encode(
                crate::tx::asset_wire_to_leaf(&input_wire)
                    .expect("input leaf ref")
                    .asset_id,
            ),
            serial_id: input_wire.serial_id,
        };
        let tx_output = TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_wire(&output_wire),
        };
        let proof_inputs = crate::tx::prepare_spend_public_inputs(
            chain_id,
            recv_sec(),
            std::slice::from_ref(&input_wire),
            std::slice::from_ref(&tx_input),
        )
        .expect("proof inputs");
        let (prev_root, membership) = membership_for_wires(
            std::slice::from_ref(&input_wire),
            std::slice::from_ref(&tx_input),
        );

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
            chain_id,
            1,
            "spend_verification",
            "spend_verification",
            &tx,
            prev_root,
            proof_inputs,
            SpendProofWitness {
                receiver_secret: crate::key::ReceiverSecret::from_bytes(recv_sec())
                    .expect("receiver secret"),
                input_s_in: vec![
                    crate::tx::resolve_input_pack(recv_sec(), &input_wire)
                        .expect("input pack")
                        .s_out,
                ],
                membership,
            },
        )
        .expect("public spend contract");
        tx.proof = proof;
        tx.auth = auth;
        tx
    }

    fn make_two_input_public_tx(chain_id: u32) -> TxWire {
        let keys = crate::key::ReceiverKeys::from_receiver_secret(
            crate::key::ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
        )
        .expect("receiver keys");
        let card = keys.export_receiver_card().expect("card");

        let left_asset = asset_from_dev_class(AssetClass::Coin, 7, 55).expect("left asset");
        let left_leaf =
            crate::stealth::build_card_stealth_leaf(&card, left_asset.amount, left_asset.serial_id)
                .expect("left leaf");
        let left_wire = crate::stealth::bind_stealth_output_wire(
            AssetWire::from_asset(&left_asset),
            &left_leaf,
        )
        .expect("left wire");

        let right_asset = asset_from_dev_class(AssetClass::Coin, 8, 66).expect("right asset");
        let right_leaf = crate::stealth::build_card_stealth_leaf(
            &card,
            right_asset.amount,
            right_asset.serial_id,
        )
        .expect("right leaf");
        let right_wire = crate::stealth::bind_stealth_output_wire(
            AssetWire::from_asset(&right_asset),
            &right_leaf,
        )
        .expect("right wire");

        let tx_inputs = vec![
            TxInputWire {
                asset_id_hex: hex::encode(
                    crate::tx::asset_wire_to_leaf(&left_wire)
                        .expect("left leaf ref")
                        .asset_id,
                ),
                serial_id: left_wire.serial_id,
            },
            TxInputWire {
                asset_id_hex: hex::encode(
                    crate::tx::asset_wire_to_leaf(&right_wire)
                        .expect("right leaf ref")
                        .asset_id,
                ),
                serial_id: right_wire.serial_id,
            },
        ];
        let mut left_output = left_wire.clone();
        left_output.leaf_ad_id = Some([0x71; 32]);
        let mut right_output = right_wire.clone();
        right_output.leaf_ad_id = Some([0x72; 32]);
        let tx_outputs = vec![
            TxOutputWire {
                role: TxOutRole::Recipient,
                asset_wire: AssetPkgWire::from_wire(&left_output),
            },
            TxOutputWire {
                role: TxOutRole::Recipient,
                asset_wire: AssetPkgWire::from_wire(&right_output),
            },
        ];
        let proof_inputs = crate::tx::prepare_spend_public_inputs(
            chain_id,
            recv_sec(),
            &[left_wire.clone(), right_wire.clone()],
            &tx_inputs,
        )
        .expect("proof inputs");
        let (prev_root, membership) =
            membership_for_wires(&[left_wire.clone(), right_wire.clone()], &tx_inputs);

        let mut tx = TxWire {
            tx_type: "regular_tx".to_string(),
            inputs: tx_inputs,
            outputs: tx_outputs,
            fee: 0,
            nonce: 0,
            context: Default::default(),
            proof: Default::default(),
            auth: Default::default(),
        };
        let (proof, auth) = build_public_spend_contract(
            &keys,
            chain_id,
            1,
            "spend_verification",
            "spend_verification",
            &tx,
            prev_root,
            proof_inputs,
            SpendProofWitness {
                receiver_secret: crate::key::ReceiverSecret::from_bytes(recv_sec())
                    .expect("receiver secret"),
                input_s_in: vec![
                    crate::tx::resolve_input_pack(recv_sec(), &left_wire)
                        .expect("left input pack")
                        .s_out,
                    crate::tx::resolve_input_pack(recv_sec(), &right_wire)
                        .expect("right input pack")
                        .s_out,
                ],
                membership,
            },
        )
        .expect("public spend contract");
        tx.proof = proof;
        tx.auth = auth;
        tx
    }

    #[test]
    fn test_spend_assets_ok() {
        let mut cs = TestCs;
        let st = one_stmt();
        let wit = SpendWitness {
            recv_sec: [2u8; 32],
            s_in_vec: vec![[7u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert!(res.is_ok());
    }

    #[test]
    fn test_spend_assets_mismatch() {
        let mut cs = TestCs;
        let mut st = one_stmt();
        st.leaf_sums.clear();
        let wit = SpendWitness {
            recv_sec: [2u8; 32],
            s_in_vec: vec![[7u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert_eq!(res, Err(SpendBuildErr::InputMismatch));
    }

    #[test]
    fn test_spend_assets_dupin() {
        let mut cs = TestCs;
        let mut st = one_stmt();
        st.inputs.push(SpendInputRef {
            asset_id: st.inputs[0].asset_id,
            serial_id: st.inputs[0].serial_id.saturating_add(1),
        });
        st.leaf_sums.push(st.leaf_sums[0].clone());
        let wit = SpendWitness {
            recv_sec: [2u8; 32],
            s_in_vec: vec![[7u8; 32], [8u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert_eq!(res, Err(SpendBuildErr::DupInput));
    }

    #[test]
    fn test_spend_assets_bad_wit() {
        let mut cs = TestCs;
        let st = one_stmt();
        let wit = SpendWitness {
            recv_sec: [0u8; 32],
            s_in_vec: vec![[7u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert_eq!(res, Err(SpendBuildErr::BadWitness { idx: 0 }));
    }

    #[test]
    fn test_spend_assets_bad_leaf() {
        let mut cs = TestCs;
        let mut st = one_stmt();
        st.leaf_sums[0].owner_tag = [0u8; 32];
        let wit = SpendWitness {
            recv_sec: [2u8; 32],
            s_in_vec: vec![[7u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert_eq!(res, Err(SpendBuildErr::BadLeaf { idx: 0 }));
    }

    #[test]
    fn test_spend_assets_bad_sin() {
        let mut cs = TestCs;
        let st = one_stmt();
        let wit = SpendWitness {
            recv_sec: [2u8; 32],
            s_in_vec: vec![[9u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert_eq!(res, Err(SpendBuildErr::BadRules));
    }

    #[test]
    fn test_spend_assets_bad_rpub() {
        let mut cs = TestCs;
        let mut st = one_stmt();
        st.leaf_sums[0].r_pub = [1u8; 32];
        let wit = SpendWitness {
            recv_sec: [2u8; 32],
            s_in_vec: vec![[7u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert_eq!(res, Err(SpendBuildErr::BadRules));
    }

    #[test]
    fn test_spend_assets_bad_tag() {
        let mut cs = TestCs;
        let mut st = one_stmt();
        st.leaf_sums[0].owner_tag[0] ^= 1;
        let wit = SpendWitness {
            recv_sec: [2u8; 32],
            s_in_vec: vec![[7u8; 32]],
        };
        let res = build_spend_assets(&mut cs, &st, &wit);
        assert_eq!(res, Err(SpendBuildErr::BadRules));
    }

    #[test]
    fn test_rejects_bad_nullifier_hex() {
        let mut tx = make_public_contract_tx(3);
        tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex = "zz".to_string();

        let err =
            verify_tx_public_spend_contract(3, 1, "spend_verification", "spend_verification", &tx)
                .expect_err("malformed nullifier hex must reject the public contract verifier");

        assert_eq!(
            err,
            SpendPublicErr::InvalidHex {
                label: "proof.inputs[].nullifier_hex",
            }
        );
    }

    #[test]
    fn test_rejects_bare_digest_root() {
        let mut tx = make_public_contract_tx(3);
        let wire_digest = compute_tx_digest_from_wire(&tx).expect("wire digest");
        tx.proof.spend.as_mut().expect("spend proof").statement_hex = hex::encode(wire_digest);

        let err =
            verify_tx_public_spend_contract(3, 1, "spend_verification", "spend_verification", &tx)
                .expect_err("bare wire digest must not be accepted as the only public spend root");

        assert_eq!(err, SpendPublicErr::StatementMismatch);
    }

    #[test]
    fn test_rejects_signed_nullifier_drift() {
        let mut tx = make_public_contract_tx(3);
        tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex =
            hex::encode([0xAB; 32]);

        let err =
            verify_tx_public_spend_contract(3, 1, "spend_verification", "spend_verification", &tx)
                .expect_err(
                    "post-signature nullifier drift must reject the public contract verifier",
                );

        assert_eq!(err, SpendPublicErr::StatementMismatch);
    }

    #[test]
    fn test_public_rejects_duplicate_nullifier() {
        let mut tx = make_two_input_public_tx(3);
        let duplicate = tx.proof.spend.as_ref().expect("spend proof").inputs[0]
            .nullifier_hex
            .clone();
        tx.proof.spend.as_mut().expect("spend proof").inputs[1].nullifier_hex = duplicate;

        let err =
            verify_tx_public_spend_contract(3, 1, "spend_verification", "spend_verification", &tx)
                .expect_err("duplicate nullifier must reject the public contract verifier");

        assert_eq!(err, SpendPublicErr::DuplicateNullifier);
    }
}
