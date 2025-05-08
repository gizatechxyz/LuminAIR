use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo_prover::{
    constraint_framework::{logup::LogupTraceGenerator, Relation},
    core::backend::simd::{
        m31::{PackedM31, LOG_N_LANES, N_LANES},
        qm31::PackedQM31,
        SimdBackend,
    },
};

use crate::{
    components::{add::table::AddColumn, AddClaim, InteractionClaim, NodeElements, TraceError},
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
    ) -> Result<(AddClaim, InteractionClaimGenerator), TraceError> {
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

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            AddClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

fn write_trace_simd(
    inputs: Vec<PackedAddTableRow>,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;

    let (mut trace, mut lookup_data) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
        )
    };

    (
        trace.par_iter_mut(),
        lookup_data.par_iter_mut(),
        inputs.into_par_iter(),
    )
        .into_par_iter()
        .for_each(|(mut row, lookup_data, input)| {
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

            *lookup_data.node_elements = [
                input.node_id,
                input.lhs,
                input.lhs_id,
                input.lhs_mult,
                input.rhs,
                input.rhs_id,
                input.rhs_mult,
                input.out,
                input.out_mult,
            ]
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    //node_id,
    //lhs,
    //lhs_id,
    //lhs_mult,
    //rhs,
    //rhs_id,
    //rhs_mult,
    //out,
    //out_mult,
    node_elements: Vec<[PackedM31; 9]>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        nodes_elements: &NodeElements,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.node_elements)
            .into_par_iter()
            .for_each(|(writer, values)| {
                let denom: PackedQM31 = nodes_elements.combine(&[
                    values[AddColumn::Lhs.interaction_index()],
                    values[AddColumn::LhsId.interaction_index()],
                ]);
                writer.write_frac(values[AddColumn::LhsMult.interaction_index()].into(), denom);
            });

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.node_elements)
            .into_par_iter()
            .for_each(|(writer, values)| {
                let denom: PackedQM31 = nodes_elements.combine(&[
                    values[AddColumn::Rhs.interaction_index()],
                    values[AddColumn::RhsId.interaction_index()],
                ]);
                writer.write_frac(values[AddColumn::RhsMult.interaction_index()].into(), denom);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.node_elements)
            .into_par_iter()
            .for_each(|(writer, values)| {
                let denom: PackedQM31 = nodes_elements.combine(&[
                    values[AddColumn::Out.interaction_index()],
                    values[AddColumn::NodeId.interaction_index()],
                ]);
                writer.write_frac(values[AddColumn::OutMult.interaction_index()].into(), denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
