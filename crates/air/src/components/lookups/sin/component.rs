use stwo_prover::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::SinLookupClaim;

use super::SinLookupElements;

/// Component for sin lookup, using `SimdBackend` with fallback to `CpuBackend` for small traces.
pub type SinLookupComponent = FrameworkComponent<SinLookupEval>;

/// Defines the AIR for the sin lookup component.
pub struct SinLookupEval {
    log_size: u32,
    lookup_elements: SinLookupElements,
}

impl SinLookupEval {
    /// Creates a new `SinLookupEval` instance from a claim and node elements.
    pub fn new(claim: &SinLookupClaim, lookup_elements: SinLookupElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for SinLookupEval {
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

    /// Evaluates the AIR constraints for the recip operation.
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let sin_lut_0 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "sin_lut_0".to_string(),
        });
        let sin_lut_1 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "sin_lut_1".to_string(),
        });

        let multiplicity = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            -E::EF::from(multiplicity),
            &[sin_lut_0, sin_lut_1],
        ));

        eval.finalize_logup();

        eval
    }
}
