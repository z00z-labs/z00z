//! Algebra and commitment balance helpers.

use z00z_crypto::{Z00ZCommitment, Z00ZScalar};

/// Transaction commitment balance diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TxBalErr {
    /// Input/output sums mismatch even before fee accounting.
    CommitMismatch,
    /// Auxiliary metadata commitment must be zero-valued.
    MetaMismatch,
}

/// Compute change blinding so that input and output blindings stay balanced.
pub fn balance_blindings(r_in: &Z00ZScalar, r_outs: &[Z00ZScalar]) -> Z00ZScalar {
    r_outs
        .iter()
        .fold(r_in.dangerous_clone(), |acc, item| &acc - item)
}

/// Verify that sum of input blindings equals sum of output blindings.
pub fn verify_blind_balance(r_ins: &[Z00ZScalar], r_outs: &[Z00ZScalar]) -> bool {
    match (r_ins.is_empty(), r_outs.is_empty()) {
        (true, true) => true,
        (true, false) | (false, true) => false,
        (false, false) => {
            let sum_in = r_ins
                .iter()
                .skip(1)
                .fold(r_ins[0].dangerous_clone(), |acc, item| &acc + item);
            let sum_out = r_outs
                .iter()
                .skip(1)
                .fold(r_outs[0].dangerous_clone(), |acc, item| &acc + item);
            sum_in.ct_eq(&sum_out)
        }
    }
}

/// Verify homomorphic commitment balance for transaction inputs and outputs.
pub fn verify_tx_balance(inputs: &[Z00ZCommitment], outputs: &[Z00ZCommitment]) -> bool {
    match (inputs.is_empty(), outputs.is_empty()) {
        (true, true) => true,
        (true, false) | (false, true) => false,
        (false, false) => {
            let sum_in = sum_commitments(inputs);
            let sum_out = sum_commitments(outputs);
            sum_in == sum_out
        }
    }
}

/// Verify transaction balance while treating the extra commitment as metadata only.
pub fn verify_tx_balance_meta(
    inputs: &[Z00ZCommitment],
    outputs: &[Z00ZCommitment],
    meta: &Z00ZCommitment,
) -> Result<(), TxBalErr> {
    let zero = &meta.clone() - meta;
    if &zero != meta {
        return Err(TxBalErr::MetaMismatch);
    }

    if verify_tx_balance(inputs, outputs) {
        Ok(())
    } else {
        Err(TxBalErr::CommitMismatch)
    }
}

fn sum_commitments(items: &[Z00ZCommitment]) -> Z00ZCommitment {
    items
        .iter()
        .skip(1)
        .fold(items[0].clone(), |acc, item| &acc + item)
}

#[cfg(test)]
mod tests {
    use super::{
        balance_blindings, verify_blind_balance, verify_tx_balance, verify_tx_balance_meta,
        TxBalErr,
    };
    use z00z_crypto::expert::traits::SecretKeyTrait;
    use z00z_crypto::{create_commitment, Z00ZCommitment, Z00ZScalar};
    use z00z_utils::rng::SystemRngProvider;

    fn key_size(key: &impl SecretKeyTrait) -> usize {
        key.as_bytes().len()
    }

    fn random_scalar() -> Z00ZScalar {
        let mut rng = SystemRngProvider.rng();
        Z00ZScalar::random(&mut rng).unwrap()
    }

    fn commit(value: u64, blinding_seed: u64) -> Z00ZCommitment {
        let mut scalar_bytes = [0u8; 32];
        scalar_bytes[..8].copy_from_slice(&blinding_seed.to_le_bytes());
        let blinding = Z00ZScalar::try_from_bytes(scalar_bytes).expect("valid scalar");
        create_commitment(value, &blinding).expect("valid commitment")
    }

    fn blind(seed: u64) -> Z00ZScalar {
        let mut scalar_bytes = [0u8; 32];
        scalar_bytes[..8].copy_from_slice(&seed.to_le_bytes());
        Z00ZScalar::try_from_bytes(scalar_bytes).expect("valid scalar")
    }

    #[test]
    fn test_blinding_add_commutative() {
        let first = random_scalar();
        let second = random_scalar();

        let sum_ab = &first + &second;
        let sum_ba = &second + &first;

        assert!(sum_ab.ct_eq(&sum_ba));
    }

    #[test]
    fn test_blinding_sub() {
        let first = random_scalar();
        let second = random_scalar();

        let sum = &first + &second;
        let diff = &sum - &second;

        assert!(diff.ct_eq(&first));
    }

    #[test]
    fn test_tx_balance_blindings() {
        let input = random_scalar();
        let payment = random_scalar();
        let payment_copy = payment.dangerous_clone();

        let change = balance_blindings(&input, &[payment_copy]);
        let restored = &payment + &change;

        assert!(restored.ct_eq(&input));
        assert_eq!(key_size(input.reveal()), 32);
    }

    #[test]
    fn test_homomorphic_balance() {
        let value1 = 17u64;
        let value2 = 29u64;
        let blind1 = random_scalar();
        let blind2 = random_scalar();

        let commit1 = create_commitment(value1, &blind1).expect("valid commitment");
        let commit2 = create_commitment(value2, &blind2).expect("valid commitment");

        let blind_sum = &blind1 + &blind2;
        let commit_sum = create_commitment(value1 + value2, &blind_sum).expect("valid commitment");

        assert_eq!((&commit1 + &commit2).as_bytes(), commit_sum.as_bytes());
    }

    #[test]
    fn test_verify_blind_balance_ok() {
        let input = random_scalar();
        let pay = random_scalar();
        let change = balance_blindings(&input, &[pay.dangerous_clone()]);

        assert!(verify_blind_balance(
            &[input.dangerous_clone()],
            &[pay.dangerous_clone(), change]
        ));
    }

    #[test]
    fn test_verify_blind_mismatch() {
        let in_one = random_scalar();
        let out_one = random_scalar();
        assert!(!verify_blind_balance(&[in_one], &[out_one]));
    }

    #[test]
    fn test_blind_balance_multi() {
        let out_one = random_scalar();
        let out_two = random_scalar();
        let out_three = random_scalar();
        let fee = random_scalar();

        let sum_out = [&out_one, &out_two, &out_three, &fee]
            .iter()
            .skip(1)
            .fold(out_one.dangerous_clone(), |acc, item| &acc + *item);

        let in_one = random_scalar();
        let in_two = &sum_out - &in_one;

        assert!(verify_blind_balance(
            &[in_one, in_two],
            &[
                out_one.dangerous_clone(),
                out_two.dangerous_clone(),
                out_three.dangerous_clone(),
                fee.dangerous_clone()
            ]
        ));
    }

    #[test]
    fn test_blind_balance_random() {
        for _ in 0..48 {
            let fee = random_scalar();
            let mut outs = Vec::new();
            let count = 1 + (usize::from(fee.as_bytes()[0]) % 8);
            for _ in 0..count {
                outs.push(random_scalar());
            }

            let sum_out = outs
                .iter()
                .skip(1)
                .fold(outs[0].dangerous_clone(), |acc, item| &acc + item);
            let in_one = random_scalar();
            let in_two = &(&sum_out + &fee) - &in_one;

            let mut out_all = outs
                .iter()
                .map(|item| item.dangerous_clone())
                .collect::<Vec<_>>();
            out_all.push(fee.dangerous_clone());
            assert!(verify_blind_balance(&[in_one, in_two], &out_all));
        }
    }

    #[test]
    fn test_scalar_invalid_bytes() {
        let invalid = [0xFFu8; 32];
        assert!(Z00ZScalar::try_from_bytes(invalid).is_err());
    }

    #[test]
    fn test_value_sum_overflow() {
        assert!(u64::MAX.checked_add(1).is_none());
    }

    #[test]
    fn test_verify_tx_balance_ok() {
        let in_a = commit(40, 7);
        let in_b = commit(60, 11);
        let out = &in_a + &in_b;

        assert!(verify_tx_balance(&[in_a, in_b], &[out]));
    }

    #[test]
    fn test_verify_tx_balance_fails() {
        let input = commit(10, 7);
        let output = commit(5, 8);
        assert!(!verify_tx_balance(&[input], &[output]));
    }

    #[test]
    fn test_meta_commit_ok() {
        let in_a_r = blind(11);
        let in_b_r = blind(13);
        let out_a_r = blind(17);
        let out_b_r = blind(19);
        let fee_r = &(&(&in_a_r + &in_b_r) - &out_a_r) - &out_b_r;

        let in_a = create_commitment(70, &in_a_r).expect("in a");
        let in_b = create_commitment(30, &in_b_r).expect("in b");
        let out_a = create_commitment(60, &out_a_r).expect("out a");
        let out_b = create_commitment(35, &out_b_r).expect("out b");
        let fee = create_commitment(5, &fee_r).expect("fee");
        let zero = &in_a - &in_a;

        assert_eq!(
            verify_tx_balance_meta(&[in_a, in_b], &[out_a, out_b, fee], &zero),
            Ok(())
        );
    }

    #[test]
    fn test_meta_commit_mismatch() {
        let in_c = commit(10, 31);
        let out_c = commit(10, 31);
        let bad_meta = commit(1, 47);

        assert_eq!(
            verify_tx_balance_meta(&[in_c], &[out_c], &bad_meta),
            Err(TxBalErr::MetaMismatch)
        );
    }

    #[test]
    fn test_meta_commit_bad_inputs() {
        let in_c = commit(10, 31);
        let out_c = commit(9, 31);
        let fee = commit(1, 47);
        let zero = &in_c - &in_c;

        assert_eq!(
            verify_tx_balance_meta(&[in_c], &[out_c, fee], &zero),
            Err(TxBalErr::CommitMismatch)
        );
    }

    #[test]
    fn test_commit_zero_balanced_ok() {
        let in_c = commit(25, 55);
        let out_c = commit(25, 55);
        let zero = &in_c - &in_c;

        assert_eq!(verify_tx_balance_meta(&[in_c], &[out_c], &zero), Ok(()));
    }

    #[test]
    fn test_fee_rule_counts_twice() {
        let in_a_r = blind(11);
        let in_b_r = blind(13);
        let out_a_r = blind(17);
        let out_b_r = blind(19);
        let fee_r = &(&(&in_a_r + &in_b_r) - &out_a_r) - &out_b_r;

        let in_a = create_commitment(70, &in_a_r).expect("in a");
        let in_b = create_commitment(30, &in_b_r).expect("in b");
        let out_a = create_commitment(60, &out_a_r).expect("out a");
        let out_b = create_commitment(35, &out_b_r).expect("out b");
        let fee = create_commitment(5, &fee_r).expect("fee");
        let zero = &in_a - &in_a;

        assert!(verify_tx_balance(
            &[in_a.clone(), in_b.clone()],
            &[out_a.clone(), out_b.clone(), fee.clone()]
        ));
        assert!(!verify_tx_balance(
            &[in_a.clone(), in_b.clone()],
            &[out_a.clone(), out_b.clone(), fee.clone(), fee.clone()]
        ));
        assert_eq!(
            verify_tx_balance_meta(&[in_a, in_b], &[out_a, out_b, fee], &zero),
            Ok(())
        );
    }

    #[test]
    fn test_tx_balance_empty_ok() {
        let empty: [Z00ZCommitment; 0] = [];
        let non_empty = [commit(1, 1)];

        assert!(verify_tx_balance(&empty, &empty));
        assert!(!verify_tx_balance(&empty, &non_empty));
        assert!(!verify_tx_balance(&non_empty, &empty));
    }
}
