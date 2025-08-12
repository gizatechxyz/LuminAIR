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

/// Logarithm base-2 lookup table structure for storing layout and multiplicities
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Log2Lookup {
    pub layout: LookupLayout,
    pub multiplicities: AtomicMultiplicityColumn,
}

impl Log2Lookup {
    /// Creates a new Log2Lookup with the given layout
    pub fn new(layout: &LookupLayout) -> Self {
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            multiplicities,
        }
    }

    /// Adds multiplicities to the trace table
    pub fn add_multiplicities_to_table(&self, table: &mut Log2LookupTraceTable) {
        for mult in &self.multiplicities.data {
            table.add_row(Log2LookupTraceTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}