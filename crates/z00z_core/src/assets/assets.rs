//! Asset structure with shared definitions and extracted validation or crypto seams.
//!
//! This root keeps the stable `assets::Asset` surface while sibling modules own
//! serde helpers, construction, validation, ownership, and cryptographic checks.

// ============================================================================
// Imports — Organized by convention
// ============================================================================

use z00z_crypto::{
    DomainHasher, Hidden, KernelSignature, RangeProof, Z00ZCommitment as Commitment,
    Z00ZRistrettoPoint, Z00ZScalar, ZkPackEncrypted, AGGREGATION_FACTOR, MIN_VALUE_PROMISE,
    RANGE_PROOF_BITS,
};
use z00z_utils::rng::SystemRngProvider;

// Standard library
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

// Internal modules
use super::amount::is_amount_in_range;
use super::definition::AssetDefinition;
use super::nonce::Nonce;

pub use super::{asset_class::AssetClass, asset_error::AssetError, asset_metadata::AssetMetadata};

#[path = "asset_arc_serde.rs"]
mod arc_serde;

// ============================================================================
// Asset State Structure
// ============================================================================

/// 💎 Asset struct representing a confidential asset
///
/// Asset is the core asset type in the Z00Z protocol. Each Asset represents
/// a spendable output with:
/// - Policy reference (`Arc<AssetDefinition>`) - shared immutable data
/// - Asset state (serial_id, amount, nonce) - unique per instance
/// - Cryptography (commitment, range_proof) - privacy layer
///
/// # Architecture
///
/// ```text
/// Asset (state instance)
///   ├─ definition: Arc<AssetDefinition>  ← Shared (8 bytes ref)
///   ├─ serial_id: u32                    ← Which series (1 per asset)
///   ├─ amount: u64                       ← Value (hidden by commitment)
///   ├─ commitment: PedersenCommitment    ← C = amount·G + blinding·H
///   ├─ range_proof: RangeProof           ← Bulletproof+ proof
///   ├─ nonce: [u8; 32]                   ← Unique per output
///   └─ Additional fields (lock_height, owner, etc.)
/// ```
///
/// # Memory Efficiency
///
/// For N assets with M unique definitions:
/// - Old: N × ~200 bytes (full AssetDefinition in each)
/// - New: M × 200 + N × (8 + sizeof(asset fields)) ≈ M × 200 + N × 120 bytes
///
/// Example: 10,000 assets, 10 definitions:
/// - Old: 2,000,000 bytes
/// - New: 2,000 + 1,200,000 = 1,202,000 bytes (~40% savings)
///
/// # Security Model
///
/// - **Commitment hiding:** `C = amount·G + blinding·H` (Pedersen commitment)
/// - **Range proof:** Bulletproof+ proves `0 ≤ amount < 2^64`
/// - **Nonce uniqueness:** Each Asset has unique nonce (no linkability)
/// - **Serial tracking:** serial_id tracks which genesis series
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::Asset;
/// use z00z_core::assets::definition::AssetDefinition;
/// use z00z_core::assets::{AssetClass, Z00ZScalar};
/// use z00z_crypto::expert::{keys::RistrettoSecretKey, traits::SecretKeyTrait};
/// use rand::rngs::OsRng;
/// // Or for deterministic/reproducible tests:
/// // use z00z_utils::rng::{DeterministicRngProvider, RngProvider};
/// use std::sync::Arc;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create definition
/// let def = AssetDefinition::new(
///     [0u8; 32], AssetClass::Coin, "Z00Z".into(), "Z00Z".into(),
///     8, 50_000, 100_000_000, "z00z.io".into(), 1, 1, 0b0001_0000, None
/// )?;
/// let arc_def = Arc::new(def);
///
/// // Create Asset
/// let blinding = Z00ZScalar::random(&mut OsRng).unwrap();
/// // For deterministic tests: let mut rng = DeterministicRngProvider::from_seed([42u8; 32]).rng();
/// // Production: use derive_nonce_simple(&wallet_seed, counter, &time_provider)
/// // Testing: use [1u8; 32] or higher for non-zero placeholder
/// let nonce = [42u8; 32];  // Non-zero nonce (for testing)
/// let asset = Asset::new(
///     arc_def,
///     100,    // serial_id
///     1_000_000,  // amount
///     &blinding,
///     nonce,
///     &mut OsRng,  // RNG for range proof (or &mut rng for deterministic)
/// )?;
///
/// assert_eq!(asset.serial_id(), 100);
/// assert_eq!(asset.amount(), 1_000_000);
/// assert!(asset.range_proof().is_some());
/// # Ok(())
/// # }
/// ```
///
/// # Security Note: No Debug Implementation
///
/// Asset deliberately does NOT derive Debug to prevent accidental logging of sensitive data:
/// - `amount` - Confidential transaction value
/// - `blinding` - Cryptographic secret (if stored internally)
/// - Private keys or signing material
///
/// For debugging, use accessor methods that return non-sensitive information:
/// - `asset_id()` - Safe to log
/// - `serial_id()` - Safe to log
/// - `definition.name` - Safe to log
///
/// Never log the entire Asset struct in production.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    /// 📋 Reference to immutable asset definition (policy)
    ///
    /// Shared across multiple Asset instances for memory efficiency.
    /// Contains: id, class, name, symbol, decimals, serials, nominal, etc.
    #[serde(with = "arc_serde")]
    pub definition: Arc<AssetDefinition>,

    /// 🔢 Base serial — assigned at asset creation and MUST NOT change.
    ///
    /// For coins: index into genesis series (0..serials); identifies the "banknote".
    /// For tokens: typically 0 or 1.
    /// For NFTs: unique identifier within the asset class.
    ///
    /// INVARIANT: `0 ≤ serial_id < definition.serials` — enforced at construction.
    /// All fragments (outputs) produced by splitting this asset keep this `serial_id`.
    /// A payment may combine fragments from multiple `serial_id` values; conservation
    /// of supply is verifiable by summing plaintext values over all fragments sharing
    /// the same `serial_id`.
    ///
    /// ⚠️ Do NOT mutate this field after construction — use `serial_id()` getter for reads.
    pub serial_id: u32,

    /// 💰 Amount in smallest units (hidden by commitment)
    ///
    /// For coins: amount in satoshi-equivalent units
    /// For tokens: token count
    /// For NFTs: typically 1
    ///
    /// Range proof ensures: 0 ≤ amount < 2^64
    pub amount: u64,

    /// 🔐 Pedersen commitment C = amount·G + blinding·H
    ///
    /// Hides amount while enabling homomorphic operations.
    /// Validator checks: Σ inputs = Σ outputs (commitment balance).
    pub commitment: Commitment,

    /// 📜 Bulletproof+ range proof
    ///
    /// Proves amount ∈ [0, 2^64) without revealing actual value.
    /// MUST always be Some() in production (None only for testing).
    pub range_proof: Option<RangeProof>,

    /// 🎲 Unique nonce for privacy
    ///
    /// Generated via derive_nonce(wallet_seed, counter, prev_hash).
    /// If wallet_seed not provided, uses DefaultWalletSeedDomain.
    /// Derived deterministically from wallet seed + counter.
    /// Prevents linkability across transactions.
    /// MUST be unique per Asset instance.
    pub nonce: Nonce,

    /// 🔒 Lock height (optional timelock)
    ///
    /// Output cannot be spent until blockchain height ≥ lock_height.
    /// None = spendable immediately.
    pub lock_height: Option<u64>,

    /// 👤 Owner public key (optional, for offline payments)
    ///
    /// Identifies the intended recipient without revealing on-chain.
    /// Used in peer-to-peer transfer protocols.
    pub owner_pub: Option<Z00ZRistrettoPoint>,

    /// ✍️ Owner signature (optional, for offline payments)
    ///
    /// Schnorr signature proving ownership of the entire asset state.
    /// Signs: definition_id, serial_id, amount, commitment, nonce, lock_height,
    /// range_proof bytes, and ordered state flags (is_burned, is_frozen, is_slashed).
    /// Prevents address substitution and partial asset modification attacks.
    pub owner_signature: Option<KernelSignature>,

    /// ❄️ Frozen flag (MUTABLE per-asset state, Bit 3)
    ///
    /// If true, THIS ASSET is frozen and cannot be spent.
    /// Used for staking locks, temporary freezes, or protocol locks.
    /// This is STATE - frozen locally at asset-instance level, not a class policy.
    pub is_frozen: bool,

    /// ⚔️ Slashed flag (MUTABLE per-asset state, Bit 5)
    ///
    /// If true, THIS ASSET has been slashed (penalty applied).
    /// Used in staking/validation systems for validator penalties.
    /// This is STATE - individual asset can be slashed, not the whole class.
    pub is_slashed: bool,

    /// 🔥 Burned flag (MUTABLE per-asset state)
    ///
    /// If true, this output is being burned (sent to burn address).
    /// Validator checks: definition.is_burnable() == true (policy flag bit 4).
    /// This is STATE, not POLICY - can be true even if burn is allowed.
    pub is_burned: bool,

    /// Ephemeral public point for stealth payments.
    pub r_pub: Option<[u8; 32]>,

    /// Ownership tag for stealth outputs.
    pub owner_tag: Option<[u8; 32]>,

    /// Encrypted payload for stealth metadata.
    pub enc_pack: Option<ZkPackEncrypted>,

    /// Raw stealth output secret bytes (`s_out`).
    pub secret: Option<[u8; 32]>,

    /// Optional scan hint for stealth filtering.
    pub tag16: Option<u16>,

    /// Optional decrypt associated-data id for stealth wire/runtime parity.
    pub leaf_ad_id: Option<[u8; 32]>,
}

/// Custom Debug implementation that redacts sensitive cryptographic data
///
/// # Security
///
/// Prevents accidental exposure of confidential information in logs/error messages:
/// - `amount`: Redacted (confidential transaction value)
/// - `commitment`: Redacted (reveals amount if combined with blinding factor)
/// - `range_proof`: Shows size only, not proof data
/// - `owner_pub`: Redacted (user identity/privacy)
/// - `owner_signature`: Shows presence only, not signature data
///
/// Safe to display:
/// - `definition_id`: Public policy identifier
/// - `serial_id`: Non-sensitive sequence number
/// - Flags: Public state (burned, frozen, slashed)
impl fmt::Debug for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("Asset");

        // Safe fields - public identifiers
        debug_struct
            .field("definition_id", &hex::encode(self.definition.id))
            .field("serial_id", &self.serial_id);

        // Sensitive fields - redacted
        debug_struct
            .field("amount", &"<redacted>")
            .field("commitment", &"<redacted>");

        // Range proof - show size only
        match &self.range_proof {
            Some(proof) => {
                debug_struct.field("range_proof", &format!("<present, {} bytes>", proof.len()));
            }
            None => {
                debug_struct.field("range_proof", &"<none>");
            }
        }

        debug_struct.field("nonce", &"<redacted>");

        // Lock height - safe
        debug_struct.field("lock_height", &self.lock_height);

        // Owner public key - redacted (privacy)
        match &self.owner_pub {
            Some(_) => {
                debug_struct.field("owner_pub", &"<present>");
            }
            None => {
                debug_struct.field("owner_pub", &"<none>");
            }
        }

        // Owner signature - show presence only
        match &self.owner_signature {
            Some(_) => {
                debug_struct.field("owner_signature", &"<present>");
            }
            None => {
                debug_struct.field("owner_signature", &"<none>");
            }
        }

        // State flags - safe (public state)
        debug_struct
            .field("is_burned", &self.is_burned)
            .field("is_frozen", &self.is_frozen)
            .field("is_slashed", &self.is_slashed);

        match &self.r_pub {
            Some(_) => {
                debug_struct.field("r_pub", &"<present, 32 bytes>");
            }
            None => {
                debug_struct.field("r_pub", &"<none>");
            }
        }

        match &self.owner_tag {
            Some(_) => {
                debug_struct.field("owner_tag", &"<present, 32 bytes>");
            }
            None => {
                debug_struct.field("owner_tag", &"<none>");
            }
        }

        match &self.enc_pack {
            Some(data) => {
                debug_struct.field(
                    "enc_pack",
                    &format!("<encrypted, {} bytes>", data.ciphertext.len()),
                );
            }
            None => {
                debug_struct.field("enc_pack", &"<none>");
            }
        }

        match &self.secret {
            Some(_) => {
                debug_struct.field("secret", &"<present, 32 bytes>");
            }
            None => {
                debug_struct.field("secret", &"<none>");
            }
        }

        match &self.tag16 {
            Some(_) => {
                debug_struct.field("tag16", &"<present>");
            }
            None => {
                debug_struct.field("tag16", &"<none>");
            }
        }

        match &self.leaf_ad_id {
            Some(_) => {
                debug_struct.field("leaf_ad_id", &"<present, 32 bytes>");
            }
            None => {
                debug_struct.field("leaf_ad_id", &"<none>");
            }
        }

        debug_struct.finish()
    }
}

#[path = "asset_construction.rs"]
mod asset_construction;
#[path = "asset_crypto.rs"]
mod asset_crypto;

#[cfg(test)]
#[path = "test_asset.rs"]
mod tests;
