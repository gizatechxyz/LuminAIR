use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_prover::core::backend::simd::{
    m31::{LOG_N_LANES, N_LANES},
    SimdBackend,
};

use crate::{
    components::{add::table::AddColumn, AddClaim, TraceError},
    utils::{pack_values, TreeBuilder},
};

use super::table::{AddTable, PackedAddTableRow};

const N_TRACE_COLUMNS: usize = 15;

pub struct ClaimGenerator {
    pub inputs: AddTable,
}

impl ClaimGenerator {
    pub fn new(inputs: AddTable) -> Self {
        Self { inputs }
    }

    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<AddClaim, TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs
            .table
            .resize(size, *self.inputs.table.first().unwrap());
        let packed_inputs = pack_values(&self.inputs.table);

        let trace = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok(AddClaim::new(log_size))
    }
}

fn write_trace_simd(inputs: Vec<PackedAddTableRow>) -> ComponentTrace<N_TRACE_COLUMNS> {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;

    let mut trace = unsafe { ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size) };

    (trace.par_iter_mut(), inputs.into_par_iter())
        .into_par_iter()
        .for_each(|(mut row, input)| {
            *row[AddColumn::NodeId.index()] = input.node_id;
            *row[AddColumn::LhsId.index()] = input.lhs_id;
            *row[AddColumn::RhsId.index()] = input.rhs_id;
            *row[AddColumn::Idx.index()] = input.idx;
            *row[AddColumn::IsLastIdx.index()] = input.is_last_idx;
            *row[AddColumn::NextNodeId.index()] = input.next_node_id;
            *row[AddColumn::NextLhsId.index()] = input.next_lhs_id;
            *row[AddColumn::NextRhsId.index()] = input.next_rhs_id;
            *row[AddColumn::NextIdx.index()] = input.next_idx;
            *row[AddColumn::Lhs.index()] = input.lhs;
            *row[AddColumn::Rhs.index()] = input.rhs;
            *row[AddColumn::Out.index()] = input.out;
            *row[AddColumn::LhsMult.index()] = input.lhs_mult;
            *row[AddColumn::RhsMult.index()] = input.rhs_mult;
            *row[AddColumn::OutMult.index()] = input.out_mult;
        });

    trace
}
