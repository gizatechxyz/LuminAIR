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

/// Represents the raw trace data collected for Sum-Reduce operations.
///
/// Stores rows capturing the step-by-step accumulation process, inputs, outputs,
/// and metadata for each SumReduce operation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SumReduceTraceTable {
    /// Vector containing all rows of the SumReduce trace.
    pub table: Vec<SumReduceTraceTableRow>,
}

/// Represents a single row in the `SumReduceTraceTable`.
///
/// Contains values for evaluating SumReduce AIR constraints: current/next state IDs,
/// current input value, current/next accumulator value, a flag indicating the last step
/// of reduction for an output element, the final output (valid on last step),
/// and LogUp multiplicities.
#[derive(Debug, Default, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SumReduceTraceTableRow {
    /// ID of the current SumReduce node.
    pub node_id: M31,
    /// ID of the node providing the input tensor.
    pub input_id: M31,
    /// Index of the output element being computed.
    pub idx: M31,
    /// Flag: is this the last output element for this node (1 if true, 0 otherwise).
    pub is_last_idx: M31,
    /// ID of the *next* SumReduce node processed in the trace.
    pub next_node_id: M31,
    /// ID of the *next* input provider node.
    pub next_input_id: M31,
    /// Index of the *next* output element.
    pub next_idx: M31,
    /// Current input value being processed in the reduction sum.
    pub input: M31,
    /// Final output value (sum for `idx`). Valid only if `is_last_step` is 1.
    pub out: M31,
    /// Accumulator value *before* adding the current `input`.
    pub acc: M31,
    /// Accumulator value *after* adding the current `input` (`acc + input`).
    pub next_acc: M31,
    /// Flag: is this the last input element being summed for the current `out` (1 if true, 0 otherwise).
    pub is_last_step: M31,
    /// Multiplicity contribution for the LogUp argument (input tensor values).
    pub input_mult: M31,
    /// Multiplicity contribution for the LogUp argument (output tensor values).
    pub out_mult: M31,
}

impl SumReduceTraceTableRow {
    /// Creates a default padding row for the SumReduce trace.
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
            acc: M31::zero(),
            next_acc: M31::zero(),
            is_last_step: M31::zero(),
            input_mult: M31::zero(),
            out_mult: M31::zero(),
        }
    }
}

/// SIMD-packed representation of a `SumReduceTraceTableRow`.
#[derive(Debug, Copy, Clone)]
pub struct PackedSumReduceTraceTableRow {
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
    /// Packed `out` (final sum) values.
    pub out: PackedM31,
    /// Packed `acc` (accumulator before input) values.
    pub acc: PackedM31,
    /// Packed `next_acc` (accumulator after input) values.
    pub next_acc: PackedM31,
    /// Packed `is_last_step` flags (for reduction step).
    pub is_last_step: PackedM31,
    /// Packed `input_mult` values.
    pub input_mult: PackedM31,
    /// Packed `out_mult` values.
    pub out_mult: PackedM31,
}

impl Pack for SumReduceTraceTableRow {
    type SimdType = PackedSumReduceTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedSumReduceTraceTableRow {
            node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].node_id)),
            input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_id)),
            idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].idx)),
            is_last_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_idx)),
            next_node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_node_id)),
            next_input_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_input_id)),
            next_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_idx)),
            input: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input)),
            out: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out)),
            acc: PackedM31::from_array(std::array::from_fn(|i| inputs[i].acc)),
            next_acc: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_acc)),
            is_last_step: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_step)),
            input_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].input_mult)),
            out_mult: PackedM31::from_array(std::array::from_fn(|i| inputs[i].out_mult)),
        }
    }
}

impl Unpack for PackedSumReduceTraceTableRow {
    type CpuType = SumReduceTraceTableRow;

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
            acc,
            next_acc,
            is_last_step,
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
            self.acc.to_array(),
            self.next_acc.to_array(),
            self.is_last_step.to_array(),
            self.input_mult.to_array(),
            self.out_mult.to_array(),
        );

        std::array::from_fn(|i| SumReduceTraceTableRow {
            node_id: node_id[i],
            input_id: input_id[i],
            idx: idx[i],
            is_last_idx: is_last_idx[i],
            next_node_id: next_node_id[i],
            next_input_id: next_input_id[i],
            next_idx: next_idx[i],
            input: input[i],
            out: out[i],
            acc: acc[i],
            next_acc: next_acc[i],
            is_last_step: is_last_step[i],
            input_mult: input_mult[i],
            out_mult: out_mult[i],
        })
    }
}

impl SumReduceTraceTable {
    /// Creates a new, empty `SumReduceTraceTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single row to the trace table.
    pub fn add_row(&mut self, row: SumReduceTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum defining the columns of the SumReduce AIR component's trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SumReduceColumn {
    NodeId, InputId, Idx, IsLastIdx, NextNodeId, NextInputId, NextIdx, Input, Out, Acc, NextAcc, IsLastStep, InputMult, OutMult
}

impl SumReduceColumn {
    /// Returns the 0-based index for this column within the SumReduce trace segment.
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
            Self::Acc => 9,
            Self::NextAcc => 10,
            Self::IsLastStep => 11,
            Self::InputMult => 12,
            Self::OutMult => 13,
        }
    }
}

/// Implements the `TraceColumn` trait for `SumReduceColumn`.
impl TraceColumn for SumReduceColumn {
    /// Specifies the number of columns used by the SumReduce component.
    /// Returns `(N_TRACE_COLUMNS, 2)`, indicating main trace columns
    /// and 2 interaction trace columns (for input and output LogUp).
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 2)
    }
}
