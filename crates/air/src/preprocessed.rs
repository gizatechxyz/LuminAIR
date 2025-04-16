use numerair::Fixed;
use stwo_prover::{
    constraint_framework::preprocessed_columns::PreProcessedColumnId,
    core::{
        backend::{simd::column::BaseColumn, Column},
        poly::circle::{CanonicCoset, CircleEvaluation},
    },
};

use crate::{components::TraceEval, utils::calculate_log_size};

pub type Range = (Fixed, Fixed);

pub trait PreProcessedColumn {
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
    fn gen_column(&self) -> TraceEval;
}

// ================== RECIP ==================
pub struct RecipLUT {
    range: Range,
    col_index: usize,
}

impl RecipLUT {
    pub const fn new(range: Range, col_index: usize) -> Self {
        assert!(col_index < 2, "Recip LUT must have 2 columns");
        Self { range, col_index }
    }
}

impl PreProcessedColumn for RecipLUT {
    fn log_size(&self) -> u32 {
        calculate_log_size((self.range.1 .0 - self.range.0 .0 + 1) as usize)
    }

    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId {
            id: format!("recip_lut_{}", self.col_index),
        }
    }

    fn gen_column(&self) -> TraceEval {
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
        vec![CircleEvaluation::new(domain, col)]
    }
}
