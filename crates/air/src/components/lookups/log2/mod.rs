use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use stwo_prover::{core::fields::m31::BaseField, relation};

use crate::{
    components::lookups::log2::table::{Log2LookupTraceTable, Log2LookupTraceTableRow},
    preprocessed::LookupLayout,
    utils::AtomicMultiplicityColumn,
};

pub mod component;
pub mod table;
pub mod witness;

// Interaction elements specifically for the Log2 Lookup Table argument.
// Drawn from the channel, used to combine `(input, output)` pairs from the Log2 LUT.
relation!(Log2LookupElements, 2);

/// Configuration and data for the Log2 Lookup Table (LUT).
///
/// Holds the `LookupLayout` (defining value ranges and size), the actual LUT data
/// (`Log2LookupData`), and an `AtomicMultiplicityColumn` to track accesses to LUT entries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Log2Lookup {
    /// The layout defining the structure and value ranges of the Log2 LUT.
    pub layout: LookupLayout,
    /// Atomic counters tracking the number of times each LUT entry is accessed.
    pub multiplicities: AtomicMultiplicityColumn,
}

impl Log2Lookup {
    /// Creates a new `Log2Lookup` instance based on the provided `LookupLayout`.
    ///
    /// Initializes the `Log2LookupData` (LUT values) from the layout and creates
    /// an `AtomicMultiplicityColumn` of the appropriate size (padded to power of two).
    pub fn new(layout: &LookupLayout) -> Self {
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            multiplicities,
        }
    }

    /// Populates a `Log2LookupTraceTable` with the final multiplicity counts.
    ///
    /// This table is used by the `Log2LookupComponent` to generate the trace columns
    /// for proving the lookup argument (i.e., that the sum of multiplicities matches accesses).
    pub fn add_multiplicities_to_table(&self, table: &mut Log2LookupTraceTable) {
        for mult in &self.multiplicities.data {
            table.add_row(Log2LookupTraceTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}