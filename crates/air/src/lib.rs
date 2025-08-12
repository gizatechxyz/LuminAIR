#![feature(portable_simd, iter_array_chunks, array_chunks, raw_slice_split)]

use ::serde::{Deserialize, Serialize};
use components::{
    add, exp2, log2, lookups, max_reduce, mul, recip, rem, sin, sqrt, sum_reduce, AddClaim,
    InteractionClaim, MaxReduceClaim, MulClaim, RecipClaim, RemClaim, SinClaim, SinLookupClaim,
    SqrtClaim, SumReduceClaim,
};
use stwo::core::{channel::Channel, pcs::TreeVec};

use crate::components::{
    contiguous, inputs, less_than, ContiguousClaim, Exp2Claim, Exp2LookupClaim, InputsClaim,
    LessThanClaim, Log2Claim, Log2LookupClaim, RangeCheckLookupClaim,
};

pub mod components;
pub mod pie;
pub mod preprocessed;
pub mod settings;
pub mod utils;

// TODO (@raphaelDkhn): We should parametizing the fixed pointscale.
pub const DEFAULT_FP_SCALE: u32 = 12;
pub const DEFAULT_FP_SCALE_FACTOR: u32 = 1 << DEFAULT_FP_SCALE;

const TWO_POW_31_MINUS_1: u32 = (1u32 << 31) - 1;

/// Main claim structure containing all component claims for LuminAIR
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LuminairClaim {
    pub add: Option<AddClaim>,
    pub mul: Option<MulClaim>,
    pub recip: Option<RecipClaim>,
    pub sin: Option<SinClaim>,
    pub sin_lookup: Option<SinLookupClaim>,
    pub sum_reduce: Option<SumReduceClaim>,
    pub max_reduce: Option<MaxReduceClaim>,
    pub sqrt: Option<SqrtClaim>,
    pub rem: Option<RemClaim>,
    pub exp2: Option<Exp2Claim>,
    pub exp2_lookup: Option<Exp2LookupClaim>,
    pub log2: Option<Log2Claim>,
    pub log2_lookup: Option<Log2LookupClaim>,
    pub less_than: Option<LessThanClaim>,
    pub range_check_lookup: Option<RangeCheckLookupClaim>,
    pub inputs: Option<InputsClaim>,
    pub contiguous: Option<ContiguousClaim>,
}

impl LuminairClaim {
    /// Mixes all component claims into the given channel
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
        if let Some(ref claim) = self.sqrt {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.rem {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.exp2 {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.exp2_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.log2 {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.log2_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.less_than {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.range_check_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.inputs {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.contiguous {
            claim.mix_into(channel);
        }
    }

    /// Returns the log sizes for all component claims
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
        if let Some(ref claim) = self.sqrt {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.rem {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.exp2 {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.exp2_lookup {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.log2 {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.log2_lookup {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.less_than {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.range_check_lookup {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.inputs {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.contiguous {
            log_sizes.push(claim.log_sizes());
        }
        TreeVec::concat_cols(log_sizes.into_iter())
    }
}

/// Generator for interaction claims across all components
#[derive(Default)]
pub struct LuminairInteractionClaimGenerator {
    pub add: Option<add::witness::InteractionClaimGenerator>,
    pub mul: Option<mul::witness::InteractionClaimGenerator>,
    pub recip: Option<recip::witness::InteractionClaimGenerator>,
    pub sin: Option<sin::witness::InteractionClaimGenerator>,
    pub sin_lookup: Option<lookups::sin::witness::InteractionClaimGenerator>,
    pub sum_reduce: Option<sum_reduce::witness::InteractionClaimGenerator>,
    pub max_reduce: Option<max_reduce::witness::InteractionClaimGenerator>,
    pub sqrt: Option<sqrt::witness::InteractionClaimGenerator>,
    pub rem: Option<rem::witness::InteractionClaimGenerator>,
    pub exp2: Option<exp2::witness::InteractionClaimGenerator>,
    pub exp2_lookup: Option<lookups::exp2::witness::InteractionClaimGenerator>,
    pub log2: Option<log2::witness::InteractionClaimGenerator>,
    pub log2_lookup: Option<lookups::log2::witness::InteractionClaimGenerator>,
    pub less_than: Option<less_than::witness::InteractionClaimGenerator>,
    pub range_check_lookup: Option<lookups::range_check::witness::InteractionClaimGenerator<1>>,
    pub inputs: Option<inputs::witness::InteractionClaimGenerator>,
    pub contiguous: Option<contiguous::witness::InteractionClaimGenerator>,
}

/// Collection of interaction claims for all components
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LuminairInteractionClaim {
    pub add: Option<InteractionClaim>,
    pub mul: Option<InteractionClaim>,
    pub recip: Option<InteractionClaim>,
    pub sin: Option<InteractionClaim>,
    pub sin_lookup: Option<InteractionClaim>,
    pub sum_reduce: Option<InteractionClaim>,
    pub max_reduce: Option<InteractionClaim>,
    pub sqrt: Option<InteractionClaim>,
    pub rem: Option<InteractionClaim>,
    pub exp2: Option<InteractionClaim>,
    pub exp2_lookup: Option<InteractionClaim>,
    pub log2: Option<InteractionClaim>,
    pub log2_lookup: Option<InteractionClaim>,
    pub less_than: Option<InteractionClaim>,
    pub range_check_lookup: Option<InteractionClaim>,
    pub inputs: Option<InteractionClaim>,
    pub contiguous: Option<InteractionClaim>,
}

impl LuminairInteractionClaim {
    /// Mixes all interaction claims into the given channel
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
        if let Some(ref claim) = self.sqrt {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.rem {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.exp2 {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.exp2_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.log2 {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.log2_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.less_than {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.range_check_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.inputs {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.contiguous {
            claim.mix_into(channel);
        }
    }
}
