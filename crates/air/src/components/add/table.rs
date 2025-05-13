use num_traits::{One, Zero};
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

/// Represents the raw trace data collected for Add operations.
///
/// This table stores rows generated during the `gen_trace` phase, capturing
/// the inputs, outputs, and necessary metadata for each Add operation instance
/// required to satisfy the AIR constraints.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AddTraceTable {
    /// Vector containing all rows of the Add trace.
    pub table: Vec<AddTraceTableRow>,
}

/// Represents a single row in the `AddTraceTable`.
///
/// Contains all the necessary values for evaluating the Add AIR constraints,
/// including current/next state IDs, input/output values, and LogUp multiplicities.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddTraceTableRow {
    /// ID of the current Add node.
    pub node_id: M31,
    /// ID of the node providing the left-hand side input.
    pub lhs_id: M31,
    /// ID of the node providing the right-hand side input.
    pub rhs_id: M31,
    /// Index within the tensor for this operation.
    pub idx: M31,
    /// Flag indicating if this is the last element processed for this node (1 if true, 0 otherwise).
    pub is_last_idx: M31,
    /// ID of the *next* Add node processed in the trace (often the same as `node_id`).
    pub next_node_id: M31,
    /// ID of the *next* LHS provider node (often the same as `lhs_id`).
    pub next_lhs_id: M31,
    /// ID of the *next* RHS provider node (often the same as `rhs_id`).
    pub next_rhs_id: M31,
    /// Index of the *next* element processed (often `idx + 1`).
    pub next_idx: M31,
    /// Value of the left-hand side input.
    pub lhs: M31,
    /// Value of the right-hand side input.
    pub rhs: M31,
    /// Value of the output (`lhs + rhs`).
    pub out: M31,
    /// Multiplicity contribution for the LogUp argument related to the LHS input.
    pub lhs_mult: M31,
    /// Multiplicity contribution for the LogUp argument related to the RHS input.
    pub rhs_mult: M31,
    /// Multiplicity contribution for the LogUp argument related to the output.
    pub out_mult: M31,
}

impl AddTraceTableRow {
    /// Creates a default padding row for the Add trace.
    /// Padding rows are added to ensure the trace length is a power of two.
    /// They should be designed to satisfy constraints trivially.
    pub(crate) fn padding() -> Self {
        Self {
            node_id: M31::zero(),
            lhs_id: M31::zero(),
            rhs_id: M31::zero(),
            idx: M31::zero(),
            is_last_idx: M31::one(),
            next_node_id: M31::zero(),
            next_lhs_id: M31::zero(),
            next_rhs_id: M31::zero(),
            next_idx: M31::zero(),
            lhs: M31::zero(),
            rhs: M31::zero(),
            out: M31::zero(),
            lhs_mult: M31::zero(),
            rhs_mult: M31::zero(),
            out_mult: M31::zero(),
        }
    }
}

/// SIMD-packed representation of an `AddTraceTableRow`.
/// Holds `N_LANES` rows packed into SIMD registers for efficient processing.
#[derive(Debug, Copy, Clone)]
pub struct PackedAddTraceTableRow {
    /// Packed `node_id` values.
    pub node_id: PackedM31,
    /// Packed `lhs_id` values.
    pub lhs_id: PackedM31,
    /// Packed `rhs_id` values.
    pub rhs_id: PackedM31,
    /// Packed `idx` values.
    pub idx: PackedM31,
    /// Packed `is_last_idx` values.
    pub is_last_idx: PackedM31,
    /// Packed `next_node_id` values.
    pub next_node_id: PackedM31,
    /// Packed `next_lhs_id` values.
    pub next_lhs_id: PackedM31,
    /// Packed `next_rhs_id` values.
    pub next_rhs_id: PackedM31,
    /// Packed `next_idx` values.
    pub next_idx: PackedM31,
    /// Packed `lhs` values.
    pub lhs: PackedM31,
    /// Packed `rhs` values.
    pub rhs: PackedM31,
    /// Packed `out` values.
    pub out: PackedM31,
    /// Packed `lhs_mult` values.
    pub lhs_mult: PackedM31,
    /// Packed `rhs_mult` values.
    pub rhs_mult: PackedM31,
    /// Packed `out_mult` values.
    pub out_mult: PackedM31,
}

impl Pack for AddTraceTableRow {
    type SimdType = PackedAddTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedAddTraceTableRow {
            node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].node_id)),
            lhs_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].lhs_id)),
            rhs_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].rhs_id)),
            idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].idx)),
            is_last_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_idx)),
            next_node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_node_id)),
            next_lhs_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_lhs_id)),
            next_rhs_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_rhs_id)),
            next_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_idx)),
            lhs: PackedM31::from_array(std::array::from_fn(|i| inputs[i].lhs)),
            rhs: PackedM31::from_array(std::array::from_fn(|i| inputs[i].rhs)),
            out: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out)),
            lhs_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].lhs_mult)),
            rhs_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].rhs_mult)),
            out_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out_mult)),
        }
    }
}

impl Unpack for PackedAddTraceTableRow {
    type CpuType = AddTraceTableRow;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        let (
            node_id,
            lhs_id,
            rhs_id,
            idx,
            is_last_idx,
            next_node_id,
            next_lhs_id,
            next_rhs_id,
            next_idx,
            lhs,
            rhs,
            out,
            lhs_mult,
            rhs_mult,
            out_mult,
        ) = (
            self.node_id.to_array(),
            self.lhs_id.to_array(),
            self.rhs_id.to_array(),
            self.idx.to_array(),
            self.is_last_idx.to_array(),
            self.next_node_id.to_array(),
            self.next_lhs_id.to_array(),
            self.next_rhs_id.to_array(),
            self.next_idx.to_array(),
            self.lhs.to_array(),
            self.rhs.to_array(),
            self.out.to_array(),
            self.lhs_mult.to_array(),
            self.rhs_mult.to_array(),
            self.out_mult.to_array(),
        );

        std::array::from_fn(|i| AddTraceTableRow {
            node_id: node_id[i],
            lhs_id: lhs_id[i],
            rhs_id: rhs_id[i],
            idx: idx[i],
            is_last_idx: is_last_idx[i],
            next_node_id: next_node_id[i],
            next_lhs_id: next_lhs_id[i],
            next_rhs_id: next_rhs_id[i],
            next_idx: next_idx[i],
            lhs: lhs[i],
            rhs: rhs[i],
            out: out[i],
            lhs_mult: lhs_mult[i],
            rhs_mult: rhs_mult[i],
            out_mult: out_mult[i],
        })
    }
}

impl AddTraceTable {
    /// Creates a new, empty `AddTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row to the trace table.
    pub fn add_row(&mut self, row: AddTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the Add AIR component's trace.
/// Provides a mapping from meaningful names to column indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddColumn {
    /// ID of the current Add node.
    NodeId,
    /// ID of the node providing the left-hand side input.
    LhsId,
    /// ID of the node providing the right-hand side input.
    RhsId,
    /// Index within the tensor for this operation.
    Idx,
    /// Flag indicating if this is the last element processed for this node.
    IsLastIdx,
    /// ID of the *next* Add node processed in the trace.
    NextNodeId,
    /// ID of the *next* LHS provider node.
    NextLhsId,
    /// ID of the *next* RHS provider node.
    NextRhsId,
    /// Index of the *next* element processed.
    NextIdx,
    /// Value of the left-hand side input.
    Lhs,
    /// Value of the right-hand side input.
    Rhs,
    /// Value of the output (`lhs + rhs`).
    Out,
    /// Multiplicity for the LogUp argument (LHS input).
    LhsMult,
    /// Multiplicity for the LogUp argument (RHS input).
    RhsMult,
    /// Multiplicity for the LogUp argument (output).
    OutMult,
}

impl AddColumn {
    /// Returns the 0-based index for this column within the Add trace segment.
    pub const fn index(self) -> usize {
        match self {
            Self::NodeId => 0,
            Self::LhsId => 1,
            Self::RhsId => 2,
            Self::Idx => 3,
            Self::IsLastIdx => 4,
            Self::NextNodeId => 5,
            Self::NextLhsId => 6,
            Self::NextRhsId => 7,
            Self::NextIdx => 8,
            Self::Lhs => 9,
            Self::Rhs => 10,
            Self::Out => 11,
            Self::LhsMult => 12,
            Self::RhsMult => 13,
            Self::OutMult => 14,
        }
    }
}

/// Implements the `TraceColumn` trait for `AddColumn`.
impl TraceColumn for AddColumn {
    /// Specifies the number of columns used by the Add component.
    /// Returns `(N_TRACE_COLUMNS, 3)`, indicating the number of main trace columns
    /// and the number of interaction trace columns (for LogUp).
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 3)
    }
}
