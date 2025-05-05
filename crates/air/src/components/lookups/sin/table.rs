use std::collections::BTreeSet;

use num_traits::One;
use numerair::Fixed;
use serde::{Deserialize, Serialize};
use stwo_prover::{
    constraint_framework::{logup::LogupTraceGenerator, Relation},
    core::{
        backend::{
            simd::{
                column::BaseColumn,
                m31::{PackedM31, LOG_N_LANES},
                qm31::PackedQM31,
            },
            Column,
        },
        fields::m31::BaseField,
        poly::circle::{CanonicCoset, CircleEvaluation},
    },
};

use crate::{
    components::{
        lookups::Layout, InteractionClaim, SinLookupClaim, TraceColumn, TraceError, TraceEval,
    },
    utils::{calculate_log_size, AtomicMultiplicityColumn},
};

use super::SinLookupElements;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinLookup {
    pub layout: Layout,
    pub data: SinLookupData,
    pub multiplicities: AtomicMultiplicityColumn,
}

impl SinLookup {
    pub fn new(layout: &Layout) -> Self {
        let data = SinLookupData::new(&layout);
        let multiplicities = AtomicMultiplicityColumn::new(1 << layout.log_size);
        Self {
            layout: layout.clone(),
            data,
            multiplicities,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SinLookupData {
    pub col_0: Vec<Fixed>,
    pub col_1: Vec<Fixed>,
}

impl SinLookupData {
    /// Build the two-column sine lookup from a layout.
    pub fn new(layout: &Layout) -> Self {
        let mut uniq = BTreeSet::<i64>::new();
        for range in &layout.ranges {
            uniq.extend(range.0 .0..=range.1 .0);
        }

        let target_len = 1_usize << layout.log_size;
        assert!(
            uniq.len() <= target_len,
            "layout.log_size = {} is too small for {} distinct values",
            layout.log_size,
            uniq.len()
        );

        let mut col_0 = Vec::with_capacity(target_len);
        let mut col_1 = Vec::with_capacity(target_len);

        for &raw in &uniq {
            let x = Fixed(raw);
            col_0.push(x);
            col_1.push(Fixed::from_f64(x.to_f64().sin()));
        }

        // Pad up to 2^log_size
        let pad_0 = *col_0.last().unwrap_or(&Fixed(0));
        let pad_1 = *col_1.last().unwrap_or(&Fixed(0));
        col_0.resize(target_len, pad_0);
        col_1.resize(target_len, pad_1);

        Self { col_0, col_1 }
    }
}

/// Represents the trace for the SinLookup component, containing the required registers for its
/// constraints.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SinLookupTable {
    /// A vector of [`SinLookupTableRow`] representing the table rows.
    pub table: Vec<SinLookupTableRow>,
}

/// Represents a single row of the [`SinLookupTable`]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SinLookupTableRow {
    pub multiplicity: BaseField,
}

impl SinLookupTable {
    /// Creates a new, empty [`SinLookupTable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new row to the Recip Table.
    pub fn add_row(&mut self, row: SinLookupTableRow) {
        self.table.push(row);
    }

    /// Transforms the [`SinLookupTable`] into [`TraceEval`] to be committed
    /// when generating a STARK proof.
    pub fn trace_evaluation(&self) -> Result<(TraceEval, SinLookupClaim), TraceError> {
        let n_rows = self.table.len();
        if n_rows == 0 {
            return Err(TraceError::EmptyTrace);
        }
        // Calculate log size
        let log_size = calculate_log_size(n_rows);

        // Calculate trace size
        let trace_size = 1 << log_size;

        // Create columns
        let mut multiplicity = BaseColumn::zeros(trace_size);

        // Fill columns
        for (vec_row, row) in self.table.iter().enumerate() {
            multiplicity.set(vec_row, row.multiplicity);
        }

        // Create domain
        let domain = CanonicCoset::new(log_size).circle_domain();

        // Create trace
        let mut trace = Vec::with_capacity(SinLookupColumn::count().0);
        trace.push(CircleEvaluation::new(domain, multiplicity));

        assert_eq!(trace.len(), SinLookupColumn::count().0);

        Ok((trace, SinLookupClaim::new(log_size)))
    }
}

/// Enum representing the column indices in the SinLookup trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SinLookupColumn {
    Multiplicity,
}

impl SinLookupColumn {
    /// Returns the index of the column in the SinLookup trace.
    pub const fn index(self) -> usize {
        match self {
            Self::Multiplicity => 0,
        }
    }
}

impl TraceColumn for SinLookupColumn {
    /// Returns the number of columns in the main trace and interaction trace.
    fn count() -> (usize, usize) {
        (1, 1)
    }
}

/// Generates the interaction trace for the SinLookup component using the main trace and node elements.
pub fn interaction_trace_evaluation(
    main_trace_eval: &TraceEval,
    preprocessed: &TraceEval,
    elements: &SinLookupElements,
) -> Result<(TraceEval, InteractionClaim), TraceError> {
    if main_trace_eval.is_empty() {
        return Err(TraceError::EmptyTrace);
    }

    let log_size = main_trace_eval[0].domain.log_size();
    let mut logup_gen = LogupTraceGenerator::new(log_size);

    let mult_col = &main_trace_eval[SinLookupColumn::Multiplicity.index()].data;
    let mut int_col = logup_gen.new_col();
    for row in 0..1 << (log_size - LOG_N_LANES) {
        let mult = mult_col[row];
        let input: PackedM31 = todo!();
        let output: PackedM31 = todo!();

        int_col.write_frac(
            row,
            -PackedQM31::one() * mult,
            elements.combine(&[input, output]),
        );
    }
    int_col.finalize_col();

    let (trace, claimed_sum) = logup_gen.finalize_last();

    Ok((trace, InteractionClaim { claimed_sum }))
}
