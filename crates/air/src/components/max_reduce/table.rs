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

/// Represents the table for the component, containing the required registers for its
/// constraints.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MaxReduceTraceTable {
    /// A vector of [`MaxReduceTraceTableRow`] representing the table rows.
    pub table: Vec<MaxReduceTraceTableRow>,
}

/// Represents a single row of the [`MaxReduceTraceTable`]
#[derive(Debug, Default, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaxReduceTraceTableRow {
    pub node_id: M31,
    pub input_id: M31,
    pub idx: M31,
    pub is_last_idx: M31,
    pub next_node_id: M31,
    pub next_input_id: M31,
    pub next_idx: M31,
    pub input: M31,
    pub out: M31,
    pub max_val: M31,
    pub next_max_val: M31,
    pub is_last_step: M31,
    pub is_max: M31,
    pub input_mult: M31,
    pub out_mult: M31,
}

impl MaxReduceTraceTableRow {
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

#[derive(Debug, Copy, Clone)]
pub struct PackedMaxReduceTraceTableRow {
    pub node_id: PackedM31,
    pub input_id: PackedM31,
    pub idx: PackedM31,
    pub is_last_idx: PackedM31,
    pub next_node_id: PackedM31,
    pub next_input_id: PackedM31,
    pub next_idx: PackedM31,
    pub input: PackedM31,
    pub out: PackedM31,
    pub max_val: PackedM31,
    pub next_max_val: PackedM31,
    pub is_last_step: PackedM31,
    pub is_max: PackedM31,
    pub input_mult: PackedM31,
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
    /// Creates a new, empty [`MaxReduceTraceTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the TraceTable.
    pub fn add_row(&mut self, row: MaxReduceTraceTableRow) {
        self.table.push(row);
    }
}

/// Enum representing the column indices in the MaxReduce trace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MaxReduceColumn {
    NodeId,
    InputId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextInputId,
    NextIdx,
    Input,
    Out,
    MaxVal,
    NextMaxVal,
    IsLastStep,
    IsMax,
    InputMult,
    OutMult,
}
impl MaxReduceColumn {
    /// Returns the index of the column in the MaxReduce trace.
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
impl TraceColumn for MaxReduceColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 2)
    }
}
