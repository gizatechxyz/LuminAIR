use crate::{
    components::{InteractionClaim, NodeElements, RecipClaim, TraceError},
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

use super::table::{PackedRecipTraceTableRow, RecipColumn, RecipTraceTable, RecipTraceTableRow};

/// Number of main trace columns for the Recip component.
pub(crate) const N_TRACE_COLUMNS: usize = 13;

/// Generates the main trace columns and initial data for interaction claims for the Recip component.
///
/// Takes the raw `RecipTraceTable`, processes it into the main STARK trace columns
/// (including input, output, remainder, scale), and prepares `LookupData` for LogUp.
pub struct ClaimGenerator {
    /// The raw trace data for Recip operations.
    pub inputs: RecipTraceTable,
}

impl ClaimGenerator {
    /// Creates a new `ClaimGenerator` with the given `RecipTraceTable`.
    pub fn new(inputs: RecipTraceTable) -> Self {
        Self { inputs }
    }

    /// Writes the main trace columns to the `tree_builder` and returns data for interaction phase.
    ///
    /// Follows the standard pattern: pads the table, packs rows, calls `write_trace_simd`,
    /// adds main trace to `tree_builder`, returns `RecipClaim` and `InteractionClaimGenerator`.
    /// Returns `TraceError::EmptyTrace` if the input table is empty.
    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(RecipClaim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs
            .table
            .resize(size, RecipTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            RecipClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

/// Populates the main trace columns and `LookupData` from SIMD-packed Recip trace rows.
///
/// Processes `PackedRecipTraceTableRow` data in parallel:
/// - Maps fields (input, out, rem, scale, etc.) to the corresponding main trace columns.
/// - Extracts `[value, id]` pairs and multiplicities into `LookupData` for the LogUp argument
///   (only for input and output, as reciprocal is unary).
/// Returns the `ComponentTrace` (main trace columns) and `LookupData`.
fn write_trace_simd(
    inputs: Vec<PackedRecipTraceTableRow>,
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
            *row[RecipColumn::NodeId.index()] = input.node_id;
            *row[RecipColumn::InputId.index()] = input.input_id;
            *row[RecipColumn::Idx.index()] = input.idx;
            *row[RecipColumn::IsLastIdx.index()] = input.is_last_idx;
            *row[RecipColumn::NextNodeId.index()] = input.next_node_id;
            *row[RecipColumn::NextInputId.index()] = input.next_input_id;
            *row[RecipColumn::NextIdx.index()] = input.next_idx;
            *row[RecipColumn::Input.index()] = input.input;
            *row[RecipColumn::Out.index()] = input.out;
            *row[RecipColumn::Rem.index()] = input.rem;
            *row[RecipColumn::Scale.index()] = input.scale;
            *row[RecipColumn::InputMult.index()] = input.input_mult;
            *row[RecipColumn::OutMult.index()] = input.out_mult;

            *lookup_data.input = [input.input, input.input_id];
            *lookup_data.input_mult = input.input_mult;
            *lookup_data.out = [input.out, input.node_id];
            *lookup_data.out_mult = input.out_mult;
        });

    (trace, lookup_data)
}

/// Intermediate data structure holding values and multiplicities for the Recip LogUp argument.
///
/// Stores value-ID pairs and multiplicities only for the input and output terms.
/// Derives helper iterators for parallel processing.
#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    /// Input value-ID pairs: `[input_value, input_node_id]`.
    input: Vec<[PackedM31; 2]>,
    /// Multiplicities for input values.
    input_mult: Vec<PackedM31>,
    /// Output value-ID pairs: `[out_value, recip_node_id]`.
    out: Vec<[PackedM31; 2]>,
    /// Multiplicities for output values.
    out_mult: Vec<PackedM31>,
}

/// Generates the interaction trace columns for the Recip component's LogUp argument.
///
/// Takes `LookupData` and `NodeElements` to build the two LogUp interaction columns
/// (one for input, one for output) and adds them to the `tree_builder`.
pub struct InteractionClaimGenerator {
    /// Log2 size of the trace.
    log_size: u32,
    /// Data (value-ID pairs and multiplicities) needed for LogUp.
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    /// Writes the LogUp interaction trace columns to the `tree_builder`.
    ///
    /// Logic is similar to Add/Mul, but only generates two columns (Input, Output):
    /// - Creates a `LogupTraceGenerator`.
    /// - Generates two columns, writing `multiplicity / denom` fractions.
    /// - Finalizes the generator, obtaining interaction trace columns and `claimed_sum`.
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
