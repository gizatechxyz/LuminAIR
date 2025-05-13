use crate::components::{NodeElements, SumReduceClaim};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// The STWO AIR component for Sum-Reduce operations.
/// Wraps the `SumReduceEval` logic within the STWO `FrameworkComponent`.
pub type SumReduceComponent = FrameworkComponent<SumReduceEval>;

/// Defines the AIR constraints evaluation logic for the SumReduce component.
/// Implements `FrameworkEval` to define trace layout, degrees, and constraints
/// for the step-by-step accumulation process.
pub struct SumReduceEval {
    /// Log2 size of the component's trace segment.
    log_size: u32,
    /// Interaction elements for node relations (used in LogUp).
    node_elements: NodeElements,
}

impl SumReduceEval {
    /// Creates a new `SumReduceEval` instance.
    /// Takes the component's claim (for `log_size`) and interaction elements.
    pub fn new(claim: &SumReduceClaim, node_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the SumReduce component.
impl FrameworkEval for SumReduceEval {
    /// Returns the log2 size of this component's trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum expected log2 degree bound for the component's constraints.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the SumReduce AIR constraints on a given evaluation point (`eval`).
    ///
    /// Defines constraints for:
    /// - **Consistency:**
    ///   - `is_last_idx` and `is_last_step` are boolean.
    ///   - Accumulator update: `next_acc = acc + input`.
    ///   - Output validity: `out = next_acc` only if `is_last_step` is true.
    /// - **Transition (for output elements):** When `is_last_idx` is false (more output elements for this node):
    ///   - Node and input tensor IDs remain the same.
    ///   - `idx` (output element index) increments by 1.
    /// - **Interaction (LogUp):** Links `input_val` (from input tensor) and `out_val` (final sum)
    ///   to the global LogUp argument.
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
        let acc_val = eval.next_trace_mask(); // Accumulative value in result tensor at index.
        let next_acc_val = eval.next_trace_mask(); // Next accumulative value.
        let is_last_step = eval.next_trace_mask(); // Flag if this is the last step.

        // Multiplicities for interaction constraints
        let input_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx and is_last_step flags are either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));
        eval.add_constraint(is_last_step.clone() * (is_last_step.clone() - E::F::one()));

        // The output value must equal the sum of the input values.
        eval.add_constraint(next_acc_val.clone() - (acc_val.clone() + input_val.clone()));
        eval.add_constraint((out_val.clone() - next_acc_val) * is_last_step);

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
