use stwo_prover::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::Log2LookupClaim;

use super::Log2LookupElements;

/// The STWO AIR component for the Log2 Lookup Table (LUT) argument.
///
/// This component ensures that the multiplicities recorded for each entry of the
/// preprocessed Log2 LUT correctly correspond to the actual values in the LUT.
/// It works in conjunction with the `Log2Component` which records accesses.
pub type Log2LookupComponent = FrameworkComponent<Log2LookupEval>;

/// Defines the AIR constraints evaluation logic for the Log2Lookup component.
/// Implements `FrameworkEval` to connect the multiplicity trace with the preprocessed LUT.
pub struct Log2LookupEval {
    /// Log2 size of the component's main trace segment.
    log_size: u32,
    /// Interaction elements specific to the Log2 LUT LogUp.
    lookup_elements: Log2LookupElements,
}

impl Log2LookupEval {
    /// Creates a new `Log2LookupEval` instance.
    /// Takes the component's claim (for `log_size`) and Log2 LUT interaction elements.
    pub fn new(claim: &Log2LookupClaim, lookup_elements: Log2LookupElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the Log2Lookup component.
impl FrameworkEval for Log2LookupEval {
    /// Returns the log2 size of this component's main trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum expected log2 degree bound for the component's constraints.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the Log2Lookup AIR constraints on a given evaluation point (`eval`).
    ///
    /// This component has one primary role: to add terms to the LogUp sum that correspond
    /// to the preprocessed Log2 Lookup Table entries, weighted by their recorded multiplicities.
    ///
    /// 1. Retrieves the preprocessed Log2 LUT columns (`log2_lut_0` for inputs, `log2_lut_1` for outputs).
    /// 2. Retrieves the `multiplicity` from the Log2Lookup component's main trace.
    /// 3. Adds an entry to the LogUp relation:
    ///    - Numerator: `-multiplicity` (negative because these are the "table side" entries).
    ///    - Denominator: Combination of `(log2_lut_0, log2_lut_1)` with `self.lookup_elements`.
    /// This constraint, when combined with the corresponding positive terms from `Log2Component`,
    /// ensures that `sum (access_multiplicity / P(access_val)) - sum (table_multiplicity / P(table_val)) = 0`,
    /// thus proving that values looked up via `Log2Component` correctly match the preprocessed LUT.
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