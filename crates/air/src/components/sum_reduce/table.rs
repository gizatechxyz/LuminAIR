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

/// Represents the table for the component, containing the required registers for its
/// constraints.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SumReduceTable {
    /// A vector of [`SumReduceTableRow`] representing the table rows.
    pub table: Vec<SumReduceTableRow>,
}

/// Represents a single row of the [`SumReduceTable`]
#[derive(Debug, Default, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SumReduceTableRow {
    pub node_id: M31,
    pub input_id: M31,
    pub idx: M31,
    pub is_last_idx: M31,
    pub next_node_id: M31,
    pub next_input_id: M31,
    pub next_idx: M31,
    pub input: M31,
    pub out: M31,
    pub acc: M31,
    pub next_acc: M31,
    pub is_last_step: M31,
    pub input_mult: M31,
    pub out_mult: M31,
}

impl SumReduceTableRow {
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

#[derive(Debug, Copy, Clone)]
pub struct PackedSumReduceTableRow {
    pub node_id: PackedM31,
    pub input_id: PackedM31,
    pub idx: PackedM31,
    pub is_last_idx: PackedM31,
    pub next_node_id: PackedM31,
    pub next_input_id: PackedM31,
    pub next_idx: PackedM31,
    pub input: PackedM31,
    pub out: PackedM31,
    pub acc: PackedM31,
    pub next_acc: PackedM31,
    pub is_last_step: PackedM31,
    pub input_mult: PackedM31,
    pub out_mult: PackedM31,
}

impl Pack for SumReduceTableRow {
    type SimdType = PackedSumReduceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedSumReduceTableRow {
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

impl Unpack for PackedSumReduceTableRow {
    type CpuType = SumReduceTableRow;

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

        std::array::from_fn(|i| SumReduceTableRow {
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

impl SumReduceTable {
    /// Creates a new, empty [`SumReduceTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the Table.
    pub fn add_row(&mut self, row: SumReduceTableRow) {
        self.table.push(row);
    }
}

/// Enum representing the column indices in the SumReduce table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SumReduceColumn {
    NodeId,
    InputId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextInputId,
    NextIdx,
    Input,
    Out,
    Acc,
    NextAcc,
    IsLastStep,
    InputMult,
    OutMult,
}

impl SumReduceColumn {
    /// Returns the index of the column in the SumReduce trace.
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
impl TraceColumn for SumReduceColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 2)
    }
}
