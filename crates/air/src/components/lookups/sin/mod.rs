use std::{collections::BTreeSet, sync::atomic::Ordering};

use numerair::Fixed;
use serde::{Deserialize, Serialize};
use stwo_prover::{core::fields::m31::BaseField, relation};
use table::{SinLookupTraceTable, SinLookupTraceTableRow};

use crate::{preprocessed::LookupLayout, utils::AtomicMultiplicityColumn, DEFAULT_FP_SCALE};

pub mod component;
pub mod table;
pub mod witness;

// Interaction elements specifically for the Sine Lookup Table argument.
// Drawn from the channel, used to combine `(input, output)` pairs from the Sin LUT.
relation!(SinLookupElements, 2);

/// Configuration and data for the Sine Lookup Table (LUT).
///
/// Holds the `LookupLayout` (defining value ranges and size), the actual LUT data
/// (`SinLookupData`), and an `AtomicMultiplicityColumn` to track accesses to LUT entries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinLookup {
    /// The layout defining the structure and value ranges of the Sine LUT.
    pub layout: LookupLayout,
    /// Atomic counters tracking the number of times each LUT entry is accessed.
    pub multiplicities: AtomicMultiplicityColumn,
}

impl SinLookup {
    /// Creates a new `SinLookup` instance based on the provided `LookupLayout`.
    ///
    /// Initializes the `SinLookupData` (LUT values) from the layout and creates
    /// an `AtomicMultiplicityColumn` of the appropriate size (padded to power of two).
    pub fn new(layout: &LookupLayout) -> Self {
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            multiplicities,
        }
    }

    /// Populates a `SinLookupTraceTable` with the final multiplicity counts.
    ///
    /// This table is used by the `SinLookupComponent` to generate the trace columns
    /// for proving the lookup argument (i.e., that the sum of multiplicities matches accesses).
    pub fn add_multiplicities_to_table(&self, table: &mut SinLookupTraceTable) {
        for mult in &self.multiplicities.data {
            table.add_row(SinLookupTraceTableRow {
                multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
            });
        }
    }
}

/// Stores the actual column data for the Sine Lookup Table (input `x` and output `sin(x)`).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinLookupData {
    /// Column of input values (`x`) to the sine function.
    pub col_0: Vec<Fixed<DEFAULT_FP_SCALE>>,
    /// Column of output values (`sin(x)`).
    pub col_1: Vec<Fixed<DEFAULT_FP_SCALE>>,
}

impl SinLookupData {
    /// Constructs the Sine LUT data (input and output columns) based on a `LookupLayout`.
    ///
    /// It iterates through all unique integer values covered by the layout's ranges,
    /// calculates `x` (as `Fixed`) and `sin(x)` (as `Fixed`), and stores them.
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
