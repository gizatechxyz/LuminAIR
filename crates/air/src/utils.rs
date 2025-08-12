use std::sync::atomic::{AtomicU32, Ordering};

use num_traits::Zero;
use stwo::core::{
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

#[inline]
pub fn calculate_log_size(max_size: usize) -> u32 {
    ((max_size + (1 << LOG_N_LANES) - 1) >> LOG_N_LANES)
        .next_power_of_two()
        .trailing_zeros()
        + LOG_N_LANES
}

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
        &interaction_claim.rem,
        &interaction_claim.exp2,
        &interaction_claim.exp2_lookup,
        &interaction_claim.log2,
        &interaction_claim.log2_lookup,
        &interaction_claim.less_than,
        &interaction_claim.range_check_lookup,
        &interaction_claim.inputs,
        &interaction_claim.contiguous,
    ] {
        if let Some(ref int_cl) = claim_opt {
            sum += int_cl.claimed_sum.into();
        }
    }

    sum.is_zero()
}

pub fn pack_values<T: Pack>(values: &[T]) -> Vec<T::SimdType> {
    values
        .array_chunks::<N_LANES>()
        .map(|c| T::pack(*c))
        .collect()
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AtomicMultiplicityColumn {
    pub data: Vec<AtomicU32>,
}

impl AtomicMultiplicityColumn {
    pub fn new(size: u32) -> Self {
        Self {
            data: (0..size).map(|_| AtomicU32::new(0)).collect(),
        }
    }

    #[inline]
    pub fn increase_at(&mut self, address: usize) {
        self.data[address].fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Clone for AtomicMultiplicityColumn {
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

pub trait TreeBuilder<B: Backend> {
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan;
}

impl<B: BackendForChannel<MC>, MC: MerkleChannel> TreeBuilder<B>
    for stwo_prover::core::pcs::TreeBuilder<'_, '_, B, MC>
{
    fn extend_evals(
        &mut self,
        columns: impl IntoIterator<Item = CircleEvaluation<B, M31, BitReversedOrder>>,
    ) -> TreeSubspan {
        self.extend_evals(columns)
    }
}
