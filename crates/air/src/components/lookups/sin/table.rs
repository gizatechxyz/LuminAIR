use num_traits::Zero;
use serde::{Deserialize, Serialize};
use stwo::{
    core::fields::m31::M31,
    prover::backend::simd::{
        conversion::{Pack, Unpack},
        m31::{PackedM31, N_LANES},
    },
};

use crate::components::TraceColumn;

use super::witness::N_TRACE_COLUMNS;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinLookupTraceTable {
    pub table: Vec<SinLookupTraceTableRow>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinLookupTraceTableRow {
    pub multiplicity: M31,
}

impl SinLookupTraceTableRow {
    pub(crate) fn padding() -> Self {
        Self {
            multiplicity: M31::zero(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PackedSinLookupTraceTableRow {
    pub multiplicity: PackedM31,
}

impl Pack for SinLookupTraceTableRow {
    type SimdType = PackedSinLookupTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedSinLookupTraceTableRow {
            multiplicity: PackedM31::from_array(std::array::from_fn(|i| inputs[i].multiplicity)),
        }
    }
}

impl Unpack for PackedSinLookupTraceTableRow {
    type CpuType = SinLookupTraceTableRow;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        let multiplicities = self.multiplicity.to_array();

        std::array::from_fn(|i| SinLookupTraceTableRow {
            multiplicity: multiplicities[i],
        })
    }
}

impl SinLookupTraceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: SinLookupTraceTableRow) {
        self.table.push(row);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinLookupColumn {
    Multiplicity,
}

impl SinLookupColumn {
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

impl TraceColumn for SinLookupColumn {
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}
