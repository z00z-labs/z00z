use blake2::{Blake2b512, Digest};
use once_cell::sync::Lazy;
use p3_field::PrimeField64;
use p3_goldilocks::{
    Goldilocks, Poseidon2GoldilocksHL, HL_GOLDILOCKS_8_EXTERNAL_ROUND_CONSTANTS,
    HL_GOLDILOCKS_8_INTERNAL_ROUND_CONSTANTS, MATRIX_DIAG_8_GOLDILOCKS,
};
use p3_poseidon2::{ExternalLayerConstants, Poseidon2};
use p3_symmetric::Permutation;

/// Width of the live width-eight Goldilocks Poseidon2 permutation.
pub const POSEIDON2_GOLDILOCKS_WIDTH_V1: usize = 8;
/// Canonical modulus of the live Goldilocks field.
pub const POSEIDON2_GOLDILOCKS_MODULUS_V1: u64 = 0xffff_ffff_0000_0001;
/// Number of state words absorbed before each live sponge permutation.
pub const POSEIDON2_GOLDILOCKS_RATE_V1: usize = POSEIDON2_GOLDILOCKS_WIDTH_V1 - 1;
/// Number of Goldilocks words serialized into each live hash output.
pub const POSEIDON2_GOLDILOCKS_OUTPUT_WORDS_V1: usize = 4;
/// Width in bytes of one packed Goldilocks input word.
pub const POSEIDON2_GOLDILOCKS_WORD_BYTES_V1: usize = 8;
/// Width in bytes of every framed byte-string length.
pub const POSEIDON2_GOLDILOCKS_FRAME_BYTES_V1: usize = 4;
/// Width in bytes of the framed item count.
pub const POSEIDON2_GOLDILOCKS_COUNT_BYTES_V1: usize = 8;
/// Terminal Goldilocks word appended to every live framed stream.
pub const POSEIDON2_GOLDILOCKS_DELIMITER_V1: u64 = 1;

/// Project-owned raw parameters for the live Goldilocks Poseidon2 profile.
///
/// The values are imported exactly once from the pinned Plonky3 instantiation.
/// Consumers receive only primitive values, so no Plonky3 type becomes part of
/// the Z00Z API boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Poseidon2GoldilocksParamsV1 {
    external_round_constants: [[[u64; POSEIDON2_GOLDILOCKS_WIDTH_V1]; 4]; 2],
    internal_round_constants: [u64; 22],
    internal_matrix_diagonal: [u64; POSEIDON2_GOLDILOCKS_WIDTH_V1],
}

impl Poseidon2GoldilocksParamsV1 {
    /// Return the saved external round constants for the live profile.
    #[must_use]
    pub const fn external_round_constants(self) -> [[[u64; POSEIDON2_GOLDILOCKS_WIDTH_V1]; 4]; 2] {
        self.external_round_constants
    }

    /// Return the saved internal round constants for the live profile.
    #[must_use]
    pub const fn internal_round_constants(self) -> [u64; 22] {
        self.internal_round_constants
    }

    /// Return the internal linear-layer diagonal for the live profile.
    #[must_use]
    pub const fn internal_matrix_diagonal(self) -> [u64; POSEIDON2_GOLDILOCKS_WIDTH_V1] {
        self.internal_matrix_diagonal
    }
}

/// Return the sole project-owned raw parameter view for live Poseidon2.
#[must_use]
pub fn poseidon2_goldilocks_params_v1() -> Poseidon2GoldilocksParamsV1 {
    Poseidon2GoldilocksParamsV1 {
        external_round_constants: HL_GOLDILOCKS_8_EXTERNAL_ROUND_CONSTANTS,
        internal_round_constants: HL_GOLDILOCKS_8_INTERNAL_ROUND_CONSTANTS,
        internal_matrix_diagonal: MATRIX_DIAG_8_GOLDILOCKS.map(|value| value.as_canonical_u64()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConsensusHash([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WalletHash([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HashFunction {
    Poseidon2,
    Blake2b,
}

pub fn hash_fn_for_domain(domain: &[u8]) -> HashFunction {
    if domain.starts_with(b"Z00Z/")
        || domain.starts_with(b"z00z.consensus.")
        || domain.starts_with(b"z00z.payment.")
        || domain.starts_with(b"z00z.receiver.")
    {
        HashFunction::Poseidon2
    } else {
        HashFunction::Blake2b
    }
}

impl ConsensusHash {
    pub(crate) fn from_poseidon2(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl WalletHash {
    pub(crate) fn from_blake2b(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

pub fn poseidon2_hash(domain: &[u8], data: &[&[u8]]) -> [u8; 32] {
    let words = poseidon2_framed_words_v1(domain, data);
    let poseidon = poseidon2_perm();
    let mut state = [Goldilocks::new(0); POSEIDON2_GOLDILOCKS_WIDTH_V1];
    let mut rate_index = 0usize;

    for word in words {
        state[rate_index] += Goldilocks::new(word);
        rate_index += 1;

        if rate_index == POSEIDON2_GOLDILOCKS_RATE_V1 {
            poseidon.permute_mut(&mut state);
            rate_index = 0;
        }
    }

    poseidon.permute_mut(&mut state);

    let mut out = [0u8; 32];
    for (index, item) in state
        .iter()
        .take(POSEIDON2_GOLDILOCKS_OUTPUT_WORDS_V1)
        .enumerate()
    {
        out[index * 8..(index + 1) * 8].copy_from_slice(&item.as_canonical_u64().to_le_bytes());
    }
    out
}

/// Build the exact framed Goldilocks word stream used by [`poseidon2_hash`].
///
/// This is the sole project owner for the byte framing and padding grammar.
/// Recursive backends consume this primitive contract rather than inventing a
/// second hash serialization.
#[must_use]
pub fn poseidon2_framed_words_v1(domain: &[u8], data: &[&[u8]]) -> Vec<u64> {
    let mut packer = WordPacker::new();
    packer.push_frame_bytes(domain);
    packer.push_u64_le(data.len() as u64);
    for item in data {
        packer.push_frame_bytes(item);
    }
    packer.finalize()
}

fn poseidon2_perm() -> &'static Poseidon2GoldilocksHL<POSEIDON2_GOLDILOCKS_WIDTH_V1> {
    static INSTANCE: Lazy<Poseidon2GoldilocksHL<POSEIDON2_GOLDILOCKS_WIDTH_V1>> = Lazy::new(|| {
        let params = poseidon2_goldilocks_params_v1();
        Poseidon2::new(
            ExternalLayerConstants::<Goldilocks, POSEIDON2_GOLDILOCKS_WIDTH_V1>::new_from_saved_array(
                params.external_round_constants(),
                Goldilocks::new_array,
            ),
            Goldilocks::new_array(params.internal_round_constants()).to_vec(),
        )
    });

    &INSTANCE
}

struct WordPacker {
    words: Vec<u64>,
    block: [u8; POSEIDON2_GOLDILOCKS_WORD_BYTES_V1],
    used: usize,
    total_len: u64,
}

impl WordPacker {
    fn new() -> Self {
        Self {
            words: Vec::new(),
            block: [0u8; POSEIDON2_GOLDILOCKS_WORD_BYTES_V1],
            used: 0,
            total_len: 0,
        }
    }

    fn push_u64_le(&mut self, value: u64) {
        self.push_bytes(&value.to_le_bytes());
    }

    fn push_frame_bytes(&mut self, bytes: &[u8]) {
        self.push_bytes(&(bytes.len() as u32).to_le_bytes());
        self.push_bytes(bytes);
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        self.total_len = self.total_len.saturating_add(bytes.len() as u64);

        for &byte in bytes {
            self.block[self.used] = byte;
            self.used += 1;
            if self.used == POSEIDON2_GOLDILOCKS_WORD_BYTES_V1 {
                self.words.push(u64::from_le_bytes(self.block));
                self.block = [0u8; POSEIDON2_GOLDILOCKS_WORD_BYTES_V1];
                self.used = 0;
            }
        }
    }

    fn finalize(mut self) -> Vec<u64> {
        let mut out = Vec::with_capacity(self.words.len() + 3);
        out.push(self.total_len);
        out.append(&mut self.words);

        if self.used > 0 {
            out.push(u64::from_le_bytes(self.block));
        }

        out.push(POSEIDON2_GOLDILOCKS_DELIMITER_V1);
        out
    }
}

pub fn compute_consensus_hash(domain: &[u8], data: &[&[u8]]) -> ConsensusHash {
    ConsensusHash::from_poseidon2(poseidon2_hash(domain, data))
}

pub fn blake2b_hash(domain: &[u8], data: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Blake2b512::new();
    hasher.update((domain.len() as u32).to_le_bytes());
    hasher.update(domain);

    for item in data {
        hasher.update((item.len() as u32).to_le_bytes());
        hasher.update(item);
    }

    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest[..32]);
    out
}

pub fn compute_wallet_seed_hash(seed: &[u8]) -> WalletHash {
    WalletHash::from_blake2b(blake2b_hash(b"z00z/wallet/seed", &[seed]))
}

pub fn hash_db_record_id(record_type: &str, key: &[u8]) -> WalletHash {
    WalletHash::from_blake2b(blake2b_hash(
        b"z00z/wallet/db_id",
        &[record_type.as_bytes(), key],
    ))
}

pub fn hash_cache_key(leaf_hash: &[u8; 32]) -> WalletHash {
    WalletHash::from_blake2b(blake2b_hash(b"z00z/wallet/cache", &[leaf_hash]))
}

#[cfg(test)]
mod poseidon2_parameter_parity_tests {
    use core::convert::TryFrom;

    use p3_field::PrimeField64;
    use p3_goldilocks::{
        Goldilocks, Poseidon2ExternalLayerGoldilocksHL, Poseidon2InternalLayerGoldilocks,
    };
    use p3_poseidon2::{
        ExternalLayer, ExternalLayerConstants, ExternalLayerConstructor, InternalLayer,
        InternalLayerConstructor,
    };

    use super::{
        poseidon2_framed_words_v1, poseidon2_goldilocks_params_v1, poseidon2_hash,
        POSEIDON2_GOLDILOCKS_MODULUS_V1, POSEIDON2_GOLDILOCKS_WIDTH_V1,
    };

    fn reduce(value: u128) -> u64 {
        u64::try_from(value % u128::from(POSEIDON2_GOLDILOCKS_MODULUS_V1))
            .expect("Goldilocks residue")
    }

    fn external_linear(state: [u64; 8]) -> [u64; 8] {
        let mut result = [0_u64; 8];
        for chunk in 0..2 {
            let input = &state[chunk * 4..chunk * 4 + 4];
            for (row, coefficients) in [[5_u64, 7, 1, 3], [4, 6, 1, 1], [1, 3, 5, 7], [1, 1, 4, 6]]
                .iter()
                .enumerate()
            {
                result[chunk * 4 + row] = reduce(
                    input
                        .iter()
                        .zip(coefficients.iter().copied())
                        .map(|(value, coefficient)| u128::from(*value) * u128::from(coefficient))
                        .sum(),
                );
            }
        }
        let sums = [
            reduce(u128::from(result[0]) + u128::from(result[4])),
            reduce(u128::from(result[1]) + u128::from(result[5])),
            reduce(u128::from(result[2]) + u128::from(result[6])),
            reduce(u128::from(result[3]) + u128::from(result[7])),
        ];
        for lane in 0..8 {
            result[lane] = reduce(u128::from(result[lane]) + u128::from(sums[lane % 4]));
        }
        result
    }

    fn pow7(value: u64) -> u64 {
        let square = reduce(u128::from(value) * u128::from(value));
        let cube = reduce(u128::from(square) * u128::from(value));
        let sixth = reduce(u128::from(cube) * u128::from(cube));
        reduce(u128::from(sixth) * u128::from(value))
    }

    fn emulated_permutation(mut state: [u64; 8]) -> [u64; 8] {
        let params = poseidon2_goldilocks_params_v1();
        state = external_linear(state);
        for constants in params.external_round_constants()[0] {
            for lane in 0..8 {
                state[lane] = pow7(reduce(
                    u128::from(state[lane]) + u128::from(constants[lane]),
                ));
            }
            state = external_linear(state);
        }
        let diagonal = params.internal_matrix_diagonal();
        for constant in params.internal_round_constants() {
            state[0] = pow7(reduce(u128::from(state[0]) + u128::from(constant)));
            let sum = reduce(state.iter().map(|value| u128::from(*value)).sum());
            state = core::array::from_fn(|lane| {
                reduce(u128::from(state[lane]) * u128::from(diagonal[lane]) + u128::from(sum))
            });
        }
        for constants in params.external_round_constants()[1] {
            for lane in 0..8 {
                state[lane] = pow7(reduce(
                    u128::from(state[lane]) + u128::from(constants[lane]),
                ));
            }
            state = external_linear(state);
        }
        state
    }

    #[test]
    fn test_poseidon2_pinned_layers() {
        let params = poseidon2_goldilocks_params_v1();
        let input = [1_u64, 2, 3, 4, 5, 6, 7, 8];
        let external =
            Poseidon2ExternalLayerGoldilocksHL::<POSEIDON2_GOLDILOCKS_WIDTH_V1>::new_from_constants(
                ExternalLayerConstants::new_from_saved_array(
                    params.external_round_constants(),
                    Goldilocks::new_array,
                ),
            );
        let internal = Poseidon2InternalLayerGoldilocks::new_from_constants(
            Goldilocks::new_array(params.internal_round_constants()).to_vec(),
        );
        let mut native = input.map(Goldilocks::new);
        external.permute_state_initial(&mut native);
        let mut emulated = external_linear(input);
        for constants in params.external_round_constants()[0] {
            for lane in 0..8 {
                emulated[lane] = pow7(reduce(
                    u128::from(emulated[lane]) + u128::from(constants[lane]),
                ));
            }
            emulated = external_linear(emulated);
        }
        assert_eq!(
            native.map(|value| value.as_canonical_u64()),
            emulated,
            "initial external layer"
        );
        internal.permute_state(&mut native);
        let diagonal = params.internal_matrix_diagonal();
        for constant in params.internal_round_constants() {
            emulated[0] = pow7(reduce(u128::from(emulated[0]) + u128::from(constant)));
            let sum = reduce(emulated.iter().map(|value| u128::from(*value)).sum());
            emulated = core::array::from_fn(|lane| {
                reduce(u128::from(emulated[lane]) * u128::from(diagonal[lane]) + u128::from(sum))
            });
        }
        assert_eq!(
            native.map(|value| value.as_canonical_u64()),
            emulated,
            "internal layer"
        );
        external.permute_state_terminal(&mut native);
        for constants in params.external_round_constants()[1] {
            for lane in 0..8 {
                emulated[lane] = pow7(reduce(
                    u128::from(emulated[lane]) + u128::from(constants[lane]),
                ));
            }
            emulated = external_linear(emulated);
        }
        assert_eq!(
            native.map(|value| value.as_canonical_u64()),
            emulated,
            "terminal external layer"
        );

        let definition_id = [b'A'; 32];
        let serial = 0_u32.to_le_bytes();
        let words = poseidon2_framed_words_v1(
            b"z00z.storage.key.serial.v1",
            &[b"", &definition_id, &serial],
        );
        assert_eq!(words.len(), 13);
        let mut sponge = [0_u64; 8];
        sponge[..7].copy_from_slice(&words[..7]);
        sponge = emulated_permutation(sponge);
        for lane in 0..6 {
            sponge[lane] = reduce(u128::from(sponge[lane]) + u128::from(words[7 + lane]));
        }
        sponge = emulated_permutation(sponge);
        let mut emulated_hash = [0_u8; 32];
        for (index, word) in sponge[..4].iter().enumerate() {
            emulated_hash[index * 8..index * 8 + 8].copy_from_slice(&word.to_le_bytes());
        }
        assert_eq!(
            poseidon2_hash(
                b"z00z.storage.key.serial.v1",
                &[b"", &definition_id, &serial],
            ),
            emulated_hash,
            "framed sponge"
        );
    }
}
