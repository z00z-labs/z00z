use z00z_crypto::domains::AssetIdHashDomain;

use super::{
    Arc, Asset, AssetDefinition, AssetError, Commitment, DomainHasher, Hidden, Nonce, RangeProof,
    SystemRngProvider, Z00ZScalar, AGGREGATION_FACTOR, MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};

impl Asset {
    pub fn new_confidential(
        definition: Arc<AssetDefinition>,
        serial_id: u32,
        amount: u64,
        nonce: [u8; 32],
    ) -> Result<(Self, Hidden<Z00ZScalar>), AssetError> {
        let blinding = Hidden::hide(
            Z00ZScalar::random_secure(&SystemRngProvider).map_err(AssetError::CryptoError)?,
        );
        let asset = Self::new_confidential_with_blinding(
            definition,
            serial_id,
            amount,
            nonce,
            blinding.reveal(),
        )?;

        Ok((asset, blinding))
    }

    pub fn new_confidential_with_blinding(
        definition: Arc<AssetDefinition>,
        serial_id: u32,
        amount: u64,
        nonce: [u8; 32],
        blinding: &Z00ZScalar,
    ) -> Result<Self, AssetError> {
        Self::validate_construction_params(&definition, serial_id, amount, true)?;

        let commitment =
            z00z_crypto::create_commitment(amount, blinding).map_err(AssetError::CryptoError)?;
        let range_proof =
            z00z_crypto::create_range_proof(amount, blinding, RANGE_PROOF_BITS, MIN_VALUE_PROMISE)
                .map_err(|source| AssetError::RangeProofCreation { source })?;

        let asset = Self {
            definition,
            serial_id,
            amount,
            commitment,
            range_proof: Some(range_proof),
            nonce,
            lock_height: None,
            owner_pub: None,
            owner_signature: None,
            is_frozen: false,
            is_slashed: false,
            is_burned: false,
            r_pub: None,
            owner_tag: None,
            enc_pack: None,
            secret: None,
            tag16: None,
            leaf_ad_id: None,
        };

        Ok(asset)
    }

    pub fn verify_commitment_opening(&self, blinding: &Z00ZScalar) -> Result<(), AssetError> {
        let recomputed = z00z_crypto::create_commitment(self.amount, blinding)
            .map_err(AssetError::CryptoError)?;

        if self.commitment != recomputed {
            let mut expected = [0u8; 32];
            expected.copy_from_slice(self.commitment.as_bytes());
            let mut got = [0u8; 32];
            got.copy_from_slice(recomputed.as_bytes());
            return Err(AssetError::CommitmentMismatch { expected, got });
        }

        Ok(())
    }

    pub fn verify_range_proof(&self) -> Result<(), AssetError> {
        let proof = self
            .range_proof
            .as_ref()
            .ok_or(AssetError::MissingRangeProof)?;

        z00z_crypto::verify_range_proof(
            proof,
            &self.commitment,
            RANGE_PROOF_BITS,
            AGGREGATION_FACTOR,
            MIN_VALUE_PROMISE,
        )
        .map_err(|source| AssetError::RangeProofVerification { source })?;

        Ok(())
    }

    pub fn validate_confidential(&self) -> Result<(), AssetError> {
        self.verify_range_proof()
    }

    /// 🔑 Calculate unique asset_id for this asset
    ///
    /// Deterministically derives asset_id from:
    /// - domain ("z00z.core.assets.asset_id.v1")
    /// - nonce (unique per Asset)
    /// - commitment (hides amount)
    /// - definition.id (asset type)
    /// - serial_id (series number)
    ///
    /// This ensures each Asset has a globally unique ID for spend tracking.
    ///
    /// # Returns
    ///
    /// 32-byte asset_id hash
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_core::assets::Asset;
    /// # use z00z_core::assets::definition::AssetDefinition;
    /// # use z00z_core::assets::{AssetClass, Z00ZScalar};
    /// # use z00z_crypto::expert::{keys::RistrettoSecretKey, traits::SecretKeyTrait};
    /// # use rand::rngs::OsRng;
    /// # use std::sync::Arc;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let def = AssetDefinition::new(
    /// #     [0u8; 32], AssetClass::Coin, "Test".into(), "TST".into(),
    /// #     8, 1000, 100_000_000, "test.io".into(), 1, 1, 0, None
    /// # )?;
    /// # let asset = Asset::new(
    /// #     Arc::new(def), 100, 1_000_000, &Z00ZScalar::random(&mut OsRng).unwrap(), [42u8; 32], &mut OsRng
    /// # )?;
    /// let asset_id = asset.asset_id();
    /// assert_eq!(asset_id.len(), 32);
    /// # Ok(())
    /// # }
    /// ```
    pub fn asset_id(&self) -> [u8; 32] {
        let hash = DomainHasher::<AssetIdHashDomain>::new_with_label("asset_id")
            .chain(self.nonce)
            .chain(self.commitment.as_bytes())
            .chain(&self.definition.id)
            .chain(self.serial_id.to_le_bytes())
            .finalize();

        let mut id = [0u8; 32];
        id.copy_from_slice(&hash.as_ref()[..32]);
        id
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Get asset definition
    pub fn definition(&self) -> &Arc<AssetDefinition> {
        &self.definition
    }

    /// Get serial_id
    pub fn serial_id(&self) -> u32 {
        self.serial_id
    }

    /// Get amount
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get commitment
    pub fn commitment(&self) -> &Commitment {
        &self.commitment
    }

    /// Get range proof
    pub fn range_proof(&self) -> &Option<RangeProof> {
        &self.range_proof
    }

    /// Get nonce
    pub fn nonce(&self) -> &Nonce {
        &self.nonce
    }

    /// Get lock height
    pub fn lock_height(&self) -> Option<u64> {
        self.lock_height
    }

    /// Check if output is burned
    pub fn is_burned(&self) -> bool {
        self.is_burned
    }

    /// Check if output is locked (timelock not yet expired)
    pub fn is_locked(&self, current_height: u64) -> bool {
        self.lock_height
            .is_some_and(|height| current_height < height)
    }
}
