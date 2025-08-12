use crate::components::{MaxReduceClaim, NodeElements};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

pub type MaxReduceComponent = FrameworkComponent<MaxReduceEval>;

pub struct MaxReduceEval {
    log_size: u32,
    node_elements: NodeElements,
}

impl MaxReduceEval {
    pub fn new(claim: &MaxReduceClaim, node_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_elements,
        }
    }
}

impl FrameworkEval for MaxReduceEval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

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
        let max_val = eval.next_trace_mask(); // Current max value.
        let next_max_val = eval.next_trace_mask(); // Next max value.
        let is_last_step = eval.next_trace_mask(); // Flag if this is the last step.
        let is_max = eval.next_trace_mask(); // Flag if current input is the max so far.

        // Multiplicities for interaction constraints
        let input_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx, is_last_step, and is_max flags are either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));
        eval.add_constraint(is_last_step.clone() * (is_last_step.clone() - E::F::one()));
        eval.add_constraint(is_max.clone() * (is_max.clone() - E::F::one()));

        // If is_max is 1, then input_val >= max_val
        // To express this constraint: (input_val - max_val) * is_max >= 0
        // But we need a direct equality constraint, so:
        // If is_max is 1, then input_val == next_max_val (the input becomes the new max)
        // If is_max is 0, then max_val == next_max_val (max doesn't change)
        eval.add_constraint(is_max.clone() * (next_max_val.clone() - input_val.clone()));
        eval.add_constraint(
            (E::F::one() - is_max.clone()) * (next_max_val.clone() - max_val.clone()),
        );

        // The output value must be the maximum value in the last step
        eval.add_constraint((out_val.clone() - next_max_val) * is_last_step);

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
