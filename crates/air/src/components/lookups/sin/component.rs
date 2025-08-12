use stwo::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::SinLookupClaim;

use super::SinLookupElements;

pub type SinLookupComponent = FrameworkComponent<SinLookupEval>;

/// Evaluation structure for sine lookup table operations
pub struct SinLookupEval {
    log_size: u32,
    lookup_elements: SinLookupElements,
}

impl SinLookupEval {
    /// Creates a new SinLookupEval with the given claim and lookup elements
    pub fn new(claim: &SinLookupClaim, lookup_elements: SinLookupElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for SinLookupEval {
    /// Returns the log size of the evaluation
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum constraint log degree bound
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the sine lookup table constraints and relations
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
