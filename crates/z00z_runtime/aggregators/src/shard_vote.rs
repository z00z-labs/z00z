#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::ShardVoteDomain;

use crate::{
    commit_subject::{
        digest_bytes, push_bytes32, push_len_prefixed, push_shard_id, push_u64, push_u8,
        COMMIT_SUBJECT_VERSION,
    },
    placement::AggregatorId,
    signature::{
        verify_vote_signature, DeterministicLocalVoteSigner, VoteSignature, VoteSignatureScheme,
        VoteSigner,
    },
    types::ShardId,
};

const SHARD_VOTE_TAG: &[u8] = b"z00z.shard_vote";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardVoteRole {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardVoteKind {
    Prepare,
    Commit,
    LocalCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShardVote {
    pub version: u8,
    pub voter_id: AggregatorId,
    pub voter_role: ShardVoteRole,
    pub shard_id: ShardId,
    pub term: u64,
    pub membership_digest: [u8; 32],
    pub subject_digest: [u8; 32],
    pub vote_kind: ShardVoteKind,
    pub signature: VoteSignature,
}

impl ShardVote {
    #[must_use]
    pub fn new(
        voter_id: AggregatorId,
        voter_role: ShardVoteRole,
        shard_id: ShardId,
        term: u64,
        membership_digest: [u8; 32],
        subject_digest: [u8; 32],
        vote_kind: ShardVoteKind,
        signer: &impl VoteSigner,
    ) -> Self {
        let unsigned = encode_unsigned(
            voter_id,
            voter_role,
            shard_id,
            term,
            membership_digest,
            subject_digest,
            vote_kind,
        );
        let signature = signer.sign_vote(&unsigned);
        Self {
            version: COMMIT_SUBJECT_VERSION,
            voter_id,
            voter_role,
            shard_id,
            term,
            membership_digest,
            subject_digest,
            vote_kind,
            signature,
        }
    }

    #[must_use]
    pub fn new_local(
        voter_id: AggregatorId,
        voter_role: ShardVoteRole,
        shard_id: ShardId,
        term: u64,
        membership_digest: [u8; 32],
        subject_digest: [u8; 32],
        vote_kind: ShardVoteKind,
    ) -> Self {
        Self::new(
            voter_id,
            voter_role,
            shard_id,
            term,
            membership_digest,
            subject_digest,
            vote_kind,
            &DeterministicLocalVoteSigner,
        )
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.unsigned_bytes();
        push_u8(&mut out, self.signature.scheme.code());
        push_len_prefixed(&mut out, &self.signature.bytes);
        out
    }

    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        digest_bytes::<ShardVoteDomain>("digest", &self.encode())
    }

    #[must_use]
    pub fn has_valid_signature(&self) -> bool {
        verify_vote_signature(&self.unsigned_bytes(), &self.signature)
    }

    #[must_use]
    pub const fn signature_scheme(&self) -> VoteSignatureScheme {
        self.signature.scheme
    }

    fn unsigned_bytes(&self) -> Vec<u8> {
        encode_unsigned(
            self.voter_id,
            self.voter_role,
            self.shard_id,
            self.term,
            self.membership_digest,
            self.subject_digest,
            self.vote_kind,
        )
    }
}

fn encode_unsigned(
    voter_id: AggregatorId,
    voter_role: ShardVoteRole,
    shard_id: ShardId,
    term: u64,
    membership_digest: [u8; 32],
    subject_digest: [u8; 32],
    vote_kind: ShardVoteKind,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(160);
    out.extend_from_slice(SHARD_VOTE_TAG);
    push_u8(&mut out, COMMIT_SUBJECT_VERSION);
    push_u64(&mut out, u64::from(voter_id.as_u16()));
    push_u8(
        &mut out,
        match voter_role {
            ShardVoteRole::Primary => 1,
            ShardVoteRole::Secondary => 2,
        },
    );
    push_shard_id(&mut out, shard_id);
    push_u64(&mut out, term);
    push_bytes32(&mut out, membership_digest);
    push_bytes32(&mut out, subject_digest);
    push_u8(
        &mut out,
        match vote_kind {
            ShardVoteKind::Prepare => 1,
            ShardVoteKind::Commit => 2,
            ShardVoteKind::LocalCommit => 3,
        },
    );
    out
}
