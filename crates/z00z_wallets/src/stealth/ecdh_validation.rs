use z00z_crypto::{Z00ZRistrettoPoint, Z00ZScalar};

use super::{compute_dh_sender, dh_eq_ct};
use crate::stealth::StealthError;

/// Verify one-way function constraints for ECDH security.
pub fn owf_constraints_ecdh(
    r: &Z00ZScalar,
    view_pk: &Z00ZRistrettoPoint,
    dh: &[u8; 32],
) -> Result<(), StealthError> {
    let computed_dh = compute_dh_sender(r, view_pk)?;
    if !dh_eq_ct(&computed_dh, dh) {
        return Err(StealthError::EcdhConstraintViolation);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::owf_constraints_ecdh;
    use crate::stealth::ecdh::compute_dh_sender;
    use z00z_crypto::{Z00ZRistrettoPoint, Z00ZScalar};
    use z00z_utils::rng::SystemRngProvider;

    fn key_pair() -> (Z00ZScalar, Z00ZRistrettoPoint) {
        let provider = SystemRngProvider;
        let mut rng = provider.rng();
        let sk = Z00ZScalar::random(&mut rng).unwrap();
        let pk = Z00ZRistrettoPoint::from_secret_key(&sk);
        (sk, pk)
    }

    #[test]
    fn test_ecdh_constraints_valid() {
        let (r, _) = key_pair();
        let (_, view_pk) = key_pair();
        let dh = compute_dh_sender(&r, &view_pk).expect("dh");

        let result = owf_constraints_ecdh(&r, &view_pk, &dh);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ecdh_constraints_invalid() {
        let (r, _) = key_pair();
        let (_, view_pk) = key_pair();
        let mut bad_dh = compute_dh_sender(&r, &view_pk).expect("dh");
        bad_dh[0] ^= 1;

        let result = owf_constraints_ecdh(&r, &view_pk, &bad_dh);
        assert!(result.is_err());
    }
}
