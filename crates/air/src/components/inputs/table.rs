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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InputsTraceTable {
    pub table: Vec<InputsTraceTableRow>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputsTraceTableRow {
    pub node_id: M31,
    pub idx: M31,
    pub is_last_idx: M31,
    pub next_node_id: M31,
    pub next_idx: M31,
    pub val: M31,
    pub multiplicity: M31,
}

impl InputsTraceTableRow {
    pub(crate) fn padding() -> Self {
        Self {
            node_id: M31::zero(),
            idx: M31::zero(),
            is_last_idx: M31::one(),
            next_node_id: M31::zero(),
            next_idx: M31::zero(),
            val: M31::zero(),
            multiplicity: M31::zero(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PackedInputsTraceTableRow {
    pub node_id: PackedM31,
    pub idx: PackedM31,
    pub is_last_idx: PackedM31,
    pub next_node_id: PackedM31,
    pub next_idx: PackedM31,
    pub val: PackedM31,
    pub multiplicity: PackedM31,
}

impl Pack for InputsTraceTableRow {
    type SimdType = PackedInputsTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedInputsTraceTableRow {
            node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].node_id)),
            idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].idx)),
            is_last_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].is_last_idx)),
            next_node_id: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_node_id)),
            next_idx: PackedM31::from_array(std::array::from_fn(|i| inputs[i].next_idx)),
            val: PackedM31::from_array(std::array::from_fn(|i| inputs[i].val)),
            multiplicity: PackedM31::from_array(std::array::from_fn(|i| inputs[i].multiplicity)),
        }
    }
}

impl Unpack for PackedInputsTraceTableRow {
    type CpuType = InputsTraceTableRow;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        let (node_id, idx, is_last_idx, next_node_id, next_idx, val, multiplicity) = (
            self.node_id.to_array(),
            self.idx.to_array(),
            self.is_last_idx.to_array(),
            self.next_node_id.to_array(),
            self.next_idx.to_array(),
            self.val.to_array(),
            self.multiplicity.to_array(),
        );

        std::array::from_fn(|i| InputsTraceTableRow {
            node_id: node_id[i],
            idx: idx[i],
            is_last_idx: is_last_idx[i],
            next_node_id: next_node_id[i],
            next_idx: next_idx[i],
            val: val[i],
            multiplicity: multiplicity[i],
        })
    }
}

impl InputsTraceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: InputsTraceTableRow) {
        self.table.push(row);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputsColumn {
    NodeId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextIdx,
    Val,
    Multiplicity,
}

impl InputsColumn {
    pub const fn index(self) -> usize {
        match self {
            Self::NodeId => 0,
            Self::Idx => 1,
            Self::IsLastIdx => 2,
            Self::NextNodeId => 3,
            Self::NextIdx => 4,
            Self::Val => 5,
            Self::Multiplicity => 6,
        }
    }
}

impl TraceColumn for InputsColumn {
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 1)
    }
}
