//! # ALU AIR
//!
//! [`AluAir`] defines the unified AIR for proving arithmetic operations over both
//! base and extension fields.
//!
//! ## Operations
//!
//! Each row encodes one or more arithmetic constraints selected by preprocessed
//! selectors:
//!
//! | Operation      | Constraint                                 | Degree |
//! |----------------|--------------------------------------------|--------|
//! | **ADD**        | `a + b - out = 0`                          | 1      |
//! | **MUL**        | `a * b - out = 0`                          | 2      |
//! | **BOOL_CHECK** | `a * (a - 1) = 0`                          | 2      |
//! | **MUL_ADD**    | `a * b + c - out = 0`                      | 2      |
//! | **HORNER_ACC** | `prev_row_out * b + c - a - out = 0`       | 2      |
//!
//! All constraint degrees are ≤ 3 (after multiplying by a selector), compatible
//! with `log_blowup = 1`.
//!
//! ## Main trace layout
//!
//! Each lane occupies `4 * D` columns: `[a[D], b[D], c[D], out[D]]`.
//! After all lanes, there are `(num_int + 2 * (K_max - 1)) * D` **global** extra columns
//! on lane 0 for variable-arity packed HornerAcc (`K_max >= 2`, fixed per AIR):
//! `num_int = (K_max - 1) / 2` compressed intermediates `int_*[D]`, then `(a_t,c_t)` for
//! `t = 1..K_max-1` (operand pairs for packed steps after the first).
//!
//! Total main width = `lanes * 4D + (num_int + 2 * (K_max - 1) + 1) * D` (extra `+ D` is `b^2`
//! witness for lane 0, keeping packed Horner constraints at degree 3 with preprocessed selectors).
//!
//! ## Preprocessed trace layout
//!
//! Each lane occupies 13 columns (see [`AluPrepLaneCols`](super::alu_columns::AluPrepLaneCols)):
//!
//! | Offset | Name              | Purpose                                        |
//! |--------|-------------------|------------------------------------------------|
//! | 0      | `mult_a`          | Multiplicity for `a` (`-1` = reader, `0` = pad)|
//! | 1      | `sel_add_vs_mul`  | ADD selector                                   |
//! | 2      | `sel_bool`        | BOOL_CHECK selector                            |
//! | 3      | `sel_muladd`      | MUL_ADD selector                               |
//! | 4      | `sel_horner`      | HORNER_ACC selector                            |
//! | 5      | `a_idx`           | Witness index for `a` (D-scaled)               |
//! | 6      | `b_idx`           | Witness index for `b` (D-scaled)               |
//! | 7      | `c_idx`           | Witness index for `c` (D-scaled)               |
//! | 8      | `out_idx`         | Witness index for `out` (D-scaled)             |
//! | 9      | `mult_b`          | Multiplicity for `b`                           |
//! | 10     | `mult_out`        | Multiplicity for `out`                         |
//! | 11     | `a_is_reader`     | 1 if `a` reads from the WitnessChecks bus      |
//! | 12     | `c_is_reader`     | 1 if `c` reads from the WitnessChecks bus      |
//!
//! After all lanes, there are `(K_max - 1) + 4 * (K_max - 1)` **global** extra preprocessed
//! columns: `sel_k` for each arity `k = 2..K_max`, then for each step `t = 1..K_max-1` four
//! columns `a_t_idx`, `c_t_idx`, `a_t_reader`, `c_t_reader`
//! (see [`AluPackedHornerStepPrepCols`](super::alu_columns::AluPackedHornerStepPrepCols)).
//!
//! Total preprocessed width = `lanes * 13 + (K_max - 1) + 6 * (K_max - 1)`.
//!
//! ## K-step packed HornerAcc
//!
//! When HornerAcc operations are present, [`compute_schedule`] places Horner chains
//! on lane 0 and greedily packs each prefix of a chain into one compact
//! [`ScheduleEntry`] with arity `k ∈ {2..K_max}` (same `b` witness index,
//! contiguous indices). Remainder ops use single-step rows.
//!
//! Inter-row and intra-row constraints fold consecutive Horner steps in pairs (degree-3
//! where needed); see `eval` implementation for the exact selector layout per `k`.
//!
//! A leading zero-arity [`ScheduleEntry`] prevents bogus inter-row Horner on row 0.
//!
//! ## WitnessChecks bus
//!
//! Each lane contributes 4 lookups. Packed Horner adds `2 * (K - 1)` extra lookups
//! for `(a_t, c_t)`, `t = 1..K-1`. Total = `lanes * 4 + 2 * (K - 1)`.

#![allow(clippy::needless_range_loop)]

use alloc::vec;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};

use p3_air::{Air, AirBuilder, BaseAir, WindowAccess};
use p3_circuit::tables::AluTrace;
use p3_field::{BasedVectorSpace, Dup, Field, PrimeCharacteristicRing};
use p3_lookup::{Count, InteractionBuilder};
use p3_matrix::dense::RowMajorMatrix;
use tracing::instrument;

use super::alu_columns::{
    AluMainLaneCols, AluPackedHornerStepPrepCols, AluPrepLaneCols, PACKED_HORNER_STEP_PREP_WIDTH,
    PREP_LANE_WIDTH, alu_main_lane_width, extra_prep_a_idx_for_step, extra_prep_sel_k_idx,
    horner_extra_prep_width, num_horner_intermediates,
};

/// Compact entry in the HornerAcc lane schedule.
///
/// The upper byte stores the arity (`0` separator, `1` ordinary operation,
/// `>= 2` packed Horner prefix) and the remaining bits store the first
/// operation index. Keeping one machine word per entry is important for large
/// one-shot recursive circuits: the schedule remains live while PCS quotient
/// work runs, but its representation has no effect on AIR rows or transcripts.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScheduleEntry(usize);

#[derive(Debug, Clone, Copy)]
enum DecodedScheduleEntry {
    Op(usize),
    PackedHorner(usize, usize),
    Separator,
}

impl ScheduleEntry {
    const ARITY_BITS: u32 = 8;
    const ARITY_SHIFT: u32 = usize::BITS - Self::ARITY_BITS;
    pub(crate) const MAX_ARITY: usize = u8::MAX as usize;
    pub(crate) const MAX_INDEX: usize = (1usize << Self::ARITY_SHIFT) - 1;

    fn pack(first_index: usize, arity: usize) -> Self {
        assert!(
            first_index <= Self::MAX_INDEX,
            "ALU schedule index exceeds compact canonical bound"
        );
        assert!(
            arity <= Self::MAX_ARITY,
            "ALU packed-Horner arity exceeds compact canonical bound"
        );
        Self(first_index | (arity << Self::ARITY_SHIFT))
    }

    fn op(index: usize) -> Self {
        Self::pack(index, 1)
    }

    fn packed_horner(first_index: usize, arity: usize) -> Self {
        debug_assert!(arity >= 2);
        Self::pack(first_index, arity)
    }

    const fn separator() -> Self {
        Self(0)
    }

    const fn decode(self) -> DecodedScheduleEntry {
        let arity = self.0 >> Self::ARITY_SHIFT;
        let first_index = self.0 & Self::MAX_INDEX;
        match arity {
            0 => DecodedScheduleEntry::Separator,
            1 => DecodedScheduleEntry::Op(first_index),
            _ => DecodedScheduleEntry::PackedHorner(first_index, arity),
        }
    }
}

const _: () = assert!(
    core::mem::size_of::<ScheduleEntry>() == core::mem::size_of::<usize>(),
    "ALU schedule entry must remain one machine word"
);

/// How extension multiplication is reduced in the MUL / MUL_ADD / Horner paths.
#[derive(Clone, Copy, Debug)]
pub enum AluExtMulKind<F: Copy> {
    /// Base field only (`D == 1`).
    Base,
    /// Binomial extension `x^D = W` for `D > 1`.
    Binomial { w: F },
    /// Quintic trinomial `X^5 + X^2 - 1` (KoalaBear-style), `D == 5` only.
    QuinticTrinomial,
}

impl<F: Copy> AluExtMulKind<F> {
    /// Resolve the extension-multiplication reduction for degree `d`.
    ///
    /// `d == 1` is the base field; `d == 5` with `quintic_trinomial` uses the
    /// quintic trinomial reduction; every other degree is a binomial reduction
    /// `x^d = w` and therefore requires `w` to be present. Returns `None` when a
    /// binomial reduction is needed but `w` is `None`, so callers map that to
    /// their own missing-`W` error.
    ///
    /// This is the single source of truth for the `(degree, w, quintic flag)`
    /// trichotomy shared by the prove, native-verify, and recursive-verify paths.
    pub fn resolve(d: usize, w: Option<F>, quintic_trinomial: bool) -> Option<Self> {
        if d == 1 {
            Some(Self::Base)
        } else if d == 5 && quintic_trinomial {
            Some(Self::QuinticTrinomial)
        } else {
            w.map(|w| Self::Binomial { w })
        }
    }
}

/// AIR for proving unified arithmetic operations.
///
/// Supports ADD, MUL, BOOL_CHECK, and MUL_ADD operations with preprocessed selectors.
#[derive(Debug, Clone)]
pub struct AluAir<F: Copy, const D: usize = 1> {
    /// Total number of logical ALU operations in the trace.
    pub(crate) num_ops: usize,
    /// Number of independent operations packed per trace row.
    pub(crate) lanes: usize,
    pub(crate) ext_mul_kind: AluExtMulKind<F>,
    /// Flattened preprocessed values (selectors + indices), in original op order.
    pub(crate) preprocessed: Vec<F>,
    /// Minimum trace height (for FRI compatibility with higher log_final_poly_len).
    pub(crate) min_height: usize,
    /// Compact HornerAcc lane schedule. When present, ops are reordered so that HornerAcc
    /// chains occupy lane 0 in consecutive rows, with zero-separators between chains.
    schedule: Option<Vec<ScheduleEntry>>,
    /// Number of entries in the allocation-free one-shot schedule. When set,
    /// the canonical schedule is replayed directly from `preprocessed` instead
    /// of retaining one machine word per entry.
    streaming_schedule_entry_count: Option<usize>,
    /// Pack size K for compact packed-Horner schedule entries (>= 2).
    pub(crate) horner_packed_steps: usize,
    /// Precomputed preprocessed trace matrix, when the caller already built it for this
    /// `(preprocessed, lanes, horner_packed_steps)` (e.g. once per circuit shape). When set,
    /// [`Self::preprocessed_trace`] returns a clone of this instead of rebuilding from
    /// `schedule`.
    precomputed_prep_trace: Option<RowMajorMatrix<F>>,
}

impl<F: Field + PrimeCharacteristicRing + Copy, const D: usize> AluAir<F, D> {
    /// Core builder: no preprocessed data, default min height and packed-Horner length.
    pub const fn from_reduction(
        num_ops: usize,
        lanes: usize,
        ext_mul_kind: AluExtMulKind<F>,
    ) -> Self {
        Self {
            num_ops,
            lanes,
            ext_mul_kind,
            preprocessed: Vec::new(),
            min_height: 1,
            schedule: None,
            streaming_schedule_entry_count: None,
            horner_packed_steps: 2,
            precomputed_prep_trace: None,
        }
    }

    /// Core builder with preprocessed data, computing the packed-Horner schedule.
    pub fn from_reduction_with_preprocessed(
        num_ops: usize,
        lanes: usize,
        ext_mul_kind: AluExtMulKind<F>,
        preprocessed: Vec<F>,
        horner_packed_steps: usize,
    ) -> Self {
        let schedule = Self::compute_schedule(&preprocessed, lanes, horner_packed_steps);
        Self {
            num_ops,
            lanes,
            ext_mul_kind,
            preprocessed,
            min_height: 1,
            schedule,
            streaming_schedule_entry_count: None,
            horner_packed_steps,
            precomputed_prep_trace: None,
        }
    }

    /// Core builder with preprocessed data and an already-computed packed-Horner schedule.
    ///
    /// The schedule depends only on `preprocessed`, `lanes`, and `horner_packed_steps` (not on
    /// `D`), so callers that already computed it for the same `(lanes, horner_packed_steps)` —
    /// e.g. once per circuit shape — can pass it here to skip [`Self::compute_schedule`].
    pub(crate) const fn from_reduction_with_schedule(
        num_ops: usize,
        lanes: usize,
        ext_mul_kind: AluExtMulKind<F>,
        preprocessed: Vec<F>,
        horner_packed_steps: usize,
        schedule: Option<Vec<ScheduleEntry>>,
    ) -> Self {
        Self {
            num_ops,
            lanes,
            ext_mul_kind,
            preprocessed,
            min_height: 1,
            schedule,
            streaming_schedule_entry_count: None,
            horner_packed_steps,
            precomputed_prep_trace: None,
        }
    }

    /// Core builder for allocation-free one-shot Horner scheduling.
    pub(crate) const fn from_reduction_with_streaming_schedule(
        num_ops: usize,
        lanes: usize,
        ext_mul_kind: AluExtMulKind<F>,
        preprocessed: Vec<F>,
        horner_packed_steps: usize,
        streaming_schedule_entry_count: Option<usize>,
    ) -> Self {
        Self {
            num_ops,
            lanes,
            ext_mul_kind,
            preprocessed,
            min_height: 1,
            schedule: None,
            streaming_schedule_entry_count,
            horner_packed_steps,
            precomputed_prep_trace: None,
        }
    }

    /// Attach an already-built preprocessed trace matrix, so [`Self::preprocessed_trace`]
    /// returns a clone of it instead of rebuilding from `schedule`.
    ///
    /// The caller is responsible for ensuring `trace` matches this instance's
    /// `(preprocessed, lanes, horner_packed_steps, min_height)`.
    #[must_use]
    pub(crate) fn with_precomputed_prep_trace(mut self, trace: RowMajorMatrix<F>) -> Self {
        self.precomputed_prep_trace = Some(trace);
        self
    }

    /// Compute the packed-Horner lane schedule for the given preprocessed data.
    ///
    /// Exposed so callers can precompute and cache it once per circuit shape (it depends only
    /// on `preprocessed`, `lanes`, and `pack_k`, not on `D`) and reuse it via
    /// [`Self::from_reduction_with_schedule`] across every proof of that shape.
    pub(crate) fn compute_schedule_for(
        preprocessed: &[F],
        lanes: usize,
        pack_k: usize,
    ) -> Option<Vec<ScheduleEntry>> {
        Self::compute_schedule(preprocessed, lanes, pack_k)
    }

    /// Count the canonical Horner schedule without allocating it.
    pub(crate) fn streaming_schedule_entry_count_for(
        preprocessed: &[F],
        lanes: usize,
        pack_k: usize,
    ) -> Option<usize> {
        Self::visit_schedule(preprocessed, lanes, pack_k, |_| {})
    }

    /// Release the deterministic packed-Horner schedule across the one-shot
    /// main PCS commitment high-water mark.
    ///
    /// Materialized callers may rebuild it from the retained public columns.
    /// Streaming one-shot callers have no schedule allocation to release.
    pub(crate) fn release_one_shot_schedule(&mut self) {
        if self.streaming_schedule_entry_count.is_none() {
            self.schedule = None;
        }
    }

    /// Release raw preprocessing columns after their canonical PCS commitment
    /// and all trace matrices have been materialized for a one-shot proof.
    ///
    /// The one-shot Batch-STARK path reconstructs the committed preprocessing
    /// matrix from PCS prover data for LogUp. AIR constraints consume builder
    /// preprocessed values, not this rebuild-only source vector, so retaining it
    /// across the main LDE commitment is unnecessary allocation overlap.
    pub(crate) fn release_one_shot_preprocessed_source(&mut self) {
        self.preprocessed = Vec::new();
        self.precomputed_prep_trace = None;
    }

    /// Rebuild a materialized schedule released by [`Self::release_one_shot_schedule`].
    #[cfg(test)]
    pub(crate) fn rebuild_one_shot_schedule(&mut self) {
        if self.streaming_schedule_entry_count.is_none() {
            self.schedule =
                Self::compute_schedule(&self.preprocessed, self.lanes, self.horner_packed_steps);
        }
    }

    /// Construct a new `AluAir` for base-field operations (D=1).
    pub const fn new(num_ops: usize, lanes: usize) -> Self {
        assert!(lanes > 0, "lane count must be non-zero");
        assert!(
            D == 1,
            "Base-field constructor requires D == 1; use new_binomial or new_quintic_trinomial"
        );
        Self::from_reduction(num_ops, lanes, AluExtMulKind::Base)
    }

    /// Construct a new `AluAir` for base-field operations with preprocessed data.
    pub fn new_with_preprocessed(
        num_ops: usize,
        lanes: usize,
        preprocessed: Vec<F>,
        horner_packed_steps: usize,
    ) -> Self {
        assert!(lanes > 0, "lane count must be non-zero");
        assert!(
            D == 1,
            "Base-field constructor requires D == 1; use new_binomial_with_preprocessed or new_quintic_trinomial_with_preprocessed"
        );
        assert!(
            horner_packed_steps >= 2,
            "horner_packed_steps must be at least 2"
        );
        Self::from_reduction_with_preprocessed(
            num_ops,
            lanes,
            AluExtMulKind::Base,
            preprocessed,
            horner_packed_steps,
        )
    }

    /// Construct a new `AluAir` for binomial extension-field operations (D > 1).
    pub const fn new_binomial(num_ops: usize, lanes: usize, w: F) -> Self {
        assert!(lanes > 0, "lane count must be non-zero");
        assert!(D >= 2, "Binomial constructor requires D >= 2");
        Self::from_reduction(num_ops, lanes, AluExtMulKind::Binomial { w })
    }

    /// Construct a new `AluAir` for binomial extension-field operations with preprocessed data.
    pub fn new_binomial_with_preprocessed(
        num_ops: usize,
        lanes: usize,
        w: F,
        preprocessed: Vec<F>,
        horner_packed_steps: usize,
    ) -> Self {
        assert!(lanes > 0, "lane count must be non-zero");
        assert!(D >= 2, "Binomial constructor requires D >= 2");
        assert!(
            horner_packed_steps >= 2,
            "horner_packed_steps must be at least 2"
        );
        Self::from_reduction_with_preprocessed(
            num_ops,
            lanes,
            AluExtMulKind::Binomial { w },
            preprocessed,
            horner_packed_steps,
        )
    }

    /// Quintic trinomial extension (`X^5 + X^2 - 1`), `D = 5` only.
    pub const fn new_quintic_trinomial(num_ops: usize, lanes: usize) -> Self {
        assert!(lanes > 0, "lane count must be non-zero");
        assert!(D == 5, "Quintic trinomial ALU requires D = 5");
        Self::from_reduction(num_ops, lanes, AluExtMulKind::QuinticTrinomial)
    }

    /// Quintic trinomial extension with preprocessed columns, `D = 5` only.
    pub fn new_quintic_trinomial_with_preprocessed(
        num_ops: usize,
        lanes: usize,
        preprocessed: Vec<F>,
        horner_packed_steps: usize,
    ) -> Self {
        assert!(lanes > 0, "lane count must be non-zero");
        assert!(D == 5, "Quintic trinomial ALU requires D = 5");
        assert!(
            horner_packed_steps >= 2,
            "horner_packed_steps must be at least 2"
        );

        Self::from_reduction_with_preprocessed(
            num_ops,
            lanes,
            AluExtMulKind::QuinticTrinomial,
            preprocessed,
            horner_packed_steps,
        )
    }

    /// Set the minimum trace height for FRI compatibility.
    ///
    /// FRI requires: `log_trace_height > log_final_poly_len + log_blowup`
    /// So `min_height` should be >= `2^(log_final_poly_len + log_blowup + 1)`.
    pub const fn with_min_height(mut self, min_height: usize) -> Self {
        self.min_height = min_height;
        self
    }

    /// Override packed Horner chain length (default 2 from [`Self::new`] / [`Self::new_binomial`]).
    ///
    /// Batch verification rebuilds a symbolic ALU without committed preprocessed data; this must
    /// match [`crate::batch_stark_prover::TablePacking::horner_packed_steps`] from the proof.
    pub const fn with_horner_pack_k(mut self, k: usize) -> Self {
        assert!(k >= 2, "horner_packed_steps must be at least 2");
        self.horner_packed_steps = k;
        self
    }

    /// Number of main columns per lane: a[D], b[D], c[D], out[D]
    pub const fn lane_width() -> usize {
        alu_main_lane_width::<D>()
    }

    /// Total main trace width for this AIR instance.
    pub const fn total_width(&self) -> usize {
        let k = self.horner_packed_steps;
        let num_int = num_horner_intermediates(k);
        let extra = (num_int + 2 * (k - 1) + 1) * D;
        self.lanes * Self::lane_width() + extra
    }

    /// Number of preprocessed columns per lane (see [`AluPrepLaneCols`](super::alu_columns::AluPrepLaneCols)).
    pub const fn preprocessed_lane_width() -> usize {
        PREP_LANE_WIDTH
    }

    /// Total preprocessed width: per-lane base columns plus global packed Horner columns.
    pub const fn preprocessed_width(&self) -> usize {
        self.lanes * PREP_LANE_WIDTH + horner_extra_prep_width(self.horner_packed_steps)
    }

    /// Total entries in the scheduled trace (including separators).
    pub fn scheduled_entry_count(&self) -> usize {
        self.schedule.as_ref().map_or_else(
            || self.streaming_schedule_entry_count.unwrap_or(self.num_ops),
            Vec::len,
        )
    }

    /// Compute a lane schedule that places HornerAcc chains in lane 0.
    ///
    /// Returns `None` if no HornerAcc ops are present.
    /// Even with `lanes == 1`, scheduling is required: chains must start at
    /// row 0 so the cyclic wrap from the last (zero-padded) row provides
    /// `prev_out = 0`, and separators must appear between chains.
    #[instrument(skip_all, name = "AluAir::compute_schedule")]
    fn compute_schedule(
        preprocessed: &[F],
        lanes: usize,
        pack_k: usize,
    ) -> Option<Vec<ScheduleEntry>> {
        let mut schedule: Vec<ScheduleEntry> = Vec::new();
        let entry_count = Self::visit_schedule(preprocessed, lanes, pack_k, |entry| {
            schedule.push(entry);
        })?;
        debug_assert_eq!(schedule.len(), entry_count);
        Some(schedule)
    }

    /// Replay the canonical Horner lane schedule without retaining it.
    ///
    /// Both the reusable materialized schedule and the one-shot streaming path
    /// use this authority, so their row order is identical by construction.
    fn visit_schedule<V>(
        preprocessed: &[F],
        lanes: usize,
        pack_k: usize,
        mut visit: V,
    ) -> Option<usize>
    where
        V: FnMut(ScheduleEntry),
    {
        assert!(lanes > 0, "lane count must be non-zero");
        let num_ops = preprocessed.len() / PREP_LANE_WIDTH;
        if num_ops == 0 || !(0..num_ops).any(|index| Self::is_horner(preprocessed, index)) {
            return None;
        }

        let mut emitted = 0usize;
        let mut non_chain_cursor = 0usize;
        let fill_row = |visit: &mut V, emitted: &mut usize, non_chain_cursor: &mut usize| {
            while !emitted.is_multiple_of(lanes) {
                let entry = Self::next_non_horner_index(preprocessed, num_ops, non_chain_cursor)
                    .map_or_else(ScheduleEntry::separator, ScheduleEntry::op);
                visit(entry);
                *emitted += 1;
            }
        };

        // The leading separator makes the cyclic predecessor of the first
        // Horner chain canonical zero.
        visit(ScheduleEntry::separator());
        emitted += 1;
        fill_row(&mut visit, &mut emitted, &mut non_chain_cursor);

        let mut chain_cursor = 0usize;
        let mut chain_index = 0usize;
        while chain_cursor < num_ops {
            while chain_cursor < num_ops && !Self::is_horner(preprocessed, chain_cursor) {
                chain_cursor += 1;
            }
            if chain_cursor == num_ops {
                break;
            }
            let chain_start = chain_cursor;
            while chain_cursor < num_ops && Self::is_horner(preprocessed, chain_cursor) {
                chain_cursor += 1;
            }
            let chain_end = chain_cursor;

            if chain_index > 0 {
                fill_row(&mut visit, &mut emitted, &mut non_chain_cursor);
                visit(ScheduleEntry::separator());
                emitted += 1;
                fill_row(&mut visit, &mut emitted, &mut non_chain_cursor);
            }
            chain_index += 1;

            let mut first_index = chain_start;
            while first_index < chain_end {
                debug_assert_eq!(emitted % lanes, 0, "chain op not at lane 0");
                let k_try = chain_end.saturating_sub(first_index).min(pack_k);
                let mut best_k = 1usize;
                for k in (2..=k_try).rev() {
                    if Self::horner_range_shares_b_idx(preprocessed, first_index, k) {
                        best_k = k;
                        break;
                    }
                }
                let entry = if best_k >= 2 {
                    ScheduleEntry::packed_horner(first_index, best_k)
                } else {
                    ScheduleEntry::op(first_index)
                };
                visit(entry);
                emitted += 1;
                first_index += best_k;
                fill_row(&mut visit, &mut emitted, &mut non_chain_cursor);
            }
        }

        fill_row(&mut visit, &mut emitted, &mut non_chain_cursor);
        while let Some(index) =
            Self::next_non_horner_index(preprocessed, num_ops, &mut non_chain_cursor)
        {
            visit(ScheduleEntry::op(index));
            emitted += 1;
        }
        while !emitted.is_multiple_of(lanes) {
            visit(ScheduleEntry::separator());
            emitted += 1;
        }

        Some(emitted)
    }

    #[inline]
    fn is_horner(preprocessed: &[F], index: usize) -> bool {
        let plw = PREP_LANE_WIDTH;
        let prep: &AluPrepLaneCols<F> = preprocessed[index * plw..(index + 1) * plw].borrow();
        prep.sel_horner == F::ONE
    }

    fn next_non_horner_index(
        preprocessed: &[F],
        num_ops: usize,
        cursor: &mut usize,
    ) -> Option<usize> {
        while *cursor < num_ops {
            let index = *cursor;
            *cursor += 1;
            if !Self::is_horner(preprocessed, index) {
                return Some(index);
            }
        }
        None
    }

    /// Consecutive Horner ops use the same `b` witness index.
    fn horner_range_shares_b_idx(preprocessed: &[F], first_index: usize, len: usize) -> bool {
        if len == 0 {
            return true;
        }
        let plw = PREP_LANE_WIDTH;
        let prep0: &AluPrepLaneCols<F> =
            preprocessed[first_index * plw..(first_index + 1) * plw].borrow();
        let b0 = prep0.b_idx;
        (first_index..first_index + len).all(|index| {
            let prep: &AluPrepLaneCols<F> = preprocessed[index * plw..(index + 1) * plw].borrow();
            prep.b_idx == b0
        })
    }

    /// Write the 4 operands `[a, b, c, out]` of operation `op_idx` into `dst`
    /// starting at `cursor`, advancing it by `4 * D`.
    #[inline(always)]
    fn write_operands<ExtF: BasedVectorSpace<F>>(
        dst: &mut [F],
        cursor: &mut usize,
        trace: &AluTrace<ExtF>,
        op_idx: usize,
    ) {
        for operand in 0..4 {
            let coeffs = trace.values[op_idx][operand].as_basis_coefficients_slice();
            dst[*cursor..*cursor + D].copy_from_slice(coeffs);
            *cursor += D;
        }
    }

    /// Convert an `AluTrace` into a `RowMajorMatrix` suitable for the STARK prover.
    pub fn trace_to_matrix<ExtF: BasedVectorSpace<F> + Field>(
        &self,
        trace: &AluTrace<ExtF>,
        min_height: usize,
    ) -> RowMajorMatrix<F> {
        self.trace_to_matrix_with_lde_capacity(trace, min_height, 0)
    }

    /// Build the same canonical matrix while reserving its final PCS LDE
    /// capacity without extending its visible length.
    ///
    /// The radix-2 PCS grows the owned evaluation buffer by
    /// `1 << lde_added_bits`. Reserving that capacity at initial allocation
    /// prevents a multi-gigabyte realloc overlap in single-use workers; only
    /// the logical trace rows are initialized here, so matrix contents,
    /// dimensions, commitments, and transcripts are unchanged.
    #[instrument(skip_all, name = "AluAir::trace_to_matrix")]
    pub(crate) fn trace_to_matrix_with_lde_capacity<ExtF: BasedVectorSpace<F> + Field>(
        &self,
        trace: &AluTrace<ExtF>,
        min_height: usize,
        lde_added_bits: usize,
    ) -> RowMajorMatrix<F> {
        let lanes = self.lanes;
        assert!(lanes > 0, "lane count must be non-zero");

        let lane_width = Self::lane_width();
        let width = self.total_width();
        let entry_count = self.scheduled_entry_count();
        let row_count = entry_count.div_ceil(lanes);
        // Allocate the canonical power-of-two matrix once. Building the exact
        // logical height and then padding with `Vec::resize` can transiently
        // retain both multi-gigabyte buffers during realloc, even though every
        // appended field is deterministically zero. Writing into the final
        // zero-filled allocation preserves every row while removing that peak.
        let padded_height = row_count
            .max(1)
            .next_power_of_two()
            .max(min_height.next_power_of_two());
        let value_len = width
            .checked_mul(padded_height)
            .expect("ALU matrix length fits usize");
        let capacity = value_len
            .checked_shl(u32::try_from(lde_added_bits).expect("LDE capacity bits fit u32"))
            .expect("ALU LDE capacity fits usize");
        let mut values = Vec::with_capacity(capacity);
        values.resize(value_len, F::ZERO);

        if self.schedule.is_some() || self.streaming_schedule_entry_count.is_some() {
            let mut prev_lane0_out = [F::ZERO; D];
            let mut write_entry = |pos: usize, entry: ScheduleEntry| {
                let row = pos / lanes;
                let lane = pos % lanes;

                match entry.decode() {
                    DecodedScheduleEntry::Op(i) => {
                        let mut cursor = row * width + lane * lane_width;
                        Self::write_operands(&mut values, &mut cursor, trace, i);
                        if lane == 0 {
                            let out_start = row * width + 3 * D;
                            prev_lane0_out[..D].copy_from_slice(&values[out_start..out_start + D]);
                        }
                    }
                    DecodedScheduleEntry::PackedHorner(first_idx, actual_k) => {
                        let k = actual_k;
                        let k_max = self.horner_packed_steps;
                        let base = row * width + lane * lane_width;
                        let mut cursor = base;

                        for operand in 0..3 {
                            let coeffs =
                                trace.values[first_idx][operand].as_basis_coefficients_slice();
                            values[cursor..cursor + D].copy_from_slice(coeffs);
                            cursor += D;
                        }
                        let last = first_idx + k - 1;
                        let out_last = trace.values[last][3].as_basis_coefficients_slice();
                        values[cursor..cursor + D].copy_from_slice(out_last);

                        if lane == 0 {
                            let extra = row * width + self.lanes * lane_width;
                            let num_int = num_horner_intermediates(k_max);
                            let prev_ext =
                                ExtF::from_basis_coefficients_slice(&prev_lane0_out[..D]).unwrap();
                            let b = trace.values[first_idx][1];
                            let mut step = 0usize;
                            let mut acc = prev_ext;
                            for s in 0..num_int {
                                let i0 = first_idx + step;
                                let i1 = first_idx + step + 1;
                                let v0 = &trace.values[i0];
                                if i1 < first_idx + k {
                                    let v1 = &trace.values[i1];
                                    let o0 = acc * b + v0[2] - v0[0];
                                    acc = o0 * b + v1[2] - v1[0];
                                    step += 2;
                                } else {
                                    acc = acc * b + v0[2] - v0[0];
                                    step += 1;
                                }
                                let off = extra + s * D;
                                values[off..off + D]
                                    .copy_from_slice(acc.as_basis_coefficients_slice());
                            }
                            let ac_base = extra + num_int * D;
                            for t in 1..k {
                                let op_t = first_idx + t;
                                let a_t = trace.values[op_t][0].as_basis_coefficients_slice();
                                let c_t = trace.values[op_t][2].as_basis_coefficients_slice();
                                let off = ac_base + 2 * (t - 1) * D;
                                values[off..off + D].copy_from_slice(a_t);
                                values[off + D..off + 2 * D].copy_from_slice(c_t);
                            }
                            let b_sq_ext = b * b;
                            let b_sq_base = ac_base + 2 * (k_max - 1) * D;
                            values[b_sq_base..b_sq_base + D]
                                .copy_from_slice(b_sq_ext.as_basis_coefficients_slice());
                            let out_start = row * width + 3 * D;
                            prev_lane0_out[..D].copy_from_slice(&values[out_start..out_start + D]);
                        }
                    }
                    DecodedScheduleEntry::Separator => {
                        if lane == 0 {
                            prev_lane0_out = [F::ZERO; D];
                        }
                    }
                }
            };
            if let Some(ref schedule) = self.schedule {
                for (pos, entry) in schedule.iter().copied().enumerate() {
                    write_entry(pos, entry);
                }
            } else {
                let mut pos = 0usize;
                let visited = Self::visit_schedule(
                    &self.preprocessed,
                    self.lanes,
                    self.horner_packed_steps,
                    |entry| {
                        write_entry(pos, entry);
                        pos += 1;
                    },
                )
                .expect("streaming schedule count requires Horner entries");
                debug_assert_eq!(visited, entry_count);
                debug_assert_eq!(pos, entry_count);
            }
        } else {
            for op_idx in 0..trace.values.len() {
                let row = op_idx / lanes;
                let lane = op_idx % lanes;
                let mut cursor = row * width + lane * lane_width;
                Self::write_operands(&mut values, &mut cursor, trace, op_idx);
            }
        }

        RowMajorMatrix::new(values, width)
    }

    /// Build the preprocessed trace matrix with HornerAcc scheduling applied.
    ///
    /// Separator entries get multiplicity=0 (no lookups), all selectors/indices=0.
    fn build_scheduled_preprocessed_trace(&self) -> RowMajorMatrix<F> {
        let plw = PREP_LANE_WIDTH;
        let entry_count = self.scheduled_entry_count();
        let row_count = entry_count.div_ceil(self.lanes);
        let row_width = self.preprocessed_width();

        let mut values = F::zero_vec(row_count.max(1) * row_width);

        let mut write_entry = |pos: usize, entry: ScheduleEntry| {
            let row = pos / self.lanes;
            let lane = pos % self.lanes;
            let base = row * row_width + lane * plw;

            match entry.decode() {
                DecodedScheduleEntry::Op(i) => {
                    let src = &self.preprocessed[i * plw..(i + 1) * plw];
                    values[base..base + plw].copy_from_slice(src);
                }
                DecodedScheduleEntry::PackedHorner(first_idx, actual_k) => {
                    if lane == 0 {
                        let k = actual_k;
                        let k_max = self.horner_packed_steps;
                        let src0 = &self.preprocessed[first_idx * plw..(first_idx + 1) * plw];
                        let last = first_idx + k - 1;
                        let src_last = &self.preprocessed[last * plw..(last + 1) * plw];

                        values[base..base + plw].copy_from_slice(src0);

                        let mult_a_lane = {
                            let lane_prep: &mut AluPrepLaneCols<F> =
                                values[base..base + plw].borrow_mut();
                            let src_last_prep: &AluPrepLaneCols<F> = src_last.borrow();
                            lane_prep.out_idx = src_last_prep.out_idx;
                            lane_prep.mult_out = src_last_prep.mult_out;

                            lane_prep.mult_b *= F::from_usize(k);
                            lane_prep.mult_a
                        };

                        let extra_base = row * row_width + self.lanes * plw;
                        values[extra_base + extra_prep_sel_k_idx(k)] = F::ONE;
                        for t in 1..k {
                            let src_t: &AluPrepLaneCols<F> = self.preprocessed
                                [(first_idx + t) * plw..(first_idx + t + 1) * plw]
                                .borrow();
                            let p = extra_base + extra_prep_a_idx_for_step(t, k_max);
                            let step: &mut AluPackedHornerStepPrepCols<F> =
                                values[p..p + PACKED_HORNER_STEP_PREP_WIDTH].borrow_mut();
                            step.a_idx = src_t.a_idx;
                            step.c_idx = src_t.c_idx;
                            step.a_reader = src_t.a_is_reader;
                            step.c_reader = src_t.c_is_reader;
                            let on = if t < k { F::ONE } else { F::ZERO };
                            step.horner_lookup_mult_a = mult_a_lane * src_t.a_is_reader * on;
                            step.horner_lookup_mult_c = mult_a_lane * src_t.c_is_reader * on;
                        }
                    }
                }
                DecodedScheduleEntry::Separator => {}
            }
        };
        if let Some(ref schedule) = self.schedule {
            for (pos, entry) in schedule.iter().copied().enumerate() {
                write_entry(pos, entry);
            }
        } else {
            let mut pos = 0usize;
            let visited = Self::visit_schedule(
                &self.preprocessed,
                self.lanes,
                self.horner_packed_steps,
                |entry| {
                    write_entry(pos, entry);
                    pos += 1;
                },
            )
            .expect("streaming schedule count requires Horner entries");
            debug_assert_eq!(visited, entry_count);
            debug_assert_eq!(pos, entry_count);
        }

        let mut mat = RowMajorMatrix::new(values, row_width);
        mat.pad_to_min_power_of_two_height(self.min_height, F::ZERO);
        mat
    }
}

impl<F: Field + Copy, const D: usize> BaseAir<F> for AluAir<F, D> {
    fn width(&self) -> usize {
        self.total_width()
    }

    fn preprocessed_width(&self) -> usize {
        self.lanes * PREP_LANE_WIDTH + horner_extra_prep_width(self.horner_packed_steps)
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        if let Some(ref trace) = self.precomputed_prep_trace {
            return Some(trace.clone());
        }
        if self.schedule.is_some() || self.streaming_schedule_entry_count.is_some() {
            Some(self.build_scheduled_preprocessed_trace())
        } else {
            // No Horner scheduling: build the preprocessed trace at the base
            // width, then widen with zero columns for scheduling slots.
            let base_width = self.lanes * PREP_LANE_WIDTH;
            let mut mat =
                RowMajorMatrix::from_flat_padded(self.preprocessed.to_vec(), base_width, F::ZERO);
            mat.widen_right(horner_extra_prep_width(self.horner_packed_steps), F::ZERO);
            mat.pad_to_min_power_of_two_height(self.min_height, F::ZERO);
            Some(mat)
        }
    }
}

/// Compute `x * y` as a D-coefficient extension-field product, where
/// `w` is the binomial parameter (only used when `D > 1`).
///
/// When `D == 1` the `w` parameter is unused and all loops degenerate to a
/// single scalar multiply, so this is zero-cost for base-field AIRs.
#[inline]
fn ext_mul_binomial<AB: AirBuilder, const D: usize>(
    x: &[AB::Var],
    y: &[AB::Var],
    w: &Option<AB::Expr>,
) -> Vec<AB::Expr> {
    let mut acc = vec![AB::Expr::ZERO; D];
    for i in 0..D {
        for j in 0..D {
            let term = x[i] * y[j];
            let k = i + j;
            if k < D {
                acc[k] = acc[k].dup() + term;
            } else {
                acc[k - D] = acc[k - D].dup() + w.as_ref().unwrap().dup() * term;
            }
        }
    }
    acc
}

/// Extension multiply in `GF(p)[X]/(X^5 + X^2 - 1)` (KoalaBear-style), on `AB::Expr`.
#[inline]
fn ext_mul_quintic_trinomial<AB: AirBuilder>(x: &[AB::Var], y: &[AB::Var]) -> Vec<AB::Expr> {
    debug_assert_eq!(x.len(), 5);
    debug_assert_eq!(y.len(), 5);
    let xi = |i: usize| AB::Expr::from(x[i]);
    let yj = |j: usize| AB::Expr::from(y[j]);

    let c0 = xi(0) * yj(0);
    let c1 = xi(0) * yj(1) + xi(1) * yj(0);
    let c2 = xi(0) * yj(2) + xi(1) * yj(1) + xi(2) * yj(0);
    let c3 = xi(0) * yj(3) + xi(1) * yj(2) + xi(2) * yj(1) + xi(3) * yj(0);
    let c4 = xi(0) * yj(4) + xi(1) * yj(3) + xi(2) * yj(2) + xi(3) * yj(1) + xi(4) * yj(0);
    let c5 = xi(1) * yj(4) + xi(2) * yj(3) + xi(3) * yj(2) + xi(4) * yj(1);
    let c6 = xi(2) * yj(4) + xi(3) * yj(3) + xi(4) * yj(2);
    let c7 = xi(3) * yj(4) + xi(4) * yj(3);
    let c8 = xi(4) * yj(4);

    let c5_minus_c8 = c5 - c8.clone();

    vec![
        c0 + c5_minus_c8.clone(),
        c1 + c6.clone(),
        c2 - c5_minus_c8 + c7.clone(),
        c3 - c6 + c8,
        c4 - c7,
    ]
}

impl<AB: AirBuilder + InteractionBuilder, const D: usize> Air<AB> for AluAir<AB::F, D>
where
    AB::F: Field + Copy,
{
    fn eval(&self, builder: &mut AB) {
        // Emit cross-table interactions first so the borrow checker doesn't fight us when
        // building the row windows below.
        eval_alu_interactions::<AB, D>(self, builder);

        let main = builder.main();
        debug_assert_eq!(
            main.current_slice().len(),
            self.total_width(),
            "column width mismatch"
        );

        let local = main.current_slice();
        let next = main.next_slice();
        let lane_width = Self::lane_width();

        let preprocessed = builder.preprocessed().clone();
        let prep_local = preprocessed.current_slice();
        let prep_next = preprocessed.next_slice();

        let kind = self.ext_mul_kind;
        #[cfg(debug_assertions)]
        {
            match kind {
                AluExtMulKind::Base => debug_assert_eq!(D, 1, "Base ext_mul_kind requires D == 1"),
                AluExtMulKind::Binomial { .. } => debug_assert!(D >= 2, "Binomial requires D >= 2"),
                AluExtMulKind::QuinticTrinomial => {
                    debug_assert_eq!(D, 5, "QuinticTrinomial requires D == 5");
                }
            }
        }

        let w: Option<AB::Expr> = match kind {
            AluExtMulKind::Binomial { w } => Some(AB::Expr::from(w)),
            _ => None,
        };
        let ext_mul_lane = |x: &[AB::Var], y: &[AB::Var]| -> Vec<AB::Expr> {
            match kind {
                AluExtMulKind::QuinticTrinomial => ext_mul_quintic_trinomial::<AB>(x, y),
                _ => ext_mul_binomial::<AB, D>(x, y, &w),
            }
        };

        for lane in 0..self.lanes {
            let m = lane * lane_width;
            let p = lane * PREP_LANE_WIDTH;

            let lane_local: &AluMainLaneCols<_, D> = local[m..m + lane_width].borrow();
            let lane_next: &AluMainLaneCols<_, D> = next[m..m + lane_width].borrow();

            let prep_cur: &AluPrepLaneCols<_> = prep_local[p..p + PREP_LANE_WIDTH].borrow();
            let prep_n: &AluPrepLaneCols<_> = prep_next[p..p + PREP_LANE_WIDTH].borrow();

            let a = &lane_local.a;
            let b = &lane_local.b;
            let c = &lane_local.c;
            let out = &lane_local.out;

            let mult_a = prep_cur.mult_a;
            let sel_add = prep_cur.sel_add;
            let sel_bool = prep_cur.sel_bool;
            let sel_muladd = prep_cur.sel_muladd;
            let sel_horner = prep_cur.sel_horner;

            // `mult_a` is stored as -1 for active rows and 0 for padding (see
            // `common.rs` generator). Negate to recover a {0,1} active selector.
            // `sel_mul` is the residual after all other op selectors are subtracted:
            // active = sel_mul + sel_bool + sel_muladd + sel_horner + sel_add.
            let active = AB::Expr::ZERO - mult_a;
            let sel_mul = active - sel_bool - sel_muladd - sel_horner - sel_add;

            // ── ADD: a + b - out = 0 ────────────────────────────────────
            for i in 0..D {
                builder.assert_zero(sel_add * (a[i] + b[i] - out[i]));
            }

            // ── MUL: a * b - out = 0 ────────────────────────────────────
            let ab = ext_mul_lane(a, b);
            for i in 0..D {
                builder.assert_zero(sel_mul.dup() * (ab[i].dup() - out[i]));
            }

            // ── BOOL_CHECK: a[0]*(a[0]-1)=0, a[1..D]=0 ─────────────────
            let one = AB::Expr::ONE;
            builder.assert_zero(sel_bool * a[0] * (a[0] - one));
            for i in 1..D {
                builder.assert_zero(sel_bool * a[i]);
            }

            // ── MUL_ADD: a * b + c - out = 0 ────────────────────────────
            for i in 0..D {
                builder.assert_zero(sel_muladd * (ab[i].dup() + c[i] - out[i]));
            }

            // ── HORNER_ACC ───────────────────────────────────────────────
            let next_sel_horner = prep_n.sel_horner;

            let next_a = &lane_next.a;
            let next_b = &lane_next.b;
            let next_c = &lane_next.c;
            let next_out = &lane_next.out;

            let out_next_b = ext_mul_lane(out, next_b);

            let extra_main = self.lanes * lane_width;
            let extra_prep = self.lanes * PREP_LANE_WIDTH;
            let k_max = self.horner_packed_steps;
            let num_int = num_horner_intermediates(k_max);
            let extra_coeff_width = (num_int + 2 * (k_max - 1) + 1) * D;
            let has_extra_cols = extra_main + extra_coeff_width <= local.len()
                && extra_prep + horner_extra_prep_width(k_max) <= prep_local.len()
                && extra_prep + horner_extra_prep_width(k_max) <= prep_next.len();

            if lane == 0 && has_extra_cols {
                let next_int0 = &next[extra_main..extra_main + D];

                let mut any_packed_cur = AB::Expr::ZERO;
                for kk in 2..=k_max {
                    any_packed_cur += prep_local[extra_prep + extra_prep_sel_k_idx(kk)];
                }

                let mut any_packed_next = AB::Expr::ZERO;
                for kk in 2..=k_max {
                    any_packed_next += prep_next[extra_prep + extra_prep_sel_k_idx(kk)];
                }

                let next_sel_k2 = prep_next[extra_prep + extra_prep_sel_k_idx(2)];
                let mut sel_ge3_next = AB::Expr::ZERO;
                for kk in 3..=k_max {
                    sel_ge3_next += prep_next[extra_prep + extra_prep_sel_k_idx(kk)];
                }

                let ac_base = extra_main + num_int * D;
                let b_sq_base = ac_base + 2 * (k_max - 1) * D;
                let b_sq = &local[b_sq_base..b_sq_base + D];
                let b_sq_next = &next[b_sq_base..b_sq_base + D];
                let bb = ext_mul_lane(b, b);
                for i in 0..D {
                    builder.assert_zero(any_packed_cur.dup() * (b_sq[i] - bb[i].dup()));
                }

                let out_b_sq = ext_mul_lane(out, b_sq_next);
                let c0_b_next = ext_mul_lane(next_c, next_b);
                let a0_b_next = ext_mul_lane(next_a, next_b);

                let ac_base_next = extra_main + num_int * D;
                let off1 = ac_base_next;
                let a1_next = &next[off1..off1 + D];
                let c1_next = &next[off1 + D..off1 + 2 * D];

                // 1) Packed inter-row: fold first two steps of next row
                for i in 0..D {
                    let poly_i = out_b_sq[i].dup() + c0_b_next[i].dup() - a0_b_next[i].dup()
                        + c1_next[i]
                        - a1_next[i];
                    builder.assert_zero(next_sel_k2.dup() * (poly_i.dup() - next_out[i]));
                    builder.assert_zero(sel_ge3_next.dup() * (poly_i - next_int0[i]));
                }

                // 2) Single-step fallback: prev_out -> next_out
                let next_sel_single = next_sel_horner - any_packed_next;
                for i in 0..D {
                    builder.assert_zero(
                        next_sel_single.dup()
                            * (out_next_b[i].dup() + next_c[i] - next_a[i] - next_out[i]),
                    );
                }

                // Chain-head seed must be zero: a Horner chain's first step reads its
                // accumulator positionally from the preceding row's `out`, which is a
                // separator row. A separator has `mult_a = 0` whereas every real ALU op
                // has `mult_a = -1`, so `next_sel_horner * (mult_a + 1)` is `1` exactly at
                // a chain start (next is a Horner step, local is a separator) and `0`
                // mid-chain or after a real op. Pinning that `out` to zero prevents a
                // prover from injecting a nonzero accumulator, which would shift the
                // evaluated polynomial by `seed * b^n`.
                let chain_head_seed = next_sel_horner * (prep_cur.mult_a + AB::Expr::ONE);
                for i in 0..D {
                    builder.assert_zero(chain_head_seed.dup() * out[i]);
                }

                // 3) Intra-row packed legs (per arity selector sel_k, k >= 3)
                for kk in 3..=k_max {
                    let sel_kk = prep_local[extra_prep + extra_prep_sel_k_idx(kk)];
                    let mut s = 2usize;
                    let mut curr_int_slot = 0usize;
                    while s < kk {
                        let int_curr = &local
                            [extra_main + curr_int_slot * D..extra_main + (curr_int_slot + 1) * D];
                        let off_s = ac_base + 2 * (s - 1) * D;
                        let a_s = &local[off_s..off_s + D];
                        let c_s = &local[off_s + D..off_s + 2 * D];
                        if s + 1 < kk {
                            let off_sp1 = ac_base + 2 * s * D;
                            let a_sp1 = &local[off_sp1..off_sp1 + D];
                            let c_sp1 = &local[off_sp1 + D..off_sp1 + 2 * D];

                            let int_b_sq = ext_mul_lane(int_curr, b_sq);
                            let c_s_b = ext_mul_lane(c_s, b);
                            let a_s_b = ext_mul_lane(a_s, b);

                            if s + 2 >= kk {
                                for i in 0..D {
                                    let prod = int_b_sq[i].dup() + c_s_b[i].dup() - a_s_b[i].dup()
                                        + c_sp1[i]
                                        - a_sp1[i];
                                    builder.assert_zero(sel_kk * (prod - out[i]));
                                }
                            } else {
                                let int_next = &local[extra_main + (curr_int_slot + 1) * D
                                    ..extra_main + (curr_int_slot + 2) * D];
                                for i in 0..D {
                                    let prod = int_b_sq[i].dup() + c_s_b[i].dup() - a_s_b[i].dup()
                                        + c_sp1[i]
                                        - a_sp1[i];
                                    builder.assert_zero(sel_kk * (prod - int_next[i]));
                                }
                                curr_int_slot += 1;
                            }
                            s += 2;
                        } else {
                            let int_b = ext_mul_lane(int_curr, b);
                            for i in 0..D {
                                builder.assert_zero(
                                    sel_kk * (int_b[i].dup() + c_s[i] - a_s[i] - out[i]),
                                );
                            }
                            s += 1;
                        }
                    }
                }
            } else {
                for i in 0..D {
                    builder.assert_zero(
                        next_sel_horner
                            * (out_next_b[i].dup() + next_c[i] - next_a[i] - next_out[i]),
                    );
                }
            }
        }
    }
}

/// Push all WitnessChecks bus interactions for one row of [`AluAir`]: four per-lane sends
/// (a, b, c, out) and 2·(K_max − 1) packed-Horner sends.
fn eval_alu_interactions<AB: AirBuilder + InteractionBuilder, const D: usize>(
    air: &AluAir<AB::F, D>,
    builder: &mut AB,
) where
    AB::F: Field + Copy,
{
    let main = builder.main();
    let main_local = main.current_slice();
    let prep = builder.preprocessed().clone();
    let prep_local = prep.current_slice();

    let lane_w = AluAir::<AB::F, D>::lane_width();

    for lane in 0..air.lanes {
        let main_off = lane * lane_w;
        let prep_off = lane * PREP_LANE_WIDTH;

        let lane_prep: &AluPrepLaneCols<_> =
            prep_local[prep_off..prep_off + PREP_LANE_WIDTH].borrow();
        let lane_main: &AluMainLaneCols<_, D> =
            main_local[main_off..main_off + alu_main_lane_width::<D>()].borrow();

        let mult_a: AB::Expr = lane_prep.mult_a.into();
        let mult_b: AB::Expr = lane_prep.mult_b.into();
        let mult_out: AB::Expr = lane_prep.mult_out.into();
        let a_is_reader: AB::Expr = lane_prep.a_is_reader.into();
        let c_is_reader: AB::Expr = lane_prep.c_is_reader.into();

        let eff_mult_a = mult_a.clone() * a_is_reader;
        let eff_mult_c = mult_a * c_is_reader;

        let multiplicities = [eff_mult_a, mult_b, eff_mult_c, mult_out];
        let idx_vars = [
            lane_prep.a_idx,
            lane_prep.b_idx,
            lane_prep.c_idx,
            lane_prep.out_idx,
        ];
        let operands: [&[AB::Var; D]; 4] =
            [&lane_main.a, &lane_main.b, &lane_main.c, &lane_main.out];

        for i in 0..4 {
            let mut fields: Vec<AB::Expr> = Vec::with_capacity(1 + D);
            fields.push(idx_vars[i].into());
            for j in 0..D {
                fields.push(operands[i][j].into());
            }
            builder.push_interaction(
                "WitnessChecks",
                fields,
                Count::bounded(multiplicities[i].clone(), 1),
            );
        }
    }

    // Extra lookups for (a_t, c_t), t = 1..K_max - 1, on packed Horner rows.
    let extra_main = air.lanes * lane_w;
    let extra_prep = air.lanes * PREP_LANE_WIDTH;
    let k_max = air.horner_packed_steps;
    let num_int = num_horner_intermediates(k_max);
    let ac_base = extra_main + num_int * D;

    for t in 1..k_max {
        let p = extra_prep + extra_prep_a_idx_for_step(t, k_max);
        let step: &AluPackedHornerStepPrepCols<_> =
            prep_local[p..p + PACKED_HORNER_STEP_PREP_WIDTH].borrow();
        let eff_mult_a: AB::Expr = step.horner_lookup_mult_a.into();
        let eff_mult_c: AB::Expr = step.horner_lookup_mult_c.into();

        let main_off = ac_base + 2 * (t - 1) * D;

        let mut a_inps: Vec<AB::Expr> = Vec::with_capacity(1 + D);
        a_inps.push(step.a_idx.into());
        for j in 0..D {
            a_inps.push(main_local[main_off + j].into());
        }
        builder.push_interaction("WitnessChecks", a_inps, Count::bounded(eff_mult_a, 1));

        let mut c_inps: Vec<AB::Expr> = Vec::with_capacity(1 + D);
        c_inps.push(step.c_idx.into());
        for j in 0..D {
            c_inps.push(main_local[main_off + D + j].into());
        }
        builder.push_interaction("WitnessChecks", c_inps, Count::bounded(eff_mult_c, 1));
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use p3_circuit::WitnessId;
    use p3_circuit::ops::AluOpKind;
    use p3_field::BasedVectorSpace;
    use p3_matrix::Matrix;
    use p3_test_utils::baby_bear_params::{
        BabyBear as Val, BinomialExtensionField, PrimeCharacteristicRing,
    };

    use super::*;
    use p3_test_utils::air_satisfaction::{assert_air_rejects, assert_air_satisfies};

    /// Convert an `AluTrace` to preprocessed values (13 columns per op) for standalone tests.
    fn trace_to_preprocessed<F: Field, ExtF: BasedVectorSpace<F>, const D: usize>(
        trace: &AluTrace<ExtF>,
    ) -> Vec<F> {
        let total_len = trace.indices.len() * AluAir::<F, D>::preprocessed_lane_width();
        let mut preprocessed_values = Vec::with_capacity(total_len);
        let neg_one = F::ZERO - F::ONE;

        for (i, kind) in trace.op_kind.iter().enumerate() {
            let (sel_add_vs_mul, sel_bool, sel_muladd, sel_horner) = match kind {
                AluOpKind::Add => (F::ONE, F::ZERO, F::ZERO, F::ZERO),
                AluOpKind::Mul => (F::ZERO, F::ZERO, F::ZERO, F::ZERO),
                AluOpKind::BoolCheck => (F::ZERO, F::ONE, F::ZERO, F::ZERO),
                AluOpKind::MulAdd => (F::ZERO, F::ZERO, F::ONE, F::ZERO),
                AluOpKind::HornerAcc => (F::ZERO, F::ZERO, F::ZERO, F::ONE),
            };

            preprocessed_values.extend(&[
                neg_one, // mult_a (base; active = 1)
                sel_add_vs_mul,
                sel_bool,
                sel_muladd,
                sel_horner,
                F::from_u32(trace.indices[i][0].0 * D as u32),
                F::from_u32(trace.indices[i][1].0 * D as u32),
                F::from_u32(trace.indices[i][2].0 * D as u32),
                F::from_u32(trace.indices[i][3].0 * D as u32),
                neg_one, // mult_b (reader placeholder)
                F::ONE,  // mult_out (creator placeholder)
                F::ONE,  // a_is_reader (standalone: constrained)
                F::ONE,  // c_is_reader (standalone: constrained)
            ]);
        }

        preprocessed_values
    }

    type EF = BinomialExtensionField<Val, 4>;

    #[test]
    fn satisfies_alu_add_base_field() {
        let n = 8;
        let trace = AluTrace {
            op_kind: vec![AluOpKind::Add; n],
            values: vec![
                [
                    Val::from_u64(3),
                    Val::from_u64(5),
                    Val::ZERO,
                    Val::from_u64(8),
                ];
                n
            ],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(0), WitnessId(3)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 1, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_eq!(matrix.width(), air.total_width());
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn satisfies_alu_mul_base_field() {
        let n = 8;
        let trace = AluTrace {
            op_kind: vec![AluOpKind::Mul; n],
            values: vec![
                [
                    Val::from_u64(3),
                    Val::from_u64(5),
                    Val::ZERO,
                    Val::from_u64(15),
                ];
                n
            ],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(0), WitnessId(3)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 1, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn satisfies_alu_bool_check() {
        let n = 8;
        let trace = AluTrace {
            op_kind: vec![AluOpKind::BoolCheck; n],
            values: (0..n)
                .map(|i| {
                    [
                        Val::from_u64(i as u64 % 2),
                        Val::ZERO,
                        Val::ZERO,
                        Val::from_u64(i as u64 % 2),
                    ]
                })
                .collect(),
            indices: vec![[WitnessId(1), WitnessId(0), WitnessId(0), WitnessId(1)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 1, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn satisfies_alu_muladd() {
        let n = 8;
        // a * b + c = out  =>  3 * 5 + 2 = 17
        let trace = AluTrace {
            op_kind: vec![AluOpKind::MulAdd; n],
            values: vec![
                [
                    Val::from_u64(3),
                    Val::from_u64(5),
                    Val::from_u64(2),
                    Val::from_u64(17),
                ];
                n
            ],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(3), WitnessId(4)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 1, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn satisfies_alu_mixed_ops() {
        let trace = AluTrace {
            op_kind: vec![AluOpKind::Add, AluOpKind::Mul],
            values: vec![
                [
                    Val::from_u64(3),
                    Val::from_u64(5),
                    Val::ZERO,
                    Val::from_u64(8),
                ],
                [
                    Val::from_u64(4),
                    Val::from_u64(6),
                    Val::ZERO,
                    Val::from_u64(24),
                ],
            ],
            indices: vec![
                [WitnessId(1), WitnessId(2), WitnessId(0), WitnessId(3)],
                [WitnessId(1), WitnessId(2), WitnessId(0), WitnessId(3)],
            ],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(2, 1, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn satisfies_alu_double_step_horner_base_field() {
        // out = prev_out * b + c - a; double-step rows share b across two steps.
        let n = 3;

        let prev_out = Val::ZERO;
        let a0 = Val::from_u64(1);
        let b0 = Val::from_u64(2);
        let c0 = Val::from_u64(5);
        let out0 = prev_out * b0 + c0 - a0;

        let a1 = Val::ZERO;
        let b1 = b0;
        let c1 = Val::from_u64(3);
        let out1 = out0 * b1 + c1 - a1;

        let a2 = Val::from_u64(1);
        let b2 = Val::from_u64(3);
        let c2 = Val::from_u64(2);
        let out2 = out1 * b2 + c2 - a2;

        let trace = AluTrace {
            op_kind: vec![AluOpKind::HornerAcc; n],
            values: vec![[a0, b0, c0, out0], [a1, b1, c1, out1], [a2, b2, c2, out2]],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(3), WitnessId(4)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 1, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    /// Regression for the chain-head-seed soundness fix: a Horner chain's first step
    /// reads its accumulator from the preceding separator row's `out`, which must be
    /// pinned to 0. Forging that seed nonzero and re-deriving each step's `out` keeps
    /// every HORNER_ACC transition satisfied, so *before* the fix the forged trace
    /// verified (shifting the evaluated polynomial); the new constraint must reject it.
    /// Distinct `b` per step keeps every step a single (unpacked) row → one op per row.
    #[test]
    fn rejects_alu_horner_forged_chain_head_seed() {
        let n = 3;
        let steps = [
            (Val::from_u64(1), Val::from_u64(2), Val::from_u64(5)),
            (Val::ZERO, Val::from_u64(3), Val::from_u64(3)),
            (Val::from_u64(1), Val::from_u64(5), Val::from_u64(2)),
        ];

        // Honest chain: seed = 0.
        let mut prev = Val::ZERO;
        let values: Vec<[Val; 4]> = steps
            .iter()
            .map(|&(a, b, c)| {
                let out = prev * b + c - a;
                prev = out;
                [a, b, c, out]
            })
            .collect();

        // Distinct witness indices per op (distinct `b_idx`) so the steps are NOT packed
        // together — each becomes a single Horner row, giving a clean one-op-per-row layout.
        let trace = AluTrace {
            op_kind: vec![AluOpKind::HornerAcc; n],
            values,
            indices: vec![
                [WitnessId(1), WitnessId(2), WitnessId(3), WitnessId(4)],
                [WitnessId(5), WitnessId(6), WitnessId(7), WitnessId(8)],
                [WitnessId(9), WitnessId(10), WitnessId(11), WitnessId(12)],
            ],
        };
        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 1, preprocessed, 2);
        let honest: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &honest);

        // The leading separator is row 0; for D=1 lane 0, `out` is column index 3.
        let width = honest.width();
        let out_col = 3;
        assert_eq!(
            honest.values[out_col],
            Val::ZERO,
            "separator seed is 0 when honest"
        );

        // Forge the separator seed to 1 and re-derive each step's `out` so every
        // HORNER_ACC transition still holds — only the chain-head-seed constraint fails.
        // (`honest` is not read past this point, so move it rather than clone.)
        let mut forged = honest;
        forged.values[out_col] = Val::ONE;
        let mut acc = Val::ONE;
        for (r, &(a, b, c)) in steps.iter().enumerate() {
            let out = acc * b + c - a;
            forged.values[(r + 1) * width + out_col] = out;
            acc = out;
        }
        assert_air_rejects::<Val, EF, _>(&air, &forged);
    }

    #[test]
    fn satisfies_alu_k4_packed_horner_base_field() {
        const K: usize = 4;
        let n = 8;
        let b = Val::from_u64(2);
        let mut acc = Val::ZERO;
        let mut values = Vec::with_capacity(n);
        for step in 0..n {
            let a = Val::from_u64((step + 1) as u64);
            let c = Val::from_u64(5);
            let out = acc * b + c - a;
            values.push([a, b, c, out]);
            acc = out;
        }
        let trace = AluTrace {
            op_kind: vec![AluOpKind::HornerAcc; n],
            values,
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(3), WitnessId(4)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 1, preprocessed, K);
        assert_eq!(air.horner_packed_steps, K);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn one_shot_schedule_release_rebuild_preserves_traces() {
        let n = 8;
        let trace = AluTrace {
            op_kind: vec![AluOpKind::HornerAcc; n],
            values: vec![[Val::ONE, Val::from_u64(2), Val::from_u64(3), Val::ONE]; n],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(3), WitnessId(4)]; n],
        };
        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let mut air = AluAir::<Val, 1>::new_with_preprocessed(n, 2, preprocessed, 2);
        let main_before = air.trace_to_matrix(&trace, 1);
        let prep_before = air.preprocessed_trace().unwrap();
        assert!(air.schedule.is_some());

        air.release_one_shot_schedule();
        assert!(air.schedule.is_none());
        air.rebuild_one_shot_schedule();

        assert_eq!(air.trace_to_matrix(&trace, 1), main_before);
        assert_eq!(air.preprocessed_trace().unwrap(), prep_before);
    }

    #[test]
    fn one_shot_streaming_schedule_matches_materialized_schedule() {
        let op_kind = vec![
            AluOpKind::Mul,
            AluOpKind::HornerAcc,
            AluOpKind::HornerAcc,
            AluOpKind::Add,
            AluOpKind::HornerAcc,
            AluOpKind::HornerAcc,
            AluOpKind::HornerAcc,
            AluOpKind::BoolCheck,
        ];
        let n = op_kind.len();
        let trace = AluTrace {
            op_kind,
            values: vec![[Val::ONE, Val::from_u64(2), Val::from_u64(3), Val::ONE]; n],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(3), WitnessId(4)]; n],
        };
        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let materialized = AluAir::<Val, 1>::new_with_preprocessed(n, 2, preprocessed.clone(), 2);
        let entry_count =
            AluAir::<Val, 1>::streaming_schedule_entry_count_for(&preprocessed, 2, 2).unwrap();
        let streaming = AluAir::<Val, 1>::from_reduction_with_streaming_schedule(
            n,
            2,
            AluExtMulKind::Base,
            preprocessed,
            2,
            Some(entry_count),
        );

        assert_eq!(materialized.scheduled_entry_count(), entry_count);
        assert_eq!(streaming.scheduled_entry_count(), entry_count);
        assert_eq!(
            streaming.trace_to_matrix(&trace, 1),
            materialized.trace_to_matrix(&trace, 1)
        );
        assert_eq!(
            streaming.preprocessed_trace().unwrap(),
            materialized.preprocessed_trace().unwrap()
        );
    }

    #[test]
    fn one_shot_lde_capacity_reserve_preserves_canonical_matrix() {
        let n = 8;
        let trace = AluTrace {
            op_kind: vec![AluOpKind::Mul; n],
            values: vec![[Val::ONE, Val::from_u64(2), Val::ZERO, Val::from_u64(2)]; n],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(0), WitnessId(3)]; n],
        };
        let preprocessed = trace_to_preprocessed::<Val, _, 1>(&trace);
        let air = AluAir::<Val, 1>::new_with_preprocessed(n, 2, preprocessed, 2);
        let canonical = air.trace_to_matrix(&trace, 1);
        let reserved = air.trace_to_matrix_with_lde_capacity(&trace, 1, 2);

        assert_eq!(reserved, canonical);
        assert!(
            reserved.values.capacity() >= reserved.values.len() << 2,
            "one-shot matrix must reserve the final four-coset LDE capacity"
        );
    }

    #[test]
    fn satisfies_alu_extension_field_d4() {
        let n = 4;
        let w = Val::from_u64(11); // BabyBear's binomial extension uses w=11

        let a = EF::from_basis_coefficients_slice(&[
            Val::from_u64(7),
            Val::from_u64(3),
            Val::from_u64(4),
            Val::from_u64(5),
        ])
        .unwrap();
        let b = EF::from_basis_coefficients_slice(&[
            Val::from_u64(11),
            Val::from_u64(2),
            Val::from_u64(9),
            Val::from_u64(6),
        ])
        .unwrap();
        let c = EF::ZERO;
        let out = a * b;

        let trace = AluTrace {
            op_kind: vec![AluOpKind::Mul; n],
            values: vec![[a, b, c, out]; n],
            indices: vec![[WitnessId(1), WitnessId(2), WitnessId(0), WitnessId(3)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 4>(&trace);
        let air = AluAir::<Val, 4>::new_binomial_with_preprocessed(n, 1, w, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_eq!(matrix.width(), air.total_width());
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn satisfies_alu_bool_check_extension_field_d4() {
        let n = 4;
        let w = Val::from_u64(11);

        // Valid booleans in EF: [0,0,0,0] and [1,0,0,0] interleaved.
        let zero = EF::from_basis_coefficients_slice(&[Val::ZERO, Val::ZERO, Val::ZERO, Val::ZERO])
            .unwrap();
        let one = EF::from_basis_coefficients_slice(&[Val::ONE, Val::ZERO, Val::ZERO, Val::ZERO])
            .unwrap();

        let trace = AluTrace {
            op_kind: vec![AluOpKind::BoolCheck; n],
            values: (0..n)
                .map(|i| {
                    let v = if i % 2 == 0 { zero } else { one };
                    [v, zero, zero, v]
                })
                .collect(),
            indices: vec![[WitnessId(1), WitnessId(0), WitnessId(0), WitnessId(1)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 4>(&trace);
        let air = AluAir::<Val, 4>::new_binomial_with_preprocessed(n, 1, w, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_satisfies::<Val, EF, _>(&air, &matrix);
    }

    /// Soundness regression: a non-zero higher coefficient on an EF bool_check input must be
    /// rejected. `a[0] = 1` would be a valid boolean by itself, but the extension-field
    /// bool_check constraint also asserts `a[1] = a[2] = a[3] = 0`.
    #[test]
    fn bool_check_extension_field_rejects_nonzero_higher_coefficients() {
        let n = 4;
        let w = Val::from_u64(11);

        let bad =
            EF::from_basis_coefficients_slice(&[Val::ONE, Val::from_u64(5), Val::ZERO, Val::ZERO])
                .unwrap();
        let zero = EF::ZERO;

        let trace = AluTrace {
            op_kind: vec![AluOpKind::BoolCheck; n],
            values: vec![[bad, zero, zero, bad]; n],
            indices: vec![[WitnessId(1), WitnessId(0), WitnessId(0), WitnessId(1)]; n],
        };

        let preprocessed = trace_to_preprocessed::<Val, _, 4>(&trace);
        let air = AluAir::<Val, 4>::new_binomial_with_preprocessed(n, 1, w, preprocessed, 2);
        let matrix: RowMajorMatrix<Val> = air.trace_to_matrix(&trace, 1);
        assert_air_rejects::<Val, EF, _>(&air, &matrix);
    }

    #[test]
    fn test_alu_air_constraint_degree() {
        let preprocessed = vec![Val::ZERO; 8 * 13]; // 8 ops * 13 preprocessed columns per op
        let air = AluAir::<Val, 1>::new_with_preprocessed(8, 2, preprocessed, 2);
        p3_test_utils::assert_air_constraint_degree!(air, "AluAir");
    }
}
