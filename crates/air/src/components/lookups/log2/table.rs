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

/// Represents the raw trace data for the Log2 Lookup Table (LUT) component.
///
/// This table primarily stores the multiplicity (count of accesses) for each entry
/// in the preprocessed Log2 LUT. It's populated from `Log2Lookup::multiplicities`.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Log2LookupTraceTable {
    /// Vector of rows, where each row corresponds to an entry in the Log2 LUT.
    pub table: Vec<Log2LookupTraceTableRow>,
}

/// Represents a single row in the `Log2LookupTraceTable`.
/// Corresponds to one entry in the preprocessed Log2 LUT.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Log2LookupTraceTableRow {
    /// The number of times this specific LUT entry (a pair of `(input, output)` values)
    /// was accessed by Log2 operations in the main computation trace.
    pub multiplicity: M31,
}

impl Log2LookupTraceTableRow {
    /// Creates a default padding row for the Log2Lookup trace (multiplicity 0).
    pub(crate) fn padding() -> Self {
        Self {
            multiplicity: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `Log2LookupTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedLog2LookupTraceTableRow {
    /// Packed multiplicity values.
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
    /// Creates a new, empty `Log2LookupTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row (multiplicity count) to the trace table.
    pub fn add_row(&mut self, row: Log2LookupTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the Log2Lookup AIR component's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Log2LookupColumn {
    /// Column storing the multiplicity of access for each LUT entry.
    Multiplicity,
}

impl Log2LookupColumn {
    /// Returns the 0-based index for this column within the Log2Lookup trace segment.
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

/// Implements the `TraceColumn` trait for `Log2LookupColumn`.
impl TraceColumn for Log2LookupColumn {
    /// Specifies the number of columns used by the Log2Lookup component.
    /// Returns `(N_TRACE_COLUMNS, 1)`, indicating main trace columns for multiplicities
    /// and 1 interaction trace column for the LogUp argument that connects these
    /// multiplicities to the preprocessed LUT values.
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}