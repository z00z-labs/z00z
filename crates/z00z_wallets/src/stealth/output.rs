use rand::{rngs::StdRng, RngCore, SeedableRng};
#[cfg(feature = "test-params-fast")]
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use z00z_core::{
    assets::{Asset, AssetClass, AssetLeaf, AssetPackPlain},
    AssetWire,
};
use z00z_crypto::{create_commitment, Z00ZScalar, ZkPackEncrypted};
use z00z_storage::settlement::TerminalLeaf;
#[cfg(all(debug_assertions, not(test), not(feature = "test-params-fast")))]
use z00z_utils::config::{ConfigSource, EnvConfig};
use z00z_utils::rng::SystemRngProvider;

use crate::key::{derive_owner_handle, derive_view_secret_key, ReceiverKeys, ReceiverSecret};
use crate::receiver::EphemeralCache;
use crate::receiver::{
    PaymentRequest, PinnedReceiverCards, ReceiverCard, TrustLevel, ValidatePaymentRequest,
    ValidateReceiverCard, ValidationOutcome, ValidityStatus,
};
use crate::stealth::zkpack::ZkPack;
use crate::tx::{OutputBundle, TxOutRole};
use crate::WalletError;

use super::{
    compute_dh_receiver, compute_dh_sender, compute_leaf_ad, compute_r_pub, compute_tag16,
    compute_tag16_with_req, decode_public_key, decode_r_pub, derive_k_dh, derive_k_dh_with_req,
    derive_r_hedged, derive_s_out, encode_r_pub, generate_r_retry, get_rng_bytes,
    output_validator::{validate_output_self, SenderValidationCtx, TagMode},
    StealthError,
};

const DEFAULT_R_CAP: usize = 64;
const MAX_R_RETRY: u32 = 4;
const LIGHT_SERIAL_ID: u32 = 0;
const SERIAL_MIN_ID: u32 = 1;
const SERIAL_MAX_ID: u32 = 50_000;

/// Canonical stealth output header fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxStealthOutput {
    /// Ephemeral public key, compressed form.
    pub r_pub: [u8; 32],
    /// Ownership tag for fast receiver filtering.
    pub owner_tag: [u8; 32],
    /// Optional 16-bit prefilter tag.
    pub tag16: Option<u16>,
    /// Encrypted asset metadata payload.
    pub enc_pack: ZkPackEncrypted,
    /// Commitment bytes bound into leaf associated data.
    pub c_amount: [u8; 32],
}

/// Sender context for stealth output construction.
#[derive(Debug)]
pub struct SenderWallet {
    /// Sender-held secret salt used in hedged ephemeral scalar derivation.
    pub secret_salt: [u8; 32],
    recent_r: EphemeralCache,
}

impl SenderWallet {
    /// Create sender context with default duplicate-R cache capacity.
    pub fn new(secret_salt: [u8; 32]) -> Self {
        Self::with_cap(secret_salt, DEFAULT_R_CAP)
    }

    /// Create sender context with explicit duplicate-R cache capacity.
    pub fn with_cap(secret_salt: [u8; 32], capacity: usize) -> Self {
        Self {
            secret_salt,
            recent_r: EphemeralCache::new(capacity),
        }
    }
}

/// Strict request-check context for validated output builds.
pub struct BuildCheck<'a> {
    /// Current pinned request-card state.
    pub pins: &'a mut PinnedReceiverCards,
    /// Active chain id used during approval checks.
    pub chain_id: u32,
}

include!("output_build.rs");

fn commitment_bytes(commitment: &z00z_crypto::Z00ZCommitment) -> [u8; 32] {
    let bytes = commitment.as_bytes();
    debug_assert_eq!(bytes.len(), 32, "commitment must be exactly 32 bytes");
    bytes
        .try_into()
        .expect("commitment must be exactly 32 bytes")
}

fn validate_serial_id(serial_id: u32) -> Result<(), WalletError> {
    if !(SERIAL_MIN_ID..=SERIAL_MAX_ID).contains(&serial_id) {
        return Err(WalletError::InvalidTransaction(format!(
            "serial_id {} out of range [{}, {}]",
            serial_id, SERIAL_MIN_ID, SERIAL_MAX_ID
        )));
    }
    Ok(())
}

#[cfg(all(debug_assertions, not(test), not(feature = "test-params-fast")))]
fn ensure_release_for_range_proof() -> Result<(), WalletError> {
    let env = EnvConfig;
    if matches!(env.get("Z00Z_ALLOW_DEBUG_RANGE_PROOF"), Ok(Some(_))) {
        return Ok(());
    }
    Err(WalletError::CryptoError(
        "range proof generation in debug build is disabled for production paths; use --release"
            .to_string(),
    ))
}

#[cfg(any(not(debug_assertions), test, feature = "test-params-fast"))]
fn ensure_release_for_range_proof() -> Result<(), WalletError> {
    Ok(())
}

fn build_output_core(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
    blinding: &Z00ZScalar,
    range_proof: Vec<u8>,
) -> Result<TerminalLeaf, WalletError> {
    let commitment = create_commitment(value, blinding)
        .map_err(|err| WalletError::CryptoError(err.to_string()))?;
    let c_amount = commitment_bytes(&commitment);

    let owner_tag = compute_owner_tag(owner_handle, k_dh);
    let asset_id =
        z00z_crypto::hash_zk::hash_zk::<z00z_crypto::domains::AssetIdDomain>("", &[&s_out]);
    let leaf_ad = compute_leaf_ad(&asset_id, serial_id, r_pub, &owner_tag, &c_amount);

    let payload = AssetPackPlain {
        value,
        blinding: blinding.to_bytes(),
        s_out,
    }
    .to_bytes();

    let enc_pack = ZkPack::encrypt(k_dh, &leaf_ad, r_pub, &asset_id, serial_id, &payload);
    let tag16 = compute_tag16(k_dh, &leaf_ad);

    Ok(TerminalLeaf {
        asset_id,
        serial_id,
        r_pub: *r_pub,
        owner_tag,
        c_amount,
        enc_pack,
        range_proof,
        tag16,
    })
}

/// Build a stealth terminal leaf with caller-provided randomness for the range proof.
///
/// The caller supplies the already-derived DH key, receiver public data, output
/// secret, and blinding factor. This compatibility helper validates `serial_id`
/// and emits the same encrypted payload and owner-tag formula as the canonical
/// sender path, but it does not own sender-side duplicate-ephemeral-key policy.
pub fn build_stealth_leaf_with_rng<R: rand::CryptoRng + rand::RngCore>(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
    blinding: &z00z_crypto::Hidden<Z00ZScalar>,
    rng: &mut R,
) -> Result<TerminalLeaf, WalletError> {
    validate_serial_id(serial_id)?;
    ensure_release_for_range_proof()?;

    let range_proof = z00z_crypto::create_range_proof_rng(
        value,
        blinding.reveal(),
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::MIN_VALUE_PROMISE,
        rng,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    build_output_core(
        k_dh,
        r_pub,
        owner_handle,
        value,
        serial_id,
        s_out,
        blinding.reveal(),
        range_proof,
    )
}

/// Build a stealth terminal leaf with an explicit hidden blinding factor.
///
/// This helper uses the default range-proof RNG path from `z00z_crypto` and is
/// intended for compatibility callers that already hold the derived stealth
/// context. Stateful wallet sends should prefer the `build_tx_output_unchecked*`
/// family so duplicate-`R` retry policy remains wallet-owned.
pub fn build_stealth_leaf_with_blind(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
    blinding: &z00z_crypto::Hidden<Z00ZScalar>,
) -> Result<TerminalLeaf, WalletError> {
    validate_serial_id(serial_id)?;
    ensure_release_for_range_proof()?;

    let range_proof = z00z_crypto::create_range_proof(
        value,
        blinding.reveal(),
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::MIN_VALUE_PROMISE,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    build_output_core(
        k_dh,
        r_pub,
        owner_handle,
        value,
        serial_id,
        s_out,
        blinding.reveal(),
        range_proof,
    )
}

/// Build a stealth terminal leaf with a fresh local blinding factor.
///
/// This stateless helper generates only the Pedersen blinding factor locally.
/// It expects the caller to provide a valid DH key and ephemeral receiver data,
/// so it is not a replacement for the full sender-wallet construction path.
pub fn build_stealth_leaf(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
) -> Result<TerminalLeaf, WalletError> {
    let blinding = z00z_crypto::Hidden::hide(Z00ZScalar::random_secure(&SystemRngProvider)?);
    build_stealth_leaf_with_blind(
        k_dh,
        r_pub,
        owner_handle,
        value,
        serial_id,
        s_out,
        &blinding,
    )
}

/// Build a full settlement terminal leaf through the canonical stealth helper layer.
///
/// This stays on the stateless helper lane and does not claim the wallet-owned
/// hedged-`r` and duplicate-`R` retry policy used by
/// `build_tx_output_unchecked` and `build_tx_output_serial_unchecked`.
pub fn build_card_stealth_leaf(
    card: &ReceiverCard,
    amount: u64,
    serial_id: u32,
) -> Result<TerminalLeaf, WalletError> {
    validate_serial_id(serial_id)?;
    // Validate view_pk before the release gate so identity-point rejection
    // is consistent across all build profiles (including debug integration tests).
    decode_public_key(&card.view_pk).map_err(|err| {
        if matches!(err, StealthError::IdentityPointRejected) {
            WalletError::IdentityPointNotAllowed
        } else {
            WalletError::CryptoError(err.to_string())
        }
    })?;
    ensure_release_for_range_proof()?;
    let state = build_leaf_state(card, None, amount, serial_id).map_err(|err| {
        if matches!(err, StealthError::IdentityPointRejected) {
            WalletError::IdentityPointNotAllowed
        } else {
            WalletError::CryptoError(err.to_string())
        }
    })?;
    let range_proof = z00z_crypto::create_range_proof(
        amount,
        &state.blinding,
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::MIN_VALUE_PROMISE,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    Ok(TerminalLeaf {
        asset_id: state.asset_id,
        serial_id,
        r_pub: state.output.r_pub,
        owner_tag: state.output.owner_tag,
        c_amount: state.output.c_amount,
        enc_pack: state.output.enc_pack,
        range_proof,
        tag16: state.output.tag16.unwrap_or_default(),
    })
}

/// Build one replayable output bundle on the stateless helper lane.
///
/// This helper keeps bundle construction on the canonical stealth build
/// formulas, but it does not participate in the wallet-owned H-3 hedged-`r`
/// and duplicate-`R` cache path used by the stateful tx-output builders.
/// Callers that need the wallet-owned sender policy must stay on
/// `build_tx_output_unchecked(...)` or `build_tx_output_serial_unchecked(...)`.
pub fn build_output_bundle(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
) -> Result<OutputBundle, String> {
    let mut rng = SystemRngProvider.rng();
    build_output_bundle_with_rng(receiver, role, class, card, value, serial_id, &mut rng)
}

/// Build one replayable output bundle with caller-provided randomness.
///
/// The injected RNG controls the stateless helper lane for replay and
/// test callers, while the canonical stealth formulas still stay owned by the
/// helper layer in `output_build.rs`.
pub fn build_output_bundle_with_rng<R: rand::CryptoRng + rand::RngCore>(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
    rng: &mut R,
) -> Result<OutputBundle, String> {
    validate_serial_id(serial_id).map_err(|e| e.to_string())?;
    ensure_release_for_range_proof().map_err(|e| e.to_string())?;
    let state =
        build_leaf_state_rng(card, None, value, serial_id, rng).map_err(|e| e.to_string())?;
    let range_proof = z00z_crypto::create_range_proof_rng(
        value,
        &state.blinding,
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::MIN_VALUE_PROMISE,
        rng,
    )
    .map_err(|e| e.to_string())?;
    let k_dh = state.ctx.k_dh;
    let s_out = derive_s_out(&k_dh, &state.output.r_pub, serial_id);
    let leaf = TerminalLeaf {
        asset_id: state.asset_id,
        serial_id,
        r_pub: state.output.r_pub,
        owner_tag: state.output.owner_tag,
        c_amount: state.output.c_amount,
        enc_pack: state.output.enc_pack,
        range_proof,
        tag16: state.output.tag16.unwrap_or_default(),
    };

    Ok(OutputBundle {
        receiver,
        role,
        class,
        value,
        leaf,
        k_dh,
        s_out,
    })
}

/// Build one explicit-asset, explicit-serial output bundle with deterministic seed bytes.
///
/// This helper stays on the canonical stealth-output formulas while allowing
/// local replay/test/simulation callers to drive the raw sender lane through a
/// caller-supplied seed instead of the default process RNG.
pub fn build_seeded_output_bundle(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    seed_bytes: [u8; 32],
    value: u64,
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<OutputBundle, WalletError> {
    validate_serial_id(serial_id)?;
    ensure_release_for_range_proof()?;

    let mut rng = StdRng::from_seed(seed_bytes);
    let mut rng_bytes = [0u8; 32];
    rng.fill_bytes(&mut rng_bytes);
    let (r, _) = select_r_seeded(
        sender_wallet,
        &receiver_card.owner_handle,
        tx_digest,
        out_index,
        rng_bytes,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    let state = build_output_state_with_rng(
        receiver_card,
        payment_request,
        value,
        AssetIdMode::Explicit(*asset_id),
        serial_id,
        &r,
        &mut rng,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    validate_output_self(&state.output, &state.ctx, value)
        .map_err(|err| WalletError::InvalidTransaction(err.to_string()))?;

    finish_output_bundle_rng(receiver, role, class, value, serial_id, state, &mut rng)
}

/// Build one request-bound output bundle on the same canonical stealth path.
///
/// This helper stays subordinate to the existing output-build formulas. It
/// exposes a proof-bearing `OutputBundle` for request-aware callers without
/// creating a second output construction authority.
pub fn build_request_output_bundle(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    request: &PaymentRequest,
    value: u64,
    serial_id: u32,
) -> Result<OutputBundle, WalletError> {
    validate_serial_id(serial_id)?;
    let state = build_leaf_state(card, Some(request), value, serial_id)
        .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    validate_output_self(&state.output, &state.ctx, value)
        .map_err(|err| WalletError::InvalidTransaction(err.to_string()))?;

    build_validated_output_bundle(receiver, role, class, value, serial_id, state)
}

fn build_validated_output_bundle(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    value: u64,
    serial_id: u32,
    state: OutputBuildState,
) -> Result<OutputBundle, WalletError> {
    let mut rng = SystemRngProvider.rng();
    finish_output_bundle_rng(receiver, role, class, value, serial_id, state, &mut rng)
}

fn finish_output_bundle_rng<R: rand::CryptoRng + rand::RngCore>(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    value: u64,
    serial_id: u32,
    state: OutputBuildState,
    rng: &mut R,
) -> Result<OutputBundle, WalletError> {
    ensure_release_for_range_proof()?;

    let range_proof = z00z_crypto::create_range_proof_rng(
        value,
        &state.blinding,
        z00z_crypto::RANGE_PROOF_BITS,
        z00z_crypto::MIN_VALUE_PROMISE,
        rng,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    let leaf = TerminalLeaf {
        asset_id: state.asset_id,
        serial_id,
        r_pub: state.output.r_pub,
        owner_tag: state.output.owner_tag,
        c_amount: state.output.c_amount,
        enc_pack: state.output.enc_pack,
        range_proof,
        tag16: state.output.tag16.unwrap_or_default(),
    };

    let s_out = derive_s_out(&state.ctx.k_dh, &leaf.r_pub, serial_id);

    Ok(OutputBundle {
        receiver,
        role,
        class,
        value,
        leaf,
        k_dh: state.ctx.k_dh,
        s_out,
    })
}

fn card_state_from_r(
    receiver_card: &ReceiverCard,
    amount: u64,
    serial_id: u32,
    r: &Z00ZScalar,
    blinding: Option<&Z00ZScalar>,
) -> Result<OutputBuildState, WalletError> {
    let state = match blinding {
        Some(blinding) => build_output_state_with_blinding(
            receiver_card,
            None,
            amount,
            AssetIdMode::HashFromSOut,
            serial_id,
            r,
            blinding,
        ),
        None => build_output_state_with_r(
            receiver_card,
            None,
            amount,
            AssetIdMode::HashFromSOut,
            serial_id,
            r,
        ),
    }
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    validate_output_self(&state.output, &state.ctx, amount)
        .map_err(|err| WalletError::InvalidTransaction(err.to_string()))?;

    Ok(state)
}

fn tx_state_from_r(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    amount: u64,
    serial_id: u32,
    r: &Z00ZScalar,
    blinding: Option<&Z00ZScalar>,
) -> Result<OutputBuildState, WalletError> {
    let state = match blinding {
        Some(blinding) => build_output_state_with_blinding(
            receiver_card,
            payment_request,
            amount,
            AssetIdMode::HashFromSOut,
            serial_id,
            r,
            blinding,
        ),
        None => build_output_state_with_r(
            receiver_card,
            payment_request,
            amount,
            AssetIdMode::HashFromSOut,
            serial_id,
            r,
        ),
    }
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    validate_output_self(&state.output, &state.ctx, amount)
        .map_err(|err| WalletError::InvalidTransaction(err.to_string()))?;

    Ok(state)
}

fn derive_seeded_output_rng(rng_bytes: [u8; 32], tx_digest: &[u8; 32], out_index: u32) -> StdRng {
    let out_index_bytes = out_index.to_le_bytes();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&z00z_crypto::blake2b_hash(
        b"z00z.wallet.tx.output.range_proof_rng.v1",
        &[&rng_bytes, tx_digest, &out_index_bytes],
    ));
    StdRng::from_seed(seed)
}

pub(crate) fn build_card_bundle_rng_checked<'a>(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    rng_bytes: [u8; 32],
    amount: u64,
    serial_id: u32,
    blinding: Option<&Z00ZScalar>,
) -> Result<OutputBundle, WalletError> {
    validate_serial_id(serial_id)?;
    approve_card(receiver_card, build_check)
        .map_err(|err| WalletError::InvalidTransaction(err.to_string()))?;
    let (r, _) = select_r_seeded(
        sender_wallet,
        &receiver_card.owner_handle,
        tx_digest,
        out_index,
        rng_bytes,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;
    let state = card_state_from_r(receiver_card, amount, serial_id, &r, blinding)?;
    let mut rng = derive_seeded_output_rng(rng_bytes, tx_digest, out_index);
    finish_output_bundle_rng(receiver, role, class, amount, serial_id, state, &mut rng)
}

pub(crate) fn build_tx_bundle_rng_checked<'a>(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    rng_bytes: [u8; 32],
    amount: u64,
    serial_id: u32,
    blinding: Option<&Z00ZScalar>,
) -> Result<OutputBundle, WalletError> {
    validate_serial_id(serial_id)?;
    approve_req(payment_request, build_check)
        .map_err(|err| WalletError::InvalidTransaction(err.to_string()))?;
    let (r, _) = select_r_seeded(
        sender_wallet,
        &receiver_card.owner_handle,
        tx_digest,
        out_index,
        rng_bytes,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;
    let state = tx_state_from_r(
        receiver_card,
        payment_request,
        amount,
        serial_id,
        &r,
        blinding,
    )?;
    let mut rng = derive_seeded_output_rng(rng_bytes, tx_digest, out_index);
    finish_output_bundle_rng(receiver, role, class, amount, serial_id, state, &mut rng)
}

#[doc(hidden)]
pub trait StealthOutputLeafView {
    fn serial_id(&self) -> u32;
    fn asset_id(&self) -> [u8; 32];
    fn r_pub(&self) -> [u8; 32];
    fn owner_tag(&self) -> [u8; 32];
    fn c_amount(&self) -> &[u8; 32];
    fn enc_pack(&self) -> &ZkPackEncrypted;
    fn range_proof(&self) -> &[u8];
    fn tag16(&self) -> u16;
}

impl StealthOutputLeafView for AssetLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> [u8; 32] {
        self.asset_id
    }

    fn r_pub(&self) -> [u8; 32] {
        self.r_pub
    }

    fn owner_tag(&self) -> [u8; 32] {
        self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &ZkPackEncrypted {
        &self.enc_pack
    }

    fn range_proof(&self) -> &[u8] {
        &self.range_proof
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

impl StealthOutputLeafView for TerminalLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> [u8; 32] {
        self.asset_id
    }

    fn r_pub(&self) -> [u8; 32] {
        self.r_pub
    }

    fn owner_tag(&self) -> [u8; 32] {
        self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &ZkPackEncrypted {
        &self.enc_pack
    }

    fn range_proof(&self) -> &[u8] {
        &self.range_proof
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

impl StealthOutputLeafView for &AssetLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> [u8; 32] {
        self.asset_id
    }

    fn r_pub(&self) -> [u8; 32] {
        self.r_pub
    }

    fn owner_tag(&self) -> [u8; 32] {
        self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &ZkPackEncrypted {
        &self.enc_pack
    }

    fn range_proof(&self) -> &[u8] {
        &self.range_proof
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

impl StealthOutputLeafView for &TerminalLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> [u8; 32] {
        self.asset_id
    }

    fn r_pub(&self) -> [u8; 32] {
        self.r_pub
    }

    fn owner_tag(&self) -> [u8; 32] {
        self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &ZkPackEncrypted {
        &self.enc_pack
    }

    fn range_proof(&self) -> &[u8] {
        &self.range_proof
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

/// Bind a stealth terminal leaf into the transaction wire representation.
///
/// The bridge copies receiver-visible stealth fields and range proof into an
/// `AssetWire` while clearing legacy owner-public-key fields that are not part
/// of the receiver-card flow.
pub fn bind_stealth_output_wire(
    mut wire: AssetWire,
    leaf: impl StealthOutputLeafView,
) -> Result<AssetWire, String> {
    wire.serial_id = leaf.serial_id();
    let commitment = z00z_crypto::Commitment::from_bytes(leaf.c_amount())
        .map_err(|e| format!("tx output bridge: commitment parse failed: {e}"))?;
    wire.commitment = commitment.as_commitment().clone();
    wire.range_proof = Some(leaf.range_proof().to_vec());
    wire.owner_pub = None;
    wire.owner_signature = None;
    wire.r_pub = Some(leaf.r_pub());
    wire.owner_tag = Some(leaf.owner_tag());
    wire.enc_pack = Some(leaf.enc_pack().clone());
    wire.tag16 = Some(leaf.tag16());
    wire.leaf_ad_id = Some(leaf.asset_id());
    wire.secret = None;
    Ok(wire)
}

/// Canonical crypto owner-tag formula used by wallet stealth output flows.
pub use z00z_crypto::kdf::compute_owner_tag;

/// Create owner tag from sender perspective.
pub fn create_owner_tag_sender(
    owner_handle: &[u8; 32],
    view_pk: &z00z_crypto::Z00ZRistrettoPoint,
    r: &z00z_crypto::Z00ZScalar,
) -> Result<[u8; 32], StealthError> {
    let dh = compute_dh_sender(r, view_pk)?;
    let k_dh = derive_k_dh(&dh);
    Ok(compute_owner_tag(owner_handle, &k_dh))
}

/// Constant-time equality check utility.
pub fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.ct_eq(right).into()
}

/// M1 owner tag filtering check.
pub fn m1_owner_tag_check(leaf_owner_tag: &[u8; 32], computed_owner_tag: &[u8; 32]) -> bool {
    constant_time_eq(leaf_owner_tag, computed_owner_tag)
}

/// Handle owner tag mismatch to classify leaf ownership.
pub fn handle_tag_mismatch(leaf: &Asset, expected_tag: &[u8; 32]) -> crate::receiver::ScanResult {
    if let Some(owner_tag) = &leaf.owner_tag {
        if !constant_time_eq(owner_tag, expected_tag) {
            return crate::receiver::ScanResult::NotMine;
        }

        return crate::receiver::ScanResult::MaybeMine {
            tag16_match: false,
            m1_failed: false,
        };
    }

    crate::receiver::ScanResult::NotMine
}

/// Validates mandatory stealth leaf fields on an on-chain asset.
pub fn validate_stealth_leaf_fields(asset: &Asset) -> Result<(), StealthError> {
    if asset.r_pub.is_none() {
        return Err(StealthError::InvalidStealthInput);
    }

    if asset.owner_tag.is_none() {
        return Err(StealthError::InvalidStealthInput);
    }

    if asset.enc_pack.is_none() {
        return Err(StealthError::InvalidStealthInput);
    }

    Ok(())
}

/// Extracting `k_dh` from owner tag is not invertible by design.
pub fn extract_k_dh(
    _owner_handle: &[u8; 32],
    _owner_tag: &[u8; 32],
) -> Result<[u8; 32], StealthError> {
    Err(StealthError::NotInvertible)
}

/// Verify owner tag (M1 check) from receiver keys and leaf data.
pub fn verify_owner_tag(
    receiver_keys: &ReceiverKeys,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
) -> Result<bool, StealthError> {
    verify_owner_tag_with_req(receiver_keys, r_pub, owner_tag, None)
}

/// Verify wallet-local ownership with two factors: receiver secret and `s_out`.
///
/// This is the accepted wallet-local spend ownership rule. It must not be read
/// as a statement that the current public verifier path already proves the same
/// property end to end.
/// `verify_owner_tag` remains an M1 receiver filter only.
pub fn verify_owner_two_factor(
    receiver_secret: &ReceiverSecret,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    s_out: &[u8; 32],
    serial_id: u32,
) -> Result<bool, StealthError> {
    let owner_handle = derive_owner_handle(receiver_secret);
    let view_sk =
        derive_view_secret_key(receiver_secret).map_err(|_| StealthError::InvalidStealthInput)?;

    let r_pub_decoded = decode_r_pub(r_pub)?;
    let dh = compute_dh_receiver(&view_sk, &r_pub_decoded)?;
    let k_dh = derive_k_dh(&dh);

    let owner_expected = compute_owner_tag(&owner_handle, &k_dh);
    if !tag_eq(&owner_expected, owner_tag) {
        return Ok(false);
    }

    let s_out_expected = derive_s_out(&k_dh, r_pub, serial_id);
    Ok(s_out_expected.ct_eq(s_out).into())
}

/// Verify owner tag (M1 check) with optional request binding.
pub fn verify_owner_tag_with_req(
    receiver_keys: &ReceiverKeys,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    req_id: Option<&[u8; 32]>,
) -> Result<bool, StealthError> {
    let r_pub_decoded = decode_r_pub(r_pub)?;
    let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub_decoded)?;
    let k_dh = match req_id {
        Some(value) => derive_k_dh_with_req(&dh, value),
        None => derive_k_dh(&dh),
    };
    let owner_expected = compute_owner_tag(&receiver_keys.owner_handle, &k_dh);
    Ok(tag_eq(&owner_expected, owner_tag))
}

fn tag_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    m1_owner_tag_check(left, right)
}

/// Construct a minimal unchecked stealth output descriptor.
///
/// Caller-side receiver validation is mandatory.
/// This builder does not verify `ReceiverCard` signatures and does not call
/// `ValidatePaymentRequest::validate_all()` internally. Callers must validate
/// the selected receiver input before invoking this function, and must not
/// treat the raw build path as an approved request/card acceptance decision.
pub fn build_tx_output_unchecked(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError> {
    let (output, _) = build_output_ctx(
        receiver_card,
        payment_request,
        sender_wallet,
        tx_digest,
        out_index,
        amount,
        asset_id,
    )?;

    Ok(output)
}

/// Construct a minimal unchecked stealth output descriptor for an explicit serial id.
///
/// This keeps runtime adapters on the same canonical raw sender seam while
/// preserving the explicit unchecked builder for
/// the lightweight `LIGHT_SERIAL_ID` lane.
/// Caller-side receiver validation is still mandatory.
pub fn build_tx_output_serial_unchecked(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<TxStealthOutput, StealthError> {
    if serial_id > SERIAL_MAX_ID {
        return Err(StealthError::InvalidStealthInput);
    }

    let (output, _) = build_output_ctx_with_serial(
        receiver_card,
        payment_request,
        sender_wallet,
        tx_digest,
        out_index,
        amount,
        asset_id,
        serial_id,
    )?;

    Ok(output)
}

/// Construct and strictly validate one lightweight stealth output from a bare
/// receiver card.
///
/// This is the accepted wallet-local approval constructor for card-only
/// Scenario 1 output building. It requires a matching stored receiver-card
/// entry with `TrustLevel::Pinned` in the wallet trust store and keeps that
/// wallet-local card approval distinct from both the raw builder path and the
/// request-capable validated path. It does not upgrade wallet-local approval
/// into a public trustless verifier claim.
pub fn build_card_stealth_output_validated<'a>(
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError> {
    approve_card(receiver_card, build_check)?;

    let (output, ctx) = build_output_ctx(
        receiver_card,
        None,
        sender_wallet,
        tx_digest,
        out_index,
        amount,
        asset_id,
    )?;
    validate_output_self(&output, &ctx, amount)?;
    Ok(output)
}

/// Construct and strictly validate one explicit-serial stealth output from a
/// bare receiver card.
///
/// This preserves the wallet-local pinned-card approval contract from
/// `build_card_stealth_output_validated(...)` while keeping explicit serial
/// callers off the raw `build_tx_output_serial_unchecked(...)` seam.
pub fn build_card_output_serial_checked<'a>(
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<TxStealthOutput, StealthError> {
    approve_card(receiver_card, build_check)?;

    let (output, ctx) = build_output_ctx_with_serial(
        receiver_card,
        None,
        sender_wallet,
        tx_digest,
        out_index,
        amount,
        asset_id,
        serial_id,
    )?;
    validate_output_self(&output, &ctx, amount)?;
    Ok(output)
}

/// Construct and strictly validate one lightweight stealth output.
///
/// This is the accepted-flow constructor for request-bound Scenario 1 output
/// building. When a payment request is supplied, it enforces explicit request
/// approval and route matching, but it does not upgrade those wallet checks
/// into a public trustless verifier claim. When `payment_request` is `None`,
/// this remains the compatibility self-check path; use
/// `build_card_stealth_output_validated(...)` for wallet-local card approval.
pub fn build_tx_stealth_output_validated<'a>(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError> {
    approve_req(payment_request, build_check)?;

    let (output, ctx) = build_output_ctx(
        receiver_card,
        payment_request,
        sender_wallet,
        tx_digest,
        out_index,
        amount,
        asset_id,
    )?;
    validate_output_self(&output, &ctx, amount)?;
    Ok(output)
}

/// Benchmark stealth output creation throughput.
#[cfg(feature = "test-params-fast")]
pub fn benchmark_stealth_output(
    receiver_card: &ReceiverCard,
    sender_wallet: &mut SenderWallet,
    iterations: usize,
) -> Result<Duration, StealthError> {
    if iterations == 0 {
        return Ok(Duration::ZERO);
    }

    let tx_digest = [0x06u8; 32];
    let asset_id = [0xEEu8; 32];
    let start = Instant::now();

    for index in 0..iterations {
        let output = build_tx_output_unchecked(
            receiver_card,
            None,
            sender_wallet,
            &tx_digest,
            index as u32,
            1_000,
            &asset_id,
        )?;
        std::hint::black_box(output);
    }

    Ok(start.elapsed())
}

#[cfg(test)]
#[path = "test_output.rs"]
mod tests;
