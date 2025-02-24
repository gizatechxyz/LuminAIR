use num_traits::{One, Zero};
use numerair::eval::EvalFixedPoint;
use stwo_prover::{
    constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry},
    core::fields::m31::BaseField,
};

use crate::{components::AddClaim, pie::NodeInfo};

use super::table::AddElements;

/// Component for addition operations, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type AddComponent = FrameworkComponent<AddEval>;

/// Defines the AIR for the addition component.
pub struct AddEval {
    log_size: u32,
    lookup_elements: AddElements,
    node_info: NodeInfo,
}

impl AddEval {
    /// Creates a new `AddEval` instance from a claim and lookup elements.
    pub fn new(claim: &AddClaim, lookup_elements: AddElements) -> Self {
        Self {
            log_size: claim.log_size,
            node_info: claim.node_info.clone(),
            lookup_elements,
        }
    }
}

impl FrameworkEval for AddEval {
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

    /// Evaluates the AIR constraints for the addition operation.
    fn evaluate<E: EvalAtRow >(&self, mut eval: E) -> E {
        let lhs = eval.next_trace_mask();
        let rhs = eval.next_trace_mask();
        let out = eval.next_trace_mask();

        eval.eval_fixed_add(lhs.clone(), rhs.clone(), out.clone());

        let lhs_multiplicity = if self.node_info.inputs[0].is_initializer {
            E::EF::zero()
        } else {
            -E::EF::one()
        };

        let rhs_multiplicity = if self.node_info.inputs[1].is_initializer {
            E::EF::zero()
        } else {
            -E::EF::one()
        };
        let out_multiplicity = if self.node_info.output.is_final_output {
            E::EF::zero()
        } else {
            E::EF::one() * BaseField::from_u32_unchecked(self.node_info.num_consumers as u32)
        };

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            lhs_multiplicity,
            &[lhs],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            rhs_multiplicity,
            &[rhs],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            out_multiplicity,
            &[out],
        ));

        eval.finalize_logup();

        eval
    }
}
