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

use crate::{
    components::{
        exp2::table::{Exp2Column, Exp2TraceTable, Exp2TraceTableRow, PackedExp2TraceTableRow}, lookups::exp2::Exp2LookupElements, Exp2Claim, InteractionClaim, NodeElements
    },
    utils::{pack_values, TreeBuilder},
};

pub(crate) const N_TRACE_COLUMNS: usize = 12;

pub struct ClaimGenerator {
    pub inputs: Exp2TraceTable,
}

impl ClaimGenerator {
    pub fn new(inputs: Exp2TraceTable) -> Self {
        Self { inputs }
    }

    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(Exp2Claim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs.table.resize(size, Exp2TraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            Exp2Claim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

fn write_trace_simd(
    inputs: Vec<PackedExp2TraceTableRow>,
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
            *row[Exp2Column::NodeId.index()] = input.node_id;
            *row[Exp2Column::InputId.index()] = input.input_id;
            *row[Exp2Column::Idx.index()] = input.idx;
            *row[Exp2Column::IsLastIdx.index()] = input.is_last_idx;
            *row[Exp2Column::NextNodeId.index()] = input.next_node_id;
            *row[Exp2Column::NextInputId.index()] = input.next_input_id;
            *row[Exp2Column::NextIdx.index()] = input.next_idx;
            *row[Exp2Column::Input.index()] = input.input;
            *row[Exp2Column::Out.index()] = input.out;
            *row[Exp2Column::InputMult.index()] = input.input_mult;
            *row[Exp2Column::OutMult.index()] = input.out_mult;
            *row[Exp2Column::LookupMult.index()] = input.lookup_mult;

            *lookup_data.input = [input.input, input.input_id];
            *lookup_data.input_mult = input.input_mult;
            *lookup_data.out = [input.out, input.node_id];
            *lookup_data.out_mult = input.out_mult;
            *lookup_data.lookup_mult = input.lookup_mult;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    input: Vec<[PackedM31; 2]>,
    input_mult: Vec<PackedM31>,
    out: Vec<[PackedM31; 2]>,
    out_mult: Vec<PackedM31>,
    lookup_mult: Vec<PackedM31>,
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
        lookup_elements: &Exp2LookupElements,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let values = &self.lookup_data.input[row];
            let multiplicity = &self.lookup_data.input_mult[row];

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

        let mut col_gen = logup_gen.new_col();
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let input = self.lookup_data.input[row][0];
            let output = self.lookup_data.out[row][0];
            let multiplicity = self.lookup_data.lookup_mult[row];

            let denom: PackedQM31 = lookup_elements.combine(&[input, output]);
            col_gen.write_frac(row, multiplicity.into(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
