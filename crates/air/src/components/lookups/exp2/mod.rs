use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use stwo_prover::{core::fields::m31::BaseField, relation};

use crate::{
    components::lookups::exp2::table::{Exp2LookupTraceTable, Exp2LookupTraceTableRow},
    preprocessed::LookupLayout,
    utils::AtomicMultiplicityColumn,
};

pub mod component;
pub mod table;
pub mod witness;

// Interaction elements specifically for the Exp2 Lookup Table argument.
// Drawn from the channel, used to combine `(input, output)` pairs from the Exp2 LUT.
relation!(Exp2LookupElements, 2);

/// Configuration and data for the Exp2 Lookup Table (LUT).
///
/// Holds the `LookupLayout` (defining value ranges and size), the actual LUT data
/// (`Exp2LookupData`), and an `AtomicMultiplicityColumn` to track accesses to LUT entries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Exp2Lookup {
    /// The layout defining the structure and value ranges of the Exp2 LUT.
    pub layout: LookupLayout,
    /// Atomic counters tracking the number of times each LUT entry is accessed.
    pub multiplicities: AtomicMultiplicityColumn,
}

impl Exp2Lookup {
    /// Creates a new `Exp2Lookup` instance based on the provided `LookupLayout`.
    ///
    /// Initializes the `Exp2LookupData` (LUT values) from the layout and creates
    /// an `AtomicMultiplicityColumn` of the appropriate size (padded to power of two).
    pub fn new(layout: &LookupLayout) -> Self {
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            multiplicities,
        }
    }

    /// Populates a `Exp2LookupTraceTable` with the final multiplicity counts.
    ///
    /// This table is used by the `Exp2LookupComponent` to generate the trace columns
    /// for proving the lookup argument (i.e., that the sum of multiplicities matches accesses).
    pub fn add_multiplicities_to_table(&self, table: &mut Exp2LookupTraceTable) {
        for mult in &self.multiplicities.data {
            table.add_row(Exp2LookupTraceTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}
