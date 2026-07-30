//! Canonical lookup-column packing for Z00Z Batch-STARK proofs.
//!
//! The upstream maximum packs a fraction column exactly to the upper edge of
//! the current quotient-degree bucket. Z00Z reserves one algebraic degree of
//! headroom so prover and native/recursive verifiers never depend on that
//! boundary case. This changes only auxiliary-column packing; it does not
//! change the AIR relation, transcript, FRI, hash, field, or security settings.

use p3_air::symbolic::{AirLayout, SymbolicExpression, SymbolicExpressionExt};
use p3_air::{Air, BaseAir};
use p3_batch_stark::symbolic::get_log_num_quotient_chunks;
use p3_batch_stark::{ProverData, StarkGenericConfig, Val};
use p3_field::{Algebra, BasedVectorSpace, ExtensionField, PrimeField};
use p3_lookup::{InteractionSymbolicBuilder, LogUpGadget, Lookups};

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

#[cfg(test)]
mod tests {
    use super::degree_budget;

    #[test]
    fn lookup_degree_budget_reserves_one_degree() {
        assert_eq!(degree_budget(3, 0), 8);
        assert_eq!(degree_budget(3, 1), 7);
    }
}
