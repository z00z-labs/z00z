use std::convert::TryFrom;

use sha2::{
    compress256,
    digest::generic_array::{typenum::U64, GenericArray},
    Digest as _, Sha256,
};
use thiserror::Error;

use super::{chain_len_prefixed, dst, CheckpointShaRole};

/// Frozen FIPS 180-4 SHA-256 initial chaining state for checkpoint circuits.
pub const SHA256_IV_V2: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Number of input bytes consumed by one SHA-256 compression block.
pub const SHA256_BLOCK_BYTES_V2: u64 = 64;

/// Largest unpadded message length permitted by the SHA-256 bit-length field.
pub const SHA256_MAX_BYTES_V2: u64 = (1_u64 << 61) - 1;

/// Exact byte width of a SHA-256 digest.
pub const SHA256_DIGEST_BYTES_V2: usize = 32;

/// Failure emitted before a V2 SHA-256 stream can overflow the FIPS bit length.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum CheckpointSha256Error {
    #[error("checkpoint SHA-256 input exceeds the FIPS 2^61 - 1 byte limit")]
    InputTooLong,
    #[error("checkpoint SHA-256 part is already open")]
    PartAlreadyOpen,
    #[error("checkpoint SHA-256 has no open part")]
    NoOpenPart,
    #[error("checkpoint SHA-256 part ended before its declared length")]
    IncompletePart,
    #[error("checkpoint SHA-256 part exceeds its declared length")]
    PartTooLong,
}

/// One FIPS 180-4 compression step of a role-framed checkpoint transcript.
///
/// This is an inspection/witness primitive, not a second hash implementation:
/// compression stays in the audited RustCrypto `sha2` dependency owned by
/// `z00z_crypto`.  The source record bytes are public checkpoint material;
/// no secret is exposed by this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointSha256BlockV2 {
    index: u64,
    byte_offset: u64,
    block: [u8; SHA256_BLOCK_BYTES_V2 as usize],
    chaining_before: [u32; 8],
    chaining_after: [u32; 8],
    final_block: bool,
}

impl CheckpointSha256BlockV2 {
    #[must_use]
    pub const fn index(&self) -> u64 {
        self.index
    }

    #[must_use]
    pub const fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    #[must_use]
    pub const fn block(&self) -> &[u8; SHA256_BLOCK_BYTES_V2 as usize] {
        &self.block
    }

    #[must_use]
    pub const fn chaining_before(&self) -> &[u32; 8] {
        &self.chaining_before
    }

    #[must_use]
    pub const fn chaining_after(&self) -> &[u32; 8] {
        &self.chaining_after
    }

    #[must_use]
    pub const fn final_block(&self) -> bool {
        self.final_block
    }

    /// Re-run the RustCrypto compression primitive for this exact block.
    #[must_use]
    pub fn verifies_transition(&self) -> bool {
        Self::verify_transition_parts(&self.block, &self.chaining_before, &self.chaining_after)
    }

    /// Verify one explicit FIPS compression transition without constructing a
    /// second hashing owner at the caller.
    #[must_use]
    pub fn verify_transition_parts(
        block: &[u8; SHA256_BLOCK_BYTES_V2 as usize],
        chaining_before: &[u32; 8],
        chaining_after: &[u32; 8],
    ) -> bool {
        let mut state = *chaining_before;
        let generic = GenericArray::<u8, U64>::clone_from_slice(block);
        compress256(&mut state, core::slice::from_ref(&generic));
        state == *chaining_after
    }

    /// Encode a SHA-256 chaining state as its standard big-endian digest form.
    #[must_use]
    pub fn digest_from_chaining(state: &[u32; 8]) -> [u8; SHA256_DIGEST_BYTES_V2] {
        let mut digest = [0_u8; SHA256_DIGEST_BYTES_V2];
        for (word, slot) in state.iter().zip(digest.chunks_exact_mut(4)) {
            slot.copy_from_slice(&word.to_be_bytes());
        }
        digest
    }
}

/// Error from emitting one inspected SHA-256 block.
#[derive(Debug)]
pub enum CheckpointSha256BlockVisitError<E> {
    Hash(CheckpointSha256Error),
    Visitor(E),
}

/// Streaming role-framed SHA-256 block expander.
///
/// It retains at most one unfinished 64-byte block.  Callers provide the
/// visitor so a 64 MiB bounded source never becomes an in-memory block tape.
/// The preimage grammar, IV, padding, bit length, and compression primitive
/// are all the same as [`CheckpointSha256V2`].
pub struct CheckpointSha256BlockStreamV2 {
    role: CheckpointShaRole,
    state: [u32; 8],
    buffer: [u8; SHA256_BLOCK_BYTES_V2 as usize],
    buffered: usize,
    framed_bytes: u64,
    block_index: u64,
    started: bool,
    open_part_bytes: Option<u64>,
}

impl CheckpointSha256BlockStreamV2 {
    /// Start one role-bound SHA-256 block stream.
    #[must_use]
    pub const fn new(role: CheckpointShaRole) -> Self {
        Self {
            role,
            state: SHA256_IV_V2,
            buffer: [0; SHA256_BLOCK_BYTES_V2 as usize],
            buffered: 0,
            framed_bytes: 0,
            block_index: 0,
            started: false,
            open_part_bytes: None,
        }
    }

    /// Return the exact FIPS message length for the role and aggregate parts.
    ///
    /// `part_bytes` is the sum of all raw part sizes and `part_count` is the
    /// count of their mandatory little-endian u64 prefixes.
    pub fn framed_bytes_for_parts(
        role: CheckpointShaRole,
        part_bytes: u64,
        part_count: u64,
    ) -> Result<u64, CheckpointSha256Error> {
        let initial = CheckpointSha256V2::new(role).framed_bytes();
        let prefixes = part_count
            .checked_mul(8)
            .ok_or(CheckpointSha256Error::InputTooLong)?;
        initial
            .checked_add(prefixes)
            .and_then(|value| value.checked_add(part_bytes))
            .filter(|value| *value <= SHA256_MAX_BYTES_V2)
            .ok_or(CheckpointSha256Error::InputTooLong)
    }

    /// Return the exact role-bound prefix that begins every checkpoint SHA
    /// transcript before its first length-prefixed application part.
    ///
    /// Callers that need a streamed or constrained view must use this owner
    /// API rather than reproduce the domain-separation grammar locally.
    #[must_use]
    pub fn framed_role_prefix(role: CheckpointShaRole) -> Vec<u8> {
        let domain = dst(role.domain(), role.label());
        let mut prefix = Vec::with_capacity(8 + domain.len());
        prefix.extend_from_slice(
            &u64::try_from(domain.len())
                .expect("fixed checkpoint domain fits u64")
                .to_le_bytes(),
        );
        prefix.extend_from_slice(&domain);
        prefix
    }

    /// Return `Q(L) = ceil((L + 9) / 64)` for one FIPS-authorized message.
    pub fn block_count_for_framed_bytes(framed_bytes: u64) -> Result<u64, CheckpointSha256Error> {
        if framed_bytes > SHA256_MAX_BYTES_V2 {
            return Err(CheckpointSha256Error::InputTooLong);
        }
        framed_bytes
            .checked_add(9)
            .and_then(|value| value.checked_add(SHA256_BLOCK_BYTES_V2 - 1))
            .map(|value| value / SHA256_BLOCK_BYTES_V2)
            .ok_or(CheckpointSha256Error::InputTooLong)
    }

    /// Append one complete length-prefixed part while streaming every full
    /// compression block to `visit`.
    pub fn update_part_with<E, F>(
        &mut self,
        part: &[u8],
        visit: &mut F,
    ) -> Result<(), CheckpointSha256BlockVisitError<E>>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), E>,
    {
        let part_len = u64::try_from(part.len()).map_err(|_| {
            CheckpointSha256BlockVisitError::Hash(CheckpointSha256Error::InputTooLong)
        })?;
        self.begin_part_with(part_len, visit)?;
        self.update_part_bytes_with(part, visit)?;
        self.finish_part()
            .map_err(CheckpointSha256BlockVisitError::Hash)
    }

    /// Begin one declared-length part of a checkpoint transcript.
    ///
    /// The caller may feed the body with [`Self::update_part_bytes_with`].
    /// This is the sole streaming extension of the canonical length-prefixed
    /// grammar; it never exposes an unframed raw-byte path.
    pub fn begin_part_with<E, F>(
        &mut self,
        part_len: u64,
        visit: &mut F,
    ) -> Result<(), CheckpointSha256BlockVisitError<E>>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), E>,
    {
        if self.open_part_bytes.is_some() {
            return Err(CheckpointSha256BlockVisitError::Hash(
                CheckpointSha256Error::PartAlreadyOpen,
            ));
        }
        self.start_with(visit)?;
        let next = self
            .framed_bytes
            .checked_add(8)
            .and_then(|value| value.checked_add(part_len))
            .filter(|value| *value <= SHA256_MAX_BYTES_V2)
            .ok_or(CheckpointSha256BlockVisitError::Hash(
                CheckpointSha256Error::InputTooLong,
            ))?;
        self.absorb_with(&part_len.to_le_bytes(), false, visit)?;
        self.framed_bytes = next;
        self.open_part_bytes = Some(part_len);
        Ok(())
    }

    /// Feed bytes into the declared part currently open in this stream.
    pub fn update_part_bytes_with<E, F>(
        &mut self,
        bytes: &[u8],
        visit: &mut F,
    ) -> Result<(), CheckpointSha256BlockVisitError<E>>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), E>,
    {
        let remaining = self
            .open_part_bytes
            .ok_or(CheckpointSha256BlockVisitError::Hash(
                CheckpointSha256Error::NoOpenPart,
            ))?;
        let byte_count = u64::try_from(bytes.len()).map_err(|_| {
            CheckpointSha256BlockVisitError::Hash(CheckpointSha256Error::InputTooLong)
        })?;
        let next_remaining =
            remaining
                .checked_sub(byte_count)
                .ok_or(CheckpointSha256BlockVisitError::Hash(
                    CheckpointSha256Error::PartTooLong,
                ))?;
        self.absorb_with(bytes, false, visit)?;
        self.open_part_bytes = Some(next_remaining);
        Ok(())
    }

    /// Seal the current declared-length part before another part or finalization.
    pub fn finish_part(&mut self) -> Result<(), CheckpointSha256Error> {
        match self.open_part_bytes {
            Some(0) => {
                self.open_part_bytes = None;
                Ok(())
            }
            Some(_) => Err(CheckpointSha256Error::IncompletePart),
            None => Err(CheckpointSha256Error::NoOpenPart),
        }
    }

    /// Finish FIPS padding, emit every remaining block, and return the digest.
    pub fn finalize_with<E, F>(
        mut self,
        visit: &mut F,
    ) -> Result<[u8; SHA256_DIGEST_BYTES_V2], CheckpointSha256BlockVisitError<E>>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), E>,
    {
        if self.open_part_bytes.is_some() {
            return Err(CheckpointSha256BlockVisitError::Hash(
                CheckpointSha256Error::IncompletePart,
            ));
        }
        self.start_with(visit)?;
        let bit_length =
            self.framed_bytes
                .checked_mul(8)
                .ok_or(CheckpointSha256BlockVisitError::Hash(
                    CheckpointSha256Error::InputTooLong,
                ))?;
        self.buffer[self.buffered] = 0x80;
        self.buffered += 1;
        if self.buffered > 56 {
            self.buffer[self.buffered..].fill(0);
            self.buffered = SHA256_BLOCK_BYTES_V2 as usize;
            self.process_buffer(false, visit)?;
        }
        self.buffer[self.buffered..56].fill(0);
        self.buffer[56..].copy_from_slice(&bit_length.to_be_bytes());
        self.buffered = SHA256_BLOCK_BYTES_V2 as usize;
        self.process_buffer(true, visit)?;

        Ok(CheckpointSha256BlockV2::digest_from_chaining(&self.state))
    }

    fn start_with<E, F>(&mut self, visit: &mut F) -> Result<(), CheckpointSha256BlockVisitError<E>>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), E>,
    {
        if self.started {
            return Ok(());
        }
        let prefix = Self::framed_role_prefix(self.role);
        self.absorb_with(&prefix, false, visit)?;
        self.framed_bytes = u64::try_from(prefix.len()).expect("fixed role prefix fits u64");
        self.started = true;
        Ok(())
    }

    fn absorb_with<E, F>(
        &mut self,
        mut bytes: &[u8],
        final_block: bool,
        visit: &mut F,
    ) -> Result<(), CheckpointSha256BlockVisitError<E>>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), E>,
    {
        if self.buffered != 0 {
            let take = (SHA256_BLOCK_BYTES_V2 as usize - self.buffered).min(bytes.len());
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(&bytes[..take]);
            self.buffered += take;
            bytes = &bytes[take..];
            if self.buffered == SHA256_BLOCK_BYTES_V2 as usize {
                self.process_buffer(final_block && bytes.is_empty(), visit)?;
            }
        }
        while bytes.len() >= SHA256_BLOCK_BYTES_V2 as usize {
            self.buffer
                .copy_from_slice(&bytes[..SHA256_BLOCK_BYTES_V2 as usize]);
            bytes = &bytes[SHA256_BLOCK_BYTES_V2 as usize..];
            self.buffered = SHA256_BLOCK_BYTES_V2 as usize;
            self.process_buffer(final_block && bytes.is_empty(), visit)?;
        }
        if !bytes.is_empty() {
            self.buffer[..bytes.len()].copy_from_slice(bytes);
            self.buffered = bytes.len();
        }
        Ok(())
    }

    fn process_buffer<E, F>(
        &mut self,
        final_block: bool,
        visit: &mut F,
    ) -> Result<(), CheckpointSha256BlockVisitError<E>>
    where
        F: FnMut(CheckpointSha256BlockV2) -> Result<(), E>,
    {
        if self.buffered != SHA256_BLOCK_BYTES_V2 as usize {
            return Ok(());
        }
        let before = self.state;
        let block = self.buffer;
        let generic = GenericArray::<u8, U64>::clone_from_slice(&block);
        compress256(&mut self.state, core::slice::from_ref(&generic));
        let byte_offset = self
            .block_index
            .checked_mul(SHA256_BLOCK_BYTES_V2)
            .expect("FIPS input bound keeps block offsets representable");
        let emitted = CheckpointSha256BlockV2 {
            index: self.block_index,
            byte_offset,
            block,
            chaining_before: before,
            chaining_after: self.state,
            final_block,
        };
        visit(emitted).map_err(CheckpointSha256BlockVisitError::Visitor)?;
        self.block_index = self
            .block_index
            .checked_add(1)
            .expect("FIPS input bound keeps block count representable");
        self.buffered = 0;
        self.buffer.fill(0);
        Ok(())
    }
}

/// Streaming form of the canonical role-framed checkpoint SHA-256 function.
///
/// Each call to [`Self::update_part`] is one complete length-prefixed part in
/// the same framing used by [`sha256_256`]. This lets bounded trace sources
/// hash a spool without retaining its full content in memory.
pub struct CheckpointSha256V2 {
    hasher: Sha256,
    framed_bytes: u64,
}

impl CheckpointSha256V2 {
    /// Start one role-bound SHA-256 transcript.
    #[must_use]
    pub fn new(role: CheckpointShaRole) -> Self {
        let mut hasher = Sha256::new();
        let domain = dst(role.domain(), role.label());
        chain_len_prefixed(&mut hasher, &domain);
        let framed_bytes = u64::try_from(domain.len())
            .expect("SHA-256 domain framing length always fits u64")
            .checked_add(8)
            .expect("SHA-256 domain framing length is fixed");
        Self {
            hasher,
            framed_bytes,
        }
    }

    /// Append one complete canonical part without retaining earlier parts.
    pub fn update_part(&mut self, part: &[u8]) -> Result<(), CheckpointSha256Error> {
        let part_len =
            u64::try_from(part.len()).map_err(|_| CheckpointSha256Error::InputTooLong)?;
        let framed = part_len
            .checked_add(8)
            .ok_or(CheckpointSha256Error::InputTooLong)?;
        let next = self
            .framed_bytes
            .checked_add(framed)
            .ok_or(CheckpointSha256Error::InputTooLong)?;
        if next > SHA256_MAX_BYTES_V2 {
            return Err(CheckpointSha256Error::InputTooLong);
        }
        self.hasher.update(part_len.to_le_bytes());
        self.hasher.update(part);
        self.framed_bytes = next;
        Ok(())
    }

    /// Finalize the sole SHA-256 digest for this role-framed transcript.
    #[must_use]
    pub fn finalize(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }

    /// Return the byte count including frozen length framing.
    #[must_use]
    pub const fn framed_bytes(&self) -> u64 {
        self.framed_bytes
    }
}

pub fn sha256_256(domain: &str, label: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    let dst = dst(domain, label);
    chain_len_prefixed(&mut hasher, &dst);
    for part in parts {
        chain_len_prefixed(&mut hasher, part);
    }
    hasher.finalize().into()
}

/// Hash checkpoint V2 data through the sole role registry.
#[must_use]
pub fn sha256_256_role(role: CheckpointShaRole, parts: &[&[u8]]) -> [u8; 32] {
    let mut stream = CheckpointSha256V2::new(role);
    for part in parts {
        // A materialized slice cannot reach the FIPS maximum on supported
        // targets. The fallible streaming API is used for external V2 input.
        stream
            .update_part(part)
            .expect("materialized role-framed SHA-256 input fits FIPS length");
    }
    stream.finalize()
}

pub fn sha256_256_simple(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
