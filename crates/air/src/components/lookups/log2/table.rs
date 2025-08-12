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
pub struct Log2LookupTraceTable {
    pub table: Vec<Log2LookupTraceTableRow>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Log2LookupTraceTableRow {
    pub multiplicity: M31,
}

impl Log2LookupTraceTableRow {
    pub(crate) fn padding() -> Self {
        Self {
            multiplicity: M31::zero(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PackedLog2LookupTraceTableRow {
    pub multiplicity: PackedM31,
}

impl Pack for Log2LookupTraceTableRow {
    type SimdType = PackedLog2LookupTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedLog2LookupTraceTableRow {
            multiplicity: PackedM31::from_array(std::array::from_fn(|i| inputs[i].multiplicity)),
        }
    }
}

impl Unpack for PackedLog2LookupTraceTableRow {
    type CpuType = Log2LookupTraceTableRow;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        let multiplicities = self.multiplicity.to_array();

        std::array::from_fn(|i| Log2LookupTraceTableRow {
            multiplicity: multiplicities[i],
        })
    }
}

impl Log2LookupTraceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: Log2LookupTraceTableRow) {
        self.table.push(row);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Log2LookupColumn {
    Multiplicity,
}

impl Log2LookupColumn {
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

impl TraceColumn for Log2LookupColumn {
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}