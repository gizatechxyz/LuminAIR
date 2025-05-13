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

/// Represents the raw trace data collected for Multiplication operations.
///
/// Stores rows generated during the `gen_trace` phase, capturing the inputs,
/// outputs, remainder (for fixed-point), and metadata for each Mul operation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MulTraceTable {
    /// Vector containing all rows of the Mul trace.
    pub table: Vec<MulTraceTableRow>,
}

/// Represents a single row in the `MulTraceTable`.
///
/// Contains values for evaluating Mul AIR constraints, including current/next state IDs,
/// input/output values, fixed-point remainder, and LogUp multiplicities.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct MulTraceTableRow {
    /// ID of the current Mul node.
    pub node_id: M31,
    /// ID of the node providing the left-hand side input.
    pub lhs_id: M31,
    /// ID of the node providing the right-hand side input.
    pub rhs_id: M31,
    /// Index within the tensor for this operation.
    pub idx: M31,
    /// Flag indicating if this is the last element processed for this node (1 if true, 0 otherwise).
    pub is_last_idx: M31,
    /// ID of the *next* Mul node processed in the trace.
    pub next_node_id: M31,
    /// ID of the *next* LHS provider node.
    pub next_lhs_id: M31,
    /// ID of the *next* RHS provider node.
    pub next_rhs_id: M31,
    /// Index of the *next* element processed.
    pub next_idx: M31,
    /// Value of the left-hand side input.
    pub lhs: M31,
    /// Value of the right-hand side input.
    pub rhs: M31,
    /// Value of the output (`(lhs * rhs) / SCALE`).
    pub out: M31,
    /// Remainder from fixed-point multiplication (`lhs * rhs % SCALE`).
    pub rem: M31,
    /// Multiplicity contribution for the LogUp argument (LHS input).
    pub lhs_mult: M31,
    /// Multiplicity contribution for the LogUp argument (RHS input).
    pub rhs_mult: M31,
    /// Multiplicity contribution for the LogUp argument (output).
    pub out_mult: M31,
}

impl MulTraceTableRow {
    /// Creates a default padding row for the Mul trace.
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
            rem: M31::zero(),
            lhs_mult: M31::zero(),
            rhs_mult: M31::zero(),
            out_mult: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `MulTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedMulTraceTableRow {
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
    /// Packed `rem` values.
    pub rem: PackedM31,
    /// Packed `lhs_mult` values.
    pub lhs_mult: PackedM31,
    /// Packed `rhs_mult` values.
    pub rhs_mult: PackedM31,
    /// Packed `out_mult` values.
    pub out_mult: PackedM31,
}

impl Pack for MulTraceTableRow {
    type SimdType = PackedMulTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedMulTraceTableRow {
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
            rem: PackedM31::from_array(std::array::from_fn(|i| inputs[i].rem)),
            lhs_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].lhs_mult)),
            rhs_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].rhs_mult)),
            out_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out_mult)),
        }
    }
}

impl Unpack for PackedMulTraceTableRow {
    type CpuType = MulTraceTableRow;

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
            rem,
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
            self.rem.to_array(),
            self.lhs_mult.to_array(),
            self.rhs_mult.to_array(),
            self.out_mult.to_array(),
        );

        std::array::from_fn(|i| MulTraceTableRow {
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
            rem: rem[i],
            lhs_mult: lhs_mult[i],
            rhs_mult: rhs_mult[i],
            out_mult: out_mult[i],
        })
    }
}

impl MulTraceTable {
    /// Creates a new, empty `MulTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row to the trace table.
    pub fn add_row(&mut self, row: MulTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the Mul AIR component's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MulColumn {
    /// ID of the current Mul node.
    NodeId,
    /// ID of the node providing the left-hand side input.
    LhsId,
    /// ID of the node providing the right-hand side input.
    RhsId,
    /// Index within the tensor for this operation.
    Idx,
    /// Flag indicating if this is the last element processed for this node.
    IsLastIdx,
    /// ID of the *next* Mul node processed in the trace.
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
    /// Value of the output.
    Out,
    /// Remainder from fixed-point multiplication.
    Rem,
    /// Multiplicity for the LogUp argument (LHS input).
    LhsMult,
    /// Multiplicity for the LogUp argument (RHS input).
    RhsMult,
    /// Multiplicity for the LogUp argument (output).
    OutMult,
}

impl MulColumn {
    /// Returns the 0-based index for this column within the Mul trace segment.
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
            Self::Rem => 12,
            Self::LhsMult => 13,
            Self::RhsMult => 14,
            Self::OutMult => 15,
        }
    }
}

/// Implements the `TraceColumn` trait for `MulColumn`.
impl TraceColumn for MulColumn {
    /// Specifies the number of columns used by the Mul component.
    /// Returns `(16, 3)`, indicating 16 main trace columns and 3 interaction trace columns.
    fn count() -> (usize, usize) {
        (16, 3)
    }
}
