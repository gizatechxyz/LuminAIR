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

/// Represents the raw trace data collected for Sine (`sin(x)`) operations.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SinTraceTable {
    /// Vector containing all rows of the Sin trace.
    pub table: Vec<SinTraceTableRow>,
}

/// Represents a single row in the `SinTraceTable`.
///
/// Contains values for evaluating Sin AIR constraints: current/next state IDs,
/// input/output values, and multiplicities for LogUp (input/output) and LUT interaction.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinTraceTableRow {
    /// ID of the current Sin node.
    pub node_id: M31,
    /// ID of the node providing the input.
    pub input_id: M31,
    /// Index within the tensor for this operation.
    pub idx: M31,
    /// Flag indicating if this is the last element processed for this node (1 if true, 0 otherwise).
    pub is_last_idx: M31,
    /// ID of the *next* Sin node processed in the trace.
    pub next_node_id: M31,
    /// ID of the *next* input provider node.
    pub next_input_id: M31,
    /// Index of the *next* element processed.
    pub next_idx: M31,
    /// Value of the input (`x`).
    pub input: M31,
    /// Value of the output (`sin(x)`).
    pub out: M31,
    /// Multiplicity contribution for the LogUp argument (input).
    pub input_mult: M31,
    /// Multiplicity contribution for the LogUp argument (output).
    pub out_mult: M31,
    /// Multiplicity contribution for the Sine Lookup Table interaction.
    pub lookup_mult: M31,
}

impl SinTraceTableRow {
    /// Creates a default padding row for the Sin trace.
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
            input_mult: M31::zero(),
            out_mult: M31::zero(),
            lookup_mult: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `SinTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedSinTraceTableRow {
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
    /// Packed `input_mult` values.
    pub input_mult: PackedM31,
    /// Packed `out_mult` values.
    pub out_mult: PackedM31,
    /// Packed `lookup_mult` values.
    pub lookup_mult: PackedM31,
}

impl Pack for SinTraceTableRow {
    type SimdType = PackedSinTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedSinTraceTableRow {
            node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].node_id)),
            input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_id)),
            idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].idx)),
            is_last_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_idx)),
            next_node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_node_id)),
            next_input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_input_id)),
            next_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_idx)),
            input: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input)),
            out: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out)),
            input_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_mult)),
            out_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out_mult)),
            lookup_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].lookup_mult)),
        }
    }
}

impl Unpack for PackedSinTraceTableRow {
    type CpuType = SinTraceTableRow;

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
            input_mult,
            out_mult,
            lookup_mult,
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
            self.input_mult.to_array(),
            self.out_mult.to_array(),
            self.lookup_mult.to_array(),
        );

        std::array::from_fn(|i| SinTraceTableRow {
            node_id: node_id[i],
            input_id: input_id[i],
            idx: idx[i],
            is_last_idx: is_last_idx[i],
            next_node_id: next_node_id[i],
            next_input_id: next_input_id[i],
            next_idx: next_idx[i],
            input: input[i],
            out: out[i],
            input_mult: input_mult[i],
            out_mult: out_mult[i],
            lookup_mult: lookup_mult[i],
        })
    }
}

impl SinTraceTable {
    /// Creates a new, empty `SinTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row to the trace table.
    pub fn add_row(&mut self, row: SinTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the Sin AIR component's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinColumn {
    /// ID of the current Sin node.
    NodeId,
    /// ID of the node providing the input.
    InputId,
    /// Index within the tensor for this operation.
    Idx,
    /// Flag indicating if this is the last element processed for this node.
    IsLastIdx,
    /// ID of the *next* Sin node processed in the trace.
    NextNodeId,
    /// ID of the *next* input provider node.
    NextInputId,
    /// Index of the *next* element processed.
    NextIdx,
    /// Value of the input (`x`).
    Input,
    /// Value of the output (`sin(x)`).
    Out,
    /// Multiplicity for the LogUp argument (input).
    InputMult,
    /// Multiplicity for the LogUp argument (output).
    OutMult,
    /// Multiplicity for the Sine Lookup Table interaction.
    LookupMult,
}

impl SinColumn {
    /// Returns the 0-based index for this column within the Sin trace segment.
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
            Self::InputMult => 9,
            Self::OutMult => 10,
            Self::LookupMult => 11,
        }
    }
}

/// Implements the `TraceColumn` trait for `SinColumn`.
impl TraceColumn for SinColumn {
    /// Specifies the number of columns used by the Sin component.
    /// Returns `(N_TRACE_COLUMNS, 3)`, indicating the number of main trace columns
    /// and 3 interaction trace columns (input LogUp, output LogUp, LUT interaction).
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 3)
    }
}
