use crate::components::{NodeElements, SinClaim};
use num_traits::One;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};
/// Component for element-wise sin operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type SinComponent = FrameworkComponent<SinEval>;

/// Defines the AIR for the sin component.
pub struct SinEval {
    log_size: u32,
    lookup_elements: NodeElements,
}

impl SinEval {
    /// Creates a new `SinEval` instance from a claim and lookup elements.
    pub fn new(claim: &SinClaim, lookup_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
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
        self.log_size + 1
    }

    /// Evaluates the AIR constraints for the sin operation.
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        // IDS
        let node_id = eval.next_trace_mask();
        let input_id = eval.next_trace_mask();
        let idx = eval.next_trace_mask();
        let is_last_idx = eval.next_trace_mask();

        // Next IDS for transition constraints
        let next_node_id = eval.next_trace_mask();
        let next_input_id = eval.next_trace_mask();
        let next_idx = eval.next_trace_mask();

        // Values for consistency constraints
        let input_val = eval.next_trace_mask();
        let output_val = eval.next_trace_mask();

        // Multiplicities for interaction constraints
        let input_mult = eval.next_trace_mask();
        let output_melt = eval.next_trace_mask();

        // let data = <E as EvalAtRow>::F::from(base);

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // TODO Look up arguments
        // let expected_output = SIN_TABLE[]
        // let expecetd  = SIN_LOOKUP_TABLE[];

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
            &self.lookup_elements,
            input_mult.into(),
            &[input_val, input_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            output_melt.into(),
            &[output_val, node_id],
        ));

        eval.finalize_logup();

        eval
    }
}
