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

/// Represents the raw trace data collected for Sqrt operations.
///
/// Stores rows capturing inputs, outputs, remainder (for fixed-point sqrt),
/// and metadata for each Sqrt operation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SqrtTraceTable {
    /// Vector containing all rows of the Sqrt trace.
    pub table: Vec<SqrtTraceTableRow>,
}

/// Represents a single row in the `SqrtTraceTable`.
///
/// Contains values for evaluating Sqrt AIR constraints: current/next state IDs,
/// input/output values, fixed-point remainder, scale factor, and LogUp multiplicities.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SqrtTraceTableRow {
    /// ID of the current Sqrt node.
    pub node_id: M31,
    /// ID of the node providing the input.
    pub input_id: M31,
    /// Index within the tensor for this operation.
    pub idx: M31,
    /// Flag indicating if this is the last element processed for this node (1 if true, 0 otherwise).
    pub is_last_idx: M31,
    /// ID of the *next* Sqrt node processed in the trace.
    pub next_node_id: M31,
    /// ID of the *next* input provider node.
    pub next_input_id: M31,
    /// Index of the *next* element processed.
    pub next_idx: M31,
    /// Value of the input (`x`).
    pub input: M31,
    /// Value of the output (`Sqrt(x)`).
    pub out: M31,
    /// Remainder from fixed-point sqrt (`SCALE % x`).
    pub rem: M31,
    /// The scale factor used (typically `numerair::SCALE_FACTOR`).
    pub scale: M31,
    /// Multiplicity contribution for the LogUp argument (input).
    pub input_mult: M31,
    /// Multiplicity contribution for the LogUp argument (output).
    pub out_mult: M31,
}

impl SqrtTraceTableRow {
    /// Creates a default padding row for the Sqrt trace.
    pub(crate) fn padding() -> Self {
        Self {
            node_id: M31::zero(),
            input_id: M31::zero(),
            idx: M31::zero(),
            is_last_idx: M31::one(),
            next_node_id: M31::zero(),
            next_input_id: M31::zero(),
            next_idx: M31::zero(),
            input: M31::zero(),
            out: M31::zero(),
            rem: M31::zero(),
            scale: M31::zero(),
            input_mult: M31::zero(),
            out_mult: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `SqrtTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedSqrtTraceTableRow {
    /// Packed `node_id` values.
    pub node_id: PackedM31,
    /// Packed `input_id` values.
    pub input_id: PackedM31,
    /// Packed `idx` values.
    pub idx: PackedM31,
    /// Packed `is_last_idx` values.
    pub is_last_idx: PackedM31,
    /// Packed `next_node_id` values.
    pub next_node_id: PackedM31,
    /// Packed `next_input_id` values.
    pub next_input_id: PackedM31,
    /// Packed `next_idx` values.
    pub next_idx: PackedM31,
    /// Packed `input` values.
    pub input: PackedM31,
    /// Packed `out` values.
    pub out: PackedM31,
    /// Packed `rem` values.
    pub rem: PackedM31,
    /// Packed `scale` values.
    pub scale: PackedM31,
    /// Packed `input_mult` values.
    pub input_mult: PackedM31,
    /// Packed `out_mult` values.
    pub out_mult: PackedM31,
}

impl Pack for SqrtTraceTableRow {
    type SimdType = PackedSqrtTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedSqrtTraceTableRow {
            node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].node_id)),
            input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_id)),
            idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].idx)),
            is_last_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_idx)),
            next_node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_node_id)),
            next_input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_input_id)),
            next_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_idx)),
            input: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input)),
            out: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out)),
            rem: PackedM31::from_array(std::array::from_fn(|i| inputs[i].rem)),
            scale: PackedM31::from_array(std::array::from_fn(|i| inputs[i].scale)),
            input_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_mult)),
            out_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out_mult)),
        }
    }
}

impl Unpack for PackedSqrtTraceTableRow {
    type CpuType = SqrtTraceTableRow;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        let (
            node_id,
            input_id,
            idx,
            is_last_idx,
            next_node_id,
            next_input_id,
            next_idx,
            input,
            out,
            rem,
            scale,
            input_mult,
            out_mult,
        ) = (
            self.node_id.to_array(),
            self.input_id.to_array(),
            self.idx.to_array(),
            self.is_last_idx.to_array(),
            self.next_node_id.to_array(),
            self.next_input_id.to_array(),
            self.next_idx.to_array(),
            self.input.to_array(),
            self.out.to_array(),
            self.rem.to_array(),
            self.scale.to_array(),
            self.input_mult.to_array(),
            self.out_mult.to_array(),
        );

        std::array::from_fn(|i| SqrtTraceTableRow {
            node_id: node_id[i],
            input_id: input_id[i],
            idx: idx[i],
            is_last_idx: is_last_idx[i],
            next_node_id: next_node_id[i],
            next_input_id: next_input_id[i],
            next_idx: next_idx[i],
            input: input[i],
            out: out[i],
            rem: rem[i],
            scale: scale[i],
            input_mult: input_mult[i],
            out_mult: out_mult[i],
        })
    }
}

impl SqrtTraceTable {
    /// Creates a new, empty `SqrtTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row to the trace table.
    pub fn add_row(&mut self, row: SqrtTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the Sqrt AIR component's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SqrtColumn {
    /// ID of the current Sqrt node.
    NodeId,
    /// ID of the node providing the input.
    InputId,
    /// Index within the tensor for this operation.
    Idx,
    /// Flag indicating if this is the last element processed for this node.
    IsLastIdx,
    /// ID of the *next* Sqrt node processed in the trace.
    NextNodeId,
    /// ID of the *next* input provider node.
    NextInputId,
    /// Index of the *next* element processed.
    NextIdx,
    /// Value of the input (`x`).
    Input,
    /// Value of the output (`Sqrt(x)`).
    Out,
    /// Remainder from fixed-point sqrt (`SCALE % x`).
    Rem,
    /// The scale factor used.
    Scale,
    /// Multiplicity for the LogUp argument (input).
    InputMult,
    /// Multiplicity for the LogUp argument (output).
    OutMult,
}

impl SqrtColumn {
    /// Returns the 0-based index for this column within the Sqrt trace segment.
    pub const fn index(self) -> usize {
        match self {
            Self::NodeId => 0,
            Self::InputId => 1,
            Self::Idx => 2,
            Self::IsLastIdx => 3,
            Self::NextNodeId => 4,
            Self::NextInputId => 5,
            Self::NextIdx => 6,
            Self::Input => 7,
            Self::Out => 8,
            Self::Rem => 9,
            Self::Scale => 10,
            Self::InputMult => 11,
            Self::OutMult => 12,
        }
    }
}

/// Implements the `TraceColumn` trait for `SqrtColumn`.
impl TraceColumn for SqrtColumn {
    /// Specifies the number of columns used by the Sqrt component.
    /// Returns `(N_TRACE_COLUMNS, 2)`, indicating the number of main trace columns
    /// and 2 interaction trace columns (for input and output LogUp).
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 2)
    }
}
