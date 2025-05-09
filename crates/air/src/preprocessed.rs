use std::{any::Any, cmp::Reverse};

use crate::components::{lookups::Lookups, TraceEval};
use stwo_prover::{
    constraint_framework::preprocessed_columns::PreProcessedColumnId,
    core::{
        backend::simd::SimdBackend,
        fields::m31::BaseField,
        poly::{circle::CircleEvaluation, BitReversedOrder},
    },
};
use typetag;

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

pub fn lookups_to_preprocessed_column(_lookups: &Lookups) -> Vec<Box<dyn PreProcessedColumn>> {
    let mut lut_cols: Vec<Box<dyn PreProcessedColumn>> = Vec::new();
    lut_cols
}
