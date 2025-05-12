use std::collections::BTreeSet;

use numerair::Fixed;
use serde::{Deserialize, Serialize};

use crate::{preprocessed::LookupLayout, utils::AtomicMultiplicityColumn};

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
