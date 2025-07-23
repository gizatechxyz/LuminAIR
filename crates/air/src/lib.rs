#![feature(portable_simd, iter_array_chunks, array_chunks, raw_slice_split)]

use ::serde::{Deserialize, Serialize};
use components::{
    add, exp2, lookups, max_reduce, mul, recip, rem, sin, sqrt, sum_reduce, AddClaim,
    InteractionClaim, MaxReduceClaim, MulClaim, RecipClaim, RemClaim, SinClaim, SinLookupClaim,
    SqrtClaim, SumReduceClaim,
};
use stwo_prover::core::{channel::Channel, pcs::TreeVec};

use crate::components::{
    less_than, Exp2Claim, Exp2LookupClaim, LessThanClaim, RangeCheckLookupClaim,
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

/// Container for claims related to the main execution trace of LuminAIR components.
///
/// Each field corresponds to a specific AIR component (like Add, Mul, Sin) and holds
/// the claim generated for that component's trace segment, if present in the computation.
/// These claims typically include commitments to the component's trace columns.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LuminairClaim {
    /// Claim for the Add component's trace.
    pub add: Option<AddClaim>,
    /// Claim for the Mul component's trace.
    pub mul: Option<MulClaim>,
    /// Claim for the Recip component's trace.
    pub recip: Option<RecipClaim>,
    /// Claim for the Sin component's trace.
    pub sin: Option<SinClaim>,
    /// Claim for the Sin Lookup component's trace.
    pub sin_lookup: Option<SinLookupClaim>,
    /// Claim for the SumReduce component's trace.
    pub sum_reduce: Option<SumReduceClaim>,
    /// Claim for the MaxReduce component's trace.
    pub max_reduce: Option<MaxReduceClaim>,
    /// Claim for the Sqrt component's trace.
    pub sqrt: Option<SqrtClaim>,
    /// Claim for the Rem component's trace.
    pub rem: Option<RemClaim>,

    /// Claim for the Exp2 component's trace.
    pub exp2: Option<Exp2Claim>,
    /// Claim for the Exp2 Lookup component's trace.
    pub exp2_lookup: Option<Exp2LookupClaim>,
    /// Claim for the LessThan component's trace.
    pub less_than: Option<LessThanClaim>,
    /// Claim for the LessThan Lookup component's trace.
    pub range_check_lookup: Option<RangeCheckLookupClaim>,
}

impl LuminairClaim {
    /// Mixes all component claims into the provided Fiat-Shamir channel.
    ///
    /// This is crucial for binding the commitments in the claims to the rest of the proof.
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
        if let Some(ref claim) = self.exp2 {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.exp2_lookup {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.less_than {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.range_check_lookup {
            claim.mix_into(channel);
        }
    }

    /// Aggregates the log-sizes (dimensions) of all present component trace segments.
    ///
    /// This information is needed by the prover and verifier to configure the polynomial commitment scheme.
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
        if let Some(ref claim) = self.exp2 {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.exp2_lookup {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.less_than {
            log_sizes.push(claim.log_sizes());
        }
        if let Some(ref claim) = self.range_check_lookup {
            log_sizes.push(claim.log_sizes());
        }
        TreeVec::concat_cols(log_sizes.into_iter())
    }
}

/// Container for interaction claim generators for each LuminAIR component.
///
/// During proof generation, after the main trace is committed and interaction randomness
/// is drawn from the channel, these generators are used to compute the interaction trace columns
/// and produce the corresponding `LuminairInteractionClaim`.
#[derive(Default)]
pub struct LuminairInteractionClaimGenerator {
    /// Generator for the Add component's interaction claim.
    pub add: Option<add::witness::InteractionClaimGenerator>,
    /// Generator for the Mul component's interaction claim.
    pub mul: Option<mul::witness::InteractionClaimGenerator>,
    /// Generator for the Recip component's interaction claim.
    pub recip: Option<recip::witness::InteractionClaimGenerator>,
    /// Generator for the Sin component's interaction claim.
    pub sin: Option<sin::witness::InteractionClaimGenerator>,
    /// Generator for the Sin Lookup component's interaction claim.
    pub sin_lookup: Option<lookups::sin::witness::InteractionClaimGenerator>,
    /// Generator for the SumReduce component's interaction claim.
    pub sum_reduce: Option<sum_reduce::witness::InteractionClaimGenerator>,
    /// Generator for the MaxReduce component's interaction claim.
    pub max_reduce: Option<max_reduce::witness::InteractionClaimGenerator>,
    /// Generator for the Sqrt component's interaction claim.
    pub sqrt: Option<sqrt::witness::InteractionClaimGenerator>,
    /// Generator for the Rem component's interaction claim.
    pub rem: Option<rem::witness::InteractionClaimGenerator>,
    /// Generator for the Exp2 component's interaction claim.
    pub exp2: Option<exp2::witness::InteractionClaimGenerator>,
    /// Generator for the Exp2 Lookup component's interaction claim.
    pub exp2_lookup: Option<lookups::exp2::witness::InteractionClaimGenerator>,
    /// Generator for the LessThan component's interaction claim.
    pub less_than: Option<less_than::witness::InteractionClaimGenerator>,
    /// Generator for the RangeCheck Lookup component's interaction claim.
    pub range_check_lookup: Option<lookups::range_check::witness::InteractionClaimGenerator<1>>,
}

/// Container for claims related to the interaction trace of LuminAIR components.
///
/// These claims typically arise from LogUp protocol, representing accumulated values
/// across different trace segments after incorporating randomness drawn from the channel.
/// They are essential for linking different parts of the trace (e.g., main trace, lookups) and for ensuring
/// the integrity of the dataflow.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LuminairInteractionClaim {
    /// Interaction claim for the Add component.
    pub add: Option<InteractionClaim>,
    /// Interaction claim for the Mul component.
    pub mul: Option<InteractionClaim>,
    /// Interaction claim for the Recip component.
    pub recip: Option<InteractionClaim>,
    /// Interaction claim for the Sin component.
    pub sin: Option<InteractionClaim>,
    /// Interaction claim for the Sin Lookup component.
    pub sin_lookup: Option<InteractionClaim>,
    /// Interaction claim for the SumReduce component.
    pub sum_reduce: Option<InteractionClaim>,
    /// Interaction claim for the MaxReduce component.
    pub max_reduce: Option<InteractionClaim>,
    /// Interaction claim for the Sqrt component.
    pub sqrt: Option<InteractionClaim>,
    /// Interaction claim for the Rem component.
    pub rem: Option<InteractionClaim>,
    /// Interaction claim for the Exp2 component.
    pub exp2: Option<InteractionClaim>,
    /// Interaction claim for the Exp2 Lookup component.
    pub exp2_lookup: Option<InteractionClaim>,
    /// Interaction claim for the LessThan component.
    pub less_than: Option<InteractionClaim>,
    /// Interaction claim for the RangeCheck Lookup component.
    pub range_check_lookup: Option<InteractionClaim>,
}

impl LuminairInteractionClaim {
    /// Mixes all component interaction claims into the provided Fiat-Shamir channel.
    /// This binds the interaction phase commitments and values into the proof transcript.
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
        if let Some(ref claim) = self.less_than {
            claim.mix_into(channel);
        }
        if let Some(ref claim) = self.range_check_lookup {
            claim.mix_into(channel);
        }
    }
}
