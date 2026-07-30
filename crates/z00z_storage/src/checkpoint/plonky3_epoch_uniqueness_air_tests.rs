use p3_air::symbolic::AirLayout;
use p3_air::DebugConstraintBuilder;
use p3_batch_stark::symbolic::{get_log_num_quotient_chunks, get_max_constraint_degree};
use p3_field::PrimeCharacteristicRing;
use p3_lookup::{LogUpGadget, LookupProtocol, Lookups};
use p3_matrix::dense::RowMajorMatrixView;
use p3_matrix::stack::VerticalPair;
use p3_matrix::Matrix;

use super::*;
use crate::checkpoint::plonky3::Plonky3ChallengeV2;
use z00z_plonky3_circuit_prover::batch_stark_prover::canonical_lookups_for_air;

fn replay_trace_with_one_active_row() -> (UniquenessTraceV2, Vec<KoalaBear>) {
    let mut public = vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2];
    public[PUBLIC_ROW_COUNT_OFFSET_V2] = KoalaBear::ONE;
    let mut rows = Vec::with_capacity(MIN_ROWS_V2);
    for row_index in 0..MIN_ROWS_V2 {
        let mut values = if row_index == 0 {
            public.clone()
        } else {
            vec![KoalaBear::ZERO; PUBLIC_FIELDS_V2]
        };
        let mut trace = vec![KoalaBear::ZERO; ROW_FIELDS_V2];
        trace[HEADER_ACTIVE_OFFSET_V2] = KoalaBear::from_bool(row_index == 0);
        trace[ACTIVE_OFFSET_V2] = KoalaBear::from_bool(row_index == 0);
        if row_index == 0 {
            trace[TRANSITION_SELECTOR_OFFSET_V2] = KoalaBear::ONE;
            trace[SET_SELECTOR_OFFSET_V2] = KoalaBear::ONE;
        }
        trace[RUNNING_ROW_COUNT_OFFSET_V2] = KoalaBear::ONE;
        values.extend(trace);
        rows.push(UniquenessRowV2 { values });
    }
    (
        UniquenessTraceV2 {
            role: UniquenessAirRoleV2::Replay,
            rows,
        },
        public,
    )
}

#[test]
fn uniqueness_replay_lookup_packing_preserves_quotient_bucket() {
    let air = UniquenessAirV2::new(UniquenessAirRoleV2::Replay);
    let layout = AirLayout::from_air::<KoalaBear>(&air);
    let gadget = LogUpGadget::new();
    let unpacked = Lookups::<KoalaBear>::from_air::<Plonky3ChallengeV2, _>(&air);
    let unpacked_log_chunks =
        get_log_num_quotient_chunks::<KoalaBear, Plonky3ChallengeV2, _, LogUpGadget>(
            &air, layout, &unpacked, 1, &gadget,
        );
    let unpacked_degree = get_max_constraint_degree::<KoalaBear, Plonky3ChallengeV2, _, LogUpGadget>(
        &air, layout, &unpacked, &gadget,
    );
    let unpacked_len = unpacked.len();
    let budget = 1_usize << unpacked_log_chunks;
    let packed = unpacked.pack_same_bus(&gadget, budget);
    let packed_log_chunks =
        get_log_num_quotient_chunks::<KoalaBear, Plonky3ChallengeV2, _, LogUpGadget>(
            &air, layout, &packed, 1, &gadget,
        );
    let packed_degree = get_max_constraint_degree::<KoalaBear, Plonky3ChallengeV2, _, LogUpGadget>(
        &air, layout, &packed, &gadget,
    );

    eprintln!(
        "uniqueness replay lookup layout: unpacked={} packed={} \
         unpacked_degree={} packed_degree={} quotient_log_chunks={}",
        unpacked_len,
        packed.len(),
        unpacked_degree,
        packed_degree,
        packed_log_chunks,
    );
    assert_eq!(unpacked_log_chunks, packed_log_chunks);
}

#[test]
fn uniqueness_replay_generated_logup_trace_satisfies_packed_constraints() {
    let (trace, public) = replay_trace_with_one_active_row();
    let air = UniquenessAirV2::new(UniquenessAirRoleV2::Replay);
    let main = UniquenessAirV2::trace_to_matrix(&trace.rows);
    let gadget = LogUpGadget::new();
    let lookups = canonical_lookups_for_air::<KoalaBear, Plonky3ChallengeV2, _>(&air, 1);
    let challenges = (0..lookups.len())
        .flat_map(|index| {
            [
                Plonky3ChallengeV2::from_u64(101 + index as u64),
                Plonky3ChallengeV2::from_u64(7),
            ]
        })
        .collect::<Vec<_>>();
    let (permutation, terminal) = gadget.generate_permutation::<Plonky3StarkConfigV2>(
        &main,
        &None,
        &public,
        &lookups,
        &challenges,
    );
    let permutation_values = [terminal.expect("lookup terminal").0];

    for row_index in 0..main.height() {
        let next_index = (row_index + 1) % main.height();
        let local = main.row_slice(row_index).expect("main row");
        let next = main.row_slice(next_index).expect("next main row");
        let permutation_local = permutation.row_slice(row_index).expect("permutation row");
        let permutation_next = permutation
            .row_slice(next_index)
            .expect("next permutation row");
        let main_rows = VerticalPair::new(
            RowMajorMatrixView::new_row(&*local),
            RowMajorMatrixView::new_row(&*next),
        );
        let preprocessed = VerticalPair::new(
            RowMajorMatrixView::<KoalaBear>::new(&[], 0),
            RowMajorMatrixView::<KoalaBear>::new(&[], 0),
        );
        let permutation_rows = VerticalPair::new(
            RowMajorMatrixView::new_row(&*permutation_local),
            RowMajorMatrixView::new_row(&*permutation_next),
        );
        let mut builder = DebugConstraintBuilder::new_with_permutation(
            row_index,
            main_rows,
            preprocessed,
            &public,
            KoalaBear::from_bool(row_index == 0),
            KoalaBear::from_bool(row_index == main.height() - 1),
            KoalaBear::from_bool(row_index != main.height() - 1),
            permutation_rows,
            &challenges,
            &permutation_values,
            &[],
        );
        gadget.eval_air_and_lookups(&air, &mut builder, &lookups);
        assert!(
            !builder.has_failures(),
            "packed LogUp constraints failed on row {row_index}: {}",
            builder.formatted_failures(),
        );
    }
}
