use std::collections::HashMap;

use num_traits::Zero;
use serde::{Deserialize, Serialize};
use stwo_prover::{
    core::{backend::simd::SimdBackend, channel::Channel, fields::m31::M31},
    relation,
};

use crate::{utils::TreeBuilder, DEFAULT_FP_SCALE};

use super::Lookups;

pub mod component;
pub mod table;
pub mod witness;

pub use component::*;
pub use table::*;
pub use witness::*;

/// Defines the layout and scale of the Exp2 Lookup Table.
/// Specifies the range of values covered, number of entries, and scale factors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp2LookupLayout {
    /// Log2 size of the lookup table (number of entries = 2^log_size).
    pub log_size: u32,
    /// Minimum input value covered by the table.
    pub min: f64,
    /// Maximum input value covered by the table.
    pub max: f64,
    /// Scale factor applied to inputs for fixed-point representation.
    pub input_scale: f64,
    /// Scale factor applied to outputs for fixed-point representation.
    pub output_scale: f64,
}

/// Configuration for the Exp2 lookup table and argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp2Lookup {
    /// The layout defining the range and size of the Exp2 LUT.
    pub layout: Exp2LookupLayout,
    /// Tracks the number of accesses to each entry in the Exp2 LUT.
    pub multiplicities: Option<Exp2LookupTraceTable>,
}

impl Exp2Lookup {
    /// Creates a new Exp2Lookup instance with the given layout.
    pub fn new(layout: Exp2LookupLayout) -> Self {
        Self {
            layout,
            multiplicities: Some(Exp2LookupTraceTable::new()),
        }
    }

    /// Returns the preprocessed table data for the Exp2 LUT.
    /// Generates a table of (input, output) pairs where output = 2^input.
    pub fn preprocessed(&self) -> HashMap<String, Vec<M31>> {
        use numerair::Fixed;

        let layout = &self.layout;
        // IMPORTANT: This must match exactly the formula in gen_column and record_lookup
        let num_entries = 1 << layout.log_size;
        let step = (layout.max - layout.min) / (num_entries - 1) as f64;

        let mut lut_0 = Vec::with_capacity(num_entries);
        let mut lut_1 = Vec::with_capacity(num_entries);

        for i in 0..num_entries {
            let x = layout.min + step * i as f64;
            let input_fixed = Fixed::<DEFAULT_FP_SCALE>::from_f64(x);
            let output_fixed = Fixed::<DEFAULT_FP_SCALE>::from_f64(2.0_f64.powf(x));

            lut_0.push(input_fixed.to_m31());
            lut_1.push(output_fixed.to_m31());
        }

        let mut map = HashMap::new();
        map.insert("exp2_lut_0".to_string(), lut_0);
        map.insert("exp2_lut_1".to_string(), lut_1);
        map
    }

    /// Records a lookup access in the Exp2 LUT.
    /// Maps the fixed-point input to the corresponding table entry and increments
    /// the multiplicity for that entry.
    pub fn record_lookup(
        &mut self,
        input_val: numerair::Fixed<DEFAULT_FP_SCALE>,
        _out_val: numerair::Fixed<DEFAULT_FP_SCALE>,
    ) -> (
        numerair::Fixed<DEFAULT_FP_SCALE>,
        numerair::Fixed<DEFAULT_FP_SCALE>,
    ) {
        let layout = &self.layout;
        let x = input_val.to_f64();

        // Make sure x is within the valid range
        let x_clamped = x.max(layout.min).min(layout.max);

        // IMPORTANT: This must match exactly the formula in gen_column and preprocessed
        let num_entries = 1 << layout.log_size;
        let step = (layout.max - layout.min) / (num_entries - 1) as f64;
        let index = ((x_clamped - layout.min) / step).round() as usize;
        // Ensure index doesn't exceed table bounds
        let index = index.min((1 << layout.log_size) - 1);

        // Calculate the exact table values for this index (same as preprocessed table)
        let table_input_f64 = layout.min + step * index as f64;
        let table_output_f64 = 2.0_f64.powf(table_input_f64);
        let table_input = numerair::Fixed::<DEFAULT_FP_SCALE>::from_f64(table_input_f64);
        let table_output = numerair::Fixed::<DEFAULT_FP_SCALE>::from_f64(table_output_f64);

        let multiplicities = self.multiplicities.as_mut().unwrap();
        // Ensure we have enough rows in the table
        if multiplicities.table.len() <= index {
            multiplicities.table.resize(
                index + 1,
                Exp2LookupTraceTableRow {
                    multiplicity: M31::zero(),
                },
            );
        }

        // Increment the multiplicity counter for this table entry
        multiplicities.table[index].multiplicity += M31::from(1);

        // Return the exact table values that should be used for LogUp
        (table_input, table_output)
    }

    /// Writes the Exp2 LUT access trace to the commitment tree.
    /// Processes the recorded multiplicities into the STARK trace format.
    pub fn write_trace(
        &mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> (Exp2LookupClaim, Exp2LookupInteractionClaimGenerator) {
        let multiplicities = self.multiplicities.take().unwrap();
        let claim_generator = Exp2LookupClaimGenerator::new(multiplicities);
        let res = claim_generator
            .write_trace(tree_builder)
            .expect("Failed to write Exp2 lookup trace");

        res
    }
}

// Define Exp2LookupElements using the exact same approach as SinLookupElements
relation!(Exp2LookupElements, 2);