use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use stwo::core::fields::m31::BaseField;
use stwo_constraint_framework::relation;

use crate::{
    components::lookups::range_check::table::{
        RangeCheckLookupTraceTable, RangeCheckLookupTraceTableRow,
    },
    utils::AtomicMultiplicityColumn,
};

pub mod component;
pub mod table;
pub mod witness;

// Interaction elements specifically for the RangeCheck Lookup Table argument.
relation!(RangeCheckLookupElements, 1);

/// Range check lookup table structure for storing layout and multiplicities
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeCheckLookup<const N: usize> {
    pub layout: RangeCheckLayout<N>,
    pub multiplicities: AtomicMultiplicityColumn,
}

/// Layout structure for range check lookup tables
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeCheckLayout<const N: usize> {
    #[serde_as(as = "[_; N]")]
    pub ranges: [u32; N],
    pub log_size: u32,
}

impl<const N: usize> RangeCheckLookup<N> {
    /// Creates a new RangeCheckLookup with the given layout
    pub fn new(layout: &RangeCheckLayout<N>) -> Self {
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            multiplicities,
        }
    }

    /// Adds multiplicities to the trace table
    pub fn add_multiplicities_to_table(&self, table: &mut RangeCheckLookupTraceTable) {
        for mult in &self.multiplicities.data {
            table.add_row(RangeCheckLookupTraceTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}
