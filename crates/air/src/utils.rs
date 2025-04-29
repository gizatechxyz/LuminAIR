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
