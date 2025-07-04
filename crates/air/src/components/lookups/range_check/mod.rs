use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use stwo_prover::{core::fields::m31::BaseField, relation};

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

/// Configuration and data for the RangeCheck Lookup Table (LUT).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeCheckLookup<const N: usize> {
    /// The layout defining the structure and value ranges of the RangeCheck LUT.
    pub layout: RangeCheckLayout<N>,
    /// Atomic counters tracking the number of times each LUT entry is accessed.
    pub multiplicities: AtomicMultiplicityColumn,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeCheckLayout<const N: usize> {
    #[serde_as(as = "[_; N]")]
    pub ranges: [u32; N],
    pub log_size: u32,
}

impl<const N: usize> RangeCheckLookup<N> {
    pub fn new(layout: &RangeCheckLayout<N>) -> Self {
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            multiplicities,
        }
    }

    pub fn add_multiplicities_to_table(&self, table: &mut RangeCheckLookupTraceTable) {
        for mult in &self.multiplicities.data {
            table.add_row(RangeCheckLookupTraceTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}
