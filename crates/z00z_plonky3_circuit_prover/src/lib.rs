//! Canonical Z00Z Plonky3 circuit prover and FRI-recursive verifier.
//!
//! Generics glossary used across this crate:
//! - `F`: Prover/verifier base field (BabyBear/KoalaBear/Goldilocks). PCS and FFTs operate over `F`.
//! - `P`: Cryptographic permutation over `F` used by hash/compress and the challenger.
//! - `EF`: Element field in circuit traces. Either `F` (base) or `BinomialExtensionField<F, D>`.
//! - `D`: Element-field extension degree. Must equal `EF::DIMENSION`. AIRs are parameterized as `<F, D>`.
//! - `CD`: FRI challenge field degree, independent of `D`.
//!
//! - Build a field-specific config via `config::{baby_bear, koala_bear, goldilocks}`.
//! - Create a `BatchStarkProver` from that config.
//! - Prove either circuit-runner traces or direct domain-specific AIR tables.
//!
//! Domain-specific tables should use [`BatchStarkProver::prove_direct_tables`]
//! so they are not redundantly lowered through general circuit witness and ALU
//! tables. Circuit-oriented consumers use
//! [`BatchStarkProver::prove_all_tables`].
//!
//! ```ignore
//! use z00z_plonky3_circuit_prover::{BatchStarkProver, config};
//!
//! let prover = BatchStarkProver::new(config::koala_bear());
//! // Register the application TableProver, construct bounded table traces,
//! // then call prover.prove_direct_tables(&traces).
//! ```
#![no_std]

extern crate alloc;

pub mod air;
pub mod backend;
pub mod batch_stark_prover;
pub mod challenger;
pub mod challenger_perm;
pub mod common;
pub mod config;
pub mod constraint_profile;
pub mod field_params;
pub mod generation;
pub mod manifest;
pub mod pcs;
pub mod prelude;
pub mod public_inputs;
pub mod recursion;
pub mod traits;
pub mod types;
pub mod verifier;

// Canonical prover API.
pub use batch_stark_prover::*;
pub use constraint_profile::ConstraintProfile;

// Canonical FRI-recursion API. Colocation guarantees that proof/prover-data
// types cannot diverge across duplicate Cargo package identities.
pub use backend::fri::FriRecursionConfig;
pub use backend::{FriRecursionBackend, FriRecursionBackendD5, FriRecursionBackendForExt};
pub use challenger::CircuitChallenger;
pub use challenger_perm::ChallengerPermConfig;
pub use generation::{GenerationError, PcsGeneration, generate_batch_challenges};
pub use p3_circuit::ops;
pub use p3_circuit::ops::{PermConfig, Poseidon2Config};
pub use pcs::fri::FriVerifierParams;
pub use public_inputs::{
    BatchStarkVerifierInputsBuilder, CommitmentOpening, FriVerifierInputs, PublicInputBuilder,
    StarkVerifierInputs, StarkVerifierInputsBuilder, construct_batch_stark_verifier_inputs,
};
pub use recursion::{
    AggregationCircuitFingerprint, AggregationPrepCache, BatchOnly, NextLayerPrepCache,
    PcsRecursionBackend, ProveNextLayerParams, RecursionInput, RecursionOutput,
    VerifierCircuitResult, build_aggregation_layer_circuit, build_and_prove_aggregation_layer,
    build_and_prove_aggregation_layer_cross, build_and_prove_next_layer, build_next_layer_circuit,
    build_next_layer_prep, prove_aggregation_layer, prove_aggregation_layer_cross,
    prove_next_layer,
};
pub use traits::{
    Recursive, RecursiveAir, RecursiveChallenger, RecursiveExtensionMmcs, RecursiveMmcs,
    RecursivePcs,
};
pub use types::{
    BatchProofTargets, CommitmentTargets, CommonDataTargets, OpenedValuesTargets, ProofTargets,
    RecursiveLagrangeSelectors, StarkChallenges, Target,
};
pub use verifier::{
    ObservableCommitment, VerificationError, verify_batch_circuit, verify_p3_uni_proof_circuit,
};
