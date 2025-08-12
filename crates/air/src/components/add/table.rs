use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use stwo::{
    core::fields::m31::M31,
    prover::backend::simd::{
        conversion::{Pack, Unpack},
        m31::{PackedM31, N_LANES},
    },
};

use crate::components::TraceColumn;

use super::witness::N_TRACE_COLUMNS;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AddTraceTable {
    pub table: Vec<AddTraceTableRow>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddTraceTableRow {
    pub node_id: M31,
    pub lhs_id: M31,
    pub rhs_id: M31,
    pub idx: M31,
    pub is_last_idx: M31,
    pub next_node_id: M31,
    pub next_lhs_id: M31,
    pub next_rhs_id: M31,
    pub next_idx: M31,
    pub lhs: M31,
    pub rhs: M31,
    pub out: M31,
    pub lhs_mult: M31,
    pub rhs_mult: M31,
    pub out_mult: M31,
}

impl AddTraceTableRow {
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

#[derive(Debug, Copy, Clone)]
pub struct PackedAddTraceTableRow {
    pub node_id: PackedM31,
    pub lhs_id: PackedM31,
    pub rhs_id: PackedM31,
    pub idx: PackedM31,
    pub is_last_idx: PackedM31,
    pub next_node_id: PackedM31,
    pub next_lhs_id: PackedM31,
    pub next_rhs_id: PackedM31,
    pub next_idx: PackedM31,
    pub lhs: PackedM31,
    pub rhs: PackedM31,
    pub out: PackedM31,
    pub lhs_mult: PackedM31,
    pub rhs_mult: PackedM31,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: AddTraceTableRow) {
        self.table.push(row);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddColumn {
    NodeId,
    LhsId,
    RhsId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextLhsId,
    NextRhsId,
    NextIdx,
    Lhs,
    Rhs,
    Out,
    LhsMult,
    RhsMult,
    OutMult,
}

impl AddColumn {
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

impl TraceColumn for AddColumn {
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 3)
    }
}
