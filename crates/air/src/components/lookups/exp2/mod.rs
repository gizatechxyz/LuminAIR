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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Exp2Lookup {
    pub layout: LookupLayout,
    pub multiplicities: AtomicMultiplicityColumn,
}

impl Exp2Lookup {
    pub fn new(layout: &LookupLayout) -> Self {
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            multiplicities,
        }
    }

    pub fn add_multiplicities_to_table(&self, table: &mut Exp2LookupTraceTable) {
        for mult in &self.multiplicities.data {
            table.add_row(Exp2LookupTraceTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}
