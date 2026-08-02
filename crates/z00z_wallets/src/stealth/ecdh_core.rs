//! Transitional wallet-runtime stealth ECDH owner.
//!
//! Phase 1 freeze keeps byte-oriented `compute_dh_*`, byte base/request `k_dh`,
//! and `derive_s_out` here until later convergence proves a smaller owner set.

use subtle::ConstantTimeEq;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::{domains::DhKeyDomain, hash_zk::hash_zk, Z00ZRistrettoPoint, Z00ZScalar};

use crate::domains::SOutProdDomain;

use crate::stealth::StealthError;

/// Sender-derived ECDH keys bundle.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EcdhSenderKeys {
    /// Shared ECDH bytes.
    pub dh: [u8; 32],
    /// Derived DH key for payload operations.
    pub k_dh: [u8; 32],
}

/// Receiver-derived ECDH keys bundle.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EcdhReceiverKeys {
    /// Shared ECDH bytes.
    pub dh: [u8; 32],
    /// Derived DH key for payload operations.
    pub k_dh: [u8; 32],
}

/// Sender side: `dh = r * view_pk`.
pub fn compute_dh_sender(
    r: &Z00ZScalar,
    receiver_view_pk: &Z00ZRistrettoPoint,
) -> Result<[u8; 32], StealthError> {
    compute_dh_inner(r, receiver_view_pk)
}

/// Receiver side: `dh = view_sk * R_pub`.
pub fn compute_dh_receiver(
    view_sk: &Z00ZScalar,
    r_pub: &Z00ZRistrettoPoint,
) -> Result<[u8; 32], StealthError> {
    compute_dh_inner(view_sk, r_pub)
}

/// Derive `k_dh` from a shared secret.
pub fn derive_k_dh(dh: &[u8; 32]) -> [u8; 32] {
    hash_zk::<DhKeyDomain>("", &[dh])
}

/// Derive `k_dh` with request binding.
pub fn derive_k_dh_with_req(dh: &[u8; 32], req_id: &[u8; 32]) -> [u8; 32] {
    hash_zk::<DhKeyDomain>("", &[dh, req_id])
}

/// Derive sender-side ECDH bundle.
#[cfg(test)]
pub fn sender_derive_keys(
    r: &Z00ZScalar,
    view_pk: &Z00ZRistrettoPoint,
    req_id: Option<&[u8; 32]>,
) -> Result<EcdhSenderKeys, StealthError> {
    let dh = compute_dh_sender(r, view_pk)?;
    let k_dh = match req_id {
        Some(req) => derive_k_dh_with_req(&dh, req),
        None => derive_k_dh(&dh),
    };

    Ok(EcdhSenderKeys { dh, k_dh })
}

/// Derive receiver-side ECDH bundle.
#[cfg(test)]
pub fn receiver_derive_keys(
    view_sk: &Z00ZScalar,
    r_pub: &Z00ZRistrettoPoint,
    req_id: Option<&[u8; 32]>,
) -> Result<EcdhReceiverKeys, StealthError> {
    let dh = compute_dh_receiver(view_sk, r_pub)?;
    let k_dh = match req_id {
        Some(req) => derive_k_dh_with_req(&dh, req),
        None => derive_k_dh(&dh),
    };

    Ok(EcdhReceiverKeys { dh, k_dh })
}

/// Derive stealth output secret `s_out` from DH key, ephemeral public key, and serial.
///
/// Canonical formula: `H<SOutProdDomain>("Z00Z/S_OUT", k_dh || r_pub || serial_le)`.
/// Serial ID MUST be LE-encoded (4 bytes) to match the OWF circuit.
/// `s_out` is the sender-known output secret on this path, not Bob's separate `receiver_secret`.
/// This is the single canonical implementation — do not duplicate in other modules.
pub fn derive_s_out(k_dh: &[u8; 32], r_pub: &[u8; 32], serial_id: u32) -> [u8; 32] {
    let serial = serial_id.to_le_bytes();
    hash_zk::<SOutProdDomain>("Z00Z/S_OUT", &[k_dh, r_pub, &serial])
}

/// Constant-time equality check for DH byte arrays.
pub fn dh_eq_ct(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.ct_eq(right).into()
}

fn compute_dh_inner(sk: &Z00ZScalar, pk: &Z00ZRistrettoPoint) -> Result<[u8; 32], StealthError> {
    if sk.as_bytes() == [0u8; 32] {
        return Err(StealthError::ZeroScalarRejected);
    }

    if pk.as_bytes() == [0u8; 32] {
        return Err(StealthError::IdentityPointRejected);
    }

    let shared = pk.reveal() * sk.reveal();
    let bytes = shared.as_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(bytes);

    if out == [0u8; 32] {
        return Err(StealthError::EcdhIdentityResult);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{
        compute_dh_receiver, compute_dh_sender, derive_k_dh, derive_k_dh_with_req, dh_eq_ct,
        receiver_derive_keys, sender_derive_keys,
    };
    use z00z_crypto::{Z00ZRistrettoPoint, Z00ZScalar};
    use z00z_utils::rng::SystemRngProvider;

    fn derive_pack_key_with_req(dh: &[u8; 32], req_id: &[u8; 32]) -> [u8; 32] {
        derive_k_dh_with_req(dh, req_id)
    }

    fn key_pair() -> (Z00ZScalar, Z00ZRistrettoPoint) {
        let provider = SystemRngProvider;
        let mut rng = provider.rng();
        let sk = Z00ZScalar::random(&mut rng).unwrap();
        let pk = Z00ZRistrettoPoint::from_secret_key(&sk);
        (sk, pk)
    }

    #[test]
    fn test_ecdh_commutativity() {
        let (sender_r, sender_r_pub) = key_pair();
        let (view_sk, view_pk) = key_pair();

        let left = compute_dh_sender(&sender_r, &view_pk).expect("left");
        let right = compute_dh_receiver(&view_sk, &sender_r_pub).expect("right");
        assert_eq!(left, right);
    }

    #[test]
    fn test_ecdh_different_different_dh() {
        let (r1, _) = key_pair();
        let (r2, _) = key_pair();
        let (_, view_pk) = key_pair();

        let dh1 = compute_dh_sender(&r1, &view_pk).expect("dh1");
        let dh2 = compute_dh_sender(&r2, &view_pk).expect("dh2");
        assert_ne!(dh1, dh2);
    }

    #[test]
    fn test_k_dh_derivation_deterministic() {
        let dh = [21u8; 32];
        let a = derive_k_dh(&dh);
        let b = derive_k_dh(&dh);
        assert_eq!(a, b);
    }

    #[test]
    fn test_dh_req_id_different() {
        let dh = [22u8; 32];
        let req_a = [1u8; 32];
        let req_b = [2u8; 32];

        let a = derive_k_dh_with_req(&dh, &req_a);
        let b = derive_k_dh_with_req(&dh, &req_b);
        assert_ne!(a, b);
    }

    #[test]
    fn test_eq_pack_key() {
        let dh = [23u8; 32];
        let req_id = [3u8; 32];

        let k_dh = derive_k_dh_with_req(&dh, &req_id);
        let pack_key = derive_pack_key_with_req(&dh, &req_id);
        assert_eq!(k_dh, pack_key);
    }

    #[test]
    fn test_ecdh_constant_time() {
        let dh = [7u8; 32];
        let same = dh;
        let mut other = dh;
        other[0] ^= 1;

        assert!(dh_eq_ct(&dh, &same));
        assert!(!dh_eq_ct(&dh, &other));
    }

    #[test]
    fn test_sender_derive_keys() {
        let (sender_r, _) = key_pair();
        let (_, view_pk) = key_pair();
        let req_id = [5u8; 32];

        let sender = sender_derive_keys(&sender_r, &view_pk, Some(&req_id)).expect("sender");
        let expected_dh = compute_dh_sender(&sender_r, &view_pk).expect("expected_dh");

        assert_eq!(sender.dh, expected_dh);
        assert_eq!(sender.k_dh, derive_k_dh_with_req(&sender.dh, &req_id));
    }

    #[test]
    fn test_receiver_derive_keys() {
        let (sender_r, sender_r_pub) = key_pair();
        let (view_sk, view_pk) = key_pair();
        let req_id = [6u8; 32];

        let sender = sender_derive_keys(&sender_r, &view_pk, Some(&req_id)).expect("sender");
        let receiver =
            receiver_derive_keys(&view_sk, &sender_r_pub, Some(&req_id)).expect("receiver");

        assert_eq!(sender.dh, receiver.dh);
        assert_eq!(sender.k_dh, receiver.k_dh);
    }
}
