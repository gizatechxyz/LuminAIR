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
pub struct RecipTraceTable {
    pub table: Vec<RecipTraceTableRow>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecipTraceTableRow {
    pub node_id: M31,
    pub input_id: M31,
    pub idx: M31,
    pub is_last_idx: M31,
    pub next_node_id: M31,
    pub next_input_id: M31,
    pub next_idx: M31,
    pub input: M31,
    pub out: M31,
    pub rem: M31,
    pub scale: M31,
    pub input_mult: M31,
    pub out_mult: M31,
}

impl RecipTraceTableRow {
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

#[derive(Debug, Copy, Clone)]
pub struct PackedRecipTraceTableRow {
    pub node_id: PackedM31,
    pub input_id: PackedM31,
    pub idx: PackedM31,
    pub is_last_idx: PackedM31,
    pub next_node_id: PackedM31,
    pub next_input_id: PackedM31,
    pub next_idx: PackedM31,
    pub input: PackedM31,
    pub out: PackedM31,
    pub rem: PackedM31,
    pub scale: PackedM31,
    pub input_mult: PackedM31,
    pub out_mult: PackedM31,
}

impl Pack for RecipTraceTableRow {
    type SimdType = PackedRecipTraceTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedRecipTraceTableRow {
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

impl Unpack for PackedRecipTraceTableRow {
    type CpuType = RecipTraceTableRow;

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

        std::array::from_fn(|i| RecipTraceTableRow {
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

impl RecipTraceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: RecipTraceTableRow) {
        self.table.push(row);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecipColumn {
    NodeId,
    InputId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextInputId,
    NextIdx,
    Input,
    Out,
    Rem,
    Scale,
    InputMult,
    OutMult,
}

impl RecipColumn {
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

impl TraceColumn for RecipColumn {
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 2)
    }
}
