use std::{any::Any, cell::OnceCell, cmp::Reverse};

use crate::{
    components::{
        //lookups::Lookups,
        lookups::Lookups,
        TraceEval,
    },
    utils::calculate_log_size,
};
use numerair::Fixed;
use serde::{Deserialize, Serialize};
use stwo_prover::{
    constraint_framework::preprocessed_columns::PreProcessedColumnId,
    core::{
        backend::{
            simd::{column::BaseColumn, SimdBackend},
            Column,
        },
        fields::m31::BaseField,
        poly::{
            circle::{CanonicCoset, CircleEvaluation},
            BitReversedOrder,
        },
    },
};
use typetag;

/// Represents a closed range [min, max] using fixed-point numbers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Range(pub Fixed, pub Fixed);

/// Defines the layout of a lookup table (LUT) based on value ranges.
///
/// Stores the value ranges covered by the LUT and calculates the necessary
/// log2 size for the table, padded to a power of two.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LookupLayout {
    /// The vector of disjoint value ranges covered by this LUT.
    pub ranges: Vec<Range>,
    /// The log2 size of the LUT column (padded to a power of two).
    pub log_size: u32,
}

impl LookupLayout {
    /// Creates a new `LookupLayout` from a vector of potentially overlapping ranges.
    ///
    /// Calculates the total number of unique integer values across the ranges
    /// and determines the minimum power-of-two `log_size` required.
    pub fn new(ranges: Vec<Range>) -> Self {
        let log_size = calculate_log_size(value_count(&ranges) as usize);
        Self { ranges, log_size }
    }

    /// Finds the row index within the conceptual LUT for a given target value.
    ///
    /// The LUT is conceptually ordered by the ranges. This function calculates the index
    /// based on the cumulative count of values in preceding ranges plus the offset within
    /// the containing range.
    /// Returns `None` if the `target` value is not covered by any range in the layout.
    pub fn find_index(&self, target: i64) -> Option<usize> {
        // Binary search to find the range containing the target
        match self.find_containing_range(target) {
            Some((range_idx, range)) => {
                // Calculate the cumulative count of values before this range
                let mut cumulative_count = 0;
                for i in 0..range_idx {
                    let r = &self.ranges[i];
                    cumulative_count += (r.1 .0 - r.0 .0 + 1) as usize;
                }

                // Add the offset within the found range
                let offset = (target - range.0 .0) as usize;
                Some(cumulative_count + offset)
            }
            None => None,
        }
    }

    /// Finds the specific `Range` within the layout that contains the `target` value.
    /// Returns the index of the range and a reference to the range itself.
    fn find_containing_range(&self, target: i64) -> Option<(usize, &Range)> {
        // Early check for empty ranges
        if self.ranges.is_empty() {
            return None;
        }

        // Binary search to find the correct range
        let mut left = 0;
        let mut right = self.ranges.len() - 1;

        while left <= right {
            let mid = left + (right - left) / 2;
            let range = &self.ranges[mid];

            // Check if target is in this range
            if target >= range.0 .0 && target <= range.1 .0 {
                return Some((mid, range));
            }

            // Adjust search boundaries
            if target < range.0 .0 {
                // Target is before this range
                if mid == 0 {
                    break; // Can't go left further
                }
                right = mid - 1;
            } else {
                // Target is after this range
                if mid == self.ranges.len() - 1 {
                    break; // Can't go right further
                }
                left = mid + 1;
            }
        }

        None
    }
}

/// Counts the total number of unique integer values covered by a set of ranges.
/// Used to determine the minimum required size of a lookup table before padding.
fn value_count(ranges: &Vec<Range>) -> u32 {
    ranges.iter().map(|r| (r.1 .0 - r.0 .0 + 1) as u32).sum()
}

/// Trait representing a preprocessed column in the trace.
///
/// Preprocessed columns contain publicly known data, like lookup tables, that
/// are generated before the main proof computation. This trait allows different
/// types of preprocessed columns (e.g., for different LUTs) to be handled generically.
#[typetag::serde]
pub trait PreProcessedColumn: Any {
    /// Returns the log2 size of this column (padded to a power of two).
    fn log_size(&self) -> u32;
    /// Returns a unique identifier for this preprocessed column type.
    fn id(&self) -> PreProcessedColumnId;
    /// Generates the actual column data as a `CircleEvaluation`.
    fn gen_column(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
    /// Creates a boxed clone of this column.
    fn clone_box(&self) -> Box<dyn PreProcessedColumn>;
    /// Allows downcasting to the concrete type.
    fn as_any(&self) -> &dyn Any;
}

/// Container holding all preprocessed columns for a STARK proof.
///
/// This structure groups different types of `PreProcessedColumn` instances together.
/// The columns are typically sorted by log_size for efficiency in the PCS.
pub struct PreProcessedTrace {
    /// Vector of boxed preprocessed column objects.
    pub(crate) columns: Vec<Box<dyn PreProcessedColumn>>,
}

impl PreProcessedTrace {
    /// Creates a new `PreProcessedTrace` from a vector of columns.
    /// Sorts the columns internally by descending log_size.
    pub fn new(mut columns: Vec<Box<dyn PreProcessedColumn>>) -> Self {
        columns.sort_by_key(|c| Reverse(c.log_size()));
        Self { columns }
    }

    /// Returns a vector containing the log_size of each column.
    pub fn log_sizes(&self) -> Vec<u32> {
        self.columns.iter().map(|c| c.log_size()).collect()
    }

    /// Returns a vector containing the ID of each column.
    pub fn ids(&self) -> Vec<PreProcessedColumnId> {
        self.columns.iter().map(|c| c.id()).collect()
    }

    /// Generates the `TraceEval` (vector of `CircleEvaluation`s) for all columns.
    pub fn gen_trace(&self) -> TraceEval {
        self.columns.iter().map(|c| c.gen_column()).collect()
    }

    /// Filters the columns and returns references to those of a specific concrete type `T`.
    pub fn columns_of<T: Any>(&self) -> Vec<&T> {
        self.columns
            .iter()
            .filter_map(|c| c.as_any().downcast_ref::<T>())
            .collect()
    }
}

/// Converts configured `Lookups` into a vector of corresponding `PreProcessedColumn` boxes.
pub fn lookups_to_preprocessed_column(lookups: &Lookups) -> Vec<Box<dyn PreProcessedColumn>> {
    let mut lut_cols: Vec<Box<dyn PreProcessedColumn>> = Vec::new();
    if let Some(sin_lookup) = &lookups.sin {
        let col_0 = SinPreProcessed::new(sin_lookup.layout.clone(), 0);
        let col_1 = SinPreProcessed::new(sin_lookup.layout.clone(), 1);
        lut_cols.push(Box::new(col_0));
        lut_cols.push(Box::new(col_1));
    }
    lut_cols
}

// ================== SIN ==================

/// Concrete implementation of `PreProcessedColumn` for the Sine Lookup Table (LUT).
///
/// Stores the layout (`LookupLayout`) and generates two columns:
/// - Column 0: Input values `x` (as `Fixed` point `M31` elements).
/// - Column 1: Output values `sin(x)` (as `Fixed` point `M31` elements).
/// Uses `OnceCell` to cache the generated column evaluations.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinPreProcessed {
    /// The layout defining the ranges and size of the LUT.
    pub layout: LookupLayout,
    /// The index of the column (0 for input `x`, 1 for output `sin(x)`).
    pub col_index: usize,

    #[serde(skip)]
    /// Lazy cache for the generated `CircleEvaluation` of this column.
    pub eval: OnceCell<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>,
}

impl SinPreProcessed {
    /// Creates a new `SinPreProcessed` column instance.
    /// Panics if `col_index` is not 0 or 1.
    pub fn new(layout: LookupLayout, col_index: usize) -> Self {
        assert!(col_index < 2, "Sin LUT must have 2 columns");

        Self {
            layout,
            col_index,
            eval: OnceCell::new(),
        }
    }

    /// Returns a reference to the generated `CircleEvaluation` for this column.
    /// Generates the column and caches it on the first call.
    pub fn evaluation(&self) -> &CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        self.eval.get_or_init(|| self.gen_column())
    }
}

#[typetag::serde]
impl PreProcessedColumn for SinPreProcessed {
    /// Returns the log_size defined by the layout.
    fn log_size(&self) -> u32 {
        self.layout.log_size
    }

    /// Returns the ID string `sin_lut_0` or `sin_lut_1`.
    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("sin_lut_{}", self.col_index),
        }
    }

    /// Creates a boxed clone of this `SinPreProcessed` instance.
    fn clone_box(&self) -> Box<dyn PreProcessedColumn> {
        Box::new(self.clone())
    }

    /// Generates the `CircleEvaluation` for this specific column (input or output).
    ///
    /// It iterates through all unique integer values covered by the `layout` ranges,
    /// calculates the corresponding `Fixed` point value (`x` or `sin(x)`),
    /// converts it to `BaseField` (`M31`), and places it in the evaluation column.
    /// The column is padded with zeros to the power-of-two size defined by `log_size`.
    fn gen_column(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        let log_size = self.log_size();
        let domain = CanonicCoset::new(log_size).circle_domain();

        // Enumerate all values from ranges
        let mut all_values: Vec<i64> = self
            .layout
            .ranges
            .iter()
            .flat_map(|r| (r.0 .0..=r.1 .0))
            .collect();
        all_values.sort_unstable();
        all_values.dedup();

        let trace_size = 1 << log_size;
        let mut column = BaseColumn::zeros(trace_size);

        for (i, value) in all_values.iter().enumerate() {
            match self.col_index {
                0 => column.set(i, Fixed(*value).to_m31()),
                1 => column.set(i, Fixed::from_f64(Fixed(*value).to_f64().sin()).to_m31()),
                _ => unreachable!(),
            }
        }

        CircleEvaluation::new(domain, column)
    }

    /// Returns this instance as `&dyn Any` for downcasting.
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod range_tests {

    use super::*;

    fn range(min: i64, max: i64) -> Range {
        Range(Fixed(min), Fixed(max))
    }

    fn calculate_expected_indices(ranges: &[Range]) -> Vec<(i64, Option<usize>)> {
        // Get all values from ranges
        let mut all_values: Vec<i64> = ranges.iter().flat_map(|r| (r.0 .0..=r.1 .0)).collect();

        // Sort and deduplicate (mimicking what SinPreProcessed does)
        all_values.sort_unstable();
        all_values.dedup();

        // Map each value to its index
        all_values
            .iter()
            .enumerate()
            .map(|(idx, &val)| (val, Some(idx)))
            .collect()
    }

    #[test]
    fn test_find_index() {
        // Test with multiple ranges having gaps between them
        let ranges = vec![
            range(-100, -50), // 51 values
            range(0, 10),     // 11 values
            range(200, 210),  // 11 values
        ];

        let layout = LookupLayout::new(ranges.clone());

        // Compute expected indices for validation
        let expected_indices = calculate_expected_indices(&ranges);

        // Test some specific values from different ranges
        let test_values = vec![
            -100, -75, -50, // First range
            0, 5, 10, // Second range
            200, 205, 210, // Third range
        ];

        for &val in &test_values {
            let expected = expected_indices
                .iter()
                .find(|&&(v, _)| v == val)
                .map(|&(_, idx)| idx)
                .unwrap();
            assert_eq!(
                layout.find_index(val),
                expected,
                "Value {} should be at index {:?}",
                val,
                expected
            );
        }

        // Test values in the gaps
        assert_eq!(
            layout.find_index(-49),
            None,
            "Value -49 should not be in the LUT"
        );
        assert_eq!(
            layout.find_index(11),
            None,
            "Value 11 should not be in the LUT"
        );
        assert_eq!(
            layout.find_index(199),
            None,
            "Value 199 should not be in the LUT"
        );
    }
}
