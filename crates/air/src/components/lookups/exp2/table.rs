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

/// Represents the raw trace data for the Exp2 Lookup Table (LUT) component.
///
/// This table primarily stores the multiplicity (count of accesses) for each entry
/// in the preprocessed Exp2 LUT. It's populated from `Exp2Lookup::multiplicities`.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Exp2LookupTraceTable {
    /// Vector of rows, where each row corresponds to an entry in the Exp2 LUT.
    pub table: Vec<Exp2LookupTraceTableRow>,
}

/// Represents a single row in the `Exp2LookupTraceTable`.
/// Corresponds to one entry in the preprocessed Exp2 LUT.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct Exp2LookupTraceTableRow {
    /// The number of times this specific LUT entry (a pair of `(input, output)` values)
    /// was accessed by Exp2 operations in the main computation trace.
    pub multiplicity: M31,
}

impl Exp2LookupTraceTableRow {
    /// Creates a default padding row for the Exp2Lookup trace (multiplicity 0).
    pub(crate) fn padding() -> Self {
        Self {
            multiplicity: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `Exp2LookupTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedExp2LookupTraceTableRow {
    /// Packed multiplicity values.
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
    /// Creates a new, empty `Exp2LookupTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row (multiplicity count) to the trace table.
    pub fn add_row(&mut self, row: Exp2LookupTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the Exp2Lookup AIR component's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Exp2LookupColumn {
    /// Column storing the multiplicity of access for each LUT entry.
    Multiplicity,
}

impl Exp2LookupColumn {
    /// Returns the 0-based index for this column within the Exp2Lookup trace segment.
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

/// Implements the `TraceColumn` trait for `Exp2LookupColumn`.
impl TraceColumn for Exp2LookupColumn {
    /// Specifies the number of columns used by the Exp2Lookup component.
    /// Returns `(N_TRACE_COLUMNS, 1)`, indicating main trace columns for multiplicities
    /// and 1 interaction trace column for the LogUp argument that connects these
    /// multiplicities to the preprocessed LUT values.
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}
