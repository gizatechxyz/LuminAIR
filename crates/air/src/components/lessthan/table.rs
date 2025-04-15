use crate::{
    components::{
        InteractionClaim, LessThanClaim, NodeElements, TraceColumn, TraceError, TraceEval,
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

/// Represents the trace for the LessThan component, containing the required registers for its
/// constraints.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LessThanTable {
    /// A vector of [`LessThanTableRow`] representing the table rows.
    pub table: Vec<LessThanTableRow>,
}

/// Represents a single row of the [`LessThanTable`]
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LessThanTableRow {
    pub node_id: BaseField,
    pub lhs_id: BaseField,
    pub rhs_id: BaseField,
    pub idx: BaseField,
    pub is_last_idx: BaseField,
    pub next_node_id: BaseField,
    pub next_lhs_id: BaseField,
    pub next_rhs_id: BaseField,
    pub next_idx: BaseField,
    pub lhs: BaseField,
    pub rhs: BaseField,
    pub out: BaseField,
    pub lhs_mult: BaseField,
    pub rhs_mult: BaseField,
    pub out_mult: BaseField,
}

impl LessThanTable {
    /// Creates a new, empty [`LessThanTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the LessThan Table.
    pub fn add_row(&mut self, row: LessThanTableRow) {
        self.table.push(row);
    }

    /// Transforms the [`LessThanTable`] into [`TraceEval`] to be commited
    /// when generating a STARK proof.
    pub fn trace_evaluation(&self) -> Result<(TraceEval, LessThanClaim), TraceError> {
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
        let mut lhs_id = BaseColumn::zeros(trace_size);
        let mut rhs_id = BaseColumn::zeros(trace_size);
        let mut idx = BaseColumn::zeros(trace_size);
        let mut is_last_idx = BaseColumn::zeros(trace_size);
        let mut next_node_id = BaseColumn::zeros(trace_size);
        let mut next_lhs_id = BaseColumn::zeros(trace_size);
        let mut next_rhs_id = BaseColumn::zeros(trace_size);
        let mut next_idx = BaseColumn::zeros(trace_size);
        let mut lhs = BaseColumn::zeros(trace_size);
        let mut rhs = BaseColumn::zeros(trace_size);
        let mut out = BaseColumn::zeros(trace_size);
        let mut lhs_mult = BaseColumn::zeros(trace_size);
        let mut rhs_mult = BaseColumn::zeros(trace_size);
        let mut out_mult = BaseColumn::zeros(trace_size);

        // Fill columns
        for (vec_row, row) in self.table.iter().enumerate() {
            node_id.set(vec_row, row.node_id);
            lhs_id.set(vec_row, row.lhs_id);
            rhs_id.set(vec_row, row.rhs_id);
            idx.set(vec_row, row.idx);
            is_last_idx.set(vec_row, row.is_last_idx);
            next_node_id.set(vec_row, row.next_node_id);
            next_lhs_id.set(vec_row, row.next_lhs_id);
            next_rhs_id.set(vec_row, row.next_rhs_id);
            next_idx.set(vec_row, row.next_idx);
            lhs.set(vec_row, row.lhs);
            rhs.set(vec_row, row.rhs);
            out.set(vec_row, row.out);
            lhs_mult.set(vec_row, row.lhs_mult);
            rhs_mult.set(vec_row, row.rhs_mult);
            out_mult.set(vec_row, row.out_mult);
        }

        // For padding, ensure is_last_idx is 1
        for i in self.table.len()..trace_size {
            is_last_idx.set(i, BaseField::one());
        }

        // Create domain
        let domain = CanonicCoset::new(log_size).circle_domain();

        // Create trace
        let mut trace = Vec::with_capacity(LessThanColumn::count().0);
        trace.push(CircleEvaluation::new(domain, node_id));
        trace.push(CircleEvaluation::new(domain, lhs_id));
        trace.push(CircleEvaluation::new(domain, rhs_id));
        trace.push(CircleEvaluation::new(domain, idx));
        trace.push(CircleEvaluation::new(domain, is_last_idx));
        trace.push(CircleEvaluation::new(domain, next_node_id));
        trace.push(CircleEvaluation::new(domain, next_lhs_id));
        trace.push(CircleEvaluation::new(domain, next_rhs_id));
        trace.push(CircleEvaluation::new(domain, next_idx));
        trace.push(CircleEvaluation::new(domain, lhs));
        trace.push(CircleEvaluation::new(domain, rhs));
        trace.push(CircleEvaluation::new(domain, out));
        trace.push(CircleEvaluation::new(domain, lhs_mult));
        trace.push(CircleEvaluation::new(domain, rhs_mult));
        trace.push(CircleEvaluation::new(domain, out_mult));

        assert_eq!(trace.len(), LessThanColumn::count().0);

        Ok((trace, LessThanClaim::new(log_size)))
    }
}

/// Enum representing the column indices in the LessThan trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LessThanColumn {
    NodeId,
    LhsId,
    RhsId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextLhsId,
    NextRhsId,
    NextIdx,
    Lhs,
    Rhs,
    Out,
    LhsMult,
    RhsMult,
    OutMult,
}

impl LessThanColumn {
    /// Returns the index of the column in the LessThan trace.
    pub const fn index(self) -> usize {
        match self {
            Self::NodeId => 0,
            Self::LhsId => 1,
            Self::RhsId => 2,
            Self::Idx => 3,
            Self::IsLastIdx => 4,
            Self::NextNodeId => 5,
            Self::NextLhsId => 6,
            Self::NextRhsId => 7,
            Self::NextIdx => 8,
            Self::Lhs => 9,
            Self::Rhs => 10,
            Self::Out => 11,
            Self::LhsMult => 12,
            Self::RhsMult => 13,
            Self::OutMult => 14,
        }
    }
}

impl TraceColumn for LessThanColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (15, 3)
    }
}

/// Generates the interaction trace for the LessThan component using the main trace and lookup elements.
pub fn interaction_trace_evaluation(
    main_trace_eval: &TraceEval,
    lookup_elements: &NodeElements,
) -> Result<(TraceEval, InteractionClaim), TraceError> {
    if main_trace_eval.is_empty() {
        return Err(TraceError::EmptyTrace);
    }

    let log_size = main_trace_eval[0].domain.log_size();
    let mut logup_gen = LogupTraceGenerator::new(log_size);

    // Create trace for LHS
    let lhs_main_col = &main_trace_eval[LessThanColumn::Lhs.index()].data;
    let lhs_id_col = &main_trace_eval[LessThanColumn::LhsId.index()].data;
    let lhs_mult_col = &main_trace_eval[LessThanColumn::LhsMult.index()].data;
    let mut lhs_int_col = logup_gen.new_col();

    for row in 0..1 << (log_size - LOG_N_LANES) {
        let lhs = lhs_main_col[row];
        let id = lhs_id_col[row];
        let multiplicity = lhs_mult_col[row];
        lhs_int_col.write_frac(
            row,
            multiplicity.into(),
            lookup_elements.combine(&[lhs, id]),
        );
    }
    lhs_int_col.finalize_col();

    // Create trace for RHS
    let rhs_main_col = &main_trace_eval[LessThanColumn::Rhs.index()].data;
    let rhs_id_col = &main_trace_eval[LessThanColumn::RhsId.index()].data;
    let rhs_mult_col = &main_trace_eval[LessThanColumn::RhsMult.index()].data;
    let mut rhs_int_col = logup_gen.new_col();

    for row in 0..1 << (log_size - LOG_N_LANES) {
        let rhs = rhs_main_col[row];
        let id = rhs_id_col[row];
        let multiplicity = rhs_mult_col[row];
        rhs_int_col.write_frac(
            row,
            multiplicity.into(),
            lookup_elements.combine(&[rhs, id]),
        );
    }
    rhs_int_col.finalize_col();

    // Create trace for OUTPUT
    let out_main_col = &main_trace_eval[LessThanColumn::Out.index()].data;
    let node_id_col = &main_trace_eval[LessThanColumn::NodeId.index()].data;
    let out_mult_col = &main_trace_eval[LessThanColumn::OutMult.index()].data;
    let mut out_int_col = logup_gen.new_col();

    for row in 0..1 << (log_size - LOG_N_LANES) {
        let out = out_main_col[row];
        let id = node_id_col[row];
        let multiplicity = out_mult_col[row];
        out_int_col.write_frac(
            row,
            multiplicity.into(),
            lookup_elements.combine(&[out, id]),
        );
    }
    out_int_col.finalize_col();

    let (trace, claimed_sum) = logup_gen.finalize_last();

    Ok((trace, InteractionClaim { claimed_sum }))
}
