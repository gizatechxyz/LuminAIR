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

/// Represents the raw trace data for the Sine Lookup Table (LUT) component.
///
/// This table primarily stores the multiplicity (count of accesses) for each entry
/// in the preprocessed Sine LUT. It's populated from `SinLookup::multiplicities`.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinLookupTraceTable {
    /// Vector of rows, where each row corresponds to an entry in the Sine LUT.
    pub table: Vec<SinLookupTraceTableRow>,
}

/// Represents a single row in the `SinLookupTraceTable`.
/// Corresponds to one entry in the preprocessed Sine LUT.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinLookupTraceTableRow {
    /// The number of times this specific LUT entry (a pair of `(input, output)` values)
    /// was accessed by Sin operations in the main computation trace.
    pub multiplicity: M31,
}

impl SinLookupTraceTableRow {
    /// Creates a default padding row for the SinLookup trace (multiplicity 0).
    pub(crate) fn padding() -> Self {
        Self {
            multiplicity: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `SinLookupTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedSinLookupTraceTableRow {
    /// Packed multiplicity values.
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
    /// Creates a new, empty `SinLookupTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row (multiplicity count) to the trace table.
    pub fn add_row(&mut self, row: SinLookupTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the SinLookup AIR component's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinLookupColumn {
    /// Column storing the multiplicity of access for each LUT entry.
    Multiplicity,
}

impl SinLookupColumn {
    /// Returns the 0-based index for this column within the SinLookup trace segment.
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

/// Implements the `TraceColumn` trait for `SinLookupColumn`.
impl TraceColumn for SinLookupColumn {
    /// Specifies the number of columns used by the SinLookup component.
    /// Returns `(N_TRACE_COLUMNS, 1)`, indicating main trace columns for multiplicities
    /// and 1 interaction trace column for the LogUp argument that connects these
    /// multiplicities to the preprocessed LUT values.
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}
