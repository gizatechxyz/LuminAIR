use std::sync::atomic::{AtomicU32, Ordering};

use num_traits::Zero;
use stwo_prover::core::{
    backend::{
        simd::{
            conversion::Pack,
            m31::{LOG_N_LANES, N_LANES},
            qm31::PackedSecureField,
        },
        Backend, BackendForChannel,
    },
    channel::MerkleChannel,
    fields::m31::M31,
    pcs::TreeSubspan,
    poly::{circle::CircleEvaluation, BitReversedOrder},
};

use crate::LuminairInteractionClaim;

/// Calculates the minimum power-of-two log size for a trace column.
///
/// Given the maximum number of elements (`max_size`) in a column,
/// this function determines the smallest `log_size` such that `2^log_size`
/// accommodates the elements, considering the SIMD vector lane width (`N_LANES`).
/// This ensures the trace fits into the STARK domain correctly.
#[inline]
pub fn calculate_log_size(max_size: usize) -> u32 {
    ((max_size + (1 << LOG_N_LANES) - 1) >> LOG_N_LANES)
        .next_power_of_two()
        .trailing_zeros()
        + LOG_N_LANES
}

/// Verifies the LogUp interaction claim consistency.
///
/// In the LogUp protocol (used for lookups and permutations), the sum of accumulated
/// interaction values across all related columns must equal zero for the proof to be valid.
/// This function sums the `claimed_sum` from all component interaction claims and checks this condition.
/// Returns `true` if the sums balance to zero, `false` otherwise.
pub fn log_sum_valid(interaction_claim: &LuminairInteractionClaim) -> bool {
    let mut sum = PackedSecureField::zero();

    for claim_opt in [
        &interaction_claim.add,
        &interaction_claim.mul,
        &interaction_claim.sum_reduce,
        &interaction_claim.recip,
        &interaction_claim.max_reduce,
        &interaction_claim.sin,
        &interaction_claim.sin_lookup,
        &interaction_claim.sqrt,
    ] {
        if let Some(ref int_cl) = claim_opt {
            sum += int_cl.claimed_sum.into();
        }
    }

    sum.is_zero()
}

/// Packs a slice of elements `T` into SIMD vectors (`T::SimdType`).
///
/// This is a utility for preparing data for efficient processing using SIMD instructions,
/// commonly used within the STWO backend.
pub fn pack_values<T: Pack>(values: &[T]) -> Vec<T::SimdType> {
    values
        .array_chunks::<N_LANES>()
        .map(|c| T::pack(*c))
        .collect()
}

/// A thread-safe column for tracking lookup argument multiplicities.
///
/// This is essential for proving lookup arguments correctly.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AtomicMultiplicityColumn {
    /// Vector of atomic counters, one for each potential lookup value index.
    pub data: Vec<AtomicU32>,
}

impl AtomicMultiplicityColumn {
    /// Creates a new `AtomicMultiplicityColumn` of the specified size, initialized to zeros.
    pub fn new(size: u32) -> Self {
        Self {
            data: (0..size).map(|_| AtomicU32::new(0)).collect(),
        }
    }

    /// Atomically increments the multiplicity count at the given `address` (index).
    #[inline]
    pub fn increase_at(&mut self, address: usize) {
        self.data[address].fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the number of elements tracked by this column.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the column tracks no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Clone for AtomicMultiplicityColumn {
    /// Clones the `AtomicMultiplicityColumn`.
    ///
    /// Creates a new `Vec` of `AtomicU32` by reading the current value of each atomic
    /// in the source vector using `Ordering::Relaxed`.
    fn clone(&self) -> Self {
        let mut new_data = Vec::with_capacity(self.len());

        let values: Vec<u32> = self
            .data
            .iter()
            .map(|atomic| atomic.load(Ordering::Relaxed))
            .collect();

        for val in values {
            new_data.push(AtomicU32::new(val));
        }

        Self { data: new_data }
    }
}

/// A trait abstracting the action of adding evaluation columns to a STWO commitment tree builder.
pub trait TreeBuilder<B: Backend> {
    /// Extends the underlying commitment tree with multiple trace column evaluations.
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan;
}

/// Implements the `TreeBuilder` trait for the STWO prover's concrete `TreeBuilder` type.
/// This simply delegates the call to the underlying `extend_evals` method.
impl<B: BackendForChannel<MC>, MC: MerkleChannel> TreeBuilder<B>
    for stwo_prover::core::pcs::TreeBuilder<'_, '_, B, MC>
{
    /// Delegates to `stwo_prover::core::pcs::TreeBuilder::extend_evals`.
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan {
        self.extend_evals(columns)
    }
}
