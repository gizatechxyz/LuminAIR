use num_traits::One;
use serde::{Deserialize, Serialize};
use stwo_prover::{
    constraint_framework::{logup::LogupTraceGenerator, Relation},
    core::{
        backend::{
            simd::{
                column::BaseColumn,
                m31::{PackedM31, LOG_N_LANES},
            },
            Column,
        },
        fields::m31::BaseField,
        poly::circle::{CanonicCoset, CircleEvaluation},
    },
};

use crate::{
    components::{InteractionClaim, NodeElements, SinClaim, TraceColumn, TraceError, TraceEval},
    utils::calculate_log_size,
    SIN_LOOKUP_TABLE,
};

/// Represents the trace for the Sin component, containing the required registers for its
/// constraints.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinTable {
    /// A vector of [`AddTableRow`] representing the table rows.
    pub table: Vec<SinTableRow>,
}

/// Represents a single row of the [`SinTable`]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SinTableRow {
    pub node_id: BaseField,
    pub input_id: BaseField,
    pub idx: BaseField,
    pub is_last_idx: BaseField,
    pub next_node_id: BaseField,
    pub next_input_id: BaseField,
    pub next_idx: BaseField,
    pub input: BaseField,
    pub output: BaseField,
    pub input_mult: BaseField,
    pub output_mult: BaseField,
}

impl SinTable {
    /// Creates a new, empty [`SinTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the Add Table.
    pub fn add_row(&mut self, row: SinTableRow) {
        self.table.push(row);
    }

    /// Transforms the [`SinTable`] into [`TraceEval`] to be commited
    /// when generating a STARK proof.
    pub fn trace_evaluation(&self) -> Result<(TraceEval, SinClaim), TraceError> {
        let n_rows = self.table.len();
        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }
        // Calculate log size
        let log_size = calculate_log_size(n_rows);

        // Calculate trace size
        let trace_size = 1 << log_size;

        // Create columns
        let mut node_id = BaseColumn::zeros(trace_size);
        let mut input_id = BaseColumn::zeros(trace_size);
        let mut idx = BaseColumn::zeros(trace_size);
        let mut is_last_idx = BaseColumn::zeros(trace_size);
        let mut next_node_id = BaseColumn::zeros(trace_size);
        let mut next_input_id = BaseColumn::zeros(trace_size);
        let mut next_idx = BaseColumn::zeros(trace_size);
        let mut input = BaseColumn::zeros(trace_size);
        let mut output = BaseColumn::zeros(trace_size);
        let mut input_mult = BaseColumn::zeros(trace_size);
        let mut output_mult = BaseColumn::zeros(trace_size);

        // Fill columns
        for (vec_row, row) in self.table.iter().enumerate() {
            node_id.set(vec_row, row.node_id);
            input_id.set(vec_row, row.input_id);
            idx.set(vec_row, row.idx);
            is_last_idx.set(vec_row, row.is_last_idx);
            next_node_id.set(vec_row, row.next_node_id);
            next_input_id.set(vec_row, row.next_input_id);
            next_idx.set(vec_row, row.next_idx);
            input.set(vec_row, row.input);
            output.set(vec_row, row.output);
            input_mult.set(vec_row, row.input_mult);
            output_mult.set(vec_row, row.output_mult);
        }

        for i in self.table.len()..trace_size {
            is_last_idx.set(i, BaseField::one());
        }

        // Create domain
        let domain = CanonicCoset::new(log_size).circle_domain();

        // Create trace
        let mut trace = Vec::with_capacity(SinColumn::count().0);
        trace.push(CircleEvaluation::new(domain, node_id));
        trace.push(CircleEvaluation::new(domain, input_id));
        trace.push(CircleEvaluation::new(domain, idx));
        trace.push(CircleEvaluation::new(domain, is_last_idx));
        trace.push(CircleEvaluation::new(domain, next_node_id));
        trace.push(CircleEvaluation::new(domain, next_input_id));
        trace.push(CircleEvaluation::new(domain, next_idx));
        trace.push(CircleEvaluation::new(domain, input));
        trace.push(CircleEvaluation::new(domain, output));
        trace.push(CircleEvaluation::new(domain, input_mult));
        trace.push(CircleEvaluation::new(domain, output_mult));

        assert_eq!(trace.len(), SinColumn::count().0);

        Ok((trace, SinClaim::new(log_size)))
    }
}

/// Enum representing the column indices in the Add trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SinColumn {
    NodeId,
    InputId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextInputId,
    NextIdx,
    Input,
    Output,
    InputMult,
    OutputMult,
}

impl SinColumn {
    /// Returns the index of the column in the sin trace.
    pub const fn index(self) -> usize {
        match self {
            Self::NodeId => 0,
            Self::InputId => 1,
            Self::Idx => 2,
            Self::IsLastIdx => 3,
            Self::NextNodeId => 4,
            Self::NextInputId => 5,
            Self::NextIdx => 6,
            Self::Input => 7,
            Self::Output => 8,
            Self::InputMult => 9,
            Self::OutputMult => 10,
        }
    }
}
impl TraceColumn for SinColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (11, 3)
    }
}

#[derive(Debug, Clone)]
pub struct SinLUTRow {
    pub node_id: BaseField,
    pub input: BaseField,
    pub output: BaseField,
    pub multiplicity: BaseField,
}

#[derive(Debug, Clone)]
pub struct SinLUTTable {
    pub table: Vec<SinLUTRow>,
}

impl SinLUTTable {
    /// Constructs the LUT from a precompiled build-time table of (input, output) BaseField pairs.
    ///
    /// The `multiplicity` is always set to 1.
    pub fn from_lookup_table() -> Self {
        Self {
            table: SIN_LOOKUP_TABLE
                .iter()
                .enumerate()
                .map(|(i, &(input, output))| SinLUTRow {
                    node_id: BaseField::from_u32_unchecked(i as u32),
                    input,
                    output,
                    multiplicity: BaseField::one(),
                })
                .collect(),
        }
    }

    /// Converts the LUT into a padded CircleEvaluation trace for use in LogUp.
    pub fn to_trace_eval(&self) -> Result<TraceEval, TraceError> {
        let n = self.table.len();
        let log_size = calculate_log_size(n);
        let trace_size = 1 << log_size;

        let mut node_ids = BaseColumn::zeros(trace_size);
        let mut inputs = BaseColumn::zeros(trace_size);
        let mut outputs = BaseColumn::zeros(trace_size);
        let mut mults = BaseColumn::zeros(trace_size);

        for (i, row) in self.table.iter().enumerate() {
            node_ids.set(i, row.node_id);
            inputs.set(i, row.input);
            outputs.set(i, row.output);
            mults.set(i, row.multiplicity);
        }

        let domain = CanonicCoset::new(log_size).circle_domain();

        let mut trace = Vec::with_capacity(4);

        trace.push(CircleEvaluation::new(domain, node_ids));
        trace.push(CircleEvaluation::new(domain, inputs));
        trace.push(CircleEvaluation::new(domain, outputs));
        trace.push(CircleEvaluation::new(domain, mults));

        Ok(trace)
    }
}

/// Builds the interaction trace for the LogUp argument using two separate lookups:
///
/// 1. Verifies that all `input` values used by the prover exist in the LUT table.
/// 2. Verifies that each `(input, output)` pair used by the prover also exists in the LUT (i.e., output is correct).
///
/// Both lookups are checked using the fractional LogUp sumcheck equation:
///     ∑_LUT (m / (z - H(...))) - ∑_Prover (1 / (z - H(...))) == 0
///
/// This guarantees:
/// - Prover used only whitelisted `input` values
/// - Prover produced correct `output` values for each input
///
/// # Parameters:
/// - `main_trace_eval`: TraceEval from the prover's execution trace (e.g., SinTable)
/// - `lookup_elements`: Fiat-Shamir challenge structure for hashing values
///
/// # Returns:
/// A combined interaction trace and claimed sum, to be verified during STARK execution.
pub fn interaction_trace_evaluation(
    main_trace_eval: &TraceEval,
    lookup_elements: &NodeElements,
) -> Result<(TraceEval, InteractionClaim), TraceError> {
    if main_trace_eval.is_empty() {
        return Err(TraceError::EmptyTrace);
    }

    // === Setup ===
    let log_size = main_trace_eval[0].domain.log_size();
    let mut logup_gen = LogupTraceGenerator::new(log_size);

    // == Create Lookup Trace ==

    let sin_table = SinLUTTable::from_lookup_table();

    let lut_trace = sin_table.to_trace_eval().unwrap();

    // ------------------------------------------------
    // LogUp #1: Verify input ∈ LUT
    // ------------------------------------------------

    // Prover side: input + input_id
    let input_col = &main_trace_eval[SinColumn::Input.index()].data;
    let input_id_col = &main_trace_eval[SinColumn::InputId.index()].data;
    let input_mult_col = &main_trace_eval[SinColumn::InputMult.index()].data;

    let mut input_trace = logup_gen.new_col();
    for row in 0..1 << (log_size - LOG_N_LANES) {
        let input = input_col[row];
        let id = input_id_col[row];
        let hash = lookup_elements.combine(&[input, id]);
        let mutliplicity = input_mult_col[row];
        input_trace.write_frac(row, mutliplicity.into(), hash);
    }
    input_trace.finalize_col();

    // LUT side: input + node_id
    let lut_input_id_col = &lut_trace[0].data; // node_id
    let lut_input_col = &lut_trace[1].data; // input
    let lut_mult_col = &lut_trace[3].data; // multiplicity

    let mut lut_input_trace = logup_gen.new_col();
    for row in 0..lut_input_col.len() {
        let input = lut_input_col[row];
        let id = lut_input_id_col[row];
        let multiplicity = lut_mult_col[row];
        let hash = lookup_elements.combine(&[input, id]);
        lut_input_trace.write_frac(row, multiplicity.into(), hash);
    }
    lut_input_trace.finalize_col();

    // ------------------------------------------------
    // LogUp #2: Verify (input, output) pair ∈ LUT
    // ------------------------------------------------

    // Prover side: input + output + node_id
    let output_col = &main_trace_eval[SinColumn::Output.index()].data;
    let output_node_id_col = &main_trace_eval[SinColumn::NodeId.index()].data;

    let mut output_trace = logup_gen.new_col();
    for row in 0..1 << (log_size - LOG_N_LANES) {
        let input = input_col[row];
        let output = output_col[row];
        let node_id = output_node_id_col[row];
        let hash = lookup_elements.combine(&[input, output, node_id]);
        output_trace.write_frac(row, (-PackedM31::one()).into(), hash);
    }
    output_trace.finalize_col();

    // LUT side: input + output + node_id
    let lut_output_col = &lut_trace[2].data; // output

    let mut lut_output_trace = logup_gen.new_col();
    for row in 0..lut_input_col.len() {
        let input = lut_input_col[row];
        let output = lut_output_col[row];
        let node_id = lut_input_id_col[row];
        let multiplicity = lut_mult_col[row];
        let hash = lookup_elements.combine(&[input, output, node_id]);
        lut_output_trace.write_frac(row, multiplicity.into(), hash);
    }
    lut_output_trace.finalize_col();

    // ------------------------------------------------
    // Finalize LogUp Interaction Trace
    // ------------------------------------------------

    // Final sum will be zero only if all prover values matched the LUT exactly.
    let (interaction_trace, claimed_sum) = logup_gen.finalize_last();

    Ok((interaction_trace, InteractionClaim { claimed_sum }))
}
