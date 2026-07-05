#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::ShardVoteLocalSignatureDomain;

use crate::commit_subject::digest_bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteSignatureScheme {
    DeterministicLocal,
}

impl VoteSignatureScheme {
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::DeterministicLocal => 1,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeterministicLocal => "deterministic_local",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteSignature {
    pub scheme: VoteSignatureScheme,
    pub bytes: Vec<u8>,
}

impl VoteSignature {
    #[must_use]
    pub fn new(scheme: VoteSignatureScheme, bytes: Vec<u8>) -> Self {
        Self { scheme, bytes }
    }
}

pub trait VoteSigner {
    fn sign_vote(&self, unsigned_vote: &[u8]) -> VoteSignature;
}

pub trait VoteSignatureVerifier {
    fn verify_vote_signature(&self, unsigned_vote: &[u8], signature: &VoteSignature) -> bool;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicLocalVoteSigner;

impl VoteSigner for DeterministicLocalVoteSigner {
    fn sign_vote(&self, unsigned_vote: &[u8]) -> VoteSignature {
        VoteSignature::new(
            VoteSignatureScheme::DeterministicLocal,
            digest_bytes::<ShardVoteLocalSignatureDomain>("sim_signature", unsigned_vote).to_vec(),
        )
    }
}

impl VoteSignatureVerifier for DeterministicLocalVoteSigner {
    fn verify_vote_signature(&self, unsigned_vote: &[u8], signature: &VoteSignature) -> bool {
        if signature.scheme != VoteSignatureScheme::DeterministicLocal
            || signature.bytes.len() != 32
        {
            return false;
        }

        signature.bytes
            == digest_bytes::<ShardVoteLocalSignatureDomain>("sim_signature", unsigned_vote)
    }
}

#[must_use]
pub fn verify_vote_signature(unsigned_vote: &[u8], signature: &VoteSignature) -> bool {
    match signature.scheme {
        VoteSignatureScheme::DeterministicLocal => {
            DeterministicLocalVoteSigner.verify_vote_signature(unsigned_vote, signature)
        }
    }
}
