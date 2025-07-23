use stwo_prover::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::{lookups::range_check::RangeCheckLookupElements, RangeCheckLookupClaim};

pub type RangeCheckLookupComponent = FrameworkComponent<RangeCheckLookupEval>;

pub struct RangeCheckLookupEval {
    n_bit: u32,
    log_size: u32,
    lookup_elements: RangeCheckLookupElements,
}

impl RangeCheckLookupEval {
    pub fn new(
        n_bit: u32,
        claim: &RangeCheckLookupClaim,
        lookup_elements: RangeCheckLookupElements,
    ) -> Self {
        Self {
            n_bit,
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

impl FrameworkEval for RangeCheckLookupEval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let range_check_lut = eval.get_preprocessed_column(PreProcessedColumnId {
            id: format!("range_check_{:?}_column_0", self.n_bit),
        });

        let multiplicity = eval.next_trace_mask();

        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            -E::EF::from(multiplicity),
            &[range_check_lut],
        ));

        eval.finalize_logup();

        eval
    }
}
