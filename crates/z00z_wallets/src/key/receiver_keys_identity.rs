/// Derives the versioned identity signing key from the receiver secret.
pub fn derive_identity_secret_key(
    receiver_secret: &ReceiverSecret,
    version: u32,
) -> Result<Z00ZScalar, StealthKeyError> {
    let version_bytes = version.to_le_bytes();
    let key = hash_to_scalar_zk::<WalletIdentityKeyHashProdDomain>(
        "IDENTITY",
        &[receiver_secret.as_bytes(), &version_bytes],
    )
    .map_err(|_| StealthKeyError::InvalidSecretKey)?;

    if key.as_bytes() == [0u8; 32] {
        return Err(StealthKeyError::ZeroScalarRejected);
    }

    Ok(key)
}

/// Derives the public identity key corresponding to a secret signing scalar.
pub fn derive_identity_public_key(
    identity_sk: &Z00ZScalar,
) -> Result<Z00ZRistrettoPoint, StealthKeyError> {
    let key = Z00ZRistrettoPoint::from_secret_key(identity_sk);
    if key.as_bytes() == [0u8; 32] {
        return Err(StealthKeyError::IdentityPointRejected);
    }
    Ok(key)
}

/// Generates a random one-off identity keypair.
pub fn generate_identity_keypair() -> Result<(Z00ZScalar, Z00ZRistrettoPoint), StealthKeyError> {
    #[cfg(test)]
    ID_GEN_COUNT.fetch_add(1, Ordering::Relaxed);

    let sk = Z00ZScalar::random_secure(&SystemRngProvider)
        .map_err(|_| StealthKeyError::InvalidSecretKey)?;
    let pk = Z00ZRistrettoPoint::from_secret_key(&sk);
    Ok((sk, pk))
}

#[cfg(test)]
fn id_gen_count() -> u64 {
    ID_GEN_COUNT.load(Ordering::Relaxed)
}

#[cfg(test)]
fn reset_id_gen_count() {
    ID_GEN_COUNT.store(0, Ordering::Relaxed);
}

/// Signs identity-bound metadata with the derived identity secret key.
pub fn sign_identity_with_rng<R>(
    identity_sk: &Z00ZScalar,
    message: &[u8],
    context: &[u8],
    rng: &mut R,
) -> Result<Z00ZSchnorrSignature, StealthKeyError>
where
    R: rand::CryptoRng + rand::RngCore,
{
    let payload = identity_payload(message, context);
    sign_kernel_signature(identity_sk, &payload, rng).map_err(|_| StealthKeyError::SignatureFailed)
}

/// Signs identity-bound metadata with the derived identity secret key.
pub fn sign_identity(
    identity_sk: &Z00ZScalar,
    message: &[u8],
    context: &[u8],
) -> Result<Z00ZSchnorrSignature, StealthKeyError> {
    let payload = identity_payload(message, context);
    let mut rng = SystemRngProvider.rng();
    sign_kernel_signature(identity_sk, &payload, &mut rng)
        .map_err(|_| StealthKeyError::SignatureFailed)
}

/// Verifies an identity signature for the supplied message and context.
pub fn verify_identity(
    identity_pk: &Z00ZRistrettoPoint,
    message: &[u8],
    context: &[u8],
    signature: &Z00ZSchnorrSignature,
) -> Result<(), StealthKeyError> {
    let payload = identity_payload(message, context);
    if verify_kernel_signature(signature, identity_pk, &payload) {
        return Ok(());
    }
    Err(StealthKeyError::SignatureVerifyFailed)
}
