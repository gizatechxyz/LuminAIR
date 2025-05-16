use crate::{
    components::{MulClaim, NodeElements},
    DEFAULT_FP_SCALE,
};
use num_traits::One;
use numerair::eval::EvalFixedPoint;
use stwo_prover::{
    constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry},
    core::fields::m31::M31,
};

/// The STWO AIR component for element-wise multiplication operations.
/// Wraps the `MulEval` logic within the STWO `FrameworkComponent`.
pub type MulComponent = FrameworkComponent<MulEval>;

/// Defines the AIR constraints evaluation logic for the Mul component.
/// Implements `FrameworkEval` to define trace layout, degrees, and constraints.
pub struct MulEval {
    /// Log2 size of the component's trace segment.
    log_size: u32,
    /// Interaction elements for node relations (used in LogUp).
    node_elements: NodeElements,
}

impl MulEval {
    /// Creates a new `MulEval` instance.
    /// Takes the component's claim (for `log_size`) and interaction elements.
    pub fn new(claim: &MulClaim, node_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the Mul component.
impl FrameworkEval for MulEval {
    /// Returns the log2 size of this component's trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum expected log2 degree bound for the component's constraints.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the Mul AIR constraints on a given evaluation point (`eval`).
    ///
    /// Defines constraints for:
    /// - **Consistency:** Checks the fixed-point multiplication relation (`lhs * rhs = out * SCALE + rem`)
    ///   using `eval_fixed_mul`, and boolean flags.
    /// - **Transition:** Ensures correct state transitions between consecutive rows (same node/input IDs,
    ///   index increments by 1) when `is_last_idx` is false.
    /// - **Interaction (LogUp):** Links LHS, RHS, and OUT values to the global LogUp argument.
    /// Receives an evaluator `E` and adds constraint evaluations to it.
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

        // Multiplicities for interaction constraints
        let lhs_mult = eval.next_trace_mask();
        let rhs_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        let scale_factor = E::F::from(M31::from_u32_unchecked(1 << DEFAULT_FP_SCALE));

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // Evaluates fixed point multiplication.
        eval.eval_fixed_mul(
            lhs_val.clone(),
            rhs_val.clone(),
            scale_factor,
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
