//! Canonical lookup-column packing for Z00Z Batch-STARK proofs.
//!
//! The upstream maximum packs a fraction column exactly to the upper edge of
//! the current quotient-degree bucket. Z00Z reserves one algebraic degree of
//! headroom so prover and native/recursive verifiers never depend on that
//! boundary case. This changes only auxiliary-column packing; it does not
//! change the AIR relation, transcript, FRI, hash, field, or security settings.

use alloc::vec::Vec;

use p3_air::symbolic::{AirLayout, SymbolicExpression, SymbolicExpressionExt};
use p3_air::{Air, BaseAir};
use p3_batch_stark::common::{GlobalPreprocessed, PreprocessedInstanceMeta};
use p3_batch_stark::symbolic::get_log_num_quotient_chunks;
use p3_batch_stark::{CommonData, ProverData, StarkGenericConfig, Val};
use p3_commit::Pcs;
use p3_field::{Algebra, BasedVectorSpace, ExtensionField, PrimeField};
use p3_lookup::{InteractionSymbolicBuilder, LogUpGadget, Lookups};
use p3_matrix::Matrix;

fn degree_budget(log_quotient_chunks: usize, is_zk: usize) -> usize {
    assert!(is_zk <= 1, "is_zk must be either 0 or 1");
    (1_usize << log_quotient_chunks).saturating_sub(is_zk)
}

/// Derive the canonical lookup layout with one degree of quotient headroom.
pub fn canonical_lookups_for_air<F, EF, A>(air: &A, is_zk: usize) -> Lookups<F>
where
    F: PrimeField,
    EF: ExtensionField<F>,
    A: BaseAir<F> + Air<InteractionSymbolicBuilder<F, EF>>,
    SymbolicExpressionExt<F, EF>: Algebra<SymbolicExpression<F>> + Algebra<EF>,
{
    let gadget = LogUpGadget::new();
    let unpacked = Lookups::<F>::from_air::<EF, A>(air);
    let log_chunks = get_log_num_quotient_chunks::<F, EF, A, LogUpGadget>(
        air,
        AirLayout::from_air(air),
        &unpacked,
        is_zk,
        &gadget,
    );
    let packed = unpacked.pack_same_bus(&gadget, degree_budget(log_chunks, is_zk));

    debug_assert_eq!(
        get_log_num_quotient_chunks::<F, EF, A, LogUpGadget>(
            air,
            AirLayout::from_air(air),
            &packed,
            is_zk,
            &gadget,
        ),
        log_chunks,
        "canonical lookup packing must preserve the quotient chunk count",
    );
    packed
}

/// Build prover data and replace upstream edge packing with the canonical layout.
pub fn canonical_prover_data_from_airs_and_degrees<SC, A>(
    config: &SC,
    airs: &[A],
    trace_ext_degree_bits: &[usize],
) -> ProverData<SC>
where
    SC: StarkGenericConfig,
    Val<SC>: PrimeField,
    SC::Challenge: BasedVectorSpace<Val<SC>>,
    A: BaseAir<Val<SC>> + Air<InteractionSymbolicBuilder<Val<SC>, SC::Challenge>>,
    SymbolicExpressionExt<Val<SC>, SC::Challenge>:
        Algebra<SymbolicExpression<Val<SC>>> + Algebra<SC::Challenge>,
{
    let mut data = ProverData::from_airs_and_degrees(config, airs, trace_ext_degree_bits);
    data.common.lookups = airs
        .iter()
        .map(|air| canonical_lookups_for_air::<Val<SC>, SC::Challenge, A>(air, config.is_zk()))
        .collect();
    data
}

/// Derive verifier-common data while consuming the AIRs table by table.
///
/// This is the bounded common-data path for large recursive verifier circuits.
/// The upstream convenience constructor first clones every preprocessed matrix
/// out of borrowed AIRs and then retains the AIR-owned copies while building all
/// LDEs. Consuming each AIR immediately after extracting its matrix preserves the
/// exact matrix order, domains, commitment, and canonical lookup layout without
/// overlapping both complete representations. The PCS prover data is discarded
/// before returning because callers of this path need only the verifier binding.
pub fn canonical_common_data_from_owned_airs_and_degrees<SC, A>(
    config: &SC,
    airs: Vec<A>,
    trace_ext_degree_bits: &[usize],
) -> CommonData<SC>
where
    SC: StarkGenericConfig,
    Val<SC>: PrimeField,
    SC::Challenge: BasedVectorSpace<Val<SC>>,
    A: BaseAir<Val<SC>> + Air<InteractionSymbolicBuilder<Val<SC>, SC::Challenge>>,
    SymbolicExpressionExt<Val<SC>, SC::Challenge>:
        Algebra<SymbolicExpression<Val<SC>>> + Algebra<SC::Challenge>,
{
    assert_eq!(
        airs.len(),
        trace_ext_degree_bits.len(),
        "airs and trace_ext_degree_bits must have the same length"
    );

    // Lookups depend on the AIR relation, not on PCS prover data. Derive them
    // before consuming the AIR-owned preprocessed matrices.
    let lookups = airs
        .iter()
        .map(|air| canonical_lookups_for_air::<Val<SC>, SC::Challenge, A>(air, config.is_zk()))
        .collect();

    let pcs = config.pcs();
    let is_zk = config.is_zk();
    let mut instances = Vec::with_capacity(airs.len());
    let mut matrix_to_instance = Vec::new();
    let mut domains_and_traces = Vec::new();

    for (instance_index, (air, &extended_degree_bits)) in airs
        .into_iter()
        .zip(trace_ext_degree_bits.iter())
        .enumerate()
    {
        let base_degree_bits = extended_degree_bits
            .checked_sub(is_zk)
            .expect("extended degree includes the ZK padding bit");
        let Some(preprocessed) = air.preprocessed_trace() else {
            instances.push(None);
            continue;
        };
        let width = preprocessed.width();
        if width == 0 {
            instances.push(None);
            continue;
        }

        let degree = 1_usize << base_degree_bits;
        let extended_degree = 1_usize << extended_degree_bits;
        assert_eq!(
            preprocessed.height(),
            degree,
            "preprocessed trace height must equal trace degree for instance {instance_index}"
        );
        let matrix_index = domains_and_traces.len();
        domains_and_traces.push((pcs.natural_domain_for_degree(extended_degree), preprocessed));
        matrix_to_instance.push(instance_index);
        instances.push(Some(PreprocessedInstanceMeta {
            matrix_index,
            width,
            degree_bits: extended_degree_bits,
        }));
        // `air` is dropped here, before the next table is extracted.
    }

    let preprocessed = if domains_and_traces.is_empty() {
        None
    } else {
        let (commitment, prover_data) = pcs.commit_preprocessing(domains_and_traces);
        drop(prover_data);
        Some(GlobalPreprocessed {
            commitment,
            instances,
            matrix_to_instance,
        })
    };

    CommonData::new(preprocessed, lookups)
}

#[cfg(test)]
mod tests {
    use super::degree_budget;

    #[test]
    fn lookup_degree_budget_reserves_one_degree() {
        assert_eq!(degree_budget(3, 0), 8);
        assert_eq!(degree_budget(3, 1), 7);
    }
}
