use super::{
    is_amount_in_range, Arc, Asset, AssetClass, AssetDefinition, AssetError, Commitment, Cow,
    Nonce, RangeProof, Z00ZRistrettoPoint, Z00ZScalar, MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};

impl Asset {
    /// 🔑 Get asset definition ID
    ///
    /// Returns the 32-byte identifier from the linked AssetDefinition.
    pub fn definition_id(&self) -> [u8; 32] {
        self.definition.id
    }

    /// Return decrypt associated-data id for stealth payload recovery.
    pub fn leaf_ad_id(&self) -> Result<[u8; 32], AssetError> {
        self.leaf_ad_id
            .ok_or(AssetError::InvalidStealth(Cow::Borrowed(
                "full stealth fields require leaf_ad_id",
            )))
    }

    /// 🔥 Check if burning is allowed via asset definition policy (flag bit 4)
    pub fn is_burnable(&self) -> bool {
        self.definition.is_burnable()
    }

    /// 🚩 Check if asset can be used for gas/transaction fees (flag bit 0)
    pub fn is_gas(&self) -> bool {
        self.definition.is_gas()
    }

    /// 🔄 Check if asset is fungible (flag bit 1)
    pub fn is_fungible(&self) -> bool {
        self.definition.is_fungible()
    }

    /// 🪙 Check if asset is mintable (flag bit 2)
    pub fn is_mintable(&self) -> bool {
        self.definition.is_mintable()
    }

    /// Check if this asset uses stealth fields.
    pub fn is_stealth(&self) -> bool {
        self.r_pub.is_some() && self.owner_tag.is_some() && self.enc_pack.is_some()
    }

    /// Check if this asset is transparent.
    pub fn is_transparent(&self) -> bool {
        !self.is_stealth()
    }

    /// Return payment type label.
    pub fn payment_type(&self) -> &'static str {
        if self.is_stealth() {
            "stealth"
        } else {
            "transparent"
        }
    }

    /// 🏗️ Create a new Asset with full validation and owner signature
    ///
    /// Creates a signed asset with automatic owner_signature generation.
    /// In Z00Z's offline-first architecture, every Asset MUST be signed by creator.
    ///
    /// # Arguments
    ///
    /// * `definition` - `Arc<AssetDefinition>` containing asset policy
    /// * `serial_id` - Series number (MUST be < definition.serials)
    /// * `amount` - Value in smallest units
    /// * `blinding` - Blinding factor (owner's secret key) for commitment AND signature
    /// * `nonce` - Unique nonce (MUST be globally unique)
    /// * `rng` - Cryptographic RNG for signature generation
    ///
    /// # Returns
    ///
    /// * `Ok(Asset)` with commitment, range_proof, AND owner_signature
    /// * `Err(AssetError)` if validation or signing fails
    ///
    /// # Validation
    ///
    /// - serial_id < definition.serials
    /// - Commitment created successfully
    /// - Range proof generated successfully
    /// - Owner signature generated and verified
    ///
    /// # Security
    ///
    /// - Blinding factor = owner's secret key (used for both commitment and signature)
    /// - Nonce MUST be globally unique (use derive_nonce())
    /// - Owner signature proves: "I created this asset with these exact parameters"
    /// - RNG MUST be cryptographically secure (OsRng recommended)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use z00z_core::assets::Asset;
    /// use z00z_core::assets::definition::AssetDefinition;
    /// use z00z_core::assets::{AssetClass, Z00ZScalar};
    /// use z00z_crypto::expert::{keys::RistrettoSecretKey, traits::SecretKeyTrait};
    /// use rand::rngs::OsRng;
    /// // Or: use z00z_utils::rng::{DeterministicRngProvider, RngProvider};
    /// use std::sync::Arc;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let def = AssetDefinition::new(
    ///     [0u8; 32], AssetClass::Coin, "Test".into(), "TST".into(),
    ///     8, 1000, 100_000_000, "test.io".into(), 1, 1, 0, None
    /// )?;
    ///
    /// let secret = Z00ZScalar::random(&mut OsRng).unwrap();
    /// let asset = Asset::new(
    ///     Arc::new(def),
    ///     100,
    ///     1_000_000,
    ///     &secret,
    ///     [42u8; 32],
    ///     &mut OsRng,
    /// )?;
    ///
    /// // Asset is automatically signed by creator
    /// assert!(asset.owner_pub.is_some());
    /// assert!(asset.owner_signature.is_some());
    /// assert!(asset.verify_owner_signature().is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        definition: Arc<AssetDefinition>,
        serial_id: u32,
        amount: u64,
        blinding: &Z00ZScalar,
        nonce: Nonce,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
    ) -> Result<Self, AssetError> {
        Self::new_prechecked(definition, serial_id, amount, blinding, nonce, rng, true)
    }

    pub(crate) fn new_prechecked(
        definition: Arc<AssetDefinition>,
        serial_id: u32,
        amount: u64,
        blinding: &Z00ZScalar,
        nonce: Nonce,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        check_id: bool,
    ) -> Result<Self, AssetError> {
        // Step 1: Validate construction parameters
        Self::validate_construction_params(&definition, serial_id, amount, check_id)?;

        // Step 2: Create cryptographic components (commitment + range proof)
        let (commitment, range_proof) = Self::create_crypto_components(amount, blinding)?;

        // Step 3: Generate owner public key from blinding factor (secret key)
        // CRITICAL: Set owner_pub BEFORE calling sign_owner() to enable secret key validation
        let owner_pub = Z00ZRistrettoPoint::from_scalar(blinding)?;

        // Step 4: Create Asset instance with owner_pub already set
        let mut asset = Self {
            definition,
            serial_id,
            amount,
            commitment,
            range_proof: Some(range_proof),
            nonce,
            lock_height: None,
            is_burned: false,
            owner_pub: Some(owner_pub), // Set BEFORE signing
            owner_signature: None,
            is_frozen: false,
            is_slashed: false,
            r_pub: None,
            owner_tag: None,
            enc_pack: None,
            secret: None,
            tag16: None,
            leaf_ad_id: None,
        };

        // Step 5: Sign the Asset with owner's secret key
        // Now sign_owner() can validate that secret matches owner_pub
        let owner_signature = asset.sign_owner(blinding, rng)?;
        asset.owner_signature = Some(owner_signature);

        Ok(asset)
    }

    /// Validate parameters for Asset construction
    ///
    /// Checks serial_id bounds and amount validity per asset class.
    /// Called by `new()` before creating cryptographic components.
    pub(super) fn validate_construction_params(
        definition: &AssetDefinition,
        serial_id: u32,
        amount: u64,
        check_id: bool,
    ) -> Result<(), AssetError> {
        if check_id {
            definition.validate()?;
        } else {
            definition.validate_fields()?;
        }

        // Validate serial_id bounds
        if serial_id >= definition.serials {
            return Err(AssetError::InvalidSerialIdStructured {
                definition_id: definition.id,
                serial_id,
                max_serials: definition.serials,
            });
        }

        // Validate amount
        // Zero amounts are allowed for:
        // - NFT assets (nominal value = 0, each NFT is unique)
        // - Void assets (burn sinks with no value)
        // - Burn transactions (amount burned to void)
        // Non-zero amounts required for Coins and Tokens
        //
        // CRITICAL: Check amount > 0 for Coin/Token BEFORE creating commitment
        // This prevents creating invalid assets that would fail validate() later
        //
        // NOTE: We do NOT enforce amount ≤ nominal here because:
        // 1. Nominal represents "typical" or "suggested" value, not hard max
        // 2. Some use cases need flexible amounts (e.g., staking rewards > nominal)
        // 3. Genesis validation already prevents overflow (serials × nominal < u64::MAX)
        // 4. Range proofs enforce cryptographic bounds (amount < 2^64)
        if amount == 0 && !matches!(definition.class, AssetClass::Nft | AssetClass::Void) {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "amount must be non-zero for Coin/Token assets",
            )));
        }

        if !is_amount_in_range(amount) {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "amount exceeds range-proof policy",
            )));
        }

        Ok(())
    }

    /// Create cryptographic components (commitment + range proof)
    ///
    /// Generates Pedersen commitment and Bulletproofs+ range proof using the
    /// universal z00z_crypto backend abstraction (currently Tari, but extensible).
    ///
    /// Uses cached BulletproofsPlusService for performance (~5ms per proof).
    ///
    /// # Security Note: Mimblewimble/Tari Standard
    ///
    /// The same `blinding` is used for BOTH:
    /// 1. Pedersen commitment: C = amount·G + blinding·H
    /// 2. Owner public key: owner_pub = blinding·G (set by caller)
    ///
    /// This is INTENTIONAL and follows Tari protocol design where:
    /// - Blinding factor serves as secret key for ownership
    /// - Same key proves knowledge of commitment opening
    /// - Enables efficient signature verification and asset spend proofs
    ///
    /// Security implications:
    /// - Standard Mimblewimble approach (used in Grin, Tari, etc.)
    /// - Binding between commitment and ownership is cryptographically enforced
    /// - Knowledge of blinding proves BOTH amount knowledge AND ownership
    /// - Do NOT reuse blinding across different Assets (breaks privacy)
    fn create_crypto_components(
        amount: u64,
        blinding: &Z00ZScalar,
    ) -> Result<(Commitment, RangeProof), AssetError> {
        // Create Pedersen commitment: C = amount·G + blinding·H
        // Uses z00z_crypto public API (backend hidden)
        let commitment = z00z_crypto::create_commitment(amount, blinding)?;

        // Create range proof (Bulletproofs+ for 64-bit range)
        // Generates a zero-knowledge proof that amount ∈ [0, 2^64) without revealing the actual value.
        // The proof is cryptographically bound to the commitment, ensuring:
        // - No negative amounts (prevents inflation attacks)
        // - No overflow beyond 2^64 (prevents protocol violations)
        // - Zero-knowledge property (amount remains confidential)
        //
        // Uses z00z_crypto public API which abstracts the backend (currently Tari)
        let range_proof =
            z00z_crypto::create_range_proof(amount, blinding, RANGE_PROOF_BITS, MIN_VALUE_PROMISE)?;

        Ok((commitment, range_proof))
    }
}
