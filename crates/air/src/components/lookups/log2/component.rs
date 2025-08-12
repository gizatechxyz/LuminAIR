use stwo_prover::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::Log2LookupClaim;

use super::Log2LookupElements;

pub type Log2LookupComponent = FrameworkComponent<Log2LookupEval>;

pub struct Log2LookupEval {
    log_size: u32,
    lookup_elements: Log2LookupElements,
}

impl Log2LookupEval {
    pub fn new(claim: &Log2LookupClaim, lookup_elements: Log2LookupElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for Log2LookupEval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let log2_lut_0 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "log2_lut_0".to_string(),
        });
        let log2_lut_1 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "log2_lut_1".to_string(),
        });

        let multiplicity = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            -E::EF::from(multiplicity),
            &[log2_lut_0, log2_lut_1],
        ));

        eval.finalize_logup();

        eval
    }
}