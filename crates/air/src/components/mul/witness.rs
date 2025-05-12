use crate::{
    components::{InteractionClaim, MulClaim, NodeElements, TraceError},
    utils::{pack_values, TreeBuilder},
};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo_prover::{
    constraint_framework::{logup::LogupTraceGenerator, Relation},
    core::backend::simd::{
        m31::{PackedM31, LOG_N_LANES, N_LANES},
        qm31::PackedQM31,
        SimdBackend,
    },
};

use super::table::{MulColumn, MulTraceTable, MulTraceTableRow, PackedMulTraceTableRow};

/// Number of main trace columns for the Mul component.
pub(crate) const N_TRACE_COLUMNS: usize = 16;

/// Generates the main trace columns and initial data for interaction claims for the Mul component.
///
/// Takes the raw `MulTraceTable`, processes it into the main STARK trace columns
/// (including the fixed-point remainder `rem`), and prepares `LookupData` for LogUp.
pub struct ClaimGenerator {
    /// The raw trace data for Mul operations.
    pub inputs: MulTraceTable,
}

impl ClaimGenerator {
    /// Creates a new `ClaimGenerator` with the given `MulTraceTable`.
    pub fn new(inputs: MulTraceTable) -> Self {
        Self { inputs }
    }

    /// Writes the main trace columns to the `tree_builder` and returns data for interaction phase.
    ///
    /// Similar to the Add component's `write_trace`, this pads the table, packs rows,
    /// calls `write_trace_simd` to generate main trace columns and `LookupData`,
    /// adds the main trace to the `tree_builder`, and returns the `MulClaim` and `InteractionClaimGenerator`.
    /// Returns `TraceError::EmptyTrace` if the input table is empty.
    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(MulClaim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs.table.resize(size, MulTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            MulClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

/// Populates the main trace columns and `LookupData` from SIMD-packed Mul trace rows.
///
/// Processes `PackedMulTraceTableRow` data in parallel:
/// - Maps fields (including `rem`) to the corresponding main trace columns.
/// - Extracts `[value, id]` pairs and multiplicities into `LookupData` for the LogUp argument
///   (LHS, RHS, OUT).
/// Returns the `ComponentTrace` (main trace columns) and `LookupData`.
fn write_trace_simd(
    inputs: Vec<PackedMulTraceTableRow>,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;

    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    (
        trace.par_iter_mut(),
        lookup_data.par_iter_mut(),
        inputs.into_par_iter(),
    )
        .into_par_iter()
        .for_each(|(mut row, lookup_data, input)| {
            *row[MulColumn::NodeId.index()] = input.node_id;
            *row[MulColumn::LhsId.index()] = input.lhs_id;
            *row[MulColumn::RhsId.index()] = input.rhs_id;
            *row[MulColumn::Idx.index()] = input.idx;
            *row[MulColumn::IsLastIdx.index()] = input.is_last_idx;
            *row[MulColumn::NextNodeId.index()] = input.next_node_id;
            *row[MulColumn::NextLhsId.index()] = input.next_lhs_id;
            *row[MulColumn::NextRhsId.index()] = input.next_rhs_id;
            *row[MulColumn::NextIdx.index()] = input.next_idx;
            *row[MulColumn::Lhs.index()] = input.lhs;
            *row[MulColumn::Rhs.index()] = input.rhs;
            *row[MulColumn::Out.index()] = input.out;
            *row[MulColumn::Rem.index()] = input.rem;
            *row[MulColumn::LhsMult.index()] = input.lhs_mult;
            *row[MulColumn::RhsMult.index()] = input.rhs_mult;
            *row[MulColumn::OutMult.index()] = input.out_mult;

            *lookup_data.lhs = [input.lhs, input.lhs_id];
            *lookup_data.lhs_mult = input.lhs_mult;
            *lookup_data.rhs = [input.rhs, input.rhs_id];
            *lookup_data.rhs_mult = input.rhs_mult;
            *lookup_data.out = [input.out, input.node_id];
            *lookup_data.out_mult = input.out_mult;
        });

    (trace, lookup_data)
}

/// Intermediate data structure holding values and multiplicities for the Mul LogUp argument.
///
/// Structure and purpose are identical to the `LookupData` in the Add component,
/// storing value-ID pairs and multiplicities for LHS, RHS, and OUT terms.
/// Derives helper iterators for parallel processing.
#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    /// LHS value-ID pairs: `[lhs_value, lhs_node_id]`.
    lhs: Vec<[PackedM31; 2]>,
    /// Multiplicities for LHS values.
    lhs_mult: Vec<PackedM31>,
    /// RHS value-ID pairs: `[rhs_value, rhs_node_id]`.
    rhs: Vec<[PackedM31; 2]>,
    /// Multiplicities for RHS values.
    rhs_mult: Vec<PackedM31>,
    /// Output value-ID pairs: `[out_value, mul_node_id]`.
    out: Vec<[PackedM31; 2]>,
    /// Multiplicities for output values.
    out_mult: Vec<PackedM31>,
}

/// Generates the interaction trace columns for the Mul component's LogUp argument.
///
/// Structure and purpose are identical to the `InteractionClaimGenerator` in the Add component.
/// It takes `LookupData` and `NodeElements` to build the three LogUp interaction columns
/// (LHS, RHS, OUT) and adds them to the `tree_builder`.
pub struct InteractionClaimGenerator {
    /// Log2 size of the trace.
    log_size: u32,
    /// Data (value-ID pairs and multiplicities) needed for LogUp.
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    /// Writes the LogUp interaction trace columns to the `tree_builder`.
    ///
    /// The logic is identical to the Add component's implementation:
    /// - Creates a `LogupTraceGenerator`.
    /// - Generates three columns (LHS, RHS, OUT), writing `multiplicity / denom` fractions,
    ///   where `denom` is derived from `[value, id]` and `NodeElements`.
    /// - Finalizes the generator, obtaining interaction trace columns and the `claimed_sum`.
    /// - Adds interaction columns to the `tree_builder`.
    /// - Returns the `InteractionClaim` containing the `claimed_sum`.
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        node_elements: &NodeElements,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.lhs[row];
            let multiplicity = &self.lookup_data.lhs_mult[row];

            let denom: PackedQM31 = node_elements.combine(values);
            col_gen.write_frac(row, (*multiplicity).into(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.rhs[row];
            let multiplicity = &self.lookup_data.rhs_mult[row];

            let denom: PackedQM31 = node_elements.combine(values);
            col_gen.write_frac(row, (*multiplicity).into(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.out[row];
            let multiplicity = &self.lookup_data.out_mult[row];

            let denom: PackedQM31 = node_elements.combine(values);
            col_gen.write_frac(row, (*multiplicity).into(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
