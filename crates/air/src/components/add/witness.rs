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
        add::table::{AddColumn, AddTraceTableRow},
        AddClaim, InteractionClaim, NodeElements, TraceError,
    },
    utils::{pack_values, TreeBuilder},
};

use super::table::{AddTraceTable, PackedAddTraceTableRow};

/// Number of main trace columns for the Add component.
pub(crate) const N_TRACE_COLUMNS: usize = 15;

/// Generates the main trace columns and initial data for interaction claims for the Add component.
///
/// Takes the raw `AddTraceTable` collected during graph execution, processes it into
/// the main STARK trace columns, and prepares the necessary data (`LookupData`)
/// for generating the LogUp interaction trace columns later.
pub struct ClaimGenerator {
    /// The raw trace data for Add operations.
    pub inputs: AddTraceTable,
}

impl ClaimGenerator {
    /// Creates a new `ClaimGenerator` with the given `AddTraceTable`.
    pub fn new(inputs: AddTraceTable) -> Self {
        Self { inputs }
    }

    /// Writes the main trace columns to the `tree_builder` and returns data for interaction phase.
    ///
    /// 1. Pads the input table to a power-of-two size.
    /// 2. Converts rows to SIMD-packed format.
    /// 3. Calls `write_trace_simd` to populate main trace columns and `LookupData`.
    /// 4. Adds the generated main trace columns to the STWO commitment `tree_builder`.
    /// 5. Returns an `AddClaim` (with trace log_size) and an `InteractionClaimGenerator`
    ///    (containing `LookupData` needed for LogUp).
    /// Returns `TraceError::EmptyTrace` if the input table is empty.
    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(AddClaim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs.table.resize(size, AddTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            AddClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

/// Populates the main trace columns and `LookupData` from SIMD-packed trace rows.
///
/// This function processes the `PackedAddTraceTableRow` data in parallel:
/// - It directly maps fields from `PackedAddTraceTableRow` to the corresponding main trace columns.
/// - It extracts and stores `[value, id]` pairs and their multiplicities (from `lhs_mult`, etc.)
///   into the `LookupData` struct. This data is crucial for building the LogUp argument,
///   which links these values to where they are defined or used elsewhere in the graph.
/// Returns the `ComponentTrace` (main trace columns) and `LookupData`.
fn write_trace_simd(
    inputs: Vec<PackedAddTraceTableRow>,
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
            *row[AddColumn::NodeId.index()] = input.node_id;
            *row[AddColumn::LhsId.index()] = input.lhs_id;
            *row[AddColumn::RhsId.index()] = input.rhs_id;
            *row[AddColumn::Idx.index()] = input.idx;
            *row[AddColumn::IsLastIdx.index()] = input.is_last_idx;
            *row[AddColumn::NextNodeId.index()] = input.next_node_id;
            *row[AddColumn::NextLhsId.index()] = input.next_lhs_id;
            *row[AddColumn::NextRhsId.index()] = input.next_rhs_id;
            *row[AddColumn::NextIdx.index()] = input.next_idx;
            *row[AddColumn::Lhs.index()] = input.lhs;
            *row[AddColumn::Rhs.index()] = input.rhs;
            *row[AddColumn::Out.index()] = input.out;
            *row[AddColumn::LhsMult.index()] = input.lhs_mult;
            *row[AddColumn::RhsMult.index()] = input.rhs_mult;
            *row[AddColumn::OutMult.index()] = input.out_mult;

            *lookup_data.lhs = [input.lhs, input.lhs_id];
            *lookup_data.lhs_mult = input.lhs_mult;
            *lookup_data.rhs = [input.rhs, input.rhs_id];
            *lookup_data.rhs_mult = input.rhs_mult;
            *lookup_data.out = [input.out, input.node_id];
            *lookup_data.out_mult = input.out_mult;
        });

    (trace, lookup_data)
}

/// Intermediate data structure holding values and multiplicities for LogUp argument construction.
///
/// For each Add operation (LHS, RHS, OUT), it stores:
/// - `[value, id_of_value_source_or_dest_node]`: The pair used in the LogUp denominator.
/// - `multiplicity`: The +1 or -1 count for this value in the LogUp sum.
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
    /// Output value-ID pairs: `[out_value, add_node_id]`.
    out: Vec<[PackedM31; 2]>,
    /// Multiplicities for output values.
    out_mult: Vec<PackedM31>,
}

/// Generates the interaction trace columns for the Add component's LogUp argument.
///
/// Takes the `LookupData` (prepared by `ClaimGenerator`) and `NodeElements` (randomness)
/// to construct the three LogUp interaction columns (one each for LHS, RHS, OUT).
/// These columns prove that the values used/produced by Add operations are consistent
/// with their occurrences elsewhere in the computation graph.
pub struct InteractionClaimGenerator {
    /// Log2 size of the trace.
    log_size: u32,
    /// Data (value-ID pairs and multiplicities) needed for LogUp.
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    /// Writes the LogUp interaction trace columns to the `tree_builder`.
    ///
    /// For each of LHS, RHS, and OUT:
    /// 1. Initializes a LogUp column generator.
    /// 2. For each entry in `lookup_data`:
    ///    a. Combines `[value, id]` with `NodeElements` to form the denominator for LogUp.
    ///    b. Writes `multiplicity / denominator` to the current LogUp column.
    /// 3. Finalizes the column.
    /// After processing all three, finalizes the `LogupTraceGenerator` to get the interaction trace
    /// columns and the overall `claimed_sum` for the LogUp argument.
    /// Adds the interaction trace columns to the STWO `tree_builder`.
    /// Returns the `InteractionClaim` containing the `claimed_sum`.
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
