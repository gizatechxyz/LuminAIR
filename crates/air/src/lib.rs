#![feature(portable_simd, iter_array_chunks, array_chunks, raw_slice_split)]

use ::serde::{Deserialize, Serialize};
use components::{
    add, lookups, max_reduce, mul, recip, sin, sum_reduce, AddClaim, InteractionClaim,
    MaxReduceClaim, MulClaim, RecipClaim, SinClaim, SinLookupClaim, SumReduceClaim,
};
use stwo_prover::core::{
    channel::Channel, pcs::TreeVec, prover::StarkProof, vcs::ops::MerkleHasher,
};

pub mod components;
pub mod pie;
pub mod preprocessed;
pub mod utils;

/// STARK proof for a Luminair computational graph execution.
///
/// Contains the proof and claims from all proof generation phases.
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairProof<H: MerkleHasher> {
    pub claim: LuminairClaim,
    pub interaction_claim: LuminairInteractionClaim,
    pub proof: StarkProof<H>,
}

/// Claim for system components.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LuminairClaim {
    pub add: Option<AddClaim>,
    pub mul: Option<MulClaim>,
    pub recip: Option<RecipClaim>,
    pub sin: Option<SinClaim>,
    pub sin_lookup: Option<SinLookupClaim>,
    pub sum_reduce: Option<SumReduceClaim>,
    pub max_reduce: Option<MaxReduceClaim>,
}

impl LuminairClaim {
    /// Mixes claim data into a Fiat-Shamir channel for proof binding.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        if let Some(ref claim) = self.add {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.mul {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.recip {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.sin {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.sin_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.sum_reduce {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.max_reduce {
            claim.mix_into(channel);
        }
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let mut log_sizes = vec![];

        if let Some(ref claim) = self.add {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.mul {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.recip {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.sin {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.sin_lookup {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.sum_reduce {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.max_reduce {
            log_sizes.push(claim.log_sizes());
        }
        TreeVec::concat_cols(log_sizes.into_iter())
    }
}

#[derive(Default)]
pub struct LuminairInteractionClaimGenerator {
    pub add: Option<add::witness::InteractionClaimGenerator>,
    pub mul: Option<mul::witness::InteractionClaimGenerator>,
    pub recip: Option<recip::witness::InteractionClaimGenerator>,
    pub sin: Option<sin::witness::InteractionClaimGenerator>,
    pub sin_lookup: Option<lookups::sin::witness::InteractionClaimGenerator>,
    pub sum_reduce: Option<sum_reduce::witness::InteractionClaimGenerator>,
    pub max_reduce: Option<max_reduce::witness::InteractionClaimGenerator>,
}

/// Claim over the sum of interaction columns per system component.
///
/// Used in the logUp protocol with AIR.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LuminairInteractionClaim {
    pub add: Option<InteractionClaim>,
    pub mul: Option<InteractionClaim>,
    pub recip: Option<InteractionClaim>,
    pub sin: Option<InteractionClaim>,
    pub sin_lookup: Option<InteractionClaim>,
    pub sum_reduce: Option<InteractionClaim>,
    pub max_reduce: Option<InteractionClaim>,
}

impl LuminairInteractionClaim {
    /// Mixes interaction claim data into a Fiat-Shamir channel.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        if let Some(ref claim) = self.add {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.mul {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.recip {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.sin {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.sin_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.sum_reduce {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.max_reduce {
            claim.mix_into(channel);
        }
    }
}
