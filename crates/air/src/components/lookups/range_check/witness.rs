use luminair_utils::TraceError;
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
    components::{
        lookups::range_check::{
            table::{
                PackedRangeCheckLookupTraceTableRow, RangeCheckLookupColumn,
                RangeCheckLookupTraceTable, RangeCheckLookupTraceTableRow,
            },
            RangeCheckLookupElements,
        },
        InteractionClaim, RangeCheckLookupClaim,
    },
    preprocessed::RangeCheckPreProcessed,
    utils::{pack_values, TreeBuilder},
};

pub(crate) const N_TRACE_COLUMNS: usize = 1;

pub struct ClaimGenerator<const N: usize> {
    pub inputs: RangeCheckLookupTraceTable,
}

impl<const N: usize> ClaimGenerator<N> {
    pub fn new(inputs: RangeCheckLookupTraceTable) -> Self {
        Self { inputs }
    }

    pub fn write_trace(
        mut self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
    ) -> Result<(RangeCheckLookupClaim, InteractionClaimGenerator<N>), TraceError> {
        let n_rows = self.inputs.table.len();

        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }

        let size = std::cmp::max(n_rows.next_power_of_two(), N_LANES);
        let log_size = size.ilog2();

        self.inputs
            .table
            .resize(size, RangeCheckLookupTraceTableRow::padding());
        let packed_inputs = pack_values(&self.inputs.table);

        let (trace, lookup_data) = write_trace_simd(packed_inputs);

        tree_builder.extend_evals(trace.to_evals());

        Ok((
            RangeCheckLookupClaim::new(log_size),
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        ))
    }
}

fn write_trace_simd(
    inputs: Vec<PackedRangeCheckLookupTraceTableRow>,
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
            *row[RangeCheckLookupColumn::Multiplicity.index()] = input.multiplicity;

            *lookup_data.multiplicities = input.multiplicity;
        });

    (trace, lookup_data)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    multiplicities: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator<const N: usize> {
    log_size: u32,
    lookup_data: LookupData,
}

impl<const N: usize> InteractionClaimGenerator<N> {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        elements: &RangeCheckLookupElements,
        lut: &Vec<&RangeCheckPreProcessed<N>>,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        let mut col_gen = logup_gen.new_col();
        let lut_col_0 = &lut
            .get(0)
            .expect("missing range check col 0")
            .evaluation()
            .data;
        for row in 0..1 << (self.log_size - LOG_N_LANES) {
            let multiplicity: PackedM31 = self.lookup_data.multiplicities[row].into();
            let val = lut_col_0[row];

            let denom: PackedQM31 = elements.combine(&[val]);
            let num: PackedQM31 = -PackedQM31::one() * multiplicity;

            col_gen.write_frac(row, num, denom);
        }
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
