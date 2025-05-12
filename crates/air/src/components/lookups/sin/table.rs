use num_traits::Zero;
use serde::{Deserialize, Serialize};
use stwo_prover::core::{
    backend::simd::{
        conversion::{Pack, Unpack},
        m31::{PackedM31, N_LANES},
    },
    fields::m31::M31,
};

use crate::components::TraceColumn;

use super::witness::N_TRACE_COLUMNS;

/// Represents the trace for the SinLookup component, containing the required registers for its
/// constraints.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinLookupTraceTable {
    /// A vector of [`SinLookupTraceTableRow`] representing the table rows.
    pub table: Vec<SinLookupTraceTableRow>,
}

/// Represents a single row of the [`SinLookupTraceTable`]
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
    /// Creates a new, empty [`SinLookupTraceTable`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the Sin Lookup table.
    pub fn add_row(&mut self, row: SinLookupTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum representing the column indices in the Sin Lookup trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinLookupColumn {
    Multiplicity,
}

impl SinLookupColumn {
    /// Returns the index of the column in the SinLookup trace.
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

impl TraceColumn for SinLookupColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}
