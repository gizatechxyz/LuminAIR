use crate::components::{NodeElements, RecipClaim};
use num_traits::{One, Zero};
use numerair::eval::EvalFixedPoint;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// Component for element-wise reciprocal operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type RecipComponent = FrameworkComponent<RecipEval>;

/// Defines the AIR for the reciprocal component.
pub struct RecipEval {
    log_size: u32,
    lookup_elements: NodeElements,
}

impl RecipEval {
    pub fn new(claim: &RecipClaim, lookup_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for RecipEval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

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

        // Multiplicities for interaction constraints
        let lhs_mult = eval.next_trace_mask();
        let rhs_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // The output value must equal the product of the input values.
        eval.eval_fixed_mul(
            lhs_val.clone(),
            out_val.clone(),
            E::F::one(),
            E::F::one(),
            E::F::zero(),
        ); //TODO: check rem equal zero 

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
