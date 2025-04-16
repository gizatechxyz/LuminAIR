use itertools::{chain, Itertools};
use num_traits::Zero;
use numerair::Fixed;
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

use crate::{components::TraceEval, utils::calculate_log_size};

pub type Range = (Fixed, Fixed);

pub trait PreProcessedColumn {
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
    fn gen_column(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
}

/// A collection of preprocessed columns, whose values are publicly acknowledged.
pub struct PreProcessedTrace {
    columns: Vec<Box<dyn PreProcessedColumn>>,
}

impl PreProcessedTrace {
    pub fn new() -> Self {
        let recip_cols = gen_recip_columns();

        let columns = chain!(recip_cols)
            .sorted_by_key(|column| std::cmp::Reverse(column.log_size()))
            .collect();

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
pub struct RecipLUT {
    range: Range,
    col_index: usize,
    node_id: usize,
}

impl RecipLUT {
    pub const fn new(range: Range, col_index: usize, node_id: usize) -> Self {
        assert!(col_index < 2, "Recip LUT must have 2 columns");
        Self {
            range,
            col_index,
            node_id,
        }
    }

    pub fn gen_constant_trace(&self) -> TraceEval {
        todo!()
    }
}

impl PreProcessedColumn for RecipLUT {
    fn log_size(&self) -> u32 {
        calculate_log_size((self.range.1 .0 - self.range.0 .0 + 1) as usize)
    }

    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("recip_lut_node_{}_col_{}", self.node_id, self.col_index),
        }
    }

    fn gen_column(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        let log_size = self.log_size();
        let trace_size = 1 << log_size;
        let mut col = BaseColumn::zeros(trace_size);

        for (i, v) in (self.range.0 .0..=self.range.1 .0)
            .filter(|&v| v != 0)
            .enumerate()
        {
            match self.col_index {
                0 => col.set(i, Fixed(v).to_m31()),
                1 => col.set(i, Fixed::from_f64(Fixed::to_f64(Fixed(v)).recip()).to_m31()),
                _ => panic!("Invalid index for RecipTable: {}", self.col_index),
            }
        }

        let domain = CanonicCoset::new(log_size).circle_domain();
        CircleEvaluation::new(domain, col)
    }
}

fn gen_recip_columns() -> Vec<Box<dyn PreProcessedColumn>> {
    // TODO: generate RecipLUT dynamically

    let recip_lut_col_0_node_0 = RecipLUT::new((Fixed::zero(), Fixed::zero()), 0, 0);
    let recip_lut_col_1_node_0 = RecipLUT::new((Fixed::zero(), Fixed::zero()), 1, 0);
    vec![
        Box::new(recip_lut_col_0_node_0),
        Box::new(recip_lut_col_1_node_0),
    ]
}
