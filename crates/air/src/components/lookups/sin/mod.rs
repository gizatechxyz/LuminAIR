use std::{collections::BTreeSet, sync::atomic::Ordering};

use numerair::Fixed;
use serde::{Deserialize, Serialize};
use stwo_prover::{core::fields::m31::BaseField, relation};
use table::{SinLookupTable, SinLookupTableRow};

use crate::{preprocessed::LookupLayout, utils::AtomicMultiplicityColumn};

pub mod component;
pub mod table;
pub mod witness;

// Defines the relation for the LUT elements.
// It allows to constrain LUTs.
relation!(SinLookupElements, 2);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinLookup {
    pub layout: LookupLayout,
    pub data: SinLookupData,
    pub multiplicities: AtomicMultiplicityColumn,
}

impl SinLookup {
    pub fn new(layout: &LookupLayout) -> Self {
        let data = SinLookupData::new(&layout);
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            data,
            multiplicities,
        }
    }

    pub fn add_multiplicities_to_table(&self, table: &mut SinLookupTable) {
        for mult in &self.multiplicities.data {
            table.add_row(SinLookupTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinLookupData {
    pub col_0: Vec<Fixed>,
    pub col_1: Vec<Fixed>,
}

impl SinLookupData {
    /// Build the two-column sine lookup from a layout.
    pub fn new(layout: &LookupLayout) -> Self {
        let mut uniq = BTreeSet::<i64>::new();
        for range in &layout.ranges {
            uniq.extend(range.0 .0..=range.1 .0);
        }

        let mut col_0 = Vec::with_capacity(uniq.len());
        let mut col_1 = Vec::with_capacity(uniq.len());

        for &raw in &uniq {
            let x = Fixed(raw);
            col_0.push(x);
            col_1.push(Fixed::from_f64(x.to_f64().sin()));
        }

        Self { col_0, col_1 }
    }
}
