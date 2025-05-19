use crate::components::{NodeElements, SqrtClaim};
use num_traits::One;
use numerair::eval::EvalFixedPoint;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// The STWO AIR component for element-wise sqrt operations.
/// Wraps the `SqrtEval` logic within the STWO `FrameworkComponent`.
pub type SqrtComponent = FrameworkComponent<SqrtEval>;

/// Defines the AIR constraints evaluation logic for the Sqrt component.
/// Implements `FrameworkEval` to define trace layout, degrees, and constraints.
pub struct SqrtEval {
    /// Log2 size of the component's trace segment.
    log_size: u32,
    /// Interaction elements for node relations (used in LogUp).
    node_elements: NodeElements,
}

impl SqrtEval {
    /// Creates a new `SqrtEval` instance.
    /// Takes the component's claim (for `log_size`) and interaction elements.
    pub fn new(claim: &SqrtClaim, node_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the Sqrt component.
impl FrameworkEval for SqrtEval {
    /// Returns the log2 size of this component's trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum expected log2 degree bound for the component's constraints.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the Sqrt AIR constraints on a given evaluation point (`eval`).
    ///
    /// Defines constraints for:
    /// - **Consistency:** Checks the fixed-point sqrt constraint.
    ///   using `eval_fixed_sqrt`.
    /// - **Transition:** Ensures correct state transitions between consecutive rows (same node/input ID,
    ///   index increments by 1) when `is_last_idx` is false.
    /// - **Interaction (LogUp):** Links input and output values to the global LogUp argument.
    /// Receives an evaluator `E` and adds constraint evaluations to it.
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        // IDs
        let node_id = eval.next_trace_mask(); // ID of the node in the computational graph.
        let input_id = eval.next_trace_mask(); // ID of the input tensor.
        let idx = eval.next_trace_mask(); // Index in the flattened tensor.
        let is_last_idx = eval.next_trace_mask(); // Flag if this is the last index for this operation.

        // Next IDs for transition constraints
        let next_node_id = eval.next_trace_mask();
        let next_input_id = eval.next_trace_mask();
        let next_idx = eval.next_trace_mask();

        // Values for consistency constraints
        let input_val = eval.next_trace_mask(); // Value from the tensor at index.
        let out_val = eval.next_trace_mask(); // Value in output tensor at index.
        let rem_val = eval.next_trace_mask(); // Rem value in result tensor at index.
        let scale = eval.next_trace_mask(); // Scale

        // Multiplicities for interaction constraints
        let input_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // Evaluates fixed point sqrt.
        eval.eval_fixed_sqrt(input_val.clone(), out_val.clone(), rem_val, scale);

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
        eval.add_constraint(not_last.clone() * (next_input_id - input_id.clone()));

        // Index increment by 1
        eval.add_constraint(not_last * (next_idx - idx - E::F::one()));

        // ┌─────────────────────────────┐
        // │   Interaction Constraints   │
        // └─────────────────────────────┘

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            input_mult.into(),
            &[input_val, input_id],
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
