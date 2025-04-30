use crate::{
    components::{
        InteractionClaim, NodeElements, SumReduceClaim, TraceColumn, TraceError, TraceEval,
    },
    utils::calculate_log_size,
};
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

/// Represents the trace for the SumReduce component, containing the required registers for its
/// constraints.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SumReduceTable {
    /// A vector of [`SumReduceTableRow`] representing the table rows.
    pub table: Vec<SumReduceTableRow>,
}

/// Represents a single row of the [`SumReduceTable`]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SumReduceTableRow {
    pub node_id: BaseField,
    pub input_id: BaseField,
    pub idx: BaseField,
    pub is_last_idx: BaseField,
    pub next_node_id: BaseField,
    pub next_input_id: BaseField,
    pub next_idx: BaseField,
    pub input: BaseField,
    pub out: BaseField,
    pub acc: BaseField,
    pub next_acc: BaseField,
    pub is_last_step: BaseField,
    pub input_mult: BaseField,
    pub out_mult: BaseField,
}

impl SumReduceTable {
    /// Creates a new, empty [`SumReduceTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the SumReduce Table.
    pub fn add_row(&mut self, row: SumReduceTableRow) {
        self.table.push(row);
    }

    /// Transforms the [`SumReduceTable`] into [`TraceEval`] to be committed
    /// when generating a STARK proof.
    pub fn trace_evaluation(&self) -> Result<(TraceEval, SumReduceClaim), TraceError> {
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
        let mut out = BaseColumn::zeros(trace_size);
        let mut acc = BaseColumn::zeros(trace_size);
        let mut next_acc = BaseColumn::zeros(trace_size);
        let mut is_last_step = BaseColumn::zeros(trace_size);
        let mut input_mult = BaseColumn::zeros(trace_size);
        let mut out_mult = BaseColumn::zeros(trace_size);

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
            out.set(vec_row, row.out);
            acc.set(vec_row, row.acc);
            next_acc.set(vec_row, row.next_acc);
            is_last_step.set(vec_row, row.is_last_step);
            input_mult.set(vec_row, row.input_mult);
            out_mult.set(vec_row, row.out_mult);
        }

        for i in self.table.len()..trace_size {
            is_last_idx.set(i, BaseField::one());
        }

        // Create domain
        let domain = CanonicCoset::new(log_size).circle_domain();

        // Create trace
        let mut trace = Vec::with_capacity(SumReduceColumn::count().0);
        trace.push(CircleEvaluation::new(domain, node_id));
        trace.push(CircleEvaluation::new(domain, input_id));
        trace.push(CircleEvaluation::new(domain, idx));
        trace.push(CircleEvaluation::new(domain, is_last_idx));
        trace.push(CircleEvaluation::new(domain, next_node_id));
        trace.push(CircleEvaluation::new(domain, next_input_id));
        trace.push(CircleEvaluation::new(domain, next_idx));
        trace.push(CircleEvaluation::new(domain, input));
        trace.push(CircleEvaluation::new(domain, out));
        trace.push(CircleEvaluation::new(domain, acc));
        trace.push(CircleEvaluation::new(domain, next_acc));
        trace.push(CircleEvaluation::new(domain, is_last_step));
        trace.push(CircleEvaluation::new(domain, input_mult));
        trace.push(CircleEvaluation::new(domain, out_mult));

        assert_eq!(trace.len(), SumReduceColumn::count().0);

        Ok((trace, SumReduceClaim::new(log_size)))
    }
}

/// Enum representing the column indices in the SumReduce trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SumReduceColumn {
    NodeId,
    InputId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextInputId,
    NextIdx,
    Input,
    Out,
    Acc,
    NextAcc,
    IsLastStep,
    InputMult,
    OutMult,
}

impl SumReduceColumn {
    /// Returns the index of the column in the SumReduce trace.
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
            Self::Out => 8,
            Self::Acc => 9,
            Self::NextAcc => 10,
            Self::IsLastStep => 11,
            Self::InputMult => 12,
            Self::OutMult => 13,
        }
    }
}
impl TraceColumn for SumReduceColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (14, 2)
    }
}

/// Generates the interaction trace for the SumReduce component using the main trace and node elements.
pub fn interaction_trace_evaluation(
    main_trace_eval: &TraceEval,
    node_elements: &NodeElements,
) -> Result<(TraceEval, InteractionClaim), TraceError> {
    if main_trace_eval.is_empty() {
        return Err(TraceError::EmptyTrace);
    }

    let log_size = main_trace_eval[0].domain.log_size();
    let mut logup_gen = LogupTraceGenerator::new(log_size);

    // Create trace for Input
    let input_main_col = &main_trace_eval[SumReduceColumn::Input.index()].data;
    let input_id_col = &main_trace_eval[SumReduceColumn::InputId.index()].data;
    let input_mult_col = &main_trace_eval[SumReduceColumn::InputMult.index()].data;
    let mut input_int_col = logup_gen.new_col();
    for row in 0..1 << (log_size - LOG_N_LANES) {
        let input = input_main_col[row];
        let id = input_id_col[row];
        let multiplicity = input_mult_col[row];

        input_int_col.write_frac(
            row,
            multiplicity.into(),
            node_elements.combine(&[input, id]),
        );
    }
    input_int_col.finalize_col();

    // Create trace for OUTPUT
    let out_main_col = &main_trace_eval[SumReduceColumn::Out.index()].data;
    let node_id_col = &main_trace_eval[SumReduceColumn::NodeId.index()].data;
    let out_mult_col = &main_trace_eval[SumReduceColumn::OutMult.index()].data;
    let mut out_int_col = logup_gen.new_col();
    for row in 0..1 << (log_size - LOG_N_LANES) {
        let out = out_main_col[row];
        let id = node_id_col[row];
        let multiplicity = out_mult_col[row];

        out_int_col.write_frac(row, multiplicity.into(), node_elements.combine(&[out, id]));
    }
    out_int_col.finalize_col();

    let (trace, claimed_sum) = logup_gen.finalize_last();

    Ok((trace, InteractionClaim { claimed_sum }))
}
