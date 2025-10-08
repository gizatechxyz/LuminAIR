use crate::{
    components::{MulClaim, NodeElements},
};
use num_traits::One;
use numerair::eval::EvalFixedPoint;
// Scale is now extracted from trace data
use stwo_constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry};

pub type MulComponent = FrameworkComponent<MulEval>;

/// Evaluation structure for multiplication operations
pub struct MulEval {
    log_size: u32,
    node_elements: NodeElements,
}

impl MulEval {
    /// Creates a new MulEval with the given claim and node elements
    pub fn new(claim: &MulClaim, node_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_elements,
        }
    }
}

impl FrameworkEval for MulEval {
    /// Returns the log size of the evaluation
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum constraint log degree bound
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the multiplication constraints and relations
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        // IDs
        let node_id = eval.next_trace_mask(); // ID of the node in the computational graph.
        let lhs_id = eval.next_trace_mask(); // ID of first input tensor.
        let rhs_id = eval.next_trace_mask(); // ID of second input tensor.
        let idx = eval.next_trace_mask(); // Index in the flattened tensor.
        let is_last_idx = eval.next_trace_mask(); // Flag if this is the last index for this operation.

        // Next IDs for transition constraints
        let next_node_id = eval.next_trace_mask();
        let next_lhs_id = eval.next_trace_mask();
        let next_rhs_id = eval.next_trace_mask();
        let next_idx = eval.next_trace_mask();

        // Values for consistency constraints
        let lhs_val = eval.next_trace_mask(); // Value from first tensor at index.
        let rhs_val = eval.next_trace_mask(); // Value from second tensor at index.
        let out_val = eval.next_trace_mask(); // Value in output tensor at index.
        let rem_val = eval.next_trace_mask(); // Rem value in result tensor at index.

        // Extract dynamic scale from trace data (attrs field in MulColumn)  
        let scale_factor = eval.next_trace_mask(); // Scale is at column index 13

        // Multiplicities for interaction constraints
        let lhs_mult = eval.next_trace_mask(); // LhsMult is at column index 14
        let rhs_mult = eval.next_trace_mask(); // RhsMult is at column index 15
        let out_mult = eval.next_trace_mask(); // OutMult is at column index 16

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // Evaluates fixed point multiplication.
        eval.eval_fixed_mul(
            lhs_val.clone(),
            rhs_val.clone(),
            scale_factor.clone(),
            out_val.clone(),
            rem_val,
        );
        // ┌────────────────────────────┐
        // │   Transition Constraints   │
        // └────────────────────────────┘

        // If this is not the last index for this operation, then:
        // 1. The next row should be for the same operation on the same tensors.
        // 2. The index should increment by 1.
        let not_last = E::F::one() - is_last_idx;

        // Same node ID
        eval.add_constraint(not_last.clone() * (next_node_id - node_id.clone()));

        // Same tensor IDs
        eval.add_constraint(not_last.clone() * (next_lhs_id - lhs_id.clone()));
        eval.add_constraint(not_last.clone() * (next_rhs_id - rhs_id.clone()));

        // Index increment by 1
        eval.add_constraint(not_last * (next_idx - idx - E::F::one()));

        // ┌─────────────────────────────┐
        // │   Interaction Constraints   │
        // └─────────────────────────────┘

        // Values are already scaled in the trace, no need to re-scale

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            lhs_mult.into(),
            &[lhs_val, lhs_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            rhs_mult.into(),
            &[rhs_val, rhs_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            out_mult.into(),
            &[out_val, node_id],
        ));

        eval.finalize_logup();

        eval
    }
}
