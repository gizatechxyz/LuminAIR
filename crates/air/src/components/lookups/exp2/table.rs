use num_traits::Zero;
use serde::{Deserialize, Serialize};
use stwo::core::{
    backend::simd::{
        conversion::{Pack, Unpack},
        m31::{PackedM31, N_LANES},
    },
    fields::m31::M31,
};

use crate::components::TraceColumn;

use super::witness::N_TRACE_COLUMNS;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Exp2LookupTraceTable {
    pub table: Vec<Exp2LookupTraceTableRow>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Exp2LookupTraceTableRow {
    pub multiplicity: M31,
}

impl Exp2LookupTraceTableRow {
    pub(crate) fn padding() -> Self {
        Self {
            multiplicity: M31::zero(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PackedExp2LookupTraceTableRow {
    pub multiplicity: PackedM31,
}

impl Pack for Exp2LookupTraceTableRow {
    type SimdType = PackedExp2LookupTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedExp2LookupTraceTableRow {
            multiplicity: PackedM31::from_array(std::array::from_fn(|i| inputs[i].multiplicity)),
        }
    }
}

impl Unpack for PackedExp2LookupTraceTableRow {
    type CpuType = Exp2LookupTraceTableRow;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        let multiplicities = self.multiplicity.to_array();

        std::array::from_fn(|i| Exp2LookupTraceTableRow {
            multiplicity: multiplicities[i],
        })
    }
}

impl Exp2LookupTraceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: Exp2LookupTraceTableRow) {
        self.table.push(row);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Exp2LookupColumn {
    Multiplicity,
}

impl Exp2LookupColumn {
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

impl TraceColumn for Exp2LookupColumn {
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}
