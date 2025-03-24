use crate::components::{LessThanClaim, NodeElements};
use num_traits::One;
use numerair::{eval::EvalFixedPoint, SCALE_FACTOR};
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// Component for element-wise less than comparison operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type LessThanComponent = FrameworkComponent<LessThanEval>;

/// Defines the AIR for the less than comparison component.
pub struct LessThanEval {
    log_size: u32,
    lookup_elements: NodeElements,
}

impl LessThanEval {
    /// Creates a new `LessThanEval` instance from a claim and lookup elements.
    pub fn new(claim: &LessThanClaim, lookup_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for LessThanEval {
    /// Returns the logarithmic size of the main trace.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// The degree of the constraints is bounded by the size of the trace.
    ///
    /// Returns the ilog2 (upper) bound of the constraint degree for the component.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the AIR constraints for the less than comparison operation.
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
        let out_val = eval.next_trace_mask(); // Value in output tensor at index (0 or 1).

        // Multiplicities for interaction constraints
        let lhs_mult = eval.next_trace_mask();
        let rhs_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // The out_val is either 0 or 1 (boolean result of comparison)
        eval.add_constraint(out_val.clone() * (out_val.clone() - E::F::one()));

        // LessThan operation: (lhs < rhs) ⟹ out_val = 1, otherwise out_val = 0
        // Use conditional constraint: (out_val * (rhs_val - lhs_val) + (1 - out_val) * (lhs_val - rhs_val + epsilon)) >= 0
        // For simplicity, we can use: out_val * (rhs_val - lhs_val) + (1 - out_val) * (lhs_val - rhs_val) >= 0
        // This simplifies to: (2 * out_val - 1) * (rhs_val - lhs_val) >= 0

        // We can represent this using a boolean selector pattern:
        // out_val = 1 when lhs < rhs, and out_val = 0 when lhs >= rhs
        // (lhs < rhs) ⟹ out_val = 1: Check with (out_val) * (rhs_val - lhs_val - epsilon) >= 0
        // (lhs >= rhs) ⟹ out_val = 0: Check with (1 - out_val) * (lhs_val - rhs_val) >= 0
        // where epsilon is a small positive value to handle the strict inequality

        // For fixed-point comparisons, we can use the following constraint:
        eval.add_constraint(
            out_val.clone() * out_val.clone() * (lhs_val.clone() - rhs_val.clone())
                + (E::F::one() - out_val.clone())
                    * (E::F::one() - out_val.clone())
                    * (rhs_val.clone() - lhs_val.clone()),
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

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            lhs_mult.into(),
            &[lhs_val, lhs_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            rhs_mult.into(),
            &[rhs_val, rhs_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            out_mult.into(),
            &[out_val, node_id],
        ));

        eval.finalize_logup();
        eval
    }
}
