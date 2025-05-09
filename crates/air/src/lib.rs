#![feature(portable_simd, iter_array_chunks, array_chunks, raw_slice_split)]

use ::serde::{Deserialize, Serialize};
use components::{add, AddClaim, InteractionClaim};
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
}

impl LuminairClaim {
    /// Mixes claim data into a Fiat-Shamir channel for proof binding.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        if let Some(ref add) = self.add {
            add.mix_into(channel);
        }
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let mut log_sizes = vec![];

        if let Some(ref add) = self.add {
            log_sizes.push(add.log_sizes());
        }
        TreeVec::concat_cols(log_sizes.into_iter())
    }
}

#[derive(Default)]
pub struct LuminairInteractionClaimGenerator {
    pub add: Option<add::witness::InteractionClaimGenerator>,
}

/// Claim over the sum of interaction columns per system component.
///
/// Used in the logUp protocol with AIR.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LuminairInteractionClaim {
    pub add: Option<InteractionClaim>,
}

impl LuminairInteractionClaim {
    /// Mixes interaction claim data into a Fiat-Shamir channel.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        if let Some(ref add) = self.add {
            add.mix_into(channel);
        }
    }
}
