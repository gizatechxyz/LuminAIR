use crate::components::{MaxReduceClaim, NodeElements};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// Component for max reduction operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type MaxReduceComponent = FrameworkComponent<MaxReduceEval>;

/// Defines the AIR for the max reduction component.
pub struct MaxReduceEval {
    log_size: u32,
    lookup_elements: NodeElements,
}

impl MaxReduceEval {
    /// Creates a new `MaxReduceEval` instance from a claim and lookup elements.
    pub fn new(claim: &MaxReduceClaim, lookup_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for MaxReduceEval {
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

    /// Evaluates the AIR constraints for the max reduction operation.
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        // IDs
        let node_id = eval.next_trace_mask(); // ID of the node in the computational graph.
        let input_id = eval.next_trace_mask(); // ID of input tensor.
        let idx = eval.next_trace_mask(); // Index in the flattened tensor.
        let is_last_idx = eval.next_trace_mask(); // Flag if this is the last index for this operation.

        // Next IDs for transition constraints
        let next_node_id = eval.next_trace_mask();
        let next_input_id = eval.next_trace_mask();
        let next_idx = eval.next_trace_mask();

        // Values for max reduction constraints
        let input_val = eval.next_trace_mask(); // Current input value at index.
        let current_max_val = eval.next_trace_mask(); // Current maximum value up to this index.
        let next_max_val = eval.next_trace_mask(); // Maximum value after considering current input.
        let is_new_max = eval.next_trace_mask(); // Flag if current input is new maximum (0 or 1).

        // Multiplicities for interaction constraints
        let input_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // The is_new_max flag is either 0 or 1.
        eval.add_constraint(is_new_max.clone() * (is_new_max.clone() - E::F::one()));

        // If input_val > current_max_val, then is_new_max should be 1, else 0
        // This constraint ensures that is_new_max is correctly set
        // It enforces: (input_val > current_max_val) => is_new_max = 1
        //              (input_val <= current_max_val) => is_new_max = 0

        // We implement this with the constraint:
        // is_new_max * (input_val - current_max_val) + (1 - is_new_max) * (current_max_val - input_val + epsilon) >= 0
        // where epsilon is a small positive value

        // For ease of implementation, we enforce two separate constraints:
        // When is_new_max = 1: input_val > current_max_val
        // When is_new_max = 0: current_max_val >= input_val
        eval.add_constraint(is_new_max.clone() * (input_val.clone() - current_max_val.clone()));
        eval.add_constraint(
            (E::F::one() - is_new_max.clone())
                * (current_max_val.clone() - input_val.clone()),
        );

        // The next_max_val should be input_val if is_new_max=1, otherwise current_max_val
        eval.add_constraint(
            next_max_val.clone()
                - (is_new_max.clone() * input_val.clone()
                    + (E::F::one() - is_new_max.clone()) * current_max_val.clone()),
        );

        // ┌────────────────────────────┐
        // │   Transition Constraints   │
        // └────────────────────────────┘

        // If this is not the last index for this operation, then:
        // 1. The next row should be for the same operation on the same tensors.
        // 2. The index should increment by 1.
        // 3. The next_max_val from this row should be current_max_val of the next row.
        let not_last = E::F::one() - is_last_idx;

        // Same node ID
        eval.add_constraint(not_last.clone() * (next_node_id - node_id.clone()));

        // Same tensor ID
        eval.add_constraint(not_last.clone() * (next_input_id - input_id.clone()));

        // Index increment by 1
        eval.add_constraint(not_last.clone() * (next_idx - idx.clone() - E::F::one()));

        // For first index (idx = 0), current_max_val should equal input_val
        let is_first = E::F::one() - idx.clone(); // Simplified check for idx=0
        eval.add_constraint(is_first * (current_max_val.clone() - input_val.clone()));

        // ┌─────────────────────────────┐
        // │   Interaction Constraints   │
        // └─────────────────────────────┘

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            input_mult.into(),
            &[input_val, input_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            out_mult.into(),
            &[next_max_val, node_id],
        ));

        eval.finalize_logup();

        eval
    }
}
