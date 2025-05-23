use luminair_utils::TraceError;
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

use crate::{
    components::{InteractionClaim, MaxReduceClaim, NodeElements},
    utils::{pack_values, TreeBuilder},
};

use super::table::{
    MaxReduceColumn, MaxReduceTraceTable, MaxReduceTraceTableRow, PackedMaxReduceTraceTableRow,
};

/// Number of main trace columns for the MaxReduce component.
pub(crate) const N_TRACE_COLUMNS: usize = 15;

/// Generates main trace columns and interaction data for the MaxReduce component.
///
/// Takes `MaxReduceTraceTable`, processes it into main STARK trace columns
/// (including running max values, input/output), and prepares `LookupData` for LogUp.
pub struct ClaimGenerator {
    /// The raw trace data for MaxReduce operations.
    pub inputs: MaxReduceTraceTable,
}

impl ClaimGenerator {
    /// Creates a new `ClaimGenerator` with the given `MaxReduceTraceTable`.
    pub fn new(inputs: MaxReduceTraceTable) -> Self {
        Self { inputs }
    }

    /// Writes main trace columns and returns data for interaction phase.
    ///
    /// Standard procedure: pads, packs, calls `write_trace_simd`,
    /// adds main trace to `tree_builder`, returns `MaxReduceClaim` and `InteractionClaimGenerator`.
    /// Returns `TraceError::EmptyTrace` if the input table is empty.
    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(MaxReduceClaim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs
            .table
            .resize(size, MaxReduceTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            MaxReduceClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

/// Populates main trace columns and `LookupData` from SIMD-packed MaxReduce trace rows.
///
/// Processes `PackedMaxReduceTraceTableRow` data in parallel:
/// - Maps fields (node/input IDs, running max, input/out values, flags) to main trace columns.
/// - Extracts `[value, id]` pairs and multiplicities for input and output LogUps into `LookupData`.
/// Returns the `ComponentTrace` (main trace columns) and `LookupData`.
fn write_trace_simd(
    inputs: Vec<PackedMaxReduceTraceTableRow>,
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
            *row[MaxReduceColumn::NodeId.index()] = input.node_id;
            *row[MaxReduceColumn::InputId.index()] = input.input_id;
            *row[MaxReduceColumn::Idx.index()] = input.idx;
            *row[MaxReduceColumn::IsLastIdx.index()] = input.is_last_idx;
            *row[MaxReduceColumn::NextNodeId.index()] = input.next_node_id;
            *row[MaxReduceColumn::NextInputId.index()] = input.next_input_id;
            *row[MaxReduceColumn::NextIdx.index()] = input.next_idx;
            *row[MaxReduceColumn::Input.index()] = input.input;
            *row[MaxReduceColumn::Out.index()] = input.out;
            *row[MaxReduceColumn::MaxVal.index()] = input.max_val;
            *row[MaxReduceColumn::NextMaxVal.index()] = input.next_max_val;
            *row[MaxReduceColumn::IsLastStep.index()] = input.is_last_step;
            *row[MaxReduceColumn::IsMax.index()] = input.is_max;
            *row[MaxReduceColumn::InputMult.index()] = input.input_mult;
            *row[MaxReduceColumn::OutMult.index()] = input.out_mult;

            *lookup_data.input = [input.input, input.input_id];
            *lookup_data.input_mult = input.input_mult;
            *lookup_data.out = [input.out, input.node_id];
            *lookup_data.out_mult = input.out_mult;
        });

    (trace, lookup_data)
}

/// Intermediate data for the MaxReduce LogUp argument (input and output terms).
#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    /// Input value-ID pairs: `[input_value, input_node_id]`.
    input: Vec<[PackedM31; 2]>,
    /// Multiplicities for input values.
    input_mult: Vec<PackedM31>,
    /// Output value-ID pairs: `[out_value, max_reduce_node_id]`.
    out: Vec<[PackedM31; 2]>,
    /// Multiplicities for output values.
    out_mult: Vec<PackedM31>,
}

/// Generates interaction trace columns for the MaxReduce component's LogUp argument.
/// Builds two LogUp columns (input, output) and adds them to the `tree_builder`.
pub struct InteractionClaimGenerator {
    /// Log2 size of the trace.
    log_size: u32,
    /// Data (value-ID pairs and multiplicities) needed for LogUp.
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    /// Writes the LogUp interaction trace columns to the `tree_builder`.
    ///
    /// Similar to SumReduce/Recip: generates two columns (Input, Output), writing `multiplicity / denom` fractions.
    /// Finalizes generator, adds columns to `tree_builder`, returns `InteractionClaim`.
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        node_elements: &NodeElements,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.input[row];
            let multiplicity = &self.lookup_data.input_mult[row];

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
