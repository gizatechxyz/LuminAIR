use crate::{
    components::{InteractionClaim, MulClaim, NodeElements},
    utils::{pack_values, TreeBuilder},
};
use luminair_utils::TraceError;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo::{
    constraint_framework::{logup::LogupTraceGenerator, Relation},
    core::backend::simd::{
        m31::{PackedM31, LOG_N_LANES, N_LANES},
        qm31::PackedQM31,
        SimdBackend,
    },
};

use super::table::{MulColumn, MulTraceTable, MulTraceTableRow, PackedMulTraceTableRow};

pub(crate) const N_TRACE_COLUMNS: usize = 16;

pub struct ClaimGenerator {
    pub inputs: MulTraceTable,
}

impl ClaimGenerator {
    pub fn new(inputs: MulTraceTable) -> Self {
        Self { inputs }
    }

    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(MulClaim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs.table.resize(size, MulTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            MulClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

fn write_trace_simd(
    inputs: Vec<PackedMulTraceTableRow>,
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
            *row[MulColumn::NodeId.index()] = input.node_id;
            *row[MulColumn::LhsId.index()] = input.lhs_id;
            *row[MulColumn::RhsId.index()] = input.rhs_id;
            *row[MulColumn::Idx.index()] = input.idx;
            *row[MulColumn::IsLastIdx.index()] = input.is_last_idx;
            *row[MulColumn::NextNodeId.index()] = input.next_node_id;
            *row[MulColumn::NextLhsId.index()] = input.next_lhs_id;
            *row[MulColumn::NextRhsId.index()] = input.next_rhs_id;
            *row[MulColumn::NextIdx.index()] = input.next_idx;
            *row[MulColumn::Lhs.index()] = input.lhs;
            *row[MulColumn::Rhs.index()] = input.rhs;
            *row[MulColumn::Out.index()] = input.out;
            *row[MulColumn::Rem.index()] = input.rem;
            *row[MulColumn::LhsMult.index()] = input.lhs_mult;
            *row[MulColumn::RhsMult.index()] = input.rhs_mult;
            *row[MulColumn::OutMult.index()] = input.out_mult;

            *lookup_data.lhs = [input.lhs, input.lhs_id];
            *lookup_data.lhs_mult = input.lhs_mult;
            *lookup_data.rhs = [input.rhs, input.rhs_id];
            *lookup_data.rhs_mult = input.rhs_mult;
            *lookup_data.out = [input.out, input.node_id];
            *lookup_data.out_mult = input.out_mult;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    lhs: Vec<[PackedM31; 2]>,
    lhs_mult: Vec<PackedM31>,
    rhs: Vec<[PackedM31; 2]>,
    rhs_mult: Vec<PackedM31>,
    out: Vec<[PackedM31; 2]>,
    out_mult: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        node_elements: &NodeElements,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.lhs[row];
            let multiplicity = &self.lookup_data.lhs_mult[row];

            let denom: PackedQM31 = node_elements.combine(values);
            col_gen.write_frac(row, (*multiplicity).into(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.rhs[row];
            let multiplicity = &self.lookup_data.rhs_mult[row];

            let denom: PackedQM31 = node_elements.combine(values);
            col_gen.write_frac(row, (*multiplicity).into(), denom);
        }
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.out[row];
            let multiplicity = &self.lookup_data.out_mult[row];

            let denom: PackedQM31 = node_elements.combine(values);
            col_gen.write_frac(row, (*multiplicity).into(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
