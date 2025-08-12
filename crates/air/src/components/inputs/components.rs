use crate::components::{InputsClaim, NodeElements};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

pub type InputsComponent = FrameworkComponent<InputsEval>;

/// Evaluation structure for input tensor operations
pub struct InputsEval {
    log_size: u32,
    node_elements: NodeElements,
}

impl InputsEval {
    /// Creates a new InputsEval with the given claim and node elements
    pub fn new(claim: &InputsClaim, node_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_elements,
        }
    }
}

impl FrameworkEval for InputsEval {
    /// Returns the log size of the evaluation
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum constraint log degree bound
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the input tensor constraints and relations
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        // IDs
        let node_id = eval.next_trace_mask();
        let idx = eval.next_trace_mask();
        let is_last_idx = eval.next_trace_mask();

        // Next IDs for transition constraints
        let next_node_id = eval.next_trace_mask();
        let next_idx: <E as EvalAtRow>::F = eval.next_trace_mask();

        // Value for consistency constraints
        let val = eval.next_trace_mask();

        // Multiplicity for interaction constraints
        let multiplicity = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // ┌────────────────────────────┐
        // │   Transition Constraints   │
        // └────────────────────────────┘

        let not_last = E::F::one() - is_last_idx;

        // Same node ID
        eval.add_constraint(not_last.clone() * (next_node_id - node_id.clone()));

        // Index increment by 1
        eval.add_constraint(not_last * (next_idx - idx - E::F::one()));

        // ┌─────────────────────────────┐
        // │   Interaction Constraints   │
        // └─────────────────────────────┘

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            multiplicity.into(),
            &[val, node_id],
        ));

        eval.finalize_logup();

        eval
    }
}
