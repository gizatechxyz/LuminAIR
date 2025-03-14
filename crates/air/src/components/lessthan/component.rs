use crate::components::{LessThanClaim, NodeElements};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};
/// Component for element-wise addition operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type LessThanComponent = FrameworkComponent<LessThanEval>;

/// Defines the AIR for the less than component.
pub struct LessThanEval {
    log_size: u32,
    lookup_elements: NodeElements,
}

impl LessThanEval {
    pub fn new(claim: &LessThanClaim, lookup_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for LessThanEval {
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

        // Store references to the values instead of moving them
        let lhs_val_ref = &eval.next_trace_mask();
        let rhs_val_ref = &eval.next_trace_mask();
        let out_val_ref = &eval.next_trace_mask();

        // Multiplicities for interaction constraints
        let lhs_mult = eval.next_trace_mask();
        let rhs_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();

        // Constraints
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        let not_last = E::F::one() - is_last_idx;

        // Same node ID
        eval.add_constraint(not_last.clone() * (next_node_id - node_id.clone()));

        // Same tensor IDs
        eval.add_constraint(not_last.clone() * (next_lhs_id - lhs_id.clone()));
        eval.add_constraint(not_last.clone() * (next_rhs_id - rhs_id.clone()));

        // Index increment by 1
        eval.add_constraint(not_last * (next_idx - idx - E::F::one()));

        // Interaction constraints
        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            lhs_mult.into(),
            &[lhs_val_ref.clone(), lhs_id.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            rhs_mult.into(),
            &[rhs_val_ref.clone(), rhs_id.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            out_mult.into(),
            &[out_val_ref.clone(), node_id.clone()],
        ));

        // Ensure out_val is either 0 or 1
        eval.add_constraint(out_val_ref.clone() * (out_val_ref.clone() - E::F::one()));

        // Comparison
        let comparison = lhs_val_ref.clone() - rhs_val_ref.clone();
        eval.add_constraint(out_val_ref.clone() - comparison);

        eval.finalize_logup();

        eval
    }
}
