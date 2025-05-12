use stwo_prover::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::SinLookupClaim;

use super::SinLookupElements;

/// The STWO AIR component for the Sine Lookup Table (LUT) argument.
///
/// This component ensures that the multiplicities recorded for each entry of the
/// preprocessed Sine LUT correctly correspond to the actual values in the LUT.
/// It works in conjunction with the `SinComponent` which records accesses.
pub type SinLookupComponent = FrameworkComponent<SinLookupEval>;

/// Defines the AIR constraints evaluation logic for the SinLookup component.
/// Implements `FrameworkEval` to connect the multiplicity trace with the preprocessed LUT.
pub struct SinLookupEval {
    /// Log2 size of the component's main trace segment.
    log_size: u32,
    /// Interaction elements specific to the Sine LUT LogUp.
    lookup_elements: SinLookupElements,
}

impl SinLookupEval {
    /// Creates a new `SinLookupEval` instance.
    /// Takes the component's claim (for `log_size`) and Sine LUT interaction elements.
    pub fn new(claim: &SinLookupClaim, lookup_elements: SinLookupElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the SinLookup component.
impl FrameworkEval for SinLookupEval {
    /// Returns the log2 size of this component's main trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum expected log2 degree bound for the component's constraints.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the SinLookup AIR constraints on a given evaluation point (`eval`).
    ///
    /// This component has one primary role: to add terms to the LogUp sum that correspond
    /// to the preprocessed Sine Lookup Table entries, weighted by their recorded multiplicities.
    ///
    /// 1. Retrieves the preprocessed Sine LUT columns (`sin_lut_0` for inputs, `sin_lut_1` for outputs).
    /// 2. Retrieves the `multiplicity` from the SinLookup component's main trace.
    /// 3. Adds an entry to the LogUp relation:
    ///    - Numerator: `-multiplicity` (negative because these are the "table side" entries).
    ///    - Denominator: Combination of `(sin_lut_0, sin_lut_1)` with `self.lookup_elements`.
    /// This constraint, when combined with the corresponding positive terms from `SinComponent`,
    /// ensures that `sum (access_multiplicity / P(access_val)) - sum (table_multiplicity / P(table_val)) = 0`,
    /// thus proving that values looked up via `SinComponent` correctly match the preprocessed LUT.
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
