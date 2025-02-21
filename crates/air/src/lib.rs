#![feature(trait_upcasting)]

use ::serde::{Deserialize, Serialize};
use components::{AddClaim, InteractionClaim, MulClaim};
use pie::ExecutionResources;
use stwo_prover::constraint_framework::PREPROCESSED_TRACE_IDX;
use stwo_prover::core::{
    channel::Channel, pcs::TreeVec, prover::StarkProof, vcs::ops::MerkleHasher,
};

pub mod components;
pub mod pie;
pub mod serde;
pub mod utils;

/// STARK proof for a Luminair computational graph execution.
///
/// Contains the proof and claims from all proof generation phases.
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairProof<H: MerkleHasher> {
    pub claim: LuminairClaim,
    pub interaction_claim: LuminairInteractionClaim,
    pub proof: StarkProof<H>,
    pub execution_resources: ExecutionResources,
}

/// Claim for system components.
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairClaim {
    pub add: Vec<AddClaim>,
    pub mul: Vec<MulClaim>,
    pub is_first_log_sizes: Vec<u32>,
}

impl LuminairClaim {
    /// Initializes a new claim with specified preprocessed trace log sizes.
    pub fn init(is_first_log_sizes: Vec<u32>) -> Self {
        Self {
            add: vec![],
            mul: vec![],
            is_first_log_sizes,
        }
    }

    /// Mixes claim data into a Fiat-Shamir channel for proof binding.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        self.add.iter().for_each(|c| c.mix_into(channel));
        self.mul.iter().for_each(|c| c.mix_into(channel));
    }

    /// Computes log sizes for all trace types in the claim.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let mut log_sizes = TreeVec::concat_cols(
            self.add
                .iter()
                .map(|c| c.log_sizes())
                .chain(self.mul.iter().map(|c| c.log_sizes()))
                .collect::<Vec<_>>()
                .into_iter(),
        );
        log_sizes[PREPROCESSED_TRACE_IDX] = self.is_first_log_sizes.clone();
        log_sizes
    }
}

/// Claim over the sum of interaction columns per system component.
///
/// Used in the logUp lookup protocol with AIR.
#[derive(Serialize, Deserialize, Debug)]
pub struct LuminairInteractionClaim {
    pub add: Vec<InteractionClaim>,
    pub mul: Vec<InteractionClaim>,
}

impl LuminairInteractionClaim {
    /// Initializes a new interaction claim.
    pub fn init() -> Self {
        Self {
            add: vec![],
            mul: vec![],
        }
    }

    /// Mixes interaction claim data into a Fiat-Shamir channel.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        self.add.iter().for_each(|c| c.mix_into(channel));
        self.mul.iter().for_each(|c| c.mix_into(channel));
    }
}
