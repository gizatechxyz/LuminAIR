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
            simd::{
                column::BaseColumn,
                m31::{PackedM31, N_LANES},
                SimdBackend,
            },
            Column,
        },
        fields::m31::{BaseField, M31},
        poly::{
            circle::{CanonicCoset, CircleEvaluation},
            BitReversedOrder,
        },
    },
};
use typetag;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Range(pub Fixed, pub Fixed);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LookupLayout {
    pub ranges: Vec<Range>,
    pub log_size: u32,
}

impl LookupLayout {
    pub fn new(ranges: Vec<Range>) -> Self {
        let log_size = calculate_log_size(value_count(&ranges) as usize);
        Self { ranges, log_size }
    }

    /// Finds the index of a value in the LUT.
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

    /// Find which range contains the target value.
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

/// Counts the exact number of **distinct, non‑zero** integer inputs that
/// will populate this lookup column **before** the column is
/// padded to the next power‑of‑two.
fn value_count(ranges: &Vec<Range>) -> u32 {
    ranges.iter().map(|r| (r.1 .0 - r.0 .0 + 1) as u32).sum()
}

#[typetag::serde]
pub trait PreProcessedColumn: Any {
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
    fn gen_column(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
    fn clone_box(&self) -> Box<dyn PreProcessedColumn>;
    fn as_any(&self) -> &dyn Any;
}

/// A collection of preprocessed columns, whose values are publicly acknowledged.
pub struct PreProcessedTrace {
    pub(crate) columns: Vec<Box<dyn PreProcessedColumn>>,
}

impl PreProcessedTrace {
    pub fn new(mut columns: Vec<Box<dyn PreProcessedColumn>>) -> Self {
        columns.sort_by_key(|c| Reverse(c.log_size()));
        Self { columns }
    }

    pub fn log_sizes(&self) -> Vec<u32> {
        self.columns.iter().map(|c| c.log_size()).collect()
    }

    pub fn ids(&self) -> Vec<PreProcessedColumnId> {
        self.columns.iter().map(|c| c.id()).collect()
    }

    pub fn gen_trace(&self) -> TraceEval {
        self.columns.iter().map(|c| c.gen_column()).collect()
    }

    pub fn columns_of<T: Any>(&self) -> Vec<&T> {
        self.columns
            .iter()
            .filter_map(|c| c.as_any().downcast_ref::<T>())
            .collect()
    }
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinPreProcessed {
    pub layout: LookupLayout,
    pub col_index: usize,

    #[serde(skip)]
    // lazy cache
    pub eval: OnceCell<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>,
}

impl SinPreProcessed {
    pub fn new(layout: LookupLayout, col_index: usize) -> Self {
        assert!(col_index < 2, "Sin LUT must have 2 columns");

        Self {
            layout,
            col_index,
            eval: OnceCell::new(),
        }
    }

    /// Lazily build (or fetch) the full column evaluation.
    pub fn evaluation(&self) -> &CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        self.eval.get_or_init(|| self.gen_column())
    }

    /// Given a vector of row index, computes the packed M31 values for that row
    pub fn packed_at(&self, vec_row: usize, values_from_range: &[i64]) -> PackedM31 {
        // Calculate starting index for this vector row
        let start_idx = vec_row * N_LANES;

        // Create array of M31 values
        let values = std::array::from_fn(|i| {
            let idx = start_idx + i;
            if idx < values_from_range.len() {
                // Get the actual input value
                let input_val = values_from_range[idx];

                match self.col_index {
                    0 => Fixed(input_val).to_m31(), // Input column
                    1 => {
                        // Compute sine
                        Fixed::from_f64(Fixed(input_val).to_f64().sin()).to_m31()
                    }
                    _ => unreachable!(),
                }
            } else {
                // Padding with zeros
                M31::from_u32_unchecked(0)
            }
        });

        PackedM31::from(values)
    }
}

#[typetag::serde]
impl PreProcessedColumn for SinPreProcessed {
    fn log_size(&self) -> u32 {
        self.layout.log_size
    }

    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("sin_lut_{}", self.col_index),
        }
    }

    fn clone_box(&self) -> Box<dyn PreProcessedColumn> {
        Box::new(self.clone())
    }

    /// Generate the entire column
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

        // Sort and deduplicate (mimicking what SinLUT does)
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
