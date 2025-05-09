use crate::components::{
    //lookups::sin::SinLookupElements,
    NodeElements,
    SinClaim,
};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// Component for sin operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type SinComponent = FrameworkComponent<SinEval>;

/// Defines the AIR for the sin component.
pub struct SinEval {
    log_size: u32,
    // lut_log_size: u32,
    node_elements: NodeElements,
    // lookup_elements: SinLookupElements,
}

impl SinEval {
    /// Creates a new `SinEval` instance from a claim and node elements.
    pub fn new(
        claim: &SinClaim,
        node_elements: NodeElements,
        // lookup_elements: SinLookupElements,
        // lut_log_size: u32,
    ) -> Self {
        Self {
            log_size: claim.log_size,
            // lut_log_size,
            node_elements,
            // lookup_elements,
        }
    }
}

impl FrameworkEval for SinEval {
    /// Returns the logarithmic size of the main trace.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// The degree of the constraints is bounded by the size of the trace.
    ///
    /// Returns the ilog2 (upper) bound of the constraint degree for the component.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        // std::cmp::max(self.log_size, self.lut_log_size) + 1
        self.log_size + 1
    }

    /// Evaluates the AIR constraints for the sin operation.
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

        // Multiplicities for interaction constraints
        let input_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();
        // let lookup_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

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
            &[input_val.clone(), input_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            out_mult.into(),
            &[out_val.clone(), node_id],
        ));

        // eval.add_to_relation(RelationEntry::new(
        //     &self.lookup_elements,
        //     lookup_mult.into(),
        //     &[input_val, out_val],
        // ));

        eval.finalize_logup();

        eval
    }
}
