use std::sync::atomic::{AtomicU32, Ordering};

use num_traits::Zero;
use stwo_prover::core::backend::simd::{m31::LOG_N_LANES, qm31::PackedSecureField};

use crate::LuminairInteractionClaim;

/// Calculates the logarithmic size of the trace based on the maximum size of the data.
pub fn calculate_log_size(max_size: usize) -> u32 {
    ((max_size + (1 << LOG_N_LANES) - 1) >> LOG_N_LANES)
        .next_power_of_two()
        .trailing_zeros()
        + LOG_N_LANES
}

/// Verifies the validity of the interaction claim by checking if the sum of claimed sums is zero.
pub fn log_sum_valid(interaction_claim: &LuminairInteractionClaim) -> bool {
    let mut sum = PackedSecureField::zero();

    if let Some(ref int_cl) = interaction_claim.add {
        sum += int_cl.claimed_sum.into();
    }
    if let Some(ref int_cl) = interaction_claim.mul {
        sum += int_cl.claimed_sum.into();
    }
    if let Some(ref int_cl) = interaction_claim.sum_reduce {
        sum += int_cl.claimed_sum.into();
    }
    if let Some(ref int_cl) = interaction_claim.recip {
        sum += int_cl.claimed_sum.into();
    }
    if let Some(ref int_cl) = interaction_claim.max_reduce {
        sum += int_cl.claimed_sum.into();
    }
    if let Some(ref int_cl) = interaction_claim.sin {
        sum += int_cl.claimed_sum.into();
    }
    sum.is_zero()
}

/// Generates a vector of logarithmic sizes for the 'is_first' trace columns.
pub fn get_is_first_log_sizes(max_log_size: u32) -> Vec<u32> {
    let padded_max = max_log_size + 2;
    (4..=padded_max).rev().collect()
}

/// A column of multiplicities for lookup arguments. Allow increasing the multiplicity at a give
/// index. This version uses atomic operations to increase the multiplicity and is `Send`.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AtomicMultiplicityColumn {
    data: Vec<AtomicU32>,
}

impl AtomicMultiplicityColumn {
    /// Creates a new `AtomicMultiplicityColumn` with the given size.
    /// The elements are initialized to 0.
    pub fn new(size: usize) -> Self {
        Self {
            data: (0..size as u32).map(|_| AtomicU32::new(0)).collect(),
        }
    }

    pub fn increase_at(&self, address: usize) {
        self.data[address].fetch_add(1, Ordering::Relaxed);
    }
}
