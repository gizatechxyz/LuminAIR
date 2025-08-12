use num_traits::Zero;
use serde::{Deserialize, Serialize};
use stwo::{
    core::fields::m31::M31,
    prover::backend::simd::{
        conversion::{Pack, Unpack},
        m31::{PackedM31, N_LANES},
    },
};

use crate::components::{lookups::range_check::witness::N_TRACE_COLUMNS, TraceColumn};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct RangeCheckLookupTraceTable {
    pub table: Vec<RangeCheckLookupTraceTableRow>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RangeCheckLookupTraceTableRow {
    pub multiplicity: M31,
}

impl RangeCheckLookupTraceTableRow {
    pub(crate) fn padding() -> Self {
        Self {
            multiplicity: M31::zero(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PackedRangeCheckLookupTraceTableRow {
    pub multiplicity: PackedM31,
}

impl Pack for RangeCheckLookupTraceTableRow {
    type SimdType = PackedRangeCheckLookupTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedRangeCheckLookupTraceTableRow {
            multiplicity: PackedM31::from_array(std::array::from_fn(|i| inputs[i].multiplicity)),
        }
    }
}

impl Unpack for PackedRangeCheckLookupTraceTableRow {
    type CpuType = RangeCheckLookupTraceTableRow;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        let multiplicities = self.multiplicity.to_array();

        std::array::from_fn(|i| RangeCheckLookupTraceTableRow {
            multiplicity: multiplicities[i],
        })
    }
}

impl RangeCheckLookupTraceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: RangeCheckLookupTraceTableRow) {
        self.table.push(row);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RangeCheckLookupColumn {
    Multiplicity,
}

impl RangeCheckLookupColumn {
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

impl TraceColumn for RangeCheckLookupColumn {
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}
