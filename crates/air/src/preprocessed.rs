use std::{any::Any, cmp::Reverse};

use crate::{components::TraceEval, utils::calculate_log_size};
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Range(pub Fixed, pub Fixed);

#[typetag::serde]
pub trait PreProcessedColumn: Any {
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
    fn gen_column(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
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
        self.columns.iter().map(|c| c.gen_column()).collect()
    }
}

// ================== RECIP ==================
#[derive(Serialize, Deserialize, Clone)]
pub struct RecipLUT {
    pub ranges: Vec<Range>,
    pub col_index: usize,
}

impl RecipLUT {
    pub const fn new(ranges: Vec<Range>, col_index: usize) -> Self {
        assert!(col_index < 2, "Recip LUT must have 2 columns");
        Self { ranges, col_index }
    }

    /// Counts the exact number of **distinct, non‑zero** integer inputs that
    /// will populate this reciprocal lookup column **before** the column is
    /// padded to the next power‑of‑two.
    fn value_count(&self) -> usize {
        self.ranges
            .iter()
            .map(|r| {
                (r.1 .0 - r.0 .0 + 1) as usize - if r.0 .0 <= 0 && 0 <= r.1 .0 { 1 } else { 0 }
            })
            .sum()
    }
}

#[typetag::serde]
impl PreProcessedColumn for RecipLUT {
    fn log_size(&self) -> u32 {
        calculate_log_size(self.value_count())
    }

    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("recip_lut_{}", self.col_index),
        }
    }

    fn clone_box(&self) -> Box<dyn PreProcessedColumn> {
        Box::new(self.clone())
    }

    fn gen_column(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        // Enumerate all admissible x that belong to *any* range and x ≠ 0
        let mut values: Vec<i64> = self
            .ranges
            .iter()
            .flat_map(|r| (r.0 .0..=r.1 .0))
            .filter(|&x| x != 0)
            .collect();
        values.sort_unstable();
        values.dedup();

        let log_size = calculate_log_size(values.len());
        let trace_size = 1 << log_size;
        let mut col = BaseColumn::zeros(trace_size);

        for (i, v) in values.into_iter().enumerate() {
            match self.col_index {
                0 => col.set(i, Fixed(v).to_m31()),
                1 => col.set(i, Fixed::from_f64(Fixed::to_f64(Fixed(v)).recip()).to_m31()),
                _ => unreachable!(),
            }
        }

        let domain = CanonicCoset::new(log_size).circle_domain();
        CircleEvaluation::new(domain, col)
    }
}
