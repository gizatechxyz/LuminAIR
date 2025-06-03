use stwo_prover::constraint_framework::{
    preprocessed_columns::PreProcessedColumnId, EvalAtRow, FrameworkComponent, FrameworkEval,
    RelationEntry,
};

use crate::components::lookups::exp2::witness::Exp2LookupClaim;
use super::{table::Exp2LookupColumn, Exp2LookupElements};

/// The STWO AIR component for the Exp2 Lookup Table (LUT) argument.
///
/// This component ensures that the multiplicities recorded for each entry of the
/// preprocessed Exp2 LUT correctly correspond to the actual values in the LUT.
/// It works in conjunction with the `Exp2Component` which records accesses.
pub type Exp2LookupComponent = FrameworkComponent<Exp2LookupEval>;

/// Defines the AIR constraints evaluation logic for the Exp2Lookup component.
/// Implements `FrameworkEval` to connect the multiplicity trace with the preprocessed LUT.
pub struct Exp2LookupEval {
    /// Log2 size of the component's main trace segment.
    log_size: u32,
    /// Interaction elements specific to the Exp2 LUT LogUp.
    lookup_elements: Exp2LookupElements,
}

impl Exp2LookupEval {
    /// Creates a new `Exp2LookupEval` instance.
    /// Takes the component's claim (for `log_size`) and Exp2 LUT interaction elements.
    pub fn new(claim: &Exp2LookupClaim, lookup_elements: Exp2LookupElements) -> Self {
        Self {
            log_size: claim.log_size,
            lookup_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the Exp2Lookup component.
impl FrameworkEval for Exp2LookupEval {
    /// Returns the log2 size of this component's main trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the maximum expected log2 degree bound for the component's constraints.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size + 1
    }

    /// Evaluates the Exp2Lookup AIR constraints on a given evaluation point (`eval`).
    ///
    /// This component has one primary role: to add terms to the LogUp sum that correspond
    /// to the preprocessed Exp2 Lookup Table entries, weighted by their recorded multiplicities.
    ///
    /// 1. Retrieves the preprocessed Exp2 LUT columns (`exp2_lut_0` for inputs, `exp2_lut_1` for outputs).
    /// 2. Retrieves the `multiplicity` from the Exp2Lookup component's main trace.
    /// 3. Adds an entry to the LogUp relation:
    ///    - Numerator: `-multiplicity` (negative because these are the "table side" entries).
    ///    - Denominator: Combination of `(exp2_lut_0, exp2_lut_1)` with `self.lookup_elements`.
    /// This constraint, when combined with the corresponding positive terms from `Exp2Component`,
    /// ensures that values looked up via `Exp2Component` correctly match the preprocessed LUT.
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let exp2_lut_0 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "exp2_lut_0".to_string(),
        });
        let exp2_lut_1 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "exp2_lut_1".to_string(),
        });

        let multiplicity = eval.next_trace_mask();

        // Match EXACTLY the same pattern as the Sin implementation
        eval.add_to_relation(RelationEntry::new(
            &self.lookup_elements,
            -E::EF::from(multiplicity), // Negative for the table side
            &[exp2_lut_0, exp2_lut_1],
        ));

        eval.finalize_logup();

        eval
    }
}
