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
    components::{
        log2::table::{Log2Column, Log2TraceTable, Log2TraceTableRow, PackedLog2TraceTableRow}, lookups::log2::Log2LookupElements, Log2Claim, InteractionClaim, NodeElements
    },
    utils::{pack_values, TreeBuilder},
};

/// Number of main trace columns for the Log2 component.
pub(crate) const N_TRACE_COLUMNS: usize = 12;

/// Generates main trace and interaction data for the Log2 component.
///
/// Takes the raw `Log2TraceTable`, processes it into main STARK trace columns,
/// and prepares `LookupData` for three LogUp arguments: input, output, and LUT interaction.
pub struct ClaimGenerator {
    /// The raw trace data for Log2 operations.
    pub inputs: Log2TraceTable,
}

impl ClaimGenerator {
    /// Creates a new `ClaimGenerator` with the given `Log2TraceTable`.
    pub fn new(inputs: Log2TraceTable) -> Self {
        Self { inputs }
    }

    /// Writes the main trace columns and returns data for the interaction phase.
    ///
    /// Standard procedure: pads table, packs rows, calls `write_trace_simd`,
    /// adds main trace to `tree_builder`, returns `Log2Claim` and `InteractionClaimGenerator`.
    /// Returns `TraceError::EmptyTrace` if the input table is empty.
    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(Log2Claim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs.table.resize(size, Log2TraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            Log2Claim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

/// Populates main trace columns and `LookupData` from SIMD-packed Log2 trace rows.
///
/// Processes `PackedLog2TraceTableRow` data in parallel:
/// - Maps fields to corresponding main trace columns.
/// - Extracts `[value, id]` pairs and multiplicities for input and output LogUps,
///   and `lookup_mult` for the LUT interaction, into `LookupData`.
/// Returns the `ComponentTrace` and `LookupData`.
fn write_trace_simd(
    inputs: Vec<PackedLog2TraceTableRow>,
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
            *row[Log2Column::NodeId.index()] = input.node_id;
            *row[Log2Column::InputId.index()] = input.input_id;
            *row[Log2Column::Idx.index()] = input.idx;
            *row[Log2Column::IsLastIdx.index()] = input.is_last_idx;
            *row[Log2Column::NextNodeId.index()] = input.next_node_id;
            *row[Log2Column::NextInputId.index()] = input.next_input_id;
            *row[Log2Column::NextIdx.index()] = input.next_idx;
            *row[Log2Column::Input.index()] = input.input;
            *row[Log2Column::Out.index()] = input.out;
            *row[Log2Column::InputMult.index()] = input.input_mult;
            *row[Log2Column::OutMult.index()] = input.out_mult;
            *row[Log2Column::LookupMult.index()] = input.lookup_mult;

            *lookup_data.input = [input.input, input.input_id];
            *lookup_data.input_mult = input.input_mult;
            *lookup_data.out = [input.out, input.node_id];
            *lookup_data.out_mult = input.out_mult;
            *lookup_data.lookup_mult = input.lookup_mult;
        });

    (trace, lookup_data)
}

/// Intermediate data for Log2 component's LogUp arguments.
///
/// Holds value-ID pairs and multiplicities for input and output terms,
/// plus multiplicities for the interaction with the Log2 Lookup Table.
/// Derives helper iterators for parallel processing.
#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    /// Input value-ID pairs: `[input_value, input_node_id]`.
    input: Vec<[PackedM31; 2]>,
    /// Multiplicities for input values (LogUp).
    input_mult: Vec<PackedM31>,
    /// Output value-ID pairs: `[out_value, log2_node_id]`.
    out: Vec<[PackedM31; 2]>,
    /// Multiplicities for output values (LogUp).
    out_mult: Vec<PackedM31>,
    /// Multiplicities for Log2 LUT interaction.
    lookup_mult: Vec<PackedM31>,
}

/// Generates interaction trace columns for the Log2 component's LogUp arguments.
///
/// Builds three LogUp interaction columns:
/// 1. Input term: `(input_value, input_node_id)` with `NodeElements`.
/// 2. Output term: `(out_value, log2_node_id)` with `NodeElements`.
/// 3. LUT term: `(input_value, out_value)` with `Log2LookupElements`.
pub struct InteractionClaimGenerator {
    /// Log2 size of the trace.
    log_size: u32,
    /// Data for LogUp arguments.
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    /// Writes the three LogUp interaction trace columns to the `tree_builder`.
    ///
    /// - Initializes a `LogupTraceGenerator`.
    /// - For Input LogUp: combines `lookup_data.input[i]` with `node_elements` for denominator.
    /// - For Output LogUp: combines `lookup_data.out[i]` with `node_elements` for denominator.
    /// - For LUT Interaction: combines `[lookup_data.input[i][0], lookup_data.out[i][0]]` (raw values)
    ///   with `lookup_elements` for the denominator.
    /// - Writes `multiplicity / denominator` fractions for each.
    /// - Finalizes the generator, adds columns to `tree_builder`, returns `InteractionClaim`.
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        node_elements: &NodeElements,
        lookup_elements: &Log2LookupElements,
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

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let input = self.lookup_data.input[row][0];
            let output = self.lookup_data.out[row][0];
            let multiplicity = self.lookup_data.lookup_mult[row];

            let denom: PackedQM31 = lookup_elements.combine(&[input, output]);
            col_gen.write_frac(row, multiplicity.into(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}