//! Direct KoalaBear AIR for the canonical uniqueness transcript.
//!
//! Five role-separated tables prove source replay equality, exact original
//! commit/product copies, original-to-sorted permutation, and strict terminal
//! ordering. Semantic bytes are range-checked through a proof-bound packed
//! range table in the same Batch-STARK.

use core::any::Any;

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::{NonPrimitiveTrace, Traces};
use p3_field::extension::BinomialExtensionField;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_koala_bear::KoalaBear;
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};

use super::{
    Plonky3StarkConfigV2, EPOCH_CHUNK_BYTES_V2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2,
    UNIQUENESS_SEMANTIC_ROW_BYTES_V2,
};

const REPLAY_NPO_ID_V2: &str = "z00z/plonky3/epoch-uniqueness-replay/v2";
const COMMIT_ORIGINAL_NPO_ID_V2: &str = "z00z/plonky3/epoch-uniqueness-commit-original/v2";
const COMMIT_SORTED_NPO_ID_V2: &str = "z00z/plonky3/epoch-uniqueness-commit-sorted/v2";
const PRODUCT_ORIGINAL_NPO_ID_V2: &str = "z00z/plonky3/epoch-uniqueness-product-original/v2";
const PRODUCT_SORTED_NPO_ID_V2: &str = "z00z/plonky3/epoch-uniqueness-product-sorted/v2";

const REPLAY_COMMIT_BUS_V2: &str = "z00z/plonky3/epoch-uniqueness-replay-commit/v2";
const COMMIT_PERMUTATION_BUS_V2: &str = "z00z/plonky3/epoch-uniqueness-commit-permutation/v2";
const ORIGINAL_PRODUCT_BUS_V2: &str = "z00z/plonky3/epoch-uniqueness-original-product/v2";
const SORTED_PRODUCT_BUS_V2: &str = "z00z/plonky3/epoch-uniqueness-sorted-product/v2";
pub(super) const RANGE_BUS_V2: &str = "z00z/plonky3/epoch-uniqueness-packed-range/v2";

pub(super) const ROLE_COUNT_V2: usize = 5;
pub(super) const MIN_ROWS_V2: usize = 32;
pub(super) const MAX_TRANSITIONS_V2: usize = EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2 as usize;
pub(super) const STATEMENT_LIMBS_V2: usize = EPOCH_CHUNK_BYTES_V2 / core::mem::size_of::<u16>();
pub(super) const PUBLIC_FIELDS_V2: usize = STATEMENT_LIMBS_V2;
const PUBLIC_ROW_COUNT_OFFSET_V2: usize = 21;

const HEADER_ACTIVE_OFFSET_V2: usize = 0;
const ACTIVE_OFFSET_V2: usize = 1;
const TRANSITION_SELECTOR_OFFSET_V2: usize = 2;
const SET_SELECTOR_OFFSET_V2: usize = TRANSITION_SELECTOR_OFFSET_V2 + MAX_TRANSITIONS_V2;
const POSITION_OFFSET_V2: usize = SET_SELECTOR_OFFSET_V2 + 2;
const SEMANTIC_OFFSET_V2: usize = POSITION_OFFSET_V2 + 1;
const SAME_TERMINAL_OFFSET_V2: usize = SEMANTIC_OFFSET_V2 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2;
const DIFF_SELECTOR_OFFSET_V2: usize = SAME_TERMINAL_OFFSET_V2 + 1;
const TERMINAL_BYTES_V2: usize = 32;
const DIFF_MINUS_ONE_OFFSET_V2: usize = DIFF_SELECTOR_OFFSET_V2 + TERMINAL_BYTES_V2;
pub(super) const RUNNING_ROW_COUNT_OFFSET_V2: usize = DIFF_MINUS_ONE_OFFSET_V2 + 1;
pub(super) const ROW_FIELDS_V2: usize = RUNNING_ROW_COUNT_OFFSET_V2 + 1;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

const DEFINITION_START_V2: usize = 0;
const TERMINAL_START_V2: usize = 36;
const TERMINAL_END_V2: usize = TERMINAL_START_V2 + TERMINAL_BYTES_V2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UniquenessAirRoleV2 {
    Replay,
    CommitOriginal,
    CommitSorted,
    ProductOriginal,
    ProductSorted,
}

impl UniquenessAirRoleV2 {
    pub(super) const ALL: [Self; ROLE_COUNT_V2] = [
        Self::Replay,
        Self::CommitOriginal,
        Self::CommitSorted,
        Self::ProductOriginal,
        Self::ProductSorted,
    ];

    pub(super) fn npo_type(self) -> NpoTypeId {
        NpoTypeId::new(match self {
            Self::Replay => REPLAY_NPO_ID_V2,
            Self::CommitOriginal => COMMIT_ORIGINAL_NPO_ID_V2,
            Self::CommitSorted => COMMIT_SORTED_NPO_ID_V2,
            Self::ProductOriginal => PRODUCT_ORIGINAL_NPO_ID_V2,
            Self::ProductSorted => PRODUCT_SORTED_NPO_ID_V2,
        })
    }

    pub(super) const fn is_sorted(self) -> bool {
        matches!(self, Self::CommitSorted | Self::ProductSorted)
    }

    const fn uses_global_product_order(self) -> bool {
        matches!(self, Self::ProductSorted)
    }
}

#[derive(Clone, Debug)]
pub(super) struct UniquenessRowV2 {
    pub(super) values: Vec<KoalaBear>,
}

#[derive(Clone, Debug)]
pub(super) struct UniquenessTraceV2 {
    pub(super) role: UniquenessAirRoleV2,
    pub(super) rows: Vec<UniquenessRowV2>,
}

impl NonPrimitiveTrace<KoalaBear> for UniquenessTraceV2 {
    fn op_type(&self) -> NpoTypeId {
        self.role.npo_type()
    }

    fn rows(&self) -> usize {
        self.rows.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn NonPrimitiveTrace<KoalaBear>> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Copy, Debug)]
struct UniquenessAirV2 {
    role: UniquenessAirRoleV2,
}

impl UniquenessAirV2 {
    const fn new(role: UniquenessAirRoleV2) -> Self {
        Self { role }
    }

    fn trace_to_matrix(rows: &[UniquenessRowV2]) -> RowMajorMatrix<KoalaBear> {
        RowMajorMatrix::new(
            rows.iter()
                .flat_map(|row| row.values[PUBLIC_FIELDS_V2..].iter().copied())
                .collect(),
            ROW_FIELDS_V2,
        )
    }
}

impl<F: Field> BaseAir<F> for UniquenessAirV2 {
    fn width(&self) -> usize {
        ROW_FIELDS_V2
    }

    fn num_public_values(&self) -> usize {
        PUBLIC_FIELDS_V2
    }

    fn main_next_row_columns(&self) -> Vec<usize> {
        (0..ROW_FIELDS_V2).collect()
    }
}

fn field<AB: AirBuilder>(row: &[AB::Var], offset: usize) -> AB::Expr {
    row[offset].into()
}

fn semantic_fields<AB: AirBuilder>(
    row: &[AB::Var],
    transition: AB::Expr,
    set: AB::Expr,
    with_position: bool,
) -> Vec<AB::Expr> {
    let mut fields =
        Vec::with_capacity(2 + usize::from(with_position) + UNIQUENESS_SEMANTIC_ROW_BYTES_V2);
    fields.push(transition);
    fields.push(set);
    if with_position {
        fields.push(field::<AB>(row, POSITION_OFFSET_V2));
    }
    fields.extend(
        (0..UNIQUENESS_SEMANTIC_ROW_BYTES_V2)
            .map(|index| field::<AB>(row, SEMANTIC_OFFSET_V2 + index)),
    );
    fields
}

impl<AB> Air<AB> for UniquenessAirV2
where
    AB: AirBuilder + InteractionBuilder,
    AB::F: Field,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.current_slice();
        let next = main.next_slice();
        let public = builder
            .public_values()
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<AB::Expr>>();
        let one = AB::Expr::ONE;
        let active = field::<AB>(local, ACTIVE_OFFSET_V2);
        let next_active = field::<AB>(next, ACTIVE_OFFSET_V2);

        let header_active = field::<AB>(local, HEADER_ACTIVE_OFFSET_V2);
        builder.assert_bool(header_active.clone());
        builder.when_first_row().assert_one(header_active);
        builder
            .when_transition()
            .assert_zero(field::<AB>(next, HEADER_ACTIVE_OFFSET_V2));
        builder.assert_bool(active.clone());
        builder
            .when_transition()
            .assert_zero(next_active.clone() * (one.clone() - active.clone()));

        let transition_selectors = (0..MAX_TRANSITIONS_V2)
            .map(|index| field::<AB>(local, TRANSITION_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let next_transition_selectors = (0..MAX_TRANSITIONS_V2)
            .map(|index| field::<AB>(next, TRANSITION_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let set_selectors = (0..2)
            .map(|index| field::<AB>(local, SET_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        let next_set_selectors = (0..2)
            .map(|index| field::<AB>(next, SET_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        for selector in transition_selectors.iter().chain(set_selectors.iter()) {
            builder.assert_bool(selector.clone());
        }
        builder.assert_eq(
            transition_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            active.clone(),
        );
        builder.assert_eq(
            set_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            active.clone(),
        );

        let transition_index = transition_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(index)
            });
        let set_index = set_selectors[1].clone();
        let same_transition = transition_selectors
            .iter()
            .zip(&next_transition_selectors)
            .fold(AB::Expr::ZERO, |sum, (local, next)| {
                sum + local.clone() * next.clone()
            });
        let same_set = set_selectors
            .iter()
            .zip(&next_set_selectors)
            .fold(AB::Expr::ZERO, |sum, (local, next)| {
                sum + local.clone() * next.clone()
            });
        // Both dot products are already Boolean: each selector vector is
        // Boolean and constrained to be either all-zero or exactly one-hot.
        // Re-asserting Booleanity would square a degree-two expression without
        // strengthening the relation.

        for (current, selector) in transition_selectors.iter().enumerate() {
            for earlier in next_transition_selectors.iter().take(current) {
                builder
                    .when_transition()
                    .assert_zero(selector.clone() * earlier.clone());
            }
        }
        if !self.role.uses_global_product_order() {
            builder.when_transition().assert_zero(
                same_transition.clone() * set_selectors[1].clone() * next_set_selectors[0].clone(),
            );
        }

        let same_group = if self.role.uses_global_product_order() {
            same_transition.clone()
        } else {
            same_transition.clone() * same_set.clone()
        };
        // A product of the derived Boolean selectors is Boolean by construction.
        let position = field::<AB>(local, POSITION_OFFSET_V2);
        let next_position = field::<AB>(next, POSITION_OFFSET_V2);
        builder.when_first_row().assert_zero(position.clone());
        {
            let mut transition = builder.when_transition();
            transition
                .assert_zero(same_group.clone() * (next_position.clone() - position - one.clone()));
            transition.assert_zero((next_active.clone() - same_group.clone()) * next_position);
        }

        let replay_with_position =
            semantic_fields::<AB>(local, transition_index.clone(), set_index.clone(), true);
        let permutation =
            semantic_fields::<AB>(local, transition_index.clone(), set_index.clone(), false);
        match self.role {
            UniquenessAirRoleV2::Replay => {
                builder.push_interaction(
                    REPLAY_COMMIT_BUS_V2,
                    replay_with_position,
                    Count::bounded(active.clone(), 1),
                );
                for pair in 0..(UNIQUENESS_SEMANTIC_ROW_BYTES_V2 / 2) {
                    builder.push_interaction(
                        RANGE_BUS_V2,
                        vec![
                            field::<AB>(local, SEMANTIC_OFFSET_V2 + pair * 2),
                            field::<AB>(local, SEMANTIC_OFFSET_V2 + pair * 2 + 1),
                            AB::Expr::ZERO,
                        ],
                        Count::bounded(-active.clone(), 1),
                    );
                }
            }
            UniquenessAirRoleV2::CommitOriginal => {
                builder.push_interaction(
                    REPLAY_COMMIT_BUS_V2,
                    replay_with_position.clone(),
                    Count::bounded(-active.clone(), 1),
                );
                builder.push_interaction(
                    COMMIT_PERMUTATION_BUS_V2,
                    permutation.clone(),
                    Count::bounded(active.clone(), 1),
                );
                builder.push_interaction(
                    ORIGINAL_PRODUCT_BUS_V2,
                    replay_with_position,
                    Count::bounded(active.clone(), 1),
                );
            }
            UniquenessAirRoleV2::CommitSorted => {
                builder.push_interaction(
                    COMMIT_PERMUTATION_BUS_V2,
                    permutation.clone(),
                    Count::bounded(-active.clone(), 1),
                );
                builder.push_interaction(
                    SORTED_PRODUCT_BUS_V2,
                    permutation,
                    Count::bounded(active.clone(), 1),
                );
            }
            UniquenessAirRoleV2::ProductOriginal => {
                builder.push_interaction(
                    ORIGINAL_PRODUCT_BUS_V2,
                    replay_with_position,
                    Count::bounded(-active.clone(), 1),
                );
            }
            UniquenessAirRoleV2::ProductSorted => {
                builder.push_interaction(
                    SORTED_PRODUCT_BUS_V2,
                    permutation,
                    Count::bounded(-active.clone(), 1),
                );
            }
        }

        let same_terminal = field::<AB>(local, SAME_TERMINAL_OFFSET_V2);
        builder.assert_bool(same_terminal.clone());
        let comparison_pair = if self.role.is_sorted() {
            same_group
        } else {
            AB::Expr::ZERO
        };
        if self.role.uses_global_product_order() {
            builder.assert_zero(same_terminal.clone() * (one.clone() - comparison_pair.clone()));
            builder.assert_zero(same_terminal.clone() * set_selectors[1].clone());
            builder
                .assert_zero(same_terminal.clone() * (next_set_selectors[1].clone() - one.clone()));
            for index in DEFINITION_START_V2..TERMINAL_END_V2 {
                builder.assert_zero(
                    same_terminal.clone()
                        * (field::<AB>(local, SEMANTIC_OFFSET_V2 + index)
                            - field::<AB>(next, SEMANTIC_OFFSET_V2 + index)),
                );
            }
        } else {
            builder.assert_zero(same_terminal.clone());
        }
        let strict_comparison = comparison_pair - same_terminal;
        // `same_terminal` is Boolean and constrained to be a subset of the
        // Boolean comparison selector, so their difference is also Boolean.

        let diff_selectors = (0..TERMINAL_BYTES_V2)
            .map(|index| field::<AB>(local, DIFF_SELECTOR_OFFSET_V2 + index))
            .collect::<Vec<_>>();
        for selector in &diff_selectors {
            builder.assert_bool(selector.clone());
        }
        builder.assert_eq(
            diff_selectors
                .iter()
                .cloned()
                .fold(AB::Expr::ZERO, |sum, value| sum + value),
            strict_comparison.clone(),
        );
        let diff_minus_one = field::<AB>(local, DIFF_MINUS_ONE_OFFSET_V2);
        builder.assert_zero((one.clone() - strict_comparison.clone()) * diff_minus_one.clone());
        for (difference_index, selector) in diff_selectors.iter().enumerate() {
            for prior in 0..difference_index {
                builder.assert_zero(
                    selector.clone()
                        * (field::<AB>(local, SEMANTIC_OFFSET_V2 + TERMINAL_START_V2 + prior)
                            - field::<AB>(next, SEMANTIC_OFFSET_V2 + TERMINAL_START_V2 + prior)),
                );
            }
            builder.assert_zero(
                selector.clone()
                    * (field::<AB>(
                        next,
                        SEMANTIC_OFFSET_V2 + TERMINAL_START_V2 + difference_index,
                    ) - field::<AB>(
                        local,
                        SEMANTIC_OFFSET_V2 + TERMINAL_START_V2 + difference_index,
                    ) - one.clone()
                        - diff_minus_one.clone()),
            );
        }
        if self.role.is_sorted() {
            builder.push_interaction(
                RANGE_BUS_V2,
                vec![diff_minus_one, AB::Expr::ZERO, one.clone()],
                Count::bounded(-strict_comparison, 1),
            );
        }

        let running = field::<AB>(local, RUNNING_ROW_COUNT_OFFSET_V2);
        let next_running = field::<AB>(next, RUNNING_ROW_COUNT_OFFSET_V2);
        builder
            .when_first_row()
            .assert_eq(running.clone(), active.clone());
        builder
            .when_transition()
            .assert_eq(next_running, running.clone() + next_active);
        let public_count = public[PUBLIC_ROW_COUNT_OFFSET_V2].clone()
            + public[PUBLIC_ROW_COUNT_OFFSET_V2 + 1].clone() * AB::Expr::from_u64(65_536);
        builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 2].clone());
        builder.assert_zero(public[PUBLIC_ROW_COUNT_OFFSET_V2 + 3].clone());
        builder.when_last_row().assert_eq(running, public_count);

        let inactive = one - active;
        for offset in TRANSITION_SELECTOR_OFFSET_V2..RUNNING_ROW_COUNT_OFFSET_V2 {
            builder.assert_zero(inactive.clone() * field::<AB>(local, offset));
        }
    }
}

impl BatchAir<Plonky3StarkConfigV2> for UniquenessAirV2 {}

#[derive(Clone, Copy, Debug)]
pub(super) struct UniquenessProverV2 {
    role: UniquenessAirRoleV2,
}

impl UniquenessProverV2 {
    pub(super) const fn new(role: UniquenessAirRoleV2) -> Self {
        Self { role }
    }

    fn batch_instance(
        &self,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<UniquenessTraceV2>(&self.role.npo_type())?;
        if trace.role != self.role
            || trace.rows.len() < MIN_ROWS_V2
            || !trace.rows.len().is_power_of_two()
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        Some(BatchTableInstance {
            op_type: self.role.npo_type(),
            air: DynamicAirEntry::new(Box::new(UniquenessAirV2::new(self.role))),
            trace: UniquenessAirV2::trace_to_matrix(&trace.rows),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: trace.rows.len(),
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for UniquenessProverV2 {
    fn op_type(&self) -> NpoTypeId {
        self.role.npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        self.batch_instance(traces)
    }

    fn batch_instance_d2(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 2>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d4(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 4>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d6(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 6>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d8(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 8>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_air_from_table_entry(
        &self,
        _config: &Plonky3StarkConfigV2,
        degree: usize,
        circuit_extension_degree: u32,
        entry: &NonPrimitiveTableEntry<Plonky3StarkConfigV2>,
    ) -> Result<DynamicAirEntry<Plonky3StarkConfigV2>, String> {
        if degree != 1
            || circuit_extension_degree != 1
            || entry.op_type != self.role.npo_type()
            || entry.rows < MIN_ROWS_V2
            || !entry.rows.is_power_of_two()
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
        {
            return Err("epoch uniqueness table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(UniquenessAirV2::new(
            self.role,
        ))))
    }
}

pub(super) fn check_constraints(trace: &UniquenessTraceV2, expected_public: &[KoalaBear]) {
    p3_air::check_constraints(
        &UniquenessAirV2::new(trace.role),
        &UniquenessAirV2::trace_to_matrix(&trace.rows),
        expected_public,
    );
}

#[cfg(test)]
#[path = "plonky3_epoch_uniqueness_air_tests.rs"]
mod tests;
