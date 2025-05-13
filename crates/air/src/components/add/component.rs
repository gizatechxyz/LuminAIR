use crate::components::{AddClaim, NodeElements};
use num_traits::One;
use numerair::eval::EvalFixedPoint;
use stwo_prover::constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry,
};

/// The STWO AIR component for element-wise addition operations.
///
/// This wraps the `AddEval` logic within the STWO `FrameworkComponent`,
/// which handles common AIR component setup and evaluation.
/// Component for element-wise addition operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type AddComponent = FrameworkComponent<AddEval>;

/// Defines the AIR constraints evaluation logic for the Add component.
///
/// Implements the `FrameworkEval` trait, providing methods to define the component's
/// trace layout, constraint degrees, and the core constraint evaluation function.
pub struct AddEval {
    /// Log2 size of the component's trace segment.
    log_size: u32,
    /// Interaction elements for node relations (used in LogUp).
    node_elements: NodeElements,
}

impl AddEval {
    /// Creates a new `AddEval` instance.
    /// Takes the component's claim (for `log_size`) and interaction elements.
    pub fn new(claim: &AddClaim, node_elements: NodeElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the Add component.
impl FrameworkEval for AddEval {
    /// Returns the log2 size of this component's trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum expected log2 degree bound for the component's constraints.
    /// Used by the framework to configure constraint evaluation domains.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the Add AIR constraints on a given evaluation point (`eval`).
    ///
    /// Defines constraints ensuring:
    /// - **Consistency:** Correctness of individual rows (e.g., `lhs + rhs == out`, boolean flags).
    /// - **Transition:** Correctness of transitions between consecutive rows (e.g., index increments).
    /// - **Interaction (LogUp):** Links values used/produced by Add operations to the global LogUp argument,
    ///   ensuring consistency across the entire computation trace.
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

        // The output value must equal the sum of the input values.
        eval.eval_fixed_add(lhs_val.clone(), rhs_val.clone(), out_val.clone());

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
            &self.node_elements,
            lhs_mult.into(),
            &[lhs_val, lhs_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            rhs_mult.into(),
            &[rhs_val, rhs_id],
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
