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
use z00z_crypto::{CheckpointSha256BlockStreamV2, CheckpointShaRole};
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchAir, BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking,
    TableProver,
};

use super::plonky3_epoch_semantic_source_air::{
    SOURCE_NET_EFFECT_BYTE_BUS_V2, SOURCE_REPLAY_SEMANTIC_BYTE_BUS_V2,
    SOURCE_UNIQUENESS_PAYLOAD_BYTE_BUS_V2,
};
use super::plonky3_epoch_sha256_columns::{SemanticShaJobKindV2, SEMANTIC_SHA_RAW_BYTE_BUS_V2};
use super::{
    Plonky3StarkConfigV2, EPOCH_CHUNK_BYTES_V2, EPOCH_TRANSITIONS_PER_TRACE_CHUNK_V2,
    UNIQUENESS_PRECOMMIT_VERSION_V2, UNIQUENESS_SEMANTIC_ROW_BYTES_V2,
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
pub(super) const PUBLIC_SLICE_START_OFFSET_V2: usize = STATEMENT_LIMBS_V2;
pub(super) const PUBLIC_SLICE_LEN_OFFSET_V2: usize = PUBLIC_SLICE_START_OFFSET_V2 + 1;
pub(super) const PUBLIC_SLICE_ROW_COUNT_OFFSET_V2: usize = PUBLIC_SLICE_LEN_OFFSET_V2 + 1;
pub(super) const PUBLIC_FIELDS_V2: usize = PUBLIC_SLICE_ROW_COUNT_OFFSET_V2 + 4;

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
const NET_PAIR_SECOND_OFFSET_V2: usize = DIFF_MINUS_ONE_OFFSET_V2 + 1;
const NET_EFFECT_SELECTOR_OFFSET_V2: usize = NET_PAIR_SECOND_OFFSET_V2 + 1;
const NET_EFFECT_SELECTOR_COUNT_V2: usize = 4;
const NET_HASH_DIFF_SELECTOR_OFFSET_V2: usize =
    NET_EFFECT_SELECTOR_OFFSET_V2 + NET_EFFECT_SELECTOR_COUNT_V2;
const NET_HASH_DIFFERENCE_OFFSET_V2: usize = NET_HASH_DIFF_SELECTOR_OFFSET_V2 + TERMINAL_BYTES_V2;
const NET_HASH_DIFF_INVERSE_OFFSET_V2: usize = NET_HASH_DIFFERENCE_OFFSET_V2 + 1;
const NET_EFFECT_POSITION_OFFSET_V2: usize = NET_HASH_DIFF_INVERSE_OFFSET_V2 + 1;
pub(super) const RUNNING_ROW_COUNT_OFFSET_V2: usize = NET_EFFECT_POSITION_OFFSET_V2 + 1;
pub(super) const ROW_FIELDS_V2: usize = RUNNING_ROW_COUNT_OFFSET_V2 + 1;
pub(super) const CALL_FIELDS_V2: usize = PUBLIC_FIELDS_V2 + ROW_FIELDS_V2;

const DEFINITION_START_V2: usize = 0;
const TERMINAL_START_V2: usize = 36;
const TERMINAL_END_V2: usize = TERMINAL_START_V2 + TERMINAL_BYTES_V2;
const VALUE_HASH_START_V2: usize = TERMINAL_END_V2;
const VALUE_HASH_END_V2: usize = VALUE_HASH_START_V2 + TERMINAL_BYTES_V2;

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

        let local_transition_index = transition_selectors
            .iter()
            .enumerate()
            .fold(AB::Expr::ZERO, |sum, (index, selector)| {
                sum + selector.clone() * AB::Expr::from_usize(index)
            });
        // The witness lanes are compact physical lanes. The public slice
        // descriptor restores the canonical chunk slot before every
        // cross-table interaction, so an upper proof cannot be replayed as a
        // lower proof.
        let transition_index =
            local_transition_index + public[PUBLIC_SLICE_START_OFFSET_V2].clone();
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
            transition.assert_zero(
                same_group.clone() * (next_position.clone() - position.clone() - one.clone()),
            );
            transition.assert_zero((next_active.clone() - same_group.clone()) * next_position);
        }

        let replay_with_position =
            semantic_fields::<AB>(local, transition_index.clone(), set_index.clone(), true);
        let permutation =
            semantic_fields::<AB>(local, transition_index.clone(), set_index.clone(), false);

        if self.role != UniquenessAirRoleV2::Replay {
            let (class_base, pass, list) = match self.role {
                UniquenessAirRoleV2::Replay => unreachable!(),
                UniquenessAirRoleV2::CommitOriginal => (0, 0, 0),
                UniquenessAirRoleV2::CommitSorted => (2, 0, 1),
                UniquenessAirRoleV2::ProductOriginal => (4, 1, 0),
                UniquenessAirRoleV2::ProductSorted => (6, 1, 1),
            };
            let class = AB::Expr::from_usize(class_base) + set_index.clone();
            let payload_bytes = [
                AB::Expr::from_u64(u64::from(UNIQUENESS_PRECOMMIT_VERSION_V2)),
                AB::Expr::from_usize(pass),
                set_index.clone(),
                AB::Expr::from_usize(list),
            ]
            .into_iter()
            .chain(
                (0..UNIQUENESS_SEMANTIC_ROW_BYTES_V2)
                    .map(|index| field::<AB>(local, SEMANTIC_OFFSET_V2 + index)),
            );
            for (payload_index, payload_byte) in payload_bytes.enumerate() {
                builder.push_interaction(
                    SOURCE_UNIQUENESS_PAYLOAD_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        class.clone(),
                        position.clone(),
                        AB::Expr::from_usize(payload_index),
                        payload_byte,
                    ],
                    -Count::bounded(active.clone(), 1),
                );
            }
        }

        if matches!(
            self.role,
            UniquenessAirRoleV2::CommitOriginal | UniquenessAirRoleV2::CommitSorted
        ) {
            let sorted = usize::from(self.role == UniquenessAirRoleV2::CommitSorted);
            let roles = if sorted == 0 {
                [
                    CheckpointShaRole::SpentOriginalIds,
                    CheckpointShaRole::OutputOriginalIds,
                ]
            } else {
                [
                    CheckpointShaRole::SpentSortedIds,
                    CheckpointShaRole::OutputSortedIds,
                ]
            };
            let first_row_part = roles
                .map(|role| CheckpointSha256BlockStreamV2::framed_role_prefix(role).len() + 8 + 4);
            let row_start = set_selectors[0].clone() * AB::Expr::from_usize(first_row_part[0])
                + set_selectors[1].clone() * AB::Expr::from_usize(first_row_part[1])
                + position.clone() * AB::Expr::from_usize(8 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2);
            let job_id = set_index.clone() * AB::Expr::from_u64(2) + AB::Expr::from_usize(sorted);
            for (offset, byte) in u64::try_from(UNIQUENESS_SEMANTIC_ROW_BYTES_V2)
                .expect("semantic row length fits u64")
                .to_le_bytes()
                .into_iter()
                .enumerate()
            {
                builder.push_interaction(
                    SEMANTIC_SHA_RAW_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        AB::Expr::from_u64(u64::from(SemanticShaJobKindV2::UniquenessList as u8)),
                        job_id.clone(),
                        row_start.clone() + AB::Expr::from_usize(offset),
                        AB::Expr::from_u64(u64::from(byte)),
                    ],
                    Count::bounded(active.clone(), 1),
                );
            }
            for semantic_index in 0..UNIQUENESS_SEMANTIC_ROW_BYTES_V2 {
                builder.push_interaction(
                    SEMANTIC_SHA_RAW_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        AB::Expr::from_u64(u64::from(SemanticShaJobKindV2::UniquenessList as u8)),
                        job_id.clone(),
                        row_start.clone() + AB::Expr::from_usize(8 + semantic_index),
                        field::<AB>(local, SEMANTIC_OFFSET_V2 + semantic_index),
                    ],
                    Count::bounded(active.clone(), 1),
                );
            }
        }

        match self.role {
            UniquenessAirRoleV2::Replay => {
                for semantic_index in 0..UNIQUENESS_SEMANTIC_ROW_BYTES_V2 {
                    builder.push_interaction(
                        SOURCE_REPLAY_SEMANTIC_BYTE_BUS_V2,
                        vec![
                            transition_index.clone(),
                            set_index.clone(),
                            position.clone(),
                            AB::Expr::from_usize(semantic_index),
                            field::<AB>(local, SEMANTIC_OFFSET_V2 + semantic_index),
                        ],
                        Count::bounded(-active.clone(), 1),
                    );
                }
                builder.push_interaction(
                    REPLAY_COMMIT_BUS_V2,
                    replay_with_position,
                    Count::bounded(active.clone(), 1),
                );
                for pair in 0..(UNIQUENESS_SEMANTIC_ROW_BYTES_V2 / 2) {
                    builder.push_interaction(
                        RANGE_BUS_V2,
                        vec![
                            transition_index.clone(),
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
        let strict_comparison = comparison_pair - same_terminal.clone();
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
                vec![
                    transition_index.clone(),
                    diff_minus_one,
                    AB::Expr::ZERO,
                    one.clone(),
                ],
                Count::bounded(-strict_comparison, 1),
            );
        }

        if self.role == UniquenessAirRoleV2::ProductSorted {
            let net_pair_second = field::<AB>(local, NET_PAIR_SECOND_OFFSET_V2);
            let next_net_pair_second = field::<AB>(next, NET_PAIR_SECOND_OFFSET_V2);
            builder.assert_bool(net_pair_second.clone());
            builder.assert_zero(net_pair_second.clone() * (one.clone() - active.clone()));
            builder.assert_zero(net_pair_second.clone() * (one.clone() - set_index.clone()));
            builder
                .when_first_row()
                .assert_zero(net_pair_second.clone());
            builder
                .when_transition()
                .assert_eq(next_net_pair_second, same_terminal.clone());

            let net_emit = active.clone() - net_pair_second;
            let net_singleton = net_emit.clone() - same_terminal.clone();
            let net_effect_selectors = (0..NET_EFFECT_SELECTOR_COUNT_V2)
                .map(|index| field::<AB>(local, NET_EFFECT_SELECTOR_OFFSET_V2 + index))
                .collect::<Vec<_>>();
            for selector in &net_effect_selectors {
                builder.assert_bool(selector.clone());
            }
            let net_delete = net_effect_selectors[0].clone();
            let net_insert = net_effect_selectors[1].clone();
            let net_replace = net_effect_selectors[2].clone();
            let net_unchanged = net_effect_selectors[3].clone();
            builder.assert_eq(
                net_effect_selectors
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, value| sum + value),
                net_emit.clone(),
            );
            builder.assert_eq(
                net_delete.clone(),
                net_singleton.clone() * (one.clone() - set_index.clone()),
            );
            builder.assert_eq(net_insert.clone(), net_singleton * set_index.clone());
            builder.assert_eq(
                net_replace.clone() + net_unchanged.clone(),
                same_terminal.clone(),
            );

            let net_hash_diff_selectors = (0..TERMINAL_BYTES_V2)
                .map(|index| field::<AB>(local, NET_HASH_DIFF_SELECTOR_OFFSET_V2 + index))
                .collect::<Vec<_>>();
            for selector in &net_hash_diff_selectors {
                builder.assert_bool(selector.clone());
            }
            builder.assert_eq(
                net_hash_diff_selectors
                    .iter()
                    .cloned()
                    .fold(AB::Expr::ZERO, |sum, value| sum + value),
                net_replace.clone(),
            );
            let mut selected_difference = AB::Expr::ZERO;
            for (difference_index, selector) in net_hash_diff_selectors.iter().enumerate() {
                for prior in 0..difference_index {
                    builder.assert_zero(
                        selector.clone()
                            * (field::<AB>(next, SEMANTIC_OFFSET_V2 + VALUE_HASH_START_V2 + prior)
                                - field::<AB>(
                                    local,
                                    SEMANTIC_OFFSET_V2 + VALUE_HASH_START_V2 + prior,
                                )),
                    );
                }
                selected_difference += selector.clone()
                    * (field::<AB>(
                        next,
                        SEMANTIC_OFFSET_V2 + VALUE_HASH_START_V2 + difference_index,
                    ) - field::<AB>(
                        local,
                        SEMANTIC_OFFSET_V2 + VALUE_HASH_START_V2 + difference_index,
                    ));
            }
            for index in VALUE_HASH_START_V2..VALUE_HASH_END_V2 {
                builder.assert_zero(
                    net_unchanged.clone()
                        * (field::<AB>(next, SEMANTIC_OFFSET_V2 + index)
                            - field::<AB>(local, SEMANTIC_OFFSET_V2 + index)),
                );
            }
            let net_hash_difference = field::<AB>(local, NET_HASH_DIFFERENCE_OFFSET_V2);
            let net_hash_diff_inverse = field::<AB>(local, NET_HASH_DIFF_INVERSE_OFFSET_V2);
            builder.assert_eq(net_hash_difference.clone(), selected_difference);
            builder.assert_eq(
                net_hash_difference * net_hash_diff_inverse.clone(),
                net_replace.clone(),
            );
            builder.assert_zero((one.clone() - net_replace.clone()) * net_hash_diff_inverse);

            let net_effect_position = field::<AB>(local, NET_EFFECT_POSITION_OFFSET_V2);
            let next_net_effect_position = field::<AB>(next, NET_EFFECT_POSITION_OFFSET_V2);
            builder
                .when_first_row()
                .assert_zero(net_effect_position.clone());
            {
                let mut transition = builder.when_transition();
                transition.assert_zero(
                    same_transition.clone()
                        * (next_net_effect_position.clone()
                            - net_effect_position.clone()
                            - net_emit.clone()),
                );
                transition.assert_zero(
                    (next_active.clone() - same_transition.clone()) * next_net_effect_position,
                );
            }

            let net_kind = net_delete.clone()
                + net_insert.clone() * AB::Expr::from_u64(2)
                + net_replace.clone() * AB::Expr::from_u64(3)
                + net_unchanged.clone() * AB::Expr::from_u64(4);
            for (payload_index, payload_byte) in [
                (
                    0,
                    AB::Expr::from_u64(u64::from(UNIQUENESS_PRECOMMIT_VERSION_V2)),
                ),
                (1, net_kind),
            ] {
                builder.push_interaction(
                    SOURCE_NET_EFFECT_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        net_effect_position.clone(),
                        AB::Expr::from_usize(payload_index),
                        payload_byte,
                    ],
                    Count::bounded(-net_emit.clone(), 1),
                );
            }
            for semantic_index in DEFINITION_START_V2..TERMINAL_END_V2 {
                builder.push_interaction(
                    SOURCE_NET_EFFECT_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        net_effect_position.clone(),
                        AB::Expr::from_usize(2 + semantic_index),
                        field::<AB>(local, SEMANTIC_OFFSET_V2 + semantic_index),
                    ],
                    Count::bounded(-net_emit.clone(), 1),
                );
            }
            let net_old_gate = net_delete + net_replace.clone() + net_unchanged.clone();
            let net_pair_gate = net_replace + net_unchanged;
            for hash_index in 0..TERMINAL_BYTES_V2 {
                let current_hash =
                    field::<AB>(local, SEMANTIC_OFFSET_V2 + VALUE_HASH_START_V2 + hash_index);
                let next_hash =
                    field::<AB>(next, SEMANTIC_OFFSET_V2 + VALUE_HASH_START_V2 + hash_index);
                builder.push_interaction(
                    SOURCE_NET_EFFECT_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        net_effect_position.clone(),
                        AB::Expr::from_usize(2 + VALUE_HASH_START_V2 + hash_index),
                        net_old_gate.clone() * current_hash.clone(),
                    ],
                    Count::bounded(-net_emit.clone(), 1),
                );
                builder.push_interaction(
                    SOURCE_NET_EFFECT_BYTE_BUS_V2,
                    vec![
                        transition_index.clone(),
                        net_effect_position.clone(),
                        AB::Expr::from_usize(2 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2 + hash_index),
                        net_insert.clone() * current_hash + net_pair_gate.clone() * next_hash,
                    ],
                    Count::bounded(-net_emit.clone(), 1),
                );
            }
        } else {
            for offset in NET_PAIR_SECOND_OFFSET_V2..RUNNING_ROW_COUNT_OFFSET_V2 {
                builder.assert_zero(field::<AB>(local, offset));
            }
        }

        let running = field::<AB>(local, RUNNING_ROW_COUNT_OFFSET_V2);
        let next_running = field::<AB>(next, RUNNING_ROW_COUNT_OFFSET_V2);
        builder
            .when_first_row()
            .assert_eq(running.clone(), active.clone());
        builder
            .when_transition()
            .assert_eq(next_running, running.clone() + next_active);
        let public_count = public[PUBLIC_SLICE_ROW_COUNT_OFFSET_V2].clone()
            + public[PUBLIC_SLICE_ROW_COUNT_OFFSET_V2 + 1].clone() * AB::Expr::from_u64(65_536);
        builder.assert_zero(public[PUBLIC_SLICE_ROW_COUNT_OFFSET_V2 + 2].clone());
        builder.assert_zero(public[PUBLIC_SLICE_ROW_COUNT_OFFSET_V2 + 3].clone());
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

#[cfg(test)]
#[path = "plonky3_epoch_uniqueness_air_tests.rs"]
mod tests;
