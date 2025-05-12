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
pub struct SinTable {
    /// A vector of [`SinTableRow`] representing the table rows.
    pub table: Vec<SinTableRow>,
}

/// Represents a single row of the [`SinTable`]
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct SinTableRow {
    pub node_id: M31,
    pub input_id: M31,
    pub idx: M31,
    pub is_last_idx: M31,
    pub next_node_id: M31,
    pub next_input_id: M31,
    pub next_idx: M31,
    pub input: M31,
    pub out: M31,
    pub input_mult: M31,
    pub out_mult: M31,
    pub lookup_mult: M31,
}

impl SinTableRow {
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

#[derive(Debug, Copy, Clone)]
pub struct PackedSinTableRow {
    pub node_id: PackedM31,
    pub input_id: PackedM31,
    pub idx: PackedM31,
    pub is_last_idx: PackedM31,
    pub next_node_id: PackedM31,
    pub next_input_id: PackedM31,
    pub next_idx: PackedM31,
    pub input: PackedM31,
    pub out: PackedM31,
    pub input_mult: PackedM31,
    pub out_mult: PackedM31,
    pub lookup_mult: PackedM31,
}

impl Pack for SinTableRow {
    type SimdType = PackedSinTableRow;

    fn pack(inputs: [Self; N_LANES]) -> Self::SimdType {
        PackedSinTableRow {
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

impl Unpack for PackedSinTableRow {
    type CpuType = SinTableRow;

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

        std::array::from_fn(|i| SinTableRow {
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

impl SinTable {
    /// Creates a new, empty [`SinTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the Sin Table.
    pub fn add_row(&mut self, row: SinTableRow) {
        self.table.push(row);
    }
}

/// Enum representing the column indices in the Sin trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinColumn {
    NodeId,
    InputId,
    Idx,
    IsLastIdx,
    NextNodeId,
    NextInputId,
    NextIdx,
    Input,
    Out,
    InputMult,
    OutMult,
    LookupMult,
}

impl SinColumn {
    /// Returns the index of the column in the Sin trace.
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

impl TraceColumn for SinColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (N_TRACE_COLUMNS, 3)
    }
}
