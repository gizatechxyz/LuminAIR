use num_traits::{One, Zero};
use stwo_prover::core::{
    backend::simd::{
        conversion::{Pack, Unpack},
        m31::{PackedM31, N_LANES},
    },
    fields::m31::M31,
};

use crate::components::TraceColumn;

use super::witness::N_TRACE_COLUMNS;

/// Represents the raw trace data collected for Max-Reduce operations.
///
/// Stores rows capturing the step-by-step comparison and update process for finding
/// the maximum value, along with inputs, outputs, and metadata.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MaxReduceTraceTable {
    /// Vector containing all rows of the MaxReduce trace.
    pub table: Vec<MaxReduceTraceTableRow>,
}

/// Represents a single row in the `MaxReduceTraceTable`.
///
/// Contains values for MaxReduce AIR constraints: current/next state IDs,
/// current input value, current/next running maximum (`max_val`, `next_max_val`),
/// a flag `is_max` indicating if `input` became `next_max_val`,
/// flags for last step/idx, the final output, and LogUp multiplicities.
#[derive(Debug, Default, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaxReduceTraceTableRow {
    /// ID of the current MaxReduce node.
    pub node_id: M31,
    /// ID of the node providing the input tensor.
    pub input_id: M31,
    /// Index of the output element being computed.
    pub idx: M31,
    /// Flag: is this the last output element for this node (1 if true, 0 otherwise).
    pub is_last_idx: M31,
    /// ID of the *next* MaxReduce node processed in the trace.
    pub next_node_id: M31,
    /// ID of the *next* input provider node.
    pub next_input_id: M31,
    /// Index of the *next* output element.
    pub next_idx: M31,
    /// Current input value being processed in the reduction.
    pub input: M31,
    /// Final output value (max for `idx`). Valid only if `is_last_step` is 1.
    pub out: M31,
    /// Running maximum value *before* considering the current `input`.
    pub max_val: M31,
    /// Running maximum value *after* considering the current `input`.
    pub next_max_val: M31,
    /// Flag: is this the last input element being processed for the current `out` (1 if true, 0 otherwise).
    pub is_last_step: M31,
    /// Flag: is the current `input` value the new maximum (1 if true, 0 otherwise).
    pub is_max: M31,
    /// Multiplicity contribution for the LogUp argument (input tensor values).
    pub input_mult: M31,
    /// Multiplicity contribution for the LogUp argument (output tensor values).
    pub out_mult: M31,
}

impl MaxReduceTraceTableRow {
    /// Creates a default padding row for the MaxReduce trace.
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
            max_val: M31::zero(),
            next_max_val: M31::zero(),
            is_last_step: M31::zero(),
            is_max: M31::zero(),
            input_mult: M31::zero(),
            out_mult: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `MaxReduceTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedMaxReduceTraceTableRow {
    /// Packed `node_id` values.
    pub node_id: PackedM31,
    /// Packed `input_id` values.
    pub input_id: PackedM31,
    /// Packed `idx` (output element index) values.
    pub idx: PackedM31,
    /// Packed `is_last_idx` flags.
    pub is_last_idx: PackedM31,
    /// Packed `next_node_id` values.
    pub next_node_id: PackedM31,
    /// Packed `next_input_id` values.
    pub next_input_id: PackedM31,
    /// Packed `next_idx` values.
    pub next_idx: PackedM31,
    /// Packed current `input` values for reduction.
    pub input: PackedM31,
    /// Packed `out` (final max) values.
    pub out: PackedM31,
    /// Packed `max_val` (running max before input) values.
    pub max_val: PackedM31,
    /// Packed `next_max_val` (running max after input) values.
    pub next_max_val: PackedM31,
    /// Packed `is_last_step` flags (for reduction step).
    pub is_last_step: PackedM31,
    /// Packed `is_max` flags (if current input is the new max).
    pub is_max: PackedM31,
    /// Packed `input_mult` values.
    pub input_mult: PackedM31,
    /// Packed `out_mult` values.
    pub out_mult: PackedM31,
}

impl Pack for MaxReduceTraceTableRow {
    type SimdType = PackedMaxReduceTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedMaxReduceTraceTableRow {
            node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].node_id)),
            input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_id)),
            idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].idx)),
            is_last_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_idx)),
            next_node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_node_id)),
            next_input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_input_id)),
            next_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_idx)),
            input: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input)),
            out: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out)),
            max_val: PackedM31::from_array(std::array::from_fn(|i| inputs[i].max_val)),
            next_max_val: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_max_val)),
            is_last_step: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_step)),
            is_max: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_max)),
            input_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_mult)),
            out_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out_mult)),
        }
    }
}

impl Unpack for PackedMaxReduceTraceTableRow {
    type CpuType = MaxReduceTraceTableRow;

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
            max_val,
            next_max_val,
            is_last_step,
            is_max,
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
            self.max_val.to_array(),
            self.next_max_val.to_array(),
            self.is_last_step.to_array(),
            self.is_max.to_array(),
            self.input_mult.to_array(),
            self.out_mult.to_array(),
        );

        std::array::from_fn(|i| MaxReduceTraceTableRow {
            node_id: node_id[i],
            input_id: input_id[i],
            idx: idx[i],
            is_last_idx: is_last_idx[i],
            next_node_id: next_node_id[i],
            next_input_id: next_input_id[i],
            next_idx: next_idx[i],
            input: input[i],
            out: out[i],
            max_val: max_val[i],
            next_max_val: next_max_val[i],
            is_last_step: is_last_step[i],
            is_max: is_max[i],
            input_mult: input_mult[i],
            out_mult: out_mult[i],
        })
    }
}

impl MaxReduceTraceTable {
    /// Creates a new, empty `MaxReduceTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row to the trace table.
    pub fn add_row(&mut self, row: MaxReduceTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the MaxReduce AIR component's trace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MaxReduceColumn {
    NodeId, InputId, Idx, IsLastIdx, NextNodeId, NextInputId, NextIdx, Input, Out, MaxVal, NextMaxVal, IsLastStep, IsMax, InputMult, OutMult
}
impl MaxReduceColumn {
    /// Returns the 0-based index for this column within the MaxReduce trace segment.
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
            Self::MaxVal => 9,
            Self::NextMaxVal => 10,
            Self::IsLastStep => 11,
            Self::IsMax => 12,
            Self::InputMult => 13,
            Self::OutMult => 14,
        }
    }
}

/// Implements the `TraceColumn` trait for `MaxReduceColumn`.
impl TraceColumn for MaxReduceColumn {
    /// Specifies the number of columns used by the MaxReduce component.
    /// Returns `(N_TRACE_COLUMNS, 2)`, indicating main trace columns
    /// and 2 interaction trace columns (for input and output LogUp).
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 2)
    }
}
