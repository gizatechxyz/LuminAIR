use num_traits::One;
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
    components::{InteractionClaim, SinLookupClaim, TraceError},
    preprocessed::SinPreProcessed,
    utils::{pack_values, TreeBuilder},
};

use super::{
    table::{PackedSinLookupTraceTableRow, SinLookupColumn, SinLookupTraceTable, SinLookupTraceTableRow},
    SinLookupElements,
};

pub(crate) const N_TRACE_COLUMNS: usize = 1;

pub struct ClaimGenerator {
    pub inputs: SinLookupTraceTable,
}

impl ClaimGenerator {
    pub fn new(inputs: SinLookupTraceTable) -> Self {
        Self { inputs }
    }

    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(SinLookupClaim, InteractionClaimGenerator), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs.table.resize(size, SinLookupTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            SinLookupClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

fn write_trace_simd(
    inputs: Vec<PackedSinLookupTraceTableRow>,
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
            *row[SinLookupColumn::Multiplicity.index()] = input.multiplicity;

            *lookup_data.multiplicities = input.multiplicity;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    multiplicities: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}

impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        elements: &SinLookupElements,
        lut: &Vec<&SinPreProcessed>,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        let mut col_gen = logup_gen.new_col();
        let lut_col_0 = &lut.get(0).expect("missing sin col 0").evaluation().data;
        let lut_col_1 = &lut.get(1).expect("missing sin col 1").evaluation().data;
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let multiplicity: PackedQM31 = self.lookup_data.multiplicities[row].into();
            let input = lut_col_0[row];
            let output = lut_col_1[row];

            let denom: PackedQM31 = elements.combine(&[input, output]);
            let num: PackedQM31 = -PackedQM31::one() * multiplicity;

            col_gen.write_frac(row, num, denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
