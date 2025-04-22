use crate::components::{Log2Claim, NodeElements};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// Component for computing base-2 logarithm operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type Log2Component = FrameworkComponent<Log2Eval>;

/// Defines the AIR for the logarithm base-2 component.
pub struct Log2Eval {
    log_size: u32,
    lookup_elements: NodeElements,
}

impl Log2Eval {
    /// Creates a new `Log2Eval` instance from a claim and lookup elements.
    pub fn new(claim: &Log2Claim, lookup_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for Log2Eval {
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

    /// Evaluates the AIR constraints for the logarithm base-2 operation.
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

        // Values for consistency constraints
        let input_val = eval.next_trace_mask(); // Value from input tensor at index.
        let output_val = eval.next_trace_mask(); // Value in output tensor at index (the logarithm).

        // Additional values for logarithm verification
        let pow2_result = eval.next_trace_mask(); // 2^output_val which should equal input_val

        // Multiplicities for interaction constraints
        let input_mult = eval.next_trace_mask();
        let output_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // The logarithm constraint: we verify that 2^output_val = input_val
        // We implement this by enforcing that pow2_result = input_val
        eval.add_constraint(pow2_result.clone() - input_val.clone());

        // We need to ensure pow2_result is indeed 2^output_val
        // Note: In a real implementation, we would need to use bit decomposition
        // or range checks to properly verify this relationship
        // This is a simplified placeholder that would need enhancement

        // ┌────────────────────────────┐
        // │   Transition Constraints   │
        // └────────────────────────────┘

        // If this is not the last index for this operation, then:
        // 1. The next row should be for the same operation on the same tensors.
        // 2. The index should increment by 1.
        let not_last = E::F::one() - is_last_idx;

        // Same node ID
        eval.add_constraint(not_last.clone() * (next_node_id - node_id.clone()));

        // Same tensor ID
        eval.add_constraint(not_last.clone() * (next_input_id - input_id.clone()));

        // Index increment by 1
        eval.add_constraint(not_last * (next_idx - idx - E::F::one()));

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
            output_mult.into(),
            &[output_val, node_id],
        ));

        eval.finalize_logup();

        eval
    }
}
