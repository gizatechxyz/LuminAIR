use luminair_utils::TraceError;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use stwo_air_utils::trace::component_trace::ComponentTrace;
use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
use stwo::prover::backend::simd::{
    m31::{PackedM31, LOG_N_LANES, N_LANES},
    qm31::PackedQM31,
    SimdBackend,
};
use stwo_constraint_framework::{LogupTraceGenerator, Relation};

use crate::{
    components::{
        inputs::table::{
            InputsColumn, InputsTraceTable, InputsTraceTableRow, PackedInputsTraceTableRow,
        }, InputsClaim, InteractionClaim, NodeElements
    },
    utils::{pack_values, TreeBuilder},
};

pub(crate) const N_TRACE_COLUMNS: usize = 7;

pub struct ClaimGenerator {
    pub inputs: InputsTraceTable,
}

impl ClaimGenerator {
    pub fn new(inputs: InputsTraceTable) -> Self {
        Self { inputs }
    }

    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(InputsClaim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs
            .table
            .resize(size, InputsTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            InputsClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

fn write_trace_simd(
    inputs: Vec<PackedInputsTraceTableRow>,
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
            *row[InputsColumn::NodeId.index()] = input.node_id;
            *row[InputsColumn::Idx.index()] = input.idx;
            *row[InputsColumn::IsLastIdx.index()] = input.is_last_idx;
            *row[InputsColumn::NextNodeId.index()] = input.next_node_id;
            *row[InputsColumn::NextIdx.index()] = input.next_idx;
            *row[InputsColumn::Val.index()] = input.val;
            *row[InputsColumn::Multiplicity.index()] = input.multiplicity;

            *lookup_data.val = input.val;
            *lookup_data.node_id = input.node_id;
            *lookup_data.multiplicity = input.multiplicity;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    val: Vec<PackedM31>,
    node_id: Vec<PackedM31>,
    multiplicity: Vec<PackedM31>,
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
            let val = self.lookup_data.val[row];
            let node_id = self.lookup_data.node_id[row];
            let multiplicity = &self.lookup_data.multiplicity[row];

            let denom: PackedQM31 = node_elements.combine(&[val, node_id]);
            col_gen.write_frac(row, (*multiplicity).into(), denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
