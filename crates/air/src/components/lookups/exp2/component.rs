use stwo_prover::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::Exp2LookupClaim;

use super::Exp2LookupElements;

pub type Exp2LookupComponent = FrameworkComponent<Exp2LookupEval>;

/// Evaluation structure for exponential base-2 lookup table operations
pub struct Exp2LookupEval {
    log_size: u32,
    lookup_elements: Exp2LookupElements,
}

impl Exp2LookupEval {
    /// Creates a new Exp2LookupEval with the given claim and lookup elements
    pub fn new(claim: &Exp2LookupClaim, lookup_elements: Exp2LookupElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for Exp2LookupEval {
    /// Returns the log size of the evaluation
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum constraint log degree bound
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the exponential base-2 lookup table constraints and relations
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let exp2_lut_0 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "exp2_lut_0".to_string(),
        });
        let exp2_lut_1 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "exp2_lut_1".to_string(),
        });

        let multiplicity = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            -E::EF::from(multiplicity),
            &[exp2_lut_0, exp2_lut_1],
        ));

        eval.finalize_logup();

        eval
    }
}
