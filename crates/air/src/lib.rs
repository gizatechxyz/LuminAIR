#![feature(trait_upcasting)]
#![feature(portable_simd)]

use std::vec;

use ::serde::{Deserialize, Serialize};
use components::{
    AddClaim, InteractionClaim, MaxReduceClaim, MulClaim, RecipClaim, SinClaim, SumReduceClaim,
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
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairClaim {
    pub add: Option<AddClaim>,
    pub mul: Option<MulClaim>,
    pub sum_reduce: Option<SumReduceClaim>,
    pub recip: Option<RecipClaim>,
    pub max_reduce: Option<MaxReduceClaim>,
    pub sin: Option<SinClaim>,
}

impl LuminairClaim {
    /// Initializes a new claim.
    pub fn new() -> Self {
        Self {
            add: None,
            mul: None,
            sum_reduce: None,
            recip: None,
            max_reduce: None,
            sin: None,
        }
    }

    /// Mixes claim data into a Fiat-Shamir channel for proof binding.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        if let Some(ref add) = self.add {
            add.mix_into(channel);
        }
        if let Some(ref mul) = self.mul {
            mul.mix_into(channel);
        }
        if let Some(ref sum_reduce) = self.sum_reduce {
            sum_reduce.mix_into(channel);
        }
        if let Some(ref recip) = self.recip {
            recip.mix_into(channel);
        }
        if let Some(ref max_reduce) = self.max_reduce {
            max_reduce.mix_into(channel);
        }
        if let Some(ref sin) = self.sin {
            sin.mix_into(channel);
        }
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let mut log_sizes = vec![];

        if let Some(ref add) = self.add {
            log_sizes.push(add.log_sizes());
        }
        if let Some(ref mul) = self.mul {
            log_sizes.push(mul.log_sizes());
        }
        if let Some(ref sum_reduce) = self.sum_reduce {
            log_sizes.push(sum_reduce.log_sizes());
        }
        if let Some(ref recip) = self.recip {
            log_sizes.push(recip.log_sizes());
        }
        if let Some(ref max_reduce) = self.max_reduce {
            log_sizes.push(max_reduce.log_sizes());
        }
        if let Some(ref sin) = self.sin {
            log_sizes.push(sin.log_sizes());
        }

        TreeVec::concat_cols(log_sizes.into_iter())
    }
}

/// Claim over the sum of interaction columns per system component.
///
/// Used in the logUp lookup protocol with AIR.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LuminairInteractionClaim {
    pub add: Option<InteractionClaim>,
    pub mul: Option<InteractionClaim>,
    pub sum_reduce: Option<InteractionClaim>,
    pub recip: Option<InteractionClaim>,
    pub max_reduce: Option<InteractionClaim>,
    pub sin: Option<InteractionClaim>,
}

impl LuminairInteractionClaim {
    /// Mixes interaction claim data into a Fiat-Shamir channel.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        if let Some(ref add) = self.add {
            add.mix_into(channel);
        }
        if let Some(ref mul) = self.mul {
            mul.mix_into(channel);
        }
        if let Some(ref sum_reduce) = self.sum_reduce {
            sum_reduce.mix_into(channel);
        }
        if let Some(ref recip) = self.recip {
            recip.mix_into(channel);
        }
        if let Some(ref max_reduce) = self.max_reduce {
            max_reduce.mix_into(channel);
        }
        if let Some(ref sin) = self.sin {
            sin.mix_into(channel);
        }
    }
}
