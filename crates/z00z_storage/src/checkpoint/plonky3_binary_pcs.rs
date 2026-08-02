//! Generation-bound binary W32 PCS adapter for the canonical Plonky3 backend.

use p3_circuit::{CircuitBuilder, CircuitBuilderError, NonPrimitiveOpId};
use p3_commit::{
    BuildPeriodicLdeTableFast, Mmcs, OpenedValues, Pcs, PeriodicLdeTable, PolynomialSpace,
};
use p3_field::coset::TwoAdicMultiplicativeCoset;
use p3_field::{PrimeCharacteristicRing, TwoAdicField};
use p3_fri::{FriParameters, TwoAdicFriPcs};
use p3_koala_bear::KoalaBear;
use p3_matrix::dense::RowMajorMatrix;
use z00z_plonky3_circuit_prover::challenger_perm::ChallengerPermConfig;
use z00z_plonky3_circuit_prover::pcs::{
    verify_fri_circuit, FriProofTargets, InputProofTargets, MerkleCapTargets, MmcsProofTargets,
    RecExtensionValMmcs, RecValMmcs, Witness,
};
use z00z_plonky3_circuit_prover::traits::ComsWithOpeningsTargets;
use z00z_plonky3_circuit_prover::types::{
    OpenedValuesTargetsWithLookups, RecursiveLagrangeSelectors,
};
use z00z_plonky3_circuit_prover::{
    CircuitChallenger, GenerationError, ObservableCommitment, PcsGeneration, Recursive,
    RecursiveChallenger, RecursiveMmcs, RecursivePcs, Target, VerificationError,
};

use super::{
    plonky3_binary_mmcs::verify_binary_paths, Plonky3ChallengeMmcsV2, Plonky3ChallengeV2,
    Plonky3ChallengerV2, Plonky3CompressionV2, Plonky3HashV2, Plonky3InnerPcsV2,
    Plonky3StarkConfigV2, Plonky3ValueMmcsV2, PLONKY3_MMCS_DIGEST_ELEMS_V2,
};

type DomainV2 = TwoAdicMultiplicativeCoset<KoalaBear>;
type CommitmentV2 = MerkleCapTargets<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2>;
type NativeProofV2 = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Proof;
type NativeCommitmentV2 =
    <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Commitment;
type NativeDataV2 = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::ProverData;
type NativeComsV2 = [(
    NativeCommitmentV2,
    Vec<(DomainV2, Vec<(Plonky3ChallengeV2, Vec<Plonky3ChallengeV2>)>)>,
)];
type UpstreamRecMmcsV2 =
    RecValMmcs<KoalaBear, PLONKY3_MMCS_DIGEST_ELEMS_V2, Plonky3HashV2, Plonky3CompressionV2>;
type UpstreamInputProofV2 = InputProofTargets<KoalaBear, Plonky3ChallengeV2, UpstreamRecMmcsV2>;
type UpstreamOpeningProofV2 = FriProofTargets<
    KoalaBear,
    Plonky3ChallengeV2,
    RecExtensionValMmcs<
        KoalaBear,
        Plonky3ChallengeV2,
        PLONKY3_MMCS_DIGEST_ELEMS_V2,
        UpstreamRecMmcsV2,
    >,
    UpstreamInputProofV2,
    Witness<KoalaBear>,
>;

pub(super) struct BinaryProofTargetsV2 {
    siblings: Vec<[Target; 4]>,
}

impl BinaryProofTargetsV2 {
    pub(super) fn siblings(&self) -> &[[Target; 4]] {
        &self.siblings
    }
}

impl Recursive<Plonky3ChallengeV2> for BinaryProofTargetsV2 {
    type Input = <Plonky3ValueMmcsV2 as Mmcs<KoalaBear>>::Proof;

    fn new(circuit: &mut CircuitBuilder<Plonky3ChallengeV2>, input: &Self::Input) -> Self {
        let siblings = input
            .iter()
            .map(|_| circuit.alloc_private_input_array("binary W32 MMCS sibling"))
            .collect();
        Self { siblings }
    }

    fn get_values(_input: &Self::Input) -> Vec<Plonky3ChallengeV2> {
        Vec::new()
    }

    fn get_private_values(input: &Self::Input) -> Vec<Plonky3ChallengeV2> {
        input
            .iter()
            .flat_map(|digest| {
                digest
                    .chunks_exact(4)
                    .map(|chunk| Plonky3ChallengeV2::new([chunk[0], chunk[1], chunk[2], chunk[3]]))
            })
            .collect()
    }
}

impl MmcsProofTargets for BinaryProofTargetsV2 {
    fn salt_targets(&self) -> &[Vec<Target>] {
        &[]
    }
}

pub(super) struct BinaryRecMmcsV2;

impl RecursiveMmcs<KoalaBear, Plonky3ChallengeV2> for BinaryRecMmcsV2 {
    type Input = Plonky3ValueMmcsV2;
    type Commitment = CommitmentV2;
    type Proof = BinaryProofTargetsV2;
}

type BinaryExtMmcsV2 = RecExtensionValMmcs<
    KoalaBear,
    Plonky3ChallengeV2,
    PLONKY3_MMCS_DIGEST_ELEMS_V2,
    BinaryRecMmcsV2,
>;
type BinaryInputProofV2 = InputProofTargets<KoalaBear, Plonky3ChallengeV2, BinaryRecMmcsV2>;
type BinaryOpeningProofV2 = FriProofTargets<
    KoalaBear,
    Plonky3ChallengeV2,
    BinaryExtMmcsV2,
    BinaryInputProofV2,
    Witness<KoalaBear>,
>;

#[derive(Clone)]
pub(super) struct Plonky3PcsV2 {
    inner: Plonky3InnerPcsV2,
}

impl Plonky3PcsV2 {
    pub(super) fn new(
        dft: p3_dft::Radix2DitParallel<KoalaBear>,
        value_mmcs: Plonky3ValueMmcsV2,
        fri: FriParameters<Plonky3ChallengeMmcsV2>,
    ) -> Self {
        Self {
            inner: TwoAdicFriPcs::new(dft, value_mmcs, fri),
        }
    }
}

impl BuildPeriodicLdeTableFast for Plonky3PcsV2 {
    type PeriodicDomain = DomainV2;

    fn maybe_build_periodic_lde_table_fast(
        &self,
        periodic_cols: &[Vec<KoalaBear>],
        trace_domain: DomainV2,
        quotient_domain: DomainV2,
    ) -> Option<PeriodicLdeTable<KoalaBear>> {
        self.inner
            .maybe_build_periodic_lde_table_fast(periodic_cols, trace_domain, quotient_domain)
    }
}

impl Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2> for Plonky3PcsV2 {
    type Domain = DomainV2;
    type Commitment = NativeCommitmentV2;
    type ProverData = NativeDataV2;
    type EvaluationsOnDomain<'a> = <Plonky3InnerPcsV2 as Pcs<
        Plonky3ChallengeV2,
        Plonky3ChallengerV2,
    >>::EvaluationsOnDomain<'a>;
    type Proof = NativeProofV2;
    type Error = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::Error;

    const ZK: bool = <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::ZK;

    fn natural_domain_for_degree(&self, degree: usize) -> Self::Domain {
        <Plonky3InnerPcsV2 as Pcs<
            Plonky3ChallengeV2,
            Plonky3ChallengerV2,
        >>::natural_domain_for_degree(&self.inner, degree)
    }

    fn log_max_lde_height(&self) -> usize {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::log_max_lde_height(
            &self.inner,
        )
    }

    fn commit(
        &self,
        evaluations: impl IntoIterator<Item = (Self::Domain, RowMajorMatrix<KoalaBear>)>,
    ) -> (Self::Commitment, Self::ProverData) {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::commit(
            &self.inner,
            evaluations,
        )
    }

    fn get_quotient_ldes(
        &self,
        evaluations: impl IntoIterator<Item = (Self::Domain, RowMajorMatrix<KoalaBear>)>,
        num_chunks: usize,
    ) -> Vec<RowMajorMatrix<KoalaBear>> {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::get_quotient_ldes(
            &self.inner,
            evaluations,
            num_chunks,
        )
    }

    fn commit_ldes(
        &self,
        ldes: Vec<RowMajorMatrix<KoalaBear>>,
    ) -> (Self::Commitment, Self::ProverData) {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::commit_ldes(
            &self.inner,
            ldes,
        )
    }

    fn get_evaluations_on_domain<'a>(
        &self,
        prover_data: &'a Self::ProverData,
        index: usize,
        domain: Self::Domain,
    ) -> Self::EvaluationsOnDomain<'a> {
        <Plonky3InnerPcsV2 as Pcs<
            Plonky3ChallengeV2,
            Plonky3ChallengerV2,
        >>::get_evaluations_on_domain(&self.inner, prover_data, index, domain)
    }

    fn open(
        &self,
        data_with_points: Vec<(&Self::ProverData, Vec<Vec<Plonky3ChallengeV2>>)>,
        challenger: &mut Plonky3ChallengerV2,
    ) -> (OpenedValues<Plonky3ChallengeV2>, Self::Proof) {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::open(
            &self.inner,
            data_with_points,
            challenger,
        )
    }

    fn verify(
        &self,
        openings: Vec<(
            Self::Commitment,
            Vec<(
                Self::Domain,
                Vec<(Plonky3ChallengeV2, Vec<Plonky3ChallengeV2>)>,
            )>,
        )>,
        proof: &Self::Proof,
        challenger: &mut Plonky3ChallengerV2,
    ) -> Result<(), Self::Error> {
        <Plonky3InnerPcsV2 as Pcs<Plonky3ChallengeV2, Plonky3ChallengerV2>>::verify(
            &self.inner,
            openings,
            proof,
            challenger,
        )
    }
}

impl PcsGeneration<Plonky3StarkConfigV2, NativeProofV2> for Plonky3PcsV2 {
    fn generate_challenges(
        &self,
        config: &Plonky3StarkConfigV2,
        challenger: &mut Plonky3ChallengerV2,
        openings: &NativeComsV2,
        proof: &NativeProofV2,
        extra_params: Option<&[usize]>,
    ) -> Result<Vec<Plonky3ChallengeV2>, GenerationError> {
        <Plonky3InnerPcsV2 as PcsGeneration<
            Plonky3StarkConfigV2,
            NativeProofV2,
        >>::generate_challenges(
            &self.inner,
            config,
            challenger,
            openings,
            proof,
            extra_params,
        )
    }

    fn num_challenges(
        proof: &NativeProofV2,
        extra_params: Option<&[usize]>,
    ) -> Result<usize, GenerationError> {
        <Plonky3InnerPcsV2 as PcsGeneration<Plonky3StarkConfigV2, NativeProofV2>>::num_challenges(
            proof,
            extra_params,
        )
    }
}

impl
    RecursivePcs<
        Plonky3StarkConfigV2,
        BinaryInputProofV2,
        BinaryOpeningProofV2,
        CommitmentV2,
        DomainV2,
    > for Plonky3PcsV2
{
    type VerifierParams = z00z_plonky3_circuit_prover::FriVerifierParams;
    type RecursiveProof = BinaryOpeningProofV2;

    fn get_challenges_circuit<const WIDTH: usize, const RATE: usize, C: ChallengerPermConfig>(
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        challenger: &mut CircuitChallenger<WIDTH, RATE, C>,
        proof: &BinaryOpeningProofV2,
        _opened_values: &OpenedValuesTargetsWithLookups<Plonky3StarkConfigV2>,
        params: &Self::VerifierParams,
    ) -> Result<Vec<Target>, CircuitBuilderError> {
        let alpha = <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
            KoalaBear,
            Plonky3ChallengeV2,
        >>::sample_ext(challenger, circuit);
        let mut betas = Vec::with_capacity(proof.commit_phase_commits.len());
        for (commitment, witness) in proof
            .commit_phase_commits
            .iter()
            .zip(&proof.commit_pow_witnesses)
        {
            <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::observe_slice(challenger, circuit, &commitment.to_observation_targets());
            <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::check_pow_witness(
                challenger, circuit, params.commit_pow_bits, witness.witness
            )?;
            betas.push(<CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::sample_ext(challenger, circuit));
        }
        <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
            KoalaBear,
            Plonky3ChallengeV2,
        >>::observe_ext_slice(challenger, circuit, &proof.final_poly);
        for log_arity in &proof.log_arities {
            let target =
                circuit.alloc_const(Plonky3ChallengeV2::from_usize(*log_arity), "FRI log arity");
            <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                KoalaBear,
                Plonky3ChallengeV2,
            >>::observe(challenger, circuit, target);
        }
        <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
            KoalaBear,
            Plonky3ChallengeV2,
        >>::check_pow_witness(
            challenger,
            circuit,
            params.query_pow_bits,
            proof.pow_witness.witness,
        )?;
        let mut challenges = Vec::with_capacity(1 + betas.len());
        challenges.push(alpha);
        challenges.extend(betas);
        Ok(challenges)
    }

    fn verify_circuit<const WIDTH: usize, const RATE: usize, C: ChallengerPermConfig>(
        &self,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        challenges: &[Target],
        challenger: &mut CircuitChallenger<WIDTH, RATE, C>,
        openings: &ComsWithOpeningsTargets<CommitmentV2, DomainV2>,
        proof: &BinaryOpeningProofV2,
        params: &Self::VerifierParams,
    ) -> Result<Vec<NonPrimitiveOpId>, VerificationError> {
        let num_betas = proof.commit_phase_commits.len();
        if challenges.len() != num_betas + 1 || proof.query_proofs.len() != params.num_queries {
            return Err(shape("binary W32 recursive FRI challenge shape mismatch"));
        }
        let total_reduction = proof
            .log_arities
            .iter()
            .try_fold(0usize, |sum, value| sum.checked_add(*value))
            .ok_or_else(|| shape("binary W32 recursive FRI reduction overflow"))?;
        let log_max_height = total_reduction
            .checked_add(params.log_final_poly_len)
            .and_then(|value| value.checked_add(params.log_blowup))
            .ok_or_else(|| shape("binary W32 recursive FRI height overflow"))?;
        if log_max_height > KoalaBear::TWO_ADICITY {
            return Err(shape(
                "binary W32 recursive FRI height exceeds field two-adicity",
            ));
        }
        let query_bits = (0..proof.query_proofs.len())
            .map(|_| {
                <CircuitChallenger<WIDTH, RATE, C> as RecursiveChallenger<
                    KoalaBear,
                    Plonky3ChallengeV2,
                >>::sample_bits(challenger, circuit, log_max_height)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| shape(format!("binary W32 query sampling failed: {error:?}")))?;
        let beta_targets = &challenges[1..];
        let op_ids = verify_fri_circuit(
            circuit,
            proof,
            challenges[0],
            beta_targets,
            &query_bits,
            openings,
            params.log_blowup,
            None,
        )?;
        if !op_ids.is_empty() {
            return Err(shape(
                "arithmetic-only recursive FRI emitted MMCS operations",
            ));
        }
        verify_binary_paths(
            circuit,
            proof,
            challenges[0],
            beta_targets,
            &query_bits,
            openings,
            params.log_blowup,
        )?;
        Ok(Vec::new())
    }

    fn selectors_at_point_circuit(
        &self,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        domain: &DomainV2,
        point: &Target,
    ) -> RecursiveLagrangeSelectors {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::selectors_at_point_circuit(&self.inner, circuit, domain, point)
    }

    fn evaluate_periodic_columns_at_point_circuit(
        &self,
        circuit: &mut CircuitBuilder<Plonky3ChallengeV2>,
        domain: &DomainV2,
        columns: &[Vec<KoalaBear>],
        point: Target,
    ) -> Result<Vec<Target>, VerificationError> {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::evaluate_periodic_columns_at_point_circuit(
            &self.inner, circuit, domain, columns, point
        )
    }

    fn create_disjoint_domain(&self, domain: DomainV2, degree: usize) -> DomainV2 {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::create_disjoint_domain(&self.inner, domain, degree)
    }

    fn split_domains(&self, domain: &DomainV2, degree: usize) -> Vec<DomainV2> {
        <Plonky3InnerPcsV2 as RecursivePcs<
            Plonky3StarkConfigV2,
            UpstreamInputProofV2,
            UpstreamOpeningProofV2,
            CommitmentV2,
            DomainV2,
        >>::split_domains(&self.inner, domain, degree)
    }

    fn log_size(&self, domain: &DomainV2) -> usize {
        domain.log_size()
    }

    fn first_point(&self, domain: &DomainV2) -> Plonky3ChallengeV2 {
        domain.first_point().into()
    }

    fn get_fri_random_opened_values(_proof: &BinaryOpeningProofV2) -> &[Vec<Vec<Vec<Target>>>] {
        &[]
    }
}

fn shape(message: impl Into<String>) -> VerificationError {
    VerificationError::InvalidProofShape(message.into())
}
