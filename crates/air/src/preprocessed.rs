use std::{any::Any, cmp::Reverse, simd::Simd};

use crate::{components::TraceEval, utils::calculate_log_size};
use numerair::Fixed;
use serde::{Deserialize, Serialize};
use stwo_prover::{
    constraint_framework::preprocessed_columns::PreProcessedColumnId,
    core::{
        backend::simd::{
            column::BaseColumn,
            m31::{PackedM31, LOG_N_LANES, N_LANES},
            SimdBackend,
        },
        fields::m31::{BaseField, M31},
        poly::{
            circle::{CanonicCoset, CircleEvaluation},
            BitReversedOrder,
        },
    },
};
use typetag;

pub const SIMD_ENUMERATION_0: Simd<u32, N_LANES> =
    Simd::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Range(pub Fixed, pub Fixed);

#[typetag::serde]
pub trait PreProcessedColumn: Any {
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
    fn clone_box(&self) -> Box<dyn PreProcessedColumn>;
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
        self.columns.iter().map(|c| c.gen_column_simd()).collect()
    }
}

// ================== SIN ==================

#[derive(Serialize, Deserialize, Clone)]
pub struct SinLUT {
    pub ranges: Vec<Range>,
    pub col_index: usize,
}

impl SinLUT {
    pub const fn new(ranges: Vec<Range>, col_index: usize) -> Self {
        assert!(col_index < 2, "Sin LUT must have 2 columns");
        Self { ranges, col_index }
    }

    /// Counts the exact number of **distinct, non‑zero** integer inputs that
    /// will populate this lookup column **before** the column is
    /// padded to the next power‑of‑two.
    fn value_count(&self) -> usize {
        self.ranges
            .iter()
            .map(|r| (r.1 .0 - r.0 .0 + 1) as usize)
            .sum()
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
impl PreProcessedColumn for SinLUT {
    fn log_size(&self) -> u32 {
        calculate_log_size(self.value_count())
    }

    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("sin_lut_{}", self.col_index),
        }
    }

    fn clone_box(&self) -> Box<dyn PreProcessedColumn> {
        Box::new(self.clone())
    }

    /// Generate the entire column using SIMD
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        let log_size = self.log_size();
        let domain = CanonicCoset::new(log_size).circle_domain();

        // Enumerate all values from ranges
        let mut all_values: Vec<i64> = self.ranges.iter().flat_map(|r| (r.0 .0..=r.1 .0)).collect();
        all_values.sort_unstable();
        all_values.dedup();

        // Generate column using packed_at
        let column = BaseColumn::from_simd(
            (0..(1 << (log_size - LOG_N_LANES)))
                .map(|i| self.packed_at(i, &all_values))
                .collect(),
        );

        CircleEvaluation::new(domain, column)
    }
}
