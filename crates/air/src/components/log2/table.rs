use num_traits::One;
use serde::{Deserialize, Serialize};
use stwo_prover::{
    constraint_framework::{logup::LogupTraceGenerator, Relation},
    core::{
        backend::{
            simd::{column::BaseColumn, m31::LOG_N_LANES},
            Column,
        },
        fields::m31::BaseField,
        poly::circle::{CanonicCoset, CircleEvaluation},
    },
};

use crate::{
    components::{InteractionClaim, Log2Claim, NodeElements, TraceColumn, TraceError, TraceEval},
    utils::calculate_log_size,
};

/// Represents the trace for the Log2 component, containing the required registers for its
/// constraints.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Log2Table {
    /// A vector of [`Log2TableRow`] representing the table rows.
    pub table: Vec<Log2TableRow>,
}

/// Represents a single row of the [`Log2Table`]
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Log2TableRow {
    pub node_id: BaseField,
    pub input_id: BaseField,
    pub idx: BaseField,
    pub is_last_idx: BaseField,
    pub next_node_id: BaseField,
    pub next_input_id: BaseField,
    pub next_idx: BaseField,
    pub input_val: BaseField,
    pub output_val: BaseField,
    pub pow2_result: BaseField,
    pub input_mult: BaseField,
    pub output_mult: BaseField,
}

impl Log2Table {
    /// Creates a new, empty [`Log2Table`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the Log2 Table.
    pub fn add_row(&mut self, row: Log2TableRow) {
        self.table.push(row);
    }

    /// Transforms the [`Log2Table`] into [`TraceEval`] to be committed
    /// when generating a STARK proof.
    pub fn trace_evaluation(&self) -> Result<(TraceEval, Log2Claim), TraceError> {
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
        let mut input_val = BaseColumn::zeros(trace_size);
        let mut output_val = BaseColumn::zeros(trace_size);
        let mut pow2_result = BaseColumn::zeros(trace_size);
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
            input_val.set(vec_row, row.input_val);
            output_val.set(vec_row, row.output_val);
            pow2_result.set(vec_row, row.pow2_result);
            input_mult.set(vec_row, row.input_mult);
            output_mult.set(vec_row, row.output_mult);
        }

        for i in self.table.len()..trace_size {
            is_last_idx.set(i, BaseField::one());
        }

        // Create domain
        let domain = CanonicCoset::new(log_size).circle_domain();

        // Create trace
        let mut trace = Vec::with_capacity(Log2Column::count().0);
        trace.push(CircleEvaluation::new(domain, node_id));
        trace.push(CircleEvaluation::new(domain, input_id));
        trace.push(CircleEvaluation::new(domain, idx));
        trace.push(CircleEvaluation::new(domain, is_last_idx));
        trace.push(CircleEvaluation::new(domain, next_node_id));
        trace.push(CircleEvaluation::new(domain, next_input_id));
        trace.push(CircleEvaluation::new(domain, next_idx));
        trace.push(CircleEvaluation::new(domain, input_val));
        trace.push(CircleEvaluation::new(domain, output_val));
        trace.push(CircleEvaluation::new(domain, pow2_result));
        trace.push(CircleEvaluation::new(domain, input_mult));
        trace.push(CircleEvaluation::new(domain, output_mult));

        assert_eq!(trace.len(), Log2Column::count().0);

        Ok((trace, Log2Claim::new(log_size)))
    }
}

/// Enum representing the column indices in the Log2 trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Log2Column {
    NodeId,
    InputId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextInputId,
    NextIdx,
    InputVal,
    OutputVal,
    Pow2Result,
    InputMult,
    OutputMult,
}

impl Log2Column {
    /// Returns the index of the column in the Log2 trace.
    pub const fn index(self) -> usize {
        match self {
            Self::NodeId => 0,
            Self::InputId => 1,
            Self::Idx => 2,
            Self::IsLastIdx => 3,
            Self::NextNodeId => 4,
            Self::NextInputId => 5,
            Self::NextIdx => 6,
            Self::InputVal => 7,
            Self::OutputVal => 8,
            Self::Pow2Result => 9,
            Self::InputMult => 10,
            Self::OutputMult => 11,
        }
    }
}

impl TraceColumn for Log2Column {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (12, 2)
    }
}

/// Generates the interaction trace for the Log2 component using the main trace and lookup elements.
pub fn interaction_trace_evaluation(
    main_trace_eval: &TraceEval,
    lookup_elements: &NodeElements,
) -> Result<(TraceEval, InteractionClaim), TraceError> {
    if main_trace_eval.is_empty() {
        return Err(TraceError::EmptyTrace);
    }

    let log_size = main_trace_eval[0].domain.log_size();
    let mut logup_gen = LogupTraceGenerator::new(log_size);

    // Create trace for INPUT
    let input_main_col = &main_trace_eval[Log2Column::InputVal.index()].data;
    let input_id_col = &main_trace_eval[Log2Column::InputId.index()].data;
    let input_mult_col = &main_trace_eval[Log2Column::InputMult.index()].data;
    let mut input_int_col = logup_gen.new_col();
    for row in 0..1 << (log_size - LOG_N_LANES) {
        let input = input_main_col[row];
        let id = input_id_col[row];
        let multiplicity = input_mult_col[row];

        input_int_col.write_frac(
            row,
            multiplicity.into(),
            lookup_elements.combine(&[input, id]),
        );
    }
    input_int_col.finalize_col();

    // Create trace for OUTPUT
    let output_main_col = &main_trace_eval[Log2Column::OutputVal.index()].data;
    let node_id_col = &main_trace_eval[Log2Column::NodeId.index()].data;
    let output_mult_col = &main_trace_eval[Log2Column::OutputMult.index()].data;
    let mut output_int_col = logup_gen.new_col();
    for row in 0..1 << (log_size - LOG_N_LANES) {
        let output = output_main_col[row];
        let id = node_id_col[row];
        let multiplicity = output_mult_col[row];

        output_int_col.write_frac(
            row,
            multiplicity.into(),
            lookup_elements.combine(&[output, id]),
        );
    }
    output_int_col.finalize_col();

    let (trace, claimed_sum) = logup_gen.finalize_last();

    Ok((trace, InteractionClaim { claimed_sum }))
}
