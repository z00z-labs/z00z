//! Canonical uniqueness event projection and AIR row construction.

use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;

use super::plonky3_epoch_event_stream::transition_event_stream;
use super::plonky3_epoch_uniqueness_air::{
    UniquenessAirRoleV2, UniquenessRowV2, UniquenessTraceV2, CALL_FIELDS_V2, MAX_TRANSITIONS_V2,
    MIN_ROWS_V2, PUBLIC_FIELDS_V2, ROLE_COUNT_V2, ROW_FIELDS_V2,
};
use super::plonky3_epoch_uniqueness_range::UniquenessRangeQueryV2;
use super::{
    decode_flow_item, decode_uniqueness_sorted_row, EpochAirTableV2, EpochPreparedTransitionV2,
    EpochTraceChunkV2, RecursiveTraceOpcodeV2, UniquenessListKindV2, UniquenessPassV2,
    UniquenessSemanticRowV2, UniquenessSetKindV2, UNIQUENESS_SEMANTIC_ROW_BYTES_V2,
};
use crate::CheckpointError;

const TERMINAL_START_V2: usize = 36;
const TERMINAL_END_V2: usize = 68;

#[derive(Clone, Copy, Debug)]
struct SourceRowV2 {
    transition: usize,
    set: usize,
    position: usize,
    semantic: [u8; UNIQUENESS_SEMANTIC_ROW_BYTES_V2],
}

#[derive(Debug)]
pub(super) struct ParsedUniquenessWitnessV2 {
    replay: Vec<SourceRowV2>,
    commit_original: Vec<SourceRowV2>,
    commit_sorted: Vec<SourceRowV2>,
    product_original: Vec<SourceRowV2>,
    product_sorted: Vec<SourceRowV2>,
}

impl ParsedUniquenessWitnessV2 {
    #[must_use]
    pub(super) fn semantic_row_count(&self) -> usize {
        self.replay.len()
    }

    fn role_rows(&self, role: UniquenessAirRoleV2) -> &[SourceRowV2] {
        match role {
            UniquenessAirRoleV2::Replay => &self.replay,
            UniquenessAirRoleV2::CommitOriginal => &self.commit_original,
            UniquenessAirRoleV2::CommitSorted => &self.commit_sorted,
            UniquenessAirRoleV2::ProductOriginal => &self.product_original,
            UniquenessAirRoleV2::ProductSorted => &self.product_sorted,
        }
    }
}

#[derive(Debug)]
pub(super) struct UniquenessAirWitnessV2 {
    pub(super) traces: Vec<UniquenessTraceV2>,
    pub(super) range_queries: Vec<UniquenessRangeQueryV2>,
}

fn set_index(set: UniquenessSetKindV2) -> usize {
    match set {
        UniquenessSetKindV2::Spent => 0,
        UniquenessSetKindV2::Output => 1,
    }
}

fn source_row(
    transition: usize,
    set: UniquenessSetKindV2,
    position: usize,
    semantic: UniquenessSemanticRowV2,
) -> SourceRowV2 {
    SourceRowV2 {
        transition,
        set: set_index(set),
        position,
        semantic: semantic.canonical_bytes(),
    }
}

pub(super) fn parse(
    prepared: &[EpochPreparedTransitionV2],
) -> Result<ParsedUniquenessWitnessV2, CheckpointError> {
    if prepared.is_empty() || prepared.len() > MAX_TRANSITIONS_V2 {
        return Err(CheckpointError::Limit);
    }
    let mut parsed = ParsedUniquenessWitnessV2 {
        replay: Vec::new(),
        commit_original: Vec::new(),
        commit_sorted: Vec::new(),
        product_original: Vec::new(),
        product_sorted: Vec::new(),
    };
    for (transition, prepared) in prepared.iter().enumerate() {
        let stream = transition_event_stream(&prepared.material)?;
        let mut replay_positions = [0_usize; 2];
        let mut commit_original_positions = [0_usize; 2];
        let mut commit_sorted_positions = [0_usize; 2];
        let mut product_original_positions = [0_usize; 2];
        let mut product_sorted_position = 0_usize;
        for record in stream.records() {
            match record.opcode() {
                RecursiveTraceOpcodeV2::ReplayInput | RecursiveTraceOpcodeV2::ReplayOutput => {
                    let set = if record.opcode() == RecursiveTraceOpcodeV2::ReplayInput {
                        UniquenessSetKindV2::Spent
                    } else {
                        UniquenessSetKindV2::Output
                    };
                    let item = decode_flow_item(record.payload())?;
                    if item.terminal_id != record.object_id() {
                        return Err(CheckpointError::Canonical);
                    }
                    let set_index = set_index(set);
                    parsed.replay.push(source_row(
                        transition,
                        set,
                        replay_positions[set_index],
                        UniquenessSemanticRowV2::from_canonical_flow_item(&item),
                    ));
                    replay_positions[set_index] = replay_positions[set_index]
                        .checked_add(1)
                        .ok_or(CheckpointError::Overflow)?;
                }
                RecursiveTraceOpcodeV2::UniquenessSorted => {
                    let (pass, set, list, semantic) =
                        decode_uniqueness_sorted_row(record.payload())?;
                    let set_index = set_index(set);
                    let (target, position) = match (pass, list) {
                        (UniquenessPassV2::Commit, UniquenessListKindV2::Original) => {
                            let position = commit_original_positions[set_index];
                            commit_original_positions[set_index] =
                                position.checked_add(1).ok_or(CheckpointError::Overflow)?;
                            (&mut parsed.commit_original, position)
                        }
                        (UniquenessPassV2::Commit, UniquenessListKindV2::Sorted) => {
                            let position = commit_sorted_positions[set_index];
                            commit_sorted_positions[set_index] =
                                position.checked_add(1).ok_or(CheckpointError::Overflow)?;
                            (&mut parsed.commit_sorted, position)
                        }
                        (UniquenessPassV2::Product, UniquenessListKindV2::Original) => {
                            let position = product_original_positions[set_index];
                            product_original_positions[set_index] =
                                position.checked_add(1).ok_or(CheckpointError::Overflow)?;
                            (&mut parsed.product_original, position)
                        }
                        (UniquenessPassV2::Product, UniquenessListKindV2::Sorted) => {
                            let position = product_sorted_position;
                            product_sorted_position =
                                position.checked_add(1).ok_or(CheckpointError::Overflow)?;
                            (&mut parsed.product_sorted, position)
                        }
                    };
                    target.push(source_row(transition, set, position, semantic));
                }
                _ => {}
            }
        }
    }
    Ok(parsed)
}

pub(super) fn public_values(
    statement: &EpochTraceChunkV2,
) -> Result<Vec<KoalaBear>, CheckpointError> {
    if statement.inputs().table != EpochAirTableV2::Uniqueness {
        return Err(CheckpointError::Canonical);
    }
    let values = statement
        .canonical_bytes()
        .chunks_exact(2)
        .map(|limb| KoalaBear::from_u16(u16::from_le_bytes([limb[0], limb[1]])))
        .collect::<Vec<_>>();
    if values.len() != PUBLIC_FIELDS_V2 {
        return Err(CheckpointError::Invariant);
    }
    Ok(values)
}

fn same_group(role: UniquenessAirRoleV2, local: SourceRowV2, next: SourceRowV2) -> bool {
    local.transition == next.transition
        && (role == UniquenessAirRoleV2::ProductSorted || local.set == next.set)
}

fn first_strict_difference(local: SourceRowV2, next: SourceRowV2) -> Option<(usize, u8)> {
    let local_terminal = &local.semantic[TERMINAL_START_V2..TERMINAL_END_V2];
    let next_terminal = &next.semantic[TERMINAL_START_V2..TERMINAL_END_V2];
    local_terminal
        .iter()
        .zip(next_terminal)
        .enumerate()
        .find_map(|(index, (local, next))| {
            (local < next)
                .then_some((index, next.wrapping_sub(*local).wrapping_sub(1)))
                .or_else(|| (local > next).then_some((usize::MAX, 0)))
        })
        .filter(|(index, _)| *index != usize::MAX)
}

fn role_trace(
    statement_public: &[KoalaBear],
    role: UniquenessAirRoleV2,
    source: &[SourceRowV2],
    trace_rows: usize,
    range_queries: &mut Vec<UniquenessRangeQueryV2>,
) -> Result<UniquenessTraceV2, CheckpointError> {
    let mut rows = Vec::with_capacity(trace_rows);
    let mut running = 0_usize;
    for index in 0..trace_rows {
        let mut values = if index == 0 {
            statement_public.to_vec()
        } else {
            vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
        };
        values.push(KoalaBear::from_bool(index == 0));
        let row = source.get(index).copied();
        values.push(KoalaBear::from_bool(row.is_some()));
        if let Some(row) = row {
            running = running.checked_add(1).ok_or(CheckpointError::Overflow)?;
            for transition in 0..MAX_TRANSITIONS_V2 {
                values.push(KoalaBear::from_bool(transition == row.transition));
            }
            values.push(KoalaBear::from_bool(row.set == 0));
            values.push(KoalaBear::from_bool(row.set == 1));
            values.push(KoalaBear::from_usize(row.position));
            values.extend(row.semantic.into_iter().map(KoalaBear::from_u8));

            let next = source.get(index + 1).copied();
            let comparison = next.filter(|next| role.is_sorted() && same_group(role, row, *next));
            let same_terminal = comparison.is_some_and(|next| {
                role == UniquenessAirRoleV2::ProductSorted
                    && row.semantic[TERMINAL_START_V2..TERMINAL_END_V2]
                        == next.semantic[TERMINAL_START_V2..TERMINAL_END_V2]
            });
            values.push(KoalaBear::from_bool(same_terminal));
            let difference = comparison
                .filter(|_| !same_terminal)
                .and_then(|next| first_strict_difference(row, next));
            for terminal_index in 0..32 {
                values.push(KoalaBear::from_bool(
                    difference.is_some_and(|(index, _)| index == terminal_index),
                ));
            }
            let diff_minus_one = difference.map(|(_, value)| value).unwrap_or(0);
            values.push(KoalaBear::from_u8(diff_minus_one));
            if comparison.is_some() && !same_terminal {
                range_queries.push(UniquenessRangeQueryV2 {
                    byte_0: diff_minus_one,
                    byte_1: 0,
                    single_byte: true,
                });
            }
            if role == UniquenessAirRoleV2::Replay {
                range_queries.extend(row.semantic.chunks_exact(2).map(|pair| {
                    UniquenessRangeQueryV2 {
                        byte_0: pair[0],
                        byte_1: pair[1],
                        single_byte: false,
                    }
                }));
            }
        } else {
            values.extend(core::iter::repeat_n(KoalaBear::ZERO, ROW_FIELDS_V2 - 3));
        }
        values.push(KoalaBear::from_usize(running));
        if values.len() != CALL_FIELDS_V2 {
            return Err(CheckpointError::Invariant);
        }
        rows.push(UniquenessRowV2 { values });
    }
    Ok(UniquenessTraceV2 { role, rows })
}

pub(super) fn air_witness(
    statement: &EpochTraceChunkV2,
    parsed: &ParsedUniquenessWitnessV2,
) -> Result<UniquenessAirWitnessV2, CheckpointError> {
    let semantic_count =
        u64::try_from(parsed.semantic_row_count()).map_err(|_| CheckpointError::Limit)?;
    if statement.inputs().row_count != semantic_count {
        return Err(CheckpointError::Canonical);
    }
    let public = public_values(statement)?;
    let max_role_rows = UniquenessAirRoleV2::ALL
        .iter()
        .map(|role| parsed.role_rows(*role).len())
        .max()
        .unwrap_or(0);
    let trace_rows = max_role_rows.max(MIN_ROWS_V2).next_power_of_two();
    let mut range_queries = Vec::new();
    let mut traces = Vec::with_capacity(ROLE_COUNT_V2);
    for role in UniquenessAirRoleV2::ALL {
        traces.push(role_trace(
            &public,
            role,
            parsed.role_rows(role),
            trace_rows,
            &mut range_queries,
        )?);
    }
    Ok(UniquenessAirWitnessV2 {
        traces,
        range_queries,
    })
}
